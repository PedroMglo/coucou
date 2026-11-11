//! Native lease preparation/status/release. These APIs never dispatch work.
#[path="native_drain_api.rs"]mod native_drain;
pub use native_drain::*;
use super::{leases::Lease,model::{self,Record,Error}};
#[cfg(target_os="linux")]use super::{admission_review,authority::Authority,lease_controller::{Controller,Pending,Owned,Validity},leases::DrainOutcome,store::Store};
use serde::{Deserialize,Serialize};
#[cfg(target_os="linux")]use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
use tauri::AppHandle;
#[cfg(target_os="linux")]use tauri::Manager;
#[cfg(target_os="linux")]struct Entry{source_revision:String,request_id:String,pending:Pending,owned:Option<Arc<Owned>>}
#[cfg(target_os="linux")]struct Service{controller:Controller,entries:BTreeMap<String,Entry>,maintenance_after:Option<String>,waiting:BTreeMap<String,requested::Waiting>,dispatch_after:Option<String>}
#[cfg(target_os="linux")]#[path="lease_maintenance.rs"]mod maintenance;
#[cfg(target_os="linux")]#[path="requested_dispatch.rs"]mod requested;
#[cfg(target_os="linux")]pub(crate)fn start_runtime(app:AppHandle){maintenance::start(app,root(),admission_review::authority_root());}
#[cfg(target_os="linux")]pub(super)fn stop_requested(goal:&Record){requested::stop(goal);}
#[cfg(target_os="linux")]pub(super)use maintenance::cleanup_unstarted;
#[cfg(target_os="linux")]fn service()->model::Result<&'static Mutex<Service>>{
    static SERVICE:OnceLock<model::Result<Mutex<Service>>>=OnceLock::new();
    SERVICE.get_or_init(||Controller::new().map(|controller|Mutex::new(Service{controller,entries:BTreeMap::new(),maintenance_after:None,waiting:BTreeMap::new(),dispatch_after:None}))).as_ref().map_err(|e|*e)
}
#[cfg(target_os="linux")]fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
#[cfg(target_os="linux")]pub(super)fn step_owner(goal:&Record,lease:&str)->model::Result<Arc<Owned>>{
    let service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
    let owner=service.entries.get(&goal.id).and_then(|e|e.owned.clone()).ok_or(Error::Conflict)?;
    if owner.lease().id!=lease{return Err(Error::Conflict);}Ok(owner)
}
/// Register the retained native incarnation with the existing maintenance and
/// Driver service. A signed lease alone cannot construct this Pending/Owned.
#[cfg(target_os="linux")]fn register_resumption_in(service:&mut Service,authority:&Authority,goal:&Record,pending:Pending,owner:Option<Arc<Owned>>)->model::Result<()>{
 pending.resume_authority(goal,authority)?;
 if let Some(entry)=service.entries.get_mut(&goal.id){
  if !entry.pending.same_live(&pending){return Err(Error::Conflict);}
  if let Some(owner)=owner{if !owner.live_matches_pending(&entry.pending){return Err(Error::Conflict);}if entry.owned.as_ref().is_some_and(|old|!old.same_live(&owner)){return Err(Error::Conflict);}entry.owned=Some(owner);}return Ok(());
 }
 if service.entries.len()>=64{return Err(Error::LimitReached);}
 if owner.as_ref().is_some_and(|o|!o.live_matches_pending(&pending)){return Err(Error::Conflict);}
 service.entries.insert(goal.id.clone(),Entry{source_revision:goal.revision.clone(),request_id:format!("resume-owner-{}",pending.lease().id),pending,owned:owner});Ok(())
}
#[cfg(target_os="linux")]pub(super)fn register_resumption(goal:&Record,pending:Pending,owner:Option<Arc<Owned>>)->model::Result<()>{
 let authority=Authority::existing(&admission_review::authority_root()).map_err(super::authority::Failure::control_error)?;
 let mut service=service()?.try_lock().map_err(|_|Error::Busy)?;register_resumption_in(&mut service,&authority,goal,pending,owner)
}
#[cfg(all(test,target_os="linux"))]pub(in crate::pet_runtime)fn fixture_resumption_cleanup(store:&Store,path:&std::path::Path,goal:&Record,pending:Pending,now:u64)->model::Result<model::Receipt>{
 let authority=Authority::existing(path).map_err(super::authority::Failure::control_error)?;
 let mut service=Service{controller:Controller::new()?,entries:BTreeMap::new(),maintenance_after:None,waiting:BTreeMap::new(),dispatch_after:None};
 register_resumption_in(&mut service,&authority,goal,pending,None)?;
 let receipt=maintenance::cleanup(&mut service,store,path,goal,now)?.ok_or(Error::Conflict)?;
 if receipt.status!=model::CommitStatus::Uncertain&&!service.entries.is_empty(){return Err(Error::Conflict);}Ok(receipt)
}
/// Cleanup is authorized by the original opaque drain proof, after durable
/// closure. A stored lease or UI status never removes a live native owner.
#[cfg(target_os="linux")]pub(super)fn retire_driver(store:&Store,proof:&super::lease_controller::DrainProof)->model::Result<()> {
    if !proof.confirmed(){return Err(Error::Conflict);}let lease=proof.lease();
    let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
    let Some(entry)=service.entries.get(&lease.goal_id)else{return Ok(());};
    if entry.pending.lease()!=lease||entry.pending.workers()!=0{return Err(Error::Conflict);}
    let pet=entry.pending.pet_id();let goal=store.read_control(&lease.goal_id,pet)?;
    if matches!(goal.state,model::State::Leased|model::State::Stopping){return Err(Error::Conflict);}
    let retired=service.entries.remove(&lease.goal_id);drop(service);drop(retired);super::step_api::release(&lease.id);super::recipe_api::release(&lease.id);Ok(())
}
/// Only the original live owner may confirm its workers ended. A stored lease
/// alone never allows this cleanup after restart.
#[cfg(target_os="linux")]pub(super)fn close_completed(store:&Store,goal_id:&str,pet_id:&str,lease_id:&str)->model::Result<()>{
    let goal=store.read(goal_id,pet_id)?;let authority=Authority::existing(&admission_review::authority_root()).map_err(|_|Error::StorageUnavailable)?;
    authority.verify_completed_goal(&goal).map_err(|_|Error::StorageUnavailable)?;
    let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
    let Some(owner)=service.entries.get(goal_id).and_then(|e|e.owned.clone())else{return Ok(());};
    if owner.lease().id!=lease_id{return Err(Error::Conflict);}let drain=owner.drain(DrainOutcome::Confirmed)?;
    let receipt=store.drain_owned(&authority,&drain,crate::assistant_memory::now())?;
    if receipt.status==model::CommitStatus::Uncertain{return Err(Error::StorageUnavailable);}
    let retired=service.entries.remove(goal_id);drop(service);drop(retired);super::step_api::release(lease_id);super::recipe_api::release(lease_id);Ok(())
}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ClaimInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String}
/// Branch Stop reuses the original owner/drain. A JSON lease, elapsed deadline
/// or missing service entry never manufactures a worker-closure proof.
#[cfg(target_os="linux")]pub(super)fn drain_cancelled_branch(store:&Store,binding:&super::funding::ChildBinding)->model::Result<()> {
 let goal=store.read_control(&binding.goal_id,&binding.pet_id)?;let row=goal.admissions.last().ok_or(Error::Conflict)?;
 if !binding.matches(&row.spec){return Err(Error::Conflict);}
 let authority=Authority::existing(&admission_review::authority_root()).map_err(super::authority::Failure::control_error)?;
 let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
 let Some(entry)=service.entries.get(&goal.id)else{let lease=row.lease.as_ref().ok_or(Error::Conflict)?;return if authority.original_lease_closed(&row.spec,row.proof.as_ref().ok_or(Error::Conflict)?,lease).map_err(super::authority::Failure::control_error)?{Ok(())}else{Err(Error::Busy)};};
 if !entry.pending.matches_goal(&goal){return Err(Error::Conflict);}
 let drain=if let Some(owner)=&entry.owned{owner.drain(DrainOutcome::Confirmed)?}else{entry.pending.abort()?};
 let receipt=store.drain_owned(&authority,&drain,crate::assistant_memory::now())?;
 if receipt.status==model::CommitStatus::Uncertain{return Err(Error::StorageUnavailable);}
 entry.pending.hold().release_drained(&drain).map_err(|_|Error::StorageUnavailable)?;
 let lease=drain.lease().id.clone();let retired=service.entries.remove(&goal.id);drop(service);drop(retired);super::step_api::release(&lease);super::recipe_api::release(&lease);Ok(())
}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReleaseInput{pub pet_id:String,pub goal_id:String,pub lease_id:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ClaimResult{pub receipt:model::Receipt,pub lease:Lease,pub owned:bool,pub reservation_owned:bool,pub execution_available:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct LeaseStatus{pub record:Record,pub lease:Option<Lease>,pub owned:bool,pub reservation_owned:bool,pub source:&'static str,pub current:bool,pub workers:Option<usize>,pub execution_available:bool,pub native_owner:super::runtime_owner::Observation,#[cfg(target_os="linux")]#[serde(skip_serializing_if="Option::is_none")]pub requested:Option<requested::DispatchStatus>}
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(Error::InvalidInput)}}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_claim(app:AppHandle,window:tauri::WebviewWindow,input:ClaimInput)->model::Result<ClaimResult>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||claim_for_app(&app,input,None)).await.map_err(|_|Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]pub(super)fn fence_for_app(app:&AppHandle,goal:&Record)->super::lease_controller::Fence{
 let app2=app.clone();let id=goal.id.clone();let pet=goal.spec.owner.pet_id.clone();
 Arc::new(move|lease:&Lease,hold:&crate::pet_resources::Hold,mode:super::lease_controller::Validation|{
     let now=crate::assistant_memory::now();
     // The native transaction already holds the goal lock and
     // supplies its exact loaded record; never re-enter that lock.
     if let super::lease_controller::Validation::Commit(goal)=mode{return super::native_validity::observe(&app2,goal,lease,hold,mode,now);}
     let store=match Store::existing_native(&root()){Ok(Some(store))=>store,Err(Error::Busy)=>return Validity::Busy,_=>return Validity::Unknown};let goal=match super::native_validity::read_goal(&store,&id,&pet){Ok(goal)=>goal,Err(validity)=>return validity};
     super::native_validity::observe(&app2,&goal,lease,hold,mode,now)
 })
}

