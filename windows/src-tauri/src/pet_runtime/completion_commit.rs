//! Bounded completion control after all physical work and native observation.
//! Typed Busy is prepublication only; never loop model/tool/target IO or renew
//! the original lease, approval, proof, target, accounting or deadline.
use super::*;
impl Owned{
    pub(super)fn complete_retained(&self,store:&Store,goal:&Record,scope:&crate::pet_runtime::recipes::Scope)->model::Result<model::Receipt>{
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
        let until=Instant::now()+Duration::from_secs(1);
        loop{
            if self.live.closed.load(Ordering::Acquire)||self.live.hold.stopped()||Instant::now()>=self.live.deadline{return Err(model::Error::Conflict);}
            let attempt=(||{
                let certified=authority.certify_completion(&self.live.spec,&self.live.proof,scope,goal).map_err(|e|if e==crate::pet_runtime::authority::Failure::Busy{model::Error::Busy}else{model::Error::StorageUnavailable})?;
                // Only the transaction holds this gate. Stop can take priority
                // between attempts; the exact loaded goal is the policy input.
                let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
                store.complete_certified(&certified,crate::assistant_memory::now(),|current|self.completion_authority(current,crate::assistant_memory::now()))
            })();
            match attempt{
                Err(model::Error::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(10)),
                // Uncertain ACK/publication, revocation and storage failures
                // retain their original outcomes without another transaction.
                other=>return other,
            }
        }
    }
}
