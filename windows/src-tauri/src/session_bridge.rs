//! Private NDJSON transport for the two reviewed native-session bridges.
//! Backend paths and identities cannot be selected by the frontend.

use std::fs::{File, Metadata, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{timeout, timeout_at, Instant};

use crate::session_chat::types::{Capabilities, Origin, Snapshot, Summary, Target};

#[derive(Clone, Copy)]
pub struct Backend {
    provider: &'static str,
    pet_id: &'static str,
    directory: &'static str,
    label: &'static str,
    empty_cwd: bool,
}

pub const COPILOT: Backend = Backend {
    provider: "copilot", pet_id: "agent_copilot", directory: "copilot", label: "Copilot", empty_cwd: false,
};
pub const GRAVITY: Backend = Backend {
    provider: "antigravity", pet_id: "agent_antigravity", directory: "antigravity", label: "Gravity", empty_cwd: true,
};

impl Backend {
    fn error(self, template: &str) -> String { template.replace("{backend}", self.label) }
}
const MAX_METADATA: u64 = 16 * 1024;
const MAX_REQUEST: usize = 1024 * 1024;
const MAX_HISTORY: usize = 128 * 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const HISTORY_TIMEOUT: Duration = Duration::from_secs(35);
const SEND_TIMEOUT: Duration = Duration::from_secs(70);
const MAX_EVENTS_BEFORE_REPLY: usize = 256;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MetadataRecord {
    provider: String,
    session_id: String,
    cwd: String,
    pid: u32,
    parent_pid: u32,
    capabilities: Capabilities,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    updated_at: Option<i64>,
    #[serde(default)]
    state: Option<String>,
}

struct Endpoint {
    backend: Backend,
    socket: PathBuf,
    socket_metadata: Metadata,
    updated_at: i64,
    record: MetadataRecord,
}

struct Connection {
    backend: Backend,
    socket: BufReader<UnixStream>,
    target: Target,
}

/// Discover only private, currently listening extension endpoints. A terminal
/// index without this extension's metadata and socket grants no capability.
pub async fn list(backend: Backend) -> Vec<Summary> {
    let Ok(directory) = directory(backend) else { return Vec::new(); };
    let Ok(entries) = std::fs::read_dir(&directory) else { return Vec::new(); };
    let mut summaries = Vec::new();
    let mut connecting = JoinSet::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue; };
        let Some(id) = name.strip_suffix(".json") else { continue; };
        let target = Target {
            pet_id: backend.pet_id.into(),
            provider: backend.provider.into(),
            origin: Origin::External,
            session_id: id.into(),
            conversation_id: None,
        };
        let Ok(endpoint) = endpoint(backend, &target) else { continue; };
        connecting.spawn(async move {
            // A stale socket inode is not proof that the terminal remains alive.
            if connect(&endpoint).await.is_err() { return None; }
            let record = endpoint.record;
            Some(Summary {
                target,
                title: record.title.filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| backend.error("Sessão {backend}")),
                cwd: record.cwd,
                updated_at: record.updated_at.unwrap_or(endpoint.updated_at),
                // Minimal metadata does not advertise whether a turn is running.
                state: record.state.unwrap_or_else(|| "available".into()),
                model: None,
                capabilities: record.capabilities,
            })
        });
        if connecting.len() >= 16 {
            if let Some(Ok(Some(summary))) = connecting.join_next().await { summaries.push(summary); }
        }
    }
    while let Some(result) = connecting.join_next().await {
        if let Ok(Some(summary)) = result { summaries.push(summary); }
    }
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)
        .then_with(|| a.target.session_id.cmp(&b.target.session_id)));
    summaries
}

pub async fn history(backend: Backend, target: &Target) -> Result<Snapshot, String> {
    validate_target(backend, target)?;
    let ticket = crate::pet_capabilities::begin_native(target);
    let result = async {
        let mut connection = Connection::open(backend, target, false).await?;
        let value = connection.request("history", None, MAX_HISTORY).await?;
        parse_snapshot(backend, target, value)
    }.await;
    crate::pet_capabilities::finish_native(ticket, result.as_ref().ok());
    result
}

