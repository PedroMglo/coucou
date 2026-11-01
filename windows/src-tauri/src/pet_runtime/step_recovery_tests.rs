//! Real private player/bus and durable ledgers; no provider or user-device IO.
use super::*;
use crate::pet_runtime::{authority::Failure,steps::{Assessment,RecoveryStatus,RecoveryBudget},step_api::{self,RecoverInput}};

fn assess(a:&Authority,g:&Record,s:&Scope)->Assessment{
    let row=&g.admissions[0];a.read_step_assessment(&row.spec,row.proof.as_ref().unwrap(),&s.plan_id,&s.step_id).unwrap().unwrap()
}
fn recover(a:&Authority,g:&Record,s:&Scope)->Assessment{
    let row=&g.admissions[0];a.recover_step(&row.spec,row.proof.as_ref().unwrap(),row.lease.as_ref().unwrap(),s,crate::assistant_memory::now()).unwrap()
}
fn evidence_path(f:&Fixture,s:&Scope)->std::path::PathBuf{f.path.join("pet-runtime-step-consent-v1").join(format!("{}.evidence.json",s.slot().unwrap()))}
fn input(g:&Record,s:&Scope)->RecoverInput{RecoverInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:s.decision_id.clone(),plan_id:s.plan_id.clone(),step_id:s.step_id.clone(),revision:g.revision.clone(),grant_id:s.grant_id.clone()}}

#[test]fn native_receipt_recovery_keeps_the_goal_guard_until_original_authority_settlement(){
    use std::os::fd::AsRawFd;
    let(f,mut a,g,_p,o,_main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Serialized receipt recovery");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::AfterStepEvidencePublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"started");
    let authority=root(&f);let store=Arc::new(crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&authority).unwrap().unwrap());let lock=fs::OpenOptions::new().read(true).write(true).open(authority.join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);
    let request=input(&g,&ready.scope);let thread_store=store.clone();let thread_authority=authority.clone();let recovery=std::thread::spawn(move||thread_store.recover_step_native(&thread_authority,&request,crate::assistant_memory::now()));let until=std::time::Instant::now()+std::time::Duration::from_secs(2);let mut observed=false;
    while std::time::Instant::now()<until{match store.test_lock(true){Err(crate::pet_runtime::model::Error::Busy)=>{observed=true;break;},Ok(guard)=>drop(guard),Err(error)=>panic!("unexpected goal guard error: {error:?}")};std::thread::sleep(std::time::Duration::from_millis(1));}
    assert!(observed,"the production recovery must hold the goal guard while waiting for authority");assert_eq!(store.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"concurrent-control".into(),crate::pet_runtime::model::Action::Cancel,crate::assistant_memory::now()).err(),Some(crate::pet_runtime::model::Error::Busy));drop(lock);
    let view=recovery.join().unwrap().unwrap();assert_eq!(view.phase,Phase::Finished);assert!(!view.execution_available&&!view.recovery.accounting_frozen);assert_eq!(store.read(&g.id,&g.spec.owner.pet_id).unwrap().revision,g.revision);assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
    let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);let again=store.recover_step_native(&authority,&input(&g,&ready.scope),crate::assistant_memory::now()).unwrap();assert_eq!(again.phase,Phase::Finished);assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]fn native_receipt_recovery_rejects_stale_foreign_and_writer_locked_sources_without_changes(){
    let(f,mut a,g,_p,o,_main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Bound original receipt source");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::AfterStepEvidencePublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
    let authority=root(&f);let store=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&authority).unwrap().unwrap();let foreign=f.path.join("foreign-authority");Authority::ensure(&foreign).unwrap();let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);
    for field in 0..4{let mut request=input(&g,&ready.scope);match field{0=>request.revision=nonce().unwrap(),1=>request.pet_id="agent_codex".into(),2=>request.decision_id=nonce().unwrap(),_=>request.grant_id=nonce().unwrap()};assert!(store.recover_step_native(&authority,&request,crate::assistant_memory::now()).is_err());assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);}
    assert!(store.recover_step_native(&foreign,&input(&g,&ready.scope),crate::assistant_memory::now()).is_err());let writer=store.test_lock(true).unwrap();assert_eq!(store.recover_step_native(&authority,&input(&g,&ready.scope),crate::assistant_memory::now()).err(),Some(crate::pet_runtime::model::Error::Busy));drop(writer);assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]fn native_receipt_recovery_does_not_infer_a_missing_original_result_from_the_current_player_state(){
    let(f,mut a,g,_p,o,_main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Unproven original result");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepEvidencePublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert_eq!(player.state.lock().unwrap().status,"Paused");
    let authority=root(&f);let store=crate::pet_runtime::store::Store::existing_verified(&f.path.join("goals"),&authority).unwrap().unwrap();let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);assert!(store.recover_step_native(&authority,&input(&g,&ready.scope),crate::assistant_memory::now()).is_err());assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"started");assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]fn aggregate_reconciliation_distinguishes_native_tool_observation_from_missing_effect_without_repair_or_dispatch(){
    use crate::pet_runtime::{reconciliation_tests::inventory,reconciliation::{EvidenceKind,EvidenceStatus}};
    for fault in [Fault::BeforeStepEvidencePublish,Fault::AfterStepEvidencePublish,Fault::BeforeStepResultPublish]{
        let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private read-only reconciliation");let ready=grant(&a,&g,&o,&bus);a.fault=fault;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());
        let before=inventory(&f.path);let view=f.store().reconciliation(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,crate::assistant_memory::now()).unwrap();let witnessed=!matches!(fault,Fault::BeforeStepEvidencePublish);assert_eq!(view.attempts.len(),1);assert_eq!(view.attempts[0].outcome_known,witnessed);assert_eq!(view.accounting.as_ref().unwrap().unknown_attempts,usize::from(!witnessed));assert!(!view.criteria_verified&&!view.goal_complete&&!view.execution_available&&!view.recovery_available);
        assert_eq!(view.evidence.iter().any(|e|e.kind==EvidenceKind::ToolObservation&&e.status==EvidenceStatus::Verified),witnessed);assert_eq!(inventory(&f.path),before);assert_eq!(player.calls.lock().unwrap().len(),1);
    }
}

