use super::*;
use crate::settings::EffectFixture;
use std::sync::{Arc,atomic::AtomicBool};

fn fixture_context(s:&Settings,p:&Path)->std::io::Result<Context>{
    let mut c=native_context(s,p)?;assert!(!crate::pet_data::bootstrap::admitted());
    c.profile=Some(hash(p.parent().unwrap().as_os_str().as_encoded_bytes()));Ok(c)
}
struct Fixture{files:EffectFixture,controller:Controller,owner:settings::review::Controller,frame:Frame,native_alive:Arc<AtomicBool>}
impl Fixture{
    fn new()->Self{let files=EffectFixture::new();files.save();let owner=settings::review::Controller::default();owner.register();let frame=owner.frame().unwrap();Self{files,controller:Controller::default(),owner,frame,native_alive:Arc::new(AtomicBool::new(true))}}
    fn prepare(&self)->Result<Preview,String>{self.controller.prepare_with(&self.frame,&self.files.current,&self.files.path,fixture_context,|_|{let alive=self.native_alive.clone();Ok(NativeCapture{destination:Destination::Plugin{instance:"a".repeat(32),previous:vec![],owner:crate::pet_runtime::native_process_identity::capture().unwrap(),environment:Environment{display:Some(":private-fixture".into()),wayland_display:None,session_type:Some("x11".into())}},current:Box::new(move||alive.load(Ordering::Acquire))})})}
    fn key(p:&Preview)->Key{Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}}
    fn inspect(&self,p:&Preview)->Result<Preview,String>{self.controller.inspect_with(&self.frame,&self.files.current,&Self::key(p),fixture_context)}
}
#[test]
fn shortcut_review_preparation_inspection_and_cancel_leave_persisted_data_and_authority_unchanged(){
    let f=Fixture::new();let before=std::fs::read(&f.files.path).unwrap();let p=f.prepare().unwrap();
    assert!(!p.activation_allowed);assert_eq!(p.scope,"nativeUserShortcutsOnly");assert_eq!(p.actions.len(),ACTIONS.len());
    assert_eq!(f.inspect(&p).unwrap().fingerprint,p.fingerprint);
    assert_eq!(std::fs::read(&f.files.path).unwrap(),before);assert!(!f.files.path.parent().unwrap().join(components::DIRECTORY).exists());
    assert!(fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().slot.is_none());
    f.controller.cancel(&f.frame,Fixture::key(&p)).unwrap();assert!(f.inspect(&p).is_err());
    assert!(f.controller.cancel(&f.frame,Fixture::key(&p)).is_err());assert_eq!(std::fs::read(&f.files.path).unwrap(),before);
}
#[test]
fn shortcut_review_old_key_wrong_frame_expiry_and_native_mutation_are_refused(){
    let f=Fixture::new();let p=f.prepare().unwrap();let second=f.prepare().unwrap();assert!(f.inspect(&p).is_err());
    let other=settings::review::Controller::default();other.register();let frame=other.frame().unwrap();
    assert!(f.controller.inspect_with(&frame,&f.files.current,&Fixture::key(&second),fixture_context).is_err());
    assert!(f.controller.cancel(&frame,Fixture::key(&second)).is_err());
    f.native_alive.store(false,Ordering::Release);assert!(f.inspect(&second).is_err());f.native_alive.store(true,Ordering::Release);
    f.controller.pending.lock().unwrap().as_mut().unwrap().expires=Instant::now()-Duration::from_secs(1);assert!(f.inspect(&second).is_err());
    let next=f.prepare().unwrap();f.owner.register();assert!(f.inspect(&next).is_err());
}
#[test]
fn shortcut_review_exact_loaded_settings_and_native_context_are_not_renewed_by_inspection(){
    let f=Fixture::new();let p=f.prepare().unwrap();
    f.files.update(|s|s.sound_volume=0.5);assert!(f.inspect(&p).is_err());
    let next=f.prepare().unwrap();std::fs::write(&f.files.path,b"external different settings").unwrap();assert!(f.inspect(&next).is_err());
    let f=Fixture::new();let p=f.prepare().unwrap();let mut key=Fixture::key(&p);key.fingerprint="f".repeat(64);assert!(f.controller.inspect_with(&f.frame,&f.files.current,&key,fixture_context).is_err());assert!(f.inspect(&p).is_ok());
    for value in [serde_json::json!({"token":p.token,"fingerprint":p.fingerprint,"peer":":1.99"}),serde_json::json!({"token":p.token,"fingerprint":p.fingerprint,"activate":true})]{assert!(serde_json::from_value::<Key>(value).is_err());}
}
#[test]
fn shortcut_review_action_projection_reuses_existing_planner_and_preserves_unknown_settings(){
    let mut stored=Bindings::new();stored.insert("openChat".into(),Binding{keys:"Ctrl+Alt+A".into(),enabled:true});
    stored.insert("unknownPreviousAction".into(),Binding{keys:"Super+P".into(),enabled:true});
    let views=actions(&stored).unwrap();assert_eq!(views.iter().find(|a|a.id=="goToAlert").unwrap().planned_status,Status::Duplicate);
    assert_eq!(views.iter().find(|a|a.id=="attachFrontWindow").unwrap().planned_status,Status::NotPorted);
    assert_eq!(views.iter().find(|a|a.id=="toggleIsland").unwrap().planned_status,Status::Off);
    assert_eq!(stored["unknownPreviousAction"].keys,"Super+P");
    let f=Fixture::new();f.files.update(|s|s.shortcuts=stored);let p=f.prepare().unwrap();assert_eq!(p.unknown_action_ids,vec!["unknownPreviousAction"]);assert_eq!(p.desired_bindings["unknownPreviousAction"].keys,"Super+P");
}
#[test]
fn shortcut_review_resource_is_same_registrar_across_keys_and_new_paths_but_not_another_peer(){
    let mut reservation=crate::portal::Reservation{generation:9,bus_id:"e".repeat(32),peer:":1.10".into(),client:":1.20".into(),session:"/fixture/s1".into(),create_request:"/fixture/c1".into(),bind_request:"/fixture/b1".into(),shortcuts:vec![],previous:crate::portal::review::Before::idle()};
    let environment=Environment{display:None,wayland_display:Some("private-1".into()),session_type:Some("wayland".into())};
    let dest=|r|Destination::Portal{instance:"a".repeat(32),reservation:r,environment:environment.clone(),owner:crate::pet_runtime::native_process_identity::capture().unwrap()};
    let before=dest(reservation.clone());reservation.generation=10;reservation.session="/fixture/s2".into();reservation.create_request="/fixture/c2".into();reservation.bind_request="/fixture/b2".into();reservation.shortcuts.push(crate::portal::Wanted{id:"openChat".into(),description:"fixture".into(),trigger:Some("CTRL+ALT+space".into())});
    let after=dest(reservation.clone());assert_eq!(before.resource().unwrap(),after.resource().unwrap());assert_ne!(encoded(&before).unwrap(),encoded(&after).unwrap());
    reservation.peer=":1.11".into();assert_ne!(before.resource().unwrap(),dest(reservation).resource().unwrap());
}

