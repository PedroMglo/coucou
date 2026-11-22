//! Genuine registry TTL plus original native consent/decision/player/HTTP.
//! This protects existing behavior; it changes no production policy or routing.
use super::*;
use crate::{chat::Chat,pet_capabilities::fixture_local_projection,spotify::goal::{Target,tests::{Bus,Player}}};
use crate::pet_runtime::{admission::{Payment,Review},criteria::{Input,Predicate,Playback},model::{Action,StepInput},plans::OperationProposal,recipe_decisions::Source,recipes::Outcome,store::Store,lease_controller::signal_stop};
use std::{sync::{Mutex,atomic::{AtomicBool,Ordering}},io::{Read,Write},net::TcpListener,thread,time::{Duration,Instant}};
use serde_json::{json,Value};

struct ToolCase {f:Fixture,authority:Authority,goal:Record,_pending:Pending,owner:Arc<Owned>,config:Arc<Mutex<Configuration>>,settings:Settings,stale:Arc<AtomicBool>,at:u64,_main:Chat}
impl ToolCase {
    fn new(endpoint:&str)->Self {
        let f=Fixture::new();let authority=Authority::ensure(&root(&f)).unwrap();let at=crate::assistant_memory::now()-100;
        let mut goal=f.goal_named_criteria("ollama","tool-readiness",vec!["Final playback is paused".into(),"Final volume is 17 percent".into()]);
        goal=f.store().mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,"task".into(),Action::ProposeTask{description:"Reviewed private player task".into()},at).unwrap().record;
        let steps=[json!({"action":"pause"}),json!({"action":"set_volume","volume":17})].into_iter().map(|arguments|StepInput{description:"Native player action".into(),postcondition:"Native player state observed".into(),operation:Some(OperationProposal{operation_id:"spotify.control".into(),version:1,arguments})}).collect();
        goal=f.store().mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,"plan".into(),Action::ProposePlan{task_id:goal.tasks[0].id.clone(),steps},at+1).unwrap().record;
        let mut configuration=crate::pet_config::defaults();let pet=configuration.pets.get_mut("ai_ollama").unwrap();pet.permissions.get_or_insert_default().music=true;pet.mode="fast".into();pet.fast.model="fixture-free".into();pet.reasoning.model="fixture-free".into();
        let mut settings=Settings::default();settings.active_integrations=vec!["ai_ollama".into()];settings.main_pill="ai_ollama".into();settings.chat_models.insert("ollama".into(),"fixture-free".into());settings.ollama_url=endpoint.into();
        let observation=|g:&Record,now| {let capture=fixture_local_projection(settings.clone(),configuration.clone(),at,now);let descriptor=capture.snapshot.pets.iter().find(|d|d.pet_id==g.spec.owner.pet_id).unwrap();Evidence::observe(g,descriptor,policy_revision(&configuration,&settings,g).unwrap(),now,false,false).unwrap()};
        let mut limits=budget();limits.deadline_at=at+300_000;limits.max_output_bytes=32_768;limits.payment=Payment::LocalOnly;
        let approved=Review::prepare(&goal,observation(&goal,at+2),limits,"queue".into(),nonce().unwrap(),at+2).unwrap().approve(&goal,observation(&goal,at+3),at+3).unwrap();
        let certified=authority.certify(&approved,&goal,at+4).unwrap();let queued=f.store().enqueue_certified(&certified,at+5).unwrap().record;
        let initial=observation(&queued,at+6);let claim=observation(&queued,at+7);
        let config=Arc::new(Mutex::new(configuration));let stale=Arc::new(AtomicBool::new(false));let cfg=config.clone();let aged=stale.clone();let native_settings=settings.clone();let goals=f.path.join("goals");let authority_path=root(&f);let id=queued.id.clone();let pet_id=queued.spec.owner.pet_id.clone();
        let fence=Arc::new(move|lease:&Lease,_:&crate::pet_resources::Hold,mode:Validation<'_>| {
            let goal=if let Validation::Commit(g)=mode {g.clone()}else{let Some(store)=Store::existing_verified(&goals,&authority_path).ok().flatten()else{return Validity::Unknown;};let Ok(goal)=store.read(&id,&pet_id)else{return Validity::Unknown;};goal};
            let Ok(config)=cfg.lock().map(|c|c.clone())else{return Validity::Unknown;};let now=if aged.load(Ordering::Acquire){at+60_001}else{crate::assistant_memory::now()};let capture=fixture_local_projection(native_settings.clone(),config,at,now);evaluate(&goal,lease,&capture,mode,now)
        });
        let main=Chat::persistent(f.path.join("main-live.json"));let controller=Controller::new().unwrap();let pending=controller.prepare(&queued,&root(&f),&f.path.join("contexts"),&main,&initial,"claim".into(),at+6,fence).unwrap();let(receipt,owned)=pending.claim(&f.store(),&authority,&claim,at+7).unwrap();
        Self{f,authority,goal:receipt.record,_pending:pending,owner:Arc::new(owned.unwrap()),config,settings,stale,at,_main:main}
    }
    fn configuration(&self)->Configuration {self.config.lock().unwrap().clone()}
    fn store(&self)->Arc<Store>{Arc::new(Store::existing_verified(&self.f.path.join("goals"),&root(&self.f)).unwrap().unwrap())}
    fn review(&self,bus:&Bus)->crate::pet_runtime::lease_controller::recipe_controller::Review {
        self.owner.review_recipe_with_criteria(&self.goal,&self.goal.plans[0].id,"ttl-review",Arc::new(Target::fixture_at(&bus.address).unwrap()),&self.configuration(),&[Input{criterion_index:0,predicate:Predicate::SpotifyPlayback{status:Playback::Paused}},Input{criterion_index:1,predicate:Predicate::SpotifyVolume{percent:17}}],crate::assistant_memory::now()).unwrap()
    }
    fn source(&self,review:&crate::pet_runtime::lease_controller::recipe_controller::Review)->Source {
        let config=self.config.clone();Source::capture(self.owner.clone(),&self.goal,review,self.store(),Arc::new(move||config.lock().map(|c|c.clone()).map_err(|_|model::Error::StorageUnavailable))).unwrap()
    }
    fn usage(&self)->Value{serde_json::from_slice::<Value>(&std::fs::read(root(&self.f).join(format!("{}.json",self.goal.admissions[0].spec.decision_id))).unwrap()).unwrap()["body"]["budgetUsage"].clone()}
    fn expire(&self){
        self.stale.store(true,Ordering::Release);let capture=fixture_local_projection(self.settings.clone(),self.configuration(),self.at,self.at+60_001);let d=capture.snapshot.pets.iter().find(|d|d.pet_id=="ai_ollama").unwrap();let cap=d.capabilities.iter().find(|c|c.id=="conversation.respond").unwrap();assert_eq!(cap.availability,Availability::Unknown);assert_eq!(cap.reason,Some(Reason::StaleObservation));assert_eq!(evaluate(&self.goal,self.owner.lease(),&capture,Validation::FreshEffect,self.at+60_001),Validity::Unknown);assert_eq!(evaluate(&self.goal,self.owner.lease(),&capture,Validation::InFlightPolicy,self.at+60_001),Validity::Current);
    }
}
fn metadata_only_server()->(crate::local_chat::Server,std::sync::mpsc::Sender<()>,Arc<AtomicBool>,thread::JoinHandle<Vec<String>>) {
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let port=listener.local_addr().unwrap().port();let stop=Arc::new(AtomicBool::new(false));let end=stop.clone();
    let(start,wait)=std::sync::mpsc::channel();
    let worker=thread::spawn(move||{let mut paths=vec![];if wait.recv().is_err(){return paths;}let until=Instant::now()+Duration::from_secs(4);
        while !end.load(Ordering::Acquire)&&Instant::now()<until {
            let(mut socket,_)=match listener.accept(){Ok(s)=>s,Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{thread::sleep(Duration::from_millis(2));continue;},Err(e)=>panic!("private accept: {e}")};socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();let mut bytes=vec![];let mut chunk=[0u8;4096];let header;
            loop{let n=socket.read(&mut chunk).unwrap();assert!(n>0);bytes.extend_from_slice(&chunk[..n]);assert!(bytes.len()<256*1024);if let Some(i)=bytes.windows(4).position(|s|s==b"\r\n\r\n"){header=i+4;break;}}
            let heads=String::from_utf8_lossy(&bytes[..header]);let path=heads.lines().next().unwrap().split_whitespace().nth(1).unwrap().to_string();let size=heads.lines().find_map(|l|l.to_lowercase().strip_prefix("content-length:").map(|n|n.trim().parse::<usize>().unwrap())).unwrap();while bytes.len()-header<size{let n=socket.read(&mut chunk).unwrap();assert!(n>0);bytes.extend_from_slice(&chunk[..n]);}paths.push(path);
            let body=json!({"model_info":{"general.architecture":"synthetic"},"capabilities":["completion"]}).to_string();write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }paths
    });
    (crate::local_chat::Server{id:"ollama",name:"Synthetic metadata only",url:format!("http://127.0.0.1:{port}"),key:None},start,stop,worker)
}
#[test]fn expired_real_model_metadata_preserves_exact_recipe_approval_and_tools_but_does_not_arm_model_io(){
    let(server,start,stop,http)=metadata_only_server();let case=ToolCase::new(&server.url);let bus=Bus::new();let player=Player::new(&bus,"Private readiness player");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());case.expire();assert!(!case.owner.current(crate::assistant_memory::now()));assert!(case.owner.policy_current(crate::assistant_memory::now()));assert!(source.current());
    pending.answer(json!({"allow":true}),pending.request["goalTarget"].clone()).unwrap();let decision=pending.decision().unwrap();let(approved,ready)=review.approve_decision(decision,&case.owner,&case.goal,&case.configuration(),crate::assistant_memory::now()).unwrap();let row=&case.goal.admissions[0];let scope=case.authority.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),case.owner.lease(),&approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();let result=case.owner.run_recipe(&case.goal,&ready,||Ok(case.configuration()),|_|{}).unwrap();assert_eq!(result.outcome,Outcome::Verified);assert!(result.progress.criteria_verified);assert_eq!(player.calls.lock().unwrap().len(),2);
    start.send(()).unwrap();let response=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(case.owner.respond_once(None,&case.goal,&server,"fresh-model-still-required",&case.configuration()));stop.store(true,Ordering::Release);let paths=http.join().unwrap();assert!(response.is_err());assert_eq!(paths,vec!["/api/show"],"response: {response:?}");assert_eq!(case.owner.workers(),0);let usage=case.usage();let attempts=usage["attempts"].as_array().unwrap();assert_eq!(attempts.len(),2);assert!(attempts.iter().all(|a|a["kind"]=="tool"&&a["phase"]=="settled"));assert_eq!(case.store().read(&case.goal.id,"ai_ollama").unwrap().state,State::Leased);
}
#[test]fn expired_metadata_never_hides_a_real_stop_or_music_permission_revocation(){
    for stop in [false,true]{let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private revoked readiness");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());case.expire();assert!(source.current());if stop{signal_stop(case.owner.lease());}else{case.config.lock().unwrap().pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;}assert!(!source.current());assert!(pending.answer(json!({"allow":true}),pending.request["goalTarget"].clone()).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());}
}
#[test]fn expired_model_metadata_does_not_allow_retargeting_to_a_replacement_player(){
    let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private original player");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source);case.expire();drop(player);let replacement=Player::new(&bus,"Private replacement player");pending.answer(json!({"allow":true}),pending.request["goalTarget"].clone()).unwrap();let decision=pending.decision().unwrap();
    if let Ok((approved,ready))=review.approve_decision(decision,&case.owner,&case.goal,&case.configuration(),crate::assistant_memory::now()) {let row=&case.goal.admissions[0];let scope=case.authority.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),case.owner.lease(),&approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();let result=case.owner.run_recipe(&case.goal,&ready,||Ok(case.configuration()),|_|{}).unwrap();assert!(!result.progress.criteria_verified);assert_eq!(result.progress.verified_steps,0);}
    assert!(replacement.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());
}

