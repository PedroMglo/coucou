//! A durable claim is preparation, never permission to call a model/tool.
//! Actual workers must keep WorkerGuard until IO is fully drained/reaped.
#[path="message_controller.rs"]pub(super)mod message_controller;
#[path="writer.rs"]mod writer;
pub(in crate::pet_runtime)use writer::{PreparedPeerRound,PeerRoundPublished};
#[cfg(target_os="linux")]#[path="completion_commit.rs"]mod completion_commit;
#[cfg(test)]pub(in crate::pet_runtime)use writer::tests::{fixture_with_hook_idle as memory_test_http_fixture_idle,fixture_with_hook_disconnect as memory_test_http_fixture_allow_disconnect,fixture_with_hook as memory_test_http_fixture,stream as memory_test_stream,local_metadata as memory_test_metadata,run as memory_test_run};
#[cfg(target_os="linux")]#[path="dispatcher.rs"]pub(super)mod dispatcher;
#[cfg(target_os="linux")]#[path="peer_plan_review.rs"]pub(in crate::pet_runtime)mod peer_plan_review;
#[path="plan_controller.rs"]mod plan_controller;
#[path="resource_wait_controller.rs"]pub(super)mod resource_wait_controller;
#[path="peer_resume_controller.rs"]pub(super)mod peer_resume_controller;
pub(in crate::pet_runtime)use plan_controller::PlanGate;
#[cfg(target_os="linux")]#[path="step_controller.rs"]pub(super)mod step_controller;
#[cfg(target_os="linux")]#[path="recipe_controller.rs"]pub(super)mod recipe_controller;
use super::{admission::{AdmissionSpec,Evidence},authority::{Authority,ConsentProof},context,leases::{Lease,DrainOutcome},model::{self,Record,CommitStatus},store::{Store,nonce}};
use crate::{chat::Chat,pet_resources::Hold};
use std::{collections::BTreeMap,path::{Path,PathBuf},sync::{Arc,Mutex,OnceLock,Weak,atomic::{AtomicBool,AtomicUsize,Ordering}},time::{Instant,Duration}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(super)enum Validity{Current,Changed,Unknown,Busy}
#[derive(Clone,Copy)]pub(super)enum Validation<'a>{FreshEffect,InFlightPolicy,PendingDecision,Commit(&'a Record)}
impl PartialEq for Validation<'_>{fn eq(&self,other:&Self)->bool{matches!((self,other),(Self::FreshEffect,Self::FreshEffect)|(Self::InFlightPolicy,Self::InFlightPolicy)|(Self::PendingDecision,Self::PendingDecision))||matches!((self,other),(Self::Commit(a),Self::Commit(b))if std::ptr::eq(*a,*b))}}
impl Eq for Validation<'_>{}
impl std::fmt::Debug for Validation<'_>{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(match self{Self::FreshEffect=>"FreshEffect",Self::InFlightPolicy=>"InFlightPolicy",Self::PendingDecision=>"PendingDecision",Self::Commit(_)=>"Commit"})}}
pub(super)type Fence=Arc<dyn for<'a>Fn(&Lease,&Hold,Validation<'a>)->Validity+Send+Sync>;
#[derive(Clone,Copy,Debug)]
enum InvalidationCause{StopRequested,OwnerClosed,MonotonicDeadline,WallDeadline,ClockRollback,ContextChanged,InferenceBindingChanged,NativeOwnerChanged,ResourceChanged,AuthorityChanged,PolicyChanged,WaitSourceChanged,WaitGoalChanged,WaitConfigurationChanged}
impl InvalidationCause{
    fn label(self)->&'static str{match self{
        Self::StopRequested=>"pedido de paragem",Self::OwnerClosed=>"executor encerrado",Self::MonotonicDeadline|Self::WallDeadline=>"prazo de execução esgotado",Self::ClockRollback=>"relógio recuou",
        Self::ContextChanged=>"contexto indisponível ou alterado",Self::InferenceBindingChanged=>"configuração de inferência alterada",Self::NativeOwnerChanged=>"executor nativo indisponível ou alterado",Self::ResourceChanged=>"recurso indisponível ou cancelado",
        Self::AuthorityChanged=>"autorização deixou de ser válida",Self::PolicyChanged=>"política alterada",Self::WaitSourceChanged=>"pedido de espera alterado",Self::WaitGoalChanged=>"objetivo em espera alterado",Self::WaitConfigurationChanged=>"configuração da espera alterada",
    }}
}
struct Live{
    lease:Lease,spec:AdmissionSpec,proof:ConsentProof,authority_path:PathBuf,chat:Chat,hold:Arc<Hold>,
    deadline:Instant,closed:AtomicBool,activated:AtomicBool,driver:AtomicBool,workers:AtomicUsize,gate:Mutex<()>,fence:Fence,
    waiting:Mutex<Option<resource_wait_controller::Waiting>>,
    native_completion:Mutex<Option<super::native_verification::Report>>,native_drain:Mutex<Option<super::native_drain::Seal>>,
    owner:super::native_owner::OwnerLease,
    interruption:OnceLock<(InvalidationCause,u64)>,
}
impl Live{
    fn invalidate(&self,cause:InvalidationCause,now:u64)->Validity{
        let first=self.interruption.set((cause,now)).is_ok();self.hold.cancel();
        #[cfg(debug_assertions)]if first{eprintln!("[pet-runtime-invalidation] lease={} cause={cause:?} at={now}",self.lease.id);}
        let _=first;Validity::Changed
    }
    fn close_with_cause(&self,cause:InvalidationCause,now:u64){
        let first=self.interruption.set((cause,now)).is_ok();self.closed.store(true,Ordering::Release);self.hold.cancel();
        #[cfg(debug_assertions)]if first{eprintln!("[pet-runtime-invalidation] lease={} cause={cause:?} at={now}",self.lease.id);}
        let _=first;
    }
    fn current(&self,now:u64)->bool{self.valid_at(now,Validation::FreshEffect)}
    fn valid_at(&self,now:u64,mode:Validation)->bool{
        self.validity_at(now,mode)==Validity::Current
    }
    fn validity_at(&self,now:u64,mode:Validation)->Validity{
        if self.closed.load(Ordering::Acquire){return self.invalidate(InvalidationCause::OwnerClosed,now);}
        if Instant::now()>=self.deadline{return self.invalidate(InvalidationCause::MonotonicDeadline,now);}
        if now<self.lease.claimed_at{return self.invalidate(InvalidationCause::ClockRollback,now);}
        if now>=self.lease.deadline_at{return self.invalidate(InvalidationCause::WallDeadline,now);}
        if !self.chat.goal_context_current(&self.lease.goal_id){return self.invalidate(InvalidationCause::ContextChanged,now);}
        if let Some(capture)=self.hold.captured(){match capture.observation(){crate::pet_inference::CaptureObservation::Current=>{},crate::pet_inference::CaptureObservation::Busy=>return Validity::Busy,crate::pet_inference::CaptureObservation::Changed=>return self.invalidate(InvalidationCause::InferenceBindingChanged,now)}}
        if !self.activated.load(Ordering::Acquire){return Validity::Unknown;}
        if self.owner.current().is_err(){return self.invalidate(InvalidationCause::NativeOwnerChanged,now);}
        let waiting=match self.waiting.lock(){Ok(w)=>w.clone(),Err(_)=>return Validity::Unknown};
        if waiting.is_some()&&!matches!(mode,Validation::PendingDecision){return Validity::Unknown;}
        if waiting.is_none()&&!self.hold.current(){return self.invalidate(InvalidationCause::ResourceChanged,now);}
        match Authority::existing(&self.authority_path).and_then(|a|{
            a.native_owner_current(&self.spec,&self.proof,&self.lease,&self.owner)?;
            if let Some(wait)=&waiting{
            if !wait.released(){return Err(super::authority::Failure::Unavailable);}
            a.resource_wait_current(&self.spec,&self.proof,&self.lease,wait.entry(),now)
        }else{a.resource_held_current(&self.spec,&self.proof,&self.lease,self.hold.generation(),now)}}){
            Ok(())=>{},Err(super::authority::Failure::Busy)=>return Validity::Busy,Err(super::authority::Failure::Unavailable)=>{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] original authority unavailable");return Validity::Unknown;},
            Err(error)=>{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] original authority changed {error:?}");return self.invalidate(InvalidationCause::AuthorityChanged,now);},
        }
        let validity=(self.fence)(&self.lease,&self.hold,mode);#[cfg(debug_assertions)]if validity!=Validity::Current{eprintln!("[pet-runtime-validity] native fence {mode:?} {validity:?}");}if validity==Validity::Changed{return self.invalidate(InvalidationCause::PolicyChanged,now);}validity
    }
}
fn live_registry()->&'static Mutex<BTreeMap<String,Weak<Live>>>{static LIVE:OnceLock<Mutex<BTreeMap<String,Weak<Live>>>>=OnceLock::new();LIVE.get_or_init(||Mutex::new(BTreeMap::new()))}
pub(super)fn signal_stop(lease:&Lease){
    let weak=live_registry().lock().ok().and_then(|r|r.get(&lease.id).cloned());
    if let Some(live)=weak.and_then(|w|w.upgrade()){if live.lease==*lease{live.close_with_cause(InvalidationCause::StopRequested,crate::assistant_memory::now());}}
}
impl Drop for Live{fn drop(&mut self){self.hold.cancel();if let Ok(mut map)=live_registry().lock(){if map.get(&self.lease.id).is_some_and(|w|std::ptr::eq(w.as_ptr(),self)){map.remove(&self.lease.id);}}}}
pub(super)struct Controller{id:String}
pub(super)struct Pending{live:Arc<Live>,drain_request:String,resumed:bool}
pub(super)struct Owned{live:Arc<Live>,drain_request:String,goal_path:PathBuf}
pub(super)struct WorkerGuard{live:Arc<Live>}
pub(crate)struct DrainProof{live:Arc<Live>,request:String,outcome:DrainOutcome,unstarted:bool}
impl Controller{
    pub(super)fn new()->model::Result<Self>{Ok(Self{id:nonce()?})}
    pub(super)fn prepare(&self,goal:&Record,authority_path:&Path,context_path:&Path,main:&Chat,evidence:&Evidence,request:String,now:u64,fence:Fence)->model::Result<Pending>{
        self.prepare_open(goal,authority_path,main,evidence,request,now,fence,None,||context::open_with_authority(context_path,goal,main,authority_path))
    }
    pub(super)fn prepare_requested(&self,goal:&Record,authority_path:&Path,context_path:&Path,main:&Chat,evidence:&Evidence,request:String,now:u64,fence:Fence,hold:Arc<Hold>)->model::Result<Pending>{
        self.prepare_open(goal,authority_path,main,evidence,request,now,fence,Some(hold),||context::open_with_authority(context_path,goal,main,authority_path))
    }
    // Fixtures supply only the catalog path. The same admission, context fence,
    // authority, resource acquisition and lease implementation remain in use.
    #[cfg(test)]
    pub(super)fn prepare_memory_fixture(&self,goal:&Record,authority_path:&Path,context_path:&Path,memory_path:&Path,main:&Chat,evidence:&Evidence,request:String,now:u64,fence:Fence)->model::Result<Pending>{
        self.prepare_open(goal,authority_path,main,evidence,request,now,fence,None,||context::open_data_fixture(context_path,goal,main,memory_path,authority_path))
    }
    fn prepare_open(&self,goal:&Record,authority_path:&Path,main:&Chat,evidence:&Evidence,request:String,now:u64,fence:Fence,resource:Option<Arc<Hold>>,open:impl FnOnce()->model::Result<(model::ContextBinding,Chat)>)->model::Result<Pending>{
        if goal.state!=model::State::Queued||!model::key(&request)||goal.events.iter().any(|e|e.request_id==request){return Err(model::Error::Conflict);}
        let(capability_revision,policy_revision)=evidence.for_claim(goal,now).map_err(|_|model::Error::Conflict)?;
        Authority::existing(authority_path).and_then(|a|a.verify_queued(goal,now)).map_err(super::authority::Failure::control_error)?;
        let row=goal.admissions.last().ok_or(model::Error::InvalidRecord)?;
        let(_,chat)=open()?;
        if !chat.goal_context_current(&goal.id){return Err(model::Error::Conflict);}
        let remaining=row.spec.budget.deadline_at.checked_sub(now).filter(|n|*n>0&&*n<=86_400_000).ok_or(model::Error::Conflict)?;
        let mut deadline=Instant::now().checked_add(Duration::from_millis(remaining)).ok_or(model::Error::Conflict)?;
        let hold=if let Some(hold)=resource{
            if hold.provider_for(&main.runtime_resources())!=Some(goal.spec.owner.provider.as_str())||hold.generation()!=1{return Err(model::Error::Conflict);}deadline=hold.bounded_deadline(deadline).ok_or(model::Error::Conflict)?;hold
        }else if row.spec.deterministic(){main.runtime_resources().acquire_runtime_goal(&goal.spec.owner.provider,Arc::new(AtomicBool::new(false)),deadline).map_err(|_|model::Error::Busy)?}
        else{main.runtime_resources().acquire_goal(&goal.spec.owner.provider,Arc::new(AtomicBool::new(false)),Some(deadline)).map_err(|_|model::Error::Busy)?};
        let lease=Lease{schema_version:if row.spec.deterministic(){2}else{1},id:nonce()?,controller_id:self.id.clone(),request_id:request,decision_id:row.spec.decision_id.clone(),goal_id:goal.id.clone(),provider:goal.spec.owner.provider.clone(),conversation_id:row.spec.context.conversation_id.clone(),model:row.spec.resolved_model.clone(),policy_revision,capability_revision,source_revision:goal.revision.clone(),claimed_at:now,deadline_at:row.spec.budget.deadline_at};
        let owner=super::native_owner::OwnerLease::capture(&Authority::existing(authority_path).map_err(|_|model::Error::StorageUnavailable)?,&lease,now)?;
        let live=Arc::new(Live{lease,spec:row.spec.clone(),proof:row.proof.clone().ok_or(model::Error::Conflict)?,authority_path:authority_path.into(),chat,hold,deadline,closed:AtomicBool::new(false),activated:AtomicBool::new(false),driver:AtomicBool::new(false),workers:AtomicUsize::new(0),gate:Mutex::new(()),fence,waiting:Mutex::new(None),native_completion:Mutex::new(None),native_drain:Mutex::new(None),owner,interruption:OnceLock::new()});
        {let mut map=live_registry().lock().map_err(|_|model::Error::StorageUnavailable)?;map.retain(|_,w|w.strong_count()>0);if map.len()>=64{return Err(model::Error::LimitReached);}map.insert(live.lease.id.clone(),Arc::downgrade(&live));}
        Ok(Pending{live,drain_request:nonce()?,resumed:false})
    }
}
impl Pending{
    pub(in crate::pet_runtime)fn resumed(&self)->bool{self.resumed}
    pub(in crate::pet_runtime)fn same_live(&self,other:&Pending)->bool{Arc::ptr_eq(&self.live,&other.live)}
    pub(in crate::pet_runtime)fn resume_authority(&self,goal:&Record,authority:&Authority)->model::Result<()>{
        if !self.resumed||!self.matches_goal(goal){return Err(model::Error::Conflict);}
        authority.native_owner_current(&self.live.spec,&self.live.proof,&self.live.lease,&self.live.owner).map_err(super::authority::Failure::control_error)
    }
    pub(super)fn pet_id(&self)->&str{&self.live.spec.owner.pet_id}
    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn hold(&self)->&Hold{&self.live.hold}
    pub(super)fn workers(&self)->usize{self.live.workers.load(Ordering::Acquire)}
    pub(super)fn matches_goal(&self,goal:&Record)->bool{original_goal(&self.live,goal)}
    pub(super)fn cleanup_due(&self,goal:&Record,now:u64)->bool{
        !self.live.activated.load(Ordering::Acquire)&&self.matches_goal(goal)
            &&(matches!(goal.state,model::State::Paused|model::State::Cancelled|model::State::Stopping)||self.live.closed.load(Ordering::Acquire)||Instant::now()>=self.live.deadline||now>=self.live.lease.deadline_at)
    }
    pub(super)fn abort(&self)->model::Result<DrainProof>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if self.live.activated.load(Ordering::Acquire)||self.live.workers.load(Ordering::Acquire)!=0{return Err(model::Error::Conflict);}
        self.live.closed.store(true,Ordering::Release);self.live.hold.cancel();Ok(DrainProof{live:self.live.clone(),request:self.drain_request.clone(),outcome:DrainOutcome::Confirmed,unstarted:!self.resumed})
    }
    pub(super)fn claim(&self,store:&Store,authority:&Authority,evidence:&Evidence,now:u64)->model::Result<(model::Receipt,Option<Owned>)>{
        if self.live.closed.load(Ordering::Acquire)||!self.live.hold.current()||!self.live.chat.goal_context_current(&self.live.lease.goal_id){return Err(model::Error::Conflict);}
        let receipt=store.claim_owned(authority,&self.live.lease,&self.live.owner,evidence,now)?;
        if receipt.status==CommitStatus::Uncertain{return Ok((receipt,None));}
        if Instant::now()>=self.live.deadline||!self.live.hold.current(){return Err(model::Error::Conflict);}
        if self.live.activated.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Ok((receipt,None));}
        Ok((receipt,Some(Owned{live:self.live.clone(),drain_request:self.drain_request.clone(),goal_path:store.root_path().to_owned()})))
    }
}
impl Owned{
    pub(in crate::pet_runtime)fn live_matches_pending(&self,pending:&Pending)->bool{Arc::ptr_eq(&self.live,&pending.live)}
    pub(super)fn with_completion_lock<T>(&self,run:impl FnOnce()->model::Result<T>)->model::Result<T>{let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;run()}
    pub(super)fn completion_current(&self,goal:&Record,now:u64)->bool{self.completion_authority(goal,now).is_ok()}
    pub(super)fn completion_authority(&self,goal:&Record,now:u64)->model::Result<()>{
        if self.workers()!=0||goal.state!=model::State::Leased||goal.id!=self.live.spec.goal_id||goal.spec.owner!=self.live.spec.owner
            ||model::fingerprint(&goal.spec).ok().as_ref()!=Some(&self.live.spec.intent_hash)
            ||goal.admissions.last().and_then(|a|a.lease.as_ref())!=Some(&self.live.lease){return Err(model::Error::Conflict);}
        match self.live.validity_at(now,Validation::Commit(goal)){Validity::Current=>Ok(()),Validity::Unknown|Validity::Busy=>Err(model::Error::Busy),Validity::Changed=>Err(model::Error::Conflict)}
    }
    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn current(&self,now:u64)->bool{self.live.current(now)}
    pub(super)fn policy_current(&self,now:u64)->bool{self.live.valid_at(now,Validation::InFlightPolicy)}
    pub(super)fn policy_validity(&self,now:u64)->Validity{self.live.validity_at(now,Validation::InFlightPolicy)}
    pub(super)fn pending_validity(&self,now:u64)->Validity{self.live.validity_at(now,Validation::PendingDecision)}
    pub(super)fn same_live(&self,other:&Owned)->bool{Arc::ptr_eq(&self.live,&other.live)}
    #[cfg(test)]pub(in crate::pet_runtime)fn fixture_owner_record(&self)->super::runtime_owner::Record{self.live.owner.record().clone()}
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
    /// Passive cleanup cannot race the driver that owns its closing proof.
    pub(super)fn driver_active(&self)->bool{self.live.driver.load(Ordering::Acquire)}
    pub(super)fn drain(&self,outcome:DrainOutcome)->model::Result<DrainProof>{
        let _gate=self.live.gate.lock().map_err(|_|model::Error::StorageUnavailable)?;
        self.live.closed.store(true,Ordering::Release);self.live.hold.cancel();
        if self.workers()!=0{return Err(model::Error::Busy);}Ok(DrainProof{live:self.live.clone(),request:self.drain_request.clone(),outcome,unstarted:false})
    }
}
impl WorkerGuard{
    pub(super)fn current(&self,now:u64)->bool{self.live.current(now)}
    pub(super)fn in_flight_current(&self,mut now:u64)->bool{
        // An existing IO observation may encounter another goal's short catalog
        // transaction. Never treat Busy as authority; return only on Current.
        // Fresh workers/effects still use the strict, non-waiting check above.
        let until=Instant::now()+Duration::from_secs(1);
        loop{match self.live.validity_at(now,Validation::InFlightPolicy){
            Validity::Current=>return true,
            Validity::Busy if Instant::now()<until=>{std::thread::sleep(Duration::from_millis(10));now=crate::assistant_memory::now();},
            other=>{#[cfg(debug_assertions)]eprintln!("[pet-runtime-validity] existing IO observation ended {other:?}");return false;},
        }}
    }
}
impl Drop for WorkerGuard{fn drop(&mut self){self.live.workers.fetch_sub(1,Ordering::AcqRel);}}
impl DrainProof{
    pub(super)fn with_native_drain<T>(&self,apply:impl FnOnce(&mut Option<super::native_drain::Seal>)->model::Result<T>)->model::Result<T>{let mut cache=self.live.native_drain.try_lock().map_err(|_|model::Error::Busy)?;apply(&mut cache)}
    pub(super)fn original_authority(&self,authority:&Authority)->model::Result<()>{
        if !self.live.closed.load(Ordering::Acquire)||self.live.workers.load(Ordering::Acquire)!=0||!self.live.owner.matches(authority,&self.live.lease){return Err(model::Error::Conflict);}
        if !self.unstarted{authority.native_owner_current(&self.live.spec,&self.live.proof,&self.live.lease,&self.live.owner).map_err(super::authority::Failure::control_error)?;}Ok(())
    }
    pub(super)fn original_goal(&self,goal:&Record)->bool{
        original_goal(&self.live,goal)
    }
    pub(super)fn deadline_elapsed(&self,now:u64)->bool{now>=self.live.lease.deadline_at||Instant::now()>=self.live.deadline}
    pub(crate)fn covers_drained_hold(&self,hold:&Hold)->bool {
        self.outcome==DrainOutcome::Confirmed&&self.live.closed.load(Ordering::Acquire)
            &&self.live.workers.load(Ordering::Acquire)==0&&std::ptr::eq(Arc::as_ptr(&self.live.hold),hold)
    }

    pub(super)fn lease(&self)->&Lease{&self.live.lease}
    pub(super)fn request_id(&self)->&str{&self.request}
    pub(super)fn confirmed(&self)->bool{self.outcome==DrainOutcome::Confirmed}
    pub(super)fn unstarted(&self)->bool{self.unstarted}
}
fn original_goal(live:&Live,goal:&Record)->bool{
    goal.id==live.lease.goal_id&&goal.spec.owner==live.spec.owner
        &&model::fingerprint(&goal.spec).ok().as_ref()==Some(&live.spec.intent_hash)
        &&goal.execution_context.as_ref()==Some(&live.spec.context)
        &&goal.admissions.last().is_some_and(|row|model::fingerprint(&row.spec).ok()==model::fingerprint(&live.spec).ok())
}

#[cfg(test)]#[path="claim_contention_tests.rs"]mod claim_contention_tests;
#[cfg(test)]#[path="native_control_edge_tests.rs"]mod native_control_edge_tests;

#[path="native_completion_controller.rs"]pub(in crate::pet_runtime)mod native_completion;

#[cfg(all(test,target_os="linux"))]pub(in crate::pet_runtime)use writer::tests::native_system_sound_task_tests;
