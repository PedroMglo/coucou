//! Fixed typed native driver. No renderer/model entry point. Read-only construction
//! cannot enable effects. Task adoption remains gated until resource mapping is proved.
//! A retained native object is needed before task effects can be enabled.
use std::{path::PathBuf,sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant},os::unix::{fs::{DirBuilderExt,PermissionsExt},net::UnixListener}};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::pet_helper::{Resident,ExchangeError,Lane};
#[path="pipewire_reader_qml.rs"]mod driver;
const MAX_OUTPUTS:usize=64;
fn parse_core_cookie(bytes:&[u8])->Result<u32,ExchangeError>{
 let text=std::str::from_utf8(bytes).map_err(|_|ExchangeError::NotStarted)?;let mut found=None;
 for line in text.lines(){if let Some(value)=line.trim().strip_prefix("cookie:"){
  let value=value.trim();if found.is_some()||value.is_empty()||!value.bytes().all(|c|c.is_ascii_digit()){return Err(ExchangeError::NotStarted);}
  found=Some(value.parse::<u32>().map_err(|_|ExchangeError::NotStarted)?);
 }}found.ok_or(ExchangeError::NotStarted)
}
/// Only core metadata, bounded and discarded after extracting the instance
/// cookie. The same explicit local endpoint is used by this query and QML.
fn read_core_cookie(runtime:&std::ffi::OsStr,profile:&std::path::Path,deadline:Instant,current:&dyn Fn()->bool)->Result<u32,ExchangeError>{
 let remaining=deadline.saturating_duration_since(Instant::now()).min(Duration::from_secs(4));
 if remaining.is_zero()||!current(){return Err(ExchangeError::NotStarted);}
 let mut cmd=crate::pet_helper::command("/usr/bin/pw-cli",&["info".into(),"0".into()]);
 cmd.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C").env("PIPEWIRE_RUNTIME_DIR",runtime).env("PIPEWIRE_REMOTE","pipewire-0")
  .env("XDG_RUNTIME_DIR",profile.join("run")).env("XDG_CONFIG_HOME",profile.join("config")).env("XDG_CACHE_HOME",profile.join("cache")).env("XDG_DATA_HOME",profile.join("data"));
 let session=crate::pet_helper::Session::acquire(Lane::AudioRead,remaining).map_err(|_|ExchangeError::NotStarted)?;
 let bytes=session.run_sync_input(cmd,None,64*1024,remaining,||if current(){Ok(())}else{Err("O pedido nativo terminou.".into())},current).map_err(|_|ExchangeError::NotStarted)?;
 parse_core_cookie(&bytes)
}
#[derive(Clone,Debug,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Output{pub id:u32,pub serial:String,pub volume_percent:Option<f64>,pub muted:Option<bool>}
impl Output{fn valid(&self)->bool{self.id>0&&self.serial.len()<=24&&!self.serial.is_empty()&&self.serial.bytes().all(|b|b.is_ascii_digit())&&self.volume_percent.is_none_or(|v|v.is_finite()&&(0.0..=150.0).contains(&v))}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Reply{version:u32,session:String,sequence:u32,ok:bool,#[serde(default)]outputs:Vec<Output>,#[serde(default)]captured:Option<Output>,#[serde(default)]lost:bool,#[serde(default)]prepared:bool,#[serde(default)]accepted:bool,#[serde(default)]transaction:Option<String>}
#[derive(Clone,Copy,PartialEq,Eq)]enum Mode{ReadOnly,Anchor,Effect}
impl Mode{fn limits(self)->(u64,u32){if self==Self::Anchor{(180,256)}else{(30,16)}}}
struct Profile(PathBuf);
impl Profile{
 fn new()->Result<Self,ExchangeError>{
  use ring::rand::{SecureRandom,SystemRandom};let mut bytes=[0;16];SystemRandom::new().fill(&mut bytes).map_err(|_|ExchangeError::NotStarted)?;
  let name:String=bytes.iter().map(|b|format!("{b:02x}")).collect();let p=std::env::temp_dir().join(format!("coucou-pw-{name}"));
  if p.as_os_str().len()>70{return Err(ExchangeError::NotStarted);}
  std::fs::DirBuilder::new().mode(0o700).create(&p).map_err(|_|ExchangeError::NotStarted)?;Ok(Self(p))
 }
}
impl Drop for Profile{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
pub(crate)struct Reader{
 // Owner/family is collected before removing its private socket/configuration.
 owner:Resident,_profile:Profile,native_guard:Option<Arc<super::effect_lock::Guard>>,session:String,sequence:u32,deadline:Instant,authority:Arc<dyn Fn()->bool+Send+Sync>,retired:AtomicBool,captured:Option<(u32,String)>,runtime:std::ffi::OsString,mode:Mode,core_cookie:u32,max_requests:u32,
}
impl Reader{
 pub(crate)fn start(deadline:Instant,authority:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self,ExchangeError>{
  if Instant::now()>=deadline||!authority()||std::env::var_os("PIPEWIRE_REMOTE").is_some_and(|v|!v.is_empty()){return Err(ExchangeError::NotStarted);}
  let runtime=std::env::var_os("PIPEWIRE_RUNTIME_DIR").or_else(||std::env::var_os("XDG_RUNTIME_DIR")).unwrap_or_else(||format!("/run/user/{}",unsafe{libc::getuid()}).into());
  Self::start_at(deadline,authority,runtime,Mode::ReadOnly,&||true)
 }
 fn start_at(deadline:Instant,authority:Arc<dyn Fn()->bool+Send+Sync>,runtime:std::ffi::OsString,mode:Mode,current:&dyn Fn()->bool)->Result<Self,ExchangeError>{
  if !std::path::Path::new(&runtime).is_absolute()||Instant::now()>=deadline||!authority()||!current(){return Err(ExchangeError::NotStarted);}
  let native_guard=if mode==Mode::Effect{Some(super::effect_lock::acquire_at(std::path::Path::new(&runtime)).map_err(|_|ExchangeError::NotStarted)?)}else{None};
  let(lifetime,max_requests)=mode.limits();let deadline=deadline.min(Instant::now()+Duration::from_secs(lifetime));let profile=Profile::new()?;
  let socket=profile.0.join("channel");let listener=UnixListener::bind(&socket).map_err(|_|ExchangeError::NotStarted)?;
  std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).map_err(|_|ExchangeError::NotStarted)?;
  let qml=profile.0.join("shell.qml");std::fs::write(&qml,driver::QML).map_err(|_|ExchangeError::NotStarted)?;
  std::fs::set_permissions(&qml,std::fs::Permissions::from_mode(0o600)).map_err(|_|ExchangeError::NotStarted)?;
  for dir in ["config","cache","data","run"]{std::fs::DirBuilder::new().mode(0o700).create(profile.0.join(dir)).map_err(|_|ExchangeError::NotStarted)?;}
  // Explicit disposable local Pulse client state; no authenticated user
  // profile, cookie or autospawn preference is read or copied.
  std::fs::write(profile.0.join("pulse-client.conf"),"autospawn = no\n").map_err(|_|ExchangeError::NotStarted)?;
  use std::io::Write;use std::os::unix::fs::OpenOptionsExt;
  let mut pulse_cookie=[0u8;256];ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(),&mut pulse_cookie).map_err(|_|ExchangeError::NotStarted)?;
  std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(profile.0.join("pulse-cookie")).and_then(|mut f|f.write_all(&pulse_cookie)).map_err(|_|ExchangeError::NotStarted)?;
  let core_cookie=read_core_cookie(&runtime,&profile.0,deadline,&||authority()&&current())?;
  use ring::rand::{SecureRandom,SystemRandom};let mut secret=[0;32];SystemRandom::new().fill(&mut secret).map_err(|_|ExchangeError::NotStarted)?;
  let session:String=secret.iter().map(|b|format!("{b:02x}")).collect();
  // Explicit environment: no profile, session-bus, account or model inheritance.
  let mut cmd=crate::pet_helper::command("/usr/bin/qs",&["--no-color".into(),"--path".into(),qml.to_string_lossy().into_owned()]);
  cmd.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C").env("QT_QPA_PLATFORM","offscreen")
   .env("XDG_CONFIG_HOME",profile.0.join("config")).env("XDG_CACHE_HOME",profile.0.join("cache")).env("XDG_DATA_HOME",profile.0.join("data")).env("XDG_RUNTIME_DIR",profile.0.join("run"))
   .env("PIPEWIRE_RUNTIME_DIR",&runtime).env("PIPEWIRE_REMOTE","pipewire-0").env("COUCOU_PW_SOCKET",&socket).env("COUCOU_PW_SESSION",&session).env("COUCOU_PW_EFFECTS",if mode==Mode::Effect{"1"}else{"0"}).env("COUCOU_PW_LIMIT_MS",(lifetime*1000).to_string()).env("COUCOU_PW_MAX_REQUESTS",max_requests.to_string());
  if let Some(guard)=&native_guard{crate::pet_process_group::inherit_file(&mut cmd,guard.file());}
  #[cfg(test)]if mode==Mode::Effect&&crate::pet_runtime::system_effect_crash_tests::detached_writer_fixture(){
   use std::os::unix::process::CommandExt;unsafe{cmd.pre_exec(||{let parent=libc::getpgid(libc::getppid());if parent<0||libc::setpgid(0,parent)<0||libc::setsid()<0{return Err(std::io::Error::last_os_error());}Ok(())});}
  }
  let retired=AtomicBool::new(false);let check=||{let ok=Instant::now()<deadline&&authority()&&current()&&native_guard.as_ref().is_none_or(|g|g.current());if !ok{retired.store(true,Ordering::Release);}ok&&!retired.load(Ordering::Acquire)};
  let owner=Resident::start_socket(if mode==Mode::Effect{Lane::AudioEffect}else{Lane::AudioRead},cmd,&listener,deadline,&check)?;
  if read_core_cookie(&runtime,&profile.0,deadline,&check)?!=core_cookie{return Err(ExchangeError::NotStarted);}
  Ok(Self{owner,_profile:profile,native_guard,session,sequence:0,deadline,authority,retired,captured:None,runtime,mode,core_cookie,max_requests})
 }
 fn exchange(&mut self,operation:&str,selected:Option<&Output>)->Result<Reply,ExchangeError>{self.exchange_checked(operation,selected,None,None,&||true)}
 fn exchange_checked(&mut self,operation:&str,selected:Option<&Output>,transaction:Option<&str>,effect:Option<&NativeAction>,fence:&dyn Fn()->bool)->Result<Reply,ExchangeError>{
  if self.retired.load(Ordering::Acquire)||self.sequence>=self.max_requests{return Err(ExchangeError::NotStarted);}
  self.sequence+=1;
  let mut input=json!({"version":2,"session":self.session,"sequence":self.sequence,"operation":operation,"id":selected.map(|o|o.id),"serial":selected.map(|o|o.serial.as_str())});
  if let Some(token)=transaction{input["transaction"]=json!(token);}if let Some(action)=effect{input["effect"]=serde_json::to_value(action).map_err(|_|ExchangeError::NotStarted)?;}
  let mut bytes=serde_json::to_vec(&input).map_err(|_|ExchangeError::NotStarted)?;bytes.push(b'\n');
  let deadline=self.deadline.min(Instant::now()+Duration::from_secs(4));let retired=&self.retired;let authority=&self.authority;
  let native_guard=&self.native_guard;let current=||{if retired.load(Ordering::Acquire){return false;}let ok=Instant::now()<deadline&&authority()&&fence()&&native_guard.as_ref().is_none_or(|g|g.current());if !ok{retired.store(true,Ordering::Release);}ok&&!retired.load(Ordering::Acquire)};
  let result=read_core_cookie(&self.runtime,&self._profile.0,deadline,&current).and_then(|cookie|{
   if cookie!=self.core_cookie{return Err(ExchangeError::NotStarted);}self.owner.exchange_line(&bytes,32*1024,deadline,&current)
  }).and_then(|b|{
   let v:Reply=serde_json::from_slice(&b).map_err(|_|ExchangeError::Uncertain)?;
   if v.version!=2||v.session!=self.session||v.sequence!=self.sequence||v.outputs.len()>MAX_OUTPUTS||v.outputs.iter().any(|x|!x.valid())||v.captured.as_ref().is_some_and(|x|!x.valid())||v.lost||!v.ok{return Err(ExchangeError::Uncertain);}
   Ok(v)
  });
  if result.is_err(){self.retired.store(true,Ordering::Release);}result
 }
 pub(crate)fn start_anchor(deadline:Instant,authority:Arc<dyn Fn()->bool+Send+Sync>,current:&dyn Fn()->bool)->Result<Self,ExchangeError>{
  if std::env::var_os("PIPEWIRE_REMOTE").is_some_and(|v|!v.is_empty()&&v!="pipewire-0"){return Err(ExchangeError::NotStarted);}
  let runtime=std::env::var_os("PIPEWIRE_RUNTIME_DIR").or_else(||std::env::var_os("XDG_RUNTIME_DIR")).unwrap_or_else(||format!("/run/user/{}",unsafe{libc::getuid()}).into());
  let pulse=std::path::Path::new(&runtime).join("pulse/native");let named=format!("unix:{}",pulse.to_string_lossy());
  if std::env::var_os("PULSE_SERVER").is_some_and(|s|!s.is_empty()&&s!=pulse.as_os_str()&&s!=std::ffi::OsStr::new(&named))
   ||std::env::var_os("PULSE_RUNTIME_PATH").is_some_and(|s|!s.is_empty()&&s!=std::path::Path::new(&runtime).join("pulse").as_os_str()){return Err(ExchangeError::NotStarted);}
  Self::start_at(deadline,authority,runtime,Mode::Anchor,current)
 }
 #[cfg(test)]pub(crate)fn fixture_effect(runtime:&std::path::Path)->Result<Self,ExchangeError>{Self::start_at(Instant::now()+Duration::from_secs(10),Arc::new(||true),runtime.as_os_str().into(),Mode::Effect,&||true)}
 #[cfg(test)]pub(crate)fn fixture_anchor(runtime:&std::path::Path,authority:Arc<dyn Fn()->bool+Send+Sync>,current:&dyn Fn()->bool)->Result<Self,ExchangeError>{Self::start_at(Instant::now()+Duration::from_secs(180),authority,runtime.as_os_str().into(),Mode::Anchor,current)}
 pub(super)fn pulse_command(&self,args:&[String])->std::process::Command{
  let mut command=crate::pet_helper::command("/usr/bin/pactl",args);command.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C")
   .env("PULSE_SERVER",format!("unix:{}",std::path::Path::new(&self.runtime).join("pulse/native").to_string_lossy())).env("PULSE_CLIENTCONFIG",self._profile.0.join("pulse-client.conf")).env("PULSE_COOKIE",self._profile.0.join("pulse-cookie"))
   .env("XDG_CONFIG_HOME",self._profile.0.join("config")).env("XDG_CACHE_HOME",self._profile.0.join("cache")).env("XDG_DATA_HOME",self._profile.0.join("data")).env("XDG_RUNTIME_DIR",self._profile.0.join("run"));command
 }
 pub(crate)fn outputs_fenced(&mut self,current:&dyn Fn()->bool)->Result<Vec<Output>,ExchangeError>{
  if self.captured.is_some(){return Err(ExchangeError::NotStarted);}let r=self.exchange_checked("outputs",None,None,None,current)?;if r.captured.is_some(){return Err(ExchangeError::Uncertain);}Ok(r.outputs)
 }
 pub(crate)fn capture_fenced(&mut self,output:&Output,current:&dyn Fn()->bool)->Result<Output,ExchangeError>{
  if self.captured.is_some()||!output.valid(){return Err(ExchangeError::NotStarted);}
  let r=self.exchange_checked("capture",Some(output),None,None,current)?;let o=r.captured.filter(|x|x.id==output.id&&x.serial==output.serial).ok_or(ExchangeError::Uncertain)?;self.captured=Some((o.id,o.serial.clone()));Ok(o)
 }
 pub(crate)fn read_fenced(&mut self,current:&dyn Fn()->bool)->Result<Output,ExchangeError>{
  let Some((id,serial))=self.captured.clone()else{return Err(ExchangeError::NotStarted)};
  self.exchange_checked("read",None,None,None,current)?.captured.filter(|o|o.id==id&&o.serial==serial).ok_or(ExchangeError::Uncertain)
 }
 pub(super)fn writer(&mut self,deadline:Instant,current:&dyn Fn()->bool)->Result<Self,ExchangeError>{
  let original=self.read_fenced(current)?;
  let mut writer=Self::start_at(deadline,self.authority.clone(),self.runtime.clone(),Mode::Effect,current)?;
  if writer.core_cookie!=self.core_cookie{return Err(ExchangeError::NotStarted);}writer.capture_fenced(&original,current)?;
  let now=self.read_fenced(current)?;if now.id!=original.id||now.serial!=original.serial{return Err(ExchangeError::NotStarted);}Ok(writer)
 }
 pub(crate)fn core_cookie(&self)->u32{self.core_cookie}
 pub(crate)fn outputs(&mut self)->Result<Vec<Output>,ExchangeError>{
  if self.captured.is_some(){return Err(ExchangeError::NotStarted);}
  let r=self.exchange("outputs",None)?;if r.captured.is_some(){self.retired.store(true,Ordering::Release);return Err(ExchangeError::Uncertain);}Ok(r.outputs)
 }
 pub(crate)fn capture(&mut self,output:&Output)->Result<Output,ExchangeError>{
  if self.captured.is_some()||!output.valid(){return Err(ExchangeError::NotStarted);}
  let r=self.exchange("capture",Some(output))?;let o=r.captured.filter(|x|x.id==output.id&&x.serial==output.serial).ok_or_else(||{self.retired.store(true,Ordering::Release);ExchangeError::Uncertain})?;
  self.captured=Some((o.id,o.serial.clone()));Ok(o)
 }
 pub(crate)fn read(&mut self)->Result<Output,ExchangeError>{
  let Some((id,serial))=self.captured.clone()else{return Err(ExchangeError::NotStarted)};
  let r=self.exchange("read",None)?;r.captured.filter(|o|o.id==id&&o.serial==serial).ok_or_else(||{self.retired.store(true,Ordering::Release);ExchangeError::Uncertain})
 }
}

/// No caller can select this driver mode from a wire request or inherited env.
/// The integration must supply original authority plus a durable effect fence.
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub(crate)enum NativeAction{Volume{percent:u8},Mute{muted:bool}}
impl NativeAction{
 fn valid(&self)->bool{!matches!(self,Self::Volume{percent}if *percent>100)}
 pub(crate)fn satisfied(&self,output:&Output)->bool{match self{Self::Volume{percent}=>output.volume_percent.is_some_and(|v|(v-f64::from(*percent)).abs()<0.6),Self::Mute{muted}=>output.muted==Some(*muted)}}
}
pub(crate)struct Dispatch{
 pub before:Option<Output>,pub after:Option<Output>,pub possible_effect:bool,pub transport_accepted:bool,pub scope_current:bool,pub verified:bool,
}
impl Reader{
 pub(crate)fn start_effect(deadline:Instant,authority:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self,ExchangeError>{
  if std::env::var_os("PIPEWIRE_REMOTE").is_some_and(|v|!v.is_empty()){return Err(ExchangeError::NotStarted);}
  let runtime=std::env::var_os("PIPEWIRE_RUNTIME_DIR").or_else(||std::env::var_os("XDG_RUNTIME_DIR")).unwrap_or_else(||format!("/run/user/{}",unsafe{libc::getuid()}).into());
  Self::start_at(deadline,authority,runtime,Mode::Effect,&||true)
 }
 /// Consume the native writer. It and all owned children are reaped inside
 /// dispatch_held, before its inherited and in-process physical holds close.
 pub(crate)fn dispatch(self,action:NativeAction,deadline:Instant,current:&dyn Fn()->bool,before_effect:impl FnOnce()->Result<(),()>)->Dispatch{
  let refusal=||Dispatch{before:None,after:None,possible_effect:false,transport_accepted:false,scope_current:false,verified:false};
  if !action.valid()||self.mode!=Mode::Effect||self.captured.is_none()||Instant::now()>=deadline||!current()||crate::settings::effect_allowed().is_err(){return refusal();}
  if self.native_guard.as_ref().is_none_or(|g|!g.current()){return refusal();}
  let authority=self.authority.clone();let deadline=deadline.min(self.deadline);
  let mut result=self.dispatch_held(action,deadline,current,before_effect);
  result.scope_current &= Instant::now()<deadline&&current()&&authority()&&crate::settings::effect_allowed().is_ok();result.verified &= result.scope_current;
  result
 }
 fn dispatch_held(mut self,action:NativeAction,deadline:Instant,current:&dyn Fn()->bool,before_effect:impl FnOnce()->Result<(),()>)->Dispatch{
  let mut result=Dispatch{before:None,after:None,possible_effect:false,transport_accepted:false,scope_current:false,verified:false};
  let deadline=deadline.min(self.deadline);let current=||Instant::now()<deadline&&current()&&crate::settings::effect_allowed().is_ok();
  let work=(||->Result<(),ExchangeError>{
   let native=self.exchange_checked("read",None,None,None,&current)?.captured.ok_or(ExchangeError::Uncertain)?;
   let expected=self.captured.as_ref().ok_or(ExchangeError::NotStarted)?;
   if (native.id,&native.serial)!=(expected.0,&expected.1){return Err(ExchangeError::Uncertain);}result.before=Some(native.clone());
   use ring::rand::{SecureRandom,SystemRandom};let mut bytes=[0;32];SystemRandom::new().fill(&mut bytes).map_err(|_|ExchangeError::NotStarted)?;
   let transaction:String=bytes.iter().map(|b|format!("{b:02x}")).collect();
   let prepared=self.exchange_checked("prepare",None,Some(&transaction),Some(&action),&current)?;
   if !prepared.prepared||prepared.accepted||prepared.transaction.as_deref()!=Some(transaction.as_str()){return Err(ExchangeError::Uncertain);}
   // This callback is the existing task intent/budget boundary, not an approval
   // reconstructed from a grant or from anything returned by this helper.
   if !current(){return Err(ExchangeError::NotStarted);}before_effect().map_err(|_|ExchangeError::NotStarted)?;
   if !current()||self.retired.load(Ordering::Acquire)||!(self.authority)(){return Err(ExchangeError::NotStarted);}
   result.possible_effect=true; // Any loss from this boundary is non-replayable.
   let accepted=self.exchange_checked("commit",None,Some(&transaction),None,&current);
   if matches!(accepted,Err(ExchangeError::NotStarted)){result.possible_effect=false;}
   result.transport_accepted=accepted.as_ref().is_ok_and(|r|r.accepted&&!r.prepared&&r.transaction.as_deref()==Some(transaction.as_str()));
   // A fresh observer never wrote a setter. Its cache starts with real native
   // node parameters, not the writer's optimistic local update.
   if !current()||!(self.authority)(){return Err(ExchangeError::Uncertain);}
   let mut observer=Self::start_at(deadline,self.authority.clone(),self.runtime.clone(),Mode::ReadOnly,&current)?;
   if observer.core_cookie!=self.core_cookie{return Err(ExchangeError::Uncertain);}
   let selected=observer.exchange_checked("capture",Some(&native),None,None,&current)?.captured.ok_or(ExchangeError::Uncertain)?;
   if selected.id!=native.id||selected.serial!=native.serial{return Err(ExchangeError::Uncertain);}observer.captured=Some((native.id,native.serial.clone()));
   let mut observed=selected;
   for _ in 0..8{
    result.after=Some(observed.clone());
    if action.satisfied(&observed){break;}
    if !current(){return Err(ExchangeError::Uncertain);}std::thread::sleep(Duration::from_millis(40));
    observed=observer.exchange_checked("read",None,None,None,&current)?.captured.filter(|v|v.id==native.id&&v.serial==native.serial).ok_or(ExchangeError::Uncertain)?;
   }
   // Revalidate the original connection after the independent observer too.
   let original=self.exchange_checked("read",None,None,None,&current)?.captured.ok_or(ExchangeError::Uncertain)?;
   if original.id!=native.id||original.serial!=native.serial{return Err(ExchangeError::Uncertain);}
   result.scope_current=current()&&(self.authority)()&&!self.retired.load(Ordering::Acquire);
   result.verified=result.scope_current&&result.after.as_ref().is_some_and(|v|action.satisfied(v));
   Ok(())
  })();let _=work;
  // Both observer and writer native families are reaped before dispatch returns
  // to its outer effect hold. There is no native worker left to flush late IO.
  result
 }
}
#[cfg(test)]#[path="pipewire_reader_tests.rs"]mod tests;
#[cfg(test)]#[path="pipewire_effect_tests.rs"]mod effect_tests;
#[cfg(test)]pub(crate)use effect_tests::Fixture as NativeFixture;

