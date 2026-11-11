//! Same canonical signed grant, same goal-store completion transaction.
use super::*;
use crate::pet_runtime::{native_verification::Report,criteria::{Certificate,CompletionRef},leases::Lease};
pub(in crate::pet_runtime)struct Certified<'a>{authority:&'a Authority,spec:AdmissionSpec,proof:ConsentProof,report:Report}
impl Authority{
 pub(in crate::pet_runtime)fn native_completion(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<Option<Report>>{
  let _guard=self.lock_shared()?;Ok(self.load(spec,proof)?.native_completion)
 }
 pub(in crate::pet_runtime)fn observe_native_criteria(&self,goal:&Record,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,now:u64)->Result<Report>{
  let _guard=self.lock_shared()?;let body=self.budget_body(spec,proof,lease,now,true)?;if let Some(c)=&spec.continuation{self.continuation_prefix(goal,c)?;}
  if let Some(report)=&body.native_completion{report.verify_source(goal).map_err(|_|Failure::Changed)?;return Ok(report.clone());}
  Report::observe(goal,spec,&body,nonce().map_err(|_|Failure::Unavailable)?,nonce().map_err(|_|Failure::Unavailable)?,now).map_err(|_|Failure::Invalid)
 }
 pub(in crate::pet_runtime)fn record_native_verification(&self,goal:&Record,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,report:&Report,now:u64,current:impl Fn()->bool)->Result<Report>{
  let _guard=self.lock()?;let mut body=self.load(spec,proof)?;
  if let Some(old)=&body.native_completion{
   if old!=report{return Err(Failure::Changed);}if let Some(c)=&spec.continuation{self.continuation_prefix(goal,c)?;}self.stabilize(&body)?;return Ok(old.clone());
  }
  crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;
  body=self.budget_body(spec,proof,lease,now,true)?;if let Some(c)=&spec.continuation{self.continuation_prefix(goal,c)?;}report.verify_source(goal).map_err(|_|Failure::Changed)?;
  if !report.matches_spec(spec)||report.source_grant_revision!=body.revision||report.source_grant_hash!=model::fingerprint(&body).map_err(|_|Failure::Invalid)?||report.recorded_at>now||!current(){return Err(Failure::Changed);}
  // Report is data, not an observation permit. Recompute its meaning from the
  // protected goal and current signed ledger before the backend signs it.
  let observed=Report::observe(goal,spec,&body,report.reference.id().to_owned(),report.reference.target_revision.clone(),report.recorded_at).map_err(|_|Failure::Invalid)?;
  if observed!=*report{return Err(Failure::Changed);}
  body.native_completion=Some(report.clone());body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;body.schema_version=body.native_schema();
  if !report.valid_for(&body){return Err(Failure::Invalid);}
  self.publish_guarded(body,false,||if current(){Ok(())}else{Err(Failure::Revoked)})?;Ok(report.clone())
 }
 pub(in crate::pet_runtime)fn certify_native_completion(&self,goal:&Record)->Result<Certified<'_>>{
  let _guard=self.lock_shared()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let body=self.load(&row.spec,proof)?;let report=body.native_completion.as_ref().ok_or(Failure::Missing)?;
  if !report.matches_spec(&row.spec)||!report.valid_for(&body){return Err(Failure::Invalid);}report.verify_source(goal).map_err(|_|Failure::Changed)?;if let Some(c)=&row.spec.continuation{self.continuation_prefix(goal,c)?;}self.stabilize(&body)?;
  Ok(Certified{authority:self,spec:row.spec.clone(),proof:proof.clone(),report:report.clone()})
 }
 pub(in crate::pet_runtime)fn observe_native_completed_goal(&self,goal:&Record)->Result<()>{
  let _guard=self.lock_shared()?;let row=goal.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;let report=body.native_completion.as_ref().ok_or(Failure::Missing)?;
  if !report.valid_for(&body)||!report.matches_spec(&row.spec){return Err(Failure::Invalid);}if let Some(c)=&row.spec.continuation{self.continuation_prefix(goal,c)?;}report.verify_source(goal).map_err(|_|Failure::Invalid)
 }
}
impl Certificate for Certified<'_>{
 fn reference(&self)->&CompletionRef{&self.report.reference}
 fn current(&self,goal:&Record,now:u64)->Result<()>{
  let _guard=self.authority.lock()?;let body=self.authority.load(&self.spec,&self.proof)?;
  if body.native_completion.as_ref()!=Some(&self.report){return Err(Failure::Changed);}self.report.verify_source(goal).map_err(|_|Failure::Changed)?;if let Some(c)=&self.spec.continuation{self.authority.continuation_prefix(goal,c)?;}
  if goal.state!=State::Completed{let lease=goal.admissions.last().and_then(|r|r.lease.as_ref()).ok_or(Failure::Missing)?;self.authority.budget_body(&self.spec,&self.proof,lease,now,true)?;}self.authority.fence()
 }
}
