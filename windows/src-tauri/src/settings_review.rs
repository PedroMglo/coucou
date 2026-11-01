//! Native, bounded, one-use review of configuration DATA. No execution consent.
use super::{Settings,IoPhase,durable::{self,history},revision};
use crate::private_registry::preferences_file::{self,Witness};
use serde::{Serialize,Deserialize};
use serde_json::Value;
use std::{path::{Path,PathBuf},sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU64,Ordering}},time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
const TTL:Duration=Duration::from_secs(300);
fn refused()->String{"A revisão mudou, expirou ou foi cancelada. Prepara outra revisão; nenhum efeito anterior será repetido.".into()}
fn digest(bytes:&[u8])->String{durable::hash(bytes)}
fn bytes(settings:&Settings)->Result<Vec<u8>,String>{let value=serde_json::to_vec_pretty(settings).map_err(|_|refused())?;if value.len()>256*1024{return Err(refused());}Ok(value)}
fn random()->Result<String,String>{use ring::rand::SecureRandom;let mut b=[0;16];ring::rand::SystemRandom::new().fill(&mut b).map_err(|_|refused())?;Ok(b.iter().map(|b|format!("{b:02x}")).collect())}
#[derive(Clone)]pub(crate)struct Owner{epoch:Arc<AtomicU64>}
impl Owner{fn new()->Self{Self{epoch:Arc::new(AtomicU64::new(1))}}fn invalidate(&self){let _=self.epoch.fetch_update(Ordering::AcqRel,Ordering::Acquire,|v|v.checked_add(1));}fn same(&self,other:&Self)->bool{Arc::ptr_eq(&self.epoch,&other.epoch)}}
/// Window identity only. It cannot approve data, a component or a pet action.
#[derive(Clone)]pub(crate)struct Frame{owner:Owner,epoch:u64}
impl Frame{
 pub(crate)fn current(&self)->bool{self.epoch!=u64::MAX&&self.owner.epoch.load(Ordering::Acquire)==self.epoch}
 pub(crate)fn same(&self,other:&Self)->bool{self.epoch==other.epoch&&self.owner.same(&other.owner)}
 #[cfg(test)]pub(crate)fn invalidate_for_test(&self){self.owner.invalidate();}
}
#[derive(Clone,Debug,PartialEq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Destinations{pub revision:String,pub values:Value}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Origin{selected_path:String,settings_sha256:Option<String>,io_sha256:Option<String>}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Preview{
 schema_version:u32,token:String,fingerprint:String,expires_at_ms:u64,origin:Origin,observed_phase:String,
 values:Value,running_values:Value,built_in_defaults:Value,defaulted_fields:Vec<String>,unknown_fields:Vec<String>,unknown_field_count:usize,
 prior_recovery_provenance_uncertain:bool,pets_and_destinations:Destinations,credential_references:Vec<String>,suspended_operations:Vec<String>,scope:&'static str,
 will_preserve_current_bytes:bool,prior_outcome_will_be_retained:bool,message:&'static str,
}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Decision{token:String,fingerprint:String,reviewed_all_values:bool,understand_prior_uncertainty:bool,keep_components_suspended:bool,accept_recovered_values:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Cancellation{token:String,fingerprint:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{schema_version:u32,data_adopted:bool,retained_review:String,status:super::IoStatus,message:&'static str}
struct Capture{source:Witness,original:Option<Vec<u8>>,journal:Witness,old_io:Option<Vec<u8>>,format:Option<crate::pet_data::SettingsIoFormat>,candidate:Settings,proposed:Vec<u8>,memory_hash:String,preview:Preview,expires:Instant,owner:Owner,epoch:u64,alive:Arc<AtomicBool>}
impl Capture{
 fn authorized(&self,owner:&Owner,now:Instant)->bool{self.owner.same(owner)&&self.epoch==owner.epoch.load(Ordering::Acquire)&&self.alive.load(Ordering::Acquire)&&now<self.expires}
 fn unchanged(&self)->bool{self.source.current()&&self.journal.current()&&self.format.as_ref().is_none_or(|f|f.current())}
}
struct Slot{token:String,fingerprint:String,owner:Owner,pending:Option<Capture>,alive:Arc<AtomicBool>}
#[derive(Default)]struct State{owner:Option<Owner>,slot:Option<Slot>}
#[derive(Default)]pub(crate)struct Controller{state:Mutex<State>}
struct InFlight<'a>{controller:&'a Controller,token:String}
impl Drop for InFlight<'_>{fn drop(&mut self){if let Ok(mut state)=self.controller.state.lock(){if state.slot.as_ref().is_some_and(|s|s.token==self.token&&s.pending.is_none()){if let Some(slot)=state.slot.take(){slot.alive.store(false,Ordering::Release);}}}}}
pub(crate)fn controller()->&'static Controller{static C:OnceLock<Controller>=OnceLock::new();C.get_or_init(Controller::default)}
impl Controller{
 pub(crate)fn frame(&self)->Result<Frame,String>{let owner=self.owner()?;let epoch=owner.epoch.load(Ordering::Acquire);let frame=Frame{owner,epoch};if frame.current(){Ok(frame)}else{Err(refused())}}
 pub(crate)fn register(&self)->Owner{let owner=Owner::new();if let Ok(mut state)=self.state.lock(){if let Some(old)=state.owner.replace(owner.clone()){old.invalidate();}if let Some(slot)=state.slot.take(){slot.alive.store(false,Ordering::Release);}}owner}
 fn owner(&self)->Result<Owner,String>{self.state.lock().map_err(|_|refused())?.owner.clone().ok_or_else(refused)}
 fn prepare(&self,owner:&Owner,current:&Mutex<Settings>,path:&Path,destinations:Destinations,canonical_profile:bool)->Result<Preview,String>{
  let epoch=owner.epoch.load(Ordering::Acquire);if epoch==u64::MAX{return Err(refused());}
  let running=current.try_lock().map_err(|_|"A gravação das preferências está ocupada; a revisão não foi iniciada.".to_string())?.clone();
  let phase=super::io_status(&running).phase;
  if phase==IoPhase::Ready{return Err("Os dados já estão confirmados. A reativação das componentes segue um percurso próprio.".into());}
  let memory_hash=digest(&bytes(&running)?);let(source,original)=preferences_file::read(path,false).map_err(|e|e.to_string())?;
  let io_path=path.parent().ok_or_else(refused)?.join(durable::FILE);let(journal,old_io)=preferences_file::read(&io_path,false).map_err(|e|e.to_string())?;
  let format=crate::pet_data::bootstrap::settings_io_observation(path).map_err(|_|refused())?;
  let candidate=original.as_deref().and_then(super::parse).unwrap_or_default();let values=serde_json::to_value(&candidate).map_err(|_|refused())?;let defaults=serde_json::to_value(Settings::default()).map_err(|_|refused())?;
  let raw=original.as_deref().and_then(|b|serde_json::from_slice::<Value>(b.strip_prefix(&[0xef,0xbb,0xbf]).unwrap_or(b)).ok());
  let expected=values.as_object().ok_or_else(refused)?;
  let fields=raw.as_ref().and_then(Value::as_object);
  let complete=fields.is_some_and(|f|expected.keys().all(|key|f.contains_key(key)));
  let canonical=fields.is_some_and(|f|f.keys().eq(expected.keys()))&&original.as_deref().is_some_and(|b|serde_json::from_slice::<Value>(b).is_ok());
  let preserve=original.as_deref().is_some_and(super::fully_valid)&&complete&&(!canonical_profile||canonical);
  let proposed=if preserve{original.clone().ok_or_else(refused)?}else if !canonical_profile{
   let mut merged=fields.cloned().unwrap_or_default();for(key,value)in expected{merged.insert(key.clone(),value.clone());}
   serde_json::to_vec_pretty(&Value::Object(merged)).map_err(|_|refused())?
  }else{bytes(&candidate)?};
  if proposed.len()>256*1024||!super::fully_valid(&proposed){return Err("A configuração revista não cabe no formato suportado. Os originais foram conservados; nenhuma publicação foi iniciada.".into());}
  let mut defaulted=vec![];let mut unknown=vec![];let mut unknown_count=0;
  for name in defaults.as_object().ok_or_else(refused)?.keys(){let good=raw.as_ref().and_then(|v|v.get(name)).is_some_and(|v|{let mut one=serde_json::Map::new();one.insert(name.clone(),v.clone());serde_json::from_value::<Settings>(Value::Object(one)).is_ok()});if !good{defaulted.push(name.clone());}}
  if let Some(fields)=raw.as_ref().and_then(Value::as_object){for key in fields.keys(){if defaults.get(key).is_none(){unknown_count+=1;if unknown.len()<128{unknown.push(key.chars().take(128).collect());}}}}
  let token=random()?;let expires=Instant::now()+TTL;let expires_at_ms=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|refused())?.as_millis().checked_add(TTL.as_millis()).and_then(|n|u64::try_from(n).ok()).ok_or_else(refused)?;
  let mut preview=Preview{schema_version:1,token:token.clone(),fingerprint:String::new(),expires_at_ms,origin:Origin{selected_path:path.to_string_lossy().into_owned(),settings_sha256:source.content_hash().map(str::to_owned),io_sha256:journal.content_hash().map(str::to_owned)},observed_phase:format!("{phase:?}"),values,running_values:serde_json::to_value(&running).map_err(|_|refused())?,built_in_defaults:defaults,defaulted_fields:defaulted,unknown_fields:unknown,unknown_field_count:unknown_count,prior_recovery_provenance_uncertain:matches!(phase,IoPhase::ReviewRequired|IoPhase::ReadUnavailable)||old_io.as_deref().and_then(|b|serde_json::from_slice::<Value>(b).ok()).is_some_and(|v|v.get("reviewRequired")==Some(&Value::Bool(true))),pets_and_destinations:destinations,credential_references:crate::secrets::KNOWN_KEYS.iter().map(|s|s.to_string()).collect(),suspended_operations:durable::OPERATIONS.iter().map(|s|s.to_string()).collect(),scope:"dataOnlyKeepComponentsSuspended",will_preserve_current_bytes:preserve,prior_outcome_will_be_retained:true,message:"Revê todos os valores, defaults e destinos. Isto aprova apenas os dados para pedidos futuros autorizados. Não repete efeitos, não renova objetivos/permissões e mantém as seis componentes suspensas. A origem por campo de uma recuperação antiga pode não ser reconstituível; os valores efetivos e os defaults são apresentados integralmente. Campos desconhecidos são conservados no original retido; um perfil restaurado publica apenas os campos do seu formato canónico."};
  preview.fingerprint=digest(&serde_json::to_vec(&preview).map_err(|_|refused())?);let alive=Arc::new(AtomicBool::new(true));
  let capture=Capture{source,original,journal,old_io,format,candidate,proposed,memory_hash,preview:preview.clone(),expires,owner:owner.clone(),epoch,alive:alive.clone()};
  if !capture.authorized(owner,Instant::now())||!capture.unchanged()||digest(&bytes(&*current.try_lock().map_err(|_|refused())?)?)!=capture.memory_hash{return Err(refused());}
  let mut state=self.state.lock().map_err(|_|refused())?;if !state.owner.as_ref().is_some_and(|o|o.same(owner)){return Err(refused());}
  if state.slot.as_ref().is_some_and(|s|s.pending.is_none()){return Err("A publicação da revisão anterior ainda está em curso. Cancelar não prova que o writer terminou.".into());}
  if let Some(previous)=state.slot.take(){previous.alive.store(false,Ordering::Release);}
  state.slot=Some(Slot{token,fingerprint:preview.fingerprint.clone(),owner:owner.clone(),pending:Some(capture),alive});Ok(preview)
 }
 fn cancel(&self,owner:&Owner,request:Cancellation)->Result<(),String>{
  let mut state=self.state.lock().map_err(|_|refused())?;let slot=state.slot.as_ref().ok_or_else(refused)?;
  if request.token.len()!=32||request.fingerprint.len()!=64||slot.token!=request.token||slot.fingerprint!=request.fingerprint||!slot.owner.same(owner){return Err(refused());}
  slot.alive.store(false,Ordering::Release);if slot.pending.is_some(){state.slot=None;}Ok(())
 }
 fn apply(&self,owner:&Owner,current:&Mutex<Settings>,request:Decision,current_destination:impl Fn()->Result<Destinations,String>,window_current:impl Fn()->bool)->Result<(Settings,Receipt),String>{
  if !request.reviewed_all_values||!request.understand_prior_uncertainty||!request.keep_components_suspended||request.token.len()!=32||request.fingerprint.len()!=64{return Err("A revisão exige aceitação explícita dos valores, da incerteza anterior e da suspensão das componentes.".into());}
  let capture={let mut state=self.state.lock().map_err(|_|refused())?;let slot=state.slot.as_mut().ok_or_else(refused)?;
   if slot.token!=request.token||slot.fingerprint!=request.fingerprint||!slot.owner.same(owner){return Err(refused());}
   let c=slot.pending.as_ref().ok_or_else(refused)?;
   if (!c.preview.defaulted_fields.is_empty()||c.preview.prior_recovery_provenance_uncertain)&&!request.accept_recovered_values{return Err("Confirma também os valores recuperados/defaults apresentados.".into());}
   if !c.authorized(owner,Instant::now())||!window_current(){return Err(refused());}slot.pending.take().ok_or_else(refused)?};
  let _flight=InFlight{controller:self,token:capture.preview.token.clone()};
  let mut current=current.lock().map_err(|_|refused())?;
  let authorized=||capture.authorized(owner,Instant::now())&&window_current()&&capture.authorized(owner,Instant::now());
  let memory_current=||bytes(&current).is_ok_and(|b|digest(&b)==capture.memory_hash);
  let destinations_current=||current_destination().is_ok_and(|d|d==capture.preview.pets_and_destinations);
  if !authorized()||!memory_current()||!destinations_current()||!capture.unchanged()||!authorized(){return Err(refused());}
  let _writer=durable::writer(capture.source.selected()).map_err(|e|e.to_string())?;
  if !authorized()||!memory_current()||!destinations_current()||!capture.unchanged()||!authorized(){return Err(refused());}
  let reference=history::retain(capture.source.selected(),history::Input{id:&capture.preview.token,fingerprint:&capture.preview.fingerprint,original_settings:capture.original.as_deref(),original_io:capture.old_io.as_deref(),approved:&capture.proposed,memory_hash:&capture.memory_hash,configuration_revision:&capture.preview.pets_and_destinations.revision,configuration_values:&capture.preview.pets_and_destinations.values,observed_phase:&capture.preview.observed_phase},||if authorized()&&memory_current()&&destinations_current()&&capture.unchanged()&&authorized(){Ok(())}else{Err("reviewRevoked")}).map_err(|e|e.to_string())?;
  // Old readers must refuse before new semantics are persisted. Own format
  // observation changes here; source/journal and native authority remain bound.
  crate::pet_data::bootstrap::settings_io_review_protocol(capture.source.selected()).map_err(|_|refused())?;
  let format=crate::pet_data::bootstrap::settings_io_observation(capture.source.selected()).map_err(|_|refused())?;
  let boundary=||authorized()&&memory_current()&&destinations_current()&&capture.source.current()&&format.as_ref().is_none_or(|f|f.current())&&authorized();
  let prepared=durable::review_prepare(capture.source.selected(),&capture.journal,capture.old_io.as_deref(),capture.source.content_hash(),&capture.proposed,reference.clone(),||if boundary(){Ok(())}else{Err(std::io::Error::other(refused()))}).map_err(|error|{revision::mark_unconfirmed(&current);error.to_string()})?;
  let witness=match capture.source.commit_guarded(&capture.proposed,||if boundary()&&prepared.current(){Ok(())}else{Err(std::io::Error::other(refused()))}){
   Ok(value)=>value,
   Err(error)=>{if !preferences_file::unconfirmed(&error){let _=prepared.finish(durable::Phase::NotPublished);}revision::mark_unconfirmed(&current);return Err(format!("{error} A revisão foi consumida; consulta o estado e prepara outra, sem repetir efeitos anteriores."));}
  };
  let outcome=prepared.finish_reviewed().map_err(|error|{revision::mark_unconfirmed(&current);error.to_string()})?;
  let mut next=capture.candidate;revision::accept_review(&mut next,witness,outcome);
  // Original bytes were retained by the same reviewed transaction above.
  super::not_loaded().retain(|path|path!=capture.source.selected());
  *current=next;let status=super::io_status(&current);let receipt=Receipt{schema_version:1,data_adopted:true,retained_review:reference.id,status,message:"Dados revistos e confirmados. As componentes mantêm-se suspensas; objetivos, decisões e efeitos anteriores conservam a sua autoridade e incerteza."};Ok((current.clone(),receipt))
 }
}
// Lifecycle belongs to the native window, never a renderer-supplied label/id.
pub(crate)fn register_window(window:&tauri::WebviewWindow){
 if window.label()!=crate::HUB_LABEL{return;}
 let owner=controller().register();window.on_window_event(move|event|{
  if matches!(event,tauri::WindowEvent::CloseRequested{..}|tauri::WindowEvent::Destroyed){owner.invalidate();}
 });
}
fn permitted(window:&tauri::WebviewWindow)->bool{window.label()==crate::HUB_LABEL&&window.is_visible().unwrap_or(false)}
fn destinations()->Result<Destinations,String>{let document=crate::pet_config::pet_configuration()?;Ok(Destinations{revision:document.revision,values:serde_json::to_value(document.config).map_err(|_|refused())?})}
#[tauri::command]
pub(crate)async fn settings_review_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Preview,String>{
 if !permitted(&window){return Err("Abre a janela de definições para rever estes dados.".into());}
 let owner=controller().owner()?;
 tauri::async_runtime::spawn_blocking(move||{
  use tauri::Manager;let shared=app.state::<crate::Shared>();
  if !permitted(&window){return Err(refused());}
  let preview=controller().prepare(&owner,&shared.settings,&crate::platform::config_dir().join("settings.json"),destinations()?,crate::pet_data::bootstrap::restored_history().map_err(|_|refused())?.is_some())?;
  if !permitted(&window){let _=controller().cancel(&owner,Cancellation{token:preview.token.clone(),fingerprint:preview.fingerprint.clone()});return Err(refused());}Ok(preview)
 }).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn settings_review_apply(app:tauri::AppHandle,window:tauri::WebviewWindow,request:Decision)->Result<Receipt,String>{
 if !permitted(&window){return Err("A aprovação exige a janela de definições aberta.".into());}let owner=controller().owner()?;
 tauri::async_runtime::spawn_blocking(move||{
  use tauri::{Manager,Emitter};let shared=app.state::<crate::Shared>();
  // Do not synchronously query GTK/window state while the Settings mutex is
  // held: another IPC may be waiting for that mutex on the main thread.
  // Visibility was checked at admission; native close/destroy/registration
  // revoke the captured Owner epoch, rechecked at every effect boundary.
  let(fresh,receipt)=controller().apply(&owner,&shared.settings,request,destinations,||true)?;
  // Data/cache publication only. Operational syncs have their own admission.
  crate::pet_native_binding::settings_committed(&app,fresh.pet_system_center);
  let _=app.emit("settings-changed",&fresh);crate::pet_native_binding::publish_current(&app);
  let _=app.emit("settings-review-state-changed",&receipt.status);Ok(receipt)
 }).await.map_err(|_|"A revisão pode ter sido publicada sem resposta. Consulta o estado; não repitas efeitos anteriores.".to_string())?
}
#[tauri::command]
pub(crate)fn settings_review_cancel(window:tauri::WebviewWindow,request:Cancellation)->Result<(),String>{
 if window.label()!=crate::HUB_LABEL{return Err(refused());}let owner=controller().owner()?;controller().cancel(&owner,request)
}
#[cfg(test)]#[path="settings_review_tests.rs"]mod tests;
