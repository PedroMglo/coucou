//! Native System review in the original admission queue. No engine resolution.
use super::*;
impl Evidence{
 pub(in crate::pet_runtime)fn observe_system(goal:&Record,d:&Descriptor,policy_revision:String,now:u64)->Result<Self>{
  goal.validate().map_err(|_|Denial::Invalid)?;
  if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}
  if !Owner::from_runtime_descriptor(d).is_ok_and(|o|o.same_runtime_binding(&goal.spec.owner)){return Err(Denial::OwnerChanged);}
  let context=goal.execution_context.clone().filter(|c|c.schema_version==1&&c.mode==model::PreparationMode::Dedicated&&c.copied_messages==0&&!c.has_selected_data()).ok_or(Denial::ContextRequired)?;
  if goal.spec.received_from.is_some()||goal.plans.len()!=1||goal.plans[0].steps.is_empty()||goal.plans[0].steps.len()>64||goal.plans[0].steps.iter().any(|s|s.operation.as_ref().is_none_or(|o|super::super::physical_target::Action::parse(o).is_err()||o.operation_id=="spotify.control")){return Err(Denial::Unsupported);}
  if !hash(&policy_revision)||now==0||now>MAX_DATE{return Err(Denial::Invalid);}
  let semantics=model::fingerprint(&("coucou-native-system-owner-v1",&d.pet_id,&d.binding.provider,&d.binding.adapter)).map_err(|_|Denial::Invalid)?;
  Ok(Self{disclosure:None,system:true,deterministic:true,memory:None,engine_binding:None,owner:goal.spec.owner.clone(),context,model:String::new(),descriptor_revision:semantics.clone(),descriptor_semantics:semantics,policy_revision,observed_at:now,expires_at:now.saturating_add(15_000).min(MAX_DATE),tokens:false,cost:false})
 }
}
impl Review{
 pub(in crate::pet_runtime)fn prepare_system(goal:&Record,evidence:Evidence,budget:Budget,target:super::super::physical_target::Target,observe:super::super::native_system_plan::Observe,criteria:&[super::super::criteria::Input],request_id:String,decision_id:String,now:u64)->Result<Self>{
  goal.validate().map_err(|_|Denial::Invalid)?;
  if !evidence.system||!evidence.deterministic||goal.state!=State::Draft||!goal.admissions.is_empty(){return Err(Denial::NotDraft);}
  if now<goal.updated_at{return Err(Denial::Expired);}evidence.current(goal,now)?;
  if !budget.valid_runtime(now)||goal.plans.iter().map(|p|p.steps.len()).sum::<usize>()>budget.max_tool_calls as usize{return Err(Denial::Budget);}
  if !model::key(&request_id)||!model::token(&decision_id)||goal.events.iter().any(|e|e.request_id==request_id){return Err(Denial::Invalid);}
  super::super::plans::reviewed_input(goal).map_err(|_|Denial::InputLimit)?;
  let native_verification=Some(super::super::criteria::Binding::system_for(goal,criteria).map_err(|_|Denial::Invalid)?);
  let mut spec=AdmissionSpec{rule_occurrence:None,native_system_plan:None,native_peer_plan:None,execution_start:None,result_disclosure:None,funding:None,work_disclosure:None,native_verification,memory_data:None,engine_binding:None,continuation:None,proposal_hash:Some(super::super::plans::fingerprint(goal).map_err(|_|Denial::Invalid)?),planning_policy:None,schema_version:if target.binding().host_read(){28}else{27},request_id,decision_id,goal_id:goal.id.clone(),source_revision:goal.revision.clone(),source_sequence:goal.sequence,owner:goal.spec.owner.clone(),context:evidence.context.clone(),context_hash:model::fingerprint(&evidence.context).map_err(|_|Denial::Invalid)?,intent_hash:model::fingerprint(&goal.spec).map_err(|_|Denial::Invalid)?,resolved_model:String::new(),descriptor_revision:evidence.descriptor_revision.clone(),policy_revision:evidence.policy_revision.clone(),evidence_hash:evidence.fingerprint().map_err(|_|Denial::Invalid)?,approved_at:now,budget};
  let captured=super::super::native_system_plan::Captured::at(goal,&spec,target,observe,now).map_err(|_|Denial::PolicyChanged)?;
  spec.native_system_plan=Some(captured.policy().clone());if !spec.binding_format(){return Err(Denial::Invalid);}
  Ok(Self{system:Some(std::sync::Arc::new(captured)),peer:None,restored_from:goal.spec.restored_from.clone(),retained_authority:None,expires_at:now.saturating_add(REVIEW_TTL).min(spec.budget.deadline_at),spec,evidence,intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),proposals:super::super::plans::snapshot(goal)})
 }
}
