//! Pure navigation lookup. No transport, subscription, resume, activity inference
//! or filesystem/configuration mutation. Inputs must be captured by the caller.
use serde::{Deserialize, Serialize};

use crate::pet_observability::core::{Coverage, InventoryInput, LibraryCapture,
    OpenTarget, Origin, Ownership, PetBinding, SessionRow};

const SCHEMA_VERSION: u32 = 1;
const MAX_AGE: u64 = 15;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LookupRequest {
    pub provider: String,
    pub native_session_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LookupState { Resolved, Ambiguous, Unavailable, NotFound, NotSupported, InvalidRequest }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LookupEvidence {
    pub scope: String,
    pub coverage: Coverage,
    pub observed_at: Option<u64>,
    pub stale: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LookupResult {
    pub schema_version: u32,
    pub generated_at: u64,
    pub request: LookupRequest,
    pub state: LookupState,
    pub reason: Option<String>,
    /// Advisory delay for a manually repeated metadata lookup, never execution.
    pub retry_after_ms: Option<u64>,
    pub owner: Option<String>,
    pub conversation_id: Option<String>,
    pub origin: Origin,
    pub open: Option<OpenTarget>,
    pub library: LookupEvidence,
    pub native: LookupEvidence,
}

fn valid_provider(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|byte|
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-'))
}
fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|byte|
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}
fn valid_conversation_id(value: &str) -> bool {
    // Same pure grammar as assistant_memory::valid_id, without storage access.
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|byte|
        byte.is_ascii_alphanumeric() || byte == b'-')
}
fn uuid(value: &str) -> bool {
    value.len() == 36 && value.bytes().enumerate().all(|(index, byte)|
        if matches!(index, 8 | 13 | 18 | 23) { byte == b'-' } else { byte.is_ascii_hexdigit() })
}

/// Codex uses Thread.id, not the sessionId shared by its subagent tree.
/// No case folding, prefixes, whitespace trimming or name-based lookup.
pub fn valid_request(request: &LookupRequest) -> bool {
    valid_provider(&request.provider) && valid_id(&request.native_session_id) &&
        (!matches!(request.provider.as_str(), "codex" | "copilot") || uuid(&request.native_session_id))
}

fn relevant(hint: &Ownership, request: &LookupRequest) -> bool {
    hint.session_id == request.native_session_id && hint.backend.as_deref()
        .is_none_or(|backend| backend == request.provider)
}

/// Only the full ownership index decides whether a native lookup is needed.
/// A true result is not proof that the provider has a supported native source.
pub fn needs_native(request: &LookupRequest, library: &LibraryCapture) -> bool {
    valid_request(request) && library.ownership_complete &&
        !library.ownership.iter().any(|hint| relevant(hint, request))
}

fn canonical(provider: &str) -> Option<(&'static str, &'static str)> {
    match provider {
        "codex" => Some(("agent_codex", "codexDaemon")),
        "copilot" => Some(("agent_copilot", "copilotLiveEndpoints")),
        _ => None,
    }
}
fn fresh(observed_at: Option<u64>, now: u64, reason: Option<&str>) -> bool {
    reason != Some("refreshFailed") && observed_at.is_some_and(|at| at <= now && now - at <= MAX_AGE)
}
fn library_evidence(library: &LibraryCapture, now: u64) -> LookupEvidence {
    let complete = library.ownership_complete;
    LookupEvidence {
        scope: "mochiLibrary".into(),
        coverage: if complete { Coverage::Complete } else if library.ownership.is_empty() {
            Coverage::Unavailable
        } else { Coverage::Partial },
        observed_at: complete.then_some(now), stale: !complete,
        reason: (!complete).then(|| "libraryOwnershipIncomplete".into()),
    }
}
fn native_evidence(request: &LookupRequest, input: Option<&InventoryInput>, now: u64) -> LookupEvidence {
    if let Some(input) = input {
        return LookupEvidence { scope: input.scope.clone(), coverage: input.coverage,
            observed_at: input.observed_at,
            stale: !fresh(input.observed_at, now, input.reason.as_deref()), reason: input.reason.clone() };
    }
    let source = canonical(&request.provider);
    LookupEvidence { scope: source.map_or("providerGlobal", |(_, scope)| scope).into(),
        coverage: if source.is_some() { Coverage::Unavailable } else { Coverage::NotSupported },
        observed_at: None, stale: true, reason: Some(if source.is_some() {
            "nativeInventoryUnavailable"
        } else { "noProviderSessionInventory" }.into()) }
}

