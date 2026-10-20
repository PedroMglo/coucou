//! Guarded metadata read, global scan and NetworkManager software Wi-Fi enable.
//! No Tauri command is registered. Native IO runs only when a backend calls it.
use serde_json::{json, Value};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::AsyncWrite;
use tokio::net::UnixStream;
use tokio::time::{timeout_at, Instant};

#[path="strict_json.rs"]
mod strict_json;

const MAX_BYTES: usize = 512 * 1024;
const MAX_FRAMES: usize = 8;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Unavailable,
    UntrustedPeer,
    Protocol,
    Timeout,
    NativeFailure,
    OriginChanged,
    Unsupported,
}

struct WireReply {
    pub api_version: Option<u64>,
    pub capabilities: Vec<String>,
    // Intentionally neither Debug nor Serialize: a domain parser must select
    // its public fields; callers must not export or log the daemon payload.
    pub result: Value,
}

#[derive(Clone)]
pub(crate) struct Config {
    socket: PathBuf,
    runtime: PathBuf,
    uid: u32,
    trusted_executable: PathBuf,
    trusted_executable_uid: u32,
    deadline: Duration,
}

impl Config {
    pub(crate) fn from_environment() -> Result<Self, Error> {
        let uid = unsafe { libc::geteuid() };
        let runtime = PathBuf::from(format!("/run/user/{uid}"));
        let socket = std::env::var_os("DMS_SOCKET")
            .map(PathBuf::from).ok_or(Error::Unavailable)?;
        let name = socket.file_name().and_then(|n| n.to_str()).ok_or(Error::Unavailable)?;
        let digits = name.strip_prefix("danklinux-")
            .and_then(|s| s.strip_suffix(".sock")).ok_or(Error::Unavailable)?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit())
            || socket.parent() != Some(runtime.as_path()) {
            return Err(Error::UntrustedPeer);
        }
        Ok(Self {
            socket, runtime, uid,
            trusted_executable: PathBuf::from("/usr/bin/dms"),
            trusted_executable_uid: 0,
            deadline: Duration::from_secs(3),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Incarnation { pid: u32, start: u64, device: u64, inode: u64 }

#[derive(Clone, PartialEq, Eq)]
struct Endpoint { device: u64, inode: u64, ctime: i64, ctime_ns: i64 }
#[derive(Clone, PartialEq, Eq)]
struct Source { peer: Incarnation, uid: u32, socket: PathBuf, endpoint: Endpoint }
/// Only minted by this transport from one authenticated, completed RPC stream.
/// No public fields/constructor, serde, Debug, process/endpoint getters or UI ID.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Origin { source: Source, scan_proven: bool, enable_wifi_proven: bool, disable_wifi_proven: bool }

pub(crate) struct Reply {
    pub api_version: Option<u64>,
    pub capabilities: Vec<String>,
    pub result: Value,
    origin: Origin,
}
impl Reply {
    pub(crate) fn origin(&self) -> &Origin { &self.origin }
    pub(crate) fn supports_scan(&self) -> bool { self.origin.scan_proven }
    pub(crate) fn supports_enable_wifi(&self) -> bool { self.origin.enable_wifi_proven }
}
pub(crate) struct ScanAck { origin: Origin }
impl ScanAck { pub(crate) fn origin(&self) -> &Origin { &self.origin } }
pub(crate) struct EnableWifiAck { origin: Origin }
impl EnableWifiAck { pub(crate) fn origin(&self) -> &Origin { &self.origin } }
pub(crate) struct DisableWifiAck { origin: Origin }
impl DisableWifiAck { pub(crate) fn origin(&self) -> &Origin { &self.origin } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WriteState { NotStarted, MayHaveStarted }
/// E is a backend-only guard reason, never a raw native error or renderer input.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Failure<E> {
    Native { reason: Error, write: WriteState },
    Guard(E),
}
impl<E> Failure<E> {
    pub(crate) fn write_state(&self) -> WriteState {
        match self { Self::Native { write, .. } => *write, Self::Guard(_) => WriteState::NotStarted }
    }
}
fn failure<E>(reason: Error, write: WriteState) -> Failure<E> { Failure::Native { reason, write } }
fn finish_result<T, E>(result: Result<T, Failure<E>>, deadline: Instant, write: WriteState) -> Result<T, Failure<E>> {
    if Instant::now() >= deadline { Err(failure(Error::Timeout, write)) } else { result }
}

async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, bytes: &[u8], state: &mut WriteState) -> Result<(), Error> {
    // Once the actual write is attempted, a transport error cannot certify that
    // no bytes/effect reached the peer, including a partially sent JSON frame.
    *state = WriteState::MayHaveStarted;
    writer.write_all(bytes).await.map_err(|_| Error::Unavailable)
}

// The reviewed upstream v1.6.3 server declares API34 and this exact scan wire
// contract. Other versions remain readable but do not grant scan protocol proof.
const SCAN_API: u64 = 34;
fn scan_supported(version: Option<u64>, capabilities: &[String]) -> bool {
    version == Some(SCAN_API) && capabilities.iter().any(|s| s == "network")
}

fn next_request_id(counter: &AtomicU64) -> Result<u64, Error> {
    // The native Go request decoder may represent IDs as a JSON float64.
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
        |n| (n > 0 && n < MAX_SAFE).then(|| n + 1)).map_err(|_| Error::Protocol)
}

async fn connect(config: &Config) -> Result<(UnixStream, Source), Error> {
    let endpoint = check_socket(config)?;
    let stream = UnixStream::connect(&config.socket).await.map_err(|_| Error::Unavailable)?;
    let credentials = stream.peer_cred().map_err(|_| Error::UntrustedPeer)?;
    if credentials.uid() != config.uid { return Err(Error::UntrustedPeer); }
    let pid = credentials.pid().and_then(|p| u32::try_from(p).ok()).ok_or(Error::UntrustedPeer)?;
    let peer = peer_incarnation(pid, config)?;
    if check_socket(config)? != endpoint { return Err(Error::OriginChanged); }
    Ok((stream, Source { peer, uid: credentials.uid(), socket: config.socket.clone(), endpoint }))
}

fn verify_source(stream: &UnixStream, source: &Source, config: &Config) -> Result<(), Error> {
    let credentials = stream.peer_cred().map_err(|_| Error::UntrustedPeer)?;
    let pid = credentials.pid().and_then(|p| u32::try_from(p).ok()).ok_or(Error::UntrustedPeer)?;
    if source.uid != credentials.uid() || source.uid != config.uid || source.peer.pid != pid
        || source.socket != config.socket || peer_incarnation(pid, config)? != source.peer
        || check_socket(config)? != source.endpoint {
        return Err(Error::OriginChanged);
    }
    Ok(())
}

/// Separate scanner framing retains the greeting's prefetch and byte/frame
/// budget. No subscription/events or unrelated pre-write frames are accepted.
#[derive(Default)]
struct ScanReader { pending: Vec<u8>, received: usize, frames: usize }
impl ScanReader {
    async fn frame(&mut self, stream: &mut UnixStream) -> Result<Value, Error> {
        loop {
            if let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
                let bytes: Vec<_> = self.pending.drain(..=end).collect();
                if bytes.len() == 1 { continue; }
                self.frames += 1;
                if self.frames > MAX_FRAMES { return Err(Error::Protocol); }
                return strict_json::decode(&bytes).map_err(|_| Error::Protocol);
            }
            let remaining = MAX_BYTES.checked_sub(self.received).filter(|n| *n > 0).ok_or(Error::Protocol)?;
            let mut bytes = [0u8; 8192];
            let limit = remaining.min(bytes.len());
            let count = stream.read(&mut bytes[..limit]).await.map_err(|_| Error::Unavailable)?;
            if count == 0 { return Err(Error::Unavailable); }
            self.received += count;
            self.pending.extend_from_slice(&bytes[..count]);
        }
    }
    async fn handshake(&mut self, stream: &mut UnixStream) -> Result<(u64, Vec<String>), Error> {
        let frame = self.frame(stream).await?;
        if !frame.is_object() || ["id", "result", "error", "method"].iter().any(|k| frame.get(k).is_some()) {
            return Err(Error::Protocol);
        }
        let (version, capabilities) = greeting(&frame)?.ok_or(Error::Protocol)?;
        if !scan_supported(Some(version), &capabilities) { return Err(Error::Unsupported); }
        // A fresh, unsubscribed stream cannot already have an RPC reply queued.
        if self.pending.iter().any(|b| !b.is_ascii_whitespace()) { return Err(Error::Protocol); }
        Ok((version, capabilities))
    }
    async fn ack(&mut self, stream: &mut UnixStream, id: u64, rpc: Rpc) -> Result<(), Error> {
        loop {
            let frame = self.frame(stream).await?;
            // This RPC stream never subscribes, so changing handshakes/events
            // cannot prove a stable accepted scan after dispatch.
            if frame.get("apiVersion").is_some() { return Err(Error::Protocol); }
            if frame.get("id") != Some(&json!(id)) { continue; }
            if frame.get("error").is_some_and(|v| !v.is_null()) { return Err(Error::NativeFailure); }
            let expected = match rpc {
                Rpc::Scan => json!({"success":true,"message":"scanning"}),
                Rpc::EnableWifi | Rpc::DisableWifi => {
                    let fields=frame.as_object().ok_or(Error::Protocol)?;
                    if fields.len()!=2||!fields.contains_key("id")||!fields.contains_key("result"){return Err(Error::Protocol);}
                    json!({"enabled":matches!(rpc,Rpc::EnableWifi)})
                },
                Rpc::Read => return Err(Error::Protocol),
            };
            if frame.get("result") != Some(&expected) { return Err(Error::Protocol); }
            return Ok(());
        }
    }
}

