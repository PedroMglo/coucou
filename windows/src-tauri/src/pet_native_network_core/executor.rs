//! Private backend actor. Native Tauri authority acquisition remains external.
use crate::{lifecycle as l, transport as t};
pub(crate) use crate::authority_fence::{AuthorityFence,NativeWindowOwner,WindowRef};
use crate::authority_fence::Stamp;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

static SINGLETON: AtomicBool = AtomicBool::new(false);
struct Permit;
impl Permit {
    fn claim() -> Result<Arc<Self>, l::Reason> {
        SINGLETON.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| l::Reason::Busy)?;
        Ok(Arc::new(Self))
    }
}
impl Drop for Permit { fn drop(&mut self) { SINGLETON.store(false, Ordering::Release); } }

/// Backend inputs only; neither Deserialize nor a renderer command payload.
pub(crate) struct Config {
    pub lifecycle: l::Config,
    pub boot_nonce: u128,
    pub producer_nonce: u64,
    pub command_capacity: usize,
}
struct ProducerCounter { nonce: u64, next: u64 }
impl ProducerCounter {
    fn reserve(&mut self) -> Result<l::ProducerId, l::Reason> {
        if self.nonce == 0 { return Err(l::Reason::Capacity); }
        let next = self.next.checked_add(1).ok_or(l::Reason::Capacity)?;
        let mut bytes = [0u8;16];
        bytes[..8].copy_from_slice(&self.nonce.to_be_bytes());
        bytes[8..].copy_from_slice(&next.to_be_bytes());
        let id = l::ProducerId::from_verified_backend(bytes)?;
        self.next = next;
        Ok(id)
    }
}
/// A bounded one-entry exact-origin map for the CURRENT accepted generation.
/// IDs may rotate on later reads of the same origin; old IDs never retarget.
#[derive(Clone)]
struct Association { origin: t::Origin, producer: l::ProducerId }

type Response<T> = oneshot::Sender<Result<T,l::Reason>>;
enum Command {
    Read { window: Stamp, force: bool, response: Response<l::Ticket> },
    Scan { window: Stamp, scope: String, intent: String, response: Response<l::Ticket> },
    EnableWifi { window: Stamp, scope: String, intent: String, response: Response<l::Ticket> },
    DisableWifi { window: Stamp, scope: String, intent: String, response: Response<l::Ticket> },
    View { window: Stamp, response: Response<l::View> },
    Ticket { window: Stamp, id: String, response: Response<l::Ticket> },
    Cancel { window: Stamp, id: String, response: Response<l::Ticket> },
    #[cfg(test)]
    Inspect { response: Response<Inspection> },
}
#[cfg(test)]
struct Inspection { association: Option<Association>, active: bool, reserved: u64 }
impl Command {
    fn abandoned(&self) -> bool {
        match self {
            Self::Read { response,.. } | Self::Scan { response,.. } | Self::EnableWifi { response,.. } | Self::DisableWifi { response,.. } |
            Self::Ticket { response,.. } | Self::Cancel { response,.. } => response.is_closed(),
            Self::View { response,.. } => response.is_closed(),
            #[cfg(test)]
            Self::Inspect { response } => response.is_closed(),
        }
    }
    fn reject(self, reason: l::Reason) {
        match self {
            Self::Read { response,.. } | Self::Scan { response,.. } | Self::EnableWifi { response,.. } | Self::DisableWifi { response,.. } |
            Self::Ticket { response,.. } | Self::Cancel { response,.. } => { let _ = response.send(Err(reason)); }
            Self::View { response,.. } => { let _ = response.send(Err(reason)); }
            #[cfg(test)]
            Self::Inspect { response } => { let _ = response.send(Err(reason)); }
        }
    }
}

