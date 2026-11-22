//! Produce only generated, signed runtime states for native product restore QA.
//! The application binary cannot invoke this helper or bypass normal admission.
use super::*;
use std::os::unix::fs::{DirBuilderExt,PermissionsExt,MetadataExt};

fn directory(path:&std::path::Path){fs::DirBuilder::new().recursive(true).mode(0o700).create(path).unwrap();}
fn file(path:&std::path::Path,bytes:&[u8]){
 directory(path.parent().unwrap());assert!(!path.exists());fs::write(path,bytes).unwrap();fs::set_permissions(path,fs::Permissions::from_mode(0o600)).unwrap();
}
fn copy(source:&std::path::Path,target:&std::path::Path){
 let meta=fs::symlink_metadata(source).unwrap();assert!(!meta.file_type().is_symlink());
 if meta.is_dir(){directory(target);for entry in fs::read_dir(source).unwrap(){let entry=entry.unwrap();copy(&entry.path(),&target.join(entry.file_name()));}}
 else{assert!(meta.is_file()&&meta.nlink()==1&&meta.len()<=16*1024*1024);file(target,&fs::read(source).unwrap());}
}
fn inventory(root:&std::path::Path,path:&std::path::Path,out:&mut std::collections::BTreeMap<String,String>){
 for entry in fs::read_dir(path).unwrap(){let entry=entry.unwrap();let path=entry.path();if path.is_dir(){inventory(root,&path,out);}else{out.insert(path.strip_prefix(root).unwrap().to_str().unwrap().into(),crate::pet_runtime::model::hash(&fs::read(path).unwrap()));}}
}

#[test]
#[ignore="explicit generated native resumption profile; no real account or provider"]
fn h3d2_native_resumption_profile(){
 let output=PathBuf::from(std::env::var("COUCOU_H3D2_NATIVE_PROFILE").expect("explicit empty project QA output"));
 assert!(output.is_absolute()&&output.is_dir()&&fs::read_dir(&output).unwrap().next().is_none());
 let mut env=Process::new("ollama","partial-native");env.crash();let closed=env.close();let grant=body(&env.s,&closed);
 let settings:Settings=serde_json::from_slice(&fs::read(env.s.f.path.join("native-qa-settings.json")).unwrap()).unwrap();
 let expected=crate::pet_runtime::native_validity::policy_revision(&env.s.config,&settings,&closed).unwrap();assert_eq!(closed.admissions[0].spec.policy_revision,expected);
 assert!(!settings.autostart&&!settings.hooks_installed&&!settings.sound_enabled);assert_eq!(settings.chat_models.get("ollama").map(String::as_str),Some("fixture-free"));
 let config=output.join("config/coucou");let data=output.join("data/coucou");directory(&config);directory(&data);
 file(&config.join("settings.json"),&serde_json::to_vec(&settings).unwrap());file(&config.join("pets.json"),&serde_json::to_vec(&env.s.config).unwrap());
 let(config,data)=crate::pet_data::fixture_publish_profile(&env.bus.address,PREFIX,crate::pet_data::Layout::new(config,data).unwrap()).unwrap();
 let mapping=[("authority","pet-runtime-consent-v1"),("authority.owners-v1","pet-runtime-consent-v1.owners-v1"),("goals","pet-runtime-v1"),("contexts","pet-runtime-contexts-v1"),("pet-runtime-planning-v1","pet-runtime-planning-v1"),("pet-runtime-results-v1","pet-runtime-results-v1"),("pet-runtime-step-consent-v1","pet-runtime-step-consent-v1")];
 let mut moved=vec![];
 for(source,target)in mapping{let source=env.s.f.path.join(source);if !source.exists(){continue;}let target=config.join(target);assert!(!target.exists());let before=fs::symlink_metadata(&source).unwrap();assert!(!before.file_type().is_symlink());fs::rename(&source,&target).unwrap();let after=fs::symlink_metadata(&target).unwrap();assert_eq!((before.dev(),before.ino()),(after.dev(),after.ino()));moved.push(json!({"target":target,"device":after.dev(),"inode":after.ino()}));}
 let main=env.s.f.path.join("common-main.json");if main.exists(){directory(&data.join("assistant"));fs::rename(main,data.join("assistant/conversations.json")).unwrap();}
 let authority=Authority::existing(&config.join("pet-runtime-consent-v1")).unwrap();let store=crate::pet_runtime::store::Store::existing_verified(&config.join("pet-runtime-v1"),&config.join("pet-runtime-consent-v1")).unwrap().unwrap();
 let read=store.read_control(&closed.id,&closed.spec.owner.pet_id).unwrap();assert_eq!(read.revision,closed.revision);let(profile_dir,profile)=authority.owner_context().unwrap();assert_eq!(profile,serde_json::from_value::<crate::pet_runtime::runtime_owner::Profile>(grant["runtimeOwner"]["profile"].clone()).unwrap());assert!(profile_dir.is_dir());
 let source=Arc::new(store).prepare_peer_planning_continuation(&config.join("pet-runtime-consent-v1"),&read.id,&read.spec.owner.pet_id,&read.revision,crate::assistant_memory::now()).unwrap();assert_eq!(source.snapshot().pending_steps.len(),1);assert_eq!(source.snapshot().charge.unknown_model_results,0);drop(source);
 let mut files=std::collections::BTreeMap::new();inventory(&output,&config,&mut files);inventory(&output,&data,&mut files);
 file(&output.join("FIXTURE.json"),&serde_json::to_vec_pretty(&json!({"goalId":closed.id,"petId":closed.spec.owner.pet_id,"revision":closed.revision,"configPayload":config,"dataPayload":data,"endpoint":settings.ollama_url,"model":"fixture-free","policyRevision":expected,"grantBefore":grant,"goalBefore":closed,"originalProcessKilledAndReaped":true,"originalClosureReviewed":true,"nativePolicyComputedFromConfiguration":true,"authorityInodesPreserved":true,"moved":moved,"files":files,"installedProfileCopied":false,"realProvidersOrQuota":false,"metadata":metadata(),"testProducerPid":std::process::id()})).unwrap());
}

