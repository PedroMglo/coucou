//! Real private process/profile loss over the existing peer runtime. Synthetic
//! HTTP only; no installed process, provider account or host policy is touched.
use super::*;
use crate::{pet_runtime::{recovery_controller::Prepared,runtime_owner::State as OwnerState},spotify::goal::tests::Bus};
use std::{path::PathBuf,process::{Child,Command,Stdio},time::{Duration,Instant}};
#[path="peer_compatibility_export_tests.rs"]mod peer_compatibility_export_tests;
#[path="selected_memory_process_recovery_tests.rs"]mod selected_memory_process_recovery_tests;

const PREFIX:&str="com.coucou.PeerRecoveryFixture";
struct ChildGuard(Child);
impl Drop for ChildGuard{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}

#[test]
#[ignore="private subprocess helper invoked by the real recovery tests"]
fn peer_process_owner_child(){
 let Ok(profile)=std::env::var("COUCOU_PEER_RECOVERY_PROFILE")else{return;};
 let profile=PathBuf::from(profile);let bus=std::env::var("COUCOU_PEER_RECOVERY_BUS").unwrap();
 let ready=PathBuf::from(std::env::var("COUCOU_PEER_RECOVERY_READY").unwrap());
 let provider=match std::env::var("COUCOU_PEER_RECOVERY_PROVIDER").unwrap().as_str(){"ollama"=>"ollama","lmstudio"=>"lmstudio",_=>panic!("private provider")};
 let mode=std::env::var("COUCOU_PEER_RECOVERY_MODE").unwrap();
 let connection=dbus::blocking::Connection::new_address(&bus).unwrap();
 assert_eq!(connection.request_name(&format!("{PREFIX}.SingleInstance"),false,false,true).unwrap(),dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner);
 let claim=crate::pet_data::fixture_native_claim(&bus,PREFIX,&profile).unwrap();claim.current().unwrap();
 let s=Arc::new(Setup::with_fixture(Fixture{path:profile}));
 if mode=="selected-memory-partial"{selected_memory_process_recovery_tests::publish_partial(&s,provider,&ready);return;}
 if mode=="received-memory-partial"{selected_memory_process_recovery_tests::publish_received_partial(&s,provider,&ready);return;}
 if mode=="funded-planning-partial"{selected_memory_process_recovery_tests::publish_funded_partial(&s,provider,&ready);return;}
 if matches!(mode.as_str(),"resume-ack-loss"|"resume-grant-loss"|"resume-grant-loss-stop"){
  let id=std::env::var("COUCOU_PEER_RECOVERY_EXISTING_GOAL").unwrap();let pet=std::env::var("COUCOU_PEER_RECOVERY_EXISTING_PET").unwrap();let g=current_store(&s).read_control(&id,&pet).unwrap();let old=s.a.recovery_body(&g).unwrap();let server=resume_server(provider,&old);
  let source=Arc::new(current_store(&s)).prepare_peer_planning_continuation(&s.f.path.join("authority"),&g.id,&g.spec.owner.pet_id,&g.revision,crate::assistant_memory::now()).unwrap();
  let review=crate::pet_runtime::lease_controller::peer_resume_controller::Review::prepare(source,"review-in-second-real-process".into(),crate::assistant_memory::now()).unwrap();let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
  approved.prepare_worker(&s.main,&s.f.path.join("contexts"),&resumed_evidence(&s,&g,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
  if mode=="resume-ack-loss"{Arc::get_mut(&mut approved.source.store).unwrap().fault=crate::pet_runtime::store::Fault::BeforePublish;assert!(approved.confirm(crate::assistant_memory::now()).is_err());assert!(approved.published().unwrap());}
  else{let mut writer=Authority::existing(&s.f.path.join("authority")).unwrap();writer.fault=crate::pet_runtime::authority_store::Fault::BeforeGrantPublish;assert!(writer.resume_peer_owner(&mut approved,crate::assistant_memory::now()).is_err());assert!(!approved.published().unwrap());assert!(current_store(&s).retire_lost_peer_owner_preparations(&s.f.path.join("authority"),&g.id,&g.spec.owner.pet_id,&g.revision).is_err());}
  fs::write(&ready,serde_json::to_vec(&json!({"goalId":g.id,"petId":g.spec.owner.pet_id})).unwrap()).unwrap();
  let until=Instant::now()+Duration::from_secs(20);while Instant::now()<until{claim.current().unwrap();std::thread::sleep(Duration::from_millis(20));}return;
 }
 if mode=="bound"||mode=="bound-inference"{
  let to=if mode=="bound"||provider=="lmstudio"{"ai_lmstudio"}else{"ai_ollama"};let sender=if to=="ai_ollama"{"ai_lmstudio"}else{"ai_ollama"};
  let(g,o)=pool_to(&s,sender,to,vec![crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION.into()],2,budget());
  let receipts:Vec<_>=(0..2).map(|i|{let a=approve(&s,&g,&o,i);o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap()}).collect();
  let queued=if mode=="bound-inference"{
   let server=crate::local_chat::Server{id:provider,name:"Original independently reviewed destination",url:std::env::var("COUCOU_PEER_RECOVERY_ENDPOINT").unwrap(),key:None};
   let child=response_draft_from_receipt(&s,to,&receipts[0],true,None);queue(&s,&child,&receipts[0],&server)
  }else{let child=own_draft(&s,&receipts[0]);let a=approved(&s,&child,child_budget(&receipts[0]),nonce().unwrap());let cert=s.a.certify(&a,&child,crate::assistant_memory::now()).unwrap();current_store(&s).enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record};assert_eq!(queued.state,State::Queued);
  let pending=ready.with_extension("pending");fs::write(&pending,serde_json::to_vec(&json!({"goalId":g.id,"petId":g.spec.owner.pet_id,"childGoalId":queued.id,"childPetId":queued.spec.owner.pet_id,"pendingMessageId":receipts[1].envelope().message_id})).unwrap()).unwrap();fs::rename(pending,ready).unwrap();
  let until=Instant::now()+Duration::from_secs(20);while Instant::now()<until{claim.current().unwrap();assert!(o.policy_current(crate::assistant_memory::now()));std::thread::sleep(Duration::from_millis(20));}return;
 }
 let r=prepare_in(s,provider,2,true,mode=="uncertain");
 if matches!(mode.as_str(),"partial"|"partial-native"){
  let lineage=r.o.reconcile_peer_round_plan(Arc::new(current_store(&r.s)),&r.request,crate::assistant_memory::now()).unwrap();
  let batch=approve_batch(&r,lineage);r.o.send_message(&r.g,&batch[0],crate::assistant_memory::now(),||r.s.observe()).unwrap();
 }
 assert_eq!(r.requests.len(),4);assert_eq!(r.o.workers(),0);claim.current().unwrap();
 let pending=ready.with_extension("pending");fs::write(&pending,serde_json::to_vec(&json!({"goalId":r.g.id,"petId":r.g.spec.owner.pet_id})).unwrap()).unwrap();fs::rename(pending,ready).unwrap();
 let until=Instant::now()+Duration::from_secs(20);
 while Instant::now()<until{claim.current().unwrap();std::thread::sleep(Duration::from_millis(20));}
}

struct Process{
 // Reap the owned process before removing its private profile on any panic.
 child:ChildGuard,s:Arc<Setup>,g:Record,store:crate::pet_runtime::store::Store,info:Value,
 connection:dbus::blocking::Connection,bus:Bus,_parent:Fixture,claim:Option<Box<dyn std::any::Any>>,
}
impl Process{
 fn new(provider:&str,mode:&str)->Self{Self::with_endpoint(provider,mode,None)}
 fn with_endpoint(provider:&str,mode:&str,server:Option<&crate::local_chat::Server>)->Self{
  let parent=if mode=="partial-native"{
   use std::os::unix::fs::DirBuilderExt;
   let output=PathBuf::from(std::env::var("COUCOU_H3D2_NATIVE_PROFILE").expect("explicit native profile"));assert!(output.is_absolute());let path=output.parent().unwrap().join(format!("native-source-{}",nonce().unwrap()));fs::DirBuilder::new().mode(0o700).create(&path).unwrap();Fixture{path}
  }else{Fixture::new()};let profile=parent.path.join("coucou");let ready=parent.path.join("ready.json");let bus=Bus::new();let log=parent.path.join("child.log");
  // Rust's module_path includes the crate; libtest's exact names do not.
  let helper=concat!(module_path!(),"::peer_process_owner_child").split_once("::").unwrap().1;
  let mut child=ChildGuard(Command::new(std::env::current_exe().unwrap()).args(["--exact",helper,"--ignored","--nocapture"])
   .env("COUCOU_PEER_RECOVERY_PROFILE",&profile).env("COUCOU_PEER_RECOVERY_BUS",&bus.address).env("COUCOU_PEER_RECOVERY_READY",&ready).env("COUCOU_PEER_RECOVERY_PROVIDER",provider).env("COUCOU_PEER_RECOVERY_MODE",mode)
   .env("COUCOU_PEER_RECOVERY_ENDPOINT",server.map(|s|s.url.as_str()).unwrap_or(""))
   .stdout(Stdio::from(fs::OpenOptions::new().create(true).append(true).open(&log).unwrap())).stderr(Stdio::from(fs::OpenOptions::new().append(true).open(&log).unwrap())).spawn().unwrap());
  let until=Instant::now()+Duration::from_secs(20);
  while !ready.exists(){assert!(Instant::now()<until&&child.0.try_wait().unwrap().is_none(),"private peer owner readiness: {}",fs::read_to_string(&log).unwrap_or_default());std::thread::sleep(Duration::from_millis(10));}
  let info:Value=serde_json::from_slice(&fs::read(&ready).unwrap()).unwrap();
  // Setup only opens existing authority/profile; it does not mint a lease.
  let s=Arc::new(Setup::with_fixture(Fixture{path:profile}));let store=current_store(&s);let g=store.read(info["goalId"].as_str().unwrap(),info["petId"].as_str().unwrap()).unwrap();
  let connection=dbus::blocking::Connection::new_address(&bus.address).unwrap();Self{child,s,g,store,info,connection,bus,_parent:parent,claim:None}
 }
 fn observation(&self)->OwnerState{let a=&self.g.admissions[0];self.s.a.owner_observation(&a.spec,a.proof.as_ref().unwrap()).unwrap().state}
 fn crash(&mut self){
  assert_eq!(self.observation(),OwnerState::OriginalProcessLive);let before=self.s.bytes(&self.g);
  let now=crate::assistant_memory::now();assert!(self.store.prepare_recovery(&self.s.f.path.join("authority"),&self.g.id,&self.g.spec.owner.pet_id,&self.g.revision,"refuse-live-owner-after-ttl".into(),self.g.admissions[0].spec.budget.deadline_at+1).is_err());assert_eq!(self.s.bytes(&self.g),before);
  self.child.0.kill().unwrap();self.child.0.wait().unwrap();
  assert_eq!(self.connection.request_name(&format!("{PREFIX}.SingleInstance"),false,false,true).unwrap(),dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner);
  let claim=crate::pet_data::fixture_native_claim(&self.bus.address,PREFIX,&self.s.f.path).unwrap();claim.current().unwrap();self.claim=Some(Box::new(claim));
  assert_eq!(self.observation(),OwnerState::OwnerAbsent);assert!(crate::assistant_memory::now()>=now);
 }
 fn close(&self)->Record{
  let now=crate::assistant_memory::now();let review=match self.store.prepare_recovery(&self.s.f.path.join("authority"),&self.g.id,&self.g.spec.owner.pet_id,&self.g.revision,format!("close-exact-lost-peer-owner-{}",self.g.sequence),now).unwrap(){Prepared::Review(r)=>r,_=>panic!("a lost original process requires its own review")};
  let p=review.proposal();assert!(!p.execution_available);assert_eq!(p.approval_scope,"closeLostExecution");
  let accounting=&p.snapshot["accounting"];let body:crate::pet_runtime::authority::Grant=serde_json::from_value(body(&self.s,&self.g)).unwrap();
  let own=body.budget_usage.as_ref().map(|u|u.attempts.iter().filter(|a|a.kind==crate::pet_runtime::effect_budget::Kind::Model).count()).unwrap_or(0);
  assert_eq!(accounting["modelCallsUsed"],own);assert_eq!(accounting["delegatedHeld"],serde_json::to_value(body.delegated_held().unwrap()).unwrap());assert_eq!(accounting["modelCallsRemaining"],body.budget_remaining(&self.g.admissions[0].spec.budget).unwrap().model_calls);
  let chosen=review.decide(crate::assistant_memory::now()).unwrap();let mut approved=review.approve(chosen);let closed=approved.confirm(&self.store,crate::assistant_memory::now()).unwrap().record;
  assert_eq!(closed.state,State::RecoveryRequired);assert!(closed.completion.is_none());
  let after=self.s.bytes(&self.g);assert_eq!(approved.confirm(&self.store,crate::assistant_memory::now()).unwrap().record.revision,closed.revision);assert_eq!(self.s.bytes(&self.g),after);
  drop(approved);let reopen=crate::pet_runtime::store::Store::existing_verified(&self.s.f.path.join("goals"),&self.s.f.path.join("authority")).unwrap().unwrap();
  assert!(matches!(reopen.prepare_recovery(&self.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,"reopen-recorded-close".into(),crate::assistant_memory::now()).unwrap(),Prepared::Closed(_)));assert_eq!(self.s.bytes(&self.g),after);closed
 }
}

#[test]fn peer_planning_handoff_capture_preserves_exact_remaining_escrow_receipts_and_unknown_outcomes(){
 use crate::pet_runtime::continuation::PeerPlanningOutcome;
 for provider in["ollama","lmstudio"]{for mode in["partial","uncertain"]{
  let mut env=Process::new(provider,mode);let store=Arc::new(current_store(&env.s));
  assert!(store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&env.g.id,&env.g.spec.owner.pet_id,&env.g.revision,crate::assistant_memory::now()).is_err());
  env.crash();let closed=env.close();let before=env.s.bytes(&closed);
  let capture=store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,crate::assistant_memory::now()).unwrap();
  let snapshot=capture.snapshot();let signed=env.s.a.recovery_body(&closed).unwrap();let original=&closed.admissions[0].spec;
  assert_eq!(snapshot.original_budget,original.budget);assert_eq!(snapshot.original_policy,original.planning_policy.clone().unwrap());
  assert_eq!(snapshot.remaining,signed.budget_remaining(&original.budget).unwrap());assert_eq!(snapshot.delegated_held,signed.delegated_held().unwrap());
  assert_eq!(snapshot.remaining.model_calls,0,"own settled/unknown attempts and the child reservation all remain charged");
  assert_eq!(snapshot.delegated_held.model_calls,1);assert_eq!(snapshot.rounds_observed,2);assert_eq!(snapshot.model,original.resolved_model);
  assert_eq!(snapshot.last_reviewed_destination,Some(signed.planning_usage.as_ref().unwrap().rounds[1].peer_data.as_ref().unwrap().source.destination.clone()));
  assert!(!snapshot.execution_available&&!snapshot.model_use_authorized&&!snapshot.additional_allowance);
  if mode=="partial"{
   assert_eq!(snapshot.outcome,PeerPlanningOutcome::CanonicalPlan);assert_eq!(snapshot.receipts.len(),2);assert_eq!(snapshot.pending_steps.len(),1);
   let pending=&snapshot.pending_steps[0];assert!(closed.plans.iter().any(|p|p.id==pending.plan_id&&p.steps.iter().any(|s|s.id==pending.step_id)));
   assert!(!snapshot.receipts.iter().any(|r|r.plan_id==pending.plan_id&&r.step_id==pending.step_id));assert_eq!(snapshot.charge.unknown_model_results,0);
  }else{
   assert_eq!(snapshot.outcome,PeerPlanningOutcome::UncertainModelResult);assert_eq!(snapshot.receipts.len(),1);assert!(snapshot.pending_steps.is_empty());assert_eq!(snapshot.charge.unknown_model_results,1);
  }
  capture.current(crate::assistant_memory::now()).unwrap();assert_eq!(env.s.bytes(&closed),before,"read-only capture must not claim, top up or acknowledge an effect");
  assert!(capture.current(original.budget.deadline_at).is_err());assert!(capture.current(crate::assistant_memory::now()).is_err(),"expiry cannot be undone by rolling back the caller clock");
 }}
}

