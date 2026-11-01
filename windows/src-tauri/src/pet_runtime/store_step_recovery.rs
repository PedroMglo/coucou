//! Data-only receipt recovery uses the original goal-before-authority order.
//! A new admission cannot freeze a prefix between this load and settlement.
use super::*;
use crate::pet_runtime::{authority::{Authority,Failure},step_api::{RecoverInput,View}};
impl Store{
    pub(in crate::pet_runtime)fn recover_step_native(&self,path:&Path,input:&RecoverInput,now:u64)->Result<View>{
        let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(&input.goal_id,true)?.ok_or(Error::NotFound)?;
        if goal.spec.owner.pet_id!=input.pet_id||goal.revision!=input.revision{return Err(Error::Conflict);}
        let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
        crate::pet_runtime::step_api::recover_for_goal(&authority,&goal,input,now)
    }
}
