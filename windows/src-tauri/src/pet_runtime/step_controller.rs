//! Native review and dispatch. No stored grant reconstructs Ready/Target/Owned.
use super::*;
use crate::{pet_runtime::{steps::{self,Scope,Phase,Receipt,StepRecord},effect_budget::{Attempt,Kind,Phase as APhase}},spotify::goal::{Target,Action,Failure as TargetFailure}};
use serde::Serialize;
use serde_json::Value;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal {
    pub scope:Scope,pub goal_revision:String,pub intent:String,pub criteria:Vec<String>,
    pub arguments:Value,pub postcondition:String,pub prepared_at:u64,pub expires_at:u64,
    pub max_uses:u8,pub goal_criteria_verified:bool,
}
pub(in crate::pet_runtime)struct Review{proposal:Proposal,target:Arc<Target>,action:Action}
#[derive(Clone)]pub(in crate::pet_runtime)struct Ready{pub(in crate::pet_runtime)scope:Scope,target:Arc<Target>,action:Action}
fn error()->model::Error{model::Error::Conflict}
pub(in crate::pet_runtime)fn resource_key(binding:&crate::spotify::goal::Binding)->model::Result<String>{Ok(format!("mpris_{}",model::fingerprint(&(&binding.bus_id,&binding.owner))?))}
fn selected<'a>(goal:&'a Record,plan:&str,step:&str)->model::Result<(&'a model::Plan,&'a model::Step)>{
    let p=goal.plans.iter().find(|p|p.id==plan).ok_or(model::Error::NotFound)?;
    if !goal.tasks.iter().any(|t|t.id==p.task_id&&t.plan_id.as_ref()==Some(&p.id)&&t.state==model::WorkState::Proposed){return Err(error());}
    let s=p.steps.iter().find(|s|s.id==step&&s.state==model::WorkState::Proposed).ok_or(error())?;Ok((p,s))
}
fn action(step:&model::Step)->model::Result<Action>{
    let op=step.operation.as_ref().ok_or(model::Error::InvalidInput)?;
    if op.operation_id!="spotify.control"||op.version!=1||!op.valid(){return Err(model::Error::UnsupportedPlatform);}
    use crate::pet_operations::Music;
    match serde_json::from_value::<Music>(op.arguments.clone()).map_err(|_|model::Error::InvalidInput)?{
        Music::Play=>Ok(Action::Play),Music::Pause=>Ok(Action::Pause),Music::SetVolume{volume}=>Ok(Action::SetVolume{volume}),
        _=>Err(model::Error::UnsupportedPlatform),
    }
}
impl Owned{
    pub(in crate::pet_runtime)fn step_goal(&self,goal:&Record,config:&crate::pet_config::Configuration,now:u64)->model::Result<()>{
        goal.validate()?;
        if !self.policy_current(now)||goal.state!=model::State::Leased||goal.id!=self.live.lease.goal_id||goal.spec.owner!=self.live.spec.owner||goal.execution_context.as_ref()!=Some(&self.live.spec.context)
            ||model::fingerprint(&goal.spec)?!=self.live.spec.intent_hash||goal.admissions.last().is_none_or(|a|model::fingerprint(&a.spec).ok()!=model::fingerprint(&self.live.spec).ok())||goal.admissions.last().and_then(|a|a.lease.as_ref())!=Some(&self.live.lease)||self.live.spec.schema_version!=2
            ||!crate::pet_runtime::plans::matches(goal,&self.live.spec)||!config.permissions(&goal.spec.owner.pet_id).map_err(|_|error())?.music{return Err(error());}
        Ok(())
    }
    fn predecessors(&self,goal:&Record,plan:&str,step:&str,authority:&Authority)->model::Result<()>{
        let(p,_)=selected(goal,plan,step)?;
        for prior in p.steps.iter().take_while(|s|s.id!=step){
            let saved=authority.read_step(&self.live.spec,&self.live.proof,plan,&prior.id).map_err(|_|error())?.ok_or(error())?;
            if saved.phase!=Phase::Finished||saved.result.as_ref().is_none_or(|r|!r.observation.effect_verified||!r.observation.scope_current){return Err(error());}
        }Ok(())
    }
    pub(in crate::pet_runtime)fn review_step(&self,goal:&Record,plan:&str,step:&str,target:Arc<Target>,config:&crate::pet_config::Configuration,now:u64)->model::Result<Review>{
        self.prepare_step_review(goal,plan,step,target,config,now,true)
    }
    pub(in crate::pet_runtime)fn review_recipe_step(&self,goal:&Record,plan:&str,step:&str,target:Arc<Target>,config:&crate::pet_config::Configuration,now:u64)->model::Result<Review>{
        self.prepare_step_review(goal,plan,step,target,config,now,false)
    }
    fn prepare_step_review(&self,goal:&Record,plan:&str,step:&str,target:Arc<Target>,config:&crate::pet_config::Configuration,now:u64,predecessors:bool)->model::Result<Review>{
        self.step_goal(goal,config,now)?;let(_,s)=selected(goal,plan,step)?;let action=action(s)?;
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|error())?;if predecessors{self.predecessors(goal,plan,step,&authority)?;}
        if authority.read_step(&self.live.spec,&self.live.proof,plan,step).map_err(|_|error())?.is_some(){return Err(error());}
        if self.live.spec.budget.max_tool_calls==0{return Err(model::Error::LimitReached);}
        let output=authority.output_remaining(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(|_|error())?.min(steps::MAX_OUTPUT);
        if output<2048{return Err(model::Error::LimitReached);}
        let _worker=self.readiness_worker(now)?;target.read().map_err(|_|error())?;
        let op=s.operation.as_ref().ok_or(error())?;let grant_id=nonce()?;
        let scope=Scope{schema_version:1,request_id:format!("tool-{grant_id}"),grant_id,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),decision_id:self.live.spec.decision_id.clone(),lease_id:self.live.lease.id.clone(),controller_id:self.live.lease.controller_id.clone(),main_spec_hash:model::fingerprint(&self.live.spec)?,context_hash:self.live.spec.context_hash.clone(),proposal_hash:self.live.spec.proposal_hash.clone().ok_or(error())?,policy_revision:self.live.spec.policy_revision.clone(),plan_id:plan.into(),step_id:step.into(),operation_id:op.operation_id.clone(),version:op.version,arguments_hash:model::fingerprint(&op.arguments)?,resource:target.binding().clone(),approved_at:now,deadline_at:self.live.lease.deadline_at,max_output_bytes:output};
        if !scope.valid()||!scope.matches(goal){return Err(error());}
        let proposal=Proposal{scope,goal_revision:goal.revision.clone(),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),arguments:op.arguments.clone(),postcondition:s.postcondition.clone(),prepared_at:now,expires_at:now.saturating_add(120_000).min(self.live.lease.deadline_at),max_uses:1,goal_criteria_verified:false};
        Ok(Review{proposal,target,action})
    }
    pub(in crate::pet_runtime)fn run_step(&self,goal:&Record,ready:&Ready,config:&crate::pet_config::Configuration)->model::Result<StepRecord>{
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|error())?;
        self.run_step_with_authority(goal,ready,config,&authority)
    }
    pub(in crate::pet_runtime)fn run_step_with_authority(&self,goal:&Record,ready:&Ready,config:&crate::pet_config::Configuration,authority:&Authority)->model::Result<StepRecord>{
        self.run_step_with_resource(goal,ready,config,authority,None)
    }
    pub(in crate::pet_runtime)fn run_step_with_resource(&self,goal:&Record,ready:&Ready,config:&crate::pet_config::Configuration,authority:&Authority,held:Option<&crate::pet_resources::Hold>)->model::Result<StepRecord>{
        let stored=authority.read_step(&self.live.spec,&self.live.proof,&ready.scope.plan_id,&ready.scope.step_id).map_err(|_|error())?.ok_or(error())?;
        if stored.scope!=ready.scope{return Err(error());}
        if stored.phase==Phase::Finished{
            let result=stored.result.clone().ok_or(error())?;
            return authority.finish_step(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,stored.attempt.as_ref(),result,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable);
        }
        if stored.phase!=Phase::Approved{return Err(error());}
        self.step_goal(goal,config,crate::assistant_memory::now())?;
        if !ready.scope.belongs(&self.live.spec,&self.live.lease)||!ready.scope.matches(goal)||ready.target.binding()!=&ready.scope.resource{return Err(error());}
        self.predecessors(goal,&ready.scope.plan_id,&ready.scope.step_id,&authority)?;
        let worker=self.readiness_worker(crate::assistant_memory::now())?;
        let key=resource_key(&ready.scope.resource)?;
        let coordinator=self.live.chat.runtime_resources();
        let local=if held.is_none(){Some(coordinator.acquire(&key,Arc::new(AtomicBool::new(false)),Some(self.live.deadline)).map_err(|_|model::Error::Busy)?)}else{None};
        let resource=held.or(local.as_deref()).ok_or(model::Error::Conflict)?;
        if resource.provider_for(&coordinator)!=Some(key.as_str()){return Err(model::Error::Conflict);}
        let mut attempt=None;let mut armed=false;
        let observation=ready.target.apply(&ready.action,self.live.deadline,
            ||worker.in_flight_current(crate::assistant_memory::now())&&resource.current(),
            ||{
                let now=crate::assistant_memory::now();
                let native=Attempt{id:nonce().map_err(|_|TargetFailure::NotAuthorized)?,request_id:ready.scope.request_id.clone(),input_hash:model::fingerprint(&ready.scope).map_err(|_|TargetFailure::NotAuthorized)?,kind:Kind::Tool,output_reserved:ready.scope.max_output_bytes,phase:APhase::Reserved,accepted_output:None,reserved_at:now,started_at:None,finished_at:None};
                attempt=Some(native.clone());
                authority.reserve_step(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,&native,now).map_err(|_|TargetFailure::NotAuthorized)?;
                authority.reserve_attempt(&self.live.spec,&self.live.proof,&self.live.lease,&native,crate::assistant_memory::now()).map_err(|_|TargetFailure::NotAuthorized)?;
                authority.start_attempt(&self.live.spec,&self.live.proof,&self.live.lease,&native,crate::assistant_memory::now()).map_err(|_|TargetFailure::NotAuthorized)?;
                authority.start_step(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,&native,crate::assistant_memory::now()).map_err(|_|TargetFailure::NotAuthorized)?;
                if !worker.in_flight_current(crate::assistant_memory::now())||!resource.current(){return Err(TargetFailure::Cancelled);}
                armed=true;Ok(())
            });
        if attempt.is_some()&&!armed{return Err(model::Error::StorageUnavailable);}
        let result=Receipt{observation,finished_at:crate::assistant_memory::now(),criteria_verified:false,goal_complete:false};
        let bytes=serde_json::to_vec(&result).map_err(|_|model::Error::InvalidRecord)?.len() as u64;
        if bytes>ready.scope.max_output_bytes{return Err(model::Error::LimitReached);}
        // Preserve the native observation before accounting/final publication.
        // Failure here leaves the original attempt non-replayable and uncertain.
        authority.record_step_result(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,attempt.as_ref(),&result,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
        if let Some(attempt)=attempt.as_ref(){
            authority.finish_attempt(&self.live.spec,&self.live.proof,&self.live.lease,attempt,Some(bytes),crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
        }
        authority.finish_step(&self.live.spec,&self.live.proof,&self.live.lease,&ready.scope,attempt.as_ref(),result,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)
    }
}
impl Review{
    pub(in crate::pet_runtime)fn limit_output(mut self,bytes:u64)->model::Result<Self>{
        if !(2048..=self.proposal.scope.max_output_bytes).contains(&bytes){return Err(model::Error::LimitReached);}
        self.proposal.scope.max_output_bytes=bytes;Ok(self)
    }
    pub(in crate::pet_runtime)fn proposal(&self)->Proposal{self.proposal.clone()}
    pub(in crate::pet_runtime)fn approve(mut self,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,now:u64)->model::Result<(steps::Approved,Ready)>{
        if now<self.proposal.prepared_at||now>=self.proposal.expires_at||goal.revision!=self.proposal.goal_revision{return Err(error());}
        owner.step_goal(goal,config,now)?;
        if !self.proposal.scope.belongs(&owner.live.spec,&owner.live.lease)||!self.proposal.scope.matches(goal)||self.target.binding()!=&self.proposal.scope.resource{return Err(error());}
        let _worker=owner.readiness_worker(now)?;self.target.read().map_err(|_|error())?;
        self.proposal.scope.approved_at=now;
        let ready=Ready{scope:self.proposal.scope.clone(),target:self.target,action:self.action};
        Ok((steps::Approved::from_native_review(self.proposal.scope),ready))
    }
}
