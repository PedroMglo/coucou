//! A bounded two-sample artifact. No model, scheduler, tuning or new authority.
use serde::{Deserialize,Serialize};
use super::task::{Binding,Evidence};
use std::time::{Duration,Instant};
const INTERVAL_MS:u64=1000;
const WINDOW_MS:u64=6000;
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Kind{SystemHostHealthBaseline}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Baseline{
 pub schema_version:u32,pub kind:Kind,pub before:Evidence,pub after:Evidence,
 pub interval_ms:u64,pub elapsed_ms:u64,pub cpu_busy_basis_points:Option<u32>,
 pub memory_available_delta_bytes:Option<i64>,pub max_metric_read_bytes:u32,pub inference_used:bool,
}
fn cpu(before:&Evidence,after:&Evidence,elapsed_ms:u64)->Option<u32>{
 let(a,b)=(before.cpu.as_ref()?,after.cpu.as_ref()?);
 let uptime=after.uptime.as_ref()?.seconds-before.uptime.as_ref()?.seconds;
 // Suspend/reset/clock discontinuity is not a meaningful short CPU baseline.
 if !(1.0..=(elapsed_ms as f64/1000.0+0.25)).contains(&uptime){return None;}
 let total=b.total_ticks.checked_sub(a.total_ticks)?;
 let idle=b.idle_ticks.checked_sub(a.idle_ticks)?;
 if total==0||idle>total{return None;}
 Some(((u128::from(total-idle)*10_000)/u128::from(total))as u32)
}
fn memory(before:&Evidence,after:&Evidence)->Option<i64>{
 let(a,b)=(before.memory.as_ref()?,after.memory.as_ref()?);
 if a.total_bytes!=b.total_bytes{return None;}
 i64::try_from(i128::from(b.available_bytes)-i128::from(a.available_bytes)).ok()
}
impl Baseline{
 fn from_samples(before:Evidence,after:Evidence,interval_ms:u64,elapsed_ms:u64)->Result<Self,String>{
  let value=Self{schema_version:1,kind:Kind::SystemHostHealthBaseline,cpu_busy_basis_points:cpu(&before,&after,elapsed_ms),memory_available_delta_bytes:memory(&before,&after),before,after,interval_ms,elapsed_ms,max_metric_read_bytes:17_670,inference_used:false};
  if !value.valid(){return Err("A medição não conserva origem, revisões ou intervalo válidos.".into());}Ok(value)
 }
 pub(crate)fn valid(&self)->bool{
  self.schema_version==1&&!self.inference_used&&self.max_metric_read_bytes==17_670
   &&self.before.valid()&&self.after.valid()&&self.before.pet_id==self.after.pet_id&&self.before.grant_revision==self.after.grant_revision&&self.before.host==self.after.host
   &&(INTERVAL_MS..WINDOW_MS).contains(&self.interval_ms)&&self.elapsed_ms>=self.interval_ms&&self.elapsed_ms<WINDOW_MS
   &&self.after.sample_started_at>=self.before.observed_at.saturating_add(INTERVAL_MS)&&self.after.observed_at.saturating_sub(self.before.sample_started_at)<WINDOW_MS
   &&self.cpu_busy_basis_points==cpu(&self.before,&self.after,self.elapsed_ms)&&self.memory_available_delta_bytes==memory(&self.before,&self.after)
 }
 pub(crate)fn samples_complete(&self)->bool{self.valid()&&self.before.complete()&&self.after.complete()}
 pub(crate)fn measured(&self)->bool{self.samples_complete()&&self.cpu_busy_basis_points.is_some()&&self.memory_available_delta_bytes.is_some()}
 pub(crate)fn matches(&self,b:&Binding)->bool{self.before.matches(b)&&self.after.matches(b)}
}
fn wait_interval(started:Instant,deadline:Instant,current:&dyn Fn()->bool)->Result<(),String>{
 loop{
  if Instant::now()>=deadline||!current(){return Err("A medição foi interrompida ou perdeu a autorização durante a espera.".into());}
  let elapsed=started.elapsed();if elapsed>=Duration::from_millis(INTERVAL_MS){return Ok(());}
  std::thread::sleep((Duration::from_millis(INTERVAL_MS)-elapsed).min(Duration::from_millis(25)));
 }
}
#[cfg(target_os="linux")]
pub(crate)fn measure(binding:&Binding,current:&dyn Fn()->bool)->Result<Baseline,String>{
 let started=Instant::now();let deadline=started+Duration::from_millis(WINDOW_MS);
 let check=||Instant::now()<deadline&&current();
 let before=super::task::sample(binding,&check)?;let interval=Instant::now();
 wait_interval(interval,deadline,&check)?;
 let interval_ms=interval.elapsed().as_millis()as u64;
 let after=super::task::sample(binding,&check)?;
 let baseline=Baseline::from_samples(before,after,interval_ms,started.elapsed().as_millis()as u64)?;
 if !check()||serde_json::to_vec(&baseline).map_err(|_|"A medição não pôde ser representada.")?.len()>8192{return Err("A medição terminou sem autorização ou excedeu o limite.".into());}Ok(baseline)
}

