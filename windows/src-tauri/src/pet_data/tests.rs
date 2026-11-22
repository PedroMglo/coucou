use super::*;
use std::{fs, os::unix::{ffi::OsStringExt, fs::{symlink, DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt}}};

pub(super) struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!("coucou-data-review-{}", crate::assistant_memory::new_id()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap(); Self(path)
    }
    pub fn layout(&self) -> Layout { Layout::new(self.0.join("config/coucou"), self.0.join("data/coucou")).unwrap() }
    pub fn file(&self, path: &Path, bytes: &[u8]) {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(path.parent().unwrap()).unwrap();
        let mut options = fs::OpenOptions::new(); options.write(true).create_new(true).mode(0o600);
        use std::io::Write;
        options.open(path).unwrap().write_all(bytes).unwrap();
    }
    pub fn basic(&self) -> Layout {
        let layout = self.layout();
        self.file(&layout.legacy_config.join("settings.json"), br#"{"language":"pt","activeIntegrations":["integration_codex"]}"#);
        self.file(&layout.legacy_config.join("pets.json"), br#"{"version":1,"pets":{}}"#);
        self.file(&layout.legacy_data.join("assistant/conversations.json"), br#"{"version":4,"pets":{},"archives":{}}"#);
        self.file(&layout.legacy_data.join("inbox/example.txt"), b"private attachment");
        layout
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test] fn review_is_read_only_bounded_and_does_not_traverse_shared_assets_or_unknown_roots() {
    let t = Temp::new(); let layout = t.basic();
    t.file(&layout.legacy_data.join("Models/not-a-profile-secret"), b"leave alone");
    t.file(&layout.legacy_config.join("unknown-token-store/secret"), b"never export this");
    symlink("/not/read", layout.legacy_data.join("bin")).unwrap();
    let before = fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap();
    let source = fs::metadata(&layout.legacy_data).unwrap();
    let review = LegacyReview::inspect(layout.clone()).unwrap();
    assert_eq!(review.summary().files, 4); assert_eq!(review.summary().excluded_roots, 3);
    assert!(review.summary().bytes < 1024); review.ensure_current().unwrap();
    assert_eq!(fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap(), before);
    let after = fs::metadata(&layout.legacy_data).unwrap();
    assert_eq!((source.ino(), source.mtime(), source.ctime()), (after.ino(), after.mtime(), after.ctime()));
    assert!(!layout.config().exists()); assert!(!layout.data().exists());
}
#[test] fn source_edit_replacement_new_file_and_new_target_invalidate_original_review() {
    for action in ["edit", "replace", "newFile", "target"] {
        let t = Temp::new(); let layout = t.basic(); let review = LegacyReview::inspect(layout.clone()).unwrap();
        let file = layout.legacy_data.join("inbox/example.txt");
        match action {
            "edit" => fs::write(&file, b"changed bytes").unwrap(),
            "replace" => { fs::remove_file(&file).unwrap(); t.file(&file, b"private attachment"); },
            "newFile" => t.file(&layout.legacy_data.join("inbox/other.txt"), b"new"),
            "target" => fs::create_dir(layout.data()).unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(review.ensure_current(), Err(Error::SourceChanged), "{action}");
    }
}
#[test] fn v5_history_runtime_authority_and_quarantine_cannot_enter_legacy_import() {
    for kind in ["v5", "historyWithV4", "authority", "goals", "contexts", "quarantine", "attachments", "unknownAssistant", "unknownMetrics"] {
        let t = Temp::new(); let layout = t.basic();
        match kind {
            "v5" => fs::write(layout.legacy_data.join("assistant/conversations.json"), br#"{"version":5,"pets":{}}"#).unwrap(),
            "historyWithV4" => fs::write(layout.legacy_data.join("assistant/conversations.json"), br#"{"version":4,"pets":{"codex":{"attachmentHistory":[]}}}"#).unwrap(),
            "authority" => t.file(&layout.legacy_config.join("pet-runtime-consent-v1/key.json"), b"not copied"),
            "goals" => t.file(&layout.legacy_config.join("pet-runtime-v1/goal.json"), b"not copied"),
            "contexts" => t.file(&layout.legacy_config.join("pet-runtime-contexts-v1/private.json"), b"not copied"),
            "quarantine" => t.file(&layout.legacy_data.join("inbox/.coucou-purge-v1/protocol"), b"inode-bound"),
            "attachments" => t.file(&layout.legacy_data.join("assistant/attachments/manifest.json"), b"not legacy"),
            "unknownAssistant" => t.file(&layout.legacy_data.join("assistant/future-format/state"), b"unknown"),
            "unknownMetrics" => t.file(&layout.legacy_data.join("assistant/metrics/future-ledger.json"), b"{}"),
            _ => unreachable!(),
        }
        assert!(matches!(LegacyReview::inspect(layout.clone()), Err(Error::UnsupportedProfile)), "{kind}");
        assert!(!layout.config().exists()); assert!(!layout.data().exists());
    }
}
#[test] fn symlink_hardlink_fifo_and_writable_source_are_refused_without_opening_payload() {
    for kind in ["symlink", "hardlink", "fifo", "writable", "ancestorLink"] {
        let t = Temp::new(); let layout = t.basic(); let target = layout.legacy_data.join("inbox/example.txt");
        match kind {
            "symlink" => { fs::remove_file(&target).unwrap(); symlink(t.0.join("outside"), &target).unwrap(); },
            "hardlink" => fs::hard_link(&target, t.0.join("outside")).unwrap(),
            "fifo" => { fs::remove_file(&target).unwrap(); use std::os::unix::ffi::OsStrExt; let name=std::ffi::CString::new(target.as_os_str().as_bytes()).unwrap(); assert_eq!(unsafe {libc::mkfifo(name.as_ptr(), 0o600)}, 0); },
            "writable" => fs::set_permissions(&target, fs::Permissions::from_mode(0o666)).unwrap(),
            "ancestorLink" => { fs::rename(t.0.join("data"), t.0.join("moved")).unwrap(); symlink(t.0.join("moved"), t.0.join("data")).unwrap(); },
            _ => unreachable!(),
        }
        assert!(matches!(LegacyReview::inspect(layout), Err(Error::UnsafePath)), "{kind}");
    }
}
#[test] fn missing_roots_are_reviewable_but_appearance_invalidates_review() {
    let t = Temp::new(); let layout = t.layout(); let review = LegacyReview::inspect(layout.clone()).unwrap();
    assert_eq!(review.summary(), Summary {files:0, directories:0, bytes:0, excluded_roots:0});
    review.ensure_current().unwrap(); t.file(&layout.legacy_config.join("settings.json"), b"{}");
    assert_eq!(review.ensure_current(), Err(Error::SourceChanged));
}
#[test] fn shared_xdg_roots_work_and_new_roots_cannot_overlap_old_inbox() {
    let t = Temp::new(); let root = t.0.join("coucou"); let layout = Layout::new(root.clone(), root.clone()).unwrap();
    t.file(&root.join("settings.json"), b"{}"); t.file(&root.join("inbox/a"), b"a");
    assert_ne!(layout.config(), layout.data()); assert_eq!(LegacyReview::inspect(layout).unwrap().summary().files, 2);
    assert_eq!(Layout::new(root.join("inbox/coucou"), root), Err(Error::UnsafePath));
    for path in [PathBuf::from("relative/coucou"), t.0.join("../coucou"), t.0.clone()] {
        assert_eq!(Layout::new(path, t.layout().legacy_data), Err(Error::UnsafePath));
    }
}
#[test] fn non_utf8_attachment_names_are_preserved_and_counted_without_lossy_aliases() {
    let t = Temp::new(); let layout = t.basic();
    let raw = std::ffi::OsString::from_vec(vec![b'a', 0xff]); t.file(&layout.legacy_data.join("inbox").join(raw), b"raw name");
    let review = LegacyReview::inspect(layout).unwrap(); assert_eq!(review.summary().files, 5); review.ensure_current().unwrap();
}
#[test] fn oversized_sparse_file_and_excessive_depth_refuse_without_creating_targets() {
    for kind in ["large", "depth"] {
        let t = Temp::new(); let layout = t.basic();
        if kind == "large" {
            let file = fs::OpenOptions::new().write(true).open(layout.legacy_data.join("inbox/example.txt")).unwrap();
            file.set_len(crate::assistant_memory::MAX_BYTES + 1).unwrap();
        } else {
            let mut path = layout.legacy_data.join("inbox"); for _ in 0..34 { path.push("d"); } t.file(&path.join("leaf"), b"deep");
        }
        assert!(matches!(LegacyReview::inspect(layout.clone()), Err(Error::Limit)), "{kind}");
        assert!(!layout.data().exists());
    }
}

