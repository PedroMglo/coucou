//! Exact native recipe approval and explicit background dispatch. A read only
//! returns ledger progress; it cannot recreate the retained execution objects.
use super::model;
use serde::{Serialize,Deserialize};
#[cfg(target_os="linux")]use super::{recipes,store::{Store,nonce},authority::{Authority,Failure},lease_controller::recipe_controller::{Review,Approved,Ready,Proposal}};
#[cfg(target_os="linux")]use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecipePrepareInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub plan_id:String,pub request_id:String,#[serde(default)]pub criteria:Vec<super::criteria::Input>}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecipeConfirmInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecipeRunInput{pub pet_id:String,pub goal_id:String,pub lease_id:String,pub revision:String,pub recipe_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecipeReadInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub plan_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecipeCompleteInput{pub pet_id:String,pub goal_id:String,pub decision_id:String,pub plan_id:String,pub recipe_id:String,pub revision:String}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RecipeTicket{pub token:String,pub proposal:Proposal}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RecipeConfirmation{Granted{scope:recipes::Scope},Declined,Ended,Blocked{reason:Failure}}
#[cfg(target_os="linux")]struct Pending{window:String,request:String,review:Review}
#[cfg(target_os="linux")]struct Retained{window:String,revision:String,token:String,approved:Approved,ready:Ready}
#[cfg(target_os="linux")]#[derive(Default)]struct Cache{pending:BTreeMap<String,Pending>,ready:BTreeMap<String,Arc<Retained>>,retired_pending:Vec<Pending>,retired_ready:Vec<Arc<Retained>>}
#[cfg(target_os="linux")]impl Cache{
    fn sweep(&mut self,now:u64){
        let expired=self.pending.iter().filter_map(|(key,p)|(now>=p.review.proposal().expires_at).then(||key.clone())).collect::<Vec<_>>();
        for key in expired{if let Some(p)=self.pending.remove(&key){self.retired_pending.push(p);}}
        let expired=self.ready.iter().filter_map(|(key,r)|(now>=r.ready.scope().deadline_at).then(||key.clone())).collect::<Vec<_>>();
        for key in expired{if let Some(r)=self.ready.remove(&key){self.retired_ready.push(r);}}
    }
    fn release(&mut self,lease:&str){
        let pending=self.pending.iter().filter_map(|(key,p)|(p.review.proposal().scope.lease_id==lease).then(||key.clone())).collect::<Vec<_>>();
        for key in pending{if let Some(p)=self.pending.remove(&key){self.retired_pending.push(p);}}
        let ready=self.ready.iter().filter_map(|(key,r)|(r.ready.scope().lease_id==lease).then(||key.clone())).collect::<Vec<_>>();
        for key in ready{if let Some(r)=self.ready.remove(&key){self.retired_ready.push(r);}}
    }
    fn take_retired(&mut self)->(Vec<Pending>,Vec<Arc<Retained>>){(std::mem::take(&mut self.retired_pending),std::mem::take(&mut self.retired_ready))}
    fn insert_pending(&mut self,token:String,window:String,request:String,review:Review,now:u64)->model::Result<()>{
        self.sweep(now);self.retired_pending.push(Pending{window,request,review});
        if self.pending.len()>=128||self.ready.len()>=64||self.pending.contains_key(&token){return Err(model::Error::LimitReached);}
        self.pending.insert(token,self.retired_pending.pop().ok_or(model::Error::Conflict)?);Ok(())
    }
    fn remember(&mut self,r:Arc<Retained>)->model::Result<()>{
        self.retired_ready.push(r.clone());if self.ready.len()>=64||self.ready.contains_key(&r.ready.scope().id){return Err(model::Error::LimitReached);}self.ready.insert(r.ready.scope().id.clone(),r);Ok(())
    }
    fn confirmed(&self,window:&str,input:&RecipeConfirmInput)->model::Result<Option<Arc<Retained>>>{
        let Some(r)=self.ready.values().find(|r|r.token==input.token)else{return Ok(None);};let s=r.ready.scope();
        if !input.approve||r.window!=window||r.revision!=input.revision||s.goal_id!=input.goal_id||s.pet_id!=input.pet_id||s.lease_id!=input.lease_id{return Err(model::Error::Conflict);}Ok(Some(r.clone()))
    }
    fn take(&mut self,window:&str,input:&RecipeConfirmInput)->model::Result<Review>{
        let p=self.pending.get(&input.token).ok_or(model::Error::Conflict)?;let v=p.review.proposal();
        if p.window!=window||v.goal_revision!=input.revision||v.scope.goal_id!=input.goal_id||v.scope.pet_id!=input.pet_id||v.scope.lease_id!=input.lease_id{return Err(model::Error::Conflict);}
        Ok(self.pending.remove(&input.token).ok_or(model::Error::Conflict)?.review)
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

#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_prepare_recipe(window:tauri::WebviewWindow,input:RecipePrepareInput)->model::Result<RecipeTicket>{
    check(&window)?;let label=window.label().to_string();tauri::async_runtime::spawn_blocking(move||{
        if !model::key(&input.request_id){return Err(model::Error::InvalidInput);}
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(model::Error::Conflict);}let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let binding=owner.recipe_input_binding(&goal,&input.plan_id,&input.criteria)?;
        let existing=with_cache(|c|{c.sweep(crate::assistant_memory::now());
            if let Some((token,p))=c.pending.iter().find(|(_,p)|{let v=p.review.proposal();p.window==label&&p.request==input.request_id&&v.goal_revision==input.revision&&v.scope.goal_id==input.goal_id&&v.scope.pet_id==input.pet_id&&v.scope.lease_id==input.lease_id&&v.scope.plan_id==input.plan_id}){if p.review.proposal().scope.verification!=binding{return Err(model::Error::Conflict);}return Ok(Some(RecipeTicket{token:token.clone(),proposal:p.review.proposal()}));}
            if c.pending.len()>=128||c.ready.len()>=64{return Err(model::Error::LimitReached);}Ok(None)})?;if let Some(ticket)=existing{return Ok(ticket);}
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;
        let worker=owner.readiness_worker(crate::assistant_memory::now())?;
        let target=super::physical_target::Target::capture(super::physical_target::step_operation(&goal,&input.plan_id,None)?,&goal.spec.owner.pet_id,&config,||worker.in_flight_current(crate::assistant_memory::now()))?;
        let review=owner.review_recipe_with_criteria(&goal,&input.plan_id,&input.request_id,target,&config,&input.criteria,crate::assistant_memory::now())?;
        let ticket=RecipeTicket{token:nonce()?,proposal:review.proposal()};with_cache(|c|c.insert_pending(ticket.token.clone(),label,input.request_id,review,crate::assistant_memory::now()))?;Ok(ticket)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_confirm_recipe(window:tauri::WebviewWindow,input:RecipeConfirmInput)->model::Result<RecipeConfirmation>{
    check(&window)?;let label=window.label().to_string();tauri::async_runtime::spawn_blocking(move||{
        let retained=with_cache(|c|{c.sweep(crate::assistant_memory::now());c.confirmed(&label,&input)})?;
        if retained.is_none()&&!input.approve{with_cache(|c|c.take(&label,&input))?;return Ok(RecipeConfirmation::Declined);}
        let store=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(model::Error::Conflict);}let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;owner.step_goal(&goal,&config,crate::assistant_memory::now())?;
        let retained=if let Some(r)=retained{r}else{
            let review=with_cache(|c|c.take(&label,&input))?;
            if !input.approve{return Ok(RecipeConfirmation::Declined);}
            let(approved,ready)=review.approve(&owner,&goal,&config,crate::assistant_memory::now())?;
            let r=Arc::new(Retained{window:label,revision:input.revision,token:input.token,approved,ready});
            with_cache(|c|c.remember(r.clone()))?;r
        };
        let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
        match authority.issue_recipe(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,row.lease.as_ref().ok_or(model::Error::Conflict)?,&retained.approved,crate::assistant_memory::now()){
            Ok(scope)=>{retained.ready.activate(&scope)?;Ok(RecipeConfirmation::Granted{scope})},Err(reason)=>Ok(RecipeConfirmation::Blocked{reason}),
        }
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_run_recipe(app:tauri::AppHandle,window:tauri::WebviewWindow,input:RecipeRunInput)->model::Result<recipes::RunResult>{
    use tauri::Emitter;check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let store=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(model::Error::Conflict);}let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let retained=with_cache(|c|c.ready.get(&input.recipe_id).cloned().ok_or(model::Error::Conflict))?;let s=retained.ready.scope();
        if s.pet_id!=input.pet_id||s.goal_id!=input.goal_id||s.lease_id!=input.lease_id||retained.revision!=input.revision{return Err(model::Error::Conflict);}
        let result=owner.run_recipe_for_store(&store,&goal,&retained.ready,||crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable),|p|{let _=app.emit("pet-runtime-progress",serde_json::json!({"petId":p.scope.pet_id,"goalId":p.scope.goal_id,"planId":p.scope.plan_id,"recipeId":p.scope.id,"phase":p.phase,"verifiedSteps":p.verified_steps,"totalSteps":p.total_steps}));})?;
        if result.progress.goal_complete{let _=super::lease_api::close_completed(&store,&input.goal_id,&input.pet_id,&input.lease_id);}
        Ok(result)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_recipe_progress(window:tauri::WebviewWindow,input:RecipeReadInput)->model::Result<Option<recipes::Progress>>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let goal=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?.read_control(&input.goal_id,&input.pet_id)?;
        let row=goal.admissions.iter().find(|a|a.spec.decision_id==input.decision_id).ok_or(model::Error::NotFound)?;
        let a=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;
        let mut progress=a.recipe_progress_for_goal(&goal,&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,&input.plan_id).map_err(super::authority::Failure::control_error)?;
        if goal.state==model::State::Completed{
            a.verify_completed_goal(&goal).map_err(|_|model::Error::StorageUnavailable)?;
            if let Some(p)=&mut progress{p.goal_complete=goal.completion.as_ref().is_some_and(|c|c.recipe_id==p.scope.id);}
        }Ok(progress)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]
pub(super)fn complete_for_goal(store:&Store,authority:&Authority,goal:&model::Record,input:&RecipeCompleteInput,current:impl Fn(&model::Record)->model::Result<()>)->model::Result<model::Receipt>{
    if input.pet_id!=goal.spec.owner.pet_id||input.goal_id!=goal.id{return Err(model::Error::Conflict);}
    let accepted=if let Some(c)=&goal.completion{input.revision==c.source_revision||input.revision==c.target_revision}else{input.revision==goal.revision};
    if !accepted{return Err(model::Error::Conflict);}let row=goal.admissions.iter().find(|a|a.spec.decision_id==input.decision_id).ok_or(model::Error::NotFound)?;let proof=row.proof.as_ref().ok_or(model::Error::Conflict)?;
    let scope=authority.read_recipe(&row.spec,proof,&input.plan_id).map_err(|e|if e==super::authority::Failure::Busy{model::Error::Busy}else{model::Error::StorageUnavailable})?.ok_or(model::Error::NotFound)?;
    if scope.id!=input.recipe_id{return Err(model::Error::Conflict);}
    let certificate=authority.certify_completion(&row.spec,proof,&scope,goal).map_err(|e|if e==super::authority::Failure::Busy{model::Error::Busy}else{model::Error::StorageUnavailable})?;
    store.complete_certified(&certificate,crate::assistant_memory::now(),current)
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_complete_recipe(window:tauri::WebviewWindow,input:RecipeCompleteInput)->model::Result<model::Receipt>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let store=Store::existing_native(&root())?.ok_or(model::Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;
        let owner=goal.admissions.last().and_then(|r|r.lease.as_ref()).and_then(|l|super::lease_api::step_owner(&goal,&l.id).ok());
        let finish=||complete_for_goal(&store,&authority,&goal,&input,|current|owner.as_ref().ok_or(model::Error::Conflict).and_then(|o|o.completion_authority(current,crate::assistant_memory::now())));
        let receipt=if goal.state==model::State::Completed{finish()?}else{owner.as_ref().ok_or(model::Error::Conflict)?.with_completion_lock(finish)?};
        if let Some(lease)=goal.admissions.last().and_then(|r|r.lease.as_ref()){if receipt.status!=model::CommitStatus::Uncertain{let _=super::lease_api::close_completed(&store,&input.goal_id,&input.pet_id,&lease.id);}}
        Ok(receipt)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_prepare_recipe(window:tauri::WebviewWindow,input:RecipePrepareInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_confirm_recipe(window:tauri::WebviewWindow,input:RecipeConfirmInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_run_recipe(app:tauri::AppHandle,window:tauri::WebviewWindow,input:RecipeRunInput)->model::Result<serde_json::Value>{check(&window)?;let _=(app,input);Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_recipe_progress(window:tauri::WebviewWindow,input:RecipeReadInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_complete_recipe(window:tauri::WebviewWindow,input:RecipeCompleteInput)->model::Result<model::Receipt>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}

/// Share a retained native recipe review between chat and voice. The request
/// starts no execution and does not take ownership of the foreground dialogue.
#[cfg(target_os="linux")]
struct SharedRequest(String);
#[cfg(target_os="linux")]
fn shared_requests()->&'static Mutex<std::collections::BTreeSet<String>>{static ACTIVE:OnceLock<Mutex<std::collections::BTreeSet<String>>>=OnceLock::new();ACTIVE.get_or_init(||Mutex::new(Default::default()))}
#[cfg(target_os="linux")]
impl SharedRequest{
    fn acquire(input:&RecipePrepareInput)->model::Result<Self>{let key=model::fingerprint(&(&input.goal_id,&input.lease_id,&input.plan_id))?;let mut active=shared_requests().lock().map_err(|_|model::Error::StorageUnavailable)?;if active.len()>=64||!active.insert(key.clone()){return Err(model::Error::Busy);}Ok(Self(key))}
}
#[cfg(target_os="linux")]
impl Drop for SharedRequest{fn drop(&mut self){if let Ok(mut active)=shared_requests().lock(){active.remove(&self.0);}}}
#[cfg(target_os="linux")]
fn shared_retained(input:&RecipePrepareInput,binding:&Option<super::criteria::Binding>)->model::Result<Option<Arc<Retained>>>{
    with_cache(|c|{c.sweep(crate::assistant_memory::now());
    let found=c.ready.values().find(|r|{let s=r.ready.scope();r.window=="goal-request"&&s.pet_id==input.pet_id&&s.goal_id==input.goal_id&&s.lease_id==input.lease_id&&s.plan_id==input.plan_id&&s.request_id==input.request_id});
    if let Some(r)=found{if r.revision!=input.revision||r.ready.scope().verification!=*binding{return Err(model::Error::Conflict);}}Ok(found.cloned())})
}
#[cfg(target_os="linux")]
fn register_shared(goal:&model::Record,retained:&Retained)->model::Result<RecipeConfirmation>{
    let authority=Authority::existing(&super::admission_review::authority_root()).map_err(|_|model::Error::StorageUnavailable)?;let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
    match authority.issue_recipe(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?,row.lease.as_ref().ok_or(model::Error::Conflict)?,&retained.approved,crate::assistant_memory::now()){
        Ok(scope)=>{retained.ready.activate(&scope)?;Ok(RecipeConfirmation::Granted{scope})},Err(reason)=>Ok(RecipeConfirmation::Blocked{reason}),
    }
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_request_recipe(app:tauri::AppHandle,window:tauri::WebviewWindow,input:RecipePrepareInput)->model::Result<RecipeConfirmation>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        if !model::key(&input.request_id){return Err(model::Error::InvalidInput);}let _request=SharedRequest::acquire(&input)?;
        let store=Arc::new(Store::existing_native(&root())?.ok_or(model::Error::NotFound)?);let goal=store.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(model::Error::Conflict);}let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;owner.step_goal(&goal,&config,crate::assistant_memory::now())?;
        let binding=owner.recipe_input_binding(&goal,&input.plan_id,&input.criteria)?;
        if let Some(r)=shared_retained(&input,&binding)?{return register_shared(&goal,&r);}
        with_cache(|c|{c.sweep(crate::assistant_memory::now());if c.ready.len()>=64{return Err(model::Error::LimitReached);}Ok(())})?;
        let worker=owner.readiness_worker(crate::assistant_memory::now())?;
        let target=super::physical_target::Target::capture(super::physical_target::step_operation(&goal,&input.plan_id,None)?,&goal.spec.owner.pet_id,&config,||worker.in_flight_current(crate::assistant_memory::now()))?;
        let review=owner.review_recipe_with_criteria(&goal,&input.plan_id,&input.request_id,target,&config,&input.criteria,crate::assistant_memory::now())?;
        let source=super::recipe_decisions::Source::capture(owner.clone(),&goal,&review,store.clone(),Arc::new(||crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)))?;
        match crate::cli_chat::ask_goal_recipe(&app,source){Some(false)=>return Ok(RecipeConfirmation::Declined),None=>return Ok(RecipeConfirmation::Ended),Some(true)=>{}}
        let current=store.read(&input.goal_id,&input.pet_id)?;if current.revision!=input.revision||model::fingerprint(&current)?!=model::fingerprint(&goal)?{return Err(model::Error::Conflict);}
        let config=crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)?;
        let(approved,ready)=review.approve(&owner,&current,&config,crate::assistant_memory::now())?;
        let retained=Arc::new(Retained{window:"goal-request".into(),revision:input.revision,token:nonce()?,approved,ready});
        with_cache(|c|c.remember(retained.clone()))?;
        register_shared(&current,&retained)
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_request_recipe(app:tauri::AppHandle,window:tauri::WebviewWindow,input:RecipePrepareInput)->model::Result<serde_json::Value>{check(&window)?;let _=(app,input);Err(model::Error::UnsupportedPlatform)}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct GoalAnswerInput{pub id:String,pub approve:bool,pub target:serde_json::Value}
#[tauri::command]
pub async fn pet_runtime_goal_pending(window:tauri::WebviewWindow)->Result<Vec<serde_json::Value>,String>{
    check(&window).map_err(|e|format!("{e:?}"))?;
    #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(crate::cli_chat::goal_pending).await.map_err(|_|"Os pedidos estão indisponíveis.".to_string())?}
    #[cfg(not(target_os="linux"))]{Err("UnsupportedPlatform".into())}
}
#[tauri::command]
pub async fn pet_runtime_goal_answer(app:tauri::AppHandle,window:tauri::WebviewWindow,input:GoalAnswerInput)->Result<(),String>{
    check(&window).map_err(|e|format!("{e:?}"))?;
    #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||{use tauri::Manager;crate::cli_chat::goal_answer(&app.state::<crate::chat::Chat>(),&input.id,input.approve,input.target)}).await.map_err(|_|"Os pedidos estão indisponíveis.".to_string())?}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err("UnsupportedPlatform".into())}
}

#[cfg(all(test,target_os="linux"))]#[path="system_recipe_cache_tests.rs"]mod native_cache_tests;
