use super::*;
use super::plan_writer_tests::{setup_plan,setup_plan_custom,setup_plan_criteria};
use crate::{spotify::goal::{Target,Delivery,tests::{Bus,Player}},pet_runtime::{plans,steps::{Phase,Scope},lease_controller::step_controller::Ready,effect_budget::{Attempt,Kind,Phase as AttemptPhase},authority_store::Fault}};
use std::{fs,os::unix::fs::{PermissionsExt,symlink}};
#[path="step_recovery_tests.rs"]mod recovery_tests;
#[path="recipe_tests.rs"]mod recipe_tests;

fn config()->crate::pet_config::Configuration{let mut c=crate::pet_config::defaults();c.pets.get_mut("ai_ollama").unwrap().permissions.get_or_insert_default().music=true;c}
fn grant(a:&Authority,g:&Record,o:&Owned,bus:&Bus)->Ready{
    let p=&g.plans[0];let review=o.review_step(g,&p.id,&p.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();
    assert_eq!(review.proposal().max_uses,1);assert!(!review.proposal().goal_criteria_verified);
    let(approved,ready)=review.approve(o,g,&config(),crate::assistant_memory::now()).unwrap();let row=g.admissions.last().unwrap();
    assert_eq!(a.issue_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap(),ready.scope);ready
}
fn stored(a:&Authority,g:&Record,s:&Scope)->crate::pet_runtime::steps::StepRecord{let row=g.admissions.last().unwrap();a.read_step(&row.spec,row.proof.as_ref().unwrap(),&s.plan_id,&s.step_id).unwrap().unwrap()}
fn path(f:&Fixture,s:&Scope)->std::path::PathBuf{f.path.join("pet-runtime-step-consent-v1").join(format!("{}.json",s.slot().unwrap()))}
fn attempt(s:&Scope)->Attempt{Attempt{id:nonce().unwrap(),request_id:s.request_id.clone(),input_hash:fingerprint(s).unwrap(),kind:Kind::Tool,output_reserved:s.max_output_bytes,phase:AttemptPhase::Reserved,accepted_output:None,reserved_at:crate::assistant_memory::now(),started_at:None,finished_at:None}}

#[test]
fn conversation_consent_and_native_review_alone_never_dispatch_or_debit_a_tool(){
    let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let p=&g.plans[0];
    let review=o.review_step(&g,&p.id,&p.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();
    assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());assert!(a.read_step(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&p.id,&p.steps[0].id).unwrap().is_none());
    drop(review);assert_eq!(o.workers(),0);
}
#[test]
fn exact_native_step_debits_the_shared_budget_and_replay_only_recovers_the_same_receipt(){
    let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);
    let receipt=o.run_step(&g,&ready,&config()).unwrap();assert_eq!(receipt.phase,Phase::Finished);assert!(receipt.valid());
    let result=receipt.result.as_ref().unwrap();assert_eq!(result.observation.delivery,Delivery::Accepted);assert!(result.observation.effect_verified);assert!(!result.criteria_verified);assert!(!result.goal_complete);
    assert_eq!(*player.calls.lock().unwrap(),vec![("Pause".into(),ready.scope.resource.owner.clone())]);assert_eq!(ledger(&f,&g)["attempts"][0]["kind"],"tool");assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"settled");
    signal_stop(o.lease());let again=o.run_step(&g,&ready,&config()).unwrap();assert_eq!(serde_json::to_value(&again).unwrap(),serde_json::to_value(&receipt).unwrap());assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
    drop(o);let reopened=Authority::existing(&root(&f)).unwrap();assert_eq!(stored(&reopened,&g,&ready.scope).phase,Phase::Finished);
}
#[test]
fn native_budget_is_durable_before_the_external_command_and_stop_retains_both_holds_until_drain(){
    let(f,a,g,_p,o,main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");player.delay.store(500,Ordering::Release);let ready=grant(&a,&g,&o,&bus);let scope=ready.scope.clone();let o=Arc::new(o);let worker=o.clone();let goal=g.clone();
    let thread=thread::spawn(move||worker.run_step(&goal,&ready,&config()));let until=Instant::now()+Duration::from_secs(3);while !player.entered.load(Ordering::Acquire){assert!(Instant::now()<until);thread::sleep(Duration::from_millis(2));}
    assert_eq!(stored(&a,&g,&scope).phase,Phase::Started);assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"started");assert_eq!(o.workers(),1);assert_eq!(main.runtime_resources().busy().unwrap().len(),2);
    signal_stop(o.lease());assert!(o.drain(DrainOutcome::Confirmed).is_err());assert_eq!(main.runtime_resources().busy().unwrap().len(),2);
    let receipt=thread.join().unwrap().unwrap();let result=receipt.result.unwrap();assert!(!result.observation.scope_current);assert!(!result.observation.effect_verified);assert!(result.observation.after.is_some());assert_eq!(o.workers(),0);assert_eq!(main.runtime_resources().busy().unwrap().len(),1);assert!(o.drain(DrainOutcome::Confirmed).is_ok());
}
#[test]
fn ack_without_observed_effect_is_not_verified_and_error_with_observed_effect_never_replays(){
    for mode in [1,2]{let(_f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);player.mode.store(mode,Ordering::Release);
        let r=o.run_step(&g,&ready,&config()).unwrap().result.unwrap();assert_eq!(r.observation.effect_verified,mode==2);assert_eq!(r.observation.delivery,if mode==1{Delivery::Accepted}else{Delivery::Uncertain});
        o.run_step(&g,&ready,&config()).unwrap();assert_eq!(player.calls.lock().unwrap().len(),1);
    }
}
#[test]
fn music_permission_and_reviewed_arguments_cannot_be_borrowed_or_changed(){
    let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);let mut denied=config();denied.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;
    assert!(o.run_step(&g,&ready,&denied).is_err());let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments=json!({"action":"play"});assert!(o.run_step(&changed,&ready,&config()).is_err());
    assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());assert_eq!(stored(&a,&g,&ready.scope).phase,Phase::Approved);
}
#[test]
fn replaced_player_after_grant_is_recorded_without_dispatch_or_budget_debit(){
    let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);drop(player);let replacement=Player::new(&bus,"Private step player");
    let r=o.run_step(&g,&ready,&config()).unwrap();assert_eq!(r.phase,Phase::Finished);assert!(r.attempt.is_none());assert_eq!(r.result.unwrap().observation.delivery,Delivery::NotDispatched);assert!(replacement.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
}
#[test]
fn physical_resource_conflict_is_refused_without_consuming_the_single_step_grant(){
    let(f,a,g,_p,o,main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);let key=crate::pet_runtime::lease_controller::step_controller::resource_key(&ready.scope.resource).unwrap();
    let held=main.runtime_resources().acquire(&key,Arc::new(AtomicBool::new(false)),None).unwrap();assert!(o.run_step(&g,&ready,&config()).is_err());assert_eq!(stored(&a,&g,&ready.scope).phase,Phase::Approved);assert!(ledger(&f,&g).is_null());assert!(player.calls.lock().unwrap().is_empty());drop(held);assert!(o.run_step(&g,&ready,&config()).unwrap().result.unwrap().observation.effect_verified);
}
#[test]
fn partial_accounting_states_never_resume_automatic_io(){
    for stage in 0..3{let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);let row=&g.admissions[0];let proof=row.proof.as_ref().unwrap();let native=attempt(&ready.scope);let now=crate::assistant_memory::now();
        a.reserve_step(&row.spec,proof,o.lease(),&ready.scope,&native,now).unwrap();
        if stage>0{a.reserve_attempt(&row.spec,proof,o.lease(),&native,now).unwrap();a.start_attempt(&row.spec,proof,o.lease(),&native,now).unwrap();a.start_step(&row.spec,proof,o.lease(),&ready.scope,&native,now).unwrap();}
        if stage>1{a.finish_attempt(&row.spec,proof,o.lease(),&native,None,now).unwrap();}
        assert!(o.run_step(&g,&ready,&config()).is_err());assert!(player.calls.lock().unwrap().is_empty());assert_eq!(stored(&a,&g,&ready.scope).phase,if stage==0{Phase::Reserved}else{Phase::Started});if stage>0{assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);}
    }
}
#[test]
fn interrupted_grant_publication_has_a_single_stable_scope_and_no_effect(){
    for fault in [Fault::BeforeStepPublish,Fault::AfterStepPublish]{let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let p=&g.plans[0];let review=o.review_step(&g,&p.id,&p.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];let proof=row.proof.as_ref().unwrap();a.fault=fault;
        assert!(a.issue_step(&row.spec,proof,o.lease(),&approved,crate::assistant_memory::now()).is_err());a.fault=Fault::None;
        a.issue_step(&row.spec,proof,o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert_eq!(stored(&a,&g,&ready.scope).scope,ready.scope);assert!(ledger(&f,&g).is_null());assert!(player.calls.lock().unwrap().is_empty());
    }
}
#[test]
fn tampered_linked_and_public_step_records_are_refused_without_rewriting(){
    for mode in 0..4{let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);let file=path(&f,&ready.scope);let alias=f.path.join("external-step");
        match mode{0=>{let mut value:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();value["body"]["scope"]["argumentsHash"]=json!("a".repeat(64));fs::write(&file,serde_json::to_vec(&value).unwrap()).unwrap();},1=>{fs::rename(&file,&alias).unwrap();symlink(&alias,&file).unwrap();},2=>{fs::hard_link(&file,&alias).unwrap();},_=>{fs::set_permissions(&file,fs::Permissions::from_mode(0o644)).unwrap();}}
        let before=fs::read(&file).unwrap();assert!(o.run_step(&g,&ready,&config()).is_err());assert_eq!(fs::read(&file).unwrap(),before);assert!(player.calls.lock().unwrap().is_empty());assert!(ledger(&f,&g).is_null());
    }
}
#[test]
fn zero_tool_budget_and_unverified_predecessor_block_review(){
    for tools in [0,2]{let step=||StepInput{description:"Pause".into(),postcondition:"Paused independently observed".into(),operation:Some(plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:json!({"action":"pause"})})};let(f,a,g,_p,o,_)=setup_plan_custom(vec![step(),step()],tools,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private step player");let p=&g.plans[0];
        if tools>0{let ready=grant(&a,&g,&o,&bus);player.mode.store(1,Ordering::Release);assert!(!o.run_step(&g,&ready,&config()).unwrap().result.unwrap().observation.effect_verified);}
        let index=if tools==0{0}else{1};assert!(o.review_step(&g,&p.id,&p.steps[index].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).is_err());if tools==0{assert!(ledger(&f,&g).is_null());assert!(player.calls.lock().unwrap().is_empty());}
    }
}

