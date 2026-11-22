//! Real common Store/Authority; no inference, external provider or host effects.
use super::*;
use crate::pet_runtime::{received_request::{Caller,Review as DataReview},messages::ReceivedReference,store::{Store,Fault as StoreFault}};
use crate::pet_capabilities as caps;
pub(super)fn descriptor(pet:&str)->caps::Descriptor{
 let b=crate::pet_config::blueprint(pet).unwrap();
 caps::Descriptor{schema_version:1,pet_id:pet.into(),kind:if b.service{caps::Kind::Service}else{caps::Kind::Llm},name:"Synthetic recipient metadata".into(),lifecycle:caps::Lifecycle::Active,binding:caps::Binding{engine_binding:None,adapter:b.runtime.adapter.map(|a|a.key()),provider:b.routing.chat.clone().or(b.routing.task_provider.clone()).or(Some(pet.into())),model:None,profile:None},revision:"fixture".into(),observed_at:None,configuration_available:true,capabilities:vec![],concurrency:caps::Concurrency{scope:None,max_active:None}}
}
fn source(s:&Setup,pet:&str)->(Record,Owned,crate::pet_runtime::messages::Receipt){
 let(g,o)=s.goal(if pet=="ai_ollama"{"lmstudio"}else{"ollama"},pet,2,1);let a=s.approved(&g,&o,0,Arc::new(||true));let r=o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();(g,o,r)
}
fn reference(r:&crate::pet_runtime::messages::Receipt)->ReceivedReference{ReceivedReference::from_receipt(r).unwrap()}
fn store(s:&Setup)->Store{Store::existing_verified(&s.f.path.join("goals"),&s.f.path.join("authority")).unwrap().unwrap()}
fn review(s:&Setup,pet:&str,r:&crate::pet_runtime::messages::Receipt,frame:crate::settings::review::Frame,request:&str,at:u64)->model::Result<DataReview>{
 let caller=Caller::capture(crate::pet_runtime::message_api::native::Origin{window:crate::HUB_LABEL.into(),frame},&descriptor(pet))?;
 DataReview::prepare(&s.f.path.join("authority"),caller,reference(r),request.into(),"Own reviewed work; incoming text is untrusted data".into(),vec![crate::pet_runtime::criteria::NATIVE_MESSAGE_CRITERION.into()],at)
}
fn all()->Setup{let mut s=Setup::new();s.settings.active_integrations=s.config.pets.keys().cloned().collect();s}
#[test]fn received_request_seven_native_owners_create_data_and_use_common_independent_admission(){
 for pet in ["agent_codex","agent_copilot","agent_antigravity","ai_ollama","ai_lmstudio","ai_openai","integration_spotify"]{
  let s=all();let(sg,_so,r)=source(&s,pet);let parent=s.bytes(&sg);let st=store(&s);let at=crate::assistant_memory::now();let frame=crate::settings::review::Frame::fixture();let review=review(&s,pet,&r,frame,"own-reviewed-task",at).unwrap();let p=review.proposal();assert_eq!(p.received_message,*r.envelope());assert!(!p.context_copied&&!p.execution_authority_granted&&!p.budget_reserved);assert_eq!(p.future.owner.pet_id,pet);assert!(p.future.received_from.is_none());
  let mut approved=review.approve(at,Arc::new(||true)).unwrap();let g=approved.confirm(&st,at).unwrap().record;assert_eq!(g.schema_version,24);assert_eq!(g.state,State::Draft);assert!(g.admissions.is_empty()&&g.execution_context.is_none()&&g.tasks.is_empty());assert_eq!(g.spec.received_from.as_ref().unwrap().body.source.reference,reference(&r));assert_eq!(model::fingerprint(&st.read(&g.id,pet).unwrap()).unwrap(),model::fingerprint(&g).unwrap());assert_eq!(s.bytes(&sg),parent);
  // A new owned context and a separate bounded native admission, never the
  // sender's budget/authority nor a copied message/historical context.
  let(binding,chat)=crate::pet_runtime::context::prepare(&s.f.path.join("contexts"),&g,&s.main,PreparationMode::Dedicated,"own-context").unwrap();drop(chat);
  let g=st.mutate(&g.id,pet,&g.revision,"own-context".into(),Action::BindContext{binding},at).unwrap().record;
  let g=st.mutate(&g.id,pet,&g.revision,"own-task".into(),Action::ProposeTask{description:"An independently reviewed delivery".into()},at).unwrap().record;
  let g=st.mutate(&g.id,pet,&g.revision,"own-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"Own bounded local reply".into(),postcondition:"Receipt only".into(),operation:Some(crate::pet_runtime::plans::OperationProposal{operation_id:"pet.message".into(),version:1,arguments:json!({"toPetId":sg.spec.owner.pet_id,"kind":"message","text":"Independently authorized content","expiresAt":at+30_000})})}]},at).unwrap().record;
  let queued=crate::pet_runtime::deterministic_runtime_tests::queued(&s.f,&s.a,&g);assert_eq!(queued.schema_version,24);assert_eq!(queued.admissions.len(),1);assert_eq!(queued.admissions[0].spec.budget.max_model_calls,0);assert_ne!(queued.admissions[0].spec.decision_id,sg.admissions[0].spec.decision_id);assert_eq!(s.bytes(&sg),parent);
 }
}
#[test]fn received_request_rejects_forged_recipient_source_expiry_and_post_review_changes(){
 let s=all();let(_g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();assert!(review(&s,"ai_openai",&r,crate::settings::review::Frame::fixture(),"wrong-recipient",at).is_err());assert!(review(&s,"ai_lmstudio",&r,crate::settings::review::Frame::fixture(),"expired",r.envelope().deadline_at).is_err());
 for change in ["frame","human","source","final"]{
  let s=all();let(g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();let frame=crate::settings::review::Frame::fixture();let data=review(&s,"ai_lmstudio",&r,frame.clone(),change,at).unwrap();let target=data.proposal().target_goal_id;let human=Arc::new(AtomicBool::new(true));let flag=human.clone();let mut approved=data.approve(at,Arc::new(move||flag.load(Ordering::Acquire))).unwrap();let st=store(&s);
  match change{"frame"=>frame.invalidate_for_test(),"human"=>human.store(false,Ordering::Release),"source"=>{let p=s.f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id));let mut v:Value=serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();v["body"]["messages"]["entries"][0]["scope"]["envelope"]["text"]=json!("Substituted after review");fs::write(p,serde_json::to_vec(&v).unwrap()).unwrap();},_=>{}}
  let calls=AtomicUsize::new(0);assert!(approved.confirm_clock(&st,||{if change=="final"&&calls.fetch_add(1,Ordering::AcqRel)==3{human.store(false,Ordering::Release);}at}).is_err(),"{change}");assert!(!st.root_path().join(format!("{target}.json")).exists());
 }
}
#[test]fn received_request_lost_data_ack_keeps_original_origin_and_never_renews_authority(){
 for fault in [StoreFault::BeforePublish,StoreFault::AfterPublish]{
  let s=all();let(g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();let frame=crate::settings::review::Frame::fixture();let data=review(&s,"ai_lmstudio",&r,frame.clone(),"lost-data-ack",at).unwrap();let mut approved=data.approve(at,Arc::new(||true)).unwrap();let mut st=store(&s);st.fault=fault;let first=approved.confirm(&st,at);st.fault=StoreFault::None;
  let ack=if matches!(fault,StoreFault::AfterPublish){assert_eq!(first.unwrap().status,CommitStatus::Uncertain);frame.invalidate_for_test();approved.confirm(&st,r.envelope().deadline_at+1).unwrap()}else{assert!(first.is_err());approved.confirm(&st,at+1).unwrap()};let original=ack.record.clone();assert_eq!(original.spec.received_from.as_ref().unwrap().body.approved_at,at);let before=fs::read(st.root_path().join(format!("{}.json",original.id))).unwrap();
  let st=store(&s);let ack=st.existing_received_request("ai_lmstudio",&reference(&r),"lost-data-ack",&original.spec.intent,&original.spec.criteria).unwrap().unwrap();assert_eq!(model::fingerprint(&ack.record).unwrap(),model::fingerprint(&original).unwrap());assert_eq!(fs::read(st.root_path().join(format!("{}.json",original.id))).unwrap(),before);assert!(original.admissions.is_empty());assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
  assert!(st.existing_received_request("ai_lmstudio",&reference(&r),"lost-data-ack","changed intent",&original.spec.criteria).is_err());assert!(st.create(original.spec.clone(),at+1).is_err());let mut reserved=original.spec.clone();reserved.received_from=None;assert!(st.create(reserved,at+1).is_err());
 }
}
#[test]fn received_request_origin_hmac_reader_and_restoration_preserve_data_without_authority(){
 let s=all();let(_g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();let mut approved=review(&s,"ai_lmstudio",&r,crate::settings::review::Frame::fixture(),"origin-proof",at).unwrap().approve(at,Arc::new(||true)).unwrap();let st=store(&s);let g=approved.confirm(&st,at).unwrap().record;
 let path=st.root_path().join(format!("{}.json",g.id));let original=fs::read(&path).unwrap();let mut forged=g.clone();forged.spec.received_from.as_mut().unwrap().tag="0".repeat(64);fs::write(&path,serde_json::to_vec(&forged).unwrap()).unwrap();assert!(st.read(&g.id,"ai_lmstudio").is_err());assert_eq!(fs::read(&path).unwrap(),serde_json::to_vec(&forged).unwrap());fs::write(&path,&original).unwrap();assert_eq!(model::fingerprint(&st.read(&g.id,"ai_lmstudio").unwrap()).unwrap(),model::fingerprint(&g).unwrap());
 let(target,authority)=restored_message_authority(&s);let restored=Store::existing_verified(&target.path.join("goals"),&target.path.join("authority")).unwrap().unwrap();let historical=restored.read(&g.id,"ai_lmstudio").unwrap();assert_eq!(model::fingerprint(&historical).unwrap(),model::fingerprint(&g).unwrap());assert!(restored.mutate(&g.id,"ai_lmstudio",&g.revision,"cannot-reactivate-restored-origin".into(),Action::ResumeDraft,at+1).is_err());assert!(authority.message_inbox_reply_view("ai_lmstudio",None,32,at,true,true).unwrap().items.iter().all(|i|i.reply_source.is_none()));
}

