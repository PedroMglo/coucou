use super::*;
use crate::pet_runtime::{admission::{Review as AdmissionReview,Evidence},admission_tests::{self,Fixture},authority::{Authority,Failure},authority_store::Fault,lease_controller::{Controller,Owned,Validity,Validation},messages::{Arguments,Target,Kind},model::*,store::nonce};
use crate::{chat::Chat,settings::Settings};
use serde_json::{json,Value};
use std::{fs,sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}}};
struct Setup{f:Fixture,a:Authority,main:Chat,config:crate::pet_config::Configuration,settings:Settings}
impl Setup{
 fn new()->Self{Self::with_fixture(Fixture::new())}
 fn with_fixture(f:Fixture)->Self{let a=Authority::ensure(&f.path.join("authority")).unwrap();let main=Chat::persistent(f.path.join("common-main.json"));let mut config=crate::pet_config::defaults();config.pets.get_mut("ai_lmstudio").unwrap().provider="lmstudio".into();let mut settings=Settings::default();settings.active_integrations=vec!["ai_ollama".into(),"ai_lmstudio".into(),"integration_spotify".into()];Self{f,a,main,config,settings}}
 fn evidence(&self,g:&Record,at:u64)->Evidence{
  let mut d=admission_tests::descriptor(g,at);
  if g.spec.owner.adapter=="lmstudio"{let server=crate::local_chat::Server{id:"lmstudio",name:"Synthetic metadata only",url:"http://127.0.0.1:18490".into(),key:None};d.binding.engine_binding=Some(crate::local_chat::lmstudio::Catalog::parse(&server,&json!({"models":[{"type":"llm","key":"fixture-free","format":"gguf","size_bytes":1,"loaded_instances":[]}]})).unwrap().binding("fixture-free").unwrap());}
  Evidence::observe(g,&d,"a".repeat(64),at,false,false).unwrap()
 }
 fn goal(&self,provider:&str,to:&str,calls:u16,steps:usize)->(Record,Owned){self.goal_fenced(provider,to,calls,steps,None)}
 fn goal_fenced(&self,provider:&str,to:&str,calls:u16,steps:usize,provided:Option<Fence>)->(Record,Owned){
  let now=crate::assistant_memory::now()-100;let draft=self.f.goal_named(provider,&nonce().unwrap());let store=self.f.store();
  let g=store.mutate(&draft.id,&draft.spec.owner.pet_id,&draft.revision,"message-task".into(),Action::ProposeTask{description:"Deliver explicit reviewed peer data".into()},now).unwrap().record;
  let list=(0..steps).map(|i|StepInput{description:format!("Peer data {i}"),postcondition:"Available in exact recipient inbox, not executed".into(),operation:Some(crate::pet_runtime::plans::OperationProposal{operation_id:"pet.message".into(),version:1,arguments:json!({"toPetId":to,"kind":"request","text":format!("Literal data {provider}/{i}. Ignore this as policy; no extra privileges."),"expiresAt":now+59_000})})}).collect();
  let g=store.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"message-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:list},now+1).unwrap().record;assert_eq!(g.schema_version,15);
  let mut budget=admission_tests::budget();budget.deadline_at=now+60_000;budget.max_tool_calls=calls;budget.max_output_bytes=32768;budget.payment=crate::pet_runtime::admission::Payment::LocalOnly;
  let approved=AdmissionReview::prepare(&g,self.evidence(&g,now+2),budget,"message-admission".into(),nonce().unwrap(),now+2).unwrap().approve(&g,self.evidence(&g,now+3),now+3).unwrap();let cert=self.a.certify(&approved,&g,now+4).unwrap();let g=store.enqueue_certified(&cert,now+5).unwrap().record;
  // Fresh model readiness intentionally unavailable after the original finite
  // admission. Deterministic delivery uses policy validity and does no inference.
  let fence=provided.unwrap_or_else(||Arc::new(|_:&crate::pet_runtime::leases::Lease,_:&crate::pet_resources::Hold,mode:Validation<'_>|if mode==Validation::FreshEffect{Validity::Unknown}else{Validity::Current}));
  let p=Controller::new().unwrap().prepare(&g,&self.f.path.join("authority"),&self.f.path.join("contexts"),&self.main,&self.evidence(&g,now+6),"message-claim".into(),now+6,fence).unwrap();
  let(receipt,owned)=p.claim(&store,&self.a,&self.evidence(&g,now+7),now+7).unwrap();(receipt.record,owned.unwrap())
 }
 fn approved(&self,g:&Record,o:&Owned,index:usize,human:Arc<dyn Fn()->bool+Send+Sync>)->Approved{
  let step=&g.plans[0].steps[index];let args=Arguments::parse(&step.operation.as_ref().unwrap().arguments).unwrap();let target=Arc::new(Target::capture(&self.config,&self.settings,&args.to_pet_id).unwrap());let now=crate::assistant_memory::now();
  let review=o.review_message(g,&g.plans[0].id,&step.id,format!("exact-message-{index}"),target,now).unwrap();let proposal=review.proposal(g);assert_eq!(proposal.envelope.origin.pet_id,g.spec.owner.pet_id);assert!(!proposal.destination_execution_authorized);
  review.approve(o,g,&self.config,&self.settings,now,human).unwrap()
 }
 fn observe(&self)->model::Result<(crate::pet_config::Configuration,Settings)>{Ok((self.config.clone(),self.settings.clone()))}
 fn bytes(&self,g:&Record)->Vec<u8>{fs::read(self.f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap()}
 fn budget(&self,g:&Record)->Value{serde_json::from_slice::<Value>(&self.bytes(g)).unwrap()["body"]["budgetUsage"].clone()}
}
#[test]fn mesh_direct_two_pets_share_the_existing_authority_without_coordinator_or_model_calls(){
 let s=Setup::new();let(a,owner_a)=s.goal("ollama","ai_lmstudio",2,1);let(b,owner_b)=s.goal("lmstudio","ai_ollama",2,1);
 assert!(!owner_a.current(crate::assistant_memory::now()),"model readiness is absent");assert!(owner_a.policy_current(crate::assistant_memory::now()));
 let approved_a=s.approved(&a,&owner_a,0,Arc::new(||true));let approved_b=s.approved(&b,&owner_b,0,Arc::new(||true));
 let first=owner_a.send_message(&a,&approved_a,crate::assistant_memory::now(),||s.observe()).unwrap();let second=owner_b.send_message(&b,&approved_b,crate::assistant_memory::now(),||s.observe()).unwrap();assert_ne!(first.envelope().message_id,second.envelope().message_id);
 for(pet,expected,g)in[("ai_lmstudio",&first,&a),("ai_ollama",&second,&b)]{let inbox=s.a.message_inbox(pet,None,32,crate::assistant_memory::now()).unwrap();assert!(inbox.scan_complete);assert_eq!(inbox.items.len(),1);assert_eq!(&inbox.items[0].receipt,expected);assert!(!inbox.execution_authorized);assert!(!inbox.items[0].receipt.envelope().provider_use_authorized);let usage=s.budget(g);assert_eq!(usage["attempts"].as_array().unwrap().len(),1);assert_eq!(usage["attempts"][0]["kind"],"tool");assert_eq!(usage["attempts"][0]["inputHash"],model::fingerprint(&expected.scope).unwrap());}
 assert!(s.main.history("ollama").is_empty()&&s.main.history("lmstudio").is_empty());let bytes=s.bytes(&a);assert_eq!(owner_a.send_message(&a,&approved_a,crate::assistant_memory::now(),||panic!("receipt before another projection")).unwrap(),first);assert_eq!(s.bytes(&a),bytes);
 drop(owner_a);drop(owner_b);let reopened=Authority::existing(&s.f.path.join("authority")).unwrap();assert_eq!(reopened.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap().items[0].receipt,first);
}
#[test]fn mesh_direct_requires_original_owner_human_source_and_exact_recipient_revision(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let allowed=Arc::new(AtomicBool::new(true));let flag=allowed.clone();let approved=s.approved(&g,&o,0,Arc::new(move||flag.load(Ordering::Acquire)));let before=s.bytes(&g);
 allowed.store(false,Ordering::Release);assert!(o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).is_err());assert_eq!(s.bytes(&g),before);allowed.store(true,Ordering::Release);
 let mut changed=s.config.clone();changed.pets.get_mut("ai_lmstudio").unwrap().name="Different native recipient revision".into();assert!(o.send_message(&g,&approved,crate::assistant_memory::now(),||Ok((changed.clone(),s.settings.clone()))).is_err());assert_eq!(s.bytes(&g),before);
 // Target retirement is sticky even after exact configuration rollback.
 assert!(o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).is_err());assert!(s.budget(&g).is_null());
}
#[test]fn mesh_direct_stop_preserves_delivered_data_and_never_replays_or_debits_again(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,2);let first=s.approved(&g,&o,0,Arc::new(||true));let receipt=o.send_message(&g,&first,crate::assistant_memory::now(),||s.observe()).unwrap();let second=s.approved(&g,&o,1,Arc::new(||true));let row=&g.admissions[0];s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,crate::assistant_memory::now()).unwrap();let before=s.bytes(&g);
 assert!(o.send_message(&g,&second,crate::assistant_memory::now(),||s.observe()).is_err());assert_eq!(o.send_message(&g,&first,crate::assistant_memory::now(),||panic!("receipt is data only")).unwrap(),receipt);assert_eq!(s.bytes(&g),before);assert_eq!(serde_json::from_slice::<Value>(&before).unwrap()["body"]["schemaVersion"],9);
}
#[test]fn mesh_direct_shared_tool_budget_cannot_be_bypassed_with_another_message_id(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",1,2);let a=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();let b=s.approved(&g,&o,1,Arc::new(||true));let before=s.bytes(&g);assert!(o.send_message(&g,&b,crate::assistant_memory::now(),||s.observe()).is_err());assert_eq!(s.bytes(&g),before);assert_eq!(s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap().items.len(),1);
}
#[test]fn mesh_direct_missing_ack_after_atomic_publication_reconciles_one_receipt_and_charge(){
 let mut s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));let row=&g.admissions[0];s.a.fault=Fault::AfterGrantPublish;
 assert_eq!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now(),||true).err(),Some(Failure::Uncertain));s.a.fault=Fault::None;let before=s.bytes(&g);let receipt=o.send_message(&g,&approved,crate::assistant_memory::now(),||panic!("no renewed authority after lost ACK")).unwrap();assert_eq!(receipt.envelope().message_id,approved.scope().envelope.message_id);assert_eq!(s.bytes(&g),before);assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
}
#[test]fn mesh_direct_unpublished_staging_is_not_an_inbox_or_a_fresh_approval(){
 let mut s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));let row=&g.admissions[0];let before=s.bytes(&g);s.a.fault=Fault::BeforeGrantPublish;
 assert_eq!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now(),||true).err(),Some(Failure::Unavailable));s.a.fault=Fault::None;assert_eq!(s.bytes(&g),before);let inbox=s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap();assert!(inbox.items.is_empty());assert_eq!(inbox.pending_artifacts,1);assert!(!inbox.scan_complete);
 let other=s.approved(&g,&o,0,Arc::new(||true));assert_eq!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&other,crate::assistant_memory::now(),||true).err(),Some(Failure::RecoveryRequired));let receipt=o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();assert_eq!(receipt.envelope().message_id,approved.scope().envelope.message_id);assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
}
#[test]fn mesh_direct_human_revocation_at_final_rename_boundary_never_publishes(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));let row=&g.admissions[0];let before=s.bytes(&g);let calls=AtomicUsize::new(0);
 assert_eq!(s.a.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now(),||calls.fetch_add(1,Ordering::AcqRel)<2).err(),Some(Failure::Revoked));assert_eq!(s.bytes(&g),before);assert!(s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap().items.is_empty());
}
#[test]fn mesh_direct_forged_origin_malformed_body_and_corrupt_canonical_records_are_refused(){
 let args=json!({"toPetId":"ai_lmstudio","kind":"message","text":"literal","expiresAt":1});assert!(Arguments::parse(&args).is_ok());for (key,value)in [("senderPetId",json!("agent_codex")),("text",json!("x".repeat(2049))),("kind",json!("delegation")),("providerUseAuthorized",json!(true))]{let mut bad=args.clone();bad[key]=value;assert!(Arguments::parse(&bad).is_err(),"{key}");}
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();let mut bytes:Value=serde_json::from_slice(&s.bytes(&g)).unwrap();bytes["body"]["messages"]["entries"][0]["scope"]["envelope"]["origin"]["petId"]=json!("agent_codex");let raw=serde_json::to_vec(&bytes).unwrap();let path=s.f.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id));fs::write(&path,&raw).unwrap();let inbox=s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap();assert!(inbox.items.is_empty()&&!inbox.scan_complete);assert_eq!(inbox.unavailable,1);assert_eq!(fs::read(path).unwrap(),raw);
}

