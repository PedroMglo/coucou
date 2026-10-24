//! Durable queue data is never an execution permit. Review/Approved are owned
//! backend values, intentionally neither Clone nor Deserialize. A native review
//! controller must retain Review and consume it only after exact user consent.
use super::model::{self,ContextBinding,Owner,Record,State};
use crate::pet_capabilities::{Availability,Caller,Descriptor,Lifecycle,Support};
use serde::{Deserialize,Serialize};

const REVIEW_TTL:u64=120_000;
const MAX_DATE:u64=8_640_000_000_000_000;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Denial{Invalid,OwnerChanged,ContextRequired,ContextChanged,NotDraft,Inactive,Unsupported,NotReady,Stale,ModelUnresolved,PolicyChanged,Budget,PaymentPolicy,MeteringUnavailable,Expired,RevisionChanged,InputLimit}
pub type Result<T>=std::result::Result<T,Denial>;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Payment{LocalOnly,ExistingAccount}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Budget{
    pub deadline_at:u64,pub max_model_calls:u16,pub max_tool_calls:u16,pub max_output_bytes:u64,
    pub max_tokens:Option<u64>,pub max_cost_micros:Option<u64>,pub payment:Payment,
}
impl Budget{
    pub fn valid(&self,at:u64)->bool{
        self.deadline_at>at&&self.deadline_at<=MAX_DATE&&self.deadline_at-at<=86_400_000
        &&(1..=64).contains(&self.max_model_calls)&&self.max_tool_calls<=128
        &&(1..=16*1024*1024).contains(&self.max_output_bytes)
        &&self.max_tokens.is_none_or(|n|(1..=10_000_000).contains(&n))
        &&self.max_cost_micros.is_none_or(|n|(1..=1_000_000_000).contains(&n))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum QueuePhase{Queued,Paused,Cancelled,Leased,StopRequested,RecoveryRequired,Completed}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AdmissionSpec{
    #[serde(default,skip_serializing_if="Option::is_none")]pub proposal_hash:Option<String>,
    pub schema_version:u32,pub request_id:String,pub decision_id:String,pub goal_id:String,
    pub source_revision:String,pub source_sequence:u64,pub owner:Owner,pub context:ContextBinding,
    pub context_hash:String,pub intent_hash:String,pub resolved_model:String,pub descriptor_revision:String,
    pub policy_revision:String,pub evidence_hash:String,pub approved_at:u64,pub budget:Budget,
}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct QueuedAdmission{pub spec:AdmissionSpec,pub phase:QueuePhase,#[serde(default,skip_serializing_if="Option::is_none")]pub proof:Option<super::authority::ConsentProof>,#[serde(default,skip_serializing_if="Option::is_none")]pub lease:Option<super::leases::Lease>,#[serde(default,skip_serializing_if="Option::is_none")]pub stop_reason:Option<super::leases::StopReason>}
fn hash(s:&str)->bool{s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))}
fn model_id(s:&str)->bool{!s.trim().is_empty()&&s.trim()==s&&s.len()<=256&&!s.starts_with(['@','/','~','\\'])&&!s.contains("://")&&!s.chars().any(|c|c.is_control())}
impl AdmissionSpec{
    fn binding_format(&self)->bool{
        matches!((self.schema_version,self.proposal_hash.as_deref()),(1,None))||self.schema_version==2&&self.proposal_hash.as_deref().is_some_and(hash)
    }
    pub fn valid(&self,record:&Record)->bool{
        self.binding_format()&&model::key(&self.request_id)&&model::token(&self.decision_id)
        &&self.goal_id==record.id&&model::token(&self.source_revision)&&self.source_sequence>0&&self.source_sequence<record.sequence
        &&self.owner==record.spec.owner&&self.context.valid(record)&&record.execution_context.as_ref()==Some(&self.context)
        &&self.context_hash==model::fingerprint(&self.context).unwrap_or_default()&&hash(&self.context_hash)
        &&self.intent_hash==model::fingerprint(&record.spec).unwrap_or_default()&&hash(&self.intent_hash)
        &&model_id(&self.resolved_model)&&!self.descriptor_revision.is_empty()&&self.descriptor_revision.len()<=256
        &&hash(&self.policy_revision)&&hash(&self.evidence_hash)&&self.approved_at>=record.created_at&&self.approved_at<=record.updated_at
        &&self.budget.valid(self.approved_at)&&(self.budget.payment!=Payment::LocalOnly||self.owner.adapter=="ollama")
    }
}

/// A fresh, typed observation supplied by the native adapter, never by a model
/// or by a deserialized admission. Actual writer checks remain mandatory.
pub(super)struct Evidence{owner:Owner,context:ContextBinding,model:String,descriptor_revision:String,descriptor_semantics:String,policy_revision:String,observed_at:u64,expires_at:u64,tokens:bool,cost:bool}
impl Evidence{
    pub(super)fn for_claim(&self,goal:&Record,now:u64)->Result<(String,String)>{
        self.current(goal,now)?;if now<goal.updated_at{return Err(Denial::Expired);}
        let row=goal.admissions.last().ok_or(Denial::NotReady)?;
        if !matches!(goal.state,State::Queued|State::Leased){return Err(Denial::NotReady);}
        if self.owner!=row.spec.owner||self.model!=row.spec.resolved_model{return Err(Denial::OwnerChanged);}
        if self.policy_revision!=row.spec.policy_revision{return Err(Denial::PolicyChanged);}
        if row.spec.budget.max_tokens.is_some()&&!self.tokens||row.spec.budget.max_cost_micros.is_some()&&!self.cost{return Err(Denial::MeteringUnavailable);}
        if now>=row.spec.budget.deadline_at||now<row.spec.approved_at{return Err(Denial::Expired);}
        Ok((self.descriptor_revision.clone(),self.policy_revision.clone()))
    }
    pub(super)fn observe(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool)->Result<Self>{
        goal.validate().map_err(|_|Denial::Invalid)?;
        if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}
        if d.pet_id!=goal.spec.owner.pet_id||d.binding.provider.as_deref()!=Some(&goal.spec.owner.provider)||d.binding.adapter!=Some(goal.spec.owner.adapter.as_str()){return Err(Denial::OwnerChanged);}
        let context=goal.execution_context.clone().ok_or(Denial::ContextRequired)?;
        let model=d.binding.model.clone().filter(|m|model_id(m)).ok_or(Denial::ModelUnresolved)?;
        if !hash(&policy_revision)||d.revision.is_empty()||d.revision.len()>256{return Err(Denial::Invalid);}
        let mut caps=d.capabilities.iter().filter(|c|c.id=="conversation.respond");let c=caps.next().ok_or(Denial::Unsupported)?;
        if caps.next().is_some()||c.support!=Support::Supported||!c.callers.contains(&Caller::TaskRuntime){return Err(Denial::Unsupported);}
        if c.availability!=Availability::Ready||c.reason.is_some(){return Err(Denial::NotReady);}
        if c.model_binding.as_deref().is_some_and(|m|m!=model){return Err(Denial::ModelUnresolved);}
        let at=c.observed_at.ok_or(Denial::Stale)?;let end=c.expires_at.ok_or(Denial::Stale)?;
        if at==0||at>now||now>=end||end>MAX_DATE||now-at>15_000{return Err(Denial::Stale);}
        Ok(Self{owner:goal.spec.owner.clone(),context,model,descriptor_revision:d.revision.clone(),descriptor_semantics:crate::pet_capabilities::admission_semantics(d).map_err(|_|Denial::Invalid)?,policy_revision,observed_at:at,expires_at:end,tokens,cost})
    }
    fn fingerprint(&self)->model::Result<String>{model::fingerprint(&(&self.owner,&self.context,&self.model,&self.descriptor_revision,&self.descriptor_semantics,&self.policy_revision,self.observed_at,self.expires_at,self.tokens,self.cost))}
    fn current(&self,goal:&Record,now:u64)->Result<()>{
        if goal.spec.owner!=self.owner{return Err(Denial::OwnerChanged);}
        if goal.execution_context.as_ref()!=Some(&self.context){return Err(Denial::ContextChanged);}
        if now<self.observed_at||now>=self.expires_at||now-self.observed_at>15_000{return Err(Denial::Stale);}Ok(())
    }
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewProposal{pub proposals:super::plans::Snapshot,pub operation:&'static str,pub tool_execution_available:bool,pub goal_id:String,pub revision:String,pub request_id:String,pub owner:Owner,pub context:ContextBinding,pub intent:String,pub criteria:Vec<String>,pub resolved_model:String,pub policy_revision:String,pub descriptor_revision:String,pub budget:Budget,pub prepared_at:u64,pub expires_at:u64}
pub(super)struct Review{spec:AdmissionSpec,expires_at:u64,evidence:Evidence,intent:String,criteria:Vec<String>,proposals:super::plans::Snapshot}
pub(super)struct Approved{spec:AdmissionSpec,expires_at:u64}
impl Review{
    pub(super)fn prepare(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64)->Result<Self>{
        goal.validate().map_err(|_|Denial::Invalid)?;if now<goal.updated_at{return Err(Denial::Expired);}if goal.state!=State::Draft{return Err(Denial::NotDraft);}
        evidence.current(goal,now)?;
        if !budget.valid(now){return Err(Denial::Budget);}
        if budget.payment==Payment::LocalOnly&&goal.spec.owner.adapter!="ollama"{return Err(Denial::PaymentPolicy);}
        if budget.max_tokens.is_some()&&!evidence.tokens||budget.max_cost_micros.is_some()&&!evidence.cost{return Err(Denial::MeteringUnavailable);}
        if !model::key(&request_id)||!model::token(&decision_id)||goal.events.iter().any(|e|e.request_id==request_id){return Err(Denial::Invalid);}
        super::plans::reviewed_input(goal).map_err(|_|Denial::InputLimit)?;
        let expires_at=now.saturating_add(REVIEW_TTL).min(budget.deadline_at);
        let spec=AdmissionSpec{schema_version:2,proposal_hash:Some(super::plans::fingerprint(goal).map_err(|_|Denial::Invalid)?),request_id,decision_id,goal_id:goal.id.clone(),source_revision:goal.revision.clone(),source_sequence:goal.sequence,owner:goal.spec.owner.clone(),context:evidence.context.clone(),context_hash:model::fingerprint(&evidence.context).map_err(|_|Denial::Invalid)?,intent_hash:model::fingerprint(&goal.spec).map_err(|_|Denial::Invalid)?,resolved_model:evidence.model.clone(),descriptor_revision:evidence.descriptor_revision.clone(),policy_revision:evidence.policy_revision.clone(),evidence_hash:evidence.fingerprint().map_err(|_|Denial::Invalid)?,approved_at:now,budget};
        Ok(Self{spec,expires_at,evidence,intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),proposals:super::plans::snapshot(goal)})
    }
    /// Native UI should render this exact value and retain the original Review.
    pub(super)fn proposal(&self)->ReviewProposal{ReviewProposal{proposals:self.proposals.clone(),operation:"conversationRespond",tool_execution_available:false,goal_id:self.spec.goal_id.clone(),revision:self.spec.source_revision.clone(),request_id:self.spec.request_id.clone(),owner:self.spec.owner.clone(),context:self.spec.context.clone(),intent:self.intent.clone(),criteria:self.criteria.clone(),resolved_model:self.spec.resolved_model.clone(),policy_revision:self.spec.policy_revision.clone(),descriptor_revision:self.spec.descriptor_revision.clone(),budget:self.spec.budget.clone(),prepared_at:self.spec.approved_at,expires_at:self.expires_at}}
    /// Consumed only after native exact-review consent; latest observations must
    /// be rebuilt by the adapter. This cannot be called with a JSON approval.
    pub(super)fn approve(mut self,goal:&Record,latest:Evidence,now:u64)->Result<Approved>{
        if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}
        if goal.state!=State::Draft||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||!super::plans::matches(goal,&self.spec){return Err(Denial::RevisionChanged);}
        latest.current(goal,now)?;
        if latest.policy_revision!=self.evidence.policy_revision{return Err(Denial::PolicyChanged);}
        // Observation epochs/timestamps can refresh. The original identity, model,
        // semantic capabilities and policy must remain exactly the same.
        if latest.owner!=self.evidence.owner||latest.context!=self.evidence.context||latest.model!=self.evidence.model||latest.descriptor_semantics!=self.evidence.descriptor_semantics||latest.tokens!=self.evidence.tokens||latest.cost!=self.evidence.cost{return Err(Denial::RevisionChanged);}
        self.spec.approved_at=now;self.spec.evidence_hash=latest.fingerprint().map_err(|_|Denial::Invalid)?;
        Ok(Approved{spec:self.spec,expires_at:self.expires_at.min(latest.expires_at)})
    }
}
impl Approved{
    pub(super)fn spec(&self)->&AdmissionSpec{&self.spec}
    pub(super)fn current(&self,goal:&Record,now:u64)->model::Result<()>{
        goal.validate()?;
        if now<self.spec.approved_at||now>=self.expires_at||now>=self.spec.budget.deadline_at||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||goal.spec.owner!=self.spec.owner||goal.execution_context.as_ref()!=Some(&self.spec.context)||model::fingerprint(&goal.spec)?!=self.spec.intent_hash||!super::plans::matches(goal,&self.spec){return Err(model::Error::Conflict);}Ok(())
    }
}
