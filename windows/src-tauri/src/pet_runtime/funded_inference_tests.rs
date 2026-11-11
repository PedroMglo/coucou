//! Actual Authority/Store/F2/Driver/model protocol; all HTTP is synthetic.
#[path="peer_iteration_driver_tests.rs"]mod peer_iteration_driver_tests;
use super::*;
use crate::pet_runtime::{Repository,memory_review,received_context::{Selection,Captured},admission::{Evidence,Review as GoalReview,Denial},lease_controller::{Controller,Validity,dispatcher::{Driver,Environment,Capture,Status,Phase}},leases::DrainOutcome};
use crate::pet_runtime::lease_controller::{memory_test_http_fixture as http_fixture,memory_test_stream as stream,memory_test_metadata as metadata,memory_test_run as run};
use crate::pet_memory::context_model::GoalConsent;
use std::sync::Mutex;
// Only explicitly requested, unexpected fixture outcomes are retained. These
// non-atomic observations contain generated DATA, never authority/private keys.
fn retain_unexpected_driver(s:&Setup,g:&Record,driver:&Driver,label:&str){
 let Some(output)=std::env::var_os("COUCOU_RUNTIME_FAILURE_OUTPUT")else{return;};
 let Ok(id)=nonce()else{return;};let path=std::path::PathBuf::from(output).join(format!("{label}-{id}"));
 use std::os::unix::fs::PermissionsExt;
 if fs::create_dir_all(&path).is_err(){return;}let _=fs::set_permissions(&path,fs::Permissions::from_mode(0o700));
 let write=|name:&str,bytes:&[u8]|{use std::io::Write;let mut options=fs::OpenOptions::new();options.write(true).create_new(true);use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);if let Ok(mut file)=options.open(path.join(name)){let _=file.write_all(bytes);}};
 if let Ok(bytes)=serde_json::to_vec_pretty(&driver.status().ok()){write("status.json",&bytes);}
 let mut sources=vec![("goal.json",s.f.path.join("goals").join(format!("{}.json",g.id)))];
 if let Some(row)=g.admissions.last(){sources.push(("grant.json",s.f.path.join("authority").join(format!("{}.json",row.spec.decision_id))));}
 for(name,source)in sources{if fs::symlink_metadata(&source).is_ok_and(|m|m.is_file()&&!m.file_type().is_symlink()&&m.len()<=2*1024*1024){if let Ok(bytes)=fs::read(&source){write(name,&bytes);}}}
 eprintln!("[pet-runtime-fixture-witness] {label}: {}",path.display());
}
fn lm_metadata()->Value{json!({"models":[{"type":"llm","key":"fixture-free","publisher":"private-fixture","format":"gguf","size_bytes":1,"loaded_instances":[]}]})}
fn draft(s:&Setup,pet:&str,select:bool)->(Record,Record,crate::pet_runtime::messages::Receipt){draft_allowance(s,pet,select,Allowance{model_calls:1,tool_calls:1,output_bytes:4096})}
fn draft_allowance(s:&Setup,pet:&str,select:bool,allowance:Allowance)->(Record,Record,crate::pet_runtime::messages::Receipt){draft_allowance_memory(s,pet,select,allowance,None)}
fn draft_allowance_memory(s:&Setup,pet:&str,select:bool,allowance:Allowance,memory:Option<&[crate::pet_memory::context_model::Selected]>)->(Record,Record,crate::pet_runtime::messages::Receipt){draft_allowance_memory_criteria(s,pet,select,allowance,memory,vec![crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION.into()])}
fn draft_allowance_memory_criteria(s:&Setup,pet:&str,select:bool,allowance:Allowance,memory:Option<&[crate::pet_memory::context_model::Selected]>,criteria:Vec<String>)->(Record,Record,crate::pet_runtime::messages::Receipt){
 let sender=if pet=="ai_ollama"{"ai_lmstudio"}else{"ai_ollama"};let(p,o)=pool_to_allowance(s,sender,pet,criteria,1,budget(),allowance);let approved=approve(s,&p,&o,0);let r=o.send_message(&p,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();let done=o.verify_native_goal(&current_store(s),&p).unwrap().completion.unwrap().record;let proof=o.drain(DrainOutcome::Confirmed).unwrap();current_store(s).drain_owned(&s.a,&proof,crate::assistant_memory::now()).unwrap();drop(proof);drop(o);
 let g=response_draft_from_receipt(s,pet,&r,select,memory);(done,g,r)
}
fn response_draft_from_receipt(s:&Setup,pet:&str,r:&crate::pet_runtime::messages::Receipt,select:bool,memory:Option<&[crate::pet_memory::context_model::Selected]>)->Record{
 let mut own=data_review(s,r,"finite-funding-data").unwrap().approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();let st=current_store(s);let g=own.confirm(&st,crate::assistant_memory::now()).unwrap().record;
 let(b,c)=crate::pet_runtime::context::prepare_selected_data_fixture(&s.f.path.join("contexts"),&g,&s.main,memory,&s.f.path.join("notes.json"),&s.f.path.join("authority"),select).unwrap();drop(c);assert_eq!(b.schema_version,if select{4}else{1});
 let g=st.mutate(&g.id,pet,&g.revision,"funded-model-context".into(),Action::BindContext{binding:b},crate::assistant_memory::now()).unwrap().record;assert_eq!(g.schema_version,if select{30}else{29});
 let g=st.mutate(&g.id,pet,&g.revision,"funded-model-task".into(),Action::ProposeTask{description:"One own bounded model response; criteria still unverified".into()},crate::assistant_memory::now()).unwrap().record;
 let g=st.mutate(&g.id,pet,&g.revision,"funded-model-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Respond with selected data only".into(),postcondition:"Own artefact, never a self-certified result".into(),operation:None}]},crate::assistant_memory::now()).unwrap().record;g
}
fn evidence(s:&Setup,g:&Record,server:&crate::local_chat::Server,at:u64)->Evidence{
 let mut d=crate::pet_runtime::admission_tests::descriptor(g,at);if server.id=="lmstudio"{d.binding.engine_binding=Some(crate::local_chat::lmstudio::Catalog::parse(server,&lm_metadata()).unwrap().binding("fixture-free").unwrap());}
 let capture=memory_review::Captured::at_with_authority(&s.f.path.join("notes.json"),Some(&s.f.path.join("authority")),g,server,"fixture-free",at).unwrap();Evidence::from_memory_capture(g,&d,if s.f.path.join("native-qa-settings.json").exists(){crate::pet_runtime::native_validity::policy_revision(&s.config,&s.settings,g).unwrap()}else{"a".repeat(64)},at,false,false,capture).unwrap()
}
fn caps(r:&crate::pet_runtime::messages::Receipt)->Budget{Budget{deadline_at:r.envelope().deadline_at.min(crate::assistant_memory::now()+30_000),max_model_calls:1,max_tool_calls:0,max_output_bytes:4096,max_tokens:None,max_cost_micros:None,payment:Payment::LocalOnly}}
fn queue(s:&Setup,g:&Record,r:&crate::pet_runtime::messages::Receipt,server:&crate::local_chat::Server)->Record{
 let now=crate::assistant_memory::now();let a=GoalReview::prepare(g,evidence(s,g,server,now),caps(r),"funded-model-admission".into(),nonce().unwrap(),now).unwrap().approve_with_data(g,evidence(s,g,server,now),Some(GoalConsent::BoundedGoalContext),now).unwrap();assert_eq!(a.spec().schema_version,if g.spec.received_from.as_ref().unwrap().body.schema_version==7{23}else{12});assert_eq!(a.spec().memory_data.as_ref().unwrap().schema_version,if g.spec.received_from.as_ref().unwrap().body.schema_version==7{8}else{3});let cert=s.a.certify(&a,g,now).unwrap();current_store(s).enqueue_certified(&cert,now).unwrap().record
}
fn own(s:&Setup,q:&Record,server:&crate::local_chat::Server)->(Record,Arc<Owned>){
 let now=crate::assistant_memory::now();let p=Controller::new().unwrap().prepare_memory_fixture(q,&s.f.path.join("authority"),&s.f.path.join("contexts"),&s.f.path.join("notes.json"),&s.main,&evidence(s,q,server,now),format!("funded-model-claim-{}",q.admissions.last().unwrap().spec.decision_id),now,Arc::new(|_,_,_|Validity::Current)).unwrap();let(r,o)=p.claim(&current_store(s),&s.a,&evidence(s,q,server,crate::assistant_memory::now()),crate::assistant_memory::now()).unwrap();(r.record,Arc::new(o.unwrap()))
}
struct Env{provider:&'static str,url:String,config:crate::pet_config::Configuration,notices:Mutex<Vec<Status>>,captures:AtomicUsize,drains:AtomicUsize}
impl Environment for Env{
 fn capture(&self,_:&Owned)->model::Result<Capture>{self.captures.fetch_add(1,Ordering::AcqRel);Ok(Capture{configuration:self.config.clone(),server:crate::local_chat::Server{id:self.provider,name:"Synthetic free",url:self.url.clone(),key:None},app:None})}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.config.clone())}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("finite funded response must not access a system/media tool")}
 fn decide(&self,_:crate::pet_runtime::recipe_decisions::Source)->Option<crate::pet_runtime::recipe_decisions::Decision>{panic!("one model call cannot request tool approval")}
 fn notify(&self,s:&Status){self.notices.lock().unwrap().push(s.clone());}
 fn drained(&self,_:&crate::pet_runtime::lease_controller::DrainProof)->model::Result<()>{self.drains.fetch_add(1,Ordering::AcqRel);Ok(())}
}
fn environment(s:&Setup,server:&crate::local_chat::Server)->Arc<Env>{Arc::new(Env{provider:server.id,url:server.url.clone(),config:s.config.clone(),notices:Mutex::new(vec![]),captures:AtomicUsize::new(0),drains:AtomicUsize::new(0)})}
fn body(s:&Setup,g:&Record)->Value{serde_json::from_slice::<Value>(&s.bytes(g)).unwrap()["body"].clone()}
fn reconcile(s:&Setup,p:&Record,r:&crate::pet_runtime::messages::Receipt)->crate::pet_runtime::authority_store::delegation::View{Authority::existing(&s.f.path.join("authority")).unwrap().prepare_delegation_branch(p.clone(),&p.admissions[0].spec.decision_id,&r.envelope().message_id,crate::pet_runtime::delegation::BranchAction::Reconcile,crate::assistant_memory::now()).unwrap().confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap().view}
#[test]fn funded_model_requires_distinct_selected_data_goal_consent_exact_destination_and_finite_budget(){
 let s=Setup::new();let(_p,g,r)=draft(&s,"ai_ollama",true);let server=crate::local_chat::Server{id:"ollama",name:"No IO",url:"http://127.0.0.1:1".into(),key:None};let at=crate::assistant_memory::now();let capture=memory_review::Captured::at_with_authority(&s.f.path.join("notes.json"),Some(&s.f.path.join("authority")),&g,&server,"fixture-free",at).unwrap().unwrap();assert_eq!(capture.preview().schema_version,3);assert_eq!(capture.preview().received_request.as_ref().unwrap().selection.schema_version,2);assert!(!capture.preview().received_request.as_ref().unwrap().received_message.provider_use_authorized);assert_eq!(capture.confirm(&g,&server,"fixture-free",None,at).err(),Some("goalMemoryConsentRequired"));
 let fresh=memory_review::Captured::at_with_authority(&s.f.path.join("notes.json"),Some(&s.f.path.join("authority")),&g,&server,"fixture-free",at).unwrap().unwrap();let changed=crate::local_chat::Server{id:"ollama",name:"Changed endpoint",url:"http://127.0.0.1:2".into(),key:None};assert!(fresh.confirmation_current(&g,&changed,"fixture-free",at).is_err());assert!(fresh.confirmation_current(&g,&server,"different-model",at).is_err());
 let a=GoalReview::prepare(&g,evidence(&s,&g,&server,at),caps(&r),"without-own-data-consent".into(),nonce().unwrap(),at).unwrap();assert_eq!(a.approve(&g,evidence(&s,&g,&server,at),at).err(),Some(Denial::DataConsentRequired));
 for dim in["calls","tools","output","deadline"]{let mut b=caps(&r);match dim{"calls"=>b.max_model_calls=2,"tools"=>b.max_tool_calls=1,"output"=>b.max_output_bytes=4097,_=>b.deadline_at=r.envelope().deadline_at+1};assert!(GoalReview::prepare(&g,evidence(&s,&g,&server,at),b,"widen-funded-model".into(),nonce().unwrap(),at).is_err(),"{dim}");}
 let mut downgraded=g.clone();downgraded.schema_version=29;assert!(downgraded.validate().is_err());let mut selection=g.execution_context.as_ref().unwrap().received_request.clone().unwrap();selection.schema_version=1;assert!(!selection.valid_for(&g));assert!(Captured::at(&s.f.path.join("authority"),&g,&selection,at).is_err());let journal=s.f.path.join("contexts").join(&g.id).join("conversation.json");assert!(!String::from_utf8(fs::read(journal).unwrap()).unwrap().contains(&r.envelope().text));
 let s=Setup::new();let(_,g,r)=draft(&s,"ai_ollama",false);let at=crate::assistant_memory::now();assert_eq!(GoalReview::prepare(&g,evidence(&s,&g,&server,at),caps(&r),"no-selected-source".into(),nonce().unwrap(),at).err(),Some(Denial::DataConsentRequired));
}
#[test]fn funded_model_driver_runs_both_loopback_protocols_and_closes_without_claiming_criteria_or_renewing_budget(){
 for provider in["ollama","lmstudio"]{
 let meta=if provider=="ollama"{metadata()}else{lm_metadata()};let(mut server,http)=http_fixture(vec![("application/json",meta.to_string()),("text/event-stream",stream("Own artefact, not policy or completed criteria",true))],|_|{});server.id=provider;let s=Setup::new();let(p,g,r)=draft(&s,if provider=="ollama"{"ai_ollama"}else{"ai_lmstudio"},true);let q=queue(&s,&g,&r,&server);let(leased,o)=own(&s,&q,&server);let env=environment(&s,&server);let driver=Driver::new(o.clone(),Arc::new(current_store(&s)),env.clone()).unwrap();let status=run(driver.run()).unwrap();assert_eq!(status.phase,Phase::AwaitingVerification);let final_goal=current_store(&s).read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(final_goal.state,State::Paused);assert!(final_goal.completion.is_none());assert_eq!(final_goal.spec.criteria,r.envelope().delegation.as_ref().unwrap().criteria);assert_eq!(env.captures.load(Ordering::Acquire),1);assert_eq!(env.drains.load(Ordering::Acquire),1);assert_eq!(o.workers(),0);assert!(s.main.runtime_resources().busy().unwrap().is_empty());
 let requests=http.join().unwrap();assert_eq!(requests.len(),2);let input=&requests[1].1;let msgs=input["messages"].as_array().unwrap();let text=msgs[msgs.len()-2]["content"].as_str().unwrap();assert!(text.contains(&r.envelope().text));assert!(text.contains("receivedDelegationData"));assert!(!text.contains(&r.envelope().origin.lease_id)&&!text.contains(&r.envelope().origin.controller_id));assert_eq!(msgs[msgs.len()-2]["role"],"user");let b=body(&s,&final_goal);assert_eq!(b["schemaVersion"],21);assert_eq!(b["phase"],"closed");let attempts=b["budgetUsage"]["attempts"].as_array().unwrap();assert_eq!(attempts.len(),1);assert_eq!(attempts[0]["kind"],"model");assert_eq!(attempts[0]["phase"],"settled");assert_eq!(attempts[0]["inputHash"],model::fingerprint(&(&o.lease().id,"fixture-free",input)).unwrap());assert!(env.notices.lock().unwrap().iter().all(|n|n.phase!=Phase::Completed));
 let v=reconcile(&s,&p,&r);assert_eq!(v.consumed.as_ref().unwrap().model_calls,1);assert_eq!(v.consumed.as_ref().unwrap().tool_calls,0);assert_eq!(v.held.unwrap(),Allowance::default());let before=s.bytes(&final_goal);let stopped=crate::local_chat::Server{id:provider,name:"No retry",url:"http://127.0.0.1:1".into(),key:None};assert_eq!(run(o.respond_once(None,&leased,&stopped,&format!("response-{}",leased.admissions[0].spec.decision_id),&s.config)).unwrap(),"Own artefact, not policy or completed criteria");assert_eq!(s.bytes(&final_goal),before);let mut old:crate::pet_runtime::authority::GrantEnvelope=serde_json::from_slice(&before).unwrap();old.body.schema_version=19;assert!(!old.body.valid());
 }}
