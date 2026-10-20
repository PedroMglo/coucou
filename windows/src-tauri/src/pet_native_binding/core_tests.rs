//! Deterministic Core/real lifecycle fixtures. Integer object identities are policy
//! stand-ins only; these tests do not certify GTK/compositor/DOM attachment.
use super::*;
use serde_json::json;
use std::time::Instant;
struct Env {m:Arc<Manager>,c:Arc<Core>,client:executor::Client,_inbox:executor::FixtureInbox,service:lifecycle::Service}
impl Env {
 fn new()->Self{
  let m=Manager::new(true);let c=m.register().unwrap();
  c.document(false,true).unwrap();c.observe_native(Facts{visible:true,mapped:true}).unwrap();
  let(client,inbox)=executor::fixture_client(&m.binding);let service=m.binding.service(lifecycle::Config::default(),123).unwrap();
  Self{m,c,client,_inbox:inbox,service}
 }
 fn mode(&self,target:Mode)->NavigationReceipt{let(p,_)=self.c.begin_trusted(target).unwrap();self.c.finish(&p,true).unwrap()}
 fn system(&self){self.mode(Mode::Expanded{route:Route::System});}
 fn prepared(&self)->PreparedContext{self.c.prepare(&self.client,self.c.context().unwrap().revision).unwrap()}
 fn read(&self)->lifecycle::Admission{self.prepared().prepared.fixture_read(&self.service).unwrap()}
 fn observe(&self)->String{
  let mut w=self.read().work.unwrap();w.begin_dispatch(peer(),Instant::now()).unwrap();
  w.finish_read(lifecycle::NativeObservation{producer:peer(),api_version:Some(34),capabilities:vec!["network".into()],result:json!({"wifiEnabled":true,"wifiNetworks":[]}),targets:vec![],scan_support:lifecycle::ScanSupport::VerifiedByAdapter,observed_at_ms:1_792_500_000_000},Instant::now()).unwrap();
  self.prepared().prepared.fixture_view(&self.service).unwrap().scan_scope.unwrap()
 }
}
fn peer()->lifecycle::ProducerId{lifecycle::ProducerId::from_verified_backend([7;16]).unwrap()}
fn request(revision:Revision,id:&str,kind:&str,route:Option<&str>)->NavigationRequest{
 let mut intent=json!({"kind":kind});if let Some(route)=route{intent["route"]=json!(route);}
 serde_json::from_value(json!({"expectedRevision":revision.0.to_string(),"intentId":id,"intent":intent})).unwrap()
}
fn code<T>(result:Result<T,Failure>,expected:ErrorCode){assert!(matches!(result,Err(Failure::Binding{code,..})if code==expected));}
#[test]fn plan01_same_label_different_gtk_denied(){assert!(!same_objects(&1,&9,&2,&9));}
#[test]fn plan02_same_gtk_different_webkit_denied(){assert!(!same_objects(&1,&9,&1,&10));assert!(same_objects(&1,&9,&1,&9));}
#[test]fn plan03_replacement_destroy_and_late_callback_cannot_regrant(){
 let e=Env::new();e.system();let old=e.prepared();let old_id=e.c.incarnation;
 let new=e.m.register().unwrap();assert_ne!(new.incarnation,old_id);assert!(!new.context().unwrap().system_eligible);
 assert!(old.validate().is_err());assert!(e.m.with_current(old_id,|c|c.observe_native(Facts{visible:true,mapped:true})).is_err());
 assert!(e.m.detach(old_id).unwrap().is_none());assert_eq!(e.m.current().unwrap().incarnation,new.incarnation);
 assert!(!e.c.context().unwrap().native_available);
}
#[test]fn plan04_missing_attachment_and_shutdown_fail_closed(){
 let m=Manager::new(true);assert!(m.current().is_err());m.shutdown();assert!(m.register().is_err());assert!(m.client().is_err());
}
#[test]fn plan05_visible_compact_is_not_system(){let e=Env::new();e.mode(Mode::Compact);assert!(!e.c.context().unwrap().system_eligible);assert!(e.c.prepare(&e.client,e.c.context().unwrap().revision).is_err());}
#[test]fn plan06_visible_pets_is_not_system(){let e=Env::new();e.mode(Mode::Expanded{route:Route::Sessions});assert!(!e.c.context().unwrap().system_eligible);assert!(e.c.prepare(&e.client,e.c.context().unwrap().revision).is_err());}
#[test]fn plan07_current_native_facts_committed_system_admits(){let e=Env::new();e.system();assert!(e.c.context().unwrap().system_eligible);let p=e.prepared();assert!(p.validate().is_ok());let w=p.prepared.fixture_read(&e.service).unwrap().work.unwrap();drop(w);}
#[test]fn plan08_stale_duplicate_and_conflict_use_current_context_without_replay(){
 let e=Env::new();let r=request(e.c.context().unwrap().revision,"first","expand",Some("system"));
 let Begin::Plan(p,_)=e.c.begin(r.clone()).unwrap()else{panic!()};
 let Begin::Duplicate(pending)=e.c.begin(r.clone()).unwrap()else{panic!()};assert!(!pending.applied&&pending.context.pending);
 let accepted=e.c.finish(&p,true).unwrap();e.mode(Mode::Compact);
 let current=e.c.context().unwrap();let Begin::Duplicate(dup)=e.c.begin(r.clone()).unwrap()else{panic!()};assert!(!dup.applied);assert_eq!(dup.context,current);assert_ne!(accepted.context.revision,dup.context.revision);
 let other=request(r.expected_revision,"second","expand",Some("system"));
 assert!(matches!(e.c.begin(other),Err(Failure::Binding{code:ErrorCode::StaleContext,context:Some(c)})if c==current));
 let conflict=request(r.expected_revision,"first","collapse",None);
 assert!(matches!(e.c.begin(conflict),Err(Failure::Binding{code:ErrorCode::IntentConflict,context:Some(c)})if c==current));
}
#[test]fn plan09_hide_show_revokes_original_preparation_and_work(){
 let e=Env::new();e.system();let p=e.prepared();let mut work=e.read().work.unwrap();
 e.c.observe_native(Facts{visible:false,mapped:false}).unwrap();e.c.observe_native(Facts{visible:true,mapped:true}).unwrap();
 assert!(e.c.context().unwrap().system_eligible);assert!(p.validate().is_err());assert!(p.prepared.fixture_read(&e.service).is_err());assert!(work.begin_dispatch(peer(),Instant::now()).is_err());
}
#[test]fn plan10_old_scan_scope_rejected_independently(){
 let e=Env::new();e.system();let scope=e.observe();let p=e.prepared();
 e.mode(Mode::Compact);e.system();assert!(p.validate().is_err());
 // Frozen lifecycle owns scope/lease validation independently of public revision.
 let new=e.prepared();assert!(new.prepared.fixture_scan(&e.service,&scope,"old-scope").is_err());
}
#[test]fn plan11_system_pets_system_and_same_system_child_revoke_old_fences(){
 let e=Env::new();e.system();let old=e.prepared();let mut work=e.read().work.unwrap();
 e.mode(Mode::Expanded{route:Route::Sessions});e.system();assert!(old.validate().is_err());assert!(work.begin_dispatch(peer(),Instant::now()).is_err());drop(work);
 let old=e.prepared();let(p,pending)=e.c.begin_trusted(Mode::Expanded{route:Route::System}).unwrap();assert!(!pending.system_eligible);assert!(old.validate().is_err());e.c.finish(&p,true).unwrap();assert!(old.validate().is_err());
}
#[test]fn plan12_settings_commit_aba_and_failed_persistence_are_distinct(){
 let e=Env::new();e.system();let old=e.prepared();let settings=Mutex::new(crate::Settings::default());let before=e.c.context().unwrap();
 let result=crate::settings::update_with(&settings,|s|s.pet_system_center=false,|_|Err(std::io::Error::other("fixture")),|s|{e.m.settings_committed(s.pet_system_center).unwrap();});
 assert!(result.is_err());assert_eq!(before,e.c.context().unwrap());assert!(old.validate().is_ok());
 crate::settings::update_with(&settings,|s|s.pet_system_center=false,|_|Ok(()),|s|{e.m.settings_committed(s.pet_system_center).unwrap();assert!(!e.c.context().unwrap().system_eligible);}).unwrap();
 e.m.settings_committed(true).unwrap();assert!(old.validate().is_err());assert!(!e.c.context().unwrap().system_eligible);e.system();assert!(old.validate().is_err());
}
#[test]fn plan13_first_map_remap_keeps_mode_but_never_old_lease(){
 let e=Env::new();e.system();let p=e.prepared();let rev=e.c.context().unwrap().revision;
 e.c.observe_native(Facts{visible:true,mapped:false}).unwrap();assert!(!e.c.context().unwrap().system_eligible);
 e.c.observe_native(Facts{visible:true,mapped:true}).unwrap();let now=e.c.context().unwrap();assert!(now.system_eligible);assert_ne!(now.revision,rev);assert!(p.validate().is_err());
}
#[test]fn plan14_hide_close_reload_crash_deny_real_lifecycle_writer(){
 for action in 0..4 {let e=Env::new();e.system();let mut work=e.read().work.unwrap();
  match action{0=>{e.c.observe_native(Facts{visible:false,mapped:false}).unwrap();},1=>{e.c.close().unwrap();},2=>{e.c.document(true,false).unwrap();e.c.document(false,true).unwrap();},_=>{e.c.crashed().unwrap();}}
  assert!(work.begin_dispatch(peer(),Instant::now()).is_err());
 }
}
#[test]fn plan15_superseded_failed_and_late_transition_cannot_regrant(){
 let e=Env::new();let(p,_)=e.c.begin_trusted(Mode::Expanded{route:Route::System}).unwrap();let(q,_)=e.c.begin_trusted(Mode::Compact).unwrap();
 code(e.c.finish(&p,true),ErrorCode::Superseded);e.c.finish(&q,true).unwrap();assert!(!e.c.context().unwrap().system_eligible);
 let(p,_)=e.c.begin_trusted(Mode::Expanded{route:Route::System}).unwrap();code(e.c.finish(&p,false),ErrorCode::NativeFailure);assert!(matches!(e.c.context().unwrap().mode,Mode::Inactive));
}
#[test]fn plan16_two_phase_native_callback_reentry_has_no_core_lock(){
 let e=Env::new();let(p,_)=e.c.begin_trusted(Mode::Expanded{route:Route::System}).unwrap();
 let _=e.c.layout(None,Some(&p)).unwrap();assert!(e.c.state.try_lock().is_ok());assert!(e.m.state.try_lock().is_ok());
 // This models synchronous map callbacks in the unlocked native apply phase.
 e.m.with_current(e.c.incarnation,|c|c.observe_native(Facts{visible:true,mapped:false})).unwrap();
 code(e.c.finish(&p,true),ErrorCode::Superseded);assert!(!e.c.context().unwrap().system_eligible);
}
#[test]fn plan17_query_geometry_keyboard_cannot_authorize_or_renew(){
 let e=Env::new();e.mode(Mode::Compact);let before=e.c.context().unwrap();
 for _ in 0..4{let l=e.c.layout(Some(Hint{x:0.,y:0.,w:900.,h:900.,keyboard:true}),None).unwrap();assert!(!l.expanded&&!l.keyboard);assert_eq!(l.rects[0].3,32.);assert_eq!(e.c.context().unwrap(),before);}
 e.system();let before=e.c.context().unwrap();let p=e.prepared();e.c.layout(Some(Hint{x:0.,y:0.,w:800.,h:900.,keyboard:true}),None).unwrap();assert_eq!(e.c.context().unwrap(),before);assert!(p.validate().is_ok());
 assert!(e.c.layout(Some(Hint{x:f64::NAN,..Hint::default()}),None).is_err());
 e.c.observe_native(Facts{visible:false,mapped:false}).unwrap();e.c.observe_native(Facts{visible:true,mapped:true}).unwrap();assert!(!e.c.layout(None,None).unwrap().keyboard);
 e.c.layout(Some(Hint{x:0.,y:0.,w:100.,h:100.,keyboard:true}),None).unwrap();e.system();assert!(!e.c.layout(None,None).unwrap().keyboard);
}
#[test]fn plan18_accepted_intent_cannot_attest_renderer_dom_or_gesture(){
 let e=Env::new();let _renderer_still_drawing_pets=true;e.system();let c=e.c.context().unwrap();assert!(c.system_eligible);assert_eq!(c.authority_contract,"nativeIdentityAndAcceptedIntent");
 let wire=serde_json::to_value(c).unwrap();assert!(wire.get("domVerified").is_none());assert!(wire.get("gestureVerified").is_none());
}
#[test]fn plan19_after_write_revoke_is_unknown_and_work_retains_slot(){
 let e=Env::new();e.system();let a=e.read();let mut work=a.work.unwrap();work.begin_dispatch(peer(),Instant::now()).unwrap();
 e.c.observe_native(Facts{visible:false,mapped:false}).unwrap();e.c.observe_native(Facts{visible:true,mapped:true}).unwrap();
 let ticket=e.prepared().prepared.fixture_ticket(&e.service,&a.ticket.id).unwrap();assert_eq!(ticket.phase,lifecycle::Phase::Unknown);assert!(!ticket.effect_confirmed);
 assert_eq!(e.prepared().prepared.fixture_read(&e.service).err(),Some(lifecycle::Reason::Busy));drop(work);assert!(e.prepared().prepared.fixture_read(&e.service).unwrap().work.is_some());
}
#[test]fn plan20_future_audio_guard_after_cache_wait_rejects_old_fence(){let e=Env::new();e.system();let guard=e.prepared();e.m.settings_committed(false).unwrap();e.m.settings_committed(true).unwrap();e.system();assert!(guard.validate().is_err());assert!(e.prepared().validate().is_ok());}
#[test]fn plan21_reload_old_document_and_uri_allowlist_fail_closed(){
 let e=Env::new();e.system();let p=e.prepared();let rev=e.c.context().unwrap().revision;e.c.document(true,false).unwrap();e.c.document(false,true).unwrap();assert!(p.validate().is_err());assert!(!e.c.context().unwrap().system_eligible);code(e.c.prepare(&e.client,rev),ErrorCode::StaleContext);
 let policy=crate::pet_native_binding::Policy{dev:None};
 assert!(policy.allowed(&"tauri://localhost/pet-control.html".parse().unwrap()));
 for url in ["tauri://localhost/companions.html","https://example.com/pet-control.html","file:///tmp/pet-control.html","tauri://other/pet-control.html","tauri://localhost/pet-control.html?route=system","tauri://user@localhost/pet-control.html"]{assert!(!policy.allowed(&url.parse().unwrap()),"{url}");}
 let dev=crate::pet_native_binding::Policy{dev:Some("http://localhost:1420/".parse().unwrap())};assert!(dev.allowed(&"http://localhost:1420/pet-control.html".parse().unwrap()));assert!(!dev.allowed(&"http://localhost:1421/pet-control.html".parse().unwrap()));
}
#[test]fn plan22_same_window_historical_ticket_is_inspection_not_old_action_grant(){
 let e=Env::new();e.system();let a=e.read();let old=e.prepared();drop(a.work);e.mode(Mode::Compact);e.system();
 assert!(old.prepared.fixture_ticket(&e.service,&a.ticket.id).is_err());
 let current=e.prepared();assert!(current.prepared.fixture_ticket(&e.service,&a.ticket.id).is_ok());let view=current.prepared.fixture_view(&e.service).unwrap();assert!(view.scan_scope.is_none());
 assert!(current.prepared.fixture_scan(&e.service,"not-a-current-scope","inspect-only").is_err());
}
#[test]fn wire_revision_and_request_validation_are_bounded_and_exact(){
 assert_eq!(serde_json::to_value(Revision(u64::MAX)).unwrap(),json!(u64::MAX.to_string()));
 for value in [json!(1),json!("01"),json!("-1"),json!("18446744073709551616"),json!(""),json!("1\n")]{assert!(serde_json::from_value::<Revision>(value).is_err());}
 for request in [json!({"expectedRevision":"1","intentId":"x","intent":{"kind":"expand","route":"unknown"}}),json!({"expectedRevision":"1","intentId":"x","intent":{"kind":"collapse","route":"system"}}),json!({"expectedRevision":"1","intentId":"a b","intent":{"kind":"collapse"}}),json!({"expectedRevision":"1","intentId":"x","intent":{"kind":"collapse"},"window":"pet-control"})]{assert!(serde_json::from_value::<NavigationRequest>(request).is_err());}
 let err=serde_json::to_value(Failure::binding(ErrorCode::StaleContext,None)).unwrap();assert_eq!(err,json!({"kind":"binding","code":"staleContext","context":null}));
}
#[test]fn poisoned_manager_revokes_prepared_authority_before_returning_unavailable(){
 let e=Env::new();e.system();let prepared=e.prepared();
 let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{let _guard=e.m.state.lock().unwrap();panic!("fixture poison");}));
 assert!(e.m.current().is_err());assert!(prepared.prepared.validate().is_err());
}