#[test]fn mesh_direct_delivery_is_known_in_common_reconciliation_history_and_native_drain(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();let before=s.bytes(&g);
 let snapshot=s.a.reconciliation(&g,crate::assistant_memory::now()).unwrap();assert!(snapshot.scan_complete&&snapshot.source_consistent);assert_eq!(snapshot.accounting.unwrap().unknown_attempts,0);assert!(snapshot.attempts[0].outcome_known);assert_eq!(snapshot.attempts[0].evidence[0].kind,crate::pet_runtime::reconciliation::EvidenceKind::PeerMessage);assert!(!snapshot.goal_complete&&!snapshot.criteria_verified);
 let history=s.a.history(&g,crate::assistant_memory::now()).unwrap();assert!(history.source_consistent&&history.scan_complete);assert_eq!(history.authenticated_subtotal.unknown_tool_results,0);assert_eq!(s.bytes(&g),before);
 let drain=o.drain(crate::pet_runtime::leases::DrainOutcome::Confirmed).unwrap();let closed=s.f.store().drain_owned(&s.a,&drain,crate::assistant_memory::now()).unwrap().record;assert_eq!(closed.state,State::RecoveryRequired);assert_eq!(closed.schema_version,15);
 let after:Value=serde_json::from_slice(&s.bytes(&g)).unwrap();let prior:Value=serde_json::from_slice(&before).unwrap();assert_eq!(after["body"]["schemaVersion"],9);assert_eq!(after["body"]["messages"],prior["body"]["messages"]);assert_eq!(after["body"]["budgetUsage"],prior["body"]["budgetUsage"]);assert_eq!(s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap().items.len(),1);
}

