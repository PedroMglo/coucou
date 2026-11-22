// Global shortcuts on Wayland, through the XDG desktop portal
// (org.freedesktop.portal.GlobalShortcuts): KDE Plasma 6, GNOME 48+,
// Hyprland's portal, COSMIC…
//
// Wayland lets no app grab keys, so the desktop holds the shortcuts and tells
// us when one is pressed. The exchange, all on the session bus:
//
//   CreateSession({handle_token, session_handle_token})  → Request → Response
//   BindShortcuts(session, [(id, {description, preferred_trigger})], "", {handle_token})
//                                                        → Request → Response
//   … then Activated(session, id, timestamp, options) on every press, and
//   ShortcutsChanged(session, shortcuts) when the user picks other keys.
//
// The desktop may show its own dialog to confirm the shortcuts or change their
// keys, so a Response can take as long as the user does: nothing here waits
// for one. Everything runs on one thread that owns the D-Bus connection and
// sleeps in poll() on the bus and a wake-up socket — no timeout, so it costs
// nothing while no key is pressed. Shortcuts.rs talks to it through a Handle
// and hears back through the callback given to `start`; this file knows
// nothing of Tauri.
//
// The D-Bus client is the `dbus` crate (libdbus), which keyring's Secret
// Service support already links.

use std::io::{ErrorKind, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use dbus::arg::{ArgType, PropMap, RefArg, Variant};
use dbus::channel::{BusType, Channel};
use dbus::message::MessageType;
use dbus::strings::Path;
use dbus::Message;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

const DEST: &str = "org.freedesktop.portal.Desktop";
const OBJECT: &str = "/org/freedesktop/portal/desktop";
const IFACE: &str = "org.freedesktop.portal.GlobalShortcuts";
const REQUEST: &str = "org.freedesktop.portal.Request";
const SESSION: &str = "org.freedesktop.portal.Session";
const BUS: &str = "org.freedesktop.DBus";
const BUS_OBJECT: &str = "/org/freedesktop/DBus";
/// Tauri's identifier. Told to the portal so the desktop files the shortcuts
/// under Coucou rather than under an unnamed app.
const APP_ID: &str = "fr.louisraille.coucou";
/// For the calls themselves, which answer at once: the user's answer comes
/// later, in a Response signal.
const CALL_TIMEOUT: Duration = Duration::from_secs(10);

// ── Pure helpers ──────────────────────────────────────────────────────────────

/// Coucou's accelerator ("Ctrl+Alt+Space") → the XDG shortcuts spec's trigger
/// ("CTRL+ALT+space"): modifiers CTRL, ALT, SHIFT, LOGO, then an xkb keysym
/// name. `None` for a binding that isn't a shortcut, or whose key has no
/// keysym here; the desktop then picks the key, or lets the user pick one.
pub fn xdg_trigger(keys: &str) -> Option<String> {
    trigger_of(&crate::shortcuts::parse(keys)?)
}

fn trigger_of(shortcut: &Shortcut) -> Option<String> {
    let mut parts = Vec::new();
    if shortcut.mods.contains(Modifiers::CONTROL) {
        parts.push("CTRL".to_string());
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        parts.push("ALT".to_string());
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        parts.push("SHIFT".to_string());
    }
    if shortcut.mods.intersects(Modifiers::SUPER | Modifiers::META) {
        parts.push("LOGO".to_string());
    }
    parts.push(keysym(shortcut.key)?);
    Some(parts.join("+"))
}

/// The xkb keysym name of the keys an accelerator may name (KEY_NAMES in
/// src/core/shortcuts.ts).
fn keysym(code: Code) -> Option<String> {
    use Code::*;
    let fixed = match code {
        Space => "space",
        Enter => "Return",
        Tab => "Tab",
        Backspace => "BackSpace",
        Delete => "Delete",
        Insert => "Insert",
        Home => "Home",
        End => "End",
        PageUp => "Page_Up",
        PageDown => "Page_Down",
        ArrowUp => "Up",
        ArrowDown => "Down",
        ArrowLeft => "Left",
        ArrowRight => "Right",
        Comma => "comma",
        Period => "period",
        Slash => "slash",
        Semicolon => "semicolon",
        Quote => "apostrophe",
        Backquote => "grave",
        BracketLeft => "bracketleft",
        BracketRight => "bracketright",
        Backslash => "backslash",
        Minus => "minus",
        Equal => "equal",
        _ => "",
    };
    if !fixed.is_empty() {
        return Some(fixed.to_string());
    }
    // KeyA → a, Digit1 → 1, F5 → F5.
    let name = code.to_string();
    if let Some(letter) = name.strip_prefix("Key").filter(|l| l.len() == 1) {
        return Some(letter.to_ascii_lowercase());
    }
    if let Some(digit) = name.strip_prefix("Digit").filter(|d| d.len() == 1) {
        return Some(digit.to_string());
    }
    let function = name
        .strip_prefix('F')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=24).contains(&n));
    function.then_some(name)
}

/// Where the portal puts the Request object for `token`: the caller's unique
/// name without its colon, dots made underscores.
pub fn request_path(sender: &str, token: &str) -> String {
    format!("{OBJECT}/request/{}/{token}", escape_sender(sender))
}

/// Same rule for the Session object.
pub fn session_path(sender: &str, token: &str) -> String {
    format!("{OBJECT}/session/{}/{token}", escape_sender(sender))
}

fn escape_sender(sender: &str) -> String {
    sender.trim_start_matches(':').replace('.', "_")
}

/// One shortcut handed to the desktop.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Wanted {
    pub id: String,
    pub description: String,
    /// XDG trigger; `None` leaves the key to the desktop.
    pub trigger: Option<String>,
}

/// One shortcut the desktop holds, and the keys it says run it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Bound {
    pub id: String,
    /// `trigger_description`, as the desktop writes it; `None` when it gave
    /// none or an empty one.
    pub trigger: Option<String>,
}

/// The value inside a variant, however deep.
fn unwrap_variant(arg: &dyn RefArg) -> &dyn RefArg {
    let mut arg = arg;
    while arg.arg_type() == ArgType::Variant {
        match arg.as_iter().and_then(|mut inner| inner.next()) {
            Some(inner) => arg = inner,
            None => break,
        }
    }
    arg
}

/// `key` in an `a{sv}` read off the bus.
fn dict_get<'a>(dict: &'a dyn RefArg, key: &str) -> Option<&'a dyn RefArg> {
    let mut items = unwrap_variant(dict).as_iter()?;
    while let (Some(k), Some(v)) = (items.next(), items.next()) {
        if k.as_str() == Some(key) {
            return Some(unwrap_variant(v));
        }
    }
    None
}

/// The `a(sa{sv})` list of BindShortcuts' Response and of ShortcutsChanged.
/// `None` when it isn't one.
pub fn parse_shortcuts(arg: &dyn RefArg) -> Option<Vec<Bound>> {
    let list = unwrap_variant(arg);
    if list.arg_type() != ArgType::Array {
        return None;
    }
    let mut out = Vec::new();
    for item in list.as_iter()? {
        if out.len()>=32{return None;}
        let mut fields = item.as_iter()?;
        let id = fields.next()?.as_str()?.to_string();
        if id.is_empty()||id.len()>128{return None;}
        let trigger = fields
            .next()
            .and_then(|props| dict_get(props, "trigger_description"))
            .and_then(|t| t.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string);
        out.push(Bound { id, trigger });
        if out.last().and_then(|b|b.trigger.as_ref()).is_some_and(|t|t.len()>1024){return None;}
    }
    Some(out)
}

/// A Request's Response: `Ok(results)` when the user went ahead (0), `Err`
/// with the code otherwise (1 cancelled, 2 anything else).
pub fn parse_response(msg: &Message) -> Result<PropMap, u32> {
    match msg.read2::<u32, PropMap>() {
        Ok((0, results)) => Ok(results),
        Ok((code, _)) => Err(code),
        Err(_) => Err(2),
    }
}

/// The session CreateSession made: `session_handle` from its results (a
/// string or an object path, depending on the portal's age).
pub fn session_from_results(results: &PropMap) -> Option<String> {
    let handle = results.get("session_handle")?.0.as_str()?;
    (!handle.is_empty()).then(|| handle.to_string())
}

// ── The portal thread ─────────────────────────────────────────────────────────

