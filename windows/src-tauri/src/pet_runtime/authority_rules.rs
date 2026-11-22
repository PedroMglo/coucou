//! Rule records share Authority/key/catalogue lock/atomic writer with goal grants.
use super::*;
use crate::pet_runtime::rules::{RuleGrant,RuleEnvelope,RulePhase,RuleReservation,RuleAuthorityStatus,Rule,Missed};
fn message(body:&RuleGrant)->Result<Vec<u8>>{let mut m=if body.schema_version==12{b"coucou-native-rule-recurrence-grant-v12\0".to_vec()}else if body.schema_version==11{b"coucou-native-rule-recurrence-grant-v11\0".to_vec()}else if body.schema_version==10{b"coucou-native-rule-recurrence-grant-v10\0".to_vec()}else if body.schema_version==9{b"coucou-native-rule-recurrence-grant-v9\0".to_vec()}else if body.schema_version==7{b"coucou-native-rule-recurrence-grant-v7\0".to_vec()}else if body.schema_version==6{b"coucou-native-rule-recurrence-grant-v6\0".to_vec()}else if body.schema_version==5{b"coucou-native-rule-recurrence-grant-v5\0".to_vec()}else if body.schema_version==4{b"coucou-native-rule-recurrence-grant-v4\0".to_vec()}else if body.schema_version==3{b"coucou-native-rule-recurrence-grant-v3\0".to_vec()}else if body.schema_version==2{b"coucou-native-rule-recurrence-grant-v2\0".to_vec()}else{b"coucou-native-rule-recurrence-grant-v1\0".to_vec()};m.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(m)}
impl Authority{
 /// Commit the existing occurrence partition, then durably record its one
 /// native read before returning the retained owner to a calendar worker.
 /// A crash between the two publications has spent no read and may complete
 /// only that same allocation. A published read is never renewed by expiry.
 pub(in crate::pet_runtime)fn begin_owned_rule_calendar(&self,decision:&str,policy_hash:&str,index:usize,now:u64,current:impl Fn()->bool)->Result<OwnedCalendarRead>{
  use crate::pet_runtime::rules::CalendarRead;
  let allocation=self.reserve_rule_occurrence_kind(decision,policy_hash,index,now,&current,true)?;
  let _lock=self.lock()?;let name=format!("{decision}.json");if self.historical_file(&name)||self.rule_has_pending_unlocked(decision)?{return Err(Failure::RecoveryRequired);}
  let(mut g,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if g.phase!=RulePhase::Active||now<g.updated_at||now>=g.policy.valid_until||!current(){return Err(Failure::Changed);}
  self.rule_execution_lineage_unlocked(&g,false)?;
  let slot=g.reservations.iter().position(|r|r.occurrence_id==allocation.occurrence_id).ok_or(Failure::Missing)?;
  let r=&g.reservations[slot];if r.calendar_read.is_some(){return Err(Failure::RecoveryRequired);}if now>=r.deadline_at{return Err(Failure::Expired);}
  let seconds=g.policy.capture_seconds().ok_or(Failure::Invalid)?;
  let id=nonce().map_err(|_|Failure::Unavailable)?;let scope=CalendarRead::scope(&g.policy,r).map_err(|_|Failure::Invalid)?;
  let owner=crate::pet_runtime::native_owner::OwnedOperation::capture(self,&id,&scope,now).map_err(|_|Failure::Unavailable)?;
  let read=CalendarRead{schema_version:1,id,scope_hash:scope,owner:owner.record().clone(),prepared_at:now,expires_at:now.saturating_add(u64::from(seconds)*1000).min(r.deadline_at),finished_at:None,outcome:None};
  g.reservations[slot].calendar_read=Some(read.clone());g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
  self.publish_rule(&g,false,||if current(){Ok(())}else{Err(Failure::Changed)})?;
  Ok(OwnedCalendarRead{decision:decision.into(),occurrence:allocation.occurrence_id,read,owner})
 }
 pub(in crate::pet_runtime)fn rule_calendar_read_current(&self,owned:&OwnedCalendarRead,now:u64)->Result<()>{
  if owned.owner.current(self).is_err(){return Err(Failure::Changed);}
  let _lock=self.lock_shared()?;let name=format!("{}.json",owned.decision);if self.historical_file(&name)||self.rule_has_pending_unlocked(&owned.decision)?{return Err(Failure::RecoveryRequired);}
  let(g,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if g.phase!=RulePhase::Active||now<g.updated_at||now<owned.read.prepared_at||now>=owned.read.expires_at||now>=g.policy.valid_until{return Err(Failure::Revoked);}
  self.rule_execution_lineage_unlocked(&g,false)?;
  let r=g.reservations.iter().find(|r|r.occurrence_id==owned.occurrence).ok_or(Failure::Missing)?;
  if r.calendar_read.as_ref()!=Some(&owned.read)||owned.read.finished_at.is_some(){return Err(Failure::Changed);}self.fence()
 }
 /// Completion preserves an observed stop even after revocation; success still
 /// needs the original live owner, deadline and current policy. This grants no
 /// child admission/model call and never releases or replenishes the partition.
 pub(in crate::pet_runtime)fn finish_rule_calendar_read(&self,owned:&OwnedCalendarRead,outcome:crate::pet_runtime::rules::CalendarReadOutcome,now:u64,current:impl Fn()->bool)->Result<()>{
  use crate::pet_runtime::rules::CalendarReadOutcome;
  if owned.owner.current(self).is_err(){return Err(Failure::Changed);}let _lock=self.lock()?;
  let name=format!("{}.json",owned.decision);if self.historical_file(&name)||self.rule_has_pending_unlocked(&owned.decision)?{return Err(Failure::RecoveryRequired);}
  let(mut g,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;if now<g.updated_at{return Err(Failure::Changed);}
  let r=g.reservations.iter_mut().find(|r|r.occurrence_id==owned.occurrence).ok_or(Failure::Missing)?;
  let old=r.calendar_read.as_mut().filter(|read|read.id==owned.read.id).ok_or(Failure::Changed)?;
  if old.finished_at.is_some(){return if old.outcome.as_ref()==Some(&outcome){self.stabilize_rule(&g)}else{Err(Failure::Changed)};}
  if old!=&owned.read{return Err(Failure::Changed);}
  let captured=matches!(outcome,CalendarReadOutcome::Captured{..});
  if captured&&(g.phase!=RulePhase::Active||now>=owned.read.expires_at||!current()){return Err(Failure::Changed);}
  old.finished_at=Some(now);old.outcome=Some(outcome);g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
  self.publish_rule(&g,false,||if !captured||current(){Ok(())}else{Err(Failure::Changed)})
 }
}

pub(in crate::pet_runtime)struct OwnedCalendarRead{
 decision:String,occurrence:String,read:crate::pet_runtime::rules::CalendarRead,owner:crate::pet_runtime::native_owner::OwnedOperation,
}
impl OwnedCalendarRead{
 pub(in crate::pet_runtime)fn read(&self)->&crate::pet_runtime::rules::CalendarRead{&self.read}
 pub(in crate::pet_runtime)fn current(&self,a:&Authority,now:u64)->bool{a.rule_calendar_read_current(self,now).is_ok()}
}

impl Authority{
 /// Recognize only the typed discriminator, then verify every byte and HMAC.
 /// Historical DATA uses the same reader; callers separately enforce its fence.
 pub(in crate::pet_runtime::authority_store)fn rule_record_data(&self,name:&str)->Result<Option<(RuleGrant,Stamp)>>{
  let(raw,stamp)=self.dir.bytes(name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
  let value:serde_json::Value=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
  if value["body"]["kind"]!="ruleRecurrence"{return Ok(None);}
  let e:RuleEnvelope=serde_json::from_value(value).map_err(|_|Failure::Invalid)?;if name.strip_suffix(".json").filter(|s|model::token(s)).is_some_and(|id|id!=e.body.decision_id){return Err(Failure::Invalid);}if !e.body.valid(){return Err(Failure::Invalid);}
  hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;Ok(Some((e.body,stamp)))
 }
 pub(in crate::pet_runtime::authority_store)fn rule_inventory_unlocked(&self)->Result<Vec<(String,RuleGrant,bool)>>{
  let mut out=vec![];for name in self.dir.names().map_err(|_|Failure::Capacity)?{
   if matches!(name.as_str(),".key.json"|".catalog.lock"|".protocol.json"){continue;}
   if self.historical_file(&name)&&!name.ends_with(".json"){continue;}
   // Immutable history/index files are catalogue metadata, not grant bodies.
   if name==".restored-history.json"||name.starts_with(".restored-history-prior-"){continue;}
   let canonical=name.strip_suffix(".json").filter(|s|model::token(s));let staged=name.strip_prefix(".grant-").is_some_and(model::token);
   if canonical.is_none()&&!staged{return Err(Failure::RecoveryRequired);}
   if let Some((body,_))=self.rule_record_data(&name)?{if canonical.is_some_and(|id|id!=body.decision_id){return Err(Failure::Invalid);}let canonical=canonical.is_some();out.push((name,body,canonical));}
   else{self.grant_file_data(&name)?;}
  }self.fence()?;Ok(out)
 }
 pub(in crate::pet_runtime::authority_store)fn publish_rule(&self,body:&RuleGrant,new:bool,current:impl Fn()->Result<()>)->Result<()>{
  if !body.valid(){return Err(Failure::Invalid);}self.restoration_current(&body.decision_id,"")?;
  let tag=encode(hmac::sign(&self.key,&message(body)?).as_ref());let bytes=serde_json::to_vec(&RuleEnvelope{body:body.clone(),tag}).map_err(|_|Failure::Invalid)?;self.publish_record_bytes(&body.decision_id,&bytes,new,current)
 }
 pub(in crate::pet_runtime::authority_store)fn stabilize_rule(&self,body:&RuleGrant)->Result<()>{self.dir.open_file(&format!("{}.json",body.decision_id),libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()}
 /// Candidate is built once from the exact native review/choice, never renderer DATA.
 pub(in crate::pet_runtime)fn issue_rule(&self,candidate:&RuleGrant,now:u64,current:impl Fn()->bool)->Result<RuleGrant>{
  if candidate.revision_authority.is_some()||candidate.succession.is_some()||!candidate.valid()||!candidate.reservations.is_empty()||candidate.phase!=RulePhase::Active||now<candidate.updated_at||now>=candidate.policy.valid_until||!current(){return Err(Failure::Changed);}
  let _lock=self.lock()?;self.restoration_current(&candidate.decision_id,"")?;if !current(){return Err(Failure::Changed);}
  let mut pending=None;
  for(name,old,canonical)in self.rule_inventory_unlocked()?{
   if old.policy.rule.id!=candidate.policy.rule.id&&old.decision_id!=candidate.decision_id{continue;}
   if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
   if canonical{if old.decision_id!=candidate.decision_id||old.policy!=candidate.policy||old.approved_at!=candidate.approved_at{return Err(Failure::Changed);}self.stabilize_rule(&old)?;return Ok(old);}
   if &old!=candidate||pending.is_some(){return Err(Failure::RecoveryRequired);}pending=Some(name);
  }
  if let Some(name)=pending{
   if !current(){return Err(Failure::Changed);}self.fence()?;self.dir.rename(&name,&self.dir,&format!("{}.json",candidate.decision_id),true).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;
  }else{if self.dir.names().map_err(|_|Failure::Capacity)?.len()>=4093{return Err(Failure::Capacity);}self.publish_rule(candidate,true,||if current(){Ok(())}else{Err(Failure::Changed)})?;}
  Ok(candidate.clone())
 }
 pub(in crate::pet_runtime)fn rule_status(&self,rule:&Rule,current:bool)->Result<RuleAuthorityStatus>{
  let _lock=self.lock_shared()?;let Some(family)=self.rule_family_unlocked(&rule.owner.pet_id,&rule.id)?else{return Ok(RuleAuthorityStatus::missing(rule));};
  if family.chain.is_empty(){let mut status=RuleAuthorityStatus::missing(rule);status.pending_artifacts=family.pending.len();status.remaining_occurrences=None;status.phase="reconciliationRequired";return Ok(status);}
  family.status(current&&family.head()?.policy.rule==*rule,crate::assistant_memory::now())
 }
 pub(in crate::pet_runtime)fn rule_decision_for(&self,pet:&str,rule:&str)->Result<Option<String>>{
  let _lock=self.lock_shared()?;match self.rule_family_unlocked(pet,rule)?{None=>Ok(None),Some(f)=>f.selected_decision()}
 }
 pub(in crate::pet_runtime)fn revoke_rule(&self,pet:&str,rule:&str,decision:&str,now:u64)->Result<RuleGrant>{
  if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock()?;let family=self.rule_family_unlocked(pet,rule)?.ok_or(Failure::Missing)?;if family.selected_decision()?.as_deref()!=Some(decision){return Err(Failure::Changed);}let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
  let existing=match self.rule_record_data(&name){Ok(Some((b,_)))=>Some(b),Err(Failure::Missing)=>None,Ok(None)=>return Err(Failure::Invalid),Err(e)=>return Err(e)};
  let new=existing.is_none();let mut body=if let Some(b)=existing{b}else{let mut staged=self.rule_inventory_unlocked()?.into_iter().filter(|(_,b,canonical)|!*canonical&&b.decision_id==decision);let(_,b,_)=staged.next().ok_or(Failure::Missing)?;if staged.next().is_some(){return Err(Failure::RecoveryRequired);}b};if body.policy.rule.id!=rule||body.policy.rule.owner.pet_id!=pet||now<body.updated_at{return Err(Failure::Changed);}
  if body.phase==RulePhase::Revoked{self.stabilize_rule(&body)?;return Ok(body);}body.phase=RulePhase::Revoked;body.revoked_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish_rule(&body,new,||Ok(()))?;Ok(body)
 }
 /// Internal parent commitment only. This DATA cannot claim/run a goal. Actual
 /// occurrence admission will bind the same reservation before any child effect.
 pub(in crate::pet_runtime)fn reserve_rule_occurrence(&self,decision:&str,policy_hash:&str,index:usize,now:u64,current:impl Fn()->bool)->Result<RuleReservation>{
  self.reserve_rule_occurrence_kind(decision,policy_hash,index,now,current,false)
 }
 fn reserve_rule_occurrence_kind(&self,decision:&str,policy_hash:&str,index:usize,now:u64,current:impl Fn()->bool,calendar:bool)->Result<RuleReservation>{
  if !model::token(decision)||!current(){return Err(Failure::Changed);}let _lock=self.lock()?;let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
  let(mut body,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if body.policy.selected_data()!=calendar{return Err(Failure::Unavailable);} // The ordinary model route never allocates selected-data work.
  let family=self.rule_execution_lineage_unlocked(&body,true)?;let prior=family.prior();let future_after=family.future_after();
  // Complete only the matching original metadata intent. A different request
  // cannot overwrite or silently discard an unresolved parent commitment.
  let pending:Vec<_>=self.rule_inventory_unlocked()?.into_iter().filter(|(_,b,c)|!*c&&b.decision_id==decision).collect();
  if !pending.is_empty(){
   if pending.len()!=1||body.phase!=RulePhase::Active{return Err(Failure::RecoveryRequired);}let(name,staged,_)=&pending[0];
   let wanted=body.policy.occurrence_id(index).ok_or(Failure::Invalid)?;
   if staged.revision_authority!=body.revision_authority||staged.succession!=body.succession||staged.intent_retirements!=body.intent_retirements||staged.decision_id!=body.decision_id||staged.revoked_at!=body.revoked_at||staged.phase!=body.phase||staged.policy!=body.policy||staged.approved_at!=body.approved_at||staged.reservations.len()!=body.reservations.len()+1||!staged.reservations.starts_with(&body.reservations)||staged.reservations.last().is_none_or(|r|r.occurrence_id!=wanted)||staged.updated_at<body.updated_at||now<staged.updated_at||!current(){return Err(Failure::RecoveryRequired);}
   let staged_ids=staged.reservations.iter().chain(&prior).map(|r|&r.occurrence_id).collect::<std::collections::BTreeSet<_>>();if staged_ids.len()!=staged.reservations.len()+prior.len()||staged_ids.len()>usize::from(body.policy.rule.definition.limits.max_occurrences)||staged.reservations.iter().any(|r|r.at<=future_after){return Err(Failure::Capacity);}
   self.fence()?;self.dir.rename(name,&self.dir,&format!("{decision}.json"),false).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;body=staged.clone();
  }
  if body.phase!=RulePhase::Active||now>=body.policy.valid_until{return Err(Failure::Revoked);}if now<body.updated_at||body.policy.hash().map_err(|_|Failure::Invalid)?.as_str()!=policy_hash||!current(){return Err(Failure::Changed);}
  let policy=&body.policy;let slot=policy.schedule.slots.get(index).ok_or(Failure::Invalid)?;let at=slot.at.ok_or(Failure::NotDue)?;let id=policy.occurrence_id(index).ok_or(Failure::Invalid)?;
  if at<=future_after{return Err(Failure::Revoked);}if now<at{return Err(Failure::NotDue);}if let Some(old)=body.reservations.iter().find(|r|r.occurrence_id==id){self.stabilize_rule(&body)?;return Ok(old.clone());}
  if prior.iter().any(|r|r.occurrence_id==id){return Err(Failure::Revoked);}
  let limit=&policy.rule.definition.limits;if body.reservations.len()+prior.len()>=usize::from(limit.max_occurrences){return Err(Failure::Capacity);}
  let deadline=at.checked_add(u64::from(limit.occurrence_timeout_seconds)*1000).ok_or(Failure::Invalid)?;if now>=deadline{return Err(Failure::Expired);}
  let grace=u64::from(policy.rule.definition.grace_seconds)*1000;
  match policy.rule.definition.missed{
   Missed::Coalesce=>if policy.schedule.slots.iter().enumerate().filter_map(|(i,s)|s.at.filter(|a|*a<=now).map(|_|i)).last()!=Some(index){return Err(Failure::NotDue);},
   Missed::Skip=>if now.saturating_sub(at)>grace{return Err(Failure::NotDue);},
   Missed::Decision=>if policy.schedule.slots.iter().enumerate().any(|(i,s)|s.at.is_some_and(|a|a<=now&&now-a>grace)&&policy.occurrence_id(i).is_some_and(|id|!body.reservations.iter().chain(&prior).any(|r|r.occurrence_id==id))){return Err(Failure::NotDue);},
  }
  let allocation=RuleReservation{notification:None,calendar_read:None,readiness:vec![],admission:None,occurrence_id:id,slot_index:index.try_into().map_err(|_|Failure::Invalid)?,at,deadline_at:deadline,reserved_at:now,model_calls:1,tool_calls:0,output_bytes:limit.occurrence_output_bytes};body.reservations.push(allocation.clone());body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish_rule(&body,false,||if current(){Ok(())}else{Err(Failure::Changed)})?;Ok(allocation)
 }
}

#[cfg(test)]#[path="authority_rules_tests.rs"]mod tests;

/// A signed staged child binding can only complete its original metadata
/// publication. It never replaces a different reservation or releases balance.
fn same_rule_admission_intent(base:&RuleGrant,staged:&RuleGrant)->bool{
 if base.revision_authority!=staged.revision_authority||base.succession!=staged.succession||base.intent_retirements!=staged.intent_retirements||base.policy!=staged.policy||base.decision_id!=staged.decision_id||base.phase!=staged.phase||base.approved_at!=staged.approved_at||base.revoked_at!=staged.revoked_at||staged.updated_at<base.updated_at||base.reservations.len()!=staged.reservations.len(){return false;}
 let mut changes=0;for(a,b)in base.reservations.iter().zip(&staged.reservations){let mut plain=b.clone();plain.admission=a.admission.clone();if &plain!=a{return false;}if a.admission!=b.admission{if a.admission.is_some()||b.admission.is_none(){return false;}changes+=1;}}
 changes==1&&staged.valid()
}
impl Authority{
 /// Read authenticated DATA for native materialization/recovery, never a permit.
 /// Resolve execution output constraints only from the authenticated original
 /// parent/reference, never from a provider or mutable renderer setting.
 pub(in crate::pet_runtime)fn rule_calendar_priority_count(&self,s:&AdmissionSpec)->Result<Option<u8>>{
  let reference=s.rule_occurrence.as_ref().ok_or(Failure::Invalid)?;let g=self.rule_grant(&reference.rule_decision_id)?;
  if s.schema_version!=30||g.phase!=RulePhase::Active||g.policy.hash().map_err(|_|Failure::Invalid)?!=reference.policy_hash||!g.reservations.iter().any(|r|g.child_matches(r,s)){return Err(Failure::Changed);}Ok(g.policy.calendar_priority_count)
 }
 pub(in crate::pet_runtime)fn rule_grant(&self,decision:&str)->Result<RuleGrant>{
  if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock_shared()?;let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}let(base,_)=self.rule_record_data(&name)?.ok_or(Failure::Invalid)?;
  let pending=self.rule_inventory_unlocked()?.into_iter().filter(|(_,b,c)|!*c&&b.decision_id==decision).collect::<Vec<_>>();
  match pending.as_slice(){[]=>Ok(base),[(_,staged,_)]if same_rule_admission_intent(&base,staged)=>Ok(staged.clone()),[(_,staged,_)]if staged.revision_authority==base.revision_authority&&staged.succession==base.succession&&staged.intent_retirements==base.intent_retirements&&staged.policy==base.policy&&staged.phase==base.phase&&staged.approved_at==base.approved_at&&staged.reservations.len()==base.reservations.len()+1&&staged.reservations.starts_with(&base.reservations)=>Ok(base),_=>Err(Failure::RecoveryRequired)}
 }
 pub(in crate::pet_runtime)fn bind_rule_admission(&self,s:&AdmissionSpec,now:u64,current:impl Fn()->bool)->Result<()>{
  let reference=s.rule_occurrence.as_ref().ok_or(Failure::Invalid)?;if !reference.for_spec(s)||!current(){return Err(Failure::Changed);}let _lock=self.lock()?;let name=format!("{}.json",reference.rule_decision_id);if self.historical_file(&name){return Err(Failure::RecoveryRequired);}let(mut body,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if body.phase!=RulePhase::Active||now<body.updated_at||now>=body.policy.valid_until||!current(){return Err(Failure::Revoked);}self.rule_execution_lineage_unlocked(&body,true)?;
  let index=body.reservations.iter().position(|r|r.occurrence_id==reference.occurrence_id).ok_or(Failure::Missing)?;if !body.child_matches(&body.reservations[index],s){return Err(Failure::Changed);}
  let snapshot=crate::pet_runtime::rules::RuleAdmission::capture(s).map_err(|_|Failure::Invalid)?;
  let pending=self.rule_inventory_unlocked()?.into_iter().filter(|(_,b,c)|!*c&&b.decision_id==body.decision_id).collect::<Vec<_>>();
  if let Some(old)=&body.reservations[index].admission{return if old==&snapshot&&pending.is_empty(){self.stabilize_rule(&body)}else{Err(Failure::RecoveryRequired)};}
  if let[(staged_name,staged,_)]=pending.as_slice(){
   if !same_rule_admission_intent(&body,staged)||staged.reservations[index].admission.as_ref()!=Some(&snapshot)||now<staged.updated_at||!current(){return Err(Failure::RecoveryRequired);}
   self.fence()?;self.dir.rename(staged_name,&self.dir,&name,false).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;
  }else if pending.is_empty(){
   body.reservations[index].admission=Some(snapshot);body.schema_version=body.required_version();body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish_rule(&body,false,||if current(){Ok(())}else{Err(Failure::Changed)})?;
  }else{return Err(Failure::RecoveryRequired);}
  self.rule_admission_current_unlocked(s,now)
 }
 pub(in crate::pet_runtime::authority_store)fn rule_admission_current_unlocked(&self,s:&AdmissionSpec,now:u64)->Result<()>{
  let Some(reference)=s.rule_occurrence.as_ref()else{return if matches!(s.schema_version,29|30|31|32){Err(Failure::Invalid)}else{Ok(())};};
  if !reference.for_spec(s){return Err(Failure::Invalid);}let name=format!("{}.json",reference.rule_decision_id);if self.historical_file(&name){return Err(Failure::RecoveryRequired);}let(body,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if body.phase!=RulePhase::Active||now<body.updated_at||now>=body.policy.valid_until||now<reference.at||now>=reference.deadline_at{return Err(Failure::Revoked);}
  let r=body.reservations.iter().find(|r|r.occurrence_id==reference.occurrence_id).ok_or(Failure::Changed)?;
  self.rule_execution_lineage_unlocked(&body,false)?;if r.readiness.iter().any(|p|p.finished_at.is_none()){return Err(Failure::RecoveryRequired);}if self.rule_has_pending_unlocked(&reference.rule_decision_id)?{return Err(Failure::RecoveryRequired);}if !body.child_matches(r,s)||r.admission.as_ref().is_none_or(|a|!a.matches(s)){return Err(Failure::Changed);}self.fence()
 }
}

impl Authority{
 pub(in crate::pet_runtime::authority_store)fn rule_has_pending_unlocked(&self,decision:&str)->Result<bool>{
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if let Some((r,_))=self.rule_record_data(&name)?{if r.decision_id==decision{return Ok(true);}}else{self.grant_file_data(&name)?;}}Ok(false)
 }
}
