//! Same native Review/Approved and signed admission. No new authority ledger.
use super::*;
impl Evidence{
 pub(in crate::pet_runtime)fn observe_deterministic(goal:&Record,d:&Descriptor,policy_revision:String,now:u64)->Result<Self>{
  goal.validate().map_err(|_|Denial::Invalid)?;
  if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}
  let owner=Owner::from_runtime_descriptor(d).map_err(|_|Denial::OwnerChanged)?;
  if !owner.same_runtime_binding(&goal.spec.owner){return Err(Denial::OwnerChanged);}
  let context=goal.execution_context.clone().ok_or(Denial::ContextRequired)?;
  if context.memory.is_some()||!super::super::deterministic::plan(goal){return Err(Denial::Unsupported);}
  if !hash(&policy_revision)||now==0||now>MAX_DATE{return Err(Denial::Invalid);}
  let semantics=model::fingerprint(&("coucou-runtime-peer-data-v1",&d.pet_id,&d.binding.provider,&d.binding.adapter)).map_err(|_|Denial::Invalid)?;
  Ok(Self{#[cfg(target_os="linux")]disclosure:None,system:false,deterministic:true,#[cfg(target_os="linux")]memory:None,engine_binding:None,owner:goal.spec.owner.clone(),context,model:String::new(),descriptor_revision:semantics.clone(),descriptor_semantics:semantics,policy_revision,observed_at:now,expires_at:now.saturating_add(15_000).min(MAX_DATE),tokens:false,cost:false})
 }
}
impl Review{
 #[cfg(target_os="linux")]
 pub(in crate::pet_runtime)fn prepare_deterministic_continuation(goal:&Record,evidence:Evidence,budget:Budget,captured:super::super::continuation::Captured,request:String,decision:String,now:u64)->Result<Self>{
  let(reference,authority)=captured.into_parts();
  if !evidence.deterministic||!matches!(reference.schema_version,3|4|5|8)||!reference.source_matches(goal)||!super::super::deterministic::proposed(goal){return Err(Denial::Unsupported);}
  authority.continuation_current(goal,&reference).map_err(continuation_failure)?;
  let native=reference.native.as_ref().ok_or(Denial::Invalid)?;let mut missing=0;
  for p in &goal.plans{for s in &p.steps{if native.covers(&p.id,&s.id){continue;}missing+=1;
   if s.operation.as_ref().and_then(|o|super::super::messages::Input::parse(o).ok()).is_none_or(|a|a.deadline()<=now||a.deadline()>budget.deadline_at){return Err(Denial::Expired);}
  }}
  if missing>budget.max_tool_calls as usize||missing>super::super::messages::MAX_MESSAGES{return Err(Denial::Budget);}
  let mut review=Self::prepare_source(goal,evidence,budget,request,decision,now,Some(reference))?;review.retained_authority=Some(authority);Ok(review)
 }

 pub(in crate::pet_runtime)fn prepare_deterministic(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64)->Result<Self>{
  if !evidence.deterministic||!super::super::deterministic::proposed(goal)||goal.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.as_ref().and_then(|o|super::super::messages::Input::parse(o).ok()).is_none_or(|a|a.deadline()<=now||a.deadline()>budget.deadline_at)){return Err(Denial::Unsupported);}
  Self::prepare_source(goal,evidence,budget,request_id,decision_id,now,None)
 }
}

impl Review{
 pub(in crate::pet_runtime)fn with_work_disclosure(mut self,policy:super::super::work_disclosure::Policy)->Result<Self>{
  if !self.evidence.deterministic||self.spec.continuation.is_some()||self.spec.native_verification.is_none()||!policy.valid()||self.spec.proposal_hash.as_ref()!=Some(&policy.proposal_hash)||policy.arguments.expires_at>self.spec.budget.deadline_at{return Err(Denial::Unsupported);}
  self.spec.schema_version=9;self.spec.work_disclosure=Some(policy);Ok(self)
 }
}

#[cfg(target_os="linux")]impl Evidence{
 pub(in crate::pet_runtime)fn observe_result(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,data:super::super::result_delivery_data::Captured)->Result<Self>{
  goal.validate().map_err(|_|Denial::Invalid)?;if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}let owner=Owner::from_runtime_descriptor(d).map_err(|_|Denial::OwnerChanged)?;if !owner.same_runtime_binding(&goal.spec.owner){return Err(Denial::OwnerChanged);}
  let context=goal.execution_context.clone().filter(|c|matches!(c.schema_version,4|6)).ok_or(Denial::ContextRequired)?;if !data.policy().matches(goal)||!super::super::deterministic::plan(goal){return Err(Denial::Unsupported);}data.current(goal,now).map_err(|_|Denial::ContextChanged)?;
  if !hash(&policy_revision)||now==0||now>MAX_DATE{return Err(Denial::Invalid);}let semantics=model::fingerprint(&("coucou-runtime-peer-data-v1",&d.pet_id,&d.binding.provider,&d.binding.adapter)).map_err(|_|Denial::Invalid)?;
  Ok(Self{disclosure:Some(data),system:false,deterministic:true,memory:None,engine_binding:None,owner:goal.spec.owner.clone(),context,model:String::new(),descriptor_revision:semantics.clone(),descriptor_semantics:semantics,policy_revision,observed_at:now,expires_at:now.saturating_add(15_000).min(MAX_DATE),tokens:false,cost:false})
 }
}
#[cfg(target_os="linux")]impl Review{
 pub(in crate::pet_runtime)fn prepare_result_disclosure(goal:&Record,evidence:Evidence,budget:Budget,captured:super::super::continuation::Captured,request:String,decision:String,now:u64)->Result<Self>{
  let(reference,authority)=captured.into_parts();let policy=evidence.result_policy().ok_or(Denial::DataConsentRequired)?;
  if !evidence.deterministic||reference.schema_version!=(if policy.schema_version==2{10}else{7})||!reference.source_matches(goal)||!policy.matches(goal)||budget.max_model_calls!=0||budget.max_tool_calls!=1||budget.deadline_at<policy.arguments.expires_at{return Err(Denial::Unsupported);}
  authority.continuation_current(goal,&reference).map_err(continuation_failure)?;let mut r=Self::prepare_source(goal,evidence,budget,request,decision,now,Some(reference))?;if !matches!(r.spec.schema_version,14|20)||!r.spec.binding_format(){return Err(Denial::Invalid);}r.retained_authority=Some(authority);Ok(r)
 }
}