#[test]
fn final_receipt_publication_failure_is_explicitly_recovered_without_another_effect_or_budget_charge(){
    for fault in [Fault::BeforeStepResultPublish,Fault::AfterStepResultPublish]{
        let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=fault;
        assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
        let before=ledger(&f,&g);let view=assess(&a,&g,&ready.scope);
        assert_eq!(view.recovery.status,if matches!(fault,Fault::BeforeStepResultPublish){RecoveryStatus::ResultRecorded}else{RecoveryStatus::Finished});
        assert_eq!(view.recovery.recorded_effect_verified,Some(true));
        signal_stop(o.lease());drop(o);let reopened=Authority::existing(&root(&f)).unwrap();
        let saved=recover(&reopened,&g,&ready.scope);assert_eq!(saved.record.phase,Phase::Finished);let result=saved.record.result.as_ref().unwrap();
        assert!(result.observation.effect_verified);assert!(!result.criteria_verified&&!result.goal_complete);assert_eq!(ledger(&f,&g),before);
        let again=recover(&reopened,&g,&ready.scope);assert_eq!(serde_json::to_value(&saved.record).unwrap(),serde_json::to_value(&again.record).unwrap());
        assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(before["attempts"].as_array().unwrap().len(),1);
    }
}

#[test]
fn canonical_evidence_with_uncertain_ack_recovers_the_same_started_attempt_after_reopen(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::AfterStepEvidencePublish;
    assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"started");assert!(evidence_path(&f,&ready.scope).exists());
    let before=assess(&a,&g,&ready.scope);assert_eq!(before.recovery.status,RecoveryStatus::ResultRecorded);assert!(before.recovery.reconciliation_available);assert_eq!(before.recovery.budget_phase,Some(RecoveryBudget::Started));
    drop(o);drop(a);let reopened=Authority::existing(&root(&f)).unwrap();let result=recover(&reopened,&g,&ready.scope);
    assert_eq!(result.recovery.status,RecoveryStatus::Finished);assert!(!result.recovery.reconciliation_available);assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"settled");assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn missing_canonical_evidence_preserves_unknown_effect_and_never_retries_the_command(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepEvidencePublish;
    assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert!(!evidence_path(&f,&ready.scope).exists());
    let before=ledger(&f,&g);let saved=assess(&a,&g,&ready.scope);assert_eq!(saved.recovery.status,RecoveryStatus::ResultMissing);assert_eq!(saved.recovery.recorded_effect_verified,None);assert!(!saved.recovery.reconciliation_available);
    let row=&g.admissions[0];assert!(matches!(a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&ready.scope,crate::assistant_memory::now()),Err(Failure::RecoveryRequired)));
    assert!(o.run_step(&g,&ready,&config()).is_err());assert_eq!(player.calls.lock().unwrap().len(),1);assert_eq!(ledger(&f,&g),before);
}

