//! Provenance for Codex threads created by our private launch operation.
//! This RAM-only registry never adopts a discovered thread, runs a tool,
//! answers a native request, cancels a turn or restores a lost connection.
use std::{collections::{BTreeMap, BTreeSet}, ffi::OsString, path::{Component, Path}};

use serde_json::Value;

use super::types::{ManagedPhase, ManagedSession, Origin, Target};

const MAX_SESSIONS: usize = 16;
const LOST: &str = "A ligação nativa terminou. O estado da sessão está por confirmar; a continuidade não foi garantida.";
const NEEDS_NATIVE: &str = "Esta sessão pediu uma ferramenta ou diálogo que precisa do cliente nativo. Abre o terminal; o pet não respondeu ao pedido.";
const EVENT_GAP: &str = "Houve perda de eventos da sessão nativa. O estado da execução está por confirmar. Abre o terminal para acompanhar a sessão.";

#[derive(Default)]
pub struct Registry {
    sessions: BTreeMap<String, ManagedSession>,
    // One bit per registered handle, so a delayed materialization read cannot
    // turn a completed first turn (Draft again) back into an Active old turn.
    observed_turns: BTreeSet<String>,
}

fn valid_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}

fn valid_turn(value: &str) -> bool { valid_text(value, 128) && value.trim() == value }

fn absolute_path(path: &Path, file: bool) -> Result<(), String> {
    let bytes = path.as_os_str().as_encoded_bytes();
    if !path.is_absolute() || bytes.len() > 4096
        || bytes.iter().any(|byte| *byte < 32 || *byte == 127)
        || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        || (file && path.file_name().is_none())
    {
        return Err("O caminho nativo deve ser absoluto e já validado.".into());
    }
    Ok(())
}

impl Registry {
    /// The private creator must serialize this check with native thread/start
    /// and register. A capacity check alone does not reserve a native thread.
    pub fn ensure_capacity(&self) -> Result<(), String> {
        if self.sessions.len() >= MAX_SESSIONS {
            Err("O pet já acompanha o limite de 16 sessões geridas. Liberta uma sessão antes de criar outra.".into())
        } else { Ok(()) }
    }

    /// Called only with the native ID returned by our private thread/start.
    /// Never expose this as a command accepting an arbitrary external target.
    pub fn register(&mut self, target: Target, generation: u64, cwd: String, model: Option<String>)
        -> Result<ManagedSession, String>
    {
        self.ensure_capacity()?;
        target.validate_external("codex", "agent_codex")?;
        if generation == 0 { return Err("Falta a geração da ligação nativa.".into()); }
        absolute_path(Path::new(&cwd), false)?;
        if model.as_ref().is_some_and(|value| !valid_text(value, 512) || value.trim() != value) {
            return Err("O modelo nativo tem metadados inválidos.".into());
        }
        if self.sessions.values().any(|session| session.target == target) {
            return Err("Esta sessão já tem um registo de criação gerida.".into());
        }
        // Opaque uniqueness, not a secret or a cryptographic capability.
        let handle = loop {
            let candidate = crate::assistant_memory::new_id();
            if !self.sessions.contains_key(&candidate) { break candidate; }
        };
        let session = ManagedSession {
            target, handle: handle.clone(), generation, phase: ManagedPhase::Draft,
            turn: None, materialized: false, terminal_launched: false, cwd, model, notice: None,
        };
        self.sessions.insert(handle, session.clone());
        Ok(session)
    }

    fn identified(&self, handle: &str, target: &Target, generation: u64)
        -> Result<&ManagedSession, String>
    {
        target.validate_external("codex", "agent_codex")?;
        let session = self.sessions.get(handle).ok_or("O registo desta sessão não existe.")?;
        if generation == 0 || session.generation != generation || &session.target != target {
            return Err("O registo não pertence a este destino e geração nativos.".into());
        }
        Ok(session)
    }