#[derive(Clone, Copy)]
enum Rpc { Read, Scan, EnableWifi, DisableWifi }

async fn request<E, F>(config: &Config, rpc: Rpc, expected: Option<&Origin>, guard: F) -> Result<Reply, Failure<E>>
where F: FnOnce(&Origin) -> Result<(), E> {
    let deadline = Instant::now() + config.deadline;
    let mut write = WriteState::NotStarted;
    let operation = async {
        if Instant::now() >= deadline { return Err(failure(Error::Timeout, write)); }
        let (mut stream, source) = connect(config).await.map_err(|e| failure(e, write))?;
        if let Some(origin) = expected {
            let proven=match rpc {Rpc::Scan=>origin.scan_proven,Rpc::EnableWifi=>origin.enable_wifi_proven,Rpc::DisableWifi=>origin.disable_wifi_proven,Rpc::Read=>false};
            if !proven { return Err(failure(Error::Unsupported, write)); }
            if source != origin.source { return Err(failure(Error::OriginChanged, write)); }
        }
        let mut scanner = ScanReader::default();
        let handshake = if matches!(rpc, Rpc::Scan | Rpc::EnableWifi | Rpc::DisableWifi) {
            Some(scanner.handshake(&mut stream).await.map_err(|e| failure(e, write))?)
        } else { None };
        let id = next_request_id(&NEXT_ID).map_err(|e| failure(e, write))?;
        let method = match rpc { Rpc::Read => "network.getState", Rpc::Scan => "network.wifi.scan", Rpc::EnableWifi=>"network.wifi.enable", Rpc::DisableWifi=>"network.wifi.disable" };
        let mut bytes = serde_json::to_vec(&json!({"id":id,"method":method})).map_err(|_| failure(Error::Protocol, write))?;
        bytes.push(b'\n');
        // Wait before the final authority checks. The guard never precedes an
        // asynchronous wait for initial writability or native peer authentication.
        stream.writable().await.map_err(|_| failure(Error::Unavailable, write))?;
        verify_source(&stream, &source, config).map_err(|e| failure(e, write))?;
        if Instant::now() >= deadline { return Err(failure(Error::Timeout, write)); }
        let candidate = Origin { source: source.clone(), scan_proven: handshake.is_some(),
            enable_wifi_proven: expected.is_some_and(|o|o.enable_wifi_proven),
            disable_wifi_proven: expected.is_some_and(|o|o.disable_wifi_proven) };
        guard(&candidate).map_err(Failure::Guard)?;
        if Instant::now() >= deadline { return Err(failure(Error::Timeout, write)); }
        write_frame(&mut stream, &bytes, &mut write).await.map_err(|e| failure(e, write))?;
        let raw = if let Some((version, capabilities)) = handshake {
            scanner.ack(&mut stream, id, rpc).await.map_err(|e| failure(e, write))?;
            WireReply { api_version: Some(version), capabilities, result: Value::Null }
        } else {
            read_reply(&mut stream, id).await.map_err(|e| failure(e, write))?
        };
        verify_source(&stream, &source, config).map_err(|e| failure(e, write))?;
        let origin = match rpc {
            Rpc::Read=>Origin {source,scan_proven:scan_supported(raw.api_version,&raw.capabilities),
                enable_wifi_proven:crate::lifecycle::enable_wifi_supported(&raw.result,raw.api_version,&raw.capabilities),
                disable_wifi_proven:crate::lifecycle::disable_wifi_supported(&raw.result,raw.api_version,&raw.capabilities)},
            Rpc::Scan|Rpc::EnableWifi|Rpc::DisableWifi=>candidate,
        };
        Ok(Reply { api_version: raw.api_version, capabilities: raw.capabilities, result: raw.result, origin })
    };
    let result = timeout_at(deadline, operation).await;
    // Tokio polls an already-ready inner future first, including after a slow
    // synchronous guard/final peer check. Keep the reviewed R2 final deadline gate.
    finish_result(result.map_err(|_| failure(Error::Timeout, write))?, deadline, write)
}

