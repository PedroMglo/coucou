use super::*;
use crate::pet_runtime::rules::{RulePolicy,RuleScope,RuleKind};
use std::{fs,os::unix::fs::PermissionsExt,path::PathBuf};
use serde_json::json;
struct F{path:PathBuf,policy:RulePolicy}impl F{
 fn new()->Self{
  let path=std::env::temp_dir().join(format!("coucou-rule-authority-{}",nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
  let definition=serde_json::from_value(json!({"name":"Bounded daily","intent":"Prepare one response per authorized occurrence","criteria":["Verify actual result"],"schedule":{"kind":"daily","firstDate":"2026-10-24","days":3,"localTime":"09:00","timeZone":"Europe/Lisbon","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"},"missed":"skip","graceSeconds":300,"dataScope":"none","limits":{"maxOccurrences":2,"maxModelCalls":2,"maxToolCalls":0,"maxOutputBytes":8192,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":900}})).unwrap();
  let owner=crate::pet_runtime::model::Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None};let rule=Rule{id:format!("rule-{}","a".repeat(64)),revision:1,owner,state:crate::pet_runtime::rules::State::Draft,created_at:1000,updated_at:1000,definition};
  let schedule={let _s=crate::pet_calendar::TEST_SERIAL.lock().unwrap();crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||true).unwrap()};let first=schedule.slots[0].at.unwrap();let last=schedule.slots.last().unwrap().at.unwrap();
  let policy=RulePolicy{project_progress_data:None,project_data:None,calendar_priority_count:None,notification:None,calendar_data:None,dispatch:None,schema_version:1,scope:RuleScope::BoundedRuleRecurrenceWithoutSelectedData,rule,payment:crate::pet_runtime::admission::Payment::LocalOnly,destination:crate::pet_memory::context_model::Destination::Server{provider:"ollama".into(),endpoint:"http://127.0.0.1:11434/v1/chat/completions".into(),model:"fixture-free".into()},model:"fixture-free".into(),engine_binding:None,policy_revision:"b".repeat(64),descriptor_semantics:"c".repeat(64),schedule,prepared_at:first-60_000,valid_until:last+900_000};assert!(policy.valid());Self{path,policy}
 }
 fn root(&self)->PathBuf{self.path.join("consent")}
 fn authority(&self)->Authority{Authority::ensure(&self.root()).unwrap()}
 fn candidate(&self)->RuleGrant{RuleGrant{schema_version:1,kind:RuleKind::RuleRecurrence,decision_id:nonce().unwrap(),revision:nonce().unwrap(),policy:self.policy.clone(),phase:RulePhase::Active,approved_at:self.policy.prepared_at,updated_at:self.policy.prepared_at,revoked_at:None,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None}}
 fn at(&self,i:usize)->u64{self.policy.schedule.slots[i].at.unwrap()}
 fn bytes(&self,g:&RuleGrant)->Vec<u8>{fs::read(self.root().join(format!("{}.json",g.decision_id))).unwrap()}
}impl Drop for F{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
#[test]fn rule_authority_budget_is_one_committed_balance_across_duplicates_reopen_and_original_confirmation(){
 let f=F::new();let a=f.authority();let g=f.candidate();a.issue_rule(&g,g.approved_at,||true).unwrap();let hash=g.policy.hash().unwrap();let before=f.bytes(&g);let mut paid=g.clone();paid.policy.payment=crate::pet_runtime::admission::Payment::ExistingAccount;assert!(!paid.valid());assert!(a.issue_rule(&paid,g.approved_at,||true).is_err());let mut cloud=g.clone();cloud.policy.model="model:cloud".into();if let crate::pet_memory::context_model::Destination::Server{model,..}=&mut cloud.policy.destination{*model="model:cloud".into();}assert!(!cloud.valid());
 assert!(a.reserve_rule_occurrence(&g.decision_id,&hash,0,f.at(0)-1,||true).is_err());assert_eq!(f.bytes(&g),before);
 let first=a.reserve_rule_occurrence(&g.decision_id,&hash,0,f.at(0),||true).unwrap();let bytes=f.bytes(&g);assert_eq!(a.reserve_rule_occurrence(&g.decision_id,&hash,0,f.at(0)+1,||true).unwrap(),first);assert_eq!(f.bytes(&g),bytes);
 let reopened=Authority::existing(&f.root()).unwrap();reopened.reserve_rule_occurrence(&g.decision_id,&hash,1,f.at(1),||true).unwrap();assert!(matches!(reopened.reserve_rule_occurrence(&g.decision_id,&hash,2,f.at(2),||true),Err(Failure::Capacity)));
 let ack=reopened.issue_rule(&g,f.at(1)+1,||true).unwrap();assert_eq!(ack.reservations.len(),2);let status=reopened.rule_status(&g.policy.rule,true).unwrap();assert_eq!(status.remaining_occurrences,Some(0));assert_eq!(status.committed_output_bytes,8192);assert!(!status.dispatch_available);assert!(reopened.active_unlocked().unwrap().is_empty());assert!(reopened.grant_file_data(&format!("{}.json",g.decision_id)).is_err(),"A recurring grant cannot be loaded as a goal permit");
}
#[test]fn rule_authority_revoke_cannot_be_replayed_as_reactivation_or_a_fresh_balance(){
 let f=F::new();let a=f.authority();let g=f.candidate();a.issue_rule(&g,g.approved_at,||true).unwrap();a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).unwrap();let revoked=a.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,f.at(0)+1).unwrap();assert_eq!(revoked.phase,RulePhase::Revoked);let before=f.bytes(&g);
 assert_eq!(a.issue_rule(&g,f.at(0)+2,||true).unwrap().phase,RulePhase::Revoked);assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),1,f.at(1),||true).is_err());assert!(a.issue_rule(&f.candidate(),f.at(0)+2,||true).is_err());assert_eq!(f.bytes(&g),before);
 let mut forged:serde_json::Value=serde_json::from_slice(&before).unwrap();forged["body"]["phase"]=json!("active");forged["body"]["revokedAt"]=serde_json::Value::Null;let changed=serde_json::to_vec(&forged).unwrap();fs::write(f.root().join(format!("{}.json",g.decision_id)),&changed).unwrap();assert!(a.rule_record_data(&format!("{}.json",g.decision_id)).is_err());assert_eq!(fs::read(f.root().join(format!("{}.json",g.decision_id))).unwrap(),changed);
}
#[test]fn rule_authority_uncertain_publications_reconcile_only_the_original_signed_intent(){
 let f=F::new();let mut a=f.authority();let g=f.candidate();a.fault=Fault::BeforeGrantPublish;assert!(a.issue_rule(&g,g.approved_at,||true).is_err());a.fault=Fault::None;let pending=a.rule_status(&g.policy.rule,true).unwrap();assert_eq!(pending.pending_artifacts,1);assert_eq!(pending.remaining_occurrences,None);assert!(a.issue_rule(&f.candidate(),g.approved_at+1,||true).is_err());a.issue_rule(&g,g.approved_at+1,||true).unwrap();
 a.fault=Fault::BeforeGrantPublish;assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).is_err());a.fault=Fault::None;assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),1,f.at(1),||true).is_err());let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0)+1,||true).unwrap();assert_eq!(r.reserved_at,f.at(0));assert_eq!(r.deadline_at,f.at(0)+900000);
 a.fault=Fault::AfterGrantPublish;assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),1,f.at(1),||true).is_err());a.fault=Fault::None;let before=f.bytes(&g);assert_eq!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),1,f.at(1)+1,||true).unwrap().reserved_at,f.at(1));assert_eq!(before,f.bytes(&g));assert_eq!(a.rule_status(&g.policy.rule,true).unwrap().committed_occurrences,2);
}
#[test]fn rule_authority_revocation_between_intent_and_publication_blocks_late_confirmation(){
 let f=F::new();let a=f.authority();let g=f.candidate();let current=||!fs::read_dir(f.root()).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().starts_with(".grant-"));assert!(a.issue_rule(&g,g.approved_at,current).is_err());assert!(!f.root().join(format!("{}.json",g.decision_id)).exists());let stopped=a.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,g.approved_at+1).unwrap();assert_eq!(stopped.phase,RulePhase::Revoked);assert_eq!(a.issue_rule(&g,g.approved_at+2,||true).unwrap().phase,RulePhase::Revoked);
}
#[test]fn rule_authority_restore_history3_and_epoch4_keep_reservations_as_data_without_fake_goals(){
 let f=F::new();let a=f.authority();let g=f.candidate();a.issue_rule(&g,g.approved_at,||true).unwrap();a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).unwrap();let before=f.bytes(&g);let goals=f.path.join("goals");let review=restoration::Review::capture(&f.root(),&goals,&"d".repeat(64),||Ok(())).unwrap();let value=review.preview().unwrap();assert_eq!(value["historicalGoals"],0);assert_eq!(value["grants"][0]["ruleId"],g.policy.rule.id);assert_eq!(value["grants"][0]["ruleReservations"],1);assert_eq!(value["grants"][0]["signatureValid"],true);let fingerprint=review.fingerprint().unwrap();review.apply(&fingerprint,||Ok(())).unwrap();assert_eq!(f.bytes(&g),before);
 let restored=Authority::existing(&f.root()).unwrap();let status=restored.rule_status(&g.policy.rule,true).unwrap();assert!(status.snapshot_only);assert_eq!(status.remaining_occurrences,None);assert!(restored.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),1,f.at(1),||true).is_err());assert_eq!(restoration::historical_view(&f.root()).unwrap()["historyBodyVersion"],3);
 let review=restoration::EpochReview::capture(&f.root(),&goals,&"e".repeat(64),||Ok(())).unwrap();let fingerprint=review.fingerprint().unwrap();review.apply(&fingerprint,||Ok(())).unwrap();assert_eq!(restoration::historical_view(&f.root()).unwrap()["historyBodyVersion"],4);assert_eq!(f.bytes(&g),before);let again=Authority::existing(&f.root()).unwrap();assert!(again.rule_status(&g.policy.rule,true).unwrap().snapshot_only);
}

