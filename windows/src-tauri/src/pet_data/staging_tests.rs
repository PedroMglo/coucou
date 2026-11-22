use crate::pet_data::Layout;
use super::*;
use crate::pet_data::tests::Temp;
use std::{fs, os::unix::fs::{symlink, DirBuilderExt, PermissionsExt}};

fn workspace(t:&Temp)->PathBuf{let path=t.0.join("copy-workspace");fs::DirBuilder::new().mode(0o700).create(&path).unwrap();path}
fn path(workspace:&Path,id:&str)->PathBuf{workspace.join(job_name(id).unwrap())}
fn prepared(t:&Temp)->(Layout,PathBuf,Prepared){let layout=t.basic();let workspace=workspace(t);let review=LegacyReview::inspect(layout.clone()).unwrap();let output=prepare(review,&workspace).unwrap();(layout,workspace,output)}
fn failed(result:std::result::Result<Prepared,Failure>)->Failure{match result{Err(f)=>f,Ok(_)=>panic!("expected interrupted/refused preparation")}}

#[test]fn known_legacy_history_backups_are_copied_as_inactive_bytes(){
    let t=Temp::new();let layout=t.basic();let name="pre-mochi14-20261019-020937";
    let relative=Path::new("assistant").join(name).join("conversations.json");let original=br#"{"version":999,"data":"opaque old snapshot"}"#;
    t.file(&layout.legacy_data.join(&relative),original);let workspace=workspace(&t);
    let copied=prepare(LegacyReview::inspect(layout.clone()).unwrap(),&workspace).unwrap();copied.verify().unwrap();
    assert_eq!(fs::read(path(&workspace,copied.id()).join("data").join(relative)).unwrap(),original);
    assert_eq!(inspect(&workspace,copied.id()).unwrap().phase,Phase::VerifiedData);
    assert_eq!(fs::read(layout.legacy_data.join("assistant").join(name).join("conversations.json")).unwrap(),original);
    for bad in ["pre-mochi14","pre-mochix-20261019-020937","pre-mochi14-20261019-020937-extra","unrelated"]{
        assert!(!inventory::legacy_history_backup(OsStr::new(bad)));
    }
}
#[test]fn active_usage_owner_lock_is_validated_but_never_copied_acquired_or_replaced(){
    let t=Temp::new();let layout=t.basic();let source=layout.legacy_data.join("assistant/metrics/.capture-owner-lock");t.file(&source,b"");
    let held=std::fs::OpenOptions::new().read(true).write(true).open(&source).unwrap();held.try_lock().unwrap();
    let before=std::os::unix::fs::MetadataExt::ino(&held.metadata().unwrap());let workspace=workspace(&t);
    let copied=prepare(LegacyReview::inspect(layout.clone()).unwrap(),&workspace).unwrap();copied.verify().unwrap();
    assert!(!path(&workspace,copied.id()).join("data/assistant/metrics/.capture-owner-lock").exists());
    assert_eq!(std::os::unix::fs::MetadataExt::ino(&fs::metadata(&source).unwrap()),before);assert_eq!(fs::read(&source).unwrap(),b"");
    let other=std::fs::OpenOptions::new().read(true).write(true).open(&source).unwrap();assert!(matches!(other.try_lock(),Err(std::fs::TryLockError::WouldBlock)));
    fs::write(&source,b"unknown lock data").unwrap();assert!(matches!(LegacyReview::inspect(layout),Err(Error::UnsafePath)));
}

