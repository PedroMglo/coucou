//! Linux command supervisor. `cwd` is an open directory, not a second pathname
//! lookup. Executable identity is rechecked best effort, not exec-by-handle.
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use super::types::*;

pub fn run(permit:RunPermit,budget:&mut TurnOutputBudget,cancel:&Arc<AtomicBool>,deadline:Instant,mut receipt:StepReceipt,on_uncertain:impl FnMut(&StepReceipt))->ProcessResult{
 if cancel.load(Ordering::Acquire){receipt.finish(ReceiptState::Cancelled,Some(ReceiptCause::UserCancel));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}
 if Instant::now()>=deadline{receipt.finish(ReceiptState::TimedOut,Some(ReceiptCause::TaskDeadline));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}
 if !budget.can_run(){receipt.finish(ReceiptState::OutputLimit,Some(ReceiptCause::OutputBudget));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}
 #[cfg(target_os="linux")]{linux(permit,budget,cancel,deadline,receipt,on_uncertain)}
 #[cfg(not(target_os="linux"))]{let _=(permit,on_uncertain);receipt.finish(ReceiptState::FailedToStart,Some(ReceiptCause::SpawnError));ProcessResult{receipt,stdout:vec![],stderr:vec![]}}
}

#[cfg(target_os="linux")]
fn linux(permit:RunPermit,budget:&mut TurnOutputBudget,cancel:&Arc<AtomicBool>,deadline:Instant,mut receipt:StepReceipt,mut on_uncertain:impl FnMut(&StepReceipt))->ProcessResult{
 use std::{io::Read,os::{fd::AsRawFd,unix::process::{CommandExt,ExitStatusExt}},process::{Command,Stdio}};
 let spec=permit.spec;
 if super::approval::stamp(&spec.executable).ok().as_ref()!=Some(&spec.executable_stamp){receipt.finish(ReceiptState::FailedToStart,Some(ReceiptCause::SpawnError));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}
 let fd=spec.directory.as_raw_fd();let mut command=Command::new(&spec.executable);
 command.args(&spec.args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).env_clear();
 for key in ["PATH","HOME","LANG","LC_ALL","LC_CTYPE","TERM","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS","XDG_SESSION_TYPE","HYPRLAND_INSTANCE_SIGNATURE"]{if let Some(value)=std::env::var_os(key){command.env(key,value);}}
 // Only async-signal-safe syscalls in the child between fork and exec. The
 // parent's File keeps this exact directory handle open through spawn.
 unsafe{command.pre_exec(move||{if libc::setpgid(0,0)!=0{return Err(std::io::Error::last_os_error());}if libc::fchdir(fd)!=0{return Err(std::io::Error::last_os_error());}Ok(())});}
 let cancelled=cancel.load(Ordering::Acquire);
 if cancelled||Instant::now()>=deadline{receipt.finish(if cancelled{ReceiptState::Cancelled}else{ReceiptState::TimedOut},Some(if cancelled{ReceiptCause::UserCancel}else{ReceiptCause::TaskDeadline}));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}
 struct OwnedChild{child:std::process::Child,reaped:bool}
 impl Drop for OwnedChild{
  fn drop(&mut self){
   if self.reaped{return;}
   let pid=self.child.id() as libc::pid_t;let mut info:libc::siginfo_t=unsafe{std::mem::zeroed()};
   // Also retain ownership during an unwinding panic. A dropped std Child on
   // its own would leave a real command running after Chat released its lease.
   loop{
    let result=unsafe{libc::waitid(libc::P_PID,pid as libc::id_t,&mut info,libc::WEXITED|libc::WNOHANG|libc::WNOWAIT)};
    if result==0{unsafe{libc::kill(-pid,libc::SIGKILL);}loop{match self.child.wait(){Ok(_)=>break,Err(e) if e.raw_os_error()==Some(libc::ECHILD)=>break,Err(_)=>std::thread::sleep(Duration::from_millis(10))}}break;}
    let error=std::io::Error::last_os_error();
    if error.raw_os_error()==Some(libc::ECHILD){break;}
    std::thread::sleep(Duration::from_millis(10));
   }
  }
 }
 let mut owned=match command.spawn(){Ok(child)=>OwnedChild{child,reaped:false},Err(_)=>{receipt.finish(ReceiptState::FailedToStart,Some(ReceiptCause::SpawnError));return ProcessResult{receipt,stdout:vec![],stderr:vec![]};}};
 receipt.state=ReceiptState::Starting;receipt.started_at=Some(now());
 let pid=owned.child.id() as libc::pid_t;
 let command_deadline=Instant::now()+Duration::from_millis(spec.timeout_ms);
 let mut stdout=owned.child.stdout.take();let mut stderr=owned.child.stderr.take();
 let mut output=Vec::new();let mut errors=Vec::new();
 let mut trigger=None;let mut stopping=None;let mut kill_sent=false;let mut reported=false;let mut exited_at=None;let mut capture_limited=false;
 let nonblock=|fd|unsafe{let flags=libc::fcntl(fd,libc::F_GETFL);flags>=0&&libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)>=0};
 if !stdout.as_ref().is_some_and(|p|nonblock(p.as_raw_fd()))||!stderr.as_ref().is_some_and(|p|nonblock(p.as_raw_fd())){stdout=None;stderr=None;trigger=Some(ReceiptCause::SpawnError);}
 loop{
  let instant=Instant::now();
  // Task deadline wins only when both are first observed expired together.
  // Preserve this cause throughout later cleanup, even across another deadline.
  if trigger.is_none(){trigger=if cancel.load(Ordering::Acquire){Some(ReceiptCause::UserCancel)}else if instant>=deadline{Some(ReceiptCause::TaskDeadline)}else if instant>=command_deadline{Some(ReceiptCause::CommandDeadline)}else{None};}
  let mut info:libc::siginfo_t=unsafe{std::mem::zeroed()};
  // WNOWAIT keeps the leader unreaped while its group is signalled: its PID
  // cannot be reused as a different process group in that interval.
  let wait=unsafe{libc::waitid(libc::P_PID,pid as libc::id_t,&mut info,libc::WEXITED|libc::WNOHANG|libc::WNOWAIT)};
  let exited=wait==0&&unsafe{info.si_pid()}==pid;
  if exited&&exited_at.is_none(){exited_at=Some(instant);}
  if trigger.is_none()&&exited_at.is_some_and(|at|instant.duration_since(at)>=Duration::from_millis(250))&&(stdout.is_some()||stderr.is_some()){trigger=Some(ReceiptCause::ProcessExit);}
  if trigger.is_some()&&stopping.is_none(){
   stopping=Some(instant);
   if wait==0{unsafe{libc::kill(-pid,libc::SIGTERM);}}
  }
  if let Some(at)=stopping{
   if !kill_sent&&instant.duration_since(at)>=Duration::from_millis(250){
    // Still not reaped here. Never signal by a PID recovered from disk.
    if wait==0{unsafe{libc::kill(-pid,libc::SIGKILL);}}kill_sent=true;
   }
   if !reported&&instant.duration_since(at)>=Duration::from_secs(2){
    let mut uncertain=receipt.clone();uncertain.state=ReceiptState::Unknown;uncertain.cause=Some(ReceiptCause::CleanupUnconfirmed);uncertain.trigger=trigger;uncertain.stdout_bytes=output.len() as u64;uncertain.stderr_bytes=errors.len() as u64;
    on_uncertain(&uncertain);reported=true;
   }
  }
  // Once a cap is reached, close capture rather than reading/discarding bytes
  // beyond that cap. The owned process is still terminated and reaped below.
  if trigger==Some(ReceiptCause::OutputBudget){stdout=None;stderr=None;}
  fn read_pipe<T:Read>(pipe:&mut Option<T>,out:&mut Vec<u8>,budget:&mut TurnOutputBudget)->std::io::Result<bool>{
   let Some(reader)=pipe.as_mut()else{return Ok(false)};
   let cap=4096.min(MAX_STREAM_OUTPUT-out.len()).min(budget.raw_remaining);
   if cap==0{*pipe=None;return Ok(true);}let mut bytes=[0u8;4096];
   match reader.read(&mut bytes[..cap]){Ok(0)=>{*pipe=None;Ok(false)},Ok(n)=>{out.extend_from_slice(&bytes[..n]);budget.raw_remaining-=n;let capped=out.len()==MAX_STREAM_OUTPUT||budget.raw_remaining==0;if capped{*pipe=None;}Ok(capped)},Err(e) if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>Ok(false),Err(e)=>{*pipe=None;Err(e)}}
  }
  let reads=[read_pipe(&mut stdout,&mut output,budget),read_pipe(&mut stderr,&mut errors,budget)];
  capture_limited|=reads.iter().any(|r|matches!(r,Ok(true)));
  if trigger.is_none(){if capture_limited{trigger=Some(ReceiptCause::OutputBudget);}else if reads.iter().any(Result::is_err){trigger=Some(ReceiptCause::SpawnError);stdout=None;stderr=None;}}
  if exited&&stdout.is_none()&&stderr.is_none(){
   // Signal cleanup before reaping the leader; a terminated leader may still
   // have group children even when they closed their inherited output pipes.
   unsafe{libc::kill(-pid,libc::SIGKILL);}
   match owned.child.wait(){
    Ok(status)=>{
     owned.reaped=true;receipt.exit_code=status.code();receipt.signal=status.signal();
     let (state,cause)=match trigger{Some(ReceiptCause::UserCancel)=>(ReceiptState::Cancelled,trigger),Some(ReceiptCause::TaskDeadline|ReceiptCause::CommandDeadline)=>(ReceiptState::TimedOut,trigger),Some(ReceiptCause::OutputBudget)=>(ReceiptState::OutputLimit,trigger),Some(ReceiptCause::SpawnError)=>(ReceiptState::Unknown,trigger),_=>(ReceiptState::Exited,None)};
     receipt.finish(state,cause);receipt.trigger=trigger;break;
    },Err(_)=>{if !reported{let mut r=receipt.clone();r.state=ReceiptState::Unknown;r.cause=Some(ReceiptCause::CleanupUnconfirmed);r.trigger=trigger;on_uncertain(&r);reported=true;}}
   }
  }
  // Cleanup that cannot be confirmed retains Child, the worker and Chat lease.
  // No later command can start and the UI remains stopping/uncertain.
  std::thread::sleep(Duration::from_millis(10));
 }
 receipt.stdout_bytes=output.len() as u64;receipt.stderr_bytes=errors.len() as u64;receipt.truncated=capture_limited;
 ProcessResult{receipt,stdout:output,stderr:errors}
}

