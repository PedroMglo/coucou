//! Native preference-file witnesses, reusing the registry's held directory and
//! metadata primitives. This is optimistic revision checking, not a sandbox or
//! an atomic CAS against non-cooperating same-UID writers. No authority lives here.
use super::{digest, stamp, Directory, Stamp};
use std::{ffi::OsString, fs::{File, Metadata}, io::{self, Read, Write},
    os::{fd::AsRawFd, unix::{fs::MetadataExt,ffi::OsStrExt}}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

const MAX_BYTES: usize = 256 * 1024;
static WRITER: Mutex<()> = Mutex::new(());
fn unavailable() -> io::Error { io::Error::other("O ficheiro de preferências está indisponível.") }
fn changed() -> io::Error { io::Error::other("O ficheiro ou o destino das preferências mudou desde a leitura; a gravação foi recusada.") }
fn owned(m: &Metadata) -> bool {
    m.is_file() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o7022 == 0 && m.nlink() == 1
}
fn directory(path: &Path) -> io::Result<Arc<Directory>> {
    Directory::open(path, false).map_err(|_| unavailable())?.ok_or_else(unavailable)
}
fn metadata(file: &File) -> io::Result<Metadata> { file.metadata().map_err(|_| unavailable()) }
fn named(directory: &Directory, name: &std::ffi::OsStr) -> io::Result<Option<File>> {
    match directory.open_file(name, libc::O_PATH, 0) {
        Ok(file) => Ok(Some(file)), Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None), Err(_) => Err(unavailable()),
    }
}
#[derive(Clone)]
struct Alias { directory: Arc<Directory>, name: OsString, file: Arc<File>, stamp: Stamp }
impl Alias {
    fn current(&self) -> bool {
        self.directory.current() && metadata(&self.file).is_ok_and(|m| stamp(&m) == self.stamp)
            && named(&self.directory, &self.name).ok().flatten().is_some_and(|f| metadata(&f).is_ok_and(|m| stamp(&m) == self.stamp))
    }
}

#[derive(Clone)]
pub(crate) struct Witness {
    selected: PathBuf, resolved: PathBuf, directory: Arc<Directory>,
    file: Option<Arc<File>>, stamp: Option<Stamp>, alias: Option<Alias>, hash: Option<String>,
}
impl std::fmt::Debug for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreferenceWitness").field("present", &self.file.is_some()).field("legacy_alias", &self.alias.is_some()).finish_non_exhaustive()
    }
}
impl Witness {
    pub(crate) fn selected(&self) -> &Path { &self.selected }
    pub(crate)fn identity(&self)->Option<String>{self.stamp.as_ref().map(|s|digest(format!("{s:?}").as_bytes()))}
    pub(crate) fn content_hash(&self) -> Option<&str> { self.hash.as_deref() }
    pub(crate) fn matches_metadata(&self, m: &Metadata) -> bool { self.stamp.as_ref() == Some(&stamp(m)) }
    pub(crate) fn matches_bytes(&self, bytes: &[u8]) -> bool { self.hash.as_ref() == Some(&digest(bytes)) }
    pub(crate) fn current(&self) -> bool {
        if !self.directory.current() || self.alias.as_ref().is_some_and(|a| !a.current()) { return false; }
        if self.alias.is_some() {
            if std::fs::canonicalize(&self.selected).ok().as_ref() != Some(&self.resolved) { return false; }
        } else if std::fs::symlink_metadata(&self.selected).is_ok_and(|m| m.file_type().is_symlink()) { return false; }
        let Some(name) = self.resolved.file_name() else { return false; };
        match (&self.file, &self.stamp) {
            (Some(file), Some(before)) => metadata(file).is_ok_and(|m| owned(&m) && stamp(&m) == *before)
                && named(&self.directory, name).ok().flatten().is_some_and(|f| metadata(&f).is_ok_and(|m| owned(&m) && stamp(&m) == *before))
                && std::fs::metadata(&self.selected).is_ok_and(|m| owned(&m) && stamp(&m) == *before),
            (None, None) => named(&self.directory, name).is_ok_and(|f| f.is_none())
                && std::fs::symlink_metadata(&self.selected).is_err_and(|e| e.kind() == io::ErrorKind::NotFound),
            _ => false,
        }
    }
    pub(crate) fn commit(&self, bytes: &[u8]) -> io::Result<Self> { self.commit_with(bytes, || Ok(()), || {}, File::sync_all) }
    #[cfg(test)]
    pub(crate) fn commit_for_test(&self, bytes: &[u8], before: impl FnOnce(), after: impl FnOnce()) -> io::Result<Self> { self.commit_with(bytes, ||{before();Ok(())}, after, File::sync_all) }
    #[cfg(test)]pub(crate)fn commit_sync_for_test(&self,bytes:&[u8],sync:impl FnOnce(&File)->io::Result<()>)->io::Result<Self>{self.commit_with(bytes,||Ok(()),||{},sync)}
    /// Exact named regular-file removal. Original data is retained by the
    /// caller's durable intent; this primitive never claims rollback.
    pub(crate)fn remove_guarded(&self,guard:impl FnOnce()->io::Result<()>)->io::Result<Self>{
        let _writer=WRITER.lock().map_err(|_|unavailable())?;
        if self.alias.is_some()||!self.current(){return Err(changed());}guard()?;
        if self.file.is_none(){return Ok(self.clone());}
        let name=std::ffi::CString::new(self.resolved.file_name().ok_or_else(unavailable)?.as_bytes()).map_err(|_|unavailable())?;
        if unsafe{libc::unlinkat(self.directory.file.as_raw_fd(),name.as_ptr(),0)}!=0{return Err(uncertain());}
        self.directory.file.sync_all().map_err(|_|uncertain())?;
        let(next,actual)=read(&self.selected,false).map_err(|_|uncertain())?;
        if actual.is_some()||!self.directory.current(){return Err(uncertain());}Ok(next)
    }
    pub(crate)fn commit_guarded(&self,bytes:&[u8],guard:impl FnOnce()->io::Result<()>)->io::Result<Self>{self.commit_with(bytes,guard,||{},File::sync_all)}
    fn commit_with(&self, bytes: &[u8], before_publish: impl FnOnce()->io::Result<()>, after_publish: impl FnOnce(), sync_directory:impl FnOnce(&File)->io::Result<()>) -> io::Result<Self> {
        let _writer = WRITER.lock().map_err(|_| unavailable())?;
        if bytes.len() > MAX_BYTES { return Err(io::Error::new(io::ErrorKind::InvalidInput, "As preferências excedem o limite de gravação.")); }
        if !self.current() { return Err(changed()); }
        let temp = self.resolved.with_extension(format!("json.coucou-{}", std::process::id()));
        let name = temp.file_name().ok_or_else(unavailable)?;
        // Failure to create means this name is not ours. Never truncate/remove it.
        let mut file = self.directory.open_file(name, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, 0o600)?;
        let created = metadata(&file)?; let mut published = false;
        let result = (|| {
            file.write_all(bytes).and_then(|_| file.sync_all())?;
            before_publish()?;
            if !self.current() || !self.directory.current() { return Err(changed()); }
            let own = metadata(&file)?;
            if !named(&self.directory, name)?.is_some_and(|f| metadata(&f).is_ok_and(|m| stamp(&m) == stamp(&own))) { return Err(changed()); }
            self.directory.rename(name, self.resolved.file_name().ok_or_else(unavailable)?).map_err(|_| unavailable())?;
            published = true;
            after_publish();
            // This is the already pinned readable directory, not an O_PATH
            // descriptor or a new lookup that could follow a replaced parent.
            sync_directory(&self.directory.file).map_err(|_| uncertain())?;
            let (next, actual) = read(&self.selected, self.alias.is_some()).map_err(|_| uncertain())?;
            let own = metadata(&file).map_err(|_| uncertain())?;
            if !self.directory.current() || self.alias.as_ref().is_some_and(|a| !a.current())
                || next.resolved != self.resolved || !next.matches_metadata(&own) || actual.as_deref() != Some(bytes) {
                return Err(uncertain());
            }
            Ok(next)
        })();
        if result.is_err() && !published {
            // A replacement at this name is foreign even if our own write failed.
            if named(&self.directory, name).ok().flatten().is_some_and(|f| metadata(&f).is_ok_and(|m| m.dev() == created.dev() && m.ino() == created.ino())) {
                self.directory.remove(name);
            }
        }
        result
    }
}

