use super::*;
use super::super::{model::{Owner,Scope,Input,Kind,Sensitivity,Mutation},projects::{Mutation as ProjectMutation,Project},mutate_at,native::Binding};
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt};
struct Fixture{root:PathBuf,pet:String,cid:String,project:Project,selected:Vec<Selected>}
impl Fixture{
 fn new(provider:&str)->Self{
  let root=std::env::temp_dir().join(format!("coucou-project-context-{}",crate::assistant_memory::new_id()));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();
  let chat=crate::chat::Chat::persistent(root.join("conversations.json"));let cid=chat.conversation_id(provider).unwrap();chat.retain_public_conversation(provider,&cid).unwrap();let pet=crate::pet_config::integration_pet(provider).unwrap().to_string();
  let old=crate::private_registry::load_at::<Registry>(&root.join("memories.json")).unwrap();let(next,id)=old.registry.apply_project(&pet,ProjectMutation::Create{request_id:"own-choice".into(),name:"Project <data>".into(),conversation_ids:vec![cid.clone()]},100).unwrap();
  let saved=crate::private_registry::save_guarded(&root.join("memories.json"),&old.revision,next,|_,_|Ok(()),||Ok(())).unwrap();let project=saved.registry.projects.iter().find(|p|p.id==id).unwrap().clone();
  let row=mutate_at(&root.join("memories.json"),Owner{pet_id:pet.clone(),scope:Scope::Project{project_id:id}},&saved.revision,Mutation::Create{input:Input{kind:Kind::Note,title:"Own project note".into(),content:"PRIVATE_PROJECT_DATA_ONLY".into(),sensitivity:Sensitivity::Sensitive,expires_at:None}},101,||Ok(())).unwrap();
  Self{root,pet,cid,project,selected:vec![Selected{id:row.id,revision:1}]}
 }
 fn path(&self)->PathBuf{self.root.join("memories.json")}
 fn choice(&self)->SelectedProject{SelectedProject{id:self.project.id.clone(),revision:self.project.revision}}
 fn capture(&self)->Capture{Capture::at_manual(&self.path(),&self.pet,&self.cid,&self.selected,Some(&self.choice()),102).unwrap()}
 fn change(&self,mutation:ProjectMutation){let old=crate::private_registry::load_at::<Registry>(&self.path()).unwrap();let(next,_)=old.registry.apply_project(&self.pet,mutation,103).unwrap();crate::private_registry::save_guarded(&self.path(),&old.revision,next,|_,_|Ok(()),||Ok(())).unwrap();}
 fn accepted(&self,dest:&Destination,now:u64)->Accepted{let mut reviews=Reviews::default();let p=Prepared::new(self.capture(),dest.clone(),"Original question".into(),None,now).unwrap();let t=reviews.insert(crate::HUB_LABEL,p,now).unwrap();assert_eq!(t.proposal.schema_version,2);assert_eq!(t.proposal.project.as_ref(),Some(&self.project));reviews.take(crate::HUB_LABEL,&t.token,&self.pet,&self.cid,"Original question",dest,None,true,if dest.native(){Consent::NativeSessionContext}else{Consent::ThisRequest},now).unwrap().unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
#[test]fn project_capture_requires_explicit_exact_choice_current_membership_and_own_notes(){
 let f=Fixture::new("ollama");assert!(Capture::at(&f.path(),&f.pet,&f.cid,&f.selected,102).is_err());
 for(pet,cid,choice)in [(f.pet.as_str(),"unassociated-cid",f.choice()),("agent_codex",f.cid.as_str(),f.choice()),(f.pet.as_str(),f.cid.as_str(),SelectedProject{id:f.project.id.clone(),revision:2})]{assert!(Capture::at_manual(&f.path(),pet,cid,&f.selected,Some(&choice),102).is_err());}
 let c=f.capture();assert_eq!(c.references()[0].project_revision,Some(1));assert!(!c.references()[0].valid(&f.pet,&f.cid));assert!(!Selection{project:None,schema_version:1,references:c.references().to_vec()}.valid(&f.pet,Some(&f.cid)));assert!(Capture::at_references(&f.path(),&f.pet,&f.cid,c.references(),102).is_err());assert!(c.goal_message("goal",Some(&f.cid),102).is_err());
 let other=SelectedProject{id:"other-project".into(),revision:1};assert!(Capture::at_manual(&f.path(),&f.pet,&f.cid,&f.selected,Some(&other),102).is_err());
}
#[test]fn explicit_project_data_is_reviewed_and_injected_only_as_request_local_user_data(){
 let f=Fixture::new("ollama");let chat=crate::chat::Chat::persistent(f.root.join("conversations.json"));let dest=Destination::Server{provider:"ollama".into(),endpoint:"http://127.0.0.1:11444/v1/chat/completions".into(),model:"fixture-free".into()};let a=f.accepted(&dest,102);let c=chat.with_selected_memory(Arc::new(a));let turn=c.begin_cli("ollama",Arc::default()).unwrap();let _lease=c.lease(&turn);
 let user=serde_json::json!({"role":"user","content":"Original question"});let mut messages=vec![serde_json::json!({"role":"system","content":"native policy"}),user.clone()];c.append_selected_memory("ollama","Original question",&dest,None,&mut messages).unwrap();assert_eq!(messages[0]["content"],"native policy");assert_eq!(messages.last(),Some(&user));assert_eq!(messages[1]["role"],"user");let content=messages[1]["content"].as_str().unwrap();assert!(content.contains("PRIVATE_PROJECT_DATA_ONLY")&&content.contains(&f.project.id)&&content.contains("Project <data>"));
 c.commit_reply(&turn,user,serde_json::json!({"role":"assistant","content":"answer"}),"Original question","answer").unwrap();assert!(!serde_json::to_string(&chat.history("ollama")).unwrap().contains("PRIVATE_PROJECT_DATA_ONLY"));assert!(c.begin_cli("ollama",Arc::default()).is_err());
}
#[test]fn project_metadata_membership_and_note_changes_terminally_revoke_but_unrelated_writes_do_not(){
 for change in ["rename","archive","detach","note","delete"]{
  let f=Fixture::new("ollama");let c=f.capture();let old=crate::private_registry::load_at::<Registry>(&f.path()).unwrap();mutate_at(&f.path(),Owner{pet_id:f.pet.clone(),scope:Scope::Pet{}},&old.revision,Mutation::Create{input:Input{kind:Kind::Note,title:"Unrelated".into(),content:"Not selected".into(),sensitivity:Sensitivity::Normal,expires_at:None}},102,||Ok(())).unwrap();assert!(c.verify(102).is_ok());let original=fs::read(f.path()).unwrap();
  match change{
   "rename"=>f.change(ProjectMutation::Rename{id:f.project.id.clone(),expected_project_revision:1,name:"New name".into()}),
   "archive"=>f.change(ProjectMutation::SetEnabled{id:f.project.id.clone(),expected_project_revision:1,enabled:false}),
   "detach"=>f.change(ProjectMutation::SetConversation{id:f.project.id.clone(),expected_project_revision:1,conversation_id:f.cid.clone(),attached:false}),
   _=>{let old=crate::private_registry::load_at::<Registry>(&f.path()).unwrap();let mutation=if change=="delete"{Mutation::Delete{id:f.selected[0].id.clone(),expected_item_revision:1}}else{Mutation::Update{id:f.selected[0].id.clone(),expected_item_revision:1,input:Input{kind:Kind::Note,title:"Revised".into(),content:"Changed selected bytes".into(),sensitivity:Sensitivity::Sensitive,expires_at:None}}};mutate_at(&f.path(),Owner{pet_id:f.pet.clone(),scope:Scope::Project{project_id:f.project.id.clone()}},&old.revision,mutation,103,||Ok(())).unwrap();},
  }
  assert!(c.verify(103).is_err(),"{change}");fs::write(f.path(),original).unwrap();assert_eq!(c.verify(104).err(),Some("memorySelectionRevoked"));
 }
}
#[test]fn native_project_references_survive_adoption_restart_and_refuse_further_use_after_detach(){
 for provider in ["codex","copilot","antigravity"]{
  let f=Fixture::new(provider);let dest=super::super::native::destination(provider,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,None).unwrap();let a=Arc::new(f.accepted(&dest,102));let binding=Binding::at(&f.path(),dest.clone(),&f.cid,None,Some(&a),"Original question",102).unwrap();let chat=crate::chat::Chat::persistent(f.root.join("conversations.json")).with_selected_memory(a).fixture_native_memory(binding);let turn=chat.begin_cli(provider,Arc::default()).unwrap();let lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();let prompt=chat.native_memory_prompt(&turn,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,None,"Original question","ordinary input".into()).unwrap();assert!(prompt.contains("PRIVATE_PROJECT_DATA_ONLY"));chat.remember_session(&turn,"same-native-session",f.root.to_str().unwrap(),false);drop(lease);drop(chat);
  let reopened=crate::chat::Chat::persistent(f.root.join("conversations.json"));let(_,_,old)=reopened.native_memory_info(provider).unwrap();let old=old.unwrap();assert_eq!(old.schema_version,2);assert!(old.valid_for(provider,&f.cid));let mut wrong=old.clone();wrong.schema_version=1;assert!(!wrong.valid_for(provider,&f.cid));assert!(!fs::read_to_string(f.root.join("conversations.json")).unwrap().contains("PRIVATE_PROJECT_DATA_ONLY"));let dest=super::super::native::destination(provider,crate::cli_chat::backend(provider).unwrap(),"fixture-free",&f.root,Some("same-native-session")).unwrap();assert!(Binding::at(&f.path(),dest.clone(),&f.cid,Some(&old),None,"new question",crate::assistant_memory::now()).is_ok());
  f.change(ProjectMutation::SetConversation{id:f.project.id.clone(),expected_project_revision:1,conversation_id:f.cid.clone(),attached:false});assert!(old.valid_for(provider,&f.cid),"historical data remains readable");assert!(Binding::at(&f.path(),dest,&f.cid,Some(&old),None,"new question",crate::assistant_memory::now()).is_err());
 }
}
#[tokio::test]async fn active_native_project_detach_durably_revokes_and_catalog_restore_does_not_revive(){
 let f=Fixture::new("codex");let now=crate::assistant_memory::now();let dest=super::super::native::destination("codex","codex","fixture-free",&f.root,None).unwrap();let a=Arc::new(f.accepted(&dest,now));let binding=Binding::at(&f.path(),dest,&f.cid,None,Some(&a),"Original question",now).unwrap();let chat=crate::chat::Chat::persistent(f.root.join("conversations.json")).with_selected_memory(a).fixture_native_memory(binding);let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);chat.publish_native_memory(&turn).unwrap();chat.remember_session(&turn,"same-native-session",f.root.to_str().unwrap(),false);let original=fs::read(f.path()).unwrap();
 f.change(ProjectMutation::SetConversation{id:f.project.id.clone(),expected_project_revision:1,conversation_id:f.cid.clone(),attached:false});tokio::time::timeout(std::time::Duration::from_secs(3),async{loop{let saved=crate::assistant_memory::load(&f.root.join("conversations.json")).unwrap();if turn.cancel.load(Ordering::Acquire)&&saved.pets["codex"].native_memory_context.as_ref().unwrap().revoked{break;}tokio::time::sleep(std::time::Duration::from_millis(5)).await;}}).await.unwrap();drop(lease);drop(chat);fs::write(f.path(),original).unwrap();
 let reopened=crate::chat::Chat::persistent(f.root.join("conversations.json"));let(_,_,old)=reopened.native_memory_info("codex").unwrap();let old=old.unwrap();assert_eq!(old.schema_version,2);assert!(old.revoked&&old.valid_for("codex",&f.cid));let dest=super::super::native::destination("codex","codex","fixture-free",&f.root,Some("same-native-session")).unwrap();assert_eq!(Binding::at(&f.path(),dest,&f.cid,Some(&old),None,"new question",crate::assistant_memory::now()).err(),Some("nativeMemoryContextRevoked"));
}