/// The backend supplies a real authority guard; no current window/UI fact is
/// acquired here. Metadata reply and its sealed origin come from the same stream.
pub(crate) async fn network_snapshot_guarded<E, F>(config: &Config, guard: F) -> Result<Reply, Failure<E>>
where F: FnOnce() -> Result<(), E> {
    request(config, Rpc::Read, None, |_| guard()).await
}

/// This client intentionally uses the no-device scan form; API34 also supports
/// optional device upstream. There is no targeted argument/fallback in this client.
/// The callback is synchronous at the writer after current native authentication.
pub(crate) async fn scan_wifi<E, F>(config: &Config, expected: &Origin, guard: F) -> Result<ScanAck, Failure<E>>
where F: FnOnce(&Origin) -> Result<(), E> {
    if !expected.scan_proven { return Err(failure(Error::Unsupported, WriteState::NotStarted)); }
    let reply = request(config, Rpc::Scan, Some(expected), guard).await?;
    Ok(ScanAck { origin: reply.origin })
}

/// NetworkManager's global SOFTWARE property, never a device selector or RF proof.
pub(crate) async fn enable_wifi<E,F>(config:&Config,expected:&Origin,guard:F)->Result<EnableWifiAck,Failure<E>>
where F:FnOnce(&Origin)->Result<(),E>{
    if !expected.enable_wifi_proven{return Err(failure(Error::Unsupported,WriteState::NotStarted));}
    let reply=request(config,Rpc::EnableWifi,Some(expected),guard).await?;
    Ok(EnableWifiAck {origin:reply.origin})
}
pub(crate) async fn disable_wifi<E,F>(config:&Config,expected:&Origin,guard:F)->Result<DisableWifiAck,Failure<E>>
where F:FnOnce(&Origin)->Result<(),E>{
    if !expected.disable_wifi_proven{return Err(failure(Error::Unsupported,WriteState::NotStarted));}
    let reply=request(config,Rpc::DisableWifi,Some(expected),guard).await?;
    Ok(DisableWifiAck {origin:reply.origin})
}

