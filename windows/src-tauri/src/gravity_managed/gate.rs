//! Private Linux gate for explicitly reviewed, native-observed operations.
//! Peer provenance identifies a process, never the meaning of its payload.
//! No IPC registration, serialization, automatic choice or native permission
//! fallback lives here. The owner serializes calls with Stop: state -> gate.
use super::types::{valid_uuid, Reference, PET, PROVIDER};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_OPERATIONS: usize = 8;
const MAX_COMMAND: usize = 16 * 1024;
const MAX_CWD: usize = 4096;
const MAX_ANCESTRY: usize = 32;
const MAX_STAT: u64 = 8192;
const MAX_TTL: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Rejection {
    Invalid,
    Unavailable,
    Untrusted,
    NoOperation,
    Ambiguous,
    Busy,
    Limit,
    Expired,
    Stale,
    Consumed,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct ProcessState {
    pid: u32,
    parent: u32,
    start_ticks: u64,
}

/// Both descriptors pin one captured process. Neither is opened from a hook ID.
struct Process {
    pid: u32,
    start_ticks: u64,
    directory: File,
    pidfd: File,
}

fn owned_fd(fd: RawFd) -> Result<File, Rejection> {
    if fd < 0 {
        Err(Rejection::Unavailable)
    } else {
        // This function receives only a new descriptor returned by our syscall.
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

fn decimal(bytes: &[u8]) -> Result<u64, Rejection> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(Rejection::Untrusted);
    }
    bytes.iter().try_fold(0_u64, |n, byte| {
        n.checked_mul(10)
            .and_then(|n| n.checked_add((byte - b'0') as u64))
            .ok_or(Rejection::Untrusted)
    })
}

fn process_stat(bytes: &[u8], expected_pid: u32) -> Result<ProcessState, Rejection> {
    let first_space = bytes
        .iter()
        .position(|b| *b == b' ')
        .ok_or(Rejection::Untrusted)?;
    if decimal(&bytes[..first_space])? != expected_pid as u64
        || bytes.get(first_space + 1) != Some(&b'(')
    {
        return Err(Rejection::Untrusted);
    }
    // comm is arbitrary Linux bytes and can itself contain spaces and ')'.
    // Only the ASCII fields after its LAST closing delimiter are interpreted.
    let end = bytes
        .windows(2)
        .rposition(|part| part == b") ")
        .ok_or(Rejection::Untrusted)?;
    if end <= first_space + 1 {
        return Err(Rejection::Untrusted);
    }
    let fields: Vec<_> = bytes[end + 2..]
        .split(u8::is_ascii_whitespace)
        .filter(|part| !part.is_empty())
        .collect();
    if fields.len() <= 19
        || fields[0].len() != 1
        || !fields[0][0].is_ascii_alphabetic()
        || matches!(fields[0][0], b'Z' | b'X' | b'x')
    {
        return Err(Rejection::Untrusted);
    }
    let parent = u32::try_from(decimal(fields[1])?).map_err(|_| Rejection::Untrusted)?;
    let start_ticks = decimal(fields[19])?;
    if parent == 0 || start_ticks == 0 {
        return Err(Rejection::Untrusted);
    }
    Ok(ProcessState {
        pid: expected_pid,
        parent,
        start_ticks,
    })
}

fn stat_from_directory(directory: &File, pid: u32) -> Result<ProcessState, Rejection> {
    let metadata = directory.metadata().map_err(|_| Rejection::Unavailable)?;
    if !metadata.is_dir() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err(Rejection::Untrusted);
    }
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            b"stat\0".as_ptr().cast(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    let mut bytes = Vec::new();
    owned_fd(fd)?
        .take(MAX_STAT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Rejection::Unavailable)?;
    if bytes.len() as u64 > MAX_STAT {
        return Err(Rejection::Limit);
    }
    process_stat(&bytes, pid)
}

fn pidfd_live(pidfd: &File) -> Result<(), Rejection> {
    let mut fd = libc::pollfd {
        fd: pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // Any exit/hangup/error, including an interrupted or unsupported probe,
    // denies provenance. No signal is ever sent to an arbitrary process.
    if unsafe { libc::poll(&mut fd, 1, 0) } != 0 || fd.revents != 0 {
        Err(Rejection::Untrusted)
    } else {
        Ok(())
    }
}

impl Process {
    fn capture(pid: u32) -> Result<Self, Rejection> {
        if pid <= 1 || pid > i32::MAX as u32 {
            return Err(Rejection::Untrusted);
        }
        let path =
            std::ffi::CString::new(format!("/proc/{pid}")).map_err(|_| Rejection::Invalid)?;
        let directory = owned_fd(unsafe {
            libc::open(
                path.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        })?;
        let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatfs(directory.as_raw_fd(), &mut filesystem) } != 0
            || filesystem.f_type != libc::PROC_SUPER_MAGIC
        {
            return Err(Rejection::Untrusted);
        }
        let before = stat_from_directory(&directory, pid)?;
        let pidfd_raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as libc::pid_t, 0_u32) };
        if pidfd_raw < 0 {
            return Err(Rejection::Unavailable);
        }
        let pidfd = owned_fd(pidfd_raw as RawFd)?;
        pidfd_live(&pidfd)?;
        let after = stat_from_directory(&directory, pid)?;
        if before != after {
            return Err(Rejection::Untrusted);
        }
        pidfd_live(&pidfd)?;
        Ok(Self {
            pid,
            start_ticks: before.start_ticks,
            directory,
            pidfd,
        })
    }

    fn state(&self) -> Result<ProcessState, Rejection> {
        pidfd_live(&self.pidfd)?;
        let state = stat_from_directory(&self.directory, self.pid)?;
        if state.start_ticks != self.start_ticks {
            return Err(Rejection::Untrusted);
        }
        pidfd_live(&self.pidfd)?;
        Ok(state)
    }
}

