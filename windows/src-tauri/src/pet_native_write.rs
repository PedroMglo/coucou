//! Bounded writes to unbuffered native pipes. A decision retains its captured
//! source through every actual IO attempt; skipped writes never sent a byte.
use std::{io::{self,Write},sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Instant,Duration}};
use crate::pet_native_decisions::Ticket;
#[derive(Clone,Copy)]pub(crate)enum Use{Decision,Refusal,Withdrawal}
/// Chosen by the typed backend route, never by serialized caller/model data.
/// A refusal that persists a policy is still a new effect, not Control.
#[derive(Clone)]pub(crate)enum Dispatch{NewEffect,Control,#[cfg(test)]ProbedEffect(crate::settings::EffectProbe)}
impl Dispatch{
    fn allowed(&self)->bool{match self{Self::NewEffect=>crate::settings::effect_allowed().is_ok(),Self::Control=>true,#[cfg(test)]Self::ProbedEffect(probe)=>probe.check().is_ok()}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum Reason{CallerGone,Expired,Cancelled,SourceEnded,PreferencesUnconfirmed,Transport}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum Outcome{Written,Skipped(Reason)}
#[derive(Debug)]pub(crate)struct Fault{pub reason:Reason,pub partial:bool}
/// Transport-owned values only. No deserialization or frontend-supplied guard.
pub(crate)struct Guard{
    pub dispatch:Dispatch,
    pub deadline:Instant,pub cancel:Option<Arc<AtomicBool>>,pub caller:Option<Arc<AtomicBool>>,
    pub source:Option<(Ticket,Use)>,
}
impl Guard{
    pub(crate)fn reason(&self)->Option<Reason>{
        if self.caller.as_ref().is_some_and(|v|!v.load(Ordering::Acquire)){return Some(Reason::CallerGone);}
        if Instant::now()>=self.deadline{return Some(Reason::Expired);}
        if self.cancel.as_ref().is_some_and(|v|v.load(Ordering::Acquire)){return Some(Reason::Cancelled);}
        if self.source.as_ref().is_some_and(|(t,use_)|!t.current()&&!(matches!(use_,Use::Withdrawal)&&t.withdrawal_pending())){return Some(Reason::SourceEnded);}
        None
    }
    /// Do not mix this with callback/result lifetime. IO changes block another
    /// write, but do not turn an already observed result into an unstarted one.
    pub(crate)fn write_reason(&self)->Option<Reason>{self.reason().or_else(||(!self.dispatch.allowed()).then_some(Reason::PreferencesUnconfirmed))}
}
/// `writer` must be unbuffered: expiry must not leave buffered bytes to flush
/// later. A partial fault requires the owner to retire this exact pipe; no retry.
pub(crate)fn frame(writer:&mut impl Write,bytes:&[u8],guard:&Guard)->Result<Outcome,Fault>{
    let mut offset=0;
    while offset<bytes.len(){
        if let Some(reason)=guard.write_reason(){return if offset==0{Ok(Outcome::Skipped(reason))}else{Err(Fault{reason,partial:true})};}
        match writer.write(&bytes[offset..]){
            Ok(0)=>return Err(Fault{reason:Reason::Transport,partial:offset>0}),
            Ok(n)=>offset+=n,
            Err(e)if e.kind()==io::ErrorKind::Interrupted=>{},
            Err(e)if e.kind()==io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(2)),
            Err(_)=>return Err(Fault{reason:Reason::Transport,partial:offset>0}),
        }
    }
    // A full newline-terminated frame may already have been consumed/resolved.
    // Do not reinterpret that success as a skipped write after the final byte.
    writer.flush().map_err(|_|Fault{reason:Reason::Transport,partial:!bytes.is_empty()})?;
    Ok(Outcome::Written)
}
#[cfg(test)]mod tests{
    use super::*;use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture,PolicyBinding}};use serde_json::json;
    fn native()->(Endpoint,Ticket){let e=Endpoint::new(Adapter::Codex,Transport::ManagedPipe).unwrap();let t=e.capture(Capture{request_id:Some(&json!(7)),session_id:"s",turn_id:Some("t"),prompt_id:None,method:"native/request",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();(e,t)}
    fn guard(t:&Ticket)->Guard{Guard{dispatch:Dispatch::NewEffect,deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:None,source:Some((t.clone(),Use::Decision))}}
    struct BlockThen{bytes:Vec<u8>,on_first:Option<Box<dyn FnOnce()>>,partial:bool}
    impl Write for BlockThen{
        fn write(&mut self,b:&[u8])->io::Result<usize>{if let Some(f)=self.on_first.take(){if self.partial{self.bytes.extend_from_slice(&b[..2]);f();return Ok(2);}f();return Err(io::ErrorKind::WouldBlock.into());}self.bytes.extend_from_slice(b);Ok(b.len())}
        fn flush(&mut self)->io::Result<()>{Ok(())}
    }
    #[test]fn source_resolution_before_the_first_byte_skips_the_entire_frame(){let(_e,t)=native();t.cancel();let mut out=vec![];assert_eq!(frame(&mut out,b"grant\n",&guard(&t)).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert!(out.is_empty());}
    #[test]fn source_resolution_during_backpressure_is_seen_before_the_next_write(){let(_e,t)=native();let old=t.clone();let mut out=BlockThen{bytes:vec![],on_first:Some(Box::new(move||old.cancel())),partial:false};assert_eq!(frame(&mut out,b"grant\n",&guard(&t)).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert!(out.bytes.is_empty());}
    #[test]fn expiry_after_a_partial_write_never_completes_the_old_decision(){let(_e,t)=native();let old=t.clone();let mut out=BlockThen{bytes:vec![],on_first:Some(Box::new(move||old.cancel())),partial:true};let fault=frame(&mut out,b"grant\n",&guard(&t)).unwrap_err();assert!(fault.partial);assert_eq!(fault.reason,Reason::SourceEnded);assert_eq!(out.bytes,b"gr");}
    #[test]fn caller_departure_cancel_and_deadline_each_prevent_a_queued_write(){let(_e,t)=native();for reason in[Reason::CallerGone,Reason::Cancelled,Reason::Expired]{let mut g=guard(&t);match reason{Reason::CallerGone=>g.caller=Some(Arc::new(AtomicBool::new(false))),Reason::Cancelled=>g.cancel=Some(Arc::new(AtomicBool::new(true))),Reason::Expired=>g.deadline=Instant::now(),_=>unreachable!()};let mut out=vec![];assert_eq!(frame(&mut out,b"grant\n",&g).unwrap(),Outcome::Skipped(reason));assert!(out.is_empty());}}
    #[test]fn policy_withdrawal_control_is_allowed_but_an_expired_decision_is_not(){let(_e,t)=native();t.invalidate_policy();let mut g=guard(&t);let mut out=vec![];assert_eq!(frame(&mut out,b"grant\n",&g).unwrap(),Outcome::Skipped(Reason::SourceEnded));g.source=Some((t.clone(),Use::Withdrawal));assert_eq!(frame(&mut out,b"withdraw\n",&g).unwrap(),Outcome::Written);assert_eq!(out,b"withdraw\n");}
    #[test]fn a_native_cancel_request_only_allows_its_pending_control_and_never_a_positive_or_resolved_reply(){let(_e,t)=native();let mut g=guard(&t);t.request_withdrawal();let mut out=vec![];assert_eq!(frame(&mut out,b"positive\n",&g).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert!(out.is_empty());g.source=Some((t.clone(),Use::Withdrawal));assert_eq!(frame(&mut out,b"cancelled\n",&g).unwrap(),Outcome::Written);t.cancel();assert_eq!(frame(&mut out,b"again\n",&g).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert_eq!(out,b"cancelled\n");}
    #[test]fn resolved_or_closed_sources_never_receive_a_policy_withdrawal_replay(){for close in[true,false]{let(e,t)=native();t.invalidate_policy();if close{e.close();}else{t.cancel();}let mut g=guard(&t);g.source=Some((t,Use::Withdrawal));let mut out=vec![];assert_eq!(frame(&mut out,b"withdraw\n",&g).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert!(out.is_empty());}}
    #[test]fn a_complete_frame_keeps_its_written_result_if_the_source_resolves_on_the_last_byte(){let(_e,t)=native();let old=t.clone();struct Complete{out:Vec<u8>,ticket:Ticket}impl Write for Complete{fn write(&mut self,b:&[u8])->io::Result<usize>{self.out.extend_from_slice(b);self.ticket.cancel();Ok(b.len())}fn flush(&mut self)->io::Result<()>{Ok(())}}let mut out=Complete{out:vec![],ticket:old};assert_eq!(frame(&mut out,b"native\n",&guard(&t)).unwrap(),Outcome::Written);assert_eq!(out.out,b"native\n");}
}
#[cfg(all(test,target_os="linux"))]
#[path="pet_native_write_settings_tests.rs"]mod settings_tests;
