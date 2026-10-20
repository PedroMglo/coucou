//! Inactive, opt-in consumption hooks. Hooks never read/write files or wait for
//! the ledger actor. Explicit activation/shutdown belong to backend bootstrap.
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::usage::{self, AttemptKey, AttemptOutcome, AttemptStart, Completeness,
    CoverageStatus, Ledger, LedgerError, PersistenceState, ServerCounters,
    ServerSnapshot, UsageSummary};

const QUEUE_CAPACITY: usize = 1024;
const MAX_GUARDS: usize = usage::MAX_ATTEMPTS;
const ACTOR_POLL: Duration = Duration::from_millis(25);
static ACTIVE: RwLock<Option<Weak<Shared>>> = RwLock::new(None);
static ACTIVATING: Mutex<()> = Mutex::new(());
// A hook unable to inspect the tiny registry cannot attribute its missed event.
// Keep that gap for the process lifetime rather than silently claiming coverage.
static REGISTRY_GAP: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub struct Origin {
    pub provider: String,
    pub owner: String,
    pub conversation: String,
}

pub struct Activation {
    pub path: PathBuf,
    pub ledger_id: String,
}

/// Prepared, inactive bootstrap state. No registry entry, actor or MutexGuard.
/// Drop only releases memory/the already-synced owner lease; it never commits.
pub struct Prepared {
    ledger: Ledger,
    path: PathBuf,
    owner_lock: File,
    nonce: String,
    published: Published,
}

/// Application-owned non-Clone lifetime. Dropping requests a drain, never joins.
pub struct Service { shared: Arc<Shared> }

#[derive(Clone, Debug)]
pub struct ShutdownReport {
    pub stopped: bool,
    pub pending_guards: usize,
    /// Includes an event currently being applied/committed by the actor.
    pub queued: usize,
    pub incomplete: bool,
    pub issue: Option<LedgerError>,
}

struct Published { provider: UsageSummary, owner: UsageSummary }
struct Shared {
    sender: SyncSender<Event>,
    published: RwLock<Published>,
    admission: Mutex<()>,
    accepting: AtomicBool,
    closing: AtomicBool,
    stopped: AtomicBool,
    queued: AtomicUsize,
    guards: AtomicUsize,
    gap: AtomicBool,
    issue: AtomicU8,
    nonce: String,
    next: AtomicU64,
}

enum Event {
    Begin(AttemptStart),
    Observe { key: AttemptKey, snapshot: ServerSnapshot },
    Finish { key: AttemptKey, snapshot: Option<ServerSnapshot>, outcome: AttemptOutcome, at: u64 },
}

/// No response/prompt bytes or frontend ownership lookups are retained.
pub struct Capture {
    shared: Option<Arc<Shared>>,
    key: Option<AttemptKey>,
    snapshot: Option<ServerSnapshot>,
    snapshot_sent: bool,
    complete: bool,
    terminal: Option<(AttemptOutcome, u64)>,
    finish_sent: bool,
}

impl Capture {
    fn inactive() -> Self {
        Self { shared: None, key: None, snapshot: None, snapshot_sent: false,
            complete: false, terminal: None, finish_sent: false }
    }