#[test]
fn evidence_survives_both_sides_of_accounting_publication_without_double_charging(){
    for fault in [Fault::BeforeStepBudgetFinish,Fault::AfterStepBudgetFinish]{
        let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=fault;
        assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
        let first=assess(&a,&g,&ready.scope);assert_eq!(first.recovery.status,RecoveryStatus::ResultRecorded);let original=ledger(&f,&g)["attempts"][0]["id"].clone();
        let saved=recover(&a,&g,&ready.scope);assert_eq!(saved.record.phase,Phase::Finished);let usage=ledger(&f,&g);assert_eq!(usage["attempts"].as_array().unwrap().len(),1);assert_eq!(usage["attempts"][0]["id"],original);assert_eq!(usage["attempts"][0]["phase"],"settled");assert_eq!(player.calls.lock().unwrap().len(),1);
    }
}

#[test]
fn interrupted_recovery_itself_is_idempotent_across_accounting_and_receipt_boundaries(){
    for fault in [Fault::BeforeStepBudgetFinish,Fault::AfterStepBudgetFinish,Fault::BeforeStepResultPublish,Fault::AfterStepResultPublish]{
        let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::AfterStepEvidencePublish;
        assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=fault;let row=&g.admissions[0];
        assert!(a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&ready.scope,crate::assistant_memory::now()).is_err());a.fault=Fault::None;
        assert_eq!(recover(&a,&g,&ready.scope).record.phase,Phase::Finished);let settled=ledger(&f,&g);recover(&a,&g,&ready.scope);assert_eq!(ledger(&f,&g),settled);assert_eq!(settled["attempts"].as_array().unwrap().len(),1);assert_eq!(player.calls.lock().unwrap().len(),1);
    }
}

#[test]
fn assessment_reads_all_transaction_gaps_without_inventing_success_or_live_workers(){
    for stage in 0..7{
        let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);let row=&g.admissions[0];let proof=row.proof.as_ref().unwrap();let native=attempt(&ready.scope);let now=crate::assistant_memory::now();
        if stage>0{a.reserve_step(&row.spec,proof,o.lease(),&ready.scope,&native,now).unwrap();}
        if stage>1{a.reserve_attempt(&row.spec,proof,o.lease(),&native,now).unwrap();}
        if stage>2{a.start_attempt(&row.spec,proof,o.lease(),&native,now).unwrap();}
        if stage>3{a.start_step(&row.spec,proof,o.lease(),&ready.scope,&native,now).unwrap();}
        if stage>4{a.finish_attempt(&row.spec,proof,o.lease(),&native,if stage==5{Some(100)}else{None},now).unwrap();}
        let before=ledger(&f,&g);let state=fs::read(path(&f,&ready.scope)).unwrap();let assessment=assess(&a,&g,&ready.scope);
        assert_eq!(assessment.recovery.status,if stage==0{RecoveryStatus::Prepared}else if stage<4{RecoveryStatus::Reserved}else{RecoveryStatus::ResultMissing});
        assert_eq!(assessment.recovery.recorded_effect_verified,None);assert!(!assessment.recovery.reconciliation_available);
        let view:step_api::View=assessment.into();assert!(!view.execution_available);assert_eq!(o.workers(),0);assert_eq!(ledger(&f,&g),before);assert_eq!(fs::read(path(&f,&ready.scope)).unwrap(),state);assert!(player.calls.lock().unwrap().is_empty());
    }
}

