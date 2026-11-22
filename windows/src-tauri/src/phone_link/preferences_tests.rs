use super::*;
use serde_json::{json, Value};
use std::cell::Cell;

fn changes() -> Vec<Change> {
    vec![Change::Link(true), Change::Relay { enabled: true, url: "wss://relay.invalid".into() },
        Change::RelayOff, Change::Details(true), Change::Answers(true), Change::Diffs(true),
        Change::Usage(true), Change::Services(vec!["integration_stripe".into()]),
        Change::Chat(true), Change::ChatModels(vec!["ollama/synthetic".into()])]
}
fn shown(s: &Settings) -> Value { serde_json::to_value(s).unwrap() }

#[test]
fn settings_consumer_phone_failure_never_changes_features_or_publishes_success() {
    for change in changes() {
        let mut original = Settings::default(); original.phone_relay = true;
        original.autostart = true; original.voice_pet_aliases.insert("codex".into(), vec!["codi".into()]);
        let before = shown(&original); let current = Mutex::new(original); let calls = Cell::new(0);
        let error = commit_with(&current, change, |_| {
            calls.set(calls.get() + 1); Err(io::Error::new(io::ErrorKind::PermissionDenied, "private fixture denied"))
        }, |_| panic!("a refused preference was advertised to windows/phones")).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied); assert_eq!(calls.get(), 1);
        assert_eq!(shown(&current.lock().unwrap()), before);
    }
}

#[test]
fn settings_consumer_phone_intents_preserve_other_owners_and_publish_only_committed_state() {
    for change in changes() {
        let mut original = Settings::default(); original.autostart = true;
        original.chat_models.insert("ollama".into(), "selected-before-wait".into());
        original.cli_workspaces.insert("copilot".into(), "/private/project".into());
        original.active_integrations.push("agent_antigravity".into());
        let before = shown(&original); let current = Mutex::new(original);
        let stored = std::cell::RefCell::new(None); let published = Cell::new(false);
        let after = commit_with(&current, change, |s| { *stored.borrow_mut() = Some(shown(s)); Ok(()) }, |s| {
            assert_eq!(stored.borrow().as_ref(), Some(&shown(s))); published.set(true);
        }).unwrap();
        assert!(published.get()); assert_eq!(shown(&after), shown(&current.lock().unwrap()));
        for field in ["autostart", "chatModels", "cliWorkspaces", "activeIntegrations", "mainPill", "petControlBar", "petSystemCenter", "voicePetAliases"] {
            assert_eq!(shown(&after)[field], before[field], "another field owner was replaced: {field}");
        }
    }
}

#[test]
fn settings_consumer_phone_partial_persistence_is_error_not_an_ack_or_a_rollback_claim() {
    // IO callbacks can fail after changing disk. This is deliberately not a
    // proof that a future writer has already implemented durable reconciliation.
    let current = Mutex::new(Settings::default()); let disk = std::cell::RefCell::new(json!({}));
    let error = commit_with(&current, Change::Answers(true), |next| {
        *disk.borrow_mut() = shown(next); Err(io::Error::other("synthetic publication uncertainty"))
    }, |_| panic!("uncertain write published a permission ACK")).unwrap_err();
    assert!(!current.lock().unwrap().phone_answers); assert_eq!(disk.borrow()["phoneAnswers"], true);
    let message = settings::save_error("Preferência não confirmada", &error);
    assert!(message.contains("synthetic publication uncertainty"));
    assert!(!message.contains("preservad"));
}

#[test]
fn settings_consumer_phone_refused_old_intent_cannot_overwrite_the_next_successful_choice() {
    use std::sync::{Arc, mpsc}; use std::time::Duration;
    let current = Arc::new(Mutex::new(Settings::default())); let a = current.clone(); let b = current.clone();
    let (entered, entered_rx) = mpsc::channel(); let (release, release_rx) = mpsc::channel();
    let (requested, requested_rx) = mpsc::channel(); let (persisted, persisted_rx) = mpsc::channel();
    let first = std::thread::spawn(move || commit_with(&a, Change::Answers(true), |_| {
        entered.send(()).unwrap(); release_rx.recv().unwrap(); Err(io::Error::other("old write failed"))
    }, |_| panic!("old failed choice published")));
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let second = std::thread::spawn(move || { requested.send(()).unwrap(); commit_with(&b, Change::Details(true), |next| {
        assert!(!next.phone_answers); persisted.send(()).unwrap(); Ok(())
    }, |_| {}) });
    requested_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(persisted_rx.recv_timeout(Duration::from_millis(30)).is_err()); release.send(()).unwrap();
    assert!(first.join().unwrap().is_err()); second.join().unwrap().unwrap();
    let actual = current.lock().unwrap(); assert!(!actual.phone_answers); assert!(actual.phone_details);
}
