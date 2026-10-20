fn enable_state(enabled:bool)->Value {json!({"backend":"networkmanager","wifiEnabled":enabled,"wifiNetworks":[],"wifiDevices":[{"name":"wlan0","hwAddress":"","state":"disconnected","connected":false,"apCapable":false,"networks":null}]})}
fn enable_observe(s:&Service,w:&SystemWindowAuthority,now:Instant,enabled:bool){
 let mut work=s.read(w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
 let mut o=observation(peer(1),vec![],ScanSupport::VerifiedByAdapter);o.result=enable_state(enabled);work.finish_read(o,now).unwrap();
}
#[test]fn enable_disabled_wifi_requires_full_private_nm_hardware_evidence_and_view2(){
 let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let v=s.view(&w,now).unwrap();
 assert_eq!(v.schema_version,2);assert!(v.capabilities.enable_wifi&&!v.capabilities.radio);
 assert!(v.enable_wifi_scope.is_some());assert_ne!(v.scan_scope,v.enable_wifi_scope);
 let p=public(&v);assert_eq!(p["data"]["schemaVersion"],1);assert_eq!(p["data"]["wifi"]["enabled"],false);
 assert_eq!(p["capabilities"]["enableWifi"],true);assert!(p.get("enableWifiScope").is_some());
 let text=p.to_string();for private in ["networkmanager","hwAddress","apCapable","disconnected"]{assert!(!text.contains(private));}
}
#[test]fn enable_support_rejects_partial_malformed_unknown_or_unbounded_evidence(){
 let caps=vec!["network".into()];let good=enable_state(false);assert!(enable_wifi_supported(&good,Some(34),&caps));
 for api in [None,Some(29),Some(33),Some(35)]{assert!(!enable_wifi_supported(&good,api,&caps));}
 assert!(!enable_wifi_supported(&good,Some(34),&[]));
 let mut bad=Vec::new();
 for backend in [Value::Null,json!("iwd"),json!("iwd+networkd"),json!("networkd"),json!("wpa_supplicant"),json!("NetworkManager")]{let mut v=good.clone();v["backend"]=backend;bad.push(v);}
 for enabled in [Value::Null,json!("false"),json!(1)]{let mut v=good.clone();v["wifiEnabled"]=enabled;bad.push(v);}
 for list in [Value::Null,json!([]),json!({}),json!([{"name":"wlan0"}]),json!([false]),json!(vec![good["wifiDevices"][0].clone();33])]{let mut v=good.clone();v["wifiDevices"]=list;bad.push(v);}
 for key in ["name","hwAddress","state","connected","apCapable","networks"]{let mut v=good.clone();v["wifiDevices"][0].as_object_mut().unwrap().remove(key);bad.push(v);}
 for name in ["","bad\n","\u{202e}name",&"x".repeat(65)]{let mut v=good.clone();v["wifiDevices"][0]["name"]=json!(name);bad.push(v);}
 for (key,value)in [("hwAddress",json!(1)),("state",json!({})),("connected",json!("false")),("apCapable",Value::Null),("networks",json!(false)),("networks",json!([{}])),("networks",json!(vec![json!({});257]))]{let mut v=good.clone();v["wifiDevices"][0][key]=value;bad.push(v);}
 let mut duplicate=good.clone();duplicate["wifiDevices"].as_array_mut().unwrap().push(good["wifiDevices"][0].clone());bad.push(duplicate);
 for value in bad{assert!(!enable_wifi_supported(&value,Some(34),&caps));}
 let mut empty_strings=good;empty_strings["wifiDevices"][0]["state"]=json!("");assert!(enable_wifi_supported(&empty_strings,Some(34),&caps));
}
#[test]fn enable_scope_kind_and_intent_cannot_alias_scan_or_be_retargeted(){
 let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let v=s.view(&w,now).unwrap();let scan=v.scan_scope.unwrap();let enable=v.enable_wifi_scope.unwrap();
 expect(s.enable_wifi(&w,&scan,"wrong",now),Reason::WrongTarget);expect(s.scan(&w,&enable,"wrong",now),Reason::WrongTarget);
 let a=s.enable_wifi(&w,&enable,"once",now).unwrap();let b=s.enable_wifi(&w,&enable,"once",now).unwrap();assert_eq!(a.ticket.id,b.ticket.id);assert!(b.work.is_none());
 expect(s.scan(&w,&scan,"once",now),Reason::IntentConflict);expect(s.enable_wifi(&w,&scan,"once",now),Reason::IntentConflict);
 drop(a.work);assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Cancelled);
}
#[test]fn enable_scope_publication_is_atomic_with_scan_and_registry_bounds(){
 let mut c=Config::default();c.max_bindings=1;let(s,_,w,now)=environment(c);enable_observe(&s,&w,now,false);
 expect(s.view(&w,now),Reason::Capacity);assert!(s.state.lock().unwrap().bindings.is_empty());
 let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);s.state.lock().unwrap().next=u64::MAX-1;
 expect(s.view(&w,now),Reason::Capacity);assert!(s.state.lock().unwrap().bindings.is_empty());
}
#[test]fn enable_rechecks_scope_ttl_generation_producer_and_original_lease(){
 {let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let old=s.view(&w,now).unwrap().enable_wifi_scope.unwrap();enable_observe(&s,&w,now,false);expect(s.enable_wifi(&w,&old,"old-generation",now),Reason::StaleTarget);}
 for changed in 0..3{
  let mut config=Config::default();config.operation_timeout=Duration::from_secs(30);let(s,_,w,now)=environment(config);enable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().enable_wifi_scope.unwrap();let a=s.enable_wifi(&w,&scope,"lease",now).unwrap();let mut work=a.work.unwrap();
  let (producer,at,reason)=match changed{0=>(peer(1),now+Duration::from_secs(16),Reason::StaleTarget),1=>(peer(2),now,Reason::SourceChanged),_=>{w.visibility_from_backend(false);w.visibility_from_backend(true);(peer(1),now,Reason::Unauthorized)}};
  assert_eq!(work.begin_dispatch(producer,at),Err(reason));expect(s.read(&w,true,at),Reason::Busy);drop(work);
 }
}
#[test]fn enable_cancellation_before_and_after_dispatch_never_confirms_or_releases_work(){
 for started in [false,true]{
  let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().enable_wifi_scope.unwrap();let a=s.enable_wifi(&w,&scope,"cancel",now).unwrap();let mut work=a.work.unwrap();
  if started{assert_eq!(work.begin_dispatch(peer(1),now),Ok(Operation::EnableWifi));}
  let t=s.cancel(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,if started{Phase::Unknown}else{Phase::Cancelled});assert!(!t.effect_confirmed);assert_eq!(t.refresh_recommended,started);
  expect(s.read(&w,true,now),Reason::Busy);drop(work);
 }
}
#[test]fn enable_ack_is_typed_ack_only_and_readback_remains_a_separate_observation(){
 let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().enable_wifi_scope.unwrap();let a=s.enable_wifi(&w,&scope,"enable",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();work.finish_enable_wifi(peer(1),now).unwrap();
 let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(public(&t)["kind"],"enableWifi");assert_eq!(t.phase,Phase::Acknowledged);assert!(!t.effect_confirmed&&t.refresh_recommended);
 let v=s.view(&w,now).unwrap();assert!(v.stale&&v.enable_wifi_scope.is_none());assert_eq!(public(&v)["data"]["wifi"]["enabled"],false);
 enable_observe(&s,&w,now,true);let v=s.view(&w,now).unwrap();assert!(!v.stale);assert_eq!(public(&v)["data"]["wifi"]["enabled"],true);assert!(!s.ticket(&w,&a.ticket.id,now).unwrap().effect_confirmed);
}
#[test]fn enable_lost_ack_and_wrong_ack_kind_stay_unknown_without_undo_or_retry(){
 for wrong_kind in [false,true]{let(s,_,w,now)=environment(Config::default());enable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().enable_wifi_scope.unwrap();let a=s.enable_wifi(&w,&scope,"unknown",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
  if wrong_kind{assert_eq!(work.finish_scan(peer(1),now),Err(Reason::InvalidObservation));}else{assert_eq!(work.fail(NativeError::Unavailable,now),Err(Reason::NativeUnavailable));}
  let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,Phase::Unknown);assert!(!t.effect_confirmed&&t.refresh_recommended);assert!(s.view(&w,now).unwrap().stale);
 }
}
