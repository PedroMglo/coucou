//! Native-user data review. JSON cannot supply an Owner, Origin, key or grant.
use super::{model,received_request,messages::ReceivedReference};
use serde::{Deserialize,Serialize};
#[derive(Clone,PartialEq,Eq,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReceivedGoalPrepareInput{pub pet_id:String,pub source:ReceivedReference,pub request_id:String,pub intent:String,pub criteria:Vec<String>}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReceivedGoalConfirmInput{pub pet_id:String,pub source:ReceivedReference,pub token:String,pub approve:bool}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReceivedGoalTicket{pub token:String,pub proposal:received_request::Proposal}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ReceivedGoalPrepared{Review{ticket:ReceivedGoalTicket},Existing{receipt:model::Receipt,#[serde(rename="executionAuthorityGranted")]execution_authority_granted:bool}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ReceivedGoalConfirmed{Created{receipt:model::Receipt,#[serde(rename="executionAuthorityGranted")]execution_authority_granted:bool},Declined}
#[cfg(target_os="linux")]
#[path="received_request_api_native.rs"]pub(in crate::pet_runtime)mod native;
#[tauri::command]
pub async fn pet_runtime_prepare_received_goal(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ReceivedGoalPrepareInput)->model::Result<ReceivedGoalPrepared>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_received_goal(window:tauri::WebviewWindow,input:ReceivedGoalConfirmInput)->model::Result<ReceivedGoalConfirmed>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::confirm(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}

#[tauri::command]
pub async fn pet_runtime_prepare_funded_goal(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ReceivedGoalPrepareInput)->model::Result<ReceivedGoalPrepared>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare_funded(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_funded_goal(window:tauri::WebviewWindow,input:ReceivedGoalConfirmInput)->model::Result<ReceivedGoalConfirmed>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::confirm_funded(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}

#[tauri::command]
pub async fn pet_runtime_prepare_received_result_goal(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ReceivedGoalPrepareInput)->model::Result<ReceivedGoalPrepared>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare_result(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_received_result_goal(window:tauri::WebviewWindow,input:ReceivedGoalConfirmInput)->model::Result<ReceivedGoalConfirmed>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::confirm_result(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}

#[tauri::command]
pub async fn pet_runtime_prepare_recursive_funded_goal(app:tauri::AppHandle,window:tauri::WebviewWindow,input:ReceivedGoalPrepareInput)->model::Result<ReceivedGoalPrepared>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare_recursive_funded(&app,origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_recursive_funded_goal(window:tauri::WebviewWindow,input:ReceivedGoalConfirmInput)->model::Result<ReceivedGoalConfirmed>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|model::Error::Conflict)?;let origin=super::message_api::native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::confirm_recursive_funded(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
}

