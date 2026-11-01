use super::*;
use crate::pet_data::{tests::Temp,startup::tests::Harness,current_profile,bootstrap,restore_cli};
use std::{fs,os::unix::fs::{DirBuilderExt,PermissionsExt,symlink}};
fn settings()->Vec<u8>{serde_json::to_vec(&crate::settings::Settings::default()).unwrap()}
fn pets()->Vec<u8>{serde_json::to_vec(&crate::pet_config::defaults()).unwrap()}
struct Fixture{source:Temp,target:Temp,owner:Harness,workspace:PathBuf,id:String}
impl Fixture{
 fn new()->Self{let source=Temp::new();let owner=Harness::new();let layout=source.basic();let active=Publication::begin(owner.claim(&layout).unwrap(),layout).ok().unwrap().publish().ok().unwrap();let paths=active.paths().unwrap();fs::write(paths.config.join("settings.json"),settings()).unwrap();fs::write(paths.config.join("pets.json"),pets()).unwrap();
  let workspace=source.0.join("backups");fs::DirBuilder::new().mode(0o700).create(&workspace).unwrap();let review=current_profile::Review::capture(active,&workspace).unwrap();let hash=review.fingerprint().unwrap();let(id,_)=review.copy(&hash).unwrap();let target=Temp::new();let layout=target.layout();for path in [&layout.legacy_config,&layout.legacy_data]{fs::DirBuilder::new().recursive(true).mode(0o700).create(path).unwrap();}let owner=Harness::new();let f=Self{source,target,owner,workspace,id};let review=f.data();let hash=review.fingerprint().unwrap();review.apply(&hash).unwrap();f}
 fn data(&self)->Review{let layout=self.target.layout();Review::capture(self.owner.claim(&layout).unwrap(),layout,&self.workspace,&self.id,if self.target.layout().config().exists(){Mode::Resume}else{Mode::Fresh}).unwrap()}
 fn review(&self)->ConfigurationReview{self.data().configuration().unwrap()}
 fn config(&self)->PathBuf{self.target.layout().config().to_owned()}
 fn bytes(&self)->Vec<(String,Vec<u8>)>{let mut out=vec![];for name in [CONFIG_REVIEW,HEADER,COMPLETE,RESTORED,INTENT,ORIGIN]{if let Ok(b)=fs::read(self.config().join(name)){out.push((name.into(),b));}}for name in ["settings.json","pets.json"]{out.push((format!("payload/{name}"),fs::read(self.config().join(PAYLOAD).join(name)).unwrap()));}out}
}
#[test]fn strict_inspection_refuses_malformed_fields_and_unknown_fields_without_salvage(){
 for raw in [b" ".as_slice(),b"[]",br#"{"soundVolume":"bad"}"#,br#"{"apiSecret":"do-not-print"}"#,br#"{"desktopMochi":{"unknownAction":true}}"#]{assert!(inspect(Some(raw),Some(&pets())).is_err());}
}
#[test]fn defaults_and_actual_factory_upgrades_are_visible_without_persisting_or_changing_provider(){
 let mut raw:Value=serde_json::from_slice(&pets()).unwrap();raw["pets"]["agent_copilot"]["provider"]=json!("");raw["pets"]["agent_copilot"]["permissions"]=Value::Null;let data=serde_json::to_vec(&raw).unwrap();let report=inspect(Some(br#"{"chatProvider":"lmstudio","chatModels":{"lmstudio":"chosen-exact"}}"#),Some(&data)).unwrap();
 assert_eq!(report["settings"]["resolvedGroups"]["inferenceSelections"]["chatProvider"],"lmstudio");assert_eq!(report["settings"]["resolvedGroups"]["inferenceSelections"]["chatModels"]["lmstudio"],"chosen-exact");assert!(report["settings"]["resolutionChanges"].as_array().unwrap().iter().any(|x|x=="autostart"));assert!(report["pets"]["resolutionChanges"].as_array().unwrap().iter().any(|x|x=="pets.agent_copilot.provider"));assert_eq!(report["pets"]["resolved"],value(crate::pet_config::resolve_existing_bytes(&data).unwrap()).unwrap());assert_eq!(report["activationGranted"],false);
}
#[test]fn unknown_pet_permissions_profiles_and_execution_flags_are_not_ignored(){
 for path in [vec!["pets","agent_codex","permissions","shellRoot"],vec!["pets","agent_codex","fast","fallbackModel"],vec!["pets","ai_ollama","localExecution","ignoreNativeApprovals"],vec!["futureRoot"]]{let mut raw:Value=serde_json::from_slice(&pets()).unwrap();let mut at=&mut raw;for key in &path[..path.len()-1]{at=&mut at[*key];}at[*path.last().unwrap()]=json!(true);assert!(inspect(Some(&settings()),Some(&serde_json::to_vec(&raw).unwrap())).is_err());}
}
#[test]fn current_settings_fields_are_classified_once_and_no_missing_file_is_created(){
 let hash="a".repeat(64);assert_eq!(value(History::Quarantined{history_hash:hash.clone()}).unwrap(),json!({"quarantined":{"historyHash":hash}}));
 let report=inspect(None,None).unwrap();let groups=report["settings"]["resolvedGroups"].as_object().unwrap();assert!(!groups.contains_key("requiresClassification"));let fields:usize=groups.values().map(|v|v.as_object().unwrap().len()).sum();assert_eq!(fields,value(crate::settings::Settings::default()).unwrap().as_object().unwrap().len());assert_eq!(report["settings"]["present"],false);assert_eq!(report["pets"]["present"],false);assert_eq!(report["runtimeActivationAvailable"],false);
}
#[test]fn native_review_is_read_only_then_receipt_changes_only_control_and_cannot_admit_bootstrap(){
 let f=Fixture::new();let before=f.bytes();let review=f.review();let p=review.preview().unwrap();assert_eq!(f.bytes(),before);assert_eq!(p["consentPurpose"],"configurationInspectionOnly");assert_eq!(p["history"],"noOriginalAuthority");let result=review.apply(p["fingerprint"].as_str().unwrap()).unwrap();assert_eq!(result["settingsActivated"],false);assert!(!f.config().join(ADMITTED).exists());assert_eq!(fs::read(f.config().join(PAYLOAD).join("settings.json")).unwrap(),settings());let layout=f.target.layout();assert!(bootstrap::Session::prepare(f.owner.claim(&layout).unwrap(),layout).is_err());assert_eq!(fs::read(f.source.layout().config().join(PAYLOAD).join("settings.json")).unwrap(),settings());
}
#[test]fn wrong_or_cross_profile_approval_cannot_publish_a_configuration_receipt(){
 let f=Fixture::new();let g=Fixture::new();let before=f.bytes();let r=g.review();let hash=r.fingerprint().unwrap();assert!(f.review().apply(&hash).is_err());assert_eq!(f.bytes(),before);assert!(f.review().apply(&"0".repeat(64)).is_err());assert_eq!(f.bytes(),before);
}
#[test]fn lost_ack_is_idempotent_and_tampered_receipt_is_never_overwritten(){
 let f=Fixture::new();let r=f.review();let hash=r.fingerprint().unwrap();let one=r.apply(&hash).unwrap();let before=f.bytes();let r=f.review();let hash=r.fingerprint().unwrap();assert_eq!(r.apply(&hash).unwrap(),one);assert_eq!(f.bytes(),before);fs::write(f.config().join(CONFIG_REVIEW),b"forged activation approval").unwrap();let before=f.bytes();assert!(f.data().configuration().is_err());assert_eq!(f.bytes(),before);
}
#[test]fn changed_configuration_after_review_and_lost_native_claim_refuse_before_publication(){
 for action in ["configuration","claim"]{let f=Fixture::new();let r=f.review();let hash=r.fingerprint().unwrap();if action=="configuration"{fs::write(f.config().join(PAYLOAD).join("settings.json"),b"{}").unwrap();}else{f.owner.release();}assert!(r.apply(&hash).is_err());assert!(!f.config().join(CONFIG_REVIEW).exists());}
}
#[test]fn native_guard_loss_at_publication_boundary_never_creates_approval(){
 let f=Fixture::new();let r=f.review();let hash=r.fingerprint().unwrap();assert!(r.apply_using(&hash,|_|{f.owner.release();Ok(())}).is_err());assert!(!f.config().join(CONFIG_REVIEW).exists());
}
#[test]fn symlinked_configuration_and_unquarantined_authority_cannot_be_reviewed(){
 for mode in ["symlink","authority"]{let f=Fixture::new();let path=f.config().join(PAYLOAD);if mode=="symlink"{let p=path.join("settings.json");fs::remove_file(&p).unwrap();symlink(f.workspace.join(format!("current-profile-{}/config/settings.json",f.id)),&p).unwrap();}else{let p=path.join("pet-runtime-consent-v1");fs::DirBuilder::new().mode(0o700).create(&p).unwrap();fs::write(p.join(".key.json"),b"invalid").unwrap();fs::set_permissions(p.join(".key.json"),fs::Permissions::from_mode(0o600)).unwrap();}let layout=f.target.layout();let result=Review::capture(f.owner.claim(&layout).unwrap(),layout,&f.workspace,&f.id,Mode::Resume).and_then(|r|r.configuration());assert!(result.is_err());assert!(!f.config().join(CONFIG_REVIEW).exists());}
}
#[test]fn actual_cli_dispatch_records_inspection_only_and_preserves_root2_rejection(){
 let f=Fixture::new();f.owner.release();let args=|hash:Option<&str>|{let mut v=vec!["coucou".to_string(),"--pets-profile-restore-configuration".into(),f.workspace.to_string_lossy().into_owned(),f.id.clone()];if let Some(h)=hash{v.extend(["--approve".into(),h.into()]);}restore_cli::parse(&v).unwrap().unwrap()};let review=restore_cli::execute(args(None),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();let result=restore_cli::execute(args(review["fingerprint"].as_str()),f.owner.address(),"fr.louisraille.coucou",f.target.layout()).unwrap();assert_eq!(result["status"],"restoredConfigurationReviewed");assert_eq!(result["runtimeActivationAvailable"],false);assert!(!f.config().join(ADMITTED).exists());
}

#[test]fn ownership_loss_after_receipt_never_reports_activation_or_success_and_retains_partial_receipt(){
 let f=Fixture::new();let r=f.review();let hash=r.fingerprint().unwrap();assert!(r.apply_using(&hash,|step|{if step==PublicationStep::AfterReceipt{f.owner.release();}Ok(())}).is_err());assert!(f.config().join(CONFIG_REVIEW).exists());assert!(!f.config().join(ADMITTED).exists());assert_eq!(fs::read(f.config().join(PAYLOAD).join("settings.json")).unwrap(),settings());
}
