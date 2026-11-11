//! Typed System output actions. Parsing is DATA only; native capture, task
//! approval and original budget/intent are still required before any dispatch.
use super::Snapshot;
pub(crate)use super::task_action::Action;
pub(crate)struct Observation{pub before:Option<Snapshot>,pub after:Option<Snapshot>,pub possible_effect:bool,pub acknowledged:bool,pub scope_current:bool,pub verified:bool}

/// Production commands stay fixed. Tests can retain a generated synthetic
/// process protocol per target; no environment/model/JSON can select it.
#[derive(Clone,Default)]pub(crate)struct Io{#[cfg(test)]fixture:Option<(std::path::PathBuf,std::path::PathBuf)>}
impl Io{
 pub(super)fn effect_runtime(&self)->std::path::PathBuf{
  #[cfg(test)]if let Some((_,state))=&self.fixture{if let Some(parent)=state.parent(){return parent.into();}}
  super::effect_lock::runtime()
 }
 pub(crate)fn command(&self,args:&[String])->std::process::Command{
  #[cfg(test)]if let Some((script,state))=&self.fixture{let mut input=vec!["-I".into(),script.to_str().unwrap().into(),state.to_str().unwrap().into()];input.extend_from_slice(args);return crate::pet_helper::command("/usr/bin/python3",&input);}
  crate::pet_helper::command("/usr/bin/pactl",args)
 }
 #[cfg(test)]pub(crate)fn fixture(script:std::path::PathBuf,state:std::path::PathBuf)->Self{Self{fixture:Some((script,state))}}
}

/// Typed native execution is available to an original admitted live task.
/// This transport gate grants no identity, policy, model catalog or permission.
pub(crate)fn native_dispatch_available()->bool{true}
