fn enable_actor_state(enabled:bool)->Value{json!({"backend":"networkmanager","wifiEnabled":enabled,"wifiNetworks":[],"wifiDevices":[{"name":"wlan0","hwAddress":"","state":"disconnected","connected":false,"apCapable":false,"networks":null}]})}
impl Step {
 fn enable_read(enabled:bool)->Self{Self{result:enable_actor_state(enabled),..Self::read()}}
 fn enable_ack()->Self{Self{result:json!({"enabled":true}),..Self::read()}}
}
async fn enable_actor_scope(client:&Client,window:&WindowRef)->String{accepted_read(client,window).await;client.view(window).await.unwrap().enable_wifi_scope.unwrap()}
#[tokio::test]async fn enable_actor_ack_is_published_after_worker_drain_then_readback_is_separate(){
 let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::enable_read(false),Step::enable_ack(),Step::enable_read(true)]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());
 let scope=enable_actor_scope(&client,&window).await;let a=client.enqueue_prepared_enable_wifi(client.prepare(&window).unwrap(),&scope,"enable-once").unwrap().wait().await.unwrap();let ack=terminal(&client,&window,&a.id).await;
 assert_eq!((ack.kind,ack.phase),(l::Kind::EnableWifi,l::Phase::Acknowledged));assert!(!ack.effect_confirmed&&ack.refresh_recommended);assert!(!inspection(&client).await.active);
 let stale=client.view(&window).await.unwrap();assert!(stale.stale);assert_eq!(serde_json::to_value(stale).unwrap()["data"]["wifi"]["enabled"],false);
 accepted_read(&client,&window).await;let current=client.view(&window).await.unwrap();assert_eq!(serde_json::to_value(current).unwrap()["data"]["wifi"]["enabled"],true);assert!(!client.ticket(&window,&a.id).await.unwrap().effect_confirmed);
 let methods=f.requests.lock().unwrap().iter().map(|v|v["method"].as_str().unwrap().to_owned()).collect::<Vec<_>>();assert_eq!(methods,vec!["network.getState","network.wifi.enable","network.getState"]);ex.shutdown().await;
}
#[tokio::test]async fn enable_prepared_before_await_never_resamples_a_reopened_context(){
 let _serial=SERIAL.lock().await;stopped().await;let f=Fixture::start(vec![Step::enable_read(false)]);let(ex,client,_,owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;let original=client.prepare(&window).unwrap();
 owner.visibility_from_backend(false);owner.visibility_from_backend(true);
 expect(client.enqueue_prepared_enable_wifi(original,&scope,"old-prepared").unwrap().wait().await,l::Reason::Revoked);assert_eq!(f.count("network.wifi.enable").await,0);assert!(!inspection(&client).await.active);ex.shutdown().await;
}
#[tokio::test]async fn enable_original_stamp_is_rechecked_at_first_write_after_native_greeting_wait(){
 for global in [false,true]{let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::enable_ack();step.greeting_gate=Some(gate.clone());let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,setting,owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;
  let a=client.enable_wifi(&window,&scope,"old-writer").await.unwrap();if global{setting.update_from_backend(false);setting.update_from_backend(true);}else{owner.visibility_from_backend(false);owner.visibility_from_backend(true);}gate.notify_one();idle(&client).await;
  let ticket=client.ticket(&window,&a.id).await.unwrap();assert_eq!(ticket.phase,l::Phase::Failed);assert_eq!(ticket.reason,Some(l::Reason::Revoked));assert_eq!(f.count("network.wifi.enable").await,0);ex.shutdown().await;
 }
}
#[tokio::test]async fn enable_cancelled_before_or_after_dispatch_retains_actual_io_slot_until_drain(){
 for dispatched in [false,true]{let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::enable_ack();if dispatched{step.reply_gate=Some(gate.clone());}else{step.greeting_gate=Some(gate.clone());}
  let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;let a=client.enable_wifi(&window,&scope,"cancel").await.unwrap();
  if dispatched{f.wait_requests(2).await;}
  let t=client.cancel(&window,&a.id).await.unwrap();assert_eq!(t.phase,if dispatched{l::Phase::Unknown}else{l::Phase::Cancelled});assert!(!t.effect_confirmed);assert!(inspection(&client).await.active);expect(client.read(&window,true).await,l::Reason::Busy);
  gate.notify_one();idle(&client).await;assert_eq!(f.count("network.wifi.enable").await,usize::from(dispatched));assert_eq!(client.ticket(&window,&a.id).await.unwrap().phase,t.phase);ex.shutdown().await;
 }
}
#[tokio::test]async fn enable_dropped_caller_cannot_abort_worker_or_release_shared_physical_slot(){
 let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::enable_ack();step.reply_gate=Some(gate.clone());let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,binding,_owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;
 let(other,w)=(client.clone(),window.clone());let(tx,rx)=oneshot::channel();let caller=tokio::spawn(async move{let a=other.enable_wifi(&w,&scope,"caller").await.unwrap();let _=tx.send(a.id.clone());loop{let _=other.ticket(&w,&a.id).await;tokio::time::sleep(Duration::from_millis(1)).await;}});
 let id=rx.await.unwrap();f.wait_requests(2).await;caller.abort();let _=caller.await;assert!(inspection(&client).await.active);expect(client.read(&window,true).await,l::Reason::Busy);expect(Executor::start(f.native.clone(),config(l::Config::default()),&binding),l::Reason::Busy);
 gate.notify_one();assert_eq!(terminal(&client,&window,&id).await.phase,l::Phase::Acknowledged);assert_eq!(f.count("network.wifi.enable").await,1);ex.shutdown().await;
}
#[tokio::test]async fn enable_native_error_never_auto_retries_scans_disables_or_reads_back(){
 let _serial=SERIAL.lock().await;stopped().await;let mut step=Step::enable_ack();step.error=Some("PRIVATE_NATIVE_ERROR");let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;
 let a=client.enable_wifi(&window,&scope,"native-error").await.unwrap();let t=terminal(&client,&window,&a.id).await;assert_eq!(t.phase,l::Phase::Unknown);assert!(!t.effect_confirmed&&t.refresh_recommended);let v=client.view(&window).await.unwrap();assert!(v.stale&&v.enable_wifi_scope.is_none());assert_eq!(f.requests.lock().unwrap().len(),2);assert_eq!(f.count("network.wifi.enable").await,1);ex.shutdown().await;
}
#[tokio::test]async fn enable_context_revoked_after_write_cannot_publish_ack_or_new_state(){
 let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::enable_ack();step.reply_gate=Some(gate.clone());let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,_,owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;
 let a=client.enable_wifi(&window,&scope,"late-ack").await.unwrap();f.wait_requests(2).await;owner.visibility_from_backend(false);owner.visibility_from_backend(true);expect(client.read(&window,true).await,l::Reason::Busy);gate.notify_one();idle(&client).await;
 let t=client.ticket(&window,&a.id).await.unwrap();assert_eq!(t.phase,l::Phase::Unknown);assert!(!t.effect_confirmed);assert!(client.view(&window).await.unwrap().enable_wifi_scope.is_none());assert_eq!(f.requests.lock().unwrap().len(),2);ex.shutdown().await;
}
#[tokio::test]async fn enable_shutdown_revokes_writer_without_aborting_owned_io_or_freeing_singleton(){
 let _serial=SERIAL.lock().await;stopped().await;let gate=Arc::new(Notify::new());let mut step=Step::enable_ack();step.greeting_gate=Some(gate.clone());let f=Fixture::start(vec![Step::enable_read(false),step]);let(ex,client,_,_owner,window)=launch(&f,l::Config::default());let scope=enable_actor_scope(&client,&window).await;
 client.enable_wifi(&window,&scope,"shutdown").await.unwrap();let shutdown=tokio::spawn(ex.shutdown());tokio::time::sleep(Duration::from_millis(10)).await;assert!(!shutdown.is_finished()&&SINGLETON.load(Ordering::Acquire));gate.notify_one();shutdown.await.unwrap();stopped().await;assert_eq!(f.count("network.wifi.enable").await,0);
}
