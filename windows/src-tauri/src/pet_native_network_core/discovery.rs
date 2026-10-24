//! Endpoint metadata from the fixed native UI method. This never grants an
//! action, selects a device, enumerates daemon sockets or changes global env.
use super::{native_socket_path, strict_json, Error};
use std::{fs, io::Read, os::{fd::AsRawFd, unix::{fs::MetadataExt, process::CommandExt}},
    path::{Path, PathBuf}, process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, OnceLock}, time::Duration};
use tokio::time::{timeout_at, Instant};

const MAX_OUTPUT: usize = 4096;
const METHOD: [&str; 4] = ["ipc", "call", "pets-system", "backendEnvironment"];
#[cfg(test)]thread_local!{static DIAGNOSTIC:std::cell::RefCell<Vec<(&'static str,i32)>>=const{std::cell::RefCell::new(vec![])};}
#[cfg(test)]fn diagnostic(stage:&'static str,code:i32){DIAGNOSTIC.with(|d|d.borrow_mut().push((stage,code)));}
#[cfg(test)]fn diagnostics()->Vec<(&'static str,i32)>{DIAGNOSTIC.with(|d|d.borrow().clone())}

#[derive(Clone)]
pub(super) struct Native {
    executable: PathBuf,
    owner: u32,
    #[cfg(test)]
    fixture_interpreter: bool,
    slot: Arc<tokio::sync::Semaphore>,
    // A path is remembered only after a completed, authenticated RPC.
    // It is not an Origin or action proof, and every connection authenticates.
    remembered: Arc<Mutex<Option<PathBuf>>>,
}
impl Native {
    pub(super) fn current() -> Self {
        static NATIVE: OnceLock<Native> = OnceLock::new();
        NATIVE.get_or_init(|| Self { executable: "/usr/bin/dms".into(), owner: 0,
            #[cfg(test)] fixture_interpreter: false,
            slot: Arc::new(tokio::sync::Semaphore::new(1)), remembered: Arc::new(Mutex::new(None)) }).clone()
    }
    pub(super) fn remembered(&self) -> Option<PathBuf> {
        self.remembered.lock().ok().and_then(|p| p.clone())
    }
    pub(super) fn remember_authenticated(&self, socket: &Path) {
        if let Ok(mut p) = self.remembered.lock() { *p = Some(socket.to_owned()); }
    }
    #[cfg(test)]
    pub(super) fn fixture(executable: PathBuf, owner: u32) -> Self {
        Self { executable, owner, fixture_interpreter: true, slot: Arc::new(tokio::sync::Semaphore::new(1)), remembered: Arc::new(Mutex::new(None)) }
    }
    pub(super) async fn endpoint(&self, runtime: &Path, deadline: Instant) -> Result<PathBuf, Error> {
        #[cfg(test)]DIAGNOSTIC.with(|d|d.borrow_mut().clear());
        // One invocation, no metadata retry. Queueing and output share this
        // budget and the caller's original whole-RPC deadline.
        let deadline = deadline.min(Instant::now() + Duration::from_secs(1));
        let _permit = timeout_at(deadline, self.slot.acquire()).await.map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Unavailable)?;
        if Instant::now() >= deadline { return Err(Error::Timeout); }
        let stamp = executable_stamp(&self.executable, self.owner)?;
        // Synthetic scripts are interpreted by a stable installed binary.
        // Directly executing a freshly written image can race delayed close
        // on test filesystems. The private script still receives every stamp,
        // owner/mode, argv and replacement check. Production always execs dms.
        #[cfg(test)]
        let mut command = if self.fixture_interpreter {
            let mut command = Command::new("/usr/bin/python3");
            command.arg("-I").arg(&self.executable); command
        } else { Command::new(&self.executable) };
        #[cfg(not(test))]
        let mut command = Command::new(&self.executable);
        command.args(METHOD).current_dir("/").env_clear().env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).process_group(0);
        for key in ["HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "LC_CTYPE", "DISPLAY",
            "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME",
            "DBUS_SESSION_BUS_ADDRESS", "XDG_SESSION_TYPE", "HYPRLAND_INSTANCE_SIGNATURE"] {
            if let Some(value) = std::env::var_os(key) { command.env(key, value); }
        }
        let mut child = OwnedChild { child: command.spawn().map_err(|e| {#[cfg(test)]diagnostic("spawn",e.raw_os_error().unwrap_or(-1));let _=e;Error::Unavailable})?, reaped: false };
        let mut output = child.child.stdout.take().ok_or(Error::Unavailable)?;
        let fd = output.as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(Error::Unavailable);
        }
        let mut bytes = Vec::new();
        loop {
            if Instant::now() >= deadline { return Err(Error::Timeout); }
            let eof = drain(&mut output, &mut bytes)?;
            if child.exited()? && eof {
                // Keep the leader unreaped until its owned group is stopped.
                let status = child.stop_and_reap()?;
                if !status.success() {#[cfg(test)]{use std::os::unix::process::ExitStatusExt;diagnostic("exit",status.code().unwrap_or_else(||-status.signal().unwrap_or(0)));}return Err(Error::Unavailable); }
                if executable_stamp(&self.executable, self.owner)? != stamp { return Err(Error::OriginChanged); }
                if Instant::now() >= deadline { return Err(Error::Timeout); }
                return parse(&bytes, runtime);
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

fn executable_stamp(path: &Path, owner: u32) -> Result<(u64, u64, i64, i64, u32), Error> {
    let m = fs::symlink_metadata(path).map_err(|_| Error::Unavailable)?;
    if !m.is_file() || m.uid() != owner || m.mode() & 0o022 != 0 || m.mode() & 0o111 == 0 {
        return Err(Error::UntrustedPeer);
    }
    Ok((m.dev(), m.ino(), m.ctime(), m.ctime_nsec(), m.mode()))
}

fn parse(bytes: &[u8], runtime: &Path) -> Result<PathBuf, Error> {
    if bytes.is_empty() || bytes.len() > MAX_OUTPUT { return Err(Error::Protocol); }
    let value = strict_json::decode(bytes).map_err(|_| Error::Protocol)?;
    let fields = value.as_object().ok_or(Error::Protocol)?;
    if fields.len() != 2 || value.get("schemaVersion").and_then(serde_json::Value::as_u64) != Some(1) {
        return Err(Error::Protocol);
    }
    let text = value.get("dmsSocket").and_then(serde_json::Value::as_str).ok_or(Error::Protocol)?;
    let socket = PathBuf::from(text);
    if !native_socket_path(&socket, runtime)
        || socket.file_name().and_then(|n| n.to_str()).map(|n| runtime.join(n).to_string_lossy() != text).unwrap_or(true) {
        return Err(Error::UntrustedPeer);
    }
    Ok(socket)
}

fn drain(output: &mut impl Read, bytes: &mut Vec<u8>) -> Result<bool, Error> {
    let mut buffer = [0u8; 1024];
    loop {
        let remaining = MAX_OUTPUT + 1 - bytes.len();
        if remaining == 0 { return Err(Error::Protocol); }
        let n = remaining.min(buffer.len());
        match output.read(&mut buffer[..n]) {
            Ok(0) => return Ok(true),
            Ok(n) => { bytes.extend_from_slice(&buffer[..n]); if bytes.len() > MAX_OUTPUT { return Err(Error::Protocol); } }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(Error::Unavailable),
        }
    }
}

struct OwnedChild { child: Child, reaped: bool }
impl OwnedChild {
    fn exited(&self) -> Result<bool, Error> {
        let pid = self.child.id() as libc::pid_t;
        loop {
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let result = unsafe { libc::waitid(libc::P_PID, pid as libc::id_t, &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT) };
            if result == 0 { return Ok(unsafe { info.si_pid() } == pid); }
            if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted { return Err(Error::Unavailable); }
        }
    }
    fn stop_and_reap(&mut self) -> Result<ExitStatus, Error> {
        // A successful waitid proves this leader is still our unreaped child;
        // its PID cannot have been reused as an unrelated process group.
        self.exited()?;
        unsafe { libc::kill(-(self.child.id() as libc::pid_t), libc::SIGKILL); }
        let status = self.child.wait().map_err(|_| Error::Unavailable)?;
        self.reaped = true;
        Ok(status)
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.reaped { let _ = self.stop_and_reap(); }
    }
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
