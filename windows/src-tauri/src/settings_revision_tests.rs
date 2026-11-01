use super::*;
use std::{fs::{self, File}, os::unix::fs::PermissionsExt, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

struct Temp { root: PathBuf, profile: PathBuf, path: PathBuf }
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!("coucou-settings-revision-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root).unwrap(); fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let profile = root.join("profile"); fs::create_dir(&profile).unwrap(); fs::set_permissions(&profile, fs::Permissions::from_mode(0o700)).unwrap();
        let path = profile.join("settings.json"); Self { root, profile, path }
    }
    fn write(&self, volume: f64) -> Vec<u8> {
        let mut s = Settings::default(); s.sound_volume = volume; let bytes = serde_json::to_vec_pretty(&s).unwrap();
        fs::write(&self.path, &bytes).unwrap(); fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600)).unwrap(); bytes
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }

#[test]
fn settings_revision_in_place_change_is_not_overwritten_or_announced() {
    let t = Temp::new(); t.write(0.2); let current = Mutex::new(load_from(&t.path)); let external = t.write(0.7);
    let result = crate::settings::update_with(&current, |next| next.autostart = true,
        |next| save_to(&t.path, next), |_| panic!("stale preferences announced success"));
    assert!(result.is_err()); let actual = current.lock().unwrap(); assert!(!actual.autostart); assert_eq!(actual.sound_volume, 0.2);
    assert_eq!(fs::read(&t.path).unwrap(), external); assert_eq!(fs::read_dir(&t.profile).unwrap().count(), 1);
}

#[test]
fn settings_revision_same_bytes_after_in_place_aba_do_not_renew_the_observation() {
    let t = Temp::new(); let original = t.write(0.2); let mut loaded = load_from(&t.path); let original_time = fs::metadata(&t.path).unwrap().modified().unwrap();
    t.write(0.9); fs::write(&t.path, &original).unwrap();
    File::open(&t.path).unwrap().set_times(fs::FileTimes::new().set_modified(original_time)).unwrap();
    loaded.sound_volume = 0.4; assert!(save_to(&t.path, &loaded).is_err()); assert_eq!(fs::read(&t.path).unwrap(), original);
}

#[test]
fn settings_revision_same_content_in_another_inode_is_not_the_loaded_file() {
    let t = Temp::new(); let bytes = t.write(0.2); let mut loaded = load_from(&t.path);
    let replacement = t.profile.join("replacement"); fs::write(&replacement, &bytes).unwrap(); fs::set_permissions(&replacement, fs::Permissions::from_mode(0o600)).unwrap(); fs::rename(replacement, &t.path).unwrap();
    loaded.sound_volume = 0.4; assert!(save_to(&t.path, &loaded).is_err()); assert_eq!(fs::read(&t.path).unwrap(), bytes);
}

#[test]
fn settings_revision_missing_and_removed_are_distinct_native_observations() {
    let t = Temp::new(); let mut missing = load_from(&t.path); let external = t.write(0.7); missing.sound_volume = 0.4;
    assert!(save_to(&t.path, &missing).is_err()); assert_eq!(fs::read(&t.path).unwrap(), external);
    let mut present = load_from(&t.path); fs::remove_file(&t.path).unwrap(); present.sound_volume = 0.5;
    assert!(save_to(&t.path, &present).is_err()); assert!(!t.path.exists());
    let mut fresh = load_from(&t.path); fresh.sound_volume = 0.6; save_to(&t.path, &fresh).unwrap(); assert_eq!(load_from(&t.path).sound_volume, 0.6);
}

#[test]
fn settings_revision_replaced_parent_is_not_adopted_even_with_identical_bytes() {
    let t = Temp::new(); let original = t.write(0.2); let mut loaded = load_from(&t.path); let parked = t.root.join("parked");
    fs::rename(&t.profile, &parked).unwrap(); fs::create_dir(&t.profile).unwrap(); fs::set_permissions(&t.profile, fs::Permissions::from_mode(0o700)).unwrap(); t.write(0.2);
    loaded.sound_volume = 0.4; assert!(save_to(&t.path, &loaded).is_err());
    assert_eq!(fs::read(&t.path).unwrap(), original); assert_eq!(fs::read(parked.join("settings.json")).unwrap(), original);
}

