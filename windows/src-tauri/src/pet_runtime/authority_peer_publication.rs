//! A native proof of an existing publication, deliberately not a model/data
//! Approved. Reconciliation never supplies a worker, quota or another budget.
use super::*;
use crate::pet_runtime::{generated_plans::{Reference,Phase},peer_round_data::Consent,leases::Lease};
pub(in crate::pet_runtime)struct ReconciledPeerPublication{goal_hash:String,request:String,reference:Reference,data:Consent,at:u64,expires_at:u64,deadline:std::time::Instant}
impl ReconciledPeerPublication{
 pub(in crate::pet_runtime)fn data(&self)->&Consent{&self.data}
 pub(in crate::pet_runtime)fn expires_at(&self)->u64{self.expires_at}
 pub(in crate::pet_runtime)fn pending(&self,now:u64)->bool{now>=self.at&&now<self.expires_at&&std::time::Instant::now()<self.deadline}
 pub(in crate::pet_runtime)fn verify(&self,a:&Authority,s:&AdmissionSpec,p:&ConsentProof,l:&Lease,g:&Record,now:u64)->Result<()>{
  if !self.pending(now){return Err(Failure::Expired);}if model::fingerprint(g).map_err(|_|Failure::Invalid)?!=self.goal_hash{return Err(Failure::Changed);}
  let current=a.reconciled_peer_publication(s,p,l,g,&self.request,now)?;
  if current.data!=self.data||current.reference!=self.reference||current.goal_hash!=self.goal_hash||current.expires_at!=self.expires_at{return Err(Failure::Changed);}Ok(())
 }
}
impl Authority{
 pub(in crate::pet_runtime)fn reconciled_peer_publication(&self,s:&AdmissionSpec,p:&ConsentProof,l:&Lease,g:&Record,request:&str,now:u64)->Result<ReconciledPeerPublication>{
  if !s.peer_iteration()||!model::key(request)||g.state!=State::Leased{return Err(Failure::Invalid);}
  let reference=crate::pet_runtime::planned_peer::reference(g,s).map_err(|_|Failure::Changed)?.clone();let(canonical,_)=self.generated_candidate(s,p,l,g,&reference.plan_id)?;if canonical!=reference{return Err(Failure::Changed);}
  let _lock=self.lock_shared()?;let body=self.budget_body(s,p,l,now,false)?;
  if !matches!(body.phase,GrantPhase::Claimed|GrantPhase::Waiting)||body.stop_reason.is_some()||body.runtime_owner.is_none()||now>=l.deadline_at{return Err(Failure::Revoked);}
  let round=body.planning_usage.as_ref().and_then(|j|j.rounds.last()).filter(|r|r.request_id==request&&r.publication.as_ref().is_some_and(|p|p.phase==Phase::Acknowledged&&p.reference==reference)).ok_or(Failure::Missing)?;
  let data=round.peer_data.as_ref().filter(|d|d.for_spec(s,l)&&d.for_source(&round.source,&round.input_hash)).ok_or(Failure::Changed)?.clone();
  if now<data.approved_at||now>=data.expires_at{return Err(Failure::Expired);}self.peer_round_sources_unlocked(&body,&data,&reference,now)?;
  let expires_at=data.expires_at.min(l.deadline_at);let deadline=std::time::Instant::now().checked_add(std::time::Duration::from_millis(expires_at-now)).ok_or(Failure::Invalid)?;self.fence()?;
  Ok(ReconciledPeerPublication{goal_hash:model::fingerprint(g).map_err(|_|Failure::Invalid)?,request:request.into(),reference,data,at:now,expires_at,deadline})
 }
}
