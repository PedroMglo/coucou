//! A rule occurrence is its own goal, backed by the original parent allocation.
//! This reference is DATA; only the authenticated common Authority validates it.
use super::*;
use super::super::admission::{AdmissionSpec,Payment,StartScope};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct OccurrenceReference{pub schema_version:u32,pub rule_id:String,pub rule_revision:u64,pub rule_decision_id:String,pub policy_hash:String,pub occurrence_id:String,pub slot_index:u8,pub at:u64,pub deadline_at:u64}
impl OccurrenceReference{
 pub(crate)fn for_spec(&self,s:&AdmissionSpec)->bool{
  self.schema_version==1&&valid_id(&self.rule_id)&&self.rule_revision>0&&model::token(&self.rule_decision_id)&&[&self.policy_hash,&self.occurrence_id].iter().all(|s|super::super::authority::decode(s).is_some())&&self.at>0&&self.at<=s.approved_at&&s.approved_at<self.deadline_at&&self.deadline_at==s.budget.deadline_at
   &&s.schema_version==29&&s.rule_occurrence.as_ref()==Some(self)&&s.owner.local_engine()&&s.context.schema_version==1&&s.context.mode==model::PreparationMode::Dedicated&&!s.context.has_selected_data()&&s.context.source_conversation_id.is_none()&&s.context.source_messages==0&&s.context.copied_messages==0
   &&s.budget.payment==Payment::LocalOnly&&s.budget.max_model_calls==1&&s.budget.max_tool_calls==0&&s.budget.max_tokens.is_none()&&s.budget.max_cost_micros.is_none()&&s.budget.valid(s.approved_at)&&s.proposal_hash.is_some()&&s.execution_start.as_ref().is_some_and(|p|p.schema_version==1&&p.not_before_at.is_none()&&p.scope==StartScope::FiniteResponse&&p.mode==super::super::planning::Start::WhenAvailable)
   &&s.native_system_plan.is_none()&&s.native_peer_plan.is_none()&&s.native_verification.is_none()&&s.funding.is_none()&&s.result_disclosure.is_none()&&s.work_disclosure.is_none()&&s.continuation.is_none()&&s.memory_data.is_none()&&s.planning_policy.is_none()&&super::super::admission::model_id(&s.resolved_model)
   &&(if s.owner.adapter=="lmstudio"{s.engine_binding.as_ref().is_some_and(|b|b.valid_for(&s.resolved_model))}else{s.engine_binding.is_none()})
 }
}
/// Retain only the fields not already fixed by the sealed policy and durable
/// goal/context. Reconstitution must match spec_hash byte-for-byte. No refreshed
/// approval date, decision, destination, deadline or allowance is permitted.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RuleAdmission{pub schema_version:u32,pub goal_id:String,pub decision_id:String,pub source_revision:String,pub source_sequence:u64,pub approved_at:u64,pub descriptor_revision:String,pub evidence_hash:String,pub spec_hash:String}
impl RulePolicy{
 pub(crate)fn goal_spec(&self,index:usize)->Option<model::Spec>{Some(model::Spec{request_id:format!("ro-{}",self.occurrence_id(index)?),owner:self.rule.owner.clone(),context:model::Context::Unbound,intent:self.rule.definition.intent.clone(),criteria:self.rule.definition.criteria.clone(),restored_from:None,received_from:None})}
}
impl RuleGrant{
 pub(crate)fn reference(&self,r:&RuleReservation)->model::Result<OccurrenceReference>{Ok(OccurrenceReference{schema_version:1,rule_id:self.policy.rule.id.clone(),rule_revision:self.policy.rule.revision,rule_decision_id:self.decision_id.clone(),policy_hash:self.policy.hash()?,occurrence_id:r.occurrence_id.clone(),slot_index:r.slot_index,at:r.at,deadline_at:r.deadline_at})}
 pub(crate)fn child_matches(&self,r:&RuleReservation,s:&AdmissionSpec)->bool{
  let Some(reference)=s.rule_occurrence.as_ref()else{return false;};let Some(goal)=self.policy.goal_spec(r.slot_index.into())else{return false;};
  reference.for_spec(s)&&self.reference(r).ok().as_ref()==Some(reference)&&s.owner==self.policy.rule.owner&&s.goal_id==model::id_for(&goal)&&model::fingerprint(&goal).ok().as_ref()==Some(&s.intent_hash)&&s.policy_revision==self.policy.policy_revision&&s.resolved_model==self.policy.model&&s.engine_binding==self.policy.engine_binding&&s.approved_at>=r.reserved_at&&s.budget.max_output_bytes==r.output_bytes&&s.request_id==format!("ra-{}",r.occurrence_id)
 }
}
impl RuleAdmission{
 pub(crate)fn capture(s:&AdmissionSpec)->model::Result<Self>{Ok(Self{schema_version:1,goal_id:s.goal_id.clone(),decision_id:s.decision_id.clone(),source_revision:s.source_revision.clone(),source_sequence:s.source_sequence,approved_at:s.approved_at,descriptor_revision:s.descriptor_revision.clone(),evidence_hash:s.evidence_hash.clone(),spec_hash:model::fingerprint(s)?})}
 pub(crate)fn matches(&self,s:&AdmissionSpec)->bool{Self::capture(s).ok().as_ref()==Some(self)}
 pub(crate)fn valid_for(&self,g:&RuleGrant,r:&RuleReservation)->bool{
  self.schema_version==1&&model::goal_id(&self.goal_id)&&g.policy.goal_spec(r.slot_index.into()).is_some_and(|s|model::id_for(&s)==self.goal_id)&&model::token(&self.decision_id)&&self.decision_id!=g.decision_id&&model::token(&self.source_revision)&&self.source_sequence>0&&self.approved_at>=r.reserved_at&&self.approved_at<r.deadline_at&&self.approved_at<=g.updated_at&&model::text(&self.descriptor_revision,256)&&[&self.evidence_hash,&self.spec_hash].iter().all(|h|super::super::authority::decode(h).is_some())
 }
}
