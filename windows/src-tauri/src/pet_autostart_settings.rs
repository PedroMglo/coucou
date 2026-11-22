//! Explicit native-user preference intent. This is not a goal permission or
//! a second authority ledger. The existing settings writer remains canonical.
use crate::settings::Settings;
use serde::{Serialize,Deserialize,Deserializer,de::{self,MapAccess,Visitor}};
use std::{fmt,io,sync::{Mutex,OnceLock}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)struct Patch{pub enabled:bool}
impl<'de>Deserialize<'de>for Patch{
 fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error>{struct V;impl<'de>Visitor<'de>for V{type Value=Patch;fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("one enabled boolean")}
  fn visit_map<M:MapAccess<'de>>(self,mut m:M)->Result<Patch,M::Error>{let mut enabled=None;while let Some(k)=m.next_key::<String>()?{if k!="enabled"{return Err(de::Error::unknown_field(&k,&["enabled"]));}if enabled.is_some(){return Err(de::Error::duplicate_field("enabled"));}enabled=Some(m.next_value::<bool>()?);}Ok(Patch{enabled:enabled.ok_or_else(||de::Error::missing_field("enabled"))?})}
 }d.deserialize_map(V)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Change{Changed,Unchanged}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Registration{ExecutorAccepted,Unconfirmed,NotAttempted}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{pub schema_version:u32,pub enabled:bool,pub change:Change,pub registration:Registration,#[serde(skip_serializing_if="Option::is_none")]pub reason:Option<String>}

#[cfg(target_os="linux")]#[path="autostart_review.rs"]pub(crate)mod native;
#[cfg(not(target_os="linux"))]#[path="autostart_review_unsupported.rs"]pub(crate)mod native;
#[derive(Default)]pub(crate)struct Controller{gate:Mutex<Option<(bool,String)>>,#[cfg(target_os="linux")]review:Mutex<native::ReviewState>}
pub(crate)fn controller()->&'static Controller{static C:OnceLock<Controller>=OnceLock::new();C.get_or_init(Controller::default)}
impl Controller{
 /// The field gate survives until the worker actually returns. The global
 /// settings mutex is released before executor IO; unrelated writers can run.
 pub(crate)fn apply_with(&self,current:&Mutex<Settings>,patch:Patch,admit:impl FnOnce()->io::Result<()>,persist:impl FnOnce(&Settings)->io::Result<()>,publish:impl FnOnce(&Settings),execute:impl FnOnce(bool)->Result<(),String>)->io::Result<Receipt>{
  let mut gate=self.gate.try_lock().map_err(|e|match e{std::sync::TryLockError::WouldBlock=>io::Error::new(io::ErrorKind::WouldBlock,"autostart operation is still running"),std::sync::TryLockError::Poisoned(_)=>io::Error::other("O executor anterior terminou sem confirmação. O registo requer reconciliação antes de outra alteração.")})?;let changed=std::cell::Cell::new(false);
  let committed=crate::settings::update_checked_with(current,|next|{changed.set(next.autostart!=patch.enabled);if changed.get(){admit()?;next.autostart=patch.enabled;}Ok(())},|next|if changed.get(){persist(next)}else{Ok(())},|next|if changed.get(){publish(next)})?;
  if !changed.get(){let reason=gate.as_ref().filter(|(enabled,_)|*enabled==committed.autostart).map(|(_,reason)|reason.clone());return Ok(Receipt{schema_version:1,enabled:committed.autostart,change:Change::Unchanged,registration:if reason.is_some(){Registration::Unconfirmed}else{Registration::NotAttempted},reason});}
  let result=execute(committed.autostart);let reason=result.err().map(|e|format!("A preferência foi guardada, mas o registo de arranque automático não foi confirmado: {e}. Consulta o estado antes de repetir."));*gate=reason.as_ref().map(|reason|(committed.autostart,reason.clone()));Ok(Receipt{schema_version:1,enabled:committed.autostart,change:Change::Changed,registration:if reason.is_none(){Registration::ExecutorAccepted}else{Registration::Unconfirmed},reason})
 }
}
pub(crate)fn preserve_incoming(current:&Settings,incoming:&mut Settings){incoming.autostart=current.autostart;}

#[cfg(test)]#[path="pet_autostart_settings_tests.rs"]mod tests;
