//! Materialize one original allocation into the COMMON goal/admission/queue.
//! All retries are metadata reconciliation; an admitted goal is never replayed.
use super::*;
#[cfg(target_os="linux")]use super::super::{admission::{Budget,Evidence,Payment,Review},authority::{Authority,Failure},context,model::{Action,CommitStatus,Error,Record,StepInput},store::Store};
#[cfg(target_os="linux")]use crate::chat::Chat;
#[cfg(target_os="linux")]use std::path::Path;
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Admitted{pub goal_id:String,pub rule_id:String,pub occurrence_id:String,pub queued:bool,pub existing_admission:bool}
#[cfg(target_os="linux")]fn commit(receipt:model::Receipt)->model::Result<Record>{if receipt.status==CommitStatus::Uncertain{Err(Error::StorageUnavailable)}else{Ok(receipt.record)}}
#[cfg(target_os="linux")]fn checked(current:&impl Fn()->bool)->model::Result<()>{if current(){Ok(())}else{Err(Error::Conflict)}}
/// Native recurrence authority is rechecked inside every subsequent common
/// grant/queue/resource/Attempt writer. No generated text can enter this method.
#[cfg(target_os="linux")]pub(in crate::pet_runtime)fn admit_at(store:&Store,authority:&Authority,contexts:&Path,main:&Chat,decision:&str,index:usize,clock:impl Fn()->u64,current:impl Fn(&RulePolicy)->bool,observe:impl Fn(&Record,u64)->model::Result<Evidence>)->model::Result<Admitted>{
 let initial=authority.rule_grant(decision).map_err(Failure::control_error)?;let policy=&initial.policy;
 let valid=||current(policy);checked(&valid)?;
 if initial.phase!=RulePhase::Active||clock()<initial.updated_at||clock()>=policy.valid_until{return Err(Error::Conflict);}
 let at=policy.schedule.slots.get(index).and_then(|s|s.at).ok_or(Error::InvalidInput)?;if clock()<at||clock()>=at.saturating_add(u64::from(policy.rule.definition.limits.occurrence_timeout_seconds)*1000){return Err(Error::Conflict);}
 let wanted=policy.occurrence_id(index).ok_or(Error::InvalidInput)?;
 let bounds=authority.rule_execution_bounds(decision).map_err(Failure::control_error)?;
 if at<=bounds.future_after||bounds.prior.iter().any(|r|r.occurrence_id==wanted){return Err(Error::Conflict);}
 let goal=prepare_draft(store,contexts,main,&initial,index,&clock,valid)?;
 let occurrence=policy.occurrence_id(index).ok_or(Error::InvalidInput)?;
 if !goal.admissions.is_empty(){
  let row=goal.admissions.last().ok_or(Error::Conflict)?;
  if row.spec.rule_occurrence.as_ref().is_none_or(|r|r.rule_decision_id!=decision||r.occurrence_id!=occurrence){return Err(Error::Conflict);}
  return Ok(Admitted{goal_id:goal.id,rule_id:policy.rule.id.clone(),occurrence_id:occurrence,queued:goal.state==model::State::Queued,existing_admission:true});
 }

 let now=clock();let evidence=observe(&goal,now)?;checked(&valid)?;
 if !evidence.rule_binding(policy){return Err(Error::Conflict);}
 // A parent commitment is not observed inference use or a second balance.
 let allocation=if let Some(held)=initial.reservations.iter().find(|r|r.occurrence_id==occurrence){held.clone()}else{authority.reserve_rule_occurrence(decision,&policy.hash()?,index,now,valid).map_err(Failure::control_error)?};
 let parent=authority.rule_grant(decision).map_err(Failure::control_error)?;if parent.policy!=*policy{return Err(Error::Conflict);}
 let held=parent.reservations.iter().find(|r|r.occurrence_id==allocation.occurrence_id).ok_or(Error::Conflict)?;
 let reference=parent.reference(held)?;
 let budget=Budget{deadline_at:held.deadline_at,max_model_calls:held.model_calls,max_tool_calls:held.tool_calls,max_output_bytes:held.output_bytes,max_tokens:None,max_cost_micros:None,payment:Payment::LocalOnly};
 let approved=Review::prepare_rule_occurrence(&goal,evidence,budget,reference,held.admission.as_ref(),now).map_err(|_|Error::Conflict)?;
 checked(&valid)?;authority.bind_rule_admission(approved.spec(),clock(),valid).map_err(Failure::control_error)?;
 checked(&valid)?;let certified=authority.certify(&approved,&goal,clock()).map_err(Failure::control_error)?;checked(&valid)?;let queued=commit(store.enqueue_certified(&certified,clock())?)?;
 Ok(Admitted{goal_id:queued.id,rule_id:policy.rule.id.clone(),occurrence_id:occurrence,queued:queued.state==model::State::Queued,existing_admission:false})
}

