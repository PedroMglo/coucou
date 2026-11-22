//! Storage/operational separation proofs; these do not simulate human consent.
use super::*;
use crate::{settings::{self,EffectFixture,IoPhase},private_registry::preferences_file};
use std::{fs,sync::{Arc,Mutex}};
use serde_json::{json,Value};
fn hex_bytes(b:&[u8])->String{b.iter().map(|v|format!("{v:02x}")).collect()}
fn adopt_storage(f:&EffectFixture,bytes:&[u8]){
 let (source,original)=preferences_file::read(&f.path,false).unwrap();let (previous,old_io)=preferences_file::read(&f.root.join(FILE),false).unwrap();
 let id="11111111111111111111111111111111";
 let reference=history::retain(&f.path,history::Input{id,fingerprint:&"a".repeat(64),original_settings:original.as_deref(),original_io:old_io.as_deref(),approved:bytes,memory_hash:&"b".repeat(64),configuration_revision:"fixture-configuration",configuration_values:&json!({"fixture":true}),observed_phase:"PublicationUnconfirmed"},||Ok(())).unwrap();
 let prepared=review_prepare(&f.path,&previous,old_io.as_deref(),source.content_hash(),bytes,reference,||Ok(())).unwrap();
 let witness=source.commit_guarded(bytes,||if prepared.current(){Ok(())}else{Err(io::Error::other("changed"))}).unwrap();let outcome=prepared.finish_reviewed().unwrap();
 let mut next:settings::Settings=serde_json::from_slice(bytes).unwrap();settings::revision::accept_review(&mut next,witness,outcome);*f.current.lock().unwrap()=next;
}
#[test]fn review_foundation_adopting_data_keeps_every_optional_component_suspended_across_write_restart(){
 let f=EffectFixture::new();f.save();let mut next=f.current.lock().unwrap().clone();next.autostart=true;next.phone_link=true;let bytes=serde_json::to_vec(&next).unwrap();
 adopt_storage(&f,&bytes);assert_eq!(f.probe.status().phase,IoPhase::Ready);assert_eq!(f.probe.status().suspended_operations.len(),6);
 for op in OPERATIONS{assert!(f.probe.operation(op).is_err());}assert!(f.current.lock().unwrap().autostart);assert!(f.current.lock().unwrap().phone_link);
 f.save();let owner=Arc::new(Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&owner);assert_eq!(probe.status().phase,IoPhase::Ready);for op in OPERATIONS{assert!(probe.operation(op).is_err());}
 assert!(owner.lock().unwrap().autostart);assert!(owner.lock().unwrap().phone_link);
}
#[test]fn review_foundation_retains_exact_uncertain_record_and_creates_a_distinct_data_transaction(){
 let f=EffectFixture::new();assert!(settings::update_with(&f.current,|s|s.sound_volume=0.321,|s|settings::revision::uncertain_save_for_test(&f.path,s,||f.external()),|_|panic!("uncertain write announced")).is_err());
 let old=fs::read(f.root.join(FILE)).unwrap();let old_json:Value=serde_json::from_slice(&old).unwrap();assert_eq!(old_json["phase"],"prepared");
 let mut next=f.current.lock().unwrap().clone();next.sound_volume=0.19;adopt_storage(&f,&serde_json::to_vec(&next).unwrap());
 let current:Value=serde_json::from_slice(&fs::read(f.root.join(FILE)).unwrap()).unwrap();assert_ne!(current["transaction"],old_json["transaction"]);assert_eq!(current["phase"],"committed");assert_eq!(current["version"],2);assert_eq!(current["suspendedOperations"].as_array().unwrap().len(),6);
 let history:Value=serde_json::from_slice(&fs::read(f.root.join(history::DIRECTORY).join("review-11111111111111111111111111111111.json")).unwrap()).unwrap();assert_eq!(history["originalIo"],hex_bytes(&old));assert_eq!(history["scope"],"dataOnlyKeepComponentsSuspended");
}
#[test]fn review_foundation_missing_or_changed_retained_artifact_refuses_new_effects(){
 let f=EffectFixture::new();let bytes=serde_json::to_vec(&*f.current.lock().unwrap()).unwrap();adopt_storage(&f,&bytes);
 let path=f.root.join(history::DIRECTORY).join("review-11111111111111111111111111111111.json");let original=fs::read(&path).unwrap();fs::write(&path,&original).unwrap();assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);
 fs::remove_file(&path).unwrap();let owner=Arc::new(Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&owner);assert!(!probe.status().new_effects_allowed);
}
#[test]fn review_foundation_does_not_clear_suspensions_or_history_during_restore_rebinding(){
 let f=EffectFixture::new();let old=serde_json::to_vec(&*f.current.lock().unwrap()).unwrap();adopt_storage(&f,&old);let before=fs::read(f.root.join(FILE)).unwrap();let mut next:Value=serde_json::from_slice(&old).unwrap();next["soundVolume"]=json!(0.78);let new=serde_json::to_vec(&next).unwrap();let rebound:Value=serde_json::from_slice(&restored_record(&before,Some(&old),&new).unwrap()).unwrap();let prior:Value=serde_json::from_slice(&before).unwrap();assert_eq!(rebound["suspendedOperations"],prior["suspendedOperations"]);assert_eq!(rebound["reconciliation"],prior["reconciliation"]);assert_eq!(rebound["version"],2);assert_eq!(rebound["reviewRequired"],true);
 fs::write(&f.path,&new).unwrap();fs::write(f.root.join(FILE),serde_json::to_vec(&rebound).unwrap()).unwrap();let restored=settings::load_from(&f.path);assert_eq!(settings::io_status(&restored).phase,IoPhase::ReviewRequired);assert!(!settings::io_status(&restored).new_effects_allowed);
}
#[test]fn review_foundation_v2_rejects_unknown_duplicate_or_unbound_suspension_metadata(){
 let f=EffectFixture::new();let bytes=serde_json::to_vec(&*f.current.lock().unwrap()).unwrap();adopt_storage(&f,&bytes);let value:Value=serde_json::from_slice(&fs::read(f.root.join(FILE)).unwrap()).unwrap();
 for (field,replacement) in [("suspendedOperations",json!(["root"])),("suspendedOperations",json!(["voice","voice"])),("reconciliation",Value::Null),("version",json!(1))]{let mut changed=value.clone();changed[field]=replacement;let parsed:Record=serde_json::from_value(changed).unwrap();assert!(parsed.validate().is_err());}
}
#[test]fn review_foundation_guard_refusal_after_temporary_sync_preserves_destination_and_cleans_own_temp(){
 let f=EffectFixture::new();let(witness,before)=preferences_file::read(&f.path,false).unwrap();let error=witness.commit_guarded(br#"{"model":"should-not-be-published"}"#,||Err(io::Error::other("review revoked during IO"))).unwrap_err();assert!(error.to_string().contains("revoked"));assert!(!preferences_file::unconfirmed(&error));assert_eq!(fs::read(&f.path).unwrap(),before.unwrap());assert!(witness.current());assert!(!fs::read_dir(&f.root).unwrap().any(|p|p.unwrap().file_name().to_string_lossy().contains("coucou-")));
}
#[test]fn review_foundation_retention_limits_are_joint_and_unknown_nodes_are_never_overwritten(){
 let f=EffectFixture::new();let dir=f.root.join("retention-fixture");let first=dir.join("one.json");crate::private_registry::retain_new(&first,b"1234",2,8,||Ok(())).unwrap();crate::private_registry::retain_new(&dir.join("two.json"),b"5678",2,8,||Ok(())).unwrap();assert!(crate::private_registry::retain_new(&dir.join("three.json"),b"x",2,8,||Ok(())).is_err());assert_eq!(fs::read(&first).unwrap(),b"1234");assert!(crate::private_registry::retain_new(&first,b"x",3,9,||Ok(())).is_err());assert_eq!(fs::read(&first).unwrap(),b"1234");
}
