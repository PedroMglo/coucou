use super::*;

#[test]fn guardian_health_capability_is_inference_independent_and_cannot_project_a_sound_grant(){
 use crate::pet_config::system_access::{Grant,Operation,Policy,Resource,Subplugin};
 let mut f=Fixture::new();f.settings.active_integrations.push("agent_system_guardian".into());f.config.version=2;
 let pet=f.config.pets.get_mut("agent_system_guardian").unwrap();pet.system_access=Some(Policy{schema_version:1,enabled:true,revision:"a".repeat(32),grants:vec![Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::HostHealth{}}]});
 assert!(pet.inference.is_none());let s=f.snapshot(100_000);let c=capability(&s,"agent_system_guardian","system.health.observe");assert_eq!(c.support,Support::Supported);assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert_eq!(c.availability,Availability::Unknown);assert!(c.model_binding.is_none());assert_eq!(capability(&s,"agent_system_guardian","system.sound.observe").reason,Some(Reason::PermissionDenied));
 assert!(!crate::pet_operations::definitions().iter().any(|d|d["function"]["name"]=="health"));
}
use serde_json::json;
struct Fixture{registry:Registry,settings:Settings,config:Configuration}
impl Fixture{
    fn new()->Self{let config=crate::pet_config::defaults();let mut settings=Settings::default();settings.main_pill="agent_codex".into();settings.active_integrations=config.pets.keys().cloned().collect();settings.ollama_url="http://127.0.0.1:11434".into();settings.chat_models.insert("ollama".into(),"synthetic-model".into());Self{registry:Registry::new().unwrap(),settings,config}}
    fn snapshot(&mut self,now:u64)->Snapshot{self.registry.snapshot(&self.settings,Some(&self.config),true,&BTreeSet::new(),now).unwrap()}
    fn model(&mut self,body:serde_json::Value,now:u64){let k=Key{provider:"ollama".into(),endpoint:cache::endpoint(&self.settings.ollama_url).unwrap(),model:Some("synthetic-model".into())};let t=self.registry.cache.begin(k,now).unwrap();assert!(self.registry.cache.finish(t,cache::model_evidence(&body),now));}
}
fn descriptor<'a>(s:&'a Snapshot,id:&str)->&'a Descriptor{s.pets.iter().find(|p|p.pet_id==id).unwrap()}
fn capability<'a>(s:&'a Snapshot,id:&str,cap:&str)->&'a Capability{descriptor(s,id).capabilities.iter().find(|c|c.id==cap).unwrap()}
#[test]fn pet_inference_config2_preserves_other_pets_and_projects_actual_engine_without_implicit_goal_authority(){
 let mut f=Fixture::new();f.model(json!({"capabilities":["completion"]}),100_000);let before=f.snapshot(100_001);f.config.version=2;
 let p=f.config.pets.get_mut("ai_lmstudio").unwrap();p.inference=Some(crate::pet_inference::Choice{schema_version:1,backend:crate::pet_inference::Backend::Ollama,endpoint:"http://127.0.0.1:18471".into(),model:"synthetic-chosen".into(),credential:crate::pet_inference::CredentialReference::None});p.inference_revision=Some("a".repeat(32));
 let cold=f.snapshot(100_002);for id in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama"]{assert_eq!(serde_json::to_value(descriptor(&before,id)).unwrap(),serde_json::to_value(descriptor(&cold,id)).unwrap(),"{id}");}
 let d=descriptor(&cold,"ai_lmstudio");assert_eq!(d.binding.provider.as_deref(),Some("lmstudio"));assert_eq!(d.binding.adapter,Some("ollama"));assert_eq!(d.binding.model.as_deref(),Some("synthetic-chosen"));assert_ne!(capability(&cold,"ai_lmstudio","conversation.respond").availability,Availability::Ready);
 let k=key(&f.settings,&d.binding,&f.config.pets["ai_lmstudio"],Some("synthetic-chosen".into())).unwrap();let t=f.registry.cache.begin(k,100_003).unwrap();assert!(f.registry.cache.finish(t,cache::model_evidence(&json!({"capabilities":["completion","tools"]})),100_003));
 let warm=f.snapshot(100_004);let response=capability(&warm,"ai_lmstudio","conversation.respond");assert_eq!(response.availability,Availability::Ready);assert!(response.callers.contains(&Caller::NativeUser));assert!(response.callers.contains(&Caller::TaskRuntime));
 for name in ["goals.planBounded","goals.executeRecipe","goals.verifyCriteria"]{let c=capability(&warm,"ai_lmstudio",name);assert_eq!(c.support,Support::Supported);assert_eq!(c.reason,Some(Reason::TargetRequired));assert!(c.callers.contains(&Caller::TaskRuntime));assert!(c.limitations.contains(&"exactHumanRecipeApprovalRequired"));}
 for name in ["goals.peerIterationPlanningReview","goals.peerIterationPlanningExecution"]{let c=capability(&warm,"ai_lmstudio",name);assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.decision,Decision::NativeApproval);assert_eq!(c.callers.contains(&Caller::TaskRuntime),name.ends_with("Execution"));assert!(!c.callers.contains(&Caller::ProviderModel));assert!(c.limitations.contains(&"eachFurtherRoundNeedsExactNativeDataBodyDestinationConsent"));assert!(c.limitations.contains(&"oldOneRoundAdmissionsNeverPromoted"));for pet in ["agent_codex","agent_copilot","agent_antigravity"]{assert_eq!(capability(&warm,pet,name).support,Support::Unsupported,"No local-engine fallback for {pet}");}}
 let command=capability(&warm,"ai_lmstudio","tools.exactCommand");assert_eq!(command.availability,Availability::Unavailable);assert!(command.limitations.contains(&"explicitBindingRuntimeIntegrationPending"));
 let memory=capability(&warm,"ai_lmstudio","memory.goalContext");assert_eq!(memory.support,Support::Supported);assert!(memory.callers.contains(&Caller::TaskRuntime));assert_eq!(memory.reason,Some(Reason::TargetRequired));
 assert_eq!(capability(&warm,"ai_lmstudio","history.read").availability,Availability::Ready);assert!(!f.config.permissions("ai_lmstudio").unwrap().email_drafts);
 f.settings.chat_models.insert("lmstudio".into(),"unrelated-old-selection".into());let unchanged=f.snapshot(100_004);assert_eq!(descriptor(&warm,"ai_lmstudio").revision,descriptor(&unchanged,"ai_lmstudio").revision);
}
#[test]fn pet_inference_metadata_auth_contexts_do_not_share_anonymous_optional_or_required_evidence(){
 use crate::local_chat::{Server,lmstudio::{Catalog,MetadataAuth,endpoint_key_for}};
 let mut f=Fixture::new();f.config.version=2;let endpoint="http://127.0.0.1:18472";f.settings.lmstudio_url=endpoint.into();f.settings.chat_models.insert("lmstudio".into(),"local/free".into());f.config.pets.get_mut("ai_lmstudio").unwrap().provider="lmstudio".into();
 let p=f.config.pets.get_mut("ai_ollama").unwrap();p.inference=Some(crate::pet_inference::Choice{schema_version:1,backend:crate::pet_inference::Backend::Lmstudio,endpoint:endpoint.into(),model:"local/free".into(),credential:crate::pet_inference::CredentialReference::None});p.inference_revision=Some("a".repeat(32));
 let metadata=json!({"models":[{"type":"llm","key":"local/free","format":"gguf","size_bytes":1,"loaded_instances":[]}]});let mut server=Server{id:"lmstudio",name:"Private metadata",url:endpoint.into(),key:None};
 let seed=|f:&mut Fixture,auth:MetadataAuth,server:&Server,now|{let k=Key{provider:"lmstudio".into(),endpoint:endpoint_key_for(endpoint,auth).unwrap(),model:None};let t=f.registry.cache.begin(k,now).unwrap();assert!(f.registry.cache.finish(t,Evidence::LmStudio(Catalog::parse(server,&metadata).unwrap()),now));};
 seed(&mut f,MetadataAuth::Anonymous,&server,100_000);let anonymous=f.snapshot(100_001);assert_eq!(capability(&anonymous,"ai_ollama","conversation.respond").availability,Availability::Ready);assert_ne!(capability(&anonymous,"ai_lmstudio","conversation.respond").availability,Availability::Ready);
 let p=f.config.pets.get_mut("ai_ollama").unwrap();p.inference.as_mut().unwrap().credential=crate::pet_inference::CredentialReference::ConfiguredLmstudioEndpoint;p.inference_revision=Some("b".repeat(32));
 server.key=Some("synthetic-credential-not-a-real-secret".into());let generation=crate::local_chat::lmstudio_key_generation();seed(&mut f,MetadataAuth::Legacy(generation),&server,100_002);let optional=f.snapshot(100_003);assert_ne!(capability(&optional,"ai_ollama","conversation.respond").availability,Availability::Ready);assert_eq!(capability(&optional,"ai_lmstudio","conversation.respond").availability,Availability::Ready);
 seed(&mut f,MetadataAuth::Configured(generation),&server,100_004);let required=f.snapshot(100_005);assert_eq!(capability(&required,"ai_ollama","conversation.respond").availability,Availability::Ready);assert!(descriptor(&required,"ai_ollama").binding.engine_binding.is_some());assert!(!serde_json::to_string(&required).unwrap().contains(server.key.as_ref().unwrap()));
}
#[test]fn queued_consent_ignores_only_occupancy_and_keeps_unknown_model_and_preferences_barriers(){
 let mut f=Fixture::new();f.model(json!({"capabilities":["completion"]}),100_000);let busy=BTreeSet::from(["ollama".into()]);
 let live=f.registry.snapshot_with_io(&f.settings,Some(&f.config),true,&busy,100_001,true).unwrap();assert_eq!(capability(&live,"ai_ollama","conversation.respond").reason,Some(Reason::Busy));
 let admission=f.registry.admission_snapshot(&f.settings,Some(&f.config),true,100_001,true).unwrap();assert_eq!(capability(&admission,"ai_ollama","conversation.respond").availability,Availability::Ready);
 let live=f.registry.snapshot_with_io(&f.settings,Some(&f.config),true,&busy,100_001,true).unwrap();assert_eq!(capability(&live,"ai_ollama","conversation.respond").reason,Some(Reason::Busy));
 let stale=f.registry.admission_snapshot(&f.settings,Some(&f.config),true,160_000,true).unwrap();assert_ne!(capability(&stale,"ai_ollama","conversation.respond").availability,Availability::Ready);
 let blocked=f.registry.admission_snapshot(&f.settings,Some(&f.config),true,100_001,false).unwrap();assert_eq!(capability(&blocked,"ai_ollama","conversation.respond").reason,Some(Reason::SettingsUnconfirmed));
 f.model(json!({"capabilities":[]}),100_002);let unsupported=f.registry.admission_snapshot(&f.settings,Some(&f.config),true,100_003,true).unwrap();assert_ne!(capability(&unsupported,"ai_ollama","conversation.respond").availability,Availability::Ready);
}
#[test]fn settings_effect_capabilities_gate_new_effects_without_changing_support_or_preservation(){
 let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"]}),100_000);let before=f.snapshot(100_001);
 let blocked=f.registry.snapshot_with_io(&f.settings,Some(&f.config),true,&BTreeSet::new(),100_001,false).unwrap();
 for id in ["ai_ollama","agent_codex","agent_antigravity","agent_copilot"]{
  let a=capability(&before,id,"conversation.respond");let b=capability(&blocked,id,"conversation.respond");assert_eq!(b.support,a.support);assert_eq!(b.reason,Some(Reason::SettingsUnconfirmed));assert_eq!(b.availability,Availability::Unavailable);
  for name in ["history.read","conversation.cancel"]{assert_eq!(serde_json::to_value(capability(&before,id,name)).unwrap(),serde_json::to_value(capability(&blocked,id,name)).unwrap());}
 }
 assert_ne!(before.revision,blocked.revision);let ready=f.snapshot(100_001);assert_ne!(blocked.revision,ready.revision);
 assert_eq!(capability(&ready,"ai_ollama","conversation.respond").reason,capability(&before,"ai_ollama","conversation.respond").reason);
}
#[test]fn configured_service_assistant_keeps_logical_identity_and_native_audio_capabilities(){
    let mut f=Fixture::new();f.config.pets.get_mut("integration_spotify").unwrap().provider="codex".into();let s=f.snapshot(100_000);
    let d=descriptor(&s,"integration_spotify");assert_eq!(d.binding.adapter,Some("codex"));assert_eq!(d.binding.provider.as_deref(),Some("integration_spotify"));
    assert_eq!(capability(&s,"integration_spotify","conversation.respond").support,Support::Supported);
    for cap in ["service.musicControl","service.audioControl"]{let c=capability(&s,"integration_spotify",cap);assert_eq!(c.support,Support::Supported);assert!(c.limitations.contains(&"notSystemPluginGrant"));}
    assert_eq!(capability(&s,"integration_spotify","system.audio").support,Support::Unsupported);
    f.config.pets.get_mut("integration_spotify").unwrap().permissions.as_mut().unwrap().music=false;let s=f.snapshot(100_001);assert_eq!(capability(&s,"integration_spotify","service.audioControl").reason,Some(Reason::PermissionDenied));
}
#[test]fn declared_thinking_controls_share_the_existing_request_parser(){
    for values in [json!(["brief","thorough"]),json!([true,false])]{let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"],"thinking":{"values":values}}),100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","models.reasoning").support,Support::Supported);}
    let mut f=Fixture::new();f.model(json!({"thinking":{"values":[false]}}),100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","models.reasoning").support,Support::Unsupported);
    let mut f=Fixture::new();f.model(json!({"thinking":{"values":[9]}}),100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","models.reasoning").support,Support::Unknown);
}
#[test]fn model_identifiers_cannot_export_paths_or_credential_urls(){
    for id in ["/home/private/model","../private/model","~/private","C:\\private\\model","https://user:token@host/model","@auto","model\nname","model name"]{assert!(!cache::model_id(id),"{id}");}
    for id in ["namespace/model:latest","synthetic.gguf","model-v1.2"]{assert!(cache::model_id(id));}
}
#[test]fn registry_resolves_all_existing_ids_and_keeps_only_github_entity_retired(){
    let mut f=Fixture::new();let s=f.snapshot(100_000);assert_eq!(s.pets.len(),26);assert_eq!(s.pets.iter().filter(|p|p.lifecycle==Lifecycle::Retired).map(|p|p.pet_id.as_str()).collect::<Vec<_>>(),vec!["integration_github"]);
    for id in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","ai_anthropic","ai_google"]{assert_eq!(descriptor(&s,id).kind,Kind::Llm);}
    assert_eq!(descriptor(&s,"agent_cursor").kind,Kind::Observer);assert_eq!(descriptor(&s,"integration_spotify").kind,Kind::Service);assert_eq!(capability(&s,"integration_github","conversation.respond").reason,Some(Reason::RetiredPet));assert_eq!(capability(&s,"integration_github","history.read").availability,Availability::Ready);
    assert_eq!(capability(&s,"agent_antigravity","session.approve").support,Support::Unknown);assert!(capability(&s,"agent_antigravity","session.approve").limitations.contains(&"interactiveContinuationUnverified"));
}
#[test]fn cold_metadata_is_unknown_and_never_claims_autonomy_or_implicit_llm_service(){
    let mut f=Fixture::new();let s=f.snapshot(100_000);let c=capability(&s,"ai_ollama","tools.exactCommand");assert_eq!(c.support,Support::Unknown);assert_eq!(c.availability,Availability::Unknown);assert!(c.observed_at.is_none());
    for pet in &s.pets{for id in ["goals.run","rules.schedule","rules.events","delegation.send","memory.retrieve","system.audio"]{let c=capability(&s,&pet.pet_id,id);assert_eq!(c.support,Support::Unsupported);assert_eq!(c.reason,Some(Reason::NotImplemented));}}
    assert_eq!(capability(&s,"integration_spotify","conversation.respond").support,Support::Unsupported);assert_eq!(capability(&s,"integration_spotify","models.tools").support,Support::Unsupported);assert_eq!(capability(&s,"agent_cursor","history.read").support,Support::Unsupported);
}
#[test]fn configured_global_api_models_do_not_bind_an_empty_pet_provider_to_codex(){
    let mut f=Fixture::new();f.settings.chat_models.insert("openai".into(),"synthetic-api-model".into());f.settings.lmstudio_url="http://127.0.0.1:1234".into();let s=f.snapshot(100_000);
    for id in ["ai_lmstudio","ai_openai","ai_anthropic","ai_google"]{let p=descriptor(&s,id);assert!(p.binding.adapter.is_none());assert!(p.binding.provider.is_none());assert_eq!(capability(&s,id,"conversation.respond").reason,Some(Reason::NotConfigured));assert_ne!(capability(&s,id,"conversation.respond").availability,Availability::Ready);}
    assert_eq!(descriptor(&s,"integration_notion").binding.adapter,Some("codex"));assert_eq!(descriptor(&s,"integration_notion").binding.provider.as_deref(),Some("integration_notion"));
}
#[test]fn exact_ollama_model_observation_can_confirm_tools_without_minting_execution_permission(){
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools","thinking"]}),100_000);let s=f.snapshot(100_001);let c=capability(&s,"ai_ollama","tools.exactCommand");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Ready);assert_eq!(c.decision,Decision::ExactCommandApproval);assert_eq!(c.model_binding.as_deref(),Some("synthetic-model"));assert_eq!(c.observed_at,Some(100_000));assert_eq!(c.expires_at,Some(160_000));assert_eq!(c.limits.max_calls,Some(8));assert!(c.limitations.contains(&"desktopUserNotFilesystemSandbox"));assert_eq!(capability(&s,"ai_ollama","models.reasoning").support,Support::Supported);
    assert_ne!(capability(&s,"agent_codex","tools.native").availability,Availability::Ready);
}
#[test]fn unavailable_malformed_stale_or_changed_metadata_does_not_become_supported(){
    for body in [json!({}),json!({"capabilities":"tools"}),json!({"capabilities":["completion",9]})]{let mut f=Fixture::new();f.model(body,100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").support,Support::Unknown);}
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"]}),100_000);let s=f.snapshot(160_000);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::StaleObservation));assert_ne!(capability(&s,"ai_ollama","tools.exactCommand").availability,Availability::Ready);
    let s=f.snapshot(99_999);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::StaleObservation));
}
#[test]fn explicit_absence_of_tools_remote_models_and_disabled_policy_are_distinct(){
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion"]}),100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").support,Support::Unsupported);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::ModelUnsupported));assert_eq!(capability(&s,"ai_ollama","conversation.respond").availability,Availability::Ready);
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"],"remote_host":"private-remote.invalid"}),100_000);let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::PolicyDenied));assert_ne!(capability(&s,"ai_ollama","tools.exactCommand").availability,Availability::Ready);
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"]}),100_000);f.config.pets.get_mut("ai_ollama").unwrap().local_execution.as_mut().unwrap().mode=crate::pet_config::LocalExecutionMode::Disabled;let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::PolicyDenied));
}
#[test]fn endpoint_model_or_profile_changes_cannot_reuse_another_model_observation(){
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"]}),100_000);let before=f.snapshot(100_001);f.settings.chat_models.insert("ollama".into(),"another-model".into());let after=f.snapshot(100_002);assert_ne!(descriptor(&before,"ai_ollama").revision,descriptor(&after,"ai_ollama").revision);assert_ne!(capability(&after,"ai_ollama","tools.exactCommand").availability,Availability::Ready);
    f.settings.chat_models.insert("ollama".into(),"synthetic-model".into());f.settings.ollama_url="http://127.0.0.1:11435".into();let s=f.snapshot(100_003);assert_ne!(capability(&s,"ai_ollama","tools.exactCommand").availability,Availability::Ready);
    f.settings.chat_models.insert("ollama".into(),"@auto".into());let s=f.snapshot(100_004);assert!(descriptor(&s,"ai_ollama").binding.model.is_none());assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::ModelUnresolved));
}
#[test]fn roster_permissions_busy_state_and_library_health_have_honest_barriers(){
    let mut f=Fixture::new();f.model(json!({"capabilities":["completion","tools"]}),100_000);f.settings.active_integrations.retain(|p|p!="ai_ollama");let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","tools.exactCommand").reason,Some(Reason::InactivePet));f.settings.active_integrations.push("ai_ollama".into());
    f.config.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().documents=false;let s=f.snapshot(100_002);assert_eq!(capability(&s,"ai_ollama","documents.text").reason,Some(Reason::PermissionDenied));
    let s=f.registry.snapshot(&f.settings,Some(&f.config),true,&BTreeSet::from(["ollama".into()]),100_003).unwrap();assert_eq!(capability(&s,"ai_ollama","conversation.respond").reason,Some(Reason::Busy));
    let s=f.registry.snapshot(&f.settings,Some(&f.config),false,&BTreeSet::new(),100_004).unwrap();assert_eq!(capability(&s,"ai_ollama","conversation.respond").reason,Some(Reason::LibraryUnavailable));
}
#[test]fn descriptor_revisions_are_stable_isolated_and_include_private_policy_changes(){
    let mut f=Fixture::new();let a=f.snapshot(100_000);let b=f.snapshot(100_001);assert_eq!(a.revision,b.revision);assert_eq!(descriptor(&a,"ai_ollama").revision,descriptor(&b,"ai_ollama").revision);
    f.config.pets.get_mut("ai_ollama").unwrap().local_execution.as_mut().unwrap().allowed_programs=Some(vec!["/tmp/private-allowed-tool".into()]);let c=f.snapshot(100_002);assert_ne!(descriptor(&b,"ai_ollama").revision,descriptor(&c,"ai_ollama").revision);assert_eq!(descriptor(&b,"agent_codex").revision,descriptor(&c,"agent_codex").revision);assert!(!serde_json::to_string(&c).unwrap().contains("/tmp/private-allowed-tool"));
    let mut other=Fixture::new();assert_ne!(a.revision,other.snapshot(100_003).revision);
}
#[test]fn no_snapshot_contains_endpoints_credentials_or_ungranted_management_tools(){
    let mut f=Fixture::new();f.settings.ollama_url="https://secret-user:secret-pass@private.invalid/private-endpoint".into();let s=f.snapshot(100_000);let text=serde_json::to_string(&s).unwrap();for secret in ["secret-user","secret-pass","private.invalid","private-endpoint"]{assert!(!text.contains(secret));}
    for id in ["documents.renew","documents.retention"]{let c=capability(&s,"agent_codex",id);assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.decision,Decision::NativeApproval);assert_ne!(c.availability,Availability::Ready);}
    for p in &s.pets{assert!(p.capabilities.len()<=128);assert_eq!(p.capabilities.iter().map(|c|c.id).collect::<BTreeSet<_>>().len(),p.capabilities.len());}
}
#[test]fn catalog_cache_ordering_expiry_eviction_and_request_deadline_are_bounded(){
    let mut cache=Cache::default();let key=Key{provider:"codex".into(),endpoint:String::new(),model:None};let first=cache.begin(key.clone(),100_000).unwrap();let second=cache.begin(key.clone(),100_001).unwrap();assert!(!cache.finish(first,Evidence::Catalog,100_002));assert!(cache.finish(second,Evidence::Catalog,100_003));assert!(matches!(cache.get(&key,100_004).evidence,Some(Evidence::Catalog)));assert!(cache.get(&key,160_003).stale);
    let late=cache.begin(key.clone(),200_000).unwrap();assert!(!cache.finish(late,Evidence::Catalog,230_001));assert!(cache.get(&key,230_002).evidence.is_none());
    let evicted=cache.begin(key.clone(),300_000).unwrap();for n in 0..64{cache.begin(Key{provider:format!("synthetic-{n}"),endpoint:String::new(),model:None},300_001+n).unwrap();}assert!(!cache.finish(evicted,Evidence::Catalog,300_100));assert!(cache.get(&key,300_101).evidence.is_none());
    assert!(cache.begin(key.clone(),0).is_none());assert!(cache.begin(key,cache::MAX_DATE+1).is_none());
}
#[test]fn catalog_observation_is_not_authentication_or_permission_to_execute(){
    let mut f=Fixture::new();let b=binding(&f.settings,"agent_codex",&f.config.pets["agent_codex"],true);let t=f.registry.cache.begin(key(&f.settings,&b,&f.config.pets["agent_codex"],None).unwrap(),100_000).unwrap();assert!(f.registry.cache.finish(t,Evidence::Catalog,100_001));let s=f.snapshot(100_002);assert_eq!(capability(&s,"agent_codex","models.list").availability,Availability::Ready);assert_ne!(capability(&s,"agent_codex","conversation.respond").availability,Availability::Ready);assert_ne!(capability(&s,"agent_codex","tools.native").availability,Availability::Ready);
}
#[test]fn missing_configuration_keeps_unknown_state_and_does_not_write_defaults_or_create_conversations(){
    let mut f=Fixture::new();let root=std::env::temp_dir().join(format!("coucou-capability-read-{}",crate::assistant_memory::new_id()));let path=root.join("missing/pets.json");let config=crate::pet_config::read_existing_at(&path).ok();assert!(config.is_none());let chat=crate::chat::Chat::default();let(healthy,busy)=chat.capability_health();let s=f.registry.snapshot(&f.settings,config.as_ref(),healthy,&busy,100_000).unwrap();assert!(!s.configuration_available);assert!(!root.exists());assert!(chat.tasks().is_empty());assert_eq!(capability(&s,"ai_ollama","conversation.respond").reason,Some(Reason::NotConfigured));assert_ne!(capability(&s,"ai_ollama","tools.exactCommand").availability,Availability::Ready);
}
#[test]fn malformed_identity_policy_or_clock_cannot_publish_a_fabricated_descriptor(){
    let mut f=Fixture::new();f.config.pets.get_mut("ai_ollama").unwrap().provider="codex".into();assert!(f.registry.snapshot(&f.settings,Some(&f.config),true,&BTreeSet::new(),100_000).is_err());
    let mut f=Fixture::new();f.config.pets.get_mut("ai_ollama").unwrap().local_execution.as_mut().unwrap().max_calls=999;assert!(f.registry.snapshot(&f.settings,Some(&f.config),true,&BTreeSet::new(),100_000).is_err());
    assert!(f.registry.snapshot(&f.settings,None,true,&BTreeSet::new(),0).is_err());assert!(f.registry.snapshot(&f.settings,None,true,&BTreeSet::new(),cache::MAX_DATE+1).is_err());
}
#[test]fn busy_snapshot_is_passive_and_stop_remains_target_bound(){
    let chat=crate::chat::Chat::default();let turn=chat.begin_cli("codex",Default::default()).unwrap();let lease=chat.lease(&turn);let(healthy,busy)=chat.capability_health();assert!(healthy);assert!(busy.contains("codex"));assert!(!busy.contains("ollama"));assert!(chat.is_current(&turn));drop(lease);
    let mut f=Fixture::new();let s=f.snapshot(100_000);let c=capability(&s,"agent_codex","tasks.stop");assert_eq!(c.reason,Some(Reason::TargetRequired));assert_ne!(c.availability,Availability::Ready);
}
#[test]fn only_existing_native_management_surfaces_can_read_descriptors(){
    for label in [crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::companions::LABEL,crate::pet_chat::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL,"settings"]{assert!(allowed(label));}
    for label in ["untrusted","pet-chat-extra","","external-web"]{assert!(!allowed(label));}
}

