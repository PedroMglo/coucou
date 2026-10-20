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

#[cfg(test)]
mod request_history_storage_tests {
    use super::*;
    use crate::chat::{Chat,RequestRecord};
    use serde_json::json;
    fn path()->std::path::PathBuf{std::env::temp_dir().join(format!("coucou-history-storage-{}",new_id())).join("library.json")}
    fn fixture()->Library{
        let record=RequestRecord{task_id:"pruned-task".into(),conversation_id:"original-cid".into(),provider:"codex".into(),pet_id:"agent_codex".into(),sequence:1,query:"Original public request".into(),phase:"stopped".into(),started:1,updated:2,source:"captured".into(),message_index:Some(0),messages_added:Some(0)};
        Library{pets:HashMap::from([("codex".into(),SavedConversation{id:"original-cid".into(),request_history:vec![record],..Default::default()})]),..Default::default()}
    }
    #[test]fn disk_v4_prevents_old_readers_from_silently_dropping_request_history(){
        let path=path();save(&path,&fixture()).unwrap();let value:serde_json::Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["version"],4);assert!(value["pets"]["codex"]["messages"].as_array().unwrap().is_empty());
        assert_eq!(value["pets"]["codex"]["request_history"][0]["taskId"],"pruned-task");
        assert_eq!(load(&path).unwrap().pets["codex"].request_history.len(),1);std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]fn invalid_history_rejects_without_replacing_the_original_file(){
        let path=path();let original=fixture();save(&path,&original).unwrap();let before=std::fs::read(&path).unwrap();
        for field in ["pet","cid","provider","sequence","source","phase","anchor","identity","duplicate"] {
            let mut invalid=original.clone();let row=&mut invalid.pets.get_mut("codex").unwrap().request_history[0];
            match field{"pet"=>row.pet_id="integration_github".into(),"cid"=>row.conversation_id="foreign-cid".into(),"provider"=>row.provider="copilot".into(),"sequence"=>row.sequence=7,"source"=>row.source="stdout".into(),"phase"=>row.phase="resumed".into(),"anchor"=>row.message_index=Some(1),"identity"=>row.task_id="../bad".into(),"duplicate"=>{let mut another=row.clone();another.sequence=2;invalid.pets.get_mut("codex").unwrap().request_history.push(another)},_=>unreachable!()}
            assert!(save(&path,&invalid).is_err(),"{field}");assert_eq!(std::fs::read(&path).unwrap(),before,"{field}");
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]fn unknown_private_fields_fail_closed_and_never_enter_the_event_dto(){
        let path=path();save(&path,&fixture()).unwrap();let mut value:serde_json::Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["pets"]["codex"]["request_history"][0]["stdout"]=json!("must never be treated as public dialogue");
        std::fs::write(&path,serde_json::to_vec(&value).unwrap()).unwrap();let before=std::fs::read(&path).unwrap();
        assert!(load(&path).is_err());let chat=Chat::persistent(path.clone());assert!(chat.conversations("codex","").is_err());assert!(chat.reserve_conversation("codex",None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(),before);std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]fn no_event_is_inferred_for_legacy_messages_without_owned_task_provenance(){
        let path=path();for version in [1,2,3]{
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path,serde_json::to_vec(&json!({"version":version,"pets":{"codex":{"id":"legacy-cid","messages":[{"role":"user","content":"original"},{"role":"assistant","content":"actual answer"}],"session":null}}})).unwrap()).unwrap();
            let chat=Chat::persistent(path.clone());let view=chat.conversation("codex").unwrap();assert!(view.request_history.is_empty());assert_eq!(view.messages.len(),2);
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}

#[cfg(test)]
mod request_history_integrity_tests {
    use super::*;
    use crate::chat::{Chat,RequestRecord};
    use serde_json::json;
    fn path()->std::path::PathBuf{std::env::temp_dir().join(format!("coucou-history-integrity-{}",new_id())).join("library.json")}
    fn record(id:&str,sequence:u64,at:usize)->RequestRecord{RequestRecord{task_id:id.into(),conversation_id:"cid".into(),provider:"codex".into(),pet_id:"agent_codex".into(),sequence,query:"synthetic".into(),phase:"stopped".into(),started:1,updated:2,source:"captured".into(),message_index:Some(at),messages_added:Some(0)}}
    #[test]fn public_message_filtering_cannot_silently_move_a_request_anchor(){
        let path=path();let c=SavedConversation{id:"cid".into(),messages:vec![json!({"role":"user","content":"actual"}),json!({"role":"tool","content":"private"})],request_history:vec![record("task",1,2)],..Default::default()};
        assert!(save(&path,&Library{pets:HashMap::from([("codex".into(),c)]),..Default::default()}).is_err());assert!(!path.exists());
    }
    #[test]fn regressing_message_positions_reject_instead_of_reordering_history(){
        let path=path();let c=SavedConversation{id:"cid".into(),messages:vec![json!({"role":"user","content":"actual"})],request_history:vec![record("first",1,1),record("second",2,0)],..Default::default()};
        assert!(save(&path,&Library{pets:HashMap::from([("codex".into(),c)]),..Default::default()}).is_err());assert!(!path.exists());
    }
    #[test]fn ambiguous_legacy_task_ids_do_not_manufacture_cross_pet_history_or_block_future_saves(){
        let path=path();let mut pets=HashMap::new();
        for provider in ["codex","copilot"]{
            let cid=format!("{provider}-cid");let mut task=crate::assistant_tasks::Task::new(provider,"legacy synthetic","/tmp",provider,"fixture",1);task.id="ambiguous-global-id".into();task.conversation_id=Some(cid.clone());task.update("stopped","");
            pets.insert(provider.into(),SavedConversation{id:cid,tasks:vec![task],..Default::default()});
        }
        save(&path,&Library{pets,..Default::default()}).unwrap();let chat=Chat::persistent(path.clone());
        assert!(chat.conversation("codex").unwrap().request_history.is_empty());assert!(chat.conversation("copilot").unwrap().request_history.is_empty());
        chat.reset_provider("codex").unwrap();assert!(load(&path).is_ok());std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}

