use super::*;
use crate::pet_data::{tests::Temp,startup::tests::Harness,publication::Publication,backup_cli::{self,Request},staging};
use std::{fs,os::unix::fs::{PermissionsExt,MetadataExt,symlink}};
fn active(t:&Temp,h:&Harness)->Active{let layout=t.basic();Publication::begin(h.claim(&layout).unwrap(),layout).ok().unwrap().publish().ok().unwrap()}
fn workspace(t:&Temp)->PathBuf{let path=t.0.join("private-backups");fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();path}
#[test]fn shortcut_format8_survives_current_backup_inspection_without_downgrading_controls(){
 let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let layout=active.layout().clone();let storage=active.into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_upgrade(&path,8).unwrap();drop(storage);let active=Active::open(h.claim(&layout).unwrap(),layout.clone()).unwrap();let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let fp=review.fingerprint().unwrap();let(id,_)=review.copy(&fp).unwrap();let archive=target.join(format!("current-profile-{id}"));let manifest:serde_json::Value=serde_json::from_slice(&fs::read(archive.join("manifest.json")).unwrap()).unwrap();assert_eq!(manifest["origin"]["settingsIoProtocol"],8);assert_eq!(staging::inspect_current(&target,&id).unwrap().phase,staging::Phase::VerifiedData);
 for(name,expected)in super::super::settings_io::extra(Some(8)).unwrap(){assert_eq!(fs::read(layout.config().join(name)).unwrap(),expected);}
}
#[test]fn shortcut_format7_survives_current_backup_inspection_without_downgrading_controls(){
 let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let layout=active.layout().clone();let storage=active.into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_upgrade(&path,7).unwrap();drop(storage);let active=Active::open(h.claim(&layout).unwrap(),layout.clone()).unwrap();let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let fp=review.fingerprint().unwrap();let(id,_)=review.copy(&fp).unwrap();let archive=target.join(format!("current-profile-{id}"));let manifest:serde_json::Value=serde_json::from_slice(&fs::read(archive.join("manifest.json")).unwrap()).unwrap();assert_eq!(manifest["origin"]["settingsIoProtocol"],7);assert_eq!(staging::inspect_current(&target,&id).unwrap().phase,staging::Phase::VerifiedData);
 for(name,expected)in super::super::settings_io::extra(Some(7)).unwrap(){assert_eq!(fs::read(layout.config().join(name)).unwrap(),expected);}
}
#[test]fn settings_durable_profile_extension_and_pending_outcome_are_portably_backed_up(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let layout=active.layout().clone();let paths=active.paths().unwrap();
    let storage=active.into_storage().unwrap();assert!(!storage.settings_io_protocol(&paths.config.join("settings.json"),false).unwrap());
    assert!(storage.settings_io_protocol(&paths.config.join("settings.json"),true).unwrap());drop(storage);
    let settings=crate::settings::load_from(&paths.config.join("settings.json"));crate::settings::fixture_directory_sync_failure(&paths.config.join("settings.json"),&settings).unwrap_err();
    let original=fs::read(paths.config.join(crate::settings::durable::FILE)).unwrap();
    let active=Active::open(h.claim(&layout).unwrap(),layout.clone()).unwrap();assert_eq!(active.backup_origin().unwrap().settings_io_protocol,Some(1));
    let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let fp=review.fingerprint().unwrap();let(id,_)=review.copy(&fp).unwrap();
    let output=target.join(format!("current-profile-{id}"));assert_eq!(fs::read(output.join("config").join(crate::settings::durable::FILE)).unwrap(),original);
    let manifest:serde_json::Value=serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();assert_eq!(manifest["origin"]["settingsIoProtocol"],1);
    assert_eq!(manifest["origin"]["schemaVersion"],1);assert_eq!(staging::inspect_current(&target,&id).unwrap().phase,staging::Phase::VerifiedData);
    let bytes=fs::read(layout.config().join(super::super::settings_io::CONTROL)).unwrap();assert_eq!(bytes,b"{\"version\":1}");
}
fn full(t:&Temp,active:&Active)->Vec<(PathBuf,Vec<u8>)>{
    let paths=active.paths().unwrap();let values=[
        (paths.config.join("pet-runtime-v1/goal.json"),b"{\"schemaVersion\":12,\"uncertain\":true}".as_slice()),
        (paths.config.join("pet-runtime-consent-v1/.key.json"),b"private synthetic application key"),
        (paths.config.join("pet-runtime-consent-v1/retained.json"),b"synthetic signed grant: consumed budget stays consumed"),
        (paths.config.join("pet-runtime-results-v1/unknown.json"),b"unknown effect must not replay"),
        (paths.config.join("pet-runtime-planning-v1/round.json"),b"retained round"),
        (paths.config.join("pet-runtime-consent-v1.owners-v1/original.lock"),b""),
        (paths.config.join("future-module/state"),b"unknown app data retained, not executed"),
        (paths.data.join("assistant/memories.json"),b"synthetic selected scoped memory"),
        (paths.data.join("assistant/attachments/item/original.txt"),b"first retained attachment"),
        (paths.data.join("inbox/other/original.txt"),b"same filename, different bytes"),
    ];values.into_iter().map(|(p,b)|{t.file(&p,b);(p,b.to_vec())}).collect()
}
#[test]fn entire_current_payload_and_application_keys_are_copied_as_data_without_legacy_filtering(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let original=full(&t,&active);let paths=active.paths().unwrap();let target=workspace(&t);
    let review=Review::capture(active,&target).unwrap();let preview=review.preview().unwrap();assert_eq!(preview["executionAuthorityGranted"],false);assert_eq!(preview["restorationAvailable"],false);assert_eq!(preview["dataRestorationAvailable"],true);assert_eq!(preview["runtimeActivationAvailable"],false);assert_eq!(preview["explicitReviewRequired"],true);assert!(!preview.to_string().contains("synthetic signed grant"));
    let fingerprint=review.fingerprint().unwrap();let(id,summary)=review.copy(&fingerprint).unwrap();assert_eq!(summary.excluded_roots,0);let output=target.join(format!("current-profile-{id}"));let manifest:serde_json::Value=serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();assert_eq!(manifest["origin"]["schemaVersion"],1);assert!(manifest["origin"]["admission"].is_array());
    for(path,bytes)in original{let(area,relative)=if let Ok(p)=path.strip_prefix(&paths.config){("config",p)}else{("data",path.strip_prefix(&paths.data).unwrap())};let copied=output.join(area).join(relative);assert_eq!(fs::read(&copied).unwrap(),bytes);assert_eq!(fs::read(&path).unwrap(),bytes);assert_eq!(fs::metadata(copied).unwrap().mode()&0o7777,0o600);}
    assert_eq!(staging::inspect_current(&target,&id).unwrap().phase,staging::Phase::VerifiedData);assert!(staging::inspect(&target,&id).is_err());
}
#[test]fn wrong_approval_source_change_and_lost_owner_refuse_before_a_candidate_exists(){
    for action in ["wrongApproval","sourceChange","lostOwner"]{
        let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let paths=active.paths().unwrap();let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let mut fingerprint=review.fingerprint().unwrap();
        match action{"wrongApproval"=>fingerprint="0".repeat(64),"sourceChange"=>fs::write(paths.config.join("settings.json"),b"{\"changed\":true}").unwrap(),"lostOwner"=>h.release(),_=>unreachable!()}
        let failure=review.copy(&fingerprint).err().expect("review must refuse");assert!(failure.candidate_id.is_none());assert_eq!(fs::read_dir(&target).unwrap().count(),0);
    }
}
#[test]fn a_replaced_destination_cannot_receive_the_previously_reviewed_copy(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let fingerprint=review.fingerprint().unwrap();fs::rename(&target,t.0.join("original-target")).unwrap();fs::create_dir(&target).unwrap();fs::set_permissions(&target,fs::Permissions::from_mode(0o700)).unwrap();
    assert!(review.copy(&fingerprint).is_err());assert_eq!(fs::read_dir(&target).unwrap().count(),0);assert_eq!(fs::read_dir(t.0.join("original-target")).unwrap().count(),0);
}
#[test]fn invalid_files_and_destinations_are_refused_without_following_or_copying_them(){
    for kind in ["symlink","hardlink","publicTarget","insideProfile"]{
        let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let paths=active.paths().unwrap();let mut target=workspace(&t);
        match kind{
            "symlink"=>symlink(t.0.join("not-owned"),paths.data.join("external")).unwrap(),
            "hardlink"=>{let p=paths.config.join("settings.json");fs::hard_link(p,t.0.join("external-link")).unwrap();},
            "publicTarget"=>fs::set_permissions(&target,fs::Permissions::from_mode(0o755)).unwrap(),
            "insideProfile"=>{target=paths.data.join("recursive");fs::create_dir(&target).unwrap();fs::set_permissions(&target,fs::Permissions::from_mode(0o700)).unwrap();},_=>unreachable!()
        }
        assert!(Review::capture(active,&target).is_err(),"{kind}");assert_eq!(fs::read_dir(&target).unwrap().count(),0);
    }
}
#[test]fn completed_backup_is_independently_readable_but_tampering_is_not_repaired(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let target=workspace(&t);let review=Review::capture(active,&target).unwrap();let fingerprint=review.fingerprint().unwrap();let(id,_)=review.copy(&fingerprint).unwrap();let root=target.join(format!("current-profile-{id}"));
    let ready=fs::read(root.join("ready.json")).unwrap();fs::write(root.join("config/settings.json"),b"changed").unwrap();assert!(staging::inspect_current(&target,&id).is_err());assert_eq!(fs::read(root.join("config/settings.json")).unwrap(),b"changed");assert_eq!(fs::read(root.join("ready.json")).unwrap(),ready);
}
#[test]fn maintenance_never_replaces_an_existing_native_owner_or_a_live_profile_lock(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let layout=active.layout().clone();let target=workspace(&t);
    let request=||Request::Backup{workspace:target.clone(),approve:None};
    assert_eq!(backup_cli::execute(request(),h.address(),"fr.louisraille.coucou",layout.clone()).unwrap_err().error,Error::Busy);active.current().unwrap();h.release();
    assert_eq!(backup_cli::execute(request(),h.address(),"fr.louisraille.coucou",layout).unwrap_err().error,Error::Busy);assert_eq!(fs::read_dir(target).unwrap().count(),0);
}
#[test]fn standalone_review_and_approved_copy_share_the_native_gate_without_constructing_app_state(){
    let t=Temp::new();let h=Harness::new();let active=active(&t,&h);let paths=active.paths().unwrap();let layout=active.layout().clone();let original=fs::read(paths.config.join("settings.json")).unwrap();drop(active);h.release();let target=workspace(&t);
    let request=||Request::Backup{workspace:target.clone(),approve:None};let preview=backup_cli::execute(request(),h.address(),"fr.louisraille.coucou",layout.clone()).unwrap();assert_eq!(preview["status"],"review");assert_eq!(fs::read_dir(&target).unwrap().count(),0);
    let result=backup_cli::execute(Request::Backup{workspace:target.clone(),approve:Some(preview["fingerprint"].as_str().unwrap().into())},h.address(),"fr.louisraille.coucou",layout.clone()).unwrap();assert_eq!(result["status"],"verifiedData");assert_eq!(result["executionAuthorityGranted"],false);
    let inspected=backup_cli::execute(Request::Inspect{workspace:target.clone(),id:result["candidateId"].as_str().unwrap().into()},"","ignored",layout).unwrap();assert_eq!(inspected["status"],"verifiedData");assert_eq!(fs::read(paths.config.join("settings.json")).unwrap(),original);
}
#[test]fn malformed_or_mixed_maintenance_arguments_cannot_fall_through_to_the_application(){
    let args=|a:&[&str]|a.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    assert_eq!(backup_cli::parse(&args(&["coucou","--voice-session"])).unwrap(),None);
    for a in [vec!["coucou","--pets-profile-restore","/tmp"],vec!["coucou","--pets-profile-backup"],vec!["coucou","--pets-profile-backup","/tmp","--approve","bad"],vec!["coucou","--voice-session","--pets-profile-backup","/tmp"],vec!["coucou","--pets-profile-inspect","/tmp","../file"]]{assert!(backup_cli::parse(&args(&a)).is_err());}
}