fn pending_intent(s:&Setup,g:&Record,o:&Owned)->Approved{
 let approved=s.approved(g,o,0,Arc::new(||true));let mut authority=Authority::existing(&s.f.path.join("authority")).unwrap();authority.fault=Fault::BeforeGrantPublish;let row=&g.admissions[0];
 assert_eq!(authority.deliver_message(&g,&row.spec,row.proof.as_ref().unwrap(),o.lease(),&approved,crate::assistant_memory::now(),||true).err(),Some(Failure::Unavailable));approved
}
fn retire_review(s:&Setup,g:&Record,message:&str,now:u64)->crate::pet_runtime::message_retirement::Review{
 let a=Authority::existing(&s.f.path.join("authority")).unwrap();match a.prepare_message_retirement(g.clone(),&g.admissions[0].spec.decision_id,message,nonce().unwrap(),now).unwrap(){crate::pet_runtime::authority_store::message_retirement::Prepared::Review(r)=>r,_=>panic!("new exact reduction review required")}
}
#[test]fn mesh_direct_retirement_preserves_budget_and_prevents_old_approval_reactivation(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let now=crate::assistant_memory::now();let review=retire_review(&s,&g,&original.scope().envelope.message_id,now);let proposal=review.proposal().unwrap();assert!(proposal.authority_reduction_only&&!proposal.budget_refund&&!proposal.delivery_authorized);assert_eq!(proposal.artifacts.len(),1);let budget=s.budget(&g);
 let mut approved=review.approve(now,Arc::new(||true)).unwrap();let receipt=approved.confirm(crate::assistant_memory::now()).unwrap();assert!(receipt.cleanup_complete&&!receipt.message_delivered&&!receipt.budget_refund&&!receipt.execution_authorized);assert_eq!(s.budget(&g),budget);
 let inbox=s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap();assert!(inbox.items.is_empty()&&inbox.scan_complete);assert_eq!(inbox.pending_artifacts,0);assert!(o.send_message(&g,&original,crate::assistant_memory::now(),||s.observe()).is_err());
 // A fresh exact review with a new request may proceed, using only the old
 // remaining allowance. The retired original id/request cannot be reused.
 let target=Arc::new(Target::capture(&s.config,&s.settings,"ai_lmstudio").unwrap());let review=o.review_message(&g,&g.plans[0].id,&g.plans[0].steps[0].id,"new-exact-human-choice".into(),target,crate::assistant_memory::now()).unwrap();let fresh=review.approve(&o,&g,&s.config,&s.settings,crate::assistant_memory::now(),Arc::new(||true)).unwrap();o.send_message(&g,&fresh,crate::assistant_memory::now(),||s.observe()).unwrap();assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
}
#[test]fn mesh_direct_retirement_after_signed_source_stop_does_not_renew_lease_or_deadline(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let row=&g.admissions[0];s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,crate::assistant_memory::now()).unwrap();let before:Value=serde_json::from_slice(&s.bytes(&g)).unwrap();
 let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();approved.confirm(crate::assistant_memory::now()).unwrap();let after:Value=serde_json::from_slice(&s.bytes(&g)).unwrap();for field in ["lease","runtimeOwner","phase","stopReason","budgetUsage"]{assert_eq!(before["body"].get(field),after["body"].get(field),"{field}");}assert!(o.send_message(&g,&original,crate::assistant_memory::now(),||s.observe()).is_err());
}
#[test]fn mesh_direct_retirement_exact_source_stamps_and_human_context_are_required(){
 for mode in ["human","artifact","budget"]{
  let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let artifact=review.files[0].clone();let flag=Arc::new(AtomicBool::new(true));let human=flag.clone();let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(move||human.load(Ordering::Acquire))).unwrap();
  match mode{"human"=>flag.store(false,Ordering::Release),"artifact"=>{let path=s.f.path.join("authority").join(&artifact.name);let bytes=fs::read(&path).unwrap();fs::write(&path,bytes).unwrap();},_=>{let row=&g.admissions[0];s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Pause,crate::assistant_memory::now()).unwrap();}}
  let before=s.bytes(&g);assert!(approved.confirm(crate::assistant_memory::now()).is_err(),"{mode}");assert_eq!(s.bytes(&g),before);assert!(s.f.path.join("authority").join(artifact.name).is_file());
 }
}
#[test]fn mesh_direct_retirement_ack_and_cleanup_recover_without_a_new_human_authority(){
 for fault in [Fault::BeforeGrantPublish,Fault::AfterGrantPublish]{
  let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let flag=Arc::new(AtomicBool::new(true));let human=flag.clone();let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(move||human.load(Ordering::Acquire))).unwrap();approved.review.authority.fault=fault;
  assert!(approved.confirm(crate::assistant_memory::now()).is_err());approved.review.authority.fault=Fault::None;
  // After publication the durable exact reduction is ACK/cleanup only. Before
  // publication the original still-valid review is required; it is not renewed.
  if matches!(fault,Fault::AfterGrantPublish){flag.store(false,Ordering::Release);}
  let r=approved.confirm(crate::assistant_memory::now()).unwrap();assert!(r.cleanup_complete);assert!(s.budget(&g).is_null());let inbox=s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap();assert!(inbox.items.is_empty()&&inbox.scan_complete);assert_eq!(inbox.pending_artifacts,0);let bytes=s.bytes(&g);approved.confirm(crate::assistant_memory::now()).unwrap();assert_eq!(s.bytes(&g),bytes);
 }
}
#[test]fn mesh_direct_retirement_unpublished_choice_can_be_reviewed_again_after_original_frame_loss(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let mut lost=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();lost.review.authority.fault=Fault::BeforeGrantPublish;assert!(lost.confirm(crate::assistant_memory::now()).is_err());drop(lost);
 let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());assert_eq!(review.files.len(),2,"both unpublished delivery and reduction images are reviewed");let mut fresh=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();fresh.confirm(crate::assistant_memory::now()).unwrap();assert!(s.a.message_inbox("ai_lmstudio",None,32,crate::assistant_memory::now()).unwrap().scan_complete);assert!(s.budget(&g).is_null());
}

