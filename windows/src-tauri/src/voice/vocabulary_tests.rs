use super::*;
fn settings()->crate::settings::Settings{let mut s=crate::settings::Settings::default();s.main_pill="agent_codex".into();s.active_integrations=vec!["agent_copilot".into(),"agent_antigravity".into()];s}
#[test]fn canonical_names_and_five_terms_exclude_aliases_learned_forms_and_greeting(){
 let mut config=crate::pet_config::defaults();config.pets.get_mut("agent_copilot").unwrap().name="Meu Copiloto".into();config.pets.get_mut("agent_copilot").unwrap().aliases=vec!["aprendido secreto".into(),"Olá".into()];config.reasoning_terms.push("não é vocabulário".into());
 let before=serde_json::to_value(&config).unwrap();let terms=asr_vocabulary(&config,&settings());assert!(terms.contains(&"Meu Copiloto".into()));assert!(!terms.contains(&"Copilot CLI".into()));for text in["aprendido secreto","Olá","não é vocabulário"]{assert!(!terms.contains(&text.into()));}for term in ASR_TECHNICAL_TERMS{assert!(terms.contains(&term.to_string()));}assert_eq!(serde_json::to_value(config).unwrap(),before);
}
#[test]fn renaming_is_reflected_in_next_snapshot_without_changing_previous_request(){
 let mut config=crate::pet_config::defaults();config.pets.get_mut("agent_codex").unwrap().name="Cody".into();let first=asr_vocabulary(&config,&settings());config.pets.get_mut("agent_codex").unwrap().name="Código".into();let second=asr_vocabulary(&config,&settings());assert!(first.contains(&"Cody".into())&&!first.contains(&"Código".into()));assert!(second.contains(&"Código".into())&&!second.contains(&"Cody".into()));
}
#[test]fn pathological_greeting_controls_and_duplicate_technical_name_do_not_expand_vocabulary(){
 let mut config=crate::pet_config::defaults();config.pets.get_mut("agent_codex").unwrap().name="Olá".into();config.pets.get_mut("agent_copilot").unwrap().name="unit tests".into();config.pets.get_mut("agent_antigravity").unwrap().name="bad\nterm".into();let terms=asr_vocabulary(&config,&settings());assert!(!terms.iter().any(|s|s.contains('\n')||wake::normalize(s)=="ola"));assert_eq!(terms.iter().filter(|s|s.as_str()=="unit tests").count(),1);
}
#[test]fn bundled_canonical_vocabulary_fits_header_contract(){
 let terms=asr_vocabulary(&crate::pet_config::defaults(),&settings());assert_eq!(terms.len(),8);assert!(terms.len()<=crate::speech::whisper::MAX_VOCABULARY_TERMS);assert!(terms.iter().map(String::len).sum::<usize>()<=crate::speech::whisper::MAX_VOCABULARY_BYTES);
}

#[test]fn unconfigured_catalogue_and_retired_voice_routes_are_excluded_before_names(){
 let mut c=crate::pet_config::defaults();c.pets.get_mut("integration_vercel").unwrap().name="Vercel dormante".into();c.pets.get_mut("integration_claude").unwrap().name="Claude dormante".into();c.pets.get_mut("agent_gemini").unwrap().name="Gemini dormante".into();
 let mut s=settings();s.active_integrations.extend(["integration_claude".into(),"agent_gemini".into(),"ai_openai".into()]);let terms=asr_vocabulary(&c,&s);
 for name in["Vercel dormante","Claude dormante","Gemini dormante","OpenAI"]{assert!(!terms.contains(&name.into()));}
 s.active_integrations.push("integration_vercel".into());assert!(asr_vocabulary(&c,&s).contains(&"Vercel dormante".into()));s.active_integrations.retain(|id|id!="integration_vercel");assert!(!asr_vocabulary(&c,&s).contains(&"Vercel dormante".into()));for term in ASR_TECHNICAL_TERMS{assert!(terms.contains(&term.to_string()));}
}
#[test]fn greeting_and_technical_terms_remain_plain_messages_not_control_or_pet_selection(){
 for text in["Olá, podes ajudar-me?","unit tests e pull requests","Docker Compose issues next steps"]{
  assert!(matches!(dialogue::parse_with_aliases(text,&std::collections::BTreeMap::new()),dialogue::Command::Message(ref message)if message==text));
 }
}
