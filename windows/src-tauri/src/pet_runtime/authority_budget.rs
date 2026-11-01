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
  let _lock=self.lock_shared()?;let body=self.budget_body(spec,proof,lease,now,true)?;
  let used=body.budget_usage.as_ref().map(|u|u.attempts.iter().map(Attempt::charge).sum::<u64>()).unwrap_or(0);
  spec.budget.max_output_bytes.checked_sub(used).ok_or(Failure::Invalid)
 }
 pub(in crate::pet_runtime)fn reserve_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,now:u64)->Result<()>{
  // Planning cannot bypass its journal through the generic model/tool path.
  if spec.planning_policy.is_some(){return Err(Failure::Invalid);}
  self.reserve_attempt_inner(spec,proof,lease,attempt,None,None,now)
 }
 pub(in crate::pet_runtime)fn reserve_planning_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,request:&crate::pet_runtime::planning_journal::Request,now:u64)->Result<()>{
  if !request.matches(spec,lease)||request.request_id!=attempt.request_id||attempt.kind!=crate::pet_runtime::effect_budget::Kind::Model{return Err(Failure::Invalid);}
  self.reserve_attempt_inner(spec,proof,lease,attempt,Some(request),None,now)
 }
 pub(in crate::pet_runtime)fn reserve_generated_step_budget(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&crate::pet_runtime::steps::Scope,attempt:&Attempt,now:u64)->Result<()>{
  if scope.schema_version!=2||!scope.belongs(spec,lease)||attempt.kind!=crate::pet_runtime::effect_budget::Kind::Tool||attempt.request_id!=scope.request_id
   ||attempt.input_hash!=model::fingerprint(scope).map_err(|_|Failure::Invalid)?||attempt.output_reserved>scope.max_output_bytes{return Err(Failure::Invalid);}
  self.reserve_attempt_inner(spec,proof,lease,attempt,None,Some(scope),now)
 }
 fn reserve_attempt_inner(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,request:Option<&crate::pet_runtime::planning_journal::Request>,step:Option<&crate::pet_runtime::steps::Scope>,now:u64)->Result<()>{
  if !attempt.valid()||attempt.phase!=Phase::Reserved||attempt.reserved_at>now||attempt.reserved_at<lease.claimed_at{return Err(Failure::Invalid);}
  // Finite unmetered caps cannot be approximated by call counts/output bytes.
  if spec.budget.max_tokens.is_some()||spec.budget.max_cost_micros.is_some(){return Err(Failure::Invalid);}
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,true)?;
  if (request.is_some()||step.is_some())!=body.planning_usage.is_some()||request.is_some()&&step.is_some(){return Err(Failure::Invalid);}
  if let Some(scope)=step{
   let reference=scope.generated_plan.as_ref().ok_or(Failure::Invalid)?;Self::generated_acknowledged(&body,spec,reference)?;
   // reserve_step already authenticated the human-approved manifest and sealed
   // this exact native attempt. Serialized scope data alone cannot debit/arm IO.
   let saved=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
   if saved.scope!=*scope||saved.phase!=crate::pet_runtime::steps::Phase::Reserved||saved.attempt.as_ref().is_none_or(|a|!a.same_request(attempt)){return Err(Failure::RecoveryRequired);}
  }
  if let Some(request)=request{
   let journal=body.planning_usage.as_ref().ok_or(Failure::Invalid)?;
   for round in &journal.rounds{if let Some(feedback)=&round.feedback{self.planning_feedback_unlocked(spec,proof,feedback)?;}}
   if journal.rounds.iter().any(|r|r.request_id==request.request_id&&r.source!=request.source){return Err(Failure::Changed);}
  }
  let ledger=body.budget_usage.get_or_insert_with(Ledger::empty);
  if let Some(old)=ledger.attempts.iter().find(|a|a.id==attempt.id||a.request_id==attempt.request_id){
   return if old.same_request(attempt)&&old.phase==Phase::Reserved{self.stabilize(&body)}else{Err(Failure::RecoveryRequired)};
  }
  ledger.attempts.push(attempt.clone());if !ledger.valid_for(spec){return Err(Failure::Capacity);}
  if let Some(request)=request{
   let journal=body.planning_usage.as_mut().ok_or(Failure::Invalid)?;
   journal.append(request,attempt).map_err(|_|Failure::RecoveryRequired)?;
   if !journal.valid_for(spec,body.lease.as_ref(),body.budget_usage.as_ref()){return Err(Failure::Capacity);}
  }
  body.schema_version=if body.runtime_owner.is_some(){6}else if body.resource_usage.is_some(){5}else if body.planning_usage.is_some(){4}else{3};body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
 }
 pub(in crate::pet_runtime)fn start_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,now:u64)->Result<()>{
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,true)?;
  let old=body.budget_usage.as_mut().and_then(|l|l.attempts.iter_mut().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
  if !old.same_request(attempt)||old.phase!=Phase::Reserved||now<old.reserved_at{return Err(Failure::RecoveryRequired);}
  old.phase=Phase::Started;old.started_at=Some(now);
  if let Some(journal)=&mut body.planning_usage{journal.update_attempt(old).map_err(|_|Failure::Invalid)?;}
  body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
 }
 pub(in crate::pet_runtime)fn finish_attempt(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,attempt:&Attempt,accepted:Option<u64>,now:u64)->Result<()>{
  let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
  let old=body.budget_usage.as_mut().and_then(|l|l.attempts.iter_mut().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
  if !old.same_request(attempt)||accepted.is_some_and(|n|n>old.output_reserved){return Err(Failure::Invalid);}
  let phase=if accepted.is_some(){Phase::Settled}else{Phase::Uncertain};
  if old.phase==phase&&old.accepted_output==accepted{return self.stabilize(&body);}
  if old.phase!=Phase::Started||now<old.started_at.ok_or(Failure::Invalid)?{return Err(Failure::RecoveryRequired);}
  old.phase=phase;old.accepted_output=accepted;old.finished_at=Some(now);
  if let Some(journal)=&mut body.planning_usage{journal.update_attempt(old).map_err(|_|Failure::Invalid)?;}
  body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;
  #[cfg(test)]if matches!(self.fault,Fault::BeforeStepBudgetFinish){return Err(Failure::Unavailable);}
  self.publish(body,false)?;
  #[cfg(test)]if matches!(self.fault,Fault::AfterStepBudgetFinish){return Err(Failure::Uncertain);}
  Ok(())
 }
}
