//! Native close/ACK is a control operation, never an inference or new grant.
use super::*;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeCloseInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct NativeCloseResult{pub receipt:model::Receipt,pub confirmed:bool,pub executor_recreated:bool}
#[tauri::command]
pub async fn pet_runtime_close_deterministic(window:tauri::WebviewWindow,input:NativeCloseInput)->model::Result<NativeCloseResult>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||{
  let store=Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;let row=goal.admissions.last().ok_or(Error::Conflict)?;
  if !row.spec.deterministic()||row.lease.as_ref().is_none_or(|l|l.id!=input.lease_id){return Err(Error::Conflict);}
  let authority=Authority::existing(&admission_review::authority_root()).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  // The signed original fact may be acknowledged after restart. No stored
  // lease, phase, PID, user flag or JSON recreates the original Owned.
  let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
  let (receipt,confirmed)=if let Some(seal)=authority.native_drain(&goal).map_err(crate::pet_runtime::authority::Failure::control_error)?{
   (store.ack_native_drain(&admission_review::authority_root(),&input.goal_id,&input.pet_id,&input.lease_id,&input.revision)?,seal.reference.confirmed)
  }else{
   if goal.revision!=input.revision||!matches!(goal.state,model::State::Leased|model::State::Stopping){return Err(Error::Conflict);}
   let owner=service.entries.get(&goal.id).and_then(|e|e.owned.clone()).ok_or(Error::Conflict)?;if owner.lease().id!=input.lease_id{return Err(Error::Conflict);}
   // Literal native effects retain WorkerGuard through local publication. A
   // busy drain stops admission first and waits for those original guards.
   let drain=owner.drain(DrainOutcome::Confirmed)?;let receipt=store.drain_owned(&authority,&drain,crate::assistant_memory::now())?;let confirmed=authority.native_drain(&receipt.record).map_err(crate::pet_runtime::authority::Failure::control_error)?.ok_or(Error::Conflict)?.reference.confirmed;(receipt,confirmed)
  };
  if receipt.status!=model::CommitStatus::Uncertain{
   if let Some(entry)=service.entries.get(&goal.id){if entry.pending.lease().id!=input.lease_id||entry.pending.workers()!=0{return Err(Error::Conflict);}
    if confirmed{let owner=entry.owned.as_ref().ok_or(Error::Conflict)?;let drain=owner.drain(DrainOutcome::Confirmed)?;entry.pending.hold().release_drained(&drain).map_err(|_|Error::StorageUnavailable)?;}
   }
   service.entries.remove(&goal.id);crate::pet_runtime::step_api::release(&input.lease_id);crate::pet_runtime::recipe_api::release(&input.lease_id);
  }
  Ok(NativeCloseResult{receipt,confirmed,executor_recreated:false})
 }).await.map_err(|_|Error::StorageUnavailable)?}
}