fn decision(p:&Preview)->Decision{Decision{token:p.token.clone(),fingerprint:p.fingerprint.clone(),reviewed_plan:true,understand_native_effects:true,keep_other_components_suspended:true,preview_schema_version:None,acknowledge_retained_uncertainty:false}}
fn observation(verified:bool,started:bool)->crate::portal::review::Outcome{crate::portal::review::Outcome{verified,native_effects_started:started,reason:"synthetic native adapter observation".into(),session:None,shortcuts:Some(vec![])}}
#[test]
fn shortcut_review_approval_requires_three_exact_acknowledgements_and_refuses_caller_overrides(){
 let f=Fixture::new();let p=f.prepare().unwrap();
 for n in 0..3{let mut d=decision(&p);match n{0=>d.reviewed_plan=false,1=>d.understand_native_effects=false,_=>d.keep_other_components_suspended=false};assert!(f.controller.apply_with(&f.frame,&f.files.current,d,fixture_context,Arc::new(||true),|_,_,_|panic!("unapproved dispatch")).is_err());assert!(f.inspect(&p).is_ok());}
 for extra in ["caller","peer","desiredBindings","previous","provider"]{let mut v=serde_json::json!({"token":p.token,"fingerprint":p.fingerprint,"reviewedPlan":true,"understandNativeEffects":true,"keepOtherComponentsSuspended":true});v[extra]=serde_json::json!("forged");assert!(serde_json::from_value::<Decision>(v).is_err());}
}
#[test]
fn shortcut_review_retains_verified_native_fact_before_publication_and_commits_completion_afterward(){
 let f=Fixture::new();let p=f.prepare().unwrap();let path=f.files.path.clone();let current=f.files.current.clone();let published=Arc::new(AtomicBool::new(false));let flag=published.clone();
 let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),move|capture,guard,_|{
  assert!(current.try_lock().is_ok(),"Shared settings lock held during native IO");assert!(guard());
  let c=fixture_context(&current.lock().unwrap(),&path).unwrap();let slot=c.slot.unwrap();assert_eq!(slot.phase,components::Phase::Dispatched);assert_eq!(slot.binding.plan,capture.preview.fingerprint);
  NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|source,lifetime|{assert!(lifetime());assert!(!guard(),"old dispatch witness renewed after own transition");assert!(source.unwrap().current());let slot=fixture_context(&current.lock().unwrap(),&path).unwrap().slot.unwrap();assert_eq!(slot.phase,components::Phase::Unconfirmed);let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(path.parent().unwrap().join(components::DIRECTORY).join(format!("component-{}.json",slot.outcome.unwrap().id))).unwrap()).unwrap();assert_eq!(raw["evidence"]["nativeObservation"]["verified"],true);assert_eq!(raw["evidence"]["publication"],"pending");flag.store(true,Ordering::Release);Publication::live(Arc::new(||true),||panic!("committed publication aborted"))}))}
 }).unwrap();assert!(published.load(Ordering::Acquire));assert_eq!(receipt.registration,"verified");assert_eq!(receipt.publication,"published");assert!(receipt.status.suspended_operations.iter().any(|s|s=="shortcuts"));
 assert!(f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|panic!("replayed approval")).is_err());
 let restart=settings::load_from(&f.files.path);assert!(settings::io_status(&restart).suspended_operations.iter().any(|s|s=="shortcuts"));
}
#[test]
fn shortcut_review_cancellation_bypasses_running_gate_but_never_frees_it_before_original_worker_returns(){
 let f=Fixture::new();let p=f.prepare().unwrap();let (ready_tx,ready_rx)=std::sync::mpsc::channel();let(release_tx,release_rx)=std::sync::mpsc::channel();
 std::thread::scope(|scope|{let f=&f;let p=&p;let worker=scope.spawn(move||f.controller.apply_with(&f.frame,&f.files.current,decision(p),fixture_context,Arc::new(||true),|_,guard,_|{
  assert!(guard());ready_tx.send(()).unwrap();release_rx.recv_timeout(Duration::from_secs(3)).unwrap();assert!(!guard());
  NativeRun{outcome:observation(false,true),finish:Some(Box::new(|source,lifetime|{assert!(source.is_none());assert!(!lifetime());Publication::refused(None)}))}
 }));ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
 assert!(f.prepare().is_err());let wrong=settings::review::Controller::default();wrong.register();assert!(f.controller.cancel(&wrong.frame().unwrap(),Fixture::key(p)).is_err());
 f.controller.cancel(&f.frame,Fixture::key(p)).unwrap();assert!(f.controller.gate.try_lock().is_err());assert!(f.prepare().is_err());assert!(f.files.current.try_lock().is_ok());
 release_tx.send(()).unwrap();let receipt=worker.join().unwrap().unwrap();assert_eq!(receipt.registration,"unconfirmed");assert_eq!(receipt.publication,"notPublished");
 });assert!(f.controller.gate.try_lock().is_ok());assert!(f.prepare().is_err(),"uncertain original cannot be replaced by a new review");
}
#[test]
fn shortcut_review_no_native_dispatch_is_proved_and_replaceable_but_partial_dispatch_remains_uncertain(){
 for started in [false,true]{let f=Fixture::new();let p=f.prepare().unwrap();let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|NativeRun{outcome:observation(false,started),finish:Some(Box::new(|source,_|{assert!(source.is_none());Publication::refused(None)}))}).unwrap();assert_eq!(receipt.registration,if started{"unconfirmed"}else{"notDispatched"});assert_eq!(f.prepare().is_ok(),!started);}
}
#[test]
fn shortcut_review_source_change_or_closed_owner_during_io_cannot_publish_success(){
 for close in [false,true]{let f=Fixture::new();let p=f.prepare().unwrap();let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,guard,_|{
  if close{f.owner.register();}else{f.files.external();}assert!(!guard());
  NativeRun{outcome:observation(true,true),finish:Some(Box::new(|source,_|{assert!(source.is_none());Publication::refused(None)}))}
 }).unwrap();assert_eq!(receipt.registration,"unconfirmed");assert_ne!(receipt.publication,"published");}
}
#[test]
fn shortcut_review_durable_outcome_failure_releases_worker_and_keeps_uncertain_dispatch_without_retry(){
 let f=Fixture::new();let p=f.prepare().unwrap();let cleaned=Arc::new(AtomicBool::new(false));let flag=cleaned.clone();
 let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|{
  // Removing the exact retained intent makes a successful native observation
  // insufficient to publish a durable outcome. No effect is repeated.
  let c=fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap();let reference=c.slot.unwrap().intent;
  std::fs::remove_file(f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",reference.id))).unwrap();
  NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|source,_|{assert!(source.is_none());flag.store(true,Ordering::Release);Publication::refused(None)}))}
 }).unwrap();assert_eq!(receipt.registration,"unconfirmed");assert!(cleaned.load(Ordering::Acquire));assert!(f.controller.gate.try_lock().is_ok());assert!(f.prepare().is_err());
}

