//! Regression tests use synthetic private files only; no models or account IPC.
use super::{cache::Cache, core::*, usage::*};
use std::{path::PathBuf, sync::atomic::{AtomicU64, Ordering}, time::{Duration, Instant}};
use std::os::unix::fs::{PermissionsExt, symlink};

fn key(id: &str) -> AttemptKey {
    AttemptKey { provider: "ollama".into(), owner: "ai_ollama".into(), conversation: "conversation-a".into(), attempt: id.into() }
}
fn start(k: AttemptKey) -> AttemptStart {
    AttemptStart { key: k, started_at: 100, server_id: Some("local-ollama".into()), requested_model: Some("synthetic-model".into()) }
}
fn snapshot(sequence: u64, input: Option<u64>, output: Option<u64>, completeness: Completeness) -> ServerSnapshot {
    ServerSnapshot { sequence, observed_at: 101, completeness, counters: ServerCounters { input_tokens: input, output_tokens: output, total_tokens: None }, reported_model: None }
}
fn row(id: &str, state: Activity) -> SessionRow {
    SessionRow { key: id.into(), provider: "codex".into(), pet_id: Some("agent_codex".into()), origin: Origin::External, session_id: Some(id.into()), conversation_id: None, title: id.into(), cwd: None, state, archived: Some(false), updated_at: Some(100), observed_at: Some(100), open: Some(OpenTarget::External { pet_id: "agent_codex".into(), provider: "codex".into(), session_id: id.into() }) }
}
fn inventory(rows: Vec<SessionRow>, coverage: super::core::Coverage) -> Inventory {
    Inventory::build(InventoryInput { scope: "codexDaemon".into(), coverage, observed_at: Some(100), reason: None, rows }, 100, 15, false)
}

#[test]
fn unobserved_usage_and_measured_zero_remain_distinct() {
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    let k = key("one"); ledger.begin(start(k.clone())).unwrap();
    let unknown = ledger.summarize("ollama", None);
    assert!(unknown.total.known_tokens.is_none()); assert_eq!(unknown.total.unknown_attempts, 1);
    ledger.observe(&k, snapshot(1, Some(0), Some(0), Completeness::Final)).unwrap();
    ledger.finish(&k, AttemptOutcome::Completed, 102).unwrap();
    let zero = ledger.summarize("ollama", None);
    assert_eq!(zero.total.known_tokens.as_deref(), Some("0"));
    assert_eq!(zero.total.derived_attempts, 1);
    assert_eq!(zero.coverage.status, CoverageStatus::CompleteWithinTrackedAttempts);
    assert_eq!(serde_json::to_value(zero.coverage).unwrap()["outsideMochi"], "notCovered");
}

#[test]
fn streaming_snapshots_replace_and_replays_do_not_add_tokens() {
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    let k = key("one"); ledger.begin(start(k.clone())).unwrap();
    ledger.observe(&k, snapshot(1, Some(4), None, Completeness::Partial)).unwrap();
    let final_value = snapshot(2, Some(6), Some(3), Completeness::Final);
    ledger.observe(&k, final_value.clone()).unwrap();
    assert_eq!(ledger.observe(&k, final_value).unwrap(), Change::Unchanged);
    assert_eq!(ledger.observe(&k, snapshot(1, Some(900), Some(900), Completeness::Final)).unwrap(), Change::Stale);
    assert_eq!(ledger.observe(&k, snapshot(2, Some(900), Some(900), Completeness::Final)).unwrap_err(), LedgerError::Conflict);
    assert_eq!(ledger.summarize("ollama", None).total.known_tokens.as_deref(), Some("9"));
}

#[test]
fn late_usage_after_stop_keeps_the_original_owner_and_cancelled_outcome() {
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    let k = key("one"); ledger.begin(start(k.clone())).unwrap();
    ledger.finish(&k, AttemptOutcome::Cancelled, 102).unwrap();
    ledger.observe(&k, snapshot(1, Some(7), Some(2), Completeness::Final)).unwrap();
    let mut other = key("two"); other.owner = "other-owner".into(); other.conversation = "conversation-b".into();
    ledger.begin(start(other.clone())).unwrap();
    ledger.observe(&other, snapshot(1, Some(3), Some(1), Completeness::Final)).unwrap();
    let original = ledger.summarize("ollama", Some("ai_ollama"));
    assert_eq!(original.total.known_tokens.as_deref(), Some("9")); assert_eq!(original.attempts.cancelled, 1); assert_eq!(original.attempts.completed, 0);
    assert_eq!(ledger.summarize("ollama", None).total.known_tokens.as_deref(), Some("13"));
    let mut rebound = start(k); rebound.requested_model = Some("different-model".into());
    assert_eq!(ledger.begin(rebound).unwrap_err(), LedgerError::Conflict);
    assert_eq!(ledger.summarize("ollama", Some("ai_ollama")).total.known_tokens.as_deref(), Some("9"));
}