#[test]fn peer_planning_handoff_capture_survives_busy_but_never_revives_after_source_tampering_or_stop(){
 let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let store=Arc::new(current_store(&env.s));
 let foreign=Setup::new();let before=env.s.bytes(&closed);
 assert!(matches!(store.prepare_peer_planning_continuation(&foreign.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,crate::assistant_memory::now()),Err(model::Error::Conflict)));
 assert!(store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,"agent_codex",&closed.revision,crate::assistant_memory::now()).is_err());
 assert!(store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&nonce().unwrap(),crate::assistant_memory::now()).is_err());
 assert_eq!(env.s.bytes(&closed),before);
 let capture=store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,crate::assistant_memory::now()).unwrap();
 let lock=store.test_lock(true).unwrap();assert_eq!(capture.current(crate::assistant_memory::now()),Err(model::Error::Busy));drop(lock);capture.current(crate::assistant_memory::now()).unwrap();
 let signed=env.s.a.recovery_body(&closed).unwrap();let attempt=&signed.planning_usage.as_ref().unwrap().rounds[0].attempt_id;
 let path=env.s.a.result_root().unwrap().join(&closed.admissions[0].spec.decision_id).join(format!("{attempt}.json"));let original=fs::read(&path).unwrap();
 fs::write(&path,b"untrusted replacement of the original model receipt").unwrap();assert!(capture.current(crate::assistant_memory::now()).is_err());fs::write(&path,original).unwrap();assert!(capture.current(crate::assistant_memory::now()).is_err(),"restoring bytes cannot revive the old capture");
 let fresh=store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,crate::assistant_memory::now()).unwrap();fresh.current(crate::assistant_memory::now()).unwrap();
 let row=&closed.admissions[0];env.s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Pause,crate::assistant_memory::now()).unwrap();
 // A durable Stop wins even before its queue projection/ACK. A fresh capture
 // cannot turn that incomplete control write into permission to resume.
 assert!(store.prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,crate::assistant_memory::now()).is_err());
 assert!(fresh.current(crate::assistant_memory::now()).is_err());
 let stopped=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,"stop-before-peer-planning-handoff".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;
 assert!(fresh.current(crate::assistant_memory::now()).is_err());assert_eq!(body(&env.s,&stopped)["recovery"],serde_json::to_value(&signed.recovery).unwrap());
}

