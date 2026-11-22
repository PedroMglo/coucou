//! Bounded asynchronous STORE writer. Neither completion type is an execution
//! permit. Opening the backend directory/Journal is an explicit caller step;
//! this module owns no native child, pipe, session authority or replay path.
use super::journal::{self, Ack, Delivery, Journal, NativeRecord, Record, RecoveryRecord, State};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

const QUEUE: usize = 32;
const MAX_OUTSTANDING: usize = QUEUE + 1;
const MAX_RECORDS: usize = 128;

// IDs are unique across Writer instances in this process, and never wrap.
// Rejected/full submissions may leave gaps. They are correlation, not authority.
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Full,
    Closed,
    Cancelled,
    Invalid,
    Limit,
    Conflict,
    Io,
    DurabilityUnknown,
    Unavailable,
}

impl From<journal::Error> for Error {
    fn from(error: journal::Error) -> Self {
        match error {
            journal::Error::Invalid => Self::Invalid,
            journal::Error::Limit => Self::Limit,
            journal::Error::Conflict => Self::Conflict,
            journal::Error::Io => Self::Io,
            journal::Error::DurabilityUnknown => Self::DurabilityUnknown,
            journal::Error::Unavailable => Self::Unavailable,
        }
    }
}

/// One exact durable store result; deliberately not Clone/Serialize/Deserialize.
/// It cannot be converted into an execution capability.
pub(crate) struct Committed {
    request_id: u64,
    delivery: Delivery,
    native: Option<NativeRecord>,
    journal_ack: Ack,
}

impl Committed {
    pub(crate) fn request_id(&self) -> u64 {
        self.request_id
    }

    pub(crate) fn record_id(&self) -> &str {
        self.journal_ack.record_id()
    }

    pub(crate) fn revision(&self) -> u64 {
        self.journal_ack.revision()
    }

    pub(crate) fn local_turn(&self) -> u64 {
        self.journal_ack.local_turn()
    }

    pub(crate) fn delivery(&self) -> Delivery {
        self.delivery
    }

    pub(crate) fn native(&self) -> Option<&NativeRecord> {
        self.native.as_ref()
    }

    pub(crate) fn journal_ack(&self) -> &Ack {
        &self.journal_ack
    }

    pub(crate) fn matches(
        &self,
        request_id: u64,
        record_id: &str,
        revision: u64,
        local_turn: u64,
        delivery: Delivery,
        native: Option<&NativeRecord>,
    ) -> bool {
        self.request_id == request_id
            && self.journal_ack.matches(record_id, revision, local_turn)
            && self.delivery == delivery
            && self.native.as_ref() == native
    }
}

