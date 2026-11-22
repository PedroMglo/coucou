//! Destination identity and source snapshots have different mutation rules.
use super::*;
use super::super::staging_fs::Dir;
use std::{fs,os::unix::fs::{PermissionsExt,symlink},path::PathBuf,sync::atomic::{AtomicU64,Ordering}};
struct Fixture(PathBuf);
impl Fixture{fn new()->Self{static NEXT:AtomicU64=AtomicU64::new(0);let p=std::env::temp_dir().join(format!("coucou-directory-binding-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir(&p).unwrap();fs::set_permissions(&p,fs::Permissions::from_mode(0o700)).unwrap();Self(p)}}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
#[test]fn legitimate_payload_mutation_between_pin_metadata_and_reopen_keeps_its_directory_binding(){
 let f=Fixture::new();let pinned=root_pin(&f.0).unwrap().unwrap();let original=location(&pinned).unwrap();
 let node=readable_directory_with(pinned,||fs::create_dir(f.0.join("new-runtime-store")).unwrap()).unwrap();assert_eq!(location(&node.file).unwrap(),original);
 let bound=Dir::root(&f.0).unwrap();for index in 0..16{let p=f.0.join(format!("live-{index}"));fs::create_dir(&p).unwrap();bound.check().unwrap();fs::remove_dir(p).unwrap();bound.check().unwrap();}
}
#[test]fn source_directory_and_regular_file_snapshots_still_reject_content_changes(){
 let f=Fixture::new();let source=Node::root(&f.0).unwrap().unwrap();let file=f.0.join("source.json");fs::write(&file,b"original").unwrap();assert!(matches!(source.current(),Err(Error::SourceChanged)));
 let parent=Node::root(&f.0).unwrap().unwrap();let original=parent.child(OsStr::new("source.json")).unwrap().unwrap();fs::write(file,b"changed source bytes").unwrap();assert!(matches!(original.current(),Err(Error::SourceChanged)));
}
#[test]fn directory_binding_does_not_accept_mode_changes_or_non_directory_nodes(){
 let f=Fixture::new();let pinned=root_pin(&f.0).unwrap().unwrap();assert!(matches!(readable_directory_with(pinned,||fs::set_permissions(&f.0,fs::Permissions::from_mode(0o750)).unwrap()),Err(Error::SourceChanged)));
 fs::set_permissions(&f.0,fs::Permissions::from_mode(0o700)).unwrap();fs::write(f.0.join("regular"),b"data").unwrap();let parent=bound_directory_root(&f.0).unwrap().unwrap();assert!(bound_directory_child(&parent.file,OsStr::new("regular")).is_err());
}
#[test]fn a_live_binding_rejects_named_directory_replacement_and_symlinks(){
 let f=Fixture::new();let path=f.0.join("payload");fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();let bound=Dir::root(&path).unwrap();fs::rename(&path,f.0.join("original-payload")).unwrap();fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();assert!(matches!(bound.check(),Err(Error::SourceChanged)));
 let parent=Dir::root(&f.0).unwrap();symlink(&path,f.0.join("linked-payload")).unwrap();assert!(parent.dir(OsStr::new("linked-payload")).is_err());assert!(Dir::root(&f.0.join("linked-payload")).is_err());
}
