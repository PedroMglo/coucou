use super::{model::*,store::{Store,Fault,nonce}};
use std::{fs,os::unix::fs::{PermissionsExt,symlink},path::PathBuf};
struct Fixture{base:PathBuf,root:PathBuf}
impl Fixture{
    fn new()->Self{let base=std::env::temp_dir().join(format!("coucou-goal-tests-{}",nonce().unwrap()));fs::create_dir(&base).unwrap();fs::set_permissions(&base,fs::Permissions::from_mode(0o700)).unwrap();Self{root:base.join("goals"),base}}
    fn store(&self)->Store{Store::ensure(&self.root).unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.base);}}
fn spec(request:&str)->Spec{Spec{restored_from:None,request_id:request.into(),owner:Owner{pet_id:"agent_codex".into(),provider:"codex".into(),adapter:"codex".into(),model_hint:None},context:Context::Unbound,intent:"Concluir o pedido original com critérios verificáveis".into(),criteria:vec!["O resultado foi verificado".into()]}}
fn record()->Record{Record::new(spec("create-1"),100_000,"0".repeat(32)).unwrap()}
fn apply(r:&mut Record,key:&str,action:Action){let now=r.updated_at+1;let rev=nonce().unwrap();r.apply(key.into(),action,now,rev).unwrap();}
fn proposal()->Action{Action::ProposeTask{description:"Preparar o resultado".into()}}
fn assert_error<T>(r:Result<T>,e:Error){assert_eq!(r.err(),Some(e));}