#[test]fn mesh_direct_native_review_cache_binds_original_frame_and_exact_selection(){
 use crate::pet_runtime::{message_api::native::{Reviews,Origin,Selection},step_api::{PrepareInput,ConfirmInput}};
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let now=crate::assistant_memory::now();let input=PrepareInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),lease_id:o.lease().id.clone(),revision:g.revision.clone(),plan_id:g.plans[0].id.clone(),step_id:g.plans[0].steps[0].id.clone(),request_id:"native-exact-choice".into()};
 let frame=crate::settings::review::Frame::fixture();let origin=Origin{window:"native-fixture".into(),frame:frame.clone()};let mut c=Reviews::default();let review=o.review_message(&g,&input.plan_id,&input.step_id,input.request_id.clone(),Arc::new(Target::capture(&s.config,&s.settings,"ai_lmstudio").unwrap()),now).unwrap();let t=c.insert(origin.clone(),Selection::from(&input),&g,review,now).unwrap();assert_eq!(c.existing(&origin,&Selection::from(&input),now).unwrap().unwrap().token,t.token);
 let mut forged=input;forged.step_id="another-step".into();assert!(c.existing(&origin,&Selection::from(&forged),now).is_err());
 let confirm=ConfirmInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),lease_id:o.lease().id.clone(),revision:g.revision.clone(),token:t.token,approve:true};let other=Origin{window:origin.window.clone(),frame:crate::settings::review::Frame::fixture()};assert!(c.lookup(&other,&confirm).is_err());frame.invalidate_for_test();assert!(c.lookup(&origin,&confirm).is_err());assert!(s.budget(&g).is_null());
}
#[test]fn mesh_direct_native_confirmation_recovers_receipt_before_owner_model_or_decline(){
 use crate::pet_runtime::{message_api::native::{Reviews,Origin,Selection,ConfirmResult},step_api::{PrepareInput,ConfirmInput}};
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let now=crate::assistant_memory::now();let o=Arc::new(o);let input=PrepareInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),lease_id:o.lease().id.clone(),revision:g.revision.clone(),plan_id:g.plans[0].id.clone(),step_id:g.plans[0].steps[0].id.clone(),request_id:"native-send".into()};
 let origin=Origin{window:"native-fixture".into(),frame:crate::settings::review::Frame::fixture()};let review=o.review_message(&g,&input.plan_id,&input.step_id,input.request_id.clone(),Arc::new(Target::capture(&s.config,&s.settings,"ai_lmstudio").unwrap()),now).unwrap();let mut c=Reviews::default();let t=c.insert(origin.clone(),Selection::from(&input),&g,review,now).unwrap();let confirm=ConfirmInput{pet_id:input.pet_id,goal_id:input.goal_id,lease_id:input.lease_id,revision:input.revision,token:t.token,approve:true};let pending=c.lookup(&origin,&confirm).unwrap();
 let result=pending.choose(true,&g,||Ok(o.clone()),||s.observe(),||Ok(None),now).unwrap();let ConfirmResult::Received{receipt}=result else{panic!("received local data")};let before=s.bytes(&g);let row=&g.admissions[0];s.a.request_stop(&row.spec,row.proof.as_ref().unwrap(),crate::pet_runtime::leases::StopReason::Cancel,now+1).unwrap();let stopped=s.bytes(&g);
 let result=pending.choose(false,&g,||panic!("late ACK cannot require an owner"),||panic!("late ACK cannot resolve inference"),||s.a.message_ack(&row.spec,row.proof.as_ref().unwrap(),&receipt.scope).map_err(Failure::control_error),now+2).unwrap();assert!(matches!(result,ConfirmResult::Received{..}));assert_ne!(before,stopped);assert_eq!(s.bytes(&g),stopped);assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
}
#[test]fn mesh_direct_pending_intent_must_be_retired_before_recovery_can_freeze_history(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);assert_eq!(s.a.recovery_seed(&g,crate::assistant_memory::now()).err(),Some(Failure::RecoveryRequired));let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();approved.confirm(crate::assistant_memory::now()).unwrap();assert!(!matches!(s.a.recovery_seed(&g,crate::assistant_memory::now()),Err(Failure::RecoveryRequired)));assert!(s.budget(&g).is_null());
}

