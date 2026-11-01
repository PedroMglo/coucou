//! Resource transitions require the original zero-worker native owner proof.
//! The signed ledger is audit data, never a replacement for that proof.
use super::*;
use crate::pet_runtime::{leases::Lease,resource_wait::{Entry,Ledger},NativeYieldProof};
impl Authority{
    pub(in crate::pet_runtime)fn resource_held_current(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,generation:u64,now:u64)->Result<()>{
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if body.phase!=GrantPhase::Claimed||body.lease.as_ref()!=Some(lease)||body.resource_usage.as_ref().map_or(1,Ledger::generation)!=generation{return Err(Failure::Revoked);}
        if now<lease.claimed_at||now>=lease.deadline_at{return Err(Failure::Expired);}self.fence()
    }
    pub(in crate::pet_runtime)fn resource_wait_current(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,entry:&Entry,now:u64)->Result<()>{
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if body.phase!=GrantPhase::Waiting||body.lease.as_ref()!=Some(lease)||body.resource_usage.as_ref().and_then(|r|r.entries.last())!=Some(entry){return Err(Failure::Revoked);}
        if now<entry.yielded_at||now>=entry.expires_at||now>=lease.deadline_at{return Err(Failure::Expired);}self.fence()
    }
    pub(in crate::pet_runtime)fn yield_resource(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,native:&NativeYieldProof,now:u64)->Result<()>{
        if !native.authorizes(spec,lease){return Err(Failure::Invalid);}let entry=native.entry();
        if spec.schema_version!=3||!entry.valid(lease)||entry.resumed_at.is_some()||now<entry.yielded_at||now>=entry.expires_at{return Err(Failure::Expired);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
        if body.lease.as_ref()!=Some(lease){return Err(Failure::Invalid);}
        if body.phase==GrantPhase::Waiting&&body.resource_usage.as_ref().and_then(|r|r.entries.last())==Some(entry){return self.stabilize(&body);}
        if body.phase!=GrantPhase::Claimed||now<body.updated_at{return Err(Failure::Revoked);}
        if body.budget_usage.as_ref().is_some_and(|u|u.attempts.iter().any(|a|a.phase!=crate::pet_runtime::effect_budget::Phase::Settled)){return Err(Failure::RecoveryRequired);}
        let usage=body.resource_usage.get_or_insert_with(||Ledger{schema_version:1,entries:vec![]});
        if usage.waiting()||usage.generation()!=entry.generation||usage.entries.len()>=8||usage.entries.iter().any(|e|e.id==entry.id||e.recipe_id==entry.recipe_id){return Err(Failure::Invalid);}
        usage.entries.push(entry.clone());body.phase=GrantPhase::Waiting;body.schema_version=if body.runtime_owner.is_some(){6}else{5};body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;
        let current=self.load(spec,proof)?;if current.phase!=GrantPhase::Waiting||current.resource_usage.as_ref().and_then(|r|r.entries.last())!=Some(entry){return Err(Failure::Changed);}Ok(())
    }
    pub(in crate::pet_runtime)fn resume_resource(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,native:&NativeYieldProof,now:u64)->Result<()>{
        if !native.authorizes_resume(spec,lease){return Err(Failure::Invalid);}let entry=native.entry();
        if now<entry.yielded_at||now>=entry.expires_at||now>=lease.deadline_at{return Err(Failure::Expired);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
        if body.lease.as_ref()!=Some(lease)||now<body.updated_at{return Err(Failure::Invalid);}
        let last=body.resource_usage.as_ref().and_then(|r|r.entries.last()).ok_or(Failure::Missing)?;
        if !last.same_wait(entry){return Err(Failure::Changed);}
        if body.phase==GrantPhase::Claimed&&last.resumed_at.is_some(){return self.stabilize(&body);}
        if body.phase!=GrantPhase::Waiting||last.resumed_at.is_some(){return Err(Failure::Revoked);}
        if self.active_unlocked()?.iter().any(|l|l.provider==lease.provider){return Err(Failure::Busy);}
        body.resource_usage.as_mut().unwrap().entries.last_mut().unwrap().resumed_at=Some(now);body.phase=GrantPhase::Claimed;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;
        let current=self.load(spec,proof)?;if current.phase!=GrantPhase::Claimed||current.resource_usage.as_ref().and_then(|r|r.entries.last()).is_none_or(|e|!e.same_wait(entry)||e.resumed_at.is_none()){return Err(Failure::Changed);}Ok(())
    }
}
