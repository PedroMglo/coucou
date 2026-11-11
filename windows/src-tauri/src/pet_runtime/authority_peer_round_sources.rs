//! Called with the SAME authority transaction held. No recursive flock, model,
//! shell, profile scan or arbitrary caller-selected filesystem root.
use super::*;
use crate::pet_runtime::{peer_round_data::{Consent,Fact},peer_round_sources::Dependency,generated_plans::Reference,Repository,results};
impl Authority{
 pub(in crate::pet_runtime::authority_store)fn peer_source_goals(&self)->Result<std::path::PathBuf>{
  let parent=self.dir.path.parent().ok_or(Failure::Unsafe)?;
  if self.dir.path.file_name().is_some_and(|n|n=="pet-runtime-consent-v1"){return Ok(parent.join("pet-runtime-v1"));}
  #[cfg(test)]if self.dir.path.file_name().is_some_and(|n|n=="authority"){return Ok(parent.join("goals"));}
  Err(Failure::Unsafe)
 }
 pub(in crate::pet_runtime::authority_store)fn peer_round_sources_unlocked(&self,body:&Grant,data:&Consent,plan:&Reference,now:u64)->Result<()>{self.peer_round_sources_core_unlocked(body,data,plan,now)?;self.peer_history_revalidate_unlocked(body,data,plan,now)}
 pub(in crate::pet_runtime::authority_store)fn peer_round_sources_core_unlocked(&self,body:&Grant,data:&Consent,plan:&Reference,now:u64)->Result<()>{
  let round=body.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|r|r.peer_data.as_ref()==Some(data)&&r.publication.as_ref().is_some_and(|p|p.phase==crate::pet_runtime::generated_plans::Phase::Acknowledged&&p.reference==*plan))).ok_or(Failure::Missing)?;
  let deps: &Dependency=round.peer_sources.as_ref().filter(|d|d.for_consent(data)).ok_or(Failure::Changed)?;
  if now<body.updated_at||!body.peer_planning||body.phase==GrantPhase::Revoked||body.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel){return Err(Failure::Revoked);}
  // Unsigned goal bytes never supply authority: they must still match the
  // signed admission, original context and latest canonical full plan hash.
  let store=crate::pet_runtime::store::Store::existing(&self.peer_source_goals()?).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
  let parent=store.source_snapshot(&body.goal_id,&data.source.pet_id).map_err(|e|match e{model::Error::Busy=>Failure::Busy,model::Error::StorageUnavailable=>Failure::Unavailable,_=>Failure::Changed})?;
  let row=parent.admissions.last().filter(|r|r.spec.decision_id==body.decision_id&&model::fingerprint(&r.spec).ok().as_ref()==Some(&body.spec_hash)).ok_or(Failure::Changed)?;
  if !row.spec.peer_iteration()||model::fingerprint(&parent.spec).ok().as_ref()!=Some(&row.spec.intent_hash)||parent.execution_context.as_ref()!=Some(&row.spec.context)||!crate::pet_runtime::plans::matches(&parent,&row.spec)||parent.state==crate::pet_runtime::model::State::Cancelled{return Err(Failure::Changed);}
  if parent.state==crate::pet_runtime::model::State::Stopping{return Err(Failure::Busy);}
  let latest=crate::pet_runtime::planned_peer::reference(&parent,&row.spec).map_err(|_|Failure::Changed)?;Self::generated_acknowledged(body,&row.spec,latest)?;
  let receipt=match results::lookup_unlocked(self,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,&round.request_id)?{results::Lookup::Available{receipt}=>receipt,_=>return Err(Failure::RecoveryRequired)};
  if receipt.input_hash!=data.input_hash||receipt.attempt_id!=round.attempt_id{return Err(Failure::Changed);}
  let facts:Vec<Fact>=serde_json::from_value(deps.exact["facts"].clone()).map_err(|_|Failure::Invalid)?;
  for item in deps.exact["observation"]["items"].as_array().ok_or(Failure::Invalid)?{
   let id=item["messageId"].as_str().ok_or(Failure::Invalid)?;let fact=facts.iter().find(|f|f.message_id==id).ok_or(Failure::Missing)?;
   let reservation=body.delegation_reservations.iter().find(|r|r.message_id==id).ok_or(Failure::Missing)?;
   if model::fingerprint(reservation).ok().as_ref()!=Some(&fact.reservation_hash){return Err(Failure::Changed);}
   if item["child"].is_null(){continue;}let captured=&item["child"];let goal_id=captured["goalId"].as_str().ok_or(Failure::Invalid)?;let pet_id=captured["petId"].as_str().ok_or(Failure::Invalid)?;
   let child=store.source_snapshot(goal_id,pet_id).map_err(|e|match e{model::Error::Busy=>Failure::Busy,model::Error::StorageUnavailable=>Failure::Unavailable,_=>Failure::Changed})?;
   if model::fingerprint(&child).ok().as_deref()!=captured["goalHash"].as_str(){return Err(Failure::Changed);}let grant=self.branch_child_unlocked(body,reservation)?;
   if model::fingerprint(&grant).ok().as_deref()!=captured["grantHash"].as_str(){return Err(Failure::Revoked);}let row=child.admissions.last().ok_or(Failure::Missing)?;
   for reference in &fact.model_results{
    let attempt=grant.budget_usage.as_ref().and_then(|b|b.attempts.iter().find(|a|a.id==reference.attempt_id)).ok_or(Failure::Missing)?;
    let fence=results::receipt_fence(self,&row.spec,&[attempt.id.clone()])?;if fence.first().is_none_or(|(_,h)|h!=&reference.file_hash){return Err(Failure::Changed);}
    if let Some(expected)=&reference.receipt_hash{let r=match results::lookup_unlocked(self,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,&attempt.request_id)?{results::Lookup::Available{receipt}=>receipt,_=>return Err(Failure::Changed)};if model::fingerprint(&r).ok().as_ref()!=Some(expected)||reference.content_hash.as_ref()!=Some(&model::hash(r.text.as_bytes())){return Err(Failure::Changed);}}
   }
  }self.fence()
 }
}
