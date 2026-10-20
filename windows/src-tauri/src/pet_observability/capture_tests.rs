//! Synthetic actor/bootstrap regressions. No provider, model or application data.
use super::*;
use serde_json::json;
use std::{fs, os::unix::fs::{MetadataExt, PermissionsExt}, path::Path};

struct PrivateDir(PathBuf);
impl PrivateDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("mochi-capture-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        Self(path)
    }
    fn activation(&self) -> Activation {
        Activation { path: self.0.join("usage.json"), ledger_id: "synthetic-ledger".into() }
    }
}
impl Drop for PrivateDir { fn drop(&mut self) { fs::remove_dir_all(&self.0).unwrap(); } }

fn origin() -> Origin { Origin { provider: "ollama".into(), owner: "ollama".into(), conversation: "conversation-a".into() } }
fn fixture_key() -> AttemptKey { AttemptKey { provider: "ollama".into(), owner: "ollama".into(), conversation: "conversation-a".into(), attempt: "synthetic-attempt".into() } }
fn fixture_start() -> AttemptStart { AttemptStart { key: fixture_key(), started_at: 100, server_id: None, requested_model: Some("synthetic-model".into()) } }
fn private_write(path: &Path, bytes: &[u8]) { fs::write(path, bytes).unwrap(); fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap(); }

// An intentionally undrained queue makes overflow deterministic, without a disk
// stall, global registry replacement or a provider request.
fn detached(capacity: usize) -> (Arc<Shared>, Receiver<Event>, Capture) {
    let ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    let (sender, receiver) = mpsc::sync_channel(capacity);
    let shared = Arc::new(Shared { sender,
        published: RwLock::new(Published { provider: ledger.summarize("ollama", None), owner: ledger.summarize("ollama", Some("ollama")) }),
        admission: Mutex::new(()), accepting: AtomicBool::new(true), closing: AtomicBool::new(false), stopped: AtomicBool::new(false),
        queued: AtomicUsize::new(0), guards: AtomicUsize::new(1), gap: AtomicBool::new(false), issue: AtomicU8::new(0), nonce: "synthetic".into(), next: AtomicU64::new(0) });
    let capture = Capture { shared: Some(shared.clone()), key: Some(fixture_key()), snapshot: None, snapshot_sent: false, complete: false, terminal: None, finish_sent: false };
    (shared, receiver, capture)
}
fn take(shared: &Shared, receiver: &Receiver<Event>) -> Event {
    let event = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
    shared.queued.fetch_sub(1, Ordering::AcqRel); event
}

#[test]
fn full_queue_keeps_the_latest_counters_and_original_terminal_outcome_for_retry() {
    let (shared, receiver, mut capture) = detached(1);
    assert!(shared.enqueue(Event::Begin(fixture_start())));
    capture.observe_oai(&json!({"usage":{"prompt_tokens":2,"completion_tokens":3}}));
    assert!(!capture.snapshot_sent);
    assert_eq!(shared.queued.load(Ordering::Acquire), 1);
    let partial = snapshot(&shared, "ollama", None);
    assert_eq!(partial.persistence.state, PersistenceState::Unknown);
    assert_eq!(partial.persistence.issue, Some(LedgerError::Limit));
    assert_eq!(partial.total.coverage, CoverageStatus::Partial);
    assert!(matches!(take(&shared, &receiver), Event::Begin(_)));
    capture.finish(AttemptOutcome::Cancelled);
    let ended = capture.terminal;
    match take(&shared, &receiver) {
        Event::Finish { key, snapshot: Some(snapshot), outcome, .. } => {
            assert_eq!(key, fixture_key()); assert_eq!(outcome, AttemptOutcome::Cancelled);
            assert_eq!(snapshot.counters.input_tokens, Some(2));
        }, _ => panic!("finish must carry counters lost to the full queue"),
    }
    assert!(shared.enqueue(Event::Begin(fixture_start())));
    capture.observe_oai(&json!({"usage":{"prompt_tokens":4,"completion_tokens":5}}));
    assert!(!capture.finish_sent);
    take(&shared, &receiver);
    capture.finish(AttemptOutcome::Completed);
    assert_eq!(capture.terminal, ended);
    match take(&shared, &receiver) {
        Event::Finish { snapshot: Some(snapshot), outcome, .. } => {
            assert_eq!(outcome, AttemptOutcome::Cancelled);
            assert_eq!(snapshot.counters.input_tokens, Some(4));
            assert_eq!(snapshot.counters.output_tokens, Some(5));
        }, _ => panic!("late usage must remain retryable after the first finish"),
    }
    drop(capture); assert_eq!(shared.guards.load(Ordering::Acquire), 0);
}

#[test]
fn native_done_does_not_invent_zero_and_counter_alias_conflicts_stay_unknown() {
    let (shared, receiver, mut capture) = detached(8);
    capture.observe_native(&json!({"done":true}));
    assert!(capture.snapshot.is_none());
    capture.observe_oai(&json!({"usage":{"prompt_tokens":2,"input_tokens":9,"completion_tokens":0}}));
    let snapshot = capture.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.counters.input_tokens, None);
    assert_eq!(snapshot.counters.output_tokens, Some(0));
    assert_eq!(snapshot.completeness, Completeness::Final);
    capture.finish(AttemptOutcome::Failed); drop(capture);
    while receiver.try_recv().is_ok() { shared.queued.fetch_sub(1, Ordering::AcqRel); }
}