/// Caller handles never own/cancel the actor or worker JoinHandle.
#[derive(Clone)]
pub(crate) struct Client { commands: mpsc::Sender<Command>, binding: Arc<AuthorityFence> }
impl Client {
    async fn submit<T>(&self, make: impl FnOnce(Response<T>) -> Command) -> Result<T,l::Reason> {
        let (response, receive) = oneshot::channel();
        self.commands.try_send(make(response)).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => l::Reason::Capacity,
            mpsc::error::TrySendError::Closed(_) => l::Reason::Closed,
        })?;
        receive.await.unwrap_or(Err(l::Reason::Closed))
    }
    pub(crate) async fn read(&self, window: &WindowRef, force: bool) -> Result<l::Ticket,l::Reason> {
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::Read {window,force,response}).await
    }
    pub(crate) async fn scan(&self, window: &WindowRef, scope: &str, intent: &str) -> Result<l::Ticket,l::Reason> {
        if scope.len()>128 { return Err(l::Reason::StaleTarget); }
        if intent.is_empty() || intent.len()>64 || !intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {
            return Err(l::Reason::InvalidIntent);
        }
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::Scan {window,scope:scope.into(),intent:intent.into(),response}).await
    }
    pub(crate) async fn enable_wifi(&self, window: &WindowRef, scope: &str, intent: &str) -> Result<l::Ticket,l::Reason> {
        if scope.len()>128 { return Err(l::Reason::StaleTarget); }
        if intent.is_empty() || intent.len()>64 || !intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {
            return Err(l::Reason::InvalidIntent);
        }
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::EnableWifi {window,scope:scope.into(),intent:intent.into(),response}).await
    }
pub(crate) async fn disable_wifi(&self, window: &WindowRef, scope: &str, intent: &str) -> Result<l::Ticket,l::Reason> {
        if scope.len()>128 { return Err(l::Reason::StaleTarget); }
        if intent.is_empty() || intent.len()>64 || !intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {
            return Err(l::Reason::InvalidIntent);
        }
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::DisableWifi {window,scope:scope.into(),intent:intent.into(),response}).await
    }
    pub(crate) async fn view(&self, window: &WindowRef) -> Result<l::View,l::Reason> {
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::View {window,response}).await
    }
    pub(crate) async fn ticket(&self, window: &WindowRef, id: &str) -> Result<l::Ticket,l::Reason> {
        if id.len()>128 { return Err(l::Reason::NotFound); }
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::Ticket {window,id:id.into(),response}).await
    }
    pub(crate) async fn cancel(&self, window: &WindowRef, id: &str) -> Result<l::Ticket,l::Reason> {
        if id.len()>128 { return Err(l::Reason::NotFound); }
        let window=window.capture(&self.binding)?;
        self.submit(|response| Command::Cancel {window,id:id.into(),response}).await
    }
}

pub(crate) struct Executor {
    client: Client,
    stop: Option<oneshot::Sender<()>>,
    actor: Option<JoinHandle<()>>,
}
impl Executor {
    pub(crate) fn start(native: t::Config, config: Config, binding: &Arc<AuthorityFence>) -> Result<Self,l::Reason> {
        if config.producer_nonce==0 || config.command_capacity==0 || config.command_capacity>128 {
            return Err(l::Reason::Capacity);
        }
        let runtime = tokio::runtime::Handle::try_current().map_err(|_| l::Reason::NativeUnavailable)?;
        let permit = Permit::claim()?;
        let service = binding.service(config.lifecycle,config.boot_nonce)?;
        let (commands,receive) = mpsc::channel(config.command_capacity);
        let (stop,shutdown) = oneshot::channel();
        let actor = runtime.spawn(actor(native,service,ProducerCounter {nonce:config.producer_nonce,next:0},receive,shutdown,permit,binding.clone()));
        Ok(Self {client:Client {commands,binding:binding.clone()},stop:Some(stop),actor:Some(actor)})
    }
    pub(crate) fn client(&self) -> Client { self.client.clone() }
    /// Close admissions/revoke Service, then wait for the actual bounded worker.
    /// No outer timeout or JoinHandle::abort is used here.
    pub(crate) async fn shutdown(mut self) {
        if let Some(stop) = self.stop.take() { let _ = stop.send(()); }
        if let Some(actor) = self.actor.take() { let _ = actor.await; }
    }
}
impl Drop for Executor {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() { let _ = stop.send(()); }
        // Dropping this handle does not abort the actor. It drains its worker
        // while their shared singleton permit prevents a replacement executor.
    }
}

