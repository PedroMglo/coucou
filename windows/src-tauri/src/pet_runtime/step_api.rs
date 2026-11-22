//! Exact native step reviews, retained targets and read-only recovery surfaces.
use super::model;
use serde::{Deserialize,Serialize};
#[cfg(target_os="linux")]use super::{authority::{Authority,Failure},lease_controller::step_controller::{Review,Ready,Proposal},steps::{self,Scope,Phase,Receipt},store::{Store,nonce}};
#[cfg(target_os="linux")]use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub plan_id:String,pub step_id:String,pub request_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConfirmInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RunInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub grant_id:String,pub plan_id:String,pub step_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReadInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub plan_id:String,pub step_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecoverInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub plan_id:String,pub step_id:String,pub revision:String,pub grant_id:String}

#[cfg(target_os="linux")]#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:Proposal}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ConfirmResult{Granted{scope:Scope},Declined,Blocked{reason:Failure}}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct View{pub scope:Scope,pub phase:Phase,pub result:Option<Receipt>,pub recovery:steps::Recovery,pub execution_available:bool}
#[cfg(target_os="linux")]impl From<steps::Assessment>for View{fn from(a:steps::Assessment)->Self{let r=a.record;Self{scope:r.scope,phase:r.phase,result:r.result,recovery:a.recovery,execution_available:false}}}
#[cfg(target_os="linux")]struct Pending{window:String,request:String,review:Review}
#[cfg(target_os="linux")]#[derive(Default)]pub(super)struct Cache{pending:BTreeMap<String,Pending>,ready:BTreeMap<String,Ready>,retired_pending:Vec<Pending>,retired_ready:Vec<Ready>}
#[cfg(target_os="linux")]impl Cache{
    fn sweep(&mut self,now:u64){
        let expired=self.pending.iter().filter_map(|(key,p)|{let v=p.review.proposal();(!(now>=v.prepared_at&&now<v.expires_at)).then(||key.clone())}).collect::<Vec<_>>();
        for key in expired{if let Some(p)=self.pending.remove(&key){self.retired_pending.push(p);}}
        let expired=self.ready.iter().filter_map(|(key,r)|(now>=r.scope.deadline_at).then(||key.clone())).collect::<Vec<_>>();
        for key in expired{self.remove_ready(&key);}
    }
    fn remove_ready(&mut self,key:&str){if let Some(r)=self.ready.remove(key){self.retired_ready.push(r);}}
    fn release(&mut self,lease:&str){
        let pending=self.pending.iter().filter_map(|(key,p)|(p.review.proposal().scope.lease_id==lease).then(||key.clone())).collect::<Vec<_>>();
        for key in pending{if let Some(p)=self.pending.remove(&key){self.retired_pending.push(p);}}
        let ready=self.ready.iter().filter_map(|(key,r)|(r.scope.lease_id==lease).then(||key.clone())).collect::<Vec<_>>();
        for key in ready{self.remove_ready(&key);}
    }
    fn take_retired(&mut self)->(Vec<Pending>,Vec<Ready>){(std::mem::take(&mut self.retired_pending),std::mem::take(&mut self.retired_ready))}
    pub(super)fn insert_pending(&mut self,token:String,window:String,request:String,review:Review,now:u64)->model::Result<()>{
        self.sweep(now);self.retired_pending.push(Pending{window,request,review});
        if self.pending.len()>=128||self.ready.len()>=64||self.pending.contains_key(&token){return Err(model::Error::LimitReached);}
        self.pending.insert(token,self.retired_pending.pop().ok_or(model::Error::Conflict)?);Ok(())
    }
    pub(super)fn take(&mut self,window:&str,input:&ConfirmInput,now:u64)->model::Result<Review>{
        self.sweep(now);let pending=self.pending.get(&input.token).ok_or(model::Error::Conflict)?;let p=pending.review.proposal();
        if pending.window!=window||p.scope.goal_id!=input.goal_id||p.scope.pet_id!=input.pet_id||p.scope.lease_id!=input.lease_id||p.goal_revision!=input.revision{return Err(model::Error::Conflict);}
        Ok(self.pending.remove(&input.token).ok_or(model::Error::Conflict)?.review)
    }
    pub(super)fn forget(&mut self,scope:&Scope){if let Ok(slot)=scope.slot(){if self.ready.get(&slot).is_some_and(|r|r.scope==*scope){self.remove_ready(&slot);}}}
    pub(super)fn remember(&mut self,ready:Ready,now:u64)->model::Result<()>{
        self.sweep(now);self.retired_ready.push(ready);let ready=self.retired_ready.last().ok_or(model::Error::Conflict)?;let slot=ready.scope.slot()?;
        if let Some(old)=self.ready.get(&slot){if old.scope!=ready.scope{return Err(model::Error::Conflict);}return Ok(());}
        if self.ready.len()>=64{return Err(model::Error::LimitReached);}let ready=self.retired_ready.pop().ok_or(model::Error::Conflict)?;self.ready.insert(slot,ready);Ok(())
    }

}
#[cfg(target_os="linux")]fn cache()->&'static Mutex<Cache>{static CACHE:OnceLock<Mutex<Cache>>=OnceLock::new();CACHE.get_or_init(||Mutex::new(Cache::default()))}
#[cfg(target_os="linux")]fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]fn with_cache_at<T>(lock:&Mutex<Cache>,operation:impl FnOnce(&mut Cache)->model::Result<T>)->model::Result<T>{
    let(result,retired)={let mut c=lock.lock().map_err(|_|model::Error::StorageUnavailable)?;let result=operation(&mut c);(result,c.take_retired())};drop(retired);result
}
#[cfg(target_os="linux")]fn with_cache<T>(operation:impl FnOnce(&mut Cache)->model::Result<T>)->model::Result<T>{with_cache_at(cache(),operation)}
#[cfg(target_os="linux")]pub(super)fn release(lease:&str){let _=with_cache(|c|{c.release(lease);Ok(())});}

