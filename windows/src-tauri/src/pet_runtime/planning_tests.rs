use super::{planning::*,planning_journal,admission::{self,Review,Payment},admission_tests::{Fixture,evidence},authority::{self,Authority,Failure,Operation},authority_tests::root,authority_store::Fault,effect_budget::{Attempt,Kind,Phase},lease_controller::{Controller,Validity},model::{self,Record},store::nonce};
use serde_json::{json,Value};
use std::sync::Arc;

fn config()->crate::pet_config::Configuration {
    let mut c=crate::pet_config::defaults();c.pets.get_mut("ai_ollama").unwrap().permissions.get_or_insert_default().music=true;c
}
fn budget()->admission::Budget {let mut b=super::admission_tests::budget();b.payment=Payment::LocalOnly;b}
fn limits()->Limits {Limits {max_rounds:3,max_steps:2,max_output_bytes:2048,no_progress_limit:2}}
fn policy()->Policy {Policy::native(limits(),&budget()).unwrap()}
fn review(g:&Record)->Review {Review::prepare_planning(g,evidence(g,100_002),budget(),limits(),&config(),"plan-consent".into(),nonce().unwrap(),100_002).unwrap()}
pub(super)fn setup()->(Fixture,Authority,Record,super::lease_controller::Pending,super::lease_controller::Owned) {
    let f=Fixture::new();let g=f.goal("ollama");let a=Authority::ensure(&root(&f)).unwrap();
    let approved=review(&g).approve(&g,evidence(&g,100_003),100_003).unwrap();let cert=a.certify(&approved,&g,100_004).unwrap();
    let g=f.store().enqueue_certified(&cert,100_005).unwrap().record;
    let c=Controller::new().unwrap();let main=crate::chat::Chat::persistent(f.path.join("live.json"));
    let pending=c.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,100_006),"claim".into(),100_006,Arc::new(|_,_,_|Validity::Current)).unwrap();
    let (r,owner)=pending.claim(&f.store(),&a,&evidence(&g,100_007),100_007).unwrap();(f,a,r.record,pending,owner.unwrap())
}
fn attempt(request:&str)->Attempt {Attempt {id:nonce().unwrap(),request_id:request.into(),input_hash:"a".repeat(64),kind:Kind::Model,output_reserved:2048,phase:Phase::Reserved,accepted_output:None,reserved_at:100_008,started_at:None,finished_at:None}}
pub(super)fn grant(f:&Fixture,g:&Record)->Value {serde_json::from_slice(&std::fs::read(root(f).join(format!("{}.json",g.admissions[0].spec.decision_id))).unwrap()).unwrap()}
pub(super)fn output()->Value {json!({"kind":"plan","description":"Pause the private player","steps":[{"description":"Pause","postcondition":"Playback is paused","operation":{"operationId":"spotify.control","version":1,"arguments":{"action":"pause"}}}],"criteria":[{"criterionIndex":0,"predicate":{"kind":"spotifyPlayback","status":"paused"}}]})}
#[test] fn malformed_stored_policy_cannot_relax_the_parser_allocation_or_step_bounds() {
    let f=Fixture::new();let g=f.goal("ollama");
    for field in ["maxOutputBytes","maxRounds","maxSteps","noProgressLimit"] {
        let mut v=serde_json::to_value(policy()).unwrap();v["limits"][field]=json!(if field=="maxOutputBytes"{1_000_000}else{255});
        let malformed:Policy=serde_json::from_value(v).unwrap();assert!(parse(&g,&malformed,&output().to_string()).is_err(),"{field}");
    }
}
macro_rules! invoke {($a:expr,$g:expr,$o:expr,$method:ident,$($args:expr),*)=>{{$a.$method(&$g.admissions[0].spec,$g.admissions[0].proof.as_ref().unwrap(),$o.lease(),$($args),*)}}}