#[test]
fn recovery_preserves_transport_uncertainty_and_native_verification_as_separate_facts(){
    let(_f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);player.mode.store(2,Ordering::Release);a.fault=Fault::BeforeStepResultPublish;
    assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;let saved=recover(&a,&g,&ready.scope).record.result.unwrap();
    assert_eq!(saved.observation.delivery,Delivery::Uncertain);assert!(saved.observation.effect_verified);assert!(!saved.criteria_verified&&!saved.goal_complete);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn stop_during_native_io_remains_unverified_after_later_receipt_recovery(){
    let(_f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");player.delay.store(250,Ordering::Release);let ready=grant(&a,&g,&o,&bus);let scope=ready.scope.clone();a.fault=Fault::BeforeStepResultPublish;
    let owner=Arc::new(o);let running=owner.clone();let goal=g.clone();let thread=thread::spawn(move||running.run_step_with_authority(&goal,&ready,&config(),&a));
    let deadline=Instant::now()+Duration::from_secs(3);while !player.entered.load(Ordering::Acquire){assert!(Instant::now()<deadline);thread::sleep(Duration::from_millis(2));}
    signal_stop(owner.lease());assert!(thread.join().unwrap().is_err());drop(owner);
    let a=Authority::existing(&root(&_f)).unwrap();let before=assess(&a,&g,&scope);assert_eq!(before.recovery.recorded_effect_verified,Some(false));
    let saved=recover(&a,&g,&scope).record.result.unwrap();assert!(!saved.observation.scope_current&&!saved.observation.effect_verified);assert!(saved.observation.after.is_some());assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn a_non_dispatched_native_observation_recovers_without_creating_a_tool_attempt(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);drop(player);let replacement=Player::new(&bus,"Private recovery player");a.fault=Fault::BeforeStepResultPublish;
    assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;let saved=recover(&a,&g,&ready.scope).record;
    assert!(saved.attempt.is_none());assert_eq!(saved.result.unwrap().observation.delivery,Delivery::NotDispatched);assert!(ledger(&f,&g).is_null());assert!(replacement.calls.lock().unwrap().is_empty());
}

#[test]
fn forged_or_missing_native_records_cannot_be_recovered_or_erase_budget_holds(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let _player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepResultPublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
    let file=evidence_path(&f,&ready.scope);let raw=fs::read(&file).unwrap();let before=ledger(&f,&g);let row=&g.admissions[0];
    let mut forged:Value=serde_json::from_slice(&raw).unwrap();forged["body"]["result"]["goalComplete"]=json!(true);fs::write(&file,serde_json::to_vec(&forged).unwrap()).unwrap();
    assert!(a.read_step_assessment(&row.spec,row.proof.as_ref().unwrap(),&ready.scope.plan_id,&ready.scope.step_id).is_err());assert!(a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&ready.scope,crate::assistant_memory::now()).is_err());assert_eq!(ledger(&f,&g),before);
    fs::write(&file,&raw).unwrap();fs::remove_file(&file).unwrap();assert_eq!(assess(&a,&g,&ready.scope).recovery.status,RecoveryStatus::ResultMissing);assert!(a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&ready.scope,crate::assistant_memory::now()).is_err());assert_eq!(ledger(&f,&g),before);
}