#[cfg(all(test,target_os="linux"))]mod tests{
 use super::*;
 use crate::pet_config::LocalExecution;
 fn permit(program:&str,args:&[&str],timeout_ms:u64,cwd:&std::path::Path)->RunPermit{let s=super::super::approval::freeze(CommandInput{program:program.into(),args:args.iter().map(|s|(*s).into()).collect(),reason:"synthetic offline test".into(),timeout_ms:Some(timeout_ms)},cwd,&LocalExecution::default()).unwrap();RunPermit{spec:s}}
 fn execute(p:RunPermit,b:&mut TurnOutputBudget,cancel:Arc<AtomicBool>)->ProcessResult{run(p,b,&cancel,Instant::now()+Duration::from_secs(5),StepReceipt::proposed(1),|_|{})}
 #[test]fn literal_argv_and_both_streams_are_bounded(){let mut b=TurnOutputBudget::new(512*1024);let r=execute(permit("/bin/echo",&["literal; $HOME"],1000,&std::env::temp_dir()),&mut b,Arc::default());assert_eq!(r.receipt.state,ReceiptState::Exited);assert_eq!(r.receipt.exit_code,Some(0));assert_eq!(r.stdout,b"literal; $HOME\n");}
 #[test]fn stderr_and_nonzero_exit_are_reported_without_retry(){let mut b=TurnOutputBudget::new(1024);let r=execute(permit("/bin/sh",&["-c","printf out; printf err >&2; exit 7"],1000,&std::env::temp_dir()),&mut b,Arc::default());assert_eq!(r.stdout,b"out");assert_eq!(r.stderr,b"err");assert_eq!(r.receipt.exit_code,Some(7));assert_eq!(b.raw_remaining,1018);}
 #[test]fn changed_executable_is_rejected_before_spawn(){use std::os::unix::fs::PermissionsExt;let dir=std::env::temp_dir().join(format!("coucou-executable-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&dir).unwrap();let file=dir.join("synthetic.sh");std::fs::write(&file,"#!/bin/sh\nprintf old").unwrap();std::fs::set_permissions(&file,std::fs::Permissions::from_mode(0o700)).unwrap();let p=permit(file.to_str().unwrap(),&[],1000,&dir);std::fs::write(&file,"#!/bin/sh\nprintf changed").unwrap();let mut b=TurnOutputBudget::new(1024);let r=execute(p,&mut b,Arc::default());assert_eq!(r.receipt.state,ReceiptState::FailedToStart);assert!(r.receipt.started_at.is_none());std::fs::remove_dir_all(dir).unwrap();}
 #[test]fn shared_budget_cannot_reset_for_the_next_command(){let mut b=TurnOutputBudget::new(12);let r=execute(permit("/bin/sh",&["-c","printf 12345678901234567890"],1000,&std::env::temp_dir()),&mut b,Arc::default());assert_eq!(r.receipt.state,ReceiptState::OutputLimit);assert_eq!(r.stdout.len()+r.stderr.len(),12);assert_eq!(b.raw_remaining,0);let r=execute(permit("/bin/true",&[],1000,&std::env::temp_dir()),&mut b,Arc::default());assert!(r.receipt.started_at.is_none());}
 #[test]fn command_timeout_is_not_relabelled_during_cleanup(){let mut b=TurnOutputBudget::new(1024);let p=permit("/bin/sh",&["-c","trap '' TERM; sleep 10"],100,&std::env::temp_dir());let r=run(p,&mut b,&Arc::default(),Instant::now()+Duration::from_millis(200),StepReceipt::proposed(1),|_|{});assert_eq!(r.receipt.state,ReceiptState::TimedOut);assert_eq!(r.receipt.cause,Some(ReceiptCause::CommandDeadline));}
 #[test]fn already_cancelled_permit_never_spawns(){let mut b=TurnOutputBudget::new(1024);let r=execute(permit("/bin/true",&[],1000,&std::env::temp_dir()),&mut b,Arc::new(AtomicBool::new(true)));assert_eq!(r.receipt.state,ReceiptState::Cancelled);assert!(r.receipt.started_at.is_none());}
 #[test]fn output_cap_during_cancel_cleanup_closes_capture_without_changing_cause(){let cancel=Arc::new(AtomicBool::new(false));let flag=cancel.clone();let stop=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(100));flag.store(true,Ordering::Release);});let mut b=TurnOutputBudget::new(12);let r=execute(permit("/bin/sh",&["-c","trap 'printf 123456789012' TERM; while :; do sleep 0.01; done"],1000,&std::env::temp_dir()),&mut b,cancel);stop.join().unwrap();assert_eq!(r.receipt.state,ReceiptState::Cancelled);assert_eq!(r.receipt.cause,Some(ReceiptCause::UserCancel));assert_eq!(r.stdout.len()+r.stderr.len(),12);assert!(r.receipt.truncated);}
 #[test]fn cwd_handle_survives_path_replacement(){let base=std::env::temp_dir().join(format!("coucou-cwd-{}-{}",std::process::id(),now()));std::fs::create_dir_all(base.join("original")).unwrap();let p=permit("/bin/pwd",&[],1000,&base.join("original"));std::fs::rename(base.join("original"),base.join("sealed")).unwrap();std::fs::create_dir(base.join("original")).unwrap();let mut b=TurnOutputBudget::new(1024);let r=execute(p,&mut b,Arc::default());assert!(String::from_utf8_lossy(&r.stdout).contains("/sealed"));std::fs::remove_dir_all(base).unwrap();}
}