#[test]
#[ignore="explicit opt-in legacy app-data backup/inspection; filesystem data only, no app/agent execution"]
fn native_known_legacy_data_backup_acceptance(){
    assert_eq!(std::env::var("COUCOU_NATIVE_LEGACY_DATA_BACKUP").as_deref(),Ok("1"));
    let workspace=PathBuf::from(std::env::var_os("COUCOU_NATIVE_BACKUP_WORKSPACE").expect("private workspace required"));
    match std::env::var("COUCOU_NATIVE_BACKUP_MODE").as_deref(){
        Ok("capture")=>{
            let config=PathBuf::from(std::env::var_os("COUCOU_NATIVE_BACKUP_CONFIG").expect("legacy config root required"));
            let data=PathBuf::from(std::env::var_os("COUCOU_NATIVE_BACKUP_DATA").expect("legacy data root required"));
            let layout=Layout::new(config,data).expect("invalid legacy layout");
            let review=LegacyReview::inspect(layout).expect("legacy data review refused");
            let expected=review.summary();
            let copied=prepare(review,&workspace).unwrap_or_else(|failure|panic!("native data copy refused: {:?}",failure.error));
            assert_eq!(copied.verify().expect("native source/readback verification failed"),expected);
            let audit=inspect(&workspace,copied.id()).expect("native copied data inspection failed");
            assert_eq!(audit.phase,Phase::VerifiedData);assert_eq!(audit.expected,expected);
            println!("COUCOU_NATIVE_LEGACY_DATA_BACKUP {}",serde_json::json!({"mode":"capture","candidateId":copied.id(),"summary":expected,"verifiedData":true,"dataOnly":true,"sourceChanges":false}));
        }
        Ok("inspect")=>{
            let id=std::env::var("COUCOU_NATIVE_BACKUP_CANDIDATE").expect("candidate id required");
            let audit=inspect(&workspace,&id).expect("native independent copy inspection failed");
            assert_eq!(audit.phase,Phase::VerifiedData);
            // An Inspection cannot reconstruct Prepared, Claim or permission.
            println!("COUCOU_NATIVE_LEGACY_DATA_BACKUP {}",serde_json::json!({"mode":"inspect","candidateId":id,"summary":audit.expected,"verifiedData":true,"dataOnly":true,"nativeAuthorityRecovered":false}));
        }
        _=>panic!("unknown native backup acceptance mode"),
    }
}

#[test] fn kernel_mount_identity_is_available_and_stable_for_pinned_private_directories(){
    let t=Temp::new();let workspace=workspace(&t);let a=Dir::root(&workspace).unwrap();let b=Dir::root(&workspace).unwrap();
    let first=private_fs::location(&a.node().unwrap().file).unwrap();let second=private_fs::location(&b.node().unwrap().file).unwrap();
    assert_eq!(first,second);assert_ne!(first.mount,0);a.check().unwrap();
}
#[test] fn same_filesystem_mount_aliases_cannot_claim_to_be_disjoint_by_path_alone(){
    let original=Location{device:10,inode:20,mount:30};
    assert_eq!(disjoint_location(original,original),Err(Error::UnsafePath));
    assert_eq!(disjoint_location(Location{mount:31,..original},original),Err(Error::UnsafePath));
    assert_eq!(disjoint_location(Location{inode:21,mount:31,..original},original),Err(Error::UnsafePath));
    assert_eq!(disjoint_location(Location{inode:21,..original},original),Ok(()));
    assert_eq!(disjoint_location(Location{device:11,mount:31,..original},original),Ok(()));
}

