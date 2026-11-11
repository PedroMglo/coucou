use super::*;
use crate::pet_runtime::{rules::{RulePolicy,RuleScope,RuleKind,DispatchPolicy},native_owner::EndedOperation};
use serde_json::json;
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt};
struct F{path:PathBuf,grant:RuleGrant}
impl F{
 fn new()->Self{
  let path=std::env::temp_dir().join(format!("coucou-rule-intent-retirement-{}",nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
  let definition=serde_json::from_value(json!({"name":"Original metadata scope","intent":"One finite response","criteria":["Verify actual result"],"schedule":{"kind":"daily","firstDate":"2026-11-10","days":2,"localTime":"09:00","timeZone":"Etc/UTC","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"},"missed":"skip","graceSeconds":300,"dataScope":"none","limits":{"maxOccurrences":2,"maxModelCalls":2,"maxToolCalls":0,"maxOutputBytes":8192,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":900}})).unwrap();
  let rule=crate::pet_runtime::rules::Rule{id:format!("rule-{}","a".repeat(64)),revision:1,owner:crate::pet_runtime::model::Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None},state:crate::pet_runtime::rules::State::Draft,created_at:1000,updated_at:1000,definition};
  let schedule={let _s=crate::pet_calendar::TEST_SERIAL.lock().unwrap();crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||true).unwrap()};let first=schedule.slots[0].at.unwrap();let last=schedule.slots.last().unwrap().at.unwrap();
  let policy=RulePolicy{dispatch:Some(DispatchPolicy::bounded()),schema_version:2,scope:RuleScope::BoundedAutomaticRuleRecurrenceWithoutSelectedData,rule,payment:crate::pet_runtime::admission::Payment::LocalOnly,destination:crate::pet_memory::context_model::Destination::Server{provider:"ollama".into(),endpoint:"http://127.0.0.1:11434/v1/chat/completions".into(),model:"fixture-free".into()},model:"fixture-free".into(),engine_binding:None,policy_revision:"b".repeat(64),descriptor_semantics:"c".repeat(64),schedule,prepared_at:first-60000,valid_until:last+900000};
  let grant=RuleGrant{schema_version:3,kind:RuleKind::RuleRecurrence,decision_id:nonce().unwrap(),revision:nonce().unwrap(),approved_at:policy.prepared_at,updated_at:policy.prepared_at,revoked_at:None,phase:RulePhase::Active,policy,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None};assert!(grant.valid());let f=Self{path,grant};f.a().issue_rule(&f.grant,f.grant.approved_at,||true).unwrap();f
 }
 fn a(&self)->Authority{Authority::existing(&self.path.join("authority")).or_else(|_|Authority::ensure(&self.path.join("authority"))).unwrap()}
 fn at(&self)->u64{self.grant.policy.schedule.slots[0].at.unwrap()}
 fn bytes(&self)->Vec<u8>{fs::read(self.path.join("authority").join(format!("{}.json",self.grant.decision_id))).unwrap()}
 fn revoke(&self,a:&Authority)->RuleGrant{a.revoke_rule("ai_ollama",&self.grant.policy.rule.id,&self.grant.decision_id,self.at()+10).unwrap()}
 fn staged(&self,mode:&str)->RuleGrant{
  let mut a=self.a();let g=&self.grant;
  if mode=="issuance"{
   // Issuance failed before the first canonical record existed.
   fs::remove_file(self.path.join("authority").join(format!("{}.json",g.decision_id))).unwrap();a.fault=Fault::BeforeGrantPublish;assert!(a.issue_rule(g,g.approved_at,||true).is_err());
  }else if mode=="reservation"{a.fault=Fault::BeforeGrantPublish;assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,self.at(),||true).is_err());}
  else{
   let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,self.at(),||true).unwrap();
   if mode=="begin"{a.fault=Fault::BeforeGrantPublish;assert!(a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,self.at(),||true).is_err());}
   else{let owned=a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,self.at(),||true).unwrap();a.fault=Fault::BeforeGrantPublish;assert!(a.finish_rule_probe(&g.decision_id,&r.occurrence_id,owned.probe(),true,self.at()+1).is_err());}
  }
  a.fault=Fault::None;self.revoke(&a)
 }
}
impl Drop for F{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
#[test]fn rule_intent_retirement_core_keeps_revocation_and_all_original_commitments(){
 for mode in ["issuance","reservation","begin","finish"]{
  let f=F::new();let revoked=f.staged(mode);let a=f.a();let before=f.bytes();let source=a.prepare_rule_intent_retirement(&f.grant.decision_id).unwrap();assert_eq!(before,f.bytes(),"Preparation is passive");
  let chosen=source.candidate(&nonce().unwrap(),f.at()+20).unwrap();assert_eq!(chosen.schema_version,5);assert_eq!(chosen.phase,RulePhase::Revoked);assert_eq!(chosen.revoked_at,revoked.revoked_at);assert_eq!(chosen.policy,f.grant.policy);assert_eq!(chosen.approved_at,f.grant.approved_at);
  if mode!="issuance"{assert_eq!(chosen.reservations.len(),1);assert_eq!(chosen.reservations[0].model_calls,1);assert_eq!(chosen.reservations[0].deadline_at,f.at()+900000);}
  if mode=="begin"{assert_eq!(chosen.reservations[0].readiness.len(),1);assert_eq!(chosen.reservations[0].readiness[0].finished_at,None);}
  if mode=="finish"{assert_eq!(chosen.reservations[0].readiness[0].observed,Some(true));assert_eq!(chosen.reservations[0].readiness[0].finished_at,Some(f.at()+1));}
  let actual=a.retire_rule_intents(&source,&chosen,f.at()+21,||true).unwrap();assert_eq!(actual,chosen);assert_eq!(a.rule_status(&chosen.policy.rule,true).unwrap().pending_artifacts,0);
  assert!(a.reserve_rule_occurrence(&chosen.decision_id,&chosen.policy.hash().unwrap(),0,f.at()+22,||true).is_err());
  let after=f.bytes();assert_eq!(a.retire_rule_intents(&source,&chosen,f.at()+22,||true).unwrap(),chosen);assert_eq!(after,f.bytes());
  let mut illicit=chosen.clone();illicit.phase=RulePhase::Active;illicit.revoked_at=None;assert!(!illicit.valid());
 }
}
#[test]fn rule_intent_retirement_core_recovers_original_choice_across_publish_and_cleanup_faults(){
 for fault in [Fault::BeforeGrantPublish,Fault::AfterGrantPublish,Fault::BeforeRuleIntentCleanup,Fault::AfterRuleIntentCleanup]{
  let f=F::new();f.staged("begin");let mut a=f.a();let source=a.prepare_rule_intent_retirement(&f.grant.decision_id).unwrap();let chosen=source.candidate(&nonce().unwrap(),f.at()+20).unwrap();
  a.fault=fault;assert!(a.retire_rule_intents(&source,&chosen,f.at()+21,||true).is_err());a.fault=Fault::None;
  let canonical=a.rule_record_data(&format!("{}.json",chosen.decision_id)).unwrap().unwrap().0;assert_eq!(canonical.phase,RulePhase::Revoked);
  drop(a);let reopened=f.a();
  let result=match reopened.prepare_rule_intent_retirement(&chosen.decision_id){
   Ok(source)=>{let original=source.candidate(&nonce().unwrap(),f.at()+2000).unwrap();assert_eq!(original,chosen,"ACK must retain original decision, timestamp, body and limits");reopened.retire_rule_intents(&source,&original,f.at()+2001,||true).unwrap()},
   Err(Failure::Missing)=>reopened.retire_rule_intents(&source,&chosen,f.at()+2001,||true).unwrap(),
   Err(e)=>panic!("Unexpected recovery error: {e:?}"),
  };
  assert_eq!(result,chosen);assert_eq!(reopened.rule_status(&chosen.policy.rule,true).unwrap().pending_artifacts,0);assert_eq!(result.reservations[0].readiness[0].finished_at,None);
 }
}
#[test]fn rule_intent_retirement_core_does_not_drain_live_worker_or_overwrite_later_observation(){
 let f=F::new();let mut a=f.a();let g=&f.grant;let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(),||true).unwrap();let owned=a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,f.at(),||true).unwrap();
 a.fault=Fault::BeforeGrantPublish;assert!(a.finish_rule_probe(&g.decision_id,&r.occurrence_id,owned.probe(),true,f.at()+1).is_err());a.fault=Fault::None;f.revoke(&a);
 let source=a.prepare_rule_intent_retirement(&g.decision_id).unwrap();let chosen=source.candidate(&nonce().unwrap(),f.at()+20).unwrap();a.retire_rule_intents(&source,&chosen,f.at()+21,||true).unwrap();
 assert!(owned.current(&a));assert!(EndedOperation::capture(&a,owned.probe().owner.as_ref().unwrap()).is_err());drop(owned);
 // A further exact observation ACK is metadata only and must retain receipt5.
 a.finish_rule_probe(&g.decision_id,&r.occurrence_id,&chosen.reservations[0].readiness[0],true,f.at()+22).unwrap();
 let after=f.bytes();assert_eq!(a.retire_rule_intents(&source,&chosen,f.at()+23,||true).unwrap(),chosen);assert_eq!(after,f.bytes());
 // A begin intent remains unknown after consolidation, and uses the separate
 // end-witness path once the original operation has really ended.
 let f=F::new();f.staged("begin");let a=f.a();let s=a.prepare_rule_intent_retirement(&f.grant.decision_id).unwrap();let c=s.candidate(&nonce().unwrap(),f.at()+20).unwrap();a.retire_rule_intents(&s,&c,f.at()+21,||true).unwrap();let recovery=a.prepare_rule_probe_recovery(&c.decision_id).unwrap();let gate=EndedOperation::capture(&a,recovery.owner().unwrap()).unwrap();let observed=recovery.candidate(&nonce().unwrap(),f.at()+22).unwrap();a.recover_rule_probe(&recovery,&observed,&gate,f.at()+23,||true).unwrap();let after=f.bytes();assert_eq!(a.retire_rule_intents(&s,&c,f.at()+24,||true).unwrap(),observed);assert_eq!(after,f.bytes());
}
#[test]fn rule_intent_retirement_core_refuses_cancel_changed_source_and_conflicting_data(){
 let f=F::new();f.staged("reservation");let a=f.a();let source=a.prepare_rule_intent_retirement(&f.grant.decision_id).unwrap();let chosen=source.candidate(&nonce().unwrap(),f.at()+20).unwrap();let before=f.bytes();
 assert!(a.retire_rule_intents(&source,&chosen,f.at()+21,||false).is_err());assert_eq!(before,f.bytes());
 let mut forged=chosen.clone();forged.intent_retirements[0].native_decision_id=nonce().unwrap();assert!(a.retire_rule_intents(&source,&forged,f.at()+21,||true).is_err());assert_eq!(before,f.bytes());
 let mut changed=source.images[0].body.clone();changed.policy.model="different".into();assert!(join(&source.base,&[Image{body:changed,..source.images[0].clone()}]).is_err());
 let mut changed=chosen.clone();changed.reservations[0].reserved_at+=1;assert!(join_bodies(&chosen,std::iter::once(&changed)).is_err());
 let image=&source.images[0];let path=f.path.join("authority").join(&image.name);let bytes=fs::read(&path).unwrap();fs::rename(&path,path.with_extension("old")).unwrap();fs::write(&path,bytes).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();
 assert!(a.retire_rule_intents(&source,&chosen,f.at()+21,||true).is_err());assert_eq!(before,f.bytes());
}

