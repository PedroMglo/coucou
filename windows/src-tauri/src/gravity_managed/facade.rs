//! Private application boundary over the one app-owned Gravity manager.
//! Backend configuration is captured once, never accepted as a request DTO.
//! This module registers no command, invokes no alternate executor and exposes
//! no gate, permission policy, process handle or terminal attachment operation.
use super::owner::Manager;
use super::types::{self, Birth, Config, Fault, Reference, Snapshot, PET, PROVIDER};
use crate::session_chat::types::{Origin, Target};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;
use std::time::Duration;

fn valid_handle(handle: &str) -> bool {
    !handle.is_empty() && handle.len() <= 256 && !handle.chars().any(char::is_control)
}

fn deserialize_handle<'de, D: Deserializer<'de>>(input: D) -> Result<String, D::Error> {
    let handle = String::deserialize(input)?;
    if valid_handle(&handle) {
        Ok(handle)
    } else {
        Err(serde::de::Error::custom("invalid managed handle"))
    }
}

fn deserialize_generation<'de, D: Deserializer<'de>>(input: D) -> Result<u64, D::Error> {
    let generation = u64::deserialize(input)?;
    if generation > 0 {
        Ok(generation)
    } else {
        Err(serde::de::Error::custom("invalid managed generation"))
    }
}

fn valid_target(target: &Target) -> bool {
    target.validate_external(PROVIDER, PET).is_ok() && types::valid_uuid(&target.session_id)
}

fn deserialize_target<'de, D: Deserializer<'de>>(input: D) -> Result<Target, D::Error> {
    let target = Target::deserialize(input)?;
    if valid_target(&target) {
        Ok(target)
    } else {
        Err(serde::de::Error::custom("invalid managed Gravity target"))
    }
}

fn deserialize_pet<'de, D: Deserializer<'de>>(input: D) -> Result<String, D::Error> {
    let pet = String::deserialize(input)?;
    if pet == PET {
        Ok(pet)
    } else {
        Err(serde::de::Error::custom("invalid Gravity pet"))
    }
}

