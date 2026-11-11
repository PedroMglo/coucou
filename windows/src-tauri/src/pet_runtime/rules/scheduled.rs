//! Bounded preparation inside the EXISTING lease service. Authority and the
//! normal requested dispatcher own all durable budgets, execution and recovery.
use super::*;
use super::super::{admission::Evidence,authority::{Authority,Failure},model::{Error,Record,State},store::Store};
use std::{path::PathBuf,sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use tauri::{AppHandle,Manager};
use crate::{chat::Chat,pet_resources::Hold};

type Observation=AutomaticObservation;
#[derive(Default)]struct Scan{after:Option<String>,last:Option<Instant>,notes:std::collections::BTreeMap<String,Observation>}
fn scan()->&'static Mutex<Scan>{static S:OnceLock<Mutex<Scan>>=OnceLock::new();S.get_or_init(||Mutex::new(Scan::default()))}
pub(crate)fn observation(id:&str)->Option<Observation>{scan().lock().ok()?.notes.get(id).cloned()}
fn note(id:&str,phase:&str,error:Option<Error>){if let Ok(mut s)=scan().lock(){if s.notes.len()<64||s.notes.contains_key(id){s.notes.insert(id.into(),Observation{observed_at:crate::assistant_memory::now(),phase:phase.into(),reason:error.map(|e|format!("{e:?}"))});}}}
static ACTIVE:AtomicBool=AtomicBool::new(false);
struct Running;impl Drop for Running{fn drop(&mut self){ACTIVE.store(false,Ordering::Release);}}

/// No avalanche: select one latest due civil slot. Decision slots and missed
/// decision policies block later work instead of silently skipping them.
pub(crate)fn due(g:&RuleGrant,prior:&[RuleReservation],future_after:u64,store:&Store,now:u64)->model::Result<Option<usize>>{
 if g.phase!=RulePhase::Active||g.policy.dispatch.is_none()||now<g.updated_at||now>=g.policy.valid_until{return Ok(None);}
 let Some(index)=g.policy.schedule.slots.iter().enumerate().filter(|(_,s)|s.at.is_some_and(|at|at<=now)).map(|(i,_)|i).last()else{return Ok(None);};
 if g.policy.schedule.slots[..=index].iter().any(|s|matches!(s.resolution,crate::pet_calendar::schedule::Resolution::DecisionGap|crate::pet_calendar::schedule::Resolution::DecisionOverlap)){return Err(Error::Conflict);}
 let at=g.policy.schedule.slots[index].at.ok_or(Error::Conflict)?;let id=g.policy.occurrence_id(index).ok_or(Error::Conflict)?;
 if at<=future_after||prior.iter().any(|r|r.occurrence_id==id){return Ok(None);}
 if let Some(r)=g.reservations.iter().find(|r|r.occurrence_id==id){
  if now>=r.deadline_at{return Ok(None);}if r.readiness.iter().any(|p|p.finished_at.is_none()){return Err(Error::Conflict);}
  if let Some(saved)=&r.admission{let goal=store.read(&saved.goal_id,&g.policy.rule.owner.pet_id)?;return Ok(goal.admissions.is_empty().then_some(index));}
  return Ok(Some(index));
 }
 if g.reservations.len()+prior.len()>=usize::from(g.policy.rule.definition.limits.max_occurrences)||now>=at.saturating_add(u64::from(g.policy.rule.definition.limits.occurrence_timeout_seconds)*1000){return Ok(None);}
 let grace=u64::from(g.policy.rule.definition.grace_seconds)*1000;
 match g.policy.rule.definition.missed{
  Missed::Skip if now.saturating_sub(at)>grace=>return Ok(None),
  Missed::Decision if g.policy.schedule.slots.iter().enumerate().any(|(i,s)|s.at.is_some_and(|at|at<=now&&now-at>grace)&&g.policy.occurrence_id(i).is_some_and(|id|!g.reservations.iter().chain(prior).any(|r|r.occurrence_id==id)))=>return Err(Error::Conflict),
  _=>{},
 }
 Ok(Some(index))
}
fn policy_current(app:&AppHandle,p:&RulePolicy)->bool{crate::pet_data::bootstrap::write_allowed().is_ok()&&authority_api::current(app,p)}
fn evidence(app:&AppHandle,g:&Record,hold:Option<&Hold>)->model::Result<Evidence>{super::super::admission_review::evidence(app,g,crate::assistant_memory::now(),hold)?.map_err(|_|Error::Conflict)}

/// Same bounded metadata executor for pre-admission and an already queued rule
/// child. Its native hold remains owned until cancelled IO has actually ended.
async fn inspect(app:&AppHandle,goal:&Record,parent:&RuleGrant,r:&RuleReservation,hold:Arc<Hold>,root:&std::path::Path,authority:&std::path::Path)->model::Result<()>{
 let a=Authority::existing(authority).map_err(Failure::control_error)?;let p=&parent.policy;
 let projection=crate::pet_capabilities::native_projection(app,Some(&hold)).map_err(|_|Error::StorageUnavailable)?;
 let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;
 if !projection.preferences_ready||!policy_current(app,p){return Err(Error::Conflict);}
 let captured=hold.captured();let server=if let Some(c)=&captured{crate::local_chat::server_for_capture(c).map_err(|_|Error::Conflict)?}else{super::super::memory_review::server_for_owner(&p.rule.owner,config,&projection.settings,&p.model).map_err(|_|Error::Conflict)?};
 if super::super::memory_review::destination_for(&p.rule.owner,&server,&p.model).map_err(|_|Error::Conflict)?!=p.destination{return Err(Error::Conflict);}
 let owned=a.begin_owned_rule_probe(&parent.decision_id,&r.occurrence_id,crate::assistant_memory::now(),||hold.current()&&policy_current(app,p)).map_err(Failure::control_error)?;let probe=owned.probe();
 let store=Store::existing_verified(root,authority)?.ok_or(Error::NotFound)?;
 let until=Instant::now()+Duration::from_millis(probe.expires_at.saturating_sub(crate::assistant_memory::now()).min(6000));
 let result=super::super::readiness_api::inspect_captured(&server,&p.model,captured.as_deref(),||{
  if !owned.current(&a)||!hold.current()||Instant::now()>=until||!policy_current(app,p){return Err(Error::Conflict);}
  a.rule_probe_current(&parent.decision_id,&r.occurrence_id,probe,crate::assistant_memory::now()).map_err(Failure::control_error)?;
  let same=store.read_control(&goal.id,&goal.spec.owner.pet_id)?;if same.revision!=goal.revision||same.spec!=goal.spec{return Err(Error::Conflict);}Ok(())
 }).await;
 let observed=result.is_ok()&&evidence(app,goal,Some(&hold)).is_ok_and(|e|e.rule_binding(p));
 a.finish_rule_probe(&parent.decision_id,&r.occurrence_id,probe,observed,crate::assistant_memory::now()).map_err(Failure::control_error)?;
 if !observed{return Err(result.err().unwrap_or(Error::Conflict));}Ok(())
}

pub(in crate::pet_runtime)async fn inspect_admitted(app:&AppHandle,goal:&Record,hold:Arc<Hold>)->model::Result<()>{
 let root=crate::settings::config_dir().join("pet-runtime-v1");let authority=super::super::admission_review::authority_root();let a=Authority::existing(&authority).map_err(Failure::control_error)?;
 a.verify_queued(goal,crate::assistant_memory::now()).map_err(Failure::control_error)?;
 let reference=goal.admissions.last().and_then(|r|r.spec.rule_occurrence.as_ref()).ok_or(Error::Conflict)?;let parent=a.rule_grant(&reference.rule_decision_id).map_err(Failure::control_error)?;let r=parent.reservations.iter().find(|r|r.occurrence_id==reference.occurrence_id).ok_or(Error::Conflict)?;
 inspect(app,goal,&parent,r,hold,&root,&authority).await
}

async fn prepare(app:&AppHandle,root:&std::path::Path,authority:&std::path::Path,parent:RuleGrant,index:usize)->model::Result<()>{
 let p=&parent.policy;if !policy_current(app,p){return Err(Error::Conflict);}let store=Store::ensure_native(root)?;let a=Authority::existing(authority).map_err(Failure::control_error)?;let main=app.state::<Chat>();
 let contexts=crate::settings::config_dir().join("pet-runtime-contexts-v1");let goal=occurrences::prepare_draft(&store,&contexts,&main,&parent,index,&crate::assistant_memory::now,||policy_current(app,p))?;
 if !goal.admissions.is_empty(){return Ok(());}if goal.state!=State::Draft{return Err(Error::Conflict);}
 let allocation=a.reserve_rule_occurrence(&parent.decision_id,&p.hash()?,index,crate::assistant_memory::now(),||policy_current(app,p)).map_err(Failure::control_error)?;
 if allocation.readiness.iter().any(|p|p.finished_at.is_none()){return Err(Error::Conflict);}
 if !evidence(app,&goal,None).is_ok_and(|e|e.rule_binding(p)){
  let until=Instant::now()+Duration::from_millis(allocation.deadline_at.saturating_sub(crate::assistant_memory::now()).min(6000));
  if allocation.readiness.len()>=3{return Err(Error::LimitReached);}if allocation.readiness.last().filter(|p|p.observed==Some(false)).and_then(|p|p.finished_at).is_some_and(|at|crate::assistant_memory::now()<at.saturating_add(30000)){return Err(Error::Busy);}
  let hold=main.runtime_resources().acquire_goal(&p.rule.owner.provider,Arc::new(AtomicBool::new(false)),Some(until)).map_err(|_|Error::Busy)?;
  inspect(app,&goal,&parent,&allocation,hold,root,authority).await?;
 }
 occurrences::admit_at(&store,&a,&contexts,&main,&parent.decision_id,index,crate::assistant_memory::now,|p|policy_current(app,p),|g,_|evidence(app,g,None))?;Ok(())
}

/// Only a mechanical scan/cursor is retained here. At most one preparation is
/// in metadata IO and two admissions are attempted per scan; the normal common
/// dispatcher can execute independent goals concurrently. Stop never takes this
/// gate, and no model/authority work runs under its mutex.
pub(in crate::pet_runtime)fn tick(app:AppHandle,root:PathBuf,authority:PathBuf){
 {let Ok(mut s)=scan().try_lock()else{return;};if s.last.is_some_and(|at|at.elapsed()<Duration::from_secs(1)){return;}if ACTIVE.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return;}s.last=Some(Instant::now());}
 tauri::async_runtime::spawn(async move{
  let _running=Running;
  let Ok(mut rules)=saved_rules()else{return;};rules.sort_by(|a,b|a.id.cmp(&b.id));
  let keys={let Ok(mut s)=scan().lock()else{return;};let pivot=s.after.as_ref().map_or(0,|a|rules.partition_point(|r|&r.id<=a));let keys=rules[pivot..].iter().chain(rules[..pivot].iter()).take(8).cloned().collect::<Vec<_>>();s.after=keys.last().map(|r|r.id.clone());keys};
  if keys.is_empty(){return;}
  let Ok(a)=Authority::existing(&authority)else{return;};let Ok(store)=Store::ensure_native(&root)else{return;};let mut prepared=0;
  for rule in keys{
   let Ok(Some(decision))=a.rule_decision_for(&rule.owner.pet_id,&rule.id)else{continue;};let Ok(parent)=a.rule_grant(&decision)else{note(&rule.id,"reconciliationRequired",Some(Error::Conflict));continue;};if parent.policy.rule!=rule||parent.policy.dispatch.is_none(){continue;}
   let Ok(bounds)=a.rule_execution_bounds(&decision)else{note(&rule.id,"lineageReconciliationRequired",Some(Error::Conflict));continue;};
   let index=match due(&parent,&bounds.prior,bounds.future_after,&store,crate::assistant_memory::now()){Ok(Some(i))=>i,Ok(None)=>continue,Err(e)=>{note(&rule.id,"decisionOrRecoveryRequired",Some(e));continue;}};
   if prepared>=2{break;}prepared+=1;note(&rule.id,"preparing",None);
   match prepare(&app,&root,&authority,parent,index).await{Ok(())=>note(&rule.id,"admitted",None),Err(e)=>note(&rule.id,"blocked",Some(e))}
  }
 });
}
