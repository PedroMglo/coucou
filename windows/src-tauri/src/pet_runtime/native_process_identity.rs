//! Narrow read-only facade over the runtime's existing pinned /proc observer.
//! This identity is evidence for native resources, never a lease or a grant.
use super::native_owner::Proc;
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct ProcessIdentity{pid:u32,uid:u32,start_ticks:String,boot_id:String,namespace_device:String,namespace_inode:String}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)enum ProcessPresence{OriginalLive,OriginalAbsent,IdentityMismatch,Unavailable}
impl ProcessIdentity{
 fn from(p:super::runtime_owner::Process)->Self{Self{pid:p.pid,uid:p.uid,start_ticks:p.start_ticks.to_string(),boot_id:p.boot_id,namespace_device:p.pid_namespace.device.to_string(),namespace_inode:p.pid_namespace.inode.to_string()}}
 fn valid(&self)->bool{self.pid>0&&self.start_ticks.parse::<u64>().is_ok_and(|n|n>0)&&super::runtime_owner::uuid(&self.boot_id)&&self.namespace_device.parse::<u64>().is_ok()&&self.namespace_inode.parse::<u64>().is_ok_and(|n|n>0)}
}
pub(crate)fn capture()->Result<ProcessIdentity,&'static str>{Proc::native().map(|p|ProcessIdentity::from(p.current())).map_err(|_|"native process identity unavailable")}
pub(crate)fn observe(original:&ProcessIdentity)->ProcessPresence{
 let Ok(proc)=Proc::native()else{return ProcessPresence::Unavailable};let now=ProcessIdentity::from(proc.current());
 if !original.valid()||now.uid!=original.uid||now.boot_id!=original.boot_id||now.namespace_device!=original.namespace_device||now.namespace_inode!=original.namespace_inode{return ProcessPresence::IdentityMismatch;}
 match proc.process(original.pid){Ok(Some(p))=>if ProcessIdentity::from(p)==*original{ProcessPresence::OriginalLive}else{ProcessPresence::IdentityMismatch},Ok(None)=>ProcessPresence::OriginalAbsent,Err(_)=>ProcessPresence::Unavailable}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn current_native_incarnation_is_not_absent_and_wrong_birth_boot_namespace_are_not_absence(){let p=capture().unwrap();assert_eq!(observe(&p),ProcessPresence::OriginalLive);for n in 0..4{let mut q=p.clone();match n{0=>q.start_ticks=(q.start_ticks.parse::<u64>().unwrap()+1).to_string(),1=>q.boot_id="00000000-0000-0000-0000-000000000000".into(),2=>q.namespace_inode=(q.namespace_inode.parse::<u64>().unwrap()+1).to_string(),_=>q.uid=q.uid.wrapping_add(1)}assert_eq!(observe(&q),ProcessPresence::IdentityMismatch);}}
 #[test]fn original_child_process_is_live_until_reaped_then_observed_absent_without_any_lease(){let mut child=std::process::Command::new("/usr/bin/sleep").arg("30").spawn().unwrap();let proc=Proc::native().unwrap();let p=ProcessIdentity::from(proc.process(child.id()).unwrap().unwrap());assert_eq!(observe(&p),ProcessPresence::OriginalLive);child.kill().unwrap();child.wait().unwrap();assert_eq!(observe(&p),ProcessPresence::OriginalAbsent);}
}