impl fmt::Debug for Committed {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("Committed")
            .field("request_id", &self.request_id)
            .field("revision", &self.revision())
            .field("local_turn", &self.local_turn())
            .field("delivery", &self.delivery)
            .field("has_native", &self.native.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(crate) enum Poll {
    Pending,
    Ready(Result<Committed, Error>),
    Consumed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Closure {
    Open,
    Manual,
    Unavailable,
}

struct Control {
    closure: Closure,
    // Preserve the originating error even though poisoning closes acceptance.
    durability_unknown: Option<u64>,
    sender: Option<SyncSender<Job>>,
    // Completed-but-unread replies retain a credit too. Without this, a
    // caller could drain the IO queue yet accumulate unlimited reply slots.
    outstanding: usize,
}

struct Shared {
    control: Mutex<Control>,
    recovery: Vec<RecoveryRecord>,
}

#[derive(Default)]
struct Reply {
    cancelled: bool,
    completion: Option<Result<Committed, Error>>,
}

struct Job {
    request_id: u64,
    record: Record,
    expected_revision: Option<u64>,
    reply: Arc<Mutex<Reply>>,
}

// No mutex below is held across Journal.save (including test IO hooks).
// Poisoned coordination locks are recovered solely to close/discard results;
// user callbacks never execute while these locks are held.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Shared {
    fn close(&self) {
        let mut control = lock(&self.control);
        if control.closure == Closure::Open {
            control.closure = Closure::Manual;
        }
        control.sender.take();
    }

    fn unavailable(&self, origin: Option<u64>) {
        let mut control = lock(&self.control);
        if control.closure == Closure::Open {
            control.closure = Closure::Unavailable;
        }
        if let Some(origin) = origin {
            control.durability_unknown = Some(origin);
        }
        control.sender.take();
    }
}

/// One app-owned handle. Use an external Arc if callers need to share it.
/// Drop/close never join the disk thread or wait for an active save to finish.
pub(crate) struct Writer {
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Hooks {
    #[cfg(test)]
    before_save: Option<Box<dyn FnMut(&Record, &mut Journal) + Send>>,
}

impl Writer {
    pub(crate) fn start(journal: Journal) -> Result<Self, Error> {
        Self::start_inner(journal, Hooks::default())
    }

    #[cfg(test)]
    pub(crate) fn start_with_hook_for_test<F>(
        journal: Journal,
        before_save: F,
    ) -> Result<Self, Error>
    where
        F: FnMut(&Record, &mut Journal) + Send + 'static,
    {
        Self::start_inner(
            journal,
            Hooks {
                before_save: Some(Box::new(before_save)),
            },
        )
    }

    fn start_inner(journal: Journal, _hooks: Hooks) -> Result<Self, Error> {
        let recovery = journal.recovery();
        let historical = recovery.iter().map(|row| row.record_id.clone()).collect();
        let (sender, receiver) = mpsc::sync_channel(QUEUE);
        let shared = Arc::new(Shared {
            control: Mutex::new(Control {
                closure: Closure::Open,
                durability_unknown: None,
                sender: Some(sender),
                outstanding: 0,
            }),
            recovery,
        });
        let worker = Worker {
            journal,
            shared: Arc::clone(&shared),
            historical,
            committed: BTreeMap::new(),
            #[cfg(test)]
            hooks: _hooks,
        };
        // Dropping the JoinHandle detaches, rather than joining, the sole owner
        // of Journal. Worker::drop closes replies on normal or unwinding exit.
        thread::Builder::new()
            .name("gravity-receipt-store".into())
            .spawn(move || worker.run(receiver))
            .map_err(|_| Error::Unavailable)?;
        Ok(Self { shared })
    }

    pub(crate) fn try_submit(
        &self,
        record: Record,
        expected_revision: Option<u64>,
    ) -> Result<Pending, Error> {
        // Length checks precede any copies. Compact owned buffers as well: a
        // short String can otherwise retain an arbitrarily large capacity in
        // the bounded queue or in an unread completion. Allocator work happens
        // outside the control lock so close/poll never wait on it.
        if !bounded_record(&record) {
            return Err(Error::Limit);
        }
        let record = compact_record(record);
        let mut control = lock(&self.shared.control);
        match control.closure {
            Closure::Manual => return Err(Error::Closed),
            Closure::Unavailable => return Err(Error::Unavailable),
            Closure::Open => {}
        }
        if control.outstanding >= MAX_OUTSTANDING {
            return Err(Error::Full);
        }
        let request_id = next_request_id()?;
        let reply = Arc::new(Mutex::new(Reply::default()));
        let job = Job {
            request_id,
            record,
            expected_revision,
            reply: Arc::clone(&reply),
        };
        let sender = control.sender.as_ref().ok_or(Error::Unavailable)?;
        match sender.try_send(job) {
            Ok(()) => {
                control.outstanding += 1;
                Ok(Pending {
                    request_id,
                    shared: Arc::clone(&self.shared),
                    reply,
                    consumed: false,
                })
            }
            Err(TrySendError::Full(_)) => Err(Error::Full),
            Err(TrySendError::Disconnected(_)) => Err(Error::Unavailable),
        }
    }

    pub(crate) fn close(&self) {
        self.shared.close();
    }

    /// Initial read-only snapshot, not a live registry or adoption mechanism.
    pub(crate) fn recovery(&self) -> Vec<RecoveryRecord> {
        self.shared.recovery.clone()
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        self.close();
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let control = lock(&self.shared.control);
        output
            .debug_struct("Writer")
            .field("closure", &control.closure)
            .field("outstanding", &control.outstanding)
            .field("historical_records", &self.shared.recovery.len())
            .finish_non_exhaustive()
    }
}

/// A single request/result. Dropping it cancels delivery, never rolls back IO.
/// No Clone means the same durable completion cannot be polled twice.
pub(crate) struct Pending {
    request_id: u64,
    shared: Arc<Shared>,
    reply: Arc<Mutex<Reply>>,
    consumed: bool,
}

impl Pending {
    pub(crate) fn request_id(&self) -> u64 {
        self.request_id
    }

    pub(crate) fn poll(&mut self) -> Poll {
        if self.consumed {
            return Poll::Consumed;
        }
        // Serialize result delivery with close. Nothing here waits for disk IO.
        let mut control = lock(&self.shared.control);
        let mut reply = lock(&self.reply);
        let result = if control.durability_unknown == Some(self.request_id) {
            Some(Err(Error::DurabilityUnknown))
        } else if reply.cancelled {
            Some(Err(Error::Cancelled))
        } else {
            match control.closure {
                Closure::Manual => Some(Err(Error::Closed)),
                Closure::Unavailable => Some(Err(Error::Unavailable)),
                Closure::Open => reply.completion.take(),
            }
        };
        if let Some(result) = result {
            // Close/cancel also consumes a previously completed OK result.
            reply.completion.take();
            self.consumed = true;
            control.outstanding -= 1;
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn cancel(&mut self) {
        if !self.consumed {
            let mut reply = lock(&self.reply);
            reply.cancelled = true;
            reply.completion.take();
        }
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !self.consumed {
            let mut control = lock(&self.shared.control);
            let mut reply = lock(&self.reply);
            reply.cancelled = true;
            reply.completion.take();
            self.consumed = true;
            control.outstanding -= 1;
        }
    }
}

fn bounded_record(record: &Record) -> bool {
    record.record_id.len() <= 128
        && record
            .error_code
            .as_ref()
            .is_none_or(|code| code.len() <= 64)
        && record.native.as_ref().is_none_or(|native| {
            native.session_id.len() <= 36
                && native.cwd.len() <= 16 * 1024
                && native.model.len() <= 256
        })
}

fn compact_record(mut record: Record) -> Record {
    fn compact(text: &mut String) {
        *text = std::mem::take(text).into_boxed_str().into_string();
    }
    compact(&mut record.record_id);
    if let Some(code) = &mut record.error_code {
        compact(code);
    }
    if let Some(native) = &mut record.native {
        compact(&mut native.session_id);
        compact(&mut native.cwd);
        compact(&mut native.model);
    }
    record
}

fn next_request_id() -> Result<u64, Error> {
    let mut current = NEXT_REQUEST.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1).ok_or(Error::Limit)?;
        match NEXT_REQUEST.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(current),
            Err(observed) => current = observed,
        }
    }
}

impl fmt::Debug for Pending {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("Pending")
            .field("request_id", &self.request_id)
            .field("consumed", &self.consumed)
            .finish_non_exhaustive()
    }
}

struct Worker {
    journal: Journal,
    shared: Arc<Shared>,
    historical: BTreeSet<String>,
    committed: BTreeMap<String, Record>,
    #[cfg(test)]
    hooks: Hooks,
}

impl Worker {
    fn run(mut self, receiver: Receiver<Job>) {
        while let Ok(job) = receiver.recv() {
            {
                let control = lock(&self.shared.control);
                if control.closure != Closure::Open {
                    break;
                }
                if lock(&job.reply).cancelled {
                    continue;
                }
                // This is the claim of the single active job. Cancellation or
                // close after this point can suppress its ACK, not promise that
                // the filesystem remains unchanged.
            }
            let result = self
                .validate(&job.record, job.expected_revision)
                .and_then(|()| {
                    #[cfg(test)]
                    if let Some(hook) = &mut self.hooks.before_save {
                        hook(&job.record, &mut self.journal);
                    }
                    self.journal
                        .save(job.record.clone(), job.expected_revision)
                        .map_err(Error::from)
                });
            let completion = match result {
                Ok(journal_ack) => {
                    // A real commit advances the fence even if its recipient
                    // cancelled/closed while IO ran and will never see the ACK.
                    self.committed
                        .insert(job.record.record_id.clone(), job.record.clone());
                    Ok(Committed {
                        request_id: job.request_id,
                        delivery: job.record.delivery,
                        native: job.record.native,
                        journal_ack,
                    })
                }
                Err(Error::DurabilityUnknown) => {
                    self.shared.unavailable(Some(job.request_id));
                    Err(Error::DurabilityUnknown)
                }
                Err(Error::Unavailable) => {
                    self.shared.unavailable(None);
                    Err(Error::Unavailable)
                }
                Err(error) => Err(error),
            };
            {
                let control = lock(&self.shared.control);
                let mut reply = lock(&job.reply);
                if control.closure == Closure::Open && !reply.cancelled {
                    reply.completion = Some(completion);
                }
            }
        }
    }

