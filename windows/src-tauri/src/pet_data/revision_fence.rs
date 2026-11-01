//! Cache of already verified immutable revisions, never a grant or an archive.
//! Full hashes still gate completion. Kernel stamps have the same local trust
//! limit as private_fs::Node; this is not isolation from root or the same UID.
use super::{inventory::{self,Snapshot},private_fs::Node,staging_fs::Dir,Error,Result};
use std::{ffi::OsStr,path::PathBuf};
pub(super)struct Tree{roots:[Dir;2],paths:[PathBuf;2],snapshot:Snapshot,nodes:Vec<Node>,pinned:bool}
fn descriptor_budget(nodes:usize)->bool{
 let mut limit:libc::rlimit=unsafe{std::mem::zeroed()};if unsafe{libc::getrlimit(libc::RLIMIT_NOFILE,&mut limit)}!=0{return false;}
 let Ok(fds)=std::fs::read_dir("/proc/self/fd")else{return false;};let mut used=0u64;for e in fds{if e.is_err(){return false;}used=used.saturating_add(1);}
 used.saturating_add(nodes as u64).saturating_add(64)<limit.rlim_cur
}
impl Tree{
 pub(super)fn capture(config:Dir,data:Dir)->Result<Self>{let s=inventory::inspect_current(&config.bound_path()?,&data.bound_path()?)?;Self::from_snapshot(config,data,&s)}
 pub(super)fn from_snapshot(config:Dir,data:Dir,snapshot:&Snapshot)->Result<Self>{Self::capture_mode(config,data,snapshot,None)}
 fn capture_mode(config:Dir,data:Dir,snapshot:&Snapshot,mode:Option<bool>)->Result<Self>{
  let paths=[config.bound_path()?,data.bound_path()?];let pinned=mode.unwrap_or_else(||descriptor_budget(snapshot.entries.len()+2));let nodes=snapshot.revision_nodes(&paths[0],&paths[1],pinned)?;
  let r=Self{roots:[config,data],paths,snapshot:snapshot.clone(),nodes,pinned};r.current()?;Ok(r)
 }
 pub(super)fn current(&self)->Result<()>{for d in &self.roots{d.check()?;}if self.pinned{for n in &self.nodes{n.current()?;}}else{self.snapshot.revision_nodes(&self.paths[0],&self.paths[1],false)?;}for d in &self.roots{d.check()?;}Ok(())}
}
pub(super)struct Controls{root:Dir,files:Vec<(String,Node)>}
impl Controls{
 pub(super)fn capture(root:&Dir,names:&[&str])->Result<Self>{let mut files=vec![];for name in names{let n=root.node()?.child(OsStr::new(name))?.ok_or(Error::SourceChanged)?;if n.directory{return Err(Error::UnsafePath);}files.push(((*name).into(),n));}let r=Self{root:root.clone(),files};r.current()?;Ok(r)}
 pub(super)fn current(&self)->Result<()>{self.root.check()?;for(name,n)in &self.files{n.current()?;self.root.node()?.same_child(OsStr::new(name),n)?;}Ok(())}
}
#[cfg(test)]mod tests{
 use super::*;use crate::pet_data::tests::Temp;use std::{fs,os::unix::fs::{DirBuilderExt,symlink}};
 fn fixture()->(Temp,Dir,Dir){let t=Temp::new();let c=t.0.join("c");let d=t.0.join("d");for p in [&c,&d]{fs::DirBuilder::new().mode(0o700).create(p).unwrap();}t.file(&c.join("nested/item.bin"),b"reviewed bytes");(t,Dir::root(&c).unwrap(),Dir::root(&d).unwrap())}
 #[test]fn both_descriptor_and_metadata_modes_reject_mutation_replacement_membership_and_root_swaps(){
  for pinned in [true,false]{for action in ["sameLength","replace","add","remove","root"]{let(t,c,d)=fixture();let s=inventory::inspect_current(&c.bound_path().unwrap(),&d.bound_path().unwrap()).unwrap();let f=Tree::capture_mode(c.clone(),d,&s,Some(pinned)).unwrap();let root=c.bound_path().unwrap();let p=root.join("nested/item.bin");match action{"sameLength"=>fs::write(&p,b"different data").unwrap(),"replace"=>{fs::rename(&p,root.join("old.bin")).unwrap();t.file(&p,b"reviewed bytes");},"add"=>t.file(&root.join("nested/new.bin"),b"new"),"remove"=>fs::remove_file(&p).unwrap(),_=>{fs::rename(&root,t.0.join("moved-root")).unwrap();fs::DirBuilder::new().mode(0o700).create(&root).unwrap();}}assert!(f.current().is_err(),"{action}, pinned={pinned}");}}
 }
 #[test]fn witness_never_upgrades_changed_or_unsafe_data_into_the_reviewed_revision(){let(t,c,d)=fixture();let s=inventory::inspect_current(&c.bound_path().unwrap(),&d.bound_path().unwrap()).unwrap();let p=c.bound_path().unwrap().join("nested/item.bin");fs::write(&p,b"changed before capture").unwrap();assert!(Tree::from_snapshot(c.clone(),d.clone(),&s).is_err());fs::remove_file(&p).unwrap();symlink(t.0.join("outside"),&p).unwrap();assert!(Tree::capture(c,d).is_err());}
 #[test]fn control_witness_uses_literal_named_file_binding_and_does_not_mistake_reads_for_writes(){let(t,c,_)=fixture();t.file(&c.bound_path().unwrap().join("header.json"),b"immutable control");let f=Controls::capture(&c,&["header.json"]).unwrap();fs::read(c.bound_path().unwrap().join("header.json")).unwrap();f.current().unwrap();fs::rename(c.bound_path().unwrap().join("header.json"),t.0.join("old-control")).unwrap();t.file(&c.bound_path().unwrap().join("header.json"),b"immutable control");assert!(f.current().is_err());}
}
