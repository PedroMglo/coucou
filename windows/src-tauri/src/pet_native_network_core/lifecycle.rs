//! Private coordinator. Produces typed work, never executes a native operation.
//! Native window/producer witnesses must come from reviewed backend adapters.
use serde::Serialize;
use serde_json::Value;
use std::{collections::{BTreeMap,BTreeSet},sync::{Arc,Mutex},time::{Duration,Instant}};

#[path="network.rs"]
mod projection;
#[path="enable_support.rs"]
mod enable_support;
pub(crate) fn wifi_protocol_supported(api:Option<u64>,caps:&[String])->bool{enable_support::protocol_supported(api,caps)}
pub(crate) fn enable_wifi_supported(value:&Value,api:Option<u64>,caps:&[String])->bool{enable_support::supported(value,api,caps)}
pub(crate) fn disable_wifi_supported(value:&Value,api:Option<u64>,caps:&[String])->bool{enable_support::supported(value,api,caps)}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum Reason {Unauthorized,Hidden,SystemDisabled,Closed,Busy,Capacity,InvalidIntent,IntentConflict,NotFound,NotObserved,CacheExpired,StaleTarget,WrongTarget,SourceChanged,Deadline,Revoked,CancelledBeforeDispatch,CancelledAfterDispatch,AbandonedBeforeDispatch,UnknownAfterDispatch,AlreadyDispatched,InvalidObservation,NativeUnavailable,NativeRejected,Unsupported}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum WindowKind {System,Other}
struct Setting {enabled:bool,epoch:u64}
#[derive(Clone)]
pub struct SystemSetting(Arc<Mutex<Setting>>);
impl SystemSetting {
 pub fn from_backend(enabled:bool)->Self{Self(Arc::new(Mutex::new(Setting{enabled,epoch:1})))}
 pub fn update_from_backend(&self,enabled:bool){let mut s=self.0.lock().unwrap();if s.enabled!=enabled{s.enabled=enabled;if let Some(n)=s.epoch.checked_add(1){s.epoch=n;}else{s.enabled=false;}}}
}
struct Window {kind:WindowKind,visible:bool,alive:bool,epoch:u64}
/// Not Deserialize/Clone. Dropping the native owner's wrapper revokes leases.
pub struct SystemWindowAuthority {window:Arc<Mutex<Window>>,setting:SystemSetting}
impl SystemWindowAuthority {
 /// Only a native adapter may call this after checking the actual window
 /// pointer/incarnation and backend-owned System presentation. Never JSON facts.
 pub fn from_backend(kind:WindowKind,visible:bool,setting:&SystemSetting)->Self{Self{window:Arc::new(Mutex::new(Window{kind,visible,alive:true,epoch:1})),setting:setting.clone()}}
 pub fn visibility_from_backend(&self,visible:bool){let mut w=self.window.lock().unwrap();if w.visible!=visible{w.visible=visible;if let Some(n)=w.epoch.checked_add(1){w.epoch=n;}else{w.alive=false;}}}
 /// Changing the actual native presentation revokes previously issued leases,
 /// including a System -> Other -> System round trip while still visible.
 pub fn presentation_from_backend(&self,kind:WindowKind){let mut w=self.window.lock().unwrap();if w.kind!=kind{w.kind=kind;if let Some(n)=w.epoch.checked_add(1){w.epoch=n;}else{w.alive=false;}}}
 pub fn close_from_backend(&self){self.window.lock().unwrap().alive=false;}
 fn lease(&self)->Result<Lease,Reason>{
  let w=self.window.lock().unwrap();let s=self.setting.0.lock().unwrap();
  if !w.alive{return Err(Reason::Closed);}if w.kind!=WindowKind::System{return Err(Reason::Unauthorized);}if !w.visible{return Err(Reason::Hidden);}if !s.enabled{return Err(Reason::SystemDisabled);}
  Ok(Lease{window:self.window.clone(),setting:self.setting.clone(),window_epoch:w.epoch,setting_epoch:s.epoch})
 }
}
impl Drop for SystemWindowAuthority{fn drop(&mut self){self.close_from_backend();}}
#[derive(Clone)]
struct Lease {window:Arc<Mutex<Window>>,setting:SystemSetting,window_epoch:u64,setting_epoch:u64}
impl Lease {
 fn valid(&self)->bool{let w=self.window.lock().unwrap();let s=self.setting.0.lock().unwrap();w.alive&&w.kind==WindowKind::System&&w.visible&&w.epoch==self.window_epoch&&s.enabled&&s.epoch==self.setting_epoch}
 fn same_window(&self,other:&Self)->bool{Arc::ptr_eq(&self.window,&other.window)}
}
#[derive(Clone,Copy,PartialEq,Eq)]
pub struct ProducerId([u8;16]);
impl ProducerId {pub fn from_verified_backend(value:[u8;16])->Result<Self,Reason>{if value==[0;16]{Err(Reason::InvalidObservation)}else{Ok(Self(value))}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize)]
#[serde(rename_all="camelCase")]
pub enum TargetKind {WifiDevice,AccessPoint,VpnProfile}
/// Private identities are deliberately neither Debug nor Serialize.
#[derive(Clone)]
pub struct NativeTarget {kind:TargetKind,key:String,label:String}
impl NativeTarget {
 pub fn from_verified_backend(kind:TargetKind,key:&str,label:&str)->Result<Self,Reason>{
  let valid=match kind{TargetKind::WifiDevice=>numeric_path(key,"/org/freedesktop/NetworkManager/Devices/"),TargetKind::AccessPoint=>numeric_path(key,"/org/freedesktop/NetworkManager/AccessPoint/"),TargetKind::VpnProfile=>uuid(key)};
  if !valid||label.is_empty()||label.len()>128||label.chars().any(|c|c.is_control()||matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')){return Err(Reason::InvalidObservation);}
  Ok(Self{kind,key:if kind==TargetKind::VpnProfile{key.to_ascii_lowercase()}else{key.into()},label:label.into()})
 }
}
fn numeric_path(value:&str,prefix:&str)->bool{value.len()<=256&&value.strip_prefix(prefix).is_some_and(|v|!v.is_empty()&&v.bytes().all(|b|b.is_ascii_digit()))}
fn uuid(value:&str)->bool{value.len()==36&&value.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_hexdigit()})&&value.bytes().any(|b|b.is_ascii_hexdigit()&&b!=b'0')}
#[derive(Clone,Copy,PartialEq,Eq)]
pub enum ScanSupport {Unproven,VerifiedByAdapter}
/// A trusted adapter must bind these fields to ONE authenticated native reply.
/// Raw payload and identities never leave this backend coordinator.
pub struct NativeObservation {pub producer:ProducerId,pub api_version:Option<u64>,pub capabilities:Vec<String>,pub result:Value,pub targets:Vec<NativeTarget>,pub scan_support:ScanSupport,pub observed_at_ms:u64}
#[derive(Clone)]
pub struct Config {pub cache_age:Duration,pub target_ttl:Duration,pub operation_timeout:Duration,pub ticket_retention:Duration,pub max_tickets:usize,pub max_bindings:usize,pub max_targets:usize}
impl Default for Config {fn default()->Self{Self{cache_age:Duration::from_secs(15),target_ttl:Duration::from_secs(15),operation_timeout:Duration::from_secs(3),ticket_retention:Duration::from_secs(120),max_tickets:128,max_bindings:256,max_targets:64}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum Kind {Read,Scan,EnableWifi,DisableWifi}
impl Kind {
 fn mutating(self)->bool{match self{Self::Read=>false,Self::Scan|Self::EnableWifi|Self::DisableWifi=>true}}
 fn operation(self)->Operation{match self{Self::Read=>Operation::ReadNetwork,Self::Scan=>Operation::ScanWifi,Self::EnableWifi=>Operation::EnableWifi,Self::DisableWifi=>Operation::DisableWifi}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum Phase {Queued,Running,Observed,Acknowledged,Cancelled,Unknown,Failed}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Ticket {pub id:String,pub kind:Kind,pub phase:Phase,pub reason:Option<Reason>,pub effect_confirmed:bool,pub refresh_recommended:bool}
#[derive(Clone,Serialize)]
#[serde(transparent)]
pub struct Projected(Value);
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct TargetView {pub handle:String,pub kind:TargetKind,pub label:String}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Capabilities {pub read:bool,pub scan:bool,pub enable_wifi:bool,pub disable_wifi:bool,pub connect:bool,pub radio:bool,pub credentials:bool}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct View {pub schema_version:u8,pub source:&'static str,pub generation:Option<String>,pub stale:bool,pub refreshing:bool,pub data:Option<Projected>,pub scan_scope:Option<String>,pub enable_wifi_scope:Option<String>,pub disable_wifi_scope:Option<String>,pub targets:Vec<TargetView>,pub capabilities:Capabilities,pub reason:Option<Reason>}
struct TicketRecord {ticket:Ticket,owner:Lease,intent:Option<String>,scope:Option<String>,created:Instant}
struct Cached {data:Projected,producer:ProducerId,at:Instant,setting_epoch:u64,scan_support:ScanSupport,enable_wifi_support:bool,disable_wifi_support:bool,targets:Vec<NativeTarget>,stale:bool,reason:Option<Reason>}
#[derive(Clone,PartialEq,Eq)]
enum BindingKind {Scope,EnableWifi,DisableWifi,Target(TargetKind,String)}
struct Binding {owner:Lease,producer:ProducerId,generation:u64,expires:Instant,kind:BindingKind}
struct Flight {id:String,kind:Kind,owner:Lease,deadline:Instant,started:bool,peer:Option<ProducerId>,tickets:Vec<String>}
struct State {alive:bool,nonce:u128,next:u64,generation:u64,setting:SystemSetting,config:Config,cache:Option<Cached>,bindings:BTreeMap<String,Binding>,tickets:BTreeMap<String,TicketRecord>,flight:Option<Flight>}
pub struct Service {state:Arc<Mutex<State>>}
pub struct Admission {pub ticket:Ticket,pub work:Option<Work>}
/// Move into the ACTUAL executor, not an outer timeout future. No Clone.
/// Dropping this while detached IO still runs violates the adapter contract.
pub struct Work {state:Arc<Mutex<State>>,id:String,owner:Lease,deadline:Instant,kind:Kind,expected:Option<ProducerId>,source_generation:Option<u64>,target_deadline:Option<Instant>,done:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Operation {ReadNetwork,ScanWifi,EnableWifi,DisableWifi}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum NativeError {Unavailable,Rejected,InvalidResponse}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum FutureOperation {ConnectWifi,SetRadio,SubmitCredentials}
impl State {
 fn authorize(&self,lease:&Lease)->Result<(),Reason>{if !self.alive{return Err(Reason::Closed);}if !Arc::ptr_eq(&self.setting.0,&lease.setting.0)||!lease.valid(){return Err(Reason::Unauthorized);}Ok(())}
 fn id(&mut self,prefix:&str)->Result<String,Reason>{self.next=self.next.checked_add(1).ok_or(Reason::Capacity)?;Ok(format!("{prefix}-{:032x}-{:016x}",self.nonce,self.next))}
 fn fresh(&self,lease:&Lease,now:Instant)->bool{self.cache.as_ref().is_some_and(|c|!c.stale&&c.setting_epoch==lease.setting_epoch&&now.saturating_duration_since(c.at)<self.config.cache_age)}
 fn stale(&mut self,reason:Reason){if let Some(c)=&mut self.cache{c.stale=true;c.reason=Some(reason);}self.bindings.clear();}
 fn sweep(&mut self,now:Instant){
  if let Some(f)=&self.flight{let reason=if !f.owner.valid(){Some(Reason::Revoked)}else if now>=f.deadline{Some(Reason::Deadline)}else{None};if let Some(reason)=reason{let ids=f.tickets.clone();for id in ids{if let Some(t)=self.tickets.get_mut(&id){if matches!(t.ticket.phase,Phase::Queued|Phase::Running){t.ticket.refresh_recommended=t.ticket.kind.mutating()&&t.ticket.phase==Phase::Running;t.ticket.phase=if t.ticket.phase==Phase::Running{Phase::Unknown}else{Phase::Failed};t.ticket.reason=Some(reason);}}}self.stale(reason);}}
  for t in self.tickets.values_mut(){if matches!(t.ticket.phase,Phase::Queued|Phase::Running)&&!t.owner.valid(){t.ticket.refresh_recommended=t.ticket.kind.mutating()&&t.ticket.phase==Phase::Running;t.ticket.phase=if t.ticket.phase==Phase::Running{Phase::Unknown}else{Phase::Failed};t.ticket.reason=Some(Reason::Revoked);}}
  // Keep every receipt referenced by the physical worker. Otherwise cancelling
  // and evicting fast joins could grow Flight.tickets beyond max_tickets.
  let held:BTreeSet<_>=self.flight.as_ref().map(|f|f.tickets.iter().cloned().collect()).unwrap_or_default();
  let retention=self.config.ticket_retention;self.tickets.retain(|id,t|held.contains(id)||matches!(t.ticket.phase,Phase::Queued|Phase::Running)||now.saturating_duration_since(t.created)<retention);
  let generation=self.generation;self.bindings.retain(|_,b|b.generation==generation&&now<b.expires&&b.owner.valid());
 }
 fn ticket(&mut self,owner:Lease,kind:Kind,phase:Phase,intent:Option<String>,scope:Option<String>,now:Instant)->Result<Ticket,Reason>{
  if self.tickets.len()>=self.config.max_tickets{return Err(Reason::Capacity);}let id=self.id("req")?;let ticket=Ticket{id:id.clone(),kind,phase,reason:None,effect_confirmed:false,refresh_recommended:false};self.tickets.insert(id,TicketRecord{ticket:ticket.clone(),owner,intent,scope,created:now});Ok(ticket)
 }
 fn bound_id(&self,lease:&Lease,kind:&BindingKind,producer:ProducerId,now:Instant)->Option<String>{self.bindings.iter().find_map(|(id,b)|(b.owner.same_window(lease)&&b.owner.window_epoch==lease.window_epoch&&b.owner.setting_epoch==lease.setting_epoch&&b.generation==self.generation&&&b.kind==kind&&b.producer==producer&&now<b.expires).then(||id.clone()))}
 fn bind(&mut self,lease:&Lease,kind:BindingKind,now:Instant)->Result<String,Reason>{
  let c=self.cache.as_ref().ok_or(Reason::NotObserved)?;let producer=c.producer;let expires=c.at+self.config.target_ttl;
  if now>=expires{return Err(Reason::StaleTarget);}if let Some(id)=self.bound_id(lease,&kind,producer,now){return Ok(id);}
  if self.bindings.len()>=self.config.max_bindings{return Err(Reason::Capacity);}let id=self.id("target")?;self.bindings.insert(id.clone(),Binding{owner:lease.clone(),producer,generation:self.generation,expires,kind});Ok(id)
 }
}
impl Service {
 /// The backend must supply a nonzero unpredictable boot nonce, never a UI value.
 pub fn new(config:Config,boot_nonce:u128,setting:&SystemSetting)->Result<Self,Reason>{
  if boot_nonce==0||config.operation_timeout.is_zero()||config.operation_timeout>Duration::from_secs(30)||config.cache_age.is_zero()||config.cache_age>Duration::from_secs(120)||config.target_ttl.is_zero()||config.target_ttl>config.ticket_retention||config.ticket_retention>Duration::from_secs(600)||config.max_targets>256||config.max_tickets==0||config.max_tickets>512||config.max_bindings==0||config.max_bindings>1024{return Err(Reason::Capacity);}
  Ok(Self{state:Arc::new(Mutex::new(State{alive:true,nonce:boot_nonce,next:0,generation:0,setting:setting.clone(),config,cache:None,bindings:BTreeMap::new(),tickets:BTreeMap::new(),flight:None}))})
 }
 pub fn read(&self,window:&SystemWindowAuthority,force:bool,now:Instant)->Result<Admission,Reason>{
  let lease=window.lease()?;let mut s=self.state.lock().unwrap();s.authorize(&lease)?;s.sweep(now);
  if !force&&s.fresh(&lease,now){let ticket=s.ticket(lease,Kind::Read,Phase::Observed,None,None,now)?;return Ok(Admission{ticket,work:None});}
  if let Some(f)=&s.flight{if f.kind!=Kind::Read||now>=f.deadline||!f.owner.valid(){return Err(Reason::Busy);}let id=f.id.clone();let phase=if f.started{Phase::Running}else{Phase::Queued};let ticket=s.ticket(lease,Kind::Read,phase,None,None,now)?;s.flight.as_mut().filter(|f|f.id==id).unwrap().tickets.push(ticket.id.clone());return Ok(Admission{ticket,work:None});}
  let ticket=s.ticket(lease.clone(),Kind::Read,Phase::Queued,None,None,now)?;let deadline=now+s.config.operation_timeout;s.flight=Some(Flight{id:ticket.id.clone(),kind:Kind::Read,owner:lease.clone(),deadline,started:false,peer:None,tickets:vec![ticket.id.clone()]});
  Ok(Admission{work:Some(Work{state:self.state.clone(),id:ticket.id.clone(),owner:lease,deadline,kind:Kind::Read,expected:None,source_generation:None,target_deadline:None,done:false}),ticket})
 }
 pub fn scan(&self,window:&SystemWindowAuthority,scope:&str,intent:&str,now:Instant)->Result<Admission,Reason>{
  self.mutate(window,Kind::Scan,scope,intent,now)
 }
 pub fn enable_wifi(&self,window:&SystemWindowAuthority,scope:&str,intent:&str,now:Instant)->Result<Admission,Reason>{
  self.mutate(window,Kind::EnableWifi,scope,intent,now)
 }
pub fn disable_wifi(&self,window:&SystemWindowAuthority,scope:&str,intent:&str,now:Instant)->Result<Admission,Reason>{
  self.mutate(window,Kind::DisableWifi,scope,intent,now)
 }
 fn mutate(&self,window:&SystemWindowAuthority,kind:Kind,scope:&str,intent:&str,now:Instant)->Result<Admission,Reason>{
  let binding_kind=match kind{Kind::Scan=>BindingKind::Scope,Kind::EnableWifi=>BindingKind::EnableWifi,Kind::DisableWifi=>BindingKind::DisableWifi,Kind::Read=>return Err(Reason::Unsupported)};
  if scope.len()>128{return Err(Reason::StaleTarget);}if intent.is_empty()||intent.len()>64||!intent.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')){return Err(Reason::InvalidIntent);}
  let lease=window.lease()?;let mut s=self.state.lock().unwrap();s.authorize(&lease)?;s.sweep(now);
  if let Some(t)=s.tickets.values().find(|t|t.owner.same_window(&lease)&&t.intent.as_deref()==Some(intent)){
   if t.ticket.kind!=kind||t.scope.as_deref()!=Some(scope){return Err(Reason::IntentConflict);}return Ok(Admission{ticket:t.ticket.clone(),work:None});
  }
  if s.flight.is_some(){return Err(Reason::Busy);}if !s.fresh(&lease,now){return Err(Reason::StaleTarget);}
  let binding=s.bindings.get(scope).ok_or(Reason::StaleTarget)?;
  if binding.kind!=binding_kind{return Err(Reason::WrongTarget);}if !binding.owner.same_window(&lease)||!binding.owner.valid()||binding.generation!=s.generation||now>=binding.expires{return Err(Reason::StaleTarget);}
  let producer=binding.producer;let source_generation=binding.generation;let target_deadline=binding.expires;
  let supported=s.cache.as_ref().is_some_and(|c|c.producer==producer&&match kind{Kind::Scan=>c.scan_support==ScanSupport::VerifiedByAdapter,Kind::EnableWifi=>c.enable_wifi_support,Kind::DisableWifi=>c.disable_wifi_support,Kind::Read=>false});
  if !supported{return Err(Reason::Unsupported);}
  let ticket=s.ticket(lease.clone(),kind,Phase::Queued,Some(intent.into()),Some(scope.into()),now)?;
  let deadline=now+s.config.operation_timeout;s.flight=Some(Flight{id:ticket.id.clone(),kind,owner:lease.clone(),deadline,started:false,peer:None,tickets:vec![ticket.id.clone()]});
  Ok(Admission{work:Some(Work{state:self.state.clone(),id:ticket.id.clone(),owner:lease,deadline,kind,expected:Some(producer),source_generation:Some(source_generation),target_deadline:Some(target_deadline),done:false}),ticket})
 }
 pub fn view(&self,window:&SystemWindowAuthority,now:Instant)->Result<View,Reason>{
  let lease=window.lease()?;let mut s=self.state.lock().unwrap();s.authorize(&lease)?;s.sweep(now);let fresh=s.fresh(&lease,now);let mut scope=None;let mut enable_scope=None;let mut disable_scope=None;let mut targets=Vec::new();
  let actionable=fresh&&s.cache.as_ref().is_some_and(|c|now.saturating_duration_since(c.at)<s.config.target_ttl);
  // All-or-none handle publication: a full registry cannot hide a missing scope
  // behind an apparently successful view. Existing bindings remain valid.
  if actionable {
   let c=s.cache.as_ref().unwrap();let native=c.targets.clone();let producer=c.producer;
   let scan=c.scan_support==ScanSupport::VerifiedByAdapter;let enable=c.enable_wifi_support;let disable=c.disable_wifi_support;
   let mut desired:Vec<_>=native.iter().map(|t|BindingKind::Target(t.kind,t.key.clone())).collect();
   if scan{desired.push(BindingKind::Scope);}if enable{desired.push(BindingKind::EnableWifi);}if disable{desired.push(BindingKind::DisableWifi);}
   let needed=desired.iter().filter(|kind|s.bound_id(&lease,kind,producer,now).is_none()).count();
   if needed>s.config.max_bindings.saturating_sub(s.bindings.len())||s.next.checked_add(needed as u64).is_none(){return Err(Reason::Capacity);}
   if scan{scope=Some(s.bind(&lease,BindingKind::Scope,now)?);}if enable{enable_scope=Some(s.bind(&lease,BindingKind::EnableWifi,now)?);}if disable{disable_scope=Some(s.bind(&lease,BindingKind::DisableWifi,now)?);}
   for target in native{let handle=s.bind(&lease,BindingKind::Target(target.kind,target.key),now)?;targets.push(TargetView{handle,kind:target.kind,label:target.label});}
  }
  let c=s.cache.as_ref();let reason=match c{None=>Some(Reason::NotObserved),Some(c)if c.setting_epoch!=lease.setting_epoch=>Some(Reason::Revoked),Some(c)if c.stale=>c.reason,Some(_)if !fresh=>Some(Reason::CacheExpired),Some(_)if !actionable=>Some(Reason::StaleTarget),_=>None};
  Ok(View{schema_version:2,source:"dmsNetwork",generation:c.map(|_|s.generation.to_string()),stale:!fresh,refreshing:s.flight.is_some(),data:c.map(|c|c.data.clone()),scan_scope:scope.clone(),enable_wifi_scope:enable_scope.clone(),disable_wifi_scope:disable_scope.clone(),targets,capabilities:Capabilities{read:true,scan:scope.is_some()&&s.flight.is_none(),enable_wifi:enable_scope.is_some()&&s.flight.is_none(),disable_wifi:disable_scope.is_some()&&s.flight.is_none(),connect:false,radio:false,credentials:false},reason})
 }
 pub fn ticket(&self,window:&SystemWindowAuthority,id:&str,now:Instant)->Result<Ticket,Reason>{let lease=window.lease()?;let mut s=self.state.lock().unwrap();s.authorize(&lease)?;if id.len()>128{return Err(Reason::NotFound);}s.sweep(now);let t=s.tickets.get(id).ok_or(Reason::NotFound)?;if !t.owner.same_window(&lease){return Err(Reason::Unauthorized);}Ok(t.ticket.clone())}
 pub fn cancel(&self,window:&SystemWindowAuthority,id:&str,now:Instant)->Result<Ticket,Reason>{let lease=window.lease()?;let mut s=self.state.lock().unwrap();s.authorize(&lease)?;if id.len()>128{return Err(Reason::NotFound);}s.sweep(now);let t=s.tickets.get_mut(id).ok_or(Reason::NotFound)?;if !t.owner.same_window(&lease){return Err(Reason::Unauthorized);}match t.ticket.phase{Phase::Queued=>{t.ticket.phase=Phase::Cancelled;t.ticket.reason=Some(Reason::CancelledBeforeDispatch)},Phase::Running=>{t.ticket.phase=Phase::Unknown;t.ticket.reason=Some(Reason::CancelledAfterDispatch);t.ticket.refresh_recommended=t.ticket.kind.mutating()},_=>{}}Ok(t.ticket.clone())}
 pub fn extension(&self,window:&SystemWindowAuthority,_operation:FutureOperation)->Result<(),Reason>{let lease=window.lease()?;self.state.lock().unwrap().authorize(&lease)?;Err(Reason::Unsupported)}
}
impl Drop for Service{fn drop(&mut self){let mut s=self.state.lock().unwrap();s.alive=false;s.stale(Reason::Closed);}}
impl Work {
 /// Call at the ACTUAL writer, after verifying the peer for this operation.
 /// Checks precede admission; they are not rollback of already-started IO.
 pub fn begin_dispatch(&mut self,peer:ProducerId,now:Instant)->Result<Operation,Reason>{
  let mut s=self.state.lock().unwrap();s.sweep(now);s.authorize(&self.owner)?;if now>=self.deadline{return Err(Reason::Deadline);}
  let f=s.flight.as_ref().filter(|f|f.id==self.id).ok_or(Reason::Closed)?;if f.started{return Err(Reason::AlreadyDispatched);}let ids=f.tickets.clone();if !ids.iter().any(|id|s.tickets.get(id).is_some_and(|t|t.ticket.phase==Phase::Queued)){return Err(Reason::CancelledBeforeDispatch);}
  if self.kind.mutating()&&(self.source_generation!=Some(s.generation)||self.target_deadline.is_none_or(|at|now>=at)||!s.fresh(&self.owner,now)){
   for id in ids{if let Some(t)=s.tickets.get_mut(&id){if t.ticket.phase==Phase::Queued{t.ticket.phase=Phase::Failed;t.ticket.reason=Some(Reason::StaleTarget);}}}s.stale(Reason::StaleTarget);return Err(Reason::StaleTarget);
  }
  if self.expected.is_some_and(|p|p!=peer){for id in ids{if let Some(t)=s.tickets.get_mut(&id){if matches!(t.ticket.phase,Phase::Queued|Phase::Running){t.ticket.phase=Phase::Failed;t.ticket.reason=Some(Reason::SourceChanged);}}}s.stale(Reason::SourceChanged);return Err(Reason::SourceChanged);}
  let f=s.flight.as_mut().unwrap();f.started=true;f.peer=Some(peer);for id in ids{if let Some(t)=s.tickets.get_mut(&id){if t.ticket.phase==Phase::Queued{t.ticket.phase=Phase::Running;}}}if self.kind.mutating(){s.stale(Reason::UnknownAfterDispatch);}Ok(self.kind.operation())
 }
 fn complete(mut self,observation:Option<NativeObservation>,ack:Option<(Kind,ProducerId)>,error:Option<Reason>,now:Instant)->Result<(),Reason>{
  let parsed=if let Some(v)=observation{
   let enable=enable_wifi_supported(&v.result,v.api_version,&v.capabilities);
   let disable=disable_wifi_supported(&v.result,v.api_version,&v.capabilities);
   let projection=projection::project(&v.result,v.api_version,&v.capabilities,v.observed_at_ms).map_err(|_|Reason::InvalidObservation);
   let mut identities=BTreeSet::new();let unique=v.targets.iter().all(|t|identities.insert((t.kind,t.key.clone())));
   match projection{Ok(value)if unique=>serde_json::to_value(value).map(|value|Some((Projected(value),v.producer,v.targets,v.scan_support,enable,disable))).map_err(|_|Reason::InvalidObservation),_=>Err(Reason::InvalidObservation)}
  }else{Ok(None)};
  let mut s=self.state.lock().unwrap();s.sweep(now);let f=s.flight.as_ref().filter(|f|f.id==self.id).ok_or(Reason::Closed)?;let ids=f.tickets.clone();let started=f.started;let peer=f.peer;let live=ids.iter().any(|id|s.tickets.get(id).is_some_and(|t|matches!(t.ticket.phase,Phase::Queued|Phase::Running)));
  // A connection/peer failure before the writer is a valid typed failure,
  // not a successful completion without dispatch. Never mark it started.
  let result=if !s.alive{Err(Reason::Closed)}else if !self.owner.valid(){Err(Reason::Revoked)}else if now>=self.deadline{Err(Reason::Deadline)}else if !live{Err(if started{Reason::CancelledAfterDispatch}else{Reason::CancelledBeforeDispatch})}else if let Some(error)=error{Err(error)}else if !started{Err(Reason::InvalidObservation)}else{parsed};
  let result=result.and_then(|data|match(self.kind,data,ack){(Kind::Read,Some((data,producer,targets,support,enable,disable)),None)if peer==Some(producer)=>{if targets.len()>s.config.max_targets{return Err(Reason::InvalidObservation);}let generation=s.generation.checked_add(1).ok_or(Reason::Capacity)?;s.generation=generation;s.bindings.clear();s.cache=Some(Cached{data,producer,targets,scan_support:support,enable_wifi_support:enable,disable_wifi_support:disable,at:now,setting_epoch:self.owner.setting_epoch,stale:false,reason:None});Ok(())},(Kind::Scan|Kind::EnableWifi|Kind::DisableWifi,None,Some((ack_kind,producer)))if self.kind==ack_kind&&peer==Some(producer)&&self.expected==Some(producer)=>Ok(()),_=>Err(Reason::InvalidObservation)});
  for id in ids{if let Some(t)=s.tickets.get_mut(&id){if matches!(t.ticket.phase,Phase::Queued|Phase::Running){match result{Ok(())=>{t.ticket.phase=match self.kind{Kind::Read=>Phase::Observed,Kind::Scan|Kind::EnableWifi|Kind::DisableWifi=>Phase::Acknowledged};t.ticket.refresh_recommended=self.kind.mutating();t.ticket.reason=None},Err(reason)=>{t.ticket.phase=if started&&self.kind.mutating(){Phase::Unknown}else{Phase::Failed};t.ticket.reason=Some(reason);t.ticket.refresh_recommended=started&&self.kind.mutating()}}}}}
  if let Err(reason)=result{s.stale(reason);}s.flight=None;self.done=true;result
 }
 pub fn finish_read(self,value:NativeObservation,now:Instant)->Result<(),Reason>{self.complete(Some(value),None,None,now)}
 pub fn finish_scan(self,peer:ProducerId,now:Instant)->Result<(),Reason>{self.complete(None,Some((Kind::Scan,peer)),None,now)}
 pub fn finish_enable_wifi(self,peer:ProducerId,now:Instant)->Result<(),Reason>{self.complete(None,Some((Kind::EnableWifi,peer)),None,now)}
pub fn finish_disable_wifi(self,peer:ProducerId,now:Instant)->Result<(),Reason>{self.complete(None,Some((Kind::DisableWifi,peer)),None,now)}
 pub fn fail(self,error:NativeError,now:Instant)->Result<(),Reason>{self.complete(None,None,Some(match error{NativeError::Unavailable=>Reason::NativeUnavailable,NativeError::Rejected=>Reason::NativeRejected,NativeError::InvalidResponse=>Reason::InvalidObservation}),now)}
}
impl Drop for Work {fn drop(&mut self){if self.done{return;}let mut s=self.state.lock().unwrap();if let Some(f)=s.flight.as_ref().filter(|f|f.id==self.id){let started=f.started;let reason=if started{Reason::UnknownAfterDispatch}else{Reason::AbandonedBeforeDispatch};let ids=f.tickets.clone();let mut settled=false;let previous=ids.iter().find_map(|id|s.tickets.get(id).and_then(|t|t.ticket.reason));for id in ids{if let Some(t)=s.tickets.get_mut(&id){if matches!(t.ticket.phase,Phase::Queued|Phase::Running){settled=true;t.ticket.phase=if started{Phase::Unknown}else{Phase::Cancelled};t.ticket.reason=Some(reason);t.ticket.refresh_recommended=started&&self.kind.mutating();}}}s.stale(if settled{reason}else{previous.unwrap_or(reason)});s.flight=None;}}}

#[cfg(test)]
#[path="lifecycle_tests.rs"]
mod tests;
