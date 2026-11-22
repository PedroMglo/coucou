use super::*;

#[cfg(target_os="linux")]
fn roster_input(f:&crate::settings::EffectFixture)->Input{
    f.update(|s|s.active_integrations.push("ai_ollama".into()));
    let owner=f.current.lock().unwrap();
    Input::prepare_with_origin(owner.clone(),crate::pet_config::defaults(),String::new(),std::sync::Arc::new(crate::settings::capture_origin(&owner))).unwrap()
}

#[cfg(target_os="linux")]
#[test]
fn own_voice_roster_cas_advances_only_its_turn_and_preserves_frozen_inputs(){
    let f=crate::settings::EffectFixture::new();let input=roster_input(&f);let playback=input.clone();
    let frozen=serde_json::to_value(&input.settings).unwrap();let old=crate::settings::capture_origin(&f.current.lock().unwrap());
    input.change_roster_with(&f.current,"ai_ollama",crate::pet_roster_settings::Request::SetMain{id:"agent_codex".into()},
        ||f.current.try_lock().is_ok(),|next|{assert!(f.current.try_lock().is_err());f.persist(next)},|_|{}).unwrap();
    assert_eq!(f.current.lock().unwrap().main_pill,"agent_codex");assert!(old.operation("voice").is_err());
    assert!(input.source_current().is_ok());assert!(playback.source_current().is_ok());
    assert_eq!(serde_json::to_value(&input.settings).unwrap(),frozen);
    f.save();assert!(input.source_current().is_err());assert!(playback.source_current().is_err());
}

#[cfg(target_os="linux")]
#[test]
fn unrelated_commit_replaced_owner_and_external_edit_cannot_borrow_roster_continuation(){
    for kind in 0..3{
        let f=crate::settings::EffectFixture::new();let input=roster_input(&f);
        match kind{0=>f.save(),1=>*f.current.lock().unwrap()=crate::settings::load_from(&f.path),_=>f.external()}
        let bytes=std::fs::read(&f.path).unwrap();
        assert!(input.change_roster_with(&f.current,"ai_ollama",crate::pet_roster_settings::Request::SetMain{id:"agent_codex".into()},||true,|_|panic!("retired origin reached persistence"),|_|panic!("retired origin published")).is_err());
        assert_eq!(std::fs::read(&f.path).unwrap(),bytes);assert!(input.source_current().is_err());
    }
}

#[cfg(target_os="linux")]
#[test]
fn cancelled_invalid_and_failed_voice_roster_changes_never_mint_a_new_origin(){
    for kind in 0..3{
        let f=crate::settings::EffectFixture::new();let input=roster_input(&f);let bytes=std::fs::read(&f.path).unwrap();
        let id=if kind==1{"integration_github"}else{"agent_codex"};
        assert!(input.change_roster_with(&f.current,"ai_ollama",crate::pet_roster_settings::Request::SetMain{id:id.into()},||kind!=0,|_|{
            assert_eq!(kind,2);Err(std::io::Error::other("synthetic disk failure"))
        },|_|panic!("failed change published")).is_err());
        assert_eq!(std::fs::read(&f.path).unwrap(),bytes);assert!(input.roster_origin.lock().unwrap().is_none());assert!(input.source_current().is_ok());
    }
}

#[cfg(target_os="linux")]
#[test]
fn removing_the_voice_pet_commits_without_authorizing_further_voice_io(){
    let f=crate::settings::EffectFixture::new();let input=roster_input(&f);
    input.change_roster_with(&f.current,"ai_ollama",crate::pet_roster_settings::Request::SetActive{id:"ai_ollama".into(),enabled:false},||true,|next|f.persist(next),|_|{}).unwrap();
    assert!(!f.current.lock().unwrap().active_integrations.contains(&"ai_ollama".into()));
    assert!(input.roster_origin.lock().unwrap().is_none());assert!(input.source_current().is_err());
}

fn configured() -> Settings {
    let mut s=Settings::default();s.chat_provider="codex".into();
    s.voice_asr_engine="whisper".into();s.voice_asr_python="/fixture/python".into();s.voice_asr_model="/fixture/whisper".into();
    s.ollama_url="http://127.0.0.1:11434".into();s.cli_workspace="/fixture/original-workspace".into();s
}
fn result(text:&str)->Option<Transcript>{Some(Transcript{text:text.into(),..Default::default()})}

#[test]
fn voice_turn_keeps_asr_microphone_and_provider_when_original_preferences_change(){
    let mut s=configured();let mut c=crate::pet_config::defaults();
    let input=Input::prepare(s.clone(),c.clone(),"Original microphone".into()).unwrap();
    s.chat_provider="copilot".into();s.voice_asr_python="/other/python".into();s.voice_asr_model="/other/model".into();s.voice_asr_engine="parakeet".into();
    c.pets.get_mut("agent_codex").unwrap().fast.model="replacement".into();
    assert_eq!(input.settings.chat_provider,"codex");
    assert_eq!(input.source,audio::Source::Mic(Some("Original microphone".into())));
    let mut calls=0;
    let text=input.recognition.transcribe_with(&[0.1;16000],None,|engine,_,_|{
        calls+=1;assert_eq!(engine,&Recognizer::Whisper{python:"/fixture/python".into(),model:"/fixture/whisper".into()});result("original")
    }).unwrap();
    assert_eq!(text.text,"original");assert_eq!(calls,1);
    assert!(input.check_with(||Ok(()),||Ok(c)).is_err());
}

