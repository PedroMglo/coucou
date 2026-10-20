//! Synthetic Linux filesystem failures; never loads the user's recap/history.
use super::*;
use serde_json::json;
use std::sync::atomic::{AtomicU64,Ordering};
static NEXT:AtomicU64=AtomicU64::new(1);
const T:i64=1_791_158_400;
struct Fixture {dir:PathBuf,path:PathBuf,recap:Recap}
impl Fixture {
 fn new()->Self {
  let dir=std::env::temp_dir().join(format!("coucou-recap-persistence-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
  std::fs::create_dir(&dir).unwrap();let path=dir.join("recap.json");let recap=Recap::load(path.clone(),T);Self{dir,path,recap}
 }
 fn seed(&mut self){
  self.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"completed","cwd":"/synthetic/project"}),T);
  self.recap.observe(&json!({"hook_event_name":"Stop","session_id":"completed"}),T+1);
  assert_eq!(self.recap.view(0).turns.len(),1);
 }
 fn block(&mut self){
  let blocked=self.dir.join("blocked-target");std::fs::create_dir(&blocked).unwrap();std::fs::write(blocked.join("marker"),b"preserve").unwrap();self.recap.path=blocked;
 }
 fn disk(&self)->History{serde_json::from_slice(&std::fs::read(&self.path).unwrap()).unwrap()}
}
impl Drop for Fixture {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.dir);}}
fn failed(receipt:&MutationReceipt){assert!(!receipt.persisted);assert!(!receipt.history_cleared);assert_eq!(receipt.error_code,Some("writeFailed"));}

#[test]fn failed_clear_preserves_ram_drafts_native_request_identity_and_disk(){
 let mut f=Fixture::new();f.seed();let disk=std::fs::read(&f.path).unwrap();let history=f.recap.history.clone();
 f.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"pending"}),T+2);
 f.recap.note_request("native-request",&json!({"coucou_agent":"codex"}));f.block();let result=f.recap.clear();failed(&result);
 assert_eq!(f.recap.history,history);assert!(f.recap.drafts.contains_key("pending"));assert_eq!(f.recap.requests.get("native-request").map(String::as_str),Some("agent_codex"));
 assert_eq!(std::fs::read(&f.path).unwrap(),disk);assert_eq!(std::fs::read(f.recap.path.join("marker")).unwrap(),b"preserve");
 assert!(!f.recap.path.with_extension(format!("json.coucou-{}",std::process::id())).exists());
}
#[test]fn failed_disable_stops_collection_in_this_process_without_claiming_persistence(){
 let mut f=Fixture::new();f.seed();f.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"pending"}),T+2);f.recap.note_request("r",&json!({}));f.block();
 let result=f.recap.set_enabled(false);failed(&result);assert!(!result.prefs.enabled);assert!(!f.recap.prefs().enabled);assert!(f.recap.drafts.is_empty()&&f.recap.requests.is_empty());assert!(f.disk().prefs.enabled);
 f.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"new"}),T+3);f.recap.observe(&json!({"hook_event_name":"Stop","session_id":"new"}),T+4);f.recap.record_decision("unknown","allow",T+5);
 assert_eq!(f.recap.view(0).turns.len(),1);assert!(f.recap.view(0).decisions.is_empty());
}
#[test]fn failed_enable_preserves_disabled_state_and_disk(){
 let mut f=Fixture::new();assert!(f.recap.set_enabled(false).persisted);f.block();let result=f.recap.set_enabled(true);failed(&result);assert!(!result.prefs.enabled);assert!(!f.disk().prefs.enabled);
 f.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"new"}),T);assert!(f.recap.drafts.is_empty());
}
#[test]fn failed_hide_protects_current_process_while_disk_remains_explicitly_unconfirmed(){
 let mut f=Fixture::new();f.seed();f.block();let result=f.recap.set_hide_projects(true);failed(&result);assert!(result.prefs.hide_projects&&f.recap.prefs().hide_projects);assert!(!f.disk().prefs.hide_projects);
}
#[test]fn failed_reveal_preserves_hidden_names(){
 let mut f=Fixture::new();assert!(f.recap.set_hide_projects(true).persisted);f.block();let result=f.recap.set_hide_projects(false);failed(&result);assert!(result.prefs.hide_projects&&f.disk().prefs.hide_projects);
}
#[test]fn failed_mark_shown_does_not_publish_new_week(){
 let mut f=Fixture::new();assert!(f.recap.mark_shown("2026-10-05").persisted);f.block();let result=f.recap.mark_shown("2026-10-12");failed(&result);assert_eq!(result.prefs.last_shown_week,"2026-10-05");assert_eq!(f.disk().prefs.last_shown_week,"2026-10-05");
}
#[test]fn invalid_mark_shown_neither_writes_nor_changes_preferences(){
 let mut f=Fixture::new();let original=f.recap.history.clone();let result=f.recap.mark_shown("not a week");assert!(!result.persisted&&!result.history_cleared);assert_eq!(result.error_code,Some("invalidWeek"));assert_eq!(f.recap.history,original);assert!(!f.path.exists());
}
#[test]fn successful_clear_and_preferences_are_acknowledged_and_survive_reload(){
 let mut f=Fixture::new();f.seed();assert!(f.recap.set_hide_projects(true).persisted);assert!(f.recap.mark_shown("2026-10-05").persisted);f.recap.note_request("still-pending",&json!({"coucou_agent":"codex"}));
 f.recap.observe(&json!({"hook_event_name":"UserPromptSubmit","session_id":"pending"}),T+2);let result=f.recap.clear();assert!(result.persisted&&result.history_cleared);assert_eq!(result.error_code,None);assert!(f.recap.drafts.is_empty());assert!(f.recap.requests.contains_key("still-pending"));
 let loaded=Recap::load(f.path.clone(),T+3);assert!(loaded.view(0).turns.is_empty()&&loaded.view(0).decisions.is_empty());assert_eq!(loaded.prefs(),&result.prefs);assert!(loaded.prefs().hide_projects);assert_eq!(loaded.prefs().last_shown_week,"2026-10-05");
}
#[test]fn explicit_retry_can_persist_existing_session_privacy_protection(){
 let mut f=Fixture::new();f.seed();f.block();failed(&f.recap.set_enabled(false));f.recap.path=f.path.clone();let result=f.recap.set_enabled(false);assert!(result.persisted&&!result.history_cleared);assert!(!result.prefs.enabled);assert!(!f.disk().prefs.enabled);assert_eq!(f.disk().turns.len(),1);
}
#[test]fn failure_receipt_is_camel_case_and_never_contains_filesystem_details(){
 let mut f=Fixture::new();f.seed();f.block();let result=serde_json::to_value(f.recap.clear()).unwrap();assert_eq!(result["persisted"],false);assert_eq!(result["historyCleared"],false);assert_eq!(result["errorCode"],"writeFailed");assert_eq!(result["prefs"]["hideProjects"],false);assert_eq!(result.as_object().unwrap().len(),4);let text=result.to_string();assert!(!text.contains("blocked-target")&&!text.contains(&f.dir.to_string_lossy().to_string()));
}
