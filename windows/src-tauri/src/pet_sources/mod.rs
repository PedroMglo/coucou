//! Explicitly enrolled persistent personal sources, separate from OS drops.
mod model;
pub(crate)use model::PetScope;
#[cfg(target_os="linux")]
mod store;

use serde::{Deserialize,Serialize};
use std::path::PathBuf;
use tauri::Emitter;

#[derive(Clone,Copy,Debug,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub(crate) enum Consent {MetadataOnly,PersistentCalendarRead}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Document {config:model::Registry,revision:String}
fn path()->PathBuf{crate::pet_config::path().with_file_name("sources.json")}
fn allowed(label:&str)->bool{[crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_control::LABEL,crate::pet_chat::LABEL].contains(&label)}
fn message(code:&str)->String{
    match code{
        "registryConflict"|"sourceRevisionConflict"|"registryChanged"=>"As fontes mudaram. Volta a carregar a configuração antes de guardar ou consultar.",
        "consentRequired"=>"Ativar ou alargar uma fonte exige consentimento explícito de leitura persistente. Entregar um documento ao pet não concede esse acesso.",
        "registryBusy"=>"Outro pedido está a guardar as fontes. Aguarda e volta a carregar a configuração.",
        "registryCommitUncertain"=>"Não foi possível confirmar a gravação. Volta a carregar as fontes antes de decidir se é necessário guardar novamente.",
        "permissionDenied"|"ownerChanged"=>"Esta fonte não está autorizada para este pet e conversa.",
        "conversationRetentionUnavailable"=>"Não foi possível confirmar a gravação da conversa original. As fontes não foram publicadas; volta a consultar antes de repetir.",
        _=>"A configuração de fontes está indisponível ou é inválida. Os dados anteriores foram preservados.",
    }.into()
}
fn consent(old:&model::Registry,next:&model::Registry,decision:Consent)->Result<(),&'static str>{
    if decision==Consent::PersistentCalendarRead{return Ok(());}
    for source in next.sources.iter().filter(|s|s.enabled){
        let previous=old.sources.iter().find(|p|p.id==source.id&&p.enabled&&p.kind==source.kind&&p.path==source.path);
        if source.owners.iter().any(|owner|previous.is_none_or(|p|!p.owners.contains(owner))){return Err("consentRequired");}
    }
    Ok(())
}

/// Under the source writer's original CAS/consent review. Persist the exact
/// existing owner before publishing an enrollment into a still-empty chat.
/// An uncertain/failed library write cannot authorize source publication.
#[cfg(target_os="linux")]
fn save_with_conversations(path:&std::path::Path,expected:&str,config:model::Registry,decision:Consent,chat:&crate::chat::Chat)->Result<store::Snapshot,&'static str>{
    store::save_checked(path,expected,config,|old,next|{
        consent(old,next,decision)?;
        let mut retained=std::collections::BTreeSet::new();
        for source in next.sources.iter().filter(|s|s.enabled){
            let previous=old.sources.iter().find(|p|p.id==source.id&&p.enabled&&p.kind==source.kind&&p.path==source.path);
            for owner in source.owners.iter().filter(|o|previous.is_none_or(|p|!p.owners.contains(o))){
                if !retained.insert(owner){continue;}
                if crate::pet_config::retired(owner.pet_id()){return Err("ownerChanged");}
                let blueprint=crate::pet_config::blueprint(owner.pet_id()).ok_or("ownerChanged")?;
                if let Some(cid)=owner.conversation_id(){
                    let provider=blueprint.routing.chat.as_deref().or(blueprint.routing.task_provider.as_deref()).ok_or("ownerChanged")?;
                    chat.retain_public_conversation(provider,cid).map_err(|_|"conversationRetentionUnavailable")?;
                }
            }
        }Ok(())
    })
}

/// Called under Chat's mutex. No Chat/configuration lock or source-content IO.
/// Disabled/legacy memberships still retain their owner's original dialogue;
/// an unreadable book conservatively preserves rather than discards history.
#[cfg(target_os="linux")]
fn retains_conversation_at(path:&std::path::Path,provider:&str,cid:&str)->bool{
    let Some(pet)=crate::pet_config::integration_pet(provider)else{return false;};
    match store::load_at(path){
        Ok(snapshot)=>!snapshot.unchanged()||snapshot.registry.sources.iter().any(|s|s.owners.contains(&model::Owner::conversation(pet,cid))),
        Err(_)=>true,
    }
}
#[cfg(target_os="linux")]
pub(crate)fn retains_conversation(provider:&str,cid:&str)->bool{retains_conversation_at(&path(),provider,cid)}

