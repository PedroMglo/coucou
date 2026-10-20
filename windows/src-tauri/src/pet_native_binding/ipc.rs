//! Wire contract only. Public revisions and intent IDs never mint authority.
use serde::{Deserialize,Deserializer,Serialize,Serializer,de::{self,Visitor}};
use std::fmt;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Revision(pub(crate) u64);
impl Serialize for Revision {
    fn serialize<S:Serializer>(&self,s:S)->Result<S::Ok,S::Error>{s.serialize_str(&self.0.to_string())}
}
impl<'de> Deserialize<'de> for Revision {
    fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error>{
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value=Revision;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("canonical revision string")}
            fn visit_str<E:de::Error>(self,s:&str)->Result<Revision,E>{
                if s.is_empty()||s.len()>20||!s.bytes().all(|b|b.is_ascii_digit()){return Err(E::custom("invalid revision"));}
                let n=s.parse::<u64>().map_err(|_|E::custom("invalid revision"))?;
                if n.to_string()!=s{return Err(E::custom("invalid revision"));}Ok(Revision(n))
            }
        }
        d.deserialize_str(V)
    }
}
#[derive(Clone,PartialEq,Eq)]
pub(crate) struct Text<const N:usize>(String);
impl<const N:usize> Text<N> {pub(crate) fn as_str(&self)->&str{&self.0}}
impl<'de,const N:usize> Deserialize<'de> for Text<N> {
    fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error>{
        struct V<const N:usize>;
        impl<'de,const N:usize> Visitor<'de> for V<N>{
            type Value=Text<N>;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("bounded text")}
            fn visit_str<E:de::Error>(self,s:&str)->Result<Text<N>,E>{
                if s.len()>N||s.chars().any(char::is_control){return Err(E::custom("invalid bounded text"));}
                Ok(Text(s.into()))
            }
        }
        d.deserialize_str(V::<N>)
    }
}
#[derive(Clone,PartialEq,Eq)]
pub(crate) struct IntentId(Text<64>);
impl IntentId {pub(crate) fn as_str(&self)->&str{self.0.as_str()}}
impl<'de> Deserialize<'de> for IntentId {
    fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error>{
        let t=Text::<64>::deserialize(d)?;
        if t.as_str().is_empty()||!t.as_str().bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')){return Err(de::Error::custom("invalid intent"));}
        Ok(Self(t))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Route {Home,System,Tasks,Sessions,Providers,Models,Profiles}
impl Route {
    pub(crate) fn from_section(section:Option<&str>)->Option<Self>{
        Some(match section.unwrap_or("sessions"){"home"=>Self::Home,"system"=>Self::System,"tasks"=>Self::Tasks,"sessions"=>Self::Sessions,"providers"=>Self::Providers,"models"=>Self::Models,"profiles"=>Self::Profiles,_=>return None})
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
pub(crate) enum Mode {Inactive,Compact,Expanded {route:Route}}
impl Mode {
    pub(crate) fn system(&self)->bool{matches!(self,Self::Expanded{route:Route::System})}
    pub(crate) fn expanded(&self)->bool{matches!(self,Self::Expanded{..})}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Context {
    pub(crate) schema_version:u8,
    pub(crate) revision:Revision,
    pub(crate) mode:Mode,
    pub(crate) pending:bool,
    pub(crate) native_available:bool,
    pub(crate) native_visible:bool,
    pub(crate) native_mapped:bool,
    pub(crate) document_ready:bool,
    pub(crate) system_enabled:bool,
    pub(crate) system_eligible:bool,
    pub(crate) authority_contract:&'static str,
}
#[derive(Clone,PartialEq,Eq,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub(crate) enum NavigationIntent {Expand {route:Route},Navigate {route:Route},Collapse {}}
#[derive(Clone,PartialEq,Eq,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct NavigationRequest {
    pub(crate) expected_revision:Revision,
    pub(crate) intent_id:IntentId,
    pub(crate) intent:NavigationIntent,
}
#[derive(Serialize,Clone)]
#[serde(rename_all="camelCase")]
pub(crate) struct NavigationReceipt {pub(crate) intent_id:String,pub(crate) applied:bool,pub(crate) context:Context}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct ContextRequest {pub(crate) expected_revision:Revision}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct ReadRequest {pub(crate) expected_revision:Revision,pub(crate) force:bool}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct ScanRequest {pub(crate) expected_revision:Revision,pub(crate) scope:Text<128>,pub(crate) intent_id:IntentId}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct EnableWifiRequest {pub(crate) expected_revision:Revision,pub(crate) scope:Text<128>,pub(crate) intent_id:IntentId}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct TicketRequest {pub(crate) expected_revision:Revision,pub(crate) ticket_id:Text<128>}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum ErrorCode {
    Unavailable,NativeCallerMismatch,StaleContext,DocumentInactive,SystemDisabled,
    TransitionPending,Unsupported,InvalidRequest,IntentConflict,NativeFailure,Superseded,Capacity,
}
#[derive(Clone,Debug,Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
pub(crate) enum Failure {
    Binding {code:ErrorCode,context:Option<Context>},
    Network {code:crate::lifecycle::Reason},
}
impl Failure {
    pub(crate) fn binding(code:ErrorCode,context:Option<Context>)->Self{Self::Binding{code,context}}
    pub(crate) fn network(code:crate::lifecycle::Reason)->Self{Self::Network{code}}
    pub(crate) fn unavailable()->Self{Self::binding(ErrorCode::Unavailable,None)}
}
/// Do not expose raw serde/native details from rejected command payloads.
/// Checked has no Deserialize impl, so the explicit CommandArg owns this boundary.
pub(crate) struct Checked<T>(pub(crate) T);
impl<'de,R:tauri::Runtime,T:Deserialize<'de>> tauri::ipc::CommandArg<'de,R> for Checked<T> {
    fn from_command(command:tauri::ipc::CommandItem<'de,R>)->Result<Self,tauri::ipc::InvokeError>{
        T::deserialize(command).map(Self).map_err(|_| {
            tauri::ipc::InvokeError::from(serde_json::json!({"kind":"binding","code":"invalidRequest","context":null}))
        })
    }
}
#[derive(Serialize,Clone)]
#[serde(rename_all="camelCase")]
pub(crate) struct OpenedEvent {
    pub(crate) pet:Option<String>,pub(crate) section:Option<String>,
    pub(crate) navigation_revision:Revision,pub(crate) context:Context,
}
#[derive(Serialize,Clone)]
#[serde(rename_all="camelCase")]
pub(crate) struct CollapseEvent {pub(crate) navigation_revision:Revision,pub(crate) context:Context}
