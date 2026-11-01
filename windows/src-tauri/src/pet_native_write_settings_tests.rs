use super::*;
use crate::settings::EffectFixture;
fn guard(f:&EffectFixture)->Guard{Guard{dispatch:Dispatch::ProbedEffect(f.probe.clone()),deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:None,source:None}}
struct Writer<'a>{bytes:Vec<u8>,first:Option<Box<dyn FnOnce()+'a>>,partial:bool,complete:bool}
impl Write for Writer<'_>{
 fn write(&mut self,b:&[u8])->io::Result<usize>{
  if let Some(first)=self.first.take(){if self.complete{self.bytes.extend_from_slice(b);first();return Ok(b.len());}if self.partial{self.bytes.extend_from_slice(&b[..2]);first();return Ok(2);}first();return Err(io::ErrorKind::WouldBlock.into());}
  self.bytes.extend_from_slice(b);Ok(b.len())
 }
 fn flush(&mut self)->io::Result<()>{Ok(())}
}
#[test]fn final_dispatch_changed_io_skips_first_byte_without_changing_callback_lifetime(){
 let f=EffectFixture::new();let g=guard(&f);f.external();assert_eq!(g.reason(),None);assert_eq!(g.write_reason(),Some(Reason::PreferencesUnconfirmed));let mut out=Vec::new();assert_eq!(frame(&mut out,b"grant\n",&g).unwrap(),Outcome::Skipped(Reason::PreferencesUnconfirmed));assert!(out.is_empty());
}
#[test]fn final_dispatch_backpressure_observes_current_io_before_the_next_write(){
 let f=EffectFixture::new();let mut w=Writer{bytes:vec![],first:Some(Box::new(||f.external())),partial:false,complete:false};assert_eq!(frame(&mut w,b"grant\n",&guard(&f)).unwrap(),Outcome::Skipped(Reason::PreferencesUnconfirmed));assert!(w.bytes.is_empty());
}
#[test]fn final_dispatch_partial_frame_never_receives_the_remaining_positive_bytes(){
 let f=EffectFixture::new();let mut w=Writer{bytes:vec![],first:Some(Box::new(||f.external())),partial:true,complete:false};let error=frame(&mut w,b"grant\n",&guard(&f)).unwrap_err();assert!(error.partial);assert_eq!(error.reason,Reason::PreferencesUnconfirmed);assert_eq!(w.bytes,b"gr");
}
#[test]fn final_dispatch_complete_frame_retains_written_receipt_when_io_changes_on_last_byte(){
 let f=EffectFixture::new();let mut w=Writer{bytes:vec![],first:Some(Box::new(||f.external())),partial:false,complete:true};assert_eq!(frame(&mut w,b"grant\n",&guard(&f)).unwrap(),Outcome::Written);assert_eq!(w.bytes,b"grant\n");assert!(f.probe.check().is_err());
}
#[test]fn final_dispatch_benign_commit_during_backpressure_keeps_current_destination(){
 let f=EffectFixture::new();let mut w=Writer{bytes:vec![],first:Some(Box::new(||f.save())),partial:false,complete:false};assert_eq!(frame(&mut w,b"captured-destination\n",&guard(&f)).unwrap(),Outcome::Written);assert_eq!(w.bytes,b"captured-destination\n");
}
#[test]fn final_dispatch_writing_blocks_effect_but_not_exact_control_or_stop(){
 let f=EffectFixture::new();let held=f.current.lock().unwrap();let mut g=guard(&f);let mut out=vec![];assert_eq!(frame(&mut out,b"grant\n",&g).unwrap(),Outcome::Skipped(Reason::PreferencesUnconfirmed));g.dispatch=Dispatch::Control;assert_eq!(frame(&mut out,b"stop\n",&g).unwrap(),Outcome::Written);assert_eq!(out,b"stop\n");drop(held);
}
#[test]fn final_dispatch_control_cannot_resurrect_a_resolved_native_request(){
 let f=EffectFixture::new();let(t,_)=Ticket::fixture();let mut g=guard(&f);g.dispatch=Dispatch::Control;g.source=Some((t.clone(),Use::Refusal));f.external();t.cancel();let mut out=vec![];assert_eq!(frame(&mut out,b"deny\n",&g).unwrap(),Outcome::Skipped(Reason::SourceEnded));assert!(out.is_empty());
}
