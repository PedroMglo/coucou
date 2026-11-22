use super::*;
use std::{path::PathBuf,os::unix::fs::PermissionsExt};
struct Files(PathBuf);
impl Files{
 fn new()->Self{let p=std::env::temp_dir().join(format!("coucou-projects-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&p).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o700)).unwrap();Self(p)}
 fn path(&self)->PathBuf{self.0.join("memories.json")}
 fn library(&self)->PathBuf{self.0.join("conversations.json")}
}
impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn create(request:&str,cids:Vec<String>)->Mutation{Mutation::Create{request_id:request.into(),name:"Projeto explicitamente escolhido".into(),conversation_ids:cids}}
fn owner(pet:&str,id:&str)->Owner{Owner{pet_id:pet.into(),scope:Scope::Project{project_id:id.into()}}}
fn query(owner:Owner)->super::super::model::Query{super::super::model::Query{owner,query:String::new(),limit:50,include_expired:true,offset:0,expected_revision:None}}
fn note()->super::super::model::Mutation{use super::super::model::*;super::super::model::Mutation::Create{input:Input{kind:Kind::Note,title:"Decisão do projeto".into(),content:"Dados explícitos, não autoridade".into(),sensitivity:Sensitivity::Normal,expires_at:None}}}

#[test]fn project_catalog_upgrades_only_on_explicit_creation_and_rejects_downgrade_and_duplicate_identity(){
 let old=Registry::empty();let value=serde_json::to_value(&old).unwrap();assert_eq!(value,serde_json::json!({"version":1,"generation":0,"items":[]}));
 let(next,id)=old.apply_project("ai_ollama",create("original-choice",vec![]),100).unwrap();assert_eq!(next.version,2);let next=next.next(&old).unwrap();assert_eq!(next.generation,1);assert_eq!(next.projects[0].id,id);
 let(_,same)=old.apply_project("ai_ollama",create("original-choice",vec![]),101).unwrap();assert_eq!(same,id);assert_eq!(next.apply_project("ai_ollama",create("original-choice",vec![]),101).err(),Some("projectAlreadyExists"));
 let(_,other)=old.apply_project("agent_codex",create("original-choice",vec![]),100).unwrap();assert_ne!(other,id);
 let mut downgraded=next.clone();downgraded.version=1;assert!(downgraded.validate().is_err());assert_eq!(downgraded.next(&next).err(),Some("registryFormatDowngrade"));
 #[derive(Deserialize)]#[serde(deny_unknown_fields)]struct Legacy{#[allow(dead_code)]version:u32,#[allow(dead_code)]generation:u64,#[allow(dead_code)]items:Vec<super::super::model::Entry>}
 assert!(serde_json::from_value::<Legacy>(serde_json::to_value(next).unwrap()).is_err(),"known legacy shape refuses project catalog instead of dropping it");
}

#[test]fn project_notes_are_isolated_from_pet_conversation_other_project_and_other_pet_and_cannot_enter_old_context(){
 let f=Files::new();let first=save_at(&f.path(),"ai_ollama","missing",create("first",vec![]),100,||Ok(()),|_|Ok(())).unwrap();
 let second=save_at(&f.path(),"ai_ollama",&first.revision,create("second",vec![]),101,||Ok(()),|_|Ok(())).unwrap();let own=owner("ai_ollama",&first.project.id);
 let saved=super::super::mutate_at(&f.path(),own.clone(),&second.revision,note(),102,||Ok(())).unwrap();
 let snapshot=crate::private_registry::load_at::<Registry>(&f.path()).unwrap();assert_eq!(snapshot.registry.page(&query(own),snapshot.revision.clone(),103).unwrap().items.len(),1);
 for scope in [Scope::Pet{},Scope::Conversation{conversation_id:"source-cid".into()},Scope::Project{project_id:second.project.id}]{assert!(snapshot.registry.page(&query(Owner{pet_id:"ai_ollama".into(),scope}),snapshot.revision.clone(),103).unwrap().items.is_empty());}
 assert!(snapshot.registry.page(&query(owner("agent_codex",&first.project.id)),snapshot.revision.clone(),103).is_err());
 let selected=[super::super::context_model::Selected{id:saved.id,revision:1}];assert_eq!(super::super::context::Capture::at(&f.path(),"ai_ollama","source-cid",&selected,103).err(),Some("memorySelectionChanged"));
}

#[test]fn project_native_membership_retains_empty_real_conversation_before_publication_and_preserves_it_after_restart(){
 for provider in ["codex","copilot","antigravity","ollama","openai"]{
  let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let cid=chat.conversation(provider).unwrap().id;assert!(!f.library().exists());let pet=crate::pet_config::integration_pet(provider).unwrap();
  let receipt=save_with_chat_at(&f.path(),pet,"missing",create("real-original-conversation",vec![cid.clone()]),&chat,||Ok(())).unwrap();assert!(!receipt.execution_authorized);
  let reopened=crate::chat::Chat::persistent(f.library());assert_eq!(reopened.conversation(provider).unwrap().id,cid);assert!(reopened.read_conversation(provider,&cid).unwrap().messages.is_empty());
  chat.reset_provider(provider).unwrap();assert!(chat.read_conversation(provider,&cid).unwrap().messages.is_empty());let raw=std::fs::read(f.path()).unwrap();assert!(!String::from_utf8(raw).unwrap().contains("messages"));
 }
}

