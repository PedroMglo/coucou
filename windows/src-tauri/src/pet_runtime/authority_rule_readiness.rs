//! Bounded metadata intents in the ORIGINAL rule allocation and Authority writer.
use super::*;
use crate::pet_runtime::rules::{RuleGrant,RulePhase,ReadinessProbe};
impl Authority{
 fn rule_probe_parent(&self,decision:&str)->Result<RuleGrant>{
  if !model::token(decision){return Err(Failure::Invalid);}let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}let(g,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;if !matches!(g.schema_version,3|4|5|6|7)||g.policy.dispatch.is_none(){return Err(Failure::Invalid);}if self.rule_has_pending_unlocked(decision)?{return Err(Failure::RecoveryRequired);}Ok(g)
 }
 #[cfg(test)]pub(in crate::pet_runtime)fn begin_rule_probe(&self,decision:&str,occurrence:&str,now:u64,current:impl Fn()->bool)->Result<ReadinessProbe>{self.begin_probe_with_owner(decision,occurrence,now,current,false).map(|(p,_)|p)}
 pub(in crate::pet_runtime)fn begin_owned_rule_probe(&self,decision:&str,occurrence:&str,now:u64,current:impl Fn()->bool)->Result<OwnedProbe>{
  let(probe,owner)=self.begin_probe_with_owner(decision,occurrence,now,current,true)?;Ok(OwnedProbe{probe,owner:owner.ok_or(Failure::Invalid)?})
 }
 fn begin_probe_with_owner(&self,decision:&str,occurrence:&str,now:u64,current:impl Fn()->bool,owned:bool)->Result<(ReadinessProbe,Option<crate::pet_runtime::native_owner::OwnedOperation>)>{
  if !current(){return Err(Failure::Changed);}let _lock=self.lock()?;let mut g=self.rule_probe_parent(decision)?;if g.phase!=RulePhase::Active||now<g.updated_at||now>=g.policy.valid_until{return Err(Failure::Revoked);}self.rule_execution_lineage_unlocked(&g,false)?;let policy=g.policy.dispatch.as_ref().ok_or(Failure::Invalid)?;
  let index=g.reservations.iter().position(|r|r.occurrence_id==occurrence).ok_or(Failure::Missing)?;let r=&g.reservations[index];if now<r.at||now>=r.deadline_at{return Err(Failure::Expired);}if r.readiness.iter().any(|p|p.finished_at.is_none()){return Err(Failure::RecoveryRequired);}if r.readiness.len()>=usize::from(policy.max_metadata_probes){return Err(Failure::Capacity);}if r.readiness.last().filter(|p|p.observed==Some(false)).and_then(|p|p.finished_at).is_some_and(|at|now<at.saturating_add(u64::from(policy.retry_delay_millis))){return Err(Failure::NotDue);}
  let mut probe=ReadinessProbe{owner:None,recovery:None,id:nonce().map_err(|_|Failure::Unavailable)?,scope_hash:ReadinessProbe::scope(&g.policy,r).map_err(|_|Failure::Invalid)?,prepared_at:now,expires_at:now.saturating_add(u64::from(policy.metadata_timeout_millis)).min(r.deadline_at),finished_at:None,observed:None};let owner=if owned{let owner=crate::pet_runtime::native_owner::OwnedOperation::capture(self,&probe.id,&probe.scope_hash,now).map_err(|_|Failure::Unavailable)?;probe.owner=Some(owner.record().clone());Some(owner)}else{None};g.reservations[index].readiness.push(probe.clone());g.schema_version=g.required_version();g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
  self.publish_rule(&g,false,||if current(){Ok(())}else{Err(Failure::Changed)})?;Ok((probe,owner))
 }
 pub(in crate::pet_runtime)fn rule_probe_current(&self,decision:&str,occurrence:&str,probe:&ReadinessProbe,now:u64)->Result<()>{
  let _lock=self.lock_shared()?;let g=self.rule_probe_parent(decision)?;if g.phase!=RulePhase::Active||now<g.updated_at||now<probe.prepared_at||now>=probe.expires_at||now>=g.policy.valid_until{return Err(Failure::Revoked);}self.rule_execution_lineage_unlocked(&g,false)?;let r=g.reservations.iter().find(|r|r.occurrence_id==occurrence).ok_or(Failure::Changed)?;if !probe.valid(&g,r)||probe.finished_at.is_some()||r.readiness.last()!=Some(probe){return Err(Failure::Changed);}self.fence()
 }
 /// Completion records observation or uncertainty, including after revocation.
 /// It never grants another probe, model call, deadline or resource slot.
 pub(in crate::pet_runtime)fn finish_rule_probe(&self,decision:&str,occurrence:&str,probe:&ReadinessProbe,observed:bool,now:u64)->Result<()>{
  let _lock=self.lock()?;let mut g=self.rule_probe_parent(decision)?;if now<g.updated_at{return Err(Failure::Changed);}let r=g.reservations.iter_mut().find(|r|r.occurrence_id==occurrence).ok_or(Failure::Missing)?;let old=r.readiness.last_mut().filter(|p|p.id==probe.id).ok_or(Failure::Missing)?;
  if old.finished_at.is_some(){return if old.observed==Some(observed){self.stabilize_rule(&g)}else{Err(Failure::Changed)};}if old!=probe{return Err(Failure::Changed);}old.finished_at=Some(now);old.observed=Some(observed);g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish_rule(&g,false,||Ok(()))
 }
}

pub(in crate::pet_runtime)struct OwnedProbe{probe:ReadinessProbe,owner:crate::pet_runtime::native_owner::OwnedOperation}
impl OwnedProbe{
 pub(in crate::pet_runtime)fn probe(&self)->&ReadinessProbe{&self.probe}
 pub(in crate::pet_runtime)fn current(&self,a:&Authority)->bool{self.owner.current(a).is_ok()&&self.probe.owner.as_ref()==Some(self.owner.record())}
}