/// Capture the selected name, optional existing legacy alias and exact bytes.
/// O_PATH classifies unknown nodes before opening anything for content IO.
pub(crate) fn read(path: &Path, allow_legacy_alias: bool) -> io::Result<(Witness, Option<Vec<u8>>)> {
    if !path.is_absolute() { return Err(unavailable()); }
    let name = path.file_name().ok_or_else(unavailable)?;
    let selected_parent = directory(path.parent().ok_or_else(unavailable)?)?;
    let selected = named(&selected_parent, name)?;
    let alias = if let Some(file) = selected {
        let m = metadata(&file)?;
        if m.file_type().is_symlink() {
            if !allow_legacy_alias || m.uid() != unsafe { libc::geteuid() } { return Err(unavailable()); }
            Some(Alias { directory: selected_parent, name: name.into(), stamp: stamp(&m), file: Arc::new(file) })
        } else { if !owned(&m) { return Err(unavailable()); } None }
    } else { None };
    let resolved = if alias.is_some() { std::fs::canonicalize(path).map_err(|_| unavailable())? }
        else { std::fs::canonicalize(path.parent().ok_or_else(unavailable)?).map_err(|_| unavailable())?.join(name) };
    let parent = directory(resolved.parent().ok_or_else(unavailable)?)?;
    let pin = named(&parent, resolved.file_name().ok_or_else(unavailable)?)?;
    let (file, before, bytes) = if let Some(pin) = pin {
        let m = metadata(&pin)?;
        if !owned(&m) || m.len() > MAX_BYTES as u64 { return Err(unavailable()); }
        let mut file = File::open(format!("/proc/self/fd/{}", pin.as_raw_fd())).map_err(|_| unavailable())?;
        if stamp(&metadata(&file)?) != stamp(&m) { return Err(changed()); }
        let mut bytes = Vec::new(); (&mut file).take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != m.len() || bytes.len() > MAX_BYTES { return Err(changed()); }
        (Some(Arc::new(file)), Some(stamp(&m)), Some(bytes))
    } else { (None, None, None) };
    let value = Witness { selected: path.into(), resolved, directory: parent, file, stamp: before, alias, hash: bytes.as_deref().map(digest) };
    if !value.current() { return Err(changed()); } Ok((value, bytes))
}

#[derive(Debug)]
struct Unconfirmed;
impl std::fmt::Display for Unconfirmed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("As preferências podem ter sido publicadas, mas a confirmação falhou. Requer reconciliação; não repetir efeitos.") }
}
impl std::error::Error for Unconfirmed {}
fn uncertain() -> io::Error { io::Error::other(Unconfirmed) }
pub(crate) fn unconfirmed(error: &io::Error) -> bool { error.get_ref().is_some_and(|e| e.is::<Unconfirmed>()) }
