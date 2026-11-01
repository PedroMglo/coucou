//! Bounded private metadata with optimistic revisions and atomic replacement.
//! A lease pins the file and parent identity; changing either revokes readers.
#[path = "preferences_file.rs"]
pub(crate) mod preferences_file;
#[path="file_target.rs"]pub(crate)mod file_target;
use serde::{Serialize,de::DeserializeOwned};
pub(crate)trait Document:Clone+Serialize+DeserializeOwned{
 const MAX_BYTES:usize;const LOCK_NAME:&'static str;const TEMP_PREFIX:&'static str;
 fn empty()->Self;fn validate(&self)->Result<(),&'static str>;fn generation(&self)->u64;fn next(self,previous:&Self)->Result<Self,&'static str>;
}
use std::{ffi::{CString,OsStr},fs::{File,OpenOptions},io::{Read,Write},os::{fd::{AsRawFd,FromRawFd},unix::{ffi::OsStrExt,fs::{DirBuilderExt,MetadataExt,OpenOptionsExt}}},path::{Path,PathBuf},sync::Arc};

#[derive(Clone,Debug,PartialEq,Eq)]
struct Stamp(u64,u64,u64,i64,i64,i64,i64,u32,u32,u64);
fn stamp(m:&std::fs::Metadata)->Stamp{Stamp(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink())}
fn private_file(m:&std::fs::Metadata)->bool{m.is_file()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0&&m.nlink()==1}
fn digest(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn absent(error:&std::io::Error)->bool{error.kind()==std::io::ErrorKind::NotFound}

pub(crate)struct WriterLock(pub(crate)File);
impl WriterLock{
    pub(crate)fn acquire(file:File)->Result<Self,&'static str>{
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

/// Hold a cooperating operation across more than one atomic document update.
/// Names are backend constants; inspect unknown nodes before opening for IO.
pub(crate)fn operation_lock(path:&Path,name:&str)->Result<WriterLock,&'static str>{
    if name.is_empty()||name=="."||name==".."||name.contains('/')||name.contains('\0'){return Err("invalidRegistryLock");}
    let directory=Directory::open(path.parent().ok_or("registryUnavailable")?,false)?.ok_or("registryUnavailable")?;
    operation_lock_in(&directory,name)
}
fn operation_lock_in(directory:&Directory,name:&str)->Result<WriterLock,&'static str>{
    operation_lock_guarded(directory,name,||Ok(()))
}
fn operation_lock_guarded(directory:&Directory,name:&str,guard:impl Fn()->Result<(),&'static str>)->Result<WriterLock,&'static str>{
    if name.is_empty()||name=="."||name==".."||name.contains('/')||name.contains('\0')||!directory.current(){return Err("invalidRegistryLock");}
    let name=OsStr::new(name);
    for _ in 0..2{
        let file=match directory.open_file(name,libc::O_PATH,0){
            Ok(pin)=>{
                let m=pin.metadata().map_err(|_|"registryUnavailable")?;
                if !private_file(&m)||m.len()!=0{return Err("invalidRegistryLock");}
                let file=File::open(format!("/proc/self/fd/{}",pin.as_raw_fd())).map_err(|_|"registryUnavailable")?;
                if stamp(&file.metadata().map_err(|_|"registryUnavailable")?)!=stamp(&m){return Err("registryChanged");}file
            },
            Err(e)if absent(&e)=>{guard()?;if !directory.current(){return Err("registryChanged");}match directory.open_file(name,libc::O_RDWR|libc::O_CREAT|libc::O_EXCL|libc::O_NONBLOCK,0o600){Ok(f)=>f,Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(_)=>return Err("registryUnavailable")}},
            Err(_)=>return Err("registryUnavailable"),
        };
        guard()?;let lock=WriterLock::acquire(file)?;
        let named=directory.open_file(name,libc::O_PATH,0).map_err(|_|"registryChanged")?;
        if !directory.current()||stamp(&named.metadata().map_err(|_|"registryUnavailable")?)!=stamp(&lock.0.metadata().map_err(|_|"registryUnavailable")?){return Err("registryChanged");}
        return Ok(lock);
    }Err("registryBusy")
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
pub(crate) struct Snapshot<T:Document>{
    pub registry:T,
    pub revision:String,
    path:PathBuf,
    directory:Option<Arc<Directory>>,
    file:Option<Arc<File>>,
    stamp:Option<Stamp>,
}
impl<T:Document> Snapshot<T>{
    /// Distinguish an ordinary atomic catalog edit from substitution of the
    /// parent originally captured by a reader. This check never creates files.
    pub(crate)fn directory_current(&self)->bool{self.directory.as_ref().is_some_and(|d|d.current())}
    fn missing(path:&Path,directory:Option<Arc<Directory>>)->Self{Self{registry:T::empty(),revision:"missing".into(),path:path.into(),directory,file:None,stamp:None}}
    pub(crate) fn unchanged(&self)->bool{
        if self.directory.as_ref().is_some_and(|d|!d.current()){return false;}
        match (&self.file,&self.stamp){
            (Some(file),Some(before))=>file.metadata().is_ok_and(|m|private_file(&m)&&stamp(&m)==*before)&&std::fs::symlink_metadata(&self.path).is_ok_and(|m|private_file(&m)&&stamp(&m)==*before),
            (None,None)=>std::fs::symlink_metadata(&self.path).is_err_and(|e|absent(&e)),
            _=>false,
        }
    }
}
fn read_in<T:Document>(path:&Path,directory:Arc<Directory>)->Result<Snapshot<T>,&'static str>{
    let name=path.file_name().ok_or("registryUnavailable")?;
    let file=match directory.open_file(name,libc::O_RDONLY|libc::O_NONBLOCK,0){Ok(f)=>f,Err(e)if absent(&e)=>return Ok(Snapshot::missing(path,Some(directory))),Err(_)=>return Err("registryUnavailable")};
    let meta=file.metadata().map_err(|_|"registryUnavailable")?;
    if !private_file(&meta){return Err("invalidRegistryFile");}if meta.len()>T::MAX_BYTES as u64{return Err("registryLimit");}
    let before=stamp(&meta);let mut bytes=vec![];(&file).take(T::MAX_BYTES as u64+1).read_to_end(&mut bytes).map_err(|_|"registryUnavailable")?;
    if bytes.len()>T::MAX_BYTES{return Err("registryLimit");}
    let registry:T=serde_json::from_slice(&bytes).map_err(|_|"invalidRegistry")?;registry.validate()?;
    if registry.generation()==0{return Err("invalidRegistry");}
    let snapshot=Snapshot{registry,revision:digest(&bytes),path:path.into(),directory:Some(directory),file:Some(Arc::new(file)),stamp:Some(before)};
    if !snapshot.unchanged(){return Err("registryChanged");}Ok(snapshot)
}
pub(crate) fn load_at<T:Document>(path:&Path)->Result<Snapshot<T>,&'static str>{
    let parent=path.parent().ok_or("registryUnavailable")?;
    match Directory::open(parent,false)?{Some(dir)=>read_in(path,dir),None=>Ok(Snapshot::missing(path,None))}
}
/// A mutator has an expected revision before it may publish. A snapshot that
/// changed while being read cannot satisfy that CAS. This is not a retry,
/// permission refresh, corruption fallback, or post-publication classification.
pub(crate) fn load_for_update<T:Document>(path:&Path,expected:&str)->Result<Snapshot<T>,&'static str>{
    let snapshot=load_at::<T>(path).map_err(cas_read_error)?;
    if snapshot.revision!=expected{return Err("registryConflict");}Ok(snapshot)
}
fn cas_read_error(reason:&'static str)->&'static str{if reason=="registryChanged"{"registryConflict"}else{reason}}
/// Immutable bounded retained data; no authority or replay semantics. The caller
/// holds its operation lock. Incomplete new files stay as data and count to quota.
pub(crate)fn retain_new(path:&Path,bytes:&[u8],max_entries:usize,max_bytes:usize,current:impl Fn()->Result<(),&'static str>)->Result<(),&'static str>{
 current()?;
 let parent=path.parent().ok_or("registryUnavailable")?;let ancestor=parent.parent().ok_or("registryUnavailable")?;
 let root=Directory::open(ancestor,false)?.ok_or("registryUnavailable")?;
 let folder=CString::new(parent.file_name().ok_or("registryUnavailable")?.as_bytes()).map_err(|_|"registryUnavailable")?;
 if unsafe{libc::mkdirat(root.file.as_raw_fd(),folder.as_ptr(),0o700)}!=0&&std::io::Error::last_os_error().kind()!=std::io::ErrorKind::AlreadyExists{return Err("registryUnavailable");}
 root.file.sync_all().map_err(|_|"registryCommitUncertain")?;
 let directory=Directory::open(parent,false)?.ok_or("registryUnavailable")?;
 if !root.current()||directory.file.metadata().map_err(|_|"registryUnavailable")?.mode()&0o7777!=0o700{return Err("invalidRegistryDirectory");}
 let mut total=bytes.len();let mut count=1usize;
 for row in std::fs::read_dir(format!("/proc/self/fd/{}",directory.file.as_raw_fd())).map_err(|_|"registryUnavailable")?{
  let name=row.map_err(|_|"registryUnavailable")?.file_name();let node=directory.open_file(&name,libc::O_PATH,0).map_err(|_|"registryUnavailable")?;
  let meta=node.metadata().map_err(|_|"registryUnavailable")?;if !private_file(&meta){return Err("invalidRegistryFile");}
  total=total.checked_add(meta.len().try_into().map_err(|_|"registryLimit")?).ok_or("registryLimit")?;count=count.checked_add(1).ok_or("registryLimit")?;
  if count>max_entries||total>max_bytes{return Err("registryLimit");}
 }
 if count>max_entries||total>max_bytes{return Err("registryLimit");}
 current()?;if !root.current()||!directory.current(){return Err("registryChanged");}
 let name=path.file_name().ok_or("registryUnavailable")?;let mut file=directory.open_file(name,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600).map_err(|_|"registryUnavailable")?;
 file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|_|"registryCommitUncertain")?;
 let own=stamp(&file.metadata().map_err(|_|"registryUnavailable")?);
 let mut actual=directory.open_file(name,libc::O_RDONLY|libc::O_NONBLOCK,0).map_err(|_|"registryChanged")?;
 let mut read=vec![];(&mut actual).take(bytes.len()as u64+1).read_to_end(&mut read).map_err(|_|"registryUnavailable")?;
 if read!=bytes||stamp(&actual.metadata().map_err(|_|"registryUnavailable")?)!=own||!root.current()||!directory.current(){return Err("registryCommitUncertain");}
 directory.file.sync_all().and_then(|_|root.file.sync_all()).map_err(|_|"registryCommitUncertain")?;current()
}
/// Enumerate only this held registry parent, never a recursively selected path.
/// Callers making quota decisions also hold their document's writer lock.
pub(crate)fn sibling_names(path:&Path,limit:usize)->Result<Vec<std::ffi::OsString>,&'static str>{
 let Some(dir)=Directory::open(path.parent().ok_or("registryUnavailable")?,false)?else{return Ok(vec![]);};
 let mut names=Vec::new();
 for entry in std::fs::read_dir(format!("/proc/self/fd/{}",dir.file.as_raw_fd())).map_err(|_|"registryUnavailable")?{
  if names.len()>=limit{return Err("registryLimit");}names.push(entry.map_err(|_|"registryUnavailable")?.file_name());
 }
 if !dir.current(){return Err("registryChanged");}names.sort();Ok(names)
}
pub(crate)fn save_checked<T:Document>(path:&Path,expected:&str,config:T,review:impl FnOnce(&T,&T)->Result<(),&'static str>)->Result<Snapshot<T>,&'static str>{save_guarded(path,expected,config,review,||Ok(()))}
pub(crate)fn save_guarded<T:Document>(path:&Path,expected:&str,config:T,review:impl FnOnce(&T,&T)->Result<(),&'static str>,current:impl FnOnce()->Result<(),&'static str>)->Result<Snapshot<T>,&'static str>{save_impl(path,expected,config,review,current)}
#[cfg(test)]pub(crate)fn save_with<T:Document>(path:&Path,expected:&str,config:T,before_publish:impl FnOnce())->Result<Snapshot<T>,&'static str>{save_impl(path,expected,config,|_,_|Ok(()),||{before_publish();Ok(())})}
fn save_impl<T:Document>(path:&Path,expected:&str,config:T,review:impl FnOnce(&T,&T)->Result<(),&'static str>,before_publish:impl FnOnce()->Result<(),&'static str>)->Result<Snapshot<T>,&'static str>{
    let _first=load_for_update::<T>(path,expected)?;
    let parent=path.parent().ok_or("registryUnavailable")?;let directory=Directory::open(parent,true)?.ok_or("registryUnavailable")?;
    let lock=directory.open_file(OsStr::new(T::LOCK_NAME),libc::O_RDWR|libc::O_CREAT|libc::O_NONBLOCK,0o600).map_err(|_|"registryUnavailable")?;
    // Cooperating processes never write through a replaced registry inode.
    // No indefinite wait is allowed on the UI's configuration save path.
    let _lock=WriterLock::acquire(lock)?;
    let previous=read_in(path,directory.clone()).map_err(cas_read_error)?;if previous.revision!=expected{return Err("registryConflict");}
    let next=config.next(&previous.registry)?;review(&previous.registry,&next)?;
    let bytes=serde_json::to_vec_pretty(&next).map_err(|_|"invalidRegistry")?;
    if bytes.len()>T::MAX_BYTES{return Err("registryLimit");}
    let mut random=[0u8;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut random).map_err(|_|"registryUnavailable")?;
    let temp=std::ffi::OsString::from(format!("{}{}.tmp",T::TEMP_PREFIX,digest(&random)));
    let result=(||{
        let mut file=directory.open_file(&temp,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600).map_err(|_|"registryUnavailable")?;
        file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|_|"registryUnavailable")?;drop(file);
        before_publish()?;if !previous.unchanged(){return Err("registryConflict");}
        directory.rename(&temp,path.file_name().ok_or("registryUnavailable")?)?;
        directory.file.sync_all().map_err(|_|"registryCommitUncertain")?;
        let published=read_in(path,directory.clone()).map_err(|_|"registryCommitUncertain")?;
        if published.revision!=digest(&bytes){return Err("registryCommitUncertain");}Ok(published)
    })();
    if result.is_err(){directory.remove(&temp);}result
}


