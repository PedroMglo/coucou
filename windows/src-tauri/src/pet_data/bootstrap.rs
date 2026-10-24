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
        let path=self.paths()?.config.join("settings.json");self.current()?;let settings=crate::settings::load_from(&path);self.current()?;Ok(settings)
    }
    fn current(&self)->Result<()>{self.storage.lock().map_err(|_|Error::Unavailable)?.current()}
}
fn global()->&'static OnceLock<Session>{static PROFILE:OnceLock<Session>=OnceLock::new();&PROFILE}
pub(crate)fn admitted()->bool{global().get().is_some()}
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
        let loaded=crate::settings::load();admitted.current()?;Ok(loaded)
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