#[test]
fn native_review_tokens_are_window_and_scope_bound_expiring_and_single_use(){
    use crate::pet_runtime::step_api::{Cache,ConfirmInput};
    for mode in 0..6{let(_f,_a,g,_p,o,_)=setup_plan();let bus=Bus::new();let _player=Player::new(&bus,"Private step player");let p=&g.plans[0];let now=crate::assistant_memory::now();let review=o.review_step(&g,&p.id,&p.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),now).unwrap();let expires=review.proposal().expires_at;let mut c=Cache::default();let token=nonce().unwrap();c.insert_pending(token.clone(),"main".into(),"review".into(),review,now).unwrap();
        let mut input=ConfirmInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),lease_id:o.lease().id.clone(),revision:g.revision.clone(),token,approve:true};let mut window="main";
        match mode{0=>window="foreign",1=>input.pet_id="agent_codex".into(),2=>input.goal_id="other".into(),3=>input.lease_id=nonce().unwrap(),4=>input.revision=nonce().unwrap(),_=>{}}
        if mode<5{assert!(c.take(window,&input,now).is_err());input.pet_id=g.spec.owner.pet_id.clone();input.goal_id=g.id.clone();input.lease_id=o.lease().id.clone();input.revision=g.revision.clone();assert!(c.take("main",&input,now).is_ok());assert!(c.take("main",&input,now).is_err());}else{assert!(c.take("main",&input,expires).is_err());}
    }
}

