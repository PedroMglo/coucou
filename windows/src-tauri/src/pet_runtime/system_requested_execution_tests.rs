//! Same common Driver, native fixed host reads, generated identities/consent.
//! No UI, real model, user audio, host configuration or installed data changes.
use super::*;
use crate::pet_runtime::{model,admission::{ExecutionStart,StartScope},planning::Start,lease_controller::{Owned,dispatcher::{Capture,Driver,Environment,Phase,Status}},recipe_decisions::{Source,Decision},store::{Store,Fault}};
use std::sync::{Mutex,atomic::AtomicBool};
type Hook=(Phase,Box<dyn FnOnce()+Send>);
struct Env{path:std::path::PathBuf,target:Target,statuses:Mutex<Vec<Status>>,hook:Mutex<Option<Hook>>}
impl Environment for Env{
 fn capture(&self,_:&Owned)->Result<Capture>{panic!("System Driver must not resolve inference or model metadata")}
 fn configuration(&self)->Result<crate::pet_config::Configuration>{serde_json::from_slice(&fs::read(&self.path).map_err(|_|Error::StorageUnavailable)?).map_err(|_|Error::InvalidInput)}
 fn target(&self)->Result<Arc<crate::spotify::goal::Target>>{panic!("System Driver must not use a music target")}
 fn system_target(&self,_:&crate::pet_runtime::plans::OperationProposal,pet:&str,_:&crate::pet_config::Configuration,current:&dyn Fn()->bool)->Result<Target>{assert!(current());assert_eq!(self.target.binding().inventory().map(|b|b.pet_id.as_str()).or_else(||self.target.binding().health().map(|b|b.pet_id.as_str())).or_else(||self.target.binding().sound().map(|b|b.pet_id.as_str())),Some(pet));Ok(self.target.clone())}
 fn decide(&self,_:Source)->Option<Decision>{panic!("Initial exact System consent must not become another decision")}
 fn notify(&self,s:&Status){self.statuses.lock().unwrap().push(s.clone());let action={let mut h=self.hook.lock().unwrap();if h.as_ref().is_some_and(|(phase,_)|*phase==s.phase){h.take()}else{None}};if let Some((_,action))=action{action();}}
}
struct Case{f:Fixture,a:Authority,g:Record,owner:Arc<Owned>,chat:crate::chat::Chat,env:Arc<Env>,driver:Arc<Driver>}
fn clock()->u64{crate::assistant_memory::now()}
fn claimed(f:Fixture,c:crate::pet_config::Configuration,g:Record,r:Review,target:Target,fault:Fault)->Case{
 let a=Authority::ensure(&f.path.join("authority")).unwrap();let approved=r.approve(&g,evidence(&g,&c),clock()).unwrap();let cert=a.certify(&approved,&g,clock()).unwrap();let q=f.store().enqueue_certified(&cert,clock()).unwrap().record;
 let chat=crate::chat::Chat::persistent(f.path.join("main.json"));let config_path=f.path.join("system.json");let original=q.admissions[0].spec.native_system_plan.clone().unwrap();
 let pending=Controller::new().unwrap().prepare(&q,&f.path.join("authority"),&f.path.join("contexts"),&chat,&evidence(&q,&c),"requested-system-claim".into(),clock(),Arc::new(move|_,_,_|{
  match fs::read(&config_path).ok().and_then(|bytes|serde_json::from_slice(&bytes).ok()){
   Some(config)if original.targets_current(&config)=>Validity::Current,Some(_)=>Validity::Changed,None=>Validity::Unknown,
  }
 })).unwrap();let(receipt,owned)=pending.claim(&f.store(),&a,&evidence(&q,&c),clock()).unwrap();let owner=Arc::new(owned.unwrap());
 let env=Arc::new(Env{path:f.path.join("system.json"),target,statuses:Mutex::new(vec![]),hook:Mutex::new(None)});let mut store=Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap();store.fault=fault;
 let driver=Driver::new(owner.clone(),Arc::new(store),env.clone()).unwrap();Case{f,a,g:receipt.record,owner,chat,env,driver}
}
fn inventory(pet:&str,fault:Fault)->Case{
 let f=Fixture::new();let c=config(pet);save(&f,&c);let g=draft(&f,pet);let r=review(&f,&g,&c,criteria::Predicate::SystemHostInventoryObserved).unwrap().with_execution_start(Some(Start::WhenAvailable)).unwrap();let t=target(&f,pet);claimed(f,c,g,r,t,fault)
}
fn baseline()->Case{
 let f=Fixture::new();let mut c=config("agent_system_guardian");c.pets.get_mut("agent_system_guardian").unwrap().system_access.as_mut().unwrap().grants=serde_json::from_value(json!([{"subplugin":"health","operation":"observe","resource":{"kind":"hostHealth"}}])).unwrap();save(&f,&c);
 let g=draft_kind(&f,"agent_system_guardian","system.health.baseline",vec!["Medição original de CPU e memória.".into()]);let target=Target::health_fixture(&f.path.join("system.json"),"agent_system_guardian").unwrap();let path=f.path.join("system.json");let observe=Arc::new(move||serde_json::from_slice(&fs::read(&path).map_err(|_|Error::StorageUnavailable)?).map_err(|_|Error::InvalidInput));
 let r=Review::prepare_system(&g,evidence(&g,&c),Budget{deadline_at:clock()+60_000,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly},target.clone(),observe,&[criteria::Input{criterion_index:0,predicate:criteria::Predicate::SystemHealthBaselineMeasured}],"requested-health-admission".into(),nonce().unwrap(),clock()).unwrap().with_execution_start(Some(Start::WhenAvailable)).unwrap();
 claimed(f,c,g,r,target,Fault::None)
}
fn run(driver:&Driver)->Result<Status>{tokio::runtime::Builder::new_current_thread().enable_all().thread_stack_size(4*1024*1024).max_blocking_threads(1).build().unwrap().block_on(driver.run())}
fn body(c:&Case)->Value{serde_json::from_slice::<Value>(&fs::read(c.f.path.join("authority").join(format!("{}.json",c.g.admissions[0].spec.decision_id))).unwrap()).unwrap()["body"].clone()}

