// Coucou for Windows — app wiring and the commands the island calls.

mod assistant_memory;
mod assistant_tasks;
mod pet_management;
mod pet_model;
mod pet_usage;
mod account_usage;
mod account_usage_core;
mod pet_observability;
mod pet_panel;
mod pet_system;
mod pet_center_settings;
mod pet_audio;
mod local_tasks;
mod agent_hooks;
mod agents;
mod chat;
mod claude;
mod codex_chat;
mod cli_chat;
mod copilot_chat;
#[path="copilot_chat/adapter.rs"]
mod copilot_adapter;
mod codex_plan;
mod config_file;
mod companions;
mod pet_control;
mod pet_native_network_core;
use pet_native_network_core::{lifecycle,authority_fence,executor};
#[cfg(target_os="linux")] use pet_native_network_core::transport;
mod pet_native_binding;
mod pet_native_network;
mod pet_bluetooth;
mod pet_chat;
mod session_chat;
mod session_bridge;
mod copilot_session;
#[path="session_chat/adapter.rs"]
mod session_chat_adapter;
mod desktop;
mod files;
mod document_text;
#[cfg(target_os="linux")]mod linux_documents;
mod document_folder;
mod pet_files;
mod pet_documents;
mod navigation;
mod flow;
mod github;
mod gravity_chat;
mod gravity_hook;
mod hooks;
mod i18n;
mod identity;
mod integrations;
mod island;
mod local_chat;
mod log;
mod net;
mod notes;
mod openai_compat;
mod phone_link;
mod pipe;
mod pet_activity;
mod pet_config;
mod pet_presence;
mod platform;
#[cfg(target_os = "linux")]
mod portal;
mod recap;
mod secrets;
mod session_window;
mod settings;
mod shortcuts;
mod sounds;
mod speech;
mod spotify;
mod tray;
mod voice;
#[cfg(windows)]
mod webview_drop;

use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};

use chat::{Chat, ChatContext, ChatReply, ModelInfo};
use hooks::{HookPreview, HookStatus};
use island::{PollGate, ScreenInfo};
use pipe::Pending;
use settings::Settings;

pub struct Shared {
    pub settings: Mutex<Settings>,
    pub gate: Arc<PollGate>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootInfo {
    settings: Settings,
    screen: ScreenInfo,
    version: String,
    hook_path: String,
    /// False where the OS has no global cursor (Wayland): the page then reports
    /// the cursor from its own mouse events.
    cursor_poll: bool,
}

#[tauri::command]
fn boot(app: AppHandle, shared: State<Shared>) -> BootInfo {
    let mut settings = shared.settings.lock().unwrap().clone();
    // The real state of ~/.claude/settings.json wins over whatever we stored.
    let hooks_status = hooks::status();
    settings.hooks_installed = hooks_status.installed;
    settings.plan_relay_installed = hooks_status.plan_relay_installed;
    let screen = island::screen_info(&app, &settings.screen);
    BootInfo {
        settings,
        screen,
        version: env!("CARGO_PKG_VERSION").to_string(),
        hook_path: settings::hook_exe_path().to_string_lossy().to_string(),
        cursor_poll: platform::CURSOR_POLL,
    }
}

#[tauri::command]
fn save_settings(app: AppHandle, shared: State<Shared>, settings: Settings) -> Result<(),String> {
    let (mut screen_changed,mut autostart_changed,mut shortcuts_changed)=(false,false,false);
    let settings=settings::update_with(&shared.settings,|current|{
        screen_changed = current.screen != settings.screen || current.companion_mode != settings.companion_mode;
        autostart_changed = current.autostart != settings.autostart;
        shortcuts_changed = current.shortcuts != settings.shortcuts;
        // Where Mochi sits on the desktop is desktop.rs's to say, not a webview's.
        let mut settings = settings.clone();
        settings.desktop_mochi = current.desktop_mochi.clone();
        // Same for the phone link: only its own commands turn it on or off.
        settings.phone_link = current.phone_link;
        settings.phone_chat = current.phone_chat;
        settings.phone_details = current.phone_details;
        settings.phone_answers = current.phone_answers;
        settings.phone_diffs = current.phone_diffs;
        settings.phone_usage = current.phone_usage;
        settings.phone_services = current.phone_services.clone();
        settings.phone_relay = current.phone_relay;
        settings.phone_relay_url = current.phone_relay_url.clone();
        settings.phone_chat_models = current.phone_chat_models.clone();
        // Ollama choices have their own validated, acknowledged command. A
        // settings page opened before that choice cannot silently undo it.
        if let Some(model)=current.chat_models.get("ollama"){
            settings.chat_models.insert("ollama".into(),model.clone());
        }else{settings.chat_models.remove("ollama");}
        // These flags have a dedicated field patch. A settings page opened
        // earlier cannot overwrite a later confirmed pet-center change.
        pet_center_settings::preserve_incoming(current, &mut settings);
        *current = settings;
    },settings::save,|settings|{pet_native_binding::settings_committed(&app,settings.pet_system_center);let _=app.emit("settings-changed",settings);})
        .map_err(|err|{log::line(format!("could not save settings: {err}"));"Não foi possível guardar as definições. As preferências anteriores foram preservadas.".to_string()})?;
    pet_native_binding::publish_current(&app);
    if autostart_changed {
        let manager = app.autolaunch();
        let result = if settings.autostart { manager.enable() } else { manager.disable() };
        if let Err(err) = result {
            eprintln!("[coucou] autostart: {err}");
        }
    }
    if screen_changed {
        let collapsed = shared.gate.collapsed.load(Ordering::Relaxed);
        island::apply_geometry(&app, &settings.screen, collapsed);
    }
    integrations::settings_saved(&app, &settings.active_integrations);
    spotify::sync(&app, &settings.active_integrations);
    if shortcuts_changed {
        shortcuts::apply(&app, &settings.shortcuts);
    }
    if i18n::set_picked(&settings.language) {
        language_changed(&app);
    }
    // Keep the other window in step (island ⇄ settings window).
    voice::sync(&app);
    Ok(())
}

/// Confirm the requested flags after successful save. Existing settings events
/// reconcile native pet surfaces asynchronously in the island renderer.
#[tauri::command]
async fn pet_center_settings_patch(
    app: AppHandle,
    window: tauri::WebviewWindow,
    request: pet_center_settings::Patch,
) -> Result<pet_center_settings::Flags, String> {
    pet_center_settings::authorize_window(window.label(), HUB_LABEL)
        .map_err(|_| "Esta janela não pode alterar estas preferências.".to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let shared = app.try_state::<Shared>()
            .ok_or_else(|| "As preferências ainda não estão disponíveis.".to_string())?;
        pet_center_settings::apply_with(
            &shared.settings,
            request,
            settings::save,
            |fresh| { pet_native_binding::settings_committed(&app,fresh.pet_system_center);let _ = app.emit("settings-changed", fresh); },
        ).inspect(|_|pet_native_binding::publish_current(&app)).map_err(|err| {
            log::line(format!("could not save pet-center settings: {err}"));
            "Não foi possível guardar estas preferências. Os valores anteriores foram preservados.".to_string()
        })
    }).await.map_err(|_| "Não foi possível concluir a atualização destas preferências.".to_string())?
}

