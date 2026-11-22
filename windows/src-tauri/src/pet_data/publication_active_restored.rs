//! Admission2 is data custody plus exact reviewed initialization; it is never
//! executable goal consent. Its immutable history is loaded before any Chat.
use super::*;
use crate::pet_data::{restored_transaction::{self as tx,Manifest},restored_history::History};
use std::sync::Arc;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Admission2{pub version:u32,pub pair:String,pub layout:String,pub config:String,pub data:String,pub manifest:String,pub history:String,pub authority:String,#[serde(default,skip_serializing_if="Option::is_none")]pub epoch:Option<tx::EpochInitialization>,#[serde(default,skip_serializing_if="Option::is_none")]pub independent:Option<restore::independent::operational::Custody>}
impl Admission2{
 pub(crate)fn depth(&self)->Result<u8>{match(self.version,&self.epoch,&self.independent){(2,None,None)=>Ok(1),(3,Some(e),None)=>e.depth(&self.history,&self.authority),(4,None,Some(i))=>{i.validate(&self.history,&self.authority)?;Ok(1)},_=>Err(Error::UnsupportedProfile)}}
 pub(crate)fn profile_version(&self)->Result<u32>{self.depth()?;Ok(if self.version==3{3}else{2})}
}
struct Blob{parent:Dir,name:String,stamp:crate::pet_data::private_fs::Stamp}
impl Blob{
 fn capture(parent:&Dir,name:&str,max:usize)->Result<(Self,Vec<u8>)>{let p=parent.node()?;let n=p.child(OsStr::new(name))?.ok_or(Error::SourceChanged)?;if n.directory||n.len>max as u64||!staging_fs::private_file(&n.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}let mut b=vec![];(&n.file).take(max as u64+1).read_to_end(&mut b).map_err(|_|Error::Unavailable)?;if b.len()>max||b.len()as u64!=n.len{return Err(Error::SourceChanged);}n.current()?;p.same_child(OsStr::new(name),&n)?;Ok((Self{parent:parent.clone(),name:name.into(),stamp:n.initial},b))}
 fn current(&self)->Result<()>{let p=self.parent.node()?;let n=p.child(OsStr::new(&self.name))?.ok_or(Error::SourceChanged)?;if n.initial!=self.stamp{return Err(Error::SourceChanged);}n.current()?;p.same_child(OsStr::new(&self.name),&n)}
}
pub(super)struct Bound{pub(super)history:Arc<History>,manifest:Manifest,admission:Vec<u8>,blobs:Vec<Blob>,opened:bool}
fn required_controls(roots:&OwnedRoots,m:&Manifest)->Result<()>{
 for(root,expected)in[(&roots.config,&m.config_controls),(&roots.data,&m.data_controls)]{for(name,hash)in expected{let b=tx::read_file(root,name,if m.independent.is_some(){8*1024*1024}else{256*1024})?.ok_or(Error::SourceChanged)?;if digest(&b)!=*hash{return Err(Error::SourceChanged);}}}Ok(())
}
impl Bound{
 pub(super)fn open(roots:&OwnedRoots)->Result<Self>{
  let admission=read(&roots.config,ADMITTED)?.ok_or(Error::UnsupportedProfile)?;let a:Admission2=parse(&admission)?;
  let profile:Header=parse(&roots.config_header)?;if a.profile_version()?!=profile.version||a.pair!=roots.pair||a.layout!=layout_key(&roots.layout){return Err(Error::UnsupportedProfile);}
  let config=read(&roots.config,COMPLETE)?.ok_or(Error::UnsupportedProfile)?;let data=read(&roots.data,COMPLETE)?.ok_or(Error::UnsupportedProfile)?;
  for(b,area)in[(&config,Area::Config),(&data,Area::Data)]{let c:Complete=parse(b)?;if c.version!=a.profile_version()?||c.pair!=roots.pair||c.area!=area{return Err(Error::UnsupportedProfile);}}
  if digest(&config)!=a.config||digest(&data)!=a.data{return Err(Error::SourceChanged);}
  let name=if a.version==3{tx::EPOCH_JOB}else{tx::JOB};let job=roots.config.dir(OsStr::new(name))?;let(mblob,mb)=Blob::capture(&job,tx::MANIFEST,64*1024)?;if digest(&mb)!=a.manifest{return Err(Error::SourceChanged);}let m:Manifest=serde_json::from_slice(&mb).map_err(|_|Error::UnsupportedProfile)?;
  if m.admission_version()?!=a.version||m.profile_version()?!=a.profile_version()?||m.epoch!=a.epoch||m.independent!=a.independent||m.pair!=a.pair||m.layout!=a.layout||m.history!=a.history||m.authority!=a.authority||m.pending_features!=["autostart","mobile","voice"]{return Err(Error::UnsupportedProfile);}m.choices.canonical()?;required_controls(roots,&m)?;
  let(rblob,ready)=Blob::capture(&job,tx::READY,4096)?;let ready:tx::Ready=serde_json::from_slice(&ready).map_err(|_|Error::UnsupportedProfile)?;if ready.version!=1||ready.manifest!=a.manifest||!valid_hex(&ready.review,64){return Err(Error::SourceChanged);}
  let(sblob,sealed)=Blob::capture(&job,tx::SEALED,4096)?;let expected_seal=serde_json::json!({"version":1,"admission":digest(&admission),"manifest":a.manifest});if serde_json::from_slice::<serde_json::Value>(&sealed).map_err(|_|Error::UnsupportedProfile)?!=expected_seal{return Err(Error::SourceChanged);}
  let(hblob,hb)=Blob::capture(&job,tx::HISTORY,crate::pet_data::restored_history::MAX_INDEX)?;let history=Arc::new(if matches!(a.version,2|4){History::decode_first(&hb,&a.history)?}else{let h=History::decode(&hb,&a.history)?;if Some(h.review()?)!=m.epoch.as_ref().map(|e|e.scope.references.clone()){return Err(Error::SourceChanged);}h});
  let auth_path=m.payload(&roots.config)?.bound_path()?.join("pet-runtime-consent-v1");let view=crate::pet_runtime::restoration::historical_view(&auth_path).map_err(|_|Error::UnsupportedProfile)?;if view["historyHash"].as_str()!=Some(a.authority.as_str())||view["historyEpochs"].as_u64()!=Some(u64::from(m.depth()?)){return Err(Error::SourceChanged);}
  let auth=Dir::root(&auth_path)?;let mut blobs=vec![mblob,rblob,hblob,sblob];for(name,max)in[(".restored-history.json",8*1024*1024),(".protocol.json",1024),(".key.json",512)]{blobs.push(Blob::capture(&auth,name,max)?.0);}
  if let Some(i)=&m.independent{restore::independent::operational::verify_retained(&roots.config,i,&m.pair,&m.layout,&m.source)?;let custody=roots.config.dir(OsStr::new(restore::independent::JOB))?;for(name,max)in[("manifest.json",8*1024*1024),("ready.json",64*1024),("history.json",8*1024*1024),("generation.json",4096)]{blobs.push(Blob::capture(&custody,name,max)?.0);}}
  if let Some(e)=&m.epoch{
   restore::epoch::verify_retained_custody(&roots.config,&e.scope.custody,&m.pair,&m.layout,&m.source)?;
   let custody=roots.config.dir(OsStr::new(tx::JOB))?;for(name,max)in[(tx::MANIFEST,64*1024),(tx::READY,4096)]{blobs.push(Blob::capture(&custody,name,max)?.0);}
   for h in view["priorHistoryHashes"].as_array().ok_or(Error::UnsupportedProfile)?{let h=h.as_str().filter(|h|valid_hex(h,64)).ok_or(Error::UnsupportedProfile)?;blobs.push(Blob::capture(&auth,&format!(".restored-history-prior-{h}.json"),8*1024*1024)?.0);}
  }
  let opened=if let Some(b)=tx::read_file(&job,tx::OPENED,4096)?{if serde_json::from_slice::<serde_json::Value>(&b).map_err(|_|Error::UnsupportedProfile)?!=serde_json::json!({"version":1,"admission":digest(&admission)}){return Err(Error::SourceChanged);}blobs.push(Blob::capture(&job,tx::OPENED,4096)?.0);true}else{false};
  if !opened{verify_initial(roots,&m,&job)?;}
  let result=Self{history,manifest:m,admission,blobs,opened};result.current(roots)?;Ok(result)
 }
 pub(super)fn current(&self,roots:&OwnedRoots)->Result<()>{
  if read(&roots.config,ADMITTED)?.as_deref()!=Some(self.admission.as_slice()){return Err(Error::SourceChanged);}required_controls(roots,&self.manifest)?;for blob in &self.blobs{blob.current()?;}Ok(())
 }
 pub(super)fn mark_opened(&mut self,roots:&OwnedRoots)->Result<()>{
  self.current(roots)?;if !self.opened{let job=roots.config.dir(OsStr::new(self.manifest.job()?))?;verify_initial(roots,&self.manifest,&job)?;let b=serde_json::to_vec(&serde_json::json!({"version":1,"admission":digest(&self.admission)})).map_err(|_|Error::Unavailable)?;job.publish_new(tx::OPENED,&b)?;self.blobs.push(Blob::capture(&job,tx::OPENED,4096)?.0);self.opened=true;}self.current(roots)
 }
 pub(super)fn ack(&self,archive_id:&str,choices:&crate::pet_data::restored_projection::Choices,approved:Option<&str>)->Result<serde_json::Value>{
  if archive_id!=self.manifest.archive_id||*choices!=self.manifest.choices{return Err(Error::SourceChanged);}let fp=digest(&serde_json::to_vec(&("coucou-read-restored-admission-v1",digest(&self.admission),choices)).map_err(|_|Error::Unavailable)?);let job=&self.blobs[0].parent;let ready:tx::Ready=serde_json::from_slice(&tx::read_file(job,tx::READY,4096)?.ok_or(Error::SourceChanged)?).map_err(|_|Error::UnsupportedProfile)?;
  if let Some(approved)=approved{if approved!=fp&&approved!=ready.review{return Err(Error::SourceChanged);}Ok(serde_json::json!({"status":"restoredProfileAdmitted","pair":self.manifest.pair,"admissionHash":digest(&self.admission),"pendingFeatures":self.manifest.pending_features,"historicalAuthorityRenewed":false,"executionAuthorityGranted":false,"runtimeActivationAvailable":true}))}else{Ok(serde_json::json!({"status":"restoredActivationAlreadyAdmitted","fingerprint":fp,"pair":self.manifest.pair,"admissionHash":digest(&self.admission),"pendingFeatures":self.manifest.pending_features,"executionAuthorityGranted":false,"runtimeActivationAvailable":true}))}
 }
 pub(super)fn paths(&self,roots:&OwnedRoots)->Result<Paths>{Ok(Paths{config:self.manifest.payload(&roots.config)?.bound_path()?,data:self.manifest.payload(&roots.data)?.bound_path()?})}
 pub(super)fn pending(&self,name:&str)->bool{self.manifest.pending_features.iter().any(|n|n==name)}
}
fn verify_initial(roots:&OwnedRoots,m:&Manifest,job:&Dir)->Result<()>{verify_initial_payload(&roots.config,&roots.data,m,job)}
pub(in crate::pet_data)fn verify_initial_payload(config_root:&Dir,data_root:&Dir,m:&Manifest,job:&Dir)->Result<()>{
 let config=config_root.dir(OsStr::new(PAYLOAD))?;let data=data_root.dir(OsStr::new(PAYLOAD))?;
 let original_job=config_root.dir(OsStr::new(tx::JOB))?;let originals=original_job.dir(OsStr::new("originals"))?;let operational=m.payload(config_root)?;
 let history_path=operational.bound_path()?.join("pet-runtime-consent-v1");let metadata=if m.independent.is_some(){Ok(vec![])}else if m.epoch.is_some(){crate::pet_runtime::restoration::epoch_publication_delta(&history_path)}else{crate::pet_runtime::restoration::publication_delta(&history_path)}.map_err(|_|Error::UnsupportedProfile)?;
 let mut delta=metadata.into_iter().map(|(n,b)|Ok((Area::Config,m.source_relative(&PathBuf::from("pet-runtime-consent-v1").join(n))?,b))).collect::<Result<Vec<_>>>()?;
 for c in &m.changes{let root=m.payload(match c.area{Area::Config=>config_root,Area::Data=>data_root})?;let b=tx::read_file(&root,&c.path,256*1024)?.ok_or(Error::SourceChanged)?;if digest(&b)!=c.new{return Err(Error::SourceChanged);}delta.push((c.area,m.source_relative(Path::new(&c.path))?,b));let temp=Path::new(&c.path).parent().unwrap_or(Path::new("")).join(&c.temp);if let Some(b)=tx::read_file(&root,temp.to_str().ok_or(Error::UnsafePath)?,256*1024)?{if c.old.as_deref()!=Some(digest(&b).as_str()){return Err(Error::SourceChanged);}delta.push((c.area,m.source_relative(&temp)?,b));}}
 let original_path=originals.bound_path()?;if let Some(i)=&m.independent{let p=restore::independent::operational::verify_retained(config_root,i,&m.pair,&m.layout,&m.source)?;restore::independent::operational::verify_originals(config_root,&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?,i,&m.pair,&m.layout,&m.source)?;staging::verify_retained_independent(&original_path.join("config"),&original_path.join("data"),&config,&data,&delta,&m.original_content,&p)?;}else{staging::verify_retained_delta(&original_path.join("config"),&original_path.join("data"),&config,&data,&delta,&m.original_content)?;}
 if let Some(e)=&m.epoch{restore::epoch::verify_retained_custody(config_root,&e.scope.custody,&m.pair,&m.layout,&m.source)?;let h=restore::configuration::references_from_roots(&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?)?;if h.fingerprint()?!=m.history||h.review()?!=e.scope.references||tx::read_file(job,tx::HISTORY,crate::pet_data::restored_history::MAX_INDEX)?.as_deref()!=Some(h.encode()?.as_slice()){return Err(Error::SourceChanged);}}
 Ok(())
}

pub(crate)fn publish(certificate:&crate::pet_data::restored_transaction::Approved<'_>)->Result<()>{
 certificate.current()?;let layout=certificate.layout();let config=Dir::root(layout.config())?;let data=Dir::root(layout.data())?;let m=certificate.manifest();let a=read(&config,COMPLETE)?.ok_or(Error::SourceChanged)?;let b=read(&data,COMPLETE)?.ok_or(Error::SourceChanged)?;
 let record=encode(&Admission2{version:m.admission_version()?,pair:m.pair.clone(),layout:m.layout.clone(),config:digest(&a),data:digest(&b),manifest:digest(certificate.manifest_bytes()),history:m.history.clone(),authority:m.authority.clone(),epoch:m.epoch.clone(),independent:m.independent.clone()})?;
 certificate.current()?;if let Some(old)=read(&config,ADMITTED)?{if old!=record{return Err(Error::SourceChanged);}}else{config.publish_new(ADMITTED,&record)?;}
 certificate.current()?;if read(&config,ADMITTED)?.as_deref()!=Some(record.as_slice()){return Err(Error::SourceChanged);}let job=config.dir(OsStr::new(m.job()?))?;let seal=serde_json::to_vec(&serde_json::json!({"version":1,"admission":digest(&record),"manifest":digest(certificate.manifest_bytes())})).map_err(|_|Error::Unavailable)?;if let Some(old)=tx::read_file(&job,tx::SEALED,4096)?{if old!=seal{return Err(Error::SourceChanged);}}else{job.publish_new(tx::SEALED,&seal)?;}certificate.owner_current()?;Ok(())
}
