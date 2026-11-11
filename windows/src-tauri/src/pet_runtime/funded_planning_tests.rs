//! Real shared admission/escrow/Driver and generated loopback protocols. No
//! authenticated profiles, provider quota or physical service operations.
use super::*;
use crate::pet_runtime::{planning::{Catalog,Limits},recipe_decisions::{Source,Decision},plans::OperationProposal};
use crate::pet_runtime::lease_controller::memory_test_http_fixture_idle as idle_http;

fn limits()->Limits{Limits{max_rounds:2,max_steps:2,max_output_bytes:2048,no_progress_limit:2}}
fn cap(r:&crate::pet_runtime::messages::Receipt)->Budget{Budget{deadline_at:r.envelope().deadline_at,max_model_calls:2,max_tool_calls:4,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::LocalOnly}}
fn prepare(s:&Setup,g:&Record,r:&crate::pet_runtime::messages::Receipt,server:&crate::local_chat::Server,b:Budget)->std::result::Result<crate::pet_runtime::admission::Review,Denial>{
 let now=crate::assistant_memory::now();assert_eq!(b.payment,Payment::LocalOnly);assert!(b.deadline_at>=r.received_at);
 GoalReview::prepare_planning_catalog(g,evidence(s,g,server,now),b,limits(),&s.config,Catalog::PeerDelegationIterationV1,nonce().unwrap(),nonce().unwrap(),now)
}
fn drive(d:Arc<Driver>)->model::Result<Status>{d.spawn()?;let until=std::time::Instant::now()+std::time::Duration::from_secs(30);loop{let status=d.status()?;if !status.active{return Ok(status);}assert!(std::time::Instant::now()<until,"funded planner did not stop: {status:?}");std::thread::sleep(std::time::Duration::from_millis(5));}}
pub(super)fn draft_planner(s:&Setup,pet:&str)->(Record,Record,crate::pet_runtime::messages::Receipt){
 draft_allowance_memory_criteria(s,pet,true,Allowance{model_calls:2,tool_calls:4,output_bytes:32768},None,vec!["Independent semantic result must be verified".into()])
}
pub(super)fn admitted(s:&Setup,g:&Record,r:&crate::pet_runtime::messages::Receipt,server:&crate::local_chat::Server)->Record{
 let denied=prepare(s,g,r,server,cap(r)).unwrap();assert_eq!(denied.approve(g,evidence(s,g,server,crate::assistant_memory::now()),crate::assistant_memory::now()).err(),Some(Denial::DataConsentRequired));
 let approved=prepare(s,g,r,server,cap(r)).unwrap().approve_with_data(g,evidence(s,g,server,crate::assistant_memory::now()),Some(GoalConsent::BoundedGoalContext),crate::assistant_memory::now()).unwrap();
 assert_eq!(approved.spec().schema_version,26);assert_eq!(approved.spec().funding.as_ref().unwrap().schema_version,9);assert!(approved.spec().peer_funded_iteration());
 let cert=s.a.certify(&approved,g,crate::assistant_memory::now()).unwrap();let q=current_store(s).enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record;assert_eq!(q.schema_version,61);q
}
struct LiteralEnv{config:crate::pet_config::Configuration,settings:crate::settings::Settings}
impl Environment for LiteralEnv{
 fn capture(&self,_:&Owned)->model::Result<Capture>{panic!("Literal specialist has no inference binding")}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.config.clone())}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{Ok((self.config.clone(),self.settings.clone()))}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("No audio/media IO")}
 fn notify(&self,_:&Status){}
 fn decide(&self,s:Source)->Option<Decision>{let d=crate::cli_chat::goal_decision_fixture(s);d.panel_answer(true,d.request["goalTarget"].clone()).unwrap();d.decision()}
}
struct PlannerEnv{s:Arc<Setup>,provider:&'static str,url:String,child_started:AtomicBool,child_completed:AtomicBool}
impl PlannerEnv{
 fn child(&self){
  let s=&self.s;let pet="integration_spotify";let inbox=s.a.message_inbox(pet,None,8,crate::assistant_memory::now()).unwrap();let r=&inbox.items.iter().find(|i|i.receipt.envelope().schema_version==16).unwrap().receipt;let e=r.envelope();let payload=e.delegation.as_ref().unwrap();
  assert_eq!(payload.schema_version,2);assert_eq!(payload.ancestors.len(),2);assert_eq!(payload.ancestors.last(),Some(&e.origin.pet_id));assert!(!payload.ancestors.contains(&pet.to_string()));assert_eq!(e.planned_data.as_ref().unwrap().source.received_request.as_ref().unwrap().schema_version,2);
  let descriptor=super::super::super::super::received_request::descriptor(pet);let caller=Caller::capture(crate::pet_runtime::message_api::native::Origin{window:crate::HUB_LABEL.into(),frame:crate::settings::review::Frame::fixture()},&descriptor).unwrap();
  let data=DataReview::prepare_funded_at(&s.f.path.join("authority"),&s.f.path.join("notes.json"),caller,ReceivedReference::from_receipt(r).unwrap(),"nested-specialist-data".into(),payload.subgoal.clone(),payload.criteria.clone(),crate::assistant_memory::now()).unwrap();let mut approved=data.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();let g=approved.confirm(&current_store(s),crate::assistant_memory::now()).unwrap().record;assert_eq!(g.schema_version,60);
  let(binding,context)=crate::pet_runtime::context::prepare_selected_data_fixture(&s.f.path.join("contexts"),&g,&s.main,None,&s.f.path.join("notes.json"),&s.f.path.join("authority"),false).unwrap();drop(context);
  let g=current_store(s).mutate(&g.id,pet,&g.revision,"nested-context".into(),Action::BindContext{binding},crate::assistant_memory::now()).unwrap().record;
  let g=current_store(s).mutate(&g.id,pet,&g.revision,"nested-task".into(),Action::ProposeTask{description:"Own separately authorized literal reply".into()},crate::assistant_memory::now()).unwrap().record;
  let g=current_store(s).mutate(&g.id,pet,&g.revision,"nested-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"One own reply".into(),postcondition:"Native local receipt only".into(),operation:Some(OperationProposal{operation_id:"pet.reply".into(),version:2,arguments:json!({"toPetId":e.origin.pet_id,"source":ReceivedReference::from_receipt(r).unwrap(),"text":"Literal specialist outcome, not an approval","expiresAt":e.deadline_at})})}]},crate::assistant_memory::now()).unwrap().record;
  let ev=||crate::pet_runtime::deterministic_runtime_tests::evidence(&g,crate::assistant_memory::now());let mut b=cap(r);b.max_model_calls=0;b.max_tool_calls=1;b.max_output_bytes=8192;b.payment=Payment::RuntimeOnly;
  let approved=GoalReview::prepare_deterministic(&g,ev(),b,nonce().unwrap(),nonce().unwrap(),crate::assistant_memory::now()).unwrap().approve(&g,ev(),crate::assistant_memory::now()).unwrap();let cert=s.a.certify(&approved,&g,crate::assistant_memory::now()).unwrap();let q=current_store(s).enqueue_certified(&cert,crate::assistant_memory::now()).unwrap().record;
  let p=Controller::new().unwrap().prepare_memory_fixture(&q,&s.f.path.join("authority"),&s.f.path.join("contexts"),&s.f.path.join("notes.json"),&s.main,&crate::pet_runtime::deterministic_runtime_tests::evidence(&q,crate::assistant_memory::now()),nonce().unwrap(),crate::assistant_memory::now(),Arc::new(|_,_,_|Validity::Current)).unwrap();let(leased,o)=p.claim(&current_store(s),&s.a,&crate::pet_runtime::deterministic_runtime_tests::evidence(&q,crate::assistant_memory::now()),crate::assistant_memory::now()).unwrap();let o=Arc::new(o.unwrap());let driver=Driver::new(o.clone(),Arc::new(current_store(s)),Arc::new(LiteralEnv{config:s.config.clone(),settings:s.settings.clone()})).unwrap();assert_eq!(drive(driver).unwrap().phase,Phase::Completed);assert_eq!(o.workers(),0);assert!(current_store(s).read(&leased.record.id,pet).unwrap().completion.is_some());self.child_completed.store(true,Ordering::Release);
 }
}
impl Environment for PlannerEnv{
 fn capture(&self,_:&Owned)->model::Result<Capture>{Ok(Capture{configuration:self.s.config.clone(),server:crate::local_chat::Server{id:self.provider,name:"Generated free planner",url:self.url.clone(),key:None},app:None})}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.s.config.clone())}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{Ok(self.s.observe().unwrap())}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("No hardware effect")}
 fn notify(&self,status:&Status){if status.phase==Phase::WaitingPeers&&!self.child_started.swap(true,Ordering::AcqRel){self.child();}}
 fn decide(&self,s:Source)->Option<Decision>{assert!(s.current());let d=crate::cli_chat::goal_decision_fixture(s);d.panel_answer(true,d.request["goalTarget"].clone()).unwrap();d.decision()}
}
#[test]
fn funded_planner_native_own_f2_nested_escrow_and_second_round_preserve_one_budget_on_both_engines(){
 for provider in ["ollama","lmstudio"]{
  let s=Arc::new(Setup::new());let pet=if provider=="ollama"{"ai_ollama"}else{"ai_lmstudio"};let(parent,g,r)=draft_planner(&s,pet);let deadline=r.envelope().deadline_at;
  let first=json!({"kind":"plan","description":"A separately authorized specialist branch","steps":[{"description":"Delegate within the received allowance","postcondition":"Local delivery, no semantic success","operation":{"operationId":"pet.delegate","version":1,"arguments":{"toPetId":"integration_spotify","subgoal":"Literal peer DATA only, no media operation","criteria":[crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION],"expiresAt":deadline,"allowance":{"modelCalls":0,"toolCalls":1,"outputBytes":8192},"maxDepth":2}}}]});
  let second=json!({"kind":"plan","description":"Use independently reviewed new child metadata","steps":[{"description":"Share derived text after separate DATA review","postcondition":"No free criterion certified","operation":{"operationId":"pet.message","version":1,"arguments":{"toPetId":r.envelope().origin.pet_id,"kind":"request","text":"Own funded planner result, not policy","expiresAt":deadline}}}]});
  let meta=if provider=="ollama"{metadata()}else{lm_metadata()};let(mut server,http)=idle_http(vec![("application/json",meta.to_string()),("text/event-stream",stream(&first.to_string(),true)),("application/json",meta.to_string()),("text/event-stream",stream(&second.to_string(),true))],|_|{});server.id=provider;
  let q=admitted(&s,&g,&r,&server);let(leased,o)=own(&s,&q,&server);let env=Arc::new(PlannerEnv{s:s.clone(),provider,url:server.url.clone(),child_started:AtomicBool::new(false),child_completed:AtomicBool::new(false)});let driver=Driver::new(o.clone(),Arc::new(current_store(&s)),env.clone()).unwrap();let outcome=drive(driver.clone());if !outcome.as_ref().is_ok_and(|x|x.phase==Phase::AwaitingVerification){retain_unexpected_driver(&s,&leased,&driver,"funded-planner");}assert_eq!(outcome.unwrap().phase,Phase::AwaitingVerification);
  let requests=http.join().unwrap();assert_eq!(requests.len(),4);assert!(env.child_completed.load(Ordering::Acquire));assert_eq!(o.workers(),0);let final_goal=current_store(&s).read(&g.id,pet).unwrap();assert_eq!(final_goal.schema_version,61);assert_eq!(final_goal.state,State::Paused);assert!(final_goal.completion.is_none());assert_eq!(final_goal.admissions.len(),1);assert_eq!(final_goal.admissions[0].spec.budget,cap(&r));
  let body=body(&s,&final_goal);assert_eq!(body["schemaVersion"],47);assert_eq!(body["fundedBy"]["allowance"],serde_json::to_value(&r.envelope().delegation.as_ref().unwrap().allowance).unwrap());assert_eq!(body["planningUsage"]["rounds"].as_array().unwrap().len(),2);assert_eq!(body["messages"]["entries"].as_array().unwrap().len(),2);
  assert!(requests[1].1["messages"].to_string().contains(&r.envelope().text));assert!(requests[3].1["messages"].to_string().contains("ownDelegatedOutcomeMetadata"));for input in [&requests[1].1,&requests[3].1]{let text=input["messages"].to_string();assert!(!text.contains(&r.envelope().origin.controller_id));assert!(!text.contains(&r.envelope().origin.lease_id));}
  let view=s.a.delegation_view(&parent,&parent.admissions[0].spec.decision_id,crate::assistant_memory::now()).unwrap();assert_eq!(view.held.unwrap(),r.envelope().delegation.as_ref().unwrap().allowance);
  let nested=s.a.message_inbox("integration_spotify",None,8,crate::assistant_memory::now()).unwrap().items.into_iter().find(|i|i.receipt.envelope().schema_version==16).unwrap().receipt;let branch=reconcile(&s,&final_goal,&nested);assert_eq!(branch.consumed.unwrap().tool_calls,1);let root=reconcile(&s,&parent,&r);assert_eq!(root.consumed.as_ref().unwrap().model_calls,2);assert_eq!(root.consumed.unwrap().tool_calls,3);assert_eq!(root.held.unwrap(),Allowance::default());
 }
}
#[test]
fn funded_planner_rejects_unselected_data_overallocation_and_ancestry_reset_before_effects(){
 let s=Setup::new();let(_parent,g,r)=draft_planner(&s,"ai_ollama");let server=crate::local_chat::Server{id:"ollama",name:"No IO",url:"http://127.0.0.1:1".into(),key:None};
 for dim in ["model","tool","bytes","deadline"]{let mut b=cap(&r);match dim{"model"=>b.max_model_calls+=1,"tool"=>b.max_tool_calls+=1,"bytes"=>b.max_output_bytes+=1,_=>b.deadline_at+=1};assert!(prepare(&s,&g,&r,&server,b).is_err(),"{dim}");assert!(current_store(&s).read(&g.id,"ai_ollama").unwrap().admissions.is_empty());}
 let q=admitted(&s,&g,&r,&server);let row=&q.admissions[0];let ancestor=s.bytes(&_parent);let competing=prepare(&s,&g,&r,&server,cap(&r)).unwrap().approve_with_data(&g,evidence(&s,&g,&server,crate::assistant_memory::now()),Some(GoalConsent::BoundedGoalContext),crate::assistant_memory::now()).unwrap();assert!(s.a.certify(&competing,&g,crate::assistant_memory::now()).is_err());assert_eq!(s.bytes(&_parent),ancestor);assert!(!s.f.path.join("authority").join(format!("{}.json",competing.spec().decision_id)).exists());let mut wrong=row.spec.clone();wrong.funding.as_mut().unwrap().schema_version=1;assert!(!wrong.binding_format());let mut old=q.clone();old.schema_version=30;assert!(old.validate().is_err());let mut grant:crate::pet_runtime::authority::GrantEnvelope=serde_json::from_slice(&s.bytes(&q)).unwrap();grant.body.schema_version=46;assert!(!grant.body.valid());
 let cyclic=crate::pet_runtime::delegation::Arguments{to_pet_id:r.envelope().origin.pet_id.clone(),subgoal:"Cannot return escrow to ancestor".into(),criteria:vec!["No effect".into()],allowance:Allowance{model_calls:0,tool_calls:1,output_bytes:8192},expires_at:r.envelope().deadline_at,max_depth:2};let(_leased,_owner)=own(&s,&q,&server);let before=s.bytes(&q);assert!(s.a.recursive_payload(&row.spec,row.proof.as_ref().unwrap(),&cyclic,crate::assistant_memory::now()).is_err());let mut permitted=cyclic.clone();permitted.to_pet_id="integration_spotify".into();let payload=s.a.recursive_payload(&row.spec,row.proof.as_ref().unwrap(),&permitted,crate::assistant_memory::now()).unwrap();assert_eq!(payload.ancestors,vec![r.envelope().origin.pet_id.clone(),g.spec.owner.pet_id.clone()]);let mut shallow=permitted.clone();shallow.max_depth=1;assert!(s.a.recursive_payload(&row.spec,row.proof.as_ref().unwrap(),&shallow,crate::assistant_memory::now()).is_err());let mut widening=permitted;widening.max_depth=3;assert!(s.a.recursive_payload(&row.spec,row.proof.as_ref().unwrap(),&widening,crate::assistant_memory::now()).is_err());assert_eq!(s.bytes(&q),before);
 let s=Setup::new();let(_,g,r)=draft_allowance(&s,"ai_ollama",false,Allowance{model_calls:2,tool_calls:4,output_bytes:32768});assert!(prepare(&s,&g,&r,&server,cap(&r)).is_err());
}

