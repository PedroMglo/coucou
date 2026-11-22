//! Immutable originals under the admitted assistant payload, including partial
//! publications. They contain data, never recovered process/authority leases.
use super::model::{Import,ImportReceipt,Retained,Registry};
use crate::private_registry::{self,Document,Snapshot};
use serde::{Deserialize,Serialize};
use std::path::{Path,PathBuf};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Archive{schema_version:u32,generation:u64,pub source:Option<Import>}
impl Document for Archive{
 const MAX_BYTES:usize=64*1024*1024;const LOCK_NAME:&'static str=".draft-sources.lock";const TEMP_PREFIX:&'static str=".draft-source-";
 fn empty()->Self{Self{schema_version:1,generation:0,source:None}}
 fn validate(&self)->Result<(),&'static str>{if self.schema_version!=1||self.generation>1||self.generation==1&&self.source.is_none(){return Err("invalidDraftImport");}if let Some(s)=&self.source{s.fingerprint()?;}Ok(())}
 fn generation(&self)->u64{self.generation}
 fn next(mut self,old:&Self)->Result<Self,&'static str>{self.validate()?;if old.generation!=0||old.source.is_some()||self.generation!=0||self.source.is_none(){return Err("registryConflict");}self.generation=1;Ok(self)}
}
fn path(catalogue:&Path,receipt:&ImportReceipt)->Result<PathBuf,&'static str>{receipt.validate()?;Ok(catalogue.with_file_name(format!("draft-source-{}-{}.json",receipt.source_id,receipt.fingerprint)))}
fn from_name(name:&std::ffi::OsStr)->Option<ImportReceipt>{let value=name.to_str()?.strip_prefix("draft-source-")?.strip_suffix(".json")?;if value.len()!=101||value.as_bytes()[36]!=b'-'{return None;}let receipt=ImportReceipt{schema_version:1,source_id:value.get(..36)?.into(),fingerprint:value.get(37..)?.into()};receipt.validate().ok()?;Some(receipt)}
pub(super)fn read(catalogue:&Path,reference:&Retained)->Result<Snapshot<Archive>,&'static str>{
 if !super::model::hash(&reference.archive_revision){return Err("invalidDraftImport");}
 let snap=private_registry::load_at::<Archive>(&path(catalogue,&reference.receipt)?)?;
 if snap.revision!=reference.archive_revision{return Err("registryChanged");}
 verify(&snap,&reference.receipt)?;Ok(snap)
}
fn verify(snap:&Snapshot<Archive>,receipt:&ImportReceipt)->Result<(),&'static str>{
 let source=snap.registry.source.as_ref().ok_or("registryUnavailable")?;
 if source.source_id!=receipt.source_id||source.fingerprint()?!=receipt.fingerprint||!snap.unchanged(){return Err("registryChanged");}Ok(())
}
pub(super)fn retain(catalogue:&Path,batch:&Import,current:impl Fn()->Result<(),&'static str>)->Result<(Retained,Snapshot<Archive>),&'static str>{
 current()?;let receipt=ImportReceipt{schema_version:1,source_id:batch.source_id.clone(),fingerprint:batch.fingerprint()?};let path=path(catalogue,&receipt)?;
 let original=private_registry::load_at::<Archive>(&path)?;
 let snap=if original.revision!="missing"{verify(&original,&receipt)?;original}else{
  let mut canonical=batch.clone();canonical.records.sort_by(|a,b|a.target.cmp(&b.target));let next=Archive{schema_version:1,generation:0,source:Some(canonical)};
  private_registry::save_guarded(&path,"missing",next,|_,_|{
   current()?;let count=private_registry::sibling_names(&path,4096)?.iter().filter(|n|from_name(n).is_some()).count();if count>=8{return Err("draftLimit");}Ok(())
  },&current)?
 };
 current()?;verify(&snap,&receipt)?;Ok((Retained{receipt,archive_revision:snap.revision.clone()},snap))
}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct SourceInfo{pub reference:Retained,pub records:usize,pub revision:String,pub status:&'static str}
pub(super)fn list(catalogue:&Path,current:impl Fn()->Result<(),&'static str>)->Result<Vec<SourceInfo>,&'static str>{
 current()?;let before=private_registry::load_at::<Registry>(catalogue)?;let names=private_registry::sibling_names(catalogue,4096)?;
 let receipts:Vec<_>=names.iter().filter_map(|n|from_name(n)).collect();if receipts.len()>8{return Err("draftLimit");}
 let mut result=vec![];let mut pinned=vec![];
 for receipt in receipts{let snap=private_registry::load_at::<Archive>(&path(catalogue,&receipt)?)?;verify(&snap,&receipt)?;let reference=Retained{receipt,archive_revision:snap.revision.clone()};let source=snap.registry.source.as_ref().ok_or("registryUnavailable")?;let status=if before.registry.retained_sources.iter().any(|r|r==&reference){"retained"}else{"pending"};result.push(SourceInfo{reference,records:source.records.len(),revision:source.revision.clone(),status});pinned.push(snap);}
 // Missing committed originals must not disappear from the management view.
 if before.registry.retained_sources.iter().any(|r|!result.iter().any(|s|&s.reference==r)){return Err("registryUnavailable");}
 current()?;if !before.unchanged()||pinned.iter().any(|s|!s.unchanged()){return Err("registryChanged");}Ok(result)
}
