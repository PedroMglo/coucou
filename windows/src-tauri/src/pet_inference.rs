//! Explicit local inference selection, independent of logical pet ownership.
//! Pure resolution/validation performs no IO, credential lookup or model load.
use serde::{Deserialize,Serialize};
use crate::pet_config::{Adapter,Blueprint,Configuration,RuntimeKind};
use std::sync::{Arc,Mutex,Weak,atomic::{AtomicBool,Ordering}};
use crate::settings::Settings;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Backend{Ollama,Lmstudio}
impl Backend{
 pub fn key(self)->&'static str{match self{Self::Ollama=>"ollama",Self::Lmstudio=>"lmstudio"}}
 pub fn adapter(self)->Adapter{match self{Self::Ollama=>Adapter::Ollama,Self::Lmstudio=>Adapter::Lmstudio}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum CredentialReference{None,ConfiguredLmstudioEndpoint}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Choice{
 pub schema_version:u32,pub backend:Backend,pub endpoint:String,pub model:String,
 pub credential:CredentialReference,
}
fn invalid()->String{"O binding de inferência é incompleto ou não é suportado por este pet. Escolhe explicitamente motor, servidor, modelo e referência de credencial.".into()}
pub(crate)fn revision_key(value:&str)->bool{value.len()==32&&value.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))}
pub(crate)fn model_key(value:&str)->bool{
 !value.is_empty()&&value.len()<=256&&!value.starts_with(['@','/','\\','~'])&&!value.contains(['?','#','\\'])&&!value.contains("://")&&!value.chars().any(|c|c.is_control()||c.is_whitespace())
  &&!std::path::Path::new(value).components().any(|c|matches!(c,std::path::Component::ParentDir))
}
impl Choice{
 pub fn validate(&self,blueprint:&Blueprint)->Result<(),String>{
  if self.schema_version!=1||blueprint.retired||blueprint.runtime.kind!=RuntimeKind::Llm
   ||!blueprint.runtime.inference_adapters.contains(&self.backend.adapter())||!model_key(&self.model)
   ||self.backend==Backend::Ollama&&(self.model.ends_with(":cloud")||self.model.ends_with("-cloud")||self.credential!=CredentialReference::None)
   ||self.endpoint.is_empty()||self.endpoint.len()>4096||self.endpoint.trim()!=self.endpoint{return Err(invalid());}
  let url=reqwest::Url::parse(&self.endpoint).map_err(|_|invalid())?;
  if !matches!(url.scheme(),"http"|"https")||!crate::net::is_loopback_url(&url)||!url.username().is_empty()||url.password().is_some()||url.query().is_some()||url.fragment().is_some()||url.port()==Some(0)
   ||url.as_str().trim_end_matches('/')!=self.endpoint.trim_end_matches('/'){return Err(invalid());}
  Ok(())
 }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate)struct Target{
 pub pet_id:String,pub provider:String,pub choice:Choice,pub revision:String,
}
impl Target{
 /// The legacy logical channel is stable, even for a different selected engine.
 pub(crate)fn configured(config:&Configuration,pet_id:&str)->Result<Option<Self>,String>{
  let pet=config.pets.get(pet_id).ok_or_else(invalid)?;
  if !matches!(config.version,1|2)||pet.inference_revision.as_ref().is_some_and(|r|config.version!=2||!revision_key(r)){return Err(invalid());}
  let Some(choice)=pet.inference.as_ref()else{return Ok(None);};
  let blueprint=crate::pet_config::blueprint(pet_id).ok_or_else(invalid)?;choice.validate(blueprint)?;
  let provider=blueprint.routing.task_provider.as_ref().filter(|p|!p.is_empty()).ok_or_else(invalid)?;
  let revision=pet.inference_revision.as_ref().filter(|r|revision_key(r)).ok_or_else(invalid)?;
  if config.version!=2{return Err(invalid());}
  Ok(Some(Self{pet_id:pet_id.into(),provider:provider.clone(),choice:choice.clone(),revision:revision.clone()}))
 }
 pub(crate)fn for_provider(config:&Configuration,provider:&str)->Result<Option<Self>,String>{
  match crate::pet_config::pet_id(provider){Some(id)=>Self::configured(config,id),None=>Ok(None)}
 }
 /// Conservative physical server exclusion. Protocol paths/models/credentials
 /// cannot create parallel ownership of the same local server origin.
 pub(crate)fn resource_key(&self)->String{
  physical_key(&self.choice.endpoint).expect("validated target")
 }
}
fn physical_key(endpoint:&str)->Result<String,String>{
 let url=crate::net::normalise_server_url(endpoint)?;
 // Paths, credentials, models and loopback host aliases cannot bypass server
 // exclusion. Different loopback listeners on one port may over-serialize.
 let origin=if crate::net::is_loopback_url(&url){format!("loopback:{}",url.port_or_known_default().ok_or_else(invalid)?)}else{url.origin().ascii_serialization()};
 let digest=ring::digest::digest(&ring::digest::SHA256,origin.as_bytes());
 Ok(format!("inference-{}",digest.as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>()))
}
pub(crate)fn managed_provider(provider:&str)->bool{
 provider=="custom"||crate::pet_config::pet_id(provider).and_then(crate::pet_config::blueprint).is_some_and(|b|b.runtime.kind==RuntimeKind::Llm&&!b.runtime.inference_adapters.is_empty())
}
/// Passive physical projection. No credentials, model lookup or profile IO.
pub(crate)fn resource_for(config:&Configuration,settings:&Settings,provider:&str)->Result<Option<String>,String>{
 if !managed_provider(provider){return Ok(None);}
 if let Some(target)=Target::for_provider(config,provider)?{return Ok(Some(target.resource_key()));}
 let endpoint=match provider{"ollama"=>&settings.ollama_url,"lmstudio"=>&settings.lmstudio_url,"custom"=>&settings.custom_url,_=>return Ok(None)};
 physical_key(endpoint).map(Some)
}
/// Immutable native reservation metadata, not an authority/grant or protocol
/// capability. Secret values are resolved only by the active transport.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum CaptureObservation{Current,Busy,Changed}
pub(crate)struct Captured{
 pub(crate)provider:String,pub(crate)adapter:&'static str,pub(crate)endpoint:String,
 pub(crate)explicit:Option<Target>,pub(crate)configuration:Arc<Configuration>,
 pub(crate)credential_revision:Option<u64>,source:Weak<Mutex<Settings>>,retired:AtomicBool,
 #[cfg(test)]profile:Option<std::path::PathBuf>,
}
impl Captured{
 pub(crate)fn capture(source:&Arc<Mutex<Settings>>,provider:&str)->Result<Option<Arc<Self>>,String>{
  if !managed_provider(provider){return Ok(None);}
  let settings=source.try_lock().map_err(|_|"A configuração de inferência está ocupada.")?.clone();
  let configuration=if provider=="custom"{Arc::new(crate::pet_config::defaults())}else{crate::pet_config::resource_snapshot()?};
  Self::resolve(configuration,&settings,provider,Arc::downgrade(source)).map(|c|Some(Arc::new(c)))
 }
 fn resolve(configuration:Arc<Configuration>,settings:&Settings,provider:&str,source:Weak<Mutex<Settings>>)->Result<Self,String>{
  let explicit=Target::for_provider(&configuration,provider)?;
  let(adapter,endpoint,uses_key)=if let Some(target)=&explicit{
   (target.choice.backend.key(),target.choice.endpoint.clone(),target.choice.credential==CredentialReference::ConfiguredLmstudioEndpoint)
  }else{match provider{
   "ollama"=>("ollama",settings.ollama_url.clone(),false),
   "lmstudio"=>("lmstudio",settings.lmstudio_url.clone(),true),
   "custom"=>("custom",settings.custom_url.clone(),false),_=>return Err(invalid()),
  }};
  physical_key(&endpoint)?;
  let credential_revision=(adapter=="lmstudio"&&uses_key).then(crate::local_chat::lmstudio_key_generation);
  if credential_revision.is_some_and(|r|r%2!=0){return Err("A credencial está a ser alterada. Volta a iniciar o pedido.".into());}
  Ok(Self{provider:provider.into(),adapter,endpoint,explicit,configuration,credential_revision,source,retired:AtomicBool::new(false),#[cfg(test)]profile:None})
 }
 #[cfg(test)]pub(crate)fn capture_fixture(source:&Arc<Mutex<Settings>>,provider:&str,profile:&std::path::Path)->Result<Option<Arc<Self>>,String>{
  if !managed_provider(provider){return Ok(None);}
  let settings=source.try_lock().map_err(|_|"fixture settings busy")?.clone();
  let config=Arc::new(crate::pet_config::read_existing_at(profile)?);
  let mut capture=Self::resolve(config,&settings,provider,Arc::downgrade(source))?;capture.profile=Some(profile.into());Ok(Some(Arc::new(capture)))
 }
 fn current_configuration(&self)->Result<Configuration,String>{
  #[cfg(test)]if let Some(path)=&self.profile{return crate::pet_config::read_existing_at(path);}
  if self.provider=="custom"{Ok(crate::pet_config::defaults())}else{crate::pet_config::read_existing()}
 }
 pub(crate)fn resource_key(&self)->Result<String,String>{physical_key(&self.endpoint)}
 pub(crate)fn current(&self)->bool{self.observation()==CaptureObservation::Current}
 pub(crate)fn observation(&self)->CaptureObservation{
  if self.retired.load(Ordering::Acquire){return CaptureObservation::Changed;}
  let retire=||{self.retired.store(true,Ordering::Release);CaptureObservation::Changed};
  let Some(source)=self.source.upgrade()else{return retire();};
  let settings=match source.try_lock(){Ok(settings)=>settings.clone(),Err(std::sync::TryLockError::WouldBlock)=>return CaptureObservation::Busy,Err(_)=>return retire()};
  let Ok(configuration)=self.current_configuration()else{return retire();};
  let Ok(now)=Self::resolve(Arc::new(configuration),&settings,&self.provider,Arc::downgrade(&source))else{return retire();};
  let same=self.adapter==now.adapter&&self.endpoint==now.endpoint&&self.explicit==now.explicit&&self.credential_revision==now.credential_revision
   &&crate::pet_config::pet_id(&self.provider).is_none_or(|id|serde_json::to_vec(&self.configuration.pets.get(id)).ok()==serde_json::to_vec(&now.configuration.pets.get(id)).ok());
  if !same{return retire();}if self.retired.load(Ordering::Acquire){CaptureObservation::Changed}else{CaptureObservation::Current}
 }
 pub(crate)fn check(&self)->Result<(),String>{if self.current(){Ok(())}else{Err("O destino ou a política de inferência mudou. O pedido original não foi redirecionado.".into())}}
}
#[cfg(test)]#[path="pet_inference_tests.rs"]mod tests;
