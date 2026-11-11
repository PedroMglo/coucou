//! Authenticate delivered derived DATA, not another execution or funding grant.
use super::*;
use crate::pet_runtime::{messages::{Envelope,ReceivedReference},result_disclosure_policy::Policy,delegation::State as Branch};
impl Authority{
 pub(in crate::pet_runtime)fn received_result_purpose(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<crate::pet_runtime::received_request::Purpose>{let _guard=self.lock_shared()?;let(_,e)=self.received_result_policy_unlocked(reference,recipient,now)?;Ok(if e.schema_version==9{crate::pet_runtime::received_request::Purpose::RecursiveResult}else{crate::pet_runtime::received_request::Purpose::Result})}

 pub(in crate::pet_runtime)fn received_result_policy_unlocked(&self,reference:&ReceivedReference,recipient:&str,now:u64)->Result<(Policy,Envelope)>{
  let r=self.received_source_unlocked(reference,recipient,now)?;let e=r.envelope();let(g,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;let p=g.result_disclosure.as_ref().ok_or(Failure::Changed)?;
  if g.schema_version!=(if p.schema_version==2{34}else{25})||g.phase==GrantPhase::Revoked||g.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel)||!p.matches_envelope(e)||p.recipient.pet_id!=recipient||now<g.approved_at||now>=p.arguments.expires_at{return Err(Failure::Revoked);}
  let original=self.received_source_unlocked(&p.source.reference,&p.pet_id,now)?;
  if p.schema_version==2{if original.envelope().schema_version!=7||!crate::pet_runtime::messages::reply_matches(e,original.envelope()){return Err(Failure::Changed);}self.delegation_result_lineage_unlocked(&p.source,now)?;}
  if received_request::Witness::fact(&p.source.reference,&original)?!=p.source{return Err(Failure::Changed);}
  let(parent,_)=self.grant_file(&format!("{}.json",p.source.reference.decision_id))?;
  let branch=parent.delegation_reservations.iter().find(|b|b.message_id==p.source.reference.message_id&&b.scope_hash==p.source.reference.scope_hash).ok_or(Failure::Changed)?;
  let binding=match &branch.state{Branch::Bound{binding}|Branch::Settled{binding,cancellation:None,..}=>binding,_=>return Err(Failure::Revoked)};
  if parent.phase==GrantPhase::Revoked||parent.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel)||binding.goal_id!=p.goal_id||binding.pet_id!=p.pet_id||binding.decision_id!=g.decision_id||binding.admission_hash!=g.spec_hash{return Err(Failure::Revoked);}self.fence()?;Ok((p.clone(),e.clone()))
 }
}
