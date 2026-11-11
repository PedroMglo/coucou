use super::*;
use crate::settings::EffectFixture as Fixture;
use std::sync::{Mutex,atomic::{AtomicUsize,Ordering}};

fn scope(f:&Fixture)->RecognitionScope{
    f.update(|s|{s.voice_asr_engine="whisper".into();s.voice_asr_python="/fixture/original-python".into();s.voice_asr_model="/fixture/original-model".into();});
    let settings=f.current.lock().unwrap();RecognitionScope::from_parts(settings.clone(),crate::pet_config::defaults(),Arc::new(crate::settings::capture_origin(&settings))).unwrap()
}
fn result()->Option<Transcript>{Some(Transcript{text:"captured result".into(),..Default::default()})}
#[test]fn capture_scope_never_refreshes_the_selected_engine_after_a_settings_commit(){
    let f=Fixture::new();let scope=scope(&f);f.update(|s|{s.voice_asr_engine="parakeet".into();s.voice_asr_model="/replacement".into();});
    assert!(matches!(scope.recognition.backend(),super::super::input::Recognizer::Whisper{model,..}if model==&std::path::PathBuf::from("/fixture/original-model")));
    assert!(scope.check_with(||scope.provenance(),||panic!("configuration queried after source retirement")).is_err());
}
#[test]fn capture_scope_refuses_replaced_configuration_before_any_recognition(){
    let f=Fixture::new();let scope=scope(&f);let mut other=scope.configuration.clone();other.pets.get_mut("agent_codex").unwrap().name="different context".into();
    let out=scope.recognition.transcribe_with_validity(&[0.1;16000],None,||scope.check_with(||scope.provenance(),||Ok(other.clone())).is_ok(),|_,_,_|panic!("unapproved recognition"));assert!(out.is_none());
}
#[test]fn capture_scope_revocation_during_a_chunk_discards_partial_output_and_blocks_next_chunk(){
    let f=Fixture::new();let scope=scope(&f);let calls=AtomicUsize::new(0);
    let out=scope.recognition.transcribe_with_validity(&vec![0.1;16000*60],None,||scope.check_with(||scope.provenance(),||Ok(scope.configuration.clone())).is_ok(),|_,_,_|{calls.fetch_add(1,Ordering::Relaxed);f.external();result()});
    assert!(out.is_none());assert_eq!(calls.load(Ordering::Relaxed),1);
}
#[test]fn capture_scope_keeps_cancellation_and_configuration_guards_between_chunks(){
    let f=Fixture::new();let scope=scope(&f);let configuration=Mutex::new(scope.configuration.clone());let calls=AtomicUsize::new(0);
    let out=scope.recognition.transcribe_with_validity(&vec![0.1;16000*60],None,||scope.check_with(||scope.provenance(),||Ok(configuration.lock().unwrap().clone())).is_ok(),|_,_,_|{
        calls.fetch_add(1,Ordering::Relaxed);configuration.lock().unwrap().pets.get_mut("agent_codex").unwrap().name="changed".into();result()
    });assert!(out.is_none());assert_eq!(calls.load(Ordering::Relaxed),1);
}
#[test]fn capture_scope_preserves_all_chunks_for_an_unchanged_original_owner(){
    let f=Fixture::new();let scope=scope(&f);let calls=AtomicUsize::new(0);
    let out=scope.recognition.transcribe_with_validity(&vec![0.1;16000*60],None,||scope.check_with(||scope.provenance(),||Ok(scope.configuration.clone())).is_ok(),|_,_,_|{calls.fetch_add(1,Ordering::Relaxed);result()}).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed),3);assert_eq!(out.text,"captured result captured result captured result");
}
#[test]fn capture_scope_stop_does_not_wait_for_or_touch_a_model(){
    let f=Fixture::new();let scope=scope(&f);let stop=AtomicBool::new(true);
    assert!(scope.recognition.transcribe_with_validity(&[0.1;16000],Some(&stop),||panic!("cancelled scope checked"),|_,_,_|panic!("cancelled model started")).is_none());
}
