// « OK Coucou » on Windows and Linux: say it, then ask anything. Port of the
// Mac's Voice/ (VoiceEngine.swift and its friends).
//
// While it is on, the microphone is open and a cheap energy detector cuts what
// you say into stretches; only a stretch of speech goes to Parakeet, on this
// computer, and only a stretch that starts with « OK Coucou » goes further:
// what you said after it goes to Claude with your own key (brain.rs), never
// the audio. Mochi shows it all in the pill at the bottom of the screen —
// listening, thinking, then the answer, said out loud by the system's voice
// (the pill's speechSynthesis) or ElevenLabs with your key.
//
// It stops listening while you dictate, while a call is recorded and while it
// speaks itself.

pub mod brain;
pub mod local;
pub mod manual;
pub mod wake;
pub mod dialogue;
mod services;
pub mod session;
mod tts;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::flow::pill;
use crate::speech::{self, audio};

/// What the pill shows of a conversation.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct VoiceView {
    /// "listening", "thinking", "speaking".
    pub phase: String,
    /// What you said (after « OK Coucou »).
    pub heard: String,
    /// Coucou's answer.
    pub answer: String,
}

struct Running {
    stop: Arc<AtomicBool>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);
static CAPTURE_OWNER: Mutex<()> = Mutex::new(());

