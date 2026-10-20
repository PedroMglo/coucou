//! User-editable identities, model policies and Mochi integration grants.
use serde::{Deserialize,Serialize};
use std::{collections::{BTreeMap},hash::{Hash,Hasher},io::{Read,Write},path::{Path,PathBuf},sync::Mutex};
use tauri::{AppHandle,Emitter};
#[path="pet_factory.rs"]
mod factory;
const MAX_BYTES:u64=256*1024;
static EDIT:Mutex<()>=Mutex::new(());
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Profile {pub model:String,pub effort:String}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Pet {pub name:String,pub aliases:Vec<String>,pub provider:String,pub mode:String,pub fast:Profile,pub reasoning:Profile,#[serde(default,skip_serializing_if="Option::is_none")]pub permissions:Option<Permissions>,#[serde(default,skip_serializing_if="Option::is_none")]pub local_execution:Option<LocalExecution>}
/// Missing in an older configuration means disabled. New bundled identities may
/// explicitly opt into proposals; every actual command still needs approval.
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct LocalExecution {
 pub mode:LocalExecutionMode,
 #[serde(default="task_timeout")]pub task_timeout_ms:u64,
 #[serde(default="approval_timeout")]pub approval_timeout_ms:u64,
 #[serde(default="command_timeout")]pub command_timeout_ms:u64,
 #[serde(default="max_calls")]pub max_calls:usize,
 #[serde(default="max_output")]pub max_output_bytes:usize,
 #[serde(default,skip_serializing_if="Option::is_none")]pub allowed_programs:Option<Vec<String>>,
}
#[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub enum LocalExecutionMode {Disabled,Ask}
fn task_timeout()->u64{300_000} fn approval_timeout()->u64{120_000}
fn command_timeout()->u64{60_000} fn max_calls()->usize{8} fn max_output()->usize{512*1024}
impl Default for LocalExecution {
 fn default()->Self{Self{mode:LocalExecutionMode::Disabled,task_timeout_ms:task_timeout(),approval_timeout_ms:approval_timeout(),command_timeout_ms:command_timeout(),max_calls:max_calls(),max_output_bytes:max_output(),allowed_programs:None}}
}
impl LocalExecution {
 pub fn validate(&self)->Result<(),String>{
  if !(100..=300_000).contains(&self.task_timeout_ms)||!(100..=120_000).contains(&self.approval_timeout_ms)||!(100..=60_000).contains(&self.command_timeout_ms)||!(1..=8).contains(&self.max_calls)||!(1024..=512*1024).contains(&self.max_output_bytes){return Err("Limites de execução local inválidos.".into());}
  if self.allowed_programs.as_ref().is_some_and(|ps|ps.len()>64||ps.iter().any(|p|p.len()>4096||!std::path::Path::new(p).is_absolute()||p.chars().any(char::is_control))){return Err("Os programas permitidos devem ter caminhos absolutos.".into());}Ok(())
 }
}
#[cfg(test)]mod local_execution_tests{
 use super::*;
 #[test]fn new_default_asks_but_missing_legacy_policy_stays_disabled(){let c=defaults();assert_eq!(c.local_execution().mode,LocalExecutionMode::Ask);let mut legacy=serde_json::to_value(c).unwrap();legacy["pets"]["ai_ollama"].as_object_mut().unwrap().remove("localExecution");let old:Configuration=serde_json::from_value(legacy).unwrap();assert_eq!(old.local_execution().mode,LocalExecutionMode::Disabled);validate(&old).unwrap();assert!(serde_json::to_value(old).unwrap()["pets"]["ai_ollama"].get("localExecution").is_none());}
 #[test]fn no_autoapprove_unknown_grant_or_excess_limit_is_accepted(){assert!(serde_json::from_value::<LocalExecution>(serde_json::json!({"mode":"always"})).is_err());assert!(serde_json::from_value::<LocalExecution>(serde_json::json!({"mode":"ask","fullAccess":true})).is_err());let mut p=LocalExecution::default();p.task_timeout_ms=300_001;assert!(p.validate().is_err());p=LocalExecution::default();p.max_calls=9;assert!(p.validate().is_err());p=LocalExecution::default();p.allowed_programs=Some(vec!["relative".into()]);assert!(p.validate().is_err());}
 #[test]fn another_pet_cannot_borrow_the_local_execution_grant(){let mut c=defaults();c.pets.get_mut("agent_codex").unwrap().local_execution=Some(LocalExecution::default());assert!(validate(&c).is_err());}
}
#[derive(Clone,Debug,Default,Serialize,Deserialize,PartialEq)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Permissions {pub documents:bool,pub email_drafts:bool,pub music:bool,pub open_apps:bool}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Configuration {pub version:u32,pub reasoning_terms:Vec<String>,pub reasoning_length:usize,pub pets:BTreeMap<String,Pet>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Document {pub config:Configuration,pub path:String,pub revision:String}
#[derive(Clone,Debug,PartialEq)]
pub struct Selection {pub model:String,pub effort:String,pub mode:String}
pub fn defaults()->Configuration {factory::bundled()}
/// Availability is bundled policy, separate from stable history/account identity.
pub fn retired(id:&str)->bool {factory::retired(id)}
pub fn require_active(id:&str)->Result<(),String>{
 if retired(id){Err("O pet GitHub está temporariamente desativado. O histórico e as preferências foram preservados.".into())}else{Ok(())}
}
pub fn require_active_provider(provider:&str)->Result<(),String>{
 if let Some(id)=integration_pet(provider){require_active(id)?;}Ok(())
}
pub fn path()->PathBuf {crate::settings::config_dir().join("pets.json")}
pub const SERVICE_PETS:&[&str]=&["integration_resend","integration_n8n","integration_vercel","integration_github","integration_notion","integration_calcom","integration_stripe","integration_spotify"];
pub fn service(id:&str)->bool {SERVICE_PETS.contains(&id)}
pub fn pet_id(provider:&str)->Option<&'static str> {match provider {"codex"=>Some("agent_codex"),"copilot"=>Some("agent_copilot"),"antigravity"=>Some("agent_antigravity"),"gemini-cli"=>Some("agent_gemini"),"claude-cli"=>Some("integration_claude"),"ollama"=>Some("ai_ollama"),"integration_resend"=>Some("integration_resend"),"integration_n8n"=>Some("integration_n8n"),"integration_vercel"=>Some("integration_vercel"),"integration_github"=>Some("integration_github"),"integration_notion"=>Some("integration_notion"),"integration_calcom"=>Some("integration_calcom"),"integration_stripe"=>Some("integration_stripe"),"integration_spotify"=>Some("integration_spotify"),_=>None}}
pub fn integration_pet(provider:&str)->Option<&'static str>{pet_id(provider).or(match provider{"anthropic"=>Some("ai_anthropic"),"openai"=>Some("ai_openai"),"google"=>Some("ai_google"),"lmstudio"=>Some("ai_lmstudio"),_=>None})}
impl Configuration {
 pub fn local_execution(&self)->LocalExecution{self.pets.get("ai_ollama").and_then(|p|p.local_execution.clone()).unwrap_or_default()}
 pub fn permissions(&self,id:&str)->Result<Permissions,String>{
  let pet=self.pets.get(id).ok_or("Pet desconhecido.")?;
  Ok(pet.permissions.clone().unwrap_or_else(||defaults().pets[id].permissions.clone().unwrap_or_default()))
 }
}
pub fn require(id:&str,capability:&str)->Result<(),String>{
 require_active(id)?;let c=read()?;let p=c.permissions(id)?;
 let allowed=match capability{"documents"=>p.documents,"emailDrafts"=>p.email_drafts,"music"=>p.music,"openApps"=>p.open_apps,_=>false};
 if allowed{Ok(())}else{Err(format!("{} não tem acesso a {capability}. Altera as permissões na gestão deste pet.",c.pets[id].name))}
}
fn normalized(text:&str)->String {crate::voice::wake::normalize(text)}
fn short_text(text:&str,max:usize)->bool {!text.trim().is_empty() && text.chars().count()<=max && !text.chars().any(char::is_control)}
fn validate(c:&Configuration)->Result<(),String> {
 let base=defaults();
 if c.version!=1 || c.pets.len()!=base.pets.len() || !(200..=12000).contains(&c.reasoning_length) || c.reasoning_terms.len()>80 {return Err("Versão ou limites inválidos na configuração dos pets.".into());}
 let mut names=BTreeMap::<String,String>::new();
 for(id,p)in &base.pets {for name in std::iter::once(&p.name).chain(&p.aliases){names.insert(normalized(name),id.clone());}}
 for (id,p) in &c.pets {
  let original=base.pets.get(id).ok_or("ID de pet desconhecido. Os IDs são estáveis; edita o nome apresentado.")?;
  if p.provider!=original.provider {return Err(format!("O provider de {id} identifica a sua conta nativa e não pode ser trocado por outra conta."));}
  if !short_text(&p.name,40) || p.aliases.len()>20 || !["auto","fast","reasoning"].contains(&p.mode.as_str()) {return Err(format!("Nome, aliases ou modo inválido em {id}."));}
  for name in std::iter::once(&p.name).chain(&p.aliases) {
   if !short_text(name,80) {return Err(format!("Nome de voz inválido em {id}."));}
   let key=normalized(name);
   if key.is_empty() || ["ola","ditado","ditar","parar conversa","desligar microfone","estado da tarefa","interrompe a tarefa"].contains(&key.as_str()) {return Err(format!("Nome de voz reservado: {name}."));}
   if let Some(owner)=names.insert(key,id.clone()) {if owner!=*id {return Err(format!("Dois pets reconhecem o nome {name}. Escolhe nomes diferentes."));}}
  }
  for profile in [&p.fast,&p.reasoning] {
   if profile.model.starts_with('@') || (id!="integration_spotify" && !matches!(p.provider.as_str(),"codex"|"copilot") && !profile.effort.is_empty()) || profile.model.len()>256 || profile.model.chars().any(char::is_control) || !["","low","medium","high","xhigh","max"].contains(&profile.effort.as_str()) {return Err(format!("Modelo ou esforço inválido em {id}."));}
   if !profile.model.is_empty() && !crate::cli_chat::model_matches(&p.provider,&profile.model) {return Err(format!("O modelo de {id} não pertence à sua conta/família."));}
   if p.provider=="ollama" && (profile.model.ends_with(":cloud") || profile.model.ends_with("-cloud")) {return Err("O pet Ollama usa apenas os modelos locais.".into());}
  }
  if let Some(policy)=&p.local_execution {if id!="ai_ollama" {return Err("A execução local só está disponível no pet Ollama.".into());}policy.validate()?;}
 }
 if c.reasoning_terms.iter().any(|s|!short_text(s,100)) {return Err("Termo de raciocínio inválido.".into());}
 Ok(())
}
fn bytes(path:&Path)->Result<Vec<u8>,String> {
 let mut options=std::fs::OpenOptions::new();options.read(true);
 #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW);}
 let file=options.open(path).map_err(|e|format!("Não foi possível ler pets.json: {e}"))?;
 if !file.metadata().map_err(|e|e.to_string())?.is_file(){return Err("pets.json deve ser um ficheiro normal.".into());}
 let mut bytes=Vec::new();file.take(MAX_BYTES+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len() as u64>MAX_BYTES {return Err("pets.json excede 256 KiB.".into());}Ok(bytes)
}
fn revision(data:&[u8])->String {let mut h=std::collections::hash_map::DefaultHasher::new();data.hash(&mut h);format!("{:016x}",h.finish())}
fn write(path:&Path,c:&Configuration)->Result<(),String> {
 validate(c)?;let dir=path.parent().ok_or("Falta a pasta dos pets.")?;
 std::fs::create_dir_all(dir).map_err(|e|e.to_string())?;
 if std::fs::symlink_metadata(dir).is_ok_and(|m|m.file_type().is_symlink()) || std::fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()) {return Err("A configuração dos pets não aceita symlinks.".into());}
 let data=serde_json::to_vec_pretty(c).map_err(|e|e.to_string())?;
 if data.len() as u64>MAX_BYTES {return Err("Configuração demasiado grande.".into());}
 static SEQ:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
 let temp=path.with_extension(format!("json.{}-{}.tmp",std::process::id(),SEQ.fetch_add(1,std::sync::atomic::Ordering::Relaxed)));
 let result=(||->Result<(),String>{
  let mut o=std::fs::OpenOptions::new();o.write(true).create_new(true);
  #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;o.mode(0o600).custom_flags(libc::O_NOFOLLOW);}
  let mut f=o.open(&temp).map_err(|e|e.to_string())?;f.write_all(&data).and_then(|_|f.write_all(b"\n")).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;drop(f);
  std::fs::rename(&temp,path).map_err(|e|e.to_string())?;Ok(())
 })();if result.is_err(){let _=std::fs::remove_file(temp);}result
}
fn read_at(path:&Path)->Result<Document,String> {read_at_policy(path,true)}
fn read_at_policy(path:&Path,create_defaults:bool)->Result<Document,String> {
 if create_defaults && !path.exists() {write(path,&defaults())?;}
 let data=bytes(path)?;let mut c:Configuration=serde_json::from_slice(&data).map_err(|e|format!("Corrige pets.json: {e}"))?;
 // Upgrade the existing Copilot identity without replacing names, aliases or grants.
 if let Some(p)=c.pets.get_mut("agent_copilot") {if p.provider.is_empty(){let base=defaults();p.provider="copilot".into();if p.fast.model.is_empty(){p.fast=base.pets["agent_copilot"].fast.clone();}if p.reasoning.model.is_empty(){p.reasoning=base.pets["agent_copilot"].reasoning.clone();}}}
 // Spoty is an audio owner. Preserve its ID, names, grants and old profiles/history.
 if let Some(p)=c.pets.get_mut("integration_spotify"){if p.provider=="codex"{p.provider.clear();}}
 // Retain stable IDs/history while retiring the duplicated Gravity model pets.
 for(id,legacy)in[("agent_gemini","gemini-cli"),("integration_claude","claude-cli")]{if let Some(p)=c.pets.get_mut(id){if p.provider==legacy{p.provider.clear();}}}
 validate(&c)?;
 let base=defaults();for(id,p)in &mut c.pets{if p.permissions.is_none(){p.permissions=base.pets[id].permissions.clone();}}
 Ok(Document{config:c,path:path.to_string_lossy().to_string(),revision:revision(&data)})
}
pub fn read()->Result<Configuration,String> {let _edit=EDIT.lock().unwrap();Ok(read_at(&path())?.config)}
/// Read existing identities with the same in-memory upgrades/validation, without
/// creating defaults, directories or files when configuration is unavailable.
pub(crate) fn read_existing()->Result<Configuration,String> {
 let _edit=EDIT.lock().map_err(|_|"As identidades dos pets estão indisponíveis.".to_string())?;
 Ok(read_at_policy(&path(),false)?.config)
}
#[cfg(test)]
#[path = "pet_config_read_tests.rs"]
mod read_existing_tests;
pub fn identity_name(id:&str)->String {read().ok().and_then(|c|c.pets.get(id).map(|p|p.name.clone())).or_else(||defaults().pets.get(id).map(|p|p.name.clone())).unwrap_or_else(||id.into())}
pub fn aliases()->Result<BTreeMap<String,Vec<String>>,String> {
 let config=read()?;Ok(config.pets.into_iter().filter(|(id,p)|!retired(id)&&(id.as_str()=="integration_spotify"||!p.provider.is_empty())).map(|(id,mut p)|{p.aliases.push(p.name);(if service(&id){id}else{p.provider},p.aliases)}).collect())
}
impl Configuration {
 /// An explicitly addressed monitor/API pet must never borrow the selected CLI.
 pub fn unavailable_address(&self,text:&str)->Option<String> {
  let normalized_text=normalized(text);
  let mut text=normalized_text.as_str();
  for prefix in ["ola ","hey ","ok "] {if let Some(rest)=text.strip_prefix(prefix){text=rest;break;}}
  for prefix in ["chama o ","chama ","muda para ","troca para ","falar com "] {if let Some(rest)=text.strip_prefix(prefix){text=rest;break;}}
  let mut addressed:Vec<_>=self.pets.iter().flat_map(|(id,p)|std::iter::once(&p.name).chain(&p.aliases).map(move|name|(normalized(name),id,p))).filter(|(name,_,_)|!name.is_empty() && (text==name || text.strip_prefix(name.as_str()).is_some_and(|rest|rest.starts_with(' ')))).collect();
  addressed.sort_by_key(|(name,_,_)|std::cmp::Reverse(name.len()));
  addressed.first().filter(|(_,id,p)|retired(id)||p.provider.is_empty()).map(|(_,_,p)|p.name.clone())
 }
 pub fn select(&self,provider:&str,requested:&str,query:&str)->Result<Selection,String> {
  require_active_provider(provider)?;let id=pet_id(provider).ok_or("Este pet não tem perfil de modelos.")?;let p=self.pets.get(id).ok_or("Falta o perfil deste pet.")?;
  if id=="integration_spotify"{return Err("O Spoty controla música e som e não tem perfil de modelos.".into());}
  if p.provider.is_empty(){return Err("Os modelos Gemini e Claude da conta Gravity estão no pet Gravity. Escolhe o modelo nesse pet.".into());}
  if !requested.is_empty() && !requested.starts_with('@') {return Ok(Selection{model:requested.into(),effort:String::new(),mode:"manual".into()});}
  let requested=if requested=="@default" || requested.is_empty(){p.mode.as_str()}else{requested.strip_prefix('@').unwrap_or(&p.mode)};
  let text=normalized(query);
  let deep=if text.starts_with("modo rapido ") || text.starts_with("resposta rapida ") || text.starts_with("fast mode ") {false} else {text.starts_with("modo raciocinio ") || self.reasoning_terms.iter().any(|term|text.contains(&normalized(term))) || query.chars().count()>=self.reasoning_length};
  let explicit=if text.starts_with("modo rapido ") || text.starts_with("fast mode "){Some("fast")}else if text.starts_with("modo raciocinio ") || text.starts_with("reasoning mode "){Some("reasoning")}else{None};
  let mode=match explicit.unwrap_or(requested) {"auto"=>if deep {"reasoning"} else {"fast"},"fast"=>"fast","reasoning"=>"reasoning",_=>return Err("Modo de modelo desconhecido.".into())};
  let profile=if mode=="reasoning" {&p.reasoning} else {&p.fast};
  if profile.model.is_empty(){return Err(format!("Escolhe um modelo {mode} para {} no centro de tarefas.",p.name));}
  Ok(Selection{model:profile.model.clone(),effort:profile.effort.clone(),mode:mode.into()})
 }
}
pub fn select(provider:&str,requested:&str,query:&str)->Result<Selection,String> {read()?.select(provider,requested,query)}
#[tauri::command]
pub fn pet_configuration()->Result<Document,String> {let _edit=EDIT.lock().unwrap();read_at(&path())}
#[tauri::command]
pub fn pet_roster(shared:tauri::State<'_,crate::Shared>)->Vec<String>{
 let settings=shared.settings.lock().unwrap();let base=defaults();let mut ids=vec![settings.main_pill.clone()];
 for id in &settings.active_integrations{if !ids.contains(id){ids.push(id.clone());}}
 ids.retain(|id|base.pets.contains_key(id)&&!retired(id));ids
}
fn save_at(file:&Path,config:&Configuration,expected:&str)->Result<Document,String> {
 let current=read_at(file)?;
 if current.revision!=expected {return Err("pets.json foi alterado entretanto. Reabre a configuração antes de guardar.".into());}
 write(file,config)?;read_at(file)
}
#[tauri::command]
pub fn pet_configuration_save(app:AppHandle,config:Configuration,expected_revision:String)->Result<Document,String> {
 let _edit=EDIT.lock().unwrap();let doc=save_at(&path(),&config,&expected_revision)?;let _=app.emit("pet-configuration-changed",&doc.config);Ok(doc)
}

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn integration_grants_are_specific_revocable_and_email_defaults_only_to_ollama(){
  let mut c=defaults();for(id,pet)in &c.pets{assert_eq!(pet.permissions.as_ref().unwrap().email_drafts,id=="ai_ollama","{id}");}
  assert!(c.permissions("ai_ollama").unwrap().documents);assert!(c.permissions("ai_ollama").unwrap().email_drafts);
  c.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().email_drafts=false;assert!(!c.permissions("ai_ollama").unwrap().email_drafts);
  c.pets.get_mut("ai_ollama").unwrap().permissions=None;assert!(c.permissions("ai_ollama").unwrap().email_drafts);
  assert!(c.permissions("unknown").is_err());
  let mut value=serde_json::to_value(&c).unwrap();value["pets"]["agent_codex"]["permissions"]["nativeCliAutoApprove"]=serde_json::json!(true);assert!(serde_json::from_value::<Configuration>(value).is_err());
 }
 #[test]fn quick_and_deep_requests_choose_native_profiles_without_permission_fields(){
  let c=defaults();validate(&c).unwrap();
  let fast=c.select("codex","@auto","Responde apenas olá").unwrap();assert_eq!(fast.model,"gpt-6-luna");assert_eq!(fast.effort,"low");
  let deep=c.select("codex","@auto","Pensa a fundo na arquitetura deste serviço").unwrap();assert_eq!(deep.model,"gpt-6.1-sol");assert_eq!(deep.effort,"high");
  assert_eq!(c.select("codex","@fast","auditoria de segurança").unwrap().mode,"fast");
  assert_eq!(c.select("antigravity","@auto","abre o Spotify").unwrap().model,"gemini-3.8-flash-low");
  assert!(c.select("gemini-cli","@reasoning","olá").is_err());assert!(c.select("claude-cli","@fast","olá").is_err());
  assert_eq!(c.select("codex","chosen-model","olá").unwrap().mode,"manual");
 }
 #[test]fn every_callable_pet_has_two_profiles_and_keeps_its_own_spoken_owner(){
  let c=defaults();let aliases:BTreeMap<_,_>=c.pets.iter().filter(|(id,p)|id.as_str()=="integration_spotify"||!p.provider.is_empty()).map(|(id,p)|(if service(id){id.clone()}else{p.provider.clone()},p.aliases.clone())).collect();
  for pet in crate::voice::dialogue::Pet::ALL {
   let owner=pet.provider();let id=pet_id(owner).unwrap();
   if retired(id)||c.pets[id].provider.is_empty(){continue;}
   for mode in ["@fast","@reasoning"] {assert!(!c.select(owner,mode,"olá").unwrap().model.is_empty(),"{id}");}
   for name in std::iter::once(&c.pets[id].name).chain(&c.pets[id].aliases) {
    assert!(matches!(crate::voice::dialogue::parse_with_aliases(&format!("{name}, check the logs por favor"),&aliases),crate::voice::dialogue::Command::Select(selected,ref query) if selected==pet && query=="check the logs por favor"),"{id}: {name}");
   }
  }
 }
 #[test]fn explicit_voice_modes_and_unavailable_addresses_do_not_fall_into_another_pet(){
  let mut c=defaults();c.pets.get_mut("agent_codex").unwrap().mode="fast".into();
  assert_eq!(c.select("codex","@default","modo raciocínio analisa este problema").unwrap().mode,"reasoning");
  assert_eq!(c.select("codex","@reasoning","modo rápido responde olá").unwrap().mode,"fast");
  assert_eq!(c.unavailable_address("Olá Copilot, abre a app"),None);
  assert_eq!(c.unavailable_address("Olá Cursor, abre a app"),Some("Cursor".into()));
  assert_eq!(c.unavailable_address("Claude Desktop, abre a app"),Some("Claude Desktop".into()));
  assert_eq!(c.unavailable_address("explica como funciona o Copilot"),None);
  assert_eq!(c.unavailable_address("GitHub, verifica o CI"),Some("GitHub".into()));
 }
 #[test]fn identities_remain_stable_and_ambiguous_names_or_cloud_local_models_are_rejected(){
  let mut c=defaults();c.pets.get_mut("agent_codex").unwrap().name="Cody".into();validate(&c).unwrap();assert_eq!(c.pets["agent_codex"].provider,"codex");
  c.pets.get_mut("agent_antigravity").unwrap().aliases.push("Cody".into());assert!(validate(&c).is_err());
  let mut c=defaults();c.pets.get_mut("ai_ollama").unwrap().fast.model="model:cloud".into();assert!(validate(&c).is_err());
  let mut c=defaults();c.pets.get_mut("agent_codex").unwrap().provider="openai".into();assert!(validate(&c).is_err());
 }
 #[test]fn configuration_roundtrip_keeps_literal_names_and_rejects_unknown_fields(){
  let root=std::env::temp_dir().join(format!("coucou-pet-prefs-{}",std::process::id()));std::fs::create_dir_all(&root).unwrap();let path=root.join("pets.json");
  let first=read_at(&path).unwrap();assert_eq!(first.config.pets.len(),defaults().pets.len());
  let mut edited=first.config;edited.pets.get_mut("agent_codex").unwrap().name="Cody".into();write(&path,&edited).unwrap();let second=read_at(&path).unwrap();assert_ne!(first.revision,second.revision);assert_eq!(second.config.pets["agent_codex"].name,"Cody");
  let changed=bytes(&path).unwrap();assert!(save_at(&path,&defaults(),&first.revision).is_err());assert_eq!(bytes(&path).unwrap(),changed);
  let mut invalid=serde_json::to_value(&second.config).unwrap();invalid["approvalPolicy"]=serde_json::json!("never");assert!(serde_json::from_value::<Configuration>(invalid).is_err());
  std::fs::remove_dir_all(root).unwrap();
 }
}

