//! Typed predicates, real private snapshots and authenticated goal publication.
use super::*;
use crate::pet_runtime::{criteria::{Input,Predicate,Playback,Status},store::Fault as GoalFault,recipe_api::complete_for_goal};
fn bindings()->Vec<Input>{vec![Input{criterion_index:0,predicate:Predicate::SpotifyPlayback{status:Playback::Paused}},Input{criterion_index:1,predicate:Predicate::SpotifyVolume{percent:17}}]}
fn setup()->(Fixture,Authority,Record,Pending,Owned,Chat){setup_plan_criteria(sequence(),2,Arc::new(|_,_,_|Validity::Current),vec!["Spotify is paused at the end".into(),"Final volume is 17 percent".into()])}
fn typed(o:&Owned,g:&Record,bus:&Bus,conditions:&[Input])->(RecipeApproved,RecipeReady){
    let review=o.review_recipe_with_criteria(g,&g.plans[0].id,"typed-recipe",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),conditions,crate::assistant_memory::now()).unwrap();
    let proposal=review.proposal();assert_eq!(proposal.scope.schema_version,2);let v=proposal.scope.verification.as_ref().unwrap();assert_eq!(v.max_observations,1);assert!(v.matches(g));assert!(!proposal.criteria_verified&&!proposal.goal_complete);
    review.approve(o,g,&config(),crate::assistant_memory::now()).unwrap()
}
fn verification_path(f:&Fixture,ready:&RecipeReady)->std::path::PathBuf{f.path.join("pet-runtime-step-consent-v1").join(format!("{}.verification.json",ready.scope().slot().unwrap()))}
fn completion_input(g:&Record,ready:&RecipeReady)->RecipeCompleteInput{let s=ready.scope();RecipeCompleteInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:s.decision_id.clone(),plan_id:s.plan_id.clone(),recipe_id:s.id.clone(),revision:g.revision.clone()}}

#[test]
fn completed_native_reads_share_the_existing_authority_reader_lock_without_replaying_effects(){
    use std::os::fd::AsRawFd;
    let(f,a,g,_p,o,_main)=setup();let bus=Bus::new();let player=Player::new(&bus,"Shared completion reader");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);
    assert_eq!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::GoalCompleted);
    let authority=crate::pet_runtime::authority_tests::root(&f);let store=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&authority).unwrap().unwrap();let saved=store.read(&g.id,"ai_ollama").unwrap();
    let goal_path=f.path.join("goals").join(format!("{}.json",g.id));let grant_path=authority.join(format!("{}.json",g.admissions[0].spec.decision_id));let verification=verification_path(&f,&ready);let bytes=[&goal_path,&grant_path,&verification].map(|p|std::fs::read(p).unwrap());
    let lock=std::fs::OpenOptions::new().read(true).write(true).open(authority.join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);
    let observed=store.read(&g.id,"ai_ollama").unwrap();assert_eq!(observed.revision,saved.revision);assert_eq!(observed.completion,saved.completion);a.verify_completed_goal(&observed).unwrap();
    assert_eq!([&goal_path,&grant_path,&verification].map(|p|std::fs::read(p).unwrap()),bytes);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(o.workers(),0);drop(lock);
}

#[test]
fn completed_native_reads_reject_a_real_writer_lock_and_corrupt_signed_verification(){
    use std::os::fd::AsRawFd;
    let(f,_a,g,_p,o,_main)=setup();let bus=Bus::new();let player=Player::new(&bus,"Exclusive completion fence");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&_a,&g,&o,&approved,&ready);
    assert_eq!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::GoalCompleted);
    let authority=crate::pet_runtime::authority_tests::root(&f);let store=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&authority).unwrap().unwrap();let lock=std::fs::OpenOptions::new().read(true).write(true).open(authority.join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);
    assert_eq!(store.read(&g.id,"ai_ollama").err(),Some(crate::pet_runtime::model::Error::Busy));drop(lock);assert_eq!(store.read(&g.id,"ai_ollama").unwrap().state,State::Completed);
    let path=verification_path(&f,&ready);let mut body:serde_json::Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();body["tag"]=serde_json::json!("00".repeat(32));let corrupt=serde_json::to_vec(&body).unwrap();std::fs::write(&path,&corrupt).unwrap();assert_eq!(store.read(&g.id,"ai_ollama").err(),Some(crate::pet_runtime::model::Error::InvalidRecord));assert_eq!(std::fs::read(&path).unwrap(),corrupt);assert_eq!(player.calls.lock().unwrap().len(),2);
}