/// Wait for the previous voice capture to close instead of guessing how long
/// its ASR or tool cancellation takes. A second Super also cancels this wait.
fn capture_lease(stop: &AtomicBool) -> Option<std::sync::MutexGuard<'static, ()>> {
    loop {
        if stop.load(Ordering::Relaxed) { return None; }
        match CAPTURE_OWNER.try_lock() {
            Ok(lease) => return Some(lease),
            Err(std::sync::TryLockError::Poisoned(error)) => return Some(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

struct Pending {speech:bool,tx:mpsc::Sender<String>}
/// Tool replies survive turning speech off while their CLI job continues.
fn pending() -> &'static Mutex<HashMap<u64, Pending>> {
    static P: OnceLock<Mutex<HashMap<u64, Pending>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(HashMap::new()))
}
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn cancel_playback(pending:&mut HashMap<u64,Pending>) {
    pending.retain(|_,p|if p.speech {let _=p.tx.send(String::new());false}else{true});
}

/// Whether « OK Coucou » can run: the voice model and a Claude key.
pub fn provider_ready(app: &AppHandle) -> bool {
    let Some(shared) = app.try_state::<crate::Shared>() else { return false };
    let settings = shared.settings.lock().unwrap();
    match settings.chat_provider.as_str() {
        "ollama" => local::config(&settings).is_some(),
        "codex" => crate::codex_chat::installed(),
        "antigravity" => crate::gravity_chat::executable().is_ok(),
        "copilot"=>crate::copilot_chat::executable().is_ok(),
        "" | "anthropic" => crate::secrets::present(crate::claude::KEY),
        _ => false,
    }
}

pub fn requirements_met(app: &AppHandle) -> bool {
    asr_ready(app) && provider_ready(app)
}

/// Starts or stops listening to follow the preference.
pub fn sync(app: &AppHandle) {
    let wanted = crate::flow::get(app).is_some_and(|f| f.store.lock().unwrap().prefs.voice);
    let explicit = session::active() || manual::active();
    let mut running = RUNNING.lock().unwrap();
    match (wanted && !explicit && !session::suspend_hotword() && requirements_met(app), running.is_some()) {
        (true, false) => {
            let stop = Arc::new(AtomicBool::new(false));
            let app2 = app.clone();
            let stop2 = stop.clone();
            std::thread::Builder::new().name("coucou-voice".into()).spawn(move || run(app2, stop2)).ok();
            preload_asr(app,stop.clone());
            *running = Some(Running { stop });
            crate::log::line("voice: « OK Coucou » listening");
        }
        (false, true) => {
            if let Some(r) = running.take() {
                r.stop.store(true, Ordering::Relaxed);
                cancel_speech(app);
            }
            let dictation = crate::flow::get(app).is_some_and(|f| f.store.lock().unwrap().prefs.enabled);
            if !dictation && !explicit && !crate::notes::is_recording(app) {
                speech::unload();
            }
            crate::log::line("voice: « OK Coucou » off");
        }
        _ => {}
    }
}

/// « OK Coucou » is listening (the voice model stays loaded).
pub fn is_listening() -> bool {
    session::active() || manual::active() || RUNNING.lock().unwrap().is_some()
}

/// Starts again on another microphone.
pub fn restart(app: &AppHandle) {
    if let Some(r) = RUNNING.lock().unwrap().take() {
        r.stop.store(true, Ordering::Relaxed);
        cancel_speech(app);
    }
    sync(app);
}

fn show(app: &AppHandle, view: Option<VoiceView>) {
    let owner = app.try_state::<crate::Shared>().and_then(|shared| {
        let settings = shared.settings.lock().unwrap();
        dialogue::Pet::from_provider(&settings.chat_provider).map(dialogue::Pet::pill)
    });
    show_for_pet(app, view, owner.or(Some("ai_ollama")));
}

fn show_for_pet(app: &AppHandle, view: Option<VoiceView>, pet: Option<&str>) {
    if crate::companions::enabled(app) {
        let payload = view.as_ref().map(|view| json!({"phase":view.phase,"heard":view.heard,"answer":view.answer,"pet":pet}));
        let _ = app.emit_to(crate::companions::LABEL, "companions-voice", payload);
        pill::update(app, |s| s.voice = None);
    } else { pill::update(app, |s| s.voice = view); }
}

enum Mode {
    /// Waiting for « OK Coucou ».
    Wake,
    /// Heard « OK Coucou » alone: what comes next is the question.
    Command { text: String, until: Instant },
    /// Coucou asked something back: an answer comes without « OK Coucou ».
    FollowUp { until: Instant },
}

fn run(app: AppHandle, stop: Arc<AtomicBool>) {
    let Some(_lease) = capture_lease(&stop) else { return };
    let mic = crate::flow::get(&app).map(|f| f.store.lock().unwrap().prefs.mic.clone()).unwrap_or_default();
    let (tx, rx) = mpsc::sync_channel::<Vec<f32>>(8);
    let capture = audio::Capture::start(audio::Source::Mic(if mic.is_empty() { None } else { Some(mic) }), move |s| {
        let _ = tx.try_send(s.to_vec());
    });
    let mut capture = match capture {
        Ok(c) => c,
        Err(err) => {
            crate::log::line(format!("voice: microphone: {err}"));
            *RUNNING.lock().unwrap() = None;
            return;
        }
    };
    let mut vad = wake::Vad::default();
    let mut mode = Mode::Wake;
    let mut brain = brain::Brain::default();
    let mut hide_at: Option<Instant> = None;
    while !stop.load(Ordering::Relaxed) {
        let chunk = match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(c) => Some(c),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let now = Instant::now();
        // The pill's last answer goes away after a while.
        if hide_at.is_some_and(|t| now > t) {
            hide_at = None;
            show(&app, None);
        }
        match &mode {
            Mode::Command { until, .. } | Mode::FollowUp { until } if now > *until => {
                mode = Mode::Wake;
                vad.hangover = 25;
                show(&app, None);
            }
            _ => {}
        }
        // Dictation and calls have the microphone: Coucou doesn't listen meanwhile.
        if pill::dictating(&app) || crate::notes::is_recording(&app) {
            vad.reset();
            continue;
        }
        let Some(chunk) = chunk else { continue };
        for segment in vad.feed(&chunk) {
            if segment.len() < speech::SAMPLE_RATE as usize * 35 / 100 {
                continue;
            }
            let Some(heard) = transcribe(&app, &segment, Some(&stop)).map(|t| t.text.trim().to_string()) else { continue };
            if heard.is_empty() {
                continue;
            }
            let question = match &mut mode {
                Mode::Wake => match wake::after_wake(&heard) {
                    Some(rest) => {
                        crate::log::line("voice: « OK Coucou » heard");
                        hide_at = None;
                        if crate::flow::text::word_count(&rest) >= 2 {
                            Some(rest)
                        } else {
                            show(&app, Some(VoiceView { phase: "listening".into(), heard: rest.clone(), answer: String::new() }));
                            mode = Mode::Command { text: rest, until: now + Duration::from_secs(7) };
                            // A question may come with pauses in it.
                            vad.hangover = 40;
                            None
                        }
                    }
                    None => None,
                },
                Mode::Command { text, .. } => {
                    let full = format!("{text} {heard}").trim().to_string();
                    Some(full)
                }
                Mode::FollowUp { .. } => Some(heard),
            };
            let Some(question) = question else { continue };
            mode = Mode::Wake;
            vad.hangover = 25;
            show(&app, Some(VoiceView { phase: "thinking".into(), heard: question.clone(), answer: String::new() }));
            let provider = app.try_state::<crate::Shared>().map(|s| s.settings.lock().unwrap().chat_provider.clone()).unwrap_or_default();
            brain.forget_if_idle();
            let reply = brain.respond_as(&app, &question, &provider, Some(stop.clone()));
            if stop.load(Ordering::Relaxed) { break; }
            let answer = reply.as_ref().map(|r| r.text.clone()).unwrap_or_default();
            if answer.is_empty() {
                let done = reply.is_some_and(|r| r.acted);
                show(&app, Some(VoiceView {
                    phase: "speaking".into(),
                    heard: question,
                    answer: if done { "✓".into() } else { crate::i18n::t("Coucou couldn't answer: check the selected voice provider.") },
                }));
                hide_at = Some(Instant::now() + Duration::from_secs(5));
                continue;
            }
            show(&app, Some(VoiceView { phase: "speaking".into(), heard: question.clone(), answer: answer.clone() }));
            speak_stoppable(&app, &answer, Some(&stop), None);
            vad.reset();
            // Drop what the microphone heard of Coucou's own voice.
            while rx.try_recv().is_ok() {}
            if answer.trim_end().ends_with('?') {
                mode = Mode::FollowUp { until: Instant::now() + Duration::from_secs(8) };
                vad.hangover = 40;
                show(&app, Some(VoiceView { phase: "listening".into(), heard: String::new(), answer }));
            } else {
                hide_at = Some(Instant::now() + Duration::from_secs(6));
            }
        }
    }
    capture.stop();
    show(&app, None);
}

// ── Speaking ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Say {
    id: u64,
    text: String,
    lang: Option<String>,
    /// Audio data URI (local WAV or ElevenLabs MP3).
    audio: Option<String>,
    gender: String,
    allow_system_fallback: bool,
}

