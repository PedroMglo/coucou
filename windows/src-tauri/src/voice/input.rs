//! Immutable inputs for one admitted audio occurrence, never a permission.
//! Capture before waiting for audio; later UI changes cannot retarget it.
use std::{path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Manager};
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

    pub(super) fn transcribe(&self, samples: &[f32], stop: Option<&AtomicBool>) -> Option<Transcript> {
        self.transcribe_with(samples, stop, |backend, samples, vocabulary| match backend {
            Recognizer::Parakeet => speech::transcribe(samples),
            Recognizer::Whisper { python, model } => {
                if !python.is_file() || !model.join("model.bin").is_file() { return None; }
                speech::whisper::transcribe_with_vocabulary(python, model, samples, stop, vocabulary)
            }
        })
    }

    // The production chunk/cancellation path is also used with synthetic ASR.
    fn transcribe_with(&self, samples: &[f32], stop: Option<&AtomicBool>,
        mut recognize: impl FnMut(&Recognizer, &[f32], &[String]) -> Option<Transcript>,
    ) -> Option<Transcript> {
        let cancelled = || stop.is_some_and(|s| s.load(Ordering::Acquire));
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
    pub settings: Settings,
    origin:std::sync::Arc<crate::settings::CapturedOrigin>,
    pub configuration: Configuration,
    pub source: audio::Source,
    pub recognition: Recognition,
    pub playback: super::tts::Plan,
    flow: Option<std::sync::Weak<crate::flow::Flow>>,
}
impl Input {
    #[cfg(test)]
    fn prepare(settings:Settings,configuration:Configuration,mic:String)->Result<Self,String>{
        let origin=std::sync::Arc::new(crate::settings::capture_origin(&settings));
        Self::prepare_with_origin(settings,configuration,mic,origin)
    }
    fn prepare_with_origin(settings: Settings, configuration: Configuration, mic: String,origin:std::sync::Arc<crate::settings::CapturedOrigin>) -> Result<Self, String> {
        let recognition = Recognition::prepare(&settings, &configuration)?;
        if mic.len() > 1024 || mic.chars().any(char::is_control) { return Err("A seleção de microfone é inválida.".into()); }
        Ok(Self { settings, origin, configuration, source: audio::Source::Mic((!mic.is_empty()).then_some(mic)), recognition, playback:super::tts::Plan::silent(), flow:None })
    }

    pub(super) fn capture(app: &AppHandle) -> Result<Self, String> {
        crate::settings::optional::check(crate::settings::optional::Operation::Voice)?;
        // Never hold Settings and Flow/configuration locks together.
        let(settings,origin)={let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().map_err(|_|"As preferências da voz estão indisponíveis.")?;
            (settings.clone(),std::sync::Arc::new(crate::settings::capture_origin(&settings)))};
        let configuration = crate::pet_config::read_existing()?;
        let flow = crate::flow::get(app).ok_or("A origem das preferências do microfone está indisponível.")?;
        let (mic,playback)={let store=flow.store.lock().map_err(|_| "As preferências do microfone estão ocupadas.")?;
            (store.prefs.mic.clone(),super::tts::Plan::prepare(&settings,&store.prefs)?)};
        let mut input = Self::prepare_with_origin(settings, configuration, mic, origin)?;
        input.playback=playback;input.flow=Some(std::sync::Arc::downgrade(&flow));
        input.check()?;
        Ok(input)
    }

    pub(super) fn check(&self) -> Result<(), String> {
        self.check_with(|| {
            if let Some(owner)=&self.flow{
                let flow=owner.upgrade().ok_or("As preferências da voz já não têm dono.")?;
                let store=flow.store.try_lock().map_err(|_|"As preferências da voz estão ocupadas.")?;
                let source=audio::Source::Mic((!store.prefs.mic.is_empty()).then(||store.prefs.mic.clone()));
                if source!=self.source||!self.playback.matches(&store.prefs){return Err("A seleção de microfone ou reprodução mudou durante o turno. Inicia outro pedido; o anterior não foi redirecionado.".into());}
            }
            crate::settings::optional::check_origin(crate::settings::optional::Operation::Voice,&self.origin)?;
            Ok(())
        }, crate::pet_config::read_existing)
    }

    fn check_with(&self, provenance: impl FnOnce() -> Result<(), String>,
        observe: impl FnOnce() -> Result<Configuration, String>,
    ) -> Result<(), String> {
        provenance()?;
        let current = observe()?;
        if serde_json::to_vec(&current).map_err(|_| "A configuração dos pets está indisponível.")?
            != serde_json::to_vec(&self.configuration).map_err(|_| "A configuração capturada está indisponível.")? {
            return Err("A configuração dos pets mudou durante este turno de voz. Inicia outro pedido; o anterior não foi redirecionado.".into());
        }
        Ok(())
    }

    pub(super) fn transcribe(&self, samples: &[f32], stop: Option<&AtomicBool>) -> Option<Transcript> {
        if stop.is_some_and(|s|s.load(Ordering::Acquire)){return None;}
        self.check().ok()?;
        let result = self.recognition.transcribe(samples, stop)?;
        // Revocation/configuration change during local ASR cannot publish a
        // transcript into a newly selected model or pet.
        self.check().ok()?;
        Some(result)
    }

    pub(super) fn settings_for(&self, provider: &str) -> Settings {
        crate::cli_chat::voice_settings(&self.settings, provider)
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