#[test]
fn typed_final_predicates_publish_native_evidence_then_complete_and_close_the_original_goal(){
    let(f,a,g,_p,o,main)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert!(result.progress.criteria_verified&&result.progress.goal_complete);assert_eq!(result.progress.verification.as_ref().unwrap().results.iter().map(|r|r.status).collect::<Vec<_>>(),vec![Status::Met,Status::Met]);
    let saved=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(saved.state,State::Completed);assert_eq!(saved.schema_version,7);assert_eq!(saved.sequence,g.sequence+1);assert!(saved.completion.is_some());a.verify_completed_goal(&saved).unwrap();assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);assert_eq!(o.workers(),0);
    let reference=saved.completion.clone();let replay=o.run_recipe_for_store(&f.store(),&g,&ready,||panic!("completed retry must not dispatch"),|_|{}).unwrap();assert_eq!(replay.outcome,Outcome::GoalCompleted);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().completion,reference);assert_eq!(player.calls.lock().unwrap().len(),2);
    let drain=o.drain(DrainOutcome::Confirmed).unwrap();let closed=f.store().drain_owned(&a,&drain,crate::assistant_memory::now()).unwrap();assert_eq!(closed.record.state,State::Completed);assert_eq!(closed.record.sequence,saved.sequence);a.verify_completed_goal(&closed.record).unwrap();assert!(!o.current(crate::assistant_memory::now()));drop(drain);drop(o);drop(main);
}

#[test]
fn a_later_action_undoing_an_earlier_success_leaves_the_final_criterion_unmet(){
    let steps=vec![proposed(json!({"action":"pause"})),proposed(json!({"action":"play"}))];let(f,a,g,_p,o,_)=setup_plan_criteria(steps,2,Arc::new(|_,_,_|Validity::Current),vec!["Final playback is paused".into()]);let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings()[..1]);register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.progress.phase,RecipePhase::Verified);assert_eq!(result.progress.verified_steps,2);assert_eq!(result.outcome,Outcome::CriteriaUnmet);let v=result.progress.verification.unwrap();assert_eq!(v.snapshot.unwrap().playback_status,"Playing");assert_eq!(v.results[0].status,Status::Unmet);assert!(!result.progress.goal_complete);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);
    player.state.lock().unwrap().status="Paused".into();assert_eq!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::CriteriaUnmet);assert_eq!(player.calls.lock().unwrap().len(),2);
}

#[test]
fn incomplete_coverage_is_unknown_and_never_silently_completes_the_remaining_criteria(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings()[1..]);register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::CriteriaIncomplete);let v=result.progress.verification.unwrap();assert_eq!(v.results[0].status,Status::Unknown);assert_eq!(v.results[1].status,Status::Met);assert!(!v.criteria_verified);assert!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().completion.is_none());assert_eq!(player.calls.lock().unwrap().len(),2);
}

