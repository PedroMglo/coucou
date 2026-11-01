//! Native classification of the exact original step. No target IO or repair.
use super::*;
use crate::pet_runtime::{uncertain_effects::{Effect,MAX_EFFECTS},continuation::Reference};
impl Authority{
    #[cfg(test)]pub(in crate::pet_runtime)fn test_uncertain_effects(&self,goal:&Record)->Result<Vec<Effect>>{let _guard=self.lock_shared()?;self.capture_uncertain_effects(goal)}
    fn uncertain_effect(&self,goal:&Record,row:&crate::pet_runtime::admission::QueuedAdmission,attempt:&str)->Result<Effect>{
        let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let mut found=None;
        for plan in &goal.plans{for step in &plan.steps{
            let Some(record)=self.step_load(&row.spec,proof,&plan.id,&step.id)?else{continue;};
            if record.attempt.as_ref().is_none_or(|a|a.id!=attempt){continue;}
            if found.is_some()||!matches!(record.phase,crate::pet_runtime::steps::Phase::Started|crate::pet_runtime::steps::Phase::Finished){return Err(Failure::RecoveryRequired);}
            let operation=step.operation.clone().ok_or(Failure::RecoveryRequired)?;let scope=&record.scope;
            if operation.operation_id!=scope.operation_id||operation.version!=scope.version||model::fingerprint(&operation.arguments).ok().as_ref()!=Some(&scope.arguments_hash){return Err(Failure::RecoveryRequired);}
            let effect=Effect{decision_id:row.spec.decision_id.clone(),attempt_id:attempt.into(),grant_id:scope.grant_id.clone(),plan_id:plan.id.clone(),step_id:step.id.clone(),scope_hash:model::fingerprint(scope).map_err(|_|Failure::Invalid)?,operation,target_label:format!("{} · {}",scope.resource.identity,scope.resource.owner)};
            if !effect.valid(){return Err(Failure::RecoveryRequired);}found=Some(effect);
        }}
        found.ok_or(Failure::RecoveryRequired)
    }
    pub(in crate::pet_runtime::authority_store)fn capture_uncertain_effects(&self,goal:&Record)->Result<Vec<Effect>>{
        let mut effects=vec![];
        for row in &goal.admissions{
            let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            let evidence=self.reconciliation_evidence_unlocked(goal,row,&body)?;
            for attempt in evidence.attempts.iter().filter(|a|a.kind=="tool"&&!a.outcome_known){
                if effects.len()>=MAX_EFFECTS{return Err(Failure::RecoveryRequired);}
                effects.push(self.uncertain_effect(goal,row,&attempt.id)?);
            }
        }
        self.fence()?;Ok(effects)
    }
    /// Once signed, the manifest records uncertainty at review time. Later
    /// authentic observations may add knowledge without erasing that decision.
    pub(in crate::pet_runtime::authority_store)fn verify_uncertain_effects(&self,goal:&Record,reference:&Reference)->Result<()>{
        let Some(effects)=&reference.uncertain_effects else{return Ok(());};
        for effect in effects{
            let row=goal.admissions.iter().take(reference.previous.len()).find(|r|r.spec.decision_id==effect.decision_id).ok_or(Failure::Changed)?;
            let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            if body.budget_usage.as_ref().is_none_or(|b|!b.attempts.iter().any(|a|a.id==effect.attempt_id&&a.kind==crate::pet_runtime::effect_budget::Kind::Tool&&a.started_at.is_some())){return Err(Failure::Changed);}
            if self.uncertain_effect(goal,row,&effect.attempt_id)?!=*effect{return Err(Failure::Changed);}
        }
        self.fence()
    }
}
