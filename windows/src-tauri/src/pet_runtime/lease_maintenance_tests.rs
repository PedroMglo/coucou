use super::*;
use crate::{chat::Chat,pet_runtime::{admission_tests::{Fixture,evidence},authority_tests::{root,queued},authority_store::mutate_with_authority,lease_controller::Validity,model::{Action,State},store::Fault}};
fn setup()->(Fixture,Authority,Record,Chat,Service){
    let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let g=queued(&f,&f.goal("ollama"),&a);let main=Chat::persistent(f.path.join("live.json"));let controller=Controller::new().unwrap();
    let pending=controller.prepare(&g,&root(&f),&f.path.join("contexts"),&main,&evidence(&g,100_006),"original-claim".into(),100_006,Arc::new(|_,_,_|Validity::Current)).unwrap();
    let mut entries=BTreeMap::new();entries.insert(g.id.clone(),Entry{source_revision:g.revision.clone(),request_id:"original-claim".into(),pending,owned:None});(f,a,g,main,Service{controller,entries,maintenance_after:None,waiting:BTreeMap::new(),dispatch_after:None})
}
#[test]fn deterministic_sweep_keeps_current_preparation_then_cleans_expiry_without_any_followup_request(){
    let(f,_a,g,main,mut service)=setup();sweep(&mut service,&f.store(),&root(&f),100_007);assert_eq!(service.entries.len(),1);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().revision,g.revision);
    sweep(&mut service,&f.store(),&root(&f),200_001);assert!(service.entries.is_empty());assert!(main.runtime_resources().busy().unwrap().is_empty());let ended=f.store().read(&g.id,"ai_ollama").unwrap();assert_eq!(ended.state,State::RecoveryRequired);assert_eq!(ended.admissions[0].stop_reason,Some(crate::pet_runtime::leases::StopReason::Expired));
}
#[test]fn paused_or_cancelled_preparation_is_retired_without_reactivating_or_reacquiring(){
    for action in [Action::Pause,Action::Cancel]{
        let(f,_a,g,main,mut service)=setup();let expected=if matches!(action,Action::Pause){State::Paused}else{State::Cancelled};mutate_with_authority(&f.store(),&root(&f),&g.id,"ai_ollama",&g.revision,"stop".into(),action,100_008).unwrap();
        sweep(&mut service,&f.store(),&root(&f),100_009);assert!(service.entries.is_empty());assert!(main.runtime_resources().busy().unwrap().is_empty());assert_eq!(f.store().read(&g.id,"ai_ollama").unwrap().state,expected);
    }
}
#[test]fn uncertain_unstarted_cleanup_retains_original_native_entry_until_durable_ack(){
    for fault in [Fault::BeforePublish,Fault::AfterPublish]{
        let(f,_a,g,main,mut service)=setup();let mut store=f.store();store.fault=fault;let lease=service.entries[&g.id].pending.lease().clone();sweep(&mut service,&store,&root(&f),200_001);assert_eq!(service.entries[&g.id].pending.lease(),&lease);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());
        sweep(&mut service,&f.store(),&root(&f),200_002);assert!(service.entries.is_empty());assert!(main.runtime_resources().busy().unwrap().is_empty());let ended=f.store().read(&g.id,"ai_ollama").unwrap();assert_eq!(ended.admissions[0].lease.as_ref(),Some(&lease));assert_eq!(ended.events.iter().filter(|e|matches!(e.kind,model::EventKind::ClaimAbandoned)).count(),1);
    }
}
#[test]fn cleanup_does_not_overwrite_a_new_draft_or_admission_from_a_historical_pause(){
    let(f,a,g,main,mut service)=setup();let paused=mutate_with_authority(&f.store(),&root(&f),&g.id,"ai_ollama",&g.revision,"pause".into(),Action::Pause,100_008).unwrap().record;let draft=f.store().mutate(&g.id,"ai_ollama",&paused.revision,"resume".into(),Action::ResumeDraft,100_009).unwrap().record;
    let review=crate::pet_runtime::admission::Review::prepare(&draft,evidence(&draft,100_010),g.admissions[0].spec.budget.clone(),"fresh-admit".into(),crate::pet_runtime::store::nonce().unwrap(),100_010).unwrap();let approved=review.approve(&draft,evidence(&draft,100_011),100_011).unwrap();let certified=a.certify(&approved,&draft,100_012).unwrap();let next=f.store().enqueue_certified(&certified,100_013).unwrap().record;
    let file=f.path.join("goals").join(format!("{}.json",g.id));let bytes=std::fs::read(&file).unwrap();sweep(&mut service,&f.store(),&root(&f),200_001);assert_eq!(std::fs::read(&file).unwrap(),bytes);assert_eq!(service.entries.len(),1);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());assert!(a.verify_queued(&next,100_014).is_ok());
}

