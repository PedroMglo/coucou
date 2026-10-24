//! Explicit, continuous conversations. Capture is owned only while listening;
//! no audio device remains open during transcription, tools or spoken replies.
use super::{
    dialogue::{self, Command, Idle, Pet},
    VoiceView,
};
use crate::{
    flow::pill,
    speech::audio,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

static SESSION: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
static FINISH: AtomicBool = AtomicBool::new(false);
// An explicit session takes over from the hotword. After Super/timeout, the
// microphone stays closed until a new session or an explicit hotword preference.
static HOTWORD_SUSPENDED: AtomicBool = AtomicBool::new(false);
pub fn active() -> bool {
    SESSION.lock().unwrap().is_some()
}
pub fn suspend_hotword() -> bool {
    HOTWORD_SUSPENDED.load(Ordering::Relaxed)
}
pub fn allow_hotword() {
    HOTWORD_SUSPENDED.store(false, Ordering::Relaxed);
}

fn reserve(app: &AppHandle) -> Result<Arc<AtomicBool>, String> {
    if super::manual::active() { return Err(crate::i18n::t("The microphone is already in use.")); }
    let mut session = SESSION.lock().unwrap();
    if session.is_some() || pill::dictating(app) || crate::notes::is_recording(app) {
        return Err(crate::i18n::t("The microphone is already in use."));
    }
    if !super::asr_ready(app) {
        return Err(crate::i18n::t(
            "Download the voice model so Coucou can hear you.",
        ));
    }
    let stop = Arc::new(AtomicBool::new(false));
    *session = Some(stop.clone());
    HOTWORD_SUSPENDED.store(true, Ordering::Relaxed);
    if let Some(background) = super::RUNNING.lock().unwrap().take() {
        background.stop.store(true, Ordering::Relaxed);
    }
    super::cancel_speech(app);
    super::preload_asr(app, stop.clone());
    Ok(stop)
}

fn owns(session:&Option<Arc<AtomicBool>>,stop:&Arc<AtomicBool>)->bool {
    session.as_ref().is_some_and(|s|Arc::ptr_eq(s,stop))
}
fn detach(session:&mut Option<Arc<AtomicBool>>)->bool {
    let Some(stop)=session.take() else{return false};
    stop.store(true,Ordering::Relaxed);true
}
fn detach_owned(session:&mut Option<Arc<AtomicBool>>,expected:&Arc<AtomicBool>)->bool {
    if !owns(session,expected) {return false;}
    detach(session)
}
fn finish(app: &AppHandle, stop: &Arc<AtomicBool>)->bool {
    let mut session = SESSION.lock().unwrap();
    if owns(&session,stop) {
        stop.store(true,Ordering::Relaxed);
        crate::speech::whisper::release_if_idle();
        *session=None;
        super::show(app, None);
        return true;
    }
    false
}
fn stop_audio(app:&AppHandle)->bool {
    stop_audio_for(app,None)
}
fn stop_audio_for(app:&AppHandle,expected:Option<&Arc<AtomicBool>>)->bool {
    let mut session=SESSION.lock().unwrap();
    let detached=match expected {Some(owner)=>detach_owned(&mut session,owner),None=>detach(&mut session)};
    if !detached {return false;}
    crate::speech::whisper::release_if_idle();
    super::cancel_speech(app);super::show(app,None);
    true
}
fn select_pet(app:&AppHandle,stop:&Arc<AtomicBool>,pet:Pet) {
    let session=SESSION.lock().unwrap();
    if owns(&session,stop) && !stop.load(Ordering::Relaxed) {
        let _=app.emit_to(crate::island::WINDOW_LABEL,"voice-pet",pet.provider());
    }
}
fn work_token(pet:Pet,voice:&Arc<AtomicBool>)->Arc<AtomicBool> {
    if crate::cli_chat::backend(pet.provider()).is_some() {Arc::new(AtomicBool::new(false))} else {voice.clone()}
}
fn cancel_pet_task(chat:&crate::chat::Chat,pet:Pet)->bool {
    let task=chat.tasks().into_iter().find(|t|t.provider==pet.provider() && !t.terminal());
    task.is_some_and(|t|chat.cancel_task(&t.id))
}
fn task_status(chat:&crate::chat::Chat,pet:Pet)->String {
    let tasks=chat.tasks();
    let task=tasks.iter().filter(|t|t.provider==pet.provider()).max_by_key(|t|(!t.terminal(),t.started));
    let Some(task)=task else{return format!("Não vejo uma tarefa do {} no Mochi.",pet.name())};
    let state=match task.phase.as_str() {
        "approval"=>"precisa da tua aprovação no cartão",
        "question"=>"precisa de uma resposta tua no cartão",
        "stopping"=>"está a interromper a tarefa",
        "done"=>"já tem uma resposta disponível",
        "failed"=>"não conseguiu terminar a tarefa",
        "stopped"|"interrupted"=>"tem uma tarefa interrompida que podes verificar no centro de tarefas",
        "resumed"=>"retomou a tarefa",
        _=>"continua a trabalhar na tarefa",
    };
    let detail=if task.phase=="done" {&task.result} else {&task.progress};
    let detail=super::brain::spoken_text(detail).chars().take(180).collect::<String>();
    if detail.is_empty() {format!("O {} {state}.",pet.name())} else {format!("O {} {state}. {detail}",pet.name())}
}

#[tauri::command]
pub fn voice_session_toggle(app: AppHandle) {
    toggle(&app);
}

/// A new pet gets its microphone now, while old CLI work keeps its own token.
#[tauri::command]
pub fn voice_session_start(app:AppHandle,provider:String)->Result<(),String> {
    let pet=Pet::from_provider(&provider).ok_or("Este pet não tem conversa por voz.")?;
    admit_pet(&app,pet).map_err(|e|e.message().to_owned())?;
    let previous=SESSION.lock().unwrap().clone();
    if let Some(previous)=previous {stop_audio_for(&app,Some(&previous));}
    begin(&app,Some(pet))
}

pub fn toggle(app: &AppHandle) {
    if stop_audio(app) {return;}
    if super::manual::active() { super::manual::cancel(app); return; }
    if pill::dictating(app) {
        crate::flow::voice_dictation(app, false);
        return;
    }
    if let Err(error)=begin(app,None) {pill::announce(app,"mic","#FFB84D",error,None,6);}
}

fn begin(app:&AppHandle,selected:Option<Pet>)->Result<(),String> {
    let configuration=crate::pet_config::pet_configuration()?;
    let _=app.emit("pet-configuration-changed",configuration.config);
    let stop=reserve(app)?;
    crate::pet_panel::hide(app);
    let (worker_app, worker_stop) = (app.clone(), stop.clone());
    if std::thread::Builder::new()
        .name("coucou-dialogue".into())
        .spawn(move || {
            let dictate = run(&worker_app, &worker_stop,selected);
            let handoff=dictate && !worker_stop.load(Ordering::Relaxed);
            let closed=finish(&worker_app, &worker_stop);
            if handoff && closed {
                crate::flow::voice_dictation(&worker_app, true);
            }
        })
        .is_err()
    {
        finish(app, &stop);
        return Err("Não foi possível iniciar a conversa por voz.".into());
    }
    Ok(())
}

pub fn cancel(app: &AppHandle) {
    stop_audio(app);
}

pub fn finish_capture() { if active() { FINISH.store(true, Ordering::Relaxed); } }

fn view(app: &AppHandle, stop:&Arc<AtomicBool>, phase: &str, heard: &str, answer: &str, pet: Option<Pet>) {
    let session=SESSION.lock().unwrap();
    if !owns(&session,stop) || stop.load(Ordering::Relaxed) {return;}
    super::show_for_pet(
        app,
        Some(VoiceView {
            phase: phase.into(),
            heard: heard.into(),
            answer: answer.into(),
        }),
        pet.map(Pet::pill),
    );
}

fn admit_pet(app:&AppHandle,pet:Pet)->Result<(),super::admission::Failure>{
    use super::admission::{self,Route,Failure};
    let descriptor=crate::pet_capabilities::provider_descriptor(app,pet.provider()).map_err(|_|Failure::StateUnavailable)?;
    admission::admit(pet.provider(),&descriptor,|candidate|match candidate.route{
        Route::Codex=>crate::codex_chat::installed(),
        Route::Copilot=>crate::copilot_chat::executable().is_ok(),
        Route::Gravity=>crate::gravity_chat::executable().is_ok(),
        Route::Spotify=>true,
        Route::Ollama=>{
            let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();
            crate::pet_config::read_existing().ok().is_some_and(|config|admission::local_ready(candidate,&settings,&config))
        },
    }).map(|_|())
}

/// Returns one speech segment or None on stop/inactivity. Bounded queues and
/// VAD bound the audio retained in memory. No recording is saved on disk.
fn listen(app: &AppHandle, stop: &Arc<AtomicBool>, idle: &mut Idle, ready: impl FnOnce()) -> Result<Option<Vec<f32>>, String> {
    let mut ready=Some(ready);
    let Some(_lease) = super::capture_lease(stop) else {
        return Ok(None);
    };
    let mic = crate::flow::get(app)
        .map(|f| f.store.lock().unwrap().prefs.mic.clone())
        .unwrap_or_default();
    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(8);
    let mut capture = audio::Capture::start(
        audio::Source::Mic((!mic.is_empty()).then_some(mic)),
        move |data| {
            let _ = tx.try_send(data.to_vec());
        },
    )
    .map_err(|_| {
        crate::i18n::t("Coucou can't hear you: the microphone is not allowed or not available.")
    })?;
    let mut vad = super::wake::Vad::default();
    vad.hangover = 38; // 1.14 s: retain clauses across natural Portuguese pauses.
    FINISH.store(false, Ordering::Relaxed);
    let mut meter = audio::BandMeter::default();
    let segment = loop {
        if stop.load(Ordering::Relaxed)
            || idle.expired(Instant::now())
            || pill::dictating(app)
            || crate::notes::is_recording(app)
        {
            break None;
        }
        if FINISH.swap(false, Ordering::Relaxed) {
            if let Some(segment) = vad.feed(&vec![0.0; vad.hangover * super::wake::FRAME]).into_iter().next() { break Some(segment); }
        }
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(chunk) => {
                if !chunk.is_empty() { if let Some(ready)=ready.take() { ready(); } }
                let segments = vad.feed(&chunk);
                if vad.in_speech() {
                    idle.touch(Instant::now());
                }
                meter.feed(&chunk);
                let _ = app.emit_to(pill::LABEL, "pill-bands", meter.levels);
                if let Some(segment) = segments.into_iter().next() {
                    break Some(segment);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break None,
        }
    };
    capture.stop();
    Ok(segment)
}

fn run(app: &AppHandle, stop: &Arc<AtomicBool>, initial:Option<Pet>) -> bool {
    let seconds = app
        .state::<crate::Shared>()
        .settings
        .lock()
        .unwrap()
        .voice_idle_seconds;
    let mut idle = Idle::new(Instant::now(), seconds);
    let mut selected = initial;
    if let Some(pet)=selected {select_pet(app,stop,pet);}
    let mut brains = HashMap::<Pet, super::brain::Brain>::new();
    let mut last_heard=String::new();
    let mut answer = crate::i18n::t("Say a pet name, such as Codex or Gravity, or say dictation.");
    while !stop.load(Ordering::Relaxed) {
        view(app, stop, "preparing", if selected.is_none(){&last_heard}else{""}, &answer, selected);
        let samples = match listen(app, stop, &mut idle, || view(app,stop,"listening",if selected.is_none(){&last_heard}else{""},&answer,selected)) {
            Ok(Some(samples)) => samples,
            Ok(None) => break,
            Err(error) => {
                pill::announce(app, "mic", "#FF6B6B", error, None, 7);
                break;
            }
        };
        if stop.load(Ordering::Relaxed) {
            break;
        }
        view(app, stop, "transcribing", "", "", selected);
        let transcript = super::transcribe(app, &samples, Some(stop)).map(|t|t.text).unwrap_or_default();
        if transcript.trim().is_empty() {
            answer = "Não consegui perceber. Podes repetir?".into();
            idle.touch(Instant::now()); continue;
        }
        if stop.load(Ordering::Relaxed) {
            break;
        }
        last_heard=transcript.clone();
        let mut aliases = app
            .state::<crate::Shared>()
            .settings
            .lock()
            .unwrap()
            .voice_pet_aliases
            .clone();
        if let Ok(configured)=crate::pet_config::aliases(){for(provider,names)in configured{aliases.entry(provider).or_default().extend(names);}}
        let command=dialogue::parse_with_aliases(&transcript,&aliases);
        if let Some(name)=(!super::admission::known_voice_address(&command)).then(||crate::pet_config::read_existing().ok().and_then(|c|c.unavailable_address(&transcript))).flatten() {
            answer=format!("{name} ainda não tem conversa de voz integrada. Escolhe Codex, Gravity ou outro pet com conversa disponível.");
            view(app,stop,"speaking",&transcript,&answer,selected);super::speak_stoppable(app,&answer,Some(stop),None);
            idle.touch(Instant::now());continue;
        }
        let question = match command {
            Command::Stop => break,
            Command::Dictate => return true,
            Command::CancelTask => "interromper tarefa".into(),
            Command::TaskStatus => "estado da tarefa".into(),
            Command::Select(pet, question) => {
                if super::admission::uses_pet_response(&question){if let Err(reason)=admit_pet(app, pet) {
                    answer = reason.message().into();
                    view(app,stop,"speaking",&transcript,&answer,selected);super::speak_stoppable(app,&answer,Some(stop),None);
                    idle.touch(Instant::now());
                    continue;
                }}
                selected = Some(pet);
                select_pet(app,stop,pet);
                if question.is_empty() {
                    answer = pet.name().into();
                    idle.touch(Instant::now());
                    continue;
                }
                question
            }
            Command::Message(question) => question,
        };
        let Some(pet) = selected else {
            answer="Não reconheci o nome do pet. Diz, por exemplo, Codex, Gravity ou Spotify, seguido do pedido.".into();
            view(app,stop,"speaking",&transcript,&answer,None);super::speak_stoppable(app,&answer,Some(stop),None);
            idle.touch(Instant::now());
            continue;
        };
        if question.is_empty() {
            idle.touch(Instant::now());
            continue;
        }
        match dialogue::parse(&question) {
            Command::Stop=>break,Command::Dictate=>return true,
            Command::CancelTask|Command::TaskStatus=>{
                if matches!(dialogue::parse(&question),Command::TaskStatus) {
                    answer=task_status(&app.state::<crate::chat::Chat>(),pet);
                } else {
                let cancelled=cancel_pet_task(&app.state::<crate::chat::Chat>(),pet);
                if cancelled {let _=app.emit("assistant-tasks-changed",());}
                answer=if cancelled {format!("Pedi ao {} para interromper a tarefa.",pet.name())} else {format!("Não enviei nenhuma interrupção ao {}. Podes confirmar a tarefa no centro de tarefas.",pet.name())};
                }
                view(app,stop,"speaking",&question,&answer,Some(pet));
                super::speak_stoppable(app,&answer,Some(stop),Some(pet.provider()));
                idle.touch(Instant::now());continue;
            },_=>{},
        }
        #[cfg(target_os="linux")]if let Some(text)=crate::cli_chat::goal_recipe_voice(app,pet.provider(),&question,stop){
            answer=text;view(app,stop,"speaking",&question,&answer,Some(pet));super::speak_stoppable(app,&answer,Some(stop),Some(pet.provider()));idle.touch(Instant::now());continue;
        }
        if let Err(reason)=admit_pet(app,pet){
            answer=reason.message().into();
            view(app,stop,"speaking",&question,&answer,Some(pet));
            super::speak_stoppable(app,&answer,Some(stop),Some(pet.provider()));
            idle.touch(Instant::now());continue;
        }
        view(app, stop, "received", &question, "", Some(pet));
        let direct=if pet==Pet::Spotify {super::services::spotify(app,&question)}else{None};
        let optional_assistant=direct.is_none()&&pet==Pet::Spotify&&crate::pet_config::read_existing().is_ok_and(|c|c.uses_optional_assistant(pet.pill()));
        let reply = if let Some(reply)=direct {Some(reply)}else if pet==Pet::Spotify&&!optional_assistant {Some(super::brain::Reply{text:"Posso controlar a música e o som do PC. Diz, por exemplo, pausa, próxima música, volume do sistema 40 ou silenciar microfone.".into(),acted:false,language:Some("pt-PT".into())})}else{brains.entry(pet).or_default().respond_as(
            app,
            &question,
            pet.provider(),
            Some(work_token(pet,stop)),
        )};
        if stop.load(Ordering::Relaxed) {
            break;
        }
        answer = reply
            .map(|r| r.text)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| crate::i18n::t("Connect this pet in Settings → Chat first."));
        view(
            app,
            stop,
            "speaking",
            &format!("{} · {question}", pet.name()),
            &answer,
            Some(pet),
        );
        super::speak_stoppable(app, &answer, Some(stop), Some(pet.provider()));
        idle.touch(Instant::now());
    }
    false
}