    pub fn begin(origin: Option<&Origin>, requested_model: &str) -> Self {
        // Legacy/custom/other-backend wrappers legitimately have no origin.
        // They must not inspect or poison an unrelated active Ollama service.
        let Some(origin) = origin else { return Self::inactive() };
        let Some(shared) = current(true) else { return Self::inactive() };
        if origin.provider != "ollama" || origin.owner != "ollama" || !valid_id(&origin.conversation) {
            shared.fail(LedgerError::InvalidInput); return Self::inactive();
        }
        if !valid_model(requested_model) {
            shared.fail(LedgerError::InvalidInput); return Self::inactive();
        }
        let at = match usage::now_seconds() {
            Ok(at) => at,
            Err(error) => { shared.fail(error); return Self::inactive() }
        };
        // Only bounded compact strings are retained, irrespective of the
        // caller's String capacity. Admission never includes filesystem IO.
        let conversation = origin.conversation.as_str().to_owned();
        let model = requested_model.to_owned();
        let guard = match shared.admission.try_lock() {
            Ok(guard) => guard,
            Err(_) => { shared.fail(LedgerError::Busy); return Self::inactive() }
        };
        if !shared.accepting.load(Ordering::Acquire) || shared.stopped.load(Ordering::Acquire) {
            shared.fail(LedgerError::NotFound); return Self::inactive();
        }
        if shared.guards.load(Ordering::Acquire) >= MAX_GUARDS {
            shared.fail(LedgerError::Limit); return Self::inactive();
        }
        let ordinal = match next_id(&shared.next) {
            Ok(id) => id,
            Err(error) => { shared.fail(error); return Self::inactive() }
        };
        let key = AttemptKey { provider: "ollama".into(), owner: "ollama".into(),
            conversation, attempt: format!("{}:{ordinal}", shared.nonce) };
        shared.guards.fetch_add(1, Ordering::AcqRel);
        let admitted = shared.enqueue(Event::Begin(AttemptStart { key: key.clone(),
            started_at: at, server_id: None, requested_model: Some(model) }));
        if !admitted {
            shared.guards.fetch_sub(1, Ordering::AcqRel);
            return Self::inactive();
        }
        drop(guard);
        Self { shared: Some(shared), key: Some(key), snapshot: None,
            snapshot_sent: false, complete: false, terminal: None, finish_sent: false }
    }

    pub fn observe_oai(&mut self, event: &Value) {
        if self.key.is_none() { return; }
        let Some(object) = event.get("usage").and_then(Value::as_object) else { return };
        let counters = ServerCounters {
            input_tokens: counter_alias(object, "prompt_tokens", "input_tokens"),
            output_tokens: counter_alias(object, "completion_tokens", "output_tokens"),
            total_tokens: object.get("total_tokens").and_then(Value::as_u64),
        };
        self.observe(counters, reported_model(event), self.complete);
    }

    pub fn observe_native(&mut self, response: &Value) {
        if self.key.is_none() { return; }
        let Some(object) = response.as_object() else { return };
        let done = response.get("done").and_then(Value::as_bool) == Some(true);
        if done { self.complete = true; }
        // A complete body without either recognized counter is not a measured
        // zero. Present-but-invalid counters become absent/unknown dimensions.
        if !object.contains_key("prompt_eval_count") && !object.contains_key("eval_count") {
            if done { self.transport_complete(); }
            return;
        }
        let counters = ServerCounters {
            input_tokens: object.get("prompt_eval_count").and_then(Value::as_u64),
            output_tokens: object.get("eval_count").and_then(Value::as_u64),
            total_tokens: None,
        };
        self.observe(counters, reported_model(response), self.complete);
    }

    pub fn transport_complete(&mut self) {
        self.complete = true;
        // Do not invent a snapshot when the transport supplied no usage.
        if let Some(snapshot) = &self.snapshot {
            self.observe(snapshot.counters, snapshot.reported_model.clone(), true);
        }
    }

    fn observe(&mut self, counters: ServerCounters, model: Option<String>, final_: bool) {
        let (Some(shared), Some(key)) = (&self.shared, &self.key) else { return };
        let completeness = if final_ { Completeness::Final } else { Completeness::Partial };
        let same = self.snapshot.as_ref().is_some_and(|old| old.counters == counters &&
            old.reported_model == model && old.completeness == completeness);
        if same && self.snapshot_sent { return; }
        if !same {
            let sequence = match self.snapshot.as_ref().map_or(Some(1), |s| s.sequence.checked_add(1)) {
                Some(sequence) => sequence,
                None => { shared.fail(LedgerError::Limit); return }
            };
            let observed_at = match usage::now_seconds() {
                Ok(at) => at,
                Err(error) => { shared.fail(error); return }
            };
            self.snapshot = Some(ServerSnapshot { sequence, observed_at, completeness,
                counters, reported_model: model });
        }
        if let Some(snapshot) = &self.snapshot {
            self.snapshot_sent = shared.enqueue(Event::Observe { key: key.clone(), snapshot: snapshot.clone() });
            if !self.snapshot_sent {
                // An earlier terminal event may already have been delivered.
                // Re-send that SAME outcome/time with the latest snapshot from
                // finish/Drop instead of silently losing a late queue failure.
                self.finish_sent = false;
            }
        }
    }

