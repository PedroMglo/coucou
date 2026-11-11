//! Typed records in the COMMON Authority catalogue. No GoalId/message fiction.
use super::*;
use crate::pet_memory::context_model::Destination;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RulePolicy{#[serde(default,skip_serializing_if="Option::is_none")]pub dispatch:Option<DispatchPolicy>,pub schema_version:u32,pub scope:RuleScope,pub rule:Rule,pub payment:super::super::admission::Payment,pub destination:Destination,pub model:String,pub engine_binding:Option<crate::local_chat::lmstudio::Binding>,pub policy_revision:String,pub descriptor_semantics:String,pub schedule:Preview,pub prepared_at:u64,pub valid_until:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum RuleScope{BoundedRuleRecurrenceWithoutSelectedData,BoundedAutomaticRuleRecurrenceWithoutSelectedData}
impl RulePolicy{
 pub(crate)fn valid(&self)->bool{
  if !matches!((self.schema_version,self.scope,&self.dispatch),(1,RuleScope::BoundedRuleRecurrenceWithoutSelectedData,None)|(2,RuleScope::BoundedAutomaticRuleRecurrenceWithoutSelectedData,Some(_)))||self.dispatch.as_ref().is_some_and(|d|!d.valid())||self.payment!=super::super::admission::Payment::LocalOnly||self.rule.owner.adapter=="ollama"&&(self.model.ends_with(":cloud")||self.model.ends_with("-cloud"))||!valid_rule(&self.rule)||self.rule.definition.schedule!=self.schedule.schedule||!crate::pet_calendar::schedule::valid_preview_shape(&self.schedule)||!super::super::admission::model_id(&self.model)||self.prepared_at<self.rule.updated_at||self.prepared_at>=self.valid_until||self.valid_until>8_640_000_000_000_000||super::super::authority::decode(&self.policy_revision).is_none()||super::super::authority::decode(&self.descriptor_semantics).is_none(){return false;}
  let Destination::Server{provider,endpoint,model}=&self.destination else{return false;};
  if !self.destination.valid()||provider!=&self.rule.owner.adapter||model!=&self.model||!reqwest::Url::parse(endpoint).is_ok_and(|u|u.host_str().is_some_and(|h|h=="localhost"||h.trim_matches(['[',']']).parse::<std::net::IpAddr>().is_ok_and(|i|i.is_loopback()))){return false;}
  if self.rule.owner.adapter=="lmstudio"{if self.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&self.model)){return false;}}else if self.engine_binding.is_some(){return false;}
  self.schedule.slots.iter().filter_map(|s|s.at).max().and_then(|at|at.checked_add(u64::from(self.rule.definition.limits.occurrence_timeout_seconds)*1000))==Some(self.valid_until)
 }
 pub(crate)fn hash(&self)->model::Result<String>{model::fingerprint(self)}
 pub(crate)fn occurrence_id(&self,index:usize)->Option<String>{let slot=self.schedule.slots.get(index)?;let at=slot.at?;model::fingerprint(&("coucou-rule-occurrence-v1",&self.rule.id,self.rule.revision,&slot.local_date_time,at)).ok()}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum RulePhase{Active,Revoked}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum RuleKind{RuleRecurrence}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleReservation{#[serde(default,skip_serializing_if="Vec::is_empty")]pub readiness:Vec<ReadinessProbe>,#[serde(default,skip_serializing_if="Option::is_none")]pub admission:Option<RuleAdmission>,pub occurrence_id:String,pub slot_index:u8,pub at:u64,pub deadline_at:u64,pub reserved_at:u64,pub model_calls:u16,pub tool_calls:u16,pub output_bytes:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleGrant{pub schema_version:u32,pub kind:RuleKind,pub decision_id:String,pub revision:String,pub policy:RulePolicy,pub phase:RulePhase,pub approved_at:u64,pub updated_at:u64,pub revoked_at:Option<u64>,pub reservations:Vec<RuleReservation>,#[serde(default,skip_serializing_if="Vec::is_empty")]pub intent_retirements:Vec<RuleIntentRetirement>,#[serde(default,skip_serializing_if="Option::is_none")]pub succession:Option<RuleSuccession>,#[serde(default,skip_serializing_if="Option::is_none")]pub revision_authority:Option<RuleRevisionAuthority>}
impl RuleGrant{
 pub(crate)fn required_version(&self)->u32{if self.revision_authority.is_some(){7}else if self.succession.is_some(){6}else if !self.intent_retirements.is_empty(){5}else if self.reservations.iter().any(|r|r.readiness.iter().any(|p|p.owner.is_some()||p.recovery.is_some())){4}else if self.policy.dispatch.is_some(){3}else if self.reservations.iter().any(|r|r.admission.is_some()){2}else{1}}
 pub(crate)fn valid(&self)->bool{
  if self.schema_version!=self.required_version()||!model::token(&self.decision_id)||!model::token(&self.revision)||!self.policy.valid()||self.approved_at<self.policy.prepared_at||self.approved_at>=self.policy.valid_until||self.updated_at<self.approved_at||self.updated_at>8_640_000_000_000_000||match(self.phase,self.revoked_at){(RulePhase::Active,None)=>false,(RulePhase::Revoked,Some(at))=>at<self.approved_at||at>self.updated_at,_=>true}{return false;}
  if self.revision_authority.as_ref().is_some_and(|r|!r.valid(self))||self.revision_authority.is_some()&&self.succession.is_some(){return false;}
  if self.succession.as_ref().is_some_and(|s|!s.valid(&self.decision_id)){return false;}
  if !self.valid_intent_retirements(){return false;}
  let mut ids=std::collections::BTreeSet::new();let l=&self.policy.rule.definition.limits;
  if self.reservations.len()>usize::from(l.max_occurrences){return false;}
  self.reservations.iter().all(|r|ids.insert(&r.occurrence_id)&&self.revision_authority.as_ref().is_none_or(|a|r.at>a.future_after)&&self.policy.occurrence_id(r.slot_index.into()).as_deref()==Some(r.occurrence_id.as_str())&&self.policy.schedule.slots[usize::from(r.slot_index)].at==Some(r.at)&&r.at.checked_add(u64::from(l.occurrence_timeout_seconds)*1000)==Some(r.deadline_at)&&r.reserved_at>=self.approved_at&&r.reserved_at>=r.at&&r.reserved_at<r.deadline_at&&r.reserved_at<=self.updated_at&&r.deadline_at<=self.policy.valid_until&&r.model_calls==1&&r.tool_calls==0&&r.output_bytes==l.occurrence_output_bytes&&r.admission.as_ref().is_none_or(|a|a.valid_for(self,r))&&self.probes_valid(r))
 }
}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleEnvelope{pub body:RuleGrant,pub tag:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RuleAuthorityStatus{#[serde(skip_serializing_if="Option::is_none")]pub lineage:Option<RuleLineageObservation>,pub observed_at:u64,#[serde(skip_serializing_if="Option::is_none")]pub automatic_status:Option<AutomaticObservation>,pub schema_version:u32,pub rule_id:String,pub decision_id:Option<String>,pub phase:&'static str,pub policy:Option<RulePolicy>,pub committed_occurrences:usize,pub remaining_occurrences:Option<u16>,pub committed_model_calls:u16,pub committed_output_bytes:u64,pub allocations:Vec<RuleReservation>,pub pending_artifacts:usize,pub snapshot_only:bool,pub execution_available:bool,pub dispatch_available:bool}
impl RuleAuthorityStatus{
 pub(crate)fn missing(rule:&Rule)->Self{Self{lineage:None,observed_at:crate::assistant_memory::now(),automatic_status:None,schema_version:1,rule_id:rule.id.clone(),decision_id:None,phase:"notAuthorized",policy:None,committed_occurrences:0,remaining_occurrences:None,committed_model_calls:0,committed_output_bytes:0,allocations:vec![],pending_artifacts:0,snapshot_only:false,execution_available:false,dispatch_available:false}}
 pub(crate)fn of(grant:&RuleGrant,current:bool,historical:bool)->Self{Self::of_at(grant,current,historical,crate::assistant_memory::now())}
 pub(crate)fn of_at(grant:&RuleGrant,current:bool,historical:bool,now:u64)->Self{let clock_uncertain=now==0||now>8_640_000_000_000_000||now<grant.updated_at;let expired=now>=grant.policy.valid_until;let n=grant.reservations.len()as u16;Self{lineage:None,observed_at:now,automatic_status:None,schema_version:1,rule_id:grant.policy.rule.id.clone(),decision_id:Some(grant.decision_id.clone()),phase:if historical{"restoredSnapshotOnly"}else if grant.phase==RulePhase::Revoked{"revoked"}else if clock_uncertain{"authorityTimeUncertain"}else if expired{"expired"}else if current&&grant.reservations.iter().any(|r|r.readiness.iter().any(|p|p.finished_at.is_none())){"readinessPendingOrUncertain"}else if current&&grant.policy.dispatch.is_some(){"authorizedScheduled"}else if current{"authorizedNotDispatched"}else{"configurationChanged"},policy:Some(grant.policy.clone()),committed_occurrences:usize::from(n),remaining_occurrences:(!historical).then_some(grant.policy.rule.definition.limits.max_occurrences-n),committed_model_calls:n,committed_output_bytes:grant.reservations.iter().map(|r|r.output_bytes).sum(),allocations:grant.reservations.clone(),pending_artifacts:0,snapshot_only:historical,execution_available:false,dispatch_available:current&&!clock_uncertain&&!expired&&!historical&&grant.phase==RulePhase::Active&&grant.policy.dispatch.is_some()&&!grant.reservations.iter().any(|r|r.readiness.iter().any(|p|p.finished_at.is_none()))}}
}
impl RuleGrant{
 fn probes_valid(&self,r:&RuleReservation)->bool{
  let Some(policy)=&self.policy.dispatch else{return r.readiness.is_empty();};let mut ids=std::collections::BTreeSet::new();
  r.readiness.len()<=usize::from(policy.max_metadata_probes)&&r.readiness.iter().enumerate().all(|(i,p)|ids.insert(&p.id)&&p.valid(self,r)&&p.prepared_at<=self.updated_at&&(i==0||r.readiness[i-1].finished_at.is_some_and(|at|p.prepared_at>=at.saturating_add(if r.readiness[i-1].observed==Some(false){u64::from(policy.retry_delay_millis)}else{0}))))
 }
}