/// What the thread tells shortcuts.rs. `gen` is the `bind` call it answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The desktop holds the shortcuts. `None` when it didn't say which.
    Bound { gen: u64, shortcuts: Option<Vec<Bound>> },
    /// The user gave some shortcuts other keys.
    Changed { gen: u64, shortcuts: Vec<Bound> },
    /// Refused, failed, or the desktop closed the session.
    Failed { gen: u64, reason: String },
    /// No GlobalShortcuts portal (or no session bus). The thread has stopped.
    Unavailable(String),
    /// A shortcut was pressed.
    Activated { gen: u64, id: String },
    /// A shortcut was released (Flow dictation's hold-to-talk key).
    Deactivated { gen: u64, id: String },
}

enum Command {
    Bind { gen: u64, shortcuts: Vec<Wanted> },
    Reserve { gen: u64, shortcuts: Vec<Wanted>, reply: mpsc::Sender<Result<Reservation, String>> },
    Reviewed { reservation:Reservation, guard:review::Guard, lifetime:review::Guard, reply:mpsc::Sender<review::Outcome> },
    FinishReviewed { generation:u64, commit:Option<review::Guard>, reply:mpsc::Sender<bool> },
    CleanupPublished { generation:u64, reply:mpsc::Sender<bool> },
    ObserveSubject { subject:observation::Subject, reply:mpsc::Sender<observation::Observation> },
    Release,
}

/// A passive allocation in this same native transport, not a bind, grant or
/// receipt. The opaque client/peer and exact paths cannot be chosen by a webview.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Reservation {
    #[serde(serialize_with = "decimal_generation")]
    pub generation: u64,
    pub peer: String,
    pub client: String,
    pub bus_id: String,
    pub session: String,
    pub create_request: String,
    pub bind_request: String,
    pub shortcuts: Vec<Wanted>,
    pub previous: review::Before,
}
fn decimal_generation<S: serde::Serializer>(n: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&n.to_string())
}

/// The way to the portal thread. Sending never blocks.
pub struct Handle {
    tx: mpsc::Sender<Command>,
    wake: UnixStream,
}

impl Handle {
    pub(crate)fn observe_subject(&self,subject:observation::Subject)->Option<mpsc::Receiver<observation::Observation>>{
        let(reply,rx)=mpsc::channel();self.send(Command::ObserveSubject{subject,reply}).then_some(rx)
    }
    pub(crate)fn cleanup_published(&self,generation:u64)->Option<mpsc::Receiver<bool>>{
        let(reply,rx)=mpsc::channel();self.send(Command::CleanupPublished{generation,reply}).then_some(rx)
    }
    pub(crate)fn apply_reserved(&self,reservation:Reservation,guard:review::Guard,lifetime:review::Guard)->Option<mpsc::Receiver<review::Outcome>>{
        let(reply,rx)=mpsc::channel();self.send(Command::Reviewed{reservation,guard,lifetime,reply}).then_some(rx)
    }
    pub(crate)fn finish_reserved(&self,generation:u64,commit:Option<review::Guard>)->Option<mpsc::Receiver<bool>>{
        let(reply,rx)=mpsc::channel();self.send(Command::FinishReviewed{generation,commit,reply}).then_some(rx)
    }
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        let (tx, _) = mpsc::channel();
        let (wake, _) = UnixStream::pair().unwrap();
        Self { tx, wake }
    }
    /// Asks the desktop for `shortcuts` in place of the current ones. False
    /// when the thread is gone (no portal).
    pub fn bind(&self, gen: u64, shortcuts: Vec<Wanted>) -> bool {
        self.send(Command::Bind { gen, shortcuts })
    }

    /// Gives every shortcut back (Settings is recording a new one).
    pub fn release(&self) -> bool {
        self.send(Command::Release)
    }

    pub(crate) fn reserve(&self, gen: u64, shortcuts: Vec<Wanted>) -> Option<mpsc::Receiver<Result<Reservation, String>>> {
        let (reply, rx) = mpsc::channel();
        self.send(Command::Reserve { gen, shortcuts, reply }).then_some(rx)
    }

    fn send(&self, command: Command) -> bool {
        if self.tx.send(command).is_err() {
            return false;
        }
        // A full socket means a wake-up is already pending.
        let _ = (&self.wake).write(&[1]);
        true
    }
}

/// Starts the portal thread. It first checks there is a portal to talk to,
/// and says `Unavailable` (then stops) when there isn't.
pub fn start(on_event: impl Fn(Event) + Send + 'static) -> Option<Handle> {
    let (tx, rx) = mpsc::channel();
    let (wake_rx, wake_tx) = UnixStream::pair().ok()?;
    wake_rx.set_nonblocking(true).ok()?;
    wake_tx.set_nonblocking(true).ok()?;
    std::thread::Builder::new()
        .name("shortcuts-portal".into())
        .spawn(move || run(rx, wake_rx, on_event))
        .ok()?;
    Some(Handle { tx, wake: wake_tx })
}

fn run(rx: mpsc::Receiver<Command>, wake: UnixStream, on_event: impl Fn(Event)) {
    let mut portal = match Portal::connect() {
        Ok(portal) => portal,
        Err(why) => return on_event(Event::Unavailable(why)),
    };
    loop {
        portal.review_tick(&on_event);
        loop {
            match rx.try_recv() {
                Ok(command) => portal.command(command, &on_event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return portal.abandon(),
            }
        }
        while let Some(msg) = portal.channel.pop_message() {
            portal.message(&msg, &on_event);
            if !portal.peer_alive {
                // The callback has invalidated availability/held input. Do
                // not send Close to a replacement or infer remote rollback.
                return;
            }
        }
        if !portal.channel.is_connected() {
            return on_event(Event::Unavailable("the session bus went away".into()));
        }
        if let Err(why) = wait(&portal.channel, &wake,portal.reviewed.is_some()) {
            portal.abandon();
            return on_event(Event::Unavailable(why));
        }
        // Drain the wake-ups; the commands themselves are in `rx`.
        let mut buf = [0u8; 64];
        while matches!((&wake).read(&mut buf), Ok(n) if n > 0) {}
        if portal.channel.read_write(Some(Duration::ZERO)).is_err() {
            return on_event(Event::Unavailable("the session bus went away".into()));
        }
    }
}

/// Sleeps until the bus or a command has something. No timeout.
fn wait(channel: &Channel, wake: &UnixStream, reviewing:bool) -> Result<(), String> {
    let watch = channel.watch();
    let mut events = libc::POLLIN;
    if channel.has_messages_to_send() {
        events |= libc::POLLOUT;
    }
    let mut fds = [
        libc::pollfd { fd: watch.fd, events, revents: 0 },
        libc::pollfd { fd: wake.as_raw_fd(), events: libc::POLLIN, revents: 0 },
    ];
    loop {
        let n = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, if reviewing{100}else{-1}) };
        if n >= 0 {
            return Ok(());
        }
        let err = std::io::Error::last_os_error();
        if err.kind() != ErrorKind::Interrupted {
            return Err(format!("poll: {err}"));
        }
    }
}

/// Where the exchange with the desktop stands.
enum Phase {
    Idle,
    /// CreateSession sent, its Response awaited.
    Creating { gen: u64, request: String, session: String, want: Vec<Wanted> },
    /// BindShortcuts sent; the desktop may be showing its dialog.
    Binding { gen: u64, request: String, session: String, want: Vec<Wanted> },
    Bound { gen: u64, session: String, want: Vec<Wanted>, shortcuts: Option<Vec<Bound>>, pressed: std::collections::BTreeSet<String> },
}

impl Phase {
    /// A release of our own held key is cleanup, even if the desktop changed
    /// its accepted list meanwhile. It cannot manufacture a new press.
    fn key_event(&mut self, from: &str, id: String, down: bool) -> Option<Event> {
        let Self::Bound { gen, session, want, shortcuts, pressed } = self else { return None };
        if from != session || !want.iter().any(|key| key.id == id) { return None; }
        if down {
            if shortcuts.as_ref().is_some_and(|keys| !keys.iter().any(|key| key.id == id)) || !pressed.insert(id.clone()) { return None; }
            Some(Event::Activated { gen: *gen, id })
        } else {
            pressed.remove(&id).then_some(Event::Deactivated { gen: *gen, id })
        }
    }
}

struct Portal {
    bus_id:String,closed:std::collections::VecDeque<String>,
    channel: Channel,
    sender: String,
    /// Unique bus peer that answered discovery. A replacement service is not
    /// silently substituted for this handle's native session owner.
    peer: String,
    peer_alive: bool,
    last_gen: u64,
    tokens: u64,
    phase: Phase,
    reviewed: Option<review::Flight>,
}
#[path="portal_review.rs"]
pub(crate)mod review;
#[path="portal_observation.rs"]pub(crate)mod observation;

