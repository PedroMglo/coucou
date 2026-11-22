//! Common physical step contracts. Legacy MPRIS objects keep their exact wire
//! form; a System output has a distinct closed shape, never a fake Spotify peer.
use super::{model,plans::OperationProposal};
use crate::{spotify::goal as spotify,pet_config::{Configuration,system_access::{Captured,Operation}}};
use serde::{Serialize,Deserialize};
use std::{sync::Arc,time::Instant};
pub(crate)use spotify::{Delivery,Failure};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]pub(crate)enum SoundKind{SystemSoundOutput}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct SoundBinding{pub schema_version:u32,pub kind:SoundKind,pub pet_id:String,pub grant_revision:String,pub output_key:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(untagged)]
pub(crate)enum Binding{Spotify(spotify::Binding),Sound(SoundBinding),Health(crate::pet_health::task::Binding),Inventory(crate::pet_host_inventory::Binding)}
impl Binding{
 pub(crate)fn spotify(&self)->Option<&spotify::Binding>{if let Self::Spotify(b)=self{Some(b)}else{None}}
 pub(crate)fn sound(&self)->Option<&SoundBinding>{if let Self::Sound(b)=self{Some(b)}else{None}}
 pub(crate)fn health(&self)->Option<&crate::pet_health::task::Binding>{if let Self::Health(b)=self{Some(b)}else{None}}
 pub(crate)fn inventory(&self)->Option<&crate::pet_host_inventory::Binding>{if let Self::Inventory(b)=self{Some(b)}else{None}}
 pub(crate)fn host_read(&self)->bool{matches!(self,Self::Health(_)|Self::Inventory(_))}
 pub(crate)fn valid(&self)->bool{match self{
  Self::Health(b)=>b.valid(),Self::Inventory(b)=>b.valid(),
  Self::Spotify(b)=>b.schema_version==1&&!b.bus_id.is_empty()&&b.bus_id.len()<=128&&b.owner.starts_with(':')&&b.owner.len()<=256&&b.pid>0&&!b.identity.trim().is_empty()&&b.identity.len()<=256,
  Self::Sound(b)=>b.schema_version==1&&model::key(&b.pet_id)&&crate::pet_inference::revision_key(&b.grant_revision)&&crate::pet_config::system_access::output_key(&b.output_key),
 }}
 pub(crate)fn resource_key(&self)->model::Result<String>{match self{Self::Spotify(b)=>Ok(format!("mpris_{}",model::fingerprint(&(&b.bus_id,&b.owner))?)),Self::Sound(b)=>Ok(format!("system_sound_{}",b.output_key)),Self::Health(b)=>Ok(format!("system_health_{}",model::fingerprint(&b.host)?)),Self::Inventory(b)=>Ok(format!("system_inventory_{}",model::fingerprint(&b.host)?))}}
 pub(crate)fn label(&self)->String{match self{Self::Spotify(b)=>format!("{} · {}",b.identity,b.owner),Self::Sound(b)=>format!("Som · {} · {}",b.pet_id,b.output_key),Self::Health(b)=>format!("Saúde · {} · {}",b.pet_id,b.host.boot_id),Self::Inventory(b)=>format!("Inventário · {} · {}",b.pet_id,b.host.boot_id)}}
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct SoundSnapshot{pub kind:SoundKind,pub output_key:String,pub volume_percent:Option<f64>,pub muted:Option<bool>,pub observed_at:u64}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(untagged)]pub(crate)enum Snapshot{Spotify(spotify::Snapshot),Sound(SoundSnapshot),Health(crate::pet_health::task::Evidence),HealthBaseline(crate::pet_health::baseline::Baseline),Inventory(crate::pet_host_inventory::Snapshot)}
impl From<spotify::Snapshot>for Snapshot{fn from(v:spotify::Snapshot)->Self{Self::Spotify(v)}}
impl Snapshot{
 pub(crate)fn spotify(&self)->Option<&spotify::Snapshot>{if let Self::Spotify(s)=self{Some(s)}else{None}}
 pub(crate)fn observed_at(&self)->u64{match self{Self::Spotify(s)=>s.observed_at,Self::Sound(s)=>s.observed_at,Self::Health(s)=>s.observed_at,Self::HealthBaseline(s)=>s.after.observed_at,Self::Inventory(s)=>s.observed_at()}}
 pub(crate)fn valid_between(&self,from:u64,to:u64)->bool{let n=self.observed_at();n>=from&&n<=to&&match self{Self::Spotify(s)=>matches!(s.playback_status.as_str(),"Playing"|"Paused"|"Stopped")&&s.volume.is_finite()&&(0.0..=1.0).contains(&s.volume),Self::Health(s)=>s.valid()&&s.sample_started_at>=from,Self::Inventory(s)=>s.valid()&&s.started_at()>=from,Self::HealthBaseline(s)=>s.valid()&&s.before.sample_started_at>=from,Self::Sound(s)=>crate::pet_config::system_access::output_key(&s.output_key)&&s.volume_percent.is_none_or(|v|v.is_finite()&&(0.0..=150.0).contains(&v))}}
 pub(crate)fn matches_resource(&self,b:&Binding)->bool{match(self,b){(Self::Spotify(_),Binding::Spotify(_))=>true,(Self::Sound(s),Binding::Sound(b))=>s.output_key==b.output_key,(Self::Health(s),Binding::Health(b))=>s.matches(b),(Self::HealthBaseline(s),Binding::Health(b))=>s.matches(b),(Self::Inventory(s),Binding::Inventory(b))=>s.matches(b),_=>false}}
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Observation{pub delivery:Delivery,pub before:Option<Snapshot>,pub after:Option<Snapshot>,pub effect_verified:bool,pub scope_current:bool,pub failure:Option<Failure>}
impl From<spotify::Observation>for Observation{fn from(v:spotify::Observation)->Self{Self{delivery:v.delivery,before:v.before.map(Into::into),after:v.after.map(Into::into),effect_verified:v.effect_verified,scope_current:v.scope_current,failure:v.failure}}}
#[derive(Clone)]pub(crate)enum Action{Spotify(spotify::Action),Sound(crate::pet_audio::task::Action),Health,HealthBaseline,Inventory}
impl Action{
 pub(crate)fn parse(op:&OperationProposal)->model::Result<Self>{
  if matches!(op.operation_id.as_str(),"system.health.observe"|"system.health.baseline"){return if crate::pet_health::task::validate(&op.operation_id,op.version,&op.arguments){Ok(if op.operation_id=="system.health.baseline"{Self::HealthBaseline}else{Self::Health})}else{Err(model::Error::InvalidInput)};}
  if op.operation_id=="system.inventory.observe"{return if crate::pet_host_inventory::validate(&op.operation_id,op.version,&op.arguments){Ok(Self::Inventory)}else{Err(model::Error::InvalidInput)};}
  if op.operation_id.starts_with("system.sound."){return crate::pet_audio::task::Action::parse(&op.operation_id,op.version,&op.arguments).map(Self::Sound).map_err(|_|model::Error::InvalidInput);}
  if op.operation_id!="spotify.control"||op.version!=1||!op.valid(){return Err(model::Error::UnsupportedPlatform);}
  use crate::pet_operations::Music;let action=match serde_json::from_value::<Music>(op.arguments.clone()).map_err(|_|model::Error::InvalidInput)?{Music::Play=>spotify::Action::Play,Music::Pause=>spotify::Action::Pause,Music::SetVolume{volume}=>spotify::Action::SetVolume{volume},_=>return Err(model::Error::UnsupportedPlatform)};Ok(Self::Spotify(action))
 }
}
enum SoundIo{Legacy(crate::pet_audio::task::Io),Native(Arc<crate::pet_audio::native_task::RetainedOutput>)}
struct SoundTarget{binding:Binding,scope:Arc<Captured>,io:SoundIo}
struct HealthTarget{binding:crate::pet_health::task::Binding,scope:Arc<Captured>,retired:std::sync::atomic::AtomicBool,#[cfg(test)]collections:std::sync::atomic::AtomicUsize}
fn host_current(scope:&Captured,host:&crate::pet_health::task::Host,retired:&std::sync::atomic::AtomicBool)->bool{use std::sync::atomic::Ordering;if retired.load(Ordering::Acquire){return false;}let ok=scope.check().is_ok()&&crate::pet_health::task::Host::capture().is_ok_and(|h|h==*host);if !ok{retired.store(true,Ordering::Release);}ok}
impl HealthTarget{
 fn current(&self)->bool{host_current(&self.scope,&self.binding.host,&self.retired)}
 fn observe(&self,current:&dyn Fn()->bool)->Result<crate::pet_health::task::Evidence,String>{#[cfg(test)]self.collections.fetch_add(1,std::sync::atomic::Ordering::Relaxed);crate::pet_health::task::sample(&self.binding,current)}
 fn baseline(&self,current:&dyn Fn()->bool)->Result<crate::pet_health::baseline::Baseline,String>{#[cfg(test)]self.collections.fetch_add(1,std::sync::atomic::Ordering::Relaxed);crate::pet_health::baseline::measure(&self.binding,current)}
}
struct InventoryTarget{binding:crate::pet_host_inventory::Binding,scope:Arc<Captured>,retired:std::sync::atomic::AtomicBool,#[cfg(test)]collections:std::sync::atomic::AtomicUsize}
impl InventoryTarget{
 fn current(&self)->bool{host_current(&self.scope,&self.binding.host,&self.retired)}
 fn observe(&self,current:&dyn Fn()->bool)->Result<crate::pet_host_inventory::Snapshot,String>{#[cfg(test)]self.collections.fetch_add(1,std::sync::atomic::Ordering::Relaxed);crate::pet_host_inventory::sample(&self.binding,current)}
}
#[derive(Clone)]enum Backend{Spotify(Arc<spotify::Target>),Sound(Arc<SoundTarget>),Health(Arc<HealthTarget>),Inventory(Arc<InventoryTarget>)}
#[derive(Clone)]pub(crate)struct Target{binding:Binding,backend:Backend}
impl From<Arc<spotify::Target>>for Target{fn from(v:Arc<spotify::Target>)->Self{Self{binding:Binding::Spotify(v.binding().clone()),backend:Backend::Spotify(v)}}}
impl From<Arc<Target>>for Target{fn from(v:Arc<Target>)->Self{v.as_ref().clone()}}
fn sound_snapshot(s:&crate::pet_audio::Snapshot,key:&str)->Result<Snapshot,Failure>{
 let node=s.outputs.iter().find(|n|n.kind==crate::pet_audio::Kind::Output&&n.key==key).ok_or(Failure::Changed)?;
 if !s.available{return Err(Failure::Unavailable);}Ok(Snapshot::Sound(SoundSnapshot{kind:SoundKind::SystemSoundOutput,output_key:key.into(),volume_percent:node.volume,muted:node.muted,observed_at:s.observed_at.ok_or(Failure::InvalidState)?}))
}
impl Target{
 #[cfg(test)]pub(in crate::pet_runtime)fn fixture_sound_lifetime(&self)->std::sync::Weak<dyn Send+Sync>{
  let Backend::Sound(target)=&self.backend else{panic!("Expected private native sound target");};let erased:Arc<dyn Send+Sync>=target.clone();Arc::downgrade(&erased)
 }
 #[cfg(test)]pub(in crate::pet_runtime)fn collector_calls(&self)->usize{match &self.backend{Backend::Health(t)=>t.collections.load(std::sync::atomic::Ordering::Relaxed),Backend::Inventory(t)=>t.collections.load(std::sync::atomic::Ordering::Relaxed),_=>0}}
 pub(crate)fn binding(&self)->&Binding{&self.binding}
 pub(in crate::pet_runtime)fn native_incarnation(&self)->Option<super::native_system_plan::Incarnation>{
  if let Backend::Sound(t)=&self.backend{if let SoundIo::Native(n)=&t.io{let(core_cookie,node_id,node_serial)=n.incarnation();return Some(super::native_system_plan::Incarnation{core_cookie,node_id,node_serial});}}None
 }
 /// Native read-only capture for a retained Review, not a public dispatch gate.
 pub(in crate::pet_runtime)fn capture_system_review(op:&OperationProposal,pet:&str,config:&Configuration,current:&dyn Fn()->bool)->model::Result<Self>{
  if matches!(Action::parse(op)?,Action::Inventory){return Self::inventory(Captured::capture_inventory_reviewed(config,pet).map_err(|_|model::Error::Conflict)?,pet,current);}
  if matches!(Action::parse(op)?,Action::Health|Action::HealthBaseline){return Self::health(Captured::capture_health_reviewed(config,pet).map_err(|_|model::Error::Conflict)?,pet,current);}
  let Action::Sound(action)=Action::parse(op)? else{return Err(model::Error::UnsupportedPlatform);};
  let discovery=Captured::capture_reviewed(config,pet,Operation::Observe,None).map_err(|_|model::Error::Conflict)?;
  Self::sound_native(discovery,&action,pet,current,|authority,check|crate::pet_audio::pipewire_reader::Reader::start_anchor(Instant::now()+std::time::Duration::from_secs(180),authority,check))
 }
 pub(crate)fn capture(op:&OperationProposal,pet:&str,config:&Configuration,current:impl Fn()->bool)->model::Result<Self>{
  if !current(){return Err(model::Error::Conflict);}
  match Action::parse(op)?{
   Action::Health|Action::HealthBaseline=>Self::health(Captured::capture_health_reviewed(config,pet).map_err(|_|model::Error::Conflict)?,pet,&current), // Scope/recipe versions below still require original native System admission.
   Action::Inventory=>Self::inventory(Captured::capture_inventory_reviewed(config,pet).map_err(|_|model::Error::Conflict)?,pet,&current),
   Action::Spotify(_)=>Ok(Arc::new(spotify::Target::capture().map_err(|_|model::Error::Conflict)?).into()),
   Action::Sound(action)=>{
    if !crate::pet_audio::task::native_dispatch_available(){return Err(model::Error::UnsupportedPlatform);}
    let discovery=Captured::capture_reviewed(config,pet,Operation::Observe,None).map_err(|_|model::Error::Conflict)?;
    Self::sound_native(discovery,&action,pet,&current,|authority,check|crate::pet_audio::pipewire_reader::Reader::start_anchor(Instant::now()+std::time::Duration::from_secs(180),authority,check))

   }
  }
 }
 fn inventory(scope:Captured,pet:&str,current:&dyn Fn()->bool)->model::Result<Self>{
  if !current()||scope.check().is_err(){return Err(model::Error::Conflict);}
  let binding=crate::pet_host_inventory::Binding{schema_version:1,kind:crate::pet_host_inventory::Kind::SystemHostInventory,pet_id:pet.into(),grant_revision:scope.revision().into(),host:crate::pet_health::task::Host::capture().map_err(|_|model::Error::Conflict)?};
  let target=Self{binding:Binding::Inventory(binding.clone()),backend:Backend::Inventory(Arc::new(InventoryTarget{binding,scope:Arc::new(scope),retired:std::sync::atomic::AtomicBool::new(false),#[cfg(test)]collections:std::sync::atomic::AtomicUsize::new(0)}))};
  if !target.identity_current(current){return Err(model::Error::Conflict);}Ok(target)
 }
 #[cfg(test)]pub(in crate::pet_runtime)fn inventory_fixture(file:&std::path::Path,pet:&str)->model::Result<Self>{Self::inventory(Captured::fixture_inventory(file,pet).map_err(|_|model::Error::Conflict)?,pet,&||true)}
 fn health(scope:Captured,pet:&str,current:&dyn Fn()->bool)->model::Result<Self>{
  if !current()||scope.check().is_err(){return Err(model::Error::Conflict);}
  let binding=crate::pet_health::task::Binding{schema_version:1,kind:crate::pet_health::task::Kind::SystemHostHealth,pet_id:pet.into(),grant_revision:scope.revision().into(),host:crate::pet_health::task::Host::capture().map_err(|_|model::Error::Conflict)?};
  let target=Self{binding:Binding::Health(binding.clone()),backend:Backend::Health(Arc::new(HealthTarget{binding,scope:Arc::new(scope),retired:std::sync::atomic::AtomicBool::new(false),#[cfg(test)]collections:std::sync::atomic::AtomicUsize::new(0)}))};
  if !target.identity_current(current){return Err(model::Error::Conflict);}Ok(target)
 }
 pub(in crate::pet_runtime)fn identity_current(&self,current:&dyn Fn()->bool)->bool{current()&&match &self.backend{Backend::Health(h)=>h.current(),Backend::Inventory(i)=>i.current(),_=>self.read_with(current).is_ok()}}
 #[cfg(test)]pub(in crate::pet_runtime)fn health_fixture(file:&std::path::Path,pet:&str)->model::Result<Self>{Self::health(Captured::fixture_health(file,pet).map_err(|_|model::Error::Conflict)?,pet,&||true)}
 fn sound(discovery:Captured,action:&crate::pet_audio::task::Action,pet:&str,io:crate::pet_audio::task::Io,current:&dyn Fn()->bool)->model::Result<Self>{
  let _pinned=discovery.pin_output(action.operation(),action.key()).map_err(|_|model::Error::Conflict)?;let scope=Arc::new(discovery);
  let binding=Binding::Sound(SoundBinding{schema_version:1,kind:SoundKind::SystemSoundOutput,pet_id:pet.into(),grant_revision:scope.revision().into(),output_key:action.key().into()});
  let target=Self{binding:binding.clone(),backend:Backend::Sound(Arc::new(SoundTarget{binding,scope,io:SoundIo::Legacy(io)}))};target.read_with(current).map_err(|_|model::Error::Conflict)?;Ok(target)
 }
 fn sound_native(discovery:Captured,action:&crate::pet_audio::task::Action,pet:&str,current:&dyn Fn()->bool,reader:impl FnOnce(Arc<dyn Fn()->bool+Send+Sync>,&dyn Fn()->bool)->Result<crate::pet_audio::pipewire_reader::Reader,crate::pet_helper::ExchangeError>)->model::Result<Self>{
  discovery.pin_output(action.operation(),action.key()).map_err(|_|model::Error::Conflict)?;let scope=Arc::new(discovery);let original=scope.clone();
  let reader=reader(Arc::new(move||original.check().is_ok()),current).map_err(|_|model::Error::Conflict)?;
  let native=crate::pet_audio::native_task::RetainedOutput::capture(reader,action.key(),&||current()&&scope.check().is_ok()).map_err(|_|model::Error::Conflict)?;
  let binding=Binding::Sound(SoundBinding{schema_version:1,kind:SoundKind::SystemSoundOutput,pet_id:pet.into(),grant_revision:scope.revision().into(),output_key:action.key().into()});
  Ok(Self{binding:binding.clone(),backend:Backend::Sound(Arc::new(SoundTarget{binding,scope,io:SoundIo::Native(Arc::new(native))}))})
 }
 #[cfg(test)]pub(in crate::pet_runtime)fn sound_native_fixture(file:&std::path::Path,action:&crate::pet_audio::task::Action,pet:&str,runtime:&std::path::Path)->model::Result<Self>{
  let discovery=Captured::fixture(file,pet,Operation::Observe,None).map_err(|_|model::Error::Conflict)?;
  Self::sound_native(discovery,action,pet,&||true,|scope,current|crate::pet_audio::pipewire_reader::Reader::fixture_anchor(runtime,scope,current))
 }
 #[cfg(test)]pub(in crate::pet_runtime)fn sound_fixture(file:&std::path::Path,action:&crate::pet_audio::task::Action,pet:&str,io:crate::pet_audio::task::Io)->model::Result<Self>{let scope=Captured::fixture(file,pet,Operation::Observe,None).map_err(|_|model::Error::Conflict)?;Self::sound(scope,action,pet,io,&||true)}
 pub(crate)fn permits(&self,action:&Action)->bool{match(&self.backend,action){(Backend::Health(h),Action::Health|Action::HealthBaseline)=>h.current(),(Backend::Inventory(i),Action::Inventory)=>i.current(),(Backend::Spotify(_),Action::Spotify(_))=>true,(Backend::Sound(v),Action::Sound(a))=>v.binding.sound().is_some_and(|b|a.key()==b.output_key)&&v.scope.pin_output(a.operation(),a.key()).is_ok(),_=>false}}
 pub(crate)fn read(&self)->Result<Snapshot,Failure>{self.read_with(&||true)}
 pub(crate)fn read_with(&self,current:&dyn Fn()->bool)->Result<Snapshot,Failure>{if !current(){return Err(Failure::Cancelled);}let snapshot=match &self.backend{
  Backend::Spotify(v)=>v.read().map(Into::into),
  Backend::Inventory(v)=>{let check=||current()&&v.current();let result=v.observe(&check).map(Snapshot::Inventory).map_err(|_|Failure::Unavailable);if !check(){return Err(Failure::NotAuthorized);}result},
  Backend::Health(v)=>{let check=||current()&&v.current();let result=v.observe(&check).map(Snapshot::Health).map_err(|_|Failure::Unavailable);if !check(){return Err(Failure::NotAuthorized);}result},
  Backend::Sound(v)=>{let key=&v.binding.sound().ok_or(Failure::InvalidState)?.output_key;
   let check=||current()&&v.scope.check().is_ok();let snapshot=match &v.io{
    SoundIo::Legacy(io)=>sound_snapshot(&crate::pet_audio::linux::task_read_outputs_with(io,&check).map_err(|_|Failure::Unavailable)?,key),
    SoundIo::Native(native)=>native.read(&check).map(|n|native_snapshot(n,key)).map_err(|_|Failure::Changed),
   };v.scope.check().map_err(|_|Failure::NotAuthorized)?;snapshot},
 };if !current(){return Err(Failure::Cancelled);}snapshot}
 pub(crate)fn apply(&self,action:&Action,deadline:Instant,current:impl Fn()->bool,before_effect:impl FnOnce()->Result<(),Failure>)->Observation{
  match(&self.backend,action){
   (Backend::Inventory(v),Action::Inventory)=>{
    let check=||Instant::now()<deadline&&current()&&v.current();
    if !check(){return refused(Failure::NotAuthorized);}if let Err(e)=before_effect(){return refused(e);}if !check(){return refused(Failure::NotAuthorized);}
    let after=v.observe(&check).ok().filter(|_|check());let scope_current=check();let verified=scope_current&&after.as_ref().is_some_and(|s|s.complete());
    Observation{delivery:if after.is_some(){Delivery::Accepted}else{Delivery::Uncertain},before:None,after:after.map(Snapshot::Inventory),effect_verified:verified,scope_current,failure:if verified{None}else{Some(if scope_current{Failure::Unavailable}else{Failure::NotAuthorized})}}
   },
   (Backend::Health(v),Action::HealthBaseline)=>{
    let check=||Instant::now()<deadline&&current()&&v.current();
    if !check(){return refused(Failure::NotAuthorized);}if let Err(e)=before_effect(){return refused(e);}if !check(){return refused(Failure::NotAuthorized);}
    let after=v.baseline(&check).ok().filter(|_|check());let scope_current=check();let verified=scope_current&&after.as_ref().is_some_and(|s|s.samples_complete());
    Observation{delivery:if after.is_some(){Delivery::Accepted}else{Delivery::Uncertain},before:None,after:after.map(Snapshot::HealthBaseline),effect_verified:verified,scope_current,failure:if verified{None}else{Some(if scope_current{Failure::Unavailable}else{Failure::NotAuthorized})}}
   },
   (Backend::Health(v),Action::Health)=>{
    let check=||Instant::now()<deadline&&current()&&v.current();
    if !check(){return refused(Failure::NotAuthorized);}if let Err(e)=before_effect(){return refused(e);}if !check(){return refused(Failure::NotAuthorized);}
    let after=v.observe(&check).ok().filter(|_|check());let scope_current=check();let verified=scope_current&&after.as_ref().is_some_and(|s|s.complete());
    Observation{delivery:if verified{Delivery::Accepted}else{Delivery::Uncertain},before:None,after:after.map(Snapshot::Health),effect_verified:verified,scope_current,failure:if verified{None}else{Some(if scope_current{Failure::Unavailable}else{Failure::NotAuthorized})}}
   },
   (Backend::Spotify(v),Action::Spotify(action))=>v.apply(action,deadline,current,before_effect).into(),
   (Backend::Sound(v),Action::Sound(action))=>{
    let check=||Instant::now()<deadline&&current()&&v.scope.check().is_ok();
    let Some(binding)=v.binding.sound()else{return refused(Failure::InvalidState);};if action.key()!=binding.output_key||v.scope.pin_output(action.operation(),action.key()).is_err(){return refused(Failure::NotAuthorized);}
    if let SoundIo::Native(native)=&v.io{
     let result=native.dispatch(action,deadline,&check,||before_effect().map_err(|_|()));
     let before=result.before.map(|n|native_snapshot(n,&binding.output_key));let after=result.after.map(|n|native_snapshot(n,&binding.output_key));
     let scope_current=result.scope_current&&check()&&after.is_some();let verified=result.verified&&scope_current;
     return Observation{delivery:if !result.possible_effect{Delivery::NotDispatched}else if result.transport_accepted&&verified{Delivery::Accepted}else{Delivery::Uncertain},before,after,effect_verified:verified,scope_current,failure:if verified{None}else{Some(if check(){Failure::Transport}else{Failure::NotAuthorized})}};
    }
    let SoundIo::Legacy(io)=&v.io else{return refused(Failure::InvalidState)};
    let result=crate::pet_audio::linux::task_control_with(io,action.request(),deadline,&check,||before_effect().map_err(|_|"A intenção ou a reserva de execução não ficou confirmada.".to_string()));
    let before=result.before.as_ref().and_then(|s|sound_snapshot(s,&binding.output_key).ok());let after=result.after.as_ref().and_then(|s|sound_snapshot(s,&binding.output_key).ok());
    let scope_current=result.scope_current&&check()&&after.is_some();let verified=result.verified&&scope_current;
    Observation{delivery:if !result.possible_effect{Delivery::NotDispatched}else if result.acknowledged{Delivery::Accepted}else{Delivery::Uncertain},before,after,effect_verified:verified,scope_current,failure:if verified{None}else{Some(if check(){Failure::Transport}else{Failure::NotAuthorized})}}
   },_=>refused(Failure::InvalidAction),
  }
 }
}
fn native_snapshot(n:crate::pet_audio::pipewire_reader::Output,key:&str)->Snapshot{Snapshot::Sound(SoundSnapshot{kind:SoundKind::SystemSoundOutput,output_key:key.into(),volume_percent:n.volume_percent,muted:n.muted,observed_at:crate::assistant_memory::now()})}
fn refused(failure:Failure)->Observation{Observation{delivery:Delivery::NotDispatched,before:None,after:None,effect_verified:false,scope_current:false,failure:Some(failure)}}
pub(in crate::pet_runtime)fn step_operation<'a>(g:&'a model::Record,plan:&str,step:Option<&str>)->model::Result<&'a OperationProposal>{g.plans.iter().find(|p|p.id==plan).and_then(|p|p.steps.iter().find(|s|step.is_none_or(|id|s.id==id))).and_then(|s|s.operation.as_ref()).ok_or(model::Error::InvalidInput)}