#[cfg(test)]mod spoty_audio_tests{
 use super::*;
 #[test]fn spoty_retains_identity_and_profiles_without_an_llm_backend(){
  let config=defaults();let pet=&config.pets["integration_spotify"];assert!(pet.provider.is_empty());assert!(pet.aliases.iter().any(|a|a=="spoty"));
  validate(&config).unwrap();assert!(config.select("integration_spotify","@fast","pausa").is_err());assert_eq!(crate::cli_chat::backend("integration_spotify"),None);
  assert!(matches!(crate::voice::dialogue::parse("Spoty, silenciar sistema"),crate::voice::dialogue::Command::Select(crate::voice::dialogue::Pet::Spotify,ref query)if query=="silenciar sistema"));
 }
 #[test]fn legacy_spotify_config_is_upgraded_in_memory_without_overwriting_private_file(){
  let dir=std::env::temp_dir().join(format!("coucou-spoty-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  std::fs::create_dir(&dir).unwrap();let path=dir.join("pets.json");let mut config=defaults();let p=config.pets.get_mut("integration_spotify").unwrap();p.provider="codex".into();p.name="Som pessoal".into();p.aliases.push("audio pessoal".into());let fast=p.fast.clone();let grants=p.permissions.clone();
  let bytes=serde_json::to_vec(&config).unwrap();std::fs::write(&path,&bytes).unwrap();let doc=read_at(&path).unwrap();let pet=&doc.config.pets["integration_spotify"];
  assert!(pet.provider.is_empty());assert_eq!(pet.name,"Som pessoal");assert!(pet.aliases.iter().any(|a|a=="audio pessoal"));assert_eq!(pet.fast,fast);assert_eq!(pet.permissions,grants);assert_eq!(std::fs::read(&path).unwrap(),bytes);
  std::fs::remove_dir_all(dir).unwrap();
 }
}