#[cfg(test)]
mod captured_anchor_validation_tests {
    use super::*;
    use crate::chat::{Chat,RequestRecord};
    use serde_json::json;
    fn path()->std::path::PathBuf{std::env::temp_dir().join(format!("coucou-anchor-raw-{}",new_id())).join("library.json")}
    fn messages()->Vec<serde_json::Value>{["A","B","C"].into_iter().flat_map(|name|[json!({"role":"user","content":name}),json!({"role":"assistant","content":format!("answer {name}")})]).collect()}
    fn record(id:&str,sequence:u64,at:usize,count:usize)->RequestRecord{RequestRecord{task_id:id.into(),conversation_id:"cid".into(),provider:"codex".into(),pet_id:"agent_codex".into(),sequence,query:"original A".into(),phase:if count==2{"done"}else{"stopped"}.into(),started:1,updated:2,source:"captured".into(),message_index:Some(at),messages_added:Some(count)}}
    fn conversation(prefix:usize,count:usize)->SavedConversation{
        let mut all=vec![json!({"role":"tool","content":"private"});prefix];all.extend(messages());
        SavedConversation{id:"cid".into(),messages:all,request_history:vec![record("request-a",1,prefix,count)],..Default::default()}
    }
    fn library(c:SavedConversation,archived:bool)->Library{
        if archived{Library{pets:HashMap::from([("codex".into(),SavedConversation{id:"current-cid".into(),..Default::default()})]),archives:HashMap::from([("codex".into(),vec![c])])}}
        else{Library{pets:HashMap::from([("codex".into(),c)]),..Default::default()}}
    }
    #[test]fn load_rejects_raw_private_prefix_even_when_shifted_anchor_is_in_range_and_another_valid_pair(){
        let path=path();std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        for archived in [false,true]{for prefix in [1,2]{for count in [0,2]{
            let c=conversation(prefix,count);
            assert!(prefix+count<=messages().len(),"range-only validation would accept after filtering");
            if prefix==2&&count==2{assert!(valid_request_span(&messages(),prefix,count),"even role checks alone would select B instead of A");}
            let raw=library(c,archived);let disk=Disk{version:4,pets:raw.pets,archives:raw.archives};let original=serde_json::to_vec(&disk).unwrap();std::fs::write(&path,&original).unwrap();
            assert!(load(&path).is_err(),"archived={archived} prefix={prefix} count={count}");
            let chat=Chat::persistent(path.clone());assert!(chat.reserve_conversation("codex",None).is_err());
            assert_eq!(std::fs::read(&path).unwrap(),original);
        }}}
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]fn save_rejects_the_same_in_range_shift_without_replacing_existing_data(){
        let path=path();save(&path,&library(conversation(0,2),false)).unwrap();let original=std::fs::read(&path).unwrap();
        for archived in [false,true]{for prefix in [1,2]{for count in [0,2]{
            assert!(save(&path,&library(conversation(prefix,count),archived)).is_err());assert_eq!(std::fs::read(&path).unwrap(),original);
        }}}
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]fn two_messages_must_be_a_real_user_assistant_pair_in_that_order(){
        for roles in [["assistant","user"],["user","user"],["assistant","assistant"]]{
            let mut c=conversation(0,2);c.messages[0]["role"]=json!(roles[0]);c.messages[1]["role"]=json!(roles[1]);
            assert!(validate_request_history(&library(c,false)).is_err());
        }
        assert!(validate_request_history(&library(conversation(0,2),false)).is_ok());
    }
    #[test]fn captured_spans_cannot_overlap_or_insert_later_requests_inside_a_prior_pair(){
        for second in [record("request-b",2,0,2),record("request-b",2,1,0)]{
            let mut c=conversation(0,2);c.request_history.push(second);assert!(validate_request_history(&library(c,false)).is_err());
        }
        let mut c=conversation(0,2);c.request_history.push(record("request-b",2,2,2));assert!(validate_request_history(&library(c,false)).is_ok());
        let mut c=conversation(0,0);c.request_history.push(record("request-b",2,0,2));assert!(validate_request_history(&library(c,false)).is_ok(),"several stopped attempts may precede a later actual pair in the same gap");
    }
    #[test]fn captured_history_rejects_noncanonical_public_shapes_but_unanchored_legacy_stays_compatible(){
        for bad in [json!({"role":"assistant","content":[{"type":"thinking","text":"private"}]}),json!({"role":"user","content":"public","tool_output":"private"}),json!({"role":"user","content":42}),json!(null)]{
            let mut c=conversation(0,0);c.messages.insert(0,bad.clone());assert!(validate_request_history(&library(c,false)).is_err());
            let mut c=conversation(0,0);c.messages.insert(0,bad);let r=&mut c.request_history[0];r.source="taskJournal".into();r.message_index=None;r.messages_added=None;
            assert!(validate_captured_messages(&c).is_ok(),"unknown legacy placement does not acquire invented anchors");
        }
    }
}
