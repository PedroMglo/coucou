use super::*;
use std::path::{Path,PathBuf};
struct Fixture(PathBuf);
impl Fixture{fn new()->Self{use std::os::unix::fs::DirBuilderExt;let p=std::env::temp_dir().join(format!("coucou-resident-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();Self(p)}fn file(&self)->PathBuf{self.0.join("observed")}}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn python(script:&str,file:&Path)->Command{command("/usr/bin/python3",&["-I".into(),"-c".into(),script.into(),file.to_str().unwrap().into()])}
fn start(resources:&Arc<Coordinator>,command:Command)->Resident{Resident::start_on(resources.clone(),Lane::ClipboardWrite,command,Instant::now()+Duration::from_secs(2),&||true).unwrap()}
fn end()->Instant{Instant::now()+Duration::from_secs(2)}
fn gone(pid:i32)->bool{std::fs::read(format!("/proc/{pid}/stat")).map_or(true,|b|{let at=b.iter().rposition(|b|*b==b')').unwrap();matches!(b.get(at+2),Some(b'Z'|b'X'|b'x'))})}
const ECHO:&str="import pathlib,sys\nfor line in sys.stdin.buffer:\n pathlib.Path(sys.argv[1]).write_bytes(line)\n sys.stdout.buffer.write(b'ACK:'+line);sys.stdout.flush()\n";
#[test]fn resident_keeps_one_physical_owner_between_bounded_requests(){
    let f=Fixture::new();let resources=Arc::default();let mut owner=start(&resources,python(ECHO,&f.file()));let pid=owner.owned.child.id();
    assert_eq!(owner.exchange_line(b"first\n",64,end(),&||true).unwrap(),b"ACK:first\n");
    assert!(owner.alive());assert_eq!(owner.owned.child.id(),pid);assert!(Resident::start_on(resources.clone(),Lane::ClipboardWrite,python(ECHO,&f.file()),end(),&||true).is_err());
    assert_eq!(owner.exchange_line(b"second\n",64,end(),&||true).unwrap(),b"ACK:second\n");
    drop(owner);assert!(resources.busy().unwrap().is_empty());assert!(gone(pid as i32));
}
#[test]fn resident_preflight_authority_cannot_expire_and_then_spawn(){
    let f=Fixture::new();let resources:Arc<Coordinator>=Arc::default();let result=Resident::start_on(resources.clone(),Lane::ClipboardWrite,python(ECHO,&f.file()),Instant::now()+Duration::from_millis(10),&||{std::thread::sleep(Duration::from_millis(30));true});
    assert!(matches!(result,Err(ExchangeError::NotStarted)));assert!(!f.file().exists());assert!(resources.busy().unwrap().is_empty());
}
#[test]fn resident_invalid_or_expired_request_preserves_idle_owner_without_sending(){
    let f=Fixture::new();let resources=Arc::default();let mut owner=start(&resources,python(ECHO,&f.file()));
    assert_eq!(owner.exchange_line(&vec![0;MAX_RESIDENT_INPUT+1],64,end(),&||panic!("oversize authority")),Err(ExchangeError::NotStarted));
    assert_eq!(owner.exchange_line(b"never\n",64,Instant::now()+Duration::from_millis(10),&||{std::thread::sleep(Duration::from_millis(30));true}),Err(ExchangeError::NotStarted));
    assert!(!f.file().exists());assert!(owner.alive());assert_eq!(owner.exchange_line(b"fresh\n",64,end(),&||true).unwrap(),b"ACK:fresh\n");
}
#[test]fn resident_uncertain_stream_cannot_be_reused_and_family_precedes_slot_release(){
    let f=Fixture::new();let resources=Arc::default();let script="import os,pathlib,sys,time\nsys.stdin.buffer.read(1)\np=os.fork()\nif p==0:\n os.close(1);time.sleep(30);os._exit(0)\npathlib.Path(sys.argv[1]).write_text(str(os.getpid())+' '+str(p))\ntime.sleep(30)\n";
    let mut owner=start(&resources,python(script,&f.file()));assert_eq!(owner.exchange_line(b"request\n",64,Instant::now()+Duration::from_millis(500),&||true),Err(ExchangeError::Uncertain));
    let ids:Vec<i32>=std::fs::read_to_string(f.file()).unwrap().split_whitespace().map(|p|p.parse().unwrap()).collect();assert!(ids.iter().any(|p|!gone(*p)));
    assert_eq!(owner.exchange_line(b"replay\n",64,end(),&||true),Err(ExchangeError::NotStarted));assert!(!resources.busy().unwrap().is_empty());
    drop(owner);assert!(resources.busy().unwrap().is_empty());assert!(ids.iter().all(|p|gone(*p)));
}
#[test]fn resident_input_backpressure_and_output_excess_are_bounded(){
    let f=Fixture::new();let resources=Arc::default();let mut blocked=start(&resources,python("import time;time.sleep(30)",&f.file()));
    assert_eq!(blocked.exchange_line(&vec![b'x';8*1024*1024],64,Instant::now()+Duration::from_millis(150),&||true),Err(ExchangeError::Uncertain));drop(blocked);assert!(resources.busy().unwrap().is_empty());
    let mut excess=start(&resources,python("import sys;sys.stdin.buffer.read(1);sys.stdout.buffer.write(b'x'*100000);sys.stdout.flush()",&f.file()));
    assert_eq!(excess.exchange_line(b"x",64,end(),&||true),Err(ExchangeError::Uncertain));drop(excess);assert!(resources.busy().unwrap().is_empty());
}
#[test]fn resident_cancel_during_response_wait_retires_without_replaying(){
    let f=Fixture::new();let resources=Arc::default();let mut owner=start(&resources,python("import pathlib,sys,time;sys.stdin.buffer.read(1);pathlib.Path(sys.argv[1]).touch();time.sleep(30)",&f.file()));
    assert_eq!(owner.exchange_line(b"x",64,end(),&||!f.file().exists()),Err(ExchangeError::Uncertain));assert!(owner.poisoned);drop(owner);assert!(resources.busy().unwrap().is_empty());
}
#[test]fn resident_transfers_full_size_input_without_blocking_or_closing_owner(){
    let f=Fixture::new();let resources=Arc::default();let mut owner=start(&resources,python("import sys,time\nb=sys.stdin.buffer.read(8*1024*1024)\nassert len(b)==8*1024*1024\nsys.stdout.buffer.write(b'ACK\\n');sys.stdout.flush()\ntime.sleep(30)",&f.file()));
    assert_eq!(owner.exchange_line(&vec![b'x';8*1024*1024],64,Instant::now()+Duration::from_secs(3),&||true).unwrap(),b"ACK\n");assert!(owner.alive());drop(owner);assert!(resources.busy().unwrap().is_empty());
}