fn rule_occurrence_evidence(g:&crate::pet_runtime::model::Record,p:&RulePolicy,now:u64)->crate::pet_runtime::admission::Evidence{
 let d=crate::pet_runtime::admission_tests::descriptor(g,now);crate::pet_runtime::admission::Evidence::observe(g,&d,p.policy_revision.clone(),now,false,false).unwrap()
}
#[test]fn rule_occurrence_native_admission_is_unique_reopens_with_original_budget_and_rejects_other_parent_or_bounds(){
 use crate::pet_runtime::{store::Store,rules::occurrences};
 let f=F::new();let a=f.authority();let grant=f.candidate();a.issue_rule(&grant,grant.approved_at,||true).unwrap();Store::ensure(&f.path.join("goals")).unwrap();let store=Store::existing_verified(&f.path.join("goals"),&f.root()).unwrap().unwrap();let main=crate::chat::Chat::persistent(f.path.join("manual.json"));let at=f.at(0);
 let first=occurrences::admit_at(&store,&a,&f.path.join("contexts"),&main,&grant.decision_id,0,||at,|p|p==&f.policy,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).unwrap();assert!(first.queued&&!first.existing_admission);
 let queued=store.read(&first.goal_id,"ai_ollama").unwrap();assert_eq!(queued.schema_version,67);assert_eq!(queued.admissions.len(),1);let spec=&queued.admissions[0].spec;assert_eq!(spec.schema_version,29);assert!(spec.finite_response()&&spec.execution_requested());assert!(spec.funding.is_none()&&queued.spec.received_from.is_none());a.verify_queued(&queued,at+1).unwrap();
 let parent=a.rule_grant(&grant.decision_id).unwrap();assert_eq!(parent.schema_version,2);assert!(parent.reservations[0].admission.as_ref().unwrap().matches(spec));let bytes=f.bytes(&grant);let before=fs::read(f.path.join("goals").join(format!("{}.json",queued.id))).unwrap();
 let reopened=Authority::existing(&f.root()).unwrap();let again=occurrences::admit_at(&store,&reopened,&f.path.join("contexts"),&main,&grant.decision_id,0,||at+5,|p|p==&f.policy,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).unwrap();assert!(again.existing_admission);assert_eq!(again.goal_id,first.goal_id);assert_eq!(f.bytes(&grant),bytes);assert_eq!(fs::read(f.path.join("goals").join(format!("{}.json",queued.id))).unwrap(),before);
 for dimension in ["parent","slot","budget","model","decision"]{let mut fake=spec.clone();match dimension{"parent"=>fake.rule_occurrence.as_mut().unwrap().rule_decision_id=nonce().unwrap(),"slot"=>fake.rule_occurrence.as_mut().unwrap().slot_index=1,"budget"=>fake.budget.max_model_calls=2,"model"=>fake.resolved_model="other-model".into(),_=>fake.decision_id=nonce().unwrap()};assert!(reopened.rule_admission_current_unlocked(&fake,at+5).is_err(),"{dimension}");}
 a.revoke_rule("ai_ollama",&grant.policy.rule.id,&grant.decision_id,at+6).unwrap();assert_eq!(a.verify_queued(&queued,at+7).err(),Some(Failure::Revoked));assert_eq!(a.rule_status(&f.policy.rule,true).unwrap().remaining_occurrences,Some(1));assert!(occurrences::admit_at(&store,&a,&f.path.join("contexts"),&main,&grant.decision_id,0,||at+7,|_|true,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).is_err());
}
#[test]fn rule_occurrence_crash_after_parent_binding_recovers_only_exact_admission_without_refreshing_approval_or_balance(){
 use crate::pet_runtime::{store::Store,rules::occurrences};
 let f=F::new();let a=f.authority();let grant=f.candidate();a.issue_rule(&grant,grant.approved_at,||true).unwrap();Store::ensure(&f.path.join("goals")).unwrap();let store=Store::existing_verified(&f.path.join("goals"),&f.root()).unwrap().unwrap();let main=crate::chat::Chat::persistent(f.path.join("manual.json"));let at=f.at(0);
 // Lose the native continuation immediately after durable parent binding.
 let current=|_:&RulePolicy|!serde_json::from_slice::<serde_json::Value>(&f.bytes(&grant)).unwrap()["body"]["reservations"].as_array().unwrap().iter().any(|r|r.get("admission").is_some());
 assert!(occurrences::admit_at(&store,&a,&f.path.join("contexts"),&main,&grant.decision_id,0,||at,current,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).is_err());
 let original=a.rule_grant(&grant.decision_id).unwrap().reservations[0].admission.clone().unwrap();let bytes=f.bytes(&grant);let reopened=Authority::existing(&f.root()).unwrap();let done=occurrences::admit_at(&store,&reopened,&f.path.join("contexts"),&main,&grant.decision_id,0,||at+5000,|_|true,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).unwrap();let g=store.read(&done.goal_id,"ai_ollama").unwrap();assert_eq!(g.admissions.len(),1);assert_eq!(g.admissions[0].spec.approved_at,at);assert_eq!(g.admissions[0].spec.budget.deadline_at,at+900000);assert!(original.matches(&g.admissions[0].spec));assert_eq!(f.bytes(&grant),bytes);
}
#[test]fn rule_occurrence_pending_binding_recovers_original_signed_snapshot_and_restores_as_history_only(){
 use crate::pet_runtime::{store::Store,rules::occurrences};
 let f=F::new();let mut a=f.authority();let grant=f.candidate();a.issue_rule(&grant,grant.approved_at,||true).unwrap();let at=f.at(0);a.reserve_rule_occurrence(&grant.decision_id,&grant.policy.hash().unwrap(),0,at,||true).unwrap();Store::ensure(&f.path.join("goals")).unwrap();let store=Store::existing_verified(&f.path.join("goals"),&f.root()).unwrap().unwrap();let main=crate::chat::Chat::persistent(f.path.join("manual.json"));
 a.fault=Fault::BeforeGrantPublish;assert!(occurrences::admit_at(&store,&a,&f.path.join("contexts"),&main,&grant.decision_id,0,||at,|_|true,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).is_err());a.fault=Fault::None;let status=a.rule_status(&f.policy.rule,true).unwrap();assert_eq!(status.pending_artifacts,1);assert_eq!(status.remaining_occurrences,None);let original=a.rule_grant(&grant.decision_id).unwrap().reservations[0].admission.clone().unwrap();
 let done=occurrences::admit_at(&store,&a,&f.path.join("contexts"),&main,&grant.decision_id,0,||at+1000,|_|true,|g,n|Ok(rule_occurrence_evidence(g,&f.policy,n))).unwrap();let goal=store.read(&done.goal_id,"ai_ollama").unwrap();assert!(original.matches(&goal.admissions[0].spec));assert_eq!(a.rule_status(&f.policy.rule,true).unwrap().pending_artifacts,0);
 let before=f.bytes(&grant);let review=restoration::Review::capture(&f.root(),&f.path.join("goals"),&"d".repeat(64),||Ok(())).unwrap();let fingerprint=review.fingerprint().unwrap();review.apply(&fingerprint,||Ok(())).unwrap();assert_eq!(f.bytes(&grant),before);let restored=Authority::existing(&f.root()).unwrap();assert!(restored.verify_queued(&goal,at+2000).is_err());assert!(restored.rule_status(&f.policy.rule,true).unwrap().snapshot_only);assert!(restored.rule_grant(&grant.decision_id).is_err());
}

