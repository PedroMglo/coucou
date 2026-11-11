//! Own native review of metadata reconciliation. Renderer/provider text is never
//! an absence proof, recovery candidate, approval, fresh budget or IO request.
use super::*;
#[cfg(target_os="linux")]use super::super::{authority::{Authority,Failure},authority_store::RuleProbeRecoverySource,native_owner::EndedOperation,store::nonce};
#[cfg(target_os="linux")]use std::sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}};
#[cfg(target_os="linux")]use std::collections::BTreeMap;
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Consent{ReconcileInterruptedRuleReadinessWithoutReplayingIo}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:serde_json::Value,pub expires_at:u64,pub approval_scope:Consent,pub execution_authority_granted:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Confirm{pub pet_id:String,pub rule_id:String,pub revision:u64,pub token:String,pub approve:bool,pub consent:Option<Consent>}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmation{Declined,Recorded{authority:RuleAuthorityStatus}}
#[cfg(target_os="linux")]
struct Review{window:String,frame:crate::settings::review::Frame,rule:Rule,source:Arc<RuleProbeRecoverySource>,gate:Arc<EndedOperation>,expires:u64,decision:String,choice:Option<bool>,candidate:Option<RuleGrant>,cancelled:Arc<AtomicBool>}
#[cfg(target_os="linux")]fn cache()->&'static Mutex<BTreeMap<String,Arc<Mutex<Review>>>>{static C:OnceLock<Mutex<BTreeMap<String,Arc<Mutex<Review>>>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
#[cfg(target_os="linux")]pub(in crate::pet_runtime)fn expire_reviews(){
 if let Ok(mut c)=cache().try_lock(){let now=crate::assistant_memory::now();c.retain(|_,r|r.try_lock().map(|r|!r.cancelled.load(Ordering::Acquire)&&r.frame.current()&&now<r.expires).unwrap_or(true));}
}
#[cfg(target_os="linux")]pub(super)fn cancel_rule(pet:&str,rule:&str)->Result<(),String>{
 let c=cache().lock().map_err(|_|"ruleRecoveryReviewUnavailable")?;for r in c.values(){let r=r.lock().map_err(|_|"ruleRecoveryReviewUnavailable")?;if r.rule.id==rule&&r.rule.owner.pet_id==pet{r.cancelled.store(true,Ordering::Release);}}Ok(())
}
#[tauri::command]
pub async fn pet_runtime_rule_probe_recovery_prepare(window:tauri::WebviewWindow,input:PreviewInput)->Result<Ticket,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=input;Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleRecoveryReviewClosed")?;let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  crate::pet_data::bootstrap::write_allowed().map_err(|_|"ruleProfileInactive")?;
  let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let decision=a.rule_decision_for(&input.pet_id,&input.rule_id).map_err(|_|"ruleAuthorityUnavailable")?.ok_or("ruleOwnAuthorityRequired")?;
  let source=a.prepare_rule_probe_recovery(&decision).map_err(|e|format!("ruleRecoverySource:{e:?}; requer sondagem original e dono nativo comprovável"))?;
  if source.rule().id!=rule.id||source.rule().owner.pet_id!=rule.owner.pet_id{return Err("ruleRecoveryOwnerChanged".into());}
  let gate=EndedOperation::capture(&a,source.owner().map_err(|_|"ruleRecoveryOwnerUnknown")?).map_err(|e|format!("ruleRecoveryWorkerEnd:{e:?}; não se presume fim por expiração"))?;
  if !frame.current()||!read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).is_ok_and(|r|r==rule){return Err("ruleRecoveryReviewChanged".into());}
  // The source is retained before native absence. Recheck its exact hashes at
  // confirmation/writer boundaries; no mutable renderer source is accepted.
  let mut proposal=source.preview();proposal["ownerWitness"]=serde_json::json!(gate.witness());let now=crate::assistant_memory::now();let expires=now.checked_add(120_000).ok_or("ruleRecoveryReviewUnavailable")?;let token=nonce().map_err(|_|"ruleRecoveryReviewUnavailable")?;
  let review=Review{window:label,frame,rule,source:Arc::new(source),gate:Arc::new(gate),expires,decision:nonce().map_err(|_|"ruleRecoveryReviewUnavailable")?,choice:None,candidate:None,cancelled:Arc::new(AtomicBool::new(false))};
  expire_reviews();let mut c=cache().lock().map_err(|_|"ruleRecoveryReviewUnavailable")?;if c.len()>=32{return Err("ruleRecoveryReviewCapacity".into());}c.insert(token.clone(),Arc::new(Mutex::new(review)));
  Ok(Ticket{token,proposal,expires_at:expires,approval_scope:Consent::ReconcileInterruptedRuleReadinessWithoutReplayingIo,execution_authority_granted:false})
 }).await.map_err(|_|"ruleRecoveryReviewUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_probe_recovery_confirm(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Confirm)->Result<Confirmation,String>{
 check(&window)?;if input.approve&&input.consent!=Some(Consent::ReconcileInterruptedRuleReadinessWithoutReplayingIo)||!input.approve&&input.consent.is_some(){return Err("ruleRecoveryOwnConsentRequired".into());}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let retained=cache().lock().map_err(|_|"ruleRecoveryReviewUnavailable")?.get(&input.token).cloned().ok_or("ruleRecoveryReviewMissing")?;
  let(source,gate,frame,rule,candidate,cancelled)={let mut r=retained.lock().map_err(|_|"ruleRecoveryReviewUnavailable")?;let now=crate::assistant_memory::now();
   if r.window!=label||r.cancelled.load(Ordering::Acquire)||!r.frame.current()||now>=r.expires||r.rule.id!=input.rule_id||r.rule.owner.pet_id!=input.pet_id||r.rule.revision!=input.revision{return Err("ruleRecoveryReviewChangedOrExpired".into());}
   if r.choice.is_some_and(|choice|choice!=input.approve){return Err("ruleRecoveryOriginalChoiceReceived".into());}r.choice=Some(input.approve);if !input.approve{r.cancelled.store(true,Ordering::Release);return Ok(Confirmation::Declined);}
   if r.candidate.is_none(){r.candidate=Some(r.source.candidate(&r.decision,now).map_err(|_|"ruleRecoveryCandidateUnavailable")?);}
   (r.source.clone(),r.gate.clone(),r.frame.clone(),r.rule.clone(),r.candidate.clone().unwrap(),r.cancelled.clone())
  };
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleAuthorityUnavailable")?;
  let g=a.recover_rule_probe(&source,&candidate,&gate,crate::assistant_memory::now(),||!cancelled.load(Ordering::Acquire)&&frame.current()&&read_rule_at(&path(),&rule.owner.pet_id,&rule.id,rule.revision).is_ok_and(|current|current==rule)).map_err(|e|format!("ruleRecovery:{e:?}; consulta os registos; a escolha original mantém-se sem repetir IO"))?;
  Ok(Confirmation::Recorded{authority:a.rule_status(&rule,authority_api::current(&app,&g.policy)).map_err(|_|"ruleRecoveryStatusUnavailable")?})
 }).await.map_err(|_|"ruleRecoveryInterrupted; consulta os registos antes de repetir".to_string())?}
}
