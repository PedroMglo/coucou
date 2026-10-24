//! Canonical publication uses the original live owner, its gate and no workers.
//! This operation writes plan data only; it neither approves nor starts effects.
use super::*;
use serde::Serialize;
/// Native gate, never Deserialize. Holding this value prevents another native
/// worker/publication from entering the original controller concurrently.
pub(in crate::pet_runtime)struct PlanGate<'a> {owned:&'a Owned,_guard:std::sync::MutexGuard<'a,()>}
impl PlanGate<'_> {
    pub(in crate::pet_runtime)fn matches(&self,spec:&AdmissionSpec,lease:&Lease)->bool {
        self.owned.live.lease==*lease&&model::fingerprint(&self.owned.live.spec).ok()==model::fingerprint(spec).ok()
    }
    pub(in crate::pet_runtime)fn current(&self,goal:&Record,now:u64)->bool {self.owned.completion_current(goal,now)}
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(in crate::pet_runtime)struct PublicationReceipt {pub receipt:model::Receipt,pub acknowledged:bool}
impl Owned {
    pub(in crate::pet_runtime)fn abandon_unpublished_plan(&self,store:&Store,request:&str,now:u64)->model::Result<()> {
        let gate=self.plan_gate()?;let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
        let reference=authority.generated_reference(&self.live.spec,&self.live.proof,&self.live.lease,request,now).map_err(|_|model::Error::Conflict)?;
        let absent=store.plan_absence(&self.live.spec.goal_id,&reference);
        authority.abandon_unpublished_plan(&self.live.spec,&self.live.proof,&self.live.lease,&gate,&absent,now).map_err(|_|model::Error::Conflict)
    }
    pub(in crate::pet_runtime)fn plan_gate(&self)->model::Result<PlanGate<'_>> {
        let guard=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if self.workers()!=0{return Err(model::Error::Busy);}Ok(PlanGate {owned:self,_guard:guard})
    }
    pub(in crate::pet_runtime)fn publish_generated_plan(&self,store:&Store,request:&str,now:u64)->model::Result<PublicationReceipt> {
        let gate=self.plan_gate()?;
        if self.workers()!=0||!model::key(request){return Err(model::Error::Busy);}
        let goal=store.read_control(&self.live.spec.goal_id,&self.live.spec.owner.pet_id)?;
        if goal.admissions.iter().find(|a|a.spec.decision_id==self.live.spec.decision_id).and_then(|a|a.lease.as_ref())!=Some(&self.live.lease){return Err(model::Error::Conflict);}
        let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
        let (_,attempt)=authority.result_attempt(&self.live.spec,&self.live.proof,request).map_err(|_|model::Error::StorageUnavailable)?;
        let attempt=attempt.ok_or(model::Error::NotFound)?;
        let output=crate::pet_runtime::planning_output::read(&authority,&self.live.spec,&self.live.proof,&attempt.id).map_err(|_|model::Error::StorageUnavailable)?.ok_or(model::Error::NotFound)?;
        let certificate=authority.prepare_generated_plan(&self.live.spec,&self.live.proof,&self.live.lease,&goal,&output,&gate,now).map_err(|_|model::Error::Conflict)?;
        let mut commit=store.commit_generated(&certificate,now)?;
        let acknowledged=if let Some(proof)=&commit.published {authority.acknowledge_generated_plan(&self.live.spec,&self.live.proof,&self.live.lease,&gate,proof,now).is_ok()}else{false};
        if !acknowledged{commit.receipt.status=model::CommitStatus::Uncertain;}
        Ok(PublicationReceipt {receipt:commit.receipt,acknowledged})
    }
}