/// Management IPC is never offered as a model tool. Source paths are explicit
/// user configuration; a transient document drop does not call this writer.
#[tauri::command]
pub(crate) async fn pet_source_configuration(window:tauri::WebviewWindow)->Result<Document,String>{
    if !allowed(window.label()){return Err("Abre a gestão de fontes no pet.".into());}
    #[cfg(target_os="linux")]
    {tauri::async_runtime::spawn_blocking(||store::load_at(&path()).map(|s|Document{config:s.registry,revision:s.revision}).map_err(message)).await.map_err(|_|"A leitura das fontes foi interrompida.".to_string())?}
    #[cfg(not(target_os="linux"))]
    {Err("As fontes persistentes ainda não estão disponíveis nesta plataforma.".into())}
}
#[tauri::command]
pub(crate) async fn pet_source_configuration_save(app:tauri::AppHandle,window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,config:model::Registry,expected_revision:String,consent:Consent)->Result<Document,String>{
    if !allowed(window.label()){return Err("Abre a gestão de fontes no pet.".into());}
    if expected_revision!="missing"&&(expected_revision.len()!=64||!expected_revision.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())){return Err(message("registryConflict"));}
    #[cfg(target_os="linux")]
    {
        let chat=chat.inner().clone();
        let result=tauri::async_runtime::spawn_blocking(move||save_with_conversations(&path(),&expected_revision,config,consent,&chat).map(|s|Document{config:s.registry,revision:s.revision}).map_err(message)).await.map_err(|_|"A gravação das fontes foi interrompida. Volta a carregar a configuração.".to_string())??;
        let _=app.emit("pet-sources-changed",serde_json::json!({"revision":result.revision}));Ok(result)
    }
    #[cfg(not(target_os="linux"))]
    {let _=(app,chat,config,expected_revision,consent);Err("As fontes persistentes ainda não estão disponíveis nesta plataforma.".into())}
}

#[cfg(all(test,target_os="linux"))]
#[path="conversation_tests.rs"]mod calendar_conversation_preservation;

#[cfg(target_os="linux")]
#[derive(Clone)]
pub(crate) struct Selected {source:model::Source,snapshot:std::sync::Arc<store::Snapshot>,owner:model::Owner,config_path:PathBuf}
#[cfg(target_os="linux")]
impl Selected{
    pub(crate)fn id(&self)->&str{&self.source.id}
    pub(crate)fn revision(&self)->u64{self.source.revision}
    pub(crate) fn path(&self)->&str{&self.source.path}
    pub(crate) fn name(&self)->&str{&self.source.name}
    pub(crate) fn binding(&self)->String{
        let scope=self.owner.conversation_id().map(|cid|serde_json::json!(cid)).unwrap_or_else(||serde_json::json!({"scope":"pet"}));
        let tuple=serde_json::json!([self.snapshot.revision,self.source.id,self.source.revision,self.owner.pet_id(),scope]);
        ring::digest::digest(&ring::digest::SHA256,tuple.to_string().as_bytes()).as_ref().iter().map(|b|format!("{b:02x}")).collect()
    }
    fn identity_with(&self,provider:&str,cid:&str,config:&crate::pet_config::Configuration)->Result<(),&'static str>{
        let pet=crate::pet_config::integration_pet(provider).ok_or("ownerChanged")?;
        self.identity_for(&model::Owner::conversation(pet,cid),config)
    }
    fn identity_for(&self,owner:&model::Owner,config:&crate::pet_config::Configuration)->Result<(),&'static str>{
        let pet=owner.pet_id();if &self.owner!=owner{return Err("ownerChanged");}
        if crate::pet_config::retired(pet)||!config.permissions(pet).is_ok_and(|p|p.documents){return Err("permissionDenied");}
        Ok(())
    }
    fn validate_with(&self,provider:&str,cid:&str,config:&crate::pet_config::Configuration)->Result<(),&'static str>{
        self.identity_with(provider,cid,config)?;if !self.snapshot.unchanged(){return Err("registryChanged");}Ok(())
    }
    pub(crate) fn validate(&self,provider:&str,cid:&str)->Result<(),String>{
        let config=crate::pet_config::read_existing_at(&self.config_path).map_err(|_|message("permissionDenied"))?;
        self.validate_with(provider,cid,&config).map_err(message)
    }
}
#[cfg(target_os="linux")]
fn select_from(snapshot:store::Snapshot,config_path:PathBuf,provider:&str,cid:&str)->Result<Option<Selected>,&'static str>{
    let Some(pet)=crate::pet_config::integration_pet(provider)else{return Ok(None)};
    let Some(source)=snapshot.registry.selected(pet,cid)?.cloned()else{return Ok(None)};
    Ok(Some(Selected{source,snapshot:std::sync::Arc::new(snapshot),owner:model::Owner::conversation(pet,cid),config_path}))
}
#[cfg(target_os="linux")]
pub(crate) fn selected(provider:&str,cid:&str)->Result<Option<Selected>,String>{
    let snapshot=store::load_at(&path()).map_err(message)?;
    let result=select_from(snapshot,crate::pet_config::path(),provider,cid).map_err(message)?;
    if let Some(value)=&result{value.validate(provider,cid)?;}Ok(result)
}

