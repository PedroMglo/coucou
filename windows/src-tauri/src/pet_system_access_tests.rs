use super::*;

#[test]fn guardian_health_grants_are_disjoint_and_revocation_cannot_revive_a_capture(){
 let f=Fixture::new();f.set("ai_ollama",grants(Resource::AudioOutputs{}));
 assert!(Captured::fixture_health(&f.file,"ai_ollama").is_err());
 f.set("agent_system_guardian",vec![Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::HostHealth{}}]);
 let scope=Captured::fixture_health(&f.file,"agent_system_guardian").unwrap();assert!(scope.check().is_ok());assert!(!scope.permits_output(OUTPUT)&&!scope.has_operation(Operation::Observe));
 assert!(Captured::fixture(&f.file,"agent_system_guardian",Operation::Observe,None).is_err());assert!(Captured::fixture_health(&f.file,"integration_spotify").is_err());
 let sound=f.capture("ai_ollama");let old=f.doc().config.pets["agent_system_guardian"].system_access.clone().unwrap();f.set("agent_system_guardian",vec![]);assert!(scope.check().is_err());assert!(sound.check().is_ok());f.set("agent_system_guardian",old.grants);assert!(scope.check().is_err());assert!(Captured::fixture_health(&f.file,"agent_system_guardian").unwrap().check().is_ok());assert!(sound.check().is_ok());
}
#[test]fn guardian_health_grant_cannot_change_subplugin_resource_or_operation(){
 let f=Fixture::new();let before=std::fs::read(&f.file).unwrap();let d=f.doc();
 for g in [Grant{subplugin:Subplugin::Sound,operation:Operation::Observe,resource:Resource::HostHealth{}},Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::AudioOutputs{}},Grant{subplugin:Subplugin::Health,operation:Operation::Mute,resource:Resource::HostHealth{}}]{
  assert!(set_at(&f.file,&Change{pet_id:"agent_system_guardian".into(),expected_revision:d.revision.clone(),grants:vec![g]},||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);
 }
}
const OUTPUT:&str="audio-0123456789abcdef";
const OTHER:&str="audio-fedcba9876543210";
struct Fixture{root:PathBuf,file:PathBuf}
impl Fixture{
 fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-system-access-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));use std::os::unix::fs::DirBuilderExt;std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();let file=root.join("pets.json");write(&file,&defaults()).unwrap();Self{root,file}}
 fn doc(&self)->Document{read_at_policy(&self.file,false).unwrap()}
 fn set(&self,pet:&str,grants:Vec<Grant>)->Document{let d=self.doc();set_at(&self.file,&Change{pet_id:pet.into(),expected_revision:d.revision,grants},||true).unwrap()}
 fn capture(&self,pet:&str)->Captured{Captured::fixture(&self.file,pet,Operation::Volume,Some(OUTPUT)).unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){std::fs::remove_dir_all(&self.root).unwrap();}}
fn grants(resource:Resource)->Vec<Grant>{[Operation::Observe,Operation::Volume,Operation::Mute].into_iter().map(|operation|Grant{subplugin:Subplugin::Sound,operation,resource:resource.clone()}).collect()}

