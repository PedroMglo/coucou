//! Direct message DATA. Deserialization never creates a sender, human review or
//! execution permit. Canonical entries live in the existing signed Grant.
use super::{model,admission::AdmissionSpec,leases::Lease,authority::Grant,effect_budget::{Kind as AttemptKind,Phase as AttemptPhase}};
use serde::{Deserialize,Serialize};
use std::sync::atomic::{AtomicBool,Ordering};
pub(super)const MAX_BODY:usize=2048;
pub(super)const MAX_MESSAGES:usize=16;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Kind{Message,Request,Reply,WorkEvent,Delegation}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Arguments{pub to_pet_id:String,pub kind:Kind,pub text:String,pub expires_at:u64}
pub(in crate::pet_runtime)fn body(s:&str)->bool{!s.trim().is_empty()&&s.len()<=MAX_BODY&&!s.chars().any(|c|c.is_control()&&!matches!(c,'\n'|'\t'))}
fn hash(s:&str)->bool{super::authority::decode(s).is_some()}
fn known(pet:&str)->bool{crate::pet_config::blueprint(pet).is_some()}
impl Arguments{
 pub(crate)fn parse(value:&serde_json::Value)->std::result::Result<Self,()>{let a:Self=serde_json::from_value(value.clone()).map_err(|_|())?;if !a.valid(){return Err(());}Ok(a)}
 fn valid(&self)->bool{matches!(self.kind,Kind::Message|Kind::Request)&&known(&self.to_pet_id)&&body(&self.text)&&self.expires_at>0&&self.expires_at<=8_640_000_000_000_000}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Origin{pub pet_id:String,pub provider:String,pub adapter:String,pub conversation_id:String,pub goal_id:String,pub decision_id:String,pub lease_id:String,pub controller_id:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Recipient{pub pet_id:String,pub name:String,pub configuration_hash:String}
/// Metadata captured by the native runtime, not a provider/model grant.
pub(in crate::pet_runtime)struct Target{recipient:Recipient,retired:AtomicBool}
impl Target{
 pub(in crate::pet_runtime)fn capture(config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,pet:&str)->model::Result<Self>{
  let p=config.pets.get(pet).filter(|_|crate::pet_config::blueprint(pet).is_some_and(|b|!b.retired)).ok_or(model::Error::Conflict)?;
  if settings.main_pill!=pet&&!settings.active_integrations.iter().any(|id|id==pet){return Err(model::Error::Conflict);}
  crate::pet_config::validate(config).map_err(|_|model::Error::InvalidInput)?;
  Ok(Self{recipient:Recipient{pet_id:pet.into(),name:p.name.clone(),configuration_hash:model::fingerprint(p)?},retired:AtomicBool::new(false)})
 }
 pub(in crate::pet_runtime)fn current(&self,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings)->bool{
  if self.retired.load(Ordering::Acquire){return false;}
  if Self::capture(config,settings,&self.recipient.pet_id).is_ok_and(|t|t.recipient==self.recipient){true}else{self.retired.store(true,Ordering::Release);false}
 }
 pub(in crate::pet_runtime)fn recipient(&self)->&Recipient{&self.recipient}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Cause{pub task_id:String,pub plan_id:String,pub step_id:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Envelope{
 #[serde(default,skip_serializing_if="Option::is_none")]pub planned_data:Option<super::planned_peer_data::Published>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub derived_result:Option<super::result_disclosure_policy::Published>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub delegation:Option<super::delegation::Payload>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub work_report:Option<super::work_report::Report>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub reply_to:Option<ReceivedReference>,
 pub schema_version:u32,pub message_id:String,pub origin:Origin,pub recipient:Recipient,
 pub correlation_id:String,pub cause:Cause,pub kind:Kind,pub text:String,pub created_at:u64,pub deadline_at:u64,
 /// Receiving data never approves forwarding it into a provider context.
 pub provider_use_authorized:bool,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Scope{#[serde(default,skip_serializing_if="Option::is_none")]pub generated_plan:Option<super::generated_plans::Reference>,pub envelope:Envelope,pub request_id:String,pub goal_revision:String,pub main_spec_hash:String,pub proposal_hash:String}
impl Scope{
 pub(super)fn valid(&self)->bool{
  let e=&self.envelope;let o=&e.origin;
  (e.planned_data.is_some()==matches!(e.schema_version,10|11|12|13|14|15|16))&&e.planned_data.as_ref().is_none_or(|p|p.matches_envelope(e)&&self.generated_plan.as_ref()==Some(&p.generated_plan))
   &&self.generated_plan.as_ref().is_none_or(|r|r.valid()&&r.plan_id==e.cause.plan_id&&r.task_id==e.cause.task_id&&r.proposal_hash==self.proposal_hash)
   &&(matches!(e.schema_version,6|9)==e.derived_result.is_some())&&e.derived_result.as_ref().is_none_or(|d|d.valid()&&d.schema_version==(if e.schema_version==9{2}else{1}))&&(matches!(e.schema_version,4|7|12|14|16)==e.delegation.is_some())&&(match(e.schema_version,&e.reply_to,e.kind){
   (7,Some(r),Kind::Delegation)=>e.work_report.is_none()&&r.valid()&&r.message_id!=e.message_id&&r.decision_id!=o.decision_id&&model::token(&e.correlation_id)&&e.delegation.as_ref().is_some_and(|p|p.valid_for(e)),
   (8|9,Some(r),Kind::Reply)=>e.work_report.is_none()&&r.valid()&&r.message_id!=e.message_id&&r.decision_id!=o.decision_id&&model::token(&e.correlation_id),
   (4|12|14|16,None,Kind::Delegation)=>e.work_report.is_none()&&e.correlation_id==e.message_id&&e.delegation.as_ref().is_some_and(|p|p.valid_for(e)),
   (1|10|11|13|15,None,Kind::Message|Kind::Request)=>e.work_report.is_none()&&e.correlation_id==e.message_id,
   (2|5|6,Some(r),Kind::Reply)=>e.work_report.is_none()&&r.valid()&&r.message_id!=e.message_id&&r.decision_id!=o.decision_id&&e.correlation_id==r.message_id,
   (3,Some(r),Kind::WorkEvent)=>r.valid()&&r.message_id!=e.message_id&&r.decision_id!=o.decision_id&&e.correlation_id==r.message_id&&e.text==super::work_report::TEXT&&e.work_report.as_ref().is_some_and(|w|w.valid()&&w.subject_pet_id==o.pet_id&&w.observed_at<=e.created_at),
   _=>false,
  })&&model::token(&e.message_id)&&known(&o.pet_id)&&known(&e.recipient.pet_id)&&o.pet_id!=e.recipient.pet_id
   &&crate::pet_config::blueprint(&o.pet_id).is_some_and(|b|b.recognizes_owner(&o.provider,&o.adapter))&&crate::assistant_memory::valid_id(&o.conversation_id)
   &&model::goal_id(&o.goal_id)&&model::token(&o.decision_id)&&model::token(&o.lease_id)&&model::token(&o.controller_id)
   &&model::key(&e.cause.task_id)&&model::key(&e.cause.plan_id)&&model::key(&e.cause.step_id)&&hash(&e.recipient.configuration_hash)
   &&!e.recipient.name.is_empty()&&e.recipient.name.chars().count()<=40&&body(&e.text)&&e.created_at>0&&e.created_at<e.deadline_at&&e.deadline_at<=8_640_000_000_000_000
   &&!e.provider_use_authorized&&model::key(&self.request_id)&&model::token(&self.goal_revision)&&hash(&self.main_spec_hash)&&hash(&self.proposal_hash)
 }
 pub(super)fn belongs(&self,spec:&AdmissionSpec,lease:&Lease)->bool{
  if self.envelope.planned_data.as_ref().is_some_and(|p|!p.for_spec(spec)||p.round_data.as_ref().is_some_and(|d|!d.for_spec(spec,lease)))||(if spec.peer_selected_data(){self.envelope.planned_data.as_ref().is_none_or(|p|p.schema_version!=if spec.peer_received_iteration(){4}else if spec.peer_memory_iteration(){3}else{1})}else{self.envelope.planned_data.is_some()&&!spec.peer_iteration()}){return false;}
  let e=&self.envelope;let o=&e.origin;
  self.valid()&&(matches!(spec.schema_version,2|4|5|6|7|8|9|10|11|14|16|17|18|20)||spec.peer_planning())&&spec.binding_format()&&(spec.planning_policy.is_none()||spec.peer_planning())&&lease.valid(spec)&&self.main_spec_hash==model::fingerprint(spec).unwrap_or_default()
   &&(if spec.peer_planning(){self.generated_plan.as_ref().is_some_and(|r|r.decision_id==spec.decision_id&&r.source.matches(spec,lease)&&r.proposal_hash==self.proposal_hash)}else{self.generated_plan.is_none()&&spec.proposal_hash.as_ref()==Some(&self.proposal_hash)})&&o.pet_id==spec.owner.pet_id&&o.provider==lease.provider&&o.adapter==spec.owner.adapter
   &&o.goal_id==lease.goal_id&&o.decision_id==lease.decision_id&&o.lease_id==lease.id&&o.controller_id==lease.controller_id&&o.conversation_id==lease.conversation_id
   &&(if e.schema_version==7{spec.schema_version==17&&spec.funding.as_ref().is_some_and(|f|f.schema_version==5&&e.reply_to.as_ref()==Some(&f.source.reference))}else if e.schema_version==9{spec.schema_version==20&&spec.funding.as_ref().is_some_and(|f|f.schema_version==8&&e.reply_to.as_ref()==Some(&f.source.reference))&&spec.result_disclosure.as_ref().is_some_and(|p|p.matches_envelope(e))}else if e.schema_version==8{spec.schema_version==18&&spec.funding.as_ref().is_some_and(|f|f.schema_version==6&&e.reply_to.as_ref()==Some(&f.source.reference))}else{true})
   &&e.created_at>=lease.claimed_at&&e.deadline_at<=lease.deadline_at
 }
 pub(super)fn matches(&self,goal:&model::Record)->bool{
  let e=&self.envelope;
  self.matches_proposal(goal)&&goal.revision==self.goal_revision
 }
 pub(super)fn matches_proposal(&self,goal:&model::Record)->bool{
  self.matches_snapshot(goal,&super::plans::snapshot(goal))
 }
 pub(super)fn matches_snapshot(&self,goal:&model::Record,snapshot:&super::plans::Snapshot)->bool{
  model::fingerprint(snapshot).ok().as_ref()==Some(&self.proposal_hash)&&self.input_matches(goal,snapshot)
 }
 // Coverage of an already authenticated receipt, never new execution authority.
 pub(super)fn input_matches(&self,goal:&model::Record,snapshot:&super::plans::Snapshot)->bool{
  let e=&self.envelope;
  self.valid()&&goal.id==e.origin.goal_id&&goal.spec.owner.pet_id==e.origin.pet_id
   &&snapshot.input(&e.cause).and_then(|s|s.operation)
    .is_some_and(|op|Input::parse(&op).is_ok_and(|a|a.matches(e)))
 }
}
/// An address of received DATA. It authenticates nothing until resolved against
/// this profile's canonical signed Grant and the caller's own native admission.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReceivedReference{pub decision_id:String,pub message_id:String,pub scope_hash:String}
impl ReceivedReference{
 pub(super)fn valid(&self)->bool{model::token(&self.decision_id)&&model::token(&self.message_id)&&hash(&self.scope_hash)}
 pub(super)fn from_receipt(r:&Receipt)->model::Result<Self>{Ok(Self{decision_id:r.envelope().origin.decision_id.clone(),message_id:r.envelope().message_id.clone(),scope_hash:model::fingerprint(&r.scope)?})}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Stage{Received}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Receipt{pub(super)scope:Scope,pub stage:Stage,pub received_at:u64,pub attempt_id:String}
impl Receipt{pub fn envelope(&self)->&Envelope{&self.scope.envelope}}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Ledger{pub schema_version:u32,pub entries:Vec<Receipt>,#[serde(default,skip_serializing_if="Vec::is_empty")]pub retired:Vec<super::message_retirement::Reference>}
impl Ledger{
 pub(super)fn empty()->Self{Self{schema_version:1,entries:vec![],retired:vec![]}}
 pub(super)fn required_schema(&self)->u32{if self.entries.iter().any(|r|r.envelope().schema_version==16)||self.retired.iter().any(|r|r.scope.envelope.schema_version==16){return 16;}if self.entries.iter().any(|r|r.envelope().schema_version==15)||self.retired.iter().any(|r|r.scope.envelope.schema_version==15){return 15;}if self.entries.iter().any(|r|r.envelope().schema_version==14)||self.retired.iter().any(|r|r.scope.envelope.schema_version==14){return 14;}if self.entries.iter().any(|r|r.envelope().schema_version==13)||self.retired.iter().any(|r|r.scope.envelope.schema_version==13){return 13;}if self.entries.iter().any(|r|r.envelope().schema_version==12)||self.retired.iter().any(|r|r.scope.envelope.schema_version==12){12}else if self.entries.iter().any(|r|r.envelope().schema_version==11)||self.retired.iter().any(|r|r.scope.envelope.schema_version==11){11}else if self.entries.iter().any(|r|r.envelope().schema_version==10)||self.retired.iter().any(|r|r.scope.envelope.schema_version==10){10}else if self.entries.iter().any(|r|r.envelope().schema_version==9)||self.retired.iter().any(|r|r.scope.envelope.schema_version==9){9}else if self.entries.iter().any(|r|r.envelope().schema_version==8)||self.retired.iter().any(|r|r.scope.envelope.schema_version==8){8}else if self.entries.iter().any(|r|r.envelope().schema_version==7)||self.retired.iter().any(|r|r.scope.envelope.schema_version==7){7}else if self.entries.iter().any(|r|r.envelope().schema_version==6)||self.retired.iter().any(|r|r.scope.envelope.schema_version==6){6}else if self.entries.iter().any(|r|r.envelope().schema_version==5)||self.retired.iter().any(|r|r.scope.envelope.schema_version==5){5}else if self.entries.iter().any(|r|r.envelope().schema_version==4)||self.retired.iter().any(|r|r.scope.envelope.schema_version==4){4}else if self.entries.iter().any(|r|r.envelope().schema_version==3)||self.retired.iter().any(|r|r.scope.envelope.schema_version==3){3}else if self.entries.iter().any(|r|r.envelope().schema_version==2)||self.retired.iter().any(|r|r.scope.envelope.schema_version==2){2}else{1}}
 pub(super)fn refresh_schema(&mut self){self.schema_version=self.required_schema();}
 pub(super)fn valid_for(&self,grant:&Grant)->bool{
  let Some(lease)=grant.lease.as_ref()else{return false;};let budget=grant.budget_usage.as_ref();
  let mut ids=std::collections::BTreeSet::new();let mut slots=std::collections::BTreeSet::new();let mut requests=std::collections::BTreeSet::new();
  self.schema_version==self.required_schema()&&(!self.entries.is_empty()||!self.retired.is_empty())&&self.entries.len()+self.retired.len()<=MAX_MESSAGES
   &&self.retired.iter().all(|r|r.valid_for(grant)&&ids.insert(&r.scope.envelope.message_id)&&!self.entries.iter().any(|e|e.envelope().message_id==r.scope.envelope.message_id))
   &&self.entries.iter().all(|r|{
   let s=&r.scope;let e=&s.envelope;let o=&e.origin;
   s.valid()&&(s.generated_plan.is_some()==grant.peer_planning)&&(matches!(e.schema_version,6|9)==grant.result_disclosure.is_some())&&grant.result_disclosure.as_ref().is_none_or(|p|p.matches_envelope(e))&&s.main_spec_hash==grant.spec_hash&&o.goal_id==grant.goal_id&&o.decision_id==grant.decision_id&&o.lease_id==lease.id&&o.controller_id==lease.controller_id&&o.provider==lease.provider&&o.conversation_id==lease.conversation_id
    &&e.created_at>=lease.claimed_at&&e.deadline_at<=lease.deadline_at&&r.received_at>=e.created_at&&r.received_at<e.deadline_at&&r.received_at<=grant.updated_at
    &&ids.insert(&e.message_id)&&slots.insert((&e.cause.plan_id,&e.cause.step_id))&&requests.insert(&s.request_id)
    &&budget.is_some_and(|budget|budget.attempts.iter().any(|a|a.id==r.attempt_id&&a.kind==AttemptKind::Tool&&a.phase==AttemptPhase::Settled&&a.input_hash==model::fingerprint(s).unwrap_or_default()
      &&a.request_id==format!("message-{}",e.message_id)&&a.reserved_at==r.received_at&&a.started_at==Some(r.received_at)&&a.finished_at==Some(r.received_at)
      &&serde_json::to_vec(e).is_ok_and(|bytes|a.output_reserved==bytes.len()as u64&&a.accepted_output==Some(bytes.len()as u64))))
  })
 }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct InboxItem{#[serde(skip_serializing_if="Option::is_none")]pub result_source:Option<ReceivedReference>,#[serde(skip_serializing_if="Option::is_none")]pub funding_source:Option<ReceivedReference>,#[serde(skip_serializing_if="Option::is_none")]pub reply_source:Option<ReceivedReference>,pub receipt:Receipt,pub available:bool,#[serde(skip_serializing_if="Option::is_none")]pub historical_source:Option<HistoricalSource>}
#[derive(Clone,Debug,serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct HistoricalSource{pub schema_version:u32,pub history_hash:String,pub source_profile_fingerprint:String,pub source_file_hash:String,pub snapshot_only:bool,pub post_snapshot_state_unknown:bool,pub execution_authorized:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Inbox{pub schema_version:u32,pub pet_id:String,pub items:Vec<InboxItem>,pub unavailable:usize,pub pending_artifacts:usize,pub scan_complete:bool,pub has_more:bool,pub next_grant:Option<String>,pub execution_authorized:bool}

#[path="message_input.rs"]mod input;
pub(crate)use input::ReplyArguments;
pub(super)use input::Input;
/// Only a depth-one answer to original received DATA; no recursive delegation.
pub(super)fn reply_matches(reply:&Envelope,parent:&Envelope)->bool{
 (parent.schema_version==1&&matches!(parent.kind,Kind::Message|Kind::Request)&&reply.schema_version==2||matches!(parent.schema_version,4|12|14|16)&&parent.kind==Kind::Delegation&&matches!(reply.schema_version,5|6)||parent.schema_version==7&&parent.kind==Kind::Delegation&&matches!(reply.schema_version,8|9))
 &&(if parent.schema_version==7{parent.reply_to.is_some()}else{parent.reply_to.is_none()})&&reply.kind==Kind::Reply&&reply.reply_to.as_ref().is_some_and(|r|r.decision_id==parent.origin.decision_id&&r.message_id==parent.message_id)
 &&reply.origin.pet_id==parent.recipient.pet_id&&reply.recipient.pet_id==parent.origin.pet_id
 &&reply.correlation_id==parent.correlation_id&&reply.created_at>=parent.created_at&&reply.deadline_at<=parent.deadline_at
 &&!reply.provider_use_authorized&&!parent.provider_use_authorized
}
