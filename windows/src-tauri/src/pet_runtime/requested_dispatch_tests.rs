use super::*;
use crate::pet_runtime::{admission::{Review,Payment},admission_tests::{Fixture,budget,evidence},authority_tests::root,generated_effect_tests::config,planning::{Limits,Start,Policy},store::{Store,Fault}};
fn setup(start:Option<Start>)->(Fixture,Authority,Record){setup_named(start,"requested-native")}
fn setup_named(start:Option<Start>,request:&str)->(Fixture,Authority,Record){
 let f=Fixture::new();let authority=Authority::ensure(&root(&f)).unwrap();let goal=f.goal_named("ollama",request);let at=crate::assistant_memory::now()-100;
 let mut bounds=budget();bounds.deadline_at=at+120_000;bounds.max_output_bytes=32_768;bounds.payment=Payment::LocalOnly;
 let approved=Review::prepare_planning(&goal,evidence(&goal,at),bounds,Limits{max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit:2},&config(),"requested-admit".into(),nonce().unwrap(),at).unwrap().with_start(start).unwrap().approve(&goal,evidence(&goal,at+1),at+1).unwrap();
 let certified=authority.certify(&approved,&goal,at+2).unwrap();let queued=f.store().enqueue_certified(&certified,at+3).unwrap().record;(f,authority,queued)
}
fn native(f:&Fixture)->Store{Store::existing_verified(&f.path.join("goals"),&root(f)).unwrap().unwrap()}
#[test]fn manual_planning_is_byte_compatible_and_cannot_be_interpreted_as_requested_execution(){
 let(f,authority,g)=setup(None);assert_eq!(g.schema_version,8);let policy=g.admissions[0].spec.planning_policy.as_ref().unwrap();let value=serde_json::to_value(policy).unwrap();assert_eq!(value["schemaVersion"],1);assert!(value.get("start").is_none());assert!(!requested(&g));authority.verify_queued(&g,crate::assistant_memory::now()).unwrap();assert!(native(&f).acknowledge_requested(&g.id,"ai_ollama",&g.revision,crate::assistant_memory::now()).is_err());
}
#[test]fn requested_admission_stabilizes_same_signed_choice_and_deadline_across_reopen_without_new_grant(){
 let(f,_authority,g)=setup(Some(Start::WhenAvailable));assert_eq!(g.schema_version,14);assert!(requested(&g));let path=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));let before=std::fs::read(&path).unwrap();let record=serde_json::to_vec(&g).unwrap();
 for _ in 0..2{let ack=native(&f).acknowledge_requested(&g.id,"ai_ollama",&g.revision,crate::assistant_memory::now()).unwrap();assert_eq!(serde_json::to_vec(&ack).unwrap(),record);assert_eq!(std::fs::read(&path).unwrap(),before);}
 let mut older=g.clone();older.schema_version=8;assert!(older.validate().is_err());
}
#[test]fn coherent_json_cannot_promote_manual_consent_to_automatic_execution(){
 let(_f,authority,mut goal)=setup(None);let policy=goal.admissions[0].spec.planning_policy.as_mut().unwrap();policy.request_start(Start::WhenAvailable);goal.schema_version=14;let hash=model::fingerprint(&goal.admissions[0].spec).unwrap();goal.events.last_mut().unwrap().fingerprint=hash;goal.validate().unwrap();assert!(requested(&goal));assert!(authority.verify_queued(&goal,crate::assistant_memory::now()).is_err());assert!(authority.acknowledge_requested(&goal,crate::assistant_memory::now()).is_err());
}
#[test]fn revoked_expired_wrong_owner_and_wrong_revision_requests_cannot_ack_or_reserve_resources(){
 let(f,a,g)=setup(Some(Start::WhenAvailable));let store=native(&f);let now=crate::assistant_memory::now();assert!(store.acknowledge_requested(&g.id,"agent_codex",&g.revision,now).is_err());assert!(store.acknowledge_requested(&g.id,"ai_ollama",&"a".repeat(32),now).is_err());assert!(store.acknowledge_requested(&g.id,"ai_ollama",&g.revision,g.admissions[0].spec.budget.deadline_at).is_err());
 let row=&g.admissions[0];a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,now).unwrap();assert!(store.acknowledge_requested(&g.id,"ai_ollama",&g.revision,now).is_err());
}
#[test]fn replacing_admission_context_or_destination_does_not_retarget_a_native_wait(){
 let(_f,_a,g)=setup(Some(Start::WhenAvailable));assert!(same_admission(&g,&g));
 for field in 0..4{let mut other=g.clone();match field{0=>other.spec.owner.pet_id="agent_codex".into(),1=>other.admissions[0].spec.resolved_model="another-model".into(),2=>other.execution_context.as_mut().unwrap().conversation_id="another-context".into(),_=>other.admissions[0].spec.budget.deadline_at+=1};assert!(!same_admission(&g,&other));}
}
#[test]fn policy_unknown_modes_or_missing_versioned_intent_are_not_ready_and_legacy_reader_refuses_new_field(){
 #[derive(serde::Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct OldPolicy{#[allow(dead_code)]schema_version:u32,#[allow(dead_code)]catalog:crate::pet_runtime::planning::Catalog,#[allow(dead_code)]limits:Limits}
 let(_f,_a,g)=setup(Some(Start::WhenAvailable));let policy=g.admissions[0].spec.planning_policy.as_ref().unwrap();let value=serde_json::to_value(policy).unwrap();assert!(serde_json::from_value::<OldPolicy>(value.clone()).is_err());let mut old=value.clone();old.as_object_mut().unwrap().remove("start");old["schemaVersion"]=serde_json::json!(1);serde_json::from_value::<OldPolicy>(old.clone()).unwrap();let legacy:Policy=serde_json::from_value(old).unwrap();assert!(!legacy.requested());
 let mut wrong=value.clone();wrong["start"]=serde_json::json!("alwaysEvenAfterRevocation");assert!(serde_json::from_value::<Policy>(wrong).is_err());let mut missing=value;missing.as_object_mut().unwrap().remove("start");let missing:Policy=serde_json::from_value(missing).unwrap();assert!(!missing.requested());assert!(!missing.valid_for(&g.admissions[0].spec.budget));
}
#[test]fn an_uncertain_admission_publication_is_acknowledged_without_duplicate_consent(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal("ollama");let now=crate::assistant_memory::now()-100;let mut b=budget();b.deadline_at=now+120_000;b.max_output_bytes=32_768;b.payment=Payment::LocalOnly;
 let approved=Review::prepare_planning(&draft,evidence(&draft,now),b,Limits{max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit:2},&config(),"uncertain-request".into(),nonce().unwrap(),now).unwrap().with_start(Some(Start::WhenAvailable)).unwrap().approve(&draft,evidence(&draft,now+1),now+1).unwrap();let cert=a.certify(&approved,&draft,now+2).unwrap();let mut store=native(&f);store.fault=Fault::AfterPublish;let receipt=store.enqueue_certified(&cert,now+3).unwrap();assert_eq!(receipt.status,CommitStatus::Uncertain);let g=receipt.record;
 let ack=native(&f).acknowledge_requested(&g.id,"ai_ollama",&g.revision,now+4).unwrap();assert_eq!(ack.admissions.len(),1);assert_eq!(ack.revision,g.revision);assert_eq!(ack.admissions[0].spec.budget,g.admissions[0].spec.budget);assert!(ack.admissions[0].lease.is_none());
}