#[test]fn mesh_direct_retirement_ack_preserves_replaced_identical_bytes_instead_of_deleting_another_inode(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let artifact=review.files[0].clone();let mut approved=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();approved.review.authority.fault=Fault::AfterGrantPublish;assert!(approved.confirm(crate::assistant_memory::now()).is_err());approved.review.authority.fault=Fault::None;
 let path=s.f.path.join("authority").join(&artifact.name);let bytes=fs::read(&path).unwrap();let replace=s.f.path.join("authority").join("replacement-fixture");fs::write(&replace,&bytes).unwrap();use std::os::unix::fs::PermissionsExt;fs::set_permissions(&replace,fs::Permissions::from_mode(0o600)).unwrap();fs::rename(&replace,&path).unwrap();let before=s.bytes(&g);assert!(approved.confirm(crate::assistant_memory::now()).is_err());assert_eq!(fs::read(&path).unwrap(),bytes);assert_eq!(s.bytes(&g),before);assert!(s.budget(&g).is_null());assert_eq!(s.a.pending_messages(&g,&g.admissions[0].spec.decision_id).unwrap().entries.len(),1);
}
#[test]fn mesh_direct_native_retirement_review_binds_frame_decision_and_original_goal_revision(){
 use crate::pet_runtime::{message_retirement_api::{native::{Reviews,Confirmed},MessageRetireInput,MessageRetireConfirmInput},message_api::native::Origin};
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=pending_intent(&s,&g,&o);let id=original.scope().envelope.message_id.clone();let inventory=s.a.pending_messages(&g,&g.admissions[0].spec.decision_id).unwrap();assert_eq!(inventory.entries[0].message.message_id,id);assert!(!inventory.entries[0].retirement_recorded);
 let now=crate::assistant_memory::now();let input=MessageRetireInput{pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),decision_id:g.admissions[0].spec.decision_id.clone(),message_id:id,request_id:"native-retire-choice".into()};let review=match Authority::existing(&s.f.path.join("authority")).unwrap().prepare_message_retirement(g.clone(),&input.decision_id,&input.message_id,input.request_id.clone(),now).unwrap(){crate::pet_runtime::authority_store::message_retirement::Prepared::Review(r)=>r,_=>panic!("review")};
 let frame=crate::settings::review::Frame::fixture();let origin=Origin{window:"native-retirement".into(),frame:frame.clone()};let mut cache=Reviews::default();let t=cache.insert(origin.clone(),&input,review,now).unwrap();let confirm=MessageRetireConfirmInput{pet_id:input.pet_id,goal_id:input.goal_id,revision:input.revision,decision_id:input.decision_id,token:t.token,approve:true};let other=Origin{window:origin.window.clone(),frame:crate::settings::review::Frame::fixture()};assert!(cache.lookup(&other,&confirm).is_err());
 let pending=cache.lookup(&origin,&confirm).unwrap();let mut changed=g.clone();changed.revision=nonce().unwrap();assert!(pending.choose(&changed,true,now).is_err());assert!(s.a.message_retirement_at(&g,&confirm.decision_id,&input.message_id).unwrap().is_none());let result=pending.choose(&g,true,now).unwrap();assert!(matches!(result,Confirmed::Retired{..}));let bytes=s.bytes(&g);assert!(matches!(pending.choose(&g,false,now+500_000).unwrap(),Confirmed::Retired{..}));assert_eq!(s.bytes(&g),bytes);assert!(s.a.pending_messages(&g,&confirm.decision_id).unwrap().entries.is_empty());assert!(s.budget(&g).is_null());frame.invalidate_for_test();assert!(cache.lookup(&origin,&confirm).is_err());
}