#[test]
fn ipc_step_inputs_reject_serialized_permits_targets_and_supplied_results(){
    use crate::pet_runtime::step_api::{PrepareInput,ConfirmInput,RunInput,ReadInput};
    let prepare=json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","revision":"revision","planId":"plan","stepId":"step","requestId":"review","target":{"owner":":1.1"}});assert!(serde_json::from_value::<PrepareInput>(prepare).is_err());
    let confirm=json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","revision":"revision","token":"token","approve":true,"scope":{"approved":true}});assert!(serde_json::from_value::<ConfirmInput>(confirm).is_err());
    let run=json!({"petId":"ai_ollama","goalId":"goal","leaseId":"lease","grantId":"grant","planId":"plan","stepId":"step","operation":{"action":"play"}});assert!(serde_json::from_value::<RunInput>(run).is_err());
    let read=json!({"petId":"ai_ollama","goalId":"goal","decisionId":"decision","planId":"plan","stepId":"step","effectVerified":true});assert!(serde_json::from_value::<ReadInput>(read).is_err());
}

#[test]
fn step_receipt_survives_a_new_process_after_pause(){
    use crate::pet_runtime::authority_store::mutate_with_authority;
    let(f,a,g,p,o,main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);assert!(o.run_step(&g,&ready,&config()).unwrap().result.unwrap().observation.effect_verified);
    let now=crate::assistant_memory::now();let stopping=mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,"pause".into(),Action::Pause,now).unwrap().record;assert_eq!(stopping.state,State::Stopping);
    let drain=o.drain(DrainOutcome::Confirmed).unwrap();assert_eq!(f.store().drain_owned(&a,&drain,now).unwrap().record.state,State::Paused);drop(drain);drop(o);drop(p);drop(main);drop(a);drop(ready);drop(player);drop(bus);
    let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","pet_runtime::lease_controller::writer::tests::step_tests::step_read_child","--ignored"]).env("COUCOU_STEP_TEST_ROOT",&f.path).env("COUCOU_STEP_TEST_GOAL",&g.id).output().unwrap();
    assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stdout));assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
}
#[test]
#[ignore="private child process fixture; executed explicitly by its parent test"]
fn step_read_child(){
    let root=std::path::PathBuf::from(std::env::var_os("COUCOU_STEP_TEST_ROOT").expect("private test root required"));let id=std::env::var("COUCOU_STEP_TEST_GOAL").unwrap();let g=crate::pet_runtime::store::Store::existing(&root.join("goals")).unwrap().unwrap().read(&id,"ai_ollama").unwrap();assert_eq!(g.state,State::Paused);
    let a=Authority::existing(&root.join("authority")).unwrap();let row=&g.admissions[0];let plan=&g.plans[0];let receipt=a.read_step(&row.spec,row.proof.as_ref().unwrap(),&plan.id,&plan.steps[0].id).unwrap().unwrap();assert_eq!(receipt.phase,Phase::Finished);let result=receipt.result.unwrap();assert!(result.observation.effect_verified);assert!(!result.criteria_verified&&!result.goal_complete);assert!(a.verify_queued(&g,crate::assistant_memory::now()).is_err());
}
#[test]
fn result_publication_failure_recovers_evidence_or_preserves_uncertainty_without_repeating_the_effect(){
    for fault in [Fault::BeforeStepResultPublish,Fault::AfterStepResultPublish]{let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);a.fault=fault;
        assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"settled");a.fault=Fault::None;
        if matches!(fault,Fault::BeforeStepResultPublish){assert_eq!(stored(&a,&g,&ready.scope).phase,Phase::Started);assert!(o.run_step(&g,&ready,&config()).is_err());}else{assert!(o.run_step(&g,&ready,&config()).unwrap().result.unwrap().observation.effect_verified);}
        assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
    }
}
#[test]
fn uncertain_shared_budget_publication_blocks_the_effect_and_all_automatic_retries(){
    for fault in [Fault::BeforeGrantPublish,Fault::AfterGrantPublish]{let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);a.fault=fault;
        assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert_eq!(stored(&a,&g,&ready.scope).phase,Phase::Reserved);assert!(o.run_step(&g,&ready,&config()).is_err());assert!(player.calls.lock().unwrap().is_empty());
        if matches!(fault,Fault::BeforeGrantPublish){assert!(ledger(&f,&g).is_null());}else{assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"reserved");}
    }
}
#[test]
fn consumed_shared_tool_budget_blocks_the_next_step_before_io(){
    let step=||StepInput{description:"Pause".into(),postcondition:"Playback observed paused".into(),operation:Some(plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:json!({"action":"pause"})})};let(f,a,g,_p,o,_)=setup_plan_custom(vec![step(),step()],1,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private step player");let ready=grant(&a,&g,&o,&bus);o.run_step(&g,&ready,&config()).unwrap();
    let plan=&g.plans[0];let review=o.review_step(&g,&plan.id,&plan.steps[1].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];a.issue_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert!(o.run_step(&g,&ready,&config()).is_err());assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);assert!(o.run_step(&g,&ready,&config()).is_err());
}
#[test]
fn replacement_before_confirmation_cannot_obtain_authority(){
    let(f,_a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private step player");let plan=&g.plans[0];let review=o.review_step(&g,&plan.id,&plan.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();drop(player);let next=Player::new(&bus,"Private step player");assert!(review.approve(&o,&g,&config(),crate::assistant_memory::now()).is_err());assert!(ledger(&f,&g).is_null());assert!(next.calls.lock().unwrap().is_empty());
}
#[test]
fn competing_native_reviews_cannot_replace_the_first_retained_target(){
    use crate::pet_runtime::{step_api::Cache,lease_controller::step_controller::resource_key};
    let(_f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let _player=Player::new(&bus,"Private step player");let plan=&g.plans[0];let mut prepared=vec![];for _ in 0..2{prepared.push(o.review_step(&g,&plan.id,&plan.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap());}
    let mut c=Cache::default();let(_approved,first)=prepared.pop().unwrap().approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let scope=first.scope.clone();let(_approved,second)=prepared.pop().unwrap().approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();c.remember(first,crate::assistant_memory::now()).unwrap();assert!(c.remember(second,crate::assistant_memory::now()).is_err());
    let mut renamed=scope.resource.clone();renamed.identity="Renamed reader".into();assert_eq!(resource_key(&scope.resource).unwrap(),resource_key(&renamed).unwrap());
    c.forget(&scope);let ready=grant(&a,&g,&o,&bus);c.remember(ready,crate::assistant_memory::now()).unwrap();
}
