//! Exact DATA contract for the existing native admission/Authority; never a permit.
use super::{admission::{AdmissionSpec,Payment},model::{self,Record},plans::OperationProposal,messages::Cause};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Incarnation{pub core_cookie:u32,pub node_id:u32,pub node_serial:String}
impl Incarnation{fn valid(&self)->bool{self.node_id>0&&!self.node_serial.is_empty()&&self.node_serial.len()<=24&&self.node_serial.bytes().all(|b|b.is_ascii_digit())}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Output{pub pet_id:String,pub grant_revision:String,pub output_key:String,pub incarnation:Incarnation}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Step{pub cause:Cause,pub operation:OperationProposal}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Policy{pub schema_version:u32,pub goal_id:String,pub owner:model::Owner,pub source_revision:String,pub source_sequence:u64,pub spec_hash:String,pub context_hash:String,pub proposal_hash:String,pub prepared_at:u64,pub expires_at:u64,#[serde(default,skip_serializing_if="Option::is_none")]pub output:Option<Output>,#[serde(default,skip_serializing_if="Option::is_none")]pub health:Option<crate::pet_health::task::Binding>,#[serde(default,skip_serializing_if="Option::is_none")]pub inventory:Option<crate::pet_host_inventory::Binding>,pub steps:Vec<Step>}
fn hash(s:&str)->bool{super::authority::decode(s).is_some()}
fn action(step:&Step)->model::Result<crate::pet_audio::task_action::Action>{crate::pet_audio::task_action::Action::parse(&step.operation.operation_id,step.operation.version,&step.operation.arguments).map_err(|_|model::Error::InvalidInput)}
impl Policy{
 pub(super)fn semantic_hash(&self)->model::Result<String>{let mut p=self.clone();p.prepared_at=0;model::fingerprint(&p)}
 pub(super)fn valid(&self)->bool{
  matches!((&self.output,&self.health,&self.inventory,self.schema_version),(Some(_),None,None,1)|(None,Some(_),None,2)|(None,None,Some(_),3))&&model::goal_id(&self.goal_id)&&self.owner.valid()&&model::token(&self.source_revision)&&self.source_sequence>0&&[&self.spec_hash,&self.context_hash,&self.proposal_hash].iter().all(|h|hash(h))&&self.prepared_at>0&&self.prepared_at<self.expires_at&&self.expires_at<=8_640_000_000_000_000
   &&self.output.as_ref().is_none_or(|o|o.pet_id==self.owner.pet_id&&crate::pet_inference::revision_key(&o.grant_revision)&&crate::pet_config::system_access::output_key(&o.output_key)&&o.incarnation.valid())
   &&self.health.as_ref().is_none_or(|h|h.valid()&&h.pet_id==self.owner.pet_id&&self.steps.len()==1)&&self.inventory.as_ref().is_none_or(|i|i.valid()&&i.pet_id==self.owner.pet_id&&self.steps.len()==1)&&(1..=64).contains(&self.steps.len())
   &&self.steps.iter().enumerate().all(|(i,s)|model::key(&s.cause.task_id)&&model::key(&s.cause.plan_id)&&model::key(&s.cause.step_id)&&s.cause.task_id==self.steps[0].cause.task_id&&s.cause.plan_id==self.steps[0].cause.plan_id&&!self.steps[..i].iter().any(|p|p.cause.step_id==s.cause.step_id)&&match &self.output{Some(o)=>action(s).is_ok_and(|a|a.key()==o.output_key),None=>if self.inventory.is_some(){crate::pet_host_inventory::validate(&s.operation.operation_id,s.operation.version,&s.operation.arguments)}else{crate::pet_health::task::validate(&s.operation.operation_id,s.operation.version,&s.operation.arguments)}})&&serde_json::to_vec(self).is_ok_and(|v|v.len()<=64*1024)
 }
 fn criteria_match(&self,s:&AdmissionSpec)->bool{
  use super::criteria::Predicate;
  // Preserve already-issued Health contracts; the new Inventory path has one exact predicate.
  let expected=self.inventory.as_ref().map(|_|Predicate::SystemHostInventoryObserved);
  expected.is_none_or(|p|s.native_verification.as_ref().is_some_and(|v|v.schema_version==7&&v.criteria_count==1&&v.conditions.len()==1&&v.conditions[0].predicate==p))
 }
 pub(super)fn for_spec(&self,s:&AdmissionSpec)->bool{
  matches!((self.schema_version,s.schema_version),(1,27)|(2|3,28))&&self.valid()&&self.criteria_match(s)&&self.goal_id==s.goal_id&&self.owner==s.owner&&self.source_revision==s.source_revision&&self.source_sequence==s.source_sequence&&self.spec_hash==s.intent_hash&&self.context_hash==s.context_hash&&s.proposal_hash.as_ref()==Some(&self.proposal_hash)&&self.prepared_at<=s.approved_at&&s.approved_at<self.expires_at&&self.expires_at==s.budget.deadline_at&&self.steps.len()<=s.budget.max_tool_calls as usize&&s.budget.payment==Payment::RuntimeOnly&&s.budget.max_model_calls==0
 }
 pub(super)fn matches(&self,g:&Record,s:&AdmissionSpec)->bool{
  if !self.for_spec(s)||g.spec.received_from.is_some()||g.id!=self.goal_id||g.spec.owner!=self.owner||g.plans.len()!=1||model::fingerprint(&g.spec).ok().as_ref()!=Some(&self.spec_hash)||g.execution_context.as_ref().and_then(|c|model::fingerprint(c).ok()).as_ref()!=Some(&self.context_hash)||super::plans::fingerprint(g).ok().as_ref()!=Some(&self.proposal_hash){return false;}
  let p=&g.plans[0];p.steps.len()==self.steps.len()&&p.steps.iter().zip(&self.steps).all(|(s,a)|a.cause.task_id==p.task_id&&a.cause.plan_id==p.id&&a.cause.step_id==s.id&&s.operation.as_ref()==Some(&a.operation))
 }
 pub(super)fn targets_current(&self,c:&crate::pet_config::Configuration)->bool{
  use crate::pet_config::system_access::Operation;
  c.pets.get(&self.owner.pet_id).and_then(|p|p.system_access.as_ref()).is_some_and(|p|match (&self.output,&self.health){
   (Some(o),None)=>p.revision==o.grant_revision&&p.permits(Operation::Observe,Some(&o.output_key))&&self.steps.iter().all(|s|action(s).is_ok_and(|a|p.permits(a.operation(),Some(a.key())))),
   (None,Some(h))=>p.revision==h.grant_revision&&p.permits_health(),(None,None)=>self.inventory.as_ref().is_some_and(|i|p.revision==i.grant_revision&&p.permits_inventory()),_=>false})
 }
 #[cfg(target_os="linux")]pub(super)fn binding_matches(&self,b:&super::physical_target::Binding)->bool{match(&self.output,&self.health){
  (Some(o),None)=>b.sound().is_some_and(|b|b.pet_id==o.pet_id&&b.grant_revision==o.grant_revision&&b.output_key==o.output_key),
  (None,Some(h))=>b.health()==Some(h),(None,None)=>self.inventory.as_ref().is_some_and(|i|b.inventory()==Some(i)),_=>false}}
 #[cfg(target_os="linux")]pub(super)fn target_matches(&self,t:&super::physical_target::Target)->bool{self.binding_matches(t.binding())&&self.output.as_ref().is_none_or(|o|t.native_incarnation().as_ref()==Some(&o.incarnation))}

}
#[cfg(target_os="linux")]pub(super)type Observe=std::sync::Arc<dyn Fn()->model::Result<crate::pet_config::Configuration>+Send+Sync>;
#[cfg(target_os="linux")]pub(super)struct Captured{policy:Policy,target:super::physical_target::Target,observe:Observe,retired:std::sync::atomic::AtomicBool}
#[cfg(target_os="linux")]impl Captured{
 pub(super)fn at(g:&Record,s:&AdmissionSpec,target:super::physical_target::Target,observe:Observe,now:u64)->model::Result<Self>{
  let output=if let Some(b)=target.binding().sound(){Some(Output{pet_id:b.pet_id.clone(),grant_revision:b.grant_revision.clone(),output_key:b.output_key.clone(),incarnation:target.native_incarnation().ok_or(model::Error::InvalidInput)?})}else{None};
  let health=target.binding().health().cloned();let inventory=target.binding().inventory().cloned();if output.is_none()&&health.is_none()&&inventory.is_none(){return Err(model::Error::InvalidInput);}
  let mut steps=vec![];for p in &g.plans{for step in &p.steps{let op=step.operation.clone().ok_or(model::Error::InvalidInput)?;if !target.permits(&super::physical_target::Action::parse(&op)?){return Err(model::Error::Conflict);}steps.push(Step{cause:Cause{task_id:p.task_id.clone(),plan_id:p.id.clone(),step_id:step.id.clone()},operation:op});}}
  let policy=Policy{schema_version:if inventory.is_some(){3}else if health.is_some(){2}else{1},goal_id:g.id.clone(),owner:g.spec.owner.clone(),source_revision:s.source_revision.clone(),source_sequence:s.source_sequence,spec_hash:s.intent_hash.clone(),context_hash:s.context_hash.clone(),proposal_hash:s.proposal_hash.clone().ok_or(model::Error::Conflict)?,prepared_at:now,expires_at:s.budget.deadline_at,output,health,inventory,steps};
  let c=Self{policy,target,observe,retired:std::sync::atomic::AtomicBool::new(false)};c.current(g,s,now)?;Ok(c)
 }
 pub(super)fn policy(&self)->&Policy{&self.policy}
 pub(super)fn current(&self,g:&Record,s:&AdmissionSpec,now:u64)->model::Result<()>{
  use std::sync::atomic::Ordering;
  if self.retired.load(Ordering::Acquire){return Err(model::Error::Conflict);}
  let result=(||{if now<self.policy.prepared_at||now>=self.policy.expires_at||!self.policy.matches(g,s)||!self.policy.target_matches(&self.target)||!self.policy.targets_current(&(self.observe)()?){return Err(model::Error::Conflict);}if !self.target.identity_current(&||true){return Err(model::Error::Conflict);}Ok(())})();
  if result.is_err(){self.retired.store(true,Ordering::Release);}result
 }
}
