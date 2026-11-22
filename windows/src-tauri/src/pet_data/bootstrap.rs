//! Actual app bootstrap: all profile-backed state follows this admission.
use super::{publication::{self,Paths,State,Storage},startup::Claim,Error,Layout,Result};
use std::{path::PathBuf,sync::{Mutex,OnceLock}};

pub(super)struct Session{storage:Mutex<Storage>}
impl Session{
    pub(super)fn prepare(claim:Claim,layout:Layout)->Result<Self>{
        claim.ensure_data_root(&layout)?;
        let active=match publication::inspect(&layout)?{
            State::Absent=>publication::Publication::begin(claim,layout).map_err(|e|e.error)?.publish().map_err(|e|e.error)?,
            State::Incomplete=>publication::recover(claim,layout)?,
            State::AdmissionRecordPresent=>publication::Active::open(claim,layout)?,
        };
        Ok(Self{storage:Mutex::new(active.into_storage()?)})
    }
    pub(super)fn paths(&self)->Result<Paths>{self.storage.lock().map_err(|_|Error::Unavailable)?.paths()}
    fn settings(&self)->Result<crate::settings::Settings>{
        let path=self.paths()?.config.join("settings.json");self.current()?;let restored=self.storage.lock().map_err(|_|Error::Unavailable)?.restored_history()?.is_some();let settings=if restored{match crate::settings::read_initialized(&path){Ok(settings)=>settings,Err(_)=>{crate::log::line("Preferências do perfil admitido disponíveis apenas para revisão; novos efeitos permanecem bloqueados.");crate::settings::load_for_review(&path)}}}else{crate::settings::load_from(&path)};self.current()?;Ok(settings)
    }
    fn current(&self)->Result<()>{self.storage.lock().map_err(|_|Error::Unavailable)?.current()}
}
fn global()->&'static OnceLock<Session>{static PROFILE:OnceLock<Session>=OnceLock::new();&PROFILE}
/// Root1 has no restored history. Admission2 supplies its verified immutable
/// index through the existing Storage before Chat is initialized.
pub(crate)fn restored_history()->Result<Option<std::sync::Arc<super::restored_history::History>>>{
 #[cfg(test)]if global().get().is_none(){return Ok(None);}
 global().get().ok_or(Error::Unavailable)?.storage.lock().map_err(|_|Error::Unavailable)?.restored_history()
}
pub(crate)fn restored_native_session(provider:&str,id:&str)->std::result::Result<(),String>{restored_pet_allowed(provider)?;if let Some(history)=restored_history().map_err(|_|"O perfil de dados está indisponível.".to_string())?{history.native_session_allowed(provider,id).map_err(|_|super::restored_history::NOTICE.to_string())?;}Ok(())}
pub(crate)fn feature_allowed(name:&str)->std::result::Result<(),String>{
 #[cfg(test)]if global().get().is_none(){return Ok(());}
 let profile=global().get().ok_or("O perfil de dados está indisponível.")?;
 let pending=profile.storage.lock().map_err(|_|"O perfil de dados está indisponível.")?.feature_pending(name).map_err(|_|"O perfil de dados mudou.")?;
 // Release the profile lock before consulting the real Settings owner.
 // Immutable restored policy stays intact; only this exact native component
 // receipt/configuration/profile can satisfy its revalidation prerequisite.
 if pending&&!(name=="autostart"&&crate::settings::component_revalidated(name)){return Err(format!("A funcionalidade {name} neste perfil restaurado aguarda revalidação explícita."));}Ok(())
}
pub(crate)fn restored_pet_allowed(provider:&str)->std::result::Result<(),String>{
 if restored_history().map_err(|_|"O perfil de dados está indisponível.")?.is_none(){return Ok(());}
 let settings=crate::settings::read_initialized(&config_dir().join("settings.json"))?;let id=crate::pet_config::integration_pet(provider).ok_or("O pet deste contexto não foi identificado.")?;
 if settings.main_pill!=id&&!settings.active_integrations.iter().any(|p|p==id){return Err("Este pet ainda não está ativo na configuração revista do perfil restaurado.".into());}Ok(())
}
pub(crate)fn admitted()->bool{global().get().is_some()}
pub(crate)fn settings_io_protocol(path:&std::path::Path,create:bool)->Result<bool>{
    #[cfg(test)]if global().get().is_none(){return Ok(false);}
    // Offline native archive inspection may read configuration as data before
    // app admission. It cannot create a protocol or authorize an effect.
    let Some(profile)=global().get()else{return if create{Err(Error::Unavailable)}else{Ok(false)}};
    profile.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_protocol(path,create)
}
pub(crate)fn settings_io_review_protocol(path:&std::path::Path)->Result<()>{
 #[cfg(test)]if global().get().is_none(){return Ok(());}
 global().get().ok_or(Error::Unavailable)?.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_upgrade(path,2)
}
pub(crate)fn settings_io_component_protocol(path:&std::path::Path,_independent:bool)->Result<()>{
 #[cfg(test)]if global().get().is_none(){return Ok(());}
 global().get().ok_or(Error::Unavailable)?.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_upgrade(path,6)
}
pub(crate)fn settings_io_shortcuts_protocol(path:&std::path::Path)->Result<()>{
 #[cfg(test)]if global().get().is_none(){return Ok(());}
 global().get().ok_or(Error::Unavailable)?.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_upgrade(path,7)
}
pub(crate)fn settings_io_shortcut_scope_protocol(path:&std::path::Path)->Result<()>{
 #[cfg(test)]if global().get().is_none(){return Ok(());}
 global().get().ok_or(Error::Unavailable)?.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_upgrade(path,8)
}
pub(crate)fn settings_io_observation(path:&std::path::Path)->Result<Option<super::SettingsIoFormat>>{
    let Some(profile)=global().get()else{return Ok(None)};
    profile.storage.lock().map_err(|_|Error::Unavailable)?.settings_io_observation(path).map(Some)
}
pub(crate)fn install(app:&tauri::AppHandle)->std::result::Result<crate::settings::Settings,std::io::Error>{
    let result=(||{
        if global().get().is_some(){return Err(Error::Busy);}
        let layout=Layout::new(crate::platform::legacy_config_dir(),crate::platform::local_dir())?;
        let claim=Claim::acquire(app,&layout)?;claim.ensure_data_root(&layout)?;
        // Initialize the stable log before the source inventory captures its
        // parent. Later startup/secondary-request messages append the same file.
        crate::log::line("A preparar o perfil de dados");
        let session=Session::prepare(claim,layout)?;session.current()?;
        // Register the admitted lease before the tolerant settings reader can
        // protect/repair a malformed file inside this already admitted profile.
        global().set(session).map_err(|_|Error::Busy)?;
        let admitted=global().get().ok_or(Error::Unavailable)?;admitted.current()?;
        let loaded=admitted.settings()?;admitted.current()?;Ok(loaded)
    })();
    result.map_err(|_|std::io::Error::other("O perfil de dados não foi admitido. Os dados anteriores foram conservados; não foram carregadas definições ou conversas por omissão."))
}
fn selected(session:Option<&Session>,config:bool)->Result<PathBuf>{
    let paths=session.ok_or(Error::Unavailable)?.paths()?;Ok(if config{paths.config}else{paths.data})
}
pub(crate)fn config_dir()->PathBuf{
    // Existing isolated unit fixtures supply explicit paths and do not start an
    // application. Production never has this legacy fallback.
    #[cfg(test)]if global().get().is_none(){return crate::platform::legacy_config_dir();}
    selected(global().get(),true).expect("O perfil de configuração está indisponível; gravação recusada")
}
pub(crate)fn data_dir()->PathBuf{
    #[cfg(test)]if global().get().is_none(){return crate::platform::local_dir();}
    selected(global().get(),false).expect("O perfil de dados está indisponível; gravação recusada")
}
pub(crate)fn write_allowed()->std::io::Result<()>{
    #[cfg(test)]if global().get().is_none(){return Ok(());}
    global().get().ok_or_else(||std::io::Error::other("Perfil não admitido"))?.current().map_err(|_|std::io::Error::other("O perfil mudou; gravação recusada"))
}

#[cfg(test)]#[path="bootstrap_tests.rs"]mod tests;
