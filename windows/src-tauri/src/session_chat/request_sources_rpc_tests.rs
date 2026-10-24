// Only private Unix-WebSocket peers. No model, native daemon or account.
fn native_approval(id:Value,thread:&str)->Value{json!({"id":id,"method":"item/commandExecution/requestApproval","params":{"threadId":thread,"turnId":"native-turn","command":"PRIVATE COMMAND","availableDecisions":["accept","decline"]},"_mochiNativeRequestSequence":{"endpoint":"remote-spoof","sequence":"999"}})}
async fn captured_native(f:&Fixture,id:Value)->(Value,Ticket){
    let mut events=f.client().events();f.emit(vec![native_approval(id,SETTINGS_THREAD)]).await;
    let event=events.try_recv().unwrap();let ticket=f.client().native_request(&event).unwrap();(event,ticket)
}
#[tokio::test]async fn native_source_captures_original_socket_request_and_sanitized_policy_before_broadcast(){
    let f=Fixture::new().await;f.emit(vec![permission_event(SETTINGS_THREAD)]).await;
    let (event,t)=captured_native(&f,json!(7)).await;assert!(t.current());let identity=t.identity();
    assert_eq!(identity.adapter,"codex");assert_eq!(identity.request_id,json!(7));assert_eq!(identity.session_id,SETTINGS_THREAD);assert_eq!(identity.turn_id.as_deref(),Some("native-turn"));assert!(identity.prompt_id.is_none());
    assert_eq!(serde_json::to_value(identity).unwrap()["transport"],"controlSocket");assert_eq!(identity.native_policy.coverage,"approvalSandboxSnapshot");assert_eq!(identity.native_policy.hash.as_ref().unwrap().len(),64);assert!(identity.native_policy.observed_at.unwrap()>1_000_000_000_000);
    assert_ne!(event["_mochiNativeRequestSequence"]["endpoint"],"remote-spoof");assert!(!serde_json::to_string(identity).unwrap().contains("PRIVATE"));
    let mut altered=event.clone();altered["params"]["command"]=json!("different command");assert!(f.client().native_request(&altered).is_none());
    altered=event.clone();altered["_mochiNativeRequestSequence"]["sequence"]=json!("999");assert!(f.client().native_request(&altered).is_none());
    altered=event.clone();altered.as_object_mut().unwrap().remove("_mochiNativeRequestSequence");assert!(f.client().native_request(&altered).is_none());
}
#[tokio::test]async fn native_source_observes_terminal_resolution_despite_broadcast_lag_and_preserves_other_ids(){
    let f=Fixture::new().await;let (event,t)=captured_native(&f,json!(7)).await;let (_,other)=captured_native(&f,json!("7")).await;
    f.emit(vec![json!({"method":"serverRequest/resolved","params":{"threadId":"foreign","requestId":7}})]).await;assert!(t.current());
    let mut slow=f.client().events();let mut events=(0..300).map(|_|json!({"method":"item/agentMessage/delta","params":{"threadId":SETTINGS_THREAD,"delta":"synthetic"}})).collect::<Vec<_>>();
    events.push(json!({"method":"serverRequest/resolved","params":{"threadId":SETTINGS_THREAD,"requestId":7}}));f.emit(events).await;
    assert!(matches!(slow.try_recv(),Err(broadcast::error::TryRecvError::Lagged(_))));assert!(!t.current());assert!(other.current());assert!(f.client().native_request(&event).is_none());
    assert!(f.client().respond_native(&t,json!({"decision":"accept"}),Arc::default()).await.is_err());
    assert!(!f.received.lock().unwrap().iter().any(|v|v.get("result").is_some()));
}
#[tokio::test]async fn native_source_reused_wire_id_cannot_be_reached_by_an_old_frame_or_second_response(){
    let f=Fixture::new().await;let (old,first)=captured_native(&f,json!(7)).await;
    f.emit(vec![json!({"method":"serverRequest/resolved","params":{"threadId":SETTINGS_THREAD,"requestId":7}})]).await;
    let (new,next)=captured_native(&f,json!(7)).await;assert_ne!(old["_mochiNativeRequestSequence"],new["_mochiNativeRequestSequence"]);assert!(f.client().native_request(&old).is_none());assert!(next.current());
    assert!(f.client().respond_native(&first,json!({"decision":"accept"}),Arc::default()).await.is_err());assert!(next.current());
    f.client().respond_native(&next,json!({"decision":"decline"}),Arc::default()).await.unwrap();assert!(!next.current());assert!(f.client().respond_native(&next,json!({"decision":"accept"}),Arc::default()).await.is_err());f.emit(vec![]).await;
    let received=f.received.lock().unwrap();assert_eq!(received.iter().filter(|v|v.get("result").is_some()).count(),1);assert!(received.iter().any(|v|v["id"]==7&&v["result"]["decision"]=="decline"));
}
#[tokio::test]async fn native_source_policy_changes_withdraw_pending_reviews_but_identical_private_or_foreign_updates_do_not(){
    let f=Fixture::new().await;f.emit(vec![permission_event(SETTINGS_THREAD)]).await;let (event,t)=captured_native(&f,json!(1)).await;
    let mut same=permission_event(SETTINGS_THREAD);same["params"]["threadSettings"]["model"]=json!("OTHER PRIVATE MODEL");same["params"]["threadSettings"]["developerInstructions"]=json!("OTHER PRIVATE INSTRUCTIONS");
    f.emit(vec![same,permission_event("22222222-2222-2222-2222-222222222222")]).await;assert!(t.current());
    let mut changed=permission_event(SETTINGS_THREAD);changed["params"]["threadSettings"]["sandboxPolicy"]=json!({"type":"dangerFullAccess"});f.emit(vec![changed]).await;
    assert!(!t.current()&&t.policy_invalidated());assert!(f.client().native_request(&event).is_none());assert!(f.client().respond_native(&t,json!({"decision":"accept"}),Arc::default()).await.is_err());
    assert!(!f.received.lock().unwrap().iter().any(|v|v.get("result").is_some()));
}
#[tokio::test]async fn native_source_unknown_and_invalid_policy_are_truthful_and_never_restore_old_review(){
    let f=Fixture::new().await;let (_,unknown)=captured_native(&f,json!(1)).await;assert_eq!(unknown.identity().native_policy.coverage,"notObserved");assert!(unknown.identity().native_policy.hash.is_none());
    f.emit(vec![permission_event(SETTINGS_THREAD)]).await;assert!(!unknown.current());let (_,known)=captured_native(&f,json!(2)).await;
    let mut invalid=permission_event(SETTINGS_THREAD);invalid["params"]["threadSettings"]["sandboxPolicy"]=json!({"type":"unrecognized"});f.emit(vec![invalid,permission_event(SETTINGS_THREAD)]).await;assert!(!known.current()&&known.policy_invalidated());
}
#[tokio::test]async fn native_source_same_id_on_another_endpoint_or_duplicate_pending_request_cannot_supply_a_ticket(){
    let one=Fixture::new().await;let two=Fixture::new().await;let (a,t)=captured_native(&one,json!(7)).await;let (_,other)=captured_native(&two,json!(7)).await;
    assert!(two.client().native_request(&a).is_none());assert!(t.current()&&other.current());
    let mut events=one.client().events();one.emit(vec![native_approval(json!(7),SETTINGS_THREAD)]).await;let duplicate=events.try_recv().unwrap();assert!(one.client().native_request(&duplicate).is_none());assert!(!t.current());assert!(other.current());
}
#[tokio::test]async fn native_source_disconnect_and_replaced_socket_refuse_late_answers(){
    let f=Fixture::new().await;let (_,t)=captured_native(&f,json!(1)).await;
    let retained=f._scratch.0.join("retained.sock");std::fs::rename(&f.socket,&retained).unwrap();let _replacement=UnixListener::bind(&f.socket).unwrap();std::fs::set_permissions(&f.socket,std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(f.client().respond_native(&t,json!({"decision":"accept"}),Arc::default()).await.is_err());
    f.commands.send(Command::Close).unwrap();let deadline=Instant::now()+Duration::from_secs(2);while t.current(){assert!(Instant::now()<deadline);tokio::task::yield_now().await;}
    assert!(!f.received.lock().unwrap().iter().any(|v|v.get("result").is_some()));
}
#[tokio::test]async fn native_source_queued_writer_keeps_the_capture_until_its_final_expiry_check(){
    let f=Fixture::new().await;let (_,t)=captured_native(&f,json!(1)).await;
    let (client,mut packets)=queued_client();let client=Arc::new(client);let writer=client.clone();let guard=t.clone();
    let writing=tokio::spawn(async move{writer.write_native_guarded(json!({"id":1,"result":{"decision":"accept"}}),None,Some(guard)).await});
    let packet=timeout(Duration::from_secs(2),packets.recv()).await.unwrap().unwrap();assert!(packet.scope.gate.source.as_ref().unwrap().0.same_capture(&t));assert!(!packet.expired());
    f.emit(vec![json!({"method":"serverRequest/resolved","params":{"threadId":SETTINGS_THREAD,"requestId":1}})]).await;assert!(packet.expired(),"queued response must become ineligible before the writer sends");let _=packet.written.send(Err(()));assert!(writing.await.unwrap().is_err());
}

