use super::*;
use crate::pet_runtime::{admission::{Evidence,Review,Denial},admission_tests,admission_review::{Reviews,ConfirmInput,ConfirmResult},authority::Authority,lease_controller::{Controller,Validity},store::Store};
use crate::pet_runtime::lease_controller::{memory_test_http_fixture as fixture_with_hook,memory_test_stream as stream,memory_test_metadata as local_metadata,memory_test_run as run};
use crate::pet_memory::context_model::{GoalConsent,GoalGrant,GOAL_INPUT_LIMIT};

fn metadata(provider:&str)->Value{if provider=="lmstudio"{json!({"models":[{"type":"llm","key":"fixture-free","publisher":"private-fixture","format":"gguf","size_bytes":1,"loaded_instances":[]}]})}else{local_metadata()}}
fn evidence(f:&Fixture,g:&Record,server:&crate::local_chat::Server,now:u64)->Evidence{
 if f.root.join("pets.json").exists(){
  let config=crate::pet_config::read_existing_at(&f.root.join("pets.json")).unwrap();let settings=explicit_settings(&g.spec.owner.pet_id);
  let projection=crate::pet_capabilities::fixture_inference_projection(settings,config.clone(),&g.spec.owner.pet_id,&metadata(server.id),now,now);
  let d=projection.snapshot.pets.iter().find(|d|d.pet_id==g.spec.owner.pet_id).unwrap();let policy=crate::pet_runtime::native_validity::policy_revision(&config,&projection.settings,g).unwrap();
  return Evidence::observe_native_at(g,d,policy,now,false,false,&projection.settings,&config,&f.catalog()).unwrap();
 }
 let mut d=admission_tests::descriptor(g,now);d.binding.adapter=Some(server.id);
 if server.id=="lmstudio"{d.binding.engine_binding=Some(crate::local_chat::lmstudio::Catalog::parse(server,&metadata("lmstudio")).unwrap().binding("fixture-free").unwrap());}
 let capture=crate::pet_runtime::memory_review::Captured::at(&f.catalog(),g,server,"fixture-free",now).unwrap();
 let policy=if f.root.join("pets.json").exists(){let config=crate::pet_config::read_existing_at(&f.root.join("pets.json")).unwrap();crate::pet_runtime::native_validity::policy_revision(&config,&crate::settings::Settings::default(),g).unwrap()}else{"a".repeat(64)};
 Evidence::from_memory_capture(g,&d,policy,now,false,false,capture).unwrap()
}
fn setup(server:&crate::local_chat::Server)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){setup_with_history(server,false)}
fn setup_with_history(server:&crate::local_chat::Server,large_history:bool)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){
 setup_configuration(server,large_history,None)
}
fn setup_configuration(server:&crate::local_chat::Server,large_history:bool,binding:Option<(&str,crate::pet_config::Configuration)>)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){setup_mode(server,large_history,binding,false)}
fn setup_mode(server:&crate::local_chat::Server,large_history:bool,binding:Option<(&str,crate::pet_config::Configuration)>,planning:bool)->(Fixture,Store,Authority,Record,crate::pet_runtime::lease_controller::Owned,Chat){
 let f=Fixture::new();let mut main=f.main();let mut spec=Fixture::goal(Context::Unbound).spec;
 let planning_config=binding.as_ref().map(|(_,c)|c.clone());
 let provider=binding.as_ref().map(|(provider,_)|*provider).unwrap_or(server.id);
 spec.owner=Owner{pet_id:crate::pet_config::pet_id(provider).unwrap().into(),provider:provider.into(),adapter:server.id.into(),model_hint:Some("fixture-free".into())};
 if let Some((_,config))=binding{
  let profile=f.root.join("pets.json");fs::write(&profile,serde_json::to_vec(&config).unwrap()).unwrap();fs::set_permissions(&profile,fs::Permissions::from_mode(0o600)).unwrap();
  let settings=explicit_settings(&spec.owner.pet_id);
  main=main.with_runtime_resources(Arc::new(crate::pet_resources::Coordinator::with_fixture(Arc::new(std::sync::Mutex::new(settings)),profile)));
 }
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
 if planning{let mut denied=planning_config.clone().unwrap();denied.pets.get_mut(&g.spec.owner.pet_id).unwrap().permissions.get_or_insert_default().music=false;let other=if g.spec.owner.pet_id=="ai_ollama"{"ai_lmstudio"}else{"ai_ollama"};denied.pets.get_mut(other).unwrap().permissions.get_or_insert_default().music=true;
  assert!(matches!(Review::prepare_planning(&g,evidence(&f,&g,server,now+2),budget.clone(),crate::pet_runtime::planning::Limits{max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit:2},&denied,"denied-other-pet-grant".into(),nonce().unwrap(),now+2),Err(Denial::Unsupported)),"another pet's permission cannot authorize this owner");
 }
 let observed=evidence(&f,&g,server,now+2);let request="queue-memory".into();let decision=nonce().unwrap();
 let review=if planning{Review::prepare_planning(&g,observed,budget,crate::pet_runtime::planning::Limits{max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit:2},planning_config.as_ref().unwrap(),request,decision,now+2)}else{Review::prepare(&g,observed,budget,request,decision,now+2)}.unwrap();let mut cache=Reviews::default();let ticket=cache.insert(&cache.origin_for_test("settings"),review,nonce().unwrap(),now+2).unwrap();
 let input=ConfirmInput{disclosure_consent:None,memory_consent:Some(GoalConsent::BoundedGoalContext),pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token,approve:true};let pending=cache.lookup(&cache.origin_for_test("settings"),&input,now+3).unwrap();let path=f.root.join("authority");
 // Missing consent cannot consume the original review or create authority.
 assert!(matches!(pending.confirm::<()>(&store,&path,true,|_,_|panic!("scope must be checked first"),||now+3).unwrap(),ConfirmResult::Blocked{reason:Denial::DataConsentRequired,retryable:false}));assert!(!path.exists());
 let result=pending.confirm_with_data(&store,&path,true,input.memory_consent,|g,n|Ok(Ok((evidence(&f,g,server,n),()))),||now+4).unwrap();let queued=match result{ConfirmResult::Queued{receipt}=>receipt.record,_=>panic!("typed data consent did not queue")};
 assert_eq!(queued.schema_version,12);let before=std::fs::read(path.join(format!("{}.json",queued.admissions[0].spec.decision_id))).unwrap();
 let ack=pending.confirm::<()>(&store,&path,true,|_,_|panic!("published ACK cannot renew or observe"),||now+200_000).unwrap();assert!(matches!(ack,ConfirmResult::Queued{..}));assert_eq!(std::fs::read(path.join(format!("{}.json",queued.admissions[0].spec.decision_id))).unwrap(),before);
 let a=Authority::existing(&path).unwrap();let controller=Controller::new().unwrap();
 // Fixture fence uses the actual stored lease, descriptor and policy. Final
 // commit validates the proposed record without rereading it under its lock.
 let goals=f.root.join("goals");let auth=path.clone();let profile=f.root.join("pets.json");let pet=queued.spec.owner.pet_id.clone();let id=queued.id.clone();let engine=server.id;
 let fence=Arc::new(move|lease:&crate::pet_runtime::leases::Lease,_:&crate::pet_resources::Hold,mode:crate::pet_runtime::lease_controller::Validation<'_>|{
  let current=if let crate::pet_runtime::lease_controller::Validation::Commit(g)=mode{g.clone()}else{let Some(store)=Store::existing_verified(&goals,&auth).ok().flatten()else{return Validity::Unknown;};match store.read(&id,&pet){Ok(g)=>g,Err(_)=>return Validity::Unknown}};
  if profile.exists(){let Ok(config)=crate::pet_config::read_existing_at(&profile)else{return Validity::Unknown;};let settings=explicit_settings(&pet);let now=crate::assistant_memory::now();let projection=crate::pet_capabilities::fixture_inference_projection(settings,config.clone(),&pet,&metadata(engine),now,now);let d=projection.snapshot.pets.iter().find(|d|d.pet_id==pet).unwrap();let Ok(policy)=crate::pet_runtime::native_validity::policy_revision(&config,&projection.settings,&current)else{return Validity::Changed;};crate::pet_runtime::native_validity::policy(&current,lease,d,&policy,now)}else if current.state==crate::pet_runtime::model::State::Leased&&current.admissions.last().and_then(|a|a.lease.as_ref())==Some(lease){Validity::Current}else{Validity::Changed}
 });
 let p=controller.prepare_memory_fixture(&queued,&path,&f.contexts(),&f.catalog(),&main,&evidence(&f,&queued,server,now+5),"claim-memory".into(),now+5,fence).unwrap();
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

