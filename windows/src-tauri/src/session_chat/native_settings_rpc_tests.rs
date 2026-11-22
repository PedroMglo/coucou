// Included in rpc::endpoint_tests. Only the private synthetic Unix-WebSocket
// fixture is used, never the user's daemon, real CLI/account or model.
fn permission_event(id:&str)->Value{json!({"method":"thread/settings/updated","params":{"threadId":id,"threadSettings":{"approvalPolicy":"never","approvalsReviewer":"user","sandboxPolicy":{"type":"readOnly"},"cwd":"/fixture/repo","model":"PRIVATE MODEL","collaborationMode":{"settings":{"developer_instructions":"PRIVATE INSTRUCTIONS"}}}}})}
const SETTINGS_THREAD:&str="11111111-1111-1111-1111-111111111111";
#[test]fn native_settings_allowlist_is_observation_delivery_not_request_authority(){
 assert!(allowed_event(&permission_event(SETTINGS_THREAD)));assert!(!allowed_event(&json!({"method":"unreviewed/private","params":{}})));
}
#[tokio::test]async fn native_settings_reader_caches_before_broadcast_and_forwards_only_an_invalidation(){
 let fixture=Fixture::new().await;let mut events=fixture.client().events();fixture.emit(vec![permission_event(SETTINGS_THREAD)]).await;
 let event=events.try_recv().unwrap();assert_eq!(event["method"],"mochi/threadSettingsObserved");assert!(!event.to_string().contains("PRIVATE"));assert!(event["params"].get("threadSettings").is_none());
 let snapshot=fixture.client().settings_observation(SETTINGS_THREAD,settings_now(),30);assert!(!snapshot.stale);let settings=snapshot.last_observed.unwrap();assert_eq!(settings.approval_policy,native_settings::ApprovalPolicy::Never);assert_eq!(settings.sandbox_policy,native_settings::SandboxPolicy::ReadOnly{network_access:false});
}
#[tokio::test]async fn native_settings_slow_broadcast_consumer_still_reads_the_latest_received_projection(){
 let fixture=Fixture::new().await;let mut events=fixture.client().events();let mut sent=Vec::new();for n in 0..300{let mut e=permission_event(SETTINGS_THREAD);e["params"]["threadSettings"]["approvalPolicy"]=json!(if n==299{"on-request"}else{"never"});sent.push(e);}fixture.emit(sent).await;
 assert!(matches!(events.try_recv(),Err(broadcast::error::TryRecvError::Lagged(_))));let snapshot=fixture.client().settings_observation(SETTINGS_THREAD,settings_now(),30);assert_eq!(snapshot.sequence,Some(300));assert!(!snapshot.stale);assert_eq!(snapshot.last_observed.unwrap().approval_policy,native_settings::ApprovalPolicy::OnRequest);
}
#[tokio::test]async fn native_settings_disconnect_retires_cache_without_retry_or_a_policy_write(){
 let fixture=Fixture::new().await;fixture.emit(vec![permission_event(SETTINGS_THREAD)]).await;fixture.commands.send(Command::Close).unwrap();
 let deadline=Instant::now()+Duration::from_secs(2);while fixture.client().connected(){assert!(Instant::now()<deadline);tokio::task::yield_now().await;}
 let snapshot=fixture.client().settings_observation(SETTINGS_THREAD,settings_now(),30);assert!(snapshot.stale);assert_eq!(snapshot.reason,Some(native_settings::Reason::Disconnected));assert!(snapshot.last_observed.is_some());assert!(fixture.client().pending.lock().unwrap().is_empty());
}

#[tokio::test]async fn native_settings_request_shaped_frames_are_never_broadcast_or_answered(){
 let fixture=Fixture::new().await;let mut events=fixture.client().events();
 let mut with_null=permission_event(SETTINGS_THREAD);with_null["id"]=Value::Null;
 let mut with_string=permission_event(SETTINGS_THREAD);with_string["id"]=json!("private-native-settings-request");
 fixture.emit(vec![with_null,with_string]).await;
 assert!(matches!(events.try_recv(),Err(broadcast::error::TryRecvError::Empty)));
 let observed=fixture.client().settings_observation(SETTINGS_THREAD,settings_now(),30);
 assert_eq!(observed.reason,Some(native_settings::Reason::NotObserved));assert!(observed.last_observed.is_none());assert!(observed.sequence.is_none());
 let received=fixture.received.lock().unwrap().clone();
 assert!(received.iter().all(|frame|frame.get("method").is_some()),"The reader must not send a JSON-RPC response/ACK");
 assert!(received.iter().all(|frame|matches!(frame["method"].as_str(),Some("initialize"|"initialized"|"fixture/barrier"))));
 assert!(!serde_json::to_string(&received).unwrap().contains("PRIVATE"));
 assert!(!serde_json::to_string(&received).unwrap().contains("private-native-settings-request"));
}