/// Sends once to the captured terminal session. An ambiguous disconnect is an
/// error; the caller must consult history rather than replay the mutation.
pub async fn send(backend: Backend, target: &Target, query: &str) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Err(backend.error("Escreve uma mensagem para esta sessão {backend}."));
    }
    if query.len() >= MAX_REQUEST {
        return Err(backend.error("O pedido {backend} excede o limite de 1 MiB."));
    }
    crate::pet_capabilities::invalidate_native(target);
    let result = async {
        let mut connection = Connection::open(backend, target, true).await?;
        connection.request("send", Some(query), MAX_REQUEST).await
    }.await;
    crate::pet_capabilities::invalidate_native(target);
    result
}

/// The installed SDK cannot cancel with a compare-and-swap turn identity.
/// Never substitute a session-wide abort for a request aimed at one turn.
pub async fn cancel(backend: Backend, target: &Target, _turn: Option<&str>) -> Result<Value, String> {
    validate_target(backend, target)?;
    Err(backend.error("Esta versão do {backend} não permite interromper com segurança um turno exato. Usa o terminal desta sessão."))
}

/// Owns a connection receiving exact-target invalidations. Dropping this value
/// closes only the observer; it never aborts the user's native terminal.
pub struct Subscription {
    updates: mpsc::Receiver<Target>,
    task: JoinHandle<()>,
}

impl Subscription {
    pub async fn recv(&mut self) -> Option<Target> { self.updates.recv().await }
}

impl Drop for Subscription {
    fn drop(&mut self) { self.task.abort(); }
}

/// Establishes an observer using the existing history request, then retains the
/// same connection for `updated` notifications. No mutation or replay occurs.
pub async fn subscribe(backend: Backend, target: &Target) -> Result<Subscription, String> {
    let mut connection = Connection::open(backend, target, false).await?;
    let value = connection.request("history", None, MAX_HISTORY).await?;
    parse_snapshot(backend, target, value)?;
    let (updates, receiver) = mpsc::channel(1);
    let task = tokio::spawn(async move {
        loop {
            let packet = match read_packet(backend, &mut connection.socket, MAX_REQUEST).await {
                Ok(packet) => packet,
                Err(_) => break,
            };
            if !updated(&packet, &connection.target) { break; }
            crate::pet_capabilities::invalidate_native(&connection.target);
            // Coalesce repeated invalidations rather than storing transcripts.
            match updates.try_send(connection.target.clone()) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => break,
            }
        }
        // Invalidate once on disconnect so the UI stops claiming a live link.
        crate::pet_capabilities::invalidate_native(&connection.target);
        let _ = updates.try_send(connection.target.clone());
    });
    Ok(Subscription { updates: receiver, task })
}

impl Connection {
    async fn open(backend: Backend, target: &Target, sending: bool) -> Result<Self, String> {
        let endpoint = endpoint(backend, target)?;
        if !endpoint.record.capabilities.history {
            return Err(backend.error("A extensão desta sessão {backend} não disponibiliza histórico completo."));
        }
        if sending && !endpoint.record.capabilities.send {
            return Err(backend.error("Esta sessão {backend} permite apenas leitura."));
        }
        Ok(Self { backend, socket: BufReader::new(connect(&endpoint).await?), target: target.clone() })
    }

