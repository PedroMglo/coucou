use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64,Ordering};
use serde_json::{json,Value};

const SERVER:&str=r#"import json,os,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'mode').read_text()
(root/'pid').write_text(str(os.getpid()))
for line in sys.stdin:
    m=json.loads(line)
    with (root/'requests.jsonl').open('a') as log:log.write(json.dumps(m)+'\n')
    method=m.get('method')
    if method=='initialized':continue
    if mode=='hang' and method=='account/usage/read':
        while True:time.sleep(1)
    if mode=='rpc-error' and method=='account/usage/read':
        print(json.dumps({'id':m['id'],'error':{'code':-32601,'message':'PRIVATE_NATIVE_DETAIL'}}),flush=True)
        continue
    result={}
    if method=='account/read':result={'account':{'type':'apiKey' if mode=='api-key' else 'chatgpt','email':'PRIVATE_EMAIL'}}
    if method=='account/usage/read':result={'summary':{'lifetimeTokens':-1 if mode=='invalid' else 23,'currentStreakDays':0},'dailyUsageBuckets':None}
    print(json.dumps({'id':m['id'],'result':result}),flush=True)
"#;

struct Fixture{dir:PathBuf}
impl Fixture{
 fn new(mode:&str)->Self{
  static NEXT:AtomicU64=AtomicU64::new(1);
  let dir=std::env::temp_dir().join(format!("coucou-account-usage-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
  use std::os::unix::fs::DirBuilderExt;
  std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();std::fs::write(dir.join("app-server"),SERVER).unwrap();std::fs::write(dir.join("mode"),mode).unwrap();Self{dir}
 }
 fn run(&self,budget:Duration)->Result<Summary,Failure>{read_from_budget(std::path::Path::new("/usr/bin/python3"),&self.dir,budget)}
 fn requests(&self)->Vec<Value>{std::fs::read_to_string(self.dir.join("requests.jsonl")).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect()}
 fn methods(&self)->Vec<String>{self.requests().iter().map(|v|v["method"].as_str().unwrap().to_owned()).collect()}
 fn reaped(&self){let pid:libc::pid_t=std::fs::read_to_string(self.dir.join("pid")).unwrap().parse().unwrap();assert_eq!(unsafe{libc::kill(pid,0)},-1);assert_eq!(std::io::Error::last_os_error().raw_os_error(),Some(libc::ESRCH));}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.dir);}}

#[test]fn native_protocol_only_initializes_and_reads_account_metadata(){
 let f=Fixture::new("ok");let s=f.run(Duration::from_secs(3)).unwrap();assert_eq!(s.lifetime_tokens.as_deref(),Some("23"));assert_eq!(s.current_streak_days.as_deref(),Some("0"));assert!(s.peak_daily_tokens.is_none());
 assert_eq!(f.methods(),["initialize","initialized","account/read","account/usage/read"]);
 let requests=f.requests();assert_eq!(requests[2]["params"],json!({"refreshToken":false}));assert_eq!(requests[3]["params"],json!({}));assert_eq!(requests[0]["params"]["capabilities"]["experimentalApi"],false);f.reaped();
}
#[test]fn api_key_auth_is_refused_before_usage_or_any_model_request(){
 let f=Fixture::new("api-key");assert_eq!(f.run(Duration::from_secs(3)),Err(Failure::NotSignedIn));assert_eq!(f.methods(),["initialize","initialized","account/read"]);f.reaped();
}
#[test]fn native_errors_are_static_and_never_leak_message_details(){
 let f=Fixture::new("rpc-error");let error=f.run(Duration::from_secs(3)).unwrap_err();assert_eq!(error,Failure::NativeUnavailable);assert!(!serde_json::to_string(&error).unwrap().contains("PRIVATE"));assert_eq!(f.methods().len(),4);f.reaped();
}
#[test]fn invalid_native_counts_reject_without_inventing_measurements(){
 let f=Fixture::new("invalid");assert_eq!(f.run(Duration::from_secs(3)),Err(Failure::InvalidData));assert_eq!(f.methods().len(),4);f.reaped();
}
#[test]fn stalled_metadata_read_is_bounded_and_reaps_owned_child(){
 let f=Fixture::new("hang");let began=Instant::now();assert_eq!(f.run(Duration::from_millis(400)),Err(Failure::NativeUnavailable));assert!(began.elapsed()<Duration::from_secs(3));assert_eq!(f.methods(),["initialize","initialized","account/read","account/usage/read"]);f.reaped();
}
#[test]fn dropped_waiter_does_not_release_single_flight_while_reader_is_alive(){
 let cache=Arc::new(Mutex::new(Cache::with_instance(Some("11111111111111111111111111111111".into()))));let ticket=cache.lock().unwrap().begin(Instant::now()).unwrap();
 let(tx,rx)=std::sync::mpsc::channel();let(release,wait)=std::sync::mpsc::channel();let shared=cache.clone();let (result,discard)=std::sync::mpsc::channel::<Snapshot>();drop(discard);
 let worker=std::thread::spawn(move||{let view=refresh_worker(&shared,ticket,||{tx.send(()).unwrap();wait.recv_timeout(Duration::from_secs(2)).unwrap();crate::account_usage_core::parse(&json!({"summary":{"lifetimeTokens":23}}))},|_|{}).unwrap();let _=result.send(view);});
 rx.recv_timeout(Duration::from_secs(2)).unwrap();assert_eq!(cache.lock().unwrap().begin(Instant::now()+Duration::from_secs(99)),Err(RefreshError::Busy));release.send(()).unwrap();worker.join().unwrap();let current=cache.lock().unwrap().snapshot(clock());assert!(!current.refreshing);assert_eq!(current.summary.unwrap().lifetime_tokens.as_deref(),Some("23"));
}
#[test]fn reader_unwind_does_not_leave_permanent_busy_or_expose_panic_text(){
 let cache=Mutex::new(Cache::with_instance(Some("11111111111111111111111111111111".into())));let ticket=cache.lock().unwrap().begin(Instant::now()).unwrap();let view=refresh_worker(&cache,ticket,||panic!("synthetic read panic"),|_|{}).unwrap();assert!(!view.refreshing);assert_eq!(view.reason,Some(Failure::NativeUnavailable));assert!(view.summary.is_none());
}

