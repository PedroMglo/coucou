//! Immutable observations of completed model calls. These values never grant
//! execution authority or establish that a goal's criteria have been met.
use super::{admission::AdmissionSpec, authority::{Authority, ConsentProof, Failure}, effect_budget::Phase, model, store::{Dir, nonce, protocol_directory}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PROTOCOL: &[u8] = b"{\"version\":1,\"kind\":\"coucou-model-results\"}\n";
const MAX_TEXT: usize = 4 * 1024 * 1024;
const MAX_RECEIPT: usize = MAX_TEXT * 6 + 16384;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ModelResult {
    pub schema_version: u32,
    pub goal_id: String,
    pub pet_id: String,
    pub provider: String,
    pub decision_id: String,
    pub lease_id: String,
    pub request_id: String,
    pub attempt_id: String,
    pub model: String,
    pub input_hash: String,
    pub intent_hash: String,
    pub text: String,
    pub accepted_output_bytes: u64,
    pub completed_at: u64,
    pub recorded_at: u64,
}
impl ModelResult {
    pub(super) fn valid(&self) -> bool {
        let bounded = |s: &str, limit: usize| !s.trim().is_empty() && s.len() <= limit && !s.chars().any(char::is_control);
        self.schema_version == 1 && model::goal_id(&self.goal_id)
            && model::token(&self.decision_id) && model::token(&self.lease_id)
            && model::token(&self.attempt_id) && model::key(&self.request_id)
            && bounded(&self.pet_id, 96) && bounded(&self.provider, 96)
            && bounded(&self.model, 256) && !self.text.is_empty()
            && self.text.len() <= MAX_TEXT && self.text.len() as u64 <= self.accepted_output_bytes
            && self.accepted_output_bytes <= MAX_TEXT as u64
            && super::authority::decode(&self.input_hash).is_some()
            && super::authority::decode(&self.intent_hash).is_some()
            && self.completed_at > 0 && self.recorded_at >= self.completed_at
            && self.recorded_at <= 8_640_000_000_000_000
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(super) struct Envelope { pub body: ModelResult, pub tag: String }
pub(super) fn message(body: &ModelResult) -> super::authority::Result<Vec<u8>> {
    let mut bytes = b"coucou-native-model-result-v1\0".to_vec();
    bytes.extend(serde_json::to_vec(body).map_err(|_| Failure::Invalid)?);
    Ok(bytes)
}

#[derive(Debug, Serialize)]
#[serde(tag="status", rename_all="camelCase")]
pub enum Lookup {
    NotStarted,
    RecoveryRequired { phase: RecoveryPhase },
    Available { receipt: ModelResult },
}
#[derive(Debug, Serialize)]
#[serde(rename_all="camelCase")]
pub enum RecoveryPhase { Reserved, Started, Uncertain, SettledReceiptMissing }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct View {
    #[serde(flatten)] pub result: Lookup,
    pub criteria_verified: bool,
    pub goal_complete: bool,
}
impl From<Lookup> for View {
    fn from(result: Lookup) -> Self { Self { result, criteria_verified: false, goal_complete: false } }
}

fn directory(path: PathBuf, decision: &str, create: bool) -> super::authority::Result<Option<Dir>> {
    if !model::token(decision) { return Err(Failure::Invalid); }
    let Some(root) = protocol_directory(&path, PROTOCOL, create).map_err(|_| Failure::Unsafe)? else { return Ok(None); };
    root.check().map_err(|_| Failure::Changed)?;
    protocol_directory(&root.path.join(decision), PROTOCOL, create).map_err(|_| Failure::Unsafe)
}
pub(super) fn lookup(authority: &Authority, spec: &AdmissionSpec, proof: &ConsentProof, request: &str) -> super::authority::Result<Lookup> {
    let (_, attempt) = authority.result_attempt(spec, proof, request)?;
    let Some(attempt) = attempt else { return Ok(Lookup::NotStarted); };
    if attempt.phase != Phase::Settled {
        return Ok(Lookup::RecoveryRequired { phase: match attempt.phase {
            Phase::Reserved => RecoveryPhase::Reserved,
            Phase::Started => RecoveryPhase::Started,
            Phase::Uncertain => RecoveryPhase::Uncertain,
            Phase::Settled => unreachable!(),
        }});
    }
    let Some(dir) = directory(authority.result_root()?, &spec.decision_id, false)? else {
        return Ok(Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing });
    };
    let Some((raw, _)) = dir.bytes(&format!("{}.json", attempt.id), MAX_RECEIPT).map_err(|_| Failure::Unsafe)? else {
        return Ok(Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing });
    };
    let envelope: Envelope = serde_json::from_slice(&raw).map_err(|_| Failure::Invalid)?;
    if envelope.body.attempt_id != attempt.id || envelope.body.request_id != request { return Err(Failure::Invalid); }
    authority.verify_result(spec, proof, &envelope)?;
    dir.check().map_err(|_| Failure::Changed)?;
    Ok(Lookup::Available { receipt: envelope.body })
}

pub(super) fn publish(authority: &Authority, spec: &AdmissionSpec, proof: &ConsentProof, envelope: &Envelope) -> super::authority::Result<()> {
    authority.verify_result(spec, proof, envelope)?;
    let bytes = serde_json::to_vec(envelope).map_err(|_| Failure::Invalid)?;
    if bytes.len() > MAX_RECEIPT { return Err(Failure::Capacity); }
    let dir = directory(authority.result_root()?, &spec.decision_id, true)?.ok_or(Failure::Unavailable)?;
    let name = format!("{}.json", envelope.body.attempt_id);
    if let Some((old, _)) = dir.bytes(&name, MAX_RECEIPT).map_err(|_| Failure::Unsafe)? {
        if old != bytes { return Err(Failure::RecoveryRequired); }
    } else {
        let temp = format!(".result-{}", nonce().map_err(|_| Failure::Unavailable)?);
        dir.write_new(&temp, &bytes).map_err(|_| Failure::Unavailable)?;
        #[cfg(test)] if matches!(authority.fault, super::authority_store::Fault::BeforeResultPublish) { return Err(Failure::Unavailable); }
        authority.verify_result(spec, proof, envelope)?;
        dir.rename(&temp, &dir, &name, true).map_err(|_| Failure::Uncertain)?;
        #[cfg(test)] if matches!(authority.fault, super::authority_store::Fault::AfterResultPublish) { return Err(Failure::Uncertain); }
    }
    dir.open_file(&name, libc::O_RDONLY).map_err(|_| Failure::Unsafe)?.ok_or(Failure::Missing)?
        .sync_all().map_err(|_| Failure::Uncertain)?;
    dir.durable().map_err(|_| Failure::Uncertain)?;
    // Verify the canonical bytes after publication, never a staged candidate.
    let (raw, _) = dir.bytes(&name, MAX_RECEIPT).map_err(|_| Failure::Unsafe)?.ok_or(Failure::Missing)?;
    if raw != bytes { return Err(Failure::Changed); }
    authority.verify_result(spec, proof, envelope)
}

/// Stabilize an already authenticated receipt before acknowledging a retried
/// execution request. Read-only lookups need not acknowledge durability.
pub(super) fn recover(authority: &Authority, spec: &AdmissionSpec, proof: &ConsentProof, request: &str) -> super::authority::Result<ModelResult> {
    let (_, attempt) = authority.result_attempt(spec, proof, request)?;
    let attempt = attempt.filter(|a| a.phase == Phase::Settled).ok_or(Failure::RecoveryRequired)?;
    let dir = directory(authority.result_root()?, &spec.decision_id, false)?.ok_or(Failure::RecoveryRequired)?;
    let (raw, _) = dir.bytes(&format!("{}.json", attempt.id), MAX_RECEIPT).map_err(|_| Failure::Unsafe)?.ok_or(Failure::RecoveryRequired)?;
    let envelope: Envelope = serde_json::from_slice(&raw).map_err(|_| Failure::Invalid)?;
    if envelope.body.attempt_id != attempt.id || envelope.body.request_id != request { return Err(Failure::Invalid); }
    publish(authority, spec, proof, &envelope)?;
    Ok(envelope.body)
}