#[test]fn system_requested_execution_all_eight_owners_complete_one_original_inventory_without_inference(){
 for pet in ["agent_system_guardian","ai_ollama","agent_codex","agent_antigravity","agent_copilot","ai_lmstudio","ai_openai","integration_spotify"]{
  let c=inventory(pet,Fault::None);let original=c.g.admissions[0].spec.clone();assert!(original.native_system_requested());assert!(!crate::pet_runtime::readiness_api::requested_inference(&original));assert_eq!(c.env.target.collector_calls(),0);
  let status=run(&c.driver).unwrap();assert_eq!(status.phase,Phase::Completed);assert!(!status.active);assert_eq!(c.env.target.collector_calls(),1);let stored=c.f.store().read(&c.g.id,pet).unwrap();c.a.verify_completed_goal(&stored).unwrap();assert_eq!(stored.schema_version,63);assert_eq!(model::fingerprint(&stored.admissions[0].spec).unwrap(),model::fingerprint(&original).unwrap());
  let attempts=body(&c)["budgetUsage"]["attempts"].as_array().unwrap().clone();assert_eq!(attempts.len(),1);assert!(attempts.iter().all(|a|a["kind"]=="tool"));assert_eq!(c.owner.workers(),0);assert!(c.chat.runtime_resources().busy().unwrap().is_empty());assert!(c.driver.context_retirable());let _=run(&c.driver);assert_eq!(c.env.target.collector_calls(),1,"completed Driver cannot repeat the tool");
 }
}
#[test]fn system_requested_execution_manual_and_forged_choices_never_gain_authority(){
 let f=Fixture::new();let c=config("ai_ollama");save(&f,&c);let g=draft(&f,"ai_ollama");let r=review(&f,&g,&c,criteria::Predicate::SystemHostInventoryObserved).unwrap();let p=r.proposal();assert!(p.execution_start.is_none());let approved=r.approve(&g,evidence(&g,&c),clock()).unwrap();let a=Authority::ensure(&f.path.join("authority")).unwrap();let cert=a.certify(&approved,&g,clock()).unwrap();let q=f.store().enqueue_certified(&cert,clock()).unwrap().record;assert!(!q.admissions[0].spec.execution_requested());assert!(serde_json::to_value(&q.admissions[0].spec).unwrap().get("executionStart").is_none());
 let mut forged=q.clone();forged.admissions[0].spec.execution_start=Some(ExecutionStart{not_before_at:None,schema_version:1,mode:Start::WhenAvailable,scope:StartScope::NativeSystemPlan});assert!(forged.admissions[0].spec.binding_format());assert!(a.verify_queued(&forged,clock()).is_err(),"changing unsigned JSON must not add initial execution consent");
 for scope in [StartScope::FiniteResponse,StartScope::NativePeerPlan,StartScope::ResultDisclosure]{let mut s=forged.admissions[0].spec.clone();s.execution_start.as_mut().unwrap().scope=scope;assert!(!s.binding_format());}
 let requested=inventory("ai_ollama",Fault::None);let mut changed=requested.g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments=json!({"path":"/home"});assert!(!changed.admissions[0].spec.native_system_plan.as_ref().unwrap().matches(&changed,&changed.admissions[0].spec));
}
#[test]fn system_requested_execution_waits_for_the_shared_physical_resource_then_uses_the_same_recipe(){
 let c=inventory("agent_system_guardian",Fault::None);let key=crate::pet_runtime::lease_controller::step_controller::resource_key(c.env.target.binding()).unwrap();let hold=c.chat.runtime_resources().acquire(&key,Arc::new(AtomicBool::new(false)),None).unwrap();let held=Arc::new(Mutex::new(Some(hold)));let release=held.clone();*c.env.hook.lock().unwrap()=Some((Phase::WaitingResource,Box::new(move||{release.lock().unwrap().take();})));
 assert_eq!(run(&c.driver).unwrap().phase,Phase::Completed);assert!(held.lock().unwrap().is_none());assert_eq!(c.env.target.collector_calls(),1);assert_eq!(body(&c)["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert!(c.env.statuses.lock().unwrap().iter().any(|s|s.phase==Phase::WaitingResource));
}
#[test]fn system_requested_execution_stop_and_revocation_prevent_new_effects_and_drain_original_owner(){
 for revoke in [false,true]{let c=inventory("ai_ollama",Fault::None);let key=crate::pet_runtime::lease_controller::step_controller::resource_key(c.env.target.binding()).unwrap();let _hold=c.chat.runtime_resources().acquire(&key,Arc::new(AtomicBool::new(false)),None).unwrap();let path=c.env.path.clone();let owner=c.owner.clone();*c.env.hook.lock().unwrap()=Some((Phase::WaitingResource,Box::new(move||{
  if revoke{let mut config:crate::pet_config::Configuration=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();config.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().enabled=false;fs::write(&path,serde_json::to_vec(&config).unwrap()).unwrap();}else{crate::pet_runtime::lease_controller::signal_stop(owner.lease());}
 })));
 assert_eq!(run(&c.driver).unwrap().phase,Phase::Stopped);assert_eq!(c.env.target.collector_calls(),0);assert!(body(&c)["budgetUsage"]["attempts"].as_array().is_none_or(|a|a.is_empty()));assert_eq!(c.owner.workers(),0);assert!(c.driver.context_retirable());let budget=c.g.admissions[0].spec.budget.clone();let _=run(&c.driver);assert_eq!(c.env.target.collector_calls(),0);assert_eq!(c.f.store().read(&c.g.id,"ai_ollama").unwrap().admissions[0].spec.budget,budget);
 }
}
#[test]fn system_requested_execution_uncertain_completion_retains_receipts_without_repeating_the_effect(){
 let c=inventory("ai_ollama",Fault::AfterPublish);let first=run(&c.driver);assert!(first.is_err()||first.as_ref().is_ok_and(|s|s.phase==Phase::RecoveryRequired||s.phase==Phase::Closing));assert_eq!(c.env.target.collector_calls(),1);let attempts=body(&c)["budgetUsage"]["attempts"].clone();let _=run(&c.driver);assert_eq!(c.env.target.collector_calls(),1);assert_eq!(body(&c)["budgetUsage"]["attempts"],attempts);assert_eq!(c.owner.workers(),0);
}
#[test]fn system_requested_execution_baseline_uses_the_original_measured_receipt_and_zero_inference(){
 let c=baseline();assert_eq!(c.env.target.collector_calls(),0);let result=run(&c.driver).unwrap();assert_eq!(result.phase,Phase::Completed);assert_eq!(c.env.target.collector_calls(),1,"one original two-sample operation, no review/verifier resampling");let stored=c.f.store().read(&c.g.id,"agent_system_guardian").unwrap();c.a.verify_completed_goal(&stored).unwrap();let attempts=body(&c)["budgetUsage"]["attempts"].as_array().unwrap().clone();assert_eq!(attempts.len(),1);assert_eq!(attempts[0]["kind"],"tool");assert!(c.driver.context_retirable());
}
#[test]fn system_requested_execution_native_worker_survives_caller_and_stop_interrupts_the_baseline_wait(){
 let c=baseline();c.driver.spawn().unwrap();let until=std::time::Instant::now()+std::time::Duration::from_secs(10);
 while c.env.target.collector_calls()==0{assert!(std::time::Instant::now()<until);std::thread::sleep(std::time::Duration::from_millis(5));}
 crate::pet_runtime::lease_controller::signal_stop(c.owner.lease());
 while c.driver.status().unwrap().active{assert!(std::time::Instant::now()<until);std::thread::sleep(std::time::Duration::from_millis(5));}
 assert_eq!(c.driver.status().unwrap().phase,Phase::Stopped);assert_eq!(c.owner.workers(),0);assert_eq!(c.env.target.collector_calls(),1);assert_ne!(c.f.store().read(&c.g.id,"agent_system_guardian").unwrap().state,State::Completed);assert!(c.driver.context_retirable());let _=run(&c.driver);assert_eq!(c.env.target.collector_calls(),1);
}
#[test]#[ignore="Private native PipeWire/Pulse virtual output; no user audio or provider"]
fn system_requested_execution_sound_reuses_original_native_grants_and_zero_model_driver_for_seven_routes(){
 use crate::pet_runtime::native_system_admission_tests as original;
 use crate::pet_runtime::lease_controller::native_system_sound_task_tests::{Audio,serial};
 let _serial=serial();let audio=Audio::new();
 for provider in ["codex","copilot","antigravity","ollama","lmstudio","openai","integration_spotify"]{
  let f=Fixture::new();let g=original::draft(&f,&audio,provider);let config=original::config(&audio,&g);save(&f,&config);let r=original::review(&f,&audio,&g,&config).with_execution_start(Some(Start::WhenAvailable)).unwrap();let target=audio.target(&f.path.join("system.json"),&g.spec.owner.pet_id,false);let c=claimed(f,config,g,r,target,Fault::None);
  assert_eq!(run(&c.driver).unwrap().phase,Phase::Completed);audio.assert_volume(35.0);let stored=c.f.store().read(&c.g.id,&c.g.spec.owner.pet_id).unwrap();assert_eq!(stored.schema_version,62);c.a.verify_completed_goal(&stored).unwrap();let attempts=body(&c)["budgetUsage"]["attempts"].as_array().unwrap().clone();assert_eq!(attempts.len(),2);assert!(attempts.iter().all(|a|a["kind"]=="tool"));assert!(c.driver.context_retirable());let before=attempts;let _=run(&c.driver);assert_eq!(body(&c)["budgetUsage"]["attempts"],json!(before));
 }
}
