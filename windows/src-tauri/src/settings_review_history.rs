//! Immutable retained review input/output, never a grant or an effect to replay.
use crate::private_registry::{self,Document,Snapshot};
use super::{ReconciliationRef,hex,hash};
use serde::{Serialize,Deserialize};
use std::{io,path::{Path,PathBuf}};
pub(crate)const DIRECTORY:&str="settings-io-review-v1";
const MAX_ENTRIES:usize=16;const MAX_TOTAL:usize=16*1024*1024;
fn error()->io::Error{io::Error::other("A retenção da revisão das preferências não foi confirmada. Os efeitos não foram autorizados.")}
fn encoded(bytes:&[u8])->String{bytes.iter().map(|v|format!("{v:02x}")).collect()}
fn valid_encoded(s:&str)->bool{s.len()<=512*1024&&s.len()%2==0&&hex(s,s.len())}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Artifact{
 version:u32,generation:u64,transaction:String,fingerprint:String,scope:String,
 original_settings:Option<String>,original_io:Option<String>,approved_settings:String,
 memory_hash:String,configuration_revision:String,configuration_values:serde_json::Value,credential_references:Vec<String>,observed_phase:String,
 all_values_reviewed:bool,uncertainty_understood:bool,components_remain_suspended:bool,
}
impl Document for Artifact{
 const MAX_BYTES:usize=2*1024*1024;const LOCK_NAME:&'static str=".review-archive-never-replace.lock";const TEMP_PREFIX:&'static str=".review-archive-never-replace-";
 fn empty()->Self{Self{version:0,generation:0,transaction:String::new(),fingerprint:String::new(),scope:String::new(),original_settings:None,original_io:None,approved_settings:String::new(),memory_hash:String::new(),configuration_revision:String::new(),configuration_values:serde_json::Value::Null,credential_references:vec![],observed_phase:String::new(),all_values_reviewed:false,uncertainty_understood:false,components_remain_suspended:false}}
 fn validate(&self)->Result<(),&'static str>{
  if self.version!=1||self.generation!=1||!hex(&self.transaction,32)||!hex(&self.fingerprint,64)||!hex(&self.memory_hash,64)||self.configuration_revision.len()>256||serde_json::to_vec(&self.configuration_values).map_or(true,|b|b.len()>256*1024)||self.credential_references.len()>64||self.credential_references.iter().any(|s|s.len()>256)||self.observed_phase.len()>64||self.scope!="dataOnlyKeepComponentsSuspended"||!self.all_values_reviewed||!self.uncertainty_understood||!self.components_remain_suspended||!valid_encoded(&self.approved_settings)||self.original_settings.as_ref().is_some_and(|s|!valid_encoded(s))||self.original_io.as_ref().is_some_and(|s|!valid_encoded(s)){return Err("invalidSettingsReviewArtifact");}Ok(())
 }
 fn generation(&self)->u64{self.generation}
 fn next(self,_:&Self)->Result<Self,&'static str>{Err("immutableSettingsReviewArtifact")}
}
pub(crate)struct Input<'a>{pub id:&'a str,pub fingerprint:&'a str,pub original_settings:Option<&'a[u8]>,pub original_io:Option<&'a[u8]>,pub approved:&'a[u8],pub memory_hash:&'a str,pub configuration_revision:&'a str,pub configuration_values:&'a serde_json::Value,pub observed_phase:&'a str}
fn path(settings:&Path,id:&str)->io::Result<PathBuf>{if !hex(id,32){return Err(error());}Ok(settings.parent().ok_or_else(error)?.join(DIRECTORY).join(format!("review-{id}.json")))}
pub(crate)fn retain(settings:&Path,input:Input<'_>,current:impl Fn()->Result<(),&'static str>)->io::Result<ReconciliationRef>{
 let value=Artifact{version:1,generation:1,transaction:input.id.into(),fingerprint:input.fingerprint.into(),scope:"dataOnlyKeepComponentsSuspended".into(),original_settings:input.original_settings.map(encoded),original_io:input.original_io.map(encoded),approved_settings:encoded(input.approved),memory_hash:input.memory_hash.into(),configuration_revision:input.configuration_revision.into(),configuration_values:input.configuration_values.clone(),credential_references:crate::secrets::KNOWN_KEYS.iter().map(|s|s.to_string()).collect(),observed_phase:input.observed_phase.into(),all_values_reviewed:true,uncertainty_understood:true,components_remain_suspended:true};
 value.validate().map_err(|_|error())?;let bytes=serde_json::to_vec(&value).map_err(|_|error())?;if bytes.len()>Artifact::MAX_BYTES{return Err(error());}
 let path=path(settings,input.id)?;private_registry::retain_new(&path,&bytes,MAX_ENTRIES,MAX_TOTAL,current).map_err(|_|error())?;
 let reference=ReconciliationRef{id:input.id.into(),sha256:hash(&bytes)};let held=observe(settings,&reference)?;if !held.unchanged(){return Err(error());}Ok(reference)
}
pub(crate)fn observe(settings:&Path,reference:&ReconciliationRef)->io::Result<Snapshot<Artifact>>{
 let held=private_registry::load_at::<Artifact>(&path(settings,&reference.id)?).map_err(|_|error())?;
 if held.revision!=reference.sha256||held.registry.transaction!=reference.id{return Err(error());}Ok(held)
}
