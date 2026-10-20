use std::{path::PathBuf,time::Instant};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};

pub const MAX_CALLS:usize=8;
pub const MAX_CALLS_PER_ROUND:usize=4;
pub const MAX_PROPOSAL:usize=8*1024;
pub const MAX_STREAM_OUTPUT:usize=128*1024;
pub const MAX_MODEL_RESULT:usize=16*1024;
pub const MAX_TURN_MODEL_RESULT:usize=64*1024;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum ReceiptState {Proposed,Starting,Exited,Denied,Expired,FailedToStart,TimedOut,OutputLimit,Cancelled,Skipped,Unknown}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum ReceiptCause {UserCancel,ApprovalDeadline,TaskDeadline,CommandDeadline,OutputBudget,PreviousDenied,PolicyRevoked,SpawnError,CrashDuringStart,CleanupUnconfirmed,ProcessExit}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct StepReceipt {
 pub step_id:String,pub ordinal:u8,pub state:ReceiptState,
 #[serde(default,skip_serializing_if="Option::is_none")]pub cause:Option<ReceiptCause>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub trigger:Option<ReceiptCause>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub started_at:Option<u64>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub finished_at:Option<u64>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub exit_code:Option<i32>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub signal:Option<i32>,
 #[serde(default)]pub stdout_bytes:u64,#[serde(default)]pub stderr_bytes:u64,
 #[serde(default)]pub truncated:bool,
}
pub fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64}
impl StepReceipt {
 pub fn proposed(ordinal:u8)->Self{Self{step_id:format!("step-{ordinal}"),ordinal,state:ReceiptState::Proposed,cause:None,trigger:None,started_at:None,finished_at:None,exit_code:None,signal:None,stdout_bytes:0,stderr_bytes:0,truncated:false}}
 pub fn finish(&mut self,state:ReceiptState,cause:Option<ReceiptCause>){self.state=state;self.cause=cause;self.finished_at=Some(now());}
 pub fn valid(&self)->bool{self.ordinal>0&&self.ordinal<=MAX_CALLS as u8&&self.step_id==format!("step-{}",self.ordinal)&&self.stdout_bytes<=MAX_STREAM_OUTPUT as u64&&self.stderr_bytes<=MAX_STREAM_OUTPUT as u64}
 pub fn recover(&mut self){if self.state==ReceiptState::Starting{self.state=ReceiptState::Unknown;self.cause=Some(ReceiptCause::CrashDuringStart);self.finished_at=None;}}
}

#[derive(Clone,Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CommandInput {pub program:String,pub args:Vec<String>,pub reason:String,#[serde(default)]pub timeout_ms:Option<u64>}
/// Constructed and frozen before asking the human. Not deserialized from IPC.
pub struct CommandSpec {
 pub executable:PathBuf,pub args:Vec<String>,pub cwd:PathBuf,pub reason:String,pub timeout_ms:u64,
 pub(crate) directory:std::fs::File,
 pub(crate) executable_stamp:ExecutableStamp,
}
#[derive(PartialEq,Eq)]pub(crate) struct ExecutableStamp {pub dev:u64,pub ino:u64,pub len:u64,pub modified:i128,pub mode:u32}
impl CommandSpec {
 pub fn display(&self)->Value{json!({"program":self.executable,"args":self.args,"cwd":self.cwd,"timeoutMs":self.timeout_ms,"reason":self.reason,"scope":"once"})}
}
#[derive(Clone)]pub struct ExecutionScope {
 pub target:crate::session_chat::types::Target,
 pub turn:crate::chat::Turn,pub task_id:String,pub model:String,pub base:reqwest::Url,
 pub deadline:Instant,pub policy:crate::pet_config::LocalExecution,
 pub reasoning_effort:Option<String>,
}
impl ExecutionScope{pub fn valid_target(&self)->bool{self.turn.provider=="ollama"&&self.target.provider=="ollama"&&self.target.pet_id=="ai_ollama"&&self.target.origin==crate::session_chat::types::Origin::Mochi&&self.target.conversation_id.as_deref()==Some(self.target.session_id.as_str())}}
/// Private, non-cloneable capability: only the approval module can mint it.
pub struct ApprovedCommand {spec:CommandSpec,task_id:String,step_id:String}
impl ApprovedCommand {
 pub(super) fn new(spec:CommandSpec,task_id:&str,step_id:&str)->Self{Self{spec,task_id:task_id.into(),step_id:step_id.into()}}
 pub(crate) fn belongs_to(&self,task_id:&str,step_id:&str)->bool{self.task_id==task_id&&self.step_id==step_id}
 pub(crate) fn into_spec(self)->CommandSpec{self.spec}
 #[cfg(test)]pub(crate) fn for_test(spec:CommandSpec,task_id:&str,step_id:&str)->Self{Self::new(spec,task_id,step_id)}
}
/// Only Chat's checked step admission creates this capability.
pub struct RunPermit {pub(super) spec:CommandSpec}
impl RunPermit {pub(crate) fn after_checked_admission(command:ApprovedCommand)->Self{Self{spec:command.into_spec()}}}
pub struct TurnOutputBudget {pub raw_remaining:usize,pub model_json_remaining:usize}
impl TurnOutputBudget {pub fn new(raw:usize)->Self{Self{raw_remaining:raw,model_json_remaining:MAX_TURN_MODEL_RESULT}}pub fn can_run(&self)->bool{self.raw_remaining>0&&self.model_json_remaining>=1024}}

