//! Native retention of an inactive preparation. Retained bytes are untrusted
//! evidence, never a recovered grant or an opportunity to regenerate a key.
use super::*;
use crate::pet_data::restored_transaction::read_file;
use serde_json::{Value,json};
use std::collections::BTreeSet;
#[path="publication_restore_retention_inventory.rs"]mod raw;
use raw::{Opaque,Budget};

pub(in crate::pet_data)const ROOT:&str=".restore-authority-retained";
const PROTOCOL:&str="protocol.json";
const MANIFEST:&str="manifest.json";
const ORIGINAL:&str="original";
const READY:&str="ready.json";
const MAX_RECORDS:usize=8;
const MAX_META:usize=16*1024;
fn bytes<T:Serialize>(v:&T)->Result<Vec<u8>>{serde_json::to_vec(v).map_err(|_|Error::Unavailable)}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Scope{pair:String,layout:String,source:String}
impl Scope{
 fn capture(config:&Dir)->Result<Self>{
  let h:Header=parse(&read(config,HEADER)?.ok_or(Error::UnsupportedProfile)?)?;
  let i:super::Intent=parse(&read(config,INTENT)?.ok_or(Error::UnsupportedProfile)?)?;
  if h.version!=2||h.area!=Area::Config||i.version!=1||h.pair!=i.pair||h.layout!=i.layout||!valid_hex(&h.pair,32)||!valid_hex(&h.layout,64)||!valid_hex(&i.archive_fence,64){return Err(Error::UnsupportedProfile);}
  Ok(Self{pair:h.pair,layout:h.layout,source:i.archive_fence})
 }
 fn protocol(&self)->Result<Vec<u8>>{bytes(&json!({"version":1,"kind":"independentPreparationRetentions","scope":self}))}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Manifest{version:u32,purpose:String,scope:Scope,previous:Option<String>,original:Opaque}
impl Manifest{
 fn valid(&self,scope:&Scope)->bool{self.version==1&&self.purpose=="retainIndependentPreparation"&&&self.scope==scope&&self.original.valid()&&self.previous.as_ref().is_none_or(|p|valid_hex(p,64))}
 fn id(&self)->Result<String>{Ok(format!("r-{}",&digest(&bytes(self)?)[..32]))}
 fn ready(&self)->Result<Vec<u8>>{bytes(&json!({"version":1,"kind":"independentPreparationRetained","manifest":digest(&bytes(self)?)}))}
}
struct Record{dir:Dir,name:String,manifest:Option<Manifest>,manifest_bytes:Option<Vec<u8>>,original:Option<Opaque>,ready:Option<Vec<u8>>,complete:bool}
struct Scan{scope:Scope,root:Option<Dir>,protocol:Option<Vec<u8>>,records:Vec<Record>,head:Option<String>,pending:Option<usize>,budget:Budget}
impl Scan{
 fn fence(&self,physical:bool,except:Option<&str>)->Result<String>{
  let mut rows=vec![];
  for r in &self.records{
   if except==Some(r.name.as_str()){continue;}
   rows.push((r.name.clone(),r.manifest_bytes.as_deref().map(digest),r.ready.as_deref().map(digest),r.original.as_ref().map(|o|o.content.clone()),if physical{Some((raw::location(&r.dir)?,r.original.as_ref().map(|o|(o.location.clone(),o.revision.clone()))))}else{None}));
  }
  bytes(&(self.protocol.as_deref().map(digest),rows,if physical{self.root.as_ref().map(raw::location).transpose()?}else{None})).map(|b|digest(&b))
 }
}
fn scan(config:&Dir,recovery:bool)->Result<Scan>{
 let scope=Scope::capture(config)?;let mut s=Scan{scope,root:None,protocol:None,records:vec![],head:None,pending:None,budget:Budget::default()};
 if config.node()?.child(OsStr::new(ROOT))?.is_none(){return Ok(s);}
 let root=config.dir(OsStr::new(ROOT))?;let names=root.node()?.names(MAX_RECORDS+1)?;
 let expected=s.scope.protocol()?;s.protocol=read_file(&root,PROTOCOL,MAX_META)?;
 if s.protocol.as_deref()!=Some(expected.as_slice()){
  if !recovery||s.protocol.as_deref().is_some_and(|p|!expected.starts_with(p))||names.iter().any(|n|n!=OsStr::new(PROTOCOL)){return Err(Error::UnsupportedProfile);}
  s.root=Some(root);return Ok(s);
 }
 for name in names{
  if name==OsStr::new(PROTOCOL){continue;}let name=name.to_str().ok_or(Error::UnsafePath)?;
  if !name.strip_prefix("r-").is_some_and(|v|valid_hex(v,32)){return Err(Error::UnsupportedProfile);}
  let dir=root.dir(OsStr::new(name))?;
  for n in dir.node()?.names(3)?{if ![MANIFEST,ORIGINAL,READY].iter().any(|v|n==OsStr::new(v)){return Err(Error::UnsupportedProfile);}}
  let mb=read_file(&dir,MANIFEST,MAX_META)?;
  let manifest=mb.as_deref().and_then(|b|serde_json::from_slice::<Manifest>(b).ok()).filter(|m|m.valid(&s.scope)&&bytes(m).ok().as_deref()==mb.as_deref());
  let original=if dir.node()?.child(OsStr::new(ORIGINAL))?.is_some(){Some(raw::inspect(&dir.dir(OsStr::new(ORIGINAL))?,&mut s.budget)?)}else{None};
  let ready=read_file(&dir,READY,MAX_META)?;
  if let Some(m)=&manifest{
   if m.id()?!=name{return Err(Error::SourceChanged);}
   if original.as_ref().is_some_and(|o|o.content!=m.original.content){return Err(Error::SourceChanged);}
   if let Some(r)=&ready{if original.is_none()||!m.ready()?.starts_with(r){return Err(Error::SourceChanged);}}
  }else if !recovery||original.is_some()||ready.is_some(){return Err(Error::UnsupportedProfile);}
  let complete=match(&manifest,&ready){(Some(m),Some(r))=>m.ready()?==*r,_=>false};
  if !complete{
   if !recovery||s.pending.is_some(){return Err(Error::UnsupportedProfile);}
   // Only the same physical original can finish a pending rename. A completed
   // retention later travels as data, so its portable content is checked above.
   if let(Some(m),Some(o))=(&manifest,&original){if &m.original!=o{return Err(Error::SourceChanged);}}
   s.pending=Some(s.records.len());
  }
  s.records.push(Record{dir,name:name.into(),manifest,manifest_bytes:mb,original,ready,complete});
 }
 if s.records.is_empty()&&!recovery{return Err(Error::UnsupportedProfile);}
 let count=s.records.iter().filter(|r|r.complete).count();let mut used=BTreeSet::new();
 for _ in 0..count{
  let next=s.records.iter().enumerate().filter(|(i,r)|r.complete&&!used.contains(i)&&r.manifest.as_ref().is_some_and(|m|m.previous==s.head)).collect::<Vec<_>>();
  if next.len()!=1{return Err(Error::SourceChanged);}let(i,r)=next[0];used.insert(i);s.head=Some(digest(r.ready.as_deref().ok_or(Error::SourceChanged)?));
 }
 if let Some(i)=s.pending{if s.records[i].manifest.as_ref().is_some_and(|m|m.previous!=s.head){return Err(Error::SourceChanged);}}
 s.root=Some(root);Ok(s)
}
pub(in crate::pet_data)fn fingerprint(config:&Dir,recovery:bool)->Result<String>{scan(config,recovery)?.fence(recovery,None)}
pub(in crate::pet_data)fn raw_fingerprint(job:&Dir)->Result<String>{bytes(&raw::inspect(job,&mut Budget::default())?).map(|b|digest(&b))}

/// Portable custody metadata only. It cannot construct a claim or a grant.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(in crate::pet_data)struct Binding{pub version:u32,pub head:String,pub fingerprint:String,pub records:u32}
impl Binding{pub(in crate::pet_data)fn validate(&self)->Result<()>{if self.version!=1||!valid_hex(&self.head,64)||!valid_hex(&self.fingerprint,64)||!(1..=MAX_RECORDS as u32).contains(&self.records){return Err(Error::UnsupportedProfile);}Ok(())}}
pub(in crate::pet_data)fn binding(config:&Dir)->Result<Option<Binding>>{
 if config.node()?.child(OsStr::new(ROOT))?.is_none(){return Ok(None);}
 let s=scan(config,false)?;let b=Binding{version:1,head:s.head.clone().ok_or(Error::UnsupportedProfile)?,fingerprint:s.fence(false,None)?,records:s.records.len()as u32};b.validate()?;Ok(Some(b))
}
pub(in crate::pet_data)fn completed_heads(config:&Dir,recovery:bool)->Result<BTreeSet<String>>{let s=scan(config,recovery)?;Ok(s.records.iter().filter(|r|r.complete).filter_map(|r|r.ready.as_deref().map(digest)).collect())}

pub(in crate::pet_data)struct RetentionReview{data:Review,config:Dir,data_root:Dir,manifest:Manifest,name:String,prior_fence:String,controls:Vec<Vec<(String,String)>>}
impl RetentionReview{
 pub(in crate::pet_data)fn capture(data:Review)->Result<Self>{
  if data.mode!=Mode::RetainIndependent||data.profile_version()!=2||data.source.archive_version()!=1{return Err(Error::UnsupportedProfile);}
  data.authority_verify_delta(&[])?;let config=Dir::root(data.layout.config())?;let data_root=Dir::root(data.layout.data())?;
  let mut s=scan(&config,true)?;let job=if config.node()?.child(OsStr::new(independent::JOB))?.is_some(){Some(raw::inspect(&config.dir(OsStr::new(independent::JOB))?,&mut s.budget)?)}else{None};
  let manifest=if let Some(i)=s.pending{
   if let Some(m)=&s.records[i].manifest{m.clone()}else{Manifest{version:1,purpose:"retainIndependentPreparation".into(),scope:s.scope.clone(),previous:s.head.clone(),original:job.clone().ok_or(Error::SourceChanged)?}}
  }else if let Some(original)=job{
   if s.records.len()>=MAX_RECORDS{return Err(Error::Limit);}Manifest{version:1,purpose:"retainIndependentPreparation".into(),scope:s.scope.clone(),previous:s.head.clone(),original}
  }else{
   let r=s.records.iter().find(|r|r.ready.as_deref().is_some_and(|b|Some(digest(b))==s.head)).ok_or(Error::UnsupportedProfile)?;r.manifest.clone().ok_or(Error::UnsupportedProfile)?
  };
  let name=manifest.id()?;let prior_fence=s.fence(true,Some(&name))?;let controls=Self::controls(&config,&data_root)?;
  let r=Self{data,config,data_root,manifest,name,prior_fence,controls};r.current()?;Ok(r)
 }
 fn controls(config:&Dir,data:&Dir)->Result<Vec<Vec<(String,String)>>>{Ok(vec![root_controls_for(config,Area::Config,true)?.into_iter().filter(|(n,_)|n!=ROOT&&n!=independent::JOB).collect(),root_controls_for(data,Area::Data,true)?])}
 fn eligible(&self)->Result<()>{
  self.config.check()?;self.data_root.check()?;self.data.authority_verify_delta(&[])?;
  if Scope::capture(&self.config)?!=self.manifest.scope||self.manifest.scope.source!=self.data.source_fence||self.manifest.scope.pair!=self.data.pair.as_deref().ok_or(Error::UnsupportedProfile)?||self.manifest.scope.layout!=layout_key(&self.data.layout){return Err(Error::SourceChanged);}
  for n in [ADMITTED,crate::pet_data::restored_transaction::JOB,crate::pet_data::restored_transaction::EPOCH_JOB]{if self.config.node()?.child(OsStr::new(n))?.is_some(){return Err(Error::UnsupportedProfile);}}
  for root in [Dir::root(self.data.source.source_root(Area::Config))?,self.config.dir(OsStr::new(PAYLOAD))?]{if root.node()?.child(OsStr::new(independent::operational::AUTHORITY))?.is_some(){return Err(Error::UnsupportedProfile);}}
  if Self::controls(&self.config,&self.data_root)?!=self.controls{return Err(Error::SourceChanged);}Ok(())
 }
 fn current(&self)->Result<Scan>{
  self.eligible()?;let mut s=scan(&self.config,true)?;
  // Protocol creation changes the container's phase, not prior retained data.
  let prior=if self.manifest.previous.is_none()&&s.records.iter().all(|r|r.name==self.name){None}else{Some(s.fence(true,Some(&self.name))?)};
  if let Some(prior)=prior{if prior!=self.prior_fence{return Err(Error::SourceChanged);}}
  if s.pending.is_some_and(|i|s.records[i].name!=self.name){return Err(Error::SourceChanged);}
  let source=if self.config.node()?.child(OsStr::new(independent::JOB))?.is_some(){Some(raw::inspect(&self.config.dir(OsStr::new(independent::JOB))?,&mut s.budget)?)}else{None};
  let record=s.records.iter().find(|r|r.name==self.name);
  if let Some(r)=record{
   let expected=bytes(&self.manifest)?;if r.manifest_bytes.as_deref().is_some_and(|b|!expected.starts_with(b)){return Err(Error::SourceChanged);}
   if r.original.is_some()&&source.is_some(){return Err(Error::SourceChanged);}
   if let Some(o)=&r.original{if if r.complete{o.content!=self.manifest.original.content}else{*o!=self.manifest.original}{return Err(Error::SourceChanged);}}
   else if source.as_ref()!=Some(&self.manifest.original){return Err(Error::SourceChanged);}
  }else if source.as_ref()!=Some(&self.manifest.original){return Err(Error::SourceChanged);}
  Ok(s)
 }
 fn review_hash(&self)->Result<String>{self.current()?;bytes(&("coucou-independent-preparation-retention-review-v1",self.data.fingerprint()?,&self.manifest)).map(|b|digest(&b))}
 pub(in crate::pet_data)fn preview(&self)->Result<Value>{let hash=self.review_hash()?;let s=self.current()?;let r=s.records.iter().find(|r|r.name==self.name);Ok(json!({"status":"independentPreparationRetentionReview","schemaVersion":1,"fingerprint":hash,"pair":self.manifest.scope.pair,"record":self.name,"original":self.manifest.original.content,"previous":self.manifest.previous,"phase":if r.is_some_and(|r|r.complete){"retained"}else if r.is_some_and(|r|r.original.is_some()){"movedPendingReceipt"}else if r.is_some(){"intentPendingTransfer"}else{"notStarted"},"retainedBytesTrustedAsAuthority":false,"keyGenerationAvailable":false,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"futurePreparationRequiresSeparateReview":true,"futurePreparationBlocked":!r.is_some_and(|r|r.complete)}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<Value,Failure>{self.apply_using(approved,|_|Ok(()))}
 fn apply_using(self,approved:&str,mut step:impl FnMut(&str)->Result<()>)->std::result::Result<Value,Failure>{
  let pair=Some(self.manifest.scope.pair.clone());let result=(||{
   if approved!=self.review_hash()?{return Err(Error::SourceChanged);}let s=self.current()?;
   let container=match s.root{Some(root)=>root,None=>{let dir=self.config.mkdir(OsStr::new(ROOT))?;step("containerCreated")?;dir}};
   self.current()?;container.complete_prefix(PROTOCOL,&self.manifest.scope.protocol()?)?;step("protocol")?;self.current()?;
   let record=if container.node()?.child(OsStr::new(&self.name))?.is_some(){container.dir(OsStr::new(&self.name))?}else{let dir=container.mkdir(OsStr::new(&self.name))?;step("recordCreated")?;dir};
   self.current()?;record.complete_prefix(MANIFEST,&bytes(&self.manifest)?)?;record.sync()?;step("intent")?;self.current()?;
   if record.node()?.child(OsStr::new(ORIGINAL))?.is_none(){
    let original=self.config.dir(OsStr::new(independent::JOB))?;self.current()?;
    self.config.transfer_dir(OsStr::new(independent::JOB),&record,OsStr::new(ORIGINAL),&original)?;step("moved")?;
   }
   self.current()?;record.complete_prefix(READY,&self.manifest.ready()?)?;record.sync()?;container.sync()?;self.config.sync()?;step("ready")?;
   self.current()?;Ok(json!({"status":"independentPreparationRetained","schemaVersion":1,"pair":self.manifest.scope.pair,"record":self.name,"receiptHash":digest(&self.manifest.ready()?),"original":self.manifest.original.content,"retainedBytesTrustedAsAuthority":false,"keyGenerationAvailable":false,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"futurePreparationRequiresSeparateReview":true,"futurePreparationBlocked":false}))
  })();result.map_err(|error|Failure{error,pair})
 }
}

#[cfg(test)]impl RetentionReview{
 pub(in crate::pet_data)fn fixture_interrupt(self,approved:&str,cut:&str)->std::result::Result<Value,Failure>{self.apply_using(approved,|s|if s==cut{Err(Error::Unavailable)}else{Ok(())})}
 pub(in crate::pet_data)fn apply_using_for_test(self,approved:&str,mut after_intent:impl FnMut())->std::result::Result<Value,Failure>{self.apply_using(approved,|s|{if s=="intent"{after_intent();}Ok(())})}
}
