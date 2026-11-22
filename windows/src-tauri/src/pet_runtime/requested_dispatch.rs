//! Initial requested execution in the existing lease service. Persistent intent
//! is the exact signed admission policy; these handles are mechanical only.
use super::*;
use crate::pet_runtime::{admission::Denial,model::{State,CommitStatus},store::nonce};
use crate::pet_resources::{GoalRequest,Failure as ResourceFailure,WaitInfo};
use std::{sync::atomic::AtomicBool,time::{Duration,Instant}};

#[derive(Clone,Copy,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(super)enum Phase{Queued,Scheduled,WaitingResource,Validating,ConfirmingClaim,RecoveryRequired,Stopped}
#[derive(Clone,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct DispatchStatus{phase:Phase,reason:Option<Error>,#[serde(skip_serializing_if="Option::is_none")]not_before_at:Option<u64>,#[serde(skip_serializing_if="Option::is_none")]resource_wait:Option<WaitInfo>}
#[derive(Clone)]pub(super)struct Waiting{source:Record,resource:Option<GoalRequest>,claim_request:String,phase:Phase,reason:Option<Error>,advancing:bool}
fn requested(goal:&Record)->bool{goal.admissions.last().is_some_and(|a|a.spec.execution_requested()&&a.spec.binding_format())}
fn same_admission(source:&Record,current:&Record)->bool{
 source.id==current.id&&source.spec==current.spec&&source.execution_context==current.execution_context
  &&source.admissions.last().zip(current.admissions.last()).is_some_and(|(a,b)|model::fingerprint(&a.spec).ok()==model::fingerprint(&b.spec).ok()&&model::fingerprint(&a.proof).ok()==model::fingerprint(&b.proof).ok())
}
pub(super)fn status(service:&Service,goal:&Record)->Option<DispatchStatus>{status_at(service,goal,crate::assistant_memory::now())}
fn status_at(service:&Service,goal:&Record,now:u64)->Option<DispatchStatus>{
 if !requested(goal){return None;}
 if let Some(wait)=service.waiting.get(&goal.id).filter(|w|same_admission(&w.source,goal)){
  return Some(DispatchStatus{phase:wait.phase,reason:wait.reason,not_before_at:None,resource_wait:wait.resource.as_ref().and_then(GoalRequest::waiting_info)});
 }
 (goal.state==State::Queued).then(||DispatchStatus{phase:if goal.admissions.last().is_some_and(|a|now>=a.spec.budget.deadline_at){Phase::Stopped}else if goal.admissions.last().is_some_and(|a|!a.spec.execution_due(now)){Phase::Scheduled}else{Phase::Queued},reason:None,not_before_at:goal.admissions.last().and_then(|a|a.spec.not_before()),resource_wait:None})
}
fn apply_update(wait:&mut Waiting,phase:Phase,reason:Option<Error>,halt:bool){
 // A result returning from IO cannot resurrect a wait retired by Stop/expiry.
 if wait.resource.is_none()&&matches!(wait.phase,Phase::Stopped){return;}
 wait.phase=phase;wait.reason=reason;
 if halt{if let Some(resource)=wait.resource.take(){resource.cancel();}}
}
fn same_wait(a:&Waiting,b:&Waiting)->bool{a.claim_request==b.claim_request&&same_admission(&a.source,&b.source)}
fn update(original:&Waiting,phase:Phase,reason:Option<Error>,halt:bool){
 if let Ok(mutex)=service(){if let Ok(mut service)=mutex.lock(){if let Some(wait)=service.waiting.get_mut(&original.source.id).filter(|w|same_wait(w,original)){apply_update(wait,phase,reason,halt);}}}
}
fn finish_advance_in(service:&mut Service,original:&Waiting,result:model::Result<bool>){
  let Some(wait)=service.waiting.get_mut(&original.source.id).filter(|w|same_wait(w,original))else{return;};wait.advancing=false;
  match result{Ok(true)=>{service.waiting.remove(&original.source.id);},Ok(false)=>apply_update(wait,Phase::ConfirmingClaim,None,false),Err(Error::Busy)=>apply_update(wait,Phase::WaitingResource,None,false),Err(error)=>apply_update(wait,Phase::RecoveryRequired,Some(error),true)}

}
fn finish_advance(original:&Waiting,result:model::Result<bool>){
 if let Ok(mutex)=service(){if let Ok(mut service)=mutex.lock(){finish_advance_in(&mut service,original,result);}}
}
fn take_advance(service:&mut Service,id:&str)->Option<Waiting>{
 if service.waiting.values().filter(|w|w.advancing).count()>=8{return None;}
 let wait=service.waiting.get_mut(id).filter(|w|w.resource.is_some()&&!w.advancing)?;wait.advancing=true;Some(wait.clone())
}
// No catalog or policy IO belongs here. Removing this request handle does not
// remove Pending/Owned, nor release a hold still borrowed by metadata/worker IO.
fn expire_waits(service:&mut Service,now:u64){
 for wait in service.waiting.values_mut(){
  let Some(resource)=wait.resource.as_ref()else{continue;};
  let Some(spec)=wait.source.admissions.last().map(|row|&row.spec)else{continue;};
  let rollback=now<spec.approved_at||!spec.execution_due(now);
  if rollback||now>=spec.budget.deadline_at||resource.stopped(){
   wait.phase=Phase::Stopped;wait.reason=rollback.then_some(Error::ClockRollback);
   if let Some(resource)=wait.resource.take(){resource.cancel();}
  }
 }
}
// Keep early Stop/monotonic/clock-rollback barriers while their signed wall
// deadline could still admit a new request. Once that original deadline has
// elapsed, a new admission may evict only these inert presentation entries.
fn prune_expired_statuses(service:&mut Service,now:u64){
 service.waiting.retain(|_,wait|!(!wait.advancing&&wait.resource.is_none()&&matches!(wait.phase,Phase::Stopped)&&wait.source.admissions.last().is_some_and(|a|now>=a.spec.budget.deadline_at)));
}
pub(super)fn housekeep(){
 if let Ok(mutex)=service(){if let Ok(mut service)=mutex.try_lock(){expire_waits(&mut service,crate::assistant_memory::now());}}
}
fn heartbeat(){
 housekeep();
 let resources=service().ok().and_then(|mutex|mutex.try_lock().ok().map(|service|service.waiting.values().filter_map(|w|w.resource.clone()).collect::<Vec<_>>()));
 if let Some(resources)=resources{for resource in resources{let _=resource.poll();}}
}
pub(super)fn stop(goal:&Record){
 if let Ok(mutex)=service(){if let Ok(mut service)=mutex.try_lock(){if let Some(wait)=service.waiting.get_mut(&goal.id).filter(|w|w.source.spec.owner.pet_id==goal.spec.owner.pet_id){
  wait.phase=Phase::Stopped;if let Some(resource)=wait.resource.take(){resource.cancel();}
 }}}
}
fn read_current(store:&Store,wait:&Waiting)->model::Result<Record>{
 let current=store.read_control(&wait.source.id,&wait.source.spec.owner.pet_id)?;
 if !same_admission(&wait.source,&current)||!matches!(current.state,State::Queued|State::Leased){return Err(Error::Conflict);}
 if current.state==State::Queued&&current.revision!=wait.source.revision{return Err(Error::Conflict);}Ok(current)
}
fn resource_error(error:ResourceFailure)->Error{match error{ResourceFailure::Busy=>Error::Busy,ResourceFailure::Unavailable=>Error::StorageUnavailable,_=>Error::Conflict}}