    pub fn get(&self, handle: &str, target: &Target, generation: u64) -> Result<ManagedSession, String> {
        let session = self.identified(handle, target, generation)?;
        if matches!(session.phase, ManagedPhase::Lost | ManagedPhase::Released) {
            return Err("A ligação deste registo terminou. Consulta a lista antes de continuar.".into());
        }
        Ok(session.clone())
    }

    /// Includes lost provenance for diagnosis; listing never restores it.
    pub fn list(&self) -> Vec<ManagedSession> { self.sessions.values().cloned().collect() }

    fn live_mut(&mut self, handle: &str) -> Result<&mut ManagedSession, String> {
        let session = self.sessions.get_mut(handle).ok_or("O registo desta sessão não existe.")?;
        if matches!(session.phase, ManagedPhase::Lost | ManagedPhase::Released) {
            return Err("Não é possível confirmar uma sessão cuja ligação terminou.".into());
        }
        Ok(session)
    }

    /// Only the creator calls this after verifying native turn/history storage.
    /// Native events may have advanced while that read was in flight: an old
    /// snapshot cannot replace an already observed Active/Ready/Needs state.
    pub fn materialized(&mut self, handle: &str, turn: Option<String>) -> Result<ManagedSession, String> {
        if turn.as_ref().is_some_and(|id| !valid_turn(id)) { return Err("O turno nativo é inválido.".into()); }
        let observed = self.observed_turns.contains(handle);
        let session = self.live_mut(handle)?;
        session.materialized = true;
        if session.phase == ManagedPhase::Draft {
            session.turn = if observed { None } else { turn };
            session.phase = if session.turn.is_some() { ManagedPhase::Active } else { ManagedPhase::Ready };
        }
        Ok(session.clone())
    }

    /// Process launch ACK is not evidence of a TUI connection, stored history,
    /// resolved tool request or continued model execution.
    pub fn terminal_result(&mut self, handle: &str, launched: bool, notice: Option<String>)
        -> Result<ManagedSession, String>
    {
        if notice.as_ref().is_some_and(|value| !valid_text(value, 2048)) {
            return Err("O aviso da abertura do terminal é inválido.".into());
        }
        let session = self.live_mut(handle)?;
        session.terminal_launched |= launched;
        if session.phase != ManagedPhase::NeedsNativeClient { session.notice = notice; }
        Ok(session.clone())
    }

    /// Caller proves native idle first. This only forgets our RAM provenance;
    /// it performs no interrupt, unsubscribe, terminal close or native write.
    /// A lost record may be forgotten using its original identity/generation.
    pub fn release(&mut self, handle: &str, target: &Target, generation: u64)
        -> Result<ManagedSession, String>
    {
        self.identified(handle, target, generation)?;
        let mut session = self.sessions.remove(handle).ok_or("O registo desta sessão não existe.")?;
        self.observed_turns.remove(handle);
        session.phase = ManagedPhase::Released;
        Ok(session)
    }

    pub fn disconnect(&mut self, generation: u64) -> Vec<Target> {
        let mut changed = Vec::new();
        for session in self.sessions.values_mut().filter(|session| session.generation == generation) {
            if session.phase != ManagedPhase::Lost {
                session.phase = ManagedPhase::Lost;
                session.notice = Some(LOST.into());
                changed.push(session.target.clone());
            }
        }
        changed
    }

    /// A lagged observer cannot confirm the current native execution state.
    /// Keep the last known metadata, but require the native client even after
    /// subsequent events or storage/terminal ACKs; only disconnect supersedes it.
    pub fn gap(&mut self, generation: u64) -> Vec<Target> {
        let mut changed = Vec::new();
        for session in self.sessions.values_mut().filter(|session|
            session.generation == generation && session.phase != ManagedPhase::Lost)
        {
            if session.phase != ManagedPhase::NeedsNativeClient || session.notice.as_deref() != Some(EVENT_GAP) {
                session.phase = ManagedPhase::NeedsNativeClient;
                session.notice = Some(EVENT_GAP.into());
                changed.push(session.target.clone());
            }
        }
        changed
    }

