//! One atomic private envelope per goal. No executor, account or model calls.
use super::model::*;
use std::{ffi::CString,fs::{File,OpenOptions},io::{Read,Write},os::{fd::{AsRawFd,FromRawFd},unix::fs::{MetadataExt,OpenOptionsExt}},path::{Path,PathBuf}};
use ring::rand::{SecureRandom,SystemRandom};
#[path="store_plan_commit.rs"]mod plan_commit;
#[path="store_recovery.rs"]mod recovery;
#[path="store_retirement.rs"]mod retirement;
#[path="store_step_recovery.rs"]mod step_recovery;
#[path="store_restored_future.rs"]mod restored_future;
pub(in crate::pet_runtime)use plan_commit::{PlanPublished,PlanAbsent};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-goals\"}\n";
const MARKER:&str=".protocol.json";
const LOCK:&str=".catalog.lock";
const CONTROL_RESERVE:usize=16384;
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]pub(super)struct Stamp(u64,u64,u64,i64,i64,i64,i64,u32,u32,u64);
fn stamp(m:&std::fs::Metadata)->Stamp{Stamp(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink())}
fn private_file(m:&std::fs::Metadata)->bool{m.is_file()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0&&m.nlink()==1}
fn name(s:&str)->Result<CString>{if s.is_empty()||s.contains('/')||s=="."||s==".."{return Err(Error::UnsafePath);}CString::new(s).map_err(|_|Error::UnsafePath)}
pub fn nonce()->Result<String>{let mut b=[0u8;16];SystemRandom::new().fill(&mut b).map_err(|_|Error::EntropyUnavailable)?;Ok(b.iter().map(|b|format!("{b:02x}")).collect())}
pub(super)struct Dir{pub(super)path:PathBuf,pub(super)file:File,dev:u64,ino:u64,private:bool}
impl Dir{
    pub(super)fn open(path:&Path,private:bool)->Result<Option<Self>>{
        let file=match OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path){Ok(f)=>f,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(_)=>return Err(Error::UnsafePath)};
        let m=file.metadata().map_err(|_|Error::StorageUnavailable)?;let dir=Self{path:path.into(),file,dev:m.dev(),ino:m.ino(),private};dir.check()?;Ok(Some(dir))
    }
    pub(super)fn check(&self)->Result<()>{
        let m=std::fs::symlink_metadata(&self.path).map_err(|_|Error::UnsafePath)?;
        if !m.is_dir()||m.uid()!=unsafe{libc::geteuid()}||m.mode()&if self.private{0o077}else{0o022}!=0||m.dev()!=self.dev||m.ino()!=self.ino{return Err(Error::UnsafePath);}Ok(())
    }
    pub(super)fn open_file(&self,s:&str,flags:i32)->Result<Option<File>>{
        self.check()?;let s=name(s)?;
        let fd=unsafe{libc::openat(self.file.as_raw_fd(),s.as_ptr(),flags|libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK,0o600)};
        if fd<0{return if std::io::Error::last_os_error().kind()==std::io::ErrorKind::NotFound{Ok(None)}else{Err(Error::UnsafePath)};}
        let f=unsafe{File::from_raw_fd(fd)};if !f.metadata().is_ok_and(|m|private_file(&m)){return Err(Error::UnsafePath);}Ok(Some(f))
    }
    pub(super)fn bytes(&self,s:&str,limit:usize)->Result<Option<(Vec<u8>,Stamp)>>{
        let Some(f)=self.open_file(s,libc::O_RDONLY)?else{return Ok(None);};let before=f.metadata().map_err(|_|Error::StorageUnavailable)?;
        if before.len()>limit as u64{return Err(Error::LimitReached);}let mut bytes=vec![];(&f).take(limit as u64+1).read_to_end(&mut bytes).map_err(|_|Error::StorageUnavailable)?;
        let after=f.metadata().map_err(|_|Error::StorageUnavailable)?;let current=self.open_file(s,libc::O_RDONLY)?.ok_or(Error::Conflict)?.metadata().map_err(|_|Error::StorageUnavailable)?;
        if bytes.len()>limit||stamp(&before)!=stamp(&after)||stamp(&before)!=stamp(&current){return Err(Error::Conflict);}Ok(Some((bytes,stamp(&before))))
    }
    pub(super)fn has_stamp(&self,name:&str,expected:&Stamp)->Result<bool>{let Some(file)=self.open_file(name,libc::O_RDONLY)?else{return Ok(false);};Ok(stamp(&file.metadata().map_err(|_|Error::StorageUnavailable)?)==*expected)}
    pub(super)fn sync(&self)->Result<()>{self.check()?;self.file.sync_all().map_err(|_|Error::StorageUnavailable)}
    pub(super)fn durable(&self)->Result<()>{
        self.sync()?;
        // Confirm a root entry that may come from an uncertain earlier setup.
        Dir::open(self.path.parent().ok_or(Error::UnsafePath)?,false)?.ok_or(Error::StorageUnavailable)?.sync()
    }
    pub(super)fn names(&self)->Result<Vec<String>>{
        self.check()?;let mut names=vec![];
        for entry in std::fs::read_dir(format!("/proc/self/fd/{}",self.file.as_raw_fd())).map_err(|_|Error::StorageUnavailable)?{
            if names.len()>=4096{return Err(Error::LimitReached);}let entry=entry.map_err(|_|Error::StorageUnavailable)?;names.push(entry.file_name().to_str().ok_or(Error::UnsafePath)?.to_owned());
        }Ok(names)
    }
    pub(super)fn rename(&self,from:&str,to:&Dir,target:&str,new:bool)->Result<()>{
        self.check()?;to.check()?;let from=name(from)?;let target=name(target)?;
        if unsafe{libc::renameat2(self.file.as_raw_fd(),from.as_ptr(),to.file.as_raw_fd(),target.as_ptr(),if new{libc::RENAME_NOREPLACE}else{0})}!=0{return Err(Error::StorageUnavailable);}Ok(())
    }
    pub(super)fn write_new(&self,s:&str,bytes:&[u8])->Result<File>{
        let mut f=self.open_file(s,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL)?.ok_or(Error::StorageUnavailable)?;
        f.write_all(bytes).and_then(|_|f.sync_all()).map_err(|_|Error::StorageUnavailable)?;Ok(f)
    }
    /// Remove only the exact native temporary inode observed under its catalog
    /// lock. The caller authenticates its bytes and confirms directory durability.
    pub(super)fn remove_exact(&self,s:&str,expected:&Stamp)->Result<()>{
        self.check()?;let file=self.open_file(s,libc::O_RDONLY)?.ok_or(Error::Conflict)?;
        if &stamp(&file.metadata().map_err(|_|Error::StorageUnavailable)?)!=expected{return Err(Error::Conflict);}
        let name=name(s)?;if unsafe{libc::unlinkat(self.file.as_raw_fd(),name.as_ptr(),0)}!=0{return Err(Error::StorageUnavailable);}self.check()
    }
    fn remove_own(&self,s:&str,f:&File){
        if !self.check().is_ok(){return;}let Ok(Some(current))=self.open_file(s,libc::O_RDONLY)else{return;};
        let (Ok(a),Ok(b))=(f.metadata(),current.metadata())else{return;};if a.dev()!=b.dev()||a.ino()!=b.ino(){return;}
        if let Ok(s)=name(s){unsafe{libc::unlinkat(self.file.as_raw_fd(),s.as_ptr(),0)};}
    }
}
pub(super)fn protocol_directory(path:&Path,protocol:&[u8],create:bool)->Result<Option<Dir>>{
    if let Some(dir)=Dir::open(path,true)?{
        if dir.bytes(MARKER,1024)?.is_none_or(|(b,_)|b!=protocol){return Err(Error::InvalidRecord);}return Ok(Some(dir));
    }
    if !create{return Ok(None);}
    let parent=Dir::open(path.parent().ok_or(Error::UnsafePath)?,false)?.ok_or(Error::StorageUnavailable)?;
    let final_name=path.file_name().and_then(|s|s.to_str()).ok_or(Error::UnsafePath)?;
    let setup=format!(".coucou-runtime-setup-{}",nonce()?);let c=name(&setup)?;
    if unsafe{libc::mkdirat(parent.file.as_raw_fd(),c.as_ptr(),0o700)}!=0{return Err(Error::StorageUnavailable);}
    let staged=Dir::open(&parent.path.join(&setup),true)?.ok_or(Error::StorageUnavailable)?;
    staged.write_new(MARKER,protocol)?;staged.write_new(LOCK,b"")?;staged.sync()?;
    if parent.rename(&setup,&parent,final_name,true).is_err(){return protocol_directory(path,protocol,false)?.ok_or(Error::StorageUnavailable).map(Some);}
    parent.sync()?;protocol_directory(path,protocol,false)
}
pub(super)struct Guard(pub(super)File);
impl Drop for Guard{fn drop(&mut self){unsafe{libc::flock(self.0.as_raw_fd(),libc::LOCK_UN);}}}
#[cfg(test)]#[derive(Clone,Copy,Default)]pub(super)enum Fault{#[default]None,BeforePublish,AfterPublish}
pub struct Store{dir:Dir,authority:Option<PathBuf>,#[cfg(test)]pub(super)fault:Fault}
impl super::Repository for Store{
    fn read(&self,id:&str,pet_id:&str)->Result<Record>{Store::read(self,id,pet_id)}
    fn read_control(&self,id:&str,pet_id:&str)->Result<Record>{Store::read_control(self,id,pet_id)}
    fn create(&self,spec:Spec,now:u64)->Result<Receipt>{Store::create(self,spec,now)}
    fn mutate(&self,id:&str,pet_id:&str,revision:&str,request:String,action:Action,now:u64)->Result<Receipt>{Store::mutate(self,id,pet_id,revision,request,action,now)}
    fn list(&self,pet_id:Option<&str>)->Result<Listing>{Store::list(self,pet_id)}
}
impl Store{
    #[cfg(test)]pub(super)fn test_lock(&self,write:bool)->Result<Guard>{self.lock(write)}
    pub fn existing(path:&Path)->Result<Option<Self>>{Ok(protocol_directory(path,PROTOCOL,false)?.map(|dir|Self{dir,authority:None,#[cfg(test)]fault:Fault::None}))}
    pub fn ensure(path:&Path)->Result<Self>{Ok(Self{dir:protocol_directory(path,PROTOCOL,true)?.ok_or(Error::StorageUnavailable)?,authority:None,#[cfg(test)]fault:Fault::None})}
    /// Bind the configured authority root explicitly; never take a key path
    /// from a stored goal or completion reference.
    pub(super)fn existing_verified(path:&Path,authority:&Path)->Result<Option<Self>>{Ok(Self::existing(path)?.map(|mut store|{store.authority=Some(authority.into());store}))}
    pub(super)fn existing_native(path:&Path)->Result<Option<Self>>{Self::existing_verified(path,&path.parent().ok_or(Error::UnsafePath)?.join("pet-runtime-consent-v1"))}
    pub(super)fn ensure_native(path:&Path)->Result<Self>{let mut store=Self::ensure(path)?;store.authority=Some(path.parent().ok_or(Error::UnsafePath)?.join("pet-runtime-consent-v1"));Ok(store)}
    fn lock(&self,write:bool)->Result<Guard>{
        let f=self.dir.open_file(LOCK,libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
        if unsafe{libc::flock(f.as_raw_fd(),(if write{libc::LOCK_EX}else{libc::LOCK_SH})|libc::LOCK_NB)}!=0{return Err(Error::Busy);}
        let guard=Guard(f);let current=self.dir.open_file(LOCK,libc::O_RDONLY)?.ok_or(Error::UnsafePath)?;
        let a=guard.0.metadata().map_err(|_|Error::UnsafePath)?;let b=current.metadata().map_err(|_|Error::UnsafePath)?;
        if a.dev()!=b.dev()||a.ino()!=b.ino(){return Err(Error::UnsafePath);}Ok(guard)
    }
    fn load(&self,id:&str)->Result<Option<(Record,Stamp)>>{self.load_with_pending(id,false)}
    fn proof_error(error:super::authority::Failure)->Error{match error{super::authority::Failure::Busy=>Error::Busy,super::authority::Failure::Unavailable=>Error::StorageUnavailable,_=>Error::InvalidRecord}}
    fn load_with_pending(&self,id:&str,allow_pending:bool)->Result<Option<(Record,Stamp)>>{
        if !goal_id(id){return Err(Error::InvalidInput);}let Some((bytes,stamp))=self.dir.bytes(&format!("{id}.json"),MAX_BYTES)?else{return Ok(None);};
        let record:Record=serde_json::from_slice(&bytes).map_err(|_|Error::InvalidRecord)?;record.validate()?;
        if let Some(path)=&self.authority{if let Ok(a)=super::authority::Authority::existing(path){if super::restoration::matches_historical_goal(&a,&record).map_err(Self::proof_error)?{if record.id!=id{return Err(Error::InvalidRecord);}return Ok(Some((record,stamp)));}}}
        if record.state==State::Completed{if let Some(path)=&self.authority{super::authority::Authority::existing(path).and_then(|a|a.verify_completed_goal(&record)).map_err(Self::proof_error)?;}}
        if let Some(path)=&self.authority {
            match super::authority::Authority::existing(path) {
                Ok(a)=>{a.verify_restored_origin(&record).map_err(Self::proof_error)?;a.verify_continuations_goal(&record).map_err(Self::proof_error)?;a.verify_generated_goal(&record,allow_pending).map_err(Self::proof_error)?;a.verify_recovered_goal(&record).map_err(Self::proof_error)?;},
                Err(super::authority::Failure::Missing) if record.spec.restored_from.is_none()&&record.generated_plans.is_empty()&&!record.admissions.iter().any(|r|r.spec.schema_version==3||r.spec.continuation.is_some())=>{},
                Err(error)=>return Err(Self::proof_error(error)),
            }
        }
        if record.id!=id{return Err(Error::InvalidRecord);}Ok(Some((record,stamp)))
    }
    fn stabilize(&self,record:Record)->Receipt{
        let good=self.dir.open_file(&format!("{}.json",record.id),libc::O_RDONLY).ok().flatten().is_some_and(|f|f.sync_all().is_ok())&&self.dir.durable().is_ok();
        Receipt{record,status:if good{CommitStatus::Unchanged}else{CommitStatus::Uncertain}}
    }
    fn publish(&self,record:Record,before:Option<&Stamp>,control:bool)->Result<Receipt>{
        self.publish_current(record,before,control,||Ok(()))
    }
    fn publish_current(&self,record:Record,before:Option<&Stamp>,control:bool,current_guard:impl Fn()->Result<()>)->Result<Receipt>{
        current_guard()?;
        record.validate()?;let bytes=serde_json::to_vec(&record).map_err(|_|Error::InvalidRecord)?;
        if bytes.len()>if control{MAX_BYTES}else{MAX_BYTES-CONTROL_RESERVE}{return Err(Error::LimitReached);}
        let target=format!("{}.json",record.id);let temp=format!(".tmp-{}",nonce()?);let file=self.dir.write_new(&temp,&bytes)?;
        if let Err(error)=current_guard(){self.dir.remove_own(&temp,&file);return Err(error);}
        let current=self.dir.bytes(&target,MAX_BYTES)?;
        if current.as_ref().map(|(_,s)|s)!=before{self.dir.remove_own(&temp,&file);return Err(Error::Conflict);}
        #[cfg(test)]if matches!(self.fault,Fault::BeforePublish){self.dir.remove_own(&temp,&file);return Err(Error::StorageUnavailable);}
        if let Err(error)=current_guard(){self.dir.remove_own(&temp,&file);return Err(error);}
        if self.dir.rename(&temp,&self.dir,&target,before.is_none()).is_err(){
            if self.load(&record.id).ok().flatten().is_some_and(|(r,_)|r.revision==record.revision){return Ok(Receipt{record,status:CommitStatus::Uncertain});}
            // Preserve the candidate on ambiguous publication; it is not replayed.
            return Err(Error::StorageUnavailable);
        }
        #[cfg(test)]if matches!(self.fault,Fault::AfterPublish){return Ok(Receipt{record,status:CommitStatus::Uncertain});}
        let status=if self.dir.durable().is_ok()&&current_guard().is_ok(){CommitStatus::Durable}else{CommitStatus::Uncertain};Ok(Receipt{record,status})
    }
    pub fn read(&self,id:&str,pet_id:&str)->Result<Record>{
        let _guard=self.lock(false)?;let(record,_)=self.load(id)?.ok_or(Error::NotFound)?;
        if record.spec.owner.pet_id!=pet_id{return Err(Error::Conflict);}Ok(record)
    }
    fn historical_source(&self,record:&Record)->Result<bool>{
        let Some(path)=&self.authority else{return Ok(false);};match super::authority::Authority::existing(path){Ok(a)=>super::restoration::matches_historical_goal(&a,record).map_err(Self::proof_error),Err(super::authority::Failure::Missing)=>Ok(false),Err(e)=>Err(Self::proof_error(e))}
    }
    fn require_current_goal(&self,record:&Record)->Result<()>{if self.historical_source(record)?{return Err(Error::Conflict);}Ok(())}
    /// Keep the goal reader while the authority reader inspects existing
    /// receipts. Unlike completion certification, this never fsyncs/publishes.
    pub(super)fn reconciliation(&self,path:&Path,id:&str,pet:&str,revision:&str,now:u64)->Result<super::reconciliation::Snapshot>{
        let _guard=self.lock(false)?;if !goal_id(id)||!token(revision){return Err(Error::InvalidInput);}
        let(raw,_)=self.dir.bytes(&format!("{id}.json"),MAX_BYTES)?.ok_or(Error::NotFound)?;let goal:Record=serde_json::from_slice(&raw).map_err(|_|Error::InvalidRecord)?;goal.validate()?;
        if goal.id!=id||goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}if now<goal.updated_at||now>8_640_000_000_000_000{return Err(Error::ClockRollback);}
        let authority=match super::authority::Authority::existing(path){Ok(a)=>a,Err(super::authority::Failure::Missing)if goal.admissions.is_empty()=>return super::reconciliation::Snapshot::empty(&goal,now),Err(e)=>return Err(e.control_error())};
        authority.reconciliation(&goal,now).map_err(super::authority::Failure::control_error)
    }
    /// Read the full original history under the same goal/authority lock order.
    pub(super)fn history(&self,path:&Path,id:&str,pet:&str,revision:&str,now:u64)->Result<super::history::Snapshot>{
        let _guard=self.lock(false)?;if !goal_id(id)||!token(revision)||!key(pet){return Err(Error::InvalidInput);}
        let(raw,_)=self.dir.bytes(&format!("{id}.json"),MAX_BYTES)?.ok_or(Error::NotFound)?;let goal:Record=serde_json::from_slice(&raw).map_err(|_|Error::InvalidRecord)?;goal.validate()?;
        if goal.id!=id||goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}if now<goal.updated_at||now>8_640_000_000_000_000{return Err(Error::ClockRollback);}
        let authority=match super::authority::Authority::existing(path){Ok(a)=>a,Err(super::authority::Failure::Missing)if goal.admissions.is_empty()=>return super::history::Snapshot::empty(&goal,now),Err(e)=>return Err(e.control_error())};
        authority.history(&goal,now).map_err(super::authority::Failure::control_error)
    }
    pub fn create(&self,spec:Spec,now:u64)->Result<Receipt>{
        spec.validate()?;if spec.restored_from.is_some(){return Err(Error::InvalidInput);}let _guard=self.lock(true)?;let id=id_for(&spec);
        if let Some((record,_))=self.load(&id)?{if record.spec!=spec{return Err(Error::Conflict);}return Ok(self.stabilize(record));}
        if spec.request_id.starts_with("rf-"){return Err(Error::InvalidInput);}
        let count=self.dir.names()?.iter().filter(|n|n.strip_suffix(".json").is_some_and(goal_id)).count();if count>=MAX_GOALS{return Err(Error::LimitReached);}
        self.publish(Record::new(spec,now,nonce()?)?,None,false)
    }
    pub fn mutate(&self,id:&str,pet_id:&str,revision:&str,request:String,action:Action,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let(mut record,before)=self.load_with_pending(id,matches!(&action,Action::Pause|Action::Cancel))?.ok_or(Error::NotFound)?;
        self.require_current_goal(&record)?;
        if record.spec.owner.pet_id!=pet_id{return Err(Error::Conflict);}
        if record.replay(&request,&action)?{return Ok(self.stabilize(record));}
        if let Some(path)=&self.authority{match super::authority::Authority::existing(path){Ok(a)=>if a.verify_recovered_goal(&record).map_err(|e|e.control_error())?{return Err(Error::Busy);},Err(super::authority::Failure::Missing)if !record.events.iter().any(|e|matches!(e.kind,EventKind::LostOwnerClosed|EventKind::UnclaimedOwnerClosed))=>{},Err(e)=>return Err(e.control_error())}}
        if record.revision!=revision{return Err(Error::Conflict);}let control=matches!(action,Action::Cancel);
        record.apply(request,action,now,nonce()?)?;self.publish(record,Some(&before),control)
    }
    pub(super)fn complete_certified(&self,certificate:&super::authority::CompletionCertified<'_>,now:u64,current:impl Fn(&Record)->Result<()>)->Result<Receipt>{
        let reference=certificate.reference();let _guard=self.lock(true)?;
        let(mut goal,before)=self.load(&reference.goal_id)?.ok_or(Error::NotFound)?;
        certificate.current(&goal,now).map_err(|reason|if reason==super::authority::Failure::Busy{Error::Busy}else{Error::Conflict})?;
        if goal.state==State::Completed{
            let receipt=self.stabilize(goal);
            // This is an ACK/durability readback, after stabilization. Never
            // classify an unavailable readback here as prepublication Busy.
            if self.load(&reference.goal_id).map_err(|e|if e==Error::Busy{Error::StorageUnavailable}else{e})?.is_none_or(|(r,_)|r.completion.as_ref()!=Some(reference)||r.revision!=reference.target_revision){return Err(Error::Conflict);}
            return Ok(receipt);
        }
        current(&goal)?;let source=goal.clone();
        goal.complete_native(reference.clone(),now)?;
        current(&source)?;self.publish(goal,Some(&before),true)
    }
    #[cfg(test)]pub(super)fn enqueue(&self,approved:&super::admission::Approved,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let a=approved.spec();let(mut record,before)=self.load(&a.goal_id)?.ok_or(Error::NotFound)?;
        if record.spec.owner!=a.owner{return Err(Error::Conflict);}
        if let Some(event)=record.events.iter().find(|e|e.request_id==a.request_id){
            if !matches!(event.kind,EventKind::AdmissionQueued)||event.fingerprint!=fingerprint(a)?{return Err(Error::Conflict);}return Ok(self.stabilize(record));
        }
        record.enqueue(approved,None,now,nonce()?)?;self.publish(record,Some(&before),false)
    }
    fn admission_ack_error(error:super::authority::Failure)->Error{if error==super::authority::Failure::Uncertain{Error::StorageUnavailable}else{Self::proof_error(error)}}
    /// A native retained approval can acknowledge its own published event even
    /// after expiry. No consent is issued and no work is dispatched here.
    pub(super)fn admission_ack(&self,authority:&super::authority::Authority,approved:&super::admission::Approved)->Result<Option<Receipt>>{
        let _guard=self.lock(true)?;let spec=approved.spec();let(record,_)=self.load_with_pending(&spec.goal_id,true)?.ok_or(Error::NotFound)?;
        if record.spec.owner!=spec.owner{return Err(Error::Conflict);}
        let Some(event)=record.events.iter().find(|e|e.request_id==spec.request_id)else{return Ok(None);};
        if !matches!(event.kind,EventKind::AdmissionQueued)||event.fingerprint!=fingerprint(spec)?{return Err(Error::Conflict);}
        authority.acknowledge_admission(approved,&record).map_err(Self::admission_ack_error)?;
        Ok(Some(self.stabilize(record)))
    }
    /// Serializes an unqueued withdrawal with every publication of this goal.
    /// A previously committed admission is returned truthfully, never undone.
    pub(super)fn withdraw_admission(&self,authority:&super::authority::Authority,approved:&super::admission::Approved,now:u64)->Result<Option<Receipt>>{
        let _guard=self.lock(true)?;let spec=approved.spec();let(record,_)=self.load_with_pending(&spec.goal_id,true)?.ok_or(Error::NotFound)?;
        if record.spec.owner!=spec.owner{return Err(Error::Conflict);}
        if let Some(event)=record.events.iter().find(|e|e.request_id==spec.request_id){
            if !matches!(event.kind,EventKind::AdmissionQueued)||event.fingerprint!=fingerprint(spec)?{return Err(Error::Conflict);}
            authority.acknowledge_admission(approved,&record).map_err(Self::admission_ack_error)?;return Ok(Some(self.stabilize(record)));
        }
        authority.withdraw_unqueued_admission(approved,now).map_err(Self::admission_ack_error)?;Ok(None)
    }
    pub(super)fn enqueue_certified(&self,certified:&super::authority::Certified<'_>,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let approved=certified.approved();let a=approved.spec();let(mut record,before)=self.load(&a.goal_id)?.ok_or(Error::NotFound)?;
        if record.spec.owner!=a.owner{return Err(Error::Conflict);}
        if let Some(event)=record.events.iter().find(|e|e.request_id==a.request_id){
            if !matches!(event.kind,EventKind::AdmissionQueued)||event.fingerprint!=fingerprint(a)?{return Err(Error::Conflict);}return Ok(self.stabilize(record));
        }
        certified.current(&record,now)?;record.enqueue(approved,Some(certified.proof().clone()),now,nonce()?)?;self.publish(record,Some(&before),false)
    }
    pub(super)fn claim_owned(&self,authority:&super::authority::Authority,lease:&super::leases::Lease,owner:&super::native_owner::OwnerLease,evidence:&super::admission::Evidence,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let(mut goal,before)=self.load(&lease.goal_id)?.ok_or(Error::NotFound)?;
        evidence.for_claim(&goal,now).map_err(|_|Error::Conflict)?;
        let row=goal.admissions.last().ok_or(Error::InvalidRecord)?;let proof=row.proof.as_ref().ok_or(Error::Conflict)?;
        if row.lease.as_ref()==Some(lease)&&goal.state==State::Leased{
            authority.claim(&row.spec,proof,lease,owner,now).map_err(super::authority::Failure::control_error)?;return Ok(self.stabilize(goal));
        }
        if goal.revision!=lease.source_revision{return Err(Error::Conflict);}
        // Validate the complete candidate before the independent ledger claim.
        let mut candidate=goal.clone();candidate.claim(lease.clone(),now,nonce()?)?;
        authority.claim(&row.spec,proof,lease,owner,now).map_err(super::authority::Failure::control_error)?;
        goal=candidate;self.publish(goal,Some(&before),false)
    }
    pub(super)fn drain_owned(&self,authority:&super::authority::Authority,drain:&super::lease_controller::DrainProof,now:u64)->Result<Receipt>{
        let _guard=self.lock(true)?;let(mut goal,before)=self.load_with_pending(&drain.lease().goal_id,true)?.ok_or(Error::NotFound)?;
        if !drain.original_goal(&goal){return Err(Error::Conflict);}drain.original_authority(authority)?;
        let row=goal.admissions.last().ok_or(Error::InvalidRecord)?;let proof=row.proof.as_ref().ok_or(Error::Conflict)?;
        let unclaimed=drain.unstarted()&&row.lease.is_none()&&matches!(goal.state,State::Queued|State::Paused|State::Cancelled);
        if row.lease.as_ref()!=Some(drain.lease())&&!unclaimed{return Err(Error::Conflict);}
        let expected=if goal.state==State::Cancelled{Some(super::leases::StopReason::Cancel)}else if unclaimed{match goal.state{State::Paused=>Some(super::leases::StopReason::Pause),_ if drain.deadline_elapsed(now)=>Some(super::leases::StopReason::Expired),_=>None}}else{row.stop_reason};
        let reason=authority.finish(&row.spec,proof,drain.lease(),drain.unstarted(),expected,now).map_err(super::authority::Failure::control_error)?;
        if unclaimed{
            goal.abandoned(drain.lease(),reason,now,nonce()?,drain.request_id().into())?;return self.publish(goal,Some(&before),true);
        }
        if !matches!(goal.state,State::Leased|State::Stopping){return Ok(self.stabilize(goal));}
        if row.stop_reason!=reason{return Err(Error::Conflict);}
        goal.drained(drain.lease(),drain.confirmed(),now,nonce()?,drain.request_id().into())?;self.publish(goal,Some(&before),true)
    }
    pub fn list(&self,pet_id:Option<&str>)->Result<Listing>{
        let _guard=self.lock(false)?;let mut goals=vec![];let mut unavailable=0;let mut pending=0;let mut seen=0;
        for n in self.dir.names()?{
            if matches!(n.as_str(),MARKER|LOCK){continue;}
            let Some(id)=n.strip_suffix(".json").filter(|s|goal_id(s))else{pending+=1;continue;};
            seen+=1;if seen>MAX_GOALS{unavailable+=1;continue;}
            match self.load(id){Ok(Some((r,_)))=>{if pet_id.is_none_or(|p|p==r.spec.owner.pet_id){match self.historical_source(&r){Ok(historical)=>{let mut summary=r.summary();summary.historical_source=historical;goals.push(summary);},Err(_)=>unavailable+=1}}},_=>unavailable+=1}
        }
        goals.sort_by(|a,b|b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));
        Ok(Listing{schema_version:1,goals,unavailable,pending_artifacts:pending,complete:unavailable==0&&pending==0})
    }
}
