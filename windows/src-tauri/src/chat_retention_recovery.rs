//! Private recovery material precedes local history removal. Pending material
//! protects inbox copies until a separate native review resolves recovery.
use super::*;
use std::{fs::{File,OpenOptions},io::{Read,Write},path::PathBuf};
const MAX_BYTES:u64=crate::assistant_memory::MAX_BYTES+1024*1024;
#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Data{version:u32,id:String,target:Target,conversation:SavedConversation,tasks:Vec<crate::assistant_tasks::Task>}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Summary{pub id:String,pub revision:String,pub target:Target}
#[derive(PartialEq,Eq)]
struct Stamp{size:u64,modified:Option<std::time::SystemTime>,#[cfg(unix)]native:(u64,u64,i64,i64,u32,u32,u64)}
fn stamp(m:&std::fs::Metadata)->Stamp{
    #[cfg(unix)]use std::os::unix::fs::MetadataExt;
    Stamp{size:m.len(),modified:m.modified().ok(),#[cfg(unix)]native:(m.dev(),m.ino(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink())}
}
fn private_file(m:&std::fs::Metadata)->bool{
    if !m.is_file(){return false;}
    #[cfg(unix)]{use std::os::unix::fs::MetadataExt;if m.uid()!=unsafe{libc::geteuid()}||m.mode()&0o077!=0||m.nlink()!=1{return false;}}true
}
pub(super) struct Backup{pub id:String,path:PathBuf,file:File,stamp:Stamp}
pub(super) struct Opened{pub backup:Backup,pub summary:Summary,pub conversation:SavedConversation,pub tasks:Vec<crate::assistant_tasks::Task>}
fn directory(library:&Path)->Result<PathBuf,String>{Ok(library.parent().ok_or(CHANGED)?.join("purge-recovery"))}
fn private_directory(path:&Path)->Result<(),String>{
    let meta=std::fs::symlink_metadata(path).map_err(|_|CHANGED)?;if !meta.is_dir(){return Err(CHANGED.into());}
    #[cfg(unix)]{use std::os::unix::fs::MetadataExt;if meta.uid()!=unsafe{libc::geteuid()}||meta.mode()&0o077!=0{return Err(CHANGED.into());}}Ok(())
}
fn sync_dir(path:&Path)->std::io::Result<()>{
    #[cfg(unix)]{File::open(path)?.sync_all()}
    #[cfg(not(unix))]{let _=path;Ok(())}
}
struct Limited<'a>{file:&'a mut File,bytes:u64}
impl Write for Limited<'_>{
    fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{if self.bytes.saturating_add(bytes.len()as u64)>MAX_BYTES{return Err(std::io::Error::other("recovery size limit"));}let n=self.file.write(bytes)?;self.bytes+=n as u64;Ok(n)}
    fn flush(&mut self)->std::io::Result<()>{self.file.flush()}
}
impl Backup{
    pub(super) fn stage(library:&Path,target:&Target,conversation:SavedConversation,tasks:Vec<crate::assistant_tasks::Task>)->Result<Self,String>{
        let directory=directory(library)?;let mut builder=std::fs::DirBuilder::new();builder.recursive(true);#[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
        builder.create(&directory).map_err(|_|CHANGED)?;private_directory(&directory)?;
        if ids(library)?.len()>=128{return Err("Resolve uma recuperação antiga antes de eliminar outra conversa.".into());}
        let id=crate::assistant_memory::new_id();let path=directory.join(format!("{id}.json"));let mut options=OpenOptions::new();options.read(true).write(true).create_new(true);
        #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC);}
        let mut file=options.open(&path).map_err(|_|CHANGED)?;let data=Data{version:1,id:id.clone(),target:target.clone(),conversation,tasks};
        let result=(||{serde_json::to_writer(Limited{file:&mut file,bytes:0},&data).map_err(|_|CHANGED)?;file.sync_all().map_err(|_|CHANGED)?;sync_dir(&directory).map_err(|_|CHANGED)?;Ok::<_,String>(())})();
        let stamp=stamp(&file.metadata().map_err(|_|CHANGED)?);let backup=Self{id,path,file,stamp};
        if let Err(error)=result{let _=backup.remove();return Err(error);}Ok(backup)
    }
    pub(super) fn open(library:&Path,id:&str)->Result<Opened,String>{
        if !crate::assistant_memory::valid_id(id){return Err(CHANGED.into());}let directory=directory(library)?;private_directory(&directory)?;let path=directory.join(format!("{id}.json"));
        let mut options=OpenOptions::new();options.read(true);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC);}
        let file=options.open(&path).map_err(|_|CHANGED)?;let meta=file.metadata().map_err(|_|CHANGED)?;if !private_file(&meta)||meta.len()>MAX_BYTES{return Err(CHANGED.into());}let before=stamp(&meta);
        let mut bytes=Vec::new();(&file).take(MAX_BYTES+1).read_to_end(&mut bytes).map_err(|_|CHANGED)?;if bytes.len()as u64>MAX_BYTES{return Err(CHANGED.into());}
        let data:Data=serde_json::from_slice(&bytes).map_err(|_|CHANGED)?;
        if data.version!=1||data.id!=id||data.conversation.id!=data.target.conversation_id||!crate::assistant_memory::supported(&data.target.provider)
            ||data.tasks.iter().any(|t|!t.terminal()||t.provider!=data.target.provider||t.conversation_id.as_deref()!=Some(data.target.conversation_id.as_str()))
            ||digest(&(&data.target.provider,&data.target.conversation_id,data.target.current,&data.conversation,&data.tasks))?!=data.target.revision{return Err(CHANGED.into());}
        let backup=Self{id:id.into(),path,file,stamp:before};if !backup.current(){return Err(CHANGED.into());}
        let revision=ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect();Ok(Opened{backup,summary:Summary{id:id.into(),revision,target:data.target},conversation:data.conversation,tasks:data.tasks})
    }
    fn current(&self)->bool{
        private_directory(self.path.parent().unwrap()).is_ok()&&self.file.metadata().is_ok_and(|m|private_file(&m)&&stamp(&m)==self.stamp)&&std::fs::symlink_metadata(&self.path).is_ok_and(|m|private_file(&m)&&stamp(&m)==self.stamp)
    }
    pub(super) fn remove(&self)->Result<(),String>{
        if !self.current(){return Err(CHANGED.into());}std::fs::remove_file(&self.path).map_err(|_|CHANGED)?;sync_dir(self.path.parent().ok_or(CHANGED)?).map_err(|_|CHANGED)?;Ok(())
    }
}
fn ids(library:&Path)->Result<Vec<String>,String>{
    let directory=directory(library)?;match std::fs::symlink_metadata(&directory){Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(vec![]),Err(_)=>return Err(CHANGED.into()),Ok(_)=>{}}
    private_directory(&directory)?;let mut ids=Vec::new();
    for entry in std::fs::read_dir(directory).map_err(|_|CHANGED)?{
        let name=entry.map_err(|_|CHANGED)?.file_name();let name=name.to_str().ok_or(CHANGED)?;let id=name.strip_suffix(".json").filter(|id|crate::assistant_memory::valid_id(id)).ok_or("Um registo de recuperação é inválido e foi preservado.")?;
        ids.push(id.into());if ids.len()>128{return Err("Há demasiadas operações do histórico por recuperar.".into());}
    }ids.sort();Ok(ids)
}
pub(super) fn pending(library:&Path)->Result<bool,String>{Ok(!ids(library)?.is_empty())}
pub(super) fn inventory(library:&Path)->Result<Vec<Summary>,String>{ids(library)?.into_iter().map(|id|Backup::open(library,&id).map(|opened|opened.summary)).collect()}
