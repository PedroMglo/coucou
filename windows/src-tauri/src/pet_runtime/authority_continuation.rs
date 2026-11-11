//! A new consent binds the original closed goal and its reviewed history. No
//! old controller, lease, budget or recipe is reopened or reconstructed.
use super::*;
use crate::pet_runtime::{continuation::{self,Reference,Anchor,Unused},history::Charge,effect_budget::Kind,admission::Denial};
impl Authority{
    pub(in crate::pet_runtime)fn peer_continuation_snapshot(&self,goal:&Record,now:u64)->Result<continuation::PeerPlanningSnapshot>{
        use crate::pet_runtime::{continuation::PeerPlanningOutcome as Outcome,generated_plans::Phase,planning_journal::OutputKind,native_verification::ReceiptRef};
        let _guard=self.lock_shared()?;
        goal.validate().map_err(|_|Failure::Invalid)?;
        let row=goal.admissions.last().ok_or(Failure::Missing)?;
        if goal.state!=State::RecoveryRequired||!row.spec.peer_iteration()||now<goal.updated_at||now>=row.spec.budget.deadline_at{return Err(Failure::Changed);}
        let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
        // A native absence closure preserves accounting, not the incoming
        // delegation's authority. Observe its exact binding under this guard.
        self.funding_current_unlocked(&row.spec,now)?;
        let closure=body.recovery.as_ref().ok_or(Failure::RecoveryRequired)?;
        if body.phase!=GrantPhase::Closed||body.stop_reason.is_some()
            ||closure.reference.target_state!=State::RecoveryRequired||closure.reference.target_goal_revision!=goal.revision
            ||closure.target_goal_hash!=model::fingerprint(goal).map_err(|_|Failure::Invalid)?
            ||!Self::recovery_event_present(goal,closure)?||self.verify_recovered_goal(goal)?{return Err(Failure::Changed);}
        self.recovery_profile(&body)?;
        let history=self.history(goal,now)?;
        if !history.scan_complete||!history.accounting_complete||!history.source_consistent{return Err(Failure::RecoveryRequired);}
        let mut receipts=vec![];let mut causes=std::collections::BTreeSet::new();
        for prior in &goal.admissions{
            let grant=self.load(&prior.spec,prior.proof.as_ref().ok_or(Failure::Missing)?)?;
            if !matches!(grant.phase,GrantPhase::Closed|GrantPhase::Revoked){return Err(Failure::RecoveryRequired);}
            for r in grant.messages.as_ref().map(|m|m.entries.as_slice()).unwrap_or(&[]){
                let reference=ReceiptRef::historical(r).map_err(|_|Failure::Invalid)?;
                if !causes.insert((reference.plan_id.clone(),reference.step_id.clone())){return Err(Failure::Changed);}
                receipts.push(reference);
            }
        }
        let journal=body.planning_usage.as_ref().ok_or(Failure::Missing)?;
        let last=journal.rounds.last().ok_or(Failure::Missing)?;
        let mut pending_steps=vec![];
        let outcome=match last.output.as_ref().map(|o|o.kind){
            None=>Outcome::UncertainModelResult,
            Some(OutputKind::Blocked|OutputKind::Invalid)=>Outcome::NoExecutablePlan,
            Some(OutputKind::Plan)=>match &last.publication{
                Some(p)if p.phase==Phase::Acknowledged&&goal.generated_plans.contains(&p.reference)=>{
                    let plan=goal.plans.iter().find(|p2|p2.id==p.reference.plan_id).ok_or(Failure::Changed)?;
                    for step in &plan.steps{if !causes.contains(&(plan.id.clone(),step.id.clone())){
                        pending_steps.push(crate::pet_runtime::messages::Cause{task_id:plan.task_id.clone(),plan_id:plan.id.clone(),step_id:step.id.clone()});
                    }}
                    Outcome::CanonicalPlan
                },_=>Outcome::UnpublishedPlan,
            },
        };
        let snapshot=continuation::PeerPlanningSnapshot{
            goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),goal_revision:goal.revision.clone(),goal_hash:history.goal_hash,
            decision_id:row.spec.decision_id.clone(),lease_id:body.lease.as_ref().ok_or(Failure::Missing)?.id.clone(),closure_id:closure.reference.id.clone(),
            spec_hash:model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?,grant_hash:model::fingerprint(&body).map_err(|_|Failure::Invalid)?,
            history_hash:history.source_hash,receipt_fence:self.recovery_receipt_fence_for(goal,&row.spec,&body)?,
            original_budget:row.spec.budget.clone(),original_policy:row.spec.planning_policy.clone().ok_or(Failure::Missing)?,
            model:row.spec.resolved_model.clone(),last_reviewed_destination:journal.rounds.iter().rev().find_map(|r|r.peer_data.as_ref()).map(|d|d.source.destination.clone()),
            remaining:body.budget_remaining(&row.spec.budget).ok_or(Failure::Invalid)?,delegated_held:body.delegated_held().ok_or(Failure::Invalid)?,
            charge:history.cumulative_charge.ok_or(Failure::RecoveryRequired)?,journal_hash:model::fingerprint(journal).map_err(|_|Failure::Invalid)?,rounds_observed:journal.rounds.len(),
            outcome,receipts,pending_steps,observed_at:now,execution_available:false,model_use_authorized:false,additional_allowance:false,
        };
        self.fence()?;Ok(snapshot)
    }
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
    pub(in crate::pet_runtime::authority_store)fn continuation_prefix(&self,goal:&Record,reference:&Reference)->Result<Vec<Grant>>{
        if !reference.basic()||goal.admissions.len()<reference.previous.len(){return Err(Failure::Invalid);}
        let mut bodies=vec![];let mut charge=Charge::default();
        for(row,anchor)in goal.admissions.iter().zip(&reference.previous){
            if row.spec.decision_id!=anchor.decision_id||model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?!=anchor.spec_hash||model::fingerprint(row).map_err(|_|Failure::Invalid)?!=anchor.row_hash||!anchor.matches_class(&row.spec)||row.spec.budget!=anchor.budget||row.spec.resolved_model!=anchor.resolved_model{return Err(Failure::Changed);}
            let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            if !matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=anchor.grant_hash{return Err(Failure::Changed);}
            charge.checked_add(&Self::continuation_charge(&body)?)?;bodies.push(body);
        }
        if charge.model_calls!=reference.charge.model_calls||charge.tool_calls!=reference.charge.tool_calls||charge.output_bytes!=reference.charge.output_bytes{return Err(Failure::Changed);}
        let last=bodies.last().ok_or(Failure::Missing)?;
        if let Some(f)=&reference.funded{
            let row=goal.admissions.get(reference.previous.len()-1).ok_or(Failure::Missing)?;
            let event=goal.events.get((reference.source_sequence-1)as usize).ok_or(Failure::Missing)?;
            if !row.spec.funded_response()||(reference.schema_version==9)!=(row.spec.schema_version==19)||last.phase!=GrantPhase::Closed||last.stop_reason!=Some(crate::pet_runtime::leases::StopReason::Pause)||last.closed_at!=Some(f.closed_at)||row.lease!=last.lease||f.goal_id!=goal.id||f.pet_id!=goal.spec.owner.pet_id||f.decision_id!=last.decision_id||!matches!(row.phase,crate::pet_runtime::admission::QueuePhase::Paused)||event.at!=f.closed_at||!matches!(event.kind,model::EventKind::LeaseDrained)||model::fingerprint(&(last.lease.as_ref().ok_or(Failure::Missing)?,true,last.stop_reason)).ok().as_ref()!=Some(&event.fingerprint){return Err(Failure::Changed);}
            let funding=last.funded_by.as_ref().ok_or(Failure::Changed)?;
            if funding.previous!=reference.previous[..reference.previous.len()-1]{return Err(Failure::Changed);}
            self.funding_prefix_unlocked(funding,&goal.id)?;
        }else{match(&reference.closure,&reference.drain,&reference.plan_revision){
            (Some(c),None,None)=>{let closure=last.recovery.as_ref().ok_or(Failure::RecoveryRequired)?;if closure.reference!=*c||closure.target_goal_hash!=reference.source_goal_hash{return Err(Failure::Changed);}},
            (None,Some(d),None)=>{let seal=last.native_drain.as_ref().ok_or(Failure::RecoveryRequired)?;if seal.reference!=*d||!d.continuable()||last.phase!=GrantPhase::Closed||seal.target_goal_hash!=reference.source_goal_hash{return Err(Failure::Changed);}},
            (None,None,Some(r))=>{if matches!(reference.schema_version,7|10){let row=goal.admissions.get(reference.previous.len()-1).ok_or(Failure::Missing)?;let f=last.funded_by.as_ref().ok_or(Failure::Changed)?;if !row.spec.funded_response()||r.schema_version!=(if reference.schema_version==10{3}else{2})||(reference.schema_version==10)!=(row.spec.schema_version==19)||last.stop_reason!=Some(crate::pet_runtime::leases::StopReason::Pause)||last.closed_at.is_none()||f.previous!=reference.previous[..reference.previous.len()-1]{return Err(Failure::Changed);}self.funding_prefix_unlocked(f,&goal.id)?;}
             let seal=last.native_plan_revisions.iter().find(|s|s.reference==*r&&s.abandoned.is_none()).ok_or(Failure::RecoveryRequired)?;if last.phase!=GrantPhase::Closed||seal.target_goal_hash!=reference.source_goal_hash||!seal.content_matches(goal){return Err(Failure::Changed);}},
            _=>return Err(Failure::Invalid),
        }
        }
        for unused in &reference.unused{
            let(body,_)=self.grant_file(&format!("{}.json",unused.decision_id))?;
            if body.goal_id!=goal.id||body.decision_id!=unused.decision_id||body.phase!=GrantPhase::Revoked||!Self::never_claimed_zero(&body)||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=unused.grant_hash{return Err(Failure::Changed);}
        }
        if reference.native.as_ref().is_some_and(|n|Self::native_carry(goal,&bodies,&n.proposal_hash).ok().as_ref()!=Some(n)){return Err(Failure::Changed);}
        self.verify_uncertain_effects(goal,reference)?;Ok(bodies)
    }
    pub(in crate::pet_runtime::authority_store)fn native_carry(goal:&Record,bodies:&[Grant],proposal:&str)->Result<crate::pet_runtime::continuation::NativeCarry>{
        let target=crate::pet_runtime::native_plan_revisions::snapshot_at(goal,proposal).map_err(|_|Failure::Changed)?;
        use crate::pet_runtime::native_verification::ReceiptRef;
        let mut found=std::collections::BTreeMap::new();
        for(row,body)in goal.admissions.iter().zip(bodies){
            if !row.spec.deterministic()||!crate::pet_runtime::plans::original_matches(goal,&row.spec){return Err(Failure::Changed);}
            if let Some(messages)=&body.messages{let lease=body.lease.as_ref().ok_or(Failure::Missing)?;
                let original=crate::pet_runtime::plans::original_snapshot(goal,&row.spec).map_err(|_|Failure::Changed)?;
                for receipt in &messages.entries{if !receipt.scope.matches_snapshot(goal,&original)||!receipt.scope.input_matches(goal,&target)||!receipt.scope.belongs(&row.spec,lease){return Err(Failure::Changed);}
                    let r=ReceiptRef::historical(receipt).map_err(|_|Failure::Invalid)?;found.entry((r.plan_id.clone(),r.step_id.clone())).or_insert(r);
                }
            }
        }
        let mut receipts=vec![];for cause in target.causes(){if let Some(r)=found.remove(&(cause.plan_id,cause.step_id)){receipts.push(r);}}
        if !found.is_empty(){return Err(Failure::Changed);}
        Ok(crate::pet_runtime::continuation::NativeCarry{schema_version:1,proposal_hash:proposal.into(),receipts})
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
            if !matches!(goal.state,State::RecoveryRequired|State::Paused)||goal.admissions.is_empty(){return Ok(Err(Denial::RecoveryNotClosed));}
            if goal.admissions.len()>=32{return Err(Failure::Capacity);}
            let row=goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
            let funded_native=row.spec.funding.is_some()&&row.spec.deterministic()&&matches!(row.spec.schema_version,11|16);if funded_native{self.funding_current_unlocked(&row.spec,now)?;}
            let goal_hash=model::fingerprint(goal).map_err(|_|Failure::Invalid)?;
            let mut funded=if row.spec.funded_response()&&goal.state==State::Paused&&body.phase==GrantPhase::Closed&&body.stop_reason==Some(crate::pet_runtime::leases::StopReason::Pause){
                self.funding_current_unlocked(&row.spec,now)?;
                Some(crate::pet_runtime::continuation::FundedClosure{goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),decision_id:row.spec.decision_id.clone(),closed_at:body.closed_at.ok_or(Failure::RecoveryRequired)?})
            }else{None};
            let closure=body.recovery.as_ref().filter(|c|c.reference.target_state==State::RecoveryRequired&&c.reference.target_goal_revision==goal.revision&&c.target_goal_hash==goal_hash).map(|c|c.reference.clone());
            let drain=if closure.is_none()&&row.spec.deterministic()&&body.phase==GrantPhase::Closed{body.native_drain.as_ref().filter(|s|s.reference.continuable()&&s.reference.target_revision==goal.revision&&s.target_goal_hash==goal_hash&&s.target_present(goal)).map(|s|s.reference.clone())}else{None};
            let plan_revision=if closure.is_none()&&drain.is_none()&&(row.spec.deterministic()||row.spec.funded_response())&&body.phase==GrantPhase::Closed{body.native_plan_revisions.iter().rev().find(|s|s.abandoned.is_none()&&s.reference.target_revision==goal.revision&&s.target_goal_hash==goal_hash&&s.content_matches(goal)).map(|s|s.reference.clone())}else{None};
            if plan_revision.is_some(){funded=None;}
            if funded.is_none()&&closure.is_none()&&drain.is_none()&&plan_revision.is_none()||self.verify_recovered_goal(goal)?{return Ok(Err(Denial::RecoveryNotClosed));}
            self.recovery_profile(&body)?;
            let history=self.history(goal,now)?;
            if !history.scan_complete||!history.accounting_complete||!history.source_consistent{return Ok(Err(Denial::HistoryIncomplete));}
            let charge=history.cumulative_charge.ok_or(Failure::RecoveryRequired)?;
            if funded.is_some()&&(charge.unknown_model_results>0||charge.unknown_tool_results>0){return Ok(Err(Denial::UnreconciledEffects));}
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
                previous.push(Anchor{native_schema:row.spec.deterministic().then_some(row.spec.schema_version),decision_id:row.spec.decision_id.clone(),spec_hash:model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?,row_hash:model::fingerprint(row).map_err(|_|Failure::Invalid)?,grant_hash:model::fingerprint(&body).map_err(|_|Failure::Invalid)?,budget:row.spec.budget.clone(),resolved_model:row.spec.resolved_model.clone()});bodies.push(body);
            }
            let native=if row.spec.deterministic(){
                if charge.unknown_tool_results>0||charge.unknown_model_results>0{return Ok(Err(Denial::UnreconciledEffects));}
                Some(Self::native_carry(goal,&bodies,&crate::pet_runtime::plans::fingerprint(goal).map_err(|_|Failure::Invalid)?)?)
            }else{None};
            let is_result_plan=plan_revision.as_ref().is_some_and(|r|matches!(r.schema_version,2|3));let recursive_result_plan=plan_revision.as_ref().is_some_and(|r|r.schema_version==3);
            let reference=Reference{schema_version:if recursive_result_plan{10}else if funded.is_some()&&row.spec.schema_version==19{9}else if funded_native{8}else if is_result_plan{7}else if funded.is_some(){6}else if plan_revision.is_some(){5}else if drain.is_some(){4}else if native.is_some(){3}else if uncertain_effects.is_some(){2}else{1},plan_revision,drain,native,uncertain_effects,source_revision:goal.revision.clone(),source_sequence:goal.sequence,source_goal_hash:history.goal_hash,closure,history_hash:history.source_hash,evidence_fence:self.continuation_fence(goal,&bodies)?,previous,unused:history.unlinked_grants.into_iter().map(|g|Unused{decision_id:g.decision_id,grant_hash:g.hash}).collect(),charge,history_continuity_verified:false,additional_allowance:funded.is_none()&&!is_result_plan&&!funded_native,funded,reviewed_at:now};
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
            if self.rule_record_data(&name)?.is_some(){continue;}let(body,stamp)=self.grant_file(&name)?;
            if body.decision_id==spec.decision_id&&body.goal_id==spec.goal_id&&body.spec_hash==hash&&body.approved_at==spec.approved_at&&body.phase==GrantPhase::Issued&&Self::never_claimed_zero(&body){self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}
        }
        self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
}
