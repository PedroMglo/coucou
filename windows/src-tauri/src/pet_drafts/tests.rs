use super::{model::*,store};
use crate::private_registry::{Document,load_at};
use std::{path::PathBuf,os::unix::fs::PermissionsExt,sync::atomic::{AtomicUsize,Ordering}};
// Frozen production model from865/F6D, unchanged bytes. Same registry engine.
#[allow(dead_code)]#[path="fixtures/reader_v1.rs"]mod reader_v1;
struct Files(PathBuf);
impl Files{fn new()->Self{let p=std::env::temp_dir().join(format!("coucou-drafts-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&p).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o700)).unwrap();Self(p)}fn path(&self)->PathBuf{self.0.join("session-drafts.json")}}
impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn utf(s:&str)->Vec<u16>{s.encode_utf16().collect()}
fn target(id:&str)->Target{Target{pet_id:utf("agent_codex"),provider:utf("codex"),origin:Origin::Mochi,session_id:utf(id),conversation_id:Some(utf(id))}}
fn saved(value:Write)->Record{match value{Write::Saved{record}=>record,_=>panic!("expected saved")}}
fn batch()->Import{Import{schema_version:1,source_id:"12345678-1234-1234-1234-123456789abc".into(),revision:"9007199254740993".into(),records:vec![Record{target:target("A"),revision:"9007199254740992".into(),text_units:vec![0xd800,0,0xdfff,0xd83d,0xdc08]},Record{target:target("B"),revision:"9007199254740993".into(),text_units:vec![]}]}}
#[test]fn passive_absence_then_private_restart_and_exact_utf16_roundtrip(){
 let f=Files::new();let p=f.path();assert!(store::read(&p,target("A"),||Ok(())).unwrap().is_none());assert_eq!(std::fs::read_dir(&f.0).unwrap().count(),0);
 let mut t=target("A");t.session_id=vec![0xd800,0xdfff];let units=vec![0xd800,0,0xdfff,0xd83d,0xdc08];
 let first=saved(store::write(&p,t.clone(),None,units.clone(),||Ok(())).unwrap());assert_eq!(first.revision,"1");
 assert_eq!(store::read(&p,t.clone(),||Ok(())).unwrap(),Some(first));assert!(store::read(&p,target("A"),||Ok(())).unwrap().is_none());
 assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode()&0o777,0o600);
 let raw=std::fs::read(&p).unwrap();let parsed:Registry=serde_json::from_slice(&raw).unwrap();assert_eq!(parsed.records[0].text_units,units);assert!(!String::from_utf8(raw).unwrap().contains("permission"));
}
#[test]fn exact_targets_cas_conflict_global_revision_and_empty_tombstone(){
 let f=Files::new();let p=f.path();let a=saved(store::write(&p,target("A"),None,utf("first"),||Ok(())).unwrap());
 let b=saved(store::write(&p,target("B"),None,utf("other"),||Ok(())).unwrap());assert_eq!(b.revision,"2");let before=std::fs::read(&p).unwrap();
 assert!(matches!(store::write(&p,target("A"),None,utf("stale"),||Ok(())).unwrap(),Write::Conflict{record:Some(r)}if r==a));assert_eq!(std::fs::read(&p).unwrap(),before);
 let empty=saved(store::write(&p,target("A"),Some("1".into()),vec![],||Ok(())).unwrap());assert_eq!(empty.revision,"3");assert_eq!(store::read(&p,target("A"),||Ok(())).unwrap(),Some(empty));
 let mut other=target("B");other.origin=Origin::External;assert!(store::read(&p,other,||Ok(())).unwrap().is_none());
}
#[test]fn import_preserves_all_targets_u64_revisions_and_tombstones_then_lost_ack_is_idempotent(){
 let f=Files::new();let p=f.path();let original=batch();let receipt=store::import(&p,original.clone(),||Ok(())).unwrap();
 for r in &original.records{assert_eq!(store::read(&p,r.target.clone(),||Ok(())).unwrap(),Some(r.clone()));}
 let changed=saved(store::write(&p,target("A"),Some(original.records[0].revision.clone()),utf("new native text"),||Ok(())).unwrap());assert_eq!(changed.revision,"9007199254740994");
 let before=std::fs::read(&p).unwrap();assert_eq!(store::import(&p,original.clone(),||Ok(())).unwrap(),receipt);assert_eq!(std::fs::read(&p).unwrap(),before);
 let mut reordered=original.clone();reordered.records.reverse();assert_eq!(store::import(&p,reordered,||Ok(())).unwrap(),receipt);assert_eq!(std::fs::read(&p).unwrap(),before);
 let mut crossed=original;crossed.records[0].text_units=utf("mutated original");assert_eq!(store::import(&p,crossed,||Ok(())).err(),Some("draftImportChanged"));assert_eq!(std::fs::read(&p).unwrap(),before);
}
#[test]fn conflicting_second_origin_import_is_all_or_nothing_and_empty_origin_can_adopt(){
 let f=Files::new();let p=f.path();store::write(&p,target("A"),None,utf("winner"),||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();
 let mut input=batch();input.records.reverse();assert_eq!(store::import(&p,input.clone(),||Ok(())).err(),Some("draftImportConflict"));assert_eq!(std::fs::read(&p).unwrap(),before);assert!(store::read(&p,target("B"),||Ok(())).unwrap().is_none());
 input.records.clear();input.revision="0".into();store::import(&p,input,||Ok(())).unwrap();assert_eq!(store::read(&p,target("A"),||Ok(())).unwrap().unwrap().text_units,utf("winner"));
}
#[test]fn maximum_imported_revision_is_not_reset_or_wrapped(){
 let f=Files::new();let p=f.path();let mut b=batch();b.revision=u64::MAX.to_string();b.records[1].revision=b.revision.clone();store::import(&p,b,||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();
 assert_eq!(store::write(&p,target("new"),None,utf("cannot wrap"),||Ok(())).err(),Some("draftLimit"));assert_eq!(std::fs::read(&p).unwrap(),before);
}
#[test]fn corrupt_unknown_version_or_additional_authority_field_refuses_without_repair(){
 for change in [serde_json::json!({"schemaVersion":4}),serde_json::json!({"permission":"allow"}),serde_json::json!({"revision":"01"})]{
  let f=Files::new();let p=f.path();store::write(&p,target("A"),None,utf("keep"),||Ok(())).unwrap();let mut raw:serde_json::Value=serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();for(k,v)in change.as_object().unwrap(){raw[k]=v.clone();}let bytes=serde_json::to_vec(&raw).unwrap();std::fs::write(&p,&bytes).unwrap();
  assert!(store::read(&p,target("A"),||Ok(())).is_err());assert!(store::write(&p,target("A"),Some("1".into()),utf("no"),||Ok(())).is_err());assert_eq!(std::fs::read(&p).unwrap(),bytes);
 }
}
#[test]fn invalid_targets_and_batch_limits_never_materialize_any_data(){
 let f=Files::new();let p=f.path();let mut invalid=target("A");invalid.pet_id=vec![0];assert_eq!(store::write(&p,invalid,None,vec![],||Ok(())).err(),Some("invalidDraft"));
 let mut duplicate=batch();duplicate.records.push(duplicate.records[0].clone());assert_eq!(store::import(&p,duplicate,||Ok(())).err(),Some("invalidDraft"));
 let mut too_large=batch();too_large.records[0].text_units=vec![42;TEXT+1];assert_eq!(store::import(&p,too_large,||Ok(())).err(),Some("draftLimit"));
 let mut malformed=batch();malformed.revision="1".into();assert_eq!(store::import(&p,malformed,||Ok(())).err(),Some("invalidDraft"));assert!(!p.exists());
}
#[test]fn owner_revocation_at_final_gate_preserves_previous_bytes_and_revision(){
 let f=Files::new();let p=f.path();store::write(&p,target("A"),None,utf("original"),||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();let calls=AtomicUsize::new(0);
 assert_eq!(store::write(&p,target("A"),Some("1".into()),utf("revoked"),||if calls.fetch_add(1,Ordering::SeqCst)>=2{Err("revoked")}else{Ok(())}).err(),Some("revoked"));assert_eq!(std::fs::read(&p).unwrap(),before);
 let native=load_at::<Registry>(&p).unwrap();assert_eq!(native.registry.generation,1);
}
#[test]fn maximum_total_utf16_capacity_is_enforced_without_truncation(){
 let mut state=Registry::empty();for n in 0..32{let(next,_)=state.write(target(&n.to_string()),None,vec![0xd800;TEXT]).unwrap();state=next;}assert_eq!(state.records.iter().map(|r|r.text_units.len()).sum::<usize>(),TOTAL);
 assert_eq!(state.write(target("overflow"),None,vec![42]).err(),Some("draftLimit"));assert_eq!(state.records.len(),32);
}
#[test]fn ipc_labels_exclude_provider_runtime_and_unrecognized_surfaces(){
 for label in [crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_chat::LABEL]{assert!(super::allowed(label).is_ok());}
 for label in ["providerModel","taskRuntime","agent_system_guardian","other"]{assert!(super::allowed(label).is_err());}
}
#[test]fn complete_existing_js_capacity_with_maximal_identifiers_imports_and_reopens(){
 let f=Files::new();let p=f.path();let mut input=batch();input.revision=RECORDS.to_string();input.records=(0..RECORDS).map(|n|{
  let mut t=Target{pet_id:vec![65535;128],provider:vec![65535;128],origin:Origin::Mochi,session_id:vec![65535;512],conversation_id:Some(vec![65535;512])};t.session_id[511]=(n+1)as u16;
  Record{target:t,revision:(n+1).to_string(),text_units:if n<32{vec![65535;TEXT]}else{vec![]}}
 }).collect();
 let last=input.records.last().unwrap().clone();let receipt=store::import(&p,input,||Ok(())).unwrap();assert_eq!(receipt.schema_version,1);
 let length=std::fs::metadata(&p).unwrap().len();assert!(length>32*1024*1024&&length<=Registry::MAX_BYTES as u64,"{length}");
 let reopened=load_at::<Registry>(&p).unwrap();assert_eq!(reopened.registry.records.len(),RECORDS);assert_eq!(reopened.registry.records.iter().map(|r|r.text_units.len()).sum::<usize>(),TOTAL);assert_eq!(reopened.registry.records.last(),Some(&last));
}
#[test]fn fresh_empty_conversation_draft_pins_original_cid_and_archives_it_after_restart_for_all_llm_pets(){
 let f=Files::new();let library=f.0.join("conversations.json");let p=f.path();
 for provider in ["ollama","lmstudio","codex","antigravity","copilot","integration_notion"]{
  let chat=crate::chat::Chat::persistent(library.clone());let cid=chat.conversation(provider).unwrap().id;let pet=crate::pet_config::integration_pet(provider).unwrap();let mut t=target(&cid);t.pet_id=utf(pet);t.provider=utf(provider);
  let saved=saved(store::write_prepared(&p,t.clone(),None,utf("original unsent draft"),||Ok(()),||super::retain(&chat,&t)).unwrap());assert_eq!(chat.conversation(provider).unwrap().messages.len(),0);drop(chat);
  let reopened=crate::chat::Chat::persistent(library.clone());assert_eq!(reopened.conversation(provider).unwrap().id,cid);assert!(super::retains_conversation(&library,provider,&cid));
  reopened.reset_provider(provider).unwrap();assert_ne!(reopened.conversation(provider).unwrap().id,cid);assert!(reopened.has_public_conversation(provider,&cid));drop(reopened);
  let again=crate::chat::Chat::persistent(library.clone());assert!(again.has_public_conversation(provider,&cid));assert_eq!(store::read(&p,t,||Ok(())).unwrap(),Some(saved));assert!(again.read_conversation(provider,&cid).unwrap().messages.is_empty());
 }
}
#[test]fn wrong_owner_absent_original_or_failed_retention_does_not_publish_draft_or_create_a_conversation(){
 let f=Files::new();let library=f.0.join("conversations.json");let p=f.path();let chat=crate::chat::Chat::persistent(library.clone());let cid=chat.conversation("ollama").unwrap().id;let mut t=target(&cid);t.provider=utf("ollama");
 assert_eq!(store::write_prepared(&p,t.clone(),None,utf("wrong pet"),||Ok(()),||super::retain(&chat,&t)).err(),Some("invalidDraft"));assert!(!p.exists());assert!(!library.exists());
 t.pet_id=utf("ai_ollama");t.session_id=utf("absent-original");t.conversation_id=Some(t.session_id.clone());assert_eq!(store::write_prepared(&p,t.clone(),None,vec![],||Ok(()),||super::retain(&chat,&t)).err(),Some("unavailable"));assert!(!p.exists());assert!(!chat.has_public_conversation("ollama","absent-original"));
 let mut crossed=t.clone();crossed.session_id=utf(&cid);assert_eq!(super::retain(&chat,&crossed),Err("invalidDraft"));assert_eq!(chat.conversation("ollama").unwrap().id,cid);
}
#[test]fn stale_draft_cas_never_calls_retention_and_unknown_registry_conservatively_retains(){
 let f=Files::new();let p=f.path();let library=f.0.join("conversations.json");assert!(!super::retains_conversation(&library,"codex","A"));
 store::write(&p,target("A"),None,vec![],||Ok(())).unwrap();assert!(super::retains_conversation(&library,"codex","A"));
 let calls=AtomicUsize::new(0);let before=std::fs::read(&p).unwrap();let r=store::write_prepared(&p,target("A"),None,utf("stale"),||Ok(()),||{calls.fetch_add(1,Ordering::SeqCst);Err("must not run")}).unwrap();assert!(matches!(r,Write::Conflict{..}));assert_eq!(calls.load(Ordering::SeqCst),0);assert_eq!(std::fs::read(&p).unwrap(),before);
 std::fs::write(&p,b"future or malformed catalog").unwrap();assert!(super::retains_conversation(&library,"codex","unrelated"));
}
fn unbound()->Target{Target{pet_id:utf("agent_codex"),provider:utf("codex"),origin:Origin::Unbound,session_id:utf("inline-composer"),conversation_id:None}}
#[test]fn old_v1_catalog_reads_unchanged_then_valid_mutation_upgrades_and_frozen_reader_refuses_without_writing(){
 let f=Files::new();let p=f.path();let old=reader_v1::Registry::empty();let t=serde_json::from_value(serde_json::to_value(target("A")).unwrap()).unwrap();let(next,_)=old.write(t,None,utf("original v1 text")).unwrap();crate::private_registry::save_checked(&p,"missing",next,|_,_|Ok(())).unwrap();
 let before=std::fs::read(&p).unwrap();let read=store::read(&p,target("A"),||Ok(())).unwrap().unwrap();assert_eq!(read.text_units,utf("original v1 text"));assert_eq!(std::fs::read(&p).unwrap(),before);assert_eq!(load_at::<Registry>(&p).unwrap().registry.schema_version,1);
 let mut invalid=unbound();invalid.conversation_id=Some(utf("forged-cid"));assert!(store::write(&p,invalid,None,utf("no upgrade"),||Ok(())).is_err());assert_eq!(std::fs::read(&p).unwrap(),before);
 store::write(&p,target("A"),Some("1".into()),utf("v2 ordinary write"),||Ok(())).unwrap();assert_eq!(load_at::<Registry>(&p).unwrap().registry.schema_version,2);let v2=std::fs::read(&p).unwrap();assert!(load_at::<reader_v1::Registry>(&p).is_err());assert_eq!(std::fs::read(&p).unwrap(),v2);
 store::write(&p,unbound(),None,vec![0xd800,0,0xdfff],||Ok(())).unwrap();let v2=std::fs::read(&p).unwrap();assert!(load_at::<reader_v1::Registry>(&p).is_err());assert_eq!(std::fs::read(&p).unwrap(),v2);
}
#[test]fn unbound_draft_uses_real_factory_owner_without_generating_a_conversation_or_authority(){
 let f=Files::new();let p=f.path();let library=f.0.join("conversations.json");let chat=crate::chat::Chat::persistent(library.clone());let t=unbound();
 store::write_prepared(&p,t.clone(),None,utf("unsent before CID"),||Ok(()),||super::retain(&chat,&t)).unwrap();assert!(!library.exists());assert!(!chat.has_public_conversation("codex","inline-composer"));assert_eq!(store::read(&p,t.clone(),||Ok(())).unwrap().unwrap().text_units,utf("unsent before CID"));
 let mut forged=t;forged.pet_id=utf("ai_ollama");assert_eq!(super::retain(&chat,&forged),Err("invalidDraft"));assert!(!super::retains_conversation(&library,"codex","inline-composer"));
 let invalid_runtime=serde_json::json!({"petId":"agent_codex","provider":"codex","origin":"unbound","sessionId":"inline-composer"});assert!(serde_json::from_value::<crate::session_chat::types::Target>(invalid_runtime).is_err());
}
#[test]fn legacy_import_cannot_mislabel_a_new_unbound_slot_as_an_original_indexeddb_record(){
 let f=Files::new();let p=f.path();let mut input=batch();input.records[0].target=unbound();assert_eq!(store::import(&p,input,||Ok(())).err(),Some("invalidDraftImport"));assert!(!p.exists());
}
#[test]fn concurrent_window_imports_wait_for_real_writer_contention_then_share_one_durable_receipt(){
 use std::{fs::OpenOptions,os::unix::fs::OpenOptionsExt,sync::Arc};
 let f=Files::new();let p=f.path();let lock=OpenOptions::new().read(true).write(true).create(true).mode(0o600).open(f.0.join(".session-drafts.lock")).unwrap();let guard=crate::private_registry::WriterLock::acquire(lock).unwrap();let count=Arc::new(AtomicUsize::new(0));
 let workers:Vec<_>=(0..2).map(|_|{let p=p.clone();let count=count.clone();std::thread::spawn(move||store::import(&p,batch(),||{count.fetch_add(1,Ordering::SeqCst);Ok(())}))}).collect();
 let until=std::time::Instant::now()+std::time::Duration::from_secs(1);while count.load(Ordering::SeqCst)<4&&std::time::Instant::now()<until{std::thread::sleep(std::time::Duration::from_millis(5));}assert!(count.load(Ordering::SeqCst)>=4);drop(guard);
 let results:Vec<_>=workers.into_iter().map(|t|t.join().unwrap().unwrap()).collect();assert_eq!(results[0],results[1]);assert_eq!(load_at::<Registry>(&p).unwrap().registry.generation,1);assert_eq!(load_at::<Registry>(&p).unwrap().registry.records.len(),2);
}
#[test]fn import_revocation_during_contention_and_uncertain_status_do_not_publish_or_retry_uncertainty(){
 use std::{fs::OpenOptions,os::unix::fs::OpenOptionsExt,sync::Arc};
 let f=Files::new();let p=f.path();let lock=OpenOptions::new().read(true).write(true).create(true).mode(0o600).open(f.0.join(".session-drafts.lock")).unwrap();let guard=crate::private_registry::WriterLock::acquire(lock).unwrap();let count=Arc::new(AtomicUsize::new(0));let calls=count.clone();
 assert_eq!(store::import(&p,batch(),||if calls.fetch_add(1,Ordering::SeqCst)>=2{Err("revoked")}else{Ok(())}).err(),Some("revoked"));drop(guard);assert!(!p.exists());
 let count=AtomicUsize::new(0);assert_eq!(store::import(&p,batch(),||{count.fetch_add(1,Ordering::SeqCst);Err("registryCommitUncertain")}).err(),Some("registryCommitUncertain"));assert_eq!(count.load(Ordering::SeqCst),1);assert!(!p.exists());
}