#[test]
fn shortcut_release_uses_exact_owned_keys_without_registry_locks_and_retains_failed_or_undispatched_keys(){
 let registry=Registry::default();let a=parse("Ctrl+Alt+A").unwrap();let b=parse("Ctrl+Alt+B").unwrap();
 for(action,key)in[("openChat",a),("goToAlert",b)]{registry.by_id.lock().unwrap().insert(key.id(),Registered{action,shortcut:key,published:true});}
 let keys=vec![(a.id(),a),(b.id(),b)];let calls=std::sync::atomic::AtomicUsize::new(0);
 let outcome=release_owned(&registry,&keys,||true,|key|{assert!(registry.by_id.try_lock().is_ok());calls.fetch_add(1,Ordering::Relaxed);key==a});
 assert!(outcome.started&&!outcome.complete);assert_eq!(calls.load(Ordering::Relaxed),2);let rows=registry.by_id.lock().unwrap();assert!(!rows.contains_key(&a.id()));assert!(!rows[&b.id()].published);drop(rows);
 let result=release_owned(&registry,&[(b.id(),b)],||false,|_|panic!("revoked release executed"));assert!(!result.started&&!result.complete);assert!(registry.by_id.lock().unwrap().contains_key(&b.id()));
 let result=release_owned(&registry,&[(b.id(),b)],||true,|_|true);assert!(result.started&&result.complete);assert!(registry.by_id.lock().unwrap().is_empty());
}

