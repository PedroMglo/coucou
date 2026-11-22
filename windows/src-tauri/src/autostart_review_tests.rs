use super::*;
use crate::settings::EffectFixture;
use std::{fs,os::unix::fs::PermissionsExt,sync::{atomic::AtomicUsize,mpsc},thread};

// Explicit synthetic admission for isolated Rust controller tests. Native IPC
// uses native_context exclusively and cannot enter this fixture callback.
fn fixture_context(s:&Settings,p:&Path)->io::Result<Context>{let mut c=native_context(s,p)?;assert!(!crate::pet_data::bootstrap::admitted());c.profile=Some(hash(p.parent().unwrap().as_os_str().as_encoded_bytes()));Ok(c)}
struct Fixture{files:EffectFixture,config:PathBuf,launcher:PathBuf,frame:Frame,controller:Arc<Controller>}
impl Fixture{
 fn new()->Self{let files=EffectFixture::new();files.save();let config=files.root.join("native-config");fs::create_dir(&config).unwrap();let launcher=files.root.join("launcher");fs::write(&launcher,b"fixture executable; never launched").unwrap();fs::set_permissions(&launcher,fs::Permissions::from_mode(0o700)).unwrap();let owner=settings::review::Controller::default();owner.register();let frame=owner.frame().unwrap();Self{files,config,launcher,frame,controller:Arc::new(Controller::default())}}
 fn plan(&self,enabled:bool)->Plan{Plan::capture(&self.config,&self.launcher,enabled).unwrap()}
 fn preview(&self,enabled:bool,action:Action)->Preview{self.controller.prepare_review(&self.frame,&self.files.current,&self.files.path,self.plan(enabled),action,fixture_context).unwrap()}
 fn decision(p:&Preview)->Decision{Decision{token:p.token.clone(),fingerprint:p.fingerprint.clone(),reviewed_plan:true,understand_effect:true,keep_other_components_suspended:true,acknowledge_external_uncertainty:false,acknowledge_legacy_uncertainty:false}}
 fn apply(&self,p:&Preview)->Result<Receipt,String>{self.controller.apply_review(&self.frame,&self.files.current,Self::decision(p),fixture_context,||Ok(()),|_|{})}
 fn slot(&self)->Slot{fixture_context(&self.files.current.lock().unwrap(),&self.files.path).unwrap().slot.unwrap()}
 fn entry(&self)->PathBuf{self.config.join("autostart/Coucou.desktop")}
}

