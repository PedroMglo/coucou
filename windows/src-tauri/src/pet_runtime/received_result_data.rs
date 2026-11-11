//! Private source-dependency observation. Raw source memories never enter the
//! recipient preview, context, prompt, grant or log. This value is not authority.
use super::{authority::{Authority,Failure},messages::{Envelope,ReceivedReference},result_disclosure_policy::Policy as ResultPolicy};
use crate::pet_memory::{context::Capture,Scope};
use std::{path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}}};
/// A dependency fence for literal work received from a memory-derived source.
/// It never selects source text for the child's model context or grants model use.
pub(super)struct OriginDependencies{authority:PathBuf,dependencies:Dependencies}
impl OriginDependencies{
 pub(super)fn capture(authority:Option<&Path>,memory:&Path,goal:&super::model::Record,now:u64)->super::model::Result<Option<Arc<Self>>>{
  if !goal.spec.received_from.as_ref().is_some_and(|o|o.body.planned_dependencies.is_some()){return Ok(None);}
  let path=authority.ok_or(super::model::Error::Conflict)?;let a=Authority::existing(path).map_err(Failure::control_error)?;
  let dependencies=a.capture_planned_origin_dependencies(goal,memory,now).map_err(Failure::control_error)?;
  Ok(Some(Arc::new(Self{authority:path.into(),dependencies})))
 }
 pub(super)fn current(&self,now:u64)->bool{
  let Ok(a)=Authority::existing(&self.authority)else{return false;};a.planned_origin_dependencies_current(&self.dependencies,now).is_ok()
 }
}
#[derive(PartialEq,Eq)]enum Policy{Result(ResultPolicy),Planned(super::planned_peer_data::Published)}
pub(super)const MAX_SOURCE_DEPTH:usize=8;
/// Native observations of a selected received source. This is never parsed from
/// a prompt/DTO and does not recreate the source's consent or execution owner.
struct ReceivedDependency{funded:Option<(super::received_context::Selection,String)>,source:super::received_request::Source,envelope:Envelope,derived:Option<Box<Dependencies>>}
impl ReceivedDependency{
 fn current(&self,authority:&Authority,now:u64)->Result<(),Failure>{
  let current=authority.selected_peer_data_source_unlocked(&self.source,now)?;
  if current!=self.envelope{return Err(Failure::Changed);}
  if let Some((selection,goal))=&self.funded{authority.funded_context_current_unlocked(selection,goal,&current,now)?;authority.delegation_lineage_unlocked(&selection.source,Some(goal),now)?;}
  if let Some(d)=&self.derived{d.verify(authority,now)?;}Ok(())
 }
}
pub(super)struct Dependencies{policy:Policy,reference:ReceivedReference,envelope:Envelope,memory:Option<Capture>,received:Option<Box<ReceivedDependency>>,captured_at:u64,retired:AtomicBool}
impl Dependencies{
 pub(super)fn capture(authority:&Authority,memory_path:&Path,reference:&ReceivedReference,envelope:&Envelope,now:u64)->Result<Self,Failure>{
  Self::capture_inner(authority,memory_path,reference,envelope,now,0,&mut std::collections::BTreeSet::new())
 }
 fn capture_inner(authority:&Authority,memory_path:&Path,reference:&ReceivedReference,envelope:&Envelope,now:u64,depth:usize,seen:&mut std::collections::BTreeSet<(String,String)>)->Result<Self,Failure>{
  if depth>=MAX_SOURCE_DEPTH||!seen.insert((reference.decision_id.clone(),reference.message_id.clone())){return Err(Failure::Invalid);}
  let(policy,e)=if envelope.planned_data.is_some(){let(p,e)=authority.received_planned_data_unlocked(reference,&envelope.recipient.pet_id,now)?;(Policy::Planned(p),e)}else{let(p,e)=authority.received_result_policy_unlocked(reference,&envelope.recipient.pet_id,now)?;(Policy::Result(p),e)};if &e!=envelope{return Err(Failure::Changed);}
  let(references,pet,cid)=match &policy{Policy::Result(p)=>{let cid=p.memory_references.iter().find_map(|r|match &r.owner.scope{Scope::Conversation{conversation_id}=>Some(conversation_id.as_str()),_=>None}).unwrap_or(&envelope.origin.conversation_id);(&p.memory_references,&p.pet_id,cid)},Policy::Planned(p)=>(&p.source.references,&p.source.pet_id,p.source.conversation_id.as_str())};
  let memory=if references.is_empty(){None}else{Some(Capture::at_references(memory_path,pet,cid,references,now).map_err(|_|Failure::Changed)?)};
  let received=if let Policy::Planned(p)=&policy{if let Some(selection)=&p.source.received_request{
   // Snapshot4 is HMAC-bound to the exact source admission and native F2.
   // Reobserve the same receipt; never treat its serializable selection as
   // permission, silently select other sources or copy ancestor raw records.
   if p.schema_version!=4||selection.source.recipient_pet_id!=p.source.pet_id||depth+1>=MAX_SOURCE_DEPTH{return Err(Failure::Invalid);}
   let original=authority.selected_peer_data_source_unlocked(&selection.source,now)?;
   let shape=match selection.schema_version{1=>original.schema_version==1&&original.kind==super::messages::Kind::Request,2=>original.schema_version==4&&original.kind==super::messages::Kind::Delegation,4=>original.schema_version==7&&original.kind==super::messages::Kind::Delegation,7=>matches!(original.schema_version,12|14|16)&&original.kind==super::messages::Kind::Delegation&&original.planned_data.is_some(),3=>original.schema_version==6&&original.kind==super::messages::Kind::Reply,5=>original.schema_version==9&&original.kind==super::messages::Kind::Reply,6=>matches!(original.schema_version,10|11|13|15)&&original.kind==super::messages::Kind::Request&&original.planned_data.is_some(),_=>false};if !shape{return Err(Failure::Changed);}
   let derived=if matches!(selection.schema_version,3|5|6|7){Some(Box::new(Self::capture_inner(authority,memory_path,&selection.source.reference,&original,now,depth+1,seen)?))}else{if !seen.insert((selection.source.reference.decision_id.clone(),selection.source.reference.message_id.clone())){return Err(Failure::Invalid);}None};
   let funded=matches!(selection.schema_version,2|4|7).then(||(selection.as_ref().clone(),p.source.goal_id.clone()));
   Some(Box::new(ReceivedDependency{funded,source:selection.source.clone(),envelope:original.clone(),derived}))
  }else{None}}else{None};
  let d=Self{policy,reference:reference.clone(),envelope:envelope.clone(),memory,received,captured_at:now,retired:AtomicBool::new(false)};d.verify(authority,now)?;Ok(d)
 }
 pub(super)fn depth(&self)->usize{1+self.received.as_ref().map(|r|r.derived.as_ref().map(|d|d.depth()).unwrap_or(1)).unwrap_or(0)}
 pub(super)fn verify(&self,authority:&Authority,now:u64)->Result<(),Failure>{
  if self.retired.load(Ordering::Acquire){return Err(Failure::Revoked);}let result=(||{if now<self.captured_at{return Err(Failure::Expired);}if let Some(c)=&self.memory{c.verify(now).map_err(|_|Failure::Revoked)?;}let(p,e)=match &self.policy{Policy::Result(_)=>{let(p,e)=authority.received_result_policy_unlocked(&self.reference,&self.envelope.recipient.pet_id,now)?;(Policy::Result(p),e)},Policy::Planned(_)=>{let(p,e)=authority.received_planned_data_unlocked(&self.reference,&self.envelope.recipient.pet_id,now)?;(Policy::Planned(p),e)}};if p!=self.policy||e!=self.envelope{return Err(Failure::Changed);}if let Some(r)=&self.received{r.current(authority,now)?;}Ok(())})();if result.as_ref().err().is_some_and(|e|!matches!(e,Failure::Busy|Failure::Unavailable|Failure::Uncertain)){self.retired.store(true,Ordering::Release);}result
 }
}
