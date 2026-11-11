//! Fixed host identity and bounded sampler for the common native task runtime.
//! These objects are evidence/data; they never confer runtime authority.
use serde::{Serialize,Deserialize};
use super::{Memory,Cpu,Uptime};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Host{pub boot_id:String,pub pid_namespace:u64,pub mount_namespace:u64}
impl Host{
 pub(crate)fn valid(&self)->bool{self.boot_id.len()==36&&self.boot_id.bytes().enumerate().all(|(i,b)|if matches!(i,8|13|18|23){b==b'-'}else{b.is_ascii_hexdigit()&& !b.is_ascii_uppercase()})&&self.pid_namespace>0&&self.mount_namespace>0}
 #[cfg(target_os="linux")]pub(crate)fn capture()->Result<Self,String>{
  use std::os::unix::fs::MetadataExt;
  let bytes=super::bounded_file(std::path::Path::new("/proc/sys/kernel/random/boot_id"),64,false).map_err(|_|"A identidade do anfitrião não está disponível.")?;
  let host=Self{boot_id:std::str::from_utf8(&bytes).map_err(|_|"Identidade inválida.")?.trim().into(),pid_namespace:std::fs::metadata("/proc/self/ns/pid").map_err(|_|"Namespace indisponível.")?.ino(),mount_namespace:std::fs::metadata("/proc/self/ns/mnt").map_err(|_|"Namespace indisponível.")?.ino()};
  if !host.valid(){return Err("Identidade inválida.".into());}Ok(host)
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Kind{SystemHostHealth}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Binding{pub schema_version:u32,pub kind:Kind,pub pet_id:String,pub grant_revision:String,pub host:Host}
impl Binding{pub(crate)fn valid(&self)->bool{self.schema_version==1&&crate::pet_config::blueprint(&self.pet_id).is_some()&&crate::pet_inference::revision_key(&self.grant_revision)&&self.host.valid()}}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Evidence{
 pub kind:Kind,pub pet_id:String,pub grant_revision:String,pub host:Host,
 pub sample_started_at:u64,pub observed_at:u64,pub sample_duration_ms:u64,
 pub(super)memory:Option<Memory>,pub(super)cpu:Option<Cpu>,pub(super)uptime:Option<Uptime>,
}
impl Evidence{
 pub(crate)fn valid(&self)->bool{self.host.valid()&&crate::pet_config::blueprint(&self.pet_id).is_some()&&crate::pet_inference::revision_key(&self.grant_revision)&&self.sample_started_at>0&&self.sample_started_at<=self.observed_at&&self.observed_at<=8_640_000_000_000_000&&self.sample_duration_ms<2000&&self.memory.as_ref().is_none_or(|v|v.total_bytes>0&&v.available_bytes<=v.total_bytes)&&self.cpu.as_ref().is_none_or(|v|v.idle_ticks<=v.total_ticks)&&self.uptime.as_ref().is_none_or(|v|v.seconds.is_finite()&&v.seconds>=0.0)}
 pub(crate)fn complete(&self)->bool{self.valid()&&self.memory.is_some()&&self.cpu.is_some()&&self.uptime.is_some()}
 pub(crate)fn matches(&self,b:&Binding)->bool{self.pet_id==b.pet_id&&self.grant_revision==b.grant_revision&&self.host==b.host}
}
pub(crate)fn validate(id:&str,version:u32,args:&serde_json::Value)->bool{matches!(id,"system.health.observe"|"system.health.baseline")&&version==1&&args.as_object().is_some_and(|v|v.is_empty())}
#[cfg(target_os="linux")]
pub(crate)fn sample(binding:&Binding,current:&dyn Fn()->bool)->Result<Evidence,String>{
 let started=std::time::Instant::now();let check=||started.elapsed()<std::time::Duration::from_millis(2000)&&current()&&Host::capture().is_ok_and(|h|h==binding.host)&&started.elapsed()<std::time::Duration::from_millis(2000);
 if !binding.valid()||!check(){return Err("O anfitrião ou a concessão mudou.".into());}
 let s=super::observe_for(&binding.pet_id,&binding.grant_revision,&check,|source|super::bounded_file(std::path::Path::new(source.path()),source.limit(),matches!(source,super::Source::Cpu)),std::time::Duration::from_millis(2000))?;
 let s=super::publish(s,&check)?;
 let e=Evidence{kind:Kind::SystemHostHealth,pet_id:s.pet_id,grant_revision:s.policy_revision,host:binding.host.clone(),sample_started_at:s.sample_started_at,observed_at:s.observed_at,sample_duration_ms:s.sample_duration_ms,memory:s.memory.value,cpu:s.cpu.value,uptime:s.uptime.value};
 if !e.valid(){return Err("A amostra não tem evidência válida.".into());}Ok(e)
}