/// Explicit native control for an already-consented due occurrence. This is not
/// another consent or a timer, and never substitutes a different model/budget.
#[tauri::command]
pub async fn pet_runtime_rule_occurrence_admit(app:tauri::AppHandle,window:tauri::WebviewWindow,input:PreviewInput)->Result<Admitted,String>{
 check(&window)?;
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err("ruleUnsupportedPlatform".into())}
 #[cfg(target_os="linux")]{
 let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"ruleReviewClosed")?;
 tauri::async_runtime::spawn_blocking(move||{
  use tauri::Manager;
  crate::pet_data::bootstrap::write_allowed().map_err(|_|"ruleProfileInactive")?;
  let rule=read_rule_at(&path(),&input.pet_id,&input.rule_id,input.revision).map_err(error)?;
  let a=Authority::existing(&super::super::admission_review::authority_root()).map_err(|_|"ruleOwnAuthorityRequired")?;
  let decision=a.rule_decision_for(&input.pet_id,&input.rule_id).map_err(|_|"ruleAuthorityUnavailable")?.ok_or("ruleOwnAuthorityRequired")?;let grant=a.rule_grant(&decision).map_err(|_|"ruleAuthorityUnavailable")?;
  if grant.policy.rule!=rule||grant.phase!=RulePhase::Active{return Err("ruleOwnAuthorityRequired".into());}
  let now=crate::assistant_memory::now();
  let index=grant.policy.schedule.slots.iter().enumerate().filter(|(_,s)|s.at.is_some_and(|at|at<=now&&now<at.saturating_add(u64::from(rule.definition.limits.occurrence_timeout_seconds)*1000))).map(|(i,_)|i).last().ok_or("ruleNoDueOccurrence")?;
  let store=Store::ensure_native(&crate::settings::config_dir().join("pet-runtime-v1")).map_err(|_|"ruleGoalStorageUnavailable")?;let main=app.state::<Chat>();
  admit_at(&store,&a,&crate::settings::config_dir().join("pet-runtime-contexts-v1"),&main,&decision,index,crate::assistant_memory::now,|p|frame.current()&&crate::pet_data::bootstrap::write_allowed().is_ok()&&authority_api::current(&app,p),|g,n|super::super::admission_review::evidence(&app,g,n,None)?.map_err(|_|Error::Conflict)).map_err(|e|format!("ruleOccurrence:{e:?}; consulta o objetivo e a prontidão do modelo antes de repetir. Nenhum saldo é renovado."))
 }).await.map_err(|_|"ruleOccurrenceInterrupted; consulta os registos antes de repetir".to_string())?
}
}

#[cfg(target_os="linux")]pub(in crate::pet_runtime)fn prepare_draft(store:&Store,contexts:&Path,main:&Chat,parent:&RuleGrant,index:usize,clock:&impl Fn()->u64,valid:impl Fn()->bool)->model::Result<Record>{
 let spec=parent.policy.goal_spec(index).ok_or(Error::InvalidInput)?;let occurrence=parent.policy.occurrence_id(index).ok_or(Error::InvalidInput)?;
 // Fixed deterministic goal/request identity is persisted before any authority
 // grant or model effect. Stop/edit of this goal is never auto-resumed.
 let mut goal=commit(store.create(spec,clock())?)?;checked(&valid)?;
 if !goal.admissions.is_empty(){return Ok(goal);}
 if goal.state!=model::State::Draft{return Err(Error::Conflict);}
 if goal.tasks.is_empty(){checked(&valid)?;goal=commit(store.mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,format!("rt-{occurrence}"),Action::ProposeTask{description:parent.policy.rule.definition.name.clone()},clock())?)?;}
 if goal.plans.is_empty(){checked(&valid)?;goal=commit(store.mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,format!("rp-{occurrence}"),Action::ProposePlan{task_id:goal.tasks.first().ok_or(Error::Conflict)?.id.clone(),steps:vec![StepInput{description:"Produzir a resposta finita desta ocorrência".into(),postcondition:"Resposta conservada; critérios ainda exigem verificação".into(),operation:None}]},clock())?)?;}
 if goal.execution_context.is_none(){checked(&valid)?;let(binding,_)=context::prepare(contexts,&goal,main,model::PreparationMode::Dedicated,&format!("rc-{occurrence}"))?;checked(&valid)?;goal=commit(store.mutate(&goal.id,&goal.spec.owner.pet_id,&goal.revision,format!("rb-{occurrence}"),Action::BindContext{binding},clock())?)?;}
 Ok(goal)
}
