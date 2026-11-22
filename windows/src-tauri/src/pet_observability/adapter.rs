//! Read-only, cached IPC. Native refreshes only query existing endpoints.
use std::{collections::BTreeMap, sync::{Mutex, OnceLock}, time::{Instant, SystemTime, UNIX_EPOCH}};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use super::{cache::Cache, core::{self, Coverage, Inventory, InventoryInput, PetBinding}, usage};

const MAX_AGE: u64 = 15;
const REFRESH_TIMEOUT: u64 = 20;
fn cache() -> &'static Mutex<Cache> { static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new(); CACHE.get_or_init(Mutex::default) }
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

struct Completion(Option<super::cache::Ticket>, AppHandle, String);
impl Drop for Completion {
    fn drop(&mut self) {
        if let Some(ticket) = self.0.take() {
            cache().lock().unwrap().finish(ticket, Err("refreshCancelled".into()));
            let _ = self.1.emit("pet-observability-changed", serde_json::json!({"schemaVersion": 1, "provider": self.2}));
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    schema_version: u32,
    generated_at: u64,
    pets: Vec<PetObservation>,
    providers: Vec<ProviderObservation>,
    live_pet_work: super::process_census::LiveWork,
    /// Nested capability dates are milliseconds; inventory dates remain seconds.
    pet_capabilities: Option<crate::pet_capabilities::Snapshot>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PetObservation { pet_id: String, provider: String, enabled: bool, mochi: Inventory, usage: usage::UsageSummary }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderObservation { provider: String, pet_ids: Vec<String>, native: Inventory, usage: usage::UsageSummary, process_census: Option<super::process_census::Census> }

fn allowed(window: &tauri::WebviewWindow) -> Result<(), String> {
    if ![crate::island::WINDOW_LABEL, crate::companions::LABEL, crate::pet_control::LABEL,
        crate::pet_chat::LABEL, crate::pet_panel::LABEL, crate::HUB_LABEL].contains(&window.label()) {
        return Err("Abre um pet para consultar as sessões.".into());
    }
    Ok(())
}

fn bindings(app: &AppHandle) -> Result<Vec<PetBinding>, String> {
    let config = crate::pet_config::read_existing()?;
    let shared = app.state::<crate::Shared>();
    let settings = shared.settings.lock().map_err(|_| "As definições estão indisponíveis.".to_string())?;
    Ok(config.pets.into_iter().filter(|(_, pet)| !pet.provider.is_empty()).map(|(id, pet)| {
        let owner = if crate::pet_config::service(&id) { id.clone() } else { pet.provider.clone() };
        PetBinding { enabled: settings.main_pill == id || settings.active_integrations.contains(&id),
            pet_id: id, provider: pet.provider, owner }
    }).collect())
}

/// Starts at most one request/provider every ten seconds, including failures.
/// The first response can be unknown + refreshing; completion emits an event.
fn native_snapshot(app: &AppHandle, provider: &str, enabled: bool) -> Inventory {
    let scope = match provider { "codex" => "codexDaemon", "copilot" => "copilotLiveEndpoints", _ => "providerGlobal" };
    if !matches!(provider, "codex" | "copilot") {
        return Inventory::build(InventoryInput { scope: scope.into(), coverage: Coverage::NotSupported,
            observed_at: None, reason: Some("noProviderSessionInventory".into()), rows: Vec::new() }, now(), MAX_AGE, false);
    }
    if !enabled { return Inventory::build(InventoryInput::unavailable(scope, "nativePetDisabled"), now(), MAX_AGE, false); }
    let (inventory, ticket) = {
        let mut cache = cache().lock().unwrap();
        let ticket = cache.begin(provider, scope, Instant::now());
        let inventory = cache.snapshot(provider, scope, now(), MAX_AGE);
        (inventory, ticket)
    };
    if let Some(ticket) = ticket {
        let app = app.clone(); let provider = provider.to_string();
        // Construct before spawn: cancellation before the first poll must also
        // clear the single-flight slot, without a second native request.
        let mut completion = Completion(Some(ticket), app.clone(), provider.clone());
        tauri::async_runtime::spawn(async move {
            let result = tokio::time::timeout(std::time::Duration::from_secs(REFRESH_TIMEOUT),
                crate::session_chat_adapter::observability_inventory(&app, &provider, now())).await
                .unwrap_or_else(|_| Err("refreshTimeout".into()));
            // Aggregate outside the cache lock once per collection. Keep only
            // counts + the bounded display list, not 100k rows per UI reader.
            let result = result.map(|input| Inventory::build(input, now(), MAX_AGE, false));
            cache().lock().unwrap().finish(completion.0.take().unwrap(), result);
            let _ = app.emit("pet-observability-changed", serde_json::json!({"schemaVersion": 1, "provider": provider}));
        });
    }
    inventory
}

#[tauri::command]
pub async fn pet_observability(window: tauri::WebviewWindow, app: AppHandle) -> Result<Snapshot, String> {
    allowed(&window)?;
    let app_for_metadata = app.clone();
    let (bindings, library, live_pet_work, pet_capabilities) = tauri::async_runtime::spawn_blocking(move || {
        let bindings = bindings(&app_for_metadata)?;
        let library = app_for_metadata.state::<crate::chat::Chat>().observability_metadata(&bindings, now());
        let live_pet_work=app_for_metadata.state::<crate::chat::Chat>().process_census_live_work(&bindings);
        let pet_capabilities=crate::pet_capabilities::passive_snapshot(&app_for_metadata).ok();
        Ok::<_, String>((bindings, library, live_pet_work, pet_capabilities))
    }).await.map_err(|_| "Não foi possível consultar as sessões.".to_string())??;
    let generated_at = now();
    let process_census=super::process_census_runtime::snapshots(&app);
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut pets = Vec::new();
    for binding in &bindings {
        groups.entry(binding.provider.clone()).or_default().push(binding.pet_id.clone());
        let input = library.inventories.get(&binding.pet_id).cloned()
            .unwrap_or_else(|| InventoryInput::unavailable("mochiLibrary", "libraryUnavailable"));
        // Reading a cached summary never activates capture or repeats a request.
        let usage = super::runtime::summarize(&app, &binding.provider, Some(&binding.owner));
        pets.push(PetObservation { pet_id: binding.pet_id.clone(), provider: binding.provider.clone(),
            enabled: binding.enabled, mochi: Inventory::build(input, generated_at, MAX_AGE, false), usage });
    }
    let providers = groups.into_iter().map(|(provider, pet_ids)| {
        let canonical = match provider.as_str() { "codex" => "agent_codex", "copilot" => "agent_copilot", _ => "" };
        let enabled = bindings.iter().any(|b| b.pet_id == canonical && b.provider == provider && b.enabled);
        let mut native = native_snapshot(&app, &provider, enabled);
        core::bind_ownership(&mut native.sessions, &library.ownership, library.ownership_complete);
        let usage = super::runtime::summarize(&app, &provider, None);
        ProviderObservation { process_census:process_census.get(&provider).cloned(), provider, pet_ids, native, usage }
    }).collect();
    Ok(Snapshot { schema_version: core::SCHEMA_VERSION, generated_at, pets, providers, live_pet_work, pet_capabilities })
}
