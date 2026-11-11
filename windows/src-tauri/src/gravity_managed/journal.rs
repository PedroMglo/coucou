//! Isolated historical receipt store. A durable Ack is NOT an execution permit.
//! No process, runtime reference, grant, prompt, command or live session is
//! restored here. The native worker is deliberately not integrated in this stage.
use super::types::valid_uuid;
use serde::de::{value::MapAccessDeserializer, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

const DATA: &[u8] = b"receipts-v1.json\0";
const MAX_RECORDS: usize = 128;
const MAX_BYTES: u64 = 2 * 1024 * 1024;
const MAX_CWD: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Invalid,
    Limit,
    Conflict,
    Io,
    DurabilityUnknown,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Delivery {
    NotSent,
    Possible,
    Observed,
    Completed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Outcome {
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
    Waiting,
    Lost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum State {
    Starting,
    Active,
    Ready,
    Stopping,
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NativeRecord {
    pub(crate) session_id: String,
    pub(crate) cwd: String,
    pub(crate) model: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Record {
    /// Historical key only. Never passed to Manager as a handle/capability.
    pub(crate) record_id: String,
    pub(crate) revision: u64,
    pub(crate) created_at: Option<u64>,
    pub(crate) updated_at: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_native")]
    pub(crate) native: Option<NativeRecord>,
    pub(crate) local_turn: u64,
    pub(crate) delivery: Delivery,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) last_known_state: State,
    pub(crate) last_known_cleanup_confirmed: bool,
    pub(crate) error_code: Option<String>,
}

fn error_code_allowed(code: &str) -> bool {
    matches!(
        code,
        "invalidConfiguration"
            | "invalidQuery"
            | "wrongReference"
            | "busy"
            | "notReady"
            | "capacity"
            | "spawnFailed"
            | "io"
            | "protocolError"
            | "identityMismatch"
            | "limit"
            | "deadline"
            | "nativeClosed"
            | "stopped"
            | "cleanupUnconfirmed"
            | "journalUnavailable"
            | "journalDurabilityUnknown"
    )
}

impl Record {
    fn validate(&self) -> Result<(), Error> {
        if self.record_id.is_empty()
            || self.record_id.len() > 128
            || !self.record_id.is_ascii()
            || self.record_id.trim().is_empty()
            || self
                .record_id
                .bytes()
                .any(|b| b.is_ascii_control() || matches!(b, b'/' | b'\\'))
            || self.revision == 0
            || self.local_turn == 0
            || self
                .error_code
                .as_deref()
                .is_some_and(|code| !error_code_allowed(code))
        {
            return Err(Error::Invalid);
        }
        if let Some(native) = &self.native {
            if !valid_uuid(&native.session_id)
                || !Path::new(&native.cwd).is_absolute()
                || native.cwd.len() > MAX_CWD
                || native.cwd.chars().any(char::is_control)
                || native.model.is_empty()
                || native.model.len() > 256
                || !native
                    .model
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
            {
                return Err(Error::Invalid);
            }
        }
        if self.last_known_cleanup_confirmed && self.last_known_state != State::Lost {
            return Err(Error::Invalid);
        }
        if self.last_known_state == State::Ready
            && (self.delivery != Delivery::Completed
                || self.last_known_cleanup_confirmed
                || self.outcome == Some(Outcome::Waiting))
        {
            return Err(Error::Invalid);
        }
        if self.last_known_state == State::Active && self.native.is_none() {
            return Err(Error::Invalid);
        }
        if self.last_known_state == State::Starting && self.native.is_some() {
            return Err(Error::Invalid);
        }
        if matches!(self.delivery, Delivery::Observed | Delivery::Completed)
            && self.native.is_none()
        {
            return Err(Error::Invalid);
        }
        if self.delivery == Delivery::Completed {
            if self.outcome.is_none() || self.outcome == Some(Outcome::Lost) {
                return Err(Error::Invalid);
            }
        } else if self.outcome.is_some()
            && !(self.outcome == Some(Outcome::Lost)
                && matches!(self.last_known_state, State::Stopping | State::Lost))
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum Provider {
    #[serde(rename = "antigravity")]
    Antigravity,
}
#[derive(Serialize, Deserialize)]
enum Pet {
    #[serde(rename = "agent_antigravity")]
    Antigravity,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    version: u32,
    provider: Provider,
    pet_id: Pet,
    #[serde(deserialize_with = "deserialize_records")]
    records: Vec<Record>,
}

// Struct derive can also deserialize positional sequences. Persisted objects
// must be maps, while their own derived map visitors still reject duplicate
// fields. Do not round-trip through Value, which would discard duplicate keys.
struct Object<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = Object<T>;

            fn expecting(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                output.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, access: A) -> Result<Self::Value, A::Error> {
                T::deserialize(MapAccessDeserializer::new(access)).map(Object)
            }
        }

        input.deserialize_map(ObjectVisitor::<T>(std::marker::PhantomData))
    }
}

fn deserialize_records<'de, D: Deserializer<'de>>(input: D) -> Result<Vec<Record>, D::Error> {
    Vec::<Object<Record>>::deserialize(input)
        .map(|records| records.into_iter().map(|Object(record)| record).collect())
}

fn deserialize_native<'de, D: Deserializer<'de>>(
    input: D,
) -> Result<Option<NativeRecord>, D::Error> {
    Option::<Object<NativeRecord>>::deserialize(input)
        .map(|native| native.map(|Object(native)| native))
}

/// Recovery has no runtime constructors or handles and cannot become a live
/// Manager reference. These booleans/outcomes describe history, not authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RecoveryState {
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecoveryRecord {
    pub(crate) record_id: String,
    pub(crate) revision: u64,
    pub(crate) created_at: Option<u64>,
    pub(crate) updated_at: Option<u64>,
    pub(crate) native: Option<NativeRecord>,
    pub(crate) local_turn: u64,
    pub(crate) state: RecoveryState,
    pub(crate) delivery: Delivery,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) interrupted: bool,
    pub(crate) last_known_state: State,
    pub(crate) last_known_cleanup_confirmed: bool,
    pub(crate) error_code: Option<String>,
}

impl From<&Record> for RecoveryRecord {
    fn from(record: &Record) -> Self {
        Self {
            record_id: record.record_id.clone(),
            revision: record.revision,
            created_at: record.created_at,
            updated_at: record.updated_at,
            native: record.native.clone(),
            local_turn: record.local_turn,
            state: RecoveryState::Lost,
            delivery: match record.delivery {
                Delivery::NotSent => Delivery::NotSent,
                Delivery::Completed => Delivery::Completed,
                Delivery::Possible | Delivery::Observed | Delivery::Unknown => Delivery::Unknown,
            },
            outcome: record.outcome,
            interrupted: matches!(
                record.last_known_state,
                State::Starting | State::Active | State::Stopping
            ),
            last_known_state: record.last_known_state,
            last_known_cleanup_confirmed: record.last_known_cleanup_confirmed,
            error_code: record.error_code.clone(),
        }
    }
}

/// Only constructed after file sync, rename and parent sync all succeeded.
/// Not Clone/Serialize/Deserialize and deliberately not an execution permit.
#[derive(Debug)]
pub(crate) struct Ack {
    record_id: String,
    revision: u64,
    local_turn: u64,
}

impl Ack {
    pub(crate) fn record_id(&self) -> &str {
        &self.record_id
    }
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }
    pub(crate) fn local_turn(&self) -> u64 {
        self.local_turn
    }
    pub(crate) fn matches(&self, record_id: &str, revision: u64, local_turn: u64) -> bool {
        self.record_id == record_id && self.revision == revision && self.local_turn == local_turn
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fingerprint {
    device: u64,
    inode: u64,
    size: u64,
    mtime: i64,
    mtime_ns: i64,
    ctime: i64,
    ctime_ns: i64,
}

impl Fingerprint {
    /// Rename can legitimately change ctime; content metadata must still match
    /// the exact inode written and synced before replacement. This detects an
    /// observed edit, without claiming atomicity against an outside writer.
    fn same_written_file(self, written: Self) -> bool {
        self.device == written.device
            && self.inode == written.inode
            && self.size == written.size
            && self.mtime == written.mtime
            && self.mtime_ns == written.mtime_ns
    }
}

fn owned_fd(fd: RawFd) -> Result<File, Error> {
    if fd < 0 {
        Err(Error::Io)
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

fn fd_stat(file: &File) -> Result<libc::stat, Error> {
    let mut stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(file.as_raw_fd(), &mut stat) } != 0 {
        Err(Error::Io)
    } else {
        Ok(stat)
    }
}

fn path_stat(directory: &File, name: &[u8]) -> Result<Option<libc::stat>, Error> {
    let mut stat = unsafe { std::mem::zeroed() };
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr().cast(),
            &mut stat,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } == 0
    {
        Ok(Some(stat))
    } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
        Ok(None)
    } else {
        Err(Error::Io)
    }
}

fn data_fingerprint(stat: &libc::stat) -> Result<Fingerprint, Error> {
    if stat.st_mode & libc::S_IFMT != libc::S_IFREG
        || stat.st_mode & 0o7777 != 0o600
        || stat.st_uid != unsafe { libc::geteuid() }
        || stat.st_nlink != 1
        || stat.st_size < 0
    {
        return Err(Error::Invalid);
    }
    if stat.st_size as u64 > MAX_BYTES {
        return Err(Error::Limit);
    }
    Ok(Fingerprint {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
        size: stat.st_size as u64,
        mtime: stat.st_mtime,
        mtime_ns: stat.st_mtime_nsec,
        ctime: stat.st_ctime,
        ctime_ns: stat.st_ctime_nsec,
    })
}

fn at_fingerprint(directory: &File) -> Result<Option<Fingerprint>, Error> {
    path_stat(directory, DATA)?
        .map(|stat| data_fingerprint(&stat))
        .transpose()
}

fn real_directory(file: &File) -> Result<(u64, u64), Error> {
    let stat = fd_stat(file)?;
    if stat.st_mode & libc::S_IFMT != libc::S_IFDIR {
        return Err(Error::Invalid);
    }
    let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatfs(file.as_raw_fd(), &mut filesystem) } != 0 {
        return Err(Error::Io);
    }
    if filesystem.f_type == libc::PROC_SUPER_MAGIC {
        return Err(Error::Invalid);
    }
    Ok((stat.st_dev as u64, stat.st_ino as u64))
}

/// Walk every component without following a symlink or entering procfs. The
/// reviewed directory must already exist; neither open nor recovery mkdirs.
fn open_directory(path: &Path) -> Result<File, Error> {
    let bytes = path.as_os_str().as_bytes();
    if !path.is_absolute()
        || bytes.len() > 4096
        || bytes.contains(&0)
        || bytes
            .split(|b| *b == b'/')
            .any(|part| part == b"." || part == b"..")
    {
        return Err(Error::Invalid);
    }
    let mut directory = owned_fd(unsafe {
        libc::open(
            b"/\0".as_ptr().cast(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    })?;
    real_directory(&directory)?;
    for part in bytes.split(|b| *b == b'/').filter(|part| !part.is_empty()) {
        let name = CString::new(part).map_err(|_| Error::Invalid)?;
        let next = owned_fd(unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        })?;
        real_directory(&next)?;
        directory = next;
    }
    let stat = fd_stat(&directory)?;
    if stat.st_uid != unsafe { libc::geteuid() } || stat.st_mode & 0o7777 != 0o700 {
        return Err(Error::Invalid);
    }
    Ok(directory)
}

struct Directory {
    file: File,
    path: std::path::PathBuf,
    identity: (u64, u64),
}

impl Directory {
    fn open(path: &Path) -> Result<Self, Error> {
        let file = open_directory(path)?;
        let identity = real_directory(&file)?;
        Ok(Self {
            file,
            path: path.into(),
            identity,
        })
    }

    fn verify(&self) -> Result<(), Error> {
        let current = open_directory(&self.path).map_err(|_| Error::Conflict)?;
        if real_directory(&current)? != self.identity
            || real_directory(&self.file)? != self.identity
        {
            return Err(Error::Conflict);
        }
        Ok(())
    }
}

fn load(directory: &Directory) -> Result<(BTreeMap<String, Record>, Option<Fingerprint>), Error> {
    directory.verify()?;
    let Some(before) = at_fingerprint(&directory.file)? else {
        return Ok((BTreeMap::new(), None));
    };
    let fd = unsafe {
        libc::openat(
            directory.file.as_raw_fd(),
            DATA.as_ptr().cast(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        )
    };
    let mut file = owned_fd(fd)?;
    if data_fingerprint(&fd_stat(&file)?)? != before {
        return Err(Error::Conflict);
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Error::Limit);
    }
    if data_fingerprint(&fd_stat(&file)?)? != before
        || at_fingerprint(&directory.file)? != Some(before)
    {
        return Err(Error::Conflict);
    }
    let Object(envelope) =
        serde_json::from_slice::<Object<Envelope>>(&bytes).map_err(|_| Error::Invalid)?;
    if envelope.version != 1 {
        return Err(Error::Invalid);
    }
    if envelope.records.len() > MAX_RECORDS {
        return Err(Error::Limit);
    }
    let mut records = BTreeMap::new();
    for record in envelope.records {
        record.validate()?;
        if records.insert(record.record_id.clone(), record).is_some() {
            return Err(Error::Invalid);
        }
    }
    directory.verify()?;
    Ok((records, Some(before)))
}

struct Temporary<'a> {
    directory: &'a File,
    file: File,
    name: CString,
    identity: (u64, u64),
    renamed: bool,
}

impl<'a> Temporary<'a> {
    fn create(directory: &'a File) -> Result<Self, Error> {
        let mut random = [0_u8; 16];
        File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(&mut random))
            .map_err(|_| Error::Io)?;
        let nonce = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let name = CString::new(format!(".receipts-v1.{nonce}.tmp")).map_err(|_| Error::Io)?;
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0o600,
            )
        };
        let file = owned_fd(fd)?;
        let stat = fd_stat(&file)?;
        let temporary = Self {
            directory,
            file,
            name,
            identity: (stat.st_dev as u64, stat.st_ino as u64),
            renamed: false,
        };
        // Only our just-created inode may be chmodded, never an existing file.
        if unsafe { libc::fchmod(temporary.file.as_raw_fd(), 0o600) } != 0 {
            return Err(Error::Io);
        }
        data_fingerprint(&fd_stat(&temporary.file)?)?;
        Ok(temporary)
    }
}