#[test]
fn shortcut_verified_native_result_with_failed_publication_stays_uncertain_for_new_review_and_restart(){
 let f=Fixture::new();let p=f.prepare().unwrap();let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|NativeRun{outcome:observation(true,true),finish:Some(Box::new(|source,_|{assert!(source.unwrap().current());Publication::refused(Some(Cleanup{attempted:true,acknowledged:false,reason:"synthetic lost cleanup ACK"}))}))}).unwrap();
 assert_eq!(receipt.registration,"verified");assert_eq!(receipt.publication,"unconfirmed");assert!(!receipt.cleanup.unwrap().acknowledged);assert_eq!(fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().slot.unwrap().phase,components::Phase::Unconfirmed);assert!(f.prepare().is_err());
 *f.files.current.lock().unwrap()=settings::load_from(&f.files.path);assert!(f.prepare().is_err());assert!(settings::io_status(&f.files.current.lock().unwrap()).suspended_operations.iter().any(|s|s=="shortcuts"));
}
#[test]
fn shortcut_revocation_after_local_publication_cleans_own_resources_and_never_finishes_the_pending_result(){
 let f=Fixture::new();let p=f.prepare().unwrap();let cleaned=Arc::new(AtomicBool::new(false));let flag=cleaned.clone();
 let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|capture,_,_|{let alive=capture.alive.clone();NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|source,_|{assert!(source.unwrap().current());alive.store(false,Ordering::Release);Publication::live(Arc::new(||true),move||{flag.store(true,Ordering::Release);Cleanup{attempted:true,acknowledged:true,reason:"synthetic exact-key cleanup"}})}))}}).unwrap();
 assert!(cleaned.load(Ordering::Acquire));assert_eq!(receipt.publication,"unconfirmed");assert!(receipt.cleanup.unwrap().acknowledged);assert_eq!(fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().slot.unwrap().phase,components::Phase::Unconfirmed);
}
#[test]
fn shortcut_final_receipt_write_failure_after_local_publication_keeps_pending_and_calls_cleanup(){
 use std::os::unix::fs::PermissionsExt;
 let f=Fixture::new();let p=f.prepare().unwrap();let dir=f.files.root.join(components::DIRECTORY);let denied=dir.clone();let cleaned=Arc::new(AtomicBool::new(false));let flag=cleaned.clone();
 let receipt=f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|source,_|{assert!(source.unwrap().current());std::fs::set_permissions(&denied,std::fs::Permissions::from_mode(0o500)).unwrap();Publication::live(Arc::new(||true),move||{flag.store(true,Ordering::Release);Cleanup{attempted:true,acknowledged:false,reason:"synthetic cleanup outcome unknown"}})}))}).unwrap();
 std::fs::set_permissions(&dir,std::fs::Permissions::from_mode(0o700)).unwrap();assert!(cleaned.load(Ordering::Acquire));assert_eq!(receipt.publication,"unconfirmed");assert!(!receipt.cleanup.unwrap().acknowledged);*f.files.current.lock().unwrap()=settings::load_from(&f.files.path);assert!(f.prepare().is_err());
}
#[test]
fn shortcut_panic_while_pending_or_while_a_publication_handle_is_armed_cannot_turn_into_new_authority(){
 for armed in [false,true]{let f=Fixture::new();let p=f.prepare().unwrap();let cleaned=Arc::new(AtomicBool::new(false));let flag=cleaned.clone();
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|_,_|{let _publication=armed.then(||Publication::live(Arc::new(||true),move||{flag.store(true,Ordering::Release);Cleanup{attempted:true,acknowledged:true,reason:"synthetic panic cleanup"}}));panic!("synthetic cut before final receipt")}))})));assert!(result.is_err());assert_eq!(cleaned.load(Ordering::Acquire),armed);let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(f.files.root.join(settings::durable::FILE)).unwrap()).unwrap();assert_eq!(raw["components"]["shortcuts"]["phase"],"unconfirmed");
 *f.files.current.lock().unwrap()=settings::load_from(&f.files.path);let fresh=Controller::default();assert!(fresh.prepare_with(&f.frame,&f.files.current,&f.files.path,fixture_context,|_|panic!("pending result reached native preparation")).is_err());}
}

