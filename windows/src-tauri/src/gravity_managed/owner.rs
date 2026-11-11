//! Native ownership with durable receipt barriers, separate from Mochi Chat and
//! external LS. A journal confirmation alone never authorizes a native effect.
use super::durability::{Backend, Confirmed, Cursor, Purpose, STORE_TIMEOUT};
use super::events::{self, NativeEvent, NativeStatus};
use super::gate::{
    Gate, HookRequest, HumanChoice, NativeOperation, Pending, Rejection, Relay, Response,
};
use super::transport::Transport;
use super::types::*;
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(crate) struct Manager {
    registry: Arc<Mutex<Registry>>,
    backend: Backend,
    store_timeout: Duration,
}

struct Registry {
    nonce: String,
    record_nonce: String,
    next: u64,
    closing: bool,
    shutdown_deadline: Option<Instant>,
    owners: HashMap<String, Entry>,
    sessions: HashMap<String, String>,
}
struct Entry {
    generation: u64,
    shared: Arc<Shared>,
    sender: SyncSender<Request>,
}
struct Shared {
    state: Mutex<Snapshot>,
    gate: Mutex<Gate>,
    stop: AtomicBool,
    epoch: AtomicU64,
}
struct Request {
    turn: u64,
    query: String,
    admitted_at: Instant,
    epoch: u64,
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        invalidate_epoch(&self.shared);
    }
}

impl Manager {
    /// Production always receives an explicitly bootstrapped receipt writer.
    pub(crate) fn with_persistence(writer: super::persistence::Writer) -> Result<Self, Fault> {
        Self::construct(Backend::Persistent(Arc::new(writer)), STORE_TIMEOUT)
    }

    #[cfg(test)]
    pub(crate) fn with_persistence_timeout_for_test(
        writer: super::persistence::Writer,
        timeout: Duration,
    ) -> Result<Self, Fault> {
        if timeout.is_zero() || timeout > STORE_TIMEOUT {
            return Err(Fault::InvalidConfig);
        }
        Self::construct(Backend::Persistent(Arc::new(writer)), timeout)
    }

    /// RAM compatibility for ownership/gate fixtures only. Absent in production.
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, Fault> {
        Self::construct(Backend::Memory, STORE_TIMEOUT)
    }

