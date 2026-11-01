//! Same original owner; a new physical generation after each authenticated wait.
//! No serialized wait entry can construct this native zero-worker proof.
use super::*;
use crate::pet_runtime::{resource_wait::Entry,recipe_decisions::Source};
#[derive(Clone,Copy,PartialEq,Eq)]enum Stage{Prepared,Released,Acquired}
#[derive(Clone)]pub(super)struct Waiting{entry:Entry,stage:Stage}
impl Waiting{
    pub(super)fn entry(&self)->&Entry{&self.entry}
    pub(super)fn released(&self)->bool{self.stage==Stage::Released}
}
pub(crate)struct YieldProof{live:Arc<Live>,entry:Entry}
impl YieldProof{
    pub(in crate::pet_runtime)fn entry(&self)->&Entry{&self.entry}
    pub(crate)fn generation(&self)->u64{self.entry.generation}
    pub(crate)fn next_generation(&self)->u64{self.entry.generation+1}
    fn same_owner(&self,hold:&Hold)->bool{
        self.live.activated.load(Ordering::Acquire)&&!self.live.closed.load(Ordering::Acquire)
            &&self.live.workers.load(Ordering::Acquire)==0&&std::ptr::eq(Arc::as_ptr(&self.live.hold),hold)
    }
    pub(crate)fn covers_release(&self,hold:&Hold)->bool{
        self.same_owner(hold)&&hold.generation()==self.generation()
            &&self.live.waiting.lock().is_ok_and(|w|w.as_ref().is_some_and(|w|w.entry==self.entry&&w.stage==Stage::Released))
    }
    pub(crate)fn covers_reacquire(&self,hold:&Hold)->bool{
        self.same_owner(hold)&&[self.generation(),self.next_generation()].contains(&hold.generation())
            &&self.live.waiting.lock().is_ok_and(|w|w.as_ref().is_some_and(|w|w.entry==self.entry&&matches!(w.stage,Stage::Released|Stage::Acquired)))
    }
    pub(in crate::pet_runtime)fn authorizes(&self,spec:&AdmissionSpec,lease:&Lease)->bool{
        self.same_owner(&self.live.hold)&&self.live.lease==*lease
            &&model::fingerprint(spec).ok()==model::fingerprint(&self.live.spec).ok()
            &&[self.generation(),self.next_generation()].contains(&self.live.hold.generation())
            &&self.live.waiting.lock().is_ok_and(|w|w.as_ref().is_some_and(|w|w.entry==self.entry))
    }
    pub(in crate::pet_runtime)fn authorizes_resume(&self,spec:&AdmissionSpec,lease:&Lease)->bool{
        self.authorizes(spec,lease)&&self.live.hold.generation()==self.next_generation()&&self.live.hold.current()
            &&self.live.waiting.lock().is_ok_and(|w|w.as_ref().is_some_and(|w|w.stage==Stage::Acquired))
    }
}
fn failure(f:crate::pet_runtime::authority::Failure)->model::Error{match f{crate::pet_runtime::authority::Failure::Busy=>model::Error::Busy,crate::pet_runtime::authority::Failure::Unavailable|crate::pet_runtime::authority::Failure::Uncertain=>model::Error::StorageUnavailable,_=>model::Error::Conflict}}
impl Owned{
    fn wait_inputs(&self,store:&Store,source:&Source,entry:&Entry,now:u64)->model::Result<()>{
        if self.live.closed.load(Ordering::Acquire)||Instant::now()>=self.live.deadline||!self.live.chat.goal_context_current(&self.live.lease.goal_id)
            ||!source.matches_wait(self,entry,now){self.live.closed.store(true,Ordering::Release);self.live.hold.cancel();return Err(model::Error::Conflict);}
        let goal=store.read(&self.live.lease.goal_id,&self.live.spec.owner.pet_id)?;
        if goal.revision!=entry.goal_revision||model::fingerprint(&goal)?!=entry.goal_hash||!source.wait_configuration_current()?{self.live.hold.cancel();return Err(model::Error::Conflict);}
        match(self.live.fence)(&self.live.lease,&self.live.hold,Validation::PendingDecision){
            Validity::Current=>Ok(()),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>{self.live.hold.cancel();Err(model::Error::Conflict)},
        }
    }
    /// Publication uncertainty retains this native proposal and physical slot.
    /// Retrying publishes/ACKs the same transition; it creates no worker or IO.
    pub(in crate::pet_runtime)fn yield_for_decision(&self,store:&Store,source:&Source,now:u64)->model::Result<()>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if self.workers()!=0{return Err(model::Error::Busy);}
        let existing=self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?.clone();
        let waiting=if let Some(w)=existing{w}else{
            let entry=source.resource_entry(self,now,self.live.hold.generation())?;
            let w=Waiting{entry,stage:Stage::Prepared};*self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?=Some(w.clone());w
        };
        if waiting.stage==Stage::Acquired{return Err(model::Error::Conflict);}
        self.wait_inputs(store,source,&waiting.entry,now)?;
        let native=YieldProof{live:self.live.clone(),entry:waiting.entry};
        let authority=Authority::existing(&self.live.authority_path).map_err(failure)?;
        authority.yield_resource(&self.live.spec,&self.live.proof,&self.live.lease,&native,now).map_err(failure)?;
        self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?.as_mut().ok_or(model::Error::Conflict)?.stage=Stage::Released;
        self.live.hold.release_waiting(&native).map_err(|_|model::Error::StorageUnavailable)
    }
    /// A busy resource is a bounded wait, never a new controller or model call.
    /// An uncertain resume keeps the newly acquired generation until its ACK.
    pub(in crate::pet_runtime)fn resume_after_decision(&self,store:&Store,source:&Source,now:u64)->model::Result<()>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if self.workers()!=0{return Err(model::Error::Busy);}
        let waiting=self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?.clone();
        let Some(waiting)=waiting else{return if self.policy_current(now){Ok(())}else{Err(model::Error::Conflict)};};
        if waiting.stage==Stage::Prepared{return Err(model::Error::StorageUnavailable);}
        self.wait_inputs(store,source,&waiting.entry,now)?;
        let native=YieldProof{live:self.live.clone(),entry:waiting.entry};
        if waiting.stage==Stage::Released{
            self.live.hold.reacquire_waiting(&native).map_err(|e|if e==crate::pet_resources::Failure::Busy{model::Error::Busy}else{model::Error::StorageUnavailable})?;
            self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?.as_mut().ok_or(model::Error::Conflict)?.stage=Stage::Acquired;
        }
        let authority=Authority::existing(&self.live.authority_path).map_err(failure)?;
        authority.resume_resource(&self.live.spec,&self.live.proof,&self.live.lease,&native,now).map_err(failure)?;
        *self.live.waiting.lock().map_err(|_|model::Error::StorageUnavailable)?=None;Ok(())
    }
    pub(in crate::pet_runtime)fn waiting_for_resource(&self)->bool{self.live.waiting.lock().is_ok_and(|w|w.is_some())}
}
#[cfg(test)]#[path="resource_wait_tests.rs"]mod tests;
