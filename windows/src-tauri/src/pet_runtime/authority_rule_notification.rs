//! The same authenticated parent reservation owns its one notification attempt.
use super::*;
use crate::pet_runtime::{rules::{RuleGrant,NotificationAttempt,NotificationOutcome},results::{self,Lookup}};
impl Authority{
 fn notification_parent(&self,s:&AdmissionSpec,proof:&ConsentProof,now:u64)->Result<(RuleGrant,usize,String)>{
  if s.schema_version!=30||!s.finite_response(){return Err(Failure::Invalid);}
  self.rule_admission_current_unlocked(s,now)?;
  let child=self.load(s,proof)?;if child.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel){return Err(Failure::Revoked);}
  let reference=s.rule_occurrence.as_ref().ok_or(Failure::Invalid)?;
  let(g,_)=self.rule_record_data(&format!("{}.json",reference.rule_decision_id))?.ok_or(Failure::Missing)?;
  let index=g.reservations.iter().position(|r|r.occurrence_id==reference.occurrence_id).ok_or(Failure::Missing)?;
  let p=g.policy.notification.as_ref().ok_or(Failure::Invalid)?;let r=&g.reservations[index];
  if now<r.at||now>=p.until(r){return Err(Failure::Expired);}
  let Lookup::Available{receipt}=results::lookup_unlocked(self,s,proof,&format!("response-{}",s.decision_id))? else{return Err(Failure::NotDue);};
  if receipt.recorded_at>now||receipt.completed_at<r.reserved_at{return Err(Failure::Changed);}
  let hash=model::fingerprint(&receipt).map_err(|_|Failure::Invalid)?;Ok((g,index,hash))
 }
 pub(in crate::pet_runtime)fn begin_rule_notification(&self,s:&AdmissionSpec,proof:&ConsentProof,now:u64,current:impl Fn()->bool)->Result<NotificationAttempt>{
  if !current(){return Err(Failure::Changed);}let _lock=self.lock()?;
  let(mut g,index,receipt_hash)=self.notification_parent(s,proof,now)?;
  if g.reservations[index].notification.is_some(){return Err(Failure::Capacity);}
  let n=NotificationAttempt{schema_version:1,id:nonce().map_err(|_|Failure::Unavailable)?,receipt_hash,attempted_at:now,finished_at:None,outcome:None};
  g.reservations[index].notification=Some(n.clone());g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
  self.publish_rule(&g,false,||if current(){Ok(())}else{Err(Failure::Changed)})?;Ok(n)
 }
 pub(in crate::pet_runtime)fn rule_notification_current(&self,s:&AdmissionSpec,proof:&ConsentProof,n:&NotificationAttempt,now:u64)->Result<()>{
  let _lock=self.lock_shared()?;let(g,index,hash)=self.notification_parent(s,proof,now)?;
  if n.finished_at.is_some()||n.receipt_hash!=hash||g.reservations[index].notification.as_ref()!=Some(n){return Err(Failure::Changed);}Ok(())
 }
 /// Acknowledging a late/failed desktop reply only records an observation. It
 /// does not require active authority and can never create or repeat an effect.
 pub(in crate::pet_runtime)fn finish_rule_notification(&self,decision:&str,occurrence:&str,n:&NotificationAttempt,outcome:NotificationOutcome,now:u64)->Result<()>{
  if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock()?;let name=format!("{decision}.json");
  if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
  let(mut g,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if self.rule_has_pending_unlocked(decision)?||now<g.updated_at{return Err(Failure::RecoveryRequired);}
  let index=g.reservations.iter().position(|r|r.occurrence_id==occurrence).ok_or(Failure::Missing)?;
  let old=g.reservations[index].notification.as_ref().ok_or(Failure::Missing)?;
  if old!=n||old.finished_at.is_some(){return Err(Failure::Changed);}
  let mut finished=n.clone();finished.finished_at=Some(now);finished.outcome=Some(outcome);g.reservations[index].notification=Some(finished);g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
  self.publish_rule(&g,false,||Ok(()))
 }
}
