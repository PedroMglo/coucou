//! Explicit ACK publishes only the already signed drain target, no IO permit.
use super::*;
use crate::pet_runtime::{model,authority::Authority,native_drain::Seal,lease_controller::DrainProof};
impl Store{
 fn publish_native_drain(&self,goal:Record,before:&Stamp,seal:&Seal)->Result<Receipt>{
  if seal.target_present(&goal){return Ok(self.stabilize(goal));}
  if !seal.reference.source_matches(&goal){return Err(Error::Conflict);}let mut target=goal;target.drained_native(&seal.reference)?;
  if model::fingerprint(&target)?!=seal.target_goal_hash{return Err(Error::Conflict);}self.publish(target,Some(before),true)
 }
 pub(super)fn drain_native_owned(&self,authority:&Authority,drain:&DrainProof,goal:Record,before:&Stamp,now:u64)->Result<Receipt>{
  drain.with_native_drain(|cache|{let seal=authority.record_native_drain(&goal,drain,cache,now).map_err(crate::pet_runtime::authority::Failure::control_error)?;self.publish_native_drain(goal,before,&seal)})
 }
 pub(in crate::pet_runtime)fn ack_native_drain(&self,path:&Path,id:&str,pet:&str,lease:&str,revision:&str)->Result<Receipt>{
  let _guard=self.lock(true)?;let(goal,before)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;
  if goal.spec.owner.pet_id!=pet{return Err(Error::Conflict);}let authority=Authority::existing(path).map_err(crate::pet_runtime::authority::Failure::control_error)?;self.bound_authority(&authority)?;
  let seal=authority.native_drain(&goal).map_err(crate::pet_runtime::authority::Failure::control_error)?.ok_or(Error::Conflict)?;
  if seal.reference.lease_id!=lease||(revision!=seal.reference.source_revision&&revision!=seal.reference.target_revision){return Err(Error::Conflict);}authority.acknowledge_native_drain(&goal,&seal).map_err(crate::pet_runtime::authority::Failure::control_error)?;self.publish_native_drain(goal,&before,&seal)
 }
}
