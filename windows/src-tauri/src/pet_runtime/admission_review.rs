//! Window-scoped exact review tokens. Confirmation only enqueues; no dispatch.
use super::{admission::*,model::{self,Error,Record}};
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,sync::{Mutex,OnceLock}};
use tauri::{AppHandle,Manager};

#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewTicket{pub token:String,pub proposal:ReviewProposal}
struct Pending{window:String,review:Review}
#[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Pending>}
impl Reviews{
    pub(super)fn insert(&mut self,window:&str,review:Review,token:String,now:u64)->model::Result<ReviewTicket>{
        if window.is_empty()||!model::token(&token){return Err(Error::InvalidInput);}
        self.pending.retain(|_,p|p.review.proposal().expires_at>now);
        let proposal=review.proposal();
        for (token,p)in &self.pending{
            let old=p.review.proposal();if p.window==window&&old.goal_id==proposal.goal_id{
                let semantic=|p:&ReviewProposal|model::fingerprint(&(&p.goal_id,&p.revision,&p.request_id,&p.owner,&p.context,&p.intent,&p.criteria,&p.resolved_model,&p.policy_revision,&p.descriptor_revision,&p.budget));
                if semantic(&old)?!=semantic(&proposal)?{return Err(Error::Conflict);}
                return Ok(ReviewTicket{token:token.clone(),proposal:old});
            }
        }
        if self.pending.len()>=64{return Err(Error::LimitReached);}if self.pending.contains_key(&token){return Err(Error::Conflict);}
        self.pending.insert(token.clone(),Pending{window:window.into(),review});Ok(ReviewTicket{token,proposal})
    }
    pub(super)fn take(&mut self,window:&str,input:&ConfirmInput,now:u64)->model::Result<Review>{
        let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;let v=p.review.proposal();
        if p.window!=window||v.owner.pet_id!=input.pet_id||v.goal_id!=input.goal_id||v.revision!=input.revision{return Err(Error::Conflict);}
        let pending=self.pending.remove(&input.token).ok_or(Error::Conflict)?;
        if now>=v.expires_at||now<v.prepared_at{return Err(Error::Conflict);}Ok(pending.review)
    }
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub budget:Budget}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConfirmInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum PrepareResult{Review{ticket:ReviewTicket},Blocked{reason:Denial}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ConfirmResult{Queued{receipt:model::Receipt},Declined,Blocked{reason:Denial},AuthorityBlocked{reason:super::authority::Failure}}
#[cfg(target_os="linux")]fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
#[cfg(target_os="linux")]pub(super)fn evidence(app:&AppHandle,goal:&Record,now:u64,owned:Option<&crate::pet_resources::Hold>)->model::Result<std::result::Result<Evidence,Denial>>{
    let projection=crate::pet_capabilities::native_projection(app,owned).map_err(|_|Error::StorageUnavailable)?;
    let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==goal.spec.owner.pet_id).ok_or(Error::Conflict)?;
    let config=projection.configuration.as_ref().ok_or(Error::StorageUnavailable)?;
    let policy=super::native_validity::policy_revision(config,&projection.settings,goal)?;
    // Passive capability metadata may be unknown/stale. Never infer account,
    // metering or model readiness by probing or by substituting another pet.
    Ok(Evidence::observe(goal,descriptor,policy,now,false,false))
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
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
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
        let review=reviews().lock().map_err(|_|Error::StorageUnavailable)?.take(&origin,&input,crate::assistant_memory::now())?;
        if !input.approve{return Ok(ConfirmResult::Declined);}
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        let now=crate::assistant_memory::now();let observation=match evidence(&app,&goal,now,None)?{Ok(e)=>e,Err(reason)=>return Ok(ConfirmResult::Blocked{reason})};
        let _context=context_guard(&app,&goal)?;
        let approved=match review.approve(&goal,observation,crate::assistant_memory::now()){Ok(a)=>a,Err(reason)=>return Ok(ConfirmResult::Blocked{reason})};
        let authority=match super::authority::Authority::ensure(&authority_root()){Ok(a)=>a,Err(reason)=>return Ok(ConfirmResult::AuthorityBlocked{reason})};
        let certified=match authority.certify(&approved,&goal,crate::assistant_memory::now()){Ok(c)=>c,Err(reason)=>return Ok(ConfirmResult::AuthorityBlocked{reason})};
        let receipt=store.enqueue_certified(&certified,crate::assistant_memory::now())?;Ok(ConfirmResult::Queued{receipt})
    }).await.map_err(|_|Error::StorageUnavailable)?}
}

#[cfg(target_os="linux")]pub(super)fn authority_root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-consent-v1")}
#[tauri::command]
pub async fn pet_runtime_admission_status(window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<super::authority::ProvenanceStatus>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(pet_id,goal_id);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let store=super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)?;let goal=store.read(&goal_id,&pet_id)?;
        let result=super::authority::Authority::existing(&authority_root()).and_then(|a|a.verify_queued(&goal,crate::assistant_memory::now()));
        Ok(super::authority::ProvenanceStatus::from_result(result))
    }).await.map_err(|_|Error::StorageUnavailable)?
}
