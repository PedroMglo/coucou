//! Native human control for reviewed plan DATA, with a separate execution
//! consent afterwards. Providers receive no callable approval route here.
use super::model;
use serde::Deserialize;
#[cfg(target_os="linux")]#[path="native_plan_revision_api_native.rs"]pub(super)mod native;
#[derive(Clone,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareNativePlanInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub inputs:Vec<super::native_plan_revisions::Input>}
#[derive(Clone,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConfirmNativePlanInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Clone,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativePlanListInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub cursor:Option<String>}
#[derive(Clone,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativePlanControlInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub review_id:String}
// Native window entry point; exact Frame, scope and signed DATA checks remain downstream.
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]async fn origin(window:&tauri::WebviewWindow)->model::Result<super::message_api::native::Origin>{let frame=crate::settings::review::capture_window(window).await.map_err(|_|model::Error::Conflict)?;Ok(super::message_api::native::Origin{window:window.label().into(),frame})}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_native_plan_revisions(window:tauri::WebviewWindow,input:NativePlanListInput)->model::Result<super::native_plan_inventory::Inventory>{check(&window)?;tauri::async_runtime::spawn_blocking(move||native::inventory(input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_native_plan_revision(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PrepareNativePlanInput)->model::Result<native::Ticket>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::prepare(app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_native_plan_revision(window:tauri::WebviewWindow,input:ConfirmNativePlanInput)->model::Result<native::Confirmed>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::confirm(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_ack_native_plan_revision(window:tauri::WebviewWindow,input:NativePlanControlInput)->model::Result<super::native_plan_review::Receipt>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::ack(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_abandon_native_plan_revision(window:tauri::WebviewWindow,input:NativePlanControlInput)->model::Result<native::Withdrawn>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::abandon(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}

#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_native_plan_revisions(window:tauri::WebviewWindow,input:NativePlanListInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_native_plan_revision(window:tauri::WebviewWindow,input:PrepareNativePlanInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_native_plan_revision(window:tauri::WebviewWindow,input:ConfirmNativePlanInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_ack_native_plan_revision(window:tauri::WebviewWindow,input:NativePlanControlInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_abandon_native_plan_revision(window:tauri::WebviewWindow,input:NativePlanControlInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
