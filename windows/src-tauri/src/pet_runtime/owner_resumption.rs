//! Signed native owner handoff in the original admission/lease. These records
//! preserve provenance; only the retained native review can acquire an owner.
use super::{authority,leases::Lease,model::{self,Record,State},runtime_owner};
use serde::{Deserialize,Serialize};
pub(super)const MAX_RESUMPTIONS:usize=32;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
 pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,pub decision_id:String,pub lease_id:String,
 pub source_revision:String,pub source_sequence:u64,pub source_goal_hash:String,pub source_grant_revision:String,pub source_grant_hash:String,
 pub target_revision:String,pub target_sequence:u64,pub prepared_at:u64,pub approved_at:u64,
}
impl Reference{
 pub(super)fn valid(&self)->bool{
  self.schema_version==1&&model::token(&self.id)&&model::key(&self.request_id)&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)
   &&[&self.decision_id,&self.lease_id,&self.source_revision,&self.source_grant_revision,&self.target_revision].iter().all(|s|model::token(s))
   &&[&self.source_goal_hash,&self.source_grant_hash].iter().all(|s|authority::decode(s).is_some())
   &&self.target_revision!=self.source_revision&&self.source_sequence>0&&self.source_sequence<model::MAX_EVENTS as u64&&self.target_sequence==self.source_sequence+1&&self.target_sequence<=model::MAX_EVENTS as u64
   &&self.prepared_at>0&&self.approved_at>=self.prepared_at&&self.approved_at-self.prepared_at<=30_000&&self.approved_at<=8_640_000_000_000_000
 }
 pub(super)fn source_matches(&self,g:&Record)->bool{self.valid()&&g.state==State::RecoveryRequired&&g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.revision==self.source_revision&&g.sequence==self.source_sequence&&model::fingerprint(g).ok().as_ref()==Some(&self.source_goal_hash)}
 pub(super)fn present(&self,g:&Record)->bool{self.valid()&&g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.events.get((self.target_sequence-1)as usize).is_some_and(|e|matches!(e.kind,model::EventKind::OwnerResumed)&&e.request_id==self.request_id&&e.at==self.approved_at&&model::fingerprint(self).ok().as_ref()==Some(&e.fingerprint))}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Seal{
 pub schema_version:u32,pub reference:Reference,pub target_goal_hash:String,
 pub closure:super::recovery::Closure,pub previous_owner:runtime_owner::Record,pub owner:runtime_owner::Record,
 pub previous_closed_at:u64,pub previous_updated_at:u64,pub previous_resource_usage:Option<super::resource_wait::Ledger>,pub usage_hash:String,
 pub capability_revision:String,pub policy_revision:String,
}
impl Seal{
 pub(super)fn valid(&self,lease:&Lease,goal:&str,decision:&str)->bool{
  let r=&self.reference;self.schema_version==1&&r.valid()&&r.goal_id==goal&&r.decision_id==decision&&r.lease_id==lease.id
   &&!self.capability_revision.is_empty()&&self.capability_revision.len()<=256&&self.policy_revision==lease.policy_revision
   &&self.previous_owner.valid_for(lease)&&self.owner.valid_for(lease)&&self.owner.profile==self.previous_owner.profile&&self.owner.process!=self.previous_owner.process
   &&self.owner.captured_at>=r.approved_at&&self.owner.captured_at<lease.deadline_at&&self.previous_closed_at>=lease.claimed_at&&self.previous_closed_at<=self.previous_updated_at&&self.previous_updated_at<=r.approved_at&&r.approved_at<lease.deadline_at
   &&self.closure.schema_version==1&&self.closure.reference.valid()&&self.closure.reference.goal_id==goal&&self.closure.reference.decision_id==decision&&self.closure.reference.lease_id==lease.id
   &&self.closure.reference.target_state==State::RecoveryRequired&&self.closure.reference.target_goal_revision==r.source_revision&&self.closure.reference.target_sequence==r.source_sequence
   &&self.closure.target_goal_hash==r.source_goal_hash&&model::fingerprint(&self.previous_owner).ok().as_ref()==Some(&self.closure.owner_hash)
   &&self.previous_resource_usage.as_ref().is_none_or(|u|u.valid(lease,self.previous_updated_at))
   &&[&self.target_goal_hash,&self.usage_hash,&self.closure.closed_base_hash,&self.closure.usage_hash,&self.closure.evidence_hash].iter().all(|s|authority::decode(s).is_some())
 }
 pub(super)fn usage_hash(g:&authority::Grant)->model::Result<String>{model::fingerprint(&(&g.budget_usage,&g.planning_usage,&g.delegation_pool,&g.delegation_reservations,&g.messages))}
}

/// Read-only proof for a canonical receipt across authenticated owner epochs.
/// This reconstructed historical record must never be used by an effect writer
/// or to create an Owned/DATA approval. Every reversed byte transition is hashed
/// against the signed closure/handoff, rather than normalizing arbitrary edits.
pub(super)fn receipt_scope_matches(goal:&Record,body:&authority::Grant,scope:&super::messages::Scope)->bool{
 if scope.matches(goal){return true;}
 let result=(||->model::Result<bool>{
  let mut historical=goal.clone();
  for seal in body.owner_resumptions.iter().rev(){
   let r=&seal.reference;let c=&seal.closure.reference;
   if !r.present(&historical)||historical.sequence!=r.target_sequence||historical.revision!=r.target_revision||model::fingerprint(&historical)?!=seal.target_goal_hash
    ||c.unclaimed||c.target_state!=State::RecoveryRequired{return Err(model::Error::Conflict);}
   historical.events.pop();historical.sequence=r.source_sequence;historical.revision=r.source_revision.clone();historical.updated_at=historical.events.last().ok_or(model::Error::Conflict)?.at;
   historical.state=State::RecoveryRequired;historical.admissions.last_mut().ok_or(model::Error::Conflict)?.phase=super::admission::QueuePhase::RecoveryRequired;historical.refresh_schema();historical.validate()?;
   if model::fingerprint(&historical)?!=r.source_goal_hash||model::fingerprint(&historical)?!=seal.closure.target_goal_hash{return Err(model::Error::Conflict);}
   let event=historical.events.last().ok_or(model::Error::Conflict)?;
   if !matches!(event.kind,model::EventKind::LostOwnerClosed)||event.request_id!=c.request_id||event.at!=c.approved_at||event.fingerprint!=model::fingerprint(c)?{return Err(model::Error::Conflict);}
   historical.events.pop();historical.sequence=c.source_sequence;historical.revision=c.source_goal_revision.clone();historical.updated_at=historical.events.last().ok_or(model::Error::Conflict)?.at;
   historical.state=State::Leased;let row=historical.admissions.last_mut().ok_or(model::Error::Conflict)?;row.phase=super::admission::QueuePhase::Leased;row.stop_reason=None;historical.refresh_schema();historical.validate()?;
   if model::fingerprint(&historical)?!=c.source_goal_hash{return Err(model::Error::Conflict);}
   if scope.matches(&historical){return Ok(true);}
  }Ok(false)
 })();result.unwrap_or(false)
}
