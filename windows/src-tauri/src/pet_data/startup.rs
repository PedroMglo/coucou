//! Native startup ownership. Existing single-instance setup can ignore some
//! bus errors: a registered plugin or JSON/profile marker is not this claim.
use super::{staging_fs::Dir,Error,Layout,Result};
use dbus::blocking::Connection;
use std::{fs::File,os::fd::AsRawFd,path::{Path,PathBuf},sync::atomic::{AtomicBool,Ordering},time::Duration};

const LOCK:&str=".pets-profile-v1.lock";
const TIMEOUT:Duration=Duration::from_millis(500);
struct Owner{connection:Connection,name:String,unique:String,pid:u32,uid:u32}
fn name(identifier:&str)->Result<String>{
    if identifier.is_empty()||identifier.len()>200||identifier.starts_with(':'){return Err(Error::UnsafePath);}
    let name=format!("{identifier}.SingleInstance");
    let _=dbus::strings::BusName::new(name.clone()).map_err(|_|Error::UnsafePath)?;Ok(name)
}
fn local_address(address:&str)->Result<()>{
    if address.len()>4096||!address.starts_with("unix:")||address.contains(';')||address.chars().any(char::is_control){Err(Error::UnsafePath)}else{Ok(())}
}
impl Owner{
    fn at(address:&str,identifier:&str)->Result<Self>{
        local_address(address)?;let name=name(identifier)?;
        // Explicit address: never auto-launch a bus or fall back to the system
        // bus/network when the original session bus is missing.
        let connection=Connection::new_address(address).map_err(|_|Error::Unavailable)?;
        let mut owner=Self{connection,name,unique:String::new(),pid:std::process::id(),uid:unsafe{libc::geteuid()}};
        owner.unique=owner.observe()?;owner.current()?;Ok(owner)
    }
    fn observe(&self)->Result<String>{
        let proxy=self.connection.with_proxy("org.freedesktop.DBus","/org/freedesktop/DBus",TIMEOUT);
        let(first,):(String,)=proxy.method_call("org.freedesktop.DBus","GetNameOwner",(&self.name,)).map_err(|_|Error::OwnershipLost)?;
        let(pid,):(u32,)=proxy.method_call("org.freedesktop.DBus","GetConnectionUnixProcessID",(&first,)).map_err(|_|Error::OwnershipLost)?;
        let(uid,):(u32,)=proxy.method_call("org.freedesktop.DBus","GetConnectionUnixUser",(&first,)).map_err(|_|Error::OwnershipLost)?;
        let(last,):(String,)=proxy.method_call("org.freedesktop.DBus","GetNameOwner",(&self.name,)).map_err(|_|Error::OwnershipLost)?;
        if first!=last||!first.starts_with(':')||pid!=self.pid||uid!=self.uid||std::process::id()!=self.pid{return Err(Error::OwnershipLost);}
        Ok(first)
    }
    fn current(&self)->Result<()>{if self.observe()?==self.unique{Ok(())}else{Err(Error::OwnershipLost)}}
}
struct Lock{directory:Dir,file:File,pid:u32}
impl Lock{
    fn acquire(root:&Path)->Result<Self>{
        let directory=Dir::parent(root)?;let file=directory.lock_file(LOCK)?;
        if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}
        let lock=Self{directory,file,pid:std::process::id()};lock.current()?;
        lock.file.sync_all().map_err(|_|Error::Unavailable)?;lock.directory.sync()?;Ok(lock)
    }
    fn current(&self)->Result<()>{
        if std::process::id()!=self.pid||self.file.metadata().map_err(|_|Error::Unavailable)?.len()!=0{return Err(Error::OwnershipLost);}
        self.directory.confirm(std::ffi::OsStr::new(LOCK),&self.file)
    }
}
impl Drop for Lock{fn drop(&mut self){unsafe{libc::flock(self.file.as_raw_fd(),libc::LOCK_UN)};}}
/// Non-cloneable/non-serializable. Keep alive through publication and app use;
/// revalidate at mutation/admission boundaries. Does not grant pet execution.
pub(crate) struct Claim{owner:Owner,lock:Lock,root:PathBuf,invalid:AtomicBool}
impl Claim{
    /// Must run after the original single-instance plugin owns its name, before
    /// constructing Settings/Chat/Recap or doing any migration/profile writes.
    /// This constructor observes ownership; it never claims/replaces a bus name.
    pub(crate) fn acquire(app:&tauri::AppHandle,layout:&Layout)->Result<Self>{
        let address=std::env::var("DBUS_SESSION_BUS_ADDRESS").map_err(|_|Error::Unavailable)?;
        Self::at(&address,&app.config().identifier,&layout.legacy_config)
    }
    fn at(address:&str,identifier:&str,root:&Path)->Result<Self>{
        let owner=Owner::at(address,identifier)?;owner.current()?;ensure_app_root(root)?;owner.current()?;let lock=Lock::acquire(root)?;
        let claim=Self{owner,lock,root:root.into(),invalid:AtomicBool::new(false)};claim.current()?;Ok(claim)
    }
    pub(crate) fn current(&self)->Result<()>{
        if self.invalid.load(Ordering::Acquire){return Err(Error::OwnershipLost);}
        let result=self.owner.current().and_then(|_|self.lock.current());
        if result.is_err(){self.invalid.store(true,Ordering::Release);}result
    }
    pub(super) fn for_layout(&self,layout:&Layout)->Result<()>{if self.root!=layout.legacy_config{return Err(Error::UnsafePath);}self.current()}
    pub(super)fn ensure_data_root(&self,layout:&Layout)->Result<()>{self.for_layout(layout)?;ensure_app_root(&layout.legacy_data)?;self.current()}
    pub(super)fn into_storage(self)->Result<StorageLease>{self.current()?;Ok(StorageLease{lock:self.lock,invalid:AtomicBool::new(false)})}
}
/// Acquired only after a genuine startup claim/admission. The physical lock,
/// not continued availability of the session bus, serializes live app data.
pub(super)struct StorageLease{lock:Lock,invalid:AtomicBool}
impl StorageLease{
    pub(super)fn current(&self)->Result<()>{
        if self.invalid.load(Ordering::Acquire){return Err(Error::OwnershipLost);}
        let result=self.lock.current();if result.is_err(){self.invalid.store(true,Ordering::Release);}result
    }
}
fn ensure_app_root(path:&Path)->Result<()>{
    if !super::absolute(path)||path.file_name().is_none_or(|n|n!="coucou"){return Err(Error::UnsafePath);}
    let mut missing=vec![];let mut at=path;
    while super::private_fs::Node::root(at)?.is_none(){
        if missing.len()>=4{return Err(Error::Limit);}missing.push(at.file_name().ok_or(Error::UnsafePath)?.to_os_string());at=at.parent().ok_or(Error::UnsafePath)?;
    }
    let mut directory=Dir::parent(at)?;
    for component in missing.into_iter().rev(){directory=directory.mkdir(&component)?;}
    Dir::parent(path)?.check()
}

#[cfg(test)]#[path="startup_tests.rs"]pub(super)mod tests;