impl Portal {
    fn connect() -> Result<Portal, String> {
        let mut channel = Channel::get_private(BusType::Session).map_err(|e| format!("session bus: {e}"))?;
        channel.set_watch_enabled(true);
        let sender = channel.unique_name().ok_or("no unique name on the session bus")?.to_string();
        let mut portal = Portal { bus_id:String::new(),closed:Default::default(),channel, sender, peer: String::new(), peer_alive: true, last_gen: 0, tokens: 0, phase: Phase::Idle, reviewed:None };

        let bus=portal.call(Message::new_method_call(BUS,BUS_OBJECT,BUS,"GetId")?)?;
        if bus.sender().as_deref()!=Some(BUS){return Err("session bus identity not authenticated".into());}
        portal.bus_id=bus.read1().map_err(|_|"session bus identity unavailable")?;
        if portal.bus_id.len()!=32||!portal.bus_id.bytes().all(|b|b.is_ascii_hexdigit()){return Err("session bus identity malformed".into());}
        // Names the app to the portal (xdg-desktop-portal 1.19+, apps outside
        // a sandbox). It has to come before any other portal call; an older
        // portal doesn't know it, and then the desktop names us itself.
        if let Err(err) = portal.call(
            method(OBJECT, "org.freedesktop.host.portal.Registry", "Register")?
                .append2(APP_ID, PropMap::new()),
        ) {
            crate::log::line(format!("shortcuts portal: no app registry ({err})"));
        }

        let reply = portal
            .call(
                method(OBJECT, "org.freedesktop.DBus.Properties", "Get")?.append2(IFACE, "version"),
            )
            .map_err(|e| format!("no GlobalShortcuts portal: {e}"))?;
        let version = reply.read1::<Variant<u32>>().map_err(|e| format!("GlobalShortcuts version: {e}"))?.0;
        portal.peer = reply.sender().filter(|name| name.starts_with(':')).ok_or("no unique GlobalShortcuts owner")?.to_string();
        crate::log::line(format!("shortcuts portal: GlobalShortcuts version {version}"));

        for rule in [
            format!("type='signal',sender='{}',interface='{IFACE}'", portal.peer),
            format!("type='signal',sender='{}',interface='{REQUEST}',member='Response'", portal.peer),
            format!("type='signal',sender='{}',interface='{SESSION}',member='Closed'", portal.peer),
            format!("type='signal',sender='{BUS}',path='{BUS_OBJECT}',interface='{BUS}',member='NameOwnerChanged',arg0='{}'", portal.peer),
        ] {
            let add = Message::new_method_call(
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                "AddMatch",
            )?
            .append1(rule);
            portal.call(add).map_err(|e| format!("AddMatch: {e}"))?;
        }
        // Subscribe before checking: a peer that died between discovery and
        // AddMatch must not leave an apparently live handle waiting forever.
        portal.require_live_peer()?;
        Ok(portal)
    }

    fn require_live_peer(&self) -> Result<(), String> {
        let reply = self.call(Message::new_method_call(BUS, BUS_OBJECT, BUS, "NameHasOwner")?.append1(self.peer.clone()))?;
        match reply.read1::<bool>() {
            Ok(true) => Ok(()),
            Ok(false) => Err("the GlobalShortcuts owner went away".into()),
            Err(e) => Err(format!("GlobalShortcuts owner status: {e}")),
        }
    }

    fn call(&self, mut msg: Message) -> Result<Message, String> {
        if !self.peer.is_empty() && msg.destination().as_deref() == Some(DEST) {
            if !self.peer_alive { return Err("the GlobalShortcuts owner went away".into()); }
            msg.set_destination(Some(self.peer.as_str().into()));
        }
        self.channel.send_with_reply_and_block(msg, CALL_TIMEOUT).map_err(|e| {
            format!("{}: {}", e.name().unwrap_or("error"), e.message().unwrap_or(""))
        })
    }

    /// A method call we don't wait on (closing a request or a session).
    fn close(&mut self, path: &str, iface: &str) {
        if iface==SESSION{let _=self.close_ack(path);return;}
        if let Ok(msg) = Message::new_method_call(self.peer.as_str(), path, iface, "Close") {
            let _ = self.channel.send(msg);
            self.channel.flush();
        }
    }

    fn token(&mut self) -> String {
        self.tokens += 1;
        format!("coucou{}_{}", std::process::id(), self.tokens)
    }

    fn gen(&self) -> Option<u64> {
        match &self.phase {
            Phase::Idle => None,
            Phase::Creating { gen, .. } | Phase::Binding { gen, .. } | Phase::Bound { gen, .. } => Some(*gen),
        }
    }

    /// Drops whatever is in flight or bound. The portal may need a new session
    /// to bind other shortcuts, so a change always starts over.
    fn abandon(&mut self) {
        match std::mem::replace(&mut self.phase, Phase::Idle) {
            Phase::Idle => {}
            Phase::Creating { request, session, .. } | Phase::Binding { request, session, .. } => {
                self.close(&request, REQUEST);
                self.close(&session, SESSION);
            }
            Phase::Bound { session, .. } => self.close(&session, SESSION),
        }
    }

    fn command(&mut self, command: Command, on_event: &impl Fn(Event)) {
        match command {
            Command::ObserveSubject{subject,reply}=>{let _=reply.send(self.observe_subject(subject));},
            Command::CleanupPublished{generation,reply}=>{let ok=self.cleanup_published(generation,on_event);let _=reply.send(ok);},
            Command::Reviewed{reservation,guard,lifetime,reply}=>self.begin_review(reservation,guard,lifetime,reply,on_event),
            Command::FinishReviewed{generation,commit,reply}=>{let ok=self.finish_review(generation,commit,on_event);let _=reply.send(ok);},
            Command::Reserve { gen, shortcuts, reply } => { let _ = reply.send(self.reserve(gen, shortcuts)); }
            Command::Release => {self.abort_review("explicit release",on_event);self.abandon();},
            Command::Bind { gen, shortcuts } => {
                if self.reviewed.is_some(){return on_event(Event::Failed{gen,reason:"reviewed operation still owns the native session".into()});}
                // Idempotent consultation of this same live generation. New
                // generations get a new session: queued old signals must not
                // be relabelled as new authority, even for identical keys.
                if let Phase::Bound { gen: current, want, shortcuts: bound, .. } = &mut self.phase {
                    if *current == gen && *want == shortcuts {
                        return on_event(Event::Bound { gen, shortcuts: bound.clone() });
                    }
                }
                if gen == 0 || gen <= self.last_gen {
                    return on_event(Event::Failed { gen, reason: "generation already used or not increasing".into() });
                }
                self.last_gen = gen;
                self.abandon();
                if shortcuts.is_empty() {
                    return on_event(Event::Bound { gen, shortcuts: Some(Vec::new()) });
                }
                if let Err(reason) = self.create_session(gen, shortcuts) {
                    on_event(Event::Failed { gen, reason });
                }
            }
        }
    }

    fn reserve(&mut self, generation: u64, shortcuts: Vec<Wanted>) -> Result<Reservation, String> {
        if self.reviewed.is_some() || !self.peer_alive || generation == 0 || generation <= self.last_gen || self.tokens > u64::MAX-3
            || shortcuts.len() > 32 || shortcuts.iter().any(|w| w.id.is_empty() || w.id.len() > 128 || w.description.len() > 1024 || w.trigger.as_ref().is_some_and(|s| s.len() > 256))
            || shortcuts.iter().map(|w| &w.id).collect::<std::collections::BTreeSet<_>>().len() != shortcuts.len() {
            return Err("invalid or stale native shortcut reservation".into());
        }
        self.require_live_peer()?;
        let create = self.token(); let session = self.token(); let bind = self.token();
        Ok(Reservation { generation, peer:self.peer.clone(), client:self.sender.clone(),bus_id:self.bus_id.clone(),
            session:session_path(&self.sender, &session), create_request:request_path(&self.sender, &create),
            bind_request:request_path(&self.sender, &bind), shortcuts,previous:review::Before::capture(&self.phase) })
    }

