//! Real signed request/own origin, common admission and original worker. HTTP
//! tests use a loopback synthetic provider; no model/account quota is consumed.
use super::*;
use crate::pet_runtime::{received_work::{Input,State as WorkState},leases::DrainOutcome};
fn input(g:&Record)->Input{Input{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),source:g.spec.received_from.as_ref().unwrap().body.source.reference.clone()}}
fn observe(s:&Setup,g:&Record)->crate::pet_runtime::received_work::View{store(s).received_work(&s.f.path.join("authority"),&input(g),crate::assistant_memory::now()).unwrap()}
pub(super)fn own(s:&Setup,pet:&str)->Record{
 let(_sg,so,r)=source(s,pet);let d=so.drain(DrainOutcome::Confirmed).unwrap();s.f.store().drain_owned(&s.a,&d,crate::assistant_memory::now()).unwrap();drop(d);drop(so);
 let at=crate::assistant_memory::now();review(s,pet,&r,crate::settings::review::Frame::fixture(),"own-work-observation",at).unwrap().approve(at,Arc::new(||true)).unwrap().confirm(&store(s),at).unwrap().record
}
pub(super)fn planned(s:&Setup,g:&Record)->Record{
 let st=store(s);let now=crate::assistant_memory::now();let(binding,c)=crate::pet_runtime::context::prepare(&s.f.path.join("contexts"),g,&s.main,PreparationMode::Dedicated,"own-data").unwrap();drop(c);
 let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"own-data".into(),Action::BindContext{binding},now).unwrap().record;
 let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"own-task".into(),Action::ProposeTask{description:"Own bounded response".into()},now).unwrap().record;
 st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"own-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Own approved message".into(),postcondition:"Local persistence only".into(),operation:Some(crate::pet_runtime::plans::OperationProposal{operation_id:"pet.message".into(),version:1,arguments:json!({"toPetId":g.spec.received_from.as_ref().unwrap().body.source.sender_pet_id,"kind":"message","text":"Own bounded reply is data, not authority","expiresAt":now+60_000})})}]},now).unwrap().record
}
#[test]fn received_work_seven_owners_observe_proposal_without_model_or_mutation_and_reject_forged_scope(){
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  let s=all();let g=own(&s,pet);let before=crate::pet_runtime::reconciliation_tests::inventory(&s.f.path);let v=observe(&s,&g);assert_eq!(v.state,WorkState::Proposed);assert!(v.evidence_complete&&v.admission.is_none()&&!v.criteria_verified&&!v.execution_authorized);let wire=serde_json::to_string(&v).unwrap();assert!(!wire.contains(&g.spec.intent)&&!wire.contains("conversationId")&&!wire.contains("runtimeOwner"));
  for kind in ["pet","goal","revision","source"]{let mut wrong=input(&g);match kind{"pet"=>wrong.pet_id="integration_github".into(),"goal"=>wrong.goal_id=format!("g-{}","0".repeat(64)),"revision"=>wrong.revision="0".repeat(32),_=>wrong.source.scope_hash="0".repeat(64)};assert!(store(&s).received_work(&s.f.path.join("authority"),&wrong,crate::assistant_memory::now()).is_err());}
  assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&s.f.path),before);
  assert!(serde_json::from_value::<Input>(json!({"petId":pet,"goalId":g.id,"revision":g.revision,"source":input(&g).source,"state":"completed"})).is_err());
 }
}
#[test]fn received_work_queued_reserved_result_and_verified_completion_are_distinct_and_never_repeat_effects(){
 let s=all();let g=planned(&s,&own(&s,"ai_ollama"));let q=crate::pet_runtime::deterministic_runtime_tests::queued(&s.f,&s.a,&g);let v=observe(&s,&q);assert_eq!(v.state,WorkState::AcceptedQueued);assert!(v.admission.is_some()&&!v.criteria_verified);let expired=store(&s).received_work(&s.f.path.join("authority"),&input(&q),q.admissions[0].spec.budget.deadline_at).unwrap();assert_eq!(expired.state,WorkState::DeadlineElapsed);
 let(g,o)=crate::pet_runtime::deterministic_runtime_tests::owned(&s.f,&s.a,&q,&s.main);assert_eq!(observe(&s,&g).state,WorkState::SessionReserved);
 let a=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();let before=crate::pet_runtime::reconciliation_tests::inventory(&s.f.path);let v=observe(&s,&g);assert_eq!(v.state,WorkState::AwaitingVerification);assert_eq!(v.attempts.as_ref().unwrap().settled,1);assert!(!v.criteria_verified);assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&s.f.path),before);
 let completed=o.verify_native_goal(&store(&s),&g).unwrap().completion.unwrap().record;let d=o.drain(DrainOutcome::Confirmed).unwrap();s.f.store().drain_owned(&s.a,&d,crate::assistant_memory::now()).unwrap();drop(d);drop(o);
 let completed=store(&s).read(&completed.id,&completed.spec.owner.pet_id).unwrap();let v=observe(&s,&completed);assert_eq!(v.state,WorkState::Completed);assert!(v.criteria_verified&&!v.execution_authorized);assert_eq!(v.attempts.as_ref().unwrap().settled,1);
}
#[test]fn received_work_actual_http_inflight_then_saved_response_is_not_goal_completion(){
 use crate::pet_runtime::lease_controller::{memory_test_http_fixture as http_fixture,memory_test_stream as stream,memory_test_metadata as metadata,memory_test_run as run};
 let s=all();let(g,_r)=super::model_context::selected(&s,"own-work-http");let root=s.f.path.clone();let target=Arc::new(std::sync::Mutex::new(None::<Input>));let captured=target.clone();
 let(server,http)=http_fixture(vec![("application/json",metadata().to_string()),("text/event-stream",stream("Own saved result is not a verified criterion",true))],move|path|{if path.contains("chat/completions"){let st=Store::existing_verified(&root.join("goals"),&root.join("authority")).unwrap().unwrap();let target=captured.lock().unwrap();let v=st.received_work(&root.join("authority"),target.as_ref().unwrap(),crate::assistant_memory::now()).unwrap();assert_eq!(v.state,WorkState::AttemptInFlight);assert_eq!(v.attempts.as_ref().unwrap().started,1);assert!(!v.criteria_verified&&!v.execution_authorized);}});
 let(g,o)=super::model_context::setup_model(&s,&g,&server);*target.lock().unwrap()=Some(input(&g));run(o.respond_once(None,&g,&server,"own-work-synthetic-response",&s.config)).unwrap();assert_eq!(http.join().unwrap().len(),2);let v=observe(&s,&g);assert_eq!(v.state,WorkState::AwaitingVerification);assert!(!v.criteria_verified);assert_eq!(v.attempts.as_ref().unwrap().settled,1);
}
#[test]fn received_work_forged_origin_or_wrong_profile_never_becomes_status_evidence(){
 let s=all();let g=own(&s,"ai_ollama");let foreign=crate::pet_runtime::admission_tests::Fixture::new();Authority::ensure(&foreign.path.join("authority")).unwrap();assert!(store(&s).received_work(&foreign.path.join("authority"),&input(&g),crate::assistant_memory::now()).is_err());
 let p=s.f.path.join("goals").join(format!("{}.json",g.id));let mut raw=serde_json::to_value(&g).unwrap();raw["spec"]["receivedFrom"]["tag"]=json!("0".repeat(64));fs::write(&p,serde_json::to_vec(&raw).unwrap()).unwrap();let before=fs::read(&p).unwrap();assert!(store(&s).received_work(&s.f.path.join("authority"),&input(&g),crate::assistant_memory::now()).is_err());assert_eq!(fs::read(&p).unwrap(),before);
}
