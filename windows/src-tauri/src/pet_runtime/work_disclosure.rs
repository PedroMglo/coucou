//! Explicit finite future-metadata policy. This is signed admission data, never
//! a model approval or a second authority/scheduler. One native report per task.
use super::{model,messages::{ReceivedReference,Recipient,Target,Envelope},received_work::State,work_report};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Arguments{pub to_pet_id:String,pub subject_goal_id:String,pub subject_spec_hash:String,pub source:ReceivedReference,pub expires_at:u64,pub include_attempts:bool,pub states:Vec<State>}
impl Arguments{
 pub(crate)fn parse(v:&serde_json::Value)->Result<Self,()>{let a:Self=serde_json::from_value(v.clone()).map_err(|_|())?;if !a.valid(){return Err(());}Ok(a)}
 fn valid(&self)->bool{crate::pet_config::blueprint(&self.to_pet_id).is_some()&&model::goal_id(&self.subject_goal_id)&&super::authority::decode(&self.subject_spec_hash).is_some()&&self.source.valid()&&self.expires_at>0&&self.expires_at<=8_640_000_000_000_000&&!self.states.is_empty()&&self.states.len()<=12&&self.states.iter().enumerate().all(|(i,s)|!matches!(s,State::Historical|State::Unavailable)&&!self.states[..i].contains(s))}
 pub(super)fn matches(&self,e:&Envelope)->bool{e.work_report.as_ref().is_some_and(|r|r.valid()&&r.subject_goal_id==self.subject_goal_id&&r.subject_pet_id==e.origin.pet_id&&self.states.contains(&r.state)&&r.observed_at<=e.created_at&&(self.include_attempts||r.attempts.is_none()))}
 #[cfg(target_os="linux")]
 pub(super)fn capture(&self,goals:&std::path::Path,authority:&std::path::Path,pet:&str,now:u64)->model::Result<work_report::Captured>{
  if !self.valid(){return Err(model::Error::InvalidInput);}let store=super::store::Store::existing_verified(goals,authority)?.ok_or(model::Error::NotFound)?;let subject=store.read(&self.subject_goal_id,pet)?;
  if model::fingerprint(&subject.spec)?!=self.subject_spec_hash{return Err(model::Error::Conflict);}
  work_report::Captured::at(goals,authority,&work_report::Arguments{to_pet_id:self.to_pet_id.clone(),subject_goal_id:self.subject_goal_id.clone(),subject_revision:subject.revision,source:self.source.clone(),expires_at:self.expires_at,include_attempts:self.include_attempts},pet,now)
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Policy{pub schema_version:u32,pub plan_id:String,pub step_id:String,pub arguments:Arguments,pub recipient:Recipient,pub proposal_hash:String}
pub use super::disclosure_consent::Consent;
impl Policy{
 pub(super)fn valid(&self)->bool{self.schema_version==1&&model::key(&self.plan_id)&&model::key(&self.step_id)&&self.arguments.valid()&&self.recipient.pet_id==self.arguments.to_pet_id&&!self.recipient.name.is_empty()&&self.recipient.name.chars().count()<=40&&super::authority::decode(&self.recipient.configuration_hash).is_some()&&super::authority::decode(&self.proposal_hash).is_some()}
 pub(super)fn matches(&self,g:&model::Record)->bool{self.valid()&&self.arguments.subject_goal_id!=g.id&&g.plans.len()==1&&g.tasks.len()==1&&g.plans[0].id==self.plan_id&&g.plans[0].steps.len()==1&&g.plans[0].steps[0].id==self.step_id&&g.plans[0].steps[0].operation.as_ref().is_some_and(|o|o.operation_id=="pet.workReport"&&o.version==2&&Arguments::parse(&o.arguments).is_ok_and(|a|a==self.arguments))&&super::plans::fingerprint(g).is_ok_and(|h|h==self.proposal_hash)}
 #[cfg(target_os="linux")]
 pub(super)fn capture(g:&model::Record,goals:&std::path::Path,authority:&std::path::Path,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->model::Result<Self>{
  if g.plans.len()!=1||g.tasks.len()!=1||g.plans[0].steps.len()!=1{return Err(model::Error::InvalidInput);}let p=&g.plans[0];let s=&p.steps[0];let o=s.operation.as_ref().filter(|o|o.operation_id=="pet.workReport"&&o.version==2).ok_or(model::Error::InvalidInput)?;let arguments=Arguments::parse(&o.arguments).map_err(|_|model::Error::InvalidInput)?;
  let _=arguments.capture(goals,authority,&g.spec.owner.pet_id,now)?;let recipient=Target::capture(config,settings,&arguments.to_pet_id)?.recipient().clone();let policy=Self{schema_version:1,plan_id:p.id.clone(),step_id:s.id.clone(),arguments,recipient,proposal_hash:super::plans::fingerprint(g)?};if !policy.matches(g){return Err(model::Error::Conflict);}Ok(policy)
 }
}
#[cfg(target_os="linux")]
pub(super)fn ready(goals:&std::path::Path,authority:&std::path::Path,g:&model::Record,now:u64)->model::Result<bool>{
 let row=g.admissions.last().ok_or(model::Error::Conflict)?;let p=row.spec.work_disclosure.as_ref().filter(|p|p.matches(g)).ok_or(model::Error::Conflict)?;
 let a=super::authority::Authority::existing(authority).map_err(super::authority::Failure::control_error)?;a.verify_owned_origin_for_report(g,now)?;
 let c=p.arguments.capture(goals,authority,&g.spec.owner.pet_id,now)?;Ok(p.arguments.states.contains(&c.report.state))
}

#[cfg(target_os="linux")]
pub(super)fn due(id:&str)->bool{
 use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::{Instant,Duration}};
 static GATE:OnceLock<Mutex<BTreeMap<String,Instant>>>=OnceLock::new();let Ok(mut m)=GATE.get_or_init(||Mutex::new(BTreeMap::new())).lock()else{return false;};let now=Instant::now();m.retain(|_,t|now.duration_since(*t)<Duration::from_secs(1));if m.contains_key(id)||m.len()>=64{return false;}m.insert(id.into(),now);true
}
