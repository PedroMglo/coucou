//! One explicitly controlled future-data capture in the existing occurrence,
//! goal and context stores. This does not admit, enqueue or call a model.
use super::*;
#[cfg(target_os="linux")]
use super::super::{authority::{Authority,Failure},context,model::{Action,CommitStatus,Error,Record},store::Store};
#[cfg(target_os="linux")]use std::{path::Path,sync::Arc};

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Prepared{
 pub goal_id:String,pub rule_id:String,pub occurrence_id:String,pub context_hash:String,
 pub sources:usize,pub unavailable_sources:usize,pub partial_sources:usize,
 pub existing_capture:bool,pub queued:bool,
}
#[cfg(target_os="linux")]
fn receipt(goal:&Record,parent:&RuleGrant,index:usize,existing_capture:bool)->model::Result<Prepared>{
 let binding=goal.execution_context.as_ref().ok_or(Error::Conflict)?;
 if let Some(data)=&parent.policy.project_progress_data{if !data.matches_context(binding){return Err(Error::Conflict);}let snapshot=binding.project_progress.as_ref().ok_or(Error::Conflict)?;return Ok(Prepared{goal_id:goal.id.clone(),rule_id:parent.policy.rule.id.clone(),occurrence_id:parent.policy.occurrence_id(index).ok_or(Error::Conflict)?,context_hash:model::fingerprint(binding)?,sources:snapshot.observations.len(),unavailable_sources:snapshot.observations.iter().filter(|o|matches!(o.response,super::super::project_goal_observation::Reply::SourceDataUnavailable{..})).count(),partial_sources:snapshot.observations.iter().filter(|o|!o.evidence_complete||!o.accounting_complete||!o.outcomes_complete).count(),existing_capture,queued:false});}
 if let Some(data)=&parent.policy.project_data{if !data.matches_context(binding){return Err(Error::Conflict);}return Ok(Prepared{goal_id:goal.id.clone(),rule_id:parent.policy.rule.id.clone(),occurrence_id:parent.policy.occurrence_id(index).ok_or(Error::Conflict)?,context_hash:model::fingerprint(binding)?,sources:data.selection.references.len(),unavailable_sources:0,partial_sources:0,existing_capture,queued:false});}
 let data=binding.calendars.as_ref().ok_or(Error::Conflict)?;
 let snapshots=data.data["snapshots"].as_array().ok_or(Error::Conflict)?;
 Ok(Prepared{goal_id:goal.id.clone(),rule_id:parent.policy.rule.id.clone(),occurrence_id:parent.policy.occurrence_id(index).ok_or(Error::Conflict)?,context_hash:model::fingerprint(binding)?,sources:snapshots.len(),unavailable_sources:snapshots.iter().filter(|s|s["status"]=="unavailable").count(),partial_sources:snapshots.iter().filter(|s|s["status"]!="unavailable"&&(s["coverage"]["eventsComplete"]!=true||s["coverage"]["fieldsComplete"]!=true)).count(),existing_capture,queued:false})
}

/// This retries only a pre-publication catalogue Busy, with the SAME captured
/// binding and request. No source reader, context preparation or reservation is
/// called here. Ambiguous publication, changed validity/revision and the
/// original read deadline stop immediately; a retry never obtains a new budget.
#[cfg(target_os="linux")]
fn bind_context(store:&Store,goal:&Record,binding:model::ContextBinding,request:String,deadline:u64,clock:&dyn Fn()->u64,current:&dyn Fn()->bool)->model::Result<Record>{
 use std::time::{Duration,Instant};
 let until=Instant::now()+Duration::from_millis(deadline.saturating_sub(clock()).min(500));
 loop{
  let now=clock();if now>=deadline||!current(){return Err(Error::Conflict);}
  match store.mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,request.clone(),Action::BindContext{binding:binding.clone()},now){
   Ok(changed)=>return if changed.status==CommitStatus::Uncertain{Err(Error::StorageUnavailable)}else{Ok(changed.record)},
   Err(Error::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(5)),
   Err(e)=>return Err(e),
  }
 }
}

