//! ASR context captured before recording. Shared by Voice and dictation; it
//! grants neither capture, external inference, playback nor desktop control.
use std::sync::{Arc,atomic::AtomicBool};
use tauri::{AppHandle,Manager};
use crate::{settings::Settings,pet_config::Configuration,speech::Transcript};
use super::input::Recognition;

pub(crate) struct RecognitionScope {
    pub(super) settings:Settings,
    pub(super) origin:Arc<crate::settings::CapturedOrigin>,
    pub(super) configuration:Configuration,
    pub(super) recognition:Recognition,
}
impl RecognitionScope {
    pub(crate) fn from_parts(settings:Settings,configuration:Configuration,origin:Arc<crate::settings::CapturedOrigin>)->Result<Self,String>{
        let recognition=Recognition::prepare(&settings,&configuration)?;
        Ok(Self{settings,origin,configuration,recognition})
    }
    pub(crate) fn capture(app:&AppHandle)->Result<Self,String>{
        crate::settings::optional::check(crate::settings::optional::Operation::Voice)?;
        let(settings,origin)={let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().map_err(|_|"As preferências do reconhecimento estão indisponíveis.")?;
            (settings.clone(),Arc::new(crate::settings::capture_origin(&settings)))};
        let scope=Self::from_parts(settings,crate::pet_config::read_existing()?,origin)?;
        scope.check()?;Ok(scope)
    }
    pub(super) fn provenance(&self)->Result<(),String>{crate::settings::optional::check_origin(crate::settings::optional::Operation::Voice,&self.origin)}
    pub(crate) fn check(&self)->Result<(),String>{self.check_with(||self.provenance(),crate::pet_config::read_existing)}
    pub(super) fn check_with(&self,provenance:impl FnOnce()->Result<(),String>,observe:impl FnOnce()->Result<Configuration,String>)->Result<(),String>{
        provenance()?;let current=observe()?;
        if serde_json::to_vec(&current).map_err(|_|"A configuração dos pets está indisponível.")?!=serde_json::to_vec(&self.configuration).map_err(|_|"A configuração capturada está indisponível.")?{
            return Err("A configuração dos pets mudou durante o reconhecimento. Inicia outro pedido; o anterior não foi redirecionado.".into());
        }Ok(())
    }
    pub(crate) fn ready(&self)->bool{self.recognition.ready()}
    pub(crate) fn transcribe(&self,samples:&[f32],stop:Option<&AtomicBool>,current:impl Fn()->bool)->Option<Transcript>{
        self.recognition.transcribe_guarded(samples,stop,||self.check().is_ok()&&current())
    }
}

#[cfg(all(test,target_os="linux"))]
#[path="recognition_scope_tests.rs"]mod tests;
