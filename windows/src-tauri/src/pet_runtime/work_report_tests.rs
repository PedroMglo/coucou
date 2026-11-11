//! Real canonical native source, independent reporting budget and delivery.
//! No model, account, external service or host configuration is used.
use super::*;
use crate::pet_runtime::{work_report,work_report_api,received_work::State as WorkState};
fn draft(s:&Setup,subject:&Record,counts:bool)->work_report_api::Draft{
 let source=subject.spec.received_from.as_ref().unwrap().body.source.clone();work_report_api::prepare(&s.f.path.join("goals"),&s.f.path.join("authority"),work_report_api::Input{autonomous:false,pet_id:subject.spec.owner.pet_id.clone(),goal_id:subject.id.clone(),revision:subject.revision.clone(),source:source.reference,expires_at:source.deadline_at,include_attempts:counts},crate::assistant_memory::now()).unwrap()
}
fn reporter(s:&Setup,subject:&Record,steps:Vec<StepInput>)->(Record,Owned){
 let st=store(s);let now=crate::assistant_memory::now();let g=st.create(Spec{received_from:None,restored_from:None,request_id:nonce().unwrap(),owner:subject.spec.owner.clone(),context:Context::Unbound,intent:"Publish only native reviewed work metadata".into(),criteria:vec![crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION.into()]},now).unwrap().record;
 let(binding,c)=crate::pet_runtime::context::prepare(&s.f.path.join("contexts"),&g,&s.main,PreparationMode::Dedicated,"report-context").unwrap();drop(c);let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"report-context".into(),Action::BindContext{binding},now).unwrap().record;
 let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"report-task".into(),Action::ProposeTask{description:"Explicit separate metadata publication".into()},now).unwrap().record;
 let g=st.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"report-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps},now).unwrap().record;assert_eq!(g.schema_version,26);
 let q=crate::pet_runtime::deterministic_runtime_tests::queued(&s.f,&s.a,&g);crate::pet_runtime::deterministic_runtime_tests::owned(&s.f,&s.a,&q,&s.main)
}
fn review_report(s:&Setup,g:&Record,o:&Owned,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<crate::pet_runtime::lease_controller::message_controller::Approved>{
 let p=&g.plans[0];let step=&p.steps[0];let args=work_report::Arguments::parse(&step.operation.as_ref().unwrap().arguments).unwrap();let target=Arc::new(Target::capture(&s.config,&s.settings,&args.to_pet_id).unwrap());let now=crate::assistant_memory::now();o.review_message(g,&p.id,&step.id,"own-report-send".into(),target,now)?.approve(o,g,&s.config,&s.settings,now,human)
}
#[test]fn work_report_seven_native_identities_publish_minimal_observation_with_independent_authority_and_one_debit(){
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  let s=all();let subject=super::work_observation::own(&s,pet);let d=draft(&s,&subject,false);assert!(!d.execution_authorized&&!d.budget_reserved);assert!(d.preview.attempts.is_none());
  let(g,o)=reporter(&s,&subject,d.steps);let frozen=crate::pet_runtime::reconciliation_tests::inventory(&s.f.path);let a=review_report(&s,&g,&o,Arc::new(||true)).unwrap();assert_eq!(crate::pet_runtime::reconciliation_tests::inventory(&s.f.path),frozen);
  let r=o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();let e=r.envelope();assert_eq!(e.schema_version,3);assert_eq!(e.kind,crate::pet_runtime::messages::Kind::WorkEvent);let w=e.work_report.as_ref().unwrap();assert_eq!(w.state,WorkState::Proposed);assert_eq!(w.subject_goal_id,subject.id);assert_eq!(w.subject_pet_id,pet);assert!(w.attempts.is_none());assert_eq!(e.correlation_id,subject.spec.received_from.as_ref().unwrap().body.source.reference.message_id);assert!(!e.provider_use_authorized);
  let bytes=s.bytes(&g);let raw:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(raw["body"]["schemaVersion"],16);assert_eq!(raw["body"]["messages"]["schemaVersion"],3);assert_eq!(raw["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert!(!serde_json::to_string(w).unwrap().contains(&subject.spec.intent));assert!(r.scope.matches(&g));
  let ack=o.send_message(&g,&a,e.deadline_at+1,||Err(Error::Conflict)).unwrap();assert_eq!(ack,r);assert_eq!(s.bytes(&g),bytes);
  let mut legacy:crate::pet_runtime::authority::GrantEnvelope=serde_json::from_slice(&bytes).unwrap();legacy.body.schema_version=15;assert!(!legacy.body.valid());let mut old=r.scope.clone();old.envelope.schema_version=2;assert!(!old.valid());
  for v in 1..=25{let mut old=g.clone();old.schema_version=v;assert!(old.validate().is_err());}
 }
}
#[test]fn work_report_cancelled_subject_uses_fresh_reporting_grant_not_revoked_work_authority(){
 let s=all();let subject=super::work_observation::planned(&s,&super::work_observation::own(&s,"ai_ollama"));let q=crate::pet_runtime::deterministic_runtime_tests::queued(&s.f,&s.a,&subject);
 let subject=crate::pet_runtime::authority_store::mutate_with_authority(&store(&s),&s.f.path.join("authority"),&q.id,&q.spec.owner.pet_id,&q.revision,"cancel-subject".into(),Action::Cancel,crate::assistant_memory::now()).unwrap().record;assert_eq!(subject.state,State::Cancelled);
 let original=s.bytes(&subject);let original_grant:Value=serde_json::from_slice(&original).unwrap();assert_eq!(original_grant["body"]["phase"],"revoked");let d=draft(&s,&subject,false);assert_eq!(d.preview.state,WorkState::Cancelled);
 let(g,o)=reporter(&s,&subject,d.steps);assert_ne!(g.admissions[0].spec.decision_id,subject.admissions[0].spec.decision_id);let a=review_report(&s,&g,&o,Arc::new(||true)).unwrap();let r=o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();assert_eq!(r.envelope().work_report.as_ref().unwrap().state,WorkState::Cancelled);assert_eq!(s.bytes(&subject),original);
}
#[test]fn work_report_rejects_forged_state_unknown_fields_wrong_source_target_subject_and_actor(){
 let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let d=draft(&s,&subject,false);let args=d.steps[0].operation.as_ref().unwrap().arguments.clone();let mut forged=args.clone();forged["state"]=json!("completed");assert!(work_report::Arguments::parse(&forged).is_err());
 let bad=json!({"toPetId":"ai_ollama","kind":"workEvent","text":"I say completed","expiresAt":crate::assistant_memory::now()+10000});assert!(crate::pet_runtime::messages::Arguments::parse(&bad).is_err());
 for kind in ["source","target","revision","actor"]{
  let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let mut d=draft(&s,&subject,false);
  let a=&mut d.steps[0].operation.as_mut().unwrap().arguments;match kind{"source"=>a["source"]["scopeHash"]=json!("0".repeat(64)),"target"=>a["toPetId"]=json!("integration_spotify"),"revision"=>a["subjectRevision"]=json!("0".repeat(32)),_=>{}};
  let mut actor=subject.clone();if kind=="actor"{actor.spec.owner=Owner{pet_id:"agent_codex".into(),provider:"codex".into(),adapter:"codex".into(),model_hint:None};}
  let(g,o)=reporter(&s,&actor,d.steps);let frozen=s.bytes(&g);assert!(review_report(&s,&g,&o,Arc::new(||true)).is_err(),"{kind}");assert_eq!(s.bytes(&g),frozen);assert!(s.budget(&g).is_null());
 }
}
#[test]fn work_report_native_review_holds_no_goal_lock_and_changes_or_revocation_block_new_publication(){
 for cut in ["subject","human","publisher","target","expiry"]{
  let mut s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let d=draft(&s,&subject,false);let(g,o)=reporter(&s,&subject,d.steps);let flag=Arc::new(AtomicBool::new(true));let f=flag.clone();let a=review_report(&s,&g,&o,Arc::new(move||f.load(Ordering::Acquire))).unwrap();let now=crate::assistant_memory::now();
  match cut{"subject"=>{store(&s).mutate(&subject.id,&subject.spec.owner.pet_id,&subject.revision,"pause-during-human-review".into(),Action::Pause,now).unwrap();},"human"=>flag.store(false,Ordering::Release),"publisher"=>s.a.revoke(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),now).unwrap(),"target"=>s.config.pets.get_mut("ai_ollama").unwrap().name="Changed reviewed recipient".into(),_=>{}};
  let before=s.bytes(&g);let at=if cut=="expiry"{a.scope().envelope.deadline_at}else{now};assert!(o.send_message(&g,&a,at,||s.observe()).is_err(),"{cut}");assert_eq!(s.bytes(&g),before);assert!(s.budget(&g).is_null());
 }
}
#[test]fn work_report_final_source_fences_and_lost_ack_use_the_same_canonical_receipt_and_budget(){
 for fault in [crate::pet_runtime::authority_store::Fault::BeforeGrantPublish,crate::pet_runtime::authority_store::Fault::AfterGrantPublish]{
  let mut s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let d=draft(&s,&subject,false);let(g,o)=reporter(&s,&subject,d.steps);let a=review_report(&s,&g,&o,Arc::new(||true)).unwrap();let row=&g.admissions[0];s.a.fault=fault;
  assert!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&a,crate::assistant_memory::now(),||true).is_err());s.a.fault=crate::pet_runtime::authority_store::Fault::None;
  let r=s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&a,crate::assistant_memory::now(),||true).unwrap();let before=s.bytes(&g);assert_eq!(s.a.message_ack(&row.spec,row.proof.as_ref().unwrap(),&r.scope).unwrap().unwrap(),r);assert_eq!(s.bytes(&g),before);assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
 }
}

#[test]fn work_report_subject_data_change_at_each_final_publish_fence_prevents_canonical_delivery(){
 for cut in [1,2]{
  let s=all();let subject=super::work_observation::own(&s,"ai_lmstudio");let d=draft(&s,&subject,false);let(g,o)=reporter(&s,&subject,d.steps);let a=review_report(&s,&g,&o,Arc::new(||true)).unwrap();let row=&g.admissions[0];let path=s.f.path.join("goals").join(format!("{}.json",subject.id));let calls=AtomicUsize::new(0);let before=s.bytes(&g);
  assert!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&a,crate::assistant_memory::now(),||{if calls.fetch_add(1,Ordering::AcqRel)==cut{fs::write(&path,b"subject changed at final publication fence").unwrap();}true}).is_err());assert_eq!(s.bytes(&g),before);assert!(s.budget(&g).is_null());assert!(calls.load(Ordering::Acquire)>cut);assert_eq!(fs::read(&path).unwrap(),b"subject changed at final publication fence");
 }
}