#[test]
fn duplicate_foreign_or_executable_criterion_proposals_fail_before_approval_or_effects(){
    let(f,_a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");
    for conditions in [vec![bindings()[0].clone();2],vec![Input{criterion_index:9,predicate:Predicate::SpotifyVolume{percent:17}}],vec![Input{criterion_index:1,predicate:Predicate::SpotifyVolume{percent:101}}]]{assert!(o.review_recipe_with_criteria(&g,&g.plans[0].id,"bad-criteria",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&conditions,crate::assistant_memory::now()).is_err());}
    assert!(serde_json::from_value::<Input>(json!({"criterionIndex":0,"predicate":{"kind":"shell","command":"forged"}})).is_err());assert!(serde_json::from_value::<Input>(json!({"criterionIndex":0,"criterionHash":"forged","predicate":{"kind":"spotifyPlayback","status":"paused"}})).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
}

#[test]
fn changing_original_criteria_invalidates_the_review_and_mutating_input_cannot_change_captured_bindings(){
    let(f,_a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let mut conditions=bindings();let review=o.review_recipe_with_criteria(&g,&g.plans[0].id,"criteria",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&conditions,crate::assistant_memory::now()).unwrap();conditions[1].predicate=Predicate::SpotifyVolume{percent:99};assert_eq!(review.proposal().scope.verification.as_ref().unwrap().conditions[1].predicate,Predicate::SpotifyVolume{percent:17});let mut changed=g.clone();changed.spec.criteria[0]="Different original criterion".into();assert!(review.approve(&o,&changed,&config(),crate::assistant_memory::now()).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
}

#[test]
fn invalidation_before_final_observation_cannot_turn_step_receipts_into_goal_success(){
    let allowed=Arc::new(AtomicBool::new(true));let gate=allowed.clone();let(f,a,g,_p,o,_)=setup_plan_criteria(sequence(),2,Arc::new(move|_,_,_|if gate.load(Ordering::Acquire){Validity::Current}else{Validity::Changed}),vec!["Paused".into(),"Volume17".into()]);let bus=Bus::new();let _player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.phase==RecipePhase::Verified&&p.verification.is_none(){allowed.store(false,Ordering::Release)}).unwrap();assert_eq!(result.outcome,Outcome::CriteriaIncomplete);assert!(result.progress.verification.unwrap().snapshot.is_none());assert!(!result.progress.goal_complete);assert!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().completion.is_none());
}

#[test]
fn stop_after_native_proof_but_before_goal_publication_blocks_the_completion_transition(){
    let allowed=Arc::new(AtomicBool::new(true));let gate=allowed.clone();let(f,a,g,_p,o,_)=setup_plan_criteria(sequence(),2,Arc::new(move|_,_,_|if gate.load(Ordering::Acquire){Validity::Current}else{Validity::Changed}),vec!["Paused".into(),"Volume17".into()]);let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);
    assert!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified{allowed.store(false,Ordering::Release)}).is_err());assert!(progress(&a,&g).criteria_verified);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);assert_eq!(player.calls.lock().unwrap().len(),2);
}

#[test]
fn verification_publication_retries_reuse_the_original_native_observation_without_another_read_or_effect(){
    for fault in [Fault::BeforeVerificationPublish,Fault::AfterVerificationPublish]{let(f,mut a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);a.fault=fault;
        assert_eq!(o.run_recipe_with_authority(&g,&ready,&a,||Ok(config()),|_|{}).unwrap().outcome,Outcome::RecoveryRequired);a.fault=Fault::None;player.state.lock().unwrap().volume=0.99;
        let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert_eq!(result.progress.verification.unwrap().snapshot.unwrap().volume,0.17);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
    }
}

#[test]
fn goal_publication_before_or_after_failure_is_recovered_from_the_same_proof_without_reexecution(){
    for fault in [GoalFault::BeforePublish,GoalFault::AfterPublish]{let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut store=f.store();store.fault=fault;
        let first=o.run_recipe_for_store(&store,&g,&ready,||Ok(config()),|_|{});if matches!(fault,GoalFault::BeforePublish){assert!(first.is_err());assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);}else{let r=first.unwrap();assert_eq!(r.outcome,Outcome::RecoveryRequired);assert_eq!(r.completion,Some(CommitStatus::Uncertain));}
        store.fault=GoalFault::None;let original=progress(&a,&g).verification.unwrap().reference;let result=o.run_recipe_for_store(&store,&g,&ready,||panic!("receipt recovery never dispatches"),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);let saved=store.read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(saved.completion.as_ref(),Some(&original));assert_eq!(saved.sequence,g.sequence+1);assert_eq!(player.calls.lock().unwrap().len(),2);
    }
}

