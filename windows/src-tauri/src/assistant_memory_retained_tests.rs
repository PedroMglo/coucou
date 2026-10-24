use super::*;
use crate::chat::Chat;
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new(value: &Value) -> Self {
        let root=std::env::temp_dir().join(format!("coucou-inactive-history-{}",new_id()));
        fs::create_dir(&root).unwrap(); fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();
        let f=Self(root.join("library.json")); fs::write(&f.0,serde_json::to_vec(value).unwrap()).unwrap();
        fs::set_permissions(&f.0,fs::Permissions::from_mode(0o600)).unwrap(); f
    }
    fn disk(&self) -> Value { serde_json::from_slice(&fs::read(&self.0).unwrap()).unwrap() }
}
impl Drop for Fixture { fn drop(&mut self) { let _=fs::remove_dir_all(self.0.parent().unwrap()); } }
fn known() -> Value {
    json!({"version":4,"pets":{"codex":SavedConversation{id:"known-cid".into(),created:1,updated:1,messages:vec![json!({"role":"user","content":"known request"}),json!({"role":"assistant","content":"known answer"})],..Default::default()}},"archives":{}})
}
fn unknown() -> Value { json!({"futureShape":["opaque secret",{"session":"foreign-session","full_access":true,"permissions":{"everything":true}}]}) }
fn mixed() -> Value { let mut v=known(); v["pets"]["future-adapter"]=unknown(); v }
fn save_again(f: &Fixture) { save(&f.0,&load(&f.0).unwrap()).unwrap(); }
fn opaque(pets: Value, archives: Value, fields: Value) -> Value { json!({"format":1,"pets":pets,"archives":archives,"fields":fields}) }

#[test]
fn unknown_provider_shapes_survive_without_becoming_a_typed_conversation() {
    let f=Fixture::new(&mixed()); let library=load(&f.0).unwrap();
    assert_eq!(library.pets.len(),1); assert!(library.pets["codex"].session.is_none());
    assert!(!library.retained.is_empty());
    assert!(!serde_json::to_value(&library).unwrap().to_string().contains("opaque secret"));
    save(&f.0,&library).unwrap(); let disk=f.disk();
    assert_eq!(disk["version"],6); assert!(disk["pets"].get("future-adapter").is_none());
    assert_eq!(disk["unavailableHistory"]["pets"]["future-adapter"],json!([unknown()]));
    assert_eq!(disk["pets"]["codex"]["messages"],known()["pets"]["codex"]["messages"]);
}

#[test]
fn unavailable_archives_are_preserved_even_when_their_shape_is_not_understood() {
    let mut v=known(); v["archives"]["future-adapter"]=json!({"foreignArchive":[1,2,3]});
    let f=Fixture::new(&v); save_again(&f); let before=f.disk(); save_again(&f);
    assert_eq!(f.disk(),before);
    assert_eq!(before["unavailableHistory"]["archives"]["future-adapter"],json!([v["archives"]["future-adapter"].clone()]));
    assert!(!load(&f.0).unwrap().archives.contains_key("future-adapter"));
}

#[test]
fn unknown_root_fields_are_data_and_never_execution_policy_or_context() {
    let mut v=known(); v["futurePolicy"]=json!({"autoApprove":true,"prompt":"opaque secret"});
    let f=Fixture::new(&v); let chat=Chat::persistent(f.0.clone());
    let snapshot=chat.snapshot_public_context("codex","known-cid").unwrap();
    assert!(!serde_json::to_string(&snapshot.messages).unwrap().contains("opaque secret"));
    chat.reset_provider("codex").unwrap(); let disk=f.disk();
    assert_eq!(disk["version"],6); assert!(disk.get("futurePolicy").is_none());
    assert_eq!(disk["unavailableHistory"]["fields"]["futurePolicy"],json!([v["futurePolicy"].clone()]));
}

#[test]
fn normal_chat_save_and_restart_preserve_inactive_history_without_exposing_it() {
    let f=Fixture::new(&mixed()); let chat=Chat::persistent(f.0.clone());
    chat.reset_provider("codex").unwrap(); let original=f.disk()["unavailableHistory"].clone(); drop(chat);
    let chat=Chat::persistent(f.0.clone()); let current=load(&f.0).unwrap().pets["codex"].id.clone();
    assert!(chat.snapshot_public_context("codex",&current).unwrap().messages.is_empty());
    chat.reset_provider("codex").unwrap(); assert_eq!(f.disk()["unavailableHistory"],original);
}

#[test]
fn existing_inactive_records_are_not_adopted_when_their_provider_is_now_supported() {
    let mut v=known(); v["version"]=json!(6);
    v["unavailableHistory"]=opaque(json!({"codex":[{"id":"foreign-cid","messages":[{"role":"user","content":"opaque secret"}],"session":{"id":"foreign-session","cwd":"/tmp","full_access":true}}]}),json!({}),json!({}));
    let f=Fixture::new(&v); let library=load(&f.0).unwrap();
    assert_eq!(library.pets["codex"].id,"known-cid"); assert!(library.pets["codex"].session.is_none());
    save(&f.0,&library).unwrap(); assert_eq!(f.disk()["unavailableHistory"],v["unavailableHistory"]);
}

