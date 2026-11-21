//! Finite future reporting over original runtime/admission/receipt machinery.
use super::*;
use crate::pet_runtime::{work_disclosure::{self,Policy,Consent},work_report_api,admission::{Review as AdmissionReview,Budget,Payment},lease_controller::dispatcher::{Driver,Environment,Capture,Status},recipe_decisions::{Source,Decision}};
fn future(s:&Setup,subject:&Record)->(Record,Policy){future_states(s,subject,None)}
fn future_states(s:&Setup,subject:&Record,states:Option<Vec<crate::pet_runtime::received_work::State>>)->(Record,Policy){
 let now=crate::assistant_memory::now();let source=subject.spec.received_from.as_ref().unwrap().body.source.clone();let d=work_report_api::prepare(&s.f.path.join("goals"),&s.f.path.join("authority"),work_report_api::Input{autonomous:true,states,pet_id:subject.spec.owner.pet_id.clone(),goal_id:subject.id.clone(),revision:subject.revision.clone(),source:source.reference,expires_at:source.deadline_at,include_attempts:false},now).unwrap();assert!(d.future_disclosure.is_some());assert_eq!(d.steps[0].operation.as_ref().unwrap().version,2);
 let st=store(s);let g=st.create(Spec{received_from:None,restored_from:None,request_id:nonce().unwrap(),owner:subject.spec.owner.clone(),context:Context::Unbound,intent:"Finite explicitly authorized future metadata only".into(),criteria:d.criteria},now).unwrap().record;
 let(binding,c)=crate::pet_runtime::context::prepare(&s.f.path.join("contexts"),&g,&s.main,PreparationMode::Dedicated,"future-context").unwrap();drop(c);let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"future-context".into(),Action::BindContext{binding},now).unwrap().record;
 let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"future-task".into(),Action::ProposeTask{description:"Finite typed future report".into()},now).unwrap().record;let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"future-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:d.steps},now).unwrap().record;assert_eq!(g.schema_version,27);
 let p=Policy::capture(&g,&s.f.path.join("goals"),&s.f.path.join("authority"),&s.config,&s.settings,now).unwrap();(g,p)
}
fn review(g:&Record,p:Policy)->AdmissionReview{let now=crate::assistant_memory::now();AdmissionReview::prepare_deterministic(g,crate::pet_runtime::deterministic_runtime_tests::evidence(g,now),Budget{deadline_at:p.arguments.expires_at,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly},"future-admission".into(),nonce().unwrap(),now).unwrap().with_work_disclosure(p).unwrap()}
fn queue(s:&Setup,g:&Record,p:Policy)->Record{let now=crate::assistant_memory::now();let approved=review(g,p).approve_with_scopes(g,crate::pet_runtime::deterministic_runtime_tests::evidence(g,now),None,Some(Consent::BoundedWorkDisclosure),now).unwrap();let cert=s.a.certify(&approved,g,now).unwrap();store(s).enqueue_certified(&cert,now).unwrap().record}
fn cancel(s:&Setup,g:&Record)->Record{crate::pet_runtime::authority_store::mutate_with_authority(&store(s),&s.f.path.join("authority"),&g.id,&g.spec.owner.pet_id,&g.revision,"subject-final-cancel".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record}
struct Env{config:crate::pet_config::Configuration,settings:Settings,statuses:Mutex<Vec<Status>>}
impl Environment for Env{
 fn capture(&self,_:&Owned)->model::Result<Capture>{panic!("No inference capture for deterministic future reports")}
 fn configuration(&self)->model::Result<crate::pet_config::Configuration>{Ok(self.config.clone())}
 fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{panic!("No system/media target")}
 fn decide(&self,_:Source)->Option<Decision>{panic!("No implicit replacement of admission disclosure consent")}
 fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,Settings)>{Ok((self.config.clone(),self.settings.clone()))}
 fn notify(&self,s:&Status){self.statuses.lock().unwrap().push(s.clone());}
}
#[test]fn future_work_disclosure_requires_own_explicit_scope_and_old_reader_refuses_format(){
 let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let(g,p)=future(&s,&subject);let now=crate::assistant_memory::now();assert_eq!(review(&g,p.clone()).approve(&g,crate::pet_runtime::deterministic_runtime_tests::evidence(&g,now),now).err(),Some(crate::pet_runtime::admission::Denial::DataConsentRequired));
 let q=queue(&s,&g,p);assert_eq!(q.admissions[0].spec.schema_version,9);let raw:Value=serde_json::from_slice(&s.bytes(&q)).unwrap();assert_eq!(raw["body"]["schemaVersion"],17);let mut old=q.clone();old.schema_version=26;assert!(old.validate().is_err());let mut old=raw.clone();old["body"]["schemaVersion"]=json!(16);let parsed:crate::pet_runtime::authority::GrantEnvelope=serde_json::from_value(old).unwrap();assert!(!parsed.body.valid());
}
#[test]fn future_work_disclosure_waits_without_claim_then_tracks_only_same_subject_spec_and_trigger(){
 let s=all();let subject=super::work_observation::own(&s,"ai_ollama");let(g,p)=future(&s,&subject);let q=queue(&s,&g,p.clone());let before=s.bytes(&q);assert!(!work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).unwrap());assert_eq!(s.bytes(&q),before);assert!(q.admissions[0].lease.is_none());
 let cancelled=cancel(&s,&subject);assert_ne!(cancelled.revision,subject.revision);assert!(work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).unwrap());let c=p.arguments.capture(&s.f.path.join("goals"),&s.f.path.join("authority"),&subject.spec.owner.pet_id,crate::assistant_memory::now()).unwrap();assert_eq!(c.report.subject_revision,cancelled.revision);assert_eq!(c.report.state,crate::pet_runtime::received_work::State::Cancelled);assert_eq!(s.bytes(&q),before);
}
#[tokio::test]async fn future_work_disclosure_common_driver_seven_owners_send_once_and_verify_without_model_or_new_decision(){
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  let s=all();let subject=super::work_observation::own(&s,pet);let(g,p)=future(&s,&subject);let q=queue(&s,&g,p);let subject=cancel(&s,&subject);let(g,o)=crate::pet_runtime::deterministic_runtime_tests::owned(&s.f,&s.a,&q,&s.main);let env=Arc::new(Env{config:s.config.clone(),settings:s.settings.clone(),statuses:Mutex::new(vec![])});let driver=Driver::new(Arc::new(o),Arc::new(store(&s)),env.clone()).unwrap();let status=driver.run().await.unwrap();assert_eq!(status.phase,crate::pet_runtime::lease_controller::dispatcher::Phase::Completed);assert!(!status.active);
  let final_goal=store(&s).read(&g.id,pet).unwrap();assert_eq!(final_goal.state,State::Completed);assert_eq!(final_goal.schema_version,27);let raw:Value=serde_json::from_slice(&s.bytes(&final_goal)).unwrap();assert_eq!(raw["body"]["schemaVersion"],17);let entries=raw["body"]["messages"]["entries"].as_array().unwrap();assert_eq!(entries.len(),1);let report=&entries[0]["scope"]["envelope"]["workReport"];assert_eq!(report["state"],"cancelled");assert_eq!(report["subjectRevision"],subject.revision);assert_eq!(raw["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert!(env.statuses.lock().unwrap().iter().all(|s|s.phase!=crate::pet_runtime::lease_controller::dispatcher::Phase::Planning));
 }
}
#[test]fn future_work_disclosure_forged_fields_origin_or_subject_cannot_expand_the_policy(){
 let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let(g,p)=future(&s,&subject);for field in ["model","state","petId","command"]{let mut v=serde_json::to_value(&p.arguments).unwrap();v[field]=json!("forged");assert!(work_disclosure::Arguments::parse(&v).is_err());}
 for kind in ["source","actor","spec"]{let mut a=p.arguments.clone();let pet=if kind=="actor"{"agent_codex"}else{&subject.spec.owner.pet_id};if kind=="source"{a.source.scope_hash="0".repeat(64);}if kind=="spec"{a.subject_spec_hash="0".repeat(64);}assert!(a.capture(&s.f.path.join("goals"),&s.f.path.join("authority"),pet,crate::assistant_memory::now()).is_err());}let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments["includeAttempts"]=json!(true);assert!(!p.matches(&changed));
}
#[test]fn future_work_disclosure_revocation_before_trigger_blocks_observation_and_publication(){
 let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let(g,p)=future(&s,&subject);let q=queue(&s,&g,p);let row=&q.admissions[0];s.a.revoke(&row.spec,row.proof.as_ref().unwrap(),crate::assistant_memory::now()).unwrap();let _=cancel(&s,&subject);let frozen=s.bytes(&q);assert!(work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).is_err());assert_eq!(s.bytes(&q),frozen);assert!(s.budget(&q).is_null());
}

