//! Native-user-only recurrence consent. Renderer text never supplies a permit.
use super::*;
#[cfg(target_os="linux")]use super::super::{authority::{Authority,Failure},store::nonce};
#[cfg(target_os="linux")]use std::sync::{Mutex,OnceLock,Arc,atomic::{AtomicBool,Ordering}};
#[cfg(target_os="linux")]use std::collections::BTreeMap;
#[cfg(target_os="linux")]
#[derive(Clone,PartialEq,Eq)]struct Binding{destination:crate::pet_memory::context_model::Destination,model:String,engine:Option<crate::local_chat::lmstudio::Binding>,policy:String,semantics:String}
#[cfg(target_os="linux")]
fn binding(app:&tauri::AppHandle,rule:&Rule)->Result<Binding,String>{
 let p=crate::pet_capabilities::native_projection(app,None).map_err(|_|"ruleConfigurationUnavailable")?;if !p.preferences_ready{return Err("ruleConfigurationUnavailable".into());}let config=p.configuration.as_ref().ok_or("ruleConfigurationUnavailable")?;
 let d=p.snapshot.pets.iter().find(|d|d.pet_id==rule.owner.pet_id).ok_or("ruleOwnerChanged")?;let owner=Owner::from_descriptor(d).map_err(|_|"ruleOwnerChanged")?;
 if !owner.local_engine()||!owner.same_runtime_binding(&rule.owner){return Err("ruleOwnerChanged".into());}let model=d.binding.model.clone().ok_or("ruleModelNotResolved")?;
 let server=super::super::memory_review::server_for_owner(&owner,config,&p.settings,&model).map_err(|_|"ruleDestinationChanged")?;
 let destination=super::super::memory_review::destination_for(&owner,&server,&model).map_err(|_|"ruleDestinationChanged")?;
 let policy=super::super::native_validity::inference_policy_revision(config,&p.settings,&owner).map_err(|_|"ruleConfigurationChanged")?;
 let semantics=descriptor_binding(d).map_err(|_|"ruleConfigurationChanged")?;
 if owner.adapter=="lmstudio"&&d.binding.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&model)){return Err("ruleModelNotResolved".into());}
 Ok(Binding{destination,model,engine:d.binding.engine_binding.clone(),policy,semantics})
}
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)fn current(app:&tauri::AppHandle,policy:&RulePolicy)->bool{
 if policy.dispatch.is_some()&&policy.rule.owner.adapter!="ollama"{return false;}
 let r=&policy.rule;read_rule_at(&path(),&r.owner.pet_id,&r.id,r.revision).is_ok_and(|same|same==*r)&&crate::pet_calendar::schedule::valid_preview(&policy.schedule)&&binding(app,r).is_ok_and(|b|b.destination==policy.destination&&b.model==policy.model&&b.engine==policy.engine_binding&&b.policy==policy.policy_revision&&b.semantics==policy.descriptor_semantics)
}
#[cfg(target_os="linux")]struct Review{window:String,frame:crate::settings::review::Frame,policy:RulePolicy,expires_at:u64,decision:String,revision:String,choice:Option<bool>,candidate:Option<RuleGrant>,cancelled:Arc<AtomicBool>}
#[cfg(target_os="linux")]fn cache()->&'static Mutex<BTreeMap<String,Review>>{static C:OnceLock<Mutex<BTreeMap<String,Review>>>=OnceLock::new();C.get_or_init(||Mutex::new(BTreeMap::new()))}
#[cfg(target_os="linux")]fn epochs()->&'static Mutex<BTreeMap<(String,String),u64>>{static E:OnceLock<Mutex<BTreeMap<(String,String),u64>>>=OnceLock::new();E.get_or_init(||Mutex::new(BTreeMap::new()))}
#[cfg(target_os="linux")]fn cancel_reviews(pet:&str,rule:&str)->Result<(),String>{super::revision_api::cancel_rule(pet,rule)?;super::succession_api::cancel_rule(pet,rule)?;super::intent_retirement_api::cancel_rule(pet,rule)?;super::recovery_api::cancel_rule(pet,rule)?;let mut e=epochs().lock().map_err(|_|"ruleReviewUnavailable")?;if let Some(n)=e.get_mut(&(pet.into(),rule.into())){*n=n.checked_add(1).ok_or("ruleReviewCapacity")?;}let c=cache().lock().map_err(|_|"ruleReviewUnavailable")?;for r in c.values().filter(|r|r.policy.rule.id==rule&&r.policy.rule.owner.pet_id==pet){r.cancelled.store(true,Ordering::Release);}Ok(())}
#[cfg(target_os="linux")]pub(super)fn capture_epoch(pet:&str,rule:&str)->Result<u64,String>{let mut e=epochs().lock().map_err(|_|"ruleReviewUnavailable")?;if e.len()>=4096&&!e.contains_key(&(pet.into(),rule.into())){return Err("ruleReviewCapacity".into());}Ok(*e.entry((pet.into(),rule.into())).or_insert(0))}
#[cfg(target_os="linux")]pub(super)fn epoch_current(pet:&str,rule:&str,expected:u64)->bool{epochs().lock().is_ok_and(|e|e.get(&(pet.into(),rule.into())).copied()==Some(expected))}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:RulePolicy,pub expires_at:u64,pub approval_scope:RuleScope,pub dispatch_available:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Confirm{pub pet_id:String,pub rule_id:String,pub revision:u64,pub token:String,pub approve:bool,pub consent:Option<RuleScope>}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmation{Declined,Recorded{authority:RuleAuthorityStatus}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Revoke{pub pet_id:String,pub rule_id:String,pub decision_id:Option<String>}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Prepare{pub pet_id:String,pub rule_id:String,pub revision:u64,#[serde(default)]pub automatic:bool}
#[cfg(target_os="linux")]pub(super)fn prepare_policy(app:&tauri::AppHandle,rule:Rule,automatic:bool,fresh:impl Fn()->bool)->Result<RulePolicy,String>{
 let captured=binding(&app,&rule)?;if automatic&&rule.owner.adapter!="ollama"{return Err("automaticRuleRequiresSupportedOllamaAdapter".into());}
  let schedule=crate::pet_calendar::schedule::resolve(&rule.definition.schedule,&fresh).map_err(error)?;
  let now=crate::assistant_memory::now();let valid_until=schedule.slots.iter().filter_map(|s|s.at).max().and_then(|n|n.checked_add(u64::from(rule.definition.limits.occurrence_timeout_seconds)*1000)).ok_or("ruleHasNoResolvedOccurrences")?;
  let policy=RulePolicy{dispatch:automatic.then(DispatchPolicy::bounded),schema_version:if automatic{2}else{1},scope:if automatic{RuleScope::BoundedAutomaticRuleRecurrenceWithoutSelectedData}else{RuleScope::BoundedRuleRecurrenceWithoutSelectedData},rule,payment:crate::pet_runtime::admission::Payment::LocalOnly,destination:captured.destination,model:captured.model,engine_binding:captured.engine,policy_revision:captured.policy,descriptor_semantics:captured.semantics,schedule,prepared_at:now,valid_until};
 if !policy.valid()||!fresh()||!current(app,&policy){return Err("ruleReviewChangedOrExpired".into());}Ok(policy)
}
#[tauri::command]
pub async fn pet_runtime_rule_authority_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Prepare)->Result<Ticket,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{if !valid_id(&input.rule_id)||!crate::pet_config::defaults().pets.contains_key(&input.pet_id){return Err("invalidRuleIdentity".into());}let epoch={let mut e=epochs().lock().map_err(|_|"ruleReviewUnavailable")?;if e.len()>=4096&&!e.contains_key(&(input.pet_id.clone(),input.rule_id.clone())){return Err("ruleReviewCapacity".into());}*e.entry((input.pet_id.clone(),input.rule_id.clone())).or_insert(0)};let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleReviewClosed")?;let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
  let policy=prepare_policy(&app,rule.clone(),input.automatic,||frame.current()&&read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).is_ok_and(|r|r==rule))?;let now=crate::assistant_memory::now();
  let token=nonce().map_err(|_|"ruleReviewUnavailable")?;let expires=now.checked_add(120_000).ok_or("ruleReviewUnavailable")?;
  let review=Review{window:label,frame,policy:policy.clone(),expires_at:expires,decision:nonce().map_err(|_|"ruleReviewUnavailable")?,revision:nonce().map_err(|_|"ruleReviewUnavailable")?,choice:None,candidate:None,cancelled:Arc::new(AtomicBool::new(false))};
  let epochs=epochs().lock().map_err(|_|"ruleReviewUnavailable")?;if epochs.get(&(input.pet_id.clone(),input.rule_id.clone())).copied()!=Some(epoch){return Err("ruleReviewCancelled".into());}let mut c=cache().lock().map_err(|_|"ruleReviewUnavailable")?;c.retain(|_,r|r.frame.current()&&r.expires_at>now);if c.len()>=32{return Err("ruleReviewCapacity".into());}c.insert(token.clone(),review);
  let scope=policy.scope;Ok(Ticket{token,proposal:policy,expires_at:expires,approval_scope:scope,dispatch_available:false})
 }).await.map_err(|_|"ruleReviewUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_authority_confirm(app:tauri::AppHandle,window:tauri::WebviewWindow,input:Confirm)->Result<Confirmation,String>{
 check(&window)?;if input.approve&&input.consent.is_none()||!input.approve&&input.consent.is_some(){return Err("ruleOwnConsentRequired".into());}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{let now=crate::assistant_memory::now();let label=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let(candidate,frame,cancelled)={let mut c=cache().lock().map_err(|_|"ruleReviewUnavailable")?;let r=c.get_mut(&input.token).ok_or("ruleReviewMissing")?;
   if r.cancelled.load(Ordering::Acquire)||r.window!=label||!r.frame.current()||now>=r.expires_at||r.policy.rule.id!=input.rule_id||r.policy.rule.owner.pet_id!=input.pet_id||r.policy.rule.revision!=input.revision{return Err("ruleReviewChangedOrExpired".into());}
   if r.choice.is_some_and(|choice|choice!=input.approve){return Err("ruleOriginalChoiceAlreadyReceived".into());}
   if !input.approve{r.choice=Some(false);return Ok(Confirmation::Declined);}if input.consent!=Some(r.policy.scope){return Err("ruleOwnConsentRequired".into());}
   if !current(&app,&r.policy){return Err("ruleConfigurationChanged".into());}r.choice=Some(true);
   let value=r.candidate.get_or_insert_with(||RuleGrant{schema_version:if r.policy.dispatch.is_some(){3}else{1},kind:RuleKind::RuleRecurrence,decision_id:r.decision.clone(),revision:r.revision.clone(),policy:r.policy.clone(),phase:RulePhase::Active,approved_at:now,updated_at:now,revoked_at:None,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None});(value.clone(),r.frame.clone(),r.cancelled.clone())
  };
  let a=Authority::ensure(&super::super::admission_review::authority_root()).map_err(|e|format!("ruleAuthority:{e:?}"))?;
  let grant=a.issue_rule(&candidate,crate::assistant_memory::now(),||!cancelled.load(Ordering::Acquire)&&frame.current()&&current(&app,&candidate.policy)).map_err(|e|format!("ruleAuthority:{e:?}"))?;
  Ok(Confirmation::Recorded{authority:RuleAuthorityStatus::of(&grant,current(&app,&grant.policy),false)})
 }).await.map_err(|_|"ruleAuthorityUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_authority_status(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PreviewInput)->Result<RuleAuthorityStatus,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||{let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;match Authority::existing(&super::super::admission_review::authority_root()){
  Err(Failure::Missing)=>Ok(RuleAuthorityStatus::missing(&rule)),Err(e)=>Err(format!("ruleAuthority:{e:?}")),Ok(a)=>{let mut status=a.rule_status(&rule,false).map_err(|e|format!("ruleAuthority:{e:?}"))?;if !status.snapshot_only&&status.pending_artifacts==0&&!matches!(status.phase,"revoked"|"expired"|"authorityTimeUncertain"){if let Some(policy)=&status.policy{let valid=current(&app,policy);status.dispatch_available=valid&&policy.dispatch.is_some()&&!status.allocations.iter().any(|r|r.readiness.iter().any(|p|p.finished_at.is_none()));status.phase=if !valid{"configurationChanged"}else if status.allocations.iter().any(|r|r.readiness.iter().any(|p|p.finished_at.is_none())){"readinessPendingOrUncertain"}else if policy.dispatch.is_some(){"authorizedScheduled"}else{"authorizedNotDispatched"};}status.automatic_status=super::scheduled::observation(&rule.id);}Ok(status)}
 }}).await.map_err(|_|"ruleAuthorityUnavailable".to_string())?}
}
#[tauri::command]
pub async fn pet_runtime_rule_authority_revoke(window:tauri::WebviewWindow,input:Revoke)->Result<RuleAuthorityStatus,String>{
 check(&window)?;if !valid_id(&input.rule_id)||!crate::pet_config::defaults().pets.contains_key(&input.pet_id){return Err("invalidRuleIdentity".into());}
 #[cfg(not(target_os="linux"))]{let _=input;Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||{
  cancel_reviews(&input.pet_id,&input.rule_id)?;
  let a=match Authority::existing(&super::super::admission_review::authority_root()){Err(Failure::Missing)=>return Ok(no_authority(&input.rule_id)),Err(e)=>return Err(format!("ruleAuthority:{e:?}")),Ok(a)=>a};
  let id=match input.decision_id{Some(id)=>Some(id),None=>a.rule_decision_for(&input.pet_id,&input.rule_id).map_err(|e|format!("ruleAuthority:{e:?}"))?};let Some(id)=id else{return Ok(no_authority(&input.rule_id));};
  let grant=a.revoke_rule(&input.pet_id,&input.rule_id,&id,crate::assistant_memory::now()).map_err(|e|format!("ruleAuthority:{e:?}"))?;a.rule_status(&grant.policy.rule,false).map_err(|e|format!("ruleAuthority:{e:?}"))
 }).await.map_err(|_|"ruleAuthorityUnavailable".to_string())?}
}
#[cfg(target_os="linux")]
pub(super)fn before_edit(old:&Rule)->Result<(),&'static str>{
 cancel_reviews(&old.owner.pet_id,&old.id).map_err(|_|"ruleReviewUnavailable")?;
 match Authority::existing(&super::super::admission_review::authority_root()){
  Err(Failure::Missing)=>Ok(()),Err(_)=>Err("ruleAuthorityUnavailable"),Ok(a)=>{let status=a.rule_status(old,false).map_err(|_|"ruleAuthorityUnavailable")?;if !status.snapshot_only{if let Some(id)=a.rule_decision_for(&old.owner.pet_id,&old.id).map_err(|_|"ruleAuthorityUnavailable")?{a.revoke_rule(&old.owner.pet_id,&old.id,&id,crate::assistant_memory::now()).map_err(|_|"ruleAuthorityRevokeRequired")?;}}Ok(())}
 }
}

