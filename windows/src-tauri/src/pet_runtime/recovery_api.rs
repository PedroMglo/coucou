use serde::{Deserialize,Serialize};
use super::model::{self,Error};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecoveryPrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecoveryConfirmInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[cfg(target_os="linux")]#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct RecoveryTicket{pub token:String,pub proposal:super::recovery_controller::Proposal}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RecoveryPrepared{Review{ticket:RecoveryTicket},Closed{receipt:model::Receipt},Blocked{reason:Error,retryable:bool}}
#[cfg(not(target_os="linux"))]pub type RecoveryPrepared=serde_json::Value;
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RecoveryConfirmed{Closed{receipt:model::Receipt},Declined,Blocked{reason:Error,retryable:bool,retained:bool}}
#[cfg(target_os="linux")]
mod native{
    use super::*;use crate::pet_runtime::{recovery_controller::{Review,Approved,Acknowledged,Prepared},store::Store};
    use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
    enum State{Review(Box<Review>),Approved(Box<Approved>),Acknowledged(Acknowledged),Ended}
    pub(super)struct Pending{window:String,request:String,ticket:RecoveryTicket,state:Mutex<State>}
    #[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
    fn retryable(error:Error)->bool{matches!(error,Error::Busy|Error::StorageUnavailable)}
    impl Pending{
        fn confirm(&self,store:&Store,approve:bool,now:u64)->model::Result<RecoveryConfirmed>{
            let mut native=self.state.try_lock().map_err(|_|Error::Busy)?;
            if let State::Review(review)=&*native{
                if !approve{*native=State::Ended;return Ok(RecoveryConfirmed::Declined);}
                let decision=match review.decide(now){Ok(d)=>d,Err(e)=>{if !retryable(e){*native=State::Ended;}return Ok(RecoveryConfirmed::Blocked{reason:e,retryable:retryable(e),retained:!matches!(*native,State::Ended)});}};
                let State::Review(review)=std::mem::replace(&mut*native,State::Ended)else{return Err(Error::Conflict);};*native=State::Approved(Box::new(review.approve(decision)));
            }
            if !approve{
                if let State::Approved(approved)=&*native{if store.withdraw_recovery(approved)?{*native=State::Ended;return Ok(RecoveryConfirmed::Declined);}}
                return Err(Error::Conflict);
            }
            match &mut*native{
                State::Approved(approved)=>match approved.confirm(store,now){
                    Ok(receipt)=>{if receipt.status!=model::CommitStatus::Uncertain{let State::Approved(approved)=std::mem::replace(&mut*native,State::Ended)else{return Err(Error::Conflict);};*native=State::Acknowledged(approved.acknowledged());}Ok(RecoveryConfirmed::Closed{receipt})},
                    Err(e)=>{if !approved.write_started&&!retryable(e){*native=State::Ended;}Ok(RecoveryConfirmed::Blocked{reason:e,retryable:retryable(e),retained:!matches!(*native,State::Ended)})},
                },State::Acknowledged(ack)=>ack.receipt(store).map(|receipt|RecoveryConfirmed::Closed{receipt}),_=>Err(Error::Conflict),
            }
        }
    }
    impl Reviews{
        fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.state.try_lock(){Ok(n)=>match &*n{State::Review(_)=>now<p.ticket.proposal.expires_at,State::Approved(_)=>true,State::Acknowledged(_)=>now<p.ticket.proposal.expires_at.saturating_add(300_000),State::Ended=>false},Err(_)=>true});}
        fn find(&mut self,window:&str,input:&RecoveryPrepareInput,now:u64)->model::Result<Option<RecoveryTicket>>{
            self.sweep(now);
            for p in self.pending.values(){if p.window==window&&p.ticket.proposal.goal_id==input.goal_id&&p.ticket.proposal.pet_id==input.pet_id{
                let state=p.state.try_lock().map_err(|_|Error::Busy)?;
                let cleanup=match &*state{State::Approved(a)=>input.revision==a.goal.revision||input.revision==a.candidate.revision,State::Review(r)=>r.recorded_target().is_some_and(|rev|rev==input.revision||p.ticket.proposal.revision==input.revision),_=>false};
                if cleanup{let mut ticket=p.ticket.clone();ticket.proposal.approval_scope="ackRecordedClosure";return Ok(Some(ticket));}
                if matches!(&*state,State::Approved(a)if a.write_started){continue;}
                if p.ticket.proposal.revision==input.revision&&p.request==input.request_id{return Ok(Some(p.ticket.clone()));}
                if !matches!(*state,State::Acknowledged(_)){return Err(Error::Conflict);}
            }}Ok(None)
        }
        fn insert(&mut self,window:&str,request:String,review:Review,now:u64)->model::Result<RecoveryTicket>{
            self.sweep(now);if self.pending.len()>=64{return Err(Error::LimitReached);}let ticket=RecoveryTicket{token:crate::pet_runtime::store::nonce()?,proposal:review.proposal()};self.pending.insert(ticket.token.clone(),Arc::new(Pending{window:window.into(),request,ticket:ticket.clone(),state:Mutex::new(State::Review(Box::new(review)))}));Ok(ticket)
        }
        fn lookup(&self,window:&str,input:&RecoveryConfirmInput)->model::Result<Arc<Pending>>{
            let p=self.pending.get(&input.token).ok_or(Error::NotFound)?;let t=&p.ticket.proposal;
            if p.window!=window||t.goal_id!=input.goal_id||t.pet_id!=input.pet_id||t.revision!=input.revision{return Err(Error::Conflict);}Ok(p.clone())
        }
    }
    fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
    pub(super)fn prepare(window:&str,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
        if !model::key(&input.request_id)||!model::token(&input.revision){return Err(Error::InvalidInput);}let now=crate::assistant_memory::now();
        if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.find(window,&input,now)?{return Ok(RecoveryPrepared::Review{ticket});}
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;
        match store.prepare_recovery(&root.join("pet-runtime-consent-v1"),&input.goal_id,&input.pet_id,&input.revision,input.request_id.clone(),now){
            Ok(Prepared::Closed(receipt))=>Ok(RecoveryPrepared::Closed{receipt}),Ok(Prepared::Review(review))=>reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(window,input.request_id,review,now).map(|ticket|RecoveryPrepared::Review{ticket}),
            Err(reason)=>Ok(RecoveryPrepared::Blocked{reason,retryable:retryable(reason)}),
        }
    }
    pub(super)fn confirm(window:&str,input:RecoveryConfirmInput)->model::Result<RecoveryConfirmed>{
        let pending=match reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(window,&input){Ok(p)=>p,Err(reason)=>return Ok(RecoveryConfirmed::Blocked{reason,retryable:false,retained:false})};
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;pending.confirm(&store,input.approve,crate::assistant_memory::now())
    }
    #[cfg(test)]mod tests{include!("recovery_cache_tests.rs");}
}
#[tauri::command]
pub async fn pet_runtime_prepare_recovery(window:tauri::WebviewWindow,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||native::prepare(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
#[tauri::command]
pub async fn pet_runtime_confirm_recovery(window:tauri::WebviewWindow,input:RecoveryConfirmInput)->model::Result<RecoveryConfirmed>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{let origin=window.label().to_owned();tauri::async_runtime::spawn_blocking(move||native::confirm(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?}
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