/// Learn what local ASR writes for the user's pronunciation. Only the spelling
/// is persisted; microphone samples and the user's voice are never retained.
#[tauri::command]
pub async fn voice_learn_alias(app: AppHandle, provider: String) -> Result<String, String> {
    let pet = Pet::from_provider(&provider).ok_or_else(|| "Unknown pet.".to_string())?;
    let stop = reserve(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        view(&app, &stop, "preparing", pet.name(), &crate::i18n::t("Say only this pet's name."), Some(pet));
        let result = (|| {
            let heard = listen(&app, &stop, &mut Idle::new(Instant::now(), 20), || view(&app,&stop,"listening",pet.name(),&crate::i18n::t("Say only this pet's name."),Some(pet)))?.ok_or_else(|| crate::i18n::t("No speech detected. Check your microphone."))?;
            let spelling = super::transcribe(&app, &heard, Some(&stop)).map(|t| super::wake::normalize(&t.text)).filter(|s| !s.is_empty() && s.len() <= 80)
                .ok_or_else(|| crate::i18n::t("No speech detected. Check your microphone."))?;
            if stop.load(Ordering::Relaxed) { return Err(crate::i18n::t("Cancelled")); }
            let shared = app.state::<crate::Shared>();
            let mut settings = shared.settings.lock().unwrap();
            let mut candidate = settings.voice_pet_aliases.clone();
            if let Ok(configured)=crate::pet_config::aliases(){for(provider,names)in configured{candidate.entry(provider).or_default().extend(names);}}
            candidate.entry(provider.clone()).or_default().push(spelling.clone());
            if !matches!(dialogue::parse_with_aliases(&spelling, &candidate), Command::Select(selected, ref question) if selected == pet && question.is_empty()) {
                return Err(crate::i18n::t("This name already selects another pet. Try a different name."));
            }
            let aliases = settings.voice_pet_aliases.entry(provider.clone()).or_default();
            if !aliases.contains(&spelling) {
                if aliases.len() >= 20 { aliases.remove(0); }
                aliases.push(spelling.clone());
            }
            crate::settings::save(&settings).map_err(|e| e.to_string())?;
            let _ = app.emit("settings-changed", settings.clone());
            Ok(spelling)
        })();
        finish(&app, &stop);
        result
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spoken_status_reads_only_selected_owned_work_without_starting_a_model_turn() {
        let chat=crate::chat::Chat::default();let voice=Arc::new(AtomicBool::new(false));
        let turn=chat.begin_cli("gemini-cli",work_token(Pet::Gemini,&voice)).unwrap();let _lease=chat.lease(&turn);
        chat.start_task(&turn,"prepara postgres","/tmp","antigravity","gemini-model",None);
        chat.task_update(&turn,"approval","");assert!(task_status(&chat,Pet::Gemini).contains("aprovação no cartão"));
        chat.task_update(&turn,"question","");assert!(task_status(&chat,Pet::Gemini).contains("resposta tua"));
        chat.task_update(&turn,"working","A verificar a base de dados.");assert!(task_status(&chat,Pet::Gemini).contains("A verificar"));
        chat.task_update(&turn,"done","Resposta pronta.");let done=task_status(&chat,Pet::Gemini);assert!(done.contains("resposta disponível"));assert!(!done.contains("concluída"));
        assert_eq!(task_status(&chat,Pet::Codex),"Não vejo uma tarefa do Codex no Mochi.");
        assert!(chat.is_current(&turn));assert!(!voice.load(Ordering::Relaxed));
    }
    #[test]
    fn switching_audio_off_keeps_owned_cli_work_and_a_new_microphone_session_independent() {
        let voice=Arc::new(AtomicBool::new(false));let mut slot=Some(voice.clone());
        let job=work_token(Pet::Codex,&voice);
        let chat=crate::chat::Chat::default();let turn=chat.begin_cli("codex",job.clone()).unwrap();let _lease=chat.lease(&turn);
        chat.start_task(&turn,"prepara postgres","/tmp","codex","model",None);
        assert!(detach(&mut slot));assert!(voice.load(Ordering::Relaxed));
        assert!(!job.load(Ordering::Relaxed));assert!(chat.is_current(&turn));
        assert!(chat.reset_provider("codex").is_err());
        let next=Arc::new(AtomicBool::new(false));slot=Some(next.clone());
        assert!(!owns(&slot,&voice));assert!(owns(&slot,&next));
        assert!(cancel_pet_task(&chat,Pet::Codex));assert!(job.load(Ordering::Relaxed));
        assert!(!next.load(Ordering::Relaxed));assert!(owns(&slot,&next));
    }
    #[test]
    fn every_cli_pet_owns_its_work_stop_and_local_chat_keeps_its_existing_stop() {
        let voice=Arc::new(AtomicBool::new(false));
        for pet in [Pet::Codex,Pet::Gravity,Pet::Gemini,Pet::Claude] {
            let job=work_token(pet,&voice);assert!(!Arc::ptr_eq(&voice,&job));
            job.store(true,Ordering::Relaxed);assert!(!voice.load(Ordering::Relaxed));
        }
        assert!(Arc::ptr_eq(&voice,&work_token(Pet::Ollama,&voice)));
    }
    #[test]
    fn a_spoken_task_stop_affects_only_the_selected_pets_current_owned_task() {
        let chat=crate::chat::Chat::default();let voice=Arc::new(AtomicBool::new(false));
        let codex=chat.begin_cli("codex",work_token(Pet::Codex,&voice)).unwrap();
        let gemini=chat.begin_cli("gemini-cli",work_token(Pet::Gemini,&voice)).unwrap();
        let _a=chat.lease(&codex);let _b=chat.lease(&gemini);
        chat.start_task(&codex,"codex","/tmp","codex","model",None);
        chat.start_task(&gemini,"gemini","/tmp","antigravity","gemini-model",None);
        assert!(cancel_pet_task(&chat,Pet::Gemini));assert!(!chat.is_current(&gemini));
        assert!(chat.is_current(&codex));assert!(!voice.load(Ordering::Relaxed));
        assert!(!cancel_pet_task(&chat,Pet::Gravity));assert!(!cancel_pet_task(&chat,Pet::Ollama));
        assert_eq!(chat.tasks().iter().find(|t|t.provider=="gemini-cli").unwrap().phase,"stopping");
    }
    #[test]
    fn stopping_an_already_closed_audio_session_cannot_claim_a_new_owner() {
        let old=Arc::new(AtomicBool::new(false));let mut slot=Some(old.clone());
        assert!(detach(&mut slot));assert!(!detach(&mut slot));assert!(!owns(&slot,&old));
        let next=Arc::new(AtomicBool::new(false));slot=Some(next.clone());
        assert!(!owns(&slot,&old));assert!(!detach_owned(&mut slot,&old));assert!(owns(&slot,&next));assert!(!next.load(Ordering::Relaxed));
    }
}
