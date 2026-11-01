//! Typed publication of the already reviewed independent seed. No RNG/ensure,
//! new grants or arbitrary directory exclusions. Seed remains immutable data.
use super::*;
use std::{collections::BTreeMap,path::{Path,PathBuf}};
pub(in crate::pet_data)const AUTHORITY:&str="pet-runtime-consent-v1";
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(in crate::pet_data)struct Custody{pub version:u32,pub original_authority_present:bool,pub manifest:String,pub ready:String,pub generation:String,pub history:String,pub seed:String,pub authority:String,pub key_id:String,#[serde(default,skip_serializing_if="Option::is_none")]pub retention:Option<super::super::retention::Binding>}
impl Custody{pub(in crate::pet_data)fn validate(&self,history:&str,authority:&str)->Result<()>{match(self.version,&self.retention){(1,None)=>{},(2,Some(b))=>b.validate()?,_=>return Err(Error::UnsupportedProfile)}if self.original_authority_present||self.history!=history||self.authority!=authority||!valid_hex(&self.key_id,32)||[&self.manifest,&self.ready,&self.generation,&self.history,&self.seed,&self.authority].iter().any(|h|!valid_hex(h,64)){return Err(Error::UnsupportedProfile);}Ok(())}}
pub(in crate::pet_data)struct PreparedAuthority{pub custody:Custody,pub history:Vec<u8>,config:Dir,seed:Dir,files:BTreeMap<String,Vec<u8>>,stage:String}
fn files(seed:&Dir)->Result<BTreeMap<String,Vec<u8>>>{let _=seed_hash(seed)?;let mut out=BTreeMap::new();for n in seed.node()?.names(20)?{let n=n.to_str().ok_or(Error::UnsafePath)?;out.insert(n.to_owned(),get(seed,n,MAX)?);}Ok(out)}
fn retained(config:&Dir,pair:&str,layout:&str,source:&str)->Result<(PreparedAuthority,Intent)>{
 job_fingerprint(config)?;let job=config.dir(OsStr::new(JOB))?;let mb=get(&job,MANIFEST,MAX)?;let m:Intent=parse(&mb)?;if m.pair!=pair||m.layout!=layout||m.source!=source{return Err(Error::SourceChanged);}
 let rb=get(&job,READY,64*1024)?;let r:Ready=parse(&rb)?;let gb=get(&job,GENERATION,4096)?;if gb!=generation(&mb)?{return Err(Error::SourceChanged);}let hb=get(&job,HISTORY,MAX)?;let h=History::decode_first(&hb,m.references["indexHash"].as_str().ok_or(Error::UnsupportedProfile)?)?;if h.review()?!=m.references{return Err(Error::SourceChanged);}
 let seed=job.dir(OsStr::new(SEED))?;let view=crate::pet_runtime::restoration::verify_retained_independent(&seed.bound_path()?,&m.goals).map_err(core)?;if view!=r.authority{return Err(Error::SourceChanged);}
 let c=Custody{version:if m.retention.is_some(){2}else{1},original_authority_present:false,manifest:digest(&mb),ready:digest(&rb),generation:digest(&gb),history:digest(&hb),seed:seed_hash(&seed)?,authority:view["historyHash"].as_str().ok_or(Error::UnsupportedProfile)?.into(),key_id:view["keyId"].as_str().ok_or(Error::UnsupportedProfile)?.into(),retention:m.retention.clone()};c.validate(&c.history,&c.authority)?;
 let stage=format!(".restore-authority-copy-{}",&digest(&serde_json::to_vec(&(pair,&c)).map_err(|_|Error::Unavailable)?)[..32]);let p=PreparedAuthority{custody:c,history:hb,config:config.clone(),files:files(&seed)?,seed,stage};Ok((p,m))
}
pub(in crate::pet_data)fn capture(data:&Review,delta:&[(Area,PathBuf,Vec<u8>)])->Result<PreparedAuthority>{
 if data.profile_version()!=2||data.source.archive_version()!=1{return Err(Error::UnsupportedProfile);}data.restored_data_current()?;let config=Dir::root(data.layout.config())?;let pair=data.pair.as_deref().ok_or(Error::UnsupportedProfile)?;let(p,m)=retained(&config,pair,&layout_key(&data.layout),&data.source_fence)?;
 let original=Dir::root(data.source.source_root(Area::Config))?;for name in [AUTHORITY,p.stage.as_str()]{if original.node()?.child(OsStr::new(name))?.is_some(){return Err(Error::UnsupportedProfile);}}
 let goals=crate::pet_runtime::restoration::IndependentScope::capture(&original.bound_path()?.join("pet-runtime-v1"),&data.source_fence).map_err(core)?;if goals.review().map_err(core)?!=m.goals{return Err(Error::SourceChanged);}
 let library=read_file(&Dir::root(data.source.source_root(Area::Data))?,"assistant/conversations.json",crate::assistant_memory::MAX_BYTES as usize)?;if History::capture(library.as_deref())?.encode()?!=p.history{return Err(Error::SourceChanged);}
 let settings=read_file(&original,"settings.json",256*1024)?;let pets=read_file(&original,"pets.json",256*1024)?;if super::super::configuration::source_report(settings.as_deref(),pets.as_deref())?!=m.configuration{return Err(Error::SourceChanged);}
 staging::verify_publication_independent(&data.source,&config.dir(OsStr::new(PAYLOAD))?,&Dir::root(data.layout.data())?.dir(OsStr::new(PAYLOAD))?,delta,&p)?;data.current()?;Ok(p)
}
impl PreparedAuthority{
 fn current(&self)->Result<()>{if super::super::retention::binding(&self.config)?!=self.custody.retention||seed_hash(&self.seed)?!=self.custody.seed||files(&self.seed)?!=self.files{return Err(Error::SourceChanged);}Ok(())}
 /// Returns ONLY exact records currently allowed by this native, verified
 /// package. A model/caller cannot construct PreparedAuthority from JSON.
 pub(in crate::pet_data)fn records(&self,config:&Dir)->Result<Vec<(PathBuf,Option<Vec<u8>>)>>{
  self.current()?;let final_exists=config.node()?.child(OsStr::new(AUTHORITY))?.is_some();let staged=config.node()?.child(OsStr::new(&self.stage))?.is_some();if final_exists&&staged{return Err(Error::SourceChanged);}if !final_exists&&!staged{return Ok(vec![]);}
  let name=if final_exists{AUTHORITY}else{&self.stage};let root=config.dir(OsStr::new(name))?;let names=root.node()?.names(20)?;if final_exists&&names.len()!=self.files.len(){return Err(Error::SourceChanged);}let mut records=vec![(PathBuf::from(name),None)];
  for n in names{let n=n.to_str().ok_or(Error::UnsafePath)?;let expected=self.files.get(n).ok_or(Error::SourceChanged)?;let b=get(&root,n,expected.len())?;if final_exists&&b!=*expected||!expected.starts_with(&b){return Err(Error::SourceChanged);}records.push((PathBuf::from(name).join(n),Some(b)));}
  root.check()?;self.current()?;Ok(records)
 }
 pub(in crate::pet_data)fn publish(&self,config:&Dir,permit:&crate::pet_data::restored_transaction::IndependentPublication<'_>,mut step:impl FnMut(usize)->Result<()>)->Result<()>{
  if permit.custody()!=Some(&self.custody){return Err(Error::SourceChanged);}permit.current()?;self.current()?;self.records(config)?;
  if config.node()?.child(OsStr::new(AUTHORITY))?.is_some(){config.dir(OsStr::new(AUTHORITY))?.sync()?;return config.sync();}
  let stage=if config.node()?.child(OsStr::new(&self.stage))?.is_some(){config.dir(OsStr::new(&self.stage))?}else{config.mkdir(OsStr::new(&self.stage))?};permit.current()?;
  for(index,(n,b))in self.files.iter().enumerate(){permit.current()?;self.current()?;stage.complete_prefix(n,b)?;step(index)?;permit.current()?;}
  stage.sync()?;permit.current()?;self.current()?;config.publish_dir(OsStr::new(&self.stage),OsStr::new(AUTHORITY),&stage)?;permit.current()?;Ok(())
 }
}
/// Local immutable custody check, also used by portable readers. Never reads
/// the mutable operational goal catalog or reclassifies new work as old.
pub(in crate::pet_data)fn verify_retained(config:&Dir,expected:&Custody,pair:&str,layout:&str,source:&str)->Result<PreparedAuthority>{let(p,_)=retained(config,pair,layout,source)?;if p.custody!=*expected{return Err(Error::SourceChanged);}Ok(p)}
pub(in crate::pet_data)fn verify_originals(config:&Dir,original_config:&Dir,original_data:&Dir,expected:&Custody,pair:&str,layout:&str,source:&str)->Result<()>{
 let(p,m)=retained(config,pair,layout,source)?;if p.custody!=*expected||original_config.node()?.child(OsStr::new(AUTHORITY))?.is_some(){return Err(Error::SourceChanged);}let goals=crate::pet_runtime::restoration::IndependentScope::capture(&original_config.bound_path()?.join("pet-runtime-v1"),source).map_err(core)?;if goals.review().map_err(core)?!=m.goals{return Err(Error::SourceChanged);}let b=read_file(original_data,"assistant/conversations.json",crate::assistant_memory::MAX_BYTES as usize)?;if History::capture(b.as_deref())?.encode()?!=p.history{return Err(Error::SourceChanged);}Ok(())
}
