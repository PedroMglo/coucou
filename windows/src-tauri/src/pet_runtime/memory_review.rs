//! Native data review and goal-specific data confirmation. This module captures
//! records and their exact destination; execution still requires the existing
//! signed admission, budget, context fence and runtime resource ownership.
//! No repository write or inference is started here.
use super::model::{self,ContextBinding,Owner,Record};
use crate::{local_chat::Server,pet_memory::{context::Capture,context_model::Destination}};
pub(super)use crate::pet_memory::context_model::{GoalConsent as DataConsent,GoalPreview as Preview};
use std::{path::Path,sync::Arc};

type Result<T>=std::result::Result<T,&'static str>;
const REVIEW_TTL:u64=120_000;

/// Neither Clone nor Deserialize: references, JSON and hashes cannot recreate a
/// native captured review, a data confirmation or any execution authority.
pub(super)struct Captured{
 progress:Option<Arc<super::project_goal_observation::Captured>>,calendars:Option<Arc<crate::pet_calendar::goal::Captured>>,capture:Option<Arc<Capture>>,received:Option<Arc<super::received_context::Captured>>,binding:ContextBinding,spec_hash:String,
 preview:Preview,credential:Option<String>,prepared_at:u64,expires_at:u64,
}
pub(super)struct ConfirmedData{captured:Captured,confirmed_at:u64}