fn automatic_fixture()->(F,RuleGrant){let mut f=F::new();f.policy.schema_version=2;f.policy.scope=RuleScope::BoundedAutomaticRuleRecurrenceWithoutSelectedData;f.policy.dispatch=Some(crate::pet_runtime::rules::DispatchPolicy::bounded());let mut g=f.candidate();g.schema_version=3;assert!(g.valid());(f,g)}
#[test]fn rule_scheduled_metadata_uses_three_fixed_probes_and_never_renews_them_after_restart_or_original_confirmation(){
 let(f,g)=automatic_fixture();let a=f.authority();a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();
 for n in 0..3{let now=at+n*30002;let p=a.begin_rule_probe(&g.decision_id,&r.occurrence_id,now,||true).unwrap();assert_eq!(p.expires_at,now+6000);a.rule_probe_current(&g.decision_id,&r.occurrence_id,&p,now+1).unwrap();let mut fake=p.clone();fake.scope_hash="e".repeat(64);assert!(a.rule_probe_current(&g.decision_id,&r.occurrence_id,&fake,now+1).is_err());assert!(a.begin_rule_probe(&g.decision_id,&r.occurrence_id,now+2,||true).is_err());a.finish_rule_probe(&g.decision_id,&r.occurrence_id,&p,false,now+1).unwrap();if n<2{assert!(a.begin_rule_probe(&g.decision_id,&r.occurrence_id,now+2,||true).is_err());}}
 let bytes=f.bytes(&g);let reopened=Authority::existing(&f.root()).unwrap();assert_eq!(reopened.rule_grant(&g.decision_id).unwrap().reservations[0].readiness.len(),3);assert_eq!(reopened.begin_rule_probe(&g.decision_id,&r.occurrence_id,at+100000,||true).err(),Some(Failure::Capacity));assert_eq!(reopened.issue_rule(&g,at+100001,||true).unwrap().reservations[0].readiness.len(),3);assert_eq!(f.bytes(&g),bytes);assert_eq!(reopened.rule_status(&f.policy.rule,true).unwrap().remaining_occurrences,Some(1));
 let manual=F::new();let m=manual.authority();let grant=manual.candidate();m.issue_rule(&grant,grant.approved_at,||true).unwrap();let r=m.reserve_rule_occurrence(&grant.decision_id,&grant.policy.hash().unwrap(),0,manual.at(0),||true).unwrap();assert!(m.begin_rule_probe(&grant.decision_id,&r.occurrence_id,manual.at(0),||true).is_err());assert!(!m.rule_status(&manual.policy.rule,true).unwrap().dispatch_available);
}
#[test]fn rule_scheduled_metadata_uncertainty_and_expiry_do_not_prove_io_ended_or_allow_another_probe(){
 let(f,g)=automatic_fixture();let mut a=f.authority();a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();a.fault=Fault::AfterGrantPublish;assert!(a.begin_rule_probe(&g.decision_id,&r.occurrence_id,at,||true).is_err());a.fault=Fault::None;let original=a.rule_grant(&g.decision_id).unwrap();let probe=original.reservations[0].readiness[0].clone();let bytes=f.bytes(&g);let reopened=Authority::existing(&f.root()).unwrap();assert_eq!(reopened.begin_rule_probe(&g.decision_id,&r.occurrence_id,at+60001,||true).err(),Some(Failure::RecoveryRequired));assert_eq!(f.bytes(&g),bytes);assert!(!reopened.rule_status(&f.policy.rule,true).unwrap().dispatch_available);
 reopened.revoke_rule("ai_ollama",&f.policy.rule.id,&g.decision_id,at+60002).unwrap();assert!(reopened.rule_probe_current(&g.decision_id,&r.occurrence_id,&probe,at+60003).is_err());reopened.finish_rule_probe(&g.decision_id,&r.occurrence_id,&probe,false,at+60003).unwrap();assert_eq!(reopened.rule_grant(&g.decision_id).unwrap().phase,RulePhase::Revoked);
 let(f,g)=automatic_fixture();let mut a=f.authority();a.issue_rule(&g,g.approved_at,||true).unwrap();let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).unwrap();a.fault=Fault::BeforeGrantPublish;assert!(a.begin_rule_probe(&g.decision_id,&r.occurrence_id,f.at(0),||true).is_err());a.fault=Fault::None;assert!(a.rule_grant(&g.decision_id).is_err());let before=fs::read_dir(f.root()).unwrap().count();assert!(a.begin_rule_probe(&g.decision_id,&r.occurrence_id,f.at(0)+60001,||true).is_err());assert_eq!(fs::read_dir(f.root()).unwrap().count(),before);assert_eq!(a.rule_status(&f.policy.rule,true).unwrap().pending_artifacts,1);
}
#[test]fn rule_scheduled_due_selection_obeys_missed_policy_without_avalanche_or_promoting_a_manual_grant(){
 use crate::pet_runtime::{store::Store,rules::scheduled};
 let(f,mut g)=automatic_fixture();let store=Store::ensure(&f.path.join("goals")).unwrap();assert_eq!(scheduled::due(&g,&[],0,&store,f.at(0)-1).unwrap(),None);assert_eq!(scheduled::due(&g,&[],0,&store,f.at(2)+1).unwrap(),Some(2));assert_eq!(scheduled::due(&g,&[],0,&store,f.at(2)+400000).unwrap(),None);
 g.policy.rule.definition.missed=Missed::Coalesce;assert_eq!(scheduled::due(&g,&[],0,&store,f.at(2)+400000).unwrap(),Some(2));g.policy.rule.definition.missed=Missed::Decision;assert!(scheduled::due(&g,&[],0,&store,f.at(2)+1).is_err());g.policy.rule.definition.missed=Missed::Skip;g.policy.schedule.slots[0].resolution=crate::pet_calendar::schedule::Resolution::DecisionGap;g.policy.schedule.slots[0].at=None;assert!(scheduled::due(&g,&[],0,&store,f.at(2)+1).is_err());let manual=F::new().candidate();assert_eq!(scheduled::due(&manual,&[],0,&store,f.at(2)+1).unwrap(),None);
}

