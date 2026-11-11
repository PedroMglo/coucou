// Native epoch invalidation is independent of persisted authority or provider IO.
#[test]fn admission_review_cannot_be_consumed_after_any_callers_context_was_closed(){
    for label in ["settings","companions","pet-control","pet-panel"]{
        let f=Fixture::new();let g=f.goal("ollama");Authority::ensure(&root(&f)).unwrap();let mut cache=super::super::Reviews::default();
        let origin=super::super::Origin{window:label.into(),frame:crate::settings::review::Frame::fixture()};let ticket=cache.insert(&origin,review(&g,"original-window-review",100_002),nonce().unwrap(),100_002).unwrap();
        let input=ConfirmInput{disclosure_consent:None,memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token,approve:true};let p=cache.lookup(&origin,&input,100_003).unwrap();let before=hashes(&f.path);p.origin.frame.invalidate_for_test();
        assert!(p.confirm(&f.store(),&root(&f),true,|_,_|->model::Result<std::result::Result<(Evidence,()),Denial>>{panic!("expired human context must not observe provider or issue authority")},||100_004).is_err());
        assert_eq!(hashes(&f.path),before);assert!(cache.lookup(&cache.origin_for_test(label),&input,100_004).is_err());
        let origin=super::super::Origin{window:label.into(),frame:crate::settings::review::Frame::fixture()};
        let ticket=cache.insert(&origin,review(&g,"fresh-context",100_005),nonce().unwrap(),100_005).unwrap();assert_ne!(ticket.token,input.token);assert_eq!(hashes(&f.path),before);
    }
}
#[test]fn closing_during_admission_observation_refuses_the_unconsumed_choice_without_writes(){
    let(f,_g,_cache,_input,p)=fixture();let before=hashes(&f.path);
    let result=p.confirm(&f.store(),&root(&f),true,|goal,now|{p.origin.frame.invalidate_for_test();Ok(Ok((evidence(goal,now),())))},||100_004);
    assert!(matches!(result,Err(Error::Conflict)));assert_eq!(hashes(&f.path),before);assert!(matches!(*p.native.lock().unwrap(),Native::Ended));
}
#[test]fn reopened_admission_alias_retains_one_exact_consumed_choice_through_seventy_generations(){
    let(f,g,mut cache,input,p)=fixture();let lock=shared(&f);assert!(matches!(run(&p,&f,100_004).unwrap(),ConfirmResult::AuthorityBlocked{reason:Failure::Busy,..}));
    let original=spec(&p);let mut current=p.clone();let mut token=input.token.clone();
    for _ in 0..70{
        current.origin.frame.invalidate_for_test();let origin=super::super::Origin{window:"settings".into(),frame:current.origin.frame.next_for_test()};
        let ticket=cache.retained_approved(&origin,&g,super::super::ReviewPath::Admission,100_005).unwrap().unwrap();assert_ne!(ticket.token,token);assert_eq!(ticket.approval_scope,"confirmOriginalAdmission");assert_eq!(ticket.proposal.prepared_at,p.proposal.prepared_at);assert_eq!(ticket.proposal.expires_at,p.proposal.expires_at);
        let refs=ConfirmInput{disclosure_consent:None,memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:ticket.proposal.revision.clone(),token:ticket.token.clone(),approve:true};
        assert!(cache.lookup(&origin,&ConfirmInput{disclosure_consent:None,memory_consent:None,pet_id:refs.pet_id.clone(),goal_id:refs.goal_id.clone(),revision:refs.revision.clone(),token:token.clone(),approve:true},100_005).is_err());
        let alias=cache.lookup(&origin,&refs,100_005).unwrap();assert!(Arc::ptr_eq(&alias.native,&p.native));assert_eq!(cache.pending.len(),1);assert_eq!(model::fingerprint(&spec(&alias)).unwrap(),model::fingerprint(&original).unwrap());current=alias;token=ticket.token;
    }
    drop(lock);let record=queued(run(&current,&f,100_006).unwrap()).record;assert_eq!(record.admissions.len(),1);assert_eq!(model::fingerprint(&record.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());
    let before=hashes(&f.path);let ack=current.confirm(&f.store(),&root(&f),true,|_,_|->model::Result<std::result::Result<(Evidence,()),Denial>>{panic!("ACK cannot refresh authority")},||400_000).unwrap();assert!(matches!(ack,ConfirmResult::Queued{..}));assert_eq!(hashes(&f.path),before);
}
#[test]fn a_different_still_live_owner_with_the_same_label_cannot_claim_an_admission(){
    let(f,g,mut cache,_input,p)=fixture();let lock=shared(&f);assert!(matches!(run(&p,&f,100_004).unwrap(),ConfirmResult::AuthorityBlocked{..}));let before=hashes(&f.path);
    let other=super::super::Origin{window:"settings".into(),frame:crate::settings::review::Frame::fixture()};assert!(matches!(cache.retained_approved(&other,&g,super::super::ReviewPath::Admission,100_005),Err(Error::Conflict)));assert_eq!(hashes(&f.path),before);drop(lock);
}