#[derive(Clone,Copy,Debug)]
enum GuardReason { Fence(l::Reason), Lifecycle(l::Reason) }
enum Outcome {
    Read { result: Result<t::Reply,t::Failure<GuardReason>>, producer: l::ProducerId, observed_at_ms: Option<u64> },
    Scan { result: Result<t::ScanAck,t::Failure<GuardReason>>, association: Association },
    EnableWifi { result: Result<t::EnableWifiAck,t::Failure<GuardReason>>, association: Association },
    DisableWifi { result: Result<t::DisableWifiAck,t::Failure<GuardReason>>, association: Association },
}
struct Completed { work: l::Work, outcome: Outcome, permit: Arc<Permit>, window: Stamp }

fn epoch_milliseconds() -> Option<u64> {
    let ms = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis()).ok()?;
    (ms>0 && ms<=8_640_000_000_000_000).then_some(ms)
}
fn spawn_read(native: t::Config, mut work: l::Work, producer: l::ProducerId, permit: Arc<Permit>, window: Stamp, binding: Arc<AuthorityFence>) -> JoinHandle<Completed> {
    tokio::spawn(async move {
        let result = t::network_snapshot_guarded(&native,|| {
            window.run(&binding,|_|work.begin_dispatch(producer,Instant::now())).map_err(GuardReason::Fence)?.map_err(GuardReason::Lifecycle).and_then(|op| {
                if op==l::Operation::ReadNetwork { Ok(()) } else { Err(GuardReason::Lifecycle(l::Reason::InvalidObservation)) }
            })
        }).await;
        let observed_at_ms = if result.is_ok() { epoch_milliseconds() } else { None };
        Completed {work,outcome:Outcome::Read {result,producer,observed_at_ms},permit,window}
    })
}
fn spawn_scan(native: t::Config, mut work: l::Work, association: Association, permit: Arc<Permit>, window: Stamp, binding: Arc<AuthorityFence>) -> JoinHandle<Completed> {
    tokio::spawn(async move {
        let result = t::scan_wifi(&native,&association.origin,|fresh| {
            if fresh!=&association.origin { return Err(GuardReason::Lifecycle(l::Reason::SourceChanged)); }
            window.run(&binding,|_|work.begin_dispatch(association.producer,Instant::now())).map_err(GuardReason::Fence)?.map_err(GuardReason::Lifecycle).and_then(|op| {
                if op==l::Operation::ScanWifi { Ok(()) } else { Err(GuardReason::Lifecycle(l::Reason::InvalidObservation)) }
            })
        }).await;
        Completed {work,outcome:Outcome::Scan {result,association},permit,window}
    })
}
fn spawn_enable_wifi(native: t::Config, mut work: l::Work, association: Association, permit: Arc<Permit>, window: Stamp, binding: Arc<AuthorityFence>) -> JoinHandle<Completed> {
    tokio::spawn(async move {
        let result = t::enable_wifi(&native,&association.origin,|fresh| {
            if fresh!=&association.origin { return Err(GuardReason::Lifecycle(l::Reason::SourceChanged)); }
            window.run(&binding,|_|work.begin_dispatch(association.producer,Instant::now())).map_err(GuardReason::Fence)?.map_err(GuardReason::Lifecycle).and_then(|op| {
                if op==l::Operation::EnableWifi { Ok(()) } else { Err(GuardReason::Lifecycle(l::Reason::InvalidObservation)) }
            })
        }).await;
        Completed {work,outcome:Outcome::EnableWifi {result,association},permit,window}
    })
}
fn spawn_disable_wifi(native: t::Config, mut work: l::Work, association: Association, permit: Arc<Permit>, window: Stamp, binding: Arc<AuthorityFence>) -> JoinHandle<Completed> {
    tokio::spawn(async move {
        let result = t::disable_wifi(&native,&association.origin,|fresh| {
            if fresh!=&association.origin { return Err(GuardReason::Lifecycle(l::Reason::SourceChanged)); }
            window.run(&binding,|_|work.begin_dispatch(association.producer,Instant::now())).map_err(GuardReason::Fence)?.map_err(GuardReason::Lifecycle).and_then(|op| {
                if op==l::Operation::DisableWifi { Ok(()) } else { Err(GuardReason::Lifecycle(l::Reason::InvalidObservation)) }
            })
        }).await;
        Completed {work,outcome:Outcome::DisableWifi {result,association},permit,window}
    })
}
fn native_error(error: t::Error) -> l::NativeError {
    match error {
        t::Error::Unavailable | t::Error::Timeout => l::NativeError::Unavailable,
        t::Error::Protocol => l::NativeError::InvalidResponse,
        t::Error::UntrustedPeer | t::Error::OriginChanged | t::Error::Unsupported | t::Error::NativeFailure => l::NativeError::Rejected,
    }
}
fn failed(work: l::Work, failure: t::Failure<GuardReason>) {
    match failure {
        // Actual Work guard already settled lease/deadline/target/cancel reasons.
        // Drop preserves those terminal reasons instead of overwriting them.
        t::Failure::Guard(GuardReason::Lifecycle(_)) => drop(work),
        t::Failure::Guard(GuardReason::Fence(_)) => {let _=work.fail(l::NativeError::Rejected,Instant::now());}
        t::Failure::Native { reason,.. } => { let _ = work.fail(native_error(reason),Instant::now()); }
    }
}
/// This synchronous step is called ONLY by the actor, before any next command.
fn publish(completed: Completed, association: &mut Option<Association>, accepting: bool, binding: &Arc<AuthorityFence>) {
    let Completed {work,outcome,permit,window} = completed;
    if !accepting { drop(work);drop(permit);return; }
    let mut payload=Some((work,outcome));
    let allowed=window.run(binding,|_| {
    let (work,outcome)=payload.take().unwrap();
    match outcome {
        Outcome::Read {result,producer,observed_at_ms} => match result {
            Ok(reply) => {
                let candidate = Association {origin:reply.origin().clone(),producer};
                let support = if reply.supports_scan() {l::ScanSupport::VerifiedByAdapter} else {l::ScanSupport::Unproven};
                if let Some(observed_at_ms) = observed_at_ms {
                    let observation = l::NativeObservation {producer,api_version:reply.api_version,capabilities:reply.capabilities,result:reply.result,targets:Vec::new(),scan_support:support,observed_at_ms};
                    if work.finish_read(observation,Instant::now()).is_ok() {
                        // Cache publication succeeded, including its projection and
                        // lease/deadline/live-ticket checks. No await in between.
                        *association = Some(candidate);
                    }
                } else { let _ = work.fail(l::NativeError::InvalidResponse,Instant::now()); }
            }
            Err(error) => failed(work,error),
        },
        Outcome::Scan {result,association:expected} => match result {
            Ok(ack) if ack.origin()==&expected.origin => { let _ = work.finish_scan(expected.producer,Instant::now()); }
            Ok(_) => { let _ = work.fail(l::NativeError::InvalidResponse,Instant::now()); }
            Err(error) => failed(work,error),
        },
        Outcome::EnableWifi {result,association:expected} => match result {
            Ok(ack) if ack.origin()==&expected.origin => { let _ = work.finish_enable_wifi(expected.producer,Instant::now()); }
            Ok(_) => { let _ = work.fail(l::NativeError::InvalidResponse,Instant::now()); }
            Err(error) => failed(work,error),
        },
        Outcome::DisableWifi {result,association:expected} => match result {
            Ok(ack) if ack.origin()==&expected.origin => { let _ = work.finish_disable_wifi(expected.producer,Instant::now()); }
            Ok(_) => { let _ = work.fail(l::NativeError::InvalidResponse,Instant::now()); }
            Err(error) => failed(work,error),
        },
    }
    });
    if allowed.is_err() {if let Some((work,_))=payload.take(){let _=work.fail(l::NativeError::Rejected,Instant::now());}}
    drop(permit);
}

