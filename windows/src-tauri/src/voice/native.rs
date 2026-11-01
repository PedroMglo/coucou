//! Backend-owned speech content and one-use native admission. No permissions
//! are minted by an ID, window label or renderer-supplied text.
use std::{sync::{Arc,atomic::{AtomicBool,AtomicU64,Ordering}},time::Instant};
use super::tts::Current;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum Surface {Pill,Companions}
impl Surface {
    pub(super) fn label(self)->&'static str{match self{Self::Pill=>crate::flow::pill::LABEL,Self::Companions=>crate::companions::LABEL}}
    pub(super) fn from_label(label:&str)->Option<Self>{[Self::Pill,Self::Companions].into_iter().find(|s|s.label()==label)}
    pub(super) fn page(self)->&'static str{match self{Self::Pill=>"/pill.html",Self::Companions=>"/companions.html"}}
}
pub(super) struct Owner {surface:Surface,epoch:AtomicU64,ready:AtomicBool,closed:AtomicBool}
impl Owner {
    pub(super) fn new(surface:Surface)->Arc<Self>{Arc::new(Self{surface,epoch:AtomicU64::new(1),ready:AtomicBool::new(false),closed:AtomicBool::new(false)})}
    pub(super) fn invalidate(&self){self.ready.store(false,Ordering::Release);let _=self.epoch.fetch_update(Ordering::AcqRel,Ordering::Acquire,|v|Some(v.saturating_add(1)));}
    pub(super) fn generation(&self)->u64{self.epoch.load(Ordering::Acquire)}
    pub(super) fn close(&self){self.closed.store(true,Ordering::Release);self.invalidate();}
    pub(super) fn ready(&self,value:bool){if !self.closed.load(Ordering::Acquire){self.ready.store(value,Ordering::Release);}}
    pub(super) fn frame(self:&Arc<Self>)->Option<Frame>{let f=Frame{owner:self.clone(),epoch:self.epoch.load(Ordering::Acquire)};f.current().then_some(f)}
}
#[derive(Clone)]pub(super) struct Frame{owner:Arc<Owner>,epoch:u64}
impl Frame {
    pub(super) fn current(&self)->bool{self.epoch!=u64::MAX&&!self.owner.closed.load(Ordering::Acquire)&&self.owner.ready.load(Ordering::Acquire)&&self.owner.epoch.load(Ordering::Acquire)==self.epoch}
    pub(super) fn same(&self,other:&Self)->bool{Arc::ptr_eq(&self.owner,&other.owner)&&self.epoch==other.epoch&&self.current()&&other.current()}
    pub(super) fn surface(&self)->Surface{self.owner.surface}
}

pub(super) struct Speech {
    frame:Frame,text:String,language:Option<String>,deadline:Instant,current:Current,
    live:AtomicBool,claimed:AtomicBool,diagnostics:AtomicU64,
}
impl Speech {
    pub(super) fn prepare(frame:Frame,text:&str,language:Option<&str>,deadline:Instant,current:Current)->Option<Arc<Self>>{
        if text.trim().is_empty()||text.chars().count()>8000||text.contains('\0')||language.is_some_and(|l|l.is_empty()||!l.as_bytes()[0].is_ascii_alphabetic()||l.len()>35||!l.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')){return None;}
        let s=Arc::new(Self{frame,text:text.into(),language:language.map(str::to_owned),deadline,current,live:AtomicBool::new(true),claimed:AtomicBool::new(false),diagnostics:AtomicU64::new(0)});
        s.current().then_some(s)
    }
    pub(super) fn event(&self,code:usize){
        const EVENTS:[&str;8]=["caller unavailable","helper unavailable","claim refused","native helper accepted","native helper unconfirmed","deadline expired","document retired","configured context retired"];
        if let Some(message)=EVENTS.get(code){let bit=1u64<<code;if self.diagnostics.fetch_or(bit,Ordering::AcqRel)&bit==0{crate::log::line(format!("voice native: {message}"));}}
    }
    pub(super) fn current(&self)->bool{
        if !self.live.load(Ordering::Acquire){return false;}
        let valid=if Instant::now()>=self.deadline{self.event(5);false}else if !self.frame.current(){self.event(6);false}else if !(self.current)(){self.event(7);false}else{true};
        if !valid{self.retire();}valid
    }
    pub(super) fn retire(&self){self.live.store(false,Ordering::Release);}
    pub(super) fn belongs_to(&self,frame:&Frame)->bool{self.frame.same(frame)}
    pub(super) fn claim(self:&Arc<Self>,frame:&Frame)->Option<Invocation>{
        if !self.belongs_to(frame)||!self.current(){return None;}
        self.claimed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).ok()?;
        let invocation=Invocation(self.clone());invocation.current().then_some(invocation)
    }
}
pub(super) struct Invocation(Arc<Speech>);
impl Invocation {
    pub(super) fn current(&self)->bool{self.0.current()}
    pub(super) fn text(&self)->&str{&self.0.text}
    pub(super) fn language(&self)->Option<&str>{self.0.language.as_deref()}
    pub(super) fn guard(&self)->Current{let speech=self.0.clone();Arc::new(move||speech.current())}
    pub(super) fn remaining(&self)->std::time::Duration{self.0.deadline.saturating_duration_since(Instant::now())}
}

