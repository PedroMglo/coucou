//! Receipt acknowledgment and withdrawal retain the original native approval.
//! These operations never issue a new consent or reconstruct a controller.
use super::*;
impl Authority{
    pub(in crate::pet_runtime)fn acknowledge_admission(&self,approved:&Approved,goal:&Record)->Result<()>{
        let spec=approved.spec();let row=goal.admissions.iter().find(|r|r.spec.decision_id==spec.decision_id).ok_or(Failure::Missing)?;
        if model::fingerprint(&row.spec).map_err(|_|Failure::Invalid)?!=model::fingerprint(spec).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}
        let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
        self.stabilize(&body)
    }
    pub(in crate::pet_runtime)fn withdraw_unqueued_admission(&self,approved:&Approved,now:u64)->Result<()>{
        let spec=approved.spec();let proof=self.proof(spec)?;let _lock=self.lock()?;
        let mut body=match self.load(spec,&proof){Ok(b)=>b,Err(Failure::Missing)=>return if spec.continuation.is_some(){self.retire_admission_staging(spec)}else{self.fence()},Err(e)=>return Err(e)};
        if body.lease.is_some()||!matches!(body.phase,GrantPhase::Issued|GrantPhase::Revoked){return Err(Failure::RecoveryRequired);}
        if body.phase==GrantPhase::Revoked{self.stabilize(&body)?;return if spec.continuation.is_some(){self.retire_admission_staging(spec)}else{Ok(())};}
        if now<body.updated_at||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        body.phase=GrantPhase::Revoked;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;if spec.continuation.is_some(){self.retire_admission_staging(spec)}else{Ok(())}
    }
}