#[test]
fn an_active_other_worker_prevents_goal_completion_until_it_has_really_drained(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let other=o.readiness_worker(crate::assistant_memory::now()).unwrap();assert!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).is_err());assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);drop(other);assert_eq!(o.run_recipe_for_store(&f.store(),&g,&ready,||panic!("no new effects"),|_|{}).unwrap().outcome,Outcome::GoalCompleted);assert_eq!(player.calls.lock().unwrap().len(),2);
}

#[test]
fn commit_policy_uses_the_exact_record_already_locked_by_the_goal_transaction(){
    let path=Arc::new(Mutex::new(None::<std::path::PathBuf>));let actual=path.clone();let seen=Arc::new(AtomicBool::new(false));let flag=seen.clone();
    let(f,a,g,_p,o,_)=setup_plan_criteria(sequence(),2,Arc::new(move|_,_,mode|{if let Validation::Commit(goal)=mode{flag.store(true,Ordering::Release);assert_eq!(goal.state,State::Leased);let p=actual.lock().unwrap().clone().unwrap();assert!(crate::pet_runtime::store::Store::existing(&p).unwrap().unwrap().read(&goal.id,&goal.spec.owner.pet_id).is_err());}Validity::Current}),vec!["Paused".into(),"Volume17".into()]);*path.lock().unwrap()=Some(f.path.join("goals"));let bus=Bus::new();let _player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);assert_eq!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap().outcome,Outcome::GoalCompleted);assert!(seen.load(Ordering::Acquire));
}

#[test]
fn tampered_native_proof_and_forged_completion_inputs_cannot_publish_success(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let file=verification_path(&f,&ready);
    assert!(o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified{let mut v:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();v["body"]["snapshot"]["volume"]=json!(0.99);fs::write(&file,serde_json::to_vec(&v).unwrap()).unwrap();}).is_err());assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);assert_eq!(player.calls.lock().unwrap().len(),2);
    assert!(serde_json::from_value::<RecipeCompleteInput>(json!({"petId":"ai_ollama","goalId":"goal","decisionId":"decision","planId":"plan","recipeId":"recipe","revision":"revision","goalComplete":true})).is_err());assert!(serde_json::from_value::<Action>(json!({"kind":"complete","verified":true})).is_err());
}

