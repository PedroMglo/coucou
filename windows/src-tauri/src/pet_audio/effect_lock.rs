//! Audio exclusion is mechanical and never grants caller authority.
use std::{path::{Path,PathBuf},sync::Arc};
pub(crate)use crate::private_registry::InheritedLock as Guard;
const NAME:&str=".coucou-audio-effect-v1.lock";
pub(crate)fn runtime()->PathBuf{std::env::var_os("PIPEWIRE_RUNTIME_DIR").or_else(||std::env::var_os("XDG_RUNTIME_DIR")).map(PathBuf::from).unwrap_or_else(||PathBuf::from(format!("/run/user/{}",unsafe{libc::getuid()})))}
pub(crate)fn acquire_at(runtime:&Path)->Result<Arc<Guard>,String>{crate::private_registry::inherited_operation_lock(runtime,NAME).map_err(|e|match e{"registryBusy"=>"O efeito de Som anterior ainda está ocupado ou a terminar.".into(),_=>"A reserva física do Som não pôde ser validada; não foi escolhido outro runtime.".into()})}
pub(crate)fn acquire()->Result<Arc<Guard>,String>{acquire_at(&runtime())}

#[cfg(test)]mod tests{
 use super::*;
 use std::{fs,io,os::{fd::AsRawFd,unix::fs::{DirBuilderExt,OpenOptionsExt,PermissionsExt,MetadataExt,symlink}},process::{Command,Stdio},time::{Duration,Instant}};
 struct Fixture(PathBuf);
 impl Fixture{fn new()->Self{let mut bytes=[0u8;16];ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(),&mut bytes).unwrap();let nonce=bytes.iter().map(|v|format!("{v:02x}")).collect::<String>();let p=std::env::temp_dir().join(format!("coucou-effect-lock-{nonce}"));fs::DirBuilder::new().mode(0o700).create(&p).unwrap();Self(p)}}
 impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
 fn private(path:&Path,bytes:&[u8]){use std::io::Write;let mut f=fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(path).unwrap();f.write_all(bytes).unwrap();}
 #[test]fn inherited_audio_lock_is_exclusive_per_runtime_and_parent_descriptor_stays_cloexec(){
  let a=Fixture::new();let b=Fixture::new();let guard=acquire_at(&a.0).unwrap();assert!(guard.current());assert!(acquire_at(&a.0).is_err());let other=acquire_at(&b.0).unwrap();let file=guard.file();assert_ne!(unsafe{libc::fcntl(file.as_raw_fd(),libc::F_GETFD)}&libc::FD_CLOEXEC,0);drop(guard);assert!(acquire_at(&a.0).is_err(),"Last descriptor, not Arc guard object, owns kernel lock");drop(file);assert!(acquire_at(&a.0).is_ok());assert!(other.current());
 }
 #[test]fn inherited_audio_lock_rejects_symlink_hardlink_nonempty_and_public_nodes(){
  for mode in["symlink","hardlink","nonempty","public","fifo"]{let f=Fixture::new();let outside=f.0.join("outside");private(&outside,b"");let node=f.0.join(NAME);match mode{"symlink"=>symlink(&outside,&node).unwrap(),"hardlink"=>fs::hard_link(&outside,&node).unwrap(),"nonempty"=>private(&node,b"unknown"),"public"=>{private(&node,b"");fs::set_permissions(&node,fs::Permissions::from_mode(0o644)).unwrap();},_=>{use std::os::unix::ffi::OsStrExt;let c=std::ffi::CString::new(node.as_os_str().as_bytes()).unwrap();assert_eq!(unsafe{libc::mkfifo(c.as_ptr(),0o600)},0);}}assert!(acquire_at(&f.0).is_err(),"{mode}");assert_eq!(fs::read(&outside).unwrap(),b"");}
  let f=Fixture::new();fs::set_permissions(&f.0,fs::Permissions::from_mode(0o755)).unwrap();assert!(acquire_at(&f.0).is_err());
 }
 #[test]fn inherited_audio_lock_detects_replaced_file_and_directory_without_recapture(){
  let f=Fixture::new();let dir=f.0.join("runtime");fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();let guard=acquire_at(&dir).unwrap();fs::rename(dir.join(NAME),dir.join("original")).unwrap();private(&dir.join(NAME),b"");assert!(!guard.current());drop(guard);let guard=acquire_at(&dir).unwrap();fs::rename(&dir,f.0.join("original-runtime")).unwrap();fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();assert!(!guard.current());
 }
 #[test]fn ordinary_registry_writer_keeps_explicit_unlock_even_with_a_fork_style_copy(){
  let f=Fixture::new();let lock=crate::private_registry::operation_lock(&f.0.join("document"),"ordinary.lock").unwrap();let copy=lock.0.try_clone().unwrap();drop(lock);assert!(crate::private_registry::operation_lock(&f.0.join("document"),"ordinary.lock").is_ok());drop(copy);
 }
 fn command(fd:i32,ready:&Path)->Command{
  let script="import os,sys,time,pathlib,json
s=os.fstat(int(sys.argv[1]))
pathlib.Path(sys.argv[2]).write_text(json.dumps({'pid':os.getpid(),'inode':s.st_ino}))
time.sleep(30)";
  crate::pet_helper::command("/usr/bin/python3",&["-I".into(),"-c".into(),script.into(),fd.to_string(),ready.to_string_lossy().into_owned()])
 }
 fn marker(path:&Path)->Option<serde_json::Value>{fs::read(path).ok().and_then(|b|serde_json::from_slice(&b).ok())}
 #[test]fn trusted_exec_inherits_only_selected_lifetime_and_holds_after_parent_guard_closes(){
  let f=Fixture::new();let guard=acquire_at(&f.0).unwrap();let file=guard.file();let inode=file.metadata().unwrap().ino();let ready=f.0.join("ready");let mut cmd=command(file.as_raw_fd(),&ready);crate::pet_process_group::inherit_file(&mut cmd,file);cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());let mut child=crate::pet_process_group::OwnedChild::new(cmd.spawn().unwrap());let until=Instant::now()+Duration::from_secs(3);while marker(&ready).is_none(){assert!(!child.exited().unwrap());assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}assert_eq!(marker(&ready).unwrap()["inode"].as_u64(),Some(inode));drop(cmd);drop(guard);assert!(acquire_at(&f.0).is_err());child.stop_and_reap().unwrap();assert!(acquire_at(&f.0).is_ok());
 }
 #[tokio::test]async fn cancelled_legacy_session_keeps_inherited_lock_until_its_worker_and_child_are_reaped(){
  let f=Fixture::new();let guard=acquire_at(&f.0).unwrap();let file=guard.file();let ready=f.0.join("ready");let cmd=command(file.as_raw_fd(),&ready);let session=crate::pet_helper::Session::acquire(crate::pet_helper::Lane::AudioEffect,Duration::from_secs(5)).unwrap().with_native_lifetime(file);let task=tokio::spawn(async move{session.run_input(cmd,None,4096,Duration::from_secs(5),||Ok(()),Arc::new(||true)).await});let until=Instant::now()+Duration::from_secs(3);while marker(&ready).is_none(){assert!(Instant::now()<until);tokio::time::sleep(Duration::from_millis(5)).await;}let pid=marker(&ready).unwrap()["pid"].as_i64().unwrap()as i32;drop(guard);assert!(acquire_at(&f.0).is_err());task.abort();let _=task.await;let until=Instant::now()+Duration::from_secs(3);loop{if let Ok(next)=acquire_at(&f.0){assert_eq!(unsafe{libc::kill(pid,0)},-1);assert_eq!(io::Error::last_os_error().raw_os_error(),Some(libc::ESRCH));drop(next);break;}assert!(Instant::now()<until);tokio::time::sleep(Duration::from_millis(5)).await;}
 }
}