#[test]fn deterministic_spotify_voice_is_implemented_without_claiming_model_or_system_authority(){
    let mut f=Fixture::new();let s=f.snapshot(100_000);
    for id in ["voice.dictate","voice.respond","voice.interrupt"]{let c=capability(&s,"integration_spotify",id);assert_eq!(c.support,Support::Supported);assert_ne!(c.availability,Availability::Ready);assert_eq!(c.callers,vec![Caller::NativeUser]);}
    assert_eq!(capability(&s,"integration_spotify","voice.decisions").support,Support::Unknown);
    assert_eq!(capability(&s,"integration_spotify","system.audio").support,Support::Unsupported);
    assert_eq!(capability(&s,"integration_spotify","conversation.respond").support,Support::Unsupported);
    assert!(descriptor(&s,"integration_spotify").binding.model.is_none());
    for pet in ["agent_gemini","integration_claude","ai_openai"]{assert_eq!(capability(&s,pet,"voice.respond").support,Support::Unsupported);}
}

#[test]fn draft_runtime_metadata_never_grants_autonomous_execution_or_model_access(){
    let mut f=Fixture::new();let s=f.snapshot(100_000);
    for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","integration_notion"]{
        for id in ["goals.draft","goals.inspect"]{let c=capability(&s,pet,id);assert_eq!(c.support,Support::Supported);assert_eq!(c.callers,vec![Caller::NativeUser]);assert_ne!(c.availability,Availability::Ready);}
        assert_eq!(capability(&s,pet,"goals.run").support,Support::Unsupported);
    }
    assert_eq!(capability(&s,"agent_cursor","goals.draft").support,Support::Unsupported);
    assert_eq!(capability(&s,"integration_github","goals.draft").reason,Some(Reason::RetiredPet));
}

