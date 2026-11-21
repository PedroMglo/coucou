//! Own rule data policy for future observations of an exact selected goal set.
//! Pinned notes are optional; this policy never borrows old note-only consent.
use super::*;
use crate::pet_memory::context_model::{Selection as Notes,SelectedProject};
use super::super::{project_goal_observation as observed,store::Store};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ProjectProgressPolicy{
 pub schema_version:u32,pub consent:ProjectConsent,pub selection:observed::Selection,
 #[serde(default,skip_serializing_if="Option::is_none")]pub notes:Option<Notes>,
 pub max_data_bytes:u32,pub max_input_bytes:u32,pub max_capture_seconds:u32,pub max_reads_per_occurrence:u16,
}
impl ProjectProgressPolicy{
 pub(super)fn valid_for(&self,rule:&Rule)->bool{
  let DataScope::ProjectProgress{source_conversation_id,project,goals,include_latest_response,notes}=&rule.definition.data_scope else{return false;};
  self.schema_version==1&&self.consent==ProjectConsent::BoundedRuleProjectGoalProgressData&&rule.owner.adapter=="ollama"&&self.selection.valid(&rule.owner.pet_id)&&self.selection.source_conversation_id==*source_conversation_id&&self.selection.project.id==project.id&&self.selection.project.revision==project.revision&&self.selection.goals==*goals&&self.selection.include_latest_response==*include_latest_response
   &&self.max_data_bytes==observed::MAX_DATA_BYTES as u32&&self.max_input_bytes==262144&&self.max_capture_seconds==10&&self.max_reads_per_occurrence==1
   &&match &self.notes{None=>notes.is_empty(),Some(n)=>n.project.as_deref()==Some(&self.selection.project)&&super::project_policy::valid_project_notes(n,&rule.owner.pet_id,source_conversation_id,project,notes)}
 }
 pub(super)fn capture(rule:&Rule)->Result<Self,String>{
  let DataScope::ProjectProgress{source_conversation_id,project,goals,include_latest_response,notes}=&rule.definition.data_scope else{return Err("ruleProjectProgressSelectionRequired".into());};
  let now=crate::assistant_memory::now();let memory=crate::pet_memory::context::Capture::catalog_path();
  let selection=observed::Selection::capture(&memory,&rule.owner.pet_id,source_conversation_id,project,goals.clone(),*include_latest_response,now).map_err(|_|"ruleProjectProgressSelectionChanged")?;
  let notes=if notes.is_empty(){None}else{let capture=crate::pet_memory::context::Capture::at_manual(&memory,&rule.owner.pet_id,source_conversation_id,notes,Some(project),now).map_err(str::to_owned)?;Some(Notes{schema_version:2,project:Some(Box::new(capture.project().ok_or("ruleProjectChanged")?)),references:capture.references().to_vec()})};
  let result=Self{schema_version:1,consent:ProjectConsent::BoundedRuleProjectGoalProgressData,selection,notes,max_data_bytes:observed::MAX_DATA_BYTES as u32,max_input_bytes:262144,max_capture_seconds:10,max_reads_per_occurrence:1};
  if !result.valid_for(rule)||!result.current(rule){return Err("ruleProjectProgressSelectionChanged".into());}Ok(result)
 }
 pub(super)fn current(&self,rule:&Rule)->bool{
  if !self.valid_for(rule){return false;}let Ok(Some(store))=Store::existing(&crate::settings::config_dir().join("pet-runtime-v1"))else{return false;};let memory=crate::pet_memory::context::Capture::catalog_path();let now=crate::assistant_memory::now();
  self.selection.verify_source_pins(&store,&memory,&rule.owner.pet_id,now).is_ok()&&self.notes.as_ref().is_none_or(|n|crate::pet_memory::context::Capture::at_goal_selection(&memory,&rule.owner.pet_id,Some(&self.selection.source_conversation_id),n,now).and_then(|c|c.verify(now)).is_ok())
 }
 pub(super)fn observation(&self,rule:&Rule)->Result<observed::Snapshot,String>{
  if !self.current(rule){return Err("ruleProjectProgressSelectionChanged".into());}
  let store=Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1")).map_err(|_|"ruleProjectGoalsUnavailable")?.ok_or("ruleProjectGoalsUnavailable")?;
  observed::capture(&store,&super::super::admission_review::authority_root(),&crate::pet_memory::context::Capture::catalog_path(),&self.selection,None,crate::assistant_memory::now()).map_err(|e|format!("ruleProjectProgress:{e:?}"))
 }
 pub(in crate::pet_runtime)fn matches_context(&self,b:&model::ContextBinding)->bool{
  b.schema_version==12&&b.project_progress.as_ref().is_some_and(|p|p.selection==self.selection&&p.valid_for(&b.pet_id,Some(&b.goal_id)))&&b.memory==self.notes&&!b.has_calendars()&&b.received_request.is_none()&&b.source_conversation_id.as_ref()==Some(&self.selection.source_conversation_id)&&b.source_messages==0&&b.copied_messages==0&&b.mode==model::PreparationMode::Dedicated
 }
 pub(super)fn records(&self,rule:&Rule)->Result<Option<Vec<crate::pet_memory::Entry>>,String>{
  if !self.current(rule){return Err("ruleProjectProgressSelectionChanged".into());}let Some(n)=&self.notes else{return Ok(None);};
  crate::pet_memory::context::Capture::at_goal_selection(&crate::pet_memory::context::Capture::catalog_path(),&rule.owner.pet_id,Some(&self.selection.source_conversation_id),n,crate::assistant_memory::now()).and_then(|c|c.records(crate::assistant_memory::now())).map(Some).map_err(str::to_owned)
 }
 pub(super)fn choice(&self)->SelectedProject{SelectedProject{id:self.selection.project.id.clone(),revision:self.selection.project.revision}}
}