fn deserialize_first_text<'de, D: Deserializer<'de>>(input: D) -> Result<String, D::Error> {
    let text = String::deserialize(input)?;
    if types::validate_query(&text).is_ok() {
        Ok(text)
    } else {
        Err(serde::de::Error::custom("invalid first text"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BirthDto {
    #[serde(deserialize_with = "deserialize_handle")]
    pub(crate) handle: String,
    #[serde(deserialize_with = "deserialize_generation")]
    pub(crate) generation: u64,
}

impl BirthDto {
    fn from_core(birth: &Birth) -> Self {
        Self {
            handle: birth.handle.clone(),
            generation: birth.generation,
        }
    }

    fn from_reference(reference: &Reference) -> Self {
        Self {
            handle: reference.handle.clone(),
            generation: reference.generation,
        }
    }

    fn core(&self) -> Result<Birth, ErrorDto> {
        if !valid_handle(&self.handle) || self.generation == 0 {
            return Err(Fault::WrongReference.into());
        }
        Ok(Birth {
            handle: self.handle.clone(),
            generation: self.generation,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReferenceDto {
    #[serde(deserialize_with = "deserialize_target")]
    pub(crate) target: Target,
    #[serde(deserialize_with = "deserialize_handle")]
    pub(crate) handle: String,
    #[serde(deserialize_with = "deserialize_generation")]
    pub(crate) generation: u64,
}

impl ReferenceDto {
    fn from_core(reference: &Reference) -> Self {
        Self {
            target: Target {
                pet_id: reference.pet_id.clone(),
                provider: reference.provider.clone(),
                origin: Origin::External,
                session_id: reference.session_id.clone(),
                conversation_id: None,
            },
            handle: reference.handle.clone(),
            generation: reference.generation,
        }
    }

    fn core(&self) -> Result<Reference, ErrorDto> {
        if !valid_handle(&self.handle) || self.generation == 0 || !valid_target(&self.target) {
            return Err(Fault::WrongReference.into());
        }
        Ok(Reference {
            handle: self.handle.clone(),
            generation: self.generation,
            provider: self.target.provider.clone(),
            pet_id: self.target.pet_id.clone(),
            session_id: self.target.session_id.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Selector {
    Birth { birth: BirthDto },
    Session { reference: ReferenceDto },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateRequest {
    #[serde(deserialize_with = "deserialize_pet")]
    pub(crate) pet_id: String,
    #[serde(deserialize_with = "deserialize_first_text")]
    pub(crate) first_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ErrorDto {
    pub(crate) code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) birth: Option<BirthDto>,
}

impl From<Fault> for ErrorDto {
    fn from(fault: Fault) -> Self {
        Self {
            code: fault.code().into(),
            birth: None,
        }
    }
}

impl ErrorDto {
    fn with_birth(fault: Fault, birth: &BirthDto) -> Self {
        Self {
            code: fault.code().into(),
            birth: Some(birth.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum StateDto {
    Starting,
    Active,
    Ready,
    Stopping,
    Lost,
}

impl From<types::Phase> for StateDto {
    fn from(phase: types::Phase) -> Self {
        match phase {
            types::Phase::Starting => Self::Starting,
            types::Phase::Active => Self::Active,
            types::Phase::Ready => Self::Ready,
            types::Phase::Stopping => Self::Stopping,
            types::Phase::Lost => Self::Lost,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DeliveryDto {
    NotSent,
    Writing,
    Sent,
    Observed,
    Completed,
    Unknown,
}

impl From<types::Delivery> for DeliveryDto {
    fn from(delivery: types::Delivery) -> Self {
        match delivery {
            types::Delivery::NotSent => Self::NotSent,
            types::Delivery::Writing => Self::Writing,
            types::Delivery::Sent => Self::Sent,
            types::Delivery::Observed => Self::Observed,
            types::Delivery::Completed => Self::Completed,
            types::Delivery::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OutcomeDto {
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
    Waiting,
    Lost,
}

impl From<types::Outcome> for OutcomeDto {
    fn from(outcome: types::Outcome) -> Self {
        match outcome {
            types::Outcome::Succeeded => Self::Succeeded,
            types::Outcome::Failed => Self::Failed,
            types::Outcome::Cancelled => Self::Cancelled,
            types::Outcome::Interrupted => Self::Interrupted,
            types::Outcome::Waiting => Self::Waiting,
            types::Outcome::Lost => Self::Lost,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReceiptDto {
    /// This is owner-local admission order, never a native turn identifier.
    pub(crate) local_turn: u64,
    pub(crate) delivery: DeliveryDto,
    pub(crate) outcome: Option<OutcomeDto>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RoleDto {
    User,
    Assistant,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageDto {
    pub(crate) role: RoleDto,
    pub(crate) content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetadataDto {
    /// Unix epoch milliseconds from the owner, absent when its clock is invalid.
    pub(crate) created_at: Option<u64>,
    pub(crate) updated_at: Option<u64>,
    pub(crate) revision: u64,
    pub(crate) display_title: Option<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CapabilitiesDto {
    pub(crate) history: bool,
    pub(crate) send: bool,
    pub(crate) stop_owner: bool,
    pub(crate) interrupt_turn: bool,
    pub(crate) approvals: bool,
    pub(crate) resume: bool,
    pub(crate) terminal_handoff: bool,
    pub(crate) live_attach: bool,
    pub(crate) forget_lost: bool,
    pub(crate) survives_view_close: bool,
    pub(crate) survives_app_exit: bool,
}

impl From<types::ViewCapabilities> for CapabilitiesDto {
    fn from(capabilities: types::ViewCapabilities) -> Self {
        Self {
            history: capabilities.history,
            send: capabilities.send,
            stop_owner: capabilities.stop_owner,
            interrupt_turn: capabilities.interrupt_turn,
            approvals: capabilities.approvals,
            resume: capabilities.resume,
            terminal_handoff: capabilities.terminal_handoff,
            live_attach: capabilities.live_attach,
            forget_lost: capabilities.forget_lost,
            survives_view_close: capabilities.survives_view_close,
            survives_app_exit: capabilities.survives_app_exit,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReceiptPersistenceDto {
    pub(crate) state: String,
    pub(crate) revision: Option<u64>,
    pub(crate) error_code: Option<String>,
}

impl From<types::ReceiptPersistence> for ReceiptPersistenceDto {
    fn from(receipt: types::ReceiptPersistence) -> Self {
        Self {
            state: match receipt.state {
                types::ReceiptPersistenceState::Pending => "pending",
                types::ReceiptPersistenceState::Durable => "durable",
                types::ReceiptPersistenceState::Unavailable => "unavailable",
                types::ReceiptPersistenceState::Unknown => "unknown",
                #[cfg(test)]
                types::ReceiptPersistenceState::Memory => "memory",
            }
            .into(),
            revision: receipt.revision,
            error_code: receipt.error.map(|fault| fault.code().into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotDto {
    pub(crate) birth: BirthDto,
    pub(crate) reference: Option<ReferenceDto>,
    pub(crate) state: StateDto,
    pub(crate) receipt: ReceiptDto,
    pub(crate) receipt_persistence: ReceiptPersistenceDto,
    pub(crate) messages: Vec<MessageDto>,
    pub(crate) partial_reply: String,
    pub(crate) metadata: MetadataDto,
    pub(crate) capabilities: CapabilitiesDto,
    pub(crate) diagnostic_codes: Vec<String>,
    pub(crate) cleanup_confirmed: bool,
    pub(crate) error_code: Option<String>,
}

impl SnapshotDto {
    fn project(birth: BirthDto, snapshot: Snapshot) -> Result<Self, ErrorDto> {
        if snapshot.reference.as_ref().is_some_and(|reference| {
            reference.handle != birth.handle
                || reference.generation != birth.generation
                || reference.provider != PROVIDER
                || reference.pet_id != PET
                || !types::valid_uuid(&reference.session_id)
        }) {
            return Err(ErrorDto::with_birth(Fault::Identity, &birth));
        }
        let capabilities = snapshot.capabilities().into();
        let reference = snapshot.reference.as_ref().map(ReferenceDto::from_core);
        let mut diagnostic_codes = Vec::new();
        if snapshot.diagnostics.seen {
            diagnostic_codes.push("nativeDiagnostics".into());
        }
        if snapshot.diagnostics.permission_limited {
            diagnostic_codes.push("permissionLimited".into());
        }
        if snapshot.diagnostics.authentication_required {
            diagnostic_codes.push("authenticationRequired".into());
        }
        Ok(Self {
            birth,
            reference,
            state: snapshot.phase.into(),
            receipt: ReceiptDto {
                local_turn: snapshot.receipt.turn,
                delivery: snapshot.receipt.delivery.into(),
                outcome: snapshot.receipt.outcome.map(Into::into),
            },
            receipt_persistence: snapshot.receipt_persistence.into(),
            messages: snapshot
                .history
                .into_iter()
                .map(|message| MessageDto {
                    role: match message.role {
                        types::Role::User => RoleDto::User,
                        types::Role::Assistant => RoleDto::Assistant,
                    },
                    content: message.text,
                })
                .collect(),
            partial_reply: snapshot.partial_reply,
            metadata: MetadataDto {
                created_at: snapshot.metadata.created_at_ms,
                updated_at: snapshot.metadata.updated_at_ms,
                revision: snapshot.metadata.revision,
                display_title: snapshot.metadata.display_title,
                cwd: snapshot.metadata.cwd,
                model: snapshot.metadata.model,
            },
            capabilities,
            diagnostic_codes,
            cleanup_confirmed: snapshot.cleanup_confirmed,
            error_code: snapshot.fault.map(|fault| fault.code().into()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendAck {
    pub(crate) reference: ReferenceDto,
    pub(crate) local_turn: u64,
    /// Queue admission only; neither a Sent nor a Done acknowledgement.
    pub(crate) admitted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ForgetAck {
    pub(crate) selector: Selector,
    pub(crate) forgotten: bool,
}

#[derive(Clone)]
pub(crate) struct Facade {
    manager: Manager,
    config: Arc<Config>,
}

impl std::fmt::Debug for Facade {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.debug_struct("Facade").finish_non_exhaustive()
    }
}

impl Facade {
    pub(crate) fn new(manager: Manager, config: Config) -> Result<Self, ErrorDto> {
        config.validate().map_err(ErrorDto::from)?;
        Ok(Self {
            manager,
            config: Arc::new(config),
        })
    }

    pub(crate) fn create(&self, request: CreateRequest) -> Result<SnapshotDto, ErrorDto> {
        if request.pet_id != PET {
            return Err(Fault::WrongReference.into());
        }
        let birth = self
            .manager
            .create((*self.config).clone(), request.first_text)
            .map_err(ErrorDto::from)?;
        let birth_dto = BirthDto::from_core(&birth);
        let snapshot = self
            .manager
            .inspect_birth(&birth)
            .map_err(|fault| ErrorDto::with_birth(fault, &birth_dto))?;
        // After admission, even projection/read failure retains the birth token.
        SnapshotDto::project(birth_dto, snapshot)
    }

    fn read(&self, selector: &Selector) -> Result<(BirthDto, Snapshot), ErrorDto> {
        match selector {
            Selector::Birth { birth } => {
                let core = birth.core()?;
                let snapshot = self
                    .manager
                    .inspect_birth(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, birth))?;
                Ok((birth.clone(), snapshot))
            }
            Selector::Session { reference } => {
                let core = reference.core()?;
                let birth = BirthDto::from_reference(&core);
                let snapshot = self
                    .manager
                    .inspect(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
                Ok((birth, snapshot))
            }
        }
    }

    pub(crate) fn inspect(&self, selector: Selector) -> Result<SnapshotDto, ErrorDto> {
        let (birth, snapshot) = self.read(&selector)?;
        SnapshotDto::project(birth, snapshot)
    }

    pub(crate) fn list(&self) -> Result<Vec<SnapshotDto>, ErrorDto> {
        let mut births = self.manager.list_births().map_err(ErrorDto::from)?;
        // Local generation orders this manager's births. Clock metadata and
        // polling never order, mutate or recreate a native session.
        births.sort_by(|a, b| {
            b.generation
                .cmp(&a.generation)
                .then_with(|| a.handle.cmp(&b.handle))
        });
        let mut snapshots = Vec::with_capacity(births.len());
        for birth in births {
            let dto = BirthDto::from_core(&birth);
            let snapshot = match self.manager.inspect_birth(&birth) {
                Ok(snapshot) => snapshot,
                Err(Fault::WrongReference) => continue, // Concurrent explicit forget.
                Err(fault) => return Err(ErrorDto::with_birth(fault, &dto)),
            };
            snapshots.push(SnapshotDto::project(dto, snapshot)?);
        }
        Ok(snapshots)
    }

    /// Complete public RAM history since this birth; never reads a CLI catalog,
    /// restores a session, replays a prompt or promises a durable journal.
    pub(crate) fn history(&self, selector: Selector) -> Result<SnapshotDto, ErrorDto> {
        self.inspect(selector)
    }

    /// Historical receipt metadata only; never creates live selectors or text.
    pub(crate) fn recovery(&self) -> Vec<super::journal::RecoveryRecord> {
        self.manager.recovery()
    }

    pub(crate) fn send(&self, reference: ReferenceDto, query: String) -> Result<SendAck, ErrorDto> {
        let core = reference.core()?;
        let birth = BirthDto::from_reference(&core);
        let local_turn = self
            .manager
            .send(&core, query)
            .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
        Ok(SendAck {
            reference,
            local_turn,
            admitted: true,
        })
    }

    pub(crate) fn stop_owner(&self, selector: Selector) -> Result<SnapshotDto, ErrorDto> {
        let (birth, snapshot) = match selector {
            Selector::Birth { birth } => {
                let core = birth.core()?;
                // The core atomically requires reference=None AND requests Stop.
                // Never inspect then stop_birth: init could bind between calls.
                self.manager
                    .stop_unconfirmed_birth(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
                let snapshot = self
                    .manager
                    .inspect_birth(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
                (birth, snapshot)
            }
            Selector::Session { reference } => {
                let core = reference.core()?;
                let birth = BirthDto::from_reference(&core);
                self.manager
                    .stop(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
                let snapshot = self
                    .manager
                    .inspect(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
                (birth, snapshot)
            }
        };
        // Stopping is a valid acknowledgement. Only core cleanup can claim Lost
        // and cleanupConfirmed=true; a stopped owner never becomes Ready here.
        SnapshotDto::project(birth, snapshot)
    }

    pub(crate) fn forget_lost(&self, selector: Selector) -> Result<ForgetAck, ErrorDto> {
        match &selector {
            Selector::Birth { birth } => {
                let core = birth.core()?;
                self.manager
                    .forget_lost_birth(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, birth))?;
            }
            Selector::Session { reference } => {
                let core = reference.core()?;
                let birth = BirthDto::from_reference(&core);
                self.manager
                    .forget_lost(&core)
                    .map_err(|fault| ErrorDto::with_birth(fault, &birth))?;
            }
        }
        Ok(ForgetAck {
            selector,
            forgotten: true,
        })
    }

    /// Internal app lifecycle only. Duration is not part of a deserialized DTO.
    pub(crate) fn shutdown(&self, timeout: Duration) -> Result<(), ErrorDto> {
        self.manager.shutdown(timeout).map_err(ErrorDto::from)
    }
}