pub(super) struct Leader {
    process: Arc<Process>,
}

impl Leader {
    pub(super) fn from_child(child: &Child) -> Result<Arc<Self>, Rejection> {
        let process = Arc::new(Process::capture(child.id())?);
        let state = process.state()?;
        if state.parent != std::process::id() {
            return Err(Rejection::Untrusted);
        }
        // WNOWAIT confirms live child custody without reaping the leader needed
        // by Transport cleanup. A dead, reused or externally reaped PID fails.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        if unsafe {
            libc::waitid(
                libc::P_PID,
                child.id(),
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        } != 0
            || unsafe { info.si_pid() } != 0
            || process.state()? != state
        {
            return Err(Rejection::Untrusted);
        }
        Ok(Arc::new(Self { process }))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Credentials {
    pid: u32,
    uid: u32,
    gid: u32,
}

fn peer(socket: &UnixStream) -> Result<Credentials, Rejection> {
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    } != 0
        || length as usize != std::mem::size_of::<libc::ucred>()
        || credentials.pid <= 1
        || credentials.uid != unsafe { libc::geteuid() }
    {
        return Err(Rejection::Untrusted);
    }
    Ok(Credentials {
        pid: credentials.pid as u32,
        uid: credentials.uid,
        gid: credentials.gid,
    })
}

/// One owned connection and its kernel-captured connecting process. Not Clone.
pub(crate) struct Relay {
    socket: UnixStream,
    credentials: Credentials,
    process: Arc<Process>,
}

impl Relay {
    pub(crate) fn capture(socket: UnixStream) -> Result<Self, Rejection> {
        let credentials = peer(&socket)?;
        let process = Arc::new(Process::capture(credentials.pid)?);
        socket
            .set_nonblocking(true)
            .map_err(|_| Rejection::Unavailable)?;
        if peer(&socket)? != credentials {
            return Err(Rejection::Untrusted);
        }
        process.state()?;
        Ok(Self {
            socket,
            credentials,
            process,
        })
    }