#[test]fn already_bound_independent_child_can_execute_after_actual_parent_process_closure_but_pending_cannot_admit(){
 let mut env=Process::new("ollama","bound");let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),env.info["childPetId"].as_str().unwrap()).unwrap();
 env.crash();let closed=env.close();let before=env.s.bytes(&closed);
 let inbox=env.s.a.message_inbox("ai_lmstudio",None,8,crate::assistant_memory::now()).unwrap();let pending=&inbox.items.iter().find(|i|i.receipt.envelope().message_id==env.info["pendingMessageId"]).unwrap().receipt;
 assert!(data_review(&env.s,pending,"no-new-child-after-lost-parent").is_err());assert_eq!(env.s.bytes(&closed),before);
 let(leased,o)=crate::pet_runtime::deterministic_runtime_tests::owned(&env.s.f,&env.s.a,&child,&env.s.main);
 let target=Arc::new(Target::capture(&env.s.config,&env.s.settings,"ai_ollama").unwrap());let now=crate::assistant_memory::now();let p=&leased.plans[0];
 let a=o.review_message(&leased,&p.id,&p.steps[0].id,"independent-child-after-lost-parent".into(),target,now).unwrap().approve(&o,&leased,&env.s.config,&env.s.settings,now,Arc::new(||true)).unwrap();
 let receipt=o.send_message(&leased,&a,crate::assistant_memory::now(),||env.s.observe()).unwrap();assert_eq!(receipt.envelope().schema_version,5);assert_eq!(receipt.envelope().origin.pet_id,"ai_lmstudio");
 assert_eq!(env.s.bytes(&closed),before);assert_eq!(env.store.read(&closed.id,"ai_ollama").unwrap().state,State::RecoveryRequired);
 let after=env.s.bytes(&leased);assert!(!o.policy_current(child.admissions[0].spec.budget.deadline_at));assert_eq!(env.s.bytes(&leased),after);
}

