use super::{history::*,admission::{Review,QueuedAdmission},admission_tests::{Fixture,evidence,budget},authority::{Authority,Failure},authority_tests::root,authority_store::{mutate_with_authority,Fault},model::{self,Record,Action,EventKind},lease_controller::{Controller,Pending,Owned,Validity},leases::DrainOutcome,store::nonce,effect_budget::{self,Kind,Phase},reconciliation_tests::inventory};
use crate::chat::Chat;
use std::{fs::{self,OpenOptions,File},path::Path,os::fd::AsRawFd,sync::Arc};

fn queued(f:&Fixture,a:&Authority,g:&Record,request:&str,now:u64)->Record {
    let approved=Review::prepare(g,evidence(g,now),budget(),request.into(),nonce().unwrap(),now).unwrap().approve(g,evidence(g,now+1),now+1).unwrap();
    let cert=a.certify(&approved,g,now+2).unwrap();f.store().enqueue_certified(&cert,now+3).unwrap().record
}
fn claim(f:&Fixture,a:&Authority,g:&Record,main:&Chat,request:&str,now:u64)->(Record,Pending,Owned) {
    let controller=Controller::new().unwrap();
    let pending=controller.prepare(g,&root(f),&f.path.join("contexts"),main,&evidence(g,now),request.into(),now,Arc::new(|_,_,_|Validity::Current)).unwrap();
    let(receipt,owner)=pending.claim(&f.store(),a,&evidence(g,now+1),now+1).unwrap();(receipt.record,pending,owner.unwrap())
}
fn read(f:&Fixture,g:&Record,now:u64)->Snapshot{f.store().history(&root(f),&g.id,&g.spec.owner.pet_id,&g.revision,now).unwrap()}
fn attempt(request:&str,now:u64)->effect_budget::Attempt{effect_budget::Attempt{id:nonce().unwrap(),request_id:request.into(),input_hash:"a".repeat(64),kind:Kind::Model,output_reserved:512,phase:Phase::Reserved,accepted_output:None,reserved_at:now,started_at:None,finished_at:None}}
fn start(a:&Authority,row:&QueuedAdmission,owner:&Owned,request:&str,now:u64)->effect_budget::Attempt {
    let call=attempt(request,now);let proof=row.proof.as_ref().unwrap();a.reserve_attempt(&row.spec,proof,owner.lease(),&call,now).unwrap();a.start_attempt(&row.spec,proof,owner.lease(),&call,now+1).unwrap();call
}
fn shared(path:&Path)->File{let f=OpenOptions::new().read(true).write(true).open(path).unwrap();assert_eq!(unsafe{libc::flock(f.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);f}
fn two(provider:&str)->(Fixture,Authority,Record,Pending,Owned,Chat) {
    let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let main=Chat::persistent(f.path.join("live.json"));
    let g=queued(&f,&a,&f.goal(provider),"first-admission",100_002);
    let(g,pending,owner)=claim(&f,&a,&g,&main,"first-claim",100_006);
    start(&a,&g.admissions[0],&owner,"old-unknown-model",100_008);
    mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,"pause-first".into(),Action::Pause,100_011).unwrap();
    let drain=owner.drain(DrainOutcome::Confirmed).unwrap();let paused=f.store().drain_owned(&a,&drain,100_012).unwrap().record;
    pending.hold().release_drained(&drain).unwrap();drop(drain);drop(owner);drop(pending);
    let draft=f.store().mutate(&g.id,&g.spec.owner.pet_id,&paused.revision,"draft-second".into(),Action::ResumeDraft,100_013).unwrap().record;
    let g=queued(&f,&a,&draft,"second-admission",100_014);
    let(g,pending,owner)=claim(&f,&a,&g,&main,"second-claim",100_018);
    let call=start(&a,&g.admissions[1],&owner,"new-known-model",100_020);let row=&g.admissions[1];let proof=row.proof.as_ref().unwrap();
    a.finish_attempt(&row.spec,proof,owner.lease(),&call,Some(37),100_022).unwrap();
    let stored=a.result_attempt(&row.spec,proof,&call.request_id).unwrap().1.unwrap();
    let receipt=a.seal_result(&row.spec,proof,owner.lease(),&stored,"Free synthetic response",37,100_023).unwrap();super::results::publish(&a,&row.spec,proof,&receipt).unwrap();
    (f,a,g,pending,owner,main)
}

#[test]fn empty_history_does_not_create_authority_or_claim_token_cost_metering(){
    let f=Fixture::new();let g=f.goal("ollama");let before=inventory(&f.path);let view=read(&f,&g,100_002);
    assert_eq!(view.cumulative_charge,Some(Charge::default()));assert!(view.admissions.is_empty()&&view.accounting_complete);assert!(view.token_usage.is_none()&&view.cost_micros.is_none());assert!(!view.execution_available&&!view.continuation_available);assert_eq!(inventory(&f.path),before);assert!(!root(&f).exists());
}
#[test]fn all_pets_preserve_unknown_old_charge_and_verify_new_receipt_without_replaying_or_resetting(){
    for provider in ["codex","copilot","antigravity","ollama"]{
        let(f,_a,g,_pending,_owner,_main)=two(provider);let before=inventory(&f.path);
        let _goal=shared(&f.path.join("goals/.catalog.lock"));let _authority=shared(&root(&f).join(".catalog.lock"));
        let view=read(&f,&g,100_024);assert_eq!(view.scope,"allRecordedAdmissions");assert_eq!(view.admissions.len(),2);assert!(view.scan_complete&&view.source_consistent&&view.accounting_complete&&!view.outcomes_complete);
        let charge=view.cumulative_charge.as_ref().unwrap();assert_eq!(charge.model_calls,2);assert_eq!(charge.output_bytes,549);assert_eq!(charge.unknown_model_results,1);assert_eq!(charge.unknown_tool_results,0);
        assert_eq!(view.admissions[0].charge.as_ref().unwrap().output_bytes,512);assert!(!view.admissions[0].attempts[0].outcome_known);assert!(view.admissions[1].attempts[0].outcome_known);
        assert_eq!(view.admissions[0].budget,g.admissions[0].spec.budget);assert_eq!(view.admissions[1].budget,g.admissions[1].spec.budget);assert_eq!(view.provider,provider);
        assert!(view.token_usage.is_none()&&view.cost_micros.is_none());assert!(!view.execution_available&&!view.continuation_available);
        let again=read(&f,&g,100_025);assert_eq!(view.source_hash,again.source_hash);assert_eq!(inventory(&f.path),before);
        let latest=f.store().reconciliation(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,100_025).unwrap();assert_eq!(latest.accounting.unwrap().output_bytes_charged,37);
    }
}
#[test]fn deleting_an_old_admission_from_otherwise_valid_json_cannot_hide_its_signed_grant(){
    let(f,a,mut g,_pending,_owner,_main)=two("ollama");let original=g.admissions.remove(0);
    g.events[original.spec.source_sequence as usize].kind=EventKind::Paused;
    g.events.iter_mut().find(|e|matches!(e.kind,EventKind::LeaseClaimed)).unwrap().kind=EventKind::Paused;
    g.validate().unwrap();let before=inventory(&f.path);let view=a.history(&g,100_024).unwrap();
    assert_eq!(view.unlinked_grants.len(),1);assert_eq!(view.unlinked_grants[0].decision_id,original.spec.decision_id);
    assert!(!view.source_consistent&&!view.accounting_complete);assert!(view.cumulative_charge.is_none());assert_eq!(view.authenticated_subtotal.output_bytes,37);assert_eq!(inventory(&f.path),before);
}
#[test]fn missing_or_forged_old_grant_never_turns_incomplete_history_into_spare_allowance(){
    let(f,_a,g,_pending,_owner,_main)=two("codex");let file=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));let original=fs::read(&file).unwrap();fs::remove_file(&file).unwrap();
    let before=inventory(&f.path);let view=read(&f,&g,100_024);assert!(!view.scan_complete&&!view.accounting_complete);assert!(view.cumulative_charge.is_none()&&view.admissions[0].charge.is_none());assert_eq!(view.authenticated_subtotal.output_bytes,37);assert_eq!(inventory(&f.path),before);
    fs::write(&file,&original).unwrap();let mut forged:serde_json::Value=serde_json::from_slice(&original).unwrap();forged["body"]["budgetUsage"]["attempts"][0]["outputReserved"]=1.into();fs::write(&file,serde_json::to_vec(&forged).unwrap()).unwrap();let before=inventory(&f.path);
    assert!(f.store().history(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,100_025).is_err());assert_eq!(inventory(&f.path),before);
}
#[test]fn invalid_model_receipt_preserves_signed_charges_and_marks_outcomes_unverifiable(){
    let(f,_a,g,_pending,_owner,_main)=two("ollama");let initial=read(&f,&g,100_024);let attempt=&initial.admissions[1].attempts[0];let path=f.path.join("pet-runtime-results-v1").join(&g.admissions[1].spec.decision_id).join(format!("{}.json",attempt.id));
    let mut forged:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();forged["body"]["text"]="forged".into();fs::write(path,serde_json::to_vec(&forged).unwrap()).unwrap();let before=inventory(&f.path);let view=read(&f,&g,100_025);
    assert!(!view.scan_complete&&!view.outcomes_complete);assert!(view.accounting_complete);assert_eq!(view.cumulative_charge.as_ref().unwrap().output_bytes,549);assert_eq!(view.cumulative_charge.as_ref().unwrap().unknown_model_results,2);assert_ne!(view.source_hash,initial.source_hash);assert_eq!(inventory(&f.path),before);
}
#[test]fn unpublished_canonical_or_staged_preparations_are_visible_without_promoting_or_discarding_them(){
    for staged in [false,true]{
        let f=Fixture::new();let g=f.goal("ollama");let mut a=Authority::ensure(&root(&f)).unwrap();let approved=super::admission_tests::approved(&g,"unpublished");
        if staged{a.fault=Fault::BeforeGrantPublish;assert!(a.certify(&approved,&g,100_004).is_err());}else{a.certify(&approved,&g,100_004).unwrap();}
        let before=inventory(&f.path);let view=read(&f,&g,100_005);assert!(view.cumulative_charge.is_none()&&!view.accounting_complete&&!view.source_consistent);assert_eq!(view.unlinked_grants.len(),usize::from(!staged));assert_eq!(view.pending_artifacts,usize::from(staged));assert_eq!(inventory(&f.path),before);
    }
}
#[test]fn genuine_legacy_claim_without_a_budget_ledger_is_unknown_instead_of_zero(){
    let(f,_a,g,_pending,_owner,_main)=two("ollama");let file=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));
    // Reproduce the historical signed schema2 format using only this private
    // fixture's generated key; production code exposes no signing entry point.
    let mut envelope:super::authority::GrantEnvelope=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    envelope.body.schema_version=2;envelope.body.budget_usage=None;envelope.body.runtime_owner=None;assert!(envelope.body.valid());
    let key:serde_json::Value=serde_json::from_slice(&fs::read(root(&f).join(".key.json")).unwrap()).unwrap();let bytes=super::authority::decode(key["key"].as_str().unwrap()).unwrap();let key=ring::hmac::Key::new(ring::hmac::HMAC_SHA256,&bytes);envelope.tag=super::authority::encode(ring::hmac::sign(&key,&super::authority::grant_message(&envelope.body).unwrap()).as_ref());fs::write(file,serde_json::to_vec(&envelope).unwrap()).unwrap();
    let before=inventory(&f.path);let view=read(&f,&g,100_024);assert_eq!(view.admissions[0].ledger_status,LedgerStatus::LegacyUnrecorded);assert!(view.admissions[0].charge.is_none()&&view.cumulative_charge.is_none()&&!view.accounting_complete);assert_eq!(view.authenticated_subtotal.output_bytes,37);assert_eq!(inventory(&f.path),before);
}
#[test]fn history_refuses_foreign_revision_profile_or_writer_contention_without_writes(){
    let(f,_a,g,_pending,_owner,_main)=two("copilot");let before=inventory(&f.path);let store=f.store();
    assert!(store.history(&root(&f),&g.id,"agent_codex",&g.revision,100_024).is_err());assert!(store.history(&root(&f),&g.id,&g.spec.owner.pet_id,&nonce().unwrap(),100_024).is_err());assert!(store.history(&f.path.join("missing"),&g.id,&g.spec.owner.pet_id,&g.revision,100_024).is_err());
    let writer=OpenOptions::new().read(true).write(true).open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(writer.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);assert_eq!(store.history(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,100_024).err(),Some(model::Error::Busy));drop(writer);assert_eq!(inventory(&f.path),before);
    let json=serde_json::json!({"petId":g.spec.owner.pet_id,"goalId":g.id,"revision":g.revision,"continuationAvailable":true,"modelCallsUsed":0});assert!(serde_json::from_value::<super::ReconcileInput>(json).is_err());
}
#[test]fn cumulative_addition_refuses_overflow_atomically_instead_of_wrapping_or_partly_updating(){
    let mut original=Charge{output_bytes:u64::MAX,..Charge::default()};let before=original.clone();assert_eq!(original.checked_add(&Charge{model_calls:1,output_bytes:1,..Charge::default()}).err(),Some(Failure::Capacity));assert_eq!(original,before);
}