#[tokio::test]async fn explicit_pause_followup_waits_then_driver_delivers_one_report_with_original_scope_and_zero_models(){
 use crate::pet_runtime::received_work::State as WorkState;
 let s=all();let subject=super::work_observation::own(&s,"agent_codex");let(g,p)=future_states(&s,&subject,Some(vec![WorkState::Paused,WorkState::AwaitingVerification,WorkState::RecoveryRequired,WorkState::DeadlineElapsed,WorkState::Completed,WorkState::Cancelled]));
 let original_policy=p.clone();let q=queue(&s,&g,p);let original_subject=store(&s).read(&subject.id,&subject.spec.owner.pet_id).unwrap();let before=s.bytes(&q);assert!(!work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).unwrap());assert!(q.admissions[0].lease.is_none());assert_eq!(s.bytes(&q),before);
 let paused=store(&s).mutate(&subject.id,&subject.spec.owner.pet_id,&original_subject.revision,"pause-needs-human-decision".into(),Action::Pause,crate::assistant_memory::now()).unwrap().record;
 assert!(work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).unwrap());let(g,o)=crate::pet_runtime::deterministic_runtime_tests::owned(&s.f,&s.a,&q,&s.main);assert_eq!(g.admissions[0].spec.work_disclosure.as_ref(),Some(&original_policy));
 let env=Arc::new(Env{config:s.config.clone(),settings:s.settings.clone(),statuses:Mutex::new(vec![])});let driver=Driver::new(Arc::new(o),Arc::new(store(&s)),env).unwrap();let done=driver.run().await.unwrap();assert_eq!(done.phase,crate::pet_runtime::lease_controller::dispatcher::Phase::Completed);
 let finished=store(&s).read(&g.id,&subject.spec.owner.pet_id).unwrap();assert_eq!(finished.schema_version,27);assert_eq!(finished.admissions[0].spec.budget.max_model_calls,0);assert_eq!(finished.admissions[0].spec.budget.max_tool_calls,1);let raw:Value=serde_json::from_slice(&s.bytes(&finished)).unwrap();assert_eq!(raw["body"]["schemaVersion"],17);let entries=raw["body"]["messages"]["entries"].as_array().unwrap();assert_eq!(entries.len(),1);let report=&entries[0]["scope"]["envelope"]["workReport"];assert_eq!(report["state"],"paused");assert_eq!(report["subjectRevision"],paused.revision);assert_eq!(report["criteriaVerified"],false);assert_eq!(entries[0]["scope"]["envelope"]["providerUseAuthorized"],false);assert_eq!(raw["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
 let stored=s.bytes(&finished);let _second_start=driver.run().await;assert_eq!(s.bytes(&finished),stored);assert_eq!(store(&s).read(&subject.id,&subject.spec.owner.pet_id).unwrap().revision,paused.revision);
}
#[test]fn changed_pause_followup_conditions_cannot_bypass_original_disclosure_consent_or_signature(){
 use crate::pet_runtime::received_work::State as WorkState;
 let s=all();let subject=super::work_observation::own(&s,"ai_ollama");let(g,p)=future_states(&s,&subject,Some(vec![WorkState::Paused]));let now=crate::assistant_memory::now();assert_eq!(review(&g,p.clone()).approve(&g,crate::pet_runtime::deterministic_runtime_tests::evidence(&g,now),now).err(),Some(crate::pet_runtime::admission::Denial::DataConsentRequired));
 let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments["states"]=json!(["completed","cancelled"]);assert!(!p.matches(&changed));let q=queue(&s,&g,p);let row=&q.admissions[0];s.a.revoke(&row.spec,row.proof.as_ref().unwrap(),crate::assistant_memory::now()).unwrap();let _=store(&s).mutate(&subject.id,&subject.spec.owner.pet_id,&subject.revision,"pause-after-revocation".into(),Action::Pause,crate::assistant_memory::now()).unwrap();let frozen=s.bytes(&q);assert!(work_disclosure::ready(&s.f.path.join("goals"),&s.f.path.join("authority"),&q,crate::assistant_memory::now()).is_err());assert_eq!(s.bytes(&q),frozen);
}