#[cfg(target_os="linux")]
fn select_many_from(snapshot:store::Snapshot,config_path:PathBuf,provider:&str,cid:&str)->Result<Vec<Selected>,&'static str>{
    let Some(pet)=crate::pet_config::integration_pet(provider)else{return Ok(vec![])};
    let sources=snapshot.registry.selected_many(pet,cid)?.into_iter().cloned().collect::<Vec<_>>();
    let snapshot=std::sync::Arc::new(snapshot);
    Ok(sources.into_iter().map(|source|Selected{source,snapshot:snapshot.clone(),owner:model::Owner::conversation(pet,cid),config_path:config_path.clone()}).collect())
}
#[cfg(target_os="linux")]
pub(crate)fn selected_many(provider:&str,cid:&str)->Result<Vec<Selected>,String>{
    let selections=select_many_from(store::load_at(&path()).map_err(message)?,crate::pet_config::path(),provider,cid).map_err(message)?;
    // Validate the complete exact selection before opening any source. A denied
    // member cannot silently become a smaller/first-source collection.
    validate_group(&selections.iter().collect::<Vec<_>>(),provider,cid)?;Ok(selections)
}

/// One configuration read and one pinned-registry check per distinct captured
/// collection on each watch tick. No source files are opened by this check.
#[cfg(target_os="linux")]
pub(crate)fn validate_group(selections:&[&Selected],provider:&str,cid:&str)->Result<(),String>{
    let pet=crate::pet_config::integration_pet(provider).ok_or_else(||message("ownerChanged"))?;
    validate_group_owner(selections,&model::Owner::conversation(pet,cid))
}
#[cfg(target_os="linux")]
fn select_pet_from(snapshot:store::Snapshot,config_path:PathBuf,provider:&str)->Result<Vec<Selected>,&'static str>{
    let pet=crate::pet_config::integration_pet(provider).ok_or("ownerChanged")?;
    let sources=snapshot.registry.selected_pet(pet)?.into_iter().cloned().collect::<Vec<_>>();let snapshot=std::sync::Arc::new(snapshot);
    Ok(sources.into_iter().map(|source|Selected{source,snapshot:snapshot.clone(),owner:model::Owner::pet(pet),config_path:config_path.clone()}).collect())
}
#[cfg(target_os="linux")]
pub(crate)fn selected_pet(provider:&str)->Result<Vec<Selected>,String>{
    let selections=select_pet_from(store::load_at(&path()).map_err(message)?,crate::pet_config::path(),provider).map_err(message)?;
    validate_pet_group(&selections.iter().collect::<Vec<_>>(),provider)?;Ok(selections)
}
#[cfg(target_os="linux")]
pub(crate)fn validate_pet_group(selections:&[&Selected],provider:&str)->Result<(),String>{
    let pet=crate::pet_config::integration_pet(provider).ok_or_else(||message("ownerChanged"))?;
    validate_group_owner(selections,&model::Owner::pet(pet))
}
#[cfg(target_os="linux")]
fn validate_group_owner(selections:&[&Selected],owner:&model::Owner)->Result<(),String>{
    let mut configurations=std::collections::BTreeMap::new();let mut snapshots=std::collections::BTreeSet::new();
    for selected in selections{
        if !configurations.contains_key(&selected.config_path){let config=crate::pet_config::read_existing_at(&selected.config_path).map_err(|_|message("permissionDenied"))?;configurations.insert(selected.config_path.clone(),config);}
        selected.identity_for(owner,&configurations[&selected.config_path]).map_err(message)?;
        if snapshots.insert(std::sync::Arc::as_ptr(&selected.snapshot)as usize)&&!selected.snapshot.unchanged(){return Err(message("registryChanged"));}
    }Ok(())
}

