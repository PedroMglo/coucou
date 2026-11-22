use super::*;
use dbus::{Message, message::MessageType, arg::{Variant, RefArg}};
use std::{collections::HashMap, process::{Child,Command,Stdio}, io::{BufRead,BufReader}, sync::{Arc,atomic::{AtomicBool,AtomicU8,AtomicU64,Ordering}}, thread};
pub(crate) struct Bus { child:Child,pub(crate) address:String }
impl Bus {
    pub(crate) fn new()->Self{
        let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let address=format!("unix:abstract=coucou-goal-spotify-{}-{stamp}",std::process::id());
        let mut child=Command::new("dbus-daemon").args(["--session","--nofork","--nopidfile","--print-address=1","--address",&address]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let stdout=child.stdout.take().unwrap();let mut bus=Self{child,address:String::new()};BufReader::new(stdout).read_line(&mut bus.address).unwrap();bus.address=bus.address.trim().into();assert!(bus.address.starts_with("unix:"));bus
    }
}
impl Drop for Bus {fn drop(&mut self){let _=self.child.kill();let _=self.child.wait();}}
#[derive(Clone)]pub(crate) struct State { pub(crate) status:String,pub(crate) volume:f64 }
pub(crate) struct Player { stop:Arc<AtomicBool>, worker:Option<thread::JoinHandle<()>>, pub(crate) calls:Arc<Mutex<Vec<(String,String)>>>, pub(crate) mode:Arc<AtomicU8>,pub(crate) delay:Arc<AtomicU64>,pub(crate) state:Arc<Mutex<State>>,pub(crate) entered:Arc<AtomicBool> }
fn var<T:RefArg+'static>(value:T)->Variant<Box<dyn RefArg>>{Variant(Box::new(value))}
impl Player {
    pub(crate) fn new(bus:&Bus,identity:&str)->Self{
        let stop=Arc::new(AtomicBool::new(false));let calls=Arc::new(Mutex::new(vec![]));let mode=Arc::new(AtomicU8::new(0));let delay=Arc::new(AtomicU64::new(0));let state=Arc::new(Mutex::new(State{status:"Playing".into(),volume:0.8}));
        let(stop2,calls2,mode2,delay2,state2)=(stop.clone(),calls.clone(),mode.clone(),delay.clone(),state.clone());let entered=Arc::new(AtomicBool::new(false));let entered2=entered.clone();let address=bus.address.clone();let identity=identity.to_string();let(tx,rx)=std::sync::mpsc::channel();
        let worker=thread::spawn(move||{
            let conn=Connection::new_address(&address).unwrap();conn.request_name(BUS,false,true,true).unwrap();tx.send(()).unwrap();
            while !stop2.load(Ordering::Acquire){
                let Ok(Some(message))=conn.channel().blocking_pop_message(Duration::from_millis(10))else{continue};
                if message.msg_type()!=MessageType::MethodCall{continue;}
                let member=message.member().unwrap().to_string();if matches!(member.as_str(),"Play"|"Pause"|"Set"){entered2.store(true,Ordering::Release);}let mut command=false;
                let reply=match member.as_str(){
                    "Get"=>{let(_,property):(String,String)=message.read2().unwrap();if property=="Identity"{message.method_return().append1(var(identity.clone()))}else{message.method_return().append1(var(true))}},
                    "GetAll"=>{let state=state2.lock().unwrap().clone();let mut properties:HashMap<String,Variant<Box<dyn RefArg>>>=HashMap::new();properties.insert("PlaybackStatus".into(),var(state.status));properties.insert("Volume".into(),var(state.volume));properties.insert("CanControl".into(),var(true));message.method_return().append1(properties)},
                    "Play"|"Pause"=>{
                        command=true;thread::sleep(Duration::from_millis(delay2.load(Ordering::Acquire)));
                        if mode2.load(Ordering::Acquire)!=1{state2.lock().unwrap().status=if member=="Play"{"Playing"}else{"Paused"}.into();}
                        message.method_return()
                    },
                    "Set"=>{
                        command=true;let(_,property,value):(String,String,Variant<Box<dyn RefArg>>)=message.read3().unwrap();assert_eq!(property,"Volume");
                        if mode2.load(Ordering::Acquire)!=1{state2.lock().unwrap().volume=value.0.as_f64().unwrap();}message.method_return()
                    },
                    _=>panic!("unexpected mock request {member}"),
                };
                let reply=if command{
                    calls2.lock().unwrap().push((member,message.destination().unwrap().to_string()));
                    if mode2.load(Ordering::Acquire)==2{Message::error(&message,&"org.freedesktop.DBus.Error.Failed".into(),&std::ffi::CString::new("effect happened; ACK lost").unwrap())}else{reply}
                }else{reply};
                let _=conn.channel().send(reply);
            }
        });
        let player=Self{stop,worker:Some(worker),calls,mode,delay,state,entered};rx.recv_timeout(Duration::from_secs(3)).unwrap();player
    }
}
impl Drop for Player {fn drop(&mut self){self.stop.store(true,Ordering::Release);if let Some(worker)=self.worker.take(){worker.join().unwrap();}}}
fn deadline()->Instant{Instant::now()+Duration::from_secs(10)}
#[test]
fn capture_is_read_only_and_commands_target_the_reviewed_unique_owner(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");let target=Target::at(&bus.address).unwrap();assert!(player.calls.lock().unwrap().is_empty());
    assert_eq!(target.binding().identity,"Private Spotify");assert_eq!(target.binding().uid,unsafe{libc::geteuid()});assert_eq!(target.binding().pid,std::process::id());
    let debits=std::cell::Cell::new(0);let result=target.apply(&Action::Pause,deadline(),||true,||{debits.set(debits.get()+1);Ok(())});
    assert_eq!(result.delivery,Delivery::Accepted);assert!(result.effect_verified);assert_eq!(debits.get(),1);
    assert_eq!(result.before.unwrap().playback_status,"Playing");assert_eq!(result.after.unwrap().playback_status,"Paused");
    assert_eq!(*player.calls.lock().unwrap(),vec![("Pause".into(),target.binding().owner.clone())]);
}
#[test]
fn play_and_volume_use_explicit_state_postconditions_without_clamping(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");let target=Target::at(&bus.address).unwrap();player.state.lock().unwrap().status="Paused".into();
    assert!(target.apply(&Action::Play,deadline(),||true,||Ok(())).effect_verified);
    let volume=target.apply(&Action::SetVolume{volume:17},deadline(),||true,||Ok(()));assert!(volume.effect_verified);assert_eq!(volume.after.unwrap().volume,0.17);
    let invalid=target.apply(&Action::SetVolume{volume:101},deadline(),||true,||panic!("invalid volume charged"));assert_eq!(invalid.failure,Some(Failure::InvalidAction));assert_eq!(player.calls.lock().unwrap().len(),2);
}
#[test]
fn an_ack_without_a_matching_native_postcondition_does_not_verify_the_effect(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");player.mode.store(1,Ordering::Release);let target=Target::at(&bus.address).unwrap();
    let result=target.apply(&Action::Pause,deadline(),||true,||Ok(()));assert_eq!(result.delivery,Delivery::Accepted);assert!(!result.effect_verified);assert_eq!(result.after.unwrap().playback_status,"Playing");assert_eq!(player.calls.lock().unwrap().len(),1);
}
#[test]
fn an_error_after_the_effect_is_reconciled_by_reading_without_repeating_the_command(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");player.mode.store(2,Ordering::Release);let target=Target::at(&bus.address).unwrap();
    let result=target.apply(&Action::Pause,deadline(),||true,||Ok(()));assert_eq!(result.delivery,Delivery::Uncertain);assert_eq!(result.failure,Some(Failure::Transport));assert!(result.effect_verified);assert_eq!(player.calls.lock().unwrap().len(),1);
}
#[test]
fn a_replacement_owner_is_never_followed_with_the_old_target(){
    let bus=Bus::new();let first=Player::new(&bus,"First Spotify");let target=Target::at(&bus.address).unwrap();drop(first);let second=Player::new(&bus,"Second Spotify");
    let result=target.apply(&Action::Pause,deadline(),||true,||panic!("replacement charged"));assert_eq!(result.delivery,Delivery::NotDispatched);assert_eq!(result.failure,Some(Failure::Changed));assert!(second.calls.lock().unwrap().is_empty());
}
#[test]
fn cancelled_or_unauthorized_steps_never_dispatch_and_late_stop_is_checked(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");let target=Target::at(&bus.address).unwrap();
    let result=target.apply(&Action::Pause,deadline(),||false,||panic!("stopped step charged"));assert_eq!(result.failure,Some(Failure::Cancelled));
    let result=target.apply(&Action::Pause,deadline(),||true,||Err(Failure::NotAuthorized));assert_eq!(result.failure,Some(Failure::NotAuthorized));
    let current=std::cell::Cell::new(true);let result=target.apply(&Action::Pause,deadline(),||current.get(),||{current.set(false);Ok(())});assert_eq!(result.delivery,Delivery::NotDispatched);assert_eq!(result.failure,Some(Failure::Cancelled));assert!(player.calls.lock().unwrap().is_empty());
}
#[test]
fn deadline_during_native_io_preserves_the_observation_without_claiming_current_verification(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");let target=Target::at(&bus.address).unwrap();player.delay.store(500,Ordering::Release);
    let result=target.apply(&Action::Pause,Instant::now()+Duration::from_millis(250),||true,||Ok(()));assert_eq!(result.delivery,Delivery::Accepted);assert!(!result.scope_current);assert!(!result.effect_verified);assert_eq!(result.after.unwrap().playback_status,"Paused");assert_eq!(player.calls.lock().unwrap().len(),1);
}
#[test]
fn missing_owner_or_nonlocal_address_does_not_start_a_player_or_connect_to_a_fallback(){
    for address in ["tcp:host=127.0.0.1,port=1","unix:path=/none;tcp:host=127.0.0.1,port=1",""]{assert!(matches!(Target::at(address),Err(Failure::InvalidAddress)));}
    let bus=Bus::new();assert!(matches!(Target::at(&bus.address),Err(Failure::Unavailable)));
}
#[test]
fn invalid_native_state_is_refused_and_a_disconnected_target_is_not_reconnected(){
    let bus=Bus::new();let player=Player::new(&bus,"Private Spotify");player.state.lock().unwrap().volume=2.0;assert!(matches!(Target::at(&bus.address),Err(Failure::InvalidState)));
    player.state.lock().unwrap().volume=0.8;let target=Target::at(&bus.address).unwrap();drop(player);drop(bus);
    let result=target.apply(&Action::Pause,deadline(),||true,||panic!("disconnected step charged"));assert_eq!(result.delivery,Delivery::NotDispatched);assert!(!result.effect_verified);
}
