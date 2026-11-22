//! An own finite future-calendar DATA policy, not a materialized request body.
use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum CalendarConsent{BoundedRuleFutureCalendarData}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CalendarOrigin{pub id:String,pub revision:u64,pub name:String,pub path:String,pub enrollment_binding:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CalendarPolicy{pub schema_version:u32,pub reader_revision:String,pub consent:CalendarConsent,pub sources:Vec<CalendarOrigin>,pub data_class:String,pub window:String,pub max_data_bytes:u32,pub max_input_bytes:u32,pub max_capture_seconds:u32,pub max_reads_per_occurrence:u16}
impl CalendarPolicy{
 pub(super)fn valid_for(&self,rule:&Rule)->bool{
  let DataScope::Calendars{sources}=&rule.definition.data_scope else{return false;};
  self.schema_version==1&&super::super::authority::decode(&self.reader_revision).is_some()&&self.consent==CalendarConsent::BoundedRuleFutureCalendarData&&self.data_class=="nativeCalendarEventsV1"&&self.window=="occurrenceCivilDay"&&self.max_data_bytes==65536&&self.max_input_bytes==262144&&self.max_capture_seconds==10&&self.max_reads_per_occurrence==1&&!sources.is_empty()&&sources.len()<=8&&sources.len()==self.sources.len()
   &&sources.iter().zip(&self.sources).all(|(a,b)|a.id==b.id&&a.revision==b.revision&&model::text(&b.name,160)&&b.path.len()<=4096&&std::path::Path::new(&b.path).is_absolute()&&!b.path.chars().any(char::is_control)&&super::super::authority::decode(&b.enrollment_binding).is_some())
 }
#[cfg(target_os="linux")]
 pub(in crate::pet_runtime)fn matches_context(&self,b:&model::ContextBinding)->bool{
  b.schema_version==11&&b.memory.is_none()&&b.received_request.is_none()&&b.source_conversation_id.is_none()&&b.calendars.as_ref().is_some_and(|c|c.selection.matches(&crate::pet_calendar::goal::Request{source_scope:Some(crate::pet_sources::PetScope::Pet),selected:self.sources.iter().map(|s|crate::pet_calendar::goal::Pick{id:s.id.clone(),revision:s.revision}).collect(),query:c.selection.query.clone()}))
 }
 #[cfg(target_os="linux")]
 pub(super)fn capture(rule:&Rule)->Result<Self,String>{
  let sources=rule.definition.data_scope.capture(&rule.owner.provider)?.into_iter().map(|s|CalendarOrigin{id:s.id().into(),revision:s.revision(),name:s.name().into(),path:s.path().into(),enrollment_binding:s.binding()}).collect();
  let policy=Self{schema_version:1,reader_revision:reader_revision(),consent:CalendarConsent::BoundedRuleFutureCalendarData,sources,data_class:"nativeCalendarEventsV1".into(),window:"occurrenceCivilDay".into(),max_data_bytes:65536,max_input_bytes:262144,max_capture_seconds:10,max_reads_per_occurrence:1};
  if !policy.valid_for(rule){return Err("ruleCalendarDataPolicyUnsupported".into());}Ok(policy)
 }
 #[cfg(target_os="linux")]
 pub(super)fn current(&self,rule:&Rule)->bool{Self::capture(rule).is_ok_and(|current|current==*self)}
}

fn reader_revision()->String{crate::pet_memory::context_model::digest(include_bytes!("../../pet_calendar/runtime-manifest.json"))}

/// One bounded multicalendar extraction, recorded in the original occurrence.
/// The serializable owner is an observation; only its retained native FD may
/// perform or finish the read. Neither expiry nor this record permits a retry.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CalendarRead{
 pub schema_version:u32,pub id:String,pub scope_hash:String,
 pub owner:super::super::runtime_owner::Operation,
 pub prepared_at:u64,pub expires_at:u64,pub finished_at:Option<u64>,pub outcome:Option<CalendarReadOutcome>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub enum CalendarReadOutcome{Captured{goal_id:String,context_hash:String},Stopped}
impl CalendarRead{
 pub(crate)fn scope(policy:&RulePolicy,r:&RuleReservation)->model::Result<String>{
  if !policy.selected_data(){return Err(model::Error::InvalidInput);}
  model::fingerprint(&(if policy.has_progress_data(){"coucou-rule-project-progress-read-v1"}else if policy.project_data.is_some(){"coucou-rule-project-read-v1"}else{"coucou-rule-calendar-read-v1"},policy.hash()?,&r.occurrence_id,r.slot_index,r.at,r.deadline_at))
 }
 pub(crate)fn valid_for(&self,g:&RuleGrant,r:&RuleReservation)->bool{
  let Some(seconds)=g.policy.capture_seconds()else{return false;};
  self.schema_version==1&&model::token(&self.id)&&Self::scope(&g.policy,r).ok().as_ref()==Some(&self.scope_hash)
   &&self.owner.valid(&self.id,&self.scope_hash,self.prepared_at)&&self.prepared_at>=r.reserved_at&&self.prepared_at<r.deadline_at
   &&self.expires_at==self.prepared_at.saturating_add(u64::from(seconds)*1000).min(r.deadline_at)
   &&self.prepared_at<=g.updated_at&&match(self.finished_at,&self.outcome){
    (None,None)=>true,
    (Some(at),Some(outcome))if at>=self.prepared_at&&at<=g.updated_at=>match outcome{
     CalendarReadOutcome::Stopped=>true,
     CalendarReadOutcome::Captured{goal_id,context_hash}=>at<self.expires_at&&g.policy.goal_spec(r.slot_index.into()).is_some_and(|s|model::id_for(&s)==*goal_id)&&super::super::authority::decode(context_hash).is_some(),
    },_=>false,
   }
 }
}
