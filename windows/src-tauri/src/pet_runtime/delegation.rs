//! Delegated DATA and escrow in the SAME signed Grant. No child execution
//! authority is constructed by this module or by receiving an envelope.
use super::{model,messages::{Envelope,Scope},admission::Budget,authority::Grant};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Default,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Allowance{pub model_calls:u16,pub tool_calls:u16,pub output_bytes:u64}
impl Allowance{
 pub(super)fn balance_valid(&self)->bool{self.model_calls<=64&&self.tool_calls<=128&&self.output_bytes<=16*1024*1024}
 pub(super)fn fits(&self,a:&Self)->bool{self.model_calls<=a.model_calls&&self.tool_calls<=a.tool_calls&&self.output_bytes<=a.output_bytes}
 pub(super)fn valid(&self)->bool{self.model_calls<=64&&self.tool_calls<=128&&(self.model_calls>0||self.tool_calls>0)&&(1..=16*1024*1024).contains(&self.output_bytes)}
 pub(super)fn add(&mut self,a:&Self)->Option<()>{self.model_calls=self.model_calls.checked_add(a.model_calls)?;self.tool_calls=self.tool_calls.checked_add(a.tool_calls)?;self.output_bytes=self.output_bytes.checked_add(a.output_bytes)?;Some(())}
 pub(super)fn within(&self,b:&Budget)->bool{self.model_calls<=b.max_model_calls&&self.tool_calls<=b.max_tool_calls&&self.output_bytes<=b.max_output_bytes}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum BranchAction{Cancel,Reconcile}
fn depth()->u8{1}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Arguments{pub to_pet_id:String,pub subgoal:String,pub criteria:Vec<String>,pub allowance:Allowance,pub expires_at:u64,#[serde(default="depth")]pub max_depth:u8}
impl Arguments{
 pub(crate)fn parse(v:&serde_json::Value)->Result<Self,()>{let a:Self=serde_json::from_value(v.clone()).map_err(|_|())?;if !a.valid(){return Err(());}Ok(a)}
 fn valid(&self)->bool{crate::pet_config::blueprint(&self.to_pet_id).is_some()&&super::messages::body(&self.subgoal)&&!self.criteria.is_empty()&&self.criteria.len()<=16&&self.criteria.iter().all(|s|super::messages::body(s)&&s.len()<=512)&&self.criteria.iter().map(|s|s.len()).sum::<usize>()<=4096&&self.allowance.valid()&&self.expires_at>0&&self.expires_at<=8_640_000_000_000_000&&(1..=8).contains(&self.max_depth)}
 pub(super)fn payload(&self,pet:&str)->Payload{Payload{schema_version:1,subgoal:self.subgoal.clone(),criteria:self.criteria.clone(),allowance:self.allowance.clone(),max_depth:self.max_depth,ancestors:vec![pet.into()]}}
 pub(super)fn matches(&self,e:&Envelope)->bool{self.valid()&&e.delegation.as_ref().is_some_and(|p|if e.schema_version==7||e.schema_version==16&&p.schema_version==2{p.schema_version==2&&p.valid_for(e)&&p.subgoal==self.subgoal&&p.criteria==self.criteria&&p.allowance==self.allowance&&p.max_depth==self.max_depth}else{p==&self.payload(&e.origin.pet_id)})}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Payload{pub schema_version:u32,pub subgoal:String,pub criteria:Vec<String>,pub allowance:Allowance,pub max_depth:u8,pub ancestors:Vec<String>}
impl Payload{
 pub(super)fn valid_for(&self,e:&Envelope)->bool{(if e.schema_version==7||e.schema_version==16&&self.schema_version==2{let mut seen=std::collections::BTreeSet::new();self.schema_version==2&&(2..=8).contains(&self.ancestors.len())&&self.ancestors.len()<=usize::from(self.max_depth)&&self.ancestors.last()==Some(&e.origin.pet_id)&&self.ancestors.iter().all(|p|crate::pet_config::blueprint(p).is_some()&&p!=&e.recipient.pet_id&&seen.insert(p))&&(if e.schema_version==7{e.reply_to.as_ref().is_some_and(|r|r.valid())}else{e.planned_data.as_ref().and_then(|p|p.source.received_request.as_ref()).is_some_and(|r|matches!(r.schema_version,2|4|7)&&r.source.recipient_pet_id==e.origin.pet_id)})}else{self.schema_version==1&&self.ancestors==vec![e.origin.pet_id.clone()]})&&e.origin.pet_id!=e.recipient.pet_id&&self.subgoal==e.text&&Arguments{to_pet_id:e.recipient.pet_id.clone(),subgoal:self.subgoal.clone(),criteria:self.criteria.clone(),allowance:self.allowance.clone(),expires_at:e.deadline_at,max_depth:self.max_depth}.valid()}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(tag="phase",rename_all="camelCase",deny_unknown_fields)]
pub enum State{Pending,Bound{binding:super::funding::ChildBinding},Withdrawn{at:u64,#[serde(rename="reviewId")]review_id:String},CancelRequested{binding:super::funding::ChildBinding,cancellation:Cancellation},Settled{binding:super::funding::ChildBinding,cancellation:Option<Cancellation>,settlement:Settlement}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Cancellation{pub at:u64,pub review_id:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Settlement{pub at:u64,pub review_id:String,pub child_grant_hash:String,pub child_revision:String,pub consumed:Allowance,pub unknown:Allowance,pub closure:Closure}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Closure{OriginalNativeClosure,RevokedNeverClaimed}
impl Cancellation{fn valid(&self,r:&Reservation,b:&Grant)->bool{self.at>=r.reserved_at&&self.at<=b.updated_at&&model::token(&self.review_id)}}
impl Settlement{fn valid(&self,r:&Reservation,b:&Grant)->bool{let mut total=self.consumed.clone();self.at>=r.reserved_at&&self.at<=b.updated_at&&model::token(&self.review_id)&&super::authority::decode(&self.child_grant_hash).is_some()&&model::token(&self.child_revision)&&self.consumed.balance_valid()&&self.unknown.balance_valid()&&total.add(&self.unknown).is_some()&&total.fits(&r.allowance)}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reservation{pub schema_version:u32,pub message_id:String,pub scope_hash:String,pub recipient_pet_id:String,pub allowance:Allowance,pub deadline_at:u64,pub reserved_at:u64,pub state:State}
impl Reservation{
 pub(super)fn capture(scope:&Scope,now:u64)->model::Result<Self>{let e=&scope.envelope;let p=e.delegation.as_ref().ok_or(model::Error::InvalidInput)?;Ok(Self{schema_version:1,message_id:e.message_id.clone(),scope_hash:model::fingerprint(scope)?,recipient_pet_id:e.recipient.pet_id.clone(),allowance:p.allowance.clone(),deadline_at:e.deadline_at,reserved_at:now,state:State::Pending})}
 pub(super)fn valid_for(&self,b:&Grant)->bool{
  let Some(r)=b.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.envelope().message_id==self.message_id))else{return false;};let e=r.envelope();
  (self.schema_version==(match self.state{State::Bound{..}=>2,State::CancelRequested{..}|State::Settled{..}=>3,_=>1})||matches!(self.schema_version,4|5|6|7|8)&&self.binding().is_some())&&matches!(e.schema_version,4|7|12|14|16)&&(e.schema_version!=12||b.hybrid_pool()&&matches!(b.message_schema(),42|43|44))&&(e.schema_version!=14||b.hybrid_pool()&&b.message_schema()==45&&e.planned_data.as_ref().is_some_and(|p|p.schema_version==3))&&(e.schema_version!=16||b.hybrid_pool()&&matches!(b.message_schema(),46|47)&&e.planned_data.as_ref().is_some_and(|p|p.schema_version==4))&&e.delegation.as_ref().is_some_and(|p|p.allowance==self.allowance)&&self.scope_hash==model::fingerprint(&r.scope).unwrap_or_default()&&self.recipient_pet_id==e.recipient.pet_id&&self.reserved_at==r.received_at&&self.deadline_at==e.deadline_at&&match &self.state{State::Pending=>true,State::Bound{binding}=>binding.valid(self,b),State::Withdrawn{at,review_id}=>*at>=self.reserved_at&&*at<=b.updated_at&&model::token(review_id),State::CancelRequested{binding,cancellation}=>binding.valid(self,b)&&cancellation.valid(self,b),State::Settled{binding,cancellation,settlement}=>binding.valid(self,b)&&cancellation.as_ref().is_none_or(|c|c.valid(self,b)&&c.at<=settlement.at)&&settlement.valid(self,b)}
 }
 /// Total deduction: consumed capacity is never returned as spare allowance.
 pub(super)fn held(&self)->Allowance{match &self.state{State::Pending|State::Bound{..}|State::CancelRequested{..}=>self.allowance.clone(),State::Settled{settlement,..}=>{let mut a=settlement.consumed.clone();a.add(&settlement.unknown).expect("validated settlement");a},_=>Allowance::default()}}
 pub(super)fn binding(&self)->Option<&super::funding::ChildBinding>{match &self.state{State::Bound{binding}|State::CancelRequested{binding,..}|State::Settled{binding,..}=>Some(binding),_=>None}}
}
pub(super)fn is_pool(g:&model::Record)->bool{g.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.as_ref().is_some_and(|o|o.operation_id=="pet.delegate"))}
impl Grant{
 pub(super)fn delegated_held(&self)->Option<Allowance>{let mut a=Allowance::default();for r in &self.delegation_reservations{a.add(&r.held())?;}Some(a)}
 pub(super)fn pool_used(&self)->Option<Allowance>{
  let mut used=self.delegated_held()?;
  if let Some(l)=&self.budget_usage{for a in &l.attempts{used.add(&Allowance{model_calls:u16::from(a.kind==super::effect_budget::Kind::Model),tool_calls:u16::from(a.kind==super::effect_budget::Kind::Tool),output_bytes:a.charge()})?;}}
  Some(used)
 }
 pub(super)fn budget_remaining(&self,budget:&Budget)->Option<Allowance>{
  let used=self.pool_used()?;Some(Allowance{model_calls:budget.max_model_calls.checked_sub(used.model_calls)?,tool_calls:budget.max_tool_calls.checked_sub(used.tool_calls)?,output_bytes:budget.max_output_bytes.checked_sub(used.output_bytes)?})
 }
 pub(super)fn hybrid_pool(&self)->bool{self.peer_planning&&matches!(self.schema_version,38|39|40|41|42|43|44|45|46|47)&&self.operation==super::authority::Operation::GoalPlanning&&(if self.schema_version==47{self.funded_by.as_ref().is_some_and(|f|f.schema_version==9)}else{self.funded_by.is_none()})}
 pub(super)fn pool_valid(&self)->bool{
  let Some(budget)=&self.delegation_pool else{return self.delegation_reservations.is_empty();};
  let hybrid=self.hybrid_pool();
  if (if hybrid{budget.payment!=super::admission::Payment::LocalOnly||!budget.valid(self.approved_at)||self.planning_usage.is_none()}else{self.operation!=super::authority::Operation::GoalDeterministic||!budget.valid_runtime_pool(self.approved_at)})
   ||self.funded_by.as_ref().is_some_and(|f|!matches!(f.schema_version,5|9)||budget.deadline_at>f.source.deadline_at||budget.max_model_calls>f.allowance.model_calls||budget.max_tool_calls>f.allowance.tool_calls||budget.max_output_bytes>f.allowance.output_bytes)||self.work_disclosure.is_some()||self.delegation_reservations.len()>16{return false;}
  let mut ids=std::collections::BTreeSet::new();if self.delegation_reservations.iter().any(|r|!ids.insert(&r.message_id)||!r.valid_for(self)){return false;}
  if self.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().delegation.is_some()&&!ids.contains(&r.envelope().message_id))){return false;}
  if !hybrid&&self.budget_usage.as_ref().is_some_and(|l|l.attempts.iter().any(|a|a.kind==super::effect_budget::Kind::Model)){return false;}
  self.pool_used().is_some_and(|used|used.within(budget))
 }
}

/// Structural causal linkage only; Authority re-derives ancestry under its writer.
pub(super)fn matches_parent(child:&Envelope,parent:&Envelope)->bool{
 child.schema_version==7&&child.kind==super::messages::Kind::Delegation&&matches!(parent.schema_version,4|7)&&parent.kind==super::messages::Kind::Delegation
 &&child.origin.pet_id==parent.recipient.pet_id&&child.reply_to.as_ref().is_some_and(|r|r.decision_id==parent.origin.decision_id&&r.message_id==parent.message_id)&&child.correlation_id==parent.correlation_id
 &&child.deadline_at<=parent.deadline_at&&child.created_at>=parent.created_at&&!child.provider_use_authorized&&!parent.provider_use_authorized
 &&child.delegation.as_ref().zip(parent.delegation.as_ref()).is_some_and(|(c,p)|{let mut ancestors=p.ancestors.clone();ancestors.push(child.origin.pet_id.clone());c.schema_version==2&&c.ancestors==ancestors&&c.max_depth<=p.max_depth&&c.ancestors.len()<=usize::from(c.max_depth)&&!c.ancestors.contains(&child.recipient.pet_id)})
}
