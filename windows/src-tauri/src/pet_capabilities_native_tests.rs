use super::*;
use crate::session_chat::types::{Capabilities, Origin};

fn target(provider: &str, n: usize) -> Target {
    let pet = match provider { "codex" => "agent_codex", "copilot" => "agent_copilot", _ => "agent_antigravity" };
    Target { pet_id: pet.into(), provider: provider.into(), origin: Origin::External,
        session_id: format!("00000000-0000-0000-0000-{n:012x}"), conversation_id: None }
}
fn snapshot(t: Target) -> Snapshot {
    Snapshot { target: t, messages: vec![], busy: false, control_conflict: false, turn: None,
        capabilities: Capabilities { history: true, send: true, cancel: false }, model: Some("synthetic-native".into()),
        notice: None, capability_descriptor: None }
}
fn cap<'a>(d: &'a TargetDescriptor, id: &str) -> &'a Capability { d.capabilities.iter().find(|c| c.id == id).unwrap() }
fn publish(store: &mut Store, s: &Snapshot, at: u64) {
    let ticket = store.begin(&s.target, at).unwrap(); assert!(store.finish(ticket, Some(s), at + 1));
}

#[test]
fn original_pet_provider_origin_and_session_are_required() {
    let mut store = Store::new("test".into());
    for provider in ["codex", "copilot", "antigravity"] {
        let valid = target(provider, 1); assert!(store.begin(&valid, 100_000).is_some());
        let mut wrong = valid.clone(); wrong.pet_id = "integration_music".into(); assert!(store.begin(&wrong, 100_000).is_none());
        wrong = valid.clone(); wrong.origin = Origin::Mochi; assert!(store.begin(&wrong, 100_000).is_none());
        wrong = valid.clone(); wrong.conversation_id = Some("foreign".into()); assert!(store.begin(&wrong, 100_000).is_none());
        wrong = valid; wrong.session_id = "arbitrary-same-pet".into(); assert!(store.begin(&wrong, 100_000).is_none());
    }
    assert!(store.begin(&target("unknown", 1), 100_000).is_none());
}

#[test]
fn native_evidence_is_exact_target_and_not_general_approval_or_background_support() {
    let mut store = Store::new("test".into());
    for provider in ["codex", "copilot", "antigravity"] {
        let s = snapshot(target(provider, 1)); publish(&mut store, &s, 100_000);
        let d = store.view(&s.target, 100_002);
        for id in ["session.history", "session.send"] { assert_eq!(cap(&d, id).availability, Availability::Ready); }
        for id in ["session.approve", "session.continueBackground"] {
            assert_eq!(cap(&d, id).support, Support::Unknown); assert_ne!(cap(&d, id).availability, Availability::Ready);
        }
        let other = store.view(&target(provider, 2), 100_002);
        assert_ne!(cap(&other, "session.send").availability, Availability::Ready);
        if provider != "codex" { assert_eq!(cap(&d, "session.interrupt").support, Support::Unsupported); }
        assert!(d.capabilities.iter().all(|c| c.callers == vec![super::super::Caller::NativeUser]));
    }
}

#[test]
fn busy_and_ambiguous_turns_do_not_grant_control() {
    let mut store = Store::new("test".into()); let mut s = snapshot(target("codex", 1));
    s.busy = true; s.turn = Some("native-turn".into()); s.capabilities.cancel = true;
    publish(&mut store, &s, 100_000); let d = store.view(&s.target, 100_002);
    assert_eq!(cap(&d, "session.send").reason, Some(Reason::Busy));
    assert_ne!(cap(&d, "session.send").support, Support::Unsupported);
    assert_eq!(cap(&d, "session.interrupt").availability, Availability::Ready);
    s.control_conflict = true; publish(&mut store, &s, 100_003); let d = store.view(&s.target, 100_005);
    for id in ["session.send", "session.interrupt"] { assert_eq!(cap(&d, id).reason, Some(Reason::ControlConflict)); }
    assert_eq!(cap(&d, "session.history").availability, Availability::Ready);
    s.control_conflict = false; s.turn = None; publish(&mut store, &s, 100_006);
    assert_eq!(cap(&store.view(&s.target, 100_008), "session.interrupt").reason, Some(Reason::TurnRequired));
}

#[test]
fn old_late_evicted_and_wrong_target_responses_cannot_republish() {
    let mut store = Store::new("test".into()); let s = snapshot(target("codex", 1));
    let old = store.begin(&s.target, 100_000).unwrap();
    let latest = store.begin(&s.target, 100_001).unwrap();
    assert!(!store.finish(old, Some(&s), 100_002)); assert!(store.finish(latest, Some(&s), 100_003));
    let late = store.begin(&s.target, 100_004).unwrap(); assert!(!store.finish(late, Some(&s), 145_005));
    let wrong = store.begin(&s.target, 200_000).unwrap();
    assert!(!store.finish(wrong, Some(&snapshot(target("codex", 2))), 200_001));
    let evicted = store.begin(&s.target, 200_002).unwrap();
    for n in 2..=129 { store.begin(&target("codex", n), 200_003 + n as u64).unwrap(); }
    assert_eq!(store.slots.len(), MAX_TARGETS); assert!(!store.finish(evicted, Some(&s), 200_200));
}

