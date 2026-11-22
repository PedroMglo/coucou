//! Worker-local receipt sequencing. Store confirmations are bookkeeping only;
//! the owner must revalidate its live identity/epoch under the actual effect lock.
use super::journal::{self, Delivery, NativeRecord, Record, RecoveryRecord, State};
use super::persistence::{self, Pending, Poll, Writer};
use super::types::{
    display_text, Birth, Fault, Outcome, ReceiptPersistence, ReceiptPersistenceState, Snapshot,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) const STORE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub(super) enum Backend {
    Persistent(Arc<Writer>),
    #[cfg(test)]
    Memory,
}

impl Backend {
    pub(super) fn recovery(&self) -> Vec<RecoveryRecord> {
        match self {
            Self::Persistent(writer) => writer.recovery(),
            #[cfg(test)]
            Self::Memory => Vec::new(),
        }
    }

    pub(super) fn close(&self) {
        match self {
            Self::Persistent(writer) => writer.close(),
            #[cfg(test)]
            Self::Memory => {}
        }
    }

    pub(super) fn initial_status(&self) -> ReceiptPersistence {
        match self {
            Self::Persistent(_) => ReceiptPersistence::pending(),
            #[cfg(test)]
            Self::Memory => ReceiptPersistence::memory(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Purpose {
    Birth,
    NewTurn,
    Possible,
    Binding,
    Terminal,
    Lost,
}

#[derive(Clone, Eq, PartialEq)]
struct OwnerKey {
    handle: String,
    generation: u64,
    shared: usize,
}

/// Not Clone and not a capability: only the owning worker can compare this with
/// its currently locked live owner. Nothing is reconstructed from recovery.
pub(super) struct Confirmed {
    owner: OwnerKey,
    epoch: u64,
    turn: u64,
    purpose: Purpose,
    valid_until: Instant,
}

impl Confirmed {
    pub(super) fn matches(
        &self,
        birth: &Birth,
        shared: usize,
        epoch: u64,
        turn: u64,
        purpose: Purpose,
    ) -> bool {
        Instant::now() < self.valid_until
            && self.owner.handle == birth.handle
            && self.owner.generation == birth.generation
            && self.owner.shared == shared
            && self.epoch == epoch
            && self.turn == turn
            && self.purpose == purpose
    }

    pub(super) fn expired(&self) -> bool {
        Instant::now() >= self.valid_until
    }
}

enum Ticket {
    Disk(Pending),
    #[cfg(test)]
    Ready(Option<persistence::Committed>),
    #[cfg(test)]
    Memory,
}

struct Flight {
    ticket: Ticket,
    request_id: u64,
    sequence: u64,
    record: Record,
    purpose: Purpose,
    epoch: u64,
    deadline: Instant,
    historical: bool,
}

pub(super) struct Cursor {
    backend: Backend,
    owner: OwnerKey,
    record_id: String,
    created_at: Option<u64>,
    timeout: Duration,
    sequence: u64,
    durable: Option<Record>,
    flight: Option<Flight>,
    native: Option<NativeRecord>,
    cleanup_deadline: Option<Instant>,
    stopping: bool,
    final_durable: bool,
    abandoned: bool,
    error: Option<Fault>,
}

impl Cursor {
    pub(super) fn new(
        backend: Backend,
        birth: &Birth,
        shared: usize,
        record_id: String,
        created_at: Option<u64>,
        timeout: Duration,
    ) -> Self {
        Self {
            backend,
            owner: OwnerKey {
                handle: birth.handle.clone(),
                generation: birth.generation,
                shared,
            },
            record_id,
            created_at,
            timeout,
            sequence: 0,
            durable: None,
            flight: None,
            native: None,
            cleanup_deadline: None,
            stopping: false,
            final_durable: false,
            abandoned: false,
            error: None,
        }
    }

    pub(super) fn status(&self) -> ReceiptPersistence {
        #[cfg(test)]
        if matches!(self.backend, Backend::Memory) {
            return ReceiptPersistence::memory();
        }
        ReceiptPersistence {
            state: if self.abandoned {
                if self.error == Some(Fault::JournalDurabilityUnknown) {
                    ReceiptPersistenceState::Unknown
                } else {
                    ReceiptPersistenceState::Unavailable
                }
            } else if self.flight.is_some()
                || (self.stopping && !self.final_durable)
                || self.durable.is_none()
            {
                ReceiptPersistenceState::Pending
            } else if self.error.is_some() {
                ReceiptPersistenceState::Unavailable
            } else {
                ReceiptPersistenceState::Durable
            },
            revision: self.durable.as_ref().map(|record| record.revision),
            error: self.error,
        }
    }

    pub(super) fn pending(&self) -> bool {
        self.flight.is_some()
    }

    pub(super) fn abandoned(&self) -> bool {
        self.abandoned
    }

    pub(super) fn final_durable(&self) -> bool {
        self.final_durable
    }

    pub(super) fn begin_stopping(&mut self) {
        self.stopping = true;
    }

    pub(super) fn effect_timeout(&mut self) {
        self.error = Some(Fault::JournalUnavailable);
    }

    pub(super) fn terminal_durable(&self, state: &Snapshot) -> bool {
        self.durable.as_ref().is_some_and(|record| {
            record.local_turn == state.receipt.turn
                && record.delivery == Delivery::Completed
                && record.outcome == state.receipt.outcome.map(outcome)
        })
    }

    pub(super) fn has_birth(&self) -> bool {
        self.durable.is_some()
    }

    pub(super) fn durable_turn(&self) -> Option<u64> {
        self.durable.as_ref().map(|record| record.local_turn)
    }

    pub(super) fn needs_binding(&self) -> bool {
        self.native.is_some()
            && self
                .durable
                .as_ref()
                .and_then(|record| record.native.as_ref())
                != self.native.as_ref()
    }

    pub(super) fn possible(&self, turn: u64) -> bool {
        self.durable.as_ref().is_some_and(|record| {
            record.local_turn == turn
                && matches!(record.delivery, Delivery::Possible | Delivery::Observed)
                && record.native == self.native
        })
    }

    pub(super) fn reference_matches(&self, reference: Option<&super::types::Reference>) -> bool {
        match (&self.native, reference) {
            (None, None) => true,
            (Some(native), Some(reference)) => {
                reference.session_id == native.session_id
                    && reference.handle == self.owner.handle
                    && reference.generation == self.owner.generation
                    && reference.provider == super::types::PROVIDER
                    && reference.pet_id == super::types::PET
            }
            _ => false,
        }
    }

    /// Inputs are the raw facts already accepted by bind, not caller/config/view
    /// guesses. The raw path remains in owner; journal uses the reviewed display
    /// projection of that exact verified value.
    pub(super) fn native(
        &mut self,
        session_id: &str,
        raw_cwd: &std::path::Path,
        model: &str,
    ) -> Result<(), Fault> {
        let native = NativeRecord {
            session_id: session_id.into(),
            cwd: display_text(&raw_cwd.to_string_lossy()),
            model: model.into(),
        };
        if self.native.as_ref().is_some_and(|old| old != &native) {
            return Err(Fault::Identity);
        }
        self.native = Some(native);
        Ok(())
    }

    /// This is an absolute reconciliation budget, shared by every subsequent
    /// binding/terminal/Lost save. It is never renewed by another submission.
    pub(super) fn begin_cleanup(&mut self, now: Instant, shutdown: Option<Instant>) {
        self.stopping = true;
        if self.cleanup_deadline.is_none() {
            self.cleanup_deadline = Some(now + self.timeout);
        }
        if let Some(shutdown) = shutdown {
            self.cleanup_deadline = self.cleanup_deadline.map(|limit| limit.min(shutdown));
        }
        if self.durable.is_none() && self.flight.is_none() && !self.abandoned {
            self.abandon(Fault::JournalUnavailable);
        }
    }

    pub(super) fn abandon(&mut self, error: Fault) {
        // A pending save may have committed. Drop cancels only ACK delivery and
        // permanently abandons CAS for this record, never claims rollback.
        self.flight.take();
        self.abandoned = true;
        self.error = Some(if self.error == Some(Fault::JournalDurabilityUnknown) {
            Fault::JournalDurabilityUnknown
        } else {
            error
        });
    }

    fn failed(&mut self, error: persistence::Error, historical: bool) -> Fault {
        let unknown = matches!(
            error,
            persistence::Error::DurabilityUnknown
                | persistence::Error::Closed
                | persistence::Error::Cancelled
                | persistence::Error::Unavailable
        );
        let fault = if unknown {
            Fault::JournalDurabilityUnknown
        } else {
            Fault::JournalUnavailable
        };
        // At most one final historical sequence after a failed effect barrier.
        // Failure within reconciliation, or a missing initial row, settles it.
        if unknown || historical || self.durable.is_none() || self.error.is_some() {
            self.abandoned = true;
        }
        self.error = Some(fault);
        fault
    }

    pub(super) fn poll(
        &mut self,
        now: Instant,
        shutdown: Option<Instant>,
    ) -> Result<Option<Confirmed>, Fault> {
        if self.abandoned {
            return Ok(None);
        }
        if let (Some(limit), Some(shutdown)) = (self.cleanup_deadline, shutdown) {
            self.cleanup_deadline = Some(limit.min(shutdown));
        }
        let Some(flight) = self.flight.as_mut() else {
            if self.cleanup_deadline.is_some_and(|limit| now >= limit) && !self.final_durable {
                self.abandon(Fault::JournalUnavailable);
                return Err(Fault::JournalUnavailable);
            }
            return Ok(None);
        };
        // Poll first: a response already available at this tick can still update
        // bookkeeping, but the owner separately rejects all expired effects.
        let response = match &mut flight.ticket {
            Ticket::Disk(pending) => match pending.poll() {
                Poll::Pending => None,
                Poll::Ready(result) => Some(result.map(Some)),
                Poll::Consumed => Some(Err(persistence::Error::Unavailable)),
            },
            #[cfg(test)]
            Ticket::Memory => Some(Ok(None)),
            #[cfg(test)]
            Ticket::Ready(committed) => Some(Ok(committed.take())),
        };
        let Some(response) = response else {
            if now >= flight.deadline || self.cleanup_deadline.is_some_and(|limit| now >= limit) {
                self.abandon(Fault::JournalDurabilityUnknown);
                return Err(Fault::JournalDurabilityUnknown);
            }
            return Ok(None);
        };
        let flight = self.flight.take().expect("flight checked above");
        match response {
            Ok(committed) => {
                let valid = flight.sequence == self.sequence
                    && committed.as_ref().is_none_or(|committed| {
                        committed.matches(
                            flight.request_id,
                            &flight.record.record_id,
                            flight.record.revision,
                            flight.record.local_turn,
                            flight.record.delivery,
                            flight.record.native.as_ref(),
                        )
                    });
                if !valid {
                    self.abandon(Fault::JournalDurabilityUnknown);
                    return Err(Fault::JournalDurabilityUnknown);
                }
                self.final_durable = flight.purpose == Purpose::Lost;
                self.durable = Some(flight.record.clone());
                if self.final_durable {
                    self.error = None;
                }
                Ok(Some(Confirmed {
                    owner: self.owner.clone(),
                    epoch: flight.epoch,
                    turn: flight.record.local_turn,
                    purpose: flight.purpose,
                    valid_until: flight.deadline,
                }))
            }
            Err(error) => Err(self.failed(error, flight.historical)),
        }
    }

    pub(super) fn submit(
        &mut self,
        purpose: Purpose,
        epoch: u64,
        state: &Snapshot,
    ) -> Result<(), Fault> {
        if self.abandoned || self.flight.is_some() {
            return Err(Fault::JournalUnavailable);
        }
        if self
            .cleanup_deadline
            .is_some_and(|limit| Instant::now() >= limit)
        {
            self.abandon(Fault::JournalUnavailable);
            return Err(Fault::JournalUnavailable);
        }
        let historical = self.cleanup_deadline.is_some();
        let revision = self
            .durable
            .as_ref()
            .map_or(Some(1), |old| old.revision.checked_add(1))
            .ok_or(Fault::JournalUnavailable)?;
        let (delivery, phase, outcome, native, cleaned) = match purpose {
            Purpose::Birth => (Delivery::NotSent, State::Starting, None, None, false),
            Purpose::NewTurn => (
                Delivery::NotSent,
                State::Active,
                None,
                self.native.clone(),
                false,
            ),
            Purpose::Possible => (
                Delivery::Possible,
                if self.native.is_some() {
                    State::Active
                } else {
                    State::Starting
                },
                None,
                self.native.clone(),
                false,
            ),
            Purpose::Binding => (
                self.durable
                    .as_ref()
                    .map_or(Delivery::NotSent, |old| old.delivery),
                State::Active,
                None,
                self.native.clone(),
                false,
            ),
            Purpose::Terminal => (
                Delivery::Completed,
                if state.receipt.outcome == Some(Outcome::Waiting) {
                    State::Stopping
                } else {
                    State::Ready
                },
                state.receipt.outcome.map(outcome),
                self.native.clone(),
                false,
            ),
            Purpose::Lost => {
                let completed = state.receipt.delivery == super::types::Delivery::Completed;
                let delivery = if completed {
                    Delivery::Completed
                } else if self
                    .durable
                    .as_ref()
                    .is_some_and(|old| old.delivery == Delivery::NotSent)
                {
                    Delivery::NotSent
                } else {
                    Delivery::Unknown
                };
                (
                    delivery,
                    State::Lost,
                    if completed {
                        state.receipt.outcome.map(outcome)
                    } else {
                        Some(journal::Outcome::Lost)
                    },
                    self.native.clone(),
                    true,
                )
            }
        };
        let record = Record {
            record_id: self.record_id.clone(),
            revision,
            created_at: self.created_at,
            updated_at: state.metadata.updated_at_ms,
            native,
            local_turn: state.receipt.turn,
            delivery,
            outcome,
            last_known_state: phase,
            last_known_cleanup_confirmed: cleaned,
            error_code: if purpose == Purpose::Lost {
                state.fault.map(|fault| fault.code().to_owned())
            } else {
                None
            },
        };
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or(Fault::JournalUnavailable)?;
        let expected = self.durable.as_ref().map(|old| old.revision);
        let (ticket, request_id) = match &self.backend {
            Backend::Persistent(writer) => match writer.try_submit(record.clone(), expected) {
                Ok(pending) => {
                    let id = pending.request_id();
                    (Ticket::Disk(pending), id)
                }
                Err(error) => {
                    // No request was admitted here, so Closed/Unavailable does
                    // not hide a commit of this attempted submission.
                    let error = match error {
                        persistence::Error::Closed | persistence::Error::Unavailable => {
                            persistence::Error::Io
                        }
                        other => other,
                    };
                    return Err(self.failed(error, historical));
                }
            },
            #[cfg(test)]
            Backend::Memory => (Ticket::Memory, sequence),
        };
        self.sequence = sequence;
        self.flight = Some(Flight {
            ticket,
            request_id,
            sequence,
            record,
            purpose,
            epoch,
            deadline: Instant::now() + self.timeout,
            historical,
        });
        Ok(())
    }
}

fn outcome(value: Outcome) -> journal::Outcome {
    match value {
        Outcome::Succeeded => journal::Outcome::Succeeded,
        Outcome::Failed => journal::Outcome::Failed,
        Outcome::Cancelled => journal::Outcome::Cancelled,
        Outcome::Interrupted => journal::Outcome::Interrupted,
        Outcome::Waiting => journal::Outcome::Waiting,
        Outcome::Lost => journal::Outcome::Lost,
    }
}

#[cfg(test)]
mod tests {
    use super::super::journal::Journal;
    use super::super::types::{Diagnostics, Phase, Receipt, ViewMetadata};
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let directory = std::env::current_dir()
                .unwrap()
                .join(".build/gravity-managed-harness/fixtures")
                .join(format!(
                    "durability-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
            Self(directory)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn cursor_with_ready_ack() -> (Fixture, Birth, Cursor) {
        let fixture = Fixture::new();
        let writer = Writer::start(Journal::open(&fixture.0).unwrap()).unwrap();
        let birth = Birth {
            handle: "unit-owner-not-persisted".into(),
            generation: 1,
        };
        let state = Snapshot {
            metadata: ViewMetadata::new(),
            reference: None,
            phase: Phase::Starting,
            receipt: Receipt {
                turn: 1,
                delivery: super::super::types::Delivery::NotSent,
                outcome: None,
            },
            receipt_persistence: ReceiptPersistence::pending(),
            history: Vec::new(),
            partial_reply: String::new(),
            diagnostics: Diagnostics::default(),
            fault: None,
            cleanup_confirmed: false,
            terminal_handoff: false,
            approvals: false,
        };
        let mut cursor = Cursor::new(
            Backend::Persistent(Arc::new(writer)),
            &birth,
            7,
            "unit-historical-record".into(),
            None,
            STORE_TIMEOUT,
        );
        cursor.submit(Purpose::Birth, 3, &state).unwrap();
        // Obtain a REAL durable completion, then hold that already-Ready value
        // inside a private cfg(test) ticket. No timer sleep fabricates readiness
        // or a durable Ack, and production has no such ticket constructor.
        let flight = cursor.flight.as_mut().unwrap();
        let Ticket::Disk(mut pending) = std::mem::replace(&mut flight.ticket, Ticket::Ready(None))
        else {
            panic!("persistent submission must have a real pending ticket");
        };
        let wait_limit = Instant::now() + Duration::from_secs(5);
        let committed = loop {
            match pending.poll() {
                Poll::Ready(Ok(committed)) => break committed,
                Poll::Pending => {
                    assert!(Instant::now() < wait_limit, "real journal commit timed out");
                    std::thread::sleep(Duration::from_millis(1));
                }
                _ => panic!("real journal commit failed"),
            }
        };
        assert_eq!(committed.revision(), 1);
        assert!(committed
            .journal_ack()
            .matches("unit-historical-record", 1, 1));
        flight.ticket = Ticket::Ready(Some(committed));
        (fixture, birth, cursor)
    }

    #[test]
    fn confirmed_receipt_is_exact_to_owner_epoch_turn_and_purpose_and_consumed_once() {
        let (_fixture, birth, mut cursor) = cursor_with_ready_ack();
        let confirmed = cursor.poll(Instant::now(), None).unwrap().unwrap();
        assert!(confirmed.matches(&birth, 7, 3, 1, Purpose::Birth));
        let wrong_handle = Birth {
            handle: "different-owner".into(),
            generation: birth.generation,
        };
        let wrong_generation = Birth {
            handle: birth.handle.clone(),
            generation: birth.generation + 1,
        };
        assert!(!confirmed.matches(&wrong_handle, 7, 3, 1, Purpose::Birth));
        assert!(!confirmed.matches(&wrong_generation, 7, 3, 1, Purpose::Birth));
        assert!(!confirmed.matches(&birth, 8, 3, 1, Purpose::Birth));
        assert!(!confirmed.matches(&birth, 7, 4, 1, Purpose::Birth));
        assert!(!confirmed.matches(&birth, 7, 3, 2, Purpose::Birth));
        assert!(!confirmed.matches(&birth, 7, 3, 1, Purpose::Possible));
        assert!(cursor.poll(Instant::now(), None).unwrap().is_none());
        cursor.backend.close();
    }

    #[test]
    fn already_ready_late_ack_advances_cursor_without_authorizing_an_effect() {
        let (_fixture, birth, mut cursor) = cursor_with_ready_ack();
        let flight = cursor.flight.as_mut().unwrap();
        flight.deadline = Instant::now() - Duration::from_millis(1);
        let confirmed = cursor.poll(Instant::now(), None).unwrap().unwrap();
        assert_eq!(cursor.durable.as_ref().unwrap().revision, 1);
        assert_eq!(cursor.status().state, ReceiptPersistenceState::Durable);
        assert!(confirmed.expired());
        assert!(!confirmed.matches(&birth, 7, 3, 1, Purpose::Birth));
        assert!(cursor.poll(Instant::now(), None).unwrap().is_none());
        cursor.backend.close();
    }
}
