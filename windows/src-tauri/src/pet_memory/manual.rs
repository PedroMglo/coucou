//! Explicit human data consent for manual requests. A review grants neither
//! agent permissions nor autonomous work. Native sessions have a durable fence.
use super::{context::{Accepted,Capture,Prepared,Reviews,Ticket},context_model::{Destination,Selected,Decision}};
use crate::{chat::Chat,settings::Settings,pet_config::Configuration};
use std::sync::{Arc,Mutex,OnceLock};

struct Resolved{destination:Destination,credential:Option<String>}
fn reviews()->&'static Mutex<Reviews>{static CACHE:OnceLock<Mutex<Reviews>>=OnceLock::new();CACHE.get_or_init(Default::default)}
fn failure(_:impl std::fmt::Display)->String{"A seleção de memória ou o seu destino mudou ou está indisponível. Revê a seleção antes de enviar; o pedido não foi autorizado por esta revisão.".into()}

/// Common factory route check. API identities deliberately have no CLI profile;
/// model-server pets additionally require their explicit configured association.
/// Pure factory/configuration projection. No keyring, filesystem, model or
/// account resolution; concrete target checks remain in the native request.
pub(crate)fn configured_route(pet:&str,p:&crate::pet_config::Pet)->Option<&'static str>{
 let b=crate::pet_config::blueprint(pet)?;if b.retired{return None;}
 let provider=b.routing.chat.as_deref()?;if crate::pet_config::integration_pet(provider)!=Some(pet){return None;}
 if ["anthropic","openai","google"].contains(&provider){return Some(provider);}
 if ["ollama","lmstudio"].contains(&provider){return(p.provider==provider).then_some(provider);}
 let adapter=b.configured_adapter(&p.provider)?.key();["codex","copilot","antigravity"].contains(&adapter).then_some(provider)
}
fn owner(config:&Configuration,provider:&str)->Result<&'static str,String>{
 let pet=crate::pet_config::integration_pet(provider).ok_or_else(||failure("owner"))?;
 let p=config.pets.get(pet).ok_or_else(||failure("configuration"))?;
 if configured_route(pet,p)!=Some(provider){return Err(failure("route"));}Ok(pet)
}

fn resolve(chat:&Chat,settings:&Settings,config:&Configuration,query:&str)->Result<Resolved,String>{
 let provider=settings.chat_provider.as_str();owner(config,provider)?;
 let requested=crate::chat::model_for(settings,provider);
 let resolved=if let Some(adapter)=crate::cli_chat::backend(provider){
  let model=config.select(provider,&requested,query)?.model;
  if !crate::cli_chat::model_matches(provider,&model){return Err(failure("model"));}
  let cwd=crate::cli_chat::workspace_for(settings,provider)?;let(_,session,_)=chat.native_memory_info(provider)?;
  Resolved{destination:super::native::destination(provider,adapter,&model,&cwd,session.as_ref().map(|s|s.id.as_str())).map_err(failure)?,credential:None}
 }else if let Some(server)=crate::local_chat::server(settings,provider){
  let model=config.select(provider,&requested,query)?.model;
  let base=crate::local_chat::base_url(&server)?;
  Resolved{destination:Destination::Server{provider:provider.into(),endpoint:crate::net::join(&base,"v1/chat/completions"),model},credential:server.key}
 }else if provider=="anthropic"{
  Resolved{destination:Destination::Api{provider:provider.into(),endpoint:crate::claude::endpoint()?.to_string(),model:requested},credential:crate::secrets::get(crate::claude::KEY)}
 }else{
  let p=crate::openai_compat::provider(provider).ok_or_else(||failure("provider"))?;
  // OpenAI calendar tools use the known sibling Responses endpoint. The
  // destination is this native provider API base, never an arbitrary caller URL.
  let endpoint=reqwest::Url::parse(p.base_url).map_err(failure)?.to_string();
  Resolved{destination:Destination::Api{provider:provider.into(),endpoint,model:requested},credential:crate::secrets::get(p.key)}
 };
 if !resolved.destination.valid()||matches!(&resolved.destination,Destination::Api{..})&&resolved.credential.is_none(){return Err(failure("destination"));}Ok(resolved)
}
fn current_owner(chat:&Chat,config:&Configuration,provider:&str,cid:&str)->Result<&'static str,String>{
 let pet=owner(config,provider)?;
 if !crate::assistant_memory::valid_id(cid)||chat.conversation_id(provider)?!=cid{return Err(failure("conversation"));}Ok(pet)
}