#[test]fn mesh_direct_native_verified_goal_fence_keeps_source_locked_without_recursive_authority_reads(){
 let s=Setup::new();let path=s.f.store().root_path().to_owned();let authority=s.f.path.join("authority");let commits=Arc::new(AtomicUsize::new(0));let seen=commits.clone();let fence:Fence=Arc::new(move|lease,_,mode|{
  let store=crate::pet_runtime::store::Store::existing_verified(&path,&authority).unwrap().unwrap();
  if let Validation::Commit(goal)=mode{
   assert_eq!(goal.id,lease.goal_id);assert_eq!(goal.spec.owner.pet_id,"ai_ollama");assert_eq!(store.mutate(&goal.id,"ai_ollama",&goal.revision,"cannot-change-reviewed-goal".into(),Action::Pause,crate::assistant_memory::now()).err(),Some(model::Error::Busy));seen.fetch_add(1,Ordering::AcqRel);return Validity::Current;
  }
  match store.read(&lease.goal_id,"ai_ollama"){Ok(_)=>Validity::Current,Err(_)=>Validity::Unknown}
 });
 let(g,o)=s.goal_fenced("ollama","ai_lmstudio",2,1,Some(fence));let approved=s.approved(&g,&o,0,Arc::new(||true));let receipt=o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();assert!(commits.load(Ordering::Acquire)>=2);assert_eq!(receipt.envelope().origin.pet_id,"ai_ollama");assert_eq!(s.f.store().read(&g.id,"ai_ollama").unwrap().revision,g.revision);assert_eq!(s.budget(&g)["attempts"].as_array().unwrap().len(),1);
}

