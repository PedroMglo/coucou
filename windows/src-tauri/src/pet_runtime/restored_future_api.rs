//! Native-user data review. No model-facing command accepts a Spec/Origin or
//! selects a signer/path. Confirmation creates no execution authority.
use super::{model,restored_future};
use serde::{Serialize,Deserialize};
use tauri::AppHandle;
#[cfg(target_os="linux")]use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock},path::PathBuf};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FuturePrepareInput{pub pet_id:String,pub source_goal_id:String,pub source_revision:String,pub request_id:String,pub intent:String,pub criteria:Vec<String>}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FutureConfirmInput{pub pet_id:String,pub source_goal_id:String,pub source_revision:String,pub token:String,pub approve:bool,pub acknowledge_unknown_state:bool}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct FutureTicket{pub token:String,pub proposal:restored_future::Proposal}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum FuturePrepared{Review{ticket:FutureTicket},Existing{receipt:model::Receipt,#[serde(rename="executionAuthorityGranted")]execution_authority_granted:bool}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum FutureConfirmed{Created{receipt:model::Receipt,#[serde(rename="executionAuthorityGranted")]execution_authority_granted:bool},Declined}
#[cfg(target_os="linux")]
struct Pending{window:String,proposal:restored_future::Proposal,root:PathBuf,authority:PathBuf,review:Mutex<restored_future::Review>}
#[cfg(target_os="linux")]
#[derive(Default)]struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
#[cfg(target_os="linux")]
impl Reviews{
 fn insert(&mut self,window:&str,root:PathBuf,authority:PathBuf,review:restored_future::Review,token:String,now:u64)->model::Result<FutureTicket>{
  if window.is_empty()||!model::token(&token){return Err(model::Error::InvalidInput);}self.pending.retain(|_,p|p.review.try_lock().map_or(true,|r|r.retained(now)));
  let proposal=review.proposal();if now<proposal.prepared_at||now>=proposal.expires_at{return Err(model::Error::Conflict);}for(token,p)in &self.pending{if p.window==window&&p.proposal.target_goal_id==proposal.target_goal_id{
   if model::fingerprint(&(&p.proposal.source,&p.proposal.future))?!=model::fingerprint(&(&proposal.source,&proposal.future))?||p.root!=root||p.authority!=authority{return Err(model::Error::Conflict);}return Ok(FutureTicket{token:token.clone(),proposal:p.proposal.clone()});
  }}if self.pending.len()>=16{return Err(model::Error::LimitReached);}if self.pending.contains_key(&token){return Err(model::Error::Conflict);}
  self.pending.insert(token.clone(),Arc::new(Pending{window:window.into(),proposal:proposal.clone(),root,authority,review:Mutex::new(review)}));Ok(FutureTicket{token,proposal})
 }
 fn lookup(&self,window:&str,input:&FutureConfirmInput)->model::Result<Arc<Pending>>{
  let p=self.pending.get(&input.token).ok_or(model::Error::Conflict)?;let s=&p.proposal.source;
  if p.window!=window||s.pet_id!=input.pet_id||s.goal_id!=input.source_goal_id||s.revision!=input.source_revision{return Err(model::Error::Conflict);}Ok(p.clone())
 }
}
#[cfg(target_os="linux")]
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
#[tauri::command]
pub async fn pet_runtime_prepare_restored_future(app:AppHandle,window:tauri::WebviewWindow,input:FuturePrepareInput)->model::Result<FuturePrepared>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let window=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let root=crate::settings::config_dir().join("pet-runtime-v1");let authority=super::admission_review::authority_root();let store=super::store::Store::existing_verified(&root,&authority)?.ok_or(model::Error::NotFound)?;
  if let Some(receipt)=store.existing_restored_future(&input.pet_id,&input.source_goal_id,&input.source_revision,&input.request_id,&input.intent,&input.criteria)?{return Ok(FuturePrepared::Existing{receipt,execution_authority_granted:false});}
  let owner=super::adapter::owner(&app,&input.pet_id)?;let request_id=restored_future::request_id(&input.pet_id,&input.source_goal_id,&input.source_revision,&input.request_id)?;let spec=model::Spec{restored_from:None,owner,request_id,context:model::Context::Unbound,intent:input.intent,criteria:input.criteria};
  let now=crate::assistant_memory::now();let review=restored_future::Review::prepare(&store,&authority,&input.source_goal_id,&input.pet_id,&input.source_revision,spec,input.request_id,now)?;let ticket=reviews().lock().map_err(|_|model::Error::StorageUnavailable)?.insert(&window,root,authority,review,super::store::nonce()?,crate::assistant_memory::now())?;Ok(FuturePrepared::Review{ticket})
 }).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_restored_future(window:tauri::WebviewWindow,input:FutureConfirmInput)->model::Result<FutureConfirmed>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let window=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
  let pending=reviews().lock().map_err(|_|model::Error::StorageUnavailable)?.lookup(&window,&input)?;let mut review=pending.review.try_lock().map_err(|_|model::Error::Busy)?;
  let store=super::store::Store::existing_verified(&pending.root,&pending.authority)?.ok_or(model::Error::NotFound)?;let now=crate::assistant_memory::now();
  if !review.decide(&store,input.approve,input.acknowledge_unknown_state,now)?{return Ok(FutureConfirmed::Declined);}let receipt=store.create_restored_future_clock(&review.approved()?,crate::assistant_memory::now)?;if receipt.status!=model::CommitStatus::Uncertain{review.acknowledge();}Ok(FutureConfirmed::Created{receipt,execution_authority_granted:false})
 }).await.map_err(|_|model::Error::StorageUnavailable)?}
}
#[cfg(all(test,target_os="linux"))]
mod tests{
 use super::*;
 #[test]fn native_future_responses_match_the_typescript_data_only_wire_contract(){
  let(_from,to,old,mut review)=super::super::restoration::fixture_restored_future_review();let store=super::super::store::Store::existing_verified(&to.path.join("goals"),&crate::pet_runtime::authority_tests::root(&to)).unwrap().unwrap();review.decide(&store,true,true,100_011).unwrap();let receipt=store.create_restored_future(&review.approved().unwrap(),100_012).unwrap();let spec=receipt.record.spec.clone();let created=serde_json::to_value(FutureConfirmed::Created{receipt,execution_authority_granted:false}).unwrap();assert_eq!(created["status"],"created");assert_eq!(created.get("executionAuthorityGranted"),Some(&serde_json::json!(false)));assert!(created.get("execution_authority_granted").is_none());
  let receipt=store.existing_restored_future(&old.spec.owner.pet_id,&old.id,&old.revision,"native-window-scope",&spec.intent,&spec.criteria).unwrap().unwrap();let existing=serde_json::to_value(FuturePrepared::Existing{receipt,execution_authority_granted:false}).unwrap();assert_eq!(existing["status"],"existing");assert_eq!(existing.get("executionAuthorityGranted"),Some(&serde_json::json!(false)));assert!(existing.get("execution_authority_granted").is_none());assert_eq!(serde_json::to_value(FutureConfirmed::Declined).unwrap(),serde_json::json!({"status":"declined"}));
 }
 #[test]fn window_owner_revision_and_token_bind_the_only_native_creation_review(){
  let(_from,to,old,review)=super::super::restoration::fixture_restored_future_review();let root=to.path.join("goals");let authority=crate::pet_runtime::authority_tests::root(&to);let mut cache=Reviews::default();let token="a".repeat(32);cache.insert("settings",root,authority,review,token.clone(),100_010).unwrap();let mut input=FutureConfirmInput{pet_id:old.spec.owner.pet_id,source_goal_id:old.id,source_revision:old.revision,token,approve:true,acknowledge_unknown_state:true};assert!(cache.lookup("settings",&input).is_ok());assert!(cache.lookup("mochi",&input).is_err());input.pet_id="ai_ollama".into();assert!(cache.lookup("settings",&input).is_err());input.pet_id="agent_codex".into();input.source_revision="b".repeat(32);assert!(cache.lookup("settings",&input).is_err());input.source_revision="c".repeat(32);input.token="d".repeat(32);assert!(cache.lookup("settings",&input).is_err());
 }
 #[test]fn caller_cannot_supply_a_spec_signer_certificate_or_destination_provider(){
  let value=serde_json::json!({"petId":"agent_codex","sourceGoalId":format!("g-{}","a".repeat(64)),"sourceRevision":"b".repeat(32),"requestId":"future-native","intent":"Review future work","criteria":["Verify"],"restoredFrom":{"tag":"forged"}});assert!(serde_json::from_value::<FuturePrepareInput>(value).is_err());
  let value=serde_json::json!({"petId":"agent_codex","sourceGoalId":format!("g-{}","a".repeat(64)),"sourceRevision":"b".repeat(32),"token":"c".repeat(32),"approve":true,"acknowledgeUnknownState":true,"owner":{"provider":"copilot"}});assert!(serde_json::from_value::<FutureConfirmInput>(value).is_err());
 }
 #[test]fn expired_data_review_is_not_a_renewed_choice_and_persisted_creation_remains_acknowledgeable(){
  let(_from,to,old,review)=super::super::restoration::fixture_restored_future_review();let root=to.path.join("goals");let authority=crate::pet_runtime::authority_tests::root(&to);let store=super::super::store::Store::existing_verified(&root,&authority).unwrap().unwrap();let mut cache=Reviews::default();let ticket=cache.insert("settings",root.clone(),authority.clone(),review,"a".repeat(32),100_010).unwrap();let input=FutureConfirmInput{pet_id:old.spec.owner.pet_id.clone(),source_goal_id:old.id.clone(),source_revision:old.revision.clone(),token:ticket.token,approve:true,acknowledge_unknown_state:true};let pending=cache.lookup("settings",&input).unwrap();let mut original=pending.review.lock().unwrap();original.decide(&store,true,true,100_011).unwrap();let created=store.create_restored_future(&original.approved().unwrap(),100_012).unwrap();assert!(!original.retained(220_010));drop(original);
  let owner=old.spec.owner.clone();let request=super::super::restored_future::request_id(&old.spec.owner.pet_id,&old.id,&old.revision,"another-choice").unwrap();let spec=model::Spec{restored_from:None,request_id:request,owner,context:model::Context::Unbound,intent:"Another explicit data review".into(),criteria:vec!["Native review only".into()]};let next=restored_future::Review::prepare(&store,&authority,&old.id,&old.spec.owner.pet_id,&old.revision,spec,"another-choice".into(),220_011).unwrap();cache.insert("settings",root,authority,next,"b".repeat(32),220_011).unwrap();assert!(cache.lookup("settings",&input).is_err());let ack=store.existing_restored_future(&old.spec.owner.pet_id,&old.id,&old.revision,"native-window-scope",&created.record.spec.intent,&created.record.spec.criteria).unwrap().unwrap();assert_eq!(ack.record.spec,created.record.spec);assert!(!ack.record.spec.restored_from.unwrap().body.execution_authority_granted);
 }
}