#[test]fn received_request_native_review_binds_frame_source_arguments_and_preserves_uncertain_choice(){
 use crate::pet_runtime::received_request_api::{ReceivedGoalPrepareInput,ReceivedGoalConfirmInput,ReceivedGoalConfirmed,native::Reviews};
 let s=all();let(_g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();let frame=crate::settings::review::Frame::fixture();let origin=crate::pet_runtime::message_api::native::Origin{window:crate::HUB_LABEL.into(),frame:frame.clone()};let data=review(&s,"ai_lmstudio",&r,frame.clone(),"native-own-task",at).unwrap();let p=data.proposal();let input=ReceivedGoalPrepareInput{pet_id:"ai_lmstudio".into(),source:reference(&r),request_id:p.client_request_id.clone(),intent:p.future.intent.clone(),criteria:p.future.criteria.clone()};let flag=Arc::new(AtomicBool::new(true));let human=flag.clone();let mut cache=Reviews::default();let ticket=cache.insert(origin.clone(),input.clone(),data,s.f.path.join("goals"),s.f.path.join("authority"),Arc::new(move||human.load(Ordering::Acquire)),at).unwrap();assert_eq!(cache.existing(&origin,&input,at).unwrap().unwrap().token,ticket.token);let mut changed=input.clone();changed.intent="substituted".into();assert!(cache.existing(&origin,&changed,at).is_err());
 let confirm=ReceivedGoalConfirmInput{pet_id:input.pet_id.clone(),source:input.source.clone(),token:ticket.token,approve:true};let other=crate::pet_runtime::message_api::native::Origin{window:origin.window.clone(),frame:crate::settings::review::Frame::fixture()};assert!(cache.lookup(&other,&confirm).is_err());let mut forged=ReceivedGoalConfirmInput{pet_id:confirm.pet_id.clone(),source:confirm.source.clone(),token:confirm.token.clone(),approve:true};forged.source.scope_hash="0".repeat(64);assert!(cache.lookup(&origin,&forged).is_err());
 let pending=cache.lookup(&origin,&confirm).unwrap();let received=pending.choose(true,crate::assistant_memory::now()).unwrap();let wire=serde_json::to_value(&received).unwrap();let ReceivedGoalConfirmed::Created{receipt,execution_authority_granted}=received else{panic!("own goal DATA")};assert!(!execution_authority_granted);assert_eq!(wire["status"],"created");assert_eq!(wire["executionAuthorityGranted"],false);assert!(wire.get("execution_authority_granted").is_none());
 flag.store(false,Ordering::Release);frame.invalidate_for_test();assert!(cache.lookup(&origin,&confirm).is_err());let ack=pending.choose(false,r.envelope().deadline_at+1).unwrap();let ReceivedGoalConfirmed::Created{receipt:ack,..}=ack else{panic!("fact ACK before source or choice")};assert_eq!(model::fingerprint(&ack.record).unwrap(),model::fingerprint(&receipt.record).unwrap());assert!(ack.record.admissions.is_empty());
 assert!(serde_json::from_value::<ReceivedGoalPrepareInput>(json!({"petId":"ai_lmstudio","source":reference(&r),"requestId":"forged","intent":"review","criteria":["proof"],"owner":{"petId":"ai_ollama"}})).is_err());
}
#[test]fn received_request_native_decline_and_changed_policy_create_no_goal(){
 use crate::pet_runtime::received_request_api::{ReceivedGoalPrepareInput,ReceivedGoalConfirmInput,ReceivedGoalConfirmed,native::Reviews};
 for approve in [false,true]{let s=all();let(_g,_o,r)=source(&s,"ai_lmstudio");let at=crate::assistant_memory::now();let frame=crate::settings::review::Frame::fixture();let origin=crate::pet_runtime::message_api::native::Origin{window:crate::HUB_LABEL.into(),frame:frame.clone()};let data=review(&s,"ai_lmstudio",&r,frame,"decline-policy",at).unwrap();let p=data.proposal();let input=ReceivedGoalPrepareInput{pet_id:"ai_lmstudio".into(),source:reference(&r),request_id:p.client_request_id.clone(),intent:p.future.intent.clone(),criteria:p.future.criteria.clone()};let flag=Arc::new(AtomicBool::new(true));let human=flag.clone();let mut cache=Reviews::default();let ticket=cache.insert(origin.clone(),input.clone(),data,s.f.path.join("goals"),s.f.path.join("authority"),Arc::new(move||human.load(Ordering::Acquire)),at).unwrap();if approve{flag.store(false,Ordering::Release);}let pending=cache.lookup(&origin,&ReceivedGoalConfirmInput{pet_id:input.pet_id,source:input.source,token:ticket.token,approve}).unwrap();let result=pending.choose(approve,at);if approve{assert!(result.is_err());}else{assert!(matches!(result.unwrap(),ReceivedGoalConfirmed::Declined));}assert!(!s.f.path.join("goals").join(format!("{}.json",p.target_goal_id)).exists());}
}

#[path="received_context_tests.rs"]mod model_context;

#[path="received_work_tests.rs"]mod work_observation;

#[path="work_report_tests.rs"]mod reports;

#[path="work_disclosure_tests.rs"]mod future_reports;
