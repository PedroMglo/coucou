//! Retains the exact old per-lease flock while verifying native absence. This
//! object cannot be cloned/deserialized and never recreates an Owned controller.
use super::*;
use crate::pet_runtime::authority::LostOwnerSource;
pub(in crate::pet_runtime)struct AbsentLease{directory:Dir,lock:File,source:LostOwnerSource,captor:Process}
impl AbsentLease{
    pub(in crate::pet_runtime)fn capture(authority:&Authority,source:LostOwnerSource)->model::Result<Self>{
        crate::pet_data::bootstrap::write_allowed().map_err(|_|model::Error::StorageUnavailable)?;
        let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;let original=source.owner();
        if profile!=original.profile||!original.valid_for(source.lease()){return Err(model::Error::Conflict);}
        let directory=protocol_directory(&path,PROTOCOL,false)?.ok_or(model::Error::NotFound)?;
        let lock=directory.open_file(&format!("{}.lock",original.lease_id),libc::O_RDWR)?.ok_or(model::Error::NotFound)?;
        if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(if std::io::Error::last_os_error().kind()==std::io::ErrorKind::WouldBlock{model::Error::Busy}else{model::Error::StorageUnavailable});}
        let captor=Proc::native()?.current();let gate=Self{directory,lock,source,captor};gate.current(authority)?;Ok(gate)
    }
    /// Recheck at each transaction boundary. Even an unlocked original process
    /// still alive, a reused PID, changed boot/namespace/profile or replaced
    /// directory/FD refuses. Dropping this object releases only its own FD.
    pub(in crate::pet_runtime)fn current(&self,authority:&Authority)->model::Result<()>{
        crate::pet_data::bootstrap::write_allowed().map_err(|_|model::Error::StorageUnavailable)?;
        let original=self.source.owner();let(path,profile)=authority.owner_context().map_err(|_|model::Error::Conflict)?;
        if profile!=original.profile||path!=self.directory.path{return Err(model::Error::Conflict);}
        let proc=Proc::native()?;
        if proc.current()!=self.captor||proc.uid!=original.process.uid||proc.boot!=original.process.boot_id||proc.namespace!=original.process.pid_namespace{return Err(model::Error::Conflict);}
        if proc.process(original.process.pid)?.is_some(){return Err(model::Error::Conflict);}
        self.directory.check()?;
        if identity(&self.directory.file.metadata().map_err(|_|model::Error::StorageUnavailable)?)!=original.owner_directory{return Err(model::Error::Conflict);}
        let current=self.directory.open_file(&format!("{}.lock",original.lease_id),libc::O_RDONLY)?.ok_or(model::Error::NotFound)?;
        for file in [&self.lock,&current]{let m=file.metadata().map_err(|_|model::Error::StorageUnavailable)?;if m.len()!=0||identity(&m)!=original.owner_lock{return Err(model::Error::Conflict);}}
        Ok(())
    }
    pub(in crate::pet_runtime)fn matches(&self,hash:&str,revision:&str)->bool{self.source.matches(hash,revision)}
}