#[test]fn real_local_registry_refresh_changes_public_epoch_but_preserves_admission_semantics(){
 let mut f=Fixture::new();let metadata=json!({"capabilities":["completion","tools"],"model_info":{"fixture":true}});f.model(metadata.clone(),100_000);let first=f.snapshot(100_001);let before=descriptor(&first,"ai_ollama");f.model(metadata,100_010);let second=f.snapshot(100_011);let after=descriptor(&second,"ai_ollama");assert_ne!(before.revision,after.revision);assert_ne!(before.observed_at,after.observed_at);assert_eq!(admission_semantics(before).unwrap(),admission_semantics(after).unwrap());
 let mut changed=after.clone();changed.capabilities.iter_mut().find(|c|c.id=="conversation.respond").unwrap().limits.max_calls=Some(1);assert_ne!(admission_semantics(before).unwrap(),admission_semantics(&changed).unwrap());let mut stale=after.clone();stale.capabilities.iter_mut().find(|c|c.id=="conversation.respond").unwrap().availability=Availability::Unknown;assert_ne!(admission_semantics(before).unwrap(),admission_semantics(&stale).unwrap());
}

#[test]fn bounded_planning_execution_and_verification_are_declared_only_for_the_implemented_native_adapter(){
    let mut f=Fixture::new();f.config.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=true;let s=f.snapshot(100_000);
    for id in ["goals.planBounded","goals.executeRecipe","goals.verifyCriteria"] {
        let c=capability(&s,"ai_ollama",id);assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.decision,Decision::ScopedGrant);assert!(c.limitations.contains(&"spotifyTransportV1Only"));
        for pet in ["agent_codex","agent_copilot","agent_antigravity","integration_spotify","ai_lmstudio"]{assert_eq!(capability(&s,pet,id).support,Support::Unsupported);}
    }
    f.config.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;let s=f.snapshot(100_001);assert_eq!(capability(&s,"ai_ollama","goals.planBounded").reason,Some(Reason::PermissionDenied));
}

