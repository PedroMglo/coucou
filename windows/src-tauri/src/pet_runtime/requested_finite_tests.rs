//! Initial finite execution uses existing signed admission/queue/resources.
use super::*;
use crate::pet_runtime::{admission::{ExecutionStart,StartScope},authority::Failure,model::{Action,StepInput}};
fn finite_named(request:&str,start:bool,extra_deadline:u64)->(Fixture,Authority,Record){
 finite_deferred(request,start,extra_deadline,None)
}
fn finite_deferred(request:&str,start:bool,extra_deadline:u64,delay:Option<u64>)->(Fixture,Authority,Record){
 let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=f.goal_named("ollama",request);let at=crate::assistant_memory::now();
 let st=f.store();let g=st.mutate(&g.id,"ai_ollama",&g.revision,format!("{request}-task"),Action::ProposeTask{description:"Own finite response".into()},at).unwrap().record;let g=st.mutate(&g.id,"ai_ollama",&g.revision,format!("{request}-plan"),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Respond once".into(),postcondition:"Own result; criteria not verified".into(),operation:None}]},at).unwrap().record;
 let b=crate::pet_runtime::admission::Budget{deadline_at:at+60_000+extra_deadline,max_model_calls:1,max_tool_calls:0,max_output_bytes:4096,max_tokens:None,max_cost_micros:None,payment:Payment::LocalOnly};
 let rev=Review::prepare(&g,evidence(&g,at),b,format!("{request}-admit"),nonce().unwrap(),at).unwrap().with_execution_start(start.then_some(Start::WhenAvailable)).unwrap().with_not_before(delay.map(|d|at+d)).unwrap();let approved=rev.approve(&g,evidence(&g,at),at).unwrap();let cert=a.certify(&approved,&g,at).unwrap();let q=f.store().enqueue_certified(&cert,at).unwrap().record;(f,a,q)
}
#[test]fn scheduled_start_native_authority_refuses_early_claim_and_budget_writer_without_any_slot_or_effect(){
 let(f,a,g)=finite_deferred("scheduled-native",true,0,Some(20_000));let row=&g.admissions[0];let at=row.spec.not_before().unwrap();assert_eq!(g.schema_version,66);assert_eq!(row.spec.execution_start.as_ref().unwrap().schema_version,2);assert!(matches!(status_at(&empty_service(),&g,at-1).unwrap().phase,Phase::Scheduled));let before=record_bytes(&f,&g);
 assert_eq!(a.verify_queued(&g,at-1).err(),Some(Failure::NotDue));assert!(a.verify_queued(&g,at).is_ok());
 let main=crate::chat::Chat::persistent(f.path.join("manual.json"));let controller=crate::pet_runtime::lease_controller::Controller::new().unwrap();let result=controller.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,at-1),"early-claim".into(),at-1,Arc::new(|_,_,_|crate::pet_runtime::lease_controller::Validity::Current));assert!(result.is_err());
 let slot=main.runtime_resources().acquire_goal("ollama",Arc::new(AtomicBool::new(false)),Some(Instant::now()+Duration::from_secs(1))).unwrap();drop(slot);
 let mut lease=crate::pet_runtime::leases::Lease{schema_version:1,id:nonce().unwrap(),controller_id:nonce().unwrap(),request_id:"forged-early-lease".into(),decision_id:row.spec.decision_id.clone(),goal_id:g.id.clone(),provider:"ollama".into(),conversation_id:row.spec.context.conversation_id.clone(),model:row.spec.resolved_model.clone(),policy_revision:row.spec.policy_revision.clone(),capability_revision:row.spec.descriptor_revision.clone(),source_revision:g.revision.clone(),claimed_at:at-1,deadline_at:row.spec.budget.deadline_at};assert!(!lease.valid(&row.spec));lease.claimed_at=at;assert!(lease.valid(&row.spec));assert_eq!(a.output_remaining(&row.spec,row.proof.as_ref().unwrap(),&lease,at-1).err(),Some(Failure::NotDue));assert_eq!(record_bytes(&f,&g),before);
 let reopened=native(&f).read(&g.id,"ai_ollama").unwrap();assert_eq!(reopened.admissions[0].spec.not_before(),Some(at));assert!(reopened.admissions[0].lease.is_none());let mut old=reopened.clone();old.schema_version=46;assert!(old.validate().is_err());
}
#[test]fn scheduled_start_tampering_and_stop_cannot_renew_time_or_authority(){
 let(f,a,g)=finite_deferred("scheduled-stop",true,0,Some(20_000));let now=crate::assistant_memory::now();let before=record_bytes(&f,&g);let mut fake=g.clone();fake.admissions[0].spec.execution_start.as_mut().unwrap().not_before_at=Some(1);fake.events.last_mut().unwrap().fingerprint=model::fingerprint(&fake.admissions[0].spec).unwrap();fake.validate().unwrap();assert!(a.verify_queued(&fake,now).is_err());assert_eq!(record_bytes(&f,&g),before);
 let stopped=crate::pet_runtime::authority_store::mutate_with_authority(&native(&f),&root(&f),&g.id,"ai_ollama",&g.revision,"stop-before-time".into(),Action::Cancel,now).unwrap().record;assert_eq!(stopped.state,State::Cancelled);assert!(status_at(&empty_service(),&stopped,now).is_none());assert!(a.verify_queued(&stopped,g.admissions[0].spec.not_before().unwrap()).is_err());
 let(_,_,manual)=finite_named("legacy-start-bytes",true,0);let start=serde_json::to_value(manual.admissions[0].spec.execution_start.as_ref().unwrap()).unwrap();assert_eq!(start,serde_json::json!({"schemaVersion":1,"mode":"whenAvailable","scope":"finiteResponse"}));
 let mut invalid=g.admissions[0].spec.clone();for value in [0,invalid.budget.deadline_at]{invalid.execution_start.as_mut().unwrap().not_before_at=Some(value);assert!(!invalid.binding_format());}
}
#[test]fn requested_finite_policy_survives_reopen_without_new_admission_budget_model_or_controller(){
 let(f,a,g)=finite_named("finite-start",true,0);assert_eq!(g.schema_version,46);let row=&g.admissions[0];assert_eq!(row.spec.schema_version,2);assert!(row.spec.finite_response()&&requested(&g));assert!(row.spec.planning_policy.is_none()&&row.spec.funding.is_none());assert_eq!(row.spec.execution_start.as_ref().unwrap().scope,StartScope::FiniteResponse);let before=record_bytes(&f,&g);
 for _ in 0..2{let current=native(&f).acknowledge_requested(&g.id,"ai_ollama",&g.revision,crate::assistant_memory::now()).unwrap();assert_eq!(model::fingerprint(&current).unwrap(),model::fingerprint(&g).unwrap());assert!(current.admissions[0].lease.is_none());assert_eq!(record_bytes(&f,&g),before);}
 a.verify_queued(&g,crate::assistant_memory::now()).unwrap();let mut old=g.clone();old.schema_version=6;assert!(old.validate().is_err());let mut malformed=serde_json::to_value(&row.spec).unwrap();malformed["executionStart"]["scope"]=serde_json::json!("unlimitedShell");assert!(serde_json::from_value::<crate::pet_runtime::admission::AdmissionSpec>(malformed).is_err());
}
#[test]fn old_manual_finite_bounds_cannot_be_promoted_by_coherent_json_or_broadened_start_scope(){
 let(f,a,manual)=finite_named("manual-original",false,0);assert_ne!(manual.schema_version,46);assert!(!requested(&manual));assert!(serde_json::to_value(&manual.admissions[0].spec).unwrap().get("executionStart").is_none());let before=record_bytes(&f,&manual);
 let mut fake=manual.clone();fake.admissions[0].spec.execution_start=Some(ExecutionStart{not_before_at:None,schema_version:1,mode:Start::WhenAvailable,scope:StartScope::FiniteResponse});fake.schema_version=46;fake.events.last_mut().unwrap().fingerprint=model::fingerprint(&fake.admissions[0].spec).unwrap();fake.validate().unwrap();assert!(a.acknowledge_requested(&fake,crate::assistant_memory::now()).is_err());assert_eq!(record_bytes(&f,&manual),before);
 for dim in ["models","tools","payment","version","scope"]{let mut s=fake.admissions[0].spec.clone();match dim{"models"=>s.budget.max_model_calls=2,"tools"=>s.budget.max_tool_calls=1,"payment"=>s.budget.payment=Payment::ExistingAccount,"version"=>s.execution_start.as_mut().unwrap().schema_version=0,_=>s.execution_start.as_mut().unwrap().scope=StartScope::ResultDisclosure};assert!(!s.binding_format(),"{dim}");}
}
#[test]fn two_requested_jobs_keep_original_owners_and_second_wait_while_stopped_worker_still_owns_slot(){
 let(f1,_a1,g1)=finite_named("first-independent",true,0);let(f2,_a2,g2)=finite_named("second-independent",true,1000);let c=Arc::new(crate::pet_resources::Coordinator::default());let first=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let worker=first.poll().unwrap();let second=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();assert!(matches!(second.poll(),Err(ResourceFailure::Busy)));let first_bytes=record_bytes(&f1,&g1);let second_bytes=record_bytes(&f2,&g2);let mut service=empty_service();service.waiting.insert(g1.id.clone(),waiting(&g1,first));service.waiting.insert(g2.id.clone(),waiting(&g2,second.clone()));
 expire_waits(&mut service,g1.admissions[0].spec.budget.deadline_at);assert!(service.waiting[&g1.id].resource.is_none());assert!(service.waiting[&g2.id].resource.is_some());assert!(!worker.current()&&worker.occupies_slot());assert!(matches!(second.poll(),Err(ResourceFailure::Busy)));assert!(same_admission(&service.waiting[&g2.id].source,&g2));drop(worker);assert!(second.poll().is_ok());assert_eq!(record_bytes(&f1,&g1),first_bytes);assert_eq!(record_bytes(&f2,&g2),second_bytes);
}

