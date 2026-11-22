//! Branch controls reduce authority in the original signed source grant.
//! Parent intent precedes child Stop; two durable files are NOT atomic.
use super::*;
use crate::pet_runtime::{delegation::{Reservation,State as Branch,Allowance,Cancellation,Settlement,Closure},funding::ChildBinding,leases::{Lease,StopReason},effect_budget::{Kind,Phase},authority_store::delegation::{View,view}};
use std::{sync::Arc,time::{Instant,Duration}};
pub use crate::pet_runtime::delegation::BranchAction as Action;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{pub schema_version:u32,pub action:Action,pub pet_id:String,pub goal_id:String,pub decision_id:String,pub reservation:Reservation,pub source_grant_hash:String,pub child_grant_hash:String,pub child_revision:String,pub child_phase:GrantPhase,pub consumed:Option<Allowance>,pub unknown:Option<Allowance>,pub review_id:String,pub expires_at:u64,pub scope:&'static str}
pub(in crate::pet_runtime)struct Candidate{authority:Authority,goal:Record,spec:AdmissionSpec,proof:ConsentProof,pub proposal:Proposal,expires:Instant}
/// A backend-created continuation of persisted cancellation. It contains only
/// the original canonical binding, never a caller-supplied target or spec.
pub(in crate::pet_runtime)struct StopTarget{pub binding:ChildBinding,pub lease:Option<Lease>,pub reason:StopReason,pub descendants:Vec<StopTarget>}
pub(in crate::pet_runtime)struct Confirmed{pub view:View,pub stop:Option<StopTarget>}
fn accounting(b:&Grant,r:&Reservation)->Result<(Allowance,Allowance,Closure)>{
 let closure=if b.phase==GrantPhase::Revoked&&b.never_claimed_zero(){Closure::RevokedNeverClaimed}
 else if matches!(b.phase,GrantPhase::Closed|GrantPhase::Revoked)&&b.lease.is_some()&&b.closed_at.is_some(){Closure::OriginalNativeClosure}
 else{return Err(Failure::Busy);};
 let mut consumed=Allowance::default();let mut unknown=Allowance::default();
 if let Some(l)=&b.budget_usage{for a in &l.attempts{
  let known=a.phase==Phase::Settled;let amount=Allowance{model_calls:u16::from(a.kind==Kind::Model),tool_calls:u16::from(a.kind==Kind::Tool),output_bytes:if known{a.accepted_output.ok_or(Failure::Invalid)?}else{a.output_reserved}};
  (if known{&mut consumed}else{&mut unknown}).add(&amount).ok_or(Failure::Invalid)?;
 }}
 // A pool's local completion proves only its own worker/delivery. All
 // descendants must first have durable terminal reservation facts. Do not
 // infer zero or release ancestral credit from a completed local message.
 if b.delegation_pool.is_some(){for child in &b.delegation_reservations{match &child.state{
  Branch::Withdrawn{..}=>{},Branch::Settled{settlement,..}=>{consumed.add(&settlement.consumed).ok_or(Failure::Invalid)?;unknown.add(&settlement.unknown).ok_or(Failure::Invalid)?;},_=>return Err(Failure::Busy),
 }}}
 let mut total=consumed.clone();total.add(&unknown).ok_or(Failure::Invalid)?;if !total.fits(&r.allowance){return Err(Failure::Invalid);}Ok((consumed,unknown,closure))
}
impl Authority{
 pub(in crate::pet_runtime::authority_store)fn branch_child_unlocked(&self,parent:&Grant,r:&Reservation)->Result<Grant>{
  let binding=r.binding().ok_or(Failure::Changed)?;let(b,_)=self.grant_file(&format!("{}.json",binding.decision_id))?;let f=b.funded_by.as_ref().ok_or(Failure::Changed)?;
  let receipt=parent.messages.as_ref().and_then(|m|m.entries.iter().find(|e|e.envelope().message_id==r.message_id)).ok_or(Failure::Changed)?;
  let reference=crate::pet_runtime::messages::ReceivedReference::from_receipt(receipt).map_err(|_|Failure::Invalid)?;
  let e=receipt.envelope();let source=crate::pet_runtime::received_request::Source{reference,sender_pet_id:e.origin.pet_id.clone(),sender_goal_id:e.origin.goal_id.clone(),sender_conversation_id:e.origin.conversation_id.clone(),recipient_pet_id:e.recipient.pet_id.clone(),receipt_hash:model::fingerprint(receipt).map_err(|_|Failure::Invalid)?,envelope_hash:model::fingerprint(e).map_err(|_|Failure::Invalid)?,received_at:receipt.received_at,deadline_at:e.deadline_at};
  if b.goal_id!=binding.goal_id||b.decision_id!=binding.decision_id||b.spec_hash!=binding.admission_hash||f.source!=source||f.allowance!=r.allowance||binding.pet_id!=f.source.recipient_pet_id||!matches!(b.operation,Operation::GoalDeterministic|Operation::ConversationRespond)&&!(f.schema_version==9&&b.operation==Operation::GoalPlanning)||b.operation==Operation::ConversationRespond&&b.schema_version!=b.funded_schema()||(if f.schema_version==5{!matches!(b.schema_version,28|32|35)||b.delegation_pool.is_none()}else if f.schema_version==9{b.schema_version!=47||!b.hybrid_pool()||b.delegation_pool.is_none()}else{b.delegation_pool.is_some()||!b.delegation_reservations.is_empty()}){return Err(Failure::Changed);}Ok(b)
 }
 fn branch_settlement_unlocked(&self,parent:&Grant,r:&Reservation)->Result<(Grant,Allowance,Allowance,Closure)>{
  let b=self.branch_child_unlocked(parent,r)?;
  // Unpublished images may contain effects absent from the canonical ledger.
  // Neither expiry nor a missing goal projection is evidence of zero IO.
  let prefix=self.funding_prefix_unlocked(b.funded_by.as_ref().ok_or(Failure::Changed)?,&b.goal_id)?;
  let(grants,pending)=self.history_inventory(&b.goal_id,None)?;
  if !pending.is_empty()||grants.values().any(|g|g.decision_id!=b.decision_id&&!prefix.iter().any(|p|p.decision_id==g.decision_id)&&!(g.phase==GrantPhase::Revoked&&g.never_claimed_zero())){return Err(Failure::RecoveryRequired);}
  let(mut known,mut unknown,closure)=accounting(&b,r)?;for old in prefix{let(k,u,_)=accounting(&old,r)?;known.add(&k).ok_or(Failure::Invalid)?;unknown.add(&u).ok_or(Failure::Invalid)?;}let mut total=known.clone();total.add(&unknown).ok_or(Failure::Invalid)?;if !total.fits(&r.allowance){return Err(Failure::Invalid);}Ok((b,known,unknown,closure))
 }
 pub(in crate::pet_runtime)fn prepare_delegation_branch(self,g:Record,decision:&str,message:&str,action:Action,now:u64)->model::Result<Candidate>{
  self.verify_goal_origin(&g).map_err(Failure::control_error)?;let row=g.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(model::Error::NotFound)?;let spec=row.spec.clone();let proof=row.proof.clone().ok_or(model::Error::Conflict)?;
  let _guard=self.lock_shared().map_err(Failure::control_error)?;let p=self.load(&spec,&proof).map_err(Failure::control_error)?;if now<p.updated_at{return Err(model::Error::ClockRollback);}let r=p.delegation_reservations.iter().find(|r|r.message_id==message&&r.binding().is_some()).ok_or(model::Error::Conflict)?.clone();
  let(child,consumed,unknown)=match action{Action::Cancel=>{let c=self.branch_child_unlocked(&p,&r).map_err(Failure::control_error)?;(c,None,None)},Action::Reconcile=>{let(c,k,u,_)=self.branch_settlement_unlocked(&p,&r).map_err(Failure::control_error)?;(c,Some(k),Some(u))}};
  let subtree=child.delegation_pool.is_some();let proposal=Proposal{schema_version:if subtree{2}else{1},action,pet_id:g.spec.owner.pet_id.clone(),goal_id:g.id.clone(),decision_id:decision.into(),reservation:r,source_grant_hash:model::fingerprint(&p)?,child_grant_hash:model::fingerprint(&child)?,child_revision:child.revision.clone(),child_phase:child.phase,consumed,unknown,review_id:nonce()?,expires_at:now.checked_add(120_000).ok_or(model::Error::ClockRollback)?,scope:if subtree{"exactBoundSubtree;persistIntentThenStop;noNewAuthority;descendantUnknownConsumptionRetained"}else{"exactBoundChildOnly;noNewExecutionAuthority;unknownConsumptionRetained"}};drop(_guard);
  Ok(Candidate{authority:self,goal:g,spec,proof,proposal,expires:Instant::now()+Duration::from_secs(120)})
 }
}
impl Candidate{
 fn fact(&self,r:&Reservation)->bool{match (&self.proposal.action,&r.state){(Action::Cancel,Branch::CancelRequested{..}|Branch::Settled{..})=>true,(Action::Reconcile,Branch::Settled{settlement,..})=>settlement.child_grant_hash==self.proposal.child_grant_hash,_=>false}}
 pub(in crate::pet_runtime)fn observed(&self,now:u64)->model::Result<Option<View>>{
  let a=&self.authority;let _guard=a.lock_shared().map_err(Failure::control_error)?;let p=a.load(&self.spec,&self.proof).map_err(Failure::control_error)?;let r=p.delegation_reservations.iter().find(|r|r.message_id==self.proposal.reservation.message_id).ok_or(model::Error::Conflict)?;
  if self.fact(r){Ok(Some(view(&self.goal,&p,now,false)?))}else{Ok(None)}
 }
 pub(in crate::pet_runtime)fn confirm(&self,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Confirmed>{
  let a=&self.authority;let _guard=a.lock().map_err(Failure::control_error)?;let mut p=a.load(&self.spec,&self.proof).map_err(Failure::control_error)?;
  let index=p.delegation_reservations.iter().position(|r|r.message_id==self.proposal.reservation.message_id).ok_or(model::Error::Conflict)?;let mut r=p.delegation_reservations[index].clone();let fact=self.fact(&r);
  if now<p.updated_at{return Err(model::Error::ClockRollback);}
  if !fact&&(r!=self.proposal.reservation||now>=self.proposal.expires_at||Instant::now()>=self.expires||!human()){return Err(model::Error::Conflict);}
  if fact{a.stabilize(&p).map_err(Failure::control_error)?;}
  let binding=r.binding().ok_or(model::Error::Conflict)?.clone();
  match self.proposal.action{
   Action::Cancel=>{
    if matches!(r.state,Branch::Settled{..}){return Ok(Confirmed{view:view(&self.goal,&p,now,false)?,stop:None});}
    if !fact{r.schema_version=r.schema_version.max(3);r.state=Branch::CancelRequested{binding:binding.clone(),cancellation:Cancellation{at:now,review_id:self.proposal.review_id.clone()}};p.delegation_reservations[index]=r.clone();p.updated_at=now;p.revision=nonce()?;p.schema_version=p.native_schema();
     a.publish_guarded(p.clone(),false,||if human()&&Instant::now()<self.expires{Ok(())}else{Err(Failure::Revoked)}).map_err(Failure::control_error)?;
    }
    // New effects are now fenced by parent CancelRequested, even if the
    // following child write/projection fails or the caller loses its ACK.
    let mut child=a.branch_child_unlocked(&p,&r).map_err(Failure::control_error)?;if now<child.updated_at{return Err(model::Error::ClockRollback);}let lease=child.lease.clone();
    if let Some(l)=&lease{crate::pet_runtime::lease_controller::signal_stop(l);}
    if !matches!(child.phase,GrantPhase::Closed|GrantPhase::Revoked)&&child.native_completion.is_none()&&!(child.phase==GrantPhase::Draining&&child.stop_reason==Some(StopReason::Cancel)){
     if child.phase==GrantPhase::Issued{child.phase=GrantPhase::Revoked;}else{child.phase=GrantPhase::Draining;child.stop_reason=Some(StopReason::Cancel);}
     child.updated_at=now;child.revision=nonce()?;a.publish(child.clone(),false).map_err(Failure::control_error)?;
    }else{a.stabilize(&child).map_err(Failure::control_error)?;}
    let descendants=a.cancel_descendants_unlocked(&p,&r,&child,now).map_err(Failure::control_error)?;
    Ok(Confirmed{view:view(&self.goal,&p,now,false)?,stop:Some(StopTarget{binding,lease,reason:StopReason::Cancel,descendants})})
   },
   Action::Reconcile=>{
    if fact{return Ok(Confirmed{view:view(&self.goal,&p,now,false)?,stop:None});}
    let(child,consumed,unknown,closure)=a.branch_settlement_unlocked(&p,&r).map_err(Failure::control_error)?;
    if model::fingerprint(&child)?!=self.proposal.child_grant_hash||child.revision!=self.proposal.child_revision||Some(&consumed)!=self.proposal.consumed.as_ref()||Some(&unknown)!=self.proposal.unknown.as_ref(){return Err(model::Error::Conflict);}
    let cancellation=match &r.state{Branch::CancelRequested{cancellation,..}=>Some(cancellation.clone()),Branch::Settled{cancellation,..}=>cancellation.clone(),_=>None};
    r.schema_version=r.schema_version.max(3);r.state=Branch::Settled{binding,cancellation,settlement:Settlement{at:now,review_id:self.proposal.review_id.clone(),child_grant_hash:self.proposal.child_grant_hash.clone(),child_revision:child.revision,consumed,unknown,closure}};
    p.delegation_reservations[index]=r;p.updated_at=now;p.revision=nonce()?;p.schema_version=p.native_schema();
    a.publish_guarded(p.clone(),false,||if human()&&Instant::now()<self.expires{Ok(())}else{Err(Failure::Revoked)}).map_err(Failure::control_error)?;
    Ok(Confirmed{view:view(&self.goal,&p,now,false)?,stop:None})
   }
  }
 }
}