#[cfg(test)]mod tests{
    use super::*;use std::time::Duration;
    fn frame(surface:Surface)->(Arc<Owner>,Frame){let o=Owner::new(surface);o.ready(true);let f=o.frame().unwrap();(o,f)}
    fn speech(frame:Frame)->Arc<Speech>{Speech::prepare(frame,"Backend content",Some("pt-PT"),Instant::now()+Duration::from_secs(2),Arc::new(||true)).unwrap()}
    #[test]fn native_speech_exact_source_and_content_are_consumed_once(){let(_,f)=frame(Surface::Pill);let s=speech(f.clone());let call=s.claim(&f).unwrap();assert_eq!(call.text(),"Backend content");assert_eq!(call.language(),Some("pt-PT"));assert!(s.claim(&f).is_none());assert!(call.current());s.retire();assert!(!call.current());}
    #[test]fn another_surface_or_replacement_with_the_same_label_cannot_claim_old_content(){let(_,f)=frame(Surface::Pill);let s=speech(f.clone());for surface in [Surface::Pill,Surface::Companions]{let(_,other)=frame(surface);assert!(s.claim(&other).is_none());}assert!(s.claim(&f).is_some());}
    #[test]fn reload_crash_close_or_late_readiness_never_renew_a_frame(){for close in [false,true]{let(o,f)=frame(Surface::Pill);let s=speech(f.clone());if close{o.close();}else{o.invalidate();}o.ready(true);assert!(!f.current());assert!(s.claim(&f).is_none());assert!(o.frame().is_some()!=close);}}
    #[test]fn queue_wait_rechecks_the_original_authority_without_replaying(){let(_,f)=frame(Surface::Pill);let alive=Arc::new(AtomicBool::new(true));let a=alive.clone();let s=Speech::prepare(f.clone(),"Original",None,Instant::now()+Duration::from_secs(2),Arc::new(move||a.load(Ordering::Acquire))).unwrap();let call=s.claim(&f).unwrap();alive.store(false,Ordering::Release);assert!(!call.current());alive.store(true,Ordering::Release);assert!(!call.current());s.retire();assert!(!call.current());assert!(s.claim(&f).is_none());}
    #[test]fn stale_or_malformed_speech_never_dispatches(){let(_,f)=frame(Surface::Pill);for text in ["", "\0"]{assert!(Speech::prepare(f.clone(),text,None,Instant::now()+Duration::from_secs(2),Arc::new(||true)).is_none());}for lang in ["-C\n", "-C", "--", "../pt", ""]{assert!(Speech::prepare(f.clone(),"text",Some(lang),Instant::now()+Duration::from_secs(2),Arc::new(||true)).is_none());}assert!(Speech::prepare(f,"text",None,Instant::now(),Arc::new(||true)).is_none());}
}
