//! Signed closure references are durable facts, never a new execution grant.
//! Native reviews/absence handles live in recovery_controller and cannot be
//! reconstructed from these serialized values without authenticating the grant.
use super::{model::{self,Record,State},authority,leases::Lease};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
    pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,pub decision_id:String,pub lease_id:String,
    pub source_goal_revision:String,pub source_goal_hash:String,pub source_sequence:u64,
    pub target_goal_revision:String,pub target_sequence:u64,pub target_state:State,
    pub source_grant_revision:String,pub source_grant_hash:String,pub prepared_at:u64,pub approved_at:u64,
    pub unclaimed:bool,
}
impl Reference{
    pub(super)fn valid(&self)->bool{
        self.schema_version==1&&model::token(&self.id)&&model::key(&self.request_id)&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)&&model::token(&self.decision_id)&&model::token(&self.lease_id)
            &&[&self.source_goal_revision,&self.target_goal_revision,&self.source_grant_revision].iter().all(|s|model::token(s))
            &&[&self.source_goal_hash,&self.source_grant_hash].iter().all(|s|authority::decode(s).is_some())&&self.source_sequence>0&&self.source_sequence<=model::MAX_EVENTS as u64
            &&self.prepared_at>0&&self.approved_at>=self.prepared_at&&self.approved_at<=8_640_000_000_000_000&&self.approved_at-self.prepared_at<=30_000
            &&match self.target_state{State::Completed=>self.target_goal_revision==self.source_goal_revision&&self.target_sequence==self.source_sequence,State::RecoveryRequired|State::Cancelled=>self.target_goal_revision!=self.source_goal_revision&&self.target_sequence==self.source_sequence+1,_=>false}
    }
    pub(super)fn source_matches(&self,goal:&Record)->bool{self.valid()&&goal.id==self.goal_id&&goal.spec.owner.pet_id==self.pet_id&&goal.revision==self.source_goal_revision&&goal.sequence==self.source_sequence&&model::fingerprint(goal).ok().as_ref()==Some(&self.source_goal_hash)}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Closure{pub schema_version:u32,pub reference:Reference,pub target_goal_hash:String,pub closed_base_hash:String,pub usage_hash:String,pub evidence_hash:String,pub owner_hash:String}
impl Closure{
    #[cfg(target_os="linux")]pub(super)fn valid_for(&self,body:&authority::Grant)->bool{
        let r=&self.reference;
        self.schema_version==1&&r.valid()&&r.goal_id==body.goal_id&&r.decision_id==body.decision_id&&body.lease.as_ref().is_some_and(|l|l.id==r.lease_id)
            &&body.runtime_owner.as_ref().is_some_and(|o|model::fingerprint(o).ok().as_ref()==Some(&self.owner_hash))
            &&[&self.target_goal_hash,&self.closed_base_hash,&self.usage_hash,&self.evidence_hash,&self.owner_hash].iter().all(|s|authority::decode(s).is_some())
            &&body.closed_at.is_some_and(|n|n<=r.approved_at)&&body.updated_at>=r.approved_at&&matches!(body.phase,authority::GrantPhase::Closed|authority::GrantPhase::Revoked)
    }
}
pub(super)fn event(goal:&Record,_reference:&Reference)->model::EventKind{
    if goal.admissions.last().is_some_and(|r|r.lease.is_none()){model::EventKind::UnclaimedOwnerClosed}else{model::EventKind::LostOwnerClosed}
}
pub(super)fn target(goal:&Record,cancelled:bool)->State{if goal.state==State::Completed{State::Completed}else if cancelled||goal.state==State::Cancelled{State::Cancelled}else{State::RecoveryRequired}}
pub(super)fn lease_matches(reference:&Reference,lease:&Lease)->bool{reference.goal_id==lease.goal_id&&reference.decision_id==lease.decision_id&&reference.lease_id==lease.id}