#[test]fn rule_authority_temporal_status_uses_original_expiry_and_authenticated_clock_without_renewal(){
 let(f,g)=automatic_fixture();let start=g.updated_at;let end=g.policy.valid_until;
 for(now,phase,dispatch)in[(start,"authorizedScheduled",true),(end-1,"authorizedScheduled",true),(end,"expired",false),(end+1,"expired",false),(start-1,"authorityTimeUncertain",false),(0,"authorityTimeUncertain",false),(8_640_000_000_000_001,"authorityTimeUncertain",false)]{
  let s=RuleAuthorityStatus::of_at(&g,true,false,now);assert_eq!(s.phase,phase);assert_eq!(s.dispatch_available,dispatch);assert!(!s.execution_available);assert_eq!(s.observed_at,now);assert_eq!(s.remaining_occurrences,Some(2));assert_eq!(s.policy,Some(g.policy.clone()));
 }
 assert_eq!(RuleAuthorityStatus::of_at(&g,false,false,end).phase,"expired");assert_eq!(RuleAuthorityStatus::of_at(&g,false,false,start).phase,"configurationChanged");let s=RuleAuthorityStatus::of_at(&g,true,true,end);assert_eq!(s.phase,"restoredSnapshotOnly");assert!(!s.dispatch_available&&s.snapshot_only);
 let mut revoked=g.clone();revoked.phase=RulePhase::Revoked;revoked.revoked_at=Some(start);assert_eq!(RuleAuthorityStatus::of_at(&revoked,true,false,end).phase,"revoked");assert!(!RuleAuthorityStatus::of_at(&revoked,true,false,start).dispatch_available);
 let a=f.authority();a.issue_rule(&g,g.approved_at,||true).unwrap();let before=f.bytes(&g);let native=a.rule_status(&g.policy.rule,true).unwrap();assert!(native.observed_at>0);if native.observed_at>=end{assert_eq!(native.phase,"expired");assert!(!native.dispatch_available);}else if native.observed_at<start{assert_eq!(native.phase,"authorityTimeUncertain");assert!(!native.dispatch_available);}assert_eq!(f.bytes(&g),before);
}

/// Explicit offline producer for one expired signed test rule, only in a marked
/// generated QA profile. It is never a production command or user grant.
#[test]#[ignore]fn rule_status_expired_fixture(){
 let Ok(raw)=std::env::var("COUCOU_TEMPORAL_FIXTURE_NATIVE")else{return;};let target=PathBuf::from(raw);assert!(target.is_absolute()&&!target.is_symlink());assert_eq!(fs::read(target.join(".qa-temporal-profile")).unwrap(),b"coucou-generated-temporal-status-only");
 let(f,g)=automatic_fixture();assert!(crate::assistant_memory::now()>g.policy.valid_until);let root=target.join("pet-runtime-consent-v1");if root.exists(){assert!(fs::read_dir(&root).unwrap().all(|f|matches!(f.unwrap().file_name().to_str(),Some(".key.json"|".catalog.lock"|".protocol.json"))));}
 let a=Authority::ensure(&root).unwrap();a.issue_rule(&g,g.approved_at,||true).unwrap();let book=target.join("pet-rules.json");if book.exists(){let old:serde_json::Value=serde_json::from_slice(&fs::read(&book).unwrap()).unwrap();assert!(old["rules"].as_array().unwrap().is_empty());}fs::write(&book,serde_json::to_vec(&json!({"version":1,"generation":1,"rules":[g.policy.rule]})).unwrap()).unwrap();fs::set_permissions(&book,fs::Permissions::from_mode(0o600)).unwrap();drop(f);
}

