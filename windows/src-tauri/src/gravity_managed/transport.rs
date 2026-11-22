//! One Linux child, one writer and bounded nonblocking pipes. Never attach to a
//! PID from disk. Keep the leader unreaped until its owned group is cleaned.
use super::gate::Leader;
use super::types::{Config, Diagnostics, Fault, MAX_LINE};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_STDOUT: usize = 32 * 1024 * 1024;
const MAX_STDERR: usize = 1024 * 1024;
const MAX_TURN_STDOUT: usize = 4 * 1024 * 1024;
const MAX_TURN_STDERR: usize = 64 * 1024;

pub(super) struct Poll {
    pub(super) lines: Vec<Vec<u8>>,
    pub(super) closed: bool,
}

pub(super) struct Transport {
    child: Child,
    input: Option<ChildStdin>,
    output: Option<ChildStdout>,
    errors: Option<ChildStderr>,
    line: Vec<u8>,
    stderr_tail: Vec<u8>,
    total_out: usize,
    total_err: usize,
    turn_out: usize,
    turn_err: usize,
    diagnostics: Diagnostics,
    custody: bool,
    reaped: bool,
    leader: Option<Arc<Leader>>,
}

fn nonblocking(fd: RawFd) -> Result<(), Fault> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        Err(Fault::Io)
    } else {
        Ok(())
    }
}

