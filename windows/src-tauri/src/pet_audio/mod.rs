//! Native PC audio shared by Spoty and own System controls. No capture, models, chat or shell commands.
mod types;
pub(crate)mod task_action;
pub(crate)mod commands;
mod system_policy;
pub(crate)mod granted_voice;
pub use system_policy::{SystemSnapshot,SystemReceipt};
pub use types::*;
#[cfg(target_os = "linux")] pub(crate)mod effect_lock;
#[cfg(target_os = "linux")] pub(crate)mod linux;
#[cfg(target_os = "linux")] pub(crate)mod task;
#[cfg(target_os = "linux")] pub(crate)mod native_task;
#[cfg(target_os = "linux")] pub(crate)mod pipewire_reader;
use tauri::{AppHandle, Manager};

fn authorize(window: &tauri::WebviewWindow, app: &AppHandle) -> Result<(), String> {
    if ![crate::island::WINDOW_LABEL, crate::pet_control::LABEL, crate::HUB_LABEL].contains(&window.label()) {
        return Err("Abre o Spoty para controlar o som.".into());
    }
    authorize_owner(app)
}
fn authorize_owner(app:&AppHandle)->Result<(),String>{
    let shared = app.state::<crate::Shared>();
    let settings = shared.settings.lock().unwrap();
    if settings.main_pill != "integration_spotify" && !settings.active_integrations.iter().any(|id| id == "integration_spotify") {
        return Err("Ativa o Spoty na lista de pets para controlar o som.".into());
    }
    drop(settings);
    crate::pet_config::require("integration_spotify", "music")?;
    Ok(())
}

#[tauri::command]
pub async fn pet_audio_snapshot(window: tauri::WebviewWindow, app: AppHandle) -> Result<Snapshot, String> {
    authorize(&window, &app)?;
    #[cfg(target_os = "linux")] { Ok(linux::snapshot().await) }
    #[cfg(not(target_os = "linux"))] { Ok(Snapshot { error: Some("O controlo de som do Spoty está disponível em Linux.".into()), ..Snapshot::default() }) }
}

#[tauri::command]
pub async fn pet_audio_control(window: tauri::WebviewWindow, app: AppHandle, request: Request) -> Result<Receipt, String> {
    authorize(&window, &app)?;
    #[cfg(target_os = "linux")] { let original_window=window.clone();let original_app=app.clone();linux::control_with(request,std::sync::Arc::new(move||authorize(&original_window,&original_app).is_ok())).await }
    #[cfg(not(target_os = "linux"))] { let _ = request; Err("O controlo de som do Spoty está disponível em Linux.".into()) }
}

/// Called only by Spoty's already addressed, user-started voice session.
/// Recognition and speech remain with the existing voice engine.
pub fn spoken(app:&AppHandle,normalized: &str) -> Option<Result<(String, bool), String>> {
    let command = commands::parse(normalized)?;
    #[cfg(target_os = "linux")] { let original_app=app.clone();let check=std::sync::Arc::new(move||authorize_legacy_spoken(&original_app).is_ok());Some(tauri::async_runtime::block_on(linux::spoken(command,check))) }
    #[cfg(not(target_os = "linux"))] { let _ = command; Some(Err("O controlo do som está disponível em Linux.".into())) }
}
fn authorize_legacy_spoken(app:&AppHandle)->Result<(),String>{
 authorize_owner(app)?;
 if crate::pet_config::read_existing()?.pets.get("integration_spotify").is_some_and(|p|p.system_access.is_some()){
  return Err("O Spotify tem uma política própria de Sistema. O comando de Som tem de usar essa concessão; a permissão de música não a substitui.".into());
 }Ok(())
}

/// Own island System audio, independently authorised from Spoty's music plugin.
fn authorize_system(window:&tauri::WebviewWindow,app:&AppHandle)->Result<(),String>{
 let enabled=app.state::<crate::Shared>().settings.lock().unwrap().pet_system_center;
 if !system_policy::allowed(window.label()==crate::pet_control::LABEL,window.is_visible().unwrap_or(false),enabled){
  return Err("Abre a página Sistema da ilha para controlar o som do computador.".into());
 }
 Ok(())
}
#[tauri::command]
pub async fn pet_system_audio_snapshot(window:tauri::WebviewWindow,app:AppHandle)->Result<SystemSnapshot,String>{
 authorize_system(&window,&app)?;
 #[cfg(target_os="linux")]{Ok(system_policy::devices(linux::snapshot().await))}
 #[cfg(not(target_os="linux"))]{Err("O controlo do som está disponível em Linux.".into())}
}
#[tauri::command]
pub async fn pet_system_audio_control(window:tauri::WebviewWindow,app:AppHandle,request:Request)->Result<SystemReceipt,String>{
 authorize_system(&window,&app)?;
 #[cfg(target_os="linux")]{
  let visible_window=window.clone();let current_app=app.clone();
  let check=std::sync::Arc::new(move||authorize_system(&visible_window,&current_app).is_ok());
  let receipt=linux::system_control(request,check).await?;
  Ok(SystemReceipt{snapshot:system_policy::devices(receipt.snapshot),confirmed:receipt.confirmed,warning:receipt.warning})
 }
 #[cfg(not(target_os="linux"))]{let _=request;Err("O controlo do som está disponível em Linux.".into())}
}

