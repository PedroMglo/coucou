//! Reviewed native plan DATA. Patches preserve stable causes and original
//! inputs; neither parsed history nor a matching hash grants any execution.
use super::{model::{self,Record,State,StepInput},messages::{Input as MessageInput,Cause},plans::{self,Snapshot},continuation::NativeCarry};
use serde::{Serialize,Deserialize};
pub(super)const MAX_REVISIONS:usize=16;
pub(super)const MAX_CHANGES:usize=16;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Input{pub cause:Cause,pub after:StepInput}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Patch{pub cause:Cause,pub before:StepInput,pub after:StepInput}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
 pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,pub decision_id:String,
 pub source_revision:String,pub source_sequence:u64,pub source_goal_hash:String,pub source_proposal_hash:String,
 pub target_revision:String,pub target_sequence:u64,pub target_state:State,pub target_proposal_hash:String,
 pub reviewed_at:u64,pub patches:Vec<Patch>,
}
fn input_valid(s:&StepInput,owner:&str)->bool{
 model::text(&s.description,4096)&&model::text(&s.postcondition,4096)&&s.operation.as_ref().is_some_and(|o|MessageInput::parse(o).is_ok_and(|a|a.to_pet()!=owner))
}
impl Reference{
 pub(super)fn valid(&self)->bool{
  let mut causes=std::collections::BTreeSet::new();
  matches!(self.schema_version,1|2|3)&&[&self.id,&self.decision_id,&self.source_revision,&self.target_revision].iter().all(|s|model::token(s))
   &&model::key(&self.request_id)&&model::goal_id(&self.goal_id)&&crate::pet_config::blueprint(&self.pet_id).is_some()
   &&[&self.source_goal_hash,&self.source_proposal_hash,&self.target_proposal_hash].iter().all(|h|super::authority::decode(h).is_some())
   &&self.source_proposal_hash!=self.target_proposal_hash&&self.target_revision!=self.source_revision&&self.source_sequence>0&&self.target_sequence==self.source_sequence.saturating_add(1)&&self.target_sequence<model::MAX_EVENTS as u64
   &&matches!(self.target_state,State::Paused|State::RecoveryRequired)&&self.reviewed_at>0&&self.reviewed_at<=8_640_000_000_000_000
   &&!self.patches.is_empty()&&self.patches.len()<=MAX_CHANGES&&self.patches.iter().all(|p|model::key(&p.cause.task_id)&&model::key(&p.cause.plan_id)&&model::key(&p.cause.step_id)
      &&causes.insert((&p.cause.task_id,&p.cause.plan_id,&p.cause.step_id))&&p.before!=p.after&&(if self.schema_version==1{input_valid(&p.before,&self.pet_id)&&input_valid(&p.after,&self.pet_id)}else{model::text(&p.before.description,4096)&&model::text(&p.before.postcondition,4096)&&p.before.operation.is_none()&&model::text(&p.after.description,4096)&&model::text(&p.after.postcondition,4096)&&p.after.operation.as_ref().is_some_and(|o|super::derived_result_plan::Arguments::parse(o).is_ok_and(|a|a.result.goal_id==self.goal_id&&a.result.pet_id==self.pet_id))}))
   &&(!matches!(self.schema_version,2|3)||self.patches.len()==1)
   &&plans::encode_input(self).is_ok()
 }
 pub(super)fn source_matches(&self,g:&Record)->bool{self.valid()&&g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.state==self.target_state&&g.revision==self.source_revision&&g.sequence==self.source_sequence&&g.updated_at<=self.reviewed_at
  &&model::fingerprint(g).ok().as_ref()==Some(&self.source_goal_hash)&&plans::fingerprint(g).ok().as_ref()==Some(&self.source_proposal_hash)
  &&g.admissions.last().is_some_and(|a|(if matches!(self.schema_version,2|3){a.spec.funded_response()&&((self.schema_version==3)==(a.spec.schema_version==19))}else{a.spec.deterministic()})&&a.spec.decision_id==self.decision_id)}
 pub(super)fn preserves_source(&self,source:&super::continuation::Reference)->bool{if matches!(self.schema_version,2|3){self.valid()&&source.schema_version==(if self.schema_version==3{9}else{6})&&source.native.is_none()&&source.funded.is_some()}else{source.native.as_ref().is_some_and(|c|self.preserves(c))}}
 pub(super)fn coverage_hash(source:&super::continuation::Reference)->model::Result<String>{match &source.native{Some(c)=>model::fingerprint(c),None if source.schema_version==6=>model::fingerprint(&"funded-result-plan-data-only-v1"),None if source.schema_version==9=>model::fingerprint(&"funded-recursive-result-plan-data-only-v1"),_=>Err(model::Error::Conflict)}}
 pub(super)fn preserves(&self,carry:&NativeCarry)->bool{self.valid()&&carry.proposal_hash==self.source_proposal_hash&&self.patches.iter().all(|p|!carry.covers(&p.cause.plan_id,&p.cause.step_id))}
 pub(super)fn apply_snapshot(&self,s:&mut Snapshot,reverse:bool)->model::Result<()>{
  if !self.valid()||model::fingerprint(s)?!=if reverse{self.target_proposal_hash.clone()}else{self.source_proposal_hash.clone()}{return Err(model::Error::Conflict);}
  let mut candidate=s.clone();for p in &self.patches{let(before,after)=if reverse{(&p.after,&p.before)}else{(&p.before,&p.after)};candidate.replace_input(&p.cause,before,after)?;}
  if model::fingerprint(&candidate)?!=if reverse{self.source_proposal_hash.clone()}else{self.target_proposal_hash.clone()}{return Err(model::Error::Conflict);}*s=candidate;Ok(())
 }
 pub(super)fn present(&self,g:&Record)->bool{self.valid()&&g.id==self.goal_id&&g.spec.owner.pet_id==self.pet_id&&g.native_plan_revisions.iter().filter(|r|*r==self).count()==1
  &&g.admissions.iter().any(|a|(if matches!(self.schema_version,2|3){a.spec.funded_response()&&((self.schema_version==3)==(a.spec.schema_version==19))}else{a.spec.deterministic()})&&a.spec.decision_id==self.decision_id)
  &&g.events.get((self.target_sequence-1)as usize).is_some_and(|e|matches!(e.kind,model::EventKind::NativePlanRevised)&&e.request_id==self.request_id&&e.at==self.reviewed_at&&model::fingerprint(self).ok().as_ref()==Some(&e.fingerprint))}
}
/// Validate every link, even when the requested snapshot is the current one.
/// Callers must separately authenticate references against the signed grants.
pub(super)fn snapshot_at(g:&Record,wanted:&str)->model::Result<Snapshot>{
 if g.native_plan_revisions.len()>MAX_REVISIONS{return Err(model::Error::LimitReached);}let mut s=plans::snapshot(g);let mut found=if model::fingerprint(&s)?==wanted{Some(s.clone())}else{None};let mut seq=g.sequence+1;
 for r in g.native_plan_revisions.iter().rev(){if !r.present(g)||r.target_sequence>seq{return Err(model::Error::Conflict);}seq=r.source_sequence;r.apply_snapshot(&mut s,true)?;if model::fingerprint(&s)?==wanted{found=Some(s.clone());}}
 found.ok_or(model::Error::Conflict)
}
#[cfg(target_os="linux")]
pub(super)fn prepare(g:&Record,carry:&NativeCarry,inputs:Vec<Input>,request:String,now:u64)->model::Result<Reference>{
 g.validate()?;if !super::deterministic::plan(g)||!matches!(g.state,State::Paused|State::RecoveryRequired)||!carry.matches(g)||g.native_plan_revisions.len()>=MAX_REVISIONS||inputs.is_empty()||inputs.len()>MAX_CHANGES||!model::key(&request)||now<g.updated_at{return Err(model::Error::Conflict);}
 let original=plans::snapshot(g);let mut target=original.clone();let mut patches=vec![];
 for input in inputs{let before=original.input(&input.cause).ok_or(model::Error::Conflict)?;if carry.covers(&input.cause.plan_id,&input.cause.step_id)||before==input.after||!input_valid(&input.after,&g.spec.owner.pet_id)||input.after.operation.as_ref().is_none_or(|o|!MessageInput::parse(o).is_ok_and(|a|a.deadline()>now)){return Err(model::Error::Conflict);}target.replace_input(&input.cause,&before,&input.after)?;patches.push(Patch{cause:input.cause,before,after:input.after});}
 let r=Reference{schema_version:1,id:super::store::nonce()?,request_id:request,goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),decision_id:g.admissions.last().ok_or(model::Error::Conflict)?.spec.decision_id.clone(),source_revision:g.revision.clone(),source_sequence:g.sequence,source_goal_hash:model::fingerprint(g)?,source_proposal_hash:plans::fingerprint(g)?,target_revision:super::store::nonce()?,target_sequence:g.sequence+1,target_state:g.state,target_proposal_hash:model::fingerprint(&target)?,reviewed_at:now,patches};
 if !r.valid()||!r.preserves(carry){return Err(model::Error::InvalidInput);}Ok(r)
}
#[cfg(target_os="linux")]
pub(super)fn prepare_result(g:&Record,source:&super::continuation::Reference,inputs:Vec<Input>,request:String,now:u64)->model::Result<Reference>{
 g.validate()?;if !matches!(source.schema_version,6|9)||!source.source_matches(g)||g.state!=State::Paused||g.admissions.last().is_none_or(|a|!a.spec.funded_response())||g.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.is_some())||g.native_plan_revisions.len()>=MAX_REVISIONS||inputs.len()!=1||!model::key(&request)||now<g.updated_at{return Err(model::Error::Conflict);}
 let original=plans::snapshot(g);let mut target=original.clone();let mut patches=vec![];
 for input in inputs{let before=original.input(&input.cause).ok_or(model::Error::Conflict)?;let args=super::derived_result_plan::Arguments::parse(input.after.operation.as_ref().ok_or(model::Error::InvalidInput)?)?;if before.operation.is_some()||args.expires_at<=now{return Err(model::Error::Conflict);}target.replace_input(&input.cause,&before,&input.after)?;patches.push(Patch{cause:input.cause,before,after:input.after});}
 let r=Reference{schema_version:if source.schema_version==9{3}else{2},id:super::store::nonce()?,request_id:request,goal_id:g.id.clone(),pet_id:g.spec.owner.pet_id.clone(),decision_id:g.admissions.last().ok_or(model::Error::Conflict)?.spec.decision_id.clone(),source_revision:g.revision.clone(),source_sequence:g.sequence,source_goal_hash:model::fingerprint(g)?,source_proposal_hash:plans::fingerprint(g)?,target_revision:super::store::nonce()?,target_sequence:g.sequence+1,target_state:g.state,target_proposal_hash:model::fingerprint(&target)?,reviewed_at:now,patches};if !r.valid()||!r.preserves_source(source){return Err(model::Error::InvalidInput);}Ok(r)
}
impl Record{
 pub(super)fn append_native_plan_revision(&mut self,r:Reference)->model::Result<()>{
  self.validate()?;if !r.source_matches(self)||self.native_plan_revisions.len()>=MAX_REVISIONS||self.events.iter().any(|e|e.request_id==r.request_id){return Err(model::Error::Conflict);}
  let mut snapshot=plans::snapshot(self);r.apply_snapshot(&mut snapshot,false)?;
  for p in &r.patches{let s=self.plans.iter_mut().find(|v|v.id==p.cause.plan_id&&v.task_id==p.cause.task_id).and_then(|v|v.steps.iter_mut().find(|s|s.id==p.cause.step_id)).ok_or(model::Error::Conflict)?;s.description=p.after.description.clone();s.postcondition=p.after.postcondition.clone();s.operation=p.after.operation.clone();}
  self.sequence=r.target_sequence;self.revision=r.target_revision.clone();self.updated_at=r.reviewed_at;self.events.push(model::Event{sequence:self.sequence,at:self.updated_at,request_id:r.request_id.clone(),fingerprint:model::fingerprint(&r)?,kind:model::EventKind::NativePlanRevised});self.native_plan_revisions.push(r);self.refresh_schema();self.validate()
 }
}
// A compact signed fact in the existing Grant. Coverage is recomputed from the
// original authenticated receipts, not copied into another delivery ledger.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Abandoned{pub goal_revision:String,pub goal_sequence:u64,pub goal_hash:String,pub observed_at:u64,pub source_grant_hash:String,pub source_grant_revision:String,pub source_grant_updated_at:u64,pub target_grant_revision:String,pub introduced:bool}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Seal{
 pub schema_version:u32,pub reference:Reference,pub target_goal_hash:String,pub source_grant_hash:String,pub source_grant_revision:String,pub source_grant_updated_at:u64,
 pub source_reference_hash:String,pub source_history_hash:String,pub source_evidence_fence:String,pub source_admissions:u8,pub coverage_hash:String,pub usage_hash:String,pub owner_hash:String,
 pub approved_at:u64,pub expires_at:u64,pub target_grant_revision:String,#[serde(default,skip_serializing_if="Option::is_none")]pub abandoned:Option<Abandoned>,
}
impl Seal{
 pub(super)fn valid_for(&self,b:&super::authority::Grant)->bool{
  let r=&self.reference;
  self.schema_version==r.schema_version&&r.valid()&&(if matches!(r.schema_version,2|3){b.operation==super::authority::Operation::ConversationRespond&&b.funded_by.as_ref().is_some_and(|f|(r.schema_version==3)==(f.schema_version==7))}else{b.operation==super::authority::Operation::GoalDeterministic})&&b.goal_id==r.goal_id&&b.decision_id==r.decision_id
   &&matches!(b.phase,super::authority::GrantPhase::Closed|super::authority::GrantPhase::Revoked)&&b.lease.as_ref().is_some_and(|l|self.source_grant_updated_at>=l.claimed_at)
   &&self.source_admissions>0&&self.source_admissions<=32&&model::token(&self.source_grant_revision)&&model::token(&self.target_grant_revision)
   &&[&self.target_goal_hash,&self.source_grant_hash,&self.source_reference_hash,&self.source_history_hash,&self.source_evidence_fence,&self.coverage_hash,&self.usage_hash,&self.owner_hash].iter().all(|h|super::authority::decode(h).is_some())
   &&self.source_grant_updated_at<=r.reviewed_at&&self.approved_at>=r.reviewed_at&&self.approved_at<self.expires_at&&self.expires_at==r.reviewed_at.saturating_add(120_000)&&self.expires_at<=8_640_000_000_000_000&&b.updated_at>=self.approved_at
   &&model::fingerprint(&(&b.budget_usage,&b.planning_usage,&b.resource_usage)).ok().as_ref()==Some(&self.usage_hash)&&b.runtime_owner.as_ref().is_some_and(|o|model::fingerprint(o).ok().as_ref()==Some(&self.owner_hash))
   &&self.abandoned.as_ref().is_none_or(|a|model::token(&a.goal_revision)&&model::token(&a.source_grant_revision)&&model::token(&a.target_grant_revision)&&a.goal_sequence>=r.source_sequence&&super::authority::decode(&a.goal_hash).is_some()&&super::authority::decode(&a.source_grant_hash).is_some()&&a.source_grant_updated_at<=a.observed_at&&a.observed_at>=self.approved_at&&a.observed_at<=b.updated_at)
 }
 pub(super)fn content_matches(&self,g:&Record)->bool{self.abandoned.is_none()&&self.reference.present(g)&&snapshot_at(g,&self.reference.target_proposal_hash).is_ok()
  &&(g.sequence!=self.reference.target_sequence||g.revision==self.reference.target_revision&&model::fingerprint(g).ok().as_ref()==Some(&self.target_goal_hash))}
 pub(super)fn absent_in(&self,g:&Record)->bool{g.id==self.reference.goal_id&&g.spec.owner.pet_id==self.reference.pet_id&&g.sequence>=self.reference.source_sequence
  &&matches!(g.state,State::Paused|State::Cancelled|State::RecoveryRequired)&&g.admissions.last().is_some_and(|a|a.spec.decision_id==self.reference.decision_id)
  &&!g.native_plan_revisions.iter().any(|r|r.id==self.reference.id||r.request_id==self.reference.request_id)&&plans::fingerprint(g).ok().as_ref()==Some(&self.reference.source_proposal_hash)}
 #[cfg(target_os="linux")]pub(super)fn prepare(g:&Record,source:&super::continuation::Reference,r:&Reference,b:&super::authority::Grant,approved_at:u64)->model::Result<Self>{
  if !r.source_matches(g)||!source.source_matches(g)||!r.preserves_source(source)||source.previous.last().is_none_or(|a|a.decision_id!=b.decision_id||model::fingerprint(b).ok().as_ref()!=Some(&a.grant_hash))||source.charge.unknown_tool_results>0||source.charge.unknown_model_results>0{return Err(model::Error::Conflict);}
  let mut target=g.clone();target.append_native_plan_revision(r.clone())?;
  Ok(Self{schema_version:r.schema_version,reference:r.clone(),target_goal_hash:model::fingerprint(&target)?,source_grant_hash:model::fingerprint(b)?,source_grant_revision:b.revision.clone(),source_grant_updated_at:b.updated_at,
   source_reference_hash:model::fingerprint(source)?,source_history_hash:source.history_hash.clone(),source_evidence_fence:source.evidence_fence.clone(),source_admissions:source.previous.len()as u8,coverage_hash:Reference::coverage_hash(source)?,usage_hash:model::fingerprint(&(&b.budget_usage,&b.planning_usage,&b.resource_usage))?,owner_hash:model::fingerprint(b.runtime_owner.as_ref().ok_or(model::Error::Conflict)?)?,approved_at,expires_at:r.reviewed_at.saturating_add(120_000),target_grant_revision:super::store::nonce()?,abandoned:None})
 }
}