#[cfg(all(test,target_os="linux"))]
pub(crate) mod test_support{
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    pub(crate) struct Enrollment{pub root:PathBuf,pub source:PathBuf,provider:String,cid:String}
    impl Enrollment{
        pub(crate) fn new(provider:&str,cid:&str)->Self{
            let root=std::env::temp_dir().join(format!("coucou-enrollment-{}",crate::assistant_memory::new_id()));
            std::fs::create_dir(&root).unwrap();std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();
            let source=root.join("synthetic.ics");
            std::fs::write(&source,"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Coucou Synthetic//EN\r\nBEGIN:VEVENT\r\nUID:enrolled-fixture\r\nDTSTAMP:20261022T000000Z\r\nDTSTART:20261023T090000Z\r\nDTEND:20261023T100000Z\r\nSUMMARY:Enrolled fixture 73\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n").unwrap();
            let mut registry=model::tests::fixture();registry.generation=0;registry.sources[0].revision=0;registry.sources[0].path=source.to_str().unwrap().into();
            registry.sources[0].owners=vec![model::Owner::conversation(crate::pet_config::integration_pet(provider).unwrap(),cid)];
            store::save_checked(&root.join("sources.json"),"missing",registry,|old,next|consent(old,next,Consent::PersistentCalendarRead)).unwrap();
            let value=Self{root,source,provider:provider.into(),cid:cid.into()};value.documents(true);value
        }
        pub(crate) fn select(&self,provider:&str,cid:&str)->Option<Selected>{
            select_from(store::load_at(&self.root.join("sources.json")).unwrap(),self.root.join("pets.json"),provider,cid).unwrap()
        }
        pub(crate) fn selected(&self)->Selected{self.select(&self.provider,&self.cid).unwrap()}
        pub(crate)fn collection(&self)->Vec<Selected>{
            let file=self.root.join("second.ics");let text=std::fs::read_to_string(&self.source).unwrap().replace("Enrolled fixture 73","Other fixture 84");std::fs::write(&file,text).unwrap();
            let path=self.root.join("sources.json");let old=store::load_at(&path).unwrap();let mut next=old.registry.clone();next.version=2;let mut source=next.sources[0].clone();source.id="second-calendar".into();source.revision=0;source.path=file.to_str().unwrap().into();next.sources.push(source);store::save_checked(&path,&old.revision,next,|old,next|consent(old,next,Consent::PersistentCalendarRead)).unwrap();
            let selections=select_many_from(store::load_at(&path).unwrap(),self.root.join("pets.json"),&self.provider,&self.cid).unwrap();validate_group(&selections.iter().collect::<Vec<_>>(),&self.provider,&self.cid).unwrap();selections
        }
        pub(crate)fn pet_collection(&self)->Vec<Selected>{
            self.collection();let path=self.root.join("sources.json");let old=store::load_at(&path).unwrap();let mut next=old.registry.clone();next.version=3;
            for source in &mut next.sources{source.owners=vec![model::Owner::pet(crate::pet_config::integration_pet(&self.provider).unwrap())];}
            let before=std::fs::read(&path).unwrap();assert_eq!(store::save_checked(&path,&old.revision,next.clone(),|a,b|consent(a,b,Consent::MetadataOnly)).err(),Some("consentRequired"));assert_eq!(std::fs::read(&path).unwrap(),before);
            store::save_checked(&path,&old.revision,next,|a,b|consent(a,b,Consent::PersistentCalendarRead)).unwrap();
            let selections=select_pet_from(store::load_at(&path).unwrap(),self.root.join("pets.json"),&self.provider).unwrap();validate_pet_group(&selections.iter().collect::<Vec<_>>(),&self.provider).unwrap();assert!(self.select(&self.provider,&self.cid).is_none());selections
        }

