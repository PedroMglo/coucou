use super::*;
use crate::pet_runtime::{admission::{Evidence,Review,Denial},admission_tests,admission_review::{Reviews,ConfirmInput,ConfirmResult},authority::Authority,lease_controller::{Controller,Validity},store::Store};
use crate::pet_runtime::lease_controller::{memory_test_http_fixture as fixture_with_hook,memory_test_stream as stream,memory_test_metadata as local_metadata,memory_test_run as run};
use crate::pet_memory::context_model::{GoalConsent,GoalGrant,GOAL_INPUT_LIMIT};

fn metadata(provider:&str)->Value{if provider=="lmstudio"{json!({"models":[{"type":"llm","key":"fixture-free","publisher":"private-fixture","format":"gguf","size_bytes":1,"loaded_instances":[]}]})}else{local_metadata()}}
fn evidence(f:&Fixture,g:&Record,server:&crate::local_chat::Server,now:u64)->Evidence{
 let mut d=admission_tests::descriptor(g,now);
 if server.id=="lmstudio"{d.binding.engine_binding=Some(crate::local_chat::lmstudio::Catalog::parse(server,&metadata("lmstudio")).unwrap().binding("fixture-free").unwrap());}
 let capture=crate::pet_runtime::memory_review::Captured::at(&f.catalog(),g,server,"fixture-free",now).unwrap();
 Evidence::from_memory_capture(g,&d,"a".repeat(64),now,false,false,capture).unwrap()
}
fn setup(server:&crate::local_chat::Server)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){setup_with_history(server,false)}
fn setup_with_history(server:&crate::local_chat::Server,large_history:bool)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){
 let f=Fixture::new();let main=f.main();let mut spec=Fixture::goal(Context::Unbound).spec;
 spec.owner=Owner{pet_id:if server.id=="ollama"{"ai_ollama"}else{"ai_lmstudio"}.into(),provider:server.id.into(),adapter:server.id.into(),model_hint:Some("fixture-free".into())};
 if large_history{
  let turn=main.begin_cli(server.id,Arc::new(AtomicBool::new(false))).unwrap();let lease=main.lease(&turn);
  let user="u".repeat(GOAL_INPUT_LIMIT/2-64);let answer="a".repeat(GOAL_INPUT_LIMIT/2-64);
  main.commit_reply(&turn,json!({"role":"user","content":user.clone()}),json!({"role":"assistant","content":answer.clone()}),&user,&answer).unwrap();drop(lease);
  let cid=main.conversation_id(server.id).unwrap();let snapshot=main.snapshot_public_context(server.id,&cid).unwrap();assert_eq!(snapshot.messages.len(),2);let bytes=serde_json::to_vec(&snapshot.messages).unwrap().len();assert!(bytes>GOAL_INPUT_LIMIT-1024&&bytes<=GOAL_INPUT_LIMIT,"snapshot bytes: {bytes}");
  spec.context=Context::Mochi{conversation_id:cid};
 }
 f.write(vec![Fixture::record("selected",&spec.owner.pet_id,None)]);
 let store=Store::ensure(&f.root.join("goals")).unwrap();let now=crate::assistant_memory::now()-100;let g=store.create(spec,now).unwrap().record;
 let(binding,private)=f.prepare(&g,&main,if large_history{Mode::Snapshot}else{Mode::Dedicated},Some(&[Fixture::selected("selected")])).unwrap();drop(private);
 let g=store.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"prepare".into(),Action::BindContext{binding},now+1).unwrap().record;
 let mut budget=admission_tests::budget();budget.deadline_at=now+60_000;budget.payment=crate::pet_runtime::admission::Payment::LocalOnly;
 let review=Review::prepare(&g,evidence(&f,&g,server,now+2),budget,"queue-memory".into(),nonce().unwrap(),now+2).unwrap();let mut cache=Reviews::default();let ticket=cache.insert("settings",review,nonce().unwrap(),now+2).unwrap();
 let input=ConfirmInput{memory_consent:Some(GoalConsent::BoundedGoalContext),pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token,approve:true};let pending=cache.lookup("settings",&input,now+3).unwrap();let path=f.root.join("authority");
 // Missing consent cannot consume the original review or create authority.
 assert!(matches!(pending.confirm::<()>(&store,&path,true,|_,_|panic!("scope must be checked first"),||now+3).unwrap(),ConfirmResult::Blocked{reason:Denial::DataConsentRequired,retryable:false}));assert!(!path.exists());
 let result=pending.confirm_with_data(&store,&path,true,input.memory_consent,|g,n|Ok(Ok((evidence(&f,g,server,n),()))),||now+4).unwrap();let queued=match result{ConfirmResult::Queued{receipt}=>receipt.record,_=>panic!("typed data consent did not queue")};
 assert_eq!(queued.schema_version,12);let before=std::fs::read(path.join(format!("{}.json",queued.admissions[0].spec.decision_id))).unwrap();
 let ack=pending.confirm::<()>(&store,&path,true,|_,_|panic!("published ACK cannot renew or observe"),||now+200_000).unwrap();assert!(matches!(ack,ConfirmResult::Queued{..}));assert_eq!(std::fs::read(path.join(format!("{}.json",queued.admissions[0].spec.decision_id))).unwrap(),before);
 let a=Authority::existing(&path).unwrap();let controller=Controller::new().unwrap();
 let p=controller.prepare_memory_fixture(&queued,&path,&f.contexts(),&f.catalog(),&main,&evidence(&f,&queued,server,now+5),"claim-memory".into(),now+5,Arc::new(|_,_,_|Validity::Current)).unwrap();
 let(receipt,owned)=p.claim(&store,&a,&evidence(&f,&queued,server,now+6),now+6).unwrap();(f,store,a,receipt.record,owned.unwrap(),main)
}
fn config(provider:&str)->crate::pet_config::Configuration{let mut c=crate::pet_config::defaults();if provider=="lmstudio"{c.pets.get_mut("ai_lmstudio").unwrap().provider="lmstudio".into();}c}
fn ledger(f:&Fixture,g:&Record)->Value{serde_json::from_slice::<Value>(&fs::read(f.root.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap()).unwrap()["body"]["budgetUsage"].clone()}

#[test]fn signed_data_scope_and_actual_http_body_match_on_both_local_engines_without_persisting_notes(){
 for provider in ["ollama","lmstudio"]{
  let(mut server,http)=fixture_with_hook(vec![("application/json",metadata(provider).to_string()),("text/event-stream",stream("Resposta baseada no contexto permitido",true))],|_|{});server.id=provider;
  let(f,_store,_authority,g,owner,_main)=setup(&server);let grant=g.admissions[0].spec.memory_data.as_ref().unwrap();assert_eq!(grant.consent,GoalConsent::BoundedGoalContext);assert!(grant.valid(provider,"fixture-free"));assert_eq!(grant.max_input_bytes,GOAL_INPUT_LIMIT);
  assert_eq!(run(owner.respond_once(None,&g,&server,"one-memory-call",&config(provider))).unwrap(),"Resposta baseada no contexto permitido");let requests=http.join().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[1].0,"/v1/chat/completions");let body=&requests[1].1;let messages=body["messages"].as_array().unwrap();let memory=&messages[messages.len()-2];assert_eq!(memory["role"],"user");assert!(memory["content"].as_str().unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));assert!(!messages.last().unwrap()["content"].as_str().unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));assert!(body.get("tools").is_none());
  let usage=ledger(&f,&g);let attempts=usage["attempts"].as_array().unwrap();assert_eq!(attempts.len(),1);let lease=&g.admissions[0].lease.as_ref().unwrap().id;assert_eq!(attempts[0]["inputHash"],crate::pet_runtime::model::fingerprint(&(lease,"fixture-free",body)).unwrap());assert_eq!(attempts[0]["phase"],"settled");
  for p in f.rglob_json(){assert!(!fs::read_to_string(p).unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));}
  let prior=ledger(&f,&g);server.url="http://127.0.0.1:1".into();assert_eq!(run(owner.respond_once(None,&g,&server,"one-memory-call",&config(provider))).unwrap(),"Resposta baseada no contexto permitido");assert_eq!(ledger(&f,&g),prior);assert_eq!(owner.workers(),0);
 }
}

