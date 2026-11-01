//! Immutable IO format, not an execution grant. Mutable outcomes live in payload.
use super::{publication::{self,Storage},staging_fs::Dir,Error,Result};
use std::path::Path;
pub(super)const CONTROL:&str=".settings-io-v1.json";
pub(super)const CONTROL_V2:&str=".settings-io-v2.json";
pub(super)const CONTROL_V3:&str=".settings-io-v3.json";
pub(super)const CONTROL_V4:&str=".settings-io-v4.json";
pub(super)const CONTROL_V5:&str=".settings-io-v5.json";
pub(super)const CONTROL_V6:&str=".settings-io-v6.json";
pub(super)const CONTROL_V7:&str=".settings-io-v7.json";
pub(super)const CONTROL_V8:&str=".settings-io-v8.json";
const BYTES_V8:&[u8]=b"{\"version\":8}";
const BYTES_V7:&[u8]=b"{\"version\":7}";
const BYTES_V6:&[u8]=b"{\"version\":6}";
const BYTES_V5:&[u8]=b"{\"version\":5}";
const BYTES_V4:&[u8]=b"{\"version\":4}";
const BYTES_V3:&[u8]=b"{\"version\":3}";
const BYTES:&[u8]=b"{\"version\":1}";
const BYTES_V2:&[u8]=b"{\"version\":2}";
#[derive(Clone)]pub(crate)struct Format{root:Dir,version:Option<u32>}
impl Format{
 pub(super)fn capture(root:&Dir)->Result<Self>{Ok(Self{root:root.clone(),version:version(root)?})}
 pub(crate)fn scope(&self)->Result<String>{
  self.root.check()?;use std::os::unix::{ffi::OsStrExt,fs::MetadataExt};
  let path=self.root.bound_path()?;let node=self.root.node()?;let m=node.file.metadata().map_err(|_|Error::Unavailable)?;
  let mut bytes=b"coucou-settings-native-scope-v1\0".to_vec();bytes.extend_from_slice(path.as_os_str().as_bytes());bytes.push(0);bytes.extend_from_slice(&m.dev().to_le_bytes());bytes.extend_from_slice(&m.ino().to_le_bytes());
  for name in [publication::HEADER,publication::ADMITTED]{let value=publication::read(&self.root,name)?.ok_or(Error::UnsupportedProfile)?;bytes.extend_from_slice(&(value.len()as u64).to_le_bytes());bytes.extend_from_slice(&value);}
  self.root.check()?;Ok(ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect())
 }
 pub(crate)fn present(&self)->bool{self.version.is_some()}
 pub(crate)fn version(&self)->Option<u32>{self.version}
 pub(crate)fn current(&self)->bool{self.root.check().is_ok()&&version(&self.root).is_ok_and(|v|v==self.version)}
}
pub(super)fn control(name:&str)->bool{matches!(name,CONTROL|CONTROL_V2|CONTROL_V3|CONTROL_V4|CONTROL_V5|CONTROL_V6|CONTROL_V7|CONTROL_V8)}
pub(super)fn version(root:&Dir)->Result<Option<u32>>{
 let eighth=publication::read(root,CONTROL_V8)?;
 if let Some(bytes)=eighth{if bytes!=BYTES_V8{return Err(Error::UnsupportedProfile);}for(name,expected)in extra(Some(7))?{if publication::read(root,name)?.as_deref()!=Some(expected){return Err(Error::UnsupportedProfile);}}return Ok(Some(8));}
 let seventh=publication::read(root,CONTROL_V7)?;
 if let Some(bytes)=seventh{if bytes!=BYTES_V7{return Err(Error::UnsupportedProfile);}for(name,expected)in extra(Some(6))?{if publication::read(root,name)?.as_deref()!=Some(expected){return Err(Error::UnsupportedProfile);}}return Ok(Some(7));}
 let sixth=publication::read(root,CONTROL_V6)?;
 if let Some(bytes)=sixth{if bytes!=BYTES_V6{return Err(Error::UnsupportedProfile);}for(name,expected)in[(CONTROL,BYTES),(CONTROL_V2,BYTES_V2),(CONTROL_V3,BYTES_V3),(CONTROL_V4,BYTES_V4),(CONTROL_V5,BYTES_V5)]{if publication::read(root,name)?.as_deref()!=Some(expected){return Err(Error::UnsupportedProfile);}}return Ok(Some(6));}
 let first=publication::read(root,CONTROL)?;let second=publication::read(root,CONTROL_V2)?;let third=publication::read(root,CONTROL_V3)?;let fourth=publication::read(root,CONTROL_V4)?;let fifth=publication::read(root,CONTROL_V5)?;
 match(first.as_deref(),second.as_deref(),third.as_deref(),fourth.as_deref(),fifth.as_deref()){
  (None,None,None,None,None)=>Ok(None),(Some(BYTES),None,None,None,None)=>Ok(Some(1)),(Some(BYTES),Some(BYTES_V2),None,None,None)=>Ok(Some(2)),(Some(BYTES),Some(BYTES_V2),Some(BYTES_V3),None,None)=>Ok(Some(3)),(Some(BYTES),Some(BYTES_V2),Some(BYTES_V3),Some(BYTES_V4),None)=>Ok(Some(4)),(Some(BYTES),Some(BYTES_V2),Some(BYTES_V3),Some(BYTES_V4),Some(BYTES_V5))=>Ok(Some(5)),_=>Err(Error::UnsupportedProfile),
 }
}
pub(super)fn present(root:&Dir)->Result<bool>{Ok(version(root)?.is_some())}
impl Storage{
 pub(super)fn settings_io_observation(&self,path:&Path)->Result<Format>{
  self.current()?;if self.paths()?.config.join("settings.json")!=path{return Err(Error::UnsafePath);}Format::capture(&self.roots.config)
 }
 pub(super)fn settings_io_protocol(&self,path:&Path,create:bool)->Result<bool>{
  if create{self.settings_io_upgrade(path,1)?;}else{self.current()?;if self.paths()?.config.join("settings.json")!=path{return Err(Error::UnsafePath);}}
  present(&self.roots.config)
 }
 pub(super)fn settings_io_upgrade(&self,path:&Path,wanted:u32)->Result<()>{
  self.current()?;if !matches!(wanted,1|2|3|4|5|6|7|8)||self.paths()?.config.join("settings.json")!=path{return Err(Error::UnsafePath);}
  let current=version(&self.roots.config)?;
  if current.is_none(){self.roots.config.publish_new(CONTROL,BYTES)?;self.current()?;}
  if wanted>=2&&version(&self.roots.config)?.is_none_or(|v|v<2){self.roots.config.publish_new(CONTROL_V2,BYTES_V2)?;self.current()?;}
  if wanted>=3&&version(&self.roots.config)?.is_none_or(|v|v<3){self.roots.config.publish_new(CONTROL_V3,BYTES_V3)?;self.current()?;}
  if wanted>=4&&version(&self.roots.config)?.is_none_or(|v|v<4){self.roots.config.publish_new(CONTROL_V4,BYTES_V4)?;self.current()?;}
  if wanted>=5&&version(&self.roots.config)?.is_none_or(|v|v<5){self.roots.config.publish_new(CONTROL_V5,BYTES_V5)?;self.current()?;}
  if wanted>=6&&version(&self.roots.config)?.is_none_or(|v|v<6){self.roots.config.publish_new(CONTROL_V6,BYTES_V6)?;self.current()?;}
  if wanted>=7&&version(&self.roots.config)?.is_none_or(|v|v<7){self.roots.config.publish_new(CONTROL_V7,BYTES_V7)?;self.current()?;}
  if wanted>=8&&version(&self.roots.config)?.is_none_or(|v|v<8){self.roots.config.publish_new(CONTROL_V8,BYTES_V8)?;self.current()?;}
  if version(&self.roots.config)?.is_none_or(|v|v<wanted){return Err(Error::SourceChanged);}Ok(())
 }
}
pub(super)fn extra(version:Option<u32>)->Result<Vec<(&'static str,&'static[u8])>>{
 if version==Some(8){let mut controls=extra(Some(7))?;controls.push((CONTROL_V8,BYTES_V8));return Ok(controls);}
 if version==Some(7){let mut controls=extra(Some(6))?;controls.push((CONTROL_V7,BYTES_V7));return Ok(controls);}
 if version==Some(6){let mut controls=extra(Some(5))?;controls.push((CONTROL_V6,BYTES_V6));return Ok(controls);}
 match version{None=>Ok(vec![]),Some(1)=>Ok(vec![(CONTROL,BYTES)]),Some(2)=>Ok(vec![(CONTROL,BYTES),(CONTROL_V2,BYTES_V2)]),Some(3)=>Ok(vec![(CONTROL,BYTES),(CONTROL_V2,BYTES_V2),(CONTROL_V3,BYTES_V3)]),Some(4)=>Ok(vec![(CONTROL,BYTES),(CONTROL_V2,BYTES_V2),(CONTROL_V3,BYTES_V3),(CONTROL_V4,BYTES_V4)]),Some(5)=>Ok(vec![(CONTROL,BYTES),(CONTROL_V2,BYTES_V2),(CONTROL_V3,BYTES_V3),(CONTROL_V4,BYTES_V4),(CONTROL_V5,BYTES_V5)]),_=>Err(Error::UnsupportedProfile)}
}
#[cfg(test)]mod tests{
    use super::*;use crate::pet_data::{tests::Temp,startup::tests::Harness,publication::Publication};
    #[test]fn shortcut_scope_recovery_v8_requires_all_controls_keeps_scope_and_refuses_downgrade(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_upgrade(&path,7).unwrap();let old=storage.settings_io_observation(&path).unwrap();let scope=old.scope().unwrap();storage.settings_io_upgrade(&path,8).unwrap();assert!(!old.current());let current=storage.settings_io_observation(&path).unwrap();assert_eq!(current.version(),Some(8));assert_eq!(current.scope().unwrap(),scope);storage.settings_io_upgrade(&path,7).unwrap();assert!(current.current());
        for(name,bytes)in extra(Some(8)).unwrap(){assert_eq!(std::fs::read(layout.config().join(name)).unwrap(),bytes);}
        std::fs::remove_file(layout.config().join(CONTROL_V7)).unwrap();assert!(!current.current());assert!(version(&storage.roots.config).is_err());assert!(storage.settings_io_upgrade(&path,8).is_err());
    }
    #[test]fn shortcut_continuity_v7_requires_complete_controls_keeps_scope_and_refuses_downgrade(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_upgrade(&path,6).unwrap();let old=storage.settings_io_observation(&path).unwrap();let scope=old.scope().unwrap();storage.settings_io_upgrade(&path,7).unwrap();assert!(!old.current());let current=storage.settings_io_observation(&path).unwrap();assert_eq!(current.version(),Some(7));assert_eq!(current.scope().unwrap(),scope);storage.settings_io_upgrade(&path,6).unwrap();assert!(current.current());
        for(name,bytes)in extra(Some(7)).unwrap(){assert_eq!(std::fs::read(layout.config().join(name)).unwrap(),bytes);}
        std::fs::remove_file(layout.config().join(CONTROL_V6)).unwrap();assert!(!current.current());assert!(version(&storage.roots.config).is_err());assert!(storage.settings_io_upgrade(&path,7).is_err());
    }
    #[test]fn physical_component_v6_keeps_scope_requires_all_predecessors_and_cannot_downgrade(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_upgrade(&path,5).unwrap();let old=storage.settings_io_observation(&path).unwrap();let scope=old.scope().unwrap();storage.settings_io_upgrade(&path,6).unwrap();assert!(!old.current());let new=storage.settings_io_observation(&path).unwrap();assert_eq!(new.version(),Some(6));assert_eq!(new.scope().unwrap(),scope);storage.settings_io_upgrade(&path,5).unwrap();assert!(new.current());assert_eq!(extra(Some(6)).unwrap().len(),6);std::fs::remove_file(layout.config().join(CONTROL_V5)).unwrap();assert!(!new.current());assert!(version(&storage.roots.config).is_err());assert!(storage.settings_io_upgrade(&path,6).is_err());
    }
    #[test]fn component_journal_v5_keeps_scope_requires_chain_and_cannot_downgrade_to_old_semantics(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");
        storage.settings_io_upgrade(&path,4).unwrap();let old=storage.settings_io_observation(&path).unwrap();let scope=old.scope().unwrap();storage.settings_io_upgrade(&path,5).unwrap();assert!(!old.current());let current=storage.settings_io_observation(&path).unwrap();assert_eq!(current.version(),Some(5));assert_eq!(current.scope().unwrap(),scope);storage.settings_io_upgrade(&path,4).unwrap();assert!(current.current());assert_eq!(extra(Some(5)).unwrap().len(),5);
        std::fs::remove_file(layout.config().join(CONTROL_V4)).unwrap();assert!(!current.current());assert!(version(&storage.roots.config).is_err());assert!(storage.settings_io_upgrade(&path,5).is_err());
    }
    #[test]fn review_foundation_v2_marker_revokes_v1_observation_and_root_controls_keep_version(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");
        storage.settings_io_protocol(&path,true).unwrap();let first=storage.settings_io_observation(&path).unwrap();assert_eq!(first.version(),Some(1));storage.settings_io_upgrade(&path,2).unwrap();assert!(!first.current());let second=storage.settings_io_observation(&path).unwrap();assert!(second.current());assert_eq!(second.version(),Some(2));assert_eq!(version(&storage.roots.config).unwrap(),Some(2));assert_eq!(extra(Some(2)).unwrap().len(),2);assert_eq!(extra(Some(3)).unwrap().len(),3);assert_eq!(extra(Some(4)).unwrap().len(),4);assert_eq!(extra(Some(5)).unwrap().len(),5);assert_eq!(extra(Some(6)).unwrap().len(),6);assert_eq!(extra(Some(7)).unwrap().len(),7);assert_eq!(extra(Some(8)).unwrap().len(),8);assert!(extra(Some(9)).is_err());
    }
    #[test]fn review_foundation_v2_without_original_marker_or_wrong_bytes_is_not_admitted(){
        let t=Temp::new();let path=t.0.join("marker-root");std::fs::create_dir(&path).unwrap();use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o700)).unwrap();let root=Dir::root(&path).unwrap();root.publish_new(CONTROL_V2,BYTES_V2).unwrap();assert!(version(&root).is_err());
    }
    #[test]fn component_journal_v3_scope_is_stable_for_writes_and_distinct_for_another_admission(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");
        let original=storage.settings_io_observation(&path).unwrap();let scope=original.scope().unwrap();assert_eq!(scope.len(),64);
        storage.settings_io_upgrade(&path,3).unwrap();assert!(!original.current());let current=storage.settings_io_observation(&path).unwrap();assert_eq!(current.version(),Some(3));assert_eq!(current.scope().unwrap(),scope);
        std::fs::write(&path,b"ordinary preference write").unwrap();assert_eq!(current.scope().unwrap(),scope);storage.settings_io_upgrade(&path,2).unwrap();assert!(current.current());assert_eq!(extra(Some(3)).unwrap().len(),3);
        let other=Temp::new();let layout=other.basic();let h=Harness::new();let other=Publication::begin(h.claim(&layout).unwrap(),layout).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let other_path=other.paths().unwrap().config.join("settings.json");assert_ne!(other.settings_io_observation(&other_path).unwrap().scope().unwrap(),scope);
    }
    #[test]fn component_journal_v3_missing_predecessor_markers_and_unknown_bytes_are_refused(){
        for predecessors in [0,1]{let t=Temp::new();let path=t.0.join("component-marker-root");std::fs::create_dir(&path).unwrap();use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o700)).unwrap();let root=Dir::root(&path).unwrap();if predecessors==1{root.publish_new(CONTROL,BYTES).unwrap();}root.publish_new(CONTROL_V3,BYTES_V3).unwrap();assert!(version(&root).is_err());}
    }
    #[test]fn component_journal_v4_requires_complete_chain_revokes_v3_and_never_downgrades(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let storage=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap().into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");
        storage.settings_io_upgrade(&path,3).unwrap();let previous=storage.settings_io_observation(&path).unwrap();let scope=previous.scope().unwrap();storage.settings_io_upgrade(&path,4).unwrap();assert!(!previous.current());let current=storage.settings_io_observation(&path).unwrap();assert_eq!(current.version(),Some(4));assert_eq!(current.scope().unwrap(),scope);storage.settings_io_upgrade(&path,3).unwrap();storage.settings_io_upgrade(&path,2).unwrap();
        assert!(current.current());assert_eq!(extra(Some(4)).unwrap().len(),4);
    }
    #[test]fn component_journal_v4_without_v3_is_not_a_supported_profile(){
        let t=Temp::new();let path=t.0.join("v4-marker-root");std::fs::create_dir(&path).unwrap();use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o700)).unwrap();let root=Dir::root(&path).unwrap();root.publish_new(CONTROL,BYTES).unwrap();root.publish_new(CONTROL_V2,BYTES_V2).unwrap();root.publish_new(CONTROL_V4,BYTES_V4).unwrap();assert!(version(&root).is_err());
    }
    #[test]fn settings_durable_format_witness_checks_native_bytes_without_waiting_on_profile_mutex(){
        let t=Temp::new();let layout=t.basic();let h=Harness::new();let active=Publication::begin(h.claim(&layout).unwrap(),layout.clone()).ok().unwrap().publish().ok().unwrap();
        let storage=active.into_storage().unwrap();let path=storage.paths().unwrap().config.join("settings.json");storage.settings_io_protocol(&path,true).unwrap();
        let bound=storage.settings_io_observation(&path).unwrap();let owner=std::sync::Mutex::new(storage);let _held=owner.lock().unwrap();assert!(bound.current());assert!(bound.present());
        std::fs::remove_file(layout.config().join(CONTROL)).unwrap();assert!(!bound.current());
    }
}
