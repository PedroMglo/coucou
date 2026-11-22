use super::*;
use crate::pet_runtime::{admission_tests::{Fixture,evidence},authority_tests::{root,queued}};
use std::{fs::OpenOptions,os::fd::AsRawFd};
fn fixture()->(Fixture,Authority,Record,Chat,Controller){let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("codex"),&a);let main=Chat::persistent(f.path.join("claim-live.json"));let c=Controller::new().unwrap();(f,a,g,main,c)}
fn prepare(f:&Fixture,g:&Record,main:&Chat,c:&Controller)->model::Result<Pending>{c.prepare(g,&root(f),&f.path.join("contexts"),main,&evidence(g,100_006),"original-claim".into(),100_006,Arc::new(|_,_,_|Validity::Current))}
#[test]fn native_queue_verification_coexists_with_real_shared_authority_readers(){
    let(f,_a,g,main,c)=fixture();let file=OpenOptions::new().read(true).write(true).open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);
    let pending=prepare(&f,&g,&main,&c).unwrap();assert_eq!(pending.lease().source_revision,g.revision);assert_eq!(pending.workers(),0);assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().state,model::State::Queued);
}
#[test]fn a_real_exclusive_ledger_writer_reports_busy_without_creating_or_losing_a_controller(){
    let(f,a,g,main,c)=fixture();let file=OpenOptions::new().read(true).write(true).open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);
    assert!(matches!(prepare(&f,&g,&main,&c),Err(model::Error::Busy)));assert!(main.runtime_resources().busy().unwrap().is_empty());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().revision,g.revision);drop(file);
    let pending=prepare(&f,&g,&main,&c).unwrap();let(receipt,owner)=pending.claim(&f.store(),&a,&evidence(&g,100_007),100_007).unwrap();assert!(owner.is_some());assert_eq!(receipt.record.admissions.len(),1);assert_eq!(receipt.record.events.iter().filter(|e|matches!(e.kind,model::EventKind::LeaseClaimed)).count(),1);
}
#[test]fn revoked_queued_consent_is_a_real_conflict_not_a_retryable_storage_failure(){
    let(f,a,g,main,c)=fixture();a.request_stop(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,100_005).unwrap();assert!(matches!(prepare(&f,&g,&main,&c),Err(model::Error::Conflict)));assert!(main.runtime_resources().busy().unwrap().is_empty());
}
