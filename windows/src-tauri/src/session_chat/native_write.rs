//! The WebSocket sink and reader share tungstenite's output buffer. Guard the
//! underlying Unix writes too, so a reader-side flush cannot send an old frame.
use super::*;
use std::{pin::Pin,task::{Context,Poll}};
use tokio::io::{AsyncRead,AsyncWrite,ReadBuf};
pub(super)struct Scope{pub gate:crate::pet_native_write::Guard,retired:AtomicBool}
impl Scope{
    pub(super)fn new(gate:crate::pet_native_write::Guard)->Arc<Self>{Arc::new(Self{gate,retired:AtomicBool::new(false)})}
    pub(super)fn expired(&self)->bool{self.retired.load(Ordering::Acquire)||self.gate.reason().is_some()}
    pub(super)fn retire(&self){self.retired.store(true,Ordering::Release);}
}
pub(super)type Slot=Arc<Mutex<Option<Arc<Scope>>>>;
pub(super)struct Socket{inner:UnixStream,slot:Slot}
impl Socket{pub(super)fn new(inner:UnixStream,slot:Slot)->Self{Self{inner,slot}}
    fn allowed(&self)->bool{self.slot.lock().is_ok_and(|s|s.as_ref().is_none_or(|s|!s.expired()))}
}
impl AsyncRead for Socket{fn poll_read(self:Pin<&mut Self>,cx:&mut Context<'_>,buf:&mut ReadBuf<'_>)->Poll<std::io::Result<()>>{Pin::new(&mut self.get_mut().inner).poll_read(cx,buf)}}
impl AsyncWrite for Socket{
    fn poll_write(self:Pin<&mut Self>,cx:&mut Context<'_>,bytes:&[u8])->Poll<std::io::Result<usize>>{
        if !self.allowed(){return Poll::Ready(Err(std::io::ErrorKind::PermissionDenied.into()));}
        Pin::new(&mut self.get_mut().inner).poll_write(cx,bytes)
    }
    fn poll_flush(self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<std::io::Result<()>>{Pin::new(&mut self.get_mut().inner).poll_flush(cx)}
    fn poll_shutdown(self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<std::io::Result<()>>{Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)}
}
pub(super)struct Caller(pub Arc<AtomicBool>);
impl Drop for Caller{fn drop(&mut self){self.0.store(false,Ordering::Release);}}
#[cfg(test)]mod tests{
    use super::*;use tokio::io::AsyncWriteExt;use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture,PolicyBinding}};
    fn scope()->(Endpoint,crate::pet_native_decisions::Ticket,Arc<Scope>){let e=Endpoint::new(Adapter::Codex,Transport::ControlSocket).unwrap();let t=e.capture(Capture{request_id:Some(&json!(7)),session_id:"s",turn_id:Some("t"),prompt_id:None,method:"native/request",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();let s=Scope::new(crate::pet_native_write::Guard{deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:None,source:Some((t.clone(),crate::pet_native_write::Use::Decision))});(e,t,s)}
    #[tokio::test]async fn every_unix_write_checks_the_original_native_scope(){let(_e,t,scope)=scope();let(client,peer)=UnixStream::pair().unwrap();let slot=Arc::new(Mutex::new(Some(scope)));let mut stream=Socket::new(client,slot);t.cancel();assert!(stream.write_all(b"grant\n").await.is_err());let mut bytes=[0;64];assert!(matches!(peer.try_read(&mut bytes),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));}
    #[tokio::test]async fn buffered_websocket_frames_cannot_be_flushed_after_native_resolution_even_by_close(){
        let(_e,t,scope)=scope();let(client,peer)=UnixStream::pair().unwrap();let slot=Arc::new(Mutex::new(Some(scope)));let stream=Socket::new(client,slot);let mut config=WebSocketConfig::default();config.write_buffer_size=1024*1024;let mut ws=tokio_tungstenite::WebSocketStream::from_raw_socket(stream,tokio_tungstenite::tungstenite::protocol::Role::Client,Some(config)).await;
        ws.feed(Message::Text("stale-native-decision".into())).await.unwrap();let mut bytes=[0;128];assert!(matches!(peer.try_read(&mut bytes),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));t.cancel();assert!(ws.flush().await.is_err());let _=ws.close(None).await;match peer.try_read(&mut bytes){Ok(0)=>{},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{},other=>panic!("a closed/retired WebSocket must not flush an old native frame: {other:?}")}
    }
    #[tokio::test]async fn retired_or_departed_callers_cannot_leave_a_flushable_buffer(){for retire in[false,true]{let(_e,_t,scope)=scope();let caller=Arc::new(AtomicBool::new(true));let s=Scope::new(crate::pet_native_write::Guard{deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:Some(caller.clone()),source:None});let(client,peer)=UnixStream::pair().unwrap();let mut stream=Socket::new(client,Arc::new(Mutex::new(Some(s.clone()))));if retire{s.retire();}else{drop(Caller(caller));}assert!(stream.write_all(b"old frame").await.is_err());let mut bytes=[0;64];assert!(matches!(peer.try_read(&mut bytes),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));drop(scope);}}
}
