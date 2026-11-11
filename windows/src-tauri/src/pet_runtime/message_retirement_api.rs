//! Native authority-reduction controls; not a resend or a generic staging delete.
use super::model;
use serde::Deserialize;
#[cfg(target_os="linux")]#[path="message_retirement_api_native.rs"]pub(super)mod native;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MessageIntentsInput{pub pet_id:String,pub goal_id:String,pub decision_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MessageRetireInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub decision_id:String,pub message_id:String,pub request_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MessageRetireConfirmInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub decision_id:String,pub token:String,pub approve:bool}
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]async fn origin(window:&tauri::WebviewWindow)->model::Result<super::message_api::native::Origin>{let frame=crate::settings::review::capture_window(window).await.map_err(|_|model::Error::Conflict)?;Ok(super::message_api::native::Origin{window:window.label().into(),frame})}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_pending_messages(window:tauri::WebviewWindow,input:MessageIntentsInput)->model::Result<super::authority_store::message_retirement::IntentInventory>{check(&window)?;tauri::async_runtime::spawn_blocking(move||native::pending(input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_message_retirement(window:tauri::WebviewWindow,input:MessageRetireInput)->model::Result<native::Prepared>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::prepare(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_message_retirement(window:tauri::WebviewWindow,input:MessageRetireConfirmInput)->model::Result<native::Confirmed>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::confirm(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_pending_messages(window:tauri::WebviewWindow,input:MessageIntentsInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_message_retirement(window:tauri::WebviewWindow,input:MessageRetireInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_message_retirement(window:tauri::WebviewWindow,input:MessageRetireConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
