//! Native Header3 review/custody around the existing Authority epoch extension.
//! The copied old admission remains DATA and Header3 is still not executable.
use super::*;
use crate::pet_data::{restored_transaction::{self as tx,read_file},staging::Source};
use crate::pet_runtime::restoration;
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Plan{version:u32,purpose:String,pair:String,layout:String,source:String,parent:String,proposed:String,core:String,config:BTreeMap<String,String>,data:BTreeMap<String,String>}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Ready{version:u32,manifest:String,review:String}
pub(in crate::pet_data)struct EpochAuthorityReview{data:Review,core:restoration::EpochReview,plan:Plan,bytes:Vec<u8>}
fn core_error(e:restoration::HistoryFailure)->Error{super::authority::error(e)}
fn operational(data:&Review)->Result<PathBuf>{Ok(tx::payload_at_depth(&Dir::root(data.layout.config())?,data.operational_depth()?)?.bound_path()?)}
fn consent(data:&Review)->Result<PathBuf>{Ok(operational(data)?.join("pet-runtime-consent-v1"))}
fn controls(root:&Dir,names:&[&str])->Result<BTreeMap<String,String>>{let mut values=BTreeMap::new();for n in names{let b=read_file(root,n,256*1024)?.ok_or(Error::SourceChanged)?;values.insert((*n).into(),digest(&b));}Ok(values)}
impl EpochAuthorityReview{
 pub(in crate::pet_data)fn capture(data:Review)->Result<Self>{let(core,plan,bytes)=capture_parts(&data)?;let r=Self{data,core,plan,bytes};r.current()?;Ok(r)}
 fn current(&self)->Result<()>{verify_current(&self.data,&self.core,&self.plan,&self.bytes)}
 fn verify_originals(&self,job:&Dir)->Result<()>{verify_originals(&self.data,job)}
 fn fingerprint(&self)->Result<String>{self.current()?;Ok(digest(&serde_json::to_vec(&("coucou-native-next-epoch-authority-v1",self.data.fingerprint()?,&self.plan)).map_err(|_|Error::Unavailable)?))}
 pub(in crate::pet_data)fn preview(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"status":"restoredEpochAuthorityReview","schemaVersion":2,"destinationProfileVersion":3,"fingerprint":self.fingerprint()?,"pair":self.plan.pair,"source":self.plan.source,"authority":self.core.preview().map_err(core_error)?,"originalRootPairRetainedBeforeChanges":true,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"sensitiveSettingsNotActivated":true}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<serde_json::Value,Failure>{self.apply_using(approved,||Ok(()))}
 fn apply_using(self,approved:&str,originals_ready:impl FnMut()->Result<()>)->std::result::Result<serde_json::Value,Failure>{self.apply_steps(approved,originals_ready,|_|Ok(()))}
 fn apply_steps(self,approved:&str,mut originals_ready:impl FnMut()->Result<()>,mut copy_step:impl FnMut(usize)->Result<()>)->std::result::Result<serde_json::Value,Failure>{
  let pair=Some(self.plan.pair.clone());let result=(||->Result<serde_json::Value>{if approved!=self.fingerprint()?{return Err(Error::SourceChanged);}self.current()?;
   let config=Dir::root(self.data.layout.config())?;let job=if config.node()?.child(OsStr::new(tx::JOB))?.is_some(){config.dir(OsStr::new(tx::JOB))?}else{let d=config.mkdir(OsStr::new(tx::JOB))?;d.publish_new(tx::MANIFEST,&self.bytes)?;d};
   let originals=if job.node()?.child(OsStr::new("originals"))?.is_some(){job.dir(OsStr::new("originals"))?}else{job.mkdir(OsStr::new("originals"))?};let mut roots=vec![];for n in ["config","data"]{roots.push(if originals.node()?.child(OsStr::new(n))?.is_some(){originals.dir(OsStr::new(n))?}else{originals.mkdir(OsStr::new(n))?});}
   if read_file(&job,tx::READY,4096)?.is_none(){
    // Only this copy phase is immutable. These witnesses retain the revisions
    // already verified in full; they neither admit bytes nor convey authority.
    // Drop them before the intentional Authority delta below.
    {
     self.current()?;
     let source=self.data.source.copy_revision()?;
     let target=crate::pet_data::revision_fence::Tree::capture(config.dir(OsStr::new(PAYLOAD))?,Dir::root(self.data.layout.data())?.dir(OsStr::new(PAYLOAD))?)?;
     let config_controls=crate::pet_data::revision_fence::Controls::capture(&config,&[HEADER,COMPLETE,INTENT,ORIGIN,RESTORED])?;
     let data_controls=crate::pet_data::revision_fence::Controls::capture(&Dir::root(self.data.layout.data())?,&[HEADER,COMPLETE,INTENT,ORIGIN])?;
     let job_controls=crate::pet_data::revision_fence::Controls::capture(&job,&[tx::MANIFEST])?;
     self.current()?;
     staging::resume_publication(&self.data.source,&roots[0],&roots[1],|index|{
      copy_step(index)?;self.data.owner_current()?;source.current()?;target.current()?;
      config_controls.current()?;data_controls.current()?;job_controls.current()?;
      if read_file(&job,tx::READY,4096)?.is_some(){return Err(Error::SourceChanged);}Ok(())
     })?;
    }
    self.verify_originals(&job)?;self.current()?;let r=serde_json::to_vec(&Ready{version:1,manifest:digest(&self.bytes),review:approved.into()}).map_err(|_|Error::Unavailable)?;job.publish_new(tx::READY,&r)?;
   }else{self.verify_originals(&job)?;}
   self.current()?;originals_ready()?;self.current()?;let fp=self.core.fingerprint().map_err(core_error)?;let result=self.core.apply(&fp,||self.current().map_err(|_|restoration::HistoryFailure::Changed)).map_err(core_error)?;self.current()?;self.verify_originals(&job)?;
   let history=restoration::historical_view(&consent(&self.data)?).map_err(core_error)?;if history["historyHash"].as_str()!=Some(self.plan.proposed.as_str()){return Err(Error::SourceChanged);}
   Ok(serde_json::json!({"status":"restoredEpochAuthorityQuarantined","schemaVersion":2,"pair":self.plan.pair,"history":history,"authority":result,"originalRootPairRetained":true,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"sensitiveSettingsNotActivated":true}))
  })();result.map_err(|error|Failure{error,pair})
 }
}
fn verify_delta(data:&Review,delta:&[(String,Vec<u8>)])->Result<()>{verify_delta_using(data,delta,&[])}
fn verify_delta_using(data:&Review,delta:&[(String,Vec<u8>)],extra:&[(Area,PathBuf,Vec<u8>)])->Result<()>{
 let config=Dir::root(data.layout.config())?.dir(OsStr::new(PAYLOAD))?;let target_data=Dir::root(data.layout.data())?.dir(OsStr::new(PAYLOAD))?;let prefix=tx::payload_prefix(data.operational_depth()?-1)?.join("pet-runtime-consent-v1");let mut delta=delta.iter().map(|(n,b)|(Area::Config,prefix.join(n),b.clone())).collect::<Vec<_>>();delta.extend_from_slice(extra);staging::verify_publication_delta(&data.source,&config,&target_data,&delta)?;data.restored_data_current()
}