// Addressed NativeUser entry points. A model-supplied PetId is not accepted as
// an authenticated task/turn identity; those caller projections stay closed.
fn addressed_sound(window:&tauri::WebviewWindow,app:&AppHandle,pet_id:&str)->Result<(),String>{
 crate::pet_config::system_access::addressed(window,app,pet_id)
}
fn filter_outputs(mut snapshot:Snapshot,permits:impl Fn(&str)->bool)->SystemSnapshot{
 snapshot.outputs.retain(|n|n.kind==Kind::Output&&permits(&n.key));snapshot.inputs.clear();snapshot.cards.clear();snapshot.playback.clear();snapshot.recording.clear();system_policy::devices(snapshot)
}
#[tauri::command]
pub async fn pet_sound_snapshot(window:tauri::WebviewWindow,app:AppHandle,pet_id:String)->Result<SystemSnapshot,String>{
 addressed_sound(&window,&app,&pet_id)?;
 #[cfg(not(target_os="linux"))]{Err("O broker de Som está disponível em Linux.".into())}
 #[cfg(target_os="linux")] {
  use crate::pet_config::system_access::{Captured,Operation};
  let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa terminou.")?;
  let scope=std::sync::Arc::new(Captured::capture(&pet_id,Operation::Observe,None)?);let original=scope.clone();
  let check=std::sync::Arc::new(move||frame.current()&&addressed_sound(&window,&app,&pet_id).is_ok()&&original.check().is_ok());
  let result=linux::pet_outputs_snapshot(check).await;scope.check()?;Ok(filter_outputs(result,|key|scope.permits_output(key)))
 }
}
#[tauri::command]
pub async fn pet_sound_control(window:tauri::WebviewWindow,app:AppHandle,pet_id:String,request:Request)->Result<SystemReceipt,String>{
 addressed_sound(&window,&app,&pet_id)?;
 #[cfg(not(target_os="linux"))]{let _=request;Err("O broker de Som está disponível em Linux.".into())}
 #[cfg(target_os="linux")] {
  use crate::pet_config::system_access::{Captured,Operation};
  let operation=match request.action{Action::Volume=>Operation::Volume,Action::Mute=>Operation::Mute,_=>return Err("Esta concessão não inclui essa operação de Som.".into())};
  let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa terminou.")?;
  let scope=std::sync::Arc::new(Captured::capture(&pet_id,operation,Some(&request.target_key))?);let original=scope.clone();
  let check=std::sync::Arc::new(move||frame.current()&&addressed_sound(&window,&app,&pet_id).is_ok()&&original.check().is_ok());
  let receipt=linux::pet_outputs_control(request,check).await?;
  let current=scope.check().is_ok();let snapshot=if current{filter_outputs(receipt.snapshot,|key|scope.permits_output(key))}else{system_policy::devices(Snapshot{error:Some("O efeito pode ter sido enviado antes da revogação. Não repitas automaticamente.".into()),..Snapshot::default()})};
  Ok(SystemReceipt{snapshot,confirmed:current&&receipt.confirmed,warning:if current{receipt.warning}else{Some("A concessão mudou depois de um possível efeito. O resultado fica por confirmar; não repitas automaticamente.".into())}})
 }
}

#[cfg(test)]mod system_access_projection_tests{
 use super::*;
 fn node(kind:Kind,key:&str,label:&str)->Node{Node{key:key.into(),kind,label:label.into(),description:label.into(),volume:Some(20.0),muted:Some(false),balance:None,is_default:false,device_key:None,monitor:false,controllable:true,can_volume:true,can_balance:false,active:true,options:vec![],active_option:None,selector:"native-selector-not-public".into(),index:1,native_id:None,native_serial:None}}
 #[test]fn system_access_output_projection_never_publishes_other_kinds_or_ungranted_resources(){
  let key="audio-0123456789abcdef";let snapshot=Snapshot{available:true,outputs:vec![node(Kind::Output,key,"Authorized output"),node(Kind::Output,"audio-fedcba9876543210","private-other-output"),node(Kind::Input,key,"private-wrong-kind")],inputs:vec![node(Kind::Input,key,"private-microphone")],cards:vec![node(Kind::Card,key,"private-card")],playback:vec![node(Kind::Playback,key,"private-playback")],recording:vec![node(Kind::Recording,key,"private-recording")],..Snapshot::default()};
  let result=filter_outputs(snapshot,|k|k==key);assert_eq!(result.outputs.len(),1);assert!(result.inputs.is_empty()&&result.cards.is_empty());let public=serde_json::to_string(&result).unwrap();assert!(!public.contains("private-")&&!public.contains("native-selector"));
 }
}
