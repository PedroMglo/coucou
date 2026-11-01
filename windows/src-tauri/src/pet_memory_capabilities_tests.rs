use super::*;
#[test]fn human_catalogue_never_becomes_a_provider_model_tool_or_an_online_goal_grant(){
 let mut f=Fixture::new();f.config.pets.get_mut("ai_lmstudio").unwrap().provider="lmstudio".into();f.config.pets.get_mut("integration_notion").unwrap().provider="codex".into();let s=f.snapshot(100_000);
 for d in &s.pets{
  let manage=capability(&s,&d.pet_id,"memory.manage");assert_eq!(manage.callers,vec![Caller::NativeUser]);assert_eq!(manage.availability,if d.lifecycle==Lifecycle::Retired{Availability::Unavailable}else{Availability::Ready});
  let retrieve=capability(&s,&d.pet_id,"memory.retrieve");assert_eq!(retrieve.callers,vec![Caller::ProviderModel]);assert_eq!(retrieve.support,Support::Unsupported);
  let goal=capability(&s,&d.pet_id,"memory.goalContext");assert_eq!(goal.callers,vec![Caller::NativeUser,Caller::TaskRuntime]);assert_eq!(goal.support,if ["ai_ollama","ai_lmstudio"].contains(&d.pet_id.as_str()){Support::Supported}else{Support::Unsupported});assert_ne!(goal.availability,Availability::Ready);
 }
 for id in ["ai_ollama","ai_lmstudio","ai_anthropic","ai_openai","ai_google","agent_codex","agent_copilot","agent_antigravity","integration_notion"]{let c=capability(&s,id,"memory.contextReview");assert_eq!(c.support,Support::Supported,"{id}");assert_eq!(c.callers,vec![Caller::NativeUser]);assert_eq!(c.availability,Availability::Unknown);assert_eq!(c.reason,Some(Reason::TargetRequired));}
}
#[test]fn inactive_catalogue_management_remains_possible_but_no_target_or_configuration_is_invented(){
 let mut f=Fixture::new();f.settings.active_integrations.clear();f.settings.main_pill="agent_codex".into();let s=f.snapshot(100_000);assert_eq!(capability(&s,"ai_ollama","memory.manage").availability,Availability::Ready);assert_eq!(capability(&s,"ai_ollama","memory.contextReview").reason,Some(Reason::InactivePet));assert_eq!(capability(&s,"ai_lmstudio","memory.contextReview").support,Support::Unsupported);
 let unhealthy=f.registry.snapshot(&f.settings,Some(&f.config),false,&BTreeSet::new(),100_001).unwrap();assert_eq!(capability(&unhealthy,"agent_codex","memory.manage").reason,Some(Reason::LibraryUnavailable));
 let missing=f.registry.snapshot(&f.settings,None,true,&BTreeSet::new(),100_002).unwrap();assert_eq!(capability(&missing,"agent_codex","memory.manage").availability,Availability::Unknown);assert_eq!(capability(&missing,"agent_codex","memory.manage").reason,Some(Reason::NotConfigured));
}
