use super::*;
use std::os::unix::fs::{DirBuilderExt,PermissionsExt,symlink};
struct Temp{root:PathBuf,meta:PathBuf,inbox:PathBuf}
impl Temp{
    fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-retention-manifest-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();let inbox=root.join("inbox");std::fs::DirBuilder::new().mode(0o700).create(&inbox).unwrap();Self{meta:root.join("metadata"),root,inbox}}
    fn seed(&self,provider:&str,cid:&str,id:&str,name:&str){let a=Attachment{id:id.into(),provider:provider.into(),conversation_id:cid.into(),sequence:1,file:crate::files::DroppedFile{name:name.into(),path:self.inbox.join(name).to_string_lossy().into(),size:3,kind:"file".into(),entries:1}};
        let mut p=append_at(&self.meta,&self.inbox,provider,cid,&[a]).unwrap();p.publish().unwrap();p.confirm().unwrap();let mut p=io::prepare(&self.path(provider,cid),None,|r|{for v in &mut r.records{v.selected=false;}Ok(())}).unwrap();p.publish().unwrap();p.confirm().unwrap();}
    fn path(&self,p:&str,c:&str)->PathBuf{path_at(&self.meta,p,c).unwrap()}
    fn review(&self,p:&str,c:&str,id:&str)->Result<Review,&'static str>{Review::at(&self.meta,p,c,&[id.into()],&io::load_at(&self.path(p,c))?.revision)}
    fn mark(&self,r:&Review){let mut p=r.mark("tx-1").unwrap();p.publish().unwrap();p.confirm().unwrap();}
}
impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
#[test]fn metadata_catalog_includes_orphan_cids_and_every_provider_without_grants(){
    let t=Temp::new();assert!(catalog_at(&t.meta).unwrap().is_empty());assert!(!t.meta.exists());
    for (i,p) in CHAT_PROVIDERS.iter().enumerate(){t.seed(p,&format!("orphan-{i}"),&format!("id-{i}"),&format!("copy-{i}.txt"));}
    let owners=catalog_at(&t.meta).unwrap();assert_eq!(owners.len(),8);assert!(owners.iter().all(|o|o.items.len()==1&&!o.items[0].selected&&!o.items[0].purging));
    let json=serde_json::to_string(&owners).unwrap();assert!(!json.contains(t.inbox.to_str().unwrap()));assert!(!json.contains("copyName"));assert!(Store::default().files.is_empty());
}
#[test]fn shared_ids_names_selected_and_reserved_copies_cannot_be_reviewed(){
    for shared in ["id","name"]{let t=Temp::new();t.seed("ollama","a","id-a","a.txt");t.seed("codex","b",if shared=="id"{"id-a"}else{"id-b"},if shared=="name"{"a.txt"}else{"b.txt"});assert!(t.review("ollama","a","id-a").is_err());}
    let t=Temp::new();t.seed("ollama","a","id-a","a.txt");let mut p=io::prepare(&t.path("ollama","a"),None,|r|{r.records[0].selected=true;Ok(())}).unwrap();p.publish().unwrap();p.confirm().unwrap();assert!(t.review("ollama","a","id-a").is_err());
    let t=Temp::new();t.seed("ollama","a","id-a",CONTROL_ROOT);assert!(t.review("ollama","a","id-a").is_err());
}
#[test]fn unrelated_owner_work_preserves_review_but_new_shared_claim_revokes_it(){
    let t=Temp::new();t.seed("ollama","a","id-a","a.txt");let review=t.review("ollama","a","id-a").unwrap();t.seed("codex","b","id-b","b.txt");let staged=review.mark("tx-1").unwrap();drop(staged);
    t.seed("openai","c","id-c","a.txt");assert!(review.mark("tx-1").is_err());assert!(io::load_at(&t.path("ollama","a")).unwrap().registry.records[0].purge_transaction.is_none());
}
#[test]fn pending_marker_blocks_renewal_append_and_old_reader_and_preserves_unselected_rows(){
    let t=Temp::new();t.seed("ollama","a","id-a","a.txt");t.seed("ollama","a","id-b","b.txt");let review=t.review("ollama","a","id-a").unwrap();t.mark(&review);
    let now=io::load_at(&t.path("ollama","a")).unwrap();assert_eq!(now.registry.records[0].purge_transaction.as_deref(),Some("tx-1"));assert!(now.registry.records[1].purge_transaction.is_none());
    assert!(super::super::restore_at(&t.meta,&t.inbox,"ollama","a",&["id-b".into()],&now.revision).is_err());
    let files=review.attachments(&t.inbox);assert!(append_at(&t.meta,&t.inbox,"ollama","a",&files).is_err());
    #[derive(serde::Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]#[allow(dead_code)]struct OldRecord{id:String,copy_name:String,name:String,kind:String,size:u64,entries:usize,selected:bool}
    assert!(serde_json::from_value::<OldRecord>(serde_json::to_value(&now.registry.records[0]).unwrap()).is_err());
    let mut bad=now.registry.clone();bad.records[0].selected=true;assert!(bad.validate().is_err());
}
#[test]fn finish_and_restore_are_exact_idempotent_and_never_alias_foreign_claims(){
    let t=Temp::new();t.seed("ollama","a","id-a","a.txt");t.seed("ollama","a","id-b","b.txt");let review=t.review("ollama","a","id-a").unwrap();t.mark(&review);
    for _ in 0..2{let mut p=finish_at(&t.meta,"ollama","a","tx-1",&review.records).unwrap();p.publish().unwrap();p.confirm().unwrap();}
    let names=BTreeMap::from([("id-a".into(),"recovered.txt".into())]);t.seed("codex","b","foreign","recovered.txt");assert!(restore_at(&t.meta,"ollama","a","tx-1",&review.records,&names).is_err());
    let names=BTreeMap::from([("id-a".into(),"available.txt".into())]);for _ in 0..2{let mut p=restore_at(&t.meta,"ollama","a","tx-1",&review.records,&names).unwrap();p.publish().unwrap();p.confirm().unwrap();}
    let r=io::load_at(&t.path("ollama","a")).unwrap().registry;assert_eq!(r.records.len(),2);assert_eq!(r.records[0].id,"id-b");assert_eq!(r.records[1].id,"id-a");assert_eq!(r.records[1].copy_name,"available.txt");assert!(!r.records[1].selected);assert!(r.records[1].purge_transaction.is_none());
    assert!(finish_at(&t.meta,"ollama","a","tx-1",&review.records).is_err());
}
#[test]fn unknown_corrupt_linked_wrong_owner_or_shared_catalog_is_preserved_and_refused(){
    for mode in ["unknown","corrupt","link","owner","shared","unicode"]{let t=Temp::new();t.seed("ollama","a","id-a","a.txt");match mode{
        "unknown"=>std::fs::write(t.meta.join("other.json"),b"unknown").unwrap(),"corrupt"=>std::fs::write(t.path("ollama","a"),b"bad").unwrap(),
        "link"=>{std::fs::remove_file(t.path("ollama","a")).unwrap();symlink("/etc/passwd",t.path("ollama","a")).unwrap();},
        "owner"=>{let mut r=io::load_at(&t.path("ollama","a")).unwrap().registry;r.conversation_id="other".into();std::fs::write(t.path("ollama","a"),serde_json::to_vec(&r).unwrap()).unwrap();},
        "shared"=>std::fs::set_permissions(&t.meta,std::fs::Permissions::from_mode(0o755)).unwrap(),_=>{std::fs::write(t.meta.join(format!("{}x.json","é".repeat(32))),b"unknown").unwrap();},
    }assert!(catalog_at(&t.meta).is_err(),"{mode}");assert!(t.path("ollama","a").symlink_metadata().is_ok());}
}