/// Says `text` and returns once it is said (or after a sensible time).
async fn until_stopped<T>(future: impl std::future::Future<Output = T>, stop: Option<&AtomicBool>) -> Option<T> {
    tokio::pin!(future);
    loop {
        if stop.is_some_and(|s| s.load(Ordering::Relaxed)) { return None; }
        tokio::select! {
            result = &mut future => return Some(result),
            _ = tokio::time::sleep(Duration::from_millis(50)), if stop.is_some() => {}
        }
    }
}

fn asr_engine(settings:&crate::settings::Settings)->&'static str {
    match settings.voice_asr_engine.as_str() {
        "parakeet"=>"parakeet",
        "whisper"=>"whisper",
        _ if !settings.voice_asr_python.is_empty()&&!settings.voice_asr_model.is_empty()=>"whisper",
        _=>"parakeet",
    }
}
fn whisper_ready(settings:&crate::settings::Settings)->bool {
    if settings.voice_asr_python.is_empty()||settings.voice_asr_model.is_empty(){return false;}
    let expand=|path: &str|path.strip_prefix("~/").map(|p|crate::platform::home_dir().join(p)).unwrap_or_else(||path.into());
    expand(&settings.voice_asr_python).is_file() && expand(&settings.voice_asr_model).join("model.bin").is_file()
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RecognitionStatus{engine:&'static str,ready:bool,whisper_available:bool}
pub(crate) fn recognition_status(app:&AppHandle)->RecognitionStatus {
    let settings=app.state::<crate::Shared>().settings.lock().unwrap().clone();
    let engine=asr_engine(&settings);let whisper_available=whisper_ready(&settings);
    RecognitionStatus{engine,ready:if engine=="whisper"{whisper_available}else{crate::speech::is_ready()},whisper_available}
}
pub(crate) fn asr_ready(app: &AppHandle) -> bool {recognition_status(app).ready}
fn preload_asr(app: &AppHandle, stop: std::sync::Arc<AtomicBool>) {
    if asr_engine(&app.state::<crate::Shared>().settings.lock().unwrap())=="parakeet" { crate::speech::preload(); return; }
    let app=app.clone(); std::thread::spawn(move || { let _=transcribe(&app,&[],Some(&stop)); });
}

const ASR_TECHNICAL_TERMS:&[&str]=&["Docker Compose","unit tests","pull requests","issues","next steps"];
/// Canonical identities only. Aliases and learned voice forms remain parser data,
/// never ASR hints. This function does not change config, grants or recognized text.
fn asr_vocabulary(config:&crate::pet_config::Configuration,settings:&crate::settings::Settings)->Vec<String>{
    let mut terms=Vec::new();let mut seen=std::collections::BTreeSet::new();
    let names=dialogue::Pet::ALL.into_iter()
        .filter(|pet|!matches!(pet,dialogue::Pet::Claude|dialogue::Pet::Gemini))
        .map(dialogue::Pet::pill)
        .filter(|id|!crate::pet_config::retired(id))
        .filter(|id|settings.main_pill==*id||settings.active_integrations.iter().any(|active|active.as_str()==*id))
        .filter_map(|id|config.pets.get(id).map(|pet|pet.name.as_str()));
    for name in names.chain(ASR_TECHNICAL_TERMS.iter().copied()){
        let name=name.trim();let key:String=name.chars().map(|c|if c.is_ascii(){c.to_ascii_lowercase()}else{c}).collect();
        if name.is_empty()||name.len()>crate::speech::whisper::MAX_VOCABULARY_TERM_BYTES||name.chars().any(char::is_control)||wake::normalize(name)=="ola"{continue;}
        if seen.insert(key){terms.push(name.to_string());}
    }
    // Only configured static voice routes; no executable/auth/capability query.
    // Claude/Gemini are retired routes (session::available always false).
    terms
}
fn current_asr_vocabulary(settings:&crate::settings::Settings)->Vec<String>{
    let config=crate::pet_config::read_existing().unwrap_or_else(|_|crate::pet_config::defaults());
    asr_vocabulary(&config,settings)
}
pub(crate) fn transcribe(app: &AppHandle, samples: &[f32], stop: Option<&AtomicBool>) -> Option<crate::speech::Transcript> {
    let settings=app.state::<crate::Shared>().settings.lock().unwrap().clone();
    if stop.is_some_and(|s|s.load(Ordering::Relaxed)){return None;}
    if asr_engine(&settings)=="parakeet" {
        let result=crate::speech::transcribe(samples);
        return if stop.is_some_and(|s|s.load(Ordering::Relaxed)){None}else{result};
    }
    if !whisper_ready(&settings){return None;}
    let expand=|path: &str|path.strip_prefix("~/").map(|p|crate::platform::home_dir().join(p)).unwrap_or_else(||path.into());
    let python=expand(&settings.voice_asr_python); let model=expand(&settings.voice_asr_model);
    if samples.is_empty() { return crate::speech::whisper::transcribe(&python,&model,samples,stop); }
    let vocabulary=current_asr_vocabulary(&settings);
    let mut result=crate::speech::Transcript::default();
    for (_,chunk) in crate::speech::audio::split_for_model(samples,crate::speech::SAMPLE_RATE,28.0) {
        let part=crate::speech::whisper::transcribe_with_vocabulary(&python,&model,chunk,stop,&vocabulary)?;
        if !part.text.is_empty() { if !result.text.is_empty() { result.text.push(' '); } result.text.push_str(&part.text); }
    }
    Some(result)
}

fn cancel_speech(app: &AppHandle) {
    let _ = app.emit_to(pill::LABEL, "voice-cancel", ());
    let _ = app.emit_to(crate::companions::LABEL, "voice-cancel", ());
    cancel_playback(&mut pending().lock().unwrap());
    tts::cancel_native();
}

fn speak_stoppable(app: &AppHandle, text: &str, stop: Option<&AtomicBool>, provider: Option<&str>) {
    speak_with_language(app, text, stop, provider, None);
}

fn speak(app: &AppHandle, text: &str, stop: Option<&AtomicBool>, language: Option<&str>) {
    speak_with_language(app, text, stop, None, language);
}

fn speak_with_language(app: &AppHandle, text: &str, stop: Option<&AtomicBool>, provider: Option<&str>, language: Option<&str>) {
    if stop.is_some_and(|s| s.load(Ordering::Relaxed)) { return; }
    let epoch = tts::epoch();
    let (output, gender) = crate::flow::get(app)
        .map(|f| {
            let p = &f.store.lock().unwrap().prefs;
            (p.voice_output.clone(), p.voice_gender.clone())
        })
        .unwrap_or_else(|| ("system".into(), "female".into()));
    if output == "off" {
        return;
    }
    if output == "edge" {
        if tts::speak_streaming(app, text, stop).is_some() { return; }
    }
    let audio = if output == "elevenlabs" { elevenlabs(text, &gender).map(|mp3| format!("data:audio/mpeg;base64,{}", base64(&mp3))) }
        else { tts::generate(app, &output, provider, text, stop) };
    if stop.is_some_and(|s| s.load(Ordering::Relaxed)) || tts::epoch() != epoch { return; }
    // An unavailable online voice keeps the written answer. Never silently
    // switch to the voices the user rejected or to a paid speech API.
    if output == "edge" && audio.is_none() { return; }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel();
    pending().lock().unwrap().insert(id, Pending{speech:true,tx});
    // WebKit can suspend the hidden pill. The visible pet surface owns audio
    // while companions are enabled, just as it owns the conversation bubble.
    let target = if crate::companions::enabled(app) { crate::companions::LABEL } else { pill::LABEL };
    crate::log::line(format!("voice: playback request id={id}, output={output}, window={target}, audio={}", audio.is_some()));
    let _ = app.emit_to(target, "voice-say", Say { id, text: text.into(), lang: language.or_else(|| wake::speech_lang(text)).map(str::to_string), audio, gender, allow_system_fallback: output != "edge" });
    let words = crate::flow::text::word_count(text) as u64;
    let until = Instant::now() + Duration::from_millis(4000 + words.min(200) * 600);
    while Instant::now() < until {
        // The controller cancels playback before releasing microphone ownership.
        // A superseded waiter must never send a global cancel to a newer voice.
        if stop.is_some_and(|s| s.load(Ordering::Relaxed)) { break; }
        if rx.recv_timeout(Duration::from_millis(50)).is_ok() { break; }
    }
    pending().lock().unwrap().remove(&id);
}

/// The pill says it has finished speaking (`id`), or the island answers a question.
#[tauri::command]
pub fn voice_done(id: u64, text: Option<String>, played: Option<bool>) {
    if let Some(played) = played { crate::log::line(format!("voice: playback complete id={id}, played={played}")); }
    if let Some(p) = pending().lock().unwrap().remove(&id) {
        let _ = p.tx.send(text.unwrap_or_default());
    }
}

/// Where the webview has no voice (WebKitGTK often): speech-dispatcher or
/// espeak-ng, when installed.
#[tauri::command]
pub async fn voice_say_native(text: String, lang: Option<String>) -> bool {
    tauri::async_runtime::spawn_blocking(move || {
        tts::say_native(&text, lang.as_deref())
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
pub async fn voice_preview(app: AppHandle, provider: Option<String>) {
    if session::active() || pill::dictating(&app) || crate::notes::is_recording(&app) { return; }
    cancel_speech(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let raquel = crate::flow::get(&app).is_some_and(|f| f.store.lock().unwrap().prefs.voice_output == "edge");
        let sample = if raquel { "Olá! Esta é a minha voz para as nossas conversas.".into() }
            else { crate::i18n::t("Hello! This is my voice for our conversations.") };
        speak_stoppable(&app, &sample, None, provider.as_deref());
    }).await.ok();
}

/// ElevenLabs' MP3 for `text`, with the user's key; None and the system voice speaks.
fn elevenlabs(text: &str, gender: &str) -> Option<Vec<u8>> {
    let key = crate::secrets::get("elevenlabs-api-key")?;
    tauri::async_runtime::block_on(async {
        let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().ok()?;
        let voices: Value = client.get("https://api.elevenlabs.io/v1/voices").header("xi-api-key", &key).send().await.ok()?.json().await.ok()?;
        let voice = pick_voice(voices["voices"].as_array()?, gender)?;
        let response = client
            .post(format!("https://api.elevenlabs.io/v1/text-to-speech/{voice}?output_format=mp3_44100_128"))
            .header("xi-api-key", &key)
            .header("accept", "audio/mpeg")
            .json(&json!({ "text": text, "model_id": "eleven_flash_v2_5" }))
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            crate::log::line(format!("voice: ElevenLabs {}", response.status()));
            return None;
        }
        Some(response.bytes().await.ok()?.to_vec())
    })
}

/// First English voice of the gender (a young one first for a man); else any
/// voice of the gender; else the first one. Same choice as the Mac.
pub fn pick_voice(voices: &[Value], gender: &str) -> Option<String> {
    let label = |v: &Value, k: &str| v["labels"][k].as_str().unwrap_or("").to_lowercase();
    let english = |v: &Value| {
        let lang = label(v, "language");
        if !lang.is_empty() {
            return lang.starts_with("en");
        }
        let accent = label(v, "accent");
        accent.is_empty() || ["american", "british", "australian", "irish", "english", "us", "uk"].iter().any(|a| accent.contains(a))
    };
    let of_gender: Vec<&Value> = voices.iter().filter(|v| label(v, "gender") == gender).collect();
    let english_ones: Vec<&&Value> = of_gender.iter().filter(|v| english(v)).collect();
    let young = if gender == "male" { english_ones.iter().find(|v| label(v, "age").contains("young")).copied() } else { None };
    let v = young.or_else(|| english_ones.first().copied()).copied().or_else(|| of_gender.first().copied()).or_else(|| voices.first())?;
    v["voice_id"].as_str().map(String::from)
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

// ── The tools ─────────────────────────────────────────────────────────────────

/// Asks the island (it knows the pills and the services' data) and waits for its answer.
pub fn ask_island(app: &AppHandle, kind: &str, input: Value) -> Option<String> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel();
    pending().lock().unwrap().insert(id, Pending{speech:false,tx});
    let _ = app.emit_to(crate::island::WINDOW_LABEL, "voice-ask", json!({ "id": id, "kind": kind, "input": input }));
    let answer = rx.recv_timeout(Duration::from_secs(5)).ok();
    pending().lock().unwrap().remove(&id);
    answer
}

/// Runs one of Claude's tools: what goes back to Claude, and whether something changed.
pub fn run_tool(app: &AppHandle, name: &str, input: &Value) -> (String, bool) {
    run_tool_for(app,"ai_anthropic",name,input)
}
pub fn run_tool_for(app:&AppHandle,pet:&str,name:&str,input:&Value)->(String,bool){
    let grant=match name{"prepare_email"=>Some("emailDrafts"),"music"=>Some("music"),"open_app"=>Some("openApps"),_=>None};
    if let Some(grant)=grant{if let Err(error)=crate::pet_config::require(pet,grant){return(error,false);}}
    match name {
        "pills" => {
            let answer = ask_island(app, "pills", input.clone()).unwrap_or_else(|| "the island did not answer".into());
            let acted = input["action"].as_str() != Some("list") && !answer.contains("unknown");
            (answer, acted)
        }
        "service_info" => (ask_island(app, "info", json!({})).unwrap_or_else(|| "no data".into()), false),
        "music" => music(app, input),
        "open_app" => open_app(input["name"].as_str().unwrap_or("")),
        "prepare_email" => {
            let to = input["recipient"].as_str().unwrap_or("").trim().to_string();
            if !to.contains('@') {
                return ("No email address: ask the user to spell it.".into(), false);
            }
            let enc = |s: &str| -> String {
                s.bytes()
                    .map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                        _ => format!("%{b:02X}"),
                    })
                    .collect()
            };
            let url = format!(
                "mailto:{}?subject={}&body={}",
                enc(&to).replace("%40", "@"),
                enc(input["subject"].as_str().unwrap_or("")),
                enc(input["body"].as_str().unwrap_or(""))
            );
            crate::platform::open_url(&url);
            (format!("The mail app opened with the email to {to}. Nothing is sent until the user sends it."), true)
        }
        _ => (format!("unknown tool {name}"), false),
    }
}

fn music(app: &AppHandle, input: &Value) -> (String, bool) {
    let action = input["action"].as_str().unwrap_or("");
    let (control, value) = match action {
        "play_pause" => ("playPause", None),
        "pause" => ("pause",None),
        "play" => ("play",None),
        "next" => ("next", None),
        "previous" => ("previous", None),
        "set_volume" => ("volume", Some(input["volume"].as_f64().unwrap_or(50.0).clamp(0.0, 100.0))),
        "play_item"=>{let ok=tauri::async_runtime::block_on(crate::spotify::spotify_play_item(app.clone(),input["uri"].as_str().unwrap_or("").into()));return(if ok{"Spotify accepted the playback request.".into()}else{"Spotify did not accept that item.".into()},ok);},
        "search" => {
            let q = input["query"].as_str().unwrap_or("").trim();
            if q.is_empty() {
                return ("missing query".into(), false);
            }
            let encoded: String = q.chars().map(|c| if c.is_alphanumeric() { c.to_string() } else { "%20".into() }).collect();
            crate::platform::open_url(&format!("spotify:search:{encoded}"));
            return (format!("Spotify opened on a search for {q}: the user picks what to play."), true);
        }
        _ => return ("unknown action".into(), false),
    };
    let app2 = app.clone();
    let done = tauri::async_runtime::block_on(crate::spotify::spotify_control(app2, control.into(), value));
    if done {
        let state=tauri::async_runtime::block_on(crate::spotify::spotify_refresh(app.clone()));
        (state.map(|s|json!({"requestAccepted":true,"playing":s.playing,"volume":s.volume,"track":s.track.map(|t|json!({"title":t.title,"artist":t.artist}))}).to_string()).unwrap_or_else(||"Request accepted; playback state is not confirmed.".into()),true)
    } else {
        ("Spotify is not running or not playing: tell the user to open it.".into(), false)
    }
}

/// Opens an app whose name matches: a Start menu shortcut on Windows, a
/// .desktop entry on Linux. Nothing else is ever run.
fn open_app(name: &str) -> (String, bool) {
    let wanted = wake::normalize(name);
    if wanted.is_empty() {
        return ("no name".into(), false);
    }
    match find_app(&wanted) {
        Some((label, launch)) => {
            launch();
            (format!("{label} opened"), true)
        }
        None => (format!("No app named {name} on this computer."), false),
    }
}

#[cfg(windows)]
fn find_app(wanted: &str) -> Option<(String, Box<dyn FnOnce()>)> {
    let mut roots = Vec::new();
    if let Some(p) = std::env::var_os("APPDATA") {
        roots.push(std::path::PathBuf::from(p).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Some(p) = std::env::var_os("ProgramData") {
        roots.push(std::path::PathBuf::from(p).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    let mut best: Option<(usize, std::path::PathBuf)> = None;
    let mut stack = roots;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")) {
                let stem = wake::normalize(&path.file_stem().unwrap_or_default().to_string_lossy());
                let score = if stem == wanted { 0 } else if stem.starts_with(wanted) { 1 } else if stem.contains(wanted) { 2 } else { continue };
                if best.as_ref().is_none_or(|(s, p)| score < *s || (score == *s && path.as_os_str().len() < p.as_os_str().len())) {
                    best = Some((score, path));
                }
            }
        }
    }
    let (_, path) = best?;
    let label = path.file_stem()?.to_string_lossy().to_string();
    Some((label, Box::new(move || crate::platform::reveal_folder(&path.to_string_lossy()))))
}

#[cfg(target_os = "linux")]
fn find_app(wanted: &str) -> Option<(String, Box<dyn FnOnce()>)> {
    let mut dirs = vec![std::path::PathBuf::from("/usr/share/applications"), std::path::PathBuf::from("/var/lib/flatpak/exports/share/applications")];
    dirs.push(crate::platform::home_dir().join(".local/share/applications"));
    dirs.push(crate::platform::home_dir().join(".local/share/flatpak/exports/share/applications"));
    let mut best: Option<(usize, String, String)> = None;
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().is_none_or(|x| x != "desktop") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            if text.lines().any(|l| l.trim() == "NoDisplay=true") {
                continue;
            }
            let Some(app_name) = text.lines().find_map(|l| l.strip_prefix("Name=")).map(str::trim) else { continue };
            let n = wake::normalize(app_name);
            let score = if n == wanted { 0 } else if n.starts_with(wanted) { 1 } else if n.contains(wanted) { 2 } else { continue };
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if best.as_ref().is_none_or(|(s, _, _)| score < *s) {
                best = Some((score, app_name.to_string(), id));
            }
        }
    }
    let (_, label, id) = best?;
    Some((label, Box::new(move || {
        let _ = std::process::Command::new("gtk-launch").arg(&id).spawn();
    })))
}

