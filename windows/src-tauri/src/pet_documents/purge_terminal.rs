//! Durable terminal proof for metadata cleanup only. A checksum binds bytes;
//! it is not an execution grant or protection against an unrestricted same-UID
//! writer. Root/record/directory ownership and identity are checked separately.
use super::*;
const LIMIT:usize=WAL_LIMIT*2+2048;
const PREFIX:&str=".terminal-";
const STAGE:&str=".terminal-stage-";
pub(super)fn name(id:&str)->String{format!("{PREFIX}{id}.json")}
pub(super)fn id(value:&str)->Option<&str>{value.strip_prefix(PREFIX)?.strip_suffix(".json").filter(|id|crate::assistant_memory::valid_id(id))}
pub(super)fn staging(value:&str)->bool{value.strip_prefix(STAGE).is_some_and(crate::assistant_memory::valid_id)}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Envelope{version:u32,root_device:u64,root_inode:u64,device:u64,inode:u64,protocol_stamp:FileStamp,catalog_stamp:FileStamp,payload_device:u64,payload_inode:u64,lock_stamp:FileStamp,wal_stamp:FileStamp,wal_hash:String,wal:String}
type FileStamp=(u64,u64,u64,i64,i64,i64,i64);
fn stamp(m:&std::fs::Metadata)->FileStamp{(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec())}
pub(in crate::pet_documents::manifest::purge)struct Terminal{
 root:Dir,_proof:File,proof_stamp:FileStamp,bytes:Vec<u8>,envelope:Envelope,plan:Plan,_tx_lock:Option<Lock>,
}
impl Terminal{
 pub(in crate::pet_documents::manifest::purge)fn open(inbox:&Path,id:&str)->Result<Option<Self>,&'static str>{
  if !crate::assistant_memory::valid_id(id){return Err(ERR);}let inbox=Dir::at(inbox)?;let Some(root)=root(&inbox,false)?else{return Ok(None);};let _catalog=Lock::named(&root,".catalog.lock")?;
  if !root.exists(&name(id))?{return Ok(None);}Self::load(&root,id,true).map(Some)
 }
 pub(super)fn load(root:&Dir,id:&str,lock_tx:bool)->Result<Self,&'static str>{
  if !crate::assistant_memory::valid_id(id){return Err(ERR);}let proof=root.open(OsStr::new(&name(id)),libc::O_RDONLY)?.ok_or(ERR)?;let before=stamp(&proof.metadata().map_err(|_|ERR)?);
  let bytes=root.bytes(&name(id),LIMIT)?;if stamp(&proof.metadata().map_err(|_|ERR)?)!=before{return Err(ERR);}
  let current=root.open(OsStr::new(&name(id)),libc::O_RDONLY)?.ok_or(ERR)?;if stamp(&current.metadata().map_err(|_|ERR)?)!=before{return Err(ERR);}
  let envelope:Envelope=serde_json::from_slice(&bytes).map_err(|_|ERR)?;if envelope.version!=1||envelope.root_device!=root.device||envelope.root_inode!=root.inode||envelope.device!=root.device||envelope.wal.len()>WAL_LIMIT||hash(&envelope.wal.as_bytes())?!=envelope.wal_hash{return Err(ERR);}
  let plan:Plan=serde_json::from_str(&envelope.wal).map_err(|_|ERR)?;plan.validate()?;if plan.id!=id||!matches!(plan.phase,Phase::Cleaned|Phase::Restored){return Err(ERR);}
  let mut terminal=Self{root:root.duplicate()?,_proof:proof,proof_stamp:before,bytes,envelope,plan,_tx_lock:None};
  terminal.namespace_current()?;
  if let Some(dir)=terminal.directory()?{if lock_tx&&dir.exists(".lock")?{terminal._tx_lock=Some(Lock::existing(&dir,".lock")?);}}
  terminal.current()?;Ok(terminal)
 }
 pub(super)fn byte_len(&self)->usize{self.bytes.len()}
 pub(in crate::pet_documents::manifest::purge)fn id(&self)->&str{&self.plan.id}
 pub(in crate::pet_documents::manifest::purge)fn summary(&self)->Result<Summary,&'static str>{self.current()?;summary(&self.plan,self.envelope.wal.as_bytes())}
 fn namespace_current(&self)->Result<(),&'static str>{
  if self.root.bytes(".protocol.json",256)?!=PROTOCOL{return Err(ERR);}
  for(name,expected)in [(".protocol.json",self.envelope.protocol_stamp),(".catalog.lock",self.envelope.catalog_stamp)]{
   let f=self.root.open(OsStr::new(name),libc::O_RDONLY)?.ok_or(ERR)?;if stamp(&f.metadata().map_err(|_|ERR)?)!=expected{return Err(ERR);}
  }Ok(())
 }
 fn current(&self)->Result<(),&'static str>{
  self.namespace_current()?;
  if self.root.bytes(&name(self.id()),LIMIT)?!=self.bytes{return Err(ERR);}let f=self.root.open(OsStr::new(&name(self.id())),libc::O_RDONLY)?.ok_or(ERR)?;
  if stamp(&f.metadata().map_err(|_|ERR)?)!=self.proof_stamp{return Err(ERR);}self.directory()?;Ok(())
 }
 fn directory(&self)->Result<Option<Dir>,&'static str>{
  if !self.root.exists(self.id())?{return Ok(None);}let dir=self.root.child(self.id())?;
  if dir.device!=self.envelope.device||dir.inode!=self.envelope.inode{return Err(ERR);}
  if dir.names()?.iter().any(|n|!matches!(n.to_str(),Some(".lock"|"payload"|"transaction.json"))){return Err(ERR);}
  if dir.exists("transaction.json")?{let f=dir.open(OsStr::new("transaction.json"),libc::O_RDONLY)?.ok_or(ERR)?;if stamp(&f.metadata().map_err(|_|ERR)?)!=self.envelope.wal_stamp||dir.bytes("transaction.json",WAL_LIMIT)?!=self.envelope.wal.as_bytes(){return Err(ERR);}}
  if dir.exists("payload")?{let payload=dir.child("payload")?;if payload.device!=self.envelope.payload_device||payload.inode!=self.envelope.payload_inode||!payload.names()?.is_empty(){return Err(ERR);}}
  if dir.exists(".lock")?{let f=dir.open(OsStr::new(".lock"),libc::O_RDONLY)?.ok_or(ERR)?;if !f.metadata().is_ok_and(|m|m.is_file()&&m.len()==0&&stamp(&m)==self.envelope.lock_stamp){return Err(ERR);}}
  Ok(Some(dir))
 }
 pub(in crate::pet_documents::manifest::purge)fn remove_metadata(&mut self)->Result<(),&'static str>{let _catalog=Lock::named(&self.root,".catalog.lock")?;self.reap(&_catalog,|_|Ok(()))}
 fn reap(&mut self,catalog:&Lock,mut step:impl FnMut(&'static str)->Result<(),&'static str>)->Result<(),&'static str>{
  catalog.current(&self.root,".catalog.lock")?;self.namespace_current()?;
  if !self.root.exists(&name(self.id()))?{if self.root.exists(self.id())?{return Err(ERR);}self.root.sync()?;return Ok(());}
  self.current()?;step("terminalDurable")?;catalog.current(&self.root,".catalog.lock")?;
  if let Some(dir)=self.directory()?{
   if dir.exists(".lock")?{dir.unlink(OsStr::new(".lock"),false)?;dir.sync()?;}step("lockRemoved")?;catalog.current(&self.root,".catalog.lock")?;
   if dir.exists("payload")?{dir.unlink(OsStr::new("payload"),true)?;dir.sync()?;}step("payloadRemoved")?;catalog.current(&self.root,".catalog.lock")?;
   self.current()?;
   if dir.exists("transaction.json")?{dir.unlink(OsStr::new("transaction.json"),false)?;dir.sync()?;}step("walRemoved")?;catalog.current(&self.root,".catalog.lock")?;
   self.current()?;self.root.unlink(OsStr::new(self.id()),true)?;step("transactionDirectoryRemoved")?;catalog.current(&self.root,".catalog.lock")?;
  }
  self.root.sync()?;step("transactionDirectoryDurable")?;catalog.current(&self.root,".catalog.lock")?;self.current()?;
  self.root.unlink(OsStr::new(&name(self.id())),false)?;step("terminalRemoved")?;catalog.current(&self.root,".catalog.lock")?;self.root.sync()?;step("terminalRemovalDurable")?;Ok(())
 }
}
pub(super)fn finish(tx:&mut Transaction,mut step:impl FnMut(&'static str)->Result<(),&'static str>)->Result<(),&'static str>{
 if !matches!(tx.plan.phase,Phase::Cleaned|Phase::Restored)||tx._root.bytes(".protocol.json",256)?!=PROTOCOL{return Err(ERR);}let _catalog=Lock::named(&tx._root,".catalog.lock")?;
 if tx.terminal_seen&&!tx._root.exists(&name(tx.id()))?&&!tx._root.exists(tx.id())?{tx._root.sync()?;return Ok(());}
 if !tx._root.exists(&name(tx.id()))?{
  tx.wal_current()?;let published:Plan=serde_json::from_slice(tx.published.as_ref().ok_or(ERR)?).map_err(|_|ERR)?;if published!=tx.plan{return Err(ERR);}if !tx.payload.names()?.is_empty()||tx.dir.names()?.iter().any(|n|!matches!(n.to_str(),Some(".lock"|"payload"|"transaction.json"))){return Err(ERR);}
  let wal=String::from_utf8(tx.published.clone().ok_or(ERR)?).map_err(|_|ERR)?;
  tx._lock.current(&tx.dir,".lock")?;
  let lock_stamp=stamp(&tx._lock.0.metadata().map_err(|_|ERR)?);let wal_file=tx.dir.open(OsStr::new("transaction.json"),libc::O_RDONLY)?.ok_or(ERR)?;let wal_stamp=stamp(&wal_file.metadata().map_err(|_|ERR)?);
  let protocol=tx._root.open(OsStr::new(".protocol.json"),libc::O_RDONLY)?.ok_or(ERR)?;let protocol_stamp=stamp(&protocol.metadata().map_err(|_|ERR)?);let catalog_stamp=stamp(&_catalog.0.metadata().map_err(|_|ERR)?);
  let envelope=Envelope{version:1,protocol_stamp,catalog_stamp,payload_device:tx.payload.device,payload_inode:tx.payload.inode,lock_stamp,wal_stamp,root_device:tx._root.device,root_inode:tx._root.inode,device:tx.dir.device,inode:tx.dir.inode,wal_hash:hash(&wal.as_bytes())?,wal};let bytes=serde_json::to_vec(&envelope).map_err(|_|ERR)?;if bytes.len()>LIMIT{return Err(ERR);}
  let temp=format!("{STAGE}{}",crate::assistant_memory::new_id());tx._root.create_bytes(&temp,&bytes)?;step("terminalStaged")?;_catalog.current(&tx._root,".catalog.lock")?;
  tx._root.rename(OsStr::new(&temp),&tx._root,OsStr::new(&name(tx.id())),true)?;step("terminalPublished")?;_catalog.current(&tx._root,".catalog.lock")?;tx._root.sync()?;
 }else{
  let f=tx._root.open(OsStr::new(&name(tx.id())),libc::O_RDONLY)?.ok_or(ERR)?;f.sync_all().map_err(|_|ERR)?;tx._root.sync()?;
 }
 let mut terminal=Terminal::load(&tx._root,tx.id(),false)?;
 if terminal.envelope.wal.as_bytes()!=tx.published.as_ref().ok_or(ERR)?.as_slice(){return Err(ERR);}tx.terminal_seen=true;
 // A fully written duplicate stage is provably the same terminal metadata.
 // Partial/foreign stages remain visible as unavailable, never blindly erased.
 for candidate in tx._root.names()?{let Some(candidate)=candidate.to_str()else{continue;};if !staging(candidate){continue;}
  let Ok(Some(file))=tx._root.open(OsStr::new(candidate),libc::O_RDONLY)else{continue;};let Ok(meta)=file.metadata()else{continue;};let expected=stamp(&meta);
  if tx._root.bytes(candidate,LIMIT).ok().as_ref()!=Some(&terminal.bytes){continue;}
  let Some(current)=tx._root.open(OsStr::new(candidate),libc::O_RDONLY)?else{continue;};if stamp(&current.metadata().map_err(|_|ERR)?)!=expected{return Err(ERR);}
  _catalog.current(&tx._root,".catalog.lock")?;tx._root.unlink(OsStr::new(candidate),false)?;tx._root.sync()?;
 }
 terminal.reap(&_catalog,step)
}
#[cfg(test)]#[path="purge_terminal_tests.rs"]mod tests;
