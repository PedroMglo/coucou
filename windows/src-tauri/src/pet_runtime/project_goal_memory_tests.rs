use super::*;
use crate::pet_memory::{context::Capture,context_model::{SelectedProject,GoalConsent},projects::Project};
use crate::pet_runtime::{admission::{Evidence,Review,Payment},admission_tests,memory_review::Captured};
struct ProjectFixture{files:Fixture,main:Chat,goal:Record,project:Project,row:Value}
impl ProjectFixture{
 fn new()->Self{
  let files=Fixture::new();let main=files.main();let public=main.conversation_id("ollama").unwrap();main.retain_public_conversation("ollama",&public).unwrap();
  let project=Project{id:crate::pet_memory::context_model::digest(&serde_json::to_vec(&("coucou-memory-project-v1","ai_ollama","own-project-choice")).unwrap())[..32].into(),pet_id:"ai_ollama".into(),creation_request_id:"own-project-choice".into(),revision:1,name:"Selected own project".into(),conversation_ids:vec![public.clone()],enabled:true,created_at:100,updated_at:100};
  let mut row=Fixture::record("project-note","ai_ollama",None);row["owner"]["scope"]=json!({"kind":"project","projectId":project.id});row["content"]=json!("PRIVATE_GOAL_PROJECT_NOTE");
  let goal=Fixture::goal(Context::Mochi{conversation_id:public});let f=Self{files,main,goal,project,row};f.write();f
 }
 fn write(&self){fs::write(self.files.catalog(),serde_json::to_vec(&json!({"version":2,"generation":1,"projects":[self.project],"items":[self.row]})).unwrap()).unwrap();fs::set_permissions(self.files.catalog(),fs::Permissions::from_mode(0o600)).unwrap();}
 fn choice(&self)->SelectedProject{SelectedProject{id:self.project.id.clone(),revision:self.project.revision}}
 fn prepare(&self)->Result<(Binding,Chat)>{prepare_project_fixture(&self.files.contexts(),&self.goal,&self.main,"project-context",&[Fixture::selected("project-note")],&self.choice(),&self.files.catalog())}
 fn bind(&mut self){let(b,c)=self.prepare().unwrap();drop(c);Fixture::bind(&mut self.goal,b);}
 fn server(&self)->crate::local_chat::Server{crate::local_chat::Server{id:"ollama",name:"Own synthetic",url:"http://127.0.0.1:12345".into(),key:None}}
 fn evidence(&self,now:u64)->Evidence{let capture=Captured::at(&self.files.catalog(),&self.goal,&self.server(),"fixture-free",now).unwrap();Evidence::from_memory_capture(&self.goal,&admission_tests::descriptor(&self.goal,now),"a".repeat(64),now,false,false,capture).unwrap()}
}
#[test]fn project_goal_binding_uses_original_native_source_without_copying_history_and_pins_exact_project(){
 let mut f=ProjectFixture::new();let t=f.main.begin_cli("ollama",Arc::new(AtomicBool::new(false))).unwrap();let lease=f.main.lease(&t);f.main.commit_reply(&t,json!({"role":"user","content":"UNSELECTED_SOURCE_DIALOGUE"}),json!({"role":"assistant","content":"Unselected source reply"}),"UNSELECTED_SOURCE_DIALOGUE","Unselected source reply").unwrap();drop(lease);
 f.bind();let b=f.goal.execution_context.as_ref().unwrap();assert_eq!(b.mode,Mode::Dedicated);assert_eq!(b.source_messages,0);assert_eq!(b.copied_messages,0);assert_ne!(b.conversation_id,b.source_conversation_id.clone().unwrap());assert!(b.project_memory());let selection=b.memory.as_ref().unwrap();assert_eq!(selection.schema_version,2);assert_eq!(selection.project.as_deref(),Some(&f.project));assert!(selection.valid("ai_ollama",b.source_conversation_id.as_deref()));let mut old=selection.clone();old.schema_version=1;assert!(!old.valid("ai_ollama",b.source_conversation_id.as_deref()));
 let(_,private)=open_at(&f.files.contexts(),&f.goal,&f.main,&f.files.catalog()).unwrap();assert!(private.history("ollama").is_empty());let header=fs::read_to_string(f.files.contexts().join(&f.goal.id).join(HEADER)).unwrap();assert!(!header.contains("PRIVATE_GOAL_PROJECT_NOTE")&&!header.contains("UNSELECTED_SOURCE_DIALOGUE"));assert!(header.contains("Selected own project"));
}
#[test]fn project_goal_wrong_source_choice_and_idempotency_cannot_create_or_replace_a_context(){
 let mut f=ProjectFixture::new();let(b,c)=f.prepare().unwrap();drop(c);let path=f.files.contexts().join(&f.goal.id).join(HEADER);let before=fs::read(&path).unwrap();let wrong=SelectedProject{id:f.project.id.clone(),revision:2};assert!(prepare_project_fixture(&f.files.contexts(),&f.goal,&f.main,"project-context",&[Fixture::selected("project-note")],&wrong,&f.files.catalog()).is_err());assert_eq!(fs::read(path).unwrap(),before);Fixture::bind(&mut f.goal,b);
 for source in [None,Some("forged-private-cid")]{let mut fresh=ProjectFixture::new();fresh.goal.spec.context=source.map(|id|Context::Mochi{conversation_id:id.into()}).unwrap_or(Context::Unbound);assert!(fresh.prepare().is_err());assert!(!fresh.files.contexts().join(&fresh.goal.id).exists());}
 let mut f=ProjectFixture::new();f.project.conversation_ids.clear();f.write();assert!(f.prepare().is_err());
}
#[test]fn project_full_metadata_is_revalidated_after_restart_even_when_revision_number_is_unchanged(){
 let mut f=ProjectFixture::new();f.bind();let before=fs::read(f.files.contexts().join(&f.goal.id).join(HEADER)).unwrap();let original=fs::read(f.files.catalog()).unwrap();f.project.name="UNREVIEWED_PROJECT_NAME_SAME_REVISION".into();f.write();assert!(open_at(&f.files.contexts(),&f.goal,&f.main,&f.files.catalog()).is_err());assert!(Captured::at(&f.files.catalog(),&f.goal,&f.server(),"fixture-free",crate::assistant_memory::now()).is_err());assert_eq!(fs::read(f.files.contexts().join(&f.goal.id).join(HEADER)).unwrap(),before);fs::write(f.files.catalog(),original).unwrap();assert!(open_at(&f.files.contexts(),&f.goal,&f.main,&f.files.catalog()).is_ok());assert!(f.goal.admissions.is_empty(),"restoring data grants no authority");
}
#[test]fn project_goal_has_own_data_review_and_existing_signed_budget_contract_and_refuses_expansion(){
 let mut f=ProjectFixture::new();f.bind();let now=crate::assistant_memory::now();let captured=Captured::at(&f.files.catalog(),&f.goal,&f.server(),"fixture-free",now).unwrap().unwrap();assert_eq!(captured.preview().schema_version,12);assert_eq!(captured.preview().project.as_deref(),Some(&f.project));assert_eq!(captured.grant().schema_version,1);assert!(!captured.peer_forwarding_supported());assert!(captured.confirm(&f.goal,&f.server(),"fixture-free",None,now).is_err());
 let mut budget=admission_tests::budget();budget.deadline_at=now+60_000;budget.max_model_calls=1;budget.max_tool_calls=0;budget.payment=Payment::LocalOnly;
 let review=Review::prepare(&f.goal,f.evidence(now),budget,"one-own-project-response".into(),nonce().unwrap(),now).unwrap();let approved=review.approve_with_data(&f.goal,f.evidence(now+1),Some(GoalConsent::BoundedGoalContext),now+1).unwrap();assert!(approved.spec().binding_format());assert_eq!(approved.spec().schema_version,2);assert_eq!(approved.spec().context.memory.as_ref().unwrap().project.as_deref(),Some(&f.project));
 for change in ["models","tools","payment"]{let mut wrong=approved.spec().clone();match change{"models"=>wrong.budget.max_model_calls=2,"tools"=>wrong.budget.max_tool_calls=1,_=>wrong.budget.payment=Payment::ExistingAccount};assert!(!wrong.binding_format(),"{change}");}
}
#[test]fn project_correspondence_preserves_the_exact_project_without_disclosing_its_metadata(){
 use crate::pet_runtime::{planning::{Catalog,Limits},planned_peer_data::Snapshot};
 let mut f=ProjectFixture::new();
 f.bind();
 let now=crate::assistant_memory::now();
 let config=crate::pet_config::defaults();
 let mut budget=admission_tests::budget();
 budget.deadline_at=now+60_000;
 budget.max_model_calls=1;
 budget.max_tool_calls=1;
 budget.max_output_bytes=65_536;
 budget.payment=Payment::LocalOnly;
 let prepare=|catalog|Review::prepare_planning_catalog(&f.goal,f.evidence(now),budget.clone(),
  Limits{max_rounds:1,max_steps:1,max_output_bytes:2048,no_progress_limit:1},
  &config,catalog,"own-project-correspondence".into(),nonce().unwrap(),now);
 for catalog in [Catalog::SpotifyTransportV1,Catalog::PeerCorrespondenceV1,Catalog::PeerDelegationV1,Catalog::PeerDelegationIterationV1]{
  assert!(prepare(catalog).is_err(),"{catalog:?}");
 }
 assert!(prepare(Catalog::PeerCorrespondenceMemoryV1).unwrap().approve(&f.goal,f.evidence(now+1),now+1).is_err());
 let approved=prepare(Catalog::PeerCorrespondenceMemoryV1).unwrap()
  .approve_with_data(&f.goal,f.evidence(now+1),Some(GoalConsent::BoundedGoalContext),now+1).unwrap();
 let spec=approved.spec();
 let snapshot=Snapshot::from_spec(spec).unwrap().unwrap();
 let source=snapshot.project.as_ref().unwrap();
 assert_eq!(source.metadata_hash,crate::pet_runtime::model::fingerprint(&f.project).unwrap());
 assert_eq!(&source.conversation_id,spec.context.source_conversation_id.as_ref().unwrap());
 assert_eq!(snapshot.conversation_id,spec.context.conversation_id);
 assert!(!serde_json::to_string(&snapshot).unwrap().contains(&f.project.name));
 assert!(!serde_json::to_string(&snapshot).unwrap().contains("PRIVATE_GOAL_PROJECT_NOTE"));
 let mut unbound=snapshot.clone();
 unbound.project=None;
 assert!(!unbound.valid());
 let mut other_capability=spec.clone();
 other_capability.planning_policy.as_mut().unwrap().catalog=Catalog::SpotifyTransportV1;
 assert!(!other_capability.binding_format());
 let capture=snapshot.reopen_project(&f.files.catalog(),now+1).unwrap().unwrap();
 f.project.name="Edited project at the same revision".into();
 f.write();
 assert!(snapshot.reopen_project(&f.files.catalog(),now+2).is_err());
 assert!(capture.verify(now+2).is_err());
}

