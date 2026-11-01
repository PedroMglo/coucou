use super::*;
use crate::pet_data::{tests::Temp,startup::tests::Harness,current_profile,publication::{Publication,Active},restore_cli};
use crate::pet_runtime::{restoration::{fixture_copied_goal_catalog,fixture_prepared_seed_denies_old_ids,IndependentFixtureRecord as Record}};
use std::{fs,os::unix::fs::{DirBuilderExt,PermissionsExt,symlink}};
#[path="publication_restore_retention_tests.rs"]mod retention_tests;
struct Fixture{source:Temp,target:Temp,owner:Harness,workspace:PathBuf,id:String,old:Record}
fn tree(path:&std::path::Path)->Vec<(PathBuf,Vec<u8>)>{fn visit(root:&std::path::Path,p:&std::path::Path,out:&mut Vec<(PathBuf,Vec<u8>)>){for e in fs::read_dir(p).unwrap(){let p=e.unwrap().path();if p.is_dir(){visit(root,&p,out);}else{out.push((p.strip_prefix(root).unwrap().into(),fs::read(p).unwrap()));}}}let mut v=vec![];visit(path,path,&mut v);v.sort();v}
impl Fixture{
 fn new()->Self{
  let source=Temp::new();let h=Harness::new();let layout=source.basic();let active=Publication::begin(h.claim(&layout).unwrap(),layout).unwrap().publish().unwrap();let p=active.paths().unwrap();
  fs::write(p.config.join("settings.json"),serde_json::to_vec(&crate::settings::Settings::default()).unwrap()).unwrap();fs::write(p.config.join("pets.json"),serde_json::to_vec(&crate::pet_config::defaults()).unwrap()).unwrap();
  let old=fixture_copied_goal_catalog(&p.config.join("pet-runtime-v1"));
  fs::write(p.data.join("assistant/conversations.json"),br#"{"version":4,"pets":{"codex":{"id":"old-current","created":1,"updated":2,"messages":[],"session":{"id":"old-native","cwd":"/private-fixture","full_access":false},"tasks":[]}},"archives":{}}"#).unwrap();source.file(&p.config.join("pet-runtime-results-v1/unknown.bin"),b"private uncertain consumption; never zero");
  assert!(!p.config.join("pet-runtime-consent-v1").exists());let workspace=source.0.join("backups");fs::DirBuilder::new().mode(0o700).create(&workspace).unwrap();let b=current_profile::Review::capture(active,&workspace).unwrap();let fp=b.fingerprint().unwrap();let id=b.copy(&fp).unwrap().0;
  let target=Temp::new();let layout=target.layout();for p in [&layout.legacy_config,&layout.legacy_data]{fs::DirBuilder::new().recursive(true).mode(0o700).create(p).unwrap();}let owner=Harness::new();let f=Self{source,target,owner,workspace,id,old};let d=f.data().unwrap();let fp=d.fingerprint().unwrap();d.apply(&fp).unwrap();f
 }
 fn config(&self)->PathBuf{self.target.layout().config().to_owned()}
 fn data(&self)->Result<Review>{let layout=self.target.layout();let mode=if layout.config().exists(){Mode::Resume}else{Mode::Fresh};Review::capture(self.owner.claim(&layout)?,layout,&self.workspace,&self.id,mode)}
 fn review(&self)->Result<IndependentReview>{IndependentReview::capture(self.data()?)}
 fn seed(&self)->PathBuf{self.config().join(JOB).join(SEED)}
 fn payload_bytes(&self)->Vec<(PathBuf,Vec<u8>)>{tree(&self.config().join(PAYLOAD))}
 fn apply(&self)->Value{let r=self.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();r.apply(&fp).unwrap()}
}
#[test]fn exact_review_prepares_same_hmac_denial_without_operational_key_or_old_goal_changes(){
 let f=Fixture::new();let before=f.payload_bytes();let source=tree(f.source.layout().config());let r=f.review().unwrap();let v=r.preview().unwrap();assert!(!f.config().join(JOB).exists());assert_eq!(v["goals"]["originalAuthorityPresent"],false);assert_eq!(v["goals"]["goals"][0]["id"],f.old.id);assert!(v["configuration"]["settings"]["resolvedGroups"]["inferenceSelections"]["chatProvider"].is_string());let fp=v["fingerprint"].as_str().unwrap();let receipt=r.apply(fp).unwrap();assert_eq!(receipt["status"],"independentAuthorityPrepared");assert_eq!(receipt["runtimeActivationAvailable"],false);assert_eq!(receipt["authority"]["grantsCreated"],0);assert_eq!(f.payload_bytes(),before);assert_eq!(tree(f.source.layout().config()),source);assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());assert!(fixture_prepared_seed_denies_old_ids(&f.seed(),&f.old));assert!(Active::open(f.owner.claim(&f.target.layout()).unwrap(),f.target.layout()).is_err());
 let index=History::decode_first(&fs::read(f.config().join(JOB).join(HISTORY)).unwrap(),v["references"]["indexHash"].as_str().unwrap()).unwrap();assert!(index.native_session_allowed("codex","old-native").is_err());assert_eq!(receipt,f.apply());
}
#[test]fn wrong_or_cross_profile_hash_and_owner_or_source_change_never_generate_a_key(){
 for action in ["wrong","cross","owner","payload","source"]{let f=Fixture::new();let r=f.review().unwrap();let mut fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();match action{"wrong"=>fp="0".repeat(64),"cross"=>{let g=Fixture::new();fp=g.review().unwrap().preview().unwrap()["fingerprint"].as_str().unwrap().into();},"owner"=>f.owner.release(),"payload"=>fs::write(f.config().join(PAYLOAD).join("settings.json"),b"{}").unwrap(),_=>fs::write(f.workspace.join(format!("current-profile-{}/config/settings.json",f.id)),b"{}").unwrap()};assert!(r.apply(&fp).is_err(),"{action}");assert!(!f.config().join(JOB).exists(),"{action}");}
}
#[test]fn durable_preparation_cuts_reuse_manifest_key_and_history_under_a_fresh_phase_review(){
 for cut in ["intent","key","base","history","ready"]{let f=Fixture::new();let before=f.payload_bytes();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply_using(&fp,|s|if s==cut{Err(Error::Unavailable)}else{Ok(())}).is_err(),"{cut}");let manifest=fs::read(f.config().join(JOB).join(MANIFEST)).unwrap();let key=fs::read(f.seed().join(".key.json")).ok();let old_index=fs::read(f.seed().join(".restored-history.json")).ok();let r=f.review().unwrap();let fresh=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert_ne!(fp,fresh);assert!(r.apply(&fp).is_err());f.apply();assert_eq!(fs::read(f.config().join(JOB).join(MANIFEST)).unwrap(),manifest);if let Some(k)=key{assert!(fs::read(f.seed().join(".key.json")).unwrap()==k,"seed key changed");}if let Some(i)=old_index{assert_eq!(fs::read(f.seed().join(".restored-history.json")).unwrap(),i);}assert_eq!(f.payload_bytes(),before);}
}
#[test]fn malformed_or_existing_authority_is_never_reclassified_as_absent(){
 for location in ["payload","source"]{let f=Fixture::new();let p=if location=="payload"{f.config().join(PAYLOAD)}else{f.workspace.join(format!("current-profile-{}/config",f.id))};fs::DirBuilder::new().mode(0o700).create(p.join("pet-runtime-consent-v1")).unwrap();f.target.file(&p.join("pet-runtime-consent-v1/.key.json"),b"invalid private key placeholder");assert!(f.review().is_err());assert!(!f.config().join(JOB).exists());}
}
#[test]fn partial_unknown_replaced_seed_or_revoked_owner_during_io_never_reports_success_or_regenerates(){
 for action in ["partialKey","extra","symlink","goal","owner"]{let f=Fixture::new();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply_using(&fp,|s|{if s=="key"{match action{"partialKey"=>fs::write(f.seed().join(".key.json"),b"{").unwrap(),"extra"=>f.target.file(&f.seed().join("foreign.json"),b"keep unknown"),"symlink"=>{let p=f.seed().join(".key.json");fs::remove_file(&p).unwrap();symlink(f.source.0.join("outside"),p).unwrap();},"goal"=>fs::write(f.config().join(PAYLOAD).join("pet-runtime-v1").join(format!("{}.json",f.old.id)),b"changed goal").unwrap(),_=>f.owner.release()}}Ok(())}).is_err(),"{action}");assert!(!f.config().join(JOB).join(READY).exists());if action=="partialKey"{let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply(&fp).is_err());assert_eq!(fs::read(f.seed().join(".key.json")).unwrap(),b"{");}}
}
#[test]fn native_cli_prepares_and_acknowledges_without_models_or_payload_publication(){
 let f=Fixture::new();f.owner.release();let args=|fp:Option<&str>|{let mut a=vec!["coucou".into(),"--pets-profile-restore-authority".into(),f.workspace.to_string_lossy().into_owned(),f.id.clone()];if let Some(fp)=fp{a.extend(["--approve".into(),fp.into()]);}restore_cli::parse(&a).unwrap().unwrap()};let p=restore_cli::execute(args(None),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();assert_eq!(p["status"],"independentAuthorityPreparationReview");let receipt=restore_cli::execute(args(p["fingerprint"].as_str()),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();let before=tree(&f.config());let p=restore_cli::execute(args(None),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();assert_eq!(restore_cli::execute(args(p["fingerprint"].as_str()),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap(),receipt);assert!(tree(&f.config())==before,"native ACK changed data");assert!(!f.config().join(ADMITTED).exists());
}
#[test]fn removed_prepared_key_is_never_regenerated_when_the_seed_directory_becomes_empty(){
 let f=Fixture::new();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply_using(&fp,|s|if s=="key"{Err(Error::Unavailable)}else{Ok(())}).is_err());assert!(f.seed().join(".key.json").is_file());assert_eq!(fs::read_dir(f.seed()).unwrap().count(),1);fs::remove_file(f.seed().join(".key.json")).unwrap();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();assert!(r.apply(&fp).is_err(),"removed prepared key must never be regenerated");assert!(!f.seed().join(".key.json").exists());assert!(!f.config().join(JOB).join(READY).exists());
}

#[test]fn generation_intent_survives_pre_key_cut_and_missing_marker_never_authorizes_retry(){
 for action in ["beforeKey","removedMarker"]{let f=Fixture::new();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();let cut=if action=="beforeKey"{"generation"}else{"key"};assert!(r.apply_using(&fp,|s|if s==cut{Err(Error::Unavailable)}else{Ok(())}).is_err());let marker=f.config().join(JOB).join(GENERATION);assert!(marker.is_file());
  if action=="beforeKey"{assert!(!f.seed().join(".key.json").exists());let r=f.review().unwrap();let p=r.preview().unwrap();assert_eq!(p["preparationState"],"keyGenerationUncertain");assert_eq!(p["keyGenerationAvailable"],false);assert_eq!(p["manualReconciliationRequired"],true);assert!(r.apply(p["fingerprint"].as_str().unwrap()).is_err());assert!(!f.seed().join(".key.json").exists());}
  else{fs::remove_file(marker).unwrap();assert!(f.review().is_err());assert!(f.seed().join(".key.json").exists());}
  assert!(!f.config().join(JOB).join(READY).exists());
 }
}

