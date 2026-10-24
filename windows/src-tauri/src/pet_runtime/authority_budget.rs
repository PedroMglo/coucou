//! Same authenticated grant transaction/lock as consent and stop. No UI data
//! can turn an attempt record into an IO permit.
use super::*;
use crate::pet_runtime::{effect_budget::{Attempt,Ledger,Phase},leases::Lease};
impl Authority{
 pub(super)fn budget_body(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,now:u64,starting:bool)->Result<Grant>{
  let body=self.load(spec,proof)?;
  if !lease.valid(spec)||body.lease.as_ref()!=Some(lease){return Err(Failure::Invalid);}
  if starting&&(body.phase!=GrantPhase::Claimed||now>=lease.deadline_at){return Err(Failure::Revoked);}
  if now<body.updated_at||now<lease.claimed_at||now>8_640_000_000_000_000{return Err(Failure::Invalid);}Ok(body)
 }
 pub(in crate::pet_runtime)fn output_remaining(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,now:u64)->Result<u64>{
  let _lock=self.lock()?;let body=self.budget_body(spec,proof,lease,now,true)?;
  let used=body.budget_usage.as_ref().map(|u|u.attempts.iter().map(Attempt::charge).sum::<u64>()).unwrap_or(0);
  spec.budget.max_output_bytes.checked_sub(used).ok_or(Failure::Invalid)
 }
 pub(in crate::pet_runtime)fn reserve_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,now:u64)->Result<()>{
  if !attempt.valid()||attempt.phase!=Phase::Reserved||attempt.reserved_at>now||attempt.reserved_at<lease.claimed_at{return Err(Failure::Invalid);}
  // Finite unmetered caps cannot be approximated by call counts/output bytes.
  if spec.budget.max_tokens.is_some()||spec.budget.max_cost_micros.is_some(){return Err(Failure::Invalid);}
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,true)?;let ledger=body.budget_usage.get_or_insert_with(Ledger::empty);
  if let Some(old)=ledger.attempts.iter().find(|a|a.id==attempt.id||a.request_id==attempt.request_id){
   return if old.same_request(attempt)&&old.phase==Phase::Reserved{self.stabilize(&body)}else{Err(Failure::RecoveryRequired)};
  }
  ledger.attempts.push(attempt.clone());if !ledger.valid_for(spec){return Err(Failure::Capacity);}
  body.schema_version=3;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
 }
 pub(in crate::pet_runtime)fn start_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,now:u64)->Result<()>{
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,true)?;
  let old=body.budget_usage.as_mut().and_then(|l|l.attempts.iter_mut().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
  if !old.same_request(attempt)||old.phase!=Phase::Reserved||now<old.reserved_at{return Err(Failure::RecoveryRequired);}
  old.phase=Phase::Started;old.started_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
 }
 pub(in crate::pet_runtime)fn finish_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,accepted:Option<u64>,now:u64)->Result<()>{
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
  let old=body.budget_usage.as_mut().and_then(|l|l.attempts.iter_mut().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
  if !old.same_request(attempt)||accepted.is_some_and(|n|n>old.output_reserved){return Err(Failure::Invalid);}
  let phase=if accepted.is_some(){Phase::Settled}else{Phase::Uncertain};
  if old.phase==phase&&old.accepted_output==accepted{return self.stabilize(&body);}
  if old.phase!=Phase::Started||now<old.started_at.ok_or(Failure::Invalid)?{return Err(Failure::RecoveryRequired);}
  old.phase=phase;old.accepted_output=accepted;old.finished_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;
  #[cfg(test)]if matches!(self.fault,Fault::BeforeStepBudgetFinish){return Err(Failure::Unavailable);}
  self.publish(body,false)?;
  #[cfg(test)]if matches!(self.fault,Fault::AfterStepBudgetFinish){return Err(Failure::Uncertain);}
  Ok(())
 }
}
