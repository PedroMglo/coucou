//! Explicit project identity and native conversation membership in the existing
//! memory catalog. This data management API grants no model or execution use.
use super::model::{Registry,Owner,Scope,text};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Project{
 pub id:String,pub pet_id:String,pub creation_request_id:String,pub revision:u64,pub name:String,
 pub conversation_ids:Vec<String>,pub enabled:bool,pub created_at:u64,pub updated_at:u64,
}
impl Project{
 pub(crate)fn valid(&self)->bool{
  let mut ids=std::collections::BTreeSet::new();
  crate::assistant_memory::valid_id(&self.creation_request_id)&&project_id(&self.pet_id,&self.creation_request_id).as_deref()==Ok(self.id.as_str())&&(Owner{pet_id:self.pet_id.clone(),scope:Scope::Pet{}}).valid()
   &&crate::pet_config::blueprint(&self.pet_id).is_some()&&self.revision>0&&text(&self.name,160,false)
   &&self.created_at>0&&self.updated_at>=self.created_at&&self.conversation_ids.len()<=64
   &&self.conversation_ids.iter().all(|c|crate::assistant_memory::valid_id(c)&&ids.insert(c))
 }
}
fn project_id(pet:&str,request:&str)->Result<String,&'static str>{
 let hash=super::context_model::digest(&serde_json::to_vec(&("coucou-memory-project-v1",pet,request)).map_err(|_|"invalidProjectInput")?);Ok(hash[..32].into())
}
#[derive(Clone,Debug,Deserialize)]
#[serde(tag="operation",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub(crate)enum Mutation{
 Create{request_id:String,name:String,conversation_ids:Vec<String>},
 Rename{id:String,expected_project_revision:u64,name:String},
 SetConversation{id:String,expected_project_revision:u64,conversation_id:String,attached:bool},
 SetEnabled{id:String,expected_project_revision:u64,enabled:bool},
}
impl Mutation{
 fn added_conversations(&self)->Vec<&str>{match self{
  Self::Create{conversation_ids,..}=>conversation_ids.iter().map(String::as_str).collect(),
  Self::SetConversation{conversation_id,attached:true,..}=>vec![conversation_id],_=>vec![],
 }}
}
impl Registry{
 pub(super)fn apply_project(&self,pet:&str,mutation:Mutation,now:u64)->Result<(Self,String),&'static str>{
  self.validate()?;if now==0||!(Owner{pet_id:pet.into(),scope:Scope::Pet{}}).valid()||crate::pet_config::blueprint(pet).is_none(){return Err("invalidOwner");}
  let mut next=self.clone();next.version=2;
  let id=match mutation{
   Mutation::Create{request_id,name,conversation_ids}=>{
    if !crate::assistant_memory::valid_id(&request_id){return Err("invalidProjectInput");}
    let id=project_id(pet,&request_id)?;
    if next.projects.iter().any(|p|p.id==id){return Err("projectAlreadyExists");}
    if next.projects.len()>=64{return Err("registryLimit");}
    next.projects.push(Project{id:id.clone(),pet_id:pet.into(),creation_request_id:request_id,revision:1,name,conversation_ids,enabled:true,created_at:now,updated_at:now});id
   },
   other=>{
    let(id,expected)=match &other{Mutation::Rename{id,expected_project_revision,..}|Mutation::SetConversation{id,expected_project_revision,..}|Mutation::SetEnabled{id,expected_project_revision,..}=>(id,*expected_project_revision),_=>unreachable!()};
    let p=next.projects.iter_mut().find(|p|p.id==*id&&p.pet_id==pet).ok_or("projectNotFound")?;
    if p.revision!=expected{return Err("projectRevisionConflict");}if now<p.updated_at{return Err("clockChanged");}
    match other{
     Mutation::Rename{name,..}=>{if p.name==name{return Err("projectUnchanged");}p.name=name;},
     Mutation::SetEnabled{enabled,..}=>{if p.enabled==enabled{return Err("projectUnchanged");}p.enabled=enabled;},
     Mutation::SetConversation{conversation_id,attached,..}=>{
      if !crate::assistant_memory::valid_id(&conversation_id){return Err("invalidProjectInput");}
      let present=p.conversation_ids.iter().any(|c|*c==conversation_id);if present==attached{return Err("projectUnchanged");}
      if attached{p.conversation_ids.push(conversation_id);}else{p.conversation_ids.retain(|c|*c!=conversation_id);}
     },_=>unreachable!(),
    }
    p.revision=p.revision.checked_add(1).ok_or("revisionLimit")?;p.updated_at=now;p.id.clone()
   },
  };
  next.validate()?;Ok((next,id))
 }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Page{pub schema_version:u32,pub pet_id:String,pub revision:String,pub items:Vec<Project>,pub execution_authorized:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{pub schema_version:u32,pub pet_id:String,pub revision:String,pub project:Project,pub execution_authorized:bool}
fn error(code:&str)->String{match code{
 "projectRevisionConflict"|"registryConflict"|"registryChanged"=>"O projeto ou catálogo mudou. Consulta novamente antes de editar.".into(),
 "projectAlreadyExists"=>"O projeto desta escolha já existe. Consulta o catálogo; não foi criado outro projeto.".into(),
 "projectNotFound"=>"O projeto não existe neste pet.".into(),
 "invalidProjectInput"|"invalidMemoryProject"=>"O nome ou as associações do projeto são inválidos.".into(),
 "projectUnchanged"=>"Esta escolha já corresponde ao projeto atual.".into(),
 _=>super::message(code),
}}
#[cfg(target_os="linux")]
fn save_at(path:&std::path::Path,pet:&str,expected:&str,mutation:Mutation,now:u64,current:impl Fn()->Result<(),&'static str>,retain:impl Fn(&str)->Result<(),&'static str>)->Result<Receipt,&'static str>{
 current()?;
 let before=crate::private_registry::load_for_update::<Registry>(path,expected)?;
 let members=mutation.added_conversations().iter().map(|s|s.to_string()).collect::<Vec<_>>();let(next,id)=before.registry.apply_project(pet,mutation,now)?;
 let saved=crate::private_registry::save_guarded(path,expected,next,|_,_|{current()?;for cid in &members{retain(cid)?;}current()},&current)?;
 let project=saved.registry.projects.iter().find(|p|p.id==id&&p.pet_id==pet).cloned().ok_or("projectNotFound")?;
 Ok(Receipt{schema_version:1,pet_id:pet.into(),revision:saved.revision,project,execution_authorized:false})
}
#[cfg(target_os="linux")]
fn save_with_chat_at(path:&std::path::Path,pet:&str,expected:&str,mutation:Mutation,chat:&crate::chat::Chat,current:impl Fn()->Result<(),&'static str>)->Result<Receipt,&'static str>{
 let provider=crate::pet_config::blueprint(pet).and_then(|b|b.routing.chat.as_deref().or(b.routing.task_provider.as_deref())).map(str::to_string);
 save_at(path,pet,expected,mutation,crate::assistant_memory::now(),current,|cid|{
  let provider=provider.as_deref().ok_or("ownerChanged")?;
  if !chat.contains_public_conversation(provider,cid).map_err(|_|"ownerChanged")?{return Err("ownerChanged");}
  chat.retain_public_conversation(provider,cid).map_err(|_|"conversationRetentionUnavailable")
 })
}
#[tauri::command]
pub(crate)async fn pet_memory_project_list(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,pet_id:String)->Result<Page,String>{
 if !super::allowed(window.label()){return Err("Abre a gestão de memória no pet.".into());}
 #[cfg(target_os="linux")]{let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let owner=Owner{pet_id:pet_id.clone(),scope:Scope::Pet{}};super::native_current(&chat,&owner,true)?;
  let snapshot=crate::private_registry::load_at::<Registry>(&super::path())?;let items=snapshot.registry.projects.iter().filter(|p|p.pet_id==pet_id).cloned().collect();
  super::native_current(&chat,&owner,true)?;if !snapshot.unchanged(){return Err("registryChanged");}
  Ok(Page{schema_version:1,pet_id,revision:snapshot.revision,items,execution_authorized:false})
 }).await.map_err(|_|"A consulta de projetos foi interrompida.".to_string())?.map_err(error)}
 #[cfg(not(target_os="linux"))]{let _=(chat,pet_id);Err("Projetos ainda não estão disponíveis nesta plataforma.".into())}
}
#[tauri::command]
pub(crate)async fn pet_memory_project_mutate(app:tauri::AppHandle,window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,pet_id:String,expected_revision:String,mutation:Mutation)->Result<Receipt,String>{
 if !super::allowed(window.label()){return Err("Abre a gestão de memória no pet.".into());}
 #[cfg(target_os="linux")]{
  let chat=chat.inner().clone();let receipt=tauri::async_runtime::spawn_blocking(move||{
   let owner=Owner{pet_id:pet_id.clone(),scope:Scope::Pet{}};
   save_with_chat_at(&super::path(),&pet_id,&expected_revision,mutation,&chat,||super::native_current(&chat,&owner,true))
  }).await.map_err(|_|"A gravação foi interrompida. Consulta os projetos antes de repetir.".to_string())?.map_err(error)?;
  use tauri::Emitter;let _=app.emit("pet-memories-changed",serde_json::json!({"petId":receipt.pet_id,"revision":receipt.revision}));Ok(receipt)
 }
 #[cfg(not(target_os="linux"))]{let _=(app,chat,pet_id,expected_revision,mutation);Err("Projetos ainda não estão disponíveis nesta plataforma.".into())}
}

