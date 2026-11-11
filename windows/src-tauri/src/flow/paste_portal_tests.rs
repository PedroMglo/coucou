//! Only runs on the SDK's explicitly-created private session bus.
use super::*;
use std::sync::atomic::{AtomicBool,Ordering};
static SERIAL:Mutex<()>=Mutex::new(());
#[derive(Clone,Copy)]enum Mode{Normal,HoldAck,HoldResponse,WrongPath,EarlyDuplicate,Forge,HoldClose,Open,HoldStart,WrongSession,CancelKey}
struct Fake{peer:String,events:Arc<Mutex<Vec<(String,String)>>>,stop:Arc<AtomicBool>,release:Arc<AtomicBool>,cancelled:Arc<AtomicBool>,thread:Option<std::thread::JoinHandle<()>>}
fn wait(mut condition:impl FnMut()->bool){let until=Instant::now()+Duration::from_secs(4);while !condition(){assert!(Instant::now()<until,"private portal boundary not observed");std::thread::sleep(Duration::from_millis(2));}}
impl Fake{
 fn new(mode:Mode)->Self{
  assert_eq!(std::env::var("COUCOU_TEST_PRIVATE_PASTE_BUS").as_deref(),Ok("1"),"use the isolated SDK dbus-run-session runner");
  let events=Arc::new(Mutex::new(Vec::new()));let recorded=events.clone();let stop=Arc::new(AtomicBool::new(false));let done=stop.clone();let release=Arc::new(AtomicBool::new(false));let ready=release.clone();let cancelled=Arc::new(AtomicBool::new(false));let retire=cancelled.clone();let(tx,rx)=std::sync::mpsc::channel();
  let thread=std::thread::spawn(move||{
   let conn=Connection::new_session().unwrap();conn.request_name(DEST,true,true,false).unwrap();let attacker=Connection::new_session().unwrap();tx.send(conn.unique_name().to_string()).unwrap();
   while !done.load(Ordering::Acquire){
    conn.channel().read_write(Some(Duration::from_millis(10))).unwrap();
    while let Some(msg)=conn.channel().pop_message(){
     if msg.msg_type()!=dbus::MessageType::MethodCall{continue;}
     let name=msg.member().unwrap().to_string();let object=msg.path().unwrap().to_string();recorded.lock().unwrap().push((name.clone(),object.clone()));
     if name=="Close"{if !matches!(mode,Mode::HoldClose){conn.channel().send(msg.method_return()).unwrap();}continue;}
     if name=="NotifyKeyboardKeysym"{
      let(_,_,sym,down):(Path,PropMap,i32,u32)=msg.read4().unwrap();recorded.lock().unwrap().push((format!("key:{sym}:{down}"),object));
      if matches!(mode,Mode::CancelKey)&&sym==XK_CONTROL_L&&down==1{retire.store(true,Ordering::Release);}
      conn.channel().send(msg.method_return()).unwrap();continue;
     }
     let opening=matches!(mode,Mode::Open|Mode::HoldStart|Mode::WrongSession|Mode::CancelKey);
     if name!="CreateSession"&&!(opening&&(name=="SelectDevices"||name=="Start")){conn.channel().send(msg.method_return()).unwrap();continue;}
     let options:PropMap=if name=="CreateSession"{msg.read1().unwrap()}else if name=="SelectDevices"{let(_,opts):(Path,PropMap)=msg.read2().unwrap();assert_eq!(opts["types"].0.as_u64(),Some(1));assert_eq!(opts["persist_mode"].0.as_u64(),Some(2));opts}else{let(_,_,opts):(Path,String,PropMap)=msg.read3().unwrap();opts};let token=options["handle_token"].0.as_str().unwrap();let path=portal::request_path(&msg.sender().unwrap(),token);
     let signal=|proof:&str|{let mut signal=dbus::Message::new_signal(path.as_str(),"org.freedesktop.portal.Request","Response").unwrap();signal.set_destination(msg.sender());let mut results:PropMap=HashMap::new();results.insert("proof".into(),variant(proof.to_string()));
      if opening&&name=="CreateSession"{let session_token=options["session_handle_token"].0.as_str().unwrap();let mut session=portal::session_path(&msg.sender().unwrap(),session_token);if matches!(mode,Mode::WrongSession){session.push_str("_foreign");}results.insert("session_handle".into(),variant(Path::from(session)));}
      if opening&&name=="Start"{results.insert("restore_token".into(),variant("private-restored-choice".to_string()));}
      signal.append2(0u32,results)};
     if matches!(mode,Mode::Forge){
      attacker.channel().send(msg.method_return().append1(Path::from(path.clone()))).unwrap();attacker.channel().send(signal("forged")).unwrap();attacker.channel().flush();recorded.lock().unwrap().push(("forged".into(),path.clone()));
      while !ready.load(Ordering::Acquire)&&!done.load(Ordering::Acquire){std::thread::sleep(Duration::from_millis(2));}
     }
     if matches!(mode,Mode::HoldAck){continue;}
     if matches!(mode,Mode::EarlyDuplicate){conn.channel().send(signal("trusted")).unwrap();conn.channel().send(signal("duplicate")).unwrap();}
     let returned=if matches!(mode,Mode::WrongPath){format!("{path}_wrong")}else{path.clone()};
     conn.channel().send(msg.method_return().append1(Path::from(returned))).unwrap();conn.channel().flush();recorded.lock().unwrap().push(("ack".into(),path.clone()));
     if matches!(mode,Mode::HoldStart)&&name=="Start"{retire.store(true,Ordering::Release);}
     if !matches!(mode,Mode::HoldResponse|Mode::HoldClose|Mode::EarlyDuplicate)&&!(matches!(mode,Mode::HoldStart)&&name=="Start"){conn.channel().send(signal("trusted")).unwrap();conn.channel().flush();}
    }
   }
  });
  let peer=rx.recv_timeout(Duration::from_secs(4)).unwrap();Self{peer,events,stop,release,cancelled,thread:Some(thread)}
 }
 fn saw(&self,name:&str)->bool{self.events.lock().unwrap().iter().any(|(n,_)|n==name)}
 fn count(&self,name:&str)->usize{self.events.lock().unwrap().iter().filter(|(n,_)|n==name).count()}
}
impl Drop for Fake{fn drop(&mut self){self.release.store(true,Ordering::Release);self.stop.store(true,Ordering::Release);self.thread.take().unwrap().join().unwrap();}}
fn create(conn:&Connection,peer:&str,current:&dyn Fn()->bool)->Result<PropMap,Failure>{let mut tokens=0;request(conn,peer,"CreateSession",|token|{let mut opts:PropMap=HashMap::new();opts.insert("handle_token".into(),variant(token.to_string()));Ok(method(peer,OBJECT,IFACE,"CreateSession")?.append1(opts))},&mut tokens,current)}
fn proof(result:&PropMap)->&str{result["proof"].0.as_str().unwrap()}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn real_bus_request_authenticates_peer_and_preserves_early_first_response(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::EarlyDuplicate);let conn=Connection::new_session().unwrap();let end=Instant::now()+Duration::from_secs(3);
 let result=create(&conn,&fake.peer,&||Instant::now()<end).unwrap();assert_eq!(proof(&result),"trusted");assert_eq!(fake.count("CreateSession"),1);assert_eq!(fake.count("Close"),0);
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn forged_response_and_method_ack_from_another_peer_cannot_complete_the_request(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::Forge);let peer=fake.peer.clone();let(tx,rx)=std::sync::mpsc::channel();
 let worker=std::thread::spawn(move||{let conn=Connection::new_session().unwrap();let end=Instant::now()+Duration::from_secs(4);tx.send(create(&conn,&peer,&||Instant::now()<end).map(|r|proof(&r).to_string())).unwrap();});
 wait(||fake.saw("forged"));assert!(rx.recv_timeout(Duration::from_millis(100)).is_err());fake.release.store(true,Ordering::Release);assert_eq!(rx.recv_timeout(Duration::from_secs(3)).unwrap(),Ok("trusted".into()));worker.join().unwrap();
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn cancellation_while_method_ack_is_missing_closes_only_the_exact_request_without_retry(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::HoldAck);let conn=Connection::new_session().unwrap();let stop=AtomicBool::new(false);let events=fake.events.clone();
 std::thread::scope(|scope|{
  let cancel=scope.spawn(||{wait(||events.lock().unwrap().iter().any(|(n,_)|n=="CreateSession"));stop.store(true,Ordering::Release);});
  let began=Instant::now();assert!(matches!(create(&conn,&fake.peer,&||!stop.load(Ordering::Acquire)),Err(Failure::Cancelled)));assert!(began.elapsed()<Duration::from_secs(2));cancel.join().unwrap();
 });
 assert_eq!(fake.count("CreateSession"),1);wait(||fake.saw("Close"));let events=fake.events.lock().unwrap();let path=events.iter().find(|(n,_)|n=="Close").unwrap().1.clone();assert!(path.starts_with(&portal::request_path(&conn.unique_name(),"coucoupaste")));drop(events);assert!(!fake.saw("NotifyKeyboardKeysym"));
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn cancellation_while_human_response_is_missing_cannot_start_keys(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::HoldResponse);let conn=Connection::new_session().unwrap();let stop=AtomicBool::new(false);let events=fake.events.clone();
 std::thread::scope(|scope|{let cancel=scope.spawn(||{wait(||events.lock().unwrap().iter().any(|(n,_)|n=="ack"));stop.store(true,Ordering::Release);});assert!(matches!(create(&conn,&fake.peer,&||!stop.load(Ordering::Acquire)),Err(Failure::Cancelled)));cancel.join().unwrap();});
 assert_eq!(fake.count("CreateSession"),1);wait(||fake.saw("Close"));assert!(!fake.saw("NotifyKeyboardKeysym"));
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn returned_foreign_path_is_not_followed_or_closed(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::WrongPath);let conn=Connection::new_session().unwrap();let end=Instant::now()+Duration::from_secs(3);assert!(matches!(create(&conn,&fake.peer,&||Instant::now()<end),Err(Failure::Unknown)));wait(||fake.saw("Close"));assert!(fake.events.lock().unwrap().iter().filter(|(n,_)|n=="Close").all(|(_,p)|!p.ends_with("_wrong")));
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn service_name_replacement_cannot_redirect_a_captured_peer(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::Normal);let replacement=Connection::new_session().unwrap();replacement.request_name(DEST,false,true,true).unwrap();let conn=Connection::new_session().unwrap();let end=Instant::now()+Duration::from_secs(3);let result=create(&conn,&fake.peer,&||Instant::now()<end).unwrap();assert_eq!(proof(&result),"trusted");assert_eq!(fake.count("CreateSession"),1);
}
#[test]#[ignore="synthetic RemoteDesktop protocol; requires the explicit private SDK dbus-run-session runner"]
fn missing_cleanup_ack_never_changes_a_failed_attempt_to_success(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::HoldClose);let conn=Connection::new_session().unwrap();let stop=AtomicBool::new(false);let events=fake.events.clone();
 std::thread::scope(|scope|{let cancel=scope.spawn(||{wait(||events.lock().unwrap().iter().any(|(n,_)|n=="CreateSession"));stop.store(true,Ordering::Release);});assert!(matches!(create(&conn,&fake.peer,&||!stop.load(Ordering::Acquire)),Err(Failure::Cancelled)));cancel.join().unwrap();});assert_eq!(fake.count("CreateSession"),1);
}

