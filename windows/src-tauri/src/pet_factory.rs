//! Bundled inheritance only. No runtime identity, permission or execution grant.
use super::Configuration;
use serde_json::{Map,Value};

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
fn configuration(source:&Value)->Result<Configuration,String>{
    if source.get("schemaVersion")!=Some(&serde_json::json!(1)){return Err(invalid());}
    let molds=source.get("molds").and_then(Value::as_object).ok_or_else(invalid)?;
    if molds.len()!=2||!molds.contains_key("llm")||!molds.contains_key("application"){return Err(invalid());}
    let extensions=source.get("extensions").and_then(Value::as_object).ok_or_else(invalid)?;
    // Templates cannot provide identity, grants or an execution policy.
    for recipe in std::iter::once(source.get("base").ok_or_else(invalid)?).chain(molds.values()).chain(extensions.values()){
        if !recipe.is_object(){return Err(invalid());}
        if let Some(prefs)=recipe.get("preferences"){
            let prefs=prefs.as_object().ok_or_else(invalid)?;
            if ["name","aliases","provider","permissions","localExecution"].iter().any(|k|prefs.contains_key(*k)){return Err(invalid());}
        }
    }
    let pets=source.get("pets").and_then(Value::as_object).ok_or_else(invalid)?;
    if pets.is_empty()||pets.len()>64{return Err(invalid());}
    let mut resolved=Map::new();
    for(id,entry)in pets{
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
    serde_json::from_value(config).map_err(|_|invalid())
}
pub(super) fn bundled()->Configuration{
    static DEFAULTS:std::sync::OnceLock<Configuration>=std::sync::OnceLock::new();
    DEFAULTS.get_or_init(||{
        let source:Value=serde_json::from_str(include_str!("../pets.factory.json")).expect("bundled pet factory JSON");
        configuration(&source).expect("bundled pet factory configuration")
    }).clone()
}
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
