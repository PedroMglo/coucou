//! Shared read recovery on synthetic local sockets. No real DMS is invoked.
use super::*;

fn discovery_for(f: &mut Fixture, endpoint: &std::path::Path) -> (PathBuf, PathBuf) {
    let target = f.root.join("metadata-target"); let count = f.root.join("metadata-count");
    std::fs::write(&target, endpoint.to_str().unwrap()).unwrap();
    let program = f.root.join("fixture-native-metadata");
    let body = format!("#!/usr/bin/python3 -I\nimport json,pathlib,sys\nassert sys.argv[1:]==['ipc','call','pets-system','backendEnvironment']\np=pathlib.Path({count})\np.write_text(str(int(p.read_text())+1) if p.exists() else '1')\nprint(json.dumps({{'schemaVersion':1,'dmsSocket':pathlib.Path({target}).read_text()}}))\n",
        count=json!(count.to_str().unwrap()), target=json!(target.to_str().unwrap()));
    std::fs::write(&program, body).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    f.config.discovery = Some(discovery::Native::fixture(program, unsafe { libc::geteuid() }));
    (target, count)
}
fn calls(path: &std::path::Path) -> u32 {
    std::fs::read_to_string(path).ok().and_then(|s| s.parse().ok()).unwrap_or(0)
}
async fn action(rpc: Rpc, f: &Fixture, origin: &Origin) -> Result<(), Failure<()>> {
    match rpc {
        Rpc::Scan => scan_wifi(&f.config, origin, |_| Ok::<_, ()>(())).await.map(|_| ()),
        Rpc::EnableWifi => enable_wifi(&f.config, origin, |_| Ok::<_, ()>(())).await.map(|_| ()),
        Rpc::DisableWifi => disable_wifi(&f.config, origin, |_| Ok::<_, ()>(())).await.map(|_| ()),
        Rpc::Read => panic!("action expected"),
    }
}

