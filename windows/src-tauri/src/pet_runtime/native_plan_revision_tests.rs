//! Private C3B data/history proofs. A revised JSON record is deliberately barred
//! from the verified runtime until signed publication and human control exist.
use super::{admission_tests::Fixture,authority::{Authority,Failure,GrantEnvelope},deterministic_runtime_tests::{draft_with,queued,owned},model::{self,Record},native_completion_tests::send,native_plan_revisions::{self as revision,Input},plans,store::{Store,nonce},continuation::NativeCarry};
use crate::chat::Chat;
use std::fs;
struct Env{f:Fixture,a:Authority,g:Record,carry:NativeCarry}
fn now()->u64{crate::assistant_memory::now()}
fn source(provider:&str)->Env{source_count(provider,2)}
fn source_count(provider:&str,count:usize)->Env{
 let f=Fixture::new();let a=Authority::ensure(&f.path.join("authority")).unwrap();let draft=draft_with(&f,provider,vec![super::criteria::NATIVE_MESSAGE_CRITERION.into()],count);let q=queued(&f,&a,&draft);
 let(g,o)=owned(&f,&a,&q,&Chat::persistent(f.path.join("main.json")));send(&g,&o,0);let d=o.drain(super::leases::DrainOutcome::Confirmed).unwrap();
 let store=Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap();let g=store.drain_owned(&a,&d,now()).unwrap().record;drop(d);drop(o);
 let(_,captured)=store.prepare_continuation(&f.path.join("authority"),&g.id,&g.spec.owner.pet_id,&g.revision,now()).unwrap().unwrap();let(reference,a)=captured.into_parts();let carry=reference.native.unwrap();assert_eq!(carry.receipts.len(),1);Env{f,a,g,carry}
}
fn input(g:&Record,index:usize,at:u64)->Input{
 let snapshot=plans::snapshot(g);let cause=snapshot.causes().nth(index).unwrap();let mut after=snapshot.input(&cause).unwrap();after.description="Explicitly revised pending message".into();let a=&mut after.operation.as_mut().unwrap().arguments;a["text"]=serde_json::json!(format!("Reviewed pending text at {at}"));a["expiresAt"]=serde_json::json!(at+60_000);Input{cause,after}
}
fn patch(e:&Env)->(revision::Reference,Record){let at=now()+180_000;let r=revision::prepare(&e.g,&e.carry,vec![input(&e.g,1,at)],nonce().unwrap(),at).unwrap();let mut target=e.g.clone();target.append_native_plan_revision(r.clone()).unwrap();(r,target)}
fn grant_bytes(e:&Env)->Vec<u8>{fs::read(e.f.path.join("authority").join(format!("{}.json",e.g.admissions[0].spec.decision_id))).unwrap()}