#[test]fn rule_calendar_selection_cannot_borrow_a_without_selected_data_execution_grant(){
 let f=F::new();let mut candidate=f.candidate();assert!(candidate.policy.valid());
 candidate.policy.rule.definition.data_scope=serde_json::from_value(json!({"calendars":{"sources":[{"id":"work","revision":1}]}})).unwrap();
 assert!(!candidate.policy.valid());assert!(!candidate.valid());
 assert!(f.authority().issue_rule(&candidate,candidate.approved_at,||true).is_err());
}

fn future_calendar_candidate(f:&F)->RuleGrant{
 use crate::pet_runtime::rules::{DataScope,CalendarReference,CalendarPolicy,CalendarOrigin,CalendarConsent};
 let mut grant=f.candidate();grant.schema_version=8;grant.policy.schema_version=3;grant.policy.scope=RuleScope::BoundedRuleRecurrenceWithFutureCalendarData;
 grant.policy.rule.definition.data_scope=DataScope::Calendars{sources:vec![CalendarReference{id:"work".into(),revision:2}]};
 grant.policy.calendar_data=Some(CalendarPolicy{schema_version:1,reader_revision:"a".repeat(64),consent:CalendarConsent::BoundedRuleFutureCalendarData,sources:vec![CalendarOrigin{id:"work".into(),revision:2,name:"Synthetic work calendar".into(),path:"/synthetic/work.ics".into(),enrollment_binding:"b".repeat(64)}],data_class:"nativeCalendarEventsV1".into(),window:"occurrenceCivilDay".into(),max_data_bytes:65536,max_input_bytes:262144,max_capture_seconds:10,max_reads_per_occurrence:1});grant
}

#[test]fn future_calendar_read_is_owned_once_and_keeps_original_occurrence_partition_after_reopen(){
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);
 let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();assert!(owned.current(&a,at+1));
 let record=a.rule_grant(&g.decision_id).unwrap();assert_eq!(record.reservations.len(),1);let allocation=&record.reservations[0];assert_eq!(allocation.calendar_read.as_ref(),Some(owned.read()));assert_eq!(owned.read().expires_at,at+10000);assert_eq!(allocation.model_calls,1);assert!(allocation.admission.is_none());
 assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,at+1,||true).is_err());
 let bytes=f.bytes(&g);let reopened=Authority::existing(&f.root()).unwrap();assert_eq!(reopened.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+10001,||true).err(),Some(Failure::RecoveryRequired));assert_eq!(f.bytes(&g),bytes);
 assert_eq!(reopened.issue_rule(&g,at+10002,||true).unwrap(),record);assert_eq!(f.bytes(&g),bytes);drop(owned);
 assert_eq!(reopened.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+10003,||true).err(),Some(Failure::RecoveryRequired));
}

#[test]fn future_calendar_capture_seals_only_its_goal_and_never_opens_model_dispatch(){
 use crate::pet_runtime::rules::CalendarReadOutcome;
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();
 let goal=model::id_for(&g.policy.goal_spec(0).unwrap());let result=CalendarReadOutcome::Captured{goal_id:goal,context_hash:"c".repeat(64)};
 assert!(a.finish_rule_calendar_read(&owned,CalendarReadOutcome::Captured{goal_id:"wrong-goal".into(),context_hash:"c".repeat(64)},at+1,||true).is_err());
 a.finish_rule_calendar_read(&owned,result.clone(),at+2,||true).unwrap();let record=a.rule_grant(&g.decision_id).unwrap();assert!(record.valid());assert_eq!(record.reservations[0].calendar_read.as_ref().unwrap().outcome,Some(result.clone()));assert!(!owned.current(&a,at+3));
 let bytes=f.bytes(&g);a.finish_rule_calendar_read(&owned,result,at+4,||true).unwrap();assert_eq!(f.bytes(&g),bytes);assert_eq!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+5,||true).err(),Some(Failure::RecoveryRequired));
 let status=RuleAuthorityStatus::of_at(&record,true,false,at+5);assert_eq!(status.phase,"authorizedNotDispatched");assert!(!status.dispatch_available&&!status.execution_available);assert_eq!(status.committed_model_calls,1);
 for mutation in ["owner","scope","expiry"]{let mut invalid=record.clone();let read=invalid.reservations[0].calendar_read.as_mut().unwrap();match mutation{"owner"=>read.owner.operation_id="forged".into(),"scope"=>read.scope_hash="d".repeat(64),_=>read.expires_at+=1};assert!(!invalid.valid(),"{mutation}");}
}

#[test]fn future_calendar_expiry_or_revocation_allows_only_observed_stop_not_success_or_retry(){
 use crate::pet_runtime::rules::CalendarReadOutcome;
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();
 assert!(!owned.current(&a,at+10000));let captured=CalendarReadOutcome::Captured{goal_id:model::id_for(&g.policy.goal_spec(0).unwrap()),context_hash:"e".repeat(64)};assert!(a.finish_rule_calendar_read(&owned,captured,at+10000,||true).is_err());
 a.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,at+10001).unwrap();a.finish_rule_calendar_read(&owned,CalendarReadOutcome::Stopped,at+10002,||false).unwrap();let record=a.rule_grant(&g.decision_id).unwrap();assert_eq!(record.phase,RulePhase::Revoked);assert_eq!(record.reservations.len(),1);assert_eq!(record.reservations[0].calendar_read.as_ref().unwrap().outcome,Some(CalendarReadOutcome::Stopped));assert!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+10003,||true).is_err());
}

