//! Exact native retirement of one unpublished local intent. It never sends a
//! message, recreates an Owned, refunds budget or cleans unrelated staging.
use super::{model::{self,Record},authority::Grant,messages::Scope};
use serde::{Serialize,Deserialize};
#[cfg(target_os="linux")]use super::{authority::Authority,store::Stamp};
use std::{collections::BTreeMap,sync::Arc,time::{Instant,Duration}};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Artifact{pub name:String,pub hash:String,pub stamp_hash:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{pub schema_version:u32,pub id:String,pub request_id:String,pub scope:Scope,pub source_grant_hash:String,pub files:Vec<Artifact>,pub prepared_at:u64,pub retired_at:u64}
impl Reference{
 pub(super)fn valid_for(&self,grant:&Grant)->bool{
  let e=&self.scope.envelope;let mut names=std::collections::BTreeSet::new();
  self.schema_version==2&&model::token(&self.id)&&model::key(&self.request_id)&&super::authority::decode(&self.source_grant_hash).is_some()&&self.scope.valid()
   &&self.scope.main_spec_hash==grant.spec_hash&&e.origin.goal_id==grant.goal_id&&e.origin.decision_id==grant.decision_id
   &&grant.lease.as_ref().is_some_and(|l|e.origin.lease_id==l.id&&e.origin.controller_id==l.controller_id&&e.origin.provider==l.provider&&e.origin.conversation_id==l.conversation_id)
   &&self.prepared_at>=e.created_at&&self.retired_at>=self.prepared_at&&self.retired_at<=grant.updated_at&&self.retired_at-self.prepared_at<120_000
   &&!self.files.is_empty()&&self.files.len()<=32&&self.files.iter().all(|f|f.name.strip_prefix(".grant-").is_some_and(model::token)&&super::authority::decode(&f.hash).is_some()&&super::authority::decode(&f.stamp_hash).is_some()&&names.insert(&f.name))
 }
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{pub goal_id:String,pub pet_id:String,pub decision_id:String,pub revision:String,pub message:super::messages::Envelope,pub artifacts:Vec<Artifact>,pub source_grant_hash:String,pub prepared_at:u64,pub expires_at:u64,pub authority_reduction_only:bool,pub budget_refund:bool,pub delivery_authorized:bool}
#[cfg(target_os="linux")]pub(super)struct Review{pub(super)authority:Authority,pub(super)goal:Record,pub(super)body:Grant,pub(super)scope:Scope,pub(super)files:Vec<Artifact>,pub(super)stamps:BTreeMap<String,Stamp>,pub(super)request:String,pub(super)id:String,pub(super)prepared_at:u64,pub(super)expires:Instant}
#[cfg(target_os="linux")]pub(super)struct Approved{pub(super)review:Review,pub(super)reference:Reference,pub(super)human:Arc<dyn Fn()->bool+Send+Sync>,pub(super)started:bool}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Receipt{pub reference:Reference,pub cleanup_complete:bool,pub message_delivered:bool,pub budget_refund:bool,pub execution_authorized:bool}
#[cfg(target_os="linux")]impl Review{
 pub(super)fn proposal(&self)->model::Result<Proposal>{Ok(Proposal{goal_id:self.goal.id.clone(),pet_id:self.goal.spec.owner.pet_id.clone(),decision_id:self.body.decision_id.clone(),revision:self.goal.revision.clone(),message:self.scope.envelope.clone(),artifacts:self.files.clone(),source_grant_hash:model::fingerprint(&self.body)?,prepared_at:self.prepared_at,expires_at:self.prepared_at+120_000,authority_reduction_only:true,budget_refund:false,delivery_authorized:false})}
 pub(super)fn approve(self,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Approved>{
  if now<self.prepared_at||now-self.prepared_at>=120_000||Instant::now()>=self.expires||!human(){return Err(model::Error::Conflict);}
  let reference=Reference{schema_version:2,id:self.id.clone(),request_id:self.request.clone(),scope:self.scope.clone(),source_grant_hash:model::fingerprint(&self.body)?,files:self.files.clone(),prepared_at:self.prepared_at,retired_at:now};
  Ok(Approved{review:self,reference,human,started:false})
 }
}
#[cfg(target_os="linux")]impl Approved{
 pub(super)fn confirm(&mut self,now:u64)->model::Result<Receipt>{
  self.review.authority.commit_message_retirement(&self.review.goal,&self.review.body,&self.review.stamps,&self.reference,now,||Instant::now()<self.review.expires&&(self.human)(),&mut self.started).map_err(super::authority::Failure::control_error)
 }
}
#[cfg(target_os="linux")]pub(super)fn deadline()->Instant{Instant::now()+Duration::from_secs(120)}
