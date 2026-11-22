//! Only a durable canonical readback can construct PlanPublished. The proof
//! retains the Store boundary; a parsed record/UI ACK cannot substitute for it.
use super::*;
use crate::pet_runtime::{authority::{self,Failure,PlanCertified},generated_plans::Reference,plan_publication::{Certificate,Change,Generated,Native}};

pub(in crate::pet_runtime)struct Commit<'a,P:Change> {pub receipt:Receipt,pub published:Option<Published<'a,P>>}
pub(in crate::pet_runtime)struct Published<'a,P:Change> {store:&'a Store,change:P}
pub(in crate::pet_runtime)type PlanCommit<'a> = Commit<'a,Generated>;
pub(in crate::pet_runtime)type PlanPublished<'a> = Published<'a,Generated>;
pub(in crate::pet_runtime)struct Absent<'a,P:Change> {store:&'a Store,change:P}
pub(in crate::pet_runtime)type PlanAbsent<'a> = Absent<'a,Generated>;
pub(in crate::pet_runtime)type NativePlanPublished<'a> = Published<'a,Native>;
pub(in crate::pet_runtime)type NativePlanAbsent<'a> = Absent<'a,Native>;
fn failure(e:Error)->Failure {match e{Error::Busy=>Failure::Busy,Error::UnsafePath=>Failure::Unsafe,Error::StorageUnavailable=>Failure::Unavailable,_=>Failure::Changed}}
impl<P:Change> Published<'_,P> {
    pub(in crate::pet_runtime)fn with_current<T>(&self,run:impl FnOnce(&Record,&P::Reference)->authority::Result<T>)->authority::Result<T> {
        let _guard=self.store.lock(false).map_err(failure)?;
        let (goal,_)=self.store.load_with_pending(self.change.goal_id(),true).map_err(failure)?.ok_or(Failure::Missing)?;
        if !self.change.content_matches(&goal){return Err(Failure::Changed);}
        if self.store.stabilize(goal.clone()).status==CommitStatus::Uncertain{return Err(Failure::Uncertain);}
        run(&goal,self.change.reference())
    }
}
impl<P:Change> Absent<'_,P> {
    pub(in crate::pet_runtime)fn with_current<T>(&self,run:impl FnOnce(&Record,&P::Reference)->authority::Result<T>)->authority::Result<T> {
        let _guard=self.store.lock(false).map_err(failure)?;
        let (goal,_)=self.store.load_with_pending(self.change.goal_id(),true).map_err(failure)?.ok_or(Failure::Missing)?;
        if !self.change.absent_in(&goal){return Err(Failure::Changed);}
        if self.store.stabilize(goal.clone()).status==CommitStatus::Uncertain{return Err(Failure::Uncertain);}
        run(&goal,self.change.reference())
    }
}
impl Store {
    pub(in crate::pet_runtime)fn plan_absence(&self,goal_id:&str,reference:&Reference)->PlanAbsent<'_> {
        Absent {store:self,change:Generated::absent(goal_id,reference.clone())}
    }
    pub(in crate::pet_runtime)fn native_plan_absence(&self,seal:crate::pet_runtime::native_plan_revisions::Seal)->NativePlanAbsent<'_>{Absent{store:self,change:Native::new(seal)}}
    pub(in crate::pet_runtime)fn commit_generated<'a>(&'a self,certificate:&PlanCertified<'_,'_>,now:u64)->Result<PlanCommit<'a>> {
        self.commit_plan(certificate,now)
    }
    /// Every source uses the same CAS, fresh native fences and durable readback.
    /// The content object alone cannot call this path without its certificate.
    pub(in crate::pet_runtime)fn commit_plan<'a,C:Certificate>(&'a self,certificate:&C,now:u64)->Result<Commit<'a,C::Change>> {
        let change=certificate.change();
        let observed=|stage:&str,error:Error|{#[cfg(debug_assertions)]eprintln!("[pet-plan-publication] {stage}: {error:?}");let _=stage;error};
        let _guard=self.lock(true).map_err(|e|observed("catalog lock",e))?;let (mut goal,before)=self.load_with_pending(change.goal_id(),true).map_err(|e|observed("authenticated source read",e))?.ok_or(Error::NotFound)?;
        certificate.current(&goal,now).map_err(|reason|observed("source certificate",reason.control_error()))?;
        let receipt=if change.present(&goal) {self.stabilize(goal)}
        else {
            let source=goal.clone();
            change.apply(&mut goal)?;
            certificate.current(&source,now).map_err(|reason|observed("candidate certificate",reason.control_error()))?;self.publish_current(goal,Some(&before),false,||certificate.current(&source,now).map_err(|reason|reason.control_error())).map_err(|e|observed("canonical publication",e))?
        };
        if receipt.status==CommitStatus::Uncertain{return Ok(Commit {receipt,published:None});}
        let (readback,_)=self.load_with_pending(change.goal_id(),true)?.ok_or(Error::NotFound)?;
        if fingerprint(&readback)?!=fingerprint(&receipt.record)?||!change.content_matches(&readback){return Err(Error::Conflict);}
        Ok(Commit {receipt,published:Some(Published {store:self,change})})
    }
    /// Internal data/control inspection can see authenticated Prepared entries.
    /// It cannot issue a worker, model channel or effect permit.
    pub(in crate::pet_runtime)fn read_control(&self,id:&str,pet:&str)->Result<Record> {
        let _guard=self.lock(false)?;let (goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=pet{return Err(Error::Conflict);}self.require_current_goal(&goal)?;Ok(goal)
    }
}
