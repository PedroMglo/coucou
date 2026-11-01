//! No Linux IO-origin proof is claimed on another platform.
fn unavailable()->String{"A revisão de origem das preferências ainda não está disponível nesta plataforma.".into()}
pub(crate)fn register_window(_: &tauri::WebviewWindow){}
#[tauri::command]pub(crate)async fn settings_review_prepare()->Result<serde_json::Value,String>{Err(unavailable())}
#[tauri::command]pub(crate)async fn settings_review_apply(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(unavailable())}
#[tauri::command]pub(crate)fn settings_review_cancel(request:serde_json::Value)->Result<(),String>{let _=request;Err(unavailable())}