async fn actor(native: t::Config, service: l::Service, mut counter: ProducerCounter,
    mut commands: mpsc::Receiver<Command>, mut shutdown: oneshot::Receiver<()>, permit: Arc<Permit>, binding: Arc<AuthorityFence>) {
    let mut service = Some(service);
    let mut current: Option<Association> = None;
    let mut active: Option<JoinHandle<Completed>> = None;
    let mut accepting = true;
    loop {
        if !accepting && active.is_none() { break; }
        tokio::select! {
            biased;
            _ = &mut shutdown, if accepting => {
                accepting = false;
                commands.close();
                while let Ok(command) = commands.try_recv() {command.reject(l::Reason::Closed);}
                // Revoke pending writer guards while retaining actual worker IO
                // and its permit; native IO already started cannot be rolled back.
                drop(service.take());
                current = None;
            }
            result = async { active.as_mut().unwrap().await }, if active.is_some() => {
                active.take();
                if let Ok(completed) = result { publish(completed,&mut current,accepting,&binding); }
                // A panicked worker's Work drop already settled a static receipt;
                // never expose/log raw JoinError/panic/native payload here.
            }
            command = commands.recv(), if accepting => {
                let Some(command) = command else {
                    accepting=false;drop(service.take());current=None;continue;
                };
                if command.abandoned() { continue; }
                let service = service.as_ref().unwrap();
                match command {
                    Command::Read {window,force,response} => {
                        let result = (|| {
                            let admission=window.run(&binding,|owner|service.read(owner,force,Instant::now()))??;
                            if let Some(work)=admission.work {
                                // Also guard against an unprocessed worker panic:
                                // never replace/drop a running JoinHandle.
                                if active.is_some() { drop(work);return Err(l::Reason::Busy); }
                                let producer=counter.reserve()?;
                                active=Some(spawn_read(native.clone(),work,producer,permit.clone(),window,binding.clone()));
                            }
                            Ok(admission.ticket)
                        })();
                        let _=response.send(result);
                    }
                    Command::Scan {window,scope,intent,response} => {
                        let result = (|| {
                            let admission=window.run(&binding,|owner|service.scan(owner,&scope,&intent,Instant::now()))??;
                            if let Some(work)=admission.work {
                                if active.is_some() { drop(work);return Err(l::Reason::Busy); }
                                let Some(association)=current.clone() else {
                                    let _=work.fail(l::NativeError::InvalidResponse,Instant::now());
                                    return Err(l::Reason::InvalidObservation);
                                };
                                active=Some(spawn_scan(native.clone(),work,association,permit.clone(),window,binding.clone()));
                            }
                            Ok(admission.ticket)
                        })();
                        let _=response.send(result);
                    }
                    Command::EnableWifi {window,scope,intent,response} => {
                        let result = (|| {
                            let admission=window.run(&binding,|owner|service.enable_wifi(owner,&scope,&intent,Instant::now()))??;
                            if let Some(work)=admission.work {
                                if active.is_some() { drop(work);return Err(l::Reason::Busy); }
                                let Some(association)=current.clone() else {
                                    let _=work.fail(l::NativeError::InvalidResponse,Instant::now());
                                    return Err(l::Reason::InvalidObservation);
                                };
                                active=Some(spawn_enable_wifi(native.clone(),work,association,permit.clone(),window,binding.clone()));
                            }
                            Ok(admission.ticket)
                        })();
                        let _=response.send(result);
                    }                    Command::DisableWifi {window,scope,intent,response} => {
                        let result = (|| {
                            let admission=window.run(&binding,|owner|service.disable_wifi(owner,&scope,&intent,Instant::now()))??;
                            if let Some(work)=admission.work {
                                if active.is_some() { drop(work);return Err(l::Reason::Busy); }
                                let Some(association)=current.clone() else {
                                    let _=work.fail(l::NativeError::InvalidResponse,Instant::now());
                                    return Err(l::Reason::InvalidObservation);
                                };
                                active=Some(spawn_disable_wifi(native.clone(),work,association,permit.clone(),window,binding.clone()));
                            }
                            Ok(admission.ticket)
                        })();
                        let _=response.send(result);
                    }
                    Command::View {window,response} => {
                        let result=window.run(&binding,|owner|service.view(owner,Instant::now())).and_then(|v|v);let _=response.send(result);
                    }
                    Command::Ticket {window,id,response} => {
                        let result=window.run(&binding,|owner|service.ticket(owner,&id,Instant::now())).and_then(|v|v);let _=response.send(result);
                    }
                    Command::Cancel {window,id,response} => {
                        let result=window.run(&binding,|owner|service.cancel(owner,&id,Instant::now())).and_then(|v|v);let _=response.send(result);
                    }
                    #[cfg(test)]
                    Command::Inspect {response} => {
                        let _=response.send(Ok(Inspection {association:current.clone(),active:active.is_some(),reserved:counter.next}));
                    }
                }
            }
        }
    }
    // Permit remains owned until actor drain and any worker/output have ended.
    drop(permit);
}