#[test] fn copies_only_admitted_data_with_private_permissions_and_exact_original_bytes(){
    let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);
    t.file(&layout.legacy_config.join("unknown-secrets/token"),b"excluded");t.file(&layout.legacy_data.join("Models/model.bin"),b"shared");
    t.file(&layout.legacy_config.join("sounds/custom.ogg"),b"original sound bytes");
    t.file(&layout.legacy_data.join("Notes/note.md"),b"keep note");t.file(&layout.legacy_data.join("Flow/capture.txt"),b"keep flow");
    t.file(&layout.legacy_data.join("assistant/metrics/ollama-usage-v1.json"),b"{}");
    let original=fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap();
    let review=LegacyReview::inspect(layout.clone()).unwrap();let expected=review.summary();
    let output=prepare(review,&workspace).unwrap();let root=path(&workspace,output.id());
    assert_eq!(output.verify().unwrap(),expected);assert_eq!(inspect(&workspace,output.id()).unwrap().phase,Phase::VerifiedData);
    for name in ["settings.json","pets.json","sounds/custom.ogg"]{assert_eq!(fs::read(root.join("config").join(name)).unwrap(),fs::read(layout.legacy_config.join(name)).unwrap());}
    for name in ["assistant/conversations.json","assistant/metrics/ollama-usage-v1.json","inbox/example.txt","Notes/note.md","Flow/capture.txt"]{assert_eq!(fs::read(root.join("data").join(name)).unwrap(),fs::read(layout.legacy_data.join(name)).unwrap());}
    assert!(!root.join("data/Models").exists());assert!(!root.join("config/unknown-secrets").exists());
    assert_eq!(fs::metadata(&root).unwrap().mode()&0o7777,0o700);assert_eq!(fs::metadata(root.join("data/inbox/example.txt")).unwrap().mode()&0o7777,0o600);
    assert_eq!(fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap(),original);
    assert!(!layout.config().exists());assert!(!layout.data().exists());
}
#[test] fn raw_filename_bytes_empty_folders_and_historical_paths_are_preserved(){
    let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);
    let raw=OsString::from_vec(vec![b'a',0xff]);t.file(&layout.legacy_data.join("inbox/pasta ç").join(&raw),b"non-UTF8");
    fs::create_dir(layout.legacy_data.join("inbox/empty")).unwrap();
    let historical=br#"{"version":4,"pets":{},"archives":{},"historicalPath":"/old/coucou/inbox/example.txt"}"#;
    fs::write(layout.legacy_data.join("assistant/conversations.json"),historical).unwrap();
    let output=prepare(LegacyReview::inspect(layout).unwrap(),&workspace).unwrap();let root=path(&workspace,output.id());
    assert_eq!(fs::read(root.join("data/inbox/pasta ç").join(raw)).unwrap(),b"non-UTF8");
    assert!(root.join("data/inbox/empty").is_dir());assert_eq!(fs::read(root.join("data/assistant/conversations.json")).unwrap(),historical);
    output.verify().unwrap();
}
#[test] fn interruptions_keep_id_and_partial_data_and_only_ready_can_verify(){
    for checkpoint in [Step::Intent,Step::Node(0),Step::Node(1),Step::Node(2),Step::Manifest,Step::Ready]{
        let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);
        let failure=failed(prepare_using(LegacyReview::inspect(layout.clone()).unwrap(),&workspace,|step|if step==checkpoint{Err(Error::Unavailable)}else{Ok(())}));
        assert_eq!(failure.error,Error::Unavailable);let id=failure.candidate_id.unwrap();assert!(path(&workspace,&id).exists());
        let inspection=inspect(&workspace,&id).unwrap();assert_eq!(inspection.phase,if checkpoint==Step::Ready{Phase::VerifiedData}else{Phase::Incomplete},"{checkpoint:?}");
        assert!(layout.legacy_data.join("inbox/example.txt").exists());assert!(!layout.config().exists());assert!(!layout.data().exists());
    }
}
#[test] fn interrupted_mid_file_retains_partial_copy_without_ready_or_source_changes(){
    let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);let file=layout.legacy_data.join("inbox/example.txt");
    let original=vec![b'x';192*1024];fs::write(&file,&original).unwrap();
    let failure=failed(prepare_using(LegacyReview::inspect(layout).unwrap(),&workspace,|step|match step{Step::Chunk(_,bytes)if bytes>=64*1024=>Err(Error::Unavailable),_=>Ok(())}));
    let id=failure.candidate_id.unwrap();let root=path(&workspace,&id);
    assert_eq!(inspect(&workspace,&id).unwrap().phase,Phase::Incomplete);assert!(!root.join("ready.json").exists());
    assert_eq!(fs::metadata(root.join("data/inbox/example.txt")).unwrap().len(),64*1024);
    assert_eq!(fs::read(&file).unwrap(),original);
}
#[test] fn source_mutation_during_copy_never_yields_a_ready_candidate(){
    for action in ["editDuringChunk","newEntryAfterCopy"]{
        let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);let source=layout.legacy_data.join("inbox/example.txt");
        fs::write(&source,vec![b'x';192*1024]).unwrap();let mut changed=false;
        let failure=failed(prepare_using(LegacyReview::inspect(layout.clone()).unwrap(),&workspace,|step|{
            if !changed&&((action=="editDuringChunk"&&matches!(step,Step::Chunk(_,bytes)if bytes>=64*1024))||(action=="newEntryAfterCopy"&&matches!(step,Step::Node(0)))){
                changed=true;if action=="editDuringChunk"{fs::write(&source,b"concurrent edit").unwrap();}else{t.file(&layout.legacy_data.join("inbox/new.txt"),b"concurrent addition");}
            }Ok(())
        }));
        assert!(changed);assert_eq!(failure.error,Error::SourceChanged);let id=failure.candidate_id.unwrap();assert!(!path(&workspace,&id).join("ready.json").exists());
        assert_eq!(inspect(&workspace,&id).unwrap().phase,Phase::Incomplete);assert!(!layout.data().exists());
    }
}
#[test] fn copied_payload_is_read_back_before_publishing_the_completion_marker(){
    let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);let mut changed=false;
    let failure=failed(prepare_using(LegacyReview::inspect(layout.clone()).unwrap(),&workspace,|step|{
        if step==Step::Manifest{
            let job=fs::read_dir(&workspace).unwrap().next().unwrap().unwrap().path();
            fs::write(job.join("data/inbox/example.txt"),b"changed after copy").unwrap();changed=true;
        }Ok(())
    }));
    assert!(changed);assert_eq!(failure.error,Error::SourceChanged);let id=failure.candidate_id.unwrap();
    assert!(!path(&workspace,&id).join("ready.json").exists());assert_eq!(inspect(&workspace,&id).unwrap().phase,Phase::Incomplete);
    assert_eq!(fs::read(layout.legacy_data.join("inbox/example.txt")).unwrap(),b"private attachment");
}
#[test] fn changed_corrupt_linked_foreign_or_unprivate_output_is_preserved_and_refused(){
    for change in ["bytes","symlink","hardlink","extra","control","unprivate","ready"]{
        let t=Temp::new();let(_,workspace,output)=prepared(&t);let root=path(&workspace,output.id());let copy=root.join("data/inbox/example.txt");
        match change{
            "bytes"=>fs::write(&copy,b"changed").unwrap(),
            "symlink"=>{fs::remove_file(&copy).unwrap();symlink(t.0.join("foreign"),&copy).unwrap();},
            "hardlink"=>fs::hard_link(&copy,t.0.join("alias")).unwrap(),
            "extra"=>t.file(&root.join("data/inbox/foreign.txt"),b"preserve me"),
            "control"=>t.file(&root.join("foreign-control.json"),b"preserve me"),
            "unprivate"=>fs::set_permissions(&copy,fs::Permissions::from_mode(0o644)).unwrap(),
            "ready"=>fs::write(root.join("ready.json"),b"{}").unwrap(),
            _=>unreachable!(),
        }
        assert!(output.verify().is_err(),"{change}");assert!(inspect(&workspace,output.id()).is_err(),"{change}");assert!(root.exists());
        if change=="extra"{assert_eq!(fs::read(root.join("data/inbox/foreign.txt")).unwrap(),b"preserve me");}
    }
}
#[test] fn byte_identical_replacement_is_readable_data_but_cannot_adopt_original_handle(){
    let t=Temp::new();let(_,workspace,output)=prepared(&t);let copy=path(&workspace,output.id()).join("data/inbox/example.txt");let content=fs::read(&copy).unwrap();
    let _original_inode=fs::File::open(&copy).unwrap();
    fs::remove_file(&copy).unwrap();t.file(&copy,&content);
    assert_eq!(inspect(&workspace,output.id()).unwrap().phase,Phase::VerifiedData);assert!(output.verify().is_err());
}
#[test] fn self_consistent_rewritten_manifest_is_not_native_preparation_authority(){
    let t=Temp::new();let(_,workspace,output)=prepared(&t);let root=path(&workspace,output.id());
    let original=fs::read(root.join("config/settings.json")).unwrap();let mut changed=original.clone();changed[2]=if changed[2]==b'L'{b'l'}else{b'L'}; // same length, valid JSON string key
    fs::write(root.join("config/settings.json"),&changed).unwrap();
    let mut manifest:Manifest=serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    manifest.entries.iter_mut().find(|r|r.area==Area::Config&&r.path==b"settings.json").unwrap().sha256=Some(digest(&changed));
    let bytes=serde_json::to_vec(&manifest).unwrap();fs::write(root.join("manifest.json"),&bytes).unwrap();
    fs::write(root.join("ready.json"),serde_json::to_vec(&Ready{version:1,id:output.id().into(),manifest:digest(&bytes)}).unwrap()).unwrap();
    assert_eq!(inspect(&workspace,output.id()).unwrap().phase,Phase::VerifiedData);assert!(output.verify().is_err());
    assert_eq!(fs::read(output.review.layout.legacy_config.join("settings.json")).unwrap(),original);
}
#[test] fn source_edit_after_preparation_preserves_historical_candidate_but_blocks_live_verification(){
    let t=Temp::new();let(layout,workspace,output)=prepared(&t);fs::write(layout.legacy_data.join("inbox/example.txt"),b"new source").unwrap();
    assert_eq!(output.verify(),Err(Error::SourceChanged));assert_eq!(inspect(&workspace,output.id()).unwrap().phase,Phase::VerifiedData);
    assert_eq!(fs::read(path(&workspace,output.id()).join("data/inbox/example.txt")).unwrap(),b"private attachment");
}
#[test] fn workspace_overlap_symlink_or_unsafe_permissions_are_refused_before_output(){
    for change in ["overlap","symlink","unprivate"]{
        let t=Temp::new();let layout=t.basic();let workspace=workspace(&t);
        let target=match change{
            "overlap"=>layout.legacy_data.clone(),
            "symlink"=>{let link=t.0.join("alias");symlink(&workspace,&link).unwrap();link},
            "unprivate"=>{fs::set_permissions(&workspace,fs::Permissions::from_mode(0o755)).unwrap();workspace.clone()},
            _=>unreachable!(),
        };
        let failure=failed(prepare(LegacyReview::inspect(layout).unwrap(),&target));assert_eq!(failure.error,Error::UnsafePath);assert!(failure.candidate_id.is_none());assert_eq!(fs::read_dir(&workspace).unwrap().count(),0);
    }
}
#[test] fn empty_source_profile_copies_empty_data_without_creating_legacy_or_live_roots(){
    let t=Temp::new();let layout=t.layout();let workspace=workspace(&t);let output=prepare(LegacyReview::inspect(layout.clone()).unwrap(),&workspace).unwrap();
    assert_eq!(output.verify().unwrap(),Summary{files:0,directories:0,bytes:0,excluded_roots:0});
    assert_eq!(inspect(&workspace,output.id()).unwrap().phase,Phase::VerifiedData);
    assert!(!layout.legacy_data.exists());assert!(!layout.legacy_config.exists());assert!(!layout.data().exists());assert!(!layout.config().exists());
}

#[test]fn current_profile_partial_copy_keeps_its_id_and_cannot_be_mistaken_for_a_complete_backup(){
    use crate::pet_data::{current_profile::Review,publication::Publication,startup::tests::Harness};
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let active=Publication::begin(h.claim(&layout).unwrap(),layout).ok().unwrap().publish().ok().unwrap();let work=t.0.join("current-copy-work");fs::create_dir(&work).unwrap();fs::set_permissions(&work,fs::Permissions::from_mode(0o700)).unwrap();
    let review=Review::capture(active,&work).unwrap();let failure=prepare_using(review,&work,|step|if matches!(step,Step::Node(0)){Err(Error::Unavailable)}else{Ok(())}).err().unwrap();let id=failure.candidate_id.unwrap();let root=work.join(format!("current-profile-{id}"));assert!(root.is_dir());assert!(!root.join("ready.json").exists());assert_eq!(inspect_current(&work,&id).unwrap().phase,Phase::Incomplete);assert!(inspect(&work,&id).is_err());
}
