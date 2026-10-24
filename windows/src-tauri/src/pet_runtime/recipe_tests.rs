//! Whole native reviewed sequences on a private player/bus, without models.
use super::*;
use crate::pet_runtime::{recipes::{Phase as RecipePhase,Outcome},lease_controller::recipe_controller::{Ready as RecipeReady,Approved as RecipeApproved},recipe_api::*};
#[path="criteria_tests.rs"]mod criteria_tests;
fn proposed(action:Value)->StepInput{StepInput{description:"Reviewed native action".into(),postcondition:"Native state independently observed".into(),operation:Some(plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:action})}}
fn sequence()->Vec<StepInput>{vec![proposed(json!({"action":"pause"})),proposed(json!({"action":"set_volume","volume":17}))]}
fn reviewed(o:&Owned,g:&Record,bus:&Bus)->(RecipeApproved,RecipeReady){
    let review=o.review_recipe(g,&g.plans[0].id,"recipe-once",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();
    let p=review.proposal();assert_eq!(p.steps.len(),2);assert!(!p.criteria_verified&&!p.goal_complete);assert_eq!(p.steps[0].scope.max_output_bytes,2048);assert_eq!(p.steps[1].scope.max_output_bytes,2048);
    review.approve(o,g,&config(),crate::assistant_memory::now()).unwrap()
}
fn register(a:&Authority,g:&Record,o:&Owned,approved:&RecipeApproved,ready:&RecipeReady){let row=&g.admissions[0];let s=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),approved,crate::assistant_memory::now()).unwrap();ready.activate(&s).unwrap();}
fn progress(a:&Authority,g:&Record)->crate::pet_runtime::recipes::Progress{let row=&g.admissions[0];a.recipe_progress(&row.spec,row.proof.as_ref().unwrap(),&g.plans[0].id).unwrap().unwrap()}
fn recipe_path(f:&Fixture,ready:&RecipeReady)->std::path::PathBuf{f.path.join("pet-runtime-step-consent-v1").join(format!("{}.recipe.json",ready.scope().slot().unwrap()))}

#[test]
fn one_review_runs_the_exact_ordered_plan_and_repeat_requests_only_return_verified_receipts(){
    let(f,a,g,_p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);
    assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());register(&a,&g,&o,&approved,&ready);
    let mut notices=vec![];let result=o.run_recipe(&g,&ready,||Ok(config()),|p|notices.push((p.phase,p.verified_steps))).unwrap();
    assert_eq!(result.outcome,Outcome::Verified);assert_eq!(result.progress.phase,RecipePhase::Verified);assert_eq!(result.progress.verified_steps,2);assert!(!result.progress.criteria_verified&&!result.progress.goal_complete&&!result.progress.execution_available);
    assert_eq!(notices,vec![(RecipePhase::Reviewed,0),(RecipePhase::Partial,1),(RecipePhase::Verified,2)]);let calls=player.calls.lock().unwrap().clone();assert_eq!(calls.iter().map(|c|c.0.as_str()).collect::<Vec<_>>(),vec!["Pause","Set"]);assert!(calls.iter().all(|c|c.1==ready.scope().steps[0].resource.owner));
    let usage=ledger(&f,&g);assert_eq!(usage["attempts"].as_array().unwrap().len(),2);assert!(usage["attempts"].as_array().unwrap().iter().all(|a|a["kind"]=="tool"&&a["phase"]=="settled"));assert_eq!(o.workers(),0);assert_eq!(main.runtime_resources().busy().unwrap().len(),1);
    signal_stop(o.lease());assert_eq!(o.run_recipe(&g,&ready,||panic!("completed retry never loads configuration"),|_|{}).unwrap().outcome,Outcome::Verified);assert_eq!(ledger(&f,&g),usage);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);
}

#[test]
fn native_approval_and_even_a_readable_uncertain_manifest_cannot_dispatch_before_registration_ack(){
    for fault in [Fault::BeforeRecipePublish,Fault::AfterRecipePublish]{
        let(f,mut a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);let row=&g.admissions[0];a.fault=fault;
        assert!(a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).is_err());assert!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
        a.fault=Fault::None;register(&a,&g,&o,&approved,&ready);let before=fs::read(recipe_path(&f,&ready)).unwrap();register(&a,&g,&o,&approved,&ready);assert_eq!(fs::read(recipe_path(&f,&ready)).unwrap(),before);assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::Verified);assert_eq!(player.calls.lock().unwrap().len(),2);
    }
}