fn waiting(goal:&Record,resource:GoalRequest)->Waiting{Waiting{source:goal.clone(),resource:Some(resource),claim_request:nonce().unwrap(),phase:Phase::WaitingResource,reason:None,advancing:false}}
fn empty_service()->Service{Service{controller:Controller::new().unwrap(),entries:BTreeMap::new(),maintenance_after:None,waiting:BTreeMap::new(),dispatch_after:None}}
fn record_bytes(f:&Fixture,g:&Record)->(Vec<u8>,Vec<u8>){(std::fs::read(f.path.join("goals").join(format!("{}.json",g.id))).unwrap(),std::fs::read(root(f).join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap())}
#[test]fn cancelled_initial_wait_is_collected_without_catalog_io_and_keeps_independent_resources(){
 let(f,_a,g)=setup(Some(Start::WhenAvailable));let c=Arc::new(crate::pet_resources::Coordinator::default());let request=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();request.cancel();let other=c.acquire("copilot",Arc::default(),None).unwrap();let mut service=empty_service();service.waiting.insert(g.id.clone(),waiting(&g,request));let original=record_bytes(&f,&g);let store=native(&f);let lock=store.test_lock(true).unwrap();assert!(matches!(store.list(None),Err(Error::Busy)));
 expire_waits(&mut service,crate::assistant_memory::now());assert!(service.waiting[&g.id].resource.is_none());assert!(matches!(service.waiting[&g.id].phase,Phase::Stopped));assert!(c.acquire("ollama",Arc::default(),None).is_ok());assert!(other.current());assert!(matches!(store.list(None),Err(Error::Busy)));assert_eq!(record_bytes(&f,&g),original);drop(lock);assert_eq!(store.read(&g.id,"ai_ollama").unwrap().revision,g.revision);
}
#[test]fn expired_wait_under_catalog_lock_does_not_free_a_worker_or_cancel_another_queued_owner(){
 let(f,_a,g)=setup(Some(Start::WhenAvailable));let c=Arc::new(crate::pet_resources::Coordinator::default());let request=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let worker=request.poll().unwrap();let unrelated=c.request_goal("lmstudio",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let unrelated_worker=unrelated.poll().unwrap();let mut service=empty_service();service.waiting.insert(g.id.clone(),waiting(&g,request));let original=record_bytes(&f,&g);let store=native(&f);let _lock=store.test_lock(true).unwrap();
 expire_waits(&mut service,g.admissions[0].spec.budget.deadline_at);assert!(service.waiting[&g.id].resource.is_none());assert!(!worker.current());assert!(worker.occupies_slot());assert!(matches!(c.acquire("ollama",Arc::default(),None),Err(ResourceFailure::Busy)));assert!(unrelated_worker.current());assert_eq!(record_bytes(&f,&g),original);drop(worker);assert!(c.acquire("ollama",Arc::default(),None).is_ok());assert!(unrelated.poll().is_ok());
}
#[test]fn original_monotonic_limit_retires_the_wait_even_when_signed_wall_deadline_is_future(){
 let(_f,_a,g)=setup(Some(Start::WhenAvailable));let c=Arc::new(crate::pet_resources::Coordinator::default());let request=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_millis(5)).unwrap();let mut service=empty_service();service.waiting.insert(g.id.clone(),waiting(&g,request));std::thread::sleep(Duration::from_millis(10));let now=crate::assistant_memory::now();assert!(now<g.admissions[0].spec.budget.deadline_at);
 expire_waits(&mut service,now);assert!(service.waiting[&g.id].resource.is_none());assert!(matches!(service.waiting[&g.id].phase,Phase::Stopped));assert!(c.busy().unwrap().is_empty());assert_eq!(model::fingerprint(&service.waiting[&g.id].source.admissions[0].spec).unwrap(),model::fingerprint(&g.admissions[0].spec).unwrap());
}
#[test]fn rollback_and_late_io_results_cannot_recreate_the_original_wait_or_renew_its_authority(){
 let(f,_a,g)=setup(Some(Start::WhenAvailable));let c=Arc::new(crate::pet_resources::Coordinator::default());let request=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let mut service=empty_service();service.waiting.insert(g.id.clone(),waiting(&g,request));let original=record_bytes(&f,&g);let store=native(&f);let _lock=store.test_lock(true).unwrap();expire_waits(&mut service,g.admissions[0].spec.approved_at-1);
 prune_expired_statuses(&mut service,g.admissions[0].spec.approved_at);assert!(service.waiting.contains_key(&g.id));
 let w=service.waiting.get_mut(&g.id).unwrap();for(phase,reason,halt)in[(Phase::WaitingResource,None,false),(Phase::ConfirmingClaim,None,false),(Phase::RecoveryRequired,Some(Error::Busy),true)]{apply_update(w,phase,reason,halt);assert!(matches!(w.phase,Phase::Stopped));assert_eq!(w.reason,Some(Error::ClockRollback));assert!(w.resource.is_none());}
 assert_eq!(record_bytes(&f,&g),original);assert_eq!(w.source.execution_context,g.execution_context);assert_eq!(model::fingerprint(&w.source.admissions[0].spec).unwrap(),model::fingerprint(&g.admissions[0].spec).unwrap());assert!(c.busy().unwrap().is_empty());
}
#[test]fn waiting_cleanup_preserves_a_prepared_native_entry_and_its_hold_until_owner_collection(){
 let(f,_a,g)=setup(Some(Start::WhenAvailable));let main=crate::chat::Chat::persistent(f.path.join("pending-main.json"));let c=main.runtime_resources();let resource=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let mut service=empty_service();let request=nonce().unwrap();let now=crate::assistant_memory::now();
 let pending=service.controller.prepare_requested(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,now),request.clone(),now,Arc::new(|_,_,_|Validity::Current),resource.poll().unwrap()).unwrap();let lease=pending.lease().clone();service.entries.insert(g.id.clone(),Entry{source_revision:g.revision.clone(),request_id:request,pending,owned:None});service.waiting.insert(g.id.clone(),waiting(&g,resource));let original=record_bytes(&f,&g);let store=native(&f);let _lock=store.test_lock(true).unwrap();
 expire_waits(&mut service,g.admissions[0].spec.budget.deadline_at);assert!(service.waiting[&g.id].resource.is_none());assert_eq!(service.entries.len(),1);assert_eq!(service.entries[&g.id].pending.lease(),&lease);assert_eq!(service.entries[&g.id].pending.workers(),0);assert!(service.entries[&g.id].pending.hold().occupies_slot());assert!(matches!(c.acquire("ollama",Arc::default(),None),Err(ResourceFailure::Busy)));assert_eq!(record_bytes(&f,&g),original);
 drop(service.entries.remove(&g.id));assert!(c.acquire("ollama",Arc::default(),None).is_ok());
}

