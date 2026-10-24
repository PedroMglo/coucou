//! Serializable lease metadata is never a worker or an execution permit.
use super::{model,admission::AdmissionSpec};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Lease{
    pub schema_version:u32,pub id:String,pub controller_id:String,pub request_id:String,
    pub decision_id:String,pub goal_id:String,pub provider:String,pub conversation_id:String,
    pub model:String,pub policy_revision:String,pub capability_revision:String,pub source_revision:String,
    pub claimed_at:u64,pub deadline_at:u64,
}
impl Lease{
    pub(super)fn basic(&self)->bool{
        self.schema_version==1&&model::token(&self.id)&&model::token(&self.controller_id)&&model::key(&self.request_id)
        &&model::token(&self.decision_id)&&model::goal_id(&self.goal_id)&&model::key(&self.provider)
        &&crate::assistant_memory::valid_id(&self.conversation_id)&&!self.model.is_empty()&&self.model.len()<=256
        &&self.policy_revision.len()==64&&self.policy_revision.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))
        &&!self.capability_revision.is_empty()&&self.capability_revision.len()<=256&&model::token(&self.source_revision)
        &&self.claimed_at>0&&self.claimed_at<self.deadline_at&&self.deadline_at<=8_640_000_000_000_000
    }
    pub(super)fn valid(&self,spec:&AdmissionSpec)->bool{
        self.basic()&&self.decision_id==spec.decision_id&&self.goal_id==spec.goal_id&&self.provider==spec.owner.provider
        &&self.conversation_id==spec.context.conversation_id&&self.model==spec.resolved_model
        &&self.policy_revision==spec.policy_revision&&self.claimed_at>=spec.approved_at&&self.deadline_at==spec.budget.deadline_at
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum StopReason{Pause,Cancel,Expired}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(super)enum DrainOutcome{Confirmed,Uncertain}
