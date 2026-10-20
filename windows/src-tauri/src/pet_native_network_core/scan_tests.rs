use super::*;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;

struct Step {
    greeting: Vec<u8>,
    response: Vec<u8>,
    before_greeting: Option<Box<dyn FnOnce(&std::path::Path) + Send>>,
    before_reply: Option<Box<dyn FnOnce(&std::path::Path) + Send>>,
    delay: Duration,
}
fn greet(version: Value, caps: Value) -> Vec<u8> {
    format!("{}\n",json!({"apiVersion":version,"capabilities":caps})).into_bytes()
}
fn normal_greeting() -> Vec<u8> { greet(json!(34),json!(["network"])) }
fn response(result: Value) -> Vec<u8> {format!("{{\"id\":REQUEST_ID,\"result\":{result}}}\n").into_bytes()}
impl Step {
    fn read() -> Self { Self {greeting:normal_greeting(),response:response(json!({"wifiEnabled":true,"wifiNetworks":[]})),before_greeting:None,before_reply:None,delay:Duration::ZERO} }
    fn scan() -> Self { Self {response:response(json!({"success":true,"message":"scanning"})),..Self::read()} }
}
struct Fixture {
    root: PathBuf,
    config: Config,
    requests: Arc<Mutex<Vec<Value>>>,
    connections: Arc<AtomicU64>,
    server: tokio::task::JoinHandle<()>,
}
impl Fixture {
    fn start(steps: Vec<Step>) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root=std::env::temp_dir().join(format!("mochi-network-r3-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        std::fs::create_dir(&root).unwrap();std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket=root.join("danklinux-123.sock");let listener=UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
        let executable=root.join("trusted-executable");symlink(std::env::current_exe().unwrap(),&executable).unwrap();
        let config=Config {socket,runtime:root.clone(),uid:unsafe{libc::geteuid()},trusted_executable:executable,trusted_executable_uid:unsafe{libc::geteuid()},deadline:Duration::from_secs(2)};
        let requests=Arc::new(Mutex::new(Vec::new()));let captured=requests.clone();let directory=root.clone();
        let connections=Arc::new(AtomicU64::new(0));let count=connections.clone();
        let server=tokio::spawn(async move {
            for step in steps {
                let Ok((mut stream,_))=listener.accept().await else {return;};count.fetch_add(1,Ordering::Relaxed);
                if let Some(f)=step.before_greeting {f(&directory);}
                if stream.write_all(&step.greeting).await.is_err(){continue;}
                let mut data=Vec::new();let mut buffer=[0u8;1024];
                while !data.contains(&b'\n') {
                    let Ok(n)=stream.read(&mut buffer).await else {break;};if n==0 {break;}
                    data.extend_from_slice(&buffer[..n]);assert!(data.len()<8192);
                }
                if data.is_empty(){continue;}
                let req:Value=serde_json::from_slice(data.split(|b|*b==b'\n').next().unwrap()).unwrap();
                captured.lock().unwrap().push(req.clone());
                if let Some(f)=step.before_reply {f(&directory);}
                tokio::time::sleep(step.delay).await;
                let bytes=String::from_utf8(step.response).unwrap().replace("REQUEST_ID",&req["id"].to_string());
                let _=stream.write_all(bytes.as_bytes()).await;
            }
        });
        Self {root,config,requests,connections,server}
    }
    async fn origin(&self) -> Origin {network_snapshot_guarded(&self.config,||Ok::<_,()>(())).await.unwrap().origin().clone()}
    async fn scans(&self) -> usize {tokio::task::yield_now().await;self.requests.lock().unwrap().iter().filter(|v|v["method"]=="network.wifi.scan").count()}
}
impl Drop for Fixture {fn drop(&mut self){self.server.abort();let _=std::fs::remove_dir_all(&self.root);}}
fn native(result:Result<ScanAck,Failure<()>>,reason:Error,state:WriteState) {
    match result {Err(Failure::Native{reason:r,write})=>assert_eq!((r,write),(reason,state)),_=>panic!("unexpected typed scan outcome")}
}

#[tokio::test]
async fn exact_global_scan_uses_same_read_origin_and_one_writer_guard() {
    let f=Fixture::start(vec![Step::read(),Step::scan()]);let origin=f.origin().await;let mut checks=0;
    let ack=scan_wifi(&f.config,&origin,|fresh|{assert!(fresh==&origin);checks+=1;Ok::<_,()>(())}).await.unwrap();
    assert!(ack.origin()==&origin);assert_eq!(checks,1);assert_eq!(f.scans().await,1);
    let requests=f.requests.lock().unwrap();assert_eq!(requests.len(),2);
    assert_eq!(requests[0]["method"],"network.getState");assert_eq!(requests[1]["method"],"network.wifi.scan");
    for r in requests.iter(){assert_eq!(r.as_object().unwrap().len(),2);assert!(r.get("params").is_none());}
}
#[tokio::test]
async fn read_guard_denial_never_writes_or_yields_an_origin() {
    let f=Fixture::start(vec![Step::read()]);
    let result=network_snapshot_guarded(&f.config,||Err("revoked")).await;
    assert!(matches!(result,Err(Failure::Guard("revoked"))));assert!(f.requests.lock().unwrap().is_empty());
}
#[tokio::test]
async fn scan_guard_denial_after_current_greeting_writes_zero_bytes() {
    let f=Fixture::start(vec![Step::read(),Step::scan()]);let origin=f.origin().await;let mut calls=0;
    let result=scan_wifi(&f.config,&origin,|_|{calls+=1;Err("hidden")}).await;
    assert!(matches!(result,Err(Failure::Guard("hidden"))));assert_eq!(calls,1);assert_eq!(f.scans().await,0);
}
#[tokio::test]
async fn slow_synchronous_guard_cannot_write_after_total_deadline() {
    let mut f=Fixture::start(vec![Step::read(),Step::scan()]);let origin=f.origin().await;f.config.deadline=Duration::from_millis(20);
    let result=scan_wifi(&f.config,&origin,|_|{std::thread::sleep(Duration::from_millis(50));Ok::<_,()>(())}).await;
    native(result,Error::Timeout,WriteState::NotStarted);assert_eq!(f.scans().await,0);
}
#[tokio::test]
async fn every_unreviewed_version_or_missing_network_capability_is_read_only() {
    for (version,caps) in [(json!(0),json!(["network"])),(json!(29),json!(["network"])),(json!(33),json!(["network"])),(json!(35),json!(["network"])),(json!(u64::MAX),json!(["network"])),(json!(34),json!([])),(json!(34),json!(["NETWORK"]))] {
        let mut step=Step::scan();step.greeting=greet(version,caps);let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
        native(scan_wifi(&f.config,&origin,|_|panic!("guard before proof")).await,Error::Unsupported,WriteState::NotStarted);
        assert_eq!(f.scans().await,0);
    }
}
#[tokio::test]
async fn malformed_handshakes_are_rejected_before_guard_or_scan_write() {
    for greeting in [greet(json!("34"),json!(["network"])),greet(json!(34),json!([true])),greet(Value::Null,json!(["network"])),
        b"not-json\n".to_vec(),b"{}\n".to_vec(),b"{\"id\":null,\"apiVersion\":34,\"capabilities\":[\"network\"]}\n".to_vec(),
        greet(json!(34),json!(["x".repeat(129)])),greet(json!(34),json!(vec!["network";257]))] {
        let mut step=Step::scan();step.greeting=greeting;let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
        native(scan_wifi(&f.config,&origin,|_|panic!("guard before proof")).await,Error::Protocol,WriteState::NotStarted);
        assert_eq!(f.scans().await,0);
    }
}
#[tokio::test]
async fn absent_greeting_times_out_without_a_scan_request_or_retry() {
    let mut step=Step::scan();step.greeting.clear();let mut f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    f.config.deadline=Duration::from_millis(100);
    native(scan_wifi(&f.config,&origin,|_|panic!("no greeting")).await,Error::Timeout,WriteState::NotStarted);
    assert_eq!(f.scans().await,0);assert_eq!(f.connections.load(Ordering::Relaxed),2);
}
#[tokio::test]
async fn a_read_without_reviewed_protocol_proof_cannot_be_used_to_scan() {
    for greeting in [greet(json!(29),json!(["network"])),greet(json!(34),json!([])),Vec::new()] {
        let mut step=Step::read();step.greeting=greeting;let f=Fixture::start(vec![step]);
        let reply=network_snapshot_guarded(&f.config,||Ok::<_,()>(())).await.unwrap();assert!(!reply.supports_scan());
        native(scan_wifi(&f.config,reply.origin(),|_|panic!("unproven scan")).await,Error::Unsupported,WriteState::NotStarted);
        assert_eq!(f.connections.load(Ordering::Relaxed),1);assert_eq!(f.scans().await,0);
    }
}
#[tokio::test]
async fn each_native_source_incarnation_field_is_checked_without_pid_only_aliases() {
    let mut steps=vec![Step::read()];for _ in 0..8{steps.push(Step::scan());}let f=Fixture::start(steps);let origin=f.origin().await;
    for i in 0..8 {let mut wrong=origin.clone();match i {
        0=>wrong.source.uid=wrong.source.uid.saturating_add(1),1=>wrong.source.peer.pid+=1,2=>wrong.source.peer.start+=1,
        3=>wrong.source.peer.device+=1,4=>wrong.source.peer.inode+=1,5=>wrong.source.endpoint.inode+=1,
        6=>wrong.source.endpoint.ctime_ns+=1,_=>wrong.source.socket=PathBuf::from("/untrusted/other.sock")};
        native(scan_wifi(&f.config,&wrong,|_|panic!("foreign origin guard")).await,Error::OriginChanged,WriteState::NotStarted);
    }
    assert_eq!(f.scans().await,0);
}
#[tokio::test]
async fn another_endpoint_of_the_same_fixture_process_is_not_the_expected_origin() {
    let f=Fixture::start(vec![Step::read()]);let origin=f.origin().await;let other=Fixture::start(vec![Step::scan()]);
    native(scan_wifi(&other.config,&origin,|_|panic!("foreign endpoint")).await,Error::OriginChanged,WriteState::NotStarted);
    assert_eq!(other.scans().await,0);
}
#[tokio::test]
async fn trusted_executable_change_while_waiting_greeting_is_rechecked_before_write() {
    let mut step=Step::scan();step.before_greeting=Some(Box::new(|dir|{
        let path=dir.join("trusted-executable");std::fs::remove_file(&path).unwrap();symlink("/usr/bin/true",path).unwrap();
    }));let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    native(scan_wifi(&f.config,&origin,|_|panic!("changed native peer")).await,Error::UntrustedPeer,WriteState::NotStarted);
    assert_eq!(f.scans().await,0);
}
#[tokio::test]
async fn already_prefetched_reply_cannot_ack_a_request_that_was_not_sent() {
    let mut step=Step::scan();step.greeting.extend_from_slice(b"{\"id\":1,\"result\":{\"success\":true,\"message\":\"scanning\"}}\n");
    let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    native(scan_wifi(&f.config,&origin,|_|panic!("prefetched response")).await,Error::Protocol,WriteState::NotStarted);
    assert_eq!(f.scans().await,0);
}
#[tokio::test]
async fn only_the_proven_correlated_success_shape_is_acknowledged() {
    for result in [Value::Null,json!({}),json!({"success":false,"message":"scanning"}),json!({"success":"true","message":"scanning"}),
        json!({"success":true,"message":"complete"}),json!({"success":true,"message":"scanning","PRIVATE":"native raw"})] {
        let mut step=Step::scan();step.response=response(result);let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
        native(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::Protocol,WriteState::MayHaveStarted);
        assert_eq!(f.scans().await,1);
    }
}
#[tokio::test]
async fn wrong_id_does_not_supply_ack_and_later_matching_id_can_ack() {
    let mut step=Step::scan();step.response=b"{\"id\":\"other\",\"result\":{\"success\":true,\"message\":\"scanning\"}}\n".to_vec();
    step.response.extend(response(json!({"success":true,"message":"scanning"})));
    let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    assert!(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await.is_ok());assert_eq!(f.scans().await,1);
}
#[tokio::test]
async fn raw_native_error_is_static_unknown_after_write_and_never_retried() {
    let mut step=Step::scan();step.response=b"{\"id\":REQUEST_ID,\"error\":\"PRIVATE_DEVICE_PASSWORD\"}\n".to_vec();
    let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;let error=scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await.err().unwrap();
    assert!(!format!("{error:?}").contains("PRIVATE"));assert_eq!(error.write_state(),WriteState::MayHaveStarted);
    assert_eq!(error,Failure::Native{reason:Error::NativeFailure,write:WriteState::MayHaveStarted});assert_eq!(f.scans().await,1);
}
#[tokio::test]
async fn timeout_after_request_is_unknown_and_does_not_trigger_a_second_action() {
    let mut step=Step::scan();step.delay=Duration::from_secs(2);let mut f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    f.config.deadline=Duration::from_millis(100);
    native(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::Timeout,WriteState::MayHaveStarted);
    assert_eq!(f.scans().await,1);assert_eq!(f.connections.load(Ordering::Relaxed),2);
}
#[tokio::test]
async fn post_write_source_change_cannot_confirm_the_action() {
    let mut step=Step::scan();step.before_reply=Some(Box::new(|dir|{
        let path=dir.join("trusted-executable");std::fs::remove_file(&path).unwrap();symlink("/usr/bin/true",path).unwrap();
    }));let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    native(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::UntrustedPeer,WriteState::MayHaveStarted);assert_eq!(f.scans().await,1);
}
#[tokio::test]
async fn greeting_and_ack_share_one_bounded_byte_budget() {
    let mut step=Step::scan();step.greeting=format!("{}\n",json!({"apiVersion":34,"capabilities":["network"],"padding":"x".repeat(MAX_BYTES-70)})).into_bytes();
    let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    native(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::Protocol,WriteState::MayHaveStarted);
}
#[tokio::test]
async fn unexpected_handshake_after_dispatch_cannot_certify_scan_completion() {
    let mut step=Step::scan();step.response=normal_greeting();step.response.extend(response(json!({"success":true,"message":"scanning"})));
    let f=Fixture::start(vec![Step::read(),step]);let origin=f.origin().await;
    native(scan_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::Protocol,WriteState::MayHaveStarted);
}

#[test]
fn request_ids_fail_before_json_float_aliasing_or_counter_wrap() {
    let counter=AtomicU64::new(9_007_199_254_740_990);assert_eq!(next_request_id(&counter),Ok(9_007_199_254_740_990));
    assert_eq!(next_request_id(&counter),Err(Error::Protocol));assert_eq!(next_request_id(&AtomicU64::new(0)),Err(Error::Protocol));
    assert_eq!(next_request_id(&AtomicU64::new(u64::MAX)),Err(Error::Protocol));
}
struct PartialWriter { bytes: Vec<u8> }
impl AsyncWrite for PartialWriter {
    fn poll_write(mut self:std::pin::Pin<&mut Self>,_:&mut std::task::Context<'_>,data:&[u8])->std::task::Poll<std::io::Result<usize>> {
        if self.bytes.is_empty(){self.bytes.extend_from_slice(&data[..2]);std::task::Poll::Ready(Ok(2))}
        else{std::task::Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)))}
    }
    fn poll_flush(self:std::pin::Pin<&mut Self>,_:&mut std::task::Context<'_>)->std::task::Poll<std::io::Result<()>>{std::task::Poll::Ready(Ok(()))}
    fn poll_shutdown(self:std::pin::Pin<&mut Self>,_:&mut std::task::Context<'_>)->std::task::Poll<std::io::Result<()>>{std::task::Poll::Ready(Ok(()))}
}
#[tokio::test]
async fn partially_written_frame_is_never_reported_as_not_started() {
    let mut writer=PartialWriter{bytes:Vec::new()};let mut state=WriteState::NotStarted;
    assert_eq!(write_frame(&mut writer,b"fixed-request\n",&mut state).await,Err(Error::Unavailable));
    assert_eq!(writer.bytes,b"fi");assert_eq!(state,WriteState::MayHaveStarted);
}
#[tokio::test]
async fn already_ready_late_ack_is_rejected_by_actual_final_gate() {
    let(mut client,mut server)=UnixStream::pair().unwrap();server.write_all(b"{\"id\":321,\"result\":{\"success\":true,\"message\":\"scanning\"}}\n").await.unwrap();
    client.readable().await.unwrap();let deadline=Instant::now()+Duration::from_millis(20);std::thread::sleep(Duration::from_millis(50));
    let mut reader=ScanReader::default();let ready=timeout_at(deadline,reader.ack(&mut client,321)).await.unwrap();assert!(ready.is_ok());
    let result=finish_result(ready.map_err(|e|failure::<()>(e,WriteState::MayHaveStarted)),deadline,WriteState::MayHaveStarted);
    assert_eq!(result,Err(Failure::Native{reason:Error::Timeout,write:WriteState::MayHaveStarted}));
}

#[tokio::test]
async fn composed_real_lifecycle_read_and_scan_keep_one_worker_and_ack_only_receipt() {
    use crate::lifecycle as l;
    let f=Fixture::start(vec![Step::read(),Step::scan()]);let setting=l::SystemSetting::from_backend(true);
    let window=l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&setting);let service=l::Service::new(l::Config::default(),99,&setting).unwrap();
    let producer=l::ProducerId::from_verified_backend([1;16]).unwrap();let a=service.read(&window,true,std::time::Instant::now()).unwrap();let mut work=a.work.unwrap();
    let reply=network_snapshot_guarded(&f.config,||work.begin_dispatch(producer,std::time::Instant::now()).map(|op|assert_eq!(op,l::Operation::ReadNetwork))).await.unwrap();
    let origin=reply.origin().clone();assert!(reply.supports_scan());
    work.finish_read(l::NativeObservation{producer,api_version:reply.api_version,capabilities:reply.capabilities,result:reply.result,targets:vec![],scan_support:l::ScanSupport::VerifiedByAdapter,observed_at_ms:42},std::time::Instant::now()).unwrap();
    let scope=service.view(&window,std::time::Instant::now()).unwrap().scan_scope.unwrap();let a=service.scan(&window,&scope,"composed",std::time::Instant::now()).unwrap();let mut work=a.work.unwrap();
    let ack=scan_wifi(&f.config,&origin,|fresh|{assert!(fresh==&origin);work.begin_dispatch(producer,std::time::Instant::now()).map(|op|assert_eq!(op,l::Operation::ScanWifi))}).await.unwrap();assert!(ack.origin()==&origin);
    work.finish_scan(producer,std::time::Instant::now()).unwrap();let ticket=service.ticket(&window,&a.ticket.id,std::time::Instant::now()).unwrap();
    assert_eq!(ticket.phase,l::Phase::Acknowledged);assert!(!ticket.effect_confirmed);assert!(ticket.refresh_recommended);assert!(service.view(&window,std::time::Instant::now()).unwrap().stale);
    assert_eq!(f.scans().await,1);
}
#[tokio::test]
async fn composed_window_revoked_during_greeting_is_denied_at_actual_writer() {
    use crate::lifecycle as l;
    let setting=l::SystemSetting::from_backend(true);let window=Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&setting));
    let hidden=window.clone();let mut step=Step::scan();step.before_greeting=Some(Box::new(move |_|hidden.visibility_from_backend(false)));
    let f=Fixture::start(vec![Step::read(),step]);let service=l::Service::new(l::Config::default(),99,&setting).unwrap();let producer=l::ProducerId::from_verified_backend([1;16]).unwrap();
    let mut work=service.read(&window,true,std::time::Instant::now()).unwrap().work.unwrap();
    let reply=network_snapshot_guarded(&f.config,||work.begin_dispatch(producer,std::time::Instant::now()).map(|_|())).await.unwrap();let origin=reply.origin().clone();
    work.finish_read(l::NativeObservation{producer,api_version:reply.api_version,capabilities:reply.capabilities,result:reply.result,targets:vec![],scan_support:l::ScanSupport::VerifiedByAdapter,observed_at_ms:42},std::time::Instant::now()).unwrap();
    let scope=service.view(&window,std::time::Instant::now()).unwrap().scan_scope.unwrap();let mut work=service.scan(&window,&scope,"hide",std::time::Instant::now()).unwrap().work.unwrap();
    let result=scan_wifi(&f.config,&origin,|_|work.begin_dispatch(producer,std::time::Instant::now()).map(|_|())).await;
    assert!(matches!(result,Err(Failure::Guard(l::Reason::Unauthorized))));drop(work);assert_eq!(f.scans().await,0);
}
