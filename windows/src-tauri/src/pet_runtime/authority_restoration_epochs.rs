//! An explicitly reviewed new epoch over the same Authority/key/ledger.
//! Previous signed indexes are immutable data, never additional allowance.
use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum EpochStep{PriorStaged,PriorPublished,IndexPublished,ProtocolPublished}
pub(crate)struct EpochReview{authority:Authority,goals_path:PathBuf,body:Body,raw:Vec<u8>,hash:String,parent_raw:Vec<u8>,parent_hash:String,staged:Vec<FileRef>}
fn prior_staging(a:&Authority,parent:&[u8])->Result<Vec<FileRef>>{
 let mut refs=vec![];
 for name in a.dir.names().map_err(|_|Failure::Capacity)?{
  if !name.strip_prefix(".restore-prior-").is_some_and(token){continue;}
  let(raw,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
  if !parent.starts_with(&raw){return Err(Failure::RecoveryRequired);}
  refs.push(FileRef{name,hash:model::hash(&raw),bytes:raw.len()});
  if refs.len()>16{return Err(Failure::Capacity);}
 }
 refs.sort_by(|a,b|a.name.cmp(&b.name));Ok(refs)
}
fn protocol_current_or_parent(a:&Authority,b:&Boundary)->Result<()>{
 let p:Protocol=serde_json::from_slice(&a.protocol).map_err(|_|Failure::RecoveryRequired)?;
 if p.history==b.hash&&p.version==b.body.schema_version+1{return Ok(());}
 if b.body.schema_version==2&&b.body.parent.as_ref()==Some(&p.history){let name=prior_name(&p.history);let(raw,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;let parent=decode_history(a,&raw)?;if model::hash(&raw)==p.history&&p.version==parent.schema_version+1{return Ok(());}}
 Err(Failure::RecoveryRequired)
}
fn epoch_raw(a:&Authority,body:&Body)->Result<Vec<u8>>{let raw=serde_json::to_vec(&Envelope{body:body.clone(),tag:encode(hmac::sign(&a.key,&message(body)?).as_ref())}).map_err(|_|Failure::Invalid)?;if raw.len()>MAX_INDEX{return Err(Failure::Capacity);}Ok(raw)}
impl EpochReview{
 pub(crate)fn capture(path:&Path,goals_path:&Path,source:&str,current:impl Fn()->Result<()>)->Result<Self>{
  current()?;if !hash_valid(source){return Err(Failure::Invalid);}let authority=Authority::restoration_base(path)?;authority.restoration_raw_fence()?;let b=epoch_marker(&authority)?.ok_or(Failure::Missing)?;protocol_current_or_parent(&authority,&b)?;
  let(body,parent_raw,parent_hash)=if b.body.schema_version==2&&b.body.source==source{
   if b.body.files!=files(&authority)?||b.body.goals!=goals(goals_path)?{return Err(Failure::Changed);}let ph=b.body.parent.clone().ok_or(Failure::Invalid)?;let(pr,_)=authority.dir.bytes(&prior_name(&ph),MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;(b.body,pr,ph)
  }else{
   if b.body.source==source{return Err(Failure::Changed);}
   // Starting another epoch requires the old boundary to be fully published.
   let p:Protocol=serde_json::from_slice(&authority.protocol).map_err(|_|Failure::RecoveryRequired)?;if p.history!=b.hash||p.version!=b.body.schema_version+1{return Err(Failure::RecoveryRequired);}
   let(old_raw,_)=authority.dir.bytes(INDEX,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;let mut body=capture(&authority,goals_path,source)?;body.schema_version=2;body.parent=Some(b.hash.clone());body.priors=b.body.priors.clone();body.priors.push(IndexRef{hash:b.hash.clone(),bytes:old_raw.len()});body.priors.sort_by(|a,b|a.hash.cmp(&b.hash));
   body.decisions.extend(b.body.decisions);body.decisions.sort();body.decisions.dedup();body.goal_ids.extend(b.body.goal_ids);body.goal_ids.sort();body.goal_ids.dedup();if !valid(&body){return Err(Failure::Capacity);}(body,old_raw,b.hash)
  };
  let raw=epoch_raw(&authority,&body)?;let hash=model::hash(&raw);let staged=prior_staging(&authority,&parent_raw)?;let r=Self{authority,goals_path:goals_path.into(),body,raw,hash,parent_raw,parent_hash,staged};r.current(&current)?;Ok(r)
 }
 fn current(&self,current:impl Fn()->Result<()>)->Result<()>{
  self.current_with_stage(current,None)
 }
 fn current_with_stage(&self,current:impl Fn()->Result<()>,own:Option<&str>)->Result<()>{
  current()?;self.authority.restoration_raw_fence()?;if files(&self.authority)?!=self.body.files||goals(&self.goals_path)?!=self.body.goals{return Err(Failure::Changed);}let b=epoch_marker(&self.authority)?.ok_or(Failure::Missing)?;
  if b.hash!=self.parent_hash&&b.hash!=self.hash{return Err(Failure::Changed);}if b.hash==self.hash&&b.body!=self.body{return Err(Failure::Changed);}protocol_current_or_parent(&self.authority,&b)?;
  for name in self.authority.dir.names().map_err(|_|Failure::Capacity)?{if let Some(h)=prior_hash(&name){let r=self.body.priors.iter().find(|r|r.hash==h).ok_or(Failure::Changed)?;let(raw,_)=self.authority.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if raw.len()!=r.bytes||model::hash(&raw)!=r.hash{return Err(Failure::Changed);}decode_history(&self.authority,&raw)?;}}
  self.staging_current(own)
 }
 fn staging_current(&self,own:Option<&str>)->Result<()>{
  let refs=prior_staging(&self.authority,&self.parent_raw)?;
  for r in &refs{if !self.staged.contains(r)&&!(own==Some(r.name.as_str())&&r.hash==self.parent_hash&&r.bytes==self.parent_raw.len()){return Err(Failure::Changed);}}
  if self.staged.iter().any(|r|!refs.contains(r)){
   let(raw,_)=self.authority.dir.bytes(&prior_name(&self.parent_hash),MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
   if raw!=self.parent_raw{return Err(Failure::Changed);}
  }
  Ok(())
 }
 pub(crate)fn checked_delta(&self)->Result<Vec<(String,Vec<u8>)>>{
  let a=Authority::restoration_base(&self.authority.dir.path)?;if a.key_id!=self.authority.key_id||a.stamp!=self.authority.stamp||a.lock_stamp!=self.authority.lock_stamp{return Err(Failure::Changed);}if files(&a)?!=self.body.files||goals(&self.goals_path)?!=self.body.goals{return Err(Failure::Changed);}let b=epoch_marker(&a)?.ok_or(Failure::Missing)?;
  if b.hash!=self.parent_hash&&(b.hash!=self.hash||b.body!=self.body){return Err(Failure::Changed);}for name in a.dir.names().map_err(|_|Failure::Capacity)?{if let Some(h)=prior_hash(&name){let r=self.body.priors.iter().find(|r|r.hash==h).ok_or(Failure::Changed)?;let(raw,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if raw.len()!=r.bytes||model::hash(&raw)!=r.hash{return Err(Failure::Changed);}}}epoch_publication_delta(&a.dir.path)
 }
 pub(crate)fn transaction_fingerprint(&self)->Result<String>{model::fingerprint(&("coucou-restored-next-authority-review-v2",self.authority.dir.path.as_os_str().as_bytes(),self.goals_path.as_os_str().as_bytes(),&self.body,&self.parent_hash,&self.hash)).map_err(|_|Failure::Invalid)}
 pub(crate)fn fingerprint(&self)->Result<String>{let tx=self.transaction_fingerprint()?;if self.staged.is_empty(){Ok(tx)}else{model::fingerprint(&("coucou-restored-prior-staging-review-v1",tx,&self.staged)).map_err(|_|Failure::Invalid)}}
 pub(crate)fn preview(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"status":"nextEpochAuthorityReview","schemaVersion":2,"fingerprint":self.fingerprint()?,"source":self.body.source,"historyHash":self.hash,"parentHistoryHash":self.parent_hash,"historyEpochs":self.body.priors.len()+1,"historicalDecisions":self.body.decisions.len(),"historicalGoals":self.body.goal_ids.len(),"grants":self.body.grants,"reviewedRedundantPriorStaging":self.staged,"oldExecutionAvailable":false,"ownerAbsenceProven":false,"postSnapshotStateUnknown":true,"runtimeActivationAvailable":false,"allowanceRenewed":false}))}
 pub(crate)fn apply(&self,approved:&str,current:impl Fn()->Result<()>)->Result<serde_json::Value>{self.apply_using(approved,current,|_|Ok(()))}
 fn apply_using(&self,approved:&str,current:impl Fn()->Result<()>,mut step:impl FnMut(EpochStep)->Result<()>)->Result<serde_json::Value>{
  if approved!=self.fingerprint()?{return Err(Failure::Changed);}self.current(&current)?;let file=self.authority.dir.open_file(".catalog.lock",libc::O_RDWR).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Failure::Busy);}let _lock=Guard(file);self.current(&current)?;
  let name=prior_name(&self.parent_hash);if let Some((raw,_))=self.authority.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?{if raw!=self.parent_raw{return Err(Failure::Changed);}}else{
   let tmp=format!(".restore-prior-{}",nonce().map_err(|_|Failure::Unavailable)?);self.authority.dir.write_new(&tmp,&self.parent_raw).map_err(|_|Failure::Unavailable)?;step(EpochStep::PriorStaged)?;self.current_with_stage(&current,Some(&tmp))?;self.authority.dir.rename(&tmp,&self.authority.dir,&name,true).map_err(|_|Failure::Uncertain)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;
  }step(EpochStep::PriorPublished)?;self.current(&current)?;
  // Only reviewed redundant prefixes are removed, after the complete signed
  // parent is durably retained. Grant/usage/user files never enter this list.
  for r in &self.staged{self.current(&current)?;if let Some((raw,stamp))=self.authority.dir.bytes(&r.name,MAX_INDEX).map_err(|_|Failure::Unsafe)?{
   if raw.len()!=r.bytes||model::hash(&raw)!=r.hash||!self.parent_raw.starts_with(&raw){return Err(Failure::Changed);}
   self.authority.dir.remove_exact(&r.name,&stamp).map_err(|_|Failure::Changed)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;
  }}self.current(&current)?;
  let b=epoch_marker(&self.authority)?.ok_or(Failure::Missing)?;if b.hash!=self.hash{
   if b.hash!=self.parent_hash{return Err(Failure::Changed);}let tmp=format!(".restore-index-{}",nonce().map_err(|_|Failure::Unavailable)?);self.authority.dir.write_new(&tmp,&self.raw).map_err(|_|Failure::Unavailable)?;self.current(&current)?;self.authority.dir.rename(&tmp,&self.authority.dir,INDEX,false).map_err(|_|Failure::Uncertain)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;
  }step(EpochStep::IndexPublished)?;self.current(&current)?;
  let protocol=serde_json::to_vec(&Protocol{version:3,kind:"coucou-native-consent".into(),history:self.hash.clone()}).map_err(|_|Failure::Invalid)?;
  if self.authority.protocol!=protocol{let tmp=format!(".restore-protocol-{}",nonce().map_err(|_|Failure::Unavailable)?);self.authority.dir.write_new(&tmp,&protocol).map_err(|_|Failure::Unavailable)?;self.current(&current)?;self.authority.dir.rename(&tmp,&self.authority.dir,".protocol.json",false).map_err(|_|Failure::Uncertain)?;self.authority.dir.durable().map_err(|_|Failure::Uncertain)?;}step(EpochStep::ProtocolPublished)?;current()?;
  let reopened=Authority::existing(&self.authority.dir.path)?;reopened.fence()?;Ok(serde_json::json!({"status":"nextEpochAuthorityQuarantined","schemaVersion":2,"historyHash":self.hash,"parentHistoryHash":self.parent_hash,"historyEpochs":self.body.priors.len()+1,"historicalDecisions":self.body.decisions.len(),"historicalGoals":self.body.goal_ids.len(),"oldExecutionAvailable":false,"ownerAbsenceProven":false,"postSnapshotStateUnknown":true,"runtimeActivationAvailable":false,"allowanceRenewed":false}))
 }
}
/// Known immutable provenance/staging bytes only. The enclosing native data
/// controller verifies them against its held original source and exact review.
pub(crate)fn epoch_publication_delta(path:&Path)->Result<Vec<(String,Vec<u8>)>>{
 let a=Authority::restoration_base(path)?;let b=epoch_marker(&a)?.ok_or(Failure::Missing)?;protocol_current_or_parent(&a,&b)?;let mut out=vec![];
 for name in a.dir.names().map_err(|_|Failure::Capacity)?{if name==INDEX||name==".protocol.json"||prior_hash(&name).is_some()||name.strip_prefix(".restore-protocol-").is_some_and(token)||name.strip_prefix(".restore-index-").is_some_and(token)||name.strip_prefix(".restore-prior-").is_some_and(token){let(raw,_)=a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;out.push((name,raw));}}
 a.restoration_raw_fence()?;Ok(out)
}
#[cfg(test)]#[path="authority_restoration_epoch_tests.rs"]mod tests;
