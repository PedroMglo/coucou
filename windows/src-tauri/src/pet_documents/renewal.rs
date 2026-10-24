//! Explicit, short-lived native review of exact saved copies. Never a model tool.
use super::*;
use super::{copy_review::{Budget,Copy},manifest::Restoration};
use serde::{Deserialize,Serialize};
const REVIEW_TTL:Duration=Duration::from_secs(120);
const INVALID:&str="A revisão expirou ou a conversa, os anexos ou as permissões mudaram. Volta a rever os itens antes de autorizar.";
type Fence=(u64,u64,u64);
struct Review{provider:String,cid:String,fence:Fence,deadline:Instant,restoration:Restoration,copies:Vec<Copy>}
#[derive(Default)]
struct Reviews(BTreeMap<String,Review>);
impl Reviews{
    fn expire(&mut self,now:Instant){self.0.retain(|_,r|now<r.deadline);}
    fn insert(&mut self,r:Review,now:Instant)->Result<String,String>{self.expire(now);if self.0.len()>=32||self.0.values().map(|r|r.copies.len()).sum::<usize>()+r.copies.len()>128{return Err("Há demasiadas revisões ou anexos pendentes. Fecha uma revisão ou aguarda que expire.".into());}let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|"Não foi possível preparar a revisão dos anexos.".to_string())?;let id=bytes.iter().map(|b|format!("{b:02x}")).collect::<String>();self.0.insert(id.clone(),r);Ok(id)}
    fn take(&mut self,id:&str,provider:&str,cid:&str,now:Instant)->Result<Review,String>{
        self.expire(now);let r=self.0.get(id).ok_or(INVALID)?;
        // A wrong caller cannot consume the real owner's one-use review.
        if r.provider!=provider||r.cid!=cid{return Err(INVALID.into());}
        self.0.remove(id).ok_or_else(||INVALID.into())
    }
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(Mutex::default)}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Preview{review_id:String,items:Vec<Item>,expires_in_seconds:u64}
#[derive(Serialize)]
struct Item{id:String,name:String,kind:String,size:u64,entries:usize}
#[derive(Clone,Copy,Deserialize)]
#[serde(rename_all="camelCase")]
pub(super) enum Consent{ReadSelectedCopies}
fn permission_at(provider:&str,path:&Path)->Result<(),String>{
    let pet=crate::pet_config::integration_pet(provider).ok_or(INVALID)?;crate::pet_config::require_active(pet)?;
    let config=crate::pet_config::read_existing_at(path)?;
    if !config.permissions(pet)?.documents{return Err("Ativa o acesso a documentos deste pet antes de autorizar a leitura dos anexos.".into());}Ok(())
}
fn configured(app:&AppHandle,provider:&str)->Result<(),String>{
    let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();
    if configured_providers(&settings).iter().any(|p|p==provider){Ok(())}else{Err("Este pet não está configurado para receber documentos.".into())}
}
impl Review{
    fn validate(&self,chat:&crate::chat::Chat,budget:&mut Budget,state:&Mutex<Store>,config_path:&Path)->Result<(),String>{
        if Instant::now()>=self.deadline||chat.conversation_id(&self.provider)?!=self.cid||!self.restoration.current()
            ||!state.lock().unwrap().review_current(&self.provider,&self.cid,self.fence)||!self.copies.iter().all(|c|c.current(budget)){return Err(INVALID.into());}
        permission_at(&self.provider,config_path)
    }
}
pub(super) fn prepare(app:&AppHandle,provider:String,cid:String,ids:Vec<String>,expected:String)->Result<Preview,String>{
    if expected.len()!=64||!expected.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase()){return Err(INVALID.into());}
    configured(app,&provider)?;let chat=app.state::<crate::chat::Chat>();
    if chat.conversation_id(&provider)?!=cid{return Err(INVALID.into());}
    permission_at(&provider,&crate::pet_config::path())?;
    let fence=store().lock().unwrap().review_fence(&provider,&cid)?;
    let restoration=manifest::restore(&provider,&cid,&ids,&expected)?;let mut budget=Budget::new();
    let copies=restoration.files.iter().map(|a|Copy::capture(&crate::files::inbox_dir(),a,&mut budget)).collect::<Result<Vec<_>,_>>()?;
    let review=Review{provider,cid,fence,deadline:Instant::now()+REVIEW_TTL,restoration,copies};review.validate(&chat,&mut budget,store(),&crate::pet_config::path())?;
    let items=review.restoration.files.iter().map(|a|Item{id:a.id.clone(),name:a.file.name.clone(),kind:a.file.kind.clone(),size:a.file.size,entries:a.file.entries}).collect();
    let review_id=reviews().lock().unwrap().insert(review,Instant::now())?;
    Ok(Preview{review_id,items,expires_in_seconds:REVIEW_TTL.as_secs()})
}
pub(super) fn renew(app:&AppHandle,provider:String,cid:String,id:String,_consent:Consent)->Result<Vec<Attachment>,String>{
    configured(app,&provider)?;let review=reviews().lock().unwrap().take(&id,&provider,&cid,Instant::now())?;
    let chat=app.state::<crate::chat::Chat>();
    finish(review,&chat,store(),&crate::pet_config::path(),||changed(app,Some(&provider),Some(&cid)),||{})
}
fn finish(review:Review,chat:&crate::chat::Chat,state:&Mutex<Store>,config_path:&Path,published:impl FnOnce(),before_commit:impl FnOnce())->Result<Vec<Attachment>,String>{
    let mut budget=Budget::new();review.validate(chat,&mut budget,state,config_path)?;
    let mut pending=review.restoration.prepare_commit()?;before_commit();
    // Check complete trees off the RAM mutex after staging/fsync, then use
    // root identity + owner epoch for the short final publication fence.
    review.validate(chat,&mut budget,state,config_path)?;
    let paths=review.restoration.files.iter().map(|a|a.file.path.as_str()).collect::<Vec<_>>();
    let grant=crate::pet_files::grant_reviewed_reads_at(&review.provider,&paths,config_path)?;
    if chat.conversation_id(&review.provider)?!=review.cid||Instant::now()>=review.deadline{return Err(INVALID.into());}
    let selected={
        let mut state=state.lock().unwrap();
        if !state.review_current(&review.provider,&review.cid,review.fence)||!review.copies.iter().all(Copy::root_current){return Err(INVALID.into());}
        let (sequence,selected)=state.stage_restored(&review.provider,&review.cid,&review.restoration.files)?;
        pending.publish()?;state.publish_restored(&review.provider,&review.cid,sequence,&selected);grant.keep();selected
    };
    published();pending.confirm()?;Ok(selected)
}

