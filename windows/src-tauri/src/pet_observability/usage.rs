//! Private, inactive preview: cumulative usage for explicitly registered requests.
//! No transport, UI, model, credential, migration or automatic filesystem access.
//! Counters are provider reports, not tokenizer estimates, quotas or chat length.
//! Every public/persisted timestamp is Unix SECONDS, never legacy chat millis.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

pub const MAX_ATTEMPTS: usize = 4096;
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_TIME: u64 = 9_007_199_254_740_991;
const DISK_VERSION: u32 = 1;

/// This preview owns its clock contract; assistant_memory::now() is milliseconds.
pub fn now_seconds() -> Result<u64, LedgerError> {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| LedgerError::InvalidInput)?.as_secs();
    if valid_time(seconds) { Ok(seconds) } else { Err(LedgerError::InvalidInput) }
}
pub fn seconds_from_millis(millis: u64) -> Result<u64, LedgerError> {
    let seconds = millis / 1000;
    if valid_time(seconds) { Ok(seconds) } else { Err(LedgerError::InvalidInput) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LedgerError {
    InvalidInput, InvalidData, Limit, Conflict, Busy, NotFound, Io,
    UnsupportedPlatform, DurabilityUnknown, LockCleanup,
}
impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "Invalid usage ledger input.",
            Self::InvalidData => "Invalid usage ledger data.",
            Self::Limit => "Usage ledger limit reached.",
            Self::Conflict => "Usage ledger compare-and-swap conflict.",
            Self::Busy => "Usage ledger writer is busy.",
            Self::NotFound => "Usage ledger is unavailable.",
            Self::Io => "Usage ledger IO failed.",
            Self::UnsupportedPlatform => "Private usage persistence is unsupported here.",
            Self::DurabilityUnknown => "Usage ledger durability is unknown.",
            Self::LockCleanup => "Usage ledger writer lock needs review.",
        })
    }
}
impl std::error::Error for LedgerError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change { Applied, Unchanged, Stale }

