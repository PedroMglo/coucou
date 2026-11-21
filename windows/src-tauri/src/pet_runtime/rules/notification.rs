//! A reviewed output channel, not another tool/model allowance. The original
//! Authority allocation consumes the attempt BEFORE the desktop effect. A lost
//! response or process can lose a notice, but must never duplicate one.
use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum NotificationConsent{BoundedRuleResultNotification}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NotificationPolicy{
 pub schema_version:u32,pub consent:NotificationConsent,pub channel:String,
 pub content:String,pub priority:String,pub silent:bool,pub max_per_occurrence:u8,
 pub max_delay_millis:u32,
}
impl NotificationPolicy{
 pub(crate)fn bounded()->Self{Self{schema_version:1,consent:NotificationConsent::BoundedRuleResultNotification,channel:"desktopSystem".into(),content:"resultReadyMetadataOnly".into(),priority:"low".into(),silent:true,max_per_occurrence:1,max_delay_millis:300_000}}
 pub(crate)fn valid(&self)->bool{self==&Self::bounded()}
 pub(crate)fn until(&self,r:&RuleReservation)->u64{r.deadline_at.min(r.at.saturating_add(u64::from(self.max_delay_millis)))}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NotificationAttempt{
 pub schema_version:u32,pub id:String,pub receipt_hash:String,pub attempted_at:u64,
 pub finished_at:Option<u64>,pub outcome:Option<NotificationOutcome>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="status",rename_all="camelCase",deny_unknown_fields)]
pub enum NotificationOutcome{AcceptedByDesktop{#[serde(rename="notificationId")]notification_id:u32},FailedOrUncertain,StoppedBeforeSend}
impl NotificationAttempt{
 pub(crate)fn valid(&self,g:&RuleGrant,r:&RuleReservation)->bool{
  let Some(p)=&g.policy.notification else{return false;};
  self.schema_version==1&&model::token(&self.id)&&super::super::authority::decode(&self.receipt_hash).is_some()
   &&r.admission.is_some()&&self.attempted_at>=r.reserved_at&&self.attempted_at>=r.at&&self.attempted_at<p.until(r)&&self.attempted_at<=g.updated_at
   &&match(&self.outcome,self.finished_at){(None,None)=>true,(Some(NotificationOutcome::AcceptedByDesktop{notification_id}),Some(at))=>*notification_id>0&&at>=self.attempted_at&&at<=g.updated_at,(Some(_),Some(at))=>at>=self.attempted_at&&at<=g.updated_at,_=>false}
 }
}

/// Fixed metadata only. No event, rule name, model output, path or credential
/// crosses into the desktop notification service. Acceptance is not proof that
/// a banner was visible (the desktop may enforce its own silence policy).
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)fn desktop()->Result<u32,()>{
 use dbus::{blocking::Connection,arg::{PropMap,Variant}};
 let bus=Connection::new_session().map_err(|_|())?;
 let proxy=bus.with_proxy("org.freedesktop.Notifications","/org/freedesktop/Notifications",std::time::Duration::from_secs(2));
 let mut hints=PropMap::new();hints.insert("urgency".into(),Variant(Box::new(0u8)));hints.insert("suppress-sound".into(),Variant(Box::new(true)));hints.insert("desktop-entry".into(),Variant(Box::new("fr.louisraille.coucou".to_owned())));
 let(id,):(u32,)=proxy.method_call("org.freedesktop.Notifications","Notify",("Coucou",0u32,"coucou","Resposta pronta","A resposta da preparação diária está disponível nos objetivos do pet no Coucou.",Vec::<String>::new(),hints,10_000i32)).map_err(|_|())?;
 if id==0{return Err(());}Ok(id)
}

/// Called by the existing rotating rule scan, including manually admitted
/// rules. No independent watcher, message queue, or replay on startup.
#[cfg(target_os="linux")]
pub(super)fn deliver(app:&tauri::AppHandle,root:&std::path::Path,authority:&std::path::Path,parent:&RuleGrant)->usize{
 use super::super::{authority::Authority,store::Store};
 if parent.policy.notification.is_none()||!authority_api::current(app,&parent.policy){return 0;}
 let Ok(a)=Authority::existing(authority)else{return 0;};let Ok(Some(store))=Store::existing_verified(root,authority)else{return 0;};
 let mut attempted=0;
 for r in &parent.reservations{
  let now=crate::assistant_memory::now();if attempted>=2{break;}
  if r.notification.is_some()||now<r.at||now>=parent.policy.notification.as_ref().unwrap().until(r){continue;}
  let Some(admission)=&r.admission else{continue;};let Ok(goal)=store.read_control(&admission.goal_id,&parent.policy.rule.owner.pet_id)else{continue;};
  // Do not contend for the Authority writer while the original response
  // is still preparing/sending/publishing. Paused/Completed are reached only
  // through the ordinary control/drain path; a timer is not a worker witness.
  if !matches!(goal.state,model::State::Paused|model::State::Completed){continue;}
  let Some(row)=goal.admissions.last().filter(|row|admission.matches(&row.spec))else{continue;};let Some(proof)=&row.proof else{continue;};
  let current=||crate::pet_data::bootstrap::write_allowed().is_ok()&&authority_api::current(app,&parent.policy);
  let Ok(n)=a.begin_rule_notification(&row.spec,proof,now,&current)else{continue;};attempted+=1;
  let outcome=if !current()||a.rule_notification_current(&row.spec,proof,&n,crate::assistant_memory::now()).is_err(){NotificationOutcome::StoppedBeforeSend}else{match desktop(){Ok(notification_id)=>NotificationOutcome::AcceptedByDesktop{notification_id},Err(())=>NotificationOutcome::FailedOrUncertain}};
  // Failure to persist an ACK leaves the original consumed attempt pending.
  // It is never permission to try the desktop effect again.
  let _=a.finish_rule_notification(&parent.decision_id,&r.occurrence_id,&n,outcome,crate::assistant_memory::now());
 }
 attempted
}
