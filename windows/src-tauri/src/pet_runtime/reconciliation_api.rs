use serde::Deserialize;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReconcileInput{pub pet_id:String,pub goal_id:String,pub revision:String}
/// Explicit read-only, revision-bound inspection. No recovery approval or
/// controller can be supplied through this IPC surface.
#[tauri::command]
pub async fn pet_runtime_reconcile(window:tauri::WebviewWindow,input:ReconcileInput)->super::model::Result<super::reconciliation::Snapshot>{
    use super::model::Error;
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        if !super::model::token(&input.revision)||!super::model::key(&input.pet_id){return Err(Error::InvalidInput);}
        let store=super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;
        store.reconciliation(&super::admission_review::authority_root(),&input.goal_id,&input.pet_id,&input.revision,crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}

/// Full authenticated history is a read-only projection, not a fresh grant.
#[tauri::command]
pub async fn pet_runtime_history(window:tauri::WebviewWindow,input:ReconcileInput)->super::model::Result<super::history::Snapshot>{
    use super::model::Error;
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        if !super::model::token(&input.revision)||!super::model::key(&input.pet_id){return Err(Error::InvalidInput);}
        let store=super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;
        store.history(&super::admission_review::authority_root(),&input.goal_id,&input.pet_id,&input.revision,crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}