#[test]
#[ignore="explicit native product compatibility fixture export; no external provider"]
fn h3d2_signed_product_compatibility_export(){
 let output=PathBuf::from(std::env::var("COUCOU_H3D2_COMPAT_EXPORT").expect("explicit private output directory"));
 assert!(output.is_absolute()&&output.is_dir()&&fs::read_dir(&output).unwrap().next().is_none());
 let mut exported=vec![];
 for provider in["ollama","lmstudio"]{for mode in["partial","uncertain","bound-inference"]{
  let endpoint=crate::local_chat::Server{id:provider,name:"Uninvoked explicit compatibility destination",url:"http://127.0.0.1:19009".into(),key:None};
  let mut env=Process::with_endpoint(provider,mode,Some(&endpoint));env.crash();let closed=env.close();
  assert_eq!(closed.state,State::RecoveryRequired);assert_eq!(env.observation(),OwnerState::Closed);
  let grant=body(&env.s,&closed);assert_eq!(grant["phase"],"closed");
  let profile=output.join(format!("{provider}-{mode}"));let config=profile.join("config/coucou");let data=profile.join("data/coucou");directory(&config);directory(&data);
  file(&config.join("settings.json"),br#"{"language":"pt","activeIntegrations":[],"autostart":false,"hooksInstalled":false,"soundEnabled":false,"petSystemCenter":false,"shortcuts":{}}"#);
  file(&config.join("pets.json"),br#"{"version":1,"pets":{}}"#);
  let layout=crate::pet_data::Layout::new(config,data).unwrap();
  let(config,data)=crate::pet_data::fixture_publish_profile(&env.bus.address,PREFIX,layout).unwrap();
  let mapping=[("authority","pet-runtime-consent-v1"),("authority.owners-v1","pet-runtime-consent-v1.owners-v1"),("goals","pet-runtime-v1"),("contexts","pet-runtime-contexts-v1"),("pet-runtime-planning-v1","pet-runtime-planning-v1"),("pet-runtime-results-v1","pet-runtime-results-v1"),("pet-runtime-step-consent-v1","pet-runtime-step-consent-v1")];
  for(source,target)in mapping{let source=env.s.f.path.join(source);if source.exists(){copy(&source,&config.join(target));}}
  let main=env.s.f.path.join("common-main.json");if main.exists(){copy(&main,&data.join("assistant/conversations.json"));}
  // Do not silently omit a new persisted runtime component from this proof.
  for entry in fs::read_dir(&env.s.f.path).unwrap(){let name=entry.unwrap().file_name();let name=name.to_str().unwrap();assert!(mapping.iter().any(|(source,_)|name==*source)||matches!(name,"common-main.json"|".pets-profile-v1.lock"),"unmapped generated runtime component: {name}");}
  let store=current_store(&env.s);let list=store.list(None).unwrap();assert!(list.complete);
  let goals:Vec<_>=list.goals.iter().map(|g|{let r=store.read(&g.id,&g.pet_id).unwrap();json!({"goalId":r.id,"petId":r.spec.owner.pet_id,"revision":r.revision,"schemaVersion":r.schema_version,"state":r.state,"decisions":r.admissions.iter().map(|a|a.spec.decision_id.clone()).collect::<Vec<_>>()})}).collect();
  let mut files=std::collections::BTreeMap::new();inventory(&profile,&config,&mut files);inventory(&profile,&data,&mut files);
  let record=json!({"profile":profile,"provider":provider,"mode":mode,"parentGoalId":closed.id,"parentPetId":closed.spec.owner.pet_id,"parentDecisionId":closed.admissions[0].spec.decision_id,"parentGrantSchema":grant["schemaVersion"],"parentBudgetUsage":grant["budgetUsage"],"goals":goals,"files":files,"executionAuthorityGrantedByExport":false,"nativePublicationUsed":true,"originalProcessKilledAndReaped":true,"originalClosureReviewed":true,"providerSynthetic":true});
  file(&profile.join("FIXTURE.json"),&serde_json::to_vec_pretty(&record).unwrap());exported.push(record);
 }}
 file(&output.join("EXPORT.json"),&serde_json::to_vec_pretty(&json!({"schemaVersion":1,"generatedProfiles":exported,"installedProfileCopied":false,"realProvidersOrQuota":false})).unwrap());
 assert_eq!(exported.len(),6);
}

#[test]
#[ignore="explicit verifier of native product restored synthetic H3D2 profiles"]
fn h3d2_signed_product_restoration_is_historical_and_non_executable(){
 let manifest=PathBuf::from(std::env::var("COUCOU_H3D2_COMPAT_RESTORED").expect("explicit product restoration receipt manifest"));
 let values:Value=serde_json::from_slice(&fs::read(manifest).unwrap()).unwrap();let profiles=values["restoredProfiles"].as_array().unwrap();
 // Accept the original six protocol scenarios or a bounded, individually
 // witnessed native profile with the newer owner-resumption formats.
 assert!(!profiles.is_empty()&&profiles.len()<=6);
 for p in profiles{
  let config=PathBuf::from(p["configPayload"].as_str().unwrap());let authority=config.join("pet-runtime-consent-v1");let goals=config.join("pet-runtime-v1");
  let mut before=std::collections::BTreeMap::new();inventory(&config,&config,&mut before);let a=Authority::existing(&authority).unwrap();
  let store=crate::pet_runtime::store::Store::existing(&goals).unwrap().unwrap();let list=store.list(None).unwrap();assert!(list.complete);
  assert_eq!(list.goals.len(),p["goals"].as_array().unwrap().len());
  for reference in p["goals"].as_array().unwrap(){
   let g=store.read(reference["goalId"].as_str().unwrap(),reference["petId"].as_str().unwrap()).unwrap();assert_eq!(g.revision,reference["revision"].as_str().unwrap());
   let h=a.history(&g,crate::assistant_memory::now()).unwrap();assert_eq!(h.schema_version,2);assert!(!h.execution_available&&!h.continuation_available&&!h.accounting_complete);assert!(h.cumulative_charge.is_none());
   assert!(h.restored_source.as_ref().is_some_and(|s|s.post_snapshot_state_unknown&&!s.owner_absence_proven&&!s.execution_authority_granted));
   if !g.admissions.is_empty(){
    let expected=if g.state==State::Queued{crate::pet_runtime::authority::Failure::RecoveryRequired}else{crate::pet_runtime::authority::Failure::NotQueued};
    assert_eq!(a.verify_queued(&g,crate::assistant_memory::now()).err(),Some(expected));
    for row in &g.admissions{
     let proof=row.proof.as_ref().unwrap();
     assert_eq!(a.owner_observation(&row.spec,proof).err(),Some(crate::pet_runtime::authority::Failure::RecoveryRequired));
     assert_eq!(a.request_stop(&row.spec,proof,crate::pet_runtime::leases::StopReason::Pause,crate::assistant_memory::now()).err(),Some(crate::pet_runtime::authority::Failure::RecoveryRequired));
    }
   }
  }
  let mut historical_messages=0;
  for pet in["ai_ollama","ai_lmstudio"]{
   let live=a.message_inbox(pet,None,32,crate::assistant_memory::now()).unwrap();assert!(live.items.is_empty()&&!live.execution_authorized);
   let inbox=a.message_inbox_view(pet,None,32,crate::assistant_memory::now(),true).unwrap();assert!(inbox.scan_complete&&!inbox.execution_authorized&&inbox.unavailable==0);
   historical_messages+=inbox.items.len();for item in &inbox.items{assert!(!item.available);assert!(item.historical_source.as_ref().is_some_and(|s|s.snapshot_only&&s.post_snapshot_state_unknown&&!s.execution_authorized));}
  }
  assert!(historical_messages>0);let mut after=std::collections::BTreeMap::new();inventory(&config,&config,&mut after);assert_eq!(after,before);
  println!("restored {} goals, {} historical messages; old owner/Stop authority refused without writing",list.goals.len(),historical_messages);
 }
}