#[cfg(test)]mod tests{
 use super::*;
 use serde_json::json;
 fn evidence(at:u64,total:u64,idle:u64,available:u64,uptime:f64)->Evidence{
  serde_json::from_value(json!({"kind":"systemHostHealth","petId":"agent_system_guardian","grantRevision":"a".repeat(32),"host":{"bootId":"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee","pidNamespace":1,"mountNamespace":2},"sampleStartedAt":at,"observedAt":at+1,"sampleDurationMs":1,"memory":{"totalBytes":10000,"availableBytes":available},"cpu":{"totalTicks":total,"idleTicks":idle},"uptime":{"seconds":uptime}})).unwrap()
 }
 fn pair()->Baseline{Baseline::from_samples(evidence(1000,100,40,5000,10.0),evidence(2100,200,65,4000,11.1),1050,1200).unwrap()}
 #[test]fn health_baseline_derives_bounded_original_counters_and_rejects_forged_results(){
  let mut b=pair();assert!(b.measured());assert_eq!(b.cpu_busy_basis_points,Some(7500));assert_eq!(b.memory_available_delta_bytes,Some(-1000));b.cpu_busy_basis_points=Some(9999);assert!(!b.valid());
  let mut b=pair();b.after.host.pid_namespace=99;assert!(!b.valid());let mut b=pair();b.after.grant_revision="b".repeat(32);assert!(!b.valid());let mut b=pair();b.after.pet_id="ai_ollama".into();assert!(!b.valid());
 }
 #[test]fn health_baseline_reset_zero_iowait_inconsistency_and_suspend_are_indeterminate(){
  for(total,idle,uptime)in[(100,40,11.1),(90,45,11.1),(110,80,11.1),(200,65,500.0),(200,65,9.0)]{
   let b=Baseline::from_samples(evidence(1000,100,40,5000,10.0),evidence(2100,total,idle,4000,uptime),1050,1200).unwrap();assert!(b.valid());assert!(b.cpu_busy_basis_points.is_none());assert!(!b.measured());
  }
  let mut after=evidence(2100,200,65,4000,11.1);after.memory.as_mut().unwrap().total_bytes=20000;let b=Baseline::from_samples(evidence(1000,100,40,5000,10.0),after,1050,1200).unwrap();assert!(b.memory_available_delta_bytes.is_none()&&!b.measured());
 }
 #[test]fn health_baseline_wait_revocation_and_deadline_do_not_complete_interval(){
  let calls=std::cell::Cell::new(0);assert!(wait_interval(Instant::now(),Instant::now()+Duration::from_secs(2),&||{calls.set(calls.get()+1);calls.get()<2}).is_err());assert_eq!(calls.get(),2);
  assert!(wait_interval(Instant::now(),Instant::now(),&||true).is_err());
 }
}
