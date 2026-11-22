use super::*;
use std::os::unix::fs::{DirBuilderExt,PermissionsExt};
struct Temp{root:PathBuf,inbox:PathBuf,meta:PathBuf,chat:Chat,state:Mutex<Store>}
impl Temp{
    fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-native-retention-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();let inbox=root.join("inbox");std::fs::DirBuilder::new().mode(0o700).create(&inbox).unwrap();Self{meta:root.join("metadata"),root,inbox,chat:Chat::default(),state:Mutex::default()}}
    fn seed(&self,provider:&str,cid:&str,id:&str)->Attachment{
        let name=format!("{id}.txt");let path=self.inbox.join(&name);std::fs::write(&path,b"abc").unwrap();std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o600)).unwrap();
        let record=Record{id:id.into(),copy_name:name.clone(),name:name.clone(),kind:"file".into(),size:3,entries:1,selected:false,purge_transaction:None};let mut p=io::prepare(&path_at(&self.meta,provider,cid).unwrap(),None,|r|{r.bind(provider,cid)?;r.records.push(record.clone());Ok(())}).unwrap();p.publish().unwrap();p.confirm().unwrap();attachment(provider,cid,&record,&self.inbox,&name)
    }
    fn review(&self,provider:&str,cid:&str,ids:&[&str])->Result<NativeReview,String>{initial_at(&self.chat,&self.state,&self.meta,&self.inbox,"pet-chat".into(),provider.into(),cid.into(),ids.iter().map(|s|(*s).into()).collect(),io::load_at(&path_at(&self.meta,provider,cid).unwrap()).unwrap().revision,Instant::now()+Duration::from_secs(120))}
    fn execute(&self,r:NativeReview,consent:&str)->Result<Receipt,String>{execute(r,&self.chat,&self.state,&self.inbox,&self.meta,consent)}
    fn recovery(&self,id:&str)->NativeReview{recovery_at(&self.chat,&self.state,&self.inbox,"pet-chat".into(),id,Instant::now()+Duration::from_secs(120)).unwrap()}
    fn tx(&self,r:&NativeReview,marked:bool,moved:bool)->Transaction{let Payload::Copies(i)=&r.payload else{panic!()};let mut tx=Transaction::create(&self.inbox,&r.provider,&r.cid,&i.manifest.records,&i.copies).unwrap();if marked{publish(i.manifest.mark(tx.id()).unwrap()).unwrap();tx.marked().unwrap();}if moved{tx.move_all().unwrap();}tx}
    fn history(&self,p:&str,id:&str){let turn=self.chat.begin_cli(p,Default::default()).unwrap();let lease=self.chat.lease(&turn);self.chat.capture_attachment_history(&turn,vec![crate::attachment_history::Reference{id:id.into(),name:"history.txt".into(),kind:"file".into(),size:3,entries:1,sequence:1}]).unwrap();self.chat.commit(&turn,serde_json::json!({"role":"user","content":"synthetic"}),serde_json::json!({"role":"assistant","content":"reply"}),"synthetic","reply").unwrap();drop(lease);}
}
impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
#[test]fn native_purge_works_for_all_eight_metadata_owners_and_keeps_other_cids(){
    for provider in crate::pet_documents::CHAT_PROVIDERS{
        let t=Temp::new();let a=t.seed(provider,"old-cid","copy-a");let b=t.seed(provider,"other-cid","copy-b");let before=std::fs::read(path_at(&t.meta,provider,"other-cid").unwrap()).unwrap();
        let r=t.review(provider,"old-cid",&["copy-a"]).unwrap();let outcome=t.execute(r,"deleteOwnedCopies").unwrap();assert!(outcome.complete,"{provider}");assert!(outcome.recovery_id.is_none());assert!(!Path::new(&a.file.path).exists());assert!(Path::new(&b.file.path).exists());assert_eq!(std::fs::read(path_at(&t.meta,provider,"other-cid").unwrap()).unwrap(),before);assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
    }
}
#[test]fn native_inventory_has_orphan_owners_reference_counts_and_no_paths_or_read_grants(){
    let t=Temp::new();let a=t.seed("ollama","orphan","copy-a");t.history("codex","copy-a");let inventory=inventory_at(&t.chat,&t.meta,&t.inbox).unwrap();assert_eq!(inventory.owners.len(),1);assert_eq!(inventory.owners[0].conversation_id,"orphan");assert_eq!(inventory.owners[0].items[0].history_references,1);let text=serde_json::to_string(&inventory).unwrap();assert!(!text.contains(&a.file.path));assert!(!text.contains("copyName"));assert!(!text.contains("grant"));assert!(t.state.lock().unwrap().files.is_empty());assert!(t.review("ollama","orphan",&["copy-a"]).is_err());
}
#[test]fn wrong_confirmation_stale_manifest_copy_selection_and_expiry_cannot_delete(){
    for mode in ["consent","manifest","copy","selection","expiry"]{
        let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let mut r=t.review("ollama","old-cid",&["copy-a"]).unwrap();match mode{
            "manifest"=>{t.seed("ollama","old-cid","copy-b");},"copy"=>std::fs::write(&a.file.path,b"changed").unwrap(),"selection"=>t.state.lock().unwrap().touch("ollama","old-cid"),"expiry"=>r.deadline=Instant::now(),_=>{},
        }let outcome=t.execute(r,if mode=="consent"{"restoreOwnedCopies"}else{"deleteOwnedCopies"});assert!(outcome.as_ref().map_or(true,|o|!o.complete),"{mode}");assert!(Path::new(&a.file.path).exists(),"{mode}");
    }
}
#[test]fn pending_owner_foreign_source_and_unclaimed_native_drop_protect_copies(){
    for mode in ["owner","source","drop","ready"]{
        let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let mut state=t.state.lock().unwrap();match mode{
            "ready"=>{state.files.insert(("codex".into(),"other".into()),vec![crate::pet_documents::Ready{attachment:a.clone(),drop_id:"earlier".into()}]);},
            "drop"=>{state.drops.insert("native".into(),crate::pet_documents::DropGrant{sequence:1,at:Instant::now(),paths:vec![a.file.path.clone()],owners:BTreeMap::from([("codex".into(),"other".into())])});},
            _=>{state.pending.insert(("codex".into(),"pending".into()),crate::pet_documents::Ticket{provider:if mode=="owner"{"ollama"}else{"codex"}.into(),id:if mode=="owner"{"old-cid"}else{"other"}.into(),sequence:1,drop_id:"pending".into(),paths:vec![if mode=="source"{a.file.path.clone()}else{"/unrelated/source".into()}],deadline:Instant::now()+Duration::from_secs(120)});},
        }drop(state);assert!(t.review("ollama","old-cid",&["copy-a"]).is_err(),"{mode}");assert!(Path::new(&a.file.path).exists());
    }
}
#[test]fn selected_reference_projection_preserves_unrelated_history_but_rejects_new_reference_and_busy_owner(){
    let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();t.history("codex","another-copy");assert!(t.execute(r,"deleteOwnedCopies").unwrap().complete);assert!(!Path::new(&a.file.path).exists());
    let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();t.history("codex","copy-a");assert!(t.execute(r,"deleteOwnedCopies").is_err());assert!(Path::new(&a.file.path).exists());
    let t=Temp::new();let cid=t.chat.conversation_id("ollama").unwrap();let a=t.seed("ollama",&cid,"copy-a");let r=t.review("ollama",&cid,&["copy-a"]).unwrap();let turn=t.chat.begin_cli("ollama",Default::default()).unwrap();let lease=t.chat.lease(&turn);assert!(t.execute(r,"deleteOwnedCopies").is_err());assert!(Path::new(&a.file.path).exists());drop(lease);
}
#[test]fn native_tokens_are_one_use_bounded_and_bind_exact_window_owner_expiry(){
    let t=Temp::new();t.seed("ollama","old-cid","copy-a");let mut cache=Reviews::default();let now=Instant::now();let p=cache.insert(t.review("ollama","old-cid",&["copy-a"]).unwrap(),now).unwrap();assert_eq!(p.review_id.len(),64);assert!(cache.take(&p.review_id,"other-window","ollama","old-cid",now).is_err());assert!(cache.take(&p.review_id,"pet-chat","codex","old-cid",now).is_err());assert!(cache.take(&p.review_id,"pet-chat","ollama","other-cid",now).is_err());assert_eq!(cache.0.len(),1);let r=cache.take(&p.review_id,"pet-chat","ollama","old-cid",now).unwrap();drop(r);assert!(cache.take(&p.review_id,"pet-chat","ollama","old-cid",now).is_err());
    let p=cache.insert(t.review("ollama","old-cid",&["copy-a"]).unwrap(),now).unwrap();cache.cancel(&p.review_id,"wrong-window","ollama","old-cid");assert_eq!(cache.0.len(),1);cache.cancel(&p.review_id,"pet-chat","ollama","old-cid");assert!(cache.0.is_empty());
    for _ in 0..32{cache.insert(t.review("ollama","old-cid",&["copy-a"]).unwrap(),now).unwrap();}assert!(cache.insert(t.review("ollama","old-cid",&["copy-a"]).unwrap(),now).is_err());cache.expire(now+Duration::from_secs(121));assert!(cache.0.is_empty());
}
#[test]fn prepared_recovery_can_clear_only_its_metadata_without_changing_original_copy(){
    let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let tx=t.tx(&r,false,false);let id=tx.id().to_owned();drop(tx);let outcome=t.execute(t.recovery(&id),"clearCopyRecovery").unwrap();assert!(outcome.complete);assert!(outcome.recovery_id.is_none());assert_eq!(std::fs::read(&a.file.path).unwrap(),b"abc");assert_eq!(io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records.len(),1);
    let tx=t.tx(&r,true,false);let id=tx.id().to_owned();drop(tx);assert!(t.execute(t.recovery(&id),"clearCopyRecovery").is_err());assert!(Path::new(&a.file.path).exists());
}
#[test]fn native_recovery_can_finish_marked_and_manifest_removed_operations_after_restart(){
    for removed in [false,true]{let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let tx=t.tx(&r,true,true);let id=tx.id().to_owned();if removed{publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();}drop(tx);let outcome=t.execute(t.recovery(&id),"finishCopyPurge").unwrap();assert!(outcome.complete);assert!(outcome.recovery_id.is_none());assert!(!Path::new(&a.file.path).exists());assert!(io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records.is_empty());}
}
#[test]fn native_restore_keeps_reused_source_and_returns_original_id_deselected_without_grants(){
    let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let tx=t.tx(&r,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();drop(tx);std::fs::write(&a.file.path,b"new copy").unwrap();std::fs::set_permissions(&a.file.path,std::fs::Permissions::from_mode(0o600)).unwrap();
    let outcome=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(outcome.complete);assert!(outcome.recovery_id.is_none());assert_eq!(std::fs::read(&a.file.path).unwrap(),b"new copy");let record=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records.remove(0);assert_eq!(record.id,"copy-a");assert!(!record.selected);assert!(record.purge_transaction.is_none());assert_ne!(record.copy_name,"copy-a.txt");assert_eq!(std::fs::read(t.inbox.join(record.copy_name)).unwrap(),b"abc");assert!(t.state.lock().unwrap().contexts("ollama","old-cid").is_empty());
}
#[test]fn uncertain_wal_publication_returns_original_recovery_and_can_finish_every_phase(){
    for number in 2..=5{
        let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let Payload::Copies(i)=&r.payload else{panic!()};
        let result=finish_initial_using(&r,i,&t.chat,&t.state,&t.inbox,||{let mut tx=Transaction::create(&t.inbox,&r.provider,&r.cid,&i.manifest.records,&i.copies).unwrap();tx.fail_after_wal_publish(number);Ok(tx)},|_|Ok(())).unwrap();
        assert!(!result.complete,"{number}");let id=result.recovery_id.unwrap();assert_eq!(quarantine::inventory(&t.inbox).unwrap().recoveries[0].id,id);let review=t.recovery(&id);
        let result=t.execute(review,"finishCopyPurge").unwrap();assert!(result.complete,"{number}");assert!(result.recovery_id.is_none());assert!(!Path::new(&a.file.path).exists());assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
    }
}
#[test]fn uncertain_manifest_publication_retains_recovery_and_does_not_allow_clearing_marked_sources(){
    for failed in ["markedManifest","removedManifest"]{
        let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let Payload::Copies(i)=&r.payload else{panic!()};
        let result=finish_initial_using(&r,i,&t.chat,&t.state,&t.inbox,||Transaction::create(&t.inbox,&r.provider,&r.cid,&i.manifest.records,&i.copies).map_err(|_|INCOMPLETE.to_owned()),|stage|if stage==failed{Err("syntheticManifestDirectorySyncFailure".into())}else{Ok(())}).unwrap();
        assert!(!result.complete);let id=result.recovery_id.unwrap();assert!(t.execute(t.recovery(&id),"clearCopyRecovery").is_err());let result=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(result.complete,"{failed}");assert!(result.recovery_id.is_none());assert_eq!(std::fs::read(&a.file.path).unwrap(),b"abc");let row=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records.remove(0);assert!(!row.selected);assert!(row.purge_transaction.is_none());
    }
}
#[test]fn selection_changed_after_wal_preparation_cannot_publish_marker_or_move_copies(){
    let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let r=t.review("ollama","old-cid",&["copy-a"]).unwrap();let Payload::Copies(i)=&r.payload else{panic!()};
    let result=finish_initial_using(&r,i,&t.chat,&t.state,&t.inbox,||{let tx=Transaction::create(&t.inbox,&r.provider,&r.cid,&i.manifest.records,&i.copies).unwrap();t.state.lock().unwrap().touch("ollama","old-cid");Ok(tx)},|_|Ok(())).unwrap();assert!(!result.complete);assert_eq!(std::fs::read(&a.file.path).unwrap(),b"abc");assert!(io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records[0].purge_transaction.is_none());
    let result=t.execute(t.recovery(result.recovery_id.as_ref().unwrap()),"clearCopyRecovery").unwrap();assert!(result.complete);assert!(Path::new(&a.file.path).exists());
}

