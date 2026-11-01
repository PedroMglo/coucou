//! Only a durable canonical readback can construct PlanPublished. The proof
//! retains the Store boundary; a parsed record/UI ACK cannot substitute for it.
use super::*;
use crate::pet_runtime::{authority::{self,Failure,PlanCertified},generated_plans::{self,Reference},planning_output::Envelope};

pub(in crate::pet_runtime)struct PlanCommit<'a> {pub receipt:Receipt,pub published:Option<PlanPublished<'a>>}
pub(in crate::pet_runtime)struct PlanPublished<'a> {store:&'a Store,reference:Reference,output:Envelope}
pub(in crate::pet_runtime)struct PlanAbsent<'a> {store:&'a Store,goal_id:String,reference:Reference}
fn failure(e:Error)->Failure {match e{Error::Busy=>Failure::Busy,Error::UnsafePath=>Failure::Unsafe,Error::StorageUnavailable=>Failure::Unavailable,_=>Failure::Changed}}
impl PlanPublished<'_> {
    pub(in crate::pet_runtime)fn with_current<T>(&self,run:impl FnOnce(&Record,&Reference)->authority::Result<T>)->authority::Result<T> {
        let _guard=self.store.lock(false).map_err(failure)?;
        let (goal,_)=self.store.load_with_pending(self.store_id(),true).map_err(failure)?.ok_or(Failure::Missing)?;
        if !generated_plans::content_matches(&goal,&self.reference,&self.output){return Err(Failure::Changed);}
        if self.store.stabilize(goal.clone()).status==CommitStatus::Uncertain{return Err(Failure::Uncertain);}
        run(&goal,&self.reference)
    }
    fn store_id(&self)->&str {&self.output.body.goal_id}
}
impl PlanAbsent<'_> {
    pub(in crate::pet_runtime)fn with_current<T>(&self,run:impl FnOnce(&Record,&Reference)->authority::Result<T>)->authority::Result<T> {
        let _guard=self.store.lock(false).map_err(failure)?;
        let (goal,_)=self.store.load_with_pending(&self.goal_id,true).map_err(failure)?.ok_or(Failure::Missing)?;
        if matches!(goal.state,State::Draft|State::Queued|State::Leased|State::Completed)||goal.generated_plans.iter().any(|r|r.output.attempt_id==self.reference.output.attempt_id)
            ||goal.tasks.iter().any(|t|t.id==self.reference.task_id)||goal.plans.iter().any(|p|p.id==self.reference.plan_id){return Err(Failure::Changed);}
        if self.store.stabilize(goal.clone()).status==CommitStatus::Uncertain{return Err(Failure::Uncertain);}
        run(&goal,&self.reference)
    }
}
impl Store {
    pub(in crate::pet_runtime)fn plan_absence(&self,goal_id:&str,reference:&Reference)->PlanAbsent<'_> {
        PlanAbsent {store:self,goal_id:goal_id.into(),reference:reference.clone()}
    }
    pub(in crate::pet_runtime)fn commit_generated<'a>(&'a self,certificate:&PlanCertified<'_,'_>,now:u64)->Result<PlanCommit<'a>> {
        let _guard=self.lock(true)?;let (mut goal,before)=self.load_with_pending(certificate.goal_id(),true)?.ok_or(Error::NotFound)?;
        certificate.current(&goal,now).map_err(|reason|reason.control_error())?;
        let reference=certificate.reference();
        let receipt=if goal.generated_plans.contains(reference) {self.stabilize(goal)}
        else {
            let source=goal.clone();
            goal.append_generated(reference.clone(),certificate.output())?;
            certificate.current(&source,now).map_err(|reason|reason.control_error())?;self.publish(goal,Some(&before),false)?
        };
        if receipt.status==CommitStatus::Uncertain{return Ok(PlanCommit {receipt,published:None});}
        let (readback,_)=self.load_with_pending(certificate.goal_id(),true)?.ok_or(Error::NotFound)?;
        if fingerprint(&readback)?!=fingerprint(&receipt.record)?||!generated_plans::content_matches(&readback,reference,certificate.output()){return Err(Error::Conflict);}
        Ok(PlanCommit {receipt,published:Some(PlanPublished {store:self,reference:reference.clone(),output:certificate.output().clone()})})
    }
    /// Internal data/control inspection can see authenticated Prepared entries.
    /// It cannot issue a worker, model channel or effect permit.
    pub(in crate::pet_runtime)fn read_control(&self,id:&str,pet:&str)->Result<Record> {
        let _guard=self.lock(false)?;let (goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=pet{return Err(Error::Conflict);}self.require_current_goal(&goal)?;Ok(goal)
    }
}