#[test]fn original_intent_and_more_than_ten_tasks_are_preserved(){
    let f=Fixture::new();let store=f.store();let mut s=spec("long-intent");s.intent="pedido completo\n".repeat(1000);let original=s.intent.clone();let mut r=store.create(s,100_000).unwrap().record;
    for n in 0..20{r=store.mutate(&r.id,"agent_codex",&r.revision,format!("task-{n}"),proposal(),100_001+n).unwrap().record;}
    let reopened=Store::existing(&f.root).unwrap().unwrap().read(&r.id,"agent_codex").unwrap();assert_eq!(reopened.spec.intent,original);assert_eq!(reopened.tasks.len(),20);assert_eq!(reopened.events.len(),21);assert_eq!(reopened.state,State::Draft);
}
#[test]fn all_current_native_pet_owners_remain_distinct(){
    let f=Fixture::new();let store=f.store();let mut ids=std::collections::BTreeSet::new();
    for (pet,provider,adapter)in [("agent_codex","codex","codex"),("agent_copilot","copilot","copilot"),("agent_antigravity","antigravity","antigravity"),("ai_ollama","ollama","ollama"),("integration_notion","integration_notion","codex")]{
        let mut s=spec("same-request");s.owner=Owner{pet_id:pet.into(),provider:provider.into(),adapter:adapter.into(),model_hint:None};let r=store.create(s,100_000).unwrap().record;assert!(ids.insert(r.id.clone()));assert_eq!(store.read(&r.id,pet).unwrap().spec.owner.provider,provider);assert_eq!(store.list(Some(pet)).unwrap().goals.len(),1);
    }
    assert_eq!(store.list(None).unwrap().goals.len(),5);
}
#[test]fn reads_of_missing_storage_do_not_initialize_or_dispatch(){let f=Fixture::new();assert!(Store::existing(&f.root).unwrap().is_none());assert!(!f.root.exists());}
#[test]fn creation_is_idempotent_and_conflicting_intent_never_overwrites(){
    let f=Fixture::new();let store=f.store();let s=spec("one");let first=store.create(s.clone(),100_000).unwrap();let again=store.create(s.clone(),100_002).unwrap();assert_eq!(first.status,CommitStatus::Durable);assert_eq!(again.status,CommitStatus::Unchanged);assert_eq!(first.record.revision,again.record.revision);
    let mut other=s;other.intent="Outro pedido".into();assert_error(store.create(other,100_003),Error::Conflict);assert_eq!(store.list(None).unwrap().goals.len(),1);
}
#[test]fn mutations_use_original_revision_and_deduplicate_uncertain_acknowledgments(){
    let f=Fixture::new();let store=f.store();let first=store.create(spec("one"),100_000).unwrap().record;
    let changed=store.mutate(&first.id,"agent_codex",&first.revision,"task-1".into(),proposal(),100_001).unwrap().record;
    let replay=store.mutate(&first.id,"agent_codex",&first.revision,"task-1".into(),proposal(),100_002).unwrap();assert_eq!(replay.status,CommitStatus::Unchanged);assert_eq!(replay.record.tasks.len(),1);assert_eq!(changed.revision,replay.record.revision);
    assert_error(store.mutate(&first.id,"agent_codex",&first.revision,"task-2".into(),proposal(),100_003),Error::Conflict);
    assert_error(store.mutate(&first.id,"agent_codex",&changed.revision,"task-1".into(),Action::Pause,100_003),Error::Conflict);
}
#[test]fn plans_keep_task_and_step_identity_and_do_not_run(){
    let mut r=record();apply(&mut r,"task",proposal());let task=r.tasks[0].id.clone();
    apply(&mut r,"plan",Action::ProposePlan{task_id:task.clone(),steps:vec![StepInput{description:"Verificar a informação".into(),postcondition:"Fonte e resultado confirmados".into(),operation:None}]});
    assert_eq!(r.plans[0].task_id,task);assert_eq!(r.tasks[0].plan_id.as_deref(),Some(r.plans[0].id.as_str()));assert_eq!(r.plans[0].steps[0].state,WorkState::Proposed);assert_eq!(r.state,State::Draft);
    let before=serde_json::to_value(&r).unwrap();let mut scratch=r.clone();assert_error(scratch.apply("foreign-plan".into(),Action::ProposePlan{task_id:"foreign-task".into(),steps:vec![StepInput{description:"A".into(),postcondition:"B".into(),operation:None}]},100_004,nonce().unwrap()),Error::NotFound);assert_eq!(serde_json::to_value(&r).unwrap(),before);
}
#[test]fn cancellation_of_unexecuted_work_is_durable_and_terminal(){
    let mut r=record();apply(&mut r,"task",proposal());apply(&mut r,"pause",Action::Pause);assert_eq!(r.state,State::Paused);apply(&mut r,"resume",Action::ResumeDraft);assert_eq!(r.state,State::Draft);apply(&mut r,"cancel",Action::Cancel);assert_eq!(r.state,State::Cancelled);assert!(r.tasks.iter().all(|t|t.state==WorkState::Cancelled));assert_error(r.apply("new-task".into(),proposal(),100_005,nonce().unwrap()),Error::Conflict);
}
#[test]fn legacy_terminal_status_never_becomes_goal_completion_or_authority(){
    for phase in ["done","interrupted","resumed","working"]{let mut r=record();apply(&mut r,"import",Action::ImportLegacy{reference:Legacy{task_id:"old-task".into(),provider:"codex".into(),backend:"codex".into(),conversation_id:Some("cid-1".into()),phase:phase.into(),intent_may_be_incomplete:true,receipts:vec![]}});assert_eq!(r.state,State::RecoveryRequired);assert!(r.legacy[0].intent_may_be_incomplete);apply(&mut r,"cancel",Action::Cancel);assert_eq!(r.state,State::RecoveryRequired);assert_error(r.apply("resume".into(),Action::ResumeDraft,100_004,nonce().unwrap()),Error::Conflict);}
}
#[test]fn import_refuses_foreign_owner_and_fabricated_complete_intent(){
    for (provider,complete)in [("copilot",true),("codex",false)]{let mut r=record();assert_error(r.apply("import".into(),Action::ImportLegacy{reference:Legacy{task_id:"old".into(),provider:provider.into(),backend:"codex".into(),conversation_id:None,phase:"done".into(),intent_may_be_incomplete:complete,receipts:vec![]}},100_001,nonce().unwrap()),Error::InvalidRecord);}
}
#[test]fn malformed_versions_unknown_states_and_mismatched_graphs_are_rejected(){
    for alter in [0,1,2,3,4]{let mut value=serde_json::to_value(record()).unwrap();match alter{0=>value["schemaVersion"]=serde_json::json!(2),1=>value["state"]=serde_json::json!("completed"),2=>value["sequence"]=serde_json::json!(7),3=>value["spec"]["owner"]["provider"]=serde_json::json!("copilot"),_=>value["state"]=serde_json::json!("paused")};let decoded=serde_json::from_value::<Record>(value);assert!(decoded.is_err()||decoded.unwrap().validate().is_err());}
}
#[test]fn size_and_shape_limits_refuse_instead_of_truncating(){
    for alter in 0..5{let mut s=spec("valid");match alter{0=>s.intent="x".repeat(65537),1=>s.criteria=vec![],2=>s.criteria=vec!["x".into();33],3=>s.request_id="../escape".into(),_=>s.context=Context::Mochi{conversation_id:"../foreign".into()}};assert_error(s.validate(),Error::InvalidInput);}
    let mut r=record();for n in 0..128{apply(&mut r,&format!("task{n}"),proposal());}assert_error(r.apply("overflow".into(),proposal(),100_200,nonce().unwrap()),Error::LimitReached);assert_eq!(r.tasks.len(),128);
}
#[test]fn cancellation_has_a_reserved_event_slot_when_activity_is_full(){
    let mut r=record();for n in 2..MAX_EVENTS{r.events.push(Event{sequence:n as u64,at:100_000+n as u64,request_id:format!("event{n}"),fingerprint:hash(b"synthetic-old-event"),kind:EventKind::DraftResumed});}r.sequence=r.events.len() as u64;r.updated_at=r.events.last().unwrap().at;r.validate().unwrap();
    let now=r.updated_at+1;assert_error(r.apply("too-much-work".into(),proposal(),now,nonce().unwrap()),Error::LimitReached);r.apply("stop".into(),Action::Cancel,now,nonce().unwrap()).unwrap();assert_eq!(r.events.len(),MAX_EVENTS);assert_eq!(r.state,State::Cancelled);
}
#[test]fn clock_rollback_and_cross_pet_references_do_not_change_files(){
    let f=Fixture::new();let store=f.store();let r=store.create(spec("one"),100_000).unwrap().record;let bytes=fs::read(f.root.join(format!("{}.json",r.id))).unwrap();
    assert_error(store.mutate(&r.id,"agent_copilot",&r.revision,"foreign".into(),proposal(),100_001),Error::Conflict);assert_error(store.read(&r.id,"agent_copilot"),Error::Conflict);assert_error(store.mutate(&r.id,"agent_codex",&r.revision,"past".into(),proposal(),99_999),Error::ClockRollback);assert_eq!(fs::read(f.root.join(format!("{}.json",r.id))).unwrap(),bytes);
}
#[test]fn writers_and_readers_use_the_same_cross_instance_lock(){
    let f=Fixture::new();let first=f.store();let second=f.store();let guard=first.test_lock(true).unwrap();assert_error(second.create(spec("one"),100_000),Error::Busy);assert_error(second.list(None),Error::Busy);drop(guard);assert!(second.create(spec("one"),100_000).is_ok());
}
#[test]fn prepublication_failure_preserves_the_previous_revision(){
    let f=Fixture::new();let mut store=f.store();let r=store.create(spec("one"),100_000).unwrap().record;store.fault=Fault::BeforePublish;
    assert_error(store.mutate(&r.id,"agent_codex",&r.revision,"task".into(),proposal(),100_001),Error::StorageUnavailable);let after=store.read(&r.id,"agent_codex").unwrap();assert_eq!(after.revision,r.revision);assert!(after.tasks.is_empty());
}
#[test]fn postpublication_failure_returns_uncertainty_and_retry_does_not_duplicate(){
    let f=Fixture::new();let mut store=f.store();store.fault=Fault::AfterPublish;let spec=spec("one");let result=store.create(spec.clone(),100_000).unwrap();assert_eq!(result.status,CommitStatus::Uncertain);drop(store);
    let reopened=Store::existing(&f.root).unwrap().unwrap();let retry=reopened.create(spec,100_001).unwrap();assert_eq!(retry.status,CommitStatus::Unchanged);assert_eq!(retry.record.revision,result.record.revision);assert_eq!(reopened.list(None).unwrap().goals.len(),1);
}
#[test]fn symlinks_hardlinks_and_public_permissions_are_never_read_as_goals(){
    for attack in 0..3{let f=Fixture::new();let store=f.store();let r=store.create(spec("one"),100_000).unwrap().record;let path=f.root.join(format!("{}.json",r.id));let original=fs::read(&path).unwrap();let foreign=f.base.join("foreign");fs::write(&foreign,b"foreign-data").unwrap();
        match attack{0=>{fs::remove_file(&path).unwrap();symlink(&foreign,&path).unwrap();},1=>{fs::hard_link(&path,f.base.join("copy")).unwrap();},_=>fs::set_permissions(&path,fs::Permissions::from_mode(0o644)).unwrap()}
        assert_error(store.read(&r.id,"agent_codex"),Error::UnsafePath);assert!(store.create(spec("one"),100_001).is_err());assert_eq!(fs::read(&foreign).unwrap(),b"foreign-data");if attack!=0{assert_eq!(fs::read(&path).unwrap(),original);}
    }
}
#[test]fn directory_replacement_and_unknown_protocol_are_not_adopted(){
    let f=Fixture::new();let store=f.store();fs::rename(&f.root,f.base.join("old")).unwrap();fs::create_dir(&f.root).unwrap();fs::set_permissions(&f.root,fs::Permissions::from_mode(0o700)).unwrap();assert_error(store.list(None),Error::UnsafePath);assert_error(Store::existing(&f.root),Error::InvalidRecord);
}
#[test]fn partial_listing_preserves_unknown_artifacts_and_corrupt_records(){
    let f=Fixture::new();let store=f.store();let r=store.create(spec("one"),100_000).unwrap().record;fs::write(f.root.join(".tmp-interrupted"),b"unpublished").unwrap();fs::write(f.root.join(format!("g-{}.json","a".repeat(64))),b"broken").unwrap();
    let list=store.list(None).unwrap();assert_eq!(list.goals.len(),1);assert_eq!(list.unavailable,1);assert_eq!(list.pending_artifacts,1);assert!(!list.complete);assert!(f.root.join(".tmp-interrupted").exists());assert_eq!(store.read(&r.id,"agent_codex").unwrap().state,State::Draft);
}
#[test]fn goals_are_bounded_without_deleting_the_oldest_record(){
    let f=Fixture::new();let store=f.store();let first=store.create(spec("first"),100_000).unwrap().record;
    for n in 1..MAX_GOALS{store.create(spec(&format!("goal-{n}")),100_000+n as u64).unwrap();}
    assert_error(store.create(spec("overflow"),101_000),Error::LimitReached);assert_eq!(store.list(None).unwrap().goals.len(),MAX_GOALS);assert_eq!(store.read(&first.id,"agent_codex").unwrap().spec.request_id,"first");
}
#[test]fn native_draft_input_cannot_inject_execution_authority_or_legacy_receipts(){
    assert!(serde_json::from_value::<super::adapter::CreateInput>(serde_json::json!({"petId":"agent_codex","requestId":"one","intent":"work","criteria":["verify"],"conversationId":null,"authorization":"approved"})).is_err());
    assert!(serde_json::from_value::<super::adapter::DraftAction>(serde_json::json!({"kind":"importLegacy","reference":{}})).is_err());
    assert!(serde_json::from_value::<super::adapter::DraftAction>(serde_json::json!({"kind":"run"})).is_err());
    for label in [crate::HUB_LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL,crate::companions::LABEL]{assert!(super::adapter::allowed(label));}for label in ["website","pet-chat","island-unknown"]{assert!(!super::adapter::allowed(label));}
}

