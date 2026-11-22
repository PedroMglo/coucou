use super::*;
use serde_json::json;
use std::cell::{Cell, RefCell};

fn config() -> Configuration {
    let mut c = crate::pet_config::defaults();
    for pet in c.pets.values_mut() { pet.permissions=Some(crate::pet_config::Permissions{documents:true,email_drafts:true,music:true,open_apps:true}); }
    c
}
#[test]fn native_call_resolution_during_configuration_load_prevents_every_effect(){
    let(t,_)=crate::pet_native_decisions::Ticket::fixture();let call=crate::pet_native_invocation::Invocation::begin(&t,std::time::Instant::now()+std::time::Duration::from_secs(2),None).unwrap();let count=Cell::new(0);let c=config();
    let r=run_native(call.fence(),"agent_codex","music",&json!({"action":"play"}),||true,||{t.cancel();Ok(c.clone())},|_|{count.set(1);NativeOutcome::new(Delivery::Accepted,"")});assert_eq!(r.refusal,Some(Failure::Cancelled));assert_eq!(count.get(),0);
}
#[test]fn native_invalidation_after_an_accepted_effect_preserves_delivery_without_claiming_verification(){
    let(t,_)=crate::pet_native_decisions::Ticket::fixture();let call=crate::pet_native_invocation::Invocation::begin(&t,std::time::Instant::now()+std::time::Duration::from_secs(2),None).unwrap();let count=Cell::new(0);let c=config();
    let r=run_native(call.fence(),"agent_codex","music",&json!({"action":"play"}),||true,||Ok(c.clone()),|_|{count.set(count.get()+1);t.cancel();NativeOutcome::new(Delivery::Accepted,"native ACK before resolution")});assert_eq!(count.get(),1);assert_eq!(r.delivery,Delivery::Accepted);assert!(!r.scope_current);assert!(!r.effect_verified);let(text,acted)=r.legacy();assert!(!acted);assert!(text.contains("accepted"));
}
#[test]fn native_call_fences_keep_factory_owners_and_existing_grants_independent(){
    let c=config();for pet in["agent_codex","agent_antigravity","agent_copilot","ai_ollama","integration_spotify"]{
        let(t,_)=crate::pet_native_decisions::Ticket::fixture();let call=crate::pet_native_invocation::Invocation::begin(&t,std::time::Instant::now()+std::time::Duration::from_secs(2),None).unwrap();let count=Cell::new(0);
        let r=run_native(call.fence(),pet,"music",&json!({"action":"pause"}),||true,||Ok(c.clone()),|_|{count.set(1);NativeOutcome::new(Delivery::Accepted,"")});assert_eq!(r.pet_id,pet);assert_eq!(count.get(),1);
        drop(call);let old=crate::pet_native_invocation::Invocation::begin(&t,std::time::Instant::now(),None);assert!(old.is_none());
    }
}
fn examples() -> Vec<(&'static str, Value)> { vec![
    ("pills",json!({"action":"list"})),
    ("pills",json!({"action":"add","names":["Ollama"]})),
    ("music",json!({"action":"set_volume","volume":17})),
    ("music",json!({"action":"play"})),
    ("music",json!({"action":"search","query":"C++ / João"})),
    ("music",json!({"action":"play_item","uri":"spotify:track:4uLU6hMCjMI75M1A2tKUQC"})),
    ("service_info",json!({})), ("open_app",json!({"name":"Firefox"})),
    ("prepare_email",json!({"recipient":"user@example.test","subject":"Revisão","body":"Uma linha.\nOutra linha."})),
] }
#[test]
fn common_catalog_matches_the_validators_without_granting_autonomous_execution() {
    let definitions=definitions();assert_eq!(definitions.len(),NAMES.len());
    let mut ids=std::collections::BTreeSet::new();
    for tool in definitions {
        let name=tool["name"].as_str().unwrap();assert!(NAMES.contains(&name));
        assert_eq!(tool["input_schema"]["additionalProperties"],false);
        let descriptor=Operation::named(name).unwrap().descriptor();
        assert!(ids.insert(descriptor.id));assert_eq!(descriptor.version,1);
        assert!(!descriptor.autonomous_execution_available);
        assert_eq!(descriptor.cancellation,"beforeDispatch");
    }
    for(name,input)in examples(){assert!(validate_arguments(name,&input).is_ok(),"{name}");}
    assert!(validate_arguments("web_search",&json!({})).is_err());
}
#[test]
fn missing_volume_out_of_range_and_unknown_arguments_never_reach_any_executor() {
    let c=config();
    for(name,input)in [
        ("music",json!({"action":"set_volume"})), ("music",json!({"action":"set_volume","volume":-1})),
        ("music",json!({"action":"set_volume","volume":101})), ("music",json!({"action":"set_volume","volume":50.5})),
        ("music",json!({"action":"set_volume","volume":"50"})), ("music",json!({"action":"play","volume":50})),
        ("music",json!({"action":"pause","permissions":true})), ("music",json!({"action":"set_volume","volume":50,"query":"unrelated"})),
        ("music",json!({"action":"search","query":" "})), ("music",json!({"action":"play_item","uri":"spotify:track:fake"})),
        ("service_info",json!({"petId":"agent_codex"})), ("open_app",json!({"name":""})),
        ("open_app",json!({"name":"Firefox","command":"shell"})), ("pills",json!({"action":"list","names":["Codex"]})),
        ("prepare_email",json!({"recipient":"a@b\nBcc:c@d","subject":"s","body":"b"})),
        ("prepare_email",json!({"recipient":"a@b,c@d","subject":"s","body":"b"})),
        ("prepare_email",json!({"recipient":"a@b","subject":"s\nBcc:c@d","body":"b"})),
        ("prepare_email",json!({"recipient":"a@b","subject":"s","body":"b","send":true})),
    ] {
        let calls=Cell::new(0);let receipt=run("ai_ollama",name,&input,||true,||Ok(c.clone()),|_|{calls.set(calls.get()+1);NativeOutcome::new(Delivery::Accepted,"unreachable")});
        assert_eq!(receipt.refusal,Some(Failure::InvalidArguments),"{name}: {input}");assert_eq!(calls.get(),0);
    }
}
#[test]
fn complete_arguments_are_bounded_before_freezing_or_native_effects() {
    let c=config();let input=json!({"recipient":"a@b","subject":"s","body":"é".repeat(6000)});
    let calls=Cell::new(0);let r=run("ai_ollama","prepare_email",&input,||true,||Ok(c.clone()),|_|{calls.set(1);NativeOutcome::new(Delivery::Accepted,"")});
    assert_eq!(r.refusal,Some(Failure::InvalidArguments));assert_eq!(calls.get(),0);
}
#[test]
fn factory_owners_share_contracts_without_borrowing_other_pets_permissions() {
    let mut c=config();
    for pet in ["agent_codex","agent_antigravity","agent_copilot","ai_ollama","integration_notion","integration_spotify"] {
        let calls=Cell::new(0);
        let r=run(pet,"music",&json!({"action":"pause"}),||true,||Ok(c.clone()),|_|{calls.set(calls.get()+1);NativeOutcome::new(Delivery::Accepted,"synthetic")});
        assert_eq!(r.pet_id,pet);assert_eq!(r.delivery,Delivery::Accepted);assert_eq!(calls.get(),1);
    }
    c.pets.get_mut("integration_notion").unwrap().permissions.as_mut().unwrap().music=false;
    let r=run("integration_notion","music",&json!({"action":"pause"}),||true,||Ok(c.clone()),|_|panic!("borrowed Codex permissions"));
    assert_eq!(r.refusal,Some(Failure::PermissionDenied));
}
#[test]
fn unknown_retired_and_observer_owners_cannot_mutate_the_host() {
    let c=config();
    for pet in ["unknown","integration_github"] {
        let r=run(pet,"open_app",&json!({"name":"Firefox"}),||true,||Ok(c.clone()),|_|panic!("unavailable owner"));
        assert!(matches!(r.refusal,Some(Failure::UnknownPet|Failure::PetUnavailable)));
    }
    let observer=crate::pet_config::blueprints().find(|b|!b.retired&&b.runtime.kind==crate::pet_config::RuntimeKind::Observer).unwrap();
    let r=run(&observer.id,"open_app",&json!({"name":"Firefox"}),||true,||Ok(c.clone()),|_|panic!("observer effect"));
    assert_eq!(r.refusal,Some(Failure::PetUnavailable));
}
#[test]
fn a_checked_proposal_freezes_typed_arguments_and_never_invents_a_default() {
    let c=config();let mut value=json!({"action":"set_volume","volume":17});
    let checked=prepare(&c,"ai_ollama","music",&value).unwrap();value["volume"]=json!(99);
    assert!(matches!(checked.arguments(),Arguments::Music(Music::SetVolume{volume:17})));
}
#[test]
fn stopped_callers_do_not_even_load_configuration_and_cannot_dispatch_later() {
    let loads=Cell::new(0);
    let r=run("ai_ollama","service_info",&json!({}),||false,||{loads.set(loads.get()+1);Ok(config())},|_|panic!("stopped caller"));
    assert_eq!(r.refusal,Some(Failure::Cancelled));assert_eq!(loads.get(),0);
    let current=Cell::new(0);let c=config();
    let r=run("ai_ollama","service_info",&json!({}),||{current.set(current.get()+1);current.get()<3},||Ok(c.clone()),|_|panic!("late cancellation"));
    assert_eq!(r.refusal,Some(Failure::Cancelled));
}
#[test]
fn revoked_or_changed_owner_configuration_blocks_the_frozen_request() {
    for change_backend in [false,true] {
        let c=config();let reads=Cell::new(0);
        let r=run("integration_notion","music",&json!({"action":"pause"}),||true,||{
            reads.set(reads.get()+1);let mut now=c.clone();if reads.get()>1 {if change_backend{now.pets.get_mut("integration_notion").unwrap().provider.clear();}else{now.pets.get_mut("integration_notion").unwrap().permissions.as_mut().unwrap().music=false;}}Ok(now)
        },|_|panic!("changed original owner"));
        assert_eq!(r.refusal,Some(Failure::ScopeChanged));
    }
}
#[test]
fn configuration_changes_for_another_pet_do_not_cancel_the_original_owner() {
    let c=config();let reads=Cell::new(0);let calls=Cell::new(0);
    let r=run("ai_ollama","music",&json!({"action":"pause"}),||true,||{
        reads.set(reads.get()+1);let mut now=c.clone();if reads.get()>1{now.pets.get_mut("agent_codex").unwrap().permissions.as_mut().unwrap().music=false;}Ok(now)
    },|_|{calls.set(calls.get()+1);NativeOutcome::new(Delivery::Accepted,"ack")});
    assert!(r.scope_current);assert_eq!(calls.get(),1);
}
#[test]
fn unavailable_configuration_never_falls_back_to_default_permissions() {
    let r=run("ai_ollama","open_app",&json!({"name":"Firefox"}),||true,||Err("unavailable".into()),|_|panic!("default grants"));
    assert_eq!(r.refusal,Some(Failure::ConfigurationUnavailable));
}
#[test]
fn policy_or_stop_changes_after_dispatch_preserve_the_observation_without_claiming_completion() {
    let c=RefCell::new(config());let current=Cell::new(true);let calls=Cell::new(0);
    let r=run("ai_ollama","music",&json!({"action":"pause"}),||current.get(),||Ok(c.borrow().clone()),|_|{
        calls.set(calls.get()+1);current.set(false);c.borrow_mut().pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;
        NativeOutcome::new(Delivery::Accepted,"Original ACK; effects require observation")
    });
    assert_eq!(r.delivery,Delivery::Accepted);assert!(!r.scope_current);assert_eq!(r.output,"Original ACK; effects require observation");
    let(text,acted)=r.legacy();assert!(!acted);let data:Value=serde_json::from_str(&text).unwrap();assert_eq!(data["effectVerified"],false);assert_eq!(calls.get(),1);
}
#[test]
fn dispatched_launches_partial_and_uncertain_effects_are_not_reported_as_verified() {
    let c=config();
    for delivery in [Delivery::Dispatched,Delivery::Partial,Delivery::Uncertain,Delivery::NotDispatched] {
        let r=run("ai_ollama","open_app",&json!({"name":"Firefox"}),||true,||Ok(c.clone()),|_|NativeOutcome::new(delivery,"original observation"));
        let(text,acted)=r.legacy();let v:Value=serde_json::from_str(&text).unwrap();assert_eq!(v["effectVerified"],false);assert!(!acted);
    }
}
#[test]
fn listing_is_read_only_while_roster_mutations_keep_their_effect_class() {
    let c=config();
    for (input,effect)in [(json!({"action":"list"}),Effect::ReadOnly),(json!({"action":"remove","names":["Ollama"]}),Effect::Configuration)] {
        let r=run("ai_ollama","pills",&input,||true,||Ok(c.clone()),|_|NativeOutcome::new(Delivery::Returned,"data"));
        assert_eq!(r.effect,Some(effect));
    }
}
#[test]
fn output_limits_cover_the_serialized_envelope_and_preserve_utf8_and_truncation() {
    let c=config();
    for text in ["é😺".repeat(20000),"\0\"\\\n".repeat(20000)] {
        let r=run("ai_ollama","service_info",&json!({}),||true,||Ok(c.clone()),|_|NativeOutcome::new(Delivery::Returned,text.clone()));
        assert!(r.output_truncated);assert_eq!(r.observed_output_bytes,text.len());assert!(text.starts_with(&r.output));
        let(wire,acted)=r.legacy();assert!(wire.len()<=MAX_RESULT_BYTES);assert!(!acted);let v:Value=serde_json::from_str(&wire).unwrap();assert_eq!(v["outputTruncated"],true);
    }
}
#[test]
fn spotify_search_and_draft_data_keep_literal_unicode_and_reserved_characters() {
    assert_eq!(encode("C++ / João?"),"C%2B%2B%20%2F%20Jo%C3%A3o%3F");
    let input=json!({"recipient":"a+b@example.test","subject":"Olá & revisão","body":"Linha 1\n# C++"});
    let Arguments::Email(email)=input::parse(Operation::Email,&input).unwrap()else{panic!("email")};
    let uri=email.uri();assert!(uri.starts_with("mailto:a%2Bb@example.test?subject=Ol%C3%A1%20%26%20revis%C3%A3o&body="));assert!(uri.ends_with("Linha%201%0A%23%20C%2B%2B"));
}
