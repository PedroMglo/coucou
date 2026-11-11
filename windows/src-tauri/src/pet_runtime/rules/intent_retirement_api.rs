//! Own native review of revoked metadata consolidation, with no worker-end claim.
//! Renderer/provider text cannot create a candidate, policy, budget or approval.
use super::*;
#[cfg(target_os="linux")]use super::super::{authority::Authority,authority_store::RuleIntentRetirementSource,store::nonce};
#[cfg(target_os="linux")]use std::sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}};
#[cfg(target_os="linux")]use std::collections::BTreeMap;
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Consent{ReconcileRevokedRuleMetadataIntentsWithoutNewAuthority}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:serde_json::Value,pub expires_at:u64,pub approval_scope:Consent,pub execution_authority_granted:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Confirm{pub pet_id:String,pub rule_id:String,pub revision:u64,pub token:String,pub approve:bool,pub consent:Option<Consent>}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmation{Declined,Recorded{authority:RuleAuthorityStatus}}
#[cfg(target_os="linux")]
struct Review{window:String,frame:crate::settings::review::Frame,rule:Rule,source:Arc<RuleIntentRetirementSource>,expires:u64,decision:String,choice:Option<bool>,candidate:Option<RuleGrant>,cancelled:Arc<AtomicBool>}
#[cfg(target_os="linux")]fn cache()->&'static Mutex<BTreeMap<String,Arc<Mutex<Review>>>>{static C:OnceLock<Mutex<BTreeMap<String,Arc<Mutex<Review>>>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
#[cfg(target_os="linux")]pub(in crate::pet_runtime)fn expire_reviews(){
 if let Ok(mut c)=cache().try_lock(){let now=crate::assistant_memory::now();c.retain(|_,r|r.try_lock().map(|r|!r.cancelled.load(Ordering::Acquire)&&r.frame.current()&&now<r.expires).unwrap_or(true));}
}
#[cfg(target_os="linux")]pub(super)fn cancel_rule(pet:&str,rule:&str)->Result<(),String>{
 let c=cache().lock().map_err(|_|"ruleIntentRetirementReviewUnavailable")?;for r in c.values(){let r=r.lock().map_err(|_|"ruleIntentRetirementReviewUnavailable")?;if r.rule.id==rule&&r.rule.owner.pet_id==pet{r.cancelled.store(true,Ordering::Release);}}Ok(())
}
#[tauri::command]
pub async fn pet_runtime_rule_intent_retirement_prepare(window:tauri::WebviewWindow,input:PreviewInput)->Result<Ticket,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=input;Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleIntentRetirementReviewClosed")?;let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  crate::pet_data::bootstrap::write_allowed().map_err(|_|"ruleProfileInactive")?;
  let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let decision=a.rule_decision_for(&input.pet_id,&input.rule_id).map_err(|_|"ruleAuthorityUnavailable")?.ok_or("ruleOwnAuthorityRequired")?;
  let source=a.prepare_rule_intent_retirement(&decision).map_err(|e|format!("ruleIntentRetirementSource:{e:?}; requer intenções assinadas e canónico revogado compatíveis"))?;
  if source.rule().id!=rule.id||source.rule().owner.pet_id!=rule.owner.pet_id{return Err("ruleIntentRetirementOwnerChanged".into());}
  if !frame.current()||!read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).is_ok_and(|r|r==rule){return Err("ruleIntentRetirementReviewChanged".into());}
  // Recheck exact native source and profile at the Authority writer boundary.
  // This review does not collect or imply a worker-end witness.
  let proposal=source.preview();let now=crate::assistant_memory::now();let expires=now.checked_add(120_000).ok_or("ruleIntentRetirementReviewUnavailable")?;let token=nonce().map_err(|_|"ruleIntentRetirementReviewUnavailable")?;
  let review=Review{window:label,frame,rule,source:Arc::new(source),expires,decision:nonce().map_err(|_|"ruleIntentRetirementReviewUnavailable")?,choice:None,candidate:None,cancelled:Arc::new(AtomicBool::new(false))};
  expire_reviews();let mut c=cache().lock().map_err(|_|"ruleIntentRetirementReviewUnavailable")?;if c.len()>=32{return Err("ruleIntentRetirementReviewCapacity".into());}c.insert(token.clone(),Arc::new(Mutex::new(review)));
  Ok(Ticket{token,proposal,expires_at:expires,approval_scope:Consent::ReconcileRevokedRuleMetadataIntentsWithoutNewAuthority,execution_authority_granted:false})
 }).await.map_err(|_|"ruleIntentRetirementReviewUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_intent_retirement_confirm(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Confirm)->Result<Confirmation,String>{
 check(&window)?;if input.approve&&input.consent!=Some(Consent::ReconcileRevokedRuleMetadataIntentsWithoutNewAuthority)||!input.approve&&input.consent.is_some(){return Err("ruleIntentRetirementOwnConsentRequired".into());}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let retained=cache().lock().map_err(|_|"ruleIntentRetirementReviewUnavailable")?.get(&input.token).cloned().ok_or("ruleIntentRetirementReviewMissing")?;
  let(source,frame,rule,candidate,cancelled,expires)={let mut r=retained.lock().map_err(|_|"ruleIntentRetirementReviewUnavailable")?;let now=crate::assistant_memory::now();
   if r.window!=label||r.cancelled.load(Ordering::Acquire)||!r.frame.current()||now>=r.expires||r.rule.id!=input.rule_id||r.rule.owner.pet_id!=input.pet_id||r.rule.revision!=input.revision{return Err("ruleIntentRetirementReviewChangedOrExpired".into());}
   if r.choice.is_some_and(|choice|choice!=input.approve){return Err("ruleIntentRetirementOriginalChoiceReceived".into());}r.choice=Some(input.approve);if !input.approve{r.cancelled.store(true,Ordering::Release);return Ok(Confirmation::Declined);}
   if r.candidate.is_none(){r.candidate=Some(r.source.candidate(&r.decision,now).map_err(|_|"ruleIntentRetirementCandidateUnavailable")?);}
   (r.source.clone(),r.frame.clone(),r.rule.clone(),r.candidate.clone().unwrap(),r.cancelled.clone(),r.expires)
  };
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let g=a.retire_rule_intents(&source,&candidate,crate::assistant_memory::now(),||!cancelled.load(Ordering::Acquire)&&crate::assistant_memory::now()<expires&&frame.current()&&read_rule_at(&path(),&rule.owner.pet_id,&rule.id,rule.revision).is_ok_and(|current|current==rule)).map_err(|e|format!("ruleIntentRetirement:{e:?}; consulta os registos; a escolha original mantém-se sem repetir IO"))?;
  Ok(Confirmation::Recorded{authority:a.rule_status(&rule,authority_api::current(&app,&g.policy)).map_err(|_|"ruleIntentRetirementStatusUnavailable")?})
 }).await.map_err(|_|"ruleIntentRetirementInterrupted; consulta os registos antes de repetir".to_string())?}
}
