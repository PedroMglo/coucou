//! Same real inactive restore fixture as independent preparation, no models.
use super::*;
use crate::pet_data::publication::restore::retention::{RetentionReview,ROOT};
use std::os::unix::{ffi::OsStringExt,fs::MetadataExt};
use std::collections::BTreeSet;

fn review(f:&Fixture)->Result<RetentionReview>{let layout=f.target.layout();RetentionReview::capture(Review::capture(f.owner.claim(&layout)?,layout,&f.workspace,&f.id,Mode::RetainIndependent)?)}
fn job(f:&Fixture)->PathBuf{f.config().join(JOB)}
fn opaque(f:&Fixture){fs::DirBuilder::new().mode(0o700).create(job(f)).unwrap();f.target.file(&job(f).join("partial.json"),b"{private opaque unknown accounting; not a policy");}
fn retained(f:&Fixture,v:&Value)->PathBuf{f.config().join(ROOT).join(v["record"].as_str().unwrap()).join("original")}
fn apply(f:&Fixture)->Value{let r=review(f).unwrap();let v=r.preview().unwrap();r.apply(v["fingerprint"].as_str().unwrap()).unwrap()}
fn pair(f:&Fixture)->[Vec<(PathBuf,Vec<u8>)>;2]{[tree(&f.config()),tree(f.target.layout().data())]}

#[test]fn retained_future_preparation_cuts_never_renew_generation_or_use_the_retention_approval(){
 for cut in ["intent","generation","key","base","history","ready"]{
  let f=Fixture::new();opaque(&f);let retained_receipt=apply(&f);let retained_before=tree(&f.config().join(ROOT));let r=f.review().unwrap();let old_fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply(retained_receipt["receiptHash"].as_str().unwrap()).is_err());assert!(!job(&f).exists());
  let r=f.review().unwrap();assert!(r.apply_using(&old_fp,|s|if s==cut{Err(Error::Unavailable)}else{Ok(())}).is_err());let key=fs::read(f.seed().join(".key.json")).ok();let r=f.review().unwrap();let v=r.preview().unwrap();assert_eq!(v["schemaVersion"],2);assert_eq!(v["retention"]["records"],1);assert_ne!(v["fingerprint"],old_fp);let before=pair(&f);assert!(r.apply(&old_fp).is_err());assert_eq!(pair(&f),before);
  let r=f.review().unwrap();let v=r.preview().unwrap();let result=r.apply(v["fingerprint"].as_str().unwrap());if cut=="generation"{assert!(result.is_err());assert!(!f.seed().join(".key.json").exists());assert!(!job(&f).join("ready.json").exists());}else{result.unwrap();if let Some(k)=key{assert_eq!(fs::read(f.seed().join(".key.json")).unwrap(),k);}}assert_eq!(tree(&f.config().join(ROOT)),retained_before);
 }
}

#[test]fn retained_future_intent_downgrade_or_binding_change_is_preserved_and_refused(){
 for action in ["version","purpose","head","bindingAbsent","keyRemoved"]{
  let f=Fixture::new();opaque(&f);apply(&f);f.apply();let path=job(&f).join("manifest.json");let mut m:Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();match action{"version"=>m["version"]=json!(1),"purpose"=>m["purpose"]=json!("independentFutureAuthorityPreparation"),"head"=>m["retention"]["head"]=json!("0".repeat(64)),"bindingAbsent"=>{m.as_object_mut().unwrap().remove("retention");},_=>{fs::remove_file(f.seed().join(".key.json")).unwrap();}}if action!="keyRemoved"{fs::write(&path,serde_json::to_vec(&m).unwrap()).unwrap();}let before=pair(&f);assert!(f.review().is_err());assert_eq!(pair(&f),before);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
 }
}

