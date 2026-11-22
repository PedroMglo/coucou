//! Private same-filesystem quarantine and restartable metadata-only WAL.
//! The caller must hold the native chat/reference/selection consent fence.
//! These primitives are deliberately not exposed as commands or model tools.
//! renameat2(RENAME_NOREPLACE): https://man7.org/linux/man-pages/man2/rename.2.html
//! Descriptor-relative unlink: https://man7.org/linux/man-pages/man2/unlink.2.html
use super::*;
use super::super::super::copy_review::{Budget,Copy,RetentionNode};
use std::{ffi::{CString,OsStr,OsString},fs::File,io::{Read,Write},os::{fd::{AsRawFd,FromRawFd},unix::{ffi::OsStrExt,fs::MetadataExt}}};
#[path="purge_terminal.rs"]mod terminal;
pub(super)use terminal::Terminal;
const ERR:&str="retentionQuarantineChanged";
pub(super)const RESTORE_UNAVAILABLE:&str="retentionRestoreCopyIncompleteOrChanged";
pub(super)const RESTORE_CLAIMED:&str="retentionRestoreDestinationClaimed";
pub(super)const CLEANUP_UNVERIFIED:&str="retentionCleanupPreflightUnverified";
pub(super)const CLEANUP_INTERRUPTED:&str="retentionCleanupInterrupted";
const WAL_LIMIT:usize=4*1024*1024;
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-copy-quarantine\"}\n";
fn c(name:&OsStr)->Result<CString,&'static str>{
    if name.is_empty()||Path::new(name).components().count()!=1||!matches!(Path::new(name).components().next(),Some(Component::Normal(_))){return Err(ERR);}
    CString::new(name.as_bytes()).map_err(|_|ERR)
}
fn private(m:&std::fs::Metadata)->bool{m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0&&(m.is_dir()||m.is_file()&&m.nlink()==1)}
struct Dir{file:File,path:PathBuf,device:u64,inode:u64}
impl Dir{
    fn at(path:&Path)->Result<Self,&'static str>{
        let f=OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path).map_err(|_|ERR)?;
        Self::from(f,path.into())
    }
    fn from(file:File,path:PathBuf)->Result<Self,&'static str>{
        let m=file.metadata().map_err(|_|ERR)?;if !m.is_dir()||!private(&m){return Err(ERR);}let d=Self{file,path,device:m.dev(),inode:m.ino()};d.check()?;Ok(d)
    }
    fn duplicate(&self)->Result<Self,&'static str>{self.check()?;Self::from(self.file.try_clone().map_err(|_|ERR)?,self.path.clone())}
    fn check(&self)->Result<(),&'static str>{
        if !std::fs::symlink_metadata(&self.path).is_ok_and(|m|private(&m)&&m.is_dir()&&m.dev()==self.device&&m.ino()==self.inode){return Err(ERR);}Ok(())
    }
    fn open(&self,name:&OsStr,flags:i32)->Result<Option<File>,&'static str>{
        self.check()?;let name=c(name)?;let fd=unsafe{libc::openat(self.file.as_raw_fd(),name.as_ptr(),flags|libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK,0o600)};
        if fd<0{if std::io::Error::last_os_error().kind()==std::io::ErrorKind::NotFound{Ok(None)}else{Err(ERR)}}else{
            let f=unsafe{File::from_raw_fd(fd)};if !f.metadata().is_ok_and(|m|private(&m)){return Err(ERR);}Ok(Some(f))
        }
    }
    fn child(&self,name:&str)->Result<Self,&'static str>{Self::from(self.open(OsStr::new(name),libc::O_RDONLY|libc::O_DIRECTORY)?.ok_or(ERR)?,self.path.join(name))}
    fn exists(&self,name:&str)->Result<bool,&'static str>{
        self.check()?;let name=c(OsStr::new(name))?;let mut st=std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe{libc::fstatat(self.file.as_raw_fd(),name.as_ptr(),st.as_mut_ptr(),libc::AT_SYMLINK_NOFOLLOW)}==0{Ok(true)}else if std::io::Error::last_os_error().kind()==std::io::ErrorKind::NotFound{Ok(false)}else{Err(ERR)}
    }
    fn mkdir(&self,name:&str)->Result<(),&'static str>{self.check()?;let name=c(OsStr::new(name))?;if unsafe{libc::mkdirat(self.file.as_raw_fd(),name.as_ptr(),0o700)}!=0{return Err(ERR);}self.sync()}
    fn sync(&self)->Result<(),&'static str>{self.check()?;self.file.sync_all().map_err(|_|ERR)}
    fn names(&self)->Result<Vec<OsString>,&'static str>{
        self.check()?;let mut names=vec![];for entry in std::fs::read_dir(format!("/proc/self/fd/{}",self.file.as_raw_fd())).map_err(|_|ERR)?{if names.len()>=4096{return Err(ERR);}names.push(entry.map_err(|_|ERR)?.file_name());}Ok(names)
    }
    fn rename(&self,name:&OsStr,to:&Dir,new:&OsStr,no_replace:bool)->Result<(),&'static str>{
        self.check()?;to.check()?;let name=c(name)?;let new=c(new)?;
        if unsafe{libc::renameat2(self.file.as_raw_fd(),name.as_ptr(),to.file.as_raw_fd(),new.as_ptr(),if no_replace{libc::RENAME_NOREPLACE}else{0})}!=0{return Err(ERR);}Ok(())
    }
    fn unlink(&self,name:&OsStr,folder:bool)->Result<(),&'static str>{
        self.check()?;let name=c(name)?;if unsafe{libc::unlinkat(self.file.as_raw_fd(),name.as_ptr(),if folder{libc::AT_REMOVEDIR}else{0})}!=0{return Err(ERR);}Ok(())
    }
    fn bytes(&self,name:&str,limit:usize)->Result<Vec<u8>,&'static str>{
        let f=self.open(OsStr::new(name),libc::O_RDONLY)?.ok_or(ERR)?;let before=f.metadata().map_err(|_|ERR)?;
        if !before.is_file()||before.len()>limit as u64{return Err(ERR);}let mut bytes=vec![];(&f).take(limit as u64+1).read_to_end(&mut bytes).map_err(|_|ERR)?;
        let after=f.metadata().map_err(|_|ERR)?;let current=self.open(OsStr::new(name),libc::O_RDONLY)?.ok_or(ERR)?.metadata().map_err(|_|ERR)?;
        let stamp=|m:&std::fs::Metadata|(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec());
        if bytes.len()>limit||stamp(&before)!=stamp(&after)||stamp(&before)!=stamp(&current){return Err(ERR);}Ok(bytes)
    }
    fn create_bytes(&self,name:&str,bytes:&[u8])->Result<(),&'static str>{
        let mut file=self.open(OsStr::new(name),libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL)?.ok_or(ERR)?;
        file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|_|ERR)?;self.sync()
    }
}
fn root(inbox:&Dir,create:bool)->Result<Option<Dir>,&'static str>{
    if inbox.open(OsStr::new(CONTROL_ROOT),libc::O_RDONLY|libc::O_DIRECTORY)?.is_none(){
        if !create{return Ok(None);}
        // A complete protocol marker is published together with its directory.
        // Incomplete setup directories are retained, never adopted as copies.
        let setup=format!(".coucou-purge-setup-{}",crate::assistant_memory::new_id());inbox.mkdir(&setup)?;
        let temp=inbox.child(&setup)?;temp.create_bytes(".protocol.json",PROTOCOL)?;
        if inbox.rename(OsStr::new(&setup),inbox,OsStr::new(CONTROL_ROOT),true).is_err(){
            // A concurrent creator may have won. Validate it below. Do not
            // remove a setup folder whose publication could be uncertain.
            if inbox.open(OsStr::new(CONTROL_ROOT),libc::O_RDONLY|libc::O_DIRECTORY)?.is_none(){return Err(ERR);}
        }inbox.sync()?;
    }
    let d=inbox.child(CONTROL_ROOT)?;if d.device!=inbox.device||d.bytes(".protocol.json",256)?!=PROTOCOL{return Err(ERR);}Ok(Some(d))
}
struct Lock(File);
impl Lock{
    fn acquire(dir:&Dir)->Result<Self,&'static str>{
        Self::named(dir,".lock")
    }
    fn named(dir:&Dir,name:&str)->Result<Self,&'static str>{Self::open(dir,name,true)}
    fn existing(dir:&Dir,name:&str)->Result<Self,&'static str>{Self::open(dir,name,false)}
    fn open(dir:&Dir,name:&str,create:bool)->Result<Self,&'static str>{
        let f=dir.open(OsStr::new(name),libc::O_RDWR|if create{libc::O_CREAT}else{0})?.ok_or(ERR)?;
        if !f.metadata().is_ok_and(|m|m.is_file()&&m.len()==0)||unsafe{libc::flock(f.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err("retentionBusy");}
        let lock=Self(f);lock.current(dir,name)?;Ok(lock)
    }
    fn current(&self,dir:&Dir,name:&str)->Result<(),&'static str>{
        let current=dir.open(OsStr::new(name),libc::O_RDONLY)?.ok_or(ERR)?.metadata().map_err(|_|ERR)?;
        let held=self.0.metadata().map_err(|_|ERR)?;
        if !private(&held)||!held.is_file()||held.len()!=0||held.dev()!=current.dev()||held.ino()!=current.ino(){return Err(ERR);}Ok(())
    }
}
impl Drop for Lock{fn drop(&mut self){unsafe{libc::flock(self.0.as_raw_fd(),libc::LOCK_UN);}}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub(super) enum Phase{Prepared,Marked,Moved,Committed,Restoring,Restored,Cleaned}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Payload{record:Record,tree:Vec<RetentionNode>,digest:String,restore_name:Option<String>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Plan{version:u32,id:String,provider:String,conversation_id:String,phase:Phase,items:Vec<Payload>}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Summary{pub id:String,pub provider:String,pub conversation_id:String,pub revision:String,pub phase:Phase,pub items:Vec<SummaryItem>}
#[derive(Clone,Serialize)]
pub(super) struct SummaryItem{pub id:String,pub name:String,pub kind:String,pub size:u64,pub entries:usize}
fn summary(plan:&Plan,bytes:&[u8])->Result<Summary,&'static str>{Ok(Summary{id:plan.id.clone(),provider:plan.provider.clone(),conversation_id:plan.conversation_id.clone(),revision:hash(&bytes)?,phase:plan.phase.clone(),items:plan.items.iter().map(|p|SummaryItem{id:p.record.id.clone(),name:p.record.name.clone(),kind:p.record.kind.clone(),size:p.record.size,entries:p.record.entries}).collect()})}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Inventory{pub recoveries:Vec<Summary>,pub unavailable:Vec<String>}
pub(super) fn inventory(inbox:&Path)->Result<Inventory,&'static str>{
    if std::fs::symlink_metadata(inbox).is_err_and(|e|e.kind()==std::io::ErrorKind::NotFound){return Ok(Inventory{recoveries:vec![],unavailable:vec![]});}
    let inbox=Dir::at(inbox)?;let Some(r)=root(&inbox,false)?else{return Ok(Inventory{recoveries:vec![],unavailable:vec![]})};let _catalog=Lock::named(&r,".catalog.lock")?;
    let deadline=Instant::now()+Duration::from_secs(8);let mut bytes=0usize;let mut recoveries=vec![];let mut unavailable=vec![];let names=r.names()?;
    if names.len()>260{return Err("retentionInventoryLimit");}
    let mut ids=std::collections::BTreeSet::new();
    for name in names{let name=name.to_str().ok_or(ERR)?;if matches!(name,".protocol.json"|".catalog.lock"){continue;}
        if let Some(id)=terminal::id(name){ids.insert(id.to_owned());}
        else if terminal::staging(name){unavailable.push(name.to_owned());}
        else if crate::assistant_memory::valid_id(name){ids.insert(name.to_owned());}else{return Err(ERR);}
    }
    for name in ids{if Instant::now()>=deadline{return Err("retentionInventoryLimit");}
        let read=(||{
            if r.exists(&terminal::name(&name))?{let tx=Terminal::load(&r,&name,true)?;bytes=bytes.checked_add(tx.byte_len()).filter(|n|*n<=32*1024*1024).ok_or("retentionInventoryLimit")?;return tx.summary();}
            let dir=r.child(&name)?;let data=dir.bytes("transaction.json",WAL_LIMIT)?;bytes=bytes.checked_add(data.len()).filter(|n|*n<=32*1024*1024).ok_or("retentionInventoryLimit")?;
            let plan:Plan=serde_json::from_slice(&data).map_err(|_|ERR)?;plan.validate()?;if plan.id!=name{return Err(ERR);}summary(&plan,&data)
        })();match read{Ok(s)=>recoveries.push(s),Err("retentionInventoryLimit")=>return Err("retentionInventoryLimit"),Err(_)=>unavailable.push(name)}
    }recoveries.sort_by(|a,b|a.id.cmp(&b.id));unavailable.sort();Ok(Inventory{recoveries,unavailable})
}
fn tree_digest(tree:&[RetentionNode])->Result<String,&'static str>{hash(&tree)}
fn tree_valid(tree:&[RetentionNode])->bool{
    if tree.is_empty()||tree.len()>257||!tree[0].path.is_empty(){return false;}
    let paths=tree.iter().map(|n|n.path.clone()).collect::<BTreeSet<_>>();
    paths.len()==tree.len()&&tree.iter().all(|n|n.valid()&&(n.path.is_empty()||{
        let path=Path::new(OsStr::from_bytes(&n.path));let parent=path.parent().unwrap_or(Path::new(""));
        tree.iter().any(|p|p.is_dir()&&p.path==parent.as_os_str().as_bytes())
    }))
}
impl Plan{
    fn validate(&self)->Result<(),&'static str>{
        owner(&self.provider,&self.conversation_id)?;if self.version!=1||!crate::assistant_memory::valid_id(&self.id)||!valid_records(&self.items.iter().map(|p|p.record.clone()).collect::<Vec<_>>())||
            self.items.iter().any(|p|!tree_valid(&p.tree)||tree_digest(&p.tree).ok().as_ref()!=Some(&p.digest)||p.restore_name.as_ref().is_some_and(|n|!component(n)||reserved(n))) {return Err(ERR);}Ok(())
    }
}
pub(super) struct Transaction{inbox:Dir,_root:Dir,dir:Dir,payload:Dir,_lock:Lock,plan:Plan,published:Option<Vec<u8>>,terminal_seen:bool,
    #[cfg(test)]save_number:usize,
    #[cfg(test)]fail_save:Option<usize>,
}
impl Transaction{
    pub(super) fn create(inbox:&Path,provider:&str,cid:&str,records:&[Record],copies:&[Copy])->Result<Self,&'static str>{
        owner(provider,cid)?;if !valid_records(records)||records.len()!=copies.len(){return Err(ERR);}
        let mut budget=Budget::new();let mut items=vec![];
        for (record,copy) in records.iter().zip(copies){if !copy.current(&mut budget){return Err(ERR);}items.push(Payload{record:record.clone(),tree:copy.retention_tree(),digest:copy.retention_digest(),restore_name:None});}
        let inbox=Dir::at(inbox)?;let r=root(&inbox,true)?.ok_or(ERR)?;let catalog=Lock::named(&r,".catalog.lock")?;if r.names()?.len()>=130{return Err("retentionInventoryLimit");}let id=crate::assistant_memory::new_id();r.mkdir(&id)?;let dir=r.child(&id)?;let lock=Lock::acquire(&dir)?;dir.mkdir("payload")?;let payload=dir.child("payload")?;
        let plan=Plan{version:1,id,provider:provider.into(),conversation_id:cid.into(),phase:Phase::Prepared,items};plan.validate()?;
        let mut tx=Self{inbox,_root:r,dir,payload,_lock:lock,plan,published:None,terminal_seen:false,#[cfg(test)]save_number:0,#[cfg(test)]fail_save:None};tx.save()?;drop(catalog);Ok(tx)
    }
    pub(super) fn open(inbox:&Path,id:&str)->Result<Self,&'static str>{
        if !crate::assistant_memory::valid_id(id){return Err(ERR);}let inbox=Dir::at(inbox)?;let r=root(&inbox,false)?.ok_or(ERR)?;let catalog=Lock::named(&r,".catalog.lock")?;if r.exists(&terminal::name(id))?{return Err("retentionMetadataOnly");}let dir=r.child(id)?;let lock=Lock::acquire(&dir)?;
        let bytes=dir.bytes("transaction.json",WAL_LIMIT)?;let plan:Plan=serde_json::from_slice(&bytes).map_err(|_|ERR)?;plan.validate()?;if plan.id!=id{return Err(ERR);}
        if !dir.exists("payload")?&&matches!(plan.phase,Phase::Cleaned|Phase::Restored){dir.mkdir("payload")?;}
        let payload=dir.child("payload")?;drop(catalog);Ok(Self{inbox,_root:r,dir,payload,_lock:lock,plan,published:Some(bytes),terminal_seen:false,#[cfg(test)]save_number:0,#[cfg(test)]fail_save:None})
    }
    pub(super) fn id(&self)->&str{&self.plan.id}
    #[cfg(test)]
    pub(super) fn fail_after_wal_publish(&mut self,number:usize){self.fail_save=Some(number);}
    #[cfg(test)]pub(super)fn fail_next_wal_publish(&mut self){self.fail_save=Some(self.save_number+1);}
    pub(super) fn phase(&self)->&Phase{&self.plan.phase}
    pub(super) fn records(&self)->Vec<Record>{self.plan.items.iter().map(|p|p.record.clone()).collect()}
    pub(super) fn abandon_prepared(&mut self)->Result<(),&'static str>{
        if self.plan.phase!=Phase::Prepared||!self.payload.names()?.is_empty(){return Err(ERR);}self.plan.phase=Phase::Restored;self.save()
    }
    pub(super) fn summary(&self)->Result<Summary,&'static str>{summary(&self.plan,self.published.as_ref().ok_or(ERR)?)}
    pub(super) fn current(&self)->Result<(),&'static str>{if self._root.exists(&terminal::name(self.id()))?{return Err("retentionMetadataOnly");}self.wal_current()}
    fn wal_current(&self)->Result<(),&'static str>{if Some(self.dir.bytes("transaction.json",WAL_LIMIT)?)!=self.published{return Err(ERR);}Ok(())}
    pub(super) fn source_files(&self)->Result<Vec<Attachment>,&'static str>{
        self.current()?;if matches!(self.plan.phase,Phase::Committed|Phase::Cleaned){return Ok(vec![]);}let mut budget=Budget::new();let mut sources=vec![];
        for item in &self.plan.items{
            if self.full(&self.payload,&item.record.id,item,&mut budget)?{continue;}
            let name=item.restore_name.as_deref().unwrap_or(&item.record.copy_name);if !self.full(&self.inbox,name,item,&mut budget)?{return Err(ERR);}
            sources.push(attachment(&self.plan.provider,&self.plan.conversation_id,&item.record,&self.inbox.path,name));
        }Ok(sources)
    }
    /// A durable root-level terminal record survives WAL/txdir removal. It
    /// grants metadata cleanup only, never renewed payload deletion/restore.
    pub(super) fn remove_metadata(&mut self)->Result<(),&'static str>{
        self.remove_metadata_using(|_|Ok(()))
    }
    fn remove_metadata_using(&mut self,step:impl FnMut(&'static str)->Result<(),&'static str>)->Result<(),&'static str>{terminal::finish(self,step)}
    #[cfg(test)]pub(super)fn interrupt_metadata(&mut self,at:&'static str)->Result<(),&'static str>{self.remove_metadata_using(|stage|if stage==at{Err("syntheticMetadataInterruption")}else{Ok(())})}

    fn save(&mut self)->Result<(),&'static str>{
        self.plan.validate()?;let bytes=serde_json::to_vec(&self.plan).map_err(|_|ERR)?;if bytes.len()>WAL_LIMIT{return Err(ERR);}
        match &self.published{Some(old)=>{if self.dir.bytes("transaction.json",WAL_LIMIT)?!=*old{return Err(ERR);}},None=>{if self.dir.open(OsStr::new("transaction.json"),libc::O_RDONLY)?.is_some(){return Err(ERR);}}}
        let temp=format!(".wal-{}.tmp",crate::assistant_memory::new_id());self.dir.create_bytes(&temp,&bytes)?;
        self.dir.rename(OsStr::new(&temp),&self.dir,OsStr::new("transaction.json"),self.published.is_none())?;
        // Even an uncertain directory sync may have published the new WAL.
        self.published=Some(bytes);self.dir.sync()?;
        #[cfg(test)]{self.save_number+=1;if self.fail_save==Some(self.save_number){return Err("syntheticPostPublicationSyncFailure");}}
        Ok(())
    }
    pub(super) fn marked(&mut self)->Result<(),&'static str>{if self.plan.phase!=Phase::Prepared{return Err(ERR);}self.plan.phase=Phase::Marked;self.save()}
    fn full(&self,dir:&Dir,name:&str,item:&Payload,budget:&mut Budget)->Result<bool,&'static str>{
        if dir.open(OsStr::new(name),libc::O_RDONLY)?.is_none(){return Ok(false);}
        let attachment=attachment(&self.plan.provider,&self.plan.conversation_id,&item.record,&dir.path,name);
        let copy=Copy::capture(&dir.path,&attachment,budget).map_err(|_|ERR)?;
        if copy.retention_digest()!=item.digest{return Err(ERR);}Ok(true)
    }
    pub(super) fn move_all(&mut self)->Result<(),&'static str>{self.move_using(|_|Ok(()))}
    fn move_using(&mut self,mut step:impl FnMut(usize)->Result<(),&'static str>)->Result<(),&'static str>{
        if !matches!(self.plan.phase,Phase::Marked|Phase::Moved){return Err(ERR);}let mut budget=Budget::new();
        for (index,item) in self.plan.items.iter().enumerate(){
            if self.full(&self.payload,&item.record.id,item,&mut budget)?{continue;}
            if !self.full(&self.inbox,&item.record.copy_name,item,&mut budget)?{return Err(ERR);}step(index)?;
            self.inbox.rename(OsStr::new(&item.record.copy_name),&self.payload,OsStr::new(&item.record.id),true)?;
            self.inbox.sync()?;self.payload.sync()?;if !self.full(&self.payload,&item.record.id,item,&mut budget)?{return Err(ERR);}
        }self.plan.phase=Phase::Moved;self.save()
    }
    /// Call only after the manifest's removal has been confirmed durable.
    pub(super) fn committed(&mut self)->Result<(),&'static str>{if self.plan.phase!=Phase::Moved{return Err(ERR);}self.plan.phase=Phase::Committed;self.save()}
    pub(super) fn restore_moves(&mut self,claimed:&BTreeSet<String>)->Result<BTreeMap<String,String>,&'static str>{
        self.restore_using(claimed,|_,_|Ok(()))
    }
    fn restore_using(&mut self,claimed:&BTreeSet<String>,mut step:impl FnMut(&'static str,usize)->Result<(),&'static str>)->Result<BTreeMap<String,String>,&'static str>{
        if matches!(self.plan.phase,Phase::Cleaned){return Err(ERR);}let mut budget=Budget::new();
        // All payloads must be complete before the first restore move. A
        // partly deleted committed copy cannot be described as restored.
        let mut locations=vec![];
        for item in &self.plan.items{
            if self.full(&self.payload,&item.record.id,item,&mut budget).map_err(|_|RESTORE_UNAVAILABLE)?{locations.push(None);continue;}
            let name=item.restore_name.as_deref().unwrap_or(&item.record.copy_name);
            if claimed.contains(name){return Err(RESTORE_CLAIMED);}if !self.full(&self.inbox,name,item,&mut budget).map_err(|_|RESTORE_UNAVAILABLE)?{return Err(RESTORE_UNAVAILABLE);}locations.push(Some(name.to_owned()));
        }
        for (i,location) in locations.iter().enumerate(){
            let item=&self.plan.items[i];let name=if let Some(name)=location{name.clone()}else{
                let original=item.restore_name.as_deref().unwrap_or(&item.record.copy_name);
                if !claimed.contains(original)&&!self.inbox.exists(original)?{original.to_owned()}else{
                    let name=format!("recuperado-{}",crate::assistant_memory::new_id());if claimed.contains(&name)||self.inbox.exists(&name)?{return Err(ERR);}name
                }
            };
            self.plan.items[i].restore_name=Some(name);
        }
        self.plan.phase=Phase::Restoring;self.save()?;step("restoreNamesPublished",0)?;
        for (i,location) in locations.iter().enumerate(){if location.is_none(){let item=&self.plan.items[i];let name=item.restore_name.as_ref().ok_or(ERR)?;
            self.payload.rename(OsStr::new(&item.record.id),&self.inbox,OsStr::new(name),true)?;step("restoreMoved",i)?;self.payload.sync()?;self.inbox.sync()?;step("restoreMoveDurable",i)?;
            if !self.full(&self.inbox,name,item,&mut budget)?{return Err(ERR);}
        }}
        Ok(self.plan.items.iter().map(|p|(p.record.id.clone(),p.restore_name.clone().expect("persisted restore name"))).collect())
    }
    #[cfg(test)]pub(super)fn interrupt_restore(&mut self,claimed:&BTreeSet<String>,at:&'static str,index:usize)->Result<BTreeMap<String,String>,&'static str>{self.restore_using(claimed,|stage,i|if stage==at&&i==index{Err("syntheticRestoreInterruption")}else{Ok(())})}
    #[cfg(test)]pub(super)fn interrupt_cleanup(&mut self,before:usize)->Result<(),&'static str>{self.cleanup_using(|removed|if removed==before{Err("syntheticCleanupInterruption")}else{Ok(())})}
    /// Call only after restored manifest metadata has been confirmed durable.
    pub(super) fn restored(&mut self)->Result<(),&'static str>{if self.plan.phase!=Phase::Restoring{return Err(ERR);}self.plan.phase=Phase::Restored;self.save()}
    pub(super) fn cleanup(&mut self)->Result<(),&'static str>{self.cleanup_using(|_|Ok(()))}
    fn cleanup_using(&mut self,mut step:impl FnMut(usize)->Result<(),&'static str>)->Result<(),&'static str>{
        if self.plan.phase!=Phase::Committed{return Err(ERR);}let mut budget=DeleteBudget{deadline:Instant::now()+Duration::from_secs(8),visits:0};let mut removed=0;
        // Check every existing node before deleting any of them. On restart,
        // missing expected nodes are allowed; new or changed nodes are kept.
        for item in &self.plan.items{inspect(&self.payload,OsStr::new(&item.record.id),Path::new(""),&item.tree,&mut budget,false,&mut removed,&mut step).map_err(|_|CLEANUP_UNVERIFIED)?;}
        for item in &self.plan.items{inspect(&self.payload,OsStr::new(&item.record.id),Path::new(""),&item.tree,&mut budget,true,&mut removed,&mut step).map_err(|_|CLEANUP_INTERRUPTED)?;}
        self.payload.sync()?;self.plan.phase=Phase::Cleaned;self.save()
    }
}
struct DeleteBudget{deadline:Instant,visits:usize}
fn inspect(dir:&Dir,name:&OsStr,relative:&Path,expected:&[RetentionNode],budget:&mut DeleteBudget,delete:bool,removed:&mut usize,step:&mut impl FnMut(usize)->Result<(),&'static str>)->Result<(),&'static str>{
    budget.visits+=1;if budget.visits>3*32*257||Instant::now()>=budget.deadline{return Err(ERR);}
    let Some(file)=dir.open(name,libc::O_RDONLY)?else{return Ok(())};let node=expected.iter().find(|n|n.path==relative.as_os_str().as_bytes()).ok_or(ERR)?;
    if !file.metadata().is_ok_and(|m|node.matches(&m,true)){return Err(ERR);}
    if node.is_dir(){let child=Dir::from(file,dir.path.join(name))?;
        for child_name in child.names()?{inspect(&child,&child_name,&relative.join(&child_name),expected,budget,delete,removed,step)?;}
    }
    if delete{
        step(*removed)?;let current=dir.open(name,libc::O_RDONLY)?.ok_or(ERR)?;if !current.metadata().is_ok_and(|m|node.matches(&m,true)){return Err(ERR);}
        dir.unlink(name,node.is_dir())?;*removed+=1;dir.sync()?;
    }Ok(())
}
#[cfg(test)]
#[path="purge_quarantine_tests.rs"]
mod tests;