#[test]fn begin_and_finish_callbacks_run_after_cache_unlock(){
 let cache=Mutex::new(Cache::with_instance(Some("11111111111111111111111111111111".into())));
 let witnesses=Mutex::new(Vec::new());
 let record=|view:&Snapshot|{let unlocked=cache.try_lock().is_ok();witnesses.lock().unwrap().push((unlocked,view.revision.clone(),view.refreshing,view.summary.is_some()));};
 let ticket=begin_refresh(&cache,Instant::now(),record).unwrap();
 let result=refresh_worker(&cache,ticket,||crate::account_usage_core::parse(&json!({"summary":{"lifetimeTokens":23}})),record).unwrap();
 // notify intentionally catches publisher panics. Assert external witnesses,
 // so a lock/regression inside the callback cannot turn into a false PASS.
 assert_eq!(witnesses.into_inner().unwrap(),vec![(true,"1".into(),true,false),(true,"2".into(),false,true)]);
 assert_eq!(result.revision,"2");
}
#[test]fn failure_and_unwind_publish_new_committed_revisions_after_unlock(){
 for panic in [false,true]{
  let cache=Mutex::new(Cache::with_instance(Some("11111111111111111111111111111111".into())));let ticket=begin_refresh(&cache,Instant::now(),|_|{}).unwrap();let event=Mutex::new(None);
  let result=refresh_worker(&cache,ticket,||{if panic{panic!("synthetic");}Err(Failure::NotSignedIn)},|view|{assert!(cache.try_lock().is_ok());*event.lock().unwrap()=Some((view.revision.clone(),view.reason));}).unwrap();
  assert_eq!(event.into_inner().unwrap(),Some(("2".into(),Some(if panic{Failure::NativeUnavailable}else{Failure::NotSignedIn}))));assert!(!result.refreshing);
 }
}
#[test]fn lost_change_publisher_does_not_strand_worker_or_completed_ack(){
 let cache=Mutex::new(Cache::with_instance(Some("11111111111111111111111111111111".into())));
 let ticket=begin_refresh(&cache,Instant::now(),|_|panic!("synthetic missing listener")).unwrap();assert!(cache.lock().unwrap().snapshot(clock()).refreshing);
 let result=refresh_worker(&cache,ticket,||crate::account_usage_core::parse(&json!({"summary":{"lifetimeTokens":23}})),|_|panic!("synthetic lost completion event")).unwrap();assert_eq!(result.revision,"2");assert!(!cache.lock().unwrap().snapshot(clock()).refreshing);
}
#[test]fn native_runtime_birth_uses_distinct_opaque_sources_without_auth_or_client(){
 let a=Runtime::default();let b=Runtime::default();let av=a.cache.lock().unwrap().snapshot(clock());let bv=b.cache.lock().unwrap().snapshot(clock());assert_ne!(av.source_instance,bv.source_instance);assert_eq!(av.source_instance.unwrap().len(),32);assert_eq!(av.revision,"0");assert!(av.summary.is_none()&&!av.refreshing);
}
