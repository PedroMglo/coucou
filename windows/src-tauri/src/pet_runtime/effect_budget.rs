//! Authenticated accounting data, never an execution capability. Unknown IO
//! retains its full output reservation and retries never renew call budgets.
use super::{admission::AdmissionSpec,model};
use serde::{Deserialize,Serialize};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super)enum Kind{Model,Tool}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super)enum Phase{Reserved,Started,Settled,Uncertain}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Attempt{
 pub id:String,pub request_id:String,pub input_hash:String,pub kind:Kind,pub output_reserved:u64,
 pub phase:Phase,pub accepted_output:Option<u64>,pub reserved_at:u64,pub started_at:Option<u64>,pub finished_at:Option<u64>,
}
impl Attempt{
 pub(super)fn valid(&self)->bool{
  model::token(&self.id)&&model::key(&self.request_id)&&super::authority::decode(&self.input_hash).is_some()
  &&(1..=16*1024*1024).contains(&self.output_reserved)&&self.reserved_at>0&&self.reserved_at<=8_640_000_000_000_000
  &&self.started_at.is_none_or(|n|n>=self.reserved_at&&n<=8_640_000_000_000_000)
  &&self.finished_at.is_none_or(|n|n>=self.started_at.unwrap_or(self.reserved_at)&&n<=8_640_000_000_000_000)
  &&match self.phase{
   Phase::Reserved=>self.accepted_output.is_none()&&self.started_at.is_none()&&self.finished_at.is_none(),
   Phase::Started=>self.accepted_output.is_none()&&self.started_at.is_some()&&self.finished_at.is_none(),
   Phase::Settled=>self.accepted_output.is_some_and(|n|n<=self.output_reserved)&&self.started_at.is_some()&&self.finished_at.is_some(),
   Phase::Uncertain=>self.accepted_output.is_none()&&self.started_at.is_some()&&self.finished_at.is_some(),
  }
 }
 pub(super)fn charge(&self)->u64{self.accepted_output.unwrap_or(self.output_reserved)}
 pub(super)fn same_request(&self,other:&Self)->bool{self.id==other.id&&self.request_id==other.request_id&&self.input_hash==other.input_hash&&self.kind==other.kind&&self.output_reserved==other.output_reserved&&self.reserved_at==other.reserved_at}
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Ledger{pub schema_version:u32,pub attempts:Vec<Attempt>}
impl Ledger{
 pub(super)fn empty()->Self{Self{schema_version:1,attempts:vec![]}}
 pub(super)fn valid(&self)->bool{
  let mut ids=std::collections::BTreeSet::new();let mut requests=std::collections::BTreeSet::new();
  self.schema_version==1&&self.attempts.len()<=192&&self.attempts.iter().all(|a|a.valid()&&ids.insert(&a.id)&&requests.insert(&a.request_id))
  &&self.attempts.iter().filter(|a|a.kind==Kind::Model).count()<=64&&self.attempts.iter().filter(|a|a.kind==Kind::Tool).count()<=128
  &&self.attempts.iter().map(Attempt::charge).sum::<u64>()<=16*1024*1024
 }
 pub(super)fn valid_for(&self,spec:&AdmissionSpec)->bool{
  self.valid()&&self.attempts.iter().filter(|a|a.kind==Kind::Model).count()<=usize::from(spec.budget.max_model_calls)
  &&self.attempts.iter().filter(|a|a.kind==Kind::Tool).count()<=usize::from(spec.budget.max_tool_calls)
  &&self.attempts.iter().map(Attempt::charge).sum::<u64>()<=spec.budget.max_output_bytes
  &&self.attempts.iter().all(|a|a.reserved_at>=spec.approved_at&&a.reserved_at<spec.budget.deadline_at&&a.started_at.is_none_or(|n|n<spec.budget.deadline_at))
 }
}
