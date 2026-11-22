//! The existing private owner registry and /proc incarnation checks, scoped to
//! metadata operations. A retained FD is not serializable or execution authority.
use super::*;
use crate::pet_runtime::runtime_owner::Operation;
use std::sync::{Arc,Weak,Mutex,OnceLock};
use std::collections::BTreeMap;
// Process-local lifetime witnesses, never serialized or restored. An unlocked
// FD/expired deadline without this exact known lifetime proves nothing.
fn registry()->&'static Mutex<BTreeMap<String,(Operation,Weak<()>)>>{static R:OnceLock<Mutex<BTreeMap<String,(Operation,Weak<()>)>>>=OnceLock::new();R.get_or_init(||Mutex::new(BTreeMap::new()))}
fn locally_ended(record:&Operation,proc:&Proc)->bool{
 proc.current()==record.process&&registry().lock().ok().is_some_and(|r|r.get(&record.operation_id).is_some_and(|(known,lifetime)|known==record&&lifetime.upgrade().is_none()))
}
fn name(record:&Operation)->String{format!("op-{}.lock",record.operation_id)}
fn check_file(directory:&Dir,lock:&File,record:&Operation)->model::Result<()>{
    directory.check()?;
    if identity(&directory.file.metadata().map_err(|_|model::Error::StorageUnavailable)?)!=record.owner_directory{return Err(model::Error::Conflict);}
    let current=directory.open_file(&name(record),libc::O_RDONLY)?.ok_or(model::Error::NotFound)?;
    for file in [lock,&current]{let m=file.metadata().map_err(|_|model::Error::StorageUnavailable)?;if m.len()!=0||identity(&m)!=record.owner_lock{return Err(model::Error::Conflict);}}
    Ok(())
}
pub(in crate::pet_runtime)struct OwnedOperation{directory:Dir,lock:File,record:Operation,_lifetime:Arc<()>}
impl OwnedOperation{
    pub(in crate::pet_runtime)fn capture(authority:&Authority,id:&str,scope:&str,now:u64)->model::Result<Self>{
        if !model::token(id)||scope.len()!=64||!scope.bytes().all(|b|b.is_ascii_hexdigit()){return Err(model::Error::InvalidInput);}
        let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;
        let directory=protocol_directory(&path,PROTOCOL,true)?.ok_or(model::Error::StorageUnavailable)?;
        if directory.names()?.len()>=4096{return Err(model::Error::LimitReached);}
        let lock=directory.open_file(&format!("op-{id}.lock"),libc::O_RDWR|libc::O_CREAT|libc::O_EXCL)?.ok_or(model::Error::StorageUnavailable)?;
        if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(model::Error::Busy);}
        lock.sync_all().map_err(|_|model::Error::StorageUnavailable)?;directory.durable()?;
        let record=Operation{schema_version:1,operation_id:id.into(),scope_hash:scope.into(),captured_at:now,profile,process:Proc::native()?.current(),owner_directory:identity(&directory.file.metadata().map_err(|_|model::Error::StorageUnavailable)?),owner_lock:identity(&lock.metadata().map_err(|_|model::Error::StorageUnavailable)?)};
        if !record.valid(id,scope,now){return Err(model::Error::InvalidRecord);}
        let lifetime=Arc::new(());let owner=Self{directory,lock,record,_lifetime:lifetime.clone()};owner.current(authority)?;
        {let mut r=registry().lock().map_err(|_|model::Error::StorageUnavailable)?;if r.len()>=4096{return Err(model::Error::LimitReached);}if r.insert(owner.record.operation_id.clone(),(owner.record.clone(),Arc::downgrade(&lifetime))).is_some(){return Err(model::Error::Conflict);}}Ok(owner)
    }
    pub(in crate::pet_runtime)fn record(&self)->&Operation{&self.record}
    pub(in crate::pet_runtime)fn current(&self,authority:&Authority)->model::Result<()>{
        let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;
        if path!=self.directory.path||profile!=self.record.profile||Proc::native()?.current()!=self.record.process{return Err(model::Error::Conflict);}
        check_file(&self.directory,&self.lock,&self.record)
    }
}
pub(in crate::pet_runtime)struct EndedOperation{directory:Dir,lock:File,record:Operation,captor:Process,local:bool}
impl EndedOperation{
    pub(in crate::pet_runtime)fn capture(authority:&Authority,record:&Operation)->model::Result<Self>{
        if !record.valid(&record.operation_id,&record.scope_hash,record.captured_at){return Err(model::Error::InvalidRecord);}
        let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;if profile!=record.profile{return Err(model::Error::Conflict);}
        let directory=protocol_directory(&path,PROTOCOL,false)?.ok_or(model::Error::NotFound)?;
        let lock=directory.open_file(&name(record),libc::O_RDWR)?.ok_or(model::Error::NotFound)?;
        if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(if std::io::Error::last_os_error().kind()==std::io::ErrorKind::WouldBlock{model::Error::Busy}else{model::Error::StorageUnavailable});}
        let proc=Proc::native()?;let local=locally_ended(record,&proc);let gate=Self{directory,lock,record:record.clone(),captor:proc.current(),local};gate.current(authority)?;Ok(gate)
    }
    pub(in crate::pet_runtime)fn record(&self)->&Operation{&self.record}
    pub(in crate::pet_runtime)fn witness(&self)->&'static str{if self.local{"sameBackendOperationEnded"}else{"originalProcessAbsent"}}
    pub(in crate::pet_runtime)fn current(&self,authority:&Authority)->model::Result<()>{
        crate::pet_data::bootstrap::write_allowed().map_err(|_|model::Error::StorageUnavailable)?;
        let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;let proc=Proc::native()?;let old=&self.record.process;
        if path!=self.directory.path||profile!=self.record.profile||proc.current()!=self.captor||proc.uid!=old.uid||proc.boot!=old.boot_id||proc.namespace!=old.pid_namespace||(if self.local{!locally_ended(&self.record,&proc)}else{proc.process(old.pid)?.is_some()}){return Err(model::Error::Conflict);}
        check_file(&self.directory,&self.lock,&self.record)
    }
}
