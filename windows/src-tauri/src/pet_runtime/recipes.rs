//! A reviewed finite sequence is data, never a reconstructed worker/target.
use super::{model,admission::AdmissionSpec,leases::Lease,steps};
use serde::{Serialize,Deserialize};
pub(super)const MAX_STEPS:usize=64;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Scope{
    #[serde(default,skip_serializing_if="Option::is_none")]pub generated_plan:Option<super::generated_plans::Reference>,
    pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,
    pub provider:String,pub decision_id:String,pub lease_id:String,pub controller_id:String,
    pub plan_id:String,pub proposal_hash:String,pub approved_at:u64,pub deadline_at:u64,pub steps:Vec<steps::Scope>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub verification:Option<super::criteria::Binding>,
}
impl Scope{
    pub(super)fn slot(&self)->model::Result<String>{slot(&self.decision_id,&self.plan_id)}
    pub(super)fn valid(&self)->bool{
        if !match(self.schema_version,&self.generated_plan,&self.verification){(1,None,None)=>true,(2,None,Some(v))=>v.valid(),(3,Some(r),Some(v))=>r.valid()&&r.published_at<=self.approved_at&&r.plan_id==self.plan_id&&r.decision_id==self.decision_id&&r.proposal_hash==self.proposal_hash&&v.valid(),_=>false}{return false;}
        if !model::token(&self.id)||!model::key(&self.request_id)||!model::goal_id(&self.goal_id)
            ||!model::key(&self.pet_id)||!model::key(&self.provider)||!model::token(&self.decision_id)||!model::token(&self.lease_id)
            ||!model::token(&self.controller_id)||!model::key(&self.plan_id)||super::authority::decode(&self.proposal_hash).is_none()
            ||self.approved_at==0||self.approved_at>=self.deadline_at||self.steps.is_empty()||self.steps.len()>MAX_STEPS{return false;}
        let mut ids=std::collections::BTreeSet::new();let mut grants=std::collections::BTreeSet::new();let mut requests=std::collections::BTreeSet::new();
        self.steps.iter().all(|s|s.valid()&&s.generated_plan==self.generated_plan&&s.goal_id==self.goal_id&&s.pet_id==self.pet_id&&s.provider==self.provider
            &&s.decision_id==self.decision_id&&s.lease_id==self.lease_id&&s.controller_id==self.controller_id&&s.plan_id==self.plan_id
            &&s.proposal_hash==self.proposal_hash&&s.approved_at==self.approved_at&&s.deadline_at==self.deadline_at
            &&s.resource==self.steps[0].resource&&ids.insert(&s.step_id)&&grants.insert(&s.grant_id)&&requests.insert(&s.request_id))
    }
    pub(super)fn belongs(&self,spec:&AdmissionSpec,lease:&Lease)->bool{self.valid()&&self.steps.iter().all(|s|s.belongs(spec,lease))}
    pub(super)fn matches(&self,goal:&model::Record)->bool{
        self.valid()&&self.verification.as_ref().is_none_or(|v|v.matches(goal))&&super::plans::fingerprint(goal).ok().as_ref()==Some(&self.proposal_hash)
            &&goal.plans.iter().find(|p|p.id==self.plan_id).is_some_and(|p|p.steps.len()==self.steps.len()
                &&p.steps.iter().zip(&self.steps).all(|(a,b)|a.id==b.step_id&&b.matches(goal)))
    }
}
pub(super)fn slot(decision:&str,plan:&str)->model::Result<String>{if !model::token(decision)||!model::key(plan){return Err(model::Error::InvalidInput);}model::fingerprint(&(decision,plan))}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Phase{Reviewed,Partial,Verified,RecoveryRequired}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct StepProgress{pub step_id:String,pub phase:Option<steps::Phase>,pub recovery:Option<steps::Recovery>,pub result:Option<steps::Receipt>}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Progress{pub scope:Scope,pub phase:Phase,pub verified_steps:usize,pub total_steps:usize,pub steps:Vec<StepProgress>,pub verification:Option<super::criteria::Verification>,pub criteria_verified:bool,pub goal_complete:bool,pub execution_available:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Outcome{Verified,Blocked,Invalidated,RecoveryRequired,CriteriaUnmet,CriteriaIncomplete,GoalCompleted}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RunResult{pub outcome:Outcome,pub progress:Progress,pub completion:Option<super::model::CommitStatus>}
