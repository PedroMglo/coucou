//! Canonical signed receipt + acknowledged original proposal. Published JSON
//! is provenance DATA, never a provider grant or an executable approval.
use super::*;
use crate::pet_runtime::{messages::{Envelope,ReceivedReference},planned_peer_data::Published};
impl Authority{
 /// Typed canonical DATA observation for dependency fences, including when the
 /// caller already owns the writer lock. It grants no execution or model use.
 pub(in crate::pet_runtime)fn selected_peer_data_source_unlocked(&self,source:&crate::pet_runtime::received_request::Source,now:u64)->Result<Envelope>{
  if !source.valid(){return Err(Failure::Invalid);}let r=self.received_source_unlocked(&source.reference,&source.recipient_pet_id,now)?;
  if super::received_request::Witness::fact(&source.reference,&r)?!=*source{return Err(Failure::Changed);}Ok(r.envelope().clone())
 }

 pub(in crate::pet_runtime)fn capture_planned_origin_dependencies(&self,goal:&Record,memory:&Path,now:u64)->Result<crate::pet_runtime::received_result_data::Dependencies>{
  let _guard=self.lock_shared()?;self.verify_received_origin(goal)?;
  let origin=goal.spec.received_from.as_ref().filter(|o|o.body.planned_dependencies.is_some()).ok_or(Failure::Invalid)?;
  let(p,e)=self.received_planned_data_unlocked(&origin.body.source.reference,&goal.spec.owner.pet_id,now)?;
  if !matches!(p.schema_version,3|4)||origin.body.planned_dependencies.as_deref()!=Some(&p){return Err(Failure::Changed);}
  crate::pet_runtime::received_result_data::Dependencies::capture(self,memory,&origin.body.source.reference,&e,now)
 }
 // Like received-context fences, this also runs INSIDE our own effect writer.
 // Verify canonical atomic/HMAC snapshots; never take a second flock here.
 pub(in crate::pet_runtime)fn planned_origin_dependencies_current(&self,dependencies:&crate::pet_runtime::received_result_data::Dependencies,now:u64)->Result<()>{self.fence()?;dependencies.verify(self,now)?;self.fence()}
 pub(in crate::pet_runtime)fn received_funding_purpose(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<crate::pet_runtime::received_request::Purpose>{let _guard=self.lock_shared()?;let r=self.received_source_unlocked(reference,recipient,now)?;let e=r.envelope();if e.kind!=crate::pet_runtime::messages::Kind::Delegation||e.delegation.is_none(){return Err(Failure::Invalid);}match e.schema_version{4=>Ok(crate::pet_runtime::received_request::Purpose::Funded),12|14|16=>{self.received_planned_data_unlocked(reference,recipient,now)?;Ok(crate::pet_runtime::received_request::Purpose::DerivedFunded)},_=>Err(Failure::Invalid)}}

 pub(in crate::pet_runtime)fn received_request_purpose(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<crate::pet_runtime::received_request::Purpose>{let _guard=self.lock_shared()?;let r=self.received_source_unlocked(reference,recipient,now)?;if r.envelope().planned_data.is_some(){self.received_planned_data_unlocked(reference,recipient,now)?;Ok(crate::pet_runtime::received_request::Purpose::PlannedData)}else{Ok(crate::pet_runtime::received_request::Purpose::Request)}}

 pub(in crate::pet_runtime)fn received_planned_data_unlocked(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<(Published,Envelope)>{
  let r=self.received_source_unlocked(reference,recipient,now)?;let e=r.envelope();let p=e.planned_data.as_ref().filter(|p|p.matches_envelope(e)).ok_or(Failure::Invalid)?;
  let(body,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;
  if !body.peer_planning||(match p.schema_version{1=>body.schema_version!=37,2=>!matches!(body.schema_version,41|42|43),3|4=>body.schema_version!=(if body.funded_by.as_ref().is_some_and(|f|f.schema_version==9){47}else if p.schema_version==4{46}else{45})||body.peer_memory_planning.as_ref()!=Some(&p.source),_=>true})||body.operation!=Operation::GoalPlanning||body.phase==GrantPhase::Revoked||body.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel)||body.spec_hash!=p.source.admission_hash||body.goal_id!=p.source.goal_id||body.decision_id!=p.generated_plan.decision_id||body.planning_usage.as_ref().is_none_or(|j|!j.rounds.iter().any(|r|r.publication.as_ref().is_some_and(|a|a.phase==crate::pet_runtime::generated_plans::Phase::Acknowledged&&a.reference==p.generated_plan))){return Err(Failure::Revoked);}if let Some(data)=&p.round_data{self.peer_round_sources_unlocked(&body,data,&p.generated_plan,now)?;}self.fence()?;Ok((p.clone(),e.clone()))
 }
}