#[test]
fn counters_beyond_javascript_integer_precision_stay_exact_decimals() {
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap();
    let k = key("one"); ledger.begin(start(k.clone())).unwrap();
    ledger.observe(&k, snapshot(1, Some(u64::MAX), Some(u64::MAX), Completeness::Final)).unwrap();
    assert_eq!(ledger.summarize("ollama", None).total.known_tokens.as_deref(), Some("36893488147419103230"));
}

struct PrivateDir(PathBuf);
impl PrivateDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("mochi-root-offline-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap(); std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap(); Self(path)
    }
}
impl Drop for PrivateDir { fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); } }

#[test]
fn durable_reload_preserves_tokens_and_a_stale_writer_cannot_overwrite() {
    let dir = PrivateDir::new(); let path = dir.0.join("ledger.json");
    let mut first = Ledger::new("synthetic-ledger", 100).unwrap(); let k = key("one"); first.begin(start(k.clone())).unwrap();
    first.observe(&k, snapshot(1, Some(2), Some(3), Completeness::Final)).unwrap(); first.commit(&path, 0, 102).unwrap();
    let mut stale = Ledger::open(&path).unwrap(); let original = std::fs::read(&path).unwrap();
    assert_eq!(stale.summarize("ollama", None).persistence.state, PersistenceState::Unknown);
    let receipt = stale.commit(&path, 1, 103).unwrap(); assert!(!receipt.changed); assert_eq!(std::fs::read(&path).unwrap(), original);
    first.finish(&k, AttemptOutcome::Completed, 104).unwrap(); first.commit(&path, 1, 104).unwrap();
    let committed = std::fs::read(&path).unwrap(); stale.begin(start(key("two"))).unwrap();
    assert_eq!(stale.commit(&path, 1, 105).unwrap_err(), LedgerError::Conflict); assert_eq!(std::fs::read(&path).unwrap(), committed);
    assert_eq!(Ledger::open(&path).unwrap().summarize("ollama", None).total.known_tokens.as_deref(), Some("5"));
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn symbolic_links_and_non_private_files_are_rejected_without_replacement() {
    let dir = PrivateDir::new(); let path = dir.0.join("ledger.json"); let link = dir.0.join("link.json");
    let mut ledger = Ledger::new("synthetic-ledger", 100).unwrap(); ledger.commit(&path, 0, 101).unwrap();
    symlink(&path, &link).unwrap(); assert!(Ledger::open(&link).is_err());
    let before = std::fs::read(&path).unwrap(); let mut replacement = Ledger::new("other-ledger", 100).unwrap(); assert!(replacement.commit(&link, 0, 102).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap(); assert!(Ledger::open(&path).is_err());
}

#[test]
fn totals_cover_all_rows_even_when_the_visible_list_is_truncated() {
    let mut rows: Vec<_> = (0..30).map(|i| row(&format!("id-{i}"), Activity::Idle)).collect();
    rows[0].state = Activity::Reserved; rows[1].state = Activity::Busy;
    let result = inventory(rows, super::core::Coverage::Complete);
    assert_eq!(result.counts.total.value, Some(30)); assert_eq!(result.listed, 12); assert!(result.truncated);
    assert_eq!(result.counts.confirmed_active.value, Some(1)); assert_eq!(result.counts.reserved.value, Some(1));
    assert_eq!(result.counts.working.value, None); // Busy is not guessed Working.
}

#[test]
fn ambiguous_identity_removes_navigation_and_does_not_invent_exact_totals() {
    let a = row("same", Activity::Working); let mut b = a.clone(); b.pet_id = Some("another-pet".into());
    let result = inventory(vec![a, b], super::core::Coverage::Complete);
    assert_eq!(result.coverage, super::core::Coverage::Partial); assert_eq!(result.counts.total.value, None);
    assert_eq!(result.counts.total.observed, 1); assert_eq!(result.counts.unknown, 1); assert!(result.sessions[0].open.is_none());
}

#[test]
fn failed_refresh_retains_age_and_subtotals_with_single_flight_cooldown() {
    let mut cache = Cache::default(); let at = Instant::now();
    let ticket = cache.begin("codex", "codexDaemon", at).unwrap(); assert!(cache.begin("codex", "codexDaemon", at).is_none());
    assert!(cache.begin("copilot", "copilotLiveEndpoints", at).is_some());
    cache.finish(ticket, Ok(inventory(vec![row("one", Activity::Working)], super::core::Coverage::Complete)));
    assert_eq!(cache.snapshot("codex", "codexDaemon", 100, 15).counts.total.value, Some(1));
    let ticket = cache.begin("codex", "codexDaemon", at + Duration::from_secs(10)).unwrap(); cache.finish(ticket, Err("synthetic failure".into()));
    let failed = cache.snapshot("codex", "codexDaemon", 101, 15);
    assert!(failed.stale); assert_eq!(failed.observed_at, Some(100)); assert_eq!(failed.counts.total.value, None); assert_eq!(failed.counts.total.observed, 1);
    assert!(cache.begin("codex", "codexDaemon", at + Duration::from_secs(11)).is_none());
}