/// Publish only the already captured result with its original owner/reference.
/// Wait only for pre-publication contention, inside the original read deadline.
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)fn seal_capture(authority:&Authority,owned:&super::super::authority_store::rules::OwnedCalendarRead,outcome:CalendarReadOutcome,clock:&dyn Fn()->u64,current:&dyn Fn()->bool)->model::Result<()>{
 use std::time::{Duration,Instant};
 if !matches!(outcome,CalendarReadOutcome::Captured{..}){return Err(Error::InvalidInput);}
 let original=owned.read();let until=Instant::now()+Duration::from_millis(original.expires_at.saturating_sub(clock()).min(500));
 let valid=||{let now=clock();now>=original.prepared_at&&now<original.expires_at&&current()};
 loop{
  let now=clock();if !valid(){return Err(Error::Conflict);}
  // Busy is the original writer acquisition before publication. Retain the
  // exact capture/owner/partition while waiting; never retry Uncertain or call
  // any data reader, context preparer, reservation, admission or model here.
  match authority.finish_rule_calendar_read(owned,outcome.clone(),now,&valid){
   Ok(())=>return Ok(()),
   Err(Failure::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(5)),
   Err(error)=>return Err(error.control_error()),
  }
 }
}

/// The caller supplies native configuration/window validity, never consent or
/// source paths from the renderer. The retained owner spans all synchronous IO.
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)fn prepare_at(store:&Store,authority:Arc<Authority>,contexts:&Path,main:&crate::chat::Chat,decision:&str,index:usize,clock:Arc<dyn Fn()->u64+Send+Sync>,current:Arc<dyn Fn(&RulePolicy)->bool+Send+Sync>)->model::Result<Prepared>{
 let parent=authority.rule_grant(decision).map_err(Failure::control_error)?;
 let policy=parent.policy.clone();if !policy.selected_data(){return Err(Error::Conflict);}
 let now=clock();
 if parent.phase!=RulePhase::Active||now<parent.updated_at||now>=policy.valid_until||!current(&policy){return Err(Error::Conflict);}
 let slot=policy.schedule.slots.get(index).ok_or(Error::InvalidInput)?;
 let at=slot.at.ok_or(Error::InvalidInput)?;
 if now<at||now>=at.saturating_add(u64::from(policy.rule.definition.limits.occurrence_timeout_seconds)*1000){return Err(Error::Conflict);}
 let occurrence=policy.occurrence_id(index).ok_or(Error::InvalidInput)?;
 let bounds=authority.rule_execution_bounds(decision).map_err(Failure::control_error)?;
 if at<=bounds.future_after||bounds.prior.iter().any(|r|r.occurrence_id==occurrence){return Err(Error::Conflict);}
 // A successful, authenticated capture may only be observed again. Never call
 // prepare_with_calendar_data here: even a missing context is not a new read.
 if let Some(read)=parent.reservations.iter().find(|r|r.occurrence_id==occurrence).and_then(|r|r.calendar_read.as_ref()){
  let Some(CalendarReadOutcome::Captured{goal_id,context_hash})=&read.outcome else{return Err(Error::Conflict);};
  let goal=store.read(goal_id,&policy.rule.owner.pet_id)?;
  let(binding,_)=context::read_snapshot(contexts,&goal)?;
  if &model::fingerprint(&binding)?!=context_hash||!current(&policy){return Err(Error::Conflict);}
  return receipt(&goal,&parent,index,true);
 }
 // Goal/plan metadata can contend with ordinary observers. Complete it before
 // committing the only extraction; Busy here leaves no calendar read to spend.
 let mut goal=occurrences::prepare_plan(store,&parent,index,&||clock(),||current(&policy))?;
 if !goal.admissions.is_empty()||goal.execution_context.is_some(){return Err(Error::Conflict);}
 let owned=Arc::new(authority.begin_owned_rule_calendar(decision,&policy.hash()?,index,now,||current(&policy)).map_err(Failure::control_error)?);
 let guard:Arc<dyn Fn()->bool+Send+Sync>={let authority=authority.clone();let owned=owned.clone();let clock=clock.clone();let current=current.clone();let policy=policy.clone();Arc::new(move||current(&policy)&&owned.current(&authority,clock()))};
 let mut stage="readOwner";
 let captured=(||{
  if !guard(){return Err(Error::Conflict);}
  let(binding,chat)=if let Some(data)=&policy.project_progress_data{
   stage="captureAndPersistProjectProgressContext";let selected=data.notes.as_ref().map(|n|n.references.iter().map(|r|crate::pet_memory::context_model::Selected{id:r.id.clone(),revision:r.revision}).collect::<Vec<_>>());
   context::prepare_with_project_progress_data(contexts,&goal,main,&format!("rc-{occurrence}"),&data.selection,selected.as_deref(),store,&authority.root_path(),&crate::pet_memory::context::Capture::catalog_path(),guard.clone())?
  }else if let Some(data)=&policy.project_data{
   stage="captureAndPersistProjectContext";
   let selected:Vec<_>=data.selection.references.iter().map(|r|crate::pet_memory::context_model::Selected{id:r.id.clone(),revision:r.revision}).collect();
   context::prepare_with_project_data(contexts,&goal,main,model::PreparationMode::Dedicated,&format!("rc-{occurrence}"),Some(&selected),&data.choice().ok_or(Error::Conflict)?,&authority.root_path())?
  }else{
  let data=policy.calendar_data.as_ref().ok_or(Error::Conflict)?;
  stage="civilDay";
  let date=slot.local_date_time.get(..10).ok_or(Error::InvalidInput)?;
  let reference=chrono::DateTime::from_timestamp_millis(i64::try_from(at).map_err(|_|Error::InvalidInput)?).ok_or(Error::InvalidInput)?.to_rfc3339();
  let query=crate::pet_calendar::day::explicit(date,&policy.rule.definition.schedule.time_zone,&reference,||guard()).map_err(|_|Error::Conflict)?;
  let selected=crate::pet_calendar::goal::Request{source_scope:Some(crate::pet_sources::PetScope::Pet),selected:data.sources.iter().map(|s|crate::pet_calendar::goal::Pick{id:s.id.clone(),revision:s.revision}).collect(),query};
  stage="captureAndPersistContext";
  context::prepare_with_calendar_data(contexts,&goal,main,model::PreparationMode::Dedicated,&format!("rc-{occurrence}"),None,&authority.root_path(),Some(&selected),guard.clone())?
  };
  stage="validatePersistedContext";
  if !guard(){return Err(Error::Conflict);}
  stage="bindPersistedContext";
  goal=bind_context(store,&goal,binding,format!("rb-{occurrence}"),owned.read().expires_at,clock.as_ref(),guard.as_ref())?;
  // Retain the existing exclusive context writer until the exact metadata
  // link is published. Neither a timer nor a read-only snapshot owns it.
  drop(chat);
  stage="validateBoundContext";
  if !guard(){return Err(Error::Conflict);}
  Ok(goal)
 })();
 let goal=match captured{
  Ok(goal)=>goal,
  Err(e)=>{
   crate::log::line(format!("rule-calendar stopped: stage={stage} reason={e:?}; original read retained, no extraction retry"));
   // All reader helpers have returned. Observe the stop, including after
   // revocation/expiry; a failed/uncertain publication still forbids a retry.
   let _=authority.finish_rule_calendar_read(&owned,CalendarReadOutcome::Stopped,clock(),||false);
   return Err(e);
  }
 };
 let prepared=receipt(&goal,&parent,index,false)?;
 seal_capture(&authority,&owned,CalendarReadOutcome::Captured{goal_id:goal.id,context_hash:prepared.context_hash.clone()},clock.as_ref(),&||current(&policy)).inspect_err(|e|crate::log::line(format!("rule-calendar stopped: stage=sealCapturedContext reason={e:?}; exact original read retained, no extraction retry")))?;
 Ok(prepared)
}

