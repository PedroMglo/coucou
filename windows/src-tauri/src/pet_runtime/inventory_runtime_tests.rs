//! Common runtime, real bounded host reads, synthetic provider/consent metadata.
use super::{model::*,admission::{Review,Budget,Payment},admission_tests::Fixture,authority::Authority,lease_controller::{Controller,Validity},store::nonce,criteria,physical_target::{Target,Action as PhysicalAction},recipes};
use std::{fs,sync::Arc};
use serde_json::{json,Value};
fn config(pet:&str)->crate::pet_config::Configuration{
 let mut c=crate::pet_config::defaults();c.version=2;c.pets.get_mut(pet).unwrap().system_access=Some(serde_json::from_value(json!({"schemaVersion":1,"revision":"a".repeat(32),"enabled":true,"grants":[{"subplugin":"inventory","operation":"observe","resource":{"kind":"hostComponents"}}]})).unwrap());c
}
fn save(f:&Fixture,c:&crate::pet_config::Configuration){super::native_system_admission_tests::save(f,c)}
fn evidence(g:&Record,c:&crate::pet_config::Configuration)->super::admission::Evidence{super::native_system_admission_tests::evidence(g,c,crate::assistant_memory::now())}
fn draft(f:&Fixture,pet:&str)->Record{draft_kind(f,pet,"system.inventory.observe",vec!["Inventário limitado com origem e cobertura completas.".into()])}
fn draft_kind(f:&Fixture,pet:&str,operation:&str,criteria:Vec<String>)->Record{
 let b=crate::pet_config::blueprint(pet).unwrap();let provider=b.routing.task_provider.as_deref().or(b.routing.chat.as_deref()).unwrap_or(pet);
 let at=crate::assistant_memory::now();let g=f.store().create(Spec{received_from:None,restored_from:None,request_id:"inventory-goal".into(),owner:Owner{pet_id:pet.into(),provider:provider.into(),adapter:b.runtime.adapter.map(|a|a.key()).unwrap_or(provider).into(),model_hint:None},context:Context::Unbound,intent:"Consultar apenas distribuição e kernel autorizados, sem concluir estado desejado ou segurança.".into(),criteria},at).unwrap().record;
 let(binding,guard)=super::context::prepare(&f.path.join("contexts"),&g,&crate::chat::Chat::persistent(f.path.join("preparation.json")),PreparationMode::Dedicated,"inventory-context").unwrap();drop(guard);
 let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"inventory-context".into(),Action::BindContext{binding},crate::assistant_memory::now()).unwrap().record;
 let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"inventory-task".into(),Action::ProposeTask{description:"Identificação de dois componentes".into()},crate::assistant_memory::now()).unwrap().record;
 f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"inventory-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Consultar distribuição e kernel".into(),postcondition:"Origem e cobertura completas, sem diagnóstico.".into(),operation:Some(super::plans::OperationProposal{operation_id:operation.into(),version:1,arguments:json!({})})}]},crate::assistant_memory::now()).unwrap().record
}
fn target(f:&Fixture,pet:&str)->Target{Target::inventory_fixture(&f.path.join("system.json"),pet).unwrap()}
fn review(f:&Fixture,g:&Record,c:&crate::pet_config::Configuration,predicate:criteria::Predicate)->super::admission::Result<Review>{
 let now=crate::assistant_memory::now();let path=f.path.join("system.json");let observe=Arc::new(move||serde_json::from_slice(&fs::read(&path).map_err(|_|Error::StorageUnavailable)?).map_err(|_|Error::InvalidInput));
 Review::prepare_system(g,evidence(g,c),Budget{deadline_at:now+60_000,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly},target(f,&g.spec.owner.pet_id),observe,&[criteria::Input{criterion_index:0,predicate}],"inventory-admission".into(),nonce().unwrap(),now)
}
#[test]fn inventory_runtime_original_queue_recipe_and_signed_receipt_work_for_all_declared_owner_routes(){
 for pet in ["agent_system_guardian","ai_ollama","agent_codex","agent_antigravity","agent_copilot","ai_lmstudio","ai_openai","integration_spotify"]{
  let f=Fixture::new();let c=config(pet);save(&f,&c);let a=Authority::ensure(&f.path.join("authority")).unwrap();let g=draft(&f,pet);let r=review(&f,&g,&c,criteria::Predicate::SystemHostInventoryObserved).unwrap();let proposal=r.proposal();let policy=proposal.native_system_plan.unwrap();assert_eq!(policy.schema_version,3);assert!(policy.output.is_none()&&policy.health.is_none()&&policy.inventory.is_some());assert!(proposal.resolved_model.is_empty()&&proposal.engine_binding.is_none());
  let approved=r.approve(&g,evidence(&g,&c),crate::assistant_memory::now()).unwrap();let cert=a.certify(&approved,&g,crate::assistant_memory::now()).unwrap();let q=f.store().enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record;assert_eq!(q.schema_version,63);assert_eq!(q.admissions[0].spec.schema_version,28);
  let chat=crate::chat::Chat::persistent(f.path.join("main.json"));let pending=Controller::new().unwrap().prepare(&q,&f.path.join("authority"),&f.path.join("contexts"),&chat,&evidence(&q,&c),"inventory-claim".into(),crate::assistant_memory::now(),Arc::new(|_,_,_|Validity::Current)).unwrap();let(receipt,owned)=pending.claim(&f.store(),&a,&evidence(&q,&c),crate::assistant_memory::now()).unwrap();let g=receipt.record;let o=owned.unwrap();
  let retained=target(&f,pet);let probe=retained.clone();let review=o.review_recipe_with_criteria(&g,&g.plans[0].id,"inventory-recipe",retained,&c,&[],crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(&o,&g,&c,crate::assistant_memory::now()).unwrap();assert_eq!(probe.collector_calls(),0,"no component IO before original tool intent/reservation");let row=&g.admissions[0];let scope=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert_eq!(scope.schema_version,6);assert_eq!(scope.steps[0].schema_version,5);assert!(scope.steps[0].resource.inventory().is_some());ready.activate(&scope).unwrap();
  let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(c.clone()),|_|{}).unwrap();assert_eq!(result.outcome,recipes::Outcome::GoalCompleted);let stored=f.store().read(&g.id,pet).unwrap();a.verify_completed_goal(&stored).unwrap();assert_eq!(result.progress.verification.as_ref().unwrap().schema_version,4);
  let original=&result.progress.steps[0].result.as_ref().unwrap().observation.after;assert!(matches!(original,Some(super::physical_target::Snapshot::Inventory(s))if s.complete()));assert_eq!(serde_json::to_value(original).unwrap(),serde_json::to_value(&result.progress.verification.as_ref().unwrap().snapshot).unwrap(),"verification must retain original signed inventory, never resample");
  let file=f.path.join("authority").join(format!("{}.json",row.spec.decision_id));let before=fs::read(&file).unwrap();let body:Value=serde_json::from_slice(&before).unwrap();assert_eq!(body["body"]["schemaVersion"],48);assert_eq!(body["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(body["body"]["budgetUsage"]["attempts"][0]["kind"],"tool");assert!(!f.path.join("main.json").exists());
  o.run_recipe_for_store(&f.store(),&stored,&ready,||Ok(c.clone()),|_|{}).unwrap();assert_eq!(fs::read(file).unwrap(),before);assert_eq!(o.workers(),0);assert_eq!(probe.collector_calls(),1,"effect once, verification/replay must not resample");a.verify_completed_goal(&f.store().read(&g.id,pet).unwrap()).unwrap();
 }
}
#[test]fn inventory_runtime_exact_operation_criterion_owner_host_and_grant_cannot_be_substituted(){
 let f=Fixture::new();let c=config("agent_system_guardian");save(&f,&c);let g=draft(&f,"agent_system_guardian");
 assert!(review(&f,&g,&c,criteria::Predicate::SystemHealthObserved).is_err());assert!(review(&f,&g,&c,criteria::Predicate::SystemHealthBaselineMeasured).is_err());
 let r=review(&f,&g,&c,criteria::Predicate::SystemHostInventoryObserved).unwrap();let policy=r.proposal().native_system_plan.unwrap();let t=target(&f,"agent_system_guardian");assert!(policy.valid()&&policy.target_matches(&t));
 let mut wrong=policy.clone();wrong.inventory.as_mut().unwrap().host.pid_namespace+=1;assert!(wrong.valid()&&!wrong.target_matches(&t));
 let mut wrong=policy.clone();wrong.inventory.as_mut().unwrap().pet_id="ai_ollama".into();assert!(!wrong.valid());
 let mut wrong=policy.clone();wrong.steps[0].operation.operation_id="system.health.observe".into();assert!(!wrong.valid());
 for args in [json!({"petId":"ai_ollama"}),json!({"path":"/etc/shadow"}),json!([]),json!(null)]{assert!(!crate::pet_host_inventory::validate("system.inventory.observe",1,&args));}
 assert!(crate::pet_operations::validate_arguments("system.inventory.observe",&json!({})).is_err(),"not a model tool");assert!(!t.permits(&PhysicalAction::Health)&&!t.permits(&PhysicalAction::HealthBaseline));assert!(Target::health_fixture(&f.path.join("system.json"),"agent_system_guardian").is_err());
 let mut revoked=c.clone();revoked.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().enabled=false;save(&f,&revoked);assert!(r.approve(&g,evidence(&g,&c),crate::assistant_memory::now()).is_err());assert!(t.read().is_err());save(&f,&c);assert!(t.read().is_err(),"an invalidated retained scope cannot revive");assert!(target(&f,"agent_system_guardian").read().is_ok());
}
#[test]fn inventory_runtime_original_intent_and_live_scope_precede_component_io(){
 for revoke in [false,true]{let f=Fixture::new();let c=config("agent_system_guardian");save(&f,&c);let t=target(&f,"agent_system_guardian");let live=std::cell::Cell::new(true);let called=std::cell::Cell::new(0);
  let result=t.apply(&PhysicalAction::Inventory,std::time::Instant::now()+std::time::Duration::from_secs(2),||live.get(),||{called.set(called.get()+1);if revoke{let mut changed=c.clone();changed.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().revision="b".repeat(32);save(&f,&changed);}else{live.set(false);}Ok(())});
  assert_eq!(called.get(),1);assert!(!result.effect_verified&&!result.scope_current&&result.after.is_none());assert_eq!(result.delivery,super::physical_target::Delivery::NotDispatched);
 }
 let f=Fixture::new();let c=config("agent_system_guardian");save(&f,&c);let t=target(&f,"agent_system_guardian");let result=t.apply(&PhysicalAction::Inventory,std::time::Instant::now()+std::time::Duration::from_secs(2),||true,||Err(super::physical_target::Failure::Unavailable));assert!(result.after.is_none()&&!result.effect_verified);
}

