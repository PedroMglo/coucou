//! A new consent binds the original closed goal and its reviewed history. No
//! old controller, lease, budget or recipe is reopened or reconstructed.
use super::*;
use crate::pet_runtime::{continuation::{self,Reference,Anchor,Unused},history::Charge,effect_budget::Kind,admission::Denial};
impl Authority{
    pub(in crate::pet_runtime::authority_store)fn never_claimed_zero(body:&Grant)->bool{
        body.never_claimed_zero()
    }
    fn continuation_charge(body:&Grant)->Result<Charge>{
        if let Some(ledger)=&body.budget_usage{
            let mut charge=Charge::default();
            for a in &ledger.attempts{charge.checked_add(&Charge{model_calls:u64::from(a.kind==Kind::Model),tool_calls:u64::from(a.kind==Kind::Tool),output_bytes:a.charge(),..Charge::default()})?;}
            Ok(charge)
        }else if body.lease.is_none()||body.runtime_owner.is_some(){Ok(Charge::default())}else{Err(Failure::RecoveryRequired)}
    }
    fn continuation_fence(&self,goal:&Record,bodies:&[Grant])->Result<String>{
        let mut fences=vec![];
        for(row,body)in goal.admissions.iter().zip(bodies){fences.push((&row.spec.decision_id,self.recovery_receipt_fence_for(goal,&row.spec,body)?));}
        model::fingerprint(&fences).map_err(|_|Failure::Invalid)
    }
    /// Authenticate the immutable prefix without synthesizing an old goal. A
    /// later receipt may add information, but cannot rewrite the charged grant.
    fn continuation_prefix(&self,goal:&Record,reference:&Reference)->Result<Vec<Grant>>{
        if !reference.basic()||goal.admissions.len()<reference.previous.len(){return Err(Failure::Invalid);}
        let mut bodies=vec![];let mut charge=Charge::default();
        for(row,anchor)in goal.admissions.iter().zip(&reference.previous){
            if row.spec.decision_id!=anchor.decision_id||model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?!=anchor.spec_hash||model::fingerprint(row).map_err(|_|Failure::Invalid)?!=anchor.row_hash||row.spec.budget!=anchor.budget||row.spec.resolved_model!=anchor.resolved_model{return Err(Failure::Changed);}
            let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            if !matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=anchor.grant_hash{return Err(Failure::Changed);}
            charge.checked_add(&Self::continuation_charge(&body)?)?;bodies.push(body);
        }
        if charge.model_calls!=reference.charge.model_calls||charge.tool_calls!=reference.charge.tool_calls||charge.output_bytes!=reference.charge.output_bytes{return Err(Failure::Changed);}
        let closure=bodies.last().and_then(|b|b.recovery.as_ref()).ok_or(Failure::RecoveryRequired)?;
        if closure.reference!=reference.closure||closure.target_goal_hash!=reference.source_goal_hash{return Err(Failure::Changed);}
        for unused in &reference.unused{
            let(body,_)=self.grant_file(&format!("{}.json",unused.decision_id))?;
            if body.goal_id!=goal.id||body.decision_id!=unused.decision_id||body.phase!=GrantPhase::Revoked||!Self::never_claimed_zero(&body)||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=unused.grant_hash{return Err(Failure::Changed);}
        }
        self.verify_uncertain_effects(goal,reference)?;Ok(bodies)
    }
    /// Read-only classification under the caller's original authority guard.
    /// Authenticate the later seal and exact immutable prefix, never trust a
    /// caller-supplied flag or construct an execution permit from these data.
    pub(in crate::pet_runtime::authority_store)fn accounting_frozen_unlocked(&self,goal:&Record,spec:&AdmissionSpec)->Result<bool>{
        goal.validate().map_err(|_|Failure::Invalid)?;
        if goal.id!=spec.goal_id||goal.spec.owner!=spec.owner||!goal.admissions.iter().any(|row|row.spec.decision_id==spec.decision_id&&model::fingerprint(&row.spec).ok()==model::fingerprint(spec).ok()){return Err(Failure::Changed);}
        let mut frozen=false;
        for row in &goal.admissions{if let Some(reference)=&row.spec.continuation{
            if !reference.previous.iter().any(|anchor|anchor.decision_id==spec.decision_id){continue;}
            self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            if !reference.present(goal,&row.spec){return Err(Failure::Changed);}self.continuation_prefix(goal,reference)?;frozen=true;
        }}self.fence()?;Ok(frozen)
    }
    pub(in crate::pet_runtime)fn verify_continuations_goal(&self,goal:&Record)->Result<()>{
        if !goal.admissions.iter().any(|row|row.spec.continuation.is_some()){return Ok(());}
        let _guard=self.lock_shared()?;
        for row in &goal.admissions{if let Some(reference)=&row.spec.continuation{
            self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            if !reference.present(goal,&row.spec){return Err(Failure::Changed);}self.continuation_prefix(goal,reference)?;
        }}self.fence()
    }
    /// This runs under the caller's SH or EX guard. The semantic snapshot was
    /// produced natively under SH; exact byte fences close the SH->EX gap.
    pub(in crate::pet_runtime::authority_store)fn continuation_current_unlocked(&self,goal:&Record,reference:&Reference,excluding:Option<&AdmissionSpec>)->Result<()>{
        if !reference.source_matches(goal){return Err(Failure::Changed);}
        let bodies=self.continuation_prefix(goal,reference)?;
        self.recovery_profile(bodies.last().ok_or(Failure::Missing)?)?;
        let(grants,pending)=self.history_inventory(&goal.id,excluding)?;
        if !pending.is_empty()||grants.len()!=reference.previous.len()+reference.unused.len(){return Err(Failure::Changed);}
        for anchor in &reference.previous{if grants.get(&anchor.decision_id).is_none_or(|b|model::fingerprint(b).ok().as_ref()!=Some(&anchor.grant_hash)){return Err(Failure::Changed);}}
        for unused in &reference.unused{if grants.get(&unused.decision_id).is_none_or(|b|model::fingerprint(b).ok().as_ref()!=Some(&unused.grant_hash)){return Err(Failure::Changed);}}
        if self.continuation_fence(goal,&bodies)?!=reference.evidence_fence{return Err(Failure::Changed);}self.fence()
    }
    pub(in crate::pet_runtime)fn continuation_current(&self,goal:&Record,reference:&Reference)->Result<()>{
        let _guard=self.lock_shared()?;self.continuation_current_unlocked(goal,reference,None)
    }
    pub(in crate::pet_runtime)fn capture_continuation(self,goal:&Record,now:u64,review_uncertain_effects:bool)->Result<std::result::Result<continuation::Captured,Denial>>{
        let reference={
            let _guard=self.lock_shared()?;
            if goal.state!=State::RecoveryRequired||goal.admissions.is_empty(){return Ok(Err(Denial::RecoveryNotClosed));}
            if goal.admissions.len()>=32{return Err(Failure::Capacity);}
            let row=goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            let Some(closure)=body.recovery.as_ref()else{return Ok(Err(Denial::RecoveryNotClosed));};
            if closure.reference.target_state!=State::RecoveryRequired||closure.reference.target_goal_revision!=goal.revision||closure.target_goal_hash!=model::fingerprint(goal).map_err(|_|Failure::Invalid)?||self.verify_recovered_goal(goal)?{return Ok(Err(Denial::RecoveryNotClosed));}
            self.recovery_profile(&body)?;
            let history=self.history(goal,now)?;
            if !history.scan_complete||!history.accounting_complete||!history.source_consistent{return Ok(Err(Denial::HistoryIncomplete));}
            let charge=history.cumulative_charge.ok_or(Failure::RecoveryRequired)?;
            let uncertain_effects=if charge.unknown_tool_results>0{
                if !review_uncertain_effects{return Ok(Err(Denial::UnreconciledEffects));}
                match self.capture_uncertain_effects(goal){
                    Ok(effects)if effects.len()as u64==charge.unknown_tool_results=>Some(effects),
                    Ok(_)|Err(Failure::RecoveryRequired)=>return Ok(Err(Denial::UnreconciledEffects)),Err(error)=>return Err(error),
                }
            }else{None};
            if history.unlinked_grants.len()>64{return Err(Failure::Capacity);}
            let mut previous=vec![];let mut bodies=vec![];
            for row in &goal.admissions{
                let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
                previous.push(Anchor{decision_id:row.spec.decision_id.clone(),spec_hash:model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?,row_hash:model::fingerprint(row).map_err(|_|Failure::Invalid)?,grant_hash:model::fingerprint(&body).map_err(|_|Failure::Invalid)?,budget:row.spec.budget.clone(),resolved_model:row.spec.resolved_model.clone()});bodies.push(body);
            }
            let reference=Reference{schema_version:if uncertain_effects.is_some(){2}else{1},uncertain_effects,source_revision:goal.revision.clone(),source_sequence:goal.sequence,source_goal_hash:history.goal_hash,closure:closure.reference.clone(),history_hash:history.source_hash,evidence_fence:self.continuation_fence(goal,&bodies)?,previous,unused:history.unlinked_grants.into_iter().map(|g|Unused{decision_id:g.decision_id,grant_hash:g.hash}).collect(),charge,history_continuity_verified:false,additional_allowance:true,reviewed_at:now};
            self.continuation_current_unlocked(goal,&reference,None)?;reference
        };
        Ok(Ok(continuation::Captured::native(reference,self)))
    }
    /// Only temporary Issued images of this retained native consent, with no
    /// claim/accounting, are retired. No historical/canonical grant is removed.
    pub(in crate::pet_runtime::authority_store)fn retire_admission_staging(&self,spec:&AdmissionSpec)->Result<()>{
        let hash=model::fingerprint(spec).map_err(|_|Failure::Invalid)?;
        for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
            if self.historical_file(&name){continue;}
            if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}
            let(body,stamp)=self.grant_file(&name)?;
            if body.decision_id==spec.decision_id&&body.goal_id==spec.goal_id&&body.spec_hash==hash&&body.approved_at==spec.approved_at&&body.phase==GrantPhase::Issued&&Self::never_claimed_zero(&body){self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}
        }
        self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
}
