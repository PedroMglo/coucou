//! Window-scoped exact review tokens. Confirmation only enqueues; no dispatch.
use super::{admission::*,model::{self,Error,Record}};
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
use tauri::{AppHandle,Manager};

#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewTicket{pub token:String,pub proposal:ReviewProposal}
#[cfg(target_os="linux")]#[path="admission_confirmation.rs"]mod confirmation;
enum Native{
    Review(Review),
    #[cfg(target_os="linux")]Approved{approved:Approved,authority:super::authority::Authority,acknowledged:bool},
    Ended,
}
pub(super)struct Pending{window:String,proposal:ReviewProposal,native:Mutex<Native>}
#[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
impl Reviews{
    pub(super)fn insert(&mut self,window:&str,review:Review,token:String,now:u64)->model::Result<ReviewTicket>{
        if window.is_empty()||!model::token(&token){return Err(Error::InvalidInput);}
        // A running transaction keeps its slot even when the public review TTL
        // expires. No filesystem/network IO holds this global cache mutex.
        self.pending.retain(|_,p|match p.native.try_lock(){
            Ok(n)=>match &*n{Native::Ended=>false,#[cfg(target_os="linux")]Native::Approved{acknowledged:false,..}=>true,_=>p.proposal.expires_at>now},Err(_)=>true,
        });
        let proposal=review.proposal();
        for (token,p)in &self.pending{
            let old=&p.proposal;if p.window==window&&old.goal_id==proposal.goal_id{
                let semantic=|p:&ReviewProposal|model::fingerprint(&(&p.restored_from,&p.memory_data,&p.continuation,&p.goal_id,&p.revision,&p.request_id,&p.owner,&p.context,&p.intent,&p.criteria,&p.resolved_model,&p.policy_revision,&p.descriptor_revision,&p.budget,&p.planning_policy));
                if old.engine_binding!=proposal.engine_binding||semantic(old)?!=semantic(&proposal)?{
                    #[cfg(target_os="linux")]if p.native.try_lock().is_ok_and(|n|matches!(*n,Native::Approved{acknowledged:true,..})){continue;}
                    return Err(Error::Conflict);
                }
                return Ok(ReviewTicket{token:token.clone(),proposal:old.clone()});
            }
        }
        if self.pending.len()>=64{return Err(Error::LimitReached);}if self.pending.contains_key(&token){return Err(Error::Conflict);}
        self.pending.insert(token.clone(),Arc::new(Pending{window:window.into(),proposal:proposal.clone(),native:Mutex::new(Native::Review(review))}));Ok(ReviewTicket{token,proposal})
    }
    pub(super)fn lookup(&self,window:&str,input:&ConfirmInput,now:u64)->model::Result<Arc<Pending>>{
        let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;let v=&p.proposal;
        if p.window!=window||v.owner.pet_id!=input.pet_id||v.goal_id!=input.goal_id||v.revision!=input.revision||now<v.prepared_at{return Err(Error::Conflict);}
        Ok(p.clone())
    }
    #[cfg(test)]pub(super)fn take(&mut self,window:&str,input:&ConfirmInput,now:u64)->model::Result<Review>{
        let p=self.lookup(window,input,now)?;let mut native=p.native.try_lock().map_err(|_|Error::Busy)?;
        if now>=p.proposal.expires_at{*native=Native::Ended;self.pending.remove(&input.token);return Err(Error::Conflict);}
        let old=std::mem::replace(&mut*native,Native::Ended);self.pending.remove(&input.token);
        if let Native::Review(review)=old{Ok(review)}else{Err(Error::Conflict)}
    }
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConfirmInput{#[serde(default)]pub memory_consent:Option<crate::pet_memory::context_model::GoalConsent>,pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum PrepareResult{Review{ticket:ReviewTicket},Blocked{reason:Denial}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ConfirmResult{Queued{receipt:model::Receipt},Declined,Blocked{reason:Denial,retryable:bool},AuthorityBlocked{reason:super::authority::Failure,retryable:bool}}
#[cfg(target_os="linux")]fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
#[cfg(target_os="linux")]fn captured_evidence(app:&AppHandle,goal:&Record,now:u64,owned:Option<&crate::pet_resources::Hold>)->model::Result<std::result::Result<(Evidence,crate::pet_config::Configuration),Denial>>{
    let projection=crate::pet_capabilities::native_projection(app,owned).map_err(|_|Error::StorageUnavailable)?;
    let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
    let config=projection.configuration.ok_or(Error::StorageUnavailable)?;
    let policy=super::native_validity::policy_revision(&config,&projection.settings,goal)?;
    Ok(Evidence::observe_native(goal,descriptor,policy,now,false,false,&projection.settings).map(|e|(e,config)))
}
#[cfg(target_os="linux")]pub(super)fn evidence(app:&AppHandle,goal:&Record,now:u64,owned:Option<&crate::pet_resources::Hold>)->model::Result<std::result::Result<Evidence,Denial>>{
    Ok(captured_evidence(app,goal,now,owned)?.map(|(e,_)|e))
}
#[cfg(target_os="linux")]fn context_guard(app:&AppHandle,goal:&Record)->model::Result<crate::chat::Chat>{
    let path=crate::settings::config_dir().join("pet-runtime-contexts-v1");
    let(_,chat)=super::context::open(&path,goal,app.state::<crate::chat::Chat>().inner())?;Ok(chat)
}
#[tauri::command]
pub async fn pet_runtime_prepare_admission(app:AppHandle,window:tauri::WebviewWindow,input:PrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
        let observation=match evidence(&app,&goal,now,None)?{Ok(e)=>e,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare(&goal,observation,input.budget,input.request_id,super::store::nonce()?,now){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(&origin,review,super::store::nonce()?,now)?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}
#[tauri::command]
pub async fn pet_runtime_confirm_admission(app:AppHandle,window:tauri::WebviewWindow,input:ConfirmInput)->model::Result<ConfirmResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
        let pending=reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input,crate::assistant_memory::now())?;
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;
        pending.confirm_with_data(&store,&authority_root(),input.approve,input.memory_consent,|goal,now|{
            Ok(match evidence(&app,goal,now,None)?{
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
pub struct PlanningPrepareInput {pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget,pub limits:super::planning::Limits}
/// Opt-in planning admission. Only bounds and record references are input; the
/// model, configuration, policy and operator catalog come from one projection.
#[tauri::command]
pub async fn pet_runtime_prepare_planning(app:AppHandle,window:tauri::WebviewWindow,input:PlanningPrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}let now=crate::assistant_memory::now();
        let(observation,config)=match captured_evidence(&app,&goal,now,None)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare_planning(&goal,observation,input.budget,input.limits,&config,input.request_id,super::store::nonce()?,now){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(&origin,review,super::store::nonce()?,now)?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ContinuationPrepareInput{#[serde(default)]pub review_uncertain_effects:bool,pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget,pub limits:Option<super::planning::Limits>}
#[cfg(target_os="linux")]
impl Reviews{
    fn retained_continuation(&self,window:&str,input:&ContinuationPrepareInput,goal:&Record,now:u64)->model::Result<Option<ReviewTicket>>{
        for(token,p)in &self.pending{
            let proposal=&p.proposal;if p.window!=window||proposal.goal_id!=input.goal_id||proposal.owner.pet_id!=input.pet_id||proposal.continuation.is_none(){continue;}
            let native=p.native.try_lock().map_err(|_|Error::Busy)?;
            let retained=match &*native{Native::Review(_)=>proposal.continuation.as_ref().is_none_or(|c|c.uncertain_effects.is_none()||input.review_uncertain_effects)&&proposal.revision==input.revision&&now>=proposal.prepared_at&&now<proposal.expires_at,Native::Approved{acknowledged:false,..}=>proposal.revision==input.revision||goal.events.iter().any(|e|e.request_id==proposal.request_id&&matches!(e.kind,model::EventKind::AdmissionQueued)),_=>false};
            if retained{return Ok(Some(ReviewTicket{token:token.clone(),proposal:proposal.clone()}));}
        }Ok(None)
    }
}
#[tauri::command]
pub async fn pet_runtime_prepare_continuation(app:AppHandle,window:tauri::WebviewWindow,input:ContinuationPrepareInput)->model::Result<PrepareResult>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}
        let now=crate::assistant_memory::now();
        if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.retained_continuation(&origin,&input,&goal,now)?{return Ok(PrepareResult::Review{ticket});}
        let(goal,captured)=match store.prepare_continuation_review(&authority_root(),&input.goal_id,&input.pet_id,&input.revision,now,input.review_uncertain_effects)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let(observation,config)=match captured_evidence(&app,&goal,now,None)?{Ok(pair)=>pair,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let review=match Review::prepare_continuation(&goal,observation,input.budget,input.limits,&config,captured,input.request_id,super::store::nonce()?,now){Ok(r)=>r,Err(reason)=>return Ok(PrepareResult::Blocked{reason})};
        let ticket=reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(&origin,review,super::store::nonce()?,now)?;
        Ok(PrepareResult::Review{ticket})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[cfg(all(test,target_os="linux"))]#[path="continuation_review_tests.rs"]mod continuation_tests;
