//! Durable queue data is never an execution permit. Review/Approved are owned
//! backend values, intentionally neither Clone nor Deserialize. A native review
//! controller must retain Review and consume it only after exact user consent.
use super::model::{self,ContextBinding,Owner,Record,State};
use crate::pet_capabilities::{Availability,Caller,Descriptor,Lifecycle,Support};
use serde::{Deserialize,Serialize};

const REVIEW_TTL:u64=120_000;
const MAX_DATE:u64=8_640_000_000_000_000;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Denial{Invalid,OwnerChanged,ContextRequired,ContextChanged,NotDraft,Inactive,Unsupported,NotReady,Stale,ModelUnresolved,PolicyChanged,Budget,PaymentPolicy,MeteringUnavailable,Expired,RevisionChanged,InputLimit,RecoveryNotClosed,HistoryIncomplete,UnreconciledEffects,DataConsentRequired}
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
    #[serde(default,skip_serializing_if="Option::is_none")]pub memory_data:Option<crate::pet_memory::context_model::GoalGrant>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub engine_binding:Option<crate::local_chat::lmstudio::Binding>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub continuation:Option<super::continuation::Reference>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub proposal_hash:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub planning_policy:Option<super::planning::Policy>,
    pub schema_version:u32,pub request_id:String,pub decision_id:String,pub goal_id:String,
    pub source_revision:String,pub source_sequence:u64,pub owner:Owner,pub context:ContextBinding,
    pub context_hash:String,pub intent_hash:String,pub resolved_model:String,pub descriptor_revision:String,
    pub policy_revision:String,pub evidence_hash:String,pub approved_at:u64,pub budget:Budget,
}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct QueuedAdmission{pub spec:AdmissionSpec,pub phase:QueuePhase,#[serde(default,skip_serializing_if="Option::is_none")]pub proof:Option<super::authority::ConsentProof>,#[serde(default,skip_serializing_if="Option::is_none")]pub lease:Option<super::leases::Lease>,#[serde(default,skip_serializing_if="Option::is_none")]pub stop_reason:Option<super::leases::StopReason>}
fn hash(s:&str)->bool{s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))}
pub(super)fn model_id(s:&str)->bool{!s.trim().is_empty()&&s.trim()==s&&s.len()<=256&&!s.starts_with(['@','/','~','\\'])&&!s.contains("://")&&!s.chars().any(|c|c.is_control())}
impl AdmissionSpec{
    pub(super)fn binding_format(&self)->bool{
        // References alone remain insufficient. This typed scope/destination is
        // certified only from the original native Approved, never from JSON.
        match (&self.context.memory,&self.memory_data){
            (None,None)=>{},
            (Some(_),Some(data))if self.context.schema_version==2&&self.owner.local_engine()&&matches!(self.schema_version,2|3)&&data.valid(&self.owner.provider,&self.resolved_model)=>{},
            _=>return false,
        }
        if self.owner.adapter=="lmstudio"{if self.owner.provider!="lmstudio"||self.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&self.resolved_model)){return false;}}
        else if self.engine_binding.is_some(){return false;}
        if self.continuation.as_ref().is_some_and(|c|!c.basic()||!matches!(self.schema_version,2|3)){return false;}
        if self.continuation.as_ref().is_some_and(|c|c.uncertain_effects.is_some())&&(self.schema_version!=3||!self.owner.local_engine()||self.budget.payment!=Payment::LocalOnly||self.planning_policy.as_ref().is_none_or(|p|p.catalog!=super::planning::Catalog::SpotifyTransportV1)){return false;}
        match (self.schema_version,self.proposal_hash.as_deref(),self.planning_policy.as_ref()){
            (1,None,None)=>true,
            (2,Some(h),None)=>hash(h),
            (3,Some(h),Some(p))=>hash(h)&&p.valid_for(&self.budget)&&self.owner.local_engine()&&self.budget.payment==Payment::LocalOnly,
            _=>false,
        }
    }
    pub fn valid(&self,record:&Record)->bool{
        self.binding_format()&&self.continuation.as_ref().is_none_or(|c|c.present(record,self))&&model::key(&self.request_id)&&model::token(&self.decision_id)
        &&self.goal_id==record.id&&model::token(&self.source_revision)&&self.source_sequence>0&&self.source_sequence<record.sequence
        &&self.owner==record.spec.owner&&self.context.valid(record)&&record.execution_context.as_ref()==Some(&self.context)
        &&self.context_hash==model::fingerprint(&self.context).unwrap_or_default()&&hash(&self.context_hash)
        &&self.intent_hash==model::fingerprint(&record.spec).unwrap_or_default()&&hash(&self.intent_hash)
        &&model_id(&self.resolved_model)&&!self.descriptor_revision.is_empty()&&self.descriptor_revision.len()<=256
        &&hash(&self.policy_revision)&&hash(&self.evidence_hash)&&self.approved_at>=record.created_at&&self.approved_at<=record.updated_at
        &&self.budget.valid(self.approved_at)&&(self.budget.payment!=Payment::LocalOnly||self.owner.local_engine())
    }
}

