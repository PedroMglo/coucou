//! Native driver, real loopback protocol and private MPRIS bus only.
use super::*;
use crate::pet_runtime::{generated_effect_tests as generated,lease_controller::dispatcher::{Driver,Environment,Capture,Status,Phase},recipe_decisions::{Source,Decision},effect_budget::{Attempt,Kind,Phase as AttemptPhase},planning_journal::Request};
use crate::spotify::goal::{Target,tests::{Bus,Player}};
use std::sync::atomic::AtomicUsize;
struct Env {address:String,url:String,allow:bool,wait:bool,asks:AtomicUsize,captures:AtomicUsize,statuses:Mutex<Vec<Status>>,registration_fault:Mutex<Option<std::path::PathBuf>>,registration_lock_path:Mutex<Option<std::path::PathBuf>>,registration_lock:Mutex<Option<std::fs::File>>,model_start:Mutex<Option<std::sync::mpsc::Sender<()>>>}
impl Env {
    fn new(bus:&Bus,url:String,allow:bool,wait:bool)->Arc<Self>{Arc::new(Self{address:bus.address.clone(),url,allow,wait,asks:AtomicUsize::new(0),captures:AtomicUsize::new(0),statuses:Mutex::new(vec![]),registration_fault:Mutex::new(None),registration_lock_path:Mutex::new(None),registration_lock:Mutex::new(None),model_start:Mutex::new(None)})}
}
impl Environment for Env {
    fn capture(&self,_:&Owned)->model::Result<Capture>{self.captures.fetch_add(1,Ordering::AcqRel);assert!(!self.url.is_empty(),"No inference or metadata is allowed on this path");if let Some(start)=self.model_start.lock().unwrap().take(){start.send(()).unwrap();}Ok(Capture{configuration:generated::config(),server:crate::local_chat::Server{id:"ollama",name:"Free synthetic fixture",url:self.url.clone(),key:None},app:None})}
    fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(generated::config())}
    fn target(&self)->model::Result<Arc<Target>>{Ok(Arc::new(Target::fixture_at(&self.address).unwrap()))}
    fn decide(&self,source:Source)->Option<Decision>{
        self.asks.fetch_add(1,Ordering::AcqRel);
        if self.wait {let deadline=Instant::now()+Duration::from_secs(2);while source.current()&&Instant::now()<deadline{thread::sleep(Duration::from_millis(5));}return None;}
        let fixture=crate::cli_chat::goal_decision_fixture(source);fixture.answer(json!({"allow":self.allow}),fixture.request["goalTarget"].clone()).unwrap();fixture.decision()
    }
    fn notify(&self,status:&Status){
        self.statuses.lock().unwrap().push(status.clone());
        if status.phase==Phase::Registering {
            if let Some(path)=self.registration_fault.lock().unwrap().take(){std::fs::rename(&path,path.with_extension("held")).unwrap();}
            if let Some(path)=self.registration_lock_path.lock().unwrap().take(){
                use std::os::fd::AsRawFd;
                let lock=std::fs::OpenOptions::new().read(true).write(true).open(path).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);*self.registration_lock.lock().unwrap()=Some(lock);
            }
        }
    }
}
struct LmStudioEnv(Arc<Env>);
impl Environment for LmStudioEnv{
 fn capture(&self,owner:&Owned)->model::Result<Capture>{assert_eq!(owner.lease().provider,"lmstudio");self.0.captures.fetch_add(1,Ordering::AcqRel);Ok(Capture{configuration:lmstudio_config(),server:crate::local_chat::Server{id:"lmstudio",name:"Private LM Studio fixture",url:self.0.url.clone(),key:None},app:None})}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(lmstudio_config())}
 fn target(&self)->model::Result<Arc<Target>>{self.0.target()}
 fn decide(&self,source:Source)->Option<Decision>{self.0.decide(source)}
 fn notify(&self,status:&Status){self.0.notify(status)}
}
#[test]fn lmstudio_driver_keeps_its_own_identity_approval_and_budget_through_a_verified_goal(){
 let metadata=lmstudio_metadata();let plan=crate::pet_runtime::planning_tests::output().to_string();let(mut server,http)=fixture_with_hook(vec![("application/json",metadata.to_string()),("text/event-stream",stream(&plan,true))],|_|{});server.id="lmstudio";
 let(f,_a,g,_pending,owner,main)=setup_lmstudio_planning(&server,&metadata);let bus=Bus::new();let player=Player::new(&bus,"Independent LM Studio goal");let env=Env::new(&bus,server.url.clone(),true,false);
 let driver=Driver::new(Arc::new(owner),Arc::new(generated::native(&f)),Arc::new(LmStudioEnv(env.clone()))).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Completed);
 let final_goal=generated::native(&f).read(&g.id,"ai_lmstudio").unwrap();assert_eq!(final_goal.state,State::Completed);assert_eq!(final_goal.spec.owner,g.spec.owner);assert_eq!(final_goal.admissions[0].spec.engine_binding,g.admissions[0].spec.engine_binding);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),1);assert_eq!(player.calls.lock().unwrap().len(),1);
 let requests=http.join().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[0].0,"/api/v1/models");assert_eq!(requests[1].0,"/v1/chat/completions");assert_eq!(requests[1].1["model"],"fixture-free");assert!(requests[1].1.get("tools").is_none());
 let usage=ledger(&f,&g);let attempts=usage["attempts"].as_array().unwrap();assert_eq!(attempts.iter().filter(|a|a["kind"]=="model").count(),1);assert_eq!(attempts.iter().filter(|a|a["kind"]=="tool").count(),1);assert!(attempts.iter().all(|a|a["phase"]=="settled"));
 assert!(main.runtime_resources().acquire("ollama",Arc::new(AtomicBool::new(false)),None).is_ok());assert_eq!(driver.status().unwrap().pet_id,"ai_lmstudio");assert!(!driver.status().unwrap().active);
}
#[test]fn driver_publishes_asks_registers_executes_verifies_and_closes_the_same_owner(){
    let(f,a,g,_p,o,main)=generated::setup(false);let owner=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private autonomous player");let env=Env::new(&bus,String::new(),true,false);let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env.clone()).unwrap();
    let status=run(driver.run()).unwrap();assert_eq!(status.phase,Phase::Completed);assert!(!status.active&&!status.retained_approval);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(player.calls.lock().unwrap().len(),2);let final_goal=generated::native(&f).read(&g.id,"ai_ollama").unwrap();assert_eq!(final_goal.state,State::Completed);a.verify_completed_goal(&final_goal).unwrap();assert_eq!(owner.lease(),g.admissions[0].lease.as_ref().unwrap());assert_eq!(owner.workers(),0);assert!(!owner.current(crate::assistant_memory::now()));assert!(main.runtime_resources().busy().unwrap().is_empty());
    let notices=env.statuses.lock().unwrap();for phase in [Phase::Publishing,Phase::AwaitingApproval,Phase::Registering,Phase::Executing,Phase::Feedback,Phase::Completed]{assert!(notices.iter().any(|s|s.phase==phase));}
    drop(notices);let fresh=main.runtime_resources().acquire("ollama",Arc::new(AtomicBool::new(false)),None).unwrap();assert!(fresh.current());assert_eq!(run(driver.run()).unwrap().phase,Phase::Completed);assert!(fresh.current());assert_eq!(main.runtime_resources().busy().unwrap().len(),1);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(env.asks.load(Ordering::Acquire),1);
}
#[test]fn driver_replans_from_signed_native_unmet_feedback_with_the_original_remaining_budget(){
    let(f,_a,g,_p,o,_main)=generated::setup_policy(true,18,3,2);let bus=Bus::new();let player=Player::new(&bus,"Private autonomous replan");
    let text=json!({"kind":"plan","description":"Meet the original remaining volume criterion","steps":[{"description":"Set volume","postcondition":"17 percent","operation":{"operationId":"spotify.control","version":1,"arguments":{"action":"set_volume","volume":17}}}],"criteria":[{"criterionIndex":0,"predicate":{"kind":"spotifyPlayback","status":"paused"}},{"criterionIndex":1,"predicate":{"kind":"spotifyVolume","percent":17}}]}).to_string();
    let(server,start,http)=fixture_armed(vec![("application/json",local_metadata().to_string()),("text/event-stream",stream(&text,true))]);let env=Env::new(&bus,server.url,true,false);*env.model_start.lock().unwrap()=Some(start);let driver=Driver::new(Arc::new(o),Arc::new(generated::native(&f)),env.clone()).unwrap();assert_eq!(run(driver.run()).unwrap_or_else(|e|panic!("driver: {e:?}; status: {:?}",driver.status())).phase,Phase::Completed);
    let requests=http.join().unwrap();assert_eq!(requests.len(),2);let prompt:Value=serde_json::from_str(requests[1].1["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap()).unwrap();assert_eq!(prompt["nativeContext"]["remaining"]["toolCalls"],1);assert_eq!(prompt["nativeContext"]["remaining"]["bestVerifiedCriteria"],1);assert_eq!(prompt["nativeContext"]["observations"][0]["feedback"]["known"],true);assert!(requests[1].1.get("tools").is_none());
    assert_eq!(env.asks.load(Ordering::Acquire),2);assert_eq!(env.captures.load(Ordering::Acquire),1);assert_eq!(player.calls.lock().unwrap().len(),3);let saved=generated::native(&f).read(&g.id,"ai_ollama").unwrap();assert_eq!(saved.generated_plans.len(),2);assert_eq!(saved.admissions.len(),1);assert_eq!(saved.state,State::Completed);
    let usage=ledger(&f,&g);assert_eq!(usage["attempts"].as_array().unwrap().iter().filter(|r|r["kind"]=="model").count(),2);assert_eq!(usage["attempts"].as_array().unwrap().iter().filter(|r|r["kind"]=="tool").count(),3);
}
#[test]fn driver_refusal_stops_without_effects_inference_or_automatic_renewal(){
    let(f,_a,_g,_p,o,_)=generated::setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private refused driver");let env=Env::new(&bus,String::new(),false,false);let driver=Driver::new(Arc::new(o),Arc::new(generated::native(&f)),env.clone()).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Declined);assert!(run(driver.run()).is_err());assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert!(player.calls.lock().unwrap().is_empty());
}
#[test]fn the_native_thread_survives_the_caller_and_only_one_driver_can_claim_the_original_lease(){
    let(f,_a,g,_p,o,_)=generated::setup(true);let owner=Arc::new(o);let bus=Bus::new();let player=Player::new(&bus,"Private stopped driver");let env=Env::new(&bus,String::new(),true,true);let store=Arc::new(generated::native(&f));let driver=Driver::new(owner.clone(),store.clone(),env.clone()).unwrap();let other=Driver::new(owner.clone(),store,env.clone()).unwrap();driver.spawn().unwrap();
    let until=Instant::now()+Duration::from_secs(2);while env.asks.load(Ordering::Acquire)==0&&Instant::now()<until{thread::sleep(Duration::from_millis(5));}assert_eq!(env.asks.load(Ordering::Acquire),1);assert!(driver.status().unwrap().active);assert_eq!(other.spawn().err(),Some(model::Error::Busy));let weak=Arc::downgrade(&driver);drop(driver);assert!(weak.upgrade().is_some());
    crate::pet_runtime::lease_controller::signal_stop(owner.lease());while weak.upgrade().is_some_and(|d|d.status().unwrap().active)&&Instant::now()<until{thread::sleep(Duration::from_millis(5));}assert!(!weak.upgrade().is_some_and(|d|d.status().unwrap().active));assert!(player.calls.lock().unwrap().is_empty());assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);assert!(env.statuses.lock().unwrap().iter().any(|s|s.phase==Phase::Stopped&&!s.active));
}
#[test]fn uncertain_registration_retains_the_original_native_approval_and_retry_never_reasks(){
    let(f,_a,g,_p,o,_)=generated::setup(true);let bus=Bus::new();let player=Player::new(&bus,"Private registration retry");let env=Env::new(&bus,String::new(),true,false);let path=root(&f).join(format!("{}.json",g.admissions[0].spec.decision_id));*env.registration_fault.lock().unwrap()=Some(path.clone());let driver=Driver::new(Arc::new(o),Arc::new(generated::native(&f)),env.clone()).unwrap();assert!(run(driver.run()).is_err());assert!(driver.status().unwrap().retained_approval);assert!(player.calls.lock().unwrap().is_empty());std::fs::rename(path.with_extension("held"),&path).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Completed);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]fn exhausted_tool_headroom_stops_before_metadata_or_inference_after_known_feedback(){
    let(f,_a,g,_p,o,_)=generated::setup_policy(true,18,2,2);let bus=Bus::new();let player=Player::new(&bus,"Private bounded driver");let env=Env::new(&bus,String::new(),true,false);let driver=Driver::new(Arc::new(o),Arc::new(generated::native(&f)),env.clone()).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Limited);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(generated::native(&f).read(&g.id,"ai_ollama").unwrap().state,State::RecoveryRequired);
}

