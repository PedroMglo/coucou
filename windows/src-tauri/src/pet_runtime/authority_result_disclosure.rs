//! Authenticate selected own derived DATA under the existing Authority. Never
//! creates an admission, reserve, recipient grant or message.
use super::*;
use crate::pet_runtime::{derived_result_plan::Arguments,results::{self,ModelResult},messages::Envelope};
impl Authority{
 pub(in crate::pet_runtime)fn capture_result_disclosure(&self,g:&Record,args:&Arguments,now:u64)->Result<(ModelResult,Envelope)>{let _guard=self.lock_shared()?;self.result_disclosure_unlocked(g,args,now)}
 pub(in crate::pet_runtime)fn result_disclosure_unlocked(&self,g:&Record,args:&Arguments,now:u64)->Result<(ModelResult,Envelope)>{
  g.validate().map_err(|_|Failure::Invalid)?;let last=g.admissions.last().filter(|r|r.spec.funded_response()).ok_or(Failure::Changed)?;
  if g.state!=State::Paused||now<g.updated_at||now>=args.expires_at{return Err(Failure::Changed);}
  let b=self.load(&last.spec,last.proof.as_ref().ok_or(Failure::Missing)?)?;
  if b.phase!=GrantPhase::Closed||b.stop_reason!=Some(crate::pet_runtime::leases::StopReason::Pause)||b.closed_at.is_none(){return Err(Failure::Changed);}
  self.funding_current_unlocked(&last.spec,now)?;args.verify_unlocked(self,g)?;
  let row=g.admissions.iter().find(|r|r.spec.decision_id==args.result.decision_id).ok_or(Failure::Changed)?;
  let results::Lookup::Available{receipt}=results::lookup_unlocked(self,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,&args.result.request_id)?else{return Err(Failure::RecoveryRequired);};
  let received=self.received_source_unlocked(&args.source,&g.spec.owner.pet_id,now)?;
  if received.envelope().schema_version!=(if last.spec.schema_version==19{7}else{4})||received.envelope().origin.pet_id!=args.to_pet_id{return Err(Failure::Changed);}self.fence()?;Ok((receipt,received.envelope().clone()))
 }
 pub(in crate::pet_runtime)fn capture_derived_data(&self,g:&Record,p:&crate::pet_runtime::result_disclosure_policy::Policy,now:u64)->Result<Envelope>{let _guard=self.lock_shared()?;self.derived_data_unlocked(g,p,now)}
 pub(in crate::pet_runtime)fn derived_data_unlocked(&self,g:&Record,p:&crate::pet_runtime::result_disclosure_policy::Policy,now:u64)->Result<Envelope>{
  g.validate().map_err(|_|Failure::Invalid)?;if !p.matches(g)||now<g.updated_at||now>=p.arguments.expires_at{return Err(Failure::Changed);}p.arguments.verify_unlocked(self,g)?;
  let row=g.admissions.iter().find(|r|r.spec.decision_id==p.arguments.result.decision_id).ok_or(Failure::Changed)?;let results::Lookup::Available{receipt:r}=results::lookup_unlocked(self,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,&p.arguments.result.request_id)?else{return Err(Failure::RecoveryRequired);};
  let origin=crate::pet_runtime::result_disclosure_policy::Origin{provider:r.provider,adapter:row.spec.owner.adapter.clone(),model:r.model,inference_destination:row.spec.memory_data.as_ref().ok_or(Failure::Changed)?.destination.clone(),input_hash:r.input_hash,intent_hash:r.intent_hash,completed_at:r.completed_at,recorded_at:r.recorded_at};if origin!=p.origin{return Err(Failure::Changed);}
  let selection=g.execution_context.as_ref().and_then(|c|c.received_request.as_ref()).ok_or(Failure::Changed)?;let received=self.received_source_unlocked(&p.arguments.source,&g.spec.owner.pet_id,now)?;self.funded_context_current_unlocked(selection,&g.id,received.envelope(),now)?;
  let(parent,_)=self.grant_file(&format!("{}.json",p.source.reference.decision_id))?;if !parent.delegation_reservations.iter().any(|r|r.message_id==p.source.reference.message_id&&r.scope_hash==p.source.reference.scope_hash&&matches!(&r.state,crate::pet_runtime::delegation::State::Bound{binding}if binding.goal_id==g.id&&binding.pet_id==g.spec.owner.pet_id)){return Err(Failure::Revoked);}self.fence()?;Ok(received.envelope().clone())
 }

}
