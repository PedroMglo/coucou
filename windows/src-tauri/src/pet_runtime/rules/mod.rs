//! Durable rule drafts are metadata, never execution/recurrence authority.
use super::{adapter,model::{self,Owner}};
use crate::pet_calendar::schedule::{Schedule,Preview};
#[cfg(target_os="linux")]use crate::private_registry;
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Definition{pub name:String,pub intent:String,pub criteria:Vec<String>,pub schedule:Schedule,pub missed:Missed,pub grace_seconds:u32,pub data_scope:DataScope,pub limits:Limits}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Missed{Skip,Coalesce,Decision}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum DataScope{None}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Limits{pub max_occurrences:u16,pub max_model_calls:u16,pub max_tool_calls:u16,pub max_output_bytes:u64,pub occurrence_output_bytes:u64,pub occurrence_timeout_seconds:u32}
impl Definition{
 pub(super)fn validate(&self)->Result<(),&'static str>{
  let l=&self.limits;
  if !model::text(&self.name,160)||!model::text(&self.intent,4096)||self.criteria.is_empty()||self.criteria.len()>16||self.criteria.iter().any(|c|!model::text(c,1024))||self.grace_seconds>3600||!(1..=64).contains(&l.max_occurrences)||l.max_model_calls!=l.max_occurrences||l.max_tool_calls!=0||!(1024..=256*1024).contains(&l.occurrence_output_bytes)||!(60..=86400).contains(&l.occurrence_timeout_seconds)||l.max_output_bytes!=l.occurrence_output_bytes*u64::from(l.max_occurrences)||l.max_output_bytes>16*1024*1024{return Err("invalidRule");}
  self.schedule.validate()
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum State{Draft}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Rule{pub id:String,pub revision:u64,pub owner:Owner,pub state:State,pub created_at:u64,pub updated_at:u64,pub definition:Definition}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Book{version:u32,generation:u64,rules:Vec<Rule>}
fn valid_id(s:&str)->bool{s.strip_prefix("rule-").is_some_and(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f')))}
#[cfg(target_os="linux")]
impl private_registry::Document for Book{
 const MAX_BYTES:usize=768*1024;const LOCK_NAME:&'static str="pet-rules.lock";const TEMP_PREFIX:&'static str=".pet-rules-";
 fn empty()->Self{Self{version:1,generation:0,rules:vec![]}}
 fn validate(&self)->Result<(),&'static str>{
  if self.version!=1||self.rules.len()>64{return Err("invalidRuleBook");}let mut ids=std::collections::BTreeSet::new();
  for r in &self.rules{if !valid_id(&r.id)||!ids.insert(&r.id)||r.revision==0||!r.owner.local_engine()||r.created_at==0||r.created_at>r.updated_at||r.updated_at>8_640_000_000_000_000{return Err("invalidRuleIdentity");}r.definition.validate()?;}Ok(())
 }
 fn generation(&self)->u64{self.generation}
 fn next(mut self,previous:&Self)->Result<Self,&'static str>{
  if self.version!=1||self.generation!=previous.generation||previous.rules.iter().any(|old|!self.rules.iter().any(|r|r.id==old.id)){return Err("ruleRevisionConflict");}
  self.generation=self.generation.checked_add(1).ok_or("revisionLimit")?;
  for r in &mut self.rules{if let Some(old)=previous.rules.iter().find(|o|o.id==r.id){if r.revision!=old.revision||r.owner!=old.owner||r.created_at!=old.created_at||r.updated_at<old.updated_at{return Err("ruleRevisionConflict");}if r!=old{r.revision=r.revision.checked_add(1).ok_or("revisionLimit")?;}}else{if r.revision>1{return Err("ruleRevisionConflict");}r.revision=1;}}
  self.validate()?;Ok(self)
 }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Document{pub schema_version:u32,pub revision:String,pub rules:Vec<Rule>,pub execution_available:bool}
#[cfg(target_os="linux")]
fn document(s:private_registry::Snapshot<Book>,pet:&str)->Document{Document{schema_version:1,revision:s.revision,rules:s.registry.rules.into_iter().filter(|r|r.owner.pet_id==pet).collect(),execution_available:false}}
fn path()->std::path::PathBuf{crate::pet_config::path().with_file_name("pet-rules.json")}
fn check(w:&tauri::WebviewWindow)->Result<(),String>{if adapter::allowed(w.label()){Ok(())}else{Err("Abre a gestão de rotinas do pet.".into())}}
fn error(e:&str)->String{match e{"registryConflict"|"registryChanged"|"ruleRevisionConflict"=>"O catálogo de regras mudou. Consulta-o antes de guardar novamente.","registryCommitUncertain"=>"A gravação pode ter ocorrido. Consulta o catálogo antes de repetir.","invalidRuleBook"=>"O catálogo de regras tem um formato incompatível; foi preservado.","unknownTimeZone"=>"O fuso IANA não existe na base de dados distribuída.",_=>e}.into()}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Save{pub pet_id:String,pub expected_revision:String,pub request_id:String,pub rule_id:Option<String>,pub definition:Definition}
#[cfg(target_os="linux")]
fn save_at(path:&std::path::Path,input:&Save,owner:Owner,now:u64,current:impl Fn()->bool)->Result<Document,&'static str>{
 save_at_reviewed(path,input,owner,now,current,|_|Ok(()))
}
#[cfg(target_os="linux")]
fn save_at_reviewed(path:&std::path::Path,input:&Save,owner:Owner,now:u64,current:impl Fn()->bool,before_edit:impl FnOnce(&Rule)->Result<(),&'static str>)->Result<Document,&'static str>{
 input.definition.validate()?;if !owner.local_engine()||owner.pet_id!=input.pet_id||!model::key(&input.request_id)||!current(){return Err("invalidRuleIdentity");}
 let snapshot=private_registry::load_for_update::<Book>(path,&input.expected_revision)?;let mut book=snapshot.registry.clone();
 let id=input.rule_id.clone().unwrap_or_else(||format!("rule-{}",model::hash(format!("{}\0{}",input.pet_id,input.request_id).as_bytes())));if !valid_id(&id){return Err("invalidRuleIdentity");}
 if let Some(old)=book.rules.iter_mut().find(|r|r.id==id){
  if input.rule_id.is_none()||!old.owner.same_runtime_binding(&owner){return Err("ruleRevisionConflict");}
  old.definition=input.definition.clone();old.updated_at=now;
 }else{
  if input.rule_id.is_some(){return Err("ruleRevisionConflict");}
  book.rules.push(Rule{id,revision:0,owner,state:State::Draft,created_at:now,updated_at:now,definition:input.definition.clone()});
 }
 let result=private_registry::save_guarded(path,&input.expected_revision,book,|old,_|{if let Some(id)=&input.rule_id{let rule=old.rules.iter().find(|r|&r.id==id&&r.owner.pet_id==input.pet_id).ok_or("ruleRevisionConflict")?;before_edit(rule)?;}Ok(())},||if current(){Ok(())}else{Err("ruleReviewClosed")})?;Ok(document(result,&input.pet_id))
}
#[tauri::command]
pub async fn pet_runtime_rules(window:tauri::WebviewWindow,pet_id:String)->Result<Document,String>{
 check(&window)?;if !crate::pet_config::defaults().pets.contains_key(&pet_id){return Err("invalidRuleOwner".into());}
 #[cfg(not(target_os="linux"))]{Err("As rotinas ainda requerem Linux.".into())}
 #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||private_registry::load_at::<Book>(&path()).map(|s|document(s,&pet_id)).map_err(error)).await.map_err(|_|"Falhou a leitura das regras.".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_save(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Save)->Result<Document,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("As rotinas ainda requerem Linux.".into())}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleReviewClosed")?;tauri::async_runtime::spawn_blocking(move||{let owner=adapter::owner(&app,&input.pet_id).map_err(|_|"invalidRuleOwner".to_string())?;save_at_reviewed(&path(),&input,owner,crate::assistant_memory::now(),||frame.current(),authority_api::before_edit).map_err(error)}).await.map_err(|_|"A gravação das regras foi interrompida. Consulta o catálogo.".to_string())?}
}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PreviewInput{pub pet_id:String,pub rule_id:String,pub revision:u64}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RulePreview{pub rule:Rule,pub schedule:Preview,pub observed_at:u64,pub backlog:Backlog,pub execution_available:bool}
#[derive(Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Backlog{pub selected_at:Option<u64>,pub skipped:usize,pub coalesced:usize,pub needs_decision:usize}
fn backlog(p:&Preview,missed:Missed,grace:u32,after:Option<u64>,now:u64)->Backlog{
 let due:Vec<u64>=p.slots.iter().filter_map(|s|s.at).filter(|n|*n<=now&&after.is_none_or(|a|*n>a)).collect();let mut r=Backlog{selected_at:None,skipped:0,coalesced:0,needs_decision:0};let Some(at)=due.last().copied()else{return r;};
 let late=due.iter().filter(|n|now.saturating_sub(**n)>u64::from(grace)*1000).count();
 match missed{Missed::Decision if late>0=>r.needs_decision=due.len(),Missed::Coalesce=>{r.selected_at=Some(at);r.coalesced=due.len()-1;},_=>{if now.saturating_sub(at)<=u64::from(grace)*1000{r.selected_at=Some(at);}r.skipped=due.len()-usize::from(r.selected_at.is_some());}}
 r
}
#[tauri::command]
pub async fn pet_runtime_rule_preview(window:tauri::WebviewWindow,input:PreviewInput)->Result<RulePreview,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=input;Err("As rotinas ainda requerem Linux.".into())}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleReviewClosed")?;tauri::async_runtime::spawn_blocking(move||{
  let snapshot=private_registry::load_at::<Book>(&path()).map_err(error)?;let rule=snapshot.registry.rules.iter().find(|r|r.id==input.rule_id&&r.owner.pet_id==input.pet_id&&r.revision==input.revision).cloned().ok_or("ruleRevisionConflict")?;
  let preview=crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||frame.current()&&snapshot.unchanged()).map_err(error)?;if !frame.current()||!snapshot.unchanged(){return Err(error("ruleRevisionConflict"));}
  let now=crate::assistant_memory::now();let backlog=backlog(&preview,rule.definition.missed,rule.definition.grace_seconds,None,now);Ok(RulePreview{rule,schedule:preview,observed_at:now,backlog,execution_available:false})
 }).await.map_err(|_|"A prévia das regras foi interrompida.".to_string())?}
}
#[cfg(all(test,target_os="linux"))]mod tests;


