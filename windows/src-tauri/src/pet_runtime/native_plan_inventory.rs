//! Bounded, passive projection of signed native plan facts. No authority or
//! worker is created by listing a reviewed revision or a staged withdrawal.
use super::{authority::Failure,native_plan_revisions::Reference};
use serde::Serialize;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum State{Staged,Sealed,Published,WithdrawalPending,Abandoned,Blocked}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Item{pub reference:Reference,pub state:State,pub pending_artifacts:bool,pub can_ack:bool,pub can_abandon:bool,pub reason:Option<Failure>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Inventory{pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub revision:String,pub source_hash:String,pub observed_at:u64,pub coverage:Option<super::continuation::NativeCarry>,pub fact_coverage_complete:bool,pub items:Vec<Item>,pub has_more:bool,pub next_cursor:Option<String>,pub execution_authorized:bool,pub repair_performed:bool}
