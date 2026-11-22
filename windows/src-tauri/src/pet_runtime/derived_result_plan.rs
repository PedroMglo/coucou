//! A reviewed plan DATA reference, never consent to disclose the result.
//! The explicit derived disclosure admission and ordinary native message
//! controller execute it; a proposal, F2 or parser result alone grants nothing.
use super::{model,plans::OperationProposal,messages::ReceivedReference};
#[cfg(target_os="linux")]use super::{authority::{Authority,Failure},results};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ResultReference{pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub decision_id:String,pub request_id:String,pub attempt_id:String,pub receipt_hash:String,pub byte_start:u64,pub byte_end:u64,pub total_bytes:u64}
impl ResultReference{
 pub(super)fn valid(&self)->bool{self.schema_version==1&&model::goal_id(&self.goal_id)&&crate::pet_config::blueprint(&self.pet_id).is_some()&&model::token(&self.decision_id)&&model::key(&self.request_id)&&model::token(&self.attempt_id)&&super::authority::decode(&self.receipt_hash).is_some()&&self.byte_start<self.byte_end&&self.byte_end<=self.total_bytes&&self.total_bytes<=4*1024*1024&&self.byte_end-self.byte_start<=super::messages::MAX_BODY as u64}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Arguments{pub to_pet_id:String,pub source:ReceivedReference,pub result:ResultReference,pub text:String,pub expires_at:u64}
impl Arguments{
 pub(super)fn parse(op:&OperationProposal)->model::Result<Self>{
  if op.operation_id!="pet.resultReply"||op.version!=1{return Err(model::Error::InvalidInput);}
  Self::parse_arguments(&op.arguments)
 }
 pub(crate)fn parse_arguments(v:&serde_json::Value)->model::Result<Self>{
  let a:Self=serde_json::from_value(v.clone()).map_err(|_|model::Error::InvalidInput)?;
  if !a.valid(){return Err(model::Error::InvalidInput);}Ok(a)
 }
 pub(super)fn valid(&self)->bool{self.result.valid()&&self.source.valid()&&crate::pet_config::blueprint(&self.to_pet_id).is_some()&&self.to_pet_id!=self.result.pet_id&&super::messages::body(&self.text)&&self.text.len()as u64==self.result.byte_end-self.result.byte_start&&self.expires_at>0&&self.expires_at<=8_640_000_000_000_000}
 /// The enclosing Authority writer/reader is already held. Lookup authenticates
 /// both the canonical attempt and the signed receipt without nested flock.
 #[cfg(target_os="linux")]
 pub(super)fn verify_unlocked(&self,a:&Authority,g:&model::Record)->super::authority::Result<()>{
  let origin=g.spec.received_from.as_ref().filter(|o|o.body.delegation.is_some()).ok_or(Failure::Changed)?;
  if !self.valid()||self.result.goal_id!=g.id||self.result.pet_id!=g.spec.owner.pet_id||self.source!=origin.body.source.reference||self.to_pet_id!=origin.body.source.sender_pet_id||self.expires_at>origin.body.source.deadline_at{return Err(Failure::Changed);}
  let row=g.admissions.iter().find(|r|r.spec.decision_id==self.result.decision_id&&r.spec.funded_response()).ok_or(Failure::Changed)?;
  let results::Lookup::Available{receipt}=results::lookup_unlocked(a,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,&self.result.request_id)?else{return Err(Failure::RecoveryRequired);};
  if receipt.goal_id!=g.id||receipt.pet_id!=g.spec.owner.pet_id||receipt.attempt_id!=self.result.attempt_id||model::fingerprint(&receipt).ok().as_ref()!=Some(&self.result.receipt_hash)||receipt.text.len()as u64!=self.result.total_bytes||receipt.text.get(self.result.byte_start as usize..self.result.byte_end as usize)!=Some(self.text.as_str()){return Err(Failure::Changed);}Ok(())
 }
}
