//! Explicit creation of future DATA from a restored source. This certificate
//! is never a grant, balance, lease, provider consent or worker-death proof.
use super::model::{self,Spec};
use serde::{Serialize,Deserialize};
pub(super)const TTL:u64=120_000;
fn hash(v:&str)->bool{v.len()==64&&v.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Source{pub pet_id:String,pub goal_id:String,pub revision:String,pub goal_hash:String,pub history_hash:String,pub source_fingerprint:String,pub snapshot_hash:String,pub historical_related_grants:usize,pub unattributed_historical_grants:usize}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Body{pub schema_version:u32,pub purpose:String,pub key_id:String,pub review_id:String,pub client_request_id:String,pub prepared_at:u64,pub approved_at:u64,pub expires_at:u64,pub source:Source,pub target_goal_id:String,pub base_spec_hash:String,pub unknown_state_accepted:bool,pub execution_authority_granted:bool,pub accounting_known:bool,pub owner_absence_proven:bool}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Origin{pub body:Body,pub tag:String}
pub(super)fn base_hash(spec:&Spec)->model::Result<String>{let mut base=spec.clone();base.restored_from=None;model::fingerprint(&base)}
pub(super)fn request_id(pet:&str,source:&str,revision:&str,request:&str)->model::Result<String>{if !model::key(pet)||!model::goal_id(source)||!model::token(revision)||!model::key(request){return Err(model::Error::InvalidInput);}Ok(format!("rf-{}",model::fingerprint(&("coucou-restored-future-request-v1",pet,source,revision,request))?))}
impl Origin{
 pub(super)fn valid_for(&self,spec:&Spec)->bool{let b=&self.body;let s=&b.source;
  b.schema_version==1&&b.purpose=="restoredFutureDraftDataOnly"&&model::token(&b.key_id)&&model::token(&b.review_id)&&model::key(&b.client_request_id)&&hash(&self.tag)
   &&b.prepared_at>0&&b.prepared_at<=b.approved_at&&b.approved_at<b.expires_at&&b.expires_at<=8_640_000_000_000_000&&b.expires_at-b.prepared_at<=TTL
   &&model::key(&s.pet_id)&&model::goal_id(&s.goal_id)&&model::token(&s.revision)&&[&s.goal_hash,&s.history_hash,&s.source_fingerprint,&s.snapshot_hash,&b.base_spec_hash].iter().all(|v|hash(v))
   &&s.pet_id==spec.owner.pet_id&&s.goal_id!=b.target_goal_id&&b.target_goal_id==model::id_for(spec)&&s.historical_related_grants<=4096&&s.unattributed_historical_grants<=4096
   &&b.unknown_state_accepted&&!b.execution_authority_granted&&!b.accounting_known&&!b.owner_absence_proven&&matches!(spec.context,model::Context::Unbound)&&base_hash(spec).ok().as_ref()==Some(&b.base_spec_hash)&&request_id(&s.pet_id,&s.goal_id,&s.revision,&b.client_request_id).ok().as_ref()==Some(&spec.request_id)
 }
}
pub(super)fn message(body:&Body)->model::Result<Vec<u8>>{let mut bytes=b"coucou-restored-future-draft-data-v1\0".to_vec();bytes.extend(serde_json::to_vec(body).map_err(|_|model::Error::InvalidRecord)?);Ok(bytes)}

#[cfg(target_os="linux")]
pub(super)struct Review{witness:super::authority_store::restored_future::Witness,base:Spec,proposal:Proposal,signed:Option<Spec>,ended:bool,acknowledged:bool,client_request_id:String}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{pub schema_version:u32,pub purpose:&'static str,pub review_id:String,pub client_request_id:String,pub source:Source,pub source_owner:model::Owner,pub source_intent:String,pub source_criteria:Vec<String>,pub future:Spec,pub target_goal_id:String,pub prepared_at:u64,pub expires_at:u64,pub context_copied:bool,pub historical_budget_reused:bool,pub execution_authority_granted:bool,pub post_snapshot_state_unknown:bool,pub owner_absence_proven:bool}
#[cfg(target_os="linux")]
pub(super)struct Approved<'a>{pub(super)spec:&'a Spec,pub(super)witness:&'a super::authority_store::restored_future::Witness,pub(super)prepared_at:u64,pub(super)expires_at:u64}
#[cfg(target_os="linux")]
impl Review{
 pub(super)fn prepare(store:&super::store::Store,authority:&std::path::Path,source_id:&str,pet:&str,revision:&str,base:Spec,client_request_id:String,now:u64)->model::Result<Self>{
  base.validate()?;if base.restored_from.is_some()||base.owner.pet_id!=pet||!matches!(base.context,model::Context::Unbound)||base.request_id!=request_id(pet,source_id,revision,&client_request_id)?{return Err(model::Error::InvalidInput);}
  let original=store.read(source_id,pet)?;if original.revision!=revision{return Err(model::Error::Conflict);}let witness=super::authority_store::restored_future::Witness::capture(authority,&original,now).map_err(|e|e.control_error())?;
  witness.target_current(&base).map_err(|e|e.control_error())?;let expires_at=now.checked_add(TTL).filter(|n|*n<=8_640_000_000_000_000).ok_or(model::Error::ClockRollback)?;let target_goal_id=model::id_for(&base);if target_goal_id==source_id{return Err(model::Error::Conflict);}
  let proposal=Proposal{schema_version:1,purpose:"reviewRestoredFutureDraftDataOnly",review_id:witness.review_id().into(),client_request_id:client_request_id.clone(),source:witness.source().clone(),source_owner:original.spec.owner.clone(),source_intent:original.spec.intent,source_criteria:original.spec.criteria,future:base.clone(),target_goal_id,prepared_at:now,expires_at,context_copied:false,historical_budget_reused:false,execution_authority_granted:false,post_snapshot_state_unknown:true,owner_absence_proven:false};
  Ok(Self{witness,base,proposal,signed:None,ended:false,acknowledged:false,client_request_id})
 }
 pub(super)fn proposal(&self)->Proposal{self.proposal.clone()}
 pub(super)fn retained(&self,now:u64)->bool{!self.ended&&!self.acknowledged&&now<self.proposal.expires_at}
 pub(super)fn decide(&mut self,store:&super::store::Store,approve:bool,acknowledge_unknown:bool,now:u64)->model::Result<bool>{
  if self.ended{return Err(model::Error::Conflict);}if !approve{if self.signed.is_some(){return Err(model::Error::Conflict);}self.ended=true;return Ok(false);}
  if !acknowledge_unknown{return Err(model::Error::InvalidInput);}if self.signed.is_none(){
   if now<self.proposal.prepared_at||now>=self.proposal.expires_at{return Err(model::Error::Conflict);}let source=&self.proposal.source;let current=store.read(&source.goal_id,&source.pet_id)?;self.witness.current(&current).map_err(|e|e.control_error())?;
   let mut spec=self.base.clone();spec.restored_from=Some(self.witness.seal(&spec,&self.client_request_id,self.proposal.prepared_at,self.proposal.expires_at,now).map_err(|e|e.control_error())?);spec.validate()?;self.signed=Some(spec);
  }Ok(true)
 }
 pub(super)fn approved(&self)->model::Result<Approved<'_>>{if self.ended{return Err(model::Error::Conflict);}Ok(Approved{spec:self.signed.as_ref().ok_or(model::Error::Conflict)?,witness:&self.witness,prepared_at:self.proposal.prepared_at,expires_at:self.proposal.expires_at})}
 pub(super)fn acknowledge(&mut self){self.acknowledged=true;}
}