#[cfg(test)]
mod tests {
    #[test]
    fn downloading_parakeet_does_not_replace_an_existing_whisper_choice(){
        let mut settings=crate::settings::Settings::default();
        assert_eq!(super::asr_engine(&settings),"parakeet");
        settings.voice_asr_python="/project/python".into();settings.voice_asr_model="/project/model".into();
        assert_eq!(super::asr_engine(&settings),"whisper");
        settings.voice_asr_engine="parakeet".into();assert_eq!(super::asr_engine(&settings),"parakeet");
        assert_eq!(settings.voice_asr_python,"/project/python");assert_eq!(settings.voice_asr_model,"/project/model");
        settings.voice_asr_engine="whisper".into();assert_eq!(super::asr_engine(&settings),"whisper");
    }
    #[test]
    fn incomplete_whisper_setup_does_not_make_auto_unusable(){
        let mut settings=crate::settings::Settings::default();settings.voice_asr_python="/project/python".into();
        assert_eq!(super::asr_engine(&settings),"parakeet");
        settings.voice_asr_engine="whisper".into();assert_eq!(super::asr_engine(&settings),"whisper");
        assert!(!super::whisper_ready(&settings));
    }
    use super::*;

    #[test]
    fn cancelling_audio_does_not_cancel_an_in_flight_mochi_tool_reply() {
        let (voice,heard)=mpsc::channel();let (tool,answer)=mpsc::channel();
        let mut waiting=HashMap::from([(1,Pending{speech:true,tx:voice}),(2,Pending{speech:false,tx:tool})]);
        cancel_playback(&mut waiting);assert_eq!(heard.recv().unwrap(),"");
        assert!(waiting.contains_key(&2));assert!(answer.try_recv().is_err());
        waiting.remove(&2).unwrap().tx.send("service data".into()).unwrap();assert_eq!(answer.recv().unwrap(),"service data");
    }

