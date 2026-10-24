//! Explicitly enrolled persistent personal sources, separate from OS drops.
mod model;
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
pub(crate) async fn pet_source_configuration_save(app:tauri::AppHandle,window:tauri::WebviewWindow,config:model::Registry,expected_revision:String,consent:Consent)->Result<Document,String>{
    if !allowed(window.label()){return Err("Abre a gestão de fontes no pet.".into());}
    if expected_revision!="missing"&&(expected_revision.len()!=64||!expected_revision.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())){return Err(message("registryConflict"));}
    #[cfg(target_os="linux")]
    {
        let result=tauri::async_runtime::spawn_blocking(move||store::save_checked(&path(),&expected_revision,config,|old,next|self::consent(old,next,consent)).map(|s|Document{config:s.registry,revision:s.revision}).map_err(message)).await.map_err(|_|"A gravação das fontes foi interrompida. Volta a carregar a configuração.".to_string())??;
        let _=app.emit("pet-sources-changed",serde_json::json!({"revision":result.revision}));Ok(result)
    }
    #[cfg(not(target_os="linux"))]
    {let _=(app,config,expected_revision,consent);Err("As fontes persistentes ainda não estão disponíveis nesta plataforma.".into())}
}

#[cfg(target_os="linux")]
#[derive(Clone)]
pub(crate) struct Selected {source:model::Source,snapshot:store::Snapshot,owner:model::Owner,config_path:PathBuf}
#[cfg(target_os="linux")]
impl Selected{
    pub(crate) fn path(&self)->&str{&self.source.path}
    pub(crate) fn name(&self)->&str{&self.source.name}
    pub(crate) fn binding(&self)->String{
        let tuple=serde_json::json!([self.snapshot.revision,self.source.id,self.source.revision,self.owner.pet_id,self.owner.conversation_id]);
        ring::digest::digest(&ring::digest::SHA256,tuple.to_string().as_bytes()).as_ref().iter().map(|b|format!("{b:02x}")).collect()
    }
    fn validate_with(&self,provider:&str,cid:&str,config:&crate::pet_config::Configuration)->Result<(),&'static str>{
        let pet=crate::pet_config::integration_pet(provider).ok_or("ownerChanged")?;
        if self.owner.pet_id!=pet||self.owner.conversation_id!=cid{return Err("ownerChanged");}
        if crate::pet_config::retired(pet)||!config.permissions(pet).is_ok_and(|p|p.documents){return Err("permissionDenied");}
        if !self.snapshot.unchanged(){return Err("registryChanged");}Ok(())
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
    Ok(Some(Selected{source,snapshot,owner:model::Owner{pet_id:pet.into(),conversation_id:cid.into()},config_path}))
}
#[cfg(target_os="linux")]
pub(crate) fn selected(provider:&str,cid:&str)->Result<Option<Selected>,String>{
    let snapshot=store::load_at(&path()).map_err(message)?;
    let result=select_from(snapshot,crate::pet_config::path(),provider,cid).map_err(message)?;
    if let Some(value)=&result{value.validate(provider,cid)?;}Ok(result)
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
            registry.sources[0].owners=vec![model::Owner{pet_id:crate::pet_config::integration_pet(provider).unwrap().into(),conversation_id:cid.into()}];
            store::save_checked(&root.join("sources.json"),"missing",registry,|old,next|consent(old,next,Consent::PersistentCalendarRead)).unwrap();
            let value=Self{root,source,provider:provider.into(),cid:cid.into()};value.documents(true);value
        }
        pub(crate) fn select(&self,provider:&str,cid:&str)->Option<Selected>{
            select_from(store::load_at(&self.root.join("sources.json")).unwrap(),self.root.join("pets.json"),provider,cid).unwrap()
        }
        pub(crate) fn selected(&self)->Selected{self.select(&self.provider,&self.cid).unwrap()}
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
        let mut changed=next.clone();changed.sources[0].owners[0].conversation_id="other-cid".into();assert_eq!(consent(&next,&changed,Consent::MetadataOnly),Err("consentRequired"));
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
        let mut next=saved.registry.clone();next.sources[0].owners[0].conversation_id="new-cid".into();
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