#[test]fn autostart_review_prepare_is_passive_and_approval_executes_exactly_once(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);assert!(!f.entry().exists());assert!(!f.config.join(".coucou-autostart.lock").exists());assert!(f.files.probe.status().suspended_operations.is_empty());
 let receipt=f.apply(&p).unwrap();assert_eq!(receipt.registration,Registration::Verified);assert!(f.files.current.lock().unwrap().autostart);assert!(f.entry().is_file());assert!(!receipt.login_verified);assert!(!receipt.reactivated,"no admitted profile in fixture");assert_eq!(f.slot().phase,Phase::Verified);
 assert!(f.apply(&p).is_err());let p=f.preview(false,Action::SetRegistration);assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);assert!(!f.entry().exists());assert!(!f.files.current.lock().unwrap().autostart);
}
#[test]fn autostart_review_unknown_arguments_replay_and_incomplete_acknowledgements_are_refused(){
 for value in [json!({"token":"a","fingerprint":"b","reviewedPlan":true,"understandEffect":true,"keepOtherComponentsSuspended":true,"enabled":true}),json!({"token":"a","fingerprint":"b","reviewedPlan":true,"understandEffect":true,"keepOtherComponentsSuspended":true,"target":"/other"})]{assert!(serde_json::from_value::<Decision>(value).is_err());}
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let mut d=Fixture::decision(&p);d.reviewed_plan=false;assert!(f.controller.apply_review(&f.frame,&f.files.current,d,fixture_context,||Ok(()),|_|{}).is_err());assert!(!f.entry().exists());
 let mut d=Fixture::decision(&p);d.fingerprint="0".repeat(64);assert!(f.controller.apply_review(&f.frame,&f.files.current,d,fixture_context,||Ok(()),|_|{}).is_err());assert!(!f.entry().exists());
 assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);
}
#[test]fn autostart_review_stale_preferences_wrong_frame_and_swapped_launcher_cannot_be_applied(){
 for mode in 0..3{let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);
  match mode{0=>{f.files.save();},1=>f.frame.invalidate_for_test(),_=>fs::write(&f.launcher,b"changed").unwrap()}
  assert!(f.apply(&p).is_err());assert!(!f.entry().exists());assert!(!f.files.current.lock().unwrap().autostart);
 }
}
#[test]fn autostart_review_cancel_pending_or_expired_decision_has_no_effect(){
 for expired in [false,true]{let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);if expired{f.controller.review.lock().unwrap().slot.as_mut().unwrap().pending.as_mut().unwrap().expires=Instant::now()-Duration::from_secs(1);}else{f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();}assert!(f.apply(&p).is_err());assert!(!f.entry().exists());}
}
#[test]fn autostart_review_cancel_during_admission_wait_is_priority_and_gate_lasts_until_worker_returns(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let controller=f.controller.clone();let current=f.files.current.clone();let frame=f.frame.clone();let d=Fixture::decision(&p);let(tx,rx)=mpsc::channel();let(go,wait)=mpsc::channel();
 let job=thread::spawn(move||{let first=AtomicBool::new(true);controller.apply_review(&frame,&current,d,fixture_context,||{if first.swap(false,Ordering::AcqRel){tx.send(()).unwrap();wait.recv().unwrap();}Ok(())},|_|{})});
 rx.recv_timeout(Duration::from_secs(5)).unwrap();let before=Instant::now();f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();assert!(before.elapsed()<Duration::from_secs(1));assert!(f.controller.gate.try_lock().is_err());go.send(()).unwrap();assert!(job.join().unwrap().is_err());assert!(f.controller.gate.try_lock().is_ok());assert!(!f.entry().exists());
}
#[test]fn autostart_review_cancel_after_preference_save_keeps_desired_flag_but_never_registers(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let controller=f.controller.clone();let current=f.files.current.clone();let frame=f.frame.clone();let d=Fixture::decision(&p);let(tx,rx)=mpsc::channel();let(go,wait)=mpsc::channel();
 let job=thread::spawn(move||controller.apply_review(&frame,&current,d,fixture_context,||Ok(()),|_|{tx.send(()).unwrap();wait.recv().unwrap();}));
 rx.recv_timeout(Duration::from_secs(5)).unwrap();assert!(f.files.current.lock().unwrap().autostart);f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();assert!(f.controller.gate.try_lock().is_err());go.send(()).unwrap();let receipt=job.join().unwrap().unwrap();assert_eq!(receipt.registration,Registration::NotDispatched);assert!(!f.entry().exists());assert!(!f.config.join(".coucou-autostart.lock").exists());assert_eq!(f.slot().phase,Phase::NotDispatched);
}
#[test]fn autostart_review_unconfirmed_dispatch_requires_observation_and_fresh_approval(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let changed=AtomicBool::new(false);let receipt=f.controller.apply_review(&f.frame,&f.files.current,Fixture::decision(&p),fixture_context,||{
  let value:Value=serde_json::from_slice(&fs::read(f.files.root.join(settings::durable::FILE))?).map_err(io::Error::other)?;
  if value["components"]["autostart"]["phase"]=="dispatched"&&!changed.swap(true,Ordering::AcqRel){fs::write(&f.launcher,b"new launcher bytes")?;}Ok(())
 },|_|{}).unwrap();assert_eq!(receipt.registration,Registration::Unconfirmed);assert_eq!(f.slot().phase,Phase::Unconfirmed);assert!(!f.entry().exists());
 assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(true),Action::SetRegistration,fixture_context).is_err());
 let p=f.preview(true,Action::ReconcileObservation);assert_eq!(f.apply(&p).unwrap().registration,Registration::ReconciledObservation);assert_eq!(f.slot().phase,Phase::ReconciledObservation);assert!(f.files.probe.operation(COMPONENT).is_err());assert!(!f.entry().exists());
 let p=f.preview(true,Action::SetRegistration);assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);assert!(f.entry().exists());
}
#[test]fn autostart_review_failed_preference_publication_records_not_dispatched_and_preserves_original(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let temp=f.files.path.with_extension(format!("json.coucou-{}",std::process::id()));fs::write(&temp,b"foreign file").unwrap();let before=fs::read(&f.files.path).unwrap();let receipt=f.apply(&p).unwrap();assert_eq!(receipt.registration,Registration::NotDispatched);assert_eq!(fs::read(&f.files.path).unwrap(),before);assert_eq!(fs::read(temp).unwrap(),b"foreign file");assert!(!f.entry().exists());assert_eq!(f.slot().phase,Phase::NotDispatched);
}
#[test]fn autostart_review_resource_fence_excludes_another_writer_and_detects_replacement(){
 let f=Fixture::new();let plan=f.plan(true);let fence=plan.fence(||Ok(())).unwrap();assert!(fence.current());assert!(f.plan(true).fence(||Ok(())).is_err());let lock=f.config.join(".coucou-autostart.lock");fs::rename(&lock,f.config.join("previous-lock")).unwrap();fs::write(&lock,b"").unwrap();fs::set_permissions(&lock,fs::Permissions::from_mode(0o600)).unwrap();assert!(!fence.current());drop(fence);assert!(f.plan(true).fence(||Ok(())).is_ok());
}
#[test]fn autostart_review_passive_settings_reader_does_not_refuse_or_deadlock_the_file_executor(){
 let f=Fixture::new();let p=f.preview(true,Action::SetRegistration);let once=AtomicBool::new(false);let reader=Mutex::new(None);let current=f.files.current.clone();let entry=f.entry();
 let receipt=f.controller.apply_review(&f.frame,&f.files.current,Fixture::decision(&p),fixture_context,||{
  let value:Value=serde_json::from_slice(&fs::read(f.files.root.join(settings::durable::FILE))?).map_err(io::Error::other)?;
  if value["components"]["autostart"]["phase"]=="dispatched"&&!once.swap(true,Ordering::AcqRel){
   let current=current.clone();let entry=entry.clone();let(tx,rx)=mpsc::channel();
   let job=thread::spawn(move||{let _read=current.lock().unwrap();tx.send(()).unwrap();let until=Instant::now()+Duration::from_secs(3);while !entry.exists()&&Instant::now()<until{thread::sleep(Duration::from_millis(2));}assert!(entry.exists(),"OS writer waited on a passive Shared reader");});
   rx.recv_timeout(Duration::from_secs(2)).unwrap();*reader.lock().unwrap()=Some(job);
  }Ok(())
 },|_|{}).unwrap();if let Some(job)=reader.lock().unwrap().take(){job.join().unwrap();}assert_eq!(receipt.registration,Registration::Verified);assert!(f.entry().exists());
}