#[test]fn inventory_runtime_preserves_legacy_health_multiple_criteria_and_omits_new_policy_field(){
 let f=Fixture::new();let mut c=config("agent_system_guardian");c.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().grants=vec![serde_json::from_value(json!({"subplugin":"health","operation":"observe","resource":{"kind":"hostHealth"}})).unwrap()];save(&f,&c);
 let g=draft_kind(&f,"agent_system_guardian","system.health.observe",vec!["Amostra válida.".into(),"Cobertura completa.".into()]);let path=f.path.join("system.json");let observe=Arc::new(move||serde_json::from_slice(&fs::read(&path).map_err(|_|Error::StorageUnavailable)?).map_err(|_|Error::InvalidInput));let now=crate::assistant_memory::now();
 let r=Review::prepare_system(&g,evidence(&g,&c),Budget{deadline_at:now+60_000,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly},Target::health_fixture(&f.path.join("system.json"),"agent_system_guardian").unwrap(),observe,&[criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemHealthObserved},criteria::Input{criterion_index:1,predicate:criteria::Predicate::SystemHealthObserved}],"legacy-health-admission".into(),nonce().unwrap(),now).unwrap();
 let p=r.proposal();assert_eq!(p.native_system_plan.as_ref().unwrap().schema_version,2);assert_eq!(p.native_verification.as_ref().unwrap().criteria_count,2);assert!(serde_json::to_value(&p.native_system_plan).unwrap().get("inventory").is_none());
 let a=Authority::ensure(&f.path.join("authority")).unwrap();let approved=r.approve(&g,evidence(&g,&c),crate::assistant_memory::now()).unwrap();let cert=a.certify(&approved,&g,crate::assistant_memory::now()).unwrap();let stored=f.store().enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record;assert_eq!(stored.schema_version,63);a.verify_goal_origin(&stored).unwrap();
}

#[path="system_requested_execution_tests.rs"]mod requested_execution_tests;
