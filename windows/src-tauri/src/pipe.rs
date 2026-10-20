// Relay server for coucou-hook.
//
// Windows: the named pipe `\\.\pipe\coucou-<sid>`, one instance per connection.
// Linux: the Unix socket `$XDG_RUNTIME_DIR/coucou.sock`. Every hook event is
// forwarded to the island as a `hook` event. `PermissionRequest` is the only one
// that keeps its connection open: it waits for the island's decision and writes
// it back on the same connection, which is how approving from the island works.
//
// Claude Code is never blocked by us. Three things guarantee it:
//   * coucou-hook gives the connection 300 ms and exits cleanly if we are closed;
//   * we only wait for a human once the island has *confirmed* the card is on
//     screen, so a paused island or a webview that is not listening costs a few
//     hundred milliseconds, not two minutes;
//   * whatever happens we drop the connection after the decision timeout, and
//     the terminal takes over.
//
// What we write back is the bare word `allow` or `deny` — or, when the request
// was Claude Code asking a question, `{"answers":{…}}` with what was picked on
// the island. Turning either into the documented hookSpecificOutput JSON is
// coucou-hook's job, so the wire format Claude Code expects lives in exactly
// one place.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::mpsc;

use crate::island::WINDOW_LABEL;
use crate::log;
use crate::session_window;

/// Slightly under coucou-hook's own 110 s wait, so we always answer first.
const DECISION_TIMEOUT: Duration = Duration::from_secs(108);
/// How long the island gets to say "the card is up". This is the whole of B4:
/// without it, an island that is paused, hidden behind a crashed webview or
/// simply not listening would leave Claude Code staring at a prompt nobody can
/// see for nearly two minutes.
const ACK_TIMEOUT: Duration = Duration::from_millis(800);
const MAX_PAYLOAD: usize = 1 << 20;

/// What the island can say about a permission request.
pub enum Reply {
    /// The card is on screen and a human can act on it.
    Ack,
    /// A human clicked: `allow` or `deny`.
    Decision(String),
    /// Nobody can act on it — paused, or another request already holds the card.
    Decline,
}

/// Permission requests the island has been told about.
struct PendingRequest {
    sender:mpsc::Sender<Reply>, acknowledged:bool,
    session_id:String, pet_id:String,
    ack_deadline:Instant, deadline:Instant,
    /// Set only by a scoped native card receipt, never by the old blind ACK.
    card_presented:bool,
}
impl PendingRequest {
    fn new(sender:mpsc::Sender<Reply>, session_id:String, pet_id:String, now:Instant)->Self {
        Self {sender,acknowledged:false,session_id,pet_id,
            ack_deadline:now+ACK_TIMEOUT,deadline:now+DECISION_TIMEOUT,card_presented:false}
    }
    fn live(&self,now:Instant)->bool {
        now < self.deadline && (self.acknowledged || now < self.ack_deadline)
    }
}
#[derive(Default)]
pub struct Pending(Mutex<HashMap<String, PendingRequest>>);

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// `\\.\pipe\coucou-<sid>` — must match coucou-hook's `pipe_path()` exactly.
#[cfg(windows)]
pub fn pipe_name() -> String {
    let key = crate::platform::current_user_sid()
        .unwrap_or_else(|| std::env::var("USERNAME").unwrap_or_else(|_| "user".into()));
    format!(r"\\.\pipe\coucou-{key}")
}

#[cfg(windows)]
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let name = pipe_name();
        // first_pipe_instance also means we refuse to join a pipe somebody else
        // already owns under our name, rather than serving on top of it.
        let mut server = match ServerOptions::new().first_pipe_instance(true).create(&name) {
            Ok(s) => s,
            Err(err) => {
                log::line(format!("cannot open the relay pipe: {err}"));
                return;
            }
        };
        loop {
            if server.connect().await.is_err() {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }
            // Hand the connected instance to a task and listen on a fresh one.
            let next = match ServerOptions::new().create(&name) {
                Ok(s) => s,
                Err(err) => {
                    log::line(format!("cannot reopen the relay pipe: {err}"));
                    return;
                }
            };
            let connected = std::mem::replace(&mut server, next);
            let app = app.clone();
            tauri::async_runtime::spawn(async move { handle(app, connected).await });
        }
    });
}

#[cfg(target_os = "linux")]
pub fn start(app: AppHandle) {
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;

    tauri::async_runtime::spawn(async move {
        let Some(path) = crate::platform::relay_socket_path() else {
            log::line("no private runtime directory ($XDG_RUNTIME_DIR) — Claude Code hooks are inactive");
            return;
        };
        // A socket file left behind by a crash answers nothing and can go. One
        // that answers belongs to a Coucou that is still running: like
        // first_pipe_instance on Windows, we refuse to serve on top of it.
        if path.exists() {
            if std::os::unix::net::UnixStream::connect(&path).is_ok() {
                log::line("another Coucou already serves the relay socket");
                return;
            }
            let _ = std::fs::remove_file(&path);
        }
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(err) => {
                log::line(format!("cannot open the relay socket: {err}"));
                return;
            }
        };
        // The runtime directory is already 0700; this is belt and braces.
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        let uid = unsafe { libc::getuid() };
        loop {
            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }
            };
            // Only the relay run by our own user may drive the island.
            if !matches!(stream.peer_cred(), Ok(c) if c.uid() == uid) {
                log::line("refused a relay connection from another user");
                continue;
            }
            let app = app.clone();
            tauri::async_runtime::spawn(async move { handle(app, stream).await });
        }
    });
}

