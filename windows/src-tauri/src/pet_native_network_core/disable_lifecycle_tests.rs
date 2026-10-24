fn disable_state(enabled:bool)->Value {json!({"backend":"networkmanager","wifiEnabled":enabled,"wifiNetworks":[],"wifiDevices":[{"name":"wlan0","hwAddress":"","state":"disconnected","connected":false,"apCapable":false,"networks":null}]})}
fn disable_observe(s:&Service,w:&SystemWindowAuthority,now:Instant,enabled:bool){
 let mut work=s.read(w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
 let mut o=observation(peer(1),vec![],ScanSupport::VerifiedByAdapter);o.result=disable_state(enabled);work.finish_read(o,now).unwrap();
}
#[test]fn disable_disabled_wifi_requires_full_private_nm_hardware_evidence_and_view2(){
 let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);let v=s.view(&w,now).unwrap();
 assert_eq!(v.schema_version,2);assert!(v.capabilities.disable_wifi&&!v.capabilities.radio);
 assert!(v.disable_wifi_scope.is_some());assert_ne!(v.scan_scope,v.disable_wifi_scope);
 let p=public(&v);assert_eq!(p["data"]["schemaVersion"],1);assert_eq!(p["data"]["wifi"]["enabled"],false);
 assert_eq!(p["capabilities"]["disableWifi"],true);assert!(p.get("disableWifiScope").is_some());
 let text=p.to_string();for private in ["networkmanager","hwAddress","apCapable","disconnected"]{assert!(!text.contains(private));}
}
#[test]fn disable_support_rejects_partial_malformed_unknown_or_unbounded_evidence(){
 let caps=vec!["network".into()];let good=disable_state(false);assert!(disable_wifi_supported(&good,Some(34),&caps));
 for api in [None,Some(29),Some(33),Some(35),Some(36),Some(37),Some(39),Some(u64::MAX)]{assert!(!disable_wifi_supported(&good,api,&caps));}
 assert!(!disable_wifi_supported(&good,Some(34),&[]));
 let mut bad=Vec::new();
 for backend in [Value::Null,json!("iwd"),json!("iwd+networkd"),json!("networkd"),json!("wpa_supplicant"),json!("NetworkManager")]{let mut v=good.clone();v["backend"]=backend;bad.push(v);}
 for enabled in [Value::Null,json!("false"),json!(1)]{let mut v=good.clone();v["wifiEnabled"]=enabled;bad.push(v);}
 for list in [Value::Null,json!([]),json!({}),json!([{"name":"wlan0"}]),json!([false]),json!(vec![good["wifiDevices"][0].clone();33])]{let mut v=good.clone();v["wifiDevices"]=list;bad.push(v);}
 for key in ["name","hwAddress","state","connected","apCapable","networks"]{let mut v=good.clone();v["wifiDevices"][0].as_object_mut().unwrap().remove(key);bad.push(v);}
 for name in ["","bad\n","\u{202e}name",&"x".repeat(65)]{let mut v=good.clone();v["wifiDevices"][0]["name"]=json!(name);bad.push(v);}
 for (key,value)in [("hwAddress",json!(1)),("state",json!({})),("connected",json!("false")),("apCapable",Value::Null),("networks",json!(false)),("networks",json!([{}])),("networks",json!(vec![json!({});257]))]{let mut v=good.clone();v["wifiDevices"][0][key]=value;bad.push(v);}
 let mut duplicate=good.clone();duplicate["wifiDevices"].as_array_mut().unwrap().push(good["wifiDevices"][0].clone());bad.push(duplicate);
 for value in bad{assert!(!disable_wifi_supported(&value,Some(34),&caps));}
 let mut empty_strings=good;empty_strings["wifiDevices"][0]["state"]=json!("");assert!(disable_wifi_supported(&empty_strings,Some(34),&caps));
}
#[test]fn disable_scope_kind_and_intent_cannot_alias_scan_or_be_retargeted(){
 let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);let v=s.view(&w,now).unwrap();let scan=v.scan_scope.unwrap();let enable=v.disable_wifi_scope.unwrap();
 expect(s.disable_wifi(&w,&scan,"wrong",now),Reason::WrongTarget);expect(s.scan(&w,&enable,"wrong",now),Reason::WrongTarget);
 let a=s.disable_wifi(&w,&enable,"once",now).unwrap();let b=s.disable_wifi(&w,&enable,"once",now).unwrap();assert_eq!(a.ticket.id,b.ticket.id);assert!(b.work.is_none());
 expect(s.scan(&w,&scan,"once",now),Reason::IntentConflict);expect(s.disable_wifi(&w,&scan,"once",now),Reason::IntentConflict);
 drop(a.work);assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Cancelled);
}
#[test]fn disable_scope_publication_is_atomic_with_scan_and_registry_bounds(){
 let mut c=Config::default();c.max_bindings=1;let(s,_,w,now)=environment(c);disable_observe(&s,&w,now,false);
 expect(s.view(&w,now),Reason::Capacity);assert!(s.state.lock().unwrap().bindings.is_empty());
 let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);s.state.lock().unwrap().next=u64::MAX-1;
 expect(s.view(&w,now),Reason::Capacity);assert!(s.state.lock().unwrap().bindings.is_empty());
}
#[test]fn disable_rechecks_scope_ttl_generation_producer_and_original_lease(){
 {let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);let old=s.view(&w,now).unwrap().disable_wifi_scope.unwrap();disable_observe(&s,&w,now,false);expect(s.disable_wifi(&w,&old,"old-generation",now),Reason::StaleTarget);}
 for changed in 0..3{
  let mut config=Config::default();config.operation_timeout=Duration::from_secs(30);let(s,_,w,now)=environment(config);disable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().disable_wifi_scope.unwrap();let a=s.disable_wifi(&w,&scope,"lease",now).unwrap();let mut work=a.work.unwrap();
  let (producer,at,reason)=match changed{0=>(peer(1),now+Duration::from_secs(16),Reason::StaleTarget),1=>(peer(2),now,Reason::SourceChanged),_=>{w.visibility_from_backend(false);w.visibility_from_backend(true);(peer(1),now,Reason::Unauthorized)}};
  assert_eq!(work.begin_dispatch(producer,at),Err(reason));expect(s.read(&w,true,at),Reason::Busy);drop(work);
 }
}
#[test]fn disable_cancellation_before_and_after_dispatch_never_confirms_or_releases_work(){
 for started in [false,true]{
  let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().disable_wifi_scope.unwrap();let a=s.disable_wifi(&w,&scope,"cancel",now).unwrap();let mut work=a.work.unwrap();
  if started{assert_eq!(work.begin_dispatch(peer(1),now),Ok(Operation::DisableWifi));}
  let t=s.cancel(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,if started{Phase::Unknown}else{Phase::Cancelled});assert!(!t.effect_confirmed);assert_eq!(t.refresh_recommended,started);
  expect(s.read(&w,true,now),Reason::Busy);drop(work);
 }
}
#[test]fn disable_ack_is_typed_ack_only_and_readback_remains_a_separate_observation(){
 let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,true);let scope=s.view(&w,now).unwrap().disable_wifi_scope.unwrap();let a=s.disable_wifi(&w,&scope,"enable",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();work.finish_disable_wifi(peer(1),now).unwrap();
 let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(public(&t)["kind"],"disableWifi");assert_eq!(t.phase,Phase::Acknowledged);assert!(!t.effect_confirmed&&t.refresh_recommended);
 let v=s.view(&w,now).unwrap();assert!(v.stale&&v.disable_wifi_scope.is_none());assert_eq!(public(&v)["data"]["wifi"]["enabled"],true);
 disable_observe(&s,&w,now,false);let v=s.view(&w,now).unwrap();assert!(!v.stale);assert_eq!(public(&v)["data"]["wifi"]["enabled"],false);assert!(!s.ticket(&w,&a.ticket.id,now).unwrap().effect_confirmed);
}
#[test]fn disable_lost_ack_and_wrong_ack_kind_stay_unknown_without_undo_or_retry(){
 for wrong_kind in [false,true]{let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,false);let scope=s.view(&w,now).unwrap().disable_wifi_scope.unwrap();let a=s.disable_wifi(&w,&scope,"unknown",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
  if wrong_kind{assert_eq!(work.finish_scan(peer(1),now),Err(Reason::InvalidObservation));}else{assert_eq!(work.fail(NativeError::Unavailable,now),Err(Reason::NativeUnavailable));}
  let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,Phase::Unknown);assert!(!t.effect_confirmed&&t.refresh_recommended);assert!(s.view(&w,now).unwrap().stale);
 }
}