#[test]
fn exact_revision_owner_and_original_grant_are_required_and_caller_results_are_rejected(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepResultPublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
    for field in ["revision","goalId","petId","decisionId","planId","stepId","grantId"]{
        let original=input(&g,&ready.scope);let mut v=json!({"petId":original.pet_id,"goalId":original.goal_id,"decisionId":original.decision_id,"planId":original.plan_id,"stepId":original.step_id,"revision":original.revision,"grantId":original.grant_id});v[field]=json!("foreign");let changed:RecoverInput=serde_json::from_value(v).unwrap();assert!(step_api::recover_for_goal(&a,&g,&changed,crate::assistant_memory::now()).is_err());
    }
    let mut forged=json!({"petId":g.spec.owner.pet_id,"goalId":g.id,"decisionId":ready.scope.decision_id,"planId":ready.scope.plan_id,"stepId":ready.scope.step_id,"revision":g.revision,"grantId":ready.scope.grant_id});forged["result"]=json!({"effectVerified":true});assert!(serde_json::from_value::<RecoverInput>(forged).is_err());
    let before=ledger(&f,&g);let saved=step_api::recover_for_goal(&a,&g,&input(&g,&ready.scope),crate::assistant_memory::now()).unwrap();assert!(!saved.execution_available);assert_eq!(saved.phase,Phase::Finished);assert_eq!(ledger(&f,&g),before);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn legacy_final_receipts_still_read_and_stabilize_without_manufacturing_new_evidence(){
    let(f,a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);let original=o.run_step(&g,&ready,&config()).unwrap();fs::remove_file(evidence_path(&f,&ready.scope)).unwrap();
    assert_eq!(assess(&a,&g,&ready.scope).recovery.status,RecoveryStatus::Finished);let recovered=recover(&a,&g,&ready.scope);assert_eq!(serde_json::to_value(&original).unwrap(),serde_json::to_value(&recovered.record).unwrap());assert!(!evidence_path(&f,&ready.scope).exists());assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn recorded_result_is_recovered_in_a_new_process_after_pause_without_a_live_target(){
    let(f,mut a,g,p,o,main)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepResultPublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert_eq!(player.calls.lock().unwrap().len(),1);
    let now=crate::assistant_memory::now();let stopping=crate::pet_runtime::authority_store::mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,"pause".into(),Action::Pause,now).unwrap().record;assert_eq!(stopping.state,State::Stopping);
    let drain=o.drain(DrainOutcome::Confirmed).unwrap();assert_eq!(f.store().drain_owned(&a,&drain,now).unwrap().record.state,State::Paused);drop(drain);drop(o);drop(p);drop(main);drop(a);drop(ready);drop(player);drop(bus);
    let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","pet_runtime::lease_controller::writer::tests::step_tests::recovery_tests::recover_child","--ignored"]).env("COUCOU_STEP_RECOVERY_TEST_ROOT",&f.path).env("COUCOU_STEP_RECOVERY_TEST_GOAL",&g.id).output().unwrap();
    assert!(child.status.success(),"{}",String::from_utf8_lossy(&child.stdout));assert!(String::from_utf8_lossy(&child.stdout).contains("1 passed"));
    let a=Authority::existing(&root(&f)).unwrap();let s=a.read_step(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&g.plans[0].id,&g.plans[0].steps[0].id).unwrap().unwrap();assert_eq!(s.phase,Phase::Finished);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
}

#[test]
fn recovering_a_receipt_does_not_silently_resume_a_goal_that_requires_recovery(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepResultPublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
    signal_stop(o.lease());let drain=o.drain(DrainOutcome::Confirmed).unwrap();let stopped=f.store().drain_owned(&a,&drain,crate::assistant_memory::now()).unwrap().record;assert_eq!(stopped.state,State::RecoveryRequired);drop(drain);drop(o);
    let result=step_api::recover_for_goal(&a,&stopped,&input(&stopped,&ready.scope),crate::assistant_memory::now()).unwrap();assert_eq!(result.phase,Phase::Finished);assert!(!result.execution_available);assert_eq!(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap().state,State::RecoveryRequired);assert_eq!(player.calls.lock().unwrap().len(),1);
}
#[test]#[ignore="child helper runs only with the private recovery fixture"]
fn recover_child(){
    let root=std::path::PathBuf::from(std::env::var_os("COUCOU_STEP_RECOVERY_TEST_ROOT").expect("private root required"));let id=std::env::var("COUCOU_STEP_RECOVERY_TEST_GOAL").unwrap();let g=crate::pet_runtime::store::Store::existing(&root.join("goals")).unwrap().unwrap().read(&id,"ai_ollama").unwrap();assert_eq!(g.state,State::Paused);
    let a=Authority::existing(&root.join("authority")).unwrap();let row=&g.admissions[0];let record=a.read_step(&row.spec,row.proof.as_ref().unwrap(),&g.plans[0].id,&g.plans[0].steps[0].id).unwrap().unwrap();let result=step_api::recover_for_goal(&a,&g,&input(&g,&record.scope),crate::assistant_memory::now()).unwrap();
    assert_eq!(result.phase,Phase::Finished);assert!(!result.execution_available);assert_eq!(result.recovery.status,RecoveryStatus::Finished);let r=result.result.unwrap();assert!(r.observation.effect_verified);assert!(!r.criteria_verified&&!r.goal_complete);assert!(a.verify_queued(&g,crate::assistant_memory::now()).is_err());
}

#[test]
fn recovery_does_not_refund_an_uncertain_attempt_even_when_native_observation_was_recorded(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::AfterStepEvidencePublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
    let row=&g.admissions[0];let stored=stored(&a,&g,&ready.scope);a.finish_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),stored.attempt.as_ref().unwrap(),None,crate::assistant_memory::now()).unwrap();let before=ledger(&f,&g);
    assert_eq!(before["attempts"][0]["phase"],"uncertain");let recovered=recover(&a,&g,&ready.scope);assert_eq!(recovered.record.phase,Phase::Finished);assert_eq!(recovered.recovery.budget_phase,Some(RecoveryBudget::Uncertain));assert_eq!(ledger(&f,&g),before);assert_eq!(player.calls.lock().unwrap().len(),1);
}

