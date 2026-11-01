// Included by the synthetic endpoint fixture; never uses the user daemon.
async fn held_request_id(fixture: &Fixture, label: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let found = fixture.held.lock().unwrap().iter()
            .find(|request| request["params"]["label"] == label)
            .and_then(|request| request["id"].as_str()).map(str::to_string);
        if let Some(id) = found { return id; }
        assert!(Instant::now() < deadline, "synthetic daemon did not observe {label}");
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

#[tokio::test]
async fn outer_cancellation_more_than_registry_capacity_does_not_exhaust_pending_slots() {
    let fixture = Fixture::new().await;
    for _ in 0..132 {
        let result = timeout(Duration::from_millis(3), fixture.client().call("fixture/hold", json!({"label":"repeated"}))).await;
        assert!(result.is_err(), "the synthetic daemon must leave this call unanswered");
        assert!(fixture.client().pending.lock().unwrap().is_empty());
    }
    assert!(fixture.client().connected());
    fixture.client().call("fixture/barrier", json!({})).await.unwrap();
    let error = fixture.client().call("fixture/unknown", json!({})).await.unwrap_err();
    assert!(error.unsupported());
    assert!(fixture.client().pending.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_cancelled_calls_late_response_cannot_resolve_another_live_call() {
    let fixture = Fixture::new().await;
    let mut live = tokio::task::JoinSet::new();
    let client = fixture.client.as_ref().unwrap().clone();
    live.spawn(async move { client.call("fixture/hold", json!({"label":"live-a"})).await });
    let live_id = held_request_id(&fixture, "live-a").await;
    let mut cancelled = tokio::task::JoinSet::new();
    let client = fixture.client.as_ref().unwrap().clone();
    cancelled.spawn(async move { client.call("fixture/hold", json!({"label":"cancelled-b"})).await });
    let cancelled_id = held_request_id(&fixture, "cancelled-b").await;
    cancelled.abort_all();
    assert!(cancelled.join_next().await.unwrap().unwrap_err().is_cancelled());
    assert_ne!(live_id, cancelled_id);
    assert_eq!(fixture.client().pending.lock().unwrap().len(), 1);
    fixture.emit(vec![json!({"id":cancelled_id,"result":{"owner":"b"}})]).await;
    assert!(live.try_join_next().is_none());
    assert_eq!(fixture.client().pending.lock().unwrap().len(), 1);
    fixture.emit(vec![json!({"id":live_id,"result":{"owner":"a"}})]).await;
    let answer = timeout(Duration::from_secs(2), live.join_next()).await.unwrap().unwrap().unwrap().unwrap();
    assert_eq!(answer, json!({"owner":"a"}));
    assert!(fixture.client().pending.lock().unwrap().is_empty());
}

#[tokio::test]
async fn disconnect_still_fails_and_removes_an_unanswered_call() {
    let fixture = Fixture::new().await; let mut live = tokio::task::JoinSet::new();
    let client = fixture.client.as_ref().unwrap().clone();
    live.spawn(async move { client.call("fixture/hold", json!({"label":"disconnect"})).await });
    held_request_id(&fixture, "disconnect").await;
    fixture.commands.send(Command::Close).unwrap();
    let answer = timeout(Duration::from_secs(2), live.join_next()).await.unwrap().unwrap().unwrap();
    assert!(answer.is_err());
    assert!(fixture.client().pending.lock().unwrap().is_empty());
}

fn queued_client() -> (Client, mpsc::Receiver<Packet>) {
    let (writer, packets) = mpsc::channel(1);
    let (events, _) = broadcast::channel(4); let (stop, _) = tokio::sync::watch::channel(false);
    (Client { writer, pending: Arc::default(), events, next: AtomicU64::new(1),
        connected: Arc::new(AtomicBool::new(true)), stop, endpoint: PathBuf::new(), endpoint_identity:(0,0,0,0),settings:Arc::default(),sources:request_sources::Sources::new().unwrap() }, packets)
}

#[tokio::test]
async fn cancellation_before_send_and_before_writer_ack_cleans_only_its_own_slot() {
    let (client, mut packets) = queued_client();
    drop(client.call("fixture/unpolled", json!({})));
    assert!(client.pending.lock().unwrap().is_empty());
    assert!(matches!(packets.try_recv(), Err(mpsc::error::TryRecvError::Empty)));
    let (written, _ack) = oneshot::channel();
    assert!(client.writer.try_send(Packet { message:json!({"method":"fixture/blocker"}), written,
        scope:native_write::Scope::new(crate::pet_native_write::Guard{dispatch:crate::pet_native_write::Dispatch::NewEffect,deadline:Instant::now()+Duration::from_secs(5),cancel:None,caller:None,source:None}) }).is_ok());
    assert!(timeout(Duration::from_millis(5), client.call("fixture/before-send",json!({}))).await.is_err());
    assert!(client.pending.lock().unwrap().is_empty());
    assert_eq!(packets.try_recv().unwrap().message["method"], "fixture/blocker");
    assert!(matches!(packets.try_recv(), Err(mpsc::error::TryRecvError::Empty)));
    assert!(timeout(Duration::from_millis(5), client.call("fixture/before-ack",json!({}))).await.is_err());
    assert!(client.pending.lock().unwrap().is_empty());
    let abandoned = packets.try_recv().unwrap();
    assert_eq!(abandoned.message["method"], "fixture/before-ack");
    assert!(abandoned.expired(), "the writer must not transmit a cancelled queued packet");
    drop(packets);
    assert!(client.call("fixture/closed-writer",json!({})).await.is_err());
    assert!(client.pending.lock().unwrap().is_empty());
}

#[tokio::test]
async fn old_guard_preserves_replacement_identity_and_admission_rejects_id_collision() {
    let (client, _packets) = queued_client();
    let id = format!("mochi-{}-1", std::process::id());
    let first = Arc::new(()); let replacement = Arc::new(()); let (answer, _receiver) = oneshot::channel();
    let guard = PendingCall { pending:client.pending.clone(), id:id.clone(), identity:first };
    client.pending.lock().unwrap().insert(id.clone(), PendingEntry { identity:replacement.clone(), answer });
    drop(guard);
    assert!(Arc::ptr_eq(&client.pending.lock().unwrap()[&id].identity, &replacement));
    let error = client.call("fixture/collision", json!({})).await.unwrap_err();
    assert!(error.message.contains("reservado"));
    assert_eq!(client.pending.lock().unwrap().len(), 1);
    assert!(Arc::ptr_eq(&client.pending.lock().unwrap()[&id].identity, &replacement));
    let removed = client.pending.lock().unwrap().remove(&id); drop(removed);
}