async fn advance(app:&AppHandle,store:&Store,wait:&Waiting)->model::Result<bool>{
 let current=read_current(store,wait)?;
 let original_preparation={let service=service()?.lock().map_err(|_|Error::StorageUnavailable)?;
  if let Some(entry)=service.entries.get(&current.id){
   if entry.source_revision!=wait.source.revision||entry.request_id!=wait.claim_request{return Err(Error::Conflict);}true
  }else{false}
 };
 // A stored lease from another process is not an executable native owner.
 if current.state==State::Leased&&!original_preparation{return Err(Error::Conflict);}
 let hold=wait.resource.as_ref().ok_or(Error::Conflict)?.poll().map_err(resource_error)?;
 if !original_preparation{
  match admission_review::evidence(app,&current,crate::assistant_memory::now(),Some(&hold))?{
   Ok(_)=>{},
   Err(Denial::NotReady|Denial::Stale)=>{
    update(wait,Phase::Validating,None,false);
    super::super::readiness_api::inspect_requested(app,&current,hold.clone()).await?;
   },
   Err(_)=>return Err(Error::Conflict),
  }
 }
 let _=read_current(store,wait)?;
 update(wait,Phase::ConfirmingClaim,None,false);
 let claim=claim_for_app(app,ClaimInput{pet_id:wait.source.spec.owner.pet_id.clone(),goal_id:wait.source.id.clone(),revision:wait.source.revision.clone(),request_id:wait.claim_request.clone()},Some(hold))?;
 if claim.receipt.status==CommitStatus::Uncertain||!claim.owned{return Ok(false);}
 // Same native start path as the manual command, with its original owner and
 // dispatcher CAS. No reconstructed approval or alternate model is supplied.
 super::super::driver_api::start_for_app(app,super::super::driver_api::DriverInput{pet_id:wait.source.spec.owner.pet_id.clone(),goal_id:wait.source.id.clone(),lease_id:claim.lease.id})?;
 Ok(true)
}

