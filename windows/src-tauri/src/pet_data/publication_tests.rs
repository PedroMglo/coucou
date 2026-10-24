use super::*;
use crate::pet_data::{startup::tests::Harness,tests::Temp};
use std::{fs,os::unix::{ffi::OsStringExt,fs::{DirBuilderExt,MetadataExt,PermissionsExt,symlink}}};

fn failed<T>(result:std::result::Result<T,Failure>)->Failure{match result{Err(f)=>f,Ok(_)=>panic!("expected publication refusal/interruption")}}
fn begin(h:&Harness,layout:&Layout)->Publication{Publication::begin(h.claim(layout).unwrap(),layout.clone()).unwrap()}
fn active(h:&Harness,layout:&Layout)->Active{begin(h,layout).publish().unwrap()}
fn assert_inactive(h:&Harness,layout:&Layout){assert!(Active::open(h.claim(layout).unwrap(),layout.clone()).is_err());}

#[test]fn destinations_are_inactive_and_owned_exceptions_do_not_weaken_general_source_review(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let publication=begin(&h,&layout);
    assert_eq!(inspect(&layout).unwrap(),State::Incomplete);assert!(matches!(LegacyReview::inspect(layout.clone()),Err(Error::TargetExists)));
    publication.current().unwrap();assert!(layout.config().join(PAYLOAD).is_dir());assert!(layout.data().join(PAYLOAD).is_dir());
    assert_eq!(fs::metadata(layout.config()).unwrap().mode()&0o7777,0o700);
    drop(publication);assert_inactive(&h,&layout);
}
#[test]fn complete_pair_admits_exact_private_copies_and_preserves_original_profile_and_assets(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();t.file(&layout.legacy_config.join("sounds/original.ogg"),b"sound bytes");t.file(&layout.legacy_data.join("Notes/note.json"),br#"{"start":100,"end":200}"#);
    let original=fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap();let live=active(&h,&layout);let paths=live.paths().unwrap();
    assert_eq!(inspect(&layout).unwrap(),State::AdmissionRecordPresent);assert_eq!(paths.config,layout.config().join(PAYLOAD));assert_eq!(paths.data,layout.data().join(PAYLOAD));
    assert_eq!(fs::read(paths.config.join("sounds/original.ogg")).unwrap(),b"sound bytes");assert_eq!(fs::read(paths.data.join("inbox/example.txt")).unwrap(),b"private attachment");
    assert_eq!(fs::read(paths.data.join("assistant/conversations.json")).unwrap(),original);assert_eq!(fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap(),original);
    assert_eq!(fs::metadata(paths.data.join("inbox/example.txt")).unwrap().mode()&0o7777,0o600);assert!(matches!(h.claim(&layout),Err(Error::Busy)));
}
#[test]fn both_shared_and_distinct_old_roots_keep_source_metadata_stable_during_copy(){
    let t=Temp::new();let root=t.0.join("coucou");let layout=Layout::new(root.clone(),root.clone()).unwrap();t.file(&root.join("settings.json"),b"{}");t.file(&root.join("assistant/conversations.json"),br#"{"version":4,"pets":{},"archives":{}}"#);t.file(&root.join("inbox/file.txt"),b"a");
    let h=Harness::new();let publication=begin(&h,&layout);let before=fs::metadata(&root).unwrap();let live=publication.publish().unwrap();let after=fs::metadata(&root).unwrap();
    assert_eq!((before.ino(),before.mtime(),before.mtime_nsec(),before.ctime(),before.ctime_nsec()),(after.ino(),after.mtime(),after.mtime_nsec(),after.ctime(),after.ctime_nsec()));live.current().unwrap();
}
#[test]fn interruption_after_copy_or_each_completion_recovers_under_fresh_native_claim(){
    for checkpoint in[Step::Copied,Step::ConfigComplete,Step::DataComplete,Step::Admitted]{
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let failure=failed(begin(&h,&layout).publish_using(|s|if s==checkpoint{Err(Error::Unavailable)}else{Ok(())}));
        assert_eq!(failure.error,Error::Unavailable);assert!(failure.pair.is_some());
        if checkpoint!=Step::Admitted{assert_eq!(inspect(&layout).unwrap(),State::Incomplete);assert_inactive(&h,&layout);}
        let live=recover_complete(h.claim(&layout).unwrap(),layout.clone()).unwrap();live.current().unwrap();assert_eq!(fs::read(live.paths().unwrap().data.join("inbox/example.txt")).unwrap(),b"private attachment");
        assert!(layout.legacy_data.join("inbox/example.txt").exists());
    }
}
#[test]fn missing_or_partial_payload_is_retained_and_never_admitted_by_recovery(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let failure=failed(begin(&h,&layout).publish_using(|s|if s==Step::BeforeCopy(1){Err(Error::Unavailable)}else{Ok(())}));assert!(failure.pair.is_some());
    let kept=layout.config().join(PAYLOAD).join("pets.json");assert!(kept.exists());let bytes=fs::read(&kept).unwrap();
    assert!(recover_complete(h.claim(&layout).unwrap(),layout.clone()).is_err());assert_eq!(fs::read(kept).unwrap(),bytes);assert!(!layout.config().join(ADMITTED).exists());assert_inactive(&h,&layout);
}
#[test]fn a_half_created_pair_is_visible_as_incomplete_and_never_overwritten(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let failure=failed(Publication::begin_using(h.claim(&layout).unwrap(),layout.clone(),|s|if s==Step::ConfigCreated{Err(Error::Unavailable)}else{Ok(())}));
    assert!(failure.pair.is_some());let header=fs::read(layout.config().join(HEADER)).unwrap();assert_eq!(inspect(&layout).unwrap(),State::Incomplete);
    assert_eq!(failed(Publication::begin(h.claim(&layout).unwrap(),layout.clone())).error,Error::TargetExists);assert_eq!(fs::read(layout.config().join(HEADER)).unwrap(),header);assert!(!layout.data().exists());
}
#[test]fn source_mutation_or_native_ownership_loss_before_effect_prevents_admission(){
    for mutation in["source","owner"]{
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let publication=begin(&h,&layout);let mut changed=false;
        let failure=failed(publication.publish_using(|s|{if !changed&&matches!(s,Step::BeforeCopy(0)){changed=true;if mutation=="owner"{h.release();}else{fs::write(layout.legacy_config.join("pets.json"),br#"{"version":1,"pets":{"changed":{}}}"#).unwrap();}}Ok(())}));
        assert!(changed);assert!(matches!(failure.error,Error::SourceChanged|Error::OwnershipLost));assert!(!layout.config().join(ADMITTED).exists());assert!(!layout.config().join(PAYLOAD).join("pets.json").exists());
    }
}
#[test]fn alteration_of_an_already_copied_source_is_caught_by_the_final_whole_profile_check(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let original=fs::read(layout.legacy_config.join("pets.json")).unwrap();let mut changed=false;
    let failure=failed(begin(&h,&layout).publish_using(|s|{if s==Step::BeforeCopy(2){fs::write(layout.legacy_config.join("pets.json"),br#"{"version":1,"pets":{"changed":{}}}"#).unwrap();changed=true;}Ok(())}));
    assert!(changed);assert_eq!(failure.error,Error::SourceChanged);assert_eq!(fs::read(layout.config().join(PAYLOAD).join("pets.json")).unwrap(),original);assert!(!layout.config().join(ADMITTED).exists());assert_inactive(&h,&layout);
}
#[test]fn corrupted_or_foreign_copy_is_preserved_and_refused_before_completion(){
    for mutation in["changed","foreign","link"]{
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let copy=layout.data().join(PAYLOAD).join("inbox/example.txt");
        let failure=failed(begin(&h,&layout).publish_using(|s|{if s==Step::Copied{match mutation{
            "changed"=>fs::write(&copy,b"changed output").unwrap(),
            "foreign"=>t.file(&layout.data().join(PAYLOAD).join("inbox/foreign.txt"),b"preserve"),
            "link"=>{fs::remove_file(&copy).unwrap();symlink(t.0.join("outside"),&copy).unwrap();},_=>unreachable!()}}Ok(())}));
        assert!(matches!(failure.error,Error::SourceChanged|Error::UnsafePath));assert!(!layout.config().join(ADMITTED).exists());assert!(layout.data().exists());
        assert!(recover_complete(h.claim(&layout).unwrap(),layout.clone()).is_err());
    }
}
#[test]fn changed_old_source_cannot_silently_reinterpret_an_unadmitted_snapshot(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();failed(begin(&h,&layout).publish_using(|s|if s==Step::ConfigComplete{Err(Error::Unavailable)}else{Ok(())}));
    fs::write(layout.legacy_data.join("inbox/example.txt"),b"new original bytes").unwrap();assert!(recover_complete(h.claim(&layout).unwrap(),layout.clone()).is_err());
    assert_eq!(fs::read(layout.data().join(PAYLOAD).join("inbox/example.txt")).unwrap(),b"private attachment");assert_eq!(fs::read(layout.legacy_data.join("inbox/example.txt")).unwrap(),b"new original bytes");assert!(!layout.config().join(ADMITTED).exists());
}
#[test]fn live_payload_changes_survive_restart_without_reimport_or_initial_hash_comparison(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let live=active(&h,&layout);let paths=live.paths().unwrap();
    let newer=br#"{"version":5,"pets":{},"archives":{}}"#;fs::write(paths.data.join("assistant/conversations.json"),newer).unwrap();t.file(&paths.data.join("inbox/new-attachment"),b"new data");live.current().unwrap();drop(live);
    fs::write(layout.legacy_config.join("settings.json"),b"{\"oldVersionChanged\":true}").unwrap();
    let reopened=Active::open(h.claim(&layout).unwrap(),layout.clone()).unwrap();assert_eq!(fs::read(reopened.paths().unwrap().data.join("assistant/conversations.json")).unwrap(),newer);assert!(reopened.paths().unwrap().data.join("inbox/new-attachment").exists());
}
#[test]fn an_unselected_old_log_added_after_copy_does_not_invalidate_equivalent_completed_data(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();failed(begin(&h,&layout).publish_using(|s|if s==Step::ConfigComplete{Err(Error::Unavailable)}else{Ok(())}));
    t.file(&layout.legacy_data.join("coucou.log"),b"new native log");
    let live=recover_complete(h.claim(&layout).unwrap(),layout.clone()).unwrap();assert_eq!(fs::read(live.paths().unwrap().data.join("inbox/example.txt")).unwrap(),b"private attachment");assert_eq!(fs::read(layout.legacy_data.join("coucou.log")).unwrap(),b"new native log");
}
#[test]fn unsupported_old_format_is_refused_before_creating_inactive_destinations(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();fs::write(layout.legacy_data.join("assistant/conversations.json"),br#"{"version":5,"pets":{},"archives":{}}"#).unwrap();
    assert_eq!(failed(Publication::begin(h.claim(&layout).unwrap(),layout.clone())).error,Error::UnsupportedProfile);assert!(!layout.config().exists());assert!(!layout.data().exists());
}
#[test]fn mismatched_pair_admission_and_unsafe_control_records_cannot_open_active_paths(){
    for mutation in["pair","admission","unprivate","extra"]{
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let live=active(&h,&layout);drop(live);
        match mutation{
            "pair"=>{let p=layout.data().join(HEADER);let mut record:Header=parse(&fs::read(&p).unwrap()).unwrap();record.pair="0".repeat(32);fs::write(p,encode(&record).unwrap()).unwrap();},
            "admission"=>fs::write(layout.config().join(ADMITTED),b"{}").unwrap(),
            "unprivate"=>fs::set_permissions(layout.config().join(COMPLETE),fs::Permissions::from_mode(0o644)).unwrap(),
            "extra"=>t.file(&layout.data().join("foreign-control"),b"preserve"),_=>unreachable!(),
        }
        assert!(Active::open(h.claim(&layout).unwrap(),layout.clone()).is_err(),"{mutation}");assert!(layout.data().exists());
    }
}
#[test]fn raw_attachment_names_and_empty_directories_survive_publication(){
    let t=Temp::new();let layout=t.basic();let raw=std::ffi::OsString::from_vec(vec![b'a',0xff]);t.file(&layout.legacy_data.join("inbox/folder").join(&raw),b"raw");fs::create_dir(layout.legacy_data.join("inbox/empty")).unwrap();
    let h=Harness::new();let live=active(&h,&layout);let data=live.paths().unwrap().data;assert_eq!(fs::read(data.join("inbox/folder").join(raw)).unwrap(),b"raw");assert!(data.join("inbox/empty").is_dir());
}
#[test]fn config_and_data_publication_crosses_two_real_filesystems_without_partial_admission(){
    let t=Temp::new();let old=t.basic();let shm=std::path::PathBuf::from("/dev/shm").join(format!("coucou-profile-pair-{}",crate::assistant_memory::new_id()));fs::DirBuilder::new().mode(0o700).create(&shm).unwrap();let shm=Temp(shm);
    let data=shm.0.join("coucou");t.file(&data.join("assistant/conversations.json"),br#"{"version":4,"pets":{},"archives":{}}"#);t.file(&data.join("inbox/example.txt"),b"separate filesystem");
    let layout=Layout::new(old.legacy_config,data).unwrap();assert_ne!(fs::metadata(&layout.legacy_config).unwrap().dev(),fs::metadata(&layout.legacy_data).unwrap().dev());
    let h=Harness::new();failed(begin(&h,&layout).publish_using(|s|if s==Step::ConfigComplete{Err(Error::Unavailable)}else{Ok(())}));assert_inactive(&h,&layout);
    let live=recover_complete(h.claim(&layout).unwrap(),layout).unwrap();assert_eq!(fs::read(live.paths().unwrap().data.join("inbox/example.txt")).unwrap(),b"separate filesystem");
}