#[test]fn future_calendar_uncertain_publication_never_mints_a_second_read_or_partition(){
 let f=F::new();let mut a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);
 a.fault=Fault::AfterGrantPublish;assert!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).is_err());a.fault=Fault::None;let allocated=a.rule_grant(&g.decision_id).unwrap();assert_eq!(allocated.reservations.len(),1);assert!(allocated.reservations[0].calendar_read.is_none());
 a.fault=Fault::AfterGrantPublish;assert!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+1,||true).is_err());a.fault=Fault::None;let pending=a.rule_grant(&g.decision_id).unwrap();assert_eq!(pending.reservations.len(),1);assert!(pending.reservations[0].calendar_read.is_some());let bytes=f.bytes(&g);
 assert_eq!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+11000,||true).err(),Some(Failure::RecoveryRequired));assert_eq!(f.bytes(&g),bytes);
}
#[test]fn rule_future_calendar_requires_both_own_consents_and_exact_policy_scope(){
 use crate::pet_runtime::rules::CalendarConsent;let f=F::new();let g=future_calendar_candidate(&f);assert!(g.valid());
 let data=Some(CalendarConsent::BoundedRuleFutureCalendarData);let scope=Some(RuleScope::BoundedRuleRecurrenceWithFutureCalendarData);
 assert!(!g.policy.consent_matches(Some(RuleScope::BoundedRuleRecurrenceWithoutSelectedData),data,None));assert!(!g.policy.consent_matches(scope,None,None));assert!(g.policy.consent_matches(scope,data,None));assert!(!f.policy.consent_matches(Some(f.policy.scope),data,None));
 for mode in ["version","source","bytes","window"]{let mut bad=g.clone();match mode{"version"=>bad.schema_version=7,"source"=>bad.policy.calendar_data.as_mut().unwrap().sources[0].revision+=1,"bytes"=>bad.policy.calendar_data.as_mut().unwrap().max_data_bytes+=1,_=>bad.policy.calendar_data.as_mut().unwrap().window="anyTime".into()};assert!(!bad.valid(),"{mode}");}
}
#[test]fn rule_future_calendar_authority_is_durable_revocable_but_cannot_allocate_before_materialization(){
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();assert_eq!(a.rule_grant(&g.decision_id).unwrap(),g);
 let observed=a.rule_status(&g.policy.rule,true).unwrap();assert!(!observed.dispatch_available);let status=RuleAuthorityStatus::of_at(&a.rule_grant(&g.decision_id).unwrap(),true,false,g.approved_at+1);assert_eq!(status.phase,"authorizedNotDispatched");assert!(!status.dispatch_available);assert_eq!(status.committed_model_calls,0);
 assert!(a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).is_err());assert!(a.rule_grant(&g.decision_id).unwrap().reservations.is_empty());
 a.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,g.approved_at+1).unwrap();let same=a.issue_rule(&g,g.approved_at+2,||true).unwrap();assert_eq!(same.phase,RulePhase::Revoked);assert!(same.reservations.is_empty());
}

#[test]fn future_calendar_status_requires_explicit_automatic_scope_and_preserves_blocking_state(){
 let f=F::new();let mut g=future_calendar_candidate(&f);g.policy.dispatch=Some(crate::pet_runtime::rules::DispatchPolicy::bounded());assert!(g.valid());
 let mut status=crate::pet_runtime::rules::RuleAuthorityStatus::of_at(&g,false,false,f.at(0));
 status.refresh_configuration(true);assert_eq!(status.phase,"authorizedScheduled");assert!(status.dispatch_available&&!status.execution_available);
 status.refresh_configuration(false);assert_eq!(status.phase,"configurationChanged");assert!(!status.dispatch_available);
 for phase in ["revoked","expired","authorityTimeUncertain","restoredSnapshotOnly","reconciliationRequired"]{status.phase=phase;status.refresh_configuration(true);assert_eq!(status.phase,phase);assert!(!status.dispatch_available);}
 let (_,plain)=automatic_fixture();let mut status=crate::pet_runtime::rules::RuleAuthorityStatus::of_at(&plain,false,false,f.at(0));status.refresh_configuration(true);assert_eq!(status.phase,"authorizedScheduled");assert!(status.dispatch_available&&!status.execution_available);
}
#[test]fn future_calendar_materializer_refuses_stale_configuration_without_creating_an_occurrence_or_goal(){
 use crate::pet_runtime::{rules::calendar_materialization,store::Store};use std::sync::Arc;
 let f=F::new();let a=Arc::new(f.authority());let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let before=f.bytes(&g);
 let goals=f.path.join("goals");Store::ensure(&goals).unwrap();let store=Store::existing_verified(&goals,&f.root()).unwrap().unwrap();let main=crate::chat::Chat::persistent(f.path.join("main.json"));let at=f.at(0);
 assert!(calendar_materialization::prepare_at(&store,a,&f.path.join("contexts"),&main,&g.decision_id,0,Arc::new(move||at),Arc::new(|_|false)).is_err());assert_eq!(f.bytes(&g),before);assert!(!f.path.join("contexts").exists());assert!(!f.path.join("main.json").exists());
}

