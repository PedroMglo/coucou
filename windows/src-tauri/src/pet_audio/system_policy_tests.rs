use super::*;
use serde_json::json;
fn node(kind:Kind,key:&str)->Node {
 Node{key:key.into(),kind,label:"Device".into(),description:"Description".into(),volume:Some(40.0),muted:Some(false),balance:Some(0.0),is_default:false,device_key:None,monitor:false,controllable:true,can_volume:true,can_balance:true,active:true,options:vec![],active_option:None,selector:"PRIVATE_SELECTOR".into(),index:991}
}
fn snapshot()->Snapshot {
 Snapshot{available:true,observed_at:Some(1_792_500_000_000),outputs:vec![node(Kind::Output,"out")],inputs:vec![node(Kind::Input,"in")],cards:vec![node(Kind::Card,"card")],playback:vec![node(Kind::Playback,"play")],recording:vec![node(Kind::Recording,"record")],error:None}
}
fn request(key:&str,action:Action)->Request{Request{target_key:key.into(),action,value:Some(40.0),muted:Some(false),destination_key:None,option:None}}
#[test]fn system_access_requires_the_actual_surface_visible_and_enabled(){
 for correct in [false,true]{for visible in [false,true]{for enabled in [false,true]{assert_eq!(allowed(correct,visible,enabled),correct&&visible&&enabled);}}}
}
#[test]fn system_is_independent_of_pet_permissions_and_has_no_stream_payload(){
 let v=serde_json::to_value(devices(snapshot())).unwrap();assert_eq!(v["scope"],"systemDevices");assert_eq!(v["outputs"].as_array().unwrap().len(),1);assert_eq!(v["inputs"].as_array().unwrap().len(),1);assert_eq!(v["cards"].as_array().unwrap().len(),1);assert!(v.get("playback").is_none());assert!(v.get("recording").is_none());assert!(v.get("pet").is_none());assert!(!v.to_string().contains("PRIVATE_SELECTOR"));
}
#[test]fn unavailable_and_zero_readings_keep_their_original_meanings(){
 let mut s=snapshot();s.outputs[0].volume=Some(0.0);s.inputs[0].muted=None;let v=serde_json::to_value(devices(s)).unwrap();assert_eq!(v["outputs"][0]["volume"].as_f64(),Some(0.0));assert!(v["inputs"][0]["muted"].is_null());let v=serde_json::to_value(devices(Snapshot::default())).unwrap();assert_eq!(v["available"],false);assert!(v["observedAt"].is_null());
}
#[test]fn physical_device_actions_are_admitted_for_engine_validation(){
 let s=snapshot();assert!(request_allowed(&s,&request("out",Action::Mute),&||true));assert!(request_allowed(&s,&request("in",Action::Default),&||true));assert!(request_allowed(&s,&request("card",Action::Profile),&||true));
}
#[test]fn real_stream_handles_cannot_escape_the_system_device_scope(){
 let s=snapshot();for key in ["play","record"]{for action in [Action::Mute,Action::Volume,Action::Default,Action::Port,Action::Balance]{assert!(!request_allowed(&s,&request(key,action),&||true));}}
}
#[test]fn routing_and_stale_or_uncontrollable_targets_never_use_a_default(){
 let mut s=snapshot();assert!(!request_allowed(&s,&request("out",Action::Route),&||true));assert!(!request_allowed(&s,&request("missing",Action::Mute),&||true));s.outputs[0].controllable=false;assert!(!request_allowed(&s,&request("out",Action::Mute),&||true));s.available=false;assert!(!request_allowed(&s,&request("in",Action::Mute),&||true));
}
#[test]fn projection_keeps_exact_keys_without_mutating_the_shared_snapshot(){
 let original=snapshot();let v=serde_json::to_value(devices(original.clone())).unwrap();assert_eq!(v["outputs"][0]["key"],"out");assert_eq!(original.playback.len(),1);assert_eq!(original.recording.len(),1);assert_eq!(v["observedAt"],json!(1_792_500_000_000u64));
}

#[test]fn an_admitted_request_is_rechecked_after_the_read_before_mutation(){
 use std::sync::atomic::{AtomicBool,Ordering};
 let visible=AtomicBool::new(true);let still_allowed=||visible.load(Ordering::SeqCst);
 let observed=snapshot();let r=request("out",Action::Mute);
 assert!(request_allowed(&observed,&r,&still_allowed));
 visible.store(false,Ordering::SeqCst);
 assert!(!request_allowed(&observed,&r,&still_allowed));
}