pub(super)fn destination_for(owner:&Owner,server:&Server,model:&str)->Result<Destination>{
 if !owner.local_engine()||server.id!=owner.adapter||!super::admission::model_id(model){return Err("goalMemoryDestinationChanged");}
 let base=crate::local_chat::base_url(server).map_err(|_|"goalMemoryDestinationUnavailable")?;
 if !crate::net::is_loopback_url(&base){return Err("goalMemoryDestinationUnavailable");}
 let destination=Destination::Server{provider:server.id.into(),endpoint:crate::net::join(&base,crate::local_chat::runtime_path(server)),model:model.into()};
 if !destination.valid(){return Err("goalMemoryDestinationUnavailable");}Ok(destination)
}
fn destination(goal:&Record,server:&Server,model:&str)->Result<Destination>{
 destination_for(&goal.spec.owner,server,model)
}
/// Resolve from the very same native projection used for the descriptor and
/// policy. A present binding never falls through to a global endpoint/key.
pub(super)fn server_for_goal(goal:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,model:&str)->Result<Server>{server_for_owner(&goal.spec.owner,config,settings,model)}
pub(super)fn server_for_owner(owner:&Owner,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,model:&str)->Result<Server>{
 if !owner.local_engine(){return Err("goalMemoryDestinationChanged");}
 let target=crate::pet_inference::Target::configured(config,&owner.pet_id).map_err(|_|"goalMemoryDestinationChanged")?;
 let server=if let Some(target)=target{
  if target.provider!=owner.provider||target.choice.backend.key()!=owner.adapter||target.choice.model!=model{return Err("goalMemoryDestinationChanged");}
  crate::local_chat::server_for_target(&target).map_err(|_|"goalMemoryDestinationUnavailable")?
 }else{crate::local_chat::server(settings,&owner.provider).ok_or("goalMemoryDestinationUnavailable")?};
 destination_for(owner,&server,model)?;Ok(server)
}
impl Captured{
 pub(super)fn native(goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  Self::at_with_authority(&Capture::catalog_path(),Some(&super::admission_review::authority_root()),goal,server,model,now)
 }
 pub(super)fn at(path:&Path,goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  Self::at_with_authority(path,None,goal,server,model,now)
 }
 pub(super)fn at_with_authority(path:&Path,authority:Option<&Path>,goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  Self::at_with_sources(path,authority,None,goal,server,model,now)
 }
 pub(super)fn at_with_sources(path:&Path,authority:Option<&Path>,goals:Option<&Path>,goal:&Record,server:&Server,model:&str,now:u64)->Result<Option<Self>>{
  goal.validate().map_err(|_|"goalMemoryContextChanged")?;
  let binding=goal.execution_context.as_ref().ok_or("goalMemoryContextRequired")?;
  if !binding.has_selected_data(){return Ok(None);}
  if !binding.valid(goal){return Err("goalMemoryContextChanged");}let destination=destination(goal,server,model)?;
  let source=binding.source_conversation_id.as_deref().unwrap_or(&binding.conversation_id);
  let capture=match &binding.memory{Some(selection)=>Some(Arc::new((if selection.project.is_some(){Capture::at_goal_selection(path,&binding.pet_id,binding.source_conversation_id.as_deref(),selection,now)?}else{Capture::at_references(path,&binding.pet_id,source,&selection.references,now)?}))),None=>None};
  let received=match &binding.received_request{Some(selection)=>Some(Arc::new(super::received_context::Captured::at_with_memory(authority.ok_or("goalReceivedAuthorityRequired")?,path,goal,selection,now).map_err(|_|"goalReceivedSourceChanged")?)),None=>None};
  let calendars=binding.calendars.as_ref().map(|c|crate::pet_calendar::goal::Captured::reopen(goal,&c.selection,c.data.clone(),Arc::new(||true)).map(Arc::new).map_err(|_|"goalCalendarSourceChanged")).transpose()?;
  let progress=super::context::progress_at(binding,goals,authority,path,now).map_err(|_|"goalProjectProgressChanged")?;
  let message=Self::compose(capture.as_deref(),received.as_deref(),calendars.as_deref(),progress.as_deref(),goal,now)?;
  let mut expires_at=now.checked_add(REVIEW_TTL).ok_or("goalMemoryReviewLimit")?;
  if let Some(selection)=&binding.memory{expires_at=selection.references.iter().filter_map(|r|r.expires_at).fold(expires_at,u64::min);}
  if let Some(data)=&received{expires_at=expires_at.min(data.deadline_at());}
  let received_preview=received.as_ref().map(|data|data.preview(&goal.id,now).map_err(|_|"goalReceivedSourceChanged")).transpose()?;
  let preview=Preview{project:binding.project().cloned().map(Box::new),calendar_selection:binding.calendars.as_ref().map(|c|c.selection.clone()),schema_version:if binding.has_project_progress(){14}else if binding.project_memory(){12}else if binding.pet_calendars(){10}else if calendars.is_some(){9}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==7){8}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==6){7}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==5){6}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==4){5}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==3){4}else if binding.received_request.as_ref().is_some_and(|r|r.schema_version==2){3}else if received.is_some(){2}else{1},received_request:received_preview,composed_message:if binding.has_project_progress()||binding.project_memory()||received.is_some()||calendars.is_some(){Some(message)}else{None},goal_id:goal.id.clone(),pet_id:binding.pet_id.clone(),source_conversation_id:binding.source_conversation_id.clone(),destination,records:match &capture{Some(c)=>c.records(now)?,None=>vec![]},required_consent:if binding.has_project_progress(){DataConsent::BoundedRuleProjectGoalProgressData}else{DataConsent::BoundedGoalContext},max_input_bytes:crate::pet_memory::context_model::GOAL_INPUT_LIMIT};
  Ok(Some(Self{progress,calendars,capture,received,binding:binding.clone(),spec_hash:model::fingerprint(&goal.spec).map_err(|_|"goalMemoryContextChanged")?,preview,credential:server.key.clone(),prepared_at:now,expires_at}))
 }
 fn compose(notes:Option<&Capture>,received:Option<&super::received_context::Captured>,calendars:Option<&crate::pet_calendar::goal::Captured>,progress:Option<&super::project_goal_observation::Captured>,goal:&Record,now:u64)->Result<serde_json::Value>{
  let memory=notes.map(|c|c.goal_selection_message(&goal.id,goal.execution_context.as_ref().and_then(|b|b.source_conversation_id.as_deref()),goal.execution_context.as_ref().and_then(|b|b.memory.as_ref()).ok_or("memorySelectionChanged")?,now)).transpose()?;
  let message=received.map(|c|c.message(&goal.id,now).map_err(|_|"goalReceivedSourceChanged")).transpose()?;
  let existing=super::goal_data_message::compose(&goal.id,memory,message).map_err(|_|"goalMemoryReviewLimit")?;
  if let Some(p)=progress{p.current(now).map_err(|_|"goalProjectProgressChanged")?;}
  let existing=super::goal_data_message::with_calendars(&goal.id,existing,calendars.map(|c|c.data().cloned()).transpose()?).map_err(|_|"goalMemoryReviewLimit")?;
  super::goal_data_message::with_project_progress(&goal.id,existing,progress.map(|p|&p.snapshot)).map_err(|_|"goalMemoryReviewLimit")?.ok_or("goalMemoryContextRequired")
 }

 pub(super)fn peer_forwarding_supported(&self)->bool{!self.binding.has_project_progress()&&!self.binding.project_memory()&&self.received.as_ref().is_none_or(|r|r.source_depth()<super::received_result_data::MAX_SOURCE_DEPTH)}
 pub(super)fn preview(&self)->&Preview{&self.preview}
 pub(super)fn grant(&self)->crate::pet_memory::context_model::GoalGrant{crate::pet_memory::context_model::GoalGrant{calendar_selection:self.binding.calendars.as_ref().map(|c|c.selection.clone()),received_request:self.binding.received_request.clone(),schema_version:if self.preview.required_consent==DataConsent::BoundedRuleProjectGoalProgressData{13}else if self.preview.required_consent==DataConsent::BoundedRulePinnedProjectData{12}else if self.binding.project_memory(){1}else{self.preview.schema_version},consent:self.preview.required_consent,destination:self.preview.destination.clone(),max_input_bytes:self.preview.max_input_bytes}}
 /// DATA framing for the already reviewed rule scope, not another human choice.
 /// Common Authority still authenticates its original read/parent/partition.
 pub(in crate::pet_runtime)fn into_rule_calendar(mut self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<Self>{
  self.current(goal,now)?;
  if !reference.calendar_context(&self.binding)||self.capture.is_some()||self.received.is_some()||self.calendars.is_none(){return Err("ruleCalendarContextRequired");}
  self.preview.schema_version=11;self.preview.required_consent=DataConsent::BoundedRuleFutureCalendarData;Ok(self)
 }
 pub(in crate::pet_runtime)fn confirm_rule_calendar(self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<ConfirmedData>{
  if self.preview.schema_version!=11||self.preview.required_consent!=DataConsent::BoundedRuleFutureCalendarData||!reference.calendar_context(&self.binding){return Err("ruleOwnCalendarConsentRequired");}
  self.current(goal,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
 pub(in crate::pet_runtime)fn into_rule_project(mut self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<Self>{
  self.current(goal,now)?;if !reference.project_context(&self.binding)||self.capture.is_none()||self.received.is_some()||self.calendars.is_some(){return Err("ruleProjectContextRequired");}
  self.preview.schema_version=13;self.preview.required_consent=DataConsent::BoundedRulePinnedProjectData;Ok(self)
 }
 pub(in crate::pet_runtime)fn confirm_rule_project(self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<ConfirmedData>{
  if self.preview.schema_version!=13||self.preview.required_consent!=DataConsent::BoundedRulePinnedProjectData||!reference.project_context(&self.binding){return Err("ruleOwnProjectConsentRequired");}
  self.current(goal,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
 pub(in crate::pet_runtime)fn into_rule_project_progress(mut self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<Self>{
  self.current(goal,now)?;if !reference.progress_context(&self.binding)||self.progress.is_none()||self.received.is_some()||self.calendars.is_some(){return Err("ruleProjectProgressContextRequired");}
  self.preview.schema_version=14;self.preview.required_consent=DataConsent::BoundedRuleProjectGoalProgressData;Ok(self)
 }
 pub(in crate::pet_runtime)fn confirm_rule_project_progress(self,goal:&Record,reference:&super::rules::OccurrenceReference,now:u64)->Result<ConfirmedData>{
  if self.preview.schema_version!=14||self.preview.required_consent!=DataConsent::BoundedRuleProjectGoalProgressData||!reference.progress_context(&self.binding){return Err("ruleOwnProjectProgressConsentRequired");}
  self.current(goal,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
 pub(super)fn message_data(&self,goal:&Record,now:u64)->Result<serde_json::Value>{self.current(goal,now)?;Self::compose(self.capture.as_deref(),self.received.as_deref(),self.calendars.as_deref(),self.progress.as_deref(),goal,now)}
 pub(super)fn expires_at(&self)->u64{self.expires_at}
 pub(super)fn current(&self,goal:&Record,now:u64)->Result<()>{
  goal.validate().map_err(|_|"goalMemoryContextChanged")?;
  if now<self.prepared_at||goal.id!=self.preview.goal_id||goal.execution_context.as_ref()!=Some(&self.binding)||model::fingerprint(&goal.spec).map_err(|_|"goalMemoryContextChanged")?!=self.spec_hash{return Err("goalMemoryContextChanged");}
  if let Some(p)=&self.progress{p.current(now).map_err(|_|"goalProjectProgressChanged")?;}if let Some(c)=&self.calendars{c.current()?;}if let Some(c)=&self.capture{c.verify(now)?;}if let Some(c)=&self.received{c.verify(now).map_err(|_|"goalReceivedSourceChanged")?;}Ok(())
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
  if consent!=Some(DataConsent::BoundedGoalContext)||self.preview.required_consent!=DataConsent::BoundedGoalContext{return Err("goalMemoryConsentRequired");}
  self.confirmation_current(goal,server,model,now)?;Ok(ConfirmedData{captured:self,confirmed_at:now})
 }
 /// Only a fresh native observation can compare the private destination/key.
 /// The original object is consumed; the original review TTL is not renewed.
 pub(super)fn confirm_against(self,goal:&Record,latest:&Self,consent:Option<DataConsent>,now:u64)->Result<ConfirmedData>{
  if consent!=Some(DataConsent::BoundedGoalContext)||self.preview.required_consent!=DataConsent::BoundedGoalContext{return Err("goalMemoryConsentRequired");}
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
  self.captured.data_current(goal,server,model,now)?;Captured::compose(self.captured.capture.as_deref(),self.captured.received.as_deref(),self.captured.calendars.as_deref(),self.captured.progress.as_deref(),goal,now)
 }
}