fn seed_unknown(f:&Fixture,profile:String,phase:Phase)->Slot{
 let plan=f.plan(false);seed_unknown_view(f,plan.view,profile,phase)
}
fn seed_unknown_view(f:&Fixture,view:View,profile:String,phase:Phase)->Slot{
 let id=random().unwrap();let binding=Binding{profile,configuration:components::autostart_configuration(f.files.current.lock().unwrap().autostart),resource:view.resource.clone(),plan:view.fingerprint.clone()};
 let artifact=Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"nativePlan":view,"fixtureOnly":true}),previous:fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().slot};
 let reference=components::retain(&f.files.path,&artifact,||Ok(())).unwrap();let mut slot=Slot{id,component:COMPONENT.into(),phase:Phase::Prepared,binding,owner:process().unwrap(),intent:reference,outcome:None};
 let s=f.files.current.lock().unwrap();components::prepare(&s,&f.files.path,slot.clone(),||Ok(())).unwrap();
 if phase!=Phase::Prepared{components::transition(&s,&f.files.path,COMPONENT,&slot.id,Phase::Dispatched,None,||Ok(())).unwrap();slot.phase=Phase::Dispatched;}
 if phase==Phase::Unconfirmed{let artifact=Artifact{version:1,generation:1,id:random().unwrap(),effect:slot.id.clone(),component:COMPONENT.into(),binding:slot.binding.clone(),phase,evidence:json!({"fixtureUnknown":true}),previous:None};let reference=components::retain(&f.files.path,&artifact,||Ok(())).unwrap();components::transition(&s,&f.files.path,COMPONENT,&slot.id,phase,Some(reference.clone()),||Ok(())).unwrap();slot.phase=phase;slot.outcome=Some(reference);}
 slot
}
#[test]fn autostart_review_independent_registration_requires_both_new_scope_and_new_resource_and_own_ack(){
 for phase in [Phase::Prepared,Phase::Dispatched,Phase::Unconfirmed]{for different_scope in [false,true]{for different_resource in [false,true]{
  let f=Fixture::new();fs::create_dir(f.config.join("autostart")).unwrap();fs::write(f.entry(),b"old uncertain resource; do not touch").unwrap();fs::set_permissions(f.entry(),fs::Permissions::from_mode(0o600)).unwrap();
  let profile=fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().profile.unwrap();
  let old=seed_unknown(&f,if different_scope{"0".repeat(64)}else{profile.clone()},phase);let old_bytes=fs::read(f.entry()).unwrap();let old_artifacts:Vec<_>=fs::read_dir(f.files.root.join(components::DIRECTORY)).unwrap().map(|p|{let p=p.unwrap().path();let b=fs::read(&p).unwrap();(p,b)}).collect();
  let config=if different_resource{let p=f.files.root.join("new-native-config");fs::create_dir(&p).unwrap();p}else{f.config.clone()};let entry=config.join("autostart/Coucou.desktop");
  let p=f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,Plan::capture(&config,&f.launcher,true).unwrap(),Action::SetIndependentRegistration,fixture_context);
  if different_scope&&different_resource{
   let p=p.unwrap();assert_eq!(p.origin.slot,Some(old.clone()));assert!(p.impact.contains("permanece incerta"));
   assert!(f.apply(&p).is_err(),"ordinary three acknowledgements cannot authorize independent work");assert_eq!(f.slot(),old);assert!(!entry.exists());assert!(!f.files.current.lock().unwrap().autostart);
   let mut decision=Fixture::decision(&p);decision.acknowledge_external_uncertainty=true;
   assert_eq!(f.controller.apply_review(&f.frame,&f.files.current,decision,fixture_context,||Ok(()),|_|{}).unwrap().registration,Registration::Verified);
   let new=f.slot();assert_eq!(new.phase,Phase::Verified);assert_eq!(new.binding.profile,profile);assert_ne!(new.binding.resource,old.binding.resource);
   let artifact=components::intent_artifact(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap().unwrap();assert_eq!(artifact.previous,Some(old.clone()));assert_eq!(artifact.evidence["previousEffectRemainsUnresolved"],true);assert_eq!(artifact.evidence["externalUncertaintyAcknowledged"],true);
   assert!(entry.is_file());assert!(f.apply(&p).is_err());
  }else{assert!(p.is_err());assert_eq!(f.slot(),old);}
  assert_eq!(fs::read(f.entry()).unwrap(),old_bytes);for(p,b)in old_artifacts{assert_eq!(fs::read(p).unwrap(),b);}
 }}}
}
#[test]fn autostart_review_independent_action_cannot_replace_a_known_outcome_or_missing_slot(){
 let f=Fixture::new();let try_independent=||f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(true),Action::SetIndependentRegistration,fixture_context);
 assert!(try_independent().is_err());let p=f.preview(false,Action::SetRegistration);assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);assert!(try_independent().is_err());
}
#[test]fn autostart_review_archived_unknown_survives_later_work_and_blocks_return_to_its_resource(){
 let f=Fixture::new();let old=seed_unknown(&f,"0".repeat(64),Phase::Dispatched);let config=f.files.root.join("independent-config");fs::create_dir(&config).unwrap();
 let p=f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,Plan::capture(&config,&f.launcher,true).unwrap(),Action::SetIndependentRegistration,fixture_context).unwrap();let mut d=Fixture::decision(&p);d.acknowledge_external_uncertainty=true;
 let receipt=f.controller.apply_review(&f.frame,&f.files.current,d,fixture_context,||Ok(()),|_|{}).unwrap();assert!(receipt.message.contains("continua incerta"));
 assert_eq!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap(),vec![old.clone()]);
 let p=f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,Plan::capture(&config,&f.launcher,false).unwrap(),Action::SetRegistration,fixture_context).unwrap();assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);
 assert_eq!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap(),vec![old.clone()]);
 assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(true),Action::SetRegistration,fixture_context).err().unwrap().contains("arquivada"));assert!(!f.entry().exists());
 // A corrupt archived reference cannot be silently omitted from the review.
 let path=f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",old.intent.id));fs::write(path,b"untrusted substituted history").unwrap();
 assert!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).is_err());assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,Plan::capture(&config,&f.launcher,true).unwrap(),Action::SetRegistration,fixture_context).is_err());assert!(!config.join("autostart/Coucou.desktop").exists());
}

