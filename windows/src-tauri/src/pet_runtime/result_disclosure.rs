//! Exact native DATA review for a derived result. This capture is neither a
//! disclosure grant nor an execution permit; no model or recipient is invoked.
use super::{model::{self,Record},derived_result_plan::Arguments,messages::{Cause,Target,Recipient,Envelope},results::ModelResult};
pub(super)use super::result_disclosure_policy::{Origin,Policy};
use crate::{pet_memory::{context::Capture,context_model::{Reference,Destination}},pet_config::Configuration,settings::Settings};
use serde::Serialize;
use std::{path::Path,sync::{Arc,atomic::{AtomicBool,Ordering}}};
pub(super)type Observe=Arc<dyn Fn()->model::Result<(Configuration,Settings)>+Send+Sync>;
#[derive(Clone,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub struct Preview{pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub revision:String,pub intent_hash:String,pub context_hash:String,pub proposal_hash:String,pub cause:Cause,pub arguments:Arguments,pub origin:Origin,pub recipient:Recipient,pub received_message:Envelope,pub memory_references:Vec<Reference>,pub dependency_policy:&'static str,pub use_scope:&'static str,pub required_consent:&'static str,pub provider_use_authorized:bool,pub sharing_authorized:bool,pub expires_at:u64}
/// Own original source captures. Re-enabling an entry/pet cannot revive this
/// particular review after any observed invalidation. No serde/Clone for it.
pub(super)struct Captured{authority:super::authority::Authority,target:Target,memory:Option<Capture>,received:super::received_context::Captured,observe:Observe,preview:Preview,prepared_at:u64,retired:AtomicBool}
impl Captured{
 pub(super)fn at(authority:&Path,memory:&Path,g:&Record,cause:Cause,args:Arguments,proposal_hash:String,observe:Observe,now:u64)->model::Result<Self>{
  g.validate()?;let context=g.execution_context.as_ref().filter(|c|matches!(c.schema_version,4|6)&&c.valid(g)).ok_or(model::Error::Conflict)?;
  if now<g.updated_at||now>=args.expires_at||super::authority::decode(&proposal_hash).is_none()||!g.plans.iter().any(|p|p.id==cause.plan_id&&p.task_id==cause.task_id&&p.steps.iter().any(|s|s.id==cause.step_id)){return Err(model::Error::Conflict);}
  let a=super::authority::Authority::existing(authority).map_err(super::authority::Failure::control_error)?;
  let(receipt,received_message)=a.capture_result_disclosure(g,&args,now).map_err(super::authority::Failure::control_error)?;
  let row=g.admissions.iter().find(|r|r.spec.decision_id==receipt.decision_id).ok_or(model::Error::Conflict)?;
  let inference_destination=row.spec.memory_data.as_ref().ok_or(model::Error::Conflict)?.destination.clone();
  let selected=context.received_request.as_ref().ok_or(model::Error::Conflict)?;let received=super::received_context::Captured::at(authority,g,selected,now)?;
  let memory=match &context.memory{Some(s)=>Some(Capture::at_references(memory,&context.pet_id,context.source_conversation_id.as_deref().unwrap_or(&context.conversation_id),&s.references,now).map_err(|_|model::Error::Conflict)?),None=>None};
  let(config,settings)=observe()?;let target=Target::capture(&config,&settings,&args.to_pet_id)?;
  let origin=Origin{provider:receipt.provider,adapter:row.spec.owner.adapter.clone(),model:receipt.model,inference_destination,input_hash:receipt.input_hash,intent_hash:receipt.intent_hash,completed_at:receipt.completed_at,recorded_at:receipt.recorded_at};
  let references=memory.as_ref().map(|m|m.references().to_vec()).unwrap_or_default();let expires_at=references.iter().filter_map(|r|r.expires_at).fold(now.saturating_add(120_000).min(args.expires_at),u64::min);
  let preview=Preview{schema_version:if context.schema_version==6{2}else{1},goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),revision:g.revision.clone(),intent_hash:model::fingerprint(&g.spec)?,context_hash:model::fingerprint(context)?,proposal_hash,cause,arguments:args,origin,recipient:target.recipient().clone(),received_message,memory_references:references,dependency_policy:"allOriginalSelectedSourcesMustRemainCurrent",use_scope:"oneLocalReplyToDelegationSource",required_consent:"boundedResultDisclosure",provider_use_authorized:false,sharing_authorized:false,expires_at};
  let c=Self{authority:a,target,memory,received,observe,preview,prepared_at:now,retired:AtomicBool::new(false)};c.current(g,now)?;Ok(c)
 }
 pub(super)fn policy(&self,g:&Record)->model::Result<Policy>{let p=&self.preview;let source=g.spec.received_from.as_ref().ok_or(model::Error::Conflict)?.body.source.clone();let policy=Policy{schema_version:p.schema_version,consent:super::disclosure_consent::Consent::BoundedResultDisclosure,goal_id:p.goal_id.clone(),pet_id:p.pet_id.clone(),source_revision:p.revision.clone(),intent_hash:p.intent_hash.clone(),context_hash:p.context_hash.clone(),proposal_hash:p.proposal_hash.clone(),cause:p.cause.clone(),arguments:p.arguments.clone(),origin:p.origin.clone(),recipient:p.recipient.clone(),memory_references:p.memory_references.clone(),source};if !policy.matches(g){return Err(model::Error::Conflict);}Ok(policy)}
 pub(super)fn preview(&self)->&Preview{&self.preview}
 pub(super)fn current(&self,g:&Record,now:u64)->model::Result<()>{self.check(g,now,false)}
 /// Final native-plan writer already owns the exact Authority flock. Do not
 /// acquire it recursively. Same canonical HMAC/source fences still apply.
 pub(super)fn current_unlocked(&self,g:&Record,now:u64)->model::Result<()>{self.check(g,now,true)}
 fn check(&self,g:&Record,now:u64,unlocked:bool)->model::Result<()>{
  if self.retired.load(Ordering::Acquire){return Err(model::Error::Conflict);}
  let result=(||{let p=&self.preview;
   if now<self.prepared_at||now>=p.expires_at||g.id!=p.goal_id||g.spec.owner.pet_id!=p.pet_id||g.revision!=p.revision||model::fingerprint(&g.spec)?!=p.intent_hash||g.execution_context.as_ref().map(model::fingerprint).transpose()?.as_ref()!=Some(&p.context_hash){return Err(model::Error::Conflict);}
   if let Some(m)=&self.memory{m.verify(now).map_err(|_|model::Error::Conflict)?;}self.received.verify(now)?;
   let(config,settings)=(self.observe)()?;if !self.target.current(&config,&settings){return Err(model::Error::Conflict);}
   let(r,e)=if unlocked{self.authority.result_disclosure_unlocked(g,&p.arguments,now)}else{self.authority.capture_result_disclosure(g,&p.arguments,now)}.map_err(super::authority::Failure::control_error)?;
   if !self.receipt_matches(&r)||e!=p.received_message{return Err(model::Error::Conflict);}Ok(())
  })();if result.is_err(){self.retired.store(true,Ordering::Release);}result
 }
 fn receipt_matches(&self,r:&ModelResult)->bool{let o=&self.preview.origin;r.provider==o.provider&&r.model==o.model&&r.input_hash==o.input_hash&&r.intent_hash==o.intent_hash&&r.completed_at==o.completed_at&&r.recorded_at==o.recorded_at}
}
