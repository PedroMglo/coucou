use super::{model::*,admission::{Evidence,Denial},admission_tests::{Fixture,descriptor},context};
use crate::{pet_config::{self,RuntimeKind},pet_capabilities::{Descriptor,Binding,Concurrency,Kind,Lifecycle}};
fn descriptor_for(id:&str,adapter:Option<&'static str>,provider:Option<&str>)->Descriptor{
    let b=pet_config::blueprint(id).unwrap();let kind=match b.runtime.kind{RuntimeKind::Llm=>Kind::Llm,RuntimeKind::Service=>Kind::Service,RuntimeKind::Observer=>Kind::Observer};
    Descriptor{schema_version:1,pet_id:id.into(),name:"Private fixture".into(),kind,lifecycle:Lifecycle::Active,binding:Binding{engine_binding:None,adapter,provider:provider.map(str::to_string),model:None,profile:None},revision:"fixture".into(),observed_at:None,configuration_available:true,capabilities:vec![],concurrency:Concurrency{scope:None,max_active:None}}
}
fn spec(owner:Owner,request:&str)->Spec{Spec{restored_from:None,request_id:request.into(),owner,context:Context::Unbound,intent:"Preserve the original logical pet".into(),criteria:vec!["Original pet and adapter independently checked".into()]}}
#[test]
fn every_declared_native_owner_uses_the_factory_without_borrowing_a_different_pets_route(){
    let f=Fixture::new();let mut count=0;
    for b in pet_config::blueprints(){let Some(adapter)=b.runtime.adapter else{continue};let provider=b.routing.task_provider.as_deref().or(b.routing.chat.as_deref()).unwrap();let d=descriptor_for(&b.id,Some(adapter.key()),Some(provider));
        if b.retired{assert!(Owner::from_descriptor(&d).is_err());continue;}
        let owner=Owner::from_descriptor(&d).unwrap();assert_eq!(owner.pet_id,b.id);assert_eq!(owner.provider,provider);let record=f.store().create(spec(owner,"factory-owner"),100_000).unwrap().record;assert_eq!(f.store().read(&record.id,&b.id).unwrap().spec,record.spec);count+=1;
    }
    assert_eq!(f.store().list(None).unwrap().goals.len(),count);assert!(count>=10);
}
#[test]
fn spotify_optional_assistant_retains_its_own_goal_context_and_serialized_owner(){
    let f=Fixture::new();let spotify=Owner::from_descriptor(&descriptor_for("integration_spotify",Some("codex"),Some("integration_spotify"))).unwrap();let codex=Owner::from_descriptor(&descriptor_for("agent_codex",Some("codex"),Some("codex"))).unwrap();
    let a=f.store().create(spec(spotify,"same-request"),100_000).unwrap().record;let b=f.store().create(spec(codex,"same-request"),100_000).unwrap().record;assert_ne!(a.id,b.id);assert_ne!(a.spec.owner.provider,b.spec.owner.provider);assert_eq!(a.spec.owner.adapter,"codex");assert_eq!(a.schema_version,1);assert_eq!(serde_json::from_slice::<Record>(&serde_json::to_vec(&a).unwrap()).unwrap().spec,a.spec);
    assert!(f.store().read(&a.id,"agent_codex").is_err());assert_eq!(f.store().list(Some("integration_spotify")).unwrap().goals.len(),1);
}
#[test]
fn historical_api_observer_and_retired_owners_stay_readable_without_becoming_available_for_new_native_work(){
    let f=Fixture::new();
    for(id,provider,adapter)in[("ai_openai","openai","openai"),("ai_anthropic","anthropic","anthropic"),("ai_google","google","google"),("agent_gemini","gemini-cli","gemini-cli"),("integration_claude","claude-cli","claude-cli"),("integration_github","integration_github","codex")]{
        let legacy=spec(Owner{pet_id:id.into(),provider:provider.into(),adapter:adapter.into(),model_hint:None},"legacy-shape");legacy.validate().unwrap();let record=f.store().create(legacy,100_000).unwrap().record;assert!(f.store().read(&record.id,id).is_ok());
        let d=descriptor_for(id,Some(adapter),Some(provider));assert!(Owner::from_descriptor(&d).is_err());
    }
}
#[test]
fn lmstudio_history_remains_readable_and_new_draft_ownership_requires_its_explicit_native_binding(){
    let f=Fixture::new();let legacy=spec(Owner{pet_id:"ai_lmstudio".into(),provider:"lmstudio".into(),adapter:"lmstudio".into(),model_hint:None},"legacy-lmstudio");legacy.validate().unwrap();let record=f.store().create(legacy,100_000).unwrap().record;assert!(f.store().read(&record.id,"ai_lmstudio").is_ok());
    let unbound=descriptor_for("ai_lmstudio",None,None);assert!(Owner::from_descriptor(&unbound).is_err());
    let bound=descriptor_for("ai_lmstudio",Some("lmstudio"),Some("lmstudio"));let owner=Owner::from_descriptor(&bound).unwrap();assert_eq!(owner,record.spec.owner);assert!(bound.binding.engine_binding.is_none());assert!(record.admissions.is_empty());
    let crossed=descriptor_for("ai_lmstudio",Some("ollama"),Some("ollama"));assert!(Owner::from_descriptor(&crossed).is_err());
}
#[test]
fn configuration_lifecycle_kind_and_missing_bindings_block_new_owners(){
    for mode in 0..7{let mut d=descriptor_for("agent_antigravity",Some("antigravity"),Some("antigravity"));match mode{0=>d.configuration_available=false,1=>d.lifecycle=Lifecycle::Inactive,2=>d.lifecycle=Lifecycle::Retired,3=>d.kind=Kind::Service,4=>d.binding.adapter=None,5=>d.binding.provider=None,_=>d.binding.adapter=Some("codex")};assert!(Owner::from_descriptor(&d).is_err());}
}
#[test]
fn crossed_or_undeclared_owner_adapter_combinations_never_validate(){
    for(id,provider,adapter)in[("integration_spotify","codex","codex"),("agent_codex","integration_spotify","codex"),("agent_copilot","copilot","codex"),("agent_antigravity","antigravity","copilot"),("integration_spotify","integration_spotify","copilot"),("ai_openai","openai","codex"),("ai_ollama","ollama","codex"),("integration_notion","integration_notion","copilot"),("unknown","unknown","codex")]{assert!(spec(Owner{pet_id:id.into(),provider:provider.into(),adapter:adapter.into(),model_hint:None},"refuse").validate().is_err());}
}
#[test]
fn changing_a_selected_model_never_changes_the_logical_owner_or_supplies_authority(){
    let mut d=descriptor_for("agent_copilot",Some("copilot"),Some("copilot"));let first=Owner::from_descriptor(&d).unwrap();d.binding.model=Some("fixture-free".into());let next=Owner::from_descriptor(&d).unwrap();assert_eq!(first.pet_id,next.pet_id);assert_eq!(first.provider,next.provider);assert_eq!(first.adapter,next.adapter);assert_ne!(first.model_hint,next.model_hint);
    for invalid in ["https://invalid/model","/tmp/model","~private","bad\0model"]{d.binding.model=Some(invalid.into());assert!(Owner::from_descriptor(&d).is_err());}
}
#[test]
fn optional_assistant_admission_requires_its_exact_adapter_and_cannot_adopt_a_changed_native_binding(){
    let f=Fixture::new();let owner=Owner::from_descriptor(&descriptor_for("integration_spotify",Some("codex"),Some("integration_spotify"))).unwrap();let g=f.store().create(spec(owner,"optional-assistant"),100_000).unwrap().record;let main=crate::chat::Chat::persistent(f.path.join("main.json"));let(binding,chat)=context::prepare(&f.path.join("contexts"),&g,&main,PreparationMode::Dedicated,"context").unwrap();drop(chat);let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"context".into(),Action::BindContext{binding},100_001).unwrap().record;
    let mut native=descriptor(&g,100_002);native.kind=Kind::Service;native.binding.adapter=Some("codex");assert!(Evidence::observe(&g,&native,"a".repeat(64),100_002,false,false).is_ok());
    native.binding.adapter=Some("spotify");assert_eq!(Evidence::observe(&g,&native,"a".repeat(64),100_002,false,false).err(),Some(Denial::OwnerChanged));native.binding.adapter=Some("codex");native.binding.provider=Some("codex".into());assert_eq!(Evidence::observe(&g,&native,"a".repeat(64),100_002,false,false).err(),Some(Denial::OwnerChanged));
}
