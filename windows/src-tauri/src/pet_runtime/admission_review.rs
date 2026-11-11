//! Window-scoped exact review tokens. Confirmation only enqueues; no dispatch.
use super::{admission::*,model::{self,Error,Record}};
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
use tauri::{AppHandle,Manager};

#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewTicket{pub token:String,pub proposal:ReviewProposal,pub approval_scope:&'static str}
#[cfg(target_os="linux")]#[path="admission_confirmation.rs"]mod confirmation;
enum Native{
    Review(Review),
    #[cfg(target_os="linux")]Approved{approved:Approved,authority:super::authority::Authority,acknowledged:bool},
    Ended,
}
#[derive(Clone)]pub(super)struct Origin{window:String,frame:crate::settings::review::Frame}
impl Origin{fn same(&self,other:&Self)->bool{self.window==other.window&&self.frame.same(&other.frame)&&self.frame.current()&&other.frame.current()}}
#[cfg(target_os="linux")]#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum ReviewPath{System,Admission,Planning,Continuation,Deterministic,NativeContinuation,ResultDisclosure}
#[cfg(target_os="linux")]fn path(proposal:&ReviewProposal)->ReviewPath{if proposal.native_system_plan.is_some(){return ReviewPath::System;}if proposal.result_disclosure.is_some(){return ReviewPath::ResultDisclosure;}if proposal.operation=="goalDeterministic"{if proposal.continuation.is_some(){ReviewPath::NativeContinuation}else{ReviewPath::Deterministic}}else if proposal.continuation.is_some(){ReviewPath::Continuation}else if proposal.planning_policy.is_some(){ReviewPath::Planning}else{ReviewPath::Admission}}
pub(super)struct Pending{origin:Origin,proposal:ReviewProposal,approval_scope:&'static str,native:Arc<Mutex<Native>>}
impl Pending{fn ticket(&self,token:String)->ReviewTicket{ReviewTicket{token,proposal:self.proposal.clone(),approval_scope:self.approval_scope}}}
#[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Arc<Pending>>,retired:Vec<Arc<Pending>>}
impl Reviews{
    fn sweep(&mut self,now:u64){
        let expired=self.pending.iter().filter_map(|(token,p)|{
            let keep=match p.native.try_lock(){Ok(n)=>match &*n{Native::Ended=>false,#[cfg(target_os="linux")]Native::Approved{acknowledged:false,..}=>true,_=>p.origin.frame.current()&&p.proposal.expires_at>now},Err(_)=>true};
            (!keep).then(||token.clone())
        }).collect::<Vec<_>>();
        for token in expired{if let Some(pending)=self.pending.remove(&token){self.retired.push(pending);}}
    }
    fn take_retired(&mut self)->Vec<Arc<Pending>>{std::mem::take(&mut self.retired)}
    #[cfg(test)]pub(super)fn origin_for_test(&self,window:&str)->Origin{self.pending.values().find(|p|p.origin.window==window).map(|p|p.origin.clone()).unwrap_or_else(||Origin{window:window.into(),frame:crate::settings::review::Frame::fixture()})}
    #[cfg(target_os="linux")]
    fn retained_approved(&mut self,origin:&Origin,goal:&Record,kind:ReviewPath,now:u64)->model::Result<Option<ReviewTicket>>{
        if !origin.frame.current(){return Err(Error::Conflict);}self.sweep(now);
        let mut alias=None;
        for(token,p)in &self.pending{
            if p.origin.window!=origin.window||p.proposal.goal_id!=goal.id||p.proposal.owner.pet_id!=goal.spec.owner.pet_id||path(&p.proposal)!=kind{continue;}
            let native=p.native.try_lock().map_err(|_|Error::Busy)?;
            if !matches!(*native,Native::Approved{acknowledged:false,..}){continue;}
            if p.origin.same(origin){let mut ticket=p.ticket(token.clone());ticket.approval_scope="confirmOriginalAdmission";return Ok(Some(ticket));}
            if p.origin.frame.current(){return Err(Error::Conflict);}
            alias=Some((p.proposal.clone(),p.native.clone()));break;
        }
        if let Some((proposal,native))=alias{
            if !origin.frame.current(){return Err(Error::Conflict);}
            let token=super::store::nonce()?;
            let pending=Arc::new(Pending{origin:origin.clone(),proposal,approval_scope:"confirmOriginalAdmission",native:native.clone()});
            self.pending.retain(|_,p|!Arc::ptr_eq(&p.native,&native));let ticket=pending.ticket(token.clone());self.pending.insert(token,pending);return Ok(Some(ticket));
        }Ok(None)
    }

    pub(super)fn insert(&mut self,origin:&Origin,review:Review,token:String,now:u64)->model::Result<ReviewTicket>{
        // Keep an outside-lock disposal reference even on duplicate/conflict/error.
        let proposal=review.proposal();let incoming=Arc::new(Pending{origin:origin.clone(),proposal:proposal.clone(),approval_scope:"admitGoal",native:Arc::new(Mutex::new(Native::Review(review)))});self.retired.push(incoming.clone());
        if !origin.frame.current(){return Err(Error::Conflict);}if origin.window.is_empty()||!model::token(&token){return Err(Error::InvalidInput);}
        // A running transaction keeps its slot even when the public review TTL
        // expires. No filesystem/network IO holds this global cache mutex.
        self.sweep(now);
        for (token,p)in &self.pending{
            let old=&p.proposal;if p.origin.window==origin.window&&old.goal_id==proposal.goal_id{
                if !p.origin.same(origin){return Err(Error::Conflict);}
                let semantic=|p:&ReviewProposal|model::fingerprint(&(&(&p.native_system_plan.as_ref().map(|p|p.semantic_hash()).transpose()?,&p.native_peer_plan.as_ref().map(|p|p.semantic_hash()).transpose()?,&p.execution_start,&p.result_disclosure,&p.work_disclosure,&p.native_verification,&p.operation),&p.restored_from,&p.memory_data,&p.continuation,&p.goal_id,&p.revision,&p.request_id,&p.owner,&p.context,&p.intent,&p.criteria,&p.resolved_model,&p.policy_revision,&p.descriptor_revision,&p.budget,&p.planning_policy));
                if old.engine_binding!=proposal.engine_binding||semantic(old)?!=semantic(&proposal)?{
                    #[cfg(target_os="linux")]if p.native.try_lock().is_ok_and(|n|matches!(*n,Native::Approved{acknowledged:true,..})){continue;}
                    return Err(Error::Conflict);
                }
                return Ok(p.ticket(token.clone()));
            }
        }
        if self.pending.len()>=64{return Err(Error::LimitReached);}if self.pending.contains_key(&token){return Err(Error::Conflict);}
        self.pending.insert(token.clone(),incoming);Ok(ReviewTicket{token,proposal,approval_scope:"admitGoal"})
    }
    pub(super)fn lookup(&self,origin:&Origin,input:&ConfirmInput,now:u64)->model::Result<Arc<Pending>>{
        let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;let v=&p.proposal;
        if !p.origin.same(origin)||v.owner.pet_id!=input.pet_id||v.goal_id!=input.goal_id||v.revision!=input.revision||now<v.prepared_at{return Err(Error::Conflict);}
        Ok(p.clone())
    }
    #[cfg(test)]pub(super)fn take(&mut self,origin:&Origin,input:&ConfirmInput,now:u64)->model::Result<Review>{
        let p=self.lookup(origin,input,now)?;let mut native=p.native.try_lock().map_err(|_|Error::Busy)?;
        if now>=p.proposal.expires_at{*native=Native::Ended;self.pending.remove(&input.token);return Err(Error::Conflict);}
        let old=std::mem::replace(&mut*native,Native::Ended);self.pending.remove(&input.token);
        if let Native::Review(review)=old{Ok(review)}else{Err(Error::Conflict)}
    }
}
pub(super)fn sweep_context(){
    let retired=if let Ok(mut cache)=reviews().lock(){cache.sweep(crate::assistant_memory::now());cache.take_retired()}else{return;};
    if !retired.is_empty(){tauri::async_runtime::spawn_blocking(move||drop(retired));}
}
/// Only in-memory changes hold the global cache. Native child reaping happens
/// after releasing it, including rejected/duplicate incoming reviews.
fn with_reviews<T>(operation:impl FnOnce(&mut Reviews)->model::Result<T>)->model::Result<T>{
    let(result,retired)={let mut cache=reviews().lock().map_err(|_|Error::StorageUnavailable)?;let result=operation(&mut cache);(result,cache.take_retired())};
    drop(retired);result
}
#[cfg(target_os="linux")]async fn capture_origin(window:&tauri::WebviewWindow)->model::Result<Origin>{
    let frame=crate::settings::review::capture_window(window).await.map_err(|_|Error::Conflict)?;Ok(Origin{window:window.label().into(),frame})
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareInput{#[serde(default)]pub not_before_at:Option<u64>,#[serde(default)]pub start:Option<super::planning::Start>,pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConfirmInput{#[serde(default)]pub disclosure_consent:Option<super::work_disclosure::Consent>,#[serde(default)]pub memory_consent:Option<crate::pet_memory::context_model::GoalConsent>,pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum PrepareResult{Review{ticket:ReviewTicket},Blocked{reason:Denial}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ConfirmResult{Queued{receipt:model::Receipt},Declined,Blocked{reason:Denial,retryable:bool},AuthorityBlocked{reason:super::authority::Failure,retryable:bool}}
#[cfg(target_os="linux")]fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
#[cfg(target_os="linux")]fn captured_evidence(app:&AppHandle,goal:&Record,now:u64,owned:Option<&crate::pet_resources::Hold>,admission:bool)->model::Result<std::result::Result<(Evidence,crate::pet_config::Configuration),Denial>>{
    if let Some(p)=goal.admissions.last().and_then(|r|r.spec.result_disclosure.as_ref()){let projection=crate::pet_capabilities::native_projection(app,owned).map_err(|_|Error::StorageUnavailable)?;let c=projection.configuration.ok_or(Error::StorageUnavailable)?;return Ok(result_evidence(app,goal,Some(p),now)?.map(|e|(e,c)));}
    let projection=if admission{crate::pet_capabilities::admission_projection(app)}else{crate::pet_capabilities::native_projection(app,owned)}.map_err(|_|Error::StorageUnavailable)?;
    let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
    if goal.admissions.last().is_some_and(|r|r.spec.system()){
        let config=projection.configuration.ok_or(Error::StorageUnavailable)?;
        if goal.admissions.last().and_then(|r|r.spec.native_system_plan.as_ref()).is_none_or(|p|now<p.prepared_at||now>=p.expires_at||!p.targets_current(&config)){return Ok(Err(Denial::PolicyChanged));}
        let policy=super::native_validity::system_policy_revision(&config,goal)?;
        return Ok(Evidence::observe_system(goal,descriptor,policy,now).map(|e|(e,config)));
    }
    if goal.admissions.last().is_some_and(|r|r.spec.deterministic()){
        let config=projection.configuration.ok_or(Error::StorageUnavailable)?;
        if matches!(goal.state,model::State::Queued|model::State::Leased){if let Some(row)=goal.admissions.last(){if let Some(p)=&row.spec.native_peer_plan{
            if now<p.prepared_at||now>=p.expires_at||!p.matches(goal,&row.spec)||!p.targets_current(&config,&projection.settings){return Ok(Err(Denial::PolicyChanged));}
            p.sources_current(goal,&authority_root(),now)?;
        }}}
        let policy=super::native_validity::deterministic_policy_revision(&config,goal)?;
        return Ok(Evidence::observe_deterministic(goal,descriptor,policy,now).map(|e|(e,config)));
    }
    let config=projection.configuration.ok_or(Error::StorageUnavailable)?;
    let policy=super::native_validity::policy_revision(&config,&projection.settings,goal)?;
    Ok(Evidence::observe_native(goal,descriptor,policy,now,false,false,&projection.settings,&config).map(|e|(e,config)))
}
#[cfg(target_os="linux")]pub(super)fn evidence(app:&AppHandle,goal:&Record,now:u64,owned:Option<&crate::pet_resources::Hold>)->model::Result<std::result::Result<Evidence,Denial>>{
    Ok(captured_evidence(app,goal,now,owned,false)?.map(|(e,_)|e))
}
#[cfg(target_os="linux")]pub(super)fn owner_resumption_evidence(app:&AppHandle,goal:&Record,now:u64)->model::Result<std::result::Result<OwnerResumptionEvidence,Denial>>{
    let projection=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;
    if !projection.preferences_ready{return Ok(Err(Denial::NotReady));}
    let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;
    let descriptor=projection.snapshot.pets.iter().find(|d|d.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
    let policy=super::native_validity::policy_revision(config,&projection.settings,goal)?;
    let model=&goal.admissions.last().ok_or(Error::Conflict)?.spec.resolved_model;
    let server=super::memory_review::server_for_goal(goal,config,&projection.settings,model).map_err(|_|Error::Conflict)?;
    let memory=super::memory_review::Captured::native(goal,&server,model,now).map_err(|_|Error::Conflict)?;
    Ok(OwnerResumptionEvidence::observe(goal,descriptor,policy,now,memory))
}
#[cfg(target_os="linux")]fn deterministic_evidence(app:&AppHandle,goal:&Record,now:u64)->model::Result<std::result::Result<Evidence,Denial>>{
 let projection=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;
 if !projection.preferences_ready{return Ok(Err(Denial::NotReady));}let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;
 let d=projection.snapshot.pets.iter().find(|p|p.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
 let policy=super::native_validity::deterministic_policy_revision(config,goal)?;Ok(Evidence::observe_deterministic(goal,d,policy,now))
}
#[cfg(target_os="linux")]fn system_evidence(app:&AppHandle,goal:&Record,now:u64)->model::Result<std::result::Result<Evidence,Denial>>{
 let projection=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;
 if !projection.preferences_ready{return Ok(Err(Denial::NotReady));}let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;
 let d=projection.snapshot.pets.iter().find(|p|p.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
 let policy=super::native_validity::system_policy_revision(config,goal)?;Ok(Evidence::observe_system(goal,d,policy,now))
}
#[cfg(target_os="linux")]fn system_observe(app:&AppHandle)->super::native_system_plan::Observe{
 let app=app.clone();Arc::new(move||{let p=crate::pet_capabilities::native_projection(&app,None).map_err(|_|Error::StorageUnavailable)?;if !p.preferences_ready{return Err(Error::Busy);}p.configuration.ok_or(Error::StorageUnavailable)})
}
#[cfg(target_os="linux")]fn peer_observe(app:&AppHandle)->super::native_peer_plan::Observe{
 let app=app.clone();Arc::new(move||{let p=crate::pet_capabilities::native_projection(&app,None).map_err(|_|Error::StorageUnavailable)?;
  if !p.preferences_ready{return Err(Error::Busy);}Ok((p.configuration.ok_or(Error::StorageUnavailable)?,p.settings))})
}
#[cfg(target_os="linux")]fn result_evidence(app:&AppHandle,g:&Record,reviewed:Option<&super::result_disclosure_policy::Policy>,now:u64)->model::Result<std::result::Result<Evidence,Denial>>{
 let projection=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;if !projection.preferences_ready{return Ok(Err(Denial::NotReady));}let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;let d=projection.snapshot.pets.iter().find(|d|d.pet_id==g.spec.owner.pet_id).ok_or(Error::Conflict)?;
 let app2=app.clone();let observe:super::result_disclosure::Observe=Arc::new(move||{let p=crate::pet_capabilities::native_projection(&app2,None).map_err(|_|Error::StorageUnavailable)?;if !p.preferences_ready{return Err(Error::Conflict);}Ok((p.configuration.ok_or(Error::StorageUnavailable)?,p.settings))});
 let policy=if let Some(p)=reviewed{p.clone()}else{
  let plan=g.plans.first().filter(|_|g.plans.len()==1).ok_or(Error::Conflict)?;let step=plan.steps.first().filter(|_|plan.steps.len()==1).ok_or(Error::Conflict)?;let args=super::derived_result_plan::Arguments::parse(step.operation.as_ref().ok_or(Error::Conflict)?)?;
  let data=super::result_disclosure::Captured::at(&authority_root(),&crate::pet_memory::context::Capture::catalog_path(),g,super::messages::Cause{task_id:plan.task_id.clone(),plan_id:plan.id.clone(),step_id:step.id.clone()},args,super::plans::fingerprint(g)?,observe.clone(),now)?;data.policy(g)?
 };
 let data=super::result_delivery_data::Captured::at(&authority_root(),&crate::pet_memory::context::Capture::catalog_path(),g,policy,observe,now)?;let revision=super::native_validity::deterministic_policy_revision(config,g)?;Ok(Evidence::observe_result(g,d,revision,now,data))
}
#[cfg(target_os="linux")]fn admission_evidence(app:&AppHandle,goal:&Record,now:u64)->model::Result<std::result::Result<Evidence,Denial>>{Ok(captured_evidence(app,goal,now,None,true)?.map(|(e,_)|e))}
#[cfg(target_os="linux")]fn context_guard(app:&AppHandle,goal:&Record)->model::Result<crate::chat::Chat>{
    let path=crate::settings::config_dir().join("pet-runtime-contexts-v1");
    let(_,chat)=super::context::open_with_authority(&path,goal,app.state::<crate::chat::Chat>().inner(),&authority_root())?;Ok(chat)
}
#[tauri::command]
pub async fn pet_runtime_prepare_admission(app:AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    if !model::key(&input.request_id){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
        if let Some(ticket)=with_reviews(|cache|cache.retained_approved(&origin,&goal,ReviewPath::Admission,now))?{return Ok(PrepareResult::Review{ticket});}
        let observation=match admission_evidence(&app,&goal,now)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare(&goal,observation,input.budget,input.request_id,super::store::nonce()?,now).and_then(|r|r.with_execution_start(input.start)).and_then(|r|r.with_not_before(input.not_before_at)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}
/// Native-only exact result disclosure. Numeric limits cannot manufacture the
/// signed source/receipt, consent, target, remaining budget or continuation.
#[tauri::command]
pub async fn pet_runtime_prepare_result_disclosure(app:AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<PrepareResult>{
 if !super::adapter::allowed(window.label())||!model::key(&input.request_id){return Err(Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
  let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
  if let Some(ticket)=with_reviews(|cache|cache.retained_approved(&origin,&goal,ReviewPath::ResultDisclosure,now))?{return Ok(PrepareResult::Review{ticket});}
  super::driver_api::retire_completed_context(&goal)?;let(goal,captured)=match store.prepare_continuation_review(&authority_root(),&input.goal_id,&input.pet_id,&input.revision,now,false)?{Ok(p)=>p,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  let evidence=match result_evidence(&app,&goal,None,now)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};let _context=context_guard(&app,&goal)?;
  let review=match Review::prepare_result_disclosure(&goal,evidence,input.budget,captured,input.request_id,super::store::nonce()?,now).and_then(|r|r.with_execution_start(input.start)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};Ok(PrepareResult::Review{ticket:with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?})
 }).await.map_err(|_|Error::StorageUnavailable)?}
}
/// Explicit operation mode. The client supplies only original references and
/// numeric limits; it cannot serialize Evidence/Review or select a fallback.
#[tauri::command]
pub async fn pet_runtime_prepare_deterministic(app:AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<PrepareResult>{
 if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}if !model::key(&input.request_id){return Err(Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
  let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
  if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
  if let Some(ticket)=with_reviews(|cache|cache.retained_approved(&origin,&goal,ReviewPath::Deterministic,now))?{return Ok(PrepareResult::Review{ticket});}
  let observation=match deterministic_evidence(&app,&goal,now)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};let _context=context_guard(&app,&goal)?;
  let review=match Review::prepare_deterministic(&goal,observation,input.budget,input.request_id,super::store::nonce()?,now){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  let review=if goal.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.as_ref().is_some_and(|o|o.operation_id=="pet.workReport"&&o.version==2)){let projection=crate::pet_capabilities::native_projection(&app,None).map_err(|_|Error::StorageUnavailable)?;let policy=super::work_disclosure::Policy::capture(&goal,&root(),&authority_root(),projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?,&projection.settings,now)?;review.with_work_disclosure(policy).map_err(|_|Error::Conflict)?}else{review};
  let review=match review.with_native_peer_plan(&goal,&root(),&authority_root(),peer_observe(&app),input.start,now){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  Ok(PrepareResult::Review{ticket:with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?})
 }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SystemPrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget,pub criteria:Vec<super::criteria::Input>,#[serde(default)]pub start:Option<super::planning::Start>}
/// Explicit native-user review. A model cannot serialize the retained target,
/// derive authority from a supplied PetId or bypass this owned confirmation.
#[tauri::command]
pub async fn pet_runtime_prepare_system(app:AppHandle,window:tauri::WebviewWindow,input:SystemPrepareInput)->model::Result<PrepareResult>{
 if !super::adapter::allowed(window.label())||!model::key(&input.request_id){return Err(Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
  let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
  if goal.revision!=input.revision||!origin.frame.current(){return Err(Error::Conflict);}let now=crate::assistant_memory::now();
  if let Some(ticket)=with_reviews(|c|c.retained_approved(&origin,&goal,ReviewPath::System,now))?{return Ok(PrepareResult::Review{ticket});}
  if !input.budget.valid_runtime(now){return Ok(PrepareResult::Blocked{reason:Denial::Budget});}
  if super::criteria::Binding::system_for(&goal,&input.criteria).is_err(){return Ok(PrepareResult::Blocked{reason:Denial::Invalid});}
  let observation=match system_evidence(&app,&goal,now)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};let _context=context_guard(&app,&goal)?;
  let config=system_observe(&app)()?;let operation=goal.plans.first().and_then(|p|p.steps.first()).and_then(|s|s.operation.as_ref()).ok_or(Error::InvalidInput)?;
  let target=match super::physical_target::Target::capture_system_review(operation,&goal.spec.owner.pet_id,&config,&||origin.frame.current()) {Ok(t)=>t,Err(Error::UnsupportedPlatform)=>return Ok(PrepareResult::Blocked{reason:Denial::Unsupported}),Err(Error::Conflict)=>return Ok(PrepareResult::Blocked{reason:Denial::PolicyChanged}),Err(e)=>return Err(e)};
  let now=crate::assistant_memory::now();if !origin.frame.current(){return Err(Error::Conflict);}
  let review=match Review::prepare_system(&goal,observation,input.budget,target,system_observe(&app),&input.criteria,input.request_id,super::store::nonce()?,now).and_then(|r|r.with_execution_start(input.start)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  Ok(PrepareResult::Review{ticket:with_reviews(|c|c.insert(&origin,review,super::store::nonce()?,now))?})
 }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[tauri::command]
pub async fn pet_runtime_confirm_admission(app:AppHandle,window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<ConfirmResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
        let pending=with_reviews(|cache|cache.lookup(&origin,&input,crate::assistant_memory::now()))?;
        let system=pending.proposal.native_system_plan.is_some();let deterministic=pending.proposal.operation=="goalDeterministic";let result_policy=pending.proposal.result_disclosure.clone();
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;
        pending.confirm_with_scopes(&store,&authority_root(),input.approve,input.memory_consent,input.disclosure_consent,|goal,now|{
            Ok(match (if let Some(p)=&result_policy{result_evidence(&app,goal,Some(p),now)}else if system{system_evidence(&app,goal,now)}else if deterministic{deterministic_evidence(&app,goal,now)}else{admission_evidence(&app,goal,now)})?{
                Ok(observation)=>Ok((observation,context_guard(&app,goal)?)),Err(reason)=>Err(reason),
            })
        },crate::assistant_memory::now)
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[cfg(target_os="linux")]pub(super)fn authority_root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-consent-v1")}
#[tauri::command]
pub async fn pet_runtime_admission_status(window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<super::authority::ProvenanceStatus>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(pet_id,goal_id);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&goal_id,&pet_id)?;
        let result=super::authority::Authority::existing(&authority_root()).and_then(|a|a.verify_queued(&goal,crate::assistant_memory::now()));
        Ok(super::authority::ProvenanceStatus::from_result(result))
    }).await.map_err(|_|Error::StorageUnavailable)?
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PlanningPrepareInput {#[serde(default)]pub catalog:Option<super::planning::Catalog>,pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget,pub limits:super::planning::Limits,#[serde(default)]pub start:Option<super::planning::Start>}
/// Opt-in planning admission. Only bounds and record references are input; the
/// model, configuration, policy and operator catalog come from one projection.
#[tauri::command]
pub async fn pet_runtime_prepare_planning(app:AppHandle,window:tauri::WebviewWindow,input:PlanningPrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    if !model::key(&input.request_id){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
        if let Some(ticket)=with_reviews(|cache|cache.retained_approved(&origin,&goal,ReviewPath::Planning,now))?{return Ok(PrepareResult::Review{ticket});}
        let(observation,config)=match captured_evidence(&app,&goal,now,None,true)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare_planning_catalog(&goal,observation,input.budget,input.limits,&config,input.catalog.unwrap_or(super::planning::Catalog::SpotifyTransportV1),input.request_id,super::store::nonce()?,now).and_then(|r|r.with_start(input.start)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ContinuationPrepareInput{#[serde(default)]pub review_uncertain_effects:bool,pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget,pub limits:Option<super::planning::Limits>,#[serde(default)]pub start:Option<super::planning::Start>}
#[cfg(target_os="linux")]
impl Reviews{
    fn retained_continuation(&mut self,origin:&Origin,input:&ContinuationPrepareInput,goal:&Record,now:u64)->model::Result<Option<ReviewTicket>>{
        if let Some(ticket)=self.retained_approved(origin,goal,if goal.admissions.last().is_some_and(|r|r.spec.deterministic()){ReviewPath::NativeContinuation}else{ReviewPath::Continuation},now)?{return Ok(Some(ticket));}
        for(token,p)in &self.pending{
            let proposal=&p.proposal;if proposal.result_disclosure.is_some()||!p.origin.same(origin)||proposal.goal_id!=input.goal_id||proposal.owner.pet_id!=input.pet_id||proposal.continuation.is_none(){continue;}
            let native=p.native.try_lock().map_err(|_|Error::Busy)?;
            let retained=match &*native{Native::Review(_)=>proposal.continuation.as_ref().is_none_or(|c|c.uncertain_effects.is_none()||input.review_uncertain_effects)&&proposal.revision==input.revision&&now>=proposal.prepared_at&&now<proposal.expires_at,Native::Approved{acknowledged:false,..}=>proposal.revision==input.revision||goal.events.iter().any(|e|e.request_id==proposal.request_id&&matches!(e.kind,model::EventKind::AdmissionQueued)),_=>false};
            if retained{return Ok(Some(p.ticket(token.clone())));}
        }Ok(None)
    }
}
#[tauri::command]
pub async fn pet_runtime_prepare_continuation(app:AppHandle,window:tauri::WebviewWindow,input:ContinuationPrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    if !model::key(&input.request_id){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}
        let now=crate::assistant_memory::now();
        if goal.admissions.last().is_some_and(|r|r.spec.deterministic()){return Ok(PrepareResult::Blocked{reason:Denial::Unsupported});}
        if let Some(ticket)=with_reviews(|cache|cache.retained_continuation(&origin,&input,&goal,now))?{return Ok(PrepareResult::Review{ticket});}
        super::driver_api::retire_completed_context(&goal)?;
        let(goal,captured)=match store.prepare_continuation_review(&authority_root(),&input.goal_id,&input.pet_id,&input.revision,now,input.review_uncertain_effects)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let(observation,config)=match captured_evidence(&app,&goal,now,None,true)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare_continuation(&goal,observation,input.budget,input.limits,&config,captured,input.request_id,super::store::nonce()?,now).and_then(|r|r.with_start(input.start)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

/// Continuation of native literal work is explicit; no provider readiness,
/// model, planning catalog, uncertain-effect override or fallback is invoked.
#[tauri::command]
pub async fn pet_runtime_prepare_deterministic_continuation(app:AppHandle,window:tauri::WebviewWindow,input:ContinuationPrepareInput)->model::Result<PrepareResult>{
 if !super::adapter::allowed(window.label())||!model::key(&input.request_id){return Err(Error::InvalidInput);}
 if input.limits.is_some()||input.review_uncertain_effects{return Ok(PrepareResult::Blocked{reason:Denial::Unsupported});}
 #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]{let origin=capture_origin(&window).await?;tauri::async_runtime::spawn_blocking(move||{
  let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
  if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
  if goal.admissions.last().is_none_or(|r|!r.spec.deterministic()){return Ok(PrepareResult::Blocked{reason:Denial::Unsupported});}
  if let Some(ticket)=with_reviews(|cache|cache.retained_continuation(&origin,&input,&goal,now))?{return Ok(PrepareResult::Review{ticket});}
  let(goal,captured)=match store.prepare_continuation_review(&authority_root(),&input.goal_id,&input.pet_id,&input.revision,now,false)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  let observation=match deterministic_evidence(&app,&goal,now)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};let _context=context_guard(&app,&goal)?;
  let review=match Review::prepare_deterministic_continuation(&goal,observation,input.budget,captured,input.request_id,super::store::nonce()?,now).and_then(|r|r.with_native_peer_plan(&goal,&root(),&authority_root(),peer_observe(&app),input.start,now)){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
  Ok(PrepareResult::Review{ticket:with_reviews(|cache|cache.insert(&origin,review,super::store::nonce()?,now))?})
 }).await.map_err(|_|Error::StorageUnavailable)?}
}
#[cfg(all(test,target_os="linux"))]#[path="continuation_review_tests.rs"]mod continuation_tests;

#[cfg(all(test,target_os="linux"))]#[path="system_admission_control_tests.rs"]mod system_control_tests;