struct Decision {
    state: LookupState,
    reason: Option<String>,
    owner: Option<String>,
    conversation_id: Option<String>,
    origin: Origin,
    open: Option<OpenTarget>,
}
impl Decision {
    fn new(state: LookupState, reason: &'static str) -> Self {
        Self { state, reason: Some(reason.into()), owner: None,
            conversation_id: None, origin: Origin::Unknown, open: None }
    }
    fn mochi(state: LookupState, reason: &'static str, hint: &Ownership) -> Self {
        Self { state, reason: Some(reason.into()), owner: Some(hint.owner.clone()),
            conversation_id: Some(hint.conversation_id.clone()), origin: Origin::Mochi, open: None }
    }
}

fn same_binding(a: &PetBinding, b: &PetBinding) -> bool {
    a.pet_id == b.pet_id && a.provider == b.provider && a.owner == b.owner && a.enabled == b.enabled
}
fn binding_conflict(binding: &PetBinding, bindings: &[PetBinding]) -> bool {
    bindings.iter().any(|other| other.pet_id == binding.pet_id && !same_binding(other, binding))
}

/// None means there is no relevant library hint and native fallback is allowed.
fn from_library(request: &LookupRequest, bindings: &[PetBinding], library: &LibraryCapture) -> Option<Decision> {
    if !library.ownership_complete {
        return Some(Decision::new(LookupState::Unavailable, "libraryOwnershipIncomplete"));
    }
    let mut first: Option<&Ownership> = None;
    let mut hinted_pet: Option<&str> = None;
    let mut legacy = false;
    let mut owners_conflict = false;
    let mut pets_conflict = false;
    for hint in library.ownership.iter().filter(|hint| relevant(hint, request)) {
        legacy |= hint.backend.is_none();
        if let Some(previous) = first {
            owners_conflict |= previous.owner != hint.owner || previous.conversation_id != hint.conversation_id;
        } else { first = Some(hint); }
        if let Some(pet) = hint.pet_id.as_deref() {
            if let Some(previous) = hinted_pet { pets_conflict |= previous != pet; }
            else { hinted_pet = Some(pet); }
        }
    }
    let hint = first?;
    if legacy { return Some(Decision::new(LookupState::Ambiguous, "unknownLibraryBackend")); }
    if owners_conflict { return Some(Decision::new(LookupState::Ambiguous, "multipleLibraryOwners")); }
    if !valid_provider(&hint.owner) || !valid_conversation_id(&hint.conversation_id) {
        return Some(Decision::new(LookupState::Unavailable, "invalidLibraryIdentity"));
    }
    let unavailable = |state, reason| Some(Decision::mochi(state, reason, hint));
    if pets_conflict { return unavailable(LookupState::Ambiguous, "conflictingLibraryPets"); }
    let mut selected: Option<&PetBinding> = None;
    for binding in bindings.iter().filter(|binding|
        binding.provider == request.provider && binding.owner == hint.owner) {
        if !valid_id(&binding.pet_id) {
            return unavailable(LookupState::Unavailable, "invalidPetBinding");
        }
        if selected.is_some_and(|previous| !same_binding(previous, binding)) {
            return unavailable(LookupState::Ambiguous, "multiplePetBindings");
        }
        selected = Some(binding);
    }
    let Some(binding) = selected else {
        // Retired/removed/unsupported owners remain Mochi-owned. Never turn
        // their exact native ID into an External route because no pet is shown.
        return unavailable(LookupState::Unavailable, "petBindingUnavailable");
    };
    if binding_conflict(binding, bindings) || hinted_pet.is_some_and(|pet| pet != binding.pet_id) {
        return unavailable(LookupState::Ambiguous, "conflictingPetBinding");
    }
    if !binding.enabled { return unavailable(LookupState::Unavailable, "petDisabled"); }
    Some(Decision { state: LookupState::Resolved, reason: None, owner: Some(hint.owner.clone()),
        conversation_id: Some(hint.conversation_id.clone()), origin: Origin::Mochi,
        open: Some(OpenTarget::Mochi { pet_id: binding.pet_id.clone(), provider: hint.owner.clone(),
            conversation_id: hint.conversation_id.clone() }) })
}

