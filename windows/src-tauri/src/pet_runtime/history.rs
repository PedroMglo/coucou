//! Authenticated cumulative facts, never an allowance or a continuation permit.
//! Missing legacy ledgers and unavailable token/cost metering are not zeroes.
use super::{admission::{Budget,QueuePhase},authority::Failure,model::{self,Record,State},reconciliation::{Attempt,Evidence,EvidenceStatus}};
use serde::{Serialize,Deserialize};

#[derive(Clone,Default,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Charge {
    pub model_calls:u64,
    pub tool_calls:u64,
    pub output_bytes:u64,
    pub unknown_model_results:u64,
    pub unknown_tool_results:u64,
}
impl Charge {
    pub(super)fn valid(&self)->bool{self.model_calls<=32*64&&self.tool_calls<=32*128&&self.output_bytes<=32*16*1024*1024&&self.unknown_model_results<=self.model_calls&&self.unknown_tool_results<=self.tool_calls}
    #[cfg(target_os="linux")]
    pub(super)fn checked_add(&mut self,other:&Self)->super::authority::Result<()> {
        let add=|a:u64,b:u64|a.checked_add(b).ok_or(Failure::Capacity);
        *self=Self{model_calls:add(self.model_calls,other.model_calls)?,tool_calls:add(self.tool_calls,other.tool_calls)?,output_bytes:add(self.output_bytes,other.output_bytes)?,unknown_model_results:add(self.unknown_model_results,other.unknown_model_results)?,unknown_tool_results:add(self.unknown_tool_results,other.unknown_tool_results)?};
        Ok(())
    }
    #[cfg(target_os="linux")]
    pub(super)fn from_attempts(attempts:&[Attempt])->super::authority::Result<Self> {
        let mut total=Self::default();
        for attempt in attempts {
            let model=attempt.kind=="model";
            total.checked_add(&Self{model_calls:u64::from(model),tool_calls:u64::from(!model),output_bytes:attempt.charged_output_bytes,unknown_model_results:u64::from(model&&!attempt.outcome_known),unknown_tool_results:u64::from(!model&&!attempt.outcome_known)})?;
        }
        Ok(total)
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum LedgerStatus{Recorded,NeverClaimed,NativeUnstarted,LegacyUnrecorded,Unavailable}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Grant {
    pub revision:String,pub hash:String,pub phase:&'static str,
    pub lease_id:Option<String>,pub goal_lease_matches:bool,pub terminal:bool,
    pub updated_at:u64,pub closed_at:Option<u64>,
    pub recovery:Option<super::recovery::Reference>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Admission {
    pub index:usize,pub decision_id:String,pub spec_hash:String,pub source_sequence:u64,
    pub queue_phase:QueuePhase,pub resolved_model:String,pub budget:Budget,
    pub provenance:EvidenceStatus,pub reason:Option<Failure>,pub grant:Option<Grant>,
    pub ledger_status:LedgerStatus,pub charge:Option<Charge>,
    pub attempts:Vec<Attempt>,pub evidence:Vec<Evidence>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct UnlinkedGrant {
    pub decision_id:String,pub hash:String,pub phase:&'static str,pub updated_at:u64,
    pub lease_id:Option<String>,
    pub revoked_without_io:bool,
    #[serde(skip_serializing_if="Option::is_none")]pub retirement:Option<super::retirement::Reference>,
}
/// Native immutable origin only. This never reports a current allowance,
/// worker death or post-snapshot accounting from a copied signed image.
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RestoredSource{
    pub schema_version:u32,pub history_hash:String,pub source_fingerprint:String,
    pub source_goal_hash:String,pub historical_related_grants:usize,
    pub unattributed_historical_grants:usize,pub owner_absence_proven:bool,
    pub post_snapshot_state_unknown:bool,pub execution_authority_granted:bool,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {
    // Old grant formats authenticate each admission, not an append-only head
    // covering every earlier admission. Even valid listed accounting cannot
    // certify a history after both an old row and its grant were deleted.
    pub history_continuity_verified:bool,pub history_coverage:&'static str,
    pub schema_version:u32,pub scope:&'static str,
    pub goal_id:String,pub pet_id:String,pub provider:String,pub goal_revision:String,
    pub goal_hash:String,pub goal_state:State,pub observed_at:u64,pub source_hash:String,
    pub admissions:Vec<Admission>,pub unlinked_grants:Vec<UnlinkedGrant>,pub pending_artifacts:usize,
    pub lineage:Vec<Evidence>,pub authenticated_subtotal:Charge,pub cumulative_charge:Option<Charge>,
    pub token_usage:Option<u64>,pub cost_micros:Option<u64>,pub metering:&'static str,
    pub scan_complete:bool,pub source_consistent:bool,pub accounting_complete:bool,pub outcomes_complete:bool,
    // Native linked-source validation before unlinked inventory is applied.
    // This subordinate fact is not serialized or an execution permit.
    #[serde(skip)]pub(super)linked_source_consistent:bool,
    pub execution_available:bool,pub continuation_available:bool,
    #[serde(skip_serializing_if="Option::is_none")]pub restored_source:Option<RestoredSource>,
}
impl Snapshot {
    #[cfg(target_os="linux")]
    pub(super)fn empty(goal:&Record,now:u64)->model::Result<Self> {
        let goal_hash=model::fingerprint(goal)?;
        Ok(Self{history_continuity_verified:false,history_coverage:"listedAdmissionsAndPresentSignedGrants",schema_version:1,scope:"allRecordedAdmissions",goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),goal_revision:goal.revision.clone(),goal_hash:goal_hash.clone(),goal_state:goal.state,observed_at:now,source_hash:goal_hash,admissions:vec![],unlinked_grants:vec![],pending_artifacts:0,lineage:vec![],authenticated_subtotal:Charge::default(),cumulative_charge:if goal.legacy.is_empty(){Some(Charge::default())}else{None},token_usage:None,cost_micros:None,metering:"tokensAndCostUnavailable",scan_complete:true,source_consistent:goal.legacy.is_empty(),linked_source_consistent:goal.legacy.is_empty(),accounting_complete:goal.legacy.is_empty(),outcomes_complete:goal.legacy.is_empty(),execution_available:false,continuation_available:false,restored_source:None})
    }
}