#[test]fn native_plan_revision_preserves_original_receipt_inputs_and_resolves_snapshots_for_every_factory_owner(){
 for provider in ["codex","copilot","antigravity","ollama","lmstudio","openai","integration_spotify"]{
  let e=source(provider);let before=grant_bytes(&e);let(r,target)=patch(&e);assert_eq!(target.schema_version,21);assert_eq!(target.id,e.g.id);assert_eq!(target.spec,e.g.spec);assert_eq!(target.sequence,e.g.sequence+1);assert!(r.preserves(&e.carry));assert!(r.present(&target));
  let old=plans::original_snapshot(&target,&e.g.admissions[0].spec).unwrap();assert_eq!(old,plans::snapshot(&e.g));assert!(!plans::matches(&target,&e.g.admissions[0].spec));
  let envelope:GrantEnvelope=serde_json::from_slice(&before).unwrap();let receipt=&envelope.body.messages.as_ref().unwrap().entries[0];assert!(receipt.scope.matches_snapshot(&target,&old));assert!(receipt.scope.input_matches(&target,&plans::snapshot(&target)));assert!(!receipt.scope.matches_proposal(&target));
  assert!(e.carry.matches_historical(&target));assert!(!e.carry.matches(&target));assert_eq!(grant_bytes(&e),before);assert_eq!(target.admissions.len(),1);
 }
}
#[test]fn native_plan_revision_refuses_covered_causes_malformed_operations_duplicates_limits_and_stale_sources(){
 let e=source("ollama");let before=super::reconciliation_tests::inventory(&e.f.path);let at=now()+180_000;
 let mut cases=vec![vec![input(&e.g,0,at)],vec![input(&e.g,1,at);2],vec![input(&e.g,1,at);revision::MAX_CHANGES+1],vec![]];
 for variant in ["self","operation","unknownCause","expired"]{let mut i=input(&e.g,1,at);match variant{
  "self"=>i.after.operation.as_mut().unwrap().arguments["toPetId"]=serde_json::json!(e.g.spec.owner.pet_id),
  "operation"=>i.after.operation.as_mut().unwrap().operation_id="shell.run".into(),
  "unknownCause"=>i.cause.step_id="not-the-reviewed-step".into(),
  "expired"=>i.after.operation.as_mut().unwrap().arguments["expiresAt"]=serde_json::json!(at),_=>unreachable!(),
 }cases.push(vec![i]);}
 for inputs in cases{assert!(revision::prepare(&e.g,&e.carry,inputs,nonce().unwrap(),at).is_err());}
 let(r,_)=patch(&e);let mut stale=e.g.clone();stale.revision=nonce().unwrap();assert!(!r.source_matches(&stale));let mut forged=r;forged.patches[0].cause=e.carry.receipts.first().map(|r|super::messages::Cause{task_id:e.g.plans[0].task_id.clone(),plan_id:r.plan_id.clone(),step_id:r.step_id.clone()}).unwrap();assert!(!forged.preserves(&e.carry));assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
}
#[test]fn native_plan_revision_consecutive_history_detects_rewritten_before_images_and_failed_patches_are_atomic(){
 let e=source("openai");let(r,first)=patch(&e);let mut projection=e.carry.clone();projection.proposal_hash=plans::fingerprint(&first).unwrap();
 // This projection is only DATA for a second history link, never a Captured,
 // native approval, signed grant or execution permit.
 let r2=revision::prepare(&first,&projection,vec![input(&first,1,r.reviewed_at+1)],nonce().unwrap(),r.reviewed_at+1).unwrap();let mut second=first.clone();second.append_native_plan_revision(r2).unwrap();
 assert_eq!(revision::snapshot_at(&second,&r.source_proposal_hash).unwrap(),plans::snapshot(&e.g));assert_eq!(revision::snapshot_at(&second,&r.target_proposal_hash).unwrap(),plans::snapshot(&first));assert_eq!(second.native_plan_revisions.len(),2);
 let mut changed=second.clone();changed.native_plan_revisions[0].patches[0].before.description="Forged past input".into();let event=changed.native_plan_revisions[0].target_sequence as usize-1;changed.events[event].fingerprint=model::fingerprint(&changed.native_plan_revisions[0]).unwrap();assert!(changed.validate().is_err());assert!(revision::snapshot_at(&changed,&plans::fingerprint(&changed).unwrap()).is_err());
 let mut bad=r;bad.target_proposal_hash="a".repeat(64);let mut snapshot=plans::snapshot(&e.g);let original=snapshot.clone();assert!(bad.apply_snapshot(&mut snapshot,false).is_err());assert_eq!(snapshot,original);
}
#[test]fn native_plan_revision_unsigned_json_cannot_enter_verified_runtime_or_obtain_continuation_authority(){
 let e=source("copilot");let(_,target)=patch(&e);let before=grant_bytes(&e);let path=e.f.path.join("goals").join(format!("{}.json",target.id));fs::write(path,serde_json::to_vec(&target).unwrap()).unwrap();
 assert!(e.a.verify_recovered_goal(&target).is_err());let store=Store::existing_verified(&e.f.path.join("goals"),&e.f.path.join("authority")).unwrap().unwrap();assert!(store.read(&target.id,&target.spec.owner.pet_id).is_err());assert!(store.prepare_continuation(&e.f.path.join("authority"),&target.id,&target.spec.owner.pet_id,&target.revision,now()+200_000).is_err());assert_eq!(grant_bytes(&e),before);assert_eq!(target.admissions[0].spec.budget.max_model_calls,0);
}

