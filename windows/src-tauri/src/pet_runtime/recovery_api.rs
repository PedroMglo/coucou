use serde::{Deserialize,Serialize};
use super::model::{self,Error};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecoveryPrepareInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct RecoveryConfirmInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub token:String,pub approve:bool}
#[cfg(target_os="linux")]#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct RecoveryTicket{pub token:String,pub proposal:super::recovery_controller::Proposal}
#[cfg(target_os="linux")]#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RecoveryPrepared{Review{ticket:RecoveryTicket},Closed{receipt:model::Receipt},Acknowledged{receipt:model::Receipt,#[serde(rename="executionAvailable")]execution_available:bool},Blocked{reason:Error,retryable:bool}}
#[cfg(not(target_os="linux"))]pub type RecoveryPrepared=serde_json::Value;
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum RecoveryConfirmed{Closed{receipt:model::Receipt},Resumed{receipt:model::Receipt,lease:super::leases::Lease,owned:bool,#[serde(rename="executionAvailable")]execution_available:bool},AcknowledgementPending{receipt:model::Receipt},Declined,Blocked{reason:Error,retryable:bool,retained:bool}}
#[cfg(target_os="linux")]
mod native{
    use super::*;use crate::pet_runtime::{recovery_controller::{Review,Approved,Acknowledged,Prepared},store::Store};
    use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
    enum State{Review(Box<Review>),Approved(Box<Approved>),Acknowledged(Acknowledged),Peer(Box<PeerState>),Ended}
    #[derive(Clone)]pub(super)struct Origin{pub(super)window:String,pub(super)frame:crate::settings::review::Frame}
    impl Origin{fn same(&self,other:&Self)->bool{self.window==other.window&&self.frame.same(&other.frame)&&self.frame.current()&&other.frame.current()}}
    #[derive(Clone,Copy,PartialEq,Eq)]enum Kind{Review,CleanupOnly,PeerResume,PeerCleanup}
    pub(super)struct Pending{origin:Origin,request:String,ticket:RecoveryTicket,kind:Kind,state:Arc<Mutex<State>>}
    #[derive(Default)]pub(super)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
    fn retryable(error:Error)->bool{matches!(error,Error::Busy|Error::StorageUnavailable)}
    impl Pending{
        fn confirm(&self,store:&Store,approve:bool,now:u64)->model::Result<RecoveryConfirmed>{
            let mut native=self.state.try_lock().map_err(|_|Error::Busy)?;
            if let State::Review(review)=&*native{
                if !approve{*native=State::Ended;return Ok(RecoveryConfirmed::Declined);}
                if self.kind!=Kind::Review||!self.origin.frame.current(){*native=State::Ended;return Err(Error::Conflict);}
                let decision=match review.decide(now){Ok(d)=>d,Err(e)=>{if !retryable(e){*native=State::Ended;}return Ok(RecoveryConfirmed::Blocked{reason:e,retryable:retryable(e),retained:!matches!(*native,State::Ended)});}};
                if !self.origin.frame.current(){*native=State::Ended;return Err(Error::Conflict);}
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
        fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.state.try_lock(){Ok(mut n)=>match &mut*n{State::Review(_)=>p.origin.frame.current()&&now<p.ticket.proposal.expires_at,State::Approved(_)=>true,State::Acknowledged(_)=>p.origin.frame.current()&&now<p.ticket.proposal.expires_at.saturating_add(300_000),State::Peer(peer)=>peer.keep(p,now),State::Ended=>false},Err(_)=>true});}
        fn find(&mut self,origin:&Origin,input:&RecoveryPrepareInput,now:u64)->model::Result<Option<RecoveryTicket>>{
            if !origin.frame.current(){return Err(Error::Conflict);}
            self.sweep(now);
            let mut rebind=None;
            for p in self.pending.values(){if p.origin.window==origin.window&&p.ticket.proposal.goal_id==input.goal_id&&p.ticket.proposal.pet_id==input.pet_id{
                if matches!(p.kind,Kind::PeerResume|Kind::PeerCleanup){continue;}
                let state=p.state.try_lock().map_err(|_|Error::Busy)?;
                let cleanup=match &*state{State::Approved(a)=>input.revision==a.goal.revision||input.revision==a.candidate.revision,State::Review(r)=>r.recorded_target().is_some_and(|rev|rev==input.revision||p.ticket.proposal.revision==input.revision),_=>false};
                if cleanup{
                    let mut ticket=p.ticket.clone();ticket.proposal.approval_scope="ackRecordedClosure";
                    if p.origin.same(origin){return Ok(Some(ticket));}
                    if p.origin.frame.current(){return Err(Error::Conflict);}
                    if !matches!(*state,State::Approved(_)){return Err(Error::Conflict);}
                    // Rebind only an already consumed choice. Share its exact
                    // state/closure; never call Review::decide from this alias.
                    ticket.token=crate::pet_runtime::store::nonce()?;rebind=Some((ticket,p.request.clone(),p.state.clone()));break;
                }
                if !p.origin.same(origin){return Err(Error::Conflict);}
                if matches!(&*state,State::Approved(a)if a.write_started){continue;}
                if p.ticket.proposal.revision==input.revision&&p.request==input.request_id{return Ok(Some(p.ticket.clone()));}
                if !matches!(*state,State::Acknowledged(_)){return Err(Error::Conflict);}
            }}
            if let Some((ticket,request,state))=rebind{
                if !origin.frame.current(){return Err(Error::Conflict);}
                let pending=Arc::new(Pending{origin:origin.clone(),request,ticket:ticket.clone(),kind:Kind::CleanupOnly,state:state.clone()});
                self.pending.retain(|_,p|!Arc::ptr_eq(&p.state,&state));self.pending.insert(ticket.token.clone(),pending);return Ok(Some(ticket));
            }Ok(None)
        }
        fn insert(&mut self,origin:&Origin,request:String,review:Review,now:u64)->model::Result<RecoveryTicket>{
            if !origin.frame.current(){return Err(Error::Conflict);}
            self.sweep(now);if self.pending.len()>=64{return Err(Error::LimitReached);}let ticket=RecoveryTicket{token:crate::pet_runtime::store::nonce()?,proposal:review.proposal()};self.pending.insert(ticket.token.clone(),Arc::new(Pending{origin:origin.clone(),request,ticket:ticket.clone(),kind:Kind::Review,state:Arc::new(Mutex::new(State::Review(Box::new(review))))}));Ok(ticket)
        }
        fn lookup(&self,origin:&Origin,input:&RecoveryConfirmInput)->model::Result<Arc<Pending>>{
            let p=self.pending.get(&input.token).ok_or(Error::NotFound)?;let t=&p.ticket.proposal;
            if !p.origin.same(origin)||t.goal_id!=input.goal_id||t.pet_id!=input.pet_id||t.revision!=input.revision{return Err(Error::Conflict);}Ok(p.clone())
        }
    }
    fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
    pub(super)fn prepare(origin:&Origin,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
        if !model::key(&input.request_id)||!model::token(&input.revision){return Err(Error::InvalidInput);}let now=crate::assistant_memory::now();
        if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.find(origin,&input,now)?{return Ok(RecoveryPrepared::Review{ticket});}
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;
        match store.prepare_recovery(&root.join("pet-runtime-consent-v1"),&input.goal_id,&input.pet_id,&input.revision,input.request_id.clone(),now){
            Ok(Prepared::Closed(receipt))=>Ok(RecoveryPrepared::Closed{receipt}),Ok(Prepared::Review(review))=>reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(origin,input.request_id,review,now).map(|ticket|RecoveryPrepared::Review{ticket}),
            Err(reason)=>Ok(RecoveryPrepared::Blocked{reason,retryable:retryable(reason)}),
        }
    }
    pub(super)fn confirm(app:&tauri::AppHandle,origin:&Origin,input:RecoveryConfirmInput)->model::Result<RecoveryConfirmed>{
        let pending=match reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(origin,&input){Ok(p)=>p,Err(reason)=>return Ok(RecoveryConfirmed::Blocked{reason,retryable:false,retained:false})};
        let root=crate::settings::config_dir();let store=Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?;
        if matches!(pending.kind,Kind::PeerResume|Kind::PeerCleanup){super::super::lease_api::start_runtime(app.clone());pending.confirm_peer(app,&store,input.approve,crate::assistant_memory::now())}else{pending.confirm(&store,input.approve,crate::assistant_memory::now())}
    }
    pub(super)fn context_changed(){
        use std::sync::atomic::{AtomicBool,Ordering};
        static PENDING:AtomicBool=AtomicBool::new(false);
        static DIRTY:AtomicBool=AtomicBool::new(false);
        DIRTY.store(true,Ordering::Release);
        if PENDING.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return;}
        let _cleanup=tauri::async_runtime::spawn_blocking(||{
            struct Reset(bool);impl Drop for Reset{fn drop(&mut self){if self.0{PENDING.store(false,Ordering::Release);}}}
            let mut reset=Reset(true);
            loop{
                DIRTY.store(false,Ordering::Release);
                if let Ok(mut cache)=reviews().lock(){cache.sweep(crate::assistant_memory::now());}
                crate::pet_runtime::admission_review::sweep_context();
                crate::pet_runtime::retirement_api::sweep_context();
                PENDING.store(false,Ordering::Release);reset.0=false;
                if !DIRTY.load(Ordering::Acquire)||PENDING.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){break;}reset.0=true;
            }
        });
    }
    include!("recovery_peer_resume.rs");
    #[cfg(test)]mod tests{include!("recovery_cache_tests.rs");}
}
pub(super)fn sweep_peer_resumptions(){#[cfg(target_os="linux")]native::sweep_peers();}
#[cfg(all(test,target_os="linux"))]pub(in crate::pet_runtime)fn fixture_peer_cache_scope(review:super::lease_controller::peer_resume_controller::Review,now:u64){native::fixture_peer_cache_scope(review,now)}
pub(crate)fn recovery_review_context_changed(){#[cfg(target_os="linux")]native::context_changed();}
#[tauri::command]
pub async fn pet_runtime_prepare_recovery(window:tauri::WebviewWindow,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{
        if !model::key(&input.request_id)||!model::token(&input.revision){return Err(Error::InvalidInput);}
        let frame=match crate::settings::review::capture_window(&window).await{Ok(frame)=>frame,Err(_)=>return Ok(RecoveryPrepared::Blocked{reason:Error::Conflict,retryable:false})};
        let origin=native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?
    }
    #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
#[tauri::command]
pub async fn pet_runtime_confirm_recovery(app:tauri::AppHandle,window:tauri::WebviewWindow,input:RecoveryConfirmInput)->model::Result<RecoveryConfirmed>{
    if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
    #[cfg(target_os="linux")]{
        let frame=match crate::settings::review::capture_window(&window).await{Ok(frame)=>frame,Err(_)=>return Ok(RecoveryConfirmed::Blocked{reason:Error::Conflict,retryable:false,retained:false})};
        let origin=native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::confirm(&app,&origin,input)).await.map_err(|_|Error::StorageUnavailable)?
    }
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
}
#[tauri::command]
pub async fn pet_runtime_prepare_peer_resume(window:tauri::WebviewWindow,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
 if !super::adapter::allowed(window.label()){return Err(Error::InvalidInput);}
 #[cfg(target_os="linux")] {
  let frame=crate::settings::review::capture_window(&window).await.map_err(|_|Error::Conflict)?;
  let origin=native::Origin{window:window.label().into(),frame};tauri::async_runtime::spawn_blocking(move||native::prepare_peer(&origin,input)).await.map_err(|_|Error::StorageUnavailable)?
 }
 #[cfg(not(target_os="linux"))]{let _=input;Err(Error::UnsupportedPlatform)}
}
