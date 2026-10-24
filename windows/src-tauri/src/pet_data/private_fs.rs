//! Read-only, descriptor-relative Linux inventory. Do not traverse arbitrary
//! HOME/config trees or open devices/FIFOs while deciding whether to migrate.
use super::{Error, Result};
use std::{ffi::{CString, OsStr, OsString}, fs::{File, Metadata, OpenOptions}, os::{fd::{AsRawFd, FromRawFd}, unix::{ffi::OsStrExt, fs::{MetadataExt, OpenOptionsExt}}}, path::{Component, Path}};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Stamp(u64, u64, u64, i64, i64, i64, i64, u32, u32, u64);
impl Stamp { pub(super) fn len(&self) -> u64 { self.2 } }
pub(super) fn stamp(m: &Metadata) -> Stamp { Stamp(m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec(), m.uid(), m.mode(), m.nlink()) }
fn owned(m: &Metadata) -> bool { m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o022 == 0 }
fn ancestor(m: &Metadata) -> bool {
    m.is_dir() && (m.uid() == 0 || m.uid() == unsafe { libc::geteuid() })
        && (m.mode() & 0o022 == 0 || m.mode() & libc::S_ISVTX != 0)
}
fn component(n: &OsStr) -> Result<CString> {
    if n.is_empty() || n.as_bytes().contains(&b'/') || n == "." || n == ".." { return Err(Error::UnsafePath); }
    CString::new(n.as_bytes()).map_err(|_| Error::UnsafePath)
}
fn pin(parent: &File, name: &OsStr) -> Result<Option<File>> {
    let name = component(name)?;
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if fd >= 0 { return Ok(Some(unsafe { File::from_raw_fd(fd) })); }
    if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound { Ok(None) } else { Err(Error::Unavailable) }
}
fn metadata(f: &File) -> Result<Metadata> { f.metadata().map_err(|_| Error::Unavailable) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Location { pub device:u64, pub inode:u64, pub mount:u64 }
pub(super) fn location(file:&File)->Result<Location>{
    let meta=metadata(file)?;let mut stat:libc::statx=unsafe{std::mem::zeroed()};
    let empty=b"\0";
    if unsafe{libc::statx(file.as_raw_fd(),empty.as_ptr().cast(),libc::AT_EMPTY_PATH|libc::AT_SYMLINK_NOFOLLOW,libc::STATX_MNT_ID,&mut stat)}!=0
        ||stat.stx_mask&libc::STATX_MNT_ID==0||stat.stx_mnt_id==0{return Err(Error::UnsafePath);}
    Ok(Location{device:meta.dev(),inode:meta.ino(),mount:stat.stx_mnt_id})
}
fn readable(pin: File) -> Result<Node> {
    let before = metadata(&pin)?;
    if !owned(&before) || !(before.is_dir() || before.is_file() && before.nlink() == 1) { return Err(Error::UnsafePath); }
    // Follow our pinned descriptor, not a user-controlled path. O_PATH above
    // lets us reject links, devices, sockets and FIFOs before opening for I/O.
    let file = OpenOptions::new().read(true).custom_flags(libc::O_CLOEXEC | libc::O_NONBLOCK | if before.is_dir() { libc::O_DIRECTORY } else { 0 })
        .open(format!("/proc/self/fd/{}", pin.as_raw_fd())).map_err(|_| Error::Unavailable)?;
    if stamp(&before) != stamp(&metadata(&file)?) { return Err(Error::SourceChanged); }
    Ok(Node { file, initial: stamp(&before), directory: before.is_dir(), device: before.dev(), len: before.len() })
}

pub(super) fn child(parent: &File, name: &OsStr) -> Result<Option<Node>> {
    let node=pin(parent, name)?.map(readable).transpose()?;
    if let Some(node)=&node{
        let a=location(parent)?;let b=location(&node.file)?;
        if a.device!=b.device||a.mount!=b.mount{return Err(Error::UnsafePath);}
    }
    Ok(node)
}

pub(super) struct Node { pub file: File, pub initial: Stamp, pub directory: bool, pub device: u64, pub len: u64 }
impl Node {
    pub(super) fn root(path: &Path) -> Result<Option<Self>> {
        if !super::absolute(path) { return Err(Error::UnsafePath); }
        let mut at = OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC).open("/").map_err(|_| Error::Unavailable)?;
        for part in path.components() {
            let Component::Normal(name) = part else { continue; };
            let Some(next) = pin(&at, name)? else { return Ok(None); };
            if !ancestor(&metadata(&next)?) { return Err(Error::UnsafePath); }
            at = next;
        }
        readable(at).map(Some)
    }
    pub(super) fn child(&self, name: &OsStr) -> Result<Option<Self>> {
        self.current()?;
        child(&self.file,name)
    }
    pub(super) fn names(&self, limit: usize) -> Result<Vec<OsString>> {
        if !self.directory { return Err(Error::UnsupportedProfile); }
        self.current()?;
        let mut names = vec![];
        for entry in std::fs::read_dir(format!("/proc/self/fd/{}", self.file.as_raw_fd())).map_err(|_| Error::Unavailable)? {
            if names.len() >= limit { return Err(Error::Limit); }
            names.push(entry.map_err(|_| Error::Unavailable)?.file_name());
        }
        names.sort(); self.current()?; Ok(names)
    }
    pub(super) fn current(&self) -> Result<()> {
        if stamp(&metadata(&self.file)?) == self.initial { Ok(()) } else { Err(Error::SourceChanged) }
    }
    pub(super) fn same_child(&self, name: &OsStr, original: &Node) -> Result<()> {
        match self.child(name)? { Some(now) if now.initial == original.initial => Ok(()), _ => Err(Error::SourceChanged) }
    }
}
