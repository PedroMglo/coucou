use super::*;
use super::super::{model::{Registry,Owner,Scope,Input,Kind,Sensitivity,Mutation},context::{Prepared,Reviews},context_model::{Consent,Selected}};
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt};
struct Fixture{root:PathBuf}
impl Fixture{
 fn new()->Self{let root=std::env::temp_dir().join(format!("native-memory-{}",crate::assistant_memory::new_id()));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();Self{root}}
 fn path(&self)->PathBuf{self.root.join("memories.json")}
 fn library(&self)->PathBuf{self.root.join("conversations.json")}
 fn chat(&self)->crate::chat::Chat{crate::chat::Chat::persistent(self.library())}
 fn create(&self,provider:&str,content:&str)->super::super::Receipt{
  let owner=Owner{pet_id:crate::pet_config::integration_pet(provider).unwrap().into(),scope:Scope::Pet{}};let snapshot=crate::private_registry::load_at::<Registry>(&self.path()).unwrap();
  super::super::mutate_at(&self.path(),owner,&snapshot.revision,Mutation::Create{input:Input{kind:Kind::Note,title:"Explicit native-session data".into(),content:content.into(),sensitivity:Sensitivity::Sensitive,expires_at:None}},crate::assistant_memory::now(),||Ok(())).unwrap()
 }
 fn destination(&self,provider:&str,id:Option<&str>)->Destination{destination(provider,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&self.root,id).unwrap()}
 fn accepted(&self,provider:&str,cid:&str,destination:&Destination,records:&[&super::super::Receipt])->Arc<Accepted>{
  let now=crate::assistant_memory::now();let selected=records.iter().map(|r|Selected{id:r.id.clone(),revision:r.item.as_ref().unwrap().revision}).collect::<Vec<_>>();let pet=crate::pet_config::integration_pet(provider).unwrap();let capture=Capture::at(&self.path(),pet,cid,&selected,now).unwrap();let prepared=Prepared::new(capture,destination.clone(),"Original public question".into(),None,now).unwrap();let mut reviews=Reviews::default();let ticket=reviews.insert(crate::HUB_LABEL,prepared,now).unwrap();Arc::new(reviews.take(crate::HUB_LABEL,&ticket.token,pet,cid,"Original public question",destination,None,true,Consent::NativeSessionContext,now).unwrap().unwrap())
 }
 fn remove(&self,row:&super::super::Receipt){let old=crate::private_registry::load_at::<Registry>(&self.path()).unwrap();super::super::mutate_at(&self.path(),row.owner.clone(),&old.revision,Mutation::Delete{id:row.id.clone(),expected_item_revision:row.item.as_ref().unwrap().revision},crate::assistant_memory::now(),||Ok(())).unwrap();}
 fn fresh(&self,provider:&str)->(crate::chat::Chat,super::super::Receipt,String,Destination){
  let chat=self.chat();let cid=chat.conversation_id(provider).unwrap();let record=self.create(provider,"PRIVATE_NATIVE_MEMORY_SENTINEL");let dest=self.destination(provider,None);let accepted=self.accepted(provider,&cid,&dest,&[&record]);
  let binding=Binding::at(&self.path(),dest.clone(),&cid,None,Some(&accepted),"Original public question",crate::assistant_memory::now()).unwrap();(chat.with_selected_memory(accepted).fixture_native_memory(binding),record,cid,dest)
 }
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}