#[test]fn invalid_existing_key_is_reviewed_as_manual_reconciliation_and_approved_apply_never_writes(){
 for variant in ["truncated","empty","null","schema","keyId","length","uppercase","extra"]{
  let f=Fixture::new();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();
  assert!(r.apply_using(&fp,|s|if s=="key"{Err(Error::Unavailable)}else{Ok(())}).is_err());
  let key=f.seed().join(".key.json");let original:Value=serde_json::from_slice(&fs::read(&key).unwrap()).unwrap();let mut invalid=original.clone();
  let stale=f.review().unwrap();let stale_fp=stale.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();
  let raw=match variant{
   "truncated"=>b"{".to_vec(),"empty"=>vec![],"null"=>b"null".to_vec(),
   _=>{match variant{"schema"=>invalid["schemaVersion"]=json!(2),"keyId"=>invalid["keyId"]=json!("not an exact native token"),"length"=>invalid["key"]=json!("a".repeat(62)),"uppercase"=>invalid["key"]=json!("A".repeat(64)),"extra"=>invalid["authorityGranted"]=json!(true),_=>unreachable!()};serde_json::to_vec(&invalid).unwrap()},
  };fs::write(&key,&raw).unwrap();
  let before=tree(&f.config());let source_before=tree(f.source.layout().config());let payload_before=f.payload_bytes();
  assert!(stale.apply(&stale_fp).is_err(),"{variant}");assert_eq!(tree(&f.config()),before,"stale approval wrote {variant}");
  for _ in 0..2{
   let r=f.review().unwrap();let v=r.preview().unwrap();assert_eq!(v["preparationState"],"keyInvalid","{variant}");
   assert_eq!(v["manualReconciliationRequired"],true);assert_eq!(v["keyGenerationAvailable"],false);assert_eq!(v["keyRegenerationAllowed"],false);
   assert_eq!(v["executionAuthorityGranted"],false);assert_eq!(v["runtimeActivationAvailable"],false);
   let shown=serde_json::to_string(&v).unwrap();assert!(!shown.contains(original["key"].as_str().unwrap()));assert!(!shown.contains(original["keyId"].as_str().unwrap()));
   assert_eq!(tree(&f.config()),before,"preview wrote {variant}");let approved=v["fingerprint"].as_str().unwrap();assert!(r.apply(approved).is_err(),"{variant}");
   assert_eq!(tree(&f.config()),before,"approved apply wrote {variant}");
  }
  assert_eq!(fs::read(&key).unwrap(),raw);assert_eq!(f.payload_bytes(),payload_before);assert_eq!(tree(f.source.layout().config()),source_before);
  assert!(!f.config().join(JOB).join(READY).exists());assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
 }
}