#[test]fn native_terminal_review_recovers_missing_wal_or_directory_as_metadata_cleanup_only(){
 for restored in [false,true]{for stage in ["walRemoved","transactionDirectoryRemoved"]{
  let t=Temp::new();let attachment=t.seed("ollama","old-cid","copy-a");let review=t.review("ollama","old-cid",&["copy-a"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();
  if restored{let names=tx.restore_moves(&Default::default()).unwrap();publish(restore_at(&t.meta,"ollama","old-cid",&id,&tx.records(),&names).unwrap()).unwrap();tx.restored().unwrap();}
  else{publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();tx.cleanup().unwrap();}
  tx.interrupt_metadata(stage).unwrap_err();drop(tx);let manifest_before=std::fs::read(path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap();let recovery=t.recovery(&id);assert!(matches!(recovery.payload,Payload::Terminal(_)));
  let result=t.execute(recovery,"clearCopyRecovery").unwrap();assert!(result.complete,"{restored}/{stage}");assert!(result.recovery_id.is_none());assert_eq!(std::fs::read(path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap(),manifest_before);assert_eq!(Path::new(&attachment.file.path).exists(),restored);assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
 }}
}
#[test]fn terminal_recovery_cannot_reauthorize_payload_restore_or_deletion(){for consent in ["restoreOwnedCopies","finishCopyPurge","deleteOwnedCopies"]{let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let review=t.review("ollama","old-cid",&["copy-a"]).unwrap();let mut tx=t.tx(&review,false,false);let id=tx.id().to_owned();tx.abandon_prepared().unwrap();tx.interrupt_metadata("walRemoved").unwrap_err();drop(tx);assert!(t.execute(t.recovery(&id),consent).is_err());assert_eq!(std::fs::read(&a.file.path).unwrap(),b"abc");assert_eq!(quarantine::inventory(&t.inbox).unwrap().recoveries.len(),1);}}
#[test]fn terminal_review_keeps_exact_native_window_pet_and_conversation_binding(){let t=Temp::new();t.seed("ollama","old-cid","copy-a");let review=t.review("ollama","old-cid",&["copy-a"]).unwrap();let mut tx=t.tx(&review,false,false);let id=tx.id().to_owned();tx.abandon_prepared().unwrap();tx.interrupt_metadata("walRemoved").unwrap_err();drop(tx);let mut reviews=Reviews::default();let preview=reviews.insert(t.recovery(&id),Instant::now()).unwrap();for(window,provider,cid)in [("settings","ollama","old-cid"),("pet-chat","codex","old-cid"),("pet-chat","ollama","different-cid")]{assert!(reviews.take(&preview.review_id,window,provider,cid,Instant::now()).is_err());}let r=reviews.take(&preview.review_id,"pet-chat","ollama","old-cid",Instant::now()).unwrap();assert!(t.execute(r,"clearCopyRecovery").unwrap().complete);assert!(reviews.take(&preview.review_id,"pet-chat","ollama","old-cid",Instant::now()).is_err());}

#[test]fn native_multi_copy_restore_resumes_after_each_planned_or_completed_move_with_reused_names(){
 for(stage,index)in [("restoreNamesPublished",0),("restoreMoved",0),("restoreMoveDurable",0),("restoreMoved",1),("restoreMoveDurable",1)]{
  let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let b=t.seed("ollama","old-cid","copy-b");let review=t.review("ollama","old-cid",&["copy-a","copy-b"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();
  for path in [&a.file.path,&b.file.path]{std::fs::write(path,b"foreign replacement").unwrap();std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o600)).unwrap();}
  let claims=claimed_names_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap();assert!(tx.interrupt_restore(&claims,stage,index).is_err());drop(tx);
  let inventory=quarantine::inventory(&t.inbox).unwrap();assert_eq!(inventory.recoveries.len(),1);assert_eq!(inventory.recoveries[0].phase,Phase::Restoring);assert_eq!(inventory.recoveries[0].id,id);
  let result=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(result.complete,"{stage}/{index}: {:?}",result.error);assert!(result.recovery_id.is_none());
  let records=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records;assert_eq!(records.len(),2);for record in &records{assert!(["copy-a","copy-b"].contains(&record.id.as_str()));assert!(!record.selected);assert!(record.purge_transaction.is_none());assert_eq!(std::fs::read(t.inbox.join(&record.copy_name)).unwrap(),b"abc");assert!(!["copy-a.txt","copy-b.txt"].contains(&record.copy_name.as_str()));}
  assert_eq!(std::fs::read(&a.file.path).unwrap(),b"foreign replacement");assert_eq!(std::fs::read(&b.file.path).unwrap(),b"foreign replacement");assert!(t.state.lock().unwrap().contexts("ollama","old-cid").is_empty());assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
 }
}
#[test]fn native_restore_keeps_a_new_file_folder_or_symlink_occupying_the_planned_destination(){
 for mode in ["file","folder","symlink"]{
  let t=Temp::new();t.seed("ollama","old-cid","copy-a");t.seed("ollama","old-cid","copy-b");let review=t.review("ollama","old-cid",&["copy-a","copy-b"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();assert!(tx.interrupt_restore(&Default::default(),"restoreNamesPublished",0).is_err());drop(tx);
  let occupied=t.inbox.join("copy-b.txt");let foreign=t.root.join("foreign-target");std::fs::write(&foreign,b"foreign planned-name owner").unwrap();
  match mode{"file"=>{std::fs::write(&occupied,b"foreign planned-name owner").unwrap();std::fs::set_permissions(&occupied,std::fs::Permissions::from_mode(0o600)).unwrap();},"folder"=>{std::fs::DirBuilder::new().mode(0o700).create(&occupied).unwrap();std::fs::write(occupied.join("keep"),b"foreign planned-name owner").unwrap();},_=>std::os::unix::fs::symlink(&foreign,&occupied).unwrap()}
  let result=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(result.complete,"{mode}");assert_eq!(std::fs::read(&foreign).unwrap(),b"foreign planned-name owner");if mode=="folder"{assert_eq!(std::fs::read(occupied.join("keep")).unwrap(),b"foreign planned-name owner");}else{assert_eq!(std::fs::read(&occupied).unwrap(),b"foreign planned-name owner");if mode=="symlink"{assert!(std::fs::symlink_metadata(&occupied).unwrap().is_symlink());}}
  let records=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records;let record=records.iter().find(|r|r.id=="copy-b").unwrap();assert_ne!(record.copy_name,"copy-b.txt");assert_eq!(std::fs::read(t.inbox.join(&record.copy_name)).unwrap(),b"abc");
 }
}

#[test]fn native_restore_reconciles_published_manifest_with_uncertain_terminal_wal(){
 let t=Temp::new();t.seed("ollama","old-cid","copy-a");t.seed("ollama","old-cid","copy-b");let review=t.review("ollama","old-cid",&["copy-a","copy-b"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();let names=tx.restore_moves(&Default::default()).unwrap();publish(restore_at(&t.meta,"ollama","old-cid",&id,&tx.records(),&names).unwrap()).unwrap();drop(tx);
 let before=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records;let outcome=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(outcome.complete);let after=io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records;assert_eq!(before,after);assert!(after.iter().all(|r|!r.selected&&r.purge_transaction.is_none()));assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
}

impl Temp{
 fn seed_folder(&self,provider:&str,cid:&str,id:&str)->Attachment{
  use std::os::unix::ffi::OsStringExt;
  let name=format!("{id}-folder");let path=self.inbox.join(&name);std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap();let nested=path.join("nested");std::fs::DirBuilder::new().mode(0o700).create(&nested).unwrap();
  for name in [std::ffi::OsString::from("one"),std::ffi::OsString::from_vec(vec![b't',0xff])]{let file=nested.join(name);std::fs::write(&file,b"abc").unwrap();std::fs::set_permissions(&file,std::fs::Permissions::from_mode(0o600)).unwrap();}
  let record=Record{id:id.into(),copy_name:name.clone(),name:name.clone(),kind:"folder".into(),size:6,entries:3,selected:false,purge_transaction:None};let mut pending=io::prepare(&path_at(&self.meta,provider,cid).unwrap(),None,|r|{r.bind(provider,cid)?;r.records.push(record.clone());Ok(())}).unwrap();pending.publish().unwrap();pending.confirm().unwrap();attachment(provider,cid,&record,&self.inbox,&name)
 }
}
#[test]fn native_partial_recursive_cleanup_restarts_without_allowing_incomplete_restore(){
 let t=Temp::new();let folder=t.seed_folder("ollama","old-cid","folder-a");let review=t.review("ollama","old-cid",&["folder-a"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();assert!(tx.interrupt_cleanup(1).is_err());drop(tx);
 let refusal=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(!refusal.complete);assert_eq!(refusal.recovery_id.as_deref(),Some(id.as_str()));assert!(refusal.error.as_ref().unwrap().contains("verificar todas as cópias completas"));assert!(!Path::new(&folder.file.path).exists());
 let result=t.execute(t.recovery(&id),"finishCopyPurge").unwrap();assert!(result.complete);assert!(result.recovery_id.is_none());assert!(io::load_at(&path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap().registry.records.is_empty());assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
}
#[test]fn native_partial_cleanup_preserves_new_foreign_nodes_and_original_recovery(){
 let t=Temp::new();t.seed_folder("ollama","old-cid","folder-a");let review=t.review("ollama","old-cid",&["folder-a"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();assert!(tx.interrupt_cleanup(1).is_err());drop(tx);
 let added=t.inbox.join(CONTROL_ROOT).join(&id).join("payload").join("folder-a").join("foreign-node");std::fs::write(&added,b"keep foreign content").unwrap();std::fs::set_permissions(&added,std::fs::Permissions::from_mode(0o600)).unwrap();
 let result=t.execute(t.recovery(&id),"finishCopyPurge").unwrap();assert!(!result.complete);assert_eq!(result.recovery_id.as_deref(),Some(id.as_str()));assert_eq!(std::fs::read(&added).unwrap(),b"keep foreign content");assert!(result.error.as_ref().unwrap().contains("antes da limpeza"));assert!(t.execute(t.recovery(&id),"clearCopyRecovery").is_err());assert_eq!(quarantine::inventory(&t.inbox).unwrap().recoveries.len(),1);
}
#[test]fn native_restore_does_not_steal_a_partially_restored_file_claimed_by_another_record_or_cid(){
 for claimant in ["old-cid","another-cid"]{
 let t=Temp::new();t.seed("ollama","old-cid","copy-a");t.seed("ollama","old-cid","copy-b");let review=t.review("ollama","old-cid",&["copy-a","copy-b"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();assert!(tx.interrupt_restore(&Default::default(),"restoreMoveDurable",0).is_err());drop(tx);
 let foreign=Record{id:"other-id".into(),copy_name:"copy-a.txt".into(),name:"copy-a.txt".into(),kind:"file".into(),size:3,entries:1,selected:false,purge_transaction:None};let foreign_path=path_at(&t.meta,"ollama",claimant).unwrap();let mut pending=io::prepare(&foreign_path,None,|r|{r.bind("ollama",claimant)?;r.records.push(foreign);Ok(())}).unwrap();pending.publish().unwrap();pending.confirm().unwrap();let before=std::fs::read(&foreign_path).unwrap();
 let refusal=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(!refusal.complete);assert_eq!(refusal.recovery_id.as_deref(),Some(id.as_str()));assert_eq!(std::fs::read(&foreign_path).unwrap(),before);assert!(refusal.error.as_ref().unwrap().contains("outro registo ou conversa"));assert_eq!(std::fs::read(t.inbox.join("copy-a.txt")).unwrap(),b"abc");assert!(!t.inbox.join("copy-b.txt").exists());assert_eq!(quarantine::inventory(&t.inbox).unwrap().recoveries.len(),1);
}
}

#[test]fn native_partial_restore_preserves_logical_owners_for_every_metadata_provider(){for provider in crate::pet_documents::CHAT_PROVIDERS{
 let t=Temp::new();t.seed(provider,"old-cid","copy-a");t.seed(provider,"old-cid","copy-b");let review=t.review(provider,"old-cid",&["copy-a","copy-b"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,provider,"old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();assert!(tx.interrupt_restore(&Default::default(),"restoreMoveDurable",0).is_err());drop(tx);
 let result=t.execute(t.recovery(&id),"restoreOwnedCopies").unwrap();assert!(result.complete,"{provider}");let registry=io::load_at(&path_at(&t.meta,provider,"old-cid").unwrap()).unwrap().registry;assert_eq!(registry.provider,*provider);assert_eq!(registry.conversation_id,"old-cid");assert_eq!(registry.records.len(),2);assert!(registry.records.iter().all(|r|!r.selected&&r.purge_transaction.is_none()));assert!(t.state.lock().unwrap().contexts(provider,"old-cid").is_empty());
}}
#[test]fn native_restore_keeps_published_terminal_wal_after_lost_ack_and_only_clears_metadata(){
 let t=Temp::new();let a=t.seed("ollama","old-cid","copy-a");let review=t.review("ollama","old-cid",&["copy-a"]).unwrap();let mut tx=t.tx(&review,true,true);let id=tx.id().to_owned();publish(finish_at(&t.meta,"ollama","old-cid",&id,&tx.records()).unwrap()).unwrap();tx.committed().unwrap();let names=tx.restore_moves(&Default::default()).unwrap();publish(restore_at(&t.meta,"ollama","old-cid",&id,&tx.records(),&names).unwrap()).unwrap();tx.fail_next_wal_publish();assert!(tx.restored().is_err());drop(tx);
 assert_eq!(quarantine::inventory(&t.inbox).unwrap().recoveries[0].phase,Phase::Restored);let before=std::fs::read(path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap();let result=t.execute(t.recovery(&id),"clearCopyRecovery").unwrap();assert!(result.complete);assert_eq!(std::fs::read(&a.file.path).unwrap(),b"abc");assert_eq!(std::fs::read(path_at(&t.meta,"ollama","old-cid").unwrap()).unwrap(),before);assert!(quarantine::inventory(&t.inbox).unwrap().recoveries.is_empty());
}
