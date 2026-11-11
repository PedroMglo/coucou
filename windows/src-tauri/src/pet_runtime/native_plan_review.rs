//! Owned native human review of plan DATA. Execution needs a later independent
//! continuation consent; no model, worker, resource hold or budget is issued.
use super::{model::{self,Record},authority::{Authority,Failure},continuation::{self,NativeCarry},native_plan_revisions::{self as revisions,Input,Reference,Seal},plans,store::Store};
use serde::Serialize;
use std::{sync::Arc,time::{Duration,Instant}};
pub(super)struct Review{result_data:Option<Arc<super::result_disclosure::Captured>>,pub(super)authority:Authority,pub(super)goal:Record,pub(super)source:continuation::Reference,pub(super)reference:Reference,expires:Instant}
pub(super)struct Approved{pub(super)review:Review,pub(super)human:Arc<dyn Fn()->bool+Send+Sync>,pub(super)approved_at:u64,pub(super)seal:Option<Seal>}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{#[serde(skip_serializing_if="Option::is_none")]pub result_data:Option<super::result_disclosure::Preview>,pub goal_id:String,pub pet_id:String,pub conversation_id:String,pub revision:String,pub intent:String,pub criteria:Vec<String>,pub before:plans::Snapshot,pub after:plans::Snapshot,pub changes:Reference,#[serde(skip_serializing_if="Option::is_none")]pub coverage:Option<NativeCarry>,pub history_hash:String,pub prepared_at:u64,pub expires_at:u64,pub execution_authorized:bool,pub budget_reserved:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Receipt{pub receipt:model::Receipt,pub reference:Reference,pub acknowledged:bool,pub execution_authorized:bool,pub new_admission_created:bool}
impl Review{
 pub(super)fn capture(goal:Record,captured:continuation::Captured,inputs:Vec<Input>,request:String,now:u64)->model::Result<Self>{
  let(source,authority)=captured.into_parts();let reference=if matches!(source.schema_version,6|9){revisions::prepare_result(&goal,&source,inputs,request,now)?}else{revisions::prepare(&goal,source.native.as_ref().ok_or(model::Error::Conflict)?,inputs,request,now)?};
  authority.native_plan_preflight(&goal,&source,&reference,now).map_err(Failure::control_error)?;
  let r=Self{result_data:None,authority,goal,source,reference,expires:Instant::now()+Duration::from_secs(120)};plans::encode_input(&r.proposal()?)?;Ok(r)
 }
 pub(super)fn with_result_data(mut self,authority:&std::path::Path,memory:&std::path::Path,observe:super::result_disclosure::Observe,now:u64)->model::Result<Self>{
  if !matches!(self.reference.schema_version,2|3){return Ok(self);}let patch=self.reference.patches.first().filter(|_|self.reference.patches.len()==1).ok_or(model::Error::Conflict)?;
  let args=super::derived_result_plan::Arguments::parse(patch.after.operation.as_ref().ok_or(model::Error::Conflict)?)?;
  self.result_data=Some(Arc::new(super::result_disclosure::Captured::at(authority,memory,&self.goal,patch.cause.clone(),args,self.reference.target_proposal_hash.clone(),observe,now)?));plans::encode_input(&self.proposal()?)?;Ok(self)
 }
 pub(super)fn proposal(&self)->model::Result<Proposal>{let before=plans::snapshot(&self.goal);let mut after=before.clone();self.reference.apply_snapshot(&mut after,false)?;let mut result_data=self.result_data.as_ref().map(|d|d.preview().clone());if let Some(d)=&mut result_data{d.expires_at=d.expires_at.min(self.reference.reviewed_at.saturating_add(120_000));}
  Ok(Proposal{result_data,goal_id:self.goal.id.clone(),pet_id:self.goal.spec.owner.pet_id.clone(),conversation_id:model::cid(&self.goal),revision:self.goal.revision.clone(),intent:self.goal.spec.intent.clone(),criteria:self.goal.spec.criteria.clone(),before,after,changes:self.reference.clone(),coverage:self.source.native.clone(),history_hash:self.source.history_hash.clone(),prepared_at:self.reference.reviewed_at,expires_at:self.result_data.as_ref().map(|d|d.preview().expires_at).unwrap_or(u64::MAX).min(self.reference.reviewed_at.saturating_add(120_000)),execution_authorized:false,budget_reserved:false})
 }
 pub(super)fn approve(self,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Approved>{
  if matches!(self.reference.schema_version,2|3)&&self.result_data.is_none(){return Err(model::Error::Conflict);}
  if let Some(d)=&self.result_data{d.current(&self.goal,now)?;}
  if now<self.reference.reviewed_at||now-self.reference.reviewed_at>=120_000||Instant::now()>=self.expires||!human(){return Err(model::Error::Conflict);}
  Ok(Approved{review:self,human,approved_at:now,seal:None})
 }
}
impl Approved{
 pub(super)fn human_current(&self)->bool{Instant::now()<self.review.expires&&(self.human)()}
 pub(super)fn confirm(&mut self,store:&Store,now:u64)->model::Result<Receipt>{
  let human=self.human.clone();let expires=self.review.expires;let lower=self.approved_at;let deadline=self.review.reference.reviewed_at.saturating_add(120_000);let data=self.review.result_data.clone();let goal=self.review.goal.clone();let current:Arc<dyn Fn()->bool+Send+Sync>=Arc::new(move||{let wall=crate::assistant_memory::now();Instant::now()<expires&&wall>=lower&&wall<deadline&&human()&&data.as_ref().is_none_or(|d|d.current_unlocked(&goal,wall).is_ok())});
  let cert=store.certify_native_plan(&self.review.authority,&self.review.goal,&self.review.source,&self.review.reference,self.approved_at,&mut self.seal,now,current)?;
  let commit=store.commit_plan(&cert,now)?;let acknowledged=if let Some(p)=commit.published.as_ref(){self.review.authority.acknowledge_native_plan(p).map_err(Failure::control_error)?;true}else{false};
  Ok(Receipt{receipt:commit.receipt,reference:self.review.reference.clone(),acknowledged,execution_authorized:false,new_admission_created:false})
 }
}