#[cfg(test)]mod retirement_tests {
 use super::*;
 #[test]fn retired_pet_cannot_select_even_an_explicit_model_or_borrow_another_owner(){
  let mut c=defaults();let before=serde_json::to_value(&c).unwrap();
  for model in ["@auto","@fast","@reasoning","chosen-model",""]{assert!(c.select("integration_github",model,"pedido").is_err());}
  assert_eq!(serde_json::to_value(&c).unwrap(),before);validate(&c).unwrap();
  c.pets.get_mut("integration_github").unwrap().name="Git antigo".into();
  assert_eq!(c.unavailable_address("Git antigo, abre issues"),Some("Git antigo".into()));
  assert_eq!(c.unavailable_address("github, verifica CI"),Some("Git antigo".into()));
  assert_eq!(c.unavailable_address("explica GitHub"),None);
  assert_eq!(c.unavailable_address("Copilot CLI, ajuda"),None);
 }
 #[test]fn history_classification_and_sibling_models_remain_available(){
  assert!(service("integration_github"));assert_eq!(crate::cli_chat::backend("integration_github"),Some("codex"));
  assert_eq!(pet_id("integration_github"),Some("integration_github"));
  assert!(crate::assistant_memory::supported("integration_github"));
  for provider in ["codex","copilot","antigravity","ollama","integration_notion","integration_vercel"]{
   assert!(require_active_provider(provider).is_ok());assert!(defaults().select(provider,"@fast","olá").is_ok(),"{provider}");
  }
  // Before reading HOME preferences or touching a dropped file.
  assert!(matches!(crate::pet_files::ingest("integration_github","/not/a/real/file"),Err(error)if error.contains("temporariamente")));
  assert!(require("integration_github","documents").unwrap_err().contains("temporariamente"));
 }
}
