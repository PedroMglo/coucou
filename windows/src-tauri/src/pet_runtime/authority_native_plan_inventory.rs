//! Signed facts from the current profile only, with pagination and no writes.
use super::*;
use crate::pet_runtime::{native_plan_inventory::{Inventory,Item,State as ViewState},native_plan_revisions::Seal};
use std::collections::BTreeMap;
impl Authority{
 pub(in crate::pet_runtime)fn native_plan_inventory(&self,g:&Record,cursor:Option<&str>)->Result<Inventory>{
  if cursor.is_some_and(|s|!model::token(s)){return Err(Failure::Invalid);}let _guard=self.lock_shared()?;
  let mut rows=BTreeMap::new();let mut facts:BTreeMap<String,Seal>=BTreeMap::new();let mut sources=vec![];let mut bodies=vec![];
  for row in &g.admissions{
   let b=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;sources.push((format!("canonical-{}",b.decision_id),model::fingerprint(&b).map_err(|_|Failure::Invalid)?));
   for s in &b.native_plan_revisions{
    if rows.len()>=1024{return Err(Failure::Capacity);}let proof=self.verify_native_plan_fact(g,&b,s);let present=s.content_matches(g);let absent=s.absent_in(g);
    let state=if proof.is_err(){ViewState::Blocked}else if s.abandoned.is_some(){ViewState::Abandoned}else if present{ViewState::Published}else if absent{ViewState::Sealed}else{ViewState::Blocked};
    let item=Item{reference:s.reference.clone(),state,pending_artifacts:false,can_ack:proof.is_ok()&&present,can_abandon:proof.is_ok()&&absent&&s.abandoned.is_none(),reason:proof.err()};
    if rows.insert(s.reference.id.clone(),item).is_some(){return Err(Failure::Changed);}facts.insert(s.reference.id.clone(),s.clone());
   }bodies.push(b);
  }
  let last=g.admissions.last().map(|r|self.load(&r.spec,r.proof.as_ref().ok_or(Failure::Missing)?)).transpose()?;
  let mut staged_count=0;
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}if self.rule_record_data(&name)?.is_some(){continue;}let(staged,_)=self.grant_file(&name)?;if staged.goal_id!=g.id{continue;}
   staged_count+=1;if staged_count>64{return Err(Failure::Capacity);}sources.push((name.clone(),model::fingerprint(&staged).map_err(|_|Failure::Invalid)?));let Some(s)=staged.native_plan_revisions.last()else{continue;};let id=s.reference.id.clone();
   if let Some(known)=facts.get(&id){
    let mut old=known.clone();old.abandoned=None;let mut candidate=s.clone();candidate.abandoned=None;
    let item=rows.get_mut(&id).ok_or(Failure::Missing)?;item.pending_artifacts=true;
    if old!=candidate||Self::native_plan_staging_source(&staged,s).is_err(){item.state=ViewState::Blocked;item.can_ack=false;item.can_abandon=false;item.reason=Some(Failure::Changed);continue;}
    if known.abandoned.is_some(){item.can_abandon=s.absent_in(g);}
    if known.abandoned.is_none()&&s.abandoned.is_some(){item.state=ViewState::WithdrawalPending;item.can_ack=false;item.can_abandon=s.absent_in(g);}
    continue;
   }
   let Some(body)=last.as_ref()else{return Err(Failure::Missing);};let origin=Self::native_plan_staging_source(&staged,s)?;
   let valid_origin=staged.decision_id==body.decision_id&&staged.spec_hash==body.spec_hash&&staged.lease==body.lease&&(origin==model::fingerprint(body).map_err(|_|Failure::Invalid)?||Self::native_plan_revoked_origin(body,&staged,s));
   let absent=s.absent_in(g);let state=if !valid_origin||!absent{ViewState::Blocked}else if s.abandoned.is_some(){ViewState::WithdrawalPending}else{ViewState::Staged};
   let item=Item{reference:s.reference.clone(),state,pending_artifacts:true,can_ack:false,can_abandon:valid_origin&&absent,reason:(!valid_origin||!absent).then_some(Failure::Changed)};
   if let Some(old)=rows.get_mut(&id){if old.reference!=item.reference{old.state=ViewState::Blocked;old.can_abandon=false;old.reason=Some(Failure::Changed);}}else{rows.insert(id,item);}
  }
  let coverage=if !bodies.is_empty()&&g.admissions.iter().all(|r|r.spec.deterministic()){Self::native_carry(g,&bodies,&crate::pet_runtime::plans::fingerprint(g).map_err(|_|Failure::Invalid)?).ok()}else{None};
  // Accounting status alone is not a result. Reuse the same receipt/lineage
  // reconciliation as continuation admission, including missing settled output.
  let observed_at=crate::assistant_memory::now();let history=self.history(g,observed_at);
  let fact_coverage_complete=coverage.is_some()&&staged_count==0&&rows.values().all(|r|matches!(r.state,ViewState::Published|ViewState::Abandoned))&&bodies.iter().all(|b|matches!(b.phase,GrantPhase::Closed|GrantPhase::Revoked))
   &&history.as_ref().is_ok_and(|h|h.outcomes_complete&&h.pending_artifacts==0);
  match &history{Ok(h)=>sources.push(("history".into(),h.source_hash.clone())),Err(e)=>sources.push(("history-unavailable".into(),model::fingerprint(e).map_err(|_|Failure::Invalid)?))}
  sources.sort();let source_hash=model::fingerprint(&(model::fingerprint(g).map_err(|_|Failure::Invalid)?,&sources)).map_err(|_|Failure::Invalid)?;
  let mut inventory=Inventory{schema_version:1,goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),revision:g.revision.clone(),source_hash,observed_at,coverage,fact_coverage_complete,items:vec![],has_more:false,next_cursor:None,execution_authorized:false,repair_performed:false};
  for(id,item)in rows{if cursor.is_some_and(|c|id.as_str()<=c){continue;}
   if inventory.items.len()>=16{inventory.has_more=true;break;}let previous=inventory.next_cursor.clone();inventory.items.push(item);inventory.next_cursor=Some(id);
   if crate::pet_runtime::plans::encode_input(&inventory).is_err(){inventory.items.pop();inventory.next_cursor=previous;if inventory.items.is_empty(){return Err(Failure::Capacity);}inventory.has_more=true;break;}
  }
  if !inventory.has_more{inventory.next_cursor=None;}self.fence()?;Ok(inventory)
 }
}
