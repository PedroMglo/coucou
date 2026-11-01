//! Native inspection receipt only. This is NOT permission to activate settings,
//! an executable data admission, a goal consent or a replacement policy ledger.
use super::*;
use crate::pet_runtime::restoration;
use serde_json::{Value,json};
use crate::pet_data::{restored_history::History as ReferenceHistory,restored_transaction,staging::Source};
const MAX_CONFIGURATION:usize=256*1024;
pub(in crate::pet_data)fn retained_receipt_head(name:&str)->Option<&str>{let head=name.strip_prefix(CONFIG_RETAINED_PREFIX)?.strip_suffix(".json")?;valid_hex(head,64).then_some(head)}
pub(in crate::pet_data)fn independent_receipt_name(custody:&super::independent::operational::Custody)->Result<String>{custody.validate(&custody.history,&custody.authority)?;Ok(match &custody.retention{None=>CONFIG_INDEPENDENT_REVIEW.into(),Some(b)=>format!("{CONFIG_RETAINED_PREFIX}{}.json",b.head)})}
fn independent_receipt_version(custody:&super::independent::operational::Custody)->u32{if custody.retention.is_some(){4}else{3}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
enum History{Quarantined{#[serde(rename="historyHash")]history_hash:String},NoOriginalAuthority}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Receipt{version:u32,purpose:String,pair:String,source:String,settings:String,pets:String,history:History,report:String,#[serde(default,skip_serializing_if="Option::is_none")]scope:Option<EpochScope>,#[serde(default,skip_serializing_if="Option::is_none")]independent:Option<super::independent::operational::Custody>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(in crate::pet_data)struct EpochScope{pub custody:super::epoch::Custody,pub references:Value}
#[derive(Clone,Copy,PartialEq,Eq)]enum PublicationStep{BeforeReceipt,AfterReceipt}
pub(in crate::pet_data)struct ConfigurationReview{data:Review,report:Value,receipt:Receipt}
fn value<T:Serialize>(v:T)->Result<Value>{serde_json::to_value(v).map_err(|_|Error::UnsupportedProfile)}
fn object(bytes:&[u8])->Result<Value>{let bytes=bytes.strip_prefix(&[0xef,0xbb,0xbf]).unwrap_or(bytes);let v:Value=serde_json::from_slice(bytes).map_err(|_|Error::UnsupportedProfile)?;if !v.is_object(){return Err(Error::UnsupportedProfile);}Ok(v)}
fn fields(v:&Value,allowed:&[&str])->Result<()>{let o=v.as_object().ok_or(Error::UnsupportedProfile)?;if o.keys().any(|k|!allowed.contains(&k.as_str())){return Err(Error::UnsupportedProfile);}Ok(())}
fn closed_pets(v:&Value)->Result<()>{
 fields(v,&["version","reasoningTerms","reasoningLength","pets"])?;
 for p in v["pets"].as_object().ok_or(Error::UnsupportedProfile)?.values(){
  fields(p,&["name","aliases","provider","mode","fast","reasoning","permissions","localExecution"])?;
  for name in ["fast","reasoning"]{fields(&p[name],&["model","effort"])?;}
  if let Some(p)=p.get("permissions").filter(|p|!p.is_null()){fields(p,&["documents","emailDrafts","music","openApps"])?;}
  if let Some(p)=p.get("localExecution").filter(|p|!p.is_null()){fields(p,&["mode","taskTimeoutMs","approvalTimeoutMs","commandTimeoutMs","maxCalls","maxOutputBytes","allowedPrograms"])?;}
 }Ok(())
}
fn known(raw:&Value,resolved:&Value)->Result<()>{match(raw,resolved){(Value::Object(a),Value::Object(b))=>for(k,v)in a{known(v,b.get(k).ok_or(Error::UnsupportedProfile)?)?;},(Value::Array(a),Value::Array(b))=>{if a.len()!=b.len(){return Err(Error::UnsupportedProfile);}for(a,b)in a.iter().zip(b){known(a,b)?;}},_=>{}}Ok(())}
fn differences(raw:&Value,resolved:&Value,at:String,out:&mut Vec<String>){
 if raw==resolved{return;}match(raw,resolved){(Value::Object(a),Value::Object(b))=>for(k,v)in b{let path=if at.is_empty(){k.clone()}else{format!("{at}.{k}")};if let Some(original)=a.get(k){differences(original,v,path,out);}else{out.push(path);}},_=>out.push(at)}
}
fn group(field:&str)->&'static str{match field{
 "model"|"chatProvider"|"chatModels"|"ollamaUrl"|"lmstudioUrl"|"customUrl"=>"inferenceSelections",
 "cliWorkspace"|"cliWorkspaces"|"cliFullAccess"=>"nativeExecution",
 "activeIntegrations"|"mainPill"|"spotifyQuietAds"|"autostart"|"hooksInstalled"|"planRelayInstalled"|"showPlanInNotch"|"showCodexPlanInNotch"|"petSystemCenter"|"shortcuts"=>"systemAndIntegrations",
 s if s.starts_with("phone")=>"mobile",
 s if s.starts_with("voice")||s=="edgeVoicePython"||s=="localVoiceDirs"=>"voice",
 "autoCloseInterval"|"openOnHover"|"absenceInterval"|"screen"|"companionMode"|"companionMotion"|"companionBarMode"|"petControlBar"|"companionTopClearance"|"mochiOutfit"|"pillColors"|"language"|"desktopMochi"=>"presentation",
 "soundEnabled"|"soundVolume"=>"systemAndIntegrations",
 _=>"requiresClassification",
}}
fn inspect(settings:Option<&[u8]>,pets:Option<&[u8]>)->Result<Value>{
 let raw_settings=match settings{Some(b)=>object(b)?,None=>json!({})};
 // Unlike settings::load_from, a malformed known field cannot be salvaged or
 // written aside. All default-derived values are explicitly shown as changes.
 let resolved:crate::settings::Settings=serde_json::from_value(raw_settings.clone()).map_err(|_|Error::UnsupportedProfile)?;let resolved=value(resolved)?;known(&raw_settings,&resolved)?;
 let mut setting_changes=vec![];differences(&raw_settings,&resolved,String::new(),&mut setting_changes);
 let mut groups=serde_json::Map::new();for(k,v)in resolved.as_object().ok_or(Error::UnsupportedProfile)?{groups.entry(group(k)).or_insert_with(||json!({})).as_object_mut().unwrap().insert(k.clone(),v.clone());}
 let raw_pets=match pets{Some(b)=>object(b)?,None=>json!({})};
 let proposed=if let Some(b)=pets{closed_pets(&raw_pets)?;crate::pet_config::resolve_existing_bytes(b).map_err(|_|Error::UnsupportedProfile)?}else{crate::pet_config::defaults()};let proposed=value(proposed)?;
 let mut pet_changes=vec![];differences(&raw_pets,&proposed,String::new(),&mut pet_changes);
 let report=json!({"schemaVersion":1,"settings":{"present":settings.is_some(),"resolvedGroups":groups,"resolutionChanges":setting_changes},"pets":{"present":pets.is_some(),"resolved":proposed,"resolutionChanges":pet_changes},"activationGranted":false,"oldApprovalsRenewed":false,"remainingActivationReview":["select exact settings to activate","sources and persistent data grants","manual/native session authority","history hash and executable data admission","no-authority and multi-epoch policy"],"runtimeActivationAvailable":false});
 if serde_json::to_vec(&report).map_err(|_|Error::Unavailable)?.len()>1024*1024{return Err(Error::Limit);}Ok(report)
}
fn configuration_file(root:&Dir,name:&str)->Result<Option<Vec<u8>>>{
 let parent=root.node()?;let Some(node)=parent.child(OsStr::new(name))?else{return Ok(None);};
 if node.directory||node.len>MAX_CONFIGURATION as u64||!staging_fs::private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
 let mut bytes=vec![];(&node.file).take(MAX_CONFIGURATION as u64+1).read_to_end(&mut bytes).map_err(|_|Error::Unavailable)?;node.current()?;parent.same_child(OsStr::new(name),&node)?;
 if bytes.len()!=node.len as usize||bytes.len()>MAX_CONFIGURATION{return Err(Error::SourceChanged);}Ok(Some(bytes))
}
fn data_current(data:&Review)->Result<()>{match data.profile_version(){2=>data.authority_data_current(),3=>data.restored_data_current(),_=>Err(Error::UnsupportedProfile)}}
fn operational_config(data:&Review)->Result<Dir>{restored_transaction::payload_at_depth(&Dir::root(data.layout.config())?,data.operational_depth()?)}
fn consent_path(data:&Review)->Result<PathBuf>{Ok(operational_config(data)?.bound_path()?.join("pet-runtime-consent-v1"))}
fn epoch_scope(data:&Review)->Result<EpochScope>{
 let custody=super::epoch::completed(data)?;
 let combined=source_references(data)?;let references=combined.review()?;data.current()?;Ok(EpochScope{custody,references})
}
pub(in crate::pet_data)fn source_references(data:&Review)->Result<ReferenceHistory>{references_from_roots(&Dir::root(data.source.source_root(Area::Config))?,&Dir::root(data.source.source_root(Area::Data))?)}
pub(in crate::pet_data)fn references_from_roots(source_config:&Dir,source_data:&Dir)->Result<ReferenceHistory>{
 // CurrentArchive has already verified the complete old Admission2, seal,
 // original roots and metadata. Reuse its exact authenticated history hash.
 let admission=restored_transaction::read_file(&source_config,ADMITTED,4096)?.ok_or(Error::UnsupportedProfile)?;
 let admission:crate::pet_data::publication::active_restored::Admission2=parse(&admission)?;let depth=crate::pet_data::publication::admitted_depth(source_config)?;
 let job=source_config.dir(OsStr::new(if admission.version==3{restored_transaction::EPOCH_JOB}else{restored_transaction::JOB}))?;
 let previous=restored_transaction::read_file(&job,restored_transaction::HISTORY,crate::pet_data::restored_history::MAX_INDEX)?.ok_or(Error::SourceChanged)?;
 let prior=if matches!(admission.version,2|4){ReferenceHistory::decode_first(&previous,&admission.history)?}else{ReferenceHistory::decode(&previous,&admission.history)?};if let Some(e)=&admission.epoch{if prior.review()?!=e.scope.references{return Err(Error::SourceChanged);}}
 let library=restored_transaction::read_file(&restored_transaction::payload_at_depth(source_data,depth)?,"assistant/conversations.json",crate::assistant_memory::MAX_BYTES as usize)?;
 ReferenceHistory::capture_next(library.as_deref(),&previous,&admission.history)
}

impl Review{
 pub(in crate::pet_data)fn configuration(self)->Result<ConfigurationReview>{
  if self.mode==Mode::RetainIndependent{return Err(Error::UnsupportedProfile);}
  data_current(&self)?;let root=Dir::root(self.layout.config())?;let payload=operational_config(&self)?;let path=consent_path(&self)?;
  let scope=if self.profile_version()==3{Some(epoch_scope(&self)?)}else{None};
  let independent=if self.profile_version()==2&&root.node()?.child(OsStr::new(super::independent::JOB))?.is_some()&&read_file_ready(&root)?{Some(super::independent::operational::capture(&self,&[])?.custody)}else{None};
  let history=if independent.is_some(){History::NoOriginalAuthority}else if let Some(scope)=&scope{History::Quarantined{history_hash:scope.custody.authority.clone()}}else if private_fs::Node::root(&path)?.is_none(){self.authority_verify_delta(&[])?;History::NoOriginalAuthority}else{
   let delta=restoration::publication_delta(&path).map_err(authority::error)?;self.authority_verify_delta(&delta)?;
   let h=restoration::historical_view(&path).map_err(authority::error)?;let history_hash=h["historyHash"].as_str().filter(|h|valid_hex(h,64)).ok_or(Error::UnsupportedProfile)?.to_owned();History::Quarantined{history_hash}
  };
  let settings=configuration_file(&payload,"settings.json")?;let pets=configuration_file(&payload,"pets.json")?;let mut report=inspect(settings.as_deref(),pets.as_deref())?;
  if let Some(scope)=&scope{report["schemaVersion"]=json!(2);report["epochReferenceReview"]=scope.references.clone();report["remainingActivationReview"]=json!(["exact activation selections","sealed operational admission3","voice/mobile/autostart revalidation","fresh goal and provider data consent"]);}
  if let Some(i)=&independent{report=independent_report(settings.as_deref(),pets.as_deref(),i)?;}
  let receipt=Receipt{version:if let Some(i)=&independent{independent_receipt_version(i)}else if scope.is_some(){2}else{1},purpose:"configurationInspectionOnly".into(),pair:self.pair.clone().ok_or(Error::UnsupportedProfile)?,source:self.source_fence.clone(),settings:settings.as_deref().map(digest).unwrap_or_else(||"absent".into()),pets:pets.as_deref().map(digest).unwrap_or_else(||"absent".into()),history,report:digest(&serde_json::to_vec(&report).map_err(|_|Error::Unavailable)?),scope,independent};
  let _=root;data_current(&self)?;let review=ConfigurationReview{data:self,report,receipt};review.current()?;Ok(review)
 }
}
impl ConfigurationReview{
 fn verify_payload(&self)->Result<()>{
  if let Some(expected)=&self.receipt.independent{let p=super::independent::operational::capture(&self.data,&[])?;if self.receipt.version!=independent_receipt_version(expected)||self.receipt.scope.is_some()||self.receipt.history!=History::NoOriginalAuthority||p.custody!=*expected{return Err(Error::SourceChanged);}return Ok(());}
  if self.data.profile_version()==3{let scope=epoch_scope(&self.data)?;if self.receipt.version!=2||self.receipt.scope.as_ref()!=Some(&scope)||self.receipt.history!= (History::Quarantined{history_hash:scope.custody.authority}){return Err(Error::SourceChanged);}return Ok(());}
  if self.receipt.version!=1||self.receipt.scope.is_some(){return Err(Error::SourceChanged);}let path=consent_path(&self.data)?;match &self.receipt.history{
   History::NoOriginalAuthority=>{if private_fs::Node::root(&path)?.is_some(){return Err(Error::SourceChanged);}self.data.authority_verify_delta(&[])},
   History::Quarantined{history_hash}=>{let h=restoration::historical_view(&path).map_err(authority::error)?;if h["historyHash"].as_str()!=Some(history_hash.as_str()){return Err(Error::SourceChanged);}let delta=restoration::publication_delta(&path).map_err(authority::error)?;self.data.authority_verify_delta(&delta)},
  }
 }
 fn receipt_name(&self)->Result<String>{match &self.receipt.independent{Some(i)=>independent_receipt_name(i),None=>Ok(CONFIG_REVIEW.into())}}
 fn current(&self)->Result<()>{self.data.fingerprint()?;self.verify_payload()?;data_current(&self.data)?;let(roots,_)=self.data.existing()?;let root=&roots[0].as_ref().ok_or(Error::UnsupportedProfile)?.dir;let payload=operational_config(&self.data)?;
  for(name,expected)in[("settings.json",&self.receipt.settings),("pets.json",&self.receipt.pets)]{let bytes=configuration_file(&payload,name)?;let hash=bytes.as_deref().map(digest).unwrap_or_else(||"absent".into());if &hash!=expected{return Err(Error::SourceChanged);}}
  if let Some(saved)=read(root,&self.receipt_name()?)?{if saved!=encode(&self.receipt)?{return Err(Error::SourceChanged);}}Ok(())
 }
 fn fingerprint(&self)->Result<String>{self.current()?;Ok(digest(&serde_json::to_vec(&("coucou-restored-configuration-inspection-v1",self.data.fingerprint()?,&self.receipt)).map_err(|_|Error::Unavailable)?))}
 pub(in crate::pet_data)fn preview(&self)->Result<Value>{Ok(json!({"status":"restoredConfigurationReview","schemaVersion":self.receipt.version,"destinationProfileVersion":self.data.profile_version(),"operationalPayloadDepth":self.data.operational_depth()?,"scope":self.receipt.scope,"independent":self.receipt.independent,"fingerprint":self.fingerprint()?,"pair":self.receipt.pair,"source":self.receipt.source,"history":self.receipt.history,"configuration":self.report,"consentPurpose":"configurationInspectionOnly","executionAuthorityGranted":false,"settingsActivated":false,"runtimeActivationAvailable":false}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<Value,Failure>{self.apply_using(approved,|_|Ok(()))}
 fn apply_using(self,approved:&str,mut boundary:impl FnMut(PublicationStep)->Result<()>)->std::result::Result<Value,Failure>{
  let pair=self.data.pair.clone();let result=(||{if approved!=self.fingerprint()?{return Err(Error::SourceChanged);}boundary(PublicationStep::BeforeReceipt)?;self.current()?;let(roots,_)=self.data.existing()?;let root=&roots[0].as_ref().ok_or(Error::UnsupportedProfile)?.dir;let bytes=encode(&self.receipt)?;
   if let Some(saved)=read(root,&self.receipt_name()?)?{if saved!=bytes{return Err(Error::SourceChanged);}}else{root.publish_new(&self.receipt_name()?,&bytes)?;}
   boundary(PublicationStep::AfterReceipt)?;self.verify_payload()?;data_current(&self.data)?;if read(root,&self.receipt_name()?)?.as_deref()!=Some(bytes.as_slice()){return Err(Error::SourceChanged);}
   Ok(json!({"status":"restoredConfigurationReviewed","schemaVersion":self.receipt.version,"destinationProfileVersion":self.data.profile_version(),"operationalPayloadDepth":self.data.operational_depth()?,"receiptHash":digest(&bytes),"pair":pair,"purpose":"configurationInspectionOnly","executionAuthorityGranted":false,"settingsActivated":false,"runtimeActivationAvailable":false}))})();result.map_err(|error|Failure{error,pair})
 }
}
#[cfg(test)]#[path="publication_restore_configuration_tests.rs"]mod tests;

/// Exact source-based inspection record, also used when an activation resumes
/// after canonical files have already been transformed. No target repair.
pub(in crate::pet_data)fn source_inspection(settings:Option<&[u8]>,pets:Option<&[u8]>,pair:&str,source:&str,history_hash:&str)->Result<(Value,Vec<u8>)>{
 let report=inspect(settings,pets)?;let receipt=Receipt{version:1,purpose:"configurationInspectionOnly".into(),pair:pair.into(),source:source.into(),settings:settings.map(digest).unwrap_or_else(||"absent".into()),pets:pets.map(digest).unwrap_or_else(||"absent".into()),history:History::Quarantined{history_hash:history_hash.into()},report:digest(&serde_json::to_vec(&report).map_err(|_|Error::Unavailable)?),scope:None,independent:None};Ok((report,encode(&receipt)?))
}

pub(in crate::pet_data)fn source_report(settings:Option<&[u8]>,pets:Option<&[u8]>)->Result<Value>{inspect(settings,pets)}
pub(in crate::pet_data)fn source_inspection_next(data:&Review,settings:Option<&[u8]>,pets:Option<&[u8]>,extra:&[(Area,PathBuf,Vec<u8>)])->Result<(Value,Vec<u8>,Vec<u8>,EpochScope)>{
 let custody=super::epoch::completed_using(data,extra)?;let history=source_references(data)?;let scope=EpochScope{custody,references:history.review()?};let mut report=inspect(settings,pets)?;
 report["schemaVersion"]=json!(2);report["epochReferenceReview"]=scope.references.clone();report["remainingActivationReview"]=json!(["exact activation selections","sealed operational admission3","voice/mobile/autostart revalidation","fresh goal and provider data consent"]);
 let receipt=Receipt{version:2,purpose:"configurationInspectionOnly".into(),pair:data.pair.clone().ok_or(Error::UnsupportedProfile)?,source:data.source_fence.clone(),settings:settings.map(digest).unwrap_or_else(||"absent".into()),pets:pets.map(digest).unwrap_or_else(||"absent".into()),history:History::Quarantined{history_hash:scope.custody.authority.clone()},report:digest(&serde_json::to_vec(&report).map_err(|_|Error::Unavailable)?),scope:Some(scope.clone()),independent:None};data.current()?;Ok((report,encode(&receipt)?,history.encode()?,scope))
}

fn read_file_ready(root:&Dir)->Result<bool>{let job=root.dir(OsStr::new(super::independent::JOB))?;Ok(restored_transaction::read_file(&job,"ready.json",64*1024)?.is_some())}
fn independent_report(settings:Option<&[u8]>,pets:Option<&[u8]>,custody:&super::independent::operational::Custody)->Result<Value>{let mut report=inspect(settings,pets)?;report["schemaVersion"]=json!(independent_receipt_version(custody));report["independentAuthorityReview"]=value(custody)?;if custody.retention.is_some(){report["retainedBytesTrustedAsAuthority"]=json!(false);}report["remainingActivationReview"]=json!(["exact activation selections","same prepared key publication and admission4","voice/mobile/autostart revalidation","fresh goal and provider data consent"]);Ok(report)}
pub(in crate::pet_data)fn source_inspection_independent(data:&Review,settings:Option<&[u8]>,pets:Option<&[u8]>,delta:&[(Area,PathBuf,Vec<u8>)])->Result<(Value,Vec<u8>,Vec<u8>,super::independent::operational::Custody)>{
 let p=super::independent::operational::capture(data,delta)?;let report=independent_report(settings,pets,&p.custody)?;let receipt=Receipt{version:independent_receipt_version(&p.custody),purpose:"configurationInspectionOnly".into(),pair:data.pair.clone().ok_or(Error::UnsupportedProfile)?,source:data.source_fence.clone(),settings:settings.map(digest).unwrap_or_else(||"absent".into()),pets:pets.map(digest).unwrap_or_else(||"absent".into()),history:History::NoOriginalAuthority,report:digest(&serde_json::to_vec(&report).map_err(|_|Error::Unavailable)?),scope:None,independent:Some(p.custody.clone())};data.current()?;Ok((report,encode(&receipt)?,p.history,p.custody))
}
