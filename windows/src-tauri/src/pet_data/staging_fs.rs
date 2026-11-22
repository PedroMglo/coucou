//! Private new output only. Held descriptors and named inode checks protect
//! against accidental substitution; this is not isolation from the same UID.
use super::{private_fs::{self, Node}, Error, Result};
use std::{ffi::{CString, OsStr, OsString}, fs::{File, Metadata}, io::Write, os::{fd::{AsRawFd, FromRawFd}, unix::{ffi::OsStrExt, fs::MetadataExt}}, path::{Path, PathBuf}, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity(u64, u64, u32, u32, u64);
fn identity(file:&File) -> Result<Identity> {
    let m=file.metadata().map_err(|_|Error::Unavailable)?;let location=private_fs::location(file)?;
    Ok(Identity(m.dev(),m.ino(),m.uid(),m.mode(),location.mount))
}
fn private_dir(m: &Metadata) -> bool { m.is_dir() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o7777 == 0o700 }
fn owned_dir(m:&Metadata,private:bool)->bool{if private{private_dir(m)}else{m.is_dir()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o7022==0}}
pub(super) fn private_file(m: &Metadata) -> bool { m.is_file() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o7777 == 0o600 && m.nlink() == 1 }
fn name(n: &OsStr) -> Result<CString> {
    if n.is_empty() || n == "." || n == ".." || n.as_bytes().contains(&b'/') { return Err(Error::UnsafePath); }
    CString::new(n.as_bytes()).map_err(|_| Error::UnsafePath)
}
enum Binding { Root(PathBuf), Child(Arc<Inner>, OsString) }
struct Inner { file: File, identity: Identity, binding: Binding, private:bool }
#[derive(Clone)] pub(super) struct Dir(Arc<Inner>);
impl Dir {
    /// Lexical path for an already held/current binding, never a native permit.
    pub(super)fn bound_path(&self)->Result<PathBuf>{self.check()?;match &self.0.binding{Binding::Root(path)=>Ok(path.clone()),Binding::Child(parent,name)=>Ok(Dir(parent.clone()).bound_path()?.join(name))}}
    pub(super) fn root(path: &Path) -> Result<Self> {
        let node = private_fs::bound_directory_root(path)?.ok_or(Error::Unavailable)?;
        Self::from_node(node, Binding::Root(path.into()),true)
    }
    /// Existing application parent may be 0755. New children/output are always
    /// private; do not chmod or adopt broader permissions for a workspace.
    pub(super) fn parent(path:&Path)->Result<Self>{
        let node=private_fs::bound_directory_root(path)?.ok_or(Error::Unavailable)?;
        Self::from_node(node,Binding::Root(path.into()),false)
    }
    fn from_node(node: Node, binding: Binding,private:bool) -> Result<Self> {
        let metadata = node.file.metadata().map_err(|_| Error::Unavailable)?;
        if !owned_dir(&metadata,private) { return Err(Error::UnsafePath); }
        Ok(Self(Arc::new(Inner { identity: identity(&node.file)?, file: node.file, binding,private })))
    }
    pub(super) fn check(&self) -> Result<()> {
        let current = self.0.file.metadata().map_err(|_| Error::Unavailable)?;
        if !owned_dir(&current,self.0.private) || identity(&self.0.file)? != self.0.identity { return Err(Error::UnsafePath); }
        let named = match &self.0.binding {
            Binding::Root(path) => private_fs::bound_directory_root(path)?,
            Binding::Child(parent, name) => { Dir(parent.clone()).check()?; private_fs::bound_directory_child(&parent.file, name)? },
        }.ok_or(Error::SourceChanged)?;
        if identity(&named.file)? != self.0.identity { return Err(Error::SourceChanged); }
        Ok(())
    }
    pub(super) fn mkdir(&self, n: &OsStr) -> Result<Self> {
        self.check()?; let c = name(n)?;
        if unsafe { libc::mkdirat(self.0.file.as_raw_fd(), c.as_ptr(), 0o700) } != 0 {
            return Err(if std::io::Error::last_os_error().kind() == std::io::ErrorKind::AlreadyExists { Error::TargetExists } else { Error::Unavailable });
        }
        let child = self.dir(n)?; child.sync()?; self.sync()?; Ok(child)
    }
    pub(super) fn dir(&self, n: &OsStr) -> Result<Self> {
        self.check()?;
        let node = private_fs::bound_directory_child(&self.0.file, n)?.ok_or(Error::Unavailable)?;
        if node.device != self.0.identity.0 { return Err(Error::UnsafePath); }
        Self::from_node(node, Binding::Child(self.0.clone(), n.into()),true)
    }
    /// Publish a fully prepared child directory without replacing any target.
    pub(super) fn publish_dir(&self, staged: &OsStr, target: &OsStr, prepared: &Dir) -> Result<Dir> {
        self.transfer_dir(staged,self,target,prepared)
    }
    /// Transfer only the exact reviewed child between pinned parents. This is
    /// a storage primitive, never a caller-facing path or execution permit.
    pub(super) fn transfer_dir(&self, staged:&OsStr, destination:&Dir, target:&OsStr, prepared:&Dir)->Result<Dir>{
        self.check()?; destination.check()?; prepared.check()?;
        if self.0.identity.0!=destination.0.identity.0||self.0.identity.4!=destination.0.identity.4{return Err(Error::UnsafePath);}
        let named = self.dir(staged)?;
        if named.0.identity != prepared.0.identity { return Err(Error::SourceChanged); }
        let from = name(staged)?; let to = name(target)?;
        prepared.sync()?;
        if unsafe { libc::renameat2(self.0.file.as_raw_fd(), from.as_ptr(), destination.0.file.as_raw_fd(), to.as_ptr(), libc::RENAME_NOREPLACE) } != 0 {
            return Err(if std::io::Error::last_os_error().kind() == std::io::ErrorKind::AlreadyExists { Error::TargetExists } else { Error::Unavailable });
        }
        destination.sync()?; self.sync()?;
        let published = destination.dir(target)?;
        if published.0.identity != prepared.0.identity { return Err(Error::SourceChanged); }
        Ok(published)
    }
    pub(super) fn new_file(&self, n: &OsStr) -> Result<File> {
        self.check()?; let c = name(n)?;
        let fd = unsafe { libc::openat(self.0.file.as_raw_fd(), c.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC, 0o600) };
        if fd < 0 { return Err(if std::io::Error::last_os_error().kind() == std::io::ErrorKind::AlreadyExists { Error::TargetExists } else { Error::Unavailable }); }
        let file = unsafe { File::from_raw_fd(fd) };
        if !private_file(&file.metadata().map_err(|_| Error::Unavailable)?) { return Err(Error::UnsafePath); }
        Ok(file)
    }
    pub(super) fn lock_file(&self,n:&str)->Result<File>{
        self.check()?;let n=OsStr::new(n);
        for _ in 0..2{
            // Inspect through O_PATH before opening for I/O; never open an
            // unknown device/FIFO as a read-write lock file.
            let file=if let Some(node)=private_fs::child(&self.0.file,n)?{
                let meta=node.file.metadata().map_err(|_|Error::Unavailable)?;
                if !private_file(&meta)||meta.len()!=0{return Err(Error::UnsafePath);}
                File::open(format!("/proc/self/fd/{}",node.file.as_raw_fd())).map_err(|_|Error::Unavailable)?
            }else{
                match self.new_file(n){Ok(file)=>file,Err(Error::TargetExists)=>continue,Err(e)=>return Err(e)}
            };
            let metadata=file.metadata().map_err(|_|Error::Unavailable)?;
            if !private_file(&metadata)||metadata.len()!=0{return Err(Error::UnsafePath);}
            self.confirm(n,&file)?;return Ok(file);
        }
        Err(Error::Busy)
    }
    pub(super) fn confirm(&self, n: &OsStr, file: &File) -> Result<()> {
        self.check()?; let before = file.metadata().map_err(|_| Error::Unavailable)?;
        let named = private_fs::child(&self.0.file, n)?.ok_or(Error::SourceChanged)?;
        let after = named.file.metadata().map_err(|_| Error::Unavailable)?;
        if !private_file(&before) || !private_file(&after) || private_fs::stamp(&before) != private_fs::stamp(&after) { return Err(Error::SourceChanged); }
        Ok(())
    }
    pub(super) fn write(&self, n: &str, bytes: &[u8]) -> Result<()> {
        let mut file = self.new_file(OsStr::new(n))?;
        file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|_| Error::Unavailable)?;
        self.confirm(OsStr::new(n), &file)?; self.sync()
    }
    /// Publish a complete new record atomically. Preserve partial/staged files
    /// on uncertainty; never replace a known or foreign destination.
    pub(super) fn publish_new(&self,n:&str,bytes:&[u8])->Result<()>{
        use ring::rand::SecureRandom;
        let mut nonce=[0u8;16];ring::rand::SystemRandom::new().fill(&mut nonce).map_err(|_|Error::Unavailable)?;
        let staged=format!(".publish-{}",nonce.iter().map(|b|format!("{b:02x}")).collect::<String>());
        self.write(&staged,bytes)?;self.check()?;let from=name(OsStr::new(&staged))?;let to=name(OsStr::new(n))?;
        if unsafe{libc::renameat2(self.0.file.as_raw_fd(),from.as_ptr(),self.0.file.as_raw_fd(),to.as_ptr(),libc::RENAME_NOREPLACE)}!=0{
            return Err(if std::io::Error::last_os_error().kind()==std::io::ErrorKind::AlreadyExists{Error::TargetExists}else{Error::Unavailable});
        }
        self.sync()
    }
    /// Continue only a verified native staging prefix; never truncate/replace.
    pub(super)fn complete_prefix(&self,n:&str,bytes:&[u8])->Result<()>{
        use std::io::{Read,Write};
        let parent=self.node()?;
        let Some(node)=parent.child(OsStr::new(n))?else{return self.write(n,bytes);};
        if node.directory||node.len>bytes.len()as u64||!private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
        let mut prior=vec![];(&node.file).take(bytes.len()as u64+1).read_to_end(&mut prior).map_err(|_|Error::Unavailable)?;
        if !bytes.starts_with(&prior)||prior.len()as u64!=node.len{return Err(Error::SourceChanged);}node.current()?;parent.same_child(OsStr::new(n),&node)?;
        let c=name(OsStr::new(n))?;let fd=unsafe{libc::openat(self.0.file.as_raw_fd(),c.as_ptr(),libc::O_WRONLY|libc::O_APPEND|libc::O_NOFOLLOW|libc::O_CLOEXEC)};
        if fd<0{return Err(Error::Unavailable);}let mut file=unsafe{File::from_raw_fd(fd)};if identity(&file)?!=identity(&node.file)?{return Err(Error::SourceChanged);}self.confirm(OsStr::new(n),&file)?;if file.metadata().map_err(|_|Error::Unavailable)?.len()!=prior.len()as u64{return Err(Error::SourceChanged);}
        file.write_all(&bytes[prior.len()..]).and_then(|_|file.sync_all()).map_err(|_|Error::Unavailable)?;self.confirm(OsStr::new(n),&file)?;self.sync()
    }
    pub(super)fn publish_staged_new(&self,staged:&str,target:&str)->Result<()>{
        self.check()?;let node=self.node()?.child(OsStr::new(staged))?.ok_or(Error::SourceChanged)?;if node.directory||!private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}node.current()?;
        let from=name(OsStr::new(staged))?;let to=name(OsStr::new(target))?;if unsafe{libc::renameat2(self.0.file.as_raw_fd(),from.as_ptr(),self.0.file.as_raw_fd(),to.as_ptr(),libc::RENAME_NOREPLACE)}!=0{return Err(Error::TargetExists);}self.sync()
    }
    /// Exchange two owned regular files, retaining the actual displaced bytes.
    /// Caller verifies both sides and its native transaction before/after this
    /// point; uncertainty is preserved, never automatically repeated.
    pub(super)fn exchange_files(&self,staged:&str,target:&str)->Result<()>{
        self.check()?;let parent=self.node()?;
        for n in [staged,target]{let node=parent.child(OsStr::new(n))?.ok_or(Error::SourceChanged)?;if node.directory||!private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}node.current()?;parent.same_child(OsStr::new(n),&node)?;}
        let from=name(OsStr::new(staged))?;let to=name(OsStr::new(target))?;
        if unsafe{libc::renameat2(self.0.file.as_raw_fd(),from.as_ptr(),self.0.file.as_raw_fd(),to.as_ptr(),libc::RENAME_EXCHANGE)}!=0{return Err(Error::Unavailable);}self.sync()
    }
    pub(super) fn sync(&self) -> Result<()> { self.check()?; self.0.file.sync_all().map_err(|_| Error::Unavailable) }
    pub(super) fn node(&self) -> Result<Node> {
        self.check()?;
        // No source traversal: reopen our already pinned, checked descriptor.
        let file = File::open(format!("/proc/self/fd/{}", self.0.file.as_raw_fd())).map_err(|_| Error::Unavailable)?;
        let meta = file.metadata().map_err(|_| Error::Unavailable)?;
        Ok(Node { file, initial: private_fs::stamp(&meta), directory: true, device: meta.dev(), len: meta.len() })
    }
}
