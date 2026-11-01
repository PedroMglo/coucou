//! Goal-before-authority transaction for unused-grant retirement. The canonical
//! goal is observed and fenced, never rewritten by this operation.
use super::*;
use crate::pet_runtime::{authority::{Authority,Failure},retirement::{Receipt as RetirementReceipt,Reference},retirement_controller::{Prepared,Approved}};
impl Store{
    pub(in crate::pet_runtime::store)fn bound_authority(&self,authority:&Authority)->Result<()>{
        let expected=self.authority.as_deref().ok_or(Error::UnsafePath)?;
        if !authority.matches_root(expected).map_err(Failure::control_error)?{return Err(Error::Conflict);}Ok(())
    }
    pub(in crate::pet_runtime)fn prepare_retirement(&self,path:&Path,id:&str,pet:&str,revision:&str,decision:&str,request:String,now:u64)->Result<Prepared>{
        let _guard=self.lock(false)?;let(goal,_)=self.load(id)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=pet||goal.revision!=revision||!token(decision)||!key(&request)||now<goal.updated_at{return Err(Error::Conflict);}
        let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;authority.prepare_retirement(goal,decision,request,now).map_err(Failure::control_error)
    }
    pub(in crate::pet_runtime)fn confirm_retirement(&self,approved:&mut Approved,now:u64)->Result<RetirementReceipt>{
        self.bound_authority(&approved.authority)?;let _guard=self.lock(true)?;let(goal,_)=self.load(&approved.goal.id)?.ok_or(Error::NotFound)?;
        if goal.spec.owner!=approved.goal.spec.owner||now<goal.updated_at||now<approved.reference.approved_at{return Err(Error::Conflict);}
        let was_started=approved.write_started;let mut started=was_started;let result=approved.authority.retire_unused(&goal,&approved.source,&approved.reference,&mut started);approved.write_started=started;let published=result.map_err(Failure::control_error)?;
        Ok(RetirementReceipt{status:if published{CommitStatus::Durable}else{CommitStatus::Unchanged},record:goal,reference:approved.reference.clone(),execution_available:false,new_admission_created:false})
    }
    pub(in crate::pet_runtime)fn ack_retirement(&self,authority:&Authority,reference:&Reference)->Result<RetirementReceipt>{
        self.bound_authority(authority)?;let _guard=self.lock(false)?;let(goal,_)=self.load(&reference.goal_id)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=reference.pet_id{return Err(Error::Conflict);}authority.acknowledge_retirement(&goal,reference).map_err(Failure::control_error)?;
        Ok(RetirementReceipt{status:CommitStatus::Unchanged,record:goal,reference:reference.clone(),execution_available:false,new_admission_created:false})
    }
    pub(in crate::pet_runtime)fn withdraw_retirement(&self,approved:&Approved)->Result<bool>{
        self.bound_authority(&approved.authority)?;let _guard=self.lock(false)?;let(goal,_)=self.load(&approved.goal.id)?.ok_or(Error::NotFound)?;
        if goal.spec.owner!=approved.goal.spec.owner{return Err(Error::Conflict);}approved.authority.withdraw_retirement(&goal,&approved.source,&approved.reference).map_err(Failure::control_error)
    }
}