struct Files(std::path::PathBuf);
impl Files{fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-paste-portal-{}-{}",std::process::id(),super::super::text::new_id()));std::fs::create_dir(&path).unwrap();Self(path)}fn token(&self)->std::path::PathBuf{self.0.join("remote-desktop-token")}}
impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
#[test]#[ignore="synthetic complete RemoteDesktop opening; private SDK D-Bus and temporary token path only"]
fn complete_open_uses_one_peer_and_session_then_dispatches_without_destination_ack(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::Open);let files=Files::new();std::fs::write(files.token(),"previous-choice").unwrap();let end=Instant::now()+Duration::from_secs(4);
 let remote=open_at(&||Instant::now()<end,&files.token()).unwrap();assert_eq!(std::fs::read_to_string(files.token()).unwrap(),"private-restored-choice");
 let result=super::super::paste::sequence(true,&||Instant::now()<end,|k,down|key(&remote,match k{super::super::paste::Key::Control=>XK_CONTROL_L,super::super::paste::Key::Shift=>XK_SHIFT_L,super::super::paste::Key::V=>XK_V},down,&||Instant::now()<end));assert_eq!(result,super::super::paste::Dispatch::Dispatched);let session=remote.session.clone();drop(remote);
 assert_eq!(fake.count("CreateSession"),1);assert_eq!(fake.count("SelectDevices"),1);assert_eq!(fake.count("Start"),1);assert_eq!(fake.count("NotifyKeyboardKeysym"),6);assert!(fake.events.lock().unwrap().iter().any(|(n,p)|n=="Close"&&p==&session));
}
#[test]#[ignore="synthetic complete RemoteDesktop opening; private SDK D-Bus and temporary token path only"]
fn cancelled_start_preserves_previous_token_and_never_sends_keys(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::HoldStart);let files=Files::new();std::fs::write(files.token(),"previous-choice").unwrap();let end=Instant::now()+Duration::from_secs(4);
 assert!(matches!(open_at(&||Instant::now()<end&&!fake.cancelled.load(Ordering::Acquire),&files.token()),Err(Failure::Cancelled)));assert_eq!(std::fs::read_to_string(files.token()).unwrap(),"previous-choice");assert_eq!(fake.count("Start"),1);assert_eq!(fake.count("NotifyKeyboardKeysym"),0);assert!(fake.events.lock().unwrap().iter().filter(|(n,_)|n=="Close").count()>=2);
}
#[test]#[ignore="synthetic complete RemoteDesktop opening; private SDK D-Bus and temporary token path only"]
fn returned_foreign_session_is_never_used_or_closed(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::WrongSession);let files=Files::new();let end=Instant::now()+Duration::from_secs(4);
 assert!(matches!(open_at(&||Instant::now()<end,&files.token()),Err(Failure::Unknown)));assert_eq!(fake.count("SelectDevices"),0);assert!(!files.token().exists());assert!(fake.events.lock().unwrap().iter().filter(|(n,_)|n=="Close").all(|(_,p)|!p.ends_with("_foreign")));
}
#[test]#[ignore="synthetic complete RemoteDesktop opening; private SDK D-Bus and temporary token path only"]
fn cancel_after_ctrl_dispatch_releases_ctrl_and_never_presses_v(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::CancelKey);let files=Files::new();let end=Instant::now()+Duration::from_secs(4);
 let current=||Instant::now()<end&&!fake.cancelled.load(Ordering::Acquire);let remote=open_at(&current,&files.token()).unwrap();
 let result=super::super::paste::sequence(true,&current,|k,down|key(&remote,match k{super::super::paste::Key::Control=>XK_CONTROL_L,super::super::paste::Key::Shift=>XK_SHIFT_L,super::super::paste::Key::V=>XK_V},down,&current));assert_eq!(result,super::super::paste::Dispatch::Uncertain);drop(remote);
 assert_eq!(fake.count(&format!("key:{XK_CONTROL_L}:1")),1);assert_eq!(fake.count(&format!("key:{XK_CONTROL_L}:0")),1);assert_eq!(fake.count(&format!("key:{XK_V}:1")),0);
}

#[test]#[ignore="synthetic complete RemoteDesktop opening; private SDK D-Bus and temporary token path only"]
fn a_revoked_key_call_before_dispatch_has_no_keyboard_effect(){
 let _serial=SERIAL.lock().unwrap();let fake=Fake::new(Mode::Open);let files=Files::new();let end=Instant::now()+Duration::from_secs(4);let remote=open_at(&||Instant::now()<end,&files.token()).unwrap();assert_eq!(key(&remote,XK_CONTROL_L,true,&||false),Err(super::super::paste::KeyFailure::NotSent));drop(remote);assert_eq!(fake.count("NotifyKeyboardKeysym"),0);
}
