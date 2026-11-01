//! Private CLI protocol peer; no agy invocation, account or model/provider IO.
use super::*;
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt};
const A:&str="11111111-1111-1111-1111-111111111111";
const B:&str="22222222-2222-2222-2222-222222222222";
const PEER:&str=r#"#!/usr/bin/env python3
import json,sys,os,pathlib
if sys.argv[1:]==['--qa-executable-probe']:sys.exit(0)
root=pathlib.Path.cwd();mode=(root/'mode').read_text()
(root/'peer.pid').write_text(str(os.getpid()))
(root/'launch.json').write_text(json.dumps({'args':sys.argv[1:],'ownedGate':os.getenv('COUCOU_ASSISTANT_GATE')}))
A='11111111-1111-1111-1111-111111111111';B='22222222-2222-2222-2222-222222222222'
def send(v):print(json.dumps(v),flush=True)
for line in sys.stdin:
 p=json.loads(line)
 with(root/'input.jsonl').open('a')as f:f.write(json.dumps(p)+'\n')
 if mode=='early-error':send({'event':'result','result':{'status':'ERROR','conversation_id':'','error':'synthetic native failure'}});continue
 if mode!='missing-init':
  v={'event':'init','conversation_id':B if mode=='wrong-resume' else A,'init':{'cwd':str(root),'permission_mode':'native-fixture'}}
  if mode=='wrong-cwd':v['init']['cwd']=str(root.parent)
  if mode=='invalid-init':v['conversation_id']='bad\nidentity'
  send(v)
  if mode=='repeat-init':send(v)
  if mode=='changed-init':v['conversation_id']=B;send(v)
 send({'event':'step_update','step_update':{'conversation_id':B if mode=='foreign-step' else A,'step_type':'agent_response','text_delta':'safe'}})
 send({'event':'result','result':{'conversation_id':B if mode=='foreign-result' else A,'status':'SUCCESS','response':'final'}})
