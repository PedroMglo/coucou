//! Dictation clipboard and key dispatch. A dispatch is not destination ACK.
#[cfg(any(windows,test))]use std::sync::{Mutex,TryLockError};
use std::time::{Duration,Instant};
#[cfg(windows)]use arboard::Clipboard;

#[cfg(windows)]static CLIPBOARD:Mutex<Option<Clipboard>>=Mutex::new(None);
const MAX_TEXT_BYTES:usize=8*1024*1024;

#[cfg(any(windows,test))]fn with_current<C,T>(slot:&Mutex<Option<C>>,current:&dyn Fn()->bool,init:impl FnOnce()->Option<C>,action:impl FnOnce(&mut C)->T)->Option<T>{
    let mut guard=loop{
        if !current(){return None;}
        match slot.try_lock(){Ok(guard)=>break guard,Err(TryLockError::WouldBlock)=>std::thread::sleep(Duration::from_millis(2)),Err(_)=>return None}
    };
    if !current(){return None;}
    if guard.is_none(){*guard=init();}
    if !current(){return None;}
    guard.as_mut().map(action)
}
#[cfg(windows)]fn set_text(clipboard:&mut Clipboard,text:&str)->Result<(),arboard::Error>{
    #[cfg(windows)]{use arboard::SetExtWindows;clipboard.set().exclude_from_history().exclude_from_cloud().text(text.to_string())}
    #[cfg(target_os="linux")]{use arboard::SetExtLinux;clipboard.set().exclude_from_history().text(text.to_string())}
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Mutation{Skipped,Done,Uncertain}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Dispatch{NotSent,Dispatched,Uncertain}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(super)enum KeyFailure{NotSent,Uncertain}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Outcome{NotStarted,OutputTooLarge,PreservationBlocked,CopyUncertain,Copied,DispatchUncertain,Dispatched}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum PreviousState{Unchecked,Text,Vacant,NoText,Unavailable,TooLarge,Retired}
// Never Debug the text returned from the native clipboard.
enum Previous{Text(String),Vacant,NoText,Unavailable,TooLarge,Retired}
impl Previous{fn state(&self)->PreviousState{match self{Self::Text(_)=>PreviousState::Text,Self::Vacant=>PreviousState::Vacant,Self::NoText=>PreviousState::NoText,Self::Unavailable=>PreviousState::Unavailable,Self::TooLarge=>PreviousState::TooLarge,Self::Retired=>PreviousState::Retired}}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Restore{NotRequested,NoPrevious,Retired,Changed,Unavailable,TooLarge,WriteSkipped,WriteUncertain,WriteAcknowledged}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Report{pub outcome:Outcome,pub previous:PreviousState,pub restore:Restore}
impl Report{
    pub fn retain_output(&self)->bool{self.outcome!=Outcome::OutputTooLarge}
    pub fn preservation_message(&self)->&'static str{match self.previous{
        PreviousState::NoText=>"O clipboard anterior não tem texto preservável; pode conter outro formato.",
        PreviousState::TooLarge=>"O texto anterior do clipboard excede o limite de preservação.",
        PreviousState::Retired=>"A observação do clipboard deixou de ser válida.",
        _=>"Não foi possível observar e preservar o clipboard anterior.",
    }}
    pub fn restore_warning(&self)->Option<&'static str>{match self.restore{
        Restore::Unavailable|Restore::TooLarge|Restore::WriteSkipped=>Some("Não foi possível repor o clipboard anterior; o texto copiado pode continuar lá."),
        Restore::WriteUncertain=>Some("A reposição do clipboard ficou incerta. Confirma o conteúdo antes de outra cópia."),
        _=>None,
    }}
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(super)enum Key{Control,Shift,V}

/// Cancel prevents new presses. Releases of our attempted presses are cleanup,
/// including a press whose acknowledgement was lost. No sequence is replayed.
pub(super)fn sequence(shift:bool,current:&dyn Fn()->bool,mut send:impl FnMut(Key,bool)->Result<(),KeyFailure>)->Dispatch{
    let mut held=Vec::with_capacity(3);let mut began=false;
    let mut seq=vec![(Key::Control,true)];
    if shift{seq.push((Key::Shift,true));}
    seq.extend([(Key::V,true),(Key::V,false)]);
    if shift{seq.push((Key::Shift,false));}seq.push((Key::Control,false));
    for (key,down)in seq{
        if !current(){for key in held.into_iter().rev(){let _=send(key,false);}return if began{Dispatch::Uncertain}else{Dispatch::NotSent};}
        match send(key,down){
            Ok(())=>{began=true;if down{held.push(key);}else{held.retain(|k|*k!=key);}},
            Err(failure)=>{
                if down&&failure==KeyFailure::Uncertain{held.push(key);}
                for key in held.into_iter().rev(){let _=send(key,false);}
                return if began||failure==KeyFailure::Uncertain{Dispatch::Uncertain}else{Dispatch::NotSent};
            },
        }
    }
    Dispatch::Dispatched
}

