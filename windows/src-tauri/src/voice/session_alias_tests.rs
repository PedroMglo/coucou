use super::*;
use crate::settings::Settings;
use std::{collections::BTreeMap, io};

#[test]
fn settings_consumer_voice_failed_save_does_not_install_a_route_or_evict_an_existing_name() {
    let mut settings = Settings::default();
    let old: Vec<String> = (0..20).map(|i| format!("previous{i}")).collect();
    settings.voice_pet_aliases.insert("codex".into(), old.clone());
    settings.voice_pet_aliases.insert("copilot".into(), vec!["piloto".into()]);
    let current = Mutex::new(settings);
    commit_alias_with(&current, Pet::Codex, "codi".into(), BTreeMap::new(),
        |_| Err(io::Error::other("private fixture full disk")), |_| panic!("failed alias announced")).unwrap_err();
    let actual = current.lock().unwrap(); assert_eq!(actual.voice_pet_aliases["codex"], old);
    assert_eq!(actual.voice_pet_aliases["copilot"], ["piloto"]);
    assert!(!matches!(dialogue::parse_with_aliases("codi", &actual.voice_pet_aliases), Command::Select(Pet::Codex, _)));
}

#[test]
fn settings_consumer_voice_conflicting_alias_is_refused_before_persistence() {
    let current = Mutex::new(Settings::default());
    let configured = BTreeMap::from([("copilot".into(), vec!["piloto".into()])]);
    let error = commit_alias_with(&current, Pet::Codex, "piloto".into(), configured,
        |_| panic!("ambiguous alias reached disk"), |_| panic!("ambiguous alias published")).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput); assert!(current.lock().unwrap().voice_pet_aliases.is_empty());
}

#[test]
fn settings_consumer_voice_success_keeps_bounded_routing_and_other_pets() {
    let mut settings = Settings::default();
    let old: Vec<String> = (0..20).map(|i| format!("previous{i}")).collect();
    settings.voice_pet_aliases.insert("codex".into(), old.clone());
    settings.voice_pet_aliases.insert("copilot".into(), vec!["piloto".into()]);
    let current = Mutex::new(settings); let stored = std::cell::Cell::new(false);
    commit_alias_with(&current, Pet::Codex, "codi".into(), BTreeMap::new(), |_| { stored.set(true); Ok(()) },
        |next| { assert!(stored.get()); assert_eq!(next.voice_pet_aliases["codex"].len(), 20); }).unwrap();
    let actual = current.lock().unwrap(); assert_eq!(actual.voice_pet_aliases["codex"][0], old[1]);
    assert_eq!(actual.voice_pet_aliases["copilot"], ["piloto"]);
    assert_eq!(dialogue::parse_with_aliases("codi", &actual.voice_pet_aliases), Command::Select(Pet::Codex, String::new()));
}
