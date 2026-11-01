use super::*;
#[path="memory_admission_review_tests.rs"]mod admission_review_tests;
use crate::pet_runtime::memory_review::{Captured,DataConsent};

#[test]fn full_goal_records_bind_both_native_destinations_without_credentials_or_history_copies(){
 for provider in ["ollama","lmstudio"]{
  let f=Fixture::new();let main=f.main();let public=main.conversation_id(provider).unwrap();
  let mut spec=Fixture::goal(Context::Mochi{conversation_id:public.clone()}).spec;
  spec.owner=Owner{pet_id:if provider=="ollama"{"ai_ollama"}else{"ai_lmstudio"}.into(),provider:provider.into(),adapter:provider.into(),model_hint:Some("fixture-free".into())};
  let mut g=Record::new(spec,100_000,nonce().unwrap()).unwrap();f.write(vec![Fixture::record("selected",&g.spec.owner.pet_id,Some(&public))]);
  let(b,private)=f.prepare(&g,&main,Mode::Snapshot,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);
  let server=crate::local_chat::Server{id:provider,name:"Synthetic free server",url:"http://127.0.0.1:12345/selected-prefix".into(),key:Some("SYNTHETIC_PRIVATE_REVIEW_CREDENTIAL".into())};let now=crate::assistant_memory::now();
  let captured=Captured::at(&f.catalog(),&g,&server,"fixture-free",now).unwrap().unwrap();let preview=serde_json::to_value(captured.preview()).unwrap();
  assert_eq!(preview["sourceConversationId"],public);assert_eq!(preview["requiredConsent"],"boundedGoalContext");assert_eq!(preview["records"][0]["content"],"PRIVATE_GOAL_MEMORY_SENTINEL");
  assert_eq!(preview["destination"]["endpoint"],"http://127.0.0.1:12345/selected-prefix/v1/chat/completions");assert_eq!(preview["destination"]["model"],"fixture-free");assert!(!preview.to_string().contains("SYNTHETIC_PRIVATE_REVIEW_CREDENTIAL"));
  let header=f.contexts().join(&g.id).join(HEADER);let journal=f.contexts().join(&g.id).join(JOURNAL);let old_header=fs::read(&header).unwrap();let old_journal=fs::read(&journal).unwrap();
  let confirmed=captured.confirm(&g,&server,"fixture-free",Some(DataConsent::BoundedGoalContext),now+1).unwrap();let message=confirmed.message(&g,&server,"fixture-free",now+2).unwrap();
  assert_eq!(message["role"],"user");let text=message["content"].as_str().unwrap();assert!(text.contains("explicitSelectedGoalMemories")&&text.contains(&g.id)&&text.contains(&public)&&text.contains("PRIVATE_GOAL_MEMORY_SENTINEL"));assert!(!text.contains("SYNTHETIC_PRIVATE_REVIEW_CREDENTIAL"));
  assert!(private.history(provider).is_empty());assert_eq!(fs::read(&header).unwrap(),old_header);assert_eq!(fs::read(&journal).unwrap(),old_journal);assert!(!serde_json::to_string(&g).unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));
  assert!(g.admissions.is_empty());let d=crate::pet_runtime::admission_tests::descriptor(&g,now);assert_eq!(crate::pet_runtime::admission::Evidence::observe(&g,&d,"a".repeat(64),now,false,false).err(),Some(crate::pet_runtime::admission::Denial::Unsupported));
 }
}

#[test]fn only_goal_specific_consent_is_accepted_and_the_original_review_expiry_is_not_renewed(){
 for wrong in ["thisRequest","nativeSessionContext","true"]{assert!(serde_json::from_value::<DataConsent>(json!(wrong)).is_err());}
 assert_eq!(serde_json::from_value::<DataConsent>(json!("boundedGoalContext")).unwrap(),DataConsent::BoundedGoalContext);
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("selected","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,_private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);
 let server=crate::local_chat::Server{id:"ollama",name:"Synthetic free",url:"http://127.0.0.1:12345".into(),key:None};let now=crate::assistant_memory::now();
 let captured=Captured::at(&f.catalog(),&g,&server,"fixture-free",now).unwrap().unwrap();assert_eq!(captured.expires_at(),now+120_000);assert!(captured.preview().source_conversation_id.is_none());assert_eq!(captured.confirm(&g,&server,"fixture-free",None,now+1).err(),Some("goalMemoryConsentRequired"));
 let captured=Captured::at(&f.catalog(),&g,&server,"fixture-free",now).unwrap().unwrap();assert_eq!(captured.confirmation_current(&g,&server,"fixture-free",now+120_000).err(),Some("goalMemoryReviewExpired"));
 let confirmed=captured.confirm(&g,&server,"fixture-free",Some(DataConsent::BoundedGoalContext),now+1).unwrap();let message=confirmed.message(&g,&server,"fixture-free",now+120_001).unwrap();assert!(message["content"].as_str().unwrap().contains("\"sourceConversationId\":null"));
 assert!(g.admissions.is_empty());assert_eq!(g.state,State::Draft);
}

