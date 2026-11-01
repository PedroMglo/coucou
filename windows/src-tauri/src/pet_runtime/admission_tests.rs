use super::{admission::*,admission_review::{Reviews,ConfirmInput},context,model::*,store::{Store,Fault,nonce}};
use crate::{chat::Chat,pet_capabilities::*};
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt};
pub(super)struct Fixture{pub(super)path:PathBuf}
impl Fixture{
 pub(super)fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-admission-{}",nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();Self{path}}
 pub(super)fn store(&self)->Store{Store::ensure(&self.path.join("goals")).unwrap()}
 pub(super)fn goal(&self,provider:&str)->Record{self.goal_named(provider,&format!("goal-{provider}"))}
 pub(super)fn goal_named(&self,provider:&str,request:&str)->Record{
  self.goal_named_criteria(provider,request,vec!["Verified original result".into()])
 }
 pub(super)fn goal_named_criteria(&self,provider:&str,request:&str,criteria:Vec<String>)->Record{
  let pet=crate::pet_config::pet_id(provider).unwrap();let adapter=crate::pet_config::blueprint(pet).unwrap().runtime.adapter.unwrap().key();
  let spec=Spec{restored_from:None,request_id:request.into(),owner:Owner{pet_id:pet.into(),provider:provider.into(),adapter:adapter.into(),model_hint:None},context:Context::Unbound,intent:"Original complete intent".into(),criteria};
  let store=self.store();let g=store.create(spec,100_000).unwrap().record;let main=Chat::persistent(self.path.join("main.json"));let(b,c)=context::prepare(&self.path.join("contexts"),&g,&main,PreparationMode::Dedicated,"context").unwrap();drop(c);
  store.mutate(&g.id,pet,&g.revision,"context".into(),Action::BindContext{binding:b},100_001).unwrap().record
 }
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
pub(super)fn descriptor(g:&Record,now:u64)->Descriptor{Descriptor{schema_version:1,pet_id:g.spec.owner.pet_id.clone(),kind:Kind::Llm,name:"Fixture".into(),lifecycle:Lifecycle::Active,binding:Binding{engine_binding:None,adapter:Some(crate::pet_config::blueprint(&g.spec.owner.pet_id).unwrap().runtime.adapter.unwrap().key()),provider:Some(g.spec.owner.provider.clone()),model:Some("fixture-free".into()),profile:None},revision:"fixture-descriptor".into(),observed_at:Some(now),configuration_available:true,capabilities:vec![Capability{id:"conversation.respond",support:Support::Supported,availability:Availability::Ready,reason:None,source:vec![Source::RuntimeObservation],observed_at:Some(now),expires_at:Some(now+15_000),model_binding:Some("fixture-free".into()),decision:Decision::NativeApproval,callers:vec![Caller::TaskRuntime],limits:Limits::default(),limitations:vec![]}],concurrency:Concurrency{scope:Some("logicalProvider"),max_active:Some(1)}}}
pub(super)fn evidence(g:&Record,now:u64)->Evidence{Evidence::observe(g,&descriptor(g,now),"a".repeat(64),now,false,false).unwrap()}
pub(super)fn budget()->Budget{Budget{deadline_at:200_000,max_model_calls:3,max_tool_calls:2,max_output_bytes:4096,max_tokens:None,max_cost_micros:None,payment:Payment::ExistingAccount}}
pub(super)fn review(g:&Record,request:&str,now:u64)->Review{Review::prepare(g,evidence(g,now),budget(),request.into(),nonce().unwrap(),now).unwrap()}
pub(super)fn approved(g:&Record,request:&str)->Approved{review(g,request,100_002).approve(g,evidence(g,100_003),100_003).unwrap()}
fn err<T>(r:std::result::Result<T,Denial>,e:Denial){assert_eq!(r.err(),Some(e));}

#[test]fn queued_original_intent_budget_and_owner_survive_reopen_without_dispatch(){
 let f=Fixture::new();let g=f.goal("codex");let a=approved(&g,"queue");let r=f.store().enqueue(&a,100_004).unwrap();assert_eq!(r.status,CommitStatus::Durable);assert_eq!(r.record.schema_version,6);assert_eq!(r.record.state,State::Queued);assert_eq!(r.record.spec,g.spec);assert_eq!(r.record.admissions.len(),1);assert_eq!(r.record.admissions[0].spec.budget,budget());
 let store=Store::existing(&f.path.join("goals")).unwrap().unwrap();let reopened=store.read(&g.id,"agent_codex").unwrap();assert_eq!(reopened.state,State::Queued);assert_eq!(reopened.admissions[0].spec.resolved_model,"fixture-free");assert!(!f.path.join("main.json").exists());
}
#[test]fn review_retains_original_deadline_and_requires_fresh_exact_observation(){
 let f=Fixture::new();let g=f.goal("codex");let r=review(&g,"queue",100_002);assert_eq!(r.proposal().intent,g.spec.intent);assert_eq!(r.proposal().budget.deadline_at,200_000);
 let a=r.approve(&g,evidence(&g,130_000),130_000).unwrap();assert_eq!(a.spec().approved_at,130_000);assert_eq!(a.spec().budget.deadline_at,200_000);assert!(f.store().enqueue(&a,145_000).is_err());
}
#[test]fn stale_unresolved_inactive_or_ambiguous_capabilities_cannot_admit(){
 let f=Fixture::new();let g=f.goal("codex");let mut d=descriptor(&g,100_002);d.binding.model=Some("@default".into());err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::ModelUnresolved);
 for model in [" "," fixture-free","fixture-free ","https://remote/model"]{let mut d=descriptor(&g,100_002);d.binding.model=Some(model.into());err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::ModelUnresolved);}
 let mut d=descriptor(&g,100_002);d.lifecycle=Lifecycle::Inactive;err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::Inactive);
 let mut d=descriptor(&g,100_002);d.capabilities.push(d.capabilities[0].clone());err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::Unsupported);
 let d=descriptor(&g,100_002);err(Evidence::observe(&g,&d,"a".repeat(64),115_002,false,false),Denial::Stale);
 let mut d=descriptor(&g,100_002);d.capabilities[0].availability=Availability::Unknown;err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::NotReady);
}
#[test]fn owner_model_context_and_policy_changes_revoke_a_review(){
 let f=Fixture::new();let g=f.goal("codex");let r=review(&g,"queue",100_002);let mut d=descriptor(&g,100_003);d.binding.model=Some("different-fixture".into());d.capabilities[0].model_binding=d.binding.model.clone();let e=Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false).unwrap();err(r.approve(&g,e,100_003),Denial::RevisionChanged);
 let r=review(&g,"queue",100_002);let e=Evidence::observe(&g,&descriptor(&g,100_003),"b".repeat(64),100_003,false,false).unwrap();err(r.approve(&g,e,100_003),Denial::PolicyChanged);
 let mut d=descriptor(&g,100_003);d.binding.provider=Some("copilot".into());err(Evidence::observe(&g,&d,"a".repeat(64),100_003,false,false),Denial::OwnerChanged);
 let mut changed=g.clone();changed.execution_context.as_mut().unwrap().seed_hash="c".repeat(64);let r=review(&g,"queue",100_002);let e=evidence(&changed,100_003);err(r.approve(&changed,e,100_003),Denial::RevisionChanged);
}
#[test]fn expired_and_changed_revisions_cannot_enqueue_or_extend_a_review(){
 let f=Fixture::new();let g=f.goal("codex");let r=review(&g,"queue",100_002);err(r.approve(&g,evidence(&g,200_000),200_000),Denial::Expired);
 let a=approved(&g,"queue");let changed=f.store().mutate(&g.id,"agent_codex",&g.revision,"task".into(),Action::ProposeTask{description:"new task".into()},100_004).unwrap().record;assert!(f.store().enqueue(&a,100_005).is_err());assert_eq!(f.store().read(&g.id,"agent_codex").unwrap().revision,changed.revision);
}
#[test]fn budget_payment_and_unmeasurable_caps_are_never_silently_relaxed(){
 let f=Fixture::new();let g=f.goal("codex");let mut b=budget();b.max_model_calls=0;err(Review::prepare(&g,evidence(&g,100_002),b,"q".into(),nonce().unwrap(),100_002),Denial::Budget);
 let mut b=budget();b.payment=Payment::LocalOnly;err(Review::prepare(&g,evidence(&g,100_002),b,"q".into(),nonce().unwrap(),100_002),Denial::PaymentPolicy);
 let mut b=budget();b.max_cost_micros=Some(1);err(Review::prepare(&g,evidence(&g,100_002),b,"q".into(),nonce().unwrap(),100_002),Denial::MeteringUnavailable);
 let mut b=budget();b.max_tokens=Some(1);err(Review::prepare(&g,evidence(&g,100_002),b,"q".into(),nonce().unwrap(),100_002),Denial::MeteringUnavailable);
 let local=f.goal("ollama");let mut b=budget();b.payment=Payment::LocalOnly;assert!(Review::prepare(&local,evidence(&local,100_002),b,"q".into(),nonce().unwrap(),100_002).is_ok());
}
#[test]fn uncertain_publication_retry_is_data_only_and_does_not_duplicate_admission(){
 let f=Fixture::new();let g=f.goal("codex");let a=approved(&g,"queue");let mut store=f.store();store.fault=Fault::AfterPublish;let r=store.enqueue(&a,100_004).unwrap();assert_eq!(r.status,CommitStatus::Uncertain);store.fault=Fault::None;
 let again=store.enqueue(&a,300_000).unwrap();assert_eq!(again.status,CommitStatus::Unchanged);assert_eq!(again.record.admissions.len(),1);assert_eq!(again.record.admissions[0].spec.budget.deadline_at,200_000);
}
#[test]fn prepublication_failure_leaves_original_draft_and_retry_uses_same_budget(){
 let f=Fixture::new();let g=f.goal("codex");let a=approved(&g,"queue");let mut store=f.store();store.fault=Fault::BeforePublish;assert!(store.enqueue(&a,100_004).is_err());let r=store.read(&g.id,"agent_codex").unwrap();assert_eq!(r.state,State::Draft);assert_eq!(r.revision,g.revision);assert!(r.admissions.is_empty());store.fault=Fault::None;assert_eq!(store.enqueue(&a,100_005).unwrap().record.admissions.len(),1);
}
#[test]fn pause_resume_and_readmission_preserve_all_original_decisions(){
 let f=Fixture::new();let g=f.goal("codex");let mut r=f.store().enqueue(&approved(&g,"queue"),100_004).unwrap().record;
 r=f.store().mutate(&g.id,"agent_codex",&r.revision,"pause".into(),Action::Pause,100_005).unwrap().record;assert_eq!(r.state,State::Paused);assert_eq!(r.admissions[0].phase,QueuePhase::Paused);
 r=f.store().mutate(&g.id,"agent_codex",&r.revision,"resume".into(),Action::ResumeDraft,100_006).unwrap().record;let a=review(&r,"queue-two",100_007).approve(&r,evidence(&r,100_008),100_008).unwrap();r=f.store().enqueue(&a,100_009).unwrap().record;assert_eq!(r.admissions.len(),2);assert_eq!(r.admissions[0].phase,QueuePhase::Paused);assert_eq!(r.admissions[1].phase,QueuePhase::Queued);
 r=f.store().mutate(&g.id,"agent_codex",&r.revision,"cancel".into(),Action::Cancel,100_010).unwrap().record;assert_eq!(r.state,State::Cancelled);assert_eq!(r.admissions[1].phase,QueuePhase::Cancelled);assert!(r.validate().is_ok());
}
#[test]fn review_tokens_are_scoped_single_use_and_retries_do_not_reset_deadlines(){
 let f=Fixture::new();let g=f.goal("codex");let mut cache=Reviews::default();let ticket=cache.insert("window",review(&g,"queue",100_002),nonce().unwrap(),100_002).unwrap();let retry=cache.insert("window",review(&g,"queue",100_003),nonce().unwrap(),100_003).unwrap();assert_eq!(ticket.token,retry.token);assert_eq!(ticket.proposal.expires_at,retry.proposal.expires_at);
 let input=ConfirmInput{memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token,approve:true};assert!(cache.take("foreign",&input,100_004).is_err());let review=cache.take("window",&input,100_004).unwrap();assert!(cache.take("window",&input,100_004).is_err());assert!(review.approve(&g,evidence(&g,100_004),100_004).is_ok());
}
#[test]fn all_configured_native_pet_and_service_queue_owners_remain_independent(){
 let f=Fixture::new();for provider in ["codex","copilot","antigravity","ollama","integration_notion"]{let g=f.goal(provider);let r=f.store().enqueue(&approved(&g,"queue"),100_004).unwrap().record;assert_eq!(r.admissions[0].spec.owner.provider,provider);assert_eq!(r.admissions[0].spec.context.provider,provider);}assert_eq!(f.store().list(None).unwrap().goals.len(),5);
}
#[test]fn serialized_proposals_or_loaded_records_cannot_construct_an_approval_action(){
 assert!(serde_json::from_value::<super::adapter::DraftAction>(serde_json::json!({"kind":"enqueue","approval":{"always":true}})).is_err());
 assert!(serde_json::from_value::<super::admission_review::PrepareInput>(serde_json::json!({"petId":"agent_codex","goalId":"g","revision":"r","requestId":"q","budget":budget(),"grant":true})).is_err());
 let f=Fixture::new();let g=f.goal("codex");let r=f.store().enqueue(&approved(&g,"queue"),100_004).unwrap().record;let mut v=serde_json::to_value(&r).unwrap();v["admissions"][0]["spec"]["resolvedModel"]=serde_json::json!("tampered");let tampered:Record=serde_json::from_value(v).unwrap();assert!(tampered.validate().is_err());
}