    fn construct(backend: Backend, store_timeout: Duration) -> Result<Self, Fault> {
        let mut nonce = [0_u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut nonce))
            .map_err(|_| Fault::Io)?;
        let historical = nonce[16..].iter().map(|n| format!("{n:02x}")).collect();
        let nonce = nonce[..16].iter().map(|n| format!("{n:02x}")).collect();
        Ok(Self {
            registry: Arc::new(Mutex::new(Registry {
                nonce,
                record_nonce: historical,
                next: 0,
                closing: false,
                shutdown_deadline: None,
                owners: HashMap::new(),
                sessions: HashMap::new(),
            })),
            backend,
            store_timeout,
        })
    }

    pub(crate) fn recovery(&self) -> Vec<super::journal::RecoveryRecord> {
        self.backend.recovery()
    }

    /// Creation requires the user's explicit first text. No resume ID, hidden
    /// warming prompt, external adoption, or inference fallback is accepted.
    pub(crate) fn create(&self, config: Config, first_query: String) -> Result<Birth, Fault> {
        config.validate()?;
        validate_query(&first_query)?;
        let mut registry = self.registry.lock().map_err(|_| Fault::Io)?;
        if registry.closing {
            return Err(Fault::NotReady);
        }
        if registry.owners.len() >= MAX_OWNERS {
            return Err(Fault::Capacity);
        }
        let generation = registry.next.checked_add(1).ok_or(Fault::Capacity)?;
        registry.next = generation;
        let admitted_at = Instant::now();
        let record_id = format!("gravity-receipt-{}-{generation}", registry.record_nonce);
        let birth = Birth {
            handle: format!("gravity-{}-{generation}", registry.nonce),
            generation,
        };
        let shared = Arc::new(Shared {
            state: Mutex::new(Snapshot {
                metadata: ViewMetadata::new(),
                reference: None,
                phase: Phase::Starting,
                receipt: Receipt {
                    turn: 1,
                    delivery: Delivery::NotSent,
                    outcome: None,
                },
                receipt_persistence: self.backend.initial_status(),
                history: Vec::new(),
                partial_reply: String::new(),
                diagnostics: Diagnostics::default(),
                fault: None,
                cleanup_confirmed: false,
                terminal_handoff: false,
                approvals: false,
            }),
            stop: AtomicBool::new(false),
            epoch: AtomicU64::new(1),
            gate: Mutex::new(Gate::new()),
        });
        let (sender, receiver) = mpsc::sync_channel(1);
        registry.owners.insert(
            birth.handle.clone(),
            Entry {
                generation,
                shared: shared.clone(),
                sender,
            },
        );
        let weak = Arc::downgrade(&self.registry);
        let worker_birth = birth.clone();
        let backend = self.backend.clone();
        let store_timeout = self.store_timeout;
        if std::thread::Builder::new()
            .name("gravity-managed".into())
            .spawn(move || {
                worker(
                    weak,
                    worker_birth,
                    shared,
                    config,
                    first_query,
                    receiver,
                    backend,
                    record_id,
                    admitted_at,
                    store_timeout,
                );
            })
            .is_err()
        {
            registry.owners.remove(&birth.handle);
            return Err(Fault::Spawn);
        }
        Ok(birth)
    }

    pub(crate) fn inspect_birth(&self, birth: &Birth) -> Result<Snapshot, Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = registry
            .owners
            .get(&birth.handle)
            .filter(|e| e.generation == birth.generation)
            .ok_or(Fault::WrongReference)?;
        let state = entry.shared.state.lock().map_err(|_| Fault::Io)?.clone();
        Ok(state)
    }

    pub(crate) fn list_births(&self) -> Result<Vec<Birth>, Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let mut births: Vec<_> = registry
            .owners
            .iter()
            .map(|(handle, entry)| Birth {
                handle: handle.clone(),
                generation: entry.generation,
            })
            .collect();
        births.sort_by_key(|birth| birth.generation);
        Ok(births)
    }

    pub(crate) fn inspect(&self, reference: &Reference) -> Result<Snapshot, Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = exact(&registry, reference)?;
        let state = entry.shared.state.lock().map_err(|_| Fault::Io)?.clone();
        Ok(state)
    }

    /// Admission only. A returned sequence never claims native delivery or a
    /// completed operation. The worker remains the sole stdin writer.
    pub(crate) fn send(&self, reference: &Reference, query: String) -> Result<u64, Fault> {
        validate_query(&query)?;
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        if registry.closing {
            return Err(Fault::NotReady);
        }
        let entry = exact(&registry, reference)?;
        let mut state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
        if state.phase != Phase::Ready || entry.shared.stop.load(Ordering::Acquire) {
            return Err(Fault::Busy);
        }
        history_room(&state, query.len() + MAX_REPLY, 2)?;
        let turn = state.receipt.turn.checked_add(1).ok_or(Fault::Capacity)?;
        let epoch = entry
            .shared
            .epoch
            .load(Ordering::Acquire)
            .checked_add(1)
            .ok_or(Fault::Capacity)?;
        let admitted_at = Instant::now();
        let mut gate = entry.shared.gate.lock().map_err(|_| Fault::Io)?;
        match entry.sender.try_send(Request {
            turn,
            query,
            admitted_at,
            epoch,
        }) {
            Ok(()) => {
                entry.shared.epoch.store(epoch, Ordering::Release);
                state.phase = Phase::Active;
                state.receipt = Receipt {
                    turn,
                    delivery: Delivery::NotSent,
                    outcome: None,
                };
                state.partial_reply.clear();
                state.fault = None;
                mark_pending(&mut state);
                state.changed();
                let _ = gate.begin_turn(turn);
                Ok(turn)
            }
            Err(mpsc::TrySendError::Full(_)) => Err(Fault::Busy),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(Fault::Closed),
        }
    }

    /// This stops the owned process/group, not just a turn. It loses the native
    /// transport. No ready/restart/resume is implied by this acknowledgement.
    pub(crate) fn stop(&self, reference: &Reference) -> Result<(), Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        request_stop(exact(&registry, reference)?)
    }

    pub(crate) fn stop_birth(&self, birth: &Birth) -> Result<(), Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = registry
            .owners
            .get(&birth.handle)
            .filter(|e| e.generation == birth.generation)
            .ok_or(Fault::WrongReference)?;
        request_stop(entry)
    }

    /// The facade's pending-birth route cannot bypass a confirmed Reference.
    /// Holding registry through the check+Stop prevents init's bind from racing.
    pub(crate) fn stop_unconfirmed_birth(&self, birth: &Birth) -> Result<(), Fault> {
        let registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = registry
            .owners
            .get(&birth.handle)
            .filter(|e| e.generation == birth.generation)
            .ok_or(Fault::WrongReference)?;
        {
            let state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
            if state.reference.is_some() {
                return Err(Fault::WrongReference);
            }
        }
        request_stop(entry)
    }

    /// Only a fully collected lost owner can be forgotten. Dropping a webview
    /// reference or subscription does not call Stop or destroy the worker.
    pub(crate) fn forget_lost(&self, reference: &Reference) -> Result<(), Fault> {
        let mut registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = exact(&registry, reference)?;
        let state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
        if !state.can_forget_lost() {
            return Err(Fault::Busy);
        }
        drop(state);
        invalidate_epoch(&entry.shared);
        registry.owners.remove(&reference.handle);
        if registry.sessions.get(&reference.session_id) == Some(&reference.handle) {
            registry.sessions.remove(&reference.session_id);
        }
        Ok(())
    }

    /// Failed births have no native reference. Retain their receipt until the
    /// caller explicitly forgets this exact, fully collected birth token.
    pub(crate) fn forget_lost_birth(&self, birth: &Birth) -> Result<(), Fault> {
        let mut registry = self.registry.lock().map_err(|_| Fault::Io)?;
        let entry = registry
            .owners
            .get(&birth.handle)
            .filter(|e| e.generation == birth.generation)
            .ok_or(Fault::WrongReference)?;
        let state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
        if state.reference.is_some() || !state.can_forget_lost() {
            return Err(Fault::Busy);
        }
        drop(state);
        invalidate_epoch(&entry.shared);
        registry.owners.remove(&birth.handle);
        Ok(())
    }

    /// App shutdown must await this on a background thread before exiting.
    /// Dropping a webview never calls it. Failure leaves owners stopping and
    /// rejects new births; it is not permission to abandon unconfirmed cleanup.
    pub(crate) fn shutdown(&self, timeout: Duration) -> Result<(), Fault> {
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(Fault::InvalidConfig);
        }
        let deadline = Instant::now() + timeout;
        let owners = {
            let mut registry = self.registry.lock().map_err(|_| Fault::Io)?;
            registry.closing = true;
            registry.shutdown_deadline = Some(
                registry
                    .shutdown_deadline
                    .map_or(deadline, |old| old.min(deadline)),
            );
            let mut owners = Vec::new();
            for entry in registry.owners.values() {
                request_stop(entry)?;
                owners.push(entry.shared.clone());
            }
            owners
        };
        loop {
            let mut all_cleaned = true;
            let mut pending = false;
            let mut storage_error = None;
            for owner in &owners {
                let state = owner.state.lock().map_err(|_| Fault::Io)?;
                all_cleaned &= state.phase == Phase::Lost && state.cleanup_confirmed;
                match state.receipt_persistence.state {
                    ReceiptPersistenceState::Pending => pending = true,
                    ReceiptPersistenceState::Unknown => {
                        storage_error = Some(Fault::JournalDurabilityUnknown)
                    }
                    ReceiptPersistenceState::Unavailable => {
                        if storage_error.is_none() {
                            storage_error = Some(Fault::JournalUnavailable);
                        }
                    }
                    _ => {}
                }
            }
            if all_cleaned && !pending {
                self.backend.close();
                return storage_error.map_or(Ok(()), Err);
            }
            if Instant::now() >= deadline {
                if !all_cleaned {
                    return Err(Fault::CleanupUnconfirmed);
                }
                self.backend.close();
                return Err(storage_error.unwrap_or(Fault::JournalUnavailable));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Socket provenance chooses the owner. The untrusted hook UUID never
    /// selects a registry entry. No public IPC or approval capability calls this.
    /// Matching the currently observed operation is not proof that no identical
    /// concurrent native hook/event is still unread. Real integration must prove
    /// native operation correlation/ordering first; public approvals remain off.
    pub(crate) fn begin_gate(
        &self,
        relay: Relay,
        request: HookRequest,
        ttl: Duration,
    ) -> Result<Pending, Rejection> {
        let registry = self.registry.lock().map_err(|_| Rejection::Unavailable)?;
        let mut selected = None;
        for (handle, entry) in &registry.owners {
            let state = entry
                .shared
                .state
                .lock()
                .map_err(|_| Rejection::Unavailable)?;
            if state.phase != Phase::Active
                || entry.shared.stop.load(Ordering::Acquire)
                || !matches!(state.receipt.delivery, Delivery::Sent | Delivery::Observed)
            {
                continue;
            }
            let gate = entry
                .shared
                .gate
                .lock()
                .map_err(|_| Rejection::Unavailable)?;
            if gate.matches_peer(&relay) {
                if selected.is_some() {
                    return Err(Rejection::Ambiguous);
                }
                selected = Some(handle.clone());
            }
        }
        let handle = selected.ok_or(Rejection::Untrusted)?;
        let entry = registry.owners.get(&handle).ok_or(Rejection::Stale)?;
        let state = entry
            .shared
            .state
            .lock()
            .map_err(|_| Rejection::Unavailable)?;
        if state.phase != Phase::Active || entry.shared.stop.load(Ordering::Acquire) {
            return Err(Rejection::Stale);
        }
        let pending = entry
            .shared
            .gate
            .lock()
            .map_err(|_| Rejection::Unavailable)?
            .request(relay, request, ttl)?;
        Ok(pending)
    }

    /// Called only by a future real human-decision broker. Pending owns the
    /// exact relay, and Stop/result/EOF share this state→gate lock ordering.
    pub(crate) fn respond_gate(
        &self,
        pending: Pending,
        choice: HumanChoice,
    ) -> Result<Response, Rejection> {
        let reference = pending.card().reference.clone();
        let turn = pending.card().turn;
        let registry = self.registry.lock().map_err(|_| Rejection::Unavailable)?;
        let entry = exact(&registry, &reference).map_err(|_| Rejection::Stale)?;
        let state = entry
            .shared
            .state
            .lock()
            .map_err(|_| Rejection::Unavailable)?;
        if state.phase != Phase::Active
            || entry.shared.stop.load(Ordering::Acquire)
            || state.receipt.turn != turn
            || !matches!(state.receipt.delivery, Delivery::Sent | Delivery::Observed)
        {
            return Err(Rejection::Stale);
        }
        let response = entry
            .shared
            .gate
            .lock()
            .map_err(|_| Rejection::Unavailable)?
            .respond(pending, choice)?;
        Ok(response)
    }
}

fn exact<'a>(registry: &'a Registry, reference: &Reference) -> Result<&'a Entry, Fault> {
    if reference.provider != PROVIDER
        || reference.pet_id != PET
        || !valid_uuid(&reference.session_id)
        || reference.generation == 0
    {
        return Err(Fault::WrongReference);
    }
    let entry = registry
        .owners
        .get(&reference.handle)
        .filter(|e| e.generation == reference.generation)
        .ok_or(Fault::WrongReference)?;
    let state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
    if state.reference.as_ref() != Some(reference) {
        return Err(Fault::WrongReference);
    }
    Ok(entry)
}

