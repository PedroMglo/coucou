//! Read-only adapters for existing native endpoints. No process discovery,
//! launcher, thread/resume, history, subscription or execution is performed.

use std::collections::{BTreeMap, HashSet};

use serde_json::{json, Value};

use crate::pet_observability::core::{
    Activity, Coverage, InventoryInput, OpenTarget, Origin, SessionRow,
};
use crate::session_chat::rpc::Client;
use crate::session_chat::types::{Origin as TargetOrigin, Target};

const CODEX_SCOPE: &str = "codexDaemon";
const COPILOT_SCOPE: &str = "copilotLiveEndpoints";
const PAGE_SIZE: usize = 100;
const MAX_SESSIONS: usize = 100_000;
// Bound even empty/duplicate-only pages and retained opaque cursor strings.
const MAX_PAGES: usize = 100_001;
const MAX_CURSOR_BYTES: usize = 8 * 1024 * 1024;
const MAX_METADATA_BYTES: usize = 128 * 1024 * 1024;
const SOURCES: &[&str] = &[
    "cli", "vscode", "exec", "appServer", "subAgent", "subAgentReview",
    "subAgentCompact", "subAgentThreadSpawn", "subAgentOther", "unknown",
];

const INVALID: &str = "codexInventoryInvalid";
const UNAVAILABLE: &str = "codexInventoryUnavailable";
const LIMIT: &str = "nativeInventoryLimit";
const PAGINATION: &str = "codexInventoryPagination";
const CONFLICT: &str = "nativeInventoryConflict";

struct Pagination {
    seen: HashSet<String>,
    pages: usize,
    bytes: usize,
}

impl Pagination {
    fn new() -> Self {
        Self { seen: HashSet::new(), pages: 0, bytes: 0 }
    }

    fn page(&mut self) -> Result<(), String> {
        self.pages += 1;
        if self.pages > MAX_PAGES { return Err(LIMIT.into()); }
        Ok(())
    }

    fn next(&mut self, page: &Value) -> Result<Option<String>, String> {
        // Both omission and null are documented end-of-list representations.
        let value = match page.get("nextCursor") {
            None | Some(Value::Null) => return Ok(None),
            Some(Value::String(value)) if !value.is_empty() => value,
            _ => return Err(PAGINATION.into()),
        };
        self.bytes = self.bytes.checked_add(value.len()).ok_or(LIMIT)?;
        if self.bytes > MAX_CURSOR_BYTES { return Err(LIMIT.into()); }
        if !self.seen.insert(value.clone()) { return Err(PAGINATION.into()); }
        Ok(Some(value.clone()))
    }
}

fn target(provider: &str, pet_id: &str, id: &str) -> Result<Target, String> {
    // Reject an oversized metadata ID before cloning it into a Target.
    if id.len() != 36 { return Err(INVALID.into()); }
    let target = Target {
        pet_id: pet_id.into(), provider: provider.into(), origin: TargetOrigin::External,
        session_id: id.into(), conversation_id: None,
    };
    target.validate_external(provider, pet_id).map_err(|_| INVALID.to_string())?;
    Ok(target)
}

fn timestamp(value: Option<&Value>) -> Option<u64> {
    // The native timestamp is signed Unix seconds. Invalid values are unknown,
    // never a fabricated epoch-zero time or a timestamp in another unit.
    value.and_then(Value::as_i64).and_then(|value| u64::try_from(value).ok())
        .filter(|value| *value <= 9_007_199_254_740_991)
}

fn codex_row(thread: &Value, archived: Option<bool>, observed_at: u64) -> Result<SessionRow, String> {
    let id = thread.get("id").and_then(Value::as_str).ok_or(INVALID)?;
    let target = target("codex", "agent_codex", id)?;
    let state = match thread.get("status").and_then(|status| status.get("type")).and_then(Value::as_str) {
        Some("active") => Activity::Busy,
        Some("idle") => Activity::Idle,
        Some("notLoaded") => Activity::NotLoaded,
        Some("systemError") => Activity::Error,
        Some(_) => Activity::Unknown,
        None => return Err(INVALID.into()),
    };
    let cwd = thread.get("cwd").and_then(Value::as_str).ok_or(INVALID)?;
    let title = thread.get("name").and_then(Value::as_str).filter(|s| !s.trim().is_empty())
        .or_else(|| thread.get("preview").and_then(Value::as_str).filter(|s| !s.trim().is_empty()))
        .unwrap_or("Sessão Codex");
    // Same history capability as session_chat::history::summary. Navigation
    // grants no send/cancel permission and cannot resume an ephemeral thread.
    let history = thread.get("ephemeral").and_then(Value::as_bool) != Some(true);
    Ok(SessionRow {
        key: format!("codex:{id}"), provider: "codex".into(),
        pet_id: Some("agent_codex".into()), origin: Origin::External,
        session_id: Some(target.session_id.clone()), conversation_id: None,
        title: title.into(), cwd: (!cwd.is_empty()).then(|| cwd.into()),
        state, archived, updated_at: timestamp(thread.get("updatedAt")),
        observed_at: Some(observed_at),
        open: history.then(|| OpenTarget::External {
            pet_id: target.pet_id, provider: target.provider, session_id: target.session_id,
        }),
    })
}

