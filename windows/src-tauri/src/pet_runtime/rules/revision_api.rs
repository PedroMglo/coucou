//! Own native review of a changed policy OR an explicitly new finite allocation.
//! Stop epoch and native exact source are revalidated at the Authority writer.
use super::*;
#[cfg(target_os="linux")]use super::super::{authority::Authority,authority_store::RuleSuccessionSource,store::nonce};
#[cfg(target_os="linux")]use std::sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}};
#[cfg(target_os="linux")]use std::collections::BTreeMap;
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Consent{ReviseRuleWithinRemainingAllocationAndDeadline,RenewRuleWithReviewedFiniteAllocation}
impl Consent{fn of(mode:RuleRevisionMode)->Self{match mode{RuleRevisionMode::CarryRemaining=>Self::ReviseRuleWithinRemainingAllocationAndDeadline,RuleRevisionMode::RenewFiniteAllocation=>Self::RenewRuleWithReviewedFiniteAllocation}}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Prepare{pub pet_id:String,pub rule_id:String,pub revision:u64,pub mode:RuleRevisionMode,#[serde(default)]pub automatic:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:serde_json::Value,pub expires_at:u64,pub approval_scope:Consent,pub execution_authority_granted:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Confirm{pub mode:RuleRevisionMode,pub pet_id:String,pub rule_id:String,pub revision:u64,pub token:String,pub approve:bool,pub consent:Option<Consent>}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmation{Declined,Recorded{authority:RuleAuthorityStatus}}
#[cfg(target_os="linux")]
struct Review{mode:RuleRevisionMode,epoch:u64,window:String,frame:crate::settings::review::Frame,rule:Rule,source:Arc<RuleSuccessionSource>,expires:u64,decision:String,choice:Option<bool>,candidate:Option<RuleGrant>,cancelled:Arc<AtomicBool>}
#[cfg(target_os="linux")]fn cache()->&'static Mutex<BTreeMap<String,Arc<Mutex<Review>>>>{static C:OnceLock<Mutex<BTreeMap<String,Arc<Mutex<Review>>>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
#[cfg(target_os="linux")]pub(in crate::pet_runtime)fn expire_reviews(){
 if let Ok(mut c)=cache().try_lock(){let now=crate::assistant_memory::now();c.retain(|_,r|r.try_lock().map(|r|!r.cancelled.load(Ordering::Acquire)&&r.frame.current()&&now<r.expires).unwrap_or(true));}
}
#[cfg(target_os="linux")]pub(super)fn cancel_rule(pet:&str,rule:&str)->Result<(),String>{
 let c=cache().lock().map_err(|_|"ruleRevisionReviewUnavailable")?;for r in c.values(){let r=r.lock().map_err(|_|"ruleRevisionReviewUnavailable")?;if r.rule.id==rule&&r.rule.owner.pet_id==pet{r.cancelled.store(true,Ordering::Release);}}Ok(())
}
#[tauri::command]
pub async fn pet_runtime_rule_revision_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Prepare)->Result<Ticket,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleRevisionReviewClosed")?;let label=window.label().to_owned();let epoch=authority_api::capture_epoch(&input.pet_id,&input.rule_id)?;tauri::async_runtime::spawn_blocking(move||{
  crate::pet_data::bootstrap::write_allowed().map_err(|_|"ruleProfileInactive")?;
  let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let policy=authority_api::prepare_policy(&app,rule.clone(),input.automatic,||frame.current()&&authority_api::epoch_current(&input.pet_id,&input.rule_id,epoch)&&read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).is_ok_and(|same|same==rule))?;
  let source=a.prepare_rule_revision(&policy,input.mode,crate::assistant_memory::now()).map_err(|e|format!("ruleRevisionSource:{e:?}; requer autoridade anterior revogada e revisão própria de política/orçamento"))?;
  if source.rule()!=&rule||!frame.current()||!authority_api::epoch_current(&input.pet_id,&input.rule_id,epoch)||!authority_api::current(&app,source.policy()){return Err("ruleRevisionReviewChanged".into());}
  let proposal=source.revision_preview();let now=crate::assistant_memory::now();let expires=now.checked_add(120_000).ok_or("ruleRevisionReviewUnavailable")?;let token=nonce().map_err(|_|"ruleRevisionReviewUnavailable")?;
  let review=Review{mode:input.mode,epoch,window:label,frame,rule,source:Arc::new(source),expires,decision:nonce().map_err(|_|"ruleRevisionReviewUnavailable")?,choice:None,candidate:None,cancelled:Arc::new(AtomicBool::new(false))};
  expire_reviews();let mut c=cache().lock().map_err(|_|"ruleRevisionReviewUnavailable")?;if c.len()>=32{return Err("ruleRevisionReviewCapacity".into());}c.insert(token.clone(),Arc::new(Mutex::new(review)));
  Ok(Ticket{token,proposal,expires_at:expires,approval_scope:Consent::of(input.mode),execution_authority_granted:false})
 }).await.map_err(|_|"ruleRevisionReviewUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_revision_confirm(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Confirm)->Result<Confirmation,String>{
 check(&window)?;if input.approve&&input.consent!=Some(Consent::of(input.mode))||!input.approve&&input.consent.is_some(){return Err("ruleRevisionOwnConsentRequired".into());}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let retained=cache().lock().map_err(|_|"ruleRevisionReviewUnavailable")?.get(&input.token).cloned().ok_or("ruleRevisionReviewMissing")?;
  let(source,frame,rule,candidate,cancelled,expires,epoch)={let mut r=retained.lock().map_err(|_|"ruleRevisionReviewUnavailable")?;let now=crate::assistant_memory::now();
   if r.mode!=input.mode||r.window!=label||r.cancelled.load(Ordering::Acquire)||!r.frame.current()||now>=r.expires||r.rule.id!=input.rule_id||r.rule.owner.pet_id!=input.pet_id||r.rule.revision!=input.revision||!authority_api::epoch_current(&input.pet_id,&input.rule_id,r.epoch){return Err("ruleRevisionReviewChangedOrExpired".into());}
   if r.choice.is_some_and(|choice|choice!=input.approve){return Err("ruleRevisionOriginalChoiceReceived".into());}r.choice=Some(input.approve);if !input.approve{r.cancelled.store(true,Ordering::Release);return Ok(Confirmation::Declined);}
   if r.candidate.is_none(){r.candidate=Some(r.source.candidate(&r.decision,now).map_err(|_|"ruleRevisionCandidateUnavailable")?);}
   (r.source.clone(),r.frame.clone(),r.rule.clone(),r.candidate.clone().unwrap(),r.cancelled.clone(),r.expires,r.epoch)
  };
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let g=a.succeed_rule(&source,&candidate,crate::assistant_memory::now(),||!cancelled.load(Ordering::Acquire)&&crate::assistant_memory::now()<expires&&authority_api::epoch_current(&rule.owner.pet_id,&rule.id,epoch)&&authority_api::current(&app,source.policy())&&frame.current()&&read_rule_at(&path(),&rule.owner.pet_id,&rule.id,rule.revision).is_ok_and(|current|current==rule)).map_err(|e|format!("ruleRevision:{e:?}; consulta os registos; a escolha original mantém-se sem repetir IO"))?;
  Ok(Confirmation::Recorded{authority:a.rule_status(&rule,authority_api::current(&app,&g.policy)).map_err(|_|"ruleRevisionStatusUnavailable")?})
 }).await.map_err(|_|"ruleRevisionInterrupted; consulta os registos antes de repetir".to_string())?}
}