fn independent_history()->(Fixture,Slot){
 let f=Fixture::new();let old=seed_unknown(&f,"0".repeat(64),Phase::Dispatched);let config=f.files.root.join("independent-config");fs::create_dir(&config).unwrap();
 let p=f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,Plan::capture(&config,&f.launcher,true).unwrap(),Action::SetIndependentRegistration,fixture_context).unwrap();let mut d=Fixture::decision(&p);d.acknowledge_external_uncertainty=true;
 assert_eq!(f.controller.apply_review(&f.frame,&f.files.current,d,fixture_context,||Ok(()),|_|{}).unwrap().registration,Registration::Verified);(f,old)
}
fn archived_preview(f:&Fixture,subject:&Slot)->Result<Preview,String>{
 f.controller.prepare_review_selected(&f.frame,&f.files.current,&f.files.path,f.plan(false),Action::ReconcileArchivedObservation,fixture_context,Some(subject.clone()))
}
#[test]fn autostart_review_historical_observation_preserves_raw_unknown_and_requires_fresh_registration(){
 let(f,old)=independent_history();let before_settings=fs::read(&f.files.path).unwrap();let old_intent_path=f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",old.intent.id));let old_intent=fs::read(&old_intent_path).unwrap();
 let p=archived_preview(&f,&old).unwrap();assert_eq!(p.archived_effect,Some(old.clone()));assert!(!f.entry().exists());let r=f.apply(&p).unwrap();assert_eq!(r.registration,Registration::ReconciledObservation);assert!(!r.reactivated);assert!(r.message.contains("resultado antigo continua desconhecido"));assert!(!f.entry().exists());assert_eq!(fs::read(&f.files.path).unwrap(),before_settings);assert!(f.files.current.lock().unwrap().autostart);
 let h=components::history_view(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap();assert!(h.unresolved.is_empty());assert_eq!(h.observed.len(),1);assert_eq!(h.observed[0].observation.subject,old);assert!(h.observed[0].current_for_profile);assert!(!h.observed[0].observation.past_effect_outcome_known);assert_eq!(fs::read(&old_intent_path).unwrap(),old_intent);assert!(f.apply(&p).is_err());
 assert!(f.files.probe.operation(COMPONENT).is_err());let p=f.preview(true,Action::SetRegistration);assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);assert!(f.entry().is_file());assert_eq!(fs::read(&old_intent_path).unwrap(),old_intent);
 let reload=settings::load_from(&f.files.path);*f.files.current.lock().unwrap()=reload;let h=components::history_view(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap();assert!(h.unresolved.is_empty());assert_eq!(h.observed[0].observation.subject.phase,Phase::Dispatched);
}
#[test]fn autostart_review_historical_selector_other_resource_late_change_and_cancel_cannot_publish_observation(){
 for mode in 0..5{
  let(f,old)=independent_history();let before=fs::read(f.files.root.join(settings::durable::FILE)).unwrap();
  if mode==0{let mut forged=old.clone();forged.id="f".repeat(32);assert!(archived_preview(&f,&forged).is_err());continue;}
  if mode==1{let other=Plan::capture(&f.files.root.join("independent-config"),&f.launcher,false).unwrap();assert!(f.controller.prepare_review_selected(&f.frame,&f.files.current,&f.files.path,other,Action::ReconcileArchivedObservation,fixture_context,Some(old)).is_err());continue;}
  let p=archived_preview(&f,&old).unwrap();match mode{
   2=>{f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();},
   3=>{fs::create_dir(f.config.join("autostart")).unwrap();fs::write(f.entry(),b"external replacement; never overwrite").unwrap();fs::set_permissions(f.entry(),fs::Permissions::from_mode(0o600)).unwrap();},
   _=>{let plan=f.plan(false);let _held=plan.fence(||Ok(())).unwrap();assert!(f.apply(&p).is_err());assert_eq!(fs::read(f.files.root.join(settings::durable::FILE)).unwrap(),before);continue;}
  }assert!(f.apply(&p).is_err());assert_eq!(fs::read(f.files.root.join(settings::durable::FILE)).unwrap(),before);
 }
}
#[test]fn autostart_review_historical_cancel_during_admission_is_priority_and_does_not_record_false_evidence(){
 let(f,old)=independent_history();let p=archived_preview(&f,&old).unwrap();let before=fs::read(f.files.root.join(settings::durable::FILE)).unwrap();let controller=f.controller.clone();let current=f.files.current.clone();let frame=f.frame.clone();let d=Fixture::decision(&p);let(tx,rx)=mpsc::channel();let(go,wait)=mpsc::channel();
 let job=thread::spawn(move||{let first=AtomicBool::new(true);controller.apply_review(&frame,&current,d,fixture_context,||{if first.swap(false,Ordering::AcqRel){tx.send(()).unwrap();wait.recv().unwrap();}Ok(())},|_|{})});rx.recv_timeout(Duration::from_secs(5)).unwrap();
 let start=Instant::now();f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();assert!(start.elapsed()<Duration::from_secs(1));assert!(f.controller.gate.try_lock().is_err());go.send(()).unwrap();assert!(job.join().unwrap().is_err());assert_eq!(fs::read(f.files.root.join(settings::durable::FILE)).unwrap(),before);assert!(!f.entry().exists());
}
#[test]fn autostart_review_new_data_review_invalidates_only_eligibility_of_historical_observation(){
 let(f,old)=independent_history();let p=archived_preview(&f,&old).unwrap();f.apply(&p).unwrap();
 f.files.review_data();
 let h=components::history_view(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap();assert_eq!(h.unresolved,vec![old.clone()]);assert_eq!(h.observed.len(),1);assert!(!h.observed[0].current_for_profile);
 assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(true),Action::SetRegistration,fixture_context).is_err());let p=archived_preview(&f,&old).unwrap();f.apply(&p).unwrap();assert!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap().is_empty());
}
#[test]fn autostart_review_historical_journal_refuses_forged_present_observation_or_old_subject(){
 for mode in 0..5{
  let(f,old)=independent_history();let p=archived_preview(&f,&old).unwrap();let id=random().unwrap();let binding=Binding{profile:p.profile_scope.clone(),configuration:components::autostart_configuration(p.previous_desired_enabled),resource:p.plan.resource.clone(),plan:p.plan.fingerprint.clone()};let previous=f.slot();
  let mut claim=components::HistoricalObservation{schema_version:1,legacy_origin_unverifiable:false,legacy_uncertainty_acknowledged:false,subject:old.clone(),profile:p.profile_scope.clone(),data_review:p.origin.data_review.clone(),native_plan:p.plan.clone(),registration_matches_expected:true,no_registration_effect:true,no_reactivation:true,login_verified:false,past_effect_outcome_known:false};
  match mode{0=>claim.past_effect_outcome_known=true,1=>claim.no_reactivation=false,2=>claim.profile="f".repeat(64),3=>claim.native_plan.diff.push_str("changed after consent"),_=>claim.subject.id="f".repeat(32)}
  let intent=Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"nativePlan":p.plan,"historicalSubject":old,"review":p}),previous:Some(previous.clone())};let intent_ref=components::retain(&f.files.path,&intent,||Ok(())).unwrap();
  let outcome=Artifact{version:1,generation:1,id:random().unwrap(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::ReconciledObservation,evidence:json!({"archivedObservation":claim}),previous:None};let outcome_ref=components::retain(&f.files.path,&outcome,||Ok(())).unwrap();let slot=Slot{id,component:COMPONENT.into(),phase:Phase::ReconciledObservation,binding,owner:process().unwrap(),intent:intent_ref,outcome:Some(outcome_ref)};
  assert!(components::record_historical_observation(&f.files.current.lock().unwrap(),&f.files.path,slot,||Ok(())).is_err());assert_eq!(f.slot(),previous);assert!(!f.entry().exists());
 }
}
fn other_fixture_scope(s:&Settings,p:&Path)->io::Result<Context>{let mut c=fixture_context(s,p)?;c.profile=Some("f".repeat(64));Ok(c)}
#[test]fn autostart_review_historical_observation_cannot_bypass_the_current_unknown_same_scope(){
 let(f,old)=independent_history();let journal=f.files.root.join(settings::durable::FILE);let mut record:Value=serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();record["components"]["autostart"]["phase"]=json!("dispatched");record["components"]["autostart"].as_object_mut().unwrap().remove("outcome");record["suspendedOperations"]=json!(["autostart"]);fs::write(&journal,serde_json::to_vec(&record).unwrap()).unwrap();*f.files.current.lock().unwrap()=settings::load_from(&f.files.path);
 let before=fs::read(&journal).unwrap();assert!(archived_preview(&f,&old).err().unwrap().contains("operação atual"));assert_eq!(fs::read(&journal).unwrap(),before);
 // Both native profile scope and resource are independent in this explicit
 // synthetic caller. The original current unknown must remain visible.
 let p=f.controller.prepare_review_selected(&f.frame,&f.files.current,&f.files.path,f.plan(false),Action::ReconcileArchivedObservation,other_fixture_scope,Some(old)).unwrap();let d=Fixture::decision(&p);
 f.controller.apply_review(&f.frame,&f.files.current,d,other_fixture_scope,||Ok(()),|_|{}).unwrap();let h=components::history_view(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap();assert_eq!(h.unresolved.len(),1);assert_eq!(h.unresolved[0].binding.resource,record["components"]["autostart"]["binding"]["resource"].as_str().unwrap());assert_eq!(h.unresolved[0].phase,Phase::Dispatched);
}
#[test]fn autostart_review_current_unknown_is_observed_before_its_unknown_ancestor_on_the_same_resource(){
 let(f,old)=independent_history();let previous=f.slot();let plan=f.plan(true);let id=random().unwrap();let profile=fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().profile.unwrap();let binding=Binding{profile,configuration:components::autostart_configuration(true),resource:plan.view.resource.clone(),plan:plan.view.fingerprint.clone()};
 let artifact=Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"nativePlan":plan.view,"legacyFixtureOnly":true}),previous:Some(previous)};let reference=components::retain(&f.files.path,&artifact,||Ok(())).unwrap();let slot=Slot{id,component:COMPONENT.into(),phase:Phase::Prepared,binding,owner:process().unwrap(),intent:reference,outcome:None};
 let journal=f.files.root.join(settings::durable::FILE);let mut record:Value=serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();record["components"]["autostart"]=serde_json::to_value(slot).unwrap();record["suspendedOperations"]=json!(["autostart"]);fs::write(&journal,serde_json::to_vec(&record).unwrap()).unwrap();*f.files.current.lock().unwrap()=settings::load_from(&f.files.path);
 assert!(archived_preview(&f,&old).is_err());let p=f.preview(true,Action::ReconcileObservation);assert_eq!(f.apply(&p).unwrap().registration,Registration::ReconciledObservation);assert_eq!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap(),vec![old.clone()]);
 let p=archived_preview(&f,&old).unwrap();f.apply(&p).unwrap();assert!(components::unresolved_history(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT).unwrap().is_empty());assert!(!f.entry().exists());
}

