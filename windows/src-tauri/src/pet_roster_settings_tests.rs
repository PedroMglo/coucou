use super::*;
use serde_json::{json, Value};
use std::{cell::Cell, sync::Arc};
fn initial() -> Settings {
    let mut s=Settings::default();s.main_pill="agent_codex".into();
    s.active_integrations=vec!["agent_copilot".into(),"agent_antigravity".into(),"ai_ollama".into(),"integration_spotify".into()];s
}
fn active(id:&str,enabled:bool)->Request {Request::SetActive{id:id.into(),enabled}}
fn encode(s:&Settings)->Value {serde_json::to_value(s).unwrap()}
fn apply(s:&Mutex<Settings>,r:Request)->Roster {apply_with(s,r,|_|Ok(()),|_|{}).unwrap()}

#[test]fn request_accepts_only_explicit_operations_with_typed_fields(){
 for input in [json!({}),Value::Null,json!([]),json!({"action":"toggle","id":"ai_ollama"}),json!({"action":"setActive","id":"ai_ollama"}),json!({"action":"setActive","id":"ai_ollama","enabled":null}),json!({"action":"setMain","id":"agent_codex","activeIntegrations":[]})]{assert!(serde_json::from_value::<Request>(input).is_err());}
 assert!(serde_json::from_str::<Request>(r#"{"action":"setActive","id":"ai_ollama","enabled":false,"enabled":true}"#).is_err());
 assert!(matches!(serde_json::from_value::<Request>(json!({"action":"setActive","id":"ai_ollama","enabled":false})).unwrap(),Request::SetActive{enabled:false,..}));
}
#[test]fn adding_a_different_pet_preserves_gravity_and_every_other_setting(){
 let before=initial();let current=Mutex::new(before.clone());let mut expected=before;expected.active_integrations.push("integration_notion".into());
 let result=apply(&current,active("integration_notion",true));assert_eq!(result,Roster::from(&expected));assert_eq!(encode(&current.lock().unwrap()),encode(&expected));
}
#[test]fn explicit_removal_affects_only_its_target_and_repeated_intentions_are_idempotent(){
 let current=Mutex::new(initial());let first=apply(&current,active("agent_copilot",false));
 assert!(first.active_integrations.contains(&"agent_antigravity".into()));assert!(!first.active_integrations.contains(&"agent_copilot".into()));
 assert_eq!(apply(&current,active("agent_copilot",false)),first);
 let added=apply(&current,active("agent_copilot",true));assert_eq!(apply(&current,active("agent_copilot",true)),added);
}
#[test]fn selecting_main_removes_only_that_pets_additional_entry(){
 let mut before=initial();before.active_integrations.push("legacy-unknown".into());let current=Mutex::new(before.clone());
 let mut expected=before;expected.main_pill="agent_antigravity".into();expected.active_integrations.retain(|id|id!="agent_antigravity");
 assert_eq!(apply(&current,Request::SetMain{id:"agent_antigravity".into()}),Roster::from(&expected));assert_eq!(encode(&current.lock().unwrap()),encode(&expected));
}
#[test]fn retired_unknown_os_unsupported_and_ineligible_main_requests_cannot_write_or_publish(){
 let mut requests=vec![active("integration_github",true),active("unknown",true),active("integration_music",true),active("agent_codex",false),Request::SetMain{id:"ai_ollama".into()}];
 if !cfg!(target_os="windows"){requests.push(active("agent_claude-desktop",true));}
 for request in requests{let before=initial();let current=Mutex::new(before.clone());let writes=Cell::new(0);let events=Cell::new(0);
  assert!(apply_with(&current,request,|_|{writes.set(writes.get()+1);Ok(())},|_|events.set(events.get()+1)).is_err());assert_eq!(writes.get(),0);assert_eq!(events.get(),0);assert_eq!(encode(&current.lock().unwrap()),encode(&before));
 }
}
#[test]fn capacity_is_checked_against_current_unique_offered_pets_without_erasing_legacy_ids(){
 let mut s=initial();s.active_integrations.extend(["integration_notion".into(),"integration_resend".into(),"legacy-unknown".into()]);let current=Mutex::new(s);
 assert!(apply_with(&current,active("integration_n8n",true),|_|panic!("capacity write"),|_|panic!("capacity publish")).is_err());
 apply(&current,active("agent_copilot",false));let roster=apply(&current,active("integration_n8n",true));assert!(roster.active_integrations.contains(&"legacy-unknown".into()));assert!(roster.active_integrations.contains(&"agent_antigravity".into()));
}
#[test]fn failed_persistence_leaves_memory_unchanged_and_emits_nothing(){
 let before=initial();let current=Mutex::new(before.clone());let published=Cell::new(false);
 assert!(apply_with(&current,active("agent_copilot",false),|_|Err(io::Error::other("synthetic disk failure")),|_|published.set(true)).is_err());assert!(!published.get());assert_eq!(encode(&current.lock().unwrap()),encode(&before));
}
#[test]fn stale_general_settings_reproduces_unguarded_loss_and_preserves_roster_with_guard(){
 let fresh=initial();let mut stale=fresh.clone();stale.active_integrations.retain(|id|id!="agent_antigravity");stale.main_pill="integration_claude".into();stale.sound_volume=0.37;
 let unguarded=Mutex::new(fresh.clone());crate::settings::update_with(&unguarded,|next|*next=stale.clone(),|_|Ok(()),|_|{}).unwrap();assert!(!unguarded.lock().unwrap().active_integrations.contains(&"agent_antigravity".into()));
 let current=Mutex::new(fresh.clone());let committed=crate::settings::update_with(&current,|next|{let mut incoming=stale.clone();preserve_incoming(next,&mut incoming);*next=incoming;},|_|Ok(()),|_|{}).unwrap();
 assert_eq!(Roster::from(&committed),Roster::from(&fresh));assert_eq!(committed.sound_volume,0.37);
}
#[test]fn concurrent_disjoint_intentions_merge_instead_of_replacing_other_pets(){
 let current=Arc::new(Mutex::new(initial()));let start=Arc::new(std::sync::Barrier::new(3));let mut threads=vec![];
 for id in ["integration_notion","integration_resend"]{let current=current.clone();let start=start.clone();threads.push(std::thread::spawn(move||{start.wait();apply(&current,active(id,true));}));}
 start.wait();for thread in threads{thread.join().unwrap();}let roster=Roster::from(&*current.lock().unwrap());
 for id in ["integration_notion","integration_resend","agent_antigravity","agent_copilot"]{assert!(roster.active_integrations.contains(&id.into()));}assert_eq!(roster.active_integrations.len(),6);
}
#[test]fn write_commit_publish_and_receipt_are_in_order_under_the_same_mutex(){
 let current=Mutex::new(initial());let saved=std::cell::RefCell::new(None);let published=Cell::new(false);
 let result=apply_with(&current,active("agent_copilot",false),|next|{assert!(current.try_lock().is_err());*saved.borrow_mut()=Some(encode(next));Ok(())},|next|{assert!(current.try_lock().is_err());assert_eq!(saved.borrow().as_ref(),Some(&encode(next)));published.set(true);}).unwrap();
 assert!(published.get());assert_eq!(result,Roster::from(&*current.lock().unwrap()));
}
#[test]fn only_the_two_exact_native_preference_surfaces_are_authorized(){
 for label in ["island","settings"]{assert!(authorize_window(label,"island","settings").is_ok());}
 for label in ["","Settings","pet-chat","island-extra"]{assert!(authorize_window(label,"island","settings").is_err());}
}
