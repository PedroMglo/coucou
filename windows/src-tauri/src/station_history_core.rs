//! Aggregate the existing station inventory without exporting paths or transcripts.
use std::{collections::BTreeMap,time::{Duration,Instant}};
use serde::Serialize;
use serde_json::Value;
const MAX_SAFE:u64=9_007_199_254_740_991;
const MAX_ROWS:usize=100_000;
#[derive(Clone,Debug,Serialize,PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Metric {pub value:Option<u64>,pub observed:u64,pub coverage:&'static str,pub scope:&'static str}
fn unknown(scope:&'static str)->Metric {Metric{value:None,observed:0,coverage:"unavailable",scope}}
#[derive(Clone,Debug,Serialize,PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Provider {pub provider:String,pub indexed_total:Option<u64>,pub known:Metric,pub archived:Metric}
#[derive(Clone,Debug,PartialEq)]
pub struct Collection {pub observed_at:u64,pub providers:Vec<Provider>}
fn count(v:&Value)->Option<u64>{v.as_u64().filter(|n|*n<=MAX_SAFE)}
fn native_id(id:&str)->Option<String>{
 if id.len()!=36||!id.bytes().enumerate().all(|(i,c)|if [8,13,18,23].contains(&i){c==b'-'}else{c.is_ascii_hexdigit()}){None}else{Some(id.to_ascii_lowercase())}
}
/// Counts come from the local index or distinct validated metadata IDs. A
/// process placeholder, loaded state or default archived:false proves neither
/// another historical session nor a measured archive total.
pub fn collect(value:&Value,now:u64)->Result<Collection,()>{
 if value.get("error").is_some(){return Err(());}
 let observed_at=value.get("updated").and_then(count).filter(|n|*n>0&&*n<=now).ok_or(())?;
 let providers=value.get("providers").and_then(Value::as_array).ok_or(())?;
 let rows=value.get("sessions").and_then(Value::as_array).filter(|r|r.len()<=MAX_ROWS).ok_or(())?;
 if providers.len()>32{return Err(());}
 let mut out=Vec::new();
 for provider in ["codex","copilot"]{
  let items:Vec<_>=providers.iter().filter(|p|p.get("id").and_then(Value::as_str)==Some(provider)).collect();
  if items.len()>1{return Err(());}
  let reported=items.first().and_then(|p|p.get("history")).and_then(count);
  let installed=items.first().and_then(|p|p.get("installed")).and_then(Value::as_bool)==Some(true);
  // Codex is skipped entirely by the legacy helper when its CLI is absent;
  // the resulting synthetic history:0 is not an index query. Copilot reads
  // its DB plus legacy workspace catalogue independently of CLI installation,
  // but an empty/missing catalogue has no distinguishable zero provenance.
  let total=if provider=="codex"{reported.filter(|_|installed)}else{reported.filter(|n|*n>0)};
  let indexed_total=if provider=="codex"{total}else{None};
  if items.first().and_then(|p|p.get("history")).is_some_and(|v|!v.is_null()&&count(v).is_none()){return Err(());}
  let mut ids=BTreeMap::new();
  for row in rows.iter().filter(|r|r.get("provider").and_then(Value::as_str)==Some(provider)){
   let raw=row.get("id").and_then(Value::as_str).ok_or(())?;
   if raw.starts_with("proc:"){continue;}
   let id=native_id(raw).ok_or(())?;
   let archived=row.get("archived").and_then(Value::as_bool);
   if let Some(previous)=ids.insert(id,archived){if previous!=archived{return Err(());}}
  }
  let observed=ids.len() as u64;
  let known=match total{
   Some(n) if n>=observed=>Metric{value:Some(n),observed,coverage:if provider=="codex"{"completeWithinIndex"}else{"completeWithinCatalogue"},scope:if provider=="codex"{"localIndex"}else{"localCatalogue"}},
   _ if observed>0=>Metric{value:None,observed,coverage:"partial",scope:"localMetadataUnion"},
   _=>unknown("localMetadataUnion"),
  };
  let archived_observed=ids.values().filter(|v|**v==Some(true)).count() as u64;
  let archived=if provider=="codex"&&archived_observed>0{
   Metric{value:None,observed:archived_observed,coverage:"partial",scope:"archiveMetadata"}
  }else if provider=="codex"&&indexed_total==Some(0)&&observed==0{
   Metric{value:Some(0),observed:0,coverage:"completeWithinIndex",scope:"localIndex"}
  }else{unknown("archiveMetadata")};
  out.push(Provider{provider:provider.into(),indexed_total,known,archived});
 }
 Ok(Collection{observed_at,providers:out})
}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {
 pub schema_version:u32,pub source:&'static str,pub status:&'static str,
 pub observed_at:Option<u64>,pub read_at:u64,pub refreshing:bool,pub stale:bool,
 pub reason:Option<&'static str>,pub providers:Vec<Provider>,
}
#[derive(Default)]
pub struct Cache {last:Option<Collection>,attempt:Option<Instant>,reading:bool,generation:u64,error:Option<&'static str>}
impl Cache {
 pub fn begin(&mut self,now:Instant,force:bool)->Option<u64>{
  let delay=Duration::from_secs(if force{2}else{15});
  if self.reading||self.attempt.is_some_and(|at|now.saturating_duration_since(at)<delay){return None;}
  self.generation=self.generation.checked_add(1)?;self.reading=true;self.attempt=Some(now);Some(self.generation)
 }
 pub fn finish(&mut self,ticket:u64,result:Result<Collection,&'static str>,now:Instant){
  if ticket!=self.generation||!self.reading{return;}
  self.reading=false;self.attempt=Some(now);
  match result{Ok(v)=>{self.last=Some(v);self.error=None},Err(e)=>self.error=Some(e)}
 }
 pub fn snapshot(&self,now:u64)->Snapshot{
  let observed_at=self.last.as_ref().map(|v|v.observed_at);
  let stale=self.error.is_some()||observed_at.is_none_or(|at|at>now||now-at>30);
  Snapshot{schema_version:1,source:"station-management-inventory",status:if self.last.is_some(){"available"}else{"unavailable"},observed_at,read_at:now,refreshing:self.reading,stale,reason:self.error.or_else(||self.last.is_none().then_some("notObserved")),providers:self.last.as_ref().map(|v|v.providers.clone()).unwrap_or_default()}
 }
}
#[cfg(test)]
mod tests{
 use super::*;use serde_json::json;
 const A:&str="11111111-1111-1111-1111-111111111111";
 const B:&str="22222222-2222-2222-2222-222222222222";
 fn fixture()->Value{json!({"updated":100,"providers":[{"id":"codex","history":1740,"installed":true},{"id":"copilot","history":33,"installed":true}],"sessions":[{"provider":"codex","id":A,"archived":true,"secret":"PRIVATE"},{"provider":"codex","id":B,"archived":false},{"provider":"copilot","id":A,"archived":false}]})}
 #[test]fn genuine_index_totals_do_not_become_row_count_or_open_clients(){
  let c=collect(&fixture(),100).unwrap();assert_eq!(c.providers[0].known.value,Some(1740));assert_eq!(c.providers[0].known.scope,"localIndex");assert_eq!(c.providers[0].known.observed,2);assert_eq!(c.providers[1].known.value,Some(33));
 }
 #[test]fn archive_lower_bound_is_distinct_from_unknown_copilot_default_false(){
  let c=collect(&fixture(),100).unwrap();assert_eq!(c.providers[0].archived.value,None);assert_eq!(c.providers[0].archived.observed,1);assert_eq!(c.providers[0].archived.coverage,"partial");assert_eq!(c.providers[1].archived.coverage,"unavailable");assert_eq!(c.providers[1].archived.value,None);
 }
 #[test]fn larger_union_uses_positive_lower_bound_not_a_smaller_index_or_a_sum(){
  let mut v=fixture();v["providers"][0]["history"]=json!(1);let c=collect(&v,100).unwrap();assert_eq!(c.providers[0].known.value,None);assert_eq!(c.providers[0].known.observed,2);assert_eq!(c.providers[0].known.scope,"localMetadataUnion");
 }
 #[test]fn native_ids_deduplicate_but_process_placeholders_are_not_history(){
  let mut v=fixture();v["sessions"].as_array_mut().unwrap().extend([json!({"provider":"codex","id":A,"archived":true}),json!({"provider":"codex","id":"proc:123:45"})]);assert_eq!(collect(&v,100).unwrap().providers[0].known.observed,2);
  v["sessions"].as_array_mut().unwrap().push(json!({"provider":"codex","id":A,"archived":false}));assert!(collect(&v,100).is_err());
 }
 #[test]fn empty_index_zero_is_scoped_not_fabricated_missing_history(){
  let c=collect(&json!({"updated":100,"providers":[{"id":"codex","history":0,"installed":true},{"id":"copilot","history":null}],"sessions":[]}),100).unwrap();assert_eq!(c.providers[0].known.value,Some(0));assert_eq!(c.providers[0].archived.value,Some(0));assert_eq!(c.providers[1].known.coverage,"unavailable");
 }
 #[test]fn missing_codex_cli_zero_is_not_an_observed_index_or_archive_zero(){
  for installed in [serde_json::Value::Null,json!(false)]{let c=collect(&json!({"updated":100,"providers":[{"id":"codex","history":0,"installed":installed}],"sessions":[]}),100).unwrap();assert_eq!(c.providers[0].indexed_total,None);assert_eq!(c.providers[0].known.coverage,"unavailable");assert_eq!(c.providers[0].archived.value,None);}
 }
 #[test]fn copilot_catalogue_includes_legacy_metadata_and_never_claims_an_empty_index(){
  let mut v=fixture();v["providers"][1]["installed"]=json!(false);let c=collect(&v,100).unwrap();assert_eq!(c.providers[1].indexed_total,None);assert_eq!(c.providers[1].known.value,Some(33));assert_eq!(c.providers[1].known.scope,"localCatalogue");assert_eq!(c.providers[1].known.coverage,"completeWithinCatalogue");
  let c=collect(&json!({"updated":100,"providers":[{"id":"copilot","history":0,"installed":true}],"sessions":[]}),100).unwrap();assert_eq!(c.providers[1].known.value,None);assert_eq!(c.providers[1].known.coverage,"unavailable");
 }
 #[test]fn malformed_identity_time_count_and_duplicate_provider_fail_closed(){
  for bad in [json!(-1),json!(0.5),json!("1740"),json!(MAX_SAFE+1)]{let mut v=fixture();v["providers"][0]["history"]=bad;assert!(collect(&v,100).is_err());}
  let mut v=fixture();v["updated"]=json!(101);assert!(collect(&v,100).is_err());v=fixture();v["sessions"][0]["id"]=json!("not-a-native-id");assert!(collect(&v,100).is_err());v=fixture();v["providers"].as_array_mut().unwrap().push(json!({"id":"codex","history":1}));assert!(collect(&v,100).is_err());
 }
 #[test]fn serialized_public_counts_exclude_helper_paths_models_auth_and_transcripts(){
  let c=collect(&fixture(),100).unwrap();let s=serde_json::to_string(&c.providers).unwrap();assert!(!s.contains("PRIVATE"));assert!(!s.contains(A));assert!(!s.contains("secret"));
 }
 #[test]fn cache_single_flight_cooldown_force_and_generation_are_independent_of_gui(){
  let now=Instant::now();let mut c=Cache::default();let a=c.begin(now,false).unwrap();assert!(c.begin(now+Duration::from_secs(20),true).is_none());c.finish(a,Ok(collect(&fixture(),100).unwrap()),now);assert!(c.begin(now+Duration::from_secs(1),true).is_none());let b=c.begin(now+Duration::from_secs(2),true).unwrap();c.finish(a,Err("collectionFailed"),now);assert!(c.snapshot(100).refreshing);c.finish(b,Err("collectionFailed"),now+Duration::from_secs(2));assert_eq!(c.snapshot(130).observed_at,Some(100));assert!(c.snapshot(130).stale);assert_eq!(c.snapshot(130).providers[0].known.value,Some(1740));assert!(c.begin(now+Duration::from_secs(16),false).is_none());assert!(c.begin(now+Duration::from_secs(17),false).is_some());
 }
 #[test]fn no_observation_and_failure_never_become_zero_or_freshness(){
  let now=Instant::now();let mut c=Cache::default();let s=c.snapshot(100);assert_eq!(s.status,"unavailable");assert!(s.providers.is_empty());let t=c.begin(now,false).unwrap();c.finish(t,Err("collectionFailed"),now);let s=c.snapshot(110);assert_eq!(s.observed_at,None);assert_eq!(s.reason,Some("collectionFailed"));
 }
}
