use super::*;
use std::{fs,os::unix::fs::PermissionsExt,sync::atomic::{AtomicUsize,Ordering}};
struct Fixture{root:PathBuf,config:PathBuf,exe:PathBuf}
impl Fixture{fn new()->Self{static N:AtomicUsize=AtomicUsize::new(0);let root=std::env::temp_dir().join(format!("coucou-autostart-plan-{}-{}",std::process::id(),N.fetch_add(1,Ordering::Relaxed)));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();let config=root.join("config");fs::create_dir(&config).unwrap();let exe=root.join("native launcher");fs::write(&exe,b"#!/bin/sh\nexit 0\n").unwrap();fs::set_permissions(&exe,fs::Permissions::from_mode(0o700)).unwrap();Self{root,config,exe}}}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
#[test]fn autostart_plan_capture_is_passive_and_owns_exact_resource_before_parent_creation(){let f=Fixture::new();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();assert!(plan.view.creates_parent);assert_eq!(plan.view.postcondition,"desktopEntryMatches");assert!(!f.config.join("autostart").exists());let observed=plan.execute(||Ok(())).unwrap();assert!(observed.enabled&&!observed.login_verified);assert_eq!(fs::read(f.config.join("autostart/Coucou.desktop")).unwrap(),plan.expected().unwrap());assert!(!plan.current());}
#[test]fn autostart_plan_enable_disable_and_noop_have_exact_postconditions(){let f=Fixture::new();let plan=Plan::capture(&f.config,&f.exe,false).unwrap();assert!(plan.already_matches());plan.execute(||Ok(())).unwrap();assert!(!f.config.join("autostart").exists());Plan::capture(&f.config,&f.exe,true).unwrap().execute(||Ok(())).unwrap();let same=Plan::capture(&f.config,&f.exe,true).unwrap();let before=fs::metadata(same.target.path()).unwrap().ino();assert!(same.already_matches());same.execute(||Ok(())).unwrap();assert_eq!(fs::metadata(same.target.path()).unwrap().ino(),before);let remove=Plan::capture(&f.config,&f.exe,false).unwrap();let observation=remove.execute(||Ok(())).unwrap();assert!(!observation.enabled);assert!(!f.config.join("autostart/Coucou.desktop").exists());}
#[test]fn autostart_plan_refuses_destination_or_launcher_substitution_after_review(){for change_launcher in [false,true]{let f=Fixture::new();fs::create_dir(f.config.join("autostart")).unwrap();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();if change_launcher{fs::write(&f.exe,b"different launcher").unwrap();}else{fs::write(plan.target.path(),b"foreign entry").unwrap();}assert!(plan.execute(||Ok(())).is_err());if !change_launcher{assert_eq!(fs::read(plan.target.path()).unwrap(),b"foreign entry");}}}
#[test]fn autostart_plan_source_scope_is_frozen_and_environment_cannot_redirect_it(){let f=Fixture::new();let other=f.root.join("other");fs::create_dir(&other).unwrap();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();plan.execute(||Ok(())).unwrap();assert!(!other.join("autostart").exists());assert_eq!(plan.view.target,f.config.join("autostart/Coucou.desktop").to_string_lossy());}
#[test]fn autostart_plan_denial_before_dispatch_creates_no_directory_or_entry(){let f=Fixture::new();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();assert!(plan.execute(||Err(io::Error::other("review revoked"))).is_err());assert!(!f.config.join("autostart").exists());}
#[test]fn autostart_plan_late_guard_refusal_keeps_previous_entry_and_partial_parent_is_not_claimed_rolled_back(){let f=Fixture::new();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();let calls=AtomicUsize::new(0);assert!(plan.execute(||if calls.fetch_add(1,Ordering::Relaxed)>=3{Err(io::Error::other("late revocation"))}else{Ok(())}).is_err());assert!(f.config.join("autostart").is_dir());assert!(!f.config.join("autostart/Coucou.desktop").exists());}
#[test]fn autostart_plan_symlink_fifo_or_foreign_parent_is_refused_without_following_it(){let f=Fixture::new();let other=f.root.join("other");fs::create_dir(&other).unwrap();std::os::unix::fs::symlink(&other,f.config.join("autostart")).unwrap();assert!(Plan::capture(&f.config,&f.exe,true).is_err());assert_eq!(fs::read_dir(&other).unwrap().count(),0);fs::remove_file(f.config.join("autostart")).unwrap();fs::write(f.config.join("autostart"),b"not a directory").unwrap();assert!(Plan::capture(&f.config,&f.exe,true).is_err());}

