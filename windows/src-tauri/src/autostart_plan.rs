//! Frozen Linux registration resource. Only the controller owns authority;
//! this executor neither grants reactivation nor chooses a target after waits.
use crate::private_registry::{file_target::Target,preferences_file::Witness};
use serde::{Serialize,Deserialize};
use std::{fs::{File,OpenOptions},io,os::unix::fs::{MetadataExt,OpenOptionsExt},path::{Path,PathBuf},sync::Arc};
fn error()->io::Error{io::Error::other("O launcher ou o destino do registo de arranque mudou. É necessária outra revisão.")}
fn hash(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn token(m:&std::fs::Metadata)->String{hash(format!("{:?}",(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode())).as_bytes())}
#[derive(Clone)]struct Launcher{path:PathBuf,file:Arc<File>,identity:String}
impl Launcher{
 fn capture(path:&Path)->io::Result<Self>{
  if !path.is_absolute(){return Err(error());}
  let file=OpenOptions::new().read(true).custom_flags(libc::O_PATH|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path)?;let m=file.metadata()?;
  if !m.is_file()||m.uid()!=unsafe{libc::geteuid()}||m.mode()&0o7022!=0||m.mode()&0o111==0{return Err(error());}
  let value=Self{path:path.into(),file:Arc::new(file),identity:token(&m)};if !value.current(){return Err(error());}Ok(value)
 }
 fn current(&self)->bool{self.file.metadata().is_ok_and(|m|token(&m)==self.identity)&&std::fs::symlink_metadata(&self.path).is_ok_and(|m|m.is_file()&&token(&m)==self.identity)}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct View{pub schema_version:u32,pub component:String,pub enabled:bool,pub target:String,pub resource:String,pub launcher:String,pub launcher_identity:String,pub before_revision:String,pub before_present:bool,pub creates_parent:bool,pub expected_sha256:Option<String>,pub diff:String,pub postcondition:String,pub fingerprint:String}
#[derive(Clone)]pub(crate)struct Plan{target:Target,launcher:Launcher,expected:Option<Vec<u8>>,pub view:View}
impl Plan{
 pub(crate)fn capture(config:&Path,launcher:&Path,enabled:bool)->io::Result<Self>{
  let launcher=Launcher::capture(launcher)?;let target=Target::capture(config,"autostart","Coucou.desktop",16*1024)?;
  let expected=if enabled{Some(super::linux::entry(&launcher.path)?.into_bytes())}else{None};
  let before=target.before().map(|b|String::from_utf8_lossy(b).into_owned()).unwrap_or_default();let after=expected.as_deref().map(|b|String::from_utf8_lossy(b).into_owned()).unwrap_or_default();
  let mut view=View{schema_version:2,component:"autostart".into(),enabled,target:target.path().to_string_lossy().into_owned(),resource:target.resource().into(),launcher:launcher.path.to_string_lossy().into_owned(),launcher_identity:launcher.identity.clone(),before_revision:target.revision(),before_present:target.before().is_some(),creates_parent:enabled&&!target.parent_present(),expected_sha256:expected.as_deref().map(hash),diff:crate::config_file::unified_diff(&before,&after),postcondition:if enabled{"desktopEntryMatches"}else{"desktopEntryAbsent"}.into(),fingerprint:String::new()};
  view.fingerprint=hash(&serde_json::to_vec(&view).map_err(|_|error())?);let value=Self{target,launcher,expected,view};if !value.current(){return Err(error());}Ok(value)
 }
 pub(crate)fn current(&self)->bool{self.target.current()&&self.launcher.current()}
 pub(crate)fn original(&self)->Option<&[u8]>{self.target.before()}
 pub(crate)fn expected(&self)->Option<&[u8]>{self.expected.as_deref()}
 pub(crate)fn already_matches(&self)->bool{self.target.before()==self.expected.as_deref()}
 pub(crate)fn fence(&self,guard:impl Fn()->io::Result<()>)->io::Result<crate::private_registry::file_target::Fence>{self.target.fence(guard)}
 /// The caller has durably marked dispatch before entering this executor.
 /// Parent creation and file publication are separate possible effects.
 pub(crate)fn execute(&self,guard:impl Fn()->io::Result<()>)->io::Result<Observation>{self.execute_with(guard,||{})}
 fn execute_with(&self,guard:impl Fn()->io::Result<()>,after:impl FnOnce())->io::Result<Observation>{
  guard()?;if !self.current(){return Err(error());}
  if self.already_matches(){after();let observed=self.observe()?;return if self.current(){Ok(observed)}else{Err(error())};}
  let resolved=self.target.materialize(self.view.enabled,||{guard()?;if self.launcher.current(){Ok(())}else{Err(error())}})?;
  let witness=match &self.expected{Some(bytes)=>Some(resolved.write(bytes,||{guard()?;if self.launcher.current(){Ok(())}else{Err(error())}})?),None=>resolved.remove(||{guard()?;if self.launcher.current(){Ok(())}else{Err(error())}})?};
  after();let pinned=||self.target.anchor_current()&&witness.as_ref().is_none_or(Witness::current);
  if !pinned(){return Err(error());}let observed=self.observe()?;
  if !pinned()||observed.file_identity!=witness.as_ref().and_then(Witness::identity){return Err(error());}Ok(observed)
 }
 pub(crate)fn observe(&self)->io::Result<Observation>{
  if !self.launcher.current()||!self.target.anchor_current(){return Err(error());}
  let parent=self.target.path().parent().and_then(Path::parent).ok_or_else(error)?;
  let actual=Target::capture(parent,"autostart","Coucou.desktop",16*1024)?;
  if actual.resource()!=self.target.resource()||actual.before()!=self.expected.as_deref(){return Err(error());}
  let identity=if actual.parent_present(){let(w,_)=crate::private_registry::preferences_file::read(actual.path(),false)?;w.identity()}else{None};
  if !self.launcher.current()||!self.target.anchor_current()||!actual.current(){return Err(error());}
  Ok(Observation{resource:self.view.resource.clone(),target:self.view.target.clone(),enabled:self.view.enabled,expected_sha256:self.view.expected_sha256.clone(),file_identity:identity,launcher_identity:self.launcher.identity.clone(),postcondition:self.view.postcondition.clone(),login_verified:false})
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Observation{pub resource:String,pub target:String,pub enabled:bool,pub expected_sha256:Option<String>,pub file_identity:Option<String>,pub launcher_identity:String,pub postcondition:String,pub login_verified:bool}
/// Native environment is read once, before a plan is admitted or waits.
pub(crate)fn native_plan(enabled:bool)->io::Result<Plan>{
 let home=crate::platform::home_dir();let exe=std::env::current_exe()?;let uid=unsafe{libc::geteuid()};
 let launcher=if super::linux::managed(&home,&exe,env!("CARGO_PKG_VERSION"),uid){home.join(".local/bin/coucou")}else{exe};
 let config=std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(||home.join(".config"));Plan::capture(&config,&launcher,enabled)
}
#[cfg(test)]#[path="autostart_plan_tests.rs"]mod tests;
