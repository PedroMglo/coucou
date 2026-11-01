//! One retained native confirmation per review. A retry acknowledges or
//! publishes only that exact consent; it never renews limits or dispatches IO.
use super::*;
use crate::pet_runtime::{authority::{Authority,Failure},store::Store};
use std::path::Path;
fn transient(reason:Failure)->bool{matches!(reason,Failure::Busy|Failure::Unavailable|Failure::Uncertain)}
fn temporary(reason:Denial)->bool{matches!(reason,Denial::Stale|Denial::NotReady|Denial::ModelUnresolved)}
fn authority_blocked(reason:Failure)->ConfirmResult{ConfirmResult::AuthorityBlocked{reason,retryable:transient(reason)}}
fn storage_error(error:Error)->model::Result<ConfirmResult>{match error{
    Error::Busy=>Ok(authority_blocked(Failure::Busy)),Error::StorageUnavailable=>Ok(authority_blocked(Failure::Unavailable)),other=>Err(other),
}}
fn published(native:&mut Native,receipt:model::Receipt)->ConfirmResult{
    if receipt.status!=model::CommitStatus::Uncertain{if let Native::Approved{acknowledged,..}=native{*acknowledged=true;}}
    ConfirmResult::Queued{receipt}
}
fn end(native:&mut Native,store:&Store,now:u64,reason:Option<Denial>)->model::Result<ConfirmResult>{
    if let Native::Approved{approved,authority,..}=native{
        // The store serializes this with publication. A committed operation is
        // reported truthfully; cancellation uses the existing explicit controls.
        if let Some(receipt)=store.withdraw_admission(authority,approved,now)?{return Ok(published(native,receipt));}
    }
    *native=Native::Ended;
    Ok(match reason{Some(reason)=>ConfirmResult::Blocked{reason,retryable:false},None=>ConfirmResult::Declined})
}
impl Pending{
    pub(in crate::pet_runtime)fn confirm<G>(&self,store:&Store,authority_path:&Path,approve:bool,
        observe:impl Fn(&Record,u64)->model::Result<std::result::Result<(Evidence,G),Denial>>,
        clock:impl Fn()->u64)->model::Result<ConfirmResult>{
        self.confirm_with_data(store,authority_path,approve,None,observe,clock)
    }
    pub(in crate::pet_runtime)fn confirm_with_data<G>(&self,store:&Store,authority_path:&Path,approve:bool,memory_consent:Option<crate::pet_memory::context_model::GoalConsent>,
        observe:impl Fn(&Record,u64)->model::Result<std::result::Result<(Evidence,G),Denial>>,
        clock:impl Fn()->u64)->model::Result<ConfirmResult>{
        let mut native=self.native.try_lock().map_err(|e|match e{std::sync::TryLockError::WouldBlock=>Error::Busy,_=>Error::StorageUnavailable})?;
        if matches!(*native,Native::Ended){return Err(Error::Conflict);}
        if let Native::Approved{approved,authority,..}=&*native{
            match store.admission_ack(authority,approved){Ok(Some(receipt))=>return Ok(published(&mut native,receipt)),Ok(None)=>{},Err(error)=>return storage_error(error)}
        }
        if !approve{return end(&mut native,store,clock(),None);}
        if let Native::Review(review)=&*native{
            if let Err(reason)=review.data_consent_current(memory_consent,clock()){
                return if reason==Denial::Expired{end(&mut native,store,clock(),Some(reason))}else{Ok(ConfirmResult::Blocked{reason,retryable:false})};
            }
        }

        let goal=match store.read(&self.proposal.goal_id,&self.proposal.owner.pet_id){Ok(g)=>g,Err(e)=>return storage_error(e)};
        let (observation,_context_guard)=match observe(&goal,clock())?{
            Ok(pair)=>pair,Err(reason)if temporary(reason)=>return Ok(ConfirmResult::Blocked{reason,retryable:true}),
            Err(reason)=>return end(&mut native,store,clock(),Some(reason)),
        };
        if let Native::Review(review)=&*native{
            if let Err(reason)=review.confirmation_current(&goal,&observation,clock()){
                return if temporary(reason){Ok(ConfirmResult::Blocked{reason,retryable:true})}else{end(&mut native,store,clock(),Some(reason))};
            }
            // Capture the actual authority once. Later retry cannot adopt a new
            // key, directory or profile after a replaced/lost authority source.
            let authority=if review.has_native_authority(){None}else{Some(match Authority::ensure(authority_path){Ok(a)=>a,Err(reason)=>return Ok(authority_blocked(reason))})};
            let Native::Review(review)=std::mem::replace(&mut*native,Native::Ended)else{return Err(Error::Conflict);};
            let mut approved=match review.approve_with_data(&goal,observation,memory_consent,clock()){Ok(a)=>a,Err(reason)=>return Ok(ConfirmResult::Blocked{reason,retryable:false})};
            let authority=approved.take_native_authority().or(authority).ok_or(Error::Conflict)?;
            *native=Native::Approved{approved,authority,acknowledged:false};
        }else if let Native::Approved{approved,..}=&*native{
            if let Err(reason)=approved.confirmation_current(&goal,&observation,clock()){
                return if temporary(reason){Ok(ConfirmResult::Blocked{reason,retryable:true})}else{end(&mut native,store,clock(),Some(reason))};
            }
        }
        let Native::Approved{approved,authority,..}=&*native else{return Err(Error::Conflict);};
        let certified=match authority.certify(approved,&goal,clock()){Ok(c)=>c,Err(reason)=>return Ok(authority_blocked(reason))};
        match store.enqueue_certified(&certified,clock()){
            Ok(receipt)=>Ok(published(&mut native,receipt)),Err(Error::Conflict)=>end(&mut native,store,clock(),Some(Denial::RevisionChanged)),Err(error)=>storage_error(error),
        }
    }
}
#[cfg(test)]#[path="admission_confirmation_tests.rs"]mod tests;