#[test]fn continuation_review_is_common_native_provenance_without_enabling_other_provider_execution(){
    let mut f=Fixture::new();let s=f.snapshot(100_000);
    for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama"]{
        let c=capability(&s,pet,"goals.continuationReview");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.decision,Decision::NativeApproval);assert_eq!(c.callers,vec![Caller::NativeUser]);
        for bound in ["authenticatedClosedSourceRequired","ordinaryAdditionalAllowanceOrFundedRemainingOriginalAllowance","unknownToolEffectsRequireReconciliation","doesNotEnableProviderExecution"]{assert!(c.limitations.contains(&bound));}
        if pet!="ai_ollama"{for id in ["goals.planBounded","goals.executeRecipe","goals.verifyCriteria"]{assert_eq!(capability(&s,pet,id).support,Support::Unsupported);}}
    }
    for pet in ["integration_spotify","ai_lmstudio"]{assert_eq!(capability(&s,pet,"goals.continuationReview").support,Support::Unsupported);}
}

#[test]fn unused_authority_retirement_review_is_common_native_reduction_without_an_execution_grant(){
    let mut fixture=Fixture::new();let snapshot=fixture.snapshot(100_000);
    for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama"]{
        let c=capability(&snapshot,pet,"goals.retirementReview");assert_eq!(c.support,Support::Supported);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.decision,Decision::NativeApproval);assert_eq!(c.callers,vec![Caller::NativeUser]);
        for limit in ["signedUnusedCanonicalGrantRequired","noLinkedAdmission","nativeNoIoEvidenceRequired","authorityReductionOnly","doesNotEnableProviderExecution"]{assert!(c.limitations.contains(&limit));}
    }
    for pet in ["integration_spotify","ai_lmstudio"]{assert_eq!(capability(&snapshot,pet,"goals.retirementReview").support,Support::Unsupported);}
}

