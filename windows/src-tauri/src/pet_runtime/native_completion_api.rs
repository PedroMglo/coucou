//! Native callers inspect/finalize only criteria sealed in original admission.
use super::model;
use serde::Deserialize;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeCompletionInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String}
#[cfg(target_os="linux")]
pub(super)fn finish(store:&super::store::Store,input:&NativeCompletionInput)->model::Result<super::lease_controller::native_completion::Progress>{
 use super::{authority::Authority,model::State,lease_controller::native_completion::Progress};
 let goal=store.read_control(&input.goal_id,&input.pet_id)?;let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
 if !matches!(row.spec.schema_version,5|6|7|8|9|10|11|14|16|17|18|20)||row.spec.native_verification.is_none()||row.lease.as_ref().is_none_or(|l|l.id!=input.lease_id){return Err(model::Error::Conflict);}
 let progress=if goal.state==State::Completed{
  let reference=goal.completion.as_ref().filter(|c|c.source.is_some()).ok_or(model::Error::Conflict)?;
  if input.revision!=goal.revision&&input.revision!=reference.source_revision{return Err(model::Error::Conflict);}
  let a=Authority::existing(&super::admission_review::authority_root()).map_err(super::authority::Failure::control_error)?;let report=a.native_completion(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?).map_err(super::authority::Failure::control_error)?.ok_or(model::Error::Conflict)?;let certificate=a.certify_native_completion(&goal).map_err(super::authority::Failure::control_error)?;
  let receipt=store.complete_certified(&certificate,crate::assistant_memory::now(),|_|Err(model::Error::Conflict))?;Progress{report,completion:Some(receipt),closure_pending:None,recipient_execution_verified:false}
 }else{
  if goal.revision!=input.revision{return Err(model::Error::Conflict);}let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;owner.verify_native_goal(store,&goal)?
 };
 Ok(record_closure(progress,||super::lease_api::close_completed(store,&input.goal_id,&input.pet_id,&input.lease_id)))
}
#[cfg(target_os="linux")]
pub(super)fn record_closure(mut progress:super::lease_controller::native_completion::Progress,close:impl FnOnce()->model::Result<()>)->super::lease_controller::native_completion::Progress{
 if progress.completion.as_ref().is_some_and(|r|r.status!=model::CommitStatus::Uncertain){if let Err(reason)=close(){progress.closure_pending=Some(reason);}}progress
}
#[cfg(target_os="linux")]
pub(super)fn after_message(app:&tauri::AppHandle,receipt:&super::messages::Receipt){
 use tauri::Emitter;let e=receipt.envelope();let store=match super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1")){Ok(Some(s))=>s,_=>return};let goal=match store.read_control(&e.origin.goal_id,&e.origin.pet_id){Ok(g)=>g,Err(_)=>return};
 if goal.admissions.last().is_none_or(|r|!matches!(r.spec.schema_version,5|6|7|8|9|10|11|14|16|17|18|20)||r.spec.native_verification.is_none()||r.spec.decision_id!=e.origin.decision_id||r.lease.as_ref().is_none_or(|l|l.id!=e.origin.lease_id)){return;}
 let input=NativeCompletionInput{pet_id:e.origin.pet_id.clone(),goal_id:goal.id.clone(),lease_id:e.origin.lease_id.clone(),revision:goal.revision.clone()};
 // Bounded retries handle pre-completion Busy or cleanup of an already
 // acknowledged completion. Original owner/revision/lease remain captured;
 // no delivery, inference or authority is repeated.
 let until=std::time::Instant::now()+std::time::Duration::from_secs(1);
 let outcome=loop{match finish(&store,&input){Err(model::Error::Busy)if std::time::Instant::now()<until=>std::thread::sleep(std::time::Duration::from_millis(10)),Ok(p)if p.closure_pending==Some(model::Error::Busy)&&std::time::Instant::now()<until=>std::thread::sleep(std::time::Duration::from_millis(10)),result=>break result}};
 let state=match outcome{Ok(p)=>serde_json::json!({"criteriaVerified":p.report.criteria_verified,"completionConfirmed":p.completion.as_ref().is_some_and(|r|r.status!=model::CommitStatus::Uncertain),"ownershipClosurePending":p.closure_pending}),Err(reason)=>serde_json::json!({"criteriaVerified":false,"completionConfirmed":false,"reason":reason})};
 // Delivery success remains its own receipt. Failure here never sends it again
 // or reports a completed goal; explicit verification is the recovery control.
 let _=app.emit("pet-runtime-progress",serde_json::json!({"goalId":goal.id,"petId":input.pet_id,"nativeVerification":state}));
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_verify_deterministic(window:tauri::WebviewWindow,input:NativeCompletionInput)->model::Result<super::lease_controller::native_completion::Progress>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}tauri::async_runtime::spawn_blocking(move||{let store=super::store::Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(model::Error::NotFound)?;finish(&store,&input)}).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_verify_deterministic(window:tauri::WebviewWindow,input:NativeCompletionInput)->model::Result<serde_json::Value>{if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}let _=input;Err(model::Error::UnsupportedPlatform)}
