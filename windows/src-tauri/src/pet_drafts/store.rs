use super::model::{Import,ImportReceipt,Record,Registry,Target,Write};
use std::path::Path;
pub(super)fn read(path:&Path,target:Target,current:impl Fn()->Result<(),&'static str>)->Result<Option<Record>,&'static str>{
 target.validate()?;current()?;let before=crate::private_registry::load_at::<Registry>(path)?;
 let result=before.registry.records.iter().find(|r|r.target==target).cloned();current()?;
 if !before.unchanged(){return Err("registryChanged");}Ok(result)
}
#[cfg(test)]
pub(super)fn write(path:&Path,target:Target,expected:Option<String>,text_units:Vec<u16>,current:impl Fn()->Result<(),&'static str>)->Result<Write,&'static str>{write_prepared(path,target,expected,text_units,current,||Ok(()))}
pub(super)fn write_prepared(path:&Path,target:Target,expected:Option<String>,text_units:Vec<u16>,current:impl Fn()->Result<(),&'static str>,prepare:impl FnOnce()->Result<(),&'static str>)->Result<Write,&'static str>{
 current()?;let before=crate::private_registry::load_at::<Registry>(path)?;
 let(next,result)=before.registry.write(target,expected.as_deref(),text_units)?;
 if matches!(&result,Write::Conflict{..}){current()?;if !before.unchanged(){return Err("registryChanged");}return Ok(result);}
 crate::private_registry::save_guarded(path,&before.revision,next,|_,_|{current()?;prepare()?;current()},&current)?;Ok(result)
}
pub(super)fn import(path:&Path,batch:Import,current:impl Fn()->Result<(),&'static str>)->Result<ImportReceipt,&'static str>{
 // Several native windows prepare the exact same legacy source at startup.
 // Only prepublication contention is retried; uncertain commits are not.
 let until=std::time::Instant::now()+std::time::Duration::from_secs(1);
 loop{
  match import_once(path,&batch,&current){
   Err("registryBusy"|"registryConflict"|"registryChanged")if std::time::Instant::now()<until=>{current()?;std::thread::sleep(std::time::Duration::from_millis(10));},
   result=>return result,
  }
 }
}
fn import_once(path:&Path,batch:&Import,current:impl Fn()->Result<(),&'static str>)->Result<ImportReceipt,&'static str>{
 current()?;let before=crate::private_registry::load_at::<Registry>(path)?;
 let(next,receipt,changed)=before.registry.import(batch)?;
 if changed{crate::private_registry::save_guarded(path,&before.revision,next,|_,_|current(),&current)?;}else{
  let archive=before.registry.retained_sources.iter().find(|r|r.receipt==receipt).map(|r|super::archives::read(path,r)).transpose()?;
  current()?;if !before.unchanged()||archive.is_some_and(|s|!s.unchanged()){return Err("registryChanged");}
 }
 Ok(receipt)
}
