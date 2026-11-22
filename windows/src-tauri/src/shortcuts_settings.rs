//! Field-only desired shortcut data. It never registers/releases native keys.
use super::{Bindings,Binding};
use crate::settings::{self,Settings,review::Frame,durable::components::{self,Context}};
use serde::{Serialize,Deserialize};
use std::{collections::BTreeMap,path::Path,sync::Mutex};

#[derive(Clone,Deserialize)]#[serde(deny_unknown_fields)]
pub(crate)struct Value{keys:String,enabled:bool}
impl Value{fn binding(&self)->Binding{Binding{keys:self.keys.clone(),enabled:self.enabled}}}
#[derive(Clone,Deserialize)]#[serde(deny_unknown_fields)]
pub(crate)struct Change{before:Option<Value>,after:Option<Value>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate)struct Request{changes:BTreeMap<String,Change>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{schema_version:u32,bindings:Bindings,changed:bool,new_registration_requires_review:bool,status:settings::IoStatus}
fn refused()->String{"Os atalhos ou a revisão de origem mudaram. Atualiza as preferências antes de voltar a alterar; nenhum registo nativo foi iniciado.".into()}
type ContextReader=fn(&Settings,&Path)->std::io::Result<Context>;
fn context(s:&Settings,p:&Path)->std::io::Result<Context>{components::context(s,p,"shortcuts")}
fn apply_with(frame:&Frame,current:&Mutex<Settings>,path:&Path,request:Request,read_context:ContextReader,admit:impl Fn()->Result<(),String>)->Result<Receipt,String>{
 if request.changes.is_empty()||request.changes.len()>super::ACTIONS.len()||request.changes.iter().any(|(id,c)|super::find(id).is_none()||c.before.as_ref().is_some_and(|v|v.keys.len()>256)||c.after.as_ref().is_some_and(|v|v.keys.len()>256)){return Err("A alteração só pode nomear ações de atalhos conhecidas e limitadas.".into());}
 let authorized=||{admit()?;if frame.current(){Ok(())}else{Err(refused())}};authorized()?;
 let mut settings=current.try_lock().map_err(|_|"A gravação das preferências está ocupada.".to_string())?;settings::io_status(&settings).check()?;
 if read_context(&settings,path).map_err(|e|e.to_string())?.profile.is_none(){return Err(refused());}
 let mut bindings=settings.shortcuts.clone();
 for(id,change)in request.changes{
  if bindings.get(&id).cloned()!=change.before.as_ref().map(Value::binding){return Err(refused());}
  match change.after{Some(value)=>{bindings.insert(id,value.binding());},None=>{bindings.remove(&id);}}
 }
 if bindings.len()>64||bindings.iter().any(|(id,b)|id.len()>128||b.keys.len()>256){return Err("A coleção de atalhos excede os limites da revisão; os dados foram conservados.".into());}
 let changed=bindings!=settings.shortcuts;
 if changed{
  // Revocation is durable BEFORE new preferences. No shadow native-effect
  // slot is invented; an old unknown effect remains exactly the same slot.
  components::suspend_shortcut_registration(&settings,path,||authorized().map_err(|_|"shortcutDataReviewRevoked")).map_err(|e|e.to_string())?;
  components::save_shortcuts(&mut settings,path,bindings,||authorized().map_err(std::io::Error::other)).map_err(|e|e.to_string())?;
 }
 let status=settings::io_status(&settings);
 Ok(Receipt{schema_version:1,bindings:settings.shortcuts.clone(),changed,new_registration_requires_review:status.suspended_operations.iter().any(|n|n=="shortcuts"),status})
}
#[tauri::command]
pub(crate)async fn shortcuts_settings_patch(app:tauri::AppHandle,window:tauri::WebviewWindow,request:Request)->Result<Receipt,String>{
 if window.label()!=crate::HUB_LABEL||!window.is_visible().unwrap_or(false){return Err("Abre as definições para alterar os atalhos.".into());}
 let frame=settings::review::controller().frame()?;let path=crate::platform::config_dir().join("settings.json");
 tauri::async_runtime::spawn_blocking(move||{use tauri::{Manager,Emitter};let shared=app.state::<crate::Shared>();
  let result=apply_with(&frame,&shared.settings,&path,request,context,||crate::pet_data::bootstrap::write_allowed().map_err(|e|e.to_string()))?;
  // Publish current state, never an old complete renderer snapshot.
  {let settings=shared.settings.lock().map_err(|_|refused())?;let _=app.emit("settings-changed",&*settings);}
  let _=app.emit("settings-review-state-changed",&result.status);Ok(result)
 }).await.map_err(|_|"A gravação terminou sem resposta confirmada. Consulta o estado antes de repetir.".to_string())?
}

#[cfg(test)]#[path="shortcuts_settings_tests.rs"]mod tests;
