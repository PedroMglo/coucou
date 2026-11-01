//! Native-session data binding. Persisted references are a reuse fence, never a
//! request to replay raw data, create another session or grant agent permissions.
use super::{context::{Accepted,Capture},context_model::{Destination,NativeContext,MAX_SELECTED}};
use std::{path::Path,sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}}};
type Result<T>=std::result::Result<T,&'static str>;
pub(crate)struct Binding{
 provider:String,pet:String,cid:String,destination:Destination,previous:Option<NativeContext>,
 capture:Arc<Capture>,session:Mutex<Option<String>>,published:AtomicBool,claimed:AtomicBool,possible_at:u64,
}
pub(crate)fn destination(provider:&str,adapter:&str,model:&str,cwd:&Path,session:Option<&str>)->Result<Destination>{
 let destination=Destination::NativeSession{provider:provider.into(),adapter:adapter.into(),model:model.into(),workspace:cwd.to_str().ok_or("nativeWorkspaceUnavailable")?.into(),session_id:session.map(str::to_string)};
 if destination.valid(){Ok(destination)}else{Err("nativeMemoryDestinationInvalid")}
}
fn session(destination:&Destination)->Option<&str>{match destination{Destination::NativeSession{session_id,..}=>session_id.as_deref(),_=>None}}
impl Binding{
 pub(crate)fn current(destination:Destination,cid:&str,previous:Option<&NativeContext>,fresh:Option<&Accepted>,question:&str,now:u64)->Result<Self>{Self::at(&super::path(),destination,cid,previous,fresh,question,now)}
 pub(super)fn at(path:&Path,destination:Destination,cid:&str,previous:Option<&NativeContext>,fresh:Option<&Accepted>,question:&str,now:u64)->Result<Self>{
  if !destination.valid()||!destination.native()||!crate::assistant_memory::valid_id(cid){return Err("nativeMemoryDestinationInvalid");}
  let provider=destination.provider().to_string();let pet=crate::pet_config::integration_pet(&provider).ok_or("nativeMemoryOwnerInvalid")?.to_string();
  if let Some(old)=previous{
   if !old.valid_for(&provider,cid)||old.revoked{return Err("nativeMemoryContextRevoked");}
   if session(&old.destination).is_none(){return Err("nativeMemorySessionUnconfirmed");}
   if old.destination!=destination{return Err("nativeMemoryDestinationChanged");}
   if now<old.possibly_delivered_at{return Err("memoryClockChanged");}
  }
  let mut references=previous.map(|p|p.references.clone()).unwrap_or_default();
  if let Some(fresh)=fresh{
   fresh.verify_owner(&provider,cid,now)?;fresh.verify(&destination,question,None,now)?;
   for reference in fresh.references(){
    match references.iter().find(|r|r.id==reference.id){Some(old)if old!=reference=>return Err("memorySelectionChanged"),Some(_)=>{},None=>references.push(reference.clone())}
   }
  }
  if references.is_empty()||references.len()>MAX_SELECTED{return Err("nativeMemoryContextLimit");}
  let capture=Capture::at_references(path,&pet,cid,&references,now)?;
  Ok(Self{provider,pet,cid:cid.into(),session:Mutex::new(session(&destination).map(str::to_string)),destination,previous:previous.cloned(),capture:Arc::new(capture),published:AtomicBool::new(false),claimed:AtomicBool::new(false),possible_at:previous.map(|p|p.possibly_delivered_at).unwrap_or(now)})
 }
 pub(crate)fn destination(&self)->&Destination{&self.destination}
 pub(crate)fn claim_turn(&self)->Result<()>{self.claimed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).map(|_|()).map_err(|_|"nativeMemoryBindingConsumed")}
 pub(crate)fn matches_start(&self,provider:&str,cid:&str,previous:Option<&NativeContext>,session_id:Option<&str>)->bool{self.provider==provider&&self.cid==cid&&self.previous.as_ref()==previous&&session(&self.destination)==session_id}
 pub(crate)fn verify_owner(&self,provider:&str,cid:&str,now:u64)->Result<()>{if provider!=self.provider||cid!=self.cid||crate::pet_config::retired(&self.pet){return Err("nativeMemoryOwnerChanged");}self.capture.verify(now)}
 pub(crate)fn metadata(&self)->Result<NativeContext>{
  let mut destination=self.destination.clone();if let Destination::NativeSession{session_id,..}=&mut destination{*session_id=self.session.lock().map_err(|_|"nativeMemoryUnavailable")?.clone();}
  Ok(NativeContext{schema_version:1,pet_id:self.pet.clone(),provider:self.provider.clone(),conversation_id:self.cid.clone(),destination,references:self.capture.references().to_vec(),possibly_delivered_at:self.possible_at,revoked:false})
 }
 pub(crate)fn mark_published(&self){self.published.store(true,Ordering::Release);}
 pub(crate)fn before_prompt(&self,actual:&Destination,now:u64)->Result<()>{
  if actual!=&self.destination||!self.published.load(Ordering::Acquire){return Err("nativeMemoryNotPublished");}self.verify_owner(&self.provider,&self.cid,now)
 }
 pub(crate)fn adopted_metadata(&self,id:&str)->Result<NativeContext>{
  if !self.published.load(Ordering::Acquire)||id.trim().is_empty()||id.len()>512||id.chars().any(char::is_control){return Err("nativeMemorySessionInvalid");}
  if self.session.lock().map_err(|_|"nativeMemoryUnavailable")?.as_deref().is_some_and(|old|old!=id){return Err("nativeMemorySessionChanged");}
  let mut metadata=self.metadata()?;if let Destination::NativeSession{session_id,..}=&mut metadata.destination{*session_id=Some(id.into());}Ok(metadata)
 }
 pub(crate)fn mark_adopted(&self,id:&str)->Result<()>{*self.session.lock().map_err(|_|"nativeMemoryUnavailable")?=Some(id.into());Ok(())}
 pub(crate)fn previous(&self)->Option<&NativeContext>{self.previous.as_ref()}
}
#[cfg(test)]#[path="native_tests.rs"]mod tests;