#[cfg(test)]
mod cas_read_tests {
 use super::*;
 use std::{cell::RefCell,sync::atomic::{AtomicUsize,Ordering},os::unix::fs::PermissionsExt};
 thread_local!{static DURING_VALIDATE:RefCell<Option<Box<dyn FnOnce()>>>=RefCell::new(None);}
 #[derive(Clone,serde::Serialize,serde::Deserialize)]struct Catalog{generation:u64,value:u32}
 impl Document for Catalog{
  const MAX_BYTES:usize=1024;const LOCK_NAME:&'static str=".cas-proof.lock";const TEMP_PREFIX:&'static str=".cas-proof-";
  fn empty()->Self{Self{generation:0,value:0}}
  fn validate(&self)->Result<(),&'static str>{let hook=DURING_VALIDATE.with(|h|h.borrow_mut().take());if let Some(hook)=hook{hook();}if self.value>10{return Err("invalidProofCatalog");}Ok(())}
  fn generation(&self)->u64{self.generation}
  fn next(mut self,previous:&Self)->Result<Self,&'static str>{if self.generation!=previous.generation{return Err("registryConflict");}self.generation+=1;self.validate()?;Ok(self)}
 }
 struct Fixture{root:PathBuf,path:PathBuf}
 impl Fixture{fn new()->Self{static NEXT:AtomicUsize=AtomicUsize::new(0);let root=std::env::temp_dir().join(format!("coucou-cas-read-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&root).unwrap();std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();let path=root.join("catalog.json");save_checked(&path,"missing",Catalog{generation:0,value:1},|_,_|Ok(())).unwrap();Self{root,path}}}
 impl Drop for Fixture{fn drop(&mut self){DURING_VALIDATE.with(|h|*h.borrow_mut()=None);let _=std::fs::remove_dir_all(&self.root);}}
 #[test]fn cas_read_superseded_by_real_atomic_writer_is_conflict_without_retry(){
  let f=Fixture::new();let old=load_at::<Catalog>(&f.path).unwrap();let path=f.path.clone();let expected=old.revision.clone();
  DURING_VALIDATE.with(|h|*h.borrow_mut()=Some(Box::new(move||{save_checked(&path,&expected,Catalog{generation:1,value:2},|_,_|Ok(())).unwrap();})));
  assert_eq!(load_for_update::<Catalog>(&f.path,&old.revision).err(),Some("registryConflict"));let winner=load_at::<Catalog>(&f.path).unwrap();assert_eq!((winner.registry.generation,winner.registry.value),(2,2));assert!(!old.unchanged());
  let bytes=std::fs::read(&f.path).unwrap();assert_eq!(save_checked(&f.path,&old.revision,Catalog{generation:1,value:3},|_,_|Ok(())).err(),Some("registryConflict"));assert_eq!(std::fs::read(&f.path).unwrap(),bytes);
 }
 #[test]fn cas_guard_error_is_not_relabelled_as_a_read_conflict(){
  let f=Fixture::new();let old=load_at::<Catalog>(&f.path).unwrap();let bytes=std::fs::read(&f.path).unwrap();assert_eq!(save_guarded(&f.path,&old.revision,Catalog{generation:1,value:2},|_,_|Ok(()),||Err("registryChanged")).err(),Some("registryChanged"));assert_eq!(std::fs::read(&f.path).unwrap(),bytes);
 }
 #[test]fn cas_postpublication_change_stays_uncertain_and_never_retries(){
  let f=Fixture::new();let old=load_at::<Catalog>(&f.path).unwrap();let path=f.path.clone();let root=f.root.clone();
  let result=save_guarded(&f.path,&old.revision,Catalog{generation:1,value:2},|_,_|Ok(()),||{DURING_VALIDATE.with(|h|*h.borrow_mut()=Some(Box::new(move||{let other=root.join("external-writer.json");std::fs::write(&other,serde_json::to_vec(&Catalog{generation:3,value:3}).unwrap()).unwrap();std::fs::set_permissions(&other,std::fs::Permissions::from_mode(0o600)).unwrap();std::fs::rename(other,path).unwrap();})));Ok(())});
  assert_eq!(result.err(),Some("registryCommitUncertain"));let retained=load_at::<Catalog>(&f.path).unwrap();assert_eq!((retained.registry.generation,retained.registry.value),(3,3));
 }
 #[test]fn cas_invalid_catalog_is_preserved_without_conflict_default_or_write(){
  let f=Fixture::new();let old=load_at::<Catalog>(&f.path).unwrap();std::fs::write(&f.path,b"invalid preserved input").unwrap();assert_eq!(load_for_update::<Catalog>(&f.path,&old.revision).err(),Some("invalidRegistry"));assert_eq!(save_checked(&f.path,&old.revision,Catalog{generation:1,value:2},|_,_|Ok(())).err(),Some("invalidRegistry"));assert_eq!(std::fs::read(&f.path).unwrap(),b"invalid preserved input");
 }
}