fn native_store(e:&Env)->Store{Store::existing_verified(&e.f.path.join("goals"),&e.f.path.join("authority")).unwrap().unwrap()}
fn reviewed(e:&Env,human:std::sync::Arc<dyn Fn()->bool+Send+Sync>)->super::native_plan_review::Approved{
 let at=now();let review=native_store(e).prepare_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,vec![input(&e.g,1,at)],nonce().unwrap(),at).unwrap();let p=review.proposal().unwrap();assert!(!p.execution_authorized&&!p.budget_reserved);review.approve(now(),human).unwrap()
}
fn body(e:&Env)->serde_json::Value{serde_json::from_slice::<serde_json::Value>(&grant_bytes(e)).unwrap()["body"].clone()}
#[test]fn native_plan_revision_signed_data_then_separate_budget_continues_without_replaying_any_owner(){
 use super::admission::{Review,Budget,Payment};use super::deterministic_runtime_tests::evidence;use std::sync::Arc;
 for provider in ["codex","copilot","antigravity","ollama","lmstudio","openai","integration_spotify"]{
  let e=source(provider);let store=native_store(&e);let usage=body(&e)["budgetUsage"].clone();let mut approved=reviewed(&e,Arc::new(||true));let result=approved.confirm(&store,now()).unwrap();assert!(result.acknowledged);assert!(!result.execution_authorized&&!result.new_admission_created);let target=result.receipt.record;assert_eq!(target.schema_version,21);assert_eq!(target.admissions.len(),1);assert_eq!(body(&e)["schemaVersion"],14);assert_eq!(body(&e)["budgetUsage"],usage);
  let frozen=grant_bytes(&e);let(source,captured)=store.prepare_continuation(&e.f.path.join("authority"),&target.id,&target.spec.owner.pet_id,&target.revision,now()).unwrap().unwrap();
  let at=now();let budget=Budget{deadline_at:at+120_000,max_model_calls:0,max_tool_calls:1,max_output_bytes:32768,max_tokens:None,max_cost_micros:None,payment:Payment::RuntimeOnly};
  let review=Review::prepare_deterministic_continuation(&source,evidence(&source,at),budget,captured,nonce().unwrap(),nonce().unwrap(),at).unwrap();let proposal=review.proposal();assert_eq!(proposal.continuation.as_ref().unwrap().schema_version,5);assert!(proposal.continuation.as_ref().unwrap().closure.is_none()&&proposal.continuation.as_ref().unwrap().drain.is_none());assert!(proposal.continuation.as_ref().unwrap().plan_revision.is_some());
  let mut consent=review.approve(&source,evidence(&source,now()),now()).unwrap();let authority=consent.take_native_authority().unwrap();let cert=authority.certify(&consent,&source,now()).unwrap();let queued=store.enqueue_certified(&cert,now()).unwrap().record;assert_eq!(queued.schema_version,22);assert_eq!(queued.admissions[1].spec.schema_version,8);
  let(g,o)=owned(&e.f,&e.a,&queued,&Chat::persistent(e.f.path.join("continued.json")));let partial=o.verify_native_goal(&store,&g).unwrap();assert!(!partial.report.criteria_verified);assert_eq!(partial.report.receipts.len(),1);assert_eq!(partial.report.receipts[0],e.carry.receipts[0]);send(&g,&o,1);
  let done=o.verify_native_goal(&store,&g).unwrap().completion.unwrap().record;assert_eq!(done.state,model::State::Completed);assert_eq!(done.id,e.g.id);let d=o.drain(super::leases::DrainOutcome::Confirmed).unwrap();store.drain_owned(&e.a,&d,now()).unwrap();assert_eq!(grant_bytes(&e),frozen);
  let history=store.history(&e.f.path.join("authority"),&done.id,&done.spec.owner.pet_id,&done.revision,now()).unwrap();assert!(history.accounting_complete&&history.source_consistent);let charge=history.cumulative_charge.unwrap();assert_eq!(charge.tool_calls,2);assert_eq!(charge.model_calls,0);assert_eq!(charge.unknown_tool_results,0);
  let before=super::reconciliation_tests::inventory(&e.f.path);let ack=store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&target.revision,&result.reference.id,now()).unwrap();assert!(ack.acknowledged);assert_eq!(ack.receipt.record.state,model::State::Completed);assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 }
}
#[test]fn native_plan_revision_grant_faults_and_goal_ack_preserve_one_original_review_and_charge(){
 use super::{authority_store::Fault,store::Fault as GoalFault};use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
 for fault in [Fault::BeforeGrantPublish,Fault::AfterGrantPublish]{let e=source("copilot");let mut approved=reviewed(&e,Arc::new(||true));let store=native_store(&e);let usage=body(&e)["budgetUsage"].clone();approved.review.authority.fault=fault;assert!(approved.confirm(&store,now()).is_err());let cached=approved.seal.clone().unwrap();approved.review.authority.fault=Fault::None;let result=approved.confirm(&store,now()).unwrap();assert!(result.acknowledged);assert_eq!(result.reference,cached.reference);assert_eq!(model::fingerprint(&result.receipt.record).unwrap(),cached.target_goal_hash);assert_eq!(body(&e)["budgetUsage"],usage);let frozen=super::reconciliation_tests::inventory(&e.f.path);let ack=store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&cached.reference.id,now()).unwrap();assert!(ack.acknowledged);assert_eq!(super::reconciliation_tests::inventory(&e.f.path),frozen);}
 for fault in [GoalFault::BeforePublish,GoalFault::AfterPublish]{let e=source("openai");let live=Arc::new(AtomicBool::new(true));let signal=live.clone();let mut approved=reviewed(&e,Arc::new(move||signal.load(Ordering::Acquire)));let mut store=native_store(&e);let usage=body(&e)["budgetUsage"].clone();store.fault=fault;let attempt=approved.confirm(&store,now());let seal=approved.seal.clone().unwrap();live.store(false,Ordering::Release);drop(approved);store.fault=GoalFault::None;
  match fault{GoalFault::BeforePublish=>{assert!(attempt.is_err());assert!(store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).is_err());let withdrawn=store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();assert!(withdrawn.abandoned.is_some());assert_eq!(store.read(&e.g.id,&e.g.spec.owner.pet_id).unwrap().revision,e.g.revision);assert!(store.prepare_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,vec![input(&e.g,1,now())],nonce().unwrap(),now()).is_ok());},GoalFault::AfterPublish=>{assert!(!attempt.unwrap().acknowledged);let ack=store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();assert!(ack.acknowledged);assert!(store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&ack.receipt.record.revision,&seal.reference.id,now()).is_err());},_=>unreachable!()}
  assert_eq!(body(&e)["budgetUsage"],usage);
 }
}
#[test]fn native_plan_revision_revocation_during_grant_and_goal_io_never_publishes_new_data(){
 use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
 for directory in ["authority","goals"]{let e=source("ollama");let dir=e.f.path.join(directory);let live=Arc::new(AtomicBool::new(true));let signal=live.clone();let human=Arc::new(move||{let staged=fs::read_dir(&dir).unwrap().filter_map(|r|r.ok()).any(|r|r.file_name().to_str().is_some_and(|s|s.starts_with(if directory=="authority"{".grant-"}else{".tmp-"})));if staged{signal.store(false,Ordering::Release);}signal.load(Ordering::Acquire)});let mut approved=reviewed(&e,human);let store=native_store(&e);let before=body(&e)["budgetUsage"].clone();assert!(approved.confirm(&store,now()).is_err());assert!(!live.load(Ordering::Acquire));let seal=approved.seal.clone().unwrap();drop(approved);assert_eq!(store.read_control(&e.g.id,&e.g.spec.owner.pet_id).unwrap().revision,e.g.revision);assert!(store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).is_err());let withdrawn=store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();assert!(withdrawn.abandoned.is_some());assert_eq!(body(&e)["budgetUsage"],before);let history=store.history(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,now()).unwrap();assert_eq!(history.pending_artifacts,0);assert!(history.accounting_complete&&history.source_consistent);}
}
#[test]fn native_plan_revision_cancel_has_priority_over_published_and_unpublished_preparations(){
 use super::{authority_store::Fault,store::Fault as GoalFault};use std::sync::Arc;
 for stage in ["grant","goal"]{let e=source("antigravity");let mut approved=reviewed(&e,Arc::new(||true));let mut store=native_store(&e);if stage=="grant"{approved.review.authority.fault=Fault::BeforeGrantPublish;}else{store.fault=GoalFault::BeforePublish;}assert!(approved.confirm(&store,now()).is_err());let seal=approved.seal.clone().unwrap();store.fault=GoalFault::None;approved.review.authority.fault=Fault::None;
  let cancelled=super::authority_store::mutate_with_authority(&store,&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,nonce().unwrap(),model::Action::Cancel,now()).unwrap().record;assert_eq!(cancelled.state,model::State::Cancelled);assert!(approved.confirm(&store,now()).is_err());drop(approved);
  let withdrawn=store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&cancelled.revision,&seal.reference.id,now()).unwrap();assert!(withdrawn.abandoned.is_some());assert_eq!(store.read(&e.g.id,&e.g.spec.owner.pet_id).unwrap().state,model::State::Cancelled);assert_eq!(body(&e)["phase"],"revoked");assert_eq!(body(&e)["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
 }
}
#[test]fn native_plan_revision_withdrawal_faults_reuse_signed_reduction_after_restart(){
 use super::{authority_store::Fault,store::Fault as GoalFault};use std::sync::Arc;
 for fault in [Fault::BeforeGrantPublish,Fault::AfterGrantPublish]{let e=source("lmstudio");let mut approved=reviewed(&e,Arc::new(||true));let mut store=native_store(&e);store.fault=GoalFault::BeforePublish;assert!(approved.confirm(&store,now()).is_err());let seal=approved.seal.clone().unwrap();store.fault=GoalFault::None;drop(approved);
  let absent=store.native_plan_absence(seal.clone());let mut authority=Authority::existing(&e.f.path.join("authority")).unwrap();authority.fault=fault;assert!(authority.abandon_native_plan(&absent,&seal,now()).is_err());drop(authority);
  let saved=store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();assert!(saved.abandoned.is_some());let before=super::reconciliation_tests::inventory(&e.f.path);let again=store.abandon_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();assert_eq!(saved,again);assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 }
}