        pub(crate) fn documents(&self,enabled:bool){
            let mut pets=crate::pet_config::defaults();let pet=crate::pet_config::integration_pet(&self.provider).unwrap();
            pets.pets.get_mut(pet).unwrap().permissions.as_mut().unwrap().documents=enabled;
            std::fs::write(self.root.join("pets.json"),serde_json::to_vec(&pets).unwrap()).unwrap();
        }
        pub(crate) fn disable(&self){
            let path=self.root.join("sources.json");let old=store::load_at(&path).unwrap();let mut next=old.registry.clone();next.sources[0].enabled=false;
            store::save_checked(&path,&old.revision,next,|old,next|consent(old,next,Consent::MetadataOnly)).unwrap();
        }
    }
    impl Drop for Enrollment{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn persistent_consent_is_distinct_from_metadata_edits_or_a_drop(){
        let old=model::Registry::empty();let next=model::tests::fixture();
        assert_eq!(consent(&old,&next,Consent::MetadataOnly),Err("consentRequired"));assert!(consent(&old,&next,Consent::PersistentCalendarRead).is_ok());
        let mut changed=next.clone();changed.sources[0].name="Renomeada".into();assert!(consent(&next,&changed,Consent::MetadataOnly).is_ok());
        changed.sources[0].path="/different/calendar.ics".into();assert_eq!(consent(&next,&changed,Consent::MetadataOnly),Err("consentRequired"));
        changed.sources[0].enabled=false;assert!(consent(&next,&changed,Consent::MetadataOnly).is_ok());
        let mut changed=next.clone();changed.sources[0].owners[0]=model::Owner::conversation("agent_codex","other-cid");assert_eq!(consent(&next,&changed,Consent::MetadataOnly),Err("consentRequired"));
        assert!(serde_json::from_str::<Consent>("true").is_err());assert!(serde_json::from_str::<Consent>("\"always\"").is_err());
    }
    #[test]
    fn only_known_native_management_windows_can_manage_source_metadata(){
        for label in [crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_control::LABEL,crate::pet_chat::LABEL]{assert!(allowed(label));}
        for label in ["","external","terminal","source-editor","island-spoof"]{assert!(!allowed(label));}
    }
    #[cfg(target_os="linux")]
    #[test]
    fn writer_checks_consent_under_the_same_lock_and_preserves_denied_bytes(){
        use std::os::{fd::AsRawFd,unix::fs::PermissionsExt};
        let dir=std::env::temp_dir().join(format!("coucou-consent-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&dir).unwrap();std::fs::set_permissions(&dir,std::fs::Permissions::from_mode(0o700)).unwrap();
        let path=dir.join("sources.json");let mut config=model::tests::fixture();config.generation=0;config.sources[0].revision=0;
        assert!(matches!(store::save_checked(&path,"missing",config.clone(),|old,next|consent(old,next,Consent::MetadataOnly)),Err("consentRequired")));assert!(!path.exists());
        let saved=store::save_checked(&path,"missing",config,|old,next|{
            let lock=std::fs::OpenOptions::new().read(true).write(true).open(dir.join(".sources.lock")).unwrap();
            assert_ne!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)},0);
            consent(old,next,Consent::PersistentCalendarRead)
        }).unwrap();let before=std::fs::read(&path).unwrap();
        let mut next=saved.registry.clone();next.sources[0].owners[0]=model::Owner::conversation("agent_codex","new-cid");
        assert!(matches!(store::save_checked(&path,&saved.revision,next,|old,next|consent(old,next,Consent::MetadataOnly)),Err("consentRequired")));assert_eq!(std::fs::read(&path).unwrap(),before);
        assert!(matches!(store::save_checked(&path,"missing",saved.registry,|_,_|panic!("stale writer must not review another revision")),Err("registryConflict")));
        assert!(!std::fs::read_dir(&dir).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));std::fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(target_os="linux")]
    #[test]
    fn every_supported_owner_is_isolated_and_github_remains_retired(){
        for provider in ["codex","antigravity","copilot","gemini-cli","claude-cli","ollama","lmstudio","openai","anthropic","google","integration_resend","integration_n8n","integration_vercel","integration_notion","integration_calcom","integration_stripe","integration_spotify"]{
            let fixture=test_support::Enrollment::new(provider,"fixture-cid");let selected=fixture.selected();assert!(selected.validate(provider,"fixture-cid").is_ok(),"{provider}");
            assert!(selected.validate(provider,"other-cid").is_err());assert!(fixture.select(provider,"other-cid").is_none());assert!(fixture.select("unknown","fixture-cid").is_none());
            fixture.documents(false);assert!(selected.validate(provider,"fixture-cid").is_err());fixture.documents(true);assert!(selected.validate(provider,"fixture-cid").is_ok());
            std::fs::remove_file(fixture.root.join("pets.json")).unwrap();assert!(selected.validate(provider,"fixture-cid").is_err());assert!(!fixture.root.join("pets.json").exists());
        }
        let fixture=test_support::Enrollment::new("integration_github","fixture-cid");assert!(fixture.selected().validate("integration_github","fixture-cid").is_err());
    }
    #[cfg(target_os="linux")]
    #[test]
    fn selected_source_cannot_borrow_another_owner_or_a_revoked_permission(){
        use std::os::unix::fs::PermissionsExt;
        let dir=std::env::temp_dir().join(format!("coucou-source-owner-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&dir).unwrap();std::fs::set_permissions(&dir,std::fs::Permissions::from_mode(0o700)).unwrap();let path=dir.join("sources.json");
        let mut cfg=model::tests::fixture();cfg.generation=0;cfg.sources[0].revision=0;let snapshot=store::save_at(&path,"missing",cfg).unwrap();
        let selected=select_from(snapshot.clone(),dir.join("pets.json"),"codex","cid-one").unwrap().unwrap();let mut pets=crate::pet_config::defaults();pets.pets.get_mut("agent_codex").unwrap().permissions.as_mut().unwrap().documents=true;
        assert!(selected.validate_with("codex","cid-one",&pets).is_ok());assert_eq!(selected.validate_with("copilot","cid-one",&pets),Err("ownerChanged"));assert_eq!(selected.validate_with("codex","other-cid",&pets),Err("ownerChanged"));
        assert!(select_from(snapshot.clone(),dir.join("pets.json"),"copilot","cid-one").unwrap().is_none());assert!(select_from(snapshot.clone(),dir.join("pets.json"),"codex","other-cid").unwrap().is_none());
        pets.pets.get_mut("agent_codex").unwrap().permissions.as_mut().unwrap().documents=false;assert_eq!(selected.validate_with("codex","cid-one",&pets),Err("permissionDenied"));
        pets.pets.get_mut("agent_codex").unwrap().permissions.as_mut().unwrap().documents=true;let binding=selected.binding();assert_eq!(binding.len(),64);assert!(!binding.contains("calendar"));
        let mut revised=snapshot.registry.clone();revised.sources[0].enabled=false;store::save_at(&path,&snapshot.revision,revised).unwrap();assert_eq!(selected.validate_with("codex","cid-one",&pets),Err("registryChanged"));std::fs::remove_dir_all(dir).unwrap();
    }
}

/// Native restore initialization only. No source path is read/materialized.
/// Fresh source enrollment still uses save_checked and explicit native consent.
pub(crate)fn suspend_for_restoration(bytes:&[u8])->Result<Vec<u8>,&'static str>{
 if bytes.len()>model::MAX_BYTES{return Err("registryLimit");}let old:model::Registry=serde_json::from_slice(bytes).map_err(|_|"invalidRegistry")?;old.validate()?;let mut next=old.clone();for source in &mut next.sources{source.enabled=false;}
 if next==old{return Ok(bytes.to_vec());}let next=next.next(&old)?;serde_json::to_vec(&next).map_err(|_|"invalidRegistry")
}