#[tauri::command]
pub(crate)async fn pet_memory_context_prepare(window:tauri::WebviewWindow,shared:tauri::State<'_,crate::Shared>,chat:tauri::State<'_,Chat>,provider:String,conversation_id:String,query:String,selected_model:Option<String>,selected:Vec<Selected>,context:Option<crate::chat::ChatContext>,document_sequence:Option<u64>)->Result<Ticket,String>{
 if !super::allowed(window.label()){return Err(failure("window"));}
 // Same ordering as chat_send: admission pins the CID before effective model
 // selection. All preparation is local; no model request or file mutation.
 let _reservation=chat.reserve_conversation(&provider,Some(&conversation_id))?;
 let mut settings=shared.settings.lock().map_err(failure)?.clone();settings.chat_provider=provider.clone();
 if let Some(model)=selected_model{crate::chat::select_for_turn(&mut settings,model)?;}
 let chat=chat.inner().clone();let label=window.label().to_string();
 tauri::async_runtime::spawn_blocking(move||{
  let config=crate::pet_config::read_existing()?;let pet=current_owner(&chat,&config,&provider,&conversation_id)?;
  let resolved=resolve(&chat,&settings,&config,&query)?;let now=crate::assistant_memory::now();
  let capture=Capture::at(&super::path(),pet,&conversation_id,&selected,now).map_err(failure)?;
  let context_hash=super::context::request_context_hash(context.as_ref(),document_sequence).map_err(failure)?;
  let prepared=Prepared::new(capture,resolved.destination,query,resolved.credential,now).and_then(|p|p.with_request_context(context_hash)).map_err(failure)?;
  reviews().lock().map_err(failure)?.insert(&label,prepared,crate::assistant_memory::now()).map_err(failure)
 }).await.map_err(failure)?
}
#[tauri::command]
pub(crate)fn pet_memory_context_cancel(window:tauri::WebviewWindow,token:String)->Result<(),String>{
 if !super::allowed(window.label()){return Err(failure("window"));}
 reviews().lock().map_err(failure)?.cancel(window.label(),&token).map_err(failure)
}
/// Called only after chat_send reserved the exact native CID. Cache ownership
/// is consumed before returning a non-serializable, request-local channel value.
pub(crate)fn accept(window:&str,chat:&Chat,settings:&Settings,query:&str,context_hash:&str,decision:Decision)->Result<Arc<Accepted>,String>{
 if !super::allowed(window){return Err(failure("window"));}
 let config=crate::pet_config::read_existing()?;let cid=chat.conversation_id(&settings.chat_provider)?;
 let pet=current_owner(chat,&config,&settings.chat_provider,&cid)?;let resolved=resolve(chat,settings,&config,query)?;
 let accepted={reviews().lock().map_err(failure)?.take_bound(context_hash,window,&decision.token,pet,&cid,query,&resolved.destination,resolved.credential.as_deref(),decision.consent,crate::assistant_memory::now()).map_err(failure)?}.ok_or_else(||failure("declined"))?;
 accepted.verify(&resolved.destination,query,resolved.credential.as_deref(),crate::assistant_memory::now()).map_err(failure)?;Ok(Arc::new(accepted))
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn factory_routes_do_not_borrow_observers_anonymous_or_retired_owners(){
  let mut config=crate::pet_config::defaults();
  for p in ["ollama","anthropic","openai","google"]{assert!(owner(&config,p).is_ok());}
  for p in ["codex","copilot","antigravity"]{assert!(owner(&config,p).is_ok());}
  config.pets.get_mut("integration_notion").unwrap().provider="codex".into();assert!(owner(&config,"integration_notion").is_ok());config.pets.get_mut("integration_notion").unwrap().provider.clear();
  assert!(owner(&config,"lmstudio").is_err());config.pets.get_mut("ai_lmstudio").unwrap().provider="lmstudio".into();assert_eq!(owner(&config,"lmstudio").unwrap(),"ai_lmstudio");
  for p in ["custom","openrouter","gemini-cli","claude-cli","integration_github","integration_notion","unknown"]{assert!(owner(&config,p).is_err(),"{p}");}
 }
 #[test]fn local_destination_is_actual_completion_endpoint_and_effective_profile_model(){
  let mut config=crate::pet_config::defaults();config.pets.get_mut("ai_ollama").unwrap().fast.model="synthetic-free:tag".into();
  let mut settings=Settings::default();settings.chat_provider="ollama".into();settings.ollama_url="http://127.0.0.1:11444/own-prefix".into();settings.chat_models.insert("ollama".into(),"@fast".into());
  let r=resolve(&Chat::default(),&settings,&config,"same original query").unwrap();assert_eq!(r.destination,Destination::Server{provider:"ollama".into(),endpoint:"http://127.0.0.1:11444/own-prefix/v1/chat/completions".into(),model:"synthetic-free:tag".into()});assert!(r.credential.is_none());
  config.pets.get_mut("ai_ollama").unwrap().provider.clear();assert!(resolve(&Chat::default(),&settings,&config,"same original query").is_err());
 }
}
