//! Deterministic verification follows only predicates sealed in the original
//! human goal admission. No new approval, allowance, model or recipient effect.
use super::*;
use crate::pet_runtime::native_verification::Report;
#[derive(serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct Progress{pub report:Report,pub completion:Option<model::Receipt>,#[serde(skip_serializing_if="Option::is_none")]pub closure_pending:Option<model::Error>,pub recipient_execution_verified:bool}
impl Owned{
 pub(in crate::pet_runtime)fn verify_native_goal(&self,store:&Store,goal:&Record)->model::Result<Progress>{
  if (!matches!(self.live.spec.schema_version,5|6|7|8|9|10|11|14|16|17|18|20)||self.live.spec.native_verification.is_none())&&!self.live.spec.peer_planning()||goal.admissions.last().is_none_or(|r|model::fingerprint(&r.spec).ok()!=model::fingerprint(&self.live.spec).ok()){return Err(model::Error::Conflict);}
  if goal.state!=model::State::Completed&&self.workers()!=0{return Err(if self.pending_validity(crate::assistant_memory::now())==Validity::Changed{model::Error::Conflict}else{model::Error::Busy});}
  let authority=Authority::existing(&self.live.authority_path).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  // Exactly one immutable prepared report is retained through storage faults.
  // Stop never takes this mutex; it keeps its native lifetime priority.
  let mut retained=self.live.native_completion.try_lock().map_err(|_|model::Error::Busy)?;
  let report=if let Some(old)=authority.native_completion(&self.live.spec,&self.live.proof).map_err(crate::pet_runtime::authority::Failure::control_error)?{old}else{
   self.completion_authority(goal,crate::assistant_memory::now())?;
   let observed=if let Some(old)=&*retained{old.clone()}else{authority.observe_native_criteria(goal,&self.live.spec,&self.live.proof,&self.live.lease,crate::assistant_memory::now()).map_err(crate::pet_runtime::authority::Failure::control_error)?};
   if !observed.criteria_verified{return Ok(Progress{report:observed,completion:None,closure_pending:None,recipient_execution_verified:false});}
   *retained=Some(observed.clone());let _worker=self.readiness_worker(crate::assistant_memory::now())?;
   store.with_message_goal(goal,|current|authority.record_native_verification(current,&self.live.spec,&self.live.proof,&self.live.lease,&observed,crate::assistant_memory::now(),||self.message_control_current(current)).map_err(crate::pet_runtime::authority::Failure::control_error))?
  };
  let certificate=authority.certify_native_completion(goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  let receipt=self.with_completion_lock(||store.complete_certified(&certificate,crate::assistant_memory::now(),|current|self.completion_authority(current,crate::assistant_memory::now())))?;
  Ok(Progress{report,completion:Some(receipt),closure_pending:None,recipient_execution_verified:false})
 }
}
