use super::*;
use crate::settings::{self,EffectFixture,IoPhase};
use std::{fs,sync::{Arc,Mutex,atomic::{AtomicUsize,Ordering}}};
use serde_json::{json,Value};
use std::os::unix::fs::FileTypeExt;
fn record(f:&EffectFixture)->Record{serde_json::from_slice(&fs::read(f.root.join(FILE)).unwrap()).unwrap()}
fn restart(f:&EffectFixture)->(Arc<Mutex<settings::Settings>>,settings::EffectProbe){let current=Arc::new(Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&current);(current,probe)}

#[test]fn settings_durable_success_binds_exact_committed_bytes_and_remains_ready_after_restart(){
 let f=EffectFixture::new();f.save();let r=record(&f);assert_eq!(r.phase,Phase::Committed);assert!(!r.review_required);assert_eq!(r.proposed,hash(&fs::read(&f.path).unwrap()));
 let(_owner,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::Ready);assert_eq!(fs::metadata(f.root.join(FILE)).unwrap().len()as usize,serde_json::to_vec_pretty(&r).unwrap().len());
 assert!(!String::from_utf8(fs::read(f.root.join(FILE)).unwrap()).unwrap().contains("explicit-fixture-model"));
}
#[test]fn settings_durable_salvage_review_survives_full_restart_and_further_saves(){
 let f=EffectFixture::new();let bad=br#"{"model":"explicit-recovered-fixture","soundVolume":"invalid"}"#;fs::write(&f.path,bad).unwrap();
 let(current,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::ReadUnavailable);
 settings::update_with(&current,|v|v.sound_volume=0.3,|v|settings::revision::save_to(&f.path,v),|_|{}).unwrap();
 assert_eq!(probe.status().phase,IoPhase::ReviewRequired);drop((current,probe));
 let(current,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::ReviewRequired);assert_eq!(current.lock().unwrap().model,"explicit-recovered-fixture");
 settings::update_with(&current,|v|v.sound_volume=0.4,|v|settings::revision::save_to(&f.path,v),|_|{}).unwrap();drop((current,probe));
 let(_owner,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::ReviewRequired);assert!(record(&f).review_required);
 assert_eq!(settings::io_status(&settings::read_initialized(&f.path).unwrap()).phase,IoPhase::ReviewRequired);
 assert!(fs::read_dir(&f.root).unwrap().filter_map(Result::ok).filter(|e|e.file_name().to_string_lossy().starts_with("settings.json.unread-")).any(|e|fs::read(e.path()).unwrap()==bad));
}
#[test]fn settings_durable_prepared_before_publication_cannot_resume_or_replay_after_restart(){
 let f=EffectFixture::new();let original=fs::read(&f.path).unwrap();let obs=Observation::read(&f.path).unwrap();let w=crate::private_registry::preferences_file::read(&f.path,false).unwrap().0;
 let _lease=writer(&f.path).unwrap();let _pending=obs.prepare(&f.path,w.content_hash(),b"different proposed bytes",false).unwrap();drop(_pending);drop(_lease);
 let(current,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::PublicationUnconfirmed);assert_eq!(record(&f).phase,Phase::Prepared);
 assert!(settings::update_with(&current,|v|v.autostart=true,|v|settings::revision::save_to(&f.path,v),|_|panic!("uncertain state published")).is_err());assert_eq!(fs::read(&f.path).unwrap(),original);
}
#[test]fn settings_durable_missing_outcome_after_real_rename_preserves_new_data_and_uncertainty(){
 let f=EffectFixture::new();let mut proposed=f.current.lock().unwrap().clone();proposed.sound_volume=0.66;let bytes=serde_json::to_vec_pretty(&proposed).unwrap();
 let w=crate::private_registry::preferences_file::read(&f.path,false).unwrap().0;let obs=Observation::read(&f.path).unwrap();let _lease=writer(&f.path).unwrap();let pending=obs.prepare(&f.path,w.content_hash(),&bytes,false).unwrap();w.commit(&bytes).unwrap();drop(pending);drop(_lease);
 let(current,probe)=restart(&f);assert_eq!(current.lock().unwrap().sound_volume,0.66);assert_eq!(probe.status().phase,IoPhase::PublicationUnconfirmed);assert_eq!(fs::read(&f.path).unwrap(),bytes);
}
#[test]fn settings_durable_directory_sync_failure_never_publishes_success_and_restart_stays_blocked(){
 let f=EffectFixture::new();let before=f.current.lock().unwrap().sound_volume;let count=AtomicUsize::new(0);
 assert!(settings::update_with(&f.current,|v|v.sound_volume=0.77,|v|settings::revision::sync_failure_for_test(&f.path,v),|_|{count.fetch_add(1,Ordering::Relaxed);}).is_err());
 assert_eq!(count.load(Ordering::Relaxed),0);assert_eq!(f.current.lock().unwrap().sound_volume,before);assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);assert_eq!(record(&f).phase,Phase::Prepared);
 let(current,probe)=restart(&f);assert_eq!(current.lock().unwrap().sound_volume,0.77);assert_eq!(probe.status().phase,IoPhase::PublicationUnconfirmed);
}
#[test]fn settings_durable_proven_prepublication_failure_preserves_foreign_temp_and_old_ready_values(){
 let f=EffectFixture::new();let original=fs::read(&f.path).unwrap();let temp=f.path.with_extension(format!("json.coucou-{}",std::process::id()));fs::write(&temp,b"foreign owned bytes").unwrap();
 assert!(settings::update_with(&f.current,|v|v.sound_volume=0.55,|v|settings::revision::save_to(&f.path,v),|_|panic!("failed commit announced")).is_err());
 assert_eq!(record(&f).phase,Phase::NotPublished);assert_eq!(f.probe.status().phase,IoPhase::Ready);assert_eq!(fs::read(&f.path).unwrap(),original);assert_eq!(fs::read(temp).unwrap(),b"foreign owned bytes");let(_owner,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::Ready);
}
#[test]fn settings_durable_external_data_change_cannot_be_silently_adopted_by_restart(){
 let f=EffectFixture::new();f.save();f.external();let changed=fs::read(&f.path).unwrap();let(_owner,probe)=restart(&f);assert_eq!(probe.status().phase,IoPhase::PublicationUnconfirmed);assert_eq!(fs::read(&f.path).unwrap(),changed);
}
#[test]fn settings_durable_unknown_or_malformed_record_is_conserved_without_default_authority(){
 for bytes in [b"not json".as_slice(),b"{\"version\":99}".as_slice(),b"[]".as_slice()]{let f=EffectFixture::new();fs::write(f.root.join(FILE),bytes).unwrap();use std::os::unix::fs::PermissionsExt;fs::set_permissions(f.root.join(FILE),fs::Permissions::from_mode(0o600)).unwrap();let(current,probe)=restart(&f);assert_eq!(current.lock().unwrap().model,"explicit-fixture-model");assert_eq!(probe.status().phase,IoPhase::PublicationUnconfirmed);assert!(settings::update_with(&current,|v|v.sound_volume=0.11,|v|settings::revision::save_to(&f.path,v),|_|panic!("invalid IO record adopted")).is_err());assert_eq!(fs::read(f.root.join(FILE)).unwrap(),bytes);}
}
#[test]fn settings_durable_replaced_journal_blocks_loaded_owner_even_when_content_is_identical(){
 let f=EffectFixture::new();f.save();let path=f.root.join(FILE);let bytes=fs::read(&path).unwrap();let replacement=f.root.join("replacement");fs::write(&replacement,&bytes).unwrap();use std::os::unix::fs::PermissionsExt;fs::set_permissions(&replacement,fs::Permissions::from_mode(0o600)).unwrap();fs::rename(replacement,&path).unwrap();assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);assert_eq!(fs::read(path).unwrap(),bytes);
}
#[test]fn settings_durable_operation_lock_rejects_fifo_and_retains_exclusive_writer(){
 let f=EffectFixture::new();let lock=writer(&f.path).unwrap();assert!(writer(&f.path).is_err());drop(lock);assert!(writer(&f.path).is_ok());
 let path=f.root.join(".settings-io-commit.lock");fs::remove_file(&path).unwrap();use std::os::unix::ffi::OsStrExt;let name=std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();assert_eq!(unsafe{libc::mkfifo(name.as_ptr(),0o600)},0);assert!(writer(&f.path).is_err());assert!(fs::symlink_metadata(path).unwrap().file_type().is_fifo());
}
#[test]fn settings_durable_reviewed_restore_rebinds_only_stable_data_and_keeps_unresolved_references(){
 let f=EffectFixture::new();f.save();let old=fs::read(&f.path).unwrap();let r=record(&f);let original=serde_json::to_vec(&r).unwrap();let proposed=b"new reviewed canonical settings";
 let stable:Record=serde_json::from_slice(&restored_record(&original,Some(&old),proposed).unwrap()).unwrap();assert_eq!(stable.phase,Phase::Committed);assert_eq!(stable.proposed,hash(proposed));assert_ne!(stable.transaction,r.transaction);assert!(stable.review_required);assert_eq!(stable.version,r.version);
 for phase in [Phase::Prepared,Phase::Committed]{let mut input=r.clone();input.phase=phase;input.review_required=true;let source=serde_json::to_vec(&input).unwrap();let restored=restored_record(&source,Some(&old),proposed).unwrap();let value:Record=serde_json::from_slice(&restored).unwrap();assert!(value.review_required);if phase==Phase::Prepared{assert_eq!(restored,source);}}
 let mut wrong:Value=serde_json::from_slice(&original).unwrap();wrong["version"]=json!(99);assert!(restored_record(&serde_json::to_vec(&wrong).unwrap(),Some(&old),proposed).is_err());
}
#[test]fn settings_durable_restore_suspends_all_components_even_when_only_shortcuts_had_a_receipt(){
 let f=EffectFixture::new();f.save();let old=fs::read(&f.path).unwrap();
 for phase in ["verified","unconfirmed"]{
  let mut input=record(&f);input.version=7;input.suspended_operations=vec!["shortcuts".into()];
  let slot:components::Slot=serde_json::from_value(json!({"id":"a".repeat(32),"component":"shortcuts","phase":phase,"binding":{"profile":"b".repeat(64),"configuration":"c".repeat(64),"resource":"d".repeat(64),"plan":"e".repeat(64)},"owner":"f".repeat(64),"intent":{"id":"1".repeat(32),"sha256":"2".repeat(64)},"outcome":{"id":"3".repeat(32),"sha256":"4".repeat(64)}})).unwrap();input.components.insert("shortcuts".into(),slot);input.validate().unwrap();
  let original=serde_json::to_vec(&input).unwrap();let restored:Record=serde_json::from_slice(&restored_record(&original,Some(&old),b"reviewed destination data").unwrap()).unwrap();restored.validate().unwrap();assert_eq!(restored.version,7);assert_eq!(restored.components,input.components);assert_eq!(restored.suspended_operations.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>(),OPERATIONS.into_iter().collect());assert_eq!(restored.review_required,input.review_required);
  // Another copy never consumes a native revalidation prerequisite.
  let next:Record=serde_json::from_slice(&restored_record(&serde_json::to_vec(&restored).unwrap(),Some(b"reviewed destination data"),b"another reviewed destination").unwrap()).unwrap();assert_eq!(next.components,input.components);assert_eq!(next.suspended_operations,restored.suspended_operations);
  input.phase=Phase::Prepared;let pending=serde_json::to_vec(&input).unwrap();assert_eq!(restored_record(&pending,Some(&old),b"new data").unwrap(),pending);
 }
}
#[test]fn settings_durable_legacy_restore_keeps_format_and_requires_existing_global_review(){
 let f=EffectFixture::new();f.save();let old=fs::read(&f.path).unwrap();
 for version in[1,2]{let mut input=record(&f);input.version=version;if version==2{input.reconciliation=Some(ReconciliationRef{id:"a".repeat(32),sha256:"b".repeat(64)});}input.validate().unwrap();let restored:Record=serde_json::from_slice(&restored_record(&serde_json::to_vec(&input).unwrap(),Some(&old),b"new reviewed data").unwrap()).unwrap();restored.validate().unwrap();assert_eq!(restored.version,version);assert_eq!(restored.components,input.components);assert_eq!(restored.reconciliation,input.reconciliation);assert!(restored.review_required);}
}