/// One accepted relay connection, whatever carries it.
trait Relay: AsyncRead + AsyncWrite + Unpin {
    /// Ends the conversation once everything has been written.
    fn finish(&mut self) {}
    /// The relay process on the other end, where the OS says.
    fn client_pid(&self) -> Option<u32> {
        None
    }
    /// Named pipes have no writer-only half-close. Unix streams override this
    /// because EOF alone does not mean their reply receiver has disappeared.
    fn peer_disconnected(&self)->bool {true}
}

#[cfg(windows)]
impl Relay for NamedPipeServer {
    fn finish(&mut self) {
        let _ = self.disconnect();
    }
    fn client_pid(&self) -> Option<u32> {
        use std::os::windows::io::AsRawHandle;
        crate::platform::pipe_client_pid(self.as_raw_handle())
    }
}

/// Dropping the stream closes it; the relay reads up to our newline first.
#[cfg(target_os = "linux")]
impl Relay for tokio::net::UnixStream {
    fn peer_disconnected(&self)->bool {
        use std::os::fd::AsRawFd;
        let mut poll=libc::pollfd{fd:self.as_raw_fd(),events:0,revents:0};
        let status=unsafe{libc::poll(&mut poll,1,0)};
        status>0 && poll.revents&(libc::POLLHUP|libc::POLLERR|libc::POLLNVAL)!=0
    }
    /// SO_PEERCRED: the process that connected, as the kernel saw it.
    fn client_pid(&self) -> Option<u32> {
        let pid = self.peer_cred().ok()?.pid()?;
        u32::try_from(pid).ok().filter(|pid| *pid > 1)
    }
}

async fn handle(app: AppHandle, mut pipe: impl Relay) {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match pipe.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if buf.contains(&b'\n') || buf.len() > MAX_PAYLOAD {
                    break;
                }
            }
            Err(_) => return,
        }
    }
    let line = match buf.iter().position(|b| *b == b'\n') {
        Some(i) => &buf[..i],
        None => &buf[..],
    };
    let Ok(mut payload) = serde_json::from_slice::<Value>(line) else { return };
    if !payload.is_object() {
        return;
    }

    // Never trust a relay-supplied pet owner. The app's live session record is
    // the authority, so unrelated CLI sessions cannot be mistaken for our jobs.
    payload.as_object_mut().unwrap().remove("coucou_pet_provider");
    let backend=payload["coucou_agent"].as_str().unwrap_or("").to_string();
    let session=payload["session_id"].as_str().unwrap_or("").to_string();
    let owner=app.state::<crate::chat::Chat>().hook_owner(&backend,&session);
    let gate=payload["coucou_assistant_gate"]==true;
    if gate && (backend!="antigravity" || owner.is_none()) {
        let _=pipe.write_all(b"unowned\n").await;let _=pipe.flush().await;pipe.finish();return;
    }
    if let Some(owner)=&owner {payload["coucou_pet_provider"]=json!(owner);}

    let event = payload
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    note_session_window(&pipe, &payload, &event);
    // Counts for the weekly recap — never the command, path or prompt itself.
    crate::recap::observe(&app, &payload);

    if event != "PermissionRequest" {
        // The status line relay calls in with every Claude Code update: not log-worthy.
        if event != "StatusLine" {
            log::line(format!("hook {event}"));
        }
        let _ = app.emit_to(WINDOW_LABEL, "hook", payload);
        pipe.finish();
        return;
    }

    if gate {
        if !app.state::<crate::chat::Chat>().hook_activity(&backend,&session,"approval") {
            let _=pipe.write_all(b"deny\n").await;let _=pipe.flush().await;pipe.finish();return;
        }
        let _=app.emit("assistant-tasks-changed",());
    }

    let id = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed));
    let (tx, mut rx) = mpsc::channel::<Reply>(4);
    let started_at=Instant::now();
    {
        let pending = app.state::<Pending>();
        pending.0.lock().unwrap().insert(id.clone(), PendingRequest::new(tx,session.clone(),request_pet(&payload),started_at));
    }
    crate::recap::note_request(&app, &id, &payload);
    payload["request_id"] = json!(id);
    log::line(format!("hook PermissionRequest id={id}"));
    let _ = app.emit_to(WINDOW_LABEL, "hook", payload);

    // The human cannot answer a relay that has already gone away. This is
    // common to every supported hook, including questions: no orphaned card.
    let outcome=until_relay_closed(&mut pipe,wait_for_decision(&id,&mut rx,started_at)).await;
    if matches!(&outcome,DecisionOutcome::Handoff("relay-closed")) {
        log::line(format!("hook id={id} handed off reason=relay-closed"));
    }
    app.state::<Pending>().0.lock().unwrap().remove(&id);
    let decision=match outcome{
        DecisionOutcome::Decision(decision)=>Some(decision),
        DecisionOutcome::Handoff(reason)=>{
            crate::recap::forget_request(&app,&id);
            // The UI no longer owns a respondible card. The native CLI may
            // still need the human in its terminal; this does not resolve it.
            if !gate {let _=app.emit_to(WINDOW_LABEL,"approval-handoff",handoff_payload(&id,&session,&backend,owner.as_deref(),reason));}
            None
        }
    };

    if gate {
        app.state::<crate::chat::Chat>().hook_activity(&backend,&session,"working");
        let _=app.emit("assistant-tasks-changed",());
        if decision.is_none() {
            let _=app.emit_to(WINDOW_LABEL,"hook",json!({"hook_event_name":"Interrupt","session_id":session,"coucou_agent":backend,"coucou_pet_provider":owner}));
        }
    }

    // No decision: say nothing at all. coucou-hook then writes nothing to stdout
    // and Claude Code asks in the terminal, exactly as if Coucou were closed.
    if let Some(d) = decision {
        let _ = pipe.write_all(format!("{d}\n").as_bytes()).await;
        let _ = pipe.flush().await;
    }
    pipe.finish();
}