    #[test]
    fn stopping_does_not_wait_for_the_previous_microphone_owner() {
        let previous = CAPTURE_OWNER.lock().unwrap();
        let stop = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let waiting = scope.spawn(|| capture_lease(&stop).is_none());
            std::thread::sleep(Duration::from_millis(20));
            stop.store(true, Ordering::Relaxed);
            assert!(waiting.join().unwrap());
        });
        drop(previous);
        assert!(capture_lease(&AtomicBool::new(false)).is_some());
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(base64(b""), "");
    }

    #[test]
    fn the_voice_is_picked_like_on_the_mac() {
        let voices: Vec<Value> = serde_json::from_str(r#"[
            {"voice_id":"a","labels":{"gender":"male","accent":"american","age":"middle aged"}},
            {"voice_id":"b","labels":{"gender":"male","accent":"british","age":"young"}},
            {"voice_id":"c","labels":{"gender":"female","accent":"american"}},
            {"voice_id":"d","labels":{"gender":"female","language":"fr"}}
        ]"#).unwrap();
        assert_eq!(pick_voice(&voices, "male").as_deref(), Some("b"));
        assert_eq!(pick_voice(&voices, "female").as_deref(), Some("c"));
        assert_eq!(pick_voice(&[], "female"), None);
    }
}