    pub fn finish(&mut self, outcome: AttemptOutcome) {
        let Some(shared) = &self.shared else { return };
        if outcome == AttemptOutcome::Pending {
            shared.fail(LedgerError::InvalidInput); return;
        }
        if self.terminal.is_none() {
            let at = match usage::now_seconds() {
                Ok(at) => at,
                Err(error) => { shared.fail(error); return }
            };
            self.terminal = Some((outcome, at));
        }
        self.flush_finish();
    }

    fn flush_finish(&mut self) {
        if self.finish_sent { return; }
        if let (Some(shared), Some(key), Some((outcome, at))) = (&self.shared, &self.key, self.terminal) {
            // Carry the last snapshot as well: a previously full observation
            // queue can recover it here. Ledger.observe is sequence-idempotent.
            self.finish_sent = shared.enqueue(Event::Finish { key: key.clone(),
                snapshot: self.snapshot.clone(), outcome, at });
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if self.key.is_none() { return; }
        if self.terminal.is_none() { self.finish(AttemptOutcome::Interrupted); }
        else { self.flush_finish(); }
        if let Some(shared) = &self.shared { shared.guards.fetch_sub(1, Ordering::AcqRel); }
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|b|
        b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}
fn valid_model(value: &str) -> bool {
    !value.is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}
fn reported_model(value: &Value) -> Option<String> {
    value.get("model").and_then(Value::as_str).filter(|v| valid_model(v)).map(str::to_owned)
}
fn counter_alias(object: &serde_json::Map<String, Value>, a: &str, b: &str) -> Option<u64> {
    match (object.get(a), object.get(b)) {
        (Some(a), Some(b)) => a.as_u64().filter(|v| b.as_u64() == Some(*v)),
        (Some(value), None) | (None, Some(value)) => value.as_u64(),
        (None, None) => None,
    }
}
fn next_id(counter: &AtomicU64) -> Result<u64, LedgerError> {
    let mut old = counter.load(Ordering::Relaxed);
    for _ in 0..16 {
        let next = old.checked_add(1).ok_or(LedgerError::Limit)?;
        match counter.compare_exchange_weak(old, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Ok(next),
            Err(current) => old = current,
        }
    }
    Err(LedgerError::Busy)
}

impl Shared {
    fn fail(&self, error: LedgerError) {
        self.gap.store(true, Ordering::Release);
        // First failure remains visible; durability uncertainty takes priority.
        let code = error_code(error);
        if error == LedgerError::DurabilityUnknown { self.issue.store(code, Ordering::Release); }
        else { let _ = self.issue.compare_exchange(0, code, Ordering::AcqRel, Ordering::Acquire); }
    }
    fn enqueue(&self, event: Event) -> bool {
        self.queued.fetch_add(1, Ordering::AcqRel);
        match self.sender.try_send(event) {
            Ok(()) => true,
            Err(error) => {
                self.queued.fetch_sub(1, Ordering::AcqRel);
                self.fail(match error { TrySendError::Full(_) => LedgerError::Limit,
                    TrySendError::Disconnected(_) => LedgerError::NotFound });
                false
            }
        }
    }
    fn request_shutdown(&self) {
        // Admission sections never include IO; this is backend lifetime control,
        // not an inference hook. Serialize close with registration/counting.
        let _guard = self.admission.lock().unwrap_or_else(|e| e.into_inner());
        self.accepting.store(false, Ordering::Release);
        self.closing.store(true, Ordering::Release);
    }
}

fn current(capture_hook: bool) -> Option<Arc<Shared>> {
    match ACTIVE.try_read() {
        Ok(slot) => slot.as_ref().and_then(Weak::upgrade),
        Err(_) => {
            if capture_hook { REGISTRY_GAP.store(true, Ordering::Release); }
            None
        }
    }
}

fn contextual_unavailable(provider: &str, owner: Option<&str>) -> UsageSummary {
    let mut summary = usage::unavailable();
    summary.provider = valid_id(provider).then(|| provider.to_owned());
    summary.owner = owner.filter(|v| valid_id(v)).map(str::to_owned);
    summary
}

/// Cached-only read. No path lookup, clock, foreground owner or ledger IO.
pub fn summarize(provider: &str, owner: Option<&str>) -> UsageSummary {
    match current(false) {
        Some(shared) => snapshot(&shared, provider, owner),
        None => contextual_unavailable(provider, owner),
    }
}

/// A known Ollama route lost attribution (e.g. Chat's try_lock failed). This is
/// separate from a legitimate begin(None), and never activates a service.
pub fn mark_untracked(provider: &str, owner: &str, issue: LedgerError) {
    if provider != "ollama" || owner != "ollama" { return; }
    let Some(shared) = current(false) else { return };
    if shared.accepting.load(Ordering::Acquire) && !shared.stopped.load(Ordering::Acquire) {
        shared.fail(issue);
    }
}

fn snapshot(shared: &Shared, provider: &str, owner: Option<&str>) -> UsageSummary {
    if provider != "ollama" || owner.is_some_and(|o| o != "ollama") {
        return contextual_unavailable(provider, owner);
    }
    let mut summary = match shared.published.try_read() {
        Ok(published) => if owner.is_some() { published.owner.clone() } else { published.provider.clone() },
        Err(std::sync::TryLockError::Poisoned(error)) => {
            shared.fail(LedgerError::Io);
            let published = error.into_inner();
            if owner.is_some() { published.owner.clone() } else { published.provider.clone() }
        }
        Err(std::sync::TryLockError::WouldBlock) => {
            let mut value = contextual_unavailable(provider, owner);
            value.coverage.status = CoverageStatus::Partial;
            value.persistence.state = PersistenceState::Unknown;
            value.persistence.issue = Some(LedgerError::Busy);
            return value;
        }
    };
    let closing = shared.closing.load(Ordering::Acquire) && !shared.stopped.load(Ordering::Acquire);
    let pending = shared.queued.load(Ordering::Acquire) != 0 || shared.guards.load(Ordering::Acquire) != 0;
    let gap = shared.gap.load(Ordering::Acquire) || REGISTRY_GAP.load(Ordering::Acquire);
    if gap || pending || closing {
        summary.coverage.status = CoverageStatus::Partial;
        summary.input.coverage = CoverageStatus::Partial;
        summary.output.coverage = CoverageStatus::Partial;
        summary.total.coverage = CoverageStatus::Partial;
    }
    if gap || closing {
        summary.persistence.state = PersistenceState::Unknown;
        summary.persistence.issue = decode_error(shared.issue.load(Ordering::Acquire))
            .or(Some(if closing { LedgerError::DurabilityUnknown } else { LedgerError::Busy }));
    } else if pending {
        summary.persistence.state = PersistenceState::Dirty;
    }
    summary
}

impl Service {
    pub fn summarize(&self, provider: &str, owner: Option<&str>) -> UsageSummary {
        snapshot(&self.shared, provider, owner)
    }
    /// Bounds caller waiting only. A blocked fsync/IO is not cancellable here.
    pub fn shutdown(&self, timeout: Duration) -> ShutdownReport {
        self.shared.request_shutdown();
        let deadline = Instant::now().checked_add(timeout);
        while !self.shared.stopped.load(Ordering::Acquire) {
            let Some(remaining) = deadline.and_then(|d| d.checked_duration_since(Instant::now())) else { break };
            if remaining.is_zero() { break; }
            std::thread::sleep(remaining.min(Duration::from_millis(10)));
        }
        let stopped = self.shared.stopped.load(Ordering::Acquire);
        let pending_guards = self.shared.guards.load(Ordering::Acquire);
        let queued = self.shared.queued.load(Ordering::Acquire);
        let incomplete = !stopped || pending_guards != 0 || queued != 0 ||
            self.shared.gap.load(Ordering::Acquire) || REGISTRY_GAP.load(Ordering::Acquire);
        ShutdownReport { stopped, pending_guards, queued, incomplete,
            issue: decode_error(self.shared.issue.load(Ordering::Acquire))
                .or((!stopped).then_some(LedgerError::DurabilityUnknown)) }
    }
}
impl Drop for Service { fn drop(&mut self) { self.shared.request_shutdown(); } }

fn publish(shared: &Shared, ledger: &Ledger) {
    let value = Published { provider: ledger.summarize("ollama", None),
        owner: ledger.summarize("ollama", Some("ollama")) };
    let mut published = shared.published.write().unwrap_or_else(|error| {
        shared.fail(LedgerError::Io); error.into_inner()
    });
    *published = value;
}
fn persist(shared: &Shared, ledger: &mut Ledger, path: &std::path::Path) {
    // Known memory counters are visible as Dirty before any filesystem work.
    publish(shared, ledger);
    match usage::now_seconds().and_then(|at| ledger.commit(path, ledger.store_revision(), at)) {
        Ok(_) => {},
        Err(error) => shared.fail(error),
    }
    publish(shared, ledger);
}
fn actor(mut ledger: Ledger, path: PathBuf, owner_lock: File, receiver: Receiver<Event>, shared: &Shared) {
    let _owner_lock = owner_lock;
    loop {
        match receiver.recv_timeout(ACTOR_POLL) {
            Ok(event) => {
                let result = match event {
                    Event::Begin(start) => ledger.begin(start).map(|_| ()),
                    Event::Observe { key, snapshot } => ledger.observe(&key, snapshot).map(|_| ()),
                    Event::Finish { key, snapshot, outcome, at } => {
                        let measured = match snapshot { Some(snapshot) => ledger.observe(&key, snapshot).map(|_| ()), None => Ok(()) };
                        let finished = ledger.finish(&key, outcome, at).map(|_| ());
                        measured.and(finished)
                    }
                };
                if let Err(error) = result { shared.fail(error); }
                if ledger.is_dirty() { persist(shared, &mut ledger, &path); }
                else { publish(shared, &ledger); }
                shared.queued.fetch_sub(1, Ordering::AcqRel);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {},
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                shared.fail(LedgerError::NotFound); break;
            }
        }
        if shared.closing.load(Ordering::Acquire) && shared.guards.load(Ordering::Acquire) == 0 &&
            shared.queued.load(Ordering::Acquire) == 0 {
            if ledger.is_dirty() { persist(shared, &mut ledger, &path); }
            break;
        }
    }
}

/// Explicit, inactive preparation: all filesystem work occurs here. No global
/// registry publication/spawn or activation mutex is held across bootstrap IO.
pub fn prepare(activation: Activation) -> Result<Prepared, LedgerError> {
    #[cfg(not(target_os = "linux"))]
    { let _ = activation; return Err(LedgerError::UnsupportedPlatform); }
    #[cfg(target_os = "linux")]
    {
        if !valid_id(&activation.ledger_id) { return Err(LedgerError::InvalidInput); }
        let owner_lock = bootstrap::owner_lock(&activation.path)?;
        let nonce = bootstrap::nonce()?;
        bootstrap::check_data(&activation.path, &activation.ledger_id)?;
        let now = usage::now_seconds()?;
        let mut ledger = match Ledger::open(&activation.path) {
            Ok(ledger) => ledger,
            Err(LedgerError::NotFound) => Ledger::new(&activation.ledger_id, now)?,
            Err(error) => return Err(error),
        };
        ledger.interrupt_open(now)?;
        ledger.commit(&activation.path, ledger.store_revision(), now)?;
        let published = Published { provider: ledger.summarize("ollama", None),
            owner: ledger.summarize("ollama", Some("ollama")) };
        Ok(Prepared { ledger, path: activation.path, owner_lock, nonce, published })
    }
}

impl Prepared {
    /// Short installation only: no filesystem read/write/fsync/ledger scan.
    /// Call under the same application lifecycle lock used to linearize Exit;
    /// discard a Prepared outside that lock when application closing won.
    pub fn install(self) -> Result<Service, LedgerError> {
        let _installation = ACTIVATING.lock().map_err(|_| LedgerError::Io)?;
        // Recheck here, not only in prepare: multiple private paths may have
        // been prepared concurrently before either one was installed.
        let mut registry = ACTIVE.write().map_err(|_| LedgerError::Io)?;
        if registry.as_ref().and_then(Weak::upgrade).is_some_and(|service|
            !service.stopped.load(Ordering::Acquire) || service.guards.load(Ordering::Acquire) != 0 ||
            service.queued.load(Ordering::Acquire) != 0 || service.gap.load(Ordering::Acquire)) {
            return Err(LedgerError::Busy);
        }
        let Prepared { ledger, path, owner_lock, nonce, published } = self;
        let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
        let shared = Arc::new(Shared { sender, published: RwLock::new(published),
            admission: Mutex::new(()), accepting: AtomicBool::new(true), closing: AtomicBool::new(false),
            stopped: AtomicBool::new(false), queued: AtomicUsize::new(0), guards: AtomicUsize::new(0),
            gap: AtomicBool::new(false), issue: AtomicU8::new(0), nonce, next: AtomicU64::new(0) });
        let worker = Arc::clone(&shared);
        // Registry validation precedes spawn, so a failure cannot strand an
        // actor with an owner lease that no returned Service can close.
        std::thread::Builder::new().name("pet-usage-capture".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
                actor(ledger, path, owner_lock, receiver, &worker)));
            if result.is_err() { worker.fail(LedgerError::Io); }
            worker.accepting.store(false, Ordering::Release);
            worker.stopped.store(true, Ordering::Release);
        }).map_err(|_| LedgerError::Io)?;
        *registry = Some(Arc::downgrade(&shared));
        Ok(Service { shared })
    }
}