#[test] fn planning_bounds_are_finite_and_cannot_exceed_the_original_budget() {
    for (field,value) in [("maxRounds",0),("maxRounds",4),("maxSteps",0),("maxSteps",3),("maxOutputBytes",2047),("maxOutputBytes",4097),("noProgressLimit",0),("noProgressLimit",4)] {
        let mut v=serde_json::to_value(limits()).unwrap();v[field]=json!(value);let l:Limits=serde_json::from_value(v).unwrap();assert!(Policy::native(l,&budget()).is_err(),"{field}={value}");
    }
    let mut b=budget();b.max_tokens=Some(1);assert!(Policy::native(limits(),&b).is_err());b.max_tokens=None;b.max_cost_micros=Some(1);assert!(Policy::native(limits(),&b).is_err());
    assert!(serde_json::from_value::<Policy>(json!({"schemaVersion":1,"catalog":"shell","limits":limits()})).is_err());
    let mut v=serde_json::to_value(policy()).unwrap();v["operators"]=json!(["system.*"]);assert!(serde_json::from_value::<Policy>(v).is_err());
}
#[test] fn typed_plan_must_cover_every_original_criterion_and_has_no_execution_authority() {
    let f=Fixture::new();let g=f.goal("ollama");let p=parse(&g,&policy(),&output().to_string()).unwrap();
    let Parsed::Plan(plan)=p else {panic!("expected plan")};assert_eq!(plan.steps.len(),1);assert!(plan.verification.matches(&g));assert_eq!(plan.verification.conditions.len(),g.spec.criteria.len());
    let two=f.goal_named_criteria("ollama","two",vec!["Paused".into(),"Volume".into()]);assert!(parse(&two,&policy(),&output().to_string()).is_err());
    let mut v=output();let duplicate=v["criteria"][0].clone();v["criteria"].as_array_mut().unwrap().push(duplicate);assert!(parse(&two,&policy(),&v.to_string()).is_err());
    assert!(matches!(parse(&g,&policy(),r#"{"kind":"blocked","reason":"No supported observation"}"#),Ok(Parsed::Blocked(_))));
}
#[test] fn model_cannot_add_tools_change_catalog_skip_criteria_or_smuggle_unknown_fields() {
    let f=Fixture::new();let g=f.goal("ollama");
    for path in ["command","operation","version","argument","extraStep","criteria","predicate","tooManySteps"] {
        let mut v=output();match path {
            "command"=>v["command"]=json!("sh -c anything"),
            "operation"=>v["steps"][0]["operation"]["operationId"]=json!("system.shell"),
            "version"=>v["steps"][0]["operation"]["version"]=json!(2),
            "argument"=>v["steps"][0]["operation"]["arguments"]["code"]=json!("arbitrary"),
            "extraStep"=>v["steps"][0]["approved"]=json!(true),
            "criteria"=>v["criteria"]=json!([]),
            "predicate"=>v["criteria"][0]["predicate"]=json!({"kind":"run","command":"anything"}),
            "tooManySteps"=>{let step=v["steps"][0].clone();v["steps"]=json!([step.clone(),step.clone(),step]);},_=>unreachable!(),
        }assert!(parse(&g,&policy(),&v.to_string()).is_err(),"{path}");
    }
    for action in [json!({"action":"next"}),json!({"action":"play_pause"}),json!({"action":"play_item","uri":"spotify:track:abc"}),json!({"action":"set_volume","volume":101})] {
        let mut v=output();v["steps"][0]["operation"]["arguments"]=action;assert!(parse(&g,&policy(),&v.to_string()).is_err());
    }
    for raw in [format!("```json\n{}\n```",output()),"x".repeat(2049),r#"{"kind":"blocked","reason":"a","reason":"b"}"#.into(),r#"{"kind":"blocked","reason":"ok","grant":true}"#.into()] {assert!(parse(&g,&policy(),&raw).is_err());}
}
#[test] fn planning_requires_supported_native_model_and_music_permission() {
    let f=Fixture::new();for provider in ["codex","copilot","antigravity","integration_notion"] {
        let g=f.goal(provider);assert_eq!(Review::prepare_planning(&g,evidence(&g,100_002),budget(),limits(),&config(),"plan".into(),nonce().unwrap(),100_002).err(),Some(admission::Denial::Unsupported));
    }
    let g=f.goal("ollama");let mut denied=config();denied.pets.get_mut("ai_ollama").unwrap().permissions.as_mut().unwrap().music=false;assert_eq!(Review::prepare_planning(&g,evidence(&g,100_002),budget(),limits(),&denied,"plan".into(),nonce().unwrap(),100_002).err(),Some(admission::Denial::Unsupported));
    let mut b=budget();b.payment=Payment::ExistingAccount;assert_eq!(Review::prepare_planning(&g,evidence(&g,100_002),b,limits(),&config(),"plan".into(),nonce().unwrap(),100_002).err(),Some(admission::Denial::PaymentPolicy));
    let r=review(&g).proposal();assert_eq!(r.operation,"goalPlanning");assert!(!r.tool_execution_available);assert_eq!(r.planning_policy,Some(policy()));
}
#[test] fn planning_consent_has_a_separate_domain_and_legacy_wire_shape_is_preserved() {
    let(f,a,g,_p,o)=setup();assert_eq!(g.schema_version,8);let row=&g.admissions[0];let proof=row.proof.as_ref().unwrap();assert_eq!(proof.operation,Operation::GoalPlanning);assert_eq!(proof.schema_version,2);
    assert!(authority::consent_message(proof,&row.spec).unwrap().starts_with(b"coucou-native-goal-planning-consent-v1\0"));
    let mut swapped=proof.clone();swapped.operation=Operation::ConversationRespond;swapped.schema_version=1;assert!(authority::consent_message(&swapped,&row.spec).is_err());
    let b=grant(&f,&g);assert_eq!(b["body"]["schemaVersion"],4);assert_eq!(b["body"]["planningUsage"]["rounds"],json!([]));assert!(invoke!(a,g,o,output_remaining,100_008).is_ok());
    let legacy=f.goal("codex");let approved=super::admission_tests::approved(&legacy,"legacy");let spec=approved.spec();assert!(serde_json::to_value(spec).unwrap().get("planningPolicy").is_none());
    let proof=authority::ConsentProof {schema_version:1,key_id:"a".repeat(32),operation:Operation::ConversationRespond,tag:"b".repeat(64)};
    let mut exact=b"coucou-native-goal-consent-v1\0".to_vec();exact.extend(serde_json::to_vec(&(1,&proof.key_id,Operation::ConversationRespond,spec)).unwrap());assert_eq!(authority::consent_message(&proof,spec).unwrap(),exact);
}
#[test] fn seed_snapshot_remains_exact_and_schema_versions_cannot_drop_planning() {
    let(_f,_a,g,_p,_o)=setup();let mut changed=g.clone();changed.tasks.push(model::Task {id:"invented".into(),description:"Added without ACK".into(),state:model::WorkState::Proposed,plan_id:None});assert!(changed.validate().is_err());
    for version in [1,2,3,6,7] {let mut changed=g.clone();changed.schema_version=version;assert!(changed.validate().is_err());}
    let mut spec=g.admissions[0].spec.clone();spec.schema_version=2;assert!(!spec.binding_format());spec.schema_version=3;spec.planning_policy=None;assert!(!spec.binding_format());
}
#[test] fn model_attempt_and_round_are_one_signed_reservation_and_phase_transaction() {
    let(f,a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();let call=attempt("round-1");
    assert_eq!(invoke!(a,g,o,reserve_attempt,&call,100_008).err(),Some(Failure::Invalid));assert!(grant(&f,&g)["body"].get("budgetUsage").is_none());
    invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).unwrap();let b=grant(&f,&g);assert_eq!(b["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(b["body"]["planningUsage"]["rounds"][0]["source"]["recordHash"],model::fingerprint(&g).unwrap());assert_eq!(invoke!(a,g,o,output_remaining,100_008).unwrap(),2048);
    invoke!(a,g,o,start_attempt,&call,100_009).unwrap();assert_eq!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["phase"],"started");
    invoke!(a,g,o,finish_attempt,&call,Some(100),100_010).unwrap();let b=grant(&f,&g);assert_eq!(b["body"]["schemaVersion"],4);assert_eq!(b["body"]["planningUsage"]["rounds"][0]["phase"],"settled");assert_eq!(invoke!(a,g,o,output_remaining,100_010).unwrap(),3996);
    // A settled attempt alone is not an authenticated plan/outcome ACK.
    let req2=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-2").unwrap();assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&attempt("round-2"),&req2,100_011).err(),Some(Failure::RecoveryRequired));
}
#[test] fn uncertain_reservation_retry_never_duplicates_a_round_or_resets_a_budget() {
    let(f,mut a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();let call=attempt("round-1");let before=grant(&f,&g);
    a.fault=Fault::BeforeGrantPublish;assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).err(),Some(Failure::Unavailable));assert_eq!(grant(&f,&g),before);
    a.fault=Fault::AfterGrantPublish;assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).err(),Some(Failure::Uncertain));a.fault=Fault::None;
    invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_009).unwrap();let b=grant(&f,&g);assert_eq!(b["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);assert_eq!(b["body"]["planningUsage"]["rounds"].as_array().unwrap().len(),1);
    invoke!(a,g,o,start_attempt,&call,100_010).unwrap();assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_011).err(),Some(Failure::RecoveryRequired));
    invoke!(a,g,o,finish_attempt,&call,None,100_012).unwrap();assert_eq!(invoke!(a,g,o,output_remaining,100_012).unwrap(),2048);assert_eq!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0]["phase"],"uncertain");
}
#[test] fn changed_planning_limits_source_or_removed_journal_cannot_be_reauthenticated() {
    let(f,a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();let call=attempt("round-1");invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).unwrap();
    let mut changed=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();changed.source.record_hash="b".repeat(64);assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&call,&changed,100_009).err(),Some(Failure::Changed));
    let mut spec=g.admissions[0].spec.clone();spec.planning_policy.as_mut().unwrap().limits.max_output_bytes=3000;assert_eq!(a.output_remaining(&spec,g.admissions[0].proof.as_ref().unwrap(),o.lease(),100_009).err(),Some(Failure::Invalid));
    let mut b=grant(&f,&g);b["body"]["planningUsage"]["rounds"]=json!([]);std::fs::write(root(&f).join(format!("{}.json",spec.decision_id)),serde_json::to_vec(&b).unwrap()).unwrap();assert!(invoke!(a,g,o,output_remaining,100_009).is_err());
}
#[test] fn stop_and_original_deadline_prevent_new_planning_io_but_allow_accounting_to_drain() {
    let(_f,a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();let call=attempt("round-1");invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).unwrap();invoke!(a,g,o,start_attempt,&call,100_009).unwrap();
    a.request_stop(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),super::leases::StopReason::Cancel,100_010).unwrap();assert_eq!(invoke!(a,g,o,start_attempt,&call,100_011).err(),Some(Failure::Revoked));invoke!(a,g,o,finish_attempt,&call,None,100_012).unwrap();
    let(_f,a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-1").unwrap();assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&attempt("round-1"),&req,200_000).err(),Some(Failure::Revoked));
}

