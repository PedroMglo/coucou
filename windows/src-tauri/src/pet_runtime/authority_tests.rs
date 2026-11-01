use super::{authority::{Authority,Failure,ProvenanceStatus},authority_store::{Fault as AuthFault,mutate_with_authority},admission::{Review,QueuePhase},admission_tests::{Fixture,approved,review,evidence,budget},model::*,store::{Fault as StoreFault,nonce}};
use std::{fs,path::Path,os::unix::fs::{PermissionsExt,symlink}};
pub(super)fn root(f:&Fixture)->std::path::PathBuf{f.path.join("authority")}
pub(super)fn queued(f:&Fixture,g:&Record,a:&Authority)->Record{let approved=approved(g,"queue");let cert=a.certify(&approved,g,100_004).unwrap();f.store().enqueue_certified(&cert,100_005).unwrap().record}
fn hash_file(path:&Path)->String{hash(&fs::read(path).unwrap())}
fn refresh_event(g:&mut Record){let spec=&g.admissions.last().unwrap().spec;let at=spec.source_sequence as usize;g.events[at].fingerprint=fingerprint(spec).unwrap();}

fn held_reader(f:&Fixture)->std::fs::File{
 use std::os::fd::AsRawFd;
 let lock=fs::OpenOptions::new().read(true).write(true).open(root(f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);lock
}
#[test]fn native_catalog_read_only_proof_uses_shared_lock_without_obtaining_write_access(){
 use std::os::fd::AsRawFd;
 let f=Fixture::new();let goal=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let queued=queued(&f,&goal,&a);drop(a);
 let path=root(&f).join(".catalog.lock");fs::set_permissions(&path,fs::Permissions::from_mode(0o400)).unwrap();let a=Authority::existing(&root(&f)).unwrap();
 let before=fs::read_dir(root(&f)).unwrap().map(|p|{let p=p.unwrap().path();(p.clone(),fs::read(p).unwrap())}).collect::<Vec<_>>();
 assert!(a.verify_queued(&queued,100_007).is_ok());
 let other=fs::OpenOptions::new().read(true).open(&path).unwrap();assert_eq!(unsafe{libc::flock(other.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);assert_eq!(a.verify_queued(&queued,100_008).err(),Some(Failure::Busy));drop(other);assert!(a.verify_queued(&queued,100_009).is_ok());
 let new_goal=f.goal_named("codex","new-readonly-proof");let new_approval=approved(&new_goal,"queue");assert_eq!(a.certify(&new_approval,&new_goal,100_004).err(),Some(Failure::Unsafe));assert!(!root(&f).join(format!("{}.json",new_approval.spec().decision_id)).exists());
 for(p,b)in before{assert_eq!(fs::read(p).unwrap(),b);}
}
#[test]fn native_catalog_writer_waits_for_a_transient_reader_without_minting_another_consent(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let a=Authority::ensure(&root(&f)).unwrap();let lock=held_reader(&f);let release=std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(20));drop(lock);});
 let cert=a.certify(&approved,&g,100_004).unwrap();release.join().unwrap();let grant=root(&f).join(format!("{}.json",approved.spec().decision_id));let body:serde_json::Value=serde_json::from_slice(&fs::read(grant).unwrap()).unwrap();assert_eq!(body["body"]["approvedAt"],approved.spec().approved_at);assert_eq!(body["body"]["phase"],"issued");assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Draft);assert_eq!(body["body"]["lease"],serde_json::Value::Null);
 assert_eq!(f.store().enqueue_certified(&cert,100_005).unwrap().record.admissions[0].spec.decision_id,approved.spec().decision_id);
}
#[test]fn native_catalog_writer_exhaustion_is_busy_without_publication_or_renewing_the_choice(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let a=Authority::ensure(&root(&f)).unwrap();let lock=held_reader(&f);let start=std::time::Instant::now();assert_eq!(a.certify(&approved,&g,100_004).err(),Some(Failure::Busy));assert!(start.elapsed()<std::time::Duration::from_secs(2));assert!(!root(&f).join(format!("{}.json",approved.spec().decision_id)).exists());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Draft);drop(lock);
 let cert=a.certify(&approved,&g,100_005).unwrap();let queued=f.store().enqueue_certified(&cert,100_006).unwrap().record;assert_eq!(queued.admissions[0].spec.decision_id,approved.spec().decision_id);assert_eq!(queued.admissions[0].spec.approved_at,approved.spec().approved_at);
}
#[test]fn native_catalog_writer_cannot_adopt_a_replaced_lock_while_waiting(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let a=Authority::ensure(&root(&f)).unwrap();let lock=held_reader(&f);let path=root(&f);let replace=std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(20));fs::rename(path.join(".catalog.lock"),path.join(".original-held-lock")).unwrap();fs::write(path.join(".catalog.lock"),b"").unwrap();fs::set_permissions(path.join(".catalog.lock"),fs::Permissions::from_mode(0o600)).unwrap();drop(lock);});
 assert_eq!(a.certify(&approved,&g,100_004).err(),Some(Failure::Changed));replace.join().unwrap();assert!(!root(&f).join(format!("{}.json",approved.spec().decision_id)).exists());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Draft);
}