#[test]fn native_retention_preserves_opaque_original_inode_and_bytes_without_authority_or_new_preparation(){
 for kind in ["empty","opaque","nonUtf8","uncertain","invalidKey","validPrepared"]{
  let f=Fixture::new();
  match kind{
   "empty"=>{fs::DirBuilder::new().mode(0o700).create(job(&f)).unwrap();},
   "uncertain"|"invalidKey"|"validPrepared"=>{f.apply();if kind!="validPrepared"{fs::remove_file(job(&f).join("ready.json")).unwrap();let key=f.seed().join(".key.json");if kind=="uncertain"{fs::remove_file(key).unwrap();}else{fs::write(key,b"{").unwrap();}}},
   _=>{opaque(&f);if kind=="nonUtf8"{f.target.file(&job(&f).join(std::ffi::OsString::from_vec(vec![0xff,b'x'])),b"retained raw filename");}},
  }
  let old=tree(&job(&f));let ino=fs::metadata(job(&f)).unwrap().ino();let payload=f.payload_bytes();let source=tree(f.source.layout().config());let before=pair(&f);
  let r=review(&f).unwrap();let v=r.preview().unwrap();assert_eq!(v["keyGenerationAvailable"],false);assert_eq!(v["retainedBytesTrustedAsAuthority"],false);assert!(pair(&f)==before);
  let receipt=r.apply(v["fingerprint"].as_str().unwrap()).unwrap();let original=retained(&f,&receipt);assert!(!job(&f).exists());assert!(tree(&original)==old);assert_eq!(fs::metadata(&original).unwrap().ino(),ino);
  assert_eq!(receipt["futurePreparationBlocked"],false);assert_eq!(receipt["futurePreparationRequiresSeparateReview"],true);assert_eq!(receipt["executionAuthorityGranted"],false);assert_eq!(receipt["runtimeActivationAvailable"],false);
  assert!(f.payload_bytes()==payload);assert!(tree(f.source.layout().config())==source);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());let r=f.review().unwrap();assert_eq!(r.preview().unwrap()["schemaVersion"],2);assert!(r.apply(v["fingerprint"].as_str().unwrap()).is_err());assert!(!job(&f).exists());
  let before=pair(&f);assert_eq!(apply(&f),receipt);assert!(pair(&f)==before);
 }
}

#[test]fn every_retention_boundary_needs_fresh_review_and_resumes_the_same_physical_original(){
 for cut in ["containerCreated","protocol","recordCreated","intent","moved","ready"]{
  let f=Fixture::new();opaque(&f);let old=tree(&job(&f));let ino=fs::metadata(job(&f)).unwrap().ino();let r=review(&f).unwrap();let v=r.preview().unwrap();let old_fp=v["fingerprint"].as_str().unwrap().to_owned();
  assert!(r.fixture_interrupt(&old_fp,cut).is_err(),"{cut}");let before=pair(&f);let r=review(&f).unwrap();let fresh=r.preview().unwrap();assert_ne!(fresh["fingerprint"],old_fp);
  assert!(r.apply(&old_fp).is_err());assert!(pair(&f)==before);let receipt=apply(&f);let original=retained(&f,&receipt);assert!(tree(&original)==old);assert_eq!(fs::metadata(original).unwrap().ino(),ino);assert!(!job(&f).exists());
  let before=pair(&f);assert_eq!(apply(&f),receipt);assert!(pair(&f)==before);
 }
}

#[test]fn exact_metadata_prefixes_can_resume_but_unknown_content_is_not_adopted(){
 for cut in ["protocol","intent","ready"]{
  let f=Fixture::new();opaque(&f);let r=review(&f).unwrap();let v=r.preview().unwrap();assert!(r.fixture_interrupt(v["fingerprint"].as_str().unwrap(),cut).is_err());
  let path=if cut=="protocol"{f.config().join(ROOT).join("protocol.json")}else{f.config().join(ROOT).join(v["record"].as_str().unwrap()).join(if cut=="intent"{"manifest.json"}else{"ready.json"})};
  let bytes=fs::read(&path).unwrap();fs::write(&path,&bytes[..bytes.len()/2]).unwrap();let before=pair(&f);let r=review(&f).unwrap();let next=r.preview().unwrap();assert!(pair(&f)==before);r.apply(next["fingerprint"].as_str().unwrap()).unwrap();assert_eq!(fs::read(&path).unwrap(),bytes);
 }
 let f=Fixture::new();opaque(&f);let r=review(&f).unwrap();let v=r.preview().unwrap();assert!(r.fixture_interrupt(v["fingerprint"].as_str().unwrap(),"intent").is_err());
 let path=f.config().join(ROOT).join(v["record"].as_str().unwrap()).join("manifest.json");fs::write(&path,b"unattributed content").unwrap();let before=pair(&f);assert!(review(&f).is_err());assert!(pair(&f)==before&&job(&f).exists());
}

#[test]fn wrong_cross_profile_or_changed_source_payload_and_owner_refuse_before_retention_effects(){
 for action in ["wrong","cross","owner","payload","source"]{
  let f=Fixture::new();opaque(&f);let r=review(&f).unwrap();let mut fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();
  match action{"wrong"=>fp="0".repeat(64),"cross"=>{let other=Fixture::new();opaque(&other);fp=review(&other).unwrap().preview().unwrap()["fingerprint"].as_str().unwrap().into();},"owner"=>f.owner.release(),"payload"=>fs::write(f.config().join(PAYLOAD).join("settings.json"),b"{}").unwrap(),_=>fs::write(f.workspace.join(format!("current-profile-{}/config/settings.json",f.id)),b"{}").unwrap()};
  let before=pair(&f);assert!(r.apply(&fp).is_err(),"{action}");assert!(pair(&f)==before);assert!(!f.config().join(ROOT).exists());
 }
}

