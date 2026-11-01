//! Exact native file resource, sharing registry Directory/Witness. No authority
//! lives here. A missing one-level parent is created only inside a caller's
//! already durably dispatched operation. This is optimistic, not same-UID CAS.
use super::{Directory,digest};
use super::preferences_file::{self,Witness};
use std::{ffi::{CString,OsStr},io,os::{fd::AsRawFd,unix::{ffi::OsStrExt,fs::MetadataExt}},path::{Path,PathBuf},sync::Arc};
fn changed()->io::Error{io::Error::other("O destino nativo mudou; não foi autorizada outra publicação.")}
fn name(s:&str)->bool{!s.is_empty()&&s!="."&&s!=".."&&s.len()<=128&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-".contains(&b))}
#[derive(Clone)]pub(crate)struct Target{base:Arc<Directory>,folder:String,file:String,child:Option<Arc<Directory>>,witness:Option<Witness>,before:Option<Vec<u8>>,selected:PathBuf,resource:String}
impl Target{
 pub(crate)fn capture(base:&Path,folder:&str,file:&str,max_bytes:usize)->io::Result<Self>{
  if !base.is_absolute()||!name(folder)||!name(file)||max_bytes>256*1024{return Err(changed());}
  let root=Directory::open(base,false).map_err(|_|changed())?.ok_or_else(changed)?;
  let child_path=base.join(folder);let child=Directory::open(&child_path,false).map_err(|_|changed())?;
  let selected=child_path.join(file);let (witness,before)=if child.is_some(){let(w,b)=preferences_file::read(&selected,false)?;if b.as_ref().is_some_and(|b|b.len()>max_bytes){return Err(changed());}(Some(w),b)}else{(None,None)};
  // A path is a presentation location, not a persistent physical identity.
  // Birth time disambiguates inode reuse; never fall back to a path-only key.
  let mut stat:libc::statx=unsafe{std::mem::zeroed()};let empty=b"\0";
  if unsafe{libc::statx(root.file.as_raw_fd(),empty.as_ptr().cast(),libc::AT_EMPTY_PATH|libc::AT_SYMLINK_NOFOLLOW,libc::STATX_BTIME,&mut stat)}!=0||stat.stx_mask&libc::STATX_BTIME==0{return Err(io::Error::other("A identidade física da pasta nativa não está disponível (birth-time)."));}
  let mut raw=b"coucou-native-file-slot-v2\0".to_vec();raw.extend_from_slice(&root.device.to_le_bytes());raw.extend_from_slice(&root.inode.to_le_bytes());raw.extend_from_slice(&stat.stx_btime.tv_sec.to_le_bytes());raw.extend_from_slice(&stat.stx_btime.tv_nsec.to_le_bytes());raw.extend(folder.as_bytes());raw.push(0);raw.extend(file.as_bytes());
  let value=Self{base:root,folder:folder.into(),file:file.into(),child,witness,before,selected,resource:digest(&raw)};if !value.current(){return Err(changed());}Ok(value)
 }
 pub(crate)fn path(&self)->&Path{&self.selected}
 pub(crate)fn resource(&self)->&str{&self.resource}
 pub(crate)fn before(&self)->Option<&[u8]>{self.before.as_deref()}
 pub(crate)fn revision(&self)->String{self.witness.as_ref().and_then(|w|w.content_hash()).unwrap_or("missing").into()}
 pub(crate)fn parent_present(&self)->bool{self.child.is_some()}
 pub(crate)fn fence(&self,guard:impl Fn()->io::Result<()>)->io::Result<Fence>{
  guard()?;if !self.current(){return Err(changed());}
  let lock=super::operation_lock_guarded(&self.base,".coucou-autostart.lock",||guard().map_err(|_|"nativeResourceRevoked")).map_err(io::Error::other)?;
  if !self.current(){return Err(changed());}Ok(Fence{lock,base:self.base.clone()})
 }
 pub(crate)fn anchor_current(&self)->bool{self.base.current()&&self.child.as_ref().is_none_or(|child|child.current())}
 pub(crate)fn current(&self)->bool{
  if !self.base.current(){return false;}
  match(&self.child,&self.witness){(Some(child),Some(w))=>child.current()&&w.current(),(None,None)=>self.base.open_file(OsStr::new(&self.folder),libc::O_PATH,0).is_err_and(|e|e.kind()==io::ErrorKind::NotFound),_=>false}
 }
 /// The caller has persisted dispatch before this function may mkdir. After
 /// own creation use the returned held witness, never recapture another target.
 pub(crate)fn materialize(&self,create:bool,guard:impl Fn()->io::Result<()>)->io::Result<Resolved>{
  guard()?;if !self.current(){return Err(changed());}
  if let Some(w)=&self.witness{return Ok(Resolved{witness:Some(w.clone()),target:self.clone(),created_parent:false});}
  if !create{return Ok(Resolved{witness:None,target:self.clone(),created_parent:false});}
  let name=CString::new(self.folder.as_str()).map_err(|_|changed())?;guard()?;
  if unsafe{libc::mkdirat(self.base.file.as_raw_fd(),name.as_ptr(),0o700)}!=0{return Err(io::Error::last_os_error());}
  self.base.file.sync_all()?;
  let child=Directory::open(&self.base.path.join(&self.folder),false).map_err(|_|changed())?.ok_or_else(changed)?;child.file.sync_all()?;
  if !self.base.current()||!child.current(){return Err(changed());}
  let(witness,bytes)=preferences_file::read(&self.selected,false)?;if bytes.is_some(){return Err(changed());}
  let mut target=self.clone();target.child=Some(child);target.witness=Some(witness.clone());
  guard()?;Ok(Resolved{witness:Some(witness),target,created_parent:true})
 }
}
pub(crate)struct Fence{lock:super::WriterLock,base:Arc<Directory>}
impl Fence{pub(crate)fn current(&self)->bool{
 self.base.current()&&self.base.open_file(OsStr::new(".coucou-autostart.lock"),libc::O_PATH,0).is_ok_and(|named|{
  let a=named.metadata();let b=self.lock.0.metadata();match(a,b){(Ok(a),Ok(b))=>super::private_file(&a)&&a.len()==0&&super::stamp(&a)==super::stamp(&b),_=>false}
 })
}}
pub(crate)struct Resolved{witness:Option<Witness>,target:Target,pub created_parent:bool}
impl Resolved{
 pub(crate)fn write(self,bytes:&[u8],guard:impl FnOnce()->io::Result<()>)->io::Result<Witness>{let witness=self.witness.ok_or_else(changed)?;witness.commit_guarded(bytes,guard)}
 pub(crate)fn remove(self,guard:impl FnOnce()->io::Result<()>)->io::Result<Option<Witness>>{match self.witness{Some(w)=>w.remove_guarded(guard).map(Some),None=>{guard()?;if self.target.current(){Ok(None)}else{Err(changed())}}}}
}
