//! Typed cleanup of a positively unpublished local message delta. The existing
//! signed Grant records retirement before any exact staged inode is removed.
use super::*;
use crate::pet_runtime::{message_retirement as retire,messages::{Scope,Ledger as Messages},admission::QueuedAdmission};
use std::collections::BTreeMap;
fn normalized(mut body:Grant)->model::Result<String>{
 // These are known control transitions of the same immutable admission/lease.
 // Budget, message data, owner, spec and every future field remain compared.
 body.schema_version=0;body.revision.clear();body.updated_at=body.approved_at;body.phase=GrantPhase::Claimed;body.stop_reason=None;body.closed_at=None;body.recovery=None;
 model::fingerprint(&body)
}
fn unpublished_delta(image:&Grant,canonical:&Grant,message:&str)->Result<Option<Scope>>{
 if image.retirement.is_some(){return Ok(None);}
 let mut base=image.clone();let Some(messages)=base.messages.as_mut()else{return Ok(None);};
 let scope=if let Some(index)=messages.entries.iter().position(|r|r.envelope().message_id==message){
  if image.phase!=GrantPhase::Claimed||image.stop_reason.is_some()||image.closed_at.is_some()||image.recovery.is_some(){return Ok(None);}
  let entry=messages.entries.remove(index);let budget=base.budget_usage.as_mut().ok_or(Failure::Invalid)?;let n=budget.attempts.len();budget.attempts.retain(|a|a.id!=entry.attempt_id);if n-budget.attempts.len()!=1{return Err(Failure::Invalid);}if budget.attempts.is_empty(){base.budget_usage=None;}entry.scope
 }else if let Some(index)=messages.retired.iter().position(|r|r.scope.envelope.message_id==message){messages.retired.remove(index).scope}else{return Ok(None);};
 if canonical.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().message_id==message)){return Err(Failure::Changed);}
 if messages.entries.is_empty()&&messages.retired.is_empty(){base.messages=None;}
 if normalized(base).map_err(|_|Failure::Invalid)?!=normalized(canonical.clone()).map_err(|_|Failure::Invalid)?{return Err(Failure::RecoveryRequired);}
 Ok(Some(scope))
}
pub(in crate::pet_runtime)enum Prepared{Review(retire::Review),Retired(retire::Receipt)}
impl Authority{
 fn message_intent_images(&self,goal:&Record,row:&QueuedAdmission,canonical:&Grant,message:&str,own:Option<&retire::Reference>)->Result<(Scope,Vec<retire::Artifact>,BTreeMap<String,Stamp>)>{
  let mut scope=None;let mut files=vec![];let mut stamps=BTreeMap::new();let mut scanned=0;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if self.historical_file(&name)||!name.strip_prefix(".grant-").is_some_and(model::token){continue;}scanned+=1;if scanned>128{return Err(Failure::Capacity);}
   if self.rule_record_data(&name)?.is_some(){continue;}let(image,stamp)=self.grant_file(&name)?;if image.decision_id!=canonical.decision_id{continue;}
   // An original still-valid confirmation may retry its own failed publication.
   // Its exact signed reduction record is not a new source artifact/approval.
   // Keep it until the canonical record is durable; cleanup then removes it.
   if let Some(reference)=own.filter(|r|image.messages.as_ref().is_some_and(|m|m.retired.iter().any(|v|v==*r))){
    let mut expected=canonical.clone();expected.messages.get_or_insert_with(Messages::empty).retired.push(reference.clone());
    if normalized(image).map_err(|_|Failure::Invalid)?!=normalized(expected).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}continue;
   }
   let Some(candidate)=unpublished_delta(&image,canonical,message)?else{continue;};
   if !candidate.belongs(&row.spec,image.lease.as_ref().ok_or(Failure::Invalid)?)||!candidate.matches_proposal(goal)||scope.as_ref().is_some_and(|old|old!=&candidate){return Err(Failure::Changed);}
   let(bytes,current)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;if current!=stamp{return Err(Failure::Changed);}if files.len()>=32{return Err(Failure::Capacity);}
   scope=Some(candidate);files.push(retire::Artifact{name:name.clone(),hash:model::hash(&bytes),stamp_hash:model::fingerprint(&stamp).map_err(|_|Failure::Invalid)?});stamps.insert(name,stamp);
  }
  files.sort_by(|a,b|a.name.cmp(&b.name));self.fence()?;Ok((scope.ok_or(Failure::Missing)?,files,stamps))
 }
 pub(in crate::pet_runtime)fn prepare_message_retirement(self,goal:Record,decision:&str,message:&str,request:String,now:u64)->Result<Prepared>{
  if !model::token(message)||!model::key(&request)||goal.validate().is_err()||now<goal.updated_at{return Err(Failure::Invalid);}
  let (body,scope,files,stamps)={
   let _guard=self.lock_shared()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
   if let Some(reference)=body.messages.as_ref().and_then(|m|m.retired.iter().find(|r|r.scope.envelope.message_id==message)).cloned(){drop(_guard);return self.ack_message_retirement(&goal,&reference).map(Prepared::Retired);}
   if body.recovery.is_some()||self.accounting_frozen_unlocked(&goal,&row.spec)?{return Err(Failure::RecoveryRequired);}
   let(scope,files,stamps)=self.message_intent_images(&goal,row,&body,message,None)?;(body,scope,files,stamps)
  };
  Ok(Prepared::Review(retire::Review{authority:self,goal,body,scope,files,stamps,request,id:nonce().map_err(|_|Failure::Unavailable)?,prepared_at:now,expires:retire::deadline()}))
 }
 fn message_retirement_cleanup(&self,canonical:&Grant,reference:&retire::Reference)->Result<retire::Receipt>{
  // Original reviewed files may already be gone after an uncertain cleanup ACK.
  // Replaced or changed files are preserved, even when the native name matches.
  for artifact in &reference.files{
   let Some((raw,stamp))=self.dir.bytes(&artifact.name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?else{continue;};
   if model::hash(&raw)!=artifact.hash||model::fingerprint(&stamp).map_err(|_|Failure::Invalid)?!=artifact.stamp_hash{return Err(Failure::Changed);}let(image,current)=self.grant_file(&artifact.name)?;if current!=stamp||image.decision_id!=canonical.decision_id{return Err(Failure::Changed);}
   self.fence()?;self.dir.remove_exact(&artifact.name,&stamp).map_err(|_|Failure::Uncertain)?;
  }
  // A before-rename failure of retirement itself has a signed image of this
  // exact already recorded reference. Remove only equivalent own data images.
  let mut scanned=0;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if self.historical_file(&name)||!name.strip_prefix(".grant-").is_some_and(model::token){continue;}scanned+=1;if scanned>128{return Err(Failure::Capacity);}
   if self.rule_record_data(&name)?.is_some(){continue;}let(image,stamp)=self.grant_file(&name)?;
   if image.decision_id==canonical.decision_id&&image.messages.as_ref().is_some_and(|m|m.retired.iter().any(|r|r==reference)){
    if normalized(image).map_err(|_|Failure::Invalid)?!=normalized(canonical.clone()).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}
    self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;
   }
  }
  self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;Ok(retire::Receipt{reference:reference.clone(),cleanup_complete:true,message_delivered:false,budget_refund:false,execution_authorized:false})
 }
 pub(in crate::pet_runtime)fn commit_message_retirement(&self,goal:&Record,source:&Grant,stamps:&BTreeMap<String,Stamp>,reference:&retire::Reference,now:u64,human:impl Fn()->bool,started:&mut bool)->Result<retire::Receipt>{
  let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==source.decision_id).ok_or(Failure::Missing)?;let mut body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if body.messages.as_ref().is_some_and(|m|m.retired.iter().any(|r|r==reference)){self.stabilize(&body)?;return self.message_retirement_cleanup(&body,reference);}
  if !human()||now<reference.retired_at||now-reference.prepared_at>=120_000||now<body.updated_at||body.recovery.is_some()||model::fingerprint(&body).map_err(|_|Failure::Invalid)?!=reference.source_grant_hash||model::fingerprint(source).map_err(|_|Failure::Invalid)?!=reference.source_grant_hash||self.accounting_frozen_unlocked(goal,&row.spec)?{return Err(Failure::Changed);}
  let(scope,files,current)=self.message_intent_images(goal,row,&body,&reference.scope.envelope.message_id,Some(reference))?;if scope!=reference.scope||files!=reference.files||&current!=stamps{return Err(Failure::Changed);}
  let messages=body.messages.get_or_insert_with(Messages::empty);if messages.entries.len()+messages.retired.len()>=crate::pet_runtime::messages::MAX_MESSAGES{return Err(Failure::Capacity);}messages.retired.push(reference.clone());body.updated_at=now;body.revision=reference.id.clone();
  *started=true;self.publish_guarded(body,false,||if human(){Ok(())}else{Err(Failure::Revoked)})?;
  let recorded=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;self.message_retirement_cleanup(&recorded,reference)
 }
 pub(in crate::pet_runtime)fn ack_message_retirement(&self,goal:&Record,reference:&retire::Reference)->Result<retire::Receipt>{
  let _guard=self.lock()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==reference.scope.envelope.origin.decision_id).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if !body.messages.as_ref().is_some_and(|m|m.retired.iter().any(|r|r==reference)){return Err(Failure::Changed);}self.stabilize(&body)?;self.message_retirement_cleanup(&body,reference)
 }
 /// A recovery closure would freeze the original grant. Retire message intents
 /// first; do not seal an unresolved stage then try to rewrite frozen history.
 pub(in crate::pet_runtime::authority_store)fn unretired_message_intents(&self,body:&Grant)->Result<bool>{
  let mut scanned=0;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if self.historical_file(&name)||!name.strip_prefix(".grant-").is_some_and(model::token){continue;}scanned+=1;if scanned>128{return Err(Failure::Capacity);}if self.rule_record_data(&name)?.is_some(){continue;}let(image,_)=self.grant_file(&name)?;
   if image.decision_id!=body.decision_id{continue;}
   if image.messages.as_ref().is_some_and(|m|m.entries.iter().any(|entry|!body.messages.as_ref().is_some_and(|c|c.entries.iter().any(|r|r.envelope().message_id==entry.envelope().message_id)||c.retired.iter().any(|r|r.scope.envelope.message_id==entry.envelope().message_id)))){return Ok(true);}
  }Ok(false)
 }
}