/// A fresh, typed observation supplied by the native adapter, never by a model
/// or by a deserialized admission. Actual writer checks remain mandatory.
pub(super)struct Evidence{#[cfg(target_os="linux")]memory:Option<super::memory_review::Captured>,engine_binding:Option<crate::local_chat::lmstudio::Binding>,owner:Owner,context:ContextBinding,model:String,descriptor_revision:String,descriptor_semantics:String,policy_revision:String,observed_at:u64,expires_at:u64,tokens:bool,cost:bool}
impl Evidence{
    pub(super)fn for_claim(&self,goal:&Record,now:u64)->Result<(String,String)>{
        self.current(goal,now)?;if now<goal.updated_at{return Err(Denial::Expired);}
        let row=goal.admissions.last().ok_or(Denial::NotReady)?;
        if !matches!(goal.state,State::Queued|State::Leased){return Err(Denial::NotReady);}
        if self.memory_grant()!=row.spec.memory_data||self.owner!=row.spec.owner||self.model!=row.spec.resolved_model||self.engine_binding!=row.spec.engine_binding{return Err(Denial::OwnerChanged);}
        if self.policy_revision!=row.spec.policy_revision{return Err(Denial::PolicyChanged);}
        if row.spec.budget.max_tokens.is_some()&&!self.tokens||row.spec.budget.max_cost_micros.is_some()&&!self.cost{return Err(Denial::MeteringUnavailable);}
        if now>=row.spec.budget.deadline_at||now<row.spec.approved_at{return Err(Denial::Expired);}
        Ok((self.descriptor_revision.clone(),self.policy_revision.clone()))
    }
    pub(super)fn observe(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool)->Result<Self>{
        Self::observe_base(goal,d,policy_revision,now,tokens,cost,false)
    }
    fn observe_base(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,has_native_memory:bool)->Result<Self>{
        goal.validate().map_err(|_|Denial::Invalid)?;
        if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}
        if d.pet_id!=goal.spec.owner.pet_id||d.binding.provider.as_deref()!=Some(&goal.spec.owner.provider)||d.binding.adapter!=Some(goal.spec.owner.adapter.as_str()){return Err(Denial::OwnerChanged);}
        let context=goal.execution_context.clone().ok_or(Denial::ContextRequired)?;
        if context.memory.is_some()&&!has_native_memory{return Err(Denial::Unsupported);}// References alone cannot manufacture native data review.
        let model=d.binding.model.clone().filter(|m|model_id(m)).ok_or(Denial::ModelUnresolved)?;
        if !hash(&policy_revision)||d.revision.is_empty()||d.revision.len()>256{return Err(Denial::Invalid);}
        let mut caps=d.capabilities.iter().filter(|c|c.id=="conversation.respond");let c=caps.next().ok_or(Denial::Unsupported)?;
        if caps.next().is_some()||c.support!=Support::Supported||!c.callers.contains(&Caller::TaskRuntime){return Err(Denial::Unsupported);}
        if c.availability!=Availability::Ready||c.reason.is_some(){return Err(Denial::NotReady);}
        if c.model_binding.as_deref().is_some_and(|m|m!=model){return Err(Denial::ModelUnresolved);}
        if goal.spec.owner.adapter=="lmstudio"&&d.binding.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&model)){return Err(Denial::ModelUnresolved);}
        let at=c.observed_at.ok_or(Denial::Stale)?;let end=c.expires_at.ok_or(Denial::Stale)?;
        if at==0||at>now||now>=end||end>MAX_DATE||now-at>15_000{return Err(Denial::Stale);}
        Ok(Self{#[cfg(target_os="linux")]memory:None,engine_binding:d.binding.engine_binding.clone(),owner:goal.spec.owner.clone(),context,model,descriptor_revision:d.revision.clone(),descriptor_semantics:crate::pet_capabilities::admission_semantics(d).map_err(|_|Denial::Invalid)?,policy_revision,observed_at:at,expires_at:end,tokens,cost})
    }
    fn fingerprint(&self)->model::Result<String>{
        let legacy=(||->model::Result<String>{if let Some(binding)=&self.engine_binding{return model::fingerprint(&(binding,&self.owner,&self.context,&self.model,&self.descriptor_revision,&self.descriptor_semantics,&self.policy_revision,self.observed_at,self.expires_at,self.tokens,self.cost));}model::fingerprint(&(&self.owner,&self.context,&self.model,&self.descriptor_revision,&self.descriptor_semantics,&self.policy_revision,self.observed_at,self.expires_at,self.tokens,self.cost))})();
        let legacy=legacy?;match self.memory_preview(){Some(data)=>model::fingerprint(&(legacy,data)),None=>Ok(legacy)}
    }
    pub(super)fn memory_preview(&self)->Option<crate::pet_memory::context_model::GoalPreview>{
        #[cfg(target_os="linux")]{self.memory.as_ref().map(|m|m.preview().clone())}
        #[cfg(not(target_os="linux"))]{None}
    }
    fn memory_grant(&self)->Option<crate::pet_memory::context_model::GoalGrant>{
        #[cfg(target_os="linux")]{self.memory.as_ref().map(|m|m.grant())}
        #[cfg(not(target_os="linux"))]{None}
    }
    fn reviewed_memory_input(&self,goal:&Record,now:u64)->Result<()>{
        #[cfg(target_os="linux")]if let Some(memory)=&self.memory{
            let message=memory.message_data(goal,now).map_err(|_|Denial::ContextChanged)?;
            let input=super::plans::reviewed_input(goal).map_err(|_|Denial::InputLimit)?;
            if input.len().saturating_add(serde_json::to_vec(&message).map_err(|_|Denial::InputLimit)?.len())>crate::pet_memory::context_model::GOAL_INPUT_LIMIT{return Err(Denial::InputLimit);}
        }
        let _=(goal,now);Ok(())
    }
    fn memory_expiry(&self)->Option<u64>{
        #[cfg(target_os="linux")]{self.memory.as_ref().map(|m|m.expires_at())}
        #[cfg(not(target_os="linux"))]{None}
    }
    fn memory_current(&self,goal:&Record,now:u64)->Result<()>{
        #[cfg(target_os="linux")]if let Some(memory)=&self.memory{return memory.current(goal,now).map_err(|_|Denial::ContextChanged);}
        if self.context.memory.is_some(){return Err(Denial::Unsupported);}Ok(())
    }
    fn same_memory(&self,other:&Self)->bool{
        #[cfg(target_os="linux")]{match(&self.memory,&other.memory){(None,None)=>true,(Some(a),Some(b))=>a.same_data(b),_=>false}}
        #[cfg(not(target_os="linux"))]{let _=other;true}
    }
    #[cfg(target_os="linux")]
    pub(super)fn from_memory_capture(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,memory:Option<super::memory_review::Captured>)->Result<Self>{
        if memory.is_none(){return Self::observe(goal,d,policy_revision,now,tokens,cost);}
        let memory=memory.unwrap();memory.current(goal,now).map_err(|_|Denial::ContextChanged)?;
        let preview=memory.preview();
        if preview.pet_id!=goal.spec.owner.pet_id||preview.goal_id!=goal.id||preview.destination.provider()!=goal.spec.owner.provider||!matches!(&preview.destination,crate::pet_memory::context_model::Destination::Server{model,..}if Some(model)==d.binding.model.as_ref()){return Err(Denial::ContextChanged);}
        let mut evidence=Self::observe_base(goal,d,policy_revision,now,tokens,cost,true)?;evidence.memory=Some(memory);Ok(evidence)
    }
    #[cfg(target_os="linux")]
    pub(super)fn observe_native(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,settings:&crate::settings::Settings)->Result<Self>{
        if goal.execution_context.as_ref().and_then(|b|b.memory.as_ref()).is_none(){return Self::observe(goal,d,policy_revision,now,tokens,cost);}
        let server=crate::local_chat::server(settings,&goal.spec.owner.provider).ok_or(Denial::Unsupported)?;
        let model=d.binding.model.as_deref().filter(|m|model_id(m)).ok_or(Denial::ModelUnresolved)?;
        let memory=super::memory_review::Captured::native(goal,&server,model,now).map_err(|_|Denial::ContextChanged)?;
        Self::from_memory_capture(goal,d,policy_revision,now,tokens,cost,memory)
    }
    fn current(&self,goal:&Record,now:u64)->Result<()>{
        if goal.spec.owner!=self.owner{return Err(Denial::OwnerChanged);}
        if goal.execution_context.as_ref()!=Some(&self.context){return Err(Denial::ContextChanged);}
        if now<self.observed_at||now>=self.expires_at||now-self.observed_at>15_000{return Err(Denial::Stale);}self.memory_current(goal,now)
    }
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewProposal{#[serde(skip_serializing_if="Option::is_none")]pub restored_from:Option<super::restored_future::Origin>,#[serde(skip_serializing_if="Option::is_none")]pub memory_data:Option<crate::pet_memory::context_model::GoalPreview>,#[serde(skip_serializing_if="Option::is_none")]pub engine_binding:Option<crate::local_chat::lmstudio::Binding>,#[serde(skip_serializing_if="Option::is_none")]pub continuation:Option<super::continuation::Reference>,#[serde(skip_serializing_if="Option::is_none")]pub planning_policy:Option<super::planning::Policy>,pub proposals:super::plans::Snapshot,pub operation:&'static str,pub tool_execution_available:bool,pub goal_id:String,pub revision:String,pub request_id:String,pub owner:Owner,pub context:ContextBinding,pub intent:String,pub criteria:Vec<String>,pub resolved_model:String,pub policy_revision:String,pub descriptor_revision:String,pub budget:Budget,pub prepared_at:u64,pub expires_at:u64}
pub(super)struct Review{restored_from:Option<super::restored_future::Origin>,#[cfg(target_os="linux")]retained_authority:Option<super::authority::Authority>,spec:AdmissionSpec,expires_at:u64,evidence:Evidence,intent:String,criteria:Vec<String>,proposals:super::plans::Snapshot}
pub(super)struct Approved{#[cfg(target_os="linux")]memory_confirmation:Option<super::memory_review::ConfirmedData>,#[cfg(target_os="linux")]retained_authority:Option<super::authority::Authority>,spec:AdmissionSpec,expires_at:u64,evidence:Evidence}
impl Review{
    pub(super)fn prepare(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64)->Result<Self>{
        Self::prepare_source(goal,evidence,budget,request_id,decision_id,now,None)
    }
    fn prepare_source(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64,continuation:Option<super::continuation::Reference>)->Result<Self>{
        goal.validate().map_err(|_|Denial::Invalid)?;if now<goal.updated_at{return Err(Denial::Expired);}
        if !source_state(goal,continuation.as_ref()){return Err(Denial::NotDraft);}
        evidence.current(goal,now)?;
        if !budget.valid(now){return Err(Denial::Budget);}
        if budget.payment==Payment::LocalOnly&&!goal.spec.owner.local_engine(){return Err(Denial::PaymentPolicy);}
        if budget.max_tokens.is_some()&&!evidence.tokens||budget.max_cost_micros.is_some()&&!evidence.cost{return Err(Denial::MeteringUnavailable);}
        if !model::key(&request_id)||!model::token(&decision_id)||goal.events.iter().any(|e|e.request_id==request_id){return Err(Denial::Invalid);}
        super::plans::reviewed_input(goal).map_err(|_|Denial::InputLimit)?;evidence.reviewed_memory_input(goal,now)?;
        let expires_at=now.saturating_add(REVIEW_TTL).min(budget.deadline_at).min(evidence.memory_expiry().unwrap_or(u64::MAX));
        let spec=AdmissionSpec{memory_data:evidence.memory_grant(),engine_binding:evidence.engine_binding.clone(),continuation,schema_version:2,planning_policy:None,proposal_hash:Some(super::plans::fingerprint(goal).map_err(|_|Denial::Invalid)?),request_id,decision_id,goal_id:goal.id.clone(),source_revision:goal.revision.clone(),source_sequence:goal.sequence,owner:goal.spec.owner.clone(),context:evidence.context.clone(),context_hash:model::fingerprint(&evidence.context).map_err(|_|Denial::Invalid)?,intent_hash:model::fingerprint(&goal.spec).map_err(|_|Denial::Invalid)?,resolved_model:evidence.model.clone(),descriptor_revision:evidence.descriptor_revision.clone(),policy_revision:evidence.policy_revision.clone(),evidence_hash:evidence.fingerprint().map_err(|_|Denial::Invalid)?,approved_at:now,budget};
        Ok(Self{restored_from:goal.spec.restored_from.clone(),#[cfg(target_os="linux")]retained_authority:None,spec,expires_at,evidence,intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),proposals:super::plans::snapshot(goal)})
    }
    /// Called only with the native configuration/evidence snapshot. A caller
    /// selects numeric bounds; it cannot supply the catalog or a grant.
    pub(super)fn prepare_planning(goal:&Record,evidence:Evidence,budget:Budget,limits:super::planning::Limits,config:&crate::pet_config::Configuration,request_id:String,decision_id:String,now:u64)->Result<Self>{
        if !goal.spec.owner.local_engine(){return Err(Denial::Unsupported);}
        if budget.payment!=Payment::LocalOnly{return Err(Denial::PaymentPolicy);}
        if !config.permissions(&goal.spec.owner.pet_id).map_err(|_|Denial::Invalid)?.music{return Err(Denial::Unsupported);}
        let policy=super::planning::Policy::native(limits,&budget).map_err(|_|Denial::Budget)?;
        let mut review=Self::prepare(goal,evidence,budget,request_id,decision_id,now)?;
        review.spec.schema_version=3;review.spec.planning_policy=Some(policy);Ok(review)
    }
    /// Native UI should render this exact value and retain the original Review.
    pub(super)fn proposal(&self)->ReviewProposal{ReviewProposal{restored_from:self.restored_from.clone(),memory_data:self.evidence.memory_preview(),engine_binding:self.spec.engine_binding.clone(),continuation:self.spec.continuation.clone(),planning_policy:self.spec.planning_policy.clone(),proposals:self.proposals.clone(),operation:if self.spec.planning_policy.is_some(){"goalPlanning"}else{"conversationRespond"},tool_execution_available:false,goal_id:self.spec.goal_id.clone(),revision:self.spec.source_revision.clone(),request_id:self.spec.request_id.clone(),owner:self.spec.owner.clone(),context:self.spec.context.clone(),intent:self.intent.clone(),criteria:self.criteria.clone(),resolved_model:self.spec.resolved_model.clone(),policy_revision:self.spec.policy_revision.clone(),descriptor_revision:self.spec.descriptor_revision.clone(),budget:self.spec.budget.clone(),prepared_at:self.spec.approved_at,expires_at:self.expires_at}}
    /// Validate current native evidence while retaining the original review.
    pub(super)fn confirmation_current(&self,goal:&Record,latest:&Evidence,now:u64)->Result<()>{
        if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}
        if !source_state(goal,self.spec.continuation.as_ref())||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||!super::plans::matches(goal,&self.spec){return Err(Denial::RevisionChanged);}
        #[cfg(target_os="linux")]if let Some(reference)=&self.spec.continuation{self.retained_authority.as_ref().ok_or(Denial::Invalid)?.continuation_current(goal,reference).map_err(continuation_failure)?;}
        self.evidence.memory_current(goal,now)?;latest.current(goal,now)?;
        if latest.policy_revision!=self.evidence.policy_revision{return Err(Denial::PolicyChanged);}
        // Observation epochs/timestamps can refresh. The original identity, model,
        // semantic capabilities and policy must remain exactly the same.
        if !latest.same_memory(&self.evidence)||latest.engine_binding!=self.spec.engine_binding||latest.owner!=self.evidence.owner||latest.context!=self.evidence.context||latest.model!=self.evidence.model||latest.descriptor_semantics!=self.evidence.descriptor_semantics||latest.tokens!=self.evidence.tokens||latest.cost!=self.evidence.cost{return Err(Denial::RevisionChanged);}
        Ok(())
    }
    /// Consumed only after exact native consent; never from a JSON approval.
    pub(super)fn data_consent_current(&self,consent:Option<crate::pet_memory::context_model::GoalConsent>,now:u64)->Result<()>{
        if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}
        match(self.spec.memory_data.is_some(),consent){
            (true,Some(crate::pet_memory::context_model::GoalConsent::BoundedGoalContext))|(false,None)=>Ok(()),
            (true,None)=>Err(Denial::DataConsentRequired),_=>Err(Denial::Invalid),
        }
    }
    pub(super)fn approve(self,goal:&Record,latest:Evidence,now:u64)->Result<Approved>{self.approve_with_data(goal,latest,None,now)}
    pub(super)fn approve_with_data(mut self,goal:&Record,latest:Evidence,consent:Option<crate::pet_memory::context_model::GoalConsent>,now:u64)->Result<Approved>{
        self.data_consent_current(consent,now)?;self.confirmation_current(goal,&latest,now)?;
        #[cfg(target_os="linux")]let memory_confirmation=match self.evidence.memory.take(){
            Some(original)=>Some(original.confirm_against(goal,latest.memory.as_ref().ok_or(Denial::ContextChanged)?,consent,now).map_err(|_|Denial::ContextChanged)?),None=>None,
        };
        self.spec.approved_at=now;self.spec.evidence_hash=latest.fingerprint().map_err(|_|Denial::Invalid)?;
        Ok(Approved{#[cfg(target_os="linux")]memory_confirmation,#[cfg(target_os="linux")]retained_authority:self.retained_authority,spec:self.spec,expires_at:self.expires_at.min(latest.expires_at),evidence:latest})
    }
}
impl Approved{
    pub(super)fn spec(&self)->&AdmissionSpec{&self.spec}
    /// Revalidates freshness without changing the original consent bytes or TTL.
    pub(super)fn confirmation_current(&self,goal:&Record,latest:&Evidence,now:u64)->Result<()>{
        if now<self.spec.approved_at||now>=self.expires_at||now>=self.spec.budget.deadline_at{return Err(Denial::Expired);}
        self.current(goal,now).map_err(|_|Denial::RevisionChanged)?;
        if !source_state(goal,self.spec.continuation.as_ref()){return Err(Denial::NotDraft);}
        self.evidence.memory_current(goal,now)?;latest.current(goal,now)?;
        if latest.policy_revision!=self.evidence.policy_revision{return Err(Denial::PolicyChanged);}
        if !latest.same_memory(&self.evidence)||latest.engine_binding!=self.spec.engine_binding||latest.owner!=self.evidence.owner||latest.context!=self.evidence.context||latest.model!=self.evidence.model||latest.descriptor_semantics!=self.evidence.descriptor_semantics||latest.tokens!=self.evidence.tokens||latest.cost!=self.evidence.cost{return Err(Denial::RevisionChanged);}
        Ok(())
    }
    pub(super)fn current(&self,goal:&Record,now:u64)->model::Result<()>{
        goal.validate()?;
        #[cfg(target_os="linux")]match(&self.spec.memory_data,&self.memory_confirmation){
            (None,None)=>{},(Some(_),Some(data))=>data.current(goal,now).map_err(|_|model::Error::Conflict)?,_=>return Err(model::Error::Conflict),
        }
        if !source_state(goal,self.spec.continuation.as_ref())||now<self.spec.approved_at||now>=self.expires_at||now>=self.spec.budget.deadline_at||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||goal.spec.owner!=self.spec.owner||goal.execution_context.as_ref()!=Some(&self.spec.context)||model::fingerprint(&goal.spec)?!=self.spec.intent_hash||!super::plans::matches(goal,&self.spec){return Err(model::Error::Conflict);}Ok(())
    }
}

fn source_state(goal:&Record,continuation:Option<&super::continuation::Reference>)->bool{match continuation{None=>goal.state==State::Draft,Some(c)=>c.source_matches(goal)}}
#[cfg(target_os="linux")]fn continuation_failure(reason:super::authority::Failure)->Denial{match reason{super::authority::Failure::Busy|super::authority::Failure::Unavailable|super::authority::Failure::Uncertain=>Denial::NotReady,_=>Denial::RevisionChanged}}
#[cfg(target_os="linux")]
impl Review{
    pub(super)fn has_native_authority(&self)->bool{self.retained_authority.is_some()}
    pub(super)fn prepare_continuation(goal:&Record,evidence:Evidence,budget:Budget,limits:Option<super::planning::Limits>,config:&crate::pet_config::Configuration,captured:super::continuation::Captured,request:String,decision:String,now:u64)->Result<Self>{
        let(reference,authority)=captured.into_parts();
        if !reference.source_matches(goal){return Err(Denial::RecoveryNotClosed);}
        authority.continuation_current(goal,&reference).map_err(continuation_failure)?;
        if reference.uncertain_effects.is_some()&&limits.is_none(){return Err(Denial::Unsupported);}
        let policy=if let Some(limits)=limits{
            if !goal.spec.owner.local_engine()||budget.payment!=Payment::LocalOnly{return Err(Denial::Unsupported);}
            if !config.permissions(&goal.spec.owner.pet_id).map_err(|_|Denial::Invalid)?.music{return Err(Denial::Unsupported);}
            Some(super::planning::Policy::native(limits,&budget).map_err(|_|Denial::Budget)?)
        }else{None};
        let mut review=Self::prepare_source(goal,evidence,budget,request,decision,now,Some(reference))?;
        if let Some(policy)=policy{review.spec.schema_version=3;review.spec.planning_policy=Some(policy);}
        review.retained_authority=Some(authority);Ok(review)
    }
}
#[cfg(target_os="linux")]impl Approved{pub(super)fn take_native_authority(&mut self)->Option<super::authority::Authority>{self.retained_authority.take()}}
