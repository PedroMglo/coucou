use super::{admission::{Review, Denial}, admission_tests::{Fixture,evidence,budget}, authority::Authority, authority_tests::root, model::*, plans, store::nonce};
use serde_json::{json,Value};
fn draft(provider:&str)->(Fixture,Record){
    let f=Fixture::new();let g=f.goal(provider);
    let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"task".into(),Action::ProposeTask{description:"Preserve the original goal".into()},100_002).unwrap().record;
    let task=g.tasks[0].id.clone();
    let operation=plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:json!({"action":"pause"})};
    let g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"plan".into(),Action::ProposePlan{task_id:task,steps:vec![StepInput{description:"Propose pausing Spotify".into(),postcondition:"Playback must be observed paused".into(),operation:Some(operation)}]},100_003).unwrap().record;
    (f,g)
}
#[test]
fn typed_proposals_round_trip_without_becoming_authority_for_any_owner(){
    for provider in ["codex","antigravity","copilot","ollama","integration_notion"]{
        let(f,g)=draft(provider);assert_eq!(g.schema_version,6);assert_eq!(g.state,State::Draft);
        let restored=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert!(restored.admissions.is_empty());
        assert_eq!(serde_json::to_value(&restored.plans).unwrap(),serde_json::to_value(&g.plans).unwrap());
        assert!(!root(&f).exists());
    }
}
#[test]
fn invalid_operation_versions_arguments_and_injected_permits_are_rejected_before_mutation(){
    let(f,g)=draft("ollama");let before=serde_json::to_value(&g).unwrap();
    for operation in [json!({"operationId":"shell.run","version":1,"arguments":{}}),json!({"operationId":"spotify.control","version":2,"arguments":{"action":"pause"}}),json!({"operationId":"spotify.control","version":1,"arguments":{"action":"set_volume"}})]{
        let operation=serde_json::from_value(operation).unwrap();
        let result=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"bad-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:vec![StepInput{description:"x".into(),postcondition:"y".into(),operation:Some(operation)}]},100_004);
        assert!(result.is_err());assert_eq!(serde_json::to_value(f.store().read(&g.id,&g.spec.owner.pet_id).unwrap()).unwrap(),before);
    }
    assert!(serde_json::from_value::<plans::OperationProposal>(json!({"operationId":"spotify.control","version":1,"arguments":{"action":"pause"},"approved":true})).is_err());
}
#[test]
fn a_typed_plan_cannot_be_disguised_as_an_old_goal_schema(){
    let(_f,g)=draft("ollama");for old in 1..=5{let mut altered=g.clone();altered.schema_version=old;assert!(altered.validate().is_err());}
}
#[test]
fn the_review_shows_the_exact_static_work_and_explicitly_excludes_tool_execution(){
    let(_f,g)=draft("ollama");let review=Review::prepare(&g,evidence(&g,100_004),budget(),"queue".into(),nonce().unwrap(),100_004).unwrap();
    let view=serde_json::to_value(review.proposal()).unwrap();
    assert_eq!(view["intent"],g.spec.intent);assert_eq!(view["criteria"],json!(g.spec.criteria));
    assert_eq!(view["proposals"]["plans"][0]["steps"][0]["operation"]["arguments"]["action"],"pause");
    assert_eq!(view["operation"],"conversationRespond");assert_eq!(view["toolExecutionAvailable"],false);
    assert!(view["proposals"]["tasks"][0].get("state").is_none());
}
#[test]
fn unchanged_revision_cannot_hide_a_plan_change_between_review_and_confirmation(){
    let(_f,g)=draft("ollama");let review=Review::prepare(&g,evidence(&g,100_004),budget(),"queue".into(),nonce().unwrap(),100_004).unwrap();
    let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments["action"]=json!("play");assert!(changed.validate().is_ok());assert_eq!(changed.revision,g.revision);
    assert!(matches!(review.approve(&changed,evidence(&changed,100_005),100_005),Err(Denial::RevisionChanged)));
}
#[test]
fn changed_work_after_confirmation_cannot_be_certified_or_enqueued(){
    let(f,g)=draft("ollama");let a=Authority::ensure(&root(&f)).unwrap();
    let approved=Review::prepare(&g,evidence(&g,100_004),budget(),"queue".into(),nonce().unwrap(),100_004).unwrap().approve(&g,evidence(&g,100_005),100_005).unwrap();
    let mut changed=g.clone();changed.tasks[0].description="replacement".into();
    assert!(a.certify(&approved,&changed,100_006).is_err());assert!(f.store().enqueue(&approved,100_006).is_ok());
}
#[test]
fn active_plan_tampering_fails_validation_and_resealed_data_still_fails_native_provenance(){
    let(f,g)=draft("ollama");let a=Authority::ensure(&root(&f)).unwrap();
    let approved=Review::prepare(&g,evidence(&g,100_004),budget(),"queue".into(),nonce().unwrap(),100_004).unwrap().approve(&g,evidence(&g,100_005),100_005).unwrap();
    let cert=a.certify(&approved,&g,100_006).unwrap();let queued=f.store().enqueue_certified(&cert,100_007).unwrap().record;
    let mut changed=queued.clone();changed.plans[0].steps[0].postcondition="unreviewed success".into();assert!(changed.validate().is_err());
    changed.admissions[0].spec.proposal_hash=Some(plans::fingerprint(&changed).unwrap());
    changed.events.last_mut().unwrap().fingerprint=fingerprint(&changed.admissions[0].spec).unwrap();assert!(changed.validate().is_ok());
    assert!(a.verify_queued(&changed,100_008).is_err());assert!(a.verify_queued(&queued,100_008).is_ok());
}
#[test]
fn historical_closed_seals_survive_new_draft_work_without_authorizing_it(){
    let(f,g)=draft("ollama");let approved=Review::prepare(&g,evidence(&g,100_004),budget(),"queue".into(),nonce().unwrap(),100_004).unwrap().approve(&g,evidence(&g,100_005),100_005).unwrap();
    let queued=f.store().enqueue(&approved,100_006).unwrap().record;
    let paused=f.store().mutate(&g.id,&g.spec.owner.pet_id,&queued.revision,"pause".into(),Action::Pause,100_007).unwrap().record;
    let resumed=f.store().mutate(&g.id,&g.spec.owner.pet_id,&paused.revision,"resume".into(),Action::ResumeDraft,100_008).unwrap().record;
    let new=f.store().mutate(&g.id,&g.spec.owner.pet_id,&resumed.revision,"new-task".into(),Action::ProposeTask{description:"New proposed work".into()},100_009).unwrap().record;
    assert!(new.validate().is_ok());assert_eq!(new.schema_version,6);
    assert_eq!(new.admissions[0].spec.proposal_hash,queued.admissions[0].spec.proposal_hash);
    assert!(!plans::matches(&new,&new.admissions[0].spec));assert!(plans::model_input(&new,&new.admissions[0].spec).is_err());
}
#[test]
fn progress_state_is_separate_from_static_proposal_identity(){
    let(_f,g)=draft("ollama");let original=plans::fingerprint(&g).unwrap();let mut progress=g.clone();progress.tasks[0].state=WorkState::RecoveryRequired;progress.plans[0].steps[0].state=WorkState::Cancelled;
    assert_eq!(plans::fingerprint(&progress).unwrap(),original);
    progress.plans[0].steps[0].operation.as_mut().unwrap().arguments["action"]=json!("play");assert_ne!(plans::fingerprint(&progress).unwrap(),original);
}
#[test]
fn oversized_reviewed_input_is_rejected_without_silent_plan_or_intent_truncation(){
    let(f,g)=draft("ollama");let large=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"large-plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps:(0..64).map(|_|StepInput{description:"x".repeat(4096),postcondition:"y".repeat(4096),operation:None}).collect()},100_004).unwrap().record;
    assert!(large.validate().is_ok());assert_eq!(plans::reviewed_input(&large).unwrap_err(),Error::LimitReached);
    assert!(matches!(Review::prepare(&large,evidence(&large,100_005),budget(),"queue".into(),nonce().unwrap(),100_005),Err(Denial::InputLimit)));
}
#[test]
fn legacy_admissions_keep_their_old_wire_shape_and_do_not_adopt_typed_plans(){
    let f=Fixture::new();let g=f.goal("ollama");let approved=Review::prepare(&g,evidence(&g,100_002),budget(),"queue".into(),nonce().unwrap(),100_002).unwrap().approve(&g,evidence(&g,100_003),100_003).unwrap();
    let queued=f.store().enqueue(&approved,100_004).unwrap().record;let mut legacy=queued.clone();legacy.admissions[0].spec.schema_version=1;legacy.admissions[0].spec.proposal_hash=None;legacy.schema_version=3;
    legacy.events.last_mut().unwrap().fingerprint=fingerprint(&legacy.admissions[0].spec).unwrap();assert!(legacy.validate().is_ok());
    let raw=serde_json::to_value(&legacy.admissions[0].spec).unwrap();assert!(raw.get("proposalHash").is_none());
    assert_eq!(plans::model_input(&legacy,&legacy.admissions[0].spec).unwrap(),g.spec.intent);
    let(_other,typed)=draft("ollama");assert!(!plans::matches(&typed,&legacy.admissions[0].spec));
}
#[test]
fn textual_input_preserves_intent_criteria_and_parameters_without_an_execution_claim(){
    let(_f,g)=draft("ollama");let raw=plans::reviewed_input(&g).unwrap();let value:Value=serde_json::from_str(&raw).unwrap();
    assert_eq!(value["intent"],g.spec.intent);assert_eq!(value["criteria"],json!(g.spec.criteria));
    assert_eq!(value["proposals"]["plans"][0]["steps"][0]["operation"]["operationId"],"spotify.control");
    assert_eq!(value["toolExecutionAuthorized"],false);assert_eq!(value["criteriaVerified"],false);
}