fn handoff_payload(request_id:&str,session_id:&str,agent:&str,provider:Option<&str>,reason:&str)->Value{
    json!({"requestId":request_id,"sessionId":session_id,"agent":agent,"provider":provider,"reason":reason})
}

/// Finds, once per session, the window it runs in — see session_window.rs.
///
/// Only while the session is unknown, so the process snapshot is not taken on
/// every event. The relay must still be running for its parents to be found:
/// a permission request always is (it waits for us), a quick event may already
/// have exited, and then a later event of the session tries again.
fn note_session_window(pipe: &impl Relay, payload: &Value, event: &str) {
    let Some(session) = payload.get("session_id").and_then(Value::as_str) else { return };
    if event == "SessionEnd" {
        session_window::forget(session);
        return;
    }
    if session_window::known(session) {
        return;
    }
    let Some(relay) = pipe.client_pid() else { return };
    let ancestors = crate::platform::process_ancestors(relay);
    // No ancestors: the relay was already gone, so try again next time. Some,
    // but none with a window (a classic console): settled, VS Code it is.
    if !ancestors.is_empty() {
        session_window::remember(session, crate::platform::window_owners(&ancestors));
    }
}

/// A short acknowledgement wait inside one fixed total decision budget.
enum DecisionOutcome {
    Decision(String),
    Handoff(&'static str),
}

async fn relay_closed(relay:&mut impl Relay) {
    let mut extra=[0u8;1];
    match relay.read(&mut extra).await {
        Ok(0)=>{
            // A client may finish its request writer while legitimately keeping
            // its receiver open for the answer. Await actual HUP, not EOF.
            while !relay.peer_disconnected(){tokio::time::sleep(Duration::from_millis(25)).await;}
        }
        // Extra input violates the one-line request contract; errors cannot
        // establish a respondible connection either. Neither creates a grant.
        _=>{},
    }
}
#[cfg(test)]impl Relay for tokio::io::DuplexStream {}
async fn until_relay_closed<R:Relay,F:std::future::Future<Output=DecisionOutcome>>(relay:&mut R,decision:F)->DecisionOutcome {
    // Actual closure/invalid extra input wins over a simultaneously ready reply.
    tokio::select! {biased;_=relay_closed(relay)=>DecisionOutcome::Handoff("relay-closed"),answer=decision=>answer}
}

async fn wait_for_decision(id: &str, rx: &mut mpsc::Receiver<Reply>, started_at:Instant) -> DecisionOutcome {
    let outcome = wait_for_decision_from(id, rx, ACK_TIMEOUT, DECISION_TIMEOUT,tokio::time::Instant::from_std(started_at)).await;
    // Only this production wrapper logs. The state machine below is RAM-only,
    // so channel fixtures never write the account's log or inspect an AppHandle.
    match &outcome {
        DecisionOutcome::Decision(decision) => {
            // Retain the existing public categories, never arbitrary reply text.
            let category = match loggable(decision) {
                "allow" => "allow",
                "deny" => "deny",
                "a question" => "a question",
                _ => "a decision",
            };
            log::line(format!("hook id={id} answered {category}"));
        }
        DecisionOutcome::Handoff(reason) => {
            log::line(format!("hook id={id} handed off reason={reason}"));
        }
    }
    outcome
}

#[cfg(test)]
async fn wait_for_decision_with_limits(id:&str,rx:&mut mpsc::Receiver<Reply>,ack_timeout:Duration,total_timeout:Duration)->DecisionOutcome {
    wait_for_decision_from(id,rx,ack_timeout,total_timeout,tokio::time::Instant::now()).await
}
async fn wait_for_decision_from(
    _id: &str,
    rx: &mut mpsc::Receiver<Reply>,
    ack_timeout: Duration,
    total_timeout: Duration,
    start:tokio::time::Instant,
) -> DecisionOutcome {
    use tokio::time::{timeout_at, Instant};

    // The total budget starts on entry, including the acknowledgement wait.
    // Neither the first acknowledgement nor duplicates can extend it.
    let total_deadline = start + total_timeout;
    let ack_deadline = total_deadline.min(start + ack_timeout);
    let mut acknowledged = false;

    loop {
        let deadline = if acknowledged { total_deadline } else { ack_deadline };
        let timeout_reason = if acknowledged { "timeout" } else { "unacknowledged" };
        // A permanently ready receiver must not starve the absolute deadline.
        if Instant::now() >= deadline {
            return DecisionOutcome::Handoff(timeout_reason);
        }
        let reply = match timeout_at(deadline, rx.recv()).await {
            Ok(reply) => reply,
            Err(_) => return DecisionOutcome::Handoff(timeout_reason),
        };
        // timeout_at may poll an already-ready receive before its elapsed timer.
        // Enforce the same strict deadline at that boundary as before the recv.
        if Instant::now() >= deadline {
            return DecisionOutcome::Handoff(timeout_reason);
        }
        match reply {
            Some(Reply::Decision(decision)) => return DecisionOutcome::Decision(decision),
            Some(Reply::Ack) => acknowledged = true,
            Some(Reply::Decline) => {
                return DecisionOutcome::Handoff(if acknowledged { "released" } else { "not-shown" });
            }
            None => return DecisionOutcome::Handoff("closed"),
        }
    }
}


/// The log says a question was answered, never with what.
fn loggable(decision: &str) -> &str {
    if decision.starts_with('{') { "a question" } else { decision }
}

fn dispatch(map:&mut HashMap<String,PendingRequest>,request_id:&str,reply:Reply)->bool{
    dispatch_at(map,request_id,reply,Instant::now())
}
fn dispatch_at(map:&mut HashMap<String,PendingRequest>,request_id:&str,reply:Reply,now:Instant)->bool{
    // A delayed UI callback cannot beat expiry merely because async cleanup
    // has not removed the record yet. Duplicate ACKs never renew either clock.
    if !map.get(request_id).is_some_and(|request|request.live(now)){return false;}
    if matches!(reply,Reply::Ack){
        let Some(request)=map.get_mut(request_id)else{return false};
        // Do not let re-renders fill the channel and displace a human decision.
        if request.acknowledged{return true;}
        if request.sender.try_send(reply).is_ok(){request.acknowledged=true;true}else{false}
    }else{
        map.remove(request_id).is_some_and(|request|request.sender.try_send(reply).is_ok())
    }
}

fn send(app: &AppHandle, request_id: &str, reply: Reply) {
    let delivered = {
        let pending = app.state::<Pending>();
        let mut map = pending.0.lock().unwrap();
        dispatch(&mut map,request_id,reply)
    };
    if !delivered{log::line(format!("reply for id={request_id} — no pending request"));}
}

/// The island has the card on screen; the long wait may begin.
pub fn acknowledge(app: &AppHandle, request_id: &str) {
    // On Linux the mapped controller can be empty. The old receipt proves only
    // that a listener exists, not that the user has a respondible card.
    #[cfg(not(target_os="linux"))]
    send(app, request_id, Reply::Ack);
    #[cfg(target_os="linux")]
    {let _=(app,request_id);}
}

/// Match the frontend's hook routing, using only the owner inserted by handle
/// after it discards relay-supplied owner metadata. Unknown agent names keep
/// their own pill; they do not gain decision support from this routing helper.
fn request_pet(payload:&Value)->String {
    if let Some(pet)=payload["coucou_pet_provider"].as_str().and_then(crate::pet_config::pet_id){return pet.into();}
    if let Some(agent)=payload["coucou_agent"].as_str().filter(|a|!a.is_empty()&&a.len()<=24&&*a!="claude"&&a.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'-')) {
        return format!("agent_{agent}");
    }
    if payload["term_editor"]=="cursor"{"agent_cursor".into()}else{"integration_claude".into()}
}
fn card_surface_ready(label:&str,visible:bool,collapsed:bool,rect:crate::island::IslandRect)->bool {
    label==WINDOW_LABEL && visible && !collapsed &&
        [rect.x,rect.y,rect.w,rect.h].iter().all(|v|v.is_finite()) &&
        rect.x>=0.0 && rect.y>=0.0 && rect.w>0.0 && rect.h>0.0
}
fn native_card_ready(app:&AppHandle,window:&tauri::WebviewWindow)->bool {
    let shared=app.state::<crate::Shared>();
    let Ok(rect)=shared.gate.rect.try_lock()else{return false;};
    let rect=*rect;
    card_surface_ready(window.label(),window.is_visible().unwrap_or(false),
        shared.gate.collapsed.load(Ordering::Acquire),rect)
}
fn present_at(map:&mut HashMap<String,PendingRequest>,id:&str,session:&str,pet:&str,ready:bool,now:Instant)->bool {
    if !ready || !map.get(id).is_some_and(|r|r.session_id==session&&r.pet_id==pet&&r.live(now)){return false;}
    if !dispatch_at(map,id,Reply::Ack,now){return false;}
    if let Some(request)=map.get_mut(id){request.card_presented=true;}
    true
}