// The unguarded compatibility helper is fixture-only; production must supply a
// backend guard. It has no native/CLI probe entry point in this proposal crate.
#[cfg(test)]
async fn network_snapshot(config: &Config) -> Result<Reply, Error> {
    network_snapshot_guarded(config, || Ok::<_, ()>(())).await.map_err(|e| match e {
        Failure::Native { reason, .. } => reason,
        Failure::Guard(()) => Error::Protocol,
    })
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "scan_tests.rs"]
mod scan_tests;

fn process_start_time(text: &str) -> Option<u64> {
    text.rsplit_once(") ")?.1.split_whitespace().nth(19)?.parse().ok()
}

fn check_socket(config: &Config) -> Result<Endpoint, Error> {
    let directory = std::fs::symlink_metadata(&config.runtime).map_err(|_| Error::Unavailable)?;
    if !directory.is_dir() || directory.uid() != config.uid || directory.mode() & 0o077 != 0 {
        return Err(Error::UntrustedPeer);
    }
    let socket = std::fs::symlink_metadata(&config.socket).map_err(|_| Error::Unavailable)?;
    if config.socket.parent() != Some(config.runtime.as_path()) || !socket.file_type().is_socket()
        || socket.uid() != config.uid || socket.mode() & 0o777 != 0o600 {
        return Err(Error::UntrustedPeer);
    }
    Ok(Endpoint { device: socket.dev(), inode: socket.ino(), ctime: socket.ctime(), ctime_ns: socket.ctime_nsec() })
}

