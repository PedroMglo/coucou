//! Actual original-owner native Driver over signed admissions and durable ACKs.
use super::*;
use crate::pet_runtime::{admission::{Budget,Payment,Review,Denial},admission_tests::Fixture,authority::Authority,deterministic_runtime_tests::{draft_with,evidence,owned},lease_controller::{Owned,dispatcher::{self,Driver,Environment,Phase}},model::{State,CommitStatus},store::{Store,nonce},planning::Start};
use std::sync::{Arc,Mutex};
fn settings()->crate::settings::Settings{let mut s=crate::settings::Settings::default();s.active_integrations=vec!["ai_ollama".into(),"integration_spotify".into()];s}
fn observe()->Observe{Arc::new(||Ok((crate::pet_config::defaults(),settings())))}
fn review(f:&Fixture,g:&Record,now:u64)->Review{
 let b=Budget{deadline_at:now+120_000,max_model_calls:0,max_tool_calls:2,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly};
 Review::prepare_deterministic(g,evidence(g,now),b,"native-peer-reviewed".into(),nonce().unwrap(),now).unwrap().with_native_peer_plan(g,&f.path.join("goals"),&f.path.join("authority"),observe(),Some(Start::WhenAvailable),now).unwrap()
}
fn setup(provider:&str,known:bool)->(Fixture,Authority,Record,Owned){
 let f=Fixture::new();let a=Authority::ensure(&f.path.join("authority")).unwrap();let g=draft_with(&f,provider,vec![if known{crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION.into()}else{"Unverified outside outcome".into()}],2);let now=crate::assistant_memory::now();
 let r=review(&f,&g,now);let approved=r.approve_with_scopes(&g,evidence(&g,now),None,Some(crate::pet_runtime::disclosure_consent::Consent::BoundedNativePeerPlan),now).unwrap();let cert=a.certify(&approved,&g,now).unwrap();let q=f.store().enqueue_certified(&cert,now).unwrap().record;assert_eq!(q.schema_version,47);a.verify_queued(&q,now).unwrap();
 let(g,o)=owned(&f,&a,&q,&crate::chat::Chat::persistent(f.path.join("main.json")));(f,a,g,o)
}
struct Env{config:crate::pet_config::Configuration,settings:crate::settings::Settings,notices:Mutex<Vec<dispatcher::Status>>}
impl Environment for Env{
 fn capture(&self,_:&Owned)->model::Result<dispatcher::Capture>{panic!("native plan must not request any model")}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.config.clone())}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{Ok((self.config.clone(),self.settings.clone()))}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("native plan must not request a player effect")}
 fn decide(&self,_:crate::pet_runtime::recipe_decisions::Source)->Option<crate::pet_runtime::recipe_decisions::Decision>{panic!("already reviewed peer plan must not ask for another recipe")}
 fn notify(&self,s:&dispatcher::Status){self.notices.lock().unwrap().push(s.clone());}
}
fn run(f:&Fixture,o:Owned)->(Arc<Driver>,dispatcher::Status){run_with(f,o,crate::pet_config::defaults(),settings())}
pub(in crate::pet_runtime)fn run_with(f:&Fixture,o:Owned,config:crate::pet_config::Configuration,settings:crate::settings::Settings)->(Arc<Driver>,dispatcher::Status){let driver=Driver::new(Arc::new(o),Arc::new(Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap()),Arc::new(Env{config,settings,notices:Mutex::new(vec![])})).unwrap();let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();let s=rt.block_on(driver.run()).unwrap_or_else(|e|panic!("native peer driver {e:?}, status {:?}",driver.status().unwrap()));(driver,s)}
#[test]fn native_peer_driver_completes_reviewed_steps_for_every_supported_factory_identity_without_inference(){
 for provider in ["codex","copilot","antigravity","ollama","lmstudio","openai","integration_spotify"]{
  let(f,a,g,o)=setup(provider,true);let spec=g.admissions[0].spec.clone();let proof=g.admissions[0].proof.clone().unwrap();let policy=spec.native_peer_plan.clone().unwrap();let(_,s)=run(&f,o);assert_eq!(s.phase,Phase::Completed);assert_eq!(s.round,0,"literal native work has no model rounds");let loaded=Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(loaded.state,State::Completed);assert_eq!(loaded.schema_version,47);assert_eq!(loaded.admissions.len(),1);assert_eq!(loaded.admissions[0].spec.budget,spec.budget);a.verify_completed_goal(&loaded).unwrap();
  for step in policy.steps{let receipt=a.message_at(&spec,&proof,&step.cause.plan_id,&step.cause.step_id).unwrap().unwrap();assert!(!receipt.envelope().provider_use_authorized);assert_eq!(receipt.envelope().recipient,step.recipient);}
 }
}
#[test]fn native_peer_driver_publishes_receipts_but_never_claims_arbitrary_success(){let(f,_a,g,o)=setup("openai",false);let(_,s)=run(&f,o);assert_eq!(s.phase,Phase::AwaitingVerification);assert_eq!(s.round,0);let loaded=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(loaded.state,State::Paused);assert!(loaded.completion.is_none());}
// A reader can allow the initial Stop read but prevent its goal publication.
// The hook uses actual catalog locks/controls, never a fake authority result.
struct ClosureEnv{inner:Env,store:Arc<Store>,authority:std::path::PathBuf,goal_id:String,pet_id:String,cancel:bool,triggered:std::sync::atomic::AtomicBool,reader:Mutex<Option<crate::pet_runtime::store::Guard>>,closing_notices:std::sync::atomic::AtomicUsize}
impl Environment for ClosureEnv{
 fn capture(&self,o:&Owned)->model::Result<dispatcher::Capture>{self.inner.capture(o)}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{self.inner.configuration()}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{self.inner.peer_configuration()}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{self.inner.target()}
 fn decide(&self,s:crate::pet_runtime::recipe_decisions::Source)->Option<crate::pet_runtime::recipe_decisions::Decision>{self.inner.decide(s)}
 fn notify(&self,s:&dispatcher::Status){
  use std::sync::atomic::Ordering;
  self.inner.notify(s);
  if !self.cancel&&s.phase==Phase::Feedback&&!self.triggered.swap(true,Ordering::AcqRel){*self.reader.lock().unwrap()=Some(self.store.test_lock(false).unwrap());}
  if s.phase==Phase::Closing{
   let count=self.closing_notices.fetch_add(1,Ordering::AcqRel);
   if count>0{drop(self.reader.lock().unwrap().take());}
   if self.cancel&&!self.triggered.swap(true,Ordering::AcqRel){
    let g=self.store.read_control(&self.goal_id,&self.pet_id).unwrap();
    let receipt=crate::pet_runtime::authority_store::mutate_with_authority(self.store.as_ref(),&self.authority,&g.id,&g.spec.owner.pet_id,&g.revision,"cancel-during-original-closure".into(),model::Action::Cancel,crate::assistant_memory::now()).unwrap();assert_ne!(receipt.status,CommitStatus::Uncertain);
   }
  }
 }
}
#[test]fn native_peer_closure_controls_keep_busy_publication_and_concurrent_cancel_distinct(){
 let mut observed=vec![];
 for cancel in [true,false]{
  let(f,a,g,o)=setup("openai",false);let store=Arc::new(Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap());
  let env=Arc::new(ClosureEnv{inner:Env{config:crate::pet_config::defaults(),settings:settings(),notices:Mutex::new(vec![])},store:store.clone(),authority:f.path.join("authority"),goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),cancel,triggered:std::sync::atomic::AtomicBool::new(false),reader:Mutex::new(None),closing_notices:std::sync::atomic::AtomicUsize::new(0)});
  let driver=Driver::new(Arc::new(o),store.clone(),env.clone()).unwrap();let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();let result=runtime.block_on(driver.run());drop(env.reader.lock().unwrap().take());
  let status=driver.status().unwrap();let loaded=store.read(&g.id,&g.spec.owner.pet_id).unwrap();observed.push((cancel,status.phase,result.err(),loaded.state,status.retained_closure,driver.context_retirable()));assert!(loaded.completion.is_none());assert!(env.triggered.load(std::sync::atomic::Ordering::Acquire));if !cancel{assert!(env.closing_notices.load(std::sync::atomic::Ordering::Acquire)>=2,"the original closure must survive the lock conflict");}
  let policy=&g.admissions[0].spec.native_peer_plan.as_ref().unwrap();for step in &policy.steps{assert!(a.message_at(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&step.cause.plan_id,&step.cause.step_id).unwrap().is_some());}
  let bytes=std::fs::read(f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap();let body:serde_json::Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(body["body"]["messages"]["entries"].as_array().unwrap().len(),policy.steps.len());assert_eq!(loaded.admissions.len(),1);assert_eq!(model::fingerprint(&loaded.admissions[0].spec).unwrap(),model::fingerprint(&g.admissions[0].spec).unwrap());
 }
 assert_eq!(observed,vec![(true,Phase::Stopped,None,State::Cancelled,false,true),(false,Phase::AwaitingVerification,None,State::Paused,false,true)],"same receipts/admission, distinct closure outcomes");
}
#[test]fn native_peer_start_and_data_scopes_are_separate_and_wrong_scope_never_mints_authority(){
 let f=Fixture::new();let g=draft_with(&f,"copilot",vec!["Not proof".into()],2);let now=crate::assistant_memory::now();let r=review(&f,&g,now);
 assert_eq!(r.disclosure_consent_current(None,now),Err(Denial::DataConsentRequired));for wrong in [crate::pet_runtime::disclosure_consent::Consent::BoundedWorkDisclosure,crate::pet_runtime::disclosure_consent::Consent::BoundedResultDisclosure]{assert_eq!(r.disclosure_consent_current(Some(wrong),now),Err(Denial::Invalid));}
 assert!(!f.path.join("authority").exists());assert_eq!(r.approve(&g,evidence(&g,now),now).err(),Some(Denial::DataConsentRequired));assert!(!f.path.join("authority").exists());
}
#[test]fn native_peer_exact_envelope_ack_is_idempotent_and_mutating_a_signed_policy_cannot_reuse_authority(){
 let(f,a,g,o)=setup("ollama",false);let config=crate::pet_config::defaults();let settings=settings();let p=&g.admissions[0].spec.native_peer_plan.as_ref().unwrap().steps[0];let now=crate::assistant_memory::now();let target=Arc::new(messages::Target::capture(&config,&settings,&p.recipient.pet_id).unwrap());let r=o.review_message(&g,&p.cause.plan_id,&p.cause.step_id,"exact-peer-ack".into(),target,now).unwrap();let approved=r.approve_native_peer_plan(&o,&g,&config,&settings,now).unwrap();
 let first=o.send_message(&g,&approved,now,||Ok((config.clone(),settings.clone()))).unwrap();let before=std::fs::read(f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap();let second=o.send_message(&g,&approved,now,||panic!("ACK is observed before revalidation")).unwrap();assert_eq!(first,second);assert_eq!(std::fs::read(f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap(),before);
 let mut fake=g.clone();fake.admissions[0].spec.native_peer_plan.as_mut().unwrap().steps[0].recipient.name="Spoofed destination".into();assert!(a.verify_queued(&fake,now).is_err());
 let mut old=g.clone();old.schema_version=46;assert!(old.validate().is_err());let v=serde_json::to_value(&g).unwrap();assert_eq!(serde_json::from_value::<Record>(v).unwrap().schema_version,47);
 let _=CommitStatus::Durable;
}
