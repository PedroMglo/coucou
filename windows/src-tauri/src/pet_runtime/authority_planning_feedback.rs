//! Only authenticated immutable native recipe evidence can finalize a planning
//! round. No model prose, new timestamp or parsed feedback DTO can do so.
use super::*;
use crate::pet_runtime::{generated_plans::Phase as PublicationPhase,leases::Lease,planning_feedback::Feedback,recipes::Scope};
impl Authority {
    /// Authenticated data only. This snapshot neither creates a driver nor
    /// permits inference, registration or physical effects.
    pub(in crate::pet_runtime)fn planning_ledger(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,now:u64)->Result<crate::pet_runtime::planning_journal::Ledger> {
        let _lock=self.lock_shared()?;let body=self.budget_body(spec,proof,lease,now,false)?;
        let journal=body.planning_usage.ok_or(Failure::Invalid)?;
        for round in &journal.rounds {if let Some(feedback)=&round.feedback {self.planning_feedback_unlocked(spec,proof,feedback)?;}}
        Ok(journal)
    }
    /// Native capture supplies observation only. Original plan ACK and every
    /// delivery must already exist; no feedback can grant DATA use or success.
    pub(in crate::pet_runtime)fn record_peer_round_feedback(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,goal:&Record,data:&crate::pet_runtime::peer_round_data::Captured,now:u64)->Result<crate::pet_runtime::peer_round_feedback::Feedback>{
        if !spec.peer_iteration(){return Err(Failure::Invalid);}let reference=crate::pet_runtime::planned_peer::reference(goal,spec).map_err(|_|Failure::Changed)?;
        let feedback=data.feedback(goal,reference,now).map_err(|_|Failure::Changed)?;
        let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;if !matches!(body.phase,GrantPhase::Claimed|GrantPhase::Waiting)||now>=lease.deadline_at{return Err(Failure::Revoked);}Self::generated_acknowledged(&body,spec,reference)?;
        let plan=goal.plans.iter().find(|p|p.id==reference.plan_id).ok_or(Failure::Invalid)?;
        if !plan.steps.iter().all(|step|body.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.scope.envelope.cause.plan_id==plan.id&&r.scope.envelope.cause.step_id==step.id))){return Err(Failure::RecoveryRequired);}
        let journal=body.planning_usage.as_mut().ok_or(Failure::Invalid)?;let round=journal.rounds.last_mut().filter(|r|r.attempt_id==reference.output.attempt_id).ok_or(Failure::Changed)?;
        if let Some(old)=&round.peer_feedback{if old!=&feedback{return Err(Failure::Changed);}return self.stabilize(&body).map(|_|feedback);}
        round.peer_feedback=Some(feedback.clone());journal.schema_version=journal.schema_version.max(3);
        if !journal.valid_for(spec,body.lease.as_ref(),body.budget_usage.as_ref()){return Err(Failure::Invalid);}
        body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;Ok(feedback)
    }
    /// Caller holds the original authority lock. Revalidate the referenced
    /// native files; a copied progress counter is not a substitute for evidence.
    pub(super)fn planning_feedback_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,feedback:&Feedback)->Result<crate::pet_runtime::criteria::Verification> {
        let scope=self.recipe_load(spec,proof,&feedback.plan_id)?.ok_or(Failure::Missing)?;
        if scope.schema_version!=3||scope.id!=feedback.recipe_id{return Err(Failure::Invalid);}
        let report=self.verification_load_unlocked(spec,proof,&scope)?.ok_or(Failure::RecoveryRequired)?;
        if Feedback::from_report(&report)!=*feedback{return Err(Failure::Changed);}Ok(report)
    }
    pub(in crate::pet_runtime)fn planning_feedback_report(&self,spec:&AdmissionSpec,proof:&ConsentProof,feedback:&Feedback)->Result<crate::pet_runtime::criteria::Verification> {
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if !body.planning_usage.as_ref().is_some_and(|j|j.rounds.iter().any(|r|r.feedback.as_ref()==Some(feedback))){return Err(Failure::Missing);}
        self.planning_feedback_unlocked(spec,proof,feedback)
    }
    pub(in crate::pet_runtime)fn record_planning_feedback(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,now:u64)->Result<Feedback> {
        if scope.schema_version!=3||!scope.belongs(spec,lease){return Err(Failure::Invalid);}
        let reference=scope.generated_plan.as_ref().ok_or(Failure::Invalid)?;
        let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
        Self::generated_acknowledged(&body,spec,reference)?;
        let report=self.verification_load_unlocked(spec,proof,scope)?.ok_or(Failure::RecoveryRequired)?;
        // Stabilize the ORIGINAL verification before sealing a reference to it.
        // No target read or physical operation occurs on this path or a retry.
        self.verification_stabilize(scope,&report)?;
        let feedback=Feedback::from_report(&report);
        if !feedback.valid()||feedback.goal_spec_hash!=spec.intent_hash||feedback.recorded_at>now{return Err(Failure::Invalid);}
        let round=body.planning_usage.as_mut().and_then(|j|j.rounds.iter_mut().find(|r|r.attempt_id==reference.output.attempt_id)).ok_or(Failure::Missing)?;
        if round.publication.as_ref().is_none_or(|p|p.phase!=PublicationPhase::Acknowledged||p.reference!=*reference){return Err(Failure::Changed);}
        if let Some(old)=&round.feedback {
            if old!=&feedback{return Err(Failure::Changed);}self.stabilize(&body)?;return Ok(feedback);
        }
        round.feedback=Some(feedback.clone());
        if !body.planning_usage.as_ref().is_some_and(|j|j.valid_for(spec,body.lease.as_ref(),body.budget_usage.as_ref())){return Err(Failure::Invalid);}
        body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;Ok(feedback)
    }
}