#[test]fn individually_signed_legacy_rows_never_claim_an_authenticated_complete_history_head(){
    let(f,a,mut g,_pending,_owner,_main)=two("ollama");let old=g.admissions.remove(0);
    g.events[old.spec.source_sequence as usize].kind=EventKind::Paused;
    g.events.iter_mut().find(|e|matches!(e.kind,EventKind::LeaseClaimed)).unwrap().kind=EventKind::Paused;
    g.validate().unwrap();fs::remove_file(root(&f).join(format!("{}.json",old.spec.decision_id))).unwrap();
    let before=inventory(&f.path);let view=a.history(&g,100_024).unwrap();
    // The remaining row is authentic, but its signature does not bind the
    // omitted earlier history. This projection must not authorize continuation.
    assert_eq!(view.cumulative_charge.as_ref().unwrap().output_bytes,37);
    assert_eq!(view.history_coverage,"listedAdmissionsAndPresentSignedGrants");
    assert!(!view.history_continuity_verified&&!view.continuation_available&&!view.execution_available);
    assert_eq!(inventory(&f.path),before);
}

#[test]fn all_thirty_two_admissions_are_counted_and_other_goals_do_not_change_the_history_source(){
    let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let mut g=f.goal("ollama");
    for index in 0..32 {
        let now=100_002+index*10;g=queued(&f,&a,&g,&format!("admission-{index}"),now);
        if index<31{
            let paused=mutate_with_authority(&f.store(),&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,format!("pause-{index}"),Action::Pause,now+4).unwrap().record;
            g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&paused.revision,format!("draft-{index}"),Action::ResumeDraft,now+5).unwrap().record;
        }
    }
    let before=inventory(&f.path);let view=read(&f,&g,100_400);assert_eq!(view.admissions.len(),32);assert!(view.accounting_complete);assert_eq!(view.cumulative_charge,Some(Charge::default()));assert!(view.admissions.iter().all(|a|a.ledger_status==LedgerStatus::NeverClaimed));assert_eq!(inventory(&f.path),before);
    let other=f.goal("copilot");queued(&f,&a,&other,"other-goal",100_002);
    let before=inventory(&f.path);let again=read(&f,&g,100_401);assert_eq!(again.source_hash,view.source_hash);assert!(again.unlinked_grants.is_empty());assert_eq!(again.admissions.len(),32);assert_eq!(inventory(&f.path),before);
}
