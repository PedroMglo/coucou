use super::*;
use crate::pet_capabilities::{Availability, Binding, Capability, Caller, Concurrency, Decision, Kind, Limits, Source};
use std::cell::Cell;

fn descriptor(provider: &str) -> Descriptor {
    let pet = Pet::from_provider(provider).unwrap();
    let adapter = match pet { Pet::Copilot => "copilot", Pet::Gravity => "antigravity", Pet::Ollama => "ollama", Pet::Spotify => "spotify", _ => "codex" };
    Descriptor { schema_version: 1, pet_id: pet.pill().into(), kind: if crate::pet_config::service(pet.pill()) { Kind::Service } else { Kind::Llm },
        name: pet.name().into(), lifecycle: Lifecycle::Active, binding: Binding { engine_binding:None, adapter: Some(adapter), provider: Some(provider.into()), model: None, profile: Some("auto".into()) },
        revision: "synthetic:1".into(), observed_at: None, configuration_available: true, concurrency: Concurrency { scope: Some("provider"), max_active: Some(1) },
        capabilities: vec![Capability { id: "voice.respond", support: Support::Supported, availability: Availability::Unknown, reason: Some(Reason::TargetRequired),
            source: vec![Source::BundledManifest, Source::Configuration], observed_at: None, expires_at: None, model_binding: None,
            decision: Decision::NativeApproval, callers: vec![Caller::NativeUser], limits: Limits::default(), limitations: vec![] }] }
}
#[test]
fn optional_service_assistant_keeps_the_native_voice_controller_and_rejects_other_accounts(){
    let mut d=descriptor("integration_spotify");d.binding.adapter=Some("codex");
    let candidate=admit("integration_spotify",&d,|c|c.route==Route::Spotify).unwrap();
    assert_eq!(candidate.pet,Pet::Spotify);assert_eq!(candidate.pet.provider(),"integration_spotify");assert_eq!(candidate.route,Route::Spotify);
    d.binding.adapter=Some("copilot");assert!(matches!(prepare("integration_spotify",&d),Err(Failure::UnsupportedPet)));
}

#[test]
fn implemented_voice_routes_do_not_infer_execution_from_observer_or_api_catalog_entries() {
    for pet in Pet::ALL { assert_eq!(implemented_for_pet(pet.pill()), !matches!(pet, Pet::Claude | Pet::Gemini)); }
    for id in ["ai_openai", "ai_anthropic", "ai_google", "ai_lmstudio", "agent_cursor", "unknown"] { assert!(!implemented_for_pet(id)); }
}

#[test]
fn configured_pets_keep_own_identity_even_when_services_use_the_codex_adapter() {
    for pet in Pet::ALL.into_iter().filter(|p| !matches!(p, Pet::GitHub | Pet::Claude | Pet::Gemini)) {
        let d = descriptor(pet.provider()); let called = Cell::new(0);
        let plan = admit(pet.provider(), &d, |candidate| { called.set(called.get()+1); assert_eq!(candidate.pet, pet); true }).unwrap();
        assert_eq!(plan.pet.provider(), pet.provider()); assert_eq!(called.get(), 1);
        if crate::pet_config::service(pet.pill()) && pet != Pet::Spotify { assert_eq!(plan.route, Route::Codex); assert_ne!(plan.pet.provider(), "codex"); }
    }
}

#[test]
fn wrong_provider_pet_or_adapter_cannot_probe_or_borrow_another_identity() {
    let called = Cell::new(false); let mut d = descriptor("codex"); d.pet_id = "agent_copilot".into();
    assert_eq!(admit("codex", &d, |_| { called.set(true); true }).unwrap_err(), Failure::IdentityMismatch);
    d = descriptor("copilot"); d.binding.provider = Some("codex".into());
    assert_eq!(admit("copilot", &d, |_| { called.set(true); true }).unwrap_err(), Failure::IdentityMismatch);
    d = descriptor("copilot"); d.binding.adapter = Some("codex");
    assert_eq!(admit("copilot", &d, |_| { called.set(true); true }).unwrap_err(), Failure::UnsupportedPet);
    assert!(!called.get());
}

#[test]
fn inactive_retired_and_missing_configuration_never_probe_dependencies() {
    let called = Cell::new(false);
    for failure in [Failure::NotConfigured, Failure::InactivePet, Failure::RetiredPet] {
        let mut d = descriptor("codex");
        match failure { Failure::NotConfigured => d.configuration_available = false, Failure::InactivePet => d.lifecycle = Lifecycle::Inactive, _ => d.lifecycle = Lifecycle::Retired }
        assert_eq!(admit("codex", &d, |_| { called.set(true); true }).unwrap_err(), failure);
    }
    // A crafted active descriptor cannot reactivate the retired GitHub entity.
    assert_eq!(admit("integration_github", &descriptor("integration_github"), |_| { called.set(true); true }).unwrap_err(), Failure::RetiredPet);
    assert!(!called.get());
}

#[test]
fn unsupported_unknown_denied_and_ambiguous_metadata_do_not_admit_new_work() {
    for variant in 0..7 {
        let mut d = descriptor("codex");
        match variant {
            0 => d.capabilities[0].support = Support::Unsupported,
            1 => d.capabilities[0].support = Support::Unknown,
            2 => d.capabilities[0].availability = Availability::Unavailable,
            3 => d.capabilities[0].callers = vec![Caller::ProviderModel],
            4 => d.capabilities[0].reason = Some(Reason::LibraryUnavailable),
            5 => d.capabilities.push(d.capabilities[0].clone()),
            _ => d.capabilities.clear(),
        }
        assert!(admit("codex", &d, |_| panic!("an inadmissible pet must not probe dependencies")).is_err());
    }
}

