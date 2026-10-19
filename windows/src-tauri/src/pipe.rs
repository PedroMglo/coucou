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
use std::time::Duration;

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
struct PendingRequest { sender:mpsc::Sender<Reply>, acknowledged:bool }
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
    {
        let pending = app.state::<Pending>();
        pending.0.lock().unwrap().insert(id.clone(), PendingRequest{sender:tx,acknowledged:false});
    }
    crate::recap::note_request(&app, &id, &payload);
    payload["request_id"] = json!(id);
    log::line(format!("hook PermissionRequest id={id}"));
    let _ = app.emit_to(WINDOW_LABEL, "hook", payload);

    let outcome = if gate {
        let mut closed=[0u8;1];
        tokio::select! {answer=wait_for_decision(&id,&mut rx)=>answer,_=pipe.read(&mut closed)=>DecisionOutcome::Handoff("relay-closed")}
    } else {wait_for_decision(&id, &mut rx).await};
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

async fn wait_for_decision(id: &str, rx: &mut mpsc::Receiver<Reply>) -> DecisionOutcome {
    let outcome = wait_for_decision_with_limits(id, rx, ACK_TIMEOUT, DECISION_TIMEOUT).await;
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

async fn wait_for_decision_with_limits(
    _id: &str,
    rx: &mut mpsc::Receiver<Reply>,
    ack_timeout: Duration,
    total_timeout: Duration,
) -> DecisionOutcome {
    use tokio::time::{timeout_at, Instant};

    // The total budget starts on entry, including the acknowledgement wait.
    // Neither the first acknowledgement nor duplicates can extend it.
    let start = Instant::now();
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
    send(app, request_id, Reply::Ack);
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
        pending.insert("synthetic".into(),PendingRequest{sender,acknowledged:false});
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
        pending.insert("synthetic".into(),PendingRequest{sender,acknowledged:false});
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
        pending.insert("a".into(),PendingRequest{sender:a,acknowledged:false});
        pending.insert("b".into(),PendingRequest{sender:b,acknowledged:false});
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
