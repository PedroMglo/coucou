//! Public provenance of the exact copies captured by a turn, never a read grant.
//! Paths, contents, native tokens and model/tool messages are deliberately absent.
use serde::{Deserialize,Serialize};
use std::collections::HashSet;
use crate::assistant_memory::{Library,valid_id};
const LIMIT:usize=10_000;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{pub id:String,pub name:String,pub kind:String,pub size:u64,pub entries:usize,pub sequence:u64}
impl From<&crate::pet_documents::Attachment> for Reference{
    fn from(a:&crate::pet_documents::Attachment)->Self{Self{id:a.id.clone(),name:a.file.name.clone(),kind:a.file.kind.clone(),size:a.file.size,entries:a.file.entries,sequence:a.sequence}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Phase{Pending,Done,Failed,Stopped,Interrupted}
impl Phase{pub fn terminal(value:&str)->Option<Self>{match value{"done"=>Some(Self::Done),"failed"=>Some(Self::Failed),"stopped"=>Some(Self::Stopped),"interrupted"=>Some(Self::Interrupted),_=>None}}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Record{
    pub id:String,pub provider:String,pub pet_id:String,pub conversation_id:String,
    #[serde(default,skip_serializing_if="Option::is_none")]pub task_id:Option<String>,
    pub started:u64,pub updated:u64,pub message_index:usize,pub messages_added:usize,
    pub phase:Phase,pub attachments:Vec<Reference>,
}
pub fn references_valid(values:&[Reference])->bool{
    let mut ids=HashSet::new();let mut size=0u64;
    !values.is_empty()&&values.len()<=32&&values.iter().all(|a|{
        let name=std::path::Path::new(&a.name);size=size.saturating_add(a.size);
        valid_id(&a.id)&&ids.insert(&a.id)&&!a.name.is_empty()&&a.name.len()<=4096&&!a.name.chars().any(char::is_control)
            &&name.components().count()==1&&matches!(name.components().next(),Some(std::path::Component::Normal(_)))
            &&matches!(a.kind.as_str(),"file"|"folder")&&a.entries<=32768&&(a.kind!="file"||a.entries==1)&&a.sequence>0&&size<=100*1024*1024
    })
}
impl Record{
    pub fn captured(provider:&str,cid:&str,index:usize,attachments:Vec<Reference>)->Result<Self,String>{
        let pet=crate::pet_config::integration_pet(provider).ok_or("Este pet não tem identidade para os anexos.")?;
        if !valid_id(cid)||!references_valid(&attachments){return Err("O conjunto de anexos não pode ser registado neste pedido.".into());}
        let now=crate::assistant_memory::now();Ok(Self{id:crate::assistant_memory::new_id(),provider:provider.into(),pet_id:pet.into(),conversation_id:cid.into(),task_id:None,started:now,updated:now,message_index:index,messages_added:0,phase:Phase::Pending,attachments})
    }
    pub fn recover(&mut self){if self.phase==Phase::Pending{self.phase=Phase::Interrupted;self.updated=crate::assistant_memory::now().max(self.started);}}
    pub fn finish(&mut self,phase:Phase,added:Option<usize>){self.phase=phase;self.updated=crate::assistant_memory::now().max(self.started);if let Some(count)=added{self.messages_added=count;}}
}
pub fn validate(library:&Library)->std::io::Result<()>{
    let invalid=||std::io::Error::other("invalid attachment history; original library preserved");let mut ids=HashSet::new();
    for(provider,c)in library.pets.iter().chain(library.archives.iter().flat_map(|(p,rows)|rows.iter().map(move|c|(p,c)))){
        if c.attachment_history.len()>LIMIT{return Err(invalid());}let mut previous=0;let mut occupied=0;
        for r in &c.attachment_history{
            let span=matches!(r.messages_added,0|2)&&r.message_index.checked_add(r.messages_added).is_some_and(|end|end<=c.messages.len())
                &&(r.messages_added==0||(c.messages[r.message_index]["role"]=="user"&&c.messages[r.message_index+1]["role"]=="assistant"));
            if !valid_id(&r.id)||!ids.insert(&r.id)||r.provider!=*provider||r.conversation_id!=c.id||!valid_id(&c.id)
                ||crate::pet_config::integration_pet(provider)!=Some(r.pet_id.as_str())||r.updated<r.started||!span||r.message_index<previous||r.message_index<occupied
                ||r.phase==Phase::Pending&&r.messages_added!=0||!references_valid(&r.attachments)
                ||r.task_id.as_ref().is_some_and(|id|!valid_id(id)||!c.request_history.iter().any(|q|q.task_id==*id&&q.provider==r.provider&&q.conversation_id==r.conversation_id&&q.message_index==Some(r.message_index)&&q.messages_added==Some(r.messages_added))) {return Err(invalid());}
            previous=r.message_index;if r.messages_added==2{occupied=r.message_index+2;}
        }
    }Ok(())
}
pub fn has_records(library:&Library)->bool{library.pets.values().chain(library.archives.values().flatten()).any(|c|!c.attachment_history.is_empty())}
pub fn limit_reached(records:&[Record])->bool{records.len()>=LIMIT}

#[cfg(test)]
mod tests{
    use super::*;use crate::chat::{Chat,SavedConversation};use serde_json::json;use std::{path::PathBuf,sync::Arc};
    struct Temp(PathBuf);
    impl Temp{fn new()->Self{let dir=std::env::temp_dir().join(format!("coucou-attachment-journal-{}",crate::assistant_memory::new_id()));Self(dir)}fn path(&self)->PathBuf{self.0.join("conversations.json")}}
    impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    fn file(id:&str)->Reference{Reference{id:id.into(),name:"igual.txt".into(),kind:"file".into(),size:3,entries:1,sequence:1}}
    fn answer(chat:&Chat,t:&crate::chat::Turn){chat.commit_reply(t,json!({"role":"user","content":"synthetic"}),json!({"role":"assistant","content":"answer"}),"synthetic","answer").unwrap();}
    fn library()->Library{
        let mut record=Record::captured("codex","cid",0,vec![file("a"),file("b")]).unwrap();record.finish(Phase::Done,Some(2));
        Library{pets:std::collections::HashMap::from([("codex".into(),SavedConversation{id:"cid".into(),messages:vec![json!({"role":"user","content":"synthetic"}),json!({"role":"assistant","content":"answer"})],attachment_history:vec![record],..Default::default()})]),..Default::default()}
    }
    #[test]
    fn reference_projection_never_copies_native_paths_or_grants_into_public_history(){
        let a=crate::pet_documents::Attachment{id:"native-copy".into(),provider:"ollama".into(),conversation_id:"cid".into(),sequence:7,file:crate::files::DroppedFile{name:"nota.txt".into(),path:"/private/inbox/not-a-source-path.txt".into(),size:3,kind:"file".into(),entries:1}};
        let r=Reference::from(&a);let json=serde_json::to_value(&r).unwrap();assert_eq!(json["id"],"native-copy");assert_eq!(json["sequence"],7);assert!(json.get("path").is_none());assert!(json.get("provider").is_none());assert!(references_valid(&[r]));
    }
    #[test]
    fn retired_github_metadata_stays_readable_without_admitting_new_work(){
        let d=Temp::new();let mut l=library();let mut c=l.pets.remove("codex").unwrap();let r=&mut c.attachment_history[0];r.provider="integration_github".into();r.pet_id="integration_github".into();l.pets.insert("integration_github".into(),c);crate::assistant_memory::save(&d.path(),&l).unwrap();
        let c=Chat::persistent(d.path());assert_eq!(c.conversation("integration_github").unwrap().attachment_history.len(),1);assert!(c.begin_cli("integration_github",Arc::default()).is_err());
    }
    #[test]
    fn all_llm_families_capture_their_own_complete_set_without_provider_calls(){
        for provider in ["codex","copilot","antigravity","anthropic","openai","google","ollama","lmstudio","integration_spotify"]{
            let dir=Temp::new();let chat=Chat::persistent(dir.path());let turn=chat.begin_cli(provider,Arc::default()).unwrap();let lease=chat.lease(&turn);let cid=chat.conversation_id(provider).unwrap();
            let files=vec![file("a"),file("b")];chat.capture_attachment_history(&turn,files.clone()).unwrap();answer(&chat,&turn);drop(lease);
            let loaded=crate::assistant_memory::load(&dir.path()).unwrap();let r=&loaded.pets[provider].attachment_history[0];assert_eq!(r.attachments,files);assert_eq!(r.provider,provider);assert_eq!(r.pet_id,crate::pet_config::integration_pet(provider).unwrap());assert_eq!(r.conversation_id,cid);assert_eq!(r.phase,Phase::Done);assert_eq!(r.message_index,0);assert_eq!(r.messages_added,2);
            let value=serde_json::to_value(chat.conversation(provider).unwrap()).unwrap();assert_eq!(value["attachmentHistory"][0]["attachments"].as_array().unwrap().len(),2);assert!(value["attachmentHistory"][0].get("taskId").is_none());
        }
    }
    #[test]
    fn same_turn_cannot_change_membership_but_repeated_identical_capture_is_idempotent(){
        let d=Temp::new();let chat=Chat::persistent(d.path());let t=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&t);let original=vec![file("a"),file("b")];chat.capture_attachment_history(&t,original.clone()).unwrap();let before=std::fs::read(d.path()).unwrap();
        chat.capture_attachment_history(&t,original.clone()).unwrap();assert_eq!(std::fs::read(d.path()).unwrap(),before);assert!(chat.capture_attachment_history(&t,vec![file("a")]).is_err());assert_eq!(chat.conversation("codex").unwrap().attachment_history[0].attachments,original);assert_eq!(std::fs::read(d.path()).unwrap(),before);drop(lease);
    }
    #[test]
    fn task_link_is_captured_before_or_after_task_start_and_survives_completion(){
        for first in [true,false]{let d=Temp::new();let c=Chat::persistent(d.path());let t=c.begin_cli("codex",Arc::default()).unwrap();let lease=c.lease(&t);
            if first{c.capture_attachment_history(&t,vec![file("a")]).unwrap();}c.start_task(&t,"synthetic","/tmp","codex","synthetic",None).unwrap();if !first{c.capture_attachment_history(&t,vec![file("a")]).unwrap();}
            let task=c.tasks()[0].id.clone();c.commit_task_reply(&t,json!({"role":"user","content":"synthetic"}),json!({"role":"assistant","content":"answer"}),"synthetic","answer").unwrap();drop(lease);
            let loaded=crate::assistant_memory::load(&d.path()).unwrap();let r=&loaded.pets["codex"].attachment_history[0];assert_eq!(r.task_id.as_deref(),Some(task.as_str()));assert_eq!(r.messages_added,2);assert_eq!(r.phase,Phase::Done);assert_eq!(loaded.pets["codex"].request_history[0].task_id,task);
        }
    }
    #[test]
    fn stopped_and_failed_attempts_have_no_invented_message_pair_and_archive_without_messages(){
        for cancelled in [true,false]{let d=Temp::new();let c=Chat::persistent(d.path());let t=c.begin_cli("openai",Arc::default()).unwrap();let lease=c.lease(&t);let cid=c.conversation_id("openai").unwrap();c.capture_attachment_history(&t,vec![file("a")]).unwrap();if cancelled{t.cancel.store(true,std::sync::atomic::Ordering::Release);}drop(lease);
            let r=c.conversation("openai").unwrap().attachment_history.remove(0);assert_eq!(r.phase,if cancelled{Phase::Stopped}else{Phase::Failed});assert_eq!(r.messages_added,0);assert!(c.history("openai").is_empty());
            c.reset_provider("openai").unwrap();let archive=c.read_conversation("openai",&cid).unwrap();assert_eq!(archive.attachment_history.len(),1);assert_eq!(archive.attachment_history[0].id,r.id);assert_eq!(archive.messages.len(),0);
        }
    }
    #[test]
    fn restart_marks_pending_metadata_interrupted_and_never_rebinds_a_reused_runtime_epoch(){
        let d=Temp::new();let c=Chat::persistent(d.path());let old=c.begin_cli("ollama",Arc::default()).unwrap();c.capture_attachment_history(&old,vec![file("a")]).unwrap();let old_id=c.conversation("ollama").unwrap().attachment_history[0].id.clone();drop(c);
        // Deliberately omit a lease to model process loss before cleanup.
        let restored=Chat::persistent(d.path());assert_eq!(restored.conversation("ollama").unwrap().attachment_history[0].phase,Phase::Interrupted);let new=restored.begin_cli("ollama",Arc::default()).unwrap();let lease=restored.lease(&new);
        assert!(restored.capture_attachment_history(&old,vec![file("stale")]).is_err());restored.capture_attachment_history(&new,vec![file("b")]).unwrap();answer(&restored,&new);drop(lease);
        let records=restored.conversation("ollama").unwrap().attachment_history;assert_eq!(records.len(),2);assert_eq!(records[0].id,old_id);assert_eq!(records[0].phase,Phase::Interrupted);assert_eq!(records[0].attachments[0].id,"a");assert_eq!(records[1].attachments[0].id,"b");assert_ne!(records[0].id,records[1].id);
    }
    #[test]
    fn archiving_and_restoring_keeps_exact_message_anchors_and_record_ids(){
        let d=Temp::new();let c=Chat::persistent(d.path());let t=c.begin_cli("anthropic",Arc::default()).unwrap();let lease=c.lease(&t);let cid=c.conversation_id("anthropic").unwrap();c.capture_attachment_history(&t,vec![file("a")]).unwrap();answer(&c,&t);drop(lease);let original=c.conversation("anthropic").unwrap().attachment_history[0].id.clone();c.reset_provider("anthropic").unwrap();c.restore_conversation("anthropic",&cid).unwrap();
        let next=c.begin_cli("anthropic",Arc::default()).unwrap();let lease=c.lease(&next);c.capture_attachment_history(&next,vec![file("b")]).unwrap();answer(&c,&next);drop(lease);drop(c);let restored=Chat::persistent(d.path());let h=restored.conversation("anthropic").unwrap().attachment_history;assert_eq!(h[0].id,original);assert_eq!(h[0].message_index,0);assert_eq!(h[1].message_index,2);assert_eq!(restored.history("anthropic").len(),4);
    }
    #[test]
    fn metadata_recovery_is_not_a_native_read_grant(){
        let d=Temp::new();let c=Chat::persistent(d.path());let t=c.begin_cli("codex",Arc::default()).unwrap();let lease=c.lease(&t);let cid=c.conversation_id("codex").unwrap();c.capture_attachment_history(&t,vec![file("a")]).unwrap();answer(&c,&t);drop(lease);drop(c);let c=Chat::persistent(d.path());let r=&c.conversation("codex").unwrap().attachment_history[0];
        assert!(crate::pet_documents::contexts("codex",&cid).is_empty());assert!(crate::pet_documents::reference("codex",&cid,&crate::chat::ContextFile{id:r.attachments[0].id.clone(),name:r.attachments[0].name.clone(),path:"/not-a-granted-copy".into(),sequence:1}).is_err());
        let encoded=serde_json::to_string(r).unwrap();for secret in ["path","dropId","token","permission","contents"]{assert!(!encoded.contains(secret));}
    }
    #[test]
    fn a_failed_metadata_write_cannot_publish_a_partial_set_or_consume_a_retry(){
        let d=Temp::new();let c=Chat::persistent(d.path());let old=c.begin_cli("codex",Arc::default()).unwrap();let lease=c.lease(&old);c.capture_attachment_history(&old,vec![file("a")]).unwrap();answer(&c,&old);drop(lease);let before=std::fs::read(d.path()).unwrap();let t=c.begin_cli("codex",Arc::default()).unwrap();let lease=c.lease(&t);
        let backup=d.0.join("previous.json");std::fs::rename(d.path(),&backup).unwrap();std::fs::create_dir(d.path()).unwrap();assert!(c.capture_attachment_history(&t,vec![file("b")]).is_err());assert_eq!(c.conversation("codex").unwrap().attachment_history.len(),1);assert_eq!(std::fs::read(&backup).unwrap(),before);
        std::fs::remove_dir(d.path()).unwrap();std::fs::rename(&backup,d.path()).unwrap();c.capture_attachment_history(&t,vec![file("b")]).unwrap();assert_eq!(c.conversation("codex").unwrap().attachment_history.len(),2);drop(lease);
    }
    #[test]
    fn malformed_or_foreign_reference_sets_never_overwrite_a_valid_library(){
        let d=Temp::new();let original=library();crate::assistant_memory::save(&d.path(),&original).unwrap();let bytes=std::fs::read(d.path()).unwrap();
        for field in ["owner","duplicate","span","phase","path","task"]{
            let mut bad=original.clone();let r=&mut bad.pets.get_mut("codex").unwrap().attachment_history[0];match field{"owner"=>r.pet_id="ai_ollama".into(),"duplicate"=>r.attachments[1].id=r.attachments[0].id.clone(),"span"=>r.message_index=1,"phase"=>r.phase=Phase::Pending,"path"=>r.attachments[0].name="../elsewhere".into(),_=>r.task_id=Some("other-task".into())}
            assert!(crate::assistant_memory::save(&d.path(),&bad).is_err(),"{field}");assert_eq!(std::fs::read(d.path()).unwrap(),bytes);
        }
    }
    #[test]
    fn version5_is_required_only_for_new_metadata_and_unknown_grants_or_native_paths_are_rejected(){
        let d=Temp::new();let c=library();crate::assistant_memory::save(&d.path(),&c).unwrap();let value:serde_json::Value=serde_json::from_slice(&std::fs::read(d.path()).unwrap()).unwrap();assert_eq!(value["version"],5);
        let mut old=value.clone();old["version"]=json!(4);std::fs::write(d.path(),serde_json::to_vec(&old).unwrap()).unwrap();assert!(crate::assistant_memory::load(&d.path()).is_err());
        for field in ["path","grant"]{let mut bad=value.clone();bad["pets"]["codex"]["attachment_history"][0]["attachments"][0][field]=json!("not-authority");let bytes=serde_json::to_vec(&bad).unwrap();std::fs::write(d.path(),&bytes).unwrap();let c=Chat::persistent(d.path());assert!(c.begin_cli("codex",Arc::default()).is_err());assert_eq!(std::fs::read(d.path()).unwrap(),bytes);}
        let mut legacy=c;legacy.pets.get_mut("codex").unwrap().attachment_history.clear();crate::assistant_memory::save(&d.path(),&legacy).unwrap();let saved:serde_json::Value=serde_json::from_slice(&std::fs::read(d.path()).unwrap()).unwrap();assert_eq!(saved["version"],4);assert!(crate::assistant_memory::load(&d.path()).unwrap().pets["codex"].attachment_history.is_empty());
    }
    #[test]
    fn completed_attachment_turn_cannot_be_committed_twice(){
        let d=Temp::new();let c=Chat::persistent(d.path());let t=c.begin_cli("codex",Arc::default()).unwrap();let lease=c.lease(&t);c.capture_attachment_history(&t,vec![file("a")]).unwrap();answer(&c,&t);assert!(c.commit_reply(&t,json!({}),json!({}),"repeat","repeat").is_err());assert_eq!(c.history("codex").len(),2);drop(lease);
    }
}
