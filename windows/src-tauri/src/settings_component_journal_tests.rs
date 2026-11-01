use super::*;
use crate::settings::{self, durable::{self, Observation}, EffectFixture, IoPhase};
use serde_json::{json, Value};
use std::{fs, sync::atomic::{AtomicUsize, Ordering}};

fn intent(f: &EffectFixture, component: &str, id: &str) -> Slot {
    let previous = Observation::read(&f.path).unwrap().component(component).cloned();
    let binding = Binding { profile: "a".repeat(64), configuration: autostart_configuration(f.current.lock().unwrap().autostart), resource: "b".repeat(64), plan: "c".repeat(64) };
    let artifact = Artifact { version: 1, generation: 1, id: id.into(), effect: id.into(), component: component.into(), binding: binding.clone(), phase: Phase::Prepared,
        evidence: json!({"fixture":true,"nativePlan":{"schemaVersion":2},"enabled":true,"scope":"privateOnly"}), previous };
    let reference = retain(&f.path, &artifact, || Ok(())).unwrap();
    Slot { id: id.into(), component: component.into(), phase: Phase::Prepared, binding, owner: "d".repeat(64), intent: reference, outcome: None }
}
fn outcome(f: &EffectFixture, slot: &Slot, phase: Phase, id: &str) -> ReconciliationRef {
    retain(&f.path, &Artifact { version: 1, generation: 1, id: id.into(), effect: slot.id.clone(), component: slot.component.clone(), binding: slot.binding.clone(), phase,
        evidence: json!({"observed":"syntheticFilesystem","loginVerified":false}), previous: None }, || Ok(())).unwrap()
}
fn prepare_owned(f: &EffectFixture, slot: Slot) {
    prepare(&f.current.lock().unwrap(), &f.path, slot, || Ok(())).unwrap();
}
fn transition_owned(f: &EffectFixture, slot: &Slot, phase: Phase, reference: Option<ReconciliationRef>) {
    transition(&f.current.lock().unwrap(), &f.path, &slot.component, &slot.id, phase, reference, || Ok(())).unwrap();
}
const FIRST: &str = "11111111111111111111111111111111";
const SECOND: &str = "22222222222222222222222222222222";
const THIRD: &str = "33333333333333333333333333333333";

#[test]
fn shortcut_verified_fact_keeps_persistent_new_registration_barrier_for_restart_and_older_readers(){
 let f=EffectFixture::new();f.save();let slot=intent(&f,"shortcuts",FIRST);prepare_owned(&f,slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);
 let reference=outcome(&f,&slot,Phase::Verified,SECOND);transition_owned(&f,&slot,Phase::Verified,Some(reference));
 let raw:Value=serde_json::from_slice(&fs::read(f.root.join(durable::FILE)).unwrap()).unwrap();assert_eq!(raw["version"],7);assert_eq!(raw["components"]["shortcuts"]["phase"],"verified");assert!(raw["suspendedOperations"].as_array().unwrap().iter().any(|v|v=="shortcuts"));
 let restart=settings::load_from(&f.path);assert!(settings::io_status(&restart).suspended_operations.iter().any(|n|n=="shortcuts"));
}

#[test]
fn shortcut_data_pause_preserves_dispatched_effect_and_other_component_slots(){
 let f=EffectFixture::new();f.save();let slot=intent(&f,"shortcuts",FIRST);prepare_owned(&f,slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);
 let auto=intent(&f,"autostart",SECOND);prepare_owned(&f,auto);
 let before=Observation::read(&f.path).unwrap().component("shortcuts").cloned().unwrap();let other=Observation::read(&f.path).unwrap().component("autostart").cloned().unwrap();let native_path=super::path(&f.path,&slot.intent.id).unwrap();let native_intent=fs::read(&native_path).unwrap();
 suspend_shortcut_registration(&f.current.lock().unwrap(),&f.path,||Ok(())).unwrap();
 let after=Observation::read(&f.path).unwrap();assert_eq!(after.component("shortcuts"),Some(&before));assert_eq!(after.component("autostart"),Some(&other));assert_eq!(fs::read(&native_path).unwrap(),native_intent);
}

