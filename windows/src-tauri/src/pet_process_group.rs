//! Mechanical ownership only: the caller still supplies its own authority,
//! durable intent and resource hold. Linux trusted helpers, not a sandbox.
use std::{io::{self,Read}, process::{Child, ExitStatus}, time::Duration};

pub(crate) struct OwnedChild { pub(crate) child: Child, reaped: bool }
impl OwnedChild {
    /// The child must have been spawned by us into its own process group.
    pub(crate) fn new(child: Child) -> Self { Self { child, reaped: false } }
    pub(crate) fn exited(&self) -> io::Result<bool> {
        let pid = self.child.id() as libc::pid_t;
        loop {
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let rc = unsafe { libc::waitid(libc::P_PID, pid as libc::id_t, &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT) };
            if rc == 0 { return Ok(unsafe { info.si_pid() } == pid); }
            let e = io::Error::last_os_error();
            if e.kind() != io::ErrorKind::Interrupted { return Err(e); }
        }
    }
    pub(crate) fn signal(&self, signal: i32) -> io::Result<()> {
        // Even a running child returns Ok(false). WNOWAIT establishes that it
        // is still ours and keeps the group id pinned until we reap the leader.
        self.exited()?;
        if unsafe { libc::kill(-(self.child.id() as libc::pid_t), signal) } == 0 { return Ok(()); }
        let e = io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::ESRCH) { Ok(()) } else { Err(e) }
    }
    /// None is cleanup still in progress; the caller retains its physical hold.
    pub(crate) fn finish_ready(&mut self) -> io::Result<Option<ExitStatus>> {
        self.signal(libc::SIGKILL)?;
        if !self.exited()? || group_has_live_members(self.child.id() as i32)? { return Ok(None); }
        let status = self.child.wait()?;
        self.reaped = true;
        Ok(Some(status))
    }
    pub(crate) fn stop_and_reap(&mut self) -> io::Result<ExitStatus> {
        loop {
            match self.finish_ready()? {
                Some(status) => return Ok(status),
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // A std Child alone would detach. On any unwind, preserve ownership
        // until cleanup; a failure never turns into permission to reuse a slot.
        while !self.reaped {
            match self.stop_and_reap() {
                Ok(_) => break,
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}

fn stat_group(bytes: &[u8]) -> Option<(u8, i32)> {
    let end = bytes.iter().rposition(|b| *b == b')')?;
    let text = std::str::from_utf8(bytes.get(end+1..)?).ok()?;
    let mut fields = text.split_whitespace();
    let state = fields.next()?.as_bytes().first().copied()?;
    fields.next()?; // ppid
    Some((state, fields.next()?.parse().ok()?))
}
fn group_has_live_members(group: i32) -> io::Result<bool> {
    // Only process metadata, no command lines, environments or file contents.
    // Keep the leader unreaped throughout this observation: no PGID reuse.
    let entries = std::fs::read_dir("/proc")?;
    for (count, entry) in entries.enumerate() {
        if count >= 131072 { return Err(io::Error::other("process observation limit")); }
        let entry = entry?;
        if !entry.file_name().as_encoded_bytes().iter().all(u8::is_ascii_digit) { continue; }
        let path = entry.path().join("stat");
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind()==io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        let mut bytes=Vec::new();file.take(8193).read_to_end(&mut bytes)?;
        if bytes.len() > 8192 { return Err(io::Error::other("process metadata limit")); }
        let (state, pgid) = stat_group(&bytes).ok_or_else(||io::Error::other("process metadata unavailable"))?;
        if pgid == group && !matches!(state, b'Z' | b'X' | b'x') { return Ok(true); }
    }
    Ok(false)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn helper_lifecycle_group_parser_handles_parentheses_without_reading_arguments() {
        assert_eq!(stat_group(b"123 (name with ) brackets) S 1 123 123 0"), Some((b'S',123)));
        assert_eq!(stat_group(b"123 (worker) Z 1 123 123 0"), Some((b'Z',123)));
        assert_eq!(stat_group(b"invalid"), None);
    }
}
