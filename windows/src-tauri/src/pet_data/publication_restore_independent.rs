//! Exact native intent before independent key generation. The prepared seed
//! remains outside operational payload; Admission is a later explicit phase.
use super::*;
use crate::pet_data::{restored_history::History,restored_transaction::read_file,staging::Source};
use crate::pet_runtime::restoration::{IndependentScope,IndependentStep,IndependentKeyState};
use serde_json::{Value,json};
pub(in crate::pet_data)const JOB:&str=".restore-authority";
const GENERATION:&str="generation.json";const MANIFEST:&str="manifest.json";const HISTORY:&str="history.json";const READY:&str="ready.json";const SEED:&str="seed";const MAX:usize=8*1024*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Intent{version:u32,purpose:String,pair:String,layout:String,source:String,goals:Value,references:Value,configuration:Value,#[serde(default,skip_serializing_if="Option::is_none")]retention:Option<super::retention::Binding>}
impl Intent{
 fn validate(&self,config:&Dir)->Result<()>{
  match(self.version,self.retention.as_ref()){
   (1,None)if self.purpose=="independentFutureAuthorityPreparation"=>{},
   (2,Some(b))if self.purpose=="independentFutureAuthorityAfterRetention"=>b.validate()?,
   _=>return Err(Error::UnsupportedProfile),
  }
  if super::retention::binding(config)?!=self.retention{return Err(Error::SourceChanged);}Ok(())
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Ready{version:u32,manifest:String,seed:String,authority:Value}
pub(in crate::pet_data)struct IndependentReview{data:Review,scope:IndependentScope,intent:Intent,bytes:Vec<u8>,history:Vec<u8>}
fn core(e:crate::pet_runtime::restoration::HistoryFailure)->Error{super::authority::error(e)}
fn get(root:&Dir,n:&str,limit:usize)->Result<Vec<u8>>{read_file(root,n,limit)?.ok_or(Error::SourceChanged)}
fn generation(manifest:&[u8])->Result<Vec<u8>>{let m:Intent=parse(manifest)?;serde_json::to_vec(&json!({"version":m.version,"manifest":digest(manifest),"purpose":if m.version==2{"generateDistinctFutureKeyAfterRetentionOnce"}else{"generateIndependentKeyOnce"}})).map_err(|_|Error::Unavailable)}
fn seed_hash(seed:&Dir)->Result<String>{
 let mut values=std::collections::BTreeMap::new();let mut used=0usize;let mut staged=0;
 for n in seed.node()?.names(20)?{let n=n.to_str().ok_or(Error::UnsafePath)?;let limit=match n{".key.json"=>512,".catalog.lock"=>0,".protocol.json"=>1024,".restored-history.json"=>MAX,_=>{if !n.strip_prefix(".restore-index-").is_some_and(|v|valid_hex(v,32))&&!n.strip_prefix(".restore-protocol-").is_some_and(|v|valid_hex(v,32)){return Err(Error::UnsupportedProfile);}staged+=1;if staged>16{return Err(Error::Limit);}MAX}};let b=get(seed,n,limit)?;used=used.checked_add(b.len()).ok_or(Error::Limit)?;if used>64*1024*1024{return Err(Error::Limit);}values.insert(n.to_owned(),digest(&b));}
 Ok(digest(&serde_json::to_vec(&values).map_err(|_|Error::Unavailable)?))
}
/// Recognize only this typed job. Metadata fingerprints never admit its seed.
pub(in crate::pet_data)fn job_fingerprint(config:&Dir)->Result<String>{
 let header:Header=parse(&get(config,HEADER,4096)?)?;if header.version!=2{return Err(Error::UnsupportedProfile);}let job=config.dir(OsStr::new(JOB))?;
 for n in job.node()?.names(5)?{if ![MANIFEST,HISTORY,READY,SEED,GENERATION].contains(&n.to_str().ok_or(Error::UnsafePath)?){return Err(Error::UnsupportedProfile);}}
 let bytes=get(&job,MANIFEST,MAX)?;let m:Intent=parse(&bytes)?;m.validate(config)?;if m.pair!=header.pair||m.layout!=header.layout||!valid_hex(&m.source,64){return Err(Error::UnsupportedProfile);}
 let generation_intent=read_file(&job,GENERATION,4096)?;if generation_intent.as_deref().is_some_and(|b|generation(&bytes).ok().as_deref()!=Some(b)){return Err(Error::SourceChanged);}
 let history=read_file(&job,HISTORY,MAX)?;if let Some(b)=&history{let hash=m.references["indexHash"].as_str().ok_or(Error::UnsupportedProfile)?;if History::decode_first(b,hash)?.review()?!=m.references{return Err(Error::SourceChanged);}}
 let seed=if job.node()?.child(OsStr::new(SEED))?.is_some(){let d=job.dir(OsStr::new(SEED))?;if generation_intent.is_none()&&!d.node()?.names(1)?.is_empty(){return Err(Error::SourceChanged);}Some(seed_hash(&d)?)}else{None};let ready=read_file(&job,READY,64*1024)?;
 if let Some(b)=&ready{let r:Ready=parse(b)?;if r.version!=1||r.manifest!=digest(&bytes)||seed.as_ref()!=Some(&r.seed)||history.is_none()||generation_intent.is_none(){return Err(Error::SourceChanged);}}
 let loc=private_fs::location(&job.node()?.file)?;Ok(digest(&serde_json::to_vec(&(loc.device,loc.inode,loc.mount,digest(&bytes),history.as_deref().map(digest),generation_intent.as_deref().map(digest),seed,ready.as_deref().map(digest))).map_err(|_|Error::Unavailable)?))
}
impl IndependentReview{
 pub(in crate::pet_data)fn capture(data:Review)->Result<Self>{
  if data.profile_version()!=2||data.source.archive_version()!=1{return Err(Error::UnsupportedProfile);}data.authority_data_current()?;
  let retention=super::retention::binding(&Dir::root(data.layout.config())?)?;
  let source=Dir::root(data.source.source_root(Area::Config))?;let target=Dir::root(data.layout.config())?.dir(OsStr::new(PAYLOAD))?;
  for root in [&source,&target]{if root.node()?.child(OsStr::new("pet-runtime-consent-v1"))?.is_some(){return Err(Error::UnsupportedProfile);}}
  data.authority_verify_delta(&[])?;let scope=IndependentScope::capture(&target.bound_path()?.join("pet-runtime-v1"),&data.source_fence).map_err(core)?;
  let library=read_file(&Dir::root(data.source.source_root(Area::Data))?,"assistant/conversations.json",crate::assistant_memory::MAX_BYTES as usize)?;let history=History::capture(library.as_deref())?;
  let settings=read_file(&source,"settings.json",256*1024)?;let pets=read_file(&source,"pets.json",256*1024)?;
  let intent=Intent{version:if retention.is_some(){2}else{1},purpose:if retention.is_some(){"independentFutureAuthorityAfterRetention"}else{"independentFutureAuthorityPreparation"}.into(),pair:data.pair.clone().ok_or(Error::UnsupportedProfile)?,layout:layout_key(&data.layout),source:data.source_fence.clone(),goals:scope.review().map_err(core)?,references:history.review()?,configuration:super::configuration::source_report(settings.as_deref(),pets.as_deref())?,retention};
  let bytes=serde_json::to_vec(&intent).map_err(|_|Error::Unavailable)?;if bytes.len()>MAX{return Err(Error::Limit);}let r=Self{data,scope,intent,bytes,history:history.encode()?};r.current()?;Ok(r)
 }
 fn current(&self)->Result<()>{
  self.data.authority_data_current()?;self.data.authority_verify_delta(&[])?;
  let target=Dir::root(self.data.layout.config())?.dir(OsStr::new(PAYLOAD))?;if target.node()?.child(OsStr::new("pet-runtime-consent-v1"))?.is_some(){return Err(Error::SourceChanged);}if self.scope.review().map_err(core)?!=self.intent.goals{return Err(Error::SourceChanged);}
  let root=Dir::root(self.data.layout.config())?;self.intent.validate(&root)?;if root.node()?.child(OsStr::new(JOB))?.is_some(){let job=root.dir(OsStr::new(JOB))?;if get(&job,MANIFEST,MAX)?!=self.bytes{return Err(Error::SourceChanged);}job_fingerprint(&root)?;if let Some(b)=read_file(&job,HISTORY,MAX)?{if b!=self.history{return Err(Error::SourceChanged);}}if let Some(b)=read_file(&job,READY,64*1024)?{let ready:Ready=parse(&b)?;if self.scope.verify(&job.dir(OsStr::new(SEED))?.bound_path()?).map_err(core)?!=ready.authority{return Err(Error::SourceChanged);}}}
  self.data.current()
 }
 fn fingerprint(&self)->Result<String>{self.current()?;Ok(digest(&serde_json::to_vec(&("coucou-independent-authority-preparation-v1",self.data.fingerprint()?,&self.intent)).map_err(|_|Error::Unavailable)?))}
 fn preparation_state(&self)->Result<&'static str>{
  let root=Dir::root(self.data.layout.config())?;if root.node()?.child(OsStr::new(JOB))?.is_none(){return Ok("notStarted");}
  let job=root.dir(OsStr::new(JOB))?;if read_file(&job,READY,64*1024)?.is_some(){return Ok("prepared");}
  if read_file(&job,GENERATION,4096)?.is_none(){return Ok("notStarted");}
  Ok(match self.scope.key_state(&job.bound_path()?.join(SEED)).map_err(core)?{
   IndependentKeyState::Missing=>"keyGenerationUncertain",IndependentKeyState::Valid=>"keyPresent",IndependentKeyState::Invalid=>"keyInvalid",
  })
 }
 pub(in crate::pet_data)fn preview(&self)->Result<Value>{self.current()?;let state=self.preparation_state()?;Ok(json!({"preparationState":state,"manualReconciliationRequired":matches!(state,"keyGenerationUncertain"|"keyInvalid"),"keyGenerationAvailable":state=="notStarted","keyRegenerationAllowed":false,"status":"independentAuthorityPreparationReview","schemaVersion":self.intent.version,"purpose":self.intent.purpose,"retention":self.intent.retention,"retainedBytesTrustedAsAuthority":false,"fingerprint":self.fingerprint()?,"pair":self.intent.pair,"source":self.intent.source,"goals":self.intent.goals,"references":self.intent.references,"configuration":self.intent.configuration,"originalAuthorityPresent":false,"keyGenerationRequiresExactApproval":true,"executionAuthorityGranted":false,"authorityPublishedToRuntime":false,"runtimeActivationAvailable":false}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<Value,Failure>{self.apply_using(approved,|_|Ok(()))}
 fn apply_using(self,approved:&str,mut step:impl FnMut(&str)->Result<()>)->std::result::Result<Value,Failure>{
  let pair=Some(self.intent.pair.clone());let result=(||{
   if approved!=self.fingerprint()?{return Err(Error::SourceChanged);}self.current()?;
   if matches!(self.preparation_state()?,"keyGenerationUncertain"|"keyInvalid"){return Err(Error::SourceChanged);}
   let root=Dir::root(self.data.layout.config())?;let job=if root.node()?.child(OsStr::new(JOB))?.is_some(){root.dir(OsStr::new(JOB))?}else{let job=root.mkdir(OsStr::new(JOB))?;job.write(MANIFEST,&self.bytes)?;job};step("intent")?;self.current()?;
   if read_file(&job,HISTORY,MAX)?.is_none(){job.write(HISTORY,&self.history)?;}self.current()?;
   let seed=if job.node()?.child(OsStr::new(SEED))?.is_some(){job.dir(OsStr::new(SEED))?}else{job.mkdir(OsStr::new(SEED))?};self.current()?;
   let authority=if read_file(&job,READY,64*1024)?.is_some(){self.scope.verify(&seed.bound_path()?).map_err(core)?}else{
    // Persist once BEFORE RNG. A later empty seed cannot mean permission to
    // repeat generation: the first key may have existed and been lost.
    let new_generation=if read_file(&job,GENERATION,4096)?.is_some(){false}else{if !seed.node()?.names(1)?.is_empty(){return Err(Error::SourceChanged);}self.current()?;job.write(GENERATION,&generation(&self.bytes)?)?;step("generation")?;self.current()?;true};
    self.scope.prepare(&seed.bound_path()?,new_generation,||self.current().map_err(|_|crate::pet_runtime::restoration::HistoryFailure::Changed),|s|step(match s{IndependentStep::KeyPersisted=>"key",IndependentStep::BaseReady=>"base",IndependentStep::HistoryPrepared=>"history"}).map_err(|_|crate::pet_runtime::restoration::HistoryFailure::Changed)).map_err(core)?};
   self.current()?;let ready=Ready{version:1,manifest:digest(&self.bytes),seed:seed_hash(&seed)?,authority:authority.clone()};let b=serde_json::to_vec(&ready).map_err(|_|Error::Unavailable)?;if let Some(saved)=read_file(&job,READY,64*1024)?{if saved!=b{return Err(Error::SourceChanged);}}else{job.write(READY,&b)?;}step("ready")?;self.current()?;
   Ok(json!({"status":"independentAuthorityPrepared","schemaVersion":self.intent.version,"retention":self.intent.retention,"pair":self.intent.pair,"manifestHash":digest(&self.bytes),"receiptHash":digest(&b),"authority":authority,"originalAuthorityPresent":false,"historicalAuthorityRenewed":false,"executionAuthorityGranted":false,"authorityPublishedToRuntime":false,"runtimeActivationAvailable":false,"operationalAdmissionPending":true}))
  })();result.map_err(|error|Failure{error,pair})
 }
}
#[cfg(test)]#[path="publication_restore_independent_tests.rs"]mod tests;

#[path="publication_restore_independent_operational.rs"]pub(in crate::pet_data)mod operational;
