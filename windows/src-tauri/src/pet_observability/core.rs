//! Portable, read-only metadata aggregation. No provider or filesystem access.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const LIST_LIMIT: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Coverage { Complete, Partial, Unavailable, NotSupported }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Activity { Working, Waiting, Busy, Reserved, Idle, NotLoaded, Error, Unknown }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin { Mochi, External, Unknown }

/// Navigation only. It never grants permission to send, stop, resume or approve.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OpenTarget {
    Mochi { pet_id: String, provider: String, conversation_id: String },
    External { pet_id: String, provider: String, session_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub key: String,
    /// Runtime backend where known; a library owner is separate in OpenTarget.
    pub provider: String,
    pub pet_id: Option<String>,
    pub origin: Origin,
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub title: String,
    pub cwd: Option<String>,
    pub state: Activity,
    /// None means the source does not provide archive membership.
    pub archived: Option<bool>,
    pub updated_at: Option<u64>,
    pub observed_at: Option<u64>,
    pub open: Option<OpenTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryInput {
    pub scope: String,
    pub coverage: Coverage,
    pub observed_at: Option<u64>,
    /// Sanitized stable explanation. Do not expose raw provider error text.
    pub reason: Option<String>,
    pub rows: Vec<SessionRow>,
}

#[derive(Clone, Debug)]
pub struct PetBinding {
    pub pet_id: String,
    pub provider: String,
    pub owner: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct Ownership {
    /// None preserves an old/ambiguous binding without guessing its backend.
    pub backend: Option<String>,
    pub session_id: String,
    pub pet_id: Option<String>,
    pub owner: String,
    pub conversation_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct LibraryCapture {
    pub inventories: BTreeMap<String, InventoryInput>,
    pub ownership: Vec<Ownership>,
    pub ownership_complete: bool,
}

/// Re-project cached raw native rows onto the current library. Never remove
/// native rows or infer ownership from recency, cwd or a display name.
pub fn bind_ownership(rows: &mut [SessionRow], hints: &[Ownership], ownership_complete: bool) {
    let mut index: BTreeMap<&str, Vec<&Ownership>> = BTreeMap::new();
    for hint in hints { index.entry(&hint.session_id).or_default().push(hint); }
    for row in rows {
        if !ownership_complete {
            row.origin = Origin::Unknown;
            row.pet_id = None;
            row.conversation_id = None;
            row.open = None;
            continue;
        }
        let Some(id) = row.session_id.as_deref() else { continue };
        let candidates: Vec<_> = index.get(id).into_iter().flatten().copied().filter(|hint|
            hint.backend.as_deref().is_none_or(|p| p == row.provider)).collect();
        let identities: std::collections::BTreeSet<_> = candidates.iter()
            .map(|h| (h.owner.as_str(), h.conversation_id.as_str())).collect();
        if identities.len() == 1 && candidates.iter().all(|h| h.backend.as_deref() == Some(row.provider.as_str())) {
            let hint = candidates[0];
            let pets: std::collections::BTreeSet<_> = candidates.iter().filter_map(|h| h.pet_id.as_ref()).collect();
            row.origin = Origin::Mochi;
            row.pet_id = (pets.len() == 1).then(|| (*pets.iter().next().unwrap()).clone());
            row.conversation_id = Some(hint.conversation_id.clone());
            row.open = row.pet_id.as_ref().map(|pet_id| OpenTarget::Mochi { pet_id: pet_id.clone(),
                provider: hint.owner.clone(), conversation_id: hint.conversation_id.clone() });
        } else if !candidates.is_empty() {
            row.origin = Origin::Unknown;
            row.pet_id = None;
            row.conversation_id = None;
            row.open = None;
        }
    }
}

impl InventoryInput {
    pub fn unavailable(scope: &str, reason: &str) -> Self {
        Self { scope: scope.into(), coverage: Coverage::Unavailable,
            observed_at: None, reason: Some(reason.into()), rows: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Count {
    /// Exact only inside the declared scope and collection interval; never a
    /// provider-account global assertion. Null when any relevant evidence lacks.
    pub value: Option<u64>,
    /// Known subtotal in the observed set. Not an estimated global total.
    pub observed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub total: Count,
    pub archived: Count,
    pub confirmed_active: Count,
    pub working: Count,
    pub waiting: Count,
    pub reserved: Count,
    pub idle: Count,
    /// Rows with unknown current runtime state, including contradictory input.
    pub unknown: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub scope: String,
    pub coverage: Coverage,
    pub observed_at: Option<u64>,
    pub stale: bool,
    pub refreshing: bool,
    pub reason: Option<String>,
    pub counts: Counts,
    /// Bounded list only. Counts always describe all observed rows, before limit.
    pub sessions: Vec<SessionRow>,
    pub listed: usize,
    pub limit: usize,
    pub truncated: bool,
}

fn rank(state: Activity) -> u8 {
    match state { Activity::Waiting => 0, Activity::Working => 1,
        Activity::Busy => 2, Activity::Reserved => 3, Activity::Unknown => 4, Activity::Error => 5,
        Activity::Idle => 6, Activity::NotLoaded => 7 }
}

impl Inventory {
    pub fn refresh_freshness(&mut self, now: u64, max_age: u64) {
        self.stale = self.reason.as_deref() == Some("refreshFailed")
            || self.observed_at.is_none_or(|stamp| stamp > now || now - stamp > max_age);
    }

    pub fn mark_refresh_failed(&mut self) {
        self.coverage = if self.observed_at.is_some() { Coverage::Partial } else { Coverage::Unavailable };
        self.reason = Some("refreshFailed".into());
        self.stale = true;
        self.refreshing = false;
        for count in [&mut self.counts.total, &mut self.counts.archived, &mut self.counts.confirmed_active,
            &mut self.counts.working, &mut self.counts.waiting, &mut self.counts.reserved, &mut self.counts.idle] {
            count.value = None;
        }
    }

    pub fn build(input: InventoryInput, now: u64, max_age: u64, refreshing: bool) -> Self {
        let stale = input.observed_at.is_none_or(|stamp| stamp > now || now - stamp > max_age);
        let mut coverage = input.coverage;
        let mut reason = input.reason;
        let mut rows: BTreeMap<String, SessionRow> = BTreeMap::new();
        for row in input.rows {
            if row.key.is_empty() {
                coverage = Coverage::Partial;
                reason = Some("invalidIdentity".into());
                continue;
            }
            if let Some(previous) = rows.get_mut(&row.key) {
                if previous != &row {
                    // Conflicting observations must not select an arbitrary
                    // current owner, archive flag or action destination.
                    previous.state = Activity::Unknown;
                    previous.archived = None;
                    previous.origin = Origin::Unknown;
                    previous.pet_id = None;
                    previous.conversation_id = None;
                    previous.open = None;
                    coverage = Coverage::Partial;
                    reason = Some("conflictingIdentity".into());
                }
            } else { rows.insert(row.key.clone(), row); }
        }
        let n = |predicate: fn(&SessionRow) -> bool| rows.values().filter(|row| predicate(row)).count() as u64;
        let total = rows.len() as u64;
        let unknown = n(|r| r.state == Activity::Unknown);
        let busy = n(|r| r.state == Activity::Busy);
        let active = n(|r| matches!(r.state, Activity::Working | Activity::Waiting | Activity::Busy));
        let complete = coverage == Coverage::Complete;
        let metric = |observed, certain| Count { value: (complete && certain).then_some(observed), observed };
        let counts = Counts {
            total: metric(total, true),
            archived: metric(n(|r| r.archived == Some(true)), n(|r| r.archived.is_none()) == 0),
            confirmed_active: metric(active, unknown == 0),
            working: metric(n(|r| r.state == Activity::Working), unknown == 0 && busy == 0),
            waiting: metric(n(|r| r.state == Activity::Waiting), unknown == 0 && busy == 0),
            reserved: metric(n(|r| r.state == Activity::Reserved), unknown == 0),
            idle: metric(n(|r| r.state == Activity::Idle), unknown == 0),
            unknown,
        };
        let mut sessions: Vec<_> = rows.into_values().collect();
        sessions.sort_by(|a, b| rank(a.state).cmp(&rank(b.state))
            .then_with(|| b.updated_at.cmp(&a.updated_at)).then_with(|| a.key.cmp(&b.key)));
        let truncated = sessions.len() > LIST_LIMIT;
        sessions.truncate(LIST_LIMIT);
        let listed = sessions.len();
        Self { scope: input.scope, coverage, observed_at: input.observed_at,
            stale, refreshing, reason, counts, sessions, listed, limit: LIST_LIMIT, truncated }
    }
}
