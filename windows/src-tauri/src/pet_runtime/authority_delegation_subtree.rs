//! Bounded authority reduction on the common ledger. Persist each parent
//! intent before signalling its original children; never release credit/holds.
use super::*;
use crate::pet_runtime::{delegation::{Reservation,State as Branch},authority_store::delegation_branch::StopTarget,leases::StopReason};
use std::collections::BTreeSet;
impl Authority{
 pub(in crate::pet_runtime::authority_store)fn cancel_descendants_unlocked(&self,root:&Grant,edge:&Reservation,child:&Grant,now:u64)->Result<Vec<StopTarget>>{
  let Branch::CancelRequested{binding,cancellation}=&edge.state else{return Err(Failure::Revoked);};
  if !root.delegation_reservations.contains(edge)||binding.goal_id!=child.goal_id||binding.decision_id!=child.decision_id||binding.admission_hash!=child.spec_hash{return Err(Failure::Changed);}
  if child.delegation_pool.is_none(){return Ok(vec![]);}
  self.stop_descendants_unlocked(child,StopReason::Cancel,&cancellation.review_id,now,Some(root))
 }
 /// The signed Stop of this exact admission is the authority to reduce all
 /// its outgoing branches. Pause retains reservations; Cancel withdraws only
 /// work never bound to a child. No resume or fresh grant is manufactured.
 pub(in crate::pet_runtime)fn stopped_pool_descendants(&self,spec:&AdmissionSpec,proof:&ConsentProof,now:u64)->Result<Vec<StopTarget>>{
  let _guard=self.lock()?;let body=self.load(spec,proof)?;
  if body.delegation_pool.is_none()||body.delegation_reservations.is_empty(){return Ok(vec![]);}
  if !matches!(body.phase,GrantPhase::Draining|GrantPhase::Closed|GrantPhase::Revoked){return Err(Failure::Revoked);}
  let reason=body.stop_reason.filter(|r|matches!(r,StopReason::Pause|StopReason::Cancel)).ok_or(Failure::Revoked)?;
  self.stop_descendants_unlocked(&body,reason,&body.revision,now,None)
 }
 fn stop_descendants_unlocked(&self,first:&Grant,reason:StopReason,review:&str,now:u64,ancestor:Option<&Grant>)->Result<Vec<StopTarget>>{
  let mut stack=vec![(first.clone(),0usize,reason)];let mut goals=BTreeSet::new();let mut decisions=BTreeSet::new();let mut targets=vec![];
  if let Some(root)=ancestor{goals.insert(root.goal_id.clone());decisions.insert(root.decision_id.clone());}
  while let Some((mut parent,depth,reason))=stack.pop(){
   // A persisted authority reduction can finish after concurrent signed
   // progress. This clock only orders Stop cleanup; it renews no allowance.
   let at=now.max(parent.updated_at);
   if depth>=8||goals.len()>=128||!goals.insert(parent.goal_id.clone())||!decisions.insert(parent.decision_id.clone())||at>8_640_000_000_000_000{return Err(Failure::Changed);}
   if parent.delegation_pool.is_none()||(depth>0||ancestor.is_some())&&!(matches!(parent.schema_version,28|32|35)&&parent.funded_by.as_ref().is_some_and(|f|f.schema_version==5)||parent.schema_version==47&&parent.hybrid_pool()&&parent.funded_by.as_ref().is_some_and(|f|f.schema_version==9)){return Err(Failure::Changed);}
   for index in 0..parent.delegation_reservations.len(){
    let mut branch=parent.delegation_reservations[index].clone();
    match &branch.state{
     Branch::Withdrawn{..}|Branch::Settled{..}=>continue,
     Branch::Pending=>{
      if reason==StopReason::Cancel{
       branch.state=Branch::Withdrawn{at,review_id:review.into()};parent.delegation_reservations[index]=branch;parent.updated_at=at;parent.revision=nonce().map_err(|_|Failure::Unavailable)?;parent.schema_version=if parent.peer_planning{parent.message_schema()}else{parent.native_schema()};self.publish(parent.clone(),false)?;
      }
      continue;
     },
     Branch::Bound{binding} if reason==StopReason::Cancel=>{
      branch.schema_version=branch.schema_version.max(3);branch.state=Branch::CancelRequested{binding:binding.clone(),cancellation:crate::pet_runtime::delegation::Cancellation{at,review_id:review.into()}};parent.delegation_reservations[index]=branch.clone();parent.updated_at=at;parent.revision=nonce().map_err(|_|Failure::Unavailable)?;parent.schema_version=if parent.peer_planning{parent.message_schema()}else{parent.native_schema()};self.publish(parent.clone(),false)?;
     },
     Branch::Bound{..}|Branch::CancelRequested{..}=>{},
    }
    // Each parent marker is durable before signalling/mutating its child.
    // Already-cancelled branches dominate a later ancestral Pause.
    let requested=if matches!(branch.state,Branch::CancelRequested{..}){StopReason::Cancel}else{reason};
    let mut next=self.branch_child_unlocked(&parent,&branch)?;if goals.contains(&next.goal_id)||decisions.contains(&next.decision_id){return Err(Failure::Changed);}let child_at=at.max(next.updated_at);
    let binding=branch.binding().ok_or(Failure::Changed)?.clone();let lease=next.lease.clone();if let Some(l)=&lease{crate::pet_runtime::lease_controller::signal_stop(l);}
    let actual=if next.stop_reason==Some(StopReason::Cancel){StopReason::Cancel}else{requested};
    if !matches!(next.phase,GrantPhase::Closed|GrantPhase::Revoked)&&next.native_completion.is_none()&&!(next.phase==GrantPhase::Draining&&next.stop_reason==Some(actual)){
     if next.phase==GrantPhase::Issued{next.phase=GrantPhase::Revoked;}else{next.phase=GrantPhase::Draining;next.stop_reason=Some(actual);}
     next.updated_at=child_at;next.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(next.clone(),false)?;
    }else{self.stabilize(&next)?;}
    targets.push(StopTarget{binding,lease,reason:actual,descendants:vec![]});if targets.len()>128{return Err(Failure::Capacity);}
    if next.delegation_pool.is_some(){stack.push((next,depth+1,actual));}
   }
  }
  self.fence()?;Ok(targets)
 }
}
/// Project only the original canonical binding; never stop a later admission.
/// All sibling signals/intents already exist, so one unreadable queue cannot
/// starve the remaining reductions. Original owners still owe drain proofs.
pub(in crate::pet_runtime)fn project_stop_queues(store:&impl crate::pet_runtime::Repository,path:&Path,targets:&[StopTarget],now:u64)->model::Result<()>{
 let mut pending=None;for target in targets{
  if let Err(e)=project_one(store,path,target,now){if pending.is_none(){pending=Some(e);}}
  if let Err(e)=project_stop_queues(store,path,&target.descendants,now){if pending.is_none(){pending=Some(e);}}
 }
 pending.map_or(Ok(()),Err)
}
fn project_one(store:&impl crate::pet_runtime::Repository,path:&Path,target:&StopTarget,now:u64)->model::Result<()>{
 let b=&target.binding;if let Some(l)=&target.lease{crate::pet_runtime::lease_controller::signal_stop(l);}
 let g=store.read_control(&b.goal_id,&b.pet_id)?;let now=now.max(g.updated_at);
 let Some(row)=g.admissions.last()else{return Ok(());};if !b.matches(&row.spec){return Err(model::Error::Conflict);}
 if matches!(g.state,State::Completed|State::Cancelled){return Ok(());}
 let reason=if row.stop_reason==Some(StopReason::Cancel){StopReason::Cancel}else{target.reason};
 if (g.state==State::Stopping&&row.stop_reason==Some(reason))||(g.state==State::Paused&&reason==StopReason::Pause){return Ok(());}
 let action=if reason==StopReason::Cancel{model::Action::Cancel}else{model::Action::Pause};
 let request=format!("descendant-{}-{}",if reason==StopReason::Cancel{"cancel"}else{"pause"},b.decision_id);
 let authority=Authority::existing(path).map_err(Failure::control_error)?;let proof=row.proof.as_ref().ok_or(model::Error::Conflict)?;
 authority.request_stop(&row.spec,proof,reason,now).map_err(Failure::control_error)?;
 let receipt=store.mutate(&g.id,&b.pet_id,&g.revision,request,action,now)?;if receipt.status==model::CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}Ok(())
}
