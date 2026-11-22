//! Canonical unique binding in the existing parent Grant. Child grant intent
//! is durable first, but cannot queue/claim/spend until this binding is durable.
use super::*;
use crate::pet_runtime::{funding::ChildBinding,delegation::State as ReservationState};
impl Authority{
 /// Waiting is a zero-worker resource transition, not withdrawal of existing
 /// child reservations. Only an authenticated, live peer pool wait qualifies;
 /// paused/revoked/expired parents remain unable to fund a new child effect.
 pub(in crate::pet_runtime::authority_store)fn delegation_parent_available(parent:&Grant,now:u64)->bool{
  match parent.phase{
   GrantPhase::Claimed=>true,
   GrantPhase::Closed=>parent.native_completion.is_some(),
   GrantPhase::Waiting=>parent.hybrid_pool()&&parent.stop_reason.is_none()&&parent.lease.as_ref().is_some_and(|l|now>=l.claimed_at&&now<l.deadline_at&&parent.resource_usage.as_ref().is_some_and(|u|u.valid(l,parent.updated_at)&&u.entries.last().is_some_and(|e|e.resumed_at.is_none()&&now>=e.yielded_at&&now<e.expires_at))),
   _=>false,
  }
 }

 /// A signed absence closure stops the old parent, not an independently
 /// admitted exact child. No pending allocation or successor admission enters
 /// through this path. Explicit pause/cancel, revocation and expiry still stop.
 pub(in crate::pet_runtime::authority_store)fn recovered_parent_bound_available(parent:&Grant,r:&crate::pet_runtime::delegation::Reservation,goal:Option<&str>,now:u64)->bool{
  parent.phase==GrantPhase::Closed&&parent.stop_reason.is_none()&&parent.recovery.as_ref().is_some_and(|c|c.valid_for(parent)&&c.reference.target_state==crate::pet_runtime::model::State::RecoveryRequired)
   &&parent.lease.as_ref().is_some_and(|l|now>=l.claimed_at&&now<l.deadline_at)&&now>=parent.updated_at&&now<r.deadline_at
   &&matches!(&r.state,ReservationState::Bound{binding}if Some(binding.goal_id.as_str())==goal&&binding.pet_id==r.recipient_pet_id)
 }
 /// Authenticate the exact prefix stored in the successor's signed Grant.
 /// This is history, never another allowance or a source of execution rights.
 pub(in crate::pet_runtime::authority_store)fn funding_prefix_unlocked(&self,f:&crate::pet_runtime::funding::Funding,goal:&str)->Result<Vec<Grant>>{
  if !f.basic(){return Err(Failure::Invalid);}let mut out=vec![];
  for(index,a)in f.previous.iter().enumerate(){
   let(b,_)=self.grant_file(&format!("{}.json",a.decision_id))?;let old=b.funded_by.as_ref().ok_or(Failure::Changed)?;
   if b.goal_id!=goal||b.spec_hash!=a.spec_hash||model::fingerprint(&b).ok().as_ref()!=Some(&a.grant_hash)||b.phase!=GrantPhase::Closed||b.stop_reason!=Some(crate::pet_runtime::leases::StopReason::Pause)||b.closed_at.is_none()||b.lease.is_none()||(if f.schema_version==4{b.operation!=Operation::GoalDeterministic||!matches!(a.native_schema,Some(11|16))}else{b.operation!=Operation::ConversationRespond})||f.schema_version==8&&(old.schema_version!=7||a.native_schema.is_some()||f.previous.len()!=1)||old.source!=f.source||old.allowance!=f.allowance||old.own_origin_hash!=f.own_origin_hash||old.previous!=f.previous[..index]{return Err(Failure::Changed);}out.push(b);
  }self.fence()?;Ok(out)
 }
 fn funding_binding_predecessor(s:&AdmissionSpec,b:&ChildBinding)->bool{
  s.funding.as_ref().is_some_and(|f|matches!(f.schema_version,2|3|4|8)&&f.previous.last().is_some_and(|a|b.goal_id==s.goal_id&&b.pet_id==s.owner.pet_id&&b.decision_id==a.decision_id&&b.admission_hash==a.spec_hash))
 }
 /// Selected delegation DATA is a separate scope from the old request reader.
 /// This method also runs inside final effect writers: do not re-enter flock.
 pub(in crate::pet_runtime)fn funded_context_current_unlocked(&self,selection:&crate::pet_runtime::received_context::Selection,goal:&str,envelope:&crate::pet_runtime::messages::Envelope,now:u64)->Result<()>{
  if !matches!(selection.schema_version,2|4|7){return Ok(());}let source=&selection.source;if selection.schema_version==4{self.delegation_lineage_unlocked(source,Some(goal),now)?;}let(p,_)=self.grant_file(&format!("{}.json",source.reference.decision_id))?;
  let d=envelope.delegation.as_ref().ok_or(Failure::Invalid)?;
  if p.goal_id!=source.sender_goal_id||now<p.updated_at||now>=source.deadline_at{return Err(Failure::Revoked);}
  let r=p.delegation_reservations.iter().find(|r|r.message_id==source.reference.message_id&&r.scope_hash==source.reference.scope_hash&&r.recipient_pet_id==source.recipient_pet_id&&r.allowance==d.allowance).ok_or(Failure::Changed)?;
  if !Self::delegation_parent_available(&p,now)&&!Self::recovered_parent_bound_available(&p,r,Some(goal),now){return Err(Failure::Revoked);}
  if !matches!(&r.state,ReservationState::Pending)&&!matches!(&r.state,ReservationState::Bound{binding}if binding.goal_id==goal&&binding.pet_id==source.recipient_pet_id){return Err(Failure::Revoked);}self.fence()
 }
 fn funding_source_unlocked(&self,s:&AdmissionSpec,now:u64)->Result<Grant>{
  let f=s.funding.as_ref().ok_or(Failure::Invalid)?;if !f.valid_for(s){return Err(Failure::Invalid);}let(p,_)=self.grant_file(&format!("{}.json",f.source.reference.decision_id))?;
  if p.goal_id!=f.source.sender_goal_id||now<p.updated_at||now>=f.source.deadline_at{return Err(Failure::Revoked);}
  let held=p.delegation_reservations.iter().find(|r|r.message_id==f.source.reference.message_id&&r.scope_hash==f.source.reference.scope_hash&&r.allowance==f.allowance).ok_or(Failure::Changed)?;
  if !Self::delegation_parent_available(&p,now)&&!(Self::recovered_parent_bound_available(&p,held,Some(&s.goal_id),now)&&held.binding().is_some_and(|b|b.matches(s))){return Err(Failure::Revoked);}
  let r=self.received_source_unlocked(&f.source.reference,&s.owner.pet_id,now)?;
  if received_request::Witness::fact(&f.source.reference,&r)?!=f.source||!matches!(r.envelope().schema_version,4|7|12|14|16)||r.envelope().delegation.as_ref().is_none_or(|d|d.allowance!=f.allowance){return Err(Failure::Changed);}if matches!(r.envelope().schema_version,12|14|16){self.received_planned_data_unlocked(&f.source.reference,&s.owner.pet_id,now)?;}self.funding_prefix_unlocked(f,&s.goal_id)?;self.delegation_lineage_unlocked(&f.source,Some(&s.goal_id),now)?;Ok(p)
 }
 pub(in crate::pet_runtime::authority_store)fn funding_preflight_unlocked(&self,s:&AdmissionSpec,now:u64)->Result<()>{
  self.rule_admission_current_unlocked(s,now)?;if s.funding.is_none(){return Ok(());}let p=self.funding_source_unlocked(s,now)?;let f=s.funding.as_ref().unwrap();let r=p.delegation_reservations.iter().find(|r|r.message_id==f.source.reference.message_id&&r.scope_hash==f.source.reference.scope_hash&&r.allowance==f.allowance).ok_or(Failure::Changed)?;
  if !(matches!(f.schema_version,1|5|6|7|9)&&matches!(&r.state,ReservationState::Pending))&&!matches!(&r.state,ReservationState::Bound{binding}if binding.matches(s)||Self::funding_binding_predecessor(s,binding)){return Err(Failure::Revoked);}Ok(())
 }
 pub(in crate::pet_runtime::authority_store)fn funding_current_unlocked(&self,s:&AdmissionSpec,now:u64)->Result<()>{
  self.rule_admission_current_unlocked(s,now)?;if s.funding.is_none(){return Ok(());}let p=self.funding_source_unlocked(s,now)?;let f=s.funding.as_ref().unwrap();let r=p.delegation_reservations.iter().find(|r|r.message_id==f.source.reference.message_id&&r.scope_hash==f.source.reference.scope_hash&&r.allowance==f.allowance).ok_or(Failure::Changed)?;
  if !matches!(&r.state,ReservationState::Bound{binding}if binding.matches(s)){return Err(Failure::Revoked);}self.fence()
 }
 /// Same directory writer is held. Never overwrites a sibling snapshot. A
 /// canonical child grant is intent only until parent binding + queued goal.
 pub(in crate::pet_runtime::authority_store)fn bind_funding_unlocked(&self,s:&AdmissionSpec,proof:&ConsentProof,now:u64)->Result<()>{
  self.rule_admission_current_unlocked(s,now)?;if s.funding.is_none(){return Ok(());}let c=self.load(s,proof)?;if c.phase!=GrantPhase::Issued||c.lease.is_some(){return Err(Failure::Revoked);}let mut p=self.funding_source_unlocked(s,now)?;let f=s.funding.as_ref().unwrap();let r=p.delegation_reservations.iter_mut().find(|r|r.message_id==f.source.reference.message_id&&r.scope_hash==f.source.reference.scope_hash&&r.allowance==f.allowance).ok_or(Failure::Changed)?;
  match &r.state{ReservationState::Bound{binding}if binding.matches(s)=>return self.stabilize(&p),ReservationState::Pending if matches!(f.schema_version,1|5|6|7|9)=>{},ReservationState::Bound{binding}if Self::funding_binding_predecessor(s,binding)=>{},_=>return Err(Failure::Revoked)}
  r.schema_version=if f.schema_version==8{8}else if f.schema_version==7{7}else if matches!(f.schema_version,5|6){6}else if f.schema_version==4{5}else if matches!(f.schema_version,2|3){4}else{2};r.state=ReservationState::Bound{binding:ChildBinding::capture(s,now).map_err(|_|Failure::Invalid)?};p.updated_at=now;p.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(p,false)?;self.funding_current_unlocked(s,now)
 }
 pub(in crate::pet_runtime)fn delegation_pending_unlocked(&self,reference:&crate::pet_runtime::messages::ReceivedReference,recipient:&str,now:u64)->Result<()>{
  let(p,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;if now<p.updated_at||!Self::delegation_parent_available(&p,now){return Err(Failure::Revoked);}
  if !p.delegation_reservations.iter().any(|r|r.message_id==reference.message_id&&r.scope_hash==reference.scope_hash&&r.recipient_pet_id==recipient&&r.state==ReservationState::Pending&&now<r.deadline_at){return Err(Failure::Revoked);}Ok(())
 }
}