pub(super)fn settle(a:&Authority,g:&Record,o:&super::lease_controller::Owned,request:&str,text:&str,at:u64)->Attempt {
    let req=planning_journal::Request::capture(g,&g.admissions[0].spec,o.lease(),request).unwrap();let mut call=attempt(request);call.reserved_at=at;
    invoke!(a,g,o,reserve_planning_attempt,&call,&req,at).unwrap();invoke!(a,g,o,start_attempt,&call,at+1).unwrap();invoke!(a,g,o,finish_attempt,&call,Some(text.len() as u64),at+2).unwrap();
    let e=invoke!(a,g,o,seal_result,&call,text,text.len() as u64,at+3).unwrap();super::results::publish(a,&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&e).unwrap();call
}
pub(super)fn reconcile(a:&Authority,g:&Record,o:&super::lease_controller::Owned,request:&str,at:u64)->authority::Result<super::planning_output::Envelope> {
    super::planning_output::reconcile(a,&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),o.lease(),g,request,at)
}
#[test] fn signed_planning_output_is_not_a_canonical_plan_or_human_approval() {
    let(f,a,g,_p,o)=setup();settle(&a,&g,&o,"round-1",&output().to_string(),100_008);let e=reconcile(&a,&g,&o,"round-1",100_012).unwrap();
    let super::planning_output::Outcome::Plan {candidate}=e.body.outcome.clone() else {panic!("expected proposed plan")};
    assert_eq!(candidate.target_sequence,g.sequence+1);assert!(model::token(&candidate.target_revision));assert_eq!(candidate.verification.spec_hash,model::fingerprint(&g.spec).unwrap());
    assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().revision,g.revision);assert!(g.plans.is_empty());assert_eq!(o.workers(),0);
    let b=grant(&f,&g);assert_eq!(b["body"]["planningUsage"]["rounds"][0]["output"]["bodyHash"],model::fingerprint(&e.body).unwrap());
    let mut tampered=e.clone();if let super::planning_output::Outcome::Plan{candidate}= &mut tampered.body.outcome {candidate.steps[0].operation.as_mut().unwrap().arguments=json!({"action":"play"});}
    assert!(a.verify_planning_output(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&tampered).is_err());
}
#[test] fn planning_output_publication_and_journal_ack_recover_the_exact_evidence_without_new_attempts() {
    let(f,mut a,g,_p,o)=setup();let call=settle(&a,&g,&o,"round-1",&output().to_string(),100_008);
    a.fault=Fault::BeforePlanningOutputPublish;assert_eq!(reconcile(&a,&g,&o,"round-1",100_012).err(),Some(Failure::Unavailable));a.fault=Fault::None;
    assert!(super::planning_output::read(&a,&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&call.id).unwrap().is_none());
    a.fault=Fault::AfterPlanningOutputPublish;assert_eq!(reconcile(&a,&g,&o,"round-1",100_012).err(),Some(Failure::Uncertain));a.fault=Fault::None;
    let original=super::planning_output::read(&a,&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),&call.id).unwrap().unwrap();assert!(grant(&f,&g)["body"]["planningUsage"]["rounds"][0].get("output").is_none());
    a.fault=Fault::AfterGrantPublish;assert_eq!(reconcile(&a,&g,&o,"round-1",100_013).err(),Some(Failure::Uncertain));a.fault=Fault::None;
    let stable=reconcile(&a,&g,&o,"round-1",100_014).unwrap();assert_eq!(serde_json::to_value(&original).unwrap(),serde_json::to_value(&stable).unwrap());assert_eq!(grant(&f,&g)["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),1);
    let file=a.planning_root().unwrap().join(&g.admissions[0].spec.decision_id).join(format!("{}.json",call.id));std::fs::remove_file(file).unwrap();assert_eq!(reconcile(&a,&g,&o,"round-1",100_015).err(),Some(Failure::RecoveryRequired));
}
#[test] fn invalid_and_blocked_receipts_bound_retries_without_extending_the_grant() {
    let(f,a,g,_p,o)=setup();for (i,text) in ["invalid model text",r#"{"kind":"blocked","reason":"Unsupported request"}"#].iter().enumerate() {
        let request=format!("round-{}",i+1);let at=100_008+i as u64*5;settle(&a,&g,&o,&request,text,at);let e=reconcile(&a,&g,&o,&request,at+4).unwrap();assert!(matches!(e.body.outcome,super::planning_output::Outcome::Invalid|super::planning_output::Outcome::Blocked{..}));
    }
    let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"round-3").unwrap();let mut call=attempt("round-3");call.reserved_at=100_020;assert_eq!(invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_020).err(),Some(Failure::Capacity));
    let b=grant(&f,&g);assert_eq!(b["body"]["budgetUsage"]["attempts"].as_array().unwrap().len(),2);assert_eq!(b["body"]["planningUsage"]["rounds"].as_array().unwrap().len(),2);assert_eq!(g.admissions[0].spec.budget.deadline_at,200_000);
}
#[test] fn missing_model_receipt_or_changed_source_never_creates_a_planning_output() {
    let(f,a,g,_p,o)=setup();let req=planning_journal::Request::capture(&g,&g.admissions[0].spec,o.lease(),"missing").unwrap();let call=attempt("missing");invoke!(a,g,o,reserve_planning_attempt,&call,&req,100_008).unwrap();invoke!(a,g,o,start_attempt,&call,100_009).unwrap();invoke!(a,g,o,finish_attempt,&call,Some(10),100_010).unwrap();assert_eq!(reconcile(&a,&g,&o,"missing",100_011).err(),Some(Failure::RecoveryRequired));assert!(!a.planning_root().unwrap().exists());
    drop((f,a,g,o));let(_f,a,g,_p,o)=setup();settle(&a,&g,&o,"round-1",&output().to_string(),100_008);let mut changed=g.clone();changed.revision=nonce().unwrap();assert!(a.seal_planning_output(&g.admissions[0].spec,g.admissions[0].proof.as_ref().unwrap(),o.lease(),&changed,"round-1",100_012).is_err());
}