#[test]fn changed_data_destination_is_refused_before_metadata_inference_or_budget(){
 let(server,http)=fixture_with_hook(vec![],|_|{});let(f,_store,_authority,g,owner,_main)=setup(&server);let mut changed=crate::local_chat::Server{id:"ollama",name:"Changed",url:"http://127.0.0.1:1".into(),key:None};
 assert!(run(owner.respond_once(None,&g,&changed,"changed-data-target",&config("ollama"))).unwrap_err().contains("destino"));assert!(http.join().unwrap().is_empty());assert!(ledger(&f,&g).is_null());changed.key=Some("synthetic-unreviewed".into());assert!(run(owner.respond_once(None,&g,&changed,"changed-credential",&config("ollama"))).is_err());assert!(ledger(&f,&g).is_null());assert_eq!(owner.workers(),0);
}

#[test]fn revocation_during_metadata_prevents_model_dispatch_and_new_budget(){
 let path=Arc::new(std::sync::Mutex::new(None::<PathBuf>));let target=path.clone();let(server,http)=fixture_with_hook(vec![("application/json",local_metadata().to_string())],move|route|{assert_eq!(route,"/api/show");fs::write(target.lock().unwrap().as_ref().unwrap(),b"{\"version\":1,\"generation\":2,\"items\":[]}").unwrap();});
 let(f,_store,_authority,g,owner,_main)=setup(&server);*path.lock().unwrap()=Some(f.catalog());assert!(run(owner.respond_once(None,&g,&server,"revoked-before-model",&config("ollama"))).is_err());let requests=http.join().unwrap();assert_eq!(requests.len(),1);assert_eq!(requests[0].0,"/api/show");assert!(ledger(&f,&g).is_null());assert_eq!(owner.workers(),0);
}