#[test]
fn component_journal_intent_dispatch_and_outcome_use_same_settings_owner_and_survive_restart() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST);
    let preferences = fs::read(&f.path).unwrap(); prepare_owned(&f, slot.clone());
    assert_eq!(fs::read(&f.path).unwrap(), preferences);
    assert_eq!(f.probe.status().phase, IoPhase::Ready);
    assert!(f.probe.operation("autostart").is_err());
    let r = Observation::read(&f.path).unwrap(); assert_eq!(r.component("autostart").unwrap().phase, Phase::Prepared);
    assert_eq!(r.snapshot.registry.version, 6);
    // Preference publication is separate; its existing writer preserves intent.
    settings::update_with(&f.current, |v| v.sound_volume = 0.65, |v| settings::revision::save_to(&f.path, v), |_| {}).unwrap();
    transition_owned(&f, &slot, Phase::Dispatched, None);
    let reference = outcome(&f, &slot, Phase::Verified, SECOND);
    transition_owned(&f, &slot, Phase::Verified, Some(reference));
    let restart = settings::load_from(&f.path); assert_eq!(settings::io_status(&restart).phase, IoPhase::Ready);
    assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().phase, Phase::Verified);
    // This fixture has no admitted native profile: it must not grant activation.
    assert!(settings::io_status(&restart).suspended_operations.iter().any(|n| n == "autostart"));
}

#[test]
fn component_journal_missing_ack_stays_uncertain_and_never_allows_automatic_replay() {
    for dispatched in [false, true] {
        let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
        if dispatched { transition_owned(&f, &slot, Phase::Dispatched, None); }
        let restart = settings::load_from(&f.path); assert_eq!(settings::io_status(&restart).phase, IoPhase::Ready);
        assert!(settings::io_status(&restart).suspended_operations.iter().any(|n| n == "autostart"));
        let replacement = intent(&f, "autostart", SECOND);
        assert!(prepare(&restart, &f.path, replacement, || Ok(())).unwrap_err().to_string().contains("Reconciliation"));
        assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().id, FIRST);
    }
}

#[test]
fn component_journal_recovered_observation_retains_barrier_and_allows_only_a_new_exact_intent() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    transition_owned(&f, &slot, Phase::Dispatched, None);
    let reference = outcome(&f, &slot, Phase::ReconciledObservation, SECOND);
    transition_owned(&f, &slot, Phase::ReconciledObservation, Some(reference));
    assert!(f.probe.operation("autostart").is_err());
    let next = intent(&f, "autostart", THIRD); prepare_owned(&f, next.clone());
    let retained = observe(&f.path, &next.intent).unwrap(); assert_eq!(retained.registry.previous.unwrap().phase, Phase::ReconciledObservation);
    assert!(f.probe.operation("autostart").is_err());
}

#[test]
fn component_journal_replay_wrong_effect_wrong_component_and_tampered_outcome_are_refused() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    assert!(transition(&f.current.lock().unwrap(), &f.path, "mobile", FIRST, Phase::Dispatched, None, || Ok(())).is_err());
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", SECOND, Phase::Dispatched, None, || Ok(())).is_err());
    let reference = outcome(&f, &slot, Phase::Verified, SECOND);
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", FIRST, Phase::Verified, Some(reference.clone()), || Ok(())).is_err());
    transition_owned(&f, &slot, Phase::Dispatched, None);
    let mut forged = reference.clone(); forged.sha256 = "f".repeat(64);
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", FIRST, Phase::Verified, Some(forged), || Ok(())).is_err());
    transition_owned(&f, &slot, Phase::Verified, Some(reference.clone()));
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", FIRST, Phase::Verified, Some(reference), || Ok(())).is_err());
}

