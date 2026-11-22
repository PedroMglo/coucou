//! Native reviewed-plan facts in the original closed Grant. Signing a data
//! revision issues no worker, inference channel, budget or execution permission.
use super::*;
use crate::pet_runtime::{continuation, native_plan_revisions::{Reference,Seal,Abandoned},plan_publication::{self,Change},store::{NativePlanPublished,NativePlanAbsent}};
use std::sync::Arc;
pub(in crate::pet_runtime)struct NativePlanCertified<'a>{authority:&'a Authority,seal:Seal,human:Option<Arc<dyn Fn()->bool+Send+Sync>>}
impl plan_publication::Certificate for NativePlanCertified<'_>{
 type Change=plan_publication::Native;
 fn change(&self)->Self::Change{Self::Change::new(self.seal.clone())}
 fn current(&self,g:&Record,now:u64)->Result<()>{
  let _guard=self.authority.lock_shared()?;let row=g.admissions.iter().find(|a|a.spec.decision_id==self.seal.reference.decision_id).ok_or(Failure::Missing)?;
  let b=self.authority.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if !b.native_plan_revisions.contains(&self.seal)||self.seal.abandoned.is_some(){return Err(Failure::Changed);}
  self.authority.verify_native_plan_fact(g,&b,&self.seal)?;
  if !self.seal.content_matches(g){
   if !self.seal.reference.source_matches(g)||b.phase!=GrantPhase::Closed||now<self.seal.approved_at||now>=self.seal.expires_at||!self.human.as_ref().is_some_and(|h|h()){return Err(Failure::Revoked);}
   if g.admissions.len()!=self.seal.source_admissions as usize{return Err(Failure::Changed);}
  }
  self.authority.fence()
 }
}
impl Authority{
 pub(in crate::pet_runtime)fn native_plan_pending_control(&self,g:&Record)->Result<bool>{
  let _guard=self.lock_shared()?;if !matches!(g.state,State::Paused|State::RecoveryRequired|State::Cancelled){return Ok(false);}let row=g.admissions.last().ok_or(Failure::Missing)?;let b=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if !(row.spec.deterministic()||row.spec.funded_response())||!matches!(b.phase,GrantPhase::Closed|GrantPhase::Revoked)||b.native_drain.as_ref().is_some_and(|s|!s.target_present(g)){return Ok(false);}
  if let Some(c)=&b.recovery{if !Self::recovery_event_present(g,c)?{return Ok(false);}}
  Ok(b.native_plan_revisions.iter().any(|s|s.abandoned.is_none()&&s.absent_in(g)))
 }
 fn native_plan_candidate(&self,g:&Record,source:&continuation::Reference,r:&Reference,body:&Grant,approved_at:u64)->Result<(Grant,Seal)>{
  let seal=Seal::prepare(g,source,r,body,approved_at).map_err(|_|Failure::Changed)?;let mut target=body.clone();target.native_plan_revisions.push(seal.clone());target.schema_version=target.plan_schema();target.revision=seal.target_grant_revision.clone();target.updated_at=approved_at;
  if !target.valid(){return Err(Failure::Invalid);}let size=serde_json::to_vec(&GrantEnvelope{body:target.clone(),tag:"a".repeat(64)}).map_err(|_|Failure::Invalid)?.len();
  if size.saturating_add(512)>MAX_GRANT_BYTES{return Err(Failure::Capacity);}Ok((target,seal))
 }
 fn result_plan_current_unlocked(&self,g:&Record,r:&Reference,now:u64)->Result<()>{
  if !matches!(r.schema_version,2|3){return Ok(());}let row=g.admissions.last().ok_or(Failure::Missing)?;self.funding_current_unlocked(&row.spec,now)?;
  for p in &r.patches{let a=crate::pet_runtime::derived_result_plan::Arguments::parse(p.after.operation.as_ref().ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;if a.expires_at<=now{return Err(Failure::Expired);}a.verify_unlocked(self,g)?;}self.fence()
 }
 pub(in crate::pet_runtime)fn native_plan_preflight(&self,g:&Record,source:&continuation::Reference,r:&Reference,now:u64)->Result<()>{
  let _guard=self.lock_shared()?;self.continuation_current_unlocked(g,source,None)?;let row=g.admissions.last().ok_or(Failure::Missing)?;let b=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if b.phase!=GrantPhase::Closed||b.native_plan_revisions.len()>=32||!r.source_matches(g){return Err(Failure::Changed);}self.result_plan_current_unlocked(g,r,now)?;self.native_plan_candidate(g,source,r,&b,now)?;self.fence()
 }
 // This reconstructs only the signed PREPARATION'S grant data to check its
 // original hash during withdrawal. It creates no Goal, authority or executor.
 pub(in crate::pet_runtime::authority_store)fn native_plan_staging_source(staged:&Grant,seal:&Seal)->Result<String>{
  if staged.native_plan_revisions.last()!=Some(seal){return Err(Failure::Changed);}let mut source=staged.clone();
  if let Some(a)=&seal.abandoned{
   if staged.revision!=a.target_grant_revision||staged.updated_at!=a.observed_at{return Err(Failure::Changed);}
   if a.introduced{source.native_plan_revisions.pop();}else{source.native_plan_revisions.last_mut().ok_or(Failure::Missing)?.abandoned=None;}
   source.schema_version=source.plan_schema();source.revision=a.source_grant_revision.clone();source.updated_at=a.source_grant_updated_at;
   let hash=model::fingerprint(&source).map_err(|_|Failure::Invalid)?;if hash!=a.source_grant_hash{return Err(Failure::Changed);}return Ok(hash);
  }
  if staged.revision!=seal.target_grant_revision||staged.updated_at!=seal.approved_at{return Err(Failure::Changed);}
  source.native_plan_revisions.pop();source.schema_version=source.plan_schema();source.revision=seal.source_grant_revision.clone();source.updated_at=seal.source_grant_updated_at;
  let hash=model::fingerprint(&source).map_err(|_|Failure::Invalid)?;if hash!=seal.source_grant_hash{return Err(Failure::Changed);}Ok(hash)
 }
 // A signed cancellation may supersede the old source before its preparation
 // was published. Compare only that exact reduction, never usage/effect data.
 pub(in crate::pet_runtime::authority_store)fn native_plan_revoked_origin(body:&Grant,staged:&Grant,seal:&Seal)->bool{
  if body.phase!=GrantPhase::Revoked||body.stop_reason!=Some(crate::pet_runtime::leases::StopReason::Cancel)||seal.abandoned.is_some()||Self::native_plan_staging_source(staged,seal).is_err(){return false;}
  let mut source=staged.clone();source.native_plan_revisions.pop();source.schema_version=source.plan_schema();source.revision=seal.source_grant_revision.clone();source.updated_at=seal.source_grant_updated_at;
  let mut reduced=body.clone();reduced.phase=source.phase;reduced.stop_reason=source.stop_reason;reduced.revision=source.revision.clone();reduced.updated_at=source.updated_at;
  model::fingerprint(&reduced).ok()==model::fingerprint(&source).ok()
 }
 fn retire_native_plan_staging(&self,body:&Grant,seal:&Seal)->Result<()>{
  let mut files=vec![];for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let(staged,stamp)=self.grant_file(&name)?;
   let Some(s)=staged.native_plan_revisions.last().filter(|s|s.reference.id==seal.reference.id)else{continue;};
   let mut active=seal.clone();active.abandoned=None;
   if (s!=&active&&s!=seal)||staged.goal_id!=body.goal_id||staged.decision_id!=body.decision_id||staged.spec_hash!=body.spec_hash||staged.lease!=body.lease
    ||model::fingerprint(&(&staged.budget_usage,&staged.planning_usage,&staged.resource_usage)).map_err(|_|Failure::Invalid)?!=seal.usage_hash{return Err(Failure::RecoveryRequired);}
   let source_hash=Self::native_plan_staging_source(&staged,s)?;
   let known_source=s.abandoned.is_none()&&model::fingerprint(body).map_err(|_|Failure::Invalid)?==source_hash;
   let known_target=body.native_plan_revisions.last()==Some(seal)&&body.native_plan_revisions[..body.native_plan_revisions.len()-1]==staged.native_plan_revisions[..staged.native_plan_revisions.len()-1]
    &&(model::fingerprint(body).map_err(|_|Failure::Invalid)?==model::fingerprint(&staged).map_err(|_|Failure::Invalid)?||seal.abandoned.is_some());
   if !known_source&&!known_target{return Err(Failure::RecoveryRequired);}files.push((name,stamp));if files.len()>64{return Err(Failure::Capacity);}
  }
  for(name,stamp)in files{self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
 }
 pub(in crate::pet_runtime)fn certify_native_plan<'a>(&'a self,g:&Record,source:&continuation::Reference,r:&Reference,approved_at:u64,cache:&mut Option<Seal>,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->Result<NativePlanCertified<'a>>{
  let _guard=self.lock()?;let row=g.admissions.last().ok_or(Failure::Missing)?;
  if row.spec.decision_id!=r.decision_id||!(if matches!(r.schema_version,2|3){row.spec.funded_response()}else{row.spec.deterministic()}){return Err(Failure::Changed);}let mut body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  let seal=if let Some(saved)=body.native_plan_revisions.iter().find(|s|s.reference.id==r.id){
   if saved.reference!=*r||saved.abandoned.is_some()||saved.source_reference_hash!=model::fingerprint(source).map_err(|_|Failure::Invalid)?||cache.as_ref().is_some_and(|s|s!=saved){return Err(Failure::Changed);}
   self.verify_native_plan_fact(g,&body,saved)?;if !saved.content_matches(g)&&(!r.source_matches(g)||body.phase!=GrantPhase::Closed||now>=saved.expires_at||now<saved.approved_at||!human()){return Err(Failure::Revoked);}
   self.stabilize(&body)?;self.retire_native_plan_staging(&body,saved)?;saved.clone()
  }else{
   if !r.source_matches(g)||body.phase!=GrantPhase::Closed||now<approved_at||now>=r.reviewed_at.saturating_add(120_000)||!human(){return Err(Failure::Revoked);}
   if let Some(saved)=cache.as_ref(){if saved.reference!=*r||saved.source_reference_hash!=model::fingerprint(source).map_err(|_|Failure::Invalid)?||saved.source_grant_hash!=model::fingerprint(&body).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}self.retire_native_plan_staging(&body,saved)?;}
   self.continuation_current_unlocked(g,source,None)?;self.result_plan_current_unlocked(g,r,now)?;
   let saved=if let Some(saved)=cache.as_ref(){saved.clone()}else{let(_,saved)=self.native_plan_candidate(g,source,r,&body,approved_at)?;*cache=Some(saved.clone());saved};
   if body.native_plan_revisions.len()>=32{return Err(Failure::Capacity);}body.native_plan_revisions.push(saved.clone());body.schema_version=body.plan_schema();body.revision=saved.target_grant_revision.clone();body.updated_at=saved.approved_at;
   if !human(){return Err(Failure::Revoked);}self.publish_guarded(body.clone(),false,||if human(){self.result_plan_current_unlocked(g,r,crate::assistant_memory::now())}else{Err(Failure::Revoked)})?;self.retire_native_plan_staging(&body,&saved)?;saved
  };
  *cache=Some(seal.clone());Ok(NativePlanCertified{authority:self,seal,human:Some(human)})
 }
 pub(in crate::pet_runtime)fn native_plan_seal(&self,g:&Record,id:&str)->Result<Seal>{
  let _guard=self.lock_shared()?;
  for row in &g.admissions{let b=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;if let Some(seal)=b.native_plan_revisions.iter().find(|s|s.reference.id==id){return Ok(seal.clone());}}
  let row=g.admissions.last().ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  // Cold withdrawal can inspect a positively signed unpublished preparation;
  // it may only record its abandonment, never adopt its plan or authority.
  let body_hash=model::fingerprint(&body).map_err(|_|Failure::Invalid)?;let mut found=None;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let(staged,_)=self.grant_file(&name)?;let Some(s)=staged.native_plan_revisions.last().filter(|s|s.reference.id==id)else{continue;};
   if staged.goal_id!=g.id||staged.decision_id!=body.decision_id||(Self::native_plan_staging_source(&staged,s)?!=body_hash&&!Self::native_plan_revoked_origin(&body,&staged,s))||!s.absent_in(g)||found.as_ref().is_some_and(|old|old!=s){return Err(Failure::Changed);}found=Some(s.clone());}
  found.ok_or(Failure::Missing)
 }
 pub(in crate::pet_runtime)fn native_plan_ack_certificate<'a>(&'a self,g:&Record,seal:&Seal)->Result<NativePlanCertified<'a>>{
  let _guard=self.lock_shared()?;let row=g.admissions.iter().find(|r|r.spec.decision_id==seal.reference.decision_id).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if !body.native_plan_revisions.contains(seal)||!seal.content_matches(g){return Err(Failure::Changed);}self.verify_native_plan_fact(g,&body,seal)?;Ok(NativePlanCertified{authority:self,seal:seal.clone(),human:None})
 }
 pub(in crate::pet_runtime)fn acknowledge_native_plan(&self,published:&NativePlanPublished<'_>)->Result<()>{
  published.with_current(|g,r|{let _guard=self.lock()?;let row=g.admissions.iter().find(|a|a.spec.decision_id==r.decision_id).ok_or(Failure::Missing)?;let b=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
   let s=b.native_plan_revisions.iter().find(|s|s.reference==*r&&s.abandoned.is_none()).ok_or(Failure::Missing)?;self.verify_native_plan_fact(g,&b,s)?;if !s.content_matches(g){return Err(Failure::Changed);}self.stabilize(&b)?;self.retire_native_plan_staging(&b,s)})
 }
 pub(in crate::pet_runtime)fn abandon_native_plan(&self,absent:&NativePlanAbsent<'_>,seal:&Seal,now:u64)->Result<Seal>{
  absent.with_current(|g,r|{let _guard=self.lock()?;if r!=&seal.reference||now<seal.approved_at{return Err(Failure::Changed);}let row=g.admissions.last().ok_or(Failure::Missing)?;let mut body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
   if row.spec.decision_id!=r.decision_id||!matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)||!seal.absent_in(g){return Err(Failure::Changed);}
   let before_hash=model::fingerprint(&body).map_err(|_|Failure::Invalid)?;let goal_hash=model::fingerprint(g).map_err(|_|Failure::Invalid)?;
   if let Some(saved)=body.native_plan_revisions.iter().find(|s|s.reference.id==r.id&&s.abandoned.is_some()){
    if saved.reference!=*r{return Err(Failure::Changed);}self.stabilize(&body)?;self.retire_native_plan_staging(&body,saved)?;return Ok(saved.clone());
   }
   // Reuse an exact signed withdrawal whose canonical ACK was lost. This is
   // authority reduction only; no staged executor/plan is ever adopted.
   let mut withdrawn:Option<Grant>=None;
   for name in self.dir.names().map_err(|_|Failure::Unavailable)?{if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let(staged,_)=self.grant_file(&name)?;let Some(s)=staged.native_plan_revisions.last().filter(|s|s.reference.id==r.id&&s.abandoned.is_some())else{continue;};let a=s.abandoned.as_ref().ok_or(Failure::Missing)?;
    if s.reference!=*r||a.goal_hash!=goal_hash||a.goal_revision!=g.revision||a.goal_sequence!=g.sequence||Self::native_plan_staging_source(&staged,s)?!=before_hash||withdrawn.as_ref().is_some_and(|b|model::fingerprint(b).ok()!=model::fingerprint(&staged).ok()){return Err(Failure::RecoveryRequired);}withdrawn=Some(staged);
   }
   if let Some(target)=withdrawn{let saved=target.native_plan_revisions.last().ok_or(Failure::Missing)?.clone();self.publish(target.clone(),false)?;self.retire_native_plan_staging(&target,&saved)?;return Ok(saved);}
   if now<body.updated_at||now<g.updated_at{return Err(Failure::Expired);}
   let introduced=!body.native_plan_revisions.iter().any(|s|s.reference.id==r.id);
   let mut revoked_origin=false;if introduced&&before_hash!=seal.source_grant_hash{for name in self.dir.names().map_err(|_|Failure::Unavailable)?{if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let(staged,_)=self.grant_file(&name)?;if staged.native_plan_revisions.last()==Some(seal)&&Self::native_plan_revoked_origin(&body,&staged,seal){revoked_origin=true;}}}
   let index=if let Some(index)=body.native_plan_revisions.iter().position(|s|s.reference.id==r.id){index}else{
    if before_hash!=seal.source_grant_hash&&!revoked_origin||body.native_plan_revisions.len()>=32{return Err(Failure::Changed);}body.native_plan_revisions.push(seal.clone());body.native_plan_revisions.len()-1
   };
   let mut saved=body.native_plan_revisions[index].clone();if saved.reference!=*r||saved.source_grant_hash!=seal.source_grant_hash{return Err(Failure::Changed);}
   let target_revision=nonce().map_err(|_|Failure::Unavailable)?;saved.abandoned=Some(Abandoned{goal_revision:g.revision.clone(),goal_sequence:g.sequence,goal_hash,observed_at:now,source_grant_hash:before_hash,source_grant_revision:body.revision.clone(),source_grant_updated_at:body.updated_at,target_grant_revision:target_revision.clone(),introduced});
   body.native_plan_revisions[index]=saved.clone();body.schema_version=body.plan_schema();body.revision=target_revision;body.updated_at=now;self.publish(body.clone(),false)?;self.retire_native_plan_staging(&body,&saved)?;Ok(saved)
  })
 }
 /// Called under the existing recovery/lineage reader guard, no second scan.
 pub(in crate::pet_runtime::authority_store)fn verify_native_plan_fact(&self,g:&Record,b:&Grant,s:&Seal)->Result<bool>{
  let r=&s.reference;let index=g.admissions.iter().position(|a|a.spec.decision_id==b.decision_id).ok_or(Failure::Missing)?;
  if !s.valid_for(b)||r.goal_id!=g.id||r.pet_id!=g.spec.owner.pet_id||s.source_admissions as usize!=index+1{return Err(Failure::Invalid);}
  let mut bodies=vec![];for row in g.admissions.iter().take(index+1){let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;if !matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked){return Err(Failure::Changed);}bodies.push(body);}
  if matches!(r.schema_version,2|3){
   if s.coverage_hash!=model::fingerprint(&(if r.schema_version==3{"funded-recursive-result-plan-data-only-v1"}else{"funded-result-plan-data-only-v1"})).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}
   for p in &r.patches{crate::pet_runtime::derived_result_plan::Arguments::parse(p.after.operation.as_ref().ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?.verify_unlocked(self,g)?;}
  }else{let carry=Self::native_carry(g,&bodies,&r.source_proposal_hash)?;if model::fingerprint(&carry).map_err(|_|Failure::Invalid)?!=s.coverage_hash||!r.preserves(&carry){return Err(Failure::Changed);}}
  if let Some(a)=&s.abandoned{
   if g.native_plan_revisions.iter().any(|v|v.id==r.id||v.request_id==r.request_id)||g.sequence<a.goal_sequence||g.sequence==a.goal_sequence&&(g.revision!=a.goal_revision||model::fingerprint(g).map_err(|_|Failure::Invalid)?!=a.goal_hash){return Err(Failure::Changed);}return Ok(false);
  }
  if s.content_matches(g){return Ok(false);}if r.source_matches(g)||s.absent_in(g){return Ok(true);}Err(Failure::Changed)
 }
}