#[test]fn autostart_plan_fingerprints_and_content_digests_are_sha256(){assert_eq!(hash(b"abc"),"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");let f=Fixture::new();let p=Plan::capture(&f.config,&f.exe,true).unwrap();for value in [&p.view.fingerprint,&p.view.launcher_identity,p.view.expected_sha256.as_ref().unwrap()]{assert_eq!(value.len(),64);assert!(value.bytes().all(|b|b.is_ascii_hexdigit()));}}

#[test]fn autostart_plan_confirmation_rejects_same_bytes_in_a_replaced_file_parent_or_root(){
 for mode in 0..3{let f=Fixture::new();let plan=Plan::capture(&f.config,&f.exe,true).unwrap();let bytes=plan.expected().unwrap().to_vec();
  let result=plan.execute_with(||Ok(()),||{let entry=f.config.join("autostart/Coucou.desktop");match mode{
   0=>{let temp=f.config.join("autostart/replacement");fs::write(&temp,&bytes).unwrap();fs::rename(temp,entry).unwrap();},
   1=>{fs::rename(f.config.join("autostart"),f.root.join("old-autostart")).unwrap();fs::create_dir(f.config.join("autostart")).unwrap();fs::write(entry,&bytes).unwrap();},
   _=>{fs::rename(&f.config,f.root.join("old-config")).unwrap();fs::create_dir(&f.config).unwrap();fs::create_dir(f.config.join("autostart")).unwrap();fs::write(entry,&bytes).unwrap();}
  }});assert!(result.is_err(),"replacement mode {mode} falsely confirmed");assert_eq!(fs::read(f.config.join("autostart/Coucou.desktop")).unwrap(),bytes);
 }
}

#[test]fn autostart_plan_persisted_resource_distinguishes_a_replaced_base_at_the_same_path(){
 let f=Fixture::new();let original=Plan::capture(&f.config,&f.exe,true).unwrap();original.execute(||Ok(())).unwrap();
 let old=Plan::capture(&f.config,&f.exe,true).unwrap();let old_base=fs::metadata(&f.config).unwrap();let bytes=fs::read(f.config.join("autostart/Coucou.desktop")).unwrap();
 // Ordinary contents/child changes do not replace the base resource.
 fs::write(f.config.join("unrelated-preference"),b"changed").unwrap();
 assert_eq!(Plan::capture(&f.config,&f.exe,true).unwrap().view.resource,old.view.resource);
 // Preserve the original inode offline; recreate the same named location with
 // identical entry bytes. Both captures use the actual native executor plan.
 fs::rename(&f.config,f.root.join("retained-base")).unwrap();fs::create_dir(&f.config).unwrap();fs::create_dir(f.config.join("autostart")).unwrap();fs::write(f.config.join("autostart/Coucou.desktop"),&bytes).unwrap();
 let new_base=fs::metadata(&f.config).unwrap();assert_ne!((old_base.dev(),old_base.ino()),(new_base.dev(),new_base.ino()));
 assert!(!old.current());let current=Plan::capture(&f.config,&f.exe,true).unwrap();assert!(current.current()&&current.already_matches());
 assert_eq!(old.view.target,current.view.target);assert_eq!(old.expected(),current.expected());
 assert_ne!(old.view.resource,current.view.resource,"a persisted path digest must not identify a replacement directory as the old physical resource");
 assert_eq!(fs::read(f.root.join("retained-base/autostart/Coucou.desktop")).unwrap(),bytes);
}

#[test]fn autostart_plan_physical_resource_survives_base_rename_and_ordinary_changes(){
 let f=Fixture::new();let old=Plan::capture(&f.config,&f.exe,false).unwrap();let moved=f.root.join("renamed-base");fs::rename(&f.config,&moved).unwrap();let current=Plan::capture(&moved,&f.exe,false).unwrap();assert_eq!(old.view.resource,current.view.resource);assert_ne!(old.view.target,current.view.target);assert!(!old.current());fs::write(moved.join("unrelated"),b"ordinary metadata change").unwrap();assert_eq!(Plan::capture(&moved,&f.exe,false).unwrap().view.resource,current.view.resource);assert!(current.current());
}