/// A receipt for an already-rendered card, not an instruction to reveal or
/// decide it. The caller and native input surface are checked independently of
/// the frontend's selected pet. This applies to every legacy hook agent.
/// All native surface reads and receipt/decision dispatch land together on
/// GTK. No Pending/geometry mutex is held across scheduling or any await.
async fn on_card_thread(app:AppHandle,window:tauri::WebviewWindow,action:impl FnOnce(&AppHandle,&tauri::WebviewWindow)->bool+Send+'static)->bool {
    let current=app.clone();let(tx,rx)=tokio::sync::oneshot::channel();
    if app.run_on_main_thread(move||{let _=tx.send(action(&current,&window));}).is_err(){return false;}
    rx.await.unwrap_or(false)
}
#[tauri::command]
pub async fn approval_presented(window:tauri::WebviewWindow,app:AppHandle,request_id:String,session_id:String,pet_id:String)->bool {
    on_card_thread(app,window,move|app,window|{
        let ready=native_card_ready(app,window);
        let pending=app.state::<Pending>();
        let shown=pending.0.try_lock().ok().is_some_and(|mut requests|
            present_at(&mut requests,&request_id,&session_id,&pet_id,ready,Instant::now()));
        // No command, prompt, session, cwd, answers, or caller-supplied IDs.
        log::line(if shown{"relay card receipt accepted"}else{"relay card receipt rejected"});
        shown
    }).await
}

