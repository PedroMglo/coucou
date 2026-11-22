//! Goal-before-authority ordering and original intent ACK; no second scheduler.
use super::*;
use crate::pet_runtime::{authority::{Authority,Failure},lease_controller::peer_resume_controller::Approved};
impl Store{
 pub(in crate::pet_runtime)fn retire_lost_peer_owner_preparations(&self,path:&Path,id:&str,pet:&str,revision:&str)->Result<usize>{
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let _guard=self.lock(true)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;self.require_current_goal(&goal)?;
  if goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}authority.retire_lost_peer_owner_preparations(&goal).map_err(Failure::control_error)
 }
 pub(in crate::pet_runtime)fn withdraw_peer_owner(&self,approved:&Approved)->Result<bool>{
  let authority=Authority::existing(&approved.source.authority.root_path()).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let _guard=self.lock(true)?;let(goal,_)=self.load_with_pending(&approved.goal.id,true)?.ok_or(Error::NotFound)?;self.require_current_goal(&goal)?;
  if goal.spec.owner!=approved.goal.spec.owner{return Err(Error::Conflict);}authority.withdraw_peer_owner(approved).map_err(Failure::control_error)
 }
 /// Explicit cleanup of the signed intent after loss of its native reviewer.
 /// A stored approval cannot activate an executor, renew time or spend budget.
 pub(in crate::pet_runtime)fn ack_peer_owner_resumption(&self,path:&Path,id:&str,pet:&str,revision:&str)->Result<Option<Receipt>>{
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let _guard=self.lock(true)?;let(goal,before)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;self.require_current_goal(&goal)?;
  if goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}
  let Some((seal,target))=authority.peer_owner_ack_target(&goal).map_err(Failure::control_error)?else{return Ok(None);};
  authority.acknowledge_peer_owner_resumption(&target,&seal).map_err(Failure::control_error)?;
  if seal.reference.present(&goal){return Ok(Some(self.stabilize(goal)));}
  Ok(Some(self.publish(target,Some(&before),true)?))
 }
 /// The common Stop entry point invokes this only for its exact pending
 /// handoff (or that ACK racing the review). Commit both metadata events in
 /// one goal write, after revocation; never publish a runnable interim owner.
 pub(in crate::pet_runtime)fn stop_peer_owner_handoff(&self,path:&Path,id:&str,pet:&str,revision:&str,request:&str,action:&Action,now:u64)->Result<Option<Receipt>>{
  if !matches!(action,Action::Pause|Action::Cancel){return Ok(None);}
  let guard=self.lock(true)?;let(goal,before)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;self.require_current_goal(&goal)?;
  if goal.spec.owner.pet_id!=pet{return Err(Error::Conflict);}
  if !goal.admissions.last().is_some_and(|r|r.spec.peer_iteration()){return Ok(None);}
  if goal.replay(request,action)?{return Ok(None);}
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let Some((seal,mut target))=authority.peer_owner_stop_target(&goal,revision).map_err(Failure::control_error)?else{return Ok(None);};
  // Validate the complete requested reduction before changing the grant.
  target.apply(request.to_owned(),action.clone(),now,nonce()?)?;
  let row=goal.admissions.last().ok_or(Error::Conflict)?;let proof=row.proof.as_ref().ok_or(Error::Conflict)?;
  let lease=row.lease.as_ref().ok_or(Error::Conflict)?;crate::pet_runtime::lease_controller::signal_stop(lease);
  let reason=if matches!(action,Action::Cancel){crate::pet_runtime::leases::StopReason::Cancel}else{crate::pet_runtime::leases::StopReason::Pause};
  if authority.recorded_stop_reason(&row.spec,proof).map_err(Failure::control_error)?==Some(crate::pet_runtime::leases::StopReason::Cancel)&&reason!=crate::pet_runtime::leases::StopReason::Cancel{return Err(Error::Conflict);}
  if authority.request_stop(&row.spec,proof,reason,now).map_err(Failure::control_error)?.as_ref().is_some_and(|l|l!=lease){return Err(Error::Conflict);}
  let targets=authority.stopped_pool_descendants(&row.spec,proof,now).map_err(Failure::control_error)?;
  authority.acknowledge_peer_owner_resumption(&target,&seal).map_err(Failure::control_error)?;
  let receipt=self.publish(target,Some(&before),true)?;drop(guard);
  if receipt.status!=CommitStatus::Uncertain{crate::pet_runtime::authority_store::project_stop_queues(self,path,&targets,now)?;}
  Ok(Some(receipt))
 }
 pub(in crate::pet_runtime)fn confirm_peer_owner_resumption(&self,approved:&mut Approved,now:u64)->Result<Receipt>{
  let authority=Authority::existing(&approved.source.authority.root_path()).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let _guard=self.lock(true)?;let(goal,before)=self.load_with_pending(&approved.goal.id,true)?.ok_or(Error::NotFound)?;
  let seal=approved.seal.as_ref().ok_or(Error::Conflict)?.clone();
  if seal.reference.present(&goal){authority.acknowledge_peer_owner_resumption(&goal,&seal).map_err(Failure::control_error)?;return Ok(self.stabilize(goal));}
  if !seal.reference.source_matches(&goal){return Err(Error::Conflict);}
  approved.current_native(now)?;authority.resume_peer_owner(approved,now).map_err(Failure::control_error)?;
  self.publish(approved.candidate.clone().ok_or(Error::Conflict)?,Some(&before),true)
 }
}