// R6: real registry/resolver/admission/control path with synthetic metadata,
// private profile and explicit fixture decisions; no human/native UI claim.
fn explicit_config(provider:&str,server:&crate::local_chat::Server)->crate::pet_config::Configuration{
 use crate::pet_inference::{Backend,Choice,CredentialReference};
 let mut c=crate::pet_config::defaults();c.version=2;let p=c.pets.get_mut(crate::pet_config::pet_id(provider).unwrap()).unwrap();
 p.inference=Some(Choice{schema_version:1,backend:if server.id=="lmstudio"{Backend::Lmstudio}else{Backend::Ollama},endpoint:server.url.clone(),model:"fixture-free".into(),credential:CredentialReference::None});p.inference_revision=Some("e".repeat(32));crate::pet_config::validate(&c).unwrap();c
}
#[test]fn pet_inference_goal_cross_engine_data_owner_body_budget_and_recovery_use_original_capture(){
 for(provider,engine)in[("lmstudio","ollama"),("ollama","lmstudio")]{
  let(mut server,http)=fixture_with_hook(vec![("application/json",metadata(engine).to_string()),("text/event-stream",stream("Explicit engine goal result",true))],|_|{});server.id=engine;
  let config=explicit_config(provider,&server);let(f,store,_authority,g,owner,main)=setup_configuration(&server,false,Some((provider,config.clone())));
  assert_ne!(g.spec.owner.provider,g.spec.owner.adapter);assert_eq!(g.execution_context.as_ref().unwrap().provider,provider);
  let grant=g.admissions[0].spec.memory_data.as_ref().unwrap();assert!(grant.valid(engine,"fixture-free"));assert!(!grant.valid(provider,"fixture-free"));g.validate().unwrap();
  // App/dispatcher projection cannot consult either old global endpoint/key.
  let mut unrelated=crate::settings::Settings::default();unrelated.ollama_url="http://127.0.0.1:1".into();unrelated.lmstudio_url="http://127.0.0.1:2".into();
  let projected=owner.server_projection(&unrelated).unwrap();assert_eq!(projected.id,engine);assert_eq!(projected.url,server.url);assert!(projected.key.is_none());
  assert_eq!(run(owner.respond_once(None,&g,&projected,"explicit-call",&config)).unwrap(),"Explicit engine goal result");
  let requests=http.join().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[1].0,"/v1/chat/completions");let body=&requests[1].1;assert_eq!(body["model"],"fixture-free");
  let memory=body["messages"].as_array().unwrap().iter().filter(|m|m["content"].as_str().is_some_and(|s|s.contains("PRIVATE_GOAL_MEMORY_SENTINEL"))).collect::<Vec<_>>();assert_eq!(memory.len(),1);assert!(memory[0]["content"].as_str().unwrap().contains(&g.spec.owner.pet_id));assert!(body.get("tools").is_none());assert!(main.history(provider).is_empty());
  let before=ledger(&f,&g);assert_eq!(before["attempts"].as_array().unwrap().len(),1);assert_eq!(before["attempts"][0]["inputHash"],crate::pet_runtime::model::fingerprint(&(&owner.lease().id,"fixture-free",body)).unwrap());assert_eq!(before["attempts"][0]["phase"],"settled");
  let reopened=Store::ensure(&f.root.join("goals")).unwrap();let restored=reopened.read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(serde_json::to_vec(&restored.admissions).unwrap(),serde_json::to_vec(&g.admissions).unwrap());assert!(store.read(&g.id,if provider=="ollama"{"ai_lmstudio"}else{"ai_ollama"}).is_err());
  // Durable receipts are consulted before re-resolving a changed endpoint or
  // renewing authority; a read-back is not another model effect.
  let mut changed=config.clone();let p=changed.pets.get_mut(&g.spec.owner.pet_id).unwrap();p.inference.as_mut().unwrap().endpoint="http://127.0.0.1:3".into();p.inference_revision=Some("f".repeat(32));fs::write(f.root.join("pets.json"),serde_json::to_vec(&changed).unwrap()).unwrap();
  assert_eq!(run(owner.respond_once(None,&g,&projected,"explicit-call",&changed)).unwrap(),"Explicit engine goal result");assert_eq!(ledger(&f,&g),before);assert!(run(owner.respond_once(None,&g,&projected,"new-call-after-change",&changed)).is_err());assert_eq!(ledger(&f,&g),before);assert_eq!(owner.workers(),0);
 }
}
#[test]fn pet_inference_goal_destination_review_uses_same_configuration_and_refuses_invalid_or_changed_binding(){
 let(mut server,http)=fixture_with_hook(vec![],|_|{});server.id="ollama";let config=explicit_config("lmstudio",&server);
 let(f,_store,_a,g,owner,_main)=setup_configuration(&server,false,Some(("lmstudio",config.clone())));assert!(http.join().unwrap().is_empty());
 let settings=crate::settings::Settings::default();let resolved=crate::pet_runtime::memory_review::server_for_goal(&g,&config,&settings,"fixture-free").unwrap();assert_eq!(resolved.id,"ollama");assert_eq!(resolved.url,server.url);
 for mode in ["model","adapter","malformed"]{let mut changed=config.clone();let choice=changed.pets.get_mut("ai_lmstudio").unwrap().inference.as_mut().unwrap();match mode{"model"=>choice.model="not-reviewed".into(),"adapter"=>choice.backend=crate::pet_inference::Backend::Lmstudio,_=>choice.endpoint="https://example.invalid".into()};assert!(crate::pet_runtime::memory_review::server_for_goal(&g,&changed,&settings,"fixture-free").is_err());}
 assert!(ledger(&f,&g).is_null());assert_eq!(owner.workers(),0);
}
#[test]fn pet_inference_goal_revocation_during_metadata_or_response_never_publishes_late_success(){
 for stage in ["metadata","response"]{
  let path=Arc::new(std::sync::Mutex::new(None::<PathBuf>));let target=path.clone();let during=stage.to_owned();
  let responses=if stage=="metadata"{vec![("application/json",local_metadata().to_string())]}else{vec![("application/json",local_metadata().to_string()),("text/event-stream",stream("Retired binding late response",true))]};
  let(server,http)=fixture_with_hook(responses,move|route|{if route==if during=="metadata"{"/api/show"}else{"/v1/chat/completions"}{let path=target.lock().unwrap();let path=path.as_ref().unwrap();let mut c=crate::pet_config::read_existing_at(path).unwrap();c.pets.get_mut("ai_lmstudio").unwrap().inference_revision=Some("f".repeat(32));fs::write(path,serde_json::to_vec(&c).unwrap()).unwrap();}});
  let config=explicit_config("lmstudio",&server);let(f,_store,_a,g,owner,main)=setup_configuration(&server,false,Some(("lmstudio",config.clone())));*path.lock().unwrap()=Some(f.root.join("pets.json"));
  assert!(run(owner.respond_once(None,&g,&server,"retired-in-flight",&config)).is_err());let requests=http.join().unwrap();assert_eq!(requests.len(),if stage=="metadata"{1}else{2});
  let before=ledger(&f,&g);if stage=="metadata"{assert!(before.is_null());}else{assert_eq!(before["attempts"].as_array().unwrap().len(),1);assert_eq!(before["attempts"][0]["inputHash"],crate::pet_runtime::model::fingerprint(&(&owner.lease().id,"fixture-free",&requests[1].1)).unwrap());}
  assert!(!fs::read_to_string(f.contexts().join(&g.id).join(JOURNAL)).unwrap().contains("Retired binding late response"));assert_eq!(owner.workers(),0);
  // Restoring the exact old bytes cannot reactivate the observed retired hold.
  fs::write(f.root.join("pets.json"),serde_json::to_vec(&config).unwrap()).unwrap();assert!(run(owner.respond_once(None,&g,&server,"another-after-rollback",&config)).is_err());assert_eq!(ledger(&f,&g),before);assert!(main.reserve_conversation("lmstudio",None).is_err(),"stopped physical owner remains held until drain/drop");
 }
}