#[test]fn sixty_four_expired_signed_waits_cannot_starve_a_new_admission_or_return_to_queued_status(){
 let mut fixtures=Vec::new();let mut service=empty_service();let mut last=None;let mut expiry=0;
 for index in 0..64{
  let(f,a,g)=setup_named(Some(Start::WhenAvailable),&format!("capacity-{index}"));expiry=expiry.max(g.admissions[0].spec.budget.deadline_at);
  service.waiting.insert(g.id.clone(),Waiting{source:g.clone(),resource:None,claim_request:nonce().unwrap(),phase:Phase::Stopped,reason:None,advancing:false});last=Some(g);fixtures.push((f,a));
 }
 assert_eq!(service.waiting.len(),64);let g=last.unwrap();let original=record_bytes(&fixtures.last().unwrap().0,&g);let store=native(&fixtures.last().unwrap().0);let _lock=store.test_lock(true).unwrap();prune_expired_statuses(&mut service,expiry);assert!(service.waiting.is_empty());
 let status=status_at(&service,&g,expiry).unwrap();assert!(matches!(status.phase,Phase::Stopped));assert!(status.resource_wait.is_none());assert_eq!(record_bytes(&fixtures.last().unwrap().0,&g),original);
}

#[path="requested_finite_tests.rs"]mod finite;