#[test]fn native_binding_is_common_to_three_accounts_and_a_service_with_durable_adoption_and_no_raw_history(){
 for provider in ["codex","copilot","antigravity","integration_notion"]{
  let f=Fixture::new();let(chat,_,cid,_)=f.fresh(provider);let turn=chat.begin_cli(provider,Arc::default()).unwrap();let lease=chat.lease(&turn);
  assert!(chat.native_memory_prompt(&turn,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,None,"Original public question","ordinary input".into()).is_err());
  chat.publish_native_memory(&turn).unwrap();let raw=fs::read_to_string(f.library()).unwrap();assert!(!raw.contains("PRIVATE_NATIVE_MEMORY_SENTINEL"));let saved=crate::assistant_memory::load(&f.library()).unwrap();assert!(saved.pets[provider].native_memory_context.is_some());
  let prompt=chat.native_memory_prompt(&turn,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,None,"Original public question","ordinary input".into()).unwrap();assert!(prompt.contains("PRIVATE_NATIVE_MEMORY_SENTINEL"));assert!(prompt.ends_with("ordinary input"));
  chat.remember_session(&turn,"exact-original-native-id",f.root.to_str().unwrap(),false);assert!(chat.is_current(&turn));
  let(_,session,metadata)=chat.native_memory_info(provider).unwrap();assert_eq!(session.unwrap().id,"exact-original-native-id");assert_eq!(metadata.unwrap().destination,f.destination(provider,Some("exact-original-native-id")));
  chat.commit_reply(&turn,serde_json::json!({"role":"user","content":"Original public question"}),serde_json::json!({"role":"assistant","content":"public answer"}),"Original public question","public answer").unwrap();drop(lease);assert!(!fs::read_to_string(f.library()).unwrap().contains("PRIVATE_NATIVE_MEMORY_SENTINEL"));
  let restarted=f.chat();let(id,session,old)=restarted.native_memory_info(provider).unwrap();assert_eq!(id,cid);let dest=f.destination(provider,Some(&session.unwrap().id));let binding=Binding::at(&f.path(),dest,&cid,old.as_ref(),None,"new ordinary question",crate::assistant_memory::now()).unwrap();let channel=restarted.fixture_native_memory(binding);let next=channel.begin_cli(provider,Arc::default()).unwrap();let _lease=channel.lease(&next);channel.publish_native_memory(&next).unwrap();
  assert_eq!(channel.native_memory_prompt(&next,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,Some("exact-original-native-id"),"new ordinary question","new ordinary input".into()).unwrap(),"new ordinary input");
 }
}
#[test]fn unknown_native_id_after_possible_delivery_survives_restart_and_explicit_archive_without_fabricated_messages(){
 let f=Fixture::new();let(chat,_,cid,_)=f.fresh("antigravity");let turn=chat.begin_cli("antigravity",Arc::default()).unwrap();let lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();drop(lease);drop(chat);
 let restarted=f.chat();let(_,session,old)=restarted.native_memory_info("antigravity").unwrap();assert!(session.is_none());assert!(restarted.history("antigravity").is_empty());assert_eq!(Binding::at(&f.path(),f.destination("antigravity",None),&cid,old.as_ref(),None,"new question",crate::assistant_memory::now()).err(),Some("nativeMemorySessionUnconfirmed"));
 restarted.reset_provider("antigravity").unwrap();let saved=crate::assistant_memory::load(&f.library()).unwrap();assert_eq!(saved.archives["antigravity"][0].id,cid);assert!(saved.archives["antigravity"][0].native_memory_context.is_some());assert!(saved.archives["antigravity"][0].messages.is_empty());
}
#[tokio::test]async fn native_lifetime_revocation_is_durable_and_restoring_catalog_bytes_cannot_revive_the_session(){
 let f=Fixture::new();let(chat,row,cid,_)=f.fresh("codex");let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();chat.remember_session(&turn,"exact-original-native-id",f.root.to_str().unwrap(),false);let original=fs::read(f.path()).unwrap();f.remove(&row);
 tokio::time::timeout(std::time::Duration::from_secs(3),async{while !turn.cancel.load(Ordering::Acquire){tokio::time::sleep(std::time::Duration::from_millis(5)).await;}}).await.unwrap();
 // Cancellation precedes durable marking; wait for the same supervisor to
 // finish the write rather than treating its first signal as a save receipt.
 tokio::time::timeout(std::time::Duration::from_secs(3),async{loop{if crate::assistant_memory::load(&f.library()).unwrap().pets["codex"].native_memory_context.as_ref().unwrap().revoked{break;}tokio::time::sleep(std::time::Duration::from_millis(5)).await;}}).await.unwrap();drop(lease);drop(chat);fs::write(f.path(),original).unwrap();
 let restarted=f.chat();let(_,_,old)=restarted.native_memory_info("codex").unwrap();assert_eq!(Binding::at(&f.path(),f.destination("codex",Some("exact-original-native-id")),&cid,old.as_ref(),None,"new question",crate::assistant_memory::now()).err(),Some("nativeMemoryContextRevoked"));assert!(restarted.begin_cli("codex",Arc::default()).is_err());
}
#[test]fn native_session_adoption_refuses_a_second_id_without_replacing_saved_identity(){
 let f=Fixture::new();let(chat,_,_,_)=f.fresh("copilot");let turn=chat.begin_cli("copilot",Arc::default()).unwrap();let _lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();chat.remember_session(&turn,"original-session",f.root.to_str().unwrap(),false);let before=fs::read(f.library()).unwrap();chat.remember_session(&turn,"foreign-session",f.root.to_str().unwrap(),false);assert!(turn.cancel.load(Ordering::Acquire));assert_eq!(fs::read(f.library()).unwrap(),before);assert_eq!(chat.native_memory_info("copilot").unwrap().1.unwrap().id,"original-session");
}
#[test]fn failed_checked_session_adoption_cancels_before_further_transport_work_and_preserves_possible_delivery(){
 let f=Fixture::new();let(chat,_,_,_)=f.fresh("codex");let turn=chat.begin_cli("codex",Arc::default()).unwrap();let _lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();let backup=f.root.join("original-library.json");fs::rename(f.library(),&backup).unwrap();fs::create_dir(f.library()).unwrap();chat.remember_session(&turn,"must-not-be-adopted",f.root.to_str().unwrap(),false);assert!(turn.cancel.load(Ordering::Acquire));let(_,session,metadata)=chat.native_memory_info("codex").unwrap();assert!(session.is_none());assert!(metadata.is_some());assert!(f.library().is_dir());let previous=crate::assistant_memory::load(&backup).unwrap();assert!(previous.pets["codex"].native_memory_context.is_some());
}
#[test]fn resumed_binding_requires_exact_model_workspace_and_native_id_and_cannot_forget_prior_references(){
 let f=Fixture::new();let(chat,first,cid,_)=f.fresh("codex");let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();chat.remember_session(&turn,"original-session",f.root.to_str().unwrap(),false);drop(lease);let old=chat.native_memory_info("codex").unwrap().2.unwrap();
 for field in ["model","workspace","session"]{let mut dest=f.destination("codex",Some("original-session"));if let Destination::NativeSession{model,workspace,session_id,..}=&mut dest{match field{"model"=>*model="different-model".into(),"workspace"=>*workspace="/different/workspace".into(),_=>*session_id=Some("other-session".into())}}assert_eq!(Binding::at(&f.path(),dest,&cid,Some(&old),None,"new question",crate::assistant_memory::now()).err(),Some("nativeMemoryDestinationChanged"));}
 let second=f.create("codex","new explicitly selected data");let dest=f.destination("codex",Some("original-session"));let accepted=f.accepted("codex",&cid,&dest,&[&second]);let joined=Binding::at(&f.path(),dest,&cid,Some(&old),Some(&accepted),"Original public question",crate::assistant_memory::now()).unwrap().metadata().unwrap();assert_eq!(joined.references.len(),2);assert_eq!(joined.references[0].id,first.id);assert_eq!(joined.references[1].id,second.id);assert_eq!(old.references.len(),1);
}
