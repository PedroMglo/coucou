//! Original signed runtime; fixed read-only host counters, no inference/audio/network.
use super::{model::*,admission::{Review,Budget,Payment},admission_tests::Fixture,authority::Authority,lease_controller::{Controller,Validity},store::nonce,criteria,physical_target::Target,recipes};
use std::{fs,sync::Arc};
use serde_json::{json,Value};
fn config()->crate::pet_config::Configuration{
 let mut c=crate::pet_config::defaults();c.version=2;
 c.pets.get_mut("agent_system_guardian").unwrap().system_access=Some(serde_json::from_value(json!({"schemaVersion":1,"revision":"a".repeat(32),"enabled":true,"grants":[{"subplugin":"health","operation":"observe","resource":{"kind":"hostHealth"}}]})).unwrap());c
}
fn save(f:&Fixture,c:&crate::pet_config::Configuration){super::native_system_admission_tests::save(f,c)}
fn evidence(g:&Record,c:&crate::pet_config::Configuration)->super::admission::Evidence{super::native_system_admission_tests::evidence(g,c,crate::assistant_memory::now())}
fn draft(f:&Fixture)->Record{draft_kind(f,false)}
fn draft_kind(f:&Fixture,baseline:bool)->Record{
 let at=crate::assistant_memory::now();let g=f.store().create(Spec{received_from:None,restored_from:None,request_id:"health-goal".into(),owner:Owner{pet_id:"agent_system_guardian".into(),provider:"agent_system_guardian".into(),adapter:"agent_system_guardian".into(),model_hint:None},context:Context::Unbound,intent:"Recolher uma amostra limitada, sem concluir segurança nem otimizar o anfitrião.".into(),criteria:vec![if baseline{"Medição válida de CPU e memória com intervalo mínimo de1s.".into()}else{"Uma amostra válida dos três contadores.".into()}]},at).unwrap().record;
 let(b,guard)=super::context::prepare(&f.path.join("contexts"),&g,&crate::chat::Chat::persistent(f.path.join("preparation.json")),PreparationMode::Dedicated,"health-context").unwrap();drop(guard);
 let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"health-context".into(),Action::BindContext{binding:b},crate::assistant_memory::now()).unwrap().record;
 let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"health-task".into(),Action::ProposeTask{description:"Amostra local".into()},crate::assistant_memory::now()).unwrap().record;
 f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"health-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Consultar três contadores".into(),postcondition:"Amostra verificável".into(),operation:Some(super::plans::OperationProposal{operation_id:if baseline{"system.health.baseline".into()}else{"system.health.observe".into()},version:1,arguments:json!({})})}]},crate::assistant_memory::now()).unwrap().record
}
fn target(f:&Fixture)->Target{Target::health_fixture(&f.path.join("system.json"),"agent_system_guardian").unwrap()}
fn inputs()->Vec<criteria::Input>{vec![criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemHealthObserved}]}
fn review(f:&Fixture,g:&Record,c:&crate::pet_config::Configuration)->Review{
 let input=if g.plans[0].steps[0].operation.as_ref().unwrap().operation_id=="system.health.baseline"{vec![criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemHealthBaselineMeasured}]}else{inputs()};
 let now=crate::assistant_memory::now();let path=f.path.join("system.json");let observe=Arc::new(move||serde_json::from_slice(&fs::read(&path).map_err(|_|Error::StorageUnavailable)?).map_err(|_|Error::InvalidInput));
 Review::prepare_system(g,evidence(g,c),Budget{deadline_at:now+60_000,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly},target(f),observe,&input,"health-admission".into(),nonce().unwrap(),now).unwrap()
}
#[test]fn health_runtime_original_queue_authority_recipe_completes_without_model_and_replays_read_only(){
 for baseline in [false,true]{
 let f=Fixture::new();let c=config();save(&f,&c);let a=Authority::ensure(&f.path.join("authority")).unwrap();let g=draft_kind(&f,baseline);let r=review(&f,&g,&c);let p=r.proposal();let policy=p.native_system_plan.unwrap();assert_eq!(policy.schema_version,2);assert!(policy.output.is_none()&&policy.health.is_some());assert_eq!(p.native_verification.unwrap().schema_version,7);
 let approved=r.approve(&g,evidence(&g,&c),crate::assistant_memory::now()).unwrap();let cert=a.certify(&approved,&g,crate::assistant_memory::now()).unwrap();let q=f.store().enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record;assert_eq!(q.schema_version,63);assert_eq!(q.admissions[0].spec.schema_version,28);
 let chat=crate::chat::Chat::persistent(f.path.join("main.json"));let pending=Controller::new().unwrap().prepare(&q,&f.path.join("authority"),&f.path.join("contexts"),&chat,&evidence(&q,&c),"health-claim".into(),crate::assistant_memory::now(),Arc::new(|_,_,_|Validity::Current)).unwrap();let(receipt,owned)=pending.claim(&f.store(),&a,&evidence(&q,&c),crate::assistant_memory::now()).unwrap();let g=receipt.record;let o=owned.unwrap();
 let retained=target(&f);let probe=retained.clone();let review=o.review_recipe_with_criteria(&g,&g.plans[0].id,"health-recipe",retained,&c,&[],crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(&o,&g,&c,crate::assistant_memory::now()).unwrap();if baseline{assert_eq!(probe.collector_calls(),0,"baseline readiness must not sample before tool intent");}let row=&g.admissions[0];let scope=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert_eq!(scope.schema_version,6);assert_eq!(scope.steps[0].schema_version,5);ready.activate(&scope).unwrap();
 let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(c.clone()),|_|{}).unwrap();assert_eq!(result.outcome,recipes::Outcome::GoalCompleted);let stored=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();a.verify_completed_goal(&stored).unwrap();assert_eq!(result.progress.verification.as_ref().unwrap().schema_version,4);
 if baseline{
  let original=&result.progress.steps[0].result.as_ref().unwrap().observation.after;
  assert_eq!(serde_json::to_value(original).unwrap(),serde_json::to_value(&result.progress.verification.as_ref().unwrap().snapshot).unwrap(),"verifier must retain the signed original sample, never resample");
  assert!(matches!(original,Some(super::physical_target::Snapshot::HealthBaseline(b))if b.measured()&&b.interval_ms>=1000));
 }

 let file=f.path.join("authority").join(format!("{}.json",row.spec.decision_id));let before=fs::read(&file).unwrap();let body:Value=serde_json::from_slice(&before).unwrap();assert_eq!(body["body"]["schemaVersion"],48);assert_eq!(body["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(body["body"]["budgetUsage"]["attempts"][0]["kind"],"tool");assert!(!f.path.join("main.json").exists());
 o.run_recipe_for_store(&f.store(),&stored,&ready,||Ok(c.clone()),|_|{}).unwrap();assert_eq!(fs::read(file).unwrap(),before);assert_eq!(o.workers(),0);if baseline{assert_eq!(probe.collector_calls(),1,"baseline effect once; no verifier or replay sample");}
 }
}
#[test]fn health_runtime_closed_arguments_exact_target_and_sticky_revocation(){
 let f=Fixture::new();let c=config();save(&f,&c);let t=target(&f);let g=draft(&f);let r=review(&f,&g,&c);let mut policy=r.proposal().native_system_plan.unwrap();assert!(policy.valid());let original=policy.clone();let mut changed_host=original.clone();changed_host.health.as_mut().unwrap().host.pid_namespace+=1;assert!(changed_host.valid()&&!changed_host.target_matches(&t));policy.health.as_mut().unwrap().pet_id="ai_ollama".into();assert!(!policy.valid());policy=original.clone();policy.health.as_mut().unwrap().host.boot_id="b".repeat(36);assert!(!policy.valid());
 for args in [json!({"petId":"ai_ollama"}),json!({"path":"/etc/shadow"}),json!([]),json!(null)]{assert!(!crate::pet_health::task::validate("system.health.observe",1,&args));}
 assert!(crate::pet_operations::validate_arguments("system.health.observe",&json!({})).is_err(),"no provider model tool");
 let mut revoked=c.clone();revoked.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().enabled=false;save(&f,&revoked);assert!(t.read().is_err());assert!(r.approve(&g,evidence(&g,&c),crate::assistant_memory::now()).is_err());save(&f,&c);assert!(t.read().is_err(),"old retained scope cannot revive");assert!(target(&f).read().is_ok());
}

