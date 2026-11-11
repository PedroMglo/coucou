//! Shared client execution resources. A queue ticket is not an execution grant.
use std::{collections::{BTreeSet,HashMap,VecDeque},sync::{Arc,Mutex,Weak,atomic::{AtomicBool,AtomicU64,Ordering}},time::{Duration,Instant}};
const MAX_HOLDS:usize=64;const MAX_WAITERS:usize=256;
const MAX_INTERACTIVE_BYPASSES:u8=3;
// Mechanical freshness only; never extends a goal/lease/consent deadline.
const WAITER_FRESHNESS:Duration=Duration::from_secs(2);
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Failure{Busy,Expired,Invalid,Unavailable}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct WaitInfo{pub position:u16,pub interactive_bypasses:u8,pub max_interactive_bypasses:u8}
struct Waiter{hold:Weak<Hold>,resource:String,generation:u64,cancel:Arc<AtomicBool>,deadline:Option<Instant>,last_poll:Instant,bypasses:u8}
impl Waiter{
 fn valid(&self,now:Instant)->bool{self.hold.strong_count()>0&&!self.cancel.load(Ordering::Acquire)&&self.deadline.is_none_or(|d|now<d)&&now.checked_duration_since(self.last_poll).is_some_and(|age|age<WAITER_FRESHNESS)}
 fn matches(&self,hold:&Hold,generation:u64)->bool{std::ptr::eq(self.hold.as_ptr(),hold)&&self.generation==generation}
}
struct Occupied{hold:Weak<Hold>,provider:String}
#[derive(Default)]struct State{holds:HashMap<String,Occupied>,waiters:VecDeque<Waiter>}
impl State{
 fn prune(&mut self,now:Instant){self.holds.retain(|_,h|h.hold.strong_count()>0);self.waiters.retain(|w|w.valid(now));}
 fn head(&self,resource:&str)->Option<usize>{self.waiters.iter().position(|w|w.resource==resource)}
 fn conflicts(&self,provider:&str,resource:&str)->bool{self.holds.contains_key(resource)||self.holds.values().any(|h|h.provider==provider)}
 fn owns(&self,hold:&Hold)->bool{self.holds.get(&hold.resource).is_some_and(|h|std::ptr::eq(h.hold.as_ptr(),hold))}
 fn insert(&mut self,hold:&Arc<Hold>){self.holds.insert(hold.resource.clone(),Occupied{hold:Arc::downgrade(hold),provider:hold.provider.clone()});}
 fn wait_info(&self,hold:&Hold,now:Instant)->Option<WaitInfo>{
  let mut position=0;for w in &self.waiters{if w.resource==hold.resource&&w.valid(now){position+=1;if w.matches(hold,hold.generation()){return Some(WaitInfo{position,interactive_bypasses:w.bypasses,max_interactive_bypasses:MAX_INTERACTIVE_BYPASSES});}}}None
 }
 // Private mechanical transition. Production enters only after validating the
 // original opaque NativeYieldProof, not from JSON or a renderer priority.
 #[cfg(any(test,target_os="linux"))]
 fn reacquire(&mut self,hold:&Arc<Hold>,generation:u64,next:u64,now:Instant)->Result<(),Failure>{
  self.prune(now);
  if hold.cancel.load(Ordering::Acquire)||hold.deadline.is_some_and(|d|now>=d){return Err(Failure::Expired);}
  if self.owns(hold){return if hold.generation()==next{Ok(())}else{Err(Failure::Invalid)};}
  if hold.generation()!=generation||generation.checked_add(1)!=Some(next){return Err(Failure::Invalid);}
  let index=if let Some(i)=self.waiters.iter().position(|w|w.matches(hold,generation)){self.waiters[i].last_poll=now;i}else{
   if self.waiters.len()>=MAX_WAITERS{return Err(Failure::Unavailable);}
   self.waiters.push_back(Waiter{hold:Arc::downgrade(hold),resource:hold.resource.clone(),generation,cancel:hold.cancel.clone(),deadline:hold.deadline,last_poll:now,bypasses:0});self.waiters.len()-1
  };
  if self.conflicts(&hold.provider,&hold.resource)||self.head(&hold.resource)!=Some(index){return Err(Failure::Busy);}
  if self.holds.len()>=MAX_HOLDS{return Err(Failure::Unavailable);}
  self.waiters.remove(index);hold.generation.store(next,Ordering::Release);self.insert(hold);Ok(())
 }
}
#[derive(Default)]pub(crate)struct Coordinator{state:Mutex<State>,settings:Option<Arc<Mutex<crate::settings::Settings>>>,#[cfg(test)]fixture_profile:Option<std::path::PathBuf>}
pub(crate)struct Hold{coordinator:Arc<Coordinator>,provider:String,resource:String,captured:Option<Arc<crate::pet_inference::Captured>>,cancel:Arc<AtomicBool>,deadline:Option<Instant>,generation:AtomicU64}
/// Mechanical initial wait only; the runtime still checks signed admission.
#[cfg(target_os="linux")]#[derive(Clone)]pub(crate)struct GoalRequest{hold:Arc<Hold>}
#[cfg(target_os="linux")]impl GoalRequest{
 pub(crate)fn poll(&self)->Result<Arc<Hold>,Failure>{
  {let mut state=self.hold.coordinator.state.lock().map_err(|_|Failure::Unavailable)?;state.reacquire(&self.hold,0,1,Instant::now())?;}Ok(self.hold.clone())
 }
 pub(crate)fn cancel(&self){self.hold.cancel();}
 pub(crate)fn stopped(&self)->bool{self.hold.stopped()}
 pub(crate)fn waiting_info(&self)->Option<WaitInfo>{self.hold.waiting_info()}
}
impl Coordinator{
 pub(crate)fn with_settings(settings:Arc<Mutex<crate::settings::Settings>>)->Self{Self{settings:Some(settings),..Self::default()}}
 #[cfg(test)]pub(crate)fn with_fixture(settings:Arc<Mutex<crate::settings::Settings>>,profile:std::path::PathBuf)->Self{Self{settings:Some(settings),fixture_profile:Some(profile),..Self::default()}}
 fn resolve(&self,provider:&str)->Result<(String,Option<Arc<crate::pet_inference::Captured>>),Failure>{
  #[cfg(test)]if let(Some(settings),Some(profile))=(&self.settings,&self.fixture_profile){
   let captured=crate::pet_inference::Captured::capture_fixture(settings,provider,profile).map_err(|_|Failure::Unavailable)?;
   let resource=captured.as_ref().map(|c|c.resource_key()).transpose().map_err(|_|Failure::Invalid)?.unwrap_or_else(||provider.into());return Ok((resource,captured));
  }
  let captured=self.settings.as_ref().map(|s|crate::pet_inference::Captured::capture(s,provider)).transpose().map_err(|_|Failure::Unavailable)?.flatten();
  let resource=captured.as_ref().map(|c|c.resource_key()).transpose().map_err(|_|Failure::Invalid)?.unwrap_or_else(||provider.into());Ok((resource,captured))
 }
 pub(crate)fn acquire(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Option<Instant>)->Result<Arc<Hold>,Failure>{self.acquire_class(provider,cancel,deadline,true)}
 #[cfg(target_os="linux")]
 pub(crate)fn acquire_goal(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Option<Instant>)->Result<Arc<Hold>,Failure>{self.acquire_class(provider,cancel,deadline,false)}
 #[cfg(target_os="linux")]
 pub(crate)fn request_goal(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Instant)->Result<GoalRequest,Failure>{
  if provider.is_empty()||provider.len()>128||!provider.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err(Failure::Invalid);}
  if cancel.load(Ordering::Acquire)||Instant::now()>=deadline{return Err(Failure::Expired);}
  let(resource,captured)=self.resolve(provider)?;
  let request=GoalRequest{hold:Arc::new(Hold{coordinator:self.clone(),provider:provider.into(),resource,captured,cancel,deadline:Some(deadline),generation:AtomicU64::new(0)})};
  // The guard must drop before the last native Hold on an error path.
  let attempt={let mut state=self.state.lock().map_err(|_|Failure::Unavailable)?;state.reacquire(&request.hold,0,1,Instant::now())};
  match attempt{Ok(())|Err(Failure::Busy)=>Ok(request),Err(error)=>Err(error)}
 }
 fn acquire_class(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Option<Instant>,interactive:bool)->Result<Arc<Hold>,Failure>{
  if provider.is_empty()||provider.len()>128||!provider.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err(Failure::Invalid);}
  let now=Instant::now();if cancel.load(Ordering::Acquire)||deadline.is_some_and(|d|now>=d){cancel.store(true,Ordering::Release);return Err(Failure::Expired);}
  let(resource,captured)=self.resolve(provider)?;
  self.acquire_resolved(provider,cancel,deadline,interactive,resource,captured)
 }
 #[cfg(target_os="linux")]
 pub(crate)fn acquire_runtime_goal(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Instant)->Result<Arc<Hold>,Failure>{
  if provider.is_empty()||provider.len()>128||!provider.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err(Failure::Invalid);}
  self.acquire_resolved(provider,cancel,Some(deadline),false,format!("runtime-goal:{provider}"),None)
 }
 #[cfg(target_os="linux")]
 pub(crate)fn request_runtime_goal(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Instant)->Result<GoalRequest,Failure>{
  if provider.is_empty()||provider.len()>128||!provider.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err(Failure::Invalid);}
  if cancel.load(Ordering::Acquire)||Instant::now()>=deadline{return Err(Failure::Expired);}
  let request=GoalRequest{hold:Arc::new(Hold{coordinator:self.clone(),provider:provider.into(),resource:format!("runtime-goal:{provider}"),captured:None,cancel,deadline:Some(deadline),generation:AtomicU64::new(0)})};
  let attempt={let mut state=self.state.lock().map_err(|_|Failure::Unavailable)?;state.reacquire(&request.hold,0,1,Instant::now())};
  match attempt{Ok(())|Err(Failure::Busy)=>Ok(request),Err(error)=>Err(error)}
 }
 fn acquire_resolved(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Option<Instant>,interactive:bool,resource:String,captured:Option<Arc<crate::pet_inference::Captured>>)->Result<Arc<Hold>,Failure>{
  let now=Instant::now();if cancel.load(Ordering::Acquire)||deadline.is_some_and(|d|now>=d){cancel.store(true,Ordering::Release);return Err(Failure::Expired);}
  let mut state=self.state.lock().map_err(|_|Failure::Unavailable)?;state.prune(now);
  // Never upgrade/drop the last Arc<Hold> under this mutex: Drop also locks it.
  if state.conflicts(provider,&resource){return Err(Failure::Busy);}
  if let Some(i)=state.head(&resource){if !interactive||state.waiters[i].bypasses>=MAX_INTERACTIVE_BYPASSES{return Err(Failure::Busy);}}
  if state.holds.len()>=MAX_HOLDS{return Err(Failure::Unavailable);}
  let hold=Arc::new(Hold{coordinator:self.clone(),provider:provider.into(),resource,captured,cancel,deadline,generation:AtomicU64::new(1)});
  if let Some(i)=state.head(&hold.resource){state.waiters[i].bypasses+=1;}
  state.insert(&hold);Ok(hold)
 }
 pub(crate)fn busy(&self)->Result<BTreeSet<String>,Failure>{let state=self.state.lock().map_err(|_|Failure::Unavailable)?;Ok(state.holds.values().filter(|h|h.hold.strong_count()>0).map(|h|h.provider.clone()).collect())}
 pub(crate)fn busy_with_config(&self,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings)->Result<BTreeSet<String>,Failure>{
  let(mut busy,resources)={let state=self.state.lock().map_err(|_|Failure::Unavailable)?;
   (state.holds.values().filter(|h|h.hold.strong_count()>0).map(|h|h.provider.clone()).collect::<BTreeSet<_>>(),state.holds.iter().filter(|(_,h)|h.hold.strong_count()>0).map(|(key,_)|key.clone()).collect::<BTreeSet<_>>())};
  for blueprint in crate::pet_config::blueprints(){if let Some(provider)=&blueprint.routing.task_provider{
   if crate::pet_inference::resource_for(config,settings,provider).ok().flatten().is_some_and(|key|resources.contains(&key)){busy.insert(provider.clone());}
  }}Ok(busy)
 }
}
impl Hold{
 pub(crate)fn captured(&self)->Option<Arc<crate::pet_inference::Captured>>{self.captured.clone()}
 #[cfg(target_os="linux")]pub(crate)fn bounded_deadline(&self,end:Instant)->Option<Instant>{self.deadline.map(|deadline|deadline.min(end))}
 pub(crate)fn generation(&self)->u64{self.generation.load(Ordering::Acquire)}
 #[cfg(target_os="linux")]
 pub(crate)fn waiting_info(&self)->Option<WaitInfo>{self.coordinator.state.lock().ok()?.wait_info(self,Instant::now())}
 #[cfg(target_os="linux")]
 pub(crate)fn release_waiting(&self,proof:&crate::pet_runtime::NativeYieldProof)->Result<(),Failure>{
  let mut state=self.coordinator.state.lock().map_err(|_|Failure::Unavailable)?;if !proof.covers_release(self){return Err(Failure::Invalid);}
  if state.owns(self){state.holds.remove(&self.resource);}Ok(())
 }
 #[cfg(target_os="linux")]
 pub(crate)fn reacquire_waiting(self:&Arc<Self>,proof:&crate::pet_runtime::NativeYieldProof)->Result<(),Failure>{
  let mut state=self.coordinator.state.lock().map_err(|_|Failure::Unavailable)?;
  if !proof.covers_reacquire(self){return Err(Failure::Invalid);}
  state.reacquire(self,proof.generation(),proof.next_generation(),Instant::now())
 }
 /// Cancellation never releases a live slot. Only native drain can do that.
 #[cfg(target_os="linux")]
 pub(crate)fn release_drained(&self,proof:&crate::pet_runtime::NativeDrainProof)->Result<(),Failure>{
  if !proof.covers_drained_hold(self){return Err(Failure::Invalid);}let mut state=self.coordinator.state.lock().map_err(|_|Failure::Unavailable)?;
  state.waiters.retain(|w|!std::ptr::eq(w.hold.as_ptr(),self));if state.owns(self){state.holds.remove(&self.resource);}Ok(())
 }
 pub(crate)fn cancel(&self){self.cancel.store(true,Ordering::Release);if let Ok(mut state)=self.coordinator.state.try_lock(){state.waiters.retain(|w|!std::ptr::eq(w.hold.as_ptr(),self));}}
 pub(crate)fn stopped(&self)->bool{self.cancel.load(Ordering::Acquire)||self.deadline.is_some_and(|d|Instant::now()>=d)}
 pub(crate)fn occupies_slot(&self)->bool{self.coordinator.state.lock().is_ok_and(|s|s.owns(self))}
 pub(crate)fn provider_for<'a>(&'a self,coordinator:&Arc<Coordinator>)->Option<&'a str>{(Arc::ptr_eq(&self.coordinator,coordinator)&&self.current()).then_some(self.provider.as_str())}
 pub(crate)fn current(&self)->bool{self.current_at(Instant::now())}
 fn current_at(&self,now:Instant)->bool{
  if self.cancel.load(Ordering::Acquire)||self.deadline.is_some_and(|d|now>=d){self.cancel.store(true,Ordering::Release);return false;}
  self.coordinator.state.lock().is_ok_and(|s|s.owns(self))
 }
}
impl Drop for Hold{fn drop(&mut self){if let Ok(mut state)=self.coordinator.state.lock(){state.waiters.retain(|w|!std::ptr::eq(w.hold.as_ptr(),self));if state.owns(self){state.holds.remove(&self.resource);}}}}
#[cfg(all(test,target_os="linux"))]#[path="pet_resources_arbitration_tests.rs"]mod arbitration_tests;
#[cfg(all(test,target_os="linux"))]#[path="pet_resources_inference_tests.rs"]mod inference_tests;
#[cfg(test)]mod tests{
    use super::*;use std::time::Duration;
    fn flag()->Arc<AtomicBool>{Arc::new(AtomicBool::new(false))}
    #[test]fn same_pet_cannot_acquire_two_physical_holds(){let c=Arc::new(Coordinator::default());let h=c.acquire("codex",flag(),None).unwrap();assert!(matches!(c.acquire("codex",flag(),None),Err(Failure::Busy)));drop(h);assert!(c.acquire("codex",flag(),None).is_ok());}
    #[test]fn independent_pet_and_service_owners_do_not_block_each_other(){let c=Arc::new(Coordinator::default());let holds:Vec<_>=["codex","copilot","antigravity","ollama","integration_notion"].into_iter().map(|p|c.acquire(p,flag(),None).unwrap()).collect();assert_eq!(c.busy().unwrap().len(),5);drop(holds);assert!(c.busy().unwrap().is_empty());}
    #[test]fn cancelled_worker_keeps_its_slot_until_the_last_hold_drops(){let c=Arc::new(Coordinator::default());let cancel=flag();let h=c.acquire("codex",cancel.clone(),None).unwrap();let worker=h.clone();cancel.store(true,Ordering::Release);assert!(!h.current());drop(h);assert!(matches!(c.acquire("codex",flag(),None),Err(Failure::Busy)));drop(worker);assert!(c.acquire("codex",flag(),None).is_ok());}
    #[test]fn expiration_cancels_future_operations_without_reusing_a_live_slot(){let c=Arc::new(Coordinator::default());let cancel=flag();let end=Instant::now()+Duration::from_secs(30);let h=c.acquire("ollama",cancel.clone(),Some(end)).unwrap();assert!(!h.current_at(end));assert!(cancel.load(Ordering::Acquire));assert!(matches!(c.acquire("ollama",flag(),None),Err(Failure::Busy)));drop(h);assert!(c.acquire("ollama",flag(),None).is_ok());}
    #[test]fn an_expired_queued_request_never_reserves_a_slot(){let c=Arc::new(Coordinator::default());assert!(matches!(c.acquire("copilot",flag(),Some(Instant::now())),Err(Failure::Expired)));assert!(c.busy().unwrap().is_empty());}
    #[test]fn invalid_or_cancelled_requests_are_rejected_without_new_resources(){let c=Arc::new(Coordinator::default());for p in ["","../codex","pet name"]{assert!(matches!(c.acquire(p,flag(),None),Err(Failure::Invalid)));}assert!(matches!(c.acquire("codex",Arc::new(AtomicBool::new(true)),None),Err(Failure::Expired)));assert!(c.busy().unwrap().is_empty());}
}
