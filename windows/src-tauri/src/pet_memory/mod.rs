//! Native user management of scoped memories. Not exposed as a model tool and
//! explicit model use is a separate, one-use native human data decision.
mod model;
pub(crate)mod context_model;
#[cfg(target_os="linux")]pub(crate)mod context;
#[cfg(target_os="linux")]pub(crate)mod manual;
#[cfg(target_os="linux")]pub(crate)mod native;
pub(crate)use model::{Owner,Scope,Query,Mutation,Page,Receipt,Entry};
use tauri::Emitter;
fn allowed(label:&str)->bool{[crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_control::LABEL,crate::pet_chat::LABEL].contains(&label)}
fn message(code:&str)->String{match code{
 "registryChanged"|"registryConflict"|"memoryRevisionConflict"=>"A memória mudou. Volta a consultar antes de editar.",
 "memoryNotFound"=>"Esta recordação não existe neste pet e âmbito.",
 "conversationRetentionUnavailable"=>"Não foi possível confirmar a gravação da conversa. A recordação não foi publicada; volta a consultar antes de repetir.",
 "invalidOwner"|"ownerChanged"=>"O pet ou a conversa já não correspondem ao âmbito desta memória.",
 "invalidMemoryInput"|"invalidMemoryQuery"=>"O conteúdo, validade ou limites do pedido de memória são inválidos.",
 "registryBusy"=>"Outro pedido está a guardar a memória. Aguarda e volta a consultar.",
 "registryCommitUncertain"=>"A gravação pode ter ocorrido. Volta a consultar a memória antes de repetir.",
 "registryLimit"=>"A memória atingiu o limite. Revê ou elimina recordações antes de acrescentar mais.",
 _=>"A memória está indisponível ou é inválida. Os dados anteriores foram preservados.",
}.into()}
fn validate_owner(config:&crate::pet_config::Configuration,chat:&crate::chat::Chat,owner:&Owner)->Result<(),&'static str>{
 validate_owner_for(config,chat,owner,true)
}
fn validate_owner_for(config:&crate::pet_config::Configuration,chat:&crate::chat::Chat,owner:&Owner,require_conversation:bool)->Result<(),&'static str>{
 if !owner.valid()||!config.pets.contains_key(&owner.pet_id)||crate::pet_config::retired(&owner.pet_id){return Err("invalidOwner");}
 let blueprint=crate::pet_config::blueprint(&owner.pet_id).ok_or("invalidOwner")?;
 if let Scope::Conversation{conversation_id}=&owner.scope{if require_conversation{
  let provider=blueprint.routing.chat.as_deref().or(blueprint.routing.task_provider.as_deref()).ok_or("invalidOwner")?;
  if !chat.contains_public_conversation(provider,conversation_id).map_err(|_|"ownerChanged")?{return Err("ownerChanged");}
 }}Ok(())
}
/// Called under the Chat mutex. Never calls Chat/configuration, writes a file
/// or acquires the memory writer lock. Unknown catalog state retains data.
pub(crate)fn retains_conversation(library:&std::path::Path,provider:&str,cid:&str)->bool{
 #[cfg(target_os="linux")]{
  let Some(pet)=crate::pet_config::integration_pet(provider)else{return false;};
  match crate::private_registry::load_at::<model::Registry>(&library.with_file_name("memories.json")){
   Ok(snapshot)=>!snapshot.unchanged()||snapshot.registry.items.iter().any(|e|e.owner.pet_id==pet&&matches!(&e.owner.scope,Scope::Conversation{conversation_id}if conversation_id==cid)),
   Err(_)=>true,
  }
 }
 #[cfg(not(target_os="linux"))]{let _=(library,provider,cid);false}
}
#[cfg(target_os="linux")]
fn retain_owner_conversation(chat:&crate::chat::Chat,owner:&Owner)->Result<(),&'static str>{
 if let Scope::Conversation{conversation_id}=&owner.scope{
  let blueprint=crate::pet_config::blueprint(&owner.pet_id).ok_or("invalidOwner")?;
  let provider=blueprint.routing.chat.as_deref().or(blueprint.routing.task_provider.as_deref()).ok_or("invalidOwner")?;
  chat.retain_public_conversation(provider,conversation_id).map_err(|_|"conversationRetentionUnavailable")?;
 }Ok(())
}
#[cfg(target_os="linux")]
fn path()->std::path::PathBuf{crate::platform::data_dir().join("assistant/memories.json")}
#[cfg(target_os="linux")]
fn native_current(chat:&crate::chat::Chat,owner:&Owner,require_conversation:bool)->Result<(),&'static str>{
 crate::pet_data::bootstrap::write_allowed().map_err(|_|"registryUnavailable")?;
 let config=crate::pet_config::read_existing().map_err(|_|"invalidOwner")?;
 if require_conversation{validate_owner(&config,chat,owner)}else{validate_owner_for(&config,chat,owner,false)}
}
#[cfg(target_os="linux")]
fn list_at(path:&std::path::Path,query:Query,now:u64,current:impl Fn()->Result<(),&'static str>)->Result<Page,&'static str>{
 current()?;let snapshot=crate::private_registry::load_at::<model::Registry>(path)?;
 let page=snapshot.registry.page(&query,snapshot.revision.clone(),now)?;
 current()?;if !snapshot.unchanged(){return Err("registryChanged");}Ok(page)
}
#[cfg(all(test,target_os="linux"))]
fn mutate_at(path:&std::path::Path,owner:Owner,expected:&str,mutation:Mutation,now:u64,current:impl Fn()->Result<(),&'static str>)->Result<Receipt,&'static str>{
 mutate_prepared_at(path,owner,expected,mutation,now,current,||Ok(()))
}
#[cfg(target_os="linux")]
fn mutate_prepared_at(path:&std::path::Path,owner:Owner,expected:&str,mutation:Mutation,now:u64,current:impl Fn()->Result<(),&'static str>,prepare:impl FnOnce()->Result<(),&'static str>)->Result<Receipt,&'static str>{
 current()?;
 if expected!="missing"&&(expected.len()!=64||!expected.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))){return Err("registryConflict");}
 let before=crate::private_registry::load_for_update::<model::Registry>(path,expected)?;
 let(next,id)=before.registry.apply(&owner,mutation,now)?;
 let saved=crate::private_registry::save_guarded(path,expected,next,|_,_|{current()?;prepare()?;current()},&current)?;
 let item=saved.registry.items.iter().find(|e|e.id==id&&e.owner==owner).cloned();
 Ok(Receipt{schema_version:1,owner,revision:saved.revision,id,item})
}
#[tauri::command]
pub(crate)async fn pet_memory_list(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,request:Query)->Result<Page,String>{
 if !allowed(window.label()){return Err("Abre a gestão de memória no pet.".into());}
 #[cfg(target_os="linux")]{let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{let owner=request.owner.clone();list_at(&path(),request,crate::assistant_memory::now(),||native_current(&chat,&owner,true)).map_err(message)}).await.map_err(|_|"A consulta da memória foi interrompida.".to_string())?}
 #[cfg(not(target_os="linux"))]{let _=(chat,request);Err("A memória estruturada ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub(crate)async fn pet_memory_mutate(app:tauri::AppHandle,window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,owner:Owner,expected_revision:String,mutation:Mutation)->Result<Receipt,String>{
 if !allowed(window.label()){return Err("Abre a gestão de memória no pet.".into());}
 #[cfg(target_os="linux")]{
  // Deletion may clean up an exact original record after its conversation was
  // removed. It grants no read, reassignment or new memory in the missing CID.
  let require_conversation=!matches!(&mutation,Mutation::Delete{..});let pin=matches!(&mutation,Mutation::Create{..});
  let chat=chat.inner().clone();let receipt=tauri::async_runtime::spawn_blocking(move||mutate_prepared_at(&path(),owner.clone(),&expected_revision,mutation,crate::assistant_memory::now(),||native_current(&chat,&owner,require_conversation),||if pin{retain_owner_conversation(&chat,&owner)}else{Ok(())}).map_err(message)).await.map_err(|_|"A gravação foi interrompida. Consulta a memória antes de repetir.".to_string())??;
  let _=app.emit("pet-memories-changed",serde_json::json!({"petId":receipt.owner.pet_id,"revision":receipt.revision}));Ok(receipt)
 }
 #[cfg(not(target_os="linux"))]{let _=(app,chat,owner,expected_revision,mutation);Err("A memória estruturada ainda não está disponível nesta plataforma.".into())}
}
#[cfg(all(test,target_os="linux"))]mod tests;