    /// Observe only native thread/turn identifiers and a few lifecycle states.
    /// Dynamic tool/MCP payloads, text, command arguments and outputs are never
    /// retained, interpreted or executed by this registry.
    pub fn event(&mut self, generation: u64, event: &Value) -> Vec<Target> {
        let Some(method) = event["method"].as_str() else { return Vec::new(); };
        let params = &event["params"];
        let explicit = params["threadId"].as_str();
        let nested = params["thread"]["id"].as_str();
        if explicit.zip(nested).is_some_and(|(a, b)| a != b) { return Vec::new(); }
        let Some(thread) = explicit.or(nested) else { return Vec::new(); };
        let turn = params["turn"]["id"].as_str();
        let flat_turn = params["turnId"].as_str();
        if turn.zip(flat_turn).is_some_and(|(a, b)| a != b) { return Vec::new(); }
        if params.get("turnId").is_some_and(|id| !(method=="mcpServer/elicitation/request"&&id.is_null())&&id.as_str().is_none_or(|id| !valid_turn(id)))
            || params["turn"].get("id").is_some_and(|id| id.as_str().is_none_or(|id| !valid_turn(id)))
        { return Vec::new(); }
        let turn = turn.or(flat_turn).filter(|id| valid_turn(id));
        let mut changed = Vec::new();
        for session in self.sessions.values_mut().filter(|session|
            session.generation == generation && session.target.session_id == thread
                && session.phase != ManagedPhase::Lost)
        {
            let before = session.clone();
            match method {
                "turn/started" => {
                    if let Some(turn) = turn {
                        self.observed_turns.insert(session.handle.clone());
                        session.turn = Some(turn.into());
                        if session.phase != ManagedPhase::NeedsNativeClient { session.phase = ManagedPhase::Active; }
                    }
                }
                "turn/completed" => {
                    if turn.is_some() && matches!(params["turn"]["status"].as_str(), Some("completed" | "failed" | "interrupted")) {
                        self.observed_turns.insert(session.handle.clone());
                    }
                    if turn.is_some() && session.turn.as_deref() == turn
                        && matches!(params["turn"]["status"].as_str(), Some("completed" | "failed" | "interrupted"))
                    {
                        session.turn = None;
                        if session.phase != ManagedPhase::NeedsNativeClient {
                            session.phase = if session.materialized { ManagedPhase::Ready } else { ManagedPhase::Draft };
                        }
                    }
                }
                "mcpServer/elicitation/request" if crate::pet_mcp_form::Form::supported(params) => {},
                "item/tool/call" | "mcpServer/elicitation/request" => {
                    if session.turn.is_none() || turn.is_none() || session.turn.as_deref() == turn {
                        if session.turn.is_none() { session.turn = turn.map(str::to_owned); }
                        if session.phase != ManagedPhase::NeedsNativeClient {
                            session.notice = Some(NEEDS_NATIVE.into());
                        }
                        session.phase = ManagedPhase::NeedsNativeClient;
                    }
                }
                "thread/closed" => { session.phase = ManagedPhase::Lost; session.notice = Some(LOST.into()); }
                _ => {}
            }
            if *session != before { changed.push(session.target.clone()); }
        }
        changed
    }
}

