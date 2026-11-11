//! Native-user controls for local peer DATA. No provider/model tool is exposed.
use super::{model,step_api::{PrepareInput,ConfirmInput,ReadInput}};
use serde::Deserialize;
#[cfg(target_os="linux")]#[path="message_api_native.rs"]pub(super)mod native;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MessageInboxInput{#[serde(default)]pub include_funding_sources:bool,pub pet_id:String,pub after:Option<String>,pub limit:usize,#[serde(default)]pub include_historical:bool,#[serde(default)]pub include_reply_sources:bool}
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_message(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<native::Ticket>{
 check(&window)?;let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=native::Origin{window:window.label().into(),frame};
 tauri::async_runtime::spawn_blocking(move||native::prepare(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_message(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<native::ConfirmResult>{
 check(&window)?;let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=native::Origin{window:window.label().into(),frame};
 tauri::async_runtime::spawn_blocking(move||native::confirm(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_message_result(window:tauri::WebviewWindow,input:ReadInput)->model::Result<Option<super::messages::Receipt>>{
 check(&window)?;tauri::async_runtime::spawn_blocking(move||native::result(input)).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_message_inbox(window:tauri::WebviewWindow,input:MessageInboxInput)->model::Result<super::messages::Inbox>{
 check(&window)?;tauri::async_runtime::spawn_blocking(move||native::inbox(input)).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_message(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<serde_json::Value>{check(&window)?;let _=(app,input);Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_message(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=(app,input);Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_message_result(window:tauri::WebviewWindow,input:ReadInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_message_inbox(window:tauri::WebviewWindow,input:MessageInboxInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
