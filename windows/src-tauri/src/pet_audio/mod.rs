//! Native PC audio shared by Spoty and own System controls. No capture, models, chat or shell commands.
mod types;
mod commands;
mod system_policy;
pub use system_policy::{SystemSnapshot,SystemReceipt};
pub use types::*;
#[cfg(target_os = "linux")] mod linux;
use tauri::{AppHandle, Manager};

fn authorize(window: &tauri::WebviewWindow, app: &AppHandle) -> Result<(), String> {
    if ![crate::island::WINDOW_LABEL, crate::pet_control::LABEL, crate::HUB_LABEL].contains(&window.label()) {
        return Err("Abre o Spoty para controlar o som.".into());
    }
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
    #[cfg(target_os = "linux")] { linux::control(request).await }
    #[cfg(not(target_os = "linux"))] { let _ = request; Err("O controlo de som do Spoty está disponível em Linux.".into()) }
}

/// Called only by Spoty's already addressed, user-started voice session.
/// Recognition and speech remain with the existing voice engine.
pub fn spoken(normalized: &str) -> Option<Result<(String, bool), String>> {
    let command = commands::parse(normalized)?;
    #[cfg(target_os = "linux")] { Some(tauri::async_runtime::block_on(linux::spoken(command))) }
    #[cfg(not(target_os = "linux"))] { let _ = command; Some(Err("O controlo do som está disponível em Linux.".into())) }
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