fn card_dispatch_at(map:&mut HashMap<String,PendingRequest>,id:&str,reply:Reply,ready:bool,now:Instant)->bool {
    if !ready || !map.get(id).is_some_and(|r|r.card_presented&&r.acknowledged&&r.live(now)){return false;}
    dispatch_at(map,id,reply,now)
}

/// Card decisions require their still-live presentation. Phone answers keep
/// their separate authenticated native hub path and do not use this function.
pub async fn answer_from_card(app:AppHandle,window:tauri::WebviewWindow,request_id:String,decision:String)->bool {
    let word=match decision.as_str(){"allow"=>"allow","deny"=>"deny",_=>return false};
    on_card_thread(app,window,move|app,window|{
        let ready=native_card_ready(app,window);let pending=app.state::<Pending>();
        let accepted=pending.0.try_lock().ok().is_some_and(|mut requests|
            card_dispatch_at(&mut requests,&request_id,Reply::Decision(word.into()),ready,Instant::now()));
        log::line(if accepted{"relay card decision accepted"}else{"relay card decision rejected"});
        accepted
    }).await
}
pub async fn answer_question_from_card(app:AppHandle,window:tauri::WebviewWindow,request_id:String,answers:std::collections::HashMap<String,Value>)->bool {
    on_card_thread(app,window,move|app,window|{
        let ready=native_card_ready(app,window);let pending=app.state::<Pending>();
        let accepted=pending.0.try_lock().ok().is_some_and(|mut requests|
            card_dispatch_at(&mut requests,&request_id,Reply::Decision(json!({"answers":answers}).to_string()),ready,Instant::now()));
        log::line(if accepted{"relay card answer accepted"}else{"relay card answer rejected"});
        accepted
    }).await
}

/// Nobody can act on this one — paused, or another card already holds the view.
pub fn decline(app: &AppHandle, request_id: &str) {
    log::line(format!("decline id={request_id}"));
    send(app, request_id, Reply::Decline);
}

/// Called by the island's Allow / Deny buttons. Only ever a bare word: turning
/// it into Claude Code's JSON is coucou-hook's job.
pub fn answer(app: &AppHandle, request_id: &str, decision: &str) {
    let word = match decision {
        "allow" | "always" => "allow",
        _ => "deny",
    };
    log::line(format!("decision id={request_id} {word}"));
    send(app, request_id, Reply::Decision(word.to_string()));
}

/// Called when an option is picked for a question Claude Code asked. `answers`
/// maps each question's text to the chosen label, which is the shape
/// AskUserQuestion takes them in.
pub fn answer_question(app: &AppHandle, request_id: &str, answers: &HashMap<String, serde_json::Value>) {
    log::line(format!("decision id={request_id} answered a question"));
    // One line: the relay reads up to the first newline.
    let line = json!({ "answers": answers }).to_string();
    send(app, request_id, Reply::Decision(line));
}

// Local channel fixtures; no UI, relay, file or model.
#[cfg(test)]
mod decision_wait_tests {
    use super::{wait_for_decision_with_limits, DecisionOutcome, Reply};
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use std::time::Duration;
    use tokio::sync::mpsc;

    const OUTER: Duration = Duration::from_secs(2);
    const LONG: Duration = Duration::from_secs(1);

    async fn queued(replies: Vec<Reply>) -> DecisionOutcome {
        let (tx, mut rx) = mpsc::channel(replies.len().max(1));
        // Avoid send().unwrap(): Reply deliberately does not implement Debug.
        for reply in replies { assert!(tx.try_send(reply).is_ok()); }
        drop(tx);
        tokio::time::timeout(OUTER,
            wait_for_decision_with_limits("synthetic-request", &mut rx, LONG, LONG))
            .await.expect("bounded local decision wait")
    }

    fn handoff(outcome: DecisionOutcome, expected: &str) {
        match outcome {
            DecisionOutcome::Handoff(reason) => assert_eq!(reason, expected),
            DecisionOutcome::Decision(_) => panic!("expected a handoff, got a decision"),
        }
    }