#[cfg(test)]impl EpochAuthorityReview{
 pub(in crate::pet_data)fn fixture_stop_before_authority(self,approved:&str)->std::result::Result<serde_json::Value,Failure>{self.apply_using(approved,||Err(Error::Unavailable))}
 pub(in crate::pet_data)fn fixture_during_originals(self,approved:&str,step:impl FnMut(usize)->Result<()>)->std::result::Result<serde_json::Value,Failure>{self.apply_steps(approved,||Ok(()),step)}
}


fn capture_parts(data:&Review)->Result<(restoration::EpochReview,Plan,Vec<u8>)>{capture_parts_using(data,&[])}
fn capture_parts_using(data:&Review,extra:&[(Area,PathBuf,Vec<u8>)])->Result<(restoration::EpochReview,Plan,Vec<u8>)>{
  if data.profile_version()!=3{return Err(Error::UnsupportedProfile);}data.restored_data_current()?;let config=Dir::root(data.layout.config())?;let data_dir=Dir::root(data.layout.data())?;
  let delta=restoration::epoch_publication_delta(&consent(data)?).map_err(core_error)?;verify_delta_using(data,&delta,extra)?;
  let source=Dir::root(data.source.source_root(Area::Config))?;let parent=digest(&read_file(&source,tx::payload_prefix(data.operational_depth()?-1)?.join("pet-runtime-consent-v1/.restored-history.json").to_str().ok_or(Error::UnsafePath)?,8*1024*1024)?.ok_or(Error::UnsupportedProfile)?);
  let core=restoration::EpochReview::capture(&consent(data)?,&operational(data)?.join("pet-runtime-v1"),&data.source_fence,||data.restored_data_current().map_err(|_|restoration::HistoryFailure::Changed)).map_err(core_error)?;
  let p=core.preview().map_err(core_error)?;if p["historyEpochs"].as_u64()!=Some(u64::from(data.operational_depth()?)){return Err(Error::UnsupportedProfile);}if p["parentHistoryHash"].as_str()!=Some(parent.as_str()){return Err(Error::SourceChanged);}
  let plan=Plan{version:2,purpose:"nextEpochAuthorityInitialization".into(),pair:data.pair.clone().ok_or(Error::UnsupportedProfile)?,layout:layout_key(&data.layout),source:data.source_fence.clone(),parent,proposed:p["historyHash"].as_str().ok_or(Error::UnsupportedProfile)?.into(),core:core.transaction_fingerprint().map_err(core_error)?,config:controls(&config,&[HEADER,COMPLETE,INTENT,ORIGIN,RESTORED])?,data:controls(&data_dir,&[HEADER,COMPLETE,INTENT,ORIGIN])?};let bytes=serde_json::to_vec(&plan).map_err(|_|Error::Unavailable)?;if bytes.len()>64*1024{return Err(Error::Limit);}
  Ok((core,plan,bytes))
}