#[cfg(target_os="linux")]
fn claim_for_app(app:&AppHandle,input:ClaimInput,resource:Option<Arc<crate::pet_resources::Hold>>)->model::Result<ClaimResult>{
        let store=Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;let now=crate::assistant_memory::now();
        if !service.entries.contains_key(&goal.id){
            if service.entries.len()>=64{return Err(Error::LimitReached);}if goal.revision!=input.revision{return Err(Error::Conflict);}
            let observation=admission_review::evidence(app,&goal,now,resource.as_deref())?.map_err(|_|Error::Conflict)?;
            let authority_path=admission_review::authority_root();let context_path=crate::settings::config_dir().join("pet-runtime-contexts-v1");
            let fence=fence_for_app(app,&goal);
            let pending=if let Some(hold)=resource.clone(){service.controller.prepare_requested(&goal,&authority_path,&context_path,app.state::<crate::chat::Chat>().inner(),&observation,input.request_id.clone(),now,fence,hold)?}else{service.controller.prepare(&goal,&authority_path,&context_path,app.state::<crate::chat::Chat>().inner(),&observation,input.request_id.clone(),now,fence)?};
            service.entries.insert(goal.id.clone(),Entry{source_revision:input.revision.clone(),request_id:input.request_id.clone(),pending,owned:None});
            maintenance::start(app.clone(),root(),authority_path);
        }
        let entry=service.entries.get_mut(&goal.id).ok_or(Error::Conflict)?;
        if entry.source_revision!=input.revision||entry.request_id!=input.request_id{return Err(Error::Conflict);}
        if let Some(owned)=&entry.owned{return Ok(ClaimResult{receipt:model::Receipt{record:goal,status:model::CommitStatus::Unchanged},lease:owned.lease().clone(),owned:true,reservation_owned:entry.pending.hold().occupies_slot(),execution_available:false});}
        let observation=admission_review::evidence(app,&goal,crate::assistant_memory::now(),Some(entry.pending.hold())).inspect_err(|e|{#[cfg(debug_assertions)]eprintln!("[pet-runtime-claim] pending-evidence: {e:?}");})?.map_err(|_|Error::Conflict)?;
        let authority=Authority::existing(&admission_review::authority_root()).map_err(|_|Error::StorageUnavailable)?;
        let(receipt,owned)=entry.pending.claim(&store,&authority,&observation,crate::assistant_memory::now())?;entry.owned=owned.map(Arc::new);
        Ok(ClaimResult{receipt,lease:entry.pending.lease().clone(),owned:entry.owned.is_some(),reservation_owned:entry.pending.hold().occupies_slot(),execution_available:false})
}

#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_claim_status(window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<LeaseStatus>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        // Status observes authenticated in-flight publication. Strict execution
        // validation still belongs to the original native owner/operation.
        let store=Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&goal_id,&pet_id)?;let service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
        let entry=service.entries.get(&goal.id);let owned=entry.and_then(|e|e.owned.as_ref());let stored=goal.admissions.last().and_then(|a|a.lease.clone());
        let lease=entry.map(|e|e.pending.lease().clone()).or(stored);let requested=requested::status(&service,&goal);let source=if entry.is_some(){"liveController"}else if lease.is_some(){"storedLease"}else if requested.is_some(){"queuedRequest"}else{"none"};
        let native_owner=if let Some(row)=goal.admissions.last(){let proof=row.proof.as_ref().ok_or(Error::Conflict)?;Authority::existing(&admission_review::authority_root()).and_then(|a|a.owner_observation(&row.spec,proof)).map_err(|_|Error::StorageUnavailable)?}else{super::runtime_owner::Observation::without_owner(super::runtime_owner::State::NotClaimed,None)};
        Ok(LeaseStatus{current:owned.is_some_and(|o|o.current(crate::assistant_memory::now())),workers:entry.map(|e|e.pending.workers()),owned:owned.is_some(),reservation_owned:entry.is_some_and(|e|e.pending.hold().occupies_slot()),source,lease,record:goal,execution_available:false,native_owner,requested})
    }).await.map_err(|_|Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_release_claim(window:tauri::WebviewWindow,input:ReleaseInput)->model::Result<model::Receipt>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let store=Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;let entry=service.entries.get(&goal.id).ok_or(Error::Conflict)?;
        if entry.pending.lease().id!=input.lease_id{return Err(Error::Conflict);}
        // The public API cannot assert native effect completion. Actual future
        // dispatcher receipts may use Confirmed after their IO is reaped.
        let drain=if let Some(owned)=&entry.owned{owned.drain(DrainOutcome::Uncertain)?}else{entry.pending.abort()?};
        let authority=Authority::existing(&admission_review::authority_root()).map_err(|_|Error::StorageUnavailable)?;
        let receipt=store.drain_owned(&authority,&drain,crate::assistant_memory::now())?;
        if receipt.status!=model::CommitStatus::Uncertain{let retired=service.entries.remove(&goal.id);drop(service);drop(retired);super::step_api::release(&input.lease_id);super::recipe_api::release(&input.lease_id);}Ok(receipt)
    }).await.map_err(|_|Error::StorageUnavailable)?
}