fn request_stop(entry: &Entry) -> Result<(), Fault> {
    let mut state = entry.shared.state.lock().map_err(|_| Fault::Io)?;
    if state.phase == Phase::Lost && state.cleanup_confirmed {
        return Ok(());
    }
    let mut changed = false;
    if state.phase != Phase::Stopping {
        state.phase = Phase::Stopping;
        changed = true;
    }
    if !entry.shared.stop.swap(true, Ordering::AcqRel) {
        invalidate_epoch(&entry.shared);
    }
    changed |= mark_pending(&mut state);
    entry.shared.gate.lock().map_err(|_| Fault::Io)?.revoke();
    if changed {
        state.changed();
    }
    Ok(())
}

fn invalidate_epoch(shared: &Shared) {
    let mut current = shared.epoch.load(Ordering::Acquire);
    while let Some(next) = current.checked_add(1) {
        match shared
            .epoch
            .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => return,
            Err(observed) => current = observed,
        }
    }
    // Exhaustion can only disable effects, never wrap into an old epoch.
    shared.stop.store(true, Ordering::Release);
}

fn mark_pending(state: &mut Snapshot) -> bool {
    if state.receipt_persistence.state == ReceiptPersistenceState::Durable {
        state.receipt_persistence.state = ReceiptPersistenceState::Pending;
        state.receipt_persistence.error = None;
        true
    } else {
        false
    }
}

