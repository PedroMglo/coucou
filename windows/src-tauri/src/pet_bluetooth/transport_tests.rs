use super::*;

#[tokio::test]
async fn bluetooth_uses_one_fixed_read_without_params_or_a_network_origin() {
    let f = Fixture::bytes(b"{\"apiVersion\":34,\"capabilities\":[\"bluetooth\"]}\n{\"id\":REQUEST_ID,\"result\":{\"powered\":true}}\n".to_vec());
    let result = bluetooth_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await.unwrap();
    assert_eq!(result.api_version, Some(34));
    assert_eq!(result.capabilities, vec!["bluetooth"]);
    assert_eq!(result.result, json!({"powered":true}));
    let requests = f.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["method"], "bluetooth.getState");
    assert_eq!(requests[0].as_object().unwrap().len(), 2);
}

#[tokio::test]
async fn bluetooth_revocation_at_the_writer_sends_no_query() {
    let f = Fixture::bytes(Vec::new());
    assert!(matches!(bluetooth_snapshot_guarded(&f.config, || Err("revoked")).await, Err(Failure::Guard("revoked"))));
    tokio::task::yield_now().await;
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn bluetooth_source_replacement_while_the_guard_runs_sends_no_query() {
    let f = Fixture::bytes(b"{\"id\":REQUEST_ID,\"result\":{}}\n".to_vec());
    let socket = f.config.socket.clone();
    let result = bluetooth_snapshot_guarded(&f.config, || {
        std::fs::remove_file(&socket).unwrap();
        Ok::<_, ()>(())
    }).await;
    assert!(matches!(result, Err(Failure::Native {reason:Error::Unavailable,write:WriteState::NotStarted}) | Err(Failure::Native {reason:Error::OriginChanged,write:WriteState::NotStarted})));
    tokio::task::yield_now().await;
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn bluetooth_source_replacement_after_reply_never_publishes_a_snapshot() {
    let f = Fixture::start(|mut stream, request, directory| async move {
        std::fs::remove_file(directory.join("danklinux-123.sock")).unwrap();
        let reply = format!("{{\"id\":{},\"result\":{{}}}}\n", request["id"]);
        let _ = stream.write_all(reply.as_bytes()).await;
    });
    let result = bluetooth_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await;
    assert!(matches!(result, Err(Failure::Native {reason:Error::Unavailable,..}) | Err(Failure::Native {reason:Error::OriginChanged,..})));
    assert_eq!(f.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn bluetooth_duplicate_reply_keys_and_native_private_errors_are_refused() {
    let f = Fixture::bytes(b"{\"id\":REQUEST_ID,\"result\":{},\"result\":{\"private\":true}}\n".to_vec());
    assert!(matches!(bluetooth_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await, Err(Failure::Native {reason:Error::Protocol,..})));
    let f = Fixture::bytes(b"{\"id\":REQUEST_ID,\"error\":\"PRIVATE pairing secret\"}\n".to_vec());
    let error = bluetooth_snapshot_guarded(&f.config, || Ok::<_, ()>(())).await.err().unwrap();
    assert!(matches!(error, Failure::Native {reason:Error::NativeFailure,..}));
    assert!(!format!("{error:?}").contains("PRIVATE"));
}
