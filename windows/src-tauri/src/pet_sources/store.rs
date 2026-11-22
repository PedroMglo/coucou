//! Calendar metadata adapter for the shared private CAS store. Format, limits,
//! names and errors remain unchanged.
use super::model::{Registry,MAX_BYTES};
use std::path::Path;
impl crate::private_registry::Document for Registry{
 const MAX_BYTES:usize=MAX_BYTES;const LOCK_NAME:&'static str=".sources.lock";const TEMP_PREFIX:&'static str=".sources-";
 fn empty()->Self{Registry::empty()}fn validate(&self)->Result<(),&'static str>{Registry::validate(self)}
 fn generation(&self)->u64{self.generation}fn next(self,previous:&Self)->Result<Self,&'static str>{Registry::next(self,previous)}
}
pub(super)type Snapshot=crate::private_registry::Snapshot<Registry>;
pub(super)fn load_at(path:&Path)->Result<Snapshot,&'static str>{crate::private_registry::load_at(path)}
pub(super)fn save_checked(path:&Path,expected:&str,config:Registry,review:impl FnOnce(&Registry,&Registry)->Result<(),&'static str>)->Result<Snapshot,&'static str>{crate::private_registry::save_checked(path,expected,config,review)}
#[cfg(test)]pub(super)fn save_at(path:&Path,expected:&str,config:Registry)->Result<Snapshot,&'static str>{save_checked(path,expected,config,|_,_|Ok(()))}
#[cfg(test)]use crate::private_registry::{WriterLock,save_with};
#[cfg(test)]use std::{ffi::CString,fs::OpenOptions,io::Write,os::unix::{ffi::OsStrExt,fs::{MetadataExt,OpenOptionsExt}},path::PathBuf,sync::Arc};
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