#[test]
fn insufficient_calls_or_output_and_unsupported_operations_are_refused_before_approval_or_effects(){
    for(mode,steps,tools)in[(0,sequence(),1),(1,vec![proposed(json!({"action":"pause"}));3],3),(2,vec![proposed(json!({"action":"pause"})),proposed(json!({"action":"next"}))],2)]{
        let(f,_a,g,_p,o,_)=setup_plan_custom(steps,tools,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");
        assert!(o.review_recipe(&g,&g.plans[0].id,"recipe",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).is_err(),"mode {mode}");assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    }
}

#[test]
fn a_changed_plan_or_expired_native_review_never_becomes_a_recipe_permission(){
    for changed in [false,true]{let(f,_a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let review=o.review_recipe(&g,&g.plans[0].id,"recipe",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let expiry=review.proposal().expires_at;
        let mut latest=g.clone();if changed{latest.plans[0].steps[1].operation.as_mut().unwrap().arguments["volume"]=json!(99);}
        assert!(review.approve(&o,&latest,&config(),if changed{crate::assistant_memory::now()}else{expiry}).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());}
}

#[test]
fn the_physical_hold_spans_the_whole_sequence_and_other_resources_remain_available(){
    let(_f,a,g,_p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let _player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let key=crate::pet_runtime::lease_controller::step_controller::resource_key(&ready.scope().steps[0].resource).unwrap();let coordinator=main.runtime_resources();let mut between=false;
    assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|p|if p.verified_steps==1{between=true;assert!(o.workers()>=1);assert!(coordinator.acquire(&key,Arc::new(AtomicBool::new(false)),None).is_err());let unrelated=coordinator.acquire("unrelated_resource",Arc::new(AtomicBool::new(false)),None).unwrap();drop(unrelated);}).unwrap().outcome,Outcome::Verified);assert!(between);assert!(coordinator.acquire(&key,Arc::new(AtomicBool::new(false)),None).is_ok());
}

#[test]
fn unverified_effect_stops_the_recipe_before_the_next_step_and_never_replays(){
    let(f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);player.mode.store(1,Ordering::Release);
    let first=o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(first.outcome,Outcome::RecoveryRequired);assert_eq!(first.progress.phase,RecipePhase::RecoveryRequired);assert_eq!(first.progress.verified_steps,0);assert!(first.progress.steps[1].phase.is_none());let usage=ledger(&f,&g);
    player.mode.store(0,Ordering::Release);assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::RecoveryRequired);assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g),usage);
}

#[test]
fn revoked_policy_between_steps_prevents_the_remaining_effect_without_reapproving_or_charging_it(){
    let allowed=Arc::new(AtomicBool::new(true));let gate=allowed.clone();let(f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(move|_,_,_|if gate.load(Ordering::Acquire){Validity::Current}else{Validity::Changed}));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe(&g,&ready,||Ok(config()),|p|if p.verified_steps==1{allowed.store(false,Ordering::Release)}).unwrap();assert_eq!(result.outcome,Outcome::Invalidated);assert_eq!(result.progress.phase,RecipePhase::Partial);assert_eq!(result.progress.verified_steps,1);assert!(result.progress.steps[1].phase.is_none());assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
}

#[test]
fn concurrent_requests_and_stop_retain_worker_and_physical_ownership_until_native_io_drains(){
    let(_f,a,g,_p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");player.delay.store(250,Ordering::Release);let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let owner=Arc::new(o);let ready=Arc::new(ready);let running=owner.clone();let selected=ready.clone();let goal=g.clone();
    let worker=thread::spawn(move||running.run_recipe(&goal,&selected,||Ok(config()),|_|{}));let until=Instant::now()+Duration::from_secs(3);while !player.entered.load(Ordering::Acquire){assert!(Instant::now()<until);thread::sleep(Duration::from_millis(2));}
    assert!(matches!(owner.run_recipe(&g,&ready,||Ok(config()),|_|{}),Err(crate::pet_runtime::model::Error::Busy)));assert_eq!(main.runtime_resources().busy().unwrap().len(),2);assert!(owner.drain(DrainOutcome::Confirmed).is_err());assert_eq!(main.runtime_resources().busy().unwrap().len(),2);
    assert_eq!(worker.join().unwrap().unwrap().outcome,Outcome::RecoveryRequired);assert_eq!(owner.workers(),0);assert_eq!(main.runtime_resources().busy().unwrap().len(),1);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn a_missing_final_receipt_requires_explicit_recovery_then_continues_without_repeating_the_first_effect(){
    let(f,mut a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);a.fault=Fault::BeforeStepResultPublish;
    let stopped=o.run_recipe_with_authority(&g,&ready,&a,||Ok(config()),|_|{}).unwrap();assert_eq!(stopped.outcome,Outcome::RecoveryRequired);assert_eq!(player.calls.lock().unwrap().len(),1);assert!(stopped.progress.steps[0].recovery.as_ref().unwrap().reconciliation_available);a.fault=Fault::None;
    assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::RecoveryRequired);let row=&g.admissions[0];a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&ready.scope().steps[0],crate::assistant_memory::now()).unwrap();
    assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::Verified);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
}

