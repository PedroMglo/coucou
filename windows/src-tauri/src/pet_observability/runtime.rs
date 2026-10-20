//! Explicit application bootstrap proposal. No startup is performed by import,
//! hooks, summary queries or module construction. The integrator controls wiring.
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use super::{capture, usage::{LedgerError, PersistenceState, UsageSummary}};

const LEDGER_ID: &str = "mochi-local-usage-v1";
const LEDGER_FILE: &str = "ollama-usage-v1.json";

#[derive(Default)]
struct Boot {
    started: bool,
    starting: bool,
    closing: bool,
    service: Option<capture::Service>,
    issue: Option<LedgerError>,
}
#[derive(Default)]
pub(crate) struct Runtime { boot: Arc<Mutex<Boot>> }

/// Called explicitly once from setup, after the application state exists.
/// Filesystem/bootstrap work runs on a blocking worker, outside chat/GUI paths.
pub(crate) fn start(app: &AppHandle) {
    let _ = app.manage(Runtime::default());
    let boot = app.state::<Runtime>().boot.clone();
    {
        let Ok(mut state) = boot.lock() else { return };
        if state.started || state.closing { return; }
        state.started = true;
        state.starting = true;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = prepare_path().and_then(|path| capture::prepare(capture::Activation {
            path, ledger_id: LEDGER_ID.into(),
        }));
        let mut discard = None;
        if let Ok(mut state) = boot.lock() {
            state.starting = false;
            match result {
                Ok(prepared) if !state.closing => {
                    // No filesystem work in install. Publication and ownership
                    // are linearized with Exit by this same Boot lock.
                    match prepared.install() {
                        Ok(service) => { state.service = Some(service); state.issue = None; },
                        Err(issue) => { state.issue = Some(issue); },
                    }
                },
                Ok(prepared) => { discard = Some(prepared); },
                Err(issue) => { state.issue = Some(issue); },
            }
        } else if let Ok(prepared) = result {
            discard = Some(prepared);
        }
        // A bootstrap finishing after Exit must not install an active service.
        // Dropping the unpublished preparation outside Boot releases its lease;
        // it never exposes an actor or admits new requests.
        drop(discard);
        let _ = app.emit("pet-observability-changed", serde_json::json!({"schemaVersion": 1, "provider": "ollama"}));
    });
}

/// Cached metadata only. Bootstrap failure is unknown, never consumed zero.
pub(crate) fn summarize(app: &AppHandle, provider: &str, owner: Option<&str>) -> UsageSummary {
    let mut summary = capture::summarize(provider, owner);
    if provider != "ollama" || owner.is_some_and(|owner| owner != "ollama") { return summary; }
    let Some(runtime) = app.try_state::<Runtime>() else { return summary };
    match runtime.boot.try_lock() {
        Ok(state) => {
            if let Some(issue) = state.issue {
                summary.persistence.state = PersistenceState::Unknown;
                summary.persistence.issue = Some(issue);
            } else if state.starting || state.closing {
                summary.persistence.state = PersistenceState::Unknown;
            }
        }
        Err(_) => {
            summary.persistence.state = PersistenceState::Unknown;
            summary.persistence.issue = Some(LedgerError::Busy);
        }
    }
    summary
}

/// Exit-only best effort. This bounds the actor waiting phase to 100 ms after
/// admission closes; filesystem IO itself is not cancelled or claimed reaped.
/// Existing worker/guard lifetime keeps the owner lease until actual completion.
pub(crate) fn shutdown_on_exit(app: &AppHandle) {
    let Some(runtime) = app.try_state::<Runtime>() else { return };
    let service = {
        let Ok(mut state) = runtime.boot.lock() else { return };
        // Close capture admission before publishing closing=true. This prevents
        // a post-Exit admission window between releasing Boot and waiting below.
        if let Some(service) = &state.service { let _ = service.shutdown(Duration::ZERO); }
        state.closing = true;
        state.service.take()
    };
    if let Some(service) = service {
        let report = service.shutdown(Duration::from_millis(100));
        if report.incomplete {
            if let Ok(mut state) = runtime.boot.lock() {
                state.issue = report.issue.or(Some(LedgerError::DurabilityUnknown));
            }
        }
        drop(service);
    }
}

#[cfg(not(target_os = "linux"))]
fn prepare_path() -> Result<std::path::PathBuf, LedgerError> { Err(LedgerError::UnsupportedPlatform) }

#[cfg(target_os = "linux")]
fn prepare_path() -> Result<std::path::PathBuf, LedgerError> {
    use std::{fs, os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt}, path::{Component, Path}};
    unsafe extern "C" { fn geteuid() -> u32; }
    let uid = unsafe { geteuid() };
    fn ancestors(path: &Path, uid: u32) -> Result<(), LedgerError> {
        let mut checked = std::path::PathBuf::new();
        for part in path.components() {
            if !matches!(part, Component::RootDir | Component::Normal(_)) { return Err(LedgerError::InvalidInput); }
            checked.push(part.as_os_str());
            let meta = fs::symlink_metadata(&checked).map_err(|_| LedgerError::Io)?;
            if !meta.is_dir() || meta.file_type().is_symlink() { return Err(LedgerError::InvalidData); }
            let mode = meta.permissions().mode();
            if meta.uid() != 0 && meta.uid() != uid { return Err(LedgerError::InvalidData); }
            // Root/user-owned sticky shared ancestors (e.g. /tmp) protect
            // existing entries. Other writable shared ancestors are untrusted.
            if mode & 0o022 != 0 && mode & 0o1000 == 0 { return Err(LedgerError::InvalidData); }
        }
        Ok(())
    }
    let data = crate::platform::local_dir();
    if !data.is_absolute() { return Err(LedgerError::InvalidInput); }
    ancestors(data.parent().ok_or(LedgerError::InvalidInput)?, uid)?;
    let assistant = data.join("assistant");
    let metrics = assistant.join("metrics");
    for path in [&data, &assistant, &metrics] {
        match fs::symlink_metadata(path) {
            Ok(_) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match fs::DirBuilder::new().mode(0o700).create(path) {
                    Ok(()) => {},
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
                    Err(_) => return Err(LedgerError::Io),
                }
            },
            Err(_) => return Err(LedgerError::Io),
        }
        let meta = fs::symlink_metadata(path).map_err(|_| LedgerError::Io)?;
        let mode = meta.permissions().mode() & 0o777;
        if !meta.is_dir() || meta.file_type().is_symlink() || meta.uid() != uid || mode & 0o022 != 0 {
            return Err(LedgerError::InvalidData);
        }
        if path == &metrics && mode != 0o700 { return Err(LedgerError::InvalidData); }
        // Confirm the new/existing directory inode AND its parent entry before
        // the ledger may report durable data. No chmod or old-data migration.
        for directory in [path.as_path(), path.parent().ok_or(LedgerError::InvalidInput)?] {
            fs::OpenOptions::new().read(true).custom_flags(0o400000 | 0o4000 | 0o200000)
                .open(directory).and_then(|file| file.sync_all()).map_err(|_| LedgerError::Io)?;
        }
    }
    // Existing directory permissions and all conversation data remain unchanged.
    // Capture bootstrap rechecks ancestry/UID/types/modes and its stable lease.
    Ok(metrics.join(LEDGER_FILE))
}
