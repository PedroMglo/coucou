use super::*;
use model::{Input,Kind,Sensitivity};
use std::{path::PathBuf,sync::{Arc,Barrier,atomic::{AtomicUsize,Ordering}},os::unix::fs::{PermissionsExt,symlink}};
struct Files{root:PathBuf}
impl Files{
 fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-memories-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&root).unwrap();std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();Self{root}}
 fn path(&self)->PathBuf{self.root.join("memories.json")}
}
impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
fn owner(pet:&str)->Owner{Owner{pet_id:pet.into(),scope:Scope::Pet{}}}
fn query(owner:Owner,text:&str)->Query{Query{owner,query:text.into(),limit:50,include_expired:false,offset:0,expected_revision:None}}
fn input(text:&str)->Input{Input{kind:Kind::Note,title:"Preferência diária".into(),content:text.into(),sensitivity:Sensitivity::Normal,expires_at:None}}
fn create(text:&str)->Mutation{Mutation::Create{input:input(text)}}
#[test]fn native_crud_reopen_search_and_stale_edits_preserve_the_original_winner(){
 let f=Files::new();let p=f.path();let own=owner("ai_ollama");let missing=list_at(&p,query(own.clone(),""),100,||Ok(())).unwrap();assert_eq!(missing.revision,"missing");assert!(std::fs::read_dir(&f.root).unwrap().next().is_none());
 let created=mutate_at(&p,own.clone(),"missing",create("Responder em português de Portugal"),100,||Ok(())).unwrap();assert_eq!(created.item.as_ref().unwrap().provenance,model::Provenance::ExplicitUser);
 let page=list_at(&p,query(own.clone(),"PORTUGUÊS portugal"),101,||Ok(())).unwrap();assert_eq!(page.items.len(),1);assert_eq!(page.items[0].entry.id,created.id);
 assert!(list_at(&p,query(own.clone(),"português ausente"),101,||Ok(())).unwrap().items.is_empty());
 let changed=mutate_at(&p,own.clone(),&created.revision,Mutation::Update{id:created.id.clone(),expected_item_revision:1,input:input("Use concise replies")},102,||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();
 assert_eq!(mutate_at(&p,own.clone(),&created.revision,create("stale"),103,||Ok(())).err(),Some("registryConflict"));assert_eq!(std::fs::read(&p).unwrap(),before);
 assert_eq!(mutate_at(&p,own.clone(),&changed.revision,Mutation::Delete{id:created.id.clone(),expected_item_revision:1},103,||Ok(())).err(),Some("memoryRevisionConflict"));
 let deleted=mutate_at(&p,own.clone(),&changed.revision,Mutation::Delete{id:created.id,expected_item_revision:2},104,||Ok(())).unwrap();assert!(deleted.item.is_none());assert!(list_at(&p,query(own,""),105,||Ok(())).unwrap().items.is_empty());assert!(!String::from_utf8(std::fs::read(&p).unwrap()).unwrap().contains("concise"));
 assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode()&0o777,0o600);
}
#[test]fn every_active_factory_pet_has_independent_memory_without_requiring_its_engine(){
 let f=Files::new();let p=f.path();let config=crate::pet_config::defaults();let chat=crate::chat::Chat::default();let mut revision="missing".to_string();let mut count=0;
 for pet in config.pets.keys().filter(|id|!crate::pet_config::retired(id)){
  let own=owner(pet);validate_owner(&config,&chat,&own).unwrap();let receipt=mutate_at(&p,own.clone(),&revision,create(pet),100,||validate_owner(&config,&chat,&own)).unwrap();revision=receipt.revision;count+=1;
 }
 assert!(count>=24);
 for pet in config.pets.keys().filter(|id|!crate::pet_config::retired(id)){
  let own=owner(pet);let page=list_at(&p,query(own.clone(),""),101,||validate_owner(&config,&chat,&own)).unwrap();assert_eq!(page.items.len(),1);assert_eq!(page.items[0].entry.content,*pet);
 }
 assert!(validate_owner(&config,&chat,&owner("integration_github")).is_err());assert!(validate_owner(&config,&chat,&owner("unknown")).is_err());
}
#[test]fn conversation_scope_checks_real_owner_and_archives_without_selecting_or_copying_messages(){
 let config=crate::pet_config::defaults();let chat=crate::chat::Chat::default();
 chat.fixture_restore_saved("codex",crate::chat::SavedConversation{id:"original-cid".into(),messages:vec![serde_json::json!({"role":"user","content":"public archive"})],..Default::default()});
 let own=Owner{pet_id:"agent_codex".into(),scope:Scope::Conversation{conversation_id:"original-cid".into()}};validate_owner(&config,&chat,&own).unwrap();
 let mut crossed=own.clone();crossed.pet_id="integration_notion".into();assert!(validate_owner(&config,&chat,&crossed).is_err());crossed=own.clone();crossed.scope=Scope::Conversation{conversation_id:"missing-cid".into()};assert!(validate_owner(&config,&chat,&crossed).is_err());
 let f=Files::new();let receipt=mutate_at(&f.path(),own.clone(),"missing",create("scoped note"),100,||validate_owner(&config,&chat,&own)).unwrap();
 assert!(list_at(&f.path(),query(owner("agent_codex"),""),101,||Ok(())).unwrap().items.is_empty());
 let mut other=own.clone();other.scope=Scope::Conversation{conversation_id:"other-cid".into()};assert_eq!(mutate_at(&f.path(),other,&receipt.revision,Mutation::Delete{id:receipt.id.clone(),expected_item_revision:1},101,||Ok(())).err(),Some("memoryNotFound"));
 chat.reset_provider("codex").unwrap();validate_owner(&config,&chat,&own).unwrap();assert_eq!(list_at(&f.path(),query(own.clone(),""),102,||validate_owner(&config,&chat,&own)).unwrap().items.len(),1);
 let absent=crate::chat::Chat::default();assert!(validate_owner(&config,&absent,&own).is_err());
 mutate_at(&f.path(),own.clone(),&receipt.revision,Mutation::Delete{id:receipt.id,expected_item_revision:1},103,||validate_owner_for(&config,&absent,&own,false)).unwrap();assert!(crate::private_registry::load_at::<model::Registry>(&f.path()).unwrap().registry.items.is_empty());
}
#[test]fn expiry_is_not_erasure_and_old_revisions_are_revoked_by_edit_or_delete(){
 let f=Files::new();let own=owner("agent_copilot");let mut value=input("sensitive original");value.expires_at=Some(110);value.sensitivity=Sensitivity::Sensitive;
 let created=mutate_at(&f.path(),own.clone(),"missing",Mutation::Create{input:value},100,||Ok(())).unwrap();let old=crate::private_registry::load_at::<model::Registry>(&f.path()).unwrap();
 assert_eq!(list_at(&f.path(),query(own.clone(),""),109,||Ok(())).unwrap().items.len(),1);assert!(list_at(&f.path(),query(own.clone(),""),110,||Ok(())).unwrap().items.is_empty());
 let mut expired=query(own.clone(),"");expired.include_expired=true;let page=list_at(&f.path(),expired,111,||Ok(())).unwrap();assert!(!page.items[0].available);assert_eq!(page.items[0].entry.sensitivity,Sensitivity::Sensitive);
 mutate_at(&f.path(),own,&created.revision,Mutation::Delete{id:created.id,expected_item_revision:1},112,||Ok(())).unwrap();assert!(!old.unchanged());assert!(crate::private_registry::load_at::<model::Registry>(&f.path()).unwrap().registry.items.is_empty());
}
#[test]fn concurrent_pet_writers_keep_one_cas_winner_and_final_native_guard_refuses_without_publication(){
 let rounds=std::env::var("COUCOU_QA_MEMORY_STRESS_ROUNDS").ok().map(|v|v.parse::<usize>().unwrap()).unwrap_or(1);assert!((1..=1024).contains(&rounds));
 for round in 0..rounds{
 let f=Files::new();let p=f.path();let own=owner("agent_antigravity");let created=mutate_at(&p,own.clone(),"missing",create("original"),100,||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();
 let calls=AtomicUsize::new(0);let denied=mutate_at(&p,own.clone(),&created.revision,create("revoked"),101,||if calls.fetch_add(1,Ordering::SeqCst)>=2{Err("ownerChanged")}else{Ok(())});assert_eq!(denied.err(),Some("ownerChanged"));assert_eq!(std::fs::read(&p).unwrap(),before);
 let barrier=Arc::new(Barrier::new(3));let mut workers=vec![];
 for text in ["first reviewed edit","second reviewed edit"]{let(p,own,revision,barrier)=(p.clone(),own.clone(),created.revision.clone(),barrier.clone());workers.push(std::thread::spawn(move||{barrier.wait();mutate_at(&p,own,&revision,create(text),102,||Ok(()))}));}
 barrier.wait();let results:Vec<_>=workers.into_iter().map(|w|w.join().unwrap()).collect();assert_eq!(results.iter().filter(|r|r.is_ok()).count(),1);let errors:Vec<_>=results.iter().filter_map(|r|r.as_ref().err()).collect();assert!(errors.iter().all(|e|["registryBusy","registryConflict"].contains(e)),"round {round}, losing writer errors: {errors:?}");assert_eq!(crate::private_registry::load_at::<model::Registry>(&p).unwrap().registry.items.len(),2);
 }
}
#[test]fn corrupt_linked_oversized_or_unsafe_memory_is_preserved_and_never_defaults_empty(){
 for kind in ["json","schema","unknownField","hardlink","symlink","permissions","oversize"]{
  let f=Files::new();let p=f.path();let own=owner("ai_lmstudio");let created=mutate_at(&p,own.clone(),"missing",create("original"),100,||Ok(())).unwrap();
  match kind{
   "json"=>std::fs::write(&p,b"not-json").unwrap(),
   "schema"|"unknownField"=>{let mut v:serde_json::Value=serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();if kind=="schema"{v["version"]=serde_json::json!(2);}else{v["items"][0]["permission"]=serde_json::json!("all");}std::fs::write(&p,serde_json::to_vec(&v).unwrap()).unwrap();},
   "hardlink"=>std::fs::hard_link(&p,f.root.join("alias")).unwrap(),
   "symlink"=>{std::fs::rename(&p,f.root.join("kept")).unwrap();symlink(f.root.join("kept"),&p).unwrap();},
   "permissions"=>std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o644)).unwrap(),
   _=>{let file=std::fs::OpenOptions::new().write(true).open(&p).unwrap();file.set_len(model::MAX_BYTES as u64+1).unwrap();},
  }
  let before=std::fs::read(&p).unwrap();assert!(list_at(&p,query(own.clone(),""),101,||Ok(())).is_err(),"{kind}");assert!(mutate_at(&p,own,&created.revision,create("replacement"),101,||Ok(())).is_err(),"{kind}");assert_eq!(std::fs::read(&p).unwrap(),before,"{kind}");
 }
}
#[test]fn bounded_pages_preserve_whole_records_and_reject_continuation_after_catalog_changes(){
 let f=Files::new();let p=f.path();let own=owner("agent_codex");let mut revision=String::from("missing");
 for _ in 0..12{revision=mutate_at(&p,own.clone(),&revision,create(&"é".repeat(4000)),100,||Ok(())).unwrap().revision;}
 let first=list_at(&p,query(own.clone(),""),101,||Ok(())).unwrap();assert!(first.has_more);assert!(first.items.len()<12);assert!(first.items.iter().all(|v|v.entry.content.len()==8000));
 let mut next=query(own.clone(),"");next.offset=first.next_offset.unwrap();assert_eq!(list_at(&p,next.clone(),101,||Ok(())).err(),Some("registryConflict"));next.expected_revision=Some(first.revision);let page=list_at(&p,next.clone(),101,||Ok(())).unwrap();assert_eq!(page.items.len()+first.items.len(),12);assert!(!page.has_more);
 mutate_at(&p,own,&revision,create("new revision"),102,||Ok(())).unwrap();assert_eq!(list_at(&p,next,103,||Ok(())).err(),Some("registryConflict"));
}
#[test]fn forged_sources_authority_project_scopes_and_unknown_native_commands_are_rejected(){
 let valid=serde_json::json!({"petId":"agent_codex","scope":{"kind":"pet"}});let decoded:Owner=serde_json::from_value(valid.clone()).unwrap();assert_eq!(serde_json::to_value(decoded).unwrap(),valid);
 for value in [serde_json::json!({"petId":"agent_codex","scope":{"kind":"project","path":"/tmp/project"}}),serde_json::json!({"petId":"agent_codex","scope":{"kind":"pet","permission":"all"}})]{assert!(serde_json::from_value::<Owner>(value.clone()).is_err(),"{value}");}
 let mut forged=serde_json::json!({"kind":"note","title":"title","content":"text","sensitivity":"normal","expiresAt":null});forged["provenance"]=serde_json::json!("verifiedTool");assert!(serde_json::from_value::<Input>(forged).is_err());
 for label in ["external","terminal","unknown",""]{assert!(!allowed(label));}for label in [crate::HUB_LABEL,crate::pet_chat::LABEL]{assert!(allowed(label));}
 let f=Files::new();let mut v=input("invalid expiry");v.expires_at=Some(100);assert_eq!(mutate_at(&f.path(),owner("agent_codex"),"missing",Mutation::Create{input:v},100,||Ok(())).err(),Some("invalidMemoryInput"));assert!(!f.path().exists());
}
#[test]fn explicit_memory_keeps_empty_conversation_identity_through_restart_and_new_conversation(){
 let f=Files::new();let library=f.root.join("conversations.json");let chat=crate::chat::Chat::persistent(library.clone());let config=crate::pet_config::defaults();
 let cid=chat.conversation("ollama").unwrap().id;assert!(!library.exists());let own=Owner{pet_id:"ai_ollama".into(),scope:Scope::Conversation{conversation_id:cid.clone()}};
 let receipt=mutate_prepared_at(&f.path(),own.clone(),"missing",create("explicit empty conversation note"),100,||validate_owner(&config,&chat,&own),||retain_owner_conversation(&chat,&own)).unwrap();assert!(library.exists());
 let reopened=crate::chat::Chat::persistent(library.clone());assert_eq!(reopened.conversation("ollama").unwrap().id,cid);assert!(reopened.read_conversation("ollama",&cid).unwrap().messages.is_empty());
 reopened.reset_provider("ollama").unwrap();assert_ne!(reopened.conversation("ollama").unwrap().id,cid);assert!(reopened.read_conversation("ollama",&cid).unwrap().messages.is_empty());
 let restarted=crate::chat::Chat::persistent(library);validate_owner(&config,&restarted,&own).unwrap();assert_eq!(list_at(&f.path(),query(own.clone(),""),101,||validate_owner(&config,&restarted,&own)).unwrap().items[0].entry.id,receipt.id);
}
#[test]fn reset_during_memory_review_keeps_the_pinned_original_scope_and_reset_before_pin_refuses(){
 for reset_after_pin in [true,false]{
  let f=Files::new();let chat=crate::chat::Chat::persistent(f.root.join("conversations.json"));let config=crate::pet_config::defaults();let cid=chat.conversation("ollama").unwrap().id;let own=Owner{pet_id:"ai_ollama".into(),scope:Scope::Conversation{conversation_id:cid.clone()}};
  let result=mutate_prepared_at(&f.path(),own.clone(),"missing",create("original explicit scope"),100,||validate_owner(&config,&chat,&own),||{
   if reset_after_pin{retain_owner_conversation(&chat,&own)?;chat.reset_provider("ollama").map_err(|_|"ownerChanged")?;}else{chat.reset_provider("ollama").map_err(|_|"ownerChanged")?;retain_owner_conversation(&chat,&own)?;}Ok(())
  });
  let current=chat.conversation("ollama").unwrap().id;assert_ne!(current,cid);
  if reset_after_pin{result.unwrap();assert!(chat.read_conversation("ollama",&cid).unwrap().messages.is_empty());assert_eq!(list_at(&f.path(),query(own.clone(),""),101,||validate_owner(&config,&chat,&own)).unwrap().items.len(),1);let fresh=Owner{pet_id:"ai_ollama".into(),scope:Scope::Conversation{conversation_id:current}};assert!(list_at(&f.path(),query(fresh,""),101,||Ok(())).unwrap().items.is_empty());}
  else{assert_eq!(result.err(),Some("conversationRetentionUnavailable"));assert!(!f.path().exists());}
 }
}
#[test]fn absent_memory_keeps_empty_reset_behavior_and_corrupt_catalog_conservatively_preserves_identity(){
 let f=Files::new();let library=f.root.join("conversations.json");let chat=crate::chat::Chat::persistent(library.clone());let cid=chat.conversation("ollama").unwrap().id;chat.reset_provider("ollama").unwrap();assert!(chat.read_conversation("ollama",&cid).is_err());assert!(!f.path().exists());
 let cid=chat.conversation("ollama").unwrap().id;std::fs::write(f.path(),b"unknown memory format retained").unwrap();std::fs::set_permissions(f.path(),std::fs::Permissions::from_mode(0o600)).unwrap();chat.reset_provider("ollama").unwrap();assert!(chat.read_conversation("ollama",&cid).unwrap().messages.is_empty());assert_eq!(std::fs::read(f.path()).unwrap(),b"unknown memory format retained");assert!(crate::chat::Chat::persistent(library).read_conversation("ollama",&cid).is_ok());
}
