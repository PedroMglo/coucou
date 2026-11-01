use super::*;
#[path="memory_goal_execution_tests.rs"]mod execution_tests;
#[path="memory_review_tests.rs"]mod review_tests;
use serde_json::{json,Value};
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt,sync::atomic::AtomicBool};
struct Fixture{root:PathBuf}
impl Fixture{
 fn new()->Self{let root=std::env::temp_dir().join(format!("goal-memory-{}",nonce().unwrap()));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();Self{root}}
 fn contexts(&self)->PathBuf{self.root.join("contexts")}
 fn catalog(&self)->PathBuf{self.root.join("memories.json")}
 fn main(&self)->Chat{Chat::persistent(self.root.join("main.json"))}
 fn write(&self,items:Vec<Value>){let generation=fs::read(self.catalog()).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok()).and_then(|v|v["generation"].as_u64()).unwrap_or(0)+1;fs::write(self.catalog(),serde_json::to_vec(&json!({"version":1,"generation":generation,"items":items})).unwrap()).unwrap();fs::set_permissions(self.catalog(),fs::Permissions::from_mode(0o600)).unwrap();}
 fn record(id:&str,pet:&str,cid:Option<&str>)->Value{json!({"id":id,"revision":1,"owner":{"petId":pet,"scope":cid.map(|c|json!({"kind":"conversation","conversationId":c})).unwrap_or(json!({"kind":"pet"}))},"kind":"note","title":"Explicit goal data","content":"PRIVATE_GOAL_MEMORY_SENTINEL","provenance":"explicitUser","sensitivity":"sensitive","retention":"untilDeleted","createdAt":100,"updatedAt":100,"observedAt":100,"expiresAt":null})}
 fn goal(context:Context)->Record{Record::new(Spec{restored_from:None,request_id:"new-goal".into(),owner:Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:Some("fixture-free".into())},context,intent:"Use only explicitly admitted data".into(),criteria:vec!["Verify the original criterion".into()]},100_000,nonce().unwrap()).unwrap()}
 fn selected(id:&str)->Selected{Selected{id:id.into(),revision:1}}
 fn prepare(&self,g:&Record,main:&Chat,mode:Mode,selection:Option<&[Selected]>)->Result<(Binding,Chat)>{prepare_at(&self.contexts(),g,main,mode,"prepare",selection,&self.catalog())}
 fn bind(g:&mut Record,b:Binding){g.apply("prepare".into(),Action::BindContext{binding:b},100_001,nonce().unwrap()).unwrap();}
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}

