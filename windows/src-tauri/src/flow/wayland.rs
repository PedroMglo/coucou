// Flow dictation on Wayland, where no app may watch the keyboard or type into
// another one.
//
// The key: a shortcut held through the desktop's GlobalShortcuts portal
// (portal.rs). The desktop tells when it is pressed (Activated) and released
// (Deactivated), which is all hold-to-talk needs; the user picks the keys in
// the desktop's own dialog, Ctrl+Alt+V suggested.
//
// The paste: Ctrl+V sent through the RemoteDesktop portal, keyboard only. The
// desktop asks once whether Coucou may control the keyboard; the answer is
// remembered with the restore token the portal gives back, kept in Coucou's
// folder. Where neither works, the words stay on the clipboard and the pill
// says so.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dbus::arg::{PropMap, RefArg, Variant};
use dbus::blocking::Connection;
use dbus::channel::{MatchingReceiver,Token};
use dbus::message::MatchRule;
use dbus::Path;

use super::KeyEvent;
use crate::portal;

pub const SHORTCUT_ID: &str = "flowDictate";

/// The portal session holding the dictation shortcut; dropping it gives the
/// shortcut back.
pub struct HoldKey {
    _handle: portal::Handle,
}

pub fn listen(tx: Sender<(KeyEvent, Instant)>) -> Option<HoldKey> {
    let handle = portal::start(move |event| match event {
        portal::Event::Activated { gen: 1, id } if id == SHORTCUT_ID => {
            crate::log::line("flow: shortcut pressed");
            let _ = tx.send((KeyEvent::Down, Instant::now()));
        }
        portal::Event::Deactivated { gen: 1, id } if id == SHORTCUT_ID => {
            crate::log::line("flow: shortcut released");
            let _ = tx.send((KeyEvent::Up, Instant::now()));
        }
        portal::Event::Activated { gen: 1, id } if id == "flowRelease" => {
            crate::log::line("flow: shortcut release recovered");
            let _ = tx.send((KeyEvent::Up, Instant::now()));
        }
        portal::Event::Activated { gen: 1, id } if id == "voiceTalk" => {
            let _ = tx.send((KeyEvent::Voice, Instant::now()));
        }
        portal::Event::Activated { gen: 1, id } if id == "flowCancel" => {
            let _ = tx.send((KeyEvent::Cancel, Instant::now()));
        }
        portal::Event::Activated { gen: 1, id } if id == "flowFinish" => {
            let _ = tx.send((KeyEvent::Finish, Instant::now()));
        }
        portal::Event::Bound { gen: 1, .. } => crate::log::line("flow: dictation shortcut held by the desktop"),
        portal::Event::Failed { gen: 1, reason } | portal::Event::Unavailable(reason) => {
            let _ = tx.send((KeyEvent::Cancel, Instant::now()));
            crate::log::line(format!("flow: no dictation shortcut from the desktop ({reason})"));
        }
        _ => {}
    })?;
    let wanted = portal::Wanted {
        id: SHORTCUT_ID.into(),
        description: crate::i18n::t("Hold to dictate"),
        trigger: Some("CTRL+ALT+v".into()),
    };
    let cancel = portal::Wanted { id: "flowCancel".into(), description: crate::i18n::t("Cancel"), trigger: Some("CTRL+ALT+Escape".into()) };
    let finish = portal::Wanted { id: "flowFinish".into(), description: crate::i18n::t("Finish"), trigger: Some("CTRL+ALT+Return".into()) };
    let release = portal::Wanted { id: "flowRelease".into(), description: crate::i18n::t("Hold to dictate"), trigger: None };
    let voice = portal::Wanted { id: "voiceTalk".into(), description: crate::i18n::t("Talk to Mochi"), trigger: Some("CTRL+ALT+c".into()) };
    if !handle.bind(1, vec![wanted, cancel, finish, release, voice]) {
        return None;
    }
    Some(HoldKey { _handle: handle })
}

// ── The paste ─────────────────────────────────────────────────────────────────

