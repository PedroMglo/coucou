use super::model;
use serde::Deserialize;
#[derive(Clone,PartialEq,Eq,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DelegationInput{pub pet_id:String,pub goal_id:String,pub decision_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DelegationWithdrawInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub message_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DelegationConfirmInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub token:String,pub approve:bool}
fn check(w:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(w.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
// DATA observation accepts authenticated restored snapshots. Control operations
// keep read_control below: history never gains withdrawal/execution authority.
#[cfg(target_os="linux")]
pub(super)fn read_snapshot(goals:&std::path::Path,authority_path:&std::path::Path,input:&DelegationInput,now:u64)->model::Result<super::authority_store::delegation::View>{
 use super::Repository;
 let g=super::store::Store::existing_verified(goals,authority_path)?.ok_or(model::Error::NotFound)?.read(&input.goal_id,&input.pet_id)?;
 super::authority::Authority::existing(authority_path).map_err(super::authority::Failure::control_error)?.delegation_view(&g,&input.decision_id,now)
}
#[cfg(target_os="linux")]
mod native{
 use super::*;use crate::pet_runtime::{Repository,store::Store,authority::Authority,authority_store::delegation::{Candidate,Proposal,View},message_api::native::Origin};use std::{sync::{Arc,Mutex,OnceLock},collections::BTreeMap};use serde::Serialize;
 #[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]pub struct Ticket{token:String,proposal:Proposal}
 #[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]pub enum Result{Withdrawn{view:View},Declined,Blocked{reason:model::Error,retained:bool}}
 struct Pending{origin:Origin,candidate:Candidate,approved:bool,ended:bool}
 fn cache()->&'static Mutex<BTreeMap<String,Arc<Mutex<Pending>>>>{static C:OnceLock<Mutex<BTreeMap<String,Arc<Mutex<Pending>>>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
 fn goal(input:&DelegationInput)->model::Result<super::super::model::Record>{Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(model::Error::NotFound)?.read_control(&input.goal_id,&input.pet_id)}
 fn authority()->model::Result<Authority>{Authority::existing(&crate::pet_runtime::admission_review::authority_root()).map_err(crate::pet_runtime::authority::Failure::control_error)}
 pub(super)fn read(input:DelegationInput)->model::Result<View>{read_snapshot(&crate::settings::config_dir().join("pet-runtime-v1"),&crate::pet_runtime::admission_review::authority_root(),&input,crate::assistant_memory::now())}
 pub(super)fn prepare(origin:Origin,input:DelegationWithdrawInput)->model::Result<Ticket>{
  let g=goal(&DelegationInput{pet_id:input.pet_id,goal_id:input.goal_id,decision_id:input.decision_id.clone()})?;let c=authority()?.prepare_delegation_withdrawal(g,&input.decision_id,&input.message_id,crate::assistant_memory::now())?;
  if !origin.frame.current(){return Err(model::Error::Conflict);}let mut cache=cache().lock().map_err(|_|model::Error::StorageUnavailable)?;cache.retain(|_,p|match p.try_lock(){Err(_)=>true,Ok(p)=>!p.ended&&(p.approved||p.origin.frame.current()&&p.candidate.proposal.expires_at>crate::assistant_memory::now())});
  for(token,p)in cache.iter(){let Ok(p)=p.try_lock()else{continue;};let old=&p.candidate.proposal;if p.origin.same(&origin)&&old.goal_id==c.proposal.goal_id&&old.decision_id==c.proposal.decision_id&&old.reservation.message_id==c.proposal.reservation.message_id{return Ok(Ticket{token:token.clone(),proposal:old.clone()});}}
  if cache.len()>=64{return Err(model::Error::LimitReached);}let token=crate::pet_runtime::store::nonce()?;let ticket=Ticket{token:token.clone(),proposal:c.proposal.clone()};cache.insert(token,Arc::new(Mutex::new(Pending{origin,candidate:c,approved:false,ended:false})));Ok(ticket)
 }
 pub(super)fn confirm(origin:Origin,i:DelegationConfirmInput)->model::Result<Result>{let p=cache().lock().map_err(|_|model::Error::StorageUnavailable)?.get(&i.token).cloned().ok_or(model::Error::Conflict)?;let mut p=p.try_lock().map_err(|_|model::Error::Busy)?;let v=&p.candidate.proposal;if !p.origin.same(&origin)||v.pet_id!=i.pet_id||v.goal_id!=i.goal_id||v.decision_id!=i.decision_id{return Err(model::Error::Conflict);}let now=crate::assistant_memory::now();if let Some(view)=p.candidate.observed(now)?{p.ended=true;return Ok(Result::Withdrawn{view});}if p.ended{return Err(model::Error::Conflict);}if !i.approve{if p.approved{return Ok(Result::Blocked{reason:model::Error::Conflict,retained:true});}p.ended=true;return Ok(Result::Declined);}p.approved=true;let frame=p.origin.frame.clone();match p.candidate.confirm(now,Arc::new(move||frame.current())){Ok(view)=>{p.ended=true;Ok(Result::Withdrawn{view})},Err(reason)=>Ok(Result::Blocked{reason,retained:true})}}
}
#[cfg(target_os="linux")]pub use native::{Ticket as DelegationWithdrawalTicket,Result as DelegationWithdrawalResult};
#[cfg(target_os="linux")]async fn origin(w:&tauri::WebviewWindow)->model::Result<super::message_api::native::Origin>{let frame=crate::settings::review::capture_window(w).await.map_err(|_|model::Error::Conflict)?;Ok(super::message_api::native::Origin{window:w.label().into(),frame})}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_delegations(window:tauri::WebviewWindow,input:DelegationInput)->model::Result<super::authority_store::delegation::View>{check(&window)?;tauri::async_runtime::spawn_blocking(move||native::read(input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_delegation_withdrawal(window:tauri::WebviewWindow,input:DelegationWithdrawInput)->model::Result<DelegationWithdrawalTicket>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::prepare(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_delegation_withdrawal(window:tauri::WebviewWindow,input:DelegationConfirmInput)->model::Result<DelegationWithdrawalResult>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::confirm(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_delegations(window:tauri::WebviewWindow,input:DelegationInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_delegation_withdrawal(window:tauri::WebviewWindow,input:DelegationWithdrawInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_delegation_withdrawal(window:tauri::WebviewWindow,input:DelegationConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
