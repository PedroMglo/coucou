//! Native review only; no caller-provided grants, spec, child identity or paths.
use super::{model,delegation_api::DelegationConfirmInput};
use serde::Deserialize;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DelegationBranchInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub message_id:String,pub action:super::delegation::BranchAction}
fn check(w:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(w.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]
mod native{
 use super::*;use crate::pet_runtime::{Repository,store::Store,authority::Authority,authority_store::{delegation::View,delegation_branch::{Candidate,Proposal,Action,StopTarget}},message_api::native::Origin};use std::{sync::{Arc,Mutex,OnceLock},collections::BTreeMap};use serde::Serialize;
 #[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]pub struct Ticket{token:String,proposal:Proposal}
 #[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]pub enum Result{Updated{view:View,#[serde(rename="stopProjection")]stop_projection:Option<&'static str>},Declined,Blocked{reason:model::Error,retained:bool}}
 struct Pending{origin:Origin,candidate:Candidate,approved:bool,ended:bool}
 fn cache()->&'static Mutex<BTreeMap<String,Arc<Mutex<Pending>>>>{static C:OnceLock<Mutex<BTreeMap<String,Arc<Mutex<Pending>>>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
 pub(super)fn prepare(origin:Origin,i:DelegationBranchInput)->model::Result<Ticket>{
  let root=crate::settings::config_dir().join("pet-runtime-v1");let st=Store::existing_native(&root)?.ok_or(model::Error::NotFound)?;let g=st.read_control(&i.goal_id,&i.pet_id)?;let a=Authority::existing(&crate::pet_runtime::admission_review::authority_root()).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  let c=a.prepare_delegation_branch(g,&i.decision_id,&i.message_id,i.action,crate::assistant_memory::now())?;
  if !origin.frame.current(){return Err(model::Error::Conflict);}let mut map=cache().lock().map_err(|_|model::Error::StorageUnavailable)?;
  map.retain(|_,p|match p.try_lock(){Err(_)=>true,Ok(p)=>!p.ended&&(p.approved||p.origin.frame.current()&&p.candidate.proposal.expires_at>crate::assistant_memory::now())});
  for(token,p)in map.iter(){let Ok(p)=p.try_lock()else{continue;};let old=&p.candidate.proposal;if p.origin.same(&origin)&&old.goal_id==c.proposal.goal_id&&old.decision_id==c.proposal.decision_id&&old.reservation==c.proposal.reservation&&old.action==c.proposal.action&&old.child_grant_hash==c.proposal.child_grant_hash{return Ok(Ticket{token:token.clone(),proposal:old.clone()});}}
  if map.len()>=64{return Err(model::Error::LimitReached);}let token=crate::pet_runtime::store::nonce()?;let t=Ticket{token:token.clone(),proposal:c.proposal.clone()};map.insert(token,Arc::new(Mutex::new(Pending{origin,candidate:c,approved:false,ended:false})));Ok(t)
 }
 fn project_stop(mut target:StopTarget)->model::Result<&'static str>{
  let descendants=std::mem::take(&mut target.descendants);let mut pending=false;for child in descendants{if project_stop(child).is_err(){pending=true;}}let own=project_stop_one(target);if pending{Err(model::Error::Busy)}else{own}
 }
 fn project_stop_one(target:StopTarget)->model::Result<&'static str>{
  let root=crate::settings::config_dir().join("pet-runtime-v1");let st=Store::existing_native(&root)?.ok_or(model::Error::NotFound)?;let path=crate::pet_runtime::admission_review::authority_root();crate::pet_runtime::authority_store::project_stop_queues(&st,&path,std::slice::from_ref(&target),crate::assistant_memory::now())?;let b=target.binding;
  let g=st.read_control(&b.goal_id,&b.pet_id)?;
  // Missing queue is an explicitly supported partial publication. It cannot
  // recreate an approval/admission, nor cancel some later unrelated grant.
  let Some(row)=g.admissions.last()else{return Ok("noQueuedAdmission;canonicalRevocationOnly");};if !b.matches(&row.spec){return Err(model::Error::Conflict);}
  let g=st.read_control(&b.goal_id,&b.pet_id)?;let row=g.admissions.last().ok_or(model::Error::Conflict)?;if !b.matches(&row.spec){return Err(model::Error::Conflict);}
  if row.lease.is_none(){crate::pet_runtime::lease_api::cleanup_unstarted(&st,&path,&g,crate::assistant_memory::now())?;return Ok("queueCancelled;noClaimedLease");}
  crate::pet_runtime::lease_api::drain_cancelled_branch(&st,&b)?;Ok("originalNativeClosureConfirmed;noWorkerOrHoldReconstructed")
 }
 pub(super)fn confirm(origin:Origin,i:DelegationConfirmInput)->model::Result<Result>{
  let p=cache().lock().map_err(|_|model::Error::StorageUnavailable)?.get(&i.token).cloned().ok_or(model::Error::Conflict)?;let mut p=p.try_lock().map_err(|_|model::Error::Busy)?;let v=&p.candidate.proposal;
  if !p.origin.same(&origin)||v.pet_id!=i.pet_id||v.goal_id!=i.goal_id||v.decision_id!=i.decision_id{return Err(model::Error::Conflict);}let now=crate::assistant_memory::now();let observed=p.candidate.observed(now)?.is_some();
  // Already persisted cancellation is a fact, including after ACK loss or a
  // late decline. Continue only authority reduction/drain, never new effects.
  if !observed{if p.ended{return Err(model::Error::Conflict);}if !i.approve{if p.approved{return Ok(Result::Blocked{reason:model::Error::Conflict,retained:true});}p.ended=true;return Ok(Result::Declined);}p.approved=true;}
  let frame=p.origin.frame.clone();match p.candidate.confirm(now,Arc::new(move||frame.current())){
   Err(reason)=>Ok(Result::Blocked{reason,retained:true}),
   Ok(c)=>{let projection=c.stop.map(|s|project_stop(s).unwrap_or("pendingOriginalStopOrDrain;reservationRetained"));p.ended=p.candidate.proposal.action==Action::Reconcile||!projection.is_some_and(|p|p.starts_with("pending"));Ok(Result::Updated{view:c.view,stop_projection:projection})}
  }
 }
}
#[cfg(target_os="linux")]pub use native::{Ticket as DelegationBranchTicket,Result as DelegationBranchResult};
#[cfg(target_os="linux")]async fn origin(w:&tauri::WebviewWindow)->model::Result<super::message_api::native::Origin>{let frame=crate::settings::review::capture_window(w).await.map_err(|_|model::Error::Conflict)?;Ok(super::message_api::native::Origin{window:w.label().into(),frame})}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_delegation_branch(window:tauri::WebviewWindow,input:DelegationBranchInput)->model::Result<DelegationBranchTicket>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::prepare(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_delegation_branch(window:tauri::WebviewWindow,input:DelegationConfirmInput)->model::Result<DelegationBranchResult>{check(&window)?;let origin=origin(&window).await?;tauri::async_runtime::spawn_blocking(move||native::confirm(origin,input)).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_delegation_branch(window:tauri::WebviewWindow,input:serde_json::Value)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_delegation_branch(window:tauri::WebviewWindow,input:DelegationConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
