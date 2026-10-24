//! Execute pinned historical source against synthetic temporary profiles only.
use super::{tests::Temp, Layout};
use std::{path::Path, time::{Duration, SystemTime}, fs::{self, File, FileTimes}};
#[allow(dead_code)]
#[path="fixtures/legacy30_memory.rs"] mod memory;
mod inbox {
    use super::{Duration, SystemTime, Path};
    include!("fixtures/legacy30_sweep.rs");
    pub(super) fn run(dir: &Path) { sweep(dir); }
}
fn aged(dir: &Path) {
    File::open(dir).unwrap().set_times(FileTimes::new().set_modified(SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60))).unwrap();
}
fn sha(bytes: &[u8]) -> String { ring::digest::digest(&ring::digest::SHA256, bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect() }

#[test] fn frozen_source_fixtures_match_provenance_without_legacy_test_modules() {
    let provenance: serde_json::Value = serde_json::from_str(include_str!("fixtures/PROVENANCE.json")).unwrap();
    let fixtures = [include_bytes!("fixtures/legacy30_memory.rs").as_slice(), include_bytes!("fixtures/legacy30_sweep.rs").as_slice()];
    for (fixture, record) in fixtures.into_iter().zip(provenance["fixtures"].as_array().unwrap()) {
        assert_eq!(sha(fixture), record["fixtureSha256"].as_str().unwrap());
        assert!(!String::from_utf8_lossy(fixture).contains("#[cfg(test)]"));
    }
}
#[test] fn actual_legacy_reader_accepts_v1_through_v4_and_refuses_v5_v6_without_writing() {
    let t = Temp::new(); let path = t.0.join("conversations.json");
    for version in 1..=6 {
        let bytes = format!("{{\"version\":{version},\"pets\":{{}},\"archives\":{{}}}}").into_bytes();
        fs::write(&path, &bytes).unwrap();
        assert_eq!(memory::load(&path).is_ok(), version <= 4);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(crate::assistant_memory::load(&path).is_ok());
    }
}
#[test] fn inactive_history_cannot_hide_behind_an_older_legacy_memory_version() {
    for version in [4,5,6] {
        let t=Temp::new();let layout=t.basic();let path=layout.legacy_data.join("assistant/conversations.json");
        let bytes=serde_json::to_vec(&serde_json::json!({"version":version,"pets":{},"archives":{},"unavailableHistory":{"format":1,"pets":{},"archives":{},"fields":{}}})).unwrap();
        fs::write(&path,&bytes).unwrap();
        assert!(matches!(super::LegacyReview::inspect(layout),Err(super::Error::UnsupportedProfile)));
        assert_eq!(fs::read(path).unwrap(),bytes);
    }
}
#[test] fn memory_format_refusal_does_not_protect_new_quarantine_from_actual_old_sweep() {
    let t = Temp::new(); let layout = t.layout();
    let memory_path = layout.legacy_data.join("assistant/conversations.json");
    t.file(&memory_path, br#"{"version":5,"pets":{},"archives":{}}"#);
    let old_control = layout.legacy_data.join("inbox/.coucou-purge-v1");
    t.file(&old_control.join("transaction/payload/important.txt"), b"would be deleted by old sweep");
    let protected = layout.data().join("inbox/.coucou-purge-v1");
    t.file(&protected.join("transaction/payload/important.txt"), b"new isolated state");
    let protected_control = layout.config().join("settings.json"); t.file(&protected_control, b"{}");
    aged(&old_control); aged(layout.data()); aged(&protected);
    assert!(memory::load(&memory_path).is_err());
    inbox::run(&layout.legacy_data.join("inbox"));
    assert!(!old_control.exists());
    assert_eq!(fs::read(protected.join("transaction/payload/important.txt")).unwrap(), b"new isolated state");
    assert!(protected_control.exists()); assert!(memory_path.exists());
}
#[test] fn pinned_old_sweep_removes_aged_copies_and_preserves_recent_copies_and_sibling_data() {
    let t = Temp::new(); let root = t.0.join("coucou"); let layout = Layout::new(root.clone(), root).unwrap();
    let inbox_path = layout.legacy_data.join("inbox");
    t.file(&inbox_path.join("aged.txt"), b"old"); t.file(&inbox_path.join("recent.txt"), b"new");
    t.file(&layout.data().join("inbox/aged.txt"), b"protected");
    aged(&inbox_path.join("aged.txt")); aged(layout.data());
    inbox::run(&inbox_path);
    assert!(!inbox_path.join("aged.txt").exists()); assert!(inbox_path.join("recent.txt").exists());
    assert_eq!(fs::read(layout.data().join("inbox/aged.txt")).unwrap(), b"protected");
}