"#;
// A parallel fork can retain a newly written script's O_WRONLY descriptor.
// Prove that this immutable synthetic executable is ready before testing the
// protocol, as the Codex fixture already avoids this ETXTBSY class entirely.
fn executable_ready(exe:&std::path::Path,budget:Duration)->std::io::Result<()>{
 let until=Instant::now()+budget;
 loop{
  let mut child=match Command::new(exe).arg("--qa-executable-probe").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn(){
   Ok(child)=>child,
   Err(error)if error.raw_os_error()==Some(libc::ETXTBSY)&&Instant::now()<until=>{std::thread::sleep(Duration::from_millis(2));continue;},
   Err(error)=>return Err(error),
  };
  loop{
   match child.try_wait(){Ok(Some(status))if status.success()=>return Ok(()),Ok(Some(_))=>return Err(std::io::Error::other("synthetic executable probe failed")),Err(error)=>{let _=child.kill();let _=child.wait();return Err(error);},Ok(None)=>{}}
   if Instant::now()>=until{let _=child.kill();let _=child.wait();return Err(std::io::Error::new(std::io::ErrorKind::TimedOut,"synthetic executable probe timed out"));}
   std::thread::sleep(Duration::from_millis(2));
  }
 }
}
struct Fixture{dir:PathBuf,exe:PathBuf}
impl Fixture{
 fn new(mode:&str)->Self{static NEXT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);let dir=std::env::temp_dir().join(format!("mochi-gravity-session-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir_all(&dir).unwrap();fs::set_permissions(&dir,fs::Permissions::from_mode(0o700)).unwrap();let exe=dir.join("peer.py");fs::write(&exe,PEER).unwrap();fs::set_permissions(&exe,fs::Permissions::from_mode(0o700)).unwrap();fs::write(dir.join("mode"),mode).unwrap();executable_ready(&exe,Duration::from_secs(2)).expect("private peer executable must be ready before protocol tests");Self{dir,exe}}
 fn json(&self,name:&str)->Value{serde_json::from_slice(&fs::read(self.dir.join(name)).unwrap()).unwrap()}
 fn reaped(&self){let pid=fs::read_to_string(self.dir.join("peer.pid")).unwrap().parse::<i32>().unwrap();assert_ne!(unsafe{libc::kill(pid,0)},0,"only the private owned child must be reaped");}
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.dir);}}
#[test]fn creation_resume_and_explicit_full_access_keep_native_arguments_and_one_confirmed_identity(){
 for(session,full)in[(None,false),(None,true),(Some(A),false),(Some(A),true)]{let f=Fixture::new("normal");let mut ids=vec![];let mut deltas=vec![];let reply=answer(&f.exe,&f.dir,"fixture-free","Synthetic instructions",&[],"Original question",session,full,None,|s|ids.push(s.to_string()),|s|deltas.push(s),|_|{}).unwrap();assert_eq!(reply.text,"final");assert_eq!(ids,vec![A]);assert_eq!(deltas,vec!["safe"]);let launch=f.json("launch.json");let args=launch["args"].as_array().unwrap();let creation_bypass=full&&session.is_none();assert_eq!(args.iter().any(|v|v=="--dangerously-skip-permissions"),creation_bypass);if session.is_some(){let i=args.iter().position(|v|v=="--conversation").unwrap();assert_eq!(args[i+1],A);}else{assert!(!args.iter().any(|v|v=="--conversation"||v=="--continue"));}
 #[cfg(target_os="linux")]
 assert_eq!(launch["ownedGate"],if creation_bypass{"0"}else{"1"});
 assert_eq!(fs::read_to_string(f.dir.join("input.jsonl")).unwrap().lines().count(),1);f.reaped();}
}
#[test]fn mismatched_resume_or_workspace_never_rebinds_or_publishes_events_from_the_wrong_context(){
 for mode in["wrong-resume","wrong-cwd","invalid-init"]{let f=Fixture::new(mode);let mut ids=vec![];let mut deltas=vec![];let mut events=0;assert!(answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|s|deltas.push(s),|_|events+=1).is_err());assert!(ids.is_empty());assert!(deltas.is_empty());assert_eq!(events,0);f.reaped();}
}
#[test]fn foreign_progress_or_result_never_publishes_or_replaces_a_confirmed_native_session(){
 for mode in["foreign-step","foreign-result"]{let f=Fixture::new(mode);let mut ids=vec![];let mut deltas=vec![];let mut accepted=vec![];assert!(answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|s|deltas.push(s),|m|accepted.push(m.clone())).is_err());assert_eq!(ids,vec![A]);assert!(!accepted.iter().any(|m|m.to_string().contains(B)));assert_eq!(deltas,if mode=="foreign-result"{vec!["safe"]}else{vec![]});f.reaped();}
}
#[test]fn missing_changed_or_repeated_init_cannot_create_a_success_or_a_second_binding(){
 for mode in["missing-init","repeat-init","changed-init"]{let f=Fixture::new(mode);let mut ids=vec![];let mut deltas=vec![];assert!(answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|s|deltas.push(s),|_|{}).is_err());assert_eq!(ids,if mode=="missing-init"{vec![]}else{vec![A]});assert!(deltas.is_empty());f.reaped();}
}
#[test]fn native_failure_before_init_is_reported_without_inventing_or_replacing_a_session(){
 let f=Fixture::new("early-error");let mut ids=vec![];let err=answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|_|panic!("failure has no delta"),|_|{}).err().unwrap();assert!(err.contains("synthetic native failure"));assert!(ids.is_empty());f.reaped();
}
#[test]fn invalid_requested_identity_never_starts_a_private_child_or_writes_a_prompt(){
 let f=Fixture::new("normal");assert!(answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some("bad\nrequest"),false,None,|_|panic!("no binding"),|_|{},|_|{}).is_err());assert!(!f.dir.join("peer.pid").exists());assert!(!f.dir.join("input.jsonl").exists());
}

#[test]fn writable_script_diagnostics_are_bounded_and_do_not_invent_a_native_session(){
 let f=Fixture::new("normal");let writer=fs::OpenOptions::new().write(true).open(&f.exe).unwrap();
 let error=Command::new(&f.exe).arg("--qa-executable-probe").spawn().unwrap_err();assert_eq!(error.raw_os_error(),Some(libc::ETXTBSY));
 let start=Instant::now();let error=executable_ready(&f.exe,Duration::from_millis(20)).unwrap_err();assert_eq!(error.raw_os_error(),Some(libc::ETXTBSY));assert!(start.elapsed()<Duration::from_secs(1));
 let mut ids=vec![];let error=answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|_|{},|_|{}).err().unwrap();assert!(error.contains("Cannot start Antigravity"));assert!(ids.is_empty());assert!(!f.dir.join("peer.pid").exists());assert!(!f.dir.join("input.jsonl").exists());
 drop(writer);executable_ready(&f.exe,Duration::from_secs(2)).unwrap();assert!(!f.dir.join("peer.pid").exists());let reply=answer(&f.exe,&f.dir,"fixture-free","Synthetic",&[],"Original",Some(A),false,None,|s|ids.push(s.to_string()),|_|{},|_|{}).unwrap();assert_eq!(reply.text,"final");assert_eq!(ids,vec![A]);f.reaped();
}