#[test]fn owner_loss_during_io_never_moves_the_original_after_its_intent(){
 let f=Fixture::new();opaque(&f);let old=tree(&job(&f));let r=review(&f).unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();
 let result=r.apply_using_for_test(&fp,||{f.owner.release();});
 assert!(result.is_err());assert!(job(&f).exists()&&tree(&job(&f))==old);let root=f.config().join(ROOT);assert!(root.exists());assert!(!root.join("ready.json").exists());
}

#[test]fn unsafe_objects_and_oversized_or_deep_jobs_are_preserved_and_refused_before_namespace_creation(){
 for kind in ["symlink","hardlink","public","fifo","large","deep"]{
  let f=Fixture::new();opaque(&f);let path=job(&f).join("unsafe");
  match kind{
   "symlink"=>{let outside=f.target.0.join("outside");f.target.file(&outside,b"outside sentinel");symlink(outside,&path).unwrap();},
   "hardlink"=>fs::hard_link(job(&f).join("partial.json"),&path).unwrap(),
   "public"=>{f.target.file(&path,b"public");fs::set_permissions(&path,fs::Permissions::from_mode(0o644)).unwrap();},
   "fifo"=>{use std::os::unix::ffi::OsStrExt;let name=std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();assert_eq!(unsafe{libc::mkfifo(name.as_ptr(),0o600)},0);},
   "large"=>{let file=fs::File::create(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();file.set_len(8*1024*1024+1).unwrap();},
   _=>{let mut p=job(&f);for _ in 0..17{p=p.join("d");fs::DirBuilder::new().mode(0o700).create(&p).unwrap();}},
  }assert!(review(&f).is_err(),"{kind}");assert!(job(&f).exists()&&!f.config().join(ROOT).exists());
 }
}

#[test]fn bounded_chain_retains_each_original_once_and_refuses_a_ninth_without_touching_it(){
 let f=Fixture::new();let mut heads=BTreeSet::new();let mut inodes=BTreeSet::new();
 for index in 0..8{opaque(&f);f.target.file(&job(&f).join("attempt.txt"),index.to_string().as_bytes());let ino=fs::metadata(job(&f)).unwrap().ino();let receipt=apply(&f);assert!(heads.insert(receipt["receiptHash"].as_str().unwrap().to_owned()));assert!(inodes.insert(ino));assert_eq!(fs::metadata(retained(&f,&receipt)).unwrap().ino(),ino);}
 opaque(&f);let before=pair(&f);assert!(matches!(review(&f),Err(Error::Limit)));assert!(pair(&f)==before&&job(&f).exists());
}

#[test]fn retained_tamper_and_a_duplicate_current_job_never_publish_another_receipt(){
 for action in ["data","manifest","duplicate"]{
  let f=Fixture::new();opaque(&f);let receipt=apply(&f);let original=retained(&f,&receipt);
  match action{"data"=>fs::write(original.join("partial.json"),b"changed retained evidence").unwrap(),"manifest"=>fs::write(original.parent().unwrap().join("manifest.json"),b"{}").unwrap(),_=>{let r=review(&f).unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();opaque(&f);let before=pair(&f);assert!(r.apply(&fp).is_err());assert!(pair(&f)==before);continue;}};
  let before=pair(&f);assert!(review(&f).is_err());assert!(pair(&f)==before);
 }
}