#[tokio::test]async fn actual_external_native_socket_preserves_the_explicit_policy_choice_and_once_consumption(){
 let f=Fixture::new().await;let mut events=f.client().events();let rule=json!({"applyNetworkPolicyAmendment":{"network_policy_amendment":{"host":"fixture.invalid","action":"deny"}}});let mut request=native_approval(json!(7),SETTINGS_THREAD);request["params"]["availableDecisions"]=json!([rule]);f.emit(vec![request]).await;let event=events.try_recv().unwrap();let ticket=f.client().native_request(&event).unwrap();let choices=crate::pet_native_choices::Choices::capture(&ticket,"native-review",&event["params"]).unwrap().unwrap();assert!(choices.resolve(&json!({"allow":false})).is_err());let public=choices.public();let human=json!({"allow":false,"nativeChoice":public[0]["id"]});let response=choices.resolve(&human).unwrap();assert_eq!(response,json!({"decision":rule}));f.client().respond_native(&ticket,response.clone(),Arc::default()).await.unwrap();assert!(f.client().respond_native(&ticket,response,Arc::default()).await.is_err());f.emit(vec![]).await;let received=f.received.lock().unwrap();let replies=received.iter().filter(|v|v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"],json!({"decision":rule}));
}

#[tokio::test]async fn legacy_and_mcp_socket_sources_keep_real_owner_and_outer_request_ids_without_inventing_turns(){
 let f=Fixture::new().await;let mut events=f.client().events();
 for(index,(method,params))in[
  ("execCommandApproval",json!({"conversationId":SETTINGS_THREAD,"callId":"not-a-turn","command":["fixture"],"cwd":"/fixture","parsedCmd":[]})),
  ("applyPatchApproval",json!({"conversationId":SETTINGS_THREAD,"callId":"patch-call","fileChanges":{},"grantRoot":"/fixture/root"})),
  ("mcpServer/elicitation/request",json!({"threadId":SETTINGS_THREAD,"serverName":"fixture","turnId":null,"mode":"url","url":"https://fixture.invalid","elicitationId":"mcp-call","message":"fixture"})),
  ("mcpServer/elicitation/request",json!({"threadId":SETTINGS_THREAD,"serverName":"fixture","mode":"form","requestedSchema":{"type":"object","properties":{}}}))
 ].into_iter().enumerate(){
  let id=json!(index+50);f.emit(vec![json!({"id":id,"method":method,"params":params})]).await;let event=events.try_recv().unwrap();let ticket=f.client().native_request(&event).unwrap();assert_eq!(ticket.identity().session_id,SETTINGS_THREAD);assert!(ticket.identity().turn_id.is_none());assert_eq!(ticket.identity().request_id,id);
  let mut altered=event.clone();altered["params"]["message"]=json!("altered");assert!(f.client().native_request(&altered).is_none());
  f.emit(vec![json!({"method":"serverRequest/resolved","params":{"threadId":SETTINGS_THREAD,"requestId":id}})]).await;let _=events.try_recv().unwrap();assert!(!ticket.current());assert!(f.client().native_request(&event).is_none());assert!(f.client().respond_native(&ticket,json!({"decision":"approved"}),Arc::default()).await.is_err());
 }
 assert!(!f.received.lock().unwrap().iter().any(|v|v.get("result").is_some()));
}
#[tokio::test]async fn legacy_socket_root_and_session_choices_use_the_common_materializer_and_cannot_replay(){
 let f=Fixture::new().await;let mut events=f.client().events();let params=json!({"conversationId":SETTINGS_THREAD,"callId":"patch-call","fileChanges":{},"grantRoot":"/fixture/root"});
 f.emit(vec![json!({"id":"outer-id","method":"applyPatchApproval","params":params})]).await;let event=events.try_recv().unwrap();let ticket=f.client().native_request(&event).unwrap();let choices=crate::pet_native_choices::Choices::capture(&ticket,"legacy-socket-card",&params).unwrap().unwrap();assert!(choices.resolve(&json!({"allow":true})).is_err());
 let public=choices.public();assert_eq!(public[1]["scope"],"session");let response=choices.resolve(&json!({"allow":true,"nativeChoice":public[1]["id"]})).unwrap();assert_eq!(response,json!({"decision":"approved_for_session"}));f.client().respond_native(&ticket,response.clone(),Arc::default()).await.unwrap();assert!(f.client().respond_native(&ticket,response,Arc::default()).await.is_err());f.emit(vec![]).await;
 let got=f.received.lock().unwrap();let replies=got.iter().filter(|v|v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["id"],"outer-id");assert_eq!(replies[0]["result"]["decision"],"approved_for_session");
}