#[test]fn mutation_after_uncertain_publication_is_reconciled_without_new_steps(){
    let f=Fixture::new();let mut store=f.store();let original=store.create(spec("one"),100_000).unwrap().record;
    store.fault=Fault::AfterPublish;let uncertain=store.mutate(&original.id,"agent_codex",&original.revision,"task".into(),proposal(),100_001).unwrap();assert_eq!(uncertain.status,CommitStatus::Uncertain);
    let reopened=Store::existing(&f.root).unwrap().unwrap();let same=reopened.mutate(&original.id,"agent_codex",&original.revision,"task".into(),proposal(),100_002).unwrap();assert_eq!(same.status,CommitStatus::Unchanged);assert_eq!(same.record.tasks.len(),1);assert_eq!(same.record.events.len(),2);assert_eq!(same.record.revision,uncertain.record.revision);
}

#[test]fn durability_confirmation_includes_the_parent_of_an_existing_repository(){
    let f=Fixture::new();let store=f.store();let original=store.create(spec("one"),100_000).unwrap().record;
    fs::set_permissions(&f.base,fs::Permissions::from_mode(0o777)).unwrap();
    let result=store.mutate(&original.id,"agent_codex",&original.revision,"task".into(),proposal(),100_001).unwrap();
    assert_eq!(result.status,CommitStatus::Uncertain);fs::set_permissions(&f.base,fs::Permissions::from_mode(0o700)).unwrap();
    let reopened=Store::existing(&f.root).unwrap().unwrap();let retry=reopened.mutate(&original.id,"agent_codex",&original.revision,"task".into(),proposal(),100_002).unwrap();assert_eq!(retry.status,CommitStatus::Unchanged);assert_eq!(retry.record.tasks.len(),1);
}