struct ResumedPeerEnv{s:Arc<Setup>,provider:&'static str,url:String,decisions:AtomicUsize}
impl Environment for ResumedPeerEnv{
 fn capture(&self,_:&Owned)->model::Result<Capture>{Ok(Capture{configuration:self.s.config.clone(),server:crate::local_chat::Server{id:self.provider,name:"Original endpoint; no new inference",url:self.url.clone(),key:None},app:None})}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.s.config.clone())}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,Settings)>{self.s.observe()}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("owner resumption is not System permission")}
 fn notify(&self,_:&Status){}
 fn decide(&self,source:Source)->Option<Decision>{
  self.decisions.fetch_add(1,Ordering::AcqRel);assert_eq!(source.target().origin,crate::pet_runtime::recipe_decisions::Origin::GoalPeerPlan);
  let review=source.review();let recipe=&review["recipe"];assert_eq!(recipe["steps"].as_array().unwrap().len(),1);
  assert_eq!(recipe["roundDataProvenance"]["reconciledFromCanonicalPlan"],true);assert_eq!(recipe["roundDataProvenance"]["nativeModelDataApprovalRetained"],false);
  assert_eq!(recipe["roundDataProvenance"]["canonicalReceiptsAlreadyObserved"].as_array().unwrap().len(),1);
  let native=crate::cli_chat::goal_decision_fixture(source);native.panel_answer(true,native.request["goalTarget"].clone()).unwrap();native.decision()
 }
}
fn resume_review(env:&Process,g:&Record,request:&str)->crate::pet_runtime::lease_controller::peer_resume_controller::Review{
 let captured=Arc::new(current_store(&env.s)).prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&g.id,&g.spec.owner.pet_id,&g.revision,crate::assistant_memory::now()).unwrap();
 crate::pet_runtime::lease_controller::peer_resume_controller::Review::prepare(captured,request.into(),crate::assistant_memory::now()).unwrap()
}
fn resume_server(provider:&'static str,g:&crate::pet_runtime::authority::Grant)->crate::local_chat::Server{
 let destination=&g.planning_usage.as_ref().unwrap().rounds.last().unwrap().peer_data.as_ref().unwrap().source.destination;
 let crate::pet_memory::context_model::Destination::Server{endpoint,..}=destination else{panic!("local synthetic endpoint")};
 crate::local_chat::Server{id:provider,name:"Original synthetic destination",url:endpoint.strip_suffix("/v1/chat/completions").unwrap().into(),key:None}
}
#[test]fn peer_owner_handoff_runs_pending_work_after_process_loss_without_replaying_model_or_receipts(){
 for provider in["ollama","lmstudio"]{
  let mut env=Process::new(provider,"partial");env.crash();let closed=env.close();let old=env.s.a.recovery_body(&closed).unwrap();let before=body(&env.s,&closed);let server=resume_server(provider,&old);
  let review=resume_review(&env,&closed,"resume-original-peer-execution");let proposal=review.proposal();assert!(!proposal.new_budget&&!proposal.new_model_data_consent);assert_eq!(proposal.approval_scope,"resumeOriginalPeerExecution");
  let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
  let ready=Arc::new(AtomicBool::new(false));let observed=ready.clone();
  approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&closed,&server,crate::assistant_memory::now()),Arc::new(move|_,_,_|if observed.load(Ordering::Acquire){Validity::Current}else{Validity::Busy}),crate::assistant_memory::now()).unwrap();
  let(receipt,owner)=approved.confirm(crate::assistant_memory::now()).unwrap();assert_ne!(receipt.status,model::CommitStatus::Uncertain);let leased=receipt.record;let owner=Arc::new(owner.unwrap());
  assert!(!owner.current(crate::assistant_memory::now()));assert!(owner.worker(crate::assistant_memory::now()).is_err());ready.store(true,Ordering::Release);assert!(owner.current(crate::assistant_memory::now()));
  assert_eq!(leased.state,State::Leased);assert_eq!(leased.schema_version,54);assert_eq!(leased.admissions.len(),1);assert_eq!(model::fingerprint(&leased.admissions[0].spec).unwrap(),model::fingerprint(&closed.admissions[0].spec).unwrap());assert_eq!(leased.admissions[0].lease,closed.admissions[0].lease);
  let after=body(&env.s,&leased);assert_eq!(after["schemaVersion"],44);assert_eq!(after["runtimeOwner"]["process"]["pid"],std::process::id());
  for field in["budgetUsage","planningUsage","delegationPool","delegationReservations","messages","lease"]{assert_eq!(after[field],before[field],"handoff preserves {field}");}
  assert_eq!(after["ownerResumptions"][0]["closure"],before["recovery"]);assert_eq!(after["ownerResumptions"][0]["previousOwner"],before["runtimeOwner"]);assert_eq!(after["ownerResumptions"][0]["previousResourceUsage"],before["resourceUsage"]);
  let current=env.s.a.recovery_body(&leased).unwrap();let historical=current.messages.as_ref().unwrap().entries.iter().find(|e|e.envelope().cause.plan_id==leased.plans.last().unwrap().id).unwrap();
  assert!(!historical.scope.matches(&leased));assert!(crate::pet_runtime::owner_resumption::receipt_scope_matches(&leased,&current,&historical.scope));
  let mut forged=current.clone();forged.owner_resumptions.clear();assert!(!crate::pet_runtime::owner_resumption::receipt_scope_matches(&leased,&forged,&historical.scope));
  let mut forged=current.clone();forged.owner_resumptions[0].closure.reference.source_goal_hash="a".repeat(64);assert!(!crate::pet_runtime::owner_resumption::receipt_scope_matches(&leased,&forged,&historical.scope));
  let mut malformed=current.owner_resumptions[0].reference.clone();malformed.source_sequence=u64::MAX;assert!(!malformed.valid());
  let mut edited=leased.clone();edited.revision="b".repeat(32);assert!(!crate::pet_runtime::owner_resumption::receipt_scope_matches(&edited,&current,&historical.scope));
  let bytes=env.s.bytes(&leased);let(ack,again)=approved.confirm(crate::assistant_memory::now()).unwrap();assert!(again.is_none());assert_eq!(ack.record.revision,leased.revision);assert_eq!(env.s.bytes(&leased),bytes);
  owner.message_goal(&leased,crate::assistant_memory::now()).expect("resumed owner and original message goal");
  let row=&leased.admissions[0];env.s.a.reconciled_peer_publication(&row.spec,row.proof.as_ref().unwrap(),owner.lease(),&leased,"driver-plan-2",crate::assistant_memory::now()).unwrap_or_else(|e|panic!("resumed canonical publication provenance: {e:?}"));
  Authority::existing(&env.s.f.path.join("authority")).unwrap().planning_ledger(&row.spec,row.proof.as_ref().unwrap(),owner.lease(),crate::assistant_memory::now()).unwrap_or_else(|e|panic!("resumed planning ledger: {e:?}"));
  Arc::new(current_store(&env.s)).read_control(&leased.id,&leased.spec.owner.pet_id).expect("resumed current goal from disk");
  let runtime=Arc::new(ResumedPeerEnv{s:env.s.clone(),provider,url:server.url.clone(),decisions:AtomicUsize::new(0)});
  let driver=Driver::new(owner.clone(),Arc::new(current_store(&env.s)),runtime.clone()).unwrap();let result=run(driver.run());if !result.as_ref().is_ok_and(|s|s.phase==Phase::AwaitingVerification){retain_unexpected_driver(&env.s,&leased,&driver,"peer-first-handoff");}let status=result.unwrap_or_else(|e|panic!("resumed driver: {e:?}; {:?}",driver.status()));assert_eq!(status.phase,Phase::AwaitingVerification,"{status:?}");assert_eq!(status.round,2,"{status:?}");assert_eq!(runtime.decisions.load(Ordering::Acquire),1);
  let done=current_store(&env.s).read(&closed.id,&closed.spec.owner.pet_id).unwrap();assert!(done.completion.is_none());let final_body=body(&env.s,&done);
  assert_eq!(final_body["planningUsage"],before["planningUsage"]);assert_eq!(final_body["delegationReservations"],before["delegationReservations"]);
  let original=before["budgetUsage"]["attempts"].as_array().unwrap();let attempts=final_body["budgetUsage"]["attempts"].as_array().unwrap();assert_eq!(attempts.len(),original.len()+1);assert_eq!(&attempts[..original.len()],original.as_slice());assert_eq!(attempts.last().unwrap()["kind"],"tool");
  let original_messages=before["messages"]["entries"].as_array().unwrap();let messages=final_body["messages"]["entries"].as_array().unwrap();assert_eq!(messages.len(),original_messages.len()+1);assert_eq!(&messages[..original_messages.len()],original_messages.as_slice());
  assert_eq!(owner.workers(),0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
 }
}
#[test]fn peer_owner_handoff_keeps_review_and_resources_separate_and_stop_still_reaches_original_children(){
 let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let old=env.s.a.recovery_body(&closed).unwrap();let before=env.s.bytes(&closed);let server=resume_server("ollama",&old);
 crate::pet_runtime::recovery_api::fixture_peer_cache_scope(resume_review(&env,&closed,"scoped-native-cache-handoff"),crate::assistant_memory::now());assert_eq!(env.s.bytes(&closed),before);
 assert!(resume_review(&env,&closed,"declined-native-handoff").approve(crate::assistant_memory::now(),Arc::new(||false)).is_err());assert_eq!(env.s.bytes(&closed),before);
 let review=resume_review(&env,&closed,"reviewed-native-handoff-with-resource-wait");let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
 let blocker=env.s.main.runtime_resources().acquire("ollama",Arc::new(AtomicBool::new(false)),None).unwrap();
 assert_eq!(approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&closed,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()),Err(model::Error::Busy));assert_eq!(env.s.bytes(&closed),before);drop(blocker);
 approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&closed,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
 let(receipt,owner)=approved.confirm(crate::assistant_memory::now()).unwrap();let leased=receipt.record;let owner=owner.unwrap();let grant=body(&env.s,&leased);let child=old.delegation_reservations[0].binding().unwrap();
 let cancelled=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&leased.id,&leased.spec.owner.pet_id,&leased.revision,"stop-new-owner-and-original-child".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;
 assert_eq!(cancelled.state,State::Stopping);assert!(!owner.policy_current(crate::assistant_memory::now()));assert_eq!(env.store.read(&child.goal_id,&child.pet_id).unwrap().state,State::Cancelled);
 let stopped=body(&env.s,&cancelled);assert_eq!(stopped["budgetUsage"],grant["budgetUsage"]);assert_eq!(stopped["planningUsage"],grant["planningUsage"]);assert_eq!(stopped["ownerResumptions"],grant["ownerResumptions"]);
 let drained=owner.drain(crate::pet_runtime::leases::DrainOutcome::Confirmed).unwrap();let cancelled=env.store.drain_owned(&env.s.a,&drained,crate::assistant_memory::now()).unwrap();assert_eq!(cancelled.record.state,State::Cancelled);assert_ne!(cancelled.status,model::CommitStatus::Uncertain);owner.live.hold.release_drained(&drained).unwrap();assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
}

