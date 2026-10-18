//! Reconcile open applications even when their SessionStart preceded Coucou.
//! Process identities only: no windows, prompts, environment or session files.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Default, Serialize)]
pub struct Presence { running: Vec<String>, activity: BTreeMap<String, u64> }
#[derive(Default)]
struct Watch { value: Mutex<Presence> }

#[tauri::command]
pub fn companions_presence(app: AppHandle) -> Presence {
    app.state::<Watch>().value.lock().unwrap().clone()
}

pub fn setup(app: &AppHandle) {
    app.manage(Watch::default());
    #[cfg(target_os = "linux")]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            let mut before = BTreeMap::new();
            loop {
                if crate::companions::enabled(&app) {
                    let current = scan();
                    let watch = app.state::<Watch>();
                    let mut value = watch.value.lock().unwrap();
                    // A new process of the same provider wakes it; an unchanged
                    // sample never postpones its rest timer.
                    for id in current.keys().chain(before.keys()).collect::<BTreeSet<_>>() {
                        if current.get(id) != before.get(id) { *value.activity.entry(id.clone()).or_default() += 1; }
                    }
                    if current != before {
                        value.running = current.keys().cloned().collect();
                        crate::log::line(format!("companions presence: {} open clients", value.running.len()));
                        let _ = app.emit_to(crate::island::WINDOW_LABEL, "companions-presence", value.clone());
                        before = current;
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        });
    }
}

fn identify(exe: &str, args: &[&str]) -> Option<&'static str> {
    // Persistent transport servers are not an open user conversation.
    if args.iter().any(|arg| matches!(*arg, "app-server" | "app-server-daemon" | "--type=utility" | "--type=renderer" | "--type=gpu-process")) { return None; }
    match exe.to_ascii_lowercase().as_str() {
        "codex" | "codex-desktop" => Some("agent_codex"),
        "agy" | "antigravity" => Some("agent_antigravity"),
        "claude" => Some("integration_claude"),
        "gemini" => Some("agent_gemini"),
        "copilot" => Some("agent_copilot"),
        "cursor" => Some("agent_cursor"),
        "opencode" => Some("agent_opencode"),
        "muse" => Some("agent_muse"),
        "amp" => Some("agent_amp"),
        "ollama" if args.contains(&"run") => Some("ai_ollama"),
        // Script CLIs have a generic interpreter executable. Inspect only
        // the entrypoint path; never interpret arbitrary prompt arguments.
        "node" | "nodejs" | "python3" => {
            let entry = args.first()?;
            if entry.contains("/@openai/codex/") { Some("agent_codex") }
            else if entry.contains("/@google/gemini-cli/") { Some("agent_gemini") }
            else if entry.contains("/@anthropic-ai/claude-code/") { Some("integration_claude") }
            else if entry.contains("/@github/copilot/") { Some("agent_copilot") }
            else { None }
        }
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn scan() -> BTreeMap<String, BTreeSet<u32>> {
    use std::{io::Read, os::unix::fs::MetadataExt};
    let mut result: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir("/proc") else { return result };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
        if !entry.metadata().is_ok_and(|m| m.uid() == unsafe { libc::getuid() }) { continue; }
        let Ok(exe) = std::fs::read_link(entry.path().join("exe")) else { continue };
        let Some(name) = exe.file_name().and_then(|n| n.to_str()) else { continue };
        // Bound the read; nothing read here is logged, emitted or persisted.
        let mut bytes = Vec::new();
        if let Ok(file) = std::fs::File::open(entry.path().join("cmdline")) { let _ = file.take(4096).read_to_end(&mut bytes); }
        let text = String::from_utf8_lossy(&bytes);
        let args: Vec<_> = text.split('\0').skip(1).take(6).collect();
        if let Some(id) = identify(name, &args) { result.entry(id.into()).or_default().insert(pid); }
    }
    result
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn distinguishes_open_clients_from_background_transports() {
        assert_eq!(identify("codex", &["resume"]), Some("agent_codex"));
        assert_eq!(identify("codex", &["app-server", "--listen"]), None);
        assert_eq!(identify("agy", &[]), Some("agent_antigravity"));
        assert_eq!(identify("node", &["/tools/node_modules/@google/gemini-cli/dist/index.js"]), Some("agent_gemini"));
        assert_eq!(identify("node", &["server.js", "codex"]), None);
        assert_eq!(identify("ollama", &["serve"]), None);
        assert_eq!(identify("ollama", &["run", "model"]), Some("ai_ollama"));
        assert_eq!(identify("Antigravity", &["--type=renderer"]), None);
    }
}