impl Drop for Temporary<'_> {
    fn drop(&mut self) {
        if self.renamed {
            return;
        }
        if let Ok(Some(stat)) = path_stat(self.directory, self.name.as_bytes_with_nul()) {
            if (stat.st_dev as u64, stat.st_ino as u64) == self.identity {
                let _ =
                    unsafe { libc::unlinkat(self.directory.as_raw_fd(), self.name.as_ptr(), 0) };
            }
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Stage {
    OpenTemp,
    Write,
    SyncFile,
    Rename,
    SyncDirectory,
}

/// Exclusive synchronous store. No Clone: a second open on the same directory
/// conflicts with its advisory lock. Readers do not create files or take locks.
pub(crate) struct Journal {
    directory: Directory,
    records: BTreeMap<String, Record>,
    saved: Option<Fingerprint>,
    recovered_at_open: Vec<RecoveryRecord>,
    poisoned: bool,
    #[cfg(test)]
    failure: Option<Stage>,
    #[cfg(test)]
    before_rename: Option<Box<dyn FnOnce(&Path) + Send>>,
    #[cfg(test)]
    after_rename: Option<Box<dyn FnOnce(&Path) + Send>>,
}

impl std::fmt::Debug for Journal {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("Journal")
            .field("records", &self.records.len())
            .field("unavailable", &self.poisoned)
            .finish_non_exhaustive()
    }
}

impl Journal {
    pub(crate) fn open(directory: &Path) -> Result<Self, Error> {
        let directory = Directory::open(directory)?;
        if unsafe { libc::flock(directory.file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let code = std::io::Error::last_os_error().raw_os_error();
            return Err(
                if code == Some(libc::EWOULDBLOCK) || code == Some(libc::EAGAIN) {
                    Error::Conflict
                } else {
                    Error::Io
                },
            );
        }
        let (records, saved) = load(&directory)?;
        let recovered_at_open = records.values().map(RecoveryRecord::from).collect();
        Ok(Self {
            directory,
            records,
            saved,
            recovered_at_open,
            poisoned: false,
            #[cfg(test)]
            failure: None,
            #[cfg(test)]
            before_rename: None,
            #[cfg(test)]
            after_rename: None,
        })
    }

    /// Immutable historical snapshot captured by open. Saving does not append
    /// runtime authority or rewrite this view; read_recovery reads current disk.
    pub(crate) fn recovery(&self) -> Vec<RecoveryRecord> {
        self.recovered_at_open.clone()
    }

    /// Fresh read-only projection, including while a writer holds the lock.
    /// Never creates a directory, data/lock file or runtime process.
    pub(crate) fn read_recovery(directory: &Path) -> Result<Vec<RecoveryRecord>, Error> {
        let directory = Directory::open(directory)?;
        let (records, _) = load(&directory)?;
        Ok(records.values().map(RecoveryRecord::from).collect())
    }

    #[cfg(test)]
    pub(crate) fn fail_next(&mut self, stage: Stage) {
        self.failure = Some(stage);
    }

    #[cfg(test)]
    pub(crate) fn before_rename_for_test<F>(&mut self, hook: F)
    where
        F: FnOnce(&Path) + Send + 'static,
    {
        self.before_rename = Some(Box::new(hook));
    }

    #[cfg(test)]
    pub(crate) fn after_rename_for_test<F>(&mut self, hook: F)
    where
        F: FnOnce(&Path) + Send + 'static,
    {
        self.after_rename = Some(Box::new(hook));
    }

    fn unchanged(&self) -> Result<(), Error> {
        self.directory.verify()?;
        let current = at_fingerprint(&self.directory.file).map_err(|_| Error::Conflict)?;
        if current != self.saved {
            return Err(Error::Conflict);
        }
        Ok(())
    }

    pub(crate) fn save(
        &mut self,
        record: Record,
        expected_revision: Option<u64>,
    ) -> Result<Ack, Error> {
        if self.poisoned {
            return Err(Error::Unavailable);
        }
        record.validate()?;
        match (self.records.get(&record.record_id), expected_revision) {
            (None, None) if record.revision == 1 => {}
            (Some(old), Some(expected))
                if old.revision == expected && expected.checked_add(1) == Some(record.revision) => {
            }
            _ => return Err(Error::Conflict),
        }
        if !self.records.contains_key(&record.record_id) && self.records.len() >= MAX_RECORDS {
            return Err(Error::Limit);
        }
        self.unchanged()?;
        let mut next = self.records.clone();
        next.insert(record.record_id.clone(), record.clone());
        let envelope = Envelope {
            version: 1,
            provider: Provider::Antigravity,
            pet_id: Pet::Antigravity,
            records: next.values().cloned().collect(),
        };
        let bytes = serde_json::to_vec(&envelope).map_err(|_| Error::Invalid)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(Error::Limit);
        }
        #[cfg(test)]
        let failure = self.failure.take();
        #[cfg(test)]
        if failure == Some(Stage::OpenTemp) {
            return Err(Error::Io);
        }
        let mut temporary = Temporary::create(&self.directory.file)?;
        #[cfg(test)]
        if failure == Some(Stage::Write) {
            return Err(Error::Io);
        }
        temporary.file.write_all(&bytes).map_err(|_| Error::Io)?;
        let written = fd_stat(&temporary.file)
            .and_then(|stat| data_fingerprint(&stat))
            .map_err(|_| Error::Conflict)?;
        if written.size != bytes.len() as u64 {
            return Err(Error::Conflict);
        }
        #[cfg(test)]
        if failure == Some(Stage::SyncFile) {
            return Err(Error::Io);
        }
        temporary.file.sync_all().map_err(|_| Error::Io)?;
        #[cfg(test)]
        if let Some(hook) = self.before_rename.take() {
            let path = self
                .directory
                .path
                .join(std::ffi::OsStr::from_bytes(temporary.name.as_bytes()));
            hook(&path);
        }
        // Cooperative exclusivity is not sufficient to overwrite an external
        // edit/replacement. Compare the known inode + content metadata again.
        self.unchanged()?;
        let source_stamp = fd_stat(&temporary.file).and_then(|stat| data_fingerprint(&stat));
        let path_stamp = path_stat(&self.directory.file, temporary.name.as_bytes_with_nul())
            .and_then(|stat| stat.map(|stat| data_fingerprint(&stat)).transpose());
        if source_stamp != Ok(written) || path_stamp != Ok(Some(written)) {
            return Err(Error::Conflict);
        }
        #[cfg(test)]
        if failure == Some(Stage::Rename) {
            return Err(Error::Io);
        }
        let installed = if self.saved.is_none() {
            // Even after the final fingerprint check, a new file must not be
            // overwritten by this first installation. No non-atomic fallback.
            unsafe {
                libc::renameat2(
                    self.directory.file.as_raw_fd(),
                    temporary.name.as_ptr(),
                    self.directory.file.as_raw_fd(),
                    DATA.as_ptr().cast(),
                    libc::RENAME_NOREPLACE,
                )
            }
        } else {
            unsafe {
                libc::renameat(
                    self.directory.file.as_raw_fd(),
                    temporary.name.as_ptr(),
                    self.directory.file.as_raw_fd(),
                    DATA.as_ptr().cast(),
                )
            }
        };
        if installed != 0 {
            return Err(
                if self.saved.is_none()
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST)
                {
                    Error::Conflict
                } else {
                    Error::Io
                },
            );
        }
        temporary.renamed = true;
        #[cfg(test)]
        if let Some(hook) = self.after_rename.take() {
            hook(&self.directory.path.join("receipts-v1.json"));
        }
        let durable = (|| -> Result<Fingerprint, Error> {
            #[cfg(test)]
            if failure == Some(Stage::SyncDirectory) {
                return Err(Error::Io);
            }
            self.directory.file.sync_all().map_err(|_| Error::Io)?;
            self.directory.verify()?;
            let stamp = data_fingerprint(&fd_stat(&temporary.file)?)?;
            if !stamp.same_written_file(written)
                || at_fingerprint(&self.directory.file)? != Some(stamp)
            {
                return Err(Error::Conflict);
            }
            Ok(stamp)
        })();
        match durable {
            Ok(stamp) => {
                self.saved = Some(stamp);
                self.records = next;
                Ok(Ack {
                    record_id: record.record_id,
                    revision: record.revision,
                    local_turn: record.local_turn,
                })
            }
            Err(_) => {
                // Rename already happened. Never claim the old file survived,
                // emit a durable Ack or accept another write from this instance.
                self.poisoned = true;
                Err(Error::DurabilityUnknown)
            }
        }
    }
}
