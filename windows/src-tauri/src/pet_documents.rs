//! A real OS drop grants one attachment to the conversation captured at drop time.
//! Tokens, source paths and copy tickets stay in RAM. Clearing context preserves
//! the inbox copy and the separate document/email permission grant.
use std::{collections::BTreeMap, path::{Component, Path}, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
use ring::rand::{SecureRandom, SystemRandom};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

const DROP_TTL: Duration = Duration::from_secs(120);
const MAX_DROPS: usize = 64;
const MAX_PATHS: usize = 32;
const MAX_PATH_BYTES: usize = 4096;
const MAX_DROP_BYTES: usize = 64 * 1024;
const MAX_ATTACHMENTS: usize = 128;
const CHAT_PROVIDERS: &[&str] = &["codex", "copilot", "antigravity", "ollama", "lmstudio", "anthropic", "openai", "google"];
const DROP_EXPIRED: &str = "A entrega do documento expirou ou foi cancelada. Volta a arrastá-lo para o pet.";
const CONTEXT_CHANGED: &str = "A conversa ou o documento selecionado mudou durante a cópia. O ficheiro ficou na caixa de entrada e não foi anexado a outra conversa.";
const CONTEXT_REVOKED: &str = "Este documento já não está selecionado nesta conversa. Volta a entregá-lo ao pet antes de o enviar.";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    #[serde(flatten)] pub file: crate::files::DroppedFile,
    pub provider: String,
    pub conversation_id: String,
    pub sequence: u64,
}
impl Attachment {
    pub fn chat_context(&self) -> crate::chat::ChatContext {
        crate::chat::ChatContext::File { name: self.file.name.clone(), path: self.file.path.clone() }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Ticket { provider: String, id: String, sequence: u64, drop_id: String }
struct Capture {
    sequence: u64,
    at: Instant,
    generation: u64,
    revisions: BTreeMap<String, u64>,
}
/// Provisional identity at native mouse release. No paths or usable grant exist
/// until GTK returns the requested URI data and the revisions still match.
pub struct NativeDrop {
    capture: Capture,
    owners: BTreeMap<String, String>,
}
struct DropGrant {
    sequence: u64,
    at: Instant,
    paths: Vec<String>,
    owners: BTreeMap<String, String>,
}
struct Ready { attachment: Attachment, drop_id: String }
#[derive(Default)]
struct Store {
    next: u64,
    generation: u64,
    revisions: BTreeMap<String, u64>,
    drops: BTreeMap<String, DropGrant>,
    // Retained after completion/cancellation so an older captured drop cannot
    // supersede a newer claimed one when IPC calls arrive out of order.
    latest: BTreeMap<String, u64>,
    pending: BTreeMap<String, Ticket>,
    files: BTreeMap<(String, String), Ready>,
}
impl Store {
    fn expire(&mut self, now: Instant) {
        self.drops.retain(|_, grant| now.saturating_duration_since(grant.at) < DROP_TTL);
    }

    fn prepare_capture(&mut self, providers: &[String], now: Instant) -> Result<Capture, String> {
        self.expire(now);
        if self.drops.len() >= MAX_DROPS { return Err("Há demasiadas entregas pendentes. Cancela uma entrega ou aguarda que expire.".into()); }
        self.next = self.next.checked_add(1).ok_or("Não foi possível reservar esta entrega.")?;
        Ok(Capture {
            sequence: self.next, at: now, generation: self.generation,
            revisions: providers.iter().map(|p| (p.clone(), *self.revisions.get(p).unwrap_or(&0))).collect(),
        })
    }

    fn finish_capture(&mut self, capture: Capture, token: String, paths: Vec<String>, mut owners: BTreeMap<String, String>, now: Instant) -> Result<String, String> {
        validate_paths(&paths)?;
        self.expire(now);
        if now.saturating_duration_since(capture.at) >= DROP_TTL || capture.generation != self.generation { return Err(DROP_EXPIRED.into()); }
        owners.retain(|provider, _| capture.revisions.get(provider).is_some_and(|revision| *revision == *self.revisions.get(provider).unwrap_or(&0)));
        if owners.is_empty() { return Err("A conversa mudou durante a entrega. Volta a arrastar o documento para o pet.".into()); }
        if self.drops.len() >= MAX_DROPS || self.drops.contains_key(&token) { return Err("Não foi possível reservar esta entrega. Volta a arrastar o documento.".into()); }
        self.drops.insert(token.clone(), DropGrant { sequence: capture.sequence, at: capture.at, paths, owners });
        Ok(token)
    }

    fn claim(&mut self, token: &str, provider: &str, path: &str, expected: Option<&str>, current: &str, now: Instant) -> Result<Ticket, String> {
        self.expire(now);
        let grant = self.drops.get(token).ok_or(DROP_EXPIRED)?;
        // Substituted paths/owners cannot consume the real owner's grant. Once
        // both match, the entire OS drop is used exactly once, even on failure.
        if !grant.paths.iter().any(|p| p == path) || !grant.owners.contains_key(provider) { return Err("Este documento não foi entregue a este pet nesta conversa.".into()); }
        let grant = self.drops.remove(token).ok_or(DROP_EXPIRED)?;
        let id = grant.owners.get(provider).ok_or(DROP_EXPIRED)?;
        if id != current || expected.is_some_and(|expected| expected != id) { return Err("A conversa mudou desde a entrega. Volta a arrastar o documento para esta conversa.".into()); }
        if self.latest.get(provider).is_some_and(|sequence| grant.sequence <= *sequence) { return Err("Já selecionaste um documento mais recente para este pet.".into()); }
        let ticket = Ticket { provider: provider.into(), id: id.clone(), sequence: grant.sequence, drop_id: token.into() };
        self.latest.insert(provider.into(), ticket.sequence);
        self.pending.insert(provider.into(), ticket.clone());
        Ok(ticket)
    }

    fn failed(&mut self, ticket: &Ticket) {
        if self.pending.get(&ticket.provider) == Some(ticket) { self.pending.remove(&ticket.provider); }
    }

    fn finish(&mut self, ticket: &Ticket, current: &str, file: crate::files::DroppedFile) -> Result<Attachment, String> {
        if ticket.id != current || self.pending.get(&ticket.provider) != Some(ticket) || self.latest.get(&ticket.provider) != Some(&ticket.sequence) {
            self.failed(ticket);
            return Err(CONTEXT_CHANGED.into());
        }
        let key = (ticket.provider.clone(), ticket.id.clone());
        if self.files.len() >= MAX_ATTACHMENTS && !self.files.contains_key(&key) {
            self.failed(ticket);
            return Err("O limite de documentos selecionados foi atingido. A cópia ficou na caixa de entrada.".into());
        }
        self.pending.remove(&ticket.provider);
        let attachment = Attachment { file, provider: ticket.provider.clone(), conversation_id: ticket.id.clone(), sequence: ticket.sequence };
        self.files.insert(key, Ready { attachment: attachment.clone(), drop_id: ticket.drop_id.clone() });
        Ok(attachment)
    }

    fn invalidate(&mut self, provider: Option<&str>) {
        if let Some(provider) = provider {
            if crate::pet_config::integration_pet(provider).is_some() {
                let revision = self.revisions.entry(provider.into()).or_default();
                *revision = revision.saturating_add(1);
            }
            self.pending.remove(provider);
            self.files.retain(|(p, _), _| p != provider);
            self.drops.retain(|_, grant| { grant.owners.remove(provider); !grant.owners.is_empty() });
        } else {
            self.generation = self.generation.saturating_add(1);
            self.drops.clear();
            self.pending.clear();
            self.files.clear();
        }
    }

    fn context(&self, provider: &str, id: &str) -> Option<Attachment> {
        self.files.get(&(provider.into(), id.into())).map(|ready| ready.attachment.clone())
    }

    fn validate(&self, provider: &str, id: &str, path: &str, sequence: u64) -> Result<(), String> {
        if self.files.get(&(provider.into(), id.into())).is_some_and(|ready| {
            let a = &ready.attachment;
            a.provider == provider && a.conversation_id == id && a.file.path == path && a.sequence == sequence
        }) { Ok(()) } else { Err(CONTEXT_REVOKED.into()) }
    }

    fn clear(&mut self, provider: &str, id: &str, sequence: u64) -> bool {
        let key = (provider.into(), id.into());
        if self.files.get(&key).is_some_and(|ready| ready.attachment.sequence == sequence) {
            self.files.remove(&key);
            true
        } else { false }
    }

    fn cancel_drop(&mut self, provider: &str, token: &str) -> bool {
        let mut changed = false;
        if let Some(grant) = self.drops.get_mut(token) { changed |= grant.owners.remove(provider).is_some(); }
        if self.drops.get(token).is_some_and(|grant| grant.owners.is_empty()) { self.drops.remove(token); }
        if self.pending.get(provider).is_some_and(|ticket| ticket.drop_id == token) {
            self.pending.remove(provider);
            changed = true;
        }
        self.files.retain(|(owner, _), ready| {
            let remove = owner == provider && ready.drop_id == token;
            changed |= remove;
            !remove
        });
        changed
    }
}

fn validate_paths(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() || paths.len() > MAX_PATHS || paths.iter().map(String::len).sum::<usize>() > MAX_DROP_BYTES || paths.iter().any(|text| {
        text.is_empty() || text.len() > MAX_PATH_BYTES || text.chars().any(char::is_control) || !Path::new(text).is_absolute()
            || Path::new(text).components().any(|c| matches!(c, Component::ParentDir))
            || text.split('/').any(|component| component == ".")
    }) { Err("A entrega contém um caminho inválido ou demasiados documentos.".into()) } else { Ok(()) }
}

fn configured_providers(settings: &crate::settings::Settings) -> Vec<String> {
    if !settings.companion_mode {
        let provider = if settings.chat_provider.is_empty() { crate::chat::ANTHROPIC } else { settings.chat_provider.as_str() };
        return crate::pet_config::integration_pet(provider).map(|_| vec![provider.into()]).unwrap_or_default();
    }
    CHAT_PROVIDERS.iter().chain(crate::pet_config::SERVICE_PETS.iter()).filter_map(|provider| {
        let pet = crate::pet_config::integration_pet(provider)?;
        (pet == settings.main_pill || settings.active_integrations.iter().any(|id| id == pet)).then(|| (*provider).into())
    }).collect()
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(Mutex::default)
}
fn changed(app: &AppHandle, provider: Option<&str>, id: Option<&str>) {
    let _ = app.emit("pet-document-context-changed", serde_json::json!({"provider":provider,"conversationId":id}));
}
pub fn invalidate(app: &AppHandle, provider: Option<&str>) {
    store().lock().unwrap().invalidate(provider);
    changed(app, provider, None);
}

/// Called at the native drop signal, before requesting asynchronous URI data.
/// This reserves identity only, never path access or a token usable by IPC.
pub fn prepare_drop(app: &AppHandle) -> Result<NativeDrop, String> {
    let providers = {
        let shared = app.state::<crate::Shared>();
        // This runs inside GTK's completed-drop callback. A settings fsync
        // elsewhere must not make the whole desktop wait for our drop target.
        let settings = shared.settings.try_lock().map_err(|_| "As preferências do pet estão a ser guardadas. Aguarda um instante e volta a entregar o documento.".to_string())?;
        configured_providers(&settings)
    };
    if providers.is_empty() { return Err("Configura um pet com conversa antes de entregar o documento.".into()); }
    let capture = store().lock().unwrap().prepare_capture(&providers, Instant::now())?;
    let chat = app.state::<crate::chat::Chat>();
    // One nonblocking snapshot at the real OS drop, never owners discovered
    // later by an asynchronous worker after the selected conversation changed.
    let owners = chat.try_conversation_ids(&providers)?;
    Ok(NativeDrop { capture, owners })
}

/// Only the matching native data-received callback can complete this identity.
/// No grant is published if a reset/restore happened while GTK supplied URIs.
pub fn capture_drop(pending: NativeDrop, paths: &[String]) -> Result<String, String> {
    validate_paths(paths)?;
    let mut random = [0u8; 32];
    SystemRandom::new().fill(&mut random).map_err(|_| "Não foi possível autorizar esta entrega.")?;
    let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    store().lock().unwrap().finish_capture(pending.capture, token, paths.to_vec(), pending.owners, Instant::now())
}

pub async fn ingest(app: &AppHandle, provider: String, path: String, expected_id: Option<String>, drop_id: String) -> Result<Attachment, String> {
    let started = Instant::now();
    let configured = {
        let shared = app.state::<crate::Shared>();
        let settings = shared.settings.lock().unwrap();
        configured_providers(&settings).iter().any(|p| p == &provider)
    };
    if !configured { return Err("Este pet não está configurado para receber este documento.".into()); }
    let id = app.state::<crate::chat::Chat>().conversation_id(&provider)?;
    let claimed = store().lock().unwrap().claim(&drop_id, &provider, &path, expected_id.as_deref(), &id, Instant::now());
    let ticket = claimed.map_err(|error| {
        crate::log::line(format!("document ingest: grant rejected ({provider})")); error
    })?;
    crate::log::line(format!("document ingest: copy started ({provider})"));
    let copy_provider = provider.clone();
    let copied = tauri::async_runtime::spawn_blocking(move || crate::pet_files::ingest(&copy_provider, &path)).await;
    let file = match copied {
        Ok(Ok(file)) => file,
        result => {
            store().lock().unwrap().failed(&ticket);
            crate::log::line(format!("document ingest: copy failed ({provider}, {} ms)", started.elapsed().as_millis()));
            return Err(match result { Ok(Err(error)) => error, _ => "A receção do documento foi interrompida.".into() });
        }
    };
    let current = match app.state::<crate::chat::Chat>().conversation_id(&provider) {
        Ok(current) => current,
        Err(error) => { store().lock().unwrap().failed(&ticket); return Err(error); }
    };
    let finished = store().lock().unwrap().finish(&ticket, &current, file);
    let value = finished.map_err(|error| {
        crate::log::line(format!("document ingest: selection rejected ({provider}, {} ms)", started.elapsed().as_millis())); error
    })?;
    // Operational phases only: never paths, file names, drop tokens or dialogue IDs.
    crate::log::line(format!("document ingest: selected ({provider}, {} ms)", started.elapsed().as_millis()));
    changed(app, Some(&provider), Some(&id));
    Ok(value)
}

fn allowed_label(label: &str) -> bool {
    [crate::island::WINDOW_LABEL, crate::pet_chat::LABEL, crate::companions::LABEL, crate::pet_control::LABEL].contains(&label)
}
fn allowed(window: &tauri::WebviewWindow) -> Result<(), String> {
    if allowed_label(window.label()) { Ok(()) }
    else { Err("Abre a conversa do pet para consultar o documento.".into()) }
}
pub fn context(provider: &str, id: &str) -> Option<Attachment> { store().lock().unwrap().context(provider, id) }
pub fn validate(provider: &str, id: &str, path: &str, sequence: u64) -> Result<(), String> {
    store().lock().unwrap().validate(provider, id, path, sequence)?;
    crate::pet_files::check(provider,path)
}
pub fn consume(app: &AppHandle, value: &Attachment) {
    if store().lock().unwrap().clear(&value.provider, &value.conversation_id, value.sequence) { changed(app, Some(&value.provider), Some(&value.conversation_id)); }
}
#[tauri::command]
pub fn pet_document_context(window: tauri::WebviewWindow, provider: String, conversation_id: String) -> Result<Option<Attachment>, String> {
    allowed(&window)?;
    Ok(context(&provider, &conversation_id))
}
#[tauri::command]
pub fn pet_document_clear(app: AppHandle, window: tauri::WebviewWindow, provider: String, conversation_id: String, sequence: u64) -> Result<(), String> {
    allowed(&window)?;
    if store().lock().unwrap().clear(&provider, &conversation_id, sequence) { changed(&app, Some(&provider), Some(&conversation_id)); }
    Ok(())
}
#[tauri::command]
pub async fn pet_document_cancel_drop(app: AppHandle, window: tauri::WebviewWindow, provider: String, drop_id: String) -> Result<(), String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let cancelled = store().lock().unwrap().cancel_drop(&provider, &drop_id);
        if cancelled {
            // A cancelled pending drop can leave an earlier attachment selected.
            // Refresh canonical context instead of telling every surface to clear it.
            let id = app.state::<crate::chat::Chat>().conversation_id(&provider)?;
            changed(&app, Some(&provider), Some(&id));
        }
        Ok(())
    }).await.map_err(|_| "Não foi possível confirmar o cancelamento desta entrega.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_pet_document_surfaces_can_access_attachment_context() {
        for label in [crate::island::WINDOW_LABEL, crate::pet_chat::LABEL, crate::companions::LABEL, crate::pet_control::LABEL] {
            assert!(allowed_label(label), "{label}");
        }
        for label in ["", "settings", "hub", "pet-manager", "untrusted", "pet-chat-extra"] {
            assert!(!allowed_label(label), "{label}");
        }
    }
    fn file(name: &str) -> crate::files::DroppedFile { crate::files::DroppedFile { name: name.into(), path: format!("/inbox/{name}"), size: 1, kind: "file".into(), entries: 1 } }
    fn capture(s: &mut Store, token: &str, paths: &[&str], owners: &[(&str, &str)], now: Instant) {
        let providers = owners.iter().map(|(p, _)| (*p).into()).collect::<Vec<_>>();
        let capture = s.prepare_capture(&providers, now).unwrap();
        s.finish_capture(capture, token.into(), paths.iter().map(|p| (*p).into()).collect(), owners.iter().map(|(p, id)| ((*p).into(), (*id).into())).collect(), now).unwrap();
    }
    fn ticket(s: &mut Store, token: &str, provider: &str, id: &str, now: Instant) -> Ticket {
        capture(s, token, &["/source/document.pdf"], &[(provider, id)], now);
        s.claim(token, provider, "/source/document.pdf", Some(id), id, now).unwrap()
    }

    #[test]
    fn reset_before_ingest_revokes_the_old_drop_even_after_restoring_the_same_id() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a.pdf"], &[("ollama", "old"), ("codex", "other")], now);
        s.invalidate(Some("ollama"));
        assert!(s.claim("a", "ollama", "/source/a.pdf", Some("old"), "old", now).is_err());
        assert!(s.claim("a", "codex", "/source/a.pdf", Some("other"), "other", now).is_ok());
    }

    #[test]
    fn provisional_mouse_release_snapshot_has_no_usable_grant_before_uri_delivery() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        assert!(s.drops.is_empty());
        assert!(s.claim("pending", "ollama", "/source/a.pdf", Some("at-release"), "at-release", now).is_err());
        s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now).unwrap();
        assert!(s.claim("completed", "ollama", "/source/a.pdf", Some("later-conversation"), "later-conversation", now).is_err());
        assert!(s.drops.is_empty());
        assert!(s.context("ollama", "later-conversation").is_none());
    }

    #[test]
    fn reset_after_mouse_release_before_uri_delivery_cannot_retarget_the_completed_drop() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        s.invalidate(Some("ollama"));
        assert!(s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now).is_err());
        assert!(s.drops.is_empty());
        assert!(s.pending.is_empty());
        assert!(s.context("ollama", "at-release").is_none());
    }

    #[test]
    fn delayed_uri_delivery_expires_from_mouse_release_without_publishing_a_grant() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        assert!(s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now + DROP_TTL).is_err());
        assert!(s.drops.is_empty());
    }

    #[test]
    fn resets_racing_the_owner_snapshot_cannot_publish_a_revoked_owner() {
        let mut s = Store::default(); let now = Instant::now();
        let providers = vec!["ollama".into(), "codex".into()];
        let pending = s.prepare_capture(&providers, now).unwrap();
        s.invalidate(Some("ollama"));
        s.finish_capture(pending, "a".into(), vec!["/source/a".into()], BTreeMap::from([("ollama".into(), "same".into()), ("codex".into(), "other".into())]), now).unwrap();
        assert!(!s.drops["a"].owners.contains_key("ollama"));
        assert!(s.drops["a"].owners.contains_key("codex"));
        let pending = s.prepare_capture(&providers, now).unwrap();
        s.invalidate(None);
        assert!(s.finish_capture(pending, "b".into(), vec!["/source/a".into()], BTreeMap::from([("codex".into(), "other".into())]), now).is_err());
    }

    #[test]
    fn slow_earlier_copy_cannot_replace_a_newer_document() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let b = ticket(&mut s, "b", "ollama", "one", now);
        s.finish(&b, "one", file("new.pdf")).unwrap();
        assert!(s.finish(&a, "one", file("old.pdf")).is_err());
        assert_eq!(s.context("ollama", "one").unwrap().file.name, "new.pdf");
    }

    #[test]
    fn capture_order_wins_when_ingest_ipc_arrives_in_reverse_order() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/same.pdf"], &[("ollama", "one")], now);
        capture(&mut s, "b", &["/source/same.pdf"], &[("ollama", "one")], now);
        let b = s.claim("b", "ollama", "/source/same.pdf", None, "one", now).unwrap();
        s.finish(&b, "one", file("b.pdf")).unwrap();
        assert!(s.claim("a", "ollama", "/source/same.pdf", None, "one", now).is_err());
        assert_eq!(s.context("ollama", "one").unwrap().sequence, b.sequence);
    }

    #[test]
    fn the_same_path_needs_distinct_one_use_tokens() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        assert!(s.claim("a", "ollama", "/source/document.pdf", None, "one", now).is_err());
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(b.sequence > a.sequence);
        assert_ne!(b.drop_id, a.drop_id);
    }

    #[test]
    fn a_claim_consumes_the_entire_drop_and_requires_exact_path_owner_and_conversation() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a.pdf", "/source/b.pdf"], &[("ollama", "one")], now);
        for (provider, path) in [("codex", "/source/a.pdf"), ("ollama", "/source/../source/a.pdf"), ("ollama", "/private/key"), ("ollama", "/source//a.pdf")] {
            assert!(s.claim("a", provider, path, Some("one"), "one", now).is_err());
        }
        assert!(s.drops.contains_key("a"));
        s.claim("a", "ollama", "/source/a.pdf", Some("one"), "one", now).unwrap();
        assert!(s.claim("a", "ollama", "/source/b.pdf", None, "one", now).is_err());
        capture(&mut s, "b", &["/source/a.pdf"], &[("ollama", "one")], now);
        assert!(s.claim("b", "ollama", "/source/a.pdf", Some("other"), "one", now).is_err());
        assert!(!s.drops.contains_key("b"));
        capture(&mut s, "c", &["/source/a.pdf"], &[("ollama", "one")], now);
        assert!(s.claim("c", "ollama", "/source/a.pdf", None, "other", now).is_err());
    }

    #[test]
    fn expiry_and_limits_are_explicit_without_evicting_a_live_grant() {
        let mut s = Store::default(); let now = Instant::now();
        for i in 0..MAX_DROPS { capture(&mut s, &format!("{i}"), &["/source/a"], &[("ollama", "one")], now); }
        assert!(s.prepare_capture(&["ollama".into()], now).is_err());
        assert_eq!(s.drops.len(), MAX_DROPS);
        assert!(s.claim("0", "ollama", "/source/a", None, "one", now + DROP_TTL).is_err());
        assert!(s.drops.is_empty());
        assert!(s.prepare_capture(&["ollama".into()], now + DROP_TTL).is_ok());
        assert!(validate_paths(&vec!["/source/a".into(); MAX_PATHS + 1]).is_err());
        assert!(validate_paths(&[format!("/{}", "a".repeat(MAX_PATH_BYTES))]).is_err());
        assert!(validate_paths(&vec![format!("/{}", "a".repeat(MAX_PATH_BYTES - 1)); MAX_PATHS]).is_err());
        for path in ["relative", "/source/../a", "/source/./a", "/source/a\0b", "/source/a\nb"] { assert!(validate_paths(&[path.into()]).is_err()); }
    }

    #[test]
    fn failed_or_reset_copies_cannot_repopulate_context_and_leave_other_pets_alone() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let other = ticket(&mut s, "other", "codex", "two", now);
        s.finish(&other, "two", file("other.pdf")).unwrap();
        s.invalidate(Some("ollama"));
        assert!(s.finish(&a, "one", file("old.pdf")).is_err());
        assert!(s.context("ollama", "one").is_none());
        assert!(s.context("codex", "two").is_some());
        let b = ticket(&mut s, "b", "ollama", "one", now);
        let c = ticket(&mut s, "c", "ollama", "one", now);
        s.failed(&b);
        assert!(s.pending.get("ollama") == Some(&c));
        s.failed(&c);
        assert!(s.pending.get("ollama").is_none());
    }

    #[test]
    fn consumption_and_validation_require_the_exact_current_attachment() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let a = s.finish(&a, "one", file("a.pdf")).unwrap();
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence).is_ok());
        assert!(s.validate("codex", "one", &a.file.path, a.sequence).is_err());
        assert!(s.validate("ollama", "other", &a.file.path, a.sequence).is_err());
        assert!(s.validate("ollama", "one", "/inbox/other.pdf", a.sequence).is_err());
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence + 1).is_err());
        assert!(!s.clear("codex", "one", a.sequence));
        assert!(!s.clear("ollama", "one", a.sequence + 1));
        let b = ticket(&mut s, "b", "ollama", "one", now);
        let b = s.finish(&b, "one", file("b.pdf")).unwrap();
        assert!(!s.clear("ollama", "one", a.sequence));
        assert!(s.clear("ollama", "one", b.sequence));
        assert!(s.validate("ollama", "one", &b.file.path, b.sequence).is_err());
        let c = ticket(&mut s, "c", "ollama", "one", now);
        let c = s.finish(&c, "one", file("c.pdf")).unwrap();
        s.invalidate(Some("ollama"));
        assert!(s.validate("ollama", "one", &c.file.path, c.sequence).is_err());
    }

    #[test]
    fn cancel_before_ingest_revokes_only_that_owner_and_cannot_reappear() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a"], &[("ollama", "one"), ("codex", "other")], now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(!s.cancel_drop("ollama", "a"));
        assert!(s.claim("a", "ollama", "/source/a", None, "one", now).is_err());
        assert!(s.claim("a", "codex", "/source/a", None, "other", now).is_ok());
    }

    #[test]
    fn cancelling_an_in_flight_drop_preserves_the_previous_ready_attachment() {
        let mut s = Store::default(); let now = Instant::now();
        let old = ticket(&mut s, "old", "ollama", "one", now);
        let old = s.finish(&old, "one", file("old.pdf")).unwrap();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(s.finish(&a, "one", file("a.pdf")).is_err());
        assert_eq!(s.context("ollama", "one").unwrap().sequence, old.sequence);
    }

    #[test]
    fn cancelling_a_completed_drop_clears_it_without_cancelling_the_new_copy() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let a = s.finish(&a, "one", file("a.pdf")).unwrap();
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence).is_err());
        assert!(s.pending.get("ollama") == Some(&b));
        assert!(s.finish(&b, "one", file("b.pdf")).is_ok());
    }

    #[test]
    fn stale_cancellation_cannot_remove_a_newer_pending_or_ready_document() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(!s.cancel_drop("ollama", "a"));
        assert!(s.pending.get("ollama") == Some(&b));
        let b = s.finish(&b, "one", file("b.pdf")).unwrap();
        assert!(!s.cancel_drop("ollama", "a"));
        assert!(s.finish(&a, "one", file("a.pdf")).is_err());
        assert!(s.validate("ollama", "one", &b.file.path, b.sequence).is_ok());
    }

    #[test]
    fn configured_owners_use_logical_service_and_api_ids() {
        let mut settings = crate::settings::Settings::default();
        settings.companion_mode = true;
        settings.main_pill = "ai_openai".into();
        settings.active_integrations = vec!["integration_spotify".into(), "agent_codex".into()];
        assert_eq!(configured_providers(&settings), vec!["codex", "openai", "integration_spotify"]);
        settings.companion_mode = false;
        settings.chat_provider.clear();
        assert_eq!(configured_providers(&settings), vec!["anthropic"]);
        settings.chat_provider = "unknown".into();
        assert!(configured_providers(&settings).is_empty());
    }
}