    fn create_session(&mut self, gen: u64, want: Vec<Wanted>) -> Result<(), String> {
        let handle_token = self.token();
        let session_token = self.token();
        let mut options = PropMap::new();
        options.insert("handle_token".into(), Variant(Box::new(handle_token.clone())));
        options.insert("session_handle_token".into(), Variant(Box::new(session_token.clone())));
        let reply = self.call(method(OBJECT, IFACE, "CreateSession")?.append1(options))?;
        // An old portal may not follow the token; its answer is the truth.
        let request = reply
            .read1::<Path>()
            .map(|p| p.to_string())
            .unwrap_or_else(|_| request_path(&self.sender, &handle_token));
        let session = session_path(&self.sender, &session_token);
        self.phase = Phase::Creating { gen, request, session, want };
        Ok(())
    }

    fn bind_shortcuts(&mut self, gen: u64, session: String, want: Vec<Wanted>) -> Result<(), String> {
        let list: Vec<(String, PropMap)> = want
            .iter()
            .map(|w| {
                let mut props = PropMap::new();
                props.insert("description".into(), Variant(Box::new(w.description.clone())));
                if let Some(trigger) = &w.trigger {
                    props.insert("preferred_trigger".into(), Variant(Box::new(trigger.clone())));
                }
                (w.id.clone(), props)
            })
            .collect();
        let handle_token = self.token();
        let mut options = PropMap::new();
        options.insert("handle_token".into(), Variant(Box::new(handle_token.clone())));
        let session_obj = Path::new(session.clone())?;
        let msg = method(OBJECT, IFACE, "BindShortcuts")?.append3(session_obj, list, "").append1(options);
        // Bound to the session even if the call fails, so abandon() closes it.
        self.phase = Phase::Binding {
            gen,
            request: request_path(&self.sender, &handle_token),
            session: session.clone(),
            want: want.clone(),
        };
        let reply = self.call(msg)?;
        if let Ok(path) = reply.read1::<Path>() {
            self.phase = Phase::Binding { gen, request: path.to_string(), session, want };
        }
        Ok(())
    }

    fn message(&mut self, msg: &Message, on_event: &impl Fn(Event)) {
        if !self.peer_alive { return; }
        // The bus daemon owns this reserved sender; another client's directed
        // signal with the same path/interface/payload cannot impersonate it.
        // Watch the unique peer, not the replaceable well-known service name:
        // relinquishing that name while the peer lives does not end our session.
        if peer_disappeared(msg, &self.peer) {
            self.peer_alive = false;
            self.abort_review("native owner disappeared; remote outcome unknown",on_event);
            self.phase = Phase::Idle;
            on_event(Event::Unavailable("the GlobalShortcuts owner went away".into()));
            return;
        }
        // Directed signals are delivered without an AddMatch subscription.
        // The actual unique sender, not interface/path/text, authenticates the
        // native peer. Calls/close also stay bound to this same owner.
        if msg.msg_type() != MessageType::Signal || msg.sender().as_deref() != Some(self.peer.as_str()) {
            return;
        }
        let (Some(path), Some(iface), Some(member)) = (msg.path(), msg.interface(), msg.member()) else {
            return;
        };
        match (&*iface, &*member) {
            (REQUEST, "Response") if self.reviewed.is_some()=>self.review_response(&path,msg,on_event),
            (REQUEST, "Response") => self.response(&path, msg, on_event),
            (IFACE, member @ ("Activated" | "Deactivated")) => {
                if self.reviewed.is_some(){return;}
                if &*path != OBJECT { return; }
                if let Ok((from, id)) = msg.read2::<Path, String>() {
                    if let Some(event) = self.phase.key_event(&from, id, member == "Activated") { on_event(event); }
                }
            }
            (IFACE, "ShortcutsChanged") => {
                if &*path != OBJECT { return; }
                if let Some(f)=self.reviewed.as_ref(){
                    if msg.read1::<Path>().is_ok_and(|session|&*session==f.plan.session.as_str()){self.abort_review("native bindings changed before receipt commit",on_event);}
                    return;
                }
                let Phase::Bound { gen, session, shortcuts, .. } = &mut self.phase else { return };
                let mut args = msg.iter_init();
                let from: Option<Path> = args.read().ok();
                if from.as_deref() != Some(session.as_str()) {
                    return;
                }
                if let Some(changed) = args.get_refarg().and_then(|a| parse_shortcuts(&*a)) {
                    *shortcuts = Some(changed.clone());
                    on_event(Event::Changed { gen: *gen, shortcuts: changed });
                }
            }
            (SESSION, "Closed") => {
                let ours = match &self.phase {
                    Phase::Binding { session, .. } | Phase::Bound { session, .. } => &*path == session.as_str(),
                    _ => false,
                };
                if ours {
                    if self.reviewed.is_some(){self.abort_review("native session closed before receipt commit",on_event);return;}
                    let gen = self.gen().unwrap_or_default();
                    self.phase = Phase::Idle;
                    on_event(Event::Failed { gen, reason: "the desktop closed the session".into() });
                }
            }
            _ => {}
        }
    }

    fn response(&mut self, path: &str, msg: &Message, on_event: &impl Fn(Event)) {
        let awaited = match &self.phase {
            Phase::Creating { request, .. } | Phase::Binding { request, .. } => request == path,
            _ => false,
        };
        if !awaited {
            return;
        }
        let answer = parse_response(msg);
        match std::mem::replace(&mut self.phase, Phase::Idle) {
            Phase::Creating { gen, session, want, .. } => {
                let results = match answer {
                    Ok(results) => results,
                    Err(code) => {
                        self.close(&session, SESSION);
                        let reason = format!("CreateSession answered {code}");
                        return on_event(Event::Failed { gen, reason });
                    }
                };
                let session = session_from_results(&results).unwrap_or(session);
                if let Err(reason) = self.bind_shortcuts(gen, session, want) {
                    self.abandon();
                    on_event(Event::Failed { gen, reason: format!("BindShortcuts: {reason}") });
                }
            }
            Phase::Binding { gen, session, want, .. } => match answer {
                Ok(results) => {
                    let shortcuts = results.get("shortcuts").and_then(|v| parse_shortcuts(&*v.0));
                    self.phase = Phase::Bound { gen, session, want, shortcuts: shortcuts.clone(), pressed: Default::default() };
                    on_event(Event::Bound { gen, shortcuts });
                }
                Err(code) => {
                    self.close(&session, SESSION);
                    let reason = if code == 1 { "the user declined".to_string() } else { format!("BindShortcuts answered {code}") };
                    on_event(Event::Failed { gen, reason });
                }
            },
            other => self.phase = other,
        }
    }
}

fn peer_disappeared(msg: &Message, peer: &str) -> bool {
    let header_matches = msg.msg_type() == MessageType::Signal
        && msg.sender().as_deref() == Some(BUS)
        && msg.path().as_deref() == Some(BUS_OBJECT)
        && msg.interface().as_deref() == Some(BUS)
        && msg.member().as_deref() == Some("NameOwnerChanged");
    if !header_matches { return false; }
    let mut args = msg.iter_init();
    matches!((args.read::<String>(), args.read::<String>(), args.read::<String>()),
        (Ok(name), Ok(old), Ok(new)) if name == peer && old == peer && new.is_empty())
        && args.arg_type() == ArgType::Invalid
}

