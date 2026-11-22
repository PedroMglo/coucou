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

#[test]fn custom_compatible_choice_preserves_owner_permissions_and_exact_remote_destination(){
 let mut config=pet_config::defaults();let before=config.pets["agent_codex"].clone();let permissions=config.pets["ai_ollama"].permissions.clone();config.version=2;
 let selected=Choice{schema_version:1,backend:Backend::Custom,endpoint:"https://models.example.invalid/private/v1".into(),model:"fixture-custom-model".into(),credential:CredentialReference::ConfiguredCustomEndpoint};
 let pet=config.pets.get_mut("ai_ollama").unwrap();pet.inference=Some(selected.clone());pet.inference_revision=Some("a".repeat(32));
 pet_config::validate(&config).unwrap();let target=Target::for_provider(&config,"ollama").unwrap().unwrap();assert_eq!(target.provider,"ollama");assert_eq!(target.choice,selected);assert!(pet_config::blueprint("ai_ollama").unwrap().recognizes_owner("ollama","custom"));assert_eq!(config.pets["ai_ollama"].permissions,permissions);assert_eq!(serde_json::to_value(&config.pets["agent_codex"]).unwrap(),serde_json::to_value(before).unwrap());
 assert_eq!(config.select("ollama","@default","test").unwrap().model,"fixture-custom-model");assert!(config.select("ollama","another-model","test").is_err());assert!(crate::local_chat::lmstudio::target_auth(&target).is_err());
 let bytes=serde_json::to_vec(&config).unwrap();assert_eq!(serde_json::to_vec(&pet_config::resolve_existing_bytes(&bytes).unwrap()).unwrap(),bytes);
}
#[test]fn custom_credentials_are_explicit_and_cannot_use_lmstudio_or_cleartext_remote_auth(){
 let blueprint=pet_config::blueprint("ai_ollama").unwrap();let anonymous=choice(Backend::Custom,"http://models.example.invalid/v1","fixture-custom");anonymous.validate(blueprint).unwrap();
 let mut selected=anonymous.clone();selected.credential=CredentialReference::ConfiguredCustomEndpoint;assert!(selected.validate(blueprint).is_err());selected.endpoint="https://models.example.invalid/v1".into();selected.validate(blueprint).unwrap();
 selected.credential=CredentialReference::ConfiguredLmstudioEndpoint;assert!(selected.validate(blueprint).is_err());
 for backend in [Backend::Lmstudio,Backend::Ollama]{let selected=Choice{backend,endpoint:"http://127.0.0.1:1234".into(),credential:CredentialReference::ConfiguredCustomEndpoint,..anonymous.clone()};assert!(selected.validate(blueprint).is_err());}
 for endpoint in ["https://user@example.invalid/v1","https://example.invalid/v1?token=x","https://example.invalid/v1#x","https://example.invalid/../private","file:///tmp/model"]{assert!(Choice{endpoint:endpoint.into(),..anonymous.clone()}.validate(blueprint).is_err());}
 assert!(anonymous.validate(pet_config::blueprint("agent_codex").unwrap()).is_err());assert!(anonymous.validate(pet_config::blueprint("agent_system_guardian").unwrap()).is_err());
}