fn restored_message_authority(s:&Setup)->(Fixture,Authority){
 fn copy(from:&std::path::Path,to:&std::path::Path){use std::os::unix::fs::{DirBuilderExt,PermissionsExt};fs::DirBuilder::new().mode(0o700).create(to).unwrap();for e in fs::read_dir(from).unwrap(){let e=e.unwrap();let target=to.join(e.file_name());if e.path().is_dir(){copy(&e.path(),&target);}else{fs::copy(e.path(),&target).unwrap();fs::set_permissions(target,fs::Permissions::from_mode(0o600)).unwrap();}}}
 let target=Fixture::new();copy(&s.f.path.join("authority"),&target.path.join("authority"));copy(s.f.store().root_path(),&target.path.join("goals"));let review=crate::pet_runtime::restoration::Review::capture(&target.path.join("authority"),&target.path.join("goals"),&"a".repeat(64),||Ok(())).unwrap();let hash=review.fingerprint().unwrap();review.apply(&hash,||Ok(())).unwrap();let authority=Authority::existing(&target.path.join("authority")).unwrap();(target,authority)
}
#[test]fn mesh_direct_restored_history_is_explicit_snapshot_data_and_never_reactivates_staging_or_retired_intents(){
 for variant in ["delivered","pending","retired"]{
  let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let original=if variant=="delivered"{let a=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&a,crate::assistant_memory::now(),||s.observe()).unwrap();a}else{pending_intent(&s,&g,&o)};
  if variant=="retired"{let review=retire_review(&s,&g,&original.scope().envelope.message_id,crate::assistant_memory::now());let mut a=review.approve(crate::assistant_memory::now(),Arc::new(||true)).unwrap();a.confirm(crate::assistant_memory::now()).unwrap();}
  let(target,a)=restored_message_authority(&s);let path=target.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id));let before=fs::read(&path).unwrap();let now=crate::assistant_memory::now();assert!(a.message_inbox("ai_lmstudio",None,16,now).unwrap().items.is_empty());
  let history=a.message_inbox_view("ai_lmstudio",None,16,now,true).unwrap();assert_eq!(history.schema_version,2);assert!(history.scan_complete&&!history.execution_authorized);assert_eq!(history.items.len(),usize::from(variant=="delivered"),"{variant}");
  if variant=="delivered"{let item=&history.items[0];assert!(!item.available);assert_eq!(item.receipt.scope,original.scope().clone());let provenance=item.historical_source.as_ref().unwrap();assert!(provenance.snapshot_only&&provenance.post_snapshot_state_unknown&&!provenance.execution_authorized);assert_eq!(provenance.source_file_hash,model::hash(&before));assert_eq!(provenance.source_profile_fingerprint,"a".repeat(64));assert!(a.message_inbox_view("ai_ollama",None,16,now,true).unwrap().items.is_empty());}
  let row=&g.admissions[0];assert_eq!(a.message_ack(&row.spec,row.proof.as_ref().unwrap(),original.scope()).err(),Some(Failure::RecoveryRequired));assert_eq!(fs::read(path).unwrap(),before);
 }
}
#[test]fn mesh_direct_historical_view_refuses_replaced_source_bytes_even_when_another_signed_grant_is_valid(){
 let s=Setup::new();let(g,o)=s.goal("ollama","ai_lmstudio",2,1);let approved=s.approved(&g,&o,0,Arc::new(||true));o.send_message(&g,&approved,crate::assistant_memory::now(),||s.observe()).unwrap();let(h,p)=s.goal("lmstudio","ai_ollama",2,1);let other=s.approved(&h,&p,0,Arc::new(||true));p.send_message(&h,&other,crate::assistant_memory::now(),||s.observe()).unwrap();let(target,a)=restored_message_authority(&s);
 let file=target.path.join("authority").join(format!("{}.json",g.admissions[0].spec.decision_id));fs::write(&file,s.bytes(&h)).unwrap();let before=fs::read(&file).unwrap();let page=a.message_inbox_view("ai_lmstudio",None,16,crate::assistant_memory::now(),true).unwrap();assert!(page.items.is_empty()&&!page.scan_complete);assert_eq!(page.unavailable,1);assert_eq!(fs::read(file).unwrap(),before);
}

#[path="message_received_tests.rs"]mod received;

#[path="message_reply_tests.rs"]mod reply;

#[path="received_request_tests.rs"]mod received_request;

#[path="delegation_tests.rs"]mod delegation_tests;
