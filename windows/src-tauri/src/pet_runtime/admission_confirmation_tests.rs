use super::*;
use crate::pet_runtime::{admission_tests::{Fixture,review,evidence,descriptor,budget},authority_tests::root,authority_store::Fault as AuthorityFault,store::{nonce,Fault},model::{CommitStatus,State,Action}};
use std::{fs::{self,File,OpenOptions},os::fd::AsRawFd,sync::Arc,collections::BTreeMap};
#[test]fn data_consent_cannot_be_spliced_into_a_legacy_review_without_data(){
    let(f,_g,_cache,_input,p)=fixture();let before=hashes(&f.path);
    let rejected=p.confirm_with_data::<()>(&f.store(),&root(&f),true,Some(crate::pet_memory::context_model::GoalConsent::BoundedGoalContext),|_,_|panic!("wrong data scope must not observe"),||100_004).unwrap();
    assert!(matches!(rejected,ConfirmResult::Blocked{reason:Denial::Invalid,retryable:false}));assert_eq!(hashes(&f.path),before);assert!(matches!(*p.native.lock().unwrap(),Native::Review(_)));
    let receipt=queued(run(&p,&f,100_005).unwrap());assert!(receipt.record.admissions[0].spec.memory_data.is_none());assert!(serde_json::to_value(&receipt.record.admissions[0].spec).unwrap().get("memoryData").is_none());
}
#[test]fn completed_receipt_retention_allows_a_fresh_review_after_native_pause_resume_without_reusing_the_old_consent(){
    let(f,_g,mut cache,input,p)=fixture();let first=queued(run(&p,&f,100_004).unwrap()).record;let original=spec(&p);
    let paused=crate::pet_runtime::authority_store::mutate_with_authority(&f.store(),&root(&f),&first.id,"ai_ollama",&first.revision,"pause-old-admission".into(),Action::Pause,100_005).unwrap().record;
    let resumed=f.store().mutate(&first.id,"ai_ollama",&paused.revision,"resume-new-draft".into(),Action::ResumeDraft,100_006).unwrap().record;assert_eq!(resumed.state,State::Draft);
    let next=cache.insert("settings",review(&resumed,"next-queue",100_007),nonce().unwrap(),100_007).unwrap();assert_ne!(next.token,input.token);assert!(cache.pending.contains_key(&input.token));
    let refs=ConfirmInput{memory_consent:None,pet_id:"ai_ollama".into(),goal_id:first.id.clone(),revision:resumed.revision.clone(),token:next.token,approve:true};let new=cache.lookup("settings",&refs,100_008).unwrap();let second=queued(run(&new,&f,100_008).unwrap()).record;assert_eq!(second.admissions.len(),2);assert_ne!(second.admissions[1].spec.decision_id,original.decision_id);assert_eq!(second.admissions[1].spec.source_revision,resumed.revision);
    let before=hashes(&f.path);let old_ack=queued(p.confirm::<()>(&f.store(),&root(&f),true,|_,_|panic!("the old token is ACK only"),||100_009).unwrap());assert_eq!(old_ack.record.revision,second.revision);assert_eq!(hashes(&f.path),before);
}
#[test]fn an_uncommitted_original_approval_cannot_be_replaced_by_a_different_review(){
    let(f,g,mut cache,input,p)=fixture();let lock=shared(&f);let _=run(&p,&f,100_004).unwrap();
    assert!(matches!(cache.insert("settings",review(&g,"replace-original-approval",100_005),nonce().unwrap(),100_005),Err(Error::Conflict)));assert!(Arc::ptr_eq(&p,&cache.lookup("settings",&input,100_005).unwrap()));drop(lock);assert_eq!(queued(run(&p,&f,100_006).unwrap()).record.admissions.len(),1);
}
fn fixture()->(Fixture,Record,Reviews,ConfirmInput,Arc<Pending>){
    let f=Fixture::new();let g=f.goal("ollama");Authority::ensure(&root(&f)).unwrap();let mut cache=Reviews::default();
    let ticket=cache.insert("settings",review(&g,"original-queue",100_002),nonce().unwrap(),100_002).unwrap();
    let input=ConfirmInput{memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token,approve:true};
    let pending=cache.lookup("settings",&input,100_003).unwrap();(f,g,cache,input,pending)
}
fn run(p:&Pending,f:&Fixture,at:u64)->model::Result<ConfirmResult>{p.confirm(&f.store(),&root(f),true,|g,n|Ok(Ok((evidence(g,n),()))),||at)}
fn shared(f:&Fixture)->File{let file=OpenOptions::new().read(true).write(true).open(root(f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);file}
fn spec(p:&Pending)->AdmissionSpec{let state=p.native.lock().unwrap();let Native::Approved{approved,..}=&*state else{panic!("original native approval missing")};approved.spec().clone()}
fn body(f:&Fixture,s:&AdmissionSpec)->serde_json::Value{serde_json::from_slice::<serde_json::Value>(&fs::read(root(f).join(format!("{}.json",s.decision_id))).unwrap()).unwrap()["body"].clone()}
fn queued(result:ConfirmResult)->model::Receipt{if let ConfirmResult::Queued{receipt}=result{receipt}else{panic!("not queued")}}
fn hashes(path:&std::path::Path)->BTreeMap<String,String>{
    let mut result=BTreeMap::new();fn visit(root:&std::path::Path,at:&std::path::Path,out:&mut BTreeMap<String,String>){for entry in fs::read_dir(at).unwrap(){let p=entry.unwrap().path();if p.is_dir(){visit(root,&p,out)}else{out.insert(p.strip_prefix(root).unwrap().to_string_lossy().into(),crate::pet_runtime::authority::encode(ring::digest::digest(&ring::digest::SHA256,&fs::read(&p).unwrap()).as_ref()));}}}visit(path,path,&mut result);result
}
#[test]fn real_shared_authority_lock_retains_exact_native_approval_and_retries_one_original_admission(){
    let(f,g,mut cache,input,p)=fixture();let lock=shared(&f);let before=p.proposal.clone();
    assert!(matches!(run(&p,&f,100_004).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Busy,retryable:true}));
    let original=spec(&p);assert_eq!(original.approved_at,100_004);assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().state,State::Draft);
    let retry=cache.insert("settings",review(&g,"original-queue",100_005),nonce().unwrap(),100_005).unwrap();assert_eq!(retry.token,input.token);assert_eq!(retry.proposal.expires_at,before.expires_at);assert!(Arc::ptr_eq(&p,&cache.lookup("settings",&input,100_005).unwrap()));drop(lock);
    let receipt=queued(run(&p,&f,100_007).unwrap());assert_eq!(receipt.record.admissions.len(),1);assert_eq!(model::fingerprint(&receipt.record.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());assert_eq!(receipt.record.admissions[0].spec.budget,before.budget);
    let saved=hashes(&f.path);let ack=queued(run(&p,&f,100_008).unwrap());assert_eq!(ack.status,CommitStatus::Unchanged);assert_eq!(ack.record.revision,receipt.record.revision);assert_eq!(hashes(&f.path),saved);
}
#[test]fn store_contention_and_temporary_readiness_do_not_consume_the_review_or_allocate_an_approval(){
    let(f,g,_c,_i,p)=fixture();let lock=OpenOptions::new().read(true).write(true).open(f.path.join("goals/.catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);
    assert!(matches!(run(&p,&f,100_004).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Busy,retryable:true}));assert!(matches!(*p.native.lock().unwrap(),Native::Review(_)));drop(lock);
    let result=p.confirm(&f.store(),&root(&f),true,|_,_|Ok(Err::<(Evidence,()),_>(Denial::NotReady)),||100_005).unwrap();assert!(matches!(result,ConfirmResult::Blocked{reason:Denial::NotReady,retryable:true}));assert!(matches!(*p.native.lock().unwrap(),Native::Review(_)));
    assert_eq!(queued(run(&p,&f,100_006).unwrap()).record.admissions.len(),1);assert_eq!(g.admissions.len(),0);
}
#[test]fn uncertain_authority_publication_reuses_the_original_consent_and_grant_without_reapproval(){
    let(f,_g,_c,_i,p)=fixture();let lock=shared(&f);assert!(matches!(run(&p,&f,100_004).unwrap(),ConfirmResult::AuthorityBlocked{..}));drop(lock);let original=spec(&p);
    {let mut n=p.native.lock().unwrap();if let Native::Approved{authority,..}=&mut*n{authority.fault=AuthorityFault::AfterGrantPublish;}}
    assert!(matches!(run(&p,&f,100_005).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Uncertain,retryable:true}));let grant=body(&f,&original);
    {let mut n=p.native.lock().unwrap();if let Native::Approved{authority,..}=&mut*n{authority.fault=AuthorityFault::None;}}
    let r=queued(run(&p,&f,100_006).unwrap());assert_eq!(body(&f,&original),grant);assert_eq!(model::fingerprint(&r.record.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());assert_eq!(r.record.admissions.len(),1);
}
#[test]fn uncertain_goal_publication_acknowledges_after_expiry_without_fresh_metadata_or_new_writes(){
    let(f,_g,_c,_i,p)=fixture();let mut store=f.store();store.fault=Fault::AfterPublish;
    let r=queued(p.confirm(&store,&root(&f),true,|g,n|Ok(Ok((evidence(g,n),()))),||100_004).unwrap());assert_eq!(r.status,CommitStatus::Uncertain);let before=hashes(&f.path);
    let ack=queued(p.confirm::<()>(&f.store(),&root(&f),true,|_,_|panic!("ACK cannot refresh a model or authorize new work"),||300_000).unwrap());assert_eq!(ack.status,CommitStatus::Unchanged);assert_eq!(ack.record.revision,r.record.revision);assert_eq!(hashes(&f.path),before);
    let repeat=queued(p.confirm::<()>(&f.store(),&root(&f),false,|_,_|panic!("committed admission is only acknowledged"),||300_001).unwrap());assert_eq!(repeat.record.state,State::Queued);assert_eq!(repeat.record.admissions.len(),1);assert_eq!(hashes(&f.path),before);
}
#[test]fn changed_policy_source_or_expiry_cannot_enqueue_and_withdraws_only_the_original_unqueued_grant(){
    for mode in 0..3{
        let(f,g,_c,_i,p)=fixture();let mut store=f.store();store.fault=Fault::BeforePublish;
        assert!(matches!(p.confirm(&store,&root(&f),true,|g,n|Ok(Ok((evidence(g,n),()))),||100_004).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Unavailable,retryable:true}));let original=spec(&p);assert_eq!(body(&f,&original)["phase"],"issued");
        if mode==1{f.store().mutate(&g.id,"ai_ollama",&g.revision,"pause-before-retry".into(),Action::Pause,100_005).unwrap();}
        let at=if mode==2{120_000}else{100_006};let result=p.confirm(&f.store(),&root(&f),true,|g,n|Ok(Ok((if mode==0{Evidence::observe(g,&descriptor(g,n),"b".repeat(64),n,false,false).unwrap()}else{evidence(g,n)},()))),||at).unwrap();
        assert!(matches!(result,ConfirmResult::Blocked{retryable:false,..}));assert_eq!(body(&f,&original)["phase"],"revoked");let final_goal=f.store().read(&g.id,"ai_ollama").unwrap();assert!(final_goal.admissions.is_empty());assert_eq!(final_goal.spec,g.spec);assert!(matches!(*p.native.lock().unwrap(),Native::Ended));
    }
}
#[test]fn decline_after_partial_certification_revokes_only_that_original_unqueued_grant(){
    let(f,g,_c,_i,p)=fixture();let mut store=f.store();store.fault=Fault::BeforePublish;let _=p.confirm(&store,&root(&f),true,|g,n|Ok(Ok((evidence(g,n),()))),||100_004).unwrap();let original=spec(&p);let goal_before=fs::read(f.path.join("goals").join(format!("{}.json",g.id))).unwrap();
    assert!(matches!(p.confirm::<()>(&f.store(),&root(&f),false,|_,_|panic!("decline cannot observe a provider"),||100_005).unwrap(),ConfirmResult::Declined));assert_eq!(body(&f,&original)["phase"],"revoked");assert_eq!(fs::read(f.path.join("goals").join(format!("{}.json",g.id))).unwrap(),goal_before);assert!(run(&p,&f,100_006).is_err());
}
#[test]fn native_transaction_mutex_and_origin_binding_do_not_block_other_cache_entries_or_renew_expired_slots(){
    let(f,g,mut cache,input,p)=fixture();let guard=p.native.lock().unwrap();assert_eq!(run(&p,&f,100_004).err(),Some(Error::Busy));assert!(cache.lookup("foreign-window",&input,100_004).is_err());
    let wrong=ConfirmInput{memory_consent:None,pet_id:"agent_copilot".into(),goal_id:input.goal_id.clone(),revision:input.revision.clone(),token:input.token.clone(),approve:true};assert!(cache.lookup("settings",&wrong,100_004).is_err());
    let now=220_001;let mut limits=budget();limits.deadline_at=400_000;let other=Review::prepare(&g,evidence(&g,now),limits,"separate-queue".into(),nonce().unwrap(),now).unwrap();cache.insert("other-window",other,nonce().unwrap(),now).unwrap();assert!(cache.pending.contains_key(&input.token));assert_eq!(cache.pending.len(),2);drop(guard);
    assert!(matches!(run(&p,&f,now+1).unwrap(),ConfirmResult::Blocked{reason:Denial::Expired,retryable:false}));assert!(f.store().read(&g.id,"ai_ollama").unwrap().admissions.is_empty());
}
#[test]fn replacing_the_authority_does_not_make_a_retained_approval_adopt_a_new_key_or_profile(){
    let(f,g,_c,_i,p)=fixture();let lock=shared(&f);let _=run(&p,&f,100_004).unwrap();drop(lock);let original=spec(&p);let moved=f.path.join("original-authority");fs::rename(root(&f),&moved).unwrap();Authority::ensure(&root(&f)).unwrap();let before=hashes(&f.path);
    assert!(matches!(run(&p,&f,100_005).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Changed,retryable:false}));assert_eq!(hashes(&f.path),before);assert!(!root(&f).join(format!("{}.json",original.decision_id)).exists());assert!(f.store().read(&g.id,"ai_ollama").unwrap().admissions.is_empty());
}
