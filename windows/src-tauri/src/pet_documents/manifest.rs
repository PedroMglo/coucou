//! Durable copy metadata is not a read grant. Records are retained when an item
//! is deselected so historical turns cannot lose their source through expiry.
use super::*;
use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
#[path="manifest_io.rs"]
mod io;
#[path="manifest_purge.rs"]
pub(super) mod purge;
const MAX_BYTES:usize=1024*1024;
const MAX_RECORDS:usize=1024;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Record {
    id:String, copy_name:String, name:String, kind:String,
    size:u64, entries:usize, selected:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    purge_transaction:Option<String>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Registry {
    version:u32, generation:u64, provider:String, conversation_id:String,
    records:Vec<Record>,
}
fn component(name:&str)->bool{
    !name.is_empty()&&name.len()<=4096&&!name.chars().any(char::is_control)
        &&Path::new(name).components().count()==1&&matches!(Path::new(name).components().next(),Some(Component::Normal(_)))
}
fn owner(provider:&str,cid:&str)->Result<(),&'static str>{
    if crate::pet_config::integration_pet(provider).is_none()||!crate::assistant_memory::valid_id(cid){Err("invalidOwner")}else{Ok(())}
}
impl Registry {
    fn empty()->Self{Self{version:1,generation:0,provider:String::new(),conversation_id:String::new(),records:vec![]}}
    fn validate(&self)->Result<(),&'static str>{
        owner(&self.provider,&self.conversation_id)?;
        if self.version!=1||self.records.len()>MAX_RECORDS{return Err("registryLimit");}
        let mut ids=BTreeSet::new();let mut copies=BTreeSet::new();let mut selected=0;let mut bytes=0u64;
        for r in &self.records{
            if r.purge_transaction.as_ref().is_some_and(|id|!crate::assistant_memory::valid_id(id)||r.selected){return Err("invalidRegistry");}
            if !crate::assistant_memory::valid_id(&r.id)||!ids.insert(&r.id)||!copies.insert(&r.copy_name)||!component(&r.copy_name)||!component(&r.name)
                ||!matches!(r.kind.as_str(),"file"|"folder")||r.size>MAX_CONVERSATION_BYTES||r.entries>32768{return Err("invalidRegistry");}
            if r.selected{selected+=1;bytes=bytes.checked_add(r.size).ok_or("registryLimit")?;}
        }
        if selected>MAX_CONVERSATION_ATTACHMENTS||bytes>MAX_CONVERSATION_BYTES{return Err("registryLimit");}Ok(())
    }
    fn next(mut self,previous:&Self)->Result<Self,&'static str>{
        if self.generation!=previous.generation{return Err("registryConflict");}
        if previous.generation>0&&(self.provider!=previous.provider||self.conversation_id!=previous.conversation_id){return Err("invalidOwner");}
        self.generation=previous.generation.checked_add(1).ok_or("registryLimit")?;self.validate()?;Ok(self)
    }
    fn bind(&mut self,provider:&str,cid:&str)->Result<(),&'static str>{
        owner(provider,cid)?;
        if self.generation==0{self.provider=provider.into();self.conversation_id=cid.into();}
        if self.provider!=provider||self.conversation_id!=cid{return Err("invalidOwner");}Ok(())
    }
}
fn path_at(directory:&Path,provider:&str,cid:&str)->Result<PathBuf,&'static str>{
    owner(provider,cid)?;
    let identity=serde_json::to_vec(&(provider,cid)).map_err(|_|"invalidOwner")?;
    let digest=ring::digest::digest(&ring::digest::SHA256,&identity).as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>();
    Ok(directory.join(format!("{digest}.json")))
}
fn directory()->PathBuf{crate::platform::data_dir().join("assistant").join("attachments")}
fn record(a:&Attachment,inbox:&Path)->Result<Record,&'static str>{
    let path=Path::new(&a.file.path);
    if path.parent()!=Some(inbox){return Err("invalidCopy");}
    let copy_name=path.file_name().and_then(|s|s.to_str()).filter(|s|component(s)).ok_or("invalidCopy")?;
    Ok(Record{id:a.id.clone(),copy_name:copy_name.into(),name:a.file.name.clone(),kind:a.file.kind.clone(),size:a.file.size,entries:a.file.entries,selected:true,purge_transaction:None})
}
fn append_at(directory:&Path,inbox:&Path,provider:&str,cid:&str,added:&[Attachment])->Result<io::Prepared,&'static str>{
    if added.is_empty(){return Err("invalidRegistry");}
    let records=added.iter().map(|a|{
        if a.provider!=provider||a.conversation_id!=cid{return Err("invalidOwner");}record(a,inbox)
    }).collect::<Result<Vec<_>,_>>()?;
    io::prepare(&path_at(directory,provider,cid)?,None,|r|{r.bind(provider,cid)?;if r.records.iter().any(|r|r.purge_transaction.is_some()){return Err("registryPurging");}r.records.extend(records);Ok(())})
}
pub(super) struct Pending(io::Prepared);
impl Pending{
    pub(super) fn publish(&mut self)->Result<(),String>{self.0.publish().map_err(message)}
    pub(super) fn confirm(self)->Result<(),String>{self.0.confirm().map(|_|()).map_err(message)}
}
pub(super) fn append(provider:&str,cid:&str,added:&[Attachment])->Result<Pending,String>{
    append_at(&directory(),&crate::files::inbox_dir(),provider,cid,added).map(Pending).map_err(message)
}
pub(super) fn ensure_not_purging(provider:&str,cid:&str)->Result<(),String>{
    let snapshot=io::load_at(&path_at(&directory(),provider,cid).map_err(message)?).map_err(message)?;
    if snapshot.registry.records.iter().any(|r|r.purge_transaction.is_some()){Err(message("registryPurging"))}else{Ok(())}
}
pub(super) fn deselect(provider:&str,cid:&str,ids:&[String])->Result<(),String>{
    if ids.is_empty(){return Ok(());}
    let mut staged=io::prepare(&path_at(&directory(),provider,cid).map_err(message)?,None,|r|{
        r.bind(provider,cid)?;for item in &mut r.records{if ids.contains(&item.id){item.selected=false;}}Ok(())
    }).map_err(message)?;
    staged.publish().map_err(message)?;staged.confirm().map(|_|()).map_err(message)
}
pub(super) fn deselect_owner(provider:&str,cid:&str)->Result<(),String>{
    // History-only providers without a pet manifest have nothing to deselect.
    if crate::pet_config::integration_pet(provider).is_none(){return Ok(());}
    let path=path_at(&directory(),provider,cid).map_err(message)?;let snapshot=io::load_at(&path).map_err(message)?;
    if snapshot.revision=="missing"||!snapshot.registry.records.iter().any(|r|r.selected){return Ok(());}
    let mut staged=io::prepare(&path,Some(&snapshot.revision),|r|{for record in &mut r.records{record.selected=false;}Ok(())}).map_err(message)?;
    staged.publish().map_err(message)?;staged.confirm().map(|_|()).map_err(message)
}
fn message(code:&str)->String{
    match code{
        "registryConflict"=>"O registo de anexos mudou. Volta a carregar a conversa antes de repetir.",
        "registryBusy"=>"Outro pedido está a guardar os anexos desta conversa. Aguarda e volta a carregar.",
        "registryPurging"=>"Há uma operação de retenção por resolver nesta conversa. Revê a recuperação antes de entregar ou autorizar anexos.",
        "registryLimit"=>"O registo de anexos atingiu o limite desta conversa. Os anexos anteriores foram preservados.",
        "registryCommitUncertain"=>"Não foi possível confirmar a gravação dos anexos. Volta a carregar a conversa antes de repetir a entrega.",
        _=>"O registo de anexos está indisponível ou é inválido. Os dados anteriores foram preservados.",
    }.into()
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Inventory {revision:String,records:Vec<Recovered>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Recovered {id:String,name:String,kind:String,size:u64,entries:usize,selected:bool,authorized:bool,purging:bool}
/// Metadata only: listing does not open a copy, populate GRANTS or publish an
/// attachment in the native active collection. Renewal needs a separate action.
pub(super) fn inventory(provider:&str,cid:&str,active:&[Attachment])->Result<Inventory,String>{
    let snapshot=io::load_at(&path_at(&directory(),provider,cid).map_err(message)?).map_err(message)?;
    let mut r=snapshot.registry;r.bind(provider,cid).map_err(message)?;
    Ok(Inventory{revision:snapshot.revision,records:r.records.into_iter().map(|r|Recovered{
        authorized:r.purge_transaction.is_none()&&active.iter().any(|a|a.id==r.id),purging:r.purge_transaction.is_some(),id:r.id,name:r.name,kind:r.kind,size:r.size,entries:r.entries,selected:r.selected,
    }).collect()})
}

pub(super) struct Restoration{snapshot:io::Snapshot,path:PathBuf,pub files:Vec<Attachment>}
impl Restoration{
    pub(super) fn current(&self)->bool{self.snapshot.unchanged()}
    pub(super) fn prepare_commit(&self)->Result<Pending,String>{
        if !self.current(){return Err(message("registryConflict"));}
        let ids=self.files.iter().map(|a|a.id.as_str()).collect::<Vec<_>>();
        io::prepare(&self.path,Some(&self.snapshot.revision),|r|{
            if r.records.iter().any(|r|r.purge_transaction.is_some()){return Err("registryPurging");}
            for id in &ids{r.records.iter_mut().find(|v|v.id==*id).ok_or("registryConflict")?.selected=true;}Ok(())
        }).map(Pending).map_err(message)
    }
}
fn restore_at(directory:&Path,inbox:&Path,provider:&str,cid:&str,ids:&[String],expected:&str)->Result<Restoration,String>{
    let mut unique=BTreeSet::new();
    if ids.is_empty()||ids.len()>MAX_CONVERSATION_ATTACHMENTS||ids.iter().any(|id|!crate::assistant_memory::valid_id(id)||!unique.insert(id)){
        return Err("Escolhe entre um e 32 anexos distintos para rever.".into());
    }
    let path=path_at(directory,provider,cid).map_err(message)?;let snapshot=io::load_at(&path).map_err(message)?;
    if expected=="missing"||snapshot.revision!=expected||!snapshot.unchanged(){return Err(message("registryConflict"));}
    let mut r=snapshot.registry.clone();r.bind(provider,cid).map_err(message)?;
    if r.records.iter().any(|r|r.purge_transaction.is_some()){return Err(message("registryPurging"));}
    let files=ids.iter().map(|id|{
        let r=r.records.iter().find(|r|r.id==*id).ok_or_else(||message("registryConflict"))?;
        Ok(Attachment{id:r.id.clone(),provider:provider.into(),conversation_id:cid.into(),sequence:0,file:crate::files::DroppedFile{
            name:r.name.clone(),path:inbox.join(&r.copy_name).to_string_lossy().into_owned(),kind:r.kind.clone(),size:r.size,entries:r.entries,
        }})
    }).collect::<Result<Vec<_>,String>>()?;
    Ok(Restoration{snapshot,path,files})
}
pub(super) fn restore(provider:&str,cid:&str,ids:&[String],expected:&str)->Result<Restoration,String>{restore_at(&directory(),&crate::files::inbox_dir(),provider,cid,ids,expected)}

#[cfg(test)]
pub(super) fn test_restoration(directory:&Path,inbox:&Path,files:&[Attachment])->Restoration{
    let owner=files.first().unwrap();let mut staged=append_at(directory,inbox,&owner.provider,&owner.conversation_id,files).unwrap();staged.publish().unwrap();let saved=staged.confirm().unwrap();
    restore_at(directory,inbox,&owner.provider,&owner.conversation_id,&files.iter().map(|a|a.id.clone()).collect::<Vec<_>>(),&saved.revision).unwrap()
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::os::unix::fs::{PermissionsExt,symlink};
    struct Temp{root:PathBuf,inbox:PathBuf,meta:PathBuf}
    impl Temp{
        fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-manifest-{}",crate::assistant_memory::new_id()));let inbox=root.join("inbox");let meta=root.join("metadata");std::fs::create_dir_all(&inbox).unwrap();Self{root,inbox,meta}}
        fn file(&self,id:&str)->Attachment{Attachment{id:id.into(),provider:"ollama".into(),conversation_id:"conversation-1".into(),sequence:1,file:crate::files::DroppedFile{name:"nota.txt".into(),path:self.inbox.join(format!("{id}.txt")).to_string_lossy().into_owned(),size:5,kind:"file".into(),entries:1}}}
        fn path(&self)->PathBuf{path_at(&self.meta,"ollama","conversation-1").unwrap()}
        fn publish(&self,added:&[Attachment]){let mut p=append_at(&self.meta,&self.inbox,"ollama","conversation-1",added).unwrap();p.publish().unwrap();p.confirm().unwrap();}
    }
    impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
    #[test]
    fn restoration_requires_exact_revision_owner_and_ids_without_falling_back_to_first(){
        let t=Temp::new();t.publish(&[t.file("a"),t.file("b")]);let old=io::load_at(&t.path()).unwrap();
        let selected=restore_at(&t.meta,&t.inbox,"ollama","conversation-1",&["b".into()],&old.revision).unwrap();assert_eq!(selected.files.len(),1);assert_eq!(selected.files[0].id,"b");assert_eq!(selected.files[0].sequence,0);
        for ids in [vec![],vec!["a".into(),"a".into()],vec!["unknown".into()]]{assert!(restore_at(&t.meta,&t.inbox,"ollama","conversation-1",&ids,&old.revision).is_err());}
        assert!(restore_at(&t.meta,&t.inbox,"codex","conversation-1",&["b".into()],&old.revision).is_err());t.publish(&[t.file("c")]);assert!(!selected.current());assert!(selected.prepare_commit().is_err());
    }
    #[test]
    fn renewed_selection_is_staged_and_published_for_only_reviewed_ids(){
        let t=Temp::new();t.publish(&[t.file("a"),t.file("b")]);let mut edit=io::prepare(&t.path(),None,|r|{for v in &mut r.records{v.selected=false;}Ok(())}).unwrap();edit.publish().unwrap();edit.confirm().unwrap();
        let old=io::load_at(&t.path()).unwrap();let review=restore_at(&t.meta,&t.inbox,"ollama","conversation-1",&["b".into()],&old.revision).unwrap();let mut staged=review.prepare_commit().unwrap();assert!(io::load_at(&t.path()).unwrap().registry.records.iter().all(|r|!r.selected));staged.publish().unwrap();staged.confirm().unwrap();
        let next=io::load_at(&t.path()).unwrap();assert!(!next.registry.records[0].selected);assert!(next.registry.records[1].selected);assert_eq!(next.registry.records.len(),2);
    }
    #[test]
    fn absence_is_read_only_and_metadata_survives_reopen_without_original_paths_or_grants(){
        let t=Temp::new();assert_eq!(io::load_at(&t.path()).unwrap().revision,"missing");assert!(!t.meta.exists());
        t.publish(&[t.file("a"),t.file("b")]);let saved=io::load_at(&t.path()).unwrap();assert_eq!(saved.registry.records.len(),2);assert_eq!(saved.registry.generation,1);assert_eq!(saved.registry.records[0].copy_name,"a.txt");
        let text=std::fs::read_to_string(t.path()).unwrap();assert!(!text.contains(t.inbox.to_str().unwrap()));assert!(!text.contains("sequence"));assert!(!text.contains("dropId"));assert_eq!(std::fs::metadata(t.path()).unwrap().permissions().mode()&0o777,0o600);
        t.publish(&[t.file("c")]);assert_eq!(io::load_at(&t.path()).unwrap().registry.records.len(),3);assert!(!saved.unchanged());
    }
    #[test]
    fn staging_is_not_publication_and_cancellation_preserves_prior_bytes(){
        let t=Temp::new();t.publish(&[t.file("a")]);let before=std::fs::read(t.path()).unwrap();let staged=append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("b")]).unwrap();assert_eq!(std::fs::read(t.path()).unwrap(),before);drop(staged);assert_eq!(std::fs::read(t.path()).unwrap(),before);
        assert!(!std::fs::read_dir(&t.meta).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    }
    #[test]
    fn competing_writer_rejects_without_losing_or_reordering_records(){
        let t=Temp::new();t.publish(&[t.file("a")]);let first=append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("b")]).unwrap();assert!(matches!(append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("c")]),Err("registryBusy")));drop(first);t.publish(&[t.file("c")]);assert_eq!(io::load_at(&t.path()).unwrap().registry.records.iter().map(|r|r.id.as_str()).collect::<Vec<_>>(),vec!["a","c"]);
    }
    #[test]
    fn corrupt_shared_linked_or_oversize_metadata_is_not_replaced_with_defaults(){
        for mode in ["corrupt","shared","hardlink","symlink","large"]{
            let t=Temp::new();t.publish(&[t.file("a")]);match mode{
                "corrupt"=>std::fs::write(t.path(),b"not JSON").unwrap(),"shared"=>std::fs::set_permissions(t.path(),std::fs::Permissions::from_mode(0o644)).unwrap(),
                "hardlink"=>std::fs::hard_link(t.path(),t.root.join("linked")).unwrap(),"symlink"=>{std::fs::remove_file(t.path()).unwrap();symlink(t.root.join("missing"),t.path()).unwrap();},_=>std::fs::write(t.path(),vec![b' ';MAX_BYTES+1]).unwrap(),
            }
            assert!(io::load_at(&t.path()).is_err(),"{mode}");assert!(append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("b")]).is_err(),"{mode}");
        }
    }
    #[test]
    fn owner_copy_identity_duplicates_and_selection_limits_are_checked_atomically(){
        let t=Temp::new();t.publish(&[t.file("a")]);let before=std::fs::read(t.path()).unwrap();
        assert!(append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("a")]).is_err());let mut a=t.file("b");a.file.path="/outside/not-dropped.txt".into();assert!(append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[a]).is_err());
        let many=(0..32).map(|i|t.file(&format!("item-{i}"))).collect::<Vec<_>>();assert!(append_at(&t.meta,&t.inbox,"ollama","conversation-1",&many).is_err());assert_eq!(std::fs::read(t.path()).unwrap(),before);
        assert_ne!(t.path(),path_at(&t.meta,"codex","conversation-1").unwrap());assert!(path_at(&t.meta,"ollama","../other").is_err());
    }
    #[test]
    fn deselection_retains_historical_copy_reference_and_cas_rejects_stale_edits(){
        let t=Temp::new();t.publish(&[t.file("a")]);let old=io::load_at(&t.path()).unwrap();let mut staged=io::prepare(&t.path(),Some(&old.revision),|r|{r.records[0].selected=false;Ok(())}).unwrap();staged.publish().unwrap();staged.confirm().unwrap();
        let next=io::load_at(&t.path()).unwrap();assert!(!next.registry.records[0].selected);assert_eq!(next.registry.records[0].copy_name,"a.txt");assert!(matches!(io::prepare(&t.path(),Some(&old.revision),|_|Ok(())),Err("registryConflict")));
    }
    #[test]
    fn late_external_edit_revokes_staged_publication_and_preserves_the_edit(){
        let t=Temp::new();t.publish(&[t.file("a")]);let mut staged=append_at(&t.meta,&t.inbox,"ollama","conversation-1",&[t.file("b")]).unwrap();std::fs::write(t.path(),b"external edit").unwrap();assert_eq!(staged.publish(),Err("registryConflict"));assert_eq!(std::fs::read(t.path()).unwrap(),b"external edit");
    }
    #[test]
    fn restored_metadata_has_no_source_or_usable_attachment_context(){
        let t=Temp::new();t.publish(&[t.file("a")]);let recovered=io::load_at(&t.path()).unwrap();assert!(recovered.registry.records[0].selected);assert!(Store::default().contexts("ollama","conversation-1").is_empty());
        let mut bad=recovered.registry.clone();bad.records[0].copy_name="../elsewhere".into();assert!(bad.validate().is_err());bad.records[0].copy_name="/absolute".into();assert!(bad.validate().is_err());
    }
}
