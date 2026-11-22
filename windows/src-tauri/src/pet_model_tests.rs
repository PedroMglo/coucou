//! Deterministic selection tests: fake catalogs and persistence, no disk/network/model.
use super::*;

#[test]
fn the_event_carries_ack_generation_without_persisting_it_as_a_preference(){
    let mut settings=Settings::default();settings.chat_models.insert("ollama".into(),"confirmed:b".into());
    let before=serde_json::to_value(&settings).unwrap();let event=selection_event(&settings,42).unwrap();
    assert_eq!(event["ollamaSelectionGeneration"],42);assert_eq!(event["chatModels"]["ollama"],"confirmed:b");
    assert_eq!(event["voiceAsrEngine"],before["voiceAsrEngine"]);
    assert_eq!(serde_json::to_value(&settings).unwrap(),before);assert!(before.get("ollamaSelectionGeneration").is_none());
}
use std::{cell::RefCell, sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}, mpsc}, thread};

fn preferences() -> Mutex<Settings> {
    let mut settings=Settings::default();
    settings.ollama_url="http://127.0.0.1:11434".into();
    settings.chat_models.insert("ollama".into(),"test:previous".into());
    Mutex::new(settings)
}
fn catalog(ids:&[&str]) -> Vec<ModelInfo> {
    ids.iter().map(|id|ModelInfo{id:(*id).into(),label:format!("Label for {id}")}).collect()
}
fn choose(selections:&Selections,settings:&Mutex<Settings>,model:&str,nonce:&str) -> Selection {
    selections.begin(settings,model.into(),nonce.into()).unwrap().0
}
fn saved_model(settings:&Mutex<Settings>) -> String { settings.lock().unwrap().chat_models["ollama"].clone() }
fn json(settings:&Settings) -> serde_json::Value { serde_json::to_value(settings).unwrap() }
fn profiles() -> crate::pet_config::Configuration {
    let mut config=crate::pet_config::defaults();
    let pet=config.pets.get_mut("ai_ollama").unwrap();
    pet.provider="ollama".into();pet.mode="auto".into();
    pet.fast.model="test:fast".into();pet.reasoning.model="test:reasoning".into();
    config
}

#[test]
fn reversed_catalog_completions_only_persist_and_publish_the_latest_intent() {
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let a=choose(&selections,&settings,"test:a","selection-a");
    let b=choose(&selections,&settings,"test:b","selection-b");
    assert!(b.generation>a.generation);
    assert!(!chat.conversation("ollama").unwrap().busy,"catalog discovery must not reserve the pet");
    let persisted=RefCell::new(Vec::new());let emitted=RefCell::new(Vec::new());
    let ack=selections.finish(b,&settings,&chat,&catalog(&["test:b"]),None,
        |next|{persisted.borrow_mut().push(next.chat_models["ollama"].clone());Ok(())},
        |next|emitted.borrow_mut().push(next.chat_models["ollama"].clone())).unwrap();
    assert_eq!(ack.model,"test:b");assert_eq!(ack.selection_id,"selection-b");
    let error=selections.finish(a,&settings,&chat,&catalog(&["test:a"]),None,
        |_|panic!("superseded A must never reach persistence"),
        |_|panic!("superseded A must never emit settings")).unwrap_err();
    assert_eq!(error,SUPERSEDED);assert_eq!(saved_model(&settings),"test:b");
    assert_eq!(*persisted.borrow(),vec!["test:b"]);assert_eq!(*emitted.borrow(),vec!["test:b"]);
}

#[test]
fn a_failed_latest_discovery_catalog_or_save_still_blocks_the_older_intent() {
    for failure in ["discovery","catalog","save"] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let a=choose(&selections,&settings,"test:a","a");let b=choose(&selections,&settings,"test:b","b");
        let writes=AtomicUsize::new(0);let events=AtomicUsize::new(0);
        if failure=="discovery" {
            // A failed async catalog lookup never invokes finish at all.
            drop(b);
        } else {
            let models=if failure=="save"{catalog(&["test:b"])}else{catalog(&["test:a"])};
            assert!(selections.finish(b,&settings,&chat,&models,None,
                |_|{writes.fetch_add(1,Ordering::SeqCst);Err("fake save failure".into())},
                |_|{events.fetch_add(1,Ordering::SeqCst);}).is_err());
        }
        assert_eq!(selections.finish(a,&settings,&chat,&catalog(&["test:a"]),None,
            |_|panic!("older intent cannot resume after latest failure"),|_|panic!("no stale event")).unwrap_err(),SUPERSEDED);
        assert_eq!(writes.load(Ordering::SeqCst),usize::from(failure=="save"));
        assert_eq!(events.load(Ordering::SeqCst),0);assert_eq!(saved_model(&settings),"test:previous");
    }
}