fn publish_persistence(shared: &Shared, cursor: &Cursor) {
    let mut state = shared
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut status = cursor.status();
    if state.phase == Phase::Stopping
        && shared.stop.load(Ordering::Acquire)
        && status.state == ReceiptPersistenceState::Durable
    {
        // Stop can win after the tick's initial stop check but before polling a
        // successful active checkpoint. That ACK is not the final Lost receipt.
        status.state = ReceiptPersistenceState::Pending;
    }
    if state.receipt_persistence != status {
        state.receipt_persistence = status;
        state.changed();
    }
}

fn history_room(state: &Snapshot, additional: usize, messages: usize) -> Result<(), Fault> {
    let size: usize = state.history.iter().map(|m| m.text.len()).sum();
    if state.history.len() + messages > MAX_MESSAGES || size + additional > MAX_HISTORY {
        Err(Fault::Limit)
    } else {
        Ok(())
    }
}

struct Turn {
    number: u64,
    query: String,
    wire: Vec<u8>,
    offset: usize,
    observed: bool,
    deadline: Instant,
    epoch: u64,
    authorized: bool,
}

impl Turn {
    fn new(
        number: u64,
        query: String,
        timeout: Duration,
        admitted_at: Instant,
        epoch: u64,
    ) -> Result<Self, Fault> {
        validate_query(&query)?;
        let mut wire =
            serde_json::to_vec(&serde_json::json!({"event":"user","message":{"content":query}}))
                .map_err(|_| Fault::Protocol)?;
        if wire.len() >= MAX_LINE {
            return Err(Fault::Limit);
        }
        wire.push(b'\n');
        Ok(Self {
            number,
            query,
            wire,
            offset: 0,
            observed: false,
            deadline: admitted_at + timeout,
            epoch,
            authorized: false,
        })
    }
}

fn bind(
    registry: &Weak<Mutex<Registry>>,
    birth: &Birth,
    shared: &Shared,
    config: &Config,
    session_id: String,
    cwd: std::path::PathBuf,
    model: Option<String>,
    permission_mode: String,
) -> Result<(), Fault> {
    // Exact echo of reviewed native options, not an arbitrary caller-supplied ID.
    if cwd != config.cwd
        || model.as_deref() != Some(&config.model)
        || permission_mode != "request-review"
    {
        return Err(Fault::Identity);
    }
    let registry = registry.upgrade().ok_or(Fault::Closed)?;
    let mut registry = registry.lock().map_err(|_| Fault::Io)?;
    let entry = registry
        .owners
        .get(&birth.handle)
        .filter(|e| e.generation == birth.generation)
        .ok_or(Fault::WrongReference)?;
    if !std::ptr::eq(Arc::as_ptr(&entry.shared), shared)
        || registry.sessions.contains_key(&session_id)
    {
        return Err(Fault::Identity);
    }
    let mut state = shared.state.lock().map_err(|_| Fault::Io)?;
    if state.reference.is_some()
        || state.phase == Phase::Stopping
        || shared.stop.load(Ordering::Acquire)
    {
        return Err(Fault::Stopped);
    }
    let reference = Reference {
        handle: birth.handle.clone(),
        generation: birth.generation,
        provider: PROVIDER.into(),
        pet_id: PET.into(),
        session_id: session_id.clone(),
    };
    registry.sessions.insert(session_id, birth.handle.clone());
    {
        let mut gate = shared.gate.lock().map_err(|_| Fault::Io)?;
        let _ = gate.bind(reference.clone());
        let _ = gate.begin_turn(state.receipt.turn);
    }
    state.reference = Some(reference);
    state.phase = Phase::Active;
    state.metadata.display_title = Some("Sessão Gravity".into());
    state.metadata.cwd = Some(display_text(&config.cwd.to_string_lossy()));
    state.metadata.model = Some(display_text(&config.model));
    state.changed();
    Ok(())
}

