//! Evidence from canonical native receipts, never a recipe or recipient result.
use super::{model::{self,Record},criteria::{Binding,CompletionRef,CompletionSource,CriterionResult,Status},admission::AdmissionSpec,authority::{Grant,Operation},messages::{Receipt,MAX_MESSAGES}};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReceiptRef{#[serde(default,skip_serializing_if="Option::is_none")]pub source_decision_id:Option<String>,pub plan_id:String,pub step_id:String,pub message_id:String,pub recipient_pet_id:String,pub hash:String}
impl ReceiptRef{
 pub(super)fn from(r:&Receipt)->model::Result<Self>{let e=r.envelope();Ok(Self{source_decision_id:None,plan_id:e.cause.plan_id.clone(),step_id:e.cause.step_id.clone(),message_id:e.message_id.clone(),recipient_pet_id:e.recipient.pet_id.clone(),hash:model::fingerprint(r)?})}
 pub(super)fn historical(r:&Receipt)->model::Result<Self>{let mut reference=Self::from(r)?;reference.source_decision_id=Some(r.envelope().origin.decision_id.clone());Ok(reference)}
 pub(super)fn valid(&self)->bool{self.source_decision_id.as_ref().is_none_or(|s|model::token(s))&&model::key(&self.plan_id)&&model::key(&self.step_id)&&model::token(&self.message_id)&&crate::pet_config::blueprint(&self.recipient_pet_id).is_some()&&super::authority::decode(&self.hash).is_some()}
 pub(super)fn matches(&self,r:&Receipt)->bool{let mut actual=match Self::from(r){Ok(v)=>v,Err(_)=>return false};if self.source_decision_id.is_some(){actual.source_decision_id=Some(r.envelope().origin.decision_id.clone());}actual==*self}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Report{
 #[serde(default,skip_serializing_if="Option::is_none")]pub generated_plan:Option<super::generated_plans::Reference>,
 pub schema_version:u32,pub reference:CompletionRef,pub binding:Binding,pub admission_hash:String,pub spec_hash:String,pub proposal_hash:String,
 pub source_sequence:u64,pub source_updated_at:u64,pub source_record_hash:String,pub source_grant_revision:String,pub source_grant_hash:String,
 pub receipts:Vec<ReceiptRef>,pub expected_messages:u32,pub budget_hash:String,pub results:Vec<CriterionResult>,pub criteria_verified:bool,pub recorded_at:u64,
}
impl Report{
 pub(super)fn content_hash(&self)->model::Result<String>{let mut b=self.clone();b.reference.report_hash.clear();model::fingerprint(&b)}
 pub(super)fn matches_spec(&self,spec:&AdmissionSpec)->bool{if self.schema_version==3{return self.planned_matches(spec);}if self.generated_plan.is_some(){return false;}self.reference.goal_id==spec.goal_id&&self.reference.pet_id==spec.owner.pet_id&&self.reference.decision_id==spec.decision_id&&matches!(spec.schema_version,5|6|7|8|9|10|11|14|16|17|18|20)&&self.schema_version==(if matches!(spec.schema_version,6|7|8|16){2}else{1})&&spec.continuation.as_ref().and_then(|c|c.native.as_ref()).is_none_or(|n|n.receipts.iter().all(|r|self.receipts.contains(r))&&self.receipts.iter().filter(|r|r.source_decision_id.is_some()).count()==n.receipts.len())&&spec.native_verification.as_ref()==Some(&self.binding)&&self.admission_hash==model::fingerprint(spec).unwrap_or_default()&&self.spec_hash==spec.intent_hash&&spec.proposal_hash.as_ref()==Some(&self.proposal_hash)}
 pub(super)fn valid_for(&self,body:&Grant)->bool{
  if self.schema_version==3{return self.planned_valid(body);}
  if self.generated_plan.is_some(){return false;}
  let c=&self.reference;let Some(lease)=body.lease.as_ref()else{return false;};
  if !matches!(self.schema_version,1|2)||!c.valid()||!matches!(c.source,Some(CompletionSource::NativeMessages{..}))||body.operation!=Operation::GoalDeterministic||self.admission_hash!=body.spec_hash||c.goal_id!=body.goal_id||c.decision_id!=body.decision_id||c.lease_id!=lease.id||self.binding.schema_version!=2||!self.binding.valid()||self.binding.spec_hash!=self.spec_hash||!self.criteria_verified||self.results.len()!=self.binding.criteria_count as usize||self.binding.conditions.len()!=self.results.len()
   ||self.source_sequence==0||self.source_updated_at==0||self.source_updated_at>self.recorded_at||self.recorded_at!=c.observed_at||c.observed_at<lease.claimed_at||c.observed_at>=lease.deadline_at||self.recorded_at>body.updated_at||!model::token(&self.source_grant_revision)||[&self.source_record_hash,&self.source_grant_hash,&self.budget_hash,&self.proposal_hash].iter().any(|h|super::authority::decode(h).is_none())||self.budget_hash!=model::fingerprint(&body.budget_usage).unwrap_or_default()||self.content_hash().ok().as_ref()!=Some(&c.report_hash){return false;}
  let entries=body.messages.as_ref().map(|m|m.entries.as_slice()).unwrap_or(&[]);let mut seen=std::collections::BTreeSet::new();
  self.expected_messages>0&&self.expected_messages as usize<=if self.schema_version==1{MAX_MESSAGES}else{512}&&self.receipts.len()==self.expected_messages as usize&&entries.len()==self.receipts.iter().filter(|r|r.source_decision_id.is_none()).count()
   &&self.receipts.iter().all(|r|r.valid()&&seen.insert((&r.plan_id,&r.step_id))&&if r.source_decision_id.is_none(){entries.iter().any(|m|r.matches(m))}else{self.schema_version==2&&r.source_decision_id.as_ref()!=Some(&body.decision_id)})
   &&self.results.iter().enumerate().all(|(i,r)|r.criterion_index as usize==i&&r.status==Status::Met&&self.binding.conditions.iter().any(|b|b.criterion_index==r.criterion_index&&b.criterion_hash==r.criterion_hash))
 }
 pub(super)fn verify_source(&self,goal:&Record)->model::Result<()>{
  if self.schema_version==3{return self.planned_source(goal);}
  super::criteria::verify_source(goal,&self.reference,self.source_sequence,self.source_updated_at,&self.source_record_hash,&self.spec_hash,&self.results)?;
  let steps=goal.plans.iter().flat_map(|p|p.steps.iter().map(move|s|(p,s))).collect::<Vec<_>>();
  if self.expected_messages as usize!=steps.len()||self.receipts.len()!=steps.len()||steps.iter().any(|(p,s)|s.operation.as_ref().is_none_or(|op|super::messages::Input::parse(op).is_err()||!self.receipts.iter().any(|r|r.plan_id==p.id&&r.step_id==s.id&&super::messages::Input::parse(op).is_ok_and(|a|a.to_pet()==r.recipient_pet_id)))){return Err(model::Error::Conflict);}Ok(())
 }
 pub(super)fn observe(goal:&Record,spec:&AdmissionSpec,body:&Grant,id:String,target:String,now:u64)->model::Result<Self>{
  if spec.peer_planning(){return Self::observe_planned(goal,spec,body,id,target,now);}
  let binding=spec.native_verification.clone().filter(|b|matches!(spec.schema_version,5|6|7|8|9|10|11|14|16|17|18|20)&&b.schema_version==2&&b.matches(goal)).ok_or(model::Error::InvalidInput)?;
  if !super::plans::matches(goal,spec)||!super::deterministic::plan(goal){return Err(model::Error::Conflict);}let mut receipts=vec![];let mut expected=0;
  for p in &goal.plans{for s in &p.steps{expected+=1;
   if let Some(old)=spec.continuation.as_ref().and_then(|c|c.native.as_ref()).and_then(|n|n.receipts.iter().find(|r|r.plan_id==p.id&&r.step_id==s.id)){
    if body.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().cause.plan_id==p.id&&r.envelope().cause.step_id==s.id)){return Err(model::Error::Conflict);}receipts.push(old.clone());continue;
   }
   if let Some(receipt)=body.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.envelope().cause.plan_id==p.id&&r.envelope().cause.step_id==s.id)){
    if !receipt.scope.matches(goal)||!receipt.scope.belongs(spec,body.lease.as_ref().ok_or(model::Error::Conflict)?){return Err(model::Error::Conflict);}receipts.push(ReceiptRef::from(receipt)?);
   }
  }}
  if expected==0||expected>if matches!(spec.schema_version,6|7|8|16){512}else{MAX_MESSAGES}{return Err(model::Error::LimitReached);}let met=receipts.len()==expected;
  let results=goal.spec.criteria.iter().enumerate().map(|(i,text)|Ok(CriterionResult{criterion_index:i as u32,criterion_hash:model::fingerprint(text)?,status:if binding.conditions.iter().any(|c|c.criterion_index as usize==i){if met{Status::Met}else{Status::Unmet}}else{Status::Unknown}})).collect::<model::Result<Vec<_>>>()?;
  let complete=results.iter().all(|r|r.status==Status::Met);let lease=body.lease.as_ref().ok_or(model::Error::Conflict)?;
  let mut report=Self{generated_plan:None,schema_version:if matches!(spec.schema_version,6|7|8|16){2}else{1},reference:CompletionRef{source:Some(CompletionSource::NativeMessages{verification_id:id}),schema_version:2,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),recipe_id:String::new(),plan_id:String::new(),decision_id:spec.decision_id.clone(),lease_id:lease.id.clone(),source_revision:goal.revision.clone(),target_revision:target,report_hash:String::new(),observed_at:now},binding,admission_hash:model::fingerprint(spec)?,spec_hash:spec.intent_hash.clone(),proposal_hash:spec.proposal_hash.clone().ok_or(model::Error::Conflict)?,source_sequence:goal.sequence,source_updated_at:goal.updated_at,source_record_hash:model::fingerprint(goal)?,source_grant_revision:body.revision.clone(),source_grant_hash:model::fingerprint(body)?,receipts,expected_messages:expected as u32,budget_hash:model::fingerprint(&body.budget_usage)?,results,criteria_verified:complete,recorded_at:now};report.reference.report_hash=report.content_hash()?;Ok(report)
 }
}

#[path="planned_peer_verification.rs"]mod planned;