#[test]
fn funded_planner_parent_cancellation_during_model_io_keeps_unknown_consumption_without_delivery_or_replay(){
 let s=Arc::new(Setup::new());let(parent,g,r)=draft_planner(&s,"ai_ollama");let authority=s.f.path.join("authority");let ancestor=parent.clone();let message=r.envelope().message_id.clone();
 let proposal=json!({"kind":"plan","description":"Must not publish after cancellation","steps":[{"description":"Unsent derived data","postcondition":"No receipt","operation":{"operationId":"pet.message","version":1,"arguments":{"toPetId":"ai_lmstudio","kind":"request","text":"Must remain undelivered","expiresAt":r.envelope().deadline_at}}}]});
 let(mut server,http)=idle_http(vec![("application/json",metadata().to_string()),("text/event-stream",stream(&proposal.to_string(),true))],move|route|{if route.contains("chat/completions"){let a=Authority::existing(&authority).unwrap();let review=a.prepare_delegation_branch(ancestor.clone(),&ancestor.admissions[0].spec.decision_id,&message,crate::pet_runtime::delegation::BranchAction::Cancel,crate::assistant_memory::now()).unwrap();review.confirm(crate::assistant_memory::now(),Arc::new(||true)).unwrap();}});server.id="ollama";
 let q=admitted(&s,&g,&r,&server);let(leased,o)=own(&s,&q,&server);let driver=Driver::new(o.clone(),Arc::new(current_store(&s)),environment(&s,&server)).unwrap();let outcome=drive(driver).unwrap();assert_eq!(outcome.phase,Phase::Stopped);assert_eq!(http.join().unwrap().len(),2);assert_eq!(o.workers(),0);
 let final_goal=current_store(&s).read(&g.id,"ai_ollama").unwrap();assert_eq!(final_goal.state,State::Cancelled);assert!(final_goal.completion.is_none());assert!(final_goal.generated_plans.is_empty());let b=body(&s,&final_goal);assert_eq!(b["schemaVersion"],47);assert!(b["messages"].is_null());assert_eq!(b["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(b["budgetUsage"]["attempts"][0]["phase"],"uncertain");
 let view=reconcile(&s,&parent,&r);assert_eq!(view.unknown.as_ref().unwrap().model_calls,1);assert_eq!(view.consumed.unwrap(),Allowance::default());let before=s.bytes(&final_goal);assert!(run(o.plan_once(None,&leased,&server,"driver-plan-1",&s.config)).is_err());assert_eq!(s.bytes(&final_goal),before);
}