const DEST: &str = "org.freedesktop.portal.Desktop";
const OBJECT: &str = "/org/freedesktop/portal/desktop";
const IFACE: &str = "org.freedesktop.portal.RemoteDesktop";
/// Keysyms of Ctrl, Shift and v.
const XK_CONTROL_L: i32 = 0xffe3;
const XK_SHIFT_L: i32 = 0xffe1;
const XK_V: i32 = 0x0076;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Failure{Cancelled,Refused,Unknown}
const BUS:&str="org.freedesktop.DBus";
const BUS_PATH:&str="/org/freedesktop/DBus";
struct Remote{conn:Connection,session:String,peer:String}
impl Drop for Remote{fn drop(&mut self){let ack=close(&self.conn,&self.peer,&self.session,"org.freedesktop.portal.Session");crate::log::line(format!("flow: own RemoteDesktop close acknowledged={ack}"));}}
static REMOTE:Mutex<Option<Remote>>=Mutex::new(None);
static REFUSED:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
fn token_file()->std::path::PathBuf{super::store::Store::folder().join("remote-desktop-token")}
fn variant<T:RefArg+'static>(v:T)->Variant<Box<dyn RefArg>>{Variant(Box::new(v))}
fn method(peer:&str,path:&str,interface:&str,member:&str)->Result<dbus::Message,Failure>{dbus::Message::new_method_call(peer,path,interface,member).map_err(|_|Failure::Unknown)}