#[test]fn native_plan_revision_storage_limit_is_refused_before_approval_and_legacy_bytes_stay_exact(){
 let e=source_count("ollama",16);let store=native_store(&e);let before=super::reconciliation_tests::inventory(&e.f.path);let at=now();let mut inputs=vec![];
 for i in 1..16{let mut v=input(&e.g,i,at);v.after.description="d".repeat(4096);v.after.postcondition="p".repeat(4096);v.after.operation.as_mut().unwrap().arguments["text"]=serde_json::json!("x".repeat(2048));inputs.push(v);}
 assert_eq!(store.prepare_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,inputs,nonce().unwrap(),at).err(),Some(model::Error::LimitReached));assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 let bytes=grant_bytes(&e);let value:serde_json::Value=serde_json::from_slice(&bytes).unwrap();assert!(value["body"].get("nativePlanRevisions").is_none());let parsed:GrantEnvelope=serde_json::from_slice(&bytes).unwrap();assert_eq!(serde_json::to_vec(&parsed).unwrap(),bytes);
 let bytes=fs::read(e.f.path.join("goals").join(format!("{}.json",e.g.id))).unwrap();assert!(serde_json::from_slice::<serde_json::Value>(&bytes).unwrap().get("nativePlanRevisions").is_none());assert_eq!(serde_json::to_vec(&serde_json::from_slice::<Record>(&bytes).unwrap()).unwrap(),bytes);
}
#[test]fn native_plan_revision_total_history_limit_and_duplicate_ids_are_data_only_and_bounded(){
 let e=source("ollama");let mut goal=e.g.clone();let mut projection=e.carry.clone();let at=now()+180_000;
 for i in 0..revision::MAX_REVISIONS{projection.proposal_hash=plans::fingerprint(&goal).unwrap();let r=revision::prepare(&goal,&projection,vec![input(&goal,1,at+i as u64)],nonce().unwrap(),at+i as u64).unwrap();goal.append_native_plan_revision(r).unwrap();}
 projection.proposal_hash=plans::fingerprint(&goal).unwrap();assert!(revision::prepare(&goal,&projection,vec![input(&goal,1,at+32)],nonce().unwrap(),at+32).is_err());assert_eq!(goal.native_plan_revisions.len(),16);assert!(e.a.verify_recovered_goal(&goal).is_err());
 goal.native_plan_revisions[1].id=goal.native_plan_revisions[0].id.clone();let seq=goal.native_plan_revisions[1].target_sequence as usize-1;goal.events[seq].fingerprint=model::fingerprint(&goal.native_plan_revisions[1]).unwrap();assert!(goal.validate().is_err());
}


