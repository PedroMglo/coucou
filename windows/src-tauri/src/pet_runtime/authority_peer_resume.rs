//! Same authority writer and original ledgers. No owner can be obtained from
//! this serialized seal; publishing requires the retained native lock/approval.
use super::*;
use crate::pet_runtime::{lease_controller::peer_resume_controller::Approved,owner_resumption::Seal};
impl Authority{
 fn peer_owner_preparation_source(staged:&Grant,seal:&Seal)->Result<String>{
  if staged.owner_resumptions.last().is_none_or(|s|model::fingerprint(s).ok()!=model::fingerprint(seal).ok())||staged.runtime_owner.as_ref()!=Some(&seal.owner)||staged.phase!=GrantPhase::Claimed||staged.stop_reason.is_some()||staged.closed_at.is_some()||staged.recovery.is_some()||staged.resource_usage.is_some()||Seal::usage_hash(staged).ok().as_ref()!=Some(&seal.usage_hash){return Err(Failure::RecoveryRequired);}
  let mut source=staged.clone();source.owner_resumptions.pop();source.runtime_owner=Some(seal.previous_owner.clone());source.recovery=Some(seal.closure.clone());source.resource_usage=seal.previous_resource_usage.clone();source.closed_at=Some(seal.previous_closed_at);source.updated_at=seal.previous_updated_at;source.revision=seal.reference.source_grant_revision.clone();source.phase=GrantPhase::Closed;source.schema_version=source.message_schema();
  let hash=model::fingerprint(&source).map_err(|_|Failure::Invalid)?;if hash!=seal.reference.source_grant_hash{return Err(Failure::RecoveryRequired);}Ok(hash)
 }
 /// Explicit recovery action, never passive status: retire only unpublished
 /// owner intents whose exact closed source is still canonical. One retained
 /// lock proves every proposed process absent; TTL cannot substitute for death.
 pub(in crate::pet_runtime)fn retire_lost_peer_owner_preparations(&self,goal:&Record)->Result<usize>{
  let _guard=self.lock()?;let Some(row)=goal.admissions.last()else{return Ok(0);};if !row.spec.peer_iteration(){return Ok(0);}
  let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;self.recovery_profile(&body)?;let hash=model::fingerprint(&body).map_err(|_|Failure::Invalid)?;
  let mut candidates=vec![];
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}
   if self.rule_record_data(&name)?.is_some(){continue;}let(staged,stamp)=self.grant_file(&name)?;if staged.goal_id!=goal.id||staged.decision_id!=row.spec.decision_id{continue;}
   let Some(seal)=staged.owner_resumptions.last()else{return Err(Failure::RecoveryRequired);};
   let source_hash=Self::peer_owner_preparation_source(&staged,seal)?;
   let original=seal.reference.source_matches(goal)&&source_hash==hash;
   // A signed Stop may reduce the still-closed original admission after this
   // preparation. Cleanup removes only the unused image, never that reduction.
   let stopped=matches!(goal.state,State::Paused|State::Cancelled|State::Stopping)&&matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)&&body.stop_reason.is_some()
    &&body.runtime_owner.as_ref()==Some(&seal.previous_owner)&&body.recovery.as_ref().is_some_and(|c|c.reference==seal.closure.reference)&&Self::recovery_event_present(goal,&seal.closure)?
    &&model::fingerprint(&(&body.budget_usage,&body.planning_usage,&body.messages)).ok()==model::fingerprint(&(&staged.budget_usage,&staged.planning_usage,&staged.messages)).ok();
   if !(original||stopped)||staged.spec_hash!=body.spec_hash||staged.lease!=body.lease{return Err(Failure::RecoveryRequired);}
   candidates.push((name,stamp,staged));if candidates.len()>64{return Err(Failure::Capacity);}
  }
  if candidates.is_empty(){return Ok(0);}
  let absence=self.prepared_owner_absence(&candidates[0].2)?;
  for(_,_,staged)in &candidates{absence.same_lock_absent(self,staged.runtime_owner.as_ref().ok_or(Failure::Missing)?).map_err(|_|Failure::Changed)?;}
  for(name,stamp,staged)in &candidates{absence.same_lock_absent(self,staged.runtime_owner.as_ref().ok_or(Failure::Missing)?).map_err(|_|Failure::Changed)?;self.fence()?;self.dir.remove_exact(name,stamp).map_err(|_|Failure::Uncertain)?;}
  self.dir.durable().map_err(|_|Failure::Uncertain)?;absence.current(self).map_err(|_|Failure::Changed)?;Ok(candidates.len())
 }
 fn retire_peer_owner_preparations(&self,body:&Grant,seal:&Seal)->Result<()>{
  let mut files=vec![];
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}
   if self.rule_record_data(&name)?.is_some(){continue;}let(staged,stamp)=self.grant_file(&name)?;let Some(saved)=staged.owner_resumptions.last().filter(|s|s.reference.id==seal.reference.id)else{continue;};
   if model::fingerprint(saved).ok()!=model::fingerprint(seal).ok()||staged.goal_id!=body.goal_id||staged.decision_id!=body.decision_id||staged.spec_hash!=body.spec_hash||staged.lease!=body.lease
    ||staged.runtime_owner.as_ref()!=Some(&seal.owner)||staged.phase!=GrantPhase::Claimed||staged.stop_reason.is_some()||staged.closed_at.is_some()||staged.recovery.is_some()||staged.resource_usage.is_some()
    ||Seal::usage_hash(&staged).ok().as_ref()!=Some(&seal.usage_hash){return Err(Failure::RecoveryRequired);}
   // Reverse only this exact native preparation to its hash-pinned old grant.
   Self::peer_owner_preparation_source(&staged,seal)?;
   let committed=body.owner_resumptions.iter().any(|s|model::fingerprint(s).ok()==model::fingerprint(seal).ok());
   let uncommitted=body.runtime_owner.as_ref()==Some(&seal.previous_owner)&&matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)&&body.recovery.as_ref().is_some_and(|c|c.reference==seal.closure.reference);
   if !committed&&!uncommitted{return Err(Failure::RecoveryRequired);}files.push((name,stamp));if files.len()>64{return Err(Failure::Capacity);}
  }
  for(name,stamp)in files{self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
 }
 pub(in crate::pet_runtime)fn withdraw_peer_owner(&self,approved:&Approved)->Result<bool>{
  let owner=approved.native_owner().map_err(|_|Failure::Changed)?;owner.current().map_err(|_|Failure::Changed)?;
  let _guard=self.lock()?;let row=approved.goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  let seal=approved.seal.as_ref().ok_or(Failure::Missing)?;
  if body.owner_resumptions.iter().any(|s|s.reference.id==seal.reference.id){return Ok(false);}
  if body.runtime_owner.as_ref()!=Some(&seal.previous_owner)||!matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)||body.recovery.as_ref().is_none_or(|c|c.reference!=seal.closure.reference)||!owner.resumes(&seal.previous_owner){return Err(Failure::Changed);}
  self.recovery_profile(&body)?;self.retire_peer_owner_preparations(&body,seal)?;Ok(true)
 }
 /// Recover only a canonical, previously approved publication. This returns
 /// data, never a native owner or an effect permit, including after expiry.
 pub(in crate::pet_runtime)fn peer_owner_ack_target(&self,goal:&Record)->Result<Option<(Seal,Record)>>{
  let _guard=self.lock_shared()?;let Some(row)=goal.admissions.last()else{return Ok(None);};
  let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  let Some(seal)=body.owner_resumptions.last()else{return Ok(None);};
  self.recovery_profile(&body)?;let r=&seal.reference;
  if r.present(goal){return if goal.sequence==r.target_sequence&&goal.revision==r.target_revision&&model::fingerprint(goal).ok().as_ref()==Some(&seal.target_goal_hash){Ok(Some((seal.clone(),goal.clone())))}else{Ok(None)};}
  if !r.source_matches(goal){return Err(Failure::Changed);}
  let mut candidate=goal.clone();candidate.owner_resumed(r).map_err(|_|Failure::Changed)?;
  if model::fingerprint(&candidate).ok().as_ref()!=Some(&seal.target_goal_hash){return Err(Failure::Changed);}
  self.fence()?;Ok(Some((seal.clone(),candidate)))
 }
 /// A Stop reviewed against the closed source may race its exact ACK. It may
 /// cross that one signed metadata transition, never unrelated goal changes.
 pub(in crate::pet_runtime)fn peer_owner_stop_target(&self,goal:&Record,revision:&str)->Result<Option<(Seal,Record)>>{
  if let Some((seal,target))=self.peer_owner_ack_target(goal)?{
   if !seal.reference.present(goal){return if seal.reference.source_revision==revision{Ok(Some((seal,target)))}else{Err(Failure::Changed)};}
  }
  let _guard=self.lock_shared()?;let Some(row)=goal.admissions.last()else{return Ok(None);};let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  let Some(seal)=body.owner_resumptions.last()else{return Ok(None);};
  if revision!=seal.reference.source_revision{return Ok(None);}
  self.recovery_profile(&body)?;
  if !seal.reference.present(goal)||goal.sequence!=seal.reference.target_sequence||goal.revision!=seal.reference.target_revision||model::fingerprint(goal).ok().as_ref()!=Some(&seal.target_goal_hash){return Err(Failure::Changed);}
  Ok(Some((seal.clone(),goal.clone())))
 }
 pub(in crate::pet_runtime)fn root_path(&self)->std::path::PathBuf{self.dir.path.clone()}
 pub(in crate::pet_runtime)fn resume_peer_owner(&self,approved:&mut Approved,now:u64)->Result<()>{
  approved.current_native(now).map_err(|_|Failure::Revoked)?;
  let seal=approved.seal.as_ref().ok_or(Failure::Invalid)?.clone();
  let row=approved.goal.admissions.last().ok_or(Failure::Missing)?;
  let owner=approved.native_owner().map_err(|_|Failure::Invalid)?;
  if owner.record()!=&seal.owner||!owner.resumes(&seal.previous_owner){return Err(Failure::Changed);}
  let _guard=self.lock()?;let mut body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  // DATA/evidence captured before a wait cannot outlive the original escrow.
  // Recheck under the same writer immediately before any owner publication.
  self.funding_current_unlocked(&row.spec,now)?;
  if body.owner_resumptions.iter().any(|r|r.reference.id==seal.reference.id){
   let found=body.owner_resumptions.last().ok_or(Failure::Missing)?;
   if model::fingerprint(found).ok()!=model::fingerprint(&seal).ok()||body.runtime_owner.as_ref()!=Some(owner.record())||body.stop_reason.is_some()||body.phase!=GrantPhase::Claimed{return Err(Failure::Changed);}
   approved.write_started=true;self.stabilize(&body)?;return self.retire_peer_owner_preparations(&body,&seal);
  }
  approved.approval_current(now).map_err(|_|Failure::Revoked)?;
  if body.phase!=GrantPhase::Closed||body.stop_reason.is_some()||body.recovery.as_ref().is_none_or(|c|c.reference!=seal.closure.reference)
   ||body.runtime_owner.as_ref()!=Some(&seal.previous_owner)||body.revision!=seal.reference.source_grant_revision
   ||model::fingerprint(&body).ok().as_ref()!=Some(&seal.reference.source_grant_hash)||Seal::usage_hash(&body).ok().as_ref()!=Some(&seal.usage_hash)
   ||body.owner_resumptions.len()>=crate::pet_runtime::owner_resumption::MAX_RESUMPTIONS{return Err(Failure::Changed);}
  self.recovery_profile(&body)?;
  if self.active_unlocked()?.iter().any(|l|l.provider==row.spec.owner.provider){return Err(Failure::Busy);}
  body.owner_resumptions.push(seal.clone());body.runtime_owner=Some(owner.record().clone());body.recovery=None;body.resource_usage=None;
  body.closed_at=None;body.phase=GrantPhase::Claimed;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;
  approved.write_started=true;self.publish_guarded(body.clone(),false,||{
   owner.current().map_err(|_|Failure::Changed)?;if !owner.resumes(&approved.seal.as_ref().ok_or(Failure::Invalid)?.previous_owner){return Err(Failure::Changed);}Ok(())
  })?;self.retire_peer_owner_preparations(&body,&seal)
 }
 pub(in crate::pet_runtime)fn acknowledge_peer_owner_resumption(&self,goal:&Record,seal:&Seal)->Result<()>{
  let _guard=self.lock()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==seal.reference.decision_id).ok_or(Failure::Missing)?;
  let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;self.recovery_profile(&body)?;
  if !seal.reference.present(goal)||body.owner_resumptions.iter().all(|r|model::fingerprint(r).ok()!=model::fingerprint(seal).ok()){return Err(Failure::Changed);}self.stabilize(&body)?;self.retire_peer_owner_preparations(&body,seal)
 }
}