#[test]fn normal_preparation_review_states_remain_read_only_and_resume_the_same_key(){
 for cut in ["intent","generation","key","base","history","ready"]{
  let f=Fixture::new();let r=f.review().unwrap();let fp=r.preview().unwrap()["fingerprint"].as_str().unwrap().to_owned();
  assert!(r.apply_using(&fp,|s|if s==cut{Err(Error::Unavailable)}else{Ok(())}).is_err());
  let before=tree(&f.config());let key_before=fs::read(f.seed().join(".key.json")).ok();let r=f.review().unwrap();let v=r.preview().unwrap();
  let state=match cut{"intent"=>"notStarted","generation"=>"keyGenerationUncertain","ready"=>"prepared",_=>"keyPresent"};
  assert_eq!(v["preparationState"],state);assert_eq!(v["manualReconciliationRequired"],cut=="generation");assert_eq!(v["keyGenerationAvailable"],cut=="intent");assert_eq!(v["keyRegenerationAllowed"],false);assert_eq!(tree(&f.config()),before);
  if cut=="generation"{assert!(r.apply(v["fingerprint"].as_str().unwrap()).is_err());assert_eq!(tree(&f.config()),before);}
  else{r.apply(v["fingerprint"].as_str().unwrap()).unwrap();if let Some(key)=key_before{assert!(fs::read(f.seed().join(".key.json")).unwrap()==key,"valid preparation key changed");}}
 }
}

#[test]fn prepared_ready_with_a_corrupted_key_is_refused_without_reclassifying_as_a_new_preparation(){
 let f=Fixture::new();f.apply();let ready=fs::read(f.config().join(JOB).join(READY)).unwrap();
 fs::write(f.seed().join(".key.json"),b"{").unwrap();let before=tree(&f.config());
 assert!(f.review().is_err());assert_eq!(tree(&f.config()),before);assert_eq!(fs::read(f.config().join(JOB).join(READY)).unwrap(),ready);
 assert!(!f.config().join(PAYLOAD).join("pet-runtime-consent-v1").exists());
}
