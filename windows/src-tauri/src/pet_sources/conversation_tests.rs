use super::*;
use std::{path::PathBuf,os::unix::fs::PermissionsExt};

struct Files(PathBuf);
impl Files{
 fn new()->Self{let p=std::env::temp_dir().join(format!("coucou-calendar-retain-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&p).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o700)).unwrap();Self(p)}
 fn sources(&self)->PathBuf{self.0.join("sources.json")}
 fn library(&self)->PathBuf{self.0.join("conversations.json")}
}
impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn enrollment(provider:&str,cid:&str)->model::Registry{
 let mut book=model::tests::fixture();book.generation=0;book.sources[0].revision=0;
 book.sources[0].owners=vec![model::Owner::conversation(crate::pet_config::integration_pet(provider).unwrap(),cid)];book
}

#[test]fn empty_owner_is_durable_before_enrollment_across_factory_routes(){
 for provider in["codex","copilot","antigravity","ollama","lmstudio","openai","anthropic","google","integration_notion"]{
  let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let cid=chat.conversation(provider).unwrap().id;assert!(!f.library().exists());
  let mut book=enrollment(provider,&cid);book.version=2;let mut other=book.sources[0].clone();other.id="second-calendar".into();book.sources.push(other);
  let saved=save_with_conversations(&f.sources(),"missing",book,Consent::PersistentCalendarRead,&chat).unwrap();
  let reopened=crate::chat::Chat::persistent(f.library());assert_eq!(reopened.conversation(provider).unwrap().id,cid,"{provider}");assert!(reopened.read_conversation(provider,&cid).unwrap().messages.is_empty());assert_eq!(saved.registry.sources.len(),2);
  assert!(retains_conversation_at(&f.sources(),provider,&cid));assert!(!retains_conversation_at(&f.sources(),provider,"another-cid"));
  // The live pin also preserves the initially empty dialogue on explicit reset.
  chat.reset_provider(provider).unwrap();assert!(chat.read_conversation(provider,&cid).unwrap().messages.is_empty());
 }
}

#[test]fn consent_and_cas_are_checked_before_retaining_an_empty_dialogue(){
 let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let cid=chat.conversation("ollama").unwrap().id;let book=enrollment("ollama",&cid);
 assert_eq!(save_with_conversations(&f.sources(),"missing",book.clone(),Consent::MetadataOnly,&chat).err(),Some("consentRequired"));assert!(!f.library().exists()&&!f.sources().exists());
 let old=store::save_at(&f.sources(),"missing",book.clone()).unwrap();let bytes=std::fs::read(f.sources()).unwrap();let mut next=old.registry.clone();next.sources[0].path="/explicitly-not-opened/changed.ics".into();
 assert_eq!(save_with_conversations(&f.sources(),"missing",next,Consent::PersistentCalendarRead,&chat).err(),Some("registryConflict"));assert_eq!(std::fs::read(f.sources()).unwrap(),bytes);assert!(!f.library().exists());
}

#[test]fn wrong_owner_and_failed_library_write_cannot_publish_but_orphan_revocation_can(){
 let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let cid=chat.conversation("ollama").unwrap().id;
 let forged=enrollment("codex",&cid);assert_eq!(save_with_conversations(&f.sources(),"missing",forged.clone(),Consent::PersistentCalendarRead,&chat).err(),Some("conversationRetentionUnavailable"));assert!(!f.sources().exists());
 let blocked=f.0.join("not-a-directory");std::fs::write(&blocked,b"preserved").unwrap();let unavailable=crate::chat::Chat::persistent(blocked.join("conversations.json"));
 assert!(save_with_conversations(&f.sources(),"missing",enrollment("ollama",&cid),Consent::PersistentCalendarRead,&unavailable).is_err());assert!(!f.sources().exists());assert_eq!(std::fs::read(&blocked).unwrap(),b"preserved");
 // Old orphaned metadata remains removable; no creation, rebinding or replay.
 let old=store::save_at(&f.sources(),"missing",forged).unwrap();let mut next=old.registry.clone();next.sources[0].owners.clear();next.sources[0].enabled=false;
 let revoked=save_with_conversations(&f.sources(),&old.revision,next,Consent::MetadataOnly,&chat).unwrap();assert!(!revoked.registry.sources[0].enabled);assert!(!f.library().exists());
}

#[test]fn retention_probe_preserves_disabled_and_unknown_books_without_opening_ics(){
 let f=Files::new();assert!(!retains_conversation_at(&f.sources(),"ollama","fixture-cid"));assert!(!f.sources().exists());
 let mut book=enrollment("ollama","fixture-cid");book.sources[0].enabled=false;let saved=store::save_at(&f.sources(),"missing",book).unwrap();let bytes=std::fs::read(f.sources()).unwrap();
 assert!(retains_conversation_at(&f.sources(),"ollama","fixture-cid"));assert!(!retains_conversation_at(&f.sources(),"codex","fixture-cid"));assert_eq!(std::fs::read(f.sources()).unwrap(),bytes);assert!(saved.unchanged());
 std::fs::write(f.sources(),b"unsupported preserved metadata").unwrap();assert!(retains_conversation_at(&f.sources(),"ollama","different-cid"));assert_eq!(std::fs::read(f.sources()).unwrap(),b"unsupported preserved metadata");
}

#[test]fn pet_source_scope_enrollment_needs_own_consent_but_no_fabricated_chat_or_retention(){
 let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let mut book=enrollment("ollama","not-a-real-conversation");book.version=3;book.sources[0].owners=vec![model::Owner::pet("ai_ollama")];
 assert_eq!(save_with_conversations(&f.sources(),"missing",book.clone(),Consent::MetadataOnly,&chat).err(),Some("consentRequired"));assert!(!f.sources().exists());assert!(!f.library().exists());
 let saved=save_with_conversations(&f.sources(),"missing",book,Consent::PersistentCalendarRead,&chat).unwrap();assert_eq!(saved.registry.selected_pet("ai_ollama").unwrap().len(),1);assert!(saved.registry.selected_many("ai_ollama","not-a-real-conversation").unwrap().is_empty());assert!(!f.library().exists());assert!(!retains_conversation_at(&f.sources(),"ollama","not-a-real-conversation"));
}