#[path="pet_memory_capabilities_tests.rs"]mod memory_projection_tests;

#[test]fn deterministic_runtime_capability_uses_native_identity_not_model_readiness_and_never_exposes_a_provider_tool(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let s=f.snapshot(100_001);
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  for id in ["goals.deterministicAdmission","goals.messageDeliver"]{let c=capability(&s,pet,id);assert_eq!(c.support,Support::Supported,"{pet}/{id}");assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert!(c.model_binding.is_none());assert!(!c.callers.contains(&Caller::ProviderModel));assert!(c.limitations.contains(&"recipientExecutionNotAuthorized"));}
  assert_eq!(capability(&s,pet,"goals.run").support,Support::Unsupported);
 }
 for pet in ["agent_cursor","integration_github"]{assert_eq!(capability(&s,pet,"goals.deterministicAdmission").support,Support::Unsupported);}
 f.settings.active_integrations.retain(|p|p!="ai_openai");let inactive=f.snapshot(100_002);assert_eq!(capability(&inactive,"ai_openai","goals.deterministicAdmission").reason,Some(Reason::InactivePet));
}

#[test]fn native_plan_revision_capability_is_human_data_only_independent_of_inference_and_activation(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let before=f.snapshot(100_001);
 let pets=["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"];
 for pet in pets{let c=capability(&before,pet,"goals.nativePlanRevision");assert_eq!(c.support,Support::Supported,"{pet}");assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.decision,Decision::NativeApproval);assert!(c.model_binding.is_none());assert_eq!(c.limits.max_items,Some(16));assert_eq!(c.limits.max_bytes,Some(256*1024));assert!(c.limitations.contains(&"dataOnlyNoBudgetWorkerOrInference"));assert!(c.limitations.contains(&"separateExecutionConsentRequired"));assert!(c.limitations.contains(&"coveredCausesAndReceiptsImmutable"));}
 for pet in ["agent_cursor","integration_github"]{assert_eq!(capability(&before,pet,"goals.nativePlanRevision").support,Support::Unsupported);}
 f.settings.active_integrations.clear();f.settings.main_pill="agent_cursor".into();let inactive=f.registry.admission_snapshot(&f.settings,Some(&f.config),true,100_002,false).unwrap();
 for pet in pets{assert_eq!(descriptor(&inactive,pet).lifecycle,Lifecycle::Inactive);let c=capability(&inactive,pet,"goals.nativePlanRevision");assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert!(c.model_binding.is_none());assert_eq!(capability(&inactive,pet,"goals.deterministicAdmission").reason,Some(Reason::SettingsUnconfirmed));}
 let unhealthy=f.registry.snapshot(&f.settings,Some(&f.config),false,&BTreeSet::new(),100_003).unwrap();assert_eq!(capability(&unhealthy,"ai_ollama","goals.nativePlanRevision").reason,Some(Reason::LibraryUnavailable));
 assert_eq!(capability(&inactive,"integration_github","goals.nativePlanRevision").reason,Some(Reason::RetiredPet));
}

