//! Complete public dialogue archive and owned session references.
//! This is application data, never a host configuration or credential backup.
use std::{collections::HashMap, io::{Read, Write}, path::Path};
use serde::{Deserialize, Serialize};
use crate::chat::SavedConversation;

pub const MAX_BYTES:u64=128*1024*1024;
pub const MAX_ARCHIVES:usize=100;
pub const MAX_CONTEXT_MESSAGES:usize=80;
pub const MAX_CONTEXT_BYTES:usize=256*1024;
pub fn supported(provider:&str)->bool {crate::pet_config::service(provider) || matches!(provider,"codex"|"copilot"|"antigravity"|"gemini-cli"|"claude-cli"|"ollama"|"lmstudio"|"custom"|"anthropic"|"openai"|"google"|"openrouter")}
#[derive(Default,Clone,Serialize,Deserialize)]
pub struct Library {
    pub pets:HashMap<String,SavedConversation>,
    #[serde(default)] pub archives:HashMap<String,Vec<SavedConversation>>,
}
#[derive(Serialize,Deserialize)]
struct Disk {
    version:u32,pets:HashMap<String,SavedConversation>,
    #[serde(default)] archives:HashMap<String,Vec<SavedConversation>>,
}

pub fn now()->u64 {std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64}
pub fn new_id()->String {
    static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
    let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{nonce:x}-{:x}-{:x}",std::process::id(),NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed))
}
pub fn valid_id(id:&str)->bool {!id.is_empty() && id.len()<=128 && id.bytes().all(|c|c.is_ascii_alphanumeric() || c==b'-')}

fn public(mut c:SavedConversation)->SavedConversation {
    if !valid_id(&c.id) {c.id=new_id();}
    if c.task_notice_id.as_deref().is_some_and(|id|!valid_id(id)){c.task_notice_id=None;}
    if c.created==0 {c.created=now();}
    if c.updated==0 {c.updated=c.created;}
    c.tasks=c.tasks.into_iter().rev().take(10).filter(|t|supported(&t.provider)).collect();c.tasks.reverse();
    for task in &mut c.tasks {task.bound();}
    c.messages=c.messages.into_iter().filter_map(|m| {
        let role=m["role"].as_str()?;
        if !matches!(role,"user"|"assistant"){return None;}
        Some(serde_json::json!({"role":role,"content":m["content"].as_str()?}))
    }).collect();
    c.session=c.session.filter(|s|!s.id.is_empty() && s.id.len()<=512 && s.cwd.len()<=4096 && Path::new(&s.cwd).is_absolute());
    c
}

/// Context is a bounded tail; it is never used as the public archive.
/// Keep whole native messages and begin at a user message, without cutting text.
pub fn context(messages:&[serde_json::Value])->Vec<serde_json::Value> {
    let mut used=2usize;let mut first=messages.len();
    for(index,message)in messages.iter().enumerate().rev().take(MAX_CONTEXT_MESSAGES){
        if Count(&mut used).write_all(b",").is_err(){break;}
        let size=serde_json::to_writer(Count(&mut used),message);
        if size.is_err(){break;}first=index;
    }
    while first<messages.len()&&messages[first]["role"]!="user"{first+=1;}
    messages[first..].to_vec()
}
struct Count<'a>(&'a mut usize);
impl Write for Count<'_>{
    fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{
        if self.0.saturating_add(bytes.len())>MAX_CONTEXT_BYTES{return Err(std::io::Error::other("context full"));}
        *self.0+=bytes.len();Ok(bytes.len())
    }
    fn flush(&mut self)->std::io::Result<()>{Ok(())}
}

/// Captured positions refer to the original array. Filtering even one private
/// entry could change which public pair a still-in-range anchor refers to.
fn validate_captured_messages(c:&SavedConversation)->std::io::Result<()> {
    if c.request_history.iter().any(|r|r.source=="captured")&&c.messages.iter().any(|m|
        !m.as_object().is_some_and(|o|o.len()==2&&o.contains_key("role")&&o.contains_key("content"))
        ||!matches!(m["role"].as_str(),Some("user"|"assistant"))||!m["content"].is_string()){
        return Err(std::io::Error::other("captured history requires original public messages; array preserved"));
    }
    Ok(())
}
fn valid_request_span(messages:&[serde_json::Value],at:usize,count:usize)->bool {
    matches!(count,0|2)&&at.checked_add(count).is_some_and(|end|end<=messages.len())
        &&(count==0||(messages[at]["role"]=="user"&&messages[at+1]["role"]=="assistant"))
}