pub(super)async fn tick(app:&AppHandle,root:&std::path::Path,authority:&std::path::Path){
 housekeep();
 let Ok(Some(store))=Store::existing_verified(root,authority)else{return;};
 let Ok(list)=store.list(None)else{return;};
 let mut rows:Vec<_>=list.goals.into_iter().filter(|g|!g.historical_source).collect();rows.sort_by(|a,b|a.id.cmp(&b.id));
 let keys={let Ok(mutex)=service()else{return;};let Ok(mut service)=mutex.try_lock()else{return;};
  // Terminal/replaced requests never retain a queue ticket. A native Pending
  // entry still owns its hold until its existing authenticated cleanup/drain.
  service.waiting.retain(|id,w|{
   let keep=w.advancing||rows.iter().any(|g|&g.id==id&&matches!(g.state,State::Queued|State::Leased));
   if !keep{if let Some(resource)=&w.resource{resource.cancel();}}keep
  });
  rows.retain(|g|matches!(g.state,State::Queued|State::Leased));
  let pivot=service.dispatch_after.as_ref().map_or(0,|after|rows.partition_point(|g|&g.id<=after));
  rows[pivot..].iter().chain(rows[..pivot].iter()).take(8).map(|g|(g.id.clone(),g.pet_id.clone())).collect::<Vec<_>>()
 };
 for(id,pet)in keys{
  let wait={
   let Ok(current)=store.read_control(&id,&pet)else{continue;};
   if current.admissions.last().is_some_and(|a|!a.spec.execution_due(crate::assistant_memory::now())){
    // No resource ticket, owner, readiness/model IO or attempt is created.
    // Advance the existing fair scan cursor so future goals cannot starve due work.
    if let Ok(mutex)=service(){if let Ok(mut service)=mutex.try_lock(){service.dispatch_after=Some(id.clone());}}
    continue;
   }
   if current.state==State::Queued&&current.admissions.last().is_some_and(|a|a.spec.work_disclosure.is_some()){
    if !super::super::work_disclosure::due(&id){continue;}
    let now=crate::assistant_memory::now();let Ok(source)=store.acknowledge_requested(&id,&pet,&current.revision,now)else{continue;};
    if !super::super::work_disclosure::ready(root,authority,&source,now).unwrap_or(false){continue;}
   }

   let Ok(mutex)=service()else{return;};let Ok(mut service)=mutex.try_lock()else{continue;};service.dispatch_after=Some(id.clone());
   if !requested(&current){continue;}
   if !service.waiting.contains_key(&id){
    if current.state!=State::Queued||service.entries.contains_key(&id){continue;}
    prune_expired_statuses(&mut service,crate::assistant_memory::now());
    if service.entries.len()+service.waiting.len()>=64{continue;}
    // ACK the exact signed row before even acquiring a client resource.
    let Ok(source)=store.acknowledge_requested(&id,&pet,&current.revision,crate::assistant_memory::now())else{continue;};
    let Some(budget)=source.admissions.last().map(|a|&a.spec.budget)else{continue;};
    let Some(left)=budget.deadline_at.checked_sub(crate::assistant_memory::now()).filter(|left|*left>0&&*left<=86_400_000)else{continue;};
    let Some(deadline)=Instant::now().checked_add(Duration::from_millis(left))else{continue;};
    let chat=app.state::<crate::chat::Chat>();
    let Ok(resource)=(if source.admissions.last().is_some_and(|a|a.spec.deterministic()){chat.runtime_resources().request_runtime_goal(&source.spec.owner.provider,Arc::new(AtomicBool::new(false)),deadline)}else{chat.runtime_resources().request_goal(&source.spec.owner.provider,Arc::new(AtomicBool::new(false)),deadline)})else{continue;};
    let Ok(claim_request)=nonce()else{resource.cancel();continue;};
    service.waiting.insert(id.clone(),Waiting{source,resource:Some(resource),claim_request,phase:Phase::WaitingResource,reason:None,advancing:false});
   }
   take_advance(&mut service,&id)
  };
  let Some(wait)=wait else{continue;};
  let app=app.clone();let root=root.to_owned();let authority=authority.to_owned();
  tauri::async_runtime::spawn(async move{
   let result=match Store::existing_verified(&root,&authority){
    Ok(Some(store))=>{let advancing=advance(&app,&store,&wait);tokio::pin!(advancing);loop{tokio::select!{
     result=&mut advancing=>break result,
     _=tokio::time::sleep(Duration::from_millis(200))=>heartbeat(),
    }}},
    _=>Err(Error::StorageUnavailable),
   };
   finish_advance(&wait,result);
  });
 }
}

#[cfg(test)]#[path="requested_dispatch_tests.rs"]mod tests;