fn exclusive(path:&std::path::Path)->std::fs::File{
    use std::os::fd::AsRawFd;
    let file=std::fs::OpenOptions::new().read(true).write(true).open(path).unwrap();
    assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);file
}
#[test]fn a_busy_native_grant_retains_the_same_pending_recipe_without_accepting_or_executing_it(){
    let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private busy grant");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());let target=pending.request["goalTarget"].clone();let id=pending.id.clone();
    let lock=exclusive(&root(&case.f).join(".catalog.lock"));
    assert!(!source.current());assert!(source.pending());assert_eq!(pending.panel_pending()[0]["id"],id);
    assert!(pending.panel_answer(true,target.clone()).is_err());assert_eq!(pending.count(),1);assert!(player.calls.lock().unwrap().is_empty());
    drop(lock);assert!(source.current());assert_eq!(pending.panel_pending()[0]["id"],id);
    pending.panel_answer(true,target).unwrap();let decision=pending.decision().unwrap();let(approved,ready)=review.approve_decision(decision,&case.owner,&case.goal,&case.configuration(),crate::assistant_memory::now()).unwrap();let row=&case.goal.admissions[0];let scope=case.authority.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),case.owner.lease(),&approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();assert!(case.owner.run_recipe(&case.goal,&ready,||Ok(case.configuration()),|_|{}).unwrap().progress.criteria_verified);assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]fn settings_effect_exact_refusal_is_consumed_once_while_positive_authority_is_unknown(){
 let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private refusal under busy grant");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());let target=pending.request["goalTarget"].clone();
 let lock=exclusive(&root(&case.f).join(".catalog.lock"));
 assert!(!source.current());assert!(source.pending());assert!(pending.panel_answer(true,target.clone()).is_err());
 pending.panel_answer(false,target.clone()).unwrap();assert!(!pending.decision().unwrap().approved());
 assert!(pending.panel_answer(false,target).is_err());assert!(player.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());drop(lock);
}
#[test]fn stop_ends_the_original_pending_decision_even_while_the_goal_store_is_busy(){
    let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private busy stopped goal");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());let lock=exclusive(&case.f.path.join("goals/.catalog.lock"));
    assert!(!source.current());assert!(source.pending());assert_eq!(pending.panel_pending().len(),1);assert!(pending.panel_answer(true,pending.request["goalTarget"].clone()).is_err());
    signal_stop(case.owner.lease());assert!(!source.pending());assert!(pending.panel_pending().is_empty());assert!(pending.panel_answer(true,pending.request["goalTarget"].clone()).is_err());drop(lock);assert!(player.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());
}
#[test]fn temporary_configuration_failure_retains_the_decision_but_real_revocation_ends_it(){
    let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private unavailable policy");let review=case.review(&bus);let unavailable=Arc::new(AtomicBool::new(false));let flag=unavailable.clone();let config=case.config.clone();
    let source=Source::capture(case.owner.clone(),&case.goal,&review,case.store(),Arc::new(move||if flag.load(Ordering::Acquire){Err(model::Error::StorageUnavailable)}else{Ok(config.lock().unwrap().clone())})).unwrap();let pending=crate::cli_chat::goal_decision_fixture(source.clone());
    unavailable.store(true,Ordering::Release);assert!(!source.current());assert!(source.pending());assert_eq!(pending.panel_pending().len(),1);assert!(pending.panel_answer(true,pending.request["goalTarget"].clone()).is_err());assert_eq!(pending.count(),1);
    case.config.lock().unwrap().pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;unavailable.store(false,Ordering::Release);assert!(!source.pending());assert!(pending.panel_pending().is_empty());assert!(player.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());
}
#[test]fn an_already_received_answer_waits_for_the_original_proof_and_stop_still_wins(){
    for stop in [false,true]{
        let case=ToolCase::new("http://127.0.0.1:1");let bus=Bus::new();let player=Player::new(&bus,"Private answer revalidation");let review=case.review(&bus);let source=case.source(&review);let pending=crate::cli_chat::goal_decision_fixture(source.clone());pending.panel_answer(true,pending.request["goalTarget"].clone()).unwrap();let lock=exclusive(&root(&case.f).join(".catalog.lock"));
        let(tx,rx)=std::sync::mpsc::channel();let worker=thread::spawn(move||tx.send(pending.decision().is_some()).unwrap());assert!(rx.recv_timeout(Duration::from_millis(60)).is_err());assert!(source.pending());if stop{signal_stop(case.owner.lease());}drop(lock);
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).unwrap(),!stop);worker.join().unwrap();assert!(player.calls.lock().unwrap().is_empty());assert!(case.usage().is_null());
    }
}
