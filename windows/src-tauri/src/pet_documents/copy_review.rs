//! Metadata leases for explicitly reviewed inbox copies. No content is read.
use super::*;
use std::{ffi::{CString,OsStr},fs::{File,OpenOptions},os::{fd::{AsRawFd,FromRawFd},unix::{ffi::OsStrExt,fs::{MetadataExt,OpenOptionsExt}}},path::PathBuf};
const INVALID:&str="Uma cópia guardada está indisponível ou mudou. Volta a rever os anexos.";
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
struct Stamp(u64,u64,u64,i64,i64,i64,i64,u32,u32,u64,bool);
fn stamp(m:&std::fs::Metadata)->Stamp{Stamp(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink(),m.is_dir())}
fn private(m:&std::fs::Metadata)->bool{m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0&&(m.is_dir()||m.is_file()&&m.nlink()==1)}
fn open(parent:&File,name:&OsStr)->Result<File,String>{
    let name=CString::new(name.as_bytes()).map_err(|_|INVALID)?;
    let fd=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC,0)};
    if fd<0{Err(INVALID.into())}else{Ok(unsafe{File::from_raw_fd(fd)})}
}
pub(super) struct Budget{deadline:Instant,entries:usize,limit:usize}
impl Budget{
    pub(super) fn new()->Self{Self{deadline:Instant::now()+Duration::from_secs(8),entries:0,limit:2048}}
    pub(super) fn for_request(deadline:Instant)->Self{Self{deadline,entries:0,limit:3*MAX_CONVERSATION_ATTACHMENTS*257}}
    fn entry(&mut self)->Result<(),String>{self.entries+=1;if self.entries>self.limit||Instant::now()>=self.deadline{Err("A revisão excede o limite de elementos ou de tempo. Escolhe menos anexos de cada vez.".into())}else{Ok(())}}
}
#[derive(PartialEq,Eq)]
struct Tree{nodes:BTreeMap<PathBuf,Stamp>,size:u64,entries:usize}
fn walk(file:&File,relative:&Path,depth:usize,tree:&mut Tree,budget:&mut Budget,visit:&mut impl FnMut(&File,&Path)->Result<(),String>)->Result<(),String>{
    budget.entry()?;if tree.entries>256{return Err(INVALID.into());}
    let meta=file.metadata().map_err(|_|INVALID)?;if !private(&meta)||meta.is_dir()&&depth>12{return Err(INVALID.into());}
    let before=stamp(&meta);tree.nodes.insert(relative.into(),before.clone());visit(file,relative)?;
    if meta.is_dir(){
        let names=std::fs::read_dir(format!("/proc/self/fd/{}",file.as_raw_fd())).map_err(|_|INVALID)?;
        for entry in names{
            let name=entry.map_err(|_|INVALID)?.file_name();tree.entries+=1;if tree.entries>256{return Err(INVALID.into());}
            let child=open(file,&name)?;walk(&child,&relative.join(name),depth+1,tree,budget,visit)?;
        }
    }else{
        tree.size=tree.size.checked_add(meta.len()).filter(|n|*n<=MAX_CONVERSATION_BYTES).ok_or(INVALID)?;
    }
    if !file.metadata().is_ok_and(|m|stamp(&m)==before){return Err(INVALID.into());}Ok(())
}
fn tree_with(file:&File,budget:&mut Budget,visit:&mut impl FnMut(&File,&Path)->Result<(),String>)->Result<Tree,String>{let mut t=Tree{nodes:BTreeMap::new(),size:0,entries:0};walk(file,Path::new(""),0,&mut t,budget,visit)?;Ok(t)}
fn tree(file:&File,budget:&mut Budget)->Result<Tree,String>{tree_with(file,budget,&mut|_,_|Ok(()))}
pub(super) struct Copy{inbox:File,inbox_path:PathBuf,inbox_stamp:Stamp,file:File,path:PathBuf,root:Stamp,tree:Tree}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RetentionNode{pub path:Vec<u8>,stamp:Stamp}
impl RetentionNode{
    pub(super) fn valid(&self)->bool{
        let p=Path::new(OsStr::from_bytes(&self.path));
        (self.path.is_empty()||p.components().all(|c|matches!(c,Component::Normal(_)))&&p.components().collect::<PathBuf>().as_os_str().as_bytes()==self.path)
            &&p.components().count()<=(if self.stamp.10{12}else{13})&&self.path.len()<=65536&&!self.path.contains(&0)
            &&self.stamp.7==unsafe{libc::geteuid()}&&self.stamp.8&0o077==0
            &&(if self.stamp.10{self.stamp.8&libc::S_IFMT==libc::S_IFDIR}else{self.stamp.8&libc::S_IFMT==libc::S_IFREG&&self.stamp.9==1})
    }
    pub(super) fn matches(&self,m:&std::fs::Metadata,partial:bool)->bool{
        if !private(m){return false;}
        let mut now=stamp(m);let mut expected=self.stamp.clone();
        if self.path.is_empty(){now.5=0;now.6=0;expected.5=0;expected.6=0;}
        if partial&&expected.10{
            // Removing our children changes directory len/times/nlink. Identity,
            // owner and permissions remain mandatory during restart cleanup.
            now.2=0;now.3=0;now.4=0;now.5=0;now.6=0;now.9=0;
            expected.2=0;expected.3=0;expected.4=0;expected.5=0;expected.6=0;expected.9=0;
        }
        now==expected
    }
    pub(super) fn is_dir(&self)->bool{self.stamp.10}
}
impl Copy{
    pub(super) fn retention_tree(&self)->Vec<RetentionNode>{
        self.tree.nodes.iter().map(|(path,s)|{let mut s=s.clone();if path.as_os_str().is_empty(){s.5=0;s.6=0;}
            RetentionNode{path:path.as_os_str().as_bytes().to_vec(),stamp:s}
        }).collect()
    }
    pub(super) fn retention_digest(&self)->String{
        let bytes=serde_json::to_vec(&self.retention_tree()).expect("metadata tuples serialize");
        ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()
    }
    pub(super) fn capture(inbox:&Path,value:&Attachment,budget:&mut Budget)->Result<Self,String>{
        let path=PathBuf::from(&value.file.path);if path.parent()!=Some(inbox){return Err(INVALID.into());}
        let directory=OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(inbox).map_err(|_|INVALID)?;
        let directory_meta=directory.metadata().map_err(|_|INVALID)?;if !private(&directory_meta){return Err(INVALID.into());}
        let file=open(&directory,path.file_name().ok_or(INVALID)?)?;let meta=file.metadata().map_err(|_|INVALID)?;
        if !private(&meta)||meta.is_dir()!=(value.file.kind=="folder"){return Err(INVALID.into());}
        let root=stamp(&meta);let tree=tree(&file,budget)?;
        let entries=if meta.is_dir(){tree.entries}else{1};if tree.size!=value.file.size||entries!=value.file.entries{return Err(INVALID.into());}
        let copy=Self{inbox:directory,inbox_path:inbox.into(),inbox_stamp:stamp(&directory_meta),file,path,root,tree};if !copy.root_current(){return Err(INVALID.into());}Ok(copy)
    }
    pub(super) fn root_current(&self)->bool{
        // New independent inbox imports may change its timestamps. Pin the
        // directory inode/ownership/mode, not its mutable child listing.
        let same_inbox=|m:std::fs::Metadata|private(&m)&&m.is_dir()&&m.dev()==self.inbox_stamp.0&&m.ino()==self.inbox_stamp.1;
        self.inbox.metadata().is_ok_and(&same_inbox)&&std::fs::symlink_metadata(&self.inbox_path).is_ok_and(same_inbox)
            &&self.file.metadata().is_ok_and(|m|private(&m)&&stamp(&m)==self.root)
            &&std::fs::symlink_metadata(&self.path).is_ok_and(|m|private(&m)&&stamp(&m)==self.root)
    }
    pub(super) fn current(&self,budget:&mut Budget)->bool{self.root_current()&&tree(&self.file,budget).is_ok_and(|t|t==self.tree)&&self.root_current()}
    // The ordinary renewal review still visits metadata only. A request with
    // an existing document grant may supply its own watch/hash visitor.
    pub(super) fn visit(&self,budget:&mut Budget,mut visit:impl FnMut(&File,&Path)->Result<(),String>)->Result<(),String>{
        if !self.root_current(){return Err(INVALID.into());}
        if tree_with(&self.file,budget,&mut visit)?!=self.tree||!self.root_current(){return Err(INVALID.into());}Ok(())
    }
}
#[cfg(test)]
mod tests{
    use super::*;use std::os::unix::fs::{DirBuilderExt,PermissionsExt,symlink};
    struct Temp(PathBuf);
    impl Temp{
        fn new()->Self{let p=std::env::temp_dir().join(format!("coucou-copy-review-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();Self(p)}
        fn file(&self,name:&str,data:&[u8])->Attachment{let p=self.0.join(name);std::fs::write(&p,data).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o600)).unwrap();Attachment{id:"item-1".into(),provider:"ollama".into(),conversation_id:"cid".into(),sequence:0,file:crate::files::DroppedFile{name:name.into(),path:p.to_string_lossy().into_owned(),size:data.len()as u64,kind:"file".into(),entries:1}}}
    }
    impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    #[test]fn regular_copy_replacement_mutation_and_symlink_revoke_review(){
        let t=Temp::new();let a=t.file("a.txt",b"old");let c=Copy::capture(&t.0,&a,&mut Budget::new()).unwrap();assert!(c.current(&mut Budget::new()));std::fs::write(&a.file.path,b"new").unwrap();assert!(!c.current(&mut Budget::new()));
        let c=Copy::capture(&t.0,&a,&mut Budget::new()).unwrap();let b=t.file("b.txt",b"new");std::fs::rename(&b.file.path,&a.file.path).unwrap();assert!(!c.root_current());
        std::fs::remove_file(&a.file.path).unwrap();symlink("/etc/passwd",&a.file.path).unwrap();assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_err());
    }
    #[test]fn folder_review_covers_nested_files_and_rejects_links_and_excessive_tree(){
        let t=Temp::new();let a=t.file("document.txt",b"text");let folder=t.0.join("folder");std::fs::DirBuilder::new().mode(0o700).create(&folder).unwrap();std::fs::DirBuilder::new().mode(0o700).create(folder.join("nested")).unwrap();std::fs::rename(&a.file.path,folder.join("nested/document.txt")).unwrap();
        let mut a=a;a.file.path=folder.to_string_lossy().into_owned();a.file.kind="folder".into();a.file.entries=2;let c=Copy::capture(&t.0,&a,&mut Budget::new()).unwrap();assert!(c.current(&mut Budget::new()));std::fs::write(folder.join("nested/document.txt"),b"edit").unwrap();assert!(!c.current(&mut Budget::new()));
        symlink("/etc/passwd",folder.join("link")).unwrap();assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_err());std::fs::remove_file(folder.join("link")).unwrap();for i in 0..257{let p=folder.join(format!("item-{i}"));std::fs::write(&p,b"").unwrap();std::fs::set_permissions(p,std::fs::Permissions::from_mode(0o600)).unwrap();}assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_err());
    }
    #[test]fn twelve_nested_directories_allow_the_leaf_file_but_not_a_thirteenth_directory(){
        let t=Temp::new();let folder=t.0.join("folder");std::fs::DirBuilder::new().mode(0o700).create(&folder).unwrap();let mut leaf=folder.clone();for i in 0..12{leaf=leaf.join(format!("level-{i}"));std::fs::DirBuilder::new().mode(0o700).create(&leaf).unwrap();}
        let path=leaf.join("leaf.txt");std::fs::write(&path,b"one").unwrap();std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o600)).unwrap();let a=Attachment{id:"depth".into(),provider:"ollama".into(),conversation_id:"cid".into(),sequence:0,file:crate::files::DroppedFile{name:"folder".into(),path:folder.to_string_lossy().into_owned(),size:3,kind:"folder".into(),entries:13}};
        assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_ok());std::fs::DirBuilder::new().mode(0o700).create(leaf.join("level-13")).unwrap();assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_err());
    }
    #[test]fn unrelated_imports_do_not_revoke_a_copy_but_shared_files_and_deadlines_do(){
        let t=Temp::new();let a=t.file("a.txt",b"one");let c=Copy::capture(&t.0,&a,&mut Budget::new()).unwrap();t.file("another.txt",b"new");assert!(c.current(&mut Budget::new()));
        std::fs::hard_link(&a.file.path,t.0.join("alias")).unwrap();assert!(Copy::capture(&t.0,&a,&mut Budget::new()).is_err());std::fs::remove_file(t.0.join("alias")).unwrap();let mut budget=Budget::new();budget.deadline=Instant::now();assert!(Copy::capture(&t.0,&a,&mut budget).is_err());
    }
}