#[test]fn future_calendar_child_binds_own_completed_read_context_destination_and_original_partition(){
 use crate::pet_runtime::{rules::{occurrences,CalendarConsent,CalendarOrigin,CalendarReadOutcome,DataScope,CalendarReference},store::Store};
 use crate::pet_memory::context_model::{GoalGrant,GoalConsent};
 // Existing no-data admission supplies the ordinary reviewed plan/identity;
 // it is not a transferable execution proof for the second private Authority.
 let base=F::new();let ba=base.authority();let bg=base.candidate();ba.issue_rule(&bg,bg.approved_at,||true).unwrap();Store::ensure(&base.path.join("goals")).unwrap();let st=Store::existing_verified(&base.path.join("goals"),&base.root()).unwrap().unwrap();let main=crate::chat::Chat::persistent(base.path.join("main.json"));let original=occurrences::admit_at(&st,&ba,&base.path.join("contexts"),&main,&bg.decision_id,0,||base.at(0),|_|true,|g,n|Ok(rule_occurrence_evidence(g,&base.policy,n))).unwrap();let original=st.read(&original.goal_id,"ai_ollama").unwrap();let mut spec=original.admissions[0].spec.clone();
 let f=F::new();let a=f.authority();let mut g=future_calendar_candidate(&f);let at=f.at(0);
 let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();let query=crate::pet_calendar::day::explicit("2026-10-24","Europe/Lisbon","2026-10-24T08:00:00Z",||true).unwrap();let(_enrolled,calendars,selected)=crate::pet_calendar::goal::rule_fixture(&spec.goal_id,query,at);
 g.policy.rule.definition.data_scope=DataScope::Calendars{sources:selected.iter().map(|s|CalendarReference{id:s.id().into(),revision:s.revision()}).collect()};g.policy.calendar_data.as_mut().unwrap().sources=selected.iter().map(|s|CalendarOrigin{id:s.id().into(),revision:s.revision(),name:s.name().into(),path:s.path().into(),enrollment_binding:s.binding()}).collect();assert_eq!(g.policy.calendar_data.as_ref().unwrap().consent,CalendarConsent::BoundedRuleFutureCalendarData);assert!(g.valid());a.issue_rule(&g,g.approved_at,||true).unwrap();
 spec.schema_version=30;spec.context.schema_version=11;spec.context.calendars=Some(calendars.clone());spec.context_hash=model::fingerprint(&spec.context).unwrap();spec.memory_data=Some(GoalGrant{schema_version:11,consent:GoalConsent::BoundedRuleFutureCalendarData,destination:g.policy.destination.clone(),max_input_bytes:262144,calendar_selection:Some(calendars.selection),received_request:None});spec.approved_at=at+3;
 let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();assert!(a.rule_grant(&g.decision_id).unwrap().reference(&a.rule_grant(&g.decision_id).unwrap().reservations[0]).is_err());
 a.finish_rule_calendar_read(&owned,CalendarReadOutcome::Captured{goal_id:spec.goal_id.clone(),context_hash:spec.context_hash.clone()},at+1,||true).unwrap();let parent=a.rule_grant(&g.decision_id).unwrap();let allocation=&parent.reservations[0];spec.rule_occurrence=Some(parent.reference(allocation).unwrap());assert!(spec.binding_format()&&spec.finite_response()&&spec.execution_requested());assert!(crate::pet_runtime::plans::matches(&original,&spec),"the new own-DATA admission must retain the ordinary exact reviewed plan");assert!(parent.child_matches(allocation,&spec));
 let before=f.bytes(&g);
 for mode in ["read","context","scope","dataConsent","destination","source","budget","deadline","legacySpec"]{
  let mut bad=spec.clone();match mode{"read"=>bad.rule_occurrence.as_mut().unwrap().calendar_read_id=Some(nonce().unwrap()),"context"=>{bad.context.seed_hash="f".repeat(64);bad.context_hash=model::fingerprint(&bad.context).unwrap();},"scope"=>bad.rule_occurrence.as_mut().unwrap().schema_version=1,"dataConsent"=>bad.memory_data.as_mut().unwrap().consent=GoalConsent::BoundedGoalContext,"destination"=>if let crate::pet_memory::context_model::Destination::Server{endpoint,..}=&mut bad.memory_data.as_mut().unwrap().destination{*endpoint="http://127.0.0.1:19001/v1/chat/completions".into();},"source"=>{bad.memory_data.as_mut().unwrap().calendar_selection.as_mut().unwrap().query.limit+=1;},"budget"=>bad.budget.max_model_calls=2,"deadline"=>bad.budget.deadline_at+=1,_=>bad.schema_version=29};
  assert!(!parent.child_matches(allocation,&bad),"{mode}");assert!(a.bind_rule_admission(&bad,at+4,||true).is_err(),"{mode}");assert_eq!(f.bytes(&g),before,"{mode}");
 }
 a.bind_rule_admission(&spec,at+4,||true).unwrap();let bound=a.rule_grant(&g.decision_id).unwrap();assert_eq!(bound.schema_version,8);assert_eq!(bound.reservations.len(),1);assert!(bound.reservations[0].admission.as_ref().unwrap().matches(&spec));let bytes=f.bytes(&g);a.bind_rule_admission(&spec,at+5,||true).unwrap();assert_eq!(f.bytes(&g),bytes);a.rule_admission_current_unlocked(&spec,at+6).unwrap();let reopened=Authority::existing(&f.root()).unwrap();reopened.rule_admission_current_unlocked(&spec,at+7).unwrap();reopened.revoke_rule("ai_ollama",&g.policy.rule.id,&g.decision_id,at+8).unwrap();assert!(reopened.rule_admission_current_unlocked(&spec,at+9).is_err());
}

#[test]fn future_calendar_metadata_reuses_owned_bounded_probes_only_after_its_completed_capture(){
 use crate::pet_runtime::rules::{CalendarReadOutcome,DispatchPolicy};
 let f=F::new();let a=f.authority();let mut g=future_calendar_candidate(&f);g.policy.dispatch=Some(DispatchPolicy::bounded());assert!(g.valid());a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();let occurrence=g.policy.occurrence_id(0).unwrap();
 assert!(a.begin_owned_rule_probe(&g.decision_id,&occurrence,at+1,||true).is_err());let goal=model::id_for(&g.policy.goal_spec(0).unwrap());a.finish_rule_calendar_read(&owned,CalendarReadOutcome::Captured{goal_id:goal,context_hash:"d".repeat(64)},at+2,||true).unwrap();
 for n in 0..3{
  let now=at+3+n*30002;let p=a.begin_owned_rule_probe(&g.decision_id,&occurrence,now,||true).unwrap();assert!(p.current(&a));a.rule_probe_current(&g.decision_id,&occurrence,p.probe(),now+1).unwrap();a.finish_rule_probe(&g.decision_id,&occurrence,p.probe(),false,now+1).unwrap();
 }
 assert!(a.begin_owned_rule_probe(&g.decision_id,&occurrence,at+100000,||true).is_err());let state=a.rule_grant(&g.decision_id).unwrap();assert_eq!(state.schema_version,8);assert_eq!(state.reservations.len(),1);assert_eq!(state.reservations[0].readiness.len(),3);assert_eq!(state.reservations[0].calendar_read.as_ref().unwrap().id,owned.read().id);assert!(a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at+100001,||true).is_err());
}

#[test]fn future_calendar_notification_is_own_explicit_bounded_policy_and_legacy_bytes_remain_unchanged(){
 use crate::pet_runtime::rules::NotificationPolicy;
 let f=F::new();let a=f.authority();let legacy=future_calendar_candidate(&f);let encoded=serde_json::to_value(&legacy).unwrap();assert!(encoded["policy"].get("notification").is_none());assert_eq!(serde_json::from_value::<RuleGrant>(encoded.clone()).unwrap(),legacy);
 let mut g=legacy.clone();g.policy.notification=Some(NotificationPolicy::bounded());assert!(!g.valid(),"old policy/authority versions cannot carry notification consent");g.policy.schema_version=4;g.schema_version=9;assert!(g.valid());
 for kind in ["channel","body","priority","sound","count","window","oldVersion","noData"]{let mut bad=g.clone();let p=bad.policy.notification.as_mut().unwrap();match kind{"channel"=>p.channel="externalWebhook".into(),"body"=>p.content="calendarText".into(),"priority"=>p.priority="urgent".into(),"sound"=>p.silent=false,"count"=>p.max_per_occurrence=2,"window"=>p.max_delay_millis+=1,"oldVersion"=>bad.schema_version=8,_=>{bad.policy.calendar_data=None;bad.policy.rule.definition.data_scope=crate::pet_runtime::rules::DataScope::None;}}assert!(!bad.valid(),"{kind}");assert!(a.issue_rule(&bad,bad.approved_at,||true).is_err(),"{kind}");}
 a.issue_rule(&g,g.approved_at,||true).unwrap();let before=f.bytes(&g);let reopened=Authority::existing(&f.root()).unwrap();assert_eq!(reopened.rule_grant(&g.decision_id).unwrap(),g);assert_eq!(f.bytes(&g),before);assert_eq!(serde_json::to_value(legacy).unwrap(),encoded);
}
#[test]fn notification_window_intersects_original_deadline_and_never_rolls_forward(){
 use crate::pet_runtime::rules::NotificationPolicy;
 let f=F::new();let a=f.authority();let g=f.candidate();a.issue_rule(&g,g.approved_at,||true).unwrap();let mut r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(0),||true).unwrap();let p=NotificationPolicy::bounded();assert_eq!(p.until(&r),r.at+300000);r.deadline_at=r.at+60000;assert_eq!(p.until(&r),r.at+60000);r.reserved_at=r.at+59000;assert_eq!(p.until(&r),r.at+60000,"late readiness or re-observation must not create a later notice window");
}

