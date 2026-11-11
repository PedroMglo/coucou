//! Original-owner autonomous driver. Durable records are observations; only
//! this retained owner and native human decisions can dispatch effects.
use super::*;
use super::recipe_controller::{Approved,Ready,Review};
use crate::pet_runtime::{planning_journal::{self,OutputKind},recipe_decisions::{Source,Decision},recipes::Outcome};
use serde::Serialize;
#[path="driver_closure.rs"]mod closure;
use closure::Closing;
#[path="funded_response_driver.rs"]mod funded_response;
#[path="native_peer_plan_driver.rs"]mod native_peer_plan;
#[path="system_plan_driver.rs"]mod system_plan;
#[path="peer_plan_review_driver.rs"]mod peer_plan_review;
#[path="peer_iteration_driver.rs"]mod peer_iteration;

// Bounded per native driver: typed ledger deserialization under a peer-plan
// review exceeds the platform's 2 MiB default in the debug composition. This
// applies to the actual executor; no environment-wide stack override is needed.
const NATIVE_DRIVER_STACK_BYTES:usize=4*1024*1024;

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
    /// Exact deterministic target, without resolving any inference destination.
    fn system_target(&self,_operation:&crate::pet_runtime::plans::OperationProposal,_pet:&str,_config:&crate::pet_config::Configuration,_current:&dyn Fn()->bool)->model::Result<crate::pet_runtime::physical_target::Target>{Err(model::Error::UnsupportedPlatform)}
    fn decide(&self,source:Source)->Option<Decision>;
    fn notify(&self,status:&Status);
    fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{Err(model::Error::UnsupportedPlatform)}
    fn drained(&self,_proof:&DrainProof)->model::Result<()>{Ok(())}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Phase {Idle,Planning,Publishing,AwaitingApproval,WaitingResource,WaitingPeers,AwaitingVerification,Closing,Registering,Executing,Feedback,Completed,Declined,ApprovalEnded,Stopped,Limited,RecoveryRequired}
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Status {
    pub goal_id:String,pub pet_id:String,pub lease_id:String,pub controller_id:String,
    pub phase:Phase,pub round:u8,pub active:bool,pub retained_approval:bool,pub retained_closure:bool,pub reason:Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]pub resource_wait:Option<crate::pet_resources::WaitInfo>,
}
struct Retained {goal:Record,approved:Approved,ready:Ready,registered:AtomicBool}
struct Awaiting {review:Mutex<Option<Review>>,source:Source,decision:Mutex<Option<Decision>>}
struct WorkRetained{goal:Record,approved:super::message_controller::Approved}
struct PeerAwaiting{goal:Record,batch:Arc<crate::pet_runtime::peer_plan_review::Batch>,source:Source,decision:Mutex<Option<Decision>>,approved:Mutex<Option<Vec<Arc<super::message_controller::Approved>>>>}
struct State {data_publication:Option<Arc<super::PeerRoundPublished>>,peer:Option<Arc<PeerAwaiting>>,work:Option<Arc<WorkRetained>>,last_failure:Option<String>,status:Status,retained:Option<Arc<Retained>>,awaiting:Option<Arc<Awaiting>>,closing:Option<Arc<Closing>>}
pub(in crate::pet_runtime)struct Driver {owner:Arc<Owned>,store:Arc<Store>,environment:Arc<dyn Environment>,state:Mutex<State>,active:AtomicBool,drain_confirmed:AtomicBool}
/// A single claim shared by ALL Driver objects on the original lease.
struct Claim<'a>{driver:&'a Driver}
impl Drop for Claim<'_>{fn drop(&mut self){self.driver.owner.live.driver.store(false,Ordering::Release);self.driver.active.store(false,Ordering::Release);}}
impl Driver {
    pub(in crate::pet_runtime)fn new(owner:Arc<Owned>,store:Arc<Store>,environment:Arc<dyn Environment>)->model::Result<Arc<Self>> {
        if !owner.live.spec.native_system_requested()&&!crate::pet_runtime::native_peer_plan::family(&owner.live.spec)&&owner.live.spec.native_peer_plan.is_none()&&owner.live.spec.result_disclosure.is_none()&&!owner.live.spec.finite_response()&&!(owner.live.spec.schema_version==9&&owner.live.spec.work_disclosure.is_some())&&(!owner.live.spec.planning()||owner.live.spec.planning_policy.is_none()||!owner.live.spec.owner.local_engine()||owner.lease().provider!=owner.live.spec.owner.provider) {return Err(model::Error::Conflict);}
        let lease=owner.lease();let status=Status{goal_id:lease.goal_id.clone(),pet_id:owner.live.spec.owner.pet_id.clone(),lease_id:lease.id.clone(),controller_id:lease.controller_id.clone(),phase:Phase::Idle,round:0,active:false,retained_approval:false,retained_closure:false,reason:None,resource_wait:None};
        Ok(Arc::new(Self{owner,store,environment,state:Mutex::new(State{data_publication:None,peer:None,work:None,last_failure:None,status,retained:None,awaiting:None,closing:None}),active:AtomicBool::new(false),drain_confirmed:AtomicBool::new(false)}))
    }
    pub(in crate::pet_runtime)fn status(&self)->model::Result<Status> {
        let state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;let mut status=state.status.clone();
        status.resource_wait=self.owner.live.hold.waiting_info();status.active=self.active.load(Ordering::Acquire);status.retained_closure=state.closing.is_some();status.retained_approval=state.data_publication.as_ref().is_some_and(|p|p.retains_native_data_approval())||state.peer.as_ref().is_some_and(|p|p.approved.lock().is_ok_and(|a|a.is_some()))||state.work.is_some()||state.retained.is_some()||state.awaiting.as_ref().is_some_and(|a|a.decision.lock().is_ok_and(|d|d.is_some()));
        if matches!(status.phase,Phase::Stopped|Phase::RecoveryRequired){if let Some((cause,_))=self.owner.live.interruption.get(){status.reason=Some(format!("Motivo inicial: {}. {}",cause.label(),status.reason.as_deref().unwrap_or("")).chars().take(1024).collect());}}
        Ok(status)
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
        if std::thread::Builder::new().name("pet-goal-driver".into()).stack_size(NATIVE_DRIVER_STACK_BYTES).spawn(move||{
            let claim=Claim{driver:&driver};
            match tokio::runtime::Builder::new_current_thread().enable_all().thread_stack_size(NATIVE_DRIVER_STACK_BYTES).max_blocking_threads(1).build(){
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
            Validity::Unknown|Validity::Busy=>tokio::time::sleep(Duration::from_millis(50)).await,
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
        if self.owner.live.spec.result_disclosure.is_some()&&self.owner.workers()==0{return self.finish(Phase::RecoveryRequired,round,Some("Entrega não confirmada; o worker original terminou. O recibo ou preparação incertos conservam-se, sem repetir efeitos.")).await;}
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
    async fn register_retained(&self,retained:&Retained,round:u8)->model::Result<bool>{
        let owner=&self.owner;let spec=&owner.live.spec;let lease=owner.lease();let authority=self.authority()?;
                if !retained.registered.load(Ordering::Acquire) {
                    if !self.current().await?{self.stopped(round).await?;return Ok(false);}
                    self.notice(Phase::Registering,round,None)?;
                    // Busy is obtained before the registration write. Keep this exact
                    // native choice; never retry an uncertain publication here.
                    let until=Instant::now()+Duration::from_secs(1);
                    let scope=loop{match authority.issue_recipe(spec,&owner.live.proof,lease,&retained.approved,crate::assistant_memory::now()){
                        Ok(scope)=>break scope,
                        Err(crate::pet_runtime::authority::Failure::Busy)if Instant::now()<until=>{
                            if !self.current().await?{self.stopped(round).await?;return Ok(false);}
                            tokio::time::sleep(Duration::from_millis(20)).await;
                        },
                        Err(failure)=>{
                            self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure=Some(format!("O registo da receita original não foi confirmado: {failure:?}"));
                            return Err(failure.control_error());
                        },
                    }};
                    retained.ready.activate(&scope)?;retained.registered.store(true,Ordering::Release);
                }
        Ok(true)
    }
    async fn drive(&self)->model::Result<()> {
        let owner=&self.owner;let spec=&owner.live.spec;let lease=owner.lease();
        loop {
            let closing=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.closing.clone();
            if let Some(closing)=closing{return self.resume_closure(closing).await;}
            if spec.native_system_requested(){return self.drive_native_system_plan().await;}
            if spec.native_peer_plan.is_some(){return self.drive_native_peer_plan().await;}
            if crate::pet_runtime::native_peer_plan::family(spec){return self.drive_peer_plan_review(0).await;}
            if spec.finite_response(){return self.drive_funded_response().await;}
            if self.owner.live.spec.work_disclosure.is_some()||self.owner.live.spec.result_disclosure.is_some(){return self.drive_disclosure().await;}
            if spec.peer_iteration(){return self.drive_peer_iteration().await;}
            let authority=self.authority()?;
            let journal=authority.planning_ledger(spec,&owner.live.proof,lease,crate::assistant_memory::now()).map_err(|_|model::Error::StorageUnavailable)?;
            let round=journal.rounds.len()as u8;
            let awaiting=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.awaiting.clone();
            if let Some(awaiting)=awaiting{if !self.await_recipe(awaiting,round).await?{return Ok(());}continue;}
            let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained.clone();
            if let Some(retained)=retained {
                // Keep the ORIGINAL approval/Ready across registration, native
                // proof and completion publication uncertainty. Never rebuild.
                if !self.register_retained(&retained,round).await?{return Ok(());}
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
                        // The ordinary policy reader intentionally refuses a
                        // Prepared publication. Re-entering it here would wait
                        // for the very ACK this exact retry must produce.
                        // Each publish attempt rechecks the original zero-worker
                        // gate, current policy, lease/deadline and certificate
                        // against the canonical control record. No inference or
                        // effects are replayed, and Stop remains prioritary.
                        if self.owner.live.closed.load(Ordering::Acquire)||self.owner.live.hold.stopped(){return self.stopped(round).await;}
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    },
                    Err(error)=>{self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.last_failure=Some(format!("Publicação do plano interrompida: {error:?}"));return Err(error);},
                }};
                if !publication.acknowledged||publication.receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}
                if !self.current().await?{return self.stopped(round).await;}
                if spec.peer_planning(){return self.drive_peer_plan_review(round).await;}
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
        // A yielded native owner may still observe its exact binding and DATA
        // review. It must not claim a physical-slot exemption while another
        // worker owns that slot. Same passive rule as the PendingDecision fence;
        // this capture grants neither readiness, reacquisition nor model IO.
        let owned=if owner.waiting_for_resource()&&!owner.live.hold.current(){None}else{Some(owner.live.hold.as_ref())};
        let projection=crate::pet_capabilities::native_projection(&self.app,owned).map_err(|_|model::Error::StorageUnavailable)?;
        let goal=self.store.read(&owner.lease().goal_id,&owner.live.spec.owner.pet_id)?;
        let configuration=projection.configuration.ok_or(model::Error::StorageUnavailable)?;
        let descriptor=projection.snapshot.pets.iter().find(|d|d.pet_id==goal.spec.owner.pet_id).ok_or(model::Error::Conflict)?;
        let policy=crate::pet_runtime::native_validity::policy_revision(&configuration,&projection.settings,&goal)?;
        if crate::pet_runtime::native_validity::policy(&goal,owner.lease(),descriptor,&policy,crate::assistant_memory::now())!=Validity::Current{return Err(model::Error::Conflict);}
        let server=owner.server_projection(&projection.settings).map_err(|_|model::Error::Conflict)?;
        Ok(Capture{configuration,server,app:Some(self.app.clone())})
    }
    fn configuration(&self)->model::Result<crate::pet_config::Configuration>{crate::pet_config::read_existing().map_err(|_|model::Error::StorageUnavailable)}
    fn peer_configuration(&self)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{let p=crate::pet_capabilities::native_projection(&self.app,None).map_err(|_|model::Error::StorageUnavailable)?;if !p.preferences_ready{return Err(model::Error::Conflict);}Ok((p.configuration.ok_or(model::Error::StorageUnavailable)?,p.settings))}
    fn target(&self)->model::Result<Arc<crate::spotify::goal::Target>>{Ok(Arc::new(crate::spotify::goal::Target::capture().map_err(|_|model::Error::Conflict)?))}
    fn system_target(&self,operation:&crate::pet_runtime::plans::OperationProposal,pet:&str,config:&crate::pet_config::Configuration,current:&dyn Fn()->bool)->model::Result<crate::pet_runtime::physical_target::Target>{crate::pet_runtime::physical_target::Target::capture(operation,pet,config,current)}
    fn decide(&self,source:Source)->Option<Decision>{crate::cli_chat::ask_goal_recipe_decision(&self.app,source)}
    fn drained(&self,proof:&DrainProof)->model::Result<()>{crate::pet_runtime::lease_api::retire_driver(&self.store,proof)}
    fn notify(&self,status:&Status){use tauri::Emitter;let _=self.app.emit("pet-runtime-driver",status);}
}
impl Driver {
    pub(in crate::pet_runtime)fn for_app(owner:Arc<Owned>,store:Arc<Store>,app:tauri::AppHandle)->model::Result<Arc<Self>> {
        Self::new(owner,store.clone(),Arc::new(NativeEnvironment{app,store}))
    }
    pub(in crate::pet_runtime)fn context_retirable(&self)->bool{
        self.drain_confirmed.load(Ordering::Acquire)&&!self.active.load(Ordering::Acquire)&&!self.owner.live.driver.load(Ordering::Acquire)&&self.owner.workers()==0&&self.state.lock().ok().is_some_and(|s|s.closing.is_none()&&s.retained.is_none()&&s.awaiting.is_none()&&s.work.is_none()&&s.peer.is_none()&&s.data_publication.is_none())
    }
    pub(in crate::pet_runtime)fn removable(&self)->bool {
        !self.active.load(Ordering::Acquire)&&self.state.lock().ok().is_some_and(|s|s.closing.is_none()&&s.retained.is_none()
            &&(s.status.phase==Phase::Completed||self.owner.live.closed.load(Ordering::Acquire)||Instant::now()>=self.owner.live.deadline))
    }
}

impl Driver{
 async fn drive_disclosure(&self)->model::Result<()>{
  let (plan_id,step_id,recipient,derived)=if let Some(p)=&self.owner.live.spec.result_disclosure{(p.cause.plan_id.clone(),p.cause.step_id.clone(),p.recipient.clone(),true)}else{let p=self.owner.live.spec.work_disclosure.as_ref().ok_or(model::Error::Conflict)?;(p.plan_id.clone(),p.step_id.clone(),p.recipient.clone(),false)};let goal=self.store.read_control(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;
  let authority=self.authority()?;
  // A canonical receipt is a fact, even if the subject/configuration has since
  // changed. Never derive a second envelope to repair a lost ACK.
  if authority.message_at(&self.owner.live.spec,&self.owner.live.proof,&plan_id,&step_id).map_err(crate::pet_runtime::authority::Failure::control_error)?.is_none(){
   if !self.current().await?{return self.stopped(0).await;}
   let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work.clone();
   let retained=if let Some(r)=retained{r}else{
    let(config,settings)=self.environment.peer_configuration()?;let target=Arc::new(crate::pet_runtime::messages::Target::capture(&config,&settings,&recipient.pet_id)?);
    let review=self.owner.review_message(&goal,&plan_id,&step_id,"bounded-native-disclosure".into(),target,crate::assistant_memory::now())?;
    let approved=review.approve_disclosure(&self.owner,&goal,&config,&settings,crate::assistant_memory::now())?;let r=Arc::new(WorkRetained{goal:goal.clone(),approved});self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work=Some(r.clone());r
   };
   self.notice(Phase::Executing,0,Some(if derived{"A entregar o excerto e a proveniência explicitamente autorizados; sem inferência adicional."}else{"A publicar apenas os metadados futuros explicitamente autorizados."}))?;
   // Only typed Busy with the retained EXACT envelope is retried. Unknown IO,
   // staging and changed source remain recovery, not a new attempt/nonce.
   let until=Instant::now()+Duration::from_secs(2);
   loop{match self.owner.send_message(&retained.goal,&retained.approved,crate::assistant_memory::now(),||self.environment.peer_configuration()){
    Ok(_)=>break,Err(model::Error::Busy)if Instant::now()<until=>tokio::time::sleep(Duration::from_millis(25)).await,Err(e)=>return Err(e),
   }}
  }
  let current=self.store.read_control(&goal.id,&goal.spec.owner.pet_id)?;
  if derived&&self.owner.live.spec.native_verification.is_none(){let cancel=self.sync_funded_stop()?;return self.finish(if cancel{Phase::Stopped}else{Phase::AwaitingVerification},0,Some("Entrega local guardada; critérios arbitrários continuam por verificar. Sem nova inferência ou orçamento.")).await;}
  let progress=self.owner.verify_native_goal(&self.store,&current)?;
  if derived&&progress.completion.is_none(){let cancel=self.sync_funded_stop()?;return self.finish(if cancel{Phase::Stopped}else{Phase::AwaitingVerification},0,Some("Recibo local verificado; restantes critérios não estão comprovados.")).await;}
  let receipt=progress.completion.ok_or(model::Error::Conflict)?;if receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}self.close_completed(&receipt.record,0).await
 }
}
