//! Native retained choice for reducing unused authority, never for execution.
use super::{authority::{Authority,Operation},authority_store::UnusedSource,retirement::{Reference,Receipt},model::{self,Record},history::Charge,store::{Store,nonce}};
use serde::Serialize;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{
    pub goal_id:String,pub pet_id:String,pub revision:String,pub request_id:String,pub decision_id:String,pub intent:String,pub criteria:Vec<String>,
    pub approval_scope:&'static str,pub original_operation:&'static str,pub original_spec_hash:String,pub original_approved_at:u64,pub source_grant_revision:String,pub source_grant_hash:String,
    pub authenticated_subtotal:Charge,pub linked_accounting_complete:bool,pub history_continuity_verified:bool,pub target_usage_known_zero:bool,
    pub same_goal_unlinked:usize,pub matching_staged_images:usize,pub prepared_at:u64,pub expires_at:u64,pub execution_available:bool,
}
pub(super)struct Review{goal:Record,authority:Authority,source:UnusedSource,proposal:Proposal,id:String}
pub(super)struct Decision{reference:Reference}
pub(super)struct Approved{pub(super)goal:Record,pub(super)authority:Authority,pub(super)source:UnusedSource,pub(super)reference:Reference,pub(super)write_started:bool}
pub(super)struct Acknowledged{authority:Authority,reference:Reference}
pub(super)enum Prepared{Review(Review),Retired(Receipt)}
impl Review{
    pub(super)fn captured(goal:Record,authority:Authority,source:UnusedSource,request:String,now:u64)->model::Result<Self>{
        if !model::key(&request)||now<goal.updated_at||now>8_640_000_000_000_000-30_000{return Err(model::Error::InvalidInput);}
        let body=source.body();let proposal=Proposal{goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),revision:goal.revision.clone(),request_id:request,decision_id:body.decision_id.clone(),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),approval_scope:"retireUnlinkedAdmission",original_operation:if body.operation==Operation::GoalPlanning{"goalPlanning"}else{"conversationRespond"},original_spec_hash:body.spec_hash.clone(),original_approved_at:body.approved_at,source_grant_revision:body.revision.clone(),source_grant_hash:model::fingerprint(body)?,authenticated_subtotal:source.charge().clone(),linked_accounting_complete:source.linked_accounting_complete(),history_continuity_verified:false,target_usage_known_zero:true,same_goal_unlinked:source.unlinked(),matching_staged_images:source.staged(),prepared_at:now,expires_at:now+30_000,execution_available:false};
        Ok(Self{goal,authority,source,proposal,id:nonce()?})
    }
    pub(super)fn proposal(&self)->Proposal{self.proposal.clone()}
    pub(super)fn decide(&self,now:u64)->model::Result<Decision>{
        if now<self.proposal.prepared_at||now>=self.proposal.expires_at{return Err(model::Error::Conflict);}
        let reference=self.source.reference(&self.goal,&self.proposal.request_id,&self.id,self.proposal.prepared_at,now)?;
        self.authority.retirement_current(&self.goal,&self.source,&reference).map_err(super::authority::Failure::control_error)?;
        Ok(Decision{reference})
    }
    pub(super)fn approve(self,decision:Decision)->Approved{Approved{goal:self.goal,authority:self.authority,source:self.source,reference:decision.reference,write_started:false}}
}
impl Approved{
    pub(super)fn confirm(&mut self,store:&Store,now:u64)->model::Result<Receipt>{store.confirm_retirement(self,now)}
    pub(super)fn acknowledged(self)->Acknowledged{Acknowledged{authority:self.authority,reference:self.reference}}
}
impl Acknowledged{pub(super)fn receipt(&self,store:&Store)->model::Result<Receipt>{store.ack_retirement(&self.authority,&self.reference)}}
