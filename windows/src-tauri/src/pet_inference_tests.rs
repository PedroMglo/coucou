//! Synthetic configurations only; no credential reads, sockets or real models.
use super::*;
use crate::pet_config::{self,LocalExecutionMode};
fn choice(backend:Backend,endpoint:&str,model:&str)->Choice{
 Choice{schema_version:1,backend,endpoint:endpoint.into(),model:model.into(),credential:CredentialReference::None}
}
fn configured()->Configuration{
 let mut c=pet_config::defaults();c.version=2;
 c.pets.get_mut("ai_ollama").unwrap().inference=Some(choice(Backend::Lmstudio,"http://127.0.0.1:1234","synthetic/lm"));
 c.pets.get_mut("ai_lmstudio").unwrap().inference=Some(choice(Backend::Ollama,"http://localhost:1234","synthetic:local"));
 c.pets.get_mut("ai_ollama").unwrap().inference_revision=Some("a".repeat(32));c.pets.get_mut("ai_lmstudio").unwrap().inference_revision=Some("b".repeat(32));c
}
#[test]fn explicit_engines_keep_logical_owners_and_do_not_copy_policy(){
 let c=configured();pet_config::validate(&c).unwrap();
 let ollama=Target::for_provider(&c,"ollama").unwrap().unwrap();let lm=Target::for_provider(&c,"lmstudio").unwrap().unwrap();
 assert_eq!((&*ollama.pet_id,&*ollama.provider,ollama.choice.backend),("ai_ollama","ollama",Backend::Lmstudio));
 assert_eq!((&*lm.pet_id,&*lm.provider,lm.choice.backend),("ai_lmstudio","lmstudio",Backend::Ollama));
 assert_eq!(c.local_execution_for("ai_lmstudio").mode,LocalExecutionMode::Disabled);
 assert_eq!(c.local_execution_for("ai_ollama").mode,LocalExecutionMode::Ask);
 assert_eq!(c.pets["ai_lmstudio"].permissions,pet_config::defaults().pets["ai_lmstudio"].permissions);
 assert_eq!(c.select("lmstudio","@default","olá").unwrap().model,"synthetic:local");
 assert!(c.select("lmstudio","different-model","olá").is_err());
 assert!(c.select("lmstudio","@reasoning","olá").is_err());
 assert_eq!(c.select("codex","@fast","olá").unwrap().model,pet_config::defaults().pets["agent_codex"].fast.model);
}
#[test]fn captured_destination_survives_later_configuration_without_alias_bypass(){
 let mut c=configured();let original=Target::configured(&c,"ai_ollama").unwrap().unwrap();
 assert_eq!(original.resource_key(),Target::configured(&c,"ai_lmstudio").unwrap().unwrap().resource_key());
 c.pets.get_mut("ai_lmstudio").unwrap().inference.as_mut().unwrap().endpoint="http://[::1]:1234".into();
 assert_eq!(original.resource_key(),Target::configured(&c,"ai_lmstudio").unwrap().unwrap().resource_key());
 c.pets.get_mut("ai_ollama").unwrap().inference.as_mut().unwrap().endpoint="http://127.0.0.1:11434".into();
 c.pets.get_mut("ai_ollama").unwrap().inference.as_mut().unwrap().model="other-model".into();
 let next=Target::configured(&c,"ai_ollama").unwrap().unwrap();assert_ne!(original.resource_key(),next.resource_key());
 assert_eq!(original.choice.endpoint,"http://127.0.0.1:1234");assert_eq!(original.choice.model,"synthetic/lm");
}
#[test]fn invalid_present_binding_never_becomes_legacy_or_another_account(){
 let mut c=configured();let selected=c.pets["ai_ollama"].inference.clone().unwrap();
 for endpoint in ["","http://example.com:1234","http://user@localhost:1234","http://localhost:1234/?secret=x","http://localhost:1234/#x","http://localhost:0","file:///tmp/model"," http://localhost:1234","http://localhost:1234/../model"]{
  c.pets.get_mut("ai_ollama").unwrap().inference=Some(Choice{endpoint:endpoint.into(),..selected.clone()});
  assert!(Target::configured(&c,"ai_ollama").is_err(),"{endpoint}");assert!(pet_config::validate(&c).is_err());
 }
 c=configured();c.pets.get_mut("agent_codex").unwrap().inference=Some(selected);assert!(Target::configured(&c,"agent_codex").is_err());
 assert!(pet_config::validate(&c).is_err());
 c=configured();c.pets.get_mut("ai_lmstudio").unwrap().inference.as_mut().unwrap().credential=CredentialReference::ConfiguredLmstudioEndpoint;
 assert!(Target::configured(&c,"ai_lmstudio").is_err());
 c=configured();c.pets.get_mut("ai_lmstudio").unwrap().inference.as_mut().unwrap().model="synthetic:cloud".into();assert!(Target::configured(&c,"ai_lmstudio").is_err());
}
#[test]fn format_roundtrip_preserves_legacy_bytes_and_refuses_partial_choices(){
 let legacy=serde_json::to_vec(&pet_config::defaults()).unwrap();
 let read=pet_config::resolve_existing_bytes(&legacy).unwrap();assert_eq!(serde_json::to_vec(&read).unwrap(),legacy);
 assert!(Target::configured(&read,"ai_lmstudio").unwrap().is_none());
 let current=configured();let encoded=serde_json::to_vec(&current).unwrap();let read=pet_config::resolve_existing_bytes(&encoded).unwrap();
 assert_eq!(serde_json::to_vec(&read).unwrap(),encoded);
 let mut v=serde_json::to_value(&current).unwrap();v["version"]=serde_json::json!(1);assert!(pet_config::resolve_existing_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
 v=serde_json::to_value(&current).unwrap();v["pets"]["ai_ollama"]["inference"].as_object_mut().unwrap().remove("credential");assert!(serde_json::from_value::<Configuration>(v).is_err());
 v=serde_json::to_value(&current).unwrap();v["pets"]["ai_ollama"]["inference"]["fallback"]=serde_json::json!("ollama");assert!(serde_json::from_value::<Configuration>(v).is_err());
}