#[test]
fn events_disconnects_and_reconnects_retire_inflight_tickets_without_widening_other_pets() {
    let mut store = Store::new("test".into());
    let codex = snapshot(target("codex", 1)); let copilot = snapshot(target("copilot", 1));
    publish(&mut store, &copilot, 100_000);
    let old = store.begin(&codex.target, 100_002).unwrap(); store.invalidate_provider("codex");
    assert!(!store.finish(old, Some(&codex), 100_003));
    assert_eq!(cap(&store.view(&copilot.target, 100_003), "session.send").availability, Availability::Ready);
    let old = store.begin(&codex.target, 100_004).unwrap(); store.invalidate(&codex.target);
    assert!(!store.finish(old, Some(&codex), 100_005));
    publish(&mut store, &codex, 100_006);
    assert_eq!(cap(&store.view(&codex.target, 100_008), "session.send").availability, Availability::Ready);
    store.invalidate(&codex.target);
    assert_eq!(cap(&store.view(&codex.target, 100_009), "session.send").reason, Some(Reason::StaleObservation));
}

#[test]
fn equal_session_ids_keep_distinct_native_models_and_independent_invalidation() {
    let mut store = Store::new("test".into());
    for provider in ["codex", "copilot", "antigravity"] {
        let mut s = snapshot(target(provider, 1)); s.model = Some(format!("{provider}-only-model"));
        publish(&mut store, &s, 100_000);
    }
    store.invalidate(&target("copilot", 1));
    for provider in ["codex", "antigravity"] {
        let d = store.view(&target(provider, 1), 100_002);
        assert_eq!(d.model, Some(format!("{provider}-only-model")));
        assert_eq!(cap(&d, "session.send").availability, Availability::Ready);
    }
    assert_ne!(cap(&store.view(&target("copilot", 1), 100_002), "session.send").availability, Availability::Ready);
}

#[test]
fn expiration_clock_rollback_and_failed_reads_remain_unknown() {
    let mut store = Store::new("test".into()); let s = snapshot(target("codex", 1)); publish(&mut store, &s, 100_000);
    for now in [100_000, 115_001, super::super::cache::MAX_DATE + 1] {
        let d = store.view(&s.target, now); assert_eq!(cap(&d, "session.send").reason, Some(Reason::StaleObservation));
        assert!(d.busy.is_none()); assert!(d.turn.is_none());
    }
    let ticket = store.begin(&s.target, 200_000).unwrap(); assert!(!store.finish(ticket, None, 200_001));
    assert_eq!(cap(&store.view(&s.target, 200_001), "session.send").reason, Some(Reason::ProviderUnavailable));
    assert!(store.begin(&s.target, 0).is_none());
}

#[test]
fn peer_metadata_cannot_inject_a_backend_descriptor_or_private_control_barrier() {
    let mut store = Store::new("test".into()); let s = snapshot(target("copilot", 1)); publish(&mut store, &s, 100_000);
    let mut value = serde_json::to_value(&s).unwrap();
    value["capabilityDescriptor"] = serde_json::to_value(store.view(&s.target, 100_002)).unwrap();
    value["control_conflict"] = serde_json::json!(true);
    let received: Snapshot = serde_json::from_value(value).unwrap();
    assert!(received.capability_descriptor.is_none()); assert!(!received.control_conflict);
}

#[test]
fn cache_keeps_no_transcript_directory_notice_or_invalid_model_path() {
    let mut store = Store::new("test".into()); let mut s = snapshot(target("antigravity", 1));
    s.messages.push(crate::session_chat::types::Message { id: "private".into(), role: crate::session_chat::types::Role::User, content: "PRIVATE_TRANSCRIPT".into() });
    s.notice = Some("PRIVATE_NOTICE".into()); s.model = Some("/PRIVATE_ENDPOINT/token".into());
    publish(&mut store, &s, 100_000); let d = store.view(&s.target, 100_002);
    let value = serde_json::to_string(&d).unwrap(); assert!(!value.contains("PRIVATE")); assert!(d.model.is_none());
}

#[test]
fn config_lifecycle_and_binding_are_revalidated_without_new_native_evidence() {
    use super::super::{Registry, Lifecycle};
    let config = crate::pet_config::defaults(); let mut settings = crate::settings::Settings::default();
    settings.main_pill = "agent_codex".into(); let mut registry = Registry::new().unwrap();
    let mut store = Store::new("test".into()); let s = snapshot(target("codex", 1)); publish(&mut store, &s, 100_000);
    let mut pet = registry.snapshot(&settings, Some(&config), true, &Default::default(), 100_002).unwrap().pets.into_iter().find(|p| p.pet_id == "agent_codex").unwrap();
    let mut d = store.view(&s.target, 100_002); super::super::gate_native(&mut d, &pet);
    assert_eq!(cap(&d, "session.send").availability, Availability::Ready);
    pet.lifecycle = Lifecycle::Inactive;
    let mut d = store.view(&s.target, 100_002); super::super::gate_native(&mut d, &pet);
    assert_eq!(cap(&d, "session.send").reason, Some(Reason::InactivePet));
    pet.lifecycle = Lifecycle::Active; pet.binding.provider = Some("copilot".into());
    let mut d = store.view(&s.target, 100_002); super::super::gate_native(&mut d, &pet);
    assert_eq!(cap(&d, "session.send").reason, Some(Reason::NotConfigured));
    let unhealthy = registry.snapshot(&settings, Some(&config), false, &Default::default(), 100_003).unwrap().pets.into_iter().find(|p| p.pet_id == "agent_codex").unwrap();
    let mut d = store.view(&s.target, 100_003); super::super::gate_native(&mut d, &unhealthy);
    assert_eq!(cap(&d, "session.send").reason, Some(Reason::LibraryUnavailable));
}
