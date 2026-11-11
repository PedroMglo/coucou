//! Fresh readiness authorizes a new effect. Once started, TTL aging alone is
//! not revocation; ownership/consent/policy/context/deadline remain mandatory.
use super::{admission::{Evidence,Denial,QueuePhase},lease_controller::{Validation,Validity},leases::Lease,model::{self,Record,State}};
use crate::{pet_capabilities::{Descriptor,Lifecycle,NativeProjection},pet_config::Configuration,settings::Settings};
/// Preserve a known pre-effect catalog lock conflict through the native fence.
/// Unknown storage/proof failures must not become permission to retry an effect.
pub(super)fn read_goal(store:&super::store::Store,id:&str,pet:&str)->std::result::Result<Record,Validity>{
 store.read(id,pet).map_err(|error|if error==model::Error::Busy{Validity::Busy}else{Validity::Unknown})
}
pub(super)fn deterministic_policy_revision(config:&Configuration,goal:&Record)->model::Result<String>{
 let pet=config.pets.get(&goal.spec.owner.pet_id).ok_or(model::Error::Conflict)?;model::fingerprint(&("coucou-native-peer-data-policy-v1",pet))
}
pub(super)fn system_policy_revision(config:&Configuration,goal:&Record)->model::Result<String>{
 let pet=config.pets.get(&goal.spec.owner.pet_id).ok_or(model::Error::Conflict)?;model::fingerprint(&("coucou-native-system-policy-v1",&goal.spec.owner.pet_id,&pet.system_access))
}
pub(super)fn policy_revision(config:&Configuration,settings:&Settings,goal:&Record)->model::Result<String>{
 if goal.admissions.last().is_some_and(|r|r.spec.system()){return system_policy_revision(config,goal);}
 if goal.admissions.last().is_some_and(|r|r.spec.deterministic()){return deterministic_policy_revision(config,goal);}
 inference_policy_revision(config,settings,&goal.spec.owner)
}
pub(super)fn inference_policy_revision(config:&Configuration,settings:&Settings,owner:&super::model::Owner)->model::Result<String>{
 let pet=config.pets.get(&owner.pet_id).ok_or(model::Error::Conflict)?;
 if let Some(target)=crate::pet_inference::Target::configured(config,&owner.pet_id).map_err(|_|model::Error::Conflict)?{
  let credential=if target.choice.credential==crate::pet_inference::CredentialReference::ConfiguredLmstudioEndpoint{Some(crate::local_chat::lmstudio_key_generation())}else{None};
  return model::fingerprint(&(pet,credential));
 }
 if owner.adapter=="lmstudio"{return model::fingerprint(&(pet,settings.chat_models.get(&owner.provider),(&settings.lmstudio_url,crate::local_chat::lmstudio_key_generation()))); }
 model::fingerprint(&(pet,settings.chat_models.get(&owner.provider),if owner.adapter=="ollama"{Some(&settings.ollama_url)}else{None}))
}
pub(super)fn policy(goal:&Record,lease:&Lease,descriptor:&Descriptor,revision:&str,now:u64)->Validity{
 let Some(row)=goal.admissions.last()else{return Validity::Changed;};
 if now<lease.claimed_at||now>=lease.deadline_at||goal.state!=State::Leased||goal.id!=lease.goal_id||row.phase!=QueuePhase::Leased||row.lease.as_ref()!=Some(lease)||!lease.valid(&row.spec)
  ||goal.spec.owner!=row.spec.owner||goal.execution_context.as_ref()!=Some(&row.spec.context)||model::fingerprint(&goal.spec).ok().as_ref()!=Some(&row.spec.intent_hash)
  ||descriptor.pet_id!=goal.spec.owner.pet_id||descriptor.lifecycle!=Lifecycle::Active||!row.spec.deterministic()&&descriptor.binding.provider.as_deref()!=Some(&lease.provider)
  ||(if row.spec.deterministic(){!model::Owner::from_runtime_descriptor(descriptor).is_ok_and(|o|o.same_runtime_binding(&goal.spec.owner))}else{descriptor.binding.adapter!=Some(goal.spec.owner.adapter.as_str())})||!row.spec.deterministic()&&descriptor.binding.model.as_deref()!=Some(&lease.model)||revision!=lease.policy_revision||revision!=row.spec.policy_revision{return Validity::Changed;}
 if !descriptor.configuration_available||now<goal.updated_at{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] configuration={} observedNow={} recordUpdated={}",descriptor.configuration_available,now,goal.updated_at);return Validity::Unknown;}
 Validity::Current
}
fn evaluate(goal:&Record,lease:&Lease,capture:&NativeProjection,mode:Validation,now:u64)->Validity{
 if !capture.preferences_ready{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] preferences not ready");return Validity::Unknown;}
 let Some(config)=capture.configuration.as_ref()else{return Validity::Unknown;};
 let Some(descriptor)=capture.snapshot.pets.iter().find(|d|d.pet_id==goal.spec.owner.pet_id)else{return Validity::Changed;};
 let Ok(revision)=policy_revision(config,&capture.settings,goal)else{return Validity::Changed;};
 let valid=policy(goal,lease,descriptor,&revision,now);
 if valid==Validity::Current{if let Some(row)=goal.admissions.last(){if let Some(p)=&row.spec.native_system_plan{if now<p.prepared_at||now>=p.expires_at||!p.matches(goal,&row.spec)||!p.targets_current(config){return Validity::Changed;}}if let Some(p)=&row.spec.native_peer_plan{if now<p.prepared_at||now>=p.expires_at||!p.matches(goal,&row.spec)||!p.targets_current(config,&capture.settings){return Validity::Changed;}}}}
 if valid!=Validity::Current||goal.admissions.last().is_some_and(|r|r.spec.deterministic())||matches!(mode,Validation::InFlightPolicy|Validation::PendingDecision|Validation::Commit(_)){return valid;}
 match Evidence::observe_native(goal,descriptor,revision,now,false,false,&capture.settings,config).and_then(|e|e.for_claim(goal,now)){
  Ok(_)=>Validity::Current,
  Err(Denial::Inactive|Denial::OwnerChanged|Denial::ContextChanged|Denial::PolicyChanged)=>Validity::Changed,
  Err(_)=>Validity::Unknown,
 }
}
pub(super)fn observe(app:&tauri::AppHandle,goal:&Record,lease:&Lease,hold:&crate::pet_resources::Hold,mode:Validation,now:u64)->Validity{
 let owned=if matches!(mode,Validation::PendingDecision)&&!hold.current(){None}else{Some(hold)};
 match crate::pet_capabilities::native_projection(app,owned){Ok(capture)=>evaluate(goal,lease,&capture,mode,now),Err(_)=>{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] native projection unavailable");Validity::Unknown}}
}
#[cfg(test)]#[path="native_validity_tests.rs"]mod tests;