fn peer_incarnation(pid: u32, config: &Config) -> Result<Incarnation, Error> {
    if pid < 2 { return Err(Error::UntrustedPeer); }
    let proc = PathBuf::from(format!("/proc/{pid}"));
    let executable = proc.join("exe").canonicalize().map_err(|_| Error::UntrustedPeer)?;
    let trusted = config.trusted_executable.canonicalize().map_err(|_| Error::UntrustedPeer)?;
    let metadata = std::fs::metadata(&executable).map_err(|_| Error::UntrustedPeer)?;
    if executable != trusted || !metadata.is_file() || metadata.uid() != config.trusted_executable_uid
        || metadata.mode() & 0o022 != 0 || metadata.mode() & 0o111 == 0 {
        return Err(Error::UntrustedPeer);
    }
    let text = std::fs::read_to_string(proc.join("stat")).map_err(|_| Error::UntrustedPeer)?;
    let start = process_start_time(&text).filter(|n| *n > 0).ok_or(Error::UntrustedPeer)?;
    Ok(Incarnation { pid, start, device: metadata.dev(), inode: metadata.ino() })
}

fn greeting(frame: &Value) -> Result<Option<(u64, Vec<String>)>, Error> {
    if frame.get("apiVersion").is_none() { return Ok(None); }
    let version = frame["apiVersion"].as_u64().ok_or(Error::Protocol)?;
    let rows = frame["capabilities"].as_array().ok_or(Error::Protocol)?;
    if rows.len() > 256 { return Err(Error::Protocol); }
    let capabilities = rows.iter().map(|v| {
        let text = v.as_str().ok_or(Error::Protocol)?;
        if text.is_empty() || text.len() > 128 || text.chars().any(char::is_control) {
            return Err(Error::Protocol);
        }
        Ok(text.to_string())
    }).collect::<Result<Vec<_>, _>>()?;
    Ok(Some((version, capabilities)))
}

fn finish_before_deadline<T>(result: Result<T, Error>, deadline: Instant) -> Result<T, Error> {
    if Instant::now() >= deadline { Err(Error::Timeout) } else { result }
}

