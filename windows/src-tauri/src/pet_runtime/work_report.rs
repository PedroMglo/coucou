//! A minimal native observation, never a sender, grant or current-state promise.
use super::{model,messages::{ReceivedReference,Envelope,Kind},received_work::{State,Attempts}};
use serde::{Serialize,Deserialize};
pub(super)const TEXT:&str="Observação de estado de trabalho; não concede execução ou utilização por modelos.";
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Arguments{pub to_pet_id:String,pub subject_goal_id:String,pub subject_revision:String,pub source:ReceivedReference,pub expires_at:u64,#[serde(default)]pub include_attempts:bool}
impl Arguments{
 pub(crate)fn parse(v:&serde_json::Value)->Result<Self,()>{let a:Self=serde_json::from_value(v.clone()).map_err(|_|())?;
  if crate::pet_config::blueprint(&a.to_pet_id).is_none()||!model::goal_id(&a.subject_goal_id)||!model::token(&a.subject_revision)||!a.source.valid()||a.expires_at==0||a.expires_at>8_640_000_000_000_000{return Err(());}Ok(a)
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Report{pub schema_version:u32,pub subject_pet_id:String,pub subject_goal_id:String,pub subject_revision:String,pub observed_at:u64,pub state:State,pub attempts:Option<Attempts>,pub evidence_complete:bool,pub criteria_verified:bool}
impl Report{
 pub(super)fn valid(&self)->bool{
  self.schema_version==1&&crate::pet_config::blueprint(&self.subject_pet_id).is_some()&&model::goal_id(&self.subject_goal_id)&&model::token(&self.subject_revision)&&self.observed_at>0&&self.observed_at<=8_640_000_000_000_000
   &&self.state!=State::Historical&&(!self.criteria_verified||self.evidence_complete)&&(self.state!=State::Completed||self.criteria_verified)&&self.attempts.as_ref().is_none_or(|a|self.evidence_complete&&a.reserved<=192&&a.started<=192&&a.settled<=192&&a.uncertain<=192&&a.reserved+a.started+a.settled+a.uncertain<=192&&a.unknown_outcomes<=a.settled+a.uncertain)
 }
 pub(super)fn matches(&self,a:&Arguments,e:&Envelope)->bool{self.valid()&&self.subject_pet_id==e.origin.pet_id&&self.subject_goal_id==a.subject_goal_id&&self.subject_revision==a.subject_revision&&self.observed_at<=e.created_at&&(a.include_attempts||self.attempts.is_none())}
}
pub(super)fn matches_parent(report:&Envelope,parent:&Envelope)->bool{
 parent.schema_version==1&&parent.kind==Kind::Request&&parent.reply_to.is_none()&&parent.work_report.is_none()
  &&report.schema_version==3&&report.kind==Kind::WorkEvent&&report.work_report.as_ref().is_some_and(|r|r.valid()&&r.subject_pet_id==report.origin.pet_id&&r.observed_at<=report.created_at)
  &&report.reply_to.as_ref().is_some_and(|r|r.decision_id==parent.origin.decision_id&&r.message_id==parent.message_id)&&report.correlation_id==parent.message_id
  &&report.origin.pet_id==parent.recipient.pet_id&&report.recipient.pet_id==parent.origin.pet_id&&report.created_at>=parent.created_at&&report.deadline_at<=parent.deadline_at&&!report.provider_use_authorized&&!parent.provider_use_authorized
}
#[cfg(target_os="linux")]
pub(super)struct Captured{dir:super::store::Dir,goal_hash:String,source:super::received_context::Captured,pub(super)report:Report}
#[cfg(target_os="linux")]
impl Captured{
 pub(super)fn at(goals:&std::path::Path,authority:&std::path::Path,args:&Arguments,pet:&str,now:u64)->model::Result<Self>{
  let store=super::store::Store::existing_verified(goals,authority)?.ok_or(model::Error::NotFound)?;let goal=store.read(&args.subject_goal_id,pet)?;
  let origin=goal.spec.received_from.as_ref().ok_or(model::Error::Conflict)?;
  if goal.revision!=args.subject_revision||origin.body.source.reference!=args.source||origin.body.source.sender_pet_id!=args.to_pet_id||args.expires_at>origin.body.source.deadline_at||args.expires_at<=now{return Err(model::Error::Conflict);}
  let selection=super::received_context::Selection::from_goal(&goal)?;let source=super::received_context::Captured::at(authority,&goal,&selection,now)?;
  let view=store.received_work(authority,&super::received_work::Input{pet_id:pet.into(),goal_id:goal.id.clone(),revision:goal.revision.clone(),source:args.source.clone()},now)?;
  let report=Report{schema_version:1,subject_pet_id:pet.into(),subject_goal_id:goal.id,subject_revision:goal.revision,observed_at:now,state:view.state,attempts:if args.include_attempts{view.attempts}else{None},evidence_complete:view.evidence_complete,criteria_verified:view.criteria_verified};
  if !report.valid(){return Err(model::Error::Conflict);}let dir=super::store::Dir::open(goals,true)?.ok_or(model::Error::NotFound)?;
  let captured=Self{dir,goal_hash:view.goal_hash,source,report};captured.current(now)?;Ok(captured)
 }
 pub(super)fn current(&self,now:u64)->model::Result<()>{
  self.source.verify(now)?;self.dir.check()?;
  let(raw,_)=self.dir.bytes(&format!("{}.json",self.report.subject_goal_id),model::MAX_BYTES)?.ok_or(model::Error::NotFound)?;
  let goal:model::Record=serde_json::from_slice(&raw).map_err(|_|model::Error::InvalidRecord)?;goal.validate()?;
  if model::fingerprint(&goal)?!=self.goal_hash{return Err(model::Error::Conflict);}Ok(())
 }
}
