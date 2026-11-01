use super::*;
use serde_json::{json,Value};
use std::os::unix::fs::{symlink,PermissionsExt};
use std::sync::{atomic::AtomicU64,Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::Notify;

// Singleton is process-wide; independent existing transport/core fixtures do not
// instantiate it. Only executor tests serialize their actor lifetime here.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
struct Step {
    result: Value,
    error: Option<&'static str>,
    greeting_gate: Option<Arc<Notify>>,
    reply_gate: Option<Arc<Notify>>,
    rebind: Option<(Arc<Notify>,Arc<Notify>)>,
}
impl Step {
    fn read() -> Self {Self {result:json!({"wifiEnabled":true,"wifiNetworks":[]}),error:None,greeting_gate:None,reply_gate:None,rebind:None}}
    fn scan() -> Self {Self {result:json!({"success":true,"message":"scanning"}),..Self::read()}}
}
struct Fixture {
    root: std::path::PathBuf,
    native: t::Config,
    requests: Arc<Mutex<Vec<Value>>>,
    connections: Arc<AtomicU64>,
    task: JoinHandle<()>,
}
impl Fixture {
    fn start(steps: Vec<Step>) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root=std::env::temp_dir().join(format!("mochi-executor-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        std::fs::create_dir(&root).unwrap();std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket=root.join("danklinux-123.sock");let listener=UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
        let executable=root.join("trusted-executable");symlink(std::env::current_exe().unwrap(),&executable).unwrap();
        let native=t::config_for_fixture(socket.clone(),root.clone(),executable,unsafe{libc::geteuid()},Duration::from_secs(2));
        let requests=Arc::new(Mutex::new(Vec::new()));let captured=requests.clone();let directory=root.clone();
        let connections=Arc::new(AtomicU64::new(0));let counted=connections.clone();
        let task=tokio::spawn(async move {
            let mut listener=listener;
            for step in steps {
                if let Some((start,ready))=step.rebind {
                    start.notified().await;std::fs::remove_file(&socket).unwrap();listener=UnixListener::bind(&socket).unwrap();
                    std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();ready.notify_one();
                }
                let Ok((mut stream,_))=listener.accept().await else {return;};counted.fetch_add(1,Ordering::Relaxed);
                if let Some(gate)=step.greeting_gate {gate.notified().await;}
                if stream.write_all(b"{\"apiVersion\":34,\"capabilities\":[\"network\"]}\n").await.is_err(){continue;}
                let mut data=Vec::new();let mut buffer=[0u8;1024];
                while !data.contains(&b'\n') {
                    let Ok(n)=stream.read(&mut buffer).await else {break;};if n==0{break;}data.extend_from_slice(&buffer[..n]);assert!(data.len()<8192);
                }
                if data.is_empty(){continue;}
                let request:Value=serde_json::from_slice(data.split(|b|*b==b'\n').next().unwrap()).unwrap();captured.lock().unwrap().push(request.clone());
                if let Some(gate)=step.reply_gate {gate.notified().await;}
                let response=if let Some(error)=step.error {json!({"id":request["id"],"error":error})}else{json!({"id":request["id"],"result":step.result})};
                let _=stream.write_all(format!("{response}\n").as_bytes()).await;
                let _=&directory; // native fixture paths are never public receipts
            }
        });
        Self {root,native,requests,connections,task}
    }
    async fn count(&self, method:&str) -> usize {
        tokio::task::yield_now().await;
        self.requests.lock().unwrap().iter().filter(|r|r["method"]==method).count()
    }
    async fn wait_requests(&self,n:usize) {
        tokio::time::timeout(Duration::from_secs(3),async {
            loop {if self.requests.lock().unwrap().len()>=n {break;}tokio::time::sleep(Duration::from_millis(1)).await;}
        }).await.unwrap();
    }
}
impl Drop for Fixture {fn drop(&mut self){self.task.abort();let _=std::fs::remove_dir_all(&self.root);}}
fn config(lifecycle:l::Config) -> Config {Config {lifecycle,boot_nonce:99,producer_nonce:77,command_capacity:64}}
fn launch(f:&Fixture,lc:l::Config)->(Executor,Client,Arc<AuthorityFence>,Arc<NativeWindowOwner>,WindowRef) {
    let native_setting=l::SystemSetting::from_backend(true);let setting=AuthorityFence::from_backend(native_setting.clone());
    let native=Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&native_setting));
    let owner=NativeWindowOwner::from_backend(native,&setting);let window=WindowRef::from_backend(&owner);let executor=Executor::start(f.native.clone(),config(lc),&setting).unwrap();let client=executor.client();
    (executor,client,setting,owner,window)
}
async fn inspection(client:&Client) -> Inspection {client.submit(|response|Command::Inspect {response}).await.unwrap()}
async fn terminal(client:&Client,window:&WindowRef,id:&str)->l::Ticket {
    tokio::time::timeout(Duration::from_secs(3),async {
        loop {let t=client.ticket(window,id).await.unwrap();if !matches!(t.phase,l::Phase::Queued|l::Phase::Running){return t;}tokio::time::sleep(Duration::from_millis(1)).await;}
    }).await.unwrap()
}
async fn idle(client:&Client) {
    tokio::time::timeout(Duration::from_secs(3),async {
        loop {if !inspection(client).await.active {break;}tokio::time::sleep(Duration::from_millis(1)).await;}
    }).await.unwrap();
}
async fn stopped() {
    tokio::time::timeout(Duration::from_secs(3),async {
        while SINGLETON.load(Ordering::Acquire){tokio::time::sleep(Duration::from_millis(1)).await;}
    }).await.unwrap();
}
async fn accepted_read(client:&Client,window:&WindowRef)->l::Ticket {
    let a=client.read(window,true).await.unwrap();let t=terminal(client,window,&a.id).await;assert_eq!(t.phase,l::Phase::Observed);t
}
fn expect<T>(result:Result<T,l::Reason>,reason:l::Reason) {assert_eq!(result.err(),Some(reason));}

