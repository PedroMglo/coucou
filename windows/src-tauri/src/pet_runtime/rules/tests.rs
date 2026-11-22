use super::*;
use serde_json::json;
use std::{fs,os::unix::fs::PermissionsExt,path::PathBuf};
struct Fixture(PathBuf);impl Fixture{fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-rule-{}",super::super::store::nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();Self(path)}fn path(&self)->PathBuf{self.0.join("pet-rules.json")}}impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn owner(pet:&str)->Owner{Owner{pet_id:pet.into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None}}
fn definition()->Definition{serde_json::from_value(json!({"name":"Daily fixture","intent":"Prepare a response without implicit data","criteria":["An independently verified result"],"schedule":{"kind":"daily","firstDate":"2026-10-24","days":3,"localTime":"01:30","timeZone":"Europe/Lisbon","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"decision"},"missed":"coalesce","graceSeconds":300,"dataScope":"none","limits":{"maxOccurrences":3,"maxModelCalls":3,"maxToolCalls":0,"maxOutputBytes":12288,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":900}})).unwrap()}
fn input(pet:&str)->Save{Save{pet_id:pet.into(),expected_revision:"missing".into(),request_id:"native-create-rule".into(),rule_id:None,definition:definition()}}
#[test]fn rule_contract_durable_draft_cas_owner_revision_and_closed_window_never_create_authority(){
 let f=Fixture::new();let mut i=input("ai_ollama");let d=save_at(&f.path(),&i,owner("ai_ollama"),1000,||true).unwrap();assert_eq!(d.rules.len(),1);assert!(!d.execution_available);assert_eq!(d.rules[0].revision,1);let first=fs::read(f.path()).unwrap();let id=d.rules[0].id.clone();assert!(save_at(&f.path(),&i,owner("ai_ollama"),1001,||true).is_err());assert_eq!(fs::read(f.path()).unwrap(),first);
 i.rule_id=Some(id.clone());i.expected_revision=d.revision;i.definition.name="Updated <untrusted> title".into();assert!(save_at(&f.path(),&i,owner("ai_ollama"),1002,||false).is_err());assert_eq!(fs::read(f.path()).unwrap(),first);
 let second=save_at(&f.path(),&i,owner("ai_ollama"),1003,||true).unwrap();assert_eq!(second.rules[0].revision,2);assert_eq!(second.rules[0].created_at,1000);assert_eq!(second.rules[0].id,id);i.pet_id="ai_lmstudio".into();i.expected_revision=second.revision;assert!(save_at(&f.path(),&i,owner("ai_lmstudio"),1004,||true).is_err());assert_eq!(fs::read_dir(&f.0).unwrap().filter_map(Result::ok).filter(|e|e.file_name().to_string_lossy().contains("consent")).count(),0);
 let snapshot=private_registry::load_at::<Book>(&f.path()).unwrap();assert_eq!(document(snapshot,"agent_codex").rules.len(),0);
}
#[test]fn rule_contract_incompatible_book_forged_activation_and_limits_are_refused_without_destroying_data(){
 let f=Fixture::new();let d=save_at(&f.path(),&input("ai_ollama"),owner("ai_ollama"),1000,||true).unwrap();let original=fs::read(f.path()).unwrap();let raw:serde_json::Value=serde_json::from_slice(&original).unwrap();for kind in["version","state","authority"]{let mut bad=raw.clone();match kind{"version"=>bad["version"]=json!(5),"state"=>bad["rules"][0]["state"]=json!("active"),_=>bad["rules"][0]["grant"]=json!("model generated permission")};fs::write(f.path(),serde_json::to_vec(&bad).unwrap()).unwrap();let before=fs::read(f.path()).unwrap();assert!(private_registry::load_at::<Book>(&f.path()).is_err());assert_eq!(fs::read(f.path()).unwrap(),before);}fs::write(f.path(),original).unwrap();
 let mut i=input("ai_ollama");i.expected_revision=d.revision;i.definition.limits.max_model_calls=64;assert!(save_at(&f.path(),&i,owner("ai_ollama"),1001,||true).is_err());let mut bad=serde_json::to_value(definition()).unwrap();bad["dataScope"]=json!("allMemory");assert!(serde_json::from_value::<Definition>(bad).is_err());
}
#[test]fn rule_contract_missed_policies_select_at_most_one_and_never_renew_budget_or_replay_a_cursor(){
 let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();let def=definition();let preview=crate::pet_calendar::schedule::resolve(&def.schedule,||true).unwrap();let last=preview.slots.last().unwrap().at.unwrap();let now=last+600000;
 let skip=backlog(&preview,Missed::Skip,300,None,now);assert!(skip.selected_at.is_none());assert_eq!(skip.skipped,3);let coalesce=backlog(&preview,Missed::Coalesce,300,None,now);assert_eq!(coalesce.selected_at,Some(last));assert_eq!(coalesce.coalesced,2);let decide=backlog(&preview,Missed::Decision,300,None,now);assert!(decide.selected_at.is_none());assert_eq!(decide.needs_decision,3);let cursor=backlog(&preview,Missed::Coalesce,300,Some(last),now);assert!(cursor.selected_at.is_none());let due=backlog(&preview,Missed::Skip,300,None,last);assert_eq!(due.selected_at,Some(last));assert_eq!(due.skipped,2);assert!(!preview.execution_authorized);
}

