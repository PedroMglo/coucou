use super::*;
use std::{fs,path::PathBuf,sync::{Arc,atomic::AtomicUsize}};
use serde_json::Value;

struct Fixture{root:PathBuf}
impl Fixture{
 fn new()->Self{static NEXT:AtomicUsize=AtomicUsize::new(0);let root=std::env::temp_dir().join(format!("coucou-whisper-io-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir(&root).unwrap();fs::write(root.join("model.bin"),b"synthetic, not a model").unwrap();Self{root}}
 fn call(&self,samples:&[f32],helper:&str,current:&dyn Fn()->bool)->Option<super::super::Transcript>{transcribe_request_guarded(Path::new("/usr/bin/python3"),&self.root,samples,None,&[],helper,current)}
 fn marker(&self)->PathBuf{self.root.join("marker.json")}
}
impl Drop for Fixture{fn drop(&mut self){release_if_idle();let _=fs::remove_dir_all(&self.root);}}
const ECHO:&str=r#"import sys,json,os
while True:
 line=sys.stdin.buffer.readline()
 if not line:break
 q=json.loads(line)
 n=q['samples']*4
 while n:
  chunk=sys.stdin.buffer.read(n)
  if not chunk:raise SystemExit(1)
  n-=len(chunk)
 print(json.dumps({'protocolVersion':q['protocolVersion'],'requestId':q['requestId'],'text':str(q['requestId']),'device':'fake'}),flush=True)
"#;
fn wait(mut condition:impl FnMut()->bool){let end=Instant::now()+Duration::from_secs(4);while !condition(){assert!(Instant::now()<end,"synthetic worker did not reach boundary");std::thread::sleep(Duration::from_millis(2));}}
fn live(pid:u64)->bool{let Ok(bytes)=fs::read(format!("/proc/{pid}/stat"))else{return false;};let end=bytes.iter().rposition(|b|*b==b')').unwrap();!matches!(bytes[end+2],b'Z'|b'X')}
const WAIT_REPLY:&str=r#"import sys,json,os,time
from pathlib import Path
q=json.loads(sys.stdin.buffer.readline());sys.stdin.buffer.read(q['samples']*4)
Path(sys.argv[1],'marker.json').write_text(json.dumps({'parent':os.getpid()}))
while True:time.sleep(1)
"#;

#[test]fn whisper_io_cancel_during_stdin_backpressure_collects_own_parent_child_and_reader(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let current=Arc::new(AtomicBool::new(true));
 let helper=r#"import os,time,json,sys
from pathlib import Path
child=os.fork()
if child==0:
 while True:time.sleep(1)
Path(sys.argv[1],'marker.json').write_text(json.dumps({'parent':os.getpid(),'child':child}))
while True:time.sleep(1)
"#;
 let began=Instant::now();std::thread::scope(|scope|{
  let marker=f.marker();let flag=current.clone();let revoke=scope.spawn(move||{wait(||marker.exists());flag.store(false,Ordering::Release);});
  assert!(f.call(&vec![0.1;16000*28],helper,&||current.load(Ordering::Acquire)).is_none());revoke.join().unwrap();
 });assert!(began.elapsed()<Duration::from_secs(6));let ids:Value=serde_json::from_slice(&fs::read(f.marker()).unwrap()).unwrap();assert!(!live(ids["parent"].as_u64().unwrap()));assert!(!live(ids["child"].as_u64().unwrap()));assert!(slot().lock().unwrap().is_none());
}
#[test]fn whisper_io_revoked_while_waiting_never_starts_a_replacement_worker(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let held=slot().lock().unwrap();let valid=AtomicBool::new(true);let observations=AtomicUsize::new(0);
 let helper="from pathlib import Path;import sys;Path(sys.argv[1],'marker.json').write_text('started')";
 std::thread::scope(|scope|{let request=scope.spawn(||f.call(&[0.1;4],helper,&||{observations.fetch_add(1,Ordering::Relaxed);valid.load(Ordering::Acquire)}));wait(||observations.load(Ordering::Acquire)>1);valid.store(false,Ordering::Release);drop(held);assert!(request.join().unwrap().is_none());});assert!(!f.marker().exists());
}
#[test]fn whisper_io_rechecks_original_guard_after_acquiring_the_slot(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let checks=AtomicUsize::new(0);let helper="from pathlib import Path;import sys;Path(sys.argv[1],'marker.json').write_text('started')";
 assert!(f.call(&[0.1;4],helper,&||checks.fetch_add(1,Ordering::Relaxed)==0).is_none());assert!(!f.marker().exists());
}
#[test]fn whisper_io_successive_requests_use_distinct_ids_on_the_same_owned_pipe(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let a=f.call(&[0.1;4],ECHO,&||true).unwrap();let b=f.call(&[0.1;4],ECHO,&||true).unwrap();assert_eq!(a.text,"1");assert_eq!(b.text,"2");
}
#[test]fn whisper_io_old_missing_or_wrong_response_identity_never_publishes_text(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();
 for response in ["{'text':'unbound'}","{'protocolVersion':9,'requestId':q['requestId'],'text':'wrong version'}","{'protocolVersion':1,'requestId':q['requestId']+1,'text':'wrong id'}"]{
  let helper=format!("import sys,json\nq=json.loads(sys.stdin.buffer.readline());sys.stdin.buffer.read(q['samples']*4)\nprint(json.dumps({response}),flush=True)\n");assert!(f.call(&[0.1;4],&helper,&||true).is_none());assert!(slot().lock().unwrap().is_none());
 }
}
#[test]fn whisper_io_excessive_or_unterminated_response_is_bounded_and_refused(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();
 for payload in ["sys.stdout.write('x'*(128*1024+2));sys.stdout.flush()","sys.stdout.write(json.dumps({'protocolVersion':1,'requestId':q['requestId'],'text':'no terminator'}));sys.stdout.flush()"]{
  let helper=format!("import sys,json\nq=json.loads(sys.stdin.buffer.readline());sys.stdin.buffer.read(q['samples']*4)\n{payload}\n");assert!(f.call(&[0.1;4],&helper,&||true).is_none());assert!(slot().lock().unwrap().is_none());
 }
}
#[test]fn whisper_io_drop_unblocks_a_reader_waiting_on_a_full_response_channel(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let helper=ECHO.replace("print(json.dumps(","print(json.dumps(");let helper=format!("{}",helper.replace(",flush=True)",",flush=True)\n for _ in range(5):print(json.dumps({'protocolVersion':q['protocolVersion'],'requestId':q['requestId'],'text':'extra'}),flush=True)"));
 assert!(f.call(&[0.1;4],&helper,&||true).is_some());std::thread::sleep(Duration::from_millis(20));let began=Instant::now();release_if_idle();assert!(began.elapsed()<Duration::from_secs(2));assert!(slot().lock().unwrap().is_none());
}
#[test]fn whisper_io_tuple_key_does_not_alias_paths_or_different_helper_programs(){
 assert!(Key::new(Path::new("/a:b"),Path::new("/c"),"one")!=Key::new(Path::new("/a"),Path::new("b:/c"),"one"));assert!(Key::new(Path::new("/a"),Path::new("/b"),"one")!=Key::new(Path::new("/a"),Path::new("/b"),"two"));
}
#[test]fn whisper_io_revocation_after_complete_delivery_stops_the_response_wait(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let valid=Arc::new(AtomicBool::new(true));
 std::thread::scope(|scope|{let marker=f.marker();let flag=valid.clone();let revoke=scope.spawn(move||{wait(||marker.exists());flag.store(false,Ordering::Release);});assert!(f.call(&[0.1;4],WAIT_REPLY,&||valid.load(Ordering::Acquire)).is_none());revoke.join().unwrap();});
 let ids:Value=serde_json::from_slice(&fs::read(f.marker()).unwrap()).unwrap();assert!(!live(ids["parent"].as_u64().unwrap()));assert!(slot().lock().unwrap().is_none());
}
#[test]fn whisper_io_deadline_keeps_the_slot_until_its_worker_is_collected(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let began=Instant::now();
 assert!(transcribe_request_until(Path::new("/usr/bin/python3"),&f.root,&[0.1;4],None,&[],WAIT_REPLY,&||true,Duration::from_secs(1)).is_none());
 assert!(began.elapsed()<Duration::from_secs(4));let ids:Value=serde_json::from_slice(&fs::read(f.marker()).unwrap()).unwrap();assert!(!live(ids["parent"].as_u64().unwrap()));assert!(slot().lock().unwrap().is_none());
}
#[test]fn whisper_io_duplicate_response_cannot_complete_the_next_request(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let helper=ECHO.replace(",flush=True)",",flush=True)\n print(json.dumps({'protocolVersion':q['protocolVersion'],'requestId':q['requestId'],'text':'duplicate'}),flush=True)");
 assert_eq!(f.call(&[0.1;4],&helper,&||true).unwrap().text,"1");assert!(f.call(&[0.1;4],&helper,&||true).is_none());assert!(slot().lock().unwrap().is_none());
}
#[test]fn whisper_io_unwind_collects_its_worker_before_the_mutex_becomes_poisoned(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();let f=Fixture::new();let marker=f.marker();
 let caught=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||f.call(&[0.1;4],WAIT_REPLY,&||{if marker.exists(){panic!("synthetic scope failure");}true})));assert!(caught.is_err());
 let guard=match slot().lock(){Err(error)=>error,Ok(_)=>panic!("synthetic unwind did not poison its mutex")};assert!(guard.get_ref().is_none());drop(guard);let ids:Value=serde_json::from_slice(&fs::read(f.marker()).unwrap()).unwrap();assert!(!live(ids["parent"].as_u64().unwrap()));
 // This reset belongs only to the isolated test harness. Production preserves
 // the poisoned/unavailable state rather than silently renewing its authority.
 slot().clear_poison();
}
#[test]fn whisper_io_python_parser_enforces_the_real_versioned_request_without_loading_models(){
 let program=r#"import ast,json,sys,unicodedata