// Legacy nativePlan1 bytes are constructed with the previous, path-only
// algorithm. The current Plan is always captured by the actual executor.
fn legacy_unknown(f:&Fixture)->Slot{
 let mut view=f.plan(false).view;view.schema_version=1;let mut old=fs::canonicalize(&f.config).unwrap().as_os_str().as_encoded_bytes().to_vec();old.extend_from_slice(b"\0autostart\0Coucou.desktop");view.resource=hash(&old);view.fingerprint.clear();view.fingerprint=hash(&serde_json::to_vec(&view).unwrap());
 let profile=fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().profile.unwrap();seed_unknown_view(f,view,profile,Phase::Dispatched)
}
fn legacy_preview(f:&Fixture,old:&Slot)->Result<Preview,String>{f.controller.prepare_review_selected(&f.frame,&f.files.current,&f.files.path,f.plan(false),Action::ReconcileLegacyObservation,fixture_context,Some(old.clone()))}
fn apply_legacy(f:&Fixture,p:&Preview)->Result<Receipt,String>{let mut d=Fixture::decision(p);d.acknowledge_legacy_uncertainty=true;f.controller.apply_review(&f.frame,&f.files.current,d,fixture_context,||Ok(()),|_|{})}
#[test]fn autostart_review_legacy_origin_requires_own_ack_preserves_raw_unknown_and_only_observes_present(){
 let f=Fixture::new();let old=legacy_unknown(&f);let prefs=fs::read(&f.files.path).unwrap();let original_intent=fs::read(f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",old.intent.id))).unwrap();
 for action in [Action::SetRegistration,Action::SetIndependentRegistration,Action::ReconcileObservation]{assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(false),action,fixture_context).is_err());}
 let p=legacy_preview(&f,&old).unwrap();assert!(p.impact.contains("não pode ser comprovada"));assert_eq!(p.archived_effect,Some(old.clone()));assert_eq!(p.plan.schema_version,2);assert_ne!(p.plan.resource,old.binding.resource);assert!(f.apply(&p).is_err());assert_eq!(f.slot(),old);assert!(!f.entry().exists());
 let receipt=apply_legacy(&f,&p).unwrap();assert_eq!(receipt.registration,Registration::ReconciledObservation);assert!(!receipt.reactivated);assert_eq!(fs::read(&f.files.path).unwrap(),prefs);assert!(!f.entry().exists());assert_eq!(fs::read(f.files.root.join(components::DIRECTORY).join(format!("component-{}.json",old.intent.id))).unwrap(),original_intent);assert!(apply_legacy(&f,&p).is_err());
 let h=components::history_for_resource(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT,&f.plan(false).view.resource).unwrap();assert!(h.unresolved.is_empty());assert_eq!(h.observed[0].observation.subject,old);assert!(h.observed[0].observation.legacy_origin_unverifiable&&h.observed[0].observation.legacy_uncertainty_acknowledged&&!h.observed[0].observation.past_effect_outcome_known);assert!(h.observed[0].current_for_profile);assert!(f.files.probe.operation(COMPONENT).is_err());
 let p=f.preview(true,Action::SetRegistration);assert_eq!(f.apply(&p).unwrap().registration,Registration::Verified);assert!(f.entry().exists());
}
#[test]fn autostart_review_legacy_observation_is_invalid_after_base_replacement_or_data_review(){
 for replace_base in [false,true]{let f=Fixture::new();let old=legacy_unknown(&f);let p=legacy_preview(&f,&old).unwrap();apply_legacy(&f,&p).unwrap();let prior_resource=f.plan(false).view.resource;
  if replace_base{fs::rename(&f.config,f.files.root.join("retained-native-base")).unwrap();fs::create_dir(&f.config).unwrap();}else{f.files.review_data();}
  let plan=f.plan(false);if replace_base{assert_ne!(plan.view.resource,prior_resource);}let h=components::history_for_resource(&f.files.current.lock().unwrap(),&f.files.path,COMPONENT,&plan.view.resource).unwrap();assert_eq!(h.unresolved,vec![old.clone()]);assert!(!h.observed[0].current_for_profile);
  assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,plan,Action::SetRegistration,fixture_context).is_err());let p=legacy_preview(&f,&old).unwrap();apply_legacy(&f,&p).unwrap();assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(false),Action::SetRegistration,fixture_context).is_ok());
 }
}
#[test]fn autostart_review_new_physical_subject_cannot_be_downgraded_to_legacy_or_reconciled_after_replacement(){
 let f=Fixture::new();let profile=fixture_context(&f.files.current.lock().unwrap(),&f.files.path).unwrap().profile.unwrap();let old=seed_unknown(&f,profile,Phase::Dispatched);assert!(legacy_preview(&f,&old).is_err());fs::rename(&f.config,f.files.root.join("retained-native-base")).unwrap();fs::create_dir(&f.config).unwrap();assert!(f.controller.prepare_review(&f.frame,&f.files.current,&f.files.path,f.plan(false),Action::ReconcileObservation,fixture_context).is_err());assert_eq!(f.slot(),old);
}

