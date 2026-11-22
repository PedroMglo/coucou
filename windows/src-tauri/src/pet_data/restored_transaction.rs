//! Reviewed initial data transformation, with the original archive retained
//! before replacement. No model/runtime effects occur before Admission2.
use super::{publication::{self,restore::Review},inventory::Area,staging::{self,Source},staging_fs::Dir,private_fs,restored_projection::{Choices,Projection},restored_history::History,Error,Layout,Result,startup::Claim};
use std::{path::{Path,PathBuf},ffi::OsStr,io::Read,collections::BTreeMap};
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
pub(crate)const JOB:&str=".restore-activation";
pub(crate)const EPOCH_JOB:&str=".restore-admission";
pub(crate)const MANIFEST:&str="manifest.json";
pub(crate)const READY:&str="ready.json";
pub(crate)const HISTORY:&str="history.json";
pub(crate)const SEALED:&str="sealed.json";
pub(crate)const OPENED:&str="opened.json";
const MAX_MANIFEST:usize=64*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Change{pub area:Area,pub path:String,pub old:Option<String>,pub new:String,pub temp:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Manifest{pub version:u32,pub pair:String,pub layout:String,pub archive_id:String,pub source:String,pub original_content:String,pub choices:Choices,pub inspection:String,pub authority:String,pub history:String,pub changes:Vec<Change>,pub config_controls:BTreeMap<String,String>,pub data_controls:BTreeMap<String,String>,pub pending_features:Vec<String>,#[serde(default,skip_serializing_if="Option::is_none")]pub epoch:Option<EpochInitialization>,#[serde(default,skip_serializing_if="Option::is_none")]pub independent:Option<publication::restore::independent::operational::Custody>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct EpochInitialization{pub payload_depth:u8,pub scope:publication::restore::configuration::EpochScope}
/// Same bound as eight authenticated prior histories plus the current epoch.
pub(crate)const MAX_PAYLOAD_DEPTH:u8=9;
impl EpochInitialization{
 pub(crate)fn depth(&self,history:&str,authority:&str)->Result<u8>{if !(2..=MAX_PAYLOAD_DEPTH).contains(&self.payload_depth)||self.scope.references["schemaVersion"]!=2||self.scope.references["indexHash"].as_str()!=Some(history)||self.scope.custody.authority!=authority||self.scope.references["lineage"].as_array().is_none_or(|a|a.len()!=usize::from(self.payload_depth-1)){return Err(Error::UnsupportedProfile);}Ok(self.payload_depth)}
}
impl Manifest{
 pub(crate)fn profile_version(&self)->Result<u32>{match(self.version,&self.epoch,&self.independent){(1,None,None)=>Ok(2),(3,Some(e),None)=>{e.depth(&self.history,&self.authority)?;Ok(3)},(4,None,Some(i))=>{i.validate(&self.history,&self.authority)?;Ok(2)},_=>Err(Error::UnsupportedProfile)}}
 pub(crate)fn admission_version(&self)->Result<u32>{Ok(if self.independent.is_some(){self.profile_version()?;4}else{self.profile_version()?})}
 pub(crate)fn depth(&self)->Result<u8>{self.profile_version()?;match &self.epoch{None=>Ok(1),Some(e)=>e.depth(&self.history,&self.authority)}}
 pub(crate)fn job(&self)->Result<&'static str>{Ok(if self.profile_version()?==3{EPOCH_JOB}else{JOB})}
 pub(crate)fn payload(&self,root:&Dir)->Result<Dir>{payload_at_depth(root,self.depth()?)}
 pub(crate)fn source_relative(&self,path:&Path)->Result<PathBuf>{Ok(payload_prefix(self.depth()?-1)?.join(path))}
}
pub(crate)fn payload_prefix(depth:u8)->Result<PathBuf>{if depth>MAX_PAYLOAD_DEPTH{return Err(Error::Limit);}let mut p=PathBuf::new();for _ in 0..depth{p.push(publication::PAYLOAD);}Ok(p)}
pub(crate)fn payload_at_depth(root:&Dir,depth:u8)->Result<Dir>{if !(1..=MAX_PAYLOAD_DEPTH).contains(&depth){return Err(Error::Limit);}let mut p=root.clone();for _ in 0..depth{p=p.dir(OsStr::new(publication::PAYLOAD))?;}Ok(p)}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Ready{pub version:u32,pub manifest:String,pub review:String}
pub(crate)struct Activation{data:Review,choices:Choices,manifest:Manifest,manifest_bytes:Vec<u8>,projection:Projection,history:Vec<u8>,fingerprint:String}
fn hash(b:&[u8])->String{publication::digest(b)}
fn encode<T:Serialize>(v:&T)->Result<Vec<u8>>{let b=serde_json::to_vec(v).map_err(|_|Error::UnsupportedProfile)?;if b.len()>MAX_MANIFEST{return Err(Error::Limit);}Ok(b)}
fn area_root(layout:&Layout,area:Area)->&Path{match area{Area::Config=>layout.config(),Area::Data=>layout.data()}}
fn relative(path:&str)->Result<&Path>{let p=Path::new(path);if p.is_absolute()||p.components().any(|c|!matches!(c,std::path::Component::Normal(_)))||path.len()>4096{return Err(Error::UnsafePath);}Ok(p)}
pub(crate)fn read_file(root:&Dir,path:&str,limit:usize)->Result<Option<Vec<u8>>>{
 let path=relative(path)?;let mut parent=root.clone();if let Some(ancestors)=path.parent(){for n in ancestors.components(){parent=match parent.node()?.child(n.as_os_str())?{None=>return Ok(None),Some(node)if node.directory=>parent.dir(n.as_os_str())?,_=>return Err(Error::UnsafePath)};}}
 let name=path.file_name().ok_or(Error::UnsafePath)?;let at=parent.node()?;let Some(node)=at.child(name)?else{return Ok(None);};if node.directory||node.len>limit as u64||!super::staging_fs::private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}let mut b=vec![];(&node.file).take(limit as u64+1).read_to_end(&mut b).map_err(|_|Error::Unavailable)?;node.current()?;at.same_child(name,&node)?;if b.len()>limit||b.len()as u64!=node.len{return Err(Error::SourceChanged);}Ok(Some(b))
}
fn root_control(root:&Dir,names:&[&str])->Result<BTreeMap<String,String>>{let mut out=BTreeMap::new();for n in names{let b=read_file(root,n,256*1024)?.ok_or(Error::UnsupportedProfile)?;out.insert((*n).into(),hash(&b));}Ok(out)}
pub(crate)fn job_fingerprint(root:&Dir)->Result<String>{job_fingerprint_named(root,JOB)}
pub(crate)fn job_fingerprint_named(root:&Dir,name:&str)->Result<String>{
 if !matches!(name,JOB|EPOCH_JOB){return Err(Error::UnsupportedProfile);}let job=root.dir(OsStr::new(name))?;let mut staged=0;for n in job.node()?.names(21)?{let name=n.to_str().ok_or(Error::UnsupportedProfile)?;if [MANIFEST,READY,HISTORY,SEALED,OPENED].contains(&name)||name=="originals"&&job.bound_path()?.file_name()==Some(OsStr::new(JOB)){continue;}if !name.strip_prefix(".publish-").is_some_and(|s|s.len()==32&&s.bytes().all(|b|b.is_ascii_hexdigit())){return Err(Error::UnsupportedProfile);}staged+=1;if staged>16{return Err(Error::Limit);}read_file(&job,name,super::restored_history::MAX_INDEX)?.ok_or(Error::SourceChanged)?;}let at=private_fs::location(&job.node()?.file)?;let manifest=read_file(&job,MANIFEST,MAX_MANIFEST)?;let ready=read_file(&job,READY,4096)?;
 Ok(hash(&serde_json::to_vec(&(at.device,at.inode,at.mount,manifest.as_deref().map(hash),ready.as_deref().map(hash))).map_err(|_|Error::Unavailable)?))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Step{OriginalsReady,BeforeChange(usize),Staged(usize),Changed(usize),BeforeAdmission,Admitted,AuthorityStaged(usize),AuthorityPublished}
impl Activation{
 pub(crate)fn capture(claim:Claim,layout:Layout,workspace:&Path,id:&str,choices:Choices)->Result<Self>{
  let data=Review::capture(claim,layout,workspace,id,publication::restore::Mode::Activation)?;let version=data.profile_version();let depth=data.operational_depth()?;let pair=data.pair.clone().ok_or(Error::UnsupportedProfile)?;let config=Dir::root(data.layout.config())?;
  let operational=payload_at_depth(&config,depth)?;let auth_path=operational.bound_path()?.join("pet-runtime-consent-v1");let independent_mode=version==2&&private_fs::Node::root(&data.source.source_root(Area::Config).join("pet-runtime-consent-v1"))?.is_none();let mut authority=if independent_mode{String::new()}else{let auth=crate::pet_runtime::restoration::historical_view(&auth_path).map_err(|_|Error::UnsupportedProfile)?;auth["historyHash"].as_str().ok_or(Error::UnsupportedProfile)?.to_owned()};
  let archive_config=Dir::root(data.source.source_root(Area::Config))?;let archive_data=Dir::root(data.source.source_root(Area::Data))?;
  let prefix=payload_prefix(depth-1)?;let archive_config=if depth>1{payload_at_depth(&archive_config,depth-1)?}else{archive_config};let archive_data=if depth>1{payload_at_depth(&archive_data,depth-1)?}else{archive_data};let _=prefix;
  let settings=read_file(&archive_config,"settings.json",256*1024)?;let pets=read_file(&archive_config,"pets.json",256*1024)?;let sources=read_file(&archive_config,"sources.json",256*1024)?;let flow=read_file(&archive_data,"Flow/prefs.json",256*1024)?;
  let report=publication::restore::configuration::source_report(settings.as_deref(),pets.as_deref())?;let mut projection=super::restored_projection::project(&report,&choices,sources.as_deref(),flow.as_deref())?;
  let settings_io=read_file(&archive_config,crate::settings::durable::FILE,crate::settings::durable::MAX_RECORD_BYTES)?;
  if let Some(original)=&settings_io{
   let proposed=crate::settings::durable::restored_record(original,settings.as_deref(),&projection.settings).map_err(|_|Error::UnsupportedProfile)?;
   projection.differences["settingsIo"]=json!({"original":hash(original),"proposed":hash(&proposed),"unresolvedStatePreserved":true,"authorityGranted":false});projection.settings_io=Some(proposed);
  }
  let mut changes=vec![];for(area,path,old,new)in[(Area::Config,"settings.json",settings.as_deref(),Some(projection.settings.as_slice())),(Area::Config,"pets.json",pets.as_deref(),Some(projection.pets.as_slice())),(Area::Config,"sources.json",sources.as_deref(),projection.sources.as_deref()),(Area::Data,"Flow/prefs.json",flow.as_deref(),projection.flow.as_deref()),(Area::Config,crate::settings::durable::FILE,settings_io.as_deref(),projection.settings_io.as_deref())]{if let Some(new)=new{if old!=Some(new){let temp=format!(".restore-replace-{}",&hash(format!("{pair}:{area:?}:{path}").as_bytes())[..32]);changes.push(Change{area,path:path.into(),old:old.map(hash),new:hash(new),temp});}}}
  let(inspection,history,epoch,independent)=if version==3{
   let delta=initialization_delta(&data,&changes,&projection,depth)?;let(_,receipt,history,scope)=publication::restore::configuration::source_inspection_next(&data,settings.as_deref(),pets.as_deref(),&delta)?;
   (receipt,history,Some(EpochInitialization{payload_depth:depth,scope}),None)
  }else if independent_mode{let delta=initialization_delta(&data,&changes,&projection,depth)?;let(_,receipt,history,scope)=publication::restore::configuration::source_inspection_independent(&data,settings.as_deref(),pets.as_deref(),&delta)?;authority=scope.authority.clone();(receipt,history,None,Some(scope))}
  else{let(_,receipt)=publication::restore::configuration::source_inspection(settings.as_deref(),pets.as_deref(),&pair,&data.source_fence,&authority)?;let library=read_file(&archive_data,"assistant/conversations.json",crate::assistant_memory::MAX_BYTES as usize)?;(receipt,History::capture(library.as_deref())?.encode()?,None,None)};
  let inspection_name=match &independent{Some(i)=>publication::restore::configuration::independent_receipt_name(i)?,None=>publication::restore::CONFIG_REVIEW.into()};
  if hash(&inspection)!=choices.inspection_receipt||read_file(&config,&inspection_name,4096)?.as_deref()!=Some(inspection.as_slice()){return Err(Error::SourceChanged);}
  let mut config_controls=root_control(&config,&[publication::HEADER,publication::COMPLETE,publication::restore::INTENT,publication::restore::ORIGIN,publication::restore::RESTORED,&inspection_name])?;
  if let Some(e)=&epoch{config_controls.insert(format!("{JOB}/{MANIFEST}"),e.scope.custody.manifest.clone());config_controls.insert(format!("{JOB}/{READY}"),e.scope.custody.ready.clone());}
  if let Some(i)=&independent{for(name,h)in[("manifest.json",&i.manifest),("ready.json",&i.ready),("generation.json",&i.generation),("history.json",&i.history)]{config_controls.insert(format!("{}/{name}",publication::restore::independent::JOB),h.clone());}}
  let manifest=Manifest{version:if independent_mode{4}else if version==3{3}else{1},pair,layout:publication::layout_key(&data.layout),archive_id:id.into(),source:data.source_fence.clone(),original_content:staging::content_fingerprint(&data.source)?,choices:choices.clone(),inspection:hash(&inspection),authority,history:hash(&history),changes,config_controls,data_controls:root_control(&Dir::root(data.layout.data())?,&[publication::HEADER,publication::COMPLETE,publication::restore::INTENT,publication::restore::ORIGIN])?,pending_features:vec!["autostart".into(),"mobile".into(),"voice".into()],epoch,independent};
  manifest.profile_version()?;let manifest_bytes=encode(&manifest)?;let fingerprint=hash(&serde_json::to_vec(&("coucou-restored-operational-activation-v1",data.fingerprint()?,&manifest)).map_err(|_|Error::Unavailable)?);
  let review=Self{data,choices,manifest,manifest_bytes,projection,history,fingerprint};review.verify_target()?;Ok(review)
 }
 pub(crate)fn preview(&self)->Result<Value>{self.data.current()?;self.verify_target()?;Ok(json!({"status":"restoredActivationReview","fingerprint":self.fingerprint,"pair":self.manifest.pair,"source":self.manifest.source,"choices":self.choices,"independent":self.manifest.independent,"changes":self.projection.differences,"historyReferences":History::decode(&self.history,&self.manifest.history)?.counts(),"pendingFeatures":self.manifest.pending_features,"originalsRetainedBeforeChanges":true,"executionAuthorityGranted":false,"activationRequiresExactApproval":true}))}
 fn proposed(&self,c:&Change)->Result<&[u8]>{proposed(&self.projection,c)}
 fn verify_target(&self)->Result<()>{
  self.data.current()?;let config=Dir::root(self.data.layout.config())?;let data=Dir::root(self.data.layout.data())?;
  for(root,controls)in[(&config,&self.manifest.config_controls),(&data,&self.manifest.data_controls)]{for(name,expected)in controls{if read_file(root,name,256*1024)?.as_deref().map(hash).as_ref()!=Some(expected){return Err(Error::SourceChanged);}}}
  let delta=initialization_delta(&self.data,&self.manifest.changes,&self.projection,self.manifest.depth()?)?;
  if let Some(i)=&self.manifest.independent{let p=publication::restore::independent::operational::capture(&self.data,&delta)?;if p.custody!=*i||p.history!=self.history{return Err(Error::SourceChanged);}}
  else if let Some(e)=&self.manifest.epoch{let custody=publication::restore::epoch::completed_using(&self.data,&delta)?;if custody!=e.scope.custody||History::decode(&self.history,&self.manifest.history)?.review()?!=e.scope.references{return Err(Error::SourceChanged);}}
  else{let payload=self.manifest.payload(&config)?;let data_payload=self.manifest.payload(&data)?;let mut all=crate::pet_runtime::restoration::publication_delta(&payload.bound_path()?.join("pet-runtime-consent-v1")).map_err(|_|Error::UnsupportedProfile)?.into_iter().map(|(n,b)|(Area::Config,PathBuf::from("pet-runtime-consent-v1").join(n),b)).collect::<Vec<_>>();all.extend(delta);staging::verify_publication_delta(&self.data.source,&payload,&data_payload,&all)?;}
  if config.node()?.child(OsStr::new(self.manifest.job()?))?.is_some(){let job=config.dir(OsStr::new(self.manifest.job()?))?;if read_file(&job,MANIFEST,MAX_MANIFEST)?.as_deref()!=Some(self.manifest_bytes.as_slice()){return Err(Error::SourceChanged);}let history=read_file(&job,HISTORY,super::restored_history::MAX_INDEX)?;if history.as_deref().is_some_and(|b|b!=self.history){return Err(Error::SourceChanged);}if let Some(ready)=read_file(&job,READY,4096)?{let ready:Ready=serde_json::from_slice(&ready).map_err(|_|Error::UnsupportedProfile)?;if ready.version!=1||ready.manifest!=hash(&self.manifest_bytes)||ready.review.len()!=64||history.is_none(){return Err(Error::SourceChanged);}}}
  self.data.current()
 }
 pub(crate)fn apply(self,approved:&str)->std::result::Result<Value,publication::Failure>{
  self.apply_using(approved,|_|Ok(()))
 }
 fn apply_using(self,approved:&str,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<Value,publication::Failure>{
  let pair=Some(self.manifest.pair.clone());let result=(||->Result<Value>{if approved!=self.fingerprint{return Err(Error::SourceChanged);}self.data.fingerprint()?;self.verify_target()?;
   let config=Dir::root(self.data.layout.config())?;let job=if config.node()?.child(OsStr::new(self.manifest.job()?))?.is_some(){config.dir(OsStr::new(self.manifest.job()?))?}else{let job=config.mkdir(OsStr::new(self.manifest.job()?))?;job.publish_new(MANIFEST,&self.manifest_bytes)?;job};
   let original_job=config.dir(OsStr::new(JOB))?;let originals=if original_job.node()?.child(OsStr::new("originals"))?.is_some(){original_job.dir(OsStr::new("originals"))?}else if self.manifest.epoch.is_none(){original_job.mkdir(OsStr::new("originals"))?}else{return Err(Error::SourceChanged);};
   let mut roots=vec![];for n in ["config","data"]{roots.push(if originals.node()?.child(OsStr::new(n))?.is_some(){originals.dir(OsStr::new(n))?}else{originals.mkdir(OsStr::new(n))?});}
   if self.manifest.epoch.is_none()&&read_file(&job,READY,4096)?.is_none(){staging::copy_publication(&self.data.source,&roots[0],&roots[1],|_|{self.data.current()?;self.verify_target()})?;}staging::verify_publication(&self.data.source,&roots[0],&roots[1])?;
   if let Some(b)=read_file(&job,HISTORY,super::restored_history::MAX_INDEX)?{if b!=self.history{return Err(Error::SourceChanged);}}else{if read_file(&job,READY,4096)?.is_some(){return Err(Error::SourceChanged);}self.data.current()?;job.publish_new(HISTORY,&self.history)?;}
   let ready=encode(&Ready{version:1,manifest:hash(&self.manifest_bytes),review:self.fingerprint.clone()})?;if let Some(b)=read_file(&job,READY,4096)?{let r:Ready=serde_json::from_slice(&b).map_err(|_|Error::UnsupportedProfile)?;if r.version!=1||r.manifest!=hash(&self.manifest_bytes)||r.review.len()!=64{return Err(Error::SourceChanged);}}else{self.data.current()?;job.publish_new(READY,&ready)?;}
   step(Step::OriginalsReady)?;self.verify_target()?;
   if self.manifest.independent.is_some(){let delta=initialization_delta(&self.data,&self.manifest.changes,&self.projection,self.manifest.depth()?)?;let p=publication::restore::independent::operational::capture(&self.data,&delta)?;p.publish(&self.manifest.payload(&config)?,&IndependentPublication{activation:&self},|i|step(Step::AuthorityStaged(i)))?;step(Step::AuthorityPublished)?;self.verify_target()?;}
   for(index,c)in self.manifest.changes.iter().enumerate(){step(Step::BeforeChange(index))?;self.verify_target()?;let root=self.manifest.payload(&Dir::root(area_root(&self.data.layout,c.area))?)?;let path=relative(&c.path)?;let mut parent=root;for n in path.parent().unwrap_or(Path::new("")).components(){parent=parent.dir(n.as_os_str())?;}let name=path.file_name().and_then(|n|n.to_str()).ok_or(Error::UnsafePath)?;let current=read_file(&parent,name,256*1024)?;
    if current.as_deref().map(hash).as_deref()==Some(c.new.as_str()){continue;}
    self.data.current()?;let new=self.proposed(c)?;
    parent.complete_prefix(&c.temp,new)?;step(Step::Staged(index))?;self.verify_target()?;
    if c.old.is_none(){self.data.current()?;parent.publish_staged_new(&c.temp,name)?;}else{
     // Revalidate after staging, then exchange; displaced actual bytes remain.
     let actual=read_file(&parent,name,256*1024)?;if actual.as_deref().map(hash)!=c.old{return Err(Error::SourceChanged);}self.data.current()?;parent.exchange_files(&c.temp,name)?;
    }
    step(Step::Changed(index))?;self.verify_target()?;
   }
   self.verify_target()?;staging::verify_publication(&self.data.source,&roots[0],&roots[1])?;self.data.current()?;
   step(Step::BeforeAdmission)?;self.verify_target()?;super::publication::active_restored::publish(&Approved{activation:&self})?;step(Step::Admitted)?;
   Ok(json!({"status":"restoredProfileAdmitted","pair":self.manifest.pair,"admissionHash":hash(&read_file(&config,publication::ADMITTED,4096)?.ok_or(Error::UnsupportedProfile)?),"pendingFeatures":self.manifest.pending_features,"historicalAuthorityRenewed":false,"executionAuthorityGranted":false,"runtimeActivationAvailable":true}))
  })();result.map_err(|error|publication::Failure{error,pair})
 }
}

fn proposed<'a>(projection:&'a Projection,c:&Change)->Result<&'a[u8]>{match(c.area,c.path.as_str()){(Area::Config,"settings.json")=>Ok(&projection.settings),(Area::Config,"pets.json")=>Ok(&projection.pets),(Area::Config,"sources.json")=>projection.sources.as_deref().ok_or(Error::UnsupportedProfile),(Area::Data,"Flow/prefs.json")=>projection.flow.as_deref().ok_or(Error::UnsupportedProfile),(Area::Config,crate::settings::durable::FILE)=>projection.settings_io.as_deref().ok_or(Error::UnsupportedProfile),_=>Err(Error::UnsupportedProfile)}}
fn initialization_delta(data:&Review,changes:&[Change],projection:&Projection,depth:u8)->Result<Vec<(Area,PathBuf,Vec<u8>)>>{
 let config=payload_at_depth(&Dir::root(data.layout.config())?,depth)?;let data_root=payload_at_depth(&Dir::root(data.layout.data())?,depth)?;let mut delta=vec![];
 let prefix=payload_prefix(depth-1)?;let prefixed=|p:&Path|prefix.join(p);
 for c in changes{let bytes=proposed(projection,c)?;if hash(bytes)!=c.new{return Err(Error::SourceChanged);}let root=match c.area{Area::Config=>&config,Area::Data=>&data_root};let actual=read_file(root,&c.path,256*1024)?;let actual_hash=actual.as_deref().map(hash);if actual_hash.as_ref()!=c.old.as_ref()&&actual_hash.as_deref()!=Some(c.new.as_str()){return Err(Error::SourceChanged);}if actual_hash.as_deref()==Some(c.new.as_str()){delta.push((c.area,prefixed(Path::new(&c.path)),actual.unwrap()));}
  let temp=relative(&c.path)?.parent().unwrap_or(Path::new("")).join(&c.temp);if let Some(old)=read_file(root,temp.to_str().ok_or(Error::UnsafePath)?,256*1024)?{let displaced=c.old.as_deref()==Some(hash(&old).as_str())&&actual_hash.as_deref()==Some(c.new.as_str());let staged=actual_hash.as_ref()==c.old.as_ref()&&bytes.starts_with(&old);if !displaced&&!staged{return Err(Error::SourceChanged);}delta.push((c.area,prefixed(&temp),old));}
 }Ok(delta)
}

/// Constructed only after exact native approval, final payload/original checks,
/// and the current offline Claim. Serialized plans cannot create this value.
pub(crate)struct Approved<'a>{activation:&'a Activation}
impl Approved<'_>{
 pub(crate)fn current(&self)->Result<()>{self.activation.verify_target()?;let job=Dir::root(self.activation.data.layout.config())?.dir(OsStr::new(JOB))?;let originals=job.dir(OsStr::new("originals"))?;staging::verify_publication(&self.activation.data.source,&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?)?;Ok(())}
 pub(crate)fn owner_current(&self)->Result<()>{self.activation.data.current()}
 pub(crate)fn layout(&self)->&Layout{&self.activation.data.layout}
 pub(crate)fn manifest(&self)->&Manifest{&self.activation.manifest}
 pub(crate)fn manifest_bytes(&self)->&[u8]{&self.activation.manifest_bytes}
}

#[cfg(test)]#[path="restored_transaction_tests.rs"]mod tests;

/// Only the approved Activation path constructs this token after retaining
/// originals/intent. JSON and a prepared seed cannot manufacture permission.
pub(in crate::pet_data)struct IndependentPublication<'a>{activation:&'a Activation}
impl IndependentPublication<'_>{
 pub(in crate::pet_data)fn custody(&self)->Option<&publication::restore::independent::operational::Custody>{self.activation.manifest.independent.as_ref()}
 pub(in crate::pet_data)fn current(&self)->Result<()>{self.activation.verify_target()?;let job=Dir::root(self.activation.data.layout.config())?.dir(OsStr::new(JOB))?;let originals=job.dir(OsStr::new("originals"))?;staging::verify_publication(&self.activation.data.source,&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?)?;Ok(())}
}
