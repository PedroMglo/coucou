use super::*;
use crate::settings::EffectFixture;
use serde_json::json;
use std::{fs,sync::atomic::{AtomicBool,Ordering}};
fn fixture_context(s:&Settings,p:&Path)->std::io::Result<Context>{let mut c=context(s,p)?;assert!(!crate::pet_data::bootstrap::admitted());c.profile=Some("a".repeat(64));Ok(c)}
fn frame()->(settings::review::Controller,Frame){let owner=settings::review::Controller::default();owner.register();let frame=owner.frame().unwrap();(owner,frame)}
fn request(before:serde_json::Value,after:serde_json::Value)->Request{serde_json::from_value(json!({"changes":{"openChat":{"before":before,"after":after}}})).unwrap()}
fn value(keys:&str)->serde_json::Value{json!({"keys":keys,"enabled":true})}
#[test]
fn shortcuts_patch_changes_only_owned_action_and_preserves_unknown_ids_and_other_preferences(){
 let f=EffectFixture::new();f.save();f.update(|s|{s.shortcuts.insert("legacyUnknown".into(),Binding{keys:"Super+P".into(),enabled:true});s.autostart=true;s.model="keep-selected-model".into();});let(_owner,frame)=frame();
 let r=apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,value("Ctrl+Alt+Space")),fixture_context,||Ok(())).unwrap();
 assert!(r.changed&&r.new_registration_requires_review);assert_eq!(r.bindings["legacyUnknown"].keys,"Super+P");let s=f.current.lock().unwrap();assert!(s.autostart);assert_eq!(s.model,"keep-selected-model");assert!(context(&s,&f.path).unwrap().slot.is_none());drop(s);
 assert!(f.probe.operation("shortcuts").is_err());assert!(f.probe.operation("voice").is_ok());let restart=settings::load_from(&f.path);assert!(settings::io_status(&restart).suspended_operations.iter().any(|n|n=="shortcuts"));
 assert!(!f.root.join(components::DIRECTORY).exists(),"no fictitious native effect artifact");
}
#[test]
fn shortcuts_patch_stale_expected_value_wrong_frame_and_malformed_scope_do_not_publish(){
 let f=EffectFixture::new();f.save();let(_owner,frame)=frame();let before=fs::read(&f.path).unwrap();
 assert!(apply_with(&frame,&f.current,&f.path,request(value("Super+Z"),value("Ctrl+Alt+Space")),fixture_context,||Ok(())).is_err());assert_eq!(fs::read(&f.path).unwrap(),before);
 frame.invalidate_for_test();assert!(apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,value("Ctrl+Alt+Space")),fixture_context,||Ok(())).is_err());assert_eq!(fs::read(&f.path).unwrap(),before);
 for v in [json!({"changes":{},"caller":"providerModel"}),json!({"changes":{"openChat":{"before":null,"after":{"keys":"Super+Z","enabled":true,"target":"other"}}}})]{assert!(serde_json::from_value::<Request>(v).is_err());}
}
#[test]
fn shortcuts_patch_pause_is_durable_before_preference_and_failure_never_dispatches(){
 let f=EffectFixture::new();f.save();let(_owner,frame)=frame();let before=fs::read(&f.path).unwrap();let observed=AtomicBool::new(false);
 let result=apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,value("Ctrl+Alt+Space")),fixture_context,||{
  let paused=fs::read(f.root.join(settings::durable::FILE)).ok().and_then(|b|serde_json::from_slice::<serde_json::Value>(&b).ok()).is_some_and(|v|v["suspendedOperations"].as_array().is_some_and(|a|a.iter().any(|v|v=="shortcuts")));
  if paused{observed.store(true,Ordering::Release);Err("cancelled after durable pause".into())}else{Ok(())}
 });
 assert!(result.is_err()&&observed.load(Ordering::Acquire));assert_eq!(fs::read(&f.path).unwrap(),before);assert!(f.current.lock().unwrap().shortcuts.get("openChat").is_none());assert!(f.probe.operation("shortcuts").is_err());
}
#[test]
fn shortcuts_patch_noop_and_per_action_cas_have_no_implicit_reset_or_retry(){
 let f=EffectFixture::new();f.save();let(_owner,frame)=frame();let old=fs::read(&f.path).unwrap();
 let r=apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,serde_json::Value::Null),fixture_context,||Ok(())).unwrap();assert!(!r.changed&&!r.new_registration_requires_review);assert_eq!(old,fs::read(&f.path).unwrap());
 apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,value("Super+Z")),fixture_context,||Ok(())).unwrap();
 assert!(apply_with(&frame,&f.current,&f.path,request(serde_json::Value::Null,value("Ctrl+Alt+Space")),fixture_context,||Ok(())).is_err());
 apply_with(&frame,&f.current,&f.path,request(value("Super+Z"),serde_json::Value::Null),fixture_context,||Ok(())).unwrap();assert!(!f.current.lock().unwrap().shortcuts.contains_key("openChat"));assert!(f.probe.operation("shortcuts").is_err());
}