#[cfg(target_os="linux")]fn no_authority(rule:&str)->RuleAuthorityStatus{RuleAuthorityStatus{lineage:None,observed_at:crate::assistant_memory::now(),automatic_status:None,schema_version:1,rule_id:rule.into(),decision_id:None,phase:"notAuthorized",policy:None,committed_occurrences:0,remaining_occurrences:None,committed_model_calls:0,committed_output_bytes:0,allocations:vec![],pending_artifacts:0,snapshot_only:false,execution_available:false,dispatch_available:false}}


/// A long-lived consent binds configured identity/engine, not transient ready,
/// busy/unknown/freshness projections. Each occurrence still needs fresh admission.
#[cfg(target_os="linux")]
fn descriptor_binding(d:&crate::pet_capabilities::Descriptor)->model::Result<String>{model::fingerprint(&(&d.pet_id,&d.kind,&d.lifecycle,&d.binding))}
#[cfg(all(test,target_os="linux"))]
#[test]fn rule_authority_configured_binding_survives_readiness_changes_but_never_model_switch(){
 use crate::pet_capabilities::{Descriptor,Binding as B,Kind,Lifecycle,Concurrency,Capability,Support,Availability,Reason,Source,Decision,Limits};
 let mut d=Descriptor{schema_version:1,pet_id:"ai_ollama".into(),kind:Kind::Llm,name:"Private helper".into(),lifecycle:Lifecycle::Active,binding:B{engine_binding:None,adapter:Some("ollama"),provider:Some("ollama".into()),model:Some("explicit-model".into()),profile:None},revision:"observation-one".into(),observed_at:Some(1),configuration_available:true,capabilities:vec![Capability{id:"goals.admit",support:Support::Supported,availability:Availability::Ready,reason:None,source:vec![Source::Configuration],observed_at:Some(1),expires_at:Some(2),model_binding:Some("explicit-model".into()),decision:Decision::NativeApproval,callers:vec![],limits:Limits::default(),limitations:vec![]}],concurrency:Concurrency{scope:None,max_active:None}};
 let original=descriptor_binding(&d).unwrap();d.revision="observation-two".into();d.observed_at=Some(4);d.capabilities[0].availability=Availability::Unknown;d.capabilities[0].reason=Some(Reason::StaleObservation);d.capabilities[0].observed_at=None;assert_eq!(descriptor_binding(&d).unwrap(),original);d.binding.model=Some("different-model".into());assert_ne!(descriptor_binding(&d).unwrap(),original);
}