fn socket_python(script:&str,path:&Path)->Command{command("/usr/bin/python3",&["-I".into(),"-c".into(),script.into(),path.to_str().unwrap().into()])}
const SOCKET_ECHO:&str="import socket,sys,time\ns=socket.socket(socket.AF_UNIX);s.connect(sys.argv[1])\nf=s.makefile('rwb',buffering=0)\nfor line in f:\n f.write(b'ACK:'+line)\n";
#[test]fn resident_socket_keeps_original_process_and_bounded_protocol(){
 let f=Fixture::new();let socket=f.0.join("socket");let listener=std::os::unix::net::UnixListener::bind(&socket).unwrap();let resources:Arc<Coordinator>=Arc::default();
 let mut owner=Resident::start_socket_on(resources.clone(),Lane::ClipboardWrite,socket_python(SOCKET_ECHO,&socket),&listener,end(),&||true).unwrap();let pid=owner.owned.child.id();
 for line in [b"first\n".as_slice(),b"second\n".as_slice()]{assert_eq!(owner.exchange_line(line,64,end(),&||true).unwrap(),[b"ACK:".as_slice(),line].concat());assert!(owner.alive());assert_eq!(owner.owned.child.id(),pid);}
 drop(owner);assert!(gone(pid as i32));assert!(resources.busy().unwrap().is_empty());
}
#[test]fn resident_socket_refuses_an_unrelated_peer_without_trusting_same_user(){
 let f=Fixture::new();let socket=f.0.join("socket");let listener=std::os::unix::net::UnixListener::bind(&socket).unwrap();let _forged=std::os::unix::net::UnixStream::connect(&socket).unwrap();let resources:Arc<Coordinator>=Arc::default();
 let result=Resident::start_socket_on(resources.clone(),Lane::ClipboardWrite,socket_python("import time;time.sleep(30)",&socket),&listener,end(),&||true);
 assert!(matches!(result,Err(ExchangeError::NotStarted)));assert!(resources.busy().unwrap().is_empty());
}
#[test]fn resident_socket_revocation_poisoning_and_child_cleanup_precede_release(){
 let f=Fixture::new();let socket=f.0.join("socket");let listener=std::os::unix::net::UnixListener::bind(&socket).unwrap();let resources:Arc<Coordinator>=Arc::default();
 let script="import socket,sys,time,pathlib\ns=socket.socket(socket.AF_UNIX);s.connect(sys.argv[1]);s.recv(1);pathlib.Path(sys.argv[1]+'.effect').touch();time.sleep(30)";
 let mut owner=Resident::start_socket_on(resources.clone(),Lane::ClipboardWrite,socket_python(script,&socket),&listener,end(),&||true).unwrap();let pid=owner.owned.child.id();let observed=PathBuf::from(format!("{}.effect",socket.display()));
 assert_eq!(owner.exchange_line(b"request\n",64,end(),&||!observed.exists()),Err(ExchangeError::Uncertain));assert!(owner.poisoned);
 assert_eq!(owner.exchange_line(b"replay\n",64,end(),&||true),Err(ExchangeError::NotStarted));assert!(!resources.busy().unwrap().is_empty());drop(owner);assert!(gone(pid as i32));assert!(resources.busy().unwrap().is_empty());
}
