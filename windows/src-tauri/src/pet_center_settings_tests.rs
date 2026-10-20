use super::{apply_with, authorize_window, preserve_incoming, Flags, Patch};
use crate::settings::{self, Settings};
use serde::Deserialize;
use serde_json::{json, Value};
use std::cell::Cell;
use std::io;
use std::sync::{Arc, Barrier, Mutex, TryLockError};
use std::thread;

fn encoded(settings: &Settings) -> Value {
    serde_json::to_value(settings).expect("synthetic Settings serialize")
}

fn patch(input: &str) -> Patch {
    serde_json::from_str(input).expect("valid patch")
}

#[test]
fn accepts_only_present_boolean_fields_and_keeps_false_present() {
    assert_eq!(
        patch(r#"{"petControlBar":false}"#),
        Patch {
            pet_control_bar: Some(false),
            pet_system_center: None,
        }
    );
    assert_eq!(
        patch(r#"{"petSystemCenter":true}"#),
        Patch {
            pet_control_bar: None,
            pet_system_center: Some(true),
        }
    );
    assert_eq!(
        patch(r#"{"petSystemCenter":false,"petControlBar":true}"#),
        Patch {
            pet_control_bar: Some(true),
            pet_system_center: Some(false),
        }
    );
}

#[test]
fn rejects_empty_null_nonboolean_unknown_duplicate_and_positional_inputs() {
    let invalid = [
        r#"{}"#,
        r#"null"#,
        r#"true"#,
        r#"0"#,
        r#""petControlBar""#,
        r#"[]"#,
        r#"[true,false]"#,
        r#"[{"petControlBar":true}]"#,
        r#"{"petControlBar":null}"#,
        r#"{"petSystemCenter":null}"#,
        r#"{"petControlBar":1}"#,
        r#"{"petSystemCenter":"false"}"#,
        r#"{"petControlBar":[]}"#,
        r#"{"petSystemCenter":{}}"#,
        r#"{"companionMode":true}"#,
        r#"{"pet_control_bar":true}"#,
        r#"{"petControlBar":true,"unknown":false}"#,
        r#"{"petControlBar":false,"petControlBar":true}"#,
        r#"{"petSystemCenter":false,"petSystemCenter":false}"#,
        r#"{"petControlBar":false,"\u0070etControlBar":true}"#,
        r#"{"petControlBar":true}{}"#,
    ];
    for input in invalid {
        assert!(serde_json::from_str::<Patch>(input).is_err(), "{input}");
    }
}

#[test]
fn direct_deserialize_still_rejects_arrays_duplicates_and_null() {
    // This tests the direct deserializer. Tauri's normalized Value boundary
    // does not preserve raw duplicate fields and is a separate ingress scope.
    for input in [
        r#"[true,false]"#,
        r#"{"petControlBar":true,"petControlBar":false}"#,
        r#"{"petSystemCenter":null}"#,
    ] {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        assert!(Patch::deserialize(&mut deserializer).is_err(), "{input}");
    }
}

#[test]
fn canonical_flags_serialize_as_exactly_two_booleans() {
    let mut settings = Settings::default();
    settings.pet_control_bar = true;
    settings.pet_system_center = false;
    assert_eq!(
        serde_json::to_value(Flags::from(&settings)).unwrap(),
        json!({"petControlBar": true, "petSystemCenter": false})
    );
}

#[test]
fn single_field_change_preserves_other_flag_and_every_other_preference() {
    for input in [
        r#"{"petControlBar":false}"#,
        r#"{"petSystemCenter":false}"#,
    ] {
        let mut initial = Settings::default();
        initial.pet_control_bar = true;
        initial.pet_system_center = true;
        initial.companion_mode = true;
        initial.companion_motion = false;
        initial.screen = "synthetic-display".into();
        initial.sound_volume = 0.37;
        initial.active_integrations = vec!["synthetic-integration".into()];
        initial.chat_models.insert("synthetic-provider".into(), "synthetic-model".into());
        let current = Mutex::new(initial.clone());
        let requested = patch(input);
        let mut expected = initial;
        if let Some(value) = requested.pet_control_bar {
            expected.pet_control_bar = value;
        }
        if let Some(value) = requested.pet_system_center {
            expected.pet_system_center = value;
        }
        let persisted = Cell::new(false);
        let published = Cell::new(false);
        let result = apply_with(
            &current,
            requested,
            |candidate| {
                assert_eq!(encoded(candidate), encoded(&expected));
                persisted.set(true);
                Ok(())
            },
            |committed| {
                assert!(persisted.get());
                assert_eq!(encoded(committed), encoded(&expected));
                published.set(true);
            },
        ).unwrap();
        assert!(published.get());
        assert_eq!(result, Flags::from(&expected));
        assert_eq!(encoded(&current.lock().unwrap()), encoded(&expected));
    }
}

#[test]
fn no_op_is_an_allowed_acknowledged_transaction() {
    let current = Mutex::new(Settings::default());
    let before = encoded(&current.lock().unwrap());
    let calls = Cell::new(0);
    let result = apply_with(
        &current,
        patch(r#"{"petControlBar":false,"petSystemCenter":false}"#),
        |_| { calls.set(calls.get() + 1); Ok(()) },
        |_| { calls.set(calls.get() + 1); },
    ).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(result, Flags { pet_control_bar: false, pet_system_center: false });
    assert_eq!(encoded(&current.lock().unwrap()), before);
}

#[test]
fn direct_empty_patch_rejects_before_persistence_or_publication() {
    let current = Mutex::new(Settings::default());
    let before = encoded(&current.lock().unwrap());
    let error = apply_with(
        &current,
        Patch::default(),
        |_| panic!("empty patch must not persist"),
        |_| panic!("empty patch must not publish"),
    ).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(encoded(&current.lock().unwrap()), before);
}

#[test]
fn failed_persistence_does_not_commit_or_publish() {
    let mut initial = Settings::default();
    initial.screen = "synthetic-display".into();
    let current = Mutex::new(initial.clone());
    let published = Cell::new(false);
    let error = apply_with(
        &current,
        patch(r#"{"petControlBar":true,"petSystemCenter":true}"#),
        |candidate| {
            assert!(candidate.pet_control_bar && candidate.pet_system_center);
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "synthetic write failure"))
        },
        |_| published.set(true),
    ).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(!published.get());
    assert_eq!(encoded(&current.lock().unwrap()), encoded(&initial));
}

#[test]
fn preserve_incoming_clamps_only_the_two_owned_flags() {
    let mut current = Settings::default();
    current.pet_control_bar = true;
    current.pet_system_center = false;
    current.screen = "current-display".into();
    let mut incoming = Settings::default();
    incoming.pet_control_bar = false;
    incoming.pet_system_center = true;
    incoming.screen = "incoming-display".into();
    incoming.companion_mode = true;
    incoming.companion_bar_mode = true;
    incoming.sound_enabled = false;
    let mut expected = incoming.clone();
    expected.pet_control_bar = true;
    expected.pet_system_center = false;
    preserve_incoming(&current, &mut incoming);
    assert_eq!(encoded(&incoming), encoded(&expected));
}

#[test]
fn stale_general_save_preserves_committed_flags_and_accepts_other_preferences() {
    let current = Mutex::new(Settings::default());
    let mut incoming = current.lock().unwrap().clone();
    incoming.screen = "cursor".into();
    incoming.companion_mode = true;
    apply_with(
        &current,
        patch(r#"{"petControlBar":true,"petSystemCenter":true}"#),
        |_| Ok(()),
        |_| {},
    ).unwrap();
    settings::update_with(
        &current,
        |latest| {
            preserve_incoming(latest, &mut incoming);
            *latest = incoming;
        },
        |_| Ok(()),
        |_| {},
    ).unwrap();
    let latest = current.lock().unwrap();
    assert!(latest.pet_control_bar && latest.pet_system_center);
    assert_eq!(latest.screen, "cursor");
    assert!(latest.companion_mode);
}

#[test]
fn concurrent_flag_patches_and_stale_general_save_serialize_without_lost_fields() {
    let mut initial = Settings::default();
    initial.companion_mode = true;
    initial.model = "synthetic-model".into();
    let mut stale = initial.clone();
    stale.screen = "cursor".into();
    let current = Arc::new(Mutex::new(initial));
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let competing_start = Arc::new(Barrier::new(3));
    let events = Arc::new(Mutex::new(Vec::<(Flags, String)>::new()));
    let first = {
        let current = current.clone();
        let entered = entered.clone();
        let release = release.clone();
        let events = events.clone();
        thread::spawn(move || apply_with(
            &current,
            patch(r#"{"petControlBar":true}"#),
            |_| { entered.wait(); release.wait(); Ok(()) },
            |committed| events.lock().unwrap().push((Flags::from(committed), committed.screen.clone())),
        ).unwrap())
    };
    entered.wait();
    assert!(matches!(current.try_lock(), Err(TryLockError::WouldBlock)));
    let second = {
        let current = current.clone();
        let start = competing_start.clone();
        let events = events.clone();
        thread::spawn(move || {
            start.wait();
            apply_with(
                &current,
                patch(r#"{"petSystemCenter":true}"#),
                |_| Ok(()),
                |committed| events.lock().unwrap().push((Flags::from(committed), committed.screen.clone())),
            ).unwrap()
        })
    };
    let general = {
        let current = current.clone();
        let start = competing_start.clone();
        let events = events.clone();
        thread::spawn(move || {
            start.wait();
            settings::update_with(
                &current,
                |latest| { preserve_incoming(latest, &mut stale); *latest = stale; },
                |_| Ok(()),
                |committed| events.lock().unwrap().push((Flags::from(committed), committed.screen.clone())),
            ).unwrap()
        })
    };
    competing_start.wait();
    release.wait();
    let first_ack = first.join().unwrap();
    second.join().unwrap();
    general.join().unwrap();
    assert_eq!(first_ack, Flags { pet_control_bar: true, pet_system_center: false });
    let latest = current.lock().unwrap();
    assert!(latest.pet_control_bar && latest.pet_system_center);
    assert_eq!(latest.screen, "cursor");
    assert!(latest.companion_mode);
    assert_eq!(latest.model, "synthetic-model");
    let events = events.lock().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0], (first_ack, "primary".into()));
    assert!(events.iter().all(|(flags, _)| flags.pet_control_bar));
    assert_eq!(events.last().unwrap(), &(Flags::from(&*latest), "cursor".into()));
}

#[test]
fn failed_concurrent_writer_cannot_publish_or_leak_its_candidate() {
    let current = Arc::new(Mutex::new(Settings::default()));
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let competitor_start = Arc::new(Barrier::new(2));
    let events = Arc::new(Mutex::new(Vec::<Flags>::new()));
    let first = {
        let current = current.clone();
        let entered = entered.clone();
        let release = release.clone();
        thread::spawn(move || apply_with(
            &current,
            patch(r#"{"petControlBar":true}"#),
            |_| {
                entered.wait();
                release.wait();
                Err(io::Error::new(io::ErrorKind::Other, "synthetic write failure"))
            },
            |_| panic!("failed writer must not publish"),
        ).unwrap_err().kind())
    };
    entered.wait();
    assert!(matches!(current.try_lock(), Err(TryLockError::WouldBlock)));
    let successor = {
        let current = current.clone();
        let start = competitor_start.clone();
        let events = events.clone();
        thread::spawn(move || {
            start.wait();
            apply_with(
                &current,
                patch(r#"{"petSystemCenter":true}"#),
                |_| Ok(()),
                |committed| events.lock().unwrap().push(Flags::from(committed)),
            ).unwrap()
        })
    };
    competitor_start.wait();
    release.wait();
    assert_eq!(first.join().unwrap(), io::ErrorKind::Other);
    let acknowledged = successor.join().unwrap();
    assert_eq!(acknowledged, Flags { pet_control_bar: false, pet_system_center: true });
    assert_eq!(Flags::from(&*current.lock().unwrap()), acknowledged);
    assert_eq!(events.lock().unwrap().as_slice(), &[acknowledged]);
}

#[test]
fn window_authorization_is_exact_and_rejects_absent_native_identity() {
    assert!(authorize_window("settings", "settings").is_ok());
    for actual in ["", "Settings", "settings\n", "settings-extra", "island", "pet-chat", "pet-control"] {
        assert_eq!(authorize_window(actual, "settings").unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }
    assert_eq!(authorize_window("", "").unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(authorize_window("settings", "").unwrap_err().kind(), io::ErrorKind::PermissionDenied);
}