#[test]fn peer_owner_handoff_ack_loss_never_activates_and_stop_can_supersede_the_pending_publication(){
 use crate::pet_runtime::store::Fault;
 for before_publish in[true,false]{for operation in["ack","cancel","pause"]{
  let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let old=env.s.a.recovery_body(&closed).unwrap();let before=body(&env.s,&closed);let server=resume_server("ollama",&old);
  let review=resume_review(&env,&closed,"exact-handoff-with-unknown-goal-ack");let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
  approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&closed,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
  Arc::get_mut(&mut approved.source.store).unwrap().fault=if before_publish{Fault::BeforePublish}else{Fault::AfterPublish};
  let result=approved.confirm(crate::assistant_memory::now());
  if before_publish{assert!(matches!(result,Err(model::Error::StorageUnavailable)));}else{let(receipt,owner)=result.unwrap();assert_eq!(receipt.status,model::CommitStatus::Uncertain);assert!(owner.is_none());}
  Arc::get_mut(&mut approved.source.store).unwrap().fault=Fault::None;
  let observed=env.store.read_control(&closed.id,&closed.spec.owner.pet_id).unwrap();let staged=body(&env.s,&observed);
  assert_eq!(staged["phase"],"claimed");assert_eq!(observed.state,if before_publish{State::RecoveryRequired}else{State::Leased});
  for field in["budgetUsage","planningUsage","delegationPool","delegationReservations","messages","lease"]{assert_eq!(staged[field],before[field]);}
  if operation=="ack"{
   // Lose the native reviewer. Cold ACK only publishes authenticated DATA;
   // it cannot manufacture an Owned or consume/replenish any allowance.
   drop(approved);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
   let result=env.store.ack_peer_owner_resumption(&env.s.f.path.join("authority"),&observed.id,&observed.spec.owner.pet_id,&observed.revision).unwrap();
   assert!(result.is_some());let ack=env.store.read_control(&closed.id,&closed.spec.owner.pet_id).unwrap();assert_eq!(ack.state,State::Leased);
   let bytes=env.s.bytes(&ack);assert_eq!(env.store.ack_peer_owner_resumption(&env.s.f.path.join("authority"),&ack.id,&ack.spec.owner.pet_id,&ack.revision).unwrap().unwrap().status,model::CommitStatus::Unchanged);assert_eq!(env.s.bytes(&ack),bytes);
   for field in["budgetUsage","planningUsage","delegationReservations","messages"]{assert_eq!(body(&env.s,&ack)[field],before[field]);}
   assert!(resume_review_from_current(&env,&ack).is_err());
  }else{
   let action=if operation=="cancel"{Action::Cancel}else{Action::Pause};let request=format!("{operation}-while-owner-ack-is-unknown");
   let receipt=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,request.clone(),action.clone(),crate::assistant_memory::now()).unwrap();
   assert_ne!(receipt.status,model::CommitStatus::Uncertain);let stopped=receipt.record;assert_eq!(stopped.state,State::Stopping);assert_eq!(stopped.sequence,closed.sequence+2);assert!(approved.confirm(crate::assistant_memory::now()).is_err());
   let grant=body(&env.s,&stopped);assert_eq!(grant["phase"],"draining");assert_eq!(grant["stopReason"],operation);assert_eq!(grant["budgetUsage"],before["budgetUsage"]);assert_eq!(grant["planningUsage"],before["planningUsage"]);assert_eq!(grant["messages"],before["messages"]);
   let child=old.delegation_reservations[0].binding().unwrap();assert_eq!(env.store.read(&child.goal_id,&child.pet_id).unwrap().state,if operation=="cancel"{State::Cancelled}else{State::Paused});
   let bytes=env.s.bytes(&stopped);assert_eq!(crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,request,action,crate::assistant_memory::now()).unwrap().record.revision,stopped.revision);assert_eq!(env.s.bytes(&stopped),bytes);
   assert!(!env.s.main.runtime_resources().busy().unwrap().is_empty());let done=crate::pet_runtime::lease_api::fixture_resumption_cleanup(&env.store,&env.s.f.path.join("authority"),&stopped,approved.pending().unwrap(),crate::assistant_memory::now()).unwrap();assert_eq!(done.record.state,if operation=="cancel"{State::Cancelled}else{State::Paused});assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
  }
 }}
}
#[test]fn peer_owner_handoff_survives_another_real_process_crash_before_goal_ack_without_replaying_the_model(){
 for phase in["resume-ack-loss","resume-grant-loss","resume-grant-loss-stop"]{
 let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let before=body(&env.s,&closed);env.g=closed.clone();
 drop(env.claim.take());env.connection.release_name(&format!("{PREFIX}.SingleInstance")).unwrap();
 let ready=env._parent.path.join("resumed-ready.json");let log=env._parent.path.join("resumed-child.log");let helper=concat!(module_path!(),"::peer_process_owner_child").split_once("::").unwrap().1;
 let child=ChildGuard(Command::new(std::env::current_exe().unwrap()).args(["--exact",helper,"--ignored","--nocapture"])
  .env("COUCOU_PEER_RECOVERY_PROFILE",&env.s.f.path).env("COUCOU_PEER_RECOVERY_BUS",&env.bus.address).env("COUCOU_PEER_RECOVERY_READY",&ready).env("COUCOU_PEER_RECOVERY_PROVIDER","ollama").env("COUCOU_PEER_RECOVERY_MODE",phase).env("COUCOU_PEER_RECOVERY_EXISTING_GOAL",&closed.id).env("COUCOU_PEER_RECOVERY_EXISTING_PET",&closed.spec.owner.pet_id)
  .stdout(Stdio::from(fs::OpenOptions::new().create(true).append(true).open(&log).unwrap())).stderr(Stdio::from(fs::OpenOptions::new().append(true).open(&log).unwrap())).spawn().unwrap());
 env.child=child;let until=Instant::now()+Duration::from_secs(20);while !ready.exists(){assert!(Instant::now()<until&&env.child.0.try_wait().unwrap().is_none(),"resumed process readiness: {}",fs::read_to_string(&log).unwrap());std::thread::sleep(Duration::from_millis(10));}
 let second_closed=if phase=="resume-ack-loss"{
  assert_ne!(body(&env.s,&closed)["runtimeOwner"]["process"]["pid"],std::process::id());assert_eq!(env.observation(),OwnerState::OriginalProcessLive);
  env.crash();let ack=env.store.ack_peer_owner_resumption(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision).unwrap().unwrap();assert_ne!(ack.status,model::CommitStatus::Uncertain);env.g=ack.record;assert_eq!(env.g.state,State::Leased);env.close()
 }else{
  assert_eq!(body(&env.s,&closed),before);env.child.0.kill().unwrap();env.child.0.wait().unwrap();
  env.connection.request_name(&format!("{PREFIX}.SingleInstance"),false,false,true).unwrap();env.claim=Some(Box::new(crate::pet_data::fixture_native_claim(&env.bus.address,PREFIX,&env.s.f.path).unwrap()));
  assert!(env.store.retire_lost_peer_owner_preparations(&env.s.f.path.join("authority"),&closed.id,"agent_codex",&closed.revision).is_err());assert!(env.store.retire_lost_peer_owner_preparations(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&"a".repeat(32)).is_err());
  if phase=="resume-grant-loss-stop"{
   let stopped=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,"cancel-unpublished-handoff-after-real-process-loss".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;assert_eq!(stopped.state,State::Cancelled);let kept=body(&env.s,&stopped);
   assert_eq!(env.store.retire_lost_peer_owner_preparations(&env.s.f.path.join("authority"),&stopped.id,&stopped.spec.owner.pet_id,&stopped.revision).unwrap(),1);assert_eq!(body(&env.s,&stopped),kept);assert_eq!(kept["budgetUsage"],before["budgetUsage"]);assert_eq!(env.s.a.history(&stopped,crate::assistant_memory::now()).unwrap().pending_artifacts,0);continue;
  }
  assert_eq!(env.store.retire_lost_peer_owner_preparations(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision).unwrap(),1);assert_eq!(env.store.retire_lost_peer_owner_preparations(&env.s.f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision).unwrap(),0);assert_eq!(body(&env.s,&closed),before);assert_eq!(env.s.a.history(&closed,crate::assistant_memory::now()).unwrap().pending_artifacts,0);closed.clone()
 };
 let grant=body(&env.s,&second_closed);assert_eq!(grant.get("ownerResumptions").and_then(|v|v.as_array()).map_or(0,Vec::len),if phase=="resume-ack-loss"{1}else{0});for field in["budgetUsage","planningUsage","delegationReservations","messages","lease"]{assert_eq!(grant[field],before[field]);}
 let old=env.s.a.recovery_body(&second_closed).unwrap();let server=resume_server("ollama",&old);let mut approved=resume_review(&env,&second_closed,"review-third-native-owner").approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();
 approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&second_closed,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
 let(receipt,owner)=approved.confirm(crate::assistant_memory::now()).unwrap();let goal=receipt.record;let owner=Arc::new(owner.unwrap());assert_eq!(body(&env.s,&goal)["ownerResumptions"].as_array().unwrap().len(),if phase=="resume-ack-loss"{2}else{1});
 let runtime=Arc::new(ResumedPeerEnv{s:env.s.clone(),provider:"ollama",url:server.url.clone(),decisions:AtomicUsize::new(0)});let driver=Driver::new(owner.clone(),Arc::new(current_store(&env.s)),runtime.clone()).unwrap();let result=run(driver.run());if !result.as_ref().is_ok_and(|s|s.phase==Phase::AwaitingVerification){retain_unexpected_driver(&env.s,&second_closed,&driver,"peer-second-handoff");}let status=result.unwrap_or_else(|e|panic!("second resumed driver: {e:?}; {:?}",driver.status()));assert_eq!(status.phase,Phase::AwaitingVerification,"{status:?}");assert_eq!(status.round,2,"{status:?}");assert_eq!(runtime.decisions.load(Ordering::Acquire),1);
 let done=env.store.read(&goal.id,&goal.spec.owner.pet_id).unwrap();let after=body(&env.s,&done);assert!(done.completion.is_none());assert_eq!(after["planningUsage"],before["planningUsage"]);assert_eq!(after["delegationReservations"],before["delegationReservations"]);assert_eq!(after["messages"]["entries"].as_array().unwrap().len(),before["messages"]["entries"].as_array().unwrap().len()+1);assert_eq!(owner.workers(),0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
}}
#[test]fn peer_owner_handoff_grant_publication_failure_keeps_the_exact_native_choice_and_cleans_only_its_preparation(){
 use crate::pet_runtime::authority_store::Fault;
 for canonical in[false,true]{for withdraw in[false,true]{
  let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let old=env.s.a.recovery_body(&closed).unwrap();let before=body(&env.s,&closed);let server=resume_server("ollama",&old);
  let frame=Arc::new(AtomicBool::new(true));let checked=frame.clone();let mut approved=resume_review(&env,&closed,"native-handoff-authority-publication").approve(crate::assistant_memory::now(),Arc::new(move||checked.load(Ordering::Acquire))).unwrap();
  approved.prepare_worker(&env.s.main,&env.s.f.path.join("contexts"),&resumed_evidence(&env.s,&closed,&server,crate::assistant_memory::now()),Arc::new(|_,_,_|Validity::Current),crate::assistant_memory::now()).unwrap();
  let mut writer=Authority::existing(&env.s.f.path.join("authority")).unwrap();writer.fault=if canonical{Fault::AfterGrantPublish}else{Fault::BeforeGrantPublish};assert!(writer.resume_peer_owner(&mut approved,crate::assistant_memory::now()).is_err());
  assert_eq!(approved.published().unwrap(),canonical);assert_eq!(env.store.read_control(&closed.id,&closed.spec.owner.pet_id).unwrap().revision,closed.revision);
  if withdraw{
   frame.store(false,Ordering::Release);
   if !canonical{assert!(approved.confirm(crate::assistant_memory::now()).is_err());assert!(approved.withdraw().unwrap());drop(approved);assert_eq!(body(&env.s,&closed),before);assert_eq!(env.s.a.history(&closed,crate::assistant_memory::now()).unwrap().pending_artifacts,0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());continue;}
   assert!(!approved.withdraw().unwrap());
  }
  let(receipt,owner)=approved.confirm(crate::assistant_memory::now()).unwrap();assert_ne!(receipt.status,model::CommitStatus::Uncertain);let goal=receipt.record;let owner=owner.unwrap();assert!(owner.current(crate::assistant_memory::now()));
  assert_eq!(env.s.a.history(&goal,crate::assistant_memory::now()).unwrap().pending_artifacts,0);
  let grant=body(&env.s,&goal);assert_ne!(grant["ownerResumptions"][0]["capabilityRevision"],before["lease"]["capabilityRevision"]);
  for field in["budgetUsage","planningUsage","delegationPool","delegationReservations","messages","lease"]{assert_eq!(grant[field],before[field]);}
  crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&goal.id,&goal.spec.owner.pet_id,&goal.revision,"stop-after-exact-grant-ack".into(),Action::Cancel,crate::assistant_memory::now()).unwrap();
  let drain=owner.drain(crate::pet_runtime::leases::DrainOutcome::Confirmed).unwrap();assert_eq!(env.store.drain_owned(&env.s.a,&drain,crate::assistant_memory::now()).unwrap().record.state,State::Cancelled);owner.live.hold.release_drained(&drain).unwrap();assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
 }}
}
fn resumed_evidence(s:&Setup,g:&Record,server:&crate::local_chat::Server,at:u64)->crate::pet_runtime::admission::OwnerResumptionEvidence{
 let mut d=crate::pet_runtime::admission_tests::descriptor(g,at);d.revision=format!("{}:1",nonce().unwrap());
 // No inference readiness or current model catalog after process loss. The
 // original engine instance remains signed; this observation cannot admit IO.
 for c in &mut d.capabilities{c.availability=crate::pet_capabilities::Availability::Unknown;c.observed_at=None;c.expires_at=None;}
 let capture=crate::pet_runtime::memory_review::Captured::at_with_authority(&s.f.path.join("notes.json"),Some(&s.f.path.join("authority")),g,server,"fixture-free",at).unwrap();
 let passive=crate::pet_runtime::admission::OwnerResumptionEvidence::observe(g,&d,"a".repeat(64),at,capture).unwrap();
 assert!(Evidence::observe(g,&d,"a".repeat(64),at,false,false).is_err(),"passive configuration must not authorize inference");
 assert!(passive.for_owner_resumption(g,at+15_000).is_err(),"a retained capture cannot renew itself");
 let refusal=|descriptor:&crate::pet_capabilities::Descriptor,policy:String|crate::pet_runtime::admission::OwnerResumptionEvidence::observe(g,descriptor,policy,at,None).err();
 let mut wrong=d.clone();wrong.pet_id="agent_codex".into();assert_eq!(refusal(&wrong,"a".repeat(64)),Some(Denial::OwnerChanged));
 let mut wrong=d.clone();wrong.binding.model=Some("another-explicit-model".into());assert_eq!(refusal(&wrong,"a".repeat(64)),Some(Denial::OwnerChanged));
 assert_eq!(refusal(&d,"b".repeat(64)),Some(Denial::PolicyChanged));
 let mut wrong=d.clone();wrong.configuration_available=false;assert_eq!(refusal(&wrong,"a".repeat(64)),Some(Denial::Inactive));
 if let Some(binding)=&g.admissions[0].spec.engine_binding{
  let mut other=binding.clone();other.model_hash=if binding.model_hash=="b".repeat(64){"c".repeat(64)}else{"b".repeat(64)};
  let mut wrong=d.clone();wrong.binding.engine_binding=Some(other);assert_eq!(refusal(&wrong,"a".repeat(64)),Some(Denial::OwnerChanged));
 }
 passive
}
fn resume_review_from_current(env:&Process,goal:&Record)->model::Result<crate::pet_runtime::continuation::CapturedPeerPlanning>{
 Arc::new(current_store(&env.s)).prepare_peer_planning_continuation(&env.s.f.path.join("authority"),&goal.id,&goal.spec.owner.pet_id,&goal.revision,crate::assistant_memory::now())
}