#[test]
fn component_journal_late_revocation_before_record_publication_does_not_dispatch() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    let calls = AtomicUsize::new(0);
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", FIRST, Phase::Dispatched, None, || {
        calls.fetch_add(1, Ordering::Relaxed); Err("nativeReviewRevoked")
    }).is_err());
    assert!(calls.load(Ordering::Relaxed) > 0);
    assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().phase, Phase::Prepared);
    assert_eq!(f.probe.status().phase, IoPhase::Ready);
}

#[test]
fn component_journal_retained_artifact_substitution_blocks_current_owner_and_restart() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    fs::write(path(&f.path, &slot.intent.id).unwrap(), b"changed evidence").unwrap();
    assert_eq!(f.probe.status().phase, IoPhase::PublicationUnconfirmed);
    let restart = settings::load_from(&f.path); assert_eq!(settings::io_status(&restart).phase, IoPhase::PublicationUnconfirmed);
    assert!(transition(&restart, &f.path, "autostart", FIRST, Phase::Dispatched, None, || Ok(())).is_err());
}

#[test]
fn component_journal_restore_keeps_exact_outcome_and_suspends_previously_verified_component() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    transition_owned(&f, &slot, Phase::Dispatched, None);
    transition_owned(&f, &slot, Phase::Verified, Some(outcome(&f, &slot, Phase::Verified, SECOND)));
    let original = fs::read(f.root.join(durable::FILE)).unwrap(); let old = fs::read(&f.path).unwrap();
    let mut changed: Value = serde_json::from_slice(&old).unwrap(); changed["soundVolume"] = json!(0.321);
    let restored = durable::restored_record(&original, Some(&old), &serde_json::to_vec(&changed).unwrap()).unwrap();
    let before: Value = serde_json::from_slice(&original).unwrap(); let after: Value = serde_json::from_slice(&restored).unwrap();
    assert_eq!(before["components"], after["components"]); assert_eq!(after["version"], 6);
    assert_eq!(after["suspendedOperations"], json!(durable::OPERATIONS));
    fs::write(&f.path,serde_json::to_vec(&changed).unwrap()).unwrap();fs::write(f.root.join(durable::FILE),&restored).unwrap();
    let owner=std::sync::Arc::new(std::sync::Mutex::new(settings::load_from(&f.path)));let probe=settings::effect_probe(&owner);
    assert!(probe.status().new_effects_allowed);for name in durable::OPERATIONS{assert!(probe.operation(name).is_err(),"copied profile revalidated {name}");}
}

#[test]
fn component_journal_v1_v2_encoding_remains_unchanged_and_v3_rejects_forged_metadata() {
    let f = EffectFixture::new(); f.save();
    let old: Value = serde_json::from_slice(&fs::read(f.root.join(durable::FILE)).unwrap()).unwrap();
    assert_eq!(old["version"], 1); assert!(old.get("components").is_none());
    let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot);
    let original: Value = serde_json::from_slice(&fs::read(f.root.join(durable::FILE)).unwrap()).unwrap();
    for (field, value) in [("id", json!("spoof")), ("component", json!("root")), ("owner", json!("renderer")), ("phase", json!("verified"))] {
        let mut changed = original.clone(); changed["components"]["autostart"][field] = value;
        let parsed: durable::Record = serde_json::from_value(changed).unwrap(); assert!(parsed.validate().is_err());
    }
    for version in [1, 2] { let mut changed = original.clone(); changed["version"] = json!(version); let parsed: durable::Record = serde_json::from_value(changed).unwrap(); assert!(parsed.validate().is_err()); }
}

#[test]
fn component_journal_verifying_autostart_does_not_release_another_component() {
    let f = EffectFixture::new(); f.save(); let first = intent(&f, "autostart", FIRST); prepare_owned(&f, first.clone());
    let mobile = intent(&f, "mobile", SECOND); prepare_owned(&f, mobile);
    transition_owned(&f, &first, Phase::Dispatched, None);
    transition_owned(&f, &first, Phase::Verified, Some(outcome(&f, &first, Phase::Verified, THIRD)));
    assert_eq!(Observation::read(&f.path).unwrap().suspended(), vec!["mobile"]);
    assert!(f.probe.operation("mobile").is_err());
}