#[test]fn correlated_reply_capability_is_native_data_with_own_authority_and_no_provider_model_tool(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let s=f.snapshot(100_001);
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{let c=capability(&s,pet,"goals.messageReply");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert!(c.model_binding.is_none());assert!(c.limitations.contains(&"ownAdmissionAndExactHumanReviewRequired"));assert!(c.limitations.contains(&"sourceDeadlineCannotBeExtended"));assert!(c.limitations.contains(&"replyDoesNotAcceptOrExecuteParent"));}
 for pet in ["agent_cursor","integration_github"]{assert_eq!(capability(&s,pet,"goals.messageReply").support,Support::Unsupported);}f.settings.active_integrations.retain(|p|p!="ai_lmstudio");let s=f.snapshot(100_002);assert_eq!(capability(&s,"ai_lmstudio","goals.messageReply").reason,Some(Reason::InactivePet));
}

#[test]fn received_request_capability_is_native_data_only_without_inference_readiness(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let s=f.snapshot(100_001);
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{let c=capability(&s,pet,"goals.receivedRequest");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert!(c.model_binding.is_none());assert!(c.limitations.contains(&"ownReviewedGoalDataOnly"));}
 for pet in ["agent_cursor","integration_github"]{assert_eq!(capability(&s,pet,"goals.receivedRequest").support,Support::Unsupported);}f.settings.active_integrations.retain(|p|p!="ai_lmstudio");assert_eq!(capability(&f.snapshot(100_002),"ai_lmstudio","goals.receivedRequest").reason,Some(Reason::InactivePet));
}

