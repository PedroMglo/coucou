//! Authenticated runtime-owner capture and read-only recovery observations.
use super::*;
use crate::pet_runtime::{native_owner::{self,OwnerLease},runtime_owner::{FileId,Profile,Observation,State,Reason},leases::Lease};
/// Constructible only after authenticating the original grant. Neither a JSON
/// owner observation nor a copied lease can construct this native input.
pub(in crate::pet_runtime)struct LostOwnerSource{lease:Lease,owner:crate::pet_runtime::runtime_owner::Record,grant_hash:String,grant_revision:String}
impl LostOwnerSource{
    pub(in crate::pet_runtime)fn lease(&self)->&Lease{&self.lease}
    pub(in crate::pet_runtime)fn owner(&self)->&crate::pet_runtime::runtime_owner::Record{&self.owner}
    pub(in crate::pet_runtime)fn matches(&self,hash:&str,revision:&str)->bool{self.grant_hash==hash&&self.grant_revision==revision}
}
impl Authority{
    pub(in crate::pet_runtime)fn retained_absence(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<native_owner::AbsentLease>{
        self.retained_absence_for(spec,proof,false)
    }
    pub(in crate::pet_runtime)fn retained_recovery_absence(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<native_owner::AbsentLease>{self.retained_absence_for(spec,proof,true)}
    fn retained_absence_for(&self,spec:&AdmissionSpec,proof:&ConsentProof,terminal:bool)->Result<native_owner::AbsentLease>{
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if !matches!(body.phase,GrantPhase::Claimed|GrantPhase::Waiting|GrantPhase::Draining)&&!(terminal&&matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)){return Err(Failure::NotQueued);}
        let lease=body.lease.clone().ok_or(Failure::Missing)?;let owner=body.runtime_owner.clone().ok_or(Failure::RecoveryRequired)?;
        let source=LostOwnerSource{lease,owner,grant_hash:model::fingerprint(&body).map_err(|_|Failure::Invalid)?,grant_revision:body.revision};
        native_owner::AbsentLease::capture(self,source).map_err(|e|match e{model::Error::Busy=>Failure::Busy,model::Error::StorageUnavailable=>Failure::Unavailable,_=>Failure::Changed})
    }
    pub(in crate::pet_runtime)fn owner_context(&self)->Result<(std::path::PathBuf,Profile)>{
        self.fence()?;let metadata=self.dir.file.metadata().map_err(|_|Failure::Unavailable)?;let name=self.dir.path.file_name().and_then(|n|n.to_str()).ok_or(Failure::Unsafe)?;
        Ok((self.dir.path.with_file_name(format!("{name}.owners-v1")),Profile{directory:FileId{device:metadata.dev(),inode:metadata.ino()},key_id:self.key_id.clone()}))
    }
    pub(in crate::pet_runtime)fn native_owner_current(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,owner:&OwnerLease)->Result<()>{
        if !owner.matches(self,lease){return Err(Failure::Changed);}let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if body.lease.as_ref()!=Some(lease)||body.runtime_owner.as_ref()!=Some(owner.record()){return Err(Failure::Changed);}Ok(())
    }
    pub(in crate::pet_runtime)fn owner_observation(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<Observation>{
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        if matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked){return Ok(Observation::without_owner(State::Closed,None));}
        if body.lease.is_none(){return Ok(Observation::without_owner(State::NotClaimed,None));}
        let Some(record)=&body.runtime_owner else{return Ok(Observation::without_owner(State::LegacyUnknown,Some(Reason::LegacyOwnerMissing)));};
        Ok(native_owner::observe(self,record))
    }
    #[cfg(test)]pub(in crate::pet_runtime)fn fixture_legacy_ownerless(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<()>{
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;body.runtime_owner=None;
        body.schema_version=if body.resource_usage.is_some(){5}else if body.planning_usage.is_some(){4}else if body.budget_usage.is_some(){3}else{2};self.publish(body,false)
    }
}