/// Compatibility/bootstrap helper. Lifecycle-aware setup should use prepare
/// off-lock and then install under its short setup/Exit lock instead.
pub fn activate(activation: Activation) -> Result<Service, LedgerError> {
    prepare(activation)?.install()
}

fn error_code(error: LedgerError) -> u8 {
    match error {
        LedgerError::InvalidInput => 1, LedgerError::InvalidData => 2,
        LedgerError::Limit => 3, LedgerError::Conflict => 4, LedgerError::Busy => 5,
        LedgerError::NotFound => 6, LedgerError::Io => 7, LedgerError::UnsupportedPlatform => 8,
        LedgerError::DurabilityUnknown => 9, LedgerError::LockCleanup => 10,
    }
}
fn decode_error(code: u8) -> Option<LedgerError> {
    Some(match code {
        1 => LedgerError::InvalidInput, 2 => LedgerError::InvalidData,
        3 => LedgerError::Limit, 4 => LedgerError::Conflict, 5 => LedgerError::Busy,
        6 => LedgerError::NotFound, 7 => LedgerError::Io, 8 => LedgerError::UnsupportedPlatform,
        9 => LedgerError::DurabilityUnknown, 10 => LedgerError::LockCleanup,
        _ => return None,
    })
}

#[cfg(target_os = "linux")]
mod bootstrap {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::path::{Component, Path};
    const NOFOLLOW: i32 = 0o400000;
    const NONBLOCK: i32 = 0o4000;
    const DIRECTORY: i32 = 0o200000;
    unsafe extern "C" { fn geteuid() -> u32; }
    fn uid() -> u32 { unsafe { geteuid() } }
    fn private_file(file: &File) -> Result<(), LedgerError> {
        let metadata = file.metadata().map_err(|_| LedgerError::Io)?;
        if !metadata.is_file() || metadata.uid() != uid() || metadata.nlink() != 1 ||
            metadata.permissions().mode() & 0o777 != 0o600 { return Err(LedgerError::InvalidData); }
        Ok(())
    }
    pub(super) fn owner_lock(path: &Path) -> Result<File, LedgerError> {
        let name = path.file_name().and_then(|v| v.to_str()).ok_or(LedgerError::InvalidInput)?;
        if !path.is_absolute() || name.is_empty() || name.len() > 128 || name == ".capture-owner-lock" {
            return Err(LedgerError::InvalidInput);
        }
        let parent = path.parent().ok_or(LedgerError::InvalidInput)?;
        let mut walked = PathBuf::new();
        for component in parent.components() {
            if !matches!(component, Component::RootDir | Component::Normal(_)) { return Err(LedgerError::InvalidInput); }
            walked.push(component.as_os_str());
            let metadata = fs::symlink_metadata(&walked).map_err(|_| LedgerError::Io)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() { return Err(LedgerError::InvalidData); }
        }
        let directory = OpenOptions::new().read(true).custom_flags(NOFOLLOW | NONBLOCK | DIRECTORY)
            .open(parent).map_err(|_| LedgerError::Io)?;
        let metadata = directory.metadata().map_err(|_| LedgerError::Io)?;
        if metadata.uid() != uid() || metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(LedgerError::InvalidData);
        }
        let lock_path = parent.join(".capture-owner-lock");
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
            .mode(0o600).custom_flags(NOFOLLOW | NONBLOCK).open(&lock_path).map_err(|_| LedgerError::Io)?;
        private_file(&lock)?;
        match lock.try_lock() {
            Ok(()) => {},
            Err(std::fs::TryLockError::WouldBlock) => return Err(LedgerError::Busy),
            Err(std::fs::TryLockError::Error(_)) => return Err(LedgerError::Io),
        }
        let actual = fs::symlink_metadata(&lock_path).map_err(|_| LedgerError::Io)?;
        let pinned = lock.metadata().map_err(|_| LedgerError::Io)?;
        if actual.dev() != pinned.dev() || actual.ino() != pinned.ino() || actual.file_type().is_symlink() {
            return Err(LedgerError::Conflict);
        }
        // The stable owner-lock inode is deliberately NEVER unlinked.
        Ok(lock)
    }
    pub(super) fn check_data(path: &Path, expected_id: &str) -> Result<(), LedgerError> {
        let file = match OpenOptions::new().read(true).custom_flags(NOFOLLOW | NONBLOCK).open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(LedgerError::Io),
        };
        private_file(&file)?;
        if file.metadata().map_err(|_| LedgerError::Io)?.len() > usage::MAX_BYTES as u64 {
            return Err(LedgerError::Limit);
        }
        let mut bytes = Vec::new();
        file.take(usage::MAX_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|_| LedgerError::Io)?;
        if bytes.len() > usage::MAX_BYTES { return Err(LedgerError::Limit); }
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| LedgerError::InvalidData)?;
        if value.get("ledgerId").and_then(Value::as_str) != Some(expected_id) { return Err(LedgerError::Conflict); }
        let attempts = value.get("attempts").and_then(Value::as_array).ok_or(LedgerError::InvalidData)?;
        if attempts.len() > usage::MAX_ATTEMPTS { return Err(LedgerError::Limit); }
        for attempt in attempts {
            // This activation cannot interrupt or adopt another logical owner.
            if attempt.pointer("/start/key/provider").and_then(Value::as_str) != Some("ollama") ||
                attempt.pointer("/start/key/owner").and_then(Value::as_str) != Some("ollama") {
                return Err(LedgerError::InvalidData);
            }
        }
        // Ledger::open immediately performs the full strict schema validation.
        Ok(())
    }
    pub(super) fn nonce() -> Result<String, LedgerError> {
        let mut bytes = [0u8; 16];
        File::open("/dev/urandom").and_then(|mut file| file.read_exact(&mut bytes)).map_err(|_| LedgerError::Io)?;
        let mut nonce = String::with_capacity(32);
        for byte in bytes { use std::fmt::Write; write!(&mut nonce, "{byte:02x}").map_err(|_| LedgerError::Io)?; }
        Ok(nonce)
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "capture_tests.rs"]
mod tests;