/// WezTerm argv only. The caller must canonicalize/check the socket and peer,
/// verify materialized provenance and then pass these arguments to Command.
/// Paths/UUID remain individual argv elements; no shell is involved.
pub fn terminal_arguments(codex_path: &Path, socket_path: &Path, cwd_path: &Path, thread_id: &str)
    -> Result<Vec<OsString>, String>
{
    absolute_path(codex_path, true)?;
    absolute_path(socket_path, true)?;
    absolute_path(cwd_path, false)?;
    let target = Target { pet_id: "agent_codex".into(), provider: "codex".into(), origin: Origin::External,
        session_id: thread_id.into(), conversation_id: None };
    target.validate_external("codex", "agent_codex")?;
    let mut socket = OsString::from("unix://"); socket.push(socket_path.as_os_str());
    Ok(vec!["start".into(), "--always-new-process".into(), "--no-auto-connect".into(), "--domain".into(), "local".into(), "--cwd".into(), cwd_path.as_os_str().into(),
        "--".into(), codex_path.as_os_str().into(), "--remote".into(), socket, "resume".into(), thread_id.into()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn target(index: u64) -> Target {
        Target { pet_id: "agent_codex".into(), provider: "codex".into(), origin: Origin::External,
            session_id: format!("019a734b-0000-7000-8000-{index:012x}"), conversation_id: None }
    }
    fn add(registry: &mut Registry, index: u64, generation: u64) -> ManagedSession {
        registry.register(target(index), generation, "/fixture/workspace".into(), Some("fixture-model".into())).unwrap()
    }
    fn entry(registry: &Registry, handle: &str) -> ManagedSession {
        registry.list().into_iter().find(|session| session.handle == handle).unwrap()
    }
    fn started(thread: &str, turn: &str) -> Value {
        json!({"method":"turn/started","params":{"threadId":thread,"turn":{"id":turn,"status":"inProgress"}}})
    }
    fn completed(thread: &str, turn: &str) -> Value {
        json!({"method":"turn/completed","params":{"threadId":thread,"turn":{"id":turn,"status":"completed"}}})
    }

    #[test]
    fn registration_is_unmaterialized_draft_with_unique_opaque_handles() {
        let mut registry = Registry::default();
        let first = add(&mut registry, 1, 1); let second = add(&mut registry, 2, 1);
        assert!(!first.handle.is_empty()); assert_ne!(first.handle, second.handle);
        assert_ne!(first.handle, first.target.session_id);
        assert_eq!(first.phase, ManagedPhase::Draft); assert!(!first.materialized);
        assert!(!first.terminal_launched); assert!(first.turn.is_none()); assert!(first.notice.is_none());
        assert_eq!(registry.get(&first.handle, &first.target, 1).unwrap(), first);
    }

    #[test]
    fn registration_rejects_other_owners_and_invalid_metadata() {
        let mut invalid = Vec::new();
        let mut value = target(1); value.pet_id = "agent_copilot".into(); invalid.push(value);
        let mut value = target(1); value.provider = "copilot".into(); invalid.push(value);
        let mut value = target(1); value.origin = Origin::Mochi; invalid.push(value);
        let mut value = target(1); value.conversation_id = Some("local-conversation".into()); invalid.push(value);
        let mut value = target(1); value.session_id = "not-native-uuid".into(); invalid.push(value);
        for value in invalid {
            let mut registry = Registry::default();
            assert!(registry.register(value, 1, "/fixture".into(), None).is_err()); assert!(registry.list().is_empty());
        }
        for (generation, cwd, model) in [(0, "/fixture".into(), None), (1, "relative".into(), None),
            (1, "/fixture".into(), Some("x".repeat(513))), (1, "/fixture".into(), Some("bad\nmodel".into()))] {
            assert!(Registry::default().register(target(1), generation, cwd, model).is_err());
        }
    }

    #[test]
    fn get_rejects_forged_handles_targets_and_generations() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 7);
        assert!(registry.get("invented", &session.target, 7).is_err());
        assert!(registry.get(&session.handle, &target(2), 7).is_err());
        assert!(registry.get(&session.handle, &session.target, 8).is_err());
        assert!(registry.get(&session.handle, &session.target, 0).is_err());
        assert_eq!(registry.list(), vec![session]);
    }

    #[test]
    fn capacity_is_bounded_and_lost_records_require_explicit_release() {
        let mut registry = Registry::default();
        let sessions: Vec<_> = (0..16).map(|index| add(&mut registry, index, 1)).collect();
        assert!(registry.ensure_capacity().is_err());
        assert!(registry.register(target(99), 1, "/fixture".into(), None).is_err());
        assert_eq!(registry.disconnect(1).len(), 16); assert!(registry.ensure_capacity().is_err());
        registry.release(&sessions[0].handle, &sessions[0].target, 1).unwrap();
        assert!(registry.ensure_capacity().is_ok()); add(&mut registry, 99, 2);
        assert_eq!(registry.list().len(), 16);
    }

    #[test]
    fn duplicate_thread_is_never_registered_as_a_new_managed_generation() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        assert!(registry.register(session.target.clone(), 1, "/fixture".into(), None).is_err());
        registry.disconnect(1);
        assert!(registry.register(session.target, 2, "/fixture".into(), None).is_err());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn only_explicit_verification_materializes_a_draft() {
        let mut registry = Registry::default(); let first = add(&mut registry, 1, 1);
        let ready = registry.materialized(&first.handle, None).unwrap();
        assert!(ready.materialized); assert_eq!(ready.phase, ManagedPhase::Ready);
        let second = add(&mut registry, 2, 1);
        let active = registry.materialized(&second.handle, Some("verified-turn".into())).unwrap();
        assert!(active.materialized); assert_eq!(active.phase, ManagedPhase::Active);
        assert_eq!(active.turn.as_deref(), Some("verified-turn"));
        assert!(registry.materialized(&second.handle, Some("bad\nturn".into())).is_err());
    }

    #[test]
    fn turn_start_is_live_activity_without_a_storage_promise() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        assert_eq!(registry.event(1, &started(&session.target.session_id, "turn-1")), vec![session.target.clone()]);
        let active = entry(&registry, &session.handle);
        assert_eq!(active.phase, ManagedPhase::Active); assert!(!active.materialized);
        assert_eq!(active.turn.as_deref(), Some("turn-1"));
        assert!(registry.event(1, &started(&session.target.session_id, "turn-1")).is_empty());
    }

    #[test]
    fn completing_an_unverified_turn_returns_to_draft() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        registry.event(1, &started(&session.target.session_id, "turn-1"));
        registry.event(1, &completed(&session.target.session_id, "turn-1"));
        let draft = entry(&registry, &session.handle);
        assert_eq!(draft.phase, ManagedPhase::Draft); assert!(!draft.materialized); assert!(draft.turn.is_none());
    }

    #[test]
    fn materialization_does_not_replace_a_newer_active_turn() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        registry.event(1, &started(&session.target.session_id, "newer-turn"));
        let verified = registry.materialized(&session.handle, Some("old-turn".into())).unwrap();
        assert!(verified.materialized); assert_eq!(verified.phase, ManagedPhase::Active);
        assert_eq!(verified.turn.as_deref(), Some("newer-turn"));
    }

    #[test]
    fn late_history_cannot_restart_a_first_turn_already_observed_completed() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        registry.event(1, &started(&session.target.session_id, "old-turn"));
        registry.event(1, &completed(&session.target.session_id, "old-turn"));
        let verified = registry.materialized(&session.handle, Some("old-turn".into())).unwrap();
        assert!(verified.materialized); assert_eq!(verified.phase, ManagedPhase::Ready); assert!(verified.turn.is_none());
    }

    #[test]
    fn stale_completion_never_clears_the_selected_active_turn() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        registry.materialized(&session.handle, None).unwrap();
        registry.event(1, &started(&session.target.session_id, "current"));
        assert!(registry.event(1, &completed(&session.target.session_id, "old")).is_empty());
        assert_eq!(entry(&registry, &session.handle).turn.as_deref(), Some("current"));
        registry.event(1, &completed(&session.target.session_id, "current"));
        assert_eq!(entry(&registry, &session.handle).phase, ManagedPhase::Ready);
    }

    #[test]
    fn unsupported_requests_need_native_client_and_never_retain_payloads() {
        for method in ["item/tool/call", "mcpServer/elicitation/request"] {
            let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
            registry.event(1, &started(&session.target.session_id, "turn-1"));
            let request = json!({"id":"request-1","method":method,"params":{"threadId":session.target.session_id,
                "turnId":"turn-1","arguments":{"secret":"PRIVATE_PAYLOAD"},"output":"PRIVATE_OUTPUT","text":"PRIVATE_TEXT"}});
            assert_eq!(registry.event(1, &request), vec![session.target.clone()]);
            let verified = registry.materialized(&session.handle, Some("old-turn".into())).unwrap();
            assert!(verified.materialized); assert_eq!(verified.phase, ManagedPhase::NeedsNativeClient);
            assert_eq!(verified.turn.as_deref(), Some("turn-1"));
            assert_eq!(registry.terminal_result(&session.handle, true, None).unwrap().phase, ManagedPhase::NeedsNativeClient);
            registry.event(1, &completed(&session.target.session_id, "turn-1"));
            assert_eq!(entry(&registry, &session.handle).phase, ManagedPhase::NeedsNativeClient);
            let retained = serde_json::to_string(&registry.list()).unwrap();
            assert!(!retained.contains("PRIVATE_")); assert!(!retained.contains("request-1"));
        }
    }

    #[test]fn supported_nullable_mcp_forms_do_not_claim_a_missing_native_client(){
        let mut registry=Registry::default();let session=add(&mut registry,1,1);registry.event(1,&started(&session.target.session_id,"turn-1"));
        let mut p=json!({"threadId":session.target.session_id,"turnId":null,"serverName":"fixture","mode":"form","message":"Synthetic","requestedSchema":{"type":"object","properties":{"x":{"type":"string","default":"PRIVATE_DEFAULT"}}}});
        assert!(registry.event(1,&json!({"id":7,"method":"mcpServer/elicitation/request","params":p})).is_empty());assert_eq!(entry(&registry,&session.handle).phase,ManagedPhase::Active);
        p["mode"]=json!("url");assert_eq!(registry.event(1,&json!({"id":8,"method":"mcpServer/elicitation/request","params":p})),vec![session.target.clone()]);assert_eq!(entry(&registry,&session.handle).phase,ManagedPhase::NeedsNativeClient);assert!(!serde_json::to_string(&registry.list()).unwrap().contains("PRIVATE_DEFAULT"));
    }

    #[test]
    fn closed_thread_is_lost_and_no_setter_or_later_event_restores_it() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        assert_eq!(registry.event(1, &json!({"method":"thread/closed","params":{"threadId":session.target.session_id}})), vec![session.target.clone()]);
        assert_eq!(entry(&registry, &session.handle).phase, ManagedPhase::Lost);
        assert!(registry.get(&session.handle, &session.target, 1).is_err());
        assert!(registry.materialized(&session.handle, None).is_err());
        assert!(registry.terminal_result(&session.handle, true, None).is_err());
        assert!(registry.event(1, &started(&session.target.session_id, "late")).is_empty());
    }

    #[test]
    fn disconnect_only_marks_matching_generation_and_keeps_inspectable_provenance() {
        let mut registry = Registry::default(); let first = add(&mut registry, 1, 7); let second = add(&mut registry, 2, 8);
        assert_eq!(registry.disconnect(7), vec![first.target.clone()]);
        assert!(registry.disconnect(7).is_empty()); assert_eq!(entry(&registry, &first.handle).phase, ManagedPhase::Lost);
        assert!(registry.get(&first.handle, &first.target, 8).is_err());
        assert!(registry.get(&second.handle, &second.target, 8).is_ok()); assert_eq!(registry.list().len(), 2);
    }

    #[test]
    fn gap_marks_only_the_exact_live_generation_and_preserves_lost_records() {
        let mut registry = Registry::default();
        let draft = add(&mut registry, 1, 7);
        let active = add(&mut registry, 2, 7);
        let lost = add(&mut registry, 3, 7);
        let other = add(&mut registry, 4, 8);
        registry.event(7, &started(&active.target.session_id, "current-turn"));
        registry.event(7, &serde_json::json!({"method":"thread/closed","params":{"threadId":lost.target.session_id}}));
        let old_lost = entry(&registry, &lost.handle);
        assert!(registry.gap(0).is_empty()); assert!(registry.gap(9).is_empty());
        let mut changed = registry.gap(7);
        changed.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        assert_eq!(changed, vec![draft.target.clone(), active.target.clone()]);
        for handle in [&draft.handle, &active.handle] {
            let session = entry(&registry, handle);
            assert_eq!(session.phase, ManagedPhase::NeedsNativeClient);
            assert_eq!(session.notice.as_deref(), Some(EVENT_GAP));
            assert!(!session.materialized); assert!(!session.terminal_launched);
        }
        assert_eq!(entry(&registry, &active.handle).turn.as_deref(), Some("current-turn"));
        assert_eq!(entry(&registry, &lost.handle), old_lost);
        assert_eq!(entry(&registry, &other.handle), other);
        assert!(registry.gap(7).is_empty());
    }

    #[test]
    fn gap_notice_survives_setters_and_events_until_disconnect_marks_lost() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 7);
        registry.event(7, &started(&session.target.session_id, "current-turn"));
        assert_eq!(registry.gap(7), vec![session.target.clone()]);
        let materialized = registry.materialized(&session.handle, Some("older-turn".into())).unwrap();
        assert!(materialized.materialized); assert_eq!(materialized.turn.as_deref(), Some("current-turn"));
        assert_eq!(materialized.phase, ManagedPhase::NeedsNativeClient);
        assert_eq!(materialized.notice.as_deref(), Some(EVENT_GAP));
        for launched in [true, false] {
            let result = registry.terminal_result(&session.handle, launched, Some("Aviso tardio do terminal.".into())).unwrap();
            assert_eq!(result.phase, ManagedPhase::NeedsNativeClient);
            assert_eq!(result.notice.as_deref(), Some(EVENT_GAP));
        }
        for method in ["item/tool/call", "mcpServer/elicitation/request"] {
            registry.event(7, &json!({"method":method,"params":{"threadId":session.target.session_id,"turnId":"current-turn","arguments":{"text":"PRIVATE_PAYLOAD"}}}));
            assert_eq!(entry(&registry, &session.handle).notice.as_deref(), Some(EVENT_GAP));
        }
        registry.event(7, &completed(&session.target.session_id, "current-turn"));
        let completed = entry(&registry, &session.handle);
        assert!(completed.turn.is_none()); assert_eq!(completed.phase, ManagedPhase::NeedsNativeClient);
        assert_eq!(completed.notice.as_deref(), Some(EVENT_GAP));
        registry.event(7, &started(&session.target.session_id, "later-turn"));
        assert_eq!(entry(&registry, &session.handle).phase, ManagedPhase::NeedsNativeClient);
        assert_eq!(entry(&registry, &session.handle).notice.as_deref(), Some(EVENT_GAP));
        assert!(!serde_json::to_string(&registry.list()).unwrap().contains("PRIVATE_PAYLOAD"));
        assert_eq!(registry.disconnect(7), vec![session.target.clone()]);
        let lost = entry(&registry, &session.handle);
        assert_eq!(lost.phase, ManagedPhase::Lost); assert_eq!(lost.notice.as_deref(), Some(LOST));
        assert!(registry.gap(7).is_empty());
        assert_eq!(entry(&registry, &session.handle), lost);
        assert!(registry.materialized(&session.handle, None).is_err());
        assert!(registry.terminal_result(&session.handle, true, None).is_err());
    }

    #[test]
    fn events_require_exact_registered_thread_generation_and_valid_ids() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        assert!(registry.event(2, &started(&session.target.session_id, "turn")).is_empty());
        assert!(registry.event(1, &started(&target(2).session_id, "turn")).is_empty());
        for event in [json!({"method":"turn/started","params":{"sessionId":session.target.session_id,"turn":{"id":"turn"}}}),
            json!({"method":"turn/started","params":{"threadId":session.target.session_id,"thread":{"id":target(2).session_id},"turn":{"id":"turn"}}}),
            json!({"method":"turn/started","params":{"threadId":session.target.session_id,"turn":{"id":"turn"},"turnId":"other"}}),
            json!({"method":"turn/started","params":{"threadId":session.target.session_id,"turn":{"id":"bad\nturn"}}}),
            json!({"method":"item/tool/call","params":{"threadId":session.target.session_id,"turnId":17}}),
            json!({"method":"item/agentMessage/delta","params":{"threadId":session.target.session_id,"delta":"PRIVATE_TEXT"}})] {
            assert!(registry.event(1, &event).is_empty());
        }
        assert_eq!(registry.list(), vec![session]);
    }

    #[test]
    fn release_is_identity_bound_forgets_lost_record_and_has_no_native_side_effects() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        registry.disconnect(1);
        assert!(registry.release(&session.handle, &target(2), 1).is_err());
        assert!(registry.release(&session.handle, &session.target, 2).is_err());
        let released = registry.release(&session.handle, &session.target, 1).unwrap();
        assert_eq!(released.phase, ManagedPhase::Released); assert_eq!(released.target, session.target);
        assert!(registry.list().is_empty()); assert!(registry.ensure_capacity().is_ok());
        assert!(registry.event(1, &started(&session.target.session_id, "late")).is_empty());
    }

    #[test]
    fn terminal_launch_ack_never_materializes_or_promises_a_native_connection() {
        let mut registry = Registry::default(); let session = add(&mut registry, 1, 1);
        let launched = registry.terminal_result(&session.handle, true, None).unwrap();
        assert!(launched.terminal_launched); assert!(!launched.materialized); assert_eq!(launched.phase, ManagedPhase::Draft);
        let later_failure = registry.terminal_result(&session.handle, false, Some("Não abriu outra janela.".into())).unwrap();
        assert!(later_failure.terminal_launched); assert_eq!(later_failure.phase, ManagedPhase::Draft);
        assert!(registry.terminal_result(&session.handle, false, Some("x".repeat(2049))).is_err());
    }

    #[test]
    fn terminal_arguments_are_literal_and_reject_invalid_paths_or_thread_ids() {
        let id = target(1).session_id;
        let args = terminal_arguments(Path::new("/fixture/bin/codex"), Path::new("/fixture/socket with spaces.sock"),
            Path::new("/fixture/project $(literal); name"), &id).unwrap();
        let expected: Vec<OsString> = vec!["start", "--always-new-process", "--no-auto-connect", "--domain", "local", "--cwd", "/fixture/project $(literal); name",
            "--", "/fixture/bin/codex", "--remote", "unix:///fixture/socket with spaces.sock", "resume", &id]
            .into_iter().map(OsString::from).collect();
        assert_eq!(args, expected);
        for (exe, socket, cwd, thread) in [
            ("relative-codex", "/fixture/socket", "/fixture", id.as_str()),
            ("/fixture/codex", "relative-socket", "/fixture", id.as_str()),
            ("/fixture/codex", "/fixture/socket", "relative-cwd", id.as_str()),
            ("/fixture/codex", "/fixture/../socket", "/fixture", id.as_str()),
            ("/", "/fixture/socket", "/fixture", id.as_str()),
            ("/fixture/codex", "/fixture/so\0cket", "/fixture", id.as_str()),
            ("/fixture/codex", "/fixture/socket", "/fixture", "--unexpected-option"),
        ] { assert!(terminal_arguments(Path::new(exe), Path::new(socket), Path::new(cwd), thread).is_err()); }
    }
}