#[test]
fn component_journal_human_data_review_preserves_an_interrupted_effect_instead_of_resetting_to_v2() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    transition_owned(&f, &slot, Phase::Dispatched, None);
    let (source, original) = crate::private_registry::preferences_file::read(&f.path, false).unwrap();
    let (journal, old_io) = crate::private_registry::preferences_file::read(&f.root.join(durable::FILE), false).unwrap();
    let bytes = original.as_deref().unwrap();
    let reference = durable::history::retain(&f.path, durable::history::Input {
        id: THIRD, fingerprint: &"a".repeat(64), original_settings: original.as_deref(), original_io: old_io.as_deref(),
        approved: bytes, memory_hash: &"b".repeat(64), configuration_revision: "fixture", configuration_values: &json!({}), observed_phase: "Ready",
    }, || Ok(())).unwrap();
    let prepared = durable::review_prepare(&f.path, &journal, old_io.as_deref(), source.content_hash(), bytes, reference, || Ok(())).unwrap();
    source.commit_guarded(bytes, || if prepared.current() { Ok(()) } else { Err(io::Error::other("changed")) }).unwrap();
    let reviewed = prepared.finish_reviewed().unwrap();
    assert_eq!(reviewed.snapshot.registry.version, 6);
    assert_eq!(reviewed.component("autostart").unwrap().phase, Phase::Dispatched);
    assert_eq!(reviewed.component("autostart").unwrap().intent, slot.intent);
    assert_eq!(reviewed.suspended().len(), 6);
}

#[test]
fn component_journal_changed_desired_preference_cannot_dispatch_the_old_review() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot);
    settings::update_with(&f.current, |v| v.autostart = true, |v| settings::revision::save_to(&f.path, v), |_| {}).unwrap();
    assert!(transition(&f.current.lock().unwrap(), &f.path, "autostart", FIRST, Phase::Dispatched, None, || Ok(())).is_err());
    assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().phase, Phase::Prepared);
}

fn native_file_effect(f: &EffectFixture) -> (Slot, crate::pet_autostart::native::Plan, crate::pet_autostart::native::Observation) {
    use std::os::unix::fs::PermissionsExt;
    let config = f.root.join("native-config"); fs::create_dir(&config).unwrap();
    let launcher = f.root.join("private-launcher"); fs::write(&launcher, b"fixture executable; never launched").unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
    let plan = crate::pet_autostart::native::Plan::capture(&config, &launcher, true).unwrap();
    let binding = Binding { profile: "a".repeat(64), configuration: autostart_configuration(true), resource: plan.view.resource.clone(), plan: plan.view.fingerprint.clone() };
    let reference = retain(&f.path, &Artifact { version: 1, generation: 1, id: FIRST.into(), effect: FIRST.into(), component: "autostart".into(), binding: binding.clone(), phase: Phase::Prepared,
        evidence: json!({"nativePlan":plan.view,"originalBytes":plan.original(),"expectedBytes":plan.expected(),"fixtureAuthorityOnly":true}), previous: None }, || Ok(())).unwrap();
    let slot = Slot { id: FIRST.into(), component: "autostart".into(), phase: Phase::Prepared, binding, owner: "d".repeat(64), intent: reference, outcome: None };
    prepare_owned(f, slot.clone()); assert!(!config.join("autostart").exists());
    settings::update_with(&f.current, |v| v.autostart = true, |v| settings::revision::save_to(&f.path, v), |_| {}).unwrap();
    transition_owned(f, &slot, Phase::Dispatched, None); assert!(!config.join("autostart").exists());
    let observed = plan.execute(|| {
        f.probe.check().map_err(io::Error::other)?;
        if !f.current.lock().unwrap().autostart { return Err(io::Error::other("desired value changed")); }
        let io = Observation::read(&f.path)?;
        if io.component("autostart").is_some_and(|s| s.id == FIRST && s.phase == Phase::Dispatched && s.binding == slot.binding) { Ok(()) }
        else { Err(io::Error::other("effect intent changed")) }
    }).unwrap();
    assert_eq!(fs::read(config.join("autostart/Coucou.desktop")).unwrap(), plan.expected().unwrap());
    (slot, plan, observed)
}

