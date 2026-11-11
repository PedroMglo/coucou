//! Exact funded61 ownership recovery; isolated XDG, generated native processes
//! and loopback protocols only. Restoration never supplies this authority.
use super::*;
use super::super::super::super::super::funded_planning as planner;

pub(super)fn publish(s:&Arc<Setup>,provider:&'static str,ready:&std::path::Path){
 let pet=crate::pet_config::pet_id(provider).unwrap();let(_parent,g,r)=planner::draft_planner(s,pet);
 let server=crate::local_chat::Server{id:provider,name:"Original generated funded destination",url:std::env::var("COUCOU_PEER_RECOVERY_ENDPOINT").unwrap(),key:None};
 assert!(crate::net::is_loopback_url(&reqwest::Url::parse(&server.url).unwrap()));
 let q=planner::admitted(s,&g,&r,&server);let store=current_store(s);
 let pending=Controller::new().unwrap().prepare_memory_fixture(&q,&s.f.path.join("authority"),&s.f.path.join("contexts"),&catalog(),&s.main,&memory_evidence(s,&q,&server),nonce().unwrap(),crate::assistant_memory::now(),Arc::new(|_,_,_|Validity::Current)).unwrap();
 let(receipt,owner)=pending.claim(&store,&s.a,&memory_evidence(s,&q,&server),crate::assistant_memory::now()).unwrap();let owner=Arc::new(owner.unwrap());
 run(owner.plan_once(None,&receipt.record,&server,"driver-plan-1",&s.config)).unwrap();
 let g=owner.publish_generated_plan(&Arc::new(current_store(s)),"driver-plan-1",crate::assistant_memory::now()).unwrap().receipt.record;
 let batch=crate::pet_runtime::peer_plan_review::Batch::capture(&owner,&g,&s.config,&s.settings,crate::assistant_memory::now()).unwrap().unwrap();
 let observe=plan_observe(s);let source=Source::capture_peer(owner.clone(),&g,batch.clone(),Arc::new(current_store(s)),observe.clone()).unwrap();
 owner.yield_for_decision(&Arc::new(current_store(s)),&source,crate::assistant_memory::now()).unwrap();
 let frame=crate::cli_chat::goal_decision_fixture(source.clone());frame.panel_answer(true,frame.request["goalTarget"].clone()).unwrap();let decision=frame.decision().unwrap();
 owner.resume_after_decision(&Arc::new(current_store(s)),&source,crate::assistant_memory::now()).unwrap();
 let approvals=batch.approve(&decision,&owner,&g,&observe,crate::assistant_memory::now()).unwrap();assert_eq!(approvals.len(),2);
 let first=owner.send_message(&g,&approvals[0],crate::assistant_memory::now(),||s.observe()).unwrap();assert_eq!(first.envelope().schema_version,16);assert_eq!(owner.workers(),0);
 fs::write(ready,serde_json::to_vec(&json!({"goalId":g.id,"petId":pet})).unwrap()).unwrap();
 let until=Instant::now()+Duration::from_secs(20);while Instant::now()<until{assert!(owner.policy_current(crate::assistant_memory::now()));std::thread::sleep(Duration::from_millis(20));}
}

