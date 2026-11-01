//! Explicit roster intentions, serialized against current native preferences.
//! A stale general settings snapshot never owns these two fields.
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io, sync::Mutex};

const MAX_DECLARED: usize = 6;
const MAIN_PETS: &[&str] = &["integration_claude", "agent_cursor", "agent_antigravity", "agent_codex"];

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    SetMain { id: String },
    SetActive { id: String, enabled: bool },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Roster {
    pub(crate) main_pill: String,
    pub(crate) active_integrations: Vec<String>,
}
impl From<&Settings> for Roster {
    fn from(settings: &Settings) -> Self {
        Self { main_pill: settings.main_pill.clone(), active_integrations: settings.active_integrations.clone() }
    }
}
fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidInput, message) }

fn offered(id: &str, windows: bool) -> bool {
    // These are the same OS exclusions as the shipped pill catalog. Retired
    // identities remain in preferences/history, but cannot be reactivated.
    crate::pet_config::defaults().pets.contains_key(id)
        && !crate::pet_config::retired(id)
        && id != "integration_music"
        && (windows || id != "agent_claude-desktop")
}
fn change(settings: &mut Settings, request: Request) -> io::Result<()> {
    let id = match &request { Request::SetMain { id } | Request::SetActive { id, .. } => id };
    if !offered(id, cfg!(target_os = "windows")) { return Err(invalid("Este pet não está disponível nesta versão.")); }
    match request {
        Request::SetMain { id } => {
            if !MAIN_PETS.contains(&id.as_str()) { return Err(invalid("Este pet não pode ser o pet principal.")); }
            settings.active_integrations.retain(|active| active != &id);
            settings.main_pill = id;
        }
        Request::SetActive { id, enabled } => {
            if id == settings.main_pill {
                return if enabled { Ok(()) } else { Err(invalid("O pet principal fica sempre ativo.")) };
            }
            if enabled && !settings.active_integrations.contains(&id) {
                let used: HashSet<_> = settings.active_integrations.iter()
                    .filter(|active| *active != &settings.main_pill && offered(active, cfg!(target_os = "windows"))).collect();
                if used.len() >= MAX_DECLARED { return Err(invalid("Já tens seis pets adicionais ativos. Retira um antes de adicionar outro.")); }
                settings.active_integrations.push(id);
            } else if !enabled { settings.active_integrations.retain(|active| active != &id); }
        }
    }
    Ok(())
}

pub(crate) fn apply_with(
    current: &Mutex<Settings>, request: Request,
    persist: impl FnOnce(&Settings) -> io::Result<()>, publish: impl FnOnce(&Settings),
) -> io::Result<Roster> {
    let committed = crate::settings::update_checked_with(current, |next| change(next, request), persist, publish)?;
    Ok(Roster::from(&committed))
}

pub(crate) fn preserve_incoming(current: &Settings, incoming: &mut Settings) {
    incoming.main_pill = current.main_pill.clone();
    incoming.active_integrations = current.active_integrations.clone();
}

pub(crate) fn authorize_window(actual: &str, island: &str, settings: &str) -> io::Result<()> {
    if actual.is_empty() || ![island, settings].contains(&actual) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "pet roster requires a native preferences surface"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "pet_roster_settings_tests.rs"]
mod tests;