#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_prepare_step(window:tauri::WebviewWindow,input:PrepareInput)->model::Result<Ticket>{
    check(&window)?;let label=window.label().to_string();tauri::async_runtime::spawn_blocking(move||{
        if !model::key(&input.request_id){return Err(model::Error::InvalidInput);}
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(model::Error::Conflict);}
        let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;let now=crate::assistant_memory::now();
        let existing=with_cache(|c|{c.sweep(now);
            if let Some((token,p))=c.pending.iter().find(|(_,p)|{let v=p.review.proposal();p.window==label&&p.request==input.request_id&&v.scope.goal_id==input.goal_id&&v.scope.pet_id==input.pet_id&&v.scope.lease_id==input.lease_id&&v.scope.plan_id==input.plan_id&&v.scope.step_id==input.step_id&&v.goal_revision==input.revision}){return Ok(Some(Ticket{token:token.clone(),proposal:p.review.proposal()}));}
            if c.pending.len()>=128||c.ready.len()>=64{return Err(model::Error::LimitReached);}
        Ok(None)})?;if let Some(ticket)=existing{return Ok(ticket);}
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;
        let worker=owner.readiness_worker(crate::assistant_memory::now())?;
        let target=super::physical_target::Target::capture(super::physical_target::step_operation(&goal,&input.plan_id,Some(&input.step_id))?,&goal.spec.owner.pet_id,&config,||worker.in_flight_current(crate::assistant_memory::now()))?;
        let review=owner.review_step(&goal,&input.plan_id,&input.step_id,target,&config,crate::assistant_memory::now())?;
        let ticket=Ticket{token:nonce()?,proposal:review.proposal()};with_cache(|c|c.insert_pending(ticket.token.clone(),label,input.request_id,review,crate::assistant_memory::now()))?;Ok(ticket)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_confirm_step(window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<ConfirmResult>{
    check(&window)?;let label=window.label().to_string();tauri::async_runtime::spawn_blocking(move||{
        let review=with_cache(|c|c.take(&label,&input,crate::assistant_memory::now()))?;
        if !input.approve{return Ok(ConfirmResult::Declined);}
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read(&input.goal_id,&input.pet_id)?;
        let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;
        let(approved,ready)=review.approve(&owner,&goal,&config,crate::assistant_memory::now())?;
        let scope=ready.scope.clone();
        with_cache(|c|c.remember(ready,crate::assistant_memory::now()))?;
        let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
        let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;
        match authority.issue_step(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,row.lease.as_ref().ok_or(model::Error::Conflict)?,&approved,crate::assistant_memory::now()){
            Ok(scope)=>Ok(ConfirmResult::Granted{scope}),Err(reason)=>{
                // A positively absent grant did not consume this native review.
                // Ambiguous/unreadable records retain the captured target for recovery.
                if matches!(authority.read_step(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&scope.plan_id,&scope.step_id),Ok(None)){
                    with_cache(|c|{c.forget(&scope);Ok(())})?;
                }
                Ok(ConfirmResult::Blocked{reason})
            },
        }
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_run_step(window:tauri::WebviewWindow,input:RunInput)->model::Result<View>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read(&input.goal_id,&input.pet_id)?;
        let row=goal.admissions.iter().find(|a|a.lease.as_ref().is_some_and(|l|l.id==input.lease_id)).ok_or(model::Error::Conflict)?;
        let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;
        let stored=authority.read_step(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&input.plan_id,&input.step_id).map_err(|_|model::Error::Conflict)?.ok_or(model::Error::NotFound)?;
        if stored.scope.grant_id!=input.grant_id{return Err(model::Error::Conflict);}
        if stored.phase==Phase::Finished{
            let result=stored.result.clone().ok_or(model::Error::InvalidRecord)?;
            authority.finish_step(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,row.lease.as_ref().ok_or(model::Error::Conflict)?,&stored.scope,stored.attempt.as_ref(),result,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
            return authority.read_step_assessment(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&input.plan_id,&input.step_id).map_err(|_|model::Error::StorageUnavailable)?.map(Into::into).ok_or(model::Error::NotFound);
        }
        let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let slot=stored.scope.slot()?;let ready=with_cache(|c|c.ready.get(&slot).cloned().ok_or(model::Error::Conflict))?;
        if ready.scope.grant_id!=input.grant_id{return Err(model::Error::Conflict);}
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;
        let result=owner.run_step(&goal,&ready,&config)?;
        if result.phase==Phase::Finished{with_cache(|c|{c.remove_ready(&slot);Ok(())})?;}
        authority.read_step_assessment(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&input.plan_id,&input.step_id).map_err(|_|model::Error::StorageUnavailable)?.map(Into::into).ok_or(model::Error::NotFound)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_step_result(window:tauri::WebviewWindow,input:ReadInput)->model::Result<Option<View>>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read_control(&input.goal_id,&input.pet_id)?;
        let row=goal.admissions.iter().find(|a|a.spec.decision_id==input.decision_id).ok_or(model::Error::NotFound)?;
        let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;
        authority.read_step_assessment_for_goal(&goal,&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&input.plan_id,&input.step_id).map(|v|v.map(Into::into)).map_err(Failure::control_error)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
#[tauri::command]
pub async fn pet_runtime_recover_step(window:tauri::WebviewWindow,input:RecoverInput)->model::Result<View>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let store=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?;
        store.recover_step_native(&super::admission_review::authority_root(),&input,crate::assistant_memory::now())
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
pub(super)fn recover_for_goal(authority:&Authority,goal:&model::Record,input:&RecoverInput,now:u64)->model::Result<View>{
    goal.validate()?;
    if goal.revision!=input.revision||goal.id!=input.goal_id||goal.spec.owner.pet_id!=input.pet_id{return Err(model::Error::Conflict);}
    let row=goal.admissions.iter().find(|a|a.spec.decision_id==input.decision_id).ok_or(model::Error::NotFound)?;
    let proof=row.proof.as_ref().ok_or(model::Error::Conflict)?;
    let stored=authority.read_step_assessment(&row.spec,proof,&input.plan_id,&input.step_id).map_err(Failure::control_error)?.ok_or(model::Error::NotFound)?;
    if stored.record.scope.grant_id!=input.grant_id{return Err(model::Error::Conflict);}
    authority.recover_step_for_goal(goal,&row.spec,proof,row.lease.as_ref().ok_or(model::Error::Conflict)?,&stored.record.scope,now).map(Into::into).map_err(Failure::control_error)
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_step(window:tauri::WebviewWindow,input:PrepareInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_step(window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_run_step(window:tauri::WebviewWindow,input:RunInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_step_result(window:tauri::WebviewWindow,input:ReadInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_recover_step(window:tauri::WebviewWindow,input:RecoverInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}

#[cfg(all(test,target_os="linux"))]#[path="system_step_cache_tests.rs"]mod native_cache_tests;