#[test]fn cancelling_recovered_parent_still_blocks_the_old_bound_child_without_freeing_bound_credit(){
 let mut env=Process::new("ollama","bound");let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),env.info["childPetId"].as_str().unwrap()).unwrap();env.crash();let closed=env.close();let before=body(&env.s,&closed);
 let cancelled=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,"ai_ollama",&closed.revision,"cancel-recovered-parent".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;
 assert_eq!(cancelled.state,State::Cancelled);let after=body(&env.s,&cancelled);assert_eq!(after["recovery"],before["recovery"]);assert_eq!(after["stopReason"],"cancel");assert_eq!(after["phase"],"revoked");
 let reservations=after["delegationReservations"].as_array().unwrap();assert!(reservations.iter().any(|r|r["state"]["phase"]=="cancelRequested"));assert!(reservations.iter().any(|r|r["state"]["phase"]=="withdrawn"));
 let view=env.s.a.delegation_view(&cancelled,&child.admissions[0].spec.funding.as_ref().unwrap().source.reference.decision_id,crate::assistant_memory::now()).unwrap();assert_eq!(view.held.unwrap().model_calls,1);
 let now=crate::assistant_memory::now();let result=Controller::new().unwrap().prepare(&child,&env.s.f.path.join("authority"),&env.s.f.path.join("contexts"),&env.s.main,&crate::pet_runtime::deterministic_runtime_tests::evidence(&child,now),"refuse-bound-child-after-parent-cancel".into(),now,Arc::new(|_,_,_|Validity::Current));
 assert!(result.is_err());assert_eq!(env.store.read(&child.id,"ai_lmstudio").unwrap().state,State::Cancelled);
 let bytes=env.s.bytes(&cancelled);let replay=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,"ai_ollama",&closed.revision,"cancel-recovered-parent".into(),Action::Cancel,crate::assistant_memory::now()).unwrap();assert_eq!(replay.record.revision,cancelled.revision);assert_eq!(env.s.bytes(&cancelled),bytes);
}