#[test]fn disable_enable_and_scan_scopes_are_distinct_and_intents_cannot_change_kind(){
 let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,true);let v=s.view(&w,now).unwrap();
 let off=v.disable_wifi_scope.unwrap();let on=v.enable_wifi_scope.unwrap();let scan=v.scan_scope.unwrap();
 assert_ne!(off,on);assert_ne!(off,scan);assert_ne!(on,scan);
 expect(s.enable_wifi(&w,&off,"wrong-direction",now),Reason::WrongTarget);
 expect(s.disable_wifi(&w,&on,"wrong-direction",now),Reason::WrongTarget);
 let a=s.enable_wifi(&w,&on,"same-intent",now).unwrap();
 expect(s.disable_wifi(&w,&off,"same-intent",now),Reason::IntentConflict);
 expect(s.disable_wifi(&w,&off,"different-intent",now),Reason::Busy);
 drop(a.work);
}
#[test]fn disable_and_enable_completion_cannot_ack_the_opposite_direction(){
 for disabling in [false,true]{
  let(s,_,w,now)=environment(Config::default());disable_observe(&s,&w,now,true);let v=s.view(&w,now).unwrap();
  let a=if disabling{s.disable_wifi(&w,&v.disable_wifi_scope.unwrap(),"opposite-ack",now)}else{s.enable_wifi(&w,&v.enable_wifi_scope.unwrap(),"opposite-ack",now)}.unwrap();
  let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
  let result=if disabling{work.finish_enable_wifi(peer(1),now)}else{work.finish_disable_wifi(peer(1),now)};
  assert_eq!(result,Err(Reason::InvalidObservation));let t=s.ticket(&w,&a.ticket.id,now).unwrap();
  assert_eq!(t.phase,Phase::Unknown);assert!(!t.effect_confirmed&&t.refresh_recommended);
  assert!(s.view(&w,now).unwrap().stale);
 }
}
#[test]fn disable_three_scope_publication_preflights_capacity_and_counter_as_one_unit(){
 for counter_bound in [false,true]{
  let mut c=Config::default();if !counter_bound{c.max_bindings=2;}
  let(s,_,w,now)=environment(c);disable_observe(&s,&w,now,true);
  if counter_bound{s.state.lock().unwrap().next=u64::MAX-2;}
  expect(s.view(&w,now),Reason::Capacity);assert!(s.state.lock().unwrap().bindings.is_empty());
 }
}