#[test]
fn legacy_recipe_and_goal_schemas_keep_their_old_shape_and_never_gain_implicit_criteria(){
    let(f,a,g,_p,o,_)=setup_plan_custom(sequence(),2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let _player=Player::new(&bus,"Private criteria player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);assert_eq!(ready.scope().schema_version,1);let raw:Value=serde_json::from_slice(&fs::read(recipe_path(&f,&ready)).unwrap()).unwrap();assert!(raw["body"].get("verification").is_none());
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::Verified);assert!(!result.progress.criteria_verified&&!result.progress.goal_complete);let old=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(old.schema_version,6);assert!(serde_json::to_value(&old).unwrap().get("completion").is_none());
}

#[test]
fn canonical_completed_goal_recovers_in_a_new_process_without_native_controller_or_target(){
    let(f,a,g,p,o,main)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut store=f.store();store.fault=GoalFault::AfterPublish;let result=o.run_recipe_for_store(&store,&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.completion,Some(CommitStatus::Uncertain));assert_eq!(player.calls.lock().unwrap().len(),2);drop(o);drop(p);drop(main);drop(player);drop(bus);drop(approved);drop(ready);
    let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","pet_runtime::lease_controller::writer::tests::step_tests::recipe_tests::criteria_tests::completion_child","--ignored"]).env("COUCOU_CRITERIA_TEST_ROOT",&f.path).env("COUCOU_CRITERIA_TEST_GOAL",&g.id).output().unwrap();assert!(child.status.success(),"{}",String::from_utf8_lossy(&child.stdout));assert!(String::from_utf8_lossy(&child.stdout).contains("1 passed"));let saved=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(saved.state,State::Completed);a.verify_completed_goal(&saved).unwrap();
}
#[test]#[ignore="private completed-goal helper invoked by its parent"]
fn completion_child(){
    let root=std::path::PathBuf::from(std::env::var_os("COUCOU_CRITERIA_TEST_ROOT").expect("private root required"));let id=std::env::var("COUCOU_CRITERIA_TEST_GOAL").unwrap();let store=crate::pet_runtime::store::Store::existing(&root.join("goals")).unwrap().unwrap();let goal=store.read(&id,"ai_ollama").unwrap();assert_eq!(goal.state,State::Completed);let c=goal.completion.as_ref().unwrap();let input=RecipeCompleteInput{pet_id:goal.spec.owner.pet_id.clone(),goal_id:id,decision_id:c.decision_id.clone(),plan_id:c.plan_id.clone(),recipe_id:c.recipe_id.clone(),revision:c.source_revision.clone()};let a=Authority::existing(&root.join("authority")).unwrap();let receipt=complete_for_goal(&store,&a,&goal,&input,|_|panic!("completed recovery must not create live authority")).unwrap();assert_eq!(receipt.status,CommitStatus::Unchanged);assert_eq!(receipt.record.revision,goal.revision);assert_eq!(receipt.record.sequence,goal.sequence);assert!(crate::pet_runtime::lease_api::step_owner(&goal,&c.lease_id).is_err());
}

#[test]
fn authenticated_native_repository_rejects_a_structurally_valid_but_forged_completion_reference(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let _player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();
    let native=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&root(&f)).unwrap().unwrap();let mut forged=native.read(&g.id,&g.spec.owner.pet_id).unwrap();forged.completion.as_mut().unwrap().report_hash="0".repeat(64);forged.events.last_mut().unwrap().fingerprint=fingerprint(forged.completion.as_ref().unwrap()).unwrap();forged.validate().unwrap();
    fs::write(f.path.join("goals").join(format!("{}.json",g.id)),serde_json::to_vec(&forged).unwrap()).unwrap();assert!(f.store().read(&g.id,&g.spec.owner.pet_id).is_ok());assert!(native.read(&g.id,&g.spec.owner.pet_id).is_err());let listing=native.list(None).unwrap();assert!(listing.goals.is_empty());assert_eq!(listing.unavailable,1);assert!(!listing.complete);
}

#[test]
fn completed_goal_lookup_does_not_fall_back_to_another_authority_key_or_a_path_from_data(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let _player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|_|{}).unwrap();let other=Fixture::new();let _key=Authority::ensure(&root(&other)).unwrap();
    let wrong=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&root(&other)).unwrap().unwrap();assert!(wrong.read(&g.id,&g.spec.owner.pet_id).is_err());assert!(crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&root(&f)).unwrap().unwrap().read(&g.id,&g.spec.owner.pet_id).is_ok());
}

#[test]
fn completion_references_are_exact_and_invalid_attempts_do_not_consume_or_replace_the_native_proof(){
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Private criteria player");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);
    assert!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap().progress.criteria_verified);let before=fs::read(verification_path(&f,&ready)).unwrap();let input=completion_input(&g,&ready);
    let data=json!({"petId":input.pet_id,"goalId":input.goal_id,"decisionId":input.decision_id,"planId":input.plan_id,"recipeId":input.recipe_id,"revision":input.revision});
    for key in ["petId","goalId","decisionId","planId","recipeId","revision"]{let mut forged=data.clone();forged[key]=json!("foreign");let wrong:RecipeCompleteInput=serde_json::from_value(forged).unwrap();assert!(complete_for_goal(&f.store(),&a,&g,&wrong,|current|o.completion_authority(current,crate::assistant_memory::now())).is_err());assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);assert_eq!(fs::read(verification_path(&f,&ready)).unwrap(),before);}
    let completed=o.with_completion_lock(||complete_for_goal(&f.store(),&a,&g,&input,|current|o.completion_authority(current,crate::assistant_memory::now()))).unwrap();assert_eq!(completed.record.state,State::Completed);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(fs::read(verification_path(&f,&ready)).unwrap(),before);
}