fn method(path: &str, iface: &str, member: &str) -> Result<Message, String> {
    Message::new_method_call(DEST, path, iface, member)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coucou_bindings_become_xdg_triggers() {
        assert_eq!(xdg_trigger("Ctrl+Alt+Space").as_deref(), Some("CTRL+ALT+space"));
        assert_eq!(xdg_trigger("Ctrl+Alt+A").as_deref(), Some("CTRL+ALT+a"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Right").as_deref(), Some("CTRL+ALT+Right"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Left").as_deref(), Some("CTRL+ALT+Left"));
        assert_eq!(xdg_trigger("Super+A").as_deref(), Some("LOGO+a"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Shift+Super+K").as_deref(), Some("CTRL+ALT+SHIFT+LOGO+k"));
        assert_eq!(xdg_trigger("Ctrl+Shift+7").as_deref(), Some("CTRL+SHIFT+7"));
        assert_eq!(xdg_trigger("Alt+F5").as_deref(), Some("ALT+F5"));
        assert_eq!(xdg_trigger("Ctrl+F24").as_deref(), Some("CTRL+F24"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Enter").as_deref(), Some("CTRL+ALT+Return"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Backspace").as_deref(), Some("CTRL+ALT+BackSpace"));
        assert_eq!(xdg_trigger("Ctrl+Alt+PageDown").as_deref(), Some("CTRL+ALT+Page_Down"));
        assert_eq!(xdg_trigger("Ctrl+Alt+BracketLeft").as_deref(), Some("CTRL+ALT+bracketleft"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Quote").as_deref(), Some("CTRL+ALT+apostrophe"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Backquote").as_deref(), Some("CTRL+ALT+grave"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Equal").as_deref(), Some("CTRL+ALT+equal"));
        // Any spelling parse() takes gives the same trigger.
        assert_eq!(xdg_trigger("control+option+arrowright"), xdg_trigger("Ctrl+Alt+Right"));
        assert_eq!(xdg_trigger("Ctrl+Alt+Super+T").as_deref(), Some("CTRL+ALT+LOGO+t"));
    }

    #[test]
    fn what_is_not_a_shortcut_has_no_trigger() {
        assert_eq!(xdg_trigger(""), None);
        assert_eq!(xdg_trigger("A"), None);
        assert_eq!(xdg_trigger("Shift+A"), None);
        assert_eq!(xdg_trigger("Ctrl+Alt+Nope"), None);
        // A real key with no keysym in the table: the desktop picks.
        assert_eq!(xdg_trigger("Ctrl+Alt+Numpad5"), None);
    }

    #[test]
    fn every_default_and_every_key_coucou_records_has_a_trigger() {
        for def in crate::shortcuts::ACTIONS {
            assert!(xdg_trigger(def.default_keys).is_some(), "{}", def.id);
        }
        // KEY_NAMES in src/core/shortcuts.ts.
        let mut keys: Vec<String> = ('A'..='Z').chain('0'..='9').map(String::from).collect();
        keys.extend((1..=24).map(|n| format!("F{n}")));
        for k in [
            "Space", "Enter", "Tab", "Backspace", "Delete", "Insert", "Home", "End", "PageUp",
            "PageDown", "Up", "Down", "Left", "Right", "Comma", "Period", "Slash", "Semicolon",
            "Quote", "Backquote", "BracketLeft", "BracketRight", "Backslash", "Minus", "Equal",
        ] {
            keys.push(k.to_string());
        }
        for key in keys {
            let trigger = xdg_trigger(&format!("Ctrl+Alt+{key}"));
            assert!(trigger.as_deref().is_some_and(|t| t.starts_with("CTRL+ALT+")), "{key}: {trigger:?}");
        }
    }

    #[test]
    fn request_and_session_paths_follow_the_portal_rule() {
        assert_eq!(
            request_path(":1.42", "coucou7_1"),
            "/org/freedesktop/portal/desktop/request/1_42/coucou7_1"
        );
        assert_eq!(
            session_path(":1.42", "coucou7_2"),
            "/org/freedesktop/portal/desktop/session/1_42/coucou7_2"
        );
    }

    /// The Response signal as it comes off the bus.
    fn response(code: u32, results: PropMap) -> Message {
        let msg = Message::new_signal("/org/freedesktop/portal/desktop/request/1_42/t", REQUEST, "Response")
            .unwrap()
            .append2(code, results);
        wire(msg)
    }

    /// Through the wire format and back, so the reader sees what libdbus
    /// hands over for a message from the bus.
    fn wire(mut msg: Message) -> Message {
        msg.set_serial(1);
        let mut bytes = Vec::new();
        msg.marshal(|b| -> Result<(), ()> {
            bytes.extend_from_slice(b);
            Ok(())
        })
        .unwrap();
        Message::demarshal(&bytes).unwrap()
    }

    fn shortcut(id: &str, trigger: Option<&str>) -> (String, PropMap) {
        let mut props = PropMap::new();
        props.insert("description".into(), Variant(Box::new(format!("{id} description"))));
        if let Some(t) = trigger {
            props.insert("trigger_description".into(), Variant(Box::new(t.to_string())));
        }
        (id.to_string(), props)
    }

    #[test]
    fn a_bind_response_lists_the_shortcuts_and_their_keys() {
        let list = vec![
            shortcut("openChat", Some("Ctrl+Alt+Space")),
            shortcut("goToAlert", Some("  ")),
            shortcut("muteToggle", None),
        ];
        let mut results = PropMap::new();
        results.insert("shortcuts".into(), Variant(Box::new(list)));
        let msg = response(0, results);
        let results = parse_response(&msg).expect("a success");
        let parsed = parse_shortcuts(&*results["shortcuts"].0).expect("a list");
        assert_eq!(
            parsed,
            vec![
                Bound { id: "openChat".into(), trigger: Some("Ctrl+Alt+Space".into()) },
                Bound { id: "goToAlert".into(), trigger: None },
                Bound { id: "muteToggle".into(), trigger: None },
            ]
        );
    }

    #[test]
    fn a_refused_or_failed_response_carries_its_code() {
        assert_eq!(parse_response(&response(1, PropMap::new())).unwrap_err(), 1);
        assert_eq!(parse_response(&response(2, PropMap::new())).unwrap_err(), 2);
        // Not a Response at all.
        let other = Message::new_signal("/x", REQUEST, "Response").unwrap().append1("nope");
        assert_eq!(parse_response(&other).unwrap_err(), 2);
    }

    #[test]
    fn the_session_handle_is_read_as_a_string_or_an_object_path() {
        let path = "/org/freedesktop/portal/desktop/session/1_42/s";
        let mut as_string = PropMap::new();
        as_string.insert("session_handle".into(), Variant(Box::new(path.to_string())));
        let mut as_path = PropMap::new();
        as_path.insert("session_handle".into(), Variant(Box::new(Path::new(path).unwrap())));
        for results in [as_string, as_path] {
            let msg = response(0, results);
            let results = parse_response(&msg).unwrap();
            assert_eq!(session_from_results(&results).as_deref(), Some(path));
        }
        assert_eq!(session_from_results(&PropMap::new()), None);
    }

    #[test]
    fn shortcuts_changed_reads_the_same_list() {
        let list = vec![shortcut("nextPill", Some("Meta+N"))];
        let msg = Message::new_signal(OBJECT, IFACE, "ShortcutsChanged")
            .unwrap()
            .append2(Path::new("/s").unwrap(), list);
        let msg = wire(msg);
        let mut args = msg.iter_init();
        let _: Path = args.read().unwrap();
        let parsed = parse_shortcuts(&*args.get_refarg().unwrap()).unwrap();
        assert_eq!(parsed, vec![Bound { id: "nextPill".into(), trigger: Some("Meta+N".into()) }]);
        // Not a list: nothing.
        assert_eq!(parse_shortcuts(&"text".to_string()), None);
    }

    // ── The whole exchange, against a stand-in portal ─────────────────────────
    //
    // Needs a session bus of its own, so it doesn't run by default:
    //   dbus-run-session -- cargo test -p coucou --lib portal -- --ignored

    /// Plays org.freedesktop.portal.Desktop: answers every request at once,
    /// reports each shortcut's keys as "Fake <preferred trigger>", presses
    /// the first one, and refuses a list holding "refuseMe".
    /// Stops (and gives the name back) once `stop` is set.
    fn fake_portal(ready: mpsc::Sender<()>, stop: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        let ch = Channel::get_private(BusType::Session).unwrap();
        let name = Message::new_method_call("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "RequestName")
            .unwrap()
            .append2(DEST, 4u32);
        let owner: u32 = ch.send_with_reply_and_block(name, CALL_TIMEOUT).unwrap().read1().unwrap();
        assert_eq!(owner, 1, "the stand-in must own {DEST} on this bus");
        ready.send(()).unwrap();
        let respond = |to: &str, request: &str, code: u32, results: PropMap| {
            let mut signal = Message::new_signal(request, REQUEST, "Response").unwrap().append2(code, results);
            signal.set_destination(Some(to.into()));
            ch.send(signal).unwrap();
        };
        let (mut creates,mut binds)=(0u32,0u32);
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            let Ok(Some(msg)) = ch.blocking_pop_message(Duration::from_millis(50)) else { continue };
            if msg.msg_type() != MessageType::MethodCall {
                continue;
            }
            let sender = msg.sender().unwrap().to_string();
            let iface = msg.interface().map(|i| i.to_string()).unwrap_or_default();
            match (iface.as_str(), &*msg.member().unwrap()) {
                ("org.freedesktop.DBus.Properties", "Get") => {
                    ch.send(msg.method_return().append1(Variant(1u32))).unwrap();
                }
                (IFACE, "FixtureReleaseName") => {
                    let release = Message::new_method_call(BUS, BUS_OBJECT, BUS, "ReleaseName").unwrap().append1(DEST);
                    let status: u32 = ch.send_with_reply_and_block(release, CALL_TIMEOUT).unwrap().read1().unwrap();
                    assert_eq!(status, 1);
                    ch.send(msg.method_return()).unwrap();
                }
                (IFACE, "FixtureOperationCounts") => { ch.send(msg.method_return().append2(creates,binds)).unwrap(); }
                (IFACE, "CreateSession") => {
                    creates+=1;
                    let options: PropMap = msg.read1().unwrap();
                    let request = request_path(&sender, options["handle_token"].0.as_str().unwrap());
                    let session = session_path(&sender, options["session_handle_token"].0.as_str().unwrap());
                    let reply_path=if options["handle_token"].0.as_str().unwrap().contains("wrong_request"){"/private/unreviewed_request"}else{request.as_str()};
                    ch.send(msg.method_return().append1(Path::new(reply_path).unwrap())).unwrap();
                    let mut results = PropMap::new();
                    let returned=if options["session_handle_token"].0.as_str().unwrap().contains("wrong_session"){String::from("/private/unreviewed_session")}else{session};
                    results.insert("session_handle".into(), Variant(Box::new(returned)));
                    respond(&sender, &request, 0, results);
                }
                (IFACE, "BindShortcuts") => {
                    binds+=1;
                    let mut args = msg.iter_init();
                    let session: Path = args.read().unwrap();
                    let list: Vec<(String, PropMap)> = args.read().unwrap();
                    let _parent: String = args.read().unwrap();
                    let options: PropMap = args.read().unwrap();
                    let request = request_path(&sender, options["handle_token"].0.as_str().unwrap());
                    ch.send(msg.method_return().append1(Path::new(request.clone()).unwrap())).unwrap();
                    if list.iter().any(|(_,p)|p["description"].0.as_str()==Some("hold fixture")){continue;}
                    if list.iter().any(|(id, _)| id == "refuseMe") {
                        respond(&sender, &request, 1, PropMap::new());
                        continue;
                    }
                    let mut bound: Vec<(String, PropMap)> = list
                        .iter()
                        .filter(|(id, _)| id != "omitMe")
                        .map(|(id, props)| {
                            let wanted = props.get("preferred_trigger").and_then(|t| t.0.as_str()).unwrap_or("?");
                            let mut out = PropMap::new();
                            out.insert("description".into(), Variant(Box::new(props["description"].0.as_str().unwrap().to_string())));
                            out.insert("trigger_description".into(), Variant(Box::new(format!("Fake {wanted}"))));
                            (id.clone(), out)
                        })
                        .collect();
                    if list.iter().any(|(id,_)|id=="duplicateMe"){bound.push(shortcut("duplicateMe",None));}
                    let mut results = PropMap::new();
                    results.insert("shortcuts".into(), Variant(Box::new(bound)));
                    respond(&sender, &request, 0, results);
                    // Authenticated service, but wrong session/action: the
                    // transport still must constrain what this binding owns.
                    for (from,id) in [("/private/other_session","openChat"),(&*session,"desktopToggle")] {
                        let wrong=Message::new_signal(OBJECT,IFACE,"Activated").unwrap().append3(Path::new(from).unwrap(),id,0u64).append1(PropMap::new());ch.send(wrong).unwrap();
                    }
                    let flow_session=session.to_string();
                    // A press, broadcast as the portal does.
                    let press = Message::new_signal(OBJECT, IFACE, "Activated")
                        .unwrap()
                        .append3(session.into_static(), list[0].0.clone(), 0u64)
                        .append1(PropMap::new());
                    ch.send(press).unwrap();
                    if list[0].0 == "flowDictate" {
                        for (member,id) in [("Deactivated","flowDictate"),("Activated","flowCancel"),("Activated","flowFinish"),("Activated","voiceTalk"),("Activated","flowRelease")] {
                            let signal=Message::new_signal(OBJECT,IFACE,member).unwrap().append3(Path::new(flow_session.as_str()).unwrap(),id,0u64).append1(PropMap::new());ch.send(signal).unwrap();
                        }
                    }
                }
                _ => {
                    ch.send(msg.method_return()).unwrap();
                }
            }
        }
    }

    fn phase_fixture() -> Phase { Phase::Bound { gen: 7, session: "/private/session".into(), want: vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None},Wanted{id:"goToAlert".into(),description:"fixture".into(),trigger:None}], shortcuts:Some(vec![Bound{id:"openChat".into(),trigger:None}]), pressed:Default::default() } }
    #[test]fn owned_key_events_require_session_requested_and_accepted_id_and_preserve_release(){
        let mut phase=phase_fixture();assert_eq!(phase.key_event("/other/session","openChat".into(),true),None);assert_eq!(phase.key_event("/private/session","desktopToggle".into(),true),None);assert_eq!(phase.key_event("/private/session","goToAlert".into(),true),None);assert_eq!(phase.key_event("/private/session","openChat".into(),false),None);
        assert_eq!(phase.key_event("/private/session","openChat".into(),true),Some(Event::Activated{gen:7,id:"openChat".into()}));assert_eq!(phase.key_event("/private/session","openChat".into(),true),None);
        if let Phase::Bound{shortcuts,..}=&mut phase{*shortcuts=Some(vec![]);}assert_eq!(phase.key_event("/private/session","openChat".into(),false),Some(Event::Deactivated{gen:7,id:"openChat".into()}));assert_eq!(phase.key_event("/private/session","openChat".into(),false),None);assert_eq!(phase.key_event("/private/session","openChat".into(),true),None);
    }
    #[test]fn release_or_new_session_cannot_relabel_queued_key_events(){
        let mut phase=phase_fixture();assert!(phase.key_event("/private/session","openChat".into(),true).is_some());phase=Phase::Idle;assert_eq!(phase.key_event("/private/session","openChat".into(),false),None);phase=phase_fixture();if let Phase::Bound{gen,session,..}=&mut phase{*gen=8;*session="/private/new_session".into();}assert_eq!(phase.key_event("/private/session","openChat".into(),true),None);assert_eq!(phase.key_event("/private/new_session","openChat".into(),true),Some(Event::Activated{gen:8,id:"openChat".into()}));
    }
    #[test]fn an_omitted_native_list_only_accepts_requested_ids(){let mut phase=phase_fixture();if let Phase::Bound{shortcuts,..}=&mut phase{*shortcuts=None;}assert_eq!(phase.key_event("/private/session","desktopToggle".into(),true),None);assert_eq!(phase.key_event("/private/session","goToAlert".into(),true),Some(Event::Activated{gen:7,id:"goToAlert".into()}));}

    include!("portal_review_tests.rs");

    static BUS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    struct StandIn { stop: std::sync::Arc<std::sync::atomic::AtomicBool>, thread: Option<std::thread::JoinHandle<()>> }
    impl StandIn { fn new()->Self{let(tx,rx)=mpsc::channel();let stop=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));let flag=stop.clone();let thread=std::thread::spawn(move||fake_portal(tx,flag));rx.recv_timeout(Duration::from_secs(15)).unwrap();Self{stop,thread:Some(thread)}} }
    impl Drop for StandIn { fn drop(&mut self){self.stop.store(true,std::sync::atomic::Ordering::Relaxed);if let Some(thread)=self.thread.take(){thread.join().unwrap();}} }

    #[test]
    fn owner_loss_requires_the_exact_daemon_header_and_unique_peer_transition() {
        let peer = ":1.42";
        for (sender, path, iface, member, name, old, new, expected) in [
            (BUS, BUS_OBJECT, BUS, "NameOwnerChanged", peer, peer, "", true),
            (":1.99", BUS_OBJECT, BUS, "NameOwnerChanged", peer, peer, "", false),
            (BUS, OBJECT, BUS, "NameOwnerChanged", peer, peer, "", false),
            (BUS, BUS_OBJECT, IFACE, "NameOwnerChanged", peer, peer, "", false),
            (BUS, BUS_OBJECT, BUS, "NameLost", peer, peer, "", false),
            (BUS, BUS_OBJECT, BUS, "NameOwnerChanged", DEST, peer, ":1.99", false),
            (BUS, BUS_OBJECT, BUS, "NameOwnerChanged", ":1.99", ":1.99", "", false),
            (BUS, BUS_OBJECT, BUS, "NameOwnerChanged", peer, "", peer, false),
            (BUS, BUS_OBJECT, BUS, "NameOwnerChanged", peer, ":1.99", "", false),
        ] {
            let mut msg = Message::new_signal(path, iface, member).unwrap().append3(name, old, new);
            msg.set_sender(Some(sender.into()));
            assert_eq!(peer_disappeared(&wire(msg), peer), expected);
        }
        for extra in [false, true] {
            let mut msg = Message::new_signal(BUS_OBJECT, BUS, "NameOwnerChanged").unwrap();
            msg.set_sender(Some(BUS.into()));
            let msg = if extra { msg.append3(peer, peer, "").append1(1u32) } else { msg.append2(peer, peer) };
            assert!(!peer_disappeared(&wire(msg), peer), "malformed daemon body accepted");
        }
        let mut msg = Message::new_method_call(DEST, BUS_OBJECT, BUS, "NameOwnerChanged").unwrap().append3(peer, peer, "");
        msg.set_sender(Some(BUS.into()));
        assert!(!peer_disappeared(&wire(msg), peer));
    }

    fn pump_until(portal: &mut Portal, observed: &std::cell::RefCell<Vec<Event>>, done: impl Fn(&[Event]) -> bool) {
        let until = std::time::Instant::now() + Duration::from_secs(3);
        while !done(&observed.borrow()) && std::time::Instant::now() < until {
            portal.channel.read_write(Some(Duration::from_millis(10))).unwrap();
            while let Some(message) = portal.channel.pop_message() {
                portal.message(&message, &|event| observed.borrow_mut().push(event));
            }
        }
        assert!(done(&observed.borrow()), "expected bus event not observed: {:?}", observed.borrow());
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn directed_forged_owner_loss_is_delivered_but_cannot_invalidate_the_live_peer() {
        let _bus = BUS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _fake = StandIn::new();
        let mut portal = Portal::connect().unwrap();
        let attacker = Channel::get_private(BusType::Session).unwrap();
        let rogue = attacker.unique_name().unwrap().to_string();
        let mut signal = Message::new_signal(BUS_OBJECT, BUS, "NameOwnerChanged").unwrap().append3(portal.peer.clone(), portal.peer.clone(), "");
        signal.set_destination(Some(portal.sender.as_str().into()));
        attacker.send(signal).unwrap(); attacker.flush();
        let observed = std::cell::RefCell::new(Vec::new());
        let until = std::time::Instant::now() + Duration::from_secs(3);
        let mut delivered = false;
        while !delivered && std::time::Instant::now() < until {
            portal.channel.read_write(Some(Duration::from_millis(10))).unwrap();
            while let Some(message) = portal.channel.pop_message() {
                delivered |= message.sender().as_deref() == Some(rogue.as_str());
                portal.message(&message, &|event| observed.borrow_mut().push(event));
            }
        }
        assert!(delivered, "directed hostile notification must actually reach the client");
        assert!(portal.peer_alive && observed.borrow().is_empty());
        portal.require_live_peer().unwrap();
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn service_name_replacement_keeps_live_peer_but_peer_death_invalidates_once() {
        let _bus = BUS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let original = StandIn::new();
        let mut portal = Portal::connect().unwrap();
        let peer = portal.peer.clone();
        portal.call(method(OBJECT, IFACE, "FixtureReleaseName").unwrap()).unwrap();
        let _replacement = StandIn::new();
        let fresh = Portal::connect().unwrap();
        assert_ne!(peer, fresh.peer);
        portal.require_live_peer().unwrap();
        let observed = std::cell::RefCell::new(Vec::new());
        portal.command(Command::Bind { gen:1, shortcuts:vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None}] }, &|event| observed.borrow_mut().push(event));
        pump_until(&mut portal, &observed, |events| events.iter().any(|e| matches!(e, Event::Activated{gen:1,..})));
        assert!(!observed.borrow().iter().any(|e| matches!(e, Event::Unavailable(_))));
        assert_eq!(portal.peer, peer);
        drop(original);
        assert!(portal.require_live_peer().is_err(), "post-subscription liveness must detect a vanished peer");
        pump_until(&mut portal, &observed, |events| events.iter().any(|e| matches!(e, Event::Unavailable(_))));
        assert!(!portal.peer_alive && matches!(portal.phase, Phase::Idle));
        assert!(portal.create_session(2, vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None}]).is_err());
        assert_eq!(portal.peer, peer, "do not adopt the replacement");
        assert_eq!(observed.borrow().iter().filter(|e| matches!(e, Event::Unavailable(_))).count(), 1);
        fresh.require_live_peer().unwrap();
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session; no audio"]
    fn idle_worker_exits_on_peer_loss_and_flow_cancels_without_another_command() {
        let _bus = BUS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let fake = StandIn::new();
        let (events, received) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let (wake_rx, wake_tx) = UnixStream::pair().unwrap();
        wake_rx.set_nonblocking(true).unwrap(); wake_tx.set_nonblocking(true).unwrap();
        let worker = std::thread::spawn(move || run(rx, wake_rx, move |e| { let _ = events.send(e); }));
        let handle = Handle { tx, wake:wake_tx };
        assert!(handle.bind(1, vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None}]));
        assert!(matches!(received.recv_timeout(Duration::from_secs(3)).unwrap(), Event::Bound{gen:1,..}));
        assert!(matches!(received.recv_timeout(Duration::from_secs(3)).unwrap(), Event::Activated{gen:1,..}));
        let (flow_tx, flow_rx) = mpsc::channel();
        let flow = crate::flow::wayland::listen(flow_tx).unwrap();
        for expected in [crate::flow::KeyEvent::Down, crate::flow::KeyEvent::Up, crate::flow::KeyEvent::Cancel, crate::flow::KeyEvent::Finish, crate::flow::KeyEvent::Voice, crate::flow::KeyEvent::Up] {
            assert_eq!(flow_rx.recv_timeout(Duration::from_secs(3)).unwrap().0, expected);
        }
        drop(fake);
        assert!(matches!(received.recv_timeout(Duration::from_secs(3)).unwrap(), Event::Unavailable(_)));
        worker.join().unwrap();
        assert!(!handle.release(), "worker receiver must be gone after join");
        assert_eq!(flow_rx.recv_timeout(Duration::from_secs(3)).unwrap().0, crate::flow::KeyEvent::Cancel);
        assert!(matches!(flow_rx.recv_timeout(Duration::from_secs(3)), Err(mpsc::RecvTimeoutError::Disconnected)));
        // The bus remains alive: only the unique service peer disappeared.
        Channel::get_private(BusType::Session).unwrap();
        drop(flow);
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn native_reservation_uses_existing_live_transport_without_registering_or_releasing_keys() {
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();
        let (tx,rx)=mpsc::channel();let handle=start(move|event|{let _=tx.send(event);}).unwrap();
        let wanted=vec![Wanted{id:"openChat".into(),description:"private reservation fixture".into(),trigger:Some("CTRL+ALT+space".into())}];
        assert!(handle.bind(1,wanted.clone()));assert!(matches!(rx.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:1,..}));assert!(matches!(rx.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Activated{gen:1,..}));
        let first=handle.reserve(2,wanted.clone()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
        let second=handle.reserve(2,wanted.clone()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
        assert_eq!(first.peer,second.peer);assert_eq!(first.client,second.client);assert_ne!(first.session,second.session);assert_ne!(first.create_request,first.bind_request);assert_ne!(first.create_request,second.create_request);assert_eq!(first.shortcuts,wanted);
        let serialized=serde_json::to_value(&first).unwrap();assert_eq!(serialized["generation"],"2");
        let observer=Channel::get_private(BusType::Session).unwrap();let counts=observer.send_with_reply_and_block(method(OBJECT,IFACE,"FixtureOperationCounts").unwrap(),CALL_TIMEOUT).unwrap().read2::<u32,u32>().unwrap();assert_eq!(counts,(1,1),"reservation must not create or rebind a native session");
        assert!(rx.try_recv().is_err(),"reservation must not publish a binding event");
        assert!(handle.reserve(1,wanted.clone()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().is_err());
        let duplicate=vec![wanted[0].clone(),wanted[0].clone()];assert!(handle.reserve(2,duplicate).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().is_err());
        assert!(handle.bind(1,wanted));assert!(matches!(rx.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:1,..}),"the original bound generation remains live");
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn directed_signal_from_other_bus_peer_is_not_a_portal_activation() {
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let mut portal=Portal::connect().unwrap();
        portal.phase=Phase::Bound{gen:1,session:"/private/authorized_session".into(),want:vec![Wanted{id:"openChat".into(),description:"private fixture".into(),trigger:None}],shortcuts:Some(vec![Bound{id:"openChat".into(),trigger:None}]),pressed:Default::default()};
        let attacker=Channel::get_private(BusType::Session).unwrap();let rogue=attacker.unique_name().unwrap().to_string();
        let mut message=Message::new_signal(OBJECT,IFACE,"Activated").unwrap().append3(Path::new("/private/authorized_session").unwrap(),"openChat",0u64).append1(PropMap::new());message.set_destination(Some(portal.sender.as_str().into()));attacker.send(message).unwrap();attacker.flush();
        let observed=std::cell::RefCell::new(Vec::new());let mut delivered=false;let until=std::time::Instant::now()+Duration::from_secs(3);
        while !delivered&&std::time::Instant::now()<until{portal.channel.read_write(Some(Duration::from_millis(10))).unwrap();while let Some(message)=portal.channel.pop_message(){let ours=message.sender().is_some_and(|s|&*s==rogue.as_str());portal.message(&message,&|event|observed.borrow_mut().push(event));delivered|=ours;}}
        assert!(delivered,"the private bus must actually deliver the directed rogue message");assert!(observed.borrow().is_empty(),"another bus peer injected a native activation: {:?}",observed.borrow());
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn directed_response_from_other_bus_peer_cannot_bind_a_session(){
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let mut portal=Portal::connect().unwrap();let request="/private/awaited_request";
        portal.phase=Phase::Creating{gen:1,request:request.into(),session:"/private/original_session".into(),want:vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None}]};
        let attacker=Channel::get_private(BusType::Session).unwrap();let rogue=attacker.unique_name().unwrap().to_string();let mut results=PropMap::new();results.insert("session_handle".into(),Variant(Box::new("/private/forged_session".to_string())));let mut message=Message::new_signal(request,REQUEST,"Response").unwrap().append2(0u32,results);message.set_destination(Some(portal.sender.as_str().into()));attacker.send(message).unwrap();attacker.flush();
        let observed=std::cell::RefCell::new(Vec::new());let mut delivered=false;let until=std::time::Instant::now()+Duration::from_secs(3);while !delivered&&std::time::Instant::now()<until{portal.channel.read_write(Some(Duration::from_millis(10))).unwrap();while let Some(message)=portal.channel.pop_message(){let ours=message.sender().is_some_and(|s|&*s==rogue.as_str());portal.message(&message,&|event|observed.borrow_mut().push(event));delivered|=ours;}}
        assert!(delivered);assert!(observed.borrow().is_empty());assert!(matches!(&portal.phase,Phase::Creating{session,..}if session=="/private/original_session"),"a directed forged approval must not advance native binding");
    }
    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session"]
    fn replacement_service_does_not_inherit_an_existing_portal_handle(){
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let original=StandIn::new();let mut portal=Portal::connect().unwrap();let peer=portal.peer.clone();drop(original);let _replacement=StandIn::new();let replacement=Portal::connect().unwrap();assert_ne!(peer,replacement.peer);assert!(portal.create_session(1,vec![Wanted{id:"openChat".into(),description:"fixture".into(),trigger:None}]).is_err(),"old handle silently followed the replacement service");assert_eq!(portal.peer,peer);assert!(matches!(portal.phase,Phase::Idle));
    }

    #[test]
    #[ignore = "synthetic protocol, needs a private dbus-run-session; no audio"]
    fn shared_portal_preserves_flow_press_release_cancel_and_finish_without_audio(){
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,rx)=mpsc::channel();let hold=crate::flow::wayland::listen(tx).unwrap();for expected in [crate::flow::KeyEvent::Down,crate::flow::KeyEvent::Up,crate::flow::KeyEvent::Cancel,crate::flow::KeyEvent::Finish,crate::flow::KeyEvent::Voice,crate::flow::KeyEvent::Up]{assert_eq!(rx.recv_timeout(Duration::from_secs(15)).unwrap().0,expected);}drop(hold);
    }

    #[test]
    #[ignore = "needs its own session bus: dbus-run-session -- cargo test -p coucou --lib portal -- --ignored"]
    fn the_whole_exchange_with_a_stand_in_portal() {
        let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());
        let recv = |rx: &mpsc::Receiver<Event>| rx.recv_timeout(Duration::from_secs(15)).expect("an event");

        let (ready_tx, ready_rx) = mpsc::channel();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fake = {
            let stop = stop.clone();
            std::thread::spawn(move || fake_portal(ready_tx, stop))
        };
        ready_rx.recv_timeout(Duration::from_secs(15)).unwrap();

        let (tx, rx) = mpsc::channel();
        let handle = start(move |e| drop(tx.send(e))).unwrap();
        let want = |ids: &[(&str, &str)]| -> Vec<Wanted> {
            ids.iter()
                .map(|(id, keys)| Wanted { id: id.to_string(), description: format!("{id}!"), trigger: xdg_trigger(keys) })
                .collect()
        };
        let bound = |ids: &[(&str, &str)]| -> Option<Vec<Bound>> {
            Some(ids.iter().map(|(id, t)| Bound { id: id.to_string(), trigger: Some(t.to_string()) }).collect())
        };

        let first = want(&[("openChat", "Ctrl+Alt+Space"), ("goToAlert", "Ctrl+Alt+A")]);
        assert!(handle.bind(1, first.clone()));
        let answer = bound(&[("openChat", "Fake CTRL+ALT+space"), ("goToAlert", "Fake CTRL+ALT+a")]);
        assert_eq!(recv(&rx), Event::Bound { gen: 1, shortcuts: answer.clone() });
        assert_eq!(recv(&rx), Event::Activated { gen: 1, id: "openChat".into() });

        // A replay only consults the same live generation; a genuinely new
        // generation gets a fresh native session even for identical keys.
        assert!(handle.bind(1, first.clone()));
        assert_eq!(recv(&rx), Event::Bound { gen: 1, shortcuts: answer.clone() });
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
        assert!(handle.bind(2, first));
        assert_eq!(recv(&rx), Event::Bound { gen: 2, shortcuts: answer });
        assert_eq!(recv(&rx), Event::Activated { gen: 2, id: "openChat".into() });

        // Other shortcuts: a new session.
        assert!(handle.bind(3, want(&[("muteToggle", "Super+M")])));
        assert_eq!(recv(&rx), Event::Bound { gen: 3, shortcuts: bound(&[("muteToggle", "Fake LOGO+m")]) });
        assert_eq!(recv(&rx), Event::Activated { gen: 3, id: "muteToggle".into() });

        // Refused in the desktop's dialog.
        assert!(handle.bind(4, want(&[("refuseMe", "Ctrl+Alt+R")])));
        assert!(matches!(recv(&rx), Event::Failed { gen: 4, .. }));

        // Released, then bound again from nothing.
        assert!(handle.release());
        assert!(handle.bind(5, want(&[("nextPill", "Ctrl+Alt+Right")])));
        assert_eq!(recv(&rx), Event::Bound { gen: 5, shortcuts: bound(&[("nextPill", "Fake CTRL+ALT+Right")]) });
        assert_eq!(recv(&rx), Event::Activated { gen: 5, id: "nextPill".into() });
        assert!(handle.bind(4, want(&[("openChat", "Ctrl+Alt+Space")])));
        assert!(matches!(recv(&rx), Event::Failed { gen: 4, .. }));
        assert!(handle.release());
        assert!(handle.bind(5, want(&[("nextPill", "Ctrl+Alt+Right")])));
        assert!(matches!(recv(&rx), Event::Failed { gen: 5, .. }));
        // The stand-in omits one requested key, then sends a hostile press of
        // that key anyway. It must not manufacture an activation.
        assert!(handle.bind(6, want(&[("omitMe", "Ctrl+Alt+O"), ("openChat", "Ctrl+Alt+Space")])));
        assert_eq!(recv(&rx), Event::Bound { gen: 6, shortcuts: bound(&[("openChat", "Fake CTRL+ALT+space")]) });
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err(), "unrequested/refused/other-session signal leaked");

        // No GlobalShortcuts portal on the bus (none at all, or one without
        // that interface): Unavailable, and the thread is gone.
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        fake.join().unwrap();
        let (tx, rx) = mpsc::channel();
        let handle = start(move |e| drop(tx.send(e))).unwrap();
        assert!(matches!(recv(&rx), Event::Unavailable(_)));
        let gone = (0..100).any(|_| {
            std::thread::sleep(Duration::from_millis(10));
            !handle.release()
        });
        assert!(gone, "the thread should stop when there is no portal");
    }
}
