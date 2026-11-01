use super::{archives,model::{Import,ImportReceipt,Record,Registry,ordinal}};
use crate::private_registry::{self,Document,Snapshot};
use serde::{Deserialize,Serialize};
use std::path::Path;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Policy{RetainOnly,MergeCompatible}
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Summary{pub total:usize,pub missing:usize,pub identical:usize,pub conflicts:usize,pub can_merge:bool,pub already_retained:bool}
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Review{pub schema_version:u32,pub native_revision:String,pub receipt:ImportReceipt,pub policy:Policy,pub expires_at:u64,pub summary:Summary,pub fingerprint:String}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Row{pub original:Record,pub native:Option<Record>,pub status:&'static str}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Page{pub review:Review,pub rows:Vec<Row>,pub next_offset:Option<usize>}
fn merged(before:&Registry,batch:&Import)->Result<Registry,&'static str>{
 let mut next=before.clone();
 for record in &batch.records{if !next.records.iter().any(|r|r.target==record.target){next.records.push(record.clone());}}
 next.revision=ordinal(&before.revision)?.max(ordinal(&batch.revision)?).to_string();next.validate()?;Ok(next)
}
fn plan(before:&Snapshot<Registry>,batch:&Import,policy:Policy,expires_at:u64,now:u64)->Result<Review,&'static str>{
 if expires_at<=now||expires_at>now.checked_add(300).ok_or("invalidDraft")?{return Err("draftReviewExpired");}
 let receipt=ImportReceipt{schema_version:1,source_id:batch.source_id.clone(),fingerprint:batch.fingerprint()?};
 let already_retained=before.registry.retained_sources.iter().any(|r|r.receipt==receipt);
 if already_retained&&policy!=Policy::RetainOnly{return Err("draftReviewChanged");}
 let(mut missing,mut identical,mut conflicts)=(0,0,0);
 for source in &batch.records{match before.registry.records.iter().find(|r|r.target==source.target){None=>missing+=1,Some(n)if n==source=>identical+=1,Some(_)=>conflicts+=1}}
 let can_merge=if already_retained{false}else{match merged(&before.registry,batch){Ok(_)=>true,Err("draftLimit")=>false,Err(e)=>return Err(e)}};
 let summary=Summary{total:batch.records.len(),missing,identical,conflicts,can_merge,already_retained};
 let bytes=serde_json::to_vec(&("coucou-draft-source-reconciliation-v1",&before.revision,&receipt,policy,expires_at,&summary)).map_err(|_|"invalidDraftImport")?;
 let fingerprint=ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect();
 Ok(Review{schema_version:1,native_revision:before.revision.clone(),receipt,policy,expires_at,summary,fingerprint})
}
pub(super)fn review(path:&Path,batch:&Import,policy:Policy,expected:Option<&str>,expires:Option<u64>,offset:usize,now:u64,current:impl Fn()->Result<(),&'static str>)->Result<Page,&'static str>{
 if offset>super::model::RECORDS{return Err("invalidDraft");}current()?;let before=private_registry::load_at::<Registry>(path)?;
 if expected.is_some_and(|v|v!=before.revision){return Err("registryConflict");}
 let review=plan(&before,batch,policy,expires.unwrap_or(now.checked_add(300).ok_or("invalidDraft")?),now)?;
 let pinned=before.registry.retained_sources.iter().find(|r|r.receipt==review.receipt).map(|r|archives::read(path,r)).transpose()?;
 let mut sorted:Vec<_>=batch.records.iter().collect();sorted.sort_by(|a,b|a.target.cmp(&b.target));
 let rows=sorted.into_iter().skip(offset).take(16).map(|r|{let native=before.registry.records.iter().find(|n|n.target==r.target).cloned();let status=match &native{None=>"missing",Some(n)if n==r=>"identical",_=>"conflict"};Row{original:r.clone(),native,status}}).collect();
 current()?;if !before.unchanged()||pinned.is_some_and(|s|!s.unchanged()){return Err("registryChanged");}
 Ok(Page{review,rows,next_offset:(offset.saturating_add(16)<batch.records.len()).then_some(offset+16)})
}
pub(super)fn commit(path:&Path,batch:&Import,reviewed:Review,approved:bool,now:impl Fn()->u64,current:impl Fn()->Result<(),&'static str>)->Result<ImportReceipt,&'static str>{commit_prepared(path,batch,reviewed,approved,now,current,||Ok(()))}
pub(super)fn commit_prepared(path:&Path,batch:&Import,reviewed:Review,approved:bool,now:impl Fn()->u64,current:impl Fn()->Result<(),&'static str>,after_archive:impl FnOnce()->Result<(),&'static str>)->Result<ImportReceipt,&'static str>{
 if !approved{return Err("blocked");}current()?;let before=private_registry::load_at::<Registry>(path)?;
 if before.revision!=reviewed.native_revision{return Err("registryConflict");}
 let exact=plan(&before,batch,reviewed.policy,reviewed.expires_at,now())?;if exact!=reviewed{return Err("draftReviewChanged");}
 if let Some(reference)=before.registry.retained_sources.iter().find(|r|r.receipt==exact.receipt){
  let archive=archives::read(path,reference)?;current()?;if now()>=exact.expires_at||!before.unchanged()||!archive.unchanged(){return Err("registryChanged");}return Ok(exact.receipt);
 }
 if before.registry.retained_sources.len()>=8||!before.registry.imports.iter().any(|r|r.source_id==batch.source_id)&&before.registry.imports.len()>=32{return Err("draftLimit");}
 let mut next=if exact.policy==Policy::MergeCompatible{if !exact.summary.can_merge{return Err("draftLimit");}merged(&before.registry,batch)?}else{before.registry.clone()};
 // Preserve all losing bytes before publishing an accepted source receipt.
 // A partial artifact is intentionally retained and discoverable, not retried.
 let(reference,archive)=archives::retain(path,batch,||{current()?;if now()>=exact.expires_at{return Err("draftReviewExpired");}if !before.unchanged(){return Err("registryConflict");}Ok(())})?;
 after_archive()?;next.schema_version=3;next.retained_sources.push(reference);
 if !next.imports.iter().any(|r|r.source_id==batch.source_id){next.imports.push(exact.receipt.clone());}next.validate()?;
 private_registry::save_guarded(path,&before.revision,next,|_,_|current(),||{current()?;if now()>=exact.expires_at{return Err("draftReviewExpired");}if !archive.unchanged(){return Err("registryChanged");}Ok(())})?;
 Ok(exact.receipt)
}