#[test]fn result_plan_data_review_never_becomes_disclosure_or_a_provider_model_tool(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let s=f.snapshot(100_001);
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama"]{let c=capability(&s,pet,"goals.resultReplyPlan");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.decision,Decision::NativeApproval);assert!(c.model_binding.is_none());assert_eq!(c.limits.max_items,Some(1));for bound in["authenticatedOwnResultReceiptAndExactUtf8Excerpt","originalDelegationRecipientAndDeadline","planDataOnlyNoDisclosureOrExecutionConsent","derivedDeliveryRequiresSeparateSignedDisclosureAdmission"]{assert!(c.limitations.contains(&bound));}}
 for pet in["agent_cursor","integration_github","integration_spotify"]{assert_eq!(capability(&s,pet,"goals.resultReplyPlan").support,Support::Unsupported);}
}

#[test]fn recursive_delegation_projects_own_native_review_only_without_promoting_model_task_runtime_or_provider_availability(){
 let mut f=Fixture::new();let s=f.snapshot(100_001);for pet in["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{let c=capability(&s,pet,"goals.recursiveDelegation");assert_eq!(c.support,Support::Supported,"{pet}");assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.decision,Decision::NativeApproval);assert!(c.model_binding.is_none());assert_eq!(c.limits.max_items,Some(16));for limit in["canonicalAncestorsDepth8Nodes128AndNoCycles","sameOriginalAllowanceAndDeadline;noBudgetRenewal","originalWorkersMustDrainBeforeCreditOrPhysicalRelease","nestedInferenceRequiresOwnExactF2Consent;recursiveContinuationAndPlanRevisionNotImplemented"]{assert!(c.limitations.contains(&limit));}}
}

#[test]fn recursive_response_capability_is_native_review_only_and_never_advertises_unknown_local_provider_as_ready(){
 let mut f=Fixture::new();f.config.version=2;for(pet,backend,port)in[("ai_ollama",crate::pet_inference::Backend::Ollama,18471),("ai_lmstudio",crate::pet_inference::Backend::Lmstudio,18472)]{let p=f.config.pets.get_mut(pet).unwrap();p.inference=Some(crate::pet_inference::Choice{schema_version:1,backend,endpoint:format!("http://127.0.0.1:{port}"),model:"synthetic-chosen".into(),credential:crate::pet_inference::CredentialReference::None});p.inference_revision=Some("f".repeat(32));}let s=f.snapshot(100_001);for pet in["ai_ollama","ai_lmstudio"]{let c=capability(&s,pet,"goals.recursiveResponse");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.decision,Decision::NativeApproval);assert_eq!(c.limits.max_calls,Some(1));assert!(c.limitations.contains(&"ownSelectedRecursiveSourceAndExactF2GoalConsent"));assert!(c.limitations.contains(&"oneOwnModelCallZeroToolsNoContinuationOrImplicitDisclosure"));}
 for pet in["agent_codex","agent_copilot","agent_antigravity","ai_openai","integration_spotify"]{let c=capability(&s,pet,"goals.recursiveResponse");assert_eq!(c.support,Support::Unsupported);assert_ne!(c.availability,Availability::Ready);}
}

#[test]fn requested_finite_execution_requires_own_native_review_and_never_projects_provider_model_authority(){
 let mut f=Fixture::new();f.config.version=2;for(pet,backend,port)in[("ai_ollama",crate::pet_inference::Backend::Ollama,18471),("ai_lmstudio",crate::pet_inference::Backend::Lmstudio,18472)]{let p=f.config.pets.get_mut(pet).unwrap();p.inference=Some(crate::pet_inference::Choice{schema_version:1,backend,endpoint:format!("http://127.0.0.1:{port}"),model:"synthetic-chosen".into(),credential:crate::pet_inference::CredentialReference::None});p.inference_revision=Some("f".repeat(32));}let s=f.snapshot(100_001);
 for pet in ["ai_ollama","ai_lmstudio"]{for id in ["goals.executionStartReview","goals.requestedExecution"]{let c=capability(&s,pet,id);assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert!(!c.callers.contains(&Caller::ProviderModel));assert_eq!(c.callers.contains(&Caller::TaskRuntime),id=="goals.requestedExecution");assert!(c.limitations.contains(&"oldManualAdmissionsNeverPromoted"));}}
 for pet in ["agent_codex","agent_copilot","agent_antigravity","integration_spotify","integration_github"]{assert_eq!(capability(&s,pet,"goals.executionStartReview").support,Support::Unsupported);}
}

#[test]fn system_access_projection_separates_task_from_model_and_preserves_per_pet_grants(){
 use crate::pet_config::system_access::{Policy,Grant,Operation,Subplugin,Resource};let mut f=Fixture::new();let before=f.snapshot(100_000);assert_eq!(capability(&before,"integration_spotify","system.sound.observe").reason,Some(Reason::PermissionDenied));f.config.version=2;
 for pet in["ai_ollama","integration_spotify"]{f.config.pets.get_mut(pet).unwrap().system_access=Some(Policy{schema_version:1,enabled:true,revision:if pet=="ai_ollama"{"a".repeat(32)}else{"b".repeat(32)},grants:[Operation::Observe,Operation::Volume].into_iter().map(|operation|Grant{subplugin:Subplugin::Sound,operation,resource:Resource::AudioOutputs{}}).collect()});}
 let both=f.snapshot(100_001);for pet in["ai_ollama","integration_spotify"]{let c=capability(&both,pet,"system.sound.volume");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert!(!c.callers.contains(&Caller::ProviderModel));assert_eq!(c.decision,Decision::ScopedGrant);assert!(c.model_binding.is_none());assert!(c.limits.timeout_ms.is_none(),"per-helper timeout is not an aggregate operation deadline");assert_eq!(capability(&both,pet,"system.sound.mute").reason,Some(Reason::PermissionDenied));}
 let other=serde_json::to_value(descriptor(&both,"ai_ollama")).unwrap();f.config.pets.get_mut("integration_spotify").unwrap().system_access.as_mut().unwrap().grants.clear();let revoked=f.snapshot(100_002);assert_eq!(capability(&revoked,"integration_spotify","system.sound.volume").reason,Some(Reason::PermissionDenied));assert_eq!(serde_json::to_value(descriptor(&revoked,"ai_ollama")).unwrap(),other);assert_eq!(capability(&revoked,"agent_cursor","system.sound.observe").support,Support::Unsupported);
}