#[test]fn actual_process_loss_preserves_partial_derived_publication_and_shared_escrow_on_both_protocols(){
 for provider in["ollama","lmstudio"]{
  let mut env=Process::new(provider,"partial");let before=body(&env.s,&env.g);assert_eq!(before["messages"]["entries"].as_array().unwrap().len(),2);assert_eq!(env.g.generated_plans.len(),2);
  env.crash();let closed=env.close();let after=body(&env.s,&closed);
  for k in["budgetUsage","planningUsage","resourceUsage","messages","delegationReservations","lease","runtimeOwner"]{assert_eq!(after.get(k),before.get(k),"{k}");}
  assert_eq!(closed.spec,env.g.spec);assert_eq!(closed.execution_context,env.g.execution_context);assert_eq!(model::fingerprint(&closed.plans).unwrap(),model::fingerprint(&env.g.plans).unwrap());assert_eq!(closed.generated_plans,env.g.generated_plans);
  assert_eq!(after["phase"],"closed");assert!(after["recovery"].is_object());assert_eq!(after["planningUsage"]["rounds"][1]["publication"]["phase"],"acknowledged");
 }
}

#[test]fn pause_after_parent_recovery_retains_both_reservations_and_blocks_the_previously_bound_child(){
 let mut env=Process::new("ollama","bound");let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),env.info["childPetId"].as_str().unwrap()).unwrap();env.crash();let closed=env.close();let before=body(&env.s,&closed);
 let paused=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,"ai_ollama",&closed.revision,"pause-recovered-parent".into(),Action::Pause,crate::assistant_memory::now()).unwrap().record;
 assert_eq!(paused.state,State::Paused);assert!(env.store.mutate(&paused.id,"ai_ollama",&paused.revision,"no-raw-draft-reopen-after-recovered-pause".into(),Action::ResumeDraft,crate::assistant_memory::now()).is_err());let after=body(&env.s,&paused);assert_eq!(after["recovery"],before["recovery"]);assert_eq!(after["stopReason"],"pause");assert_eq!(after["delegationReservations"],before["delegationReservations"]);
 let now=crate::assistant_memory::now();assert!(Controller::new().unwrap().prepare(&child,&env.s.f.path.join("authority"),&env.s.f.path.join("contexts"),&env.s.main,&crate::pet_runtime::deterministic_runtime_tests::evidence(&child,now),"refuse-bound-child-after-parent-pause".into(),now,Arc::new(|_,_,_|Validity::Current)).is_err());assert_eq!(env.store.read(&child.id,"ai_lmstudio").unwrap().state,State::Paused);
}

#[test]fn cancel_after_recovery_of_an_iterative_peer_parent_keeps_its_planning_schema_and_original_closure(){
 let mut env=Process::new("ollama","partial");env.crash();let closed=env.close();let before=body(&env.s,&closed);
 let cancelled=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,"ai_ollama",&closed.revision,"cancel-recovered-iterative-parent".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;
 assert_eq!(cancelled.state,State::Cancelled);let after=body(&env.s,&cancelled);assert_eq!(after["schemaVersion"],43);assert_eq!(after["recovery"],before["recovery"]);assert_eq!(after["planningUsage"],before["planningUsage"]);assert_eq!(after["budgetUsage"],before["budgetUsage"]);assert_eq!(after["messages"],before["messages"]);assert_eq!(after["stopReason"],"cancel");
 assert!(after["delegationReservations"].as_array().unwrap().iter().all(|r|r["state"]["phase"]=="cancelRequested"));
 let bytes=env.s.bytes(&cancelled);let replay=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&closed.id,"ai_ollama",&closed.revision,"cancel-recovered-iterative-parent".into(),Action::Cancel,crate::assistant_memory::now()).unwrap();assert_eq!(replay.record.revision,cancelled.revision);assert_eq!(env.s.bytes(&cancelled),bytes);
}

#[test]fn actual_process_loss_preserves_unknown_second_model_io_without_publication_or_budget_refund(){
 for provider in["ollama","lmstudio"]{
  let mut env=Process::new(provider,"uncertain");let before=body(&env.s,&env.g);assert_eq!(env.g.generated_plans.len(),1);assert_eq!(before["messages"]["entries"].as_array().unwrap().len(),1);
  let second=before["budgetUsage"]["attempts"].as_array().unwrap().iter().filter(|a|a["kind"]=="model").nth(1).unwrap();assert_eq!(second["phase"],"uncertain");assert!(second["acceptedOutput"].is_null());
  let snap=env.s.a.reconciliation(&env.g,crate::assistant_memory::now()).unwrap();assert_eq!(snap.accounting.unwrap().unknown_attempts,1);
  env.crash();let closed=env.close();let after=body(&env.s,&closed);
  for k in["budgetUsage","planningUsage","resourceUsage","messages","delegationReservations","lease","runtimeOwner"]{assert_eq!(after.get(k),before.get(k),"{k}");}
  assert_eq!(closed.generated_plans,env.g.generated_plans);assert_eq!(model::fingerprint(&closed.plans).unwrap(),model::fingerprint(&env.g.plans).unwrap());let snap=env.s.a.reconciliation(&closed,crate::assistant_memory::now()).unwrap();let accounting=snap.accounting.unwrap();assert_eq!(accounting.unknown_attempts,1);assert_eq!(accounting.model_calls_remaining,0);assert!(!snap.goal_complete);
 }
}

#[test]fn bound_inference_after_parent_loss_keeps_original_f2_endpoint_budget_and_canonical_result_on_both_protocols(){
 for provider in["ollama","lmstudio"]{
  let meta=if provider=="lmstudio"{lm_metadata()}else{metadata()};
  let(mut server,http)=http_fixture(vec![("application/json",meta.to_string()),("text/event-stream",stream("Independent bounded result; never an approval or completed criterion",true))],|_|{});server.id=provider;
  let mut env=Process::with_endpoint(provider,"bound-inference",Some(&server));
  let pet=env.info["childPetId"].as_str().unwrap().to_owned();let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),&pet).unwrap();let original=child.admissions[0].spec.clone();
  assert_eq!(original.budget.max_model_calls,1);assert_eq!(original.budget.max_tool_calls,0);assert_eq!(original.budget.max_output_bytes,4096);
  assert_eq!(original.memory_data.as_ref().unwrap().destination,crate::pet_memory::context_model::Destination::Server{provider:provider.into(),endpoint:format!("{}/v1/chat/completions",server.url),model:"fixture-free".into()});
  env.crash();let parent=env.close();let parent_bytes=env.s.bytes(&parent);let parent_before=body(&env.s,&parent);
  let inbox=env.s.a.message_inbox(&pet,None,8,crate::assistant_memory::now()).unwrap();let pending=&inbox.items.iter().find(|i|i.receipt.envelope().message_id==env.info["pendingMessageId"]).unwrap().receipt;
  assert!(data_review(&env.s,pending,"cannot-admit-new-child-after-parent-loss").is_err());assert_eq!(env.s.bytes(&parent),parent_bytes);
  let(leased,owner)=own(&env.s,&child,&server);let driver=Driver::new(owner.clone(),Arc::new(current_store(&env.s)),environment(&env.s,&server)).unwrap();
  assert_eq!(run(driver.run()).unwrap().phase,Phase::AwaitingVerification);let done=env.store.read(&child.id,&pet).unwrap();
  assert_eq!(done.state,State::Paused);assert!(done.completion.is_none());assert_eq!(model::fingerprint(&done.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());
  let requests=http.join().unwrap();assert_eq!(requests.len(),2);let wire=&requests[1].1;let grant=body(&env.s,&done);let attempts=grant["budgetUsage"]["attempts"].as_array().unwrap();assert_eq!(attempts.len(),1);assert_eq!(attempts[0]["phase"],"settled");assert_eq!(attempts[0]["kind"],"model");assert_eq!(attempts[0]["inputHash"],model::fingerprint(&(&owner.lease().id,"fixture-free",wire)).unwrap());
  assert!(wire["messages"].to_string().contains("receivedDelegationData"));assert_eq!(env.s.bytes(&parent),parent_bytes);assert_eq!(owner.workers(),0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
  let reference=&original.funding.as_ref().unwrap().source.reference;
  let view=Authority::existing(&env.s.f.path.join("authority")).unwrap().prepare_delegation_branch(parent.clone(),&reference.decision_id,&reference.message_id,crate::pet_runtime::delegation::BranchAction::Reconcile,crate::assistant_memory::now()).unwrap().confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap().view;
  assert_eq!(view.consumed.as_ref().unwrap().model_calls,1);assert_eq!(view.consumed.as_ref().unwrap().tool_calls,0);assert_eq!(view.unknown.as_ref().unwrap().model_calls,0);assert_eq!(view.held.as_ref().unwrap().model_calls,1);
  let parent_after=body(&env.s,&parent);assert_eq!(parent_after["recovery"],parent_before["recovery"]);assert_eq!(parent_after["budgetUsage"],parent_before["budgetUsage"]);
  let ack=run(owner.respond_once(None,&leased,&server,&format!("response-{}",original.decision_id),&env.s.config)).unwrap();assert_eq!(ack,"Independent bounded result; never an approval or completed criterion");assert_eq!(body(&env.s,&parent),parent_after);
 }
}

