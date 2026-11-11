//! Exact local peer-plan DATA policy in the existing signed admission.
//! No inference, new ledger, shell authority or recipient execution consent.
use super::{admission::{AdmissionSpec,Payment},model::{self,Record},messages::{self,Recipient,ReceivedReference,Input},plans::OperationProposal,received_request::Source,work_report::Report};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Step{pub cause:messages::Cause,pub operation:OperationProposal,pub recipient:Recipient,#[serde(default,skip_serializing_if="Option::is_none")]pub work_report:Option<Report>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Policy{pub schema_version:u32,pub goal_id:String,pub owner:super::model::Owner,pub source_revision:String,pub source_sequence:u64,pub spec_hash:String,pub context_hash:String,pub proposal_hash:String,pub prepared_at:u64,pub expires_at:u64,pub consent:super::disclosure_consent::Consent,pub steps:Vec<Step>,pub sources:Vec<Source>}
fn hash(s:&str)->bool{super::authority::decode(s).is_some()}
fn input(step:&Step)->model::Result<Input>{Input::parse(&step.operation).map_err(|_|model::Error::InvalidInput)}
pub(super)fn family(s:&AdmissionSpec)->bool{matches!(s.schema_version,4|5|6|7|8|10|11|16|17|18)&&s.deterministic()&&s.work_disclosure.is_none()&&s.result_disclosure.is_none()&&s.memory_data.is_none()&&!s.context.has_selected_data()&&s.engine_binding.is_none()&&s.resolved_model.is_empty()&&s.budget.payment==Payment::RuntimeOnly}
pub(super)fn covered(s:&AdmissionSpec,plan:&str,step:&str)->bool{s.continuation.as_ref().and_then(|c|c.native.as_ref()).is_some_and(|n|n.covers(plan,step))}
fn supported(i:&Input)->bool{!matches!(i,Input::ResultReply(_)|Input::WorkDisclosure(_))}
fn same_report(a:&Report,b:&Report)->bool{let mut normalized=a.clone();normalized.observed_at=b.observed_at;normalized==*b}
impl Policy{
 pub(super)fn semantic_hash(&self)->model::Result<String>{let mut p=self.clone();p.prepared_at=0;for step in &mut p.steps{if let Some(r)=&mut step.work_report{r.observed_at=0;}}model::fingerprint(&p)}
 pub(super)fn targets_current(&self,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings)->bool{self.steps.iter().all(|s|messages::Target::capture(config,settings,&s.recipient.pet_id).is_ok_and(|t|t.recipient()==&s.recipient))}
 #[cfg(target_os="linux")]pub(super)fn sources_current(&self,g:&Record,authority:&std::path::Path,now:u64)->model::Result<()>{
  let refs=self.sources.iter().map(|s|s.reference.clone()).collect::<Vec<_>>();if Captured::sources(authority,g,&refs,now)?!=self.sources{return Err(model::Error::Conflict);}Ok(())
 }
 pub(super)fn valid(&self)->bool{
  self.schema_version==1&&model::goal_id(&self.goal_id)&&self.owner.valid()&&model::token(&self.source_revision)&&self.source_sequence>0
   &&hash(&self.spec_hash)&&hash(&self.context_hash)&&hash(&self.proposal_hash)&&self.prepared_at>0&&self.prepared_at<self.expires_at&&self.expires_at<=8_640_000_000_000_000&&self.consent==super::disclosure_consent::Consent::BoundedNativePeerPlan
   &&self.steps.len()<=messages::MAX_MESSAGES&&self.sources.len()<=messages::MAX_MESSAGES+1&&self.sources.iter().enumerate().all(|(n,s)|s.valid()&&s.recipient_pet_id==self.owner.pet_id&&!self.sources[..n].iter().any(|old|old.reference==s.reference))
   &&self.steps.iter().enumerate().all(|(n,s)|model::key(&s.cause.task_id)&&model::key(&s.cause.plan_id)&&model::key(&s.cause.step_id)&&!self.steps[..n].iter().any(|old|old.cause.plan_id==s.cause.plan_id&&old.cause.step_id==s.cause.step_id)&&hash(&s.recipient.configuration_hash)&&!s.recipient.name.is_empty()&&s.recipient.name.chars().count()<=40&&input(s).is_ok_and(|i|supported(&i)&&i.to_pet()==s.recipient.pet_id&&i.to_pet()!=self.owner.pet_id&&i.deadline()>self.prepared_at&&i.deadline()<=self.expires_at&&match i{Input::WorkReport(a)=>s.work_report.as_ref().is_some_and(|r|r.valid()&&r.subject_pet_id==self.owner.pet_id&&r.subject_goal_id==a.subject_goal_id&&r.subject_revision==a.subject_revision&&r.observed_at==self.prepared_at&&(a.include_attempts||r.attempts.is_none())),_=>s.work_report.is_none()}))
   &&serde_json::to_vec(self).is_ok_and(|v|v.len()<=64*1024)
 }
 pub(super)fn for_spec(&self,s:&AdmissionSpec)->bool{
  family(s)&&self.valid()&&self.goal_id==s.goal_id&&self.owner==s.owner&&self.source_revision==s.source_revision&&self.source_sequence==s.source_sequence&&self.spec_hash==s.intent_hash&&self.context_hash==s.context_hash&&s.proposal_hash.as_ref()==Some(&self.proposal_hash)&&self.prepared_at<=s.approved_at&&s.approved_at<self.expires_at&&self.expires_at==s.budget.deadline_at&&self.steps.len()<=s.budget.max_tool_calls as usize&&(self.steps.len()>0||s.continuation.is_some()&&s.native_verification.is_some())
 }
 pub(super)fn matches(&self,g:&Record,s:&AdmissionSpec)->bool{
  if !self.for_spec(s)||g.id!=self.goal_id||g.spec.owner!=self.owner||model::fingerprint(&g.spec).ok().as_ref()!=Some(&self.spec_hash)||g.execution_context.as_ref().and_then(|c|model::fingerprint(c).ok()).as_ref()!=Some(&self.context_hash)||super::plans::fingerprint(g).ok().as_ref()!=Some(&self.proposal_hash){return false;}
  let mut n=0;for p in &g.plans{for step in &p.steps{if covered(s,&p.id,&step.id){continue;}let Some(allowed)=self.steps.get(n)else{return false;};if allowed.cause.task_id!=p.task_id||allowed.cause.plan_id!=p.id||allowed.cause.step_id!=step.id||step.operation.as_ref()!=Some(&allowed.operation){return false;}n+=1;}}n==self.steps.len()
 }
 pub(super)fn matches_envelope(&self,s:&AdmissionSpec,g:&Record,e:&messages::Envelope)->bool{
  self.matches(g,s)&&e.origin.pet_id==self.owner.pet_id&&e.origin.goal_id==self.goal_id&&!e.provider_use_authorized&&self.steps.iter().any(|a|a.cause==e.cause&&a.recipient==e.recipient&&input(a).is_ok_and(|i|i.matches(e))&&match(&a.work_report,&e.work_report){(None,None)=>true,(Some(old),Some(now))=>same_report(old,now)&&now.observed_at>=self.prepared_at&&now.observed_at<=e.created_at,_=>false})
 }
}
#[cfg(target_os="linux")]pub(super)type Observe=std::sync::Arc<dyn Fn()->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>+Send+Sync>;
#[cfg(target_os="linux")]pub(super)struct Captured{policy:Policy,authority:std::path::PathBuf,targets:Vec<std::sync::Arc<messages::Target>>,work:Vec<super::work_report::Captured>,observe:Observe,retired:std::sync::atomic::AtomicBool}
#[cfg(target_os="linux")]impl Captured{
 pub(super)fn at(goals:&std::path::Path,authority:&std::path::Path,g:&Record,s:&AdmissionSpec,observe:Observe,now:u64)->model::Result<Self>{
  if !family(s)||!super::deterministic::plan(g){return Err(model::Error::InvalidInput);}let(config,settings)=observe()?;let mut targets=vec![];let mut work=vec![];let mut steps=vec![];let mut references=vec![];
  if let Some(source)=g.spec.received_from.as_ref().map(|o|o.body.source.reference.clone()){references.push(source);}
  for p in &g.plans{for step in &p.steps{
   if covered(s,&p.id,&step.id){continue;}let op=step.operation.as_ref().ok_or(model::Error::InvalidInput)?;let i=Input::parse(op).map_err(|_|model::Error::InvalidInput)?;
   if !supported(&i){return Err(model::Error::UnsupportedPlatform);}if i.deadline()<=now||i.deadline()>s.budget.deadline_at{return Err(model::Error::Conflict);}
   let target=std::sync::Arc::new(messages::Target::capture(&config,&settings,i.to_pet())?);
   if let Some(r)=i.source(){if !references.contains(r){references.push(r.clone());}}
   let report=if let Input::WorkReport(a)=&i{if a.subject_goal_id==g.id{return Err(model::Error::Conflict);}let c=super::work_report::Captured::at(goals,authority,a,&g.spec.owner.pet_id,now)?;let report=c.report.clone();work.push(c);Some(report)}else{None};
   steps.push(Step{cause:messages::Cause{task_id:p.task_id.clone(),plan_id:p.id.clone(),step_id:step.id.clone()},operation:op.clone(),recipient:target.recipient().clone(),work_report:report});targets.push(target);
  }}
  let sources=Self::sources(authority,g,&references,now)?;let policy=Policy{schema_version:1,goal_id:g.id.clone(),owner:g.spec.owner.clone(),source_revision:s.source_revision.clone(),source_sequence:s.source_sequence,spec_hash:s.intent_hash.clone(),context_hash:s.context_hash.clone(),proposal_hash:s.proposal_hash.clone().ok_or(model::Error::Conflict)?,prepared_at:now,expires_at:s.budget.deadline_at,consent:super::disclosure_consent::Consent::BoundedNativePeerPlan,steps,sources};
  if !policy.matches(g,s){return Err(model::Error::Conflict);}let c=Self{policy,authority:authority.into(),targets,work,observe,retired:std::sync::atomic::AtomicBool::new(false)};c.current(g,s,now)?;Ok(c)
 }
 fn sources(authority:&std::path::Path,g:&Record,refs:&[ReceivedReference],now:u64)->model::Result<Vec<Source>>{
  if refs.is_empty(){return Ok(vec![]);}let a=super::authority::Authority::existing(authority).map_err(super::authority::Failure::control_error)?;refs.iter().map(|r|a.peer_plan_source(g,r,now).map_err(super::authority::Failure::control_error)).collect()
 }
 pub(super)fn policy(&self)->&Policy{&self.policy}
 pub(super)fn current(&self,g:&Record,s:&AdmissionSpec,now:u64)->model::Result<()>{
  use std::sync::atomic::Ordering;
  if self.retired.load(Ordering::Acquire)||now<self.policy.prepared_at||now>=self.policy.expires_at||!self.policy.matches(g,s){self.retired.store(true,Ordering::Release);return Err(model::Error::Conflict);}
  let(config,settings)=(self.observe)()?;if self.targets.iter().any(|t|!t.current(&config,&settings)){self.retired.store(true,Ordering::Release);return Err(model::Error::Conflict);}
  let refs=self.policy.sources.iter().map(|s|s.reference.clone()).collect::<Vec<_>>();let current=Self::sources(&self.authority,g,&refs,now)?;
  if current!=self.policy.sources||self.work.iter().any(|w|w.current(now).is_err()){self.retired.store(true,Ordering::Release);return Err(model::Error::Conflict);}Ok(())
 }
}

#[cfg(all(test,target_os="linux"))]#[path="native_peer_plan_tests.rs"]pub(in crate::pet_runtime)mod tests;
