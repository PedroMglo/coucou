//! Bounded private metadata with optimistic revisions and atomic replacement.
//! A lease pins the file and parent identity; changing either revokes readers.
use super::{Registry,MAX_BYTES};
use std::{ffi::{CString,OsStr},fs::{File,OpenOptions},io::{Read,Write},os::{fd::{AsRawFd,FromRawFd},unix::{ffi::OsStrExt,fs::{DirBuilderExt,MetadataExt,OpenOptionsExt}}},path::{Path,PathBuf},sync::Arc};

#[derive(Clone,Debug,PartialEq,Eq)]
struct Stamp(u64,u64,u64,i64,i64,i64,i64,u32,u32,u64);
fn stamp(m:&std::fs::Metadata)->Stamp{Stamp(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink())}
fn private_file(m:&std::fs::Metadata)->bool{m.is_file()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0&&m.nlink()==1}
fn digest(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn absent(error:&std::io::Error)->bool{error.kind()==std::io::ErrorKind::NotFound}

struct WriterLock(File);
impl WriterLock{
    fn acquire(file:File)->Result<Self,&'static str>{
        if !file.metadata().is_ok_and(|m|private_file(&m)){return Err("invalidRegistryLock");}
        if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err("registryBusy");}
        Ok(Self(file))
    }
}
impl Drop for WriterLock{
    fn drop(&mut self){
        // Another thread may fork while this descriptor is open. CLOEXEC
        // closes it at exec, but does not prevent a forked copy retaining the
        // open-file description meanwhile. Explicit unlock releases our
        // writer lease even while such a copy is still alive.
        unsafe{libc::flock(self.0.as_raw_fd(),libc::LOCK_UN);}
    }
}

struct Directory{path:PathBuf,file:File,device:u64,inode:u64}
impl Directory{
    fn open(path:&Path,create:bool)->Result<Option<Arc<Self>>,&'static str>{
        if create{std::fs::DirBuilder::new().recursive(true).mode(0o700).create(path).map_err(|_|"registryUnavailable")?;}
        let file=match OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path){Ok(f)=>f,Err(e)if absent(&e)=>return Ok(None),Err(_)=>return Err("registryUnavailable")};
        let meta=file.metadata().map_err(|_|"registryUnavailable")?;
        let value=Arc::new(Self{path:path.into(),device:meta.dev(),inode:meta.ino(),file});
        if !value.current(){return Err("invalidRegistryDirectory");}Ok(Some(value))
    }
    fn current(&self)->bool{
        std::fs::symlink_metadata(&self.path).is_ok_and(|m|m.is_dir()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o022==0&&m.dev()==self.device&&m.ino()==self.inode)
    }
    fn open_file(&self,name:&OsStr,flags:i32,mode:u32)->std::io::Result<File>{
        let name=CString::new(name.as_bytes()).map_err(|_|std::io::Error::other("invalid name"))?;
        let fd=unsafe{libc::openat(self.file.as_raw_fd(),name.as_ptr(),flags|libc::O_NOFOLLOW|libc::O_CLOEXEC,mode as libc::mode_t)};
        if fd<0{Err(std::io::Error::last_os_error())}else{Ok(unsafe{File::from_raw_fd(fd)})}
    }
    fn remove(&self,name:&OsStr){if let Ok(name)=CString::new(name.as_bytes()){unsafe{libc::unlinkat(self.file.as_raw_fd(),name.as_ptr(),0)};}}
    fn rename(&self,from:&OsStr,to:&OsStr)->Result<(),&'static str>{
        let from=CString::new(from.as_bytes()).map_err(|_|"registryUnavailable")?;let to=CString::new(to.as_bytes()).map_err(|_|"registryUnavailable")?;
        if unsafe{libc::renameat(self.file.as_raw_fd(),from.as_ptr(),self.file.as_raw_fd(),to.as_ptr())}==0{Ok(())}else{Err("registryUnavailable")}
    }
}

