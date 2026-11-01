use super::*;
use crate::chat::{ChatContext,ContextFile};
#[test]fn reviewed_api_endpoints_use_the_same_native_route_choice_as_dispatch_without_claiming_provider_containment(){
 for(provider,expected)in [("openai",vec!["https://api.example.invalid/prefix/v1/chat/completions","https://api.example.invalid/prefix/v1/responses"]),("google",vec!["https://api.example.invalid/prefix/v1/chat/completions"]) ]{
  let d=Destination::Api{provider:provider.into(),endpoint:"https://api.example.invalid/prefix/v1".into(),model:"requested-model".into()};let c=dispatch_coverage(&d);assert_eq!(c.endpoints,expected);assert_eq!(c.kind,"knownApiEndpointSet");assert!(!c.provider_internals_observed&&!c.tools_containment_observed);
  for calendar in [false,true]{let actual=crate::net::join(&reqwest::Url::parse("https://api.example.invalid/prefix/v1").unwrap(),crate::openai_compat::completion_tail(provider,calendar));assert!(c.endpoints.contains(&actual));}
 }
 for d in [Destination::Api{provider:"anthropic".into(),endpoint:"https://api.example.invalid/v1/messages".into(),model:"requested-model".into()},destination("ollama"),destination("lmstudio")]{let c=dispatch_coverage(&d);assert_eq!(c.endpoints.len(),1);match d{Destination::Api{endpoint,..}|Destination::Server{endpoint,..}=>assert_eq!(c.endpoints,vec![endpoint]),_=>unreachable!()}}
 let c=dispatch_coverage(&destination("codex"));assert!(c.endpoints.is_empty());assert_eq!(c.kind,"nativeSessionProtocol");assert!(!c.provider_internals_observed);
}
#[test]fn swapped_context_reference_path_order_sequence_or_legacy_document_receipt_cannot_consume_the_original_data_review(){
 let f=Files::new();let entry=f.create("ai_ollama",None,"selected fixture data",None);
 let first=ContextFile{id:"attachment-a".into(),name:"same.txt".into(),path:"/private/fixture/a.txt".into(),sequence:4};let second=ContextFile{id:"attachment-b".into(),name:"same.txt".into(),path:"/private/fixture/b.txt".into(),sequence:7};
 let context=ChatContext::Files{files:vec![first.clone(),second.clone()]};let hash=request_context_hash(Some(&context),Some(7)).unwrap();let mut reviews=Reviews::default();let p=prepared(&f,"ai_ollama","ollama",&entry.id).with_request_context(hash.clone()).unwrap();let ticket=reviews.insert(crate::HUB_LABEL,p,101).unwrap();assert_eq!(ticket.proposal.request_context_hash.as_deref(),Some(hash.as_str()));
 let mut changed=first.clone();changed.path="/private/fixture/other.txt".into();let mut seq=first.clone();seq.sequence+=1;let mut id=first.clone();id.id="attachment-changed".into();
 for (context,sequence)in [(None,Some(7)),(Some(ChatContext::Files{files:vec![changed,second.clone()]}),Some(7)),(Some(ChatContext::Files{files:vec![seq,second.clone()]}),Some(7)),(Some(ChatContext::Files{files:vec![id,second.clone()]}),Some(7)),(Some(ChatContext::Files{files:vec![second.clone(),first.clone()]}),Some(7)),(Some(context.clone()),None)]{
  let other=request_context_hash(context.as_ref(),sequence).unwrap();assert_ne!(other,hash);assert!(reviews.take_bound(&other,crate::HUB_LABEL,&ticket.token,"ai_ollama","original-cid","Original public question",&destination("ollama"),Some("PRIVATE_CREDENTIAL_SNAPSHOT"),Consent::ThisRequest,102).is_err());
 }
 assert!(reviews.take_bound(&hash,crate::HUB_LABEL,&ticket.token,"ai_ollama","original-cid","Original public question",&destination("ollama"),Some("PRIVATE_CREDENTIAL_SNAPSHOT"),Consent::ThisRequest,102).unwrap().is_some());assert!(reviews.take_bound(&hash,crate::HUB_LABEL,&ticket.token,"ai_ollama","original-cid","Original public question",&destination("ollama"),Some("PRIVATE_CREDENTIAL_SNAPSHOT"),Consent::ThisRequest,102).is_err());
}
#[test]fn a_production_bound_take_refuses_unbound_review_and_context_hash_has_a_byte_limit_without_reading_files(){
 let f=Files::new();let entry=f.create("ai_ollama",None,"fixture",None);let mut reviews=Reviews::default();let t=reviews.insert(crate::HUB_LABEL,prepared(&f,"ai_ollama","ollama",&entry.id),101).unwrap();let hash=request_context_hash(None,None).unwrap();assert!(reviews.take_bound(&hash,crate::HUB_LABEL,&t.token,"ai_ollama","original-cid","Original public question",&destination("ollama"),Some("PRIVATE_CREDENTIAL_SNAPSHOT"),Consent::ThisRequest,102).is_err());
 let huge=ChatContext::File{name:"fixture".into(),path:"x".repeat(256*1024)};assert_eq!(request_context_hash(Some(&huge),None),Err("memoryContextLimit"));
}
#[test]fn window_context_frontend_and_legacy_names_resolve_the_same_typed_context_but_changed_targets_have_different_hashes(){
 let current:ChatContext=serde_json::from_value(serde_json::json!({"kind":"window","appName":"Fixture","title":"Original","url":null})).unwrap();let legacy:ChatContext=serde_json::from_value(serde_json::json!({"kind":"window","app_name":"Fixture","title":"Original","url":null})).unwrap();assert_eq!(request_context_hash(Some(&current),None).unwrap(),request_context_hash(Some(&legacy),None).unwrap());let changed=ChatContext::Window{app_name:"Fixture".into(),title:"Changed".into(),url:None};assert_ne!(request_context_hash(Some(&current),None).unwrap(),request_context_hash(Some(&changed),None).unwrap());
}