source=ast.parse(sys.stdin.read())
nodes=[n for n in source.body if isinstance(n,ast.FunctionDef) and n.name in {'unique_object','parse_request'} or isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id.startswith('MAX_') for t in n.targets)]
exec(compile(ast.Module(body=nodes,type_ignores=[]),'<selected-protocol-functions>','exec'))
def frame(**changes):
 q={'protocolVersion':1,'requestId':17,'samples':4};q.update(changes);return (json.dumps(q)+'\n').encode()
assert parse_request(frame())==(4,[],17)
for line in [frame(protocolVersion=2),frame(protocolVersion=True),frame(requestId=True),frame(requestId=0),frame(requestId=2**64),frame(samples=-1),frame(vocabulary=['bad\nterm']),b'{"protocolVersion":1,"requestId":1,"requestId":2,"samples":4}\n']:
 try:parse_request(line)
 except ValueError:pass
 else:raise AssertionError('invalid frame accepted')
print('protocol validated; no model imports')
"#;
 let mut child=Command::new("/usr/bin/python3").args(["-I","-c",program]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();child.stdin.take().unwrap().write_all(HELPER.as_bytes()).unwrap();let out=child.wait_with_output().unwrap();assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));assert!(String::from_utf8_lossy(&out.stdout).contains("protocol validated"));
}
