//! Durable native worker closure is a fact, never a restored controller.
use super::{model::{self,Record,State},leases::{Lease,StopReason}};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
 pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,pub decision_id:String,pub lease_id:String,
 pub source_revision:String,pub source_sequence:u64,pub source_goal_hash:String,pub target_revision:String,pub target_sequence:u64,pub target_state:State,
 pub confirmed:bool,pub reason:Option<StopReason>,pub observed_at:u64,
}
impl Reference{
 pub(super)fn resulting_state(confirmed:bool,reason:Option<StopReason>)->State{if confirmed{match reason{Some(StopReason::Cancel)=>State::Cancelled,Some(StopReason::Pause)=>State::Paused,_=>State::RecoveryRequired}}else{State::RecoveryRequired}}
 pub(super)fn valid(&self)->bool{self.schema_version==1&&[&self.id,&self.decision_id,&self.lease_id,&self.source_revision,&self.target_revision].iter().all(|s|model::token(s))&&model::key(&self.request_id)&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)&&super::authority::decode(&self.source_goal_hash).is_some()&&self.source_sequence>0&&self.source_sequence<model::MAX_EVENTS as u64&&self.target_sequence==self.source_sequence+1&&self.target_revision!=self.source_revision&&self.target_state==Self::resulting_state(self.confirmed,self.reason)&&self.observed_at>0&&self.observed_at<=8_640_000_000_000_000}
 pub(super)fn source_matches(&self,g:&Record)->bool{self.valid()&&g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.revision==self.source_revision&&g.sequence==self.source_sequence&&model::fingerprint(g).ok().as_ref()==Some(&self.source_goal_hash)}
 pub(super)fn belongs(&self,g:&Record)->bool{g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.admissions.iter().any(|r|r.spec.deterministic()&&r.spec.decision_id==self.decision_id&&r.lease.as_ref().is_some_and(|l|l.id==self.lease_id))}
 pub(super)fn present(&self,g:&Record)->bool{self.valid()&&self.belongs(g)&&g.native_drains.contains(self)&&g.events.get((self.target_sequence-1)as usize).is_some_and(|e|e.at==self.observed_at&&e.request_id==self.request_id&&matches!(e.kind,model::EventKind::NativeDrained)&&model::fingerprint(self).ok().as_ref()==Some(&e.fingerprint))}
 pub(super)fn continuable(&self)->bool{self.valid()&&self.confirmed&&self.reason!=Some(StopReason::Cancel)&&matches!(self.target_state,State::Paused|State::RecoveryRequired)}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Superseded{pub reference:Reference,pub source_grant_hash:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Seal{pub superseded_preparations:Vec<Superseded>,pub target_grant_revision:String,pub schema_version:u32,pub reference:Reference,pub target_goal_hash:String,pub source_grant_revision:String,pub source_grant_hash:String,pub usage_hash:String,pub owner_hash:String}
impl Seal{
 pub(super)fn valid_for(&self,b:&super::authority::Grant)->bool{
  let r=&self.reference;self.schema_version==1&&model::token(&self.target_grant_revision)&&self.superseded_preparations.len()<=32&&self.superseded_preparations.iter().all(|s|s.reference.valid()&&s.reference.goal_id==r.goal_id&&s.reference.pet_id==r.pet_id&&s.reference.decision_id==r.decision_id&&s.reference.lease_id==r.lease_id&&s.reference.request_id==r.request_id&&s.reference.observed_at<=r.observed_at&&super::authority::decode(&s.source_grant_hash).is_some())&&r.valid()&&b.operation==super::authority::Operation::GoalDeterministic&&r.goal_id==b.goal_id&&r.decision_id==b.decision_id&&b.lease.as_ref().is_some_and(|l|l.id==r.lease_id&&r.observed_at>=l.claimed_at)&&b.runtime_owner.as_ref().is_some_and(|o|model::fingerprint(o).ok().as_ref()==Some(&self.owner_hash))&&model::token(&self.source_grant_revision)&&[&self.target_goal_hash,&self.source_grant_hash,&self.usage_hash,&self.owner_hash].iter().all(|h|super::authority::decode(h).is_some())&&b.closed_at.is_some_and(|t|t==r.observed_at)&&b.updated_at>=r.observed_at&&matches!(b.phase,super::authority::GrantPhase::Closed|super::authority::GrantPhase::Revoked)
 }
 pub(super)fn target_present(&self,g:&Record)->bool{self.reference.present(g)&&(g.sequence!=self.reference.target_sequence||g.revision==self.reference.target_revision&&model::fingerprint(g).ok().as_ref()==Some(&self.target_goal_hash))}
 #[cfg(target_os="linux")]pub(super)fn prepare(g:&Record,b:&super::authority::Grant,drain:&super::lease_controller::DrainProof,now:u64)->model::Result<Self>{
  let row=g.admissions.last().ok_or(model::Error::Conflict)?;let l=drain.lease();
  if !row.spec.deterministic()||row.lease.as_ref()!=Some(l)||b.lease.as_ref()!=Some(l)||b.stop_reason!=row.stop_reason||!matches!(g.state,State::Leased|State::Stopping)||!matches!(b.phase,super::authority::GrantPhase::Claimed|super::authority::GrantPhase::Draining)||drain.unstarted()||now<g.updated_at||now<b.updated_at{return Err(model::Error::Conflict);}
  let reference=Reference{schema_version:1,id:super::store::nonce()?,request_id:drain.request_id().into(),goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),decision_id:l.decision_id.clone(),lease_id:l.id.clone(),source_revision:g.revision.clone(),source_sequence:g.sequence,source_goal_hash:model::fingerprint(g)?,target_revision:super::store::nonce()?,target_sequence:g.sequence+1,target_state:Reference::resulting_state(drain.confirmed(),row.stop_reason),confirmed:drain.confirmed(),reason:row.stop_reason,observed_at:now};
  let mut target=g.clone();target.drained_native(&reference)?;
  Ok(Self{superseded_preparations:vec![],target_grant_revision:super::store::nonce()?,schema_version:1,reference,target_goal_hash:model::fingerprint(&target)?,source_grant_revision:b.revision.clone(),source_grant_hash:model::fingerprint(b)?,usage_hash:model::fingerprint(&(&b.budget_usage,&b.planning_usage,&b.resource_usage))?,owner_hash:model::fingerprint(b.runtime_owner.as_ref().ok_or(model::Error::Conflict)?)?})
 }
 pub(super)fn covers(&self,l:&Lease,request:&str,confirmed:bool)->bool{self.reference.lease_id==l.id&&self.reference.decision_id==l.decision_id&&self.reference.goal_id==l.goal_id&&self.reference.request_id==request&&self.reference.confirmed==confirmed}
}
