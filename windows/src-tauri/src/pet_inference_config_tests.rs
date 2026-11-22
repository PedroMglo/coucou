//! Private profile fixtures. A Frame fixture proves invalidation mechanics,
//! never human acceptance or native window mapping.
use super::*;
use crate::{pet_inference::{Backend,Choice,CredentialReference,Captured},settings::Settings};
use std::sync::Arc;
struct Fixture{root:PathBuf,file:PathBuf}
impl Fixture{
 fn new()->Self{
  let root=std::env::temp_dir().join(format!("coucou-binding-edit-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  std::fs::create_dir(&root).unwrap();use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();
  let file=root.join("pets.json");write(&file,&defaults()).unwrap();Self{root,file}
 }
 fn doc(&self)->Document{read_at_policy(&self.file,false).unwrap()}
 fn change(&self,pet:&str,update:Update)->Change{Change{pet_id:pet.into(),expected_revision:self.doc().revision,update}}
 fn apply(&self,pet:&str,update:Update)->Document{set_at(&self.file,&self.change(pet,update),||true).unwrap()}
 fn capture(&self,settings:&Arc<Mutex<Settings>>,provider:&str)->Arc<Captured>{Captured::capture_fixture(settings,provider,&self.file).unwrap().unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){std::fs::remove_dir_all(&self.root).unwrap();}}
fn selected(model:&str)->Update{Update::Set{choice:Choice{schema_version:1,backend:Backend::Ollama,endpoint:"http://127.0.0.1:18471".into(),model:model.into(),credential:CredentialReference::None}}}
#[test]fn pet_inference_native_revision_reapply_and_legacy_save_preserve_exact_owner_and_policy(){
 let f=Fixture::new();let original=f.doc();let settings=Arc::new(Mutex::new(Settings::default()));let first=f.apply("ai_ollama",selected("synthetic"));
 assert_eq!(first.config.version,2);let first_revision=first.config.pets["ai_ollama"].inference_revision.clone().unwrap();assert!(crate::pet_inference::revision_key(&first_revision));
 let old=f.capture(&settings,"ollama");assert!(old.current());let second=f.apply("ai_ollama",selected("synthetic"));assert_ne!(second.config.pets["ai_ollama"].inference_revision.as_ref(),Some(&first_revision));assert!(!old.current());
 let mut legacy=second.config.clone();legacy.version=1;for p in legacy.pets.values_mut(){p.inference=None;p.inference_revision=None;}
 let saved=save_at(&f.file,&legacy,&second.revision).unwrap();assert_eq!(saved.config.version,2);assert_eq!(saved.config.pets["ai_ollama"].inference_revision,second.config.pets["ai_ollama"].inference_revision);
 for(id,p)in &original.config.pets{
  let now=&saved.config.pets[id];assert_eq!((&now.name,&now.aliases,&now.provider,&now.permissions,&now.local_execution),(&p.name,&p.aliases,&p.provider,&p.permissions,&p.local_execution));
  if id!="ai_ollama"{assert_eq!(serde_json::to_value(now).unwrap(),serde_json::to_value(p).unwrap());}
 }
}
#[test]fn pet_inference_native_clear_rebind_and_legacy_aba_do_not_revive_unobserved_captures(){
 let f=Fixture::new();let mut selected_settings=Settings::default();selected_settings.ollama_url="http://127.0.0.1:18471".into();let settings=Arc::new(Mutex::new(selected_settings));
 f.apply("ai_ollama",selected("synthetic"));let bound=f.capture(&settings,"ollama");
 let cleared=f.apply("ai_ollama",Update::Clear{});assert_eq!(cleared.config.version,2);assert!(cleared.config.pets["ai_ollama"].inference.is_none());let clear_revision=cleared.config.pets["ai_ollama"].inference_revision.clone().unwrap();
 let legacy=f.capture(&settings,"ollama");f.apply("ai_ollama",selected("synthetic"));assert!(!bound.current(),"A -> clear -> A without an intermediate observer");
 let cleared_again=f.apply("ai_ollama",Update::Clear{});assert_ne!(cleared_again.config.pets["ai_ollama"].inference_revision.as_ref(),Some(&clear_revision));assert!(!legacy.current(),"legacy -> explicit -> legacy without an intermediate observer");
 assert!(f.capture(&settings,"ollama").current());
}
#[test]fn pet_inference_native_observed_revocation_is_sticky_even_after_exact_private_file_restore(){
 let f=Fixture::new();let settings=Arc::new(Mutex::new(Settings::default()));f.apply("ai_ollama",selected("first"));let bytes=std::fs::read(&f.file).unwrap();let captured=f.capture(&settings,"ollama");
 f.apply("ai_ollama",selected("second"));assert!(!captured.current());std::fs::write(&f.file,&bytes).unwrap();assert!(!captured.current());assert!(f.capture(&settings,"ollama").current());
}
#[test]fn pet_inference_native_stale_origin_and_withdrawal_during_write_preserve_original_bytes(){
 let f=Fixture::new();let bytes=std::fs::read(&f.file).unwrap();let request=f.change("ai_ollama",selected("synthetic"));
 assert!(set_at(&f.file,&request,||false).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),bytes);
 let frame=crate::settings::review::Frame::fixture();
 let result=set_at(&f.file,&request,||{
  if std::fs::read_dir(&f.root).unwrap().any(|p|p.unwrap().path().extension().is_some_and(|e|e=="tmp")){frame.invalidate_for_test();}
  frame.current()
 });assert!(result.is_err());assert_eq!(std::fs::read(&f.file).unwrap(),bytes);assert_eq!(std::fs::read_dir(&f.root).unwrap().count(),1);
 f.apply("ai_ollama",selected("synthetic"));let after=std::fs::read(&f.file).unwrap();assert!(set_at(&f.file,&request,||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),after);
}
#[test]fn pet_inference_native_malformed_or_unsupported_changes_never_write_or_default_to_clear(){
 let f=Fixture::new();let bytes=std::fs::read(&f.file).unwrap();assert!(set_at(&f.file,&f.change("agent_codex",selected("synthetic")),||true).is_err());
 let mut request=f.change("ai_ollama",selected("synthetic"));if let Update::Set{choice}=&mut request.update{choice.endpoint="https://example.invalid".into();}assert!(set_at(&f.file,&request,||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),bytes);
 for value in [serde_json::json!({"petId":"ai_ollama","expectedRevision":"x"}),serde_json::json!({"petId":"ai_ollama","expectedRevision":"x","update":{"kind":"set"}}),serde_json::json!({"petId":"ai_ollama","expectedRevision":"x","update":{"kind":"clear","choice":{}}}),serde_json::json!({"petId":"ai_ollama","expectedRevision":"x","update":{"kind":"clear"},"inferenceRevision":"forged"})]{assert!(serde_json::from_value::<Change>(value).is_err());}
 for label in ["island","pet-chat","untrusted"]{assert!(!allowed(label));}
 let mut invalid=defaults();invalid.version=2;invalid.pets.get_mut("ai_ollama").unwrap().inference_revision=Some("forged".into());assert!(validate(&invalid).is_err());
}
#[test]fn pet_inference_external_edit_during_native_write_is_preserved_without_false_ack(){
 let f=Fixture::new();let request=f.change("ai_ollama",selected("synthetic"));let mut external=defaults();external.pets.get_mut("agent_codex").unwrap().name="External edit fixture".into();let external=serde_json::to_vec(&external).unwrap();
 let changed=std::sync::atomic::AtomicBool::new(false);
 let result=set_at(&f.file,&request,||{
  if std::fs::read_dir(&f.root).unwrap().any(|p|p.unwrap().path().extension().is_some_and(|e|e=="tmp"))&&!changed.swap(true,std::sync::atomic::Ordering::AcqRel){std::fs::write(&f.file,&external).unwrap();}true
 });assert!(result.is_err());assert!(changed.load(std::sync::atomic::Ordering::Acquire));assert_eq!(std::fs::read(&f.file).unwrap(),external);assert_eq!(std::fs::read_dir(&f.root).unwrap().count(),1);assert!(f.doc().config.pets["ai_ollama"].inference.is_none());
}
