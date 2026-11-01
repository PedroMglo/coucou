//! Immutable denial/provenance metadata over the EXISTING grant ledger. Old
//! signed bodies and their consumption remain data; no new allowance is minted.
use super::*;
use crate::pet_runtime::{store::Store,model::{goal_id,token}};
use std::{collections::{BTreeMap,BTreeSet},path::PathBuf,os::unix::ffi::OsStrExt};
pub(crate)use crate::pet_runtime::authority::Failure as HistoryFailure;
const INDEX:&str=".restored-history.json";
const MAX_INDEX:usize=8*1024*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct FileRef{name:String,hash:String,bytes:usize}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct HistoricalGrant{file:String,file_hash:String,signature_valid:bool,decision_id:Option<String>,goal_id:Option<String>,source_phase:Option<GrantPhase>,usage_ledger_recorded:Option<bool>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct GoalRef{id:String,pet_id:String,hash:String,decisions:Vec<String>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Body{schema_version:u32,key_id:String,source:String,files:Vec<FileRef>,grants:Vec<HistoricalGrant>,goals:Vec<GoalRef>,decisions:Vec<String>,goal_ids:Vec<String>,#[serde(default,skip_serializing_if="Option::is_none")]parent:Option<String>,#[serde(default,skip_serializing_if="Vec::is_empty")]priors:Vec<IndexRef>}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct IndexRef{hash:String,bytes:usize}
const MAX_PRIORS:usize=8;const MAX_PRIOR_BYTES:usize=64*1024*1024;
fn prior_name(hash:&str)->String{format!(".restored-history-prior-{hash}.json")}
fn prior_hash(name:&str)->Option<&str>{name.strip_prefix(".restored-history-prior-").and_then(|s|s.strip_suffix(".json")).filter(|s|hash_valid(s))}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Envelope{body:Body,tag:String}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Protocol{version:u32,kind:String,history:String}
pub(super)struct Boundary{body:Body,stamp:Stamp,hash:String,prior_stamps:Vec<(String,Stamp)>,historical_files:BTreeSet<String>}
fn hash_valid(s:&str)->bool{s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
pub(super)fn protocol_hash(bytes:&[u8])->Result<Option<String>>{
 if bytes==PROTOCOL{return Ok(None);}let p:Protocol=serde_json::from_slice(bytes).map_err(|_|Failure::Invalid)?;
 if !matches!(p.version,2|3)||p.kind!="coucou-native-consent"||!hash_valid(&p.history){return Err(Failure::Invalid);}Ok(Some(p.history))
}
fn protocol_version(bytes:&[u8])->Result<u32>{if bytes==PROTOCOL{Ok(1)}else{serde_json::from_slice::<Protocol>(bytes).map(|p|p.version).map_err(|_|Failure::Invalid)}}
fn message(body:&Body)->Result<Vec<u8>>{let mut b=if body.schema_version==1{b"coucou-restored-authority-history-v1\0".to_vec()}else{b"coucou-restored-authority-history-v2\0".to_vec()};b.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(b)}
fn valid(body:&Body)->bool{
 if !matches!(body.schema_version,1|2)||!token(&body.key_id)||!hash_valid(&body.source)||body.files.len()>4096||body.grants.len()>4096||body.goals.len()>4096||body.decisions.len()>32768||body.goal_ids.len()>8192{return false;}
 if body.schema_version==1{if body.parent.is_some()||!body.priors.is_empty(){return false;}}else{
  if body.priors.is_empty()||body.priors.len()>MAX_PRIORS||body.parent.as_ref().is_none_or(|p|!body.priors.iter().any(|r|&r.hash==p))||body.priors.iter().any(|r|!hash_valid(&r.hash)||r.bytes==0||r.bytes>MAX_INDEX)||body.priors.windows(2).any(|w|w[0].hash>=w[1].hash)||body.priors.iter().map(|p|p.bytes).sum::<usize>()>MAX_PRIOR_BYTES{return false;}
 }
 let unique=|v:Vec<&str>|v.iter().copied().collect::<BTreeSet<_>>().len()==v.len();
 unique(body.files.iter().map(|f|f.name.as_str()).collect())&&unique(body.decisions.iter().map(String::as_str).collect())&&unique(body.goal_ids.iter().map(String::as_str).collect())
 &&body.files.iter().all(|f|!f.name.is_empty()&&!f.name.contains('/')&&hash_valid(&f.hash)&&f.bytes<=MAX_GRANT_BYTES)
 &&body.decisions.iter().all(|d|token(d))&&body.goal_ids.iter().all(|g|goal_id(g))
 &&body.grants.iter().all(|g|body.files.iter().any(|f|f.name==g.file&&f.hash==g.file_hash)&&g.decision_id.as_ref().is_none_or(|d|body.decisions.contains(d))&&g.goal_id.as_ref().is_none_or(|id|body.goal_ids.contains(id)))
 &&body.goals.iter().all(|g|goal_id(&g.id)&&body.goal_ids.contains(&g.id)&&hash_valid(&g.hash)&&g.decisions.iter().all(|d|body.decisions.contains(d)))
}
fn decode_history(a:&Authority,raw:&[u8])->Result<Body>{
 let value:serde_json::Value=serde_json::from_slice(raw).map_err(|_|Failure::Invalid)?;
 if value["body"]["schemaVersion"]==1&&value["body"].as_object().is_some_and(|o|o.contains_key("parent")||o.contains_key("priors")){return Err(Failure::Invalid);}
 let e:Envelope=serde_json::from_value(value).map_err(|_|Failure::Invalid)?;if !valid(&e.body)||e.body.key_id!=a.key_id{return Err(Failure::Invalid);}
 hmac::verify(&a.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;Ok(e.body)
}
fn marker(a:&Authority)->Result<Option<Boundary>>{marker_mode(a,false)}
fn epoch_marker(a:&Authority)->Result<Option<Boundary>>{marker_mode(a,true)}
fn marker_mode(a:&Authority,allow_current_prior:bool)->Result<Option<Boundary>>{
 let Some((raw,stamp))=a.dir.bytes(INDEX,MAX_INDEX).map_err(|_|Failure::Unsafe)?else{return Ok(None);};let body=decode_history(a,&raw)?;let hash=model::hash(&raw);
 let mut prior_stamps=vec![];let mut ancestors=BTreeMap::new();let mut historical_files=body.grants.iter().map(|g|g.file.clone()).collect::<BTreeSet<_>>();
 for r in &body.priors{if r.hash==hash{return Err(Failure::Invalid);}let name=prior_name(&r.hash);let(bytes,s)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if bytes.len()!=r.bytes||model::hash(&bytes)!=r.hash{return Err(Failure::Changed);}let p=decode_history(a,&bytes)?;historical_files.extend(p.grants.iter().map(|g|g.file.clone()));ancestors.insert(r.hash.clone(),p);prior_stamps.push((name,s));}
 for name in a.dir.names().map_err(|_|Failure::Capacity)?{if let Some(h)=prior_hash(&name){if !body.priors.iter().any(|p|p.hash==h){if !allow_current_prior||h!=hash{return Err(Failure::RecoveryRequired);}let(extra,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if extra!=raw{return Err(Failure::Changed);}}}}
 let mut cursor=&body;let mut seen=BTreeSet::new();while let Some(parent)=&cursor.parent{if !seen.insert(parent.clone()){return Err(Failure::Invalid);}let p=ancestors.get(parent).ok_or(Failure::Invalid)?;let mut expected=p.priors.clone();let own=body.priors.iter().find(|r|&r.hash==parent).ok_or(Failure::Invalid)?;expected.push(own.clone());expected.sort_by(|a,b|a.hash.cmp(&b.hash));if cursor.priors!=expected||p.decisions.iter().any(|d|!cursor.decisions.contains(d))||p.goal_ids.iter().any(|g|!cursor.goal_ids.contains(g)){return Err(Failure::Invalid);}cursor=p;}
 if seen.len()!=body.priors.len(){return Err(Failure::Invalid);}Ok(Some(Boundary{body,stamp,hash,prior_stamps,historical_files}))
}
pub(super)fn load(a:&Authority)->Result<Option<Boundary>>{
 let expected=protocol_hash(&a.protocol)?;let boundary=marker(a)?;
 match(expected,boundary){(None,None)=>Ok(None),(Some(hash),Some(b))if hash==b.hash&&protocol_version(&a.protocol)?==b.body.schema_version+1=>Ok(Some(b)),_=>Err(Failure::RecoveryRequired)}
}
pub(super)fn fence(a:&Authority)->Result<()>{
 for n in a.dir.names().map_err(|_|Failure::Capacity)?{if n.strip_prefix(".restore-prior-").is_some_and(token)||prior_hash(&n).is_some()&&!a.restoration.as_ref().is_some_and(|b|b.prior_stamps.iter().any(|(name,_)|name==&n)){return Err(Failure::RecoveryRequired);}}
 match &a.restoration{Some(b)=>{if !a.dir.has_stamp(INDEX,&b.stamp).map_err(|_|Failure::Changed)?{return Err(Failure::Changed);}for(n,s)in &b.prior_stamps{if !a.dir.has_stamp(n,s).map_err(|_|Failure::Changed)?{return Err(Failure::Changed);}}},None=>{if a.dir.open_file(INDEX,libc::O_RDONLY).map_err(|_|Failure::Changed)?.is_some(){return Err(Failure::RecoveryRequired);}}}Ok(())
}
impl Authority{
 pub(super)fn historical_file(&self,name:&str)->bool{self.restoration.as_ref().is_some_and(|b|name==INDEX||b.prior_stamps.iter().any(|(n,_)|n==name)||b.historical_files.contains(name))}
 pub(super)fn restoration_current(&self,decision:&str,goal:&str)->Result<()>{if self.restoration.as_ref().is_some_and(|b|b.body.decisions.iter().any(|d|d==decision)||b.body.goal_ids.iter().any(|g|g==goal)){return Err(Failure::RecoveryRequired);}Ok(())}
}
fn goals(path:&Path)->Result<Vec<GoalRef>>{
 let Some(store)=Store::existing(path).map_err(|_|Failure::Unsafe)?else{return Ok(vec![]);};let listing=store.list(None).map_err(|_|Failure::Unavailable)?;if !listing.complete{return Err(Failure::RecoveryRequired);}
 let mut rows=vec![];for summary in listing.goals{let record=store.read(&summary.id,&summary.pet_id).map_err(|_|Failure::Changed)?;rows.push(GoalRef{id:record.id.clone(),pet_id:record.spec.owner.pet_id.clone(),hash:model::fingerprint(&record).map_err(|_|Failure::Invalid)?,decisions:record.admissions.iter().map(|r|r.spec.decision_id.clone()).collect()});}rows.sort_by(|a,b|a.id.cmp(&b.id));Ok(rows)
}
fn grant_name(name:&str)->Option<&str>{name.strip_suffix(".json").filter(|s|token(s)).or_else(||name.strip_prefix(".grant-").filter(|s|token(s)))}
fn files(a:&Authority)->Result<Vec<FileRef>>{
 let mut files=vec![];let mut staged=0;
 for name in a.dir.names().map_err(|_|Failure::Capacity)?{
  if name==INDEX||prior_hash(&name).is_some(){continue;}
  if name.strip_prefix(".restore-protocol-").is_some_and(token)||name.strip_prefix(".restore-index-").is_some_and(token)||name.strip_prefix(".restore-prior-").is_some_and(token){staged+=1;if staged>16{return Err(Failure::Capacity);}continue;}
  if !matches!(name.as_str(),KEY|".catalog.lock"|".protocol.json")&&grant_name(&name).is_none(){return Err(Failure::Invalid);}
  let(raw,_)=a.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
  let bytes=if name==".protocol.json"{PROTOCOL}else{raw.as_slice()};files.push(FileRef{name,hash:model::hash(bytes),bytes:bytes.len()});
 }files.sort_by(|a,b|a.name.cmp(&b.name));Ok(files)
}
fn capture(a:&Authority,path:&Path,source:&str)->Result<Body>{
 if !hash_valid(source){return Err(Failure::Invalid);}let files=files(a)?;let goals=goals(path)?;let mut decisions=BTreeSet::new();let mut goal_ids=BTreeSet::new();let mut grants=vec![];
 for g in &goals{goal_ids.insert(g.id.clone());for id in &g.decisions{decisions.insert(id.clone());}}
 for file in &files{let Some(id)=grant_name(&file.name)else{continue;};if file.name.ends_with(".json"){decisions.insert(id.into());}
  let verified=a.grant_file_data(&file.name).ok().map(|(b,_)|b);
  if let Some(body)=&verified{decisions.insert(body.decision_id.clone());goal_ids.insert(body.goal_id.clone());}
  grants.push(HistoricalGrant{file:file.name.clone(),file_hash:file.hash.clone(),signature_valid:verified.is_some(),decision_id:verified.as_ref().map(|b|b.decision_id.clone()),goal_id:verified.as_ref().map(|b|b.goal_id.clone()),source_phase:verified.as_ref().map(|b|b.phase),usage_ledger_recorded:verified.as_ref().map(|b|b.budget_usage.is_some())});
 }
 let body=Body{schema_version:1,key_id:a.key_id.clone(),source:source.into(),files,grants,goals,decisions:decisions.into_iter().collect(),goal_ids:goal_ids.into_iter().collect(),parent:None,priors:vec![]};if !valid(&body){return Err(Failure::Capacity);}Ok(body)
}
pub(crate)struct Review{authority:Authority,goals_path:PathBuf,body:Body}
#[derive(Clone,Copy,PartialEq,Eq)]enum Step{IndexPublished,ProtocolPublished}
impl Review{
 /// Caller holds the SAME native inactive-data claim; no model-facing route.
 pub(crate)fn capture(path:&Path,goals_path:&Path,source:&str,current:impl Fn()->Result<()>)->Result<Self>{
  current()?;let authority=Authority::restoration_base(path)?;authority.restoration_raw_fence()?;let existing=marker(&authority)?;
  let body=if let Some(boundary)=existing{if boundary.body.schema_version!=1||boundary.body.source!=source||boundary.body.files!=files(&authority)?||boundary.body.goals!=goals(goals_path)?{return Err(Failure::Changed);}if protocol_hash(&authority.protocol)?.is_some_and(|h|h!=boundary.hash){return Err(Failure::Changed);}boundary.body}else{if protocol_hash(&authority.protocol)?.is_some(){return Err(Failure::RecoveryRequired);}capture(&authority,goals_path,source)?};
  let result=Self{authority,goals_path:goals_path.into(),body};result.current(&current)?;Ok(result)
 }
 fn current(&self,current:impl Fn()->Result<()>)->Result<()>{current()?;self.authority.restoration_raw_fence()?;if files(&self.authority)?!=self.body.files||goals(&self.goals_path)?!=self.body.goals{return Err(Failure::Changed);}Ok(())}
 pub(crate)fn fingerprint(&self)->Result<String>{model::fingerprint(&("coucou-restored-authority-review-v1",self.authority.dir.path.as_os_str().as_bytes(),self.goals_path.as_os_str().as_bytes(),&self.body)).map_err(|_|Failure::Invalid)}
 pub(crate)fn preview(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"status":"historicalAuthorityReview","schemaVersion":1,"fingerprint":self.fingerprint()?,"source":self.body.source,"historicalDecisions":self.body.decisions.len(),"historicalGoals":self.body.goal_ids.len(),"grants":self.body.grants,"oldExecutionAvailable":false,"ownerAbsenceProven":false,"postSnapshotStateUnknown":true,"runtimeActivationAvailable":false}))}
 pub(crate)fn apply(self,approved:&str,current:impl Fn()->Result<()>)->Result<serde_json::Value>{self.apply_using(approved,current,|_|Ok(()))}
 fn apply_using(self,approved:&str,current:impl Fn()->Result<()>,mut step:impl FnMut(Step)->Result<()>)->Result<serde_json::Value>{
  if approved!=self.fingerprint()?{return Err(Failure::Changed);}self.current(&current)?;
  let file=self.authority.dir.open_file(".catalog.lock",libc::O_RDWR).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
  if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Failure::Busy);}let _guard=Guard(file);self.current(&current)?;
  let raw=serde_json::to_vec(&Envelope{body:self.body.clone(),tag:encode(hmac::sign(&self.authority.key,&message(&self.body)?).as_ref())}).map_err(|_|Failure::Invalid)?;if raw.len()>MAX_INDEX{return Err(Failure::Capacity);}let hash=model::hash(&raw);
  if let Some(existing)=marker(&self.authority)?{if existing.body!=self.body||existing.hash!=hash{return Err(Failure::Changed);}}else{
   let temp=format!(".restore-index-{}",nonce().map_err(|_|Failure::Unavailable)?);self.authority.dir.write_new(&temp,&raw).map_err(|_|Failure::Unavailable)?;self.current(&current)?;self.authority.dir.rename(&temp,&self.authority.dir,INDEX,true).map_err(|_|Failure::Uncertain)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;step(Step::IndexPublished)?;
  }
  let protocol=serde_json::to_vec(&Protocol{version:2,kind:"coucou-native-consent".into(),history:hash.clone()}).map_err(|_|Failure::Invalid)?;
  if self.authority.protocol!=protocol{self.current(&current)?;let temp=format!(".restore-protocol-{}",nonce().map_err(|_|Failure::Unavailable)?);self.authority.dir.write_new(&temp,&protocol).map_err(|_|Failure::Unavailable)?;self.current(&current)?;self.authority.dir.rename(&temp,&self.authority.dir,".protocol.json",false).map_err(|_|Failure::Uncertain)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;step(Step::ProtocolPublished)?;}
  current()?;let reopened=Authority::existing(&self.authority.dir.path)?;reopened.fence()?;
  Ok(serde_json::json!({"status":"historicalAuthorityQuarantined","schemaVersion":1,"historyHash":hash,"historicalDecisions":self.body.decisions.len(),"historicalGoals":self.body.goal_ids.len(),"oldExecutionAvailable":false,"ownerAbsenceProven":false,"postSnapshotStateUnknown":true,"runtimeActivationAvailable":false}))
 }
}
pub(crate)fn historical_view(path:&Path)->Result<serde_json::Value>{
 let a=Authority::existing(path)?;let _lock=a.lock_shared()?;let b=a.restoration.as_ref().ok_or(Failure::Missing)?;
 verify_historical_grants(&a)?;
 a.fence()?;Ok(serde_json::json!({"schemaVersion":1,"historyHash":b.hash,"source":b.body.source,"historyBodyVersion":b.body.schema_version,"historyEpochs":b.body.priors.len()+1,"priorHistoryHashes":b.body.priors.iter().map(|r|&r.hash).collect::<Vec<_>>(),"grants":b.body.grants,"historicalDecisions":b.body.decisions.len(),"historicalGoals":b.body.goal_ids.len(),"oldExecutionAvailable":false,"ownerAbsenceProven":false,"postSnapshotStateUnknown":true,"runtimeActivationAvailable":false}))
}
fn verify_historical_grants(a:&Authority)->Result<Vec<(String,Stamp)>>{
 let b=a.restoration.as_ref().ok_or(Failure::Missing)?;
 let mut stamps=vec![];for entry in &b.body.grants{let(raw,stamp)=a.dir.bytes(&entry.file,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if model::hash(&raw)!=entry.file_hash{return Err(Failure::Changed);}if entry.signature_valid{a.grant_file_data(&entry.file)?;}stamps.push((entry.file.clone(),stamp));}
 a.fence()?;Ok(stamps)
}
#[path="authority_restoration_history.rs"]mod history_snapshot;
pub(in crate::pet_runtime::authority_store)use history_snapshot::goal_snapshot;
pub(in crate::pet_runtime::authority_store)use history_snapshot::captured_goal_snapshot;
pub(in crate::pet_runtime)fn matches_historical_goal(a:&Authority,goal:&Record)->Result<bool>{
 let Some(b)=&a.restoration else{return Ok(false);};if !b.body.goal_ids.contains(&goal.id){return Ok(false);}let r=b.body.goals.iter().find(|r|r.id==goal.id).ok_or(Failure::Missing)?;if r.pet_id!=goal.spec.owner.pet_id||r.hash!=model::fingerprint(goal).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}a.fence()?;Ok(true)
}
/// Exact native metadata delta for the inactive data publisher. No secret key
/// or grant body is returned; the archive's remaining bytes must still match.
pub(crate)fn publication_delta(path:&Path)->Result<Vec<(String,Vec<u8>)>>{
 let a=Authority::restoration_base(path)?;let Some(boundary)=marker(&a)?else{return Ok(vec![]);};
 if protocol_hash(&a.protocol)?.is_some_and(|h|h!=boundary.hash){return Err(Failure::Changed);}
 let mut result=vec![];for name in a.dir.names().map_err(|_|Failure::Capacity)?{
  if name==INDEX||name==".protocol.json"||name.strip_prefix(".restore-protocol-").is_some_and(token)||name.strip_prefix(".restore-index-").is_some_and(token){let(raw,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;result.push((name,raw));}
 }a.restoration_raw_fence()?;Ok(result)
}
#[cfg(test)]#[path="authority_restoration_tests.rs"]mod tests;

#[cfg(test)]pub(crate)fn fixture_empty_authority(path:&std::path::Path)->std::result::Result<(),HistoryFailure>{Authority::ensure(path).map(|_|())}

#[path="authority_restoration_epochs.rs"]mod epochs;
pub(crate)use epochs::{EpochReview,epoch_publication_delta};

#[path="authority_restoration_independent.rs"]mod independent;
pub(crate)use independent::{IndependentScope,IndependentStep,IndependentKeyState};

#[cfg(test)]pub(crate)use independent::{fixture_copied_goal_catalog,fixture_prepared_seed_denies_old_ids};

#[cfg(test)]pub(crate)type IndependentFixtureRecord=crate::pet_runtime::model::Record;

pub(crate)use independent::verify_retained_independent;

#[cfg(test)]pub(crate)use independent::fixture_fresh_independent_goal;
#[cfg(test)]pub(in crate::pet_runtime)use tests::future_tests::prepared_fixture as fixture_restored_future_review;