fn observe(state: &mut Snapshot, turn: &mut Turn) -> Result<(), Fault> {
    if turn.offset != turn.wire.len() {
        return Err(Fault::Protocol);
    }
    let changed = !turn.observed || state.receipt.delivery != Delivery::Observed;
    if !turn.observed {
        history_room(state, turn.query.len(), 1)?;
        state.history.push(Message {
            role: Role::User,
            text: turn.query.clone(),
        });
        turn.observed = true;
    }
    state.receipt.delivery = Delivery::Observed;
    if changed {
        state.changed();
    }
    Ok(())
}

fn event(shared: &Shared, active: &mut Option<Turn>, event: NativeEvent) -> Result<(), Fault> {
    let mut state = shared.state.lock().map_err(|_| Fault::Io)?;
    if state.phase == Phase::Stopping || shared.stop.load(Ordering::Acquire) {
        return Err(Fault::Stopped);
    }
    let turn = active.as_mut().ok_or(Fault::Protocol)?;
    let id = match &event {
        NativeEvent::UserObserved { session_id }
        | NativeEvent::Delta { session_id, .. }
        | NativeEvent::Step { session_id }
        | NativeEvent::ToolProposed { session_id, .. }
        | NativeEvent::ToolFinished { session_id, .. }
        | NativeEvent::ToolUnverified { session_id, .. }
        | NativeEvent::Result { session_id, .. } => session_id,
        NativeEvent::Init { .. } => return Err(Fault::Protocol),
    };
    if state.reference.as_ref().is_none_or(|r| &r.session_id != id) {
        return Err(Fault::Identity);
    }
    if state.receipt.turn != turn.number || turn.offset != turn.wire.len() {
        return Err(Fault::Protocol);
    }
    match event {
        NativeEvent::UserObserved { .. } => observe(&mut state, turn)?,
        NativeEvent::Delta { text, .. } => {
            observe(&mut state, turn)?;
            if state.partial_reply.len() + text.len() > MAX_REPLY {
                return Err(Fault::Limit);
            }
            state.partial_reply.push_str(&text);
            if !text.is_empty() {
                state.changed();
            }
        }
        NativeEvent::Step { .. } => {}
        NativeEvent::ToolProposed {
            step, command, cwd, ..
        } => {
            let mut gate = shared.gate.lock().map_err(|_| Fault::Io)?;
            match NativeOperation::from_native(step, command, cwd).and_then(|op| gate.record(op)) {
                Ok(()) => {}
                Err(_) => gate.revoke(),
            }
        }
        NativeEvent::ToolFinished { step, .. } => {
            shared
                .gate
                .lock()
                .map_err(|_| Fault::Io)?
                .complete_step(step);
        }
        NativeEvent::ToolUnverified { step, .. } => {
            let mut gate = shared.gate.lock().map_err(|_| Fault::Io)?;
            if let Some(step) = step {
                gate.complete_step(step);
            } else {
                gate.revoke();
            }
        }
        NativeEvent::Result {
            status,
            response,
            num_turns,
            ..
        } => {
            shared.gate.lock().map_err(|_| Fault::Io)?.revoke();
            if num_turns.is_some_and(|count| count != turn.number) {
                return Err(Fault::Protocol);
            }
            if response.len() > MAX_REPLY {
                return Err(Fault::Limit);
            }
            observe(&mut state, turn)?;
            if !response.is_empty() {
                history_room(&state, response.len(), 1)?;
                state.history.push(Message {
                    role: Role::Assistant,
                    text: response,
                });
            }
            state.partial_reply.clear();
            state.receipt.delivery = Delivery::Completed;
            state.receipt.outcome = Some(match status {
                NativeStatus::Success => Outcome::Succeeded,
                NativeStatus::Error => Outcome::Failed,
                NativeStatus::Cancelled => Outcome::Cancelled,
                NativeStatus::Interrupted => Outcome::Interrupted,
                NativeStatus::Waiting => Outcome::Waiting,
            });
            state.changed();
            // Headless WAITING does not provide an answerable native input or
            // permission channel. Do not admit another turn over that state.
            if status == NativeStatus::Waiting {
                state.phase = Phase::Stopping;
                active.take();
                return Err(Fault::NotReady);
            }
            // Native completion is visible, but the owner remains busy until
            // the exact terminal receipt has committed and is revalidated.
            state.phase = Phase::Active;
            active.take();
        }
        NativeEvent::Init { .. } => unreachable!(),
    }
    Ok(())
}

struct FinishedTurn {
    number: u64,
    epoch: u64,
    deadline: Instant,
}

fn shutdown_deadline(registry: &Weak<Mutex<Registry>>) -> Option<Instant> {
    registry.upgrade().and_then(|registry| {
        registry
            .lock()
            .ok()
            .and_then(|registry| registry.shutdown_deadline)
    })
}

