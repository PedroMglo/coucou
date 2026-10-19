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

pub fn load(path:&Path)->std::io::Result<Library> {
    let result=(||->std::io::Result<_>{
        let mut options=std::fs::OpenOptions::new();options.read(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW);}
        let file=options.open(path)?;
        if !file.metadata()?.is_file(){return Err(std::io::Error::other("not a regular file"));}
        let mut bytes=Vec::new();file.take(MAX_BYTES+1).read_to_end(&mut bytes)?;
        if bytes.len() as u64>MAX_BYTES{return Err(std::io::Error::other("memory too large"));}
        let disk:Disk=serde_json::from_slice(&bytes)?;
        if !matches!(disk.version,1|2|3) {return Err(std::io::Error::other("unknown memory version"));}
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
        Ok(Library{pets,archives})
    })();
    match result {Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(Library::default()),other=>other}
}

pub fn save(path:&Path,library:&Library)->std::io::Result<()> {
    let pets=library.pets.iter().filter(|(p,_)|supported(p)).map(|(p,c)|(p.clone(),public(c.clone()))).collect();
    let mut archives=HashMap::new();
    for (p,conversations) in library.archives.iter().filter(|(p,_)|supported(p)) {
        if conversations.len()>MAX_ARCHIVES {return Err(std::io::Error::other("too many archived conversations"));}
        archives.insert(p.clone(),conversations.iter().map(|c|{let mut c=public(c.clone());c.tasks.clear();c}).collect());
    }
    let disk=Disk{version:3,pets,archives};
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

#[cfg(test)]mod tests {
    use super::*;
    use serde_json::json;
    #[test]fn retains_public_dialogue_and_session_but_never_native_tool_blocks(){
        let c=public(SavedConversation{messages:vec![json!({"role":"user","content":"Cria o projeto"}),json!({"role":"assistant","content":[{"type":"thinking","text":"private"}]}),json!({"role":"tool","content":"secret arguments"}),json!({"role":"assistant","content":"Concluído"})],session:Some(crate::chat::CliSession{id:"owned".into(),cwd:"/tmp/project".into(),full_access:false}),..Default::default()});
        assert_eq!(c.messages.len(),2);assert_eq!(c.session.unwrap().id,"owned");
        assert!(!serde_json::to_string(&c.messages).unwrap().contains("private"));
    }
    #[test]fn keeps_the_public_archive_separate_from_context_and_rejects_invalid_session_paths(){
        let c=public(SavedConversation{messages:(0..200).map(|i|json!({"role":if i%2==0{"user"}else{"assistant"},"content":"a".repeat(15000)})).collect(),session:Some(crate::chat::CliSession{id:"id".into(),cwd:"relative".into(),full_access:true}),..Default::default()});
        assert_eq!(c.messages.len(),200);assert_eq!(c.messages[0]["content"].as_str().unwrap().len(),15000);assert!(c.session.is_none());
        let selected=context(&c.messages);assert!(!selected.is_empty());assert!(selected.len()<=MAX_CONTEXT_MESSAGES);assert!(serde_json::to_vec(&selected).unwrap().len()<=MAX_CONTEXT_BYTES);
    }
}
