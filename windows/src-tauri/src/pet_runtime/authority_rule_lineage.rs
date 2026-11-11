//! One authenticated chain and explicitly approved finite allocations, all in
//! the existing Authority catalogue. No latest-timestamp routing or new ledger.
use super::*;
use crate::pet_runtime::rules::{RuleGrant,RulePhase,RuleReservation,RuleAuthorityStatus,RuleLineageObservation,RuleRevisionMode,RuleExecutionBounds,RuleAllocationObservation};
pub(super)struct RuleFamily{
 pub(super)chain:Vec<RuleGrant>,pub(super)pending:Vec<(String,RuleGrant)>,
 pub(super)pending_successor:Option<RuleGrant>,pub(super)historical:bool,
}
fn allocation_start(chain:&[RuleGrant])->usize{chain.iter().rposition(RuleGrant::starts_allocation).unwrap_or(0)}
fn link(chain:&[RuleGrant],child:&RuleGrant)->Result<()>{
 let root=chain.first().ok_or(Failure::Missing)?;let parent=chain.last().ok_or(Failure::Missing)?;
 let (root_id,parent_id,generation,commitments)=if let Some(r)=&child.revision_authority{
  if parent.policy.rule.id!=child.policy.rule.id||parent.policy.rule.owner!=child.policy.rule.owner||parent.policy.rule.created_at!=child.policy.rule.created_at||child.policy.rule.revision<parent.policy.rule.revision||child.policy.rule.updated_at<parent.policy.rule.updated_at||child.policy.rule.revision==parent.policy.rule.revision&&child.policy.rule!=parent.policy.rule{return Err(Failure::RecoveryRequired);}
  if r.mode==RuleRevisionMode::CarryRemaining{let allocation=&chain[allocation_start(chain)];if child.policy.rule.definition.limits!=allocation.policy.rule.definition.limits||child.policy.valid_until>allocation.policy.valid_until{return Err(Failure::RecoveryRequired);}}
  (&r.root_decision_id,&r.predecessor_decision_id,r.generation,&r.predecessor_commitments_hash)
 }else{let s=child.succession.as_ref().ok_or(Failure::RecoveryRequired)?;if child.policy!=parent.policy{return Err(Failure::RecoveryRequired);}(&s.root_decision_id,&s.predecessor_decision_id,s.generation,&s.predecessor_commitments_hash)};
 if !child.valid()||root_id!=&root.decision_id||parent_id!=&parent.decision_id||usize::from(generation)!=chain.len()||parent.phase!=RulePhase::Revoked||parent.revoked_at.is_none_or(|at|child.approved_at<at)||commitments!=&parent.commitments_hash().map_err(|_|Failure::Invalid)?{return Err(Failure::RecoveryRequired);}Ok(())
}
fn chain(records:Vec<RuleGrant>)->Result<Vec<RuleGrant>>{
 if records.is_empty(){return Ok(vec![]);}if records.len()>64{return Err(Failure::Capacity);}
 let roots=records.iter().filter(|g|g.parent_id().is_none()).collect::<Vec<_>>();if roots.len()!=1{return Err(Failure::RecoveryRequired);}
 let mut result=vec![roots[0].clone()];let mut seen=std::collections::BTreeSet::from([result[0].decision_id.clone()]);
 loop{let last=result.last().ok_or(Failure::Invalid)?;let children=records.iter().filter(|g|g.parent_id()==Some(last.decision_id.as_str())).collect::<Vec<_>>();match children.as_slice(){[]=>break,[child]=>{link(&result,child)?;if !seen.insert(child.decision_id.clone()){return Err(Failure::RecoveryRequired);}result.push((*child).clone());},_=>return Err(Failure::RecoveryRequired)}}
 if result.len()!=records.len(){return Err(Failure::RecoveryRequired);}
 let mut ids=std::collections::BTreeSet::new();let mut instants=std::collections::BTreeSet::new();let mut allocation=0;let(mut count,mut calls,mut bytes)=(0usize,0u16,0u64);let mut future_after=0;
 for(i,g)in result.iter().enumerate(){
  if g.starts_allocation(){allocation=i;count=0;calls=0;bytes=0;}if let Some(r)=&g.revision_authority{future_after=r.future_after;}
  for r in &g.reservations{
   if r.at<=future_after||!ids.insert(&r.occurrence_id)||!instants.insert(r.at){return Err(Failure::RecoveryRequired);}count+=1;calls=calls.checked_add(r.model_calls).ok_or(Failure::Capacity)?;bytes=bytes.checked_add(r.output_bytes).ok_or(Failure::Capacity)?;
  }
  let l=&result[allocation].policy.rule.definition.limits;if count>usize::from(l.max_occurrences)||calls>l.max_model_calls||bytes>l.max_output_bytes{return Err(Failure::RecoveryRequired);}
 }Ok(result)
}
impl RuleFamily{
 pub(super)fn head(&self)->Result<&RuleGrant>{self.chain.last().ok_or(Failure::Missing)}
 pub(super)fn allocation_start(&self)->usize{allocation_start(&self.chain)}
 pub(super)fn allocation(&self)->Result<&RuleGrant>{self.chain.get(self.allocation_start()).ok_or(Failure::Missing)}
 pub(super)fn used(&self)->usize{self.chain.iter().skip(self.allocation_start()).map(|g|g.reservations.len()).sum()}
 pub(super)fn future_after(&self)->u64{self.chain.iter().rev().find_map(|g|g.revision_authority.as_ref().map(|r|r.future_after)).unwrap_or(0)}
 pub(super)fn selected_decision(&self)->Result<Option<String>>{
  if let Some(g)=&self.pending_successor{return Ok(Some(g.decision_id.clone()));}if let Some(g)=self.chain.last(){return Ok(Some(g.decision_id.clone()));}
  let ids=self.pending.iter().map(|(_,g)|g.decision_id.clone()).collect::<std::collections::BTreeSet<_>>();if ids.len()>1{return Err(Failure::RecoveryRequired);}Ok(ids.into_iter().next())
 }
 pub(super)fn prior(&self)->Vec<RuleReservation>{self.chain.iter().skip(self.allocation_start()).take(self.chain.len().saturating_sub(self.allocation_start()+1)).flat_map(|g|g.reservations.clone()).collect()}
 pub(super)fn current(&self,g:&RuleGrant,allow_head_pending:bool)->Result<()>{
  if self.historical||self.pending_successor.is_some(){return Err(Failure::RecoveryRequired);}let head=self.head()?;if head.decision_id!=g.decision_id||head.policy!=g.policy||head.approved_at!=g.approved_at||head.succession!=g.succession||head.revision_authority!=g.revision_authority{return Err(Failure::Revoked);}
  if self.pending.iter().any(|(_,p)|!allow_head_pending||p.decision_id!=g.decision_id){return Err(Failure::RecoveryRequired);}Ok(())
 }
 pub(super)fn status(&self,current:bool,now:u64)->Result<RuleAuthorityStatus>{
  let head=self.head()?;let allocation=self.allocation()?;let prior=self.prior();let mut s=RuleAuthorityStatus::of_at(head,current,self.historical,now);
  s.committed_occurrences=self.chain.iter().map(|g|g.reservations.len()).sum();s.committed_model_calls=self.chain.iter().flat_map(|g|&g.reservations).map(|r|r.model_calls).sum();s.committed_output_bytes=self.chain.iter().flat_map(|g|&g.reservations).map(|r|r.output_bytes).sum();
  let remaining=allocation.policy.rule.definition.limits.max_occurrences-self.used()as u16;s.remaining_occurrences=(!self.historical).then_some(remaining);
  let reference=head.revision_authority.as_ref().map(|r|(&r.root_decision_id,&r.predecessor_decision_id,r.generation)).or_else(||head.succession.as_ref().map(|r|(&r.root_decision_id,&r.predecessor_decision_id,r.generation)));
  if let Some((root,predecessor,generation))=reference{s.lineage=Some(RuleLineageObservation{root_decision_id:root.clone(),predecessor_decision_id:predecessor.clone(),generation,original_approved_at:self.chain[0].approved_at,inherited_occurrences:prior.len(),inherited_output_bytes:prior.iter().map(|r|r.output_bytes).sum(),allocation:self.chain.iter().any(|g|g.revision_authority.is_some()).then_some(RuleAllocationObservation{root_decision_id:allocation.decision_id.clone(),approved_at:allocation.approved_at,valid_until:allocation.policy.valid_until,max_occurrences:allocation.policy.rule.definition.limits.max_occurrences,committed_occurrences:self.used(),remaining_occurrences:remaining,cumulative_committed_occurrences:s.committed_occurrences,cumulative_committed_output_bytes:s.committed_output_bytes,future_after:self.future_after()})});}
  if self.chain.iter().any(|g|now<g.updated_at)&&!matches!(s.phase,"revoked"|"restoredSnapshotOnly"){s.phase="authorityTimeUncertain";s.dispatch_available=false;}
  s.pending_artifacts=self.pending.len();if !self.pending.is_empty(){s.remaining_occurrences=None;s.dispatch_available=false;if !matches!(s.phase,"revoked"|"restoredSnapshotOnly"){s.phase="reconciliationRequired";}}Ok(s)
 }
}
impl Authority{
 pub(super)fn rule_family_unlocked(&self,pet:&str,rule:&str)->Result<Option<RuleFamily>>{
  let(mut records,mut pending)=(vec![],vec![]);let mut historical=false;for(name,g,canonical)in self.rule_inventory_unlocked()?{if g.policy.rule.id!=rule{continue;}if g.policy.rule.owner.pet_id!=pet{return Err(Failure::Changed);}historical|=self.historical_file(&name);if canonical{records.push(g);}else{pending.push((name,g));}}
  if records.is_empty()&&pending.is_empty(){return Ok(None);}let chain=chain(records)?;let mut pending_successor=None;
  for(_,g)in &pending{if let Some(c)=chain.iter().find(|c|c.decision_id==g.decision_id){if c.policy!=g.policy||c.approved_at!=g.approved_at||c.succession!=g.succession||c.revision_authority!=g.revision_authority{return Err(Failure::RecoveryRequired);}}
   else if !chain.is_empty(){if pending_successor.is_some()||!g.reservations.is_empty()||!g.intent_retirements.is_empty()||chain.len()>=64{return Err(Failure::RecoveryRequired);}link(&chain,g)?;pending_successor=Some(g.clone());}
   else if g.parent_id().is_some(){return Err(Failure::RecoveryRequired);}}
  Ok(Some(RuleFamily{chain,pending,pending_successor,historical}))
 }
 pub(super)fn rule_execution_lineage_unlocked(&self,g:&RuleGrant,allow_head_pending:bool)->Result<RuleFamily>{let f=self.rule_family_unlocked(&g.policy.rule.owner.pet_id,&g.policy.rule.id)?.ok_or(Failure::Missing)?;f.current(g,allow_head_pending)?;Ok(f)}
 pub(in crate::pet_runtime)fn rule_execution_bounds(&self,decision:&str)->Result<RuleExecutionBounds>{
  if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock_shared()?;let(g,_)=self.rule_record_data(&format!("{decision}.json"))?.ok_or(Failure::Missing)?;let f=self.rule_execution_lineage_unlocked(&g,true)?;Ok(RuleExecutionBounds{prior:f.prior(),future_after:f.future_after()})
 }
 pub(in crate::pet_runtime)fn rule_prior_allocations(&self,decision:&str)->Result<Vec<RuleReservation>>{Ok(self.rule_execution_bounds(decision)?.prior)}
}