#[test]fn rule_calendar_selection_draft_roundtrip_upgrades_only_metadata_and_refuses_stale_writes(){
 let f=Fixture::new();let mut i=input("ai_ollama");let old=save_at(&f.path(),&i,owner("ai_ollama"),1000,||true).unwrap();
 let initial:serde_json::Value=serde_json::from_slice(&fs::read(f.path()).unwrap()).unwrap();assert_eq!(initial["version"],1);
 i.rule_id=Some(old.rules[0].id.clone());i.expected_revision=old.revision.clone();
 i.definition.data_scope=serde_json::from_value(json!({"calendars":{"sources":[{"id":"work","revision":2},{"id":"personal","revision":1}]}})).unwrap();
 let updated=save_at(&f.path(),&i,owner("ai_ollama"),1001,||true).unwrap();assert_eq!(updated.schema_version,2);assert_eq!(updated.rules[0].revision,2);assert!(!updated.execution_available);
 let bytes=fs::read(f.path()).unwrap();let raw:serde_json::Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(raw["version"],2);
 assert_eq!(document(private_registry::load_at::<Book>(&f.path()).unwrap(),"ai_ollama").rules[0].definition.data_scope,i.definition.data_scope);
 assert!(save_at(&f.path(),&i,owner("ai_ollama"),1002,||true).is_err());assert_eq!(fs::read(f.path()).unwrap(),bytes);
 let mut forged=raw.clone();forged["version"]=json!(1);assert!(private_registry::Document::validate(&serde_json::from_value::<Book>(forged).unwrap()).is_err());
 i.expected_revision=updated.revision;i.definition.data_scope=DataScope::None;let without=save_at(&f.path(),&i,owner("ai_ollama"),1003,||true).unwrap();assert_eq!(without.schema_version,2,"never silently downgrade a published catalog");
}
#[test]fn rule_calendar_selection_requires_explicit_bounded_unique_source_revisions(){
 for sources in [json!([]),json!([{"id":"work","revision":0}]),json!([{"id":"work","revision":1},{"id":"work","revision":2}]),json!([{"id":"../private","revision":1}])]{
  let mut d=definition();d.data_scope=serde_json::from_value(json!({"calendars":{"sources":sources}})).unwrap();assert!(d.validate().is_err());
 }
 assert!(serde_json::from_value::<DataScope>(json!({"calendars":{"sources":[{"id":"work","revision":1}],"execute":true}})).is_err());
}

