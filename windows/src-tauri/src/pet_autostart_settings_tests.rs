use super::*;
use std::cell::RefCell;

#[test]fn autostart_settings_intent_and_stale_general_snapshot_preserve_each_others_fields(){
 let current=Mutex::new(Settings::default());let initial_volume=current.lock().unwrap().sound_volume;let mut stale=current.lock().unwrap().clone();stale.sound_volume=0.043;let calls=RefCell::new(vec![]);let controller=Controller::default();
 let receipt=controller.apply_with(&current,Patch{enabled:true},||Ok(()),|s|{assert_eq!(s.sound_volume,initial_volume);calls.borrow_mut().push("write");Ok(())},|_|calls.borrow_mut().push("publish"),|v|{assert!(v);calls.borrow_mut().push("effect");Ok(())});
 // The initial volume is independent of the incoming stale snapshot.
 assert!(receipt.is_ok());let saved=crate::settings::update_with(&current,|next|{preserve_incoming(next,&mut stale);*next=stale;},|s|{assert!(s.autostart);Ok(())},|_|{}).unwrap();assert!(saved.autostart);assert_eq!(saved.sound_volume,0.043);assert_eq!(*calls.borrow(),["write","publish","effect"]);
}

#[test]fn autostart_settings_denial_or_persistence_failure_never_publishes_or_executes(){
 for denied in [true,false]{let current=Mutex::new(Settings::default());let calls=RefCell::new(vec![]);let result=Controller::default().apply_with(&current,Patch{enabled:true},||if denied{Err(io::Error::new(io::ErrorKind::PermissionDenied,"pending restoration"))}else{Ok(())},|_|{calls.borrow_mut().push("write");Err(io::Error::other("disk failure"))},|_|panic!("failed preference published"),|_|panic!("failed preference executed"));assert!(result.is_err());assert!(!current.lock().unwrap().autostart);assert_eq!(calls.borrow().len(),usize::from(!denied));}
}

#[test]fn autostart_settings_executor_error_is_a_typed_saved_receipt_and_noop_does_not_retry_it(){
 let current=Mutex::new(Settings::default());let controller=Controller::default();let receipt=controller.apply_with(&current,Patch{enabled:true},||Ok(()),|_|Ok(()),|_|{},|_|Err("synthetic registration failure".into())).unwrap();assert!(current.lock().unwrap().autostart);assert_eq!(receipt.registration,Registration::Unconfirmed);assert!(receipt.reason.unwrap().contains("preferência foi guardada"));let again=controller.apply_with(&current,Patch{enabled:true},||panic!("no-op cannot grant admission"),|_|panic!("no-op cannot write"),|_|panic!("no-op cannot publish"),|_|panic!("no-op cannot repeat executor")).unwrap();assert_eq!(again.change,Change::Unchanged);assert_eq!(again.registration,Registration::Unconfirmed);assert!(again.reason.unwrap().contains("synthetic registration failure"));
}

#[test]fn autostart_settings_busy_executor_does_not_hold_the_global_settings_lock_or_accept_another_intent(){
 use std::{sync::{Arc,mpsc},time::Duration};let current=Arc::new(Mutex::new(Settings::default()));let controller=Arc::new(Controller::default());let(entered,ready)=mpsc::channel();let(release,wait)=mpsc::channel();let c=current.clone();let gate=controller.clone();let job=std::thread::spawn(move||gate.apply_with(&c,Patch{enabled:true},||Ok(()),|_|Ok(()),|_|{},|_|{entered.send(()).unwrap();wait.recv().unwrap();Ok(())}));ready.recv_timeout(Duration::from_secs(2)).unwrap();assert!(current.try_lock().is_ok());let error=controller.apply_with(&current,Patch{enabled:false},||panic!("busy admission"),|_|panic!("busy write"),|_|panic!("busy publish"),|_|panic!("busy effect")).unwrap_err();assert_eq!(error.kind(),io::ErrorKind::WouldBlock);crate::settings::update_with(&current,|s|s.sound_volume=0.031,|s|{assert!(s.autostart);Ok(())},|_|{}).unwrap();release.send(()).unwrap();job.join().unwrap().unwrap();let s=current.lock().unwrap();assert!(s.autostart);assert_eq!(s.sound_volume,0.031);
}

#[test]fn autostart_settings_wire_requires_exact_boolean_object_and_receipt_does_not_claim_host_verification(){
 for input in ["null","true","[]","[true]","{}","{\"enabled\":null}","{\"enabled\":\"true\"}","{\"enabled\":true,\"window\":\"settings\"}","{\"enabled\":true,\"enabled\":false}"]{assert!(serde_json::from_str::<Patch>(input).is_err(),"{input}");}assert!(serde_json::from_str::<Patch>("{\"enabled\":false}").is_ok());let value=serde_json::to_value(Receipt{schema_version:1,enabled:true,change:Change::Changed,registration:Registration::ExecutorAccepted,reason:None}).unwrap();assert_eq!(value["schemaVersion"],1);assert_eq!(value["registration"],"executorAccepted");assert!(value.get("executionAuthorityGranted").is_none());assert!(value.get("effectVerified").is_none());assert!(crate::pet_center_settings::authorize_window("forged","settings").is_err());assert!(crate::pet_center_settings::authorize_window("settings","settings").is_ok());
}

#[test]fn autostart_settings_terminated_worker_is_not_reported_as_live_busy_or_retried(){
 use std::sync::Arc;let controller=Arc::new(Controller::default());let c=controller.clone();assert!(std::thread::spawn(move||{let _guard=c.gate.lock().unwrap();panic!("synthetic worker ended after unknown phase");}).join().is_err());let current=Mutex::new(Settings::default());let error=controller.apply_with(&current,Patch{enabled:true},||panic!("no new admission"),|_|panic!("no new persistence"),|_|panic!("no publication"),|_|panic!("no replay")).unwrap_err();assert_ne!(error.kind(),io::ErrorKind::WouldBlock);assert!(error.to_string().contains("terminou sem confirmação"));assert!(!current.lock().unwrap().autostart);
}
