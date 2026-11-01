//! The existing signed grant carries the old closure receipt. No independent
//! task journal, execution grant or reconstructed original owner is created.
use super::*;
use crate::pet_runtime::{recovery::Closure,recovery_controller::{self,Approved},native_owner::AbsentLease,reconciliation::Snapshot,results,planning_output};
impl Authority{
    pub(in crate::pet_runtime)fn recovery_body(&self,goal:&Record)->Result<Grant>{
        let _guard=self.lock_shared()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)
    }
    pub(in crate::pet_runtime)fn recovery_seed(&self,goal:&Record,now:u64)->Result<(Grant,Option<AbsentLease>,Snapshot,String)>{
        let _guard=self.lock_shared()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let body=self.load(&row.spec,proof)?;
        let gate=if body.recovery.is_some(){self.recovery_profile(&body)?;None}else{Some(self.retained_recovery_absence(&row.spec,proof)?)};let snapshot=self.reconciliation_with_gate(goal,now,gate.as_ref())?;let fence=self.recovery_receipt_fence(goal,&body)?;
        Ok((body,gate,snapshot,fence))
    }
    pub(in crate::pet_runtime::authority_store)fn recovery_profile(&self,body:&Grant)->Result<()>{
        crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;let(_,profile)=self.owner_context()?;
        if body.runtime_owner.as_ref().is_none_or(|o|o.profile!=profile){return Err(Failure::Changed);}Ok(())
    }
    fn recovery_receipt_fence(&self,goal:&Record,body:&Grant)->Result<String>{
        let spec=&goal.admissions.last().ok_or(Failure::Missing)?.spec;self.recovery_receipt_fence_for(goal,spec,body)
    }
    pub(in crate::pet_runtime::authority_store)fn recovery_receipt_fence_for(&self,goal:&Record,spec:&AdmissionSpec,body:&Grant)->Result<String>{
        let attempts=body.budget_usage.as_ref().map(|b|b.attempts.iter().filter(|a|a.kind==crate::pet_runtime::effect_budget::Kind::Model).map(|a|a.id.clone()).collect::<Vec<_>>()).unwrap_or_default();
        let models=results::receipt_fence(self,spec,&attempts)?;let planning=planning_output::receipt_fence(self,spec,&attempts)?;let dir=self.step_directory(false)?;let mut names=std::collections::BTreeSet::new();
        for plan in &goal.plans{
            let slot=crate::pet_runtime::recipes::slot(&spec.decision_id,&plan.id).map_err(|_|Failure::Invalid)?;names.insert(format!("{slot}.recipe.json"));names.insert(format!("{slot}.verification.json"));
            for step in &plan.steps{let slot=crate::pet_runtime::steps::slot(&spec.decision_id,&plan.id,&step.id).map_err(|_|Failure::Invalid)?;names.insert(format!("{slot}.json"));names.insert(format!("{slot}.evidence.json"));}
        }
        let mut steps=vec![];for name in names{let hash=match &dir{Some(d)=>d.bytes(&name,256*1024).map_err(|_|Failure::Unsafe)?.map(|(b,_)|model::hash(&b)),None=>None};steps.push((name,hash));}
        self.fence()?;model::fingerprint(&(models,planning,steps)).map_err(|_|Failure::Invalid)
    }
    pub(in crate::pet_runtime)fn close_lost_owner(&self,approved:&Approved,started:&mut bool)->Result<()> {
        let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;
        let row=approved.goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
        if body.recovery.as_ref()==Some(&approved.closure){self.recovery_profile(&body)?;self.stabilize(&body)?;return Ok(());}
        let gate=approved.gate.as_ref().ok_or(Failure::Changed)?;gate.current(self).map_err(|_|Failure::Changed)?;
        if body.recovery.is_some()||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=approved.closure.reference.source_grant_hash
            ||!gate.matches(&approved.closure.reference.source_grant_hash,&approved.closure.reference.source_grant_revision)
            ||self.recovery_receipt_fence(&approved.goal,&body)?!=approved.fence{return Err(Failure::Changed);}
        let mut closed=recovery_controller::closed_base(&body,approved.closure.reference.approved_at);
        if model::fingerprint(&closed).map_err(|_|Failure::Invalid)?!=approved.closure.closed_base_hash{return Err(Failure::Changed);}
        closed.recovery=Some(approved.closure.clone());closed.schema_version=7;closed.revision=approved.closure.reference.id.clone();
        gate.current(self).map_err(|_|Failure::Changed)?;*started=true;self.publish(closed,false)
    }
    pub(in crate::pet_runtime)fn acknowledge_recovery(&self,goal:&Record,reference:&crate::pet_runtime::recovery::Reference)->Result<()> {
        let _guard=self.lock()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==reference.decision_id).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
        self.recovery_profile(&body)?;if body.recovery.as_ref().is_none_or(|c|c.reference!=*reference)||!Self::recovery_event_present(goal,body.recovery.as_ref().unwrap())?{return Err(Failure::Changed);}self.stabilize(&body)
    }
    pub(in crate::pet_runtime)fn recovery_uncommitted(&self,approved:&Approved)->Result<bool>{
        let _guard=self.lock()?;let row=approved.goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
        if body.recovery.is_some(){return Ok(false);}let gate=approved.gate.as_ref().ok_or(Failure::Changed)?;gate.current(self).map_err(|_|Failure::Changed)?;
        if body.lease.as_ref().is_none_or(|l|l.id!=approved.closure.reference.lease_id)||model::fingerprint(body.runtime_owner.as_ref().ok_or(Failure::Missing)?).map_err(|_|Failure::Invalid)?!=approved.closure.owner_hash{return Err(Failure::Changed);}Ok(true)
    }
    fn recovery_event_present(goal:&Record,closure:&Closure)->Result<bool>{
        let r=&closure.reference;
        if r.target_state==State::Completed{return Ok(goal.state==State::Completed&&goal.revision==r.target_goal_revision&&model::fingerprint(goal).map_err(|_|Failure::Invalid)?==closure.target_goal_hash);}
        let Some(event)=goal.events.get((r.target_sequence-1)as usize)else{return Ok(false);};
        let kind=if r.unclaimed{matches!(event.kind,model::EventKind::UnclaimedOwnerClosed)}else{matches!(event.kind,model::EventKind::LostOwnerClosed)};
        let linked=kind&&event.request_id==r.request_id&&event.at==r.approved_at&&event.fingerprint==model::fingerprint(r).map_err(|_|Failure::Invalid)?;
        if linked&&goal.sequence==r.target_sequence&&(goal.revision!=r.target_goal_revision||model::fingerprint(goal).map_err(|_|Failure::Invalid)?!=closure.target_goal_hash){return Err(Failure::Changed);}Ok(linked)
    }
    /// Native closure history cannot be deleted or promoted by editing goal
    /// JSON. The exact old source may remain while an approved ACK is pending.
    pub(in crate::pet_runtime)fn verify_recovered_goal(&self,goal:&Record)->Result<bool>{
        let _guard=self.lock_shared()?;let mut references=vec![];let mut pending=false;
        for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
            if self.historical_file(&name){continue;}
            if !name.strip_suffix(".json").is_some_and(model::token){continue;}
            let(raw,_)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;let envelope:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
            if !envelope.body.valid(){return Err(Failure::Invalid);}hmac::verify(&self.key,&grant_message(&envelope.body)?,&decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
            if envelope.body.goal_id!=goal.id{continue;}
            if let Some(retirement)=&envelope.body.retirement{
                self.retirement_profile(retirement)?;
                if retirement.pet_id!=goal.spec.owner.pet_id||goal.admissions.iter().any(|r|r.spec.decision_id==retirement.decision_id)||goal.sequence<retirement.source_sequence
                    ||goal.sequence==retirement.source_sequence&&(goal.revision!=retirement.source_goal_revision||model::fingerprint(goal).ok().as_ref()!=Some(&retirement.source_goal_hash)){return Err(Failure::Changed);}
            }
            if envelope.body.recovery.is_none(){continue;}
            let row=goal.admissions.iter().find(|r|r.spec.decision_id==envelope.body.decision_id).ok_or(Failure::Invalid)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;let closure=body.recovery.as_ref().ok_or(Failure::Missing)?;
            if !Self::recovery_event_present(goal,closure)?{if !closure.reference.source_matches(goal){return Err(Failure::Changed);}pending=true;}references.push(closure.reference.clone());
        }
        for e in goal.events.iter().filter(|e|matches!(e.kind,model::EventKind::LostOwnerClosed|model::EventKind::UnclaimedOwnerClosed)){
            if !references.iter().any(|r|r.target_sequence==e.sequence&&r.request_id==e.request_id&&r.approved_at==e.at&&model::fingerprint(r).ok().as_ref()==Some(&e.fingerprint)){return Err(Failure::Invalid);}
        }
        self.fence()?;Ok(pending)
    }
}