/// The island reports the system's languages at launch, for "System" in
/// Settings → Language (WebView2 and WebKitGTK know them best).
#[tauri::command]
fn set_system_languages(app: AppHandle, languages: Vec<String>) {
    if i18n::set_system(languages) {
        language_changed(&app);
    }
}

/// What Rust labels itself follows the new language: the tray menu and the
/// settings window's title. The webviews switch on their own.
fn language_changed(app: &AppHandle) {
    tray::retitle(app);
    if let Some(window) = app.get_webview_window(HUB_LABEL) {
        let _ = window.set_title("Coucou");
    }
}

/// Close/open explicit cards. Linux keeps only the mapped controller alive,
/// with an empty input region while closed and no passive wake surface.
#[tauri::command]
fn set_collapsed(app: AppHandle, shared: State<Shared>, collapsed: bool) {
    if collapsed{if let Some(win)=island::window(&app){platform::set_activating(&win,false);}}
    let pref = shared.settings.lock().unwrap().screen.clone();
    shared.gate.collapsed.store(collapsed, Ordering::Relaxed);
    // This webview owns background event handlers. Hiding the GTK window
    // suspends WebKit and would prevent a pet from opening its card again.
    island::apply_geometry(&app, &pref, collapsed);
    let keep_controller=cfg!(target_os="linux")||shared.settings.lock().unwrap().companion_mode;
    if keep_controller { if let Some(win) = island::window(&app) { let _ = win.show(); } }
    // Card input follows its actual shape, and a resize invalidates the flag.
    island::refresh_click_through(&app, &shared.gate);
    shared.gate.set_active(!collapsed&&!cfg!(target_os="linux"));
    platform::set_pointer_watch(!collapsed);
}

/// The front end pushes the island shape; Rust decides click-through from it.
#[tauri::command]
fn set_island_rect(app: AppHandle, shared: State<Shared>, x: f64, y: f64, width: f64, height: f64) {
    shared.gate.set_rect(island::IslandRect { x, y, w: width, h: height });
    // Without the cursor poll the input region is the click-through: it follows the island.
    if !platform::CURSOR_POLL {
        island::refresh_click_through(&app, &shared.gate);
    }
}

#[tauri::command]
fn focus_window(app: AppHandle, focused: bool) {
    let Some(win) = island::window(&app) else { return };
    platform::set_activating(&win, focused);
    if focused {
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn reposition(app: AppHandle, shared: State<Shared>) {
    if shared.gate.collapsed.load(Ordering::Relaxed) && (cfg!(target_os="linux")||shared.settings.lock().unwrap().companion_mode) { return; }
    let pref = shared.settings.lock().unwrap().screen.clone();
    let collapsed = shared.gate.collapsed.load(Ordering::Relaxed);
    island::apply_geometry(&app, &pref, collapsed);
}

/// The panel is laid out in CSS px against the window's own scale. If the webview
/// reports a different devicePixelRatio (it ends up bigger than the window and gets
/// cut), correct its zoom so one CSS px is one logical px again.
#[tauri::command]
fn fit_zoom(app: AppHandle, dpr: f64) -> bool {
    let Some(win) = island::window(&app) else { return false };
    let scale = win.scale_factor().unwrap_or(1.0);
    static ZOOM: std::sync::Mutex<f64> = std::sync::Mutex::new(1.0);
    let mut zoom = ZOOM.lock().unwrap();
    if let Some(new_zoom) = island::calculate_zoom_correction(*zoom, scale, dpr) {
        *zoom = new_zoom;
        log::line(format!("webview dpr {dpr} != window scale {scale} — setting zoom {new_zoom:.3}"));
        let _ = win.set_zoom(new_zoom);
        true
    } else {
        false
    }
}


/// The displays the island can be pinned to, for Settings.
#[tauri::command]
fn list_monitors(app: AppHandle) -> Vec<island::MonitorChoice> {
    island::monitor_choices(&app)
}

#[tauri::command]
fn open_url(url: String) {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return;
    }
    platform::open_url(&url);
}

/// "Open terminal" opens the working folder in VS Code when `code` is on PATH,
/// and falls back to the file manager otherwise.
#[tauri::command]
fn open_in_vscode(path: Option<String>) -> bool {
    // No shell anywhere near this. The path is a project folder chosen by
    // whoever is using Claude Code, and a shell would happily read `&`, `^`, `%`
    // or `$` in a folder name as syntax. Finding the launcher ourselves and
    // handing the path over as a separate argument keeps it a path.
    let path = path.filter(|p| !p.is_empty());
    // It arrives in a hook payload: only an existing folder, given by its full
    // path, goes any further. `code` would read `--something` as an option, and
    // xdg-open would launch a file with whatever handles its type.
    if let Some(p) = path.as_deref() {
        let p = std::path::Path::new(p);
        if !(p.is_absolute() && p.is_dir()) {
            return false;
        }
    }
    if let Some(code) = platform::find_on_path("code") {
        let mut cmd = Command::new(code);
        if let Some(p) = path.as_deref() {
            cmd.arg(p);
        }
        if platform::no_console(&mut cmd).spawn().is_ok() {
            return true;
        }
    }
    if let Some(p) = path.as_deref() {
        platform::reveal_folder(p);
    }
    false
}

/// "Open terminal": brings forward the terminal or editor window the session
/// runs in, when it was found (see session_window.rs); otherwise opens the
/// folder in VS Code, as before.
#[tauri::command]
fn open_session(session_id: Option<String>, path: Option<String>) -> bool {
    let Some(owners) = session_id.as_deref().and_then(session_window::lookup) else {
        return open_in_vscode(path);
    };
    let folder = path.as_deref().map(session_window::folder_name).unwrap_or_default().to_string();
    #[cfg(windows)]
    {
        if platform::focus_session_window(&owners, &folder) {
            return true;
        }
        open_in_vscode(path)
    }
    // Linux asks the display server, KWin or the terminal, which can take a
    // moment: never on the UI thread a sync command runs on. VS Code still
    // opens when none of them could bring the window forward.
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(move || {
            if !platform::focus_session_window(&owners, &folder) {
                open_in_vscode(path);
            }
        });
        true
    }
}