    async fn request(&mut self, method: &str, query: Option<&str>, limit: usize) -> Result<Value, String> {
        let backend = self.backend;
        let id = format!("mochi-{}-{}-{}", backend.provider, std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let mut request = json!({"id": id, "method": method, "target": self.target});
        if let Some(query) = query { request["query"] = Value::String(query.into()); }
        let mut bytes = serde_json::to_vec(&request).map_err(|_| backend.error("Não foi possível preparar o pedido {backend}."))?;
        if bytes.len() >= MAX_REQUEST {
            return Err(backend.error("O pedido {backend} excede o limite de 1 MiB."));
        }
        bytes.push(b'\n');
        let deadline = Instant::now() + if method == "send" { SEND_TIMEOUT } else { HISTORY_TIMEOUT };
        timeout_at(deadline, async {
            self.socket.get_mut().write_all(&bytes).await
                .map_err(|_| backend.error("A ligação {backend} terminou durante o envio. Consulta o histórico antes de repetir."))?;
            self.socket.get_mut().flush().await
                .map_err(|_| backend.error("Não foi possível confirmar o envio {backend}. Consulta o histórico antes de repetir."))?;
            let mut remaining = limit;
            for _ in 0..=MAX_EVENTS_BEFORE_REPLY {
                let (mut packet, consumed) = read_packet_counted(backend, &mut self.socket, remaining).await?;
                remaining = remaining.checked_sub(consumed)
                    .ok_or_else(|| backend.error("A resposta {backend} excede o limite de leitura."))?;
                if packet.get("method").is_some() {
                    if !updated(&packet, &self.target) {
                        return Err(backend.error("A extensão {backend} devolveu uma notificação inválida."));
                    }
                    crate::pet_capabilities::invalidate_native(&self.target);
                    continue;
                }
                if packet.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                    return Err(backend.error("A resposta {backend} não corresponde ao pedido enviado."));
                }
                if packet.get("error").is_some() {
                    let code = packet["error"]["code"].as_str().unwrap_or("request_refused");
                    let message = match code {
                        "busy" | "session_busy" => backend.error("Esta sessão {backend} já está a executar uma tarefa."),
                        "cancel_unavailable" => backend.error("Esta versão do {backend} não permite interromper um turno exato."),
                        "history_unavailable" => backend.error("O histórico completo desta sessão {backend} não está disponível."),
                        "history_too_large" => backend.error("O histórico {backend} excede o limite de 128 MiB; não foi apresentado histórico parcial."),
                        "live_history_overflow" => backend.error("O {backend} ainda não confirmou todo o texto público recebido. Atualiza o histórico antes de continuar."),
                        "session_unavailable" => backend.error("A sessão {backend} já não está disponível através da ligação local."),
                        "not_sent" => backend.error("O pedido não foi enviado à sessão {backend}. Atualiza o histórico antes de enviar."),
                        "send_status_unknown" => backend.error("A entrega à sessão {backend} está por confirmar. Consulta o histórico e não reenvies automaticamente."),
                        "model_unavailable" => backend.error("Muda o modelo desta sessão no cliente {backend} nativo; o painel preserva a sua configuração."),
                        _ => backend.error("A sessão {backend} recusou o pedido. Consulta o histórico antes de repetir."),
                    };
                    return Err(message);
                }
                let value = packet.as_object_mut().and_then(|packet| packet.remove("result"))
                    .ok_or_else(|| backend.error("A extensão {backend} devolveu uma resposta incompleta."))?;
                validate_result(backend, &self.target, &value)?;
                return Ok(value);
            }
            Err(backend.error("A extensão {backend} enviou demasiadas notificações sem responder."))
        }).await.map_err(|_| backend.error("A sessão {backend} não confirmou o resultado dentro do prazo. Consulta o histórico antes de repetir."))?
    }
}

fn validate_target(backend: Backend, target: &Target) -> Result<(), String> {
    target.validate_external(backend.provider, backend.pet_id)
}

fn validate_result(backend: Backend, target: &Target, value: &Value) -> Result<(), String> {
    let received: Target = serde_json::from_value(value.get("target").cloned().unwrap_or(Value::Null))
        .map_err(|_| backend.error("A resposta {backend} não identifica a sessão de destino."))?;
    if &received != target {
        return Err(backend.error("A resposta {backend} pertence a outra sessão."));
    }
    Ok(())
}

fn parse_snapshot(backend: Backend, target: &Target, value: Value) -> Result<Snapshot, String> {
    validate_result(backend, target, &value)?;
    let snapshot: Snapshot = serde_json::from_value(value)
        .map_err(|_| backend.error("A extensão {backend} devolveu um histórico público inválido."))?;
    if !snapshot.capabilities.history || snapshot.capabilities.cancel {
        return Err(backend.error("A extensão {backend} anunciou capacidades incompatíveis com esta ligação."));
    }
    if snapshot.turn.as_ref().is_some_and(|turn| turn.is_empty() || turn.len() > 1024 || turn.chars().any(char::is_control)) {
        return Err(backend.error("A extensão {backend} devolveu um identificador de turno inválido."));
    }
    Ok(snapshot)
}