impl Transport {
    /// Err means no Child was acquired. Any post-spawn setup fault returns the
    /// owned transport as well, so its worker retains custody until cleanup.
    pub(super) fn start(config: &Config) -> Result<(Self, Option<Fault>), Fault> {
        config.validate()?;
        let directory = File::open(&config.cwd).map_err(|_| Fault::InvalidConfig)?;
        let directory_fd = directory.as_raw_fd();
        let mut command = Command::new(&config.executable);
        command
            .args([
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--model",
                &config.model,
            ])
            .current_dir(&config.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // The existing Chat gate cannot authorize this new owner yet.
            // Native configuration/account/policies otherwise remain intact.
            .env("COUCOU_ASSISTANT_GATE", "0")
            .process_group(0);
        unsafe {
            command.pre_exec(move || {
                if libc::fchdir(directory_fd) < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let mut child = command.spawn().map_err(|_| Fault::Spawn)?;
        let transport = Self {
            // Failure disables gate provenance, never falls back to a bare PID.
            leader: Leader::from_child(&child).ok(),
            input: child.stdin.take(),
            output: child.stdout.take(),
            errors: child.stderr.take(),
            child,
            line: Vec::new(),
            stderr_tail: Vec::new(),
            total_out: 0,
            total_err: 0,
            turn_out: 0,
            turn_err: 0,
            diagnostics: Diagnostics::default(),
            custody: true,
            reaped: false,
        };
        let descriptors = [
            transport.input.as_ref().map(AsRawFd::as_raw_fd),
            transport.output.as_ref().map(AsRawFd::as_raw_fd),
            transport.errors.as_ref().map(AsRawFd::as_raw_fd),
        ];
        for fd in descriptors {
            if fd.ok_or(Fault::Io).and_then(nonblocking).is_err() {
                return Ok((transport, Some(Fault::Io)));
            }
        }
        Ok((transport, None))
    }

    pub(super) fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }

    pub(super) fn leader(&self) -> Option<Arc<Leader>> {
        self.leader.clone()
    }

    pub(super) fn begin_turn(&mut self) {
        self.turn_out = 0;
        self.turn_err = 0;
    }

    /// At most one bounded write per worker tick, so Stop cannot be held behind
    /// a native process that has stopped reading its stdin.
    pub(super) fn write(&mut self, bytes: &[u8]) -> Result<usize, Fault> {
        match self
            .input
            .as_mut()
            .ok_or(Fault::Closed)?
            .write(&bytes[..bytes.len().min(8192)])
        {
            Ok(0) => Err(Fault::Closed),
            Ok(n) => Ok(n),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                Ok(0)
            }
            Err(_) => Err(Fault::Io),
        }
    }

    fn exited(&mut self) -> Result<bool, Fault> {
        if !self.custody {
            return Err(Fault::CleanupUnconfirmed);
        }
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            libc::waitid(
                libc::P_PID,
                self.child.id(),
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if rc < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD) {
                // Someone else reaped it: its PID/group is no longer safe to signal.
                self.custody = false;
            }
            return Err(Fault::Io);
        }
        Ok(unsafe { info.si_pid() } != 0)
    }

    fn classify_stderr(&mut self, bytes: &[u8]) {
        self.diagnostics.seen = true;
        self.diagnostics.bytes = self.total_err as u64;
        // A tiny rolling window handles split notices. It is never returned,
        // formatted into an error, logged or written to a journal.
        for part in bytes.chunks(128) {
            self.stderr_tail
                .extend(part.iter().map(u8::to_ascii_lowercase));
            let text = String::from_utf8_lossy(&self.stderr_tail);
            self.diagnostics.permission_limited |= text.contains("soft-den")
                || text.contains("permission denied")
                || text.contains("approval unavailable")
                || (text.contains("permission")
                    && (text.contains("denied")
                        || text.contains("unavailable")
                        || text.contains("non-interactive")))
                || (text.contains("approval")
                    && (text.contains("denied")
                        || text.contains("headless")
                        || text.contains("non-interactive")));
            self.diagnostics.authentication_required |= text.contains("authentication required")
                || text.contains("login required")
                || text.contains("sign in");
            if self.stderr_tail.len() > 256 {
                self.stderr_tail.drain(..self.stderr_tail.len() - 256);
            }
        }
    }

    pub(super) fn poll(&mut self) -> Result<Poll, Fault> {
        let mut buffer = [0_u8; 8192];
        // Fair, bounded reads: a noisy stdout cannot starve stderr or Stop.
        if let Some(errors) = self.errors.as_mut() {
            match errors.read(&mut buffer) {
                Ok(0) => {
                    self.errors.take();
                }
                Ok(n) => {
                    self.total_err += n;
                    self.turn_err += n;
                    self.classify_stderr(&buffer[..n]);
                    if self.total_err > MAX_STDERR || self.turn_err > MAX_TURN_STDERR {
                        return Err(Fault::Limit);
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(Fault::Io),
            }
        }
        let mut lines = Vec::new();
        let mut received_output = false;
        if let Some(output) = self.output.as_mut() {
            match output.read(&mut buffer) {
                Ok(0) => {
                    self.output.take();
                }
                Ok(n) => {
                    received_output = true;
                    self.total_out += n;
                    self.turn_out += n;
                    if self.total_out > MAX_STDOUT || self.turn_out > MAX_TURN_STDOUT {
                        return Err(Fault::Limit);
                    }
                    for byte in &buffer[..n] {
                        if *byte == b'\n' {
                            if self.line.is_empty() {
                                return Err(Fault::Protocol);
                            }
                            lines.push(std::mem::take(&mut self.line));
                        } else {
                            if self.line.len() >= MAX_LINE {
                                return Err(Fault::Limit);
                            }
                            self.line.push(*byte);
                        }
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(Fault::Io),
            }
        }
        // An exited leader may still have complete buffered results to drain.
        // If a descendant holds the pipe idle after leader exit, do not wait
        // forever for EOF: we will collect that owned group instead.
        let closed = self.output.is_none() || (self.exited()? && !received_output);
        if closed && !self.line.is_empty() {
            return Err(Fault::Protocol);
        }
        Ok(Poll { lines, closed })
    }

    /// Retain custody and report false until both the leader exited and no live
    /// member remains in its process group. Zombies do not execute or hold pipes.
    pub(super) fn cleanup(&mut self) -> bool {
        if self.reaped {
            return true;
        }
        self.input.take();
        if self.exited().is_err() || !self.custody {
            return false;
        }
        let group = -(self.child.id() as i32);
        unsafe {
            libc::kill(group, libc::SIGTERM);
        }
        let grace = Instant::now() + Duration::from_millis(100);
        while Instant::now() < grace {
            std::thread::sleep(Duration::from_millis(5));
        }
        unsafe {
            libc::kill(group, libc::SIGKILL);
        }
        let deadline = Instant::now() + Duration::from_millis(600);
        loop {
            if self.exited().unwrap_or(false) && no_live_group(self.child.id()) {
                if self.child.wait().is_ok() {
                    self.reaped = true;
                    self.output.take();
                    self.errors.take();
                    self.line.clear();
                    self.stderr_tail.clear();
                    return true;
                }
                self.custody = false;
                return false;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn no_live_group(group: u32) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for (index, entry) in entries.enumerate() {
        if index > 65536 {
            return false;
        }
        let Ok(entry) = entry else {
            return false;
        };
        if !entry
            .file_name()
            .as_encoded_bytes()
            .iter()
            .all(u8::is_ascii_digit)
        {
            continue;
        }
        let stat = match std::fs::read(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return false,
        };
        // Linux comm may contain non-UTF8 bytes and ')' characters. Decode
        // only the ASCII fields after the last closing comm delimiter.
        let Some(end) = stat.windows(2).rposition(|part| part == b") ") else {
            return false;
        };
        let Ok(fields) = std::str::from_utf8(&stat[end + 2..]) else {
            return false;
        };
        let mut fields = fields.split_whitespace();
        let state = fields.next();
        let _ppid = fields.next();
        let pgrp = fields.next().and_then(|n| n.parse::<u32>().ok());
        if pgrp == Some(group) && !matches!(state, Some("Z" | "X")) {
            return false;
        }
    }
    true
}

impl Drop for Transport {
    fn drop(&mut self) {
        if !self.reaped && self.custody {
            // Normal worker flow retains this object on unconfirmed cleanup.
            // This is the unwind fallback; it never publishes a ready state.
            if !self.cleanup() && self.custody {
                unsafe {
                    libc::kill(-(self.child.id() as i32), libc::SIGKILL);
                }
                let _ = self.child.wait();
            }
        }
    }
}
