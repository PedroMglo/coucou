//! One bounded lease-service loop: native preparation cleanup and explicitly
//! requested dispatch. Cleanup never reconstructs ownership or grants.
use super::*;
use std::path::{Path,PathBuf};

pub(super)fn cleanup(service:&mut Service,store:&Store,path:&Path,goal:&Record,now:u64)->model::Result<Option<model::Receipt>>{
    let Some(entry)=service.entries.get(&goal.id)else{return Ok(None);};
    if entry.owned.is_some(){return Ok(None);}
    if !entry.pending.matches_goal(goal){return Err(Error::Conflict);}
    if !entry.pending.cleanup_due(goal,now){return Ok(None);}
    // A resumed Pending already has a canonical claimed owner. Complete only
    // that signed metadata ACK before closing its native epoch; no activation.
    let acknowledged=if entry.pending.resumed(){store.ack_peer_owner_resumption(path,&goal.id,&goal.spec.owner.pet_id,&goal.revision)?}else{None};
    if acknowledged.as_ref().is_some_and(|r|r.status==model::CommitStatus::Uncertain){return Err(Error::StorageUnavailable);}
    let proof=entry.pending.abort()?;
    let authority=Authority::existing(path).map_err(super::super::authority::Failure::control_error)?;
    let receipt=store.drain_owned(&authority,&proof,now)?;
    if receipt.status!=model::CommitStatus::Uncertain{
        entry.pending.hold().release_drained(&proof).map_err(|_|Error::StorageUnavailable)?;
        let lease=proof.lease().id.clone();service.entries.remove(&goal.id);
        super::super::step_api::release(&lease);super::super::recipe_api::release(&lease);
    }
    Ok(Some(receipt))
}

/// A caller supplies the exact control snapshot it just observed/committed.
/// Store rechecks the original admission under its exclusive lock. Historical
/// control replays must not call this helper with a newer admission.
pub(in crate::pet_runtime)fn cleanup_unstarted(store:&Store,path:&Path,goal:&Record,now:u64)->model::Result<Option<model::Receipt>>{
    let mut service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;cleanup(&mut service,store,path,goal,now)
}

/// Stop of a claimed manual owner is also cleaned after its original workers
/// end. Driver-owned closure remains with that driver. No reconstruction from
/// a stored lease or expiry; uncertain publication keeps the same native entry.
fn cleanup_stopped_owned(service:&mut Service,store:&Store,path:&Path,goal:&Record,now:u64)->model::Result<Option<model::Receipt>>{
 let Some(entry)=service.entries.get(&goal.id)else{return Ok(None);};let Some(owner)=&entry.owned else{return Ok(None);};
 if !entry.pending.matches_goal(goal){return Err(Error::Conflict);}
 if owner.driver_active()||!matches!(goal.state,model::State::Stopping|model::State::Paused|model::State::Cancelled|model::State::RecoveryRequired)||!goal.admissions.last().is_some_and(|r|matches!(r.stop_reason,Some(super::super::leases::StopReason::Pause|super::super::leases::StopReason::Cancel))){return Ok(None);}
 let proof=owner.drain(DrainOutcome::Confirmed)?;let authority=Authority::existing(path).map_err(super::super::authority::Failure::control_error)?;
 let receipt=store.drain_owned(&authority,&proof,now)?;
 if receipt.status!=model::CommitStatus::Uncertain{entry.pending.hold().release_drained(&proof).map_err(|_|Error::StorageUnavailable)?;let lease=proof.lease().id.clone();service.entries.remove(&goal.id);super::super::step_api::release(&lease);super::super::recipe_api::release(&lease);}
 Ok(Some(receipt))
}

fn sweep(service:&mut Service,store:&Store,path:&Path,now:u64){
    // Rotate through at most eight of the bounded 64 entries. A persistently
    // unavailable original source cannot starve independent pets.
    let keys:Vec<_>=service.entries.keys().cloned().collect();
    let pivot=service.maintenance_after.as_ref().map_or(0,|after|keys.partition_point(|id|id<=after));
    for id in keys[pivot..].iter().chain(keys[..pivot].iter()).take(8){
        service.maintenance_after=Some(id.clone());
        let Some(pet)=service.entries.get(id).map(|e|e.pending.pet_id().to_owned())else{continue;};
        if let Ok(goal)=store.read_control(id,&pet){let _=cleanup(service,store,path,&goal,now);let _=cleanup_stopped_owned(service,store,path,&goal,now);}
    }
}

#[cfg(not(test))]
pub(super)fn start(app:AppHandle,root:PathBuf,authority:PathBuf){
    use std::sync::atomic::{AtomicBool,Ordering};
    static STARTED:AtomicBool=AtomicBool::new(false);
    if STARTED.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return;}
    let started=std::thread::Builder::new().name("pet-lease-runtime".into()).spawn(move||loop{
        std::thread::sleep(std::time::Duration::from_millis(200));
        // Lifetime cleanup reduces authority and needs neither readable goal
        // catalogs nor permission to start a new effect in the current profile.
        requested::housekeep();
        super::super::rules::recovery_api::expire_reviews();
        super::super::rules::intent_retirement_api::expire_reviews();
        super::super::rules::succession_api::expire_reviews();
        super::super::rules::revision_api::expire_reviews();
        crate::pet_runtime::recovery_api::sweep_peer_resumptions();
        if crate::pet_data::bootstrap::write_allowed().is_err(){continue;}
        {
            let Ok(mutex)=service()else{continue;};let Ok(mut service)=mutex.try_lock()else{continue;};
            if !service.entries.is_empty(){
                if let Ok(Some(store))=Store::existing_verified(&root,&authority){sweep(&mut service,&store,&authority,crate::assistant_memory::now());}
            }
        }
        // No service/cache mutex is held during metadata IO or driver start.
        tauri::async_runtime::block_on(requested::tick(&app,&root,&authority));
        super::super::rules::scheduled::tick(app.clone(),root.clone(),authority.clone());
    });
    if started.is_err(){STARTED.store(false,Ordering::Release);}
}
#[cfg(test)]pub(super)fn start(_app:AppHandle,_root:PathBuf,_authority:PathBuf){}

#[cfg(test)]#[path="lease_maintenance_tests.rs"]mod tests;
