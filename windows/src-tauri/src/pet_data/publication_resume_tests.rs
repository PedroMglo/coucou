use super::*;
use crate::pet_data::{bootstrap::Session, startup::tests::Harness, tests::Temp};
use std::{fs, os::unix::fs::{MetadataExt, symlink}};

fn interrupted(h: &Harness, layout: &Layout, checkpoint: Step) {
    let publication = Publication::begin(h.claim(layout).unwrap(), layout.clone()).unwrap();
    assert!(publication.publish_using(|step| if step == checkpoint { Err(Error::Unavailable) } else { Ok(()) }).is_err());
}

#[test]
fn bootstrap_resumes_missing_nodes_under_a_fresh_native_claim() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    let original = fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap();
    interrupted(&h, &layout, Step::BeforeCopy(1));
    assert!(!layout.config().join(ADMITTED).exists());
    let session = Session::prepare(h.claim(&layout).unwrap(), layout.clone()).unwrap();
    let paths = session.paths().unwrap();
    assert_eq!(fs::read(paths.data.join("assistant/conversations.json")).unwrap(), original);
    assert_eq!(fs::read(paths.data.join("inbox/example.txt")).unwrap(), b"private attachment");
    assert_eq!(fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap(), original);
    assert!(matches!(h.claim(&layout), Err(Error::Busy)));
}

#[test]
fn partial_attachment_is_appended_without_replacing_its_existing_inode() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    let bytes: Vec<u8> = (0..196_613).map(|n| (n % 251) as u8).collect();
    t.file(&layout.legacy_data.join("inbox/large.bin"), &bytes);
    drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
    let partial = layout.data().join(PAYLOAD).join("inbox/large.bin");
    t.file(&partial, &bytes[..70_001]); let inode = fs::metadata(&partial).unwrap().ino();
    let active = recover(h.claim(&layout).unwrap(), layout.clone()).unwrap();
    active.current().unwrap();
    assert_eq!(fs::metadata(&partial).unwrap().ino(), inode);
    assert_eq!(fs::read(&partial).unwrap(), bytes);
    assert_eq!(fs::read(layout.legacy_data.join("inbox/large.bin")).unwrap(), bytes);
}

#[test]
fn every_existing_conflict_is_checked_before_any_missing_file_is_written() {
    for conflict in ["different", "oversized", "directory", "foreign", "symlink"] {
        let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
        drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
        let copy = layout.data().join(PAYLOAD).join("inbox/example.txt");
        match conflict {
            "different" => t.file(&copy, b"wrong"),
            "oversized" => t.file(&copy, &[b'x'; 100]),
            "directory" => { fs::create_dir_all(&copy).unwrap();
                use std::os::unix::fs::PermissionsExt;
                for p in [copy.parent().unwrap(), copy.as_path()] { fs::set_permissions(p, fs::Permissions::from_mode(0o700)).unwrap(); }
            },
            "foreign" => t.file(&copy.with_file_name("extra.txt"), b"preserve"),
            "symlink" => { t.file(&copy.with_file_name("outside.txt"), b"preserve"); symlink("outside.txt", &copy).unwrap(); },
            _ => unreachable!(),
        }
        assert!(recover(h.claim(&layout).unwrap(), layout.clone()).is_err(), "{conflict}");
        assert!(!layout.config().join(PAYLOAD).join("pets.json").exists(), "wrote before finding {conflict}");
        assert!(!layout.config().join(ADMITTED).exists());
        assert_eq!(fs::read(layout.legacy_data.join("inbox/example.txt")).unwrap(), b"private attachment");
        match conflict {
            "different" => assert_eq!(fs::read(copy).unwrap(), b"wrong"),
            "oversized" => assert_eq!(fs::metadata(copy).unwrap().len(), 100),
            "directory" => assert!(copy.is_dir()),
            "foreign" => assert_eq!(fs::read(copy.with_file_name("extra.txt")).unwrap(), b"preserve"),
            "symlink" => assert!(fs::symlink_metadata(copy).unwrap().file_type().is_symlink()),
            _ => unreachable!(),
        }
    }
}