fn from_native(request: &LookupRequest, bindings: &[PetBinding], native: Option<&InventoryInput>, evidence: &LookupEvidence) -> Decision {
    let Some((pet_id, scope)) = canonical(&request.provider) else {
        return Decision::new(LookupState::NotSupported, "noProviderSessionInventory");
    };
    let Some(input) = native else {
        return Decision::new(LookupState::Unavailable, "nativeInventoryUnavailable");
    };
    if input.scope != scope { return Decision::new(LookupState::Unavailable, "nativeScopeMismatch"); }
    match input.coverage {
        Coverage::Unavailable => return Decision::new(LookupState::Unavailable, "nativeInventoryUnavailable"),
        Coverage::NotSupported => return Decision::new(LookupState::NotSupported, "noProviderSessionInventory"),
        Coverage::Complete | Coverage::Partial => {},
    }
    // Revalidate the final configuration before either positive navigation or
    // an authoritative scoped negative. A revoked pet is not a NotFound proof.
    let mut selected_binding: Option<&PetBinding> = None;
    for binding in bindings.iter().filter(|binding| binding.pet_id == pet_id) {
        if selected_binding.is_some_and(|previous| !same_binding(previous, binding)) {
            return Decision::new(LookupState::Ambiguous, "conflictingPetBinding");
        }
        selected_binding = Some(binding);
    }
    let Some(binding) = selected_binding else {
        return Decision::new(LookupState::Unavailable, "petBindingUnavailable");
    };
    if binding.provider != request.provider || binding.owner != request.provider {
        return Decision::new(LookupState::Unavailable, "nativePetBindingMismatch");
    }
    if !binding.enabled { return Decision::new(LookupState::Unavailable, "petDisabled"); }
    // Malformed identities cannot justify an authoritative negative lookup.
    if input.rows.iter().any(|row| row.provider != request.provider ||
        row.session_id.as_deref().is_none_or(|id| !uuid(id))) {
        return Decision::new(LookupState::Unavailable, "invalidNativeInventory");
    }
    let mut selected: Option<&SessionRow> = None;
    for row in input.rows.iter().filter(|row| row.session_id.as_deref() == Some(request.native_session_id.as_str())) {
        if selected.is_some_and(|previous| previous != row) {
            return Decision::new(LookupState::Ambiguous, "conflictingNativeRows");
        }
        selected = Some(row);
    }
    let Some(row) = selected else {
        return if input.coverage == Coverage::Complete && !evidence.stale {
            Decision::new(LookupState::NotFound, "notFoundInNativeScope")
        } else { Decision::new(LookupState::Unavailable, "nativeAbsenceUnconfirmed") };
    };
    if input.rows.iter().any(|other| other.key == row.key && other != row) {
        return Decision::new(LookupState::Ambiguous, "conflictingNativeRows");
    }
    let valid_open = matches!(&row.open, Some(OpenTarget::External { pet_id: target_pet,
        provider, session_id }) if target_pet == pet_id && provider == &request.provider &&
        session_id == &request.native_session_id && uuid(session_id));
    if !valid_open || row.origin != Origin::External || row.pet_id.as_deref() != Some(pet_id) ||
        row.conversation_id.is_some() {
        return Decision::new(LookupState::Unavailable, "nativeOpenUnavailable");
    }
    Decision { state: LookupState::Resolved, reason: None, owner: None, conversation_id: None,
        origin: Origin::External, open: row.open.clone() }
}

pub fn resolve(request: LookupRequest, bindings: &[PetBinding], library: &LibraryCapture,
    native: Option<&InventoryInput>, now: u64) -> LookupResult {
    let library_evidence = library_evidence(library, now);
    let native_evidence = native_evidence(&request, native, now);
    let decision = if !valid_request(&request) {
        Decision::new(LookupState::InvalidRequest, "invalidRequest")
    } else {
        from_library(&request, bindings, library)
            .unwrap_or_else(|| from_native(&request, bindings, native, &native_evidence))
    };
    LookupResult { schema_version: SCHEMA_VERSION, generated_at: now, request,
        state: decision.state, reason: decision.reason, retry_after_ms: None, owner: decision.owner,
        conversation_id: decision.conversation_id, origin: decision.origin, open: decision.open,
        library: library_evidence, native: native_evidence }
}