#[test]fn project_goal_active_fence_and_confirmed_data_revoke_on_detach_without_erasing_history_or_replaying(){
 let mut f=ProjectFixture::new();f.bind();let now=crate::assistant_memory::now();let captured=Captured::at(&f.files.catalog(),&f.goal,&f.server(),"fixture-free",now).unwrap().unwrap();let data=captured.confirm(&f.goal,&f.server(),"fixture-free",Some(GoalConsent::BoundedGoalContext),now).unwrap();let msg=data.message(&f.goal,&f.server(),"fixture-free",now).unwrap();assert_eq!(msg["role"],"user");assert!(msg.to_string().contains("PRIVATE_GOAL_PROJECT_NOTE"));
 let(_,private)=open_at(&f.files.contexts(),&f.goal,&f.main,&f.files.catalog()).unwrap();assert!(private.goal_context_current(&f.goal.id));let original=fs::read(f.files.catalog()).unwrap();f.project.conversation_ids.clear();f.project.revision+=1;f.project.updated_at+=1;f.write();assert!(!private.goal_context_current(&f.goal.id));assert!(data.message(&f.goal,&f.server(),"fixture-free",now+1).is_err());fs::write(f.files.catalog(),original).unwrap();assert!(data.message(&f.goal,&f.server(),"fixture-free",now+2).is_err(),"revoked capture cannot revive");assert!(f.goal.admissions.is_empty());
}