async fn read_reply(stream: &mut UnixStream, id: u64) -> Result<WireReply, Error> {
    let mut pending = Vec::new();
    let mut received = 0usize;
    let mut frames = 0usize;
    let mut api_version = None;
    let mut capabilities = Vec::new();
    loop {
        if let Some(end) = pending.iter().position(|b| *b == b'\n') {
            let bytes: Vec<_> = pending.drain(..=end).collect();
            if bytes.len() == 1 { continue; }
            frames += 1;
            if frames > MAX_FRAMES { return Err(Error::Protocol); }
            let frame: Value = strict_json::decode(&bytes).map_err(|_| Error::Protocol)?;
            if frame.get("id") == Some(&json!(id)) {
                if frame.get("error").is_some_and(|v| !v.is_null()) { return Err(Error::NativeFailure); }
                let result = frame.get("result").cloned().ok_or(Error::Protocol)?;
                return Ok(WireReply { api_version, capabilities, result });
            }
            if let Some((version, values)) = greeting(&frame)? {
                api_version = Some(version);
                capabilities = values;
            }
            continue;
        }
        let remaining = MAX_BYTES.checked_sub(received).filter(|n| *n > 0).ok_or(Error::Protocol)?;
        let mut bytes = [0u8; 8192];
        let limit = remaining.min(bytes.len());
        let count = stream.read(&mut bytes[..limit]).await.map_err(|_| Error::Unavailable)?;
        if count == 0 { return Err(Error::Unavailable); }
        received += count;
        pending.extend_from_slice(&bytes[..count]);
    }
}


// Executor harness only. Original R3 source above is an exact byte prefix.
#[cfg(test)]
pub(crate) fn config_for_fixture(socket: PathBuf, runtime: PathBuf, trusted_executable: PathBuf, uid: u32, deadline: Duration) -> Config {
    Config {socket,runtime,uid,trusted_executable,trusted_executable_uid:uid,deadline}
}


/// Read-only domain result: deliberately has no network Origin, target scope or
/// mutation proof. Socket authentication and framing are shared with network.
pub(crate) struct BluetoothReply {
    pub(crate) api_version: Option<u64>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) result: Value,
}

/// Fixed query only. No method, parameters, subscription or selector is supplied
/// by a renderer. Original context is checked at the writer after peer checks.
pub(crate) async fn bluetooth_snapshot_guarded<E, F>(config: &Config, guard: F) -> Result<BluetoothReply, Failure<E>>
where F: FnOnce() -> Result<(), E> {
    let deadline = Instant::now() + config.deadline;
    let mut write = WriteState::NotStarted;
    let operation = async {
        let (mut stream, source) = connect(config).await.map_err(|e| failure(e, write))?;
        let id = next_request_id(&NEXT_ID).map_err(|e| failure(e, write))?;
        let mut bytes = serde_json::to_vec(&json!({"id":id,"method":"bluetooth.getState"})).map_err(|_| failure(Error::Protocol, write))?;
        bytes.push(b'\n');
        stream.writable().await.map_err(|_| failure(Error::Unavailable, write))?;
        verify_source(&stream, &source, config).map_err(|e| failure(e, write))?;
        if Instant::now() >= deadline { return Err(failure(Error::Timeout, write)); }
        guard().map_err(Failure::Guard)?;
        verify_source(&stream, &source, config).map_err(|e| failure(e, write))?;
        if Instant::now() >= deadline { return Err(failure(Error::Timeout, write)); }
        write_frame(&mut stream, &bytes, &mut write).await.map_err(|e| failure(e, write))?;
        let reply = read_reply(&mut stream, id).await.map_err(|e| failure(e, write))?;
        verify_source(&stream, &source, config).map_err(|e| failure(e, write))?;
        Ok(BluetoothReply {api_version:reply.api_version,capabilities:reply.capabilities,result:reply.result})
    };
    let result = timeout_at(deadline, operation).await;
    finish_result(result.map_err(|_| failure(Error::Timeout, write))?, deadline, write)
}
