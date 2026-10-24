use super::*;
use crate::pet_data::{startup::tests::Harness,tests::Temp,publication};
use std::{fs,os::unix::fs::PermissionsExt};

#[test]fn unadmitted_selector_never_returns_default_or_legacy_paths(){
    assert_eq!(selected(None,true),Err(Error::Unavailable));assert_eq!(selected(None,false),Err(Error::Unavailable));
}
#[test]fn profile_is_admitted_before_settings_and_conversation_readers_obtain_paths(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();
    let config=selected(Some(&session),true).unwrap();let data=selected(Some(&session),false).unwrap();
    assert_eq!(config,layout.config().join("payload"));assert_eq!(data,layout.data().join("payload"));assert_eq!(session.settings().unwrap().language,"pt");
    let library=crate::assistant_memory::load(&data.join("assistant/conversations.json")).unwrap();assert!(library.pets.is_empty());assert_eq!(publication::inspect(&layout).unwrap(),publication::State::AdmissionRecordPresent);
    assert!(layout.legacy_data.join("inbox/example.txt").exists());assert!(matches!(h.claim(&layout),Err(Error::Busy)));
}
#[test]fn fresh_application_directories_follow_genuine_bus_ownership_and_admission(){
    let t=Temp::new();let layout=t.layout();let h=Harness::new();assert!(!layout.legacy_config.exists());assert!(!layout.legacy_data.exists());
    let claim=h.claim(&layout).unwrap();assert!(layout.legacy_config.exists());assert!(!layout.legacy_data.exists());
    let session=Session::prepare(claim,layout.clone()).unwrap();session.current().unwrap();assert!(layout.legacy_data.exists());assert!(session.settings().is_ok());
    assert_eq!(fs::metadata(layout.legacy_config).unwrap().permissions().mode()&0o7777,0o700);
}
#[test]fn runtime_storage_survives_session_bus_name_loss_without_issuing_new_startup_authority(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();h.release();
    session.current().unwrap();assert_eq!(session.settings().unwrap().language,"pt");assert!(h.claim(&layout).is_err());
}
#[test]fn replacing_a_lock_irrevocably_invalidates_runtime_storage(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let lock=layout.legacy_config.join(".pets-profile-v1.lock");
    fs::remove_file(&lock).unwrap();t.file(&lock,b"");assert!(session.current().is_err());assert_eq!(session.paths().err(),Some(Error::OwnershipLost));assert!(session.settings().is_err());
}
#[test]fn corrupt_control_record_cannot_revive_storage_when_repaired_in_place(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let record=layout.config().join(".admitted.json");let original=fs::read(&record).unwrap();
    fs::write(&record,b"{}").unwrap();assert!(session.current().is_err());fs::write(&record,original).unwrap();assert_eq!(session.current(),Err(Error::OwnershipLost));
}
#[test]fn mutable_v5_conversation_data_survives_bootstrap_restart_without_old_profile_reimport(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let paths=session.paths().unwrap();
    let current=br#"{"version":5,"pets":{},"archives":{}}"#;fs::write(paths.data.join("assistant/conversations.json"),current).unwrap();fs::write(paths.config.join("settings.json"),br#"{"language":"en"}"#).unwrap();drop(session);
    let next=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();assert_eq!(next.settings().unwrap().language,"en");assert_eq!(fs::read(next.paths().unwrap().data.join("assistant/conversations.json")).unwrap(),current);
    assert!(String::from_utf8(fs::read(layout.legacy_config.join("settings.json")).unwrap()).unwrap().contains("pt"));
}
#[test]fn divergent_incomplete_pair_does_not_create_settings_or_conversations_by_default(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let publication=publication::Publication::begin(h.claim(&layout).unwrap(),layout.clone()).unwrap();drop(publication);
    let conflict=layout.data().join("payload/inbox/example.txt");t.file(&conflict,b"conflicting bytes");
    assert!(Session::prepare(h.claim(&layout).unwrap(),layout.clone()).is_err());
    assert!(!layout.config().join("payload/settings.json").exists());assert!(!layout.data().join("payload/assistant/conversations.json").exists());assert!(layout.legacy_data.join("assistant/conversations.json").exists());
    assert_eq!(fs::read(conflict).unwrap(),b"conflicting bytes");
}
#[test]fn stable_hook_model_and_log_names_remain_outside_mutable_profile_paths(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();
    for name in["bin/coucou-hook","Models/model.bin","coucou.log"]{t.file(&layout.legacy_data.join(name),b"stable");}
    let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let paths=session.paths().unwrap();
    for name in["bin/coucou-hook","Models/model.bin","coucou.log"]{assert_eq!(fs::read(layout.legacy_data.join(name)).unwrap(),b"stable");assert!(!paths.data.join(name).exists());}
}
#[test]fn storage_paths_refuse_root_substitution_and_keep_original_files(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let kept=t.0.join("kept-profile");
    fs::rename(layout.data(),&kept).unwrap();fs::create_dir(layout.data()).unwrap();assert!(session.paths().is_err());assert_eq!(fs::read(kept.join("payload/inbox/example.txt")).unwrap(),b"private attachment");
}
#[test]fn all_declared_provider_histories_including_retired_github_survive_migrate_read_save(){
    let t=Temp::new();let layout=t.basic();let h=Harness::new();
    let mut providers=vec!["codex","copilot","antigravity","gemini-cli","claude-cli","ollama","lmstudio","custom","anthropic","openai","google","openrouter"];
    providers.extend(crate::pet_config::service_ids());providers.sort_unstable();providers.dedup();assert!(providers.contains(&"integration_github"));
    let mut conversations=std::collections::HashMap::new();
    for(index,provider)in providers.iter().enumerate(){
        conversations.insert(provider.to_string(),crate::chat::SavedConversation{id:format!("preserved-provider-{index}"),created:10,updated:20,messages:vec![serde_json::json!({"role":"user","content":format!("history for {provider}")}),serde_json::json!({"role":"assistant","content":"preserved reply"})],..Default::default()});
    }
    let disk=serde_json::json!({"version":4,"pets":conversations,"archives":{}});fs::write(layout.legacy_data.join("assistant/conversations.json"),serde_json::to_vec(&disk).unwrap()).unwrap();
    let session=Session::prepare(h.claim(&layout).unwrap(),layout.clone()).unwrap();let path=session.paths().unwrap().data.join("assistant/conversations.json");
    let library=crate::assistant_memory::load(&path).unwrap();assert_eq!(library.pets.len(),providers.len());crate::assistant_memory::save(&path,&library).unwrap();let restored=crate::assistant_memory::load(&path).unwrap();
    for provider in providers{assert_eq!(restored.pets[provider].id,conversations[provider].id);assert_eq!(restored.pets[provider].messages,conversations[provider].messages);}
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&fs::read(layout.legacy_data.join("assistant/conversations.json")).unwrap()).unwrap(),disk);
}
