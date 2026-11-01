// Included inside the existing portal tests: same private no-autolaunch bus,
// same stand-in native peer; no compositor, physical keys or host services.
fn review_wanted(id:&str,hold:bool)->Vec<Wanted>{vec![Wanted{id:id.into(),description:if hold{"hold fixture"}else{"private reviewed fixture"}.into(),trigger:Some("CTRL+ALT+space".into())}]}
fn review_true()->review::Guard{std::sync::Arc::new(||true)}
fn review_flag(flag:&std::sync::Arc<std::sync::atomic::AtomicBool>)->review::Guard{let flag=flag.clone();std::sync::Arc::new(move||flag.load(std::sync::atomic::Ordering::Acquire))}
fn review_events_empty(rx:&mpsc::Receiver<Event>){assert!(matches!(rx.recv_timeout(Duration::from_millis(150)),Err(mpsc::RecvTimeoutError::Timeout)),"published before a verified durable commit");}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_reservation_is_consumed_exactly_and_only_publishes_after_durable_commit(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();
 let plan=handle.reserve(1,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();assert_eq!(plan.previous.phase,"idle");
 let source=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));let outcome=handle.apply_reserved(plan.clone(),review_flag(&source),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(outcome.verified);assert!(outcome.native_effects_started);assert_eq!(outcome.session.as_deref(),Some(plan.session.as_str()));assert_eq!(outcome.shortcuts.unwrap()[0].id,"openChat");review_events_empty(&events);
 // Own Verified journal transition invalidates the old Dispatched witness.
 // It must not revoke a waiting observation before its fresh commit witness.
 source.store(false,std::sync::atomic::Ordering::Release);review_events_empty(&events);
 assert!(handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:1,..}));review_events_empty(&events);
 let replay=handle.apply_reserved(plan,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(!replay.verified&&!replay.native_effects_started);review_events_empty(&events);
}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_cancel_is_observed_while_native_prompt_waits_and_does_not_repeat_io(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();
 let plan=handle.reserve(1,review_wanted("openChat",true)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();let alive=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));let outcome=handle.apply_reserved(plan,review_true(),review_flag(&alive)).unwrap();
 assert!(matches!(outcome.recv_timeout(Duration::from_millis(200)),Err(mpsc::RecvTimeoutError::Timeout)));alive.store(false,std::sync::atomic::Ordering::Release);
 let observed=outcome.recv_timeout(Duration::from_secs(3)).unwrap();assert!(!observed.verified&&observed.native_effects_started);assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Failed{gen:1,..}));assert!(!handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());review_events_empty(&events);
 let ch=Channel::get_private(BusType::Session).unwrap();assert_eq!(ch.send_with_reply_and_block(method(OBJECT,IFACE,"FixtureOperationCounts").unwrap(),CALL_TIMEOUT).unwrap().read2::<u32,u32>().unwrap(),(1,1));
}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_partial_and_duplicate_native_lists_never_publish_complete_success(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();
 for id in ["omitMe","duplicateMe"]{let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();let plan=handle.reserve(1,review_wanted(id,false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
 let observed=handle.apply_reserved(plan,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(!observed.verified&&observed.native_effects_started);review_events_empty(&events);assert!(!handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Failed{gen:1,..}));}
}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_unreviewed_request_or_session_is_not_followed_as_a_fallback(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();
 for wrong in ["wrong_request","wrong_session"]{let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();let mut plan=handle.reserve(1,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
 if wrong=="wrong_request"{plan.create_request=format!("{}/wrong_request",plan.create_request.rsplit_once('/').unwrap().0);}else{plan.session=format!("{}/wrong_session",plan.session.rsplit_once('/').unwrap().0);}
 let observed=handle.apply_reserved(plan,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(!observed.verified&&observed.native_effects_started);assert!(observed.reason.contains("unreviewed"));assert!(!handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Failed{gen:1,..}));}
 let ch=Channel::get_private(BusType::Session).unwrap();assert_eq!(ch.send_with_reply_and_block(method(OBJECT,IFACE,"FixtureOperationCounts").unwrap(),CALL_TIMEOUT).unwrap().read2::<u32,u32>().unwrap(),(2,0));
}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_changed_previous_state_is_refused_without_touching_existing_binding(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();
 let plan=handle.reserve(2,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();assert!(handle.bind(1,review_wanted("openChat",false)));assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:1,..}));assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Activated{gen:1,..}));
 let observed=handle.apply_reserved(plan,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(!observed.verified&&!observed.native_effects_started);review_events_empty(&events);
 let next=handle.reserve(2,vec![]).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();assert_eq!(next.previous.phase,"bound");assert_eq!(next.previous.generation.as_deref(),Some("1"));assert!(next.previous.session.is_some());
 let observed=handle.apply_reserved(next,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(observed.verified&&observed.native_effects_started);assert_eq!(observed.shortcuts,Some(vec![]));review_events_empty(&events);assert!(handle.finish_reserved(2,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:2,..}));
}
#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_cancel_after_observation_cannot_publish_keys_with_a_new_commit_witness(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();let plan=handle.reserve(1,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
 let alive=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));assert!(handle.apply_reserved(plan,review_true(),review_flag(&alive)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().verified);review_events_empty(&events);alive.store(false,std::sync::atomic::Ordering::Release);
 assert!(!handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Failed{gen:1,..}));review_events_empty(&events);
}