#[derive(serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct IntentItem{pub message:crate::pet_runtime::messages::Envelope,pub retirement_recorded:bool}
#[derive(serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct IntentInventory{pub goal_revision:String,pub decision_id:String,pub entries:Vec<IntentItem>,pub pending_artifacts:usize,pub scan_complete:bool}
impl Authority{
 pub(in crate::pet_runtime)fn message_retirement_at(&self,goal:&Record,decision:&str,message:&str)->Result<Option<retire::Reference>>{
  let _guard=self.lock_shared()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  Ok(body.messages.and_then(|m|m.retired.into_iter().find(|r|r.scope.envelope.message_id==message)))
 }
 pub(in crate::pet_runtime)fn pending_messages(&self,goal:&Record,decision:&str)->Result<IntentInventory>{
  let _guard=self.lock_shared()?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;let mut ids=std::collections::BTreeSet::new();let mut count=0;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if self.historical_file(&name)||!name.strip_prefix(".grant-").is_some_and(model::token){continue;}count+=1;if count>128{return Err(Failure::Capacity);}if self.rule_record_data(&name)?.is_some(){continue;}let(image,_)=self.grant_file(&name)?;
   if image.decision_id!=body.decision_id{continue;}if let Some(m)=image.messages{for scope in m.entries.into_iter().map(|r|r.scope).chain(m.retired.into_iter().map(|r|r.scope)){
    if body.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().message_id==scope.envelope.message_id)){continue;}ids.insert(scope.envelope.message_id);if ids.len()>16{return Err(Failure::Capacity);}
   }}
  }
  let mut entries=vec![];for id in ids{
   if let Some(reference)=body.messages.as_ref().and_then(|m|m.retired.iter().find(|r|r.scope.envelope.message_id==id)){entries.push(IntentItem{message:reference.scope.envelope.clone(),retirement_recorded:true});}
   else{let(scope,_,_)=self.message_intent_images(goal,row,&body,&id,None)?;entries.push(IntentItem{message:scope.envelope,retirement_recorded:false});}
  }self.fence()?;Ok(IntentInventory{goal_revision:goal.revision.clone(),decision_id:decision.into(),entries,pending_artifacts:count,scan_complete:true})
 }
}
