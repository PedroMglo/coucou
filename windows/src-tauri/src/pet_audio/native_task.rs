//! Existing Pulse-derived resource key -> retained native PipeWire object.
//! No key/grant migration, public model tool, scheduler or execution ledger.
use super::{pipewire_reader::{Reader,Output,NativeAction,Dispatch},task::Action,Snapshot,Kind};
use crate::pet_helper::ExchangeError;
use std::{sync::Mutex,time::Instant};
fn pulse_cookie(value:&str)->Result<u32,ExchangeError>{
 if value.len()!=9||value.as_bytes()[4]!=b':'||!value.bytes().enumerate().all(|(n,b)|n==4||b.is_ascii_hexdigit()){return Err(ExchangeError::NotStarted);}
 u32::from_str_radix(&value.replace(':',""),16).map_err(|_|ExchangeError::NotStarted)
}
fn inventory(reader:&Reader,current:&dyn Fn()->bool)->Result<(u32,Snapshot),ExchangeError>{
 let(cookie,snapshot)=super::linux::task_inventory_with(&|args|reader.pulse_command(args),current).map_err(|_|ExchangeError::NotStarted)?;
 let cookie=pulse_cookie(&cookie)?;if cookie!=reader.core_cookie(){return Err(ExchangeError::NotStarted);}Ok((cookie,snapshot))
}
fn selected(snapshot:&Snapshot,key:&str)->Result<(u32,String),ExchangeError>{
 let mut matches=snapshot.outputs.iter().filter(|n|n.kind==Kind::Output&&n.key==key&&n.controllable);
 let node=matches.next().ok_or(ExchangeError::NotStarted)?;if matches.next().is_some(){return Err(ExchangeError::NotStarted);}
 let id=node.native_id.filter(|id|*id>0).ok_or(ExchangeError::NotStarted)?;let serial=node.native_serial.clone().filter(|s|!s.is_empty()&&s.len()<=24&&s.bytes().all(|b|b.is_ascii_digit())).ok_or(ExchangeError::NotStarted)?;Ok((id,serial))
}
pub(crate)struct RetainedOutput{key:String,core_cookie:u32,id:u32,serial:String,anchor:Mutex<Reader>}
impl RetainedOutput{
 pub(crate)fn incarnation(&self)->(u32,u32,String){(self.core_cookie,self.id,self.serial.clone())}
 pub(crate)fn capture(mut reader:Reader,key:&str,current:&dyn Fn()->bool)->Result<Self,ExchangeError>{
  if !crate::pet_config::system_access::output_key(key)||!current(){return Err(ExchangeError::NotStarted);}
  let(cookie,before)=inventory(&reader,current)?;let(id,serial)=selected(&before,key)?;
  let mut matches=reader.outputs_fenced(current)?.into_iter().filter(|n|n.id==id&&n.serial==serial);let node=matches.next().ok_or(ExchangeError::NotStarted)?;if matches.next().is_some(){return Err(ExchangeError::NotStarted);}
  reader.capture_fenced(&node,current)?;
  let(after_cookie,after)=inventory(&reader,current)?;if after_cookie!=cookie||selected(&after,key)?!=(id,serial.clone()){return Err(ExchangeError::NotStarted);}
  let now=reader.read_fenced(current)?;if now.id!=id||now.serial!=serial{return Err(ExchangeError::NotStarted);}
  Ok(Self{key:key.into(),core_cookie:cookie,id,serial,anchor:Mutex::new(reader)})
 }
 fn same(&self,reader:&Reader,output:&Output)->bool{reader.core_cookie()==self.core_cookie&&output.id==self.id&&output.serial==self.serial}
 pub(crate)fn read(&self,current:&dyn Fn()->bool)->Result<Output,ExchangeError>{
  let mut reader=self.anchor.try_lock().map_err(|_|ExchangeError::NotStarted)?;let output=reader.read_fenced(current)?;
  if !self.same(&reader,&output){return Err(ExchangeError::NotStarted);}Ok(output)
 }
 pub(crate)fn dispatch(&self,action:&Action,deadline:Instant,current:&dyn Fn()->bool,before_effect:impl FnOnce()->Result<(),()>)->Dispatch{
  let refused=||Dispatch{before:None,after:None,possible_effect:false,transport_accepted:false,scope_current:false,verified:false};
  if action.key()!=self.key||Instant::now()>=deadline||!current(){return refused();}
  let Ok(mut anchor)=self.anchor.try_lock()else{return refused()};
  let Ok(before)=anchor.read_fenced(current)else{return refused()};if !self.same(&anchor,&before){return refused();}
  let Ok(writer)=anchor.writer(deadline,current)else{return refused()};let native=match action{Action::Volume{percent,..}=>NativeAction::Volume{percent:*percent},Action::Mute{muted,..}=>NativeAction::Mute{muted:*muted}};
  let mut result=writer.dispatch(native.clone(),deadline,current,before_effect);
  if result.possible_effect{super::linux::invalidate_output_cache();}
  // The retained anchor never wrote a setter. Its final observation catches
  // another native client's changes after the temporary writer was reaped.
  let after=anchor.read_fenced(current).ok().filter(|v|self.same(&anchor,v));
  result.after=after;result.scope_current &= Instant::now()<deadline&&current()&&result.after.is_some();
  result.verified &= result.scope_current&&result.after.as_ref().is_some_and(|v|native.satisfied(v));result
 }
 #[cfg(test)]pub(crate)fn inventory_fixture(reader:&Reader)->Result<Snapshot,ExchangeError>{inventory(reader,&||true).map(|(_,s)|s)}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn pulse_cookie_normalization_only_compares_the_existing_exact_identity(){
  assert_eq!(pulse_cookie("0000:0000"),Ok(0));assert_eq!(pulse_cookie("FFFF:ffff"),Ok(u32::MAX));assert_eq!(pulse_cookie("0123:abcd"),Ok(0x0123abcd));
  for bad in ["123:abcd","0123-abcd","0123:abcg"," 0123:abcd","0123:abcd ","19088743","unix:else"]{assert!(pulse_cookie(bad).is_err());}
 }
 #[test]#[ignore="Read-only identity bridge against the user's default local Pulse/PipeWire services; no effects"]
 fn native_user_output_identity_bridge_observed_readonly(){
  use std::{sync::Arc,time::Duration};
  let reader=Reader::start_anchor(Instant::now()+Duration::from_secs(15),Arc::new(||true),&||true).unwrap();
  let(_,snapshot)=inventory(&reader,&||true).unwrap();assert!(!snapshot.outputs.is_empty());let count=snapshot.outputs.len();let key=snapshot.outputs[0].key.clone();
  let target=RetainedOutput::capture(reader,&key,&||true).unwrap();let first=target.read(&||true).unwrap();let second=target.read(&||true).unwrap();assert_eq!(first.id,second.id);assert_eq!(first.serial,second.serial);
  println!("NATIVE_HOST_BRIDGE outputs={} sameCoreKeyIdSerial=true retainedReads=2 readOnly=true userAudioWrites=0",count);
 }
}
