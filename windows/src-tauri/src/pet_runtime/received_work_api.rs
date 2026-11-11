//! Read-only native-user IPC. No frame/approval, model request or new grant.
#[tauri::command]
pub async fn pet_runtime_received_work(window:tauri::WebviewWindow,input:super::received_work::Input)->super::model::Result<super::received_work::View>{
 if !super::adapter::allowed(window.label()){return Err(super::model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(super::model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]
 tauri::async_runtime::spawn_blocking(move||{
  let store=super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(super::model::Error::NotFound)?;
  store.received_work(&super::admission_review::authority_root(),&input,crate::assistant_memory::now())
 }).await.map_err(|_|super::model::Error::StorageUnavailable)?
}
