//! Real private player/bus plus native admission, journal, publication and
//! approval protocol. No real model/provider quota or original user profile.
use super::{admission::{Review,Payment},admission_tests::{Fixture,evidence,budget},authority::{Authority,Failure},authority_tests::root,criteria::{Input,Predicate,Playback},effect_budget::{Attempt,Kind,Phase},lease_controller::{Controller,Pending,Owned,Validity,Validation,recipe_controller::{Approved,Ready}},leases::DrainOutcome,model::{self,Record,State},planning::Limits,planning_tests::{settle,reconcile,grant},recipes::Outcome,store::{Store,Fault,nonce}};
use crate::{chat::Chat,spotify::goal::{Target,tests::{Bus,Player}}};
use serde_json::json;
use std::{sync::{Arc,atomic::Ordering},thread,time::{Duration,Instant}};

pub(super)fn config()->crate::pet_config::Configuration {let mut c=crate::pet_config::defaults();c.pets.get_mut("ai_ollama").unwrap().permissions.get_or_insert_default().music=true;c}
pub(super)fn native(f:&Fixture)->Store {Store::existing_verified(&f.path.join("goals"),&root(f)).unwrap().unwrap()}
pub(super)fn setup(acknowledge:bool)->(Fixture,Authority,Record,Pending,Owned,Chat) {setup_custom(acknowledge,17,2)}
fn setup_custom(acknowledge:bool,volume:u8,tools:u16)->(Fixture,Authority,Record,Pending,Owned,Chat) {setup_policy(acknowledge,volume,tools,2)}
pub(super)fn setup_policy(acknowledge:bool,volume:u8,tools:u16,no_progress_limit:u8)->(Fixture,Authority,Record,Pending,Owned,Chat) {
    let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal_named_criteria("ollama","generated-effects",vec!["Playback is paused".into(),"Volume is 17 percent".into()]);let at=crate::assistant_memory::now()-100;
    let mut b=budget();b.deadline_at=at+120_000;b.max_output_bytes=32_768;b.max_tool_calls=tools;b.payment=Payment::LocalOnly;
    let l=Limits{max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit};
    let approved=Review::prepare_planning(&draft,evidence(&draft,at),b,l,&config(),"admit".into(),nonce().unwrap(),at).unwrap().approve(&draft,evidence(&draft,at+1),at+1).unwrap();
    let cert=a.certify(&approved,&draft,at+2).unwrap();let queued=f.store().enqueue_certified(&cert,at+3).unwrap().record;let main=Chat::persistent(f.path.join("main-live.json"));
    let goals=f.path.join("goals");let authority=root(&f);let pet=queued.spec.owner.pet_id.clone();let id=queued.id.clone();
    let fence=Arc::new(move|lease:&super::leases::Lease,_:&crate::pet_resources::Hold,mode:Validation<'_>|{
        let g=if let Validation::Commit(g)=mode {g.clone()}else{match Store::existing_verified(&goals,&authority).ok().flatten().and_then(|s|s.read(&id,&pet).ok()){Some(g)=>g,None=>return Validity::Unknown}};
        if g.state==State::Leased&&g.admissions.last().and_then(|a|a.lease.as_ref())==Some(lease)&&g.execution_context.as_ref().is_some_and(|c|c.conversation_id==lease.conversation_id){Validity::Current}else{Validity::Changed}
    });
    let controller=Controller::new().unwrap();let pending=controller.prepare(&queued,&root(&f),&f.path.join("contexts"),&main,&evidence(&queued,at+4),"claim".into(),at+4,fence).unwrap();let(r,o)=pending.claim(&f.store(),&a,&evidence(&queued,at+5),at+5).unwrap();let g=r.record;let o=o.unwrap();
    let output=json!({"kind":"plan","description":"Pause and set volume","steps":[{"description":"Pause","postcondition":"Paused","operation":{"operationId":"spotify.control","version":1,"arguments":{"action":"pause"}}},{"description":"Volume","postcondition":format!("{volume} percent"),"operation":{"operationId":"spotify.control","version":1,"arguments":{"action":"set_volume","volume":volume}}}],"criteria":[{"criterionIndex":0,"predicate":{"kind":"spotifyPlayback","status":"paused"}},{"criterionIndex":1,"predicate":{"kind":"spotifyVolume","percent":17}}]}).to_string();
    settle(&a,&g,&o,"native-plan",&output,at+8);reconcile(&a,&g,&o,"native-plan",at+12).unwrap();let mut store=native(&f);if !acknowledge{store.fault=Fault::AfterPublish;}
    let published=o.publish_generated_plan(&store,"native-plan",at+13).unwrap();assert_eq!(published.acknowledged,acknowledge);(f,a,published.receipt.record,pending,o,main)
}
fn reviewed(o:&Owned,g:&Record,bus:&Bus)->(Approved,Ready) {
    let review=o.review_recipe_with_criteria(g,&g.plans[0].id,"approve-generated",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&[],crate::assistant_memory::now()).unwrap();
    let p=review.proposal();assert_eq!(p.scope.schema_version,3);assert!(p.scope.generated_plan.is_some());assert!(p.scope.verification.as_ref().unwrap().matches(g));assert!(p.scope.steps.iter().all(|s|s.schema_version==2&&s.generated_plan==p.scope.generated_plan));
    review.approve(o,g,&config(),crate::assistant_memory::now()).unwrap()
}
fn register(a:&Authority,g:&Record,o:&Owned,approved:&Approved,ready:&Ready) {let row=&g.admissions[0];let scope=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();}
fn attempt(scope:&super::steps::Scope)->Attempt {Attempt{id:nonce().unwrap(),request_id:scope.request_id.clone(),input_hash:model::fingerprint(scope).unwrap(),kind:Kind::Tool,output_reserved:scope.max_output_bytes,phase:Phase::Reserved,accepted_output:None,reserved_at:crate::assistant_memory::now(),started_at:None,finished_at:None}}