fn validate_request_history(library:&Library)->std::io::Result<()> {
    let mut seen=std::collections::HashSet::new();
    let tasks:Vec<_>=library.pets.values().flat_map(|c|c.tasks.iter()).collect();
    for(provider,c)in library.pets.iter().chain(library.archives.iter().flat_map(|(p,rows)|rows.iter().map(move|c|(p,c)))){
        validate_captured_messages(c)?;
        let mut previous_anchor=None;let mut occupied_until=0;
        for(index,r)in c.request_history.iter().enumerate(){
            let anchor=match(r.message_index,r.messages_added){
                (Some(at),Some(count))=>r.source=="captured"&&at>=occupied_until&&valid_request_span(&c.messages,at,count),
                (None,None)=>r.source=="taskJournal",
                _=>false,
            };
            let ordered=r.message_index.is_none_or(|at|previous_anchor.is_none_or(|before|at>=before));
            if let Some(at)=r.message_index{previous_anchor=Some(at);}
            if anchor&&r.messages_added==Some(2){occupied_until=r.message_index.unwrap()+2;}
            if !ordered||!valid_id(&r.task_id)||!valid_id(&c.id)||r.conversation_id!=c.id||r.provider!=*provider
                ||crate::pet_config::pet_id(provider)!=Some(r.pet_id.as_str())
                ||r.sequence!=index as u64+1||!anchor||!matches!(r.phase.as_str(),"pending"|"done"|"failed"|"stopped"|"interrupted")
                ||tasks.iter().any(|t|t.id==r.task_id&&(t.provider!=r.provider||t.conversation_id.as_deref()!=Some(r.conversation_id.as_str())))
                ||!seen.insert(r.task_id.clone()) {
                return Err(std::io::Error::other("invalid public request history; original library preserved"));
            }
        }
    }
    Ok(())
}

pub fn load(path:&Path)->std::io::Result<Library> {
    let result=(||->std::io::Result<_>{
        let mut options=std::fs::OpenOptions::new();options.read(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW);}
        let file=options.open(path)?;
        if !file.metadata()?.is_file(){return Err(std::io::Error::other("not a regular file"));}
        let mut bytes=Vec::new();file.take(MAX_BYTES+1).read_to_end(&mut bytes)?;
        if bytes.len() as u64>MAX_BYTES{return Err(std::io::Error::other("memory too large"));}
        let disk:Disk=serde_json::from_slice(&bytes)?;
        if !matches!(disk.version,1|2|3|4) {return Err(std::io::Error::other("unknown memory version"));}
        // Validate before public() can remove or normalize any array entry.
        for c in disk.pets.values().chain(disk.archives.values().flatten()){validate_captured_messages(c)?;}
        let pets:HashMap<_,_>=disk.pets.into_iter().filter(|(p,_)|supported(p)).map(|(p,c)|{let mut c=public(c);c.tasks.retain(|t|t.provider==p && t.valid());(p,c)}).collect();
        let mut archives=HashMap::new();
        for (provider,conversations) in disk.archives.into_iter().filter(|(p,_)|supported(p)) {
            if conversations.len()>MAX_ARCHIVES {return Err(std::io::Error::other("too many archived conversations"));}
            let mut seen=std::collections::HashSet::new();
            if let Some(current)=pets.get(&provider) {seen.insert(current.id.clone());}
            let mut saved=Vec::new();
            for conversation in conversations {
                let mut c=public(conversation);c.tasks.clear();
                if !seen.insert(c.id.clone()) {return Err(std::io::Error::other("duplicate conversation id"));}
                saved.push(c);
            }
            archives.insert(provider,saved);
        }
        let library=Library{pets,archives};validate_request_history(&library)?;Ok(library)
    })();
    match result {Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(Library::default()),other=>other}
}

pub fn save(path:&Path,library:&Library)->std::io::Result<()> {
    // Never truncate or silently discard malformed provenance. An old version
    // cannot read v4 and therefore cannot overwrite data it does not understand.
    validate_request_history(library)?;
    let pets=library.pets.iter().filter(|(p,_)|supported(p)).map(|(p,c)|(p.clone(),public(c.clone()))).collect();
    let mut archives=HashMap::new();
    for (p,conversations) in library.archives.iter().filter(|(p,_)|supported(p)) {
        if conversations.len()>MAX_ARCHIVES {return Err(std::io::Error::other("too many archived conversations"));}
        archives.insert(p.clone(),conversations.iter().map(|c|{let mut c=public(c.clone());c.tasks.clear();c}).collect());
    }
    let normalized=Library{pets,archives};validate_request_history(&normalized)?;
    let disk=Disk{version:4,pets:normalized.pets,archives:normalized.archives};
    let dir=path.parent().ok_or_else(||std::io::Error::other("missing memory directory"))?;
    std::fs::create_dir_all(dir)?;
    if std::fs::symlink_metadata(dir)?.file_type().is_symlink() || std::fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()) {return Err(std::io::Error::other("memory symlink rejected"));}
    #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(dir,std::fs::Permissions::from_mode(0o700))?;}
    let temp=path.with_extension(format!("json.{}.tmp",new_id()));
    let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600).custom_flags(libc::O_NOFOLLOW);}
    let result=(|| {
        let mut file=options.open(&temp)?;
        {let mut limited=Limited{inner:&mut file,written:0};serde_json::to_writer(&mut limited,&disk)?;}
        file.sync_all()?;drop(file);
        std::fs::rename(&temp,path)?;
        #[cfg(unix)] std::fs::File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err(){let _=std::fs::remove_file(temp);}
    result
}

struct Limited<W>{inner:W,written:u64}
impl<W:Write> Write for Limited<W>{
    fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{
        if self.written.saturating_add(bytes.len() as u64)>MAX_BYTES{return Err(std::io::Error::other("library exceeds 128 MiB; previous file preserved"));}
        let size=self.inner.write(bytes)?;self.written+=size as u64;Ok(size)
    }
    fn flush(&mut self)->std::io::Result<()>{self.inner.flush()}
}

