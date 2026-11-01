//! Original native review/choice. Only a signed previously approved closure
//! can construct its cleanup-only continuation after restart.
use super::{authority::{Authority,Grant,GrantPhase},native_owner::AbsentLease,recovery::{self,Closure,Reference},reconciliation::Snapshot,model::{self,Record,State},store::{Store,nonce}};
use serde::Serialize;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{
    pub goal_id:String,pub pet_id:String,pub revision:String,pub request_id:String,pub intent:String,pub criteria:Vec<String>,pub provider:String,pub lease_id:String,
    pub approval_scope:&'static str,pub resulting_state:State,pub snapshot:serde_json::Value,pub prepared_at:u64,pub expires_at:u64,pub execution_available:bool,
}
pub(super)struct Review{goal:Record,authority:Authority,gate:Option<AbsentLease>,body:Grant,snapshot:Snapshot,fence:String,proposal:Proposal,id:String,target_revision:String}
pub(super)struct Approved{pub(super)goal:Record,pub(super)authority:Authority,pub(super)gate:Option<AbsentLease>,pub(super)closure:Closure,pub(super)candidate:Record,pub(super)fence:String,pub(super)write_started:bool}
pub(super)struct Acknowledged{authority:Authority,reference:Reference}
pub(super)enum Prepared{Review(Review),Closed(model::Receipt)}
impl Review{
    pub(super)fn captured(goal:Record,authority:Authority,gate:Option<AbsentLease>,body:Grant,snapshot:Snapshot,fence:String,request:String,now:u64)->model::Result<Self>{
        let lease=body.lease.as_ref().ok_or(model::Error::Conflict)?;
        if !snapshot.scan_complete||!model::key(&request)||now<goal.updated_at{return Err(model::Error::Conflict);}
        let unclaimed=matches!(goal.state,State::Queued|State::Paused|State::Cancelled)&&goal.admissions.last().is_some_and(|a|a.lease.is_none())&&snapshot.attempts.is_empty();
        if !snapshot.source_consistent&&!unclaimed{return Err(model::Error::Conflict);}
        if !matches!(goal.state,State::Queued|State::Paused|State::Leased|State::Stopping|State::RecoveryRequired|State::Cancelled|State::Completed){return Err(model::Error::Conflict);}
        if body.recovery.is_none()&&goal.events.iter().any(|e|e.request_id==request){return Err(model::Error::Conflict);}
        let target=body.recovery.as_ref().map(|c|c.reference.target_state).unwrap_or_else(||recovery::target(&goal,body.stop_reason==Some(super::leases::StopReason::Cancel)));
        let proposal=Proposal{goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),revision:goal.revision.clone(),request_id:body.recovery.as_ref().map(|c|c.reference.request_id.clone()).unwrap_or(request),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),provider:lease.provider.clone(),lease_id:lease.id.clone(),approval_scope:if body.recovery.is_some(){"ackRecordedClosure"}else{"closeLostExecution"},resulting_state:target,snapshot:serde_json::to_value(&snapshot).map_err(|_|model::Error::InvalidRecord)?,prepared_at:now,expires_at:now.checked_add(30_000).ok_or(model::Error::ClockRollback)?,execution_available:false};
        let target_revision=if target==State::Completed{goal.revision.clone()}else{nonce()?};Ok(Self{goal,authority,gate,body,snapshot,fence,proposal,id:nonce()?,target_revision})
    }
    pub(super)fn proposal(&self)->Proposal{self.proposal.clone()}
    pub(super)fn recorded_target(&self)->Option<&str>{self.body.recovery.as_ref().map(|c|c.reference.target_goal_revision.as_str())}
    pub(super)fn decide(&self,now:u64)->model::Result<(Closure,Record)>{
        if now<self.proposal.prepared_at||now>8_640_000_000_000_000||self.body.recovery.is_none()&&now>=self.proposal.expires_at{return Err(model::Error::Conflict);}
        if let Some(gate)=&self.gate{gate.current(&self.authority)?;}else if self.body.recovery.is_none(){return Err(model::Error::Conflict);}
        let row=self.goal.admissions.last().ok_or(model::Error::Conflict)?;let lease=self.body.lease.as_ref().ok_or(model::Error::Conflict)?;
        let mut candidate=self.goal.clone();
        let closure=if let Some(c)=&self.body.recovery{candidate.lost_owner_closed(&c.reference,lease)?;if model::fingerprint(&candidate)?!=c.target_goal_hash{return Err(model::Error::Conflict);}c.clone()}else{
            let r=Reference{schema_version:1,id:self.id.clone(),request_id:self.proposal.request_id.clone(),goal_id:self.goal.id.clone(),pet_id:self.goal.spec.owner.pet_id.clone(),decision_id:row.spec.decision_id.clone(),lease_id:lease.id.clone(),source_goal_revision:self.goal.revision.clone(),source_goal_hash:model::fingerprint(&self.goal)?,source_sequence:self.goal.sequence,target_goal_revision:self.target_revision.clone(),target_sequence:self.goal.sequence+u64::from(self.proposal.resulting_state!=State::Completed),target_state:self.proposal.resulting_state,source_grant_revision:self.body.revision.clone(),source_grant_hash:model::fingerprint(&self.body)?,prepared_at:self.proposal.prepared_at,approved_at:now,unclaimed:row.lease.is_none()};
            candidate.lost_owner_closed(&r,lease)?;let base=closed_base(&self.body,now);
            Closure{schema_version:1,reference:r,target_goal_hash:model::fingerprint(&candidate)?,closed_base_hash:model::fingerprint(&base)?,usage_hash:model::fingerprint(&(&self.body.budget_usage,&self.body.planning_usage,&self.body.resource_usage))?,evidence_hash:model::fingerprint(&self.snapshot)?,owner_hash:model::fingerprint(self.body.runtime_owner.as_ref().ok_or(model::Error::Conflict)?)?}
        };
        Ok((closure,candidate))
    }
    pub(super)fn approve(self,decision:(Closure,Record))->Approved{
        let write_started=self.body.recovery.is_some();Approved{goal:self.goal,authority:self.authority,gate:self.gate,closure:decision.0,candidate:decision.1,fence:self.fence,write_started}
    }
}
pub(super)fn closed_base(body:&Grant,now:u64)->Grant{
    let mut closed=body.clone();closed.recovery=None;
    closed.phase=if body.phase==GrantPhase::Revoked||body.stop_reason==Some(super::leases::StopReason::Cancel){GrantPhase::Revoked}else{GrantPhase::Closed};closed.closed_at=body.closed_at.or(Some(now));closed.updated_at=now;closed
}
impl Approved{
    pub(super)fn confirm(&mut self,store:&Store,now:u64)->model::Result<model::Receipt>{store.confirm_recovery(self,now)}
    pub(super)fn acknowledged(self)->Acknowledged{Acknowledged{authority:self.authority,reference:self.closure.reference}}
}
impl Acknowledged{pub(super)fn receipt(&self,store:&Store)->model::Result<model::Receipt>{store.ack_recovery(&self.authority,&self.reference)}}