/// Same D-Bus connection, serial and unique peer; no retry after dispatch.
/// process() pumps signals too, so a Response arriving before its method ACK
/// is retained by the exact request filter rather than lost.
fn call(conn:&Connection,msg:dbus::Message,peer:&str,current:&dyn Fn()->bool,timeout:Duration)->Result<dbus::Message,Failure>{
    call_tracked(conn,msg,peer,current,timeout,&std::cell::Cell::new(false))
}
fn call_tracked(conn:&Connection,msg:dbus::Message,peer:&str,current:&dyn Fn()->bool,timeout:Duration,sent:&std::cell::Cell<bool>)->Result<dbus::Message,Failure>{
    if !current(){return Err(Failure::Cancelled);}
    sent.set(true);
    let serial=conn.channel().send(msg).map_err(|_|Failure::Unknown)?;
    let answer=Arc::new(Mutex::new(None));let seen=answer.clone();
    let mut rule=MatchRule::new();rule.sender=Some(peer.to_string().into());
    let token=conn.start_receive(rule,Box::new(move|msg,_|{
        if msg.get_reply_serial()==Some(serial){*seen.lock().unwrap()=Some(msg);}true
    }));
    let until=Instant::now()+timeout;
    let result=loop{
        if !current(){break Err(Failure::Cancelled);}
        if let Some(mut reply)=answer.lock().unwrap().take(){
            if reply.sender().as_deref()!=Some(peer)||reply.as_result().is_err(){break Err(Failure::Unknown);}
            break Ok(reply);
        }
        if Instant::now()>=until{break Err(Failure::Unknown);}
        if conn.process(Duration::from_millis(20)).is_err(){break Err(Failure::Unknown);}
    };
    conn.stop_receive(token);result
}
fn close(conn:&Connection,peer:&str,path:&str,interface:&str)->bool{
    let Ok(msg)=method(peer,path,interface,"Close")else{return false;};
    call(conn,msg,peer,&||true,Duration::from_millis(500)).is_ok()
}
struct Request<'a>{conn:&'a Connection,peer:&'a str,path:String,rule:String,filter:Token,answered:bool}
impl Drop for Request<'_>{fn drop(&mut self){
    self.conn.stop_receive(self.filter);
    if let Ok(msg)=method(BUS,BUS_PATH,BUS,"RemoveMatch"){let _=call(self.conn,msg.append1(self.rule.clone()),BUS,&||true,Duration::from_millis(500));}
    if !self.answered{let ack=close(self.conn,self.peer,&self.path,"org.freedesktop.portal.Request");crate::log::line(format!("flow: own paste request close acknowledged={ack}"));}
}}
fn request(conn:&Connection,peer:&str,_member:&str,args:impl FnOnce(&str)->Result<dbus::Message,Failure>,tokens:&mut u32,current:&dyn Fn()->bool)->Result<PropMap,Failure>{
    if !current(){return Err(Failure::Cancelled);}
    *tokens=tokens.checked_add(1).ok_or(Failure::Unknown)?;
    let token=format!("coucoupaste{}_{}",std::process::id(),tokens);
    let path=portal::request_path(&conn.unique_name().to_string(),&token);
    let answer:Arc<Mutex<Option<Result<PropMap,u32>>>>=Arc::new(Mutex::new(None));let seen=answer.clone();
    let expected_peer=peer.to_string();let expected_path=path.clone();
    let rule=MatchRule::new_signal("org.freedesktop.portal.Request","Response").with_sender(peer.to_string()).with_path(Path::from(path.clone()));
    let match_text=rule.match_str();
    call(conn,method(BUS,BUS_PATH,BUS,"AddMatch")?.append1(match_text.clone()),BUS,current,Duration::from_secs(10))?;
    let filter=conn.start_receive(rule,Box::new(move|msg,_|{
        if msg.sender().as_deref()==Some(expected_peer.as_str())&&msg.path().as_deref()==Some(expected_path.as_str()){
            let mut slot=seen.lock().unwrap();if slot.is_none(){*slot=Some(portal::parse_response(&msg));}
        }true
    }));
    let mut guard=Request{conn,peer,path,rule:match_text,filter,answered:false};
    let reply=call(conn,args(&token)?,peer,current,Duration::from_secs(10))?;
    let actual:Path<'static>=reply.read1().map_err(|_|Failure::Unknown)?;
    if actual.to_string()!=guard.path{return Err(Failure::Unknown);}
    let until=Instant::now()+Duration::from_secs(120);
    loop{
        if !current(){return Err(Failure::Cancelled);}
        if let Some(result)=answer.lock().unwrap().take(){
            guard.answered=true;
            return match result{Ok(results)=>Ok(results),Err(1)=>Err(Failure::Refused),Err(_)=>Err(Failure::Unknown)};
        }
        if Instant::now()>=until{return Err(Failure::Unknown);}
        if conn.process(Duration::from_millis(20)).is_err(){return Err(Failure::Unknown);}
    }
}
fn open(current:&dyn Fn()->bool)->Result<Remote,Failure>{open_at(current,&token_file())}
fn open_at(current:&dyn Fn()->bool,token_path:&std::path::Path)->Result<Remote,Failure>{
    if !current(){return Err(Failure::Cancelled);}
    let conn=Connection::new_session().map_err(|_|Failure::Unknown)?;
    let reply=call(&conn,method(BUS,BUS_PATH,BUS,"GetNameOwner")?.append1(DEST),BUS,current,Duration::from_secs(10))?;
    let peer:String=reply.read1().map_err(|_|Failure::Unknown)?;
    if !peer.starts_with(':')||dbus::strings::BusName::new(peer.clone()).is_err(){return Err(Failure::Unknown);}
    static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1);
    let id=NEXT.fetch_update(std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire,|n|n.checked_add(1)).map_err(|_|Failure::Unknown)?;
    let session_token=format!("coucoupastes{}_{}",std::process::id(),id);
    let session=portal::session_path(&conn.unique_name().to_string(),&session_token);
    // Own the predicted session before CreateSession can have an effect.
    let remote=Remote{conn,session,peer};let mut tokens=0;
    let results=request(&remote.conn,&remote.peer,"CreateSession",|token|{
        let mut options:PropMap=HashMap::new();options.insert("handle_token".into(),variant(token.to_string()));options.insert("session_handle_token".into(),variant(session_token));
        Ok(method(&remote.peer,OBJECT,IFACE,"CreateSession")?.append1(options))
    },&mut tokens,current)?;
    if portal::session_from_results(&results).as_deref()!=Some(remote.session.as_str()){return Err(Failure::Unknown);}
    if !current(){return Err(Failure::Cancelled);}
    let restore={use std::io::Read;let mut text=String::new();std::fs::File::open(token_path).ok().and_then(|file|file.take(4097).read_to_string(&mut text).ok()).and_then(|_|(text.len()<=4096&&!text.trim().is_empty()&&!text.chars().any(char::is_control)).then_some(text))};
    request(&remote.conn,&remote.peer,"SelectDevices",|token|{
        let mut options:PropMap=HashMap::new();options.insert("handle_token".into(),variant(token.to_string()));options.insert("types".into(),variant(1u32));options.insert("persist_mode".into(),variant(2u32));
        if let Some(restore)=restore{options.insert("restore_token".into(),variant(restore.trim().to_string()));}
        Ok(method(&remote.peer,OBJECT,IFACE,"SelectDevices")?.append2(Path::from(remote.session.clone()),options))
    },&mut tokens,current)?;
    let results=request(&remote.conn,&remote.peer,"Start",|token|{
        let mut options:PropMap=HashMap::new();options.insert("handle_token".into(),variant(token.to_string()));
        Ok(method(&remote.peer,OBJECT,IFACE,"Start")?.append3(Path::from(remote.session.clone()),"",options))
    },&mut tokens,current)?;
    if !current(){return Err(Failure::Cancelled);}
    if let Some(token)=results.get("restore_token").and_then(|v|v.0.as_str()).filter(|t|t.len()<=4096&&!t.chars().any(char::is_control)){
        if !current(){return Err(Failure::Cancelled);}
        if let Some(folder)=token_path.parent(){if crate::platform::ensure_private_dir(folder).is_ok()&&current(){let _=std::fs::write(token_path,token);}}
    }
    if !current(){return Err(Failure::Cancelled);}Ok(remote)
}
fn key(remote:&Remote,sym:i32,down:bool,current:&dyn Fn()->bool)->Result<(),super::paste::KeyFailure>{
    use super::paste::KeyFailure;
    let msg=method(&remote.peer,OBJECT,IFACE,"NotifyKeyboardKeysym").map_err(|_|KeyFailure::NotSent)?.append3(Path::from(remote.session.clone()),PropMap::new(),sym).append1(down as u32);
    let sent=std::cell::Cell::new(false);
    call_tracked(&remote.conn,msg,&remote.peer,if down{current}else{&||true},Duration::from_secs(5),&sent).map(|_|()).map_err(|_|if sent.get(){KeyFailure::Uncertain}else{KeyFailure::NotSent})
}