#[tokio::test]
async fn actor_publishes_one_verified_read_then_dispatches_one_ack_only_global_scan() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read(),Step::scan()]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
    accepted_read(&client,&window).await;let state=inspection(&client).await;assert_eq!(state.reserved,1);assert!(state.association.is_some());assert!(!state.active);
    let view=client.view(&window).await.unwrap();assert_eq!(view.generation,Some("1".into()));assert!(view.targets.is_empty());assert!(view.capabilities.scan);
    let a=client.scan(&window,&view.scan_scope.unwrap(),"first").await.unwrap();let t=terminal(&client,&window,&a.id).await;
    assert_eq!(t.phase,l::Phase::Acknowledged);assert!(!t.effect_confirmed);assert!(t.refresh_recommended);
    assert!(client.view(&window).await.unwrap().stale);assert_eq!(f.count("network.getState").await,1);assert_eq!(f.count("network.wifi.scan").await,1);
    for r in f.requests.lock().unwrap().iter(){assert_eq!(r.as_object().unwrap().len(),2);assert!(r.get("params").is_none());}
    ex.shutdown().await;stopped().await;
}
#[tokio::test]
async fn concurrent_reads_join_one_worker_and_keep_reserved_origin_private_until_accepted() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());
    let f=Fixture::start(vec![step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
    let first=client.read(&window,true).await.unwrap();f.wait_requests(1).await;
    let mut ids=vec![first.id];for _ in 0..10{ids.push(client.read(&window,true).await.unwrap().id);}
    let state=inspection(&client).await;assert!(state.active);assert_eq!(state.reserved,1);assert!(state.association.is_none());
    assert_eq!(f.count("network.getState").await,1);gate.notify_one();
    for id in ids{assert_eq!(terminal(&client,&window,&id).await.phase,l::Phase::Observed);}
    assert!(inspection(&client).await.association.is_some());ex.shutdown().await;
}
#[tokio::test]
async fn aborting_a_caller_future_does_not_abort_owned_io_or_free_its_worker() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());
    let f=Fixture::start(vec![step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
    let other=client.clone();let w=window.clone();let(tx,rx)=oneshot::channel();
    let caller=tokio::spawn(async move {let a=other.read(&w,true).await.unwrap();let _=tx.send(a.id.clone());loop{let _=other.ticket(&w,&a.id).await;tokio::time::sleep(Duration::from_millis(1)).await;}});
    let id=rx.await.unwrap();f.wait_requests(1).await;caller.abort();let _=caller.await;
    assert!(inspection(&client).await.active);let joined=client.read(&window,true).await.unwrap();assert_eq!(inspection(&client).await.reserved,1);
    assert_eq!(f.count("network.getState").await,1);gate.notify_one();assert_eq!(terminal(&client,&window,&id).await.phase,l::Phase::Observed);
    assert_eq!(terminal(&client,&window,&joined.id).await.phase,l::Phase::Observed);ex.shutdown().await;
}
#[tokio::test]
async fn logical_deadline_retains_actual_worker_until_native_io_terminates() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());
    let f=Fixture::start(vec![step,Step::read()]);let mut lc=l::Config::default();lc.operation_timeout=Duration::from_millis(40);
    let(ex,client,_,_owner,window)=launch(&f,lc);let a=client.read(&window,true).await.unwrap();f.wait_requests(1).await;
    tokio::time::sleep(Duration::from_millis(60)).await;let t=client.ticket(&window,&a.id).await.unwrap();assert_eq!((t.phase,t.reason),(l::Phase::Unknown,Some(l::Reason::Deadline)));
    expect(client.read(&window,true).await,l::Reason::Busy);assert!(inspection(&client).await.active);assert_eq!(f.count("network.getState").await,1);
    gate.notify_one();idle(&client).await;assert!(inspection(&client).await.association.is_none());assert!(client.view(&window).await.unwrap().data.is_none());
    accepted_read(&client,&window).await;assert_eq!(inspection(&client).await.reserved,2);ex.shutdown().await;
}
#[tokio::test]
async fn cancelling_all_read_tickets_blocks_publication_without_aborting_io() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());
    let f=Fixture::start(vec![step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let a=client.read(&window,true).await.unwrap();f.wait_requests(1).await;
    let t=client.cancel(&window,&a.id).await.unwrap();assert_eq!(t.reason,Some(l::Reason::CancelledAfterDispatch));assert!(inspection(&client).await.active);
    gate.notify_one();idle(&client).await;assert!(inspection(&client).await.association.is_none());assert!(client.view(&window).await.unwrap().data.is_none());
    assert_eq!(client.ticket(&window,&a.id).await.unwrap().phase,l::Phase::Unknown);ex.shutdown().await;
}
#[tokio::test]
async fn cancelling_one_read_join_allows_other_ticket_to_publish_same_origin() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());
    let f=Fixture::start(vec![step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
    let a=client.read(&window,true).await.unwrap();f.wait_requests(1).await;let b=client.read(&window,true).await.unwrap();client.cancel(&window,&a.id).await.unwrap();gate.notify_one();
    assert_eq!(terminal(&client,&window,&b.id).await.phase,l::Phase::Observed);assert_eq!(client.ticket(&window,&a.id).await.unwrap().phase,l::Phase::Unknown);
    assert!(inspection(&client).await.association.is_some());assert_eq!(f.count("network.getState").await,1);ex.shutdown().await;
}
#[tokio::test]
async fn malformed_projection_never_publishes_reserved_id_or_scan_scope() {
    let _serial=SERIAL.lock().await;stopped().await;let mut bad=Step::read();bad.result=json!([]);let f=Fixture::start(vec![bad,Step::read()]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let a=client.read(&window,true).await.unwrap();let t=terminal(&client,&window,&a.id).await;
    assert_eq!(t.reason,Some(l::Reason::InvalidObservation));let state=inspection(&client).await;assert_eq!(state.reserved,1);assert!(state.association.is_none());
    assert!(client.view(&window).await.unwrap().scan_scope.is_none());accepted_read(&client,&window).await;
    let state=inspection(&client).await;let mut counter=ProducerCounter{nonce:77,next:0};let failed=counter.reserve().unwrap();let accepted=counter.reserve().unwrap();
    assert!(state.association.as_ref().unwrap().producer==accepted);assert!(state.association.unwrap().producer!=failed);ex.shutdown().await;
}
#[tokio::test]
async fn failed_read_keeps_prior_cache_time_and_exact_association_stale() {
    let _serial=SERIAL.lock().await;stopped().await;let mut failure=Step::read();failure.error=Some("PRIVATE_DEVICE_ERROR");let f=Fixture::start(vec![Step::read(),failure]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let before=client.view(&window).await.unwrap();
    let before_data=serde_json::to_value(&before).unwrap()["data"].clone();let old=inspection(&client).await.association.unwrap();
    let a=client.read(&window,true).await.unwrap();assert_eq!(terminal(&client,&window,&a.id).await.phase,l::Phase::Failed);
    let after=client.view(&window).await.unwrap();assert!(after.stale);assert!(after.scan_scope.is_none());assert_eq!(before_data,serde_json::to_value(&after).unwrap()["data"]);
    let state=inspection(&client).await;assert_eq!(state.reserved,2);let current=state.association.unwrap();assert!(current.origin==old.origin&&current.producer==old.producer);
    expect(client.scan(&window,&before.scan_scope.unwrap(),"stale").await,l::Reason::StaleTarget);assert_eq!(f.count("network.wifi.scan").await,0);ex.shutdown().await;
}
#[tokio::test]
async fn accepted_read_can_rotate_id_of_exact_origin_only_at_next_cache_generation() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut second=Step::read();second.reply_gate=Some(gate.clone());let f=Fixture::start(vec![Step::read(),second,Step::scan()]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let old=inspection(&client).await.association.unwrap();let old_view=client.view(&window).await.unwrap();
    let second=client.read(&window,true).await.unwrap();f.wait_requests(2).await;let pending=inspection(&client).await;assert_eq!(pending.reserved,2);assert!(pending.association.unwrap().producer==old.producer);
    expect(client.scan(&window,old_view.scan_scope.as_ref().unwrap(),"during").await,l::Reason::Busy);
    gate.notify_one();assert_eq!(terminal(&client,&window,&second.id).await.phase,l::Phase::Observed);let new=inspection(&client).await.association.unwrap();
    assert!(new.origin==old.origin);assert!(new.producer!=old.producer);let view=client.view(&window).await.unwrap();assert_eq!(view.generation,Some("2".into()));
    expect(client.scan(&window,&old_view.scan_scope.unwrap(),"old").await,l::Reason::StaleTarget);
    let a=client.scan(&window,&view.scan_scope.unwrap(),"current").await.unwrap();assert_eq!(terminal(&client,&window,&a.id).await.phase,l::Phase::Acknowledged);ex.shutdown().await;
}
#[tokio::test]
async fn new_endpoint_same_pid_updates_exact_origin_without_retargeting_old_handles() {
    let _serial=SERIAL.lock().await;stopped().await;let start=Arc::new(Notify::new());let ready=Arc::new(Notify::new());let mut second=Step::read();second.rebind=Some((start.clone(),ready.clone()));
    let f=Fixture::start(vec![Step::read(),second,Step::scan()]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;
    let old=inspection(&client).await.association.unwrap();let scope=client.view(&window).await.unwrap().scan_scope.unwrap();start.notify_one();ready.notified().await;
    accepted_read(&client,&window).await;let new=inspection(&client).await.association.unwrap();assert!(new.origin!=old.origin);assert!(new.producer!=old.producer);
    expect(client.scan(&window,&scope,"old-source").await,l::Reason::StaleTarget);let scope=client.view(&window).await.unwrap().scan_scope.unwrap();
    let a=client.scan(&window,&scope,"new-source").await.unwrap();assert_eq!(terminal(&client,&window,&a.id).await.phase,l::Phase::Acknowledged);assert_eq!(f.count("network.wifi.scan").await,1);ex.shutdown().await;
}
#[tokio::test]
async fn cancelling_scan_during_greeting_holds_slot_and_sends_no_scan() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut scan=Step::scan();scan.greeting_gate=Some(gate.clone());let f=Fixture::start(vec![Step::read(),scan]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let scope=client.view(&window).await.unwrap().scan_scope.unwrap();
    let a=client.scan(&window,&scope,"cancel-queued").await.unwrap();let t=client.cancel(&window,&a.id).await.unwrap();assert_eq!(t.phase,l::Phase::Cancelled);
    expect(client.read(&window,true).await,l::Reason::Busy);assert!(inspection(&client).await.active);gate.notify_one();idle(&client).await;
    assert_eq!(f.count("network.wifi.scan").await,0);let repeated=client.scan(&window,&scope,"cancel-queued").await.unwrap();assert_eq!(repeated.id,a.id);assert_eq!(repeated.phase,l::Phase::Cancelled);ex.shutdown().await;
}
#[tokio::test]
async fn cancelling_sent_scan_keeps_unknown_after_ack_and_no_action_retry() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut scan=Step::scan();scan.reply_gate=Some(gate.clone());let f=Fixture::start(vec![Step::read(),scan]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let scope=client.view(&window).await.unwrap().scan_scope.unwrap();
    let a=client.scan(&window,&scope,"cancel-running").await.unwrap();f.wait_requests(2).await;let t=client.cancel(&window,&a.id).await.unwrap();assert_eq!(t.phase,l::Phase::Unknown);assert!(!t.effect_confirmed);
    expect(client.read(&window,true).await,l::Reason::Busy);assert!(inspection(&client).await.active);gate.notify_one();idle(&client).await;
    let repeated=client.scan(&window,&scope,"cancel-running").await.unwrap();assert_eq!(repeated.id,a.id);assert_eq!(repeated.phase,l::Phase::Unknown);
    assert_eq!(f.count("network.wifi.scan").await,1);assert!(client.view(&window).await.unwrap().stale);ex.shutdown().await;
}
#[tokio::test]
async fn native_window_owner_drop_is_not_delayed_by_worker_or_command_references() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());let f=Fixture::start(vec![step]);
    let(ex,client,_,owner,window)=launch(&f,l::Config::default());client.read(&window,true).await.unwrap();f.wait_requests(1).await;drop(owner);
    assert!(window.owner_gone());expect(client.view(&window).await,l::Reason::Closed);gate.notify_one();idle(&client).await;
    assert!(inspection(&client).await.association.is_none());ex.shutdown().await;
}
#[tokio::test]
async fn owner_drop_revokes_admissions_but_retains_singleton_until_actual_io_end() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::read();step.reply_gate=Some(gate.clone());let f=Fixture::start(vec![step]);
    let(ex,client,setting,_owner,window)=launch(&f,l::Config::default());client.read(&window,true).await.unwrap();f.wait_requests(1).await;drop(ex);
    tokio::time::timeout(Duration::from_secs(1),async{loop{if client.view(&window).await.err()==Some(l::Reason::Closed){break;}tokio::task::yield_now().await;}}).await.unwrap();
    expect(Executor::start(f.native.clone(),config(l::Config::default()),&setting),l::Reason::Busy);
    gate.notify_one();stopped().await;let next=Executor::start(f.native.clone(),config(l::Config::default()),&setting).unwrap();next.shutdown().await;
}
#[tokio::test]
async fn graceful_shutdown_waits_for_queued_scan_io_and_revokes_its_writer() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut scan=Step::scan();scan.greeting_gate=Some(gate.clone());let f=Fixture::start(vec![Step::read(),scan]);
    let(ex,client,_,_owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let scope=client.view(&window).await.unwrap().scan_scope.unwrap();client.scan(&window,&scope,"shutdown").await.unwrap();
    let shutdown=tokio::spawn(ex.shutdown());tokio::time::sleep(Duration::from_millis(10)).await;assert!(!shutdown.is_finished());assert!(SINGLETON.load(Ordering::Acquire));
    gate.notify_one();shutdown.await.unwrap();stopped().await;assert_eq!(f.count("network.wifi.scan").await,0);expect(client.view(&window).await,l::Reason::Closed);
}
#[tokio::test]
async fn singleton_and_invalid_constructor_fail_closed_without_daemon_io() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![]);let setting=AuthorityFence::from_backend(l::SystemSetting::from_backend(true));
    let mut bad=config(l::Config::default());bad.producer_nonce=0;expect(Executor::start(f.native.clone(),bad,&setting),l::Reason::Capacity);
    let mut bad=config(l::Config::default());bad.lifecycle.max_tickets=0;expect(Executor::start(f.native.clone(),bad,&setting),l::Reason::Capacity);assert!(!SINGLETON.load(Ordering::Acquire));
    let ex=Executor::start(f.native.clone(),config(l::Config::default()),&setting).unwrap();expect(Executor::start(f.native.clone(),config(l::Config::default()),&setting),l::Reason::Busy);
    assert_eq!(f.connections.load(Ordering::Relaxed),0);ex.shutdown().await;stopped().await;
}
#[tokio::test]
async fn hidden_wrong_role_disabled_and_foreign_settings_cannot_schedule_io() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![]);let(ex,client,setting,owner,window)=launch(&f,l::Config::default());
    owner.visibility_from_backend(false);expect(client.read(&window,true).await,l::Reason::Hidden);owner.visibility_from_backend(true);
    let wrong=NativeWindowOwner::from_backend(Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::Other,true,&setting.fixture_setting())),&setting);expect(client.read(&WindowRef::from_backend(&wrong),true).await,l::Reason::Unauthorized);
    let unrelated=l::SystemSetting::from_backend(true);let foreign_binding=AuthorityFence::from_backend(unrelated.clone());let foreign=NativeWindowOwner::from_backend(Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&unrelated)),&foreign_binding);expect(client.read(&WindowRef::from_backend(&foreign),true).await,l::Reason::Unauthorized);
    setting.update_from_backend(false);expect(client.read(&window,true).await,l::Reason::SystemDisabled);assert_eq!(f.connections.load(Ordering::Relaxed),0);ex.shutdown().await;
}
#[tokio::test]
async fn cached_reads_polling_and_idempotent_scan_intent_create_no_extra_io() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read(),Step::scan()]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let t=accepted_read(&client,&window).await;
    let before=client.view(&window).await.unwrap();for _ in 0..5 {assert_eq!(client.read(&window,false).await.unwrap().phase,l::Phase::Observed);client.ticket(&window,&t.id).await.unwrap();client.view(&window).await.unwrap();}
    assert_eq!(inspection(&client).await.reserved,1);let after=client.view(&window).await.unwrap();assert_eq!(serde_json::to_value(&before).unwrap()["data"],serde_json::to_value(&after).unwrap()["data"]);
    let scope=after.scan_scope.unwrap();let a=client.scan(&window,&scope,"same").await.unwrap();terminal(&client,&window,&a.id).await;
    for _ in 0..5{assert_eq!(client.scan(&window,&scope,"same").await.unwrap().id,a.id);}
    assert_eq!(f.count("network.getState").await,1);assert_eq!(f.count("network.wifi.scan").await,1);ex.shutdown().await;
}
#[tokio::test]
async fn bounded_channel_and_input_validation_never_admit_unbounded_payloads() {
    let native=l::SystemSetting::from_backend(true);let binding=AuthorityFence::from_backend(native.clone());let owner=NativeWindowOwner::from_backend(Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&native)),&binding);let window=WindowRef::from_backend(&owner);
    let(commands,receive)=mpsc::channel(1);let client=Client{commands,binding};let(response,_hold)=oneshot::channel();
    assert!(client.commands.try_send(Command::View{window:window.capture(&client.binding).unwrap(),response}).is_ok());expect(client.view(&window).await,l::Reason::Capacity);
    expect(client.scan(&window,&"s".repeat(129),"x").await,l::Reason::StaleTarget);expect(client.scan(&window,"scope",&"i".repeat(65)).await,l::Reason::InvalidIntent);
    expect(client.ticket(&window,&"r".repeat(129)).await,l::Reason::NotFound);drop(receive);expect(client.view(&window).await,l::Reason::Closed);
}
#[tokio::test]
async fn raw_native_identity_errors_and_payload_fields_stay_out_of_public_receipts() {
    let _serial=SERIAL.lock().await;stopped().await;let mut read=Step::read();read.result["wifiIP"]=json!("PRIVATE_IP");read.result["credentials"]=json!({"psk":"PRIVATE_PASSWORD"});
    let mut scan=Step::scan();scan.error=Some("PRIVATE_NATIVE_ERROR");let f=Fixture::start(vec![read,scan]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
    accepted_read(&client,&window).await;let v=client.view(&window).await.unwrap();let mut public=serde_json::to_string(&v).unwrap();let a=client.scan(&window,&v.scan_scope.unwrap(),"private").await.unwrap();
    let t=terminal(&client,&window,&a.id).await;assert_eq!(t.phase,l::Phase::Unknown);public.push_str(&serde_json::to_string(&t).unwrap());
    for text in ["PRIVATE_","mochi-executor-","trusted-executable","/org/freedesktop"] {assert!(!public.contains(text));}ex.shutdown().await;
}
#[test]
fn backend_producer_nonce_counter_is_unique_checked_and_has_no_fixture_pid_alias() {
    let mut a=ProducerCounter{nonce:77,next:0};let mut b=ProducerCounter{nonce:78,next:0};let first=a.reserve().unwrap();assert!(a.reserve().unwrap()!=first);assert!(b.reserve().unwrap()!=first);
    expect(ProducerCounter{nonce:0,next:0}.reserve(),l::Reason::Capacity);expect(ProducerCounter{nonce:77,next:u64::MAX}.reserve(),l::Reason::Capacity);
}
#[tokio::test]
async fn read_queued_before_native_hide_show_cannot_acquire_a_new_lease_after_reopen() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read()]);let(ex,client,_,owner,window)=launch(&f,l::Config::default());
    let mut request=Box::pin(client.read(&window,true));
    let mut context=std::task::Context::from_waker(std::task::Waker::noop());
    assert!(std::future::Future::poll(request.as_mut(),&mut context).is_pending());drop(context);
    // The actor has not been polled. The same real backend wrapper changes its
    // epoch twice before the previously queued command reaches admission.
    owner.visibility_from_backend(false);owner.visibility_from_backend(true);
    let result=request.await;let denied=result.err();
    if denied.is_none(){f.wait_requests(1).await;}
    ex.shutdown().await;stopped().await;
    assert_eq!(denied,Some(l::Reason::Revoked));assert_eq!(f.count("network.getState").await,0);
}
fn enqueue_once<F:std::future::Future>(request:std::pin::Pin<&mut F>) {
    let mut context=std::task::Context::from_waker(std::task::Waker::noop());
    assert!(std::future::Future::poll(request,&mut context).is_pending());
}
#[tokio::test]
async fn read_queued_before_global_disable_enable_requires_a_new_request_context() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read()]);let(ex,client,setting,_owner,window)=launch(&f,l::Config::default());
    let mut request=Box::pin(client.read(&window,true));enqueue_once(request.as_mut());setting.update_from_backend(false);setting.update_from_backend(true);
    expect(request.await,l::Reason::Revoked);assert_eq!(inspection(&client).await.reserved,0);assert_eq!(f.count("network.getState").await,0);
    accepted_read(&client,&window).await;assert_eq!(f.count("network.getState").await,1);ex.shutdown().await;
}
#[tokio::test]
async fn read_queued_before_route_round_trip_is_not_reminted_as_system_intent() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read()]);let(ex,client,_,owner,window)=launch(&f,l::Config::default());
    let mut request=Box::pin(client.read(&window,true));enqueue_once(request.as_mut());owner.presentation_from_backend(l::WindowKind::Other);owner.presentation_from_backend(l::WindowKind::System);
    expect(request.await,l::Reason::Revoked);assert_eq!(inspection(&client).await.reserved,0);assert_eq!(f.count("network.getState").await,0);
    accepted_read(&client,&window).await;ex.shutdown().await;
}
#[tokio::test]
async fn queued_scan_does_not_survive_hide_show_but_new_explicit_context_can_scan() {
    let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::read(),Step::scan()]);let(ex,client,_,owner,window)=launch(&f,l::Config::default());
    accepted_read(&client,&window).await;let old=client.view(&window).await.unwrap().scan_scope.unwrap();
    let mut request=Box::pin(client.scan(&window,&old,"queued-old"));enqueue_once(request.as_mut());owner.visibility_from_backend(false);owner.visibility_from_backend(true);
    expect(request.await,l::Reason::Revoked);assert_eq!(f.count("network.wifi.scan").await,0);
    let current=client.view(&window).await.unwrap().scan_scope.unwrap();assert_ne!(old,current);
    let a=client.scan(&window,&current,"new-explicit").await.unwrap();assert_eq!(terminal(&client,&window,&a.id).await.phase,l::Phase::Acknowledged);ex.shutdown().await;
}
#[tokio::test]
async fn old_read_completion_after_hide_show_cannot_publish_cache_or_origin() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut read=Step::read();read.reply_gate=Some(gate.clone());let f=Fixture::start(vec![read]);
    let(ex,client,_,owner,window)=launch(&f,l::Config::default());let a=client.read(&window,true).await.unwrap();f.wait_requests(1).await;
    owner.visibility_from_backend(false);owner.visibility_from_backend(true);expect(client.read(&window,true).await,l::Reason::Busy);
    gate.notify_one();idle(&client).await;let t=client.ticket(&window,&a.id).await.unwrap();assert_eq!((t.phase,t.reason),(l::Phase::Unknown,Some(l::Reason::Revoked)));
    assert!(inspection(&client).await.association.is_none());assert!(client.view(&window).await.unwrap().data.is_none());ex.shutdown().await;
}
#[tokio::test]
async fn scan_guard_rejects_context_revoked_during_native_greeting_wait() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut scan=Step::scan();scan.greeting_gate=Some(gate.clone());let f=Fixture::start(vec![Step::read(),scan]);
    let(ex,client,_,owner,window)=launch(&f,l::Config::default());accepted_read(&client,&window).await;let scope=client.view(&window).await.unwrap().scan_scope.unwrap();
    let a=client.scan(&window,&scope,"old-writer").await.unwrap();owner.visibility_from_backend(false);owner.visibility_from_backend(true);gate.notify_one();idle(&client).await;
    let t=client.ticket(&window,&a.id).await.unwrap();assert_eq!((t.phase,t.reason),(l::Phase::Failed,Some(l::Reason::Revoked)));
    assert_eq!(f.count("network.wifi.scan").await,0);ex.shutdown().await;
}
#[tokio::test]
async fn global_setting_round_trip_during_io_discards_late_read_publication() {
    let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut read=Step::read();read.reply_gate=Some(gate.clone());let f=Fixture::start(vec![read]);
    let(ex,client,setting,_owner,window)=launch(&f,l::Config::default());let a=client.read(&window,true).await.unwrap();f.wait_requests(1).await;
    setting.update_from_backend(false);setting.update_from_backend(true);gate.notify_one();idle(&client).await;
    let t=client.ticket(&window,&a.id).await.unwrap();assert_eq!(t.reason,Some(l::Reason::Revoked));assert!(inspection(&client).await.association.is_none());ex.shutdown().await;
}