#[test]
fn save_failure_preserves_every_memory_field_emits_nothing_and_releases_the_guard() {
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let choice=choose(&selections,&settings,"test:new","save-failure");
    settings.lock().unwrap().language="pt-PT".into();
    let before=json(&settings.lock().unwrap());let events=AtomicUsize::new(0);
    let error=selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
        |next|{
            assert_eq!(next.chat_models["ollama"],"test:new");
            assert_eq!(next.language,"pt-PT");assert!(chat.conversation("ollama").unwrap().busy);
            Err("fake persistence failure".into())
        },|_|{events.fetch_add(1,Ordering::SeqCst);}).unwrap_err();
    assert_eq!(error,"fake persistence failure");assert_eq!(events.load(Ordering::SeqCst),0);
    assert_eq!(json(&settings.lock().unwrap()),before);
    assert!(!chat.conversation("ollama").unwrap().busy);
    assert!(selections.generation.try_lock().is_ok());
    let next=chat.reserve_conversation("ollama",None).unwrap();drop(next);
}

#[test]
fn manual_selection_requires_the_exact_catalog_id_instead_of_a_label_or_an_implicit_default() {
    let models=vec![ModelInfo{id:"test:exact".into(),label:"Friendly model label".into()}];
    for requested in ["Friendly model label","TEST:EXACT","test:missing","@default"] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,requested,"invalid-catalog-id");
        assert!(selections.finish(choice,&settings,&chat,&models,None,|_|panic!("invalid ID persisted"),|_|panic!("invalid ID emitted")).is_err());
        assert_eq!(saved_model(&settings),"test:previous");
    }
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let choice=choose(&selections,&settings,"test:exact","exact-id");
    let ack=selections.finish(choice,&settings,&chat,&models,None,|_|Ok(()),|_|{}).unwrap();
    assert_eq!(ack.model,"test:exact");assert_eq!(saved_model(&settings),"test:exact");
}

#[test]
fn a_changed_server_url_invalidates_the_catalog_without_reverting_the_new_url() {
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let (choice,snapshot)=selections.begin(&settings,"test:new".into(),"url-change".into()).unwrap();
    assert_eq!(snapshot.ollama_url,"http://127.0.0.1:11434");
    settings.lock().unwrap().ollama_url="http://127.0.0.1:11435".into();
    assert!(selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
        |_|panic!("catalog belongs to the old URL"),|_|panic!("URL mismatch emitted")).is_err());
    assert_eq!(settings.lock().unwrap().ollama_url,"http://127.0.0.1:11435");
    assert_eq!(saved_model(&settings),"test:previous");assert!(!chat.conversation("ollama").unwrap().busy);
}

#[test]
fn committing_ollama_merges_only_its_model_into_the_latest_unrelated_preferences() {
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let choice=choose(&selections,&settings,"test:new","merge");
    let expected={
        let mut current=settings.lock().unwrap();
        current.language="pt-PT".into();current.sound_volume=0.37;current.companion_motion=false;
        current.chat_provider="codex".into();current.model="unrelated-api-model".into();
        current.chat_models.insert("codex".into(),"unrelated-codex-model".into());
        current.voice_models.insert("ollama".into(),"unrelated-voice-model".into());
        current.lmstudio_url="http://127.0.0.1:12345".into();current.cli_workspace="/fake/project".into();
        let mut expected=current.clone();expected.chat_models.insert("ollama".into(),"test:new".into());json(&expected)
    };
    selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
        |next|{assert_eq!(json(next),expected);Ok(())},
        |next|assert_eq!(json(next),expected)).unwrap();
    assert_eq!(json(&settings.lock().unwrap()),expected);
}

#[test]
fn queued_text_and_active_voice_refuse_selection_before_any_save_or_event() {
    for voice in [false,true] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,"test:new","busy");
        let (text,lease)=if voice {
            let turn=chat.begin_cli("ollama",Arc::default()).unwrap();(None,Some(chat.lease(&turn)))
        } else {(Some(chat.reserve_conversation("ollama",None).unwrap()),None)};
        assert!(selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
            |_|panic!("busy selection reached disk"),|_|panic!("busy selection emitted")).is_err());
        assert_eq!(saved_model(&settings),"test:previous");assert!(chat.conversation("ollama").unwrap().busy);
        drop(text);drop(lease);assert!(!chat.conversation("ollama").unwrap().busy);
    }
}

