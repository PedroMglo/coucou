//! Shared interactive operation contract. Catalogs and checked arguments are
//! data, never a goal grant or a substitute for native caller authorization.
mod input;
mod catalog;
pub(crate) mod roster;
pub(crate) use input::{Arguments, Music, encode};
use crate::pet_config::Configuration;
use serde::Serialize;
use serde_json::Value;
pub(crate) const NAMES: &[&str] = &["pills", "music", "service_info", "open_app", "prepare_email"];
pub(crate) const MAX_INPUT_BYTES: usize = 8000;
pub(crate) const MAX_RESULT_BYTES: usize = 16_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation { Roster, Music, Snapshot, OpenApp, Email }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Effect { ReadOnly, Configuration, Media, Launch, Compose }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Delivery { NotDispatched, Returned, Accepted, Dispatched, Partial, Uncertain }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Failure { UnknownOperation, InvalidArguments, UnknownPet, PetUnavailable, PermissionDenied, ConfigurationUnavailable, ScopeChanged, Cancelled }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Descriptor {
    id: &'static str, name: &'static str, version: u32, effect: Effect,
    resource: &'static str, existing_permission: Option<&'static str>,
    max_input_bytes: usize, max_result_bytes: usize,
    cancellation: &'static str, autonomous_execution_available: bool,
}
impl Operation {
    fn named(name: &str) -> Option<Self> {
        Some(match name { "pills"=>Self::Roster, "music"=>Self::Music, "service_info"=>Self::Snapshot, "open_app"=>Self::OpenApp, "prepare_email"=>Self::Email, _=>return None })
    }
    fn descriptor(self) -> Descriptor {
        let (id,name,effect,resource,grant) = match self {
            Self::Roster => ("pets.roster","pills",Effect::Configuration,"pet-roster",None),
            Self::Music => ("spotify.control","music",Effect::Media,"spotify-player",Some("music")),
            Self::Snapshot => ("services.snapshot","service_info",Effect::ReadOnly,"service-cache",None),
            Self::OpenApp => ("desktop.launch","open_app",Effect::Launch,"desktop-launcher",Some("openApps")),
            Self::Email => ("mail.prepare","prepare_email",Effect::Compose,"mail-composer",Some("emailDrafts")),
        };
        Descriptor { id,name,version:1,effect,resource,existing_permission:grant,max_input_bytes:MAX_INPUT_BYTES,max_result_bytes:MAX_RESULT_BYTES,cancellation:"beforeDispatch",autonomous_execution_available:false }
    }
}
pub(crate) fn definitions() -> Vec<Value> { catalog::definitions() }
pub(crate) fn validate_arguments(name: &str, value: &Value) -> Result<(), Failure> {
    input::parse(Operation::named(name).ok_or(Failure::UnknownOperation)?, value).map(|_| ())
}
/// Validate proposal data by the stable operation ID. This constructs no call
/// or permit and never checks/grants execution authority.
pub(crate) fn validate_proposal(id: &str, version: u32, arguments: &Value) -> Result<(), Failure> {
    if version != 1 { return Err(Failure::UnknownOperation); }
    let operation = NAMES.iter().filter_map(|name| Operation::named(name)).find(|op| op.descriptor().id == id).ok_or(Failure::UnknownOperation)?;
    input::parse(operation, arguments).map(|_| ())
}
/// Native construction only. No serialized proposal can create this value.
pub(crate) struct Call { pet: String, operation: Operation, arguments: Arguments, owner: Value, effect: Effect }
impl Call {
    pub(crate) fn arguments(&self) -> &Arguments { &self.arguments }
    fn matches(&self, config: &Configuration) -> bool {
        owner(config, &self.pet).is_ok_and(|owner| owner == self.owner)
            && permission(config, &self.pet, self.operation).is_ok()
    }
}
fn owner(config: &Configuration, pet: &str) -> Result<Value, Failure> {
    let blueprint = crate::pet_config::blueprint(pet).ok_or(Failure::UnknownPet)?;
    if blueprint.retired { return Err(Failure::PetUnavailable); }
    serde_json::to_value(config.pets.get(pet).ok_or(Failure::UnknownPet)?).map_err(|_| Failure::ConfigurationUnavailable)
}
fn permission(config: &Configuration, pet: &str, operation: Operation) -> Result<(), Failure> {
    let grants = config.permissions(pet).map_err(|_| Failure::ConfigurationUnavailable)?;
    let allowed = match operation { Operation::Music=>grants.music, Operation::OpenApp=>grants.open_apps, Operation::Email=>grants.email_drafts, _=>true };
    if allowed { Ok(()) } else { Err(Failure::PermissionDenied) }
}
fn prepare(config: &Configuration, pet: &str, name: &str, value: &Value) -> Result<Call, Failure> {
    let operation = Operation::named(name).ok_or(Failure::UnknownOperation)?;
    let arguments = input::parse(operation, value)?;
    let owner = owner(config, pet)?;
    permission(config, pet, operation)?;
    let effect = if matches!(&arguments, Arguments::Roster(input) if input["action"]=="list") { Effect::ReadOnly } else { operation.descriptor().effect };
    if crate::pet_config::blueprint(pet).is_some_and(|b| b.runtime.kind == crate::pet_config::RuntimeKind::Observer) && effect != Effect::ReadOnly { return Err(Failure::PetUnavailable); }
    Ok(Call { pet:pet.into(),operation,arguments,owner,effect })
}
pub(crate) struct NativeOutcome { pub text: String, pub delivery: Delivery }
impl NativeOutcome {
    pub(crate) fn new(delivery: Delivery, text: impl Into<String>) -> Self { Self { delivery, text:text.into() } }
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Receipt {
    schema_version: u32, pet_id: String, operation: Option<Descriptor>,
    pub(crate) effect: Option<Effect>, pub(crate) delivery: Delivery,
    pub(crate) refusal: Option<Failure>, pub(crate) scope_current: bool,
    effect_verified: bool, pub(crate) output_truncated: bool,
    observed_output_bytes: usize, pub(crate) output: String,
}
impl Receipt {
    fn refused(pet: &str, name: &str, failure: Failure) -> Self {
        Self { schema_version:1,pet_id:if pet.len()<=128{pet.into()}else{"unknown".into()},operation:Operation::named(name).map(Operation::descriptor),effect:None,delivery:Delivery::NotDispatched,refusal:Some(failure),scope_current:false,effect_verified:false,output_truncated:false,observed_output_bytes:0,output:String::new() }
    }
    /// Keep the whole wire envelope bounded, including escaping and metadata.
    fn bound_output(&mut self, text: String) {
        self.observed_output_bytes = text.len();
        let overhead = serde_json::to_vec(self).map(|v| v.len()).unwrap_or(MAX_RESULT_BYTES);
        let mut remaining = MAX_RESULT_BYTES.saturating_sub(overhead);
        for c in text.chars() {
            let cost = match c { '"'|'\\'|'\n'|'\r'|'\t'|'\u{08}'|'\u{0c}'=>2, '\0'..='\u{1f}'=>6, _=>c.len_utf8() };
            if cost > remaining { self.output_truncated = true; break; }
            self.output.push(c); remaining -= cost;
        }
    }
    pub(crate) fn legacy(self) -> (String, bool) {
        // Accepted is an ACK, never an independent verification of the effect.
        let acted = self.scope_current && self.delivery == Delivery::Accepted;
        let text = serde_json::to_string(&self).unwrap_or_else(|_| "{\"delivery\":\"uncertain\",\"effectVerified\":false}".into());
        (text, acted)
    }
}
/// The caller owns session/turn approval and cancellation. No external IPC
/// endpoint exposes this dispatcher; existing native callers supply the fence.
pub(crate) fn run_native(
    native:&crate::pet_native_invocation::Fence,pet:&str,name:&str,value:&Value,
    current:impl Fn()->bool,load:impl Fn()->Result<Configuration,String>,execute:impl FnOnce(&Call)->NativeOutcome,
)->Receipt{
    run(pet,name,value,||native.current()&&current(),load,execute)
}
pub(crate) fn run(
    pet: &str, name: &str, value: &Value,
    current: impl Fn() -> bool,
    load: impl Fn() -> Result<Configuration, String>,
    execute: impl FnOnce(&Call) -> NativeOutcome,
) -> Receipt {
    if !current() { return Receipt::refused(pet,name,Failure::Cancelled); }
    let config = match load() { Ok(c)=>c, Err(_)=>return Receipt::refused(pet,name,Failure::ConfigurationUnavailable) };
    let call = match prepare(&config,pet,name,value) { Ok(call)=>call, Err(error)=>return Receipt::refused(pet,name,error) };
    if !current() { return Receipt::refused(pet,name,Failure::Cancelled); }
    if !load().is_ok_and(|config| call.matches(&config)) { return Receipt::refused(pet,name,Failure::ScopeChanged); }
    if !current() { return Receipt::refused(pet,name,Failure::Cancelled); }
    let outcome = execute(&call);
    let scope_current = current() && load().is_ok_and(|config| call.matches(&config));
    let mut receipt = Receipt { schema_version:1,pet_id:pet.into(),operation:Some(call.operation.descriptor()),effect:Some(call.effect),delivery:outcome.delivery,refusal:None,scope_current,effect_verified:false,output_truncated:false,observed_output_bytes:0,output:String::new() };
    receipt.bound_output(outcome.text);
    receipt
}

#[cfg(test)] mod tests;
