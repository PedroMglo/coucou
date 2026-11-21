//! Native bounded DATA observations of explicitly selected goals, not an
//! execution/delegation grant. Goal revisions may progress; identity/definition
//! and project membership remain exact. No background observer or new store.
use super::{model::{self,Record,State},history::Charge,store::Store,authority::{Authority,Failure},results};
use crate::pet_memory::{projects::{Project,Capture as ProjectCapture},context_model::SelectedProject};
use serde::{Serialize,Deserialize};
use std::path::Path;
pub(super)const MAX_GOALS:usize=8;
pub(super)const MAX_DATA_BYTES:usize=128*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Reference{pub goal_id:String,pub spec_hash:String}
impl Reference{
 pub(super)fn of(goal:&Record)->model::Result<Self>{goal.validate()?;Ok(Self{goal_id:goal.id.clone(),spec_hash:model::fingerprint(&goal.spec)?})}
 pub(super)fn valid(&self)->bool{model::goal_id(&self.goal_id)&&super::authority::decode(&self.spec_hash).is_some()}
 fn matches(&self,goal:&Record,project:&Project)->bool{
  self.goal_id==goal.id&&model::fingerprint(&goal.spec).ok().as_ref()==Some(&self.spec_hash)&&goal.spec.owner.pet_id==project.pet_id
   &&goal.execution_context.as_ref().and_then(model::ContextBinding::project).is_some_and(|p|p.id==project.id&&p.pet_id==project.pet_id)
 }
}
pub(super)fn valid_choices(refs:&[Reference])->bool{!refs.is_empty()&&refs.len()<=MAX_GOALS&&refs.iter().enumerate().all(|(i,r)|r.valid()&&!refs[..i].iter().any(|p|p.goal_id==r.goal_id))}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Selection{pub schema_version:u32,pub project:Project,pub source_conversation_id:String,pub goals:Vec<Reference>,pub include_latest_response:bool}
impl Selection{
 pub(super)fn capture(memory:&Path,pet:&str,source:&str,project:&SelectedProject,goals:Vec<Reference>,include_latest_response:bool,now:u64)->model::Result<Self>{
  let project=ProjectCapture::at(memory,pet,source,project,now).map_err(|_|model::Error::Conflict)?;
  let data=Self{schema_version:1,project:project.project().clone(),source_conversation_id:source.into(),goals,include_latest_response};if !data.valid(pet){return Err(model::Error::InvalidInput);}project.verify(now).map_err(|_|model::Error::Conflict)?;Ok(data)
 }
 pub(super)fn valid(&self,pet:&str)->bool{self.schema_version==1&&self.project.valid()&&self.project.pet_id==pet&&self.project.enabled&&self.project.revision>0&&crate::assistant_memory::valid_id(&self.source_conversation_id)&&self.project.conversation_ids.contains(&self.source_conversation_id)&&valid_choices(&self.goals)}
 /// Only identity/definition is live after a capture; facts below are explicitly
 /// as-of observations. Ordinary source progress never becomes fresh consent.
 pub(super)fn verify_pins(&self,store:&Store,memory:&Path,pet:&str,now:u64)->model::Result<ProjectCapture>{
  self.verify_pins_mode(store,memory,pet,now,false)
 }
 /// Scope checks may run inside the already owned Authority/catalog transaction.
 /// Only source identity is checked here; fresh facts still require capture().
 pub(super)fn verify_source_pins(&self,store:&Store,memory:&Path,pet:&str,now:u64)->model::Result<ProjectCapture>{self.verify_pins_mode(store,memory,pet,now,true)}
 fn verify_pins_mode(&self,store:&Store,memory:&Path,pet:&str,now:u64,atomic_source:bool)->model::Result<ProjectCapture>{
  if !self.valid(pet){return Err(model::Error::InvalidInput);}let project=ProjectCapture::at(memory,pet,&self.source_conversation_id,&SelectedProject{id:self.project.id.clone(),revision:self.project.revision},now).map_err(|_|model::Error::Conflict)?;
  if project.project()!=&self.project{return Err(model::Error::Conflict);}
  for reference in &self.goals{let goal=if atomic_source{store.source_snapshot(&reference.goal_id,pet)?}else{store.read(&reference.goal_id,pet)?};if !reference.matches(&goal,&self.project){return Err(model::Error::Conflict);}}
  project.verify(now).map_err(|_|model::Error::Conflict)?;Ok(project)
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="status",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub(super)enum Reply{NotRequested,NotAdmitted,NotFiniteResponse,SourceDataUnavailable{reason:String},NotStarted,OutcomeUnknown{phase:String},Available{receipt:results::ModelResult,receipt_hash:String}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct AdmissionObservation{pub decision_id:String,pub deadline_at:u64,pub grant_phase:String,pub owner_state:String,pub control_projection_current:bool,pub goal_lease_matches:bool}
impl AdmissionObservation{fn valid(&self)->bool{model::token(&self.decision_id)&&self.deadline_at>0&&self.deadline_at<=8_640_000_000_000_000&&matches!(self.grant_phase.as_str(),"issued"|"claimed"|"waiting"|"draining"|"closed"|"revoked")&&matches!(self.owner_state.as_str(),"originalProcessLive"|"ownerAbsent"|"identityMismatch"|"unavailable"|"legacyUnknown"|"notClaimed"|"closed")}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Observation{
 pub reference:Reference,pub goal_revision:String,pub goal_hash:String,pub history_hash:String,pub observed_at:u64,
 pub intent:String,pub criteria:Vec<String>,pub state:State,pub admission:Option<AdmissionObservation>,pub evidence_complete:bool,pub criteria_verified:bool,pub goal_complete:bool,
 pub cumulative_charge:Option<Charge>,pub authenticated_subtotal:Charge,pub accounting_complete:bool,pub outcomes_complete:bool,
 pub history_continuity_verified:bool,pub response:Reply,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Snapshot{pub schema_version:u32,pub selection:Selection,pub observed_at:u64,pub observations:Vec<Observation>}
impl Snapshot{
 pub(super)fn valid_for(&self,pet:&str,target:Option<&str>)->bool{
  self.schema_version==1&&self.selection.valid(pet)&&self.observed_at>0&&self.observed_at<=8_640_000_000_000_000&&self.observations.len()==self.selection.goals.len()
   &&self.observations.iter().zip(&self.selection.goals).all(|(o,r)|&o.reference==r&&target!=Some(r.goal_id.as_str())&&model::token(&o.goal_revision)&&super::authority::decode(&o.goal_hash).is_some()&&super::authority::decode(&o.history_hash).is_some()&&o.observed_at==self.observed_at&&model::text(&o.intent,65536)&&!o.criteria.is_empty()&&o.criteria.len()<=32&&o.criteria.iter().all(|c|model::text(c,4096))&&o.admission.as_ref().is_none_or(AdmissionObservation::valid)&&(!o.criteria_verified||o.evidence_complete)&&(!o.goal_complete||o.criteria_verified)&&(!o.accounting_complete||o.cumulative_charge.is_some())&&o.authenticated_subtotal.valid()&&o.cumulative_charge.as_ref().is_none_or(Charge::valid)&&match &o.response{
    Reply::NotRequested=>!self.selection.include_latest_response,
    Reply::NotAdmitted|Reply::NotFiniteResponse|Reply::NotStarted=>self.selection.include_latest_response,
    Reply::SourceDataUnavailable{reason}=>self.selection.include_latest_response&&matches!(reason.as_str(),"selectedDataChangedOrUnavailable"|"selectedDataKindRequiresOwnForwardingScope"),
    Reply::OutcomeUnknown{phase}=>self.selection.include_latest_response&&matches!(phase.as_str(),"reserved"|"started"|"uncertain"|"settledReceiptMissing"),
    Reply::Available{receipt,receipt_hash}=>self.selection.include_latest_response&&receipt.valid()&&receipt.goal_id==r.goal_id&&receipt.pet_id==pet&&receipt.recorded_at<=self.observed_at&&model::fingerprint(receipt).ok().as_ref()==Some(receipt_hash),
   })&&serde_json::to_vec(self).is_ok_and(|b|b.len()<=MAX_DATA_BYTES)
 }
}
fn response(authority:&Path,memory:&Path,goal:&Record,requested:bool,now:u64)->model::Result<Reply>{
 if !requested{return Ok(Reply::NotRequested);}let Some(row)=goal.admissions.last()else{return Ok(Reply::NotAdmitted);};
 if !row.spec.finite_response(){return Ok(Reply::NotFiniteResponse);}
 // Keep original selected-data revocation effective for derived text. A
 // progress review never bypasses provenance or promotes a different data kind.
 let binding=goal.execution_context.as_ref().ok_or(model::Error::Conflict)?;
 if binding.schema_version!=2||binding.received_request.is_some()||binding.has_calendars(){return Ok(Reply::SourceDataUnavailable{reason:"selectedDataKindRequiresOwnForwardingScope".into()});}
 let selection=binding.memory.as_ref().ok_or(model::Error::Conflict)?;
 if crate::pet_memory::context::Capture::at_goal_selection(memory,&goal.spec.owner.pet_id,binding.source_conversation_id.as_deref(),selection,now).and_then(|c|c.verify(now)).is_err(){return Ok(Reply::SourceDataUnavailable{reason:"selectedDataChangedOrUnavailable".into()});}
 let a=Authority::existing(authority).map_err(Failure::control_error)?;
 let view=results::lookup_for_user(&a,&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&format!("response-{}",row.spec.decision_id)).map_err(Failure::control_error)?;
 if view.historical_source.is_some(){return Err(model::Error::Conflict);}
 Ok(match view.result{results::Lookup::NotStarted=>Reply::NotStarted,results::Lookup::RecoveryRequired{phase}=>Reply::OutcomeUnknown{phase:match phase{results::RecoveryPhase::Reserved=>"reserved",results::RecoveryPhase::Started=>"started",results::RecoveryPhase::Uncertain=>"uncertain",results::RecoveryPhase::SettledReceiptMissing=>"settledReceiptMissing"}.into()},results::Lookup::Available{receipt}=>{if receipt.decision_id!=row.spec.decision_id||receipt.request_id!=format!("response-{}",row.spec.decision_id){return Err(model::Error::Conflict);}Reply::Available{receipt_hash:view.receipt_hash.ok_or(model::Error::Conflict)?,receipt}}})
}
/// One native snapshot. History is rechecked as metadata after the single
/// receipt lookup; changes refuse this capture rather than silently refreshing
/// the result, observation date, definition or selected IDs. No IO is started.
pub(super)fn capture(store:&Store,authority:&Path,memory:&Path,selection:&Selection,target:Option<&str>,now:u64)->model::Result<Snapshot>{
 let project=selection.verify_pins(store,memory,&selection.project.pet_id,now)?;let mut observations=vec![];
 for reference in &selection.goals{
  if target==Some(reference.goal_id.as_str()){return Err(model::Error::Conflict);}
  let pet=&selection.project.pet_id;let goal=store.read(&reference.goal_id,pet)?;
  if !reference.matches(&goal,&selection.project)||now<goal.updated_at{return Err(model::Error::Conflict);}
  let history=store.history(authority,&goal.id,pet,&goal.revision,now)?;
  if history.scope!="allRecordedAdmissions"||!history.source_consistent||!history.scan_complete||history.pending_artifacts!=0{return Err(model::Error::Conflict);}
  let latest=store.reconciliation(authority,&goal.id,pet,&goal.revision,now)?;
  if !latest.source_consistent||!latest.scan_complete||latest.goal_hash!=history.goal_hash{return Err(model::Error::Conflict);}
  let admission=latest.grant.as_ref().map(|g|{let row=goal.admissions.last().ok_or(model::Error::Conflict)?;if row.spec.decision_id!=g.decision_id{return Err(model::Error::Conflict);}Ok(AdmissionObservation{decision_id:g.decision_id.clone(),deadline_at:row.spec.budget.deadline_at,grant_phase:g.phase.into(),owner_state:serde_json::to_value(g.owner.state).map_err(|_|model::Error::InvalidRecord)?.as_str().ok_or(model::Error::InvalidRecord)?.to_owned(),control_projection_current:g.control_projection_current,goal_lease_matches:g.goal_lease_matches})}).transpose()?;
  let evidence_complete=latest.grant.as_ref().is_none_or(|g|g.control_projection_current&&g.goal_lease_matches);
  let response=response(authority,memory,&goal,selection.include_latest_response,now)?;
  let after=store.history(authority,&goal.id,pet,&goal.revision,now)?;
  if after.source_hash!=history.source_hash||model::fingerprint(&store.read(&goal.id,pet)?)?!=history.goal_hash{return Err(model::Error::Conflict);}
  observations.push(Observation{reference:reference.clone(),goal_revision:goal.revision,goal_hash:history.goal_hash,history_hash:history.source_hash,observed_at:now,intent:goal.spec.intent,criteria:goal.spec.criteria,state:goal.state,admission,evidence_complete,criteria_verified:evidence_complete&&latest.criteria_verified,goal_complete:evidence_complete&&latest.goal_complete,cumulative_charge:history.cumulative_charge,authenticated_subtotal:history.authenticated_subtotal,accounting_complete:history.accounting_complete,outcomes_complete:history.outcomes_complete,history_continuity_verified:history.history_continuity_verified,response});
 }
 project.verify(now).map_err(|_|model::Error::Conflict)?;let snapshot=Snapshot{schema_version:1,selection:selection.clone(),observed_at:now,observations};if !snapshot.valid_for(&selection.project.pet_id,target){return Err(model::Error::LimitReached);}Ok(snapshot)
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Prepare{pub pet_id:String,pub source_conversation_id:String,pub project:SelectedProject,pub goal_ids:Vec<String>,pub include_latest_response:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Prepared{pub schema_version:u32,pub selection:Selection,pub snapshot:Snapshot,pub execution_authorized:bool,pub model_use_authorized:bool}
/// Pure native source selection/review. The own rule review and writer must
/// independently recapture and seal this scope; this DTO grants no future use.
pub(super)fn prepare_at(store:&Store,authority:&Path,memory:&Path,input:&Prepare,now:u64)->model::Result<Prepared>{
 if input.goal_ids.is_empty()||input.goal_ids.len()>MAX_GOALS{return Err(model::Error::InvalidInput);}
 let mut references=vec![];
 for id in &input.goal_ids{let goal=store.read(id,&input.pet_id)?;references.push(Reference::of(&goal)?);}
 let selection=Selection::capture(memory,&input.pet_id,&input.source_conversation_id,&input.project,references,input.include_latest_response,now)?;
 let snapshot=capture(store,authority,memory,&selection,None,now)?;
 Ok(Prepared{schema_version:1,selection,snapshot,execution_authorized:false,model_use_authorized:false})
}
#[tauri::command]
pub async fn pet_runtime_project_goal_observation_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Prepare)->model::Result<Prepared>{
 use tauri::Manager;
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;
 tauri::async_runtime::spawn_blocking(move||{
  let owner=super::adapter::owner(&app,&input.pet_id)?;
  if owner.adapter!="ollama"||!app.state::<crate::chat::Chat>().has_public_conversation(&owner.provider,&input.source_conversation_id){return Err(model::Error::Conflict);}
  let store=Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(model::Error::NotFound)?;
  let draft=prepare_at(&store,&super::admission_review::authority_root(),&crate::pet_memory::context::Capture::catalog_path(),&input,crate::assistant_memory::now())?;
  if !frame.current(){return Err(model::Error::Conflict);}Ok(draft)
 }).await.map_err(|_|model::Error::StorageUnavailable)?
}

/// Live provenance of already captured DATA, not a future observation refresh.
/// Keep original note captures so revocation cannot be undone for this writer.
pub(super)struct Captured{pub snapshot:Snapshot,store:Store,authority:Authority,project:ProjectCapture,notes:Vec<crate::pet_memory::context::Capture>,retired:std::sync::atomic::AtomicBool}
impl Captured{
 pub(super)fn reopen(goals:&Path,authority:&Path,memory:&Path,snapshot:Snapshot,target:&str,now:u64)->model::Result<Self>{
  if !snapshot.valid_for(&snapshot.selection.project.pet_id,Some(target)){return Err(model::Error::Conflict);}
  let store=Store::existing_verified(goals,authority)?.ok_or(model::Error::NotFound)?;
  let project=snapshot.selection.verify_pins(&store,memory,&snapshot.selection.project.pet_id,now)?;
  // The continuous fence can run inside the existing goal catalog writer.
  // Reuse its atomic source_snapshot reader, as received-goal publication does;
  // fresh accounting is captured above, never refreshed by this guard.
  let store=Store::existing(goals)?.ok_or(model::Error::NotFound)?;
  let authority=Authority::existing(authority).map_err(Failure::control_error)?;let mut notes=vec![];
  for observation in &snapshot.observations{if let Reply::Available{receipt,..}=&observation.response{
   let goal=store.read(&receipt.goal_id,&receipt.pet_id)?;
   let row=goal.admissions.iter().find(|r|r.spec.decision_id==receipt.decision_id).ok_or(model::Error::Conflict)?;
   if row.spec.context.schema_version!=2||!row.spec.context.project_memory()||row.spec.context.received_request.is_some()||row.spec.context.has_calendars(){return Err(model::Error::Conflict);}
   let selection=row.spec.context.memory.as_ref().ok_or(model::Error::Conflict)?;
   notes.push(crate::pet_memory::context::Capture::at_goal_selection(memory,&receipt.pet_id,row.spec.context.source_conversation_id.as_deref(),selection,now).map_err(|_|model::Error::Conflict)?);
  }}
  let result=Self{snapshot,store,authority,project,notes,retired:std::sync::atomic::AtomicBool::new(false)};result.current(now)?;Ok(result)
 }
 pub(super)fn current(&self,now:u64)->model::Result<()>{
  use std::sync::atomic::Ordering;
  if self.retired.load(Ordering::Acquire){return Err(model::Error::Conflict);}
  let current=(||{
   if now<self.snapshot.observed_at{return Err(model::Error::ClockRollback);}
   self.project.verify(now).map_err(|_|model::Error::Conflict)?;
   self.authority.fence().map_err(Failure::control_error)?;
   for reference in &self.snapshot.selection.goals{let source=self.store.source_snapshot(&reference.goal_id,&self.snapshot.selection.project.pet_id)?;if !reference.matches(&source,&self.snapshot.selection.project)||super::restoration::matches_historical_goal(&self.authority,&source).map_err(Failure::control_error)?{return Err(model::Error::Conflict);}}
   for capture in &self.notes{capture.verify(now).map_err(|_|model::Error::Conflict)?;}
   for observation in &self.snapshot.observations{if let Reply::Available{receipt,..}=&observation.response{
    let goal=self.store.source_snapshot(&receipt.goal_id,&receipt.pet_id)?;
    let row=goal.admissions.iter().find(|r|r.spec.decision_id==receipt.decision_id).ok_or(model::Error::Conflict)?;
    let(lease,attempt,historical)=self.authority.result_attempt_for_user(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&receipt.request_id).map_err(Failure::control_error)?;
    let attempt=attempt.ok_or(model::Error::Conflict)?;
    if historical.is_some()||lease.as_ref().is_none_or(|l|l.id!=receipt.lease_id)||attempt.id!=receipt.attempt_id||attempt.input_hash!=receipt.input_hash||attempt.phase!=super::effect_budget::Phase::Settled||attempt.accepted_output!=Some(receipt.accepted_output_bytes){return Err(model::Error::Conflict);}
   }}Ok(())
  })();
  if current.as_ref().err().is_some_and(|e|!matches!(e,model::Error::Busy|model::Error::StorageUnavailable)){self.retired.store(true,Ordering::Release);}current
 }
}