#[test]fn health_runtime_original_scope_is_rechecked_after_intent_and_cannot_borrow_sound(){
 let f=Fixture::new();let c=config();save(&f,&c);let t=target(&f);
 let a=super::physical_target::Action::Health;
 let obs=t.apply(&a,std::time::Instant::now()+std::time::Duration::from_secs(1),||true,||{let mut revoked=c.clone();revoked.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().revision="b".repeat(32);save(&f,&revoked);Ok(())});
 assert!(!obs.effect_verified&&obs.after.is_none());assert_eq!(obs.delivery,super::physical_target::Delivery::NotDispatched);
 assert!(!t.permits(&super::physical_target::Action::Sound(crate::pet_audio::task::Action::parse("system.sound.mute",1,&json!({"targetKey":"audio-0123456789abcdef","muted":true})).unwrap())));
}

#[test]fn health_runtime_bounded_native_authority_checks_can_exceed_the_manual_window(){
 let binding=crate::pet_health::task::Binding{schema_version:1,kind:crate::pet_health::task::Kind::SystemHostHealth,pet_id:"agent_system_guardian".into(),grant_revision:"a".repeat(32),host:crate::pet_health::task::Host::capture().unwrap()};
 let evidence=crate::pet_health::task::sample(&binding,&||{std::thread::sleep(std::time::Duration::from_millis(60));true}).unwrap();assert!(evidence.complete());assert!(evidence.sample_duration_ms>=250&&evidence.sample_duration_ms<2000);
 assert!(crate::pet_health::task::sample(&binding,&||false).is_err());
}

#[test]fn health_baseline_native_operation_cancel_or_revoke_withholds_publication(){
 for revoke in [false,true]{
  let f=Fixture::new();let c=config();save(&f,&c);let t=target(&f);let flag=Arc::new(std::sync::atomic::AtomicBool::new(true));let control=flag.clone();let(sender,started)=std::sync::mpsc::channel();
  let task=std::thread::spawn(move||t.apply(&super::physical_target::Action::HealthBaseline,std::time::Instant::now()+std::time::Duration::from_secs(6),||control.load(std::sync::atomic::Ordering::Acquire),||{sender.send(()).unwrap();Ok(())}));
  started.recv_timeout(std::time::Duration::from_secs(2)).unwrap();std::thread::sleep(std::time::Duration::from_millis(100));
  if revoke{let mut changed=c.clone();changed.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().revision="b".repeat(32);save(&f,&changed);}else{flag.store(false,std::sync::atomic::Ordering::Release);}
  let result=task.join().unwrap();assert!(!result.effect_verified&&!result.scope_current&&result.after.is_none());
 }
}