#[test]
fn the_commit_guard_excludes_text_voice_and_maintenance_through_persistence_and_publication() {
    let selections=Selections::default();let settings=preferences();let chat=Chat::default();
    let choice=choose(&selections,&settings,"test:new","guard");
    let stages=RefCell::new(Vec::new());let disk=RefCell::new(None::<Settings>);
    let guarded=||{
        assert!(chat.conversation("ollama").unwrap().busy);
        assert!(chat.reserve_conversation("ollama",None).is_err());
        assert!(chat.begin_cli("ollama",Arc::default()).is_err());
        assert!(chat.reset_provider("ollama").is_err());assert!(chat.reset_checked().is_err());
        assert!(settings.try_lock().is_err());assert!(selections.generation.try_lock().is_err());
    };
    let ack=selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
        |next|{guarded();assert!(stages.borrow().is_empty());*disk.borrow_mut()=Some(next.clone());stages.borrow_mut().push("persist");Ok(())},
        |next|{
            guarded();assert_eq!(*stages.borrow(),vec!["persist"]);
            assert_eq!(disk.borrow().as_ref().unwrap().chat_models["ollama"],next.chat_models["ollama"]);
            stages.borrow_mut().push("event");
        }).unwrap();
    stages.borrow_mut().push("ack");
    assert_eq!(*stages.borrow(),vec!["persist","event","ack"]);
    assert_eq!(ack.model,"test:new");assert_eq!(ack.provider,"ollama");assert_eq!(ack.selection_id,"guard");
    assert_eq!(ack.generation,1);assert_eq!(saved_model(&settings),ack.model);
    let wire=serde_json::to_value(&ack).unwrap();assert_eq!(wire["selectionId"],"guard");
    assert!(!chat.conversation("ollama").unwrap().busy);
    let voice=chat.begin_cli("ollama",Arc::default()).unwrap();drop(chat.lease(&voice));
    chat.reset_provider("ollama").unwrap();
}

#[test]
fn concurrent_preference_writers_wait_until_the_same_model_is_persisted_and_published() {
    let selections=Arc::new(Selections::default());let settings=Arc::new(preferences());let chat=Chat::default();
    let choice=choose(&selections,&settings,"test:new","concurrent");
    let entered=Arc::new(Barrier::new(2));let release=Arc::new(Barrier::new(2));
    let stages=Arc::new(Mutex::new(Vec::new()));let (ack_tx,ack_rx)=mpsc::channel();
    let worker={
        let selections=selections.clone();let settings=settings.clone();let chat=chat.clone();
        let entered=entered.clone();let release=release.clone();let stages=stages.clone();
        thread::spawn(move||{
            let result=selections.finish(choice,&settings,&chat,&catalog(&["test:new"]),None,
                |next|{
                    stages.lock().unwrap().push(("persist",next.chat_models["ollama"].clone()));
                    entered.wait();release.wait();Ok(())
                },|next|stages.lock().unwrap().push(("event",next.chat_models["ollama"].clone())));
            ack_tx.send(result).unwrap();
        })
    };
    entered.wait();
    let busy=chat.conversation("ollama").unwrap().busy;
    let settings_locked=settings.try_lock().is_err();let intent_locked=selections.generation.try_lock().is_err();
    let ack_was_pending=matches!(ack_rx.try_recv(),Err(mpsc::TryRecvError::Empty));
    let (attempt_tx,attempt_rx)=mpsc::channel();let writer_gate=Arc::new(Barrier::new(2));
    let writer={
        let settings=settings.clone();let stages=stages.clone();let gate=writer_gate.clone();
        thread::spawn(move||{
            gate.wait();let blocked=settings.try_lock().is_err();attempt_tx.send(blocked).unwrap();
            let mut current=settings.lock().unwrap();current.language="concurrent-preference".into();
            current.chat_models.insert("codex".into(),"concurrent-codex-model".into());
            stages.lock().unwrap().push(("writer",current.chat_models["ollama"].clone()));
        })
    };
    writer_gate.wait();let writer_was_blocked=attempt_rx.recv().unwrap();
    release.wait();worker.join().unwrap();writer.join().unwrap();
    let ack=ack_rx.recv().unwrap().unwrap();
    assert!(busy&&settings_locked&&intent_locked&&ack_was_pending&&writer_was_blocked);
    assert_eq!(*stages.lock().unwrap(),vec![("persist","test:new".to_string()),("event","test:new".to_string()),("writer","test:new".to_string())]);
    let current=settings.lock().unwrap();assert_eq!(current.chat_models["ollama"],ack.model);
    assert_eq!(current.chat_models["codex"],"concurrent-codex-model");assert_eq!(current.language,"concurrent-preference");
    assert!(!chat.conversation("ollama").unwrap().busy);
}

