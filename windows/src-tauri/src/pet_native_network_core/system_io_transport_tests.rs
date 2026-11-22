use super::*;
use crate::settings::EffectFixture;
use std::{pin::Pin,task::{Context,Poll},io};
struct Writer<'a>{out:Vec<u8>,first:Option<Box<dyn FnOnce()+'a>>,partial:bool,complete:bool,calls:usize}
impl AsyncWrite for Writer<'_>{
 fn poll_write(mut self:Pin<&mut Self>,cx:&mut Context<'_>,bytes:&[u8])->Poll<io::Result<usize>>{
  self.calls+=1;if let Some(first)=self.first.take(){if self.complete{self.out.extend_from_slice(bytes);first();return Poll::Ready(Ok(bytes.len()));}if self.partial{self.out.extend_from_slice(&bytes[..2]);first();return Poll::Ready(Ok(2));}first();cx.waker().wake_by_ref();return Poll::Pending;}self.out.extend_from_slice(bytes);Poll::Ready(Ok(bytes.len()))
 }
 fn poll_flush(self:Pin<&mut Self>,_:&mut Context<'_>)->Poll<io::Result<()>>{Poll::Ready(Ok(()))}
 fn poll_shutdown(self:Pin<&mut Self>,_:&mut Context<'_>)->Poll<io::Result<()>>{Poll::Ready(Ok(()))}
}
#[tokio::test]async fn system_io_before_first_network_attempt_is_not_started(){
 let io=EffectFixture::new();io.external();let mut w=Writer{out:vec![],first:None,partial:false,complete:false,calls:0};let mut state=WriteState::NotStarted;assert_eq!(write_frame_checked(&mut w,b"effect\n",&mut state,||io.probe.check().is_ok()).await,Err(Error::PreferencesUnconfirmed));assert_eq!(state,WriteState::NotStarted);assert_eq!(w.calls,0);assert!(w.out.is_empty());
}
#[tokio::test]async fn system_io_pending_or_partial_network_attempt_remains_uncertain_without_completing_frame(){
 for partial in [false,true]{let io=EffectFixture::new();let mut w=Writer{out:vec![],first:Some(Box::new(||io.external())),partial,complete:false,calls:0};let mut state=WriteState::NotStarted;assert_eq!(write_frame_checked(&mut w,b"effect\n",&mut state,||io.probe.check().is_ok()).await,Err(Error::PreferencesUnconfirmed));assert_eq!(state,WriteState::MayHaveStarted);assert_eq!(w.calls,1);assert_eq!(w.out,if partial{b"ef".to_vec()}else{vec![]});}
}
#[tokio::test]async fn system_io_complete_native_frame_is_not_reinterpreted_after_last_byte(){
 let io=EffectFixture::new();let mut w=Writer{out:vec![],first:Some(Box::new(||io.external())),partial:false,complete:true,calls:0};let mut state=WriteState::NotStarted;write_frame_checked(&mut w,b"effect\n",&mut state,||io.probe.check().is_ok()).await.unwrap();assert_eq!(w.out,b"effect\n");assert!(io.probe.check().is_err());
}
#[tokio::test]async fn system_io_benign_current_commit_during_pending_write_preserves_original_frame(){
 let io=EffectFixture::new();let mut w=Writer{out:vec![],first:Some(Box::new(||io.save())),partial:false,complete:false,calls:0};let mut state=WriteState::NotStarted;write_frame_checked(&mut w,b"original-target\n",&mut state,||io.probe.check().is_ok()).await.unwrap();assert_eq!(w.out,b"original-target\n");assert_eq!(w.calls,2);
}
