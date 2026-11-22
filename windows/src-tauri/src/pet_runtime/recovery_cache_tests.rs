use super::*;
use crate::pet_runtime::{recovery_tests::Environment,authority_tests::root,reconciliation_tests::inventory,model::State as GoalState,store::Fault};
#[test]fn recovery_inputs_accept_only_native_references_and_explicit_choice(){
    let extra=serde_json::json!({"petId":"ai_ollama","goalId":"g","revision":"r","token":"t","approve":true,"ownerAbsent":true,"closure":{"signed":true}});assert!(serde_json::from_value::<RecoveryConfirmInput>(extra).is_err());
    let extra=serde_json::json!({"petId":"ai_ollama","goalId":"g","revision":"r","requestId":"q","budget":{"maxModelCalls":999},"sourceGrantHash":"forged"});assert!(serde_json::from_value::<RecoveryPrepareInput>(extra).is_err());
}
fn origin(label:&str)->Origin{Origin{window:label.into(),frame:crate::settings::review::Frame::fixture()}}
fn input(env:&Environment,request:&str)->RecoveryPrepareInput{RecoveryPrepareInput{pet_id:env.goal.spec.owner.pet_id.clone(),goal_id:env.goal.id.clone(),revision:env.goal.revision.clone(),request_id:request.into()}}
fn choice(ticket:&RecoveryTicket)->RecoveryConfirmInput{RecoveryConfirmInput{pet_id:ticket.proposal.pet_id.clone(),goal_id:ticket.proposal.goal_id.clone(),revision:ticket.proposal.revision.clone(),token:ticket.token.clone(),approve:true}}
#[test]fn native_recovery_ticket_is_window_pet_goal_and_revision_scoped_and_decline_has_no_writes(){
    let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin("settings");let before=inventory(&env.fixture.path);let ticket=cache.insert(&window,"close".into(),env.review("close"),100_020).unwrap();let same=cache.find(&window,&input(&env,"close"),100_021).unwrap().unwrap();assert_eq!(same.token,ticket.token);let mut c=choice(&ticket);assert!(cache.lookup(&origin("pet-control"),&c).is_err());c.pet_id="agent_codex".into();assert!(cache.lookup(&window,&c).is_err());c=choice(&ticket);c.revision="0".repeat(32);assert!(cache.lookup(&window,&c).is_err());
    let pending=cache.lookup(&window,&choice(&ticket)).unwrap();assert!(matches!(pending.confirm(&env.store,false,100_022).unwrap(),RecoveryConfirmed::Declined));assert_eq!(inventory(&env.fixture.path),before);drop(pending);cache.sweep(100_023);assert!(cache.pending.is_empty());drop(env.review("after-decline"));
}
#[test]fn approval_survives_busy_and_uncertain_ack_and_same_context_returns_original_cleanup_ticket(){
    let mut env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin("settings");let ticket=cache.insert(&window,"close".into(),env.review("close"),100_020).unwrap();let pending=cache.lookup(&window,&choice(&ticket)).unwrap();let lock=env.store.test_lock(true).unwrap();let before=inventory(&env.fixture.path);let blocked=pending.confirm(&env.store,true,100_021).unwrap();assert!(matches!(blocked,RecoveryConfirmed::Blocked{reason:Error::Busy,retryable:true,..}));assert_eq!(inventory(&env.fixture.path),before);drop(lock);
    env.store.fault=Fault::AfterPublish;let uncertain=pending.confirm(&env.store,true,100_022).unwrap();let RecoveryConfirmed::Closed{receipt}=uncertain else{panic!("approved closure must publish")};assert_eq!(receipt.status,model::CommitStatus::Uncertain);let mut refreshed=input(&env,"after-ui-reload");refreshed.revision=receipt.record.revision;let cleanup=cache.find(&window,&refreshed,300_000).unwrap().unwrap();assert_eq!(cleanup.token,ticket.token);assert_eq!(cleanup.proposal.approval_scope,"ackRecordedClosure");assert_eq!(cleanup.proposal.revision,ticket.proposal.revision);let grant=env.grant();env.store.fault=Fault::None;
    let RecoveryConfirmed::Closed{receipt}=pending.confirm(&env.store,true,300_001).unwrap()else{panic!("same original ACK required")};assert_eq!(receipt.record.state,GoalState::RecoveryRequired);assert_eq!(env.grant(),grant);assert_eq!(env.grant()["recovery"]["reference"]["approvedAt"],100_021);
}
#[test]fn an_expired_unapproved_ticket_is_released_but_an_uncommitted_choice_can_be_explicitly_withdrawn(){
    let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin("settings");let ticket=cache.insert(&window,"expiry".into(),env.review("expiry"),100_020).unwrap();let before=inventory(&env.fixture.path);cache.sweep(130_021);assert!(cache.lookup(&window,&choice(&ticket)).is_err());assert_eq!(inventory(&env.fixture.path),before);
    let ticket=cache.insert(&window,"choice".into(),env.review("choice"),100_020).unwrap();let pending=cache.lookup(&window,&choice(&ticket)).unwrap();let lock=env.store.test_lock(true).unwrap();assert!(matches!(pending.confirm(&env.store,true,100_021).unwrap(),RecoveryConfirmed::Blocked{reason:Error::Busy,..}));drop(lock);let before=inventory(&env.fixture.path);assert!(matches!(pending.confirm(&env.store,false,100_022).unwrap(),RecoveryConfirmed::Declined));assert_eq!(inventory(&env.fixture.path),before);assert!(env.grant().get("recovery").is_none());assert!(root(&env.fixture).is_dir());
}
#[test]fn all_recovery_callers_refuse_queued_approval_after_close_without_cancelling_the_goal(){
    for label in [crate::HUB_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL]{
        let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin(label);let before=inventory(&env.fixture.path);
        let ticket=cache.insert(&window,"old-review".into(),env.review("old-review"),100_020).unwrap();let queued=cache.lookup(&window,&choice(&ticket)).unwrap();
        window.frame.invalidate_for_test();assert!(queued.confirm(&env.store,true,100_021).is_err());assert_eq!(inventory(&env.fixture.path),before);
        let reopened=Origin{window:label.into(),frame:window.frame.next_for_test()};assert!(cache.lookup(&reopened,&choice(&ticket)).is_err());drop(queued);cache.sweep(100_022);assert!(cache.pending.is_empty());
        let fresh=cache.insert(&reopened,"new-review".into(),env.review("new-review"),100_023).unwrap();assert_ne!(ticket.token,fresh.token);
        assert!(matches!(cache.lookup(&reopened,&choice(&fresh)).unwrap().confirm(&env.store,false,100_024).unwrap(),RecoveryConfirmed::Declined));assert_eq!(inventory(&env.fixture.path),before);
    }
}
#[test]fn matching_label_with_a_different_live_native_owner_cannot_claim_or_rebind_a_review(){
    let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let original=origin("settings");let impostor=origin("settings");let before=inventory(&env.fixture.path);
    let ticket=cache.insert(&original,"close".into(),env.review("close"),100_020).unwrap();assert!(cache.lookup(&impostor,&choice(&ticket)).is_err());assert!(cache.find(&impostor,&input(&env,"close"),100_021).is_err());assert_eq!(inventory(&env.fixture.path),before);
}
#[test]fn expired_origin_cannot_publish_a_review_prepared_while_waiting_for_storage(){
    let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin("settings");let prepared=env.review("prepared-before-close");let before=inventory(&env.fixture.path);window.frame.invalidate_for_test();assert!(cache.insert(&window,"prepared-before-close".into(),prepared,100_020).is_err());assert!(cache.pending.is_empty());assert_eq!(inventory(&env.fixture.path),before);drop(env.review("gate-was-released"));
}
#[test]fn reopening_rebinds_only_the_original_approved_closure_and_never_renews_its_authority(){
    let mut env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let window=origin("settings");let ticket=cache.insert(&window,"original".into(),env.review("original"),100_020).unwrap();let old=cache.lookup(&window,&choice(&ticket)).unwrap();
    let lock=env.store.test_lock(true).unwrap();assert!(matches!(old.confirm(&env.store,true,100_021).unwrap(),RecoveryConfirmed::Blocked{reason:Error::Busy,..}));drop(lock);let before=env.grant();window.frame.invalidate_for_test();let reopened=Origin{window:"settings".into(),frame:window.frame.next_for_test()};
    assert!(cache.lookup(&reopened,&choice(&ticket)).is_err());let cleanup=cache.find(&reopened,&input(&env,"new-ui-request"),300_000).unwrap().unwrap();assert_ne!(cleanup.token,ticket.token);assert_eq!(cleanup.proposal.approval_scope,"ackRecordedClosure");assert_eq!(cleanup.proposal.request_id,"original");assert_eq!(cleanup.proposal.prepared_at,ticket.proposal.prepared_at);assert_eq!(cleanup.proposal.expires_at,ticket.proposal.expires_at);assert_eq!(cache.pending.len(),1);
    let alias=cache.lookup(&reopened,&choice(&cleanup)).unwrap();assert!(alias.kind==Kind::CleanupOnly);assert!(Arc::ptr_eq(&alias.state,&old.state));assert_eq!(env.grant(),before);
    env.store.fault=Fault::AfterPublish;let RecoveryConfirmed::Closed{receipt}=alias.confirm(&env.store,true,300_001).unwrap()else{panic!("original closure required")};assert_eq!(receipt.status,model::CommitStatus::Uncertain);assert_eq!(env.grant()["recovery"]["reference"]["approvedAt"],100_021);
    let mut current=input(&env,"ack");current.revision=receipt.record.revision;assert_eq!(cache.find(&reopened,&current,300_002).unwrap().unwrap().token,cleanup.token);let grant=env.grant();env.store.fault=Fault::None;
    assert!(matches!(alias.confirm(&env.store,true,300_003).unwrap(),RecoveryConfirmed::Closed{..}));assert_eq!(env.grant(),grant);
}
#[test]fn repeated_window_generations_do_not_duplicate_the_approved_state_or_exhaust_the_cache(){
    let env=Environment::new("ollama","started",true);let mut cache=Reviews::default();let mut window=origin("settings");let ticket=cache.insert(&window,"choice".into(),env.review("choice"),100_020).unwrap();let original=cache.lookup(&window,&choice(&ticket)).unwrap();let lock=env.store.test_lock(true).unwrap();assert!(matches!(original.confirm(&env.store,true,100_021).unwrap(),RecoveryConfirmed::Blocked{reason:Error::Busy,..}));drop(lock);let before=inventory(&env.fixture.path);let mut current=ticket;
    for n in 0..70{window.frame.invalidate_for_test();window=Origin{window:"settings".into(),frame:window.frame.next_for_test()};assert!(cache.lookup(&window,&choice(&current)).is_err());current=cache.find(&window,&input(&env,"same-closure"),100_022+n).unwrap().unwrap();assert!(Arc::ptr_eq(&cache.lookup(&window,&choice(&current)).unwrap().state,&original.state));assert_eq!(cache.pending.len(),1);}
    assert!(matches!(cache.lookup(&window,&choice(&current)).unwrap().confirm(&env.store,false,100_100).unwrap(),RecoveryConfirmed::Declined));assert_eq!(inventory(&env.fixture.path),before);
}