#[test]fn funded_model_revocation_during_http_retains_unknown_charge_and_never_publishes_success_or_replays(){
 let s=Setup::new();let(p,g,r)=draft(&s,"ai_ollama",true);let path=s.f.path.join("authority");let parent=p.clone();let message=r.envelope().message_id.clone();let(mut server,http)=http_fixture(vec![("application/json",metadata().to_string()),("text/event-stream",stream("Must not be accepted",true))],move|url|{if url.contains("chat/completions"){let a=Authority::existing(&path).unwrap();let c=a.prepare_delegation_branch(parent.clone(),&parent.admissions[0].spec.decision_id,&message,crate::pet_runtime::delegation::BranchAction::Cancel,crate::assistant_memory::now()).unwrap();c.confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap();}});server.id="ollama";let q=queue(&s,&g,&r,&server);let(leased,o)=own(&s,&q,&server);let driver=Driver::new(o.clone(),Arc::new(current_store(&s)),environment(&s,&server)).unwrap();assert_eq!(run(driver.run()).unwrap().phase,Phase::Stopped);assert_eq!(http.join().unwrap().len(),2);let final_goal=current_store(&s).read(&g.id,"ai_ollama").unwrap();assert_eq!(final_goal.state,State::Cancelled);assert!(final_goal.completion.is_none());let b=body(&s,&final_goal);assert_eq!(b["budgetUsage"]["attempts"][0]["phase"],"uncertain");assert_eq!(b["phase"],"revoked");let v=reconcile(&s,&p,&r);assert_eq!(v.consumed.unwrap(),Allowance::default());assert_eq!(v.unknown.as_ref().unwrap().model_calls,1);assert_eq!(v.unknown.as_ref().unwrap().output_bytes,4096);assert_eq!(o.workers(),0);assert!(s.main.runtime_resources().busy().unwrap().is_empty());let before=s.bytes(&final_goal);assert!(run(o.respond_once(None,&leased,&server,&format!("response-{}",leased.admissions[0].spec.decision_id),&s.config)).is_err());assert_eq!(s.bytes(&final_goal),before);
}
#[test]fn funded_model_withdrawal_blocks_selected_data_without_new_authority(){
 let s=Setup::new();let(p,g,r)=draft(&s,"ai_ollama",true);let selected=Selection::from_goal(&g).unwrap();let captured=Captured::at(&s.f.path.join("authority"),&g,&selected,crate::assistant_memory::now()).unwrap();let withdraw=Authority::existing(&s.f.path.join("authority")).unwrap().prepare_delegation_withdrawal(p.clone(),&p.admissions[0].spec.decision_id,&r.envelope().message_id,crate::assistant_memory::now()).unwrap();withdraw.confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap();assert!(captured.message(&g.id,crate::assistant_memory::now()).is_err());assert!(Captured::at(&s.f.path.join("authority"),&g,&selected,crate::assistant_memory::now()).is_err());assert!(current_store(&s).read_control(&g.id,"ai_ollama").unwrap().admissions.is_empty());
}

#[path="recursive_inference_tests.rs"]mod recursive;

#[path="funded_continuation_tests.rs"]mod continuation;

#[path="derived_result_plan_tests.rs"]mod derived_result;

#[path="result_disclosure_tests.rs"]mod result_disclosure;

#[path="result_delivery_tests.rs"]mod result_delivery;

#[path="peer_feedback_tests.rs"]mod peer_feedback_tests;

#[path="peer_round_data_tests.rs"]mod peer_round_data_tests;

#[path="funded_planning_tests.rs"]mod funded_planning;
