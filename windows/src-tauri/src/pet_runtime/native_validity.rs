//! Fresh readiness authorizes a new effect. Once started, TTL aging alone is
//! not revocation; ownership/consent/policy/context/deadline remain mandatory.
use super::{admission::{Evidence,Denial,QueuePhase},lease_controller::{Validation,Validity},leases::Lease,model::{self,Record,State}};
use crate::{pet_capabilities::{Descriptor,Lifecycle,NativeProjection},pet_config::Configuration,settings::Settings};
pub(super)fn policy_revision(config:&Configuration,settings:&Settings,goal:&Record)->model::Result<String>{
 let pet=config.pets.get(&goal.spec.owner.pet_id).ok_or(model::Error::Conflict)?;
 model::fingerprint(&(pet,settings.chat_models.get(&goal.spec.owner.provider),if goal.spec.owner.adapter=="ollama"{Some(&settings.ollama_url)}else{None}))
}
pub(super)fn policy(goal:&Record,lease:&Lease,descriptor:&Descriptor,revision:&str,now:u64)->Validity{
 let Some(row)=goal.admissions.last()else{return Validity::Changed;};
 if now<lease.claimed_at||now>=lease.deadline_at||goal.state!=State::Leased||goal.id!=lease.goal_id||row.phase!=QueuePhase::Leased||row.lease.as_ref()!=Some(lease)||!lease.valid(&row.spec)
  ||goal.spec.owner!=row.spec.owner||goal.execution_context.as_ref()!=Some(&row.spec.context)||model::fingerprint(&goal.spec).ok().as_ref()!=Some(&row.spec.intent_hash)
  ||descriptor.pet_id!=goal.spec.owner.pet_id||descriptor.lifecycle!=Lifecycle::Active||descriptor.binding.provider.as_deref()!=Some(&lease.provider)
  ||descriptor.binding.adapter!=Some(goal.spec.owner.adapter.as_str())||descriptor.binding.model.as_deref()!=Some(&lease.model)||revision!=lease.policy_revision||revision!=row.spec.policy_revision{return Validity::Changed;}
 if !descriptor.configuration_available||now<goal.updated_at{return Validity::Unknown;}
 Validity::Current
}
fn evaluate(goal:&Record,lease:&Lease,capture:&NativeProjection,mode:Validation,now:u64)->Validity{
 let Some(config)=capture.configuration.as_ref()else{return Validity::Unknown;};
 let Some(descriptor)=capture.snapshot.pets.iter().find(|d|d.pet_id==goal.spec.owner.pet_id)else{return Validity::Changed;};
 let Ok(revision)=policy_revision(config,&capture.settings,goal)else{return Validity::Changed;};
 let valid=policy(goal,lease,descriptor,&revision,now);if valid!=Validity::Current||matches!(mode,Validation::InFlightPolicy|Validation::Commit(_)){return valid;}
 match Evidence::observe(goal,descriptor,revision,now,false,false).and_then(|e|e.for_claim(goal,now)){
  Ok(_)=>Validity::Current,
  Err(Denial::Inactive|Denial::OwnerChanged|Denial::ContextChanged|Denial::PolicyChanged)=>Validity::Changed,
  Err(_)=>Validity::Unknown,
 }
}
pub(super)fn observe(app:&tauri::AppHandle,goal:&Record,lease:&Lease,hold:&crate::pet_resources::Hold,mode:Validation,now:u64)->Validity{
 match crate::pet_capabilities::native_projection(app,Some(hold)){Ok(capture)=>evaluate(goal,lease,&capture,mode,now),Err(_)=>Validity::Unknown}
}
#[cfg(test)]#[path="native_validity_tests.rs"]mod tests;