#[test]
fn changed_source_cannot_redefine_the_pre_copy_intent() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
    let receipt = fs::read(layout.config().join(INTENT)).unwrap();
    fs::write(layout.legacy_data.join("inbox/example.txt"), b"different source bytes").unwrap();
    assert!(matches!(recover(h.claim(&layout).unwrap(), layout.clone()), Err(Error::SourceChanged)));
    assert_eq!(fs::read(layout.config().join(INTENT)).unwrap(), receipt);
    assert!(!layout.config().join(PAYLOAD).join("pets.json").exists());
    assert!(!layout.config().join(ADMITTED).exists());
}

#[test]
fn partial_data_without_a_prior_intent_is_preserved_and_refused() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
    fs::remove_file(layout.config().join(INTENT)).unwrap();
    let partial = layout.data().join(PAYLOAD).join("inbox/example.txt"); t.file(&partial, b"private");
    assert!(recover(h.claim(&layout).unwrap(), layout.clone()).is_err());
    assert_eq!(fs::read(partial).unwrap(), b"private");
    assert!(!layout.config().join(INTENT).exists());
    assert!(!layout.config().join(ADMITTED).exists());
}

#[test]
fn an_empty_half_pair_resumes_with_its_original_pair_identity() {
    for config_present in [true, false] {
        let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
        let at = if config_present { Step::ConfigCreated } else { Step::DataCreated };
        assert!(Publication::begin_using(h.claim(&layout).unwrap(), layout.clone(), |step|
            if step == at { Err(Error::Unavailable) } else { Ok(()) }).is_err());
        if !config_present { fs::remove_dir_all(layout.config()).unwrap(); }
        let present = if config_present { layout.config() } else { layout.data() };
        let header = fs::read(present.join(HEADER)).unwrap();
        let active = recover(h.claim(&layout).unwrap(), layout.clone()).unwrap();
        active.current().unwrap(); assert_eq!(fs::read(present.join(HEADER)).unwrap(), header);
        assert_eq!(fs::read(active.paths().unwrap().data.join("inbox/example.txt")).unwrap(), b"private attachment");
    }
}

#[test]
fn nonempty_or_previously_admitted_half_pairs_are_never_recreated() {
    for previously_active in [false, true] {
        let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
        if previously_active {
            let active = Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap().publish().unwrap();
            drop(active); fs::remove_dir_all(layout.data()).unwrap();
        } else {
            assert!(Publication::begin_using(h.claim(&layout).unwrap(), layout.clone(), |step|
                if step == Step::ConfigCreated { Err(Error::Unavailable) } else { Ok(()) }).is_err());
            t.file(&layout.config().join(PAYLOAD).join("pets.json"), b"preserve partial config");
        }
        let header = fs::read(layout.config().join(HEADER)).unwrap();
        assert!(recover(h.claim(&layout).unwrap(), layout.clone()).is_err());
        assert!(!layout.data().exists()); assert_eq!(fs::read(layout.config().join(HEADER)).unwrap(), header);
        if !previously_active { assert_eq!(fs::read(layout.config().join(PAYLOAD).join("pets.json")).unwrap(), b"preserve partial config"); }
    }
}

#[test]
fn missing_data_after_completion_is_not_filled_by_the_new_resume_route() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    interrupted(&h, &layout, Step::ConfigComplete);
    let gone = layout.data().join(PAYLOAD).join("inbox/example.txt"); fs::remove_file(&gone).unwrap();
    assert!(recover(h.claim(&layout).unwrap(), layout.clone()).is_err());
    assert!(!gone.exists()); assert!(!layout.config().join(ADMITTED).exists());
}

#[test]
fn admitted_payload_is_never_reimported_even_when_legacy_source_changes() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap().publish().unwrap());
    let live = layout.data().join(PAYLOAD).join("inbox/example.txt"); fs::write(&live, b"new live data").unwrap();
    fs::write(layout.legacy_data.join("inbox/example.txt"), b"changed legacy data").unwrap();
    let active = recover(h.claim(&layout).unwrap(), layout).unwrap(); active.current().unwrap();
    assert_eq!(fs::read(live).unwrap(), b"new live data");
}

#[test]
fn lost_native_owner_cannot_resume_missing_data() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
    let claim = h.claim(&layout).unwrap(); h.release();
    assert!(matches!(recover(claim, layout.clone()), Err(Error::OwnershipLost)));
    assert!(!layout.config().join(PAYLOAD).join("pets.json").exists());
    assert!(!layout.config().join(ADMITTED).exists());
}

