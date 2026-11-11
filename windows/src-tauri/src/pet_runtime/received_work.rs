//! Native observation of ONE own goal linked to a certified request. These
//! DTOs are facts, never executable approvals or a delegated allowance.
use super::{model::{self,Record,State as GoalState},messages::ReceivedReference,reconciliation::Snapshot,runtime_owner::State as OwnerState};
use serde::{Serialize,Deserialize};
#[derive(Clone,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Input{pub pet_id:String,pub goal_id:String,pub revision:String,pub source:ReceivedReference}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum State{Proposed,AcceptedQueued,SessionReserved,AttemptInFlight,Waiting,AwaitingVerification,Stopping,Paused,Cancelled,Completed,RecoveryRequired,DeadlineElapsed,Historical,Unavailable}
#[derive(Clone,Debug,Default,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Attempts{pub reserved:usize,pub started:usize,pub settled:usize,pub uncertain:usize,pub unknown_outcomes:usize}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Link{pub reference:ReceivedReference,pub sender_pet_id:String,pub sender_goal_id:String,pub recipient_pet_id:String,pub receipt_hash:String,pub envelope_hash:String,pub received_at:u64,pub deadline_at:u64}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Admission{pub decision_id:String,pub approved_at:u64,pub deadline_at:u64,pub grant_revision:String,pub grant_hash:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct View{
 pub schema_version:u32,pub scope:&'static str,pub pet_id:String,pub goal_id:String,pub goal_revision:String,pub goal_hash:String,
 pub source:Link,pub origin_hash:String,pub observed_at:u64,pub state:State,pub admission:Option<Admission>,pub owner_state:Option<OwnerState>,
 pub attempts:Option<Attempts>,pub evidence_complete:bool,pub criteria_verified:bool,pub request_within_deadline:bool,pub execution_authorized:bool,
}
impl View{
 pub(super)fn base(goal:&Record,now:u64)->model::Result<Self>{
  let origin=goal.spec.received_from.as_ref().ok_or(model::Error::InvalidInput)?;let s=&origin.body.source;
  if now<goal.updated_at||now<s.received_at||now>8_640_000_000_000_000{return Err(model::Error::ClockRollback);}
  Ok(Self{schema_version:1,scope:"ownGoalLinkedToReceivedRequest",pet_id:goal.spec.owner.pet_id.clone(),goal_id:goal.id.clone(),goal_revision:goal.revision.clone(),goal_hash:model::fingerprint(goal)?,
   source:Link{reference:s.reference.clone(),sender_pet_id:s.sender_pet_id.clone(),sender_goal_id:s.sender_goal_id.clone(),recipient_pet_id:s.recipient_pet_id.clone(),receipt_hash:s.receipt_hash.clone(),envelope_hash:s.envelope_hash.clone(),received_at:s.received_at,deadline_at:s.deadline_at},origin_hash:model::fingerprint(origin)?,observed_at:now,state:State::Historical,admission:None,owner_state:None,attempts:None,evidence_complete:false,criteria_verified:false,request_within_deadline:now<s.deadline_at,execution_authorized:false})
 }
 pub(super)fn observed(mut self,goal:&Record,snapshot:Snapshot)->model::Result<Self>{
  if snapshot.goal_id!=self.goal_id||snapshot.pet_id!=self.pet_id||snapshot.goal_revision!=self.goal_revision||snapshot.goal_hash!=self.goal_hash||snapshot.observed_at!=self.observed_at{return Err(model::Error::Conflict);}
  self.evidence_complete=snapshot.source_consistent&&snapshot.scan_complete&&snapshot.grant.as_ref().is_none_or(|g|g.control_projection_current&&g.goal_lease_matches);
  let mut attempts=Attempts::default();
  for a in &snapshot.attempts{match a.phase{"reserved"=>attempts.reserved+=1,"started"=>attempts.started+=1,"settled"=>attempts.settled+=1,"uncertain"=>attempts.uncertain+=1,_=>self.evidence_complete=false};if matches!(a.phase,"settled"|"uncertain")&&!a.outcome_known{attempts.unknown_outcomes+=1;}}
  if let Some(grant)=&snapshot.grant{
   let row=goal.admissions.last().ok_or(model::Error::Conflict)?;if row.spec.decision_id!=grant.decision_id{return Err(model::Error::Conflict);}
   self.admission=Some(Admission{decision_id:grant.decision_id.clone(),approved_at:row.spec.approved_at,deadline_at:row.spec.budget.deadline_at,grant_revision:grant.revision.clone(),grant_hash:grant.hash.clone()});self.owner_state=Some(grant.owner.state);
  }
  self.criteria_verified=self.evidence_complete&&(snapshot.criteria_verified||snapshot.goal_complete);
  self.state=if !self.evidence_complete{State::Unavailable}else{match goal.state{
   GoalState::Draft=>if goal.admissions.is_empty(){State::Proposed}else{State::Unavailable},
   GoalState::Paused=>State::Paused,GoalState::Cancelled=>State::Cancelled,GoalState::Stopping=>State::Stopping,GoalState::RecoveryRequired=>State::RecoveryRequired,
   GoalState::Completed=>if snapshot.goal_complete{State::Completed}else{State::Unavailable},
   GoalState::Queued|GoalState::Leased=>{
    if self.admission.is_none(){State::Unavailable}
    else if self.observed_at>=self.admission.as_ref().unwrap().deadline_at{State::DeadlineElapsed}
    else if attempts.unknown_outcomes>0{State::RecoveryRequired}
    else if goal.state==GoalState::Queued{State::AcceptedQueued}
    else if self.owner_state!=Some(OwnerState::OriginalProcessLive){State::RecoveryRequired}
    else if snapshot.grant.as_ref().is_some_and(|g|g.phase=="waiting"){State::Waiting}
    else if attempts.started>0{State::AttemptInFlight}
    else if attempts.reserved>0||attempts.settled==0{State::SessionReserved}
    else{State::AwaitingVerification}
   }
  }};if self.evidence_complete{self.attempts=Some(attempts);}Ok(self)
 }
}