#[test]fn review_origin_target_and_expiration_cannot_be_reused(){
 let f=Fixture::new();let g=f.goal("codex");let mut cache=Reviews::default();let ticket=cache.insert("window",review(&g,"queue",100_002),nonce().unwrap(),100_002).unwrap();
 let mut input=ConfirmInput{memory_consent:None,pet_id:"agent_copilot".into(),goal_id:g.id.clone(),revision:g.revision.clone(),token:ticket.token.clone(),approve:true};assert!(cache.take("window",&input,100_003).is_err());input.pet_id=g.spec.owner.pet_id.clone();assert!(cache.take("window",&input,ticket.proposal.expires_at).is_err());assert!(cache.take("window",&input,100_003).is_err());
}
#[test]fn review_capacity_refuses_new_entries_without_evicting_live_decisions(){
 let f=Fixture::new();let g=f.goal("codex");let mut cache=Reviews::default();let mut first=None;
 for n in 0..64{let ticket=cache.insert(&format!("window-{n}"),review(&g,&format!("queue-{n}"),100_002),nonce().unwrap(),100_002).unwrap();if n==0{first=Some(ticket);}}
 assert!(matches!(cache.insert("overflow",review(&g,"overflow",100_002),nonce().unwrap(),100_002),Err(Error::LimitReached)));
 let t=first.unwrap();let input=ConfirmInput{memory_consent:None,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),revision:g.revision.clone(),token:t.token,approve:false};assert!(cache.take("window-0",&input,100_003).is_ok());
}
#[test]fn admission_history_limit_preserves_intent_and_all_previous_decisions(){
 let f=Fixture::new();let mut r=f.goal("codex");let original=r.spec.clone();let mut now=100_002;
 for n in 0..32{let a=review(&r,&format!("queue-{n}"),now).approve(&r,evidence(&r,now+1),now+1).unwrap();r=f.store().enqueue(&a,now+2).unwrap().record;r=f.store().mutate(&r.id,"agent_codex",&r.revision,format!("pause-{n}"),Action::Pause,now+3).unwrap().record;r=f.store().mutate(&r.id,"agent_codex",&r.revision,format!("resume-{n}"),Action::ResumeDraft,now+4).unwrap().record;now+=5;}
 let a=review(&r,"overflow",now).approve(&r,evidence(&r,now+1),now+1).unwrap();assert!(matches!(f.store().enqueue(&a,now+2),Err(Error::LimitReached)));let saved=f.store().read(&r.id,"agent_codex").unwrap();assert_eq!(saved.admissions.len(),32);assert_eq!(saved.spec,original);assert_eq!(saved.revision,r.revision);
}