#[test]
fn validly_signed_evidence_from_another_step_of_the_same_goal_cannot_be_substituted(){
    let proposed=||StepInput{description:"Native pause".into(),postcondition:"Paused independently observed".into(),operation:Some(plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:json!({"action":"pause"})})};
    let(f,mut a,g,_p,o,_)=setup_plan_custom(vec![proposed(),proposed()],2,Arc::new(|_,_,_|Validity::Current));let bus=Bus::new();let player=Player::new(&bus,"Private recovery player");let first=grant(&a,&g,&o,&bus);o.run_step(&g,&first,&config()).unwrap();
    let plan=&g.plans[0];let review=o.review_step(&g,&plan.id,&plan.steps[1].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let(approved,second)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];a.issue_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();
    a.fault=Fault::BeforeStepResultPublish;assert!(o.run_step_with_authority(&g,&second,&config(),&a).is_err());a.fault=Fault::None;
    let file=evidence_path(&f,&second.scope);let original=fs::read(&file).unwrap();fs::write(&file,fs::read(evidence_path(&f,&first.scope)).unwrap()).unwrap();let before=ledger(&f,&g);
    assert!(a.read_step_assessment(&row.spec,row.proof.as_ref().unwrap(),&second.scope.plan_id,&second.scope.step_id).is_err());assert!(a.recover_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&second.scope,crate::assistant_memory::now()).is_err());assert_eq!(ledger(&f,&g),before);
    fs::write(&file,original).unwrap();assert_eq!(recover(&a,&g,&second.scope).record.phase,Phase::Finished);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
}

#[test]fn native_uncertain_state_manifest_requires_the_original_signed_step_and_preserves_its_unknown_charge(){
    for fault in [Fault::BeforeStepEvidencePublish,Fault::AfterStepEvidencePublish]{
        let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Native uncertain state assignment");let ready=grant(&a,&g,&o,&bus);a.fault=fault;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;
        let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);let manifest=a.test_uncertain_effects(&g).unwrap();assert_eq!(manifest.len(),usize::from(matches!(fault,Fault::BeforeStepEvidencePublish)));
        if let Some(effect)=manifest.first(){assert_eq!(effect.scope_hash,fingerprint(&ready.scope).unwrap());assert_eq!(effect.grant_id,ready.scope.grant_id);assert_eq!(effect.operation.arguments,serde_json::json!({"action":"pause"}));assert!(effect.target_label.contains("Native uncertain state assignment"));assert!(effect.valid());}
        assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);assert_eq!(ledger(&f,&g)["attempts"][0]["phase"],"started");assert_eq!(player.calls.lock().unwrap().len(),1);
    }
}
#[test]fn unknown_state_review_cannot_bless_changed_arguments_or_a_corrupt_or_missing_original_step(){
    let(f,mut a,g,_p,o,_)=setup_plan();let bus=Bus::new();let player=Player::new(&bus,"Original unknown scope");let ready=grant(&a,&g,&o,&bus);a.fault=Fault::BeforeStepEvidencePublish;assert!(o.run_step_with_authority(&g,&ready,&config(),&a).is_err());a.fault=Fault::None;assert_eq!(a.test_uncertain_effects(&g).unwrap().len(),1);
    let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments=serde_json::json!({"action":"next"});let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);assert!(a.test_uncertain_effects(&changed).is_err());assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);
    let file=path(&f,&ready.scope);let original=fs::read(&file).unwrap();let mut corrupt:serde_json::Value=serde_json::from_slice(&original).unwrap();corrupt["tag"]=serde_json::json!("00".repeat(32));fs::write(&file,serde_json::to_vec(&corrupt).unwrap()).unwrap();let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);assert!(a.test_uncertain_effects(&g).is_err());assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);
    fs::remove_file(&file).unwrap();let before=crate::pet_runtime::reconciliation_tests::inventory(&f.path);assert!(a.test_uncertain_effects(&g).is_err());assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&f.path),before);assert_eq!(player.calls.lock().unwrap().len(),1);
}