/// The Claude Desktop pill's target: the Claude app (Windows only — it has no
/// Linux build).
#[tauri::command]
fn open_claude_desktop() -> bool {
    platform::open_claude_desktop()
}

/// The file behind a live diff, if it may be handed to the editor: an existing
/// regular file given by its full path. Anything else — a relative path, a
/// folder, a path `code` could read as an option — goes no further.
fn diff_file(path: &str) -> Option<&std::path::Path> {
    let p = std::path::Path::new(path);
    (p.is_absolute() && p.is_file()).then_some(p)
}

/// The diff card's ↗: opens the edited file in VS Code when `code` is on PATH,
/// otherwise shows its folder. The file itself is never opened by its type —
/// xdg-open or Explorer would run a script that Claude just wrote.
#[tauri::command]
fn open_file_in_vscode(path: String) -> bool {
    let Some(file) = diff_file(&path) else { return false };
    if let Some(code) = platform::find_on_path("code") {
        let mut cmd = Command::new(code);
        cmd.arg(file);
        if platform::no_console(&mut cmd).spawn().is_ok() {
            return true;
        }
    }
    if let Some(folder) = file.parent().filter(|d| d.is_dir()) {
        platform::reveal_folder(&folder.to_string_lossy());
    }
    false
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Tray → Pause. Paused means paused: the pollers stop talking to the network,
/// not just the island stopping showing things.
#[tauri::command]
fn set_paused(app:AppHandle,paused: bool) {
    integrations::set_paused(paused);
    if paused{pet_chat::pet_chat_hide(app);}
}

// ── Claude Code hooks ─────────────────────────────────────────────────────────

#[tauri::command]
fn hooks_status() -> HookStatus {
    hooks::status()
}

/// Pill ID → whether that agent's hooks reach Coucou. Read-only.
#[tauri::command]
fn agent_hooks_status() -> std::collections::HashMap<String, bool> {
    agent_hooks::status()
}

/// Returns the diff the user has to look at before anything is written.
#[tauri::command]
fn hooks_preview(install: bool) -> Result<HookPreview, String> {
    hooks::preview(install)
}

/// Only ever called from an explicit click in the settings window.
#[tauri::command]
fn hooks_apply(
    app: AppHandle,
    shared: State<Shared>,
    install: bool,
    fingerprint: String,
) -> Result<String, String> {
    // The fingerprint comes from the preview the user actually looked at, so a
    // settings.json that changed in between is refused rather than overwritten.
    let backup = hooks::write(install, &fingerprint)?;
    let updated = {
        let mut current = shared.settings.lock().unwrap();
        current.hooks_installed = install;
        if let Err(err) = settings::save(&current) {
            log::line(format!("could not save settings: {err}"));
        }
        current.clone()
    };
    let _ = app.emit("settings-changed", updated);
    Ok(backup)
}

// ── Other agents' hooks and plugins ──────────────────────────────────────────

#[tauri::command]
fn agent_hooks_list() -> Vec<agents::AgentStatus> {
    agents::list()
}

/// The diff the user has to look at before anything is written.
#[tauri::command]
fn agent_hooks_preview(agent: String, install: bool) -> Result<config_file::Plan, String> {
    agents::preview(&agent, install)
}

/// Only ever called from an explicit click in the settings window, with the
/// fingerprint of the preview the user looked at.
#[tauri::command]
fn agent_hooks_apply(agent: String, install: bool, fingerprint: String) -> Result<String, String> {
    let backups = agents::apply(&agent, install, &fingerprint)?;
    let done = if install { "installed" } else { "removed" };
    log::line(format!("agent hooks {done} for {agent}"));
    Ok(backups)
}

// ── Plan usage ────────────────────────────────────────────────────────────────

/// The diff of putting the plan usage relay into (or taking it out of) the
/// status line, before anything is written.
#[tauri::command]
fn status_line_preview(install: bool) -> Result<HookPreview, String> {
    hooks::status_line_preview(install)
}

/// Only ever called from an explicit click in the settings window.
#[tauri::command]
fn status_line_apply(
    app: AppHandle,
    shared: State<Shared>,
    install: bool,
    fingerprint: String,
) -> Result<String, String> {
    let backup = hooks::status_line_write(install, &fingerprint)?;
    let updated = {
        let mut current = shared.settings.lock().unwrap();
        current.plan_relay_installed = install;
        // As on the Mac: taking the relay out turns the pill off with it.
        if !install {
            current.show_plan_in_notch = false;
        }
        if let Err(err) = settings::save(&current) {
            log::line(format!("could not save settings: {err}"));
        }
        current.clone()
    };
    let _ = app.emit("settings-changed", updated);
    Ok(backup)
}

/// Codex plan usage, asked of the Codex CLI (`codex app-server`) when its pill
/// shows. Off the main thread: it can take a few seconds.
#[tauri::command]
async fn codex_plan_usage() -> Option<serde_json::Value> {
    tauri::async_runtime::spawn_blocking(codex_plan::read).await.ok().flatten()
}

#[tauri::command]
async fn approval_decision(window:tauri::WebviewWindow,app: AppHandle, request_id: String, decision: String)->bool {
    #[cfg(target_os="linux")]
    let accepted=pipe::answer_from_card(app.clone(),window,request_id.clone(),decision.clone()).await;
    #[cfg(not(target_os="linux"))]
    let accepted={let _=window;pipe::answer(&app,&request_id,&decision);true};
    if accepted{recap::record_decision(&app,&request_id,&decision);}
    accepted
}

/// An option picked on the island for a question Claude Code asked.
#[tauri::command]
async fn approval_answer(
    window:tauri::WebviewWindow,
    app: AppHandle,
    request_id: String,
    answers: std::collections::HashMap<String, serde_json::Value>,
)->bool {
    #[cfg(target_os="linux")]
    let accepted=pipe::answer_question_from_card(app.clone(),window,request_id.clone(),answers).await;
    #[cfg(not(target_os="linux"))]
    let accepted={let _=window;pipe::answer_question(&app,&request_id,&answers);true};
    // An answered question is not an Allow / Deny: nothing for the recap.
    if accepted{recap::forget_request(&app,&request_id);}
    accepted
}

/// The island has the card on screen, so the long wait for a human may begin.
/// Until this arrives the relay only waits a few hundred milliseconds, which is
/// what stops a paused or unresponsive island from freezing Claude Code.
#[tauri::command]
fn approval_ack(app: AppHandle, request_id: String) {
    pipe::acknowledge(&app, &request_id);
}

/// Nobody can act on this request — the island is paused, or another card is
/// already up. Claude Code falls back to asking in the terminal immediately.
#[tauri::command]
fn approval_decline(app: AppHandle, request_id: String) {
    recap::forget_request(&app, &request_id);
    pipe::decline(&app, &request_id);
}

// ── Chat, files and secrets ───────────────────────────────────────────────────

/// One chat turn with the provider picked in the chat view. API keys and any
/// file bytes stay on the Rust side.
#[tauri::command]
async fn chat_send(
    app: AppHandle,
    shared: State<'_, Shared>,
    chat: State<'_, Chat>,
    query: String,
    context: Option<ChatContext>,
    provider: Option<String>,
    conversation_id:Option<String>,
    selected_model:Option<String>,
    document_sequence:Option<u64>,
) -> Result<ChatReply, String> {
    let provider=provider.unwrap_or_else(||shared.settings.lock().unwrap().chat_provider.clone());
    let provider=if provider.is_empty(){"anthropic".to_string()}else{provider};
    if provider=="integration_spotify" {return Err("O Spoty controla música e som e não inicia conversas com modelos.".into());}
    if !crate::pet_config::service(&provider) && !["anthropic","openai","google","openrouter","codex","copilot","antigravity","gemini-cli","claude-cli","ollama","lmstudio","custom"].contains(&provider.as_str()) { return Err("Unknown chat provider".into()); }
    // Admission precedes the effective model snapshot. A model ACK that wins
    // the race is visible here; an admitted turn prevents a later selection.
    let _conversation=chat.reserve_conversation(&provider,conversation_id.as_deref())?;
    let mut settings=shared.settings.lock().unwrap().clone();settings.chat_provider=provider;
    if let Some(ChatContext::File{path,..})=&context{
        let id=chat.conversation_id(&settings.chat_provider)?;
        pet_documents::validate(&settings.chat_provider,&id,path,document_sequence.ok_or("Volta a entregar este documento à conversa antes de o enviar.")?)?;
    }
    if let Some(model)=selected_model {
        chat::select_for_turn(&mut settings,model)?;
    }
    chat::send(&app, &chat, &settings, query, context).await
}

/// The models a provider offers, for the picker in the chat view. Only asked
/// once the user picked that provider, and only with its key or address.
#[tauri::command]
async fn chat_models(shared: State<'_, Shared>, provider: String) -> Result<Vec<ModelInfo>, String> {
    let settings = shared.settings.lock().unwrap().clone();
    chat::models(&settings, &provider).await
}

/// Settings → Local models → Connect: does the server answer, and with which models?
#[tauri::command]
async fn local_connect(provider: String, url: String) -> Result<local_chat::Connected, String> {
    local_chat::connect(&provider, &url).await
}

/// Stores the custom server's key for the address typed next to it; it is only
/// ever sent to that address.
#[tauri::command]
fn local_set_key(url: String, key: String) -> Result<(), String> {
    local_chat::set_custom_key(&url, &key)
}

#[tauri::command]
async fn chat_reset(app:AppHandle,chat: State<'_,Chat>, provider: Option<String>)->Result<Option<chat::ConversationView>,String> {
    let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
    if let Some(provider) = provider {
        chat.reset_provider(&provider)?;
        pet_documents::invalidate(&app,Some(&provider));
        let view=chat.conversation(&provider)?;let _=app.emit("assistant-history-changed",&provider);Ok(Some(view))
    } else { chat.reset_checked()?;pet_documents::invalidate(&app,None);pet_chat::pet_chat_hide(app);Ok(None) }
    }).await.map_err(|_|"Não foi possível criar a conversa.".to_string())?
}

