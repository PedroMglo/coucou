//! Bundled preferences and typed route metadata. No permission or execution grant.
use super::Configuration;
use serde_json::{Map,Value};
#[path="pet_factory_runtime.rs"]
mod runtime;
pub use runtime::{Adapter,Blueprint,RuntimeKind};

fn invalid()->String{"Invalid bundled pet factory".into()}
fn merge(base:&mut Value,overlay:&Value)->Result<(),String>{
    let target=base.as_object_mut().ok_or_else(invalid)?;
    for(key,next)in overlay.as_object().ok_or_else(invalid)? {
        if matches!(key.as_str(),"__proto__"|"prototype"|"constructor"){return Err(invalid());}
        if next.is_object()&&target.get(key).is_some_and(Value::is_object){
            merge(target.get_mut(key).unwrap(),next)?;
        }else{target.insert(key.clone(),next.clone());}
    }
    Ok(())
}

#[cfg(test)]mod runtime_tests {
 use super::*;
 fn source()->Value{serde_json::from_str(include_str!("../pets.factory.json")).unwrap()}
 #[test]fn catalog_covers_native_identities_without_conflating_logical_owners(){
  assert_eq!(blueprints().count(),25);
  for p in blueprints(){
   for provider in [&p.routing.chat,&p.routing.voice,&p.routing.task_provider].into_iter().flatten(){assert_eq!(for_provider(provider),Some(p.id.as_str()));}
   if let Some(provider)=&p.routing.task_provider{assert_eq!(for_task_provider(provider),Some(p.id.as_str()));}
  }
  let notion=blueprint("integration_notion").unwrap();assert!(notion.service);assert_eq!(notion.configured_adapter("codex"),Some(Adapter::Codex));assert_eq!(for_provider("integration_notion"),Some("integration_notion"));assert_eq!(for_provider("codex"),Some("agent_codex"));
  assert_eq!(blueprint("integration_music").unwrap().runtime.kind,RuntimeKind::Service);assert!(!blueprint("integration_music").unwrap().service);assert!(!blueprint("integration_music").unwrap().history_owner());
  for provider in ["","unknown","constructor"]{assert_eq!(for_provider(provider),None);assert_eq!(for_task_provider(provider),None);}
 }
 #[test]fn cross_channel_identity_ambiguity_is_rejected(){
  let mut s=source();s["pets"]["ai_openai"]["routing"]["chat"]=serde_json::json!("gemini-cli");assert!(resolve(&s).is_err());
 }
 #[test]fn runtime_metadata_is_explicit_and_cannot_be_inherited(){
  for key in ["runtime","routing","service"]{for place in ["base","llm","application","codex-account"]{
   let mut s=source();let target=match place{"base"=>&mut s["base"],"llm"|"application"=>&mut s["molds"][place],_=>&mut s["extensions"][place]};target[key]=serde_json::json!({});assert!(resolve(&s).is_err());
  }}
  for key in ["kind","adapter"]{let mut s=source();s["pets"]["agent_codex"]["runtime"].as_object_mut().unwrap().remove(key);assert!(resolve(&s).is_err());}
 }
 #[test]fn adapter_claims_cannot_promote_unimplemented_or_incompatible_routes(){
  for (id,field,value)in [("ai_openai","adapter",serde_json::json!("openai")),("agent_copilot","adapter",serde_json::json!("codex")),("integration_spotify","adapter",serde_json::json!("codex")),("agent_cursor","kind",serde_json::json!("llm"))]{let mut s=source();s["pets"][id]["runtime"][field]=value;assert!(resolve(&s).is_err());}
  let mut s=source();s["pets"]["ai_ollama"]["routing"]["execute"]=serde_json::json!(true);assert!(resolve(&s).is_err());
  let mut s=source();s["pets"]["agent_codex"]["runtime"]["permissions"]=serde_json::json!({});assert!(resolve(&s).is_err());
 }
 #[test]fn adding_a_service_owner_uses_the_existing_adapter_contract(){
  let mut s=source();let mut p=s["pets"]["integration_notion"].clone();for k in ["chat","voice","taskProvider"]{p["routing"][k]=serde_json::json!("integration_fixture");}p["preferences"]["name"]=serde_json::json!("Fixture");p["preferences"]["aliases"]=serde_json::json!(["fixture"]);p["preferences"]["permissions"]=serde_json::json!({"documents":false,"emailDrafts":false,"music":false,"openApps":false});s["pets"]["integration_fixture"]=p;
  let resolved=resolve(&s).unwrap();assert_eq!(resolved.entries.len(),26);let p=&resolved.entries["integration_fixture"];assert!(p.service);assert_eq!(p.configured_adapter("codex"),Some(Adapter::Codex));assert_eq!(p.configured_adapter("copilot"),None);assert_eq!(p.routing.task_provider.as_deref(),Some("integration_fixture"));assert!(!resolved.config.pets["integration_fixture"].permissions.as_ref().unwrap().open_apps);
 }
 #[test]fn retirement_and_missing_model_routes_never_grant_new_execution(){
  let github=blueprint("integration_github").unwrap();assert!(github.retired);assert!(!github.routing.execute);assert_eq!(for_provider("integration_github"),Some("integration_github"));
  for id in ["ai_openai","ai_anthropic","ai_google"]{let p=blueprint(id).unwrap();assert_eq!(p.runtime.kind,RuntimeKind::Llm);assert!(p.runtime.adapter.is_none());assert!(!p.routing.execute);assert!(p.history_owner());}
  let lm=blueprint("ai_lmstudio").unwrap();assert_eq!(lm.runtime.adapter,Some(Adapter::Lmstudio));assert_eq!(lm.configured_adapter(""),None);assert_eq!(lm.configured_adapter("lmstudio"),Some(Adapter::Lmstudio));assert!(lm.routing.voice.is_none()&&!lm.routing.execute);
  for id in ["agent_gemini","integration_claude"]{let p=blueprint(id).unwrap();assert_eq!(p.runtime.kind,RuntimeKind::Observer);assert!(p.runtime.adapter.is_none());assert!(p.routing.voice.is_none());assert!(p.history_owner());}
 }
 #[test]fn native_service_and_optional_assistant_are_declared_without_changing_defaults(){
  let p=blueprint("integration_spotify").unwrap();assert_eq!(p.runtime.adapter,Some(Adapter::Spotify));assert_eq!(p.runtime.assistant_adapter,Some(Adapter::Codex));
  assert_eq!(p.configured_adapter(""),Some(Adapter::Spotify));assert_eq!(p.configured_adapter("codex"),Some(Adapter::Codex));assert_eq!(p.configured_adapter("copilot"),None);
  assert_eq!(p.routing.chat.as_deref(),Some("integration_spotify"));assert_eq!(for_provider("codex"),Some("agent_codex"));assert!(bundled().pets["integration_spotify"].provider.is_empty());
 }
 #[test]fn optional_assistant_claims_cannot_promote_an_undeclared_account_or_runtime(){
  for(id,adapter)in[("integration_spotify","copilot"),("ai_ollama","codex"),("integration_notion","codex")]{let mut s=source();s["pets"][id]["runtime"]["assistantAdapter"]=serde_json::json!(adapter);assert!(resolve(&s).is_err());}
  let mut s=source();s["pets"]["integration_spotify"]["runtime"]["assistantAdapter"]=Value::Null;assert!(resolve(&s).is_err());
 }
 #[test]fn another_composite_service_can_use_the_same_factory_contract(){
  let mut s=source();let mut p=s["pets"]["integration_spotify"].clone();for key in ["chat","voice","taskProvider"]{p["routing"][key]=serde_json::json!("integration_audio_fixture");}p["preferences"]["name"]=serde_json::json!("Audio fixture");p["preferences"]["aliases"]=serde_json::json!(["audio fixture"]);s["pets"]["integration_audio_fixture"]=p;
  let catalog=resolve(&s).unwrap();let p=&catalog.entries["integration_audio_fixture"];assert_eq!(p.configured_adapter("codex"),Some(Adapter::Codex));assert_eq!(p.configured_adapter(""),Some(Adapter::Spotify));assert_eq!(p.routing.chat.as_deref(),Some("integration_audio_fixture"));assert!(p.recognizes_owner("integration_audio_fixture","codex"));assert!(p.recognizes_owner("integration_audio_fixture","spotify"));assert!(!p.recognizes_owner("codex","codex"));assert!(!p.recognizes_owner("integration_audio_fixture","copilot"));
 }

}
fn resolve(source:&Value)->Result<Bundled,String>{
    if source.get("schemaVersion")!=Some(&serde_json::json!(2)){return Err(invalid());}
    let molds=source.get("molds").and_then(Value::as_object).ok_or_else(invalid)?;
    if molds.len()!=2||!molds.contains_key("llm")||!molds.contains_key("application"){return Err(invalid());}
    let extensions=source.get("extensions").and_then(Value::as_object).ok_or_else(invalid)?;
    // Templates cannot provide identity, grants or an execution policy.
    for recipe in std::iter::once(source.get("base").ok_or_else(invalid)?).chain(molds.values()).chain(extensions.values()){
        if !recipe.is_object()||["retired","runtime","routing","service"].iter().any(|k|recipe.get(k).is_some()){return Err(invalid());}
        if let Some(prefs)=recipe.get("preferences"){
            let prefs=prefs.as_object().ok_or_else(invalid)?;
            if ["name","aliases","provider","permissions","localExecution"].iter().any(|k|prefs.contains_key(*k)){return Err(invalid());}
        }
    }
    let pets=source.get("pets").and_then(Value::as_object).ok_or_else(invalid)?;
    if pets.is_empty()||pets.len()>64{return Err(invalid());}
    let mut resolved=Map::new();
    for(id,entry)in pets{
        if entry.get("retired").is_some_and(|v|!v.is_boolean()){return Err(invalid());}
        let own=entry.get("preferences").and_then(Value::as_object).ok_or_else(invalid)?;
        if !["name","aliases","provider","permissions"].iter().all(|k|own.contains_key(*k)){return Err(invalid());}
        let mold=entry.get("mold").and_then(Value::as_str).filter(|m|matches!(*m,"llm"|"application")).ok_or_else(invalid)?;
        let mut recipe=source.get("base").cloned().ok_or_else(invalid)?;
        merge(&mut recipe,&molds[mold])?;
        let mut preferences=recipe.get("preferences").cloned().ok_or_else(invalid)?;
        if let Some(extension)=entry.get("extension"){
            let name=extension.as_str().ok_or_else(invalid)?;
            let extra=extensions.get(name).ok_or_else(invalid)?;
            if let Some(value)=extra.get("preferences"){merge(&mut preferences,value)?;}
        }
        merge(&mut preferences,entry.get("preferences").ok_or_else(invalid)?)?;
        resolved.insert(id.clone(),preferences);
    }
    let mut config=source.get("configuration").cloned().ok_or_else(invalid)?;
    config.as_object_mut().ok_or_else(invalid)?.insert("pets".into(),Value::Object(resolved));
    let config:Configuration=serde_json::from_value(config).map_err(|_|invalid())?;
    let entries=runtime::parse(source,&config)?;
    Ok(Bundled{config,entries})
}
#[cfg(test)]fn configuration(source:&Value)->Result<Configuration,String>{Ok(resolve(source)?.config)}
struct Bundled {config:Configuration,entries:std::collections::BTreeMap<String,Blueprint>}
fn catalog()->&'static Bundled {
    static CATALOG:std::sync::OnceLock<Bundled>=std::sync::OnceLock::new();
    CATALOG.get_or_init(||{
        let source:Value=serde_json::from_str(include_str!("../pets.factory.json")).expect("bundled pet factory JSON");
        resolve(&source).expect("bundled pet factory configuration and runtime metadata")
    })
}
pub(super) fn bundled()->Configuration {catalog().config.clone()}
pub(super) fn retired(id:&str)->bool {blueprint(id).is_some_and(|p|p.retired)}
pub(super) fn blueprint(id:&str)->Option<&'static Blueprint>{catalog().entries.get(id)}
pub(super) fn blueprints()->impl Iterator<Item=&'static Blueprint>{catalog().entries.values()}
pub(super) fn for_task_provider(provider:&str)->Option<&'static str>{blueprints().find(|p|p.routing.task_provider.as_deref()==Some(provider)).map(|p|p.id.as_str())}
pub(super) fn for_provider(provider:&str)->Option<&'static str>{blueprints().find(|p|[&p.routing.chat,&p.routing.voice,&p.routing.task_provider].iter().any(|r|r.as_deref()==Some(provider))).map(|p|p.id.as_str())}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn templates_preserve_all_legacy_defaults_and_grants(){
        let legacy:Configuration=serde_json::from_str(include_str!("../pets.default.json")).unwrap();
        assert_eq!(serde_json::to_value(bundled()).unwrap(),serde_json::to_value(legacy).unwrap());
    }
    #[test]fn unknown_mold_or_extension_cannot_fall_back_to_llm(){
        for field in ["mold","extension"]{
            let mut source:Value=serde_json::from_str(include_str!("../pets.factory.json")).unwrap();
            source["pets"]["integration_spotify"][field]=serde_json::json!("unknown");
            assert!(configuration(&source).is_err());
        }
        for key in ["permissions","localExecution","provider"]{
            let mut source:Value=serde_json::from_str(include_str!("../pets.factory.json")).unwrap();
            source["extensions"]["codex-account"]["preferences"][key]=serde_json::json!({});
            assert!(configuration(&source).is_err());
        }
    }
    #[test]fn instance_overrides_stay_separate_from_family_changes(){
        let mut source:Value=serde_json::from_str(include_str!("../pets.factory.json")).unwrap();
        source["molds"]["llm"]=serde_json::json!({"preferences":{"mode":"reasoning"}});
        let result=configuration(&source).unwrap();
        assert_eq!(result.pets["agent_codex"].mode,"reasoning");
        assert_eq!(result.pets["agent_copilot"].mode,"reasoning");
        assert_eq!(result.pets["integration_spotify"].mode,"auto");
        assert_eq!(result.pets["ai_ollama"].local_execution.as_ref().unwrap().mode,super::super::LocalExecutionMode::Ask);
    }
}

