//! Window-scoped native retirement choices. No model, claim or effect dispatch.
use super::model::{self,Error};
use serde::{Deserialize,Serialize};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RetirementPrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub decision_id:String,pub request_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RetirementConfirmInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub decision_id:String,pub token:String,pub approve:bool}
#[cfg(target_os="linux")]#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct RetirementTicket{pub token:String,pub proposal:super::retirement_controller::Proposal}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RetirementPrepared{Review{ticket:RetirementTicket},Retired{receipt:super::retirement::Receipt},Blocked{reason:Error,retryable:bool}}
#[cfg(not(target_os="linux"))]pub type RetirementPrepared=serde_json::Value;
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RetirementConfirmed{Retired{receipt:super::retirement::Receipt},Declined,Blocked{reason:Error,retryable:bool,retained:bool}}
#[cfg(target_os="linux")]
mod native{
    use super::*;use crate::pet_runtime::{retirement_controller::{Review,Approved,Acknowledged,Prepared},store::Store,model::Record};
    use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
    enum State{Review(Box<Review>),Approved(Box<Approved>),Acknowledged(Acknowledged),Ended}
    pub(super)struct Pending{window:String,ticket:RetirementTicket,state:Mutex<State>}
    #[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
    fn retryable(error:Error)->bool{matches!(error,Error::Busy|Error::StorageUnavailable)}
    impl Pending{
        fn confirm(&self,store:&Store,approve:bool,now:u64)->model::Result<RetirementConfirmed>{
            let mut state=self.state.try_lock().map_err(|_|Error::Busy)?;
            if let State::Review(review)=&*state{
                if !approve{*state=State::Ended;return Ok(RetirementConfirmed::Declined);}
                let decision=match review.decide(now){Ok(choice)=>choice,Err(error)=>{if !retryable(error){*state=State::Ended;}return Ok(RetirementConfirmed::Blocked{reason:error,retryable:retryable(error),retained:!matches!(*state,State::Ended)});}};
                let State::Review(review)=std::mem::replace(&mut*state,State::Ended)else{return Err(Error::Conflict);};*state=State::Approved(Box::new(review.approve(decision)));
            }
            if !approve{
                if let State::Approved(approved)=&*state{if store.withdraw_retirement(approved)?{*state=State::Ended;return Ok(RetirementConfirmed::Declined);}}
                return Err(Error::Conflict);
            }
            match &mut*state{
                State::Approved(approved)=>match approved.confirm(store,now){
                    Ok(receipt)=>{if receipt.status!=model::CommitStatus::Uncertain{let State::Approved(approved)=std::mem::replace(&mut*state,State::Ended)else{return Err(Error::Conflict);};*state=State::Acknowledged(approved.acknowledged());}Ok(RetirementConfirmed::Retired{receipt})},
                    Err(error)=>{if !approved.write_started&&!retryable(error){*state=State::Ended;}Ok(RetirementConfirmed::Blocked{reason:error,retryable:retryable(error),retained:!matches!(*state,State::Ended)})},
                },State::Acknowledged(ack)=>ack.receipt(store).map(|receipt|RetirementConfirmed::Retired{receipt}),_=>Err(Error::Conflict),
            }
        }
    }
    impl Reviews{
        fn sweep(&mut self,now:u64){self.pending.retain(|_,pending|match pending.state.try_lock(){Ok(state)=>match &*state{State::Review(_)=>now<pending.ticket.proposal.expires_at,State::Approved(_)=>true,State::Acknowledged(_)=>now<pending.ticket.proposal.expires_at.saturating_add(300_000),State::Ended=>false},Err(_)=>true});}
        fn find(&mut self,window:&str,input:&RetirementPrepareInput,goal:&Record,now:u64)->model::Result<Option<RetirementTicket>>{
            self.sweep(now);
            for pending in self.pending.values(){let p=&pending.ticket.proposal;
                if pending.window!=window||p.goal_id!=input.goal_id||p.pet_id!=input.pet_id||p.decision_id!=input.decision_id{continue;}
                let mut state=pending.state.try_lock().map_err(|_|Error::Busy)?;
                if matches!(*state,State::Approved(_)){let mut ticket=pending.ticket.clone();ticket.proposal.approval_scope="confirmOriginalRetirement";return Ok(Some(ticket));}
                if matches!(*state,State::Review(_)){
                    if p.revision!=goal.revision{*state=State::Ended;continue;}
                    return Ok(Some(pending.ticket.clone()));
                }
            }Ok(None)
        }
        fn insert(&mut self,window:&str,review:Review,now:u64)->model::Result<RetirementTicket>{
            self.sweep(now);if self.pending.len()>=64{return Err(Error::LimitReached);}let ticket=RetirementTicket{token:crate::pet_runtime::store::nonce()?,proposal:review.proposal()};
            for existing in self.pending.values(){let p=&existing.ticket.proposal;
                if existing.window==window&&p.goal_id==ticket.proposal.goal_id&&p.pet_id==ticket.proposal.pet_id&&p.decision_id==ticket.proposal.decision_id{
                    let state=existing.state.try_lock().map_err(|_|Error::Busy)?;
                    if matches!(*state,State::Review(_)|State::Approved(_)){return Ok(existing.ticket.clone());}
                }
            }
            self.pending.insert(ticket.token.clone(),Arc::new(Pending{window:window.into(),ticket:ticket.clone(),state:Mutex::new(State::Review(Box::new(review)))}));Ok(ticket)
        }
        fn lookup(&self,window:&str,input:&RetirementConfirmInput)->model::Result<Arc<Pending>>{
            let pending=self.pending.get(&input.token).ok_or(Error::NotFound)?;let p=&pending.ticket.proposal;
            if pending.window!=window||p.goal_id!=input.goal_id||p.pet_id!=input.pet_id||p.revision!=input.revision||p.decision_id!=input.decision_id{return Err(Error::Conflict);}Ok(pending.clone())
        }
    }
    fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
    pub(super)fn prepare(window:&str,input:RetirementPrepareInput)->model::Result<RetirementPrepared>{
        if !model::key(&input.request_id)||!model::token(&input.revision)||!model::token(&input.decision_id){return Err(Error::InvalidInput);}let now=crate::assistant_memory::now();
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        if goal.revision!=input.revision{return Err(Error::Conflict);}
        if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.find(window,&input,&goal,now)?{return Ok(RetirementPrepared::Review{ticket});}
        match store.prepare_retirement(&root.join("pet-runtime-consent-v1"),&input.goal_id,&input.pet_id,&input.revision,&input.decision_id,input.request_id,now){
            Ok(Prepared::Review(review))=>reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(window,review,now).map(|ticket|RetirementPrepared::Review{ticket}),
            Ok(Prepared::Retired(receipt))=>Ok(RetirementPrepared::Retired{receipt}),Err(reason)=>Ok(RetirementPrepared::Blocked{reason,retryable:retryable(reason)}),
        }
    }
    pub(super)fn confirm(window:&str,input:RetirementConfirmInput)->model::Result<RetirementConfirmed>{
        let pending=match reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(window,&input){Ok(pending)=>pending,Err(reason)=>return Ok(RetirementConfirmed::Blocked{reason,retryable:false,retained:false})};
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;pending.confirm(&store,input.approve,crate::assistant_memory::now())
    }
    #[cfg(test)]mod tests{include!("retirement_cache_tests.rs");}
}
#[tauri::command]
pub async fn pet_runtime_prepare_retirement(window:tauri::WebviewWindow,input:RetirementPrepareInput)->model::Result<RetirementPrepared>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||native::prepare(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
#[tauri::command]
pub async fn pet_runtime_confirm_retirement(window:tauri::WebviewWindow,input:RetirementConfirmInput)->model::Result<RetirementConfirmed>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||native::confirm(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
