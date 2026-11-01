use super::*;
use serde_json::json;
struct Fixture{registry:Registry,settings:Settings,config:Configuration}
impl Fixture{
    fn new()->Self{let config=crate::pet_config::defaults();let mut settings=Settings::default();settings.main_pill="agent_codex".into();settings.active_integrations=config.pets.keys().cloned().collect();settings.ollama_url="http://127.0.0.1:11434".into();settings.chat_models.insert("ollama".into(),"synthetic-model".into());Self{registry:Registry::new().unwrap(),settings,config}}
    fn snapshot(&mut self,now:u64)->Snapshot{self.registry.snapshot(&self.settings,Some(&self.config),true,&BTreeSet::new(),now).unwrap()}
    fn model(&mut self,body:serde_json::Value,now:u64){let k=Key{provider:"ollama".into(),endpoint:cache::endpoint(&self.settings.ollama_url).unwrap(),model:Some("synthetic-model".into())};let t=self.registry.cache.begin(k,now).unwrap();assert!(self.registry.cache.finish(t,cache::model_evidence(&body),now));}
}
fn descriptor<'a>(s:&'a Snapshot,id:&str)->&'a Descriptor{s.pets.iter().find(|p|p.pet_id==id).unwrap()}
fn capability<'a>(s:&'a Snapshot,id:&str,cap:&str)->&'a Capability{descriptor(s,id).capabilities.iter().find(|c|c.id==cap).unwrap()}
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
    let mut f=Fixture::new();let s=f.snapshot(100_000);assert_eq!(s.pets.len(),25);assert_eq!(s.pets.iter().filter(|p|p.lifecycle==Lifecycle::Retired).map(|p|p.pet_id.as_str()).collect::<Vec<_>>(),vec!["integration_github"]);
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
    let mut f=Fixture::new();let b=binding(&f.settings,"agent_codex",&f.config.pets["agent_codex"],true);let t=f.registry.cache.begin(key(&f.settings,&b,None).unwrap(),100_000).unwrap();assert!(f.registry.cache.finish(t,Evidence::Catalog,100_001));let s=f.snapshot(100_002);assert_eq!(capability(&s,"agent_codex","models.list").availability,Availability::Ready);assert_ne!(capability(&s,"agent_codex","conversation.respond").availability,Availability::Ready);assert_ne!(capability(&s,"agent_codex","tools.native").availability,Availability::Ready);
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
        for bound in ["authenticatedClosedSourceRequired","additionalAllowanceOnly","unknownToolEffectsRequireReconciliation","doesNotEnableProviderExecution"]{assert!(c.limitations.contains(&bound));}
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
