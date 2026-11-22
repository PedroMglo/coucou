//! Drafts are opaque user data, never send intents, prompts or authority.
//! UTF-16 arrays retain even unpaired JS surrogates without lossy conversion.
use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
pub(super)const RECORDS:usize=1024;
pub(super)const TEXT:usize=65536;
pub(super)const TOTAL:usize=2097152;
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Target{pub pet_id:Vec<u16>,pub provider:Vec<u16>,pub origin:Origin,pub session_id:Vec<u16>,pub conversation_id:Option<Vec<u16>>}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Origin{Mochi,External,Unbound}
fn id(v:&[u16],max:usize)->bool{!v.is_empty()&&v.len()<=max&&!v.contains(&0)}
impl Target{pub(super)fn validate(&self)->Result<(),&'static str>{
 if (self.origin==Origin::Unbound&&(self.conversation_id.is_some()||self.session_id!="inline-composer".encode_utf16().collect::<Vec<_>>()))||!id(&self.pet_id,128)||!id(&self.provider,128)||!id(&self.session_id,512)||self.conversation_id.as_ref().is_some_and(|v|!id(v,512)){return Err("invalidDraft");}Ok(())
}}
pub(super)fn ordinal(v:&str)->Result<u64,&'static str>{
 if v.is_empty()||v.len()>20||!v.bytes().all(|b|b.is_ascii_digit())||(v.len()>1&&v.starts_with('0')){return Err("invalidDraft");}
 v.parse().map_err(|_|"invalidDraft")
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Record{pub target:Target,pub revision:String,pub text_units:Vec<u16>}
fn records_valid(records:&[Record],revision:u64)->Result<(),&'static str>{
 let mut targets=BTreeSet::new();let mut units=0usize;
 if records.len()>RECORDS{return Err("draftLimit");}
 for item in records{item.target.validate()?;let n=ordinal(&item.revision)?;if n==0||n>revision||!targets.insert(&item.target){return Err("invalidDraft");}if item.text_units.len()>TEXT{return Err("draftLimit");}units=units.checked_add(item.text_units.len()).ok_or("draftLimit")?;}
 if units>TOTAL{return Err("draftLimit");}Ok(())
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Import{pub schema_version:u32,pub source_id:String,pub revision:String,pub records:Vec<Record>}
fn source_id(v:&str)->bool{v.len()==36&&v.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_digit()||(b'a'..=b'f').contains(&b)})}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct ImportReceipt{pub schema_version:u32,pub source_id:String,pub fingerprint:String}
pub(super)fn hash(v:&str)->bool{v.len()==64&&v.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
impl ImportReceipt{pub(super)fn validate(&self)->Result<(),&'static str>{if self.schema_version!=1||!source_id(&self.source_id)||!hash(&self.fingerprint){Err("invalidDraftImport")}else{Ok(())}}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Retained{pub receipt:ImportReceipt,pub archive_revision:String}
impl Import{
 pub(super)fn fingerprint(&self)->Result<String,&'static str>{
  if self.schema_version!=1||!source_id(&self.source_id){return Err("invalidDraftImport");}
  records_valid(&self.records,ordinal(&self.revision)?)?;if self.records.iter().any(|r|r.target.origin==Origin::Unbound){return Err("invalidDraftImport");}
  let mut exact=self.clone();exact.records.sort_by(|a,b|a.target.cmp(&b.target));
  let bytes=serde_json::to_vec(&exact).map_err(|_|"invalidDraftImport")?;
  Ok(ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect())
 }
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Registry{pub schema_version:u32,pub generation:u64,pub revision:String,pub records:Vec<Record>,pub imports:Vec<ImportReceipt>,#[serde(default,skip_serializing_if="Vec::is_empty")]pub retained_sources:Vec<Retained>}
impl Registry{
 pub(super)fn import(&self,batch:&Import)->Result<(Self,ImportReceipt,bool),&'static str>{
  <Self as crate::private_registry::Document>::validate(self)?;let fingerprint=batch.fingerprint()?;
  let receipt=ImportReceipt{schema_version:1,source_id:batch.source_id.clone(),fingerprint};
  if self.retained_sources.iter().any(|r|r.receipt==receipt){return Ok((self.clone(),receipt,false));}
  if let Some(prior)=self.imports.iter().find(|r|r.source_id==batch.source_id){return if prior==&receipt{Ok((self.clone(),receipt,false))}else{Err("draftImportChanged")};}
  if self.imports.len()>=32{return Err("draftLimit");}
  let mut next=self.clone();next.schema_version=self.schema_version.max(2);
  for record in &batch.records{
   match next.records.iter().find(|r|r.target==record.target){Some(prior)if prior!=record=>return Err("draftImportConflict"),Some(_)=>{},None=>next.records.push(record.clone())}
  }
  next.revision=ordinal(&self.revision)?.max(ordinal(&batch.revision)?).to_string();next.imports.push(receipt.clone());
  <Self as crate::private_registry::Document>::validate(&next)?;Ok((next,receipt,true))
 }
 pub(super)fn write(&self,target:Target,expected:Option<&str>,text_units:Vec<u16>)->Result<(Self,Write),&'static str>{
  <Self as crate::private_registry::Document>::validate(self)?;target.validate()?;if expected.is_some_and(|v|ordinal(v).is_err()||v=="0"){return Err("invalidDraft");}if text_units.len()>TEXT{return Err("draftLimit");}
  let prior=self.records.iter().find(|r|r.target==target);
  if prior.map(|r|r.revision.as_str())!=expected{return Ok((self.clone(),Write::Conflict{record:prior.cloned()}));}
  let mut next=self.clone();next.schema_version=self.schema_version.max(2);next.revision=ordinal(&self.revision)?.checked_add(1).ok_or("draftLimit")?.to_string();
  let record=Record{target,revision:next.revision.clone(),text_units};
  if let Some(old)=next.records.iter_mut().find(|r|r.target==record.target){*old=record.clone();}else{next.records.push(record.clone());}
  <Self as crate::private_registry::Document>::validate(&next)?;Ok((next,Write::Saved{record}))
 }
}
impl crate::private_registry::Document for Registry{
 // Pretty JSON unit arrays include indentation and maximal target IDs as well
 // as text. Keep the full existing JS limits representable (inventory:128MiB).
 const MAX_BYTES:usize=64*1024*1024;const LOCK_NAME:&'static str=".session-drafts.lock";const TEMP_PREFIX:&'static str=".session-drafts-";
 fn empty()->Self{Self{schema_version:2,generation:0,revision:"0".into(),records:vec![],imports:vec![],retained_sources:vec![]}}
 fn validate(&self)->Result<(),&'static str>{
  if !matches!(self.schema_version,1|2|3)||(self.schema_version==1&&self.records.iter().any(|r|r.target.origin==Origin::Unbound))||self.imports.len()>32||self.retained_sources.len()>8||(self.schema_version<3&&!self.retained_sources.is_empty()){return Err("invalidDraftRegistry");}
  records_valid(&self.records,ordinal(&self.revision)?)?;let mut ids=BTreeSet::new();
  if self.imports.iter().any(|r|r.validate().is_err()||!ids.insert(&r.source_id)){return Err("invalidDraftRegistry");}
  let mut refs=BTreeSet::new();if self.retained_sources.iter().any(|r|r.receipt.validate().is_err()||!hash(&r.archive_revision)||!refs.insert((&r.receipt.source_id,&r.receipt.fingerprint))||!self.imports.iter().any(|i|i.source_id==r.receipt.source_id)){return Err("invalidDraftRegistry");}Ok(())
 }
 fn generation(&self)->u64{self.generation}
 fn next(mut self,old:&Self)->Result<Self,&'static str>{self.validate()?;if self.generation!=old.generation||ordinal(&self.revision)?<ordinal(&old.revision)?||self.schema_version<old.schema_version{return Err("registryConflict");}self.schema_version=self.schema_version.max(2);self.generation=old.generation.checked_add(1).ok_or("draftLimit")?;self.validate()?;Ok(self)}
}
#[derive(Clone,Debug,Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
pub(crate)enum Write{Saved{record:Record},Conflict{record:Option<Record>}}