    fn validate(&self, next: &Record, expected_revision: Option<u64>) -> Result<(), Error> {
        if self.historical.contains(&next.record_id) {
            return Err(Error::Conflict);
        }
        let Some(old) = self.committed.get(&next.record_id) else {
            if self.historical.len() + self.committed.len() >= MAX_RECORDS {
                return Err(Error::Limit);
            }
            return if expected_revision.is_none()
                && next.revision == 1
                && next.local_turn == 1
                && next.delivery == Delivery::NotSent
                && next.last_known_state == State::Starting
                && next.native.is_none()
                && next.outcome.is_none()
                && !next.last_known_cleanup_confirmed
            {
                Ok(())
            } else {
                Err(Error::Conflict)
            };
        };
        if expected_revision != Some(old.revision)
            || old.revision.checked_add(1) != Some(next.revision)
            || (old.last_known_cleanup_confirmed && !next.last_known_cleanup_confirmed)
        {
            return Err(Error::Conflict);
        }
        if next.local_turn != old.local_turn {
            return if old.local_turn.checked_add(1) == Some(next.local_turn)
                && old.delivery == Delivery::Completed
                && old.last_known_state == State::Ready
                && old.native.is_some()
                && !old.last_known_cleanup_confirmed
                && next.delivery == Delivery::NotSent
                && next.last_known_state == State::Active
                && next.outcome.is_none()
                && next.native == old.native
                && !next.last_known_cleanup_confirmed
            {
                Ok(())
            } else {
                Err(Error::Conflict)
            };
        }
        match (&old.native, &next.native) {
            (Some(previous), Some(current)) if previous == current => {}
            (None, None) => {}
            (None, Some(_))
                if old.last_known_state == State::Starting
                    && next.last_known_state == State::Active => {}
            _ => return Err(Error::Conflict),
        }
        if !phase_follows(old.last_known_state, next.last_known_state)
            || !delivery_follows(old.delivery, next.delivery)
            || (old.delivery == Delivery::Completed && old.outcome != next.outcome)
        {
            return Err(Error::Conflict);
        }
        // Journal remains the authoritative schema/value/CAS and fsync guard.
        Ok(())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Also prevents permanently Pending replies if a test IO hook panics.
        self.shared.unavailable(None);
    }
}

fn phase_follows(previous: State, next: State) -> bool {
    previous == next
        || matches!(
            (previous, next),
            (
                State::Starting,
                State::Active | State::Stopping | State::Lost
            ) | (State::Active, State::Ready | State::Stopping | State::Lost)
                | (State::Ready, State::Stopping | State::Lost)
                | (State::Stopping, State::Lost)
        )
}

fn delivery_follows(previous: Delivery, next: Delivery) -> bool {
    matches!(
        (previous, next),
        (
            Delivery::NotSent,
            Delivery::NotSent | Delivery::Possible | Delivery::Unknown
        ) | (
            Delivery::Possible,
            Delivery::Possible | Delivery::Observed | Delivery::Completed | Delivery::Unknown
        ) | (
            Delivery::Observed,
            Delivery::Observed | Delivery::Completed | Delivery::Unknown
        ) | (Delivery::Completed, Delivery::Completed)
            | (Delivery::Unknown, Delivery::Unknown)
    )
}