#[test]fn transient_registration_contention_waits_on_the_same_native_approval_without_new_io(){
    let(f,_a,g,_p,o,_main)=generated::setup(true);let owner=Arc::new(o);let lease=owner.lease().clone();let bus=Bus::new();let player=Player::new(&bus,"Transient registration lock");let env=Env::new(&bus,String::new(),true,false);*env.registration_lock_path.lock().unwrap()=Some(root(&f).join(".catalog.lock"));let thread_env=env.clone();
    let release=thread::spawn(move||{let until=Instant::now()+Duration::from_secs(5);while thread_env.registration_lock.lock().unwrap().is_none(){assert!(Instant::now()<until);thread::sleep(Duration::from_millis(5));}thread::sleep(Duration::from_millis(150));thread_env.registration_lock.lock().unwrap().take();});
    let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env.clone()).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Completed);release.join().unwrap();assert_eq!(owner.lease(),&lease);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(player.calls.lock().unwrap().len(),2);assert!(!driver.status().unwrap().retained_approval);
    assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().iter().filter(|a|a["kind"]=="model").count(),1);
}

#[test]fn sustained_registration_contention_returns_busy_and_preserves_the_original_ready_for_explicit_retry(){
    let(f,_a,g,_p,o,_main)=generated::setup(true);let owner=Arc::new(o);let lease=owner.lease().clone();let bus=Bus::new();let player=Player::new(&bus,"Retained registration lock");let env=Env::new(&bus,String::new(),true,false);*env.registration_lock_path.lock().unwrap()=Some(root(&f).join(".catalog.lock"));let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env.clone()).unwrap();
    assert_eq!(run(driver.run()).err(),Some(model::Error::Busy));let status=driver.status().unwrap();assert!(status.retained_approval&&!status.active);assert!(status.reason.as_deref().is_some_and(|s|s.contains("Busy")));assert!(player.calls.lock().unwrap().is_empty());assert_eq!(owner.workers(),0);assert_eq!(owner.lease(),&lease);let usage=ledger(&f,&g);let original_controller=status.controller_id.clone();
    env.registration_lock.lock().unwrap().take();let completed=run(driver.run()).unwrap();assert_eq!(completed.phase,Phase::Completed);assert_eq!(completed.controller_id,original_controller);assert_eq!(completed.lease_id,lease.id);assert_eq!(owner.lease(),&lease);assert_eq!(env.asks.load(Ordering::Acquire),1);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(player.calls.lock().unwrap().len(),2);assert_eq!(usage["attempts"].as_array().unwrap().len(),1);
}
#[test]fn unknown_original_model_attempt_halts_without_replay_or_effects(){
    let(f,a,g,_p,o,_)=setup_planning();let now=crate::assistant_memory::now();let row=&g.admissions[0];let request=Request::capture(&g,&row.spec,o.lease(),"unknown-original").unwrap();let attempt=Attempt{id:nonce().unwrap(),request_id:request.request_id.clone(),input_hash:model::fingerprint(&"synthetic-unknown").unwrap(),kind:Kind::Model,phase:AttemptPhase::Reserved,output_reserved:2048,reserved_at:now,started_at:None,finished_at:None,accepted_output:None};a.reserve_planning_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&attempt,&request,now).unwrap();a.start_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&attempt,now).unwrap();
    let bus=Bus::new();let player=Player::new(&bus,"Private unknown model");let env=Env::new(&bus,String::new(),true,false);let driver=Driver::new(Arc::new(o),Arc::new(generated::native(&f)),env.clone()).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::RecoveryRequired);assert_eq!(env.asks.load(Ordering::Acquire),0);assert_eq!(env.captures.load(Ordering::Acquire),0);assert!(player.calls.lock().unwrap().is_empty());assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);
}

