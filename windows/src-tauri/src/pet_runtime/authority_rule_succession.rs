//! Native reviewed successor, same original allocation/deadline, new decision.
use super::*;
use crate::pet_runtime::{rules::{RuleGrant,RulePhase,RuleSuccession,RulePolicy,RuleRevisionMode,RuleRevisionAuthority},runtime_owner::Profile};
fn fp<T:Serialize>(value:&T)->Result<String>{model::fingerprint(value).map_err(|_|Failure::Invalid)}
/// Authenticated source, never deserializable or supplied by a provider.
pub(in crate::pet_runtime)struct RuleSuccessionSource{
 chain:Vec<(RuleGrant,Stamp)>,pending:Option<(String,RuleGrant,Stamp)>,profile:Profile,hash:String,allocation_start:usize,revision_policy:Option<(RulePolicy,RuleRevisionMode)>,
}
impl RuleSuccessionSource{
 fn head(&self)->&RuleGrant{&self.chain.last().unwrap().0}
 pub(in crate::pet_runtime)fn rule(&self)->&crate::pet_runtime::rules::Rule{&self.policy().rule}
 pub(in crate::pet_runtime)fn policy(&self)->&crate::pet_runtime::rules::RulePolicy{self.revision_policy.as_ref().map(|(p,_)|p).unwrap_or(&self.head().policy)}
 pub(in crate::pet_runtime)fn preview(&self)->serde_json::Value{
  let h=self.head();let spent=self.chain.iter().skip(self.allocation_start).flat_map(|(g,_)|&g.reservations).collect::<Vec<_>>();
  serde_json::json!({"schemaVersion":1,"sourceHash":self.hash,"ruleId":self.rule().id,"petId":self.rule().owner.pet_id,
   "predecessorDecisionId":h.decision_id,"rootDecisionId":self.chain[0].0.decision_id,"generation":self.chain.len(),
   "recordedChoice":self.pending.is_some(),"originalApprovedAt":self.chain[0].0.approved_at,"originalValidUntil":h.policy.valid_until,
   "committedOccurrences":spent.len(),"remainingOccurrences":usize::from(self.chain[self.allocation_start].0.policy.rule.definition.limits.max_occurrences)-spent.len(),
   "committedOutputBytes":spent.iter().map(|r|r.output_bytes).sum::<u64>(),"unknownReadinessProbes":spent.iter().flat_map(|r|&r.readiness).filter(|p|p.finished_at.is_none()).count(),
   "originalPolicy":h.policy,"signedChain":self.chain.iter().map(|(g,_)|g).collect::<Vec<_>>(),
   "stagedChoice":self.pending.as_ref().map(|(_,g,_)|g),"oldReferencesRemainRevoked":true,
   "freshBudgetGranted":false,"deadlineRenewed":false,"workerEndProven":false,"executionAuthorityGranted":false})
 }
 pub(in crate::pet_runtime)fn candidate(&self,decision:&str,now:u64)->Result<RuleGrant>{
  if let Some((_,original,_))=&self.pending{return Ok(original.clone());}
  let head=self.head();
  if let Some((policy,mode))=&self.revision_policy{
   if !model::token(decision)||self.chain.iter().any(|(g,_)|g.decision_id==decision)||self.chain.iter().any(|(g,_)|now<g.updated_at)||now<policy.prepared_at||now>=policy.valid_until||!policy.schedule.slots.iter().any(|s|s.at.is_some_and(|at|at>now)){return Err(Failure::Changed);}
   let g=RuleGrant{schema_version:7,kind:head.kind,decision_id:decision.into(),revision:decision.into(),policy:policy.clone(),phase:RulePhase::Active,approved_at:now,updated_at:now,revoked_at:None,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:Some(RuleRevisionAuthority{schema_version:1,mode:*mode,root_decision_id:self.chain[0].0.decision_id.clone(),predecessor_decision_id:head.decision_id.clone(),generation:self.chain.len().try_into().map_err(|_|Failure::Capacity)?,source_hash:self.hash.clone(),predecessor_commitments_hash:head.commitments_hash().map_err(|_|Failure::Invalid)?,future_after:now})};if !g.valid(){return Err(Failure::Invalid);}return Ok(g);
  }
  if !model::token(decision)||self.chain.iter().any(|(g,_)|g.decision_id==decision)||self.chain.iter().any(|(g,_)|now<g.updated_at)||now>=head.policy.valid_until{return Err(Failure::Changed);}
  let g=RuleGrant{schema_version:6,kind:head.kind,decision_id:decision.into(),revision:decision.into(),policy:head.policy.clone(),phase:RulePhase::Active,approved_at:now,updated_at:now,revoked_at:None,reservations:vec![],intent_retirements:vec![],revision_authority:None,succession:Some(RuleSuccession{schema_version:1,root_decision_id:self.chain[0].0.decision_id.clone(),predecessor_decision_id:head.decision_id.clone(),generation:self.chain.len().try_into().map_err(|_|Failure::Capacity)?,source_hash:self.hash.clone(),predecessor_commitments_hash:head.commitments_hash().map_err(|_|Failure::Invalid)?})};
  if !g.valid(){return Err(Failure::Invalid);}Ok(g)
 }
 pub(in crate::pet_runtime)fn revision_preview(&self)->serde_json::Value{
  let policy=self.policy();let mode=self.revision_policy.as_ref().map(|(_,m)|*m);let allocation=&self.chain[self.allocation_start].0;let used=self.chain.iter().skip(self.allocation_start).map(|(g,_)|g.reservations.len()).sum::<usize>();let old_remaining=usize::from(allocation.policy.rule.definition.limits.max_occurrences)-used;let renew=mode==Some(RuleRevisionMode::RenewFiniteAllocation);
  serde_json::json!({"schemaVersion":1,"sourceHash":self.hash,"ruleId":self.rule().id,"petId":self.rule().owner.pet_id,"mode":mode,"proposedPolicy":policy,"previousPolicy":self.head().policy,"predecessorDecisionId":self.head().decision_id,"rootDecisionId":self.chain[0].0.decision_id,"generation":self.chain.len(),"recordedChoice":self.pending.is_some(),"allocationRootDecisionId":allocation.decision_id,"originalAllocationValidUntil":allocation.policy.valid_until,"currentAllocationCommittedOccurrences":used,"currentAllocationRemainingOccurrences":old_remaining,"proposedAvailableOccurrences":if renew{usize::from(policy.rule.definition.limits.max_occurrences)}else{old_remaining},"oldUnusedAuthorityRetired":renew,"oldUnusedOccurrencesRetired":if renew{old_remaining}else{0},"cumulativeCommittedOccurrences":self.chain.iter().map(|(g,_)|g.reservations.len()).sum::<usize>(),"eligibleFutureSlotsAtPreparation":policy.schedule.slots.iter().filter(|s|s.at.is_some_and(|at|at>policy.prepared_at)).count(),"futureOnlyAtApproval":true,"oldReferencesRemainRevoked":true,"workerEndProven":false,"executionAuthorityGranted":false,"signedChain":self.chain.iter().map(|(g,_)|g).collect::<Vec<_>>(),"stagedChoice":self.pending.as_ref().map(|(_,g,_)|g)})
 }
 fn matches(&self,g:&RuleGrant)->bool{
  if let Some((_,original,_))=&self.pending{return g==original;}
  self.candidate(&g.decision_id,g.approved_at).is_ok_and(|expected|expected==*g)
 }
}
impl Authority{
 pub(in crate::pet_runtime)fn prepare_rule_succession(&self,pet:&str,rule:&str,now:u64)->Result<RuleSuccessionSource>{
  let _lock=self.lock_shared()?;let family=self.rule_family_unlocked(pet,rule)?.ok_or(Failure::Missing)?;
  if family.historical{return Err(Failure::RecoveryRequired);}let h=family.head()?;
  if h.phase!=RulePhase::Revoked{return Err(Failure::Revoked);}if family.chain.iter().any(|g|now<g.updated_at)||now>=h.policy.valid_until{return Err(Failure::Expired);}
  if family.chain.len()>=64{return Err(Failure::Capacity);}if family.used()>=usize::from(family.allocation()?.policy.rule.definition.limits.max_occurrences){return Err(Failure::Capacity);}
  let pending=match(family.pending_successor.as_ref(),family.pending.as_slice()){
   (None,[])=>None,
   (Some(g),[(name,p)])if g==p&&g.phase==RulePhase::Active&&g.succession.is_some()&&g.revision_authority.is_none()=>{let(body,stamp)=self.rule_record_data(name)?.ok_or(Failure::Missing)?;if body!=*p{return Err(Failure::Changed);}Some((name.clone(),body,stamp))},
   _=>return Err(Failure::RecoveryRequired),
  };
  self.capture_rule_source(family,pending,None)
 }
 /// Native7 owns its distinct mode and exact proposed policy. Pending7 is ACKed
 /// only under that mode;6 cannot be used to approve a fresh finite allocation.
 pub(in crate::pet_runtime)fn prepare_rule_revision(&self,policy:&RulePolicy,mode:RuleRevisionMode,now:u64)->Result<RuleSuccessionSource>{
  if !policy.valid()||policy.prepared_at>now{return Err(Failure::Invalid);}let _lock=self.lock_shared()?;let family=self.rule_family_unlocked(&policy.rule.owner.pet_id,&policy.rule.id)?.ok_or(Failure::Missing)?;let head=family.head()?;
  if family.historical||head.phase!=RulePhase::Revoked{return Err(Failure::RecoveryRequired);}if family.chain.iter().any(|g|now<g.updated_at)||family.chain.len()>=64{return Err(Failure::Changed);}
  let mut selected=policy.clone();let pending=match(family.pending_successor.as_ref(),family.pending.as_slice()){
   (None,[])=>None,
   (Some(g),[(name,p)])if g==p&&g.phase==RulePhase::Active&&g.revision_authority.as_ref().is_some_and(|r|r.mode==mode)=>{
    let mut normalized=policy.clone();normalized.prepared_at=g.policy.prepared_at;if normalized!=g.policy{return Err(Failure::Changed);}let(body,stamp)=self.rule_record_data(name)?.ok_or(Failure::Missing)?;if body!=*g{return Err(Failure::Changed);}selected=g.policy.clone();Some((name.clone(),body,stamp))
   },_=>return Err(Failure::RecoveryRequired),
  };
  if now>=selected.valid_until||pending.is_none()&&!selected.schedule.slots.iter().any(|s|s.at.is_some_and(|at|at>now)){return Err(Failure::Expired);}
  let rule=&selected.rule;let old=&head.policy.rule;if rule.owner!=old.owner||rule.created_at!=old.created_at||rule.revision<old.revision||rule.updated_at<old.updated_at||rule.revision==old.revision&&rule!=old{return Err(Failure::Changed);}
  if mode==RuleRevisionMode::CarryRemaining{let allocation=family.allocation()?;if selected.rule.definition.limits!=allocation.policy.rule.definition.limits||selected.valid_until>allocation.policy.valid_until||now>=allocation.policy.valid_until{return Err(Failure::Changed);}if family.used()>=usize::from(allocation.policy.rule.definition.limits.max_occurrences){return Err(Failure::Capacity);}}
  self.capture_rule_source(family,pending,Some((selected,mode)))
 }
 fn capture_rule_source(&self,family:super::rule_lineage::RuleFamily,pending:Option<(String,RuleGrant,Stamp)>,revision_policy:Option<(RulePolicy,RuleRevisionMode)>)->Result<RuleSuccessionSource>{
  let allocation_start=family.allocation_start();let mut chain=vec![];let mut bytes=pending.as_ref().map(|(_,g,_)|serde_json::to_vec(g).map(|b|b.len())).transpose().map_err(|_|Failure::Invalid)?.unwrap_or(0)+revision_policy.as_ref().map(|p|serde_json::to_vec(p).map(|b|b.len())).transpose().map_err(|_|Failure::Invalid)?.unwrap_or(0);
  for g in family.chain{let(body,stamp)=self.rule_record_data(&format!("{}.json",g.decision_id))?.ok_or(Failure::Missing)?;if body!=g{return Err(Failure::Changed);}bytes=bytes.checked_add(serde_json::to_vec(&body).map_err(|_|Failure::Invalid)?.len()).ok_or(Failure::Capacity)?;if bytes>1024*1024{return Err(Failure::Capacity);}chain.push((body,stamp));}
  let(_,profile)=self.owner_context()?;let hash=if let Some(proposed)=&revision_policy{fp(&("coucou-native-rule-revision-source-v1",&chain,&profile,proposed))?}else{fp(&("coucou-native-rule-succession-source-v1",&chain,&profile))?};self.fence()?;Ok(RuleSuccessionSource{chain,pending,profile,hash,allocation_start,revision_policy})
 }