#[tauri::command]
async fn chat_cancel(chat: State<'_,Chat>, provider: Option<String>,conversation_id:Option<String>,turn:Option<u64>)->Result<bool,String> {
    let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
    if let Some(provider)=provider {chat.cancel_conversation(&provider,conversation_id.as_deref(),turn)} else {chat.cancel();true}
    }).await.map_err(|_|"Não foi possível confirmar a interrupção.".to_string())
}

#[tauri::command]
async fn chat_history(chat: State<'_,Chat>, provider:String)->Result<Vec<serde_json::Value>,String> {let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||chat.history(&provider)).await.map_err(|_|"Não foi possível consultar o histórico.".to_string())}

#[tauri::command]
async fn chat_conversation(chat:State<'_,Chat>,provider:String)->Result<chat::ConversationView,String> {let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||chat.conversation(&provider)).await.map_err(|_|"Não foi possível consultar a conversa.".to_string())?}

#[tauri::command]
async fn chat_read_conversation(chat:State<'_,Chat>,provider:String,id:String)->Result<chat::ConversationView,String> {let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||chat.read_conversation(&provider,&id)).await.map_err(|_|"Não foi possível consultar a conversa.".to_string())?}

#[tauri::command]
async fn chat_conversations(chat:State<'_,Chat>,provider:String,query:Option<String>)->Result<Vec<chat::ConversationSummary>,String> {let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||chat.conversations(&provider,query.as_deref().unwrap_or(""))).await.map_err(|_|"Não foi possível consultar as conversas.".to_string())?}

#[tauri::command]
async fn chat_restore_conversation(app:AppHandle,chat:State<'_,Chat>,provider:String,id:String)->Result<chat::ConversationView,String> {
    let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
    let view=chat.restore_conversation(&provider,&id)?;pet_documents::invalidate(&app,Some(&provider));let _=app.emit("assistant-history-changed",provider);Ok(view)
    }).await.map_err(|_|"Não foi possível retomar a conversa.".to_string())?
}

/// Copies a dropped file into the inbox and reports its name back.
#[tauri::command]
async fn ingest_file(app:AppHandle,window:tauri::WebviewWindow,path: String,provider:Option<String>,conversation_id:Option<String>,drop_id:Option<String>) -> Result<pet_documents::Attachment, String> {
    if ![island::WINDOW_LABEL,companions::LABEL,pet_chat::LABEL,pet_control::LABEL].contains(&window.label()){return Err("Entrega o documento diretamente ao pet.".into());}
    let provider=provider.unwrap_or_else(||app.state::<Shared>().settings.lock().unwrap().chat_provider.clone());
    pet_documents::ingest(&app,provider,path,conversation_id,drop_id.ok_or("O documento não tem uma entrega ativa. Volta a arrastá-lo para o pet.")?).await
}

/// The island may only ask whether a key exists — never read it.
#[tauri::command]
fn secret_present(key: String) -> bool {
    secrets::present(&key)
}

#[tauri::command]
fn secret_set(app: AppHandle, key: String, value: String) -> Result<(), String> {
    // Bound to its server's address: only local_set_key may store it.
    if key == local_chat::CUSTOM_KEY {
        return Err("use local_set_key".into());
    }
    let before = (key == "github-token").then(|| secrets::get(&key));
    secrets::set(&key, &value)?;
    if key == claude::KEY {
        voice::sync(&app);
    }
    if let Some(before) = before {
        if secrets::get(&key) != before {
            integrations::github_token_changed(&app);
        }
    }
    Ok(())
}

#[tauri::command]
fn secret_clear(app: AppHandle, key: String) -> Result<(), String> {
    let had = key == "github-token" && secrets::present(&key);
    secrets::clear(&key)?;
    if key == claude::KEY {
        voice::sync(&app);
    }
    if had {
        integrations::github_token_changed(&app);
    }
    Ok(())
}

/// Opens the configured n8n instance — the URL lives in the Credential Manager.
#[tauri::command]
fn open_n8n() {
    if let Some(url) = secrets::get("n8n-url") {
        open_url(url);
    }
}

/// Refresh buttons in the integration cards.
#[tauri::command]
async fn refresh_integration(app: AppHandle, id: String) {
    integrations::poll_once(app, &id).await;
}

/// The GitHub card was opened: refetch its `section` ("pulse" or "activity")
/// if it is stale. The pollers still decline while the pill is off or paused.
#[tauri::command]
fn github_refresh(section: String) {
    integrations::github_refresh_if_stale(&section);
}

/// Lets the island write to the same log as the Rust side.
#[tauri::command]
fn log_line(message: String) {
    log::line(format!("ui  {message}"));
}

// ── Global shortcuts ──────────────────────────────────────────────────────────

/// How each global shortcut went: registered, taken by another app, and so on.
#[tauri::command]
fn shortcuts_status(app: AppHandle) -> shortcuts::Report {
    shortcuts::status(&app)
}

/// Settings is recording a new combination: let go of ours meanwhile, so the
/// keys reach the recorder instead of running an action. `false` takes them back.
#[tauri::command]
fn shortcuts_suspend(app: AppHandle, shared: State<Shared>, suspended: bool) {
    if suspended {
        shortcuts::suspend(&app);
    } else {
        let stored = shared.settings.lock().unwrap().shortcuts.clone();
        shortcuts::apply(&app, &stored);
    }
}

// ── Settings window ───────────────────────────────────────────────────────────

/// WebView2 allows exactly one browser environment per app, and its options are
/// fixed by whichever webview is created first. Every window must therefore ask
/// for the *same* arguments as the island (see `additionalBrowserArgs` in
/// tauri.conf.json) — a mismatch makes the second window come up blank, with no
/// error anywhere.
pub(crate) const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required";

/// In a dev build the pages are served by Vite, so the second window needs the
/// absolute dev URL; a bundled build resolves it inside the app bundle.
fn settings_page_url(app: &AppHandle) -> WebviewUrl {
    #[cfg(dev)]
    if let Some(mut base) = app.config().build.dev_url.clone() {
        base.set_path("/settings.html");
        return WebviewUrl::External(base);
    }
    let _ = app;
    WebviewUrl::App("settings.html".into())
}

/// The Coucou window (Flow dictation, the Notetaker and every setting of the
/// island), labelled "settings" since it grew out of the settings window.
pub const HUB_LABEL: &str = "settings";

/// The Coucou window is created hidden at launch and only ever shown and
/// hidden afterwards. A WebView2 window created later — on the main thread or
/// not — silently comes up blank in this app, so the window that works is the
/// one that exists before the island's webview does.
fn create_settings_window(app: &AppHandle) {
    let url = settings_page_url(app);
    match WebviewWindowBuilder::new(app, HUB_LABEL, url)
        .additional_browser_args(BROWSER_ARGS)
        .title("Coucou")
        .inner_size(1100.0, 720.0)
        .min_inner_size(820.0, 560.0)
        .resizable(true)
        .visible(false)
        .center()
        .build()
    {
        Ok(win) => {
            // Closing it must only hide it, or it could never be reopened.
            let hidden = win.clone();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = hidden.emit("hub-visible", false);
                    let _ = hidden.hide();
                }
            });
        }
        Err(err) => log::line(format!("Coucou window failed: {err}")),
    }
}

