use super::{admission_tests::Fixture,authority::{Authority,Failure},authority_store::Fault,authority_tests::root,generated_plans,lease_controller::{Owned,Pending},leases::DrainOutcome,model::{self,Record,State,CommitStatus},planning_output::Envelope,planning_tests::{setup,settle,reconcile,output,grant},store::{Store,Fault as StoreFault,nonce}};
use serde_json::json;
fn native(f:&Fixture)->Store {Store::existing_verified(&f.path.join("goals"),&root(f)).unwrap().unwrap()}
fn proposal()->(Fixture,Authority,Record,Pending,Owned,Envelope) {
    let(f,a,g,p,o)=setup();settle(&a,&g,&o,"plan-1",&output().to_string(),100_008);let e=reconcile(&a,&g,&o,"plan-1",100_012).unwrap();(f,a,g,p,o,e)
}
macro_rules! prepare {($a:expr,$g:expr,$o:expr,$e:expr,$gate:expr,$at:expr)=>{$a.prepare_generated_plan(&$g.admissions[0].spec,$g.admissions[0].proof.as_ref().unwrap(),$o.lease(),&$g,&$e,&$gate,$at)}}
macro_rules! ack {($a:expr,$g:expr,$o:expr,$gate:expr,$published:expr,$at:expr)=>{$a.acknowledge_generated_plan(&$g.admissions[0].spec,$g.admissions[0].proof.as_ref().unwrap(),$o.lease(),&$gate,$published,$at)}}
fn write(f:&Fixture,g:&Record) {std::fs::write(f.path.join("goals").join(format!("{}.json",g.id)),serde_json::to_vec(g).unwrap()).unwrap();}