#[test]
fn prepared_directory_publication_preserves_existing_targets_and_rebinds_success() {
    let t = Temp::new(); let layout = t.basic(); let parent = Dir::parent(&layout.legacy_config).unwrap();
    let staged = parent.mkdir(OsStr::new("prepared-test")).unwrap(); staged.write("ready", b"complete").unwrap();
    let existing = parent.mkdir(OsStr::new("existing-test")).unwrap(); existing.write("keep", b"original").unwrap();
    assert!(matches!(parent.publish_dir(OsStr::new("prepared-test"), OsStr::new("existing-test"), &staged), Err(Error::TargetExists)));
    staged.check().unwrap(); existing.check().unwrap();
    assert_eq!(fs::read(layout.legacy_config.join("existing-test/keep")).unwrap(), b"original");
    let published = parent.publish_dir(OsStr::new("prepared-test"), OsStr::new("published-test"), &staged).unwrap();
    published.check().unwrap(); assert!(!layout.legacy_config.join("prepared-test").exists());
    assert_eq!(fs::read(layout.legacy_config.join("published-test/ready")).unwrap(), b"complete");
}

#[test]
fn older_fully_copied_data_without_an_intent_can_finish_but_is_not_reimported() {
    let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
    interrupted(&h, &layout, Step::Copied); fs::remove_file(layout.config().join(INTENT)).unwrap();
    let copy = layout.data().join(PAYLOAD).join("inbox/example.txt"); let inode = fs::metadata(&copy).unwrap().ino();
    let active = recover(h.claim(&layout).unwrap(), layout.clone()).unwrap(); active.current().unwrap();
    assert_eq!(fs::metadata(copy).unwrap().ino(), inode); assert!(!layout.config().join(INTENT).exists());
}

#[test]
fn interrupted_metadata_stages_are_preserved_but_never_used_as_receipts() {
    for at in [Step::BeforeCopy(1), Step::Copied, Step::ConfigComplete, Step::DataComplete] {
        let t = Temp::new(); let layout = t.basic(); let h = Harness::new(); interrupted(&h, &layout, at);
        let name = format!(".publish-{}", "a".repeat(32));
        let config = layout.config().join(&name); let data = layout.data().join(&name);
        t.file(&config, b"{\"version\":1,\"pair\":\"incomplete"); t.file(&data, b"not even json");
        assert_eq!(inspect(&layout).unwrap(), State::Incomplete);
        assert!(Active::open(h.claim(&layout).unwrap(), layout.clone()).is_err());
        let active = recover(h.claim(&layout).unwrap(), layout.clone()).unwrap(); active.current().unwrap();
        assert_eq!(fs::read(config).unwrap(), b"{\"version\":1,\"pair\":\"incomplete");
        assert_eq!(fs::read(data).unwrap(), b"not even json");
        assert_eq!(fs::read(active.paths().unwrap().data.join("inbox/example.txt")).unwrap(), b"private attachment");
    }
}

#[test]
fn unsafe_or_unbounded_metadata_stages_prevent_writes_and_remain_in_place() {
    use std::os::unix::fs::PermissionsExt;
    for variant in ["permissions", "symlink", "oversized", "many", "name"] {
        let t = Temp::new(); let layout = t.basic(); let h = Harness::new();
        drop(Publication::begin(h.claim(&layout).unwrap(), layout.clone()).unwrap());
        let stage = layout.config().join(format!(".publish-{}", "a".repeat(32)));
        match variant {
            "permissions" => { t.file(&stage, b"keep"); fs::set_permissions(&stage, fs::Permissions::from_mode(0o644)).unwrap(); },
            "symlink" => { symlink("settings.json", &stage).unwrap(); },
            "oversized" => t.file(&stage, &vec![b'x'; MAX_RECORD + 1]),
            "many" => { for n in 0..=MAX_STAGED_RECORDS { t.file(&layout.config().join(format!(".publish-{n:032x}")), b"keep"); } },
            "name" => t.file(&layout.config().join(".publish-not-a-native-nonce"), b"keep"),
            _ => unreachable!(),
        }
        assert!(recover(h.claim(&layout).unwrap(), layout.clone()).is_err(), "{variant}");
        assert!(!layout.config().join(PAYLOAD).join("pets.json").exists());
        assert!(!layout.config().join(ADMITTED).exists());
        if variant != "many" && variant != "name" { assert!(fs::symlink_metadata(stage).is_ok()); }
    }
}