#[test]
fn native_profile_aliases_are_validated_but_persisted_and_acknowledged_without_expansion() {
    let config=profiles();let models=catalog(&["test:fast","test:reasoning"]);
    for alias in ["@fast","@reasoning","@auto","@default"] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,alias,"native-profile");let events=RefCell::new(Vec::new());
        let ack=selections.finish(choice,&settings,&chat,&models,Some(&config),
            |next|{assert_eq!(next.chat_models["ollama"],alias);Ok(())},
            |next|events.borrow_mut().push(next.chat_models["ollama"].clone())).unwrap();
        assert_eq!(ack.model,alias);assert_eq!(saved_model(&settings),alias);assert_eq!(*events.borrow(),vec![alias]);
    }
}

#[test]
fn profile_aliases_require_exact_catalog_models_for_the_native_mode() {
    for (alias,mode,ids,valid) in [
        ("@fast","auto",vec!["test:fast"],true),
        ("@reasoning","auto",vec!["test:reasoning"],true),
        ("@auto","fast",vec!["test:fast"],false),
        ("@auto","reasoning",vec!["test:reasoning"],false),
        ("@default","fast",vec!["test:fast"],true),
        ("@default","reasoning",vec!["test:reasoning"],true),
        ("@default","auto",vec!["test:fast"],false),
        ("@default","invalid",vec!["test:fast","test:reasoning"],false),
    ] {
        let mut config=profiles();config.pets.get_mut("ai_ollama").unwrap().mode=mode.into();
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,alias,"mode-profile");let writes=AtomicUsize::new(0);let events=AtomicUsize::new(0);
        let result=selections.finish(choice,&settings,&chat,&catalog(&ids),Some(&config),
            |_|{writes.fetch_add(1,Ordering::SeqCst);Ok(())},|_|{events.fetch_add(1,Ordering::SeqCst);});
        assert_eq!(result.is_ok(),valid,"alias {alias}, mode {mode}");
        assert_eq!(writes.load(Ordering::SeqCst),usize::from(valid));assert_eq!(events.load(Ordering::SeqCst),usize::from(valid));
    }
}

#[test]
fn missing_invalid_or_cloud_profiles_never_save_or_publish_a_selection() {
    let config=profiles();
    let mut missing=config.clone();missing.pets.remove("ai_ollama");
    let mut empty=config.clone();empty.pets.get_mut("ai_ollama").unwrap().fast.model.clear();
    let mut other_provider=config.clone();other_provider.pets.get_mut("ai_ollama").unwrap().provider="codex".into();
    let mut cloud=config.clone();cloud.pets.get_mut("ai_ollama").unwrap().fast.model="remote:cloud".into();
    for (alias,config) in [("@fast",None),("@unknown",Some(&config)),("@fast",Some(&missing)),("@fast",Some(&empty)),("@fast",Some(&other_provider)),("@fast",Some(&cloud))] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,alias,"bad-profile");
        assert!(selections.finish(choice,&settings,&chat,&catalog(&["test:fast","test:reasoning","remote:cloud","@unknown"]),config,
            |_|panic!("bad profile persisted"),|_|panic!("bad profile emitted")).is_err());
        assert_eq!(saved_model(&settings),"test:previous");assert!(!chat.conversation("ollama").unwrap().busy);
    }
}

#[test]
fn cloud_model_ids_are_rejected_even_if_the_server_lists_them() {
    for id in ["remote:cloud","remote-cloud"] {
        let selections=Selections::default();let settings=preferences();let chat=Chat::default();
        let choice=choose(&selections,&settings,id,"cloud-id");
        assert!(selections.finish(choice,&settings,&chat,&catalog(&[id]),None,
            |_|panic!("cloud model persisted"),|_|panic!("cloud model emitted")).is_err());
        assert_eq!(saved_model(&settings),"test:previous");
    }
}