#[test]
fn component_journal_exact_autostart_file_effect_follows_saved_intent_and_observed_outcome() {
    let f = EffectFixture::new(); f.save(); let (slot, _, observed) = native_file_effect(&f);
    assert!(!observed.login_verified);
    let reference = retain(&f.path, &Artifact { version: 1, generation: 1, id: SECOND.into(), effect: FIRST.into(), component: "autostart".into(), binding: slot.binding.clone(), phase: Phase::Verified,
        evidence: serde_json::to_value(&observed).unwrap(), previous: None }, || Ok(())).unwrap();
    transition_owned(&f, &slot, Phase::Verified, Some(reference));
    let restart = Observation::read(&f.path).unwrap(); assert_eq!(restart.component("autostart").unwrap().phase, Phase::Verified);
    // The admitted-profile authority and actual login remain unproved here.
    assert!(!restart.component_revalidated("autostart", &autostart_configuration(true)));
}

#[test]
fn component_journal_missing_ack_after_file_rename_keeps_unknown_and_refuses_repeat() {
    let f = EffectFixture::new(); f.save(); let (_, plan, _) = native_file_effect(&f);
    let restart = settings::load_from(&f.path);
    assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().phase, Phase::Dispatched);
    assert!(!plan.current());
    let replacement = intent(&f, "autostart", SECOND);
    assert!(prepare(&restart, &f.path, replacement, || Ok(())).is_err());
    assert_eq!(fs::read(&plan.view.target).unwrap(), plan.expected().unwrap());
}

#[test]
fn component_journal_interrupted_marker_upgrade_can_be_reviewed_without_erasing_unknown_effects() {
    let f = EffectFixture::new(); f.save(); let bytes = fs::read(f.root.join(durable::FILE)).unwrap();
    let prior: durable::Record = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(durable::review_version(Some(4), Some(&bytes), Some(&prior)).unwrap(), 4);
    assert_eq!(durable::review_version(Some(4), None, None).unwrap(), 4);
    assert_eq!(durable::review_version(Some(3), Some(&bytes), Some(&prior)).unwrap(), 3);
    assert!(durable::review_version(Some(4), Some(b"malformed or future component journal"), None).is_err());
    assert_eq!(durable::review_version(None, Some(&bytes), Some(&prior)).unwrap(), 2);
}

#[test]
fn component_journal_execution_witness_tracks_real_owner_and_failed_io_without_disk_changes() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());
    transition_owned(&f, &slot, Phase::Dispatched, None);
    let current=f.current.lock().unwrap();let witness=execution(&current,&f.path,FIRST,&slot.binding,None).unwrap();let detached=current.clone();
    let settings_bytes=fs::read(&f.path).unwrap();let io_bytes=fs::read(f.root.join(durable::FILE)).unwrap();
    assert!(witness.current());crate::settings::revision::mark_unconfirmed(&detached);assert!(witness.current(),"detached data clone is not the actual owner");
    crate::settings::revision::mark_unconfirmed(&current);assert!(!witness.current());
    assert_eq!(fs::read(&f.path).unwrap(),settings_bytes);assert_eq!(fs::read(f.root.join(durable::FILE)).unwrap(),io_bytes);
}
#[test]
fn component_journal_execution_witness_cannot_survive_replacing_the_actual_owner_with_an_equal_clone() {
    let f = EffectFixture::new(); f.save(); let slot = intent(&f, "autostart", FIRST); prepare_owned(&f, slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);
    let mut current=f.current.lock().unwrap();let witness=execution(&current,&f.path,FIRST,&slot.binding,None).unwrap();assert!(witness.current());let next=current.clone();*current=next;assert!(!witness.current());
}