/// This is the actual native-effect lock, shared with Stop/shutdown/bind. Store
/// confirmations are checked inside the closure, immediately before the effect.
fn effect<T>(
    registry: &Weak<Mutex<Registry>>,
    birth: &Birth,
    shared: &Shared,
    turn: u64,
    epoch: u64,
    deadline: Instant,
    action: impl FnOnce(&mut Snapshot) -> Result<T, Fault>,
) -> Result<T, Fault> {
    let registry = registry.upgrade().ok_or(Fault::Stopped)?;
    let registry = registry.lock().map_err(|_| Fault::Io)?;
    let entry = registry
        .owners
        .get(&birth.handle)
        .filter(|entry| entry.generation == birth.generation)
        .ok_or(Fault::WrongReference)?;
    if registry.closing || !std::ptr::eq(Arc::as_ptr(&entry.shared), shared) {
        return Err(Fault::Stopped);
    }
    let mut state = shared.state.lock().map_err(|_| Fault::Io)?;
    if shared.stop.load(Ordering::Acquire)
        || shared.epoch.load(Ordering::Acquire) != epoch
        || state.receipt.turn != turn
        || matches!(state.phase, Phase::Stopping | Phase::Lost)
    {
        return Err(Fault::Stopped);
    }
    if Instant::now() >= deadline {
        return Err(Fault::Deadline);
    }
    if state.reference.as_ref().is_some_and(|reference| {
        reference.handle != birth.handle
            || reference.generation != birth.generation
            || reference.provider != PROVIDER
            || reference.pet_id != PET
    }) {
        return Err(Fault::Identity);
    }
    action(&mut state)
}

fn submit_checkpoint(cursor: &mut Cursor, purpose: Purpose, shared: &Shared) -> Result<(), Fault> {
    let state = shared.state.lock().map_err(|_| Fault::Io)?.clone();
    let epoch = shared.epoch.load(Ordering::Acquire);
    let result = cursor.submit(purpose, epoch, &state);
    publish_persistence(shared, cursor);
    result
}