fn verify_current(data:&Review,core:&restoration::EpochReview,plan:&Plan,bytes:&[u8])->Result<()>{verify_current_using(data,core,plan,bytes,&[])}
fn verify_current_using(data:&Review,core:&restoration::EpochReview,plan:&Plan,bytes:&[u8],extra:&[(Area,PathBuf,Vec<u8>)])->Result<()>{
  data.restored_data_current()?;for(root,expected)in[(Dir::root(data.layout.config())?,&plan.config),(Dir::root(data.layout.data())?,&plan.data)]{for(n,h)in expected{let b=read_file(&root,n,256*1024)?.ok_or(Error::SourceChanged)?;if digest(&b)!=*h{return Err(Error::SourceChanged);}}}
  let delta=core.checked_delta().map_err(core_error)?;verify_delta_using(data,&delta,extra)?;let config=Dir::root(data.layout.config())?;
  if config.node()?.child(OsStr::new(tx::JOB))?.is_some(){let job=config.dir(OsStr::new(tx::JOB))?;if read_file(&job,tx::MANIFEST,64*1024)?.as_deref()!=Some(bytes){return Err(Error::SourceChanged);}if let Some(b)=read_file(&job,tx::READY,4096)?{let r:Ready=parse(&b)?;if r.version!=1||r.manifest!=digest(&bytes)||!valid_hex(&r.review,64){return Err(Error::SourceChanged);}verify_originals(data,&job)?;}}
  data.current()
}

fn verify_originals(data:&Review,job:&Dir)->Result<()>{let originals=job.dir(OsStr::new("originals"))?;staging::verify_publication(&data.source,&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?)?;Ok(())}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(in crate::pet_data)struct Custody{pub manifest:String,pub ready:String,pub authority:String}
/// Complete C3 custody, not a promise to finish a partial authority publication.
pub(in crate::pet_data)fn completed(data:&Review)->Result<Custody>{completed_using(data,&[])}
pub(in crate::pet_data)fn completed_using(data:&Review,extra:&[(Area,PathBuf,Vec<u8>)])->Result<Custody>{
 let(core,plan,bytes)=capture_parts_using(data,extra)?;verify_current_using(data,&core,&plan,&bytes,extra)?;
 let view=restoration::historical_view(&consent(data)?).map_err(core_error)?;
 if view["historyHash"].as_str()!=Some(plan.proposed.as_str())||view["historyBodyVersion"]!=2{return Err(Error::UnsupportedProfile);}
 let job=Dir::root(data.layout.config())?.dir(OsStr::new(tx::JOB))?;let ready=read_file(&job,tx::READY,4096)?.ok_or(Error::UnsupportedProfile)?;verify_originals(data,&job)?;verify_current_using(data,&core,&plan,&bytes,extra)?;
 Ok(Custody{manifest:digest(&bytes),ready:digest(&ready),authority:plan.proposed})
}

/// Runtime checks are self-contained. No external archive/source or old
/// mutable grant snapshot is required after a sealed admission has opened.
pub(in crate::pet_data)fn verify_retained_custody(config:&Dir,expected:&Custody,pair:&str,layout:&str,source:&str)->Result<()>{
 let job=config.dir(OsStr::new(tx::JOB))?;let bytes=read_file(&job,tx::MANIFEST,64*1024)?.ok_or(Error::SourceChanged)?;let ready=read_file(&job,tx::READY,4096)?.ok_or(Error::SourceChanged)?;
 if digest(&bytes)!=expected.manifest||digest(&ready)!=expected.ready{return Err(Error::SourceChanged);}let p:Plan=parse(&bytes)?;let r:Ready=parse(&ready)?;
 if p.version!=2||p.purpose!="nextEpochAuthorityInitialization"||p.pair!=pair||p.layout!=layout||p.source!=source||p.proposed!=expected.authority||r.version!=1||r.manifest!=expected.manifest||!valid_hex(&r.review,64){return Err(Error::UnsupportedProfile);}
 for(n,h)in p.config{if digest(&read_file(config,&n,256*1024)?.ok_or(Error::SourceChanged)?)!=h{return Err(Error::SourceChanged);}}Ok(())
}