#[tauri::command]
pub async fn pet_runtime_rule_calendar_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PreviewInput)->Result<Prepared,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{
  let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleReviewClosed")?;
  tauri::async_runtime::spawn_blocking(move||{
   use tauri::Manager;
   crate::pet_data::bootstrap::write_allowed().map_err(|_|"ruleProfileInactive")?;
   let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
   let authority=Arc::new(Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleOwnAuthorityRequired")?);
   let decision=authority.rule_decision_for(&input.pet_id,&input.rule_id).map_err(|_|"ruleAuthorityUnavailable")?.ok_or("ruleOwnAuthorityRequired")?;
   let grant=authority.rule_grant(&decision).map_err(|_|"ruleAuthorityUnavailable")?;
   if grant.policy.rule!=rule||grant.phase!=RulePhase::Active{return Err("ruleOwnAuthorityRequired".into());}
   let now=crate::assistant_memory::now();
   let index=grant.policy.schedule.slots.iter().enumerate().filter(|(_,s)|s.at.is_some_and(|at|at<=now&&now<at.saturating_add(u64::from(rule.definition.limits.occurrence_timeout_seconds)*1000))).map(|(i,_)|i).last().ok_or("ruleNoDueOccurrence")?;
   let store=Store::ensure_native(&crate::settings::config_dir().join("pet-runtime-v1")).map_err(|_|"ruleGoalStorageUnavailable")?;
   let main=app.state::<crate::chat::Chat>();let owner=app.clone();
   prepare_at(&store,authority,&crate::settings::config_dir().join("pet-runtime-contexts-v1"),&main,&decision,index,Arc::new(crate::assistant_memory::now),Arc::new(move|p|frame.current()&&crate::pet_data::bootstrap::write_allowed().is_ok()&&authority_api::current(&owner,p))).map_err(|e|format!("ruleCalendar:{e:?}; consulta o objetivo e a leitura original. Nenhuma captura, saldo ou prazo é renovado."))
  }).await.map_err(|_|"ruleCalendarInterrupted; consulta a ocorrência antes de repetir".to_string())?
 }
}