fn updated(value: &Value, target: &Target) -> bool {
    value.get("id").is_none()
        && value.get("method").and_then(Value::as_str) == Some("updated")
        && serde_json::from_value::<Target>(value.get("target").cloned().unwrap_or(Value::Null))
            .is_ok_and(|received| received == *target)
}

fn directory(backend: Backend) -> Result<PathBuf, String> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "A sessão gráfica não disponibiliza uma pasta de execução privada.".to_string())?;
    private_directory(backend, &runtime)?;
    let coucou = runtime.join("coucou");
    private_directory(backend, &coucou)?;
    let bridge = coucou.join(backend.directory);
    private_directory(backend, &bridge)?;
    Ok(bridge)
}

fn private_directory(backend: Backend, path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| backend.error("A extensão local do {backend} ainda não está ligada nesta sessão."))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o777 != 0o700 {
        return Err(backend.error("A pasta da extensão {backend} tem de ser privada (0700) e pertencer ao utilizador."));
    }
    Ok(())
}

fn endpoint(backend: Backend, target: &Target) -> Result<Endpoint, String> {
    validate_target(backend, target)?;
    let directory = directory(backend)?;
    let path = directory.join(format!("{}.json", target.session_id));
    let before = std::fs::symlink_metadata(&path)
        .map_err(|_| backend.error("Esta sessão {backend} não tem uma extensão local ligada."))?;
    private_file(backend, &before)?;
    let mut file: File = OpenOptions::new().read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(&path).map_err(|_| backend.error("Não foi possível ler os metadados privados do {backend}."))?;
    let metadata = file.metadata().map_err(|_| backend.error("Não foi possível verificar os metadados {backend}."))?;
    private_file(backend, &metadata)?;
    if before.dev() != metadata.dev() || before.ino() != metadata.ino() {
        return Err(backend.error("A sessão {backend} mudou durante a descoberta. Atualiza a lista."));
    }
    let mut data = Vec::new();
    (&mut file).take(MAX_METADATA + 1).read_to_end(&mut data)
        .map_err(|_| backend.error("Não foi possível ler os metadados {backend}."))?;
    if data.len() as u64 > MAX_METADATA { return Err(backend.error("Os metadados {backend} excedem o limite permitido.")); }
    let record: MetadataRecord = serde_json::from_slice(&data)
        .map_err(|_| backend.error("Os metadados da sessão {backend} são inválidos."))?;
    if record.provider != backend.provider || record.session_id != target.session_id
        || record.pid == 0 || record.parent_pid == 0
        || (!Path::new(&record.cwd).is_absolute() && !(backend.empty_cwd && record.cwd.is_empty()))
        || record.cwd.chars().any(char::is_control)
        || (record.capabilities.send && !record.capabilities.history)
        || record.capabilities.cancel {
        return Err(backend.error("Os metadados {backend} não correspondem a uma sessão controlável."));
    }
    let socket = directory.join(format!("{}.sock", target.session_id));
    let socket_metadata = std::fs::symlink_metadata(&socket)
        .map_err(|_| backend.error("O terminal desta sessão {backend} já não disponibiliza a ligação local."))?;
    if !socket_metadata.file_type().is_socket()
        || socket_metadata.uid() != unsafe { libc::geteuid() }
        || socket_metadata.mode() & 0o077 != 0 {
        return Err(backend.error("O socket {backend} tem de ser privado e pertencer ao utilizador."));
    }
    Ok(Endpoint { backend, socket, socket_metadata, updated_at: metadata.mtime(), record })
}

fn private_file(backend: Backend, metadata: &Metadata) -> Result<(), String> {
    if !metadata.is_file() || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o777 != 0o600 || metadata.len() > MAX_METADATA {
        return Err(backend.error("Os metadados {backend} têm de ser um ficheiro privado (0600), regular e limitado a 16 KiB."));
    }
    Ok(())
}

