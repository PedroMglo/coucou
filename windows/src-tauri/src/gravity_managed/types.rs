use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const PROVIDER: &str = "antigravity";
pub(crate) const PET: &str = "agent_antigravity";
pub(crate) const MAX_LINE: usize = 1024 * 1024;
pub(crate) const MAX_QUERY: usize = 128 * 1024;
pub(crate) const MAX_REPLY: usize = 256 * 1024;
pub(crate) const MAX_HISTORY: usize = 2 * 1024 * 1024;
pub(crate) const MAX_MESSAGES: usize = 256;
pub(crate) const MAX_OWNERS: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Fault {
    InvalidConfig,
    InvalidQuery,
    WrongReference,
    Busy,
    NotReady,
    Capacity,
    Spawn,
    Io,
    Protocol,
    Identity,
    Limit,
    Deadline,
    Closed,
    Stopped,
    CleanupUnconfirmed,
    JournalUnavailable,
    JournalDurabilityUnknown,
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) executable: PathBuf,
    pub(crate) cwd: PathBuf,
    pub(crate) model: String,
    pub(crate) turn_timeout: Duration,
    pub(crate) init_timeout: Duration,
}

impl Config {
    /// Backend configuration only; never deserialize this from a session target.
    pub(crate) fn new(executable: &Path, cwd: &Path, model: &str) -> Result<Self, Fault> {
        if !executable.is_absolute()
            || !cwd.is_absolute()
            || model.is_empty()
            || model.len() > 256
            || !model
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
        {
            return Err(Fault::InvalidConfig);
        }
        let executable = executable
            .canonicalize()
            .map_err(|_| Fault::InvalidConfig)?;
        let cwd = cwd.canonicalize().map_err(|_| Fault::InvalidConfig)?;
        use std::os::unix::fs::PermissionsExt;
        let meta = executable.metadata().map_err(|_| Fault::InvalidConfig)?;
        if !meta.is_file() || meta.permissions().mode() & 0o111 == 0 || !cwd.is_dir() {
            return Err(Fault::InvalidConfig);
        }
        Ok(Self {
            executable,
            cwd,
            model: model.into(),
            turn_timeout: Duration::from_secs(1800),
            init_timeout: Duration::from_secs(30),
        })
    }

    pub(crate) fn validate(&self) -> Result<(), Fault> {
        let reviewed = Self::new(&self.executable, &self.cwd, &self.model)?;
        if reviewed.executable != self.executable
            || reviewed.cwd != self.cwd
            || self.turn_timeout.is_zero()
            || self.turn_timeout > Duration::from_secs(1800)
            || self.init_timeout.is_zero()
            || self.init_timeout > Duration::from_secs(30)
        {
            return Err(Fault::InvalidConfig);
        }
        Ok(())
    }
}

pub(crate) fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
        && value.bytes().any(|b| b.is_ascii_hexdigit() && b != b'0')
}

pub(crate) fn validate_query(query: &str) -> Result<(), Fault> {
    if query.trim().is_empty() || query.len() > MAX_QUERY || query.contains('\0') {
        Err(Fault::InvalidQuery)
    } else {
        Ok(())
    }
}