#[cfg(all(test,target_os="linux"))]
mod publication_tests{
 use super::*;
 use std::{fs,os::{fd::AsRawFd,unix::fs::PermissionsExt},sync::atomic::{AtomicBool,Ordering},time::Duration};
 struct F{path:std::path::PathBuf,store:Store,goal:Record,binding:model::ContextBinding,_writer:crate::chat::Chat}
 impl F{
  fn new()->Self{
   let path=std::env::temp_dir().join(format!("coucou-calendar-link-{}",super::super::super::store::nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
   let store=Store::ensure_native(&path.join("goals")).unwrap();let now=crate::assistant_memory::now();
   let spec=model::Spec{request_id:"original-calendar-occurrence".into(),owner:Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None},context:model::Context::Unbound,intent:"Publish the exact captured context".into(),criteria:vec!["Review the actual response".into()],restored_from:None,received_from:None};
   let goal=store.create(spec,now-1).unwrap().record;let main=crate::chat::Chat::persistent(path.join("main.json"));let(binding,writer)=context::prepare(&path.join("contexts"),&goal,&main,model::PreparationMode::Dedicated,"original-capture").unwrap();Self{path,store,goal,binding,_writer:writer}
  }
  fn shared(&self)->fs::File{let f=fs::File::open(self.path.join("goals/.catalog.lock")).unwrap();assert_eq!(unsafe{libc::flock(f.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)},0);f}
  fn header(&self)->Vec<u8>{fs::read(self.path.join("contexts").join(&self.goal.id).join(".context.json")).unwrap()}
 }
 impl Drop for F{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
 #[test]fn busy_goal_reader_does_not_discard_the_already_captured_context_or_repeat_its_preparation(){
  let f=F::new();let header=f.header();let shared=f.shared();let now=crate::assistant_memory::now;
  assert!(matches!(f.store.mutate(&f.goal.id,"ai_ollama",&f.goal.revision,"original-link".into(),Action::BindContext{binding:f.binding.clone()},now()),Err(Error::Busy)));
  let released=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(60));drop(shared);});
  let bound=bind_context(&f.store,&f.goal,f.binding.clone(),"original-link".into(),now()+1000,&now,&||true).unwrap();released.join().unwrap();assert_eq!(bound.execution_context,Some(f.binding.clone()));assert_eq!(f.header(),header);assert_eq!(bound.events.iter().filter(|e|matches!(e.kind,model::EventKind::ContextPrepared)).count(),1);
  let replay=bind_context(&f.store,&f.goal,f.binding.clone(),"original-link".into(),now()+1000,&now,&||true).unwrap();assert_eq!(serde_json::to_value(replay).unwrap(),serde_json::to_value(bound).unwrap());assert_eq!(f.header(),header);
 }
 #[test]fn validity_loss_while_waiting_never_links_a_captured_context(){
  let f=F::new();let before=fs::read(f.path.join("goals").join(format!("{}.json",f.goal.id))).unwrap();let header=f.header();let shared=f.shared();let valid=Arc::new(AtomicBool::new(true));let flag=valid.clone();let stopped=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(20));flag.store(false,Ordering::Release);std::thread::sleep(Duration::from_millis(20));drop(shared);});
  assert!(matches!(bind_context(&f.store,&f.goal,f.binding.clone(),"original-link".into(),crate::assistant_memory::now()+1000,&crate::assistant_memory::now,&||valid.load(Ordering::Acquire)),Err(Error::Conflict)));stopped.join().unwrap();assert_eq!(f.header(),header);assert_eq!(fs::read(f.path.join("goals").join(format!("{}.json",f.goal.id))).unwrap(),before);
 }
 #[test]fn original_deadline_is_not_extended_while_waiting_for_catalogue(){
  let f=F::new();let shared=f.shared();let deadline=crate::assistant_memory::now()+25;
  assert!(bind_context(&f.store,&f.goal,f.binding.clone(),"original-link".into(),deadline,&crate::assistant_memory::now,&||true).is_err());drop(shared);assert!(f.store.read(&f.goal.id,"ai_ollama").unwrap().execution_context.is_none());
 }
 #[test]fn ambiguous_publication_stops_without_retrying_or_creating_another_link_event(){
  let mut f=F::new();f.store.fault=super::super::super::store::Fault::AfterPublish;
  assert!(matches!(bind_context(&f.store,&f.goal,f.binding.clone(),"original-link".into(),crate::assistant_memory::now()+1000,&crate::assistant_memory::now,&||true),Err(Error::StorageUnavailable)));
  let saved=f.store.read(&f.goal.id,"ai_ollama").unwrap();assert_eq!(saved.execution_context,Some(f.binding.clone()));assert_eq!(saved.events.iter().filter(|e|matches!(e.kind,model::EventKind::ContextPrepared)).count(),1);
 }
 fn metadata_parent(f:&F)->RuleGrant{
  let now=crate::assistant_memory::now();let date=chrono::DateTime::from_timestamp_millis(now as i64).unwrap().format("%Y-%m-%d").to_string();
  let definition:Definition=serde_json::from_value(serde_json::json!({"name":"Original metadata requests","intent":"One finite response","criteria":["Verify actual outcome"],"schedule":{"kind":"daily","firstDate":date,"days":1,"localTime":"00:00","timeZone":"UTC","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"},"missed":"coalesce","graceSeconds":300,"dataScope":"none","limits":{"maxOccurrences":1,"maxModelCalls":1,"maxToolCalls":0,"maxOutputBytes":4096,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":86400}})).unwrap();
  let rule=Rule{id:format!("rule-{}","a".repeat(64)),revision:1,owner:f.goal.spec.owner.clone(),state:super::super::State::Draft,created_at:now,updated_at:now,definition};
  let schedule={let _lock=crate::pet_calendar::TEST_SERIAL.lock().unwrap();crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||true).unwrap()};let valid_until=schedule.slots[0].at.unwrap()+86400000;
  let policy:RulePolicy=serde_json::from_value(serde_json::json!({"schemaVersion":1,"scope":"boundedRuleRecurrenceWithoutSelectedData","rule":rule,"payment":"localOnly","destination":{"kind":"server","provider":"ollama","endpoint":"http://127.0.0.1:12345/api/chat","model":"fixture-free"},"model":"fixture-free","engineBinding":null,"policyRevision":"a".repeat(64),"descriptorSemantics":"b".repeat(64),"schedule":schedule,"preparedAt":now,"validUntil":valid_until})).unwrap();
  let grant=RuleGrant{schema_version:1,kind:RuleKind::RuleRecurrence,decision_id:super::super::super::store::nonce().unwrap(),revision:super::super::super::store::nonce().unwrap(),policy,phase:RulePhase::Active,approved_at:now,updated_at:now,revoked_at:None,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None};assert!(grant.valid());grant
 }
 #[test]fn original_metadata_requests_wait_for_short_readers_without_duplicate_tasks_or_data_reads(){
  let f=F::new();let parent=metadata_parent(&f);let shared=f.shared();let thread=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(60));drop(shared);});
  let goal=occurrences::prepare_plan(&f.store,&parent,0,&crate::assistant_memory::now,||true).unwrap();thread.join().unwrap();assert_eq!(goal.tasks.len(),1);assert_eq!(goal.plans.len(),1);assert!(goal.execution_context.is_none()&&goal.admissions.is_empty());let before=serde_json::to_value(&goal).unwrap();let again=occurrences::prepare_plan(&f.store,&parent,0,&crate::assistant_memory::now,||true).unwrap();assert_eq!(serde_json::to_value(again).unwrap(),before);assert!(!f.path.join("authority").exists());
 }
 #[test]fn metadata_wait_stops_when_the_original_scope_changes_without_spending_a_capture(){
  let f=F::new();let parent=metadata_parent(&f);let shared=f.shared();let flag=Arc::new(AtomicBool::new(true));let change=flag.clone();let thread=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(20));change.store(false,Ordering::Release);std::thread::sleep(Duration::from_millis(40));drop(shared);});
  assert!(matches!(occurrences::prepare_plan(&f.store,&parent,0,&crate::assistant_memory::now,||flag.load(Ordering::Acquire)),Err(Error::Conflict)));thread.join().unwrap();let id=model::id_for(&parent.policy.goal_spec(0).unwrap());assert!(matches!(f.store.read(&id,"ai_ollama"),Err(Error::NotFound)));assert!(!f.path.join("authority").exists());
 }
 #[test]fn ambiguous_metadata_publication_is_not_retried_even_with_an_original_request(){
  let mut f=F::new();let parent=metadata_parent(&f);f.store.fault=super::super::super::store::Fault::AfterPublish;
  assert!(matches!(occurrences::prepare_plan(&f.store,&parent,0,&crate::assistant_memory::now,||true),Err(Error::StorageUnavailable)));let goal=f.store.read(&model::id_for(&parent.policy.goal_spec(0).unwrap()),"ai_ollama").unwrap();assert!(goal.tasks.is_empty()&&goal.plans.is_empty()&&goal.admissions.is_empty()&&goal.execution_context.is_none());assert_eq!(goal.events.len(),1);
 }

}
