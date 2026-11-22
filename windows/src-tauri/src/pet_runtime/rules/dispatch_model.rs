//! Native-reviewed, fixed metadata allowance; never another model-call balance.
use super::*;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DispatchPolicy{pub schema_version:u32,pub max_metadata_probes:u8,pub metadata_timeout_millis:u32,pub retry_delay_millis:u32}
impl DispatchPolicy{
 pub(crate)fn bounded()->Self{Self{schema_version:1,max_metadata_probes:3,metadata_timeout_millis:6000,retry_delay_millis:30000}}
 pub(crate)fn valid(&self)->bool{self==&Self::bounded()}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReadinessProbe{#[serde(default,skip_serializing_if="Option::is_none")]pub(in crate::pet_runtime) owner:Option<super::super::runtime_owner::Operation>,#[serde(default,skip_serializing_if="Option::is_none")]pub(in crate::pet_runtime) recovery:Option<ProbeRecovery>,pub id:String,pub scope_hash:String,pub prepared_at:u64,pub expires_at:u64,pub finished_at:Option<u64>,pub observed:Option<bool>}
impl ReadinessProbe{
 pub(crate)fn scope(p:&RulePolicy,r:&RuleReservation)->model::Result<String>{model::fingerprint(&("coucou-rule-metadata-probe-v1",p.hash()?,&r.occurrence_id,&p.destination,&p.model,&p.engine_binding,r.deadline_at))}
 pub(crate)fn valid(&self,g:&RuleGrant,r:&RuleReservation)->bool{
  let Some(policy)=&g.policy.dispatch else{return false;};
  self.owner.as_ref().is_none_or(|o|o.valid(&self.id,&self.scope_hash,self.prepared_at))&&self.recovery.as_ref().is_none_or(|r|r.valid(self)&&r.reconciled_at<=g.updated_at)&&model::token(&self.id)&&Self::scope(&g.policy,r).ok().as_ref()==Some(&self.scope_hash)&&self.prepared_at>=r.reserved_at&&self.prepared_at<self.expires_at&&self.expires_at==self.prepared_at.saturating_add(u64::from(policy.metadata_timeout_millis)).min(r.deadline_at)&&match(self.finished_at,self.observed){(None,None)=>true,(Some(at),Some(_))=>at>=self.prepared_at&&at<=g.updated_at,_=>false}
 }
}

#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct AutomaticObservation{pub observed_at:u64,pub phase:String,pub reason:Option<String>}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct ProbeRecovery{pub schema_version:u32,pub source_hash:String,pub native_decision_id:String,pub reconciled_at:u64,pub outcome:RecoveryOutcome}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum RecoveryOutcome{NativeWorkerEndedWithoutObservation,OriginalCompletionAcknowledged}
impl ProbeRecovery{fn valid(&self,p:&ReadinessProbe)->bool{
 self.schema_version==1&&self.source_hash.len()==64&&super::super::authority::decode(&self.source_hash).is_some()&&model::token(&self.native_decision_id)&&p.owner.is_some()&&p.finished_at.is_some_and(|at|at<=self.reconciled_at)&&self.reconciled_at<=8_640_000_000_000_000&&match self.outcome{RecoveryOutcome::NativeWorkerEndedWithoutObservation=>p.observed==Some(false)&&p.finished_at==Some(self.reconciled_at),RecoveryOutcome::OriginalCompletionAcknowledged=>p.observed.is_some()}
}}