async fn connect(endpoint: &Endpoint) -> Result<UnixStream, String> {
    let backend = endpoint.backend;
    let stream = timeout(CONNECT_TIMEOUT, UnixStream::connect(&endpoint.socket)).await
        .map_err(|_| backend.error("O terminal {backend} não respondeu à ligação local."))?
        .map_err(|_| backend.error("O terminal desta sessão {backend} já não está ligado."))?;
    let credentials = stream.peer_cred()
        .map_err(|_| backend.error("Não foi possível verificar o processo {backend} local."))?;
    if credentials.uid() != unsafe { libc::geteuid() }
        || credentials.pid().map(|pid| pid as u32) != Some(endpoint.record.pid) {
        return Err(backend.error("O socket {backend} pertence a outro processo ou utilizador."));
    }
    let after = std::fs::symlink_metadata(&endpoint.socket)
        .map_err(|_| backend.error("O socket {backend} mudou durante a ligação."))?;
    if after.dev() != endpoint.socket_metadata.dev() || after.ino() != endpoint.socket_metadata.ino() {
        return Err(backend.error("A sessão {backend} mudou durante a ligação. Atualiza o histórico."));
    }
    Ok(stream)
}

async fn read_packet(backend: Backend, socket: &mut BufReader<UnixStream>, limit: usize) -> Result<Value, String> {
    read_packet_counted(backend, socket, limit).await.map(|(value, _)| value)
}

async fn read_packet_counted(backend: Backend, socket: &mut BufReader<UnixStream>, limit: usize) -> Result<(Value, usize), String> {
    read_packet_with_deadline(backend,socket,limit,HISTORY_TIMEOUT).await
}

// An idle subscription can wait indefinitely. Once the first byte arrives the
// complete NDJSON frame must follow within a fixed, non-renewing deadline.
async fn read_packet_with_deadline(backend: Backend, socket: &mut BufReader<UnixStream>, limit: usize, frame_timeout:Duration) -> Result<(Value, usize), String> {
    let mut data = Vec::new();
    let mut deadline=None;
    loop {
        let read=socket.fill_buf();
        let buffer=match deadline {
            Some(deadline)=>timeout_at(deadline,read).await.map_err(|_|backend.error("A extensão {backend} não terminou a mensagem dentro do prazo."))?,
            None=>read.await,
        }.map_err(|_| backend.error("A ligação à sessão {backend} foi interrompida."))?;
        if buffer.is_empty() { return Err(backend.error("A ligação à sessão {backend} terminou. Consulta o histórico antes de repetir.")); }
        if deadline.is_none(){deadline=Some(Instant::now()+frame_timeout);}
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map(|index| index + 1).unwrap_or(buffer.len());
        if data.len().checked_add(consumed).is_none_or(|size| size > limit) {
            return Err(backend.error("A resposta {backend} excede o limite de leitura; não foi apresentado histórico parcial."));
        }
        data.extend_from_slice(&buffer[..consumed]);
        socket.consume(consumed);
        if newline.is_some() {
            let count = data.len();
            let value = serde_json::from_slice(&data)
                .map_err(|_| backend.error("A extensão {backend} devolveu uma mensagem inválida."))?;
            return Ok((value, count));
        }
    }
}

#[cfg(test)]mod tests{
 use super::*;
 #[tokio::test]async fn partial_subscription_frame_has_a_fixed_deadline(){
  let(mut writer,reader)=UnixStream::pair().unwrap();let mut reader=BufReader::new(reader);
  writer.write_all(b"{").await.unwrap();
  let result=read_packet_with_deadline(COPILOT,&mut reader,1024,Duration::from_millis(20)).await;
  assert!(result.unwrap_err().contains("prazo"));
 }
 #[tokio::test]async fn a_quiet_subscription_does_not_expire_before_a_frame_starts(){
  let(mut writer,reader)=UnixStream::pair().unwrap();let mut reader=BufReader::new(reader);
  let send=tokio::spawn(async move{tokio::time::sleep(Duration::from_millis(40)).await;writer.write_all(b"{\"idle\":true}\n").await.unwrap();});
  let(value,_)=read_packet_with_deadline(COPILOT,&mut reader,1024,Duration::from_millis(20)).await.unwrap();
  assert_eq!(value["idle"],true);send.await.unwrap();
 }
}