#[test]fn stop_during_the_actual_model_request_drains_without_retry_and_frees_the_original_resource(){
    let(f,_a,g,_p,o,main)=setup_planning();let owner=Arc::new(o);let lease=owner.lease().clone();
    let(server,http)=fixture_with_hook(vec![("application/json",local_metadata().to_string()),("text/event-stream",stream("interrupted proposal",true))],move|path|{if path=="/v1/chat/completions"{crate::pet_runtime::lease_controller::signal_stop(&lease);}});
    let bus=Bus::new();let player=Player::new(&bus,"Private model stop");let env=Env::new(&bus,server.url,true,false);let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env.clone()).unwrap();assert!(run(driver.run()).is_err());let status=driver.status().unwrap();assert_eq!(status.phase,Phase::Stopped);assert!(!status.active);assert_eq!(owner.workers(),0);assert_eq!(env.asks.load(Ordering::Acquire),0);assert_eq!(env.captures.load(Ordering::Acquire),1);assert_eq!(http.join().unwrap().len(),2);assert!(player.calls.lock().unwrap().is_empty());
    let usage=ledger(&f,&g);assert_eq!(usage["attempts"].as_array().unwrap().len(),1);assert_eq!(usage["attempts"][0]["phase"],"uncertain");assert_eq!(generated::native(&f).read(&g.id,"ai_ollama").unwrap().state,State::RecoveryRequired);let fresh=main.runtime_resources().acquire("ollama",Arc::new(AtomicBool::new(false)),None).unwrap();assert!(fresh.current());
}
