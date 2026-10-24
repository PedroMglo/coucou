//! One mutation queue for the original copied folder set. Hot callbacks only
//! read this queue/root stamps; full metadata/content hashes run at IO/commit
//! boundaries. inotify alone cannot cover mmap or remote filesystem changes.
use super::*;
use super::copy_review::{Budget,Copy};
use std::{ffi::CString,fs::File,io::Read,os::fd::{AsRawFd,FromRawFd},path::PathBuf,sync::atomic::{AtomicBool,Ordering}};
const INVALID:&str="O conteúdo de uma pasta mudou ou a vigilância ficou indisponível. O resultado foi descartado.";
const MASK:u32=libc::IN_MODIFY|libc::IN_ATTRIB|libc::IN_CLOSE_WRITE|libc::IN_CREATE|libc::IN_DELETE|libc::IN_MOVED_FROM|libc::IN_MOVED_TO|libc::IN_DELETE_SELF|libc::IN_MOVE_SELF|libc::IN_UNMOUNT|libc::IN_MASK_CREATE;
struct Monitor{file:Mutex<File>,revoked:AtomicBool}
impl Monitor{
    fn new()->Result<Self,String>{
        let fd=unsafe{libc::inotify_init1(libc::IN_NONBLOCK|libc::IN_CLOEXEC)};
        if fd<0{return Err("Não foi possível vigiar estas pastas. Fecha pedidos antigos ou escolhe menos anexos.".into());}
        Ok(Self{file:Mutex::new(unsafe{File::from_raw_fd(fd)}),revoked:AtomicBool::new(false)})
    }
    fn add(&mut self,file:&File)->Result<(),String>{
        // Follow only our own proc-fd alias, after openat(O_NOFOLLOW) and the
        // private-copy stamp checks; no model-supplied path enters this API.
        let path=CString::new(format!("/proc/self/fd/{}",file.as_raw_fd())).unwrap();
        let fd=self.file.get_mut().map_err(|_|INVALID)?.as_raw_fd();
        if unsafe{libc::inotify_add_watch(fd,path.as_ptr(),MASK)}<0{return Err("Não foi possível vigiar todos os elementos da pasta. Escolhe uma seleção menor.".into());}Ok(())
    }
    fn revoke(&self)->bool{self.revoked.store(true,Ordering::Release);false}
    fn current(&self)->bool{
        let Ok(mut file)=self.file.lock()else{return self.revoke()};
        if self.revoked.load(Ordering::Acquire){return false;}
        let mut buffer=[0u8;4096];
        for _ in 0..4{
            match file.read(&mut buffer){
                Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>return true,
                Err(e)if e.kind()==std::io::ErrorKind::Interrupted=>continue,
                // Every subscribed event revokes this immutable turn. This
                // includes ignored watches, overflow, unmount and malformed IO.
                _=>return self.revoke(),
            }
        }self.revoke()
    }
}
struct ReadBudget{deadline:Instant,bytes:u64,cancel:Arc<AtomicBool>}
impl ReadBudget{
    fn check(&self)->Result<(),String>{if Instant::now()>=self.deadline||self.cancel.load(Ordering::Acquire){Err(INVALID.into())}else{Ok(())}}
    fn hash(&mut self,mut file:&File)->Result<[u8;32],String>{
        self.check()?;let meta=file.metadata().map_err(|_|INVALID)?;
        if !meta.is_file()||meta.len()>MAX_CONVERSATION_BYTES.saturating_sub(self.bytes){return Err(INVALID.into());}
        let mut hash=ring::digest::Context::new(&ring::digest::SHA256);let mut buffer=[0u8;64*1024];
        loop{
            self.check()?;let cap=(MAX_CONVERSATION_BYTES.saturating_sub(self.bytes)+1).min(buffer.len()as u64)as usize;
            let n=file.read(&mut buffer[..cap]).map_err(|_|INVALID)?;if n==0{break;}
            self.bytes+=n as u64;if self.bytes>MAX_CONVERSATION_BYTES{return Err(INVALID.into());}hash.update(&buffer[..n]);
        }
        let mut result=[0;32];result.copy_from_slice(hash.finish().as_ref());Ok(result)
    }
}
struct Folder{copy:Copy,hashes:BTreeMap<PathBuf,[u8;32]>}
pub(super) struct Folders{folders:Vec<Folder>,monitor:Monitor,cancel:Arc<AtomicBool>}
impl Folders{
    pub(super) fn capture(inbox:&Path,attachments:&[Attachment],deadline:Instant,cancel:Arc<AtomicBool>)->Result<Option<Self>,String>{
        Self::capture_using(inbox,attachments,deadline,cancel,|_|{})
    }
    fn capture_using(inbox:&Path,attachments:&[Attachment],deadline:Instant,cancel:Arc<AtomicBool>,mut after_snapshot:impl FnMut(&Attachment))->Result<Option<Self>,String>{
        let selected=attachments.iter().filter(|a|a.file.kind=="folder").collect::<Vec<_>>();
        if selected.is_empty(){return Ok(None);}
        if selected.len()>MAX_CONVERSATION_ATTACHMENTS{return Err(INVALID.into());}
        let(mut budget,mut reads)=(Budget::for_request(deadline),ReadBudget{deadline,bytes:0,cancel:cancel.clone()});
        let mut monitor=Monitor::new()?;let mut folders=Vec::new();
        for attachment in selected{
            reads.check()?;let copy=Copy::capture(inbox,attachment,&mut budget)?;let mut hashes=BTreeMap::new();after_snapshot(attachment);
            copy.visit(&mut budget,|file,path|{
                reads.check()?;monitor.add(file)?;
                if file.metadata().map_err(|_|INVALID)?.is_file(){hashes.insert(path.to_path_buf(),reads.hash(file)?);}Ok(())
            })?;folders.push(Folder{copy,hashes});
        }
        // Close the setup window: all watches must be armed before the final
        // metadata pass and queue check. Never silently recapture a changed set.
        if !monitor.current()||!folders.iter().all(|f|f.copy.current(&mut budget))||!monitor.current(){return Err(INVALID.into());}
        Ok(Some(Self{folders,monitor,cancel}))
    }
    pub(super) fn current(&self)->bool{
        if self.cancel.load(Ordering::Acquire)||!self.monitor.current(){return false;}
        if !self.folders.iter().all(|f|f.copy.root_current()){return self.monitor.revoke();}self.monitor.current()
    }
    pub(super) fn contents_current(&self,deadline:Instant)->bool{
        if !self.current(){return false;}
        let(mut budget,mut reads)=(Budget::for_request(deadline),ReadBudget{deadline,bytes:0,cancel:self.cancel.clone()});
        for folder in &self.folders{
            let mut hashes=BTreeMap::new();
            if folder.copy.visit(&mut budget,|file,path|{
                reads.check()?;if file.metadata().map_err(|_|INVALID)?.is_file(){hashes.insert(path.to_path_buf(),reads.hash(file)?);}Ok(())
            }).is_err()||hashes!=folder.hashes{return self.monitor.revoke();}
        }self.current()
    }
}

#[cfg(test)]
#[path="folder_watch_tests.rs"]
pub(super) mod tests;