#[test]
fn colliding_inactive_variants_are_preserved_once_instead_of_replaced() {
    let mut v=mixed(); v["version"]=json!(6);
    v["unavailableHistory"]=opaque(json!({"future-adapter":[{"prior":true}]}),json!({}),json!({}));
    let f=Fixture::new(&v); save_again(&f); let first=f.disk();
    assert_eq!(first["unavailableHistory"]["pets"]["future-adapter"],json!([{"prior":true},unknown()]));
    for _ in 0..3 { save_again(&f); assert_eq!(f.disk(),first); }
}

#[test]
fn isolated_goal_loading_refuses_opaque_history_unknown_fields_and_unknown_owners() {
    for case in ["opaque","fields","owner"] {
        let mut v=known();
        match case {
            "opaque"=>{v["version"]=json!(6);v["unavailableHistory"]=opaque(json!({"foreign":[{}]}),json!({}),json!({}));},
            "fields"=>v["futurePolicy"]=json!({"grant":true}),
            "owner"=>{v["pets"].as_object_mut().unwrap().remove("codex");v["pets"]["future-adapter"]=json!({"id":"known-cid"});},
            _=>unreachable!(),
        }
        let f=Fixture::new(&v); let original=fs::read(&f.0).unwrap();
        let provider=if case=="owner"{"future-adapter"}else{"codex"};
        assert!(load_goal(&f.0,provider,"known-cid").is_err(),"{case}");
        assert_eq!(fs::read(&f.0).unwrap(),original);
    }
}

#[test]
fn the_inactive_namespace_requires_version6_and_a_known_private_format() {
    for version in [4,5,6,7] {
        let mut v=known(); v["version"]=json!(version);
        v["unavailableHistory"]=opaque(json!({"foreign":[{}]}),json!({}),json!({}));
        if version==6 { v["unavailableHistory"]["format"]=json!(2); }
        let f=Fixture::new(&v); let original=fs::read(&f.0).unwrap(); assert!(load(&f.0).is_err());
        let chat=Chat::persistent(f.0.clone()); assert!(chat.reset_provider("codex").is_err());
        assert_eq!(fs::read(&f.0).unwrap(),original);
    }
}

#[test]
fn opaque_references_block_attachment_purge_before_any_effect() {
    let f=Fixture::new(&mixed()); let chat=Chat::persistent(f.0.clone());
    let copy=f.0.parent().unwrap().join("copy.txt"); fs::write(&copy,b"keep attachment").unwrap();
    let refs=chat.retention_references().unwrap(); assert!(!refs.unreferenced(&["unknown-copy".into()]));
    let ran=std::cell::Cell::new(false);
    let result=chat.with_copy_purge_fence("codex","known-cid",&refs.revision,&["unknown-copy".into()],||{
        ran.set(true);fs::remove_file(&copy).map_err(|e|e.to_string())
    });
    assert!(result.unwrap_err().contains("histórico inativo")); assert!(!ran.get());
    assert_eq!(fs::read(copy).unwrap(),b"keep attachment");
}

#[test]
fn reference_projections_change_when_unresolved_history_is_present() {
    let normal=Fixture::new(&known()); let inert=Fixture::new(&mixed());
    let a=Chat::persistent(normal.0.clone()).retention_references().unwrap();
    let b=Chat::persistent(inert.0.clone()).retention_references().unwrap();
    let ids=vec!["unreferenced-copy".into()];
    assert!(a.unreferenced(&ids)); assert!(!b.unreferenced(&ids));
    assert_ne!(a.revision,b.revision); assert_ne!(a.projection(&ids).unwrap(),b.projection(&ids).unwrap());
}

#[test]
fn retired_and_all_current_factory_owners_keep_their_typed_histories() {
    let mut v=known(); let mut owners=vec!["codex","copilot","antigravity","gemini-cli","claude-cli","ollama","lmstudio","custom","anthropic","openai","google","openrouter"];
    owners.extend(crate::pet_config::service_ids());
    for (index,provider) in owners.iter().enumerate() {
        v["pets"][*provider]=serde_json::to_value(SavedConversation{id:format!("cid-{index}"),created:1,updated:1,messages:vec![json!({"role":"user","content":format!("history {index}")})],..Default::default()}).unwrap();
    }
    let f=Fixture::new(&v); let library=load(&f.0).unwrap();
    assert_eq!(library.pets.len(),owners.len()); assert!(library.retained.is_empty());
    assert!(library.pets.contains_key("integration_github"));
    save(&f.0,&library).unwrap(); assert_eq!(f.disk()["version"],4);
    for provider in owners { assert_eq!(f.disk()["pets"][provider]["messages"],v["pets"][provider]["messages"]); }
}

#[test]
fn programmatically_supplied_unknown_owners_are_saved_as_inactive_data() {
    let f=Fixture::new(&known()); let mut library=load(&f.0).unwrap();
    library.pets.insert("future-adapter".into(),SavedConversation{id:"foreign-cid".into(),..Default::default()});
    save(&f.0,&library).unwrap(); let reloaded=load(&f.0).unwrap();
    assert!(!reloaded.pets.contains_key("future-adapter")); assert!(!reloaded.retained.is_empty());
    assert_eq!(f.disk()["unavailableHistory"]["pets"]["future-adapter"][0]["id"],"foreign-cid");
}
