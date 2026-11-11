//! Existing native-user read boundary. This command cannot execute a pet or
//! send a result to an inference provider; supplied PetId is only a query target.
use super::{model,delegation_api::DelegationInput};
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_peer_feedback(window:tauri::WebviewWindow,input:DelegationInput)->model::Result<super::peer_feedback::View>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 tauri::async_runtime::spawn_blocking(move||super::peer_feedback::capture(&crate::settings::config_dir().join("pet-runtime-v1"),&super::admission_review::authority_root(),&input,crate::assistant_memory::now())).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(not(target_os="linux"))]
#[tauri::command]
pub async fn pet_runtime_peer_feedback(window:tauri::WebviewWindow,input:DelegationInput)->model::Result<serde_json::Value>{let _=(window,input);Err(model::Error::UnsupportedPlatform)}
