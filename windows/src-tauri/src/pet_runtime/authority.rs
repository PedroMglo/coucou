//! Native consent provenance. A proof is data; verification and fresh policy
//! checks are still required before any execution permit can be constructed.
use super::{admission::AdmissionSpec,model};
use serde::{Deserialize,Serialize};
#[cfg(target_os="linux")]pub(super)use super::authority_store::{Authority,Certified};
#[cfg(target_os="linux")]pub(super)use super::authority_store::CompletionCertified;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Failure{Missing,Unsafe,Changed,Invalid,WrongKey,Revoked,Expired,NotQueued,Capacity,Unavailable,Uncertain,Busy,RecoveryRequired}
pub(super)type Result<T>=std::result::Result<T,Failure>;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Operation{ConversationRespond}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConsentProof{pub schema_version:u32,pub key_id:String,pub operation:Operation,pub tag:String}
impl ConsentProof{
    pub(super)fn valid(&self)->bool{self.schema_version==1&&model::token(&self.key_id)&&decode(&self.tag).is_some()}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super)enum GrantPhase{Issued,Revoked,Claimed,Draining,Closed}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Grant{
    #[serde(default,skip_serializing_if="Option::is_none")]pub budget_usage:Option<super::effect_budget::Ledger>,
    pub schema_version:u32,pub decision_id:String,pub goal_id:String,pub operation:Operation,
    pub spec_hash:String,pub phase:GrantPhase,pub revision:String,pub approved_at:u64,pub updated_at:u64,
    #[serde(default,skip_serializing_if="Option::is_none")]pub lease:Option<super::leases::Lease>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub stop_reason:Option<super::leases::StopReason>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub closed_at:Option<u64>,
}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct GrantEnvelope{pub body:Grant,pub tag:String}
impl Grant{
    pub(super)fn valid(&self)->bool{
        if !model::token(&self.decision_id)||!model::goal_id(&self.goal_id)||decode(&self.spec_hash).is_none()||!model::token(&self.revision)||self.approved_at==0||self.updated_at<self.approved_at||self.updated_at>8_640_000_000_000_000{return false;}
        if self.budget_usage.as_ref().is_some_and(|u|!u.valid()){return false;}
        match &self.lease{
            None=>self.budget_usage.is_none()&&self.schema_version==1&&matches!(self.phase,GrantPhase::Issued|GrantPhase::Revoked)&&self.stop_reason.is_none()&&self.closed_at.is_none(),
            Some(l)=>self.schema_version==(if self.budget_usage.is_some(){3}else{2})&&l.basic()&&l.goal_id==self.goal_id&&l.decision_id==self.decision_id
                &&match self.phase{
                    GrantPhase::Claimed=>self.stop_reason.is_none()&&self.closed_at.is_none(),
                    GrantPhase::Draining=>self.stop_reason.is_some()&&self.closed_at.is_none(),
                    GrantPhase::Closed|GrantPhase::Revoked=>self.closed_at.is_some_and(|n|n>=l.claimed_at&&n<=self.updated_at),
                    GrantPhase::Issued=>false,
                },
        }
    }
}
pub(super)fn encode(bytes:&[u8])->String{bytes.iter().map(|b|format!("{b:02x}")).collect()}
pub(super)fn decode(text:&str)->Option<[u8;32]>{
    if text.len()!=64||!text.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f')){return None;}
    let mut bytes=[0u8;32];for(i,b)in bytes.iter_mut().enumerate(){*b=u8::from_str_radix(&text[i*2..i*2+2],16).ok()?;}Some(bytes)
}
pub(super)fn consent_message(proof:&ConsentProof,spec:&AdmissionSpec)->Result<Vec<u8>>{
    let mut bytes=b"coucou-native-goal-consent-v1\0".to_vec();bytes.extend(serde_json::to_vec(&(proof.schema_version,&proof.key_id,proof.operation,spec)).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
pub(super)fn grant_message(body:&Grant)->Result<Vec<u8>>{
    let mut bytes=b"coucou-native-goal-grant-state-v1\0".to_vec();bytes.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ProvenanceStatus{pub verified:bool,pub reason:Option<Failure>,pub execution_available:bool}
impl ProvenanceStatus{pub(super)fn from_result(r:Result<()>)->Self{Self{verified:r.is_ok(),reason:r.err(),execution_available:false}}}
