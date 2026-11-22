//! Rule-owned consent to repeat the exact reviewed project/note revisions.
//! It is neither a manual data choice nor permission for later note revisions.
use super::*;
use crate::pet_memory::context_model::{Selection,SelectedProject};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum ProjectConsent{BoundedRulePinnedProjectData,BoundedRuleProjectGoalProgressData}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ProjectPolicy{
 pub schema_version:u32,pub consent:ProjectConsent,pub source_conversation_id:String,pub selection:Selection,
 pub max_input_bytes:u32,pub max_capture_seconds:u32,pub max_reads_per_occurrence:u16,
}
pub(super)fn valid_project_notes(selection:&Selection,pet:&str,source:&str,project:&SelectedProject,notes:&[crate::pet_memory::context_model::Selected])->bool{
 selection.schema_version==2&&selection.valid(pet,Some(source))&&selection.matches_project(Some(project))&&crate::pet_memory::context_model::same_selection(Some(selection),Some(notes))&&selection.references.iter().all(|r|matches!(&r.owner.scope,crate::pet_memory::Scope::Project{project_id}if project_id==&project.id))
}
impl ProjectPolicy{
 pub(super)fn valid_for(&self,rule:&Rule)->bool{
  let DataScope::Project{source_conversation_id,project,notes}=&rule.definition.data_scope else{return false;};
  self.schema_version==1&&self.consent==ProjectConsent::BoundedRulePinnedProjectData&&self.source_conversation_id==*source_conversation_id&&self.max_input_bytes==262144&&self.max_capture_seconds==10&&self.max_reads_per_occurrence==1
   &&rule.owner.adapter=="ollama"&&valid_project_notes(&self.selection,&rule.owner.pet_id,source_conversation_id,project,notes)
 }
 pub(in crate::pet_runtime)fn matches_context(&self,b:&model::ContextBinding)->bool{
  b.schema_version==2&&b.memory.as_ref()==Some(&self.selection)&&!b.has_calendars()&&b.received_request.is_none()&&b.source_conversation_id.as_ref()==Some(&self.source_conversation_id)&&b.source_messages==0&&b.copied_messages==0&&b.mode==model::PreparationMode::Dedicated
 }
 #[cfg(target_os="linux")]
 pub(super)fn capture(rule:&Rule)->Result<Self,String>{
  let capture=rule.definition.data_scope.capture_project(&rule.owner.pet_id)?.ok_or("ruleProjectChoiceRequired")?;
  let DataScope::Project{source_conversation_id,..}=&rule.definition.data_scope else{return Err("ruleProjectChoiceRequired".into());};
  let selection=Selection{schema_version:2,project:Some(Box::new(capture.project().ok_or("ruleProjectChoiceRequired")?)),references:capture.references().to_vec()};
  let policy=Self{schema_version:1,consent:ProjectConsent::BoundedRulePinnedProjectData,source_conversation_id:source_conversation_id.clone(),selection,max_input_bytes:262144,max_capture_seconds:10,max_reads_per_occurrence:1};
  if !policy.valid_for(rule){return Err("ruleProjectDataUnsupported".into());}Ok(policy)
 }
 #[cfg(target_os="linux")]
 pub(super)fn reopen(&self,rule:&Rule)->Result<crate::pet_memory::context::Capture,String>{
  if !self.valid_for(rule){return Err("ruleProjectDataChanged".into());}
  crate::pet_memory::context::Capture::at_goal_selection(&crate::pet_memory::context::Capture::catalog_path(),&rule.owner.pet_id,Some(&self.source_conversation_id),&self.selection,crate::assistant_memory::now()).map_err(str::to_owned)
 }
 #[cfg(target_os="linux")]
 pub(super)fn current(&self,rule:&Rule)->bool{self.reopen(rule).is_ok()}
 #[cfg(target_os="linux")]
 pub(super)fn records(&self,rule:&Rule)->Result<Vec<crate::pet_memory::Entry>,String>{self.reopen(rule)?.records(crate::assistant_memory::now()).map_err(str::to_owned)}
 pub(super)fn choice(&self)->Option<SelectedProject>{self.selection.project.as_ref().map(|p|SelectedProject{id:p.id.clone(),revision:p.revision})}
}