/// Caller generates an opaque, stable ID per actual HTTP attempt. Epoch alone
/// is insufficient: it restarts, and one public turn can make multiple requests.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttemptKey {
    pub provider: String,
    pub owner: String,
    pub conversation: String,
    pub attempt: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttemptStart {
    pub key: AttemptKey,
    pub started_at: u64,
    /// Sanitized opaque backend identity; never a URL containing credentials.
    pub server_id: Option<String>,
    pub requested_model: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerCounters {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Completeness { Partial, Final }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerSnapshot {
    /// Positive, monotonically increasing observation ordinal for this attempt.
    /// Same ordinal + same content is idempotent; same ordinal + different
    /// content conflicts. These are snapshots, never additive stream deltas.
    pub sequence: u64,
    pub observed_at: u64,
    pub completeness: Completeness,
    pub counters: ServerCounters,
    pub reported_model: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttemptOutcome { Pending, Completed, Failed, Cancelled, Interrupted }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AttemptRecord {
    start: AttemptStart,
    outcome: AttemptOutcome,
    ended_at: Option<u64>,
    snapshot: Option<ServerSnapshot>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Disk {
    version: u32,
    ledger_id: String,
    revision: u64,
    since: u64,
    committed_at: u64,
    attempts: Vec<AttemptRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SummaryStatus { Unavailable, NoObservations, Observed }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CoverageStatus { Unavailable, Partial, CompleteWithinTrackedAttempts }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NotCovered { NotCovered }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PersistenceState { Unavailable, MemoryOnly, Dirty, Durable, Error, Unknown }
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionSummary {
    /// Exact decimal; null means no counter observed, not a measured zero.
    pub known_tokens: Option<String>,
    pub known_attempts: u32,
    pub partial_attempts: u32,
    pub missing_attempts: u32,
    pub unknown_attempts: u32,
    /// Subset of known/partial attempts; nonzero only for derived totals.
    pub derived_attempts: u32,
    pub coverage: CoverageStatus,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptCounts {
    pub tracked: u32, pub pending: u32, pub completed: u32,
    pub failed: u32, pub cancelled: u32, pub interrupted: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    pub status: CoverageStatus,
    pub historical_before_since: NotCovered,
    pub outside_mochi: NotCovered,
    pub untracked_mochi: NotCovered,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Persistence {
    pub state: PersistenceState,
    /// Last confirmed disk revision, never a request ordinal/UI generation.
    pub revision: Option<String>,
    pub last_committed_at: Option<u64>,
    pub issue: Option<LedgerError>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub status: SummaryStatus,
    pub scope: &'static str,
    pub provider: Option<String>,
    pub owner: Option<String>,
    pub since: Option<u64>,
    pub last_observation: Option<u64>,
    /// Timestamp presence only; never a TTL/global-provider freshness claim.
    pub freshness: SummaryStatus,
    pub input: DimensionSummary,
    pub output: DimensionSummary,
    pub total: DimensionSummary,
    pub attempts: AttemptCounts,
    pub coverage: Coverage,
    pub persistence: Persistence,
}

#[derive(Default)]
struct Dimension {
    sum: u128, seen: bool, known: u32, partial: u32,
    missing: u32, unknown: u32, derived: u32,
}
impl Dimension {
    fn observe(&mut self, snapshot: Option<&ServerSnapshot>, value: Option<u128>, derived: bool) {
        match (snapshot, value) {
            (None, _) => self.unknown += 1,
            (Some(_), None) => self.missing += 1,
            (Some(s), Some(value)) => {
                // MAX_ATTEMPTS * (2 * u64::MAX) < 2^77, safely below u128::MAX.
                self.sum += value; self.seen = true;
                if s.completeness == Completeness::Final { self.known += 1; }
                else { self.partial += 1; }
                if derived { self.derived += 1; }
            }
        }
    }
    fn finish(self, tracked: u32) -> DimensionSummary {
        let coverage = if tracked == 0 { CoverageStatus::Unavailable }
            else if self.unknown == 0 && self.missing == 0 && self.partial == 0 {
                CoverageStatus::CompleteWithinTrackedAttempts
            } else { CoverageStatus::Partial };
        DimensionSummary {
            known_tokens: self.seen.then(|| self.sum.to_string()),
            known_attempts: self.known, partial_attempts: self.partial,
            missing_attempts: self.missing, unknown_attempts: self.unknown,
            derived_attempts: self.derived, coverage,
        }
    }
}
fn uncovered(status: CoverageStatus) -> Coverage {
    Coverage { status, historical_before_since: NotCovered::NotCovered,
        outside_mochi: NotCovered::NotCovered, untracked_mochi: NotCovered::NotCovered }
}
/// Safe bridge response before an explicitly authorized ledger/transport rollout.
pub fn unavailable() -> UsageSummary {
    UsageSummary {
        status: SummaryStatus::Unavailable, scope: "mochi-observed-attempts",
        provider: None, owner: None, since: None, last_observation: None,
        freshness: SummaryStatus::Unavailable,
        input: Dimension::default().finish(0), output: Dimension::default().finish(0),
        total: Dimension::default().finish(0), attempts: AttemptCounts::default(),
        coverage: uncovered(CoverageStatus::Unavailable),
        persistence: Persistence { state: PersistenceState::Unavailable,
            revision: None, last_committed_at: None, issue: None },
    }
}

/// Inert until a caller explicitly creates/opens it. No default path or singleton.
/// All mutating operations require &mut self; writers serialize externally or
/// behind their own Mutex. Persistence CAS coordinates cooperating processes.
pub struct Ledger {
    ledger_id: String,
    since: u64,
    attempts: BTreeMap<AttemptKey, AttemptRecord>,
    base_disk: Option<Disk>,
    bound_path: Option<PathBuf>,
    dirty: bool,
    confirmed_revision: Option<u64>,
    confirmed_at: Option<u64>,
    state: PersistenceState,
    issue: Option<LedgerError>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReceipt { pub revision: String, pub changed: bool }

impl Ledger {
    /// `since` is the caller's explicit observation-period start, not history.
    pub fn new(ledger_id: &str, since: u64) -> Result<Self, LedgerError> {
        if !valid_id(ledger_id) || !valid_time(since) { return Err(LedgerError::InvalidInput); }
        Ok(Self { ledger_id: ledger_id.into(), since, attempts: BTreeMap::new(),
            base_disk: None, bound_path: None, dirty: true, confirmed_revision: None,
            confirmed_at: None, state: PersistenceState::MemoryOnly, issue: None })
    }
    pub fn store_revision(&self) -> u64 { self.base_disk.as_ref().map_or(0, |d| d.revision) }
    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn begin(&mut self, start: AttemptStart) -> Result<Change, LedgerError> {
        validate_start(&start).map_err(|_| LedgerError::InvalidInput)?;
        if let Some(old) = self.attempts.get(&start.key) {
            return if old.start == start { Ok(Change::Unchanged) } else { Err(LedgerError::Conflict) };
        }
        if self.attempts.len() >= MAX_ATTEMPTS { return Err(LedgerError::Limit); }
        let key = start.key.clone();
        self.attempts.insert(key, AttemptRecord { start, outcome: AttemptOutcome::Pending,
            ended_at: None, snapshot: None });
        self.changed(); Ok(Change::Applied)
    }
    pub fn observe(&mut self, key: &AttemptKey, snapshot: ServerSnapshot) -> Result<Change, LedgerError> {
        validate_snapshot(&snapshot).map_err(|_| LedgerError::InvalidInput)?;
        let old = self.attempts.get_mut(key).ok_or(LedgerError::NotFound)?;
        if let Some(previous) = &old.snapshot {
            if snapshot.sequence < previous.sequence { return Ok(Change::Stale); }
            if snapshot.sequence == previous.sequence {
                return if snapshot == *previous { Ok(Change::Unchanged) } else { Err(LedgerError::Conflict) };
            }
            if previous.completeness == Completeness::Final && snapshot.completeness != Completeness::Final {
                return Err(LedgerError::InvalidInput);
            }
        }
        // A late final counter can still arrive after Stop/failed UI validation.
        old.snapshot = Some(snapshot); self.changed(); Ok(Change::Applied)
    }
    pub fn finish(&mut self, key: &AttemptKey, outcome: AttemptOutcome, ended_at: u64) -> Result<Change, LedgerError> {
        if outcome == AttemptOutcome::Pending || !valid_time(ended_at) { return Err(LedgerError::InvalidInput); }
        let old = self.attempts.get_mut(key).ok_or(LedgerError::NotFound)?;
        if old.outcome != AttemptOutcome::Pending {
            return if old.outcome == outcome && old.ended_at == Some(ended_at) {
                Ok(Change::Unchanged)
            } else { Err(LedgerError::Conflict) };
        }
        old.outcome = outcome; old.ended_at = Some(ended_at);
        self.changed(); Ok(Change::Applied)
    }
    /// Explicit recovery decision; reading an existing ledger never mutates it.
    /// Pending attempts retain all measurements and become Interrupted, not zero.
    pub fn interrupt_open(&mut self, at: u64) -> Result<usize, LedgerError> {
        if !valid_time(at) { return Err(LedgerError::InvalidInput); }
        let mut changed = 0;
        for record in self.attempts.values_mut() {
            if record.outcome == AttemptOutcome::Pending {
                record.outcome = AttemptOutcome::Interrupted; record.ended_at = Some(at); changed += 1;
            }
        }
        if changed != 0 { self.changed(); }
        Ok(changed)
    }
    fn changed(&mut self) {
        self.dirty = true;
        self.state = if self.base_disk.is_some() { PersistenceState::Dirty } else { PersistenceState::MemoryOnly };
        self.issue = None;
    }
    pub fn summarize(&self, provider: &str, owner: Option<&str>) -> UsageSummary {
        if !valid_id(provider) || owner.is_some_and(|v| !valid_id(v)) { return unavailable(); }
        let mut counts = AttemptCounts::default();
        let (mut input, mut output, mut total) = (Dimension::default(), Dimension::default(), Dimension::default());
        let mut last = None;
        for r in self.attempts.values().filter(|r| r.start.key.provider == provider &&
            owner.is_none_or(|o| r.start.key.owner == o)) {
            counts.tracked += 1;
            match r.outcome {
                AttemptOutcome::Pending => counts.pending += 1,
                AttemptOutcome::Completed => counts.completed += 1,
                AttemptOutcome::Failed => counts.failed += 1,
                AttemptOutcome::Cancelled => counts.cancelled += 1,
                AttemptOutcome::Interrupted => counts.interrupted += 1,
            }
            let s = r.snapshot.as_ref();
            if let Some(s) = s { last = Some(last.map_or(s.observed_at, |v: u64| v.max(s.observed_at))); }
            let i = s.and_then(|s| s.counters.input_tokens).map(u128::from);
            let o = s.and_then(|s| s.counters.output_tokens).map(u128::from);
            let supplied = s.and_then(|s| s.counters.total_tokens).map(u128::from);
            let derived = supplied.is_none() && i.is_some() && o.is_some();
            let t = supplied.or_else(|| i.zip(o).map(|(i, o)| i + o));
            input.observe(s, i, false); output.observe(s, o, false); total.observe(s, t, derived);
        }
        let input = input.finish(counts.tracked);
        let output = output.finish(counts.tracked);
        let total = total.finish(counts.tracked);
        let coverage = if counts.tracked == 0 { CoverageStatus::Unavailable }
            else if counts.pending == 0 && [input.coverage, output.coverage, total.coverage].iter()
                .all(|c| *c == CoverageStatus::CompleteWithinTrackedAttempts) {
                CoverageStatus::CompleteWithinTrackedAttempts
            } else { CoverageStatus::Partial };
        let status = if last.is_some() { SummaryStatus::Observed } else { SummaryStatus::NoObservations };
        UsageSummary {
            status, scope: "mochi-observed-attempts", provider: Some(provider.into()), owner: owner.map(str::to_owned),
            since: Some(self.since), last_observation: last, freshness: status, input, output, total, attempts: counts,
            coverage: uncovered(coverage), persistence: Persistence { state: self.state,
                revision: self.confirmed_revision.map(|v| v.to_string()), last_committed_at: self.confirmed_at, issue: self.issue },
        }
    }
    /// Explicit, read-only load. No profiles/history/config discovery or fallback.
    pub fn open(path: &Path) -> Result<Self, LedgerError> {
        #[cfg(target_os = "linux")]
        {
            let _dir = io::private_parent(path)?;
            let disk = io::read_disk(path)?.ok_or(LedgerError::NotFound)?;
            let attempts = validate_disk(&disk)?;
            Ok(Self { ledger_id: disk.ledger_id.clone(), since: disk.since, attempts,
                // Visibility of a valid file does not confirm a preceding
                // rename's directory fsync. Loading remains strictly read-only.
                // A no-change commit can confirm this CAS base without adding
                // tokens, advancing its revision or repeating inference.
                bound_path: Some(path.into()), dirty: false, confirmed_revision: None,
                confirmed_at: None, state: PersistenceState::Unknown,
                issue: Some(LedgerError::DurabilityUnknown), base_disk: Some(disk) })
        }
        #[cfg(not(target_os = "linux"))]
        { let _ = path; Err(LedgerError::UnsupportedPlatform) }
    }
    /// Atomic private replacement, under an exclusive cooperative writer lock.
    /// `expected_revision` comes from store_revision(), not request/UI ordinals.
    /// Content and exact previous Disk snapshot are compared inside the lock.
    pub fn commit(&mut self, path: &Path, expected_revision: u64, at: u64) -> Result<CommitReceipt, LedgerError> {
        #[cfg(target_os = "linux")]
        let result = self.commit_linux(path, expected_revision, at);
        #[cfg(not(target_os = "linux"))]
        let result = { let _ = (path, expected_revision, at); Err(LedgerError::UnsupportedPlatform) };
        if let Err(error) = &result {
            if self.state != PersistenceState::Unknown { self.state = PersistenceState::Error; }
            self.issue = Some(*error);
        }
        result
    }
    #[cfg(target_os = "linux")]
    fn commit_linux(&mut self, path: &Path, expected: u64, at: u64) -> Result<CommitReceipt, LedgerError> {
        if !valid_time(at) || self.bound_path.as_ref().is_some_and(|p| p.as_path() != path) {
            return Err(LedgerError::InvalidInput);
        }
        if expected != self.store_revision() { return Err(LedgerError::Conflict); }
        let dir = io::private_parent(path)?;
        let lock = io::OwnedFile::lock(path)?;
        let actual = io::read_disk(path)?;
        if actual.as_ref() != self.base_disk.as_ref() { return Err(LedgerError::Conflict); }
        if !self.dirty {
            let base = self.base_disk.as_ref().ok_or(LedgerError::InvalidData)?;
            if self.confirmed_revision != Some(base.revision) {
                // Retry only this filesystem confirmation after rename uncertainty;
                // never repeat an LLM request or add its counter again.
                io::sync_existing(path).map_err(|_| LedgerError::DurabilityUnknown)?;
                dir.sync_all().map_err(|_| LedgerError::DurabilityUnknown)?;
            }
            self.confirmed_revision = Some(base.revision); self.confirmed_at = Some(base.committed_at);
            self.state = PersistenceState::Durable; self.issue = None;
            let receipt = CommitReceipt { revision: base.revision.to_string(), changed: false };
            lock.remove().map_err(|_| LedgerError::LockCleanup)?;
            dir.sync_all().map_err(|_| LedgerError::LockCleanup)?;
            return Ok(receipt);
        }
        let revision = expected.checked_add(1).ok_or(LedgerError::Limit)?;
        let disk = Disk { version: DISK_VERSION, ledger_id: self.ledger_id.clone(), revision,
            since: self.since, committed_at: at, attempts: self.attempts.values().cloned().collect() };
        validate_disk(&disk)?;
        let encoded = io::encode(&disk)?;
        let mut temp = io::OwnedFile::temporary(path)?;
        io::write_sync(&mut temp, &encoded)?;
        std::fs::rename(&temp.path, path).map_err(|_| LedgerError::Io)?;
        temp.remove_on_drop = false;
        // Rename is already observable: CAS base advances even if dir sync fails.
        // The public confirmed revision/time stay unchanged until durable proof.
        self.base_disk = Some(disk); self.bound_path = Some(path.into()); self.dirty = false;
        self.state = PersistenceState::Unknown;
        dir.sync_all().map_err(|_| LedgerError::DurabilityUnknown)?;
        self.confirmed_revision = Some(revision); self.confirmed_at = Some(at);
        self.state = PersistenceState::Durable; self.issue = None;
        lock.remove().map_err(|_| LedgerError::LockCleanup)?;
        dir.sync_all().map_err(|_| LedgerError::LockCleanup)?;
        Ok(CommitReceipt { revision: revision.to_string(), changed: true })
    }
}

fn valid_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}
fn valid_time(t: u64) -> bool { t <= MAX_TIME }
fn valid_model(s: &Option<String>) -> bool {
    s.as_ref().is_none_or(|s| !s.is_empty() && s.len() <= 512 && !s.chars().any(char::is_control))
}
fn validate_start(start: &AttemptStart) -> Result<(), LedgerError> {
    if [&start.key.provider, &start.key.owner, &start.key.conversation, &start.key.attempt]
        .iter().any(|s| !valid_id(s)) || !valid_time(start.started_at) ||
        start.server_id.as_ref().is_some_and(|s| !valid_id(s)) || !valid_model(&start.requested_model) {
        return Err(LedgerError::InvalidData);
    }
    Ok(())
}
fn validate_snapshot(s: &ServerSnapshot) -> Result<(), LedgerError> {
    if s.sequence == 0 || !valid_time(s.observed_at) || !valid_model(&s.reported_model) { return Err(LedgerError::InvalidData); }
    Ok(())
}
fn validate_disk(disk: &Disk) -> Result<BTreeMap<AttemptKey, AttemptRecord>, LedgerError> {
    if disk.version != DISK_VERSION || !valid_id(&disk.ledger_id) || disk.revision == 0 ||
        !valid_time(disk.since) || !valid_time(disk.committed_at) || disk.attempts.len() > MAX_ATTEMPTS {
        return Err(LedgerError::InvalidData);
    }
    let mut seen = BTreeSet::new(); let mut records = BTreeMap::new();
    for r in &disk.attempts {
        validate_start(&r.start)?;
        if !seen.insert(r.start.key.clone()) || r.ended_at.is_some_and(|v| !valid_time(v)) ||
            (r.outcome == AttemptOutcome::Pending) != r.ended_at.is_none() { return Err(LedgerError::InvalidData); }
        if let Some(snapshot) = &r.snapshot { validate_snapshot(snapshot)?; }
        records.insert(r.start.key.clone(), r.clone());
    }
    Ok(records)
}

/// Linux-only persistence for the station's explicit project-private data path.
/// No libc/dependency import: only std's Linux OpenOptionsExt constants below.
/// This is not a sandbox against hostile same-UID replacement of ancestors.
#[cfg(target_os = "linux")]
mod io {
    use super::*;
    use std::fs::{self, File, OpenOptions};
    use std::io::{Read, Write};
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::sync::atomic::{AtomicU64, Ordering};
    const NOFOLLOW: i32 = 0o400000;
    const NONBLOCK: i32 = 0o4000;
    const DIRECTORY: i32 = 0o200000;
    static NEXT: AtomicU64 = AtomicU64::new(0);

    pub(super) fn private_parent(path: &Path) -> Result<File, LedgerError> {
        use std::path::Component;
        let name = path.file_name().and_then(|s| s.to_str()).ok_or(LedgerError::InvalidInput)?;
        if !path.is_absolute() || name.is_empty() || name.len() > 128 { return Err(LedgerError::InvalidInput); }
        let mut walked = PathBuf::new();
        for component in path.parent().ok_or(LedgerError::InvalidInput)?.components() {
            if !matches!(component, Component::RootDir | Component::Normal(_)) { return Err(LedgerError::InvalidInput); }
            walked.push(component.as_os_str());
            let metadata = fs::symlink_metadata(&walked).map_err(|_| LedgerError::Io)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() { return Err(LedgerError::InvalidData); }
        }
        let parent = path.parent().ok_or(LedgerError::InvalidInput)?;
        let file = OpenOptions::new().read(true).custom_flags(NOFOLLOW | NONBLOCK | DIRECTORY)
            .open(parent).map_err(|_| LedgerError::Io)?;
        let metadata = file.metadata().map_err(|_| LedgerError::Io)?;
        if !metadata.is_dir() || metadata.permissions().mode() & 0o777 != 0o700 { return Err(LedgerError::InvalidData); }
        Ok(file)
    }
    fn private_file(file: &File) -> Result<(), LedgerError> {
        let m = file.metadata().map_err(|_| LedgerError::Io)?;
        if !m.is_file() || m.nlink() != 1 || m.permissions().mode() & 0o777 != 0o600 { return Err(LedgerError::InvalidData); }
        Ok(())
    }
    pub(super) fn read_disk(path: &Path) -> Result<Option<Disk>, LedgerError> {
        let file = match OpenOptions::new().read(true).custom_flags(NOFOLLOW | NONBLOCK).open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(LedgerError::Io),
        };
        private_file(&file)?;
        if file.metadata().map_err(|_| LedgerError::Io)?.len() > MAX_BYTES as u64 { return Err(LedgerError::Limit); }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|_| LedgerError::Io)?;
        if bytes.len() > MAX_BYTES { return Err(LedgerError::Limit); }
        let disk: Disk = serde_json::from_slice(&bytes).map_err(|_| LedgerError::InvalidData)?;
        validate_disk(&disk)?; Ok(Some(disk))
    }
    pub(super) fn sync_existing(path: &Path) -> Result<(), LedgerError> {
        let file = OpenOptions::new().read(true).write(true).custom_flags(NOFOLLOW | NONBLOCK)
            .open(path).map_err(|_| LedgerError::Io)?;
        private_file(&file)?; file.sync_all().map_err(|_| LedgerError::Io)
    }
    struct Limited { bytes: Vec<u8> }
    impl Write for Limited {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len().saturating_add(buf.len()) > MAX_BYTES {
                return Err(std::io::Error::other("usage ledger limit"));
            }
            self.bytes.extend_from_slice(buf); Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    pub(super) fn encode(disk: &Disk) -> Result<Vec<u8>, LedgerError> {
        let mut limited = Limited { bytes: Vec::new() };
        serde_json::to_writer(&mut limited, disk).map_err(|_| LedgerError::Limit)?;
        Ok(limited.bytes)
    }
    pub(super) struct OwnedFile {
        pub(super) path: PathBuf,
        file: Option<File>, dev: u64, ino: u64,
        pub(super) remove_on_drop: bool,
    }
    impl OwnedFile {
        fn create(path: PathBuf, lock: bool) -> Result<Self, LedgerError> {
            let file = OpenOptions::new().write(true).create_new(true).mode(0o600)
                .custom_flags(NOFOLLOW | NONBLOCK).open(&path).map_err(|e| {
                    if lock && e.kind() == std::io::ErrorKind::AlreadyExists { LedgerError::Busy }
                    else { LedgerError::Io }
                })?;
            let m = file.metadata().map_err(|_| LedgerError::Io)?;
            let owned = Self { path, file: Some(file), dev: m.dev(), ino: m.ino(), remove_on_drop: true };
            owned.file.as_ref().ok_or(LedgerError::Io)?.set_permissions(fs::Permissions::from_mode(0o600)).map_err(|_| LedgerError::Io)?;
            private_file(owned.file.as_ref().ok_or(LedgerError::Io)?)?;
            Ok(owned)
        }
        pub(super) fn lock(target: &Path) -> Result<Self, LedgerError> {
            let mut name = target.file_name().ok_or(LedgerError::InvalidInput)?.to_os_string();
            name.push(".lock"); Self::create(target.with_file_name(name), true)
        }
        pub(super) fn temporary(target: &Path) -> Result<Self, LedgerError> {
            let n = NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1)).map_err(|_| LedgerError::Limit)?;
            let mut name = target.file_name().ok_or(LedgerError::InvalidInput)?.to_os_string();
            name.push(format!(".tmp.{}.{n}", std::process::id()));
            Self::create(target.with_file_name(name), false)
        }
        fn same_path(&self) -> bool {
            fs::symlink_metadata(&self.path).is_ok_and(|m| m.is_file() &&
                !m.file_type().is_symlink() && m.dev() == self.dev && m.ino() == self.ino)
        }
        pub(super) fn remove(mut self) -> Result<(), LedgerError> {
            if !self.same_path() { return Err(LedgerError::Conflict); }
            fs::remove_file(&self.path).map_err(|_| LedgerError::Io)?;
            self.remove_on_drop = false; self.file.take(); Ok(())
        }
    }
    impl Drop for OwnedFile {
        fn drop(&mut self) {
            // Never remove a replacement inode. A stale lock is never adopted or
            // cleared automatically on a later invocation; it returns Busy.
            if self.remove_on_drop && self.same_path() { let _ = fs::remove_file(&self.path); }
        }
    }
    pub(super) fn write_sync(temp: &mut OwnedFile, bytes: &[u8]) -> Result<(), LedgerError> {
        let file = temp.file.as_mut().ok_or(LedgerError::Io)?;
        file.write_all(bytes).map_err(|_| LedgerError::Io)?;
        file.sync_all().map_err(|_| LedgerError::Io)?;
        temp.file.take(); Ok(())
    }
}