#[test]fn system_access_independent_pets_revoke_and_reapply_without_reviving_an_old_scope(){
 let f=Fixture::new();let default=serde_json::to_value(f.doc().config).unwrap();
 assert!(Captured::fixture(&f.file,"ai_ollama",Operation::Observe,None).is_err());
 f.set("ai_ollama",grants(Resource::AudioOutputs{}));f.set("integration_spotify",grants(Resource::AudioOutputs{}));let ollama=f.capture("ai_ollama");let spotify=f.capture("integration_spotify");
 assert!(ollama.check().is_ok()&&spotify.check().is_ok());let previous=f.doc().config.pets["integration_spotify"].system_access.clone().unwrap();
 f.set("integration_spotify",vec![]);assert!(spotify.check().is_err());assert!(ollama.check().is_ok());f.set("integration_spotify",previous.grants.clone());assert!(spotify.check().is_err());assert!(ollama.check().is_ok());
 assert_ne!(f.doc().config.pets["integration_spotify"].system_access.as_ref().unwrap().revision,previous.revision);assert!(f.capture("integration_spotify").check().is_ok());
 for(id,pet)in &f.doc().config.pets{let mut now=serde_json::to_value(pet).unwrap();now.as_object_mut().unwrap().remove("systemAccess");assert_eq!(now,default["pets"][id]);}
}
#[test]fn system_access_exact_output_and_operations_do_not_expand_to_other_resources(){
 let f=Fixture::new();f.set("ai_ollama",grants(Resource::AudioOutput{key:OUTPUT.into()}));let c=f.capture("ai_ollama");assert!(c.permits_output(OUTPUT));assert!(!c.permits_output(OTHER));assert!(Captured::fixture(&f.file,"ai_ollama",Operation::Volume,Some(OTHER)).is_err());
 let d=f.doc();let before=std::fs::read(&f.file).unwrap();for bad in [vec![Grant{subplugin:Subplugin::Sound,operation:Operation::Volume,resource:Resource::AudioOutputs{}}],grants(Resource::AudioOutput{key:"name-reused".into()}),vec![grants(Resource::AudioOutputs{})[0].clone();2]]{
  assert!(set_at(&f.file,&Change{pet_id:"ai_ollama".into(),expected_revision:d.revision.clone(),grants:bad},||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);
 }
 for forged in [serde_json::json!({"subplugin":"network","operation":"observe","resource":{"kind":"audioOutputs"}}),serde_json::json!({"subplugin":"sound","operation":"shell","resource":{"kind":"audioOutputs"}}),serde_json::json!({"subplugin":"sound","operation":"observe","resource":{"kind":"audioInputs"}})]{assert!(serde_json::from_value::<Grant>(forged).is_err());}
}
#[test]fn system_access_legacy_bytes_and_whole_document_saves_cannot_create_or_overwrite_grants(){
 let f=Fixture::new();let old=f.doc();let before=serde_json::to_vec(&old.config).unwrap();assert_eq!(serde_json::to_vec(&resolve_existing_bytes(&before).unwrap()).unwrap(),before);
 let granted=f.set("ai_ollama",grants(Resource::AudioOutputs{}));let saved=save_at(&f.file,&old.config,&granted.revision).unwrap();assert_eq!(saved.config.pets["ai_ollama"].system_access,granted.config.pets["ai_ollama"].system_access);
 let mut forged=saved.config.clone();forged.pets.get_mut("integration_spotify").unwrap().system_access=granted.config.pets["ai_ollama"].system_access.clone();let saved=save_at(&f.file,&forged,&saved.revision).unwrap();assert!(saved.config.pets["integration_spotify"].system_access.is_none());
}
#[test]fn system_access_native_stale_choice_and_withdrawal_before_publication_preserve_bytes(){
 let f=Fixture::new();let before=std::fs::read(&f.file).unwrap();let d=f.doc();let change=Change{pet_id:"ai_ollama".into(),expected_revision:d.revision,grants:grants(Resource::AudioOutputs{})};
 assert!(set_at(&f.file,&change,||false).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);
 let frame=crate::settings::review::Frame::fixture();assert!(set_at(&f.file,&change,||{if std::fs::read_dir(&f.root).unwrap().any(|p|p.unwrap().path().extension().is_some_and(|s|s=="tmp")){frame.invalidate_for_test();}frame.current()}).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);
 f.set("integration_spotify",grants(Resource::AudioOutputs{}));let after=std::fs::read(&f.file).unwrap();assert!(set_at(&f.file,&change,||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),after);
 for label in ["pet-chat","phone","unknown"]{assert!(!allowed(label));}
}
#[test]fn system_access_observed_revocation_is_sticky_even_after_private_bytes_restore(){
 let f=Fixture::new();f.set("ai_ollama",grants(Resource::AudioOutputs{}));let before=std::fs::read(&f.file).unwrap();let c=f.capture("ai_ollama");f.set("ai_ollama",vec![]);assert!(c.check().is_err());std::fs::write(&f.file,before).unwrap();assert!(c.check().is_err());assert!(f.capture("ai_ollama").check().is_ok());
}