#[test]fn autostart_review_historical_observation_cannot_bypass_current_legacy_unknown(){
 let(f,old)=independent_history();let mut view=f.plan(true).view;view.schema_version=1;view.resource="f".repeat(64);view.fingerprint.clear();view.fingerprint=hash(&serde_json::to_vec(&view).unwrap());let current=seed_unknown_view(&f,view,"e".repeat(64),Phase::Dispatched);assert!(archived_preview(&f,&old).is_err());assert_eq!(f.slot(),current);
}
#[test]fn autostart_review_legacy_cancel_during_admission_wait_remains_priority_and_preserves_unknown(){
 let f=Fixture::new();let old=legacy_unknown(&f);let p=legacy_preview(&f,&old).unwrap();let controller=f.controller.clone();let current=f.files.current.clone();let frame=f.frame.clone();let mut decision=Fixture::decision(&p);decision.acknowledge_legacy_uncertainty=true;let(tx,rx)=mpsc::channel();let(go,wait)=mpsc::channel();
 let job=thread::spawn(move||{let once=AtomicBool::new(true);controller.apply_review(&frame,&current,decision,fixture_context,||{if once.swap(false,Ordering::AcqRel){tx.send(()).unwrap();wait.recv().unwrap();}Ok(())},|_|{})});rx.recv_timeout(Duration::from_secs(5)).unwrap();let before=Instant::now();f.controller.cancel_review(&f.frame,Key{token:p.token.clone(),fingerprint:p.fingerprint.clone()}).unwrap();assert!(before.elapsed()<Duration::from_secs(1));assert!(f.controller.gate.try_lock().is_err());go.send(()).unwrap();assert!(job.join().unwrap().is_err());assert_eq!(f.slot(),old);assert!(!f.entry().exists());
}