fn control_input(e:&Env)->super::native_plan_revision_api::PrepareNativePlanInput{let at=now();super::native_plan_revision_api::PrepareNativePlanInput{pet_id:e.g.spec.owner.pet_id.clone(),goal_id:e.g.id.clone(),revision:e.g.revision.clone(),request_id:nonce().unwrap(),inputs:vec![input(&e.g,1,at)]}}
fn control_review(e:&Env,i:&super::native_plan_revision_api::PrepareNativePlanInput)->super::native_plan_review::Review{native_store(e).prepare_native_plan(&e.f.path.join("authority"),&i.goal_id,&i.pet_id,&i.revision,i.inputs.clone(),i.request_id.clone(),now()).unwrap()}
fn origin()->super::message_api::native::Origin{super::message_api::native::Origin{window:"settings".into(),frame:crate::settings::review::Frame::fixture()}}
#[test]fn native_plan_revision_frame_binds_exact_inputs_owner_and_window_and_decline_never_grants_data(){
 use super::native_plan_revision_api::{ConfirmNativePlanInput,native::{Reviews,Confirmed}};
 let e=source("ollama");let store=native_store(&e);let input=control_input(&e);let native=origin();let mut cache=Reviews::default();let before=super::reconciliation_tests::inventory(&e.f.path);let t=cache.insert(native.clone(),&input,control_review(&e,&input),now()).unwrap();assert_eq!(cache.existing(&native,&input,now()).unwrap().unwrap().token,t.token);
 let mut changed=input.clone();changed.inputs[0].after.operation.as_mut().unwrap().arguments["text"]=serde_json::json!("Different arguments after review");assert!(cache.existing(&native,&changed,now()).is_err());
 let confirm=ConfirmNativePlanInput{pet_id:input.pet_id.clone(),goal_id:input.goal_id.clone(),revision:input.revision.clone(),token:t.token,approve:true};assert!(cache.lookup(&origin(),&confirm).is_err());let mut wrong=confirm.clone();wrong.pet_id="agent_codex".into();assert!(cache.lookup(&native,&wrong).is_err());wrong=confirm.clone();wrong.revision=nonce().unwrap();assert!(cache.lookup(&native,&wrong).is_err());
 let p=cache.lookup(&native,&confirm).unwrap();assert!(matches!(p.choose(&store,&e.f.path.join("authority"),&e.g,false,now()).unwrap(),Confirmed::Declined));assert!(p.choose(&store,&e.f.path.join("authority"),&e.g,true,now()).is_err());assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 let fresh=control_input(&e);let fresh_ticket=cache.insert(native.clone(),&fresh,control_review(&e,&fresh),now()).unwrap();let expired=ConfirmNativePlanInput{pet_id:fresh.pet_id,goal_id:fresh.goal_id,revision:fresh.revision,token:fresh_ticket.token,approve:true};let stale=cache.lookup(&native,&expired).unwrap();assert!(stale.choose(&store,&e.f.path.join("authority"),&e.g,true,fresh_ticket.proposal.expires_at+1).is_err());assert!(stale.choose(&store,&e.f.path.join("authority"),&e.g,true,now()).is_err());assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 native.frame.invalidate_for_test();assert!(cache.lookup(&native,&confirm).is_err());let next=super::message_api::native::Origin{window:native.window.clone(),frame:native.frame.next_for_test()};assert!(cache.lookup(&next,&confirm).is_err());
 let raw=serde_json::json!({"petId":input.pet_id,"goalId":input.goal_id,"revision":input.revision,"token":confirm.token,"approve":true,"inputs":[]});assert!(serde_json::from_value::<ConfirmNativePlanInput>(raw).is_err());
}
#[test]fn native_plan_revision_frame_ack_is_a_past_fact_even_after_a_later_decline(){
 use super::{native_plan_revision_api::{ConfirmNativePlanInput,native::{Reviews,Confirmed}},store::Fault};
 let e=source("copilot");let input=control_input(&e);let native=origin();let mut cache=Reviews::default();let ticket=cache.insert(native.clone(),&input,control_review(&e,&input),now()).unwrap();let confirm=ConfirmNativePlanInput{pet_id:input.pet_id,goal_id:input.goal_id,revision:input.revision,token:ticket.token,approve:true};let pending=cache.lookup(&native,&confirm).unwrap();let mut store=native_store(&e);store.fault=Fault::AfterPublish;
 let first=pending.choose(&store,&e.f.path.join("authority"),&e.g,true,now()).unwrap();let Confirmed::Revised{receipt}=first else{panic!("same native review must retain uncertain data publication")};assert!(!receipt.acknowledged);let target=receipt.receipt.record;assert_eq!(target.admissions.len(),1);store.fault=Fault::None;let before=body(&e)["budgetUsage"].clone();
 let second=pending.choose(&store,&e.f.path.join("authority"),&target,false,now()).unwrap();let Confirmed::Revised{receipt}=second else{panic!("decline cannot pretend already persisted data was undone")};assert!(receipt.acknowledged);assert!(!receipt.execution_authorized&&!receipt.new_admission_created);assert_eq!(body(&e)["budgetUsage"],before);
 native.frame.invalidate_for_test();assert!(cache.lookup(&native,&confirm).is_err());let ack=store.ack_native_plan(&e.f.path.join("authority"),&e.g.id,&e.g.spec.owner.pet_id,&target.revision,&receipt.reference.id,now()).unwrap();assert!(ack.acknowledged);assert_eq!(ack.receipt.record.admissions.len(),1);
}
#[test]fn native_plan_revision_inventory_is_passive_and_distinguishes_staged_withdrawn_and_wrong_scope(){
 use super::{authority_store::Fault,native_plan_inventory::State};use std::sync::Arc;
 let e=source("openai");let store=native_store(&e);let path=e.f.path.join("authority");let first=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,None).unwrap();assert!(first.items.is_empty());assert!(first.fact_coverage_complete);assert_eq!(first.coverage.as_ref().unwrap(),&e.carry);assert!(!first.execution_authorized&&!first.repair_performed);let before=super::reconciliation_tests::inventory(&e.f.path);let second=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,None).unwrap();assert_eq!(first.source_hash,second.source_hash);assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 let mut approved=reviewed(&e,Arc::new(||true));approved.review.authority.fault=Fault::BeforeGrantPublish;assert!(approved.confirm(&store,now()).is_err());let seal=approved.seal.clone().unwrap();let before=super::reconciliation_tests::inventory(&e.f.path);let staged=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,None).unwrap();assert_eq!(staged.items.len(),1);assert!(!staged.fact_coverage_complete);assert_eq!(staged.coverage.as_ref().unwrap(),&e.carry);assert_eq!(staged.items[0].state,State::Staged);assert_eq!(staged.items[0].reference.id,seal.reference.id);assert!(staged.items[0].can_abandon&&!staged.items[0].can_ack);assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);
 assert!(store.native_plan_inventory(&path,&e.g.id,"agent_codex",&e.g.revision,None).is_err());assert!(store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&nonce().unwrap(),None).is_err());assert!(store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,Some("../other-profile")).is_err());
 store.abandon_native_plan(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();let withdrawn=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,None).unwrap();assert_eq!(withdrawn.items[0].state,State::Abandoned);assert!(withdrawn.fact_coverage_complete);assert!(!withdrawn.items[0].can_ack&&!withdrawn.items[0].can_abandon);assert_ne!(staged.source_hash,withdrawn.source_hash);
}
#[test]fn native_plan_revision_inventory_pages_share_a_source_fence_and_do_not_renew_any_budget(){
 use super::{native_plan_inventory::State,store::Fault};use std::sync::Arc;
 let e=source("lmstudio");let path=e.f.path.join("authority");let mut store=native_store(&e);let budget=body(&e)["budgetUsage"].clone();
 for _ in 0..17{let mut approved=reviewed(&e,Arc::new(||true));store.fault=Fault::BeforePublish;assert!(approved.confirm(&store,now()).is_err());let seal=approved.seal.clone().unwrap();store.fault=Fault::None;store.abandon_native_plan(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,&seal.reference.id,now()).unwrap();}
 let before=super::reconciliation_tests::inventory(&e.f.path);let first=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,None).unwrap();assert_eq!(first.items.len(),16);assert!(first.has_more);let second=store.native_plan_inventory(&path,&e.g.id,&e.g.spec.owner.pet_id,&e.g.revision,first.next_cursor.as_deref()).unwrap();assert_eq!(second.items.len(),1);assert!(!second.has_more);assert_eq!(first.source_hash,second.source_hash);assert!(first.items.iter().chain(&second.items).all(|i|i.state==State::Abandoned&&!i.can_ack&&!i.can_abandon));assert_eq!(super::reconciliation_tests::inventory(&e.f.path),before);assert_eq!(body(&e)["budgetUsage"],budget);
}

