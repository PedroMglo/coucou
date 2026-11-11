//! Immutable inputs for one admitted audio occurrence, never a permission.
//! Capture before waiting for audio; later UI changes cannot retarget it.
use std::{path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
use tauri::AppHandle;
use crate::{pet_config::Configuration, settings::Settings, speech::{self, audio, Transcript}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Recognizer {
    Parakeet,
    Whisper { python: PathBuf, model: PathBuf },
}

#[derive(Clone)]
pub(super) struct Recognition {
    backend: Recognizer,
    vocabulary: Vec<String>,
}
impl Recognition {
    pub(super) fn backend(&self) -> &Recognizer { &self.backend }
    pub(super) fn prepare(settings: &Settings, config: &Configuration) -> Result<Self, String> {
        if !matches!(settings.voice_asr_engine.as_str(), "" | "auto" | "parakeet" | "whisper") {
            return Err("O motor ASR configurado não é suportado. Não foi escolhido outro motor.".into());
        }
        let backend = if super::asr_engine(settings) == "whisper" {
            if settings.voice_asr_python.is_empty() || settings.voice_asr_model.is_empty() {
                return Err("Configura o runtime e o modelo Whisper antes de iniciar a captura.".into());
            }
            let expand = |p: &str| p.strip_prefix("~/")
                .map(|p| crate::platform::home_dir().join(p)).unwrap_or_else(|| p.into());
            Recognizer::Whisper { python: expand(&settings.voice_asr_python), model: expand(&settings.voice_asr_model) }
        } else { Recognizer::Parakeet };
        Ok(Self { backend, vocabulary: super::asr_vocabulary(config, settings) })
    }

    pub(super) fn ready(&self)->bool{match &self.backend{Recognizer::Parakeet=>speech::is_ready(),Recognizer::Whisper{python,model}=>python.is_file()&&model.join("model.bin").is_file()}}
    pub(super) fn transcribe_guarded(&self, samples: &[f32], stop: Option<&AtomicBool>,current:impl Fn()->bool) -> Option<Transcript> {
        self.transcribe_with_validity(samples, stop, ||current(), |backend, samples, vocabulary| match backend {
            Recognizer::Parakeet => speech::transcribe(samples),
            Recognizer::Whisper { python, model } => {
                if !python.is_file() || !model.join("model.bin").is_file() { return None; }
                speech::whisper::transcribe_with_current(python, model, samples, stop, vocabulary,&current)
            }
        })
    }

    // The production chunk/cancellation path is also used with synthetic ASR.
    #[cfg(test)]
    fn transcribe_with(&self,samples:&[f32],stop:Option<&AtomicBool>,recognize:impl FnMut(&Recognizer,&[f32],&[String])->Option<Transcript>)->Option<Transcript>{self.transcribe_with_validity(samples,stop,||true,recognize)}
    pub(super) fn transcribe_with_validity(&self, samples: &[f32], stop: Option<&AtomicBool>,current:impl Fn()->bool,
        mut recognize: impl FnMut(&Recognizer, &[f32], &[String]) -> Option<Transcript>,
    ) -> Option<Transcript> {
        let cancelled = || stop.is_some_and(|s| s.load(Ordering::Acquire))||!current();
        if cancelled() { return None; }
        if matches!(self.backend, Recognizer::Parakeet) || samples.is_empty() {
            let result = recognize(&self.backend, samples, &self.vocabulary);
            return if cancelled() { None } else { result };
        }
        let mut result = Transcript::default();
        for (_, chunk) in audio::split_for_model(samples, speech::SAMPLE_RATE, 28.0) {
            if cancelled() { return None; }
            let part = recognize(&self.backend, chunk, &self.vocabulary)?;
            if cancelled() { return None; }
            if !part.text.is_empty() {
                if !result.text.is_empty() { result.text.push(' '); }
                result.text.push_str(&part.text);
            }
        }
        if cancelled() { None } else { Some(result) }
    }
}

#[derive(Clone)]
pub(super) struct Input {
    context:std::sync::Arc<super::RecognitionScope>,
    pub source: audio::Source,
    pub playback: super::tts::Plan,
    flow: Option<std::sync::Weak<crate::flow::Flow>>,
}
impl std::ops::Deref for Input{type Target=super::RecognitionScope;fn deref(&self)->&Self::Target{&self.context}}
impl Input {
    #[cfg(test)]
    fn prepare(settings:Settings,configuration:Configuration,mic:String)->Result<Self,String>{
        let origin=std::sync::Arc::new(crate::settings::capture_origin(&settings));
        Self::prepare_with_origin(settings,configuration,mic,origin)
    }
    #[cfg(test)]
    fn prepare_with_origin(settings: Settings, configuration: Configuration, mic: String,origin:std::sync::Arc<crate::settings::CapturedOrigin>) -> Result<Self, String> {
        let context=std::sync::Arc::new(super::RecognitionScope::from_parts(settings,configuration,origin)?);
        if mic.len() > 1024 || mic.chars().any(char::is_control) { return Err("A seleção de microfone é inválida.".into()); }
        Ok(Self { context, source: audio::Source::Mic((!mic.is_empty()).then_some(mic)), playback:super::tts::Plan::silent(), flow:None })
    }

    pub(super) fn capture(app: &AppHandle) -> Result<Self, String> {
        let context=std::sync::Arc::new(super::RecognitionScope::capture(app)?);
        let flow = crate::flow::get(app).ok_or("A origem das preferências do microfone está indisponível.")?;
        let (mic,playback)={let store=flow.store.lock().map_err(|_| "As preferências do microfone estão ocupadas.")?;
            (store.prefs.mic.clone(),super::tts::Plan::prepare(&context.settings,&store.prefs)?)};
        if mic.len()>1024||mic.chars().any(char::is_control){return Err("A seleção de microfone é inválida.".into());}
        let mut input=Self{context,source:audio::Source::Mic((!mic.is_empty()).then_some(mic)),playback:super::tts::Plan::silent(),flow:None};
        input.playback=playback;input.flow=Some(std::sync::Arc::downgrade(&flow));
        input.check()?;
        Ok(input)
    }

    pub(super) fn check(&self) -> Result<(), String> {
        self.check_with(||self.source_current(), crate::pet_config::read_existing)
    }

    /// After recognition a deterministic operation pins its own pet policy.
    /// Keep voice/source/IO provenance without making another pet's unrelated
    /// configuration change revoke this pet's captured System grant.
    pub(super) fn source_current(&self) -> Result<(), String> {
            if let Some(owner)=&self.flow{
                let flow=owner.upgrade().ok_or("As preferências da voz já não têm dono.")?;
                let store=flow.store.try_lock().map_err(|_|"As preferências da voz estão ocupadas.")?;
                let source=audio::Source::Mic((!store.prefs.mic.is_empty()).then(||store.prefs.mic.clone()));
                if source!=self.source||!self.playback.matches(&store.prefs){return Err("A seleção de microfone ou reprodução mudou durante o turno. Inicia outro pedido; o anterior não foi redirecionado.".into());}
            }
            crate::settings::optional::check_origin(crate::settings::optional::Operation::Voice,&self.origin)?;
            Ok(())
    }

    fn check_with(&self, provenance: impl FnOnce() -> Result<(), String>,
        observe: impl FnOnce() -> Result<Configuration, String>,
    ) -> Result<(), String> {
        self.context.check_with(provenance,observe)
    }

    pub(super) fn transcribe(&self, samples: &[f32], stop: Option<&AtomicBool>) -> Option<Transcript> {
        self.recognition.transcribe_guarded(samples,stop,||self.check().is_ok())
    }

    pub(super) fn settings_for(&self, provider: &str) -> Settings {
        crate::cli_chat::voice_settings(&self.settings, provider)
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
