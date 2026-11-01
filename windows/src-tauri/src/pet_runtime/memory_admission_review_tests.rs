use super::*;
use crate::pet_runtime::{admission::{Evidence,Review,Denial,Budget},admission_review::{Reviews,ConfirmInput,ConfirmResult},admission_tests,store::Store};

fn observation(f:&Fixture,g:&Record,server:&crate::local_chat::Server,now:u64)->Evidence{
 let memory=Captured::at(&f.catalog(),g,server,"fixture-free",now).unwrap();
 Evidence::from_memory_capture(g,&admission_tests::descriptor(g,now),"a".repeat(64),now,false,false,memory).unwrap()
}
fn budget(now:u64)->Budget{let mut b=admission_tests::budget();b.deadline_at=now+60_000;b}
fn fixture()->(Fixture,Record,Chat,crate::local_chat::Server,u64){
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("selected","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);
 let server=crate::local_chat::Server{id:"ollama",name:"Synthetic free",url:"http://127.0.0.1:12345/exact-base".into(),key:None};(f,g,private,server,crate::assistant_memory::now())
}

#[test]fn native_data_evidence_prepares_a_full_proposal_but_bare_observation_and_legacy_approval_stay_blocked(){
 let(f,g,_private,server,now)=fixture();
 assert_eq!(Evidence::observe(&g,&admission_tests::descriptor(&g,now),"a".repeat(64),now,false,false).err(),Some(Denial::Unsupported));
 let review=Review::prepare(&g,observation(&f,&g,&server,now),budget(now),"review-data".into(),nonce().unwrap(),now).unwrap();
 let proposal=serde_json::to_value(review.proposal()).unwrap();assert_eq!(proposal["memoryData"]["requiredConsent"],"boundedGoalContext");assert_eq!(proposal["memoryData"]["records"][0]["content"],"PRIVATE_GOAL_MEMORY_SENTINEL");assert_eq!(proposal["memoryData"]["destination"]["endpoint"],"http://127.0.0.1:12345/exact-base/v1/chat/completions");assert_eq!(proposal["memoryData"]["sourceConversationId"],Value::Null);
 assert_eq!(review.approve(&g,observation(&f,&g,&server,now+1),now+1).err(),Some(Denial::DataConsentRequired));assert!(g.admissions.is_empty());assert!(!serde_json::to_string(&g).unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));
}

#[test]fn old_confirmation_cannot_consume_the_data_review_create_authority_read_the_store_or_dispatch(){
 let(f,g,_private,server,now)=fixture();let review=Review::prepare(&g,observation(&f,&g,&server,now),budget(now),"review-data".into(),nonce().unwrap(),now).unwrap();let mut cache=Reviews::default();let ticket=cache.insert("settings",review,nonce().unwrap(),now).unwrap();
 let input=ConfirmInput{memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token.clone(),approve:true};let pending=cache.lookup("settings",&input,now+1).unwrap();let store=Store::ensure(&f.root.join("empty-goal-store")).unwrap();let authority=f.root.join("must-not-create-authority");
 for at in [now+1,now+2]{let result=pending.confirm::<()>(&store,&authority,true,|_,_|panic!("incomplete data admission must block before observation or store read"),||at).unwrap();assert!(matches!(result,ConfirmResult::Blocked{reason:Denial::DataConsentRequired,retryable:false}));assert!(!authority.exists());}
 assert!(std::sync::Arc::ptr_eq(&pending,&cache.lookup("settings",&input,now+3).unwrap()));
 assert!(matches!(pending.confirm::<()>(&store,&authority,false,|_,_|panic!("decline does not observe"),||now+4).unwrap(),ConfirmResult::Declined));assert!(!authority.exists());assert!(g.admissions.is_empty());
}

#[test]fn retained_native_data_review_rechecks_private_credential_and_original_records_before_confirmation(){
 let(f,g,_private,server,now)=fixture();let review=Review::prepare(&g,observation(&f,&g,&server,now),budget(now),"review-data".into(),nonce().unwrap(),now).unwrap();
 let changed=crate::local_chat::Server{id:"ollama",name:"Synthetic free",url:server.url.clone(),key:Some("SYNTHETIC_DIFFERENT_KEY".into())};
 assert_eq!(review.confirmation_current(&g,&observation(&f,&g,&changed,now+1),now+1).err(),Some(Denial::RevisionChanged));assert!(review.confirmation_current(&g,&observation(&f,&g,&server,now+2),now+2).is_ok());
 let fresh=observation(&f,&g,&server,now+3);f.write(vec![]);assert_eq!(review.confirmation_current(&g,&fresh,now+4).err(),Some(Denial::ContextChanged));f.write(vec![Fixture::record("selected","ai_ollama",None)]);let restored=observation(&f,&g,&server,now+5);assert_eq!(review.confirmation_current(&g,&restored,now+5).err(),Some(Denial::ContextChanged));assert!(g.admissions.is_empty());
}