#[cfg(all(test,target_os="linux"))]#[path="project_tests.rs"]mod tests;

/// Own metadata scope for observing a project, including when no notes are
/// selected. Never a note/model grant. The original capture cannot revive after
/// an observed membership/identity change, even if old bytes are restored.
#[cfg(target_os="linux")]
pub(crate)struct Capture{path:std::path::PathBuf,source:String,project:Project,revoked:std::sync::atomic::AtomicBool}
#[cfg(target_os="linux")]
impl Capture{
 pub(crate)fn at(path:&std::path::Path,pet:&str,source:&str,choice:&super::context_model::SelectedProject,now:u64)->Result<Self,&'static str>{
  if !crate::assistant_memory::valid_id(source)||now==0||crate::pet_config::retired(pet){return Err("invalidProjectScope");}
  let snapshot=crate::private_registry::load_at::<Registry>(path)?;
  let project=snapshot.registry.projects.iter().find(|p|p.pet_id==pet&&p.id==choice.id&&p.revision==choice.revision&&p.enabled&&p.conversation_ids.iter().any(|c|c==source)).cloned().ok_or("memoryProjectChanged")?;
  if now<project.updated_at||!snapshot.unchanged(){return Err("memoryProjectChanged");}
  Ok(Self{path:path.into(),source:source.into(),project,revoked:std::sync::atomic::AtomicBool::new(false)})
 }
 pub(crate)fn project(&self)->&Project{&self.project}
 pub(crate)fn verify(&self,now:u64)->Result<(),&'static str>{
  use std::sync::atomic::Ordering;
  if self.revoked.load(Ordering::Acquire){return Err("memoryProjectChanged");}
  let current=(||{let snapshot=crate::private_registry::load_at::<Registry>(&self.path)?;
   if now<self.project.updated_at||!self.project.enabled||!self.project.conversation_ids.contains(&self.source)||!snapshot.registry.projects.iter().any(|p|p==&self.project)||!snapshot.unchanged(){return Err("memoryProjectChanged");}Ok(())})();
  // A short writer conflict is not evidence of identity revocation. Callers
  // still fail closed for this observation; no write/read allowance is renewed.
  if current.as_ref().err().is_some_and(|e|!matches!(*e,"registryBusy")){self.revoked.store(true,Ordering::Release);}current
 }
}