#[test]
fn an_existing_step_grant_or_a_replaced_native_target_cannot_be_borrowed_by_a_new_recipe(){
    let(f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let _existing=grant(&a,&g,&o,&bus);
    assert!(o.review_recipe(&g,&g.plans[0].id,"recipe",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    let(_f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);drop(player);let replacement=Player::new(&bus,"Private recipe player");
    assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::RecoveryRequired);assert!(replacement.calls.lock().unwrap().is_empty());assert!(progress(&a,&g).steps[1].phase.is_none());
}

#[test]
fn tampered_recipe_data_and_forged_ipc_fields_never_create_execution_authority(){
    let(f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let file=recipe_path(&f,&ready);let mut body:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();body["body"]["steps"][0]["argumentsHash"]=json!("0".repeat(64));fs::write(&file,serde_json::to_vec(&body).unwrap()).unwrap();assert!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    assert!(serde_json::from_value::<RecipePrepareInput>(json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","revision":"revision","planId":"plan","requestId":"review","steps":[]})).is_err());
    assert!(serde_json::from_value::<RecipeConfirmInput>(json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","revision":"revision","token":"token","approve":true,"scope":{"allow":true}})).is_err());
    assert!(serde_json::from_value::<RecipeRunInput>(json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","revision":"revision","recipeId":"recipe","goalComplete":true})).is_err());
    assert!(serde_json::from_value::<RecipeReadInput>(json!({"petId":"ai_ollama","goalId":"goal","decisionId":"decision","planId":"plan","run":true})).is_err());
}

#[test]
fn durable_recipe_progress_survives_a_new_process_without_recreating_a_controller(){
    let(f,a,g,p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recipe player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::Verified);assert_eq!(player.calls.lock().unwrap().len(),2);
    let now=crate::assistant_memory::now();crate::pet_runtime::authority_store::mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,"pause-recipe".into(),Action::Pause,now).unwrap();let drain=o.drain(DrainOutcome::Confirmed).unwrap();assert_eq!(f.store().drain_owned(&a,&drain,now).unwrap().record.state,State::Paused);drop(drain);drop(o);drop(p);drop(main);drop(ready);drop(approved);drop(player);drop(bus);
    let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","pet_runtime::lease_controller::writer::tests::step_tests::recipe_tests::read_child","--ignored"]).env("COUCOU_RECIPE_TEST_ROOT",&f.path).env("COUCOU_RECIPE_TEST_GOAL",&g.id).output().unwrap();assert!(child.status.success(),"{}",String::from_utf8_lossy(&child.stdout));assert!(String::from_utf8_lossy(&child.stdout).contains("1 passed"));
}
#[test]#[ignore="private recipe child helper invoked explicitly by parent"]
fn read_child(){
    let root=std::path::PathBuf::from(std::env::var_os("COUCOU_RECIPE_TEST_ROOT").expect("private root required"));let id=std::env::var("COUCOU_RECIPE_TEST_GOAL").unwrap();let g=crate::pet_runtime::store::Store::existing(&root.join("goals")).unwrap().unwrap().read(&id,"ai_ollama").unwrap();assert_eq!(g.state,State::Paused);let a=Authority::existing(&root.join("authority")).unwrap();let view=progress(&a,&g);assert_eq!(view.phase,RecipePhase::Verified);assert_eq!(view.verified_steps,2);assert!(!view.execution_available&&!view.criteria_verified&&!view.goal_complete);assert!(crate::pet_runtime::lease_api::step_owner(&g,&view.scope.lease_id).is_err());
}

fn decision_source(f:&Fixture,g:&Record,o:Arc<Owned>,review:&crate::pet_runtime::lease_controller::recipe_controller::Review)->crate::pet_runtime::recipe_decisions::Source{
    crate::pet_runtime::recipe_decisions::Source::capture(o,g,review,Arc::new(f.store()),Arc::new(||Ok(config()))).unwrap()
}
#[test]
fn shared_goal_decision_keeps_its_real_background_origin_and_only_the_exact_answer_can_be_consumed(){
    let(f,a,g,_p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let o=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private recipe decision player");
    let review=o.review_recipe(&g,&g.plans[0].id,"shared-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();
    let source=decision_source(&f,&g,o.clone(),&review);let decision=crate::cli_chat::goal_decision_fixture(source.clone());let target=decision.request["goalTarget"].clone();
    assert_eq!(target["origin"],"goal");assert_eq!(target["contextId"],o.lease().conversation_id);assert_eq!(target["petId"],g.spec.owner.pet_id);assert_eq!(target["recipeId"],review.proposal().scope.id);assert!(decision.request.get("target").is_none());assert_eq!(decision.request["approvalScope"],"recipe");assert_eq!(decision.request["goalReview"]["recipe"]["intent"],g.spec.intent);assert_eq!(decision.request["goalReview"]["recipe"]["steps"].as_array().unwrap().len(),2);assert_eq!(decision.request["goalReview"]["budget"],serde_json::to_value(&g.admissions[0].spec.budget).unwrap());
    assert!(main.conversation_id("ollama").unwrap()!=o.lease().conversation_id);assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    for key in ["petId","provider","goalId","contextId","leaseId","controllerId","recipeId","planId","revision","origin"]{let mut wrong=target.clone();wrong[key]=json!("foreign");assert!(decision.answer(json!({"allow":true}),wrong).is_err(),"{key}");assert_eq!(decision.count(),1);}
    let mut disguised=target.clone();disguised["sessionId"]=json!(o.lease().conversation_id);assert!(decision.answer(json!({"allow":true}),disguised).is_err());
    for answer in [json!({"allow":"true"}),json!({"allow":true,"scope":"always"}),json!({"allow":true,"nativeChoice":"accept"}),json!({"allow":true,"answers":{"grant":["yes"]}})]{assert!(decision.answer(answer,target.clone()).is_err());assert_eq!(decision.count(),1);}
    decision.answer(json!({"allow":true,"answers":{}}),target.clone()).unwrap();assert_eq!(decision.reply.recv().unwrap()["allow"],true);assert!(decision.answer(json!({"allow":true}),target).is_err());assert_eq!(decision.count(),0);
    // The transport decision alone is not a native grant, budget charge or effect.
    assert!(ledger(&f,&g).is_null());assert!(a.read_recipe(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&g.plans[0].id).unwrap().is_none());assert!(player.calls.lock().unwrap().is_empty());
    let(approved,ready)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();register(&a,&g,&o,&approved,&ready);assert_eq!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::Verified);assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]
fn goal_decision_refusal_never_changes_the_foreground_or_grants_or_runs_the_recipe(){
    let(f,a,g,_p,o,main)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let foreground=main.conversation_id("ollama").unwrap();let o=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private declined recipe");let review=o.review_recipe(&g,&g.plans[0].id,"decline-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let source=decision_source(&f,&g,o,&review);let decision=crate::cli_chat::goal_decision_fixture(source);decision.answer(json!({"allow":false}),decision.request["goalTarget"].clone()).unwrap();assert_eq!(decision.reply.recv().unwrap()["allow"],false);drop(review);
    assert_eq!(main.conversation_id("ollama").unwrap(),foreground);assert!(ledger(&f,&g).is_null());assert!(player.calls.lock().unwrap().is_empty());assert!(a.read_recipe(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&g.plans[0].id).unwrap().is_none());
}
#[test]
fn goal_review_lifetime_tracks_actual_stop_policy_configuration_and_original_goal_record(){
    for mode in 0..4{
        let allowed=Arc::new(AtomicBool::new(true));let fence=allowed.clone();let(f,_a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(move|_,_,_|if fence.load(Ordering::Acquire){Validity::Current}else{Validity::Changed}));let o=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private invalidated recipe");let review=o.review_recipe(&g,&g.plans[0].id,"invalidate-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let source=decision_source(&f,&g,o.clone(),&review);let decision=crate::cli_chat::goal_decision_fixture(source.clone());assert!(source.current());
        match mode{0=>signal_stop(o.lease()),1=>{allowed.store(false,Ordering::Release);},2=>{let now=crate::assistant_memory::now();crate::pet_runtime::authority_store::mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,"pause-shared-review".into(),Action::Pause,now).unwrap();},_=>source.retire()}
        assert!(!source.current());assert!(decision.answer(json!({"allow":true}),decision.request["goalTarget"].clone()).is_err());assert!(matches!(decision.reply.try_recv(),Err(std::sync::mpsc::TryRecvError::Empty)));assert!(ledger(&f,&g).is_null());assert!(player.calls.lock().unwrap().is_empty());
    }
    let(f,_a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let o=Arc::new(o);let bus=Bus::new();let _player=Player::new(&bus,"Private configured recipe");let review=o.review_recipe(&g,&g.plans[0].id,"config-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let music=Arc::new(AtomicBool::new(true));let current=music.clone();let source=crate::pet_runtime::recipe_decisions::Source::capture(o,&g,&review,Arc::new(f.store()),Arc::new(move||{let mut c=config();c.pets.get_mut("ai_ollama").unwrap().permissions.get_or_insert_default().music=current.load(Ordering::Acquire);Ok(c)})).unwrap();assert!(source.current());music.store(false,Ordering::Release);assert!(!source.current());
}
#[test]
fn spoken_recipe_commands_use_the_same_pending_decision_and_never_guess_a_pet_or_generic_yes(){
    let(f,_a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let o=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private spoken recipe");let review=o.review_recipe(&g,&g.plans[0].id,"voice-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let source=decision_source(&f,&g,o,&review);let decision=crate::cli_chat::goal_decision_fixture(source);let code=decision.request["voiceCode"].as_str().unwrap();let live=AtomicBool::new(false);
    assert!(decision.voice("ollama","sim",&live).is_none());assert!(decision.voice("ollama","aprovar receita",&live).unwrap().contains("seis dígitos"));assert!(decision.voice("codex",&format!("aprovar receita {code}"),&live).unwrap().contains("Não há"));assert_eq!(decision.count(),1);
    let spoken=decision.voice("ollama",&format!("rever receita {code}"),&live).unwrap();assert!(spoken.contains(&g.spec.intent));assert!(spoken.contains("Passo 1:"));assert!(spoken.contains("Passo 2:"));assert!(spoken.contains("Orçamento máximo:"));assert!(spoken.contains("não inicia a execução"));assert_eq!(decision.count(),1);
    assert!(decision.voice("ollama",&format!("aprovar receita {code}"),&AtomicBool::new(true)).unwrap().contains("interrompida"));assert_eq!(decision.count(),1);
    assert!(decision.voice("ollama",&format!("aprovar receita {code}"),&live).unwrap().contains("enviada"));assert_eq!(decision.reply.recv().unwrap()["allow"],true);assert_eq!(decision.count(),0);assert!(decision.answer(json!({"allow":false}),decision.request["goalTarget"].clone()).is_err());assert!(decision.voice("ollama",&format!("aprovar receita {code}"),&live).unwrap().contains("Não há"));assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
}

#[test]
fn management_goal_decisions_share_the_broker_without_exposing_or_answering_terminal_permissions(){
    let(f,_a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let o=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private management recipe");let review=o.review_recipe(&g,&g.plans[0].id,"panel-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let source=decision_source(&f,&g,o,&review);let decision=crate::cli_chat::goal_decision_fixture(source);decision.assert_panel_isolation();
    let mut foreign=decision.request["goalTarget"].clone();foreign["petId"]=json!("agent_codex");assert!(decision.panel_answer(true,foreign).is_err());assert_eq!(decision.count(),1);
    decision.panel_answer(false,decision.request["goalTarget"].clone()).unwrap();assert_eq!(decision.reply.recv().unwrap()["allow"],false);assert!(decision.answer(json!({"allow":true}),decision.request["goalTarget"].clone()).is_err());let code=decision.request["voiceCode"].as_str().unwrap();assert!(decision.voice("ollama",&format!("aprovar receita {code}"),&AtomicBool::new(false)).unwrap().contains("Não há"));assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    assert!(serde_json::from_value::<GoalAnswerInput>(json!({"id":"request","approve":"yes","target":{}})).is_err());assert!(serde_json::from_value::<GoalAnswerInput>(json!({"id":"request","approve":true,"target":{},"nativeChoice":"always"})).is_err());
}
