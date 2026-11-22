//! Current archive data publication. A restored root-pair uses
//! inactive Header3 until multi-epoch authority/initialization is reviewed.
use super::*;
use crate::pet_data::{inventory,staging::{CurrentArchive,Source},private_fs};
pub(in crate::pet_data)const INTENT:&str=".restore.json";
pub(in crate::pet_data)const ORIGIN:&str=".restore-origin.json";
pub(in crate::pet_data)const RESTORED:&str=".restored.json";
pub(in crate::pet_data)const CONFIG_REVIEW:&str=".configuration-review.json";
pub(in crate::pet_data)const CONFIG_INDEPENDENT_REVIEW:&str=".independent-configuration-review.json";
pub(in crate::pet_data)const CONFIG_RETAINED_PREFIX:&str=".retained-configuration-review-";
const MAX_ORIGIN:usize=256*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Intent{version:u32,pair:String,layout:String,archive_id:String,archive_fence:String,origin:String,content:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Restored{version:u32,pair:String,layout:String,source:String,config:String,data:String,authority_state:String}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(in crate::pet_data)enum Mode{Fresh,Resume,Activation,RetainIndependent}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Step{ConfigCreated,DataCreated,BeforeCopy(usize),Copied,ConfigComplete,DataComplete,Restored}
struct Root{dir:Dir,header:Vec<u8>,intent:Vec<u8>,origin:Vec<u8>}
pub(in crate::pet_data)struct Review{pub(in crate::pet_data)claim:Claim,pub(in crate::pet_data)layout:Layout,pub(in crate::pet_data)source:CurrentArchive,parents:[Dir;2],mode:Mode,pub(in crate::pet_data)pair:Option<String>,target_fence:String,pub(in crate::pet_data)source_fence:String,origin:Vec<u8>,content:String}
fn read_origin(root:&Dir)->Result<Vec<u8>>{
 let parent=root.node()?;let node=parent.child(OsStr::new(ORIGIN))?.ok_or(Error::UnsupportedProfile)?;
 if node.directory||node.len>MAX_ORIGIN as u64||!staging_fs::private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
 let mut bytes=vec![];(&node.file).take(MAX_ORIGIN as u64+1).read_to_end(&mut bytes).map_err(|_|Error::Unavailable)?;
 node.current()?;parent.same_child(OsStr::new(ORIGIN),&node)?;if bytes.len()>MAX_ORIGIN||bytes.len()as u64!=node.len{return Err(Error::SourceChanged);}let origin:BackupOrigin=parse(&bytes)?;if !origin.bounded(){return Err(Error::UnsupportedProfile);}Ok(bytes)
}
pub(in crate::pet_data)fn root_controls(root:&Dir,area:Area)->Result<Vec<(String,String)>>{
 root_controls_for(root,area,false)
}
fn root_controls_for(root:&Dir,area:Area,recovery:bool)->Result<Vec<(String,String)>>{
 let mut result=vec![];let mut staged=0;
 for name in root.node()?.names(22+MAX_STAGED_RECORDS)?{
  let name=name.to_str().ok_or(Error::UnsupportedProfile)?;
  if area==Area::Config&&super::super::settings_io::control(name){super::super::settings_io::present(root)?;let bytes=read(root,name)?.ok_or(Error::SourceChanged)?;result.push((name.into(),digest(&bytes)));continue;}
  if area==Area::Config&&name==crate::pet_data::restored_transaction::EPOCH_JOB{let h:Header=parse(&read(root,HEADER)?.ok_or(Error::UnsupportedProfile)?)?;if h.version!=3{return Err(Error::UnsupportedProfile);}result.push((name.into(),crate::pet_data::restored_transaction::job_fingerprint_named(root,name)?));continue;}
  if area==Area::Config&&name==crate::pet_data::restored_transaction::JOB{result.push((name.into(),crate::pet_data::restored_transaction::job_fingerprint(root)?));continue;}
  if area==Area::Config&&name==independent::JOB{result.push((name.into(),if recovery{retention::raw_fingerprint(&root.dir(OsStr::new(name))?)?}else{independent::job_fingerprint(root)?}));continue;}
  if area==Area::Config&&name==retention::ROOT{result.push((name.into(),retention::fingerprint(root,recovery)?));continue;}
  if area==Area::Config{if let Some(head)=configuration::retained_receipt_head(name){if !retention::completed_heads(root,recovery)?.contains(head){return Err(Error::UnsupportedProfile);}let b=read(root,name)?.ok_or(Error::SourceChanged)?;result.push((name.into(),digest(&b)));continue;}}
  if name==PAYLOAD{root.dir(OsStr::new(PAYLOAD))?.check()?;continue;}
  if !matches!(name,HEADER|COMPLETE|INTENT|ORIGIN)&&!(area==Area::Config&&matches!(name,RESTORED|CONFIG_REVIEW|CONFIG_INDEPENDENT_REVIEW|ADMITTED)){
   if !name.strip_prefix(".publish-").is_some_and(|v|valid_hex(v,32)){return Err(Error::UnsupportedProfile);}staged+=1;if staged>MAX_STAGED_RECORDS{return Err(Error::Limit);}
  }
  let bytes=if name==ORIGIN{read_origin(root)?}else{read(root,name)?.ok_or(Error::SourceChanged)?};result.push((name.into(),digest(&bytes)));
 }Ok(result)
}
fn parents(layout:&Layout)->Result<[Dir;2]>{
 let config=Dir::parent(&layout.legacy_config)?;let data=Dir::parent(&layout.legacy_data)?;
 for dir in [&config,&data]{for name in dir.node()?.names(4)?{
  // No restore into a legacy/operational application directory. Even unknown
  // metadata/abandoned staging is retained and refused, never cleaned/adopted.
  if name!=".pets-profile-v1.lock"&&name!=layout.config().file_name().ok_or(Error::UnsafePath)?&&name!=layout.data().file_name().ok_or(Error::UnsafePath)?{return Err(Error::TargetExists);}
 }}Ok([config,data])
}
impl Review{
 pub(in crate::pet_data)fn capture(claim:Claim,layout:Layout,workspace:&Path,id:&str,mode:Mode)->Result<Self>{
  claim.for_layout(&layout)?;let parents=parents(&layout)?;let source=CurrentArchive::open_restore(workspace,id)?;
  source.disjoint(&layout.legacy_config)?;source.disjoint(&layout.legacy_data)?;
  let origin=source.origin_bytes()?;if origin.len()>MAX_ORIGIN{return Err(Error::Limit);}let source_fence=source.fence()?;let content=staging::content_fingerprint(&source)?;
  let mut review=Self{claim,layout,source,parents,mode,pair:None,target_fence:String::new(),source_fence,origin,content};
  review.operational_depth()?;let(roots,pair)=review.existing()?;if mode!=Mode::Activation&&roots[0].as_ref().is_some_and(|root|read(&root.dir,ADMITTED).ok().flatten().is_some()){return Err(Error::UnsupportedProfile);}if mode==Mode::Fresh&&roots.iter().any(Option::is_some){return Err(Error::TargetExists);}if mode==Mode::Resume&&pair.is_none(){return Err(Error::UnsupportedProfile);}review.pair=pair;review.target_fence=review.target_snapshot(&roots)?;review.current()?;Ok(review)
 }
 pub(in crate::pet_data)fn operational_depth(&self)->Result<u8>{let depth=self.source.source_depth()?.checked_add(1).ok_or(Error::Limit)?;if depth>crate::pet_data::restored_transaction::MAX_PAYLOAD_DEPTH{return Err(Error::Limit);}Ok(depth)}
 pub(in crate::pet_data)fn profile_version(&self)->u32{if self.source.archive_version()==1{2}else{3}}
 fn intent(&self,pair:&str)->Intent{Intent{version:self.source.archive_version(),pair:pair.into(),layout:layout_key(&self.layout),archive_id:self.source.id().into(),archive_fence:self.source_fence.clone(),origin:digest(&self.origin),content:self.content.clone()}}
 fn existing(&self)->Result<([Option<Root>;2],Option<String>)>{
  let mut roots:[Option<Root>;2]=[None,None];let mut pair:Option<String>=None;
  for(index,(path,area))in[(self.layout.config(),Area::Config),(self.layout.data(),Area::Data)].into_iter().enumerate(){
   if missing(path)?{continue;}let dir=Dir::root(path)?;root_controls_for(&dir,area,self.mode==Mode::RetainIndependent)?;
   let header=read(&dir,HEADER)?.ok_or(Error::UnsupportedProfile)?;let h:Header=parse(&header)?;
   if h.version!=self.profile_version()||h.area!=area||h.layout!=layout_key(&self.layout)||!valid_hex(&h.pair,32)||pair.as_ref().is_some_and(|p|p!=&h.pair){return Err(Error::UnsupportedProfile);}
   let intent=read(&dir,INTENT)?.ok_or(Error::UnsupportedProfile)?;if parse::<Intent>(&intent)?!=self.intent(&h.pair){return Err(Error::SourceChanged);}
   let origin=read_origin(&dir)?;if origin!=self.origin{return Err(Error::SourceChanged);}pair=Some(h.pair);roots[index]=Some(Root{dir,header,intent,origin});
  }Ok((roots,pair))
 }
 fn target_snapshot(&self,roots:&[Option<Root>;2])->Result<String>{
  let mut controls=vec![];for(index,root)in roots.iter().enumerate(){controls.push(match root{Some(root)=>Some(root_controls_for(&root.dir,if index==0{Area::Config}else{Area::Data},self.mode==Mode::RetainIndependent)?),None=>None});}
  let snapshot=inventory::inspect_partial_current(&self.layout.config().join(PAYLOAD),&self.layout.data().join(PAYLOAD))?;
  Ok(digest(&serde_json::to_vec(&(controls,snapshot.fingerprint()?)).map_err(|_|Error::Unavailable)?))
 }
 pub(in crate::pet_data)fn owner_current(&self)->Result<()>{self.claim.for_layout(&self.layout)?;for p in &self.parents{p.check()?;}Ok(())}
 pub(in crate::pet_data)fn current(&self)->Result<()>{self.claim.for_layout(&self.layout)?;for dir in &self.parents{dir.check()?;}self.source.ensure_current()}
 pub(in crate::pet_data)fn fingerprint(&self)->Result<String>{
  self.current()?;let(roots,pair)=self.existing()?;if pair!=self.pair||self.target_snapshot(&roots)?!=self.target_fence{return Err(Error::SourceChanged);}
  let mut locations=vec![];for dir in &self.parents{let at=private_fs::location(&dir.node()?.file)?;locations.push((at.device,at.inode,at.mount));}
  let bytes=serde_json::to_vec(&("coucou-current-profile-restore-data-review-v1",&self.source_fence,&self.content,digest(&self.origin),layout_key(&self.layout),locations,&self.target_fence,&self.pair,match self.mode{Mode::Fresh=>"fresh",Mode::Resume=>"resume",Mode::Activation=>"activation",Mode::RetainIndependent=>"retainIndependent"})).map_err(|_|Error::Unavailable)?;self.current()?;Ok(digest(&bytes))
 }
 pub(in crate::pet_data)fn preview(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"status":"restoreDataReview","schemaVersion":self.source.archive_version(),"archiveVersion":self.source.archive_version(),"destinationProfileVersion":self.profile_version(),"operationalPayloadDepth":self.operational_depth()?,"payloadScope":if self.source.archive_version()==1{"currentPayloadPair"}else{"historicalRootPair"},"fingerprint":self.fingerprint()?,"archiveId":self.source.id(),"archiveFence":self.source_fence,"destination":{"config":self.layout.config(),"data":self.layout.data()},"pair":self.pair,"mode":if self.mode==Mode::Fresh{"fresh"}else{"resume"},"summary":self.source.summary(),"result":"dataRestoredInactive","executionAuthorityGranted":false,"runtimeActivationAvailable":false,"historicalAuthorityReconciliationRequired":true,"sensitiveSettingsNotActivated":true}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<serde_json::Value,Failure>{self.apply_using(approved,|_|Ok(()))}
 fn apply_using(self,approved:&str,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<serde_json::Value,Failure>{
  let mut pair=self.pair.clone();let result=(||->Result<serde_json::Value>{
   if self.mode==Mode::RetainIndependent{return Err(Error::UnsupportedProfile);}if approved!=self.fingerprint()?{return Err(Error::SourceChanged);}
   let(mut roots,_)=self.existing()?;
   if roots.iter().any(Option::is_none){for root in roots.iter().flatten(){if read(&root.dir,COMPLETE)?.is_some()||read(&root.dir,RESTORED)?.is_some(){return Err(Error::SourceChanged);}}}
   let id=match &pair{Some(p)=>p.clone(),None=>{let mut random=[0u8;16];SystemRandom::new().fill(&mut random).map_err(|_|Error::Unavailable)?;hex(&random)}};pair=Some(id.clone());let intent=encode(&self.intent(&id))?;
   for(index,(path,parent,area))in[(self.layout.config(),&self.layout.legacy_config,Area::Config),(self.layout.data(),&self.layout.legacy_data,Area::Data)].into_iter().enumerate(){
    if roots[index].is_none(){self.current()?;let origin:BackupOrigin=parse(&self.origin)?;let mut extras=vec![(INTENT,intent.as_slice()),(ORIGIN,self.origin.as_slice())];if area==Area::Config{extras.extend(super::super::settings_io::extra(origin.settings_io_protocol)?);}let(dir,header)=root_with_controls(parent,path,area,&id,&layout_key(&self.layout),self.profile_version(),&extras)?;roots[index]=Some(Root{dir,header,intent:intent.clone(),origin:self.origin.clone()});step(if index==0{Step::ConfigCreated}else{Step::DataCreated})?;self.current()?;}
   }
   let config=roots[0].as_ref().ok_or(Error::SourceChanged)?;let data=roots[1].as_ref().ok_or(Error::SourceChanged)?;
   let check=||->Result<()>{self.claim.for_layout(&self.layout)?;for (root,area)in [(config,Area::Config),(data,Area::Data)]{root.dir.check()?;root_controls(&root.dir,area)?;if read(&root.dir,HEADER)?.as_deref()!=Some(root.header.as_slice())||read(&root.dir,INTENT)?.as_deref()!=Some(root.intent.as_slice())||read_origin(&root.dir)?!=root.origin{return Err(Error::SourceChanged);}}Ok(())};
   check()?;let cp=config.dir.dir(OsStr::new(PAYLOAD))?;let dp=data.dir.dir(OsStr::new(PAYLOAD))?;
   let any_complete=read(&config.dir,COMPLETE)?.is_some()||read(&data.dir,COMPLETE)?.is_some()||read(&config.dir,RESTORED)?.is_some();
   if any_complete{staging::verify_publication(&self.source,&cp,&dp)?;}else{staging::resume_publication(&self.source,&cp,&dp,|index|{step(Step::BeforeCopy(index))?;check()})?;}
   staging::verify_publication(&self.source,&cp,&dp)?;self.current()?;step(Step::Copied)?;check()?;
   let records=[encode(&Complete{version:self.profile_version(),pair:id.clone(),area:Area::Config,content:self.content.clone(),summary:self.source.summary()})?,encode(&Complete{version:self.profile_version(),pair:id.clone(),area:Area::Data,content:self.content.clone(),summary:self.source.summary()})?];
   for(index,root)in[config,data].into_iter().enumerate(){check()?;match read(&root.dir,COMPLETE)?{Some(existing)if existing!=records[index]=>return Err(Error::SourceChanged),Some(_)=>{},None=>{root.dir.publish_new(COMPLETE,&records[index])?;}}step(if index==0{Step::ConfigComplete}else{Step::DataComplete})?;self.current()?;}
   let restored=encode(&Restored{version:self.source.archive_version(),pair:id.clone(),layout:layout_key(&self.layout),source:digest(&intent),config:digest(&records[0]),data:digest(&records[1]),authority_state:if self.source.archive_version()==1{"historicalOnlyPendingReconciliation"}else{"historicalRootPairPendingEpochReconciliation"}.into()})?;
   staging::verify_publication(&self.source,&cp,&dp)?;self.current()?;check()?;
   if let Some(existing)=read(&config.dir,RESTORED)?{if existing!=restored{return Err(Error::SourceChanged);}}else{config.dir.publish_new(RESTORED,&restored)?;}
   config.dir.sync()?;data.dir.sync()?;step(Step::Restored)?;self.current()?;check()?;
   Ok(serde_json::json!({"status":"dataRestoredInactive","schemaVersion":self.source.archive_version(),"archiveVersion":self.source.archive_version(),"destinationProfileVersion":self.profile_version(),"operationalPayloadDepth":self.operational_depth()?,"payloadScope":if self.source.archive_version()==1{"currentPayloadPair"}else{"historicalRootPair"},"pair":id,"archiveId":self.source.id(),"receiptHash":digest(&restored),"summary":self.source.summary(),"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"historicalAuthorityReconciliationRequired":true,"sensitiveSettingsNotActivated":true}))
  })();result.map_err(|error|Failure{error,pair})
 }
}
#[cfg(test)]impl Review{
 pub(in crate::pet_data)fn fixture_interrupt(self,approved:&str,phase:u8)->std::result::Result<serde_json::Value,Failure>{self.apply_using(approved,|s|if matches!((phase,s),(0,Step::ConfigCreated)|(1,Step::DataCreated)|(2,Step::BeforeCopy(0))|(3,Step::Copied)|(4,Step::ConfigComplete)|(5,Step::DataComplete)|(6,Step::Restored)){Err(Error::Unavailable)}else{Ok(())})}
}

#[cfg(test)]#[path="publication_restore_tests.rs"]mod tests;
#[path="publication_restore_authority.rs"]mod authority;
pub(in crate::pet_data)use authority::AuthorityReview;

#[path="publication_restore_configuration.rs"]pub(in crate::pet_data)mod configuration;

#[path="publication_restore_epoch.rs"]pub(in crate::pet_data)mod epoch;

#[path="publication_restore_independent.rs"]pub(in crate::pet_data)mod independent;

#[path="publication_restore_retention.rs"]pub(in crate::pet_data)mod retention;
