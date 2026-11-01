//! Original-owner autonomous driver. Durable records are observations; only
//! this retained owner and native human decisions can dispatch effects.
use super::*;
use super::recipe_controller::{Approved,Ready,Review};
use crate::pet_runtime::{planning_journal::{self,OutputKind},recipe_decisions::{Source,Decision},recipes::Outcome};
use serde::Serialize;
#[path="driver_closure.rs"]mod closure;
use closure::Closing;

pub(in crate::pet_runtime)struct Capture {
    pub configuration:crate::pet_config::Configuration,
    pub server:crate::local_chat::Server,
    pub app:Option<tauri::AppHandle>,
}
pub(in crate::pet_runtime)trait Environment:Send+Sync {
    /// Production implementations capture configuration/model from ONE native
    /// projection. Callers cannot supply serialized server/catalog authority.
    fn capture(&self,owner:&Owned)->model::Result<Capture>;
    fn configuration(&self)->model::Result<crate::pet_config::Configuration>;
    fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>;
    fn decide(&self,source:Source)->Option<Decision>;
    fn notify(&self,status:&Status);
    fn drained(&self,_proof:&DrainProof)->model::Result<()>{Ok(())}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Phase {Idle,Planning,Publishing,AwaitingApproval,WaitingResource,Closing,Registering,Executing,Feedback,Completed,Declined,ApprovalEnded,Stopped,Limited,RecoveryRequired}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Status {
    pub goal_id:String,pub pet_id:String,pub lease_id:String,pub controller_id:String,
    pub phase:Phase,pub round:u8,pub active:bool,pub retained_approval:bool,pub retained_closure:bool,pub reason:Option<String>,
}
struct Retained {goal:Record,approved:Approved,ready:Ready,registered:AtomicBool}
struct Awaiting {review:Mutex<Option<Review>>,source:Source,decision:Mutex<Option<Decision>>}
struct State {last_failure:Option<String>,status:Status,retained:Option<Arc<Retained>>,awaiting:Option<Arc<Awaiting>>,closing:Option<Arc<Closing>>}
pub(in crate::pet_runtime)struct Driver {owner:Arc<Owned>,store:Arc<Store>,environment:Arc<dyn Environment>,state:Mutex<State>,active:AtomicBool}
/// A single claim shared by ALL Driver objects on the original lease.
struct Claim<'a>{driver:&'a Driver}
impl Drop for Claim<'_>{fn drop(&mut self){self.driver.owner.live.driver.store(false,Ordering::Release);self.driver.active.store(false,Ordering::Release);}}
impl Driver {
    pub(in crate::pet_runtime)fn new(owner:Arc<Owned>,store:Arc<Store>,environment:Arc<dyn Environment>)->model::Result<Arc<Self>> {
        if owner.live.spec.schema_version!=3||owner.live.spec.planning_policy.is_none()||!owner.live.spec.owner.local_engine()||owner.lease().provider!=owner.live.spec.owner.provider {return Err(model::Error::Conflict);}
        let lease=owner.lease();let status=Status{goal_id:lease.goal_id.clone(),pet_id:owner.live.spec.owner.pet_id.clone(),lease_id:lease.id.clone(),controller_id:lease.controller_id.clone(),phase:Phase::Idle,round:0,active:false,retained_approval:false,retained_closure:false,reason:None};
        Ok(Arc::new(Self{owner,store,environment,state:Mutex::new(State{last_failure:None,status,retained:None,awaiting:None,closing:None}),active:AtomicBool::new(false)}))
    }
    pub(in crate::pet_runtime)fn status(&self)->model::Result<Status> {
        let state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;let mut status=state.status.clone();
        status.active=self.active.load(Ordering::Acquire);status.retained_closure=state.closing.is_some();status.retained_approval=state.retained.is_some()||state.awaiting.as_ref().is_some_and(|a|a.decision.lock().is_ok_and(|d|d.is_some()));Ok(status)
    }
    fn notice(&self,phase:Phase,round:u8,reason:Option<&str>)->model::Result<()> {
        {let mut state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;state.status.phase=phase;state.status.round=round;state.status.reason=reason.map(|s|s.chars().take(1024).collect());}
        self.environment.notify(&self.status()?);Ok(())
    }
    fn claim(&self)->model::Result<Claim<'_>> {
        let state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if matches!(state.status.phase,Phase::Declined|Phase::ApprovalEnded){return Err(model::Error::Conflict);}drop(state);
        if self.active.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(model::Error::Busy);}
        let gate=match self.owner.live.gate.lock(){Ok(g)=>g,Err(_)=>{self.active.store(false,Ordering::Release);return Err(model::Error::StorageUnavailable);}};
        if self.owner.workers()!=0||self.owner.live.driver.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){self.active.store(false,Ordering::Release);return Err(model::Error::Busy);}
        drop(gate);Ok(Claim{driver:self})
    }
    /// Retained by the native thread, independently of any window or IPC future.
    pub(in crate::pet_runtime)fn spawn(self:&Arc<Self>)->model::Result<()> {
        // Reserve synchronously so a second start cannot race thread creation.
        let claim=self.claim()?;std::mem::forget(claim);let driver=self.clone();
        if std::thread::Builder::new().name("pet-goal-driver".into()).spawn(move||{
            let claim=Claim{driver:&driver};
            match tokio::runtime::Builder::new_current_thread().enable_all().build(){
                Ok(runtime)=>{runtime.block_on(async{if let Err(error)=driver.drive().await{let round=driver.status().map(|s|s.round).unwrap_or(0);let _=driver.failed(round,&error).await;}});},
                Err(_)=>{let _=driver.notice(Phase::RecoveryRequired,0,Some("O executor nativo está indisponível."));},
            }
            drop(claim);if let Ok(status)=driver.status(){driver.environment.notify(&status);}
        }).is_err(){self.owner.live.driver.store(false,Ordering::Release);self.active.store(false,Ordering::Release);return Err(model::Error::StorageUnavailable);}Ok(())
    }
    #[cfg(test)]pub(in crate::pet_runtime)async fn run(&self)->model::Result<Status> {
        let claim=self.claim()?;let result=self.drive().await;
        if let Err(error)=result{let round=self.status()?.round;self.failed(round,&error).await?;drop(claim);return Err(error);}drop(claim);self.status()
    }
    fn authority(&self)->model::Result<Authority>{Authority::existing(&self.owner.live.authority_path).map_err(crate::pet_runtime::authority::Failure::control_error)}
    async fn current(&self)->model::Result<bool>{
        loop{match self.owner.policy_validity(crate::assistant_memory::now()){
            Validity::Current=>return Ok(true),Validity::Changed=>return Ok(false),
            Validity::Unknown=>tokio::time::sleep(Duration::from_millis(50)).await,
        }}
    }
    async fn failed(&self,round:u8,error:&model::Error)->model::Result<()> {
        let detail=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure.take();
        // A retained closure never falls back to planning/recipe/model work.
        if self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.closing.is_some(){return Ok(());}
        if (self.owner.live.closed.load(Ordering::Acquire)||self.owner.live.hold.stopped())&&self.owner.workers()==0 {
            if self.finish(Phase::Stopped,round,Some("Execução interrompida e IO terminado; os resultados incertos conservam os recibos.")).await.is_ok(){return Ok(());}
            if self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.closing.is_some(){return Ok(());}
        }
        let reason=if self.owner.waiting_for_resource(){format!("É necessária confirmação da cedência ou retoma do recurso; o estado original foi conservado. {error:?}")}else{detail.unwrap_or_else(||format!("{error:?}"))};
        self.notice(Phase::RecoveryRequired,round,Some(&reason))
    }
    async fn stopped(&self,round:u8)->model::Result<()> {self.finish(Phase::Stopped,round,Some("O consentimento, o prazo ou a disponibilidade original deixaram de estar atuais.")).await}
    async fn await_recipe(&self,awaiting:Arc<Awaiting>,round:u8)->model::Result<bool>{
        let now=crate::assistant_memory::now();
        if awaiting.decision.lock().map_err(|_|model::Error::StorageUnavailable)?.is_none(){
            self.owner.yield_for_decision(&self.store,&awaiting.source,now)?;
            self.notice(Phase::AwaitingApproval,round,None)?;
            let Some(decision)=self.environment.decide(awaiting.source.clone())else{
                if self.owner.live.closed.load(Ordering::Acquire)||self.owner.live.hold.stopped(){self.stopped(round).await?;return Ok(false);}
                self.finish(Phase::ApprovalEnded,round,Some("O pedido de aprovação terminou; não foi renovado automaticamente.")).await?;return Ok(false);
            };
            if !decision.approved(){self.finish(Phase::Declined,round,Some("Receita recusada; não foi executada nem foi pedido novo consentimento.")).await?;return Ok(false);}
            *awaiting.decision.lock().map_err(|_|model::Error::StorageUnavailable)?=Some(decision);
        }
        loop{
            match self.owner.resume_after_decision(&self.store,&awaiting.source,crate::assistant_memory::now()){
                Ok(())=>break,Err(model::Error::Busy)=>{self.notice(Phase::WaitingResource,round,Some("A aguardar o recurso original; o prazo e o orçamento mantêm-se."))?;tokio::time::sleep(Duration::from_millis(50)).await;},
                Err(error)=>return Err(error),
            }
        }
        let current=self.store.read(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;
        let review=awaiting.review.lock().map_err(|_|model::Error::StorageUnavailable)?.take().ok_or(model::Error::Conflict)?;
        let decision=awaiting.decision.lock().map_err(|_|model::Error::StorageUnavailable)?.take().ok_or(model::Error::Conflict)?;
        let(approved,ready)=review.approve_decision(decision,&self.owner,&current,&self.environment.configuration()?,crate::assistant_memory::now())?;
        let mut state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;state.retained=Some(Arc::new(Retained{goal:current,approved,ready,registered:AtomicBool::new(false)}));state.awaiting=None;Ok(true)
    }
    async fn drive(&self)->model::Result<()> {
        let owner=&self.owner;let spec=&owner.live.spec;let lease=owner.lease();
        loop {
            let closing=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.closing.clone();
            if let Some(closing)=closing{return self.resume_closure(closing).await;}
            let authority=self.authority()?;
            let journal=authority.planning_ledger(spec,&owner.live.proof,lease,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
            let round=journal.rounds.len()as u8;
            let awaiting=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.awaiting.clone();
            if let Some(awaiting)=awaiting{if !self.await_recipe(awaiting,round).await?{return Ok(());}continue;}
            let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained.clone();
            if let Some(retained)=retained {
                // Keep the ORIGINAL approval/Ready across registration, native
                // proof and completion publication uncertainty. Never rebuild.
                if !retained.registered.load(Ordering::Acquire) {
                    if !self.current().await?{return self.stopped(round).await;}
                    self.notice(Phase::Registering,round,None)?;
                    // Busy is obtained before the registration write. Keep this exact
                    // native choice; never retry an uncertain publication here.
                    let until=Instant::now()+Duration::from_secs(1);
                    let scope=loop{match authority.issue_recipe(spec,&owner.live.proof,lease,&retained.approved,crate::assistant_memory::now()){
                        Ok(scope)=>break scope,
                        Err(crate::pet_runtime::authority::Failure::Busy)if Instant::now()<until=>{
                            if !self.current().await?{return self.stopped(round).await;}
                            tokio::time::sleep(Duration::from_millis(20)).await;
                        },
                        Err(failure)=>{
                            self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure=Some(format!("O registo da receita original não foi confirmado: {failure:?}"));
                            return Err(failure.control_error());
                        },
                    }};
                    retained.ready.activate(&scope)?;retained.registered.store(true,Ordering::Release);
                }
                self.notice(Phase::Executing,round,None)?;
                let result=owner.run_recipe_for_store(&self.store,&retained.goal,&retained.ready,||self.environment.configuration(),|_|{})?;
                if !matches!(result.outcome,Outcome::GoalCompleted|Outcome::CriteriaUnmet) {
                    if (owner.live.closed.load(Ordering::Acquire)||owner.live.hold.stopped())&&owner.workers()==0{return self.stopped(round).await;}
                    return self.notice(Phase::RecoveryRequired,round,Some("A execução ou os critérios finais têm um resultado desconhecido; não repetir nem replanear."));
                }
                self.notice(Phase::Feedback,round,None)?;
                let feedback=authority.record_planning_feedback(spec,&owner.live.proof,lease,retained.ready.scope(),crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
                if result.outcome==Outcome::GoalCompleted {
                    let goal=self.store.read(&lease.goal_id,&spec.owner.pet_id)?;
                    self.close_completed(&goal,round).await?;
                    self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained=None;return Ok(());
                }
                if !feedback.continuable(){return self.notice(Phase::RecoveryRequired,round,Some("Não há observação nativa final conhecida que permita outra ronda."));}
                self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained=None;
                continue;
            }
            let goal=self.store.read_control(&lease.goal_id,&spec.owner.pet_id)?;
            if goal.state==model::State::Completed {return self.close_completed(&goal,round).await;}
            let last=journal.rounds.last();
            if let Some(last)=last.filter(|r|r.output.as_ref().is_some_and(|o|o.kind==OutputKind::Plan)&&r.feedback.is_none()) {
                self.notice(Phase::Publishing,round,None)?;
                // Retry only a typed Busy while retaining this same original
                // owner, request and signed result. Never replay inference or
                // retry an uncertain publication/ACK as if it had not happened.
                let until=std::time::Instant::now()+std::time::Duration::from_secs(1);
                let publication=loop{match owner.publish_generated_plan(&self.store,&last.request_id,crate::assistant_memory::now()){
                    Ok(publication)=>break publication,
                    Err(model::Error::Busy)if std::time::Instant::now()<until=>{
                        if !self.current().await?{return self.stopped(round).await;}
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    },
                    Err(error)=>{self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure=Some(format!("Publicação do plano interrompida: {error:?}"));return Err(error);},
                }};
                if !publication.acknowledged||publication.receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}
                if !self.current().await?{return self.stopped(round).await;}
                let goal=publication.receipt.record;let reference=last.publication.as_ref().map(|p|p.reference.plan_id.clone()).or_else(||goal.generated_plans.last().map(|r|r.plan_id.clone())).ok_or(model::Error::Conflict)?;
                // A manifest without this retained native approval is recovery,
                // never a reason to recreate an executable Ready from JSON.
                if authority.read_recipe(spec,&owner.live.proof,&reference).map_err(|_|model::Error::StorageUnavailable)?.is_some(){return self.notice(Phase::RecoveryRequired,round,Some("A aprovação nativa original não está retida; a receita guardada não recria autoridade de execução."));}
                let configuration=self.environment.configuration()?;
                let review=owner.review_recipe_with_criteria(&goal,&reference,&format!("driver-recipe-{round}"),self.environment.target()?,&configuration,&[],crate::assistant_memory::now())?;
                let environment=self.environment.clone();
                let source=Source::capture(owner.clone(),&goal,&review,self.store.clone(),Arc::new(move||environment.configuration()))?;
                self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.awaiting=Some(Arc::new(Awaiting{review:Mutex::new(Some(review)),source,decision:Mutex::new(None)}));
                continue;
            }
            if !self.current().await?{return self.stopped(round).await;}
            let request=if let Some(last)=last.filter(|r|r.output.is_none()) {
                if last.phase!=crate::pet_runtime::effect_budget::Phase::Settled{return self.notice(Phase::RecoveryRequired,round,Some("A tentativa anterior não tem um resultado final conhecido; não repetir a inferência."));}
                last.request_id.clone()
            }else {
                let request=format!("driver-plan-{}",round+1);
                let captured=planning_journal::Request::capture(&goal,spec,lease,&request)?;
                match authority.planning_context(spec,&owner.live.proof,lease,captured,crate::assistant_memory::now()) {
                    Ok(_)=>{},Err(crate::pet_runtime::authority::Failure::Capacity)=>return self.finish(Phase::Limited,round,Some("Atingido o limite original de rondas, progresso, chamadas ou saída.")).await,
                    Err(_)=>return Err(model::Error::Conflict),
                }request
            };
            self.notice(Phase::Planning,round+1,None)?;
            let capture=self.environment.capture(owner)?;
            if let Err(message)=owner.plan_once(capture.app.as_ref(),&goal,&capture.server,&request,&capture.configuration).await{
                self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure=Some(format!("Planeamento interrompido: {message}"));
                return Err(model::Error::StorageUnavailable);
            }
            // Read back the signed journal on the next iteration. Model text is
            // never the state, publication ACK, human approval or final proof.
        }
    }
}

/// App-owned bindings. The frontend never supplies this projection or target.
struct NativeEnvironment {app:tauri::AppHandle,store:Arc<Store>}
impl Environment for NativeEnvironment {
    fn capture(&self,owner:&Owned)->model::Result<Capture> {
        let projection=crate::pet_capabilities::native_projection(&self.app,Some(&owner.live.hold)).map_err(|_|model::Error::StorageUnavailable)?;
        let goal=self.store.read(&owner.lease().goal_id,&owner.live.spec.owner.pet_id)?;
        let configuration=projection.configuration.ok_or(model::Error::StorageUnavailable)?;
        let descriptor=projection.snapshot.pets.iter().find(|d|d.pet_id==goal.spec.owner.pet_id).ok_or(model::Error::Conflict)?;
        let policy=crate::pet_runtime::native_validity::policy_revision(&configuration,&projection.settings,&goal)?;
        if crate::pet_runtime::native_validity::policy(&goal,owner.lease(),descriptor,&policy,crate::assistant_memory::now())!=Validity::Current{return Err(model::Error::Conflict);}
        let server=crate::local_chat::server(&projection.settings,&owner.lease().provider).ok_or(model::Error::Conflict)?;
        Ok(Capture{configuration,server,app:Some(self.app.clone())})
    }
    fn configuration(&self)->model::Result<crate::pet_config::Configuration>{crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)}
    fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{Ok(Arc::new(crate::spotify::goal::Target::capture().map_err(|_|model::Error::Conflict)?))}
    fn decide(&self,source:Source)->Option<Decision>{crate::cli_chat::ask_goal_recipe_decision(&self.app,source)}
    fn drained(&self,proof:&DrainProof)->model::Result<()>{crate::pet_runtime::lease_api::retire_driver(&self.store,proof)}
    fn notify(&self,status:&Status){use tauri::Emitter;let _=self.app.emit("pet-runtime-driver",status);}
}
impl Driver {
    pub(in crate::pet_runtime)fn for_app(owner:Arc<Owned>,store:Arc<Store>,app:tauri::AppHandle)->model::Result<Arc<Self>> {
        Self::new(owner,store.clone(),Arc::new(NativeEnvironment{app,store}))
    }
    pub(in crate::pet_runtime)fn removable(&self)->bool {
        !self.active.load(Ordering::Acquire)&&self.state.lock().ok().is_some_and(|s|s.closing.is_none()&&s.retained.is_none()
            &&(s.status.phase==Phase::Completed||self.owner.live.closed.load(Ordering::Acquire)||Instant::now()>=self.owner.live.deadline))
    }
}