#[test]fn system_access_observers_and_retired_pets_cannot_be_promoted_by_a_grant(){let f=Fixture::new();let d=f.doc();let before=std::fs::read(&f.file).unwrap();for pet in ["agent_cursor","agent_gemini","integration_github"]{assert!(set_at(&f.file,&Change{pet_id:pet.into(),expected_revision:d.revision.clone(),grants:grants(Resource::AudioOutputs{})},||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);assert!(Captured::fixture(&f.file,pet,Operation::Observe,None).is_err());}}

#[test]fn system_access_suspended_restore_is_data_until_new_native_revision(){let f=Fixture::new();f.set("ai_ollama",grants(Resource::AudioOutputs{}));let old=f.doc().config.pets["ai_ollama"].system_access.clone().unwrap();let mut restored=f.doc().config;restored.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().enabled=false;write(&f.file,&restored).unwrap();assert!(Captured::fixture(&f.file,"ai_ollama",Operation::Observe,None).is_err());let revised=f.set("ai_ollama",old.grants);assert!(revised.config.pets["ai_ollama"].system_access.as_ref().unwrap().enabled);assert_ne!(revised.config.pets["ai_ollama"].system_access.as_ref().unwrap().revision,old.revision);assert!(f.capture("ai_ollama").check().is_ok());}

#[test]fn host_inventory_grants_do_not_borrow_health_sound_or_another_pet_and_revoke_stickily(){
 let f=Fixture::new();let health=Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::HostHealth{}};let inventory=Grant{subplugin:Subplugin::Inventory,operation:Operation::Observe,resource:Resource::HostComponents{}};
 f.set("ai_ollama",vec![health]);assert!(Captured::fixture_inventory(&f.file,"ai_ollama").is_err());
 f.set("agent_system_guardian",vec![inventory.clone()]);let g=Captured::fixture_inventory(&f.file,"agent_system_guardian").unwrap();assert!(g.check().is_ok());assert!(Captured::fixture_health(&f.file,"agent_system_guardian").is_err());assert!(Captured::fixture(&f.file,"agent_system_guardian",Operation::Observe,None).is_err());
 f.set("integration_spotify",grants(Resource::AudioOutputs{}));assert!(Captured::fixture_inventory(&f.file,"integration_spotify").is_err());
 f.set("ai_ollama",vec![inventory.clone()]);let own=Captured::fixture_inventory(&f.file,"ai_ollama").unwrap();f.set("agent_system_guardian",vec![]);assert!(g.check().is_err()&&own.check().is_ok());f.set("agent_system_guardian",vec![inventory]);assert!(g.check().is_err()&&own.check().is_ok());assert!(Captured::fixture_inventory(&f.file,"agent_system_guardian").unwrap().check().is_ok());
}
#[test]fn host_inventory_scope_rejects_wrong_operations_resources_and_suspended_grants(){
 let f=Fixture::new();let d=f.doc();let before=std::fs::read(&f.file).unwrap();
 for grant in [Grant{subplugin:Subplugin::Inventory,operation:Operation::Volume,resource:Resource::HostComponents{}},Grant{subplugin:Subplugin::Health,operation:Operation::Observe,resource:Resource::HostComponents{}},Grant{subplugin:Subplugin::Inventory,operation:Operation::Observe,resource:Resource::HostHealth{}}]{assert!(set_at(&f.file,&Change{pet_id:"ai_ollama".into(),expected_revision:d.revision.clone(),grants:vec![grant]},||true).is_err());assert_eq!(std::fs::read(&f.file).unwrap(),before);}
 f.set("ai_ollama",vec![Grant{subplugin:Subplugin::Inventory,operation:Operation::Observe,resource:Resource::HostComponents{}}]);let mut c=f.doc().config;c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().enabled=false;write(&f.file,&c).unwrap();assert!(Captured::fixture_inventory(&f.file,"ai_ollama").is_err());
}
