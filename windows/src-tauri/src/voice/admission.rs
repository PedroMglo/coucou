//! Prepare a spoken pet using its shared identity. No account or model IO here.
//! Preparation is not permission to infer, invoke tools or answer native dialogs.
use crate::pet_capabilities::{Availability, Caller, Descriptor, Lifecycle, Reason, Support};
use crate::settings::Settings;
use super::dialogue::Pet;

/// Existing work remains addressable after a pet is disabled or retired.
pub(super) fn uses_pet_response(question: &str) -> bool {
    #[cfg(target_os="linux")]if crate::cli_chat::goal_recipe_voice_requested(question){return false;}
    !matches!(super::dialogue::parse(question), super::dialogue::Command::Stop | super::dialogue::Command::Dictate | super::dialogue::Command::CancelTask | super::dialogue::Command::TaskStatus)
}

/// A known deterministic route can legitimately have no LLM provider in config.
pub(super) fn known_voice_address(command: &super::dialogue::Command) -> bool {
    matches!(command, super::dialogue::Command::Select(pet, _) if implemented_for_pet(pet.pill()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route { Codex, Copilot, Gravity, Ollama, Spotify }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure { UnsupportedPet, IdentityMismatch, NotConfigured, InactivePet, RetiredPet, StateUnavailable, AdapterUnavailable, LocalModelUnavailable }

impl Failure {
    pub fn message(self) -> &'static str {
        match self {
            Self::UnsupportedPet => "Este pet ainda não tem uma rota de conversa por voz implementada.",
            Self::IdentityMismatch => "A configuração de voz não corresponde à identidade deste pet.",
            Self::NotConfigured => "A configuração deste pet não está disponível. Confirma-a na gestão dos pets.",
            Self::InactivePet => "Ativa este pet na gestão dos pets antes de iniciar novos pedidos de voz.",
            Self::RetiredPet => "O pet GitHub está temporariamente desativado. O histórico e as preferências foram preservados.",
            Self::StateUnavailable => "Não foi possível confirmar o estado deste pet. Não foi iniciado um novo pedido de voz.",
            Self::AdapterUnavailable => "O adaptador deste pet não está disponível neste computador. Confirma a instalação na gestão dos pets.",
            Self::LocalModelUnavailable => "Escolhe um modelo Ollama local válido nas definições deste pet antes de conversar por voz.",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Candidate { pub pet: Pet, pub route: Route }

pub(crate) fn implemented_for_pet(id: &str) -> bool {
    Pet::ALL.into_iter().any(|p|p.pill()==id)&&crate::pet_config::blueprint(id).is_some_and(|p|p.routing.voice.is_some()&&p.runtime.adapter.is_some())
}

fn prepare(provider: &str, descriptor: &Descriptor) -> Result<Candidate, Failure> {
    let pet = Pet::from_provider(provider).ok_or(Failure::UnsupportedPet)?;
    if descriptor.schema_version != 1 || descriptor.revision.is_empty() { return Err(Failure::StateUnavailable); }
    if descriptor.pet_id != pet.pill() { return Err(Failure::IdentityMismatch); }
    if descriptor.lifecycle == Lifecycle::Retired || crate::pet_config::retired(&descriptor.pet_id) { return Err(Failure::RetiredPet); }
    if !descriptor.configuration_available { return Err(Failure::NotConfigured); }
    if descriptor.lifecycle != Lifecycle::Active { return Err(Failure::InactivePet); }
    if descriptor.binding.provider.as_deref() != Some(provider) { return Err(Failure::IdentityMismatch); }
    let mut capabilities = descriptor.capabilities.iter().filter(|c| c.id == "voice.respond");
    let capability = capabilities.next().ok_or(Failure::UnsupportedPet)?;
    if capabilities.next().is_some() { return Err(Failure::StateUnavailable); }
    if capability.reason == Some(Reason::LibraryUnavailable) { return Err(Failure::StateUnavailable); }
    match capability.support {
        Support::Unsupported => return Err(Failure::UnsupportedPet),
        Support::Unknown => return Err(Failure::StateUnavailable),
        Support::Supported => {},
    }
    if capability.availability == Availability::Unavailable || !capability.callers.contains(&Caller::NativeUser) { return Err(Failure::StateUnavailable); }
    // A target/model can still be unresolved. This admits only explicit
    // preparation; the existing resolver and executor retain all their gates.
    use crate::pet_config::Adapter;
    let declared=crate::pet_config::blueprint(pet.pill()).ok_or(Failure::UnsupportedPet)?;
    if declared.routing.voice.as_deref()!=Some(provider){return Err(Failure::UnsupportedPet);}
    let adapter=declared.runtime.adapter.ok_or(Failure::UnsupportedPet)?;
    if descriptor.binding.adapter!=Some(adapter.key())&&!declared.runtime.assistant_adapter.is_some_and(|a|descriptor.binding.adapter==Some(a.key())){return Err(Failure::UnsupportedPet);}
    let route = match adapter {
        Adapter::Codex=>Route::Codex,Adapter::Copilot=>Route::Copilot,
        Adapter::Antigravity=>Route::Gravity,Adapter::Ollama=>Route::Ollama,Adapter::Spotify=>Route::Spotify,Adapter::Lmstudio=>return Err(Failure::UnsupportedPet),
    };
    Ok(Candidate { pet, route })
}

/// Probe only existing local prerequisites, after resolving the original owner.
pub(crate) fn admit(provider: &str, descriptor: &Descriptor, probe: impl FnOnce(Candidate) -> bool) -> Result<Candidate, Failure> {
    #[cfg(target_os="linux")]if crate::pet_data::bootstrap::feature_allowed("voice").is_err(){return Err(Failure::StateUnavailable);}
    let candidate = prepare(provider, descriptor)?;
    if !probe(candidate) {
        return Err(if candidate.route == Route::Ollama { Failure::LocalModelUnavailable } else { Failure::AdapterUnavailable });
    }
    Ok(candidate)
}

/// Audio selection must not overwrite or depend on the text chat's provider.
pub(crate) fn settings_for(candidate: Candidate, settings: &Settings) -> Settings {
    crate::cli_chat::voice_settings(settings, candidate.pet.provider())
}

/// Resolve the existing voice profile from the supplied validated configuration.
/// Giving local::config a concrete model avoids a second global/default read.
pub(crate) fn local_ready(candidate: Candidate, settings: &Settings, config: &crate::pet_config::Configuration) -> bool {
    if candidate.route != Route::Ollama { return false; }
    let Ok(selection) = config.select(candidate.pet.provider(), "@default", "") else { return false; };
    let mut local = settings_for(candidate, settings);
    local.chat_models.insert(candidate.pet.provider().into(), selection.model);
    super::local::config(&local).is_some()
}

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;

#[cfg(all(test,target_os="linux"))]mod goal_voice_tests{#[test]fn recipe_decisions_never_require_a_new_model_turn(){for query in ["rever receita 001234","aprovar receita 001234","recusar receita 001234","aprovar receita"]{assert!(!super::uses_pet_response(query));}assert!(super::uses_pet_response("explica esta receita"));}}