fn calendar_scope(selected:&[crate::pet_sources::Selected])->DataScope{DataScope::Calendars{sources:selected.iter().map(|s|CalendarReference{id:s.id().into(),revision:s.revision()}).collect()}}
#[test]fn rule_calendar_selection_native_enrollment_requires_exact_pet_scope_revision_and_no_content_read(){
 let enrolled=crate::pet_sources::test_support::Enrollment::new("ollama","source-cid");let selected=enrolled.pet_collection();assert_eq!(selected.len(),2);
 let scope=calendar_scope(&selected);
 for source in &selected{let path=std::path::Path::new(source.path());assert!(path.starts_with(&enrolled.root));fs::remove_file(path).unwrap();}
 let pinned=scope.capture_enrolled("ollama",selected.clone()).unwrap();assert_eq!(pinned.len(),2,"metadata selection must not need readable ICS content");
 assert!(scope.capture_enrolled("codex",selected.clone()).is_err());
 let mut stale=scope.clone();if let DataScope::Calendars{sources}=&mut stale{sources[0].revision+=1;}assert!(stale.capture_enrolled("ollama",selected.clone()).is_err());
 let mut foreign=scope.clone();if let DataScope::Calendars{sources}=&mut foreign{sources[0].id="not-enrolled".into();}assert!(foreign.capture_enrolled("ollama",selected.clone()).is_err());
 let conversation_fixture=crate::pet_sources::test_support::Enrollment::new("ollama","another-cid");let conversation=conversation_fixture.collection();crate::pet_sources::validate_group(&conversation.iter().collect::<Vec<_>>(),"ollama","another-cid").unwrap();assert!(calendar_scope(&conversation).capture_enrolled("ollama",conversation).is_err());
 enrolled.disable();assert!(scope.capture_enrolled("ollama",selected).is_err(),"revocation cannot reuse a captured source snapshot");
}
#[test]fn rule_calendar_selection_revocation_before_draft_publication_preserves_original_rule_bytes(){
 let enrolled=crate::pet_sources::test_support::Enrollment::new("ollama","source-cid");let selected=enrolled.pet_collection();let scope=calendar_scope(&selected);let pinned=scope.capture_enrolled("ollama",selected).unwrap();
 let f=Fixture::new();let mut i=input("ai_ollama");let old=save_at(&f.path(),&i,owner("ai_ollama"),1000,||true).unwrap();let before=fs::read(f.path()).unwrap();i.rule_id=Some(old.rules[0].id.clone());i.expected_revision=old.revision;i.definition.data_scope=scope;
 let result=save_at_reviewed(&f.path(),&i,owner("ai_ollama"),1001,||crate::pet_sources::validate_pet_group(&pinned.iter().collect::<Vec<_>>(),"ollama").is_ok(),|_|{enrolled.disable();Ok(())});
 assert!(result.is_err());assert_eq!(fs::read(f.path()).unwrap(),before);
}

#[test]fn project_rule_book3_refuses_older_readers_and_never_downgrades_when_data_is_removed(){
 let f=Fixture::new();let mut input=input("ai_ollama");input.definition.data_scope=serde_json::from_value(json!({"project":{"sourceConversationId":"original-native-cid","project":{"id":"a".repeat(32),"revision":1},"notes":[{"id":"selected-note","revision":2}]}})).unwrap();
 let doc=save_at(&f.path(),&input,owner("ai_ollama"),1000,||true).unwrap();assert_eq!(doc.schema_version,3);assert!(!doc.execution_available);let before=fs::read(f.path()).unwrap();let mut old:serde_json::Value=serde_json::from_slice(&before).unwrap();old["version"]=json!(2);fs::write(f.path(),serde_json::to_vec(&old).unwrap()).unwrap();let retained=fs::read(f.path()).unwrap();assert!(private_registry::load_at::<Book>(&f.path()).is_err());assert_eq!(fs::read(f.path()).unwrap(),retained);fs::write(f.path(),before).unwrap();
 input.expected_revision=doc.revision;input.rule_id=Some(doc.rules[0].id.clone());input.definition.data_scope=DataScope::None;let removed=save_at(&f.path(),&input,owner("ai_ollama"),1001,||true).unwrap();assert_eq!(removed.schema_version,3);assert_eq!(removed.rules[0].revision,2);assert_eq!(removed.rules[0].definition.data_scope,DataScope::None);assert!(!removed.execution_available);
}