fn insert(rows: &mut BTreeMap<String, SessionRow>, bytes: &mut usize, row: SessionRow) -> Result<(), String> {
    if let Some(previous) = rows.get(&row.key) {
        // In particular, membership in both archive partitions is not resolved
        // by arbitrarily choosing the first page's archive/runtime state.
        return if previous == &row { Ok(()) } else { Err(CONFLICT.into()) };
    }
    if rows.len() >= MAX_SESSIONS { return Err(LIMIT.into()); }
    let navigation_size = match &row.open {
        Some(OpenTarget::External { pet_id, provider, session_id }) =>
            pet_id.len().saturating_add(provider.len()).saturating_add(session_id.len()),
        Some(OpenTarget::Mochi { pet_id, provider, conversation_id }) =>
            pet_id.len().saturating_add(provider.len()).saturating_add(conversation_id.len()),
        None => 0,
    };
    let size = row.key.len().saturating_add(row.provider.len())
        .saturating_add(row.pet_id.as_ref().map_or(0, String::len))
        .saturating_add(row.session_id.as_ref().map_or(0, String::len))
        .saturating_add(row.conversation_id.as_ref().map_or(0, String::len))
        .saturating_add(row.title.len()).saturating_add(row.cwd.as_ref().map_or(0, String::len))
        .saturating_add(navigation_size);
    *bytes = bytes.checked_add(size).ok_or(LIMIT)?;
    if *bytes > MAX_METADATA_BYTES { return Err(LIMIT.into()); }
    rows.insert(row.key.clone(), row);
    Ok(())
}

fn data(page: &Value) -> Result<&Vec<Value>, String> {
    let data = page.get("data").and_then(Value::as_array).ok_or(INVALID)?;
    if data.len() > PAGE_SIZE { return Err(LIMIT.into()); }
    Ok(data)
}

/// Enumerate one already-connected daemon and its stored/loaded thread union.
/// Complete is scoped to these endpoints and the declared sourceKinds; it is
/// not an account-global inventory or an atomic observation of all statuses.
pub(crate) async fn codex_inventory(client: &Client, observed_at: u64) -> Result<InventoryInput, String> {
    let mut rows = BTreeMap::new();
    let mut bytes = 0;
    for archived in [false, true] {
        let mut cursor = Value::Null;
        let mut pagination = Pagination::new();
        loop {
            pagination.page()?;
            let page = client.call("thread/list", json!({
                "limit": PAGE_SIZE, "cursor": cursor, "sortKey": "updated_at",
                "sortDirection": "desc", "sourceKinds": SOURCES, "archived": archived,
            })).await.map_err(|_| UNAVAILABLE.to_string())?;
            for thread in data(&page)? {
                insert(&mut rows, &mut bytes, codex_row(thread, Some(archived), observed_at)?)?;
            }
            let Some(next) = pagination.next(&page)? else { break };
            cursor = json!(next);
        }
    }

    let mut cursor = Value::Null;
    let mut pagination = Pagination::new();
    loop {
        pagination.page()?;
        let page = client.call("thread/loaded/list", json!({ "limit": PAGE_SIZE, "cursor": cursor }))
            .await.map_err(|_| UNAVAILABLE.to_string())?;
        for id in data(&page)? {
            let id = id.as_str().ok_or(INVALID)?;
            target("codex", "agent_codex", id)?;
            if rows.contains_key(&format!("codex:{id}")) { continue; }
            if rows.len() >= MAX_SESSIONS { return Err(LIMIT.into()); }
            // includeTurns:false is metadata only. Do not call snapshot,
            // thread/resume, turns/list, items/list or a model/account endpoint.
            let response = client.call("thread/read", json!({ "threadId": id, "includeTurns": false }))
                .await.map_err(|_| UNAVAILABLE.to_string())?;
            let thread = response.get("thread").ok_or(INVALID)?;
            if thread.get("id").and_then(Value::as_str) != Some(id) { return Err(INVALID.into()); }
            // Loaded-only does not establish archive membership.
            insert(&mut rows, &mut bytes, codex_row(thread, None, observed_at)?)?;
        }
        let Some(next) = pagination.next(&page)? else { break };
        cursor = json!(next);
    }
    Ok(InventoryInput {
        scope: CODEX_SCOPE.into(), coverage: Coverage::Complete,
        observed_at: Some(observed_at), reason: None,
        rows: rows.into_values().collect(),
    })
}

/// Only opt-in live bridge endpoints are visible. The existing list API hides
/// missing/unreadable directories and failed endpoints, so even [] is partial.
pub(crate) async fn copilot_inventory(observed_at: u64) -> Result<InventoryInput, String> {
    let summaries = crate::copilot_session::list().await;
    let mut rows = BTreeMap::new();
    let mut bytes = 0;
    for summary in summaries {
        let target = summary.target;
        target.validate_external("copilot", "agent_copilot")
            .map_err(|_| "copilotInventoryInvalid".to_string())?;
        let row = SessionRow {
            key: format!("copilot:{}", target.session_id), provider: "copilot".into(),
            pet_id: Some("agent_copilot".into()), origin: Origin::External,
            session_id: Some(target.session_id.clone()), conversation_id: None,
            title: summary.title, cwd: (!summary.cwd.is_empty()).then_some(summary.cwd),
            // Metadata/mtime and a listening socket do not prove a running turn.
            state: Activity::Unknown, archived: None,
            updated_at: u64::try_from(summary.updated_at).ok().filter(|value| *value <= 9_007_199_254_740_991), observed_at: Some(observed_at),
            open: summary.capabilities.history.then(|| OpenTarget::External {
                pet_id: target.pet_id, provider: target.provider, session_id: target.session_id,
            }),
        };
        insert(&mut rows, &mut bytes, row)?;
    }
    Ok(InventoryInput {
        scope: COPILOT_SCOPE.into(), coverage: Coverage::Partial,
        observed_at: Some(observed_at), reason: Some("liveEndpointsOnly".into()),
        rows: rows.into_values().collect(),
    })
}
