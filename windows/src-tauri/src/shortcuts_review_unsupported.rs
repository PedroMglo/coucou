fn unavailable()->String{"A revisão nativa dos atalhos ainda não está disponível nesta plataforma.".into()}
#[tauri::command]pub(crate)fn shortcuts_review_prepare()->Result<serde_json::Value,String>{Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_review_inspect(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_review_cancel(request:serde_json::Value)->Result<(),String>{let _=request;Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_review_apply(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_review_status()->Result<serde_json::Value,String>{Err(unavailable())}

pub(crate)mod reconciliation{
 use super::*;

#[tauri::command]pub(crate)fn shortcuts_reconcile_prepare()->Result<serde_json::Value,String>{Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_reconcile_inspect(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(unavailable())}
#[tauri::command]pub(crate)fn shortcuts_reconcile_apply(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(unavailable())}

}