#[test]fn rule_intent_retirement_core_partial_cleanup_keeps_exact_inventory_and_repeated_revoke(){
 let f=F::new();let revoked=f.staged("reservation");let mut a=f.a();let mut redundant=f.grant.clone();redundant.phase=RulePhase::Revoked;redundant.revoked_at=revoked.revoked_at;redundant.updated_at=revoked.updated_at;redundant.revision=nonce().unwrap();a.fault=Fault::BeforeGrantPublish;assert!(a.publish_rule(&redundant,false,||Ok(())).is_err());a.fault=Fault::None;
 let source=a.prepare_rule_intent_retirement(&f.grant.decision_id).unwrap();assert_eq!(source.images.len(),2);let chosen=source.candidate(&nonce().unwrap(),f.at()+20).unwrap();a.fault=Fault::AfterRuleIntentCleanup;assert!(a.retire_rule_intents(&source,&chosen,f.at()+21,||true).is_err());drop(a);
 let a=f.a();assert_eq!(a.rule_status(&f.grant.policy.rule,true).unwrap().pending_artifacts,1);assert_eq!(a.revoke_rule("ai_ollama",&f.grant.policy.rule.id,&chosen.decision_id,f.at()+22).unwrap(),chosen);let next=a.prepare_rule_intent_retirement(&chosen.decision_id).unwrap();assert_eq!(next.candidate(&nonce().unwrap(),f.at()+200).unwrap(),chosen);a.retire_rule_intents(&next,&chosen,f.at()+201,||true).unwrap();assert_eq!(chosen.intent_retirements[0].intents.len(),2);assert_eq!(a.rule_status(&chosen.policy.rule,true).unwrap().pending_artifacts,0);
}
#[test]fn rule_intent_retirement_core_rejects_conflicting_observations_and_preserves_old_empty_bytes(){
 let f=F::new();let a=f.a();let g=&f.grant;let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(),||true).unwrap();let owned=a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,f.at(),||true).unwrap();a.finish_rule_probe(&g.decision_id,&r.occurrence_id,owned.probe(),true,f.at()+1).unwrap();let revoked=f.revoke(&a);let mut conflict=revoked.clone();conflict.reservations[0].readiness[0].observed=Some(false);assert!(conflict.valid());assert!(join_bodies(&revoked,std::iter::once(&conflict)).is_err());
 let serialized=serde_json::to_value(&revoked).unwrap();assert!(serialized.get("intentRetirements").is_none());assert!(serde_json::from_value::<RuleGrant>(serialized.clone()).unwrap()==revoked);let mut future=serialized;future["intentRetirements"]=json!([{"inventedApproval":true}]);assert!(serde_json::from_value::<RuleGrant>(future).is_err());
}
/// Offline fixture producer, explicitly invoked only for a marked generated
/// profile. No production endpoint, copied profile, key export or model IO.
#[test]#[ignore]fn rule_intent_retirement_native_fixture(){
 let Ok(raw)=std::env::var("COUCOU_INTENT_FIXTURE_NATIVE")else{return;};let target=PathBuf::from(raw);assert!(target.is_absolute()&&!target.is_symlink());assert_eq!(fs::read(target.join(".qa-intent-profile")).unwrap(),b"coucou-generated-revoked-intents-only");
 let endpoint=std::env::var("COUCOU_INTENT_FIXTURE_ENDPOINT_NATIVE").unwrap();let url=reqwest::Url::parse(&endpoint).unwrap();assert_eq!(url.host_str(),Some("127.0.0.1"));assert_eq!(url.scheme(),"http");assert_eq!(url.path(),"/v1/chat/completions");
 let f=F::new();let mut g=f.grant.clone();if let crate::pet_memory::context_model::Destination::Server{endpoint:ep,..}=&mut g.policy.destination{*ep=endpoint;}assert!(g.valid());let root=target.join("pet-runtime-consent-v1");
 if root.exists(){assert!(fs::read_dir(&root).unwrap().all(|f|matches!(f.unwrap().file_name().to_str(),Some(".key.json"|".catalog.lock"|".protocol.json"))));}
 let mut a=Authority::ensure(&root).unwrap();a.issue_rule(&g,g.approved_at,||true).unwrap();let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(),||true).unwrap();a.fault=Fault::BeforeGrantPublish;assert!(a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,f.at(),||true).is_err());a.fault=Fault::None;a.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,f.at()+10).unwrap();
 let book=target.join("pet-rules.json");if book.exists(){let old:serde_json::Value=serde_json::from_slice(&fs::read(&book).unwrap()).unwrap();assert!(old["rules"].as_array().unwrap().is_empty());}fs::write(&book,serde_json::to_vec(&json!({"version":1,"generation":1,"rules":[g.policy.rule]})).unwrap()).unwrap();fs::set_permissions(&book,fs::Permissions::from_mode(0o600)).unwrap();
}
