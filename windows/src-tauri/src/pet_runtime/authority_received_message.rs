//! Passive resolution for a reply's source. Neither this owned observation nor
//! its serializable address is a delegation, model data grant or execution permit.
use super::*;
use crate::pet_runtime::{leases::Lease,messages::{Envelope,Receipt,ReceivedReference}};

pub(in crate::pet_runtime)struct Received{reference:ReceivedReference,receipt:Receipt}
impl Received{
 pub(in crate::pet_runtime)fn reference(&self)->&ReceivedReference{&self.reference}
 pub(in crate::pet_runtime)fn envelope(&self)->&Envelope{self.receipt.envelope()}
 pub(super)fn receipt_hash(&self)->Result<String>{model::fingerprint(&self.receipt).map_err(|_|Failure::Invalid)}
 pub(super)fn received_at(&self)->u64{self.receipt.received_at}
}
impl Authority{
 pub(in crate::pet_runtime)fn peer_plan_source(&self,goal:&Record,reference:&ReceivedReference,now:u64)->Result<crate::pet_runtime::received_request::Source>{
  goal.validate().map_err(|_|Failure::Invalid)?;let _lock=self.lock_shared()?;let source=self.received_source_unlocked(reference,&goal.spec.owner.pet_id,now)?;super::received_request::Witness::fact(reference,&source)
 }
 /// Sender identity is taken from the caller's HMAC-bound spec/lease. There is
 /// deliberately no caller-PetId input or scan of all inboxes. Effects using the
 /// observation must revalidate it and the original human review at dispatch.
 pub(in crate::pet_runtime)fn received_message(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,reference:&ReceivedReference,now:u64)->Result<Received>{
  let _lock=self.lock_shared()?;self.received_message_unlocked(spec,proof,lease,reference,now)
 }
 pub(super)fn received_message_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,reference:&ReceivedReference,now:u64)->Result<Received>{
  if !reference.valid()||(!matches!(spec.schema_version,2|4|5|6|7|8|9|11|14|16|17|18|20)&&!spec.peer_planning())||spec.planning_policy.is_some()&&!spec.peer_planning()||!lease.valid(spec)||now<lease.claimed_at||now>=lease.deadline_at{return Err(Failure::Invalid);}
  let caller=self.load(spec,proof)?;
  if caller.phase!=GrantPhase::Claimed||caller.lease.as_ref()!=Some(lease)||caller.runtime_owner.is_none()||caller.native_completion.is_some()||now<caller.updated_at{return Err(Failure::Revoked);}
  if let Some(f)=&spec.funding{if &f.source.reference!=reference{return Err(Failure::Changed);}self.funding_current_unlocked(spec,now)?;}
  self.received_source_unlocked(reference,&spec.owner.pet_id,now)
 }
 pub(super)fn received_source_unlocked(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<Received>{
  if !reference.valid(){return Err(Failure::Invalid);}
  // grant_file refuses restored historical entries and verifies both the native
  // body/ledger bounds and HMAC. A .grant staging is never searched or adopted.
  let(source,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;
  if source.decision_id!=reference.decision_id{return Err(Failure::Changed);}
  let receipt=source.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.envelope().message_id==reference.message_id)).ok_or(Failure::Missing)?;
  let e=receipt.envelope();
  if e.origin.decision_id!=reference.decision_id||e.recipient.pet_id!=recipient||e.origin.pet_id==recipient||model::fingerprint(&receipt.scope).map_err(|_|Failure::Invalid)?!=reference.scope_hash{return Err(Failure::Changed);}
  if now<e.created_at||now>=e.deadline_at{return Err(Failure::Expired);}
  self.fence()?;Ok(Received{reference:reference.clone(),receipt:receipt.clone()})
 }
}
