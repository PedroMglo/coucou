//! Common signed task steps against an owned synthetic native helper process.
//! No desktop/pactl/provider/account or human acceptance is exercised here.
use super::*;
use crate::pet_runtime::{physical_target::{self as physical,Target},steps::{self,Phase},criteria,recipes,plans};
use super::plan_writer_tests::{setup_plan_custom,setup_plan_criteria};
use crate::{pet_audio::task::{Io,Action as SoundAction},pet_config::system_access::{Policy,Grant,Resource,Subplugin,Operation}};
use std::{fs,path::{Path,PathBuf},sync::{Mutex,OnceLock},time::{Duration,Instant}};
fn serial()->std::sync::MutexGuard<'static,()>{static LOCK:OnceLock<Mutex<()>>=OnceLock::new();LOCK.get_or_init(||Mutex::new(())).lock().unwrap()}
struct Audio{root:PathBuf,io:Io,key:String}
impl Audio{
 fn new()->Self{
  let root=std::env::temp_dir().join(format!("coucou-sound-native-{}",nonce().unwrap()));{use std::os::unix::fs::DirBuilderExt;fs::DirBuilder::new().mode(0o700).create(&root).unwrap();}let script=root.join("protocol.py");let state=root.join("state.json");fs::write(&script,HELPER).unwrap();fs::write(&state,serde_json::to_vec(&json!({"cookie":"generated-server-a","volume":20,"muted":false,"calls":[]})).unwrap()).unwrap();let io=Io::fixture(script,state);
  let snapshot=crate::pet_audio::linux::task_read_outputs_with(&io,&||true).unwrap();assert_eq!(snapshot.outputs.len(),1);let key=snapshot.outputs[0].key.clone();Self{root,io,key}
 }
 fn state(&self)->Value{serde_json::from_slice(&fs::read(self.root.join("state.json")).unwrap()).unwrap()}
 fn update(&self,f:impl FnOnce(&mut Value)){let mut v=self.state();f(&mut v);fs::write(self.root.join("state.json"),serde_json::to_vec(&v).unwrap()).unwrap();}
 fn calls(&self)->usize{self.state()["calls"].as_array().unwrap().len()}
 fn bind_intent(&self,f:&Fixture,g:&Record,scope:&steps::Scope){let grant=crate::pet_runtime::authority_tests::root(f).join(format!("{}.json",g.admissions[0].spec.decision_id));let step=f.path.join("pet-runtime-step-consent-v1").join(format!("{}.json",scope.slot().unwrap()));self.update(|v|{v["grantFile"]=json!(grant);v["stepFile"]=json!(step);});}
}
impl Drop for Audio{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
fn config()->crate::pet_config::Configuration{
 let mut c=crate::pet_config::defaults();c.version=2;
 for(pet,ch)in[("ai_ollama",'a'),("integration_spotify",'b')]{let p=c.pets.get_mut(pet).unwrap();p.permissions=Some(crate::pet_config::Permissions::default());p.system_access=Some(Policy{schema_version:1,enabled:true,revision:ch.to_string().repeat(32),grants:[Operation::Observe,Operation::Volume,Operation::Mute].into_iter().map(|operation|Grant{subplugin:Subplugin::Sound,operation,resource:Resource::AudioOutputs{}}).collect()});}c
}
fn save(file:&Path,c:&crate::pet_config::Configuration){fs::write(file,serde_json::to_vec(c).unwrap()).unwrap();}
fn proposed(key:&str,mute:bool)->StepInput{StepInput{description:"Exact native output assignment".into(),postcondition:"Original output state independently observed".into(),operation:Some(plans::OperationProposal{operation_id:if mute{"system.sound.mute"}else{"system.sound.volume"}.into(),version:1,arguments:if mute{json!({"targetKey":key,"muted":true})}else{json!({"targetKey":key,"percent":35})}})}}
fn target(audio:&Audio,file:&Path,mute:bool)->Target{let op=proposed(&audio.key,mute).operation.unwrap();let a=SoundAction::parse(&op.operation_id,1,&op.arguments).unwrap();Target::sound_fixture(file,&a,"ai_ollama",audio.io.clone()).unwrap()}
fn ready(f:&Fixture,a:&Authority,g:&Record,o:&Owned,audio:&Audio,file:&Path)->crate::pet_runtime::lease_controller::step_controller::Ready{
 let p=&g.plans[0];let review=o.review_step(g,&p.id,&p.steps[0].id,target(audio,file,false),&config(),crate::assistant_memory::now()).unwrap();assert_eq!(review.proposal().scope.schema_version,3);assert_eq!(review.proposal().scope.resource.sound().unwrap().pet_id,"ai_ollama");
 let(approved,ready)=review.approve(o,g,&config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];a.issue_step(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();audio.bind_intent(f,g,&ready.scope);ready
}
#[test]fn system_sound_step_native_transport_requires_own_grant_and_model_catalog_stays_closed(){
 let op=proposed("audio-0123456789abcdef",false).operation.unwrap();assert!(op.valid());assert!(crate::pet_audio::task::native_dispatch_available());let mut denied=config();denied.pets.get_mut("ai_ollama").unwrap().system_access=None;assert!(matches!(Target::capture(&op,"ai_ollama",&denied,||true),Err(model::Error::Conflict)),"No own grant: refuse before native IO, even when transport exists");
 let missing=physical::Snapshot::Sound(physical::SoundSnapshot{kind:physical::SoundKind::SystemSoundOutput,output_key:"audio-0123456789abcdef".into(),volume_percent:None,muted:None,observed_at:1});assert_eq!(criteria::Predicate::SystemSoundVolume{percent:35}.target_status(&missing),criteria::Status::Unknown);assert!(crate::pet_operations::validate_arguments("system.sound.volume",&op.arguments).is_err(),"no model tool registered");
 for change in [json!({"targetKey":"audio-0123456789abcdef","percent":101}),json!({"targetKey":"audio-0123456789abcdef","percent":35,"petId":"agent_codex"}),json!({"targetKey":"replacement-name","percent":35})]{assert!(SoundAction::parse(&op.operation_id,1,&change).is_err());}
 let legacy=crate::spotify::goal::Binding{schema_version:1,bus_id:"generated-bus".into(),owner:":1.52".into(),uid:1000,pid:52,identity:"Generated player".into()};let bytes=serde_json::to_vec(&legacy).unwrap();assert_eq!(serde_json::to_vec(&physical::Binding::Spotify(legacy)).unwrap(),bytes);
 let binding=physical::Binding::Sound(physical::SoundBinding{schema_version:1,kind:physical::SoundKind::SystemSoundOutput,pet_id:"ai_ollama".into(),grant_revision:"a".repeat(32),output_key:"audio-0123456789abcdef".into()});let value=serde_json::to_value(&binding).unwrap();assert!(serde_json::from_value::<crate::spotify::goal::Binding>(value.clone()).is_err());let mut mixed=value.clone();mixed["owner"]=json!(":1.52");assert!(serde_json::from_value::<physical::Binding>(mixed).is_err());let mut future=value;future["kind"]=json!("unknownFutureTarget");assert!(serde_json::from_value::<physical::Binding>(future).is_err());
}
#[test]fn system_sound_step_uses_original_task_journal_before_the_native_effect_and_replay_is_read_only(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![proposed(&audio.key,false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&config());let r=ready(&f,&a,&g,&o,&audio,&file);
 assert!(!config().permissions("ai_ollama").unwrap().music);let stored=o.run_step(&g,&r,&config()).unwrap();let observation=&stored.result.as_ref().unwrap().observation;assert_eq!(stored.phase,Phase::Finished);assert!(observation.effect_verified&&observation.scope_current);assert_eq!(audio.calls(),1);assert_eq!(audio.state()["volume"],35);
 let before=ledger(&f,&g);assert_eq!(before["attempts"][0]["kind"],"tool");assert_eq!(before["attempts"][0]["phase"],"settled");assert_eq!(before["attempts"].as_array().unwrap().len(),1);assert!(!before["attempts"].as_array().unwrap().iter().any(|a|a["kind"]=="model"));
 o.run_step(&g,&r,&config()).unwrap();assert_eq!(audio.calls(),1);assert_eq!(ledger(&f,&g),before);
 let mut forged=r.scope.clone();if let physical::Binding::Sound(b)=&mut forged.resource{b.pet_id="integration_spotify".into();}assert!(!forged.valid());assert_eq!(o.workers(),0);
}
#[test]fn system_sound_step_own_policy_revoked_before_review_or_run_cannot_borrow_spotify(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![proposed(&audio.key,false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&config());let r=ready(&f,&a,&g,&o,&audio,&file);let mut c=config();c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);save(&file,&c);
 assert!(o.run_step(&g,&r,&config()).is_ok());assert_eq!(audio.calls(),0);assert!(ledger(&f,&g).is_null());let row=&g.admissions[0];let saved=a.read_step(&row.spec,row.proof.as_ref().unwrap(),&r.scope.plan_id,&r.scope.step_id).unwrap().unwrap();assert!(!saved.result.unwrap().observation.effect_verified);
}
#[test]fn system_sound_step_revalidates_physical_identity_after_the_durable_intent_wait(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![proposed(&audio.key,false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&config());let r=ready(&f,&a,&g,&o,&audio,&file);audio.update(|v|v["swapAfterIntent"]=json!(true));let result=o.run_step(&g,&r,&config()).unwrap();assert_eq!(audio.calls(),0);assert!(!result.result.as_ref().unwrap().observation.effect_verified);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);o.run_step(&g,&r,&config()).unwrap();assert_eq!(audio.calls(),0);
}
#[test]fn system_sound_step_revocation_during_native_io_retains_uncertainty_and_never_repeats_the_effect(){
 let _serial=serial();let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_custom(vec![proposed(&audio.key,false)],1,Arc::new(|_,_,_|Validity::Current));let file=f.path.join("pets.json");save(&file,&config());let r=ready(&f,&a,&g,&o,&audio,&file);audio.update(|v|v["blockAck"]=json!(true));let o=Arc::new(o);let original=o.clone();let g2=g.clone();let r2=r.clone();let thread=std::thread::spawn(move||original.run_step(&g2,&r2,&config()));let until=Instant::now()+Duration::from_secs(10);while !audio.root.join("effect-started").exists(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
 let mut c=config();c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);save(&file,&c);let result=thread.join().unwrap().unwrap();let obs=result.result.unwrap().observation;assert_eq!(obs.delivery,physical::Delivery::Uncertain);assert!(!obs.effect_verified&&!obs.scope_current);assert_eq!(audio.calls(),1);let before=ledger(&f,&g);o.run_step(&g,&r,&config()).unwrap();assert_eq!(audio.calls(),1);assert_eq!(ledger(&f,&g),before);assert_eq!(o.workers(),0);
}
#[test]fn system_sound_step_recipe_uses_the_same_budget_and_requires_independent_final_criteria(){
 let _serial=serial();
 for changed in [false,true]{let audio=Audio::new();let(f,a,g,_pending,o,_main)=setup_plan_criteria(vec![proposed(&audio.key,false),proposed(&audio.key,true)],2,Arc::new(|_,_,_|Validity::Current),vec!["Final output volume35".into(),"Final output muted".into()]);let file=f.path.join("pets.json");save(&file,&config());let input=[criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemSoundVolume{percent:35}},criteria::Input{criterion_index:1,predicate:criteria::Predicate::SystemSoundMute{muted:true}}];let review=o.review_recipe_with_criteria(&g,&g.plans[0].id,"own-sound-recipe",target(&audio,&file,false),&config(),&input,crate::assistant_memory::now()).unwrap();assert_eq!(review.proposal().scope.schema_version,4);let(approved,ready)=review.approve(&o,&g,&config(),crate::assistant_memory::now()).unwrap();let row=&g.admissions[0];let scope=a.issue_recipe(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now()).unwrap();ready.activate(&scope).unwrap();audio.bind_intent(&f,&g,&scope.steps[0]);
 // Each helper checks the native step matching its actual invocation.
 audio.update(|v|{v["stepFiles"]=json!([f.path.join("pet-runtime-step-consent-v1").join(format!("{}.json",scope.steps[0].slot().unwrap())),f.path.join("pet-runtime-step-consent-v1").join(format!("{}.json",scope.steps[1].slot().unwrap()))]);});
 let result=o.run_recipe_for_store(&f.store(),&g,&ready,||Ok(config()),|progress|{if changed&&progress.verified_steps==2{audio.update(|v|v["volume"]=json!(5));}}).unwrap();assert_eq!(audio.calls(),2);assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),2);
 if changed{assert_eq!(result.outcome,recipes::Outcome::CriteriaUnmet);assert!(!result.progress.goal_complete);}else{assert_eq!(result.outcome,recipes::Outcome::GoalCompleted);assert!(result.progress.criteria_verified&&result.progress.goal_complete);a.verify_completed_goal(&f.store().read(&g.id,&g.spec.owner.pet_id).unwrap()).unwrap();}
 }
}
const HELPER:&str=r#"import json,sys,pathlib,time
p=pathlib.Path(sys.argv[1]);a=sys.argv[2:];s=json.loads(p.read_text())
def save():
 t=p.with_suffix('.next');t.write_text(json.dumps(s));t.replace(p)
