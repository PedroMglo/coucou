//! Original opaque worker closure, in the same signed grant as its effects.
use super::*;
use crate::pet_runtime::{native_drain::Seal,lease_controller::DrainProof};
impl Authority{
 fn retire_native_drain_preparations(&self,body:&Grant,seal:&Seal)->Result<()>{
  // Only positively unpublished/duplicate images of this exact native close.
  // A later effect/result image is not cleanup and stays pending for recovery.
  let mut files=vec![];for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let (staged,stamp)=self.grant_file(&name)?;
   if staged.native_drain.as_ref()!=Some(seal){continue;}
   if staged.goal_id!=body.goal_id||staged.decision_id!=body.decision_id||staged.spec_hash!=body.spec_hash||staged.lease!=body.lease||staged.revision!=seal.target_grant_revision||staged.updated_at!=seal.reference.observed_at||staged.closed_at!=Some(seal.reference.observed_at)||staged.stop_reason!=seal.reference.reason||staged.recovery.is_some()||model::fingerprint(&(&staged.budget_usage,&staged.planning_usage,&staged.resource_usage)).map_err(|_|Failure::Invalid)?!=seal.usage_hash{return Err(Failure::RecoveryRequired);}
   files.push((name,stamp));if files.len()>64{return Err(Failure::Capacity);}
  }
  for(name,stamp)in files{self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
 }
 pub(in crate::pet_runtime)fn acknowledge_native_drain(&self,goal:&Record,seal:&Seal)->Result<()>{
  let _guard=self.lock()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if body.native_drain.as_ref()!=Some(seal)||!seal.reference.source_matches(goal)&&!seal.target_present(goal){return Err(Failure::Changed);}self.stabilize(&body)?;self.retire_native_drain_preparations(&body,seal)
 }

 pub(in crate::pet_runtime)fn native_drain(&self,goal:&Record)->Result<Option<Seal>>{let _guard=self.lock_shared()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;Ok(body.native_drain)}
 pub(in crate::pet_runtime)fn record_native_drain(&self,goal:&Record,drain:&DrainProof,cache:&mut Option<Seal>,now:u64)->Result<Seal>{
  let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let mut body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if let Some(seal)=&body.native_drain{if !seal.covers(drain.lease(),drain.request_id(),drain.confirmed())||!seal.reference.source_matches(goal)&&!seal.target_present(goal){return Err(Failure::Changed);}self.stabilize(&body)?;self.retire_native_drain_preparations(&body,seal)?;return Ok(seal.clone());}
  if let Some(saved)=cache.as_ref(){if !saved.reference.source_matches(goal)||saved.source_grant_revision!=body.revision||saved.source_grant_hash!=model::fingerprint(&body).map_err(|_|Failure::Invalid)?{
   if body.native_drain.is_some()||body.lease.as_ref()!=Some(drain.lease())||!matches!(body.phase,GrantPhase::Claimed|GrantPhase::Draining)||!drain.original_goal(goal){return Err(Failure::Changed);}
   self.retire_native_drain_preparations(&body,saved)?;
   let mut fresh=Seal::prepare(goal,&body,drain,now).map_err(|_|Failure::Changed)?;fresh.superseded_preparations=saved.superseded_preparations.clone();fresh.superseded_preparations.push(crate::pet_runtime::native_drain::Superseded{reference:saved.reference.clone(),source_grant_hash:saved.source_grant_hash.clone()});*cache=Some(fresh);
  }}
  let prepared=if let Some(saved)=cache{saved.clone()}else{let saved=Seal::prepare(goal,&body,drain,now).map_err(|_|Failure::Changed)?;*cache=Some(saved.clone());saved};
  if !prepared.covers(drain.lease(),drain.request_id(),drain.confirmed())||!prepared.reference.source_matches(goal)||prepared.source_grant_revision!=body.revision||prepared.source_grant_hash!=model::fingerprint(&body).map_err(|_|Failure::Invalid)?||body.lease.as_ref()!=Some(drain.lease())||body.stop_reason!=row.stop_reason||!matches!(body.phase,GrantPhase::Claimed|GrantPhase::Draining){return Err(Failure::Changed);}
  body.phase=if body.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel){GrantPhase::Revoked}else{GrantPhase::Closed};body.closed_at=Some(prepared.reference.observed_at);body.updated_at=prepared.reference.observed_at;body.revision=prepared.target_grant_revision.clone();body.native_drain=Some(prepared.clone());self.publish(body.clone(),false)?;self.retire_native_drain_preparations(&body,&prepared)?;Ok(prepared)
 }
}