pub struct ProcessResult {pub receipt:StepReceipt,pub stdout:Vec<u8>,pub stderr:Vec<u8>}
impl ProcessResult {
 /// Count the complete serialized tool message, including escaping and metadata.
 pub fn model_message(&self,call_id:&str,budget:&mut TurnOutputBudget)->Result<Value,String>{
  let cap=MAX_MODEL_RESULT.min(budget.model_json_remaining);
  let mut bytes=(self.stdout.len()+self.stderr.len()).min(cap);
  loop {
   let out=bytes.min(self.stdout.len());let err=(bytes-out).min(self.stderr.len());
   let content=json!({"receipt":self.receipt,"stdout":String::from_utf8_lossy(&self.stdout[..out]),"stderr":String::from_utf8_lossy(&self.stderr[..err]),"truncatedForModel":out<self.stdout.len()||err<self.stderr.len()}).to_string();
   let message=json!({"role":"tool","tool_call_id":call_id,"content":content});
   let n=serde_json::to_vec(&message).map_err(|e|e.to_string())?.len();
   if n<=cap{budget.model_json_remaining-=n;return Ok(message);}
   if bytes==0{return Err("O limite de resultados do modelo foi atingido.".into());}bytes/=2;
  }
 }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum LocalTerminal {Done,Failed,Interrupted,Stopped}
impl LocalTerminal {pub fn phase(self)->&'static str{match self{Self::Done=>"done",Self::Failed=>"failed",Self::Interrupted=>"interrupted",Self::Stopped=>"stopped"}}}
pub struct LocalCommit {pub origin:String,pub phase:LocalTerminal,pub durable:bool,pub text:String}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn recovery_never_mints_authorization_or_replays_started_steps(){let mut r=StepReceipt::proposed(1);r.state=ReceiptState::Starting;r.started_at=Some(1);r.recover();assert_eq!(r.state,ReceiptState::Unknown);assert_eq!(r.cause,Some(ReceiptCause::CrashDuringStart));assert!(r.finished_at.is_none());}
 #[test]fn serialized_results_share_the_turn_budget_and_bound_escaping(){let r=ProcessResult{receipt:StepReceipt::proposed(1),stdout:vec![b'"';128*1024],stderr:vec![b'\\';128*1024]};let mut b=TurnOutputBudget::new(1);let first=r.model_message("one",&mut b).unwrap();let n=serde_json::to_vec(&first).unwrap().len();assert!(n<=MAX_MODEL_RESULT);assert_eq!(b.model_json_remaining,MAX_TURN_MODEL_RESULT-n);assert!(first["content"].as_str().unwrap().contains("truncatedForModel"));}
 #[test]fn command_schema_cannot_choose_scope_environment_or_cwd(){assert!(serde_json::from_value::<CommandInput>(json!({"program":"x","args":[],"reason":"r","cwd":"/"})).is_err());}
 #[test]fn every_receipt_state_and_cause_round_trips_without_command_data(){for state in [ReceiptState::Proposed,ReceiptState::Starting,ReceiptState::Exited,ReceiptState::Denied,ReceiptState::Expired,ReceiptState::FailedToStart,ReceiptState::TimedOut,ReceiptState::OutputLimit,ReceiptState::Cancelled,ReceiptState::Skipped,ReceiptState::Unknown]{for cause in [ReceiptCause::UserCancel,ReceiptCause::ApprovalDeadline,ReceiptCause::TaskDeadline,ReceiptCause::CommandDeadline,ReceiptCause::OutputBudget,ReceiptCause::PreviousDenied,ReceiptCause::PolicyRevoked,ReceiptCause::SpawnError,ReceiptCause::CrashDuringStart,ReceiptCause::CleanupUnconfirmed,ReceiptCause::ProcessExit]{let mut r=StepReceipt::proposed(1);r.state=state;r.cause=Some(cause);let value=serde_json::to_value(&r).unwrap();assert!(value.get("argv").is_none());let back:StepReceipt=serde_json::from_value(value).unwrap();assert_eq!(back.state,state);assert_eq!(back.cause,Some(cause));assert!(back.valid());}}}
}
