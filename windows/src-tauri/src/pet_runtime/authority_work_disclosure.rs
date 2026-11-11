//! Validate the reporting caller using its own current signed admission.
use super::*;
impl Authority{
 pub(in crate::pet_runtime)fn verify_owned_origin_for_report(&self,g:&Record,now:u64)->model::Result<()>{
  g.validate()?;self.verify_goal_origin(g).map_err(Failure::control_error)?;let row=g.admissions.last().ok_or(model::Error::Conflict)?;
  if row.spec.schema_version!=9||row.spec.work_disclosure.as_ref().is_none_or(|p|!p.matches(g))||now<row.spec.approved_at||now>=row.spec.budget.deadline_at{return Err(model::Error::Conflict);}
  let _guard=self.lock_shared().map_err(Failure::control_error)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?).map_err(Failure::control_error)?;
  if body.work_disclosure!=row.spec.work_disclosure||!matches!((g.state,body.phase),(model::State::Queued,GrantPhase::Issued)|(model::State::Leased,GrantPhase::Claimed)){return Err(model::Error::Conflict);}Ok(())
 }
}