#[test]
fn settings_revision_legacy_alias_survives_own_write_and_retargeting_is_refused() {
    let t = Temp::new(); t.write(0.2); let target = t.profile.join("dotfile.json"); fs::rename(&t.path, &target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap(); std::os::unix::fs::symlink(&target, &t.path).unwrap();
    let mut loaded = load_from(&t.path); assert_eq!(loaded.sound_volume, 0.2); loaded.sound_volume = 0.3; save_to(&t.path, &loaded).unwrap();
    assert!(fs::symlink_metadata(&t.path).unwrap().file_type().is_symlink()); assert_eq!(load_from(&t.path).sound_volume, 0.3);
    assert_eq!(fs::metadata(&target).unwrap().permissions().mode() & 0o7777, 0o600); assert!(crate::settings::read_initialized(&t.path).is_err());
    let replacement = t.profile.join("replacement.json"); fs::copy(&target, &replacement).unwrap(); let bytes = fs::read(&replacement).unwrap();
    fs::remove_file(&t.path).unwrap(); std::os::unix::fs::symlink(&replacement, &t.path).unwrap(); loaded.sound_volume = 0.5;
    assert!(save_to(&t.path, &loaded).is_err()); assert_eq!(fs::read(&replacement).unwrap(), bytes); assert_eq!(fs::read(&target).unwrap(), bytes);
}

#[test]
fn settings_revision_cloned_native_snapshots_have_one_successful_writer() {
    let t = Temp::new(); t.write(0.2); let mut a = load_from(&t.path); let mut b = a.clone(); a.sound_volume = 0.3; b.sound_volume = 0.4;
    let path_a = t.path.clone(); let path_b = t.path.clone(); let barrier = Arc::new(std::sync::Barrier::new(3)); let ba = barrier.clone(); let bb = barrier.clone();
    let first = std::thread::spawn(move || { ba.wait(); save_to(&path_a, &a) });
    let second = std::thread::spawn(move || { bb.wait(); save_to(&path_b, &b) }); barrier.wait();
    let results = [first.join().unwrap(), second.join().unwrap()]; assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!([0.3, 0.4].contains(&load_from(&t.path).sound_volume)); assert_eq!(fs::read_dir(&t.profile).unwrap().count(), 4);
}

#[test]
fn settings_revision_deserialized_snapshot_never_supplies_or_renews_backend_origin() {
    let t = Temp::new(); t.write(0.2); let current = Mutex::new(load_from(&t.path));
    let incoming = || { let mut raw = serde_json::to_value(Settings::default()).unwrap(); raw["soundVolume"] = 0.3.into(); raw["fileRevision"] = "forged".into(); serde_json::from_value::<Settings>(raw).unwrap() };
    crate::settings::update_with(&current, |next| *next = incoming(), |next| save_to(&t.path, next), |_| {}).unwrap();
    assert_eq!(load_from(&t.path).sound_volume, 0.3); let external = t.write(0.7);
    assert!(crate::settings::update_with(&current, |next| *next = incoming(), |next| save_to(&t.path, next), |_| panic!("browser renewed stale native origin")).is_err());
    assert_eq!(current.lock().unwrap().sound_volume, 0.3); assert_eq!(fs::read(&t.path).unwrap(), external);
}

#[test]
fn settings_revision_strict_reader_and_own_successive_commits_keep_the_old_json_shape() {
    let t = Temp::new(); let original = t.write(0.2); let mut loaded = crate::settings::read_initialized(&t.path).unwrap();
    for volume in [0.3, 0.4] { loaded.sound_volume = volume; save_to(&t.path, &loaded).unwrap(); assert_eq!(crate::settings::read_initialized(&t.path).unwrap().sound_volume, volume); }
    let before: serde_json::Value = serde_json::from_slice(&original).unwrap(); let after: serde_json::Value = serde_json::from_slice(&fs::read(&t.path).unwrap()).unwrap();
    assert_eq!(before.as_object().unwrap().keys().collect::<Vec<_>>(), after.as_object().unwrap().keys().collect::<Vec<_>>());
    assert!(after.get("fileRevision").is_none()); assert_eq!(fs::metadata(&t.path).unwrap().permissions().mode() & 0o7777, 0o600);
}

#[test]
fn settings_revision_temp_substitution_is_preserved_without_overwriting_original() {
    let t = Temp::new(); let original = t.write(0.2); let (witness, _) = preferences_file::read(&t.path, false).unwrap();
    let temp = t.path.with_extension(format!("json.coucou-{}", std::process::id()));
    assert!(witness.commit_for_test(&original, || { fs::remove_file(&temp).unwrap(); fs::write(&temp, b"foreign replacement").unwrap(); }, || {}).is_err());
    assert_eq!(fs::read(&t.path).unwrap(), original); assert_eq!(fs::read(&temp).unwrap(), b"foreign replacement");
}

#[test]
fn settings_revision_before_publication_change_refuses_and_cleans_only_own_temp() {
    let t = Temp::new(); let original = t.write(0.2); let (witness, _) = preferences_file::read(&t.path, false).unwrap();
    let error = witness.commit_for_test(&original, || { t.write(0.7); }, || {}).unwrap_err();
    assert!(!preferences_file::unconfirmed(&error)); assert_eq!(load_from(&t.path).sound_volume, 0.7); assert_eq!(fs::read_dir(&t.profile).unwrap().count(), 1);
}

#[test]
fn settings_revision_after_publication_change_reports_uncertainty_without_deleting_it() {
    let t = Temp::new(); let original = t.write(0.2); let (witness, _) = preferences_file::read(&t.path, false).unwrap();
    let error = witness.commit_for_test(&original, || {}, || { t.write(0.7); }).unwrap_err();
    assert!(preferences_file::unconfirmed(&error)); assert_eq!(load_from(&t.path).sound_volume, 0.7); assert_eq!(fs::read_dir(&t.profile).unwrap().count(), 1);
}

#[test]fn settings_review_only_projection_cannot_authorize_a_fully_parseable_noncanonical_profile(){
 let f=crate::settings::EffectFixture::new();let mut raw:serde_json::Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();raw["opaqueFutureData"]=serde_json::json!({"note":"retain"});std::fs::write(&f.path,serde_json::to_vec(&raw).unwrap()).unwrap();assert!(crate::settings::read_initialized(&f.path).is_err());let protected=crate::settings::load_for_review(&f.path);assert_eq!(crate::settings::io_status(&protected).phase,crate::settings::IoPhase::ReadUnavailable);assert_eq!(protected.model,"explicit-fixture-model");assert_eq!(protected.autostart,f.current.lock().unwrap().autostart);assert!(crate::settings::io_status(&protected).check().is_err());
}