#[cfg(test)]mod retirement_tests {
 use super::*;
 #[test]fn retired_metadata_is_typed_and_cannot_be_inherited(){
  for value in [serde_json::json!("true"),Value::Null,serde_json::json!(1),serde_json::json!({})]{
   let mut source:Value=serde_json::from_str(include_str!("../pets.factory.json")).unwrap();
   source["pets"]["integration_github"]["retired"]=value;assert!(configuration(&source).is_err());
  }
  for place in ["base","llm","application","codex-account"]{
   let mut source:Value=serde_json::from_str(include_str!("../pets.factory.json")).unwrap();
   let target=match place{"base"=>&mut source["base"],"llm"|"application"=>&mut source["molds"][place],_=>&mut source["extensions"][place]};
   target["retired"]=serde_json::json!(true);assert!(configuration(&source).is_err());
  }
 }
 #[test]fn retirement_is_separate_from_saved_preferences_and_account_identity(){
  assert!(retired("integration_github"));
  for id in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","integration_notion","unknown"]{assert!(!retired(id));}
  let snapshot:Configuration=serde_json::from_str(include_str!("../pets.default.json")).unwrap();
  assert_eq!(serde_json::to_value(bundled()).unwrap(),serde_json::to_value(snapshot).unwrap());
 }
}
