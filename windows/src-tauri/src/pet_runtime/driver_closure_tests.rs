use super::*;
use crate::pet_runtime::{generated_effect_tests as generated,admission_tests::Fixture,authority_tests::root,authority_store::Fault as AuthFault,store::Fault as StoreFault,model::{self,State,Action}};
use crate::pet_runtime::lease_controller::writer::tests::{run,setup_planning,fixture_with_hook,local_metadata,stream,ledger};
use std::{fs::{self,OpenOptions,File},os::fd::AsRawFd,sync::atomic::AtomicUsize};
struct Env{url:Option<String>,captures:AtomicUsize,drains:AtomicUsize,notices:Mutex<Vec<Status>>}
#[test]fn a_bounded_retry_keeps_the_same_proof_for_an_explicit_cleanup_only_restart(){
    let(f,_a,g,_p,owner,main)=generated::setup(true);let deadline=owner.lease().deadline_at;let env=Env::new(None);let driver=Driver::new(Arc::new(owner),Arc::new(generated::native(&f)),env.clone()).unwrap();let lock=shared(&f);let closing=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();let usage=ledger(&f,&g);
    assert_eq!(run(driver.run()).err(),Some(model::Error::Busy));let status=driver.status().unwrap();assert_eq!(status.phase,Phase::Closing);assert!(!status.active&&status.retained_closure);assert!(!driver.removable());assert_eq!(driver.owner.lease().deadline_at,deadline);assert!(Arc::ptr_eq(&closing,driver.state.lock().unwrap().closing.as_ref().unwrap()));assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());drop(lock);
    assert_eq!(run(driver.run()).unwrap().phase,Phase::Stopped);assert_eq!(ledger(&f,&g),usage);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(driver.owner.lease().deadline_at,deadline);assert!(main.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn an_old_unacknowledged_closure_cannot_write_over_a_new_admission_of_the_same_goal(){
    let(f,a,g,_p,owner,_main)=generated::setup(true);crate::pet_runtime::authority_store::mutate_with_authority(&generated::native(&f),&root(&f),&g.id,"ai_ollama",&g.revision,"pause-before-uncertain-close".into(),Action::Pause,crate::assistant_memory::now()).unwrap();let env=Env::new(None);let mut store=generated::native(&f);store.fault=StoreFault::AfterPublish;let driver=Driver::new(Arc::new(owner),Arc::new(store),env.clone()).unwrap();let closing=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();assert_eq!(closing.attempt(&driver).err(),Some(model::Error::StorageUnavailable));
    let old=generated::native(&f).read(&g.id,"ai_ollama").unwrap();let now=crate::assistant_memory::now();let draft=generated::native(&f).mutate(&g.id,"ai_ollama",&old.revision,"new-draft-after-native-drain".into(),Action::ResumeDraft,now).unwrap().record;
    let approved=crate::pet_runtime::admission::Review::prepare(&draft,crate::pet_runtime::admission_tests::evidence(&draft,now+1),g.admissions[0].spec.budget.clone(),"new-explicit-admission".into(),crate::pet_runtime::store::nonce().unwrap(),now+1).unwrap().approve(&draft,crate::pet_runtime::admission_tests::evidence(&draft,now+2),now+2).unwrap();let certificate=a.certify(&approved,&draft,now+3).unwrap();let queued=generated::native(&f).enqueue_certified(&certificate,now+4).unwrap().record;
    let before=fs::read(f.path.join("goals").join(format!("{}.json",g.id))).unwrap();assert_eq!(closing.attempt(&driver).err(),Some(model::Error::Conflict));assert_eq!(fs::read(f.path.join("goals").join(format!("{}.json",g.id))).unwrap(),before);assert_eq!(queued.admissions.len(),2);assert_eq!(env.drains.load(Ordering::Acquire),0);assert_eq!(env.captures.load(Ordering::Acquire),0);
}
impl Env{fn new(url:Option<String>)->Arc<Self>{Arc::new(Self{url,captures:AtomicUsize::new(0),drains:AtomicUsize::new(0),notices:Mutex::new(vec![])})}}
impl Environment for Env{
    fn capture(&self,_:&Owned)->model::Result<Capture>{self.captures.fetch_add(1,Ordering::AcqRel);Ok(Capture{configuration:generated::config(),server:crate::local_chat::Server{id:"ollama",name:"Free fixture",url:self.url.clone().expect("closure must never perform inference"),key:None},app:None})}
    fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(generated::config())}
    fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("closure cannot execute a player operation")}
    fn decide(&self,_:Source)->Option<Decision>{panic!("closure cannot request another approval")}
    fn notify(&self,s:&Status){self.notices.lock().unwrap().push(s.clone());}
    fn drained(&self,_:&DrainProof)->model::Result<()>{self.drains.fetch_add(1,Ordering::AcqRel);Ok(())}
}
fn shared(f:&Fixture)->File{let file=OpenOptions::new().read(true).write(true).open(root(f).join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);file}
// A concurrent native closure may briefly own the catalog. Retry only typed
// Busy on the same fixture operation; uncertainty/other errors remain errors.
fn when_catalog_available<T>(mut operation:impl FnMut()->model::Result<T>)->T{
    let until=Instant::now()+Duration::from_secs(2);
    loop{match operation(){Err(model::Error::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(5)),result=>return result.unwrap()}}
}
fn grant(f:&Fixture,g:&Record)->serde_json::Value{serde_json::from_slice::<serde_json::Value>(&fs::read(root(f).join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap()).unwrap()["body"].clone()}
fn wait_closing(driver:&Driver){let until=Instant::now()+Duration::from_secs(3);while !driver.status().unwrap().retained_closure{assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}}
#[test]fn actual_http_stop_retains_one_native_proof_and_unknown_attempt_through_lock_and_projection_delay(){
    let(f,a,g,_pending,owner,main)=setup_planning();let owner=Arc::new(owner);let lock=Arc::new(Mutex::new(None));let thread_lock=lock.clone();let path=root(&f);let spec=g.admissions[0].spec.clone();let proof=g.admissions[0].proof.clone().unwrap();let lease=owner.lease().clone();
    let(server,http)=fixture_with_hook(vec![("application/json",local_metadata().to_string()),("text/event-stream",stream("interrupted",true))],move|endpoint|{if endpoint=="/v1/chat/completions"{
        let authority=Authority::existing(&path).unwrap();authority.request_stop(&spec,&proof,crate::pet_runtime::leases::StopReason::Pause,crate::assistant_memory::now()).unwrap();let file=OpenOptions::new().read(true).write(true).open(path.join(".catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);*thread_lock.lock().unwrap()=Some(file);crate::pet_runtime::lease_controller::signal_stop(&lease);
    }});
    let env=Env::new(Some(server.url));let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env.clone()).unwrap();driver.spawn().unwrap();wait_closing(&driver);
    let closing=driver.state.lock().unwrap().closing.clone().unwrap();assert_eq!(owner.workers(),0);assert!(driver.status().unwrap().active);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());let original_usage=ledger(&f,&g);assert_eq!(original_usage["attempts"].as_array().unwrap().len(),1);assert_eq!(original_usage["attempts"][0]["phase"],"started");lock.lock().unwrap().take();
    std::thread::sleep(Duration::from_millis(100));assert_eq!(grant(&f,&g)["phase"],"draining");assert_eq!(when_catalog_available(||generated::native(&f).read(&g.id,"ai_ollama")).state,State::Leased);assert!(Arc::ptr_eq(&closing,driver.state.lock().unwrap().closing.as_ref().unwrap()));
    let pause_at=crate::assistant_memory::now();let projected=when_catalog_available(||generated::native(&f).mutate(&g.id,"ai_ollama",&g.revision,"project-original-stop".into(),Action::Pause,pause_at));assert_ne!(projected.status,model::CommitStatus::Uncertain);let until=Instant::now()+Duration::from_secs(5);while driver.status().unwrap().active{assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
    assert_eq!(driver.status().unwrap().phase,Phase::Stopped);assert!(!driver.status().unwrap().retained_closure);assert_eq!(generated::native(&f).read(&g.id,"ai_ollama").unwrap().state,State::Paused);assert_eq!(ledger(&f,&g),original_usage);assert_eq!(http.join().unwrap().len(),2);assert_eq!(env.captures.load(Ordering::Acquire),1);assert_eq!(env.drains.load(Ordering::Acquire),1);assert!(main.runtime_resources().busy().unwrap().is_empty());drop(a);
}
#[test]fn uncertain_goal_publication_retries_only_the_exact_closure_and_never_releases_a_new_owner(){
    for fault in [StoreFault::BeforePublish,StoreFault::AfterPublish]{
        let(f,a,g,_pending,owner,main)=generated::setup(true);let env=Env::new(None);let mut store=generated::native(&f);store.fault=fault;let mut driver=Driver::new(Arc::new(owner),Arc::new(store),env.clone()).unwrap();
        let closing=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();assert_eq!(closing.attempt(&driver).err(),Some(model::Error::StorageUnavailable));assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());let original=grant(&f,&g);assert_eq!(original["phase"],"closed");
        Arc::get_mut(&mut Arc::get_mut(&mut driver).unwrap().store).unwrap().fault=StoreFault::None;assert_eq!(run(driver.run()).unwrap().phase,Phase::Stopped);assert_eq!(grant(&f,&g),original);assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(env.drains.load(Ordering::Acquire),1);
        let fresh=main.runtime_resources().acquire("ollama",Arc::default(),None).unwrap();closing.attempt(&driver).unwrap();assert!(fresh.current());assert_eq!(env.drains.load(Ordering::Acquire),1);assert_eq!(grant(&f,&g)["budgetUsage"],original["budgetUsage"]);drop(a);
    }
}
#[test]fn uncertain_authority_closure_retains_the_original_proof_and_stabilizes_without_reissuing_a_close(){
    let(f,mut authority,g,_p,owner,main)=generated::setup(true);let env=Env::new(None);let driver=Driver::new(Arc::new(owner),Arc::new(generated::native(&f)),env.clone()).unwrap();let closing=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();authority.fault=AuthFault::AfterGrantPublish;*closing.authority.lock().unwrap()=Some(authority);
    assert_eq!(closing.attempt(&driver).err(),Some(model::Error::StorageUnavailable));let first=grant(&f,&g);assert_eq!(first["phase"],"closed");assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());closing.authority.lock().unwrap().as_mut().unwrap().fault=AuthFault::None;
    assert_eq!(run(driver.run()).unwrap().phase,Phase::Stopped);assert_eq!(grant(&f,&g),first);assert_eq!(env.captures.load(Ordering::Acquire),0);assert!(main.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn a_live_worker_prevents_creating_a_closure_proof_or_releasing_its_slot(){
    let(f,_a,_g,_p,owner,main)=generated::setup(true);let owner=Arc::new(owner);let worker=owner.readiness_worker(crate::assistant_memory::now()).unwrap();let env=Env::new(None);let driver=Driver::new(owner.clone(),Arc::new(generated::native(&f)),env).unwrap();assert!(matches!(driver.prepare_closure(Phase::Stopped,0,None,None),Err(model::Error::Busy)));assert!(!driver.status().unwrap().retained_closure);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());drop(worker);let c=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();run(driver.resume_closure(c)).unwrap();assert!(main.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn retained_closure_rejects_a_replaced_profile_authority_without_releasing_or_replaying(){
    let(f,_a,g,_p,owner,main)=generated::setup(true);let env=Env::new(None);let driver=Driver::new(Arc::new(owner),Arc::new(generated::native(&f)),env.clone()).unwrap();let lock=shared(&f);let closing=driver.prepare_closure(Phase::Stopped,0,None,None).unwrap();assert_eq!(closing.attempt(&driver).err(),Some(model::Error::Busy));drop(lock);let moved=f.path.join("original-authority");fs::rename(root(&f),&moved).unwrap();Authority::ensure(&root(&f)).unwrap();let old=fs::read(moved.join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap();
    assert_eq!(closing.attempt(&driver).err(),Some(model::Error::Conflict));assert_eq!(fs::read(moved.join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap(),old);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());assert_eq!(env.captures.load(Ordering::Acquire),0);assert_eq!(env.drains.load(Ordering::Acquire),0);
}
