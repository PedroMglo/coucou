use super::*;
use crate::pet_runtime::{authority::{Authority,GrantPhase},recovery_controller::{Prepared,Review,Approved},recovery::Reference};
impl Store{
    pub(in crate::pet_runtime)fn prepare_recovery(&self,path:&Path,id:&str,pet:&str,revision:&str,request:String,now:u64)->Result<Prepared>{
        let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=pet||goal.revision!=revision||!key(&request)||now<goal.updated_at{return Err(Error::Conflict);}
        let authority=Authority::existing(path).map_err(crate::pet_runtime::authority::Failure::control_error)?;let body=authority.recovery_body(&goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;
        let pending=authority.verify_recovered_goal(&goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;
        if let Some(c)=&body.recovery{if !pending{authority.acknowledge_recovery(&goal,&c.reference).map_err(crate::pet_runtime::authority::Failure::control_error)?;return Ok(Prepared::Closed(self.stabilize(goal)));}}
        if body.recovery.is_none()&&matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)&&matches!(goal.state,State::Draft|State::Paused|State::Cancelled){return Ok(Prepared::Closed(self.stabilize(goal)));}
        let(body,gate,snapshot,fence)=authority.recovery_seed(&goal,now).map_err(crate::pet_runtime::authority::Failure::control_error)?;
        Ok(Prepared::Review(Review::captured(goal,authority,gate,body,snapshot,fence,request,now)?))
    }
    pub(in crate::pet_runtime)fn confirm_recovery(&self,approved:&mut Approved,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let(goal,before)=self.load_with_pending(&approved.goal.id,true)?.ok_or(Error::NotFound)?;
        if now<approved.closure.reference.approved_at||now<goal.updated_at{return Err(Error::ClockRollback);}
        if goal.spec.owner!=approved.goal.spec.owner{return Err(Error::Conflict);}
        if !approved.closure.reference.source_matches(&goal){
            approved.authority.acknowledge_recovery(&goal,&approved.closure.reference).map_err(crate::pet_runtime::authority::Failure::control_error)?;return Ok(self.stabilize(goal));
        }
        let mut started=approved.write_started;let result=approved.authority.close_lost_owner(approved,&mut started);approved.write_started=started;result.map_err(crate::pet_runtime::authority::Failure::control_error)?;
        if approved.closure.reference.target_state==State::Completed{return Ok(self.stabilize(goal));}
        self.publish(approved.candidate.clone(),Some(&before),true)
    }
    pub(in crate::pet_runtime)fn ack_recovery(&self,authority:&Authority,reference:&Reference)->Result<Receipt>{
        let _guard=self.lock(true)?;let(goal,_)=self.load_with_pending(&reference.goal_id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=reference.pet_id{return Err(Error::Conflict);}authority.acknowledge_recovery(&goal,reference).map_err(crate::pet_runtime::authority::Failure::control_error)?;Ok(self.stabilize(goal))
    }
    pub(in crate::pet_runtime)fn withdraw_recovery(&self,approved:&Approved)->Result<bool>{
        let _guard=self.lock(true)?;let(goal,_)=self.load_with_pending(&approved.goal.id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner!=approved.goal.spec.owner{return Err(Error::Conflict);}approved.authority.recovery_uncommitted(approved).map_err(crate::pet_runtime::authority::Failure::control_error)
    }
}

impl Store{
    #[cfg(test)]pub(in crate::pet_runtime)fn prepare_continuation(&self,path:&Path,id:&str,pet:&str,revision:&str,now:u64)->Result<std::result::Result<(Record,crate::pet_runtime::continuation::Captured),crate::pet_runtime::admission::Denial>>{
        self.prepare_continuation_review(path,id,pet,revision,now,false)
    }
    pub(in crate::pet_runtime)fn prepare_continuation_review(&self,path:&Path,id:&str,pet:&str,revision:&str,now:u64,review_uncertain_effects:bool)->Result<std::result::Result<(Record,crate::pet_runtime::continuation::Captured),crate::pet_runtime::admission::Denial>>{
        let _guard=self.lock(false)?;let(goal,_)=self.load(id)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=pet||goal.revision!=revision||now<goal.updated_at{return Err(Error::Conflict);}
        let authority=Authority::existing(path).map_err(crate::pet_runtime::authority::Failure::control_error)?;self.bound_authority(&authority)?;
        Ok(authority.capture_continuation(&goal,now,review_uncertain_effects).map_err(crate::pet_runtime::authority::Failure::control_error)?.map(|source|(goal,source)))
    }
}
