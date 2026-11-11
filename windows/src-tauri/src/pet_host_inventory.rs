//! Fixed, read-only native inventory. Source text is data, never instructions.
use serde::{Serialize,Deserialize};
use std::time::{Duration,Instant};
use crate::pet_config::system_access::{self,Captured};
use crate::pet_health::{Failure,task::Host};
const WINDOW:Duration=Duration::from_secs(2);
const RESULT_LIMIT:usize=8192;
#[derive(Clone,Copy)]enum Source{Distribution,KernelType,KernelRelease}
impl Source{
 fn limit(self)->usize{match self{Self::Distribution=>8192,_=>128}}
 fn name(self)->&'static str{match self{Self::Distribution=>"os-release.effective",Self::KernelType=>"procfs.kernel.ostype",Self::KernelRelease=>"procfs.kernel.osrelease"}}
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Distribution{id:String,version_id:Option<String>,pretty_name:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Observation<T>{source:String,path:Option<String>,sha256:Option<String>,available:bool,value:Option<T>,error:Option<Failure>}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Snapshot{
 schema_version:u32,kind:Kind,pet_id:String,grant_revision:String,host:Host,
 sample_started_at:u64,observed_at:u64,sample_duration_ms:u64,
 distribution:Observation<Distribution>,kernel_type:Observation<String>,kernel_release:Observation<String>,
 distribution_management:String,kernel_management:String,desired_state:String,trust:String,
 max_source_bytes:usize,max_result_bytes:usize,inference_used:bool,limitations:Vec<String>,
}
const LIMITATIONS:&[&str]=&["twoComponentsOnlyDistributionAndRunningKernel","sequentialReadNotAtomic","notPackageVersionOrVulnerabilityProof","noServicesProcessesLogsNetworkPersonalDataOrMutation","noSysworkspaceManagedOrDesiredStateClaim","noModelMemoryTaskOrDelegationPublication","duplicateSelectedReleaseKeysOrUnsupportedSymlinkRejected","blockedSyscallCannotBePreemptedPublicationStillBounded"];
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Kind{SystemHostInventory}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Binding{pub schema_version:u32,pub kind:Kind,pub pet_id:String,pub grant_revision:String,pub host:Host}
impl Binding{pub(crate)fn valid(&self)->bool{self.schema_version==1&&crate::pet_config::blueprint(&self.pet_id).is_some()&&crate::pet_inference::revision_key(&self.grant_revision)&&self.host.valid()}}
fn digest_valid(s:&str)->bool{s.len()==64&&s.bytes().all(|b|matches!(b,b'0'..=b'9'|b'a'..=b'f'))}
impl<T> Observation<T>{
 fn valid(&self,source:Source,paths:&[&str],value:impl FnOnce(&T)->bool)->bool{
  self.source==source.name()&&self.path.as_deref().is_none_or(|s|paths.contains(&s))&&self.sha256.as_deref().is_none_or(digest_valid)&&self.path.is_some()==self.sha256.is_some()&&match(&self.value,&self.error){(Some(v),None)=>self.available&&self.path.is_some()&&value(v),(None,Some(_))=>!self.available,_=>false}
 }
}
impl Snapshot{
 pub(crate)fn observed_at(&self)->u64{self.observed_at}
 pub(crate)fn started_at(&self)->u64{self.sample_started_at}
 pub(crate)fn matches(&self,b:&Binding)->bool{self.pet_id==b.pet_id&&self.grant_revision==b.grant_revision&&self.host==b.host}
 pub(crate)fn valid(&self)->bool{
  self.schema_version==1&&self.host.valid()&&crate::pet_config::blueprint(&self.pet_id).is_some()&&crate::pet_inference::revision_key(&self.grant_revision)&&self.sample_started_at>0&&self.sample_started_at<=self.observed_at&&self.observed_at<=8_640_000_000_000_000&&self.sample_duration_ms<2000&&self.max_source_bytes==8451&&self.max_result_bytes==RESULT_LIMIT&&!self.inference_used&&self.distribution_management=="externalObserved"&&self.kernel_management=="dynamicObserved"&&self.desired_state=="unknownNotCompared"&&self.trust=="untrustedObservedData"&&self.limitations.iter().map(String::as_str).eq(LIMITATIONS.iter().copied())
   &&self.distribution.valid(Source::Distribution,&["/etc/os-release","/usr/lib/os-release"],|d|printable(&d.id,256)&&d.id.bytes().all(|b|matches!(b,b'a'..=b'z'|b'0'..=b'9'|b'_'|b'-'|b'.'))&&d.version_id.as_ref().is_none_or(|s|printable(s,256)&&s.bytes().all(|b|matches!(b,b'a'..=b'z'|b'A'..=b'Z'|b'0'..=b'9'|b'_'|b'-'|b'.')))&&d.pretty_name.as_ref().is_none_or(|s|printable(s,256)))
   &&self.kernel_type.valid(Source::KernelType,&["/proc/sys/kernel/ostype"],|s|kernel(s.as_bytes()).is_ok_and(|v|v==*s))&&self.kernel_release.valid(Source::KernelRelease,&["/proc/sys/kernel/osrelease"],|s|kernel(s.as_bytes()).is_ok_and(|v|v==*s))&&serde_json::to_vec(self).is_ok_and(|v|v.len()<=RESULT_LIMIT)
 }
 pub(crate)fn complete(&self)->bool{self.valid()&&self.distribution.available&&self.kernel_type.available&&self.kernel_release.available}
}
pub(crate)fn validate(id:&str,version:u32,args:&serde_json::Value)->bool{id=="system.inventory.observe"&&version==1&&args.as_object().is_some_and(|v|v.is_empty())}
#[cfg(target_os="linux")]
pub(crate)fn sample(binding:&Binding,current:&dyn Fn()->bool)->Result<Snapshot,String>{
 let started=Instant::now();let check=||started.elapsed()<WINDOW&&current()&&Host::capture().is_ok_and(|h|h==binding.host);
 if !binding.valid()||!check(){return Err("O anfitrião ou a concessão de Inventário mudou.".into());}
 let result=collect(&binding.pet_id,&binding.grant_revision,&binding.host,&check,read_source)?;
 if !result.matches(binding)||!check(){return Err("O inventário perdeu a origem antes da publicação.".into());}Ok(result)
}
fn now()->u64{crate::assistant_memory::now()}
fn printable(s:&str,max:usize)->bool{!s.is_empty()&&s.len()<=max&&!s.chars().any(char::is_control)}
// os-release assignments only. No environment expansion, eval, or subprocess.
fn assignment(raw:&str)->Result<String,Failure>{
 if raw.len()>512{return Err(Failure::LimitExceeded);}
 let quote=raw.as_bytes().first().copied().filter(|b|matches!(b,b'\''|b'"'));
 let text=if let Some(q)=quote{
  if raw.len()<2||raw.as_bytes().last()!=Some(&q){return Err(Failure::InvalidData);}
  &raw[1..raw.len()-1]
 }else{raw};
 let mut result=String::new();let mut chars=text.chars();
 while let Some(c)=chars.next(){
  if c=='\\'&&quote!=Some(b'\''){
   let escaped=chars.next().ok_or(Failure::InvalidData)?;
   if !matches!(escaped,'\\'|'"'|'$'|'`'){return Err(Failure::InvalidData);}result.push(escaped);
  }else{
   if quote==Some(b'"')&&matches!(c,'"'|'$'|'`')||quote==Some(b'\'')&&c=='\''||quote.is_none()&&!matches!(c,'a'..='z'|'A'..='Z'|'0'..='9'|'_'|'-'|'.'){return Err(Failure::InvalidData);}
   result.push(c);
  }
 }
 if !printable(&result,256){return Err(Failure::InvalidData);}Ok(result)
}
fn distribution(bytes:&[u8])->Result<Distribution,Failure>{
 if bytes.len()>8192{return Err(Failure::LimitExceeded);}
 let text=std::str::from_utf8(bytes).map_err(|_|Failure::InvalidData)?;
 let(mut id,mut version,mut pretty)=(None,None,None);
 for line in text.lines(){
  if line.is_empty()||line.starts_with('#'){continue;}
  let Some((key,value))=line.split_once('=')else{continue};
  let target=match key{"ID"=>&mut id,"VERSION_ID"=>&mut version,"PRETTY_NAME"=>&mut pretty,_=>continue};
  // Duplicate selected keys are ambiguous evidence: fail closed, never merge.
  if target.is_some(){return Err(Failure::InvalidData);}*target=Some(assignment(value)?);
 }
 let id=id.ok_or(Failure::InvalidData)?;
 if !id.bytes().all(|b|matches!(b,b'a'..=b'z'|b'0'..=b'9'|b'_'|b'-'|b'.')){return Err(Failure::InvalidData);}
 if version.as_ref().is_some_and(|s|!s.bytes().all(|b|matches!(b,b'a'..=b'z'|b'A'..=b'Z'|b'0'..=b'9'|b'_'|b'-'|b'.'))){return Err(Failure::InvalidData);}
 Ok(Distribution{id,version_id:version,pretty_name:pretty})
}
fn kernel(bytes:&[u8])->Result<String,Failure>{
 if bytes.len()>128{return Err(Failure::LimitExceeded);}
 let raw=std::str::from_utf8(bytes).map_err(|_|Failure::InvalidData)?;let text=raw.strip_suffix('\n').unwrap_or(raw);
 if !printable(text,127)||!text.bytes().all(|b|matches!(b,b'a'..=b'z'|b'A'..=b'Z'|b'0'..=b'9'|b'_'|b'-'|b'.'|b'+')){return Err(Failure::InvalidData);}Ok(text.into())
}
struct Read{path:&'static str,bytes:Vec<u8>}
fn observation<T>(source:Source,read:Result<Read,Failure>,parse:impl FnOnce(&[u8])->Result<T,Failure>)->Observation<T>{
 let(path,hash,value)=match read{
  Ok(read)=>{let hash=ring::digest::digest(&ring::digest::SHA256,&read.bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>();(Some(read.path.into()),Some(hash),parse(&read.bytes))},
  Err(error)=>(None,None,Err(error)),
 };
 match value{Ok(value)=>Observation{source:source.name().into(),path,sha256:hash,available:true,value:Some(value),error:None},Err(error)=>Observation{source:source.name().into(),path,sha256:hash,available:false,value:None,error:Some(error)}}
}
fn collect(pet:&str,revision:&str,host:&Host,current:&dyn Fn()->bool,mut read:impl FnMut(Source)->Result<Read,Failure>)->Result<Snapshot,String>{
 let started=Instant::now();let sampled=now();let valid=||started.elapsed()<WINDOW&&current();
 let mut get=|source|{if !valid(){return Err("A origem, a concessão ou o prazo terminou.".to_string());}let data=read(source);if !valid(){return Err("O inventário foi retirado durante a leitura e não foi publicado.".into());}Ok(data)};
 let distribution=observation(Source::Distribution,get(Source::Distribution)?,distribution);
 let kernel_type=observation(Source::KernelType,get(Source::KernelType)?,kernel);
 let kernel_release=observation(Source::KernelRelease,get(Source::KernelRelease)?,kernel);
 let snapshot=Snapshot{schema_version:1,kind:Kind::SystemHostInventory,pet_id:pet.into(),grant_revision:revision.into(),host:host.clone(),sample_started_at:sampled,observed_at:now(),sample_duration_ms:started.elapsed().as_millis()as u64,distribution,kernel_type,kernel_release,distribution_management:"externalObserved".into(),kernel_management:"dynamicObserved".into(),desired_state:"unknownNotCompared".into(),trust:"untrustedObservedData".into(),max_source_bytes:8451,max_result_bytes:RESULT_LIMIT,inference_used:false,limitations:LIMITATIONS.iter().map(|v|(*v).into()).collect()};
 if !valid()||!snapshot.valid()||serde_json::to_vec(&snapshot).map_err(|_|"O inventário não pôde ser representado.")?.len()>RESULT_LIMIT{return Err("O inventário perdeu autorização ou excedeu o limite.".into());}Ok(snapshot)
}
#[cfg(target_os="linux")]
fn read_source(source:Source)->Result<Read,Failure>{
 use std::path::Path;
 let path=match source{
  Source::Distribution=>match std::fs::symlink_metadata("/etc/os-release"){
   Ok(meta)if meta.is_symlink()=>{let target=std::fs::read_link("/etc/os-release").map_err(|_|Failure::Unavailable)?;if !matches!(target.to_str(),Some("../usr/lib/os-release"|"/usr/lib/os-release")){return Err(Failure::InvalidData);} "/usr/lib/os-release"},
   Ok(meta)if meta.is_file()=>"/etc/os-release",
   Err(error)if error.kind()==std::io::ErrorKind::NotFound=>"/usr/lib/os-release",
   _=>return Err(Failure::Unavailable),
  },
  Source::KernelType=>"/proc/sys/kernel/ostype",Source::KernelRelease=>"/proc/sys/kernel/osrelease",
 };
 Ok(Read{path,bytes:crate::pet_health::bounded_file(Path::new(path),source.limit(),false)?})
}
#[tauri::command]
pub async fn pet_host_inventory(window:tauri::WebviewWindow,app:tauri::AppHandle,pet_id:String)->Result<Snapshot,String>{
 system_access::addressed(&window,&app,&pet_id)?;
 #[cfg(not(target_os="linux"))]{Err("O inventário limitado está disponível em Linux.".into())}
 #[cfg(target_os="linux")]{
  let frame=std::sync::Arc::new(crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa terminou.")?);
  let scope=std::sync::Arc::new(Captured::capture_inventory(&pet_id)?);let host=Host::capture()?;let revision=scope.revision().to_owned();
  let started=Instant::now();let(f,s,w,a,pet,original)=(frame.clone(),scope.clone(),window.clone(),app.clone(),pet_id.clone(),host.clone());
  let binding=Binding{schema_version:1,kind:Kind::SystemHostInventory,pet_id:pet.clone(),grant_revision:revision,host:original};
  let snapshot=tauri::async_runtime::spawn_blocking(move||sample(&binding,&||started.elapsed()<WINDOW&&f.current()&&system_access::addressed(&w,&a,&pet).is_ok()&&s.check().is_ok())).await.map_err(|_|"O inventário terminou sem confirmação.")??;
  if started.elapsed()>=WINDOW||!frame.current()||system_access::addressed(&window,&app,&pet_id).is_err()||scope.check().is_err()||!Host::capture().is_ok_and(|h|h==host){return Err("O inventário perdeu a origem ou autorização antes da publicação.".into());}Ok(snapshot)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn host()->Host{Host{boot_id:"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),pid_namespace:1,mount_namespace:2}}
 #[test]fn host_inventory_parser_is_bounded_selective_and_never_executes_release_text(){
  let d=distribution(b"ID=fedora\nVERSION_ID=43\nPRETTY_NAME=\"Fedora Linux 43\"\nPRIVATE_TOKEN=ignored\nCOMMAND=$(touch ignored)\n").unwrap();assert_eq!(d.id,"fedora");assert_eq!(d.version_id.as_deref(),Some("43"));assert!(!serde_json::to_string(&d).unwrap().contains("ignored"));
  assert_eq!(assignment(r#""literal \$(command) \`text\`""#).unwrap(),"literal $(command) `text`");
  for bad in [b"ID=fedora\nID=other".as_slice(),b"ID=$(command)",b"ID=Fedora",b"ID=fedora\nPRETTY_NAME=\"$(command)\"",b"ID=fedora\nPRETTY_NAME=\"bad\nname\""]{assert!(distribution(bad).is_err());}
  assert!(distribution(&vec![b'x';8193]).is_err());assert!(distribution(b"VERSION_ID=43").is_err());assert!(kernel(b"6.17.2-300.fc43.x86_64\n").is_ok());assert!(kernel(b"Linux\nInjected").is_err());
 }
 #[test]fn host_inventory_partial_coverage_preserves_origin_and_never_claims_managed_or_healthy(){
  let snapshot=collect("agent_system_guardian",&"a".repeat(32),&host(),&||true,|s|match s{Source::Distribution=>Ok(Read{path:"/etc/os-release",bytes:b"ID=fedora\nVERSION_ID=43".to_vec()}),_=>Err(Failure::Unavailable)}).unwrap();
  assert!(snapshot.distribution.available&&!snapshot.kernel_type.available&&!snapshot.kernel_release.available);assert_eq!(snapshot.distribution.path.as_deref(),Some("/etc/os-release"));assert_eq!(snapshot.distribution.sha256.as_ref().unwrap().len(),64);assert_eq!(snapshot.desired_state,"unknownNotCompared");assert!(!snapshot.inference_used);assert!(serde_json::to_vec(&snapshot).unwrap().len()<8192);
 }
 #[test]fn host_inventory_runtime_evidence_rejects_false_origin_management_and_incomplete_success(){
  let snapshot=collect("agent_system_guardian",&"a".repeat(32),&host(),&||true,|s|Ok(match s{Source::Distribution=>Read{path:"/etc/os-release",bytes:b"ID=fedora\nVERSION_ID=43".to_vec()},Source::KernelType=>Read{path:"/proc/sys/kernel/ostype",bytes:b"Linux\n".to_vec()},Source::KernelRelease=>Read{path:"/proc/sys/kernel/osrelease",bytes:b"6.17.2\n".to_vec()}})).unwrap();assert!(snapshot.complete());let original=serde_json::to_value(&snapshot).unwrap();
  for (field,value)in [("desiredState",serde_json::json!("managed")),("trust",serde_json::json!("authority")),("inferenceUsed",serde_json::json!(true)),("maxSourceBytes",serde_json::json!(999999))]{let mut changed=original.clone();changed[field]=value;let changed:Snapshot=serde_json::from_value(changed).unwrap();assert!(!changed.valid());}
  let mut changed=original.clone();changed["distribution"]["path"]=serde_json::json!("/etc/shadow");assert!(!serde_json::from_value::<Snapshot>(changed).unwrap().valid());
  let mut changed=original.clone();changed["kernelRelease"]["available"]=serde_json::json!(false);changed["kernelRelease"]["value"]=serde_json::Value::Null;changed["kernelRelease"]["error"]=serde_json::json!("unavailable");let changed:Snapshot=serde_json::from_value(changed).unwrap();assert!(changed.valid()&&!changed.complete());
  let binding=Binding{schema_version:1,kind:Kind::SystemHostInventory,pet_id:"ai_ollama".into(),grant_revision:"a".repeat(32),host:host()};assert!(!snapshot.matches(&binding));let mut changed=original.clone();changed["unreviewed"]=serde_json::json!(true);assert!(serde_json::from_value::<Snapshot>(changed).is_err());
 }
 #[test]fn host_inventory_revocation_during_io_discards_all_publication(){
  let live=std::cell::Cell::new(true);let reads=std::cell::Cell::new(0);
  assert!(collect("ai_ollama",&"a".repeat(32),&host(),&||live.get(),|_|{reads.set(reads.get()+1);live.set(false);Err(Failure::Unavailable)}).is_err());assert_eq!(reads.get(),1);
  assert!(collect("ai_ollama",&"a".repeat(32),&host(),&||false,|_|panic!("denied before IO")).is_err());
 }
}