#[test]fn refreshing_only_the_observation_epoch_can_confirm_the_original_review_without_renewing_limits(){
 let f=Fixture::new();let g=f.goal("ollama");let r=review(&g,"refresh-observation",100_002);let original=r.proposal();let mut d=descriptor(&g,130_020);d.revision="same-capabilities-new-observation-epoch".into();let fresh=Evidence::observe(&g,&d,"a".repeat(64),130_020,false,false).unwrap();let approved=r.approve(&g,fresh,130_020).unwrap();let spec=approved.spec();assert_eq!(spec.descriptor_revision,original.descriptor_revision);assert_eq!(spec.source_revision,original.revision);assert_eq!(spec.budget,original.budget);assert_eq!(spec.resolved_model,original.resolved_model);assert_eq!(spec.owner,original.owner);assert_eq!(spec.context,original.context);assert_eq!(spec.approved_at,130_020);assert!(!approved.current(&g,original.expires_at).is_ok());
}
#[test]fn refreshing_metadata_never_accepts_changed_capability_limits_callers_decision_or_profile(){
 let f=Fixture::new();let g=f.goal("ollama");for mode in 0..4{let r=review(&g,&format!("changed-semantics-{mode}"),100_002);let mut d=descriptor(&g,100_020);d.revision="fresh-observation-epoch".into();match mode{0=>d.capabilities[0].limits.max_calls=Some(2),1=>d.capabilities[0].callers.push(Caller::ProviderModel),2=>d.capabilities[0].decision=Decision::ScopedGrant,_=>d.binding.profile=Some("different-profile".into())};let fresh=Evidence::observe(&g,&d,"a".repeat(64),100_020,false,false).unwrap();err(r.approve(&g,fresh,100_020),Denial::RevisionChanged);}
}
