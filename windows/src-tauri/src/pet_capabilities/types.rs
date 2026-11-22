//! Public metadata only. No descriptor or revision is an execution permit.
use serde::Serialize;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Kind{Llm,Service,Observer}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Lifecycle{Active,Inactive,Retired}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Support{Supported,Unsupported,Unknown}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Availability{Ready,Unavailable,Unknown}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Reason{NotImplemented,UnsupportedPlatform,NotConfigured,InactivePet,RetiredPet,ModelUnresolved,ModelUnsupported,NativeApprovalUnavailable,NativeControlUnavailable,ControlConflict,TurnRequired,TargetRequired,StaleObservation,ProviderUnavailable,Busy,PermissionDenied,PolicyDenied,LibraryUnavailable,SettingsUnconfirmed,Unknown}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Source{BundledManifest,Configuration,ProviderCatalog,RuntimeObservation}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Decision{None,NativeApproval,ExactCommandApproval,ScopedGrant}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Caller{NativeUser,TaskRuntime,ProviderModel}
#[derive(Clone,Default,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Limits{pub max_items:Option<usize>,pub max_bytes:Option<u64>,pub max_calls:Option<usize>,pub timeout_ms:Option<u64>}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Capability{
    pub id:&'static str,pub support:Support,pub availability:Availability,pub reason:Option<Reason>,pub source:Vec<Source>,
    pub observed_at:Option<u64>,pub expires_at:Option<u64>,pub model_binding:Option<String>,pub decision:Decision,pub callers:Vec<Caller>,pub limits:Limits,pub limitations:Vec<&'static str>,
}
impl Capability{
    pub(super) fn unsupported(id:&'static str)->Self{Self{id,support:Support::Unsupported,availability:Availability::Unavailable,reason:Some(Reason::NotImplemented),source:vec![Source::BundledManifest],observed_at:None,expires_at:None,model_binding:None,decision:Decision::None,callers:vec![Caller::NativeUser],limits:Limits::default(),limitations:vec![]}}
}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Binding{#[serde(skip_serializing_if="Option::is_none")]pub engine_binding:Option<crate::local_chat::lmstudio::Binding>,pub adapter:Option<&'static str>,pub provider:Option<String>,pub model:Option<String>,pub profile:Option<String>}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Concurrency{pub scope:Option<&'static str>,pub max_active:Option<usize>}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Descriptor{
    pub schema_version:u32,pub pet_id:String,pub kind:Kind,pub name:String,pub lifecycle:Lifecycle,pub binding:Binding,pub revision:String,
    pub observed_at:Option<u64>,pub configuration_available:bool,pub capabilities:Vec<Capability>,pub concurrency:Concurrency,
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Snapshot{pub schema_version:u32,pub revision:String,pub observed_at:u64,pub configuration_available:bool,pub pets:Vec<Descriptor>}