#[test]fn native_plan_revision_settled_accounting_without_an_effect_receipt_is_still_incomplete(){
 use super::effect_budget::{Attempt,Kind,Phase};
 let f=Fixture::new();let a=Authority::ensure(&f.path.join("authority")).unwrap();let draft=draft_with(&f,"ollama",vec![super::criteria::NATIVE_MESSAGE_CRITERION.into()],2);let q=queued(&f,&a,&draft);let(g,o)=owned(&f,&a,&q,&Chat::persistent(f.path.join("main.json")));send(&g,&o,0);let row=&g.admissions[0];let at=now();
 let attempt=Attempt{id:nonce().unwrap(),request_id:"settled-without-effect-receipt".into(),input_hash:"a".repeat(64),kind:Kind::Tool,output_reserved:512,phase:Phase::Reserved,accepted_output:None,reserved_at:at,started_at:None,finished_at:None};
 a.reserve_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&attempt,at).unwrap();a.start_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&attempt,now()).unwrap();a.finish_attempt(&row.spec,row.proof.as_ref().unwrap(),o.lease(),&attempt,Some(0),now()).unwrap();
 let store=Store::existing_verified(&f.path.join("goals"),&f.path.join("authority")).unwrap().unwrap();let d=o.drain(super::leases::DrainOutcome::Confirmed).unwrap();let closed=store.drain_owned(&a,&d,now()).unwrap().record;drop(d);drop(o);
 let before=super::reconciliation_tests::inventory(&f.path);let history=store.history(&f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,now()).unwrap();assert!(!history.outcomes_complete);assert_eq!(history.authenticated_subtotal.unknown_tool_results,1);
 let inventory=store.native_plan_inventory(&f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,None).unwrap();assert_eq!(inventory.coverage.as_ref().unwrap().receipts.len(),1);assert!(!inventory.fact_coverage_complete);assert!(!inventory.execution_authorized&&!inventory.repair_performed);assert_eq!(super::reconciliation_tests::inventory(&f.path),before);
 assert!(store.prepare_native_plan(&f.path.join("authority"),&closed.id,&closed.spec.owner.pet_id,&closed.revision,vec![input(&closed,1,now())],nonce().unwrap(),now()).is_err());
}