pub(super) fn cancel(provider:&str,cid:&str,id:&str){let mut reviews=reviews().lock().unwrap();if reviews.0.get(id).is_some_and(|r|r.provider==provider&&r.cid==cid){reviews.0.remove(id);}}

#[cfg(test)]
mod tests{
    use super::*;
    use std::os::unix::fs::{DirBuilderExt,PermissionsExt};
    struct Temp{root:std::path::PathBuf,config:std::path::PathBuf,copy:std::path::PathBuf,chat:crate::chat::Chat,state:Mutex<Store>,cid:String}
    impl Temp{
        fn new()->Self{
            let root=std::env::temp_dir().join(format!("coucou-renew-fixture-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();std::fs::DirBuilder::new().mode(0o700).create(root.join("inbox")).unwrap();
            let config=root.join("pets.json");let copy=root.join("inbox/copy.txt");std::fs::write(&copy,b"one").unwrap();std::fs::set_permissions(&copy,std::fs::Permissions::from_mode(0o600)).unwrap();let chat=crate::chat::Chat::default();let cid=chat.conversation_id("ollama").unwrap();let t=Self{root,config,copy,chat,state:Mutex::default(),cid};t.permissions(true);t
        }
        fn permissions(&self,documents:bool){let mut c=crate::pet_config::defaults();c.pets.get_mut("ai_ollama").unwrap().permissions.get_or_insert_with(Default::default).documents=documents;std::fs::write(&self.config,serde_json::to_vec(&c).unwrap()).unwrap();}
        fn review(&self)->Review{
            let file=Attachment{id:"saved-1".into(),provider:"ollama".into(),conversation_id:self.cid.clone(),sequence:0,file:crate::files::DroppedFile{name:"source.txt".into(),path:self.copy.to_string_lossy().into_owned(),size:3,kind:"file".into(),entries:1}};
            let restoration=manifest::test_restoration(&self.root.join(crate::assistant_memory::new_id()),&self.root.join("inbox"),&[file]);let mut budget=Budget::new();let copies=restoration.files.iter().map(|a|Copy::capture(&self.root.join("inbox"),a,&mut budget).unwrap()).collect();
            Review{provider:"ollama".into(),cid:self.cid.clone(),fence:self.state.lock().unwrap().review_fence("ollama",&self.cid).unwrap(),deadline:Instant::now()+REVIEW_TTL,restoration,copies}
        }
    }
    impl Drop for Temp{fn drop(&mut self){crate::pet_files::revoke_reviewed_reads(Some("ollama"),Some(&[self.copy.to_string_lossy().into_owned()]));let _=std::fs::remove_dir_all(&self.root);}}
    #[test]fn reviewed_copies_reach_native_collection_only_after_exact_metadata_commit(){
        let t=Temp::new();let review=t.review();let original=review.restoration.files[0].id.clone();let published=std::cell::Cell::new(false);
        assert!(t.state.lock().unwrap().contexts("ollama",&t.cid).is_empty());let selected=finish(review,&t.chat,&t.state,&t.config,||published.set(true),||{}).unwrap();assert!(published.get());assert_eq!(selected[0].id,original);assert!(selected[0].sequence>0);
        let state=t.state.lock().unwrap();assert_eq!(state.contexts("ollama",&t.cid)[0].id,original);assert!(state.references("ollama",&t.cid,&[selected[0].context_file()]).is_ok());assert!(state.references("codex",&t.cid,&[selected[0].context_file()]).is_err());
    }
    #[test]fn removal_during_staging_or_permission_revocation_never_restores_authority(){
        for reason in ["remove","permission","reset","file"]{
            let t=Temp::new();let review=t.review();let published=std::cell::Cell::new(false);
            let result=finish(review,&t.chat,&t.state,&t.config,||published.set(true),||match reason{
                "remove"=>{t.state.lock().unwrap().remove("ollama",&t.cid,"saved-1");},"permission"=>t.permissions(false),"reset"=>t.state.lock().unwrap().invalidate(Some("ollama")),_=>std::fs::write(&t.copy,b"two").unwrap(),
            });assert!(result.is_err(),"{reason}");assert!(!published.get(),"{reason}");assert!(t.state.lock().unwrap().contexts("ollama",&t.cid).is_empty());
        }
    }
    #[test]fn another_pet_or_conversation_does_not_cancel_the_reviewed_owner(){
        let t=Temp::new();let review=t.review();let result=finish(review,&t.chat,&t.state,&t.config,||{},||{let mut s=t.state.lock().unwrap();s.remove("codex",&t.cid,"saved-1");s.remove("ollama","another-cid","saved-1");});assert!(result.is_ok());
    }
    #[test]fn expired_review_changed_conversation_and_missing_copy_cannot_publish(){
        for reason in ["expired","conversation","missing"]{
            let t=Temp::new();let mut review=t.review();match reason{"expired"=>review.deadline=Instant::now()-Duration::from_millis(1),"conversation"=>{t.chat.reset_provider("ollama").unwrap();},_=>std::fs::remove_file(&t.copy).unwrap()}
            let published=std::cell::Cell::new(false);assert!(finish(review,&t.chat,&t.state,&t.config,||published.set(true),||{}).is_err());assert!(!published.get());assert!(t.state.lock().unwrap().contexts("ollama",&t.cid).is_empty());
        }
    }
    #[test]fn review_tokens_are_bounded_one_use_and_cannot_be_consumed_by_wrong_owner(){
        let t=Temp::new();let review=t.review();let cid=t.cid.clone();let mut cache=Reviews::default();let id=cache.insert(review,Instant::now()).unwrap();assert_eq!(id.len(),64);assert!(cache.take(&id,"codex",&cid,Instant::now()).is_err());assert!(cache.take(&id,"ollama","other-cid",Instant::now()).is_err());assert!(cache.take(&id,"ollama",&cid,Instant::now()).is_ok());assert!(cache.take(&id,"ollama",&cid,Instant::now()).is_err());
        let review=t.review();let id=cache.insert(review,Instant::now()).unwrap();assert!(cache.take(&id,"ollama",&cid,Instant::now()+REVIEW_TTL).is_err());assert!(cache.0.is_empty());
        for _ in 0..32{cache.insert(t.review(),Instant::now()).unwrap();}assert!(cache.insert(t.review(),Instant::now()).is_err());
    }
    #[test]fn renewed_ids_receive_a_fresh_incarnation_without_retargeting_old_requests(){
        let t=Temp::new();let review=t.review();let first=finish(review,&t.chat,&t.state,&t.config,||{},||{}).unwrap();let old=first[0].context_file();t.state.lock().unwrap().remove("ollama",&t.cid,&old.id);
        let review=t.review();let next=finish(review,&t.chat,&t.state,&t.config,||{},||{}).unwrap();assert_eq!(next[0].id,old.id);assert!(next[0].sequence>old.sequence);assert!(t.state.lock().unwrap().references("ollama",&t.cid,&[old]).is_err());
    }
    #[test]fn consent_is_explicit_and_never_accepts_broad_or_missing_access(){
        assert!(serde_json::from_str::<Consent>("\"readSelectedCopies\"").is_ok());
        for value in ["null","\"always\"","\"fullAccess\"","{}"]{assert!(serde_json::from_str::<Consent>(value).is_err());}
    }
    #[test]fn permission_review_does_not_create_defaults_or_allow_retired_pet(){
        let directory=std::env::temp_dir().join(format!("coucou-renew-permission-{}",crate::assistant_memory::new_id()));let path=directory.join("pets.json");
        assert!(permission_at("ollama",&path).is_err());assert!(!directory.exists());
        let mut config=crate::pet_config::defaults();config.pets.get_mut("ai_ollama").unwrap().permissions=Some(crate::pet_config::Permissions::default());std::fs::create_dir(&directory).unwrap();std::fs::write(&path,serde_json::to_vec(&config).unwrap()).unwrap();
        assert!(permission_at("ollama",&path).is_err());assert!(permission_at("integration_github",&path).is_err());
        config.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().documents=true;std::fs::write(&path,serde_json::to_vec(&config).unwrap()).unwrap();assert!(permission_at("ollama",&path).is_ok());std::fs::remove_dir_all(directory).unwrap();
    }
}