#[test]fn read_only_reconciliation_verifies_native_planning_steps_and_completion_under_shared_catalog_readers(){
    use super::{reconciliation_tests::inventory,reconciliation::{EvidenceKind,EvidenceStatus}};
    let(f,a,g,_p,o,_main)=setup(true);let before=inventory(&f.path);let planned=f.store().reconciliation(&root(&f),&g.id,"ai_ollama",&g.revision,crate::assistant_memory::now()).unwrap();assert!(planned.attempts[0].outcome_known);assert!(planned.attempts[0].evidence.iter().any(|e|e.kind==EvidenceKind::PlanningOutput&&e.status==EvidenceStatus::Verified));assert!(!planned.goal_complete&&!planned.criteria_verified);assert_eq!(inventory(&f.path),before);
    let bus=Bus::new();let player=Player::new(&bus,"Private complete read-only reconciliation");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let store=native(&f);let completed=o.run_recipe_for_store(&store,&g,&ready,||Ok(config()),|_|{}).unwrap();assert!(completed.progress.goal_complete);let goal=store.read(&g.id,"ai_ollama").unwrap();
    let locks:Vec<_>=[f.path.join("goals/.catalog.lock"),root(&f).join(".catalog.lock")].into_iter().map(|p|{let lock=std::fs::OpenOptions::new().read(true).write(true).open(p).unwrap();assert_eq!(unsafe{libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock),libc::LOCK_SH|libc::LOCK_NB)},0);lock}).collect();let before=inventory(&f.path);let view=store.reconciliation(&root(&f),&goal.id,"ai_ollama",&goal.revision,crate::assistant_memory::now()).unwrap();assert!(view.scan_complete&&view.source_consistent&&view.criteria_verified&&view.goal_complete);assert_eq!(view.attempts.len(),3);assert_eq!(view.accounting.as_ref().unwrap().unknown_attempts,0);assert!(view.evidence.iter().any(|e|e.kind==EvidenceKind::Completion&&e.status==EvidenceStatus::Verified));assert!(!view.execution_available&&!view.recovery_available);assert_eq!(inventory(&f.path),before);assert_eq!(player.calls.lock().unwrap().len(),2);drop(locks);
}