#[test]
fn transient_completion_catalogue_locks_retain_original_proof_and_never_repeat_native_effects(){
    use std::os::fd::AsRawFd;
    for authority_writer in [false,true]{
        let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Bounded completion reader contention");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut release=None;
        let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified&&release.is_none(){
            let path=if authority_writer{crate::pet_runtime::authority_tests::root(&f)}else{f.path.join("goals")};let file=fs::OpenOptions::new().read(true).write(true).open(path.join(".catalog.lock")).unwrap();let mode=if authority_writer{libc::LOCK_EX}else{libc::LOCK_SH};assert_eq!(unsafe{libc::flock(file.as_raw_fd(),mode|libc::LOCK_NB)},0);
            release=Some(std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(120));drop(file);}));
        }).unwrap();release.unwrap().join().unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().sequence,g.sequence+1);
    }
}
#[test]
fn unknown_commit_currentness_waits_for_known_policy_without_new_effects_or_authority(){
    let unknown=Arc::new(AtomicBool::new(false));let check=unknown.clone();let(f,a,g,_p,o,_)=setup_plan_criteria(sequence(),2,Arc::new(move|_,_,mode|if matches!(mode,Validation::Commit(_))&&check.load(Ordering::Acquire){Validity::Unknown}else{Validity::Current}),vec!["Paused".into(),"Volume17".into()]);let bus=Bus::new();let player=Player::new(&bus,"Unknown completion policy");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut release=None;
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified&&release.is_none(){unknown.store(true,Ordering::Release);let flag=unknown.clone();release=Some(std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(120));flag.store(false,Ordering::Release);}));}).unwrap();release.unwrap().join().unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
}
#[test]
fn stop_takes_the_live_gate_during_completion_lock_wait_and_prevents_success(){
    use std::os::fd::AsRawFd;
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Stop during native completion wait");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut stopper=None;let lease=o.lease().clone();let live=o.live.clone();let acquired=Arc::new(AtomicBool::new(false));
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified&&stopper.is_none(){
        let file=fs::OpenOptions::new().read(true).write(true).open(f.path.join("goals/.catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);let lease=lease.clone();let live=live.clone();let flag=acquired.clone();
        stopper=Some(std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(30));let until=Instant::now()+Duration::from_millis(500);loop{if let Ok(gate)=live.gate.try_lock(){flag.store(true,Ordering::Release);drop(gate);break;}assert!(Instant::now()<until,"Stop must not wait behind the completion retry loop");std::thread::sleep(Duration::from_millis(2));}crate::pet_runtime::lease_controller::signal_stop(&lease);drop(file);}));
    });stopper.unwrap().join().unwrap();assert_eq!(result.err(),Some(model::Error::Conflict));assert!(acquired.load(Ordering::Acquire));assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);assert!(!o.current(crate::assistant_memory::now()));assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]
fn persistent_completion_busy_is_bounded_and_explicit_retry_reuses_the_original_evidence(){
    use std::os::fd::AsRawFd;
    let(f,a,g,_p,o,_)=setup();let bus=Bus::new();let player=Player::new(&bus,"Persistent completion reader");let(approved,ready)=typed(&o,&g,&bus,&bindings());register(&a,&g,&o,&approved,&ready);let mut hold=None;let start=Instant::now();
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|p|if p.criteria_verified&&hold.is_none(){let file=fs::OpenOptions::new().read(true).write(true).open(f.path.join("goals/.catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);hold=Some(file);});
    assert_eq!(result.err(),Some(model::Error::Busy));assert!(start.elapsed()>=Duration::from_millis(900));assert!(start.elapsed()<Duration::from_secs(5));drop(hold);let original=progress(&a,&g).verification.unwrap().reference;assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::Leased);assert_eq!(player.calls.lock().unwrap().len(),2);
    let result=o.run_recipe_for_store(&f.store(),&g,&ready,||panic!("completion retry must not dispatch native operations"),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().completion,Some(original));assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
}
