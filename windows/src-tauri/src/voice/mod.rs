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

pub(crate) mod admission;
pub mod brain;
pub mod local;
mod local_task;
mod pill_action;
pub mod manual;
pub mod wake;
pub mod dialogue;
mod services;
pub mod session;
mod tts;
mod native;
pub(crate) mod native_surface;
mod native_worker;
mod speech_http;
mod input;

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
fn release_running(running:&mut Option<Running>,stop:&Arc<AtomicBool>){
    if running.as_ref().is_some_and(|r|Arc::ptr_eq(&r.stop,stop)){*running=None;}
}
struct ListeningOwner(Arc<AtomicBool>);
impl Drop for ListeningOwner{fn drop(&mut self){
    self.0.store(true,Ordering::Release);
    release_running(&mut RUNNING.lock().unwrap(),&self.0);
}}
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

struct NativeReply{speech:Arc<native::Speech>,backend:Option<native_worker::Plan>}
struct Pending {speech:bool,tx:mpsc::Sender<String>,native:Option<NativeReply>,target:&'static str}
impl Drop for Pending{fn drop(&mut self){if let Some(reply)=&self.native{reply.speech.retire();}}}
/// Tool replies survive turning speech off while their CLI job continues.
fn pending() -> &'static Mutex<HashMap<u64, Pending>> {
    static P: OnceLock<Mutex<HashMap<u64, Pending>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(HashMap::new()))
}
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn next_id()->Option<u64>{NEXT_ID.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n|(n<9_007_199_254_740_991).then(||n+1)).ok()}
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
    let wanted=wanted&&crate::settings::optional::check(crate::settings::optional::Operation::Voice).is_ok();
    let explicit = session::active() || manual::active();
    let mut running = RUNNING.lock().unwrap();
    match (wanted && !explicit && !session::suspend_hotword() && requirements_met(app), running.is_some()) {
        (true, false) => {
            let stop = Arc::new(AtomicBool::new(false));
            let app2 = app.clone();
            let stop2 = stop.clone();
            *running = Some(Running { stop:stop.clone() });
            if std::thread::Builder::new().name("coucou-voice".into()).spawn(move || run(app2, stop2)).is_err(){
                release_running(&mut running,&stop);return;
            }
            preload_asr(app,stop.clone());
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
    let _owner=ListeningOwner(stop.clone());
    let input=match input::Input::capture(&app){Ok(input)=>input,Err(_)=>return};
    let provider=input.settings.chat_provider.clone();
    let Some(_lease) = capture_lease(&stop) else { return };
    if input.check().is_err() {return;}
    let (tx, rx) = mpsc::sync_channel::<Vec<f32>>(8);
    let capture = audio::Capture::start_stoppable(input.source.clone(), stop.clone(), move |s| {
        let _ = tx.try_send(s.to_vec());
    });
    let mut capture = match capture {
        Ok(c) => c,
        Err(err) => {
            crate::log::line(format!("voice: microphone: {err}"));
            return;
        }
    };
    let mut vad = wake::Vad::default();
    let mut mode = Mode::Wake;
    let mut brain = brain::Brain::default();
    let mut hide_at: Option<Instant> = None;
    while !stop.load(Ordering::Relaxed) {
        if input.check().is_err(){break;}
        let chunk = match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(c) => Some(c),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let now = Instant::now();
        // The pill's last answer goes away after a while.
        if hide_at.is_some_and(|t| now > t) {
            hide_at = None;
            show_for_pet(&app, None, dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
        }
        match &mode {
            Mode::Command { until, .. } | Mode::FollowUp { until } if now > *until => {
                mode = Mode::Wake;
                vad.hangover = 25;
                show_for_pet(&app, None, dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
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
            let Some(heard) = input.transcribe(&segment, Some(&stop)).map(|t| t.text.trim().to_string()) else { continue };
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
                            show_for_pet(&app, Some(VoiceView { phase: "listening".into(), heard: rest.clone(), answer: String::new() }), dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
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
            show_for_pet(&app, Some(VoiceView { phase: "thinking".into(), heard: question.clone(), answer: String::new() }), dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
            brain.forget_if_idle();
            let reply = brain.respond_as(&app, &question, &provider, &input, Some(stop.clone()));
            if stop.load(Ordering::Relaxed) { break; }
            let answer = reply.as_ref().map(|r| r.text.clone()).unwrap_or_default();
            if answer.is_empty() {
                let done = reply.is_some_and(|r| r.acted);
                show_for_pet(&app, Some(VoiceView {
                    phase: "speaking".into(),
                    heard: question,
                    answer: if done { "✓".into() } else { crate::i18n::t("Coucou couldn't answer: check the selected voice provider.") },
                }), dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
                hide_at = Some(Instant::now() + Duration::from_secs(5));
                continue;
            }
            show_for_pet(&app, Some(VoiceView { phase: "speaking".into(), heard: question.clone(), answer: answer.clone() }), dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
            speak_prepared(&app, &answer, Some(&stop), None, None, &input);
            vad.reset();
            // Drop what the microphone heard of Coucou's own voice.
            while rx.try_recv().is_ok() {}
            if answer.trim_end().ends_with('?') {
                mode = Mode::FollowUp { until: Instant::now() + Duration::from_secs(8) };
                vad.hangover = 40;
                show_for_pet(&app, Some(VoiceView { phase: "listening".into(), heard: String::new(), answer }), dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
            } else {
                hide_at = Some(Instant::now() + Duration::from_secs(6));
            }
        }
    }
    capture.stop();
    show_for_pet(&app, None, dialogue::Pet::from_provider(&provider).map(dialogue::Pet::pill));
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
        "" | "auto" if !settings.voice_asr_python.is_empty()&&!settings.voice_asr_model.is_empty()=>"whisper",
        "" | "auto"=>"parakeet",
        _=>"unsupported",
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
    RecognitionStatus{engine,ready:if engine=="whisper"{whisper_available}else{engine=="parakeet"&&crate::speech::is_ready()},whisper_available}
}
pub(crate) fn asr_ready(app: &AppHandle) -> bool {recognition_status(app).ready}
fn preload_asr(app: &AppHandle, stop: std::sync::Arc<AtomicBool>) {
    let settings=match app.state::<crate::Shared>().settings.lock(){Ok(s)=>s.clone(),Err(_)=>return};
    let Ok(configuration)=crate::pet_config::read_existing()else{return};
    let Ok(plan)=input::Recognition::prepare(&settings,&configuration)else{return};
    if stop.load(Ordering::Acquire){return;}
    if matches!(plan.backend(),input::Recognizer::Parakeet){crate::speech::preload();return;}
    std::thread::spawn(move || {let _=plan.transcribe(&[],Some(&stop));});
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

/// Existing dictation callers prepare ASR at transcription time. Voice
/// occurrences use Input captured before audio; this is not their admission.
pub(crate) fn transcribe(app:&AppHandle,samples:&[f32],stop:Option<&AtomicBool>)->Option<crate::speech::Transcript>{
    if stop.is_some_and(|s|s.load(Ordering::Acquire)){return None;}
    let settings=app.state::<crate::Shared>().settings.lock().ok()?.clone();
    let configuration=crate::pet_config::read_existing().ok()?;
    input::Recognition::prepare(&settings,&configuration).ok()?.transcribe(samples,stop)
}

fn cancel_speech(app: &AppHandle) {
    let _ = app.emit_to(pill::LABEL, "voice-cancel", ());
    let _ = app.emit_to(crate::companions::LABEL, "voice-cancel", ());
    cancel_playback(&mut pending().lock().unwrap());
    tts::cancel_native();
}

fn speak_prepared(app:&AppHandle,text:&str,stop:Option<&Arc<AtomicBool>>,provider:Option<&str>,language:Option<&str>,input:&input::Input){
    let captured=input.clone();let stop=stop.cloned();
    let current:tts::Current=Arc::new(move||!stop.as_ref().is_some_and(|s|s.load(Ordering::Acquire))&&captured.check().is_ok());
    speak_plan(app,text,provider,language,&input.playback,current);
}

fn speak_plan(app:&AppHandle,text:&str,provider:Option<&str>,language:Option<&str>,plan:&tts::Plan,valid:tts::Current){
    let epoch=tts::epoch();let diagnosed=AtomicBool::new(false);
    let current:tts::Current=Arc::new(move||{if epoch==u64::MAX||tts::epoch()!=epoch{
        if !diagnosed.swap(true,Ordering::AcqRel){crate::log::line("voice playback retired: generation changed");}return false;
    }valid()});
    if !current()||plan.output=="off"||text.chars().count()>8000{return;}
    if plan.output=="edge"&&tts::speak_streaming(app,plan,text,current.clone()).is_some(){return;}
    let audio=if plan.output=="elevenlabs"{elevenlabs(text,&plan.gender,current.clone()).map(|b|format!("data:audio/mpeg;base64,{}",base64(&b)))}
        else{tts::generate(app,plan,provider,text,current.clone())};
    if !current()||(!plan.system_allowed()&&audio.is_none()){return;}
    let output=&plan.output;let gender=plan.gender.clone();
    let Some(id)=next_id()else{return;};let(tx,rx)=mpsc::channel();
    // WebKit can suspend the hidden pill. The visible pet surface owns audio
    // while companions are enabled, just as it owns the conversation bubble.
    let target = if crate::companions::enabled(app) { crate::companions::LABEL } else { pill::LABEL };
    crate::log::line(format!("voice: playback request id={id}, output={output}, window={target}, audio={}", audio.is_some()));
    let language=language.or_else(||wake::speech_lang(text));
    let words=crate::flow::text::word_count(text)as u64;
    let until=Instant::now()+Duration::from_millis(4000+words.min(200)*600);
    #[cfg(target_os="linux")]
    let speech={
        let Some(window)=app.get_webview_window(target)else{return;};
        let Ok(frame)=tauri::async_runtime::block_on(native_surface::capture(&window))else{return;};
        if frame.surface().label()!=target{return;}
        let Some(speech)=native::Speech::prepare(frame,text,language,until,current.clone())else{return;};Some(speech)
    };
    // Native speech was already unavailable on these platforms. Preserve their
    // browser playback/ACK without claiming Linux physical-window admission.
    #[cfg(not(target_os="linux"))]
    let speech:Option<Arc<native::Speech>>=None;
    let native=speech.as_ref().map(|speech|NativeReply{speech:speech.clone(),backend:if plan.system_allowed(){native_worker::Plan::capture()}else{None}});
    pending().lock().unwrap().insert(id,Pending{speech:true,tx,native,target});
    if app.emit_to(target,"voice-say",Say{id,text:text.into(),lang:language.map(str::to_owned),audio,gender,allow_system_fallback:plan.system_allowed()}).is_err(){pending().lock().unwrap().remove(&id);return;}
    while Instant::now() < until {
        // The controller cancels playback before releasing microphone ownership.
        // A superseded waiter must never send a global cancel to a newer voice.
        if !current()||speech.as_ref().is_some_and(|s|!s.current()){let _=app.emit_to(target,"voice-cancel",json!({"id":id}));break;}
        if rx.recv_timeout(Duration::from_millis(50)).is_ok() { break; }
    }
    pending().lock().unwrap().remove(&id);
}

/// The pill says it has finished speaking (`id`), or the island answers a question.
#[tauri::command]
pub async fn voice_done(window:tauri::WebviewWindow,id:u64,text:Option<String>,played:Option<bool>){
    let scope=pending().lock().unwrap().get(&id).map(|p|(p.speech,p.target,p.native.as_ref().map(|n|n.speech.clone())));
    let Some((speech,target,reply))=scope else{return;};
    if speech{
        #[cfg(target_os="linux")]{let(Some(reply),Ok(frame))=(reply,native_surface::capture(&window).await)else{return;};if !reply.belongs_to(&frame){return;}}
        #[cfg(not(target_os="linux"))]{let _=reply;if window.label()!=target{return;}}
        #[cfg(target_os="linux")]let _=target;
    }else if window.label()!=crate::island::WINDOW_LABEL{return;}
    if let Some(played)=played{crate::log::line(format!("voice: renderer playback complete id={id}, played={played}"));}
    if let Some(p)=pending().lock().unwrap().remove(&id){let _=p.tx.send(text.unwrap_or_default());}
}

/// A renderer requests only a backend-owned reply. Payload text/language,
/// another surface, a tool id, replay or an obsolete document cannot dispatch.
#[tauri::command]
pub async fn voice_say_native(window:tauri::WebviewWindow,id:Option<u64>)->bool{
    let Some(id)=id.filter(|n|*n>0&&*n<=9_007_199_254_740_991)else{return false;};
    let selected={let pending=pending().lock().unwrap();pending.get(&id).filter(|p|p.speech)
        .and_then(|p|p.native.as_ref()).map(|n|(n.speech.clone(),n.backend.clone()))};
    let Some((speech,backend))=selected else{return false;};
    let Ok(frame)=native_surface::capture(&window).await else{speech.event(0);return false;};
    let Some(backend)=backend else{speech.event(1);return false;};
    let Some(call)=speech.claim(&frame)else{speech.event(2);return false;};
    speech.event(3);let result=native_worker::run(call,backend).await;if !result{speech.event(4);}result
}

#[tauri::command]
pub async fn voice_preview(app: AppHandle, provider: Option<String>) {
    if session::active() || pill::dictating(&app) || crate::notes::is_recording(&app) { return; }
    let Ok((plan,current))=tts::preview(&app)else{return;};
    cancel_speech(&app);
    // Own the preview generation before queuing: a delayed old worker must
    // not borrow the newer preview's epoch when it eventually starts.
    let preview_epoch=tts::epoch();
    let current:tts::Current=Arc::new(move||tts::epoch()==preview_epoch&&current());
    tauri::async_runtime::spawn_blocking(move || {
        let raquel = plan.output=="edge";
        let sample = if raquel { "Olá! Esta é a minha voz para as nossas conversas.".into() }
            else { crate::i18n::t("Hello! This is my voice for our conversations.") };
        speak_plan(&app, &sample, provider.as_deref(), None, &plan, current);
    }).await.ok();
}

fn elevenlabs(text:&str,gender:&str,current:tts::Current)->Option<Vec<u8>>{
    speech_http::elevenlabs(text,gender,current)
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
    let id = next_id()?;
    let (tx, rx) = mpsc::channel();
    pending().lock().unwrap().insert(id, Pending{speech:false,tx,native:None,target:crate::island::WINDOW_LABEL});
    let _ = app.emit_to(crate::island::WINDOW_LABEL, "voice-ask", json!({ "id": id, "kind": kind, "input": input }));
    let answer = rx.recv_timeout(Duration::from_secs(5)).ok();
    pending().lock().unwrap().remove(&id);
    answer
}

/// Runs one of Claude's tools: what goes back to Claude, and whether something changed.
pub fn run_tool_for(app:&AppHandle,pet:&str,name:&str,input:&Value,current:impl Fn()->bool)->(String,bool){
    if let Err(error)=crate::settings::effect_allowed(){return(error,false);}
    crate::pet_operations::run(pet,name,input,current,crate::pet_config::read_existing,|call|run_native_tool(app,call)).legacy()
}
pub(crate) fn run_native_tool_for(app:&AppHandle,pet:&str,name:&str,input:&Value,native:&crate::pet_native_invocation::Fence,current:impl Fn()->bool)->(String,bool){
    if let Err(error)=crate::settings::effect_allowed(){return(error,false);}
    crate::pet_operations::run_native(native,pet,name,input,current,crate::pet_config::read_existing,|call|run_native_tool(app,call)).legacy()
}

fn run_native_tool(app:&AppHandle,call:&crate::pet_operations::Call)->crate::pet_operations::NativeOutcome {
    use crate::pet_operations::{Arguments,Delivery as D,NativeOutcome as O};
    match call.arguments() {
        Arguments::Roster(input)=>{
            let mut answered=false;
            let (text,accepted)=pill_action::run(input,|request|{let answer=ask_island(app,"pills",request.clone());answered=answer.is_some();answer});
            let delivery=if !answered{D::Uncertain}else if input["action"]=="list"{D::Returned}else if accepted{D::Accepted}else{D::Partial};
            O::new(delivery,text)
        },
        Arguments::Snapshot=>match ask_island(app,"info",json!({})){
            Some(text)=>O::new(D::Returned,text),
            None=>O::new(D::Uncertain,"A ilha não respondeu; os dados não foram confirmados."),
        },
        Arguments::Music(request)=>music(app,request),
        Arguments::OpenApp(name)=>open_app(name),
        Arguments::Email(email)=>{
            crate::platform::open_url(&email.uri());
            O::new(D::Dispatched,format!("Foi pedido à aplicação de correio um rascunho para {}. A abertura não foi observada; este pedido não envia o email.",email.recipient))
        },
    }
}
fn music(app:&AppHandle,request:&crate::pet_operations::Music)->crate::pet_operations::NativeOutcome {
    use crate::pet_operations::{Music,Delivery as D,NativeOutcome as O};
    let (control,value)=match request{
        Music::PlayPause=>("playPause",None), Music::Pause=>("pause",None), Music::Play=>("play",None),
        Music::Next=>("next",None), Music::Previous=>("previous",None),
        Music::SetVolume{volume}=>("volume",Some(f64::from(*volume))),
        Music::PlayItem{uri}=>{
            let accepted=tauri::async_runtime::block_on(crate::spotify::spotify_play_item(app.clone(),uri.clone()));
            return if accepted{O::new(D::Accepted,"O Spotify aceitou o pedido de reprodução; o efeito ainda não foi verificado.")}
                else{O::new(D::Uncertain,"O pedido não foi confirmado pelo Spotify. Verifica o estado antes de o repetir.")};
        },
        Music::Search{query}=>{
            crate::platform::open_url(&format!("spotify:search:{}",crate::pet_operations::encode(query)));
            return O::new(D::Dispatched,format!("Foi pedida a abertura da pesquisa Spotify por {query:?}; a abertura não foi observada e não foi pedido o início de uma faixa."));
        },
    };
    let accepted=tauri::async_runtime::block_on(crate::spotify::spotify_control(app.clone(),control.into(),value));
    if accepted{
        let state=tauri::async_runtime::block_on(crate::spotify::spotify_refresh(app.clone()));
        O::new(D::Accepted,state.map(|s|json!({"requestAccepted":true,"playing":s.playing,"volume":s.volume,"track":s.track.map(|t|json!({"title":t.title,"artist":t.artist}))}).to_string()).unwrap_or_else(||"Pedido aceite; o estado de reprodução não foi confirmado.".into()))
    }else{O::new(D::Uncertain,"O controlo Spotify não foi confirmado. Verifica o estado antes de repetir o pedido.")}
}

/// The launcher request is not proof that a window was opened.
fn open_app(name:&str)->crate::pet_operations::NativeOutcome{
    use crate::pet_operations::{Delivery as D,NativeOutcome as O};
    let wanted=wake::normalize(name);
    match find_app(&wanted){
        Some((label,launch))=>{launch();O::new(D::Dispatched,format!("Foi pedido o arranque de {label}; a abertura da janela não foi observada."))},
        None=>O::new(D::NotDispatched,format!("Não foi encontrada uma aplicação com o nome {name:?}.")),
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
        let mut waiting=HashMap::from([(1,Pending{speech:true,tx:voice,native:None,target:crate::flow::pill::LABEL}),(2,Pending{speech:false,tx:tool,native:None,target:crate::island::WINDOW_LABEL})]);
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