#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_claim(app:AppHandle,window:tauri::WebviewWindow,input:ClaimInput)->model::Result<ClaimResult>{check(&window)?;let _=(app,input);Err(Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_claim_status(window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<LeaseStatus>{check(&window)?;let _=(pet_id,goal_id);Err(Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_release_claim(window:tauri::WebviewWindow,input:ReleaseInput)->model::Result<model::Receipt>{check(&window)?;let _=input;Err(Error::UnsupportedPlatform)}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RespondInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub request_id:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RespondResult{pub text:String,pub criteria_verified:bool,pub goal_complete:bool}
/// One exact approved goal intent. This does not execute plans/tools or declare
/// criteria satisfied. A retry with the same request is receipt/recovery only.
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_respond_once(app:AppHandle,window:tauri::WebviewWindow,input:RespondInput)->Result<RespondResult,String>{
 check(&window).map_err(|e|format!("{e:?}"))?;
 let(owner,goal,settings,config)=tauri::async_runtime::spawn_blocking({let app=app.clone();move||->model::Result<_>{
  let store=Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
  if !model::key(&input.request_id)||goal.revision!=input.revision{return Err(Error::Conflict);}
  let controller=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
  let owner=controller.entries.get(&goal.id).and_then(|e|e.owned.clone()).ok_or(Error::Conflict)?;
  drop(controller);
  if owner.lease().id!=input.lease_id||!owner.policy_current(crate::assistant_memory::now()){return Err(Error::Conflict);}
  let settings=app.state::<crate::Shared>().settings.lock().map_err(|_|Error::StorageUnavailable)?.clone();let config=crate::pet_config::read_existing().map_err(|_|Error::StorageUnavailable)?;Ok((owner,(goal,input.request_id),settings,config))
 }}).await.map_err(|_|"O controlador está indisponível.")?.map_err(|e|format!("{e:?}"))?;
 let server=owner.server_projection(&settings)?;
 let text=owner.respond_once(Some(&app),&goal.0,&server,&goal.1,&config).await?;Ok(RespondResult{text,criteria_verified:false,goal_complete:false})
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_respond_once(app:AppHandle,window:tauri::WebviewWindow,input:RespondInput)->Result<RespondResult,String>{check(&window).map_err(|e|format!("{e:?}"))?;let _=(app,input);Err("UnsupportedPlatform".into())}
