//! Original native System admission/Authority, then retirement with its native
//! server stopped. Provider/human metadata are fixtures; no user audio/quota.
use super::{admission_tests::Fixture,native_system_admission_tests as setup,lease_controller::{Controller,Validity},lease_controller::native_system_sound_task_tests::{Audio,serial},authority::{Authority,Failure},authority_store::Fault,model::{self,Record,State,Action},store::{Store,nonce},retirement_controller::{Prepared,Review},reconciliation_tests::inventory};
use crate::chat::Chat;
use std::{fs,sync::Arc};
fn clock()->u64{crate::assistant_memory::now()}
fn root(f:&Fixture)->std::path::PathBuf{f.path.join("authority")}
fn initial(provider:&str)->(Fixture,Store,Record,Record,String){
 let audio=Audio::new();let f=Fixture::new();let g=setup::draft(&f,&audio,provider);let c=setup::config(&audio,&g);setup::save(&f,&c);let review=setup::review(&f,&audio,&g,&c);let now=clock();let approved=review.approve(&g,setup::evidence(&g,&c,now),now).unwrap();let a=Authority::ensure(&root(&f)).unwrap();let decision=approved.spec().decision_id.clone();let mut preview=g.clone();
 {let cert=a.certify(&approved,&g,clock()).unwrap();preview.enqueue(&approved,Some(cert.proof().clone()),clock(),nonce().unwrap()).unwrap();}
 assert_eq!(preview.schema_version,62);drop(approved);drop(audio);
 let store=Store::existing_verified(&f.path.join("goals"),&root(&f)).unwrap().unwrap();assert!(store.read(&g.id,&g.spec.owner.pet_id).unwrap().admissions.is_empty());(f,store,g,preview,decision)
}
fn review(f:&Fixture,s:&Store,g:&Record,d:&str)->Review{match s.prepare_retirement(&root(f),&g.id,&g.spec.owner.pet_id,&g.revision,d,"retire-original-system".into(),clock()).unwrap(){Prepared::Review(r)=>r,_=>panic!("Original exact review required")}}
fn body(f:&Fixture,d:&str)->serde_json::Value{serde_json::from_slice::<serde_json::Value>(&fs::read(root(f).join(format!("{d}.json"))).unwrap()).unwrap()["body"].clone()}
#[test]#[ignore="Private native grant preparation; retirement performs no provider/native output IO"]
fn native_system_orphan_retirement_keeps_original_hash_budget_approval_and_revokes_preview_without_output_or_model(){
 let _serial=serial();for provider in["ollama","copilot","integration_spotify"]{
  let(f,s,g,preview,d)=initial(provider);let before=inventory(&f.path);let original=body(&f,&d);assert_eq!(original["schemaVersion"],48);let r=review(&f,&s,&g,&d);let p=r.proposal();assert_eq!(inventory(&f.path),before);assert_eq!(p.original_operation,"goalDeterministic");assert_eq!(p.native_system_plan_hash.as_deref(),original["nativeSystemPlanHash"].as_str());assert!(p.target_usage_known_zero&&!p.execution_available&&!p.history_continuity_verified);
  let choice=r.decide(clock()).unwrap();let mut approved=r.approve(choice);let receipt=approved.confirm(&s,clock()).unwrap();assert_eq!(receipt.reference.schema_version,3);assert_eq!(receipt.reference.source_grant_schema,48);assert!(!receipt.execution_available&&!receipt.new_admission_created);assert_eq!(model::fingerprint(&receipt.record).unwrap(),model::fingerprint(&g).unwrap());let current=body(&f,&d);assert_eq!(current["schemaVersion"],48);assert_eq!(current["phase"],"revoked");for field in["specHash","approvedAt","nativeSystemPlanHash"]{assert_eq!(current[field],original[field]);}for field in["lease","runtimeOwner","budgetUsage","planningUsage"]{assert!(current.get(field).is_none());}
  let a=Authority::existing(&root(&f)).unwrap();assert_eq!(a.verify_queued(&preview,clock()).err(),Some(Failure::Revoked));let mut wrong=receipt.reference.clone();wrong.schema_version=2;assert!(!wrong.basic());let mut wrong=receipt.reference.clone();wrong.source_grant_schema=10;assert!(!wrong.basic());
  let changed=inventory(&f.path).into_iter().filter(|(n,v)|before.get(n)!=Some(v)).collect::<Vec<_>>();assert_eq!(changed.len(),1);let h=s.history(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,clock()).unwrap();assert!(h.accounting_complete&&h.unlinked_grants[0].revoked_without_io);let usage=h.cumulative_charge.unwrap();assert_eq!(usage.model_calls,0);assert_eq!(usage.tool_calls,0);let stable=inventory(&f.path);approved.acknowledged().receipt(&s).unwrap();assert_eq!(inventory(&f.path),stable);
 }
 println!("SYSTEM_ORPHAN_RETIRE owners=3 originalGrant48HashApprovalPreserved=true reference3=true previewProofRevoked=true noModelsOutputsOrNewBudget=true");
}
#[test]#[ignore="Private native grant preparation and signed fault injection; no provider/effect IO"]
fn native_system_orphan_retirement_partial_publish_decline_and_restart_ack_keep_one_original_choice(){
 let _serial=serial();for fault in[Fault::BeforeGrantPublish,Fault::AfterGrantPublish]{
  let(f,s,g,_preview,d)=initial("ollama");let original=body(&f,&d);let r=review(&f,&s,&g,&d);let choice=r.decide(clock()).unwrap();let mut approved=r.approve(choice);let reference=approved.reference.clone();approved.authority.fault=fault;assert!(approved.confirm(&s,clock()).is_err());assert!(approved.write_started);approved.authority.fault=Fault::None;
  if matches!(fault,Fault::BeforeGrantPublish){assert!(s.withdraw_retirement(&approved).unwrap());assert_eq!(body(&f,&d),original);assert!(!fs::read_dir(root(&f)).unwrap().any(|p|p.unwrap().file_name().to_string_lossy().starts_with(".grant-")));let r=review(&f,&s,&g,&d);let choice=r.decide(clock()).unwrap();let mut retry=r.approve(choice);retry.authority.fault=Fault::BeforeGrantPublish;assert!(retry.confirm(&s,clock()).is_err());retry.authority.fault=Fault::None;let reference=retry.reference.clone();let receipt=retry.confirm(&s,clock()+300_000).unwrap();assert_eq!(receipt.reference,reference);assert_eq!(receipt.reference.schema_version,3);let stable=inventory(&f.path);retry.confirm(&s,clock()+300_001).unwrap();assert_eq!(inventory(&f.path),stable);}
  else{drop(approved);let cancelled=s.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"cancel-after-recorded-system-retirement".into(),Action::Cancel,clock()).unwrap().record;let before=inventory(&f.path);let receipt=match s.prepare_retirement(&root(&f),&g.id,&g.spec.owner.pet_id,&cancelled.revision,&d,"ack-after-restart".into(),clock()+300_000).unwrap(){Prepared::Retired(r)=>r,_=>panic!("Recorded reduction must be ACK only")};assert_eq!(receipt.reference,reference);assert_eq!(receipt.record.state,State::Cancelled);assert_eq!(receipt.record.revision,cancelled.revision);assert_eq!(inventory(&f.path),before);}
  let current=body(&f,&d);assert_eq!(current["schemaVersion"],48);assert_eq!(current["specHash"],original["specHash"]);assert_eq!(current["approvedAt"],original["approvedAt"]);assert!(current.get("budgetUsage").is_none());
 }
 println!("SYSTEM_ORPHAN_FAULT originalChoiceRetained=true declineBeforeCommitPreservesIssued=true restartAckDoesNotRenew=true noIOOrBudget=true");
}
#[test]#[ignore="Private signed queue/claim source and native grant preparation; no provider/effect IO"]
fn native_system_orphan_retirement_refuses_linked_and_omitted_claims_without_zero_usage_inference(){
 let _serial=serial();for claimed in[false,true]{
  let(f,s,g,preview,d)=initial("ollama");let file=f.path.join("goals").join(format!("{}.json",g.id));fs::write(&file,serde_json::to_vec(&preview).unwrap()).unwrap();let a=Authority::existing(&root(&f)).unwrap();let mut owned=None;let mut leased=None;
  if claimed{let c:crate::pet_config::Configuration=serde_json::from_slice(&fs::read(f.path.join("system.json")).unwrap()).unwrap();let chat=Chat::persistent(f.path.join("main.json"));let now=clock();let pending=Controller::new().unwrap().prepare(&preview,&root(&f),&f.path.join("contexts"),&chat,&setup::evidence(&preview,&c,now),"claim-system-for-negative-retirement".into(),now,Arc::new(|_,_,_|Validity::Current)).unwrap();let now=clock();let(r,o)=pending.claim(&s,&a,&setup::evidence(&preview,&c,now),now).unwrap();leased=Some(r.record);owned=o;fs::write(&file,serde_json::to_vec(&g).unwrap()).unwrap();}
  let source=if claimed{&g}else{&preview};let before=inventory(&f.path);assert!(s.prepare_retirement(&root(&f),&source.id,&source.spec.owner.pet_id,&source.revision,&d,"cannot-retire-used-system".into(),clock()).is_err());assert_eq!(inventory(&f.path),before);assert!(body(&f,&d).get("retirement").is_none());
  if let(Some(leased),Some(owner))=(leased,owned){fs::write(&file,serde_json::to_vec(&leased).unwrap()).unwrap();let drain=owner.drain(super::leases::DrainOutcome::Confirmed).unwrap();s.drain_owned(&a,&drain,clock()).unwrap();}
 }
 println!("SYSTEM_ORPHAN_USED linkedAndOmittedClaimRefused=true noZeroUsageFromMissingGoalRow=true noRetirementOrEffects=true");
}