#[test]
fn component_journal_shortcuts_execution_uses_own_dispatched_slot_and_cannot_borrow_autostart() {
    let f=EffectFixture::new();f.save();let slot=intent(&f,"shortcuts",FIRST);prepare_owned(&f,slot.clone());
    let current=f.current.lock().unwrap();assert!(execution_for_component(&current,&f.path,"shortcuts",FIRST,&slot.binding,None).is_err());drop(current);
    transition_owned(&f,&slot,Phase::Dispatched,None);
    let current=f.current.lock().unwrap();let witness=execution_for_component(&current,&f.path,"shortcuts",FIRST,&slot.binding,None).unwrap();assert!(witness.current());
    assert!(execution(&current,&f.path,FIRST,&slot.binding,None).is_err());assert!(execution_for_component(&current,&f.path,"hooks",FIRST,&slot.binding,None).is_err());assert!(execution_for_component(&current,&f.path,"shortcuts",SECOND,&slot.binding,None).is_err());
    let mut wrong=slot.binding.clone();wrong.resource="f".repeat(64);assert!(execution_for_component(&current,&f.path,"shortcuts",FIRST,&wrong,None).is_err());
    assert!(execution_for_component(&current,&f.path,"shortcuts",FIRST,&slot.binding,Some(&ReconciliationRef{id:SECOND.into(),sha256:"f".repeat(64)})).is_err());drop(current);
    assert!(f.probe.operation("shortcuts").is_err(),"a provenance witness grants no native activation");
    f.save();assert!(!witness.current(),"source mutation revokes before another native effect");
}

#[test]
fn component_journal_independent_autostart_requires_explicit_retained_review_and_keeps_unknown_previous() {
    for acknowledged in [false,true] { for reviewed_action in [false,true] {
        let f=EffectFixture::new();f.save();let old=intent(&f,"autostart",FIRST);prepare_owned(&f,old.clone());transition_owned(&f,&old,Phase::Dispatched,None);
        let old=Observation::read(&f.path).unwrap().component("autostart").unwrap().clone();
        let mut binding=old.binding.clone();binding.profile="e".repeat(64);binding.resource="f".repeat(64);
        let artifact=Artifact{version:1,generation:1,id:SECOND.into(),effect:SECOND.into(),component:"autostart".into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"externalUncertaintyAcknowledged":acknowledged,"review":{"action":if reviewed_action{"setIndependentRegistration"}else{"setRegistration"}}}),previous:Some(old.clone())};
        let reference=retain(&f.path,&artifact,||Ok(())).unwrap();let slot=Slot{id:SECOND.into(),component:"autostart".into(),phase:Phase::Prepared,binding,owner:"d".repeat(64),intent:reference,outcome:None};
        let settings=f.current.lock().unwrap();assert!(prepare(&settings,&f.path,slot.clone(),||Ok(())).is_err());
        let result=prepare_independent_autostart(&settings,&f.path,slot,||Ok(()));
        if acknowledged&&reviewed_action{result.unwrap();assert_eq!(intent_artifact(&settings,&f.path,"autostart").unwrap().unwrap().previous,Some(old));assert_eq!(Observation::read(&f.path).unwrap().snapshot.registry.version,6);}
        else{assert!(result.is_err());assert_eq!(Observation::read(&f.path).unwrap().component("autostart"),Some(&old));}
    }}
}