#[test]fn native_consent_proof_survives_reopen_without_exporting_a_key_or_permit(){
 let f=Fixture::new();let g=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&g,&a);assert_eq!(g.schema_version,6);assert!(a.verify_queued(&g,100_006).is_ok());drop(a);
 let reopened=Authority::existing(&root(&f)).unwrap();assert!(reopened.verify_queued(&g,100_007).is_ok());let status=serde_json::to_value(ProvenanceStatus::from_result(reopened.verify_queued(&g,100_007))).unwrap();assert_eq!(status["verified"],true);assert_eq!(status["executionAvailable"],false);assert!(status.get("key").is_none());
}
#[test]fn changed_model_budget_policy_or_context_cannot_be_reauthenticated_by_plain_checksums(){
 let f=Fixture::new();let original=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&original,&a);
 for field in ["model","calls","deadline","policy","intent","context"]{
  let mut changed=g.clone();match field{
   "model"=>changed.admissions[0].spec.resolved_model="another-free-fixture".into(),
   "calls"=>changed.admissions[0].spec.budget.max_model_calls+=1,
   "deadline"=>changed.admissions[0].spec.budget.deadline_at+=1,
   "policy"=>changed.admissions[0].spec.policy_revision="b".repeat(64),
   "intent"=>{changed.spec.intent="untrusted replacement intent".into();changed.events[0].fingerprint=fingerprint(&changed.spec).unwrap();changed.admissions[0].spec.intent_hash=fingerprint(&changed.spec).unwrap();},
   "context"=>{changed.execution_context.as_mut().unwrap().seed_hash="b".repeat(64);changed.admissions[0].spec.context=changed.execution_context.clone().unwrap();changed.admissions[0].spec.context_hash=fingerprint(&changed.admissions[0].spec.context).unwrap();},_=>unreachable!(),
  }refresh_event(&mut changed);assert!(changed.validate().is_ok());assert_eq!(a.verify_queued(&changed,100_006).err(),Some(Failure::Invalid));
 }
}
#[test]fn another_profile_or_forged_operation_never_reuses_a_consent(){
 let f=Fixture::new();let g=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&g,&a);let b=Authority::ensure(&f.path.join("other-profile")).unwrap();assert_eq!(b.verify_queued(&g,100_006).err(),Some(Failure::WrongKey));
 let mut json=serde_json::to_value(&g).unwrap();json["admissions"][0]["proof"]["operation"]=serde_json::json!("systemControl");assert!(serde_json::from_value::<Record>(json).is_err());
}
#[test]fn an_unsigned_older_queue_is_not_upgraded_by_opening_the_authority(){
 let f=Fixture::new();let g=f.goal("codex");let mut unsigned=f.store().enqueue(&approved(&g,"queue"),100_004).unwrap().record;
 // Persist an actual legacy v3 / admission-v1 queue, rather than a newly sealed v6 queue.
 unsigned.admissions[0].spec.schema_version=1;unsigned.admissions[0].spec.proposal_hash=None;unsigned.schema_version=3;
 unsigned.events.last_mut().unwrap().fingerprint=crate::pet_runtime::model::fingerprint(&unsigned.admissions[0].spec).unwrap();unsigned.validate().unwrap();
 let old_file=f.path.join("goals").join(format!("{}.json",unsigned.id));let before=serde_json::to_vec(&unsigned).unwrap();std::fs::write(&old_file,&before).unwrap();
 let a=Authority::ensure(&root(&f)).unwrap();assert_eq!(std::fs::read(&old_file).unwrap(),before);assert_eq!(f.store().read(&unsigned.id,&unsigned.spec.owner.pet_id).unwrap().schema_version,3);assert_eq!(a.verify_queued(&unsigned,100_005).err(),Some(Failure::Missing));assert!(serde_json::to_vec(&unsigned).unwrap()==before);assert_eq!(unsigned.schema_version,3);
}
#[test]fn missing_key_is_never_regenerated_in_an_existing_namespace(){
 let f=Fixture::new();let g=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&g,&a);let grant=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));let before=hash_file(&grant);fs::remove_file(root(&f).join(".key.json")).unwrap();assert_eq!(Authority::ensure(&root(&f)).err(),Some(Failure::Missing));assert!(!root(&f).join(".key.json").exists());assert_eq!(hash_file(&grant),before);assert_eq!(a.verify_queued(&g,100_006).err(),Some(Failure::Changed));
}
#[test]fn key_permissions_links_and_namespace_replacement_revoke_live_handles(){
 let f=Fixture::new();let g=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&g,&a);let key=root(&f).join(".key.json");fs::set_permissions(&key,fs::Permissions::from_mode(0o644)).unwrap();assert_eq!(a.verify_queued(&g,100_006).err(),Some(Failure::Changed));assert_eq!(Authority::existing(&root(&f)).err(),Some(Failure::Unsafe));
 fs::set_permissions(&key,fs::Permissions::from_mode(0o600)).unwrap();let alias=f.path.join("key-alias");fs::hard_link(&key,&alias).unwrap();assert_eq!(Authority::existing(&root(&f)).err(),Some(Failure::Unsafe));fs::remove_file(alias).unwrap();
 fs::rename(root(&f),f.path.join("previous-authority")).unwrap();let other=Authority::ensure(&root(&f)).unwrap();assert_eq!(a.verify_queued(&g,100_006).err(),Some(Failure::Changed));assert_eq!(other.verify_queued(&g,100_006).err(),Some(Failure::WrongKey));
}
#[test]fn foreign_protocol_or_symlink_is_never_adopted_as_a_key_namespace(){
 let f=Fixture::new();let path=root(&f);fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();fs::write(path.join("foreign"),b"keep").unwrap();assert!(Authority::ensure(&path).is_err());assert_eq!(fs::read(path.join("foreign")).unwrap(),b"keep");
 let target=f.path.join("real-authority");let _=Authority::ensure(&target).unwrap();let alias=f.path.join("alias-authority");symlink(&target,&alias).unwrap();assert_eq!(Authority::ensure(&alias).err(),Some(Failure::Unsafe));
}
#[test]fn native_pause_revokes_the_independent_grant_before_goal_publication(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);let mut store=f.store();store.fault=StoreFault::BeforePublish;
 assert!(mutate_with_authority(&store,&root(&f),&g.id,"agent_codex",&g.revision,"pause".into(),Action::Pause,100_006).is_err());let stored=store.read(&g.id,"agent_codex").unwrap();assert_eq!(stored.state,State::Queued);assert_eq!(a.verify_queued(&stored,100_007).err(),Some(Failure::Revoked));
 store.fault=StoreFault::None;let paused=mutate_with_authority(&store,&root(&f),&g.id,"agent_codex",&g.revision,"pause".into(),Action::Pause,100_008).unwrap().record;assert_eq!(paused.state,State::Paused);assert_eq!(a.verify_queued(&g,100_009).err(),Some(Failure::Revoked));
}
#[test]fn replaying_an_old_pause_receipt_never_revokes_a_newer_admission(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);let paused=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&g.revision,"pause".into(),Action::Pause,100_006).unwrap().record;
 let draft=f.store().mutate(&g.id,"agent_codex",&paused.revision,"resume".into(),Action::ResumeDraft,100_007).unwrap().record;let approved=review(&draft,"second-queue",100_008).approve(&draft,evidence(&draft,100_009),100_009).unwrap();let cert=a.certify(&approved,&draft,100_010).unwrap();let new=f.store().enqueue_certified(&cert,100_011).unwrap().record;
 let replay=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&g.revision,"pause".into(),Action::Pause,100_012).unwrap();assert_eq!(replay.status,CommitStatus::Unchanged);assert_eq!(replay.record.revision,new.revision);assert!(a.verify_queued(&new,100_013).is_ok());assert_eq!(a.verify_queued(&g,100_013).err(),Some(Failure::Revoked));
}
#[test]fn plain_ledger_edit_cannot_undo_a_durable_native_revocation(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);a.revoke(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),100_006).unwrap();let path=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));let mut json:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();json["body"]["phase"]=serde_json::json!("issued");fs::write(path,serde_json::to_vec(&json).unwrap()).unwrap();assert_eq!(a.verify_queued(&g,100_007).err(),Some(Failure::Invalid));
}
#[test]fn uncertain_grant_publication_cannot_enqueue_without_durable_reconciliation(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let mut a=Authority::ensure(&root(&f)).unwrap();a.fault=AuthFault::AfterGrantPublish;assert_eq!(a.certify(&approved,&g,100_004).err(),Some(Failure::Uncertain));assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Draft);
 a.fault=AuthFault::None;let cert=a.certify(&approved,&g,100_005).unwrap();let queued=f.store().enqueue_certified(&cert,100_006).unwrap().record;assert!(a.verify_queued(&queued,100_007).is_ok());
}
#[test]fn prepublication_grant_failure_leaves_no_queue_or_usable_proof(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let mut a=Authority::ensure(&root(&f)).unwrap();a.fault=AuthFault::BeforeGrantPublish;assert_eq!(a.certify(&approved,&g,100_004).err(),Some(Failure::Unavailable));assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,State::Draft);assert!(!root(&f).join(format!("{}.json",approved.spec().decision_id)).exists());
 a.fault=AuthFault::None;let cert=a.certify(&approved,&g,100_005).unwrap();assert!(f.store().enqueue_certified(&cert,100_006).is_ok());
}
#[test]fn durable_grant_without_a_goal_commit_is_not_an_executable_queue(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let a=Authority::ensure(&root(&f)).unwrap();let cert=a.certify(&approved,&g,100_004).unwrap();let mut store=f.store();store.fault=StoreFault::BeforePublish;assert!(store.enqueue_certified(&cert,100_005).is_err());let stored=store.read(&g.id,"agent_codex").unwrap();assert_eq!(a.verify_queued(&stored,100_006).err(),Some(Failure::NotQueued));store.fault=StoreFault::None;assert!(store.enqueue_certified(&cert,100_007).is_ok());
}
#[test]fn lost_goal_acknowledgment_retries_a_receipt_without_renewing_the_consent(){
 let f=Fixture::new();let g=f.goal("codex");let approved=approved(&g,"queue");let a=Authority::ensure(&root(&f)).unwrap();let cert=a.certify(&approved,&g,100_004).unwrap();let mut store=f.store();store.fault=StoreFault::AfterPublish;assert_eq!(store.enqueue_certified(&cert,100_005).unwrap().status,CommitStatus::Uncertain);store.fault=StoreFault::None;
 let data=store.enqueue_certified(&cert,300_000).unwrap();assert_eq!(data.status,CommitStatus::Unchanged);assert_eq!(data.record.admissions.len(),1);assert_eq!(a.verify_queued(&data.record,300_000).err(),Some(Failure::Expired));
}
#[test]fn colliding_decision_ids_cannot_overwrite_another_goals_consent(){
 let f=Fixture::new();let first=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&first,&a);let other=f.goal("copilot");let review=Review::prepare(&other,evidence(&other,100_006),budget(),"queue".into(),g.admissions[0].spec.decision_id.clone(),100_006).unwrap();let approved=review.approve(&other,evidence(&other,100_007),100_007).unwrap();assert_eq!(a.certify(&approved,&other,100_008).err(),Some(Failure::Invalid));assert!(a.verify_queued(&g,100_009).is_ok());
}
#[test]fn stale_control_revision_does_not_revoke_current_authority(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);assert!(mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&nonce().unwrap(),"cancel".into(),Action::Cancel,100_006).is_err());assert!(a.verify_queued(&g,100_007).is_ok());
}
#[test]fn expiry_still_allows_durable_cancellation_and_old_json_stays_revoked(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);let r=mutate_with_authority(&f.store(),&root(&f),&g.id,"agent_codex",&g.revision,"cancel".into(),Action::Cancel,300_000).unwrap().record;assert_eq!(r.state,State::Cancelled);assert_eq!(r.admissions[0].phase,QueuePhase::Cancelled);assert_eq!(a.verify_queued(&g,300_001).err(),Some(Failure::Revoked));
}