#[test]fn project_wrong_native_owner_missing_cid_failed_retention_and_stale_cas_cannot_publish(){
 let f=Files::new();let chat=crate::chat::Chat::persistent(f.library());let cid=chat.conversation("ollama").unwrap().id;
 for(pet,cid)in [("agent_codex",cid),("ai_ollama","nonexistent-cid".into())]{assert_eq!(save_with_chat_at(&f.path(),pet,"missing",create("denied-membership",vec![cid]),&chat,||Ok(())).err(),Some("ownerChanged"));assert!(!f.path().exists()&&!f.library().exists());}
 assert_eq!(save_at(&f.path(),"ai_ollama","missing",create("retention-fails",vec!["original-cid".into()]),100,||Ok(()),|_|Err("conversationRetentionUnavailable")).err(),Some("conversationRetentionUnavailable"));assert!(!f.path().exists());
 let saved=save_at(&f.path(),"ai_ollama","missing",create("valid-empty-project",vec![]),100,||Ok(()),|_|Ok(())).unwrap();let bytes=std::fs::read(f.path()).unwrap();
 let stale=Mutation::SetConversation{id:saved.project.id,expected_project_revision:1,conversation_id:"original-cid".into(),attached:true};
 assert_eq!(save_at(&f.path(),"ai_ollama","missing",stale,101,||Ok(()),|_|panic!("stale CAS must not retain another dialogue")).err(),Some("registryConflict"));assert_eq!(std::fs::read(f.path()).unwrap(),bytes);
}

#[test]fn project_rename_membership_archive_and_stale_revision_preserve_owned_notes(){
 let f=Files::new();let made=save_at(&f.path(),"ai_ollama","missing",create("one-project",vec!["original-cid".into()]),100,||Ok(()),|_|Ok(())).unwrap();let id=made.project.id.clone();
 let saved=super::super::mutate_at(&f.path(),owner("ai_ollama",&id),&made.revision,note(),101,||Ok(())).unwrap();let note_id=saved.id;
 let renamed=save_at(&f.path(),"ai_ollama",&saved.revision,Mutation::Rename{id:id.clone(),expected_project_revision:1,name:"Nome revisto".into()},102,||Ok(()),|_|Ok(())).unwrap();assert_eq!(renamed.project.revision,2);
 let before=std::fs::read(f.path()).unwrap();assert_eq!(save_at(&f.path(),"ai_ollama",&renamed.revision,Mutation::SetEnabled{id:id.clone(),expected_project_revision:1,enabled:false},103,||Ok(()),|_|Ok(())).err(),Some("projectRevisionConflict"));assert_eq!(std::fs::read(f.path()).unwrap(),before);
 let detached=save_at(&f.path(),"ai_ollama",&renamed.revision,Mutation::SetConversation{id:id.clone(),expected_project_revision:2,conversation_id:"original-cid".into(),attached:false},103,||Ok(()),|_|panic!("removing an old membership must not retain it again")).unwrap();assert!(detached.project.conversation_ids.is_empty());
 let archived=save_at(&f.path(),"ai_ollama",&detached.revision,Mutation::SetEnabled{id,expected_project_revision:3,enabled:false},104,||Ok(()),|_|Ok(())).unwrap();assert!(!archived.project.enabled);
 let reopened=crate::private_registry::load_at::<Registry>(&f.path()).unwrap();assert_eq!(reopened.registry.items.len(),1);assert_eq!(reopened.registry.items[0].id,note_id);assert_eq!(reopened.registry.version,2);
}

#[test]fn project_unknown_authority_and_path_fields_do_not_become_project_identity_or_membership(){
 for value in [serde_json::json!({"operation":"create","requestId":"original","name":"Project","conversationIds":[],"path":"/arbitrary/path"}),serde_json::json!({"operation":"create","requestId":"original","name":"Project","conversationIds":[],"permission":"all"}),serde_json::json!({"operation":"setEnabled","id":"original","expectedProjectRevision":1,"enabled":true,"executionAuthorized":true})]{assert!(serde_json::from_value::<Mutation>(value).is_err());}
 for member in ["/arbitrary/path","../another-project",""]{assert!(Registry::empty().apply_project("ai_ollama",create("original",vec![member.into()]),100).is_err());}
}