#[test]fn bound_inference_after_parent_recovery_cancel_during_io_retains_unknown_charge_without_success_or_replay(){
 for provider in["ollama","lmstudio"]{
  let cancel:Arc<Mutex<Option<(PathBuf,Record)>>>=Arc::new(Mutex::new(None));let captured=cancel.clone();let partial=Arc::new(AtomicBool::new(false));let pending=partial.clone();let meta=if provider=="lmstudio"{lm_metadata()}else{metadata()};
  let(mut server,http)=crate::pet_runtime::lease_controller::memory_test_http_fixture_allow_disconnect(vec![("application/json",meta.to_string()),("text/event-stream",stream("This cancelled output must not become an accepted result",true))],move|url|{
   if url.contains("chat/completions"){
    let(path,parent)=captured.lock().unwrap().clone().unwrap();let store=crate::pet_runtime::store::Store::existing_verified(&path.join("goals"),&path.join("authority")).unwrap().unwrap();
    let at=crate::assistant_memory::now();match crate::pet_runtime::authority_store::mutate_with_authority(&store,&path.join("authority"),&parent.id,&parent.spec.owner.pet_id,&parent.revision,"cancel-recovered-parent-during-child-io".into(),Action::Cancel,at){
     Ok(stopped)=>assert_eq!(stopped.record.state,State::Cancelled),
     Err(model::Error::Busy|model::Error::Conflict)=>{
      // Keep the original control unresolved until signed readback below.
      // Busy is neither approval nor evidence that cancellation completed.
      pending.store(true,Ordering::Release);
     },Err(e)=>panic!("native Stop failed before a recorded reduction: {e:?}"),
    }
   }
  });server.id=provider;
  let mut env=Process::with_endpoint(provider,"bound-inference",Some(&server));let pet=env.info["childPetId"].as_str().unwrap().to_owned();let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),&pet).unwrap();let original=child.admissions[0].spec.clone();
  env.crash();let parent=env.close();let original_close=body(&env.s,&parent)["recovery"].clone();*cancel.lock().unwrap()=Some((env.s.f.path.clone(),parent.clone()));
  let(leased,owner)=own(&env.s,&child,&server);let driver=Driver::new(owner.clone(),Arc::new(current_store(&env.s)),environment(&env.s,&server)).unwrap();let first=run(driver.run());
  assert_eq!(http.join().unwrap().len(),2);
  if partial.load(Ordering::Acquire){
   let until=Instant::now()+Duration::from_millis(500);let signed=loop{match env.s.a.recovery_body(&parent){Ok(body)=>break body,Err(crate::pet_runtime::authority::Failure::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(1)),Err(e)=>panic!("recorded Stop unavailable: {e:?}")}};
   // Assert the original callback's durable reduction BEFORE retrying its ACK.
   // Thus a failed admission/changed target can never be hidden by a new Stop.
   assert_eq!(signed.decision_id,parent.admissions[0].spec.decision_id);assert_eq!(signed.phase,crate::pet_runtime::authority::GrantPhase::Revoked);assert_eq!(signed.stop_reason,Some(crate::pet_runtime::leases::StopReason::Cancel));
   let acknowledged=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&parent.id,&parent.spec.owner.pet_id,&parent.revision,"cancel-recovered-parent-during-child-io".into(),Action::Cancel,crate::assistant_memory::now()).unwrap();assert_eq!(acknowledged.record.state,State::Cancelled);
  }
  let stopped=match first{Ok(status)=>status,Err(model::Error::Busy)=>run(driver.run()).unwrap(),Err(e)=>panic!("native cancelled driver: {e:?}")};assert_eq!(stopped.phase,Phase::Stopped);
  let done=env.store.read(&child.id,&pet).unwrap();assert_eq!(done.state,State::Cancelled);assert!(done.completion.is_none());assert_eq!(model::fingerprint(&done.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());
  let grant=body(&env.s,&done);let attempts=grant["budgetUsage"]["attempts"].as_array().unwrap();assert_eq!(attempts.len(),1);assert_eq!(attempts[0]["phase"],"uncertain");assert!(attempts[0]["acceptedOutput"].is_null());assert_eq!(owner.workers(),0);assert!(env.s.main.runtime_resources().busy().unwrap().is_empty());
  let current_parent=env.store.read(&parent.id,&parent.spec.owner.pet_id).unwrap();assert_eq!(current_parent.state,State::Cancelled);let reference=&original.funding.as_ref().unwrap().source.reference;
  let view=Authority::existing(&env.s.f.path.join("authority")).unwrap().prepare_delegation_branch(current_parent,&reference.decision_id,&reference.message_id,crate::pet_runtime::delegation::BranchAction::Reconcile,crate::assistant_memory::now()).unwrap().confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap().view;
  assert_eq!(view.consumed.as_ref().unwrap().model_calls,0);assert_eq!(view.unknown.as_ref().unwrap().model_calls,1);assert_eq!(view.unknown.as_ref().unwrap().output_bytes,4096);assert_eq!(body(&env.s,&parent)["recovery"],original_close);
  let bytes=env.s.bytes(&done);assert!(run(owner.respond_once(None,&leased,&server,&format!("response-{}",original.decision_id),&env.s.config)).is_err());assert_eq!(env.s.bytes(&done),bytes);
 }
}

#[test]fn bound_inference_after_recovery_recorded_stop_finishes_when_the_bound_child_has_a_later_signed_clock(){
 let server=crate::local_chat::Server{id:"ollama",name:"No inference is dispatched",url:"http://127.0.0.1:1".into(),key:None};
 let mut env=Process::with_endpoint("ollama","bound-inference",Some(&server));let child=env.store.read(env.info["childGoalId"].as_str().unwrap(),env.info["childPetId"].as_str().unwrap()).unwrap();env.crash();let parent=env.close();
 let at=crate::assistant_memory::now();let row=&parent.admissions[0];let child_row=&child.admissions[0];
 env.s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,at).unwrap();
 assert!(!env.s.a.reconciliation(&parent,at).unwrap().grant.unwrap().control_projection_current);
 // A concurrent native child has already recorded the same authority reduction
 // at a later instant. Finishing the original Stop must not reopen it or fail
 // as a stale new effect, and must never refund the bound reservation.
 env.s.a.request_stop(&child_row.spec,child_row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,at+1).unwrap();
 let targets=env.s.a.stopped_pool_descendants(&row.spec,row.proof.as_ref().unwrap(),at).unwrap();
 crate::pet_runtime::authority_store::project_stop_queues(&env.store,&env.s.f.path.join("authority"),&targets,at).unwrap();
 assert_eq!(env.store.read(&child.id,&child.spec.owner.pet_id).unwrap().state,State::Cancelled);
 let view=env.s.a.delegation_view(&parent,&row.spec.decision_id,at+2).unwrap();assert_eq!(view.held.unwrap().model_calls,1);assert_eq!(body(&env.s,&child)["phase"],"revoked");assert!(body(&env.s,&child)["budgetUsage"].is_null());
 let acknowledged=crate::pet_runtime::authority_store::mutate_with_authority(&env.store,&env.s.f.path.join("authority"),&parent.id,&parent.spec.owner.pet_id,&parent.revision,"ack-original-recorded-stop".into(),Action::Cancel,at+2).unwrap().record;
 assert_eq!(acknowledged.state,State::Cancelled);assert!(env.s.a.reconciliation(&acknowledged,at+3).unwrap().grant.unwrap().control_projection_current);
}