#[test]
fn shutdown_timeout_does_not_claim_that_pending_guards_or_io_have_finished() {
    let (shared, _receiver, capture) = detached(8);
    let service = Service { shared: shared.clone() };
    let began = Instant::now(); let report = service.shutdown(Duration::from_millis(5));
    assert!(began.elapsed() < Duration::from_secs(1));
    assert!(!report.stopped && report.incomplete);
    assert_eq!(report.pending_guards, 1);
    assert_eq!(report.issue, Some(LedgerError::DurabilityUnknown));
    assert!(!shared.accepting.load(Ordering::Acquire));
    drop(capture); drop(service);
}

#[test]
fn prepared_recovery_keeps_counters_and_uses_a_stable_exclusive_owner_lock() {
    let dir = PrivateDir::new(); let path = dir.activation().path;
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    ledger.begin(fixture_start()).unwrap();
    ledger.observe(&fixture_key(), ServerSnapshot { sequence:1, observed_at:101, completeness:Completeness::Final,
        counters:ServerCounters { input_tokens:Some(4), output_tokens:Some(5), total_tokens:None }, reported_model:None }).unwrap();
    ledger.commit(&path, 0, 102).unwrap();
    let prepared = prepare(dir.activation()).unwrap();
    assert!(matches!(prepare(dir.activation()), Err(LedgerError::Busy)));
    let recovered = Ledger::open(&path).unwrap().summarize("ollama", None);
    assert_eq!(recovered.total.known_tokens.as_deref(), Some("9"));
    assert_eq!(recovered.attempts.interrupted, 1); assert_eq!(recovered.attempts.pending, 0);
    let lock_path = dir.0.join(".capture-owner-lock");
    let first = fs::metadata(&lock_path).unwrap(); drop(prepared);
    let prepared = prepare(dir.activation()).unwrap(); let second = fs::metadata(&lock_path).unwrap();
    assert_eq!((first.dev(), first.ino()), (second.dev(), second.ino()));
    assert_eq!(second.permissions().mode() & 0o777, 0o600); drop(prepared);
}

#[test]
fn bootstrap_refuses_wrong_ledger_owner_unsafe_lock_and_symlink_without_replacement() {
    let dir = PrivateDir::new(); let path = dir.activation().path;
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap(); let mut start = fixture_start();
    start.key.owner = "another-owner".into(); ledger.begin(start).unwrap(); ledger.commit(&path, 0, 101).unwrap();
    let original = fs::read(&path).unwrap();
    assert!(matches!(prepare(dir.activation()), Err(LedgerError::InvalidData)));
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    let lock = dir.0.join(".capture-owner-lock"); fs::set_permissions(&lock, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(prepare(dir.activation()), Err(LedgerError::InvalidData)));
    assert_eq!(fs::metadata(&lock).unwrap().permissions().mode() & 0o777, 0o644);
    fs::remove_file(&lock).unwrap();
    let target = dir.0.join("synthetic-target"); private_write(&target, b"preserve");
    std::os::unix::fs::symlink(&target, &lock).unwrap();
    assert!(prepare(dir.activation()).is_err()); assert_eq!(fs::read(&target).unwrap(), b"preserve");
    assert!(fs::symlink_metadata(&lock).unwrap().file_type().is_symlink());
}

#[test]
fn actual_actor_deduplicates_within_attempts_but_counts_distinct_dispatches_and_late_stop_usage() {
    // This is the only test installing the process registry. Other actor tests
    // use detached queues/preparation and cannot contaminate this lifetime.
    let dir = PrivateDir::new(); let prepared = prepare(dir.activation()).unwrap();
    assert!(current(false).is_none());
    drop(Capture::begin(None, "synthetic-model"));
    let service = prepared.install().unwrap(); let origin = origin();
    drop(Capture::begin(None, "synthetic-model"));
    mark_untracked("codex", "codex", LedgerError::InvalidInput);
    let mut first = Capture::begin(Some(&origin), "synthetic-model");
    first.observe_oai(&json!({"usage":{"prompt_tokens":2,"completion_tokens":3}}));
    first.transport_complete(); first.finish(AttemptOutcome::Cancelled);
    first.observe_oai(&json!({"usage":{"prompt_tokens":4,"completion_tokens":5}}));
    first.finish(AttemptOutcome::Completed); drop(first);
    let mut second = Capture::begin(Some(&origin), "synthetic-model");
    let measured = json!({"done":true,"prompt_eval_count":4,"eval_count":5});
    second.observe_native(&measured); second.observe_native(&measured);
    second.finish(AttemptOutcome::Completed); drop(second);
    drop(Capture::begin(Some(&origin), "synthetic-model"));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let summary = service.summarize("ollama", None);
        if summary.attempts.tracked == 3 && summary.attempts.pending == 0 && summary.persistence.state == PersistenceState::Durable { break; }
        assert!(Instant::now() < deadline, "actor did not drain: {:?}", summary);
        std::thread::sleep(Duration::from_millis(2));
    }
    let summary = service.summarize("ollama", Some("ollama"));
    assert_eq!(summary.total.known_tokens.as_deref(), Some("18"));
    assert_eq!(summary.total.unknown_attempts, 1);
    assert_eq!(summary.attempts.cancelled, 1); assert_eq!(summary.attempts.completed, 1); assert_eq!(summary.attempts.interrupted, 1);
    assert_eq!(summary.persistence.issue, None);
    assert!(service.summarize("codex", None).total.known_tokens.is_none());
    let report = service.shutdown(Duration::from_secs(2));
    assert!(report.stopped && !report.incomplete, "{report:?}"); drop(service);
    let reopened = Ledger::open(&dir.activation().path).unwrap().summarize("ollama", None);
    assert_eq!(reopened.total.known_tokens.as_deref(), Some("18"));
    drop(prepare(dir.activation()).unwrap());
}
