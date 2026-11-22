//! Read-only observations on the same native portal actor and bus connection.
use super::*;
#[derive(Clone,Debug,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Subject{pub bus_id:String,pub peer:String,pub client:String,pub sessions:Vec<String>}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Observation{pub bus_matches:bool,pub original_client_live:Option<bool>,pub original_peer_live:Option<bool>,pub sessions_closed:bool,pub reason:&'static str}
impl Portal{
 pub(super)fn remember_close(&mut self,path:&str){if self.closed.iter().any(|s|s==path){return;}if self.closed.len()==64{self.closed.pop_front();}self.closed.push_back(path.into());}
 pub(super)fn close_ack(&mut self,path:&str)->bool{
  if !self.peer_alive{return false;}
  let ok=Message::new_method_call(self.peer.as_str(),path,SESSION,"Close").ok().is_some_and(|msg|self.call(msg).is_ok());
  if ok{self.remember_close(path);}ok
 }
 pub(super)fn observe_subject(&self,subject:Subject)->Observation{
  let mut out=Observation{bus_matches:self.bus_id==subject.bus_id,original_client_live:None,original_peer_live:None,sessions_closed:false,reason:"native owner not established"};
  if !out.bus_matches{out.reason="different session bus; original names cannot be reused";return out;}
  let escaped=subject.client.strip_prefix(':').unwrap_or("").replace('.',"_");
  if escaped.is_empty()||subject.sessions.len()>2||subject.sessions.iter().any(|s|s.len()>512||!s.starts_with(&format!("{OBJECT}/session/{escaped}/")))||!subject.peer.starts_with(':')||subject.peer.len()>128||!subject.client.starts_with(':')||subject.client.len()>128{out.reason="invalid retained native subject";return out;}
  let alive=|name:&str|->Option<bool>{let reply=self.call(Message::new_method_call(BUS,BUS_OBJECT,BUS,"NameHasOwner").ok()?.append1(name)).ok()?;if reply.sender().as_deref()!=Some(BUS){return None;}reply.read1().ok()};
  out.original_client_live=alive(&subject.client);out.original_peer_live=alive(&subject.peer);
  out.sessions_closed=if subject.sessions.is_empty(){true}else if out.original_client_live==Some(false){true}else{subject.client==self.sender&&subject.peer==self.peer&&subject.sessions.iter().all(|s|self.closed.iter().any(|known|known==s))};
  out.reason=if out.sessions_closed{if out.original_client_live==Some(false){"original client absent on the same bus; its sessions are closed by the protocol"}else{"exact owned session close ACKs observed"}}else if out.original_client_live==Some(true){"original client lives without complete close evidence"}else{"original native owner unavailable"};out
 }
}