pub fn send_paste(shift:bool,current:&dyn Fn()->bool)->super::paste::Dispatch{
    use super::paste::{Dispatch,Key};use std::sync::atomic::Ordering;
    if !current()||REFUSED.load(Ordering::Acquire){return Dispatch::NotSent;}
    let mut slot=loop{if !current(){return Dispatch::NotSent;}match REMOTE.try_lock(){Ok(slot)=>break slot,Err(std::sync::TryLockError::WouldBlock)=>std::thread::sleep(Duration::from_millis(2)),Err(_)=>return Dispatch::NotSent}};
    if !current(){return Dispatch::NotSent;}
    if slot.is_none(){match open(current){Ok(remote)=>*slot=Some(remote),Err(Failure::Refused)=>{REFUSED.store(true,Ordering::Release);return Dispatch::NotSent;},Err(_)=>return Dispatch::NotSent}}
    let remote=slot.as_ref().unwrap();
    let result=super::paste::sequence(shift,current,|k,down|key(remote,match k{Key::Control=>XK_CONTROL_L,Key::Shift=>XK_SHIFT_L,Key::V=>XK_V},down,current));
    if result==Dispatch::Uncertain{*slot=None;}result
}

#[cfg(test)]#[path="paste_portal_tests.rs"]mod paste_tests;