#[test]
fn shortcut_dispatched_to_not_dispatched_needs_exact_no_effect_proof_and_cannot_clear_other_components(){
 for(component,proof)in[("shortcuts",false),("shortcuts",true),("autostart",true)]{
  let f=EffectFixture::new();f.save();let slot=intent(&f,component,FIRST);prepare_owned(&f,slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);
  let reference=retain(&f.path,&Artifact{version:1,generation:1,id:SECOND.into(),effect:FIRST.into(),component:component.into(),binding:slot.binding.clone(),phase:Phase::NotDispatched,evidence:if proof{json!({"nativeEffectsStarted":false})}else{json!({"nativeEffectsStarted":true})},previous:None},||Ok(())).unwrap();
  let result=transition(&f.current.lock().unwrap(),&f.path,component,FIRST,Phase::NotDispatched,Some(reference),||Ok(()));
  assert_eq!(result.is_ok(),component=="shortcuts"&&proof);let phase=Observation::read(&f.path).unwrap().component(component).unwrap().phase;assert_eq!(phase,if component=="shortcuts"&&proof{Phase::NotDispatched}else{Phase::Dispatched});
 }
}

#[test]
fn shortcut_publication_requires_exact_pending_verified_native_fact_and_retains_its_hash_dependency(){
 for mode in ["valid","missingReference","anotherReference","changedNativeFact"]{
  let f=EffectFixture::new();f.save();let slot=intent(&f,"shortcuts",FIRST);prepare_owned(&f,slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);
  let native=json!({"verified":true,"nativeEffectsStarted":true,"reason":"synthetic native result","session":null,"shortcuts":[]});
  let pending=retain(&f.path,&Artifact{version:1,generation:1,id:SECOND.into(),effect:FIRST.into(),component:"shortcuts".into(),binding:slot.binding.clone(),phase:Phase::Unconfirmed,evidence:json!({"publicationProtocol":1,"publication":"pending","nativeObservation":native}),previous:None},||Ok(())).unwrap();
  transition_owned(&f,&slot,Phase::Unconfirmed,Some(pending.clone()));
  assert!(shortcut_publication_execution(&f.current.lock().unwrap(),&f.path,FIRST,&slot.binding,&pending,None).unwrap().current());
  let mut evidence=json!({"publicationProtocol":1,"publication":"published","pendingOutcome":pending,"nativeObservation":native});
  match mode{"missingReference"=>{evidence.as_object_mut().unwrap().remove("pendingOutcome");},"anotherReference"=>{evidence["pendingOutcome"]["sha256"]=json!("f".repeat(64));},"changedNativeFact"=>{evidence["nativeObservation"]["reason"]=json!("unreviewed altered fact");},_=>{}}
  let final_ref=retain(&f.path,&Artifact{version:1,generation:1,id:THIRD.into(),effect:FIRST.into(),component:"shortcuts".into(),binding:slot.binding.clone(),phase:Phase::Verified,evidence,previous:None},||Ok(())).unwrap();
  let result=transition(&f.current.lock().unwrap(),&f.path,"shortcuts",FIRST,Phase::Verified,Some(final_ref),||Ok(()));assert_eq!(result.is_ok(),mode=="valid");
  if mode=="valid"{assert_eq!(Observation::read(&f.path).unwrap().component("shortcuts").unwrap().phase,Phase::Verified);fs::write(path(&f.path,&pending.id).unwrap(),b"changed pending native proof").unwrap();assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);assert_ne!(settings::io_status(&settings::load_from(&f.path)).phase,IoPhase::Ready);}
  else{assert_eq!(Observation::read(&f.path).unwrap().component("shortcuts").unwrap().phase,Phase::Unconfirmed);}
 }
}
#[test]
fn another_component_cannot_complete_an_uncertain_effect_as_a_shortcut_publication(){
 let f=EffectFixture::new();f.save();let slot=intent(&f,"autostart",FIRST);prepare_owned(&f,slot.clone());transition_owned(&f,&slot,Phase::Dispatched,None);transition_owned(&f,&slot,Phase::Unconfirmed,Some(outcome(&f,&slot,Phase::Unconfirmed,SECOND)));
 let reference=outcome(&f,&slot,Phase::Verified,THIRD);assert!(transition(&f.current.lock().unwrap(),&f.path,"autostart",FIRST,Phase::Verified,Some(reference),||Ok(())).is_err());assert_eq!(Observation::read(&f.path).unwrap().component("autostart").unwrap().phase,Phase::Unconfirmed);
}
