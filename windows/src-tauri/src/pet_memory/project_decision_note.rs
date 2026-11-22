//! Ordinary explicit-user project notes. Deterministic identity makes a retry
//! observable without another journal, consent grant or memory format.
use super::model::{Registry,Owner,Scope,Input,Kind,Sensitivity,Mutation,Receipt,Entry};
use std::path::Path;

pub(crate)fn read_at(path:&Path,owner:&Owner,id:&str)->Result<(String,Option<Entry>),&'static str>{
 let snapshot=crate::private_registry::load_at::<Registry>(path)?;
 if !owner.valid()||!matches!(owner.scope,Scope::Project{..})||!snapshot.registry.accepts_owner(owner){return Err("invalidOwner");}
 let note=snapshot.registry.items.iter().find(|e|e.id==id).cloned();
 if note.as_ref().is_some_and(|e|e.owner!=*owner){return Err("memoryRevisionConflict");}
 if !snapshot.unchanged(){return Err("registryChanged");}Ok((snapshot.revision,note))
}

pub(crate)fn save_at(path:&Path,owner:&Owner,expected:&str,id:&str,title:&str,content:&str,now:u64,current:impl Fn()->Result<(),&'static str>)->Result<Receipt,&'static str>{
 current()?;if !crate::assistant_memory::valid_id(id){return Err("invalidMemoryInput");}
 let (revision,existing)=read_at(path,owner,id)?;
 if let Some(note)=existing{
  // A known exact publication may be acknowledged after an uncertain commit.
  // Edited/conflicting notes are never replaced by a retry or another answer.
  if note.kind!=Kind::Note||note.title!=title||note.content!=content||note.revision!=1{return Err("projectDecisionAlreadyRecorded");}
  current()?;return Ok(Receipt{schema_version:1,owner:owner.clone(),revision,id:id.into(),item:Some(note)});
 }
 if revision!=expected{return Err("registryConflict");}
 let before=crate::private_registry::load_for_update::<Registry>(path,expected)?;
 let input=Input{kind:Kind::Note,title:title.into(),content:content.into(),sensitivity:Sensitivity::Sensitive,expires_at:None};
 let(mut next,random)=before.registry.apply(owner,Mutation::Create{input},now)?;
 next.items.iter_mut().find(|e|e.id==random).ok_or("memoryNotFound")?.id=id.into();
 let saved=crate::private_registry::save_guarded(path,expected,next,|_,_|current(),&current)?;
 let item=saved.registry.items.iter().find(|e|e.id==id&&e.owner==*owner).cloned();
 Ok(Receipt{schema_version:1,owner:owner.clone(),revision:saved.revision,id:id.into(),item})
}

#[cfg(test)]mod tests{
 use super::*;
 use std::{fs,os::unix::fs::PermissionsExt};
 struct F{root:std::path::PathBuf,path:std::path::PathBuf,owner:Owner,revision:String}
 impl F{
  fn new()->Self{
   let root=std::env::temp_dir().join(format!("coucou-decision-note-{}",crate::assistant_memory::new_id()));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();let path=root.join("memories.json");
   let(registry,id)=Registry::empty().apply_project("ai_ollama",super::super::projects::Mutation::Create{request_id:"explicit-project".into(),name:"Projeto de QA".into(),conversation_ids:vec!["source-cid".into()]},1).unwrap();
   let saved=crate::private_registry::save_checked(&path,"missing",registry,|_,_|Ok(())).unwrap();let owner=Owner{pet_id:"ai_ollama".into(),scope:Scope::Project{project_id:id}};
   Self{root,path,owner,revision:saved.revision}
  }
  fn save(&self,content:&str)->Result<Receipt,&'static str>{save_at(&self.path,&self.owner,&self.revision,"qd-exact-result","Decisão QA",content,10,||Ok(()))}
 }
 impl Drop for F{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
 #[test]fn uncertain_ack_and_restart_lookup_keep_one_exact_ordinary_note(){
  let f=F::new();let one=f.save("one exact reviewed answer").unwrap();let bytes=fs::read(&f.path).unwrap();let two=f.save("one exact reviewed answer").unwrap();assert_eq!(one.id,two.id);assert_eq!(bytes,fs::read(&f.path).unwrap());let snapshot=crate::private_registry::load_at::<Registry>(&f.path).unwrap();assert_eq!(snapshot.registry.version,2);assert_eq!(snapshot.registry.items.len(),1);assert_eq!(snapshot.registry.items[0].provenance,super::super::model::Provenance::ExplicitUser);assert!(f.save("a different answer").is_err());assert_eq!(bytes,fs::read(&f.path).unwrap());
  let(_,read)=read_at(&f.path,&f.owner,"qd-exact-result").unwrap();assert_eq!(read.unwrap().content,"one exact reviewed answer");
 }
 #[test]fn revocation_conflict_foreign_owner_and_limits_never_publish(){
  let f=F::new();let before=fs::read(&f.path).unwrap();assert!(save_at(&f.path,&f.owner,&f.revision,"qd-exact-result","Decisão QA","answer",10,||Err("originalSourceRevoked")).is_err());assert!(save_at(&f.path,&f.owner,"stale","qd-exact-result","Decisão QA","answer",10,||Ok(())).is_err());assert!(f.save(&"x".repeat(8193)).is_err());let mut foreign=f.owner.clone();foreign.pet_id="agent_codex".into();assert!(read_at(&f.path,&foreign,"qd-exact-result").is_err());assert_eq!(fs::read(&f.path).unwrap(),before);
 }
 #[test]fn edited_note_is_user_data_and_is_never_overwritten_by_original_ack(){
  let f=F::new();let receipt=f.save("original").unwrap();let s=crate::private_registry::load_at::<Registry>(&f.path).unwrap();let(mut next,_)=s.registry.apply(&f.owner,Mutation::Update{id:receipt.id,expected_item_revision:1,input:Input{kind:Kind::Note,title:"Decisão QA".into(),content:"Explicit edit".into(),sensitivity:Sensitivity::Sensitive,expires_at:None}},20).unwrap();next.generation=s.registry.generation;crate::private_registry::save_checked(&f.path,&s.revision,next,|_,_|Ok(())).unwrap();let bytes=fs::read(&f.path).unwrap();assert!(f.save("original").is_err());assert_eq!(bytes,fs::read(&f.path).unwrap());
 }
 #[test]fn decision_note_time_boundary_refuses_publication_on_rollback_or_frozen_wall_time(){
  use std::time::{Instant,Duration};use crate::pet_runtime::project_decision::review_time_current;
  let f=F::new();let before=fs::read(&f.path).unwrap();let start=Instant::now();let deadline=start+Duration::from_millis(120_000);
  for(name,now,instant)in[("rollback",99,start),("frozen-clock",101,deadline),("expired",120100,start)]{
   let result=save_at(&f.path,&f.owner,&f.revision,&format!("qd-{name}"),"Reviewed answer","No new action",now,||if review_time_current(100,120100,deadline,now,instant){Ok(())}else{Err("projectDecisionReviewEnded")});
   assert!(result.is_err(),"{name}: a real ordinary note was published outside the original native review interval");assert_eq!(fs::read(&f.path).unwrap(),before,"{name}: memory bytes changed");
  }
  assert!(review_time_current(100,120100,deadline,100,start));let saved=save_at(&f.path,&f.owner,&f.revision,"qd-valid","Reviewed answer","No new action",101,||if review_time_current(100,120100,deadline,101,start){Ok(())}else{Err("projectDecisionReviewEnded")} ).unwrap();assert_eq!(saved.item.unwrap().created_at,101);
 }
 #[test]fn decision_note_time_boundary_is_rechecked_after_staging_before_atomic_publish(){
  use std::time::{Instant,Duration};use crate::pet_runtime::project_decision::review_time_current;
  let f=F::new();let before=fs::read(&f.path).unwrap();let start=Instant::now();let deadline=start+Duration::from_millis(120_000);
  let guard=||{let staged=fs::read_dir(&f.root).unwrap().any(|f|f.unwrap().path().extension().is_some_and(|e|e=="tmp"));let instant=if staged{deadline}else{start};if review_time_current(100,120100,deadline,101,instant){Ok(())}else{Err("projectDecisionReviewEnded")}};
  assert!(save_at(&f.path,&f.owner,&f.revision,"qd-staged","Reviewed answer","No new action",101,guard).is_err(),"a frozen wall clock allowed publication after the monotonic review deadline");assert_eq!(fs::read(&f.path).unwrap(),before);
 }
}
