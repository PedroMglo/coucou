//! A durable claim is preparation, never permission to call a model/tool.
//! Actual workers must keep WorkerGuard until IO is fully drained/reaped.
#[path="writer.rs"]mod writer;
#[path="plan_controller.rs"]mod plan_controller;
pub(in crate::pet_runtime)use plan_controller::PlanGate;
#[cfg(target_os="linux")]#[path="step_controller.rs"]pub(super)mod step_controller;
#[cfg(target_os="linux")]#[path="recipe_controller.rs"]pub(super)mod recipe_controller;
use super::{admission::{AdmissionSpec,Evidence},authority::{Authority,ConsentProof},context,leases::{Lease,DrainOutcome},model::{self,Record,CommitStatus},store::{Store,nonce}};
use crate::{chat::Chat,pet_resources::Hold};
use std::{collections::BTreeMap,path::{Path,PathBuf},sync::{Arc,Mutex,OnceLock,Weak,atomic::{AtomicBool,AtomicUsize,Ordering}},time::{Instant,Duration}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(super)enum Validity{Current,Changed,Unknown}
#[derive(Clone,Copy)]pub(super)enum Validation<'a>{FreshEffect,InFlightPolicy,Commit(&'a Record)}
impl PartialEq for Validation<'_>{fn eq(&self,other:&Self)->bool{matches!((self,other),(Self::FreshEffect,Self::FreshEffect)|(Self::InFlightPolicy,Self::InFlightPolicy))||matches!((self,other),(Self::Commit(a),Self::Commit(b))if std::ptr::eq(*a,*b))}}
impl Eq for Validation<'_>{}
impl std::fmt::Debug for Validation<'_>{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(match self{Self::FreshEffect=>"FreshEffect",Self::InFlightPolicy=>"InFlightPolicy",Self::Commit(_)=>"Commit"})}}
type Fence=Arc<dyn for<'a>Fn(&Lease,&Hold,Validation<'a>)->Validity+Send+Sync>;
struct Live{
    lease:Lease,spec:AdmissionSpec,proof:ConsentProof,authority_path:PathBuf,chat:Chat,hold:Arc<Hold>,
    deadline:Instant,closed:AtomicBool,activated:AtomicBool,workers:AtomicUsize,gate:Mutex<()>,fence:Fence,
}
impl Live{
    fn current(&self,now:u64)->bool{self.valid_at(now,Validation::FreshEffect)}
    fn valid_at(&self,now:u64,mode:Validation)->bool{
        if self.closed.load(Ordering::Acquire)||Instant::now()>=self.deadline||now<self.lease.claimed_at||now>=self.lease.deadline_at||!self.hold.current()||!self.chat.goal_context_current(&self.lease.goal_id){self.hold.cancel();return false;}
        if !self.activated.load(Ordering::Acquire){return false;}
        match Authority::existing(&self.authority_path).and_then(|a|a.lease_current(&self.spec,&self.proof,&self.lease,now)){
            Ok(())=>{},Err(super::authority::Failure::Busy|super::authority::Failure::Unavailable)=>return false,
            Err(_)=>{self.hold.cancel();return false;},
        }
        match (self.fence)(&self.lease,&self.hold,mode){Validity::Current=>true,Validity::Unknown=>false,Validity::Changed=>{self.hold.cancel();false}}
    }
}
fn live_registry()->&'static Mutex<BTreeMap<String,Weak<Live>>>{static LIVE:OnceLock<Mutex<BTreeMap<String,Weak<Live>>>>=OnceLock::new();LIVE.get_or_init(||Mutex::new(BTreeMap::new()))}
pub(super)fn signal_stop(lease:&Lease){
    let weak=live_registry().lock().ok().and_then(|r|r.get(&lease.id).cloned());
    if let Some(live)=weak.and_then(|w|w.upgrade()){if live.lease==*lease{live.closed.store(true,Ordering::Release);live.hold.cancel();}}
}
impl Drop for Live{fn drop(&mut self){self.hold.cancel();if let Ok(mut map)=live_registry().lock(){if map.get(&self.lease.id).is_some_and(|w|std::ptr::eq(w.as_ptr(),self)){map.remove(&self.lease.id);}}}}
pub(super)struct Controller{id:String}
pub(super)struct Pending{live:Arc<Live>,drain_request:String}
pub(super)struct Owned{live:Arc<Live>,drain_request:String}
pub(super)struct WorkerGuard{live:Arc<Live>}
pub(super)struct DrainProof{live:Arc<Live>,request:String,outcome:DrainOutcome,unstarted:bool}
impl Controller{
    pub(super)fn new()->model::Result<Self>{Ok(Self{id:nonce()?})}
    pub(super)fn prepare(&self,goal:&Record,authority_path:&Path,context_path:&Path,main:&Chat,evidence:&Evidence,request:String,now:u64,fence:Fence)->model::Result<Pending>{
        if goal.state!=model::State::Queued||!model::key(&request)||goal.events.iter().any(|e|e.request_id==request){return Err(model::Error::Conflict);}
        let(capability_revision,policy_revision)=evidence.for_claim(goal,now).map_err(|_|model::Error::Conflict)?;
        Authority::existing(authority_path).and_then(|a|a.verify_queued(goal,now)).map_err(|_|model::Error::Conflict)?;
        let row=goal.admissions.last().ok_or(model::Error::InvalidRecord)?;
        let(_,chat)=context::open(context_path,goal,main)?;
        if !chat.goal_context_current(&goal.id){return Err(model::Error::Conflict);}
        let remaining=row.spec.budget.deadline_at.checked_sub(now).filter(|n|*n>0&&*n<=86_400_000).ok_or(model::Error::Conflict)?;
        let deadline=Instant::now().checked_add(Duration::from_millis(remaining)).ok_or(model::Error::Conflict)?;
        let hold=main.runtime_resources().acquire(&goal.spec.owner.provider,Arc::new(AtomicBool::new(false)),Some(deadline)).map_err(|_|model::Error::Busy)?;
        let lease=Lease{schema_version:1,id:nonce()?,controller_id:self.id.clone(),request_id:request,decision_id:row.spec.decision_id.clone(),goal_id:goal.id.clone(),provider:goal.spec.owner.provider.clone(),conversation_id:row.spec.context.conversation_id.clone(),model:row.spec.resolved_model.clone(),policy_revision,capability_revision,source_revision:goal.revision.clone(),claimed_at:now,deadline_at:row.spec.budget.deadline_at};
        let live=Arc::new(Live{lease,spec:row.spec.clone(),proof:row.proof.clone().ok_or(model::Error::Conflict)?,authority_path:authority_path.into(),chat,hold,deadline,closed:AtomicBool::new(false),activated:AtomicBool::new(false),workers:AtomicUsize::new(0),gate:Mutex::new(()),fence});
        {let mut map=live_registry().lock().map_err(|_|model::Error::StorageUnavailable)?;map.retain(|_,w|w.strong_count()>0);if map.len()>=64{return Err(model::Error::LimitReached);}map.insert(live.lease.id.clone(),Arc::downgrade(&live));}
        Ok(Pending{live,drain_request:nonce()?})
    }
}
impl Pending{
    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn hold(&self)->&Hold{&self.live.hold}
    pub(super)fn workers(&self)->usize{self.live.workers.load(Ordering::Acquire)}
    pub(super)fn abort(&self)->model::Result<DrainProof>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if self.live.activated.load(Ordering::Acquire)||self.live.workers.load(Ordering::Acquire)!=0{return Err(model::Error::Conflict);}
        self.live.closed.store(true,Ordering::Release);self.live.hold.cancel();Ok(DrainProof{live:self.live.clone(),request:self.drain_request.clone(),outcome:DrainOutcome::Confirmed,unstarted:true})
    }
    pub(super)fn claim(&self,store:&Store,authority:&Authority,evidence:&Evidence,now:u64)->model::Result<(model::Receipt,Option<Owned>)>{
        if self.live.closed.load(Ordering::Acquire)||!self.live.hold.current()||!self.live.chat.goal_context_current(&self.live.lease.goal_id){return Err(model::Error::Conflict);}
        let receipt=store.claim_owned(authority,&self.live.lease,evidence,now)?;
        if receipt.status==CommitStatus::Uncertain{return Ok((receipt,None));}
        if Instant::now()>=self.live.deadline||!self.live.hold.current(){return Err(model::Error::Conflict);}
        if self.live.activated.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Ok((receipt,None));}
        Ok((receipt,Some(Owned{live:self.live.clone(),drain_request:self.drain_request.clone()})))
    }
}
impl Owned{
    pub(super)fn with_completion_lock<T>(&self,run:impl FnOnce()->model::Result<T>)->model::Result<T>{let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;run()}
    pub(super)fn completion_current(&self,goal:&Record,now:u64)->bool{
        self.workers()==0&&goal.state==model::State::Leased&&goal.id==self.live.spec.goal_id&&goal.spec.owner==self.live.spec.owner
            &&model::fingerprint(&goal.spec).ok().as_ref()==Some(&self.live.spec.intent_hash)
            &&goal.admissions.last().and_then(|a|a.lease.as_ref())==Some(&self.live.lease)
            &&self.live.valid_at(now,Validation::Commit(goal))
    }
    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn current(&self,now:u64)->bool{self.live.current(now)}
    pub(super)fn policy_current(&self,now:u64)->bool{self.live.valid_at(now,Validation::InFlightPolicy)}
    pub(super)fn workers(&self)->usize{self.live.workers.load(Ordering::Acquire)}
    pub(super)fn worker(&self,now:u64)->model::Result<WorkerGuard>{self.register_worker(now,Validation::FreshEffect)}
    /// Read-only native readiness IO is allowed under an existing owned grant;
    /// it cannot arm model/tool IO without a new fresh-effect check.
    pub(super)fn readiness_worker(&self,now:u64)->model::Result<WorkerGuard>{self.register_worker(now,Validation::InFlightPolicy)}
    fn register_worker(&self,now:u64,mode:Validation)->model::Result<WorkerGuard>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if !self.live.valid_at(now,mode){return Err(model::Error::Conflict);}if self.live.workers.fetch_add(1,Ordering::AcqRel)>=8{self.live.workers.fetch_sub(1,Ordering::AcqRel);return Err(model::Error::LimitReached);}
        Ok(WorkerGuard{live:self.live.clone()})
    }
    pub(super)fn drain(&self,outcome:DrainOutcome)->model::Result<DrainProof>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        self.live.closed.store(true,Ordering::Release);self.live.hold.cancel();
        if self.workers()!=0{return Err(model::Error::Busy);}Ok(DrainProof{live:self.live.clone(),request:self.drain_request.clone(),outcome,unstarted:false})
    }
}
impl WorkerGuard{
    pub(super)fn current(&self,now:u64)->bool{self.live.current(now)}
    pub(super)fn in_flight_current(&self,now:u64)->bool{self.live.valid_at(now,Validation::InFlightPolicy)}
}
impl Drop for WorkerGuard{fn drop(&mut self){self.live.workers.fetch_sub(1,Ordering::AcqRel);}}
impl DrainProof{
    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn request_id(&self)->&str{&self.request}
    pub(super)fn confirmed(&self)->bool{self.outcome==DrainOutcome::Confirmed}
    pub(super)fn unstarted(&self)->bool{self.unstarted}
}