#[test]
fn voice_turn_freezes_factory_models_effort_and_workspace_for_all_inference_routes(){
    let s=configured();let mut config=crate::pet_config::defaults();
    for (id,model,effort) in [("agent_codex","codex-fixture","high"),("agent_copilot","copilot-fixture","low"),("agent_antigravity","gravity-fixture",""),("ai_ollama","ollama-fixture","")] {
        let p=config.pets.get_mut(id).unwrap();p.mode="fast".into();p.fast.model=model.into();p.fast.effort=effort.into();
    }
    let input=Input::prepare(s,config.clone(),String::new()).unwrap();
    for (provider,id,model,effort) in [("codex","agent_codex","codex-fixture","high"),("copilot","agent_copilot","copilot-fixture","low"),("antigravity","agent_antigravity","gravity-fixture",""),("ollama","ai_ollama","ollama-fixture","")] {
        config.pets.get_mut(id).unwrap().fast.model="changed-after-capture".into();
        let settings=input.settings_for(provider);
        let selection=input.configuration.select(provider,&crate::chat::model_for(&settings,provider),"hello").unwrap();
        assert_eq!(selection.model,model);assert_eq!(selection.effort,effort);
        assert_eq!(settings.cli_workspace,"/fixture/original-workspace");
        assert_eq!(settings.ollama_url,"http://127.0.0.1:11434");
    }
    assert!(input.check_with(||Ok(()),||Ok(config)).is_err());
}

#[test]
fn denied_or_unavailable_provenance_never_queries_a_replacement_configuration(){
    let input=Input::prepare(configured(),crate::pet_config::defaults(),String::new()).unwrap();
    assert!(input.check_with(||Err("revoked".into()),||panic!("read after revocation")).is_err());
    assert!(input.check_with(||Ok(()),||Err("missing config".into())).is_err());
    assert!(input.check_with(||Ok(()),||Ok(input.configuration.clone())).is_ok());
}

#[test]
fn asr_cancel_before_call_and_between_chunks_never_publishes_partial_text(){
    let plan=Recognition::prepare(&configured(),&crate::pet_config::defaults()).unwrap();
    let stop=AtomicBool::new(true);
    assert!(plan.transcribe_with(&[0.1;100],Some(&stop),|_,_,_|panic!("cancelled inference")).is_none());
    stop.store(false,Ordering::Release);let mut calls=0;
    assert!(plan.transcribe_with(&vec![0.1;speech::SAMPLE_RATE as usize*60],Some(&stop),|_,_,_|{
        calls+=1;stop.store(true,Ordering::Release);result("partial must not escape")
    }).is_none());
    assert_eq!(calls,1);
}

#[test]
fn failing_selected_asr_never_falls_back_or_retries_another_engine(){
    for engine in ["parakeet","whisper"]{
        let mut s=configured();s.voice_asr_engine=engine.into();let plan=Recognition::prepare(&s,&crate::pet_config::defaults()).unwrap();let mut calls=0;
        assert!(plan.transcribe_with(&[0.1;16000],None,|backend,_,_|{
            calls+=1;assert_eq!(matches!(backend,Recognizer::Whisper{..}),engine=="whisper");None
        }).is_none());assert_eq!(calls,1);
    }
}

#[test]
fn whisper_chunks_share_original_target_and_vocabulary(){
    let mut s=configured();s.active_integrations.push("agent_codex".into());
    let mut c=crate::pet_config::defaults();c.pets.get_mut("agent_codex").unwrap().name="Original Codex".into();
    let plan=Recognition::prepare(&s,&c).unwrap();c.pets.get_mut("agent_codex").unwrap().name="Replacement".into();let mut calls=0;
    let out=plan.transcribe_with(&vec![0.1;speech::SAMPLE_RATE as usize*60],None,|backend,chunk,vocabulary|{
        assert!(matches!(backend,Recognizer::Whisper{model,..}if model==&PathBuf::from("/fixture/whisper")));
        assert!(chunk.len()<=speech::SAMPLE_RATE as usize*28);assert!(vocabulary.iter().any(|s|s=="Original Codex"));assert!(!vocabulary.iter().any(|s|s=="Replacement"));calls+=1;result("part")
    }).unwrap();assert!(calls>=3);assert_eq!(out.text.split_whitespace().count(),calls);
}

#[test]
fn malformed_or_incomplete_asr_selection_does_not_choose_defaults(){
    let c=crate::pet_config::defaults();let mut s=configured();
    s.voice_asr_engine="unknown".into();assert_eq!(super::super::asr_engine(&s),"unsupported");assert!(Recognition::prepare(&s,&c).is_err());
    s.voice_asr_engine="whisper".into();s.voice_asr_python.clear();assert!(Recognition::prepare(&s,&c).is_err());
    assert!(Input::prepare(configured(),c,"mic\nreplacement".into()).is_err());
}

#[test]
fn completing_an_old_hotword_worker_does_not_clear_its_replacement(){
    let previous=std::sync::Arc::new(AtomicBool::new(false));
    let replacement=std::sync::Arc::new(AtomicBool::new(false));
    let mut running = Some(super::super::Running {
        stop: replacement.clone(),
        capture: super::super::HotwordCapture::Starting,
    });
    super::super::release_running(&mut running,&previous);
    assert!(running.as_ref().is_some_and(|r|std::sync::Arc::ptr_eq(&r.stop,&replacement)));
    super::super::release_running(&mut running,&replacement);
    assert!(running.is_none());
}