#[test]
#[ignore="synthetic reviewed protocol, requires private no-autolaunch dbus-run-session"]
fn reviewed_portal_publication_without_final_receipt_cleans_only_its_exact_generation(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();let plan=handle.reserve(1,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();
 assert!(handle.apply_reserved(plan,review_true(),review_true()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().verified);assert!(handle.finish_reserved(1,Some(review_true())).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Bound{gen:1,..}));
 assert!(!handle.cleanup_published(2).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());review_events_empty(&events);
 assert!(handle.cleanup_published(1).unwrap().recv_timeout(Duration::from_secs(3)).unwrap());assert!(matches!(events.recv_timeout(Duration::from_secs(3)).unwrap(),Event::Failed{gen:1,..}));
 let next=handle.reserve(2,review_wanted("openChat",false)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();assert_eq!(next.previous.phase,"idle");
}

#[test]
#[ignore="synthetic protocol; private no-autolaunch session bus"]
fn shortcut_observation_uses_close_acks_and_refuses_different_bus_or_peer_without_native_replay(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let(tx,_events)=mpsc::channel();let handle=start(move|e|{let _=tx.send(e);}).unwrap();let plan=handle.reserve(1,review_wanted("openChat",true)).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().unwrap();let alive=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));let outcome=handle.apply_reserved(plan.clone(),review_true(),review_flag(&alive)).unwrap();assert!(matches!(outcome.recv_timeout(Duration::from_millis(150)),Err(mpsc::RecvTimeoutError::Timeout)));alive.store(false,std::sync::atomic::Ordering::Release);assert!(!outcome.recv_timeout(Duration::from_secs(3)).unwrap().verified);
 let subject=observation::Subject{bus_id:plan.bus_id.clone(),client:plan.client.clone(),peer:plan.peer.clone(),sessions:vec![plan.session.clone()]};let value=handle.observe_subject(subject.clone()).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(value.bus_matches&&value.sessions_closed);assert_eq!(value.original_client_live,Some(true));
 let mut wrong=subject.clone();wrong.bus_id="0".repeat(32);let value=handle.observe_subject(wrong).unwrap().recv_timeout(Duration::from_secs(3)).unwrap();assert!(!value.bus_matches&&!value.sessions_closed);assert_eq!(value.original_client_live,None);
 let mut wrong=subject;wrong.peer=":1.99999".into();assert!(!handle.observe_subject(wrong).unwrap().recv_timeout(Duration::from_secs(3)).unwrap().sessions_closed);
 let ch=Channel::get_private(BusType::Session).unwrap();assert_eq!(ch.send_with_reply_and_block(method(OBJECT,IFACE,"FixtureOperationCounts").unwrap(),CALL_TIMEOUT).unwrap().read2::<u32,u32>().unwrap(),(1,1));
}
#[test]
#[ignore="synthetic protocol; private no-autolaunch session bus"]
fn shortcut_observation_requires_original_client_absence_on_the_same_actual_bus(){
 let _bus=BUS_TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _fake=StandIn::new();let mut original=Portal::connect().unwrap();let plan=original.reserve(1,review_wanted("openChat",false)).unwrap();let observer=Portal::connect().unwrap();let subject=observation::Subject{bus_id:plan.bus_id,client:plan.client,peer:plan.peer,sessions:vec![plan.session]};let live=observer.observe_subject(subject.clone());assert_eq!(live.original_client_live,Some(true));assert!(!live.sessions_closed);drop(original);let absent=observer.observe_subject(subject);assert_eq!(absent.original_client_live,Some(false));assert!(absent.sessions_closed);assert_eq!(absent.original_peer_live,Some(true));
}
