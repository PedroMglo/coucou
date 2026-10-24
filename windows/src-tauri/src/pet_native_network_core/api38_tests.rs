//! Synthetic local sockets only; no running DMS or physical radio is used.
use super::*;

fn api38_read() -> Step {
    Step { greeting: greet(json!(38), json!(["network"])),
        response: response(enable_raw(true)), ..Step::read() }
}
fn api38_ack(rpc: Rpc) -> Step {
    let step = match rpc { Rpc::Scan => Step::scan(), Rpc::EnableWifi => Step::enable_ack(),
        Rpc::DisableWifi => Step::disable_ack(), Rpc::Read => panic!("action expected") };
    Step { greeting: greet(json!(38), json!(["network"])), ..step }
}
async fn api38_action<E, F>(rpc: Rpc, f: &Fixture, origin: &Origin, guard: F) -> Result<(), Failure<E>>
where F: FnOnce(&Origin) -> Result<(), E> {
    match rpc {
        Rpc::Scan => scan_wifi(&f.config, origin, guard).await.map(|_| ()),
        Rpc::EnableWifi => enable_wifi(&f.config, origin, guard).await.map(|_| ()),
        Rpc::DisableWifi => disable_wifi(&f.config, origin, guard).await.map(|_| ()),
        Rpc::Read => panic!("action expected"),
    }
}
fn rejected(result: Result<(), Failure<()>>, reason: Error, write: WriteState) {
    assert_eq!(result, Err(Failure::Native { reason, write }));
}

#[tokio::test]
async fn api38_fixed_scan_enable_disable_use_same_origin_and_exact_no_params_request_once() {
    for (rpc, method) in [(Rpc::Scan, "network.wifi.scan"), (Rpc::EnableWifi, "network.wifi.enable"),
        (Rpc::DisableWifi, "network.wifi.disable")] {
        let f = Fixture::start(vec![api38_read(), api38_ack(rpc)]);
        let read = network_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await.unwrap();
        assert_eq!(read.api_version, Some(38));
        assert!(read.supports_scan() && read.supports_enable_wifi() && read.origin().disable_wifi_proven);
        let origin = read.origin().clone(); let mut guards = 0;
        api38_action(rpc, &f, &origin, |current| {
            assert!(current == &origin); guards += 1; Ok::<_, ()>(())
        }).await.unwrap();
        assert_eq!(guards, 1);
        let requests = f.requests.lock().unwrap(); assert_eq!(requests.len(), 2);
        assert_eq!(requests[1]["method"], method);
        assert_eq!(requests[1].as_object().unwrap().len(), 2);
        assert!(requests[1]["id"].as_u64().is_some_and(|id| id > 0));
        assert!(requests[1].get("params").is_none());
    }
}

#[tokio::test]
async fn api38_missing_or_partial_hardware_evidence_remains_read_only_for_enable_disable() {
    for value in [json!({"wifiEnabled":true}), json!({"backend":"iwd","wifiEnabled":true,"wifiDevices":[]}),
        json!({"backend":"networkmanager","wifiEnabled":true,"wifiDevices":[{"name":"wlan0"}]})] {
        let f = Fixture::start(vec![Step { response: response(value), ..api38_read() }]);
        let origin = f.origin().await; assert!(origin.scan_proven);
        assert!(!origin.enable_wifi_proven && !origin.disable_wifi_proven);
        for rpc in [Rpc::EnableWifi, Rpc::DisableWifi] {
            rejected(api38_action(rpc, &f, &origin, |_| panic!("unproven action"))
                .await, Error::Unsupported, WriteState::NotStarted);
        }
        assert_eq!(f.connections.load(Ordering::Relaxed), 1);
        assert_eq!(f.requests.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn api38_current_greeting_must_still_be_reviewed_and_network_capable_before_action() {
    for rpc in [Rpc::Scan, Rpc::EnableWifi, Rpc::DisableWifi] {
        for (version, caps) in [(37, json!(["network"])), (39, json!(["network"])),
            (38, json!([])), (38, json!(["NETWORK"]))] {
            let f = Fixture::start(vec![api38_read(), Step {
                greeting: greet(json!(version), caps), ..api38_ack(rpc) }]);
            let origin = f.origin().await;
            rejected(api38_action(rpc, &f, &origin, |_| panic!("unreviewed greeting"))
                .await, Error::Unsupported, WriteState::NotStarted);
            assert_eq!(f.requests.lock().unwrap().len(), 1);
        }
    }
}

#[tokio::test]
async fn api38_guard_revocation_after_current_handshake_writes_no_action_bytes() {
    for rpc in [Rpc::Scan, Rpc::EnableWifi, Rpc::DisableWifi] {
        let f = Fixture::start(vec![api38_read(), api38_ack(rpc)]); let origin = f.origin().await;
        let mut guards = 0;
        let result = api38_action(rpc, &f, &origin, |_| { guards += 1; Err("revoked") }).await;
        assert_eq!(result, Err(Failure::Guard("revoked"))); assert_eq!(guards, 1);
        assert_eq!(f.requests.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn api38_failure_malformed_or_missing_ack_remains_unknown_without_retry() {
    for rpc in [Rpc::Scan, Rpc::EnableWifi, Rpc::DisableWifi] {
        for (body, reason) in [
            (response(json!({"enabled":"true"})), Error::Protocol),
            (b"{\"id\":REQUEST_ID,\"result\":{},\"result\":{\"enabled\":true}}\n".to_vec(), Error::Protocol),
            (b"{\"id\":REQUEST_ID,\"error\":\"PRIVATE_NATIVE_ERROR\"}\n".to_vec(), Error::NativeFailure),
            (Vec::new(), Error::Unavailable),
        ] {
            let f = Fixture::start(vec![api38_read(), Step { response: body, ..api38_ack(rpc) }]);
            let origin = f.origin().await;
            rejected(api38_action(rpc, &f, &origin, |_| Ok::<_, ()>(())).await,
                reason, WriteState::MayHaveStarted);
            assert_eq!(f.connections.load(Ordering::Relaxed), 2);
            assert_eq!(f.requests.lock().unwrap().len(), 2);
        }
    }
}
