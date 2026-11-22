//! Passive, exact-target native observations. No transport or account IO.
use std::collections::HashMap;
use serde::Serialize;
use crate::session_chat::types::{Snapshot, Target};
use super::{Availability, Capability, Decision, Reason, Source, Support};

const MAX_TARGETS: usize = 128;
const PUBLISH_MS: u64 = 45_000;
const TTL_MS: u64 = 15_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetDescriptor {
    pub schema_version: u32,
    pub target: Target,
    pub revision: String,
    pub observed_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub busy: Option<bool>,
    pub turn: Option<String>,
    pub model: Option<String>,
    pub capabilities: Vec<Capability>,
}

// Neither tickets nor evidence implement Deserialize; no IPC writer exists.
pub(crate) struct Ticket { target: Target, sequence: u64, started: u64 }
#[derive(Clone)]
struct Evidence {
    at: u64, busy: bool, conflict: bool, turn: Option<String>, model: Option<String>,
    history: bool, send: bool, cancel: bool,
}
struct Slot { sequence: u64, started: u64, evidence: Option<Evidence>, reason: Reason }
pub(super) struct Store { instance: String, sequence: u64, slots: HashMap<Target, Slot> }

pub(super) fn validate(target: &Target) -> Result<(), String> {
    let pet = match target.provider.as_str() {
        "codex" => "agent_codex", "copilot" => "agent_copilot", "antigravity" => "agent_antigravity",
        _ => return Err("Este pet não disponibiliza observações de sessões nativas.".into()),
    };
    target.validate_external(&target.provider, pet)
}

impl Store {
    pub fn new(instance: String) -> Self { Self { instance, sequence: 0, slots: HashMap::new() } }
    pub fn begin(&mut self, target: &Target, now: u64) -> Option<Ticket> {
        validate(target).ok()?;
        if now == 0 || now > super::cache::MAX_DATE { return None; }
        let sequence = self.sequence.checked_add(1)?;
        self.sequence = sequence;
        if self.slots.len() >= MAX_TARGETS && !self.slots.contains_key(target) {
            let oldest = self.slots.iter().min_by_key(|(_, s)| s.sequence).map(|(k, _)| k.clone())?;
            self.slots.remove(&oldest);
        }
        self.slots.insert(target.clone(), Slot { sequence, started: now, evidence: None, reason: Reason::Unknown });
        Some(Ticket { target: target.clone(), sequence, started: now })
    }
    pub fn finish(&mut self, ticket: Ticket, snapshot: Option<&Snapshot>, now: u64) -> bool {
        let Some(slot) = self.slots.get_mut(&ticket.target) else { return false; };
        if slot.sequence != ticket.sequence { return false; }
        slot.reason = Reason::ProviderUnavailable;
        if now < ticket.started || now > super::cache::MAX_DATE || now - ticket.started > PUBLISH_MS {
            slot.reason = Reason::StaleObservation;
            return false;
        }
        let Some(s) = snapshot.filter(|s| s.target == ticket.target) else { return false; };
        if s.turn.as_ref().is_some_and(|t| t.is_empty() || t.len() > 1024 || t.chars().any(char::is_control)) { return false; }
        slot.evidence = Some(Evidence {
            at: now, busy: s.busy, conflict: s.control_conflict, turn: s.turn.clone(),
            model: s.model.as_ref().filter(|m| super::cache::model_id(m)).cloned(),
            history: s.capabilities.history, send: s.capabilities.send, cancel: s.capabilities.cancel,
        });
        // Consume the publication right even if an internal caller retained a ticket.
        slot.reason = Reason::Unknown;
        true
    }
    pub fn invalidate(&mut self, target: &Target) {
        if let Some(slot) = self.slots.get_mut(target) {
            // All minted ticket sequences start at one. Zero cannot publish.
            slot.sequence = 0; slot.evidence = None; slot.reason = Reason::StaleObservation;
        }
    }
    pub fn invalidate_provider(&mut self, provider: &str) {
        for (target, slot) in &mut self.slots {
            if target.provider == provider { slot.sequence = 0; slot.evidence = None; slot.reason = Reason::StaleObservation; }
        }
    }
    pub fn view(&self, target: &Target, now: u64) -> TargetDescriptor {
        let slot = self.slots.get(target);
        let observed = slot.and_then(|s| s.evidence.as_ref());
        let expires = observed.and_then(|e| e.at.checked_add(TTL_MS)).filter(|at| *at <= super::cache::MAX_DATE);
        let fresh = now > 0 && now <= super::cache::MAX_DATE && observed.is_some_and(|e| now >= e.at) && expires.is_some_and(|at| now < at);
        let evidence = observed.filter(|_| fresh);
        let reason = if observed.is_some() && !fresh || slot.is_some_and(|s| now < s.started || now.saturating_sub(s.started) > PUBLISH_MS) {
            Reason::StaleObservation
        } else { slot.map(|s| s.reason).unwrap_or(Reason::Unknown) };
        let mut capabilities = Vec::new();
        for id in ["session.history", "session.send", "session.interrupt", "session.approve", "session.continueBackground"] {
            let mut c = super::base(id);
            c.source = vec![Source::BundledManifest];
            c.support = Support::Unknown; c.reason = Some(reason);
            c.observed_at = observed.map(|e| e.at); c.expires_at = expires;
            if observed.is_some() { c.source.push(Source::RuntimeObservation); }
            if id != "session.history" { c.decision = Decision::NativeApproval; }
            if matches!(id, "session.approve" | "session.continueBackground") {
                c.reason = Some(Reason::NativeApprovalUnavailable);
            } else if id == "session.interrupt" && target.provider != "codex" {
                c.support = Support::Unsupported; c.availability = Availability::Unavailable;
                c.reason = Some(Reason::NativeControlUnavailable);
            } else if let Some(e) = evidence {
                let ready = match id {
                    "session.history" => e.history,
                    "session.send" => e.send && !e.busy && !e.conflict,
                    _ => e.cancel && e.busy && e.turn.is_some() && !e.conflict,
                };
                c.support = if ready || id != "session.send" { Support::Supported } else { Support::Unknown };
                c.availability = if ready { Availability::Ready } else { Availability::Unavailable };
                c.reason = if ready { None } else if e.conflict { Some(Reason::ControlConflict) }
                    else if id == "session.send" && e.busy { Some(Reason::Busy) }
                    else if id == "session.interrupt" && (!e.busy || e.turn.is_none()) { Some(Reason::TurnRequired) }
                    else { Some(Reason::NativeControlUnavailable) };
            }
            capabilities.push(c);
        }
        TargetDescriptor {
            schema_version: 1, target: target.clone(),
            revision: format!("{}:{}:{}", self.instance, slot.map(|s| s.sequence).unwrap_or(0), if fresh { "fresh" } else { "unknown" }),
            observed_at: observed.map(|e| e.at), expires_at: expires,
            busy: evidence.map(|e| e.busy), turn: evidence.and_then(|e| e.turn.clone()),
            model: evidence.and_then(|e| e.model.clone()), capabilities,
        }
    }
}

#[cfg(test)]
#[path = "../pet_capabilities_native_tests.rs"]
mod tests;
