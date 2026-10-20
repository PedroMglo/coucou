fn enable_raw(enabled:bool)->Value {json!({"backend":"networkmanager","wifiEnabled":enabled,"wifiNetworks":[],"wifiDevices":[{"name":"wlan0","hwAddress":"","state":"disconnected","connected":false,"apCapable":false,"networks":null}]})}
impl Step {
 fn enable_read()->Self{Self{response:response(enable_raw(false)),..Self::read()}}
 fn enable_ack()->Self{Self{response:response(json!({"enabled":true})),..Self::read()}}
}
fn en_native(result:Result<EnableWifiAck,Failure<()>>,reason:Error,state:WriteState){match result{Err(Failure::Native{reason:r,write})=>assert_eq!((r,write),(reason,state)),_=>panic!("unexpected typed enable outcome")}}
#[tokio::test]async fn enable_exact_fixed_no_params_wire_uses_same_read_origin_and_one_guard(){
 let f=Fixture::start(vec![Step::enable_read(),Step::enable_ack()]);
 let read=network_snapshot_guarded(&f.config,||Ok::<_,()>(())).await.unwrap();assert!(read.supports_enable_wifi());let origin=read.origin().clone();let mut calls=0;
 let ack=enable_wifi(&f.config,&origin,|fresh|{assert!(fresh==&origin);calls+=1;Ok::<_,()>(())}).await.unwrap();assert!(ack.origin()==&origin);assert_eq!(calls,1);
 let requests=f.requests.lock().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[1]["method"],"network.wifi.enable");assert!(requests[1]["id"].as_u64().unwrap()>0);assert_eq!(requests[1].as_object().unwrap().len(),2);assert!(requests[1].get("params").is_none());
}
#[tokio::test]async fn enable_without_native_hardware_proof_does_not_connect_or_reuse_scan_proof(){
 for value in [json!({"wifiEnabled":true,"wifiNetworks":[]}),json!({"backend":"iwd","wifiEnabled":false,"wifiDevices":[]}),json!({"backend":"networkmanager","wifiEnabled":true,"wifiDevices":[{"name":"wlan0"}]})]{
  let f=Fixture::start(vec![Step{response:response(value),..Step::read()}]);let origin=f.origin().await;
  assert!(origin.scan_proven&&!origin.enable_wifi_proven);en_native(enable_wifi(&f.config,&origin,|_|panic!("unproven enable guard")).await,Error::Unsupported,WriteState::NotStarted);assert_eq!(f.connections.load(Ordering::Relaxed),1);
 }
}
#[tokio::test]async fn enable_rejects_scan_false_string_missing_extra_result_and_ambiguous_envelopes(){
 let mut bodies=vec![response(Value::Null),response(json!({})),response(json!({"enabled":false})),response(json!({"enabled":"true"})),response(json!({"success":true,"message":"scanning"})),response(json!({"enabled":true,"extra":"PRIVATE"}))];
 bodies.extend([
  b"{\"id\":REQUEST_ID,\"result\":{\"enabled\":true},\"error\":null}\n".to_vec(),
  b"{\"id\":REQUEST_ID,\"result\":{\"enabled\":true},\"method\":\"event\"}\n".to_vec(),
  b"{\"id\":REQUEST_ID,\"result\":{\"enabled\":false,\"enabled\":true}}\n".to_vec(),
  b"{\"id\":REQUEST_ID,\"result\":false,\"result\":{\"enabled\":true}}\n".to_vec(),
  b"{\"id\":0,\"id\":REQUEST_ID,\"result\":{\"enabled\":true}}\n".to_vec(),
 ]);
 for bytes in bodies{let f=Fixture::start(vec![Step::enable_read(),Step{response:bytes,..Step::enable_ack()}]);let origin=f.origin().await;en_native(enable_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,Error::Protocol,WriteState::MayHaveStarted);assert_eq!(f.requests.lock().unwrap().len(),2);}
}
#[tokio::test]async fn enable_native_error_and_lost_ack_are_unknown_after_one_write_without_retry(){
 for bytes in [b"{\"id\":REQUEST_ID,\"error\":\"PRIVATE_PASSWORD\"}\n".to_vec(),Vec::new()]{let failure=if bytes.is_empty(){Error::Unavailable}else{Error::NativeFailure};let f=Fixture::start(vec![Step::enable_read(),Step{response:bytes,..Step::enable_ack()}]);let origin=f.origin().await;en_native(enable_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await,failure,WriteState::MayHaveStarted);assert_eq!(f.connections.load(Ordering::Relaxed),2);assert_eq!(f.requests.lock().unwrap().len(),2);}
}
#[tokio::test]async fn enable_wrong_id_cannot_ack_but_later_exact_matching_id_can(){
 let mut step=Step::enable_ack();step.response=b"{\"id\":\"wrong\",\"result\":{\"enabled\":true}}\n".to_vec();step.response.extend(response(json!({"enabled":true})));
 let f=Fixture::start(vec![Step::enable_read(),step]);let origin=f.origin().await;assert!(enable_wifi(&f.config,&origin,|_|Ok::<_,()>(())).await.is_ok());
}
#[tokio::test]async fn enable_guard_denial_and_expired_guard_make_no_native_action_write(){
 for slow in [false,true]{let mut f=Fixture::start(vec![Step::enable_read(),Step::enable_ack()]);let origin=f.origin().await;
  if slow{f.config.deadline=Duration::from_millis(20);en_native(enable_wifi(&f.config,&origin,|_|{std::thread::sleep(Duration::from_millis(50));Ok::<_,()>(())}).await,Error::Timeout,WriteState::NotStarted);}else{assert!(matches!(enable_wifi(&f.config,&origin,|_|Err("revoked")).await,Err(Failure::Guard("revoked"))));}
  assert_eq!(f.requests.lock().unwrap().len(),1);
 }
}
#[tokio::test]async fn enable_reauthenticates_each_source_incarnation_and_rejects_replaced_endpoints(){
 let mut steps=vec![Step::enable_read()];for _ in 0..8{steps.push(Step::enable_ack());}let f=Fixture::start(steps);let origin=f.origin().await;
 for i in 0..8{let mut wrong=origin.clone();match i{0=>wrong.source.uid+=1,1=>wrong.source.peer.pid+=1,2=>wrong.source.peer.start+=1,3=>wrong.source.peer.device+=1,4=>wrong.source.peer.inode+=1,5=>wrong.source.endpoint.inode+=1,6=>wrong.source.endpoint.ctime_ns+=1,_=>wrong.source.socket=PathBuf::from("/different/socket")};en_native(enable_wifi(&f.config,&wrong,|_|panic!("foreign enable guard")).await,Error::OriginChanged,WriteState::NotStarted);}
 assert_eq!(f.requests.lock().unwrap().len(),1);
}
#[tokio::test]async fn enable_requires_current_reviewed_greeting_before_guard_and_dispatch(){
 for greeting in [greet(json!(33),json!(["network"])),greet(json!(34),json!([])),b"{\"apiVersion\":0,\"apiVersion\":34,\"capabilities\":[\"network\"]}\n".to_vec()]{
  let expected=if greeting.windows(4).any(|w|w==b"\":0,"){Error::Protocol}else{Error::Unsupported};
  let f=Fixture::start(vec![Step::enable_read(),Step{greeting,..Step::enable_ack()}]);let origin=f.origin().await;en_native(enable_wifi(&f.config,&origin,|_|panic!("bad enable greeting")).await,expected,WriteState::NotStarted);assert_eq!(f.requests.lock().unwrap().len(),1);
 }
}
#[tokio::test]async fn enable_proof_cannot_come_from_duplicate_keys_in_the_authenticated_read(){
 let raw=enable_raw(false).to_string().replacen("\"backend\":\"networkmanager\"","\"backend\":\"iwd\",\"backend\":\"networkmanager\"",1);
 let f=Fixture::start(vec![Step{response:format!("{{\"id\":REQUEST_ID,\"result\":{raw}}}\n").into_bytes(),..Step::read()}]);
 assert!(matches!(network_snapshot_guarded(&f.config,||Ok::<_,()>(())).await,Err(Failure::Native{reason:Error::Protocol,..})));assert_eq!(f.requests.lock().unwrap().len(),1);
}
