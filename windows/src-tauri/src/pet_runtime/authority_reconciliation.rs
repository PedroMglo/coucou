//! A coherent shared-reader snapshot of existing authenticated evidence. No
//! receipt repair, durability ACK, target IO or new authority is performed.
use super::*;
use crate::pet_runtime::{reconciliation::{self as view,Evidence,EvidenceKind as K,EvidenceStatus as S},effect_budget::{Kind,Phase},runtime_owner::{Observation,State as OwnerState,Reason},results,planning_output};
impl Authority{
    pub(in crate::pet_runtime)fn reconciliation(&self,goal:&Record,now:u64)->Result<view::Snapshot>{
        self.reconciliation_with_gate(goal,now,None)
    }
    pub(in crate::pet_runtime)fn reconciliation_with_gate(&self,goal:&Record,now:u64,retained:Option<&crate::pet_runtime::native_owner::AbsentLease>)->Result<view::Snapshot>{
        goal.validate().map_err(|_|Failure::Invalid)?;let _guard=self.lock_shared()?;let mut snapshot=view::Snapshot::empty(goal,now).map_err(|_|Failure::Invalid)?;
        let lineage=match self.verify_continuations_goal(goal).and_then(|_|self.verify_generated_goal(goal,true)).and_then(|_|self.verify_recovered_goal(goal).map(|_|())){Ok(())=>Evidence::verified(K::GoalLineage,goal.id.clone(),&snapshot.goal_hash)?,Err(e)=>Evidence::failed(K::GoalLineage,goal.id.clone(),e)};
        snapshot.source_consistent=lineage.status==S::Verified;snapshot.scan_complete=!matches!(lineage.status,S::Invalid|S::Unavailable);snapshot.evidence.push(lineage);
        let Some(row)=goal.admissions.last()else{return Ok(snapshot);};let spec=&row.spec;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let body=self.load(spec,proof)?;
        if spec.owner!=goal.spec.owner||spec.intent_hash!=model::fingerprint(&goal.spec).map_err(|_|Failure::Invalid)?||goal.execution_context.as_ref()!=Some(&spec.context){return Err(Failure::Changed);}
        if now<body.updated_at{return Err(Failure::Invalid);}
        let hash=model::fingerprint(&body).map_err(|_|Failure::Invalid)?;
        let owner=if let Some(gate)=retained{
            gate.current(self).map_err(|_|Failure::Changed)?;if !gate.matches(&hash,&body.revision){return Err(Failure::Changed);}
            Observation::from_record(body.runtime_owner.as_ref().ok_or(Failure::RecoveryRequired)?,OwnerState::OwnerAbsent,None)
        }else if matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked){Observation::without_owner(OwnerState::Closed,None)}else if body.lease.is_none(){Observation::without_owner(OwnerState::NotClaimed,None)}else if let Some(owner)=&body.runtime_owner{crate::pet_runtime::native_owner::observe(self,owner)}else{Observation::without_owner(OwnerState::LegacyUnknown,Some(Reason::LegacyOwnerMissing))};
        // During an absent-owner scan retain its exact native lock. The DTO
        // does not carry this object and cannot authorize a later mutation.
        let absence=if retained.is_none()&&owner.state==OwnerState::OwnerAbsent{Some(self.retained_absence(spec,proof)?)}else{None};
        let lease_matches=row.lease==body.lease;
        let owner_known=matches!(owner.state,OwnerState::OriginalProcessLive|OwnerState::OwnerAbsent|OwnerState::NotClaimed|OwnerState::Closed);
        let projection=match body.phase{GrantPhase::Issued=>goal.state==State::Queued,GrantPhase::Claimed|GrantPhase::Waiting=>goal.state==State::Leased,GrantPhase::Draining=>goal.state==State::Stopping&&row.stop_reason==body.stop_reason,GrantPhase::Closed|GrantPhase::Revoked=>!matches!(goal.state,State::Queued|State::Leased|State::Stopping)};
        snapshot.budget=Some(spec.budget.clone());snapshot.grant=Some(view::Grant{decision_id:body.decision_id.clone(),revision:body.revision.clone(),hash:hash.clone(),phase:match body.phase{GrantPhase::Issued=>"issued",GrantPhase::Claimed=>"claimed",GrantPhase::Waiting=>"waiting",GrantPhase::Draining=>"draining",GrantPhase::Closed=>"closed",GrantPhase::Revoked=>"revoked"},lease:body.lease.clone(),goal_lease_matches:lease_matches,control_projection_current:projection,stop_reason:body.stop_reason,updated_at:body.updated_at,closed_at:body.closed_at,owner});
        snapshot.source_consistent&=lease_matches&&owner_known;
        let inspected=self.reconciliation_evidence_unlocked(goal,row,&body)?;snapshot.attempts=inspected.attempts;snapshot.evidence.extend(inspected.evidence);snapshot.criteria_verified=inspected.criteria_verified;
        if goal.state==State::Completed{
            let evidence=match self.observe_completed_goal(goal){Ok(())=>{snapshot.goal_complete=true;Evidence::verified(K::Completion,goal.id.clone(),&goal.completion)?},Err(e)=>{snapshot.source_consistent=false;Evidence::failed(K::Completion,goal.id.clone(),e)}};snapshot.evidence.push(evidence);
        }
        let model_calls=snapshot.attempts.iter().filter(|a|a.kind=="model").count();let tool_calls=snapshot.attempts.len()-model_calls;let charged=snapshot.attempts.iter().map(|a|a.charged_output_bytes).sum();
        snapshot.accounting=Some(view::Accounting{scope:"latestAdmission",model_calls_used:model_calls,tool_calls_used:tool_calls,output_bytes_charged:charged,model_calls_remaining:usize::from(spec.budget.max_model_calls).saturating_sub(model_calls),tool_calls_remaining:usize::from(spec.budget.max_tool_calls).saturating_sub(tool_calls),output_bytes_remaining:spec.budget.max_output_bytes.saturating_sub(charged),unknown_attempts:snapshot.attempts.iter().filter(|a|!a.outcome_known).count()});
        snapshot.scan_complete=owner_known&&snapshot.evidence.iter().chain(snapshot.attempts.iter().flat_map(|a|&a.evidence)).all(|e|!matches!(e.status,S::Invalid|S::Unavailable));
        snapshot.criteria_verified&=snapshot.source_consistent&&snapshot.scan_complete;snapshot.goal_complete&=snapshot.source_consistent&&snapshot.scan_complete;
        let latest=self.load(spec,proof)?;if model::fingerprint(&latest).map_err(|_|Failure::Invalid)?!=hash{return Err(Failure::Changed);}
        if let Some(gate)=absence{gate.current(self).map_err(|_|Failure::Changed)?;if !gate.matches(&hash,&body.revision){return Err(Failure::Changed);}}
        self.fence()?;Ok(snapshot)
    }
    /// The caller retains the original goal and authority readers. Select an
    /// authentic admission directly; never fabricate a truncated goal/history.
    pub(in crate::pet_runtime::authority_store)fn reconciliation_evidence_unlocked(&self,goal:&Record,row:&crate::pet_runtime::admission::QueuedAdmission,body:&Grant)->Result<view::AdmissionEvidence>{
        let spec=&row.spec;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
        let mut snapshot=view::AdmissionEvidence::default();
        for a in body.budget_usage.as_ref().map(|b|b.attempts.as_slice()).unwrap_or(&[]){
            let mut attempt=view::Attempt{id:a.id.clone(),request_id:a.request_id.clone(),kind:match a.kind{Kind::Model=>"model",Kind::Tool=>"tool"},phase:match a.phase{Phase::Reserved=>"reserved",Phase::Started=>"started",Phase::Settled=>"settled",Phase::Uncertain=>"uncertain"},charged_output_bytes:a.charge(),accepted_output_bytes:a.accepted_output,outcome_known:false,evidence:vec![]};
            if a.kind==Kind::Model{
                let evidence=match results::lookup(self,spec,proof,&a.request_id){Ok(results::Lookup::Available{receipt})=>{let mut e=Evidence::verified(K::ModelResult,a.id.clone(),&receipt)?;e.outcome_known=true;attempt.outcome_known=true;e},Ok(_)=>Evidence::missing(K::ModelResult,a.id.clone()),Err(e)=>Evidence::failed(K::ModelResult,a.id.clone(),e)};attempt.evidence.push(evidence);
                if body.planning_usage.as_ref().is_some_and(|p|p.rounds.iter().any(|r|r.attempt_id==a.id)){
                    attempt.evidence.push(match planning_output::read(self,spec,proof,&a.id){Ok(Some(e))=>Evidence::verified(K::PlanningOutput,a.id.clone(),&e.body)?,Ok(None)=>Evidence::missing(K::PlanningOutput,a.id.clone()),Err(e)=>Evidence::failed(K::PlanningOutput,a.id.clone(),e)});
                }
            }
            snapshot.attempts.push(attempt);
        }
        if body.lease.is_some(){for plan in &goal.plans{
            match self.recipe_load(spec,proof,&plan.id){
                Ok(Some(scope))=>{
                    snapshot.evidence.push(Evidence::verified(K::Recipe,plan.id.clone(),&scope)?);
                    if scope.verification.is_some(){snapshot.evidence.push(match self.verification_load_unlocked(spec,proof,&scope){Ok(Some(report))=>{let mut e=Evidence::verified(K::Verification,scope.id.clone(),&report)?;e.criteria_verified=report.criteria_verified;snapshot.criteria_verified|=e.criteria_verified;e},Ok(None)=>Evidence::missing(K::Verification,scope.id.clone()),Err(e)=>Evidence::failed(K::Verification,scope.id.clone(),e)});}
                },Ok(None)=>{},Err(e)=>snapshot.evidence.push(Evidence::failed(K::Recipe,plan.id.clone(),e)),
            }
            for step in &plan.steps{let reference=format!("{}/{}",plan.id,step.id);
                match self.step_reconciliation_unlocked(spec,proof,&plan.id,&step.id){
                    Ok(Some((assessment,observed,evidence_hash)))=>{
                        let mut e=Evidence::verified(K::ToolStep,reference.clone(),&assessment.record)?;
                        if let Some(result)=observed{e.effect_verified=Some(result.observation.effect_verified);e.outcome_known=result.observation.effect_verified||matches!(result.observation.delivery,crate::spotify::goal::Delivery::NotDispatched);}
                        if let Some(a)=snapshot.attempts.iter_mut().find(|a|a.kind=="tool"&&a.request_id==assessment.record.scope.request_id){a.outcome_known=e.outcome_known;a.evidence.push(e.clone());}
                        snapshot.evidence.push(e);
                        if let Some(hash)=evidence_hash{snapshot.evidence.push(Evidence{kind:K::ToolObservation,reference,status:S::Verified,hash:Some(hash),reason:None,effect_verified:assessment.recovery.recorded_effect_verified,outcome_known:false,criteria_verified:false});}
                    },Ok(None)=>{},Err(e)=>snapshot.evidence.push(Evidence::failed(K::ToolStep,reference,e)),
                }
            }
        }}
        Ok(snapshot)
    }

}