pub fn show_settings_window(app: &AppHandle) {
    show_hub(app, Some("general"));
}

/// Shows the Coucou window, on `page` when given ("dictation", "notes",
/// "settings", "general", "shortcuts"…).
pub fn show_hub(app: &AppHandle, page: Option<&str>) {
    let Some(win) = app.get_webview_window(HUB_LABEL) else {
        log::line("Coucou window missing");
        return;
    };
    if let Some(page) = page {
        let _ = win.emit("hub-page", page.to_string());
    }
    let _ = win.unminimize();
    let _ = win.show();
    let _ = win.emit("hub-visible", true);
    let _ = win.set_focus();
}

/// The argument the login item starts Coucou with.
const AUTOSTART_ARG: &str = "--autostart";

/// Coucou started with the computer, not opened by the user: the window stays hidden.
pub fn launched_at_login() -> bool {
    std::env::args().any(|a| a == AUTOSTART_ARG)
}

#[tauri::command]
fn open_settings_window(app: AppHandle, page: Option<String>) {
    show_hub(&app, Some(page.as_deref().unwrap_or("general")));
}

fn voice_request(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--voice-session")
}

pub fn run() {
    if gravity_hook::maybe_run() {return;}
    if agents::gravity_config_cli() {return;}
    platform::prepare_environment();
    let loaded = settings::load();
    i18n::set_picked(&loaded.language);
    let gate = Arc::new(PollGate::new());

    let mut builder = tauri::Builder::default()
        .plugin(navigation::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|arg| arg == "--voice-preview") {
                let app = app.clone();
                tauri::async_runtime::spawn(async move { voice::voice_preview(app, None).await; });
                return;
            }
            if voice_request(&argv) { voice::session::toggle(app); return; }
            // `coucou --shortcut <action>`: what a desktop's own keyboard
            // settings run where we can't listen for keys ourselves (Wayland).
            match shortcuts::from_args(&argv) {
                Some(action) => shortcuts::dispatch(app, action),
                // Opening Coucou again (its shortcut, the Start menu) shows its window.
                None => show_hub(app, None),
            }
        }))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![AUTOSTART_ARG])));
    // Where no global shortcut can work, the plugin isn't even started.
    if platform::global_shortcuts_blocked().is_none() {
        builder = builder.plugin(shortcuts::plugin());
    }

    builder
        .manage(Shared {
            settings: Mutex::new(loaded.clone()),
            gate: gate.clone(),
        })
        .manage(Pending::default())
        .manage(account_usage::Runtime::default())
        .manage(Chat::persistent(platform::local_dir().join("assistant").join("conversations.json")))
        .manage(shortcuts::Registry::default())
        .manage(recap::load())
        .invoke_handler(tauri::generate_handler![
            boot,
            save_settings,
            pet_center_settings_patch,
            pet_model::pet_model_select,
            pet_usage::pet_usage,
            account_usage::pet_account_usage,
            account_usage::pet_account_usage_refresh,
            pet_observability::pet_observability,
            pet_observability::pet_session_lookup,
            set_system_languages,
            set_collapsed,
            set_island_rect,
            focus_window,
            reposition,
            fit_zoom,
            list_monitors,
            open_url,
            open_in_vscode,
            open_session,
            open_claude_desktop,
            open_file_in_vscode,
            quit_app,
            hooks_status,
            agent_hooks_status,
            hooks_preview,
            hooks_apply,
            agent_hooks_list,
            agent_hooks_preview,
            agent_hooks_apply,
            status_line_preview,
            status_line_apply,
            codex_plan_usage,
            approval_decision,
            approval_answer,
            approval_ack,
            pipe::approval_presented,
            approval_decline,
            log_line,
            chat_send,
            chat_models,
            cli_chat::assistant_set_workspace,
            cli_chat::cli_agent_answer,
            cli_chat::cli_agent_pending,
            local_connect,
            local_set_key,
            chat_reset,
            chat_cancel,
            chat_history,
            chat_conversation,
            chat_read_conversation,
            chat_conversations,
            chat_restore_conversation,
            pet_management::pet_management,
            pet_management::pet_cached_usage,
            pet_management::station_history::pet_station_history,
            pet_management::pet_focus_session,
            pet_panel::pet_panel_open,
            pet_panel::pet_panel_hide,
            pet_panel::pet_panel_context,
            pet_config::pet_configuration,
            pet_config::pet_roster,
            pet_config::pet_configuration_save,
            assistant_tasks::assistant_tasks,
            assistant_tasks::assistant_cancel_task,
            assistant_tasks::assistant_resume_task,
            ingest_file,
            pet_files::pet_mail_draft,
            pet_documents::pet_document_context,
            pet_documents::pet_document_clear,
            pet_documents::pet_document_cancel_drop,
            secret_present,
            secret_set,
            secret_clear,
            refresh_integration,
            github_refresh,
            open_n8n,
            open_settings_window,
            set_paused,
            shortcuts_status,
            shortcuts_suspend,
            phone_link::phone_link_status,
            phone_link::phone_link_set_enabled,
            phone_link::phone_link_pairing,
            phone_link::phone_link_new_pairing,
            phone_link::phone_link_publish,
            phone_link::phone_link_publish_question,
            phone_link::phone_link_publish_prefs,
            phone_link::phone_answers_status,
            phone_link::phone_answers_set_enabled,
            phone_link::phone_diffs_status,
            phone_link::phone_diffs_set_enabled,
            phone_link::phone_link_send_diff,
            phone_link::phone_usage_status,
            phone_link::phone_usage_set_enabled,
            phone_link::phone_relay_status,
            phone_link::phone_relay_set,
            phone_link::phone_relay_set_access,
            phone_link::phone_link_publish_usage,
            phone_link::phone_services_status,
            phone_link::phone_services_set,
            phone_link::phone_link_publish_services,
            phone_link::phone_details_status,
            phone_link::phone_details_set_enabled,
            phone_link::phone_chat_status,
            phone_link::phone_chat_set_enabled,
            phone_link::phone_chat_set_models,
            recap::recap_history,
            recap::recap_prefs,
            recap::recap_set_enabled,
            recap::recap_set_hide_projects,
            recap::recap_mark_shown,
            recap::recap_clear,
            recap::recap_save_png,
            recap::recap_reveal_saved,
            companions::companions_configure,
            pet_control::pet_control_configure,
            pet_control::pet_control_open,
            pet_system::pet_system_panel,
            pet_system::pet_system_status,
            pet_system::pet_system_tray,
            pet_audio::pet_audio_snapshot,
            pet_audio::pet_audio_control,
            pet_audio::pet_system_audio_snapshot,
            pet_audio::pet_system_audio_control,
            pet_control::pet_control_input,
            pet_control::pet_control_context,
            pet_native_binding::pet_control_navigation_context,
            pet_native_binding::pet_control_navigate,
            pet_bluetooth::pet_system_bluetooth_read,
            pet_native_network::pet_system_network_read,
            pet_native_network::pet_system_network_view,
            pet_native_network::pet_system_network_scan,
            pet_native_network::pet_system_network_enable_wifi,
            pet_native_network::pet_system_network_disable_wifi,
            pet_native_network::pet_system_network_ticket,
            pet_native_network::pet_system_network_cancel,
            pet_chat::pet_chat_open,
            pet_chat::pet_chat_context,
            pet_chat::pet_chat_hide,
            pet_chat::pet_chat_layout,
            pet_chat::pet_chat_rect,
            pet_chat::pet_chat_pointer,
            session_chat_adapter::session_chat_list,
            session_chat_adapter::session_chat_history,
            session_chat_adapter::session_chat_send,
            session_chat_adapter::session_chat_cancel,
            session_chat_adapter::session_chat_managed_create,
            session_chat_adapter::session_chat_managed_list,
            session_chat_adapter::session_chat_managed_inspect,
            session_chat_adapter::session_chat_managed_open,
            session_chat_adapter::session_chat_managed_release,
            companions::companions_input,
            companions::companions_open,
            companions::companions_anchor,
            pet_presence::companions_presence,
            desktop::desktop_mochi_info,
            desktop::desktop_mochi_pick_up,
            desktop::desktop_mochi_carry,
            desktop::desktop_mochi_carry_end,
            desktop::desktop_mochi_drag_begin,
            desktop::desktop_mochi_drag_move,
            desktop::desktop_mochi_drag_end,
            desktop::desktop_mochi_fly_out,
            desktop::desktop_mochi_fly_home,
            desktop::desktop_mochi_set_asleep,
            sounds::custom_sounds,
            sounds::custom_sound,
            sounds::reveal_sounds_folder,
            sounds::reload_sounds,
            spotify::spotify_refresh,
            spotify::spotify_control,
            spotify::spotify_play_item,
            spotify::spotify_open,
            spotify::spotify_installed,
            flow::flow_status,
            flow::flow_set_prefs,
            flow::flow_history,
            flow::flow_delete,
            flow::flow_clear,
            flow::flow_stats,
            flow::flow_words,
            flow::flow_set_words,
            flow::flow_snippets,
            flow::flow_set_snippets,
            flow::flow_styles,
            flow::flow_set_styles,
            flow::flow_running_apps,
            flow::flow_copy,
            flow::flow_paste_again,
            flow::flow_models_download,
            flow::flow_models_remove,
            flow::pill_state,
            flow::pill_hot,
            flow::pill_hover,
            flow::pill_action,
            notes::notes_state,
            notes::notes_start,
            notes::notes_stop,
            notes::notes_rename,
            notes::notes_rename_speaker,
            notes::notes_toggle_todo,
            notes::notes_delete,
            notes::notes_summarize,
            notes::notes_retry,
            notes::notes_markdown,
            notes::notes_export,
            voice::voice_done,
            voice::voice_say_native,
            voice::voice_preview,
            voice::session::voice_session_toggle,
            voice::session::voice_session_start,
            voice::session::voice_learn_alias,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            pet_native_binding::setup(&handle,loaded.pet_system_center);
            pet_observability::runtime::start(&handle);
            app.manage(phone_link::PhoneLink::new(handle.clone()));
            tray::build(&handle)?;
            // Before the island: see create_settings_window.
            create_settings_window(&handle);
            pet_panel::create(&handle);
            // Same rule for Mochi's desktop window and the dictation pill.
            desktop::setup(&handle);
            companions::setup(&handle);
            pet_control::create(&handle);
            pet_chat::create(&handle);
            pet_presence::setup(&handle);
            flow::setup(&handle);
            notes::setup(&handle);

            if let Some(win) = island::window(&handle) {
                #[cfg(target_os="linux")]if let Err(error)=linux_documents::install(&win){log::line(error);}
                // Where Tauri takes the drop itself (Linux), its paths are the
                // ones ingest_file may copy (files.rs).
                win.on_window_event(|event| {
                    if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                        files::allow_dropped(paths.iter().map(|p| p.to_string_lossy().to_string()));
                    }
                });
                #[cfg(target_os="linux")]platform::set_pointer_watch(false);
                platform::make_non_activating(&win);
                #[cfg(windows)]
                webview_drop::install(&handle);
                island::apply_geometry(&handle, &loaded.screen, false);
                let _ = win.show();
            }
            gate.collapsed.store(cfg!(target_os="linux"), Ordering::Relaxed);
            // Nothing drawn yet, so nothing takes the mouse until the page
            // reports the island's shape.
            if !platform::CURSOR_POLL {
                island::refresh_click_through(&handle, &gate);
            }
            #[cfg(target_os="linux")]island::spawn_display_watch(handle.clone());
            #[cfg(not(target_os="linux"))]{
                gate.set_active(true);
                island::spawn_cursor_poll(handle.clone(), gate.clone());
            }

            log::line(format!("--- Coucou {} started ---", env!("CARGO_PKG_VERSION")));
            hooks::ensure_hook_exe(&handle);
            pipe::start(handle.clone());
            phone_link::start_if_enabled(&handle);
            integrations::start(handle.clone());
            spotify::sync(&handle, &loaded.active_integrations);
            shortcuts::apply(&handle, &loaded.shortcuts);
            // A Linux debug/QA launch must not replace the installed login
            // entry with a build-tree executable. Explicit settings changes
            // still manage autostart through their normal action.
            let register_autostart = !cfg!(all(target_os="linux",debug_assertions));
            if loaded.autostart && register_autostart {
                // Login items written before 0.5 start Coucou without --autostart:
                // writing it again adds it, so the window stays hidden at login.
                let _ = handle.autolaunch().enable();
            }
            let open_window = flow::get(&handle).map(|f| f.store.lock().unwrap().prefs.open_at_launch).unwrap_or(true);
            let arguments = std::env::args().collect::<Vec<_>>();
            let voice_requested = voice_request(&arguments);
            let preview_requested = arguments.iter().any(|arg| arg == "--voice-preview");
            if voice_requested || preview_requested {
                let voice_app = handle.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    if preview_requested { voice::voice_preview(voice_app, None).await; }
                    else { voice::session::toggle(&voice_app); }
                });
            }
            if open_window && !launched_at_login() && !voice_requested && !preview_requested {
                show_hub(&handle, None);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Coucou")
        .run(|app, event| {
            // Withdraw the phone link's announcement on the local network before the process ends.
            if let tauri::RunEvent::Exit = event {
                pet_native_binding::shutdown(app);
                pet_system::release_on_exit();
                phone_link::stop_on_exit(app);
                notes::shutdown(app);
                pet_observability::runtime::shutdown_on_exit(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::diff_file;

    #[test]
    fn voice_cli_routes_only_explicit_requests() {
        let args = |words: &[&str]| words.iter().map(|word| word.to_string()).collect::<Vec<_>>();
        assert!(super::voice_request(&args(&["coucou", "--voice-session"])));
        assert!(!super::voice_request(&args(&["coucou", "--shortcut", "voiceTalk"])));
        assert!(!super::voice_request(&args(&["coucou", "--shortcut", "openChat"])));
        assert!(!super::voice_request(&args(&["coucou", "voiceTalk"])));
        assert!(!super::voice_request(&args(&["coucou", "--voice-preview"])));
    }

    #[test]
    fn only_an_existing_file_by_its_full_path_reaches_the_editor() {
        let dir = std::env::temp_dir().join(format!("coucou-diff-file-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("edited.ts");
        std::fs::write(&file, "x").unwrap();

        assert!(diff_file(&file.to_string_lossy()).is_some());
        // A folder, a missing file, a relative path or an option never pass.
        assert!(diff_file(&dir.to_string_lossy()).is_none());
        assert!(diff_file(&dir.join("missing.ts").to_string_lossy()).is_none());
        assert!(diff_file("edited.ts").is_none());
        assert!(diff_file("--help").is_none());
        assert!(diff_file("").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