#[test]fn project_progress_policy_has_distinct_own_consent_and_allows_no_notes_but_no_self_observation(){
 use crate::pet_runtime::{project_goal_observation::{Selection,Reference},rules::{ProjectProgressPolicy,ProjectConsent,DataScope}};
 let f=F::new();let mut p=f.policy.clone();let project=crate::pet_memory::projects::Project{id:crate::pet_memory::context_model::digest(&serde_json::to_vec(&("coucou-memory-project-v1","ai_ollama","progress-policy")).unwrap())[..32].into(),pet_id:"ai_ollama".into(),creation_request_id:"progress-policy".into(),revision:1,name:"Reviewed progress".into(),conversation_ids:vec!["original-source".into()],enabled:true,created_at:100,updated_at:100};
 let refs=vec![Reference{goal_id:format!("g-{}","f".repeat(64)),spec_hash:"a".repeat(64)}];
 p.rule.definition.data_scope=DataScope::ProjectProgress{source_conversation_id:"original-source".into(),project:crate::pet_memory::context_model::SelectedProject{id:project.id.clone(),revision:1},goals:refs.clone(),include_latest_response:true,notes:vec![]};p.schema_version=7;p.scope=RuleScope::BoundedRuleRecurrenceWithProjectGoalProgress;
 p.project_progress_data=Some(ProjectProgressPolicy{schema_version:1,consent:ProjectConsent::BoundedRuleProjectGoalProgressData,selection:Selection{schema_version:1,project,source_conversation_id:"original-source".into(),goals:refs,include_latest_response:true},notes:None,max_data_bytes:131072,max_input_bytes:262144,max_capture_seconds:10,max_reads_per_occurrence:1});
 assert!(p.valid());assert!(p.selected_data());assert!(p.consent_matches(Some(p.scope),None,Some(ProjectConsent::BoundedRuleProjectGoalProgressData)));assert!(!p.consent_matches(Some(p.scope),None,Some(ProjectConsent::BoundedRulePinnedProjectData)));assert!(!p.consent_matches(Some(p.scope),None,None));assert!(p.calendar_options(Some(1),false).is_err());assert!(p.calendar_options(None,true).is_err());p.calendar_options(None,false).unwrap();assert_eq!(p.schema_version,7);
 let mut grant=f.candidate();grant.policy=p.clone();grant.schema_version=grant.required_version();assert_eq!(grant.schema_version,12);assert!(grant.valid());let a=f.authority();a.issue_rule(&grant,grant.approved_at,||true).unwrap();assert_eq!(a.rule_grant(&grant.decision_id).unwrap(),grant);
 let target=model::id_for(&p.goal_spec(0).unwrap());p.project_progress_data.as_mut().unwrap().selection.goals[0].goal_id=target.clone();if let DataScope::ProjectProgress{goals,..}=&mut p.rule.definition.data_scope{goals[0].goal_id=target;}assert!(!p.valid(),"reject self-monitoring before spending an occurrence capture");
}

#[test]fn captured_data_seal_waits_for_the_same_native_reader_without_extracting_again(){
 use crate::pet_runtime::rules::{CalendarReadOutcome,calendar_materialization::seal_capture};
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();let original=owned.read().clone();
 let goal=model::id_for(&g.policy.goal_spec(0).unwrap());let output=CalendarReadOutcome::Captured{goal_id:goal,context_hash:"e".repeat(64)};
 let reader=a.lock_shared().unwrap();let release=std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(180));drop(reader);});
 let result=seal_capture(&a,&owned,output.clone(),&||at+1,&||true);release.join().unwrap();assert!(result.is_ok(),"a prepublication reader stranded a completed capture: {result:?}");
 let saved=a.rule_grant(&g.decision_id).unwrap();assert_eq!(saved.reservations.len(),1);let allocation=&saved.reservations[0];let read=allocation.calendar_read.as_ref().unwrap();assert_eq!(read.id,original.id);assert_eq!(read.prepared_at,original.prepared_at);assert_eq!(read.expires_at,original.expires_at);assert_eq!(read.outcome,Some(output));assert!(allocation.admission.is_none());assert_eq!(allocation.model_calls,1);assert_eq!(a.rule_status(&g.policy.rule,true).unwrap().remaining_occurrences,Some(1));
}
#[test]fn captured_data_seal_does_not_retry_an_uncertain_publication(){
 use crate::pet_runtime::rules::{CalendarReadOutcome,calendar_materialization::seal_capture};
 let f=F::new();let mut a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();let output=CalendarReadOutcome::Captured{goal_id:model::id_for(&g.policy.goal_spec(0).unwrap()),context_hash:"e".repeat(64)};
 a.fault=Fault::AfterGrantPublish;assert_eq!(seal_capture(&a,&owned,output.clone(),&||at+1,&||true),Err(model::Error::StorageUnavailable));a.fault=Fault::None;let saved=a.rule_grant(&g.decision_id).unwrap();assert_eq!(saved.reservations.len(),1);let read=saved.reservations[0].calendar_read.as_ref().unwrap();assert_eq!(read.outcome,Some(output));assert_eq!(read.id,owned.read().id);assert_eq!(read.finished_at,Some(at+1));assert!(saved.reservations[0].admission.is_none());
}
#[test]fn captured_data_seal_cannot_extend_the_original_read_deadline_while_waiting(){
 use crate::pet_runtime::rules::{CalendarReadOutcome,calendar_materialization::seal_capture};use std::sync::{Arc,atomic::{AtomicU64,Ordering}};
 let f=F::new();let a=f.authority();let g=future_calendar_candidate(&f);a.issue_rule(&g,g.approved_at,||true).unwrap();let at=f.at(0);let owned=a.begin_owned_rule_calendar(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();let before=f.bytes(&g);let now=Arc::new(AtomicU64::new(at+1));let moved=now.clone();let reader=a.lock_shared().unwrap();let release=std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(140));moved.store(at+10000,Ordering::Release);std::thread::sleep(std::time::Duration::from_millis(40));drop(reader);});
 let result=seal_capture(&a,&owned,CalendarReadOutcome::Captured{goal_id:model::id_for(&g.policy.goal_spec(0).unwrap()),context_hash:"e".repeat(64)},&||now.load(Ordering::Acquire),&||true);release.join().unwrap();assert!(result.is_err());assert_eq!(f.bytes(&g),before);assert_eq!(owned.read().expires_at,at+10000);
}
