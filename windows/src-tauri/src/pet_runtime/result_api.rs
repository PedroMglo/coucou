use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ResultInput { pub pet_id: String, pub goal_id: String, pub decision_id: String, pub request_id: String }

/// Explicit read-only receipt lookup. It does not require or revive a running
/// controller, and never turns a stored receipt into execution authority.
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_result(window: tauri::WebviewWindow, input: ResultInput) -> Result<super::results::View, String> {
    if !super::adapter::allowed(window.label()) { return Err("InvalidInput".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        use super::{authority::Authority, store::Store, results};
        let store = Store::existing(&crate::settings::config_dir().join("pet-runtime-v1")).map_err(|e| format!("{e:?}"))?.ok_or("NotFound")?;
        let goal = store.read(&input.goal_id, &input.pet_id).map_err(|e| format!("{e:?}"))?;
        let row = goal.admissions.iter().find(|a| a.spec.decision_id == input.decision_id).ok_or("NotFound")?;
        let authority = Authority::existing(&super::admission_review::authority_root()).map_err(|e| format!("{e:?}"))?;
        results::lookup(&authority, &row.spec, row.proof.as_ref().ok_or("Missing")?, &input.request_id)
            .map(Into::into).map_err(|e| format!("{e:?}"))
    }).await.map_err(|_| "StorageUnavailable".to_string())?
}
#[cfg(not(target_os="linux"))]
#[tauri::command]
pub async fn pet_runtime_result(window: tauri::WebviewWindow, input: ResultInput) -> Result<serde_json::Value, String> {
    if !super::adapter::allowed(window.label()) { return Err("InvalidInput".into()); }
    let _ = input;
    Err("UnsupportedPlatform".into())
}
