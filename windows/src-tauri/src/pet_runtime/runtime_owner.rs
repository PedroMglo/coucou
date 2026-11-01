//! Authenticated owner observations. No serialized identity is a controller,
//! native lock, worker-drain proof or permission to recover/execute a goal.
use super::{leases::Lease,model};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct FileId{pub device:u64,pub inode:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Profile{pub directory:FileId,pub key_id:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Process{pub pid:u32,pub uid:u32,pub start_ticks:u64,pub boot_id:String,pub pid_namespace:FileId}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Record{
    pub schema_version:u32,pub controller_id:String,pub lease_id:String,pub captured_at:u64,
    pub profile:Profile,pub process:Process,pub owner_directory:FileId,pub owner_lock:FileId,
}
pub(super)fn uuid(s:&str)->bool{s.len()==36&&s.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_hexdigit()})}
impl Record{
    pub(super)fn valid_for(&self,lease:&Lease)->bool{
        self.schema_version==1&&model::token(&self.controller_id)&&model::token(&self.lease_id)
            &&self.controller_id==lease.controller_id&&self.lease_id==lease.id
            &&self.captured_at>=lease.claimed_at&&self.captured_at<lease.deadline_at
            &&self.profile.directory.inode>0&&model::token(&self.profile.key_id)
            &&self.process.pid>0&&self.process.start_ticks>0&&uuid(&self.process.boot_id)&&self.process.pid_namespace.inode>0
            &&self.owner_directory.inode>0&&self.owner_lock.inode>0
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum State{OriginalProcessLive,OwnerAbsent,IdentityMismatch,Unavailable,LegacyUnknown,NotClaimed,Closed}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Reason{OwnerLockHeld,ControllerLockNotHeld,ProcessIdentityChanged,BootChanged,PidNamespaceChanged,ProfileChanged,OwnerFileChanged,ProcessUnobservable,LegacyOwnerMissing}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Observation{
    pub state:State,pub reason:Option<Reason>,pub backend_pid:Option<u32>,pub captured_at:Option<u64>,
    pub source:&'static str,pub execution_available:bool,pub recovery_available:bool,
}
impl Observation{
    pub(super)fn without_owner(state:State,reason:Option<Reason>)->Self{Self{state,reason,backend_pid:None,captured_at:None,source:"nativeGrant",execution_available:false,recovery_available:false}}
    pub(super)fn from_record(record:&Record,state:State,reason:Option<Reason>)->Self{Self{state,reason,backend_pid:Some(record.process.pid),captured_at:Some(record.captured_at),source:"authenticatedRuntimeOwner",execution_available:false,recovery_available:false}}
}