#[test]fn replacing_a_catalog_lock_invalidates_existing_authority_handles(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);fs::rename(root(&f).join(".catalog.lock"),root(&f).join(".old-lock")).unwrap();fs::write(root(&f).join(".catalog.lock"),b"").unwrap();fs::set_permissions(root(&f).join(".catalog.lock"),fs::Permissions::from_mode(0o600)).unwrap();assert_eq!(a.verify_queued(&g,100_006).err(),Some(Failure::Changed));
}
#[test]fn backwards_clock_or_corrupt_key_never_mints_a_new_consent(){
 let f=Fixture::new();let initial=f.goal("codex");assert_eq!(Review::prepare(&initial,evidence(&initial,100_002),budget(),"q".into(),nonce().unwrap(),100_000).err(),Some(super::admission::Denial::Expired));let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&initial,&a);assert_eq!(a.revoke(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),u64::MAX).err(),Some(Failure::Invalid));assert!(a.verify_queued(&g,100_006).is_ok());
 fs::write(root(&f).join(".key.json"),b"invalid").unwrap();assert_eq!(Authority::ensure(&root(&f)).err(),Some(Failure::Invalid));assert_eq!(fs::read(root(&f).join(".key.json")).unwrap(),b"invalid");assert_eq!(a.verify_queued(&g,100_007).err(),Some(Failure::Changed));
}
#[test]fn authority_capacity_refuses_new_consent_without_discarding_existing_metadata(){
 let f=Fixture::new();let initial=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let approval=approved(&initial,"q");for n in 0..4090{fs::write(root(&f).join(format!(".reserved-{n}")),b"keep").unwrap();}
 assert_eq!(a.certify(&approval,&initial,100_004).err(),Some(Failure::Capacity));assert_eq!(fs::read(root(&f).join(".reserved-0")).unwrap(),b"keep");assert_eq!(f.store().read(&initial.id,"agent_codex").unwrap().state,State::Draft);
}

#[test]fn consent_and_revocation_stay_independent_for_all_native_pet_and_service_owners(){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let mut records=vec![];
 for provider in ["codex","copilot","antigravity","ollama","integration_notion"]{let draft=f.goal(provider);let g=queued(&f,&draft,&a);assert_eq!(g.admissions[0].spec.owner.provider,provider);assert!(a.verify_queued(&g,100_006).is_ok());records.push(g);}
 let first=&records[0];a.revoke(&first.admissions[0].spec,first.admissions[0].proof.as_ref().unwrap(),100_007).unwrap();assert_eq!(a.verify_queued(first,100_008).err(),Some(Failure::Revoked));for other in &records[1..]{assert!(a.verify_queued(other,100_008).is_ok());}
}