#[test]fn native_system_admission_stays_native_user_only_and_task_effects_require_original_scope(){
 let mut f=Fixture::new();let snapshot=f.snapshot(100_000);let c=capability(&snapshot,"ai_ollama","goals.systemAdmission");assert_eq!(c.support,Support::Supported);assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.availability,Availability::Unknown);assert!(c.model_binding.is_none());assert!(c.limitations.contains(&"admissionOnlyNoAutomaticSystemDispatch"));assert!(!c.limitations.contains(&"nativeLiteralPeerDataOnly"));for id in["system.sound.volume","system.sound.mute"]{let c=capability(&snapshot,"ai_ollama",id);assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert!(c.limitations.contains(&"taskRequiresOriginalDeterministicSystemAdmissionAndLiveOwner"));assert!(!c.callers.contains(&Caller::ProviderModel));}for id in["system.access.manage","system.sound.observe"]{assert_eq!(capability(&snapshot,"ai_ollama",id).callers,vec![Caller::NativeUser]);}
}

#[test]fn host_inventory_capability_is_passive_scoped_runtime_and_health_is_not_authority(){
 use crate::pet_config::system_access::{Policy,Grant,Operation,Subplugin,Resource};let mut f=Fixture::new();f.settings.active_integrations.push("agent_system_guardian".into());f.config.version=2;
 f.config.pets.get_mut("agent_system_guardian").unwrap().system_access=Some(Policy{schema_version:1,enabled:true,revision:"a".repeat(32),grants:vec![Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::HostHealth{}}]});
 let denied=f.snapshot(100_000);assert_eq!(capability(&denied,"agent_system_guardian","system.inventory.observe").reason,Some(Reason::PermissionDenied));
 let pet=f.config.pets.get_mut("agent_system_guardian").unwrap();pet.system_access.as_mut().unwrap().grants=vec![Grant{subplugin:Subplugin::Inventory,operation:Operation::Observe,resource:Resource::HostComponents{}}];assert!(pet.inference.is_none());
 let snapshot=f.snapshot(100_001);let c=capability(&snapshot,"agent_system_guardian","system.inventory.observe");assert_eq!(c.support,Support::Supported);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert!(c.model_binding.is_none());assert_eq!(c.limits.max_items,Some(2));assert_eq!(capability(&snapshot,"agent_system_guardian","system.health.observe").reason,Some(Reason::PermissionDenied));
}


#[test]fn persistent_collection_management_is_native_user_metadata_without_inference_or_implicit_query_authority(){
 let mut f=Fixture::new();f.settings.chat_models.clear();let snapshot=f.snapshot(100_000);
 for pet in ["agent_system_guardian","agent_codex","agent_antigravity","agent_copilot","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  let c=capability(&snapshot,pet,"sources.calendar.manage");assert_eq!(c.support,Support::Supported);assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.availability,Availability::Unknown);assert!(c.model_binding.is_none());assert_eq!(c.reason,Some(Reason::TargetRequired));assert!(c.limitations.contains(&"metadataOnlyNoFileOrModelIo"));
 }
 assert_eq!(capability(&snapshot,"integration_github","sources.calendar.manage").support,Support::Unsupported);
}

#[test]fn goal_calendar_context_projects_native_and_runtime_only_with_explicit_target_and_no_implicit_engine(){
 let mut f=Fixture::new();let cold=f.snapshot(100_000);
 for pet in["ai_ollama"]{let c=capability(&cold,pet,"sources.calendar.goalContext");assert_eq!(c.support,Support::Supported);assert_eq!(c.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));assert_eq!(c.limits.max_items,Some(8));assert!(c.limitations.contains(&"sourceConversationIsOriginNotAutonomousConsent"));}
 for pet in["ai_lmstudio","agent_codex","agent_antigravity","agent_copilot","agent_system_guardian","ai_openai","integration_github"]{assert_eq!(capability(&cold,pet,"sources.calendar.goalContext").support,Support::Unsupported);}
 f.config.version=2;let p=f.config.pets.get_mut("ai_lmstudio").unwrap();p.inference=Some(crate::pet_inference::Choice{schema_version:1,backend:crate::pet_inference::Backend::Ollama,endpoint:"http://127.0.0.1:18477".into(),model:"explicit-free".into(),credential:crate::pet_inference::CredentialReference::None});p.inference_revision=Some("a".repeat(32));let s=f.snapshot(100_001);let d=descriptor(&s,"ai_lmstudio");assert_eq!(d.binding.provider.as_deref(),Some("lmstudio"));assert_eq!(d.binding.adapter,Some("ollama"));let c=capability(&s,"ai_lmstudio","sources.calendar.goalContext");assert_eq!(c.support,Support::Supported);assert_eq!(c.model_binding.as_deref(),Some("explicit-free"));assert_eq!(c.availability,Availability::Unknown);assert!(!c.callers.contains(&Caller::ProviderModel));
}

#[test]fn rule_metadata_reuse_requires_same_installed_model_freshness_and_latest_observation(){
 let mut cache=cache::Cache::default();let key=cache::Key{provider:"ollama".into(),endpoint:"http://127.0.0.1:11434".into(),model:Some("fixture-free".into())};
 assert!(!cache.fresh_installed_local_model(&key,100_000));
 let valid=serde_json::json!({"model_info":{"general.architecture":"synthetic"},"capabilities":["completion"]});
 let t=cache.begin(key.clone(),100_000).unwrap();assert!(cache.finish(t,cache::model_evidence(&valid),100_001));assert!(cache.fresh_installed_local_model(&key,100_002));assert!(!cache.fresh_installed_local_model(&key,100_000));assert!(!cache.fresh_installed_local_model(&key,115_001));
 let mut other=key.clone();other.model=Some("other-model".into());assert!(!cache.fresh_installed_local_model(&other,100_002));other=key.clone();other.endpoint="http://127.0.0.1:11435".into();assert!(!cache.fresh_installed_local_model(&other,100_002));
 for data in [serde_json::json!({"capabilities":["completion"]}),serde_json::json!({"model_info":{},"capabilities":["completion"]}),serde_json::json!({"model_info":{"x":1},"capabilities":["completion"],"remote_host":"host.invalid"})]{let t=cache.begin(key.clone(),100_010).unwrap();assert!(!cache.fresh_installed_local_model(&key,100_010));cache.finish(t,cache::model_evidence(&data),100_011);assert!(!cache.fresh_installed_local_model(&key,100_012));}
 let t=cache.begin(key.clone(),100_020).unwrap();cache.finish(t,cache::model_evidence(&valid),100_021);assert!(cache.fresh_installed_local_model(&key,100_022));let _pending=cache.begin(key.clone(),100_023).unwrap();assert!(!cache.fresh_installed_local_model(&key,100_024));
}