#[test]fn deterministic_runtime_unconfigured_engine_api_and_service_pending_cleanup_uses_captured_pet_not_task_provider_lookup(){
 use crate::pet_runtime::deterministic_runtime_tests::{draft,queued,evidence};
 for provider in ["openai","integration_spotify","lmstudio"]{
  let f=Fixture::new();let path=root(&f);let a=Authority::ensure(&path).unwrap();let g=queued(&f,&a,&draft(&f,provider));let now=crate::assistant_memory::now();let main=Chat::persistent(f.path.join("native-main.json"));let controller=Controller::new().unwrap();let pending=controller.prepare(&g,&path,&f.path.join("contexts"),&main,&evidence(&g,now),"native-pending".into(),now,Arc::new(|_,_,_|Validity::Current)).unwrap();assert_eq!(pending.pet_id(),g.spec.owner.pet_id);
  let mut entries=BTreeMap::new();entries.insert(g.id.clone(),Entry{source_revision:g.revision.clone(),request_id:"native-pending".into(),pending,owned:None});let mut service=Service{controller,entries,maintenance_after:None,waiting:BTreeMap::new(),dispatch_after:None};let deadline=g.admissions[0].spec.budget.deadline_at;sweep(&mut service,&f.store(),&path,deadline+1);assert!(service.entries.is_empty(),"{provider}");assert!(main.runtime_resources().busy().unwrap().is_empty());let ended=f.store().read(&g.id,&g.spec.owner.pet_id).unwrap();assert_eq!(ended.state,State::RecoveryRequired);assert_eq!(ended.admissions[0].spec.budget.max_model_calls,0);assert_eq!(ended.admissions[0].spec.schema_version,4);
 }
}

#[test]fn stopped_claimed_manual_owner_sweep_waits_workers_then_releases_only_after_durable_original_close(){
 for action in [Action::Pause,Action::Cancel]{for fault in [Fault::BeforePublish,Fault::AfterPublish]{
  let(f,a,g,main,mut service)=setup();let entry=service.entries.get_mut(&g.id).unwrap();let(leased,owner)=entry.pending.claim(&f.store(),&a,&evidence(&g,100_007),100_007).unwrap();let owner=Arc::new(owner.unwrap());entry.owned=Some(owner.clone());let worker=owner.readiness_worker(100_007).unwrap();
  let expected=if matches!(action,Action::Pause){State::Paused}else{State::Cancelled};let stopped=mutate_with_authority(&f.store(),&root(&f),&g.id,"ai_ollama",&leased.record.revision,"owned-stop".into(),action.clone(),100_008).unwrap().record;
  sweep(&mut service,&f.store(),&root(&f),100_009);assert_eq!(service.entries.len(),1);assert_eq!(f.store().read_control(&g.id,"ai_ollama").unwrap().state,State::Stopping);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());
  drop(worker);let mut broken=f.store();broken.fault=fault;sweep(&mut service,&broken,&root(&f),100_010);assert_eq!(service.entries.len(),1);assert!(main.runtime_resources().acquire("ollama",Arc::default(),None).is_err());
  sweep(&mut service,&f.store(),&root(&f),100_011);assert!(service.entries.is_empty());assert!(main.runtime_resources().busy().unwrap().is_empty());let ended=f.store().read_control(&g.id,"ai_ollama").unwrap();assert_eq!(ended.state,expected);assert_eq!(ended.admissions[0].spec.decision_id,stopped.admissions[0].spec.decision_id);assert_eq!(ended.events.iter().filter(|e|matches!(e.kind,model::EventKind::LeaseDrained)).count(),1);
 }}
}
