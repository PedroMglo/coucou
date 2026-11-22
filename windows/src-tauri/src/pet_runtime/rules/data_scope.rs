//! Rule-owned source selection is metadata, never permission for source/model IO.
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CalendarReference{pub id:String,pub revision:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum DataScope{None,#[cfg(target_os="linux")]ProjectProgress{source_conversation_id:String,project:crate::pet_memory::context_model::SelectedProject,goals:Vec<super::super::project_goal_observation::Reference>,include_latest_response:bool,notes:Vec<crate::pet_memory::context_model::Selected>},Calendars{sources:Vec<CalendarReference>},Project{source_conversation_id:String,project:crate::pet_memory::context_model::SelectedProject,notes:Vec<crate::pet_memory::context_model::Selected>}}
impl DataScope{
 pub(super)fn project_source(&self)->Option<&str>{match self{Self::Project{source_conversation_id,..}=>Some(source_conversation_id),#[cfg(target_os="linux")]Self::ProjectProgress{source_conversation_id,..}=>Some(source_conversation_id),_=>None}}
 #[cfg(target_os="linux")]
 pub(super)fn validate_progress(&self,pet:&str)->Result<(),String>{
  let Self::ProjectProgress{source_conversation_id,project,goals,include_latest_response,notes}=self else{return Ok(());};
  if !self.valid(){return Err("ruleProjectProgressSelectionRequired".into());}
  let memory=crate::pet_memory::context::Capture::catalog_path();let now=crate::assistant_memory::now();let selection=super::super::project_goal_observation::Selection::capture(&memory,pet,source_conversation_id,project,goals.clone(),*include_latest_response,now).map_err(|_|"ruleProjectProgressSelectionChanged")?;
  let store=super::super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1")).map_err(|_|"ruleProjectGoalsUnavailable")?.ok_or("ruleProjectGoalsUnavailable")?;
  selection.verify_pins(&store,&memory,pet,now).map_err(|_|"ruleProjectGoalsChanged")?;
  if !notes.is_empty(){crate::pet_memory::context::Capture::at_manual(&memory,pet,source_conversation_id,notes,Some(project),now).and_then(|c|c.verify(now)).map_err(str::to_owned)?;}Ok(())
 }

 pub(super)fn valid(&self)->bool{match self{
  Self::None=>true,
  #[cfg(target_os="linux")]Self::ProjectProgress{source_conversation_id,project,goals,notes,..}=>{let mut ids=std::collections::BTreeSet::new();crate::assistant_memory::valid_id(source_conversation_id)&&project.id.len()==32&&project.id.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))&&project.revision>0&&super::super::project_goal_observation::valid_choices(goals)&&notes.len()<=crate::pet_memory::context_model::MAX_SELECTED&&notes.iter().all(|n|crate::assistant_memory::valid_id(&n.id)&&n.revision>0&&ids.insert(&n.id))},
  Self::Project{source_conversation_id,project,notes}=>{let mut ids=std::collections::BTreeSet::new();crate::assistant_memory::valid_id(source_conversation_id)&&project.id.len()==32&&project.id.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))&&project.revision>0&&!notes.is_empty()&&notes.len()<=crate::pet_memory::context_model::MAX_SELECTED&&notes.iter().all(|n|crate::assistant_memory::valid_id(&n.id)&&n.revision>0&&ids.insert(&n.id))},
  Self::Calendars{sources}=>{let mut ids=std::collections::BTreeSet::new();!sources.is_empty()&&sources.len()<=32&&sources.iter().all(|s|!s.id.is_empty()&&s.id.len()<=64&&s.id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))&&s.revision>0&&ids.insert(&s.id))}
 }}
 #[cfg(target_os="linux")]
 pub(super)fn capture_project(&self,pet:&str)->Result<Option<crate::pet_memory::context::Capture>,String>{
  let Self::Project{source_conversation_id,project,notes}=self else{return Ok(None);};
  if !self.valid(){return Err("invalidRuleProjectSelection".into());}
  crate::pet_memory::context::Capture::at_manual(&crate::pet_memory::context::Capture::catalog_path(),pet,source_conversation_id,notes,Some(project),crate::assistant_memory::now()).map(Some).map_err(str::to_owned)
 }
 #[cfg(target_os="linux")]
 pub(super)fn capture(&self,provider:&str)->Result<Vec<crate::pet_sources::Selected>,String>{
  if !matches!(self,Self::Calendars{..}){return Ok(vec![]);}
  if !self.valid(){return Err("invalidRuleCalendarSelection".into());}
  self.capture_enrolled(provider,crate::pet_sources::selected_pet(provider)?)
 }
 #[cfg(target_os="linux")]
 pub(super)fn capture_enrolled(&self,provider:&str,enrolled:Vec<crate::pet_sources::Selected>)->Result<Vec<crate::pet_sources::Selected>,String>{
  let Self::Calendars{sources}=self else{return Ok(vec![]);};
  if !self.valid(){return Err("invalidRuleCalendarSelection".into());}
  let chosen:Vec<_>=sources.iter().map(|s|enrolled.iter().find(|v|v.id()==s.id&&v.revision()==s.revision).cloned().ok_or_else(||"ruleCalendarSelectionChanged".to_string())).collect::<Result<_,_>>()?;
  crate::pet_sources::validate_pet_group(&chosen.iter().collect::<Vec<_>>(),provider)?;
  Ok(chosen)
 }
}