/// With the models downloaded: `COUCOU_TEST_WAKE_WAV=… cargo test -- --ignored wake_in`.
#[cfg(test)]
mod with_models {
    #[test]
    #[ignore]
    fn hears_the_wake_phrase_in_a_recording() {
        let path = std::env::var("COUCOU_TEST_WAKE_WAV").expect("COUCOU_TEST_WAKE_WAV");
        let bytes = std::fs::read(&path).unwrap();
        let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let pcm: Vec<f32> = bytes[44..].chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).collect();
        let mut r = crate::speech::audio::Resampler::new(rate);
        let mut samples = Vec::new();
        r.process(&pcm, &mut samples);
        let mut vad = super::wake::Vad::default();
        let mut heard = Vec::new();
        for chunk in samples.chunks(1600) {
            for segment in vad.feed(chunk) {
                let text = crate::speech::transcribe(&segment).expect("models").text;
                println!("segment {:.2} s: {text}", segment.len() as f32 / 16_000.0);
                heard.push(super::wake::after_wake(&text));
            }
        }
        println!("{heard:?}");
        assert!(heard.iter().any(|h| h.as_deref().is_some_and(|q| !q.is_empty())));
    }
}

#[cfg(test)]
#[path="vocabulary_tests.rs"]
mod vocabulary_tests;
