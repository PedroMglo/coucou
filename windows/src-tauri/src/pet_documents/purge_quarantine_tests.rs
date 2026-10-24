use super::*;
use std::os::unix::{ffi::OsStringExt,fs::{DirBuilderExt,PermissionsExt,symlink}};
struct Temp{path:PathBuf,meta:PathBuf}
impl Temp{
    fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-quarantine-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap();Self{meta:path.join("metadata"),path}}
    fn file(&self,id:&str,name:&str,bytes:&[u8])->(Record,Copy){
        let path=self.path.join(name);std::fs::write(&path,bytes).unwrap();std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o600)).unwrap();
        let r=Record{id:id.into(),copy_name:name.into(),name:name.into(),size:bytes.len()as u64,kind:"file".into(),entries:1,selected:false,purge_transaction:None};
        let c=Copy::capture(&self.path,&attachment("ollama","cid",&r,&self.path,name),&mut Budget::new()).unwrap();(r,c)
    }
    fn folder(&self)->(Record,Copy){
        let path=self.path.join("folder");std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap();let nested=path.join("nested");std::fs::DirBuilder::new().mode(0o700).create(&nested).unwrap();
        for name in [OsString::from("a.txt"),OsString::from_vec(vec![b'b',0xff])]{let p=nested.join(name);std::fs::write(&p,b"abc").unwrap();std::fs::set_permissions(p,std::fs::Permissions::from_mode(0o600)).unwrap();}
        let r=Record{id:"folder-id".into(),copy_name:"folder".into(),name:"folder".into(),size:6,kind:"folder".into(),entries:3,selected:false,purge_transaction:None};
        let c=Copy::capture(&self.path,&attachment("ollama","cid",&r,&self.path,"folder"),&mut Budget::new()).unwrap();(r,c)
    }
    fn tx(&self,values:Vec<(Record,Copy)>)->Transaction{let(records,copies):(Vec<_>,Vec<_>)=values.into_iter().unzip();Transaction::create(&self.path,"ollama","cid",&records,&copies).unwrap()}
    fn payload(&self,id:&str,item:&str)->PathBuf{self.path.join(CONTROL_ROOT).join(id).join("payload").join(item)}
}
impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.path);}}
fn publish(mut pending:Pending){pending.publish().unwrap();pending.confirm().unwrap();}
fn register(t:&Temp,records:&[Record]){
    let path=path_at(&t.meta,"ollama","cid").unwrap();let mut pending=io::prepare(&path,None,|r|{r.bind("ollama","cid")?;r.records.extend_from_slice(records);Ok(())}).unwrap();pending.publish().unwrap();pending.confirm().unwrap();
}
#[test]fn exact_manifest_quarantine_delete_flow_keeps_other_records_and_legacy_data(){
    let t=Temp::new();let(a,ac)=t.file("a-id","a.txt",b"abc");let(b,_)=t.file("b-id","b.txt",b"def");register(&t,&[a.clone(),b.clone()]);t.file("legacy","legacy.txt",b"keep");
    let path=path_at(&t.meta,"ollama","cid").unwrap();let rev=io::load_at(&path).unwrap().revision;let review=Review::at(&t.meta,"ollama","cid",&[a.id.clone()],&rev).unwrap();let mut tx=Transaction::create(&t.path,"ollama","cid",&review.records,&[ac]).unwrap();
    publish(review.mark(tx.id()).unwrap());tx.marked().unwrap();tx.move_all().unwrap();publish(finish_at(&t.meta,"ollama","cid",tx.id(),&tx.records()).unwrap());tx.committed().unwrap();tx.cleanup().unwrap();
    assert_eq!(io::load_at(&path).unwrap().registry.records,vec![b]);assert_eq!(std::fs::read(t.path.join("b.txt")).unwrap(),b"def");assert_eq!(std::fs::read(t.path.join("legacy.txt")).unwrap(),b"keep");assert!(!t.path.join("a.txt").exists());
}
#[test]fn restart_restore_after_manifest_removal_preserves_reused_copy_and_original_id(){
    let t=Temp::new();let(a,ac)=t.file("a-id","a.txt",b"abc");register(&t,&[a.clone()]);let path=path_at(&t.meta,"ollama","cid").unwrap();let review=Review::at(&t.meta,"ollama","cid",&[a.id.clone()],&io::load_at(&path).unwrap().revision).unwrap();
    let mut tx=Transaction::create(&t.path,"ollama","cid",&review.records,&[ac]).unwrap();let id=tx.id().to_owned();publish(review.mark(&id).unwrap());tx.marked().unwrap();tx.move_all().unwrap();publish(finish_at(&t.meta,"ollama","cid",&id,&tx.records()).unwrap());drop(tx);
    t.file("foreign","a.txt",b"new");let mut tx=Transaction::open(&t.path,&id).unwrap();let names=tx.restore_moves(&BTreeSet::from(["a.txt".into()])).unwrap();publish(restore_at(&t.meta,"ollama","cid",&id,&tx.records(),&names).unwrap());tx.restored().unwrap();
    let records=io::load_at(&path).unwrap().registry.records;assert_eq!(records.len(),1);assert_eq!(records[0].id,"a-id");assert!(!records[0].selected);assert!(records[0].purge_transaction.is_none());assert_ne!(records[0].copy_name,"a.txt");assert_eq!(std::fs::read(t.path.join(&records[0].copy_name)).unwrap(),b"abc");assert_eq!(std::fs::read(t.path.join("a.txt")).unwrap(),b"new");
}
#[test]fn wal_is_private_metadata_only_and_unknown_control_directory_is_not_adopted(){
    let t=Temp::new();let a=t.file("a-id","a.txt",b"secret content");let tx=t.tx(vec![a]);let id=tx.id().to_owned();assert_eq!(tx.phase(),&Phase::Prepared);assert!(t.path.join("a.txt").exists());
    let wal=t.path.join(CONTROL_ROOT).join(&id).join("transaction.json");let bytes=std::fs::read(&wal).unwrap();let text=String::from_utf8(bytes).unwrap();assert!(!text.contains("secret content"));assert!(!text.contains(t.path.to_str().unwrap()));assert!(!text.contains("grant"));assert_eq!(std::fs::metadata(wal).unwrap().mode()&0o777,0o600);
    assert!(Transaction::open(&t.path,&id).is_err());drop(tx);assert_eq!(Transaction::open(&t.path,&id).unwrap().phase(),&Phase::Prepared);
    let t=Temp::new();std::fs::DirBuilder::new().mode(0o700).create(t.path.join(CONTROL_ROOT)).unwrap();std::fs::write(t.path.join(CONTROL_ROOT).join("foreign"),b"keep").unwrap();let(r,c)=t.file("a","a.txt",b"abc");assert!(Transaction::create(&t.path,"ollama","cid",&[r],&[c]).is_err());assert_eq!(std::fs::read(t.path.join(CONTROL_ROOT).join("foreign")).unwrap(),b"keep");assert!(t.path.join("a.txt").exists());
}
#[test]fn move_requires_marker_preserves_legacy_and_restarts_a_partial_move(){
    let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let b=t.file("b-id","b.txt",b"def");let mut tx=t.tx(vec![a,b]);let id=tx.id().to_owned();assert!(tx.move_all().is_err());tx.marked().unwrap();
    assert!(tx.move_using(|index|if index==1{Err("injected")}else{Ok(())}).is_err());assert!(!t.path.join("a.txt").exists());assert!(t.path.join("b.txt").exists());assert!(t.payload(&id,"a-id").exists());
    t.file("legacy","unindexed.txt",b"keep");drop(tx);let mut tx=Transaction::open(&t.path,&id).unwrap();tx.move_all().unwrap();assert_eq!(tx.phase(),&Phase::Moved);assert!(tx.cleanup().is_err());tx.committed().unwrap();tx.cleanup().unwrap();assert_eq!(tx.phase(),&Phase::Cleaned);assert!(!t.payload(&id,"a-id").exists());assert!(!t.payload(&id,"b-id").exists());assert_eq!(std::fs::read(t.path.join("unindexed.txt")).unwrap(),b"keep");
}
#[test]fn destination_collision_source_replacement_and_postreview_change_preserve_every_copy(){
    for mode in ["destination","replacement","edit"]{
        let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let mut tx=t.tx(vec![a]);let id=tx.id().to_owned();tx.marked().unwrap();match mode{
            "destination"=>{let p=t.payload(&id,"a-id");std::fs::write(&p,b"other").unwrap();std::fs::set_permissions(p,std::fs::Permissions::from_mode(0o600)).unwrap();},
            "replacement"=>{std::fs::rename(t.path.join("a.txt"),t.path.join("old.txt")).unwrap();t.file("other","a.txt",b"new");},
            _=>{std::fs::write(t.path.join("a.txt"),b"edit").unwrap();},
        }
        assert!(tx.move_all().is_err(),"{mode}");assert!(t.path.join("a.txt").exists());if mode=="destination"{assert_eq!(std::fs::read(t.payload(&id,"a-id")).unwrap(),b"other");}
    }
}
#[test]fn restore_never_overwrites_reused_file_symlink_or_claimed_absent_basename(){
    for mode in ["file","symlink","claim"]{
        let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let mut tx=t.tx(vec![a]);let id=tx.id().to_owned();tx.marked().unwrap();tx.move_all().unwrap();let mut claimed=BTreeSet::new();match mode{
            "file"=>{t.file("new","a.txt",b"new");},"symlink"=>symlink("/etc/passwd",t.path.join("a.txt")).unwrap(),_=>{claimed.insert("a.txt".into());},
        }
        let names=tx.restore_moves(&claimed).unwrap();let restored=&names["a-id"];assert_ne!(restored,"a.txt");assert_eq!(std::fs::read(t.path.join(restored)).unwrap(),b"abc");if mode=="file"{assert_eq!(std::fs::read(t.path.join("a.txt")).unwrap(),b"new");}if mode=="symlink"{assert!(std::fs::symlink_metadata(t.path.join("a.txt")).unwrap().is_symlink());}
        drop(tx);let mut tx=Transaction::open(&t.path,&id).unwrap();assert_eq!(tx.restore_moves(&claimed).unwrap(),names);tx.restored().unwrap();assert_eq!(tx.phase(),&Phase::Restored);assert!(tx.cleanup().is_err());
    }
}
#[test]fn prepared_and_partial_move_can_restore_without_inventing_grants(){
    let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let b=t.file("b-id","b.txt",b"def");let mut tx=t.tx(vec![a,b]);tx.marked().unwrap();tx.move_using(|i|if i==1{Err("injected")}else{Ok(())}).unwrap_err();
    let names=tx.restore_moves(&BTreeSet::new()).unwrap();assert_eq!(names["a-id"],"a.txt");assert_eq!(names["b-id"],"b.txt");assert_eq!(std::fs::read(t.path.join("a.txt")).unwrap(),b"abc");assert_eq!(std::fs::read(t.path.join("b.txt")).unwrap(),b"def");assert!(tx.move_all().is_err());
    let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let mut tx=t.tx(vec![a]);assert_eq!(tx.restore_moves(&BTreeSet::new()).unwrap()["a-id"],"a.txt");
}
#[test]fn nested_non_utf8_tree_restores_and_partial_delete_restarts_but_cannot_restore(){
    let t=Temp::new();let a=t.folder();let mut tx=t.tx(vec![a]);let id=tx.id().to_owned();tx.marked().unwrap();tx.move_all().unwrap();let names=tx.restore_moves(&BTreeSet::new()).unwrap();assert_eq!(names["folder-id"],"folder");assert!(t.path.join("folder/nested").join(OsString::from_vec(vec![b'b',0xff])).exists());
    let t=Temp::new();let a=t.folder();let mut tx=t.tx(vec![a]);let id=tx.id().to_owned();tx.marked().unwrap();tx.move_all().unwrap();tx.committed().unwrap();assert!(tx.cleanup_using(|removed|if removed==1{Err("injected")}else{Ok(())}).is_err());drop(tx);
    let mut tx=Transaction::open(&t.path,&id).unwrap();assert!(tx.restore_moves(&BTreeSet::new()).is_err());tx.cleanup().unwrap();assert!(!t.payload(&id,"folder-id").exists());assert!(Transaction::open(&t.path,&id).is_err());drop(tx);assert_eq!(Transaction::open(&t.path,&id).unwrap().phase(),&Phase::Cleaned);
}
#[test]fn changed_foreign_symlink_hardlink_and_extra_nodes_prevent_cleanup(){
    for mode in ["edit","extra","symlink","hardlink"]{
        let t=Temp::new();let a=t.folder();let mut tx=t.tx(vec![a]);let id=tx.id().to_owned();tx.marked().unwrap();tx.move_all().unwrap();tx.committed().unwrap();let p=t.payload(&id,"folder-id").join("nested");match mode{
            "edit"=>std::fs::write(p.join("a.txt"),b"changed").unwrap(),"extra"=>{std::fs::write(p.join("extra"),b"keep").unwrap();std::fs::set_permissions(p.join("extra"),std::fs::Permissions::from_mode(0o600)).unwrap();},
            "symlink"=>{std::fs::remove_file(p.join("a.txt")).unwrap();symlink("/etc/passwd",p.join("a.txt")).unwrap();},_=>std::fs::hard_link(p.join("a.txt"),t.path.join("link")).unwrap(),
        }assert!(tx.cleanup().is_err(),"{mode}");assert!(p.join("a.txt").symlink_metadata().is_ok());assert!(p.join(OsString::from_vec(vec![b'b',0xff])).exists());
    }
}
#[test]fn invalid_wal_shared_permissions_or_root_replacement_cannot_resume(){
    for mode in ["wal","permission","root"]{
        let t=Temp::new();let a=t.file("a-id","a.txt",b"abc");let tx=t.tx(vec![a]);let id=tx.id().to_owned();drop(tx);let root=t.path.join(CONTROL_ROOT);match mode{
            "wal"=>std::fs::write(root.join(&id).join("transaction.json"),b"{\"version\":99}").unwrap(),"permission"=>std::fs::set_permissions(root.join(&id),std::fs::Permissions::from_mode(0o755)).unwrap(),_=>{std::fs::rename(&root,t.path.join("preserved")).unwrap();symlink(t.path.join("preserved"),&root).unwrap();},
        }assert!(Transaction::open(&t.path,&id).is_err());assert_eq!(std::fs::read(t.path.join("a.txt")).unwrap(),b"abc");
    }
}