#[test]fn selected_goal_context_uses_conditional_formats_and_never_copies_raw_memory_into_seed_or_public_history(){
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("note-one","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("note-one")])).unwrap();assert_eq!(b.schema_version,2);assert_eq!(b.memory.as_ref().unwrap().references.len(),1);assert!(private.history("ollama").is_empty());assert!(!f.root.join("main.json").exists());
 let header=f.contexts().join(&g.id).join(HEADER);let journal=f.contexts().join(&g.id).join(JOURNAL);assert!(!fs::read_to_string(&header).unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));assert!(!fs::read_to_string(&journal).unwrap().contains("PRIVATE_GOAL_MEMORY_SENTINEL"));Fixture::bind(&mut g,b.clone());assert_eq!(g.schema_version,11);drop(private);let(reopened,private)=open_at(&f.contexts(),&g,&main,&f.catalog()).unwrap();assert_eq!(reopened,b);assert!(private.goal_context_current(&g.id));let bytes=serde_json::to_vec(&g).unwrap();let decoded:Record=serde_json::from_slice(&bytes).unwrap();decoded.validate().unwrap();
 for version in 1..=10{let mut old:Value=serde_json::from_slice(&bytes).unwrap();old["schemaVersion"]=json!(version);assert!(serde_json::from_value::<Record>(old).unwrap().validate().is_err());}
}
#[test]fn selection_scope_never_uses_a_generated_goal_cid_to_authorize_other_pet_or_conversation_records(){
 for mode in ["dedicated-conversation","foreign-pet","foreign-cid"]{
  let f=Fixture::new();let main=f.main();let public=main.conversation_id("ollama").unwrap();let context=if mode=="dedicated-conversation"{Context::Unbound}else{Context::Mochi{conversation_id:public.clone()}};let g=Fixture::goal(context);let mut row=Fixture::record("selected","ai_ollama",None);
  match mode{"dedicated-conversation"=>row["owner"]["scope"]=json!({"kind":"conversation","conversationId":cid(&g)}),"foreign-pet"=>row["owner"]["petId"]=json!("agent_codex"),_=>row["owner"]["scope"]=json!({"kind":"conversation","conversationId":"another-public-cid"})};f.write(vec![row]);let before=fs::read(f.catalog()).unwrap();assert!(f.prepare(&g,&main,if mode=="dedicated-conversation"{Mode::Dedicated}else{Mode::Snapshot},Some(&[Fixture::selected("selected")])).is_err());assert!(!f.contexts().join(&g.id).exists());assert_eq!(fs::read(f.catalog()).unwrap(),before);
 }
 let f=Fixture::new();let main=f.main();let public=main.conversation_id("ollama").unwrap();let g=Fixture::goal(Context::Mochi{conversation_id:public.clone()});f.write(vec![Fixture::record("selected","ai_ollama",Some(&public))]);let(b,_)=f.prepare(&g,&main,Mode::Snapshot,Some(&[Fixture::selected("selected")])).unwrap();assert_eq!(b.source_conversation_id.as_deref(),Some(public.as_str()));assert_ne!(b.conversation_id,public);
}
#[test]fn retries_preserve_exact_selected_data_and_none_retains_the_legacy_byte_shape(){
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("first","ai_ollama",None),Fixture::record("second","ai_ollama",None)]);let g=Fixture::goal(Context::Unbound);let(b,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("first")])).unwrap();drop(private);let path=f.contexts().join(&g.id).join(HEADER);let bytes=fs::read(&path).unwrap();
 for input in [None,Some(vec![Fixture::selected("second")]),Some(vec![Fixture::selected("first"),Fixture::selected("second")])]{assert!(f.prepare(&g,&main,Mode::Dedicated,input.as_deref()).is_err());assert_eq!(fs::read(&path).unwrap(),bytes);}
 let(retry,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("first")])).unwrap();assert_eq!(retry,b);drop(private);
 let f=Fixture::new();let main=f.main();let mut g=Fixture::goal(Context::Unbound);let(b,_)=f.prepare(&g,&main,Mode::Dedicated,None).unwrap();assert_eq!(b.schema_version,1);assert!(b.memory.is_none());assert!(serde_json::to_value(&b).unwrap().get("memory").is_none());Fixture::bind(&mut g,b);assert_eq!(g.schema_version,2);
}
#[test]fn selected_memory_revocation_invalidates_existing_goal_fence_without_releasing_its_live_worker_or_rewriting_data(){
 let f=Fixture::new();let main=f.main();let row=Fixture::record("selected","ai_ollama",None);f.write(vec![row.clone()]);let mut g=Fixture::goal(Context::Unbound);let(b,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);let turn=private.begin_cli("ollama",Arc::new(AtomicBool::new(false))).unwrap();let worker=private.lease(&turn);let journal=f.contexts().join(&g.id).join(JOURNAL);let original=fs::read(&journal).unwrap();
 f.write(vec![row.clone(),Fixture::record("other","agent_copilot",None)]);assert!(private.goal_context_current(&g.id));f.write(vec![]);assert!(!private.goal_context_current(&g.id));assert!(!private.is_current(&turn));assert!(private.commit_reply(&turn,json!({"role":"user","content":"must not commit"}),json!({"role":"assistant","content":"late answer"}),"must not commit","late answer").is_err());assert_eq!(fs::read(&journal).unwrap(),original);assert!(main.reserve_conversation("ollama",None).is_err());
 f.write(vec![row]);assert!(!private.goal_context_current(&g.id),"a revoked live capture cannot revive from restored catalog data");drop(worker);assert!(main.reserve_conversation("ollama",None).is_ok());
}
#[test]fn expired_corrupt_and_oversized_selected_data_never_normalizes_the_existing_context_or_catalog(){
 for mode in ["expired","oversized"]{let f=Fixture::new();let main=f.main();let g=Fixture::goal(Context::Unbound);let mut row=Fixture::record("selected","ai_ollama",None);let rows=if mode=="expired"{row["expiresAt"]=json!(1000);vec![row]}else{(0..9).map(|i|{let mut r=row.clone();r["id"]=json!(format!("note-{i}"));r["content"]=json!("x".repeat(8000));r}).collect()};let selected=rows.iter().map(|r|Fixture::selected(r["id"].as_str().unwrap())).collect::<Vec<_>>();f.write(rows);let before=fs::read(f.catalog()).unwrap();assert!(f.prepare(&g,&main,Mode::Dedicated,Some(&selected)).is_err());assert!(!f.contexts().join(&g.id).exists());assert_eq!(fs::read(f.catalog()).unwrap(),before);}
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("selected","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,private)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);drop(private);let header=f.contexts().join(&g.id).join(HEADER);let before=fs::read(&header).unwrap();fs::write(f.catalog(),b"malformed retained catalog").unwrap();assert!(open_at(&f.contexts(),&g,&main,&f.catalog()).is_err());assert_eq!(fs::read(f.catalog()).unwrap(),b"malformed retained catalog");assert_eq!(fs::read(&header).unwrap(),before);
}
#[test]fn data_references_alone_do_not_create_an_autonomous_admission_or_bypass_goal_specific_review(){
 let f=Fixture::new();let main=f.main();f.write(vec![Fixture::record("selected","ai_ollama",None)]);let mut g=Fixture::goal(Context::Unbound);let(b,_)=f.prepare(&g,&main,Mode::Dedicated,Some(&[Fixture::selected("selected")])).unwrap();Fixture::bind(&mut g,b);let now=100_002;let descriptor=crate::pet_runtime::admission_tests::descriptor(&g,now);let result=crate::pet_runtime::admission::Evidence::observe(&g,&descriptor,"a".repeat(64),now,false,false);assert_eq!(result.err(),Some(crate::pet_runtime::admission::Denial::Unsupported));assert!(g.admissions.is_empty());assert_eq!(g.state,State::Draft);
}
