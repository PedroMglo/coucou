//! Other platforms keep their explicit existing native preference path.
fn message()->String{"A revisão de registo de autostart está disponível neste incremento apenas em Linux.".into()}
#[tauri::command]pub(crate)fn pet_autostart_review_prepare(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(message())}
#[tauri::command]pub(crate)fn pet_autostart_review_apply(request:serde_json::Value)->Result<serde_json::Value,String>{let _=request;Err(message())}
#[tauri::command]pub(crate)fn pet_autostart_review_cancel(request:serde_json::Value)->Result<(),String>{let _=request;Err(message())}
#[tauri::command]pub(crate)fn pet_autostart_review_status()->Result<serde_json::Value,String>{Err(message())}