    fn descendant_of(&self, leader: &Leader) -> Result<(), Rejection> {
        if peer(&self.socket)? != self.credentials {
            return Err(Rejection::Untrusted);
        }
        let leader_before = leader.process.state()?;
        let mut current = self.process.clone();
        let mut chain: Vec<(Arc<Process>, ProcessState)> = Vec::new();
        for depth in 0..=MAX_ANCESTRY {
            let state = current.state()?;
            if chain.iter().any(|(_, old)| old.pid == state.pid) {
                return Err(Rejection::Untrusted);
            }
            chain.push((current.clone(), state));
            if state.pid == leader_before.pid {
                if state != leader_before {
                    return Err(Rejection::Untrusted);
                }
                // This is a bounded sequence of stable observations + pidfds,
                // not an atomic Linux ancestry syscall. Every captured link is
                // rechecked for live identity AND the same PPID after the walk.
                for (process, expected) in chain.iter().rev() {
                    if process.state()? != *expected {
                        return Err(Rejection::Untrusted);
                    }
                }
                if leader.process.state()? != leader_before
                    || peer(&self.socket)? != self.credentials
                {
                    return Err(Rejection::Untrusted);
                }
                return Ok(());
            }
            if depth == MAX_ANCESTRY || state.parent <= 1 {
                return Err(Rejection::Untrusted);
            }
            let parent = Arc::new(Process::capture(state.parent)?);
            if current.state()? != state {
                return Err(Rejection::Untrusted);
            }
            current = parent;
        }
        Err(Rejection::Untrusted)
    }
}

fn command_valid(command: &str) -> Result<(), Rejection> {
    if command.len() > MAX_COMMAND {
        return Err(Rejection::Limit);
    }
    if command.trim().is_empty() || command.contains('\0') {
        Err(Rejection::Invalid)
    } else {
        Ok(())
    }
}

fn cwd_valid(cwd: &PathBuf) -> Result<(), Rejection> {
    let bytes = cwd.as_os_str().as_bytes();
    if bytes.len() > MAX_CWD {
        return Err(Rejection::Limit);
    }
    if !cwd.is_absolute()
        || cwd.to_str().is_none()
        || bytes.iter().any(|b| *b < 32 || *b == 127)
        || bytes
            .split(|b| *b == b'/')
            .any(|part| part == b"." || part == b"..")
    {
        return Err(Rejection::Invalid);
    }
    Ok(())
}

#[derive(Clone)]
pub(super) struct NativeOperation {
    step: u64,
    command: String,
    cwd: PathBuf,
}

impl NativeOperation {
    /// Only the private native stdout parser/owner may mint this value. Missing
    /// native fields are never filled from current config or a hook request.
    pub(super) fn from_native(step: u64, command: String, cwd: PathBuf) -> Result<Self, Rejection> {
        command_valid(&command)?;
        cwd_valid(&cwd)?;
        Ok(Self { step, command, cwd })
    }

    fn same_spec(&self, other: &Self) -> bool {
        self.step == other.step
            && self.command == other.command
            && self.cwd.as_os_str() == other.cwd.as_os_str()
    }