#[cfg(all(test,target_os="linux"))]mod persistent_collection_tests{
 use super::*;
 #[test]fn persistent_collection_members_need_own_consent_and_source_revisions_survive_reopen(){
  let f=test_support::Enrollment::new("ollama","collection-cid");let path=f.root.join("sources.json");let old=store::load_at(&path).unwrap();let mut next=old.registry.clone();next.version=2;let mut source=next.sources[0].clone();source.id="second".into();source.revision=0;next.sources.push(source);let before=std::fs::read(&path).unwrap();assert_eq!(store::save_checked(&path,&old.revision,next,|a,b|consent(a,b,Consent::MetadataOnly)).err(),Some("consentRequired"));assert_eq!(std::fs::read(&path).unwrap(),before);
  let selected=f.collection();assert_eq!(selected.len(),2);assert!(std::sync::Arc::ptr_eq(&selected[0].snapshot,&selected[1].snapshot));assert!(select_from(store::load_at(&path).unwrap(),f.root.join("pets.json"),"ollama","collection-cid").is_err());assert!(validate_group(&selected.iter().collect::<Vec<_>>(),"copilot","collection-cid").is_err());f.disable();assert!(validate_group(&selected.iter().collect::<Vec<_>>(),"ollama","collection-cid").is_err());assert_eq!(store::load_at(&path).unwrap().registry.selected_many("ai_ollama","collection-cid").unwrap().len(),1);
 }
 #[test]fn persistent_collection_restore_suspends_every_source_without_downgrading_or_reading_paths(){
  let f=test_support::Enrollment::new("codex","collection-cid");f.collection();let path=f.root.join("sources.json");let original=std::fs::read(&path).unwrap();let restored=suspend_for_restoration(&original).unwrap();let book:model::Registry=serde_json::from_slice(&restored).unwrap();assert_eq!(book.version,2);assert!(book.sources.iter().all(|s|!s.enabled));assert!(book.selected_many("agent_codex","collection-cid").unwrap().is_empty());assert_eq!(std::fs::read(path).unwrap(),original);
 }
}


