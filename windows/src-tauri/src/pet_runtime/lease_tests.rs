use super::{admission_tests::{Fixture,evidence,budget},admission::Review,authority::{Authority,Failure},authority_tests::{root,queued},lease_controller::{Controller,Pending,Owned,Validity},leases::DrainOutcome,model::*,store::{Store,Fault as StoreFault},authority_store::{Fault as AuthFault,mutate_with_authority}};
use crate::chat::Chat;
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
fn prepare(f:&Fixture,g:&Record,main:&Chat,controller:&Controller,request:&str,now:u64)->Pending{controller.prepare(g,&root(f),&f.path.join("contexts"),main,&evidence(g,now),request.into(),now,Arc::new(|_,_,_|Validity::Current)).unwrap()}
fn claim(p:&Pending,f:&Fixture,a:&Authority,g:&Record,now:u64)->(Record,Owned){let(r,o)=p.claim(&f.store(),a,&evidence(g,now),now).unwrap();assert_ne!(r.status,CommitStatus::Uncertain);(r.record,o.unwrap())}

#[test]fn durable_claim_fixes_original_owner_context_model_and_deadline_before_workers(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal("codex");let g=queued(&f,&draft,&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(leased,owner)=claim(&p,&f,&a,&g,100_007);
 assert_eq!(leased.schema_version,6);assert_eq!(leased.state,State::Leased);assert_eq!(owner.lease().goal_id,g.id);assert_eq!(owner.lease().conversation_id,g.execution_context.as_ref().unwrap().conversation_id);assert_eq!(owner.lease().model,"fixture-free");assert_eq!(owner.lease().deadline_at,g.admissions[0].spec.budget.deadline_at);assert_eq!(owner.workers(),0);assert!(owner.current(100_008));assert_eq!(main.runtime_resources().busy().unwrap().len(),1);
}
#[test]fn manual_and_goal_claims_share_actual_physical_slots_until_io_drains(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal("codex");let g=queued(&f,&draft,&a);let main=Chat::persistent(f.path.join("live-main.json"));let turn=main.begin("codex");let manual=main.lease(&turn);main.cancel_provider("codex");let c=Controller::new().unwrap();
 assert!(c.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,100_006),"claim".into(),100_006,Arc::new(|_,_,_|Validity::Current)).is_err());drop(manual);
 let p=prepare(&f,&g,&main,&c,"claim",100_007);let(_,owner)=claim(&p,&f,&a,&g,100_008);let worker=owner.worker(100_009).unwrap();assert!(main.reserve_conversation("codex",None).is_err());assert!(owner.drain(DrainOutcome::Confirmed).is_err());assert!(!worker.current(100_010));drop(worker);assert!(owner.drain(DrainOutcome::Uncertain).is_ok());
}
#[test]fn native_stop_is_requested_until_the_owned_worker_is_really_gone(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal("codex");let g=queued(&f,&draft,&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(leased,owner)=claim(&p,&f,&a,&g,100_007);let worker=owner.worker(100_008).unwrap();
 let stopping=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&leased.revision,"cancel".into(),Action::Cancel,100_009).unwrap().record;assert_eq!(stopping.state,State::Stopping);assert!(!worker.current(100_010));assert!(owner.worker(100_010).is_err());assert!(owner.drain(DrainOutcome::Confirmed).is_err());assert!(main.reserve_conversation("codex",None).is_err());drop(worker);
 let drain=owner.drain(DrainOutcome::Confirmed).unwrap();let done=f.store().drain_owned(&a,&drain,100_011).unwrap().record;assert_eq!(done.state,State::Cancelled);assert_eq!(done.spec,g.spec);drop(drain);drop(owner);drop(p);assert!(main.reserve_conversation("codex",None).is_ok());
}
#[test]fn replaying_a_claim_only_reconciles_data_and_never_mints_a_second_owner(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let draft=f.goal("codex");let g=queued(&f,&draft,&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(leased,_owner)=claim(&p,&f,&a,&g,100_007);
 let(receipt,second)=p.claim(&f.store(),&a,&evidence(&leased,100_008),100_008).unwrap();assert_eq!(receipt.status,CommitStatus::Unchanged);assert!(second.is_none());assert_eq!(receipt.record.events.iter().filter(|e|matches!(e.kind,EventKind::LeaseClaimed)).count(),1);
}
#[test]fn separate_controller_resources_cannot_double_claim_the_same_provider(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let first=queued(&f,&f.goal("codex"),&a);let second=queued(&f,&f.goal_named("codex","second-goal"),&a);let main=Chat::persistent(f.path.join("instance-one.json"));let other=Chat::persistent(f.path.join("instance-two.json"));let c=Controller::new().unwrap();let d=Controller::new().unwrap();let p=prepare(&f,&first,&main,&c,"claim-one",100_006);let _owned=claim(&p,&f,&a,&first,100_007);
 let q=prepare(&f,&second,&other,&d,"claim-two",100_008);assert!(matches!(q.claim(&f.store(),&a,&evidence(&second,100_009),100_009),Err(Error::Busy)));assert_eq!(f.store().read(&second.id,"agent_codex").unwrap().state,State::Queued);
}
#[test]fn uncertain_goal_publication_keeps_the_original_pending_claim_for_reconciliation(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let mut store=f.store();store.fault=StoreFault::AfterPublish;
 let(r,owner)=p.claim(&store,&a,&evidence(&g,100_007),100_007).unwrap();assert_eq!(r.status,CommitStatus::Uncertain);assert!(owner.is_none());assert!(main.reserve_conversation("codex",None).is_err());store.fault=StoreFault::None;
 let(retry,owner)=p.claim(&store,&a,&evidence(&r.record,100_008),100_008).unwrap();assert_eq!(retry.status,CommitStatus::Unchanged);assert!(owner.is_some());assert_eq!(retry.record.admissions[0].lease.as_ref(),Some(p.lease()));
}
#[test]fn uncertain_private_ledger_claim_does_not_start_or_reassign_a_worker(){
 let f=Fixture::new();let mut a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);a.fault=AuthFault::AfterGrantPublish;
 assert!(p.claim(&f.store(),&a,&evidence(&g,100_007),100_007).is_err());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Queued);a.fault=AuthFault::None;let(_,owner)=claim(&p,&f,&a,&g,100_008);assert_eq!(owner.workers(),0);assert!(owner.current(100_009));
}
#[test]fn failed_goal_claim_can_be_abandoned_only_by_its_unstarted_live_controller(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let mut store=f.store();store.fault=StoreFault::BeforePublish;assert!(p.claim(&store,&a,&evidence(&g,100_007),100_007).is_err());store.fault=StoreFault::None;
 let drain=p.abort().unwrap();let result=store.drain_owned(&a,&drain,200_001).unwrap().record;assert_eq!(result.state,State::RecoveryRequired);assert_eq!(result.spec,g.spec);assert_eq!(result.admissions[0].lease.as_ref(),Some(p.lease()));assert!(matches!(result.events.last().unwrap().kind,EventKind::ClaimAbandoned));assert!(p.claim(&store,&a,&evidence(&result,200_002),200_002).is_err());
}
#[test]fn a_reopened_lease_is_not_a_new_controller_or_a_worker_permit(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(leased,owner)=claim(&p,&f,&a,&g,100_007);drop(owner);drop(p);
 let reopened=Store::existing(&f.path.join("goals")).unwrap().unwrap().read(&g.id,"agent_codex").unwrap();assert_eq!(reopened.revision,leased.revision);let next=Controller::new().unwrap();let other=Chat::persistent(f.path.join("restarted.json"));assert!(next.prepare(&reopened,&root(&f),&f.path.join("contexts"),&other,&evidence(&reopened,100_008),"takeover".into(),100_008,Arc::new(|_,_,_|Validity::Current)).is_err());
}
#[test]fn unverified_worker_end_is_recovery_required_instead_of_goal_completion(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(_,owner)=claim(&p,&f,&a,&g,100_007);let w=owner.worker(100_008).unwrap();drop(w);let drain=owner.drain(DrainOutcome::Uncertain).unwrap();let done=f.store().drain_owned(&a,&drain,100_009).unwrap().record;assert_eq!(done.state,State::RecoveryRequired);assert_eq!(done.spec.criteria,g.spec.criteria);assert_eq!(a.verify_queued(&g,100_010).err(),Some(Failure::Revoked));
}
#[test]fn policy_context_and_deadline_revocation_keep_the_live_worker_slot(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let valid=Arc::new(AtomicBool::new(true));let flag=valid.clone();let p=c.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,100_006),"claim".into(),100_006,Arc::new(move|_,_,_|if flag.load(Ordering::Acquire){Validity::Current}else{Validity::Changed})).unwrap();let(_,owner)=claim(&p,&f,&a,&g,100_007);let worker=owner.worker(100_008).unwrap();valid.store(false,Ordering::Release);assert!(!worker.current(100_009));assert!(main.reserve_conversation("codex",None).is_err());assert!(owner.drain(DrainOutcome::Confirmed).is_err());drop(worker);
}
#[test]fn expiry_does_not_free_a_durable_claim_for_another_instance(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let first=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&first,&main,&c,"claim-one",100_006);let(_,owner)=claim(&p,&f,&a,&first,100_007);let worker=owner.worker(100_008).unwrap();assert!(!worker.current(200_000));
 let draft=f.goal_named("codex","later-goal");let mut limits=budget();limits.deadline_at=300_000;let approved=Review::prepare(&draft,evidence(&draft,200_001),limits,"later-queue".into(),super::store::nonce().unwrap(),200_001).unwrap().approve(&draft,evidence(&draft,200_002),200_002).unwrap();let cert=a.certify(&approved,&draft,200_003).unwrap();let later=f.store().enqueue_certified(&cert,200_004).unwrap().record;let other=Chat::persistent(f.path.join("other-instance.json"));let d=Controller::new().unwrap();let q=prepare(&f,&later,&other,&d,"claim-two",200_005);assert!(matches!(q.claim(&f.store(),&a,&evidence(&later,200_006),200_006),Err(Error::Busy)));drop(worker);
}
#[test]fn all_native_pet_and_service_claims_have_independent_owned_lifetimes(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let mut pending=vec![];let mut owners=vec![];
 for provider in ["codex","copilot","antigravity","ollama","integration_notion"]{let g=queued(&f,&f.goal(provider),&a);let p=prepare(&f,&g,&main,&c,&format!("claim-{provider}"),100_006);let(_,owner)=claim(&p,&f,&a,&g,100_007);assert_eq!(owner.lease().provider,provider);assert!(owner.current(100_008));pending.push(p);owners.push(owner);}
 assert_eq!(main.runtime_resources().busy().unwrap().len(),5);super::lease_controller::signal_stop(owners[0].lease());assert!(!owners[0].current(100_009));for other in &owners[1..]{assert!(other.current(100_009));}
}
#[test]fn stale_scope_or_serialized_claim_data_cannot_supply_an_owned_handle(){
 assert!(serde_json::from_value::<super::lease_api::ClaimInput>(serde_json::json!({"petId":"agent_codex","goalId":"g","revision":"r","requestId":"q","permit":true})).is_err());
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);
 let changed_policy=super::admission::Evidence::observe(&g,&super::admission_tests::descriptor(&g,100_007),"b".repeat(64),100_007,false,false).unwrap();assert!(p.claim(&f.store(),&a,&changed_policy,100_007).is_err());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Queued);
 let mut changed=g.clone();changed.admissions[0].spec.policy_revision="b".repeat(64);let at=changed.admissions[0].spec.source_sequence as usize;changed.events[at].fingerprint=fingerprint(&changed.admissions[0].spec).unwrap();std::fs::write(f.path.join("goals").join(format!("{}.json",g.id)),serde_json::to_vec(&changed).unwrap()).unwrap();let changed_policy=super::admission::Evidence::observe(&changed,&super::admission_tests::descriptor(&changed,100_008),"b".repeat(64),100_008,false,false).unwrap();assert!(p.claim(&f.store(),&a,&changed_policy,100_008).is_err());

}
#[test]fn pause_then_cancel_escalation_waits_for_real_drain_without_spending_a_new_lease(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(leased,owner)=claim(&p,&f,&a,&g,100_007);let worker=owner.worker(100_008).unwrap();let paused=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&leased.revision,"pause".into(),Action::Pause,100_009).unwrap().record;assert_eq!(paused.state,State::Stopping);let cancel=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&paused.revision,"cancel".into(),Action::Cancel,100_010).unwrap().record;assert_eq!(cancel.state,State::Stopping);assert_eq!(cancel.admissions[0].lease.as_ref(),Some(owner.lease()));drop(worker);let drain=owner.drain(DrainOutcome::Confirmed).unwrap();assert_eq!(f.store().drain_owned(&a,&drain,100_011).unwrap().record.state,State::Cancelled);
}

#[test]fn transient_authority_contention_blocks_new_operations_without_cancelling_another_worker(){
 use std::os::fd::AsRawFd;
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=prepare(&f,&g,&main,&c,"claim",100_006);let(_,owner)=claim(&p,&f,&a,&g,100_007);let worker=owner.worker(100_008).unwrap();
 let lock=std::fs::File::open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);assert!(!worker.current(100_009));assert!(main.runtime_resources().busy().unwrap().contains("codex"));assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_UN)},0);assert!(worker.current(100_010));
}
#[test]fn unknown_policy_observation_can_recover_without_an_implicit_stop(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let unknown=Arc::new(AtomicBool::new(false));let flag=unknown.clone();let p=c.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,100_006),"claim".into(),100_006,Arc::new(move|_,_,_|if flag.load(Ordering::Acquire){Validity::Unknown}else{Validity::Current})).unwrap();let(_,owner)=claim(&p,&f,&a,&g,100_007);let worker=owner.worker(100_008).unwrap();unknown.store(true,Ordering::Release);assert!(!worker.current(100_009));unknown.store(false,Ordering::Release);assert!(worker.current(100_010));
}
