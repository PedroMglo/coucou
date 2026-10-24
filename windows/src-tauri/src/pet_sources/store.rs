//! Bounded private metadata with optimistic revisions and atomic replacement.
//! A lease pins the file and parent identity; changing either revokes readers.
use super::model::{Registry,MAX_BYTES};
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
#[cfg(test)]
pub(super) fn save_at(path:&Path,expected:&str,config:Registry)->Result<Snapshot,&'static str>{save_checked(path,expected,config,|_,_|Ok(()))}
pub(super) fn save_checked(path:&Path,expected:&str,config:Registry,review:impl FnOnce(&Registry,&Registry)->Result<(),&'static str>)->Result<Snapshot,&'static str>{save_impl(path,expected,config,review,||{})}
#[cfg(test)]
fn save_with(path:&Path,expected:&str,config:Registry,before_publish:impl FnOnce())->Result<Snapshot,&'static str>{save_impl(path,expected,config,|_,_|Ok(()),before_publish)}
fn save_impl(path:&Path,expected:&str,config:Registry,review:impl FnOnce(&Registry,&Registry)->Result<(),&'static str>,before_publish:impl FnOnce())->Result<Snapshot,&'static str>{
    let first=load_at(path)?;if first.revision!=expected{return Err("registryConflict");}
    let parent=path.parent().ok_or("registryUnavailable")?;let directory=Directory::open(parent,true)?.ok_or("registryUnavailable")?;
    let lock=directory.open_file(OsStr::new(".sources.lock"),libc::O_RDWR|libc::O_CREAT|libc::O_NONBLOCK,0o600).map_err(|_|"registryUnavailable")?;
    // Cooperating processes never write through a replaced registry inode.
    // No indefinite wait is allowed on the UI's configuration save path.
    let _lock=WriterLock::acquire(lock)?;
    let previous=read_in(path,directory.clone())?;if previous.revision!=expected{return Err("registryConflict");}
    let next=config.next(&previous.registry)?;review(&previous.registry,&next)?;
    let bytes=serde_json::to_vec_pretty(&next).map_err(|_|"invalidRegistry")?;
    if bytes.len()>MAX_BYTES{return Err("registryLimit");}
    let mut random=[0u8;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut random).map_err(|_|"registryUnavailable")?;
    let temp=std::ffi::OsString::from(format!(".sources-{}.tmp",digest(&random)));
    let result=(||{
        let mut file=directory.open_file(&temp,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600).map_err(|_|"registryUnavailable")?;
        file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|_|"registryUnavailable")?;drop(file);
        before_publish();if !previous.unchanged(){return Err("registryConflict");}
        directory.rename(&temp,path.file_name().ok_or("registryUnavailable")?)?;
        directory.file.sync_all().map_err(|_|"registryCommitUncertain")?;
        let published=read_in(path,directory.clone()).map_err(|_|"registryCommitUncertain")?;
        if published.revision!=digest(&bytes){return Err("registryCommitUncertain");}Ok(published)
    })();
    if result.is_err(){directory.remove(&temp);}result
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::os::unix::fs::{PermissionsExt,symlink};
    struct Temp(PathBuf);
    impl Temp{fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-sources-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&path).unwrap();std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o700)).unwrap();Self(path)}fn path(&self)->PathBuf{self.0.join("sources.json")}}
    impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    fn initial()->Registry{let mut r=super::super::model::tests::fixture();r.generation=0;r.sources[0].revision=0;r}
    #[test]
    fn absence_is_read_only_and_atomic_save_survives_reopen_with_private_modes(){
        let t=Temp::new();let path=t.path();let missing=load_at(&path).unwrap();assert_eq!(missing.revision,"missing");assert!(missing.registry.sources.is_empty());assert!(std::fs::read_dir(&t.0).unwrap().next().is_none());
        let saved=save_at(&path,"missing",initial()).unwrap();assert_eq!(saved.registry.generation,1);assert_eq!(saved.registry.sources[0].revision,1);assert!(saved.unchanged());
        let loaded=load_at(&path).unwrap();assert_eq!(loaded.revision,saved.revision);assert_eq!(loaded.registry,saved.registry);assert_eq!(std::fs::metadata(&path).unwrap().mode()&0o777,0o600);assert_eq!(std::fs::metadata(t.0.join(".sources.lock")).unwrap().mode()&0o777,0o600);
    }
    #[test]
    fn stale_edits_and_generations_preserve_existing_bytes(){
        let t=Temp::new();let path=t.path();let old=save_at(&path,"missing",initial()).unwrap();let before=std::fs::read(&path).unwrap();
        assert!(matches!(save_at(&path,"missing",old.registry.clone()),Err("registryConflict")));let mut stale=old.registry.clone();stale.generation=0;assert!(matches!(save_at(&path,&old.revision,stale),Err("registryConflict")));assert_eq!(std::fs::read(&path).unwrap(),before);
        let mut next=old.registry.clone();next.sources[0].enabled=false;let next=save_at(&path,&old.revision,next).unwrap();assert!(!old.unchanged());assert!(next.unchanged());assert_eq!(next.registry.generation,2);assert_eq!(next.registry.sources[0].revision,2);
    }
    #[test]
    fn malformed_oversize_shared_and_symlink_files_never_become_empty_defaults(){
        for bytes in [b"not JSON".to_vec(),vec![b' ';MAX_BYTES+1]]{let t=Temp::new();let p=t.path();std::fs::write(&p,bytes).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o600)).unwrap();assert!(load_at(&p).is_err());assert!(save_at(&p,"missing",initial()).is_err());}
        let t=Temp::new();let p=t.path();save_at(&p,"missing",initial()).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o644)).unwrap();assert!(matches!(load_at(&p),Err("invalidRegistryFile")));
        let t=Temp::new();let p=t.path();symlink(t.0.join("not-there"),&p).unwrap();assert!(load_at(&p).is_err());assert!(save_at(&p,"missing",initial()).is_err());assert!(std::fs::symlink_metadata(&p).unwrap().is_symlink());
    }
    #[test]
    fn hardlinks_and_writable_or_replaced_directories_revoke_the_lease(){
        let t=Temp::new();let p=t.path();let old=save_at(&p,"missing",initial()).unwrap();std::fs::hard_link(&p,t.0.join("alias")).unwrap();assert!(!old.unchanged());assert!(load_at(&p).is_err());std::fs::remove_file(t.0.join("alias")).unwrap();
        let current=load_at(&p).unwrap();std::fs::set_permissions(&t.0,std::fs::Permissions::from_mode(0o777)).unwrap();assert!(!current.unchanged());assert!(load_at(&p).is_err());std::fs::set_permissions(&t.0,std::fs::Permissions::from_mode(0o700)).unwrap();
        let current=load_at(&p).unwrap();let moved=t.0.with_extension("old");std::fs::rename(&t.0,&moved).unwrap();std::fs::create_dir(&t.0).unwrap();std::fs::set_permissions(&t.0,std::fs::Permissions::from_mode(0o700)).unwrap();assert!(!current.unchanged());std::fs::remove_dir_all(moved).unwrap();
    }
    #[test]
    fn another_writer_or_late_external_edit_is_never_silently_overwritten(){
        let t=Temp::new();let p=t.path();let old=save_at(&p,"missing",initial()).unwrap();let lock=WriterLock::acquire(OpenOptions::new().read(true).write(true).open(t.0.join(".sources.lock")).unwrap()).unwrap();
        assert!(matches!(save_at(&p,&old.revision,old.registry.clone()),Err("registryBusy")));drop(lock);
        let result=save_with(&p,&old.revision,old.registry.clone(),||std::fs::write(&p,b"external reviewed edit").unwrap());assert!(matches!(result,Err("registryConflict")));assert_eq!(std::fs::read(&p).unwrap(),b"external reviewed edit");assert!(!std::fs::read_dir(&t.0).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    }
    #[test]
    fn deletion_or_inplace_changes_revoke_instead_of_reusing_old_grants(){
        let t=Temp::new();let p=t.path();let old=save_at(&p,"missing",initial()).unwrap();std::fs::remove_file(&p).unwrap();assert!(!old.unchanged());assert_eq!(load_at(&p).unwrap().revision,"missing");
        let current=save_at(&p,"missing",initial()).unwrap();let mut f=OpenOptions::new().append(true).open(&p).unwrap();f.write_all(b" ").unwrap();assert!(!current.unchanged());
    }
    #[test]
    fn completed_writer_releases_even_a_descriptor_copy_inherited_during_spawn(){
        let t=Temp::new();let path=t.0.join(".sources.lock");
        let open=||OpenOptions::new().read(true).write(true).create(true).mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(&path).unwrap();
        let lock=WriterLock::acquire(open()).unwrap();let inherited=lock.0.try_clone().unwrap();
        assert!(matches!(WriterLock::acquire(open()),Err("registryBusy")));drop(lock);
        // try_clone retains the same open-file description, as fork does.
        // Keeping it open must not retain a completed writer's lock.
        let next=WriterLock::acquire(open()).unwrap();drop(next);assert!(inherited.metadata().is_ok());drop(inherited);
    }
    #[test]
    fn competing_edits_publish_one_revision_without_losing_the_winner(){
        let t=Temp::new();let path=t.path();let old=save_at(&path,"missing",initial()).unwrap();
        let barrier=Arc::new(std::sync::Barrier::new(3));let mut writers=vec![];
        for name in ["first reviewed edit","second reviewed edit"]{
            let (path,expected,mut config,barrier)=(path.clone(),old.revision.clone(),old.registry.clone(),barrier.clone());config.sources[0].name=name.into();
            writers.push(std::thread::spawn(move||{barrier.wait();(name,save_at(&path,&expected,config))}));
        }
        barrier.wait();let results=writers.into_iter().map(|w|w.join().unwrap()).collect::<Vec<_>>();
        let winners=results.iter().filter(|(_,r)|r.is_ok()).collect::<Vec<_>>();assert_eq!(winners.len(),1);
        assert!(results.iter().filter_map(|(_,r)|r.as_ref().err()).all(|e|["registryBusy","registryConflict"].contains(e)));
        let current=load_at(&path).unwrap();assert_eq!(current.registry.generation,2);assert_eq!(current.registry.sources[0].revision,2);assert_eq!(current.registry.sources[0].name,winners[0].0);assert!(current.unchanged());assert!(!old.unchanged());
    }
    #[test]
    fn special_or_redirected_lock_files_cannot_become_writer_authority(){
        for kind in ["fifo","symlink","shared"]{
            let t=Temp::new();let lock=t.0.join(".sources.lock");
            match kind{
                "fifo"=>{let name=CString::new(lock.as_os_str().as_bytes()).unwrap();assert_eq!(unsafe{libc::mkfifo(name.as_ptr(),0o600)},0);}
                "symlink"=>symlink(t.0.join("not-a-lock"),&lock).unwrap(),
                _=>{std::fs::write(&lock,b"").unwrap();std::fs::set_permissions(&lock,std::fs::Permissions::from_mode(0o666)).unwrap();}
            }
            assert!(save_at(&t.path(),"missing",initial()).is_err(),"{kind}");assert!(!t.path().exists());
        }
    }
}