#[test]fn changed_destination_credential_model_and_original_intent_refuse_without_catalog_mutation(){
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("selected","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,_private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);
 let server=crate::local_chat::Server{id:"ollama",name:"Synthetic free",url:"http://127.0.0.1:12345/prefix".into(),key:Some("SYNTHETIC_KEY_ONE".into())};let now=crate::assistant_memory::now();let captured=Captured::at(&f.catalog(),&g,&server,"fixture-free",now).unwrap().unwrap();let bytes=fs::read(f.catalog()).unwrap();
 for (id,url,key)in [("lmstudio",server.url.as_str(),server.key.clone()),("ollama","http://127.0.0.1:12346/prefix",server.key.clone()),("ollama",server.url.as_str(),Some("SYNTHETIC_KEY_TWO".into())),("ollama",server.url.as_str(),None)]{
  let changed=crate::local_chat::Server{id,name:"Synthetic free",url:url.into(),key};assert!(captured.confirmation_current(&g,&changed,"fixture-free",now+1).is_err());
 }
 assert!(captured.confirmation_current(&g,&server,"another-model",now+1).is_err());let mut changed=g.clone();changed.spec.intent="Different original objective".into();assert!(captured.confirmation_current(&changed,&server,"fixture-free",now+1).is_err());
 assert!(captured.confirmation_current(&g,&server,"fixture-free",now+1).is_ok());assert_eq!(fs::read(f.catalog()).unwrap(),bytes);
 let remote=crate::local_chat::Server{id:"ollama",name:"Unapproved remote",url:"https://example.invalid".into(),key:None};assert!(Captured::at(&f.catalog(),&g,&remote,"fixture-free",now).is_err());
}

#[test]fn exact_references_revalidate_deletion_edit_expiry_and_unrelated_catalog_changes(){
 for event in ["delete","edit","expire"]{
  let f=Fixture::new();let main=f.main();let now=crate::assistant_memory::now();let mut row=Fixture::record("selected","ai_ollama",None);row["expiresAt"]=json!(now+60_000);f.write(vec![row.clone()]);let mut g=Fixture::goal(Context::Unbound);let(b,_private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);
  let server=crate::local_chat::Server{id:"ollama",name:"Synthetic free",url:"http://127.0.0.1:12345".into(),key:None};let captured=Captured::at(&f.catalog(),&g,&server,"fixture-free",now).unwrap().unwrap();assert_eq!(captured.expires_at(),now+60_000);
  let confirmed=captured.confirm(&g,&server,"fixture-free",Some(DataConsent::BoundedGoalContext),now+1).unwrap();f.write(vec![row.clone(),Fixture::record("unrelated","agent_copilot",None)]);assert!(confirmed.message(&g,&server,"fixture-free",now+2).is_ok());
  match event{"delete"=>f.write(vec![]),"edit"=>{let mut changed=row.clone();changed["revision"]=json!(2);changed["content"]=json!("NEW_UNREVIEWED_DATA");f.write(vec![changed]);},_=>{}}
  assert!(confirmed.message(&g,&server,"fixture-free",if event=="expire"{now+60_000}else{now+3}).is_err());f.write(vec![row]);assert!(confirmed.message(&g,&server,"fixture-free",now+4).is_err(),"invalidated data cannot revive from restored bytes");
 }
}

#[test]fn nil_selection_does_not_read_a_catalog_or_resolve_an_engine(){
 let f=Fixture::new();let main=f.main();let mut g=Fixture::goal(Context::Unbound);let(b,_private)=f.prepare(&g,&main,Mode::Dedicated,None).unwrap();Fixture::bind(&mut g,b);let missing=f.root.join("nonexistent").join("memory.json");
 let invalid=crate::local_chat::Server{id:"custom",name:"Not configured",url:"".into(),key:None};assert!(Captured::at(&missing,&g,&invalid,"@auto",crate::assistant_memory::now()).unwrap().is_none());assert!(!missing.exists());assert_eq!(g.schema_version,2);
}