#[test]fn native_cli_exact_form_retains_and_acknowledges_without_falling_into_preparation(){
 let f=Fixture::new();opaque(&f);f.owner.release();let args=|approve:Option<&str>|{let mut args=vec!["coucou".into(),"--pets-profile-restore-retain-preparation".into(),f.workspace.to_string_lossy().into_owned(),f.id.clone()];if let Some(hash)=approve{args.extend(["--approve".into(),hash.into()]);}restore_cli::parse(&args).unwrap().unwrap()};
 let preview=restore_cli::execute(args(None),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();let receipt=restore_cli::execute(args(preview["fingerprint"].as_str()),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();assert_eq!(receipt["status"],"independentPreparationRetained");let before=pair(&f);let preview=restore_cli::execute(args(None),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();assert_eq!(restore_cli::execute(args(preview["fingerprint"].as_str()),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap(),receipt);assert!(pair(&f)==before);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}


#[test]fn normal_readers_refuse_every_pending_retention_phase_with_an_otherwise_valid_preparation(){
 for cut in ["containerCreated","protocol","recordCreated","intent","moved"]{
  let f=Fixture::new();f.apply();assert!(f.data().is_ok());
  let r=review(&f).unwrap();let v=r.preview().unwrap();assert!(r.fixture_interrupt(v["fingerprint"].as_str().unwrap(),cut).is_err());
  let before=pair(&f);assert!(f.data().is_err(),"normal reader accepted pending {cut}");assert!(f.review().is_err());assert!(pair(&f)==before);
  let receipt=apply(&f);assert_eq!(receipt["futurePreparationBlocked"],false);assert!(f.data().is_ok());assert!(f.review().is_ok());assert!(!job(&f).exists());
 }
}

#[test]fn completed_retention_has_portable_content_but_current_recovery_keeps_a_physical_fence(){
 fn copy(from:&std::path::Path,to:&std::path::Path){
  fs::DirBuilder::new().mode(0o700).create(to).unwrap();
  for entry in fs::read_dir(from).unwrap(){let entry=entry.unwrap();let target=to.join(entry.file_name());if entry.path().is_dir(){copy(&entry.path(),&target);}else{fs::copy(entry.path(),&target).unwrap();fs::set_permissions(target,fs::Permissions::from_mode(0o600)).unwrap();}}
 }
 let f=Fixture::new();opaque(&f);apply(&f);let source=Dir::root(&f.config()).unwrap();
 let semantic=crate::pet_data::publication::restore::retention::fingerprint(&source,false).unwrap();let physical=crate::pet_data::publication::restore::retention::fingerprint(&source,true).unwrap();
 let t=Temp::new();let path=t.0.join("retained-config-copy");copy(&f.config(),&path);let imported=Dir::root(&path).unwrap();
 assert_eq!(crate::pet_data::publication::restore::retention::fingerprint(&imported,false).unwrap(),semantic);
 assert_ne!(crate::pet_data::publication::restore::retention::fingerprint(&imported,true).unwrap(),physical);
 // Portable data validation is not an admission, native claim or executable grant.
 assert!(f.review().is_ok());assert!(!job(&f).exists());assert!(!path.join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}

#[test]fn joint_retention_entry_boundary_counts_retained_and_current_originals_including_directories(){
 let f=Fixture::new();opaque(&f);for i in 0..2046{f.target.file(&job(&f).join(format!("entry-{i:04}")),b"");}let first=apply(&f);let original=retained(&f,&first);let original_inode=fs::metadata(&original).unwrap().ino();
 fs::DirBuilder::new().mode(0o700).create(job(&f)).unwrap();for i in 0..2047{f.target.file(&job(&f).join(format!("current-{i:04}")),b"");}let current_inode=fs::metadata(job(&f)).unwrap().ino();let before=pair(&f);let r=review(&f).unwrap();let preview=r.preview().unwrap();assert_eq!(preview["retainedBytesTrustedAsAuthority"],false);drop(r);assert_eq!(pair(&f),before);
 f.target.file(&job(&f).join("one-entry-too-many"),b"");let before=pair(&f);assert!(matches!(review(&f),Err(Error::Limit)));assert_eq!(pair(&f),before);assert_eq!(fs::metadata(job(&f)).unwrap().ino(),current_inode);assert_eq!(fs::metadata(original).unwrap().ino(),original_inode);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}

#[test]fn joint_retention_byte_boundary_does_not_reset_the_limit_for_another_original(){
 let f=Fixture::new();fs::DirBuilder::new().mode(0o700).create(job(&f)).unwrap();let chunk=vec![0x6du8;8*1024*1024];for i in 0..4{f.target.file(&job(&f).join(format!("chunk-{i}")),&chunk);}let first=apply(&f);let original=retained(&f,&first);let original_inode=fs::metadata(&original).unwrap().ino();
 fs::DirBuilder::new().mode(0o700).create(job(&f)).unwrap();for i in 0..4{f.target.file(&job(&f).join(format!("chunk-{i}")),&chunk);}drop(chunk);let current_inode=fs::metadata(job(&f)).unwrap().ino();let before=pair(&f);let r=review(&f).unwrap();assert_eq!(r.preview().unwrap()["retainedBytesTrustedAsAuthority"],false);drop(r);assert_eq!(pair(&f),before);
 f.target.file(&job(&f).join("one-byte-too-many"),b"x");let before=pair(&f);assert!(matches!(review(&f),Err(Error::Limit)));assert_eq!(pair(&f),before);assert_eq!(fs::metadata(job(&f)).unwrap().ino(),current_inode);assert_eq!(fs::metadata(original).unwrap().ino(),original_inode);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}

#[test]fn joint_retention_individual_file_limit_refuses_an_oversized_current_original_without_publication(){
 let f=Fixture::new();fs::DirBuilder::new().mode(0o700).create(job(&f)).unwrap();f.target.file(&job(&f).join("oversized.bin"),&vec![0x2au8;8*1024*1024+1]);let before=pair(&f);let current_inode=fs::metadata(job(&f)).unwrap().ino();assert!(matches!(review(&f),Err(Error::Limit)));assert_eq!(pair(&f),before);assert_eq!(fs::metadata(job(&f)).unwrap().ino(),current_inode);assert!(!f.config().join(ROOT).exists());assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}
