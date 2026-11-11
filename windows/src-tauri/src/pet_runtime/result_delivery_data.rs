//! Selected derived DATA for a finite successor. This is not a grant: admission
//! and effect writers separately verify the exact own signed Spec14/funding.
use super::{model::{self,Record},result_disclosure::{Observe,Preview},result_disclosure_policy::Policy,messages::Target};
use crate::pet_memory::context::Capture;
use std::{path::Path,sync::atomic::{AtomicBool,Ordering}};
pub(super)struct Captured{policy:Policy,authority:super::authority::Authority,memory:Option<Capture>,received:super::received_context::Captured,target:Target,observe:Observe,preview:Preview,prepared_at:u64,retired:AtomicBool}
impl Captured{
 pub(super)fn at(authority:&Path,memory:&Path,g:&Record,policy:Policy,observe:Observe,now:u64)->model::Result<Self>{
  g.validate()?;if !policy.matches(g)||now<g.updated_at||now>=policy.arguments.expires_at{return Err(model::Error::Conflict);}
  let a=super::authority::Authority::existing(authority).map_err(super::authority::Failure::control_error)?;let envelope=a.capture_derived_data(g,&policy,now).map_err(super::authority::Failure::control_error)?;
  let b=g.execution_context.as_ref().ok_or(model::Error::Conflict)?;let received=super::received_context::Captured::at(authority,g,b.received_request.as_ref().ok_or(model::Error::Conflict)?,now)?;
  let memory=if policy.memory_references.is_empty(){None}else{Some(Capture::at_references(memory,&b.pet_id,b.source_conversation_id.as_deref().unwrap_or(&b.conversation_id),&policy.memory_references,now).map_err(|_|model::Error::Conflict)?)};
  let(config,settings)=observe()?;let target=Target::capture(&config,&settings,&policy.arguments.to_pet_id)?;if target.recipient()!=&policy.recipient{return Err(model::Error::Conflict);}
  let expires_at=policy.memory_references.iter().filter_map(|r|r.expires_at).fold(policy.arguments.expires_at,u64::min);
  let preview=Preview{schema_version:policy.schema_version,goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),revision:policy.source_revision.clone(),intent_hash:policy.intent_hash.clone(),context_hash:policy.context_hash.clone(),proposal_hash:policy.proposal_hash.clone(),cause:policy.cause.clone(),arguments:policy.arguments.clone(),origin:policy.origin.clone(),recipient:policy.recipient.clone(),received_message:envelope,memory_references:policy.memory_references.clone(),dependency_policy:"allOriginalSelectedSourcesMustRemainCurrent",use_scope:"oneLocalReplyToDelegationSource",required_consent:"boundedResultDisclosure",provider_use_authorized:false,sharing_authorized:false,expires_at};
  let c=Self{policy,authority:a,memory,received,target,observe,preview,prepared_at:now,retired:AtomicBool::new(false)};c.current(g,now)?;Ok(c)
 }
 pub(super)fn policy(&self)->&Policy{&self.policy}
 pub(super)fn preview(&self)->&Preview{&self.preview}
 pub(super)fn current(&self,g:&Record,now:u64)->model::Result<()>{
  if self.retired.load(Ordering::Acquire){return Err(model::Error::Conflict);}
  let valid=(||{if now<self.prepared_at||now>=self.preview.expires_at||!self.policy.matches(g){return Err(model::Error::Conflict);}if let Some(m)=&self.memory{m.verify(now).map_err(|_|model::Error::Conflict)?;}self.received.verify(now)?;let(config,settings)=(self.observe)()?;if !self.target.current(&config,&settings){return Err(model::Error::Conflict);}
   // Same atomic-file/HMAC DATA observation as received_context::Witness. It
   // also runs inside the enclosing admission/message Authority writer.
   let e=self.authority.derived_data_unlocked(g,&self.policy,now).map_err(super::authority::Failure::control_error)?;if e!=self.preview.received_message{return Err(model::Error::Conflict);}Ok(())})();if valid.is_err(){self.retired.store(true,Ordering::Release);}valid
 }
 pub(super)fn same(&self,other:&Self)->bool{self.policy==other.policy}
}