fn exercise(provider:&'static str,revoke:bool){
 let deadline=crate::assistant_memory::now()+120_000;let mut plan=proposal(provider,deadline);
 // The other local engine is already an ancestor of this funded recipient.
 // Its nested branch goes to an independent deterministic specialist instead.
 plan["steps"][0]["operation"]["arguments"]["toPetId"]=json!("integration_spotify");
 plan["steps"][0]["operation"]["arguments"]["allowance"]["outputBytes"]=json!(8192);
 let meta=if provider=="lmstudio"{lm_metadata()}else{metadata()};
 let(mut original_server,http)=crate::pet_runtime::lease_controller::memory_test_http_fixture(vec![("application/json",meta.to_string()),("text/event-stream",stream(&plan.to_string(),true)),("application/json",meta.to_string())],|_|{});original_server.id=provider;
 let mut env=Process::with_endpoint(provider,"funded-planning-partial",Some(&original_server));assert_eq!(env.g.schema_version,61);
 env.crash();let closed=env.close();let before=body(&env.s,&closed);assert_eq!(before["schemaVersion"],47);
 let signed=env.s.a.recovery_body(&closed).unwrap();let server=crate::local_chat::Server{id:provider,name:"Original destination; passive observation only",url:match &signed.peer_memory_planning.as_ref().unwrap().destination{crate::pet_memory::context_model::Destination::Server{endpoint,..}=>endpoint.trim_end_matches("/v1/chat/completions").to_owned(),_=>panic!("local destination")},key:None};
 let memory=passive(&env.s,&closed,&server).unwrap();let review=resume_review(&env,&closed,"resume-exact-funded61-owner");
 let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
 approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&memory,Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
 if revoke{
  let source=&closed.spec.received_from.as_ref().unwrap().body.source;let parent=current_store(&env.s).read(&source.sender_goal_id,&source.sender_pet_id).unwrap();
  let branch=Authority::existing(&env.s.f.path.join("authority")).unwrap().prepare_delegation_branch(parent,&source.reference.decision_id,&source.reference.message_id,crate::pet_runtime::delegation::BranchAction::Cancel,crate::assistant_memory::now()).unwrap();
  branch.confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap();assert!(approved.confirm(crate::assistant_memory::now()).is_err());assert!(!approved.published().unwrap());
  let after=body(&env.s,&closed);assert_eq!(after["ownerResumptions"],before["ownerResumptions"]);assert_eq!(after["runtimeOwner"],before["runtimeOwner"]);assert_eq!(after["budgetUsage"],before["budgetUsage"]);
  assert!(approved.withdraw().unwrap());drop(approved);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());assert!(passive(&env.s,&closed,&server).is_err());
 }else{
  let(receipt,owner)=approved.confirm(crate::assistant_memory::now()).unwrap();let g=receipt.record;let owner=Arc::new(owner.unwrap());
  assert_eq!(g.schema_version,61);assert_eq!(g.admissions.len(),1);assert_eq!(model::fingerprint(&g.admissions[0].spec).unwrap(),model::fingerprint(&closed.admissions[0].spec).unwrap());assert_eq!(g.admissions[0].lease,closed.admissions[0].lease);assert_eq!(g.execution_context,closed.execution_context);
  let resumed=body(&env.s,&g);assert_eq!(resumed["schemaVersion"],47);assert_eq!(resumed["ownerResumptions"].as_array().unwrap().len(),1);
  for field in["fundedBy","budgetUsage","planningUsage","delegationPool","delegationReservations","messages","peerMemoryPlanning"]{assert_eq!(resumed[field],before[field],"handoff changed {field}");}
  let runtime=Arc::new(ResumeEnv{s:env.s.clone(),owner:owner.clone(),url:server.url.clone(),decisions:AtomicUsize::new(0),waits:AtomicUsize::new(0),received:true});
  let driver=Driver::new(owner.clone(),Arc::new(current_store(&env.s)),runtime.clone()).unwrap();driver.spawn().unwrap();
  let until=Instant::now()+Duration::from_secs(20);let status=loop{let status=driver.status().unwrap();if !status.active{break status;}assert!(Instant::now()<until,"{status:?}");std::thread::sleep(Duration::from_millis(5));};
  assert_eq!(status.phase,Phase::Stopped,"{status:?}");assert_eq!(runtime.decisions.load(Ordering::Acquire),1);assert_eq!(runtime.waits.load(Ordering::Acquire),1);
  let latest=current_store(&env.s).read(&g.id,&g.spec.owner.pet_id).unwrap();assert!(latest.completion.is_none());let after=body(&env.s,&latest);
  assert_eq!(after["messages"]["entries"].as_array().unwrap().len(),2);assert_eq!(after["messages"]["entries"][0],before["messages"]["entries"][0]);assert_eq!(after["fundedBy"],before["fundedBy"]);assert_eq!(after["delegationReservations"],before["delegationReservations"]);
  let old=before["budgetUsage"]["attempts"].as_array().unwrap();let attempts=after["budgetUsage"]["attempts"].as_array().unwrap();assert_eq!(attempts.len(),old.len()+1);assert_eq!(&attempts[..old.len()],old.as_slice());assert_eq!(attempts.last().unwrap()["kind"],"tool");
  assert_eq!(owner.workers(),0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
 }
 let requests=http.join().unwrap();assert_eq!(requests.len(),2,"funded handoff repeated model/metadata IO");let input=requests[1].1["messages"].to_string();assert!(input.contains("receivedDelegationData"));assert!(!input.contains("PRIVATE_HANDOFF_MEMORY_SENTINEL"));
}

#[test]#[ignore="explicit isolated-XDG funded61 subprocess control; generated providers only"]
fn funded_planning_handoff_control(){catalog();for provider in["ollama","lmstudio"]{exercise(provider,false);}exercise("ollama",true);}

#[test]
fn funded_planning_real_process_handoff_preserves_original_escrow_and_late_revocation(){
 let isolation=Fixture::new();let root=isolation.path.join("xdg");fs::create_dir(&root).unwrap();
 let helper=concat!(module_path!(),"::funded_planning_handoff_control").split_once("::").unwrap().1;
 let mut cmd=Command::new(std::env::current_exe().unwrap());cmd.args(["--exact",helper,"--ignored","--nocapture","--test-threads=1"]).env("COUCOU_MEMORY_HANDOFF_XDG",&root);
 for(key,name)in[("XDG_CONFIG_HOME","config"),("XDG_DATA_HOME","data"),("XDG_CACHE_HOME","cache")]{let path=root.join(name);fs::create_dir(&path).unwrap();cmd.env(key,path);}
 let log=isolation.path.join("control.log");let output=fs::OpenOptions::new().create_new(true).write(true).open(&log).unwrap();cmd.stdout(Stdio::from(output.try_clone().unwrap())).stderr(Stdio::from(output));
 let mut child=ChildGuard(cmd.spawn().unwrap());let until=Instant::now()+Duration::from_secs(90);
 let status=loop{if let Some(s)=child.0.try_wait().unwrap(){break s;}assert!(Instant::now()<until,"funded61 recovery process timed out");std::thread::sleep(Duration::from_millis(10));};assert!(status.success(),"funded61 native handoff: {}",fs::read_to_string(log).unwrap());
}