// NEW native binding cumulative regression overlay; frozen R1 tests above exact.
#[tokio::test]
async fn binding_prepared_before_await_then_hide_show_late_enqueue_cannot_resample(){
    let _serial=SERIAL.lock().await;
    let f=Fixture::start(vec![]);
    let manager=crate::pet_native_binding::core::Manager::new(true);
    let core=crate::pet_native_binding::core::fixture_register(&manager);
    core.document(false,true).unwrap();core.observe_native(crate::pet_native_binding::core::Facts{visible:true,mapped:true}).unwrap();
    let(plan,_)=core.begin_trusted(crate::pet_native_binding::ipc::Mode::Expanded{route:crate::pet_native_binding::ipc::Route::System}).unwrap();core.finish(&plan,true).unwrap();
    let owner=Executor::start(f.native.clone(),config(l::Config::default()),&manager.binding).unwrap();let client=owner.client();
    // SYNC resolution captures original stamp, before any channel wait/poll.
    let prepared=core.prepare(&client,core.context().unwrap().revision).unwrap();
    tokio::task::yield_now().await;
    core.observe_native(crate::pet_native_binding::core::Facts{visible:false,mapped:false}).unwrap();
    core.observe_native(crate::pet_native_binding::core::Facts{visible:true,mapped:true}).unwrap();
    assert!(core.context().unwrap().system_eligible);
    let response=client.enqueue_prepared_read(prepared.into_prepared(),true).unwrap().wait().await;
    expect(response,l::Reason::Revoked);
    assert_eq!(f.connections.load(Ordering::Relaxed),0);assert_eq!(f.count("network.getState").await,0);
    // A separately resolved CURRENT request may inspect state, but does not IO.
    let prepared=core.prepare(&client,core.context().unwrap().revision).unwrap();
    assert!(client.enqueue_prepared_view(prepared.into_prepared()).unwrap().wait().await.is_ok());
    owner.shutdown().await;stopped().await;
}
#[tokio::test]
async fn binding_native_hide_close_reload_crash_during_real_r3_scan_greeting_rejects_writer(){
    let _serial=SERIAL.lock().await;
    for action in 0..4 {
        let greeting=Arc::new(Notify::new());let mut step=Step::scan();step.greeting_gate=Some(greeting.clone());let f=Fixture::start(vec![Step::read(),step]);
        let manager=crate::pet_native_binding::core::Manager::new(true);let core=crate::pet_native_binding::core::fixture_register(&manager);
        core.document(false,true).unwrap();core.observe_native(crate::pet_native_binding::core::Facts{visible:true,mapped:true}).unwrap();
        let(plan,_)=core.begin_trusted(crate::pet_native_binding::ipc::Mode::Expanded{route:crate::pet_native_binding::ipc::Route::System}).unwrap();core.finish(&plan,true).unwrap();
        let owner=Executor::start(f.native.clone(),config(l::Config::default()),&manager.binding).unwrap();let client=owner.client();
        let p=core.prepare(&client,core.context().unwrap().revision).unwrap();client.enqueue_prepared_read(p.into_prepared(),true).unwrap().wait().await.unwrap();idle(&client).await;
        let p=core.prepare(&client,core.context().unwrap().revision).unwrap();let scope=client.enqueue_prepared_view(p.into_prepared()).unwrap().wait().await.unwrap().scan_scope.unwrap();
        let p=core.prepare(&client,core.context().unwrap().revision).unwrap();client.enqueue_prepared_scan(p.into_prepared(),&scope,"native-writer").unwrap().wait().await.unwrap();
        tokio::time::timeout(Duration::from_secs(3),async{while f.connections.load(Ordering::Relaxed)<2{tokio::time::sleep(Duration::from_millis(1)).await;}}).await.unwrap();
        match action{0=>{core.observe_native(crate::pet_native_binding::core::Facts{visible:false,mapped:false}).unwrap();},1=>{core.close().unwrap();},2=>{core.document(true,false).unwrap();},_=>{core.crashed().unwrap();}}
        greeting.notify_one();idle(&client).await;assert_eq!(f.count("network.wifi.scan").await,0);assert_eq!(f.count("network.getState").await,1);
        owner.shutdown().await;stopped().await;
    }
}

include!("enable_executor_tests.rs");

include!("disable_executor_tests.rs");
#[cfg(target_os="linux")]#[path="system_io_executor_tests.rs"]mod system_io_tests;
