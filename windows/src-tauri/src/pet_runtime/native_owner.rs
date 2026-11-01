//! Linux process incarnation plus a private per-lease lock retained by Live and
//! every WorkerGuard. Read-only absence observations never reconstruct authority.
use super::{authority::Authority,leases::Lease,model,runtime_owner::{self,FileId,Process,Record,Observation,State,Reason},store::{Dir,protocol_directory}};
use std::{ffi::CString,fs::{File,Metadata},io::Read,os::{fd::{AsRawFd,FromRawFd},unix::{fs::{MetadataExt,OpenOptionsExt}}}};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-runtime-owner-locks\"}\n";
#[path="absent_owner.rs"]mod absent;
pub(super)use absent::AbsentLease;
fn identity(m:&Metadata)->FileId{FileId{device:m.dev(),inode:m.ino()}}
fn open_at(parent:&File,name:&str,flags:i32)->std::io::Result<File>{
    let name=CString::new(name).map_err(|_|std::io::Error::other("invalid native path"))?;
    let fd=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),flags|libc::O_CLOEXEC,0)};
    if fd<0{Err(std::io::Error::last_os_error())}else{Ok(unsafe{File::from_raw_fd(fd)})}
}
fn bytes(file:File,max:u64)->model::Result<String>{let mut text=String::new();file.take(max+1).read_to_string(&mut text).map_err(|_|model::Error::StorageUnavailable)?;if text.len()as u64>max{return Err(model::Error::LimitReached);}Ok(text)}
fn start(directory:&File,pid:u32)->model::Result<u64>{
    let text=bytes(open_at(directory,"stat",libc::O_RDONLY|libc::O_NOFOLLOW).map_err(|_|model::Error::StorageUnavailable)?,8192)?;
    if text.split_once(' ').and_then(|(p,_)|p.parse::<u32>().ok())!=Some(pid){return Err(model::Error::Conflict);}
    crate::pet_observability::process_census::native_process_start_ticks(&text).ok_or(model::Error::InvalidRecord)
}
pub(super)struct Proc{root:File,boot:String,namespace:FileId,uid:u32,self_pid:u32,self_start:u64}
impl Proc{
    pub(super)fn native()->model::Result<Self>{
        let root=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open("/proc").map_err(|_|model::Error::StorageUnavailable)?;
        let mut info=std::mem::MaybeUninit::<libc::statfs>::uninit();if unsafe{libc::fstatfs(root.as_raw_fd(),info.as_mut_ptr())}!=0||unsafe{info.assume_init()}.f_type as u64!=0x9fa0{return Err(model::Error::UnsafePath);}
        let directory=open_at(&root,"self",libc::O_RDONLY|libc::O_DIRECTORY).map_err(|_|model::Error::StorageUnavailable)?;let pid=std::process::id();let uid=unsafe{libc::geteuid()};
        let first=start(&directory,pid)?;if directory.metadata().map_err(|_|model::Error::StorageUnavailable)?.uid()!=uid{return Err(model::Error::Conflict);}
        let namespace=identity(&open_at(&directory,"ns/pid",libc::O_RDONLY).map_err(|_|model::Error::StorageUnavailable)?.metadata().map_err(|_|model::Error::StorageUnavailable)?);
        let boot=bytes(open_at(&root,"sys/kernel/random/boot_id",libc::O_RDONLY|libc::O_NOFOLLOW).map_err(|_|model::Error::StorageUnavailable)?,128)?.trim().to_string();
        if !runtime_owner::uuid(&boot)||start(&directory,pid)?!=first{return Err(model::Error::Conflict);}
        Ok(Self{root,boot,namespace,uid,self_pid:pid,self_start:first})
    }
    pub(super)fn current(&self)->Process{Process{pid:self.self_pid,uid:self.uid,start_ticks:self.self_start,boot_id:self.boot.clone(),pid_namespace:self.namespace.clone()}}
    pub(super)fn process(&self,pid:u32)->model::Result<Option<Process>>{
        let directory=match open_at(&self.root,&pid.to_string(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW){Ok(f)=>f,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(_)=>return Err(model::Error::StorageUnavailable)};
        let before=start(&directory,pid)?;let uid=directory.metadata().map_err(|_|model::Error::StorageUnavailable)?.uid();let namespace=identity(&open_at(&directory,"ns/pid",libc::O_RDONLY).map_err(|_|model::Error::StorageUnavailable)?.metadata().map_err(|_|model::Error::StorageUnavailable)?);
        if start(&directory,pid)?!=before{return Err(model::Error::Conflict);}
        Ok(Some(Process{pid,uid,start_ticks:before,boot_id:self.boot.clone(),pid_namespace:namespace}))
    }
}
/// Not Clone/Deserialize. Closing the FD releases only this open description;
/// never explicitly unlock a potentially inherited description after fork.
pub(super)struct OwnerLease{directory:Dir,lock:File,record:Record}
impl OwnerLease{
    pub(super)fn capture(authority:&Authority,lease:&Lease,now:u64)->model::Result<Self>{
        let(context,profile)=authority.owner_context().map_err(|_|model::Error::StorageUnavailable)?;let proc=Proc::native()?;
        let directory=protocol_directory(&context,PROTOCOL,true)?.ok_or(model::Error::StorageUnavailable)?;
        if directory.names()?.len()>=4096{return Err(model::Error::LimitReached);}
        let lock=directory.open_file(&format!("{}.lock",lease.id),libc::O_RDWR|libc::O_CREAT|libc::O_EXCL)?.ok_or(model::Error::StorageUnavailable)?;
        if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(model::Error::Busy);}
        lock.sync_all().map_err(|_|model::Error::StorageUnavailable)?;directory.durable()?;
        let record=Record{schema_version:1,controller_id:lease.controller_id.clone(),lease_id:lease.id.clone(),captured_at:now,profile,process:proc.current(),owner_directory:identity(&directory.file.metadata().map_err(|_|model::Error::StorageUnavailable)?),owner_lock:identity(&lock.metadata().map_err(|_|model::Error::StorageUnavailable)?)};
        if !record.valid_for(lease){return Err(model::Error::InvalidRecord);}let owner=Self{directory,lock,record};owner.current()?;Ok(owner)
    }
    pub(super)fn record(&self)->&Record{&self.record}
    pub(super)fn current(&self)->model::Result<()>{
        if std::process::id()!=self.record.process.pid||unsafe{libc::geteuid()}!=self.record.process.uid{return Err(model::Error::Conflict);}
        self.directory.check()?;if identity(&self.directory.file.metadata().map_err(|_|model::Error::StorageUnavailable)?)!=self.record.owner_directory{return Err(model::Error::Conflict);}let file=self.directory.open_file(&format!("{}.lock",self.record.lease_id),libc::O_RDONLY)?.ok_or(model::Error::NotFound)?;
        let first=self.lock.metadata().map_err(|_|model::Error::StorageUnavailable)?;let current=file.metadata().map_err(|_|model::Error::StorageUnavailable)?;
        if first.len()!=0||current.len()!=0||identity(&first)!=self.record.owner_lock||identity(&current)!=self.record.owner_lock{return Err(model::Error::Conflict);}Ok(())
    }
    pub(super)fn matches(&self,authority:&Authority,lease:&Lease)->bool{self.current().is_ok()&&self.record.valid_for(lease)&&authority.owner_context().is_ok_and(|(_,profile)|profile==self.record.profile)}
}
pub(super)fn observe(authority:&Authority,record:&Record)->Observation{
    let result=(||->std::result::Result<(State,Option<Reason>),Reason>{
        let(path,profile)=authority.owner_context().map_err(|_|Reason::ProfileChanged)?;
        if profile!=record.profile{return Err(Reason::ProfileChanged);}
        let proc=Proc::native().map_err(|_|Reason::ProcessUnobservable)?;
        if proc.uid!=record.process.uid{return Err(Reason::ProcessIdentityChanged);}
        if proc.boot!=record.process.boot_id{return Err(Reason::BootChanged);}
        if proc.namespace!=record.process.pid_namespace{return Err(Reason::PidNamespaceChanged);}
        let directory=protocol_directory(&path,PROTOCOL,false).map_err(|_|Reason::OwnerFileChanged)?.ok_or(Reason::OwnerFileChanged)?;
        if identity(&directory.file.metadata().map_err(|_|Reason::OwnerFileChanged)?)!=record.owner_directory{return Err(Reason::OwnerFileChanged);}
        let lock=directory.open_file(&format!("{}.lock",record.lease_id),libc::O_RDWR).map_err(|_|Reason::OwnerFileChanged)?.ok_or(Reason::OwnerFileChanged)?;
        let metadata=lock.metadata().map_err(|_|Reason::OwnerFileChanged)?;if metadata.len()!=0||identity(&metadata)!=record.owner_lock{return Err(Reason::OwnerFileChanged);}
        let held=if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}==0{false}else if std::io::Error::last_os_error().kind()==std::io::ErrorKind::WouldBlock{true}else{return Err(Reason::ProcessUnobservable);};
        let observation=match proc.process(record.process.pid).map_err(|_|Reason::ProcessUnobservable)?{
            Some(process)if process==record.process=>(State::OriginalProcessLive,if held{None}else{Some(Reason::ControllerLockNotHeld)}),
            Some(_)=>(State::IdentityMismatch,Some(Reason::ProcessIdentityChanged)),
            None if held=>(State::Unavailable,Some(Reason::OwnerLockHeld)),None=>(State::OwnerAbsent,None),
        };
        directory.check().map_err(|_|Reason::OwnerFileChanged)?;let current=directory.open_file(&format!("{}.lock",record.lease_id),libc::O_RDONLY).map_err(|_|Reason::OwnerFileChanged)?.ok_or(Reason::OwnerFileChanged)?;
        if identity(&current.metadata().map_err(|_|Reason::OwnerFileChanged)?)!=record.owner_lock{return Err(Reason::OwnerFileChanged);}Ok(observation)
    })();
    match result{Ok((state,reason))=>Observation::from_record(record,state,reason),Err(reason)=>Observation::from_record(record,if matches!(reason,Reason::ProcessUnobservable|Reason::OwnerLockHeld){State::Unavailable}else{State::IdentityMismatch},Some(reason))}
}
#[cfg(test)]#[path="native_owner_tests.rs"]mod tests;