#[cfg(all(test,target_os="linux"))]
#[test]#[ignore="Explicit empty generated profile output for native source-format CLI QA; no user files"]
fn native_persistent_collection_profile_export(){
 use std::{fs,os::unix::fs::{DirBuilderExt,PermissionsExt},path::PathBuf};
 let output=PathBuf::from(std::env::var("COUCOU_CALENDAR_COLLECTION_EXPORT").expect("Explicit private output"));assert!(output.is_absolute()&&output.is_dir()&&fs::read_dir(&output).unwrap().next().is_none());let mut profiles=vec![];
 for version in [1,2]{
  let profile=output.join(format!("sources-{version}"));let config=profile.join("config/coucou");let data=profile.join("data/coucou");fs::DirBuilder::new().recursive(true).mode(0o700).create(&config).unwrap();fs::DirBuilder::new().recursive(true).mode(0o700).create(&data).unwrap();
  let mut settings=crate::settings::Settings::default();settings.autostart=false;settings.hooks_installed=false;settings.sound_enabled=false;settings.pet_system_center=false;settings.phone_link=false;settings.phone_relay=false;settings.main_pill="ai_ollama".into();settings.active_integrations=vec!["ai_ollama".into()];settings.shortcuts.clear();settings.chat_models.insert("ollama".into(),"fixture-free".into());
  for(name,bytes)in[("settings.json",serde_json::to_vec(&settings).unwrap()),("pets.json",br#"{"version":1,"pets":{}}"#.to_vec())]{let f=config.join(name);fs::write(&f,bytes).unwrap();fs::set_permissions(f,fs::Permissions::from_mode(0o600)).unwrap();}
  let bus=crate::spotify::goal::tests::Bus::new();let connection=dbus::blocking::Connection::new_address(&bus.address).unwrap();let prefix="com.coucou.CalendarCollectionFixture";connection.request_name(&format!("{prefix}.SingleInstance"),false,false,true).unwrap();let(config,data)=crate::pet_data::fixture_publish_profile(&bus.address,prefix,crate::pet_data::Layout::new(config,data).unwrap()).unwrap();
  // Publish complete operational preferences, not the intentionally minimal legacy bootstrap seed.
  fs::write(config.join("pets.json"),serde_json::to_vec(&crate::pet_config::defaults()).unwrap()).unwrap();
  let mut registry=model::tests::fixture();registry.version=version;registry.generation=0;registry.sources[0].revision=0;registry.sources[0].owners=vec![model::Owner::conversation("ai_ollama","native-collection-fixture")];
  if version==2{let mut second=registry.sources[0].clone();second.id="second-calendar".into();second.path="/not-opened-by-maintenance/second.ics".into();registry.sources.push(second);}
  let snapshot=store::save_checked(&config.join("sources.json"),"missing",registry,|old,next|consent(old,next,Consent::PersistentCalendarRead)).unwrap();assert!(snapshot.registry.sources.iter().all(|s|s.enabled));profiles.push(serde_json::json!({"profile":profile,"configPayload":config,"dataPayload":data,"sourceVersion":version,"sourceCount":snapshot.registry.sources.len(),"sourceRevision":snapshot.revision}));
 }
 fs::write(output.join("EXPORT.json"),serde_json::to_vec_pretty(&serde_json::json!({"schemaVersion":1,"profiles":profiles,"installedProfileCopied":false,"sourceFilesRead":false,"providerOrPaidQuota":false})).unwrap()).unwrap();fs::set_permissions(output.join("EXPORT.json"),fs::Permissions::from_mode(0o600)).unwrap();
}
