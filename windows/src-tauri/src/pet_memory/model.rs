//! Explicit user memories. These records are data, never instructions, grants,
//! verified tool results, or permission to read the referenced conversation.
use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
pub(super)const MAX_BYTES:usize=1024*1024;
pub(super)const MAX_ITEMS:usize=256;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Owner{pub pet_id:String,pub scope:Scope}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub(crate)enum Scope{Pet{},Conversation{conversation_id:String}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Kind{Preference,Note}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Sensitivity{Normal,Sensitive}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Provenance{ExplicitUser}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Retention{UntilDeleted}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Entry{
 pub id:String,pub revision:u64,pub owner:Owner,pub kind:Kind,pub title:String,pub content:String,
 pub provenance:Provenance,pub sensitivity:Sensitivity,pub retention:Retention,
 pub created_at:u64,pub updated_at:u64,pub observed_at:u64,pub expires_at:Option<u64>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Registry{pub version:u32,pub generation:u64,pub items:Vec<Entry>}
#[derive(Clone,Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Input{pub kind:Kind,pub title:String,pub content:String,pub sensitivity:Sensitivity,pub expires_at:Option<u64>}
#[derive(Clone,Debug,Deserialize)]
#[serde(tag="operation",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub(crate)enum Mutation{Create{input:Input},Update{id:String,expected_item_revision:u64,input:Input},Delete{id:String,expected_item_revision:u64}}
#[derive(Clone,Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Query{pub owner:Owner,pub query:String,pub limit:u16,pub include_expired:bool,#[serde(default)]pub offset:u16,#[serde(default)]pub expected_revision:Option<String>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct View{pub entry:Entry,pub available:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Page{pub schema_version:u32,pub owner:Owner,pub revision:String,pub items:Vec<View>,pub has_more:bool,pub next_offset:Option<u16>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{pub schema_version:u32,pub owner:Owner,pub revision:String,pub id:String,pub item:Option<Entry>}
fn text(value:&str,max:usize,multiline:bool)->bool{!value.trim().is_empty()&&value.len()<=max&&!value.chars().any(|c|c.is_control()&&!(multiline&&matches!(c,'\n'|'\t')))}
impl Owner{
 pub(super)fn valid(&self)->bool{
  !self.pet_id.is_empty()&&self.pet_id.len()<=64&&self.pet_id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))
   &&match &self.scope{Scope::Pet{}=>true,Scope::Conversation{conversation_id}=>crate::assistant_memory::valid_id(conversation_id)}
 }
}
impl Input{
 fn valid(&self,now:u64)->bool{text(&self.title,160,false)&&text(&self.content,8192,true)&&self.expires_at.is_none_or(|at|at>now)}
}
impl Entry{
 pub(super)fn available(&self,now:u64)->bool{self.expires_at.is_none_or(|at|at>now)}
 fn valid(&self)->bool{crate::assistant_memory::valid_id(&self.id)&&self.revision>0&&self.owner.valid()&&text(&self.title,160,false)&&text(&self.content,8192,true)&&self.created_at>0&&self.updated_at>=self.created_at&&self.observed_at==self.updated_at&&self.expires_at.is_none_or(|at|at>self.created_at)}
}
impl Registry{
 pub(super)fn empty()->Self{Self{version:1,generation:0,items:vec![]}}
 pub(super)fn validate(&self)->Result<(),&'static str>{
  let mut ids=BTreeSet::new();
  if self.version!=1||self.items.len()>MAX_ITEMS||self.items.iter().any(|e|!e.valid()||!ids.insert(&e.id)){return Err("invalidMemory");}Ok(())
 }
 pub(super)fn next(mut self,old:&Self)->Result<Self,&'static str>{
  if self.generation!=old.generation{return Err("registryConflict");}
  for item in &self.items{
   match old.items.iter().find(|e|e.id==item.id){
    Some(before)if item!=before=>if item.owner!=before.owner||item.created_at!=before.created_at||item.revision!=before.revision.checked_add(1).ok_or("revisionLimit")?{return Err("memoryRevisionConflict");},
    None if item.revision!=1=>return Err("memoryRevisionConflict"),_=>{},
   }
  }
  self.generation=old.generation.checked_add(1).ok_or("revisionLimit")?;self.validate()?;Ok(self)
 }
 pub(super)fn apply(&self,owner:&Owner,mutation:Mutation,now:u64)->Result<(Self,String),&'static str>{
  if !owner.valid()||now==0{return Err("invalidOwner");}self.validate()?;let mut next=self.clone();
  let id=match mutation{
   Mutation::Create{input}=>{
    if !input.valid(now){return Err("invalidMemoryInput");}if next.items.len()>=MAX_ITEMS{return Err("registryLimit");}
    let id=crate::assistant_memory::new_id();next.items.push(Entry{id:id.clone(),revision:1,owner:owner.clone(),kind:input.kind,title:input.title,content:input.content,provenance:Provenance::ExplicitUser,sensitivity:input.sensitivity,retention:Retention::UntilDeleted,created_at:now,updated_at:now,observed_at:now,expires_at:input.expires_at});id
   },
   Mutation::Update{id,expected_item_revision,input}=>{
    if !input.valid(now){return Err("invalidMemoryInput");}
    let item=next.items.iter_mut().find(|e|e.id==id&&e.owner==*owner).ok_or("memoryNotFound")?;
    if item.revision!=expected_item_revision{return Err("memoryRevisionConflict");}
    if now<item.updated_at{return Err("clockChanged");}
    item.revision=item.revision.checked_add(1).ok_or("revisionLimit")?;item.kind=input.kind;item.title=input.title;item.content=input.content;item.sensitivity=input.sensitivity;item.expires_at=input.expires_at;item.updated_at=now;item.observed_at=now;id
   },
   Mutation::Delete{id,expected_item_revision}=>{
    let at=next.items.iter().position(|e|e.id==id&&e.owner==*owner).ok_or("memoryNotFound")?;
    if next.items[at].revision!=expected_item_revision{return Err("memoryRevisionConflict");}next.items.remove(at);id
   },
  };Ok((next,id))
 }
 pub(super)fn page(&self,query:&Query,revision:String,now:u64)->Result<Page,&'static str>{
  self.validate()?;
  if !query.owner.valid()||query.query.len()>256||query.query.chars().any(char::is_control)||!(1..=50).contains(&query.limit)||query.offset as usize>MAX_ITEMS{return Err("invalidMemoryQuery");}
  if query.expected_revision.as_ref().is_some_and(|r|*r!=revision)||query.offset>0&&query.expected_revision.is_none(){return Err("registryConflict");}
  let lower=query.query.to_lowercase();let terms:Vec<_>=lower.split_whitespace().collect();if terms.len()>8{return Err("invalidMemoryQuery");}
  let mut matches:Vec<_>=self.items.iter().filter(|e|e.owner==query.owner&&(query.include_expired||e.available(now))).filter_map(|e|{
   let title=e.title.to_lowercase();let content=e.content.to_lowercase();
   terms.iter().all(|t|title.contains(t)||content.contains(t)).then(||(terms.iter().filter(|t|title.contains(**t)).count(),e))
  }).collect();
  matches.sort_by(|(sa,a),(sb,b)|sb.cmp(sa).then_with(||b.updated_at.cmp(&a.updated_at)).then_with(||a.id.cmp(&b.id)));
  let mut used=0;let mut items=vec![];let mut more=false;
  for(_,entry)in matches.into_iter().skip(query.offset as usize){
   let bytes=serde_json::to_vec(entry).map_err(|_|"invalidMemory")?.len();
   if items.len()>=query.limit as usize||used+bytes>64*1024{more=true;break;}used+=bytes;items.push(View{entry:entry.clone(),available:entry.available(now)});
  }
  let next_offset=more.then_some(query.offset+items.len()as u16);
  Ok(Page{schema_version:1,owner:query.owner.clone(),revision,items,has_more:more,next_offset})
 }
}
#[cfg(target_os="linux")]
impl crate::private_registry::Document for Registry{
 const MAX_BYTES:usize=MAX_BYTES;const LOCK_NAME:&'static str=".memories.lock";const TEMP_PREFIX:&'static str=".memories-";
 fn empty()->Self{Self::empty()}fn validate(&self)->Result<(),&'static str>{Self::validate(self)}fn generation(&self)->u64{self.generation}fn next(self,old:&Self)->Result<Self,&'static str>{Self::next(self,old)}
}
