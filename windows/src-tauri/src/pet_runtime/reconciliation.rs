//! Read-only recovery facts. These DTOs deliberately cannot be deserialized
//! into a review, native absence gate, approval or execution controller.
use super::{authority::Failure,leases::{Lease,StopReason},model::{self,Record,State}};
use serde::Serialize;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum EvidenceKind{GoalLineage,ModelResult,PlanningOutput,ToolStep,ToolObservation,Recipe,Verification,Completion}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum EvidenceStatus{Verified,Missing,Invalid,Unavailable}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Evidence{pub kind:EvidenceKind,pub reference:String,pub status:EvidenceStatus,pub hash:Option<String>,pub reason:Option<Failure>,pub effect_verified:Option<bool>,pub outcome_known:bool,pub criteria_verified:bool}
impl Evidence{
    #[cfg(target_os="linux")]pub(super)fn verified<T:Serialize>(kind:EvidenceKind,reference:String,body:&T)->super::authority::Result<Self>{
        Ok(Self{kind,reference,status:EvidenceStatus::Verified,hash:Some(model::fingerprint(body).map_err(|_|Failure::Invalid)?),reason:None,effect_verified:None,outcome_known:false,criteria_verified:false})
    }
    #[cfg(target_os="linux")]pub(super)fn missing(kind:EvidenceKind,reference:String)->Self{Self{kind,reference,status:EvidenceStatus::Missing,hash:None,reason:None,effect_verified:None,outcome_known:false,criteria_verified:false}}
    #[cfg(target_os="linux")]pub(super)fn failed(kind:EvidenceKind,reference:String,reason:Failure)->Self{
        let status=match reason{Failure::Missing|Failure::RecoveryRequired=>EvidenceStatus::Missing,Failure::Busy|Failure::Unavailable|Failure::Uncertain=>EvidenceStatus::Unavailable,_=>EvidenceStatus::Invalid};
        Self{reason:Some(reason),status,..Self::missing(kind,reference)}
    }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Attempt{pub id:String,pub request_id:String,pub kind:&'static str,pub phase:&'static str,pub charged_output_bytes:u64,pub accepted_output_bytes:Option<u64>,pub outcome_known:bool,pub evidence:Vec<Evidence>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Accounting{pub scope:&'static str,pub model_calls_used:usize,pub tool_calls_used:usize,pub output_bytes_charged:u64,pub model_calls_remaining:usize,pub tool_calls_remaining:usize,pub output_bytes_remaining:u64,pub unknown_attempts:usize}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Grant{pub decision_id:String,pub revision:String,pub hash:String,pub phase:&'static str,pub lease:Option<Lease>,pub goal_lease_matches:bool,pub control_projection_current:bool,pub stop_reason:Option<StopReason>,pub updated_at:u64,pub closed_at:Option<u64>,pub owner:super::runtime_owner::Observation}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Snapshot{
    pub schema_version:u32,pub scope:&'static str,pub goal_id:String,pub pet_id:String,pub provider:String,pub goal_revision:String,pub goal_hash:String,pub goal_state:State,pub observed_at:u64,
    pub previous_admissions:usize,pub grant:Option<Grant>,pub budget:Option<super::admission::Budget>,pub accounting:Option<Accounting>,pub attempts:Vec<Attempt>,pub evidence:Vec<Evidence>,
    pub scan_complete:bool,pub source_consistent:bool,pub criteria_verified:bool,pub goal_complete:bool,pub execution_available:bool,pub recovery_available:bool,
}
impl Snapshot{
    #[cfg(target_os="linux")]pub(super)fn empty(goal:&Record,now:u64)->model::Result<Self>{
        Ok(Self{schema_version:1,scope:"latestAdmission",goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),goal_revision:goal.revision.clone(),goal_hash:model::fingerprint(goal)?,goal_state:goal.state,observed_at:now,previous_admissions:goal.admissions.len().saturating_sub(1),grant:None,budget:None,accounting:None,attempts:vec![],evidence:vec![],scan_complete:true,source_consistent:true,criteria_verified:false,goal_complete:false,execution_available:false,recovery_available:false})
    }
}

/// Shared internal receipt inspection for an original admission. This is not a
/// synthetic goal or an approval and is never deserialized from client data.
#[cfg(target_os="linux")]
#[derive(Default)]
pub(super)struct AdmissionEvidence{pub attempts:Vec<Attempt>,pub evidence:Vec<Evidence>,pub criteria_verified:bool}