def started(path):
 try:return json.loads(pathlib.Path(path).read_text())['body']
 except (OSError,KeyError,ValueError):return {}
if a==['--format=json','info']:
 if s.get('swapAfterIntent') and any(x['phase']=='started' for x in started(s['grantFile']).get('budgetUsage',{}).get('attempts',[])):
  s['cookie']='generated-server-b';save()
 print(json.dumps({'cookie':s['cookie'],'default_sink_name':'generated-output'}))
elif a==['--format=json','list','sinks']:
 print(json.dumps([{'index':1,'name':'generated-output','description':'Synthetic native output','properties':{'object.serial':'17'},'volume':{'mono':{'value':s['volume']/100*65536}},'mute':s['muted'],'state':'RUNNING'}]))
elif a[0] in ['set-sink-volume','set-sink-mute']:
 g=started(s['grantFile']);assert any(x['phase']=='started' for x in g['budgetUsage']['attempts'])
 paths=s.get('stepFiles',[s['stepFile']]);assert any(started(path).get('phase')=='started' for path in paths)
 assert a[1]=='generated-output';s['calls'].append(a)
 if a[0]=='set-sink-volume':s['volume']=int(a[2][:-1])
 else:s['muted']=a[2]=='1'
 save()
 if s.get('blockAck'):
  p.parent.joinpath('effect-started').write_text('native helper reached exact effect after signed intent')
  time.sleep(10)
 print('{}')
else:raise SystemExit('Unexpected native operation')
"#;