#[test]fn original_controller_publishes_one_acknowledged_plan_without_changing_the_budget() {
    let(f,a,g,_p,o,e)=proposal();let before=grant(&f,&g)["body"]["budgetUsage"].clone();let store=native(&f);
    let result=o.publish_generated_plan(&store,"plan-1",100_013).unwrap();assert!(result.acknowledged);assert_eq!(result.receipt.status,CommitStatus::Durable);
    let actual=store.read(&g.id,"ai_ollama").unwrap();assert_eq!(actual.sequence,g.sequence+1);assert_eq!(actual.schema_version,8);assert_eq!(actual.spec,g.spec);assert_eq!(actual.generated_plans.len(),1);
    assert!(generated_plans::content_matches(&actual,&actual.generated_plans[0],&e));assert!(super::plans::matches(&actual,&actual.admissions[0].spec));assert_eq!(actual.generated_plans[0].proposal_hash,super::plans::fingerprint(&actual).unwrap());
    assert_eq!(grant(&f,&g)["body"]["budgetUsage"],before);assert_eq!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["publication"]["phase"],"acknowledged");
    let again=o.publish_generated_plan(&store,"plan-1",100_014).unwrap();assert!(again.acknowledged);assert_eq!(again.receipt.status,CommitStatus::Unchanged);assert_eq!(again.receipt.record.revision,actual.revision);assert_eq!(o.workers(),0);
    // Acknowledged plan readiness is data, not human effect approval. The shared
    // budget still contains just the original model call and no signed step.
    o.step_goal(&actual,&crate::pet_config::defaults(),100_014).unwrap();
    assert!(a.read_step(&actual.admissions[0].spec,actual.admissions[0].proof.as_ref().unwrap(),&actual.plans[0].id,&actual.plans[0].steps[0].id).unwrap().is_none());
    assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
    a.verify_generated_goal(&actual,false).unwrap();
}
#[test]fn publication_requires_the_original_opaque_gate_and_no_active_workers() {
    let(_f,a,g,_p,o,e)=proposal();let(_other_f,_other_a,_other_g,_other_p,other,_)=proposal();let gate=other.plan_gate().unwrap();assert!(prepare!(a,g,o,e,gate,100_013).is_err());drop(gate);
    let worker=o.worker(100_013).unwrap();assert!(o.plan_gate().is_err());drop(worker);let gate=o.plan_gate().unwrap();assert!(prepare!(a,g,o,e,gate,100_013).is_ok());
}
#[test]fn uncertain_journal_preparation_retains_one_exact_reference_before_canonical_write() {
    let(f,mut a,g,_p,o,e)=proposal();let store=native(&f);let gate=o.plan_gate().unwrap();let original=grant(&f,&g);
    a.fault=Fault::BeforeGrantPublish;assert_eq!(prepare!(a,g,o,e,gate,100_013).err(),Some(Failure::Unavailable));assert_eq!(grant(&f,&g),original);assert_eq!(store.read(&g.id,"ai_ollama").unwrap().revision,g.revision);
    a.fault=Fault::AfterGrantPublish;assert_eq!(prepare!(a,g,o,e,gate,100_013).err(),Some(Failure::Uncertain));a.fault=Fault::None;
    assert!(store.read(&g.id,"ai_ollama").is_err());let saved=grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["publication"]["reference"].clone();
    let cert=prepare!(a,g,o,e,gate,100_014).unwrap();assert_eq!(serde_json::to_value(cert.reference()).unwrap(),saved);assert_eq!(cert.reference().published_at,100_013);
    assert_eq!(store.read_control(&g.id,"ai_ollama").unwrap().revision,g.revision);assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
}
#[test]fn failed_or_uncertain_canonical_publication_never_manufactures_an_ack_proof() {
    let(f,a,g,_p,o,e)=proposal();let mut store=native(&f);let gate=o.plan_gate().unwrap();let cert=prepare!(a,g,o,e,gate,100_013).unwrap();let reference=cert.reference().clone();
    store.fault=StoreFault::BeforePublish;assert!(store.commit_generated(&cert,100_013).is_err());assert_eq!(store.read_control(&g.id,"ai_ollama").unwrap().revision,g.revision);assert!(store.read(&g.id,"ai_ollama").is_err());
    store.fault=StoreFault::AfterPublish;let pending=store.commit_generated(&cert,100_014).unwrap();assert_eq!(pending.receipt.status,CommitStatus::Uncertain);assert!(pending.published.is_none());drop(pending);store.fault=StoreFault::None;
    let candidate=store.read_control(&g.id,"ai_ollama").unwrap();assert_eq!(candidate.generated_plans,vec![reference]);assert!(store.read(&g.id,"ai_ollama").is_err());
    let stable=store.commit_generated(&cert,100_015).unwrap();assert_eq!(stable.receipt.status,CommitStatus::Unchanged);ack!(a,g,o,gate,stable.published.as_ref().unwrap(),100_015).unwrap();assert_eq!(store.read(&g.id,"ai_ollama").unwrap().revision,candidate.revision);
}
#[test]fn uncertain_final_ack_recovers_the_same_plan_event_and_authentic_reference() {
    let(f,mut a,g,_p,o,e)=proposal();let store=native(&f);let gate=o.plan_gate().unwrap();let cert=prepare!(a,g,o,e,gate,100_013).unwrap();let commit=store.commit_generated(&cert,100_013).unwrap();drop(cert);
    a.fault=Fault::BeforeGrantPublish;assert_eq!(ack!(a,g,o,gate,commit.published.as_ref().unwrap(),100_014).err(),Some(Failure::Unavailable));assert!(store.read(&g.id,"ai_ollama").is_err());
    a.fault=Fault::AfterGrantPublish;assert_eq!(ack!(a,g,o,gate,commit.published.as_ref().unwrap(),100_015).err(),Some(Failure::Uncertain));a.fault=Fault::None;
    ack!(a,g,o,gate,commit.published.as_ref().unwrap(),100_016).unwrap();let current=store.read(&g.id,"ai_ollama").unwrap();assert_eq!(current.sequence,g.sequence+1);assert_eq!(current.revision,commit.receipt.record.revision);assert_eq!(current.events.iter().filter(|e|matches!(e.kind,model::EventKind::PlanGenerated)).count(),1);
}
#[test]fn native_reader_detects_deleted_refs_plans_or_model_candidate_substitution() {
    for variant in ["allRemoved","allHistoryRemoved","operation","description","reference"] {
        let(f,_a,g,_p,o,_)=proposal();let store=native(&f);let current=o.publish_generated_plan(&store,"plan-1",100_013).unwrap().receipt.record;
        let mut changed=current.clone();match variant {
            "allRemoved"=>changed=g.clone(),
            "allHistoryRemoved"=>changed=Record::new(g.spec.clone(),g.created_at,nonce().unwrap()).unwrap(),
            "operation"=>changed.plans[0].steps[0].operation.as_mut().unwrap().arguments=json!({"action":"play"}),
            "description"=>changed.tasks[0].description="Replaced outside native proposal".into(),
            "reference"=>{changed.generated_plans[0].proposal_hash="f".repeat(64);let seq=changed.generated_plans[0].target_sequence as usize-1;changed.events[seq].fingerprint=model::fingerprint(&changed.generated_plans[0]).unwrap();},_=>unreachable!(),
        }
        changed.validate().unwrap();write(&f,&changed);assert!(store.read(&g.id,"ai_ollama").is_err(),"{variant}");assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().revision,changed.revision);
    }
}
#[test]fn source_drift_or_missing_native_output_refuses_canonical_preparation() {
    let(f,a,g,_p,o,e)=proposal();let gate=o.plan_gate().unwrap();let mut changed=g.clone();changed.revision=nonce().unwrap();changed.validate().unwrap();assert!(prepare!(a,changed,o,e,gate,100_013).is_err());
    let path=a.planning_root().unwrap().join(&g.admissions[0].spec.decision_id).join(format!("{}.json",e.body.attempt_id));std::fs::remove_file(path).unwrap();assert_eq!(prepare!(a,g,o,e,gate,100_013).err(),Some(Failure::RecoveryRequired));assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().revision,g.revision);
}
#[test]fn stop_during_pending_ack_drains_and_can_stabilize_existing_plan_without_restarting_io() {
    let(f,a,g,_p,o,e)=proposal();let mut store=native(&f);let gate=o.plan_gate().unwrap();let cert=prepare!(a,g,o,e,gate,100_013).unwrap();store.fault=StoreFault::AfterPublish;let pending=store.commit_generated(&cert,100_013).unwrap();let canonical=pending.receipt.record.clone();drop(pending);drop(cert);drop(gate);store.fault=StoreFault::None;
    let stopped=super::authority_store::mutate_with_authority(&store,&root(&f),&g.id,"ai_ollama",&canonical.revision,"stop-pending".into(),model::Action::Pause,100_014).unwrap();assert_eq!(stopped.record.state,State::Stopping);assert!(!o.current(100_014));
    let drained=o.drain(DrainOutcome::Confirmed).unwrap();let paused=store.drain_owned(&a,&drained,100_015).unwrap();assert_eq!(paused.record.state,State::Paused);assert!(store.read(&g.id,"ai_ollama").is_err());
    let stable=o.publish_generated_plan(&store,"plan-1",100_016).unwrap();assert!(stable.acknowledged);assert_eq!(store.read(&g.id,"ai_ollama").unwrap().state,State::Paused);assert!(!o.current(100_016));assert_eq!(grant(&f,&g)["body"]["phase"],"closed");assert_eq!(o.workers(),0);
}
#[test]fn unpublished_preparation_can_be_abandoned_only_after_native_stop_and_drain() {
    let(f,a,g,_p,o,e)=proposal();let store=native(&f);let gate=o.plan_gate().unwrap();let cert=prepare!(a,g,o,e,gate,100_013).unwrap();drop(cert);drop(gate);
    assert!(o.abandon_unpublished_plan(&store,"plan-1",100_014).is_err());
    let stop=super::authority_store::mutate_with_authority(&store,&root(&f),&g.id,"ai_ollama",&g.revision,"pause-unpublished".into(),model::Action::Pause,100_015).unwrap();assert_eq!(stop.record.state,State::Stopping);assert!(o.abandon_unpublished_plan(&store,"plan-1",100_016).is_err());
    let drain=o.drain(DrainOutcome::Confirmed).unwrap();let paused=store.drain_owned(&a,&drain,100_017).unwrap();assert_eq!(paused.record.state,State::Paused);o.abandon_unpublished_plan(&store,"plan-1",100_018).unwrap();o.abandon_unpublished_plan(&store,"plan-1",100_019).unwrap();
    let actual=store.read(&g.id,"ai_ollama").unwrap();assert!(actual.generated_plans.is_empty());assert!(actual.plans.is_empty());assert_eq!(actual.state,State::Paused);assert_eq!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["publication"]["phase"],"abandoned");assert!(o.publish_generated_plan(&store,"plan-1",100_020).is_err());
}
#[test]fn acknowledged_plan_cannot_be_abandoned_after_cancellation() {
    let(f,a,g,_p,o,_)=proposal();let store=native(&f);let current=o.publish_generated_plan(&store,"plan-1",100_013).unwrap().receipt.record;
    let stopped=super::authority_store::mutate_with_authority(&store,&root(&f),&g.id,"ai_ollama",&current.revision,"cancel-acknowledged".into(),model::Action::Cancel,100_014).unwrap();assert_eq!(stopped.record.state,State::Stopping);
    let drain=o.drain(DrainOutcome::Confirmed).unwrap();assert_eq!(store.drain_owned(&a,&drain,100_015).unwrap().record.state,State::Cancelled);
    assert!(o.abandon_unpublished_plan(&store,"plan-1",100_016).is_err());assert_eq!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["publication"]["phase"],"acknowledged");
}

#[test]fn a_shared_native_reader_keeps_publication_busy_without_repeating_the_model_result(){
    use std::{fs::OpenOptions,os::fd::AsRawFd};let(f,_a,g,_pending,owner,_output)=proposal();let store=native(&f);let reader=OpenOptions::new().read(true).write(true).open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(reader.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);let before=super::reconciliation_tests::inventory(&f.path);
    assert_eq!(owner.publish_generated_plan(&store,"plan-1",100_013).err(),Some(model::Error::Busy));assert_eq!(super::reconciliation_tests::inventory(&f.path),before);drop(reader);let receipt=owner.publish_generated_plan(&store,"plan-1",100_014).unwrap();assert!(receipt.acknowledged);assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(receipt.receipt.record.generated_plans.len(),1);
}
