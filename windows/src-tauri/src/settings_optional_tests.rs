use super::*;
use crate::settings::{self,EffectFixture,IoPhase};
use std::{fs,sync::{Arc,Mutex},cell::Cell};
const ALL:[Operation;6]=[Operation::Voice,Operation::Mobile,Operation::Autostart,Operation::Shortcuts,Operation::Hooks,Operation::Credentials];
fn denied(f:&EffectFixture){for operation in ALL {
    let policy=Cell::new(0);let effects=Cell::new(0);
    let result=checked(operation,||f.probe.check(),|_|{policy.set(policy.get()+1);Ok(())}).map(|_|effects.set(effects.get()+1));
    assert!(result.is_err(),"{operation:?}");assert_eq!(policy.get(),0);assert_eq!(effects.get(),0);
}}
#[test]fn optional_effects_changed_origin_refuses_all_before_component_policy_or_effect(){
    let f=EffectFixture::new();f.external();denied(&f);
    assert_eq!(f.current.lock().unwrap().model,"explicit-fixture-model");
}
#[test]fn optional_effects_exact_restored_component_policy_is_additional_and_never_bypassed(){
    let f=EffectFixture::new();
    for operation in ALL {let mut calls=Vec::new();let checks=Cell::new(0);
        let result=checked(operation,||{checks.set(checks.get()+1);f.probe.check()},|name|{calls.push(name.to_owned());Err("restored feature pending".into())});
        match operation.restored_feature(){Some(name)=>{assert_eq!(calls,vec![name]);assert!(result.is_err());assert_eq!(checks.get(),1);},None=>{assert!(calls.is_empty());assert!(result.is_ok());assert_eq!(checks.get(),2);}}
    }
}
#[test]fn optional_effects_recheck_current_owner_after_component_policy_wait(){
    let f=EffectFixture::new();let effects=Cell::new(0);
    let result=checked(Operation::Mobile,||f.probe.check(),|name|{assert_eq!(name,"mobile");f.external();Ok(())}).map(|_|effects.set(1));
    assert!(result.is_err());assert_eq!(effects.get(),0);assert_eq!(f.probe.status().phase,IoPhase::Changed);
}
#[test]fn optional_effects_mutex_busy_is_backpressure_and_release_does_not_renew_authority(){
    let f=EffectFixture::new();let held=f.current.lock().unwrap();denied(&f);drop(held);
    for operation in ALL {assert!(checked(operation,||f.probe.check(),|_|Ok(())).is_ok());}
    f.external();denied(&f);
}
#[test]fn optional_effects_preserve_desired_flags_but_refuse_recovered_data_after_save_and_restart(){
    let f=EffectFixture::new();fs::write(&f.path,br#"{"autostart":true,"phoneLink":true,"soundVolume":"invalid"}"#).unwrap();
    let owner=Arc::new(Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&owner);
    settings::update_with(&owner,|s|s.sound_volume=0.4,|s|settings::save_to(&f.path,s),|_|{}).unwrap();
    assert_eq!(probe.status().phase,IoPhase::ReviewRequired);
    drop(probe);drop(owner);
    let owner=Arc::new(Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&owner);
    assert!(owner.lock().unwrap().autostart);assert!(owner.lock().unwrap().phone_link);
    for operation in ALL{assert!(checked(operation,||probe.check(),|_|Ok(())).is_err());}
    settings::update_with(&owner,|s|s.sound_volume=0.6,|s|settings::save_to(&f.path,s),|_|{}).unwrap();
    assert_eq!(probe.status().phase,IoPhase::ReviewRequired);
    assert!(owner.lock().unwrap().autostart);assert!(owner.lock().unwrap().phone_link);
}
#[test]fn optional_effects_durable_uncertain_outcome_is_not_retried_by_optional_admission(){
    let f=EffectFixture::new();let callbacks=Cell::new(0);
    assert!(settings::update_with(&f.current,|s|s.autostart=true,
        |s|settings::revision::uncertain_save_for_test(&f.path,s,||f.external()),
        |_|callbacks.set(callbacks.get()+1)).is_err());
    let before=fs::read(&f.path).unwrap();let io=fs::read(f.root.join(settings::durable::FILE)).unwrap();
    assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);denied(&f);
    assert_eq!(fs::read(&f.path).unwrap(),before);assert_eq!(fs::read(f.root.join(settings::durable::FILE)).unwrap(),io);assert_eq!(callbacks.get(),0);
}
#[test]fn optional_effects_legitimate_unrelated_commit_uses_current_owner_not_stale_snapshot(){
    let f=EffectFixture::new();let captured=f.current.lock().unwrap().clone();f.save();
    assert_eq!(settings::io_status(&captured).phase,IoPhase::Changed);
    for operation in ALL{assert!(checked(operation,||f.probe.check(),|_|Ok(())).is_ok());}
}