fn enable_binding_observe(e:&Env)->String {
 let mut w=e.read().work.unwrap();w.begin_dispatch(peer(),Instant::now()).unwrap();
 w.finish_read(lifecycle::NativeObservation{producer:peer(),api_version:Some(34),capabilities:vec!["network".into()],result:json!({"backend":"networkmanager","wifiEnabled":false,"wifiDevices":[{"name":"wlan0","hwAddress":"","state":"disconnected","connected":false,"apCapable":false,"networks":null}]}),targets:vec![],scan_support:lifecycle::ScanSupport::VerifiedByAdapter,observed_at_ms:1_792_500_000_000},Instant::now()).unwrap();
 e.prepared().prepared.fixture_view(&e.service).unwrap().enable_wifi_scope.unwrap()
}
#[test]fn enable_checked_request_has_only_revision_scope_and_intent(){
 let good=json!({"expectedRevision":"1","scope":"opaque-scope","intentId":"enable_1"});assert!(serde_json::from_value::<EnableWifiRequest>(good.clone()).is_ok());
 for (key,value)in [("expectedRevision",json!(1)),("scope",json!("x".repeat(129))),("intentId",json!("bad intent")),("enabled",json!(true)),("device",json!("wlan0")),("target",json!("native")),("method",json!("network.wifi.disable"))]{let mut v=good.clone();v[key]=value;assert!(serde_json::from_value::<EnableWifiRequest>(v).is_err());}
}
#[test]fn enable_original_native_prepared_context_is_never_reminted_after_route_roundtrip(){
 let e=Env::new();e.system();let scope=enable_binding_observe(&e);let original=e.prepared();e.mode(Mode::Compact);e.system();assert!(original.prepared.fixture_enable_wifi(&e.service,&scope,"old-original").is_err());
 let view=e.prepared().prepared.fixture_view(&e.service).unwrap();assert!(!view.refreshing);
}