#[cfg(test)]
#[path="executor_tests.rs"]
mod tests;

// NEW cumulative seam appended after the exact frozen executor R1 source.
// Native/Core resolve calls prepare synchronously; later enqueue never resamples.
pub(crate) struct Prepared { stamp: Stamp, binding: Arc<AuthorityFence> }
impl Prepared {
    /// Future strict audio adapter seam only; no audio mutation is wired here.
    pub(crate) fn validate(&self) -> Result<(),l::Reason> {
        self.stamp.run(&self.binding, |_| ())
    }
    #[cfg(test)]
    pub(crate) fn fixture_read(&self,service:&l::Service)->Result<l::Admission,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.read(owner,true,Instant::now()))?
    }
    #[cfg(test)]
    pub(crate) fn fixture_view(&self,service:&l::Service)->Result<l::View,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.view(owner,Instant::now()))?
    }
    #[cfg(test)]
    pub(crate) fn fixture_ticket(&self,service:&l::Service,id:&str)->Result<l::Ticket,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.ticket(owner,id,Instant::now()))?
    }
    #[cfg(test)]
    pub(crate) fn fixture_scan(&self,service:&l::Service,scope:&str,intent:&str)->Result<l::Admission,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.scan(owner,scope,intent,Instant::now()))?
    }
    #[cfg(test)]
    pub(crate) fn fixture_enable_wifi(&self,service:&l::Service,scope:&str,intent:&str)->Result<l::Admission,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.enable_wifi(owner,scope,intent,Instant::now()))?
    }
    #[cfg(test)]
    pub(crate) fn fixture_disable_wifi(&self,service:&l::Service,scope:&str,intent:&str)->Result<l::Admission,l::Reason>{
        self.stamp.run(&self.binding,|owner|service.disable_wifi(owner,scope,intent,Instant::now()))?
    }
}
/// Test-only inert inbox: captures real fenced request state but starts no actor,
/// singleton, Tokio worker, socket, account or native device operation.
#[cfg(test)]
pub(crate) struct FixtureInbox(mpsc::Receiver<Command>);
#[cfg(test)]
pub(crate) fn fixture_client(binding:&Arc<AuthorityFence>)->(Client,FixtureInbox){
    let(commands,receive)=mpsc::channel(8);(Client {commands,binding:binding.clone()},FixtureInbox(receive))
}
pub(crate) struct Pending<T> { response: oneshot::Receiver<Result<T,l::Reason>> }
impl<T> Pending<T> {
    pub(crate) async fn wait(self) -> Result<T,l::Reason> {
        self.response.await.unwrap_or(Err(l::Reason::Closed))
    }
}
impl Client {
    pub(crate) fn prepare(&self, window:&WindowRef) -> Result<Prepared,l::Reason> {
        Ok(Prepared { stamp:window.capture(&self.binding)?,binding:self.binding.clone() })
    }
    fn enqueue<T>(&self,make:impl FnOnce(Response<T>)->Command) -> Result<Pending<T>,l::Reason> {
        let (response,receive)=oneshot::channel();
        self.commands.try_send(make(response)).map_err(|e|match e {
            mpsc::error::TrySendError::Full(_)=>l::Reason::Capacity,
            mpsc::error::TrySendError::Closed(_)=>l::Reason::Closed,
        })?;
        Ok(Pending {response:receive})
    }
    pub(crate) fn enqueue_prepared_read(&self,prepared:Prepared,force:bool)->Result<Pending<l::Ticket>,l::Reason> {
        self.enqueue(|response|Command::Read {window:prepared.stamp,force,response})
    }
    pub(crate) fn enqueue_prepared_view(&self,prepared:Prepared)->Result<Pending<l::View>,l::Reason> {
        self.enqueue(|response|Command::View {window:prepared.stamp,response})
    }
    pub(crate) fn enqueue_prepared_scan(&self,prepared:Prepared,scope:&str,intent:&str)->Result<Pending<l::Ticket>,l::Reason> {
        if scope.len()>128{return Err(l::Reason::StaleTarget);}
        if intent.is_empty()||intent.len()>64||!intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {return Err(l::Reason::InvalidIntent);}
        self.enqueue(|response|Command::Scan {window:prepared.stamp,scope:scope.into(),intent:intent.into(),response})
    }
    pub(crate) fn enqueue_prepared_enable_wifi(&self,prepared:Prepared,scope:&str,intent:&str)->Result<Pending<l::Ticket>,l::Reason> {
        if scope.len()>128{return Err(l::Reason::StaleTarget);}
        if intent.is_empty()||intent.len()>64||!intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {return Err(l::Reason::InvalidIntent);}
        self.enqueue(|response|Command::EnableWifi {window:prepared.stamp,scope:scope.into(),intent:intent.into(),response})
    }
pub(crate) fn enqueue_prepared_disable_wifi(&self,prepared:Prepared,scope:&str,intent:&str)->Result<Pending<l::Ticket>,l::Reason> {
        if scope.len()>128{return Err(l::Reason::StaleTarget);}
        if intent.is_empty()||intent.len()>64||!intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) {return Err(l::Reason::InvalidIntent);}
        self.enqueue(|response|Command::DisableWifi {window:prepared.stamp,scope:scope.into(),intent:intent.into(),response})
    }
    pub(crate) fn enqueue_prepared_ticket(&self,prepared:Prepared,id:&str)->Result<Pending<l::Ticket>,l::Reason> {
        if id.len()>128{return Err(l::Reason::NotFound);}
        self.enqueue(|response|Command::Ticket {window:prepared.stamp,id:id.into(),response})
    }
    pub(crate) fn enqueue_prepared_cancel(&self,prepared:Prepared,id:&str)->Result<Pending<l::Ticket>,l::Reason> {
        if id.len()>128{return Err(l::Reason::NotFound);}
        self.enqueue(|response|Command::Cancel {window:prepared.stamp,id:id.into(),response})
    }
}