#[test]fn nonblocking_advances_are_bounded_and_late_results_cannot_remove_a_replacement_or_resurrect_stop(){
 let mut fixtures=Vec::new();let mut service=empty_service();let c=Arc::new(crate::pet_resources::Coordinator::default());let mut originals=Vec::new();
 for i in 0..9{let(f,a,g)=finite_named(&format!("independent-{i}"),true,0);let r=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();service.waiting.insert(g.id.clone(),waiting(&g,r));originals.push(g.clone());fixtures.push((f,a));}
 let mut advances=Vec::new();for g in &originals[..8]{advances.push(take_advance(&mut service,&g.id).unwrap());}
 assert!(take_advance(&mut service,&originals[8].id).is_none());assert!(take_advance(&mut service,&originals[0].id).is_none());assert_eq!(service.waiting.values().filter(|w|w.advancing).count(),8);
 // A returning observation is fenced to its original claim and signed admission.
 let first=&advances[0];let mut replacement=first.clone();replacement.claim_request=nonce().unwrap();service.waiting.insert(first.source.id.clone(),replacement.clone());finish_advance_in(&mut service,first,Ok(true));assert!(same_wait(&service.waiting[&first.source.id],&replacement));
 let second=&advances[1];{let w=service.waiting.get_mut(&second.source.id).unwrap();w.phase=Phase::Stopped;w.resource.take().unwrap().cancel();}
 finish_advance_in(&mut service,second,Err(Error::Busy));let stopped=&service.waiting[&second.source.id];assert!(!stopped.advancing&&stopped.resource.is_none()&&matches!(stopped.phase,Phase::Stopped));assert!(take_advance(&mut service,&originals[8].id).is_some());
 let third=&advances[2];let at=third.source.admissions[0].spec.budget.deadline_at;expire_waits(&mut service,at);prune_expired_statuses(&mut service,at);assert!(service.waiting.contains_key(&third.source.id));finish_advance_in(&mut service,third,Err(Error::Conflict));prune_expired_statuses(&mut service,at);assert!(!service.waiting.contains_key(&third.source.id));
 for(i,g)in originals.iter().enumerate(){assert_eq!(native(&fixtures[i].0).read(&g.id,"ai_ollama").unwrap().revision,g.revision);}
}

#[test]fn requested_metadata_uses_the_signed_finite_or_planning_policy_and_never_promotes_manual_or_tool_scope(){
 use crate::pet_runtime::readiness_api::requested_inference;
 let(_f,_a,g)=finite_named("finite-native-metadata",true,0);assert!(requested_inference(&g.admissions[0].spec));let(_f,_a,g)=finite_named("manual-native-metadata",false,0);assert!(!requested_inference(&g.admissions[0].spec));
 let(_f,_a,g)=setup_named(Some(Start::WhenAvailable),"planning-native-metadata");assert!(requested_inference(&g.admissions[0].spec));let mut invalid=g.admissions[0].spec.clone();invalid.execution_start=Some(ExecutionStart{not_before_at:None,schema_version:1,mode:Start::WhenAvailable,scope:StartScope::ResultDisclosure});assert!(!requested_inference(&invalid));
 let(_f,_a,g)=setup_named(None,"manual-planning-native-metadata");assert!(!requested_inference(&g.admissions[0].spec));
}