fn worker(
    registry: Weak<Mutex<Registry>>,
    birth: Birth,
    shared: Arc<Shared>,
    config: Config,
    query: String,
    receiver: Receiver<Request>,
    backend: Backend,
    record_id: String,
    admitted_at: Instant,
    store_timeout: Duration,
) {
    let created_at = shared
        .state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .metadata
        .created_at_ms;
    let shared_id = Arc::as_ptr(&shared) as usize;
    let mut cursor = Cursor::new(
        backend,
        &birth,
        shared_id,
        record_id,
        created_at,
        store_timeout,
    );
    let mut transport: Option<Transport> = None;
    let mut active = None;
    let mut finished: Option<FinishedTurn> = None;
    let mut idle = false;
    let mut identified = false;
    let mut init_deadline: Option<Instant> = None;
    let result = (|| -> Result<(), Fault> {
        active = Some(Turn::new(1, query, config.turn_timeout, admitted_at, 1)?);
        submit_checkpoint(&mut cursor, Purpose::Birth, &shared)?;
        loop {
            let now = Instant::now();
            if shared.stop.load(Ordering::Acquire) || registry.strong_count() == 0 {
                return Err(Fault::Stopped);
            }
            if active.as_ref().is_some_and(|turn| now >= turn.deadline)
                || finished.as_ref().is_some_and(|turn| now >= turn.deadline)
                || init_deadline
                    .is_some_and(|limit| now >= limit && (!identified || cursor.needs_binding()))
            {
                return Err(Fault::Deadline);
            }
            let confirmation: Option<Confirmed> = cursor.poll(now, shutdown_deadline(&registry))?;
            publish_persistence(&shared, &cursor);
            if confirmation
                .as_ref()
                .is_some_and(|confirmation| confirmation.expired())
            {
                cursor.effect_timeout();
                return Err(Fault::JournalUnavailable);
            }

            if transport.is_none() {
                if let (Some(confirmation), Some(turn)) = (confirmation.as_ref(), active.as_ref()) {
                    if confirmation.expired() {
                        cursor.effect_timeout();
                        return Err(Fault::JournalUnavailable);
                    }
                    if confirmation.matches(
                        &birth,
                        shared_id,
                        turn.epoch,
                        turn.number,
                        Purpose::Birth,
                    ) {
                        let startup = effect(
                            &registry,
                            &birth,
                            &shared,
                            turn.number,
                            turn.epoch,
                            turn.deadline,
                            |state| {
                                if confirmation.expired() {
                                    return Err(Fault::JournalUnavailable);
                                }
                                if state.phase != Phase::Starting
                                    || state.reference.is_some()
                                    || !confirmation.matches(
                                        &birth,
                                        shared_id,
                                        turn.epoch,
                                        turn.number,
                                        Purpose::Birth,
                                    )
                                {
                                    return Err(Fault::Stopped);
                                }
                                Transport::start(&config)
                            },
                        )?;
                        init_deadline = Some(Instant::now() + config.init_timeout);
                        let (child, startup_fault) = startup;
                        {
                            let _state = shared.state.lock().map_err(|_| Fault::Io)?;
                            shared
                                .gate
                                .lock()
                                .map_err(|_| Fault::Io)?
                                .set_leader(child.leader());
                        }
                        transport = Some(child);
                        if let Some(fault) = startup_fault {
                            return Err(fault);
                        }
                    }
                }
                if transport.is_none() {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }
            }

            // Continue native reads during every disk flight. In particular,
            // process init/EOF before arming any first-byte/Ready confirmation.
            let child = transport.as_mut().expect("transport established above");
            let poll = child.poll()?;
            {
                let mut state = shared.state.lock().map_err(|_| Fault::Io)?;
                update_diagnostics(&mut state, child.diagnostics());
            }
            for line in poll.lines {
                match events::parse(&line)? {
                    NativeEvent::Init {
                        session_id,
                        cwd,
                        model,
                        permission_mode,
                    } => {
                        if identified {
                            return Err(Fault::Protocol);
                        }
                        let native_id = session_id.clone();
                        let native_cwd = cwd.clone();
                        let native_model = model.clone().ok_or(Fault::Identity)?;
                        bind(
                            &registry,
                            &birth,
                            &shared,
                            &config,
                            session_id,
                            cwd,
                            model,
                            permission_mode,
                        )?;
                        cursor.native(&native_id, &native_cwd, &native_model)?;
                        identified = true;
                        if let Some(turn) = active.as_mut() {
                            // Pause even partial writes until the exact verified
                            // identity has its ordered durable checkpoint.
                            turn.authorized = false;
                        }
                    }
                    native => {
                        if !identified {
                            return Err(Fault::Identity);
                        }
                        let terminal = if matches!(&native, NativeEvent::Result { .. }) {
                            active.as_ref().map(|turn| FinishedTurn {
                                number: turn.number,
                                epoch: turn.epoch,
                                deadline: turn.deadline,
                            })
                        } else {
                            None
                        };
                        let result = event(&shared, &mut active, native);
                        if let Some(terminal) = terminal {
                            if shared.state.lock().map_err(|_| Fault::Io)?.receipt.delivery
                                == Delivery::Completed
                            {
                                finished = Some(terminal);
                            }
                        }
                        result?;
                    }
                }
            }
            if poll.closed {
                return Err(Fault::Closed);
            }
            if confirmation
                .as_ref()
                .is_some_and(|confirmation| confirmation.expired())
            {
                cursor.effect_timeout();
                return Err(Fault::JournalUnavailable);
            }

            if let (Some(confirmation), Some(turn)) = (confirmation.as_ref(), finished.as_ref()) {
                if confirmation.matches(
                    &birth,
                    shared_id,
                    turn.epoch,
                    turn.number,
                    Purpose::Terminal,
                ) {
                    effect(
                        &registry,
                        &birth,
                        &shared,
                        turn.number,
                        turn.epoch,
                        turn.deadline,
                        |state| {
                            if confirmation.expired() {
                                return Err(Fault::JournalUnavailable);
                            }
                            if state.phase != Phase::Active
                                || state.receipt.delivery != Delivery::Completed
                                || state.receipt.outcome == Some(Outcome::Waiting)
                                || init_deadline.is_some_and(|limit| {
                                    Instant::now() >= limit
                                        && (!identified || cursor.needs_binding())
                                })
                                || !confirmation.matches(
                                    &birth,
                                    shared_id,
                                    turn.epoch,
                                    turn.number,
                                    Purpose::Terminal,
                                )
                                || !cursor.reference_matches(state.reference.as_ref())
                            {
                                return Err(Fault::Stopped);
                            }
                            state.receipt_persistence = cursor.status();
                            state.phase = Phase::Ready;
                            state.changed();
                            Ok(())
                        },
                    )?;
                    finished.take();
                    idle = true;
                }
            }

            if idle {
                match receiver.try_recv() {
                    Ok(request) => {
                        let turn = Turn::new(
                            request.turn,
                            request.query,
                            config.turn_timeout,
                            request.admitted_at,
                            request.epoch,
                        )?;
                        effect(
                            &registry,
                            &birth,
                            &shared,
                            turn.number,
                            turn.epoch,
                            turn.deadline,
                            |state| {
                                if state.phase != Phase::Active
                                    || state.receipt.delivery != Delivery::NotSent
                                {
                                    return Err(Fault::Stopped);
                                }
                                Ok(())
                            },
                        )?;
                        child.begin_turn();
                        active = Some(turn);
                        idle = false;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => return Err(Fault::Stopped),
                }
            }

            if !cursor.pending() {
                let purpose = if cursor.needs_binding() {
                    Some(Purpose::Binding)
                } else if let Some(turn) = active.as_ref() {
                    if cursor.durable_turn() != Some(turn.number) {
                        Some(Purpose::NewTurn)
                    } else if !cursor.possible(turn.number) {
                        Some(Purpose::Possible)
                    } else {
                        None
                    }
                } else if finished.is_some() {
                    Some(Purpose::Terminal)
                } else {
                    None
                };
                if let Some(purpose) = purpose {
                    submit_checkpoint(&mut cursor, purpose, &shared)?;
                }
            }

            if let Some(turn) = active.as_mut().filter(|turn| turn.offset < turn.wire.len()) {
                if cursor.possible(turn.number) && !cursor.needs_binding() {
                    effect(
                        &registry,
                        &birth,
                        &shared,
                        turn.number,
                        turn.epoch,
                        turn.deadline,
                        |state| {
                            if !matches!(state.phase, Phase::Starting | Phase::Active) {
                                return Err(Fault::Stopped);
                            }
                            if !cursor.reference_matches(state.reference.as_ref()) {
                                return Err(Fault::Identity);
                            }
                            if init_deadline.is_some_and(|limit| {
                                Instant::now() >= limit && (!identified || cursor.needs_binding())
                            }) {
                                return Err(Fault::Deadline);
                            }
                            if !turn.authorized {
                                if confirmation
                                    .as_ref()
                                    .is_some_and(|confirmation| confirmation.expired())
                                {
                                    return Err(Fault::JournalUnavailable);
                                }
                                let valid = confirmation.as_ref().is_some_and(|confirmation| {
                                    confirmation.matches(
                                        &birth,
                                        shared_id,
                                        turn.epoch,
                                        turn.number,
                                        Purpose::Possible,
                                    ) || confirmation.matches(
                                        &birth,
                                        shared_id,
                                        turn.epoch,
                                        turn.number,
                                        Purpose::Binding,
                                    )
                                });
                                if !valid {
                                    return Ok(());
                                }
                                // Consume this confirmation into this immutable
                                // turn only, before the first nonblocking attempt.
                                turn.authorized = true;
                            }
                            let n = child.write(&turn.wire[turn.offset..])?;
                            turn.offset += n;
                            if n > 0 {
                                state.receipt.delivery = if turn.offset == turn.wire.len() {
                                    Delivery::Sent
                                } else {
                                    Delivery::Writing
                                };
                                state.changed();
                            }
                            Ok(())
                        },
                    )?;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    })();
    finish(
        &registry,
        &shared,
        &mut transport,
        &mut cursor,
        result.err().unwrap_or(Fault::Closed),
    );
}

fn lost(shared: &Shared, fault: Fault, cleaned: bool, persistence: ReceiptPersistence) {
    let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
    if !shared.stop.swap(true, Ordering::AcqRel) {
        invalidate_epoch(shared);
    }
    shared
        .gate
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .revoke();
    let before = (
        state.phase,
        state.cleanup_confirmed,
        state.fault,
        state.receipt.clone(),
        state.receipt_persistence,
    );
    state.phase = if cleaned {
        Phase::Lost
    } else {
        Phase::Stopping
    };
    state.cleanup_confirmed = cleaned;
    state.receipt_persistence = persistence;
    state.fault = Some(if cleaned {
        fault
    } else {
        Fault::CleanupUnconfirmed
    });
    if state.receipt.delivery != Delivery::Completed {
        if state.receipt.delivery != Delivery::NotSent {
            state.receipt.delivery = Delivery::Unknown;
        }
        state.receipt.outcome = Some(Outcome::Lost);
    }
    if before
        != (
            state.phase,
            state.cleanup_confirmed,
            state.fault,
            state.receipt.clone(),
            state.receipt_persistence,
        )
    {
        state.changed();
    }
}

fn update_diagnostics(state: &mut Snapshot, diagnostics: Diagnostics) {
    if state.diagnostics != diagnostics {
        state.diagnostics = diagnostics;
        state.changed();
    }
}

fn finish(
    registry: &Weak<Mutex<Registry>>,
    shared: &Shared,
    transport: &mut Option<Transport>,
    cursor: &mut Cursor,
    fault: Fault,
) {
    cursor.begin_stopping();
    lost(shared, fault, false, cursor.status());
    loop {
        // This path never waits on the disk actor. Even a stuck fsync cannot
        // delay our attempt to terminate and reap this exact native group.
        let cleaned = transport
            .as_mut()
            .is_none_or(|transport| transport.cleanup());
        if let Some(transport) = transport.as_ref() {
            let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
            update_diagnostics(&mut state, transport.diagnostics());
        }
        let _ = cursor.poll(Instant::now(), shutdown_deadline(registry));
        if cleaned {
            cursor.begin_cleanup(Instant::now(), shutdown_deadline(registry));
        }
        lost(shared, fault, cleaned, cursor.status());
        if cleaned {
            break;
        }
        // Keep custody and busy state if the OS has not confirmed cleanup.
        // No replacement, next request, release or PID adoption is attempted.
        std::thread::sleep(Duration::from_millis(100));
    }
    // Dropping a fully cleaned transport does not touch disk, and no native
    // process/capability is retained during historical reconciliation.
    transport.take();
    loop {
        let _ = cursor.poll(Instant::now(), shutdown_deadline(registry));
        publish_persistence(shared, cursor);
        if cursor.final_durable() || cursor.abandoned() {
            return;
        }
        if !cursor.pending() {
            if !cursor.has_birth() {
                cursor.abandon(Fault::JournalUnavailable);
            } else {
                let state = shared
                    .state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone();
                let purpose = if cursor.durable_turn() != Some(state.receipt.turn) {
                    // An admitted next turn can be stopped before its worker
                    // receives the request. Record that NotSent turn before Lost
                    // without starting/reviving native execution.
                    Purpose::NewTurn
                } else if cursor.needs_binding() {
                    Purpose::Binding
                } else if state.receipt.outcome == Some(Outcome::Waiting)
                    && !cursor.terminal_durable(&state)
                {
                    Purpose::Terminal
                } else {
                    Purpose::Lost
                };
                let _ = cursor.submit(purpose, shared.epoch.load(Ordering::Acquire), &state);
            }
            publish_persistence(shared, cursor);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