fn recovery_native(portal:bool,instance:&str)->NativeCapture{
 let owner=crate::pet_runtime::native_process_identity::capture().unwrap();let environment=Environment{display:Some(":99".into()),wayland_display:portal.then(||"private-fixture".into()),session_type:Some(if portal{"wayland"}else{"x11"}.into())};
 let destination=if portal{Destination::Portal{instance:instance.repeat(32),environment,owner,reservation:crate::portal::Reservation{generation:2,bus_id:"b".repeat(32),peer:":1.10".into(),client:":1.20".into(),session:"/org/freedesktop/portal/desktop/session/1_20/new_session".into(),create_request:"/org/freedesktop/portal/desktop/request/1_20/create".into(),bind_request:"/org/freedesktop/portal/desktop/request/1_20/bind".into(),shortcuts:vec![],previous:crate::portal::review::Before::idle()}}}else{Destination::Plugin{instance:instance.repeat(32),environment,owner,previous:vec![]}};
 NativeCapture{destination,current:Box::new(||true)}
}
fn completed_run()->NativeRun{NativeRun{outcome:observation(true,true),finish:Some(Box::new(|source,lifetime|{assert!(source.unwrap().current()&&lifetime());Publication::live(Arc::new(||true),||panic!("successful fixture publication aborted"))}))}}
fn recovery_decision(p:&Preview)->Decision{let mut d=decision(p);d.preview_schema_version=Some(3);d.acknowledge_retained_uncertainty=true;d}
#[test]
fn shortcut_current_scope_attempt_preserves_exact_original_facts_for_both_transports_and_after_restart(){
 for portal in[false,true]{for old_verified in[false,true]{
  let f=Fixture::new();let p=f.controller.prepare_with(&f.frame,&f.files.current,&f.files.path,fixture_context,|_|Ok(recovery_native(portal,"a"))).unwrap();
  f.controller.apply_with(&f.frame,&f.files.current,decision(&p),fixture_context,Arc::new(||true),|_,_,_|if old_verified{completed_run()}else{NativeRun{outcome:observation(false,true),finish:Some(Box::new(|_,_|Publication::refused(None)))}}).unwrap();
  let(old,intent,_)=components::current_artifacts(&f.files.current.lock().unwrap(),&f.files.path,"shortcuts").unwrap().unwrap();let old_intent_path=f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",old.intent.id));let old_bytes=std::fs::read(&old_intent_path).unwrap();
  assert!(f.controller.prepare_with(&f.frame,&f.files.current,&f.files.path,fixture_context,|_|Ok(recovery_native(portal,"b"))).is_err());
  let next=f.controller.prepare_mode_with(&f.frame,&f.files.current,&f.files.path,fixture_context,true,|_|Ok(recovery_native(portal,"b"))).unwrap();assert_eq!(next.schema_version,3);assert_ne!(next.token,old.id);let shown=serde_json::to_value(&next).unwrap();assert_eq!(shown["recovery"]["priorEffects"][0]["slot"],serde_json::to_value(&old).unwrap());let original=&shown["recovery"]["priorEffects"][0]["original"];for key in["kind","instance","owner","environment"]{assert_eq!(original[key],intent.evidence["review"]["destination"][key]);}
  assert_eq!(shown["recovery"]["portalApplicationBindingsMayBeShared"],portal);assert_eq!(shown["recovery"]["priorScopeClosureClaimed"],false);
  let before=std::fs::read(f.files.root.join(settings::durable::FILE)).unwrap();
  for version in[None,Some(1),Some(2),Some(4)]{let mut d=recovery_decision(&next);d.preview_schema_version=version;assert!(f.controller.apply_with(&f.frame,&f.files.current,d,fixture_context,Arc::new(||true),|_,_,_|panic!("stale control dispatched")).is_err());}
  let mut d=recovery_decision(&next);d.acknowledge_retained_uncertainty=false;assert!(f.controller.apply_with(&f.frame,&f.files.current,d,fixture_context,Arc::new(||true),|_,_,_|panic!("missing fourth acknowledgement dispatched")).is_err());assert_eq!(std::fs::read(f.files.root.join(settings::durable::FILE)).unwrap(),before);
  let receipt=f.controller.apply_with(&f.frame,&f.files.current,recovery_decision(&next),fixture_context,Arc::new(||true),|_,guard,_|{assert!(guard());let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(f.files.root.join(settings::durable::FILE)).unwrap()).unwrap();assert_eq!(raw["version"],8);assert_eq!(raw["components"]["shortcuts"]["phase"],"dispatched");completed_run()}).unwrap();assert_eq!(receipt.publication,"published");assert!(receipt.retained_scope_uncertainty);assert!(receipt.message.contains("incerteza"));assert_eq!(std::fs::read(&old_intent_path).unwrap(),old_bytes);
  let(_,new_intent,_)=components::current_artifacts(&f.files.current.lock().unwrap(),&f.files.path,"shortcuts").unwrap().unwrap();assert_eq!(new_intent.previous.as_ref(),Some(&old));assert_eq!(new_intent.evidence["review"],shown);assert_eq!(new_intent.evidence["acknowledgeRetainedUncertainty"],true);
  assert!(f.controller.apply_with(&f.frame,&f.files.current,recovery_decision(&next),fixture_context,Arc::new(||true),|_,_,_|panic!("replayed decision")).is_err());
  *f.files.current.lock().unwrap()=settings::load_from(&f.files.path);assert_eq!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,"shortcuts").unwrap(),vec![old]);assert!(f.prepare().is_err());assert!(settings::io_status(&f.files.current.lock().unwrap()).suspended_operations.iter().any(|s|s=="shortcuts"));
  let again=f.controller.prepare_mode_with(&f.frame,&f.files.current,&f.files.path,fixture_context,true,|_|Ok(recovery_native(portal,"c"))).unwrap();assert_eq!(serde_json::to_value(again).unwrap()["recovery"]["priorEffects"].as_array().unwrap().len(),2);
 }}
}
#[test]
fn shortcut_current_scope_retained_sources_are_checked_during_io_and_not_renewed_after_cancellation(){
 for cancel in[false,true]{let f=Fixture::new();let old=f.prepare().unwrap();f.controller.apply_with(&f.frame,&f.files.current,decision(&old),fixture_context,Arc::new(||true),|_,_,_|completed_run()).unwrap();
 let original=components::current_artifacts(&f.files.current.lock().unwrap(),&f.files.path,"shortcuts").unwrap().unwrap().0;let path=f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",original.intent.id));let bytes=std::fs::read(&path).unwrap();
 let p=f.controller.prepare_mode_with(&f.frame,&f.files.current,&f.files.path,fixture_context,true,|_|Ok(recovery_native(false,"b"))).unwrap();
 let receipt=f.controller.apply_with(&f.frame,&f.files.current,recovery_decision(&p),fixture_context,Arc::new(||true),|_,guard,_|{assert!(guard());if cancel{f.controller.cancel(&f.frame,Fixture::key(&p)).unwrap();}else{let mut changed=bytes.clone();changed.push(b' ');std::fs::write(&path,changed).unwrap();}assert!(!guard());NativeRun{outcome:observation(true,true),finish:Some(Box::new(move|source,lifetime|{assert!(source.is_none());if cancel{assert!(!lifetime());}Publication::refused(None)}))}}).unwrap();assert_ne!(receipt.publication,"published");assert!(receipt.retained_scope_uncertainty);
 if !cancel{std::fs::write(path,bytes).unwrap();}*f.files.current.lock().unwrap()=settings::load_from(&f.files.path);assert!(f.prepare().is_err());
 }
}
