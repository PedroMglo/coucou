//! Authenticated observations of exact delegated children. DATA only: neither
//! a child result, a snapshot hash nor this DTO can authorize inference/effects.
use super::{model::{self,Record},authority::Authority,delegation::{Reservation,State as Escrow},delegation_api::DelegationInput,received_work,reconciliation,store::Store,Repository};
use serde::Serialize;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Status{NotAccepted,Withdrawn,Observed,Unavailable}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ModelAttempt{pub attempt_id:String,pub phase:&'static str,pub charged_output_bytes:u64,pub outcome_known:bool,pub result_evidence:Vec<reconciliation::Evidence>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Child{pub goal_id:String,pub pet_id:String,pub goal_revision:String,pub goal_hash:String,pub grant_hash:String,pub state:received_work::State,pub model_attempts:Vec<ModelAttempt>,pub request_within_deadline:bool,pub evidence_complete:bool,pub criteria_verified:bool,pub goal_complete:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Item{pub message_id:String,pub scope_hash:String,pub recipient_pet_id:String,pub status:Status,pub binding:Option<super::funding::ChildBinding>,pub child:Option<Child>,pub reason:Option<model::Error>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct View{pub schema_version:u32,pub scope:&'static str,pub pet_id:String,pub goal_id:String,pub decision_id:String,pub parent_goal_revision:String,pub parent_goal_hash:String,pub parent_grant_hash:String,pub observed_at:u64,pub items:Vec<Item>,pub evidence_complete:bool,pub execution_authorized:bool,pub model_use_authorized:bool}
fn reference(input:&DelegationInput,r:&Reservation)->super::messages::ReceivedReference{super::messages::ReceivedReference{decision_id:input.decision_id.clone(),message_id:r.message_id.clone(),scope_hash:r.scope_hash.clone()}}
fn observe(store:&Store,a:&Authority,parent:&Record,input:&DelegationInput,r:&Reservation,now:u64)->model::Result<Child>{
 let binding=r.binding().ok_or(model::Error::Conflict)?;let child=store.read(&binding.goal_id,&binding.pet_id)?;let row=child.admissions.last().ok_or(model::Error::Conflict)?;
 let source=child.spec.received_from.as_ref().ok_or(model::Error::Conflict)?;
 if !binding.matches(&row.spec)||source.body.source.reference!=reference(input,r)||source.body.source.sender_goal_id!=parent.id||source.body.source.sender_pet_id!=parent.spec.owner.pet_id||source.body.source.recipient_pet_id!=r.recipient_pet_id||child.spec.owner.pet_id!=r.recipient_pet_id{return Err(model::Error::Conflict);}
 let snapshot=a.reconciliation(&child,now).map_err(super::authority::Failure::control_error)?;let grant_hash=snapshot.grant.as_ref().ok_or(model::Error::Conflict)?.hash.clone();let goal_complete=snapshot.goal_complete;
 let model_attempts:Vec<ModelAttempt>=snapshot.attempts.iter().filter(|x|x.kind=="model").map(|x|ModelAttempt{attempt_id:x.id.clone(),phase:x.phase,charged_output_bytes:x.charged_output_bytes,outcome_known:x.outcome_known,result_evidence:x.evidence.iter().filter(|e|e.kind==reconciliation::EvidenceKind::ModelResult).cloned().collect()}).collect();
 let view=received_work::View::base(&child,now)?.observed(&child,snapshot)?;let complete=view.evidence_complete&&model_attempts.iter().all(|a|a.outcome_known);
 a.peer_feedback_child_fence(parent,&input.decision_id,r,&row.spec,&grant_hash,now).map_err(super::authority::Failure::control_error)?;
 let again=store.read(&child.id,&child.spec.owner.pet_id)?;if model::fingerprint(&again)?!=view.goal_hash{return Err(model::Error::Conflict);}
 Ok(Child{goal_id:child.id,pet_id:child.spec.owner.pet_id,goal_revision:child.revision,goal_hash:view.goal_hash,grant_hash,state:view.state,model_attempts,request_within_deadline:view.request_within_deadline,evidence_complete:complete,criteria_verified:view.criteria_verified&&complete,goal_complete:goal_complete&&complete})
}
pub(super)fn capture(goals:&std::path::Path,authority:&std::path::Path,input:&DelegationInput,now:u64)->model::Result<View>{
 if !model::key(&input.pet_id)||!model::goal_id(&input.goal_id)||!model::token(&input.decision_id){return Err(model::Error::InvalidInput);}
 let store=Store::existing_verified(goals,authority)?.ok_or(model::Error::NotFound)?;let parent=store.read(&input.goal_id,&input.pet_id)?;let a=Authority::existing(authority).map_err(super::authority::Failure::control_error)?;let before=a.delegation_view(&parent,&input.decision_id,now)?;
 if before.historical{return Err(model::Error::Conflict);}let parent_goal_hash=model::fingerprint(&parent)?;let mut items=vec![];
 for r in &before.reservations{
  let mut item=Item{message_id:r.message_id.clone(),scope_hash:r.scope_hash.clone(),recipient_pet_id:r.recipient_pet_id.clone(),status:Status::Unavailable,binding:r.binding().cloned(),child:None,reason:None};
  if r.binding().is_some(){match observe(&store,&a,&parent,input,r,now){Ok(child)=>{item.status=Status::Observed;item.child=Some(child);},Err(reason)=>item.reason=Some(reason)}}
  else{item.status=if matches!(r.state,Escrow::Withdrawn{..}){Status::Withdrawn}else{Status::NotAccepted};}items.push(item);
 }
 let after=store.read(&parent.id,&parent.spec.owner.pet_id)?;if model::fingerprint(&after)?!=parent_goal_hash||a.delegation_view(&after,&input.decision_id,now)?.grant_hash!=before.grant_hash{return Err(model::Error::Conflict);}
 let complete=items.iter().all(|i|i.status!=Status::Unavailable&&i.child.as_ref().is_none_or(|c|c.evidence_complete));
 Ok(View{schema_version:1,scope:"nativeUserExactDelegationOutcomeEvidence",pet_id:input.pet_id.clone(),goal_id:parent.id,decision_id:input.decision_id.clone(),parent_goal_revision:parent.revision,parent_goal_hash,parent_grant_hash:before.grant_hash,observed_at:now,items,evidence_complete:complete,execution_authorized:false,model_use_authorized:false})
}
