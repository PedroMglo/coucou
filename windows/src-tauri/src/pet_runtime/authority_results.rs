//! Read-only accounting inspection and signing of observations. No live lease
//! can be created or renewed through these methods.
use super::*;
use crate::pet_runtime::{effect_budget::{Attempt, Kind, Phase}, leases::Lease, results::{Envelope, ModelResult, message}};
use std::path::PathBuf;
impl Authority {
    pub(in crate::pet_runtime) fn result_root(&self) -> Result<PathBuf> {
        self.fence()?;
        Ok(self.dir.path.parent().ok_or(Failure::Unsafe)?.join("pet-runtime-results-v1"))
    }
    pub(in crate::pet_runtime) fn result_attempt(&self, spec: &AdmissionSpec, proof: &ConsentProof, request: &str) -> Result<(Option<Lease>, Option<Attempt>)> {
        if !model::key(request) { return Err(Failure::Invalid); }
        let _lock = self.lock()?;
        let body = self.load(spec, proof)?;
        let attempt = body.budget_usage.as_ref().and_then(|l| l.attempts.iter().find(|a| a.request_id == request)).cloned();
        if attempt.as_ref().is_some_and(|a| a.kind != Kind::Model) { return Err(Failure::Invalid); }
        self.fence()?;
        Ok((body.lease, attempt))
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::pet_runtime) fn seal_result(&self, spec: &AdmissionSpec, proof: &ConsentProof, lease: &Lease, attempt: &Attempt, text: &str, accepted: u64, now: u64) -> Result<Envelope> {
        let (stored_lease, stored) = self.result_attempt(spec, proof, &attempt.request_id)?;
        let stored = stored.ok_or(Failure::Missing)?;
        if stored_lease.as_ref() != Some(lease) || !stored.same_request(attempt)
            || stored.phase != Phase::Settled || stored.accepted_output != Some(accepted) { return Err(Failure::RecoveryRequired); }
        let body = ModelResult {
            schema_version: 1, goal_id: spec.goal_id.clone(), pet_id: spec.owner.pet_id.clone(),
            provider: spec.owner.provider.clone(), decision_id: spec.decision_id.clone(),
            lease_id: lease.id.clone(), request_id: stored.request_id, attempt_id: stored.id,
            model: lease.model.clone(), input_hash: stored.input_hash, intent_hash: spec.intent_hash.clone(),
            text: text.into(), accepted_output_bytes: accepted,
            completed_at: stored.finished_at.ok_or(Failure::Invalid)?, recorded_at: now,
        };
        if !body.valid() { return Err(Failure::Invalid); }
        let tag = encode(hmac::sign(&self.key, &message(&body)?).as_ref());
        let envelope = Envelope { body, tag };
        self.verify_result(spec, proof, &envelope)?;
        Ok(envelope)
    }
    pub(in crate::pet_runtime) fn verify_result(&self, spec: &AdmissionSpec, proof: &ConsentProof, envelope: &Envelope) -> Result<()> {
        let r = &envelope.body;
        if !r.valid() { return Err(Failure::Invalid); }
        hmac::verify(&self.key, &message(r)?, &decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_| Failure::Invalid)?;
        let (lease, attempt) = self.result_attempt(spec, proof, &r.request_id)?;
        let lease = lease.ok_or(Failure::Missing)?;
        let a = attempt.ok_or(Failure::Missing)?;
        if r.goal_id != spec.goal_id || r.pet_id != spec.owner.pet_id || r.provider != spec.owner.provider
            || r.decision_id != spec.decision_id || r.intent_hash != spec.intent_hash
            || r.lease_id != lease.id || r.model != lease.model || r.attempt_id != a.id
            || r.input_hash != a.input_hash || a.phase != Phase::Settled
            || a.accepted_output != Some(r.accepted_output_bytes) || a.finished_at != Some(r.completed_at) { return Err(Failure::Invalid); }
        self.fence()
    }
}
