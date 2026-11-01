//! Separate, exact authority for one goal step. Serializable scope/records are
//! data: only a native retained Target, live Owned controller and review may act.
use super::{admission::AdmissionSpec, effect_budget::{Attempt,Kind,Phase as AttemptPhase}, leases::Lease, model::{self,Record,WorkState}};
use crate::spotify::goal::{Binding,Observation,Delivery};
use serde::{Serialize,Deserialize};
pub(super) const MAX_OUTPUT:u64=16_000;
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Scope {
    #[serde(default,skip_serializing_if="Option::is_none")]pub generated_plan:Option<super::generated_plans::Reference>,
    pub schema_version:u32,pub grant_id:String,pub request_id:String,
    pub goal_id:String,pub pet_id:String,pub provider:String,pub decision_id:String,
    pub lease_id:String,pub controller_id:String,pub main_spec_hash:String,
    pub context_hash:String,pub proposal_hash:String,pub policy_revision:String,
    pub plan_id:String,pub step_id:String,pub operation_id:String,pub version:u32,
    pub arguments_hash:String,pub resource:Binding,pub approved_at:u64,
    pub deadline_at:u64,pub max_output_bytes:u64,
}
impl Scope {
    pub(super) fn slot(&self)->model::Result<String>{slot(&self.decision_id,&self.plan_id,&self.step_id)}
    pub(super) fn valid(&self)->bool{
        (match(self.schema_version,&self.generated_plan){(1,None)=>true,(2,Some(r))=>r.valid()&&r.published_at<=self.approved_at&&r.plan_id==self.plan_id&&r.decision_id==self.decision_id&&r.source.lease_id==self.lease_id&&r.source.controller_id==self.controller_id&&r.proposal_hash==self.proposal_hash,_=>false})&&model::token(&self.grant_id)&&model::key(&self.request_id)&&model::goal_id(&self.goal_id)
            &&model::key(&self.pet_id)&&model::key(&self.provider)&&model::token(&self.decision_id)
            &&model::token(&self.lease_id)&&model::token(&self.controller_id)&&model::key(&self.plan_id)&&model::key(&self.step_id)
            &&self.operation_id=="spotify.control"&&self.version==1
            &&[&self.main_spec_hash,&self.context_hash,&self.proposal_hash,&self.policy_revision,&self.arguments_hash].iter().all(|s|super::authority::decode(s).is_some())
            &&self.resource.schema_version==1&&!self.resource.bus_id.is_empty()&&self.resource.bus_id.len()<=128
            &&self.resource.owner.starts_with(':')&&self.resource.owner.len()<=256&&self.resource.pid>0
            &&!self.resource.identity.trim().is_empty()&&self.resource.identity.len()<=256
            &&self.approved_at>0&&self.approved_at<self.deadline_at&&self.deadline_at<=8_640_000_000_000_000
            &&(2048..=MAX_OUTPUT).contains(&self.max_output_bytes)
    }
    pub(super) fn belongs(&self,spec:&AdmissionSpec,lease:&Lease)->bool{
        self.valid()&&(match(self.schema_version,spec.schema_version,&self.generated_plan){(1,2,None)=>Some(&self.proposal_hash)==spec.proposal_hash.as_ref(),(2,3,Some(r))=>r.source.matches(spec,lease)&&r.decision_id==spec.decision_id,_=>false})&&self.main_spec_hash==model::fingerprint(spec).unwrap_or_default()
            &&self.goal_id==spec.goal_id&&self.pet_id==spec.owner.pet_id&&self.provider==spec.owner.provider
            &&self.decision_id==spec.decision_id&&self.lease_id==lease.id&&self.controller_id==lease.controller_id
            &&self.context_hash==spec.context_hash
            &&self.policy_revision==spec.policy_revision&&self.deadline_at==lease.deadline_at&&self.approved_at>=lease.claimed_at
    }
    pub(super) fn matches(&self,goal:&Record)->bool{
        if goal.validate().is_err()||goal.id!=self.goal_id||goal.spec.owner.pet_id!=self.pet_id||super::plans::fingerprint(goal).ok().as_ref()!=Some(&self.proposal_hash){return false;}
        if self.generated_plan.as_ref().is_some_and(|r|!r.structurally_present(goal)){return false;}
        let Some(plan)=goal.plans.iter().find(|p|p.id==self.plan_id)else{return false;};
        if !goal.tasks.iter().any(|t|t.id==plan.task_id&&t.plan_id.as_ref()==Some(&plan.id)&&t.state==WorkState::Proposed){return false;}
        let Some(step)=plan.steps.iter().find(|s|s.id==self.step_id&&s.state==WorkState::Proposed)else{return false;};
        step.operation.as_ref().is_some_and(|op|op.operation_id==self.operation_id&&op.version==self.version&&model::fingerprint(&op.arguments).ok().as_ref()==Some(&self.arguments_hash))
    }
}
pub(super) fn slot(decision:&str,plan:&str,step:&str)->model::Result<String>{
    if !model::token(decision)||!model::key(plan)||!model::key(step){return Err(model::Error::InvalidInput);}
    model::fingerprint(&(decision,plan,step))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Phase { Approved,Reserved,Started,Finished }
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Receipt { pub observation:Observation,pub finished_at:u64,pub criteria_verified:bool,pub goal_complete:bool }
impl Receipt {
    pub(super) fn valid_for(&self,scope:&Scope,recorded_at:u64)->bool{
        if self.criteria_verified||self.goal_complete||self.finished_at<scope.approved_at||self.finished_at>recorded_at{return false;}
        for snapshot in [&self.observation.before,&self.observation.after].into_iter().flatten(){
            if !matches!(snapshot.playback_status.as_str(),"Playing"|"Paused"|"Stopped")||!snapshot.volume.is_finite()||!(0.0..=1.0).contains(&snapshot.volume)||snapshot.observed_at<scope.approved_at||snapshot.observed_at>self.finished_at{return false;}
        }
        !self.observation.effect_verified||(self.observation.scope_current&&self.observation.after.is_some()&&self.observation.delivery!=Delivery::NotDispatched)
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum RecoveryStatus{Prepared,Reserved,ResultMissing,ResultRecorded,Finished}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum RecoveryBudget{Reserved,Started,Settled,Uncertain}
impl From<AttemptPhase>for RecoveryBudget{fn from(p:AttemptPhase)->Self{match p{AttemptPhase::Reserved=>Self::Reserved,AttemptPhase::Started=>Self::Started,AttemptPhase::Settled=>Self::Settled,AttemptPhase::Uncertain=>Self::Uncertain}}}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Recovery {
    pub status:RecoveryStatus,pub budget_phase:Option<RecoveryBudget>,
    pub recorded_effect_verified:Option<bool>,pub reconciliation_available:bool,
    #[serde(skip_serializing_if="not_frozen")]pub accounting_frozen:bool,
    #[serde(skip_serializing_if="Option::is_none")]pub recorded_result:Option<Receipt>,
}
fn not_frozen(value:&bool)->bool{!*value}
impl Recovery{pub(super)fn freeze_accounting(&mut self){self.accounting_frozen=true;self.reconciliation_available=false;}}
pub(super) struct Assessment{pub record:StepRecord,pub recovery:Recovery}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct StepRecord {
    pub schema_version:u32,pub scope:Scope,pub phase:Phase,pub updated_at:u64,
    pub attempt:Option<Attempt>,pub result:Option<Receipt>,
}
impl StepRecord {
    pub(super) fn valid(&self)->bool{
        if self.schema_version!=1||!self.scope.valid()||self.updated_at<self.scope.approved_at||self.updated_at>8_640_000_000_000_000{return false;}
        if self.attempt.as_ref().is_some_and(|a|!a.valid()||a.kind!=Kind::Tool||a.request_id!=self.scope.request_id||a.input_hash!=model::fingerprint(&self.scope).unwrap_or_default()||a.output_reserved>self.scope.max_output_bytes){return false;}
        if let Some(result)=&self.result{
            if !result.valid_for(&self.scope,self.updated_at){return false;}
        }
        match self.phase{
            Phase::Approved=>self.attempt.is_none()&&self.result.is_none(),
            Phase::Reserved=>self.attempt.as_ref().is_some_and(|a|a.phase==AttemptPhase::Reserved)&&self.result.is_none(),
            Phase::Started=>self.attempt.as_ref().is_some_and(|a|a.phase==AttemptPhase::Started)&&self.result.is_none(),
            Phase::Finished=>self.result.is_some()&&match &self.attempt{
                Some(a)=>matches!(a.phase,AttemptPhase::Settled|AttemptPhase::Uncertain),
                None=>self.result.as_ref().is_some_and(|r|r.observation.delivery==Delivery::NotDispatched&&!r.observation.effect_verified),
            },
        }
    }
}
/// Native review owns this value; no IPC or JSON can construct it.
pub(super) struct Approved { scope:Scope }
impl Approved {
    pub(super) fn from_native_review(scope:Scope)->Self{Self{scope}}
    pub(super) fn scope(&self)->&Scope{&self.scope}
}
