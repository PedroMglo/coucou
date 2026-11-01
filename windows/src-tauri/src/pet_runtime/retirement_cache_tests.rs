use super::*;
use crate::pet_runtime::{retirement_tests::{initial,review},authority_store::Fault,retirement_controller::Prepared,reconciliation_tests::inventory,authority_tests::root};
use std::{fs::OpenOptions,os::fd::AsRawFd};
fn setup()->(crate::pet_runtime::admission_tests::Fixture,Store,Record,String,Reviews,RetirementTicket,Arc<Pending>){
    let(f,store,g,decision)=initial("ollama",false);let mut cache=Reviews::default();let ticket=cache.insert("settings",review(&f,&store,&g,&decision,100_040),100_040).unwrap();let input=RetirementConfirmInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:decision.clone(),revision:g.revision.clone(),token:ticket.token.clone(),approve:true};let pending=cache.lookup("settings",&input).unwrap();(f,store,g,decision,cache,ticket,pending)
}
#[test]fn native_scope_cannot_be_switched_between_windows_pets_goals_decisions_or_source_revisions(){
    let(f,store,g,decision,cache,ticket,pending)=setup();let input=||RetirementConfirmInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:decision.clone(),revision:g.revision.clone(),token:ticket.token.clone(),approve:true};let before=inventory(&f.path);
    assert!(cache.lookup("mochi",&input()).is_err());for field in 0..4{let mut wrong=input();match field{0=>wrong.pet_id="agent_codex".into(),1=>wrong.goal_id="different-goal".into(),2=>wrong.decision_id="different-decision".into(),_=>wrong.revision="different-revision".into()};assert!(cache.lookup("settings",&wrong).is_err());}assert_eq!(inventory(&f.path),before);
    assert!(matches!(pending.confirm(&store,false,100_041).unwrap(),RetirementConfirmed::Declined));assert_eq!(inventory(&f.path),before);
}
#[test]fn a_received_retirement_choice_survives_busy_and_expiry_with_one_original_timestamp(){
    let(f,store,g,decision,mut cache,ticket,pending)=setup();let lock=OpenOptions::new().read(true).write(true).open(root(&f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);
    assert!(matches!(pending.confirm(&store,true,100_041).unwrap(),RetirementConfirmed::Blocked{reason:Error::Busy,retryable:true,retained:true}));drop(lock);cache.sweep(400_000);
    let input=RetirementPrepareInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),decision_id:decision,request_id:"changed-ui-request".into()};let retained=cache.find("settings",&input,&g,400_000).unwrap().unwrap();assert_eq!(retained.token,ticket.token);assert_eq!(retained.proposal.approval_scope,"confirmOriginalRetirement");assert_eq!(retained.proposal.prepared_at,100_040);
    let receipt=match pending.confirm(&store,true,400_001).unwrap(){RetirementConfirmed::Retired{receipt}=>receipt,_=>panic!("same original native choice must complete")};assert_eq!(receipt.reference.approved_at,100_041);let before=inventory(&f.path);pending.confirm(&store,true,400_002).unwrap();assert_eq!(inventory(&f.path),before);
}
#[test]fn uncertain_retirement_can_acknowledge_after_restart_without_a_recreated_choice_or_new_authority(){
    let(f,store,g,decision,mut cache,ticket,pending)=setup();{
        let mut state=pending.state.lock().unwrap();let State::Review(review)=std::mem::replace(&mut*state,State::Ended)else{panic!("original review required")};let choice=review.decide(100_041).unwrap();let mut approved=review.approve(choice);approved.authority.fault=Fault::AfterGrantPublish;*state=State::Approved(Box::new(approved));
    }
    assert!(matches!(pending.confirm(&store,true,100_042).unwrap(),RetirementConfirmed::Blocked{retained:true,..}));cache.sweep(400_000);assert!(cache.pending.contains_key(&ticket.token));drop(pending);drop(cache);let before=inventory(&f.path);
    match store.prepare_retirement(&root(&f),&g.id,&g.spec.owner.pet_id,&g.revision,&decision,"after-cache-loss".into(),400_001).unwrap(){Prepared::Retired(receipt)=>{assert_eq!(receipt.reference.approved_at,100_041);assert!(!receipt.execution_available&&!receipt.new_admission_created);},Prepared::Review(_)=>panic!("signed recorded retirement is only acknowledged")};assert_eq!(inventory(&f.path),before);
}