#[test]
fn unresolved_model_allows_explicit_preparation_without_claiming_inference_ready() {
    let mut d = descriptor("codex"); d.capabilities[0].reason = Some(Reason::ModelUnresolved);
    assert!(admit("codex", &d, |_| true).is_ok());
    assert_eq!(d.capabilities[0].availability, Availability::Unknown); assert!(d.binding.model.is_none());
}

#[test]
fn unavailable_adapter_and_local_model_have_distinct_non_authentication_failures() {
    for (provider, expected) in [("codex", Failure::AdapterUnavailable), ("copilot", Failure::AdapterUnavailable), ("antigravity", Failure::AdapterUnavailable), ("ollama", Failure::LocalModelUnavailable)] {
        let reason = admit(provider, &descriptor(provider), |_| false).unwrap_err(); assert_eq!(reason, expected);
        assert!(!reason.message().contains("conta")); assert!(!reason.message().contains("token"));
    }
}

#[test]
fn revalidating_each_request_detects_deactivation_after_a_successful_preparation() {
    let mut d = descriptor("codex"); let original = admit("codex", &d, |_| true).unwrap();
    d.lifecycle = Lifecycle::Inactive;
    assert_eq!(admit("codex", &d, |_| panic!("no new work after deactivation")).unwrap_err(), Failure::InactivePet);
    assert_eq!(original.pet, Pet::Codex); // The original identity is not reassigned to another pet.
}

#[test]
fn ollama_voice_uses_own_profile_while_text_uses_another_provider() {
    let mut config = crate::pet_config::defaults(); let p = config.pets.get_mut("ai_ollama").unwrap();
    p.mode = "fast".into(); p.fast.model = "synthetic-local-model".into();
    let mut settings = Settings::default(); settings.chat_provider = "codex".into(); settings.ollama_url = "http://127.0.0.1:11434".into();
    settings.chat_models.insert("ollama".into(), "text-only-override".into());
    let plan = admit("ollama", &descriptor("ollama"), |_| true).unwrap();
    assert!(local_ready(plan, &settings, &config));
    let selected = settings_for(plan, &settings); assert_eq!(selected.chat_provider, "ollama"); assert_eq!(selected.chat_models["ollama"], "@default");
    assert_eq!(settings.chat_provider, "codex"); assert_eq!(settings.chat_models["ollama"], "text-only-override");
}

#[test]
fn ollama_preparation_preserves_local_only_policy_and_never_falls_back_to_another_adapter() {
    let mut config = crate::pet_config::defaults(); let p = config.pets.get_mut("ai_ollama").unwrap(); p.mode = "fast".into(); p.fast.model = "synthetic-local".into();
    let mut settings = Settings::default(); settings.ollama_url = "http://127.0.0.1:11434".into();
    let plan = admit("ollama", &descriptor("ollama"), |_| true).unwrap(); assert_eq!(plan.route, Route::Ollama);
    for url in ["https://example.test", "not a url"] { settings.ollama_url = url.into(); assert!(!local_ready(plan, &settings, &config)); }
    settings.ollama_url = "http://127.0.0.1:11434".into();
    for model in ["remote:cloud", "remote-cloud", ""] { config.pets.get_mut("ai_ollama").unwrap().fast.model = model.into(); assert!(!local_ready(plan, &settings, &config)); }
}

#[test]
fn deterministic_spotify_does_not_require_an_llm_model_or_borrow_codex() {
    let d = descriptor("integration_spotify"); let plan = admit("integration_spotify", &d, |_| true).unwrap();
    assert_eq!(plan.route, Route::Spotify); assert_eq!(plan.pet.provider(), "integration_spotify"); assert!(d.binding.model.is_none());
    assert!(!local_ready(plan, &Settings::default(), &crate::pet_config::defaults()));
}

#[test]
fn addressed_recovery_intents_do_not_require_new_pet_execution() {
    for text in ["interromper tarefa", "estado da tarefa", "parar conversa", "ditar"] { assert!(!uses_pet_response(text), "{text}"); }
    for text in ["resume o documento", "abre uma aplicação", "altera o volume", ""] { assert!(uses_pet_response(text), "{text}"); }
    for name in ["Codex", "Gravity", "Copilot", "Spotify", "GitHub"] {
        let command = super::super::dialogue::parse(&format!("{name} interromper tarefa"));
        assert!(known_voice_address(&command));
        if let super::super::dialogue::Command::Select(_, question) = command { assert!(!uses_pet_response(&question)); } else { panic!("pet address was not resolved"); }
    }
}

#[test]
fn spotify_deterministic_address_is_resolved_before_the_generic_empty_provider_warning() {
    let config = crate::pet_config::defaults(); let command = super::super::dialogue::parse("Spotify pausa");
    assert!(config.pets["integration_spotify"].provider.is_empty());
    assert!(config.unavailable_address("Spotify pausa").is_some());
    assert!(known_voice_address(&command));
    assert!(!known_voice_address(&super::super::dialogue::parse("um pet desconhecido pausa")));
}

#[test]
fn each_catalog_voice_route_has_a_native_controller_and_original_owner(){
 for p in crate::pet_config::blueprints(){if let Some(provider)=&p.routing.voice{let voice=Pet::from_provider(provider).expect("declare the native voice controller when adding a voice route");assert_eq!(voice.pill(),p.id);assert_eq!(voice.provider(),provider);assert!(p.runtime.adapter.is_some());}}
}
