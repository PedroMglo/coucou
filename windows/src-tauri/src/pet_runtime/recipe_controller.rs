//! Native exact plan review and bounded execution. No serialized manifest can
//! recreate these retained approvals, Ready steps, worker or physical target.
use super::*;
use super::step_controller::{Review as StepReview,Ready as StepReady};
use crate::{pet_runtime::{recipes::{self,Scope,Progress,Phase,Outcome,RunResult},steps::Approved as ApprovedStep},spotify::goal::Target};
use serde::Serialize;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{pub scope:Scope,pub goal_revision:String,pub intent:String,pub criteria:Vec<String>,pub steps:Vec<super::step_controller::Proposal>,pub prepared_at:u64,pub expires_at:u64,pub criteria_verified:bool,pub goal_complete:bool}
impl Proposal {
    /// The exact review repeats original context once, retaining every step's
    /// native scope, arguments and postcondition without serializing it N times.
    pub(in crate::pet_runtime)fn decision_review(&self)->serde_json::Value {
        let steps=self.steps.iter().map(|s|serde_json::json!({"scope":s.scope,"goalRevision":s.goal_revision,"arguments":s.arguments,"postcondition":s.postcondition,"preparedAt":s.prepared_at,"expiresAt":s.expires_at,"maxUses":s.max_uses,"goalCriteriaVerified":s.goal_criteria_verified})).collect::<Vec<_>>();
        serde_json::json!({"scope":self.scope,"goalRevision":self.goal_revision,"intent":self.intent,"criteria":self.criteria,"steps":steps,"preparedAt":self.prepared_at,"expiresAt":self.expires_at,"criteriaVerified":self.criteria_verified,"goalComplete":self.goal_complete})
    }
}
pub(in crate::pet_runtime)struct Review{proposal:Proposal,reviews:Vec<StepReview>,target:Arc<Target>}
pub(in crate::pet_runtime)struct Approved{scope:Scope}
pub(in crate::pet_runtime)struct Ready{scope:Scope,steps:Vec<(ApprovedStep,StepReady)>,target:Arc<Target>,observed:Mutex<Option<Observed>>,active:AtomicBool,running:AtomicBool}
pub(in crate::pet_runtime)struct Observed{report:crate::pet_runtime::criteria::Verification}
impl Observed{pub(in crate::pet_runtime)fn report(&self)->&crate::pet_runtime::criteria::Verification{&self.report}}
impl Approved{pub(in crate::pet_runtime)fn scope(&self)->&Scope{&self.scope}}
impl Ready{
    pub(in crate::pet_runtime)fn scope(&self)->&Scope{&self.scope}
    pub(in crate::pet_runtime)fn activate(&self,registered:&Scope)->model::Result<()>{if registered!=&self.scope{return Err(blocked());}self.active.store(true,Ordering::Release);Ok(())}
}
struct Running<'a>(&'a AtomicBool);
impl Drop for Running<'_>{fn drop(&mut self){self.0.store(false,Ordering::Release);}}
fn blocked()->model::Error{model::Error::Conflict}
impl Review{
    pub(in crate::pet_runtime)fn approve_decision(self,decision:crate::pet_runtime::recipe_decisions::Decision,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,now:u64)->model::Result<(Approved,Ready)>{
        if !decision.approved()||!decision.matches(&self.proposal,goal,owner,now){return Err(blocked());}self.approve(owner,goal,config,now)
    }
    pub(in crate::pet_runtime)fn proposal(&self)->Proposal{self.proposal.clone()}
    pub(in crate::pet_runtime)fn approve(self,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,now:u64)->model::Result<(Approved,Ready)>{
        if goal.revision!=self.proposal.goal_revision||now<self.proposal.prepared_at||now>=self.proposal.expires_at||!self.proposal.scope.matches(goal){return Err(blocked());}
        owner.step_goal(goal,config,now)?;let mut steps=Vec::with_capacity(self.reviews.len());
        for review in self.reviews{steps.push(review.approve(owner,goal,config,now)?);}
        let mut scope=self.proposal.scope;scope.approved_at=now;scope.steps=steps.iter().map(|(_,r)|r.scope.clone()).collect();
        if !scope.belongs(&owner.live.spec,&owner.live.lease)||!scope.matches(goal){return Err(blocked());}
        let authority=Authority::existing(&owner.live.authority_path).map_err(|_|blocked())?;
        authority.recipe_headroom(&owner.live.spec,&owner.live.proof,&owner.live.lease,steps.len(),scope.steps.iter().map(|s|s.max_output_bytes).sum(),now).map_err(|_|model::Error::LimitReached)?;
        Ok((Approved{scope:scope.clone()},Ready{scope,steps,target:self.target,observed:Mutex::new(None),active:AtomicBool::new(false),running:AtomicBool::new(false)}))
    }
}
impl Owned{
    /// Resolve exactly the same native binding for a cached/shared approval as
    /// for a new review. Empty generated-plan input selects its signed mapping.
    pub(in crate::pet_runtime)fn recipe_input_binding(&self,goal:&Record,plan:&str,input:&[crate::pet_runtime::criteria::Input])->model::Result<Option<crate::pet_runtime::criteria::Binding>> {
        if self.live.spec.schema_version==3 {
            let authority=Authority::existing(&self.live.authority_path).map_err(|_|blocked())?;
            let (_,candidate)=authority.generated_candidate(&self.live.spec,&self.live.proof,&self.live.lease,goal,plan).map_err(|_|blocked())?;
            if !input.is_empty()&&crate::pet_runtime::criteria::Binding::bind(goal,input)?!=candidate.verification{return Err(blocked());}Ok(Some(candidate.verification))
        }else if input.is_empty(){Ok(None)}else{Ok(Some(crate::pet_runtime::criteria::Binding::bind(goal,input)?))}
    }
    #[cfg(test)]pub(in crate::pet_runtime)fn review_recipe(&self,goal:&Record,plan_id:&str,request:&str,target:Arc<Target>,config:&crate::pet_config::Configuration,now:u64)->model::Result<Review>{
        self.review_recipe_with_criteria(goal,plan_id,request,target,config,&[],now)
    }
    pub(in crate::pet_runtime)fn review_recipe_with_criteria(&self,goal:&Record,plan_id:&str,request:&str,target:Arc<Target>,config:&crate::pet_config::Configuration,criteria:&[crate::pet_runtime::criteria::Input],now:u64)->model::Result<Review>{
        self.step_goal(goal,config,now)?;if !model::key(request){return Err(model::Error::InvalidInput);}
        let generated=if self.live.spec.schema_version==3{
            let authority=Authority::existing(&self.live.authority_path).map_err(|_|blocked())?;
            Some(authority.generated_candidate(&self.live.spec,&self.live.proof,&self.live.lease,goal,plan_id).map_err(|_|blocked())?)
        }else{None};
        let verification=if let Some((_,candidate))=&generated{
            if !criteria.is_empty()&&crate::pet_runtime::criteria::Binding::bind(goal,criteria)?!=candidate.verification{return Err(blocked());}
            Some(candidate.verification.clone())
        }else if criteria.is_empty(){None}else{Some(crate::pet_runtime::criteria::Binding::bind(goal,criteria)?)};
        let plan=goal.plans.iter().find(|p|p.id==plan_id).ok_or(model::Error::NotFound)?;
        if plan.steps.is_empty()||plan.steps.len()>recipes::MAX_STEPS{return Err(model::Error::LimitReached);}
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|blocked())?;
        if authority.read_recipe(&self.live.spec,&self.live.proof,plan_id).map_err(|_|blocked())?.is_some(){return Err(blocked());}
        let output=authority.output_remaining(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(|_|model::Error::LimitReached)?;
        let each=(output/plan.steps.len()as u64).min(crate::pet_runtime::steps::MAX_OUTPUT);
        if each<2048{return Err(model::Error::LimitReached);}
        authority.recipe_headroom(&self.live.spec,&self.live.proof,&self.live.lease,plan.steps.len(),each*plan.steps.len()as u64,now).map_err(|_|model::Error::LimitReached)?;
        let mut reviews=vec![];for step in &plan.steps{reviews.push(self.review_recipe_step(goal,plan_id,&step.id,target.clone(),config,now)?.limit_output(each)?);}
        let proposed:Vec<_>=reviews.iter().map(StepReview::proposal).collect();let scopes:Vec<_>=proposed.iter().map(|p|p.scope.clone()).collect();
        let generated_plan=generated.map(|(reference,_)|reference);
        let proposal_hash=if generated_plan.is_some(){crate::pet_runtime::plans::fingerprint(goal)?}else{self.live.spec.proposal_hash.clone().ok_or(blocked())?};
        let scope=Scope{schema_version:if generated_plan.is_some(){3}else if verification.is_some(){2}else{1},generated_plan,id:nonce()?,request_id:request.into(),goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),decision_id:self.live.spec.decision_id.clone(),lease_id:self.live.lease.id.clone(),controller_id:self.live.lease.controller_id.clone(),plan_id:plan_id.into(),proposal_hash,approved_at:now,deadline_at:self.live.lease.deadline_at,steps:scopes,verification};
        if !scope.belongs(&self.live.spec,&self.live.lease)||!scope.matches(goal){return Err(blocked());}
        let expires_at=now.saturating_add(120_000).min(scope.deadline_at);
        let proposal=Proposal{scope,goal_revision:goal.revision.clone(),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),steps:proposed,prepared_at:now,expires_at,criteria_verified:false,goal_complete:false};
        if serde_json::to_vec(&proposal.decision_review()).map_err(|_|model::Error::InvalidRecord)?.len()>768*1024{return Err(model::Error::LimitReached);}
        Ok(Review{proposal,reviews,target})
    }
    fn recipe_progress(&self,authority:&Authority,ready:&Ready)->model::Result<Progress>{
        let progress=authority.recipe_progress(&self.live.spec,&self.live.proof,&ready.scope.plan_id).map_err(|_|model::Error::StorageUnavailable)?.ok_or(model::Error::NotFound)?;
        if progress.scope!=ready.scope{return Err(blocked());}Ok(progress)
    }
    pub(in crate::pet_runtime)fn run_recipe(&self,goal:&Record,ready:&Ready,mut configuration:impl FnMut()->model::Result<crate::pet_config::Configuration>,mut notify:impl FnMut(&Progress))->model::Result<RunResult>{
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|blocked())?;
        self.run_recipe_with_authority(goal,ready,&authority,&mut configuration,&mut notify)
    }
    pub(in crate::pet_runtime)fn run_recipe_with_authority(&self,goal:&Record,ready:&Ready,authority:&Authority,mut configuration:impl FnMut()->model::Result<crate::pet_config::Configuration>,mut notify:impl FnMut(&Progress))->model::Result<RunResult>{
        if !ready.active.load(Ordering::Acquire){return Err(blocked());}
        if ready.running.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(model::Error::Busy);}
        let _running=Running(&ready.running);
        let mut progress=self.recipe_progress(authority,ready)?;notify(&progress);
        if progress.phase==Phase::Verified&&(ready.scope.verification.is_none()||progress.verification.is_some()){return Ok(RunResult{outcome:Outcome::Verified,progress,completion:None});}
        if progress.phase==Phase::RecoveryRequired{return Ok(RunResult{outcome:Outcome::RecoveryRequired,progress,completion:None});}
        if !ready.scope.matches(goal)||!ready.scope.belongs(&self.live.spec,&self.live.lease){return Err(blocked());}
        let config=configuration()?;self.step_goal(goal,&config,crate::assistant_memory::now())?;
        let worker=self.readiness_worker(crate::assistant_memory::now())?;
        let first=ready.scope.steps.first().ok_or(blocked())?;let key=super::step_controller::resource_key(&first.resource)?;
        let resource=self.live.chat.runtime_resources().acquire(&key,Arc::new(AtomicBool::new(false)),Some(self.live.deadline)).map_err(|_|model::Error::Busy)?;
        for(approved,step)in &ready.steps{
            if !worker.in_flight_current(crate::assistant_memory::now())||!resource.current(){return Ok(RunResult{outcome:Outcome::Invalidated,progress,completion:None});}
            let stored=authority.read_step_assessment(&self.live.spec,&self.live.proof,&step.scope.plan_id,&step.scope.step_id).map_err(|_|model::Error::StorageUnavailable)?;
            if stored.as_ref().is_some_and(|a|a.record.scope!=step.scope){return Err(blocked());}
            if stored.as_ref().is_some_and(|a|a.record.phase==crate::pet_runtime::steps::Phase::Finished&&a.record.result.as_ref().is_some_and(|r|r.observation.effect_verified&&r.observation.scope_current)){continue;}
            if stored.as_ref().is_some_and(|a|a.record.phase!=crate::pet_runtime::steps::Phase::Approved){return Ok(RunResult{outcome:Outcome::RecoveryRequired,progress,completion:None});}
            let config=configuration()?;
            if self.step_goal(goal,&config,crate::assistant_memory::now()).is_err(){return Ok(RunResult{outcome:Outcome::Invalidated,progress,completion:None});}
            if authority.recipe_headroom(&self.live.spec,&self.live.proof,&self.live.lease,1,step.scope.max_output_bytes,crate::assistant_memory::now()).is_err(){return Ok(RunResult{outcome:Outcome::Blocked,progress,completion:None});}
            if authority.issue_step(&self.live.spec,&self.live.proof,&self.live.lease,approved,crate::assistant_memory::now()).is_err(){return Ok(RunResult{outcome:Outcome::Blocked,progress,completion:None});}
            let result=self.run_step_with_resource(goal,step,&config,authority,Some(&resource));
            progress=self.recipe_progress(authority,ready)?;notify(&progress);
            if result.is_err()||progress.phase==Phase::RecoveryRequired{return Ok(RunResult{outcome:if progress.phase==Phase::RecoveryRequired{Outcome::RecoveryRequired}else{Outcome::Blocked},progress,completion:None});}
        }
        progress=self.recipe_progress(authority,ready)?;
        if progress.phase==Phase::Verified&&ready.scope.verification.is_some(){
            let mut captured=ready.observed.lock().map_err(|_|model::Error::StorageUnavailable)?;
            if captured.is_none(){*captured=Some(self.observe_final(goal,ready,&resource,&worker)?);}
            let observed=captured.as_ref().ok_or(blocked())?;
            if authority.record_verification(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,observed,crate::assistant_memory::now()).is_err(){return Ok(RunResult{outcome:Outcome::RecoveryRequired,progress,completion:None});}
            progress=self.recipe_progress(authority,ready)?;notify(&progress);
        }
        Ok(RunResult{outcome:if progress.phase==Phase::Verified{Outcome::Verified}else{Outcome::Blocked},progress,completion:None})
    }
    fn observe_final(&self,goal:&Record,ready:&Ready,resource:&crate::pet_resources::Hold,worker:&WorkerGuard)->model::Result<Observed>{
        use crate::pet_runtime::criteria::{Verification,CompletionRef,CriterionResult,Status};
        let binding=ready.scope.verification.as_ref().ok_or(blocked())?;
        if !binding.matches(goal){return Err(blocked());}let expected=&ready.scope.steps.first().ok_or(blocked())?.resource;
        let current=worker.in_flight_current(crate::assistant_memory::now())&&resource.current()&&ready.target.binding()==expected;
        let snapshot=if current{ready.target.read().ok()}else{None};
        let scope_current=current&&snapshot.is_some()&&worker.in_flight_current(crate::assistant_memory::now())&&resource.current();
        let results=goal.spec.criteria.iter().enumerate().map(|(i,text)|{
            let condition=binding.conditions.iter().find(|c|c.criterion_index as usize==i);
            let status=match(condition,snapshot.as_ref(),scope_current){(Some(c),Some(s),true)=>if c.predicate.matches(s){Status::Met}else{Status::Unmet},_=>Status::Unknown};
            Ok(CriterionResult{criterion_index:i as u32,criterion_hash:model::fingerprint(text)?,status})
        }).collect::<model::Result<Vec<_>>>()?;
        let criteria_verified=scope_current&&results.iter().all(|r|r.status==Status::Met);let now=crate::assistant_memory::now();
        let reference=CompletionRef{schema_version:1,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),recipe_id:ready.scope.id.clone(),decision_id:self.live.spec.decision_id.clone(),lease_id:self.live.lease.id.clone(),plan_id:ready.scope.plan_id.clone(),source_revision:goal.revision.clone(),target_revision:nonce()?,report_hash:String::new(),observed_at:snapshot.as_ref().map(|s|s.observed_at).unwrap_or(0)};
        let mut report=Verification{schema_version:1,reference,recipe_hash:model::fingerprint(&ready.scope)?,spec_hash:model::fingerprint(&goal.spec)?,source_sequence:goal.sequence,source_updated_at:goal.updated_at,source_record_hash:model::fingerprint(goal)?,snapshot,scope_current,recorded_at:now,results,criteria_verified};
        report.reference.report_hash=report.content_hash()?;if !report.valid_for(&ready.scope){return Err(blocked());}Ok(Observed{report})
    }
    pub(in crate::pet_runtime)fn run_recipe_for_store(&self,store:&Store,goal:&Record,ready:&Ready,configuration:impl FnMut()->model::Result<crate::pet_config::Configuration>,notify:impl FnMut(&Progress))->model::Result<RunResult>{
        let mut result=self.run_recipe(goal,ready,configuration,notify)?;
        if result.progress.phase!=Phase::Verified||ready.scope.verification.is_none(){return Ok(result);}
        if !result.progress.criteria_verified{result.outcome=match result.progress.verification.as_ref(){Some(v)if v.results.iter().any(|r|r.status==crate::pet_runtime::criteria::Status::Unknown)=>Outcome::CriteriaIncomplete,Some(_)=>Outcome::CriteriaUnmet,None=>Outcome::RecoveryRequired};return Ok(result);}
        let receipt=self.complete_retained(store,goal,&ready.scope)?;
        let confirmed=receipt.status!=CommitStatus::Uncertain;result.completion=Some(receipt.status);result.progress.goal_complete=confirmed;
        result.outcome=if confirmed{Outcome::GoalCompleted}else{Outcome::RecoveryRequired};Ok(result)
    }
}