#[tokio::test]async fn typed_mcp_form_over_verified_socket_uses_the_same_values_and_one_native_reply(){
 let f=Fixture::new().await;let mut events=f.client().events();let p=json!({"threadId":SETTINGS_THREAD,"turnId":null,"serverName":"fixture","mode":"form","message":"Select a value","requestedSchema":{"type":"object","properties":{"choice":{"type":"string","oneOf":[{"const":"native-wire-value","title":"Friendly title"}]}},"required":["choice"]}});
 f.emit(vec![json!({"id":"native-mcp-id","method":"mcpServer/elicitation/request","params":p})]).await;let event=events.try_recv().unwrap();let ticket=f.client().native_request(&event).unwrap();let form=crate::pet_mcp_form::Form::capture(&ticket,"socket-form",&p).unwrap();let q=form.questions();let id=q[0]["id"].as_str().unwrap();let bad=json!({"allow":true,"answers":{id:["Friendly title"]}});assert!(form.resolve(&bad).is_err());assert!(ticket.current());let response=form.resolve(&json!({"allow":true,"answers":{id:[q[0]["options"][0]["value"]]}})).unwrap();f.client().respond_native(&ticket,response.clone(),Arc::default()).await.unwrap();assert!(f.client().respond_native(&ticket,response,Arc::default()).await.is_err());f.emit(vec![]).await;
 let received=f.received.lock().unwrap();let replies=received.iter().filter(|v|v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["id"],"native-mcp-id");assert_eq!(replies[0]["result"]["content"],json!({"choice":"native-wire-value"}));
}
