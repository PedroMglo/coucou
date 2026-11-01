use super::*;
use crate::pet_data::restored_history::History;
use serde_json::json;
fn fixture()->(Chat,String,Arc<History>){
 let raw=json!({"version":4,"pets":{"codex":{"id":"restored-original-cid","created":1,"updated":2,"messages":[{"role":"user","content":"Preserved private original text"}],"session":{"id":"restored-native-sid","cwd":"/private-fixture/work","full_access":false},"tasks":[]}},"archives":{}});let bytes=serde_json::to_vec(&raw).unwrap();let h=Arc::new(History::capture(Some(&bytes)).unwrap());let mut chat=Chat::from_library(std::env::temp_dir().join(format!("coucou-restored-history-{}.json",crate::assistant_memory::new_id())),crate::assistant_memory::inspect_history_bytes(&bytes).unwrap(),false);chat.disk=None;chat.restored_history=Some(h.clone());(chat,"restored-original-cid".into(),h)
}
#[test]fn historical_conversation_is_readable_with_notice_but_cannot_reserve_or_start_a_turn(){
 let(chat,cid,_)=fixture();let before=chat.conversation("codex").unwrap();assert_eq!(before.id,cid);assert_eq!(before.messages[0]["content"],"Preserved private original text");assert!(before.notice.as_deref().unwrap().contains("restaurado"));assert!(chat.reserve_conversation("codex",Some(&cid)).is_err());assert!(chat.begin_cli("codex",Arc::new(AtomicBool::new(false))).is_err());let after=chat.conversation("codex").unwrap();assert_eq!(after.messages,before.messages);assert!(!after.busy);assert!(chat.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn a_fresh_conversation_can_obtain_normal_reservation_while_old_history_is_preserved(){
 let(chat,old,_)=fixture();chat.reset_provider("codex").unwrap();let view=chat.conversation("codex").unwrap();assert_ne!(view.id,old);assert!(view.notice.is_none());let pending=chat.reserve_conversation("codex",Some(&view.id)).unwrap();drop(pending);let rows=chat.conversations("codex","").unwrap();assert!(rows.iter().any(|r|r.id==old));
}
#[test]fn copying_a_native_session_to_a_new_cid_does_not_restore_its_authority(){
 let(chat,old,_)=fixture();{let mut all=chat.inner.lock().unwrap();all.pets.get_mut("codex").unwrap().id="fresh-cid-with-copied-session".into();}assert!(chat.reserve_conversation("codex",None).is_err());assert!(chat.begin_cli("codex",Arc::new(AtomicBool::new(false))).is_err());assert_ne!(chat.conversation("codex").unwrap().id,old);assert!(chat.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn provider_returning_an_old_sid_cancels_instead_of_adopting_it_for_new_work(){
 let(chat,_,_)=fixture();chat.reset_provider("codex").unwrap();let turn=chat.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();chat.remember_session(&turn,"restored-native-sid","/private-fixture/work",false);assert!(turn.cancel.load(Ordering::Acquire));let all=chat.inner.lock().unwrap();assert!(all.pets["codex"].session.is_none());
}
#[test]fn independent_native_session_can_be_remembered_without_changing_historical_index(){
 let(chat,_,history)=fixture();let hash=history.fingerprint().unwrap();chat.reset_provider("codex").unwrap();let turn=chat.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();chat.remember_session(&turn,"new-independent-native","/private-fixture/work",false);assert!(!turn.cancel.load(Ordering::Acquire));let all=chat.inner.lock().unwrap();assert_eq!(all.pets["codex"].session.as_ref().unwrap().id,"new-independent-native");assert_eq!(history.fingerprint().unwrap(),hash);
}
#[test]fn old_non_cli_begin_returns_cancelled_empty_turn_without_panicking_or_copying_history(){
 let(chat,_,_)=fixture();let turn=chat.begin("codex");assert!(turn.cancel.load(Ordering::Acquire));assert!(turn.history.is_empty());assert!(turn.session.is_none());assert!(chat.runtime_resources().busy().unwrap().is_empty());
}
#[test]fn old_task_without_native_or_conversation_origin_cannot_escape_through_fresh_resume(){
 let mut raw=json!({"version":4,"pets":{"codex":{"id":"restored-task-cid","created":1,"updated":2,"messages":[],"session":null,"tasks":[]}},"archives":{}});let mut task=crate::assistant_tasks::Task::new("codex","Preserved original task","/private-fixture/work","codex","fixture-free",1);task.phase="interrupted".into();raw["pets"]["codex"]["tasks"]=json!([task]);let bytes=serde_json::to_vec(&raw).unwrap();let mut chat=Chat::from_library(std::env::temp_dir().join("unused-restored-task-fixture.json"),crate::assistant_memory::inspect_history_bytes(&bytes).unwrap(),false);chat.disk=None;chat.restored_history=Some(Arc::new(History::capture(Some(&bytes)).unwrap()));let before=chat.tasks();assert_eq!(before.len(),1);assert!(chat.reserve_task_resume(&before[0].id).is_err());assert!(chat.runtime_resources().busy().unwrap().is_empty());assert_eq!(chat.tasks()[0].id,before[0].id);
}
#[test]fn guard_rechecked_before_io_cancels_without_freeing_the_owned_lease_early(){
 let(mut chat,_,_)=fixture();chat.restored_history=None;let turn=chat.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();let lease=chat.lease(&turn);let cid=chat.conversation("codex").unwrap().id;let raw=json!({"version":4,"pets":{"codex":{"id":cid,"created":1,"updated":2,"messages":[],"session":null,"tasks":[]}},"archives":{}});chat.restored_history=Some(Arc::new(History::capture(Some(&serde_json::to_vec(&raw).unwrap())).unwrap()));assert!(!chat.is_current(&turn));assert!(turn.cancel.load(Ordering::Acquire));assert!(!chat.runtime_resources().busy().unwrap().is_empty());drop(lease);assert!(chat.runtime_resources().busy().unwrap().is_empty());
}

#[test]fn historical_notice_keeps_existing_write_error_and_is_also_present_in_archived_read(){
 let(chat,cid,_)=fixture();{chat.inner.lock().unwrap().pets.get_mut("codex").unwrap().save_error=Some("original write failure preserved".into());}let v=chat.conversation("codex").unwrap();let notice=v.notice.unwrap();assert!(notice.contains("restaurado"));assert!(notice.contains("original write failure preserved"));let direct=chat.read_conversation("codex",&cid).unwrap();assert!(direct.notice.unwrap().contains("original write failure preserved"));
}