#[test]fn revocation_after_dispatch_preserves_the_bound_attempt_and_refuses_late_success(){
 let path=Arc::new(std::sync::Mutex::new(None::<PathBuf>));let target=path.clone();let(server,http)=fixture_with_hook(vec![("application/json",local_metadata().to_string()),("text/event-stream",stream("Late answer must not commit",true))],move|route|{if route=="/v1/chat/completions"{fs::write(target.lock().unwrap().as_ref().unwrap(),b"{\"version\":1,\"generation\":2,\"items\":[]}").unwrap();}});
 let(f,_store,_authority,g,owner,_main)=setup(&server);*path.lock().unwrap()=Some(f.catalog());assert!(run(owner.respond_once(None,&g,&server,"revoked-during-model",&config("ollama"))).is_err());let requests=http.join().unwrap();assert_eq!(requests.len(),2);let usage=ledger(&f,&g);assert_eq!(usage["attempts"].as_array().unwrap().len(),1);assert_eq!(usage["attempts"][0]["inputHash"],crate::pet_runtime::model::fingerprint(&(&g.admissions[0].lease.as_ref().unwrap().id,"fixture-free",&requests[1].1)).unwrap());assert!(!fs::read_to_string(f.contexts().join(&g.id).join(JOURNAL)).unwrap().contains("Late answer must not commit"));assert_eq!(owner.workers(),0);
 let prior=ledger(&f,&g);assert!(run(owner.respond_once(None,&g,&server,"revoked-during-model",&config("ollama"))).is_err());assert_eq!(ledger(&f,&g),prior);
}

#[test]fn data_grant_format_and_conditional_record12_cannot_be_downlabelled_or_widened(){
 let(server,http)=fixture_with_hook(vec![],|_|{});let(_f,_store,_authority,g,_owner,_main)=setup(&server);assert!(http.join().unwrap().is_empty());let bytes=serde_json::to_vec(&g).unwrap();assert!(!String::from_utf8_lossy(&bytes).contains("PRIVATE_GOAL_MEMORY_SENTINEL"));
 for version in 1..12{let mut raw:Value=serde_json::from_slice(&bytes).unwrap();raw["schemaVersion"]=json!(version);assert!(serde_json::from_value::<Record>(raw).unwrap().validate().is_err());}
 for change in ["limit","endpoint","model","selection"]{let mut altered=g.clone();match change{"limit"=>altered.admissions[0].spec.memory_data.as_mut().unwrap().max_input_bytes+=1,"endpoint"=>if let crate::pet_memory::context_model::Destination::Server{endpoint,..}=&mut altered.admissions[0].spec.memory_data.as_mut().unwrap().destination{*endpoint="https://example.invalid/v1/chat/completions".into()},"model"=>if let crate::pet_memory::context_model::Destination::Server{model,..}=&mut altered.admissions[0].spec.memory_data.as_mut().unwrap().destination{*model="other-model".into()},_=>altered.admissions[0].spec.memory_data=None};assert!(altered.validate().is_err());}
 for scope in ["thisRequest","nativeSessionContext"]{let mut raw=serde_json::to_value(g.admissions[0].spec.memory_data.as_ref().unwrap()).unwrap();raw["consent"]=json!(scope);assert!(serde_json::from_value::<GoalGrant>(raw).is_err());}
}

impl Fixture{
 fn rglob_json(&self)->Vec<PathBuf>{fn visit(at:&std::path::Path,rows:&mut Vec<PathBuf>){for entry in fs::read_dir(at).unwrap(){let p=entry.unwrap().path();if p.is_dir(){visit(&p,rows)}else if p.extension().is_some_and(|e|e=="json")&&p.file_name().is_none_or(|n|n!="memories.json"){rows.push(p);}}}let mut rows=vec![];visit(&self.root,&mut rows);rows}
}

#[test]fn full_body_limit_includes_the_existing_history_before_hash_or_model_budget(){
 let(server,http)=fixture_with_hook(vec![("application/json",local_metadata().to_string())],|_|{});let(f,_store,_authority,g,owner,_main)=setup_with_history(&server,true);
 let error=run(owner.respond_once(None,&g,&server,"oversized-actual-body",&config("ollama"))).unwrap_err();assert!(error.contains("limite"),"{error}");let requests=http.join().unwrap();assert_eq!(requests.len(),1);assert_eq!(requests[0].0,"/api/show");assert!(ledger(&f,&g).is_null());assert_eq!(owner.workers(),0);
}