#[tokio::test]
async fn recovery_missing_and_stale_inherited_endpoint_resolve_once_on_current_read() {
    for absent in [false, true] {
        let mut f = Fixture::start(vec![Step::enable_read()]); let actual = f.config.socket.clone();
        let (_, count) = discovery_for(&mut f, &actual);
        f.config.socket = if absent { PathBuf::new() } else { f.root.join("dms-999.sock") };
        let mut guards = 0;
        let read = network_snapshot_guarded(&f.config, || { guards += 1; Ok::<_, ()>(()) }).await.unwrap();
        assert_eq!(guards, 1); assert_eq!(calls(&count), 1); assert_eq!(read.origin().source.socket, actual);
        assert!(read.supports_enable_wifi()); assert_eq!(f.requests.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn recovery_authenticated_path_cache_is_shared_by_config_clones_and_bluetooth_reads() {
    let mut f = Fixture::start(vec![Step::enable_read(), Step::read()]); let actual = f.config.socket.clone();
    let (_, count) = discovery_for(&mut f, &actual); f.config.socket = PathBuf::new();
    network_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await.unwrap();
    let next = f.config.clone();
    bluetooth_snapshot_guarded(&next, || Ok::<_, ()>(())).await.unwrap();
    assert_eq!(calls(&count), 1);
    let requests = f.requests.lock().unwrap(); assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["method"], "network.getState"); assert_eq!(requests[1]["method"], "bluetooth.getState");
}

#[tokio::test]
async fn recovery_bluetooth_can_be_the_first_scoped_reader_of_a_new_endpoint() {
    let mut f = Fixture::start(vec![Step::read()]); let actual = f.config.socket.clone();
    let (_, count) = discovery_for(&mut f, &actual); f.config.socket = f.root.join("dms-999.sock");
    let mut guards = 0;
    bluetooth_snapshot_guarded(&f.config, || { guards += 1; Ok::<_, ()>(()) }).await.unwrap();
    assert_eq!(guards, 1); assert_eq!(calls(&count), 1);
    assert_eq!(f.requests.lock().unwrap()[0]["method"], "bluetooth.getState");
}

#[tokio::test]
async fn recovery_actions_use_the_completed_read_origin_without_rediscovery() {
    for rpc in [Rpc::Scan, Rpc::EnableWifi, Rpc::DisableWifi] {
        let ack = match rpc { Rpc::Scan => Step::scan(), Rpc::EnableWifi => Step::enable_ack(),
            Rpc::DisableWifi => Step::disable_ack(), Rpc::Read => unreachable!() };
        let mut f = Fixture::start(vec![Step::enable_read(), ack]); let actual = f.config.socket.clone();
        let (_, count) = discovery_for(&mut f, &actual); f.config.socket = f.root.join("dms-999.sock");
        let origin = f.origin().await; action(rpc, &f, &origin).await.unwrap();
        assert_eq!(calls(&count), 1); assert_eq!(f.requests.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn recovery_scope_revoked_during_resolution_still_writes_no_domain_request() {
    for bluetooth in [false, true] {
        let mut f = Fixture::start(vec![Step::read()]); let actual = f.config.socket.clone();
        let (_, count) = discovery_for(&mut f, &actual); f.config.socket = PathBuf::new();
        let result = if bluetooth {
            bluetooth_snapshot_guarded(&f.config, || Err("revoked")).await.map(|_| ())
        } else { network_snapshot_guarded(&f.config, || Err("revoked")).await.map(|_| ()) };
        assert_eq!(result, Err(Failure::Guard("revoked"))); assert_eq!(calls(&count), 1);
        assert!(f.requests.lock().unwrap().is_empty());
        assert!(f.config.discovery.as_ref().unwrap().remembered().is_none());
    }
}

#[tokio::test]
async fn recovery_does_not_hide_an_untrusted_socket_or_runtime_with_metadata_fallback() {
    for runtime in [false, true] {
        let mut f = Fixture::start(vec![Step::read()]); let actual = f.config.socket.clone();
        let (_, count) = discovery_for(&mut f, &actual);
        if runtime { std::fs::set_permissions(&f.root, std::fs::Permissions::from_mode(0o755)).unwrap(); }
        else {
            let alias = f.root.join("dms-999.sock"); symlink(&actual, &alias).unwrap(); f.config.socket = alias;
        }
        let result = network_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await;
        assert!(matches!(result, Err(Failure::Native { reason:Error::UntrustedPeer,write:WriteState::NotStarted })));
        assert_eq!(calls(&count), 0); assert!(f.requests.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn recovery_lost_read_response_never_causes_rediscovery_or_a_second_query() {
    let mut f = Fixture::start(vec![Step { response:Vec::new(), ..Step::read() }]);
    let actual = f.config.socket.clone(); let (_, count) = discovery_for(&mut f, &actual);
    let result = network_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await;
    assert!(matches!(result, Err(Failure::Native { reason:Error::Unavailable,write:WriteState::MayHaveStarted })));
    assert_eq!(calls(&count), 0); assert_eq!(f.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn recovery_new_endpoint_needs_a_fresh_read_and_old_origins_cannot_follow_its_cache() {
    let mut f = Fixture::start(vec![Step::enable_read()]); let actual = f.config.socket.clone();
    let (target, count) = discovery_for(&mut f, &actual); f.config.socket = PathBuf::new();
    let old = f.origin().await;
    f.server.abort(); let _ = (&mut f.server).await; std::fs::remove_file(&actual).unwrap();
    let next = f.root.join("dms-456.sock"); let listener = UnixListener::bind(&next).unwrap();
    std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&target, next.to_str().unwrap()).unwrap();
    let received = Arc::new(AtomicU64::new(0)); let tally = received.clone();
    let server = tokio::spawn(async move {
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().await.unwrap(); stream.write_all(&normal_greeting()).await.unwrap();
            let mut data = Vec::new(); let mut buf = [0;1024];
            while !data.contains(&b'\n') { let n=stream.read(&mut buf).await.unwrap(); if n==0{return;} data.extend_from_slice(&buf[..n]); assert!(data.len()<8192); }
            let request:Value=serde_json::from_slice(data.split(|b|*b==b'\n').next().unwrap()).unwrap();
            tally.fetch_add(1,Ordering::Relaxed);
            let result = match request["method"].as_str().unwrap() {
                "network.getState" => enable_raw(false), "network.wifi.scan" => json!({"success":true,"message":"scanning"}),
                "network.wifi.enable" => json!({"enabled":true}), "network.wifi.disable" => json!({"enabled":false}),
                _ => panic!("unexpected method"),
            };
            stream.write_all(format!("{}\n",json!({"id":request["id"],"result":result})).as_bytes()).await.unwrap();
        }
    });
    for rpc in [Rpc::Scan, Rpc::EnableWifi, Rpc::DisableWifi] {
        assert!(matches!(action(rpc,&f,&old).await, Err(Failure::Native {write:WriteState::NotStarted,..})));
    }
    assert_eq!(calls(&count),1); assert_eq!(received.load(Ordering::Relaxed),0);
    let fresh=f.origin().await; assert!(fresh!=old); assert_eq!(fresh.source.socket,next);
    assert_eq!(calls(&count),2);
    // Even after the new path is cached, the old proof does not retarget.
    assert!(matches!(action(Rpc::EnableWifi,&f,&old).await, Err(Failure::Native {write:WriteState::NotStarted,..})));
    for rpc in [Rpc::Scan,Rpc::EnableWifi,Rpc::DisableWifi] { action(rpc,&f,&fresh).await.unwrap(); }
    server.await.unwrap(); assert_eq!(received.load(Ordering::Relaxed),4); assert_eq!(calls(&count),2);
}
