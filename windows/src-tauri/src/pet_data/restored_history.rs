//! Immutable source references for Admission2. This index DENIES old control;
//! it cannot authenticate a sender, approve a model request or grant a lease.
use super::{Error,Result};
use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
const MAX_REFERENCES:usize=8192;
const MAX_LINEAGE:usize=8;
pub(crate)const MAX_INDEX:usize=8*1024*1024;
pub(crate)const NOTICE:&str="Este histórico foi restaurado e está disponível para consulta. Inicia uma nova conversa para trabalhar; retomar a sessão anterior exige uma revalidação explícita.";
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Reference{provider:String,id:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct History{schema_version:u32,source_hash:String,conversations:Vec<Reference>,native_sessions:Vec<Reference>,tasks:Vec<Reference>,#[serde(default,skip_serializing_if="Vec::is_empty")]lineage:Vec<HistorySource>}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct HistorySource{history_hash:String,source_hash:String}
fn hash(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn provider(id:&str)->bool{!id.is_empty()&&id.len()<=64&&id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))}
fn native_id(id:&str)->bool{!id.trim().is_empty()&&id.len()<=512&&!id.chars().any(char::is_control)}
fn add(rows:&mut BTreeSet<Reference>,p:&str,id:&str,native:bool)->Result<()>{
 if !provider(p)||!(if native{native_id(id)}else{crate::assistant_memory::valid_id(id)}){return Err(Error::UnsupportedProfile);}rows.insert(Reference{provider:p.into(),id:id.into()});if rows.len()>MAX_REFERENCES{return Err(Error::Limit);}Ok(())
}
impl History{
 /// Validate raw identities before the real codec can normalize legacy IDs.
 /// Capture only the bytes supplied by the native reviewed data publisher.
 pub(crate)fn capture(bytes:Option<&[u8]>)->Result<Self>{
  let mut conversations=BTreeSet::new();let mut native_sessions=BTreeSet::new();let mut tasks_index=BTreeSet::new();
  if let Some(bytes)=bytes{
   if bytes.len()as u64>crate::assistant_memory::MAX_BYTES{return Err(Error::Limit);}
   let raw:serde_json::Value=serde_json::from_slice(bytes).map_err(|_|Error::UnsupportedProfile)?;
   let current=raw["pets"].as_object().ok_or(Error::UnsupportedProfile)?;
   let archives=raw.get("archives").map(|v|v.as_object().ok_or(Error::UnsupportedProfile)).transpose()?;
   let mut inspect=|p:&str,c:&serde_json::Value|->Result<()>{
    let id=c["id"].as_str().ok_or(Error::UnsupportedProfile)?;add(&mut conversations,p,id,false)?;
    if let Some(s)=c.get("session").filter(|s|!s.is_null()){add(&mut native_sessions,p,s["id"].as_str().ok_or(Error::UnsupportedProfile)?,true)?;}
    if let Some(tasks)=c.get("tasks"){
     for task in tasks.as_array().ok_or(Error::UnsupportedProfile)?{
      add(&mut tasks_index,p,task["id"].as_str().ok_or(Error::UnsupportedProfile)?,false)?;
      if let Some(cid)=task.get("conversationId").filter(|v|!v.is_null()){add(&mut conversations,p,cid.as_str().ok_or(Error::UnsupportedProfile)?,false)?;}
      if let Some(sid)=task.get("sessionId").filter(|v|!v.is_null()){add(&mut native_sessions,p,sid.as_str().ok_or(Error::UnsupportedProfile)?,true)?;}
     }
    }
    for key in ["native_memory_context","nativeMemoryContext"]{if let Some(ctx)=c.get(key).filter(|v|!v.is_null()){if let Some(sid)=ctx["destination"].get("sessionId").filter(|v|!v.is_null()){add(&mut native_sessions,p,sid.as_str().ok_or(Error::UnsupportedProfile)?,true)?;}}}
    Ok(())
   };
   for(p,c)in current{inspect(p,c)?;}
   if let Some(archives)=archives{for(p,rows)in archives{for c in rows.as_array().ok_or(Error::UnsupportedProfile)?{inspect(p,c)?;}}}
   // Same codec as Chat::persistent; no alternative model of usable history.
   crate::assistant_memory::inspect_history_bytes(bytes).map_err(|_|Error::UnsupportedProfile)?;
  }
  let result=Self{schema_version:1,source_hash:bytes.map(hash).unwrap_or_else(||hash(b"coucou-no-public-library-v1")),conversations:conversations.into_iter().collect(),native_sessions:native_sessions.into_iter().collect(),tasks:tasks_index.into_iter().collect(),lineage:vec![]};result.validate()?;Ok(result)
 }
 /// The caller obtains the previous bytes/hash from a verified sealed source
 /// admission. Sharing IDs does not grant control or any use of chat content.
 pub(crate)fn capture_next(bytes:Option<&[u8]>,previous:&[u8],expected:&str)->Result<Self>{
  let prior=Self::decode(previous,expected)?;let mut current=Self::capture(bytes)?;
  for (now,old) in [(&mut current.conversations,prior.conversations),(&mut current.native_sessions,prior.native_sessions),(&mut current.tasks,prior.tasks)]{
   let mut union=std::mem::take(now).into_iter().collect::<BTreeSet<_>>();union.extend(old);if union.len()>MAX_REFERENCES{return Err(Error::Limit);}*now=union.into_iter().collect();
  }
  current.schema_version=2;current.lineage=prior.lineage;current.lineage.push(HistorySource{history_hash:expected.into(),source_hash:prior.source_hash});current.lineage.sort();
  if current.lineage.len()>MAX_LINEAGE{return Err(Error::Limit);}current.validate()?;Ok(current)
 }
 fn validate(&self)->Result<()>{
  let digest=|v:&str|v.len()==64&&v.bytes().all(|b|b.is_ascii_hexdigit());
  if !matches!(self.schema_version,1|2)||!digest(&self.source_hash)||self.conversations.len()>MAX_REFERENCES||self.native_sessions.len()>MAX_REFERENCES||self.tasks.len()>MAX_REFERENCES{return Err(Error::UnsupportedProfile);}
  if self.schema_version==1{if !self.lineage.is_empty(){return Err(Error::UnsupportedProfile);}}else if self.lineage.is_empty()||self.lineage.len()>MAX_LINEAGE||self.lineage.iter().any(|r|!digest(&r.history_hash)||!digest(&r.source_hash))||self.lineage.windows(2).any(|w|w[0].history_hash>=w[1].history_hash){return Err(Error::UnsupportedProfile);}
  for(rows,native)in[(&self.conversations,false),(&self.native_sessions,true),(&self.tasks,false)]{
   let mut previous=None;for row in rows{if !provider(&row.provider)||!(if native{native_id(&row.id)}else{crate::assistant_memory::valid_id(&row.id)})||previous.is_some_and(|p:&Reference|p>=row){return Err(Error::UnsupportedProfile);}previous=Some(row);}
  }Ok(())
 }
 pub(crate)fn encode(&self)->Result<Vec<u8>>{self.validate()?;let b=serde_json::to_vec(self).map_err(|_|Error::UnsupportedProfile)?;if b.len()>MAX_INDEX{return Err(Error::Limit);}Ok(b)}
 pub(crate)fn decode(bytes:&[u8],expected:&str)->Result<Self>{if bytes.len()>MAX_INDEX||hash(bytes)!=expected{return Err(Error::SourceChanged);}let value:serde_json::Value=serde_json::from_slice(bytes).map_err(|_|Error::UnsupportedProfile)?;if value["schemaVersion"]==1&&value.as_object().is_some_and(|o|o.contains_key("lineage")){return Err(Error::UnsupportedProfile);}let h:Self=serde_json::from_value(value).map_err(|_|Error::UnsupportedProfile)?;h.validate()?;if h.lineage.iter().any(|r|r.history_hash==expected){return Err(Error::UnsupportedProfile);}Ok(h)}
 pub(crate)fn decode_first(bytes:&[u8],expected:&str)->Result<Self>{let h=Self::decode(bytes,expected)?;if h.schema_version!=1{return Err(Error::UnsupportedProfile);}Ok(h)}
 pub(crate)fn fingerprint(&self)->Result<String>{Ok(hash(&self.encode()?))}
 pub(crate)fn review(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"schemaVersion":self.schema_version,"indexHash":self.fingerprint()?,"sourceLibraryHash":self.source_hash,"references":self.counts(),"lineage":self.lineage,"contentCopied":false,"executionAuthorityGranted":false}))}
 pub(crate)fn historical_conversation(&self,p:&str,id:&str)->bool{self.conversations.binary_search(&Reference{provider:p.into(),id:id.into()}).is_ok()}
 pub(crate)fn historical_native_session(&self,p:&str,id:&str)->bool{self.native_sessions.binary_search(&Reference{provider:p.into(),id:id.into()}).is_ok()}
 pub(crate)fn conversation_allowed(&self,p:&str,id:&str,sid:Option<&str>)->Result<()>{if self.historical_conversation(p,id)||sid.is_some_and(|sid|self.historical_native_session(p,sid)){Err(Error::UnsupportedProfile)}else{Ok(())}}
 pub(crate)fn native_session_allowed(&self,p:&str,id:&str)->Result<()>{if self.historical_native_session(p,id){Err(Error::UnsupportedProfile)}else{Ok(())}}
 pub(crate)fn task_allowed(&self,p:&str,id:&str,cid:Option<&str>,sid:Option<&str>)->Result<()>{if self.tasks.binary_search(&Reference{provider:p.into(),id:id.into()}).is_ok()||cid.is_some_and(|id|self.historical_conversation(p,id))||sid.is_some_and(|id|self.historical_native_session(p,id)){Err(Error::UnsupportedProfile)}else{Ok(())}}
 pub(crate)fn counts(&self)->(usize,usize,usize){(self.conversations.len(),self.native_sessions.len(),self.tasks.len())}
}
#[cfg(test)]#[path="restored_history_tests.rs"]mod tests;
