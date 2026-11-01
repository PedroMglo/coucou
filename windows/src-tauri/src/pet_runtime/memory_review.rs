//! Native data review and goal-specific data confirmation. This module captures
//! records and their exact destination; execution still requires the existing
//! signed admission, budget, context fence and runtime resource ownership.
//! No repository write or inference is started here.
use super::model::{self,ContextBinding,Record};
use crate::{local_chat::Server,pet_memory::{context::Capture,context_model::Destination}};
pub(super)use crate::pet_memory::context_model::{GoalConsent as DataConsent,GoalPreview as Preview};
use std::{path::Path,sync::Arc};

type Result<T>=std::result::Result<T,&'static str>;
const REVIEW_TTL:u64=120_000;

/// Neither Clone nor Deserialize: references, JSON and hashes cannot recreate a
/// native captured review, a data confirmation or any execution authority.
pub(super)struct Captured{
 capture:Arc<Capture>,binding:ContextBinding,spec_hash:String,
 preview:Preview,credential:Option<String>,prepared_at:u64,expires_at:u64,
}
pub(super)struct ConfirmedData{captured:Captured,confirmed_at:u64}

pub(super)fn destination_for(provider:&str,server:&Server,model:&str)->Result<Destination>{
 if !matches!(provider,"ollama"|"lmstudio")||server.id!=provider||!super::admission::model_id(model){return Err("goalMemoryDestinationChanged");}
 let base=crate::local_chat::base_url(server).map_err(|_|"goalMemoryDestinationUnavailable")?;
 if !crate::net::is_loopback_url(&base){return Err("goalMemoryDestinationUnavailable");}
 let destination=Destination::Server{provider:server.id.into(),endpoint:crate::net::join(&base,"v1/chat/completions"),model:model.into()};
 if !destination.valid(){return Err("goalMemoryDestinationUnavailable");}Ok(destination)
}
fn destination(goal:&Record,server:&Server,model:&str)->Result<Destination>{
 if !goal.spec.owner.local_engine()||server.id!=goal.spec.owner.provider||!super::admission::model_id(model){return Err("goalMemoryDestinationChanged");}
 destination_for(&goal.spec.owner.provider,server,model)
}
impl Captured{
 pub(super)fn native(goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  Self::at(&Capture::catalog_path(),goal,server,model,now)
 }
 pub(super)fn at(path:&Path,goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  goal.validate().map_err(|_|"goalMemoryContextChanged")?;
  let binding=goal.execution_context.as_ref().ok_or("goalMemoryContextRequired")?;
  let Some(selection)=&binding.memory else{return Ok(None);};
  if !binding.valid(goal)||!selection.valid(&goal.spec.owner.pet_id,binding.source_conversation_id.as_deref()){return Err("goalMemoryContextChanged");}
  let destination=destination(goal,server,model)?;
  let source=binding.source_conversation_id.as_deref().unwrap_or(&binding.conversation_id);
  let capture=Arc::new(Capture::at_references(path,&binding.pet_id,source,&selection.references,now)?);
  // Enforce the actual serialized goal envelope's bound before showing review.
  capture.goal_message(&goal.id,binding.source_conversation_id.as_deref(),now)?;
  let expires_at=selection.references.iter().filter_map(|r|r.expires_at).fold(now.checked_add(REVIEW_TTL).ok_or("goalMemoryReviewLimit")?,u64::min);
  let preview=Preview{schema_version:1,goal_id:goal.id.clone(),pet_id:binding.pet_id.clone(),source_conversation_id:binding.source_conversation_id.clone(),destination,records:capture.records(now)?,required_consent:DataConsent::BoundedGoalContext,max_input_bytes:crate::pet_memory::context_model::GOAL_INPUT_LIMIT};
  Ok(Some(Self{capture,binding:binding.clone(),spec_hash:model::fingerprint(&goal.spec).map_err(|_|"goalMemoryContextChanged")?,preview,credential:server.key.clone(),prepared_at:now,expires_at}))
 }
 pub(super)fn preview(&self)->&Preview{&self.preview}
 pub(super)fn grant(&self)->crate::pet_memory::context_model::GoalGrant{crate::pet_memory::context_model::GoalGrant{schema_version:1,consent:DataConsent::BoundedGoalContext,destination:self.preview.destination.clone(),max_input_bytes:self.preview.max_input_bytes}}
 pub(super)fn message_data(&self,goal:&Record,now:u64)->Result<serde_json::Value>{self.current(goal,now)?;self.capture.goal_message(&goal.id,self.binding.source_conversation_id.as_deref(),now)}
 pub(super)fn expires_at(&self)->u64{self.expires_at}
 pub(super)fn current(&self,goal:&Record,now:u64)->Result<()>{
  goal.validate().map_err(|_|"goalMemoryContextChanged")?;
  if now<self.prepared_at||goal.id!=self.preview.goal_id||goal.execution_context.as_ref()!=Some(&self.binding)||model::fingerprint(&goal.spec).map_err(|_|"goalMemoryContextChanged")?!=self.spec_hash{return Err("goalMemoryContextChanged");}
  self.capture.verify(now)
 }
 pub(super)fn same_data(&self,other:&Self)->bool{self.preview==other.preview&&self.binding==other.binding&&self.spec_hash==other.spec_hash&&self.credential==other.credential}
 fn data_current(&self,goal:&Record,server:&Server,model:&str,now:u64)->Result<()>{
  self.current(goal,now)?;
  if destination(goal,server,model)?!=self.preview.destination||server.key!=self.credential{return Err("goalMemoryDestinationChanged");}Ok(())
 }
 pub(super)fn confirmation_current(&self,goal:&Record,server:&Server,model:&str,now:u64)->Result<()>{
  if now>=self.expires_at{return Err("goalMemoryReviewExpired");}self.data_current(goal,server,model,now)
 }
 pub(super)fn confirm(self,goal:&Record,server:&Server,model:&str,consent:Option<DataConsent>,now:u64)->Result<ConfirmedData>{
  if consent!=Some(DataConsent::BoundedGoalContext){return Err("goalMemoryConsentRequired");}
  self.confirmation_current(goal,server,model,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
 /// Only a fresh native observation can compare the private destination/key.
 /// The original object is consumed; the original review TTL is not renewed.
 pub(super)fn confirm_against(self,goal:&Record,latest:&Self,consent:Option<DataConsent>,now:u64)->Result<ConfirmedData>{
  if consent!=Some(DataConsent::BoundedGoalContext){return Err("goalMemoryConsentRequired");}
  if now>=self.expires_at||!self.same_data(latest){return Err("goalMemoryReviewChanged");}
  self.current(goal,now)?;latest.current(goal,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
}
impl ConfirmedData{
 pub(super)fn current(&self,goal:&Record,now:u64)->Result<()>{if now<self.confirmed_at{return Err("goalMemoryContextChanged");}self.captured.current(goal,now)}
 /// Data only. The writer must separately verify the original admission,
 /// budget, context fence and resource ownership before hashing/sending this.
 pub(super)fn message(&self,goal:&Record,server:&Server,model:&str,now:u64)->Result<serde_json::Value>{
  if now<self.confirmed_at{return Err("goalMemoryContextChanged");}
  self.captured.data_current(goal,server,model,now)?;self.captured.capture.goal_message(&goal.id,self.captured.binding.source_conversation_id.as_deref(),now)
 }
}
