//! Escrow observations/withdrawal in the original Authority. No child can run
//! on schema1 Pending escrow; unknown bound-child formats are rejected.
use super::*;
use crate::pet_runtime::delegation::{Allowance,Reservation,State as EscrowState};
use std::{sync::Arc,time::{Instant,Duration}};
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct View{pub schema_version:u32,pub pet_id:String,pub goal_id:String,pub decision_id:String,pub grant_hash:String,pub observed_at:u64,pub historical:bool,pub reservations:Vec<Reservation>,pub held:Option<Allowance>,pub consumed:Option<Allowance>,pub unknown:Option<Allowance>,pub remaining:Option<Allowance>,pub execution_authorized:bool,pub recipient_admission_support:&'static str,pub recipient_admission_available:bool,pub limitation:&'static str}
pub(super)fn view(g:&Record,b:&Grant,now:u64,historical:bool)->model::Result<View>{
 if now<b.updated_at{return Err(model::Error::ClockRollback);}let mut held=Allowance::default();let mut consumed=Allowance::default();let mut unknown=Allowance::default();
 for r in &b.delegation_reservations{match &r.state{EscrowState::Settled{settlement,..}=>{consumed.add(&settlement.consumed).ok_or(model::Error::InvalidRecord)?;unknown.add(&settlement.unknown).ok_or(model::Error::InvalidRecord)?;held.add(&settlement.unknown).ok_or(model::Error::InvalidRecord)?;},_=>{held.add(&r.held()).ok_or(model::Error::InvalidRecord)?;}}}
 let held=if historical{None}else{Some(held)};let consumed=if historical{None}else{Some(consumed)};let unknown=if historical{None}else{Some(unknown)};let remaining=if historical{None}else{b.delegation_pool.as_ref().map(|budget|{
  b.budget_remaining(budget)
 }).flatten()};
 Ok(View{schema_version:2,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:b.decision_id.clone(),grant_hash:model::fingerprint(b)?,observed_at:now,historical,reservations:b.delegation_reservations.clone(),held,consumed,unknown,remaining,execution_authorized:false,recipient_admission_support:"nativeDeterministicRequiresOwnReview",recipient_admission_available:false,limitation:"targetAvailabilityNotChecked;fundedPlanningNotImplemented;recursiveDeterministicDelegationRequiresOwnReview;nestedInferenceRequiresOwnExactF2Consent;recursiveContinuationAndPlanRevisionNotImplemented;finiteLoopbackResponseRequiresOwnF2Review;unknownConsumptionRetained;remainingIsAccountingNotAuthority"})
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{pub schema_version:u32,pub pet_id:String,pub goal_id:String,pub decision_id:String,pub reservation:Reservation,pub subgoal:String,pub recipient:String,pub source_grant_hash:String,pub review_id:String,pub expires_at:u64,pub scope:&'static str}
pub(in crate::pet_runtime)struct Candidate{authority:Authority,goal:Record,spec:AdmissionSpec,proof:ConsentProof,pub proposal:Proposal,expires:Instant}
impl Authority{
 pub(in crate::pet_runtime)fn peer_feedback_child_fence(&self,parent:&Record,decision:&str,r:&Reservation,spec:&AdmissionSpec,child_hash:&str,now:u64)->Result<()>{
  let row=parent.admissions.iter().find(|x|x.spec.decision_id==decision).ok_or(Failure::Missing)?;let _guard=self.lock_shared()?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if now<body.updated_at||!body.delegation_reservations.contains(r)||r.binding().is_none_or(|b|!b.matches(spec)){return Err(Failure::Changed);}let child=self.branch_child_unlocked(&body,r)?;
  if model::fingerprint(&child).map_err(|_|Failure::Invalid)?!=child_hash{return Err(Failure::Changed);}self.fence()
 }
 pub(in crate::pet_runtime)fn delegation_view(&self,g:&Record,decision:&str,now:u64)->model::Result<View>{
  let row=g.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(model::Error::NotFound)?;let _lock=self.lock_shared().map_err(Failure::control_error)?;
  if restoration::matches_historical_goal(self,g).map_err(Failure::control_error)?{let(body,_)=restoration::message_snapshot_file(self,&format!("{decision}.json")).map_err(Failure::control_error)?;if body.goal_id!=g.id{return Err(model::Error::Conflict);}return view(g,&body,now,true);}
  self.verify_goal_origin(g).map_err(Failure::control_error)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?).map_err(Failure::control_error)?;view(g,&body,now,false)
 }
 pub(in crate::pet_runtime)fn prepare_delegation_withdrawal(self,g:Record,decision:&str,message:&str,now:u64)->model::Result<Candidate>{
  self.verify_goal_origin(&g).map_err(Failure::control_error)?;let row=g.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(model::Error::NotFound)?;let spec=row.spec.clone();let proof=row.proof.clone().ok_or(model::Error::Conflict)?;
  let _guard=self.lock_shared().map_err(Failure::control_error)?;let body=self.load(&spec,&proof).map_err(Failure::control_error)?;let r=body.delegation_reservations.iter().find(|r|r.message_id==message&&r.state==EscrowState::Pending).ok_or(model::Error::Conflict)?.clone();let e=body.messages.as_ref().and_then(|m|m.entries.iter().find(|e|e.envelope().message_id==message)).ok_or(model::Error::Conflict)?.envelope();let proposal=Proposal{schema_version:1,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:decision.into(),reservation:r,subgoal:e.text.clone(),recipient:e.recipient.name.clone(),source_grant_hash:model::fingerprint(&body)?,review_id:nonce()?,expires_at:now.checked_add(120_000).ok_or(model::Error::ClockRollback)?,scope:"withdrawOnlyUnboundDelegationReservation"};drop(_guard);
  Ok(Candidate{authority:self,goal:g,spec,proof,proposal,expires:Instant::now()+Duration::from_secs(120)})
 }
}
impl Candidate{
 pub(in crate::pet_runtime)fn observed(&self,now:u64)->model::Result<Option<View>>{
  let a=&self.authority;let _guard=a.lock_shared().map_err(Failure::control_error)?;let b=a.load(&self.spec,&self.proof).map_err(Failure::control_error)?;let r=b.delegation_reservations.iter().find(|r|r.message_id==self.proposal.reservation.message_id).ok_or(model::Error::Conflict)?;
  if matches!(r.state,EscrowState::Withdrawn{..}){return Ok(Some(view(&self.goal,&b,now,false)?));}Ok(None)
 }
 pub(in crate::pet_runtime)fn confirm(&self,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<View>{
  let a=&self.authority;let _guard=a.lock().map_err(Failure::control_error)?;let mut b=a.load(&self.spec,&self.proof).map_err(Failure::control_error)?;let r=b.delegation_reservations.iter_mut().find(|r|r.message_id==self.proposal.reservation.message_id).ok_or(model::Error::Conflict)?;
  // Canonical withdrawal is a fact before checking a new-effect lifetime. It
  // cannot credit twice or claim a later decline undid a known withdrawal.
  if matches!(r.state,EscrowState::Withdrawn{..}){return view(&self.goal,&b,now,false);}
  if *r!=self.proposal.reservation||now>=self.proposal.expires_at||Instant::now()>=self.expires||now<b.updated_at||!human(){return Err(model::Error::Conflict);}
  // Compare/mutate only the exact branch in the CURRENT parent body. Unrelated
  // sibling progress is preserved and never overwritten with the review copy.
  r.state=EscrowState::Withdrawn{at:now,review_id:self.proposal.review_id.clone()};b.updated_at=now;b.revision=nonce()?;
  a.publish_guarded(b.clone(),false,||{if !human()||Instant::now()>=self.expires{return Err(Failure::Revoked);}Ok(())}).map_err(Failure::control_error)?;view(&self.goal,&b,now,false)
 }
}
