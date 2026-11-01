//! Cleanup of original native preparations only. This loop never schedules
//! work, reconstructs ownership, activates a grant or performs inference.
use super::*;
use std::path::{Path,PathBuf};

fn cleanup(service:&mut Service,store:&Store,path:&Path,goal:&Record,now:u64)->model::Result<Option<model::Receipt>>{
    let Some(entry)=service.entries.get(&goal.id)else{return Ok(None);};
    if entry.owned.is_some(){return Ok(None);}
    if !entry.pending.matches_goal(goal){return Err(Error::Conflict);}
    if !entry.pending.cleanup_due(goal,now){return Ok(None);}
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

fn sweep(service:&mut Service,store:&Store,path:&Path,now:u64){
    // Rotate through at most eight of the bounded 64 entries. A persistently
    // unavailable original source cannot starve independent pets.
    let keys:Vec<_>=service.entries.iter().filter(|(_,e)|e.owned.is_none()).map(|(id,_)|id.clone()).collect();
    let pivot=service.maintenance_after.as_ref().map_or(0,|after|keys.partition_point(|id|id<=after));
    for id in keys[pivot..].iter().chain(keys[..pivot].iter()).take(8){
        service.maintenance_after=Some(id.clone());
        let Some(pet)=service.entries.get(id).and_then(|e|crate::pet_config::pet_id(&e.pending.lease().provider))else{continue;};
        if let Ok(goal)=store.read_control(id,pet){let _=cleanup(service,store,path,&goal,now);}
    }
}

#[cfg(not(test))]
pub(super)fn start(root:PathBuf,authority:PathBuf){
    use std::sync::atomic::{AtomicBool,Ordering};
    static STARTED:AtomicBool=AtomicBool::new(false);
    if STARTED.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return;}
    let started=std::thread::Builder::new().name("pet-lease-cleanup".into()).spawn(move||loop{
        std::thread::sleep(std::time::Duration::from_millis(200));
        if crate::pet_data::bootstrap::write_allowed().is_err(){continue;}
        let Ok(mutex)=service()else{continue;};let Ok(mut service)=mutex.try_lock()else{continue;};
        if service.entries.values().all(|e|e.owned.is_some()){continue;}
        let Ok(Some(store))=Store::existing_verified(&root,&authority)else{continue;};
        sweep(&mut service,&store,&authority,crate::assistant_memory::now());
    });
    if started.is_err(){STARTED.store(false,Ordering::Release);}
}
#[cfg(test)]pub(super)fn start(_root:PathBuf,_authority:PathBuf){}

#[cfg(test)]#[path="lease_maintenance_tests.rs"]mod tests;