#[tokio::test]async fn pause_followup_completion_waits_for_catalog_without_resending_an_existing_receipt(){
 use std::{fs::OpenOptions,os::fd::AsRawFd,time::Duration};
 use crate::pet_runtime::received_work::State as WorkState;
 let s=all();let subject=super::work_observation::own(&s,"ai_ollama");let(g,p)=future_states(&s,&subject,Some(vec![WorkState::Paused]));let q=queue(&s,&g,p);
 let subject=store(&s).mutate(&subject.id,&subject.spec.owner.pet_id,&subject.revision,"pause-before-report-catalog-contention".into(),Action::Pause,crate::assistant_memory::now()).unwrap().record;
 let(g,o)=crate::pet_runtime::deterministic_runtime_tests::owned(&s.f,&s.a,&q,&s.main);let p=g.admissions[0].spec.work_disclosure.as_ref().unwrap();let target=Arc::new(Target::capture(&s.config,&s.settings,&p.recipient.pet_id).unwrap());
 let approved=o.review_message(&g,&p.plan_id,&p.step_id,"original-report-before-contention".into(),target,crate::assistant_memory::now()).unwrap().approve_disclosure(&o,&g,&s.config,&s.settings,crate::assistant_memory::now()).unwrap();
 let receipt=o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();let before:Value=serde_json::from_slice(&s.bytes(&g)).unwrap();
 let lock=OpenOptions::new().read(true).write(true).open(s.f.path.join("authority/.catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);
 let release=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(150));drop(lock);});
 let env=Arc::new(Env{config:s.config.clone(),settings:s.settings.clone(),statuses:Mutex::new(vec![])});let driver=Driver::new(Arc::new(o),Arc::new(store(&s)),env).unwrap();let done=driver.run().await;release.join().unwrap();
 assert_eq!(done.unwrap().phase,crate::pet_runtime::lease_controller::dispatcher::Phase::Completed);
 let finished=store(&s).read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(finished.state,State::Completed);let after:Value=serde_json::from_slice(&s.bytes(&finished)).unwrap();assert_eq!(after["body"]["messages"],before["body"]["messages"]);assert_eq!(after["body"]["budgetUsage"],before["body"]["budgetUsage"]);assert_eq!(after["body"]["messages"]["entries"].as_array().unwrap().len(),1);
 assert_eq!(s.a.message_ack(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&receipt.scope).unwrap().unwrap(),receipt);assert_eq!(store(&s).read(&subject.id,&subject.spec.owner.pet_id).unwrap().revision,subject.revision);assert!(s.main.runtime_resources().busy().unwrap().is_empty());
}