    fn decision(outcome: DecisionOutcome, expected: &str) {
        match outcome {
            DecisionOutcome::Decision(value) => assert_eq!(value, expected),
            DecisionOutcome::Handoff(reason) => panic!("expected decision, got {reason}"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn duplicate_ack_does_not_discard_a_following_decision() {
        decision(queued(vec![Reply::Ack, Reply::Ack, Reply::Decision("allow".into())]).await, "allow");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn decision_before_ack_is_valid_and_preserves_the_exact_value() {
        let value = r#"{"answers":{"synthetic-question":"synthetic-answer"}}"#;
        decision(queued(vec![Reply::Decision(value.into()), Reply::Ack]).await, value);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn decline_before_ack_means_not_shown() {
        handoff(queued(vec![Reply::Decline, Reply::Decision("allow".into())]).await, "not-shown");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn decline_after_duplicate_ack_means_released() {
        handoff(queued(vec![Reply::Ack, Reply::Ack, Reply::Decline]).await, "released");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn channel_closed_before_ack_means_closed() {
        handoff(queued(vec![]).await, "closed");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn channel_closed_after_ack_means_closed() {
        handoff(queued(vec![Reply::Ack]).await, "closed");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn open_channel_without_ack_expires_as_unacknowledged() {
        let (tx, mut rx) = mpsc::channel(1);
        let result = tokio::time::timeout(OUTER, wait_for_decision_with_limits(
            "synthetic-unacknowledged", &mut rx, Duration::from_millis(15), LONG)).await;
        // Keep the sender alive throughout the wait: closure cannot pass this.
        drop(tx);
        handoff(result.expect("ack deadline must be bounded"), "unacknowledged");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn expired_total_budget_beats_ready_ack_and_decision() {
        let (tx, mut rx) = mpsc::channel(2);
        assert!(tx.try_send(Reply::Ack).is_ok());
        assert!(tx.try_send(Reply::Decision("allow".into())).is_ok());
        handoff(wait_for_decision_with_limits("synthetic-expired", &mut rx,
            LONG, Duration::ZERO).await, "unacknowledged");
        // The zero total budget must be checked before receiving even ready data.
        assert!(matches!(rx.try_recv(), Ok(Reply::Ack)));
        drop(tx);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn continuous_duplicate_ack_cannot_extend_the_absolute_total_deadline() {
        let (tx, mut rx) = mpsc::channel(1);
        assert!(tx.try_send(Reply::Ack).is_ok());
        let count = Arc::new(AtomicUsize::new(0));
        let counted = count.clone();
        let flooding_tx = tx.clone();
        let flood = tokio::spawn(async move {
            while flooding_tx.send(Reply::Ack).await.is_ok() {
                counted.fetch_add(1, Ordering::Relaxed);
            }
        });
        let result = tokio::time::timeout(OUTER, wait_for_decision_with_limits(
            "synthetic-flood", &mut rx, LONG, Duration::from_millis(100))).await;
        // Clean up before assertions, also on an outer timeout. The original
        // sender remains alive until here, so "closed" cannot masquerade as expiry.
        flood.abort();
        let _ = flood.await;
        drop(tx);
        handoff(result.expect("ready ACK flood must not starve the deadline"), "timeout");
        assert!(count.load(Ordering::Relaxed) > 1, "fixture must deliver repeated ACKs");
    }
}

#[cfg(test)]
mod pending_dispatch_tests {
    use super::*;

    #[test]
    fn repeated_ack_never_fills_the_queue_or_displaces_a_human_decision() {
        let (sender,mut rx)=mpsc::channel(4);
        let mut pending=HashMap::new();
        pending.insert("synthetic".into(),PendingRequest::new(sender,"synthetic-session".into(),"codex".into(),Instant::now()));
        for _ in 0..1000 { assert!(dispatch(&mut pending,"synthetic",Reply::Ack)); }
        assert!(dispatch(&mut pending,"synthetic",Reply::Decision("deny".into())));
        assert!(pending.is_empty());
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));
        assert!(matches!(rx.try_recv(),Ok(Reply::Decision(value)) if value=="deny"));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn a_decision_before_ack_is_preserved_and_late_replies_cannot_replace_it() {
        let (sender,mut rx)=mpsc::channel(4);
        let mut pending=HashMap::new();
        pending.insert("synthetic".into(),PendingRequest::new(sender,"synthetic-session".into(),"codex".into(),Instant::now()));
        assert!(dispatch(&mut pending,"synthetic",Reply::Decision("allow".into())));
        assert!(!dispatch(&mut pending,"synthetic",Reply::Ack));
        assert!(!dispatch(&mut pending,"synthetic",Reply::Decline));
        assert!(matches!(rx.try_recv(),Ok(Reply::Decision(value)) if value=="allow"));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn a_decline_releases_only_its_exact_pending_request() {
        let (a,mut a_rx)=mpsc::channel(4);let (b,mut b_rx)=mpsc::channel(4);
        let mut pending=HashMap::new();
        pending.insert("a".into(),PendingRequest::new(a,"a-session".into(),"claude".into(),Instant::now()));
        pending.insert("b".into(),PendingRequest::new(b,"b-session".into(),"copilot".into(),Instant::now()));
        assert!(dispatch(&mut pending,"a",Reply::Decline));
        assert!(!pending.contains_key("a"));assert!(pending.contains_key("b"));
        assert!(matches!(a_rx.try_recv(),Ok(Reply::Decline)));
        assert!(b_rx.try_recv().is_err());
        assert!(!dispatch(&mut pending,"missing",Reply::Decision("allow".into())));
    }

    #[test]
    fn handoff_event_has_only_the_exact_scoped_identity_and_reason() {
        let payload=handoff_payload("synthetic-request","synthetic-session","codex",Some("integration_github"),"released");
        assert_eq!(payload,json!({"requestId":"synthetic-request","sessionId":"synthetic-session","agent":"codex","provider":"integration_github","reason":"released"}));
        assert_eq!(handoff_payload("r","s","claude",None,"timeout")["provider"],Value::Null);
        assert!(payload.get("decision").is_none());assert!(payload.get("command").is_none());
    }
}

#[cfg(test)]
mod card_receipt_tests {
    use super::*;
    fn fixture(agent:&str)->(HashMap<String,PendingRequest>,mpsc::Receiver<Reply>,Instant) {
        let now=Instant::now();let(tx,rx)=mpsc::channel(4);let mut requests=HashMap::new();
        requests.insert("synthetic-request".into(),PendingRequest::new(tx,"synthetic-session".into(),agent.into(),now));
        (requests,rx,now)
    }
    fn decision()->Reply{Reply::Decision("allow".into())}
    #[test]fn canonical_pet_comes_from_trusted_owner_then_agent_then_claude_editor() {
        for(agent,pet)in[("codex","agent_codex"),("copilot","agent_copilot"),("muse","agent_muse"),("antigravity","agent_antigravity")] {
            assert_eq!(request_pet(&json!({"coucou_agent":agent})),pet);
            assert_eq!(request_pet(&json!({"coucou_agent":agent,"coucou_pet_provider":"integration_github"})),"integration_github");
        }
        assert_eq!(request_pet(&json!({})),"integration_claude");
        assert_eq!(request_pet(&json!({"term_editor":"cursor"})),"agent_cursor");
        assert_eq!(request_pet(&json!({"coucou_agent":"claude","term_editor":"cursor"})),"agent_cursor");
        assert_eq!(request_pet(&json!({"coucou_agent":"../../bad","coucou_pet_provider":"unknown"})),"integration_claude");
        assert_eq!(request_pet(&json!({"coucou_agent":"codex","coucou_pet_provider":"ollama"})),"ai_ollama");
    }
    #[test]fn scoped_presentations_cover_every_supported_hook_without_answering() {
        for agent in ["integration_claude","agent_cursor","agent_codex","agent_copilot","agent_muse","agent_antigravity","integration_github","ai_ollama"] {
            let(mut requests,mut rx,now)=fixture(agent);
            assert!(!present_at(&mut requests,"wrong-id","synthetic-session",agent,true,now));
            assert!(!present_at(&mut requests,"synthetic-request","wrong-session",agent,true,now));
            assert!(!present_at(&mut requests,"synthetic-request","synthetic-session","wrong-agent",true,now));
            assert!(rx.try_recv().is_err());
            assert!(present_at(&mut requests,"synthetic-request","synthetic-session",agent,true,now));
            assert!(requests["synthetic-request"].card_presented);
            assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));
            assert!(rx.try_recv().is_err());assert_eq!(requests.len(),1);
        }
    }
    #[test]fn hidden_or_foreign_surface_never_receives_a_card_receipt() {
        let rect=crate::island::IslandRect{x:10.0,y:0.0,w:600.0,h:180.0};
        assert!(card_surface_ready("island",true,false,rect));
        for(label,visible,collapsed)in[("pet-control",true,false),("companions",true,false),("island",false,false),("island",true,true)] {
            assert!(!card_surface_ready(label,visible,collapsed,rect));
        }
        for rect in [crate::island::IslandRect::default(),crate::island::IslandRect{x:f64::NAN,..rect},crate::island::IslandRect{w:f64::INFINITY,..rect},crate::island::IslandRect{h:-1.0,..rect}] {
            assert!(!card_surface_ready("island",true,false,rect));
        }
        let(mut requests,mut rx,now)=fixture("codex");
        assert!(!present_at(&mut requests,"synthetic-request","synthetic-session","codex",false,now));
        assert!(!requests["synthetic-request"].acknowledged);assert!(rx.try_recv().is_err());
    }
    #[test]fn a_card_cannot_decide_before_its_receipt_or_after_it_is_hidden() {
        let(mut requests,mut rx,now)=fixture("codex");
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),true,now));
        assert!(present_at(&mut requests,"synthetic-request","synthetic-session","codex",true,now));
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),false,now));
        assert!(requests.contains_key("synthetic-request"));assert!(rx.try_recv().is_err());
        assert!(card_dispatch_at(&mut requests,"synthetic-request",decision(),true,now));
        assert!(matches!(rx.try_recv(),Ok(Reply::Decision(value))if value=="allow"));
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),true,now));
    }
    #[test]fn old_blind_ack_is_not_a_card_receipt() {
        let(mut requests,mut rx,now)=fixture("copilot");
        assert!(dispatch_at(&mut requests,"synthetic-request",Reply::Ack,now));
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),true,now));
        assert!(rx.try_recv().is_err());assert_eq!(requests.len(),1);
    }
    #[test]fn expiry_is_enforced_before_async_cleanup_and_duplicate_receipts_cannot_renew_it() {
        let(mut requests,mut rx,now)=fixture("codex");
        let ack_deadline=requests["synthetic-request"].ack_deadline;
        assert!(!present_at(&mut requests,"synthetic-request","synthetic-session","codex",true,ack_deadline));
        assert!(!dispatch_at(&mut requests,"synthetic-request",decision(),ack_deadline));
        assert!(present_at(&mut requests,"synthetic-request","synthetic-session","codex",true,now));
        let deadline=requests["synthetic-request"].deadline;
        for _ in 0..1000{assert!(present_at(&mut requests,"synthetic-request","synthetic-session","codex",true,now));}
        assert_eq!(requests["synthetic-request"].deadline,deadline);
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));assert!(rx.try_recv().is_err());
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),true,deadline));
        assert!(!present_at(&mut requests,"synthetic-request","synthetic-session","codex",true,deadline));
        assert!(rx.try_recv().is_err());
    }
    #[test]fn ending_a_relay_never_reopens_or_consumes_another_request() {
        let(mut requests,mut rx,now)=fixture("muse");
        assert!(present_at(&mut requests,"synthetic-request","synthetic-session","muse",true,now));
        let(tx,mut other_rx)=mpsc::channel(4);
        requests.insert("other".into(),PendingRequest::new(tx,"other-session".into(),"codex".into(),now));
        assert!(dispatch_at(&mut requests,"synthetic-request",Reply::Decline,now));
        assert!(!present_at(&mut requests,"synthetic-request","synthetic-session","muse",true,now));
        assert!(!card_dispatch_at(&mut requests,"synthetic-request",decision(),true,now));
        assert!(requests.contains_key("other"));assert!(other_rx.try_recv().is_err());
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));assert!(matches!(rx.try_recv(),Ok(Reply::Decline)));
    }
    #[tokio::test(flavor="current_thread")]
    async fn disconnected_client_retires_before_a_simultaneously_ready_answer() {
        let(mut server,client)=tokio::io::duplex(64);drop(client);
        let outcome=until_relay_closed(&mut server,async{DecisionOutcome::Decision("allow".into())}).await;
        assert!(matches!(outcome,DecisionOutcome::Handoff("relay-closed")));
    }
    #[tokio::test(flavor="current_thread")]
    async fn connected_client_still_receives_the_exact_human_question_result() {
        let(mut server,_client)=tokio::io::duplex(64);
        let reply=json!({"answers":{"synthetic-question":"synthetic-choice"}}).to_string();
        let outcome=until_relay_closed(&mut server,async{DecisionOutcome::Decision(reply.clone())}).await;
        assert!(matches!(outcome,DecisionOutcome::Decision(actual)if actual==reply));
    }
    #[cfg(target_os="linux")]
    #[tokio::test(flavor="current_thread")]
    async fn unix_writer_half_close_keeps_its_reply_receiver_usable() {
        let(mut server,mut client)=tokio::net::UnixStream::pair().unwrap();
        client.shutdown().await.unwrap();
        let reply=json!({"answers":{"question":"choice"}}).to_string();
        let outcome=until_relay_closed(&mut server,async{
            tokio::time::sleep(Duration::from_millis(35)).await;
            DecisionOutcome::Decision(reply.clone())
        }).await;
        assert!(matches!(outcome,DecisionOutcome::Decision(actual)if actual==reply));
        server.write_all(b"synthetic-reply\n").await.unwrap();
        let mut buf=[0u8;16];let n=client.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n],b"synthetic-reply\n");
    }
    #[cfg(target_os="linux")]
    #[tokio::test(flavor="current_thread")]
    async fn unix_full_close_after_writer_half_close_retires_without_any_decision() {
        let(mut server,mut client)=tokio::net::UnixStream::pair().unwrap();
        client.shutdown().await.unwrap();
        let closer=tokio::spawn(async move{tokio::time::sleep(Duration::from_millis(35)).await;drop(client);});
        let outcome=tokio::time::timeout(Duration::from_secs(2),until_relay_closed(&mut server,std::future::pending())).await.unwrap();
        closer.await.unwrap();
        assert!(matches!(outcome,DecisionOutcome::Handoff("relay-closed")));
    }
    #[tokio::test(flavor="current_thread")]
    async fn time_spent_before_emit_counts_toward_the_original_ack_deadline() {
        let(tx,mut rx)=mpsc::channel(4);assert!(tx.try_send(Reply::Ack).is_ok());assert!(tx.try_send(decision()).is_ok());
        let started=tokio::time::Instant::now()-Duration::from_secs(2);
        assert!(matches!(wait_for_decision_from("synthetic",&mut rx,Duration::from_secs(1),Duration::from_secs(10),started).await,DecisionOutcome::Handoff("unacknowledged")));
        assert!(matches!(rx.try_recv(),Ok(Reply::Ack)));
    }
}
