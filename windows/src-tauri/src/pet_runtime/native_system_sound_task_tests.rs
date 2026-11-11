//! Original task Authority/intent/budget and native PipeWire/Pulse virtual nodes.
//! No real provider/model, user devices, Tauri grant UI or installed acceptance.
use super::*;
use super::plan_writer_tests::{setup_plan_custom,setup_plan_criteria};
use crate::{pet_runtime::{physical_target::{self as physical,Target},plans,steps::{self,Phase},criteria,recipes},pet_audio::{pipewire_reader::{Reader,NativeFixture},native_task::RetainedOutput,task::Action as SoundAction},pet_config::system_access::{Policy,Grant,Resource,Subplugin,Operation}};
use std::{fs,path::Path,sync::{Mutex,OnceLock},time::{Duration,Instant}};
pub(in crate::pet_runtime)fn serial()->std::sync::MutexGuard<'static,()>{static LOCK:OnceLock<Mutex<()>>=OnceLock::new();LOCK.get_or_init(||Mutex::new(())).lock().unwrap_or_else(|p|p.into_inner())}
pub(in crate::pet_runtime)struct Audio{runtime:NativeFixture,key:String}
impl Audio{
 pub(in crate::pet_runtime)fn new()->Self{let mut runtime=NativeFixture::new();runtime.enable_pulse();let reader=runtime.open(false,Arc::new(||true));let inventory=RetainedOutput::inventory_fixture(&reader).unwrap();assert_eq!(inventory.outputs.len(),1);assert!(inventory.outputs[0].native_id.is_some()&&inventory.outputs[0].native_serial.is_some());let public=serde_json::to_value(&inventory.outputs[0]).unwrap();assert!(public.get("nativeId").is_none()&&public.get("nativeSerial").is_none());let key=inventory.outputs[0].key.clone();Self{runtime,key}}
 pub(in crate::pet_runtime)fn directory(&self)->&Path{self.runtime.directory()}
 pub(in crate::pet_runtime)fn config(&self)->crate::pet_config::Configuration{let mut c=crate::pet_config::defaults();c.version=2;
  for(pet,ch)in[("ai_ollama",'a'),("integration_spotify",'b')]{let p=c.pets.get_mut(pet).unwrap();p.permissions=Some(crate::pet_config::Permissions::default());p.system_access=Some(Policy{schema_version:1,enabled:true,revision:ch.to_string().repeat(32),grants:[Operation::Observe,Operation::Volume,Operation::Mute].into_iter().map(|operation|Grant{subplugin:Subplugin::Sound,operation,resource:Resource::AudioOutput{key:self.key.clone()}}).collect()});}c
 }
 pub(in crate::pet_runtime)fn proposed(&self,mute:bool)->StepInput{StepInput{description:"Exact original native output".into(),postcondition:"Original output state independently observed".into(),operation:Some(plans::OperationProposal{operation_id:if mute{"system.sound.mute"}else{"system.sound.volume"}.into(),version:1,arguments:if mute{json!({"targetKey":self.key,"muted":true})}else{json!({"targetKey":self.key,"percent":35})}})}}
 pub(in crate::pet_runtime)fn target(&self,file:&Path,pet:&str,mute:bool)->Target{let op=self.proposed(mute).operation.unwrap();let action=SoundAction::parse(&op.operation_id,1,&op.arguments).unwrap();Target::sound_native_fixture(file,&action,pet,self.runtime.directory()).unwrap()}
 fn ready(&self,f:&Fixture,a:&Authority,g:&Record,o:&Owned,file:&Path)->crate::pet_runtime::lease_controller::step_controller::Ready{
  let p=&g.plans[0];let review=o.review_step(g,&p.id,&p.steps[0].id,self.target(file,"ai_ollama",false),&self.config(),crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(o,g,&self.config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];a.issue_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();assert!(f.path.exists());ready
 }
 pub(in crate::pet_runtime)fn assert_volume(&self,value:f64){assert!((self.runtime.observed().volume_percent.unwrap()-value).abs()<0.6);}
}
fn save(file:&Path,c:&crate::pet_config::Configuration){fs::write(file,serde_json::to_vec(c).unwrap()).unwrap();}
#[test]#[ignore="Private native PipeWire/Pulse fixture and signed common task Authority; no user audio"]
fn native_system_task_step_uses_existing_exact_grant_ledger_and_replay_never_writes_again(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![audio.proposed(false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&audio.config());let before=fs::read(&file).unwrap();let ready=audio.ready(&f,&a,&g,&o,&file);
 assert!(!audio.config().permissions("ai_ollama").unwrap().music);assert_eq!(ready.scope.schema_version,3);assert_eq!(ready.scope.resource.sound().unwrap().output_key,audio.key);
 let result=o.run_step(&g,&ready,&audio.config()).unwrap();assert_eq!(result.phase,Phase::Finished);let obs=&result.result.as_ref().unwrap().observation;assert!(obs.effect_verified&&obs.scope_current);assert_eq!(obs.delivery,physical::Delivery::Accepted);audio.assert_volume(35.0);
 let budget=ledger(&f,&g);assert_eq!(budget["attempts"].as_array().unwrap().len(),1);assert_eq!(budget["attempts"][0]["kind"],"tool");assert_eq!(budget["attempts"][0]["phase"],"settled");
 audio.runtime.external_volume(11);o.run_step(&g,&ready,&audio.config()).unwrap();audio.assert_volume(11.0);assert_eq!(ledger(&f,&g),budget);assert_eq!(fs::read(&file).unwrap(),before);assert_eq!(o.workers(),0);
 println!("NATIVE_TASK_STEP signedIntentBudget=true unchangedExactGrant=true toolAttempts=1 modelCalls=0 replayPreservedExternalVolume=11 userAudioWrites=0");
}
#[test]#[ignore="Private native PipeWire/Pulse fixture and signed common recipe Authority; no user audio"]
fn native_system_task_recipe_completes_only_with_the_independent_final_criteria(){
 let _serial=serial();for changed in [false,true]{let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_criteria(vec![audio.proposed(false),audio.proposed(true)],2,Arc::new(|_,_,_|Validity::Current),vec!["Native output volume35".into(),"Native output muted".into()]);let file=f.path.join("pets.json");save(&file,&audio.config());
 let input=[criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemSoundVolume{percent:35}},criteria::Input{criterion_index:1,predicate:criteria::Predicate::SystemSoundMute{muted:true}}];
 let review=o.review_recipe_with_criteria(&g,&g.plans[0].id,"native-exact-sound",audio.target(&file,"ai_ollama",false),&audio.config(),&input,crate::assistant_memory::now()).unwrap();let(approved,ready)=review.approve(&o,&g,&audio.config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];let scope=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();
 let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(audio.config()),|progress|{if changed&&progress.verified_steps==2{audio.runtime.external_volume(5);}}).unwrap();
 assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);assert!(ledger(&f,&g)["attempts"].as_array().unwrap().iter().all(|a|a["kind"]=="tool"));
 if changed{assert_eq!(result.outcome,recipes::Outcome::CriteriaUnmet);assert!(!result.progress.goal_complete);audio.assert_volume(5.0);}else{assert_eq!(result.outcome,recipes::Outcome::GoalCompleted);assert!(result.progress.criteria_verified&&result.progress.goal_complete);a.verify_completed_goal(&f.store().read(&g.id,&g.spec.owner.pet_id).unwrap()).unwrap();audio.assert_volume(35.0);assert_eq!(audio.runtime.observed().muted,Some(true));}
 }println!("NATIVE_TASK_RECIPE twoOriginalToolAttempts=true independentFinalCriteria=true signedCompletion=true externalChangeRefusedCompletion=true userAudioWrites=0");
}
#[test]#[ignore="Private native PipeWire/Pulse fixture and exact per-pet Sound grants; no user audio"]
fn native_system_task_shared_output_keeps_pet_grants_independent_and_observed_revocation_sticky(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![audio.proposed(false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&audio.config());let other=audio.target(&file,"integration_spotify",false);let ready=audio.ready(&f,&a,&g,&o,&file);
 let mut c=audio.config();c.pets.get_mut("integration_spotify").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);save(&file,&c);assert!(other.read().is_err());let result=o.run_step(&g,&ready,&c).unwrap();assert!(result.result.unwrap().observation.effect_verified);audio.assert_volume(35.0);
 let retained=audio.target(&file,"ai_ollama",false);c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().revision="e".repeat(32);save(&file,&c);assert!(retained.read().is_err());save(&file,&audio.config());assert!(retained.read().is_err());
 let mut denied=audio.config();denied.pets.get_mut("ai_ollama").unwrap().system_access=None;save(&file,&denied);let op=audio.proposed(false).operation.unwrap();let action=SoundAction::parse(&op.operation_id,1,&op.arguments).unwrap();assert!(Target::sound_native_fixture(&file,&action,"ai_ollama",audio.runtime.directory()).is_err());
 println!("NATIVE_TASK_GRANTS exactOutputShared=true otherPetRevocationIndependent=true observedOwnRevocationSticky=true noGrantBorrow=true userAudioWrites=0");
}
#[test]#[ignore="Private native PipeWire/Pulse fixture and original task target; no user audio"]
fn native_system_task_replacement_and_core_restart_never_rebind_the_authorized_target(){
 let _serial=serial();for restart in [false,true]{let mut audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![audio.proposed(false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&audio.config());let ready=audio.ready(&f,&a,&g,&o,&file);let original=audio.runtime.observed();
 if restart{audio.runtime.restart();}else{audio.runtime.cli(&["destroy".into(),original.id.to_string()]);audio.runtime.cli(&["create-node".into(),"adapter".into(),"{ factory.name=support.null-audio-sink node.name=coucou-fixture-output node.description=PrivateOutput media.class=Audio/Sink audio.position=[ FL FR ] object.linger=true }".into()]);}
 let after=audio.runtime.observed();let result=o.run_step(&g,&ready,&audio.config()).unwrap();let obs=result.result.unwrap().observation;assert_eq!(obs.delivery,physical::Delivery::NotDispatched);assert!(!obs.effect_verified&&!obs.scope_current);assert!(ledger(&f,&g).is_null());o.run_step(&g,&ready,&audio.config()).unwrap();assert!(ledger(&f,&g).is_null());let current=audio.runtime.observed();assert_eq!(current.id,after.id);assert_eq!(current.serial,after.serial);assert_eq!(current.volume_percent,after.volume_percent);assert_eq!(current.muted,after.muted);
 }println!("NATIVE_TASK_TARGET replacementNoDispatch=true restartedCoreNoDispatch=true noBudgetBeforeEffect=true noReplay=true userAudioWrites=0");
}
fn own_pw_cli_live()->bool{fs::read_dir("/proc").unwrap().take(131072).filter_map(|entry|entry.ok()).any(|e|{let Ok(s)=fs::read_to_string(e.path().join("stat"))else{return false};let Some(at)=s.rfind(')')else{return false};let name=s.find('(').map(|start|&s[start+1..at]);let fields:Vec<_>=s[at+1..].split_whitespace().take(2).collect();name==Some("pw-cli")&&fields.get(1).and_then(|s|s.parse::<u32>().ok())==Some(std::process::id())})}
#[test]#[ignore="Private stopped native core exercises cancellation during actual IO; no user audio"]
fn native_system_task_revocation_during_actual_core_io_stops_without_effect_or_new_budget(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![audio.proposed(false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&audio.config());let ready=audio.ready(&f,&a,&g,&o,&file);let original=audio.runtime.observed();audio.runtime.pause_core();
 let o=Arc::new(o);let owner=o.clone();let goal=g.clone();let next=ready.clone();let config=audio.config();let run=std::thread::spawn(move||owner.run_step(&goal,&next,&config));let end=Instant::now()+Duration::from_secs(3);while !own_pw_cli_live(){assert!(Instant::now()<end,"Expected original native core query to be live");std::thread::sleep(Duration::from_millis(5));}
 let mut revoked=audio.config();revoked.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);save(&file,&revoked);let result=run.join().unwrap().unwrap();audio.runtime.resume_core();let obs=result.result.unwrap().observation;assert!(!obs.effect_verified&&!obs.scope_current);assert_eq!(obs.delivery,physical::Delivery::NotDispatched);assert!(ledger(&f,&g).is_null());assert_eq!(audio.runtime.observed().volume_percent,original.volume_percent);assert_eq!(o.workers(),0);assert!(!own_pw_cli_live());
 println!("NATIVE_TASK_IO originalCoreQueryObservedLive=true revokedDuringIO=true workerReaped=true noEffectOrBudget=true userAudioWrites=0");
}
