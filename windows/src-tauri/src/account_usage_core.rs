//! Server-reported account activity. No credentials, IO, estimates or timers.
use serde::Serialize;
use serde_json::Value;
use std::time::{Duration,Instant};

const TTL:u64=60;
const COOLDOWN:Duration=Duration::from_secs(5);
const MAX_TIME:u64=9_007_199_254_740_991;

#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Summary {
 pub lifetime_tokens:Option<String>,
 pub peak_daily_tokens:Option<String>,
 pub longest_running_turn_sec:Option<String>,
 pub current_streak_days:Option<String>,
 pub longest_streak_days:Option<String>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum Failure {NotObserved,NativeUnavailable,NotSignedIn,InvalidData,ClockUnavailable,UnsupportedPlatform}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum RefreshError {Busy,Cooldown,Unavailable}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {
 pub schema_version:u8,pub provider:&'static str,pub source:&'static str,
 pub source_instance:Option<String>,pub revision:String,
 pub scope:&'static str,pub status:&'static str,
 pub observed_at:Option<u64>,pub read_at:Option<u64>,
 pub stale:bool,pub refreshing:bool,
 pub summary:Option<Summary>,pub reason:Option<Failure>,
}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Changed {
 pub schema_version:u8,pub provider:&'static str,pub source:&'static str,
 pub source_instance:String,pub revision:String,
}
impl Snapshot {
 pub fn changed(&self)->Option<Changed>{Some(Changed{schema_version:1,provider:"codex",
  source:"codexAppServerAccountUsage",source_instance:self.source_instance.clone()?,revision:self.revision.clone()})}
}
fn metric(value:Option<&Value>)->Result<Option<String>,Failure>{
 match value {
  None|Some(Value::Null)=>Ok(None),
  Some(value)=>value.as_i64().filter(|n|*n>=0).map(|n|Some(n.to_string())).ok_or(Failure::InvalidData),
 }
}
pub fn parse(value:&Value)->Result<Summary,Failure>{
 let root=value.as_object().ok_or(Failure::InvalidData)?;
 if root.contains_key("error"){return Err(Failure::InvalidData);}
 let v=root.get("summary").and_then(Value::as_object).ok_or(Failure::InvalidData)?;
 Ok(Summary{
  lifetime_tokens:metric(v.get("lifetimeTokens"))?,
  peak_daily_tokens:metric(v.get("peakDailyTokens"))?,
  longest_running_turn_sec:metric(v.get("longestRunningTurnSec"))?,
  current_streak_days:metric(v.get("currentStreakDays"))?,
  longest_streak_days:metric(v.get("longestStreakDays"))?,
 })
}
pub fn valid_time(at:u64)->bool{at>0&&at<=MAX_TIME}

pub struct Cache {
 source_instance:Option<String>,revision:u64,
 next:u64,pending:Option<u64>,attempt:Option<Instant>,
 summary:Option<Summary>,observed_at:Option<u64>,reason:Option<Failure>,
}
impl Default for Cache {
 fn default()->Self{Self::with_instance(None)}
}
impl Cache {
 pub fn with_instance(instance:Option<String>)->Self{
  let source_instance=instance.filter(|s|s.len()==32&&s.bytes().all(|c|c.is_ascii_digit()||(b'a'..=b'f').contains(&c)));
  let reason=if source_instance.is_some(){Failure::NotObserved}else{Failure::NativeUnavailable};
  Self{source_instance,revision:0,next:0,pending:None,attempt:None,summary:None,observed_at:None,reason:Some(reason)}
 }
 pub fn begin(&mut self,now:Instant)->Result<u64,RefreshError>{
  if self.source_instance.is_none(){return Err(RefreshError::Unavailable);}
  if self.pending.is_some(){return Err(RefreshError::Busy);}
  if self.attempt.is_some_and(|at|now.saturating_duration_since(at)<COOLDOWN){return Err(RefreshError::Cooldown);}
  // Reserve both transition ordinals before any physical work is admitted.
  self.revision.checked_add(2).ok_or(RefreshError::Unavailable)?;
  let ticket=self.next.checked_add(1).ok_or(RefreshError::Unavailable)?;
  self.next=ticket;self.revision+=1;self.pending=Some(ticket);self.attempt=Some(now);
  // A new native login may refer to another account. Never retain prior data
  // as the result of the new refresh, including a failed reauthentication.
  self.summary=None;self.observed_at=None;self.reason=None;
  Ok(ticket)
 }
 pub fn finish(&mut self,ticket:u64,result:Result<Summary,Failure>,at:u64,now:Instant)->bool{
  if self.pending!=Some(ticket){return false;}
  // begin reserved this ordinal. No other mutation can advance it in flight.
  self.revision+=1;self.pending=None;self.attempt=Some(now);
  match result {
   Ok(summary)if valid_time(at)=>{self.summary=Some(summary);self.observed_at=Some(at);self.reason=None;}
   Ok(_)=>{self.summary=None;self.observed_at=None;self.reason=Some(Failure::ClockUnavailable);}
   Err(reason)=>{self.summary=None;self.observed_at=None;self.reason=Some(reason);}
  }true
 }
 pub fn snapshot(&self,now:u64)->Snapshot{
  let clock_ok=valid_time(now);
  let stale=!clock_ok||self.observed_at.is_none_or(|at|at>now||now-at>TTL);
  Snapshot{schema_version:1,provider:"codex",source:"codexAppServerAccountUsage",
   source_instance:self.source_instance.clone(),revision:self.revision.to_string(),
   scope:"lastObservedCodexAccount",status:if self.summary.is_some(){"available"}else{"unavailable"},
   observed_at:self.observed_at,read_at:clock_ok.then_some(now),stale,
   refreshing:self.pending.is_some(),summary:self.summary.clone(),
   reason:if clock_ok{self.reason}else{Some(Failure::ClockUnavailable)}}
 }
}

#[cfg(test)]
#[path="account_usage_tests.rs"]
mod tests;
