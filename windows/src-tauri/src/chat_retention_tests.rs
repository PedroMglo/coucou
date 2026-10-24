use super::*;
#[test]
fn passive_capability_health_does_not_create_or_identify_conversations(){
    let chat=Chat::default();let before={let all=chat.inner.lock().unwrap();(all.pets.len(),all.archives.len(),all.next)};
    for _ in 0..2{let(healthy,busy)=chat.capability_health();assert!(healthy);assert!(busy.is_empty());}
    let after={let all=chat.inner.lock().unwrap();(all.pets.len(),all.archives.len(),all.next)};assert_eq!(before,after);
}
struct Fixture{files:crate::pet_documents::fixtures::Files,chat:Chat,path:std::path::PathBuf}
impl Fixture{
    fn new()->Self{let files=crate::pet_documents::fixtures::Files::new();let path=files.root.join("conversations.json");let chat=Chat::persistent(path.clone());Self{files,chat,path}}
    fn seed(&self,provider:&str,text:&str,attachment:Option<&str>,task:bool)->String{
        // Journaling APIs only: no provider, CLI, model or account is invoked.
        let turn=self.chat.begin_cli(provider,Default::default()).unwrap();let lease=self.chat.lease(&turn);let cid=self.chat.usage_origin(&turn).unwrap().0;
        if let Some(id)=attachment{self.chat.capture_attachment_history(&turn,vec![crate::attachment_history::Reference{id:id.into(),name:"same.txt".into(),kind:"file".into(),size:3,entries:1,sequence:1}]).unwrap();}
        if task{self.chat.start_task(&turn,text,self.files.root.to_str().unwrap(),"codex","synthetic-model",None).unwrap();self.chat.commit_task_reply(&turn,json!({"role":"user","content":text}),json!({"role":"assistant","content":"reply"}),text,"reply").unwrap();}
        else{self.chat.commit(&turn,json!({"role":"user","content":text}),json!({"role":"assistant","content":"reply"}),text,"reply").unwrap();}
        drop(lease);cid
    }
}
fn deadline()->Instant{Instant::now()+std::time::Duration::from_secs(120)}
#[test]
fn inactive_records_survive_normal_delete_and_uncertain_commit_restore(){
    for durable in [true,false]{
        let f=Fixture::new();let cid=f.seed("codex","known original",None,false);
        let opaque=json!({"futureShape":["preserved",{"session":"foreign","full_access":true}]});
        let mut original:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();
        original["pets"]["future-adapter"]=opaque.clone();std::fs::write(&f.path,serde_json::to_vec(&original).unwrap()).unwrap();
        let chat=Chat::persistent(f.path.clone());let target=chat.purge_target("codex",&cid).unwrap();
        let receipt=chat.purge_using(&target,deadline(),|path,library,current|{
            let saved=crate::assistant_memory::save_checked(path,library,current)?;
            Ok(crate::assistant_memory::SaveReceipt{durable:durable&&saved.durable})
        }).unwrap();
        let disk:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();
        assert_eq!(disk["unavailableHistory"]["pets"]["future-adapter"],json!([opaque.clone()]));
        assert!(Chat::persistent(f.path.clone()).read_conversation("codex",&cid).is_err());
        if durable{assert!(receipt.recovery_id.is_none());assert!(receipt.durable);}
        else{
            assert!(!receipt.durable);let recovered=Chat::persistent(f.path.clone());
            let review=recovered.purge_recovery_target(&receipt.recovery_id.unwrap()).unwrap();
            assert!(recovered.restore_purge_recovery(&review,deadline()).unwrap());
            assert_eq!(recovered.read_conversation("codex",&cid).unwrap().messages.len(),2);
            let restored:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();
            assert_eq!(restored["unavailableHistory"]["pets"]["future-adapter"],json!([opaque]));
            assert!(recovered.purge_recovery_inventory().unwrap().is_empty());
        }
    }
}
#[test]
fn current_history_removal_is_exact_for_every_llm_owner_and_keeps_copies(){
    for provider in ["codex","copilot","antigravity","ollama","openai","anthropic","google","lmstudio"]{
        let f=Fixture::new();let cid=f.seed(provider,"private source",Some("copy-one"),false);let other=if provider=="codex"{"ollama"}else{"codex"};let other_cid=f.seed(other,"keep other",Some("other-copy"),false);
        f.files.write("inbox/source.txt",b"source copy preserved");let target=f.chat.purge_target(provider,&cid).unwrap();assert_eq!(target.message_count,2);assert_eq!(target.attachment_count,1);
        let receipt=f.chat.purge_local_conversation(&target,deadline()).unwrap();assert!(receipt.durable);assert!(receipt.recovery_id.is_none());assert_ne!(receipt.current.unwrap().id,cid);assert!(f.chat.history(provider).is_empty());assert_eq!(f.chat.read_conversation(other,&other_cid).unwrap().messages.len(),2);assert_eq!(std::fs::read(f.files.root.join("inbox/source.txt")).unwrap(),b"source copy preserved");
        let restored=Chat::persistent(f.path.clone());assert!(restored.read_conversation(provider,&cid).is_err());assert!(restored.history(provider).is_empty());assert_eq!(restored.history(other).len(),2);assert!(restored.purge_recovery_inventory().unwrap().is_empty());
    }
}
#[test]
fn archived_removal_keeps_current_same_provider_turn_and_owned_tasks(){
    let f=Fixture::new();let archived=f.seed("codex","old",Some("old-copy"),true);f.chat.reset_provider("codex").unwrap();let current=f.seed("codex","new",None,true);
    let turn=f.chat.begin_cli("codex",Default::default()).unwrap();let lease=f.chat.lease(&turn);let target=f.chat.purge_target("codex",&archived).unwrap();assert!(!target.current);assert_eq!(target.task_count,1);
    let receipt=f.chat.purge_local_conversation(&target,deadline()).unwrap();assert!(receipt.current.is_none());assert!(f.chat.is_current(&turn));assert!(!turn.cancel.load(Ordering::Acquire));assert_eq!(f.chat.conversation_id("codex").unwrap(),current);assert_eq!(f.chat.history("codex")[0]["content"],"new");assert!(f.chat.tasks().iter().all(|t|t.conversation_id.as_deref()!=Some(archived.as_str())));assert_eq!(f.chat.tasks().len(),1);drop(lease);
}
#[test]
fn reference_fence_includes_archives_and_other_owners_without_granting_io(){
    let f=Fixture::new();let first=f.seed("codex","first",Some("shared-copy"),false);f.chat.reset_provider("codex").unwrap();let second=f.seed("ollama","second",Some("shared-copy"),false);
    let refs=f.chat.retention_references().unwrap();assert_eq!(refs.owners["shared-copy"].len(),2);let mut ran=false;assert!(f.chat.with_copy_purge_fence("codex",&first,&refs.revision,&["shared-copy".into()],||{ran=true;Ok(())}).is_err());assert!(!ran);
    f.chat.purge_local_conversation(&f.chat.purge_target("codex",&first).unwrap(),deadline()).unwrap();let refs=f.chat.retention_references().unwrap();assert_eq!(refs.owners["shared-copy"].len(),1);assert!(f.chat.with_copy_purge_fence("ollama",&second,&refs.revision,&["shared-copy".into()],||Ok(())).is_err());
    f.chat.purge_local_conversation(&f.chat.purge_target("ollama",&second).unwrap(),deadline()).unwrap();let refs=f.chat.retention_references().unwrap();f.chat.with_copy_purge_fence("ollama",&second,&refs.revision,&["shared-copy".into()],||Ok(())).unwrap();
}
#[test]
fn busy_and_stale_reviews_preserve_original_library_and_other_turns(){
    let f=Fixture::new();let cid=f.seed("ollama","original",None,false);let target=f.chat.purge_target("ollama",&cid).unwrap();let before=std::fs::read(&f.path).unwrap();
    let turn=f.chat.begin_cli("ollama",Default::default()).unwrap();let lease=f.chat.lease(&turn);assert!(f.chat.purge_target("ollama",&cid).is_err());assert!(f.chat.purge_local_conversation(&target,deadline()).is_err());assert_eq!(std::fs::read(&f.path).unwrap(),before);assert!(f.chat.is_current(&turn));drop(lease);
    f.seed("ollama","new message",None,false);assert!(f.chat.purge_local_conversation(&target,deadline()).is_err());assert_eq!(f.chat.history("ollama").len(),4);
    let target=f.chat.purge_target("ollama",&cid).unwrap();assert!(f.chat.purge_local_conversation(&target,Instant::now()-std::time::Duration::from_secs(1)).is_err());
}
#[test]
fn unrelated_messages_do_not_revoke_a_target_review(){
    let f=Fixture::new();let cid=f.seed("codex","remove me",None,false);let target=f.chat.purge_target("codex",&cid).unwrap();f.seed("ollama","background work",None,false);f.chat.purge_local_conversation(&target,deadline()).unwrap();assert_eq!(f.chat.history("ollama").len(),2);
}
#[test]
fn failed_publication_preserves_history_and_can_resolve_only_its_recovery(){
    let f=Fixture::new();let cid=f.seed("codex","original",Some("source"),true);let target=f.chat.purge_target("codex",&cid).unwrap();let before=std::fs::read(&f.path).unwrap();
    assert!(f.chat.purge_using(&target,deadline(),|_,_,_|Err(std::io::Error::other("synthetic pre-publication failure"))).is_err());assert_eq!(f.chat.history("codex").len(),2);assert_eq!(std::fs::read(&f.path).unwrap(),before);
    let recoveries=f.chat.purge_recovery_inventory().unwrap();assert_eq!(recoveries.len(),1);let review=f.chat.purge_recovery_target(&recoveries[0].id).unwrap();assert!(f.chat.finish_purge_recovery(&review,deadline()).unwrap());assert!(f.chat.purge_recovery_inventory().unwrap().is_empty());assert_eq!(f.chat.history("codex").len(),2);
}
#[test]
fn uncertain_commit_adopts_new_ram_protects_copies_and_supports_restore(){
    let f=Fixture::new();let cid=f.seed("codex","recover me",Some("source"),true);let target=f.chat.purge_target("codex",&cid).unwrap();
    let receipt=f.chat.purge_using(&target,deadline(),|path,library,current|{crate::assistant_memory::save_checked(path,library,current)?;Ok(crate::assistant_memory::SaveReceipt{durable:false})}).unwrap();assert!(!receipt.durable);assert!(receipt.recovery_id.is_some());assert!(f.chat.history("codex").is_empty());assert!(Chat::persistent(f.path.clone()).read_conversation("codex",&cid).is_err());
    let refs=f.chat.retention_references().unwrap();assert!(f.chat.with_copy_purge_fence("codex",&cid,&refs.revision,&["source".into()],||Ok(())).is_err());
    let recovered=Chat::persistent(f.path.clone());let review=recovered.purge_recovery_target(receipt.recovery_id.as_ref().unwrap()).unwrap();assert!(recovered.restore_purge_recovery(&review,deadline()).unwrap());assert_eq!(recovered.read_conversation("codex",&cid).unwrap().messages.len(),2);assert_eq!(recovered.tasks().len(),1);assert!(recovered.purge_recovery_inventory().unwrap().is_empty());assert_eq!(recovered.retention_references().unwrap().owners["source"].len(),1);
}
#[test]
fn uncertain_commit_can_be_finalized_without_resurrecting_removed_history(){
    let f=Fixture::new();let cid=f.seed("ollama","removed",Some("source"),false);let target=f.chat.purge_target("ollama",&cid).unwrap();let receipt=f.chat.purge_using(&target,deadline(),|path,library,current|{crate::assistant_memory::save_checked(path,library,current)?;Ok(crate::assistant_memory::SaveReceipt{durable:false})}).unwrap();
    let review=f.chat.purge_recovery_target(receipt.recovery_id.as_ref().unwrap()).unwrap();assert!(f.chat.finish_purge_recovery(&review,deadline()).unwrap());assert!(f.chat.read_conversation("ollama",&cid).is_err());assert!(f.chat.purge_recovery_inventory().unwrap().is_empty());let refs=f.chat.retention_references().unwrap();f.chat.with_copy_purge_fence("ollama",&cid,&refs.revision,&["source".into()],||Ok(())).unwrap();
}
#[test]
fn recovery_replacement_links_and_changed_review_are_rejected(){
    let f=Fixture::new();let cid=f.seed("codex","original",None,false);let target=f.chat.purge_target("codex",&cid).unwrap();assert!(f.chat.purge_using(&target,deadline(),|_,_,_|Err(std::io::Error::other("fail"))).is_err());
    let mut review=f.chat.purge_recovery_inventory().unwrap().remove(0);review.revision="changed".into();assert!(f.chat.finish_purge_recovery(&review,deadline()).is_err());assert_eq!(f.chat.history("codex").len(),2);
    #[cfg(unix)]{let path=f.files.root.join("purge-recovery").join(format!("{}.json",review.id));let other=f.files.root.join("backup-copy.json");std::fs::rename(&path,&other).unwrap();std::os::unix::fs::symlink(&other,&path).unwrap();assert!(f.chat.purge_recovery_target(&review.id).is_err());assert!(other.exists());}
}
#[test]
fn malformed_library_is_never_replaced_by_purge(){
    let f=Fixture::new();std::fs::write(&f.path,b"not a library").unwrap();let chat=Chat::persistent(f.path.clone());assert!(chat.retention_references().is_err());assert!(chat.purge_target("codex","anything").is_err());assert_eq!(std::fs::read(&f.path).unwrap(),b"not a library");
}
