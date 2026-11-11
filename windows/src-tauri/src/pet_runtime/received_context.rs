//! Selected peer request DATA for R43. A selection/preview/owned capture is not
//! an admission, provider permission, budget or a substitute for GoalConsent.
use super::{model,received_request::Source,messages::Envelope};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Selection{pub schema_version:u32,pub own_origin_hash:String,pub source:Source}
impl Selection{
 pub(in crate::pet_runtime)fn from_goal(goal:&model::Record)->model::Result<Self>{
  goal.validate()?;let origin=goal.spec.received_from.as_ref().ok_or(model::Error::InvalidInput)?;
  let selected=Self{schema_version:if origin.body.schema_version==7{7}else if origin.body.schema_version==6{6}else if origin.body.schema_version==5{5}else if origin.body.schema_version==4{4}else if origin.body.schema_version==3{3}else if origin.body.delegation.is_some(){2}else{1},own_origin_hash:model::fingerprint(origin)?,source:origin.body.source.clone()};
  if !selected.valid_for(goal){return Err(model::Error::InvalidInput);}Ok(selected)
 }
 pub(crate)fn structurally_valid(&self)->bool{matches!(self.schema_version,1|2|3|4|5|6|7)&&self.source.valid()&&super::authority::decode(&self.own_origin_hash).is_some()}
 pub(crate)fn valid_for(&self,goal:&model::Record)->bool{
  self.structurally_valid()&&goal.spec.received_from.as_ref().is_some_and(|origin|self.schema_version==(if origin.body.schema_version==7{7}else if origin.body.schema_version==6{6}else if origin.body.schema_version==5{5}else if origin.body.schema_version==4{4}else if origin.body.schema_version==3{3}else if origin.body.delegation.is_some(){2}else{1})&&origin.valid_for(&goal.spec)&&origin.body.source==self.source&&model::fingerprint(origin).ok().as_ref()==Some(&self.own_origin_hash))
 }
}
#[derive(Clone,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Preview{pub selection:Selection,pub received_message:Envelope,pub model_message:serde_json::Value}
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)struct Captured{witness:super::authority_store::received_context::Witness}
#[cfg(target_os="linux")]
impl Captured{
 pub(in crate::pet_runtime)fn at(authority:&std::path::Path,goal:&model::Record,selection:&Selection,now:u64)->model::Result<Self>{
  Ok(Self{witness:super::authority_store::received_context::Witness::capture(authority,goal,selection,now).map_err(super::authority::Failure::control_error)?})
 }
 pub(in crate::pet_runtime)fn at_with_memory(authority:&std::path::Path,memory:&std::path::Path,goal:&model::Record,selection:&Selection,now:u64)->model::Result<Self>{Ok(Self{witness:super::authority_store::received_context::Witness::capture_with_memory(authority,memory,goal,selection,now).map_err(super::authority::Failure::control_error)?})}
 pub(in crate::pet_runtime)fn verify(&self,now:u64)->model::Result<()>{self.witness.verify(now).map_err(super::authority::Failure::control_error)}
 pub(in crate::pet_runtime)fn source_depth(&self)->usize{self.witness.source_depth()}
 pub(in crate::pet_runtime)fn selection(&self)->&Selection{self.witness.selection()}
 pub(in crate::pet_runtime)fn deadline_at(&self)->u64{self.selection().source.deadline_at}
 pub(in crate::pet_runtime)fn preview(&self,goal_id:&str,now:u64)->model::Result<Preview>{Ok(Preview{selection:self.selection().clone(),received_message:self.witness.envelope().clone(),model_message:self.message(goal_id,now)?})}
 pub(in crate::pet_runtime)fn message(&self,goal_id:&str,now:u64)->model::Result<serde_json::Value>{
  self.verify(now)?;if goal_id!=self.witness.goal_id(){return Err(model::Error::Conflict);}let e=self.witness.envelope();let source=&self.selection().source;
  // Only selected minimal DATA is materialized. Do not send lease/controller,
  // account/session credentials or histories; peer text never grants authority.
  let derived_funded=self.selection().schema_version==7;let planned=matches!(self.selection().schema_version,6|7);let recursive=self.selection().schema_version==4;let funded=matches!(self.selection().schema_version,2|4|7);let derived=matches!(self.selection().schema_version,3|5);let recursive_result=self.selection().schema_version==5;
  let mut body=serde_json::json!({"schemaVersion":if derived_funded{7}else if planned{6}else if recursive_result{5}else if recursive{4}else if derived{3}else if funded{2}else{1},"kind":if derived_funded{"receivedDerivedDelegationData"}else if planned{"receivedPlannedPeerMemoryData"}else if recursive_result{"receivedRecursiveDerivedResultData"}else if recursive{"receivedRecursiveDelegationData"}else if derived{"receivedDerivedResultData"}else if funded{"receivedDelegationData"}else{"receivedRequestData"},"goalId":goal_id,"source":{
   "reference":source.reference,"receiptHash":source.receipt_hash,"envelopeHash":source.envelope_hash,"senderPetId":source.sender_pet_id,"recipientPetId":source.recipient_pet_id,"receivedAt":source.received_at,"deadlineAt":source.deadline_at},
   "derivedResult":e.derived_result,"text":e.text,"untrustedData":true,"authorizesActions":false,"sourceProviderUseAuthorized":e.provider_use_authorized});
  if planned{body["plannedPeerData"]=e.planned_data.as_ref().ok_or(model::Error::Conflict)?.model_metadata();}
  if !derived{body.as_object_mut().ok_or(model::Error::InvalidRecord)?.remove("derivedResult");}
  let message=serde_json::json!({"role":"user","content":serde_json::to_string(&body).map_err(|_|model::Error::InvalidRecord)?});
  if serde_json::to_vec(&message).map_err(|_|model::Error::InvalidRecord)?.len()>crate::pet_memory::context_model::GOAL_INPUT_LIMIT{return Err(model::Error::LimitReached);}Ok(message)
 }
}