 pub(in crate::pet_runtime)fn succeed_rule(&self,source:&RuleSuccessionSource,chosen:&RuleGrant,now:u64,current:impl Fn()->bool)->Result<RuleGrant>{
  if !chosen.valid()||!source.matches(chosen)||now<chosen.updated_at||!current(){return Err(Failure::Changed);}
  let _lock=self.lock()?;let check=||{if !current()||self.owner_context()?.1!=source.profile{return Err(Failure::Changed);}self.fence()};check()?;
  let family=self.rule_family_unlocked(&source.rule().owner.pet_id,&source.rule().id)?.ok_or(Failure::Missing)?;if family.historical{return Err(Failure::RecoveryRequired);}
  // Original publication may have succeeded before its ACK. Never overwrite
  // later reservations, revocation, observations, or an independently approved
  // later successor. A fresh choice cannot match these exact signed headers.
  if let Some(recorded)=family.chain.iter().find(|g|g.decision_id==chosen.decision_id){
   if recorded.policy!=chosen.policy||recorded.approved_at!=chosen.approved_at||recorded.succession!=chosen.succession||recorded.revision_authority!=chosen.revision_authority{return Err(Failure::Changed);}
   check()?;self.stabilize_rule(recorded)?;return Ok(recorded.clone());
  }
  if now>=chosen.policy.valid_until||family.chain.len()!=source.chain.len(){return Err(Failure::Changed);}
  for(g,stamp)in &source.chain{let(actual,actual_stamp)=self.rule_record_data(&format!("{}.json",g.decision_id))?.ok_or(Failure::Missing)?;if actual!=*g||actual_stamp!=*stamp{return Err(Failure::Changed);}}
  if family.head()?.decision_id!=source.head().decision_id||family.head()?.phase!=RulePhase::Revoked{return Err(Failure::Changed);}
  let pending=match family.pending.as_slice(){
   []if source.pending.is_none()=>None,
   [(name,body)]if body==chosen=>{
    let(actual,stamp)=self.rule_record_data(name)?.ok_or(Failure::Missing)?;if actual!=*chosen{return Err(Failure::Changed);}
    if source.pending.as_ref().is_some_and(|(old,_,old_stamp)|old!=name||old_stamp!=&stamp){return Err(Failure::Changed);}Some(name.clone())
   },_=>return Err(Failure::RecoveryRequired),
  };
  if let Some(name)=pending{check()?;self.dir.rename(&name,&self.dir,&format!("{}.json",chosen.decision_id),true).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;}
  else{if self.dir.names().map_err(|_|Failure::Capacity)?.len()>=4093{return Err(Failure::Capacity);}self.publish_rule(chosen,true,check)?;}
  Ok(chosen.clone())
 }
}
#[cfg(test)]#[path="rule_succession_tests.rs"]mod tests;
#[cfg(test)]#[path="rule_revision_tests.rs"]mod revision_tests;
