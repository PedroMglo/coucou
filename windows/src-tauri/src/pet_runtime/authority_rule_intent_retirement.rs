//! Consolidate authenticated DATA without ever publishing Active authority.
//! No worker-end witness, physical-resource release, refund or new execution.
use super::*;
use crate::pet_runtime::{rules::{RuleGrant,RulePhase,RuleReservation,ReadinessProbe,RuleIntentRetirement,RetiredRuleIntent},runtime_owner::Profile};
// Thirty-two original images plus the one already-approved publication intent.
const MAX_IMAGES:usize=33;
const MAX_SOURCE_BYTES:usize=1024*1024;
#[derive(Clone)]struct Image{name:String,body:RuleGrant,stamp:Stamp}
/// Native-only authenticated source. Renderer cannot manufacture one.
pub(in crate::pet_runtime)struct RuleIntentRetirementSource{
 base:RuleGrant,base_stamp:Stamp,images:Vec<Image>,profile:Profile,
 merged:RuleGrant,recorded:Option<RuleGrant>,hash:String,
}
fn fp<T:Serialize>(v:&T)->Result<String>{model::fingerprint(v).map_err(|_|Failure::Invalid)}
fn entries(images:&[Image])->Result<Vec<RetiredRuleIntent>>{
 images.iter().map(|i|Ok(RetiredRuleIntent{name:i.name.clone(),body_hash:fp(&i.body)?})).collect()
}
fn same_parent(a:&RuleGrant,b:&RuleGrant)->bool{
 a.revision_authority==b.revision_authority&&a.succession==b.succession&&a.kind==b.kind&&a.decision_id==b.decision_id&&a.approved_at==b.approved_at&&a.policy==b.policy
}
fn join_probe(a:&mut ReadinessProbe,b:&ReadinessProbe)->Result<()>{
 let mut immutable=b.clone();immutable.finished_at=a.finished_at;immutable.observed=a.observed;immutable.recovery=a.recovery.clone();
 if immutable!=*a{return Err(Failure::RecoveryRequired);}
 match(a.finished_at,b.finished_at){
  (None,Some(_))=>{a.finished_at=b.finished_at;a.observed=b.observed;},
  (Some(x),Some(y))if x!=y||a.observed!=b.observed=>return Err(Failure::RecoveryRequired),_=>{},
 }
 match(&a.recovery,&b.recovery){(None,Some(r))=>a.recovery=Some(r.clone()),(Some(x),Some(y))if x!=y=>return Err(Failure::RecoveryRequired),_=>{}}
 Ok(())
}
fn join_reservation(a:&mut RuleReservation,b:&RuleReservation)->Result<()>{
 let mut immutable=b.clone();immutable.readiness=a.readiness.clone();immutable.admission=a.admission.clone();
 if immutable!=*a{return Err(Failure::RecoveryRequired);}
 match(&a.admission,&b.admission){(None,Some(v))=>a.admission=Some(v.clone()),(Some(x),Some(y))if x!=y=>return Err(Failure::RecoveryRequired),_=>{}}
 // Sequence order is causal, not lexical. A different ID at the same index
 // is a conflicting branch, not another allowance to add.
 for(i,p)in b.readiness.iter().enumerate(){if let Some(old)=a.readiness.get_mut(i){join_probe(old,p)?;}else{a.readiness.push(p.clone());}}
 Ok(())
}
fn join(base:&RuleGrant,images:&[Image])->Result<RuleGrant>{join_bodies(base,images.iter().map(|i|&i.body))}
fn join_bodies<'a>(base:&RuleGrant,bodies:impl IntoIterator<Item=&'a RuleGrant>)->Result<RuleGrant>{
 if base.phase!=RulePhase::Revoked{return Err(Failure::Revoked);}let mut g=base.clone();
 for b in bodies{
  if !b.valid()||!same_parent(base,b)||!base.intent_retirements.starts_with(&b.intent_retirements)
   ||match b.phase{RulePhase::Active=>b.updated_at>base.revoked_at.ok_or(Failure::Invalid)?,RulePhase::Revoked=>b.revoked_at!=base.revoked_at}{return Err(Failure::RecoveryRequired);}
  for r in &b.reservations{if let Some(old)=g.reservations.iter_mut().find(|v|v.occurrence_id==r.occurrence_id){join_reservation(old,r)?;}else{g.reservations.push(r.clone());}}
  g.updated_at=g.updated_at.max(b.updated_at);
 }
 g.schema_version=g.required_version();
 if !g.valid(){return Err(Failure::RecoveryRequired);}Ok(g)
}
fn subsumed(current:&RuleGrant,image:&RuleGrant)->bool{
 join_bodies(current,std::iter::once(image)).is_ok_and(|merged|merged==*current)
}
fn candidate(base:&RuleGrant,images:&[Image],decision:&str,now:u64)->Result<RuleGrant>{
 if !model::token(decision)||images.is_empty(){return Err(Failure::Invalid);}let mut g=join(base,images)?;if now<g.updated_at{return Err(Failure::Changed);}
 let intents=entries(images)?;let receipt=RuleIntentRetirement{schema_version:1,native_decision_id:decision.into(),canonical_hash:fp(base)?,inventory_hash:fp(&intents)?,merged_ledger_hash:fp(&g.reservations)?,source_schema_version:base.schema_version,source_updated_at:base.updated_at,approved_at:now,intents};
 g.intent_retirements.push(receipt);g.schema_version=g.required_version();g.updated_at=now;g.revision=decision.into();
 if !g.valid()||serde_json::to_vec(&g).map_err(|_|Failure::Invalid)?.len()+256>MAX_GRANT_BYTES{return Err(Failure::Capacity);}Ok(g)
}
impl RuleIntentRetirementSource{
 pub(in crate::pet_runtime)fn rule(&self)->&crate::pet_runtime::rules::Rule{&self.base.policy.rule}
 pub(in crate::pet_runtime)fn preview(&self)->serde_json::Value{serde_json::json!({
  "schemaVersion":1,"sourceHash":self.hash,"ruleId":self.rule().id,"petId":self.rule().owner.pet_id,
  "parentDecisionId":self.base.decision_id,"parentRevision":self.base.revision,"canonicalHash":fp(&self.base).ok(),
  "pendingIntents":self.images.len(),"intentDigests":entries(&self.images).ok(),"recordedChoice":self.recorded.is_some(),
  "committedOccurrences":self.merged.reservations.len(),"unknownReadinessProbes":self.merged.reservations.iter().flat_map(|r|&r.readiness).filter(|p|p.finished_at.is_none()).count(),
  "originalRevokedAt":self.base.revoked_at,"originalValidUntil":self.base.policy.valid_until,
  "model":self.base.policy.model,"destination":self.base.policy.destination,"selectedSignedSource":self.merged,
  "executionAuthorityGranted":false,"workerEndProven":false,"resourcesReleased":false,"ioWillReplay":false,
 })}
 pub(in crate::pet_runtime)fn candidate(&self,decision:&str,now:u64)->Result<RuleGrant>{
  if let Some(g)=&self.recorded{return Ok(g.clone());}candidate(&self.base,&self.images,decision,now)
 }
 fn matches(&self,g:&RuleGrant)->bool{
  if let Some(recorded)=&self.recorded{return g==recorded;}
  g.intent_retirements.last().is_some_and(|r|candidate(&self.base,&self.images,&r.native_decision_id,r.approved_at).is_ok_and(|expected|expected==*g))
 }
}
impl Authority{
 fn rule_intent_images(&self,decision:&str)->Result<Vec<Image>>{
  let mut images=vec![];let mut bytes=0usize;
  for(name,g,canonical)in self.rule_inventory_unlocked()?{if canonical||g.decision_id!=decision{continue;}
   if images.len()>=MAX_IMAGES{return Err(Failure::Capacity);}
   let(body,stamp)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;if body!=g{return Err(Failure::Changed);}
   bytes=bytes.checked_add(serde_json::to_vec(&body).map_err(|_|Failure::Invalid)?.len()).ok_or(Failure::Capacity)?;if bytes>MAX_SOURCE_BYTES{return Err(Failure::Capacity);}
   images.push(Image{name,body,stamp});
  }images.sort_by(|a,b|a.name.cmp(&b.name));Ok(images)
 }
 pub(in crate::pet_runtime)fn prepare_rule_intent_retirement(&self,decision:&str)->Result<RuleIntentRetirementSource>{
  if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock_shared()?;let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
  let(base,base_stamp)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;if base.phase!=RulePhase::Revoked{return Err(Failure::Revoked);}
  let images=self.rule_intent_images(decision)?;if images.is_empty(){return Err(Failure::Missing);}
  let(_,profile)=self.owner_context()?;let hash=fp(&("coucou-revoked-rule-intent-source-v1",&base,&base_stamp,entries(&images)?,&profile))?;
  // A durable receipt permits ACK of only its exact original images, or the
  // identical signed candidate itself. Partial cleanup needs no fresh choice.
  let covered=|i:&Image|->bool{if i.body==base{return true;}let Ok(h)=fp(&i.body)else{return false;};base.intent_retirements.iter().any(|r|r.intents.iter().any(|e|e.name==i.name&&e.body_hash==h))&&subsumed(&base,&i.body)};
  if !base.intent_retirements.is_empty()&&images.iter().all(covered){return Ok(RuleIntentRetirementSource{merged:base.clone(),recorded:Some(base.clone()),base,base_stamp,images,profile,hash});}
  // Publication before rename may have retained the ORIGINAL approved choice.
  // Validate it against all the other exact images; never mint another choice.
  let staged:Vec<_>=images.iter().enumerate().filter(|(_,i)|i.body.intent_retirements.len()==base.intent_retirements.len()+1&&i.body.intent_retirements.starts_with(&base.intent_retirements)).collect();
  if !staged.is_empty(){
   if staged.len()!=1{return Err(Failure::RecoveryRequired);}let(index,image)=staged[0];let originals:Vec<_>=images.iter().enumerate().filter(|(n,_)|*n!=index).map(|(_,i)|i.clone()).collect();
   let r=image.body.intent_retirements.last().ok_or(Failure::Invalid)?;let expected=candidate(&base,&originals,&r.native_decision_id,r.approved_at)?;
   if expected!=image.body{return Err(Failure::RecoveryRequired);}
   return Ok(RuleIntentRetirementSource{merged:expected.clone(),recorded:Some(expected),base,base_stamp,images,profile,hash});
  }
  let merged=join(&base,&images)?;self.fence()?;Ok(RuleIntentRetirementSource{base,base_stamp,images,profile,merged,recorded:None,hash})
 }
 /// No user cache lock is held during durable IO. The same Authority lock and
 /// exact native choice govern publication, loss of ACK and stamp-safe cleanup.
 pub(in crate::pet_runtime)fn retire_rule_intents(&self,source:&RuleIntentRetirementSource,chosen:&RuleGrant,now:u64,current:impl Fn()->bool)->Result<RuleGrant>{
  if !current()||!chosen.valid()||!source.matches(chosen)||now<chosen.updated_at{return Err(Failure::Changed);}
  let _lock=self.lock()?;let check=||{if !current()||self.owner_context()?.1!=source.profile{return Err(Failure::Changed);}self.fence()};check()?;
  let name=format!("{}.json",source.base.decision_id);if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
  let(canonical,stamp)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;let images=self.rule_intent_images(&canonical.decision_id)?;
  let already=canonical.intent_retirements.starts_with(&chosen.intent_retirements)&&subsumed(&canonical,chosen);
  if !already{
   if canonical!=source.base||stamp!=source.base_stamp{return Err(Failure::Changed);}
   // The original review may now additionally have exactly one publication
   // intent of this same candidate after a failed first attempt.
   for i in &images{if !source.images.iter().any(|s|s.name==i.name&&s.body==i.body&&s.stamp==i.stamp)&&i.body!=*chosen{return Err(Failure::Changed);}}
   if source.images.iter().any(|s|!images.iter().any(|i|s.name==i.name&&s.body==i.body&&s.stamp==i.stamp)){return Err(Failure::Changed);}
   if let Some(i)=images.iter().find(|i|i.body==*chosen){
    check()?;self.dir.rename(&i.name,&self.dir,&name,false).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;
   }else{self.publish_rule(chosen,false,check)?;}
  }
  let(final_body,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
  if !final_body.intent_retirements.starts_with(&chosen.intent_retirements)||!subsumed(&final_body,chosen){return Err(Failure::Changed);}
  check()?;self.stabilize_rule(&final_body)?;
  for i in self.rule_intent_images(&canonical.decision_id)?{
   let h=fp(&i.body)?;let reviewed=source.images.iter().any(|s|s.name==i.name&&s.body==i.body&&s.stamp==i.stamp)
    ||i.body==*chosen||final_body.intent_retirements.iter().any(|r|r.intents.iter().any(|e|e.name==i.name&&e.body_hash==h));
   if !reviewed||!subsumed(&final_body,&i.body){return Err(Failure::RecoveryRequired);}
   check()?;
   #[cfg(test)]if matches!(self.fault,Fault::BeforeRuleIntentCleanup){return Err(Failure::Uncertain);}
   self.dir.remove_exact(&i.name,&i.stamp).map_err(|_|Failure::Uncertain)?;
   #[cfg(test)]if matches!(self.fault,Fault::AfterRuleIntentCleanup){return Err(Failure::Uncertain);}
  }
  self.dir.durable().map_err(|_|Failure::Uncertain)?;check()?;Ok(final_body)
 }
}
#[cfg(test)]#[path="rule_intent_retirement_tests.rs"]mod tests;
