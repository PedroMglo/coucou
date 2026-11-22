//! On-demand navigation lookup. No resume, subscription, model request or
//! process discovery. Inventory reads use existing verified native endpoints.
use std::{sync::Mutex, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};
use super::{core::{InventoryInput, LibraryCapture, PetBinding}, lookup::{self, LookupRequest, LookupResult, LookupState}};

const QUERY_TIMEOUT: Duration = Duration::from_secs(20);
const QUERY_COOLDOWN: Duration = Duration::from_secs(10);

struct Context { bindings: Vec<PetBinding>, library: LibraryCapture }
struct Gate { started: Option<Instant>, running: bool }
static CODEX: Mutex<Gate> = Mutex::new(Gate { started: None, running: false });
static COPILOT: Mutex<Gate> = Mutex::new(Gate { started: None, running: false });
struct Lease(&'static Mutex<Gate>);
struct Backoff { reason: &'static str, retry_after_ms: u64 }
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut gate) = self.0.lock() { gate.running = false; }
    }
}

fn acquire(provider: &str) -> Result<Lease, Backoff> {
    let gate = match provider { "codex" => &CODEX, "copilot" => &COPILOT,
        _ => return Err(Backoff { reason: "nativeInventoryNotSupported", retry_after_ms: 1000 }) };
    let mut state = gate.try_lock().map_err(|_| Backoff { reason: "nativeLookupBusy", retry_after_ms: 1000 })?;
    if state.running { return Err(Backoff { reason: "nativeLookupBusy", retry_after_ms: 1000 }); }
    if let Some(remaining) = state.started.and_then(|stamp| QUERY_COOLDOWN.checked_sub(stamp.elapsed())) {
        return Err(Backoff { reason: "nativeLookupThrottled", retry_after_ms: (remaining.as_millis() as u64).max(1) });
    }
    state.started = Some(Instant::now());
    state.running = true;
    drop(state);
    Ok(Lease(gate))
}

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn scope(provider: &str) -> &'static str {
    match provider { "codex" => "codexDaemon", "copilot" => "copilotLiveEndpoints", _ => "providerGlobal" }
}

fn allowed(window: &tauri::WebviewWindow) -> Result<(), String> {
    if ![crate::island::WINDOW_LABEL, crate::companions::LABEL, crate::pet_control::LABEL,
        crate::pet_chat::LABEL, crate::pet_panel::LABEL, crate::HUB_LABEL].contains(&window.label()) {
        return Err("Abre um pet para consultar a sessão.".into());
    }
    Ok(())
}

async fn context(app: &AppHandle) -> Result<Context, String> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Unlike read(), this accessor never creates a missing configuration.
        let config = crate::pet_config::read_existing()
            .map_err(|_| "As identidades dos pets estão indisponíveis.".to_string())?;
        let bindings = {
            let shared = app.state::<crate::Shared>();
            let settings = shared.settings.lock()
                .map_err(|_| "As definições estão indisponíveis.".to_string())?;
            config.pets.into_iter().filter(|(_, pet)| !pet.provider.is_empty()).map(|(id, pet)| {
                let owner = if crate::pet_config::service(&id) { id.clone() } else { pet.provider.clone() };
                PetBinding { enabled: settings.main_pill == id || settings.active_integrations.contains(&id),
                    pet_id: id, provider: pet.provider, owner }
            }).collect::<Vec<_>>()
        };
        let library = app.state::<crate::chat::Chat>().observability_metadata(&bindings, now());
        Ok::<_, String>(Context { bindings, library })
    }).await.map_err(|_| "Não foi possível consultar a sessão.".to_string())?
}

fn native_authorized(context: &Context, provider: &str) -> bool {
    let canonical = match provider { "codex" => "agent_codex", "copilot" => "agent_copilot", _ => return false };
    context.bindings.iter().any(|binding|
        binding.pet_id == canonical && binding.provider == provider && binding.enabled)
}

#[tauri::command]
pub async fn pet_session_lookup(window: tauri::WebviewWindow, app: AppHandle,
    request: LookupRequest) -> Result<LookupResult, String> {
    allowed(&window)?;
    if !lookup::valid_request(&request) { return Err("O identificador da sessão é inválido.".into()); }
    let initial = context(&app).await?;
    if !lookup::needs_native(&request, &initial.library) || !matches!(request.provider.as_str(), "codex" | "copilot") {
        return Ok(lookup::resolve(request, &initial.bindings, &initial.library, None, now()));
    }

    let provider = &request.provider;
    let (native, backoff) = if !native_authorized(&initial, provider) {
        (InventoryInput::unavailable(scope(provider), "nativePetUnavailable"), None)
    } else {
        match acquire(provider) {
            Err(backoff) => (InventoryInput::unavailable(scope(provider), backoff.reason), Some(backoff)),
            Ok(_lease) => {
                let native = match tokio::time::timeout(QUERY_TIMEOUT,
                    crate::session_chat_adapter::observability_inventory(&app, provider, now())).await {
                    Ok(Ok(input)) => input,
                    Ok(Err(_)) => InventoryInput::unavailable(scope(provider), "nativeInventoryUnavailable"),
                    Err(_) => InventoryInput::unavailable(scope(provider), "nativeInventoryTimeout"),
                };
                (native, None)
            },
        }
    };
    // Config/settings/Chat can change during metadata RPC. Re-read after await:
    // a newly recorded Mochi owner must not be opened as an external session.
    let final_context = context(&app).await?;
    let mut result = lookup::resolve(request, &final_context.bindings, &final_context.library, Some(&native), now());
    if result.state == LookupState::Unavailable && result.reason.as_deref() == Some("nativeInventoryUnavailable") {
        if let Some(backoff) = backoff {
            result.reason = Some(backoff.reason.into());
            result.retry_after_ms = Some(backoff.retry_after_ms);
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "lookup_adapter_tests.rs"]
mod tests;