#[test]fn model_plan_and_unregistered_native_approval_do_not_dispatch_a_recipe() {
    let(f,a,g,_p,o,_)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private generated player");let(approved,ready)=reviewed(&o,&g,&bus);
    assert!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).is_err());assert!(a.read_recipe(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&g.plans[0].id).unwrap().is_none());assert!(player.calls.lock().unwrap().is_empty());assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
    register(&a,&g,&o,&approved,&ready);let result=o.run_recipe(&g,&ready,||Ok(config()),|_|{}).unwrap();assert!(result.progress.criteria_verified);assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]fn exact_human_generated_recipe_completes_with_native_proof_and_original_shared_budget() {
    let(f,a,g,_p,o,main)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private generated completion");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);
    let result=o.run_recipe_for_store(&native(&f),&g,&ready,||Ok(config()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::GoalCompleted);assert!(result.progress.criteria_verified&&result.progress.goal_complete);
    let completed=native(&f).read(&g.id,"ai_ollama").unwrap();assert_eq!(completed.state,State::Completed);assert_eq!(completed.schema_version,8);a.verify_completed_goal(&completed).unwrap();a.verify_generated_goal(&completed,false).unwrap();
    let usage=grant(&f,&g)["body"]["budgetUsage"].clone();let rows=usage["attempts"].as_array().unwrap();assert_eq!(rows.len(),3);assert_eq!(rows.iter().filter(|r|r["kind"]=="model").count(),1);assert_eq!(rows.iter().filter(|r|r["kind"]=="tool"&&r["phase"]=="settled").count(),2);assert_eq!(o.workers(),0);assert_eq!(main.runtime_resources().busy().unwrap().len(),1);
    let hash=result.progress.verification.unwrap().reference.report_hash;player.state.lock().unwrap().status="Playing".into();player.state.lock().unwrap().volume=0.9;
    let same=o.run_recipe_for_store(&native(&f),&g,&ready,||panic!("completed retry cannot dispatch"),|_|{}).unwrap();assert_eq!(same.progress.verification.unwrap().reference.report_hash,hash);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(grant(&f,&g)["body"]["budgetUsage"],usage);
}
#[test]fn generated_recipe_cannot_replace_the_signed_criterion_mapping() {
    let(_f,_a,g,_p,o,_)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private predicate player");
    let different=vec![Input{criterion_index:0,predicate:Predicate::SpotifyPlayback{status:Playback::Playing}},Input{criterion_index:1,predicate:Predicate::SpotifyVolume{percent:18}}];
    assert!(o.review_recipe_with_criteria(&g,&g.plans[0].id,"change-predicate",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&different,crate::assistant_memory::now()).is_err());assert!(player.calls.lock().unwrap().is_empty());
    let exact=vec![Input{criterion_index:0,predicate:Predicate::SpotifyPlayback{status:Playback::Paused}},Input{criterion_index:1,predicate:Predicate::SpotifyVolume{percent:17}}];assert!(o.review_recipe_with_criteria(&g,&g.plans[0].id,"same-predicate",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&exact,crate::assistant_memory::now()).is_ok());
}
#[test]fn generated_tool_budget_requires_the_signed_approved_reserved_step_not_just_scope_json() {
    let(f,a,g,_p,o,_)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private budget player");let plan=&g.plans[0];let review=o.review_step(&g,&plan.id,&plan.steps[0].id,Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),crate::assistant_memory::now()).unwrap();let scope=review.proposal().scope;let call=attempt(&scope);let row=&g.admissions[0];let proof=row.proof.as_ref().unwrap();let at=crate::assistant_memory::now();
    assert_eq!(a.reserve_attempt(&row.spec,proof,o.lease(),&call,at).err(),Some(Failure::Invalid));assert_eq!(a.reserve_generated_step_budget(&row.spec,proof,o.lease(),&scope,&call,at).err(),Some(Failure::Missing));
    let(approved,ready)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let scope=ready.scope;let call=attempt(&scope);a.issue_step(&row.spec,proof,o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert!(a.reserve_generated_step_budget(&row.spec,proof,o.lease(),&scope,&call,crate::assistant_memory::now()).is_err());
    a.reserve_step(&row.spec,proof,o.lease(),&scope,&call,crate::assistant_memory::now()).unwrap();let mut replaced=call.clone();replaced.id=nonce().unwrap();assert!(a.reserve_generated_step_budget(&row.spec,proof,o.lease(),&scope,&replaced,crate::assistant_memory::now()).is_err());
    a.reserve_generated_step_budget(&row.spec,proof,o.lease(),&scope,&call,crate::assistant_memory::now()).unwrap();assert!(player.calls.lock().unwrap().is_empty());assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),2);assert_eq!(grant(&f,&g)["body"]["schemaVersion"],6);
}
#[test]fn pending_plan_publication_cannot_prepare_effects_until_the_original_ack_is_stabilized() {
    let(f,_a,g,_p,o,_)=setup(false);let bus=Bus::new();let player=Player::new(&bus,"Private pending player");assert!(o.review_recipe_with_criteria(&g,&g.plans[0].id,"pending",Arc::new(Target::fixture_at(&bus.address).unwrap()),&config(),&[],crate::assistant_memory::now()).is_err());assert!(player.calls.lock().unwrap().is_empty());
    assert!(o.publish_generated_plan(&native(&f),"native-plan",crate::assistant_memory::now()).unwrap().acknowledged);let actual=native(&f).read(&g.id,"ai_ollama").unwrap();let _=reviewed(&o,&actual,&bus);assert!(player.calls.lock().unwrap().is_empty());
}
#[test]fn lost_model_proposal_evidence_invalidates_a_retained_generated_recipe_before_io() {
    let(f,a,g,_p,o,_)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private evidence player");let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let reference=&g.generated_plans[0];
    std::fs::remove_file(a.planning_root().unwrap().join(&reference.decision_id).join(format!("{}.json",reference.output.attempt_id))).unwrap();assert!(o.run_recipe(&g,&ready,||Ok(config()),|_|{}).is_err());assert!(player.calls.lock().unwrap().is_empty());assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
}
#[test]fn stop_during_generated_native_io_preserves_unknown_effect_and_never_runs_the_next_step() {
    let(f,a,g,_p,o,main)=setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private stopped player");player.delay.store(500,Ordering::Release);let(approved,ready)=reviewed(&o,&g,&bus);register(&a,&g,&o,&approved,&ready);let o=Arc::new(o);let worker=o.clone();let goal=g.clone();
    let thread=thread::spawn(move||worker.run_recipe(&goal,&ready,||Ok(config()),|_|{}));let until=Instant::now()+Duration::from_secs(3);while !player.entered.load(Ordering::Acquire){assert!(Instant::now()<until);thread::sleep(Duration::from_millis(2));}
    super::lease_controller::signal_stop(o.lease());assert!(o.drain(DrainOutcome::Confirmed).is_err());assert_eq!(main.runtime_resources().busy().unwrap().len(),2);let result=thread.join().unwrap().unwrap();assert!(!result.progress.goal_complete);assert!(!result.progress.criteria_verified);assert!(player.calls.lock().unwrap().iter().all(|c|c.0!="Set"));assert_eq!(o.workers(),0);assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),2);
}
#[test]fn legacy_scope_versions_cannot_be_used_for_an_acknowledged_generated_plan() {
    let(_f,_a,g,_p,o,_)=setup(true);let bus=Bus::new();let _player=Player::new(&bus,"Private scope player");let(_,ready)=reviewed(&o,&g,&bus);let scope=ready.scope();let mut legacy=scope.clone();legacy.schema_version=2;assert!(!legacy.valid());legacy.generated_plan=None;
    for step in &mut legacy.steps{step.schema_version=1;step.generated_plan=None;assert!(!step.belongs(&g.admissions[0].spec,o.lease()));}
    assert!(!legacy.belongs(&g.admissions[0].spec,o.lease()));
}

#[path="planning_feedback_tests.rs"]mod feedback_tests;