#[test] fn legacy_import_rejects_delivered_memory_even_with_an_old_version_marker() {
    for version in 1..=4 {
        for field in ["native_memory_context", "nativeMemoryContext"] {
            for archived in [false, true] {
                let t=Temp::new();let layout=t.basic();let path=layout.legacy_data.join("assistant/conversations.json");
                let mut c=serde_json::json!({"id":"preserved-cid","messages":[],"created":101,"updated":101});
                c[field]=serde_json::json!({"schemaVersion":1,"petId":"agent_codex","possiblyDeliveredAt":101});
                let disk=if archived{serde_json::json!({"version":version,"pets":{},"archives":{"codex":[c]}})}else{serde_json::json!({"version":version,"pets":{"codex":c},"archives":{}})};
                let bytes=serde_json::to_vec(&disk).unwrap();fs::write(&path,&bytes).unwrap();
                assert!(matches!(LegacyReview::inspect(layout.clone()),Err(Error::UnsupportedProfile)),"{version}/{field}/archive={archived}");
                assert_eq!(fs::read(&path).unwrap(),bytes);assert!(!layout.config().exists());assert!(!layout.data().exists());
            }
        }
    }
}
#[test] fn legacy_import_cannot_silently_exclude_new_runtime_catalogues() {
    for name in ["pet-runtime-results-v1","pet-runtime-planning-v1","pet-runtime-step-consent-v1","pet-runtime-consent-v1.owners-v1","pet-runtime-future-v2"] {
        let t=Temp::new();let layout=t.basic();let state=layout.legacy_config.join(name).join("private-state.json");
        let bytes=b"{\"syntheticRetainedState\":true}";t.file(&state,bytes);
        assert!(matches!(LegacyReview::inspect(layout.clone()),Err(Error::UnsupportedProfile)),"{name}");
        assert_eq!(fs::read(&state).unwrap(),bytes);assert!(!layout.config().exists());assert!(!layout.data().exists());
    }
}
#[test] fn legacy_import_refuses_v7_without_creating_or_rewriting_a_profile() {
    let t=Temp::new();let layout=t.basic();let path=layout.legacy_data.join("assistant/conversations.json");
    let bytes=br#"{"version":7,"pets":{},"archives":{}}"#;fs::write(&path,bytes).unwrap();
    assert!(matches!(LegacyReview::inspect(layout.clone()),Err(Error::UnsupportedProfile)));
    assert_eq!(fs::read(&path).unwrap(),bytes);assert!(!layout.config().exists());assert!(!layout.data().exists());
}
