//! Opaque preparation evidence only. Never parse a retained file as authority.
use super::*;
use crate::pet_data::private_fs::Node;
use std::{io::Read,os::unix::{ffi::OsStrExt,fs::MetadataExt}};

const MAX_ENTRIES:usize=4096;
const MAX_FILE:u64=8*1024*1024;
const MAX_TOTAL:u64=64*1024*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Location{device:u64,inode:u64,mount:u64}
pub(super)fn location(dir:&Dir)->Result<Location>{let at=private_fs::location(&dir.node()?.file)?;Ok(Location{device:at.device,inode:at.inode,mount:at.mount})}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Content{pub hash:String,pub entries:usize,pub bytes:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Opaque{pub content:Content,pub location:Location,pub revision:String}
impl Opaque{
 pub(super)fn valid(&self)->bool{valid_hex(&self.content.hash,64)&&valid_hex(&self.revision,64)&&self.content.entries>0&&self.content.entries<=MAX_ENTRIES&&self.content.bytes<=MAX_TOTAL&&self.location.inode>0&&self.location.mount>0}
}
#[derive(Default)]pub(super)struct Budget{entries:usize,bytes:u64}
impl Budget{
 fn add(&mut self,bytes:u64)->Result<()>{self.entries=self.entries.checked_add(1).ok_or(Error::Limit)?;self.bytes=self.bytes.checked_add(bytes).ok_or(Error::Limit)?;if self.entries>MAX_ENTRIES||self.bytes>MAX_TOTAL{return Err(Error::Limit);}Ok(())}
}
#[derive(Serialize)]struct Item{path:Vec<Vec<u8>>,directory:bool,bytes:u64,hash:Option<String>}
pub(super)fn inspect(dir:&Dir,budget:&mut Budget)->Result<Opaque>{
 fn walk(node:&Node,path:Vec<Vec<u8>>,depth:usize,budget:&mut Budget,items:&mut Vec<Item>,revisions:&mut Vec<String>)->Result<()>{
  if depth>16{return Err(Error::Limit);}node.current()?;let m=node.file.metadata().map_err(|_|Error::Unavailable)?;
  if node.directory{
   if !m.is_dir()||m.uid()!=unsafe{libc::geteuid()}||m.mode()&0o7777!=0o700{return Err(Error::UnsafePath);}
   budget.add(0)?;items.push(Item{path:path.clone(),directory:true,bytes:0,hash:None});
   if depth>0{revisions.push(digest(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?));}
   for name in node.names(MAX_ENTRIES)?{let child=node.child(&name)?.ok_or(Error::SourceChanged)?;let mut next=path.clone();next.push(name.as_bytes().to_vec());walk(&child,next,depth+1,budget,items,revisions)?;node.same_child(&name,&child)?;}
  }else{
   if !crate::pet_data::staging_fs::private_file(&m){return Err(Error::UnsafePath);}if node.len>MAX_FILE{return Err(Error::Limit);}budget.add(node.len)?;
   let mut raw=vec![];(&node.file).take(MAX_FILE+1).read_to_end(&mut raw).map_err(|_|Error::Unavailable)?;
   if raw.len()as u64!=node.len{return Err(Error::SourceChanged);}node.current()?;
   items.push(Item{path,directory:false,bytes:node.len,hash:Some(digest(&raw))});revisions.push(digest(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?));
  }node.current()
 }
 dir.check()?;let mut items=vec![];let mut revisions=vec![];let before=(budget.entries,budget.bytes);walk(&dir.node()?,vec![],0,budget,&mut items,&mut revisions)?;
 let bytes=serde_json::to_vec(&items).map_err(|_|Error::Unavailable)?;if bytes.len()>8*1024*1024{return Err(Error::Limit);}
 let at=location(dir)?;let revision=digest(&serde_json::to_vec(&(&at,revisions)).map_err(|_|Error::Unavailable)?);dir.check()?;
 Ok(Opaque{content:Content{hash:digest(&bytes),entries:budget.entries-before.0,bytes:budget.bytes-before.1},location:at,revision})
}