#[test]fn project_progress_summary_links_only_the_explicit_snapshot_without_notes_or_authority(){
 let mut f=ProjectFixture::new();let legacy=serde_json::to_value(f.goal.summary()).unwrap();assert!(legacy.get("project").unwrap().is_null());
 f.bind();let summary=serde_json::to_value(f.goal.summary()).unwrap();assert_eq!(summary["project"],json!({"id":f.project.id,"revision":1}));let raw=summary.to_string();assert!(!raw.contains("PRIVATE_GOAL_PROJECT_NOTE")&&!raw.contains("conversationId")&&!raw.contains("Selected own project"));
 f.project.name="Current catalog can change; the original context remains linked".into();f.project.revision=2;f.project.conversation_ids.clear();f.write();assert_eq!(serde_json::to_value(f.goal.summary()).unwrap()["project"],summary["project"]);
 let before=fs::read(f.files.catalog()).unwrap();let unrelated=Fixture::goal(f.goal.spec.context.clone());assert!(unrelated.summary().project.is_none(),"sharing a conversation does not associate an unrelated goal with the project");
 assert!(f.goal.admissions.is_empty());assert_eq!(fs::read(f.files.catalog()).unwrap(),before);
}

#[test]fn project_rule_two_occurrences_keep_own_data_consent_partition_restart_and_changed_note_refusal(){
 use crate::pet_runtime::{rules::{self,RulePolicy,RuleGrant,RuleKind,RulePhase,CalendarReadOutcome,RuleScope,ProjectConsent},authority::Authority,store::Store};
 let mut f=ProjectFixture::new();f.bind();let now=crate::assistant_memory::now();
 let first=chrono::DateTime::from_timestamp_millis(now as i64).unwrap().format("%Y-%m-%d").to_string();
 let definition=serde_json::from_value(json!({"name":"Own pinned project routine","intent":"Summarize only the selected project notes","criteria":["Review actual response; never presume criterion verified"],"schedule":{"kind":"daily","firstDate":first,"days":3,"localTime":"00:00","timeZone":"UTC","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"},"missed":"coalesce","graceSeconds":300,"dataScope":{"project":{"sourceConversationId":f.project.conversation_ids[0],"project":f.choice(),"notes":[Fixture::selected("project-note")]}},"limits":{"maxOccurrences":2,"maxModelCalls":2,"maxToolCalls":0,"maxOutputBytes":8192,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":86400}})).unwrap();
 let rule=rules::Rule{id:format!("rule-{}","a".repeat(64)),revision:1,owner:f.goal.spec.owner.clone(),state:rules::State::Draft,created_at:now,updated_at:now,definition};
 let schedule={let _lock=crate::pet_calendar::TEST_SERIAL.lock().unwrap();crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||true).unwrap()};
 let valid_until=schedule.slots.last().unwrap().at.unwrap()+86400000;
 let policy:RulePolicy=serde_json::from_value(json!({"schemaVersion":6,"scope":"boundedRuleRecurrenceWithPinnedProjectData","rule":rule,"payment":"localOnly","destination":{"kind":"server","provider":"ollama","endpoint":"http://127.0.0.1:12345/api/chat","model":"fixture-free"},"model":"fixture-free","engineBinding":null,"policyRevision":"a".repeat(64),"descriptorSemantics":"b".repeat(64),"schedule":schedule,"preparedAt":now,"validUntil":valid_until,"projectData":{"schemaVersion":1,"consent":"boundedRulePinnedProjectData","sourceConversationId":f.project.conversation_ids[0],"selection":f.goal.execution_context.as_ref().unwrap().memory,"maxInputBytes":262144,"maxCaptureSeconds":10,"maxReadsPerOccurrence":1}})).unwrap();
 assert!(policy.valid());assert!(!policy.consent_matches(Some(policy.scope),None,None));assert!(policy.consent_matches(Some(policy.scope),None,Some(ProjectConsent::BoundedRulePinnedProjectData)));assert!(!policy.consent_matches(Some(RuleScope::BoundedRuleRecurrenceWithFutureCalendarData),None,Some(ProjectConsent::BoundedRulePinnedProjectData)));
 let root=f.files.root.join("authority");let authority=Authority::ensure(&root).unwrap();let store=Store::ensure_native(&f.files.root.join("goals")).unwrap();
 let parent=RuleGrant{schema_version:11,kind:RuleKind::RuleRecurrence,decision_id:nonce().unwrap(),revision:nonce().unwrap(),policy:policy.clone(),phase:RulePhase::Active,approved_at:now,updated_at:now,revoked_at:None,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None};assert!(parent.valid());authority.issue_rule(&parent,now,||true).unwrap();
 let mut goals=vec![];
 for index in 0..2{
  let at=policy.schedule.slots[index].at.unwrap().max(now)+10;
  let owned=authority.begin_owned_rule_calendar(&parent.decision_id,&policy.hash().unwrap(),index,at,||true).unwrap();
  let mut goal=store.create(policy.goal_spec(index).unwrap(),at).unwrap().record;
  goal=store.mutate(&goal.id,"ai_ollama",&goal.revision,format!("task-{index}"),Action::ProposeTask{description:"Own exact project".into()},at).unwrap().record;
  goal=store.mutate(&goal.id,"ai_ollama",&goal.revision,format!("plan-{index}"),Action::ProposePlan{task_id:goal.tasks[0].id.clone(),steps:vec![crate::pet_runtime::model::StepInput{description:"One finite response".into(),postcondition:"Response retained; criteria unverified".into(),operation:None}]},at).unwrap().record;
  let(binding,writer)=prepare_project_fixture(&f.files.contexts(),&goal,&f.main,&format!("capture-{index}"),&[Fixture::selected("project-note")],&f.choice(),&f.files.catalog()).unwrap();
  goal=store.mutate(&goal.id,"ai_ollama",&goal.revision,format!("bind-{index}"),Action::BindContext{binding:binding.clone()},at).unwrap().record;drop(writer);
  authority.finish_rule_calendar_read(&owned,CalendarReadOutcome::Captured{goal_id:goal.id.clone(),context_hash:crate::pet_runtime::model::fingerprint(&binding).unwrap()},at+1,||true).unwrap();drop(owned);
  let actual=authority.rule_grant(&parent.decision_id).unwrap();let allocation=&actual.reservations[index];let reference=actual.reference(allocation).unwrap();assert!(reference.project_context(&binding));assert_eq!(reference.schema_version,3);
  let capture=Captured::at(&f.files.catalog(),&goal,&f.server(),"fixture-free",at+2).unwrap().unwrap();assert!(capture.confirm(&goal,&f.server(),"fixture-free",Some(GoalConsent::BoundedRulePinnedProjectData),at+2).is_err(),"ordinary goal review cannot impersonate a rule consent");
  let capture=Captured::at(&f.files.catalog(),&goal,&f.server(),"fixture-free",at+2).unwrap();let evidence=Evidence::from_memory_capture(&goal,&admission_tests::descriptor(&goal,at+2),policy.policy_revision.clone(),at+2,false,false,capture).unwrap();assert!(evidence.rule_binding(&policy));
  let budget=crate::pet_runtime::admission::Budget{deadline_at:allocation.deadline_at,max_model_calls:1,max_tool_calls:0,max_output_bytes:4096,max_tokens:None,max_cost_micros:None,payment:Payment::LocalOnly};
  let approved=Review::prepare_rule_occurrence(&goal,evidence,budget,reference,None,at+2).unwrap();assert_eq!(approved.spec().schema_version,31);assert_eq!(approved.spec().memory_data.as_ref().unwrap().schema_version,12);assert!(actual.child_matches(allocation,approved.spec()));
  let mut forged=approved.spec().clone();forged.memory_data.as_mut().unwrap().consent=GoalConsent::BoundedGoalContext;assert!(!forged.binding_format());forged=approved.spec().clone();forged.context.source_conversation_id=None;assert!(!forged.binding_format());
  forged=approved.spec().clone();forged.context.memory.as_mut().unwrap().project.as_mut().unwrap().name="Other metadata at same revision".into();forged.context_hash=crate::pet_runtime::model::fingerprint(&forged.context).unwrap();assert!(!actual.child_matches(allocation,&forged));
  authority.bind_rule_admission(approved.spec(),at+2,||true).unwrap();let certified=authority.certify(&approved,&goal,at+2).unwrap();let queued=store.enqueue_certified(&certified,at+2).unwrap().record;assert_eq!(queued.schema_version,68);queued.validate().unwrap();authority.verify_queued(&queued,at+2).unwrap();goals.push(queued);
 }
 let observation_at=goals[1].admissions[0].spec.approved_at+1;
 let observation_scope=crate::pet_runtime::project_goal_observation::Selection::capture(&f.files.catalog(),"ai_ollama",&f.project.conversation_ids[0],&f.choice(),goals.iter().map(|g|crate::pet_runtime::project_goal_observation::Reference::of(g).unwrap()).collect(),true,observation_at).unwrap();
 let observation=crate::pet_runtime::project_goal_observation::capture(&store,&root,&f.files.catalog(),&observation_scope,None,observation_at).unwrap();assert_eq!(observation.observations.len(),2);assert!(observation.observations.iter().all(|o|o.response==crate::pet_runtime::project_goal_observation::Reply::NotStarted&&o.admission.as_ref().is_some_and(|a|a.grant_phase=="issued")&&!o.criteria_verified&&!o.goal_complete));
 let reopened=Authority::existing(&root).unwrap();let sealed=fs::read(root.join(format!("{}.json",parent.decision_id))).unwrap();assert_eq!(reopened.rule_status(&rule,true).unwrap().remaining_occurrences,Some(0));assert!(reopened.begin_owned_rule_calendar(&parent.decision_id,&policy.hash().unwrap(),2,policy.schedule.slots[2].at.unwrap(),||true).is_err());assert_eq!(fs::read(root.join(format!("{}.json",parent.decision_id))).unwrap(),sealed);
 for goal in &goals{let capture=Captured::at(&f.files.catalog(),goal,&f.server(),"fixture-free",now+30).unwrap().unwrap();let ref0=goal.admissions.last().unwrap().spec.rule_occurrence.as_ref().unwrap();let data=capture.into_rule_project(goal,ref0,now+30).unwrap();assert_eq!(data.grant().consent,GoalConsent::BoundedRulePinnedProjectData);}
 // Parent revocation must block fresh effects while allowing the original
 // opaque owner to finish Stop/drain. It cannot retain the model resource.
 let goal=goals[1].clone();let at=goal.admissions[0].spec.approved_at+10;
 let observe=|g:&Record,t|{let capture=Captured::at(&f.files.catalog(),g,&f.server(),"fixture-free",t).unwrap();Evidence::from_memory_capture(g,&admission_tests::descriptor(g,t),policy.policy_revision.clone(),t,false,false,capture).unwrap()};
 let controller=crate::pet_runtime::lease_controller::Controller::new().unwrap();let evidence=observe(&goal,at);
 let pending=controller.prepare_memory_fixture(&goal,&root,&f.files.contexts(),&f.files.catalog(),&f.main,&evidence,"own-project-claim".into(),at,Arc::new(|_,_,_|crate::pet_runtime::lease_controller::Validity::Current)).unwrap();
 let(leased,owner0)=pending.claim(&store,&reopened,&evidence,at+1).unwrap();let owner0=owner0.unwrap();reopened.revoke_rule("ai_ollama",&rule.id,&parent.decision_id,at+2).unwrap();assert!(reopened.verify_queued(&goal,at+3).is_err());
 let stopped=crate::pet_runtime::authority_store::mutate_with_authority(&store,&root,&goal.id,"ai_ollama",&leased.record.revision,"close-only-original-project-owner".into(),Action::Pause,at+3).unwrap();assert_eq!(stopped.record.state,crate::pet_runtime::model::State::Stopping);
 let proof=owner0.drain(crate::pet_runtime::leases::DrainOutcome::Confirmed).unwrap();let closed=store.drain_owned(&reopened,&proof,at+4).unwrap();assert_eq!(closed.record.state,crate::pet_runtime::model::State::Paused);goals[1]=closed.record;
 pending.hold().release_drained(&proof).unwrap();let next=f.main.runtime_resources().acquire("ollama",Arc::default(),None).unwrap();assert!(next.current());drop(next);let sealed=fs::read(root.join(format!("{}.json",parent.decision_id))).unwrap();
 f.row["content"]=json!("Changed without adopting silently");f.row["revision"]=json!(2);f.write();for goal in &goals{assert!(Captured::at(&f.files.catalog(),goal,&f.server(),"fixture-free",now+31).is_err());assert!(store.read(&goal.id,"ai_ollama").unwrap().validate().is_ok());}assert_eq!(fs::read(root.join(format!("{}.json",parent.decision_id))).unwrap(),sealed);
 let restore=crate::pet_runtime::restoration::Review::capture(&root,&f.files.root.join("goals"),&"d".repeat(64),||Ok(())).unwrap();let preview=restore.preview().unwrap();assert_eq!(preview["historicalGoals"],2);let parent_view=preview["grants"].as_array().unwrap().iter().find(|r|r["ruleId"]==rule.id).unwrap();assert_eq!(parent_view["signatureValid"],true);assert_eq!(parent_view["ruleReservations"],2);let fingerprint=restore.fingerprint().unwrap();restore.apply(&fingerprint,||Ok(())).unwrap();let restored=Authority::existing(&root).unwrap();assert!(crate::pet_runtime::project_goal_observation::capture(&store,&root,&f.files.catalog(),&observation_scope,None,observation_at+100).is_err(),"historical metadata must not become a current project/model observation");let historical=restored.rule_status(&rule,true).unwrap();assert!(historical.snapshot_only);assert_eq!(historical.remaining_occurrences,None);assert!(!historical.dispatch_available);assert_eq!(fs::read(root.join(format!("{}.json",parent.decision_id))).unwrap(),sealed);for goal in &goals{assert!(restored.verify_queued(goal,now+32).is_err());assert_eq!(serde_json::to_value(store.read(&goal.id,"ai_ollama").unwrap()).unwrap(),serde_json::to_value(goal).unwrap());}
}

#[path="project_goal_observation_tests.rs"]mod observation_tests;
