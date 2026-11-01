use super::{context,model::*,store::nonce};
use crate::chat::Chat;
use serde_json::json;
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt,sync::{Arc,atomic::AtomicBool}};
struct Fixture{path:PathBuf}
impl Fixture{fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-context-{}",nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();Self{path}}fn root(&self)->PathBuf{self.path.join("contexts")}fn main(&self)->Chat{Chat::persistent(self.path.join("main.json"))}}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
fn goal(provider:&str,context:Context)->Record{let id=crate::pet_config::pet_id(provider).unwrap();Record::new(Spec{restored_from:None,request_id:"goal-one".into(),owner:Owner{pet_id:id.into(),provider:provider.into(),adapter:if crate::pet_config::service(id){"codex".into()}else{provider.into()},model_hint:None},context,intent:"Complete the original goal".into(),criteria:vec!["Verify results".into()]},100_000,nonce().unwrap()).unwrap()}
fn message(chat:&Chat,provider:&str,q:&str){let t=chat.begin(provider);chat.commit(&t,json!({"role":"user","content":q}),json!({"role":"assistant","content":"answer"}),q,"answer").unwrap();}
fn bind(r:&mut Record,binding:ContextBinding){r.apply("prepare".into(),Action::BindContext{binding},100_001,nonce().unwrap()).unwrap();}

#[test]fn dedicated_goal_context_does_not_create_or_select_a_main_conversation(){
    let f=Fixture::new();let main=f.main();message(&main,"codex","visible conversation");let visible=main.conversation("codex").unwrap();let bytes=fs::read(f.path.join("main.json")).unwrap();let mut g=goal("codex",Context::Unbound);
    let(b,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();assert_ne!(b.conversation_id,visible.id);assert!(private.conversation("codex").unwrap().messages.is_empty());assert_eq!(main.conversation("codex").unwrap().id,visible.id);assert_eq!(fs::read(f.path.join("main.json")).unwrap(),bytes);
    bind(&mut g,b.clone());assert_eq!(g.schema_version,2);drop(private);let(reopened,private)=context::open(&f.root(),&g,&main).unwrap();assert_eq!(reopened,b);assert_eq!(private.conversation("codex").unwrap().id,b.conversation_id);
}
#[test]fn explicit_snapshot_copies_public_history_without_sessions_tasks_or_grants(){
    let f=Fixture::new();let main=f.main();let t=main.begin("codex");main.start_task(&t,"source request","/tmp","codex","fixture",None).unwrap();main.remember_session(&t,"native-original","/tmp",false);main.commit_task_reply(&t,json!({"role":"user","content":"source"}),json!({"role":"assistant","content":"answer"}),"source","answer").unwrap();
    let original=main.conversation("codex").unwrap();let g=goal("codex",Context::Mochi{conversation_id:original.id.clone()});
    let(b,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Snapshot,"prepare").unwrap();assert_eq!(b.source_conversation_id.as_deref(),Some(original.id.as_str()));assert_eq!(b.copied_messages,2);assert_eq!(private.conversation("codex").unwrap().messages,original.messages);assert!(private.tasks().is_empty());
    let turn=private.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();assert!(turn.session.is_none());let lease=private.lease(&turn);drop(lease);
    let saved=crate::assistant_memory::load_goal(&f.root().join(&g.id).join("conversation.json"),"codex",&b.conversation_id).unwrap();let s=&saved.pets["codex"];assert!(s.attachment_history.is_empty());assert!(s.request_history.is_empty());assert!(s.tasks.is_empty());assert!(s.session.is_none());
}
#[test]fn context_retries_keep_the_original_snapshot_when_main_history_changes(){
    let f=Fixture::new();let main=f.main();message(&main,"codex","first");let cid=main.conversation("codex").unwrap().id;let mut g=goal("codex",Context::Mochi{conversation_id:cid});
    let(b,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Snapshot,"prepare").unwrap();let old=private.conversation("codex").unwrap().messages;drop(private);message(&main,"codex","later");
    let(retry,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Snapshot,"prepare").unwrap();assert_eq!(retry,b);assert_eq!(private.conversation("codex").unwrap().messages,old);drop(private);bind(&mut g,b);
    assert!(context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"different").is_err());
}
#[test]fn manual_and_isolated_goal_workers_share_physical_admission_until_drain(){
    let f=Fixture::new();let main=f.main();message(&main,"codex","visible");let g=goal("codex",Context::Unbound);let(_,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();
    let turn=private.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();let worker=private.lease(&turn);private.cancel_provider("codex");assert!(main.reserve_conversation("codex",None).is_err());assert!(main.capability_health().1.contains("codex"));drop(worker);assert!(main.reserve_conversation("codex",None).is_ok());
    let reservation=main.reserve_conversation("codex",None).unwrap();assert!(private.reserve_conversation("codex",None).is_err());drop(reservation);assert!(private.reserve_conversation("codex",None).is_ok());
}
#[test]fn native_callback_origin_stays_on_the_goal_after_main_selection_changes(){
    let f=Fixture::new();let main=f.main();message(&main,"codex","original view");let g=goal("codex",Context::Unbound);let(b,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();
    let turn=private.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();let _worker=private.lease(&turn);main.reset_provider("codex").unwrap();let visible=main.conversation("codex").unwrap();let origin=private.request_origin(&turn).unwrap();assert_eq!(origin["goalId"],g.id);assert_eq!(origin["conversationId"],b.conversation_id);assert_ne!(origin["conversationId"],visible.id);
}
#[test]fn all_configured_llm_pet_contexts_have_independent_ids_and_locks(){
    let f=Fixture::new();let main=f.main();let mut ids=std::collections::BTreeSet::new();let mut held=vec![];
    for p in ["codex","copilot","antigravity","ollama","integration_notion"]{let g=goal(p,Context::Unbound);let(b,c)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();assert!(ids.insert(b.conversation_id));assert!(context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").is_err());held.push(c);}
    assert_eq!(held.len(),5);
}
#[test]fn missing_or_foreign_goal_journals_are_never_initialized_or_normalized_on_read(){
    let f=Fixture::new();let main=f.main();let mut g=goal("codex",Context::Unbound);let(b,c)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();drop(c);bind(&mut g,b);
    let path=f.root().join(&g.id).join("conversation.json");fs::remove_file(&path).unwrap();assert!(context::open(&f.root(),&g,&main).is_err());assert!(!path.exists());
    let bytes=br#"{"version":4,"pets":{"foreign-provider":{"id":"foreign","messages":[],"session":null}},"archives":{}}"#;fs::write(&path,bytes).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();assert!(context::open(&f.root(),&g,&main).is_err());assert_eq!(fs::read(&path).unwrap(),bytes);
}
#[test]fn changed_context_marker_revokes_operations_but_not_a_live_workers_slot(){
    let f=Fixture::new();let main=f.main();let g=goal("codex",Context::Unbound);let(_,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();let turn=private.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();let worker=private.lease(&turn);
    let journal=f.root().join(&g.id).join("conversation.json");let before=fs::read(&journal).unwrap();fs::write(f.root().join(&g.id).join(".context.json"),b"changed").unwrap();assert!(!private.is_current(&turn));assert!(private.commit(&turn,json!({}),json!({}),"unsafe","unsafe").is_err());assert_eq!(fs::read(&journal).unwrap(),before);assert!(main.reserve_conversation("codex",None).is_err());drop(worker);assert!(main.reserve_conversation("codex",None).is_ok());
}
#[test]fn snapshot_bounds_are_explicit_and_original_history_remains_complete(){
    let f=Fixture::new();let main=f.main();for n in 0..50{message(&main,"codex",&format!("public {n}"));}let original=main.conversation("codex").unwrap();let g=goal("codex",Context::Mochi{conversation_id:original.id.clone()});let(b,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Snapshot,"prepare").unwrap();assert_eq!(b.source_messages,100);assert_eq!(b.copied_messages,80);assert_eq!(main.conversation("codex").unwrap().messages.len(),100);assert_eq!(private.conversation("codex").unwrap().messages.len(),80);
}

#[test]fn replacing_a_context_lock_revokes_writes_without_releasing_a_live_worker(){
    let f=Fixture::new();let main=f.main();let g=goal("codex",Context::Unbound);let(_,private)=context::prepare(&f.root(),&g,&main,PreparationMode::Dedicated,"prepare").unwrap();
    let turn=private.begin_cli("codex",Arc::new(AtomicBool::new(false))).unwrap();let worker=private.lease(&turn);
    let dir=f.root().join(&g.id);let journal=dir.join("conversation.json");let before=fs::read(&journal).unwrap();
    fs::rename(dir.join(".context.lock"),dir.join(".original-lock")).unwrap();fs::write(dir.join(".context.lock"),b"").unwrap();fs::set_permissions(dir.join(".context.lock"),fs::Permissions::from_mode(0o600)).unwrap();
    assert!(!private.is_current(&turn));assert!(private.commit(&turn,json!({}),json!({}),"unsafe","unsafe").is_err());assert_eq!(fs::read(&journal).unwrap(),before);
    assert!(main.reserve_conversation("codex",None).is_err());drop(worker);assert!(main.reserve_conversation("codex",None).is_ok());
}
