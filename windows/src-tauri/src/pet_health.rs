//! Bounded read-only health broker. No model, shell, process/log scan or mutation.
use serde::{Serialize,Deserialize};
#[path="pet_health_task.rs"]pub(crate)mod task;
#[path="pet_health_baseline.rs"]pub(crate)mod baseline;
use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
use crate::pet_config::system_access::{self,Captured};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Failure{Unavailable,InvalidData,LimitExceeded}
#[derive(Clone,Copy)]
enum Source{Memory,Cpu,Uptime}
impl Source{
 fn name(self)->&'static str{match self{Self::Memory=>"procfs.meminfo",Self::Cpu=>"procfs.stat.cpu",Self::Uptime=>"procfs.uptime"}}
 fn limit(self)->usize{match self{Self::Memory=>8192,Self::Cpu=>512,Self::Uptime=>128}}
 #[cfg(target_os="linux")]
 fn path(self)->&'static str{match self{Self::Memory=>"/proc/meminfo",Self::Cpu=>"/proc/stat",Self::Uptime=>"/proc/uptime"}}
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Observation<T>{source:&'static str,available:bool,value:Option<T>,error:Option<Failure>}
impl<T> Observation<T>{
 fn from(source:Source,value:Result<T,Failure>)->Self{match value{
  Ok(value)=>Self{source:source.name(),available:true,value:Some(value),error:None},
  Err(error)=>Self{source:source.name(),available:false,value:None,error:Some(error)},
 }}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Memory{total_bytes:u64,available_bytes:u64}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Cpu{total_ticks:u64,idle_ticks:u64}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Uptime{seconds:f64}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Finding{id:&'static str,severity:&'static str,confidence:&'static str,status:&'static str,source:&'static str,threshold_percent:u8,available_bytes:u64,total_bytes:u64,recommendation:&'static str}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot{
 schema_version:u32,pet_id:String,policy_revision:String,management:&'static str,
 sample_started_at:u64,observed_at:u64,sample_duration_ms:u64,
 memory:Observation<Memory>,cpu:Observation<Cpu>,uptime:Observation<Uptime>,findings:Vec<Finding>,
 max_read_bytes:usize,max_result_bytes:usize,inference_used:bool,limitations:Vec<&'static str>,
}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128)as u64}
fn text(bytes:&[u8],limit:usize)->Result<&str,Failure>{
 if bytes.len()>limit{return Err(Failure::LimitExceeded);}
 std::str::from_utf8(bytes).map_err(|_|Failure::InvalidData)
}
fn memory(bytes:&[u8])->Result<Memory,Failure>{
 let mut total=None;let mut available=None;
 for line in text(bytes,Source::Memory.limit())?.lines(){
  let mut words=line.split_whitespace();
  let target=match words.next(){Some("MemTotal:")=>&mut total,Some("MemAvailable:")=>&mut available,_=>continue};
  if target.is_some(){return Err(Failure::InvalidData);}
  let value=words.next().and_then(|s|s.parse::<u64>().ok()).and_then(|n|n.checked_mul(1024)).ok_or(Failure::InvalidData)?;
  if words.next()!=Some("kB")||words.next().is_some(){return Err(Failure::InvalidData);}*target=Some(value);
 }
 let(total,available)=(total.ok_or(Failure::InvalidData)?,available.ok_or(Failure::InvalidData)?);
 if total==0||available>total{return Err(Failure::InvalidData);}Ok(Memory{total_bytes:total,available_bytes:available})
}
fn cpu(bytes:&[u8])->Result<Cpu,Failure>{
 let mut words=text(bytes,Source::Cpu.limit())?.lines().next().ok_or(Failure::InvalidData)?.split_whitespace();
 if words.next()!=Some("cpu"){return Err(Failure::InvalidData);}
 let values=words.map(|s|s.parse::<u64>().map_err(|_|Failure::InvalidData)).collect::<Result<Vec<_>,_>>()?;
 if !(4..=10).contains(&values.len()){return Err(Failure::InvalidData);}
 // Linux guest counters are already included in user/nice. Never double count.
 let total=values.iter().take(8).try_fold(0u64,|sum,n|sum.checked_add(*n).ok_or(Failure::InvalidData))?;
 let idle=values[3].checked_add(values.get(4).copied().unwrap_or(0)).ok_or(Failure::InvalidData)?;
 if total<idle{return Err(Failure::InvalidData);}Ok(Cpu{total_ticks:total,idle_ticks:idle})
}
fn uptime(bytes:&[u8])->Result<Uptime,Failure>{
 let values=text(bytes,Source::Uptime.limit())?.split_whitespace().map(|s|s.parse::<f64>().map_err(|_|Failure::InvalidData)).collect::<Result<Vec<_>,_>>()?;
 if values.len()!=2||values.iter().any(|n|!n.is_finite()||*n<0.0){return Err(Failure::InvalidData);}Ok(Uptime{seconds:values[0]})
}
fn observe(pet:&str,revision:&str,current:impl Fn()->bool,read:impl FnMut(Source)->Result<Vec<u8>,Failure>)->Result<Snapshot,String>{observe_for(pet,revision,current,read,Duration::from_millis(250))}
fn observe_for(pet:&str,revision:&str,current:impl Fn()->bool,mut read:impl FnMut(Source)->Result<Vec<u8>,Failure>,window:Duration)->Result<Snapshot,String>{
 let started=Instant::now();let sampled=now();
 let valid=||current()&&started.elapsed()<window;
 let mut get=|source|{if !valid(){return Err("A concessão ou a janela de observação terminou.".to_string());}let bytes=read(source);if !valid(){return Err("A observação foi retirada ou ultrapassou o prazo; não foi publicada.".into());}Ok(bytes)};
 let memory=Observation::from(Source::Memory,get(Source::Memory)?.and_then(|b|memory(&b)));
 let cpu=Observation::from(Source::Cpu,get(Source::Cpu)?.and_then(|b|cpu(&b)));
 let uptime=Observation::from(Source::Uptime,get(Source::Uptime)?.and_then(|b|uptime(&b)));
 let mut findings=Vec::new();
 if let Some(m)=&memory.value{
  if (m.available_bytes as u128)*100<(m.total_bytes as u128)*5{
   findings.push(Finding{id:"health.memory.available.low",severity:"warning",confidence:"observed",status:"sampleOnly",source:Source::Memory.name(),threshold_percent:5,available_bytes:m.available_bytes,total_bytes:m.total_bytes,recommendation:"Comparar outra amostra com o trabalho em curso antes de propor qualquer alteração."});
  }
 }
 let snapshot=Snapshot{schema_version:1,pet_id:pet.into(),policy_revision:revision.into(),management:"dynamicObserved",sample_started_at:sampled,observed_at:now(),sample_duration_ms:started.elapsed().as_millis()as u64,memory,cpu,uptime,findings,max_read_bytes:8192+512+128+3,max_result_bytes:8192,inference_used:false,limitations:vec!["threeFixedProcfsCountersOnly","sequentialSampleNotAtomic","cpuTicksNeedSeparateBaselineNoUsagePercent","noProcessLogPackageNetworkOrPersonalDataInventory","noVulnerabilityOrDesiredStateConclusion","noAutomaticModelMemoryOrTaskPublication"]};
 if !valid(){return Err("A observação deixou de estar autorizada.".into());}
 if serde_json::to_vec(&snapshot).map_err(|_|"A observação não pôde ser representada.")?.len()>8192{return Err("A observação excedeu o limite de saída.".into());}Ok(snapshot)
}
fn publish(snapshot:Snapshot,current:impl Fn()->bool)->Result<Snapshot,String>{
 if !current(){return Err("A origem, a concessão ou o prazo terminou antes da publicação.".into());}Ok(snapshot)
}
#[cfg(target_os="linux")]
pub(crate)fn bounded_file(path:&std::path::Path,limit:usize,first_line:bool)->Result<Vec<u8>,Failure>{
 use std::io::{Read,BufRead};use std::os::unix::fs::OpenOptionsExt;
 let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK).open(path).map_err(|_|Failure::Unavailable)?;
 if !file.metadata().map_err(|_|Failure::Unavailable)?.is_file(){return Err(Failure::InvalidData);}
 let mut bytes=Vec::new();let limited=file.take(limit as u64+1);
 if first_line{std::io::BufReader::new(limited).read_until(b'\n',&mut bytes).map_err(|_|Failure::Unavailable)?;}else{limited.take(limit as u64+1).read_to_end(&mut bytes).map_err(|_|Failure::Unavailable)?;}
 if bytes.len()>limit{return Err(Failure::LimitExceeded);}Ok(bytes)
}
#[tauri::command]
pub async fn pet_health_snapshot(window:tauri::WebviewWindow,app:tauri::AppHandle,pet_id:String)->Result<Snapshot,String>{
 system_access::addressed(&window,&app,&pet_id)?;
 #[cfg(not(target_os="linux"))]{Err("A observação de saúde está disponível em Linux.".into())}
 #[cfg(target_os="linux")]{
  let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa terminou.")?;
  let frame=std::sync::Arc::new(frame);let scope=std::sync::Arc::new(Captured::capture_health(&pet_id)?);let revision=scope.revision().to_owned();
  let started=Instant::now();let worker_frame=frame.clone();let worker_scope=scope.clone();let worker_window=window.clone();let worker_app=app.clone();let worker_pet=pet_id.clone();
  let snapshot=tauri::async_runtime::spawn_blocking(move||{
   observe(&worker_pet,&revision,||started.elapsed()<Duration::from_millis(250)&&worker_frame.current()&&system_access::addressed(&worker_window,&worker_app,&worker_pet).is_ok()&&worker_scope.check().is_ok(),|source|bounded_file(std::path::Path::new(source.path()),source.limit(),matches!(source,Source::Cpu)))
  }).await.map_err(|_|"A observação terminou sem confirmação.".to_string())??;
  // Worker completion is not permission to publish after an await. Retain the
  // original scope/frame and include resource wait in the publication deadline.
  publish(snapshot,||started.elapsed()<Duration::from_millis(250)&&frame.current()&&system_access::addressed(&window,&app,&pet_id).is_ok()&&scope.check().is_ok())
 }
}
/// Native user observation; the goal path supplies its own original authority.
#[tauri::command]
pub async fn pet_health_baseline(window:tauri::WebviewWindow,app:tauri::AppHandle,pet_id:String)->Result<baseline::Baseline,String>{
 system_access::addressed(&window,&app,&pet_id)?;
 #[cfg(not(target_os="linux"))]{Err("A medição de saúde está disponível em Linux.".into())}
 #[cfg(target_os="linux")]{
  let frame=std::sync::Arc::new(crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa terminou.")?);let scope=std::sync::Arc::new(Captured::capture_health(&pet_id)?);
  let binding=task::Binding{schema_version:1,kind:task::Kind::SystemHostHealth,pet_id:pet_id.clone(),grant_revision:scope.revision().into(),host:task::Host::capture()?};
  let started=Instant::now();let f=frame.clone();let s=scope.clone();let w=window.clone();let a=app.clone();let pet=pet_id.clone();
  let measured=tauri::async_runtime::spawn_blocking(move||baseline::measure(&binding,&||started.elapsed()<Duration::from_secs(6)&&f.current()&&system_access::addressed(&w,&a,&pet).is_ok()&&s.check().is_ok())).await.map_err(|_|"A medição terminou sem confirmação.")??;
  if started.elapsed()>=Duration::from_secs(6)||!frame.current()||system_access::addressed(&window,&app,&pet_id).is_err()||scope.check().is_err(){return Err("A medição perdeu a autorização antes da publicação.".into());}Ok(measured)
 }
}
#[cfg(test)]#[path="pet_health_tests.rs"]mod tests;