/// Opaque pending-birth token. A native Target does not exist until init.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Birth {
    pub(super) handle: String,
    pub(super) generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Reference {
    pub(crate) handle: String,
    pub(crate) generation: u64,
    pub(crate) provider: String,
    pub(crate) pet_id: String,
    pub(crate) session_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Phase {
    Starting,
    Active,
    Ready,
    Stopping,
    Lost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Delivery {
    NotSent,
    Writing,
    Sent,
    Observed,
    Completed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Outcome {
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
    Waiting,
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Receipt {
    /// Owner-local sequence, never a claimed native turn ID.
    pub(crate) turn: u64,
    pub(crate) delivery: Delivery,
    pub(crate) outcome: Option<Outcome>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Role {
    User,
    Assistant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Message {
    pub(crate) role: Role,
    pub(crate) text: String,
}

/// Classification only. Raw stderr, URLs, credentials and tool data never leave
/// the transport. A result SUCCESS is not a receipt for a tool effect.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Diagnostics {
    pub(crate) bytes: u64,
    pub(crate) seen: bool,
    pub(crate) permission_limited: bool,
    pub(crate) authentication_required: bool,
}

/// Receipt metadata only; no transcript persistence or native cleanup proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReceiptPersistenceState {
    Pending,
    Durable,
    Unavailable,
    Unknown,
    #[cfg(test)]
    Memory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReceiptPersistence {
    pub(crate) state: ReceiptPersistenceState,
    /// Last acknowledged store revision, independent of view revision.
    pub(crate) revision: Option<u64>,
    pub(crate) error: Option<Fault>,
}

impl ReceiptPersistence {
    pub(crate) fn pending() -> Self {
        Self {
            state: ReceiptPersistenceState::Pending,
            revision: None,
            error: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn memory() -> Self {
        Self {
            state: ReceiptPersistenceState::Memory,
            revision: None,
            error: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) metadata: ViewMetadata,
    pub(crate) reference: Option<Reference>,
    pub(crate) phase: Phase,
    pub(crate) receipt: Receipt,
    pub(crate) receipt_persistence: ReceiptPersistence,
    pub(crate) history: Vec<Message>,
    pub(crate) partial_reply: String,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) fault: Option<Fault>,
    pub(crate) cleanup_confirmed: bool,
    /// Deliberately false until a separate resume-only/exclusive-writer proof.
    pub(crate) terminal_handoff: bool,
    /// There is no invented generic stdin permission protocol.
    pub(crate) approvals: bool,
}

/// Private view contract. Wall-clock fields never determine ordering/delivery;
/// revision is scoped to this birth/handle+generation, and polling cannot bump it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ViewMetadata {
    pub(crate) created_at_ms: Option<u64>,
    pub(crate) updated_at_ms: Option<u64>,
    pub(crate) revision: u64,
    pub(crate) display_title: Option<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
}

fn epoch_ms() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| d.as_millis().try_into().ok())
}

impl ViewMetadata {
    pub(super) fn new() -> Self {
        let now = epoch_ms();
        Self {
            created_at_ms: now,
            updated_at_ms: now,
            revision: 1,
            display_title: None,
            cwd: None,
            model: None,
        }
    }
}

impl Snapshot {
    pub(crate) fn can_forget_lost(&self) -> bool {
        self.phase == Phase::Lost
            && self.cleanup_confirmed
            && self.receipt_persistence.state != ReceiptPersistenceState::Pending
    }

    pub(super) fn changed(&mut self) {
        // Lifetime input/output/message budgets bound observable transitions to
        // far below u64 (and JavaScript's exact-integer range).
        self.metadata.revision += 1;
        self.metadata.updated_at_ms = epoch_ms();
    }

    pub(crate) fn capabilities(&self) -> ViewCapabilities {
        ViewCapabilities {
            history: self.reference.is_some(),
            send: self.reference.is_some() && self.phase == Phase::Ready,
            stop_owner: matches!(self.phase, Phase::Starting | Phase::Active | Phase::Ready),
            interrupt_turn: false,
            approvals: false,
            resume: false,
            terminal_handoff: false,
            live_attach: false,
            forget_lost: self.can_forget_lost(),
            survives_view_close: true,
            survives_app_exit: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ViewCapabilities {
    pub(crate) history: bool,
    pub(crate) send: bool,
    pub(crate) stop_owner: bool,
    pub(crate) interrupt_turn: bool,
    pub(crate) approvals: bool,
    pub(crate) resume: bool,
    pub(crate) terminal_handoff: bool,
    pub(crate) live_attach: bool,
    pub(crate) forget_lost: bool,
    pub(crate) survives_view_close: bool,
    pub(crate) survives_app_exit: bool,
}

impl Fault {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::InvalidConfig => "invalidConfiguration",
            Self::InvalidQuery => "invalidQuery",
            Self::WrongReference => "wrongReference",
            Self::Busy => "busy",
            Self::NotReady => "notReady",
            Self::Capacity => "capacity",
            Self::Spawn => "spawnFailed",
            Self::Io => "io",
            Self::Protocol => "protocolError",
            Self::Identity => "identityMismatch",
            Self::Limit => "limit",
            Self::Deadline => "deadline",
            Self::Closed => "nativeClosed",
            Self::Stopped => "stopped",
            Self::CleanupUnconfirmed => "cleanupUnconfirmed",
            Self::JournalUnavailable => "journalUnavailable",
            Self::JournalDurabilityUnknown => "journalDurabilityUnknown",
        }
    }
}

/// Escape control characters visibly rather than silently changing path text.
pub(super) fn display_text(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