fn explicit_settings(pet:&str)->crate::settings::Settings{
 let mut s=crate::settings::Settings::default();s.main_pill=pet.into();s.ollama_url="http://127.0.0.1:1".into();s.lmstudio_url="http://127.0.0.1:2".into();s
}

struct ExplicitEnvironment{profile:PathBuf,address:String,allow:bool,decisions:std::sync::atomic::AtomicUsize}
impl crate::pet_runtime::lease_controller::dispatcher::Environment for ExplicitEnvironment{
 fn capture(&self,owner:&crate::pet_runtime::lease_controller::Owned)->crate::pet_runtime::model::Result<crate::pet_runtime::lease_controller::dispatcher::Capture>{
  let configuration=self.configuration()?;let settings=explicit_settings(crate::pet_config::pet_id(&owner.lease().provider).unwrap());
  Ok(crate::pet_runtime::lease_controller::dispatcher::Capture{server:owner.server_projection(&settings).map_err(|_|crate::pet_runtime::model::Error::Conflict)?,configuration,app:None})
 }
 fn configuration(&self)->crate::pet_runtime::model::Result<crate::pet_config::Configuration>{crate::pet_config::read_existing_at(&self.profile).map_err(|_|crate::pet_runtime::model::Error::StorageUnavailable)}
 fn target(&self)->crate::pet_runtime::model::Result<Arc<crate::spotify::goal::Target>>{Ok(Arc::new(crate::spotify::goal::Target::fixture_at(&self.address).unwrap()))}
 fn decide(&self,source:crate::pet_runtime::recipe_decisions::Source)->Option<crate::pet_runtime::recipe_decisions::Decision>{
  self.decisions.fetch_add(1,std::sync::atomic::Ordering::AcqRel);let fixture=crate::cli_chat::goal_decision_fixture(source);fixture.answer(json!({"allow":self.allow}),fixture.request["goalTarget"].clone()).unwrap();fixture.decision()
 }
 fn notify(&self,_:&crate::pet_runtime::lease_controller::dispatcher::Status){}
}
#[test]fn pet_inference_goal_full_driver_uses_own_permissions_native_recipe_approval_and_final_evidence(){
 use crate::pet_runtime::{lease_controller::dispatcher::{Driver,Phase},model::State};
 use crate::spotify::goal::tests::{Bus,Player};
 for(provider,engine,allow)in[("lmstudio","ollama",true),("ollama","lmstudio",true),("lmstudio","ollama",false)]{
  let plan=crate::pet_runtime::planning_tests::output().to_string();let(mut server,http)=fixture_with_hook(vec![("application/json",metadata(engine).to_string()),("text/event-stream",stream(&plan,true))],|_|{});server.id=engine;
  let mut config=explicit_config(provider,&server);let pet=crate::pet_config::pet_id(provider).unwrap();config.pets.get_mut(pet).unwrap().permissions.get_or_insert_default().music=true;
  let(f,_store,authority,g,owner,main)=setup_mode(&server,false,Some((provider,config.clone())),true);let bus=Bus::new();let player=Player::new(&bus,"Private explicit-engine driver");let env=Arc::new(ExplicitEnvironment{profile:f.root.join("pets.json"),address:bus.address.clone(),allow,decisions:std::sync::atomic::AtomicUsize::new(0)});
  let owner=Arc::new(owner);let store=Arc::new(Store::existing_verified(&f.root.join("goals"),&f.root.join("authority")).unwrap().unwrap());let driver=Driver::new(owner.clone(),store.clone(),env.clone()).unwrap();let status=run(driver.run()).unwrap();assert_eq!(status.phase,if allow{Phase::Completed}else{Phase::Declined});assert_eq!(status.pet_id,pet);assert_eq!(env.decisions.load(std::sync::atomic::Ordering::Acquire),1);
  let final_goal=store.read(&g.id,pet).unwrap();assert_eq!(final_goal.spec.owner,g.spec.owner);assert_eq!(final_goal.state,if allow{State::Completed}else{State::RecoveryRequired});if allow{authority.verify_completed_goal(&final_goal).unwrap();}
  assert_eq!(player.calls.lock().unwrap().len(),usize::from(allow));assert_eq!(owner.workers(),0);let requests=http.join().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[1].0,"/v1/chat/completions");assert!(requests[1].1.to_string().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));
  let attempts=ledger(&f,&g)["attempts"].as_array().unwrap().clone();assert_eq!(attempts.iter().filter(|a|a["kind"]=="model").count(),1);assert_eq!(attempts.iter().filter(|a|a["kind"]=="tool").count(),usize::from(allow));assert!(!status.active);assert!(main.runtime_resources().busy().unwrap().is_empty());
 }
}