    fn matches(&self, request: &HookRequest) -> bool {
        self.command == request.command && self.cwd.as_os_str() == request.cwd.as_os_str()
    }
}

/// Untrusted hook text: provenance is checked BEFORE its session ID is compared.
pub(crate) struct HookRequest {
    pub(crate) session_id: String,
    pub(crate) command: String,
    pub(crate) cwd: PathBuf,
}

#[derive(Clone)]
pub(crate) struct Card {
    pub(crate) reference: Reference,
    pub(crate) turn: u64,
    pub(crate) request_seq: u64,
    pub(crate) command: String,
    pub(crate) cwd: PathBuf,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Claim {
    Available,
    Pending(u64),
    Consumed,
}

struct Operation {
    native: NativeOperation,
    claim: Claim,
}

/// Not Clone, no public constructor, no mutable access to the reviewed card.
pub(crate) struct Pending {
    relay: Relay,
    card: Card,
    step: u64,
    revision: Arc<()>,
    leader: Arc<Leader>,
    deadline: Instant,
}

impl Pending {
    pub(crate) fn card(&self) -> &Card {
        &self.card
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HumanChoice {
    AllowOnce,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Response {
    Responded,
    DeliveryUnknown,
}

pub(super) struct Gate {
    leader: Option<Arc<Leader>>,
    reference: Option<Reference>,
    turn: Option<u64>,
    revision: Arc<()>,
    next_request: u64,
    operations: BTreeMap<u64, Operation>,
    // A scalar tombstone prevents a completed native step from being minted
    // again by a late ACTIVE. Already recorded older active steps still match.
    retired_through: Option<u64>,
}

impl Gate {
    pub(super) fn new() -> Self {
        Self {
            leader: None,
            reference: None,
            turn: None,
            revision: Arc::new(()),
            next_request: 0,
            operations: BTreeMap::new(),
            retired_through: None,
        }
    }

    pub(super) fn set_leader(&mut self, leader: Option<Arc<Leader>>) {
        self.revoke();
        self.reference = None;
        self.leader = leader;
    }

    pub(super) fn bind(&mut self, reference: Reference) -> Result<(), Rejection> {
        if reference.provider != PROVIDER
            || reference.pet_id != PET
            || reference.generation == 0
            || reference.handle.is_empty()
            || reference.handle.len() > 256
            || reference.handle.chars().any(char::is_control)
            || !valid_uuid(&reference.session_id)
        {
            return Err(Rejection::Invalid);
        }
        self.revoke();
        self.reference = Some(reference);
        Ok(())
    }

    pub(super) fn begin_turn(&mut self, turn: u64) -> Result<(), Rejection> {
        if turn == 0 || self.reference.is_none() {
            return Err(Rejection::Invalid);
        }
        self.revoke();
        self.turn = Some(turn);
        Ok(())
    }

    pub(super) fn record(&mut self, native: NativeOperation) -> Result<(), Rejection> {
        if self.turn.is_none() || self.reference.is_none() {
            return Err(Rejection::Stale);
        }
        if let Some(existing) = self.operations.get(&native.step) {
            if existing.native.same_spec(&native) {
                return Ok(());
            }
            // A conflicting native update invalidates the old card as well;
            // returning an error must not leave its previous spec authorizable.
            self.complete_step(native.step);
            return Err(Rejection::Invalid);
        }
        if self.retired_through.is_some_and(|step| native.step <= step) {
            return Err(Rejection::Stale);
        }
        if self.operations.len() >= MAX_OPERATIONS {
            return Err(Rejection::Limit);
        }
        self.operations.insert(
            native.step,
            Operation {
                native,
                claim: Claim::Available,
            },
        );
        Ok(())
    }

    pub(super) fn complete_step(&mut self, step: u64) {
        self.operations.remove(&step);
        self.retired_through = Some(self.retired_through.map_or(step, |old| old.max(step)));
    }

    pub(super) fn revoke(&mut self) {
        self.turn = None;
        self.operations.clear();
        self.retired_through = None;
        self.revision = Arc::new(());
        // next_request deliberately survives every revocation and new turn.
    }

    pub(super) fn matches_peer(&self, relay: &Relay) -> bool {
        self.leader
            .as_ref()
            .is_some_and(|leader| relay.descendant_of(leader).is_ok())
    }

    pub(super) fn request(
        &mut self,
        relay: Relay,
        request: HookRequest,
        ttl: Duration,
    ) -> Result<Pending, Rejection> {
        if ttl.is_zero() || ttl > MAX_TTL {
            return Err(Rejection::Invalid);
        }
        let deadline = Instant::now().checked_add(ttl).ok_or(Rejection::Limit)?;
        let leader = self.leader.clone().ok_or(Rejection::Unavailable)?;
        // The hook's UUID never selects an owner or replaces this provenance.
        relay.descendant_of(&leader)?;
        let reference = self.reference.as_ref().ok_or(Rejection::Stale)?;
        let turn = self.turn.ok_or(Rejection::Stale)?;
        if !valid_uuid(&request.session_id) || request.session_id != reference.session_id {
            return Err(Rejection::Untrusted);
        }
        command_valid(&request.command)?;
        cwd_valid(&request.cwd)?;
        let mut matching = self
            .operations
            .values()
            .filter(|op| op.native.matches(&request));
        let first = matching.next().ok_or(Rejection::NoOperation)?;
        if matching.next().is_some() {
            return Err(Rejection::Ambiguous);
        }
        if first.claim != Claim::Available {
            return Err(Rejection::Busy);
        }
        let step = first.native.step;
        if Instant::now() >= deadline {
            return Err(Rejection::Expired);
        }
        let sequence = self.next_request.checked_add(1).ok_or(Rejection::Limit)?;
        let card = Card {
            reference: reference.clone(),
            turn,
            request_seq: sequence,
            command: first.native.command.clone(),
            cwd: first.native.cwd.clone(),
        };
        // Claim and mint are one mutation under the owner's gate lock. A second
        // identical relay/request cannot produce another Pending for this step.
        self.next_request = sequence;
        self.operations
            .get_mut(&step)
            .ok_or(Rejection::Stale)?
            .claim = Claim::Pending(sequence);
        Ok(Pending {
            relay,
            card,
            step,
            revision: self.revision.clone(),
            leader,
            deadline,
        })
    }

    fn current(&self, pending: &Pending) -> Result<(), Rejection> {
        if !Arc::ptr_eq(&self.revision, &pending.revision)
            || self
                .leader
                .as_ref()
                .is_none_or(|leader| !Arc::ptr_eq(leader, &pending.leader))
            || self.reference.as_ref() != Some(&pending.card.reference)
            || self.turn != Some(pending.card.turn)
        {
            return Err(Rejection::Stale);
        }
        let op = self.operations.get(&pending.step).ok_or(Rejection::Stale)?;
        match op.claim {
            Claim::Pending(sequence) if sequence == pending.card.request_seq => {}
            Claim::Consumed => return Err(Rejection::Consumed),
            _ => return Err(Rejection::Stale),
        }
        if op.native.command != pending.card.command
            || op.native.cwd.as_os_str() != pending.card.cwd.as_os_str()
        {
            return Err(Rejection::Stale);
        }
        if self
            .operations
            .values()
            .filter(|other| {
                other.native.command == pending.card.command
                    && other.native.cwd.as_os_str() == pending.card.cwd.as_os_str()
            })
            .count()
            != 1
        {
            return Err(Rejection::Ambiguous);
        }
        if Instant::now() >= pending.deadline {
            return Err(Rejection::Expired);
        }
        Ok(())
    }

    pub(super) fn respond(
        &mut self,
        pending: Pending,
        choice: HumanChoice,
    ) -> Result<Response, Rejection> {
        self.current(&pending)?;
        pending.relay.descendant_of(&pending.leader)?;
        // Final deadline/revocation/owner/operation check comes AFTER ancestry,
        // directly before consuming the claim and the single nonblocking write.
        // Caller holds state -> gate throughout, so Stop cannot interleave here.
        self.current(&pending)?;
        self.operations
            .get_mut(&pending.step)
            .ok_or(Rejection::Stale)?
            .claim = Claim::Consumed;
        let bytes: &[u8] = match choice {
            HumanChoice::AllowOnce => b"allow\n",
            HumanChoice::Deny => b"deny\n",
        };
        let sent = unsafe {
            libc::send(
                pending.relay.socket.as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len(),
                libc::MSG_NOSIGNAL | libc::MSG_DONTWAIT,
            )
        };
        // No retry, even after EINTR/EAGAIN, zero bytes or partial delivery. The
        // operation remains consumed until native DONE/revoke removes it.
        Ok(if sent == bytes.len() as isize {
            Response::Responded
        } else {
            Response::DeliveryUnknown
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "019a734b-0000-7000-8000-000000000001";

    fn reference() -> Reference {
        Reference {
            handle: "synthetic-owner".into(),
            generation: 1,
            provider: PROVIDER.into(),
            pet_id: PET.into(),
            session_id: SESSION.into(),
        }
    }

    fn context() -> Gate {
        // Pure storage tests: without an actual captured Leader, this gate can
        // never match a peer or authorize a response. No fake provenance path.
        let mut gate = Gate::new();
        gate.bind(reference()).unwrap();
        gate.begin_turn(1).unwrap();
        gate
    }

    fn native(step: u64, command: &str) -> NativeOperation {
        NativeOperation::from_native(step, command.into(), PathBuf::from("/synthetic/workspace"))
            .unwrap()
    }

    fn stat(pid: u32, parent: u32, ticks: u64, state: u8, comm: &[u8]) -> Vec<u8> {
        let mut bytes = format!("{pid} (").into_bytes();
        bytes.extend_from_slice(comm);
        bytes.extend_from_slice(b") ");
        let mut fields = vec!["0".to_owned(); 20];
        fields[0] = char::from(state).to_string();
        fields[1] = parent.to_string();
        fields[19] = ticks.to_string();
        bytes.extend_from_slice(fields.join(" ").as_bytes());
        bytes.push(b'\n');
        bytes
    }

    #[test]
    fn proc_stat_accepts_arbitrary_comm_bytes_and_reads_exact_parent_and_start() {
        let bytes = stat(1234, 4321, 98765, b'S', b"odd ) name\xff\n) tail");
        let state = process_stat(&bytes, 1234).unwrap();
        assert_eq!(state.pid, 1234);
        assert_eq!(state.parent, 4321);
        assert_eq!(state.start_ticks, 98765);
    }

    #[test]
    fn proc_stat_rejects_dead_missing_or_contradictory_identity() {
        for state in [b'Z', b'X', b'x', b'?'] {
            assert!(process_stat(&stat(1234, 4321, 42, state, b"fixture"), 1234).is_err());
        }
        assert!(process_stat(&stat(1234, 4321, 42, b'S', b"fixture"), 1235).is_err());
        assert!(process_stat(&stat(1234, 0, 42, b'S', b"fixture"), 1234).is_err());
        assert!(process_stat(&stat(1234, 4321, 0, b'S', b"fixture"), 1234).is_err());
        for bytes in [
            b"".as_slice(),
            b"1234 (fixture) S 1",
            b"1234 fixture S 1",
            b"x (fixture) S 1",
        ] {
            assert!(process_stat(bytes, 1234).is_err());
        }
        assert_eq!(decimal(b"18446744073709551616"), Err(Rejection::Untrusted));
        assert_eq!(decimal(b"-1"), Err(Rejection::Untrusted));
    }

    #[test]
    fn native_operation_requires_explicit_bounded_command_and_absolute_cwd() {
        for command in ["", " \n\t", "synthetic\0command"] {
            assert!(matches!(
                NativeOperation::from_native(1, command.into(), "/synthetic".into()),
                Err(Rejection::Invalid)
            ));
        }
        for cwd in [
            "",
            "relative",
            "/synthetic/../other",
            "/synthetic/./other",
            "/synthetic\nother",
            "/synthetic\0other",
        ] {
            assert!(matches!(
                NativeOperation::from_native(1, "fixture".into(), cwd.into()),
                Err(Rejection::Invalid)
            ));
        }
        assert!(
            NativeOperation::from_native(0, "x".repeat(MAX_COMMAND), "/synthetic".into()).is_ok()
        );
        assert!(matches!(
            NativeOperation::from_native(0, "é".repeat(MAX_COMMAND / 2 + 1), "/synthetic".into()),
            Err(Rejection::Limit)
        ));
        assert!(matches!(
            NativeOperation::from_native(
                0,
                "fixture".into(),
                PathBuf::from(format!("/{}", "x".repeat(MAX_CWD)))
            ),
            Err(Rejection::Limit)
        ));
    }

    #[test]
    fn matching_uses_raw_cwd_and_command_bytes_without_path_or_text_normalization() {
        let operation = NativeOperation::from_native(
            3,
            "fixture --exact".into(),
            "/synthetic//workspace".into(),
        )
        .unwrap();
        let mut request = HookRequest {
            session_id: SESSION.into(),
            command: "fixture --exact".into(),
            cwd: "/synthetic//workspace".into(),
        };
        assert!(operation.matches(&request));
        request.cwd = "/synthetic/workspace".into();
        assert!(!operation.matches(&request));
        request.cwd = "/synthetic//workspace".into();
        request.command.push(' ');
        assert!(!operation.matches(&request));
    }

    #[test]
    fn repeated_active_is_idempotent_and_never_releases_a_claim() {
        let mut gate = context();
        gate.record(native(7, "fixture one")).unwrap();
        gate.operations.get_mut(&7).unwrap().claim = Claim::Pending(23);
        gate.next_request = 23;
        gate.record(native(7, "fixture one")).unwrap();
        assert!(matches!(gate.operations[&7].claim, Claim::Pending(23)));
        gate.operations.get_mut(&7).unwrap().claim = Claim::Consumed;
        gate.record(native(7, "fixture one")).unwrap();
        assert!(matches!(gate.operations[&7].claim, Claim::Consumed));
        assert_eq!(gate.next_request, 23);
    }

    #[test]
    fn conflicting_native_update_revokes_the_old_operation_instead_of_replacing_it() {
        let mut gate = context();
        gate.record(native(7, "fixture one")).unwrap();
        gate.operations.get_mut(&7).unwrap().claim = Claim::Pending(23);
        assert_eq!(
            gate.record(native(7, "fixture replacement")),
            Err(Rejection::Invalid)
        );
        assert!(!gate.operations.contains_key(&7));
        assert_eq!(gate.record(native(7, "fixture one")), Err(Rejection::Stale));
        gate.begin_turn(2).unwrap();
        gate.record(native(7, "fixture next turn")).unwrap();
    }

    #[test]
    fn capacity_and_done_are_bounded_without_reopening_completed_steps() {
        let mut gate = context();
        for step in 0..MAX_OPERATIONS as u64 {
            gate.record(native(step, "fixture")).unwrap();
        }
        assert_eq!(gate.record(native(8, "fixture")), Err(Rejection::Limit));
        gate.complete_step(2);
        assert!(!gate.operations.contains_key(&2));
        assert_eq!(gate.record(native(2, "fixture")), Err(Rejection::Stale));
        // Older operations already observed ACTIVE remain idempotent even if a
        // concurrent later operation completed first. New old steps fail closed.
        gate.record(native(0, "fixture")).unwrap();
        gate.record(native(8, "fixture")).unwrap();
        assert_eq!(gate.operations.len(), MAX_OPERATIONS);
    }

    #[test]
    fn revocation_and_turn_changes_invalidate_tokens_but_do_not_reset_sequences() {
        let mut gate = context();
        gate.next_request = 41;
        gate.record(native(1, "fixture")).unwrap();
        let before = gate.revision.clone();
        gate.begin_turn(2).unwrap();
        assert!(!Arc::ptr_eq(&before, &gate.revision));
        assert!(gate.operations.is_empty());
        assert_eq!(gate.next_request, 41);
        assert_eq!(gate.turn, Some(2));
        gate.record(native(1, "fixture")).unwrap();
        gate.revoke();
        assert!(gate.reference.is_some());
        assert!(gate.turn.is_none() && gate.operations.is_empty());
        assert_eq!(gate.next_request, 41);
        gate.set_leader(None);
        assert!(gate.reference.is_none());
        assert_eq!(gate.next_request, 41);
    }

    #[test]
    fn invalid_bind_and_turn_do_not_mutate_an_existing_context() {
        let mut gate = context();
        gate.record(native(1, "fixture")).unwrap();
        let before = gate.revision.clone();
        let mut wrong = reference();
        wrong.provider = "codex".into();
        assert_eq!(gate.bind(wrong), Err(Rejection::Invalid));
        assert_eq!(gate.begin_turn(0), Err(Rejection::Invalid));
        assert!(Arc::ptr_eq(&before, &gate.revision));
        assert_eq!(gate.turn, Some(1));
        assert_eq!(gate.reference, Some(reference()));
        assert!(gate.operations.contains_key(&1));
    }
}