#[derive(Clone)]
pub(super) struct Snapshot{
    pub registry:Registry,
    pub revision:String,
    path:PathBuf,
    directory:Option<Arc<Directory>>,
    file:Option<Arc<File>>,
    stamp:Option<Stamp>,
}
impl Snapshot{
    fn missing(path:&Path,directory:Option<Arc<Directory>>)->Self{Self{registry:Registry::empty(),revision:"missing".into(),path:path.into(),directory,file:None,stamp:None}}
    pub(super) fn unchanged(&self)->bool{
        if self.directory.as_ref().is_some_and(|d|!d.current()){return false;}
        match (&self.file,&self.stamp){
            (Some(file),Some(before))=>file.metadata().is_ok_and(|m|private_file(&m)&&stamp(&m)==*before)&&std::fs::symlink_metadata(&self.path).is_ok_and(|m|private_file(&m)&&stamp(&m)==*before),
            (None,None)=>std::fs::symlink_metadata(&self.path).is_err_and(|e|absent(&e)),
            _=>false,
        }
    }
}
fn read_in(path:&Path,directory:Arc<Directory>)->Result<Snapshot,&'static str>{
    let name=path.file_name().ok_or("registryUnavailable")?;
    let file=match directory.open_file(name,libc::O_RDONLY|libc::O_NONBLOCK,0){Ok(f)=>f,Err(e)if absent(&e)=>return Ok(Snapshot::missing(path,Some(directory))),Err(_)=>return Err("registryUnavailable")};
    let meta=file.metadata().map_err(|_|"registryUnavailable")?;
    if !private_file(&meta){return Err("invalidRegistryFile");}if meta.len()>MAX_BYTES as u64{return Err("registryLimit");}
    let before=stamp(&meta);let mut bytes=vec![];(&file).take(MAX_BYTES as u64+1).read_to_end(&mut bytes).map_err(|_|"registryUnavailable")?;
    if bytes.len()>MAX_BYTES{return Err("registryLimit");}
    let registry:Registry=serde_json::from_slice(&bytes).map_err(|_|"invalidRegistry")?;registry.validate()?;
    if registry.generation==0{return Err("invalidRegistry");}
    let snapshot=Snapshot{registry,revision:digest(&bytes),path:path.into(),directory:Some(directory),file:Some(Arc::new(file)),stamp:Some(before)};
    if !snapshot.unchanged(){return Err("registryChanged");}Ok(snapshot)
}
pub(super) fn load_at(path:&Path)->Result<Snapshot,&'static str>{
    let parent=path.parent().ok_or("registryUnavailable")?;
    match Directory::open(parent,false)?{Some(dir)=>read_in(path,dir),None=>Ok(Snapshot::missing(path,None))}
}
pub(super) struct Prepared {
    previous:Snapshot, directory:Arc<Directory>, path:PathBuf,
    temp:std::ffi::OsString, revision:String,
    published:bool, _lock:WriterLock,
}
impl Prepared {
    /// Only atomic rename runs under the native selection mutex. Copying,
    /// parsing, serializing and fsync happen outside that mutex.
    pub(super) fn publish(&mut self)->Result<(), &'static str>{
        if self.published||!self.previous.unchanged(){return Err("registryConflict");}
        self.directory.rename(&self.temp,self.path.file_name().ok_or("registryUnavailable")?)?;
        self.published=true;Ok(())
    }
    pub(super) fn confirm(self)->Result<Snapshot,&'static str>{
        if !self.published{return Err("registryUnavailable");}
        self.directory.file.sync_all().map_err(|_|"registryCommitUncertain")?;
        let result=read_in(&self.path,self.directory.clone()).map_err(|_|"registryCommitUncertain")?;
        if result.revision!=self.revision{return Err("registryCommitUncertain");}Ok(result)
    }
}
impl Drop for Prepared{fn drop(&mut self){if !self.published{self.directory.remove(&self.temp);}}}
pub(super) fn prepare(path:&Path,expected:Option<&str>,update:impl FnOnce(&mut Registry)->Result<(),&'static str>)->Result<Prepared,&'static str>{
    let parent=path.parent().ok_or("registryUnavailable")?;
    let directory=Directory::open(parent,true)?.ok_or("registryUnavailable")?;
    let name=format!(".{}.lock",path.file_name().ok_or("registryUnavailable")?.to_string_lossy());
    let lock=directory.open_file(OsStr::new(&name),libc::O_RDWR|libc::O_CREAT|libc::O_NONBLOCK,0o600).map_err(|_|"registryUnavailable")?;
    let lock=WriterLock::acquire(lock)?;
    let previous=read_in(path,directory.clone())?;
    if expected.is_some_and(|r|r!=previous.revision){return Err("registryConflict");}
    let mut next=previous.registry.clone();update(&mut next)?;next=next.next(&previous.registry)?;
    let bytes=serde_json::to_vec_pretty(&next).map_err(|_|"invalidRegistry")?;
    if bytes.len()>MAX_BYTES{return Err("registryLimit");}
    let temp=std::ffi::OsString::from(format!(".attachments-{}.tmp",crate::assistant_memory::new_id()));
    let prepared=Prepared{previous,directory,path:path.into(),temp,revision:digest(&bytes),published:false,_lock:lock};
    let mut file=prepared.directory.open_file(&prepared.temp,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600).map_err(|_|"registryUnavailable")?;
    file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|_|"registryUnavailable")?;
    drop(file);
    // Detect edits while staging, before taking the native selection fence.
    if !prepared.previous.unchanged(){return Err("registryConflict");}
    Ok(prepared)
}