mod occurrence_model;
pub(crate)use occurrence_model::{OccurrenceReference,RuleAdmission};
mod dispatch_model;
pub(crate)use dispatch_model::{DispatchPolicy,ReadinessProbe,AutomaticObservation,ProbeRecovery,RecoveryOutcome};
mod authority_model;
pub(super)use authority_model::*;
mod intent_retirement_model;
pub(crate)use intent_retirement_model::{RuleIntentRetirement,RetiredRuleIntent};
mod succession_model;
pub(crate)use succession_model::{RuleSuccession,RuleLineageObservation};
mod revision_model;
pub(crate)use revision_model::{RuleRevisionMode,RuleRevisionAuthority,RuleExecutionBounds,RuleAllocationObservation};
#[cfg(target_os="linux")]
pub(super)fn read_rule_at(path:&std::path::Path,pet:&str,id:&str,revision:u64)->Result<Rule,&'static str>{
 let snapshot=private_registry::load_at::<Book>(path)?;snapshot.registry.rules.iter().find(|r|r.owner.pet_id==pet&&r.id==id&&r.revision==revision).cloned().ok_or("ruleRevisionConflict")
}
pub(super)fn valid_rule(rule:&Rule)->bool{valid_id(&rule.id)&&rule.revision>0&&rule.owner.local_engine()&&rule.created_at>0&&rule.updated_at>=rule.created_at&&rule.updated_at<=8_640_000_000_000_000&&rule.definition.validate().is_ok()}

pub(crate)mod authority_api;

pub(crate)mod occurrences;

#[cfg(target_os="linux")]pub(crate)mod scheduled;
#[cfg(target_os="linux")]pub(crate)fn saved_rules()->Result<Vec<Rule>,&'static str>{if !path().try_exists().map_err(|_|"ruleStorageUnavailable")?{return Ok(vec![]);}Ok(private_registry::load_at::<Book>(&path())?.registry.rules)}

pub(crate)mod recovery_api;

pub(crate)mod intent_retirement_api;

pub(crate)mod succession_api;

pub(crate)mod revision_api;