fn wait_current(duration:Duration,current:&dyn Fn()->bool)->bool{
    let end=Instant::now()+duration;
    loop{if !current(){return false;}let now=Instant::now();if now>=end{return true;}std::thread::sleep((end-now).min(Duration::from_millis(5)));}
}
trait Port{
    fn observe(&mut self,current:&dyn Fn()->bool)->Previous;
    fn put(&mut self,text:&str,current:&dyn Fn()->bool)->Mutation;
    fn dispatch(&mut self,shift:bool,current:&dyn Fn()->bool)->Dispatch;
}
struct Native;
impl Port for Native{
    fn observe(&mut self,current:&dyn Fn()->bool)->Previous{
        #[cfg(target_os="linux")]{
            match super::clipboard_read::read(current){
                Ok(super::clipboard_read::Reply::Text(text))=>Previous::Text(text),
                Ok(super::clipboard_read::Reply::Empty)=>Previous::NoText,
                Ok(super::clipboard_read::Reply::Vacant)=>Previous::Vacant,
                Ok(super::clipboard_read::Reply::Unavailable)=>Previous::Unavailable,
                Ok(super::clipboard_read::Reply::TooLarge)=>Previous::TooLarge,
                Err(super::clipboard_read::Failure::Retired)=>Previous::Retired,
                Err(_)=>Previous::Unavailable,
            }
        }
        #[cfg(not(target_os="linux"))]{
            with_current(&CLIPBOARD,current,||Clipboard::new().ok(),|c|match c.get_text(){Ok(text)if text.len()<=MAX_TEXT_BYTES=>Previous::Text(text),Ok(_)=>Previous::TooLarge,Err(arboard::Error::ContentNotAvailable)=>Previous::NoText,Err(_)=>Previous::Unavailable}).unwrap_or(Previous::Unavailable)
        }
    }
    fn put(&mut self,text:&str,current:&dyn Fn()->bool)->Mutation{
        #[cfg(target_os="linux")]{super::clipboard_write::write(text,current)}
        #[cfg(windows)]{
        with_current(&CLIPBOARD,current,||Clipboard::new().ok(),|c|{
            if !current(){Mutation::Skipped}else if set_text(c,text).is_ok(){Mutation::Done}else{Mutation::Uncertain}
        }).unwrap_or(Mutation::Skipped)
        }
    }
    fn dispatch(&mut self,shift:bool,current:&dyn Fn()->bool)->Dispatch{super::os::send_paste(shift,current)}
}
fn perform(port:&mut impl Port,text:&str,shift:bool,current:&dyn Fn()->bool,announce:Duration,restore_after:Duration)->Report{
    let mut report=Report{outcome:Outcome::NotStarted,previous:PreviousState::Unchecked,restore:Restore::NotRequested};
    if text.len()>MAX_TEXT_BYTES{report.outcome=Outcome::OutputTooLarge;return report;}
    if !current(){return report;}
    let previous=port.observe(current);report.previous=previous.state();
    if !current(){return report;}
    let previous=match previous{
        Previous::Text(text)=>Some(text),Previous::Vacant=>None,
        Previous::Retired=>return report,
        _=>{report.outcome=Outcome::PreservationBlocked;return report;},
    };
    match port.put(text,current){Mutation::Skipped=>return report,Mutation::Uncertain=>{report.outcome=Outcome::CopyUncertain;return report;},Mutation::Done=>report.outcome=Outcome::Copied}
    if !wait_current(announce,current){return report;}
    report.outcome=match port.dispatch(shift,current){Dispatch::NotSent=>Outcome::Copied,Dispatch::Uncertain=>Outcome::DispatchUncertain,Dispatch::Dispatched=>Outcome::Dispatched};
    if report.outcome!=Outcome::Dispatched{return report;}
    if !wait_current(restore_after,current){report.restore=Restore::Retired;return report;}
    let Some(previous)=previous else{report.restore=Restore::NoPrevious;return report;};
    // The same bounded observer is used outside the write mutex. Content-only
    // matching remains a known limit; this is not a native revision receipt.
    let observed=port.observe(current);
    if !current(){report.restore=Restore::Retired;return report;}
    report.restore=match observed{
        Previous::Text(ours)if ours==text=>{
            if !current(){Restore::Retired}else{match port.put(&previous,current){Mutation::Skipped=>Restore::WriteSkipped,Mutation::Uncertain=>Restore::WriteUncertain,Mutation::Done=>Restore::WriteAcknowledged}}
        },
        Previous::Text(_)|Previous::Vacant|Previous::NoText=>Restore::Changed,
        Previous::Unavailable=>Restore::Unavailable,Previous::TooLarge=>Restore::TooLarge,Previous::Retired=>Restore::Retired,
    };
    report
}
/// The occurrence remains owned through the delayed restoration. No detached
/// cleanup can write after its scope is retired or admit another Flow writer.
pub fn paste(text:&str,shift:bool,current:&dyn Fn()->bool)->Report{
    let report=perform(&mut Native,text,shift,current,Duration::from_millis(40),Duration::from_millis(700));
    crate::log::line(format!("flow: paste outcome={:?} previous={:?} restore={:?}",report.outcome,report.previous,report.restore));report
}
pub fn copy(text:&str,current:&dyn Fn()->bool)->bool{
    if text.len()>MAX_TEXT_BYTES||!current(){return false;}
    let result=Native.put(text,current);
    crate::log::line(format!("flow: explicit copy outcome={result:?}"));result==Mutation::Done
}
#[cfg(test)]#[path="paste_tests.rs"]mod tests;
