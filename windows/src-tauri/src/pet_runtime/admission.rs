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
pub enum Payment{LocalOnly,ExistingAccount,RuntimeOnly}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Budget{
    pub deadline_at:u64,pub max_model_calls:u16,pub max_tool_calls:u16,pub max_output_bytes:u64,
    pub max_tokens:Option<u64>,pub max_cost_micros:Option<u64>,pub payment:Payment,
}
impl Budget{
    pub(super)fn valid_runtime_pool(&self,at:u64)->bool{let mut native=self.clone();native.max_model_calls=0;self.max_model_calls<=64&&native.valid_runtime(at)}
    pub(super)fn valid_runtime(&self,at:u64)->bool{
        self.payment==Payment::RuntimeOnly&&self.max_model_calls==0&&(1..=128).contains(&self.max_tool_calls)&&self.max_tokens.is_none()&&self.max_cost_micros.is_none()
        &&self.deadline_at>at&&self.deadline_at<=MAX_DATE&&self.deadline_at-at<=86_400_000&&(1..=16*1024*1024).contains(&self.max_output_bytes)
    }

    pub(super)fn valid_runtime_continuation(&self,at:u64)->bool{
        let mut bounded=self.clone();bounded.max_tool_calls=bounded.max_tool_calls.max(1);self.max_tool_calls<=128&&bounded.valid_runtime(at)
    }
    pub fn valid(&self,at:u64)->bool{
        self.payment!=Payment::RuntimeOnly&&self.deadline_at>at&&self.deadline_at<=MAX_DATE&&self.deadline_at-at<=86_400_000
        &&(1..=64).contains(&self.max_model_calls)&&self.max_tool_calls<=128
        &&(1..=16*1024*1024).contains(&self.max_output_bytes)
        &&self.max_tokens.is_none_or(|n|(1..=10_000_000).contains(&n))
        &&self.max_cost_micros.is_none_or(|n|(1..=1_000_000_000).contains(&n))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum QueuePhase{Queued,Paused,Cancelled,Leased,StopRequested,RecoveryRequired,Completed}
/// Native reviewed initial execution choice; JSON is DATA, not authority.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum StartScope{FiniteResponse,ResultDisclosure,NativePeerPlan,NativeSystemPlan}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ExecutionStart{
 #[serde(default,skip_serializing_if="Option::is_none")]pub not_before_at:Option<u64>,
 pub schema_version:u32,pub mode:super::planning::Start,pub scope:StartScope,
}
impl ExecutionStart{
 pub(super)fn valid_for(&self,s:&AdmissionSpec)->bool{
  (match(self.schema_version,self.not_before_at){(1,None)=>true,(2,Some(at))=>self.scope==StartScope::FiniteResponse&&at>0&&at<s.budget.deadline_at&&at<=MAX_DATE,_=>false})&&self.mode==super::planning::Start::WhenAvailable&&s.planning_policy.is_none()&&s.work_disclosure.is_none()
   &&match self.scope{
    StartScope::FiniteResponse=>matches!(s.schema_version,2|12|13|15|19|21|22|23|29)&&s.owner.local_engine()&&s.budget.payment==Payment::LocalOnly&&s.budget.max_model_calls==1&&s.budget.max_tool_calls==0&&s.result_disclosure.is_none()&&(s.schema_version!=2||s.continuation.is_none()),
    StartScope::NativePeerPlan=>s.native_peer_plan.as_ref().is_some_and(|p|p.for_spec(s)),
    StartScope::NativeSystemPlan=>s.system()&&s.native_system_plan.as_ref().is_some_and(|p|p.for_spec(s))&&s.resolved_model.is_empty()&&s.budget.payment==Payment::RuntimeOnly&&s.budget.max_model_calls==0,
    StartScope::ResultDisclosure=>matches!(s.schema_version,14|20)&&s.result_disclosure.as_ref().is_some_and(|p|p.for_spec(s))&&s.budget.payment==Payment::RuntimeOnly&&s.budget.max_model_calls==0&&s.budget.max_tool_calls==1,
   }
 }
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AdmissionSpec{
    #[serde(default,skip_serializing_if="Option::is_none")]pub rule_occurrence:Option<super::rules::OccurrenceReference>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_system_plan:Option<super::native_system_plan::Policy>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_peer_plan:Option<super::native_peer_plan::Policy>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub execution_start:Option<ExecutionStart>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub result_disclosure:Option<super::result_disclosure_policy::Policy>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub funding:Option<super::funding::Funding>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub work_disclosure:Option<super::work_disclosure::Policy>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_verification:Option<super::criteria::Binding>,
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
    pub(super)fn not_before(&self)->Option<u64>{self.execution_start.as_ref().and_then(|s|s.not_before_at)}
    pub(super)fn execution_due(&self,now:u64)->bool{self.not_before().is_none_or(|at|now>=at)}
    pub(super)fn planning(&self)->bool{matches!(self.schema_version,3|24|25|26)&&self.planning_policy.is_some()}
    pub(super)fn peer_delegation(&self)->bool{self.planning()&&self.planning_policy.as_ref().is_some_and(|p|p.peer_delegation())}
    pub(super)fn peer_iteration(&self)->bool{self.planning()&&self.planning_policy.as_ref().is_some_and(|p|p.peer_iteration())}
    pub(super)fn pool_budget(&self)->bool{matches!(self.schema_version,10|17)||self.peer_delegation()}
    pub(super)fn peer_memory(&self)->bool{self.schema_version==3&&self.planning_policy.as_ref().is_some_and(|p|p.peer_memory())}
    pub(super)fn peer_memory_iteration(&self)->bool{self.schema_version==24&&self.peer_iteration()}
    pub(super)fn peer_received_iteration(&self)->bool{matches!(self.schema_version,25|26)&&self.peer_iteration()}
    pub(super)fn peer_funded_iteration(&self)->bool{self.schema_version==26&&self.peer_iteration()}
    pub(super)fn peer_selected_iteration(&self)->bool{self.peer_memory_iteration()||self.peer_received_iteration()}
    pub(super)fn peer_selected_data(&self)->bool{self.peer_memory()||self.peer_selected_iteration()}
    pub(super)fn peer_round_destination_matches(&self,destination:&crate::pet_memory::context_model::Destination)->bool{
        self.peer_delegation()&&(if self.peer_selected_iteration(){self.memory_data.as_ref().is_some_and(|d|d.valid(&self.owner.adapter,&self.resolved_model)&&d.destination==*destination)}else{self.memory_data.is_none()})
    }
    pub(super)fn peer_planning(&self)->bool{super::planned_peer::enabled(self)}
    pub(super)fn funded_response(&self)->bool{matches!(self.schema_version,12|13|19|23)&&self.funding.is_some()}
    pub(super)fn finite_response(&self)->bool{if self.schema_version==29{return self.rule_occurrence.as_ref().is_some_and(|r|r.for_spec(self));}self.funded_response()||matches!(self.schema_version,15|21|22)||self.schema_version==2&&self.execution_start.as_ref().is_some_and(|p|p.scope==StartScope::FiniteResponse&&p.valid_for(self))}
    pub(super)fn system(&self)->bool{matches!(self.schema_version,27|28)}
    pub(super)fn native_system_requested(&self)->bool{self.system()&&self.execution_start.as_ref().is_some_and(|p|p.scope==StartScope::NativeSystemPlan&&p.valid_for(self))}
    pub(super)fn deterministic(&self)->bool{matches!(self.schema_version,4|5|6|7|8|9|10|11|14|16|17|18|20|27|28)}
    pub(super)fn valid_budget(&self)->bool{if matches!(self.schema_version,10|17){self.budget.valid_runtime_pool(self.approved_at)}else if matches!(self.schema_version,6|7|8|16){self.budget.valid_runtime_continuation(self.approved_at)}else if self.deterministic(){self.budget.valid_runtime(self.approved_at)}else{self.budget.valid(self.approved_at)}}

    pub(super)fn execution_requested(&self)->bool{self.execution_start.as_ref().is_some_and(|p|p.valid_for(self))||self.planning_policy.as_ref().is_some_and(|p|p.requested())||self.work_disclosure.is_some()}
    pub(super)fn binding_format(&self)->bool{
        if self.schema_version==29{return self.rule_occurrence.as_ref().is_some_and(|r|r.for_spec(self));}if self.rule_occurrence.is_some(){return false;}
        #[cfg(target_os="linux")]{
            if self.context.calendars.as_ref().map(|c|&c.selection)!=self.memory_data.as_ref().and_then(|d|d.calendar_selection.as_ref()){return false;}
            if self.context.has_calendars()&&(self.schema_version!=2||self.context.received_request.is_some()||self.planning_policy.is_some()||self.continuation.is_some()||self.budget.max_model_calls!=1||self.budget.max_tool_calls!=0||self.budget.payment!=Payment::LocalOnly){return false;}
        }

        if self.system(){return self.native_system_plan.as_ref().is_some_and(|p|p.for_spec(self))&&self.native_verification.as_ref().is_some_and(|b|b.schema_version==(if self.schema_version==28{7}else{5})&&b.valid()&&b.conditions.len()==b.criteria_count as usize&&b.spec_hash==self.intent_hash)&&self.context.schema_version==1&&self.context.mode==model::PreparationMode::Dedicated&&self.context.copied_messages==0&&!self.context.has_selected_data()&&self.execution_start.as_ref().is_none_or(|p|p.scope==StartScope::NativeSystemPlan&&p.valid_for(self))&&self.native_peer_plan.is_none()&&self.result_disclosure.is_none()&&self.funding.is_none()&&self.work_disclosure.is_none()&&self.memory_data.is_none()&&self.engine_binding.is_none()&&self.continuation.is_none()&&self.planning_policy.is_none()&&self.resolved_model.is_empty()&&self.valid_budget();}
        if self.native_system_plan.is_some(){return false;}
        if self.schema_version==24&&(!self.peer_memory_iteration()||self.context.memory.as_ref().is_none_or(|m|m.references.is_empty())||self.memory_data.as_ref().is_none_or(|d|d.schema_version!=1)){return false;}
        if self.schema_version==25&&(!self.peer_received_iteration()||self.context.received_request.as_ref().is_none_or(|r|!matches!(r.schema_version,1|3|5|6)||self.budget.deadline_at>r.source.deadline_at)||self.memory_data.as_ref().is_none_or(|d|d.received_request!=self.context.received_request)){return false;}
        if self.schema_version==26&&(!self.peer_funded_iteration()||self.context.received_request.as_ref().is_none_or(|r|!matches!(r.schema_version,2|4|7)||self.budget.deadline_at>r.source.deadline_at)||self.memory_data.as_ref().is_none_or(|d|d.received_request!=self.context.received_request)||self.funding.as_ref().is_none_or(|f|f.schema_version!=9)){return false;}
        if self.peer_planning()&&((if self.peer_received_iteration(){self.context.mode!=model::PreparationMode::Dedicated||self.context.received_request.is_none()}else if self.peer_selected_data(){self.context.mode!=model::PreparationMode::Dedicated||self.context.memory.is_none()||self.context.received_request.is_some()}else{self.context.has_selected_data()})||self.context.copied_messages!=0||!self.owner.local_engine()||self.budget.payment!=Payment::LocalOnly||self.continuation.is_some()){return false;}
        if self.peer_delegation()&&self.context.mode!=model::PreparationMode::Dedicated{return false;}
        if self.execution_start.as_ref().is_some_and(|p|!p.valid_for(self))||self.native_peer_plan.as_ref().is_some_and(|p|!p.for_spec(self)||self.execution_start.as_ref().is_none_or(|s|s.scope!=StartScope::NativePeerPlan)){return false;}
        if self.schema_version==23&&(!self.owner.local_engine()||self.context.schema_version!=9||self.context.received_request.as_ref().is_none_or(|r|r.schema_version!=7||self.budget.deadline_at>r.source.deadline_at)||self.memory_data.as_ref().is_none_or(|g|g.schema_version!=8)||self.budget.payment!=Payment::LocalOnly||self.budget.max_model_calls!=1||self.budget.max_tool_calls!=0||self.continuation.is_some()||self.planning_policy.is_some()||self.funding.as_ref().is_none_or(|f|f.schema_version!=1)){return false;}
        if self.schema_version==19&&(!self.owner.local_engine()||self.context.schema_version!=6||self.context.received_request.as_ref().is_none_or(|r|r.schema_version!=4||self.budget.deadline_at>r.source.deadline_at)||self.memory_data.as_ref().is_none_or(|d|d.schema_version!=5)||self.budget.payment!=Payment::LocalOnly||self.budget.max_model_calls!=1||self.budget.max_tool_calls!=0||self.continuation.is_some()||self.planning_policy.is_some()){return false;}
        if matches!(self.schema_version,15|21|22)&&(!self.owner.local_engine()||self.context.schema_version!=(if self.schema_version==22{8}else if self.schema_version==21{7}else{5})||self.context.received_request.as_ref().is_none_or(|r|r.schema_version!=(if self.schema_version==22{6}else if self.schema_version==21{5}else{3})||self.budget.deadline_at>r.source.deadline_at)||self.memory_data.as_ref().is_none_or(|d|d.schema_version!=(if self.schema_version==22{7}else if self.schema_version==21{6}else{4}))||self.budget.payment!=Payment::LocalOnly||self.budget.max_model_calls!=1||self.budget.max_tool_calls!=0||self.continuation.is_some()||self.planning_policy.is_some()){return false;}
        if matches!(self.schema_version,14|20)!=self.result_disclosure.is_some()||self.result_disclosure.as_ref().is_some_and(|p|!p.for_spec(self)){return false;}
        if matches!(self.schema_version,11|12|13|14|16|17|18|19|20|23|26)!=self.funding.is_some()||self.funding.as_ref().is_some_and(|f|!f.valid_for(self)){return false;}
        if matches!(self.schema_version,12|13)&&(!self.owner.local_engine()||self.context.schema_version!=4||self.context.received_request.as_ref().is_none_or(|r|r.schema_version!=2)||self.memory_data.as_ref().is_none_or(|d|d.schema_version!=3)||self.budget.payment!=Payment::LocalOnly||self.budget.max_model_calls!=1||self.budget.max_tool_calls!=0||(if self.schema_version==12{self.continuation.is_some()}else{self.continuation.as_ref().is_none_or(|c|c.schema_version!=6)})||self.planning_policy.is_some()){return false;}
        if (self.schema_version==9)!=self.work_disclosure.is_some()||self.work_disclosure.as_ref().is_some_and(|p|!p.valid()||self.proposal_hash.as_ref()!=Some(&p.proposal_hash)||p.arguments.expires_at>self.budget.deadline_at||p.arguments.expires_at<=self.approved_at){return false;}
        // References alone remain insufficient. This typed scope/destination is
        // certified only from the original native Approved, never from JSON.
        if self.deterministic(){return (match(self.schema_version,&self.native_verification){(4,None)|(10,None)|(11,None)|(14,None)|(16,None)|(17,None)|(18,None)|(20,None)=>true,(5,Some(b))|(6,Some(b))|(7,Some(b))|(8,Some(b))|(9,Some(b))|(10,Some(b))|(11,Some(b))|(14,Some(b))|(16,Some(b))|(17,Some(b))|(18,Some(b))|(20,Some(b))=>b.schema_version==2&&b.valid()&&b.spec_hash==self.intent_hash,(6|7|8,None)=>true,_=>false})&&(if matches!(self.schema_version,14|20){self.context.schema_version==(if self.schema_version==20{6}else{4})&&self.result_disclosure.is_some()}else{!self.context.has_selected_data()})&&self.memory_data.is_none()&&self.engine_binding.is_none()&&(if matches!(self.schema_version,6|7|8|16){self.continuation.as_ref().is_some_and(|c|c.schema_version==match self.schema_version{16=>8,8=>5,7=>4,_=>3}&&c.basic())}else if matches!(self.schema_version,14|20){self.continuation.as_ref().is_some_and(|c|c.schema_version==(if self.schema_version==20{10}else{7})&&c.basic())}else{self.continuation.is_none()})&&self.planning_policy.is_none()&&self.proposal_hash.as_deref().is_some_and(hash)&&self.resolved_model.is_empty()&&self.valid_budget();}
        if self.native_verification.is_some()||self.budget.payment==Payment::RuntimeOnly{return false;}
        match (self.context.has_selected_data(),&self.memory_data){
            (false,None)=>{},
            (true,Some(data))if self.owner.local_engine()&&matches!(self.schema_version,2|3|12|13|15|19|21|22|23|24|25|26)&&data.valid(&self.owner.adapter,&self.resolved_model)&&data.received_request==self.context.received_request
             &&(matches!((self.context.schema_version,data.schema_version),(10,9)|(11,10))&&self.context.has_calendars()&&data.has_calendars()||self.context.schema_version==2&&self.context.memory.is_some()&&data.schema_version==1||self.context.schema_version==3&&self.context.received_request.is_some()&&data.schema_version==2||matches!(self.schema_version,12|13)&&self.context.schema_version==4&&data.schema_version==3||self.schema_version==15&&self.context.schema_version==5&&data.schema_version==4||self.schema_version==19&&self.context.schema_version==6&&data.schema_version==5||self.schema_version==21&&self.context.schema_version==7&&data.schema_version==6||self.schema_version==22&&self.context.schema_version==8&&data.schema_version==7||self.schema_version==23&&self.context.schema_version==9&&data.schema_version==8||self.schema_version==25&&matches!((self.context.schema_version,data.schema_version),(3,2)|(5,4)|(7,6)|(8,7))||self.schema_version==26&&matches!((self.context.schema_version,data.schema_version),(4,3)|(6,5)|(9,8)))=>{},
            _=>return false,
        }
        if self.owner.adapter=="lmstudio"{if self.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&self.resolved_model)){return false;}}
        else if self.engine_binding.is_some(){return false;}
        if self.continuation.as_ref().is_some_and(|c|matches!(c.schema_version,3|4|5|9|10)||!c.basic()||!matches!(self.schema_version,2|3|13)||self.schema_version==13&&c.schema_version!=6||self.schema_version!=13&&c.schema_version==6){return false;}
        if self.continuation.as_ref().is_some_and(|c|c.uncertain_effects.is_some())&&(self.schema_version!=3||!self.owner.local_engine()||self.budget.payment!=Payment::LocalOnly||self.planning_policy.as_ref().is_none_or(|p|p.catalog!=super::planning::Catalog::SpotifyTransportV1)){return false;}
        match (self.schema_version,self.proposal_hash.as_deref(),self.planning_policy.as_ref()){
            (1,None,None)=>true,
            (2,Some(h),None)=>hash(h),
            (12|13|15|19|21|22|23,Some(h),None)=>hash(h),
            (3|24|25|26,Some(h),Some(p))=>hash(h)&&p.valid_for(&self.budget)&&self.owner.local_engine()&&self.budget.payment==Payment::LocalOnly,
            _=>false,
        }
    }
    pub fn valid(&self,record:&Record)->bool{
        // Received planning has its own exact selected-data scope. The legacy
        // derived-source formats below remain finite unless that scope is25.
        let received_planning=self.peer_received_iteration();
        ((self.schema_version==23)==(!received_planning&&record.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==7)&&!self.deterministic()))&&((self.schema_version==22)==(!received_planning&&record.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==6)))&&((self.schema_version==15)==(!received_planning&&record.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==3)))&&((self.schema_version==21)==(!received_planning&&record.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==5)))&&(self.funding.as_ref().is_some_and(|f|f.matches(record))==record.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some()))&&((self.work_disclosure.is_some())==record.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.as_ref().is_some_and(|o|o.operation_id=="pet.workReport"&&o.version==2)))&&self.binding_format()&&self.native_system_plan.as_ref().is_none_or(|p|p.matches(record,self))&&self.native_peer_plan.as_ref().is_none_or(|p|p.matches(record,self))&&self.result_disclosure.as_ref().is_none_or(|p|p.matches(record))&&self.work_disclosure.as_ref().is_none_or(|p|p.matches(record))&&self.native_verification.as_ref().is_none_or(|b|b.matches(record))&&self.continuation.as_ref().is_none_or(|c|c.present(record,self))&&model::key(&self.request_id)&&model::token(&self.decision_id)
        &&self.goal_id==record.id&&model::token(&self.source_revision)&&self.source_sequence>0&&self.source_sequence<record.sequence
        &&self.owner==record.spec.owner&&self.context.valid(record)&&record.execution_context.as_ref()==Some(&self.context)
        &&self.context_hash==model::fingerprint(&self.context).unwrap_or_default()&&hash(&self.context_hash)
        &&self.intent_hash==model::fingerprint(&record.spec).unwrap_or_default()&&hash(&self.intent_hash)
        &&(if self.deterministic(){self.resolved_model.is_empty()&&(if self.system(){self.native_system_plan.as_ref().is_some_and(|p|p.matches(record,self))}else{super::deterministic::plan(record)})}else{model_id(&self.resolved_model)&&(!self.finite_response()||super::plans::original_response_plan(record,self))})&&!self.descriptor_revision.is_empty()&&self.descriptor_revision.len()<=256
        &&hash(&self.policy_revision)&&hash(&self.evidence_hash)&&self.approved_at>=record.created_at&&self.approved_at<=record.updated_at
        &&self.valid_budget()&&(self.budget.payment!=Payment::LocalOnly||self.owner.local_engine())
    }
}

/// A fresh, typed observation supplied by the native adapter, never by a model
/// or by a deserialized admission. Actual writer checks remain mandatory.
pub(super)struct Evidence{#[cfg(target_os="linux")]disclosure:Option<super::result_delivery_data::Captured>,system:bool,deterministic:bool,#[cfg(target_os="linux")]memory:Option<super::memory_review::Captured>,engine_binding:Option<crate::local_chat::lmstudio::Binding>,owner:Owner,context:ContextBinding,model:String,descriptor_revision:String,descriptor_semantics:String,policy_revision:String,observed_at:u64,expires_at:u64,tokens:bool,cost:bool}
/// Passive native identity/configuration evidence. It cannot be passed to
/// admission, model IO or DATA confirmation, even when the provider is offline.
#[cfg(target_os="linux")]
pub(super)struct OwnerResumptionEvidence{evidence:Evidence}
#[cfg(target_os="linux")]
impl OwnerResumptionEvidence{
    pub(super)fn observe(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,memory:Option<super::memory_review::Captured>)->Result<Self>{
        let row=goal.admissions.last().filter(|r|goal.state==State::RecoveryRequired&&r.spec.peer_iteration()).ok_or(Denial::NotReady)?;
        let mut evidence=Evidence::observe_base(goal,d,policy_revision,now,false,false,memory.is_some(),false)?;
        // Absence of live catalog metadata does not change the original engine
        // instance. An observed different instance still refuses this handoff.
        if d.binding.engine_binding.as_ref().is_some_and(|b|Some(b)!=row.spec.engine_binding.as_ref()){return Err(Denial::OwnerChanged);}
        evidence.engine_binding=row.spec.engine_binding.clone();
        if let Some(memory)=memory{
            memory.current(goal,now).map_err(|_|Denial::ContextChanged)?;
            let preview=memory.preview();
            if preview.pet_id!=goal.spec.owner.pet_id||preview.goal_id!=goal.id||preview.destination.provider()!=goal.spec.owner.adapter||!matches!(&preview.destination,crate::pet_memory::context_model::Destination::Server{model,..}if model==&evidence.model){return Err(Denial::ContextChanged);}
            evidence.memory=Some(memory);
        }
        evidence.for_claim_state(goal,now,true)?;Ok(Self{evidence})
    }
    pub(super)fn for_owner_resumption(&self,goal:&Record,now:u64)->Result<(String,String)>{self.evidence.for_claim_state(goal,now,true)}
}
impl Evidence{
    pub(super)fn for_claim(&self,goal:&Record,now:u64)->Result<(String,String)>{
        self.for_claim_state(goal,now,false)
    }
    fn for_claim_state(&self,goal:&Record,now:u64,resuming:bool)->Result<(String,String)>{
        self.current(goal,now)?;if now<goal.updated_at{return Err(Denial::Expired);}
        let row=goal.admissions.last().ok_or(Denial::NotReady)?;
        if !(if resuming{goal.state==State::RecoveryRequired&&row.spec.peer_iteration()}else{matches!(goal.state,State::Queued|State::Leased)}){return Err(Denial::NotReady);}
        if self.result_policy()!=row.spec.result_disclosure||self.system!=row.spec.system()||self.deterministic!=row.spec.deterministic()||self.memory_grant()!=row.spec.memory_data||self.owner!=row.spec.owner||self.model!=row.spec.resolved_model||self.engine_binding!=row.spec.engine_binding{return Err(Denial::OwnerChanged);}
        if self.policy_revision!=row.spec.policy_revision{return Err(Denial::PolicyChanged);}
        if row.spec.budget.max_tokens.is_some()&&!self.tokens||row.spec.budget.max_cost_micros.is_some()&&!self.cost{return Err(Denial::MeteringUnavailable);}
        if now>=row.spec.budget.deadline_at||now<row.spec.approved_at{return Err(Denial::Expired);}
        Ok((self.descriptor_revision.clone(),self.policy_revision.clone()))
    }
    pub(super)fn observe(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool)->Result<Self>{
        Self::observe_base(goal,d,policy_revision,now,tokens,cost,false,true)
    }
    fn observe_base(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,has_native_memory:bool,inference:bool)->Result<Self>{
        goal.validate().map_err(|_|Denial::Invalid)?;
        if d.schema_version!=1||!d.configuration_available||d.lifecycle!=Lifecycle::Active{return Err(Denial::Inactive);}
        if d.pet_id!=goal.spec.owner.pet_id||d.binding.provider.as_deref()!=Some(&goal.spec.owner.provider)||d.binding.adapter!=Some(goal.spec.owner.adapter.as_str()){return Err(Denial::OwnerChanged);}
        let context=goal.execution_context.clone().ok_or(Denial::ContextRequired)?;
        if context.has_selected_data()&&!has_native_memory{return Err(Denial::Unsupported);}// References alone cannot manufacture native data review.
        let model=d.binding.model.clone().filter(|m|model_id(m)).ok_or(Denial::ModelUnresolved)?;
        if !hash(&policy_revision)||d.revision.is_empty()||d.revision.len()>256{return Err(Denial::Invalid);}
        let(at,end)=if inference{
        let mut caps=d.capabilities.iter().filter(|c|c.id=="conversation.respond");let c=caps.next().ok_or(Denial::Unsupported)?;
        if caps.next().is_some()||c.support!=Support::Supported||!c.callers.contains(&Caller::TaskRuntime){return Err(Denial::Unsupported);}
        if c.availability!=Availability::Ready||c.reason.is_some(){return Err(Denial::NotReady);}
        if c.model_binding.as_deref().is_some_and(|m|m!=model){return Err(Denial::ModelUnresolved);}
        if goal.spec.owner.adapter=="lmstudio"&&d.binding.engine_binding.as_ref().is_none_or(|b|!b.valid_for(&model)){return Err(Denial::ModelUnresolved);}
        let at=c.observed_at.ok_or(Denial::Stale)?;let end=c.expires_at.ok_or(Denial::Stale)?;
        if at==0||at>now||now>=end||end>MAX_DATE||now-at>15_000{return Err(Denial::Stale);}
        (at,end)
        }else{if now==0||now>MAX_DATE{return Err(Denial::Invalid);}(now,now.saturating_add(15_000).min(MAX_DATE))};
        Ok(Self{#[cfg(target_os="linux")]disclosure:None,system:false,deterministic:false,#[cfg(target_os="linux")]memory:None,engine_binding:d.binding.engine_binding.clone(),owner:goal.spec.owner.clone(),context,model,descriptor_revision:d.revision.clone(),descriptor_semantics:crate::pet_capabilities::admission_semantics(d).map_err(|_|Denial::Invalid)?,policy_revision,observed_at:at,expires_at:end,tokens,cost})
    }
    fn fingerprint(&self)->model::Result<String>{
        let legacy=(||->model::Result<String>{if let Some(binding)=&self.engine_binding{return model::fingerprint(&(binding,&self.owner,&self.context,&self.model,&self.descriptor_revision,&self.descriptor_semantics,&self.policy_revision,self.observed_at,self.expires_at,self.tokens,self.cost));}model::fingerprint(&(&self.owner,&self.context,&self.model,&self.descriptor_revision,&self.descriptor_semantics,&self.policy_revision,self.observed_at,self.expires_at,self.tokens,self.cost))})();
        let legacy=legacy?;let legacy=if self.system{model::fingerprint(&("coucou-native-system-evidence-v1",legacy))?}else{legacy};let legacy=match self.result_policy(){Some(p)=>model::fingerprint(&("coucou-derived-result-evidence-v1",legacy,p))?,None=>legacy};let legacy=if self.deterministic{model::fingerprint(&("coucou-deterministic-evidence-v1",legacy))?}else{legacy};match self.memory_preview(){Some(data)=>model::fingerprint(&(legacy,data)),None=>Ok(legacy)}
    }
    fn result_policy(&self)->Option<super::result_disclosure_policy::Policy>{
        #[cfg(target_os="linux")]{self.disclosure.as_ref().map(|d|d.policy().clone())}
        #[cfg(not(target_os="linux"))]{None}
    }
    #[cfg(target_os="linux")]fn result_preview(&self)->Option<super::result_disclosure::Preview>{self.disclosure.as_ref().map(|d|d.preview().clone())}
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
        #[cfg(target_os="linux")]if let Some(d)=&self.disclosure{return Some(d.preview().expires_at);}
        #[cfg(target_os="linux")]{self.memory.as_ref().map(|m|m.expires_at())}
        #[cfg(not(target_os="linux"))]{None}
    }
    fn memory_current(&self,goal:&Record,now:u64)->Result<()>{
        #[cfg(target_os="linux")]if let Some(d)=&self.disclosure{return d.current(goal,now).map_err(|_|Denial::ContextChanged);}
        #[cfg(target_os="linux")]if let Some(memory)=&self.memory{return memory.current(goal,now).map_err(|_|Denial::ContextChanged);}
        if self.context.has_selected_data(){return Err(Denial::Unsupported);}Ok(())
    }
    fn same_memory(&self,other:&Self)->bool{
        if self.result_policy()!=other.result_policy(){return false;}
        #[cfg(target_os="linux")]{match(&self.memory,&other.memory){(None,None)=>true,(Some(a),Some(b))=>a.same_data(b),_=>false}}
        #[cfg(not(target_os="linux"))]{let _=other;true}
    }
    #[cfg(target_os="linux")]
    pub(super)fn from_memory_capture(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,memory:Option<super::memory_review::Captured>)->Result<Self>{
        if memory.is_none(){return Self::observe(goal,d,policy_revision,now,tokens,cost);}
        let memory=memory.unwrap();memory.current(goal,now).map_err(|_|Denial::ContextChanged)?;
        let preview=memory.preview();
        if preview.pet_id!=goal.spec.owner.pet_id||preview.goal_id!=goal.id||preview.destination.provider()!=goal.spec.owner.adapter||!matches!(&preview.destination,crate::pet_memory::context_model::Destination::Server{model,..}if Some(model)==d.binding.model.as_ref()){return Err(Denial::ContextChanged);}
        let mut evidence=Self::observe_base(goal,d,policy_revision,now,tokens,cost,true,true)?;evidence.memory=Some(memory);Ok(evidence)
    }
    #[cfg(target_os="linux")]
    pub(super)fn observe_native(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,settings:&crate::settings::Settings,config:&crate::pet_config::Configuration)->Result<Self>{
        Self::observe_native_at(goal,d,policy_revision,now,tokens,cost,settings,config,&crate::pet_memory::context::Capture::catalog_path())
    }
    #[cfg(target_os="linux")]
    pub(super)fn observe_native_at(goal:&Record,d:&Descriptor,policy_revision:String,now:u64,tokens:bool,cost:bool,settings:&crate::settings::Settings,config:&crate::pet_config::Configuration,path:&std::path::Path)->Result<Self>{
        if let Some(p)=goal.admissions.last().and_then(|r|r.spec.result_disclosure.as_ref()){
            let c=config.clone();let s=settings.clone();let observe:super::result_disclosure::Observe=std::sync::Arc::new(move||Ok((c.clone(),s.clone())));
            let data=super::result_delivery_data::Captured::at(&super::admission_review::authority_root(),path,goal,p.clone(),observe,now).map_err(|_|Denial::ContextChanged)?;
            return Self::observe_result(goal,d,policy_revision,now,data);
        }
        if !goal.execution_context.as_ref().is_some_and(|b|b.has_selected_data()){return Self::observe(goal,d,policy_revision,now,tokens,cost);}
        let model=d.binding.model.as_deref().filter(|m|model_id(m)).ok_or(Denial::ModelUnresolved)?;
        let server=super::memory_review::server_for_goal(goal,config,settings,model).map_err(|_|Denial::ContextChanged)?;
        let memory=super::memory_review::Captured::at_with_authority(path,Some(&super::admission_review::authority_root()),goal,&server,model,now).map_err(|_|Denial::ContextChanged)?;
        Self::from_memory_capture(goal,d,policy_revision,now,tokens,cost,memory)
    }
    fn peer_forwarding_supported(&self)->bool{
        #[cfg(target_os="linux")]{self.memory.as_ref().is_some_and(|m|m.peer_forwarding_supported())}
        #[cfg(not(target_os="linux"))]{false}
    }
    fn current(&self,goal:&Record,now:u64)->Result<()>{
        if goal.spec.owner!=self.owner{return Err(Denial::OwnerChanged);}
        if goal.execution_context.as_ref()!=Some(&self.context){return Err(Denial::ContextChanged);}
        if now<self.observed_at||now>=self.expires_at||now-self.observed_at>15_000{return Err(Denial::Stale);}self.memory_current(goal,now)
    }
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct ReviewProposal{
    #[serde(skip_serializing_if="Option::is_none")]pub native_system_plan:Option<super::native_system_plan::Policy>,
    #[serde(skip_serializing_if="Option::is_none")]pub native_peer_plan:Option<super::native_peer_plan::Policy>,
    #[serde(skip_serializing_if="Option::is_none")]pub execution_start:Option<ExecutionStart>,
    #[serde(skip_serializing_if="Option::is_none")]pub result_disclosure:Option<super::result_disclosure_policy::Policy>,
    #[cfg(target_os="linux")]#[serde(skip_serializing_if="Option::is_none")]pub result_data:Option<super::result_disclosure::Preview>,
    #[serde(skip_serializing_if="Option::is_none")]pub funding:Option<super::funding::Funding>,
    #[serde(skip_serializing_if="Option::is_none")]pub work_disclosure:Option<super::work_disclosure::Policy>,
    #[serde(skip_serializing_if="Option::is_none")]pub native_verification:Option<super::criteria::Binding>,#[serde(skip_serializing_if="Option::is_none")]pub restored_from:Option<super::restored_future::Origin>,#[serde(skip_serializing_if="Option::is_none")]pub memory_data:Option<crate::pet_memory::context_model::GoalPreview>,#[serde(skip_serializing_if="Option::is_none")]pub engine_binding:Option<crate::local_chat::lmstudio::Binding>,#[serde(skip_serializing_if="Option::is_none")]pub continuation:Option<super::continuation::Reference>,#[serde(skip_serializing_if="Option::is_none")]pub planning_policy:Option<super::planning::Policy>,pub proposals:super::plans::Snapshot,pub operation:&'static str,pub tool_execution_available:bool,pub goal_id:String,pub revision:String,pub request_id:String,pub owner:Owner,pub context:ContextBinding,pub intent:String,pub criteria:Vec<String>,pub resolved_model:String,pub policy_revision:String,pub descriptor_revision:String,pub budget:Budget,pub prepared_at:u64,pub expires_at:u64}
pub(super)struct Review{#[cfg(target_os="linux")]system:Option<std::sync::Arc<super::native_system_plan::Captured>>,#[cfg(target_os="linux")]peer:Option<std::sync::Arc<super::native_peer_plan::Captured>>,restored_from:Option<super::restored_future::Origin>,#[cfg(target_os="linux")]retained_authority:Option<super::authority::Authority>,spec:AdmissionSpec,expires_at:u64,evidence:Evidence,intent:String,criteria:Vec<String>,proposals:super::plans::Snapshot}
pub(super)struct Approved{#[cfg(target_os="linux")]system:Option<std::sync::Arc<super::native_system_plan::Captured>>,#[cfg(target_os="linux")]peer:Option<std::sync::Arc<super::native_peer_plan::Captured>>,#[cfg(target_os="linux")]memory_confirmation:Option<super::memory_review::ConfirmedData>,#[cfg(target_os="linux")]retained_authority:Option<super::authority::Authority>,spec:AdmissionSpec,expires_at:u64,evidence:Evidence}
impl Review{
    pub(super)fn prepare(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64)->Result<Self>{
        Self::prepare_source(goal,evidence,budget,request_id,decision_id,now,None)
    }
    fn prepare_source(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64,continuation:Option<super::continuation::Reference>)->Result<Self>{
        Self::prepare_source_mode(goal,evidence,budget,request_id,decision_id,now,continuation,None)
    }
    fn prepare_source_mode(goal:&Record,evidence:Evidence,budget:Budget,request_id:String,decision_id:String,now:u64,continuation:Option<super::continuation::Reference>,received_planning:Option<super::planning::Policy>)->Result<Self>{
        if evidence.system{return Err(Denial::Unsupported);}
        goal.validate().map_err(|_|Denial::Invalid)?;if now<goal.updated_at{return Err(Denial::Expired);}
        if !source_state(goal,continuation.as_ref()){return Err(Denial::NotDraft);}
        let received_iteration=received_planning.is_some();let funded_iteration=received_iteration&&goal.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some());
        if received_iteration&&(received_planning.as_ref().is_none_or(|p|!p.peer_iteration())||continuation.is_some()||evidence.deterministic||goal.spec.received_from.as_ref().is_none_or(|o|!matches!(o.body.schema_version,1|2|3|4|5|6|7))||evidence.context.received_request.is_none()||!evidence.peer_forwarding_supported()||goal.spec.received_from.as_ref().is_some_and(|o|budget.deadline_at>o.body.source.deadline_at)){return Err(Denial::Unsupported);}
        let derived_funded=goal.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==7);if derived_funded&&continuation.is_some(){return Err(Denial::Unsupported);}
        let planned_source=goal.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==6);let recursive_result_source=goal.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==5);let result_source=goal.spec.received_from.as_ref().is_some_and(|o|matches!(o.body.schema_version,3|5|6));
        if result_source&&!received_iteration&&(evidence.deterministic||continuation.is_some()||evidence.context.schema_version!=(if planned_source{8}else if recursive_result_source{7}else{5})||evidence.memory_grant().as_ref().is_none_or(|m|m.schema_version!=(if planned_source{7}else if recursive_result_source{6}else{4}))||budget.max_model_calls!=1||budget.max_tool_calls!=0||budget.payment!=Payment::LocalOnly||goal.spec.received_from.as_ref().is_some_and(|o|budget.deadline_at>o.body.source.deadline_at)){return Err(Denial::Unsupported);}
        let result_disclosure=evidence.result_policy();
        let mut funding=super::funding::Funding::from_goal(goal).map_err(|_|Denial::Invalid)?;let recursive_pool=funding.is_some()&&evidence.deterministic&&super::delegation::is_pool(goal);let recursive_source=goal.spec.received_from.as_ref().is_some_and(|o|o.body.schema_version==4);let recursive_disclosure=recursive_source&&evidence.deterministic&&result_disclosure.as_ref().is_some_and(|p|p.schema_version==2)&&continuation.as_ref().is_some_and(|c|c.schema_version==10);let recursive_leaf=recursive_source&&evidence.deterministic&&!recursive_pool&&!recursive_disclosure;let recursive_response=recursive_source&&!evidence.deterministic&&!received_iteration;if (recursive_pool||recursive_leaf||recursive_response)&&continuation.is_some(){return Err(Denial::Unsupported);}if recursive_pool{funding.as_mut().ok_or(Denial::Invalid)?.schema_version=5;}if recursive_response{funding.as_mut().ok_or(Denial::Invalid)?.schema_version=7;}if funding.is_some()&&(continuation.as_ref().is_some_and(|c|c.schema_version!=6&&!(c.schema_version==7&&result_disclosure.is_some())&&!(c.schema_version==8&&evidence.deterministic)&&!recursive_disclosure)||super::delegation::is_pool(goal)&&!recursive_pool||goal.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.as_ref().is_some_and(|o|o.operation_id=="pet.workReport"))){return Err(Denial::Unsupported);}
        if let(Some(f),Some(c))=(&mut funding,&continuation){if recursive_disclosure{f.schema_version=8;}else if c.schema_version==8&&evidence.deterministic&&result_disclosure.is_none(){f.schema_version=4;}else if c.schema_version==7&&evidence.deterministic&&result_disclosure.is_some(){f.schema_version=3;}else if c.schema_version==6&&!evidence.deterministic{f.schema_version=2;}else{return Err(Denial::Unsupported);}f.previous=c.previous.clone();}
        if funded_iteration{funding.as_mut().ok_or(Denial::Invalid)?.schema_version=9;}
        if funding.is_some()&&!evidence.deterministic&&!received_iteration{
            if !goal.spec.owner.local_engine()||goal.plans.is_empty()||goal.plans.iter().flat_map(|p|&p.steps).any(|s|s.operation.is_some()){return Err(Denial::Unsupported);}
            if budget.payment!=Payment::LocalOnly||budget.max_model_calls!=1||budget.max_tool_calls!=0{return Err(Denial::Budget);}
            if evidence.context.schema_version!=(if derived_funded{9}else if recursive_response{6}else{4})||evidence.context.received_request.as_ref().is_none_or(|s|s.schema_version!=(if derived_funded{7}else if recursive_response{4}else{2}))||evidence.memory_grant().as_ref().is_none_or(|g|g.schema_version!=(if derived_funded{8}else if recursive_response{5}else{3})){return Err(Denial::DataConsentRequired);}
        }
        evidence.current(goal,now)?;if super::delegation::is_pool(goal)&&(!evidence.deterministic||continuation.is_some()){return Err(Denial::Unsupported);}
        if !(if evidence.deterministic&&continuation.is_some(){budget.valid_runtime_continuation(now)}else if evidence.deterministic&&super::delegation::is_pool(goal){budget.valid_runtime_pool(now)}else if evidence.deterministic{budget.valid_runtime(now)}else{budget.valid(now)}){return Err(Denial::Budget);}
        if budget.payment==Payment::LocalOnly&&!goal.spec.owner.local_engine(){return Err(Denial::PaymentPolicy);}
        if budget.max_tokens.is_some()&&!evidence.tokens||budget.max_cost_micros.is_some()&&!evidence.cost{return Err(Denial::MeteringUnavailable);}
        if !model::key(&request_id)||!model::token(&decision_id)||goal.events.iter().any(|e|e.request_id==request_id){return Err(Denial::Invalid);}
        super::plans::reviewed_input(goal).map_err(|_|Denial::InputLimit)?;evidence.reviewed_memory_input(goal,now)?;
        let expires_at=now.saturating_add(REVIEW_TTL).min(budget.deadline_at).min(evidence.memory_expiry().unwrap_or(u64::MAX));
        let native_verification=if evidence.deterministic{super::criteria::Binding::native_for(goal).map_err(|_|Denial::Invalid)?}else{None};
        let spec=AdmissionSpec{rule_occurrence:None,native_system_plan:None,native_peer_plan:None,execution_start:None,result_disclosure:result_disclosure.clone(),funding,work_disclosure:None,schema_version:if funded_iteration{26}else if received_iteration{25}else if derived_funded&&!evidence.deterministic{23}else if recursive_disclosure{20}else if recursive_response{19}else if recursive_pool{17}else if recursive_leaf{18}else if result_source{if planned_source{22}else if recursive_result_source{21}else{15}}else if result_disclosure.is_some(){14}else if goal.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some()){if evidence.deterministic{if continuation.is_some(){16}else{11}}else if continuation.is_some(){13}else{12}}else if evidence.deterministic&&super::delegation::is_pool(goal)&&continuation.is_none(){10}else if evidence.deterministic&&continuation.is_some(){match continuation.as_ref().map(|c|c.schema_version){Some(5)=>8,Some(4)=>7,_=>6}}else if native_verification.is_some(){5}else if evidence.deterministic{4}else{2},native_verification,memory_data:evidence.memory_grant(),engine_binding:evidence.engine_binding.clone(),continuation,planning_policy:received_planning,proposal_hash:Some(super::plans::fingerprint(goal).map_err(|_|Denial::Invalid)?),request_id,decision_id,goal_id:goal.id.clone(),source_revision:goal.revision.clone(),source_sequence:goal.sequence,owner:goal.spec.owner.clone(),context:evidence.context.clone(),context_hash:model::fingerprint(&evidence.context).map_err(|_|Denial::Invalid)?,intent_hash:model::fingerprint(&goal.spec).map_err(|_|Denial::Invalid)?,resolved_model:evidence.model.clone(),descriptor_revision:evidence.descriptor_revision.clone(),policy_revision:evidence.policy_revision.clone(),evidence_hash:evidence.fingerprint().map_err(|_|Denial::Invalid)?,approved_at:now,budget};
        if spec.funding.as_ref().is_some_and(|f|!f.valid_for(&spec)){return Err(Denial::Budget);}
        Ok(Self{#[cfg(target_os="linux")]system:None,#[cfg(target_os="linux")]peer:None,restored_from:goal.spec.restored_from.clone(),#[cfg(target_os="linux")]retained_authority:None,spec,expires_at,evidence,intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),proposals:super::plans::snapshot(goal)})
    }
    /// Called only with the native configuration/evidence snapshot. A caller
    /// selects numeric bounds; it cannot supply the catalog or a grant.
    pub(super)fn prepare_planning(goal:&Record,evidence:Evidence,budget:Budget,limits:super::planning::Limits,config:&crate::pet_config::Configuration,request_id:String,decision_id:String,now:u64)->Result<Self>{Self::prepare_planning_catalog(goal,evidence,budget,limits,config,super::planning::Catalog::SpotifyTransportV1,request_id,decision_id,now)}
    pub(super)fn prepare_planning_catalog(goal:&Record,evidence:Evidence,budget:Budget,limits:super::planning::Limits,config:&crate::pet_config::Configuration,catalog:super::planning::Catalog,request_id:String,decision_id:String,now:u64)->Result<Self>{
        // Every explicit catalogue uses ordinary native evidence, consent and
        // certification. Selecting iteration never promotes a one-round grant.
        Self::prepare_planning_catalog_native(goal,evidence,budget,limits,config,catalog,request_id,decision_id,now)
    }
    fn prepare_planning_catalog_native(goal:&Record,evidence:Evidence,budget:Budget,limits:super::planning::Limits,config:&crate::pet_config::Configuration,catalog:super::planning::Catalog,request_id:String,decision_id:String,now:u64)->Result<Self>{
        // A separately chosen catalogue requires the real selected native data
        // capture; JSON references/old turn consent are never sufficient.
        if catalog==super::planning::Catalog::PeerCorrespondenceMemoryV1&&(super::plans::typed(goal)||goal.execution_context.as_ref().is_none_or(|c|c.mode!=model::PreparationMode::Dedicated||c.memory.is_none()||c.received_request.is_some()||c.copied_messages!=0)){return Err(Denial::Unsupported);}
        let received_iteration=catalog==super::planning::Catalog::PeerDelegationIterationV1&&goal.spec.received_from.as_ref().is_some_and(|o|matches!(o.body.schema_version,1|2|3|4|5|6|7))&&goal.execution_context.as_ref().is_some_and(|c|c.mode==model::PreparationMode::Dedicated&&c.received_request.is_some()&&c.copied_messages==0);
        if goal.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some()||matches!(o.body.schema_version,3|5|6))&&!received_iteration{return Err(Denial::Unsupported);}
        let selected_iteration=catalog==super::planning::Catalog::PeerDelegationIterationV1&&goal.execution_context.as_ref().is_some_and(|c|c.mode==model::PreparationMode::Dedicated&&c.memory.as_ref().is_some_and(|m|!m.references.is_empty())&&c.received_request.is_none()&&c.copied_messages==0);
        if matches!(catalog,super::planning::Catalog::PeerCorrespondenceV1|super::planning::Catalog::PeerDelegationV1|super::planning::Catalog::PeerDelegationIterationV1)&&(super::plans::typed(goal)||goal.execution_context.as_ref().is_none_or(|c|c.has_selected_data()&&!selected_iteration&&!received_iteration||c.copied_messages!=0)){return Err(Denial::Unsupported);}
        if matches!(catalog,super::planning::Catalog::PeerDelegationV1|super::planning::Catalog::PeerDelegationIterationV1)&&goal.execution_context.as_ref().is_none_or(|c|c.mode!=model::PreparationMode::Dedicated){return Err(Denial::Unsupported);}
        if !goal.spec.owner.local_engine(){return Err(Denial::Unsupported);}
        if budget.payment!=Payment::LocalOnly{return Err(Denial::PaymentPolicy);}
        if catalog==super::planning::Catalog::SpotifyTransportV1&&!config.permissions(&goal.spec.owner.pet_id).map_err(|_|Denial::Invalid)?.music{return Err(Denial::Unsupported);}
        let policy=super::planning::Policy::native(limits,&budget).and_then(|p|p.with_catalog(catalog,&budget)).map_err(|_|Denial::Budget)?;
        if received_iteration{let review=Self::prepare_source_mode(goal,evidence,budget,request_id,decision_id,now,None,Some(policy))?;if !review.spec.binding_format(){return Err(Denial::Unsupported);}return Ok(review);}
        let mut review=Self::prepare(goal,evidence,budget,request_id,decision_id,now)?;
        review.spec.schema_version=if selected_iteration{24}else{3};review.spec.planning_policy=Some(policy);if !review.spec.binding_format(){return Err(Denial::Unsupported);}Ok(review)
    }
    #[cfg(target_os="linux")]
    pub(super)fn with_native_peer_plan(mut self,goal:&Record,goals:&std::path::Path,authority:&std::path::Path,observe:super::native_peer_plan::Observe,start:Option<super::planning::Start>,now:u64)->Result<Self>{
        let Some(start)=start else{return Ok(self);};if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}
        self.spec.approved_at=now;
        let captured=super::native_peer_plan::Captured::at(goals,authority,goal,&self.spec,observe,now).map_err(|e|match e{model::Error::Busy|model::Error::StorageUnavailable=>Denial::NotReady,model::Error::UnsupportedPlatform=>Denial::Unsupported,_=>Denial::RevisionChanged})?;
        self.spec.native_peer_plan=Some(captured.policy().clone());self.peer=Some(std::sync::Arc::new(captured));self.with_execution_start(Some(start))
    }
    #[cfg(target_os="linux")]fn peer_current(&self,goal:&Record,now:u64)->Result<()>{match(&self.spec.native_system_plan,&self.system){(None,None)=>{},(Some(policy),Some(system))if system.policy()==policy=>system.current(goal,&self.spec,now).map_err(|_|Denial::RevisionChanged)?,_=>return Err(Denial::Invalid)};match(&self.spec.native_peer_plan,&self.peer){(None,None)=>Ok(()),(Some(policy),Some(peer))if peer.policy()==policy=>peer.current(goal,&self.spec,now).map_err(|e|if matches!(e,model::Error::Busy|model::Error::StorageUnavailable){Denial::NotReady}else{Denial::RevisionChanged}),_=>Err(Denial::Invalid)}}
    pub(super)fn with_execution_start(mut self,start:Option<super::planning::Start>)->Result<Self>{
        if let Some(mode)=start{
            let scope=if self.spec.system(){StartScope::NativeSystemPlan}else if self.spec.native_peer_plan.is_some(){StartScope::NativePeerPlan}else if self.spec.result_disclosure.is_some(){StartScope::ResultDisclosure}else{StartScope::FiniteResponse};
            let choice=ExecutionStart{not_before_at:None,schema_version:1,mode,scope};
            if !choice.valid_for(&self.spec)||scope==StartScope::FiniteResponse&&!self.proposals.response_only(){return Err(Denial::Unsupported);}
            self.spec.execution_start=Some(choice);
            if !self.spec.binding_format(){return Err(Denial::Invalid);}
        }Ok(self)
    }
    pub(super)fn with_not_before(mut self,at:Option<u64>)->Result<Self>{
        if let Some(at)=at{
            let start=self.spec.execution_start.as_mut().ok_or(Denial::Unsupported)?;
            if start.scope!=StartScope::FiniteResponse{return Err(Denial::Unsupported);}
            start.schema_version=2;start.not_before_at=Some(at);
            if !self.spec.execution_start.as_ref().is_some_and(|s|s.valid_for(&self.spec))||!self.spec.binding_format(){return Err(Denial::Budget);}
        }Ok(self)
    }
    /// The requested execution mode is reviewed/signed with this exact policy;
    /// it cannot be supplied at confirmation or reconstructed from a model.
    pub(super)fn with_start(mut self,start:Option<super::planning::Start>)->Result<Self>{
        if self.spec.planning_policy.is_none(){return self.with_execution_start(start);}
        if let Some(start)=start{
            let policy=self.spec.planning_policy.as_mut().ok_or(Denial::Unsupported)?;
            policy.request_start(start);if !policy.valid_for(&self.spec.budget){return Err(Denial::Budget);}
        }Ok(self)
    }
    /// Native UI should render this exact value and retain the original Review.
    pub(super)fn proposal(&self)->ReviewProposal{ReviewProposal{native_system_plan:self.spec.native_system_plan.clone(),native_peer_plan:self.spec.native_peer_plan.clone(),execution_start:self.spec.execution_start.clone(),result_disclosure:self.spec.result_disclosure.clone(),#[cfg(target_os="linux")]result_data:self.evidence.result_preview(),funding:self.spec.funding.clone(),work_disclosure:self.spec.work_disclosure.clone(),native_verification:self.spec.native_verification.clone(),restored_from:self.restored_from.clone(),memory_data:self.evidence.memory_preview(),engine_binding:self.spec.engine_binding.clone(),continuation:self.spec.continuation.clone(),planning_policy:self.spec.planning_policy.clone(),proposals:self.proposals.clone(),operation:if self.spec.deterministic(){"goalDeterministic"}else if self.spec.planning_policy.is_some(){"goalPlanning"}else{"conversationRespond"},tool_execution_available:false,goal_id:self.spec.goal_id.clone(),revision:self.spec.source_revision.clone(),request_id:self.spec.request_id.clone(),owner:self.spec.owner.clone(),context:self.spec.context.clone(),intent:self.intent.clone(),criteria:self.criteria.clone(),resolved_model:self.spec.resolved_model.clone(),policy_revision:self.spec.policy_revision.clone(),descriptor_revision:self.spec.descriptor_revision.clone(),budget:self.spec.budget.clone(),prepared_at:self.spec.approved_at,expires_at:self.expires_at}}
    /// Validate current native evidence while retaining the original review.
    pub(super)fn confirmation_current(&self,goal:&Record,latest:&Evidence,now:u64)->Result<()>{
        if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}
        if !source_state(goal,self.spec.continuation.as_ref())||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||!super::plans::matches(goal,&self.spec){return Err(Denial::RevisionChanged);}
        #[cfg(target_os="linux")]if let Some(reference)=&self.spec.continuation{self.retained_authority.as_ref().ok_or(Denial::Invalid)?.continuation_current(goal,reference).map_err(continuation_failure)?;}
        #[cfg(target_os="linux")]self.peer_current(goal,now)?;
        self.evidence.memory_current(goal,now)?;latest.current(goal,now)?;
        if latest.system!=self.evidence.system||latest.deterministic!=self.evidence.deterministic||latest.policy_revision!=self.evidence.policy_revision{return Err(Denial::PolicyChanged);}
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
    pub(super)fn approve_with_data(self,goal:&Record,latest:Evidence,consent:Option<crate::pet_memory::context_model::GoalConsent>,now:u64)->Result<Approved>{self.approve_with_scopes(goal,latest,consent,None,now)}
    pub(super)fn disclosure_consent_current(&self,consent:Option<super::work_disclosure::Consent>,now:u64)->Result<()>{if now<self.spec.approved_at||now>=self.expires_at{return Err(Denial::Expired);}match(self.spec.work_disclosure.is_some(),self.spec.result_disclosure.is_some(),self.spec.native_peer_plan.is_some(),consent){(true,false,false,Some(super::work_disclosure::Consent::BoundedWorkDisclosure))|(false,true,false,Some(super::work_disclosure::Consent::BoundedResultDisclosure))|(false,false,true,Some(super::work_disclosure::Consent::BoundedNativePeerPlan))|(false,false,false,None)=>Ok(()),(true,false,false,None)|(false,true,false,None)|(false,false,true,None)=>Err(Denial::DataConsentRequired),_=>Err(Denial::Invalid)}}
    pub(super)fn approve_with_scopes(mut self,goal:&Record,latest:Evidence,consent:Option<crate::pet_memory::context_model::GoalConsent>,disclosure:Option<super::work_disclosure::Consent>,now:u64)->Result<Approved>{
        self.disclosure_consent_current(disclosure,now)?;
        self.data_consent_current(consent,now)?;self.confirmation_current(goal,&latest,now)?;
        #[cfg(target_os="linux")]let memory_confirmation=match self.evidence.memory.take(){
            Some(original)=>Some(original.confirm_against(goal,latest.memory.as_ref().ok_or(Denial::ContextChanged)?,consent,now).map_err(|_|Denial::ContextChanged)?),None=>None,
        };
        self.spec.approved_at=now;self.spec.evidence_hash=latest.fingerprint().map_err(|_|Denial::Invalid)?;
        Ok(Approved{#[cfg(target_os="linux")]system:self.system,#[cfg(target_os="linux")]peer:self.peer,#[cfg(target_os="linux")]memory_confirmation,#[cfg(target_os="linux")]retained_authority:self.retained_authority,spec:self.spec,expires_at:self.expires_at.min(latest.expires_at),evidence:latest})
    }
}
impl Approved{
    #[cfg(target_os="linux")]fn peer_current(&self,goal:&Record,now:u64)->Result<()>{match(&self.spec.native_system_plan,&self.system){(None,None)=>{},(Some(policy),Some(system))if system.policy()==policy=>system.current(goal,&self.spec,now).map_err(|_|Denial::RevisionChanged)?,_=>return Err(Denial::Invalid)};match(&self.spec.native_peer_plan,&self.peer){(None,None)=>Ok(()),(Some(policy),Some(peer))if peer.policy()==policy=>peer.current(goal,&self.spec,now).map_err(|e|if matches!(e,model::Error::Busy|model::Error::StorageUnavailable){Denial::NotReady}else{Denial::RevisionChanged}),_=>Err(Denial::Invalid)}}
    pub(super)fn spec(&self)->&AdmissionSpec{&self.spec}
    /// Revalidates freshness without changing the original consent bytes or TTL.
    pub(super)fn confirmation_current(&self,goal:&Record,latest:&Evidence,now:u64)->Result<()>{
        if now<self.spec.approved_at||now>=self.expires_at||now>=self.spec.budget.deadline_at{return Err(Denial::Expired);}
        self.current(goal,now).map_err(|_|Denial::RevisionChanged)?;
        if !source_state(goal,self.spec.continuation.as_ref()){return Err(Denial::NotDraft);}
        #[cfg(target_os="linux")]self.peer_current(goal,now)?;
        self.evidence.memory_current(goal,now)?;latest.current(goal,now)?;
        if latest.system!=self.evidence.system||latest.deterministic!=self.evidence.deterministic||latest.policy_revision!=self.evidence.policy_revision{return Err(Denial::PolicyChanged);}
        if !latest.same_memory(&self.evidence)||latest.engine_binding!=self.spec.engine_binding||latest.owner!=self.evidence.owner||latest.context!=self.evidence.context||latest.model!=self.evidence.model||latest.descriptor_semantics!=self.evidence.descriptor_semantics||latest.tokens!=self.evidence.tokens||latest.cost!=self.evidence.cost{return Err(Denial::RevisionChanged);}
        Ok(())
    }
    pub(super)fn current(&self,goal:&Record,now:u64)->model::Result<()>{
        goal.validate()?;#[cfg(target_os="linux")]self.peer_current(goal,now).map_err(|e|if e==Denial::NotReady{model::Error::Busy}else{model::Error::Conflict})?;if self.spec.result_disclosure.is_some(){self.evidence.memory_current(goal,now).map_err(|_|model::Error::Conflict)?;}
        #[cfg(target_os="linux")]match(&self.spec.memory_data,&self.memory_confirmation){
            (None,None)=>{},(Some(_),Some(data))=>data.current(goal,now).map_err(|_|model::Error::Conflict)?,_=>return Err(model::Error::Conflict),
        }
        if !source_state(goal,self.spec.continuation.as_ref())||now<self.spec.approved_at||now>=self.expires_at||now>=self.spec.budget.deadline_at||goal.revision!=self.spec.source_revision||goal.sequence!=self.spec.source_sequence||goal.spec.owner!=self.spec.owner||goal.execution_context.as_ref()!=Some(&self.spec.context)||model::fingerprint(&goal.spec)?!=self.spec.intent_hash||!super::plans::matches(goal,&self.spec){return Err(model::Error::Conflict);}Ok(())
    }
}

fn source_state(goal:&Record,continuation:Option<&super::continuation::Reference>)->bool{match continuation{None=>goal.state==State::Draft&&!goal.admissions.iter().any(|r|r.spec.deterministic()&&r.lease.is_some()),Some(c)=>c.source_matches(goal)}}
#[cfg(target_os="linux")]fn continuation_failure(reason:super::authority::Failure)->Denial{match reason{super::authority::Failure::Busy|super::authority::Failure::Unavailable|super::authority::Failure::Uncertain=>Denial::NotReady,_=>Denial::RevisionChanged}}
#[cfg(target_os="linux")]
impl Review{
    pub(super)fn has_native_authority(&self)->bool{self.retained_authority.is_some()}
    pub(super)fn prepare_continuation(goal:&Record,evidence:Evidence,budget:Budget,limits:Option<super::planning::Limits>,config:&crate::pet_config::Configuration,captured:super::continuation::Captured,request:String,decision:String,now:u64)->Result<Self>{
        let(reference,authority)=captured.into_parts();
        if evidence.deterministic||matches!(reference.schema_version,3|4|5|8){return Err(Denial::Unsupported);}
        if !reference.source_matches(goal){return Err(Denial::RecoveryNotClosed);}
        authority.continuation_current(goal,&reference).map_err(continuation_failure)?;
        if reference.schema_version==6&&(limits.is_some()||goal.admissions.last().is_none_or(|r|!r.spec.funded_response())){return Err(Denial::Unsupported);}
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

#[path="deterministic_admission.rs"]mod deterministic_admission;
#[cfg(target_os="linux")]#[path="system_admission.rs"]mod system_admission;

/// Native recurrence admits only the exact original allocation. The common
/// Authority authenticates the parent and this complete spec before certifying.
#[cfg(target_os="linux")]
impl Review{
 pub(in crate::pet_runtime)fn prepare_rule_occurrence(goal:&Record,evidence:Evidence,budget:Budget,reference:super::rules::OccurrenceReference,prior:Option<&super::rules::RuleAdmission>,now:u64)->Result<Approved>{
  let request=format!("ra-{}",reference.occurrence_id);
  let decision=prior.map(|p|p.decision_id.clone()).unwrap_or_else(||model::hash(format!("coucou-rule-child-decision-v1\0{}\0{}",reference.rule_decision_id,reference.occurrence_id).as_bytes())[..32].into());
  let mut review=Self::prepare(goal,evidence,budget,request,decision,now)?.with_execution_start(Some(super::planning::Start::WhenAvailable))?;
  review.spec.schema_version=29;review.spec.rule_occurrence=Some(reference);
  if let Some(prior)=prior{
   review.spec.source_revision=prior.source_revision.clone();review.spec.source_sequence=prior.source_sequence;review.spec.approved_at=prior.approved_at;review.spec.descriptor_revision=prior.descriptor_revision.clone();review.spec.evidence_hash=prior.evidence_hash.clone();
   if !prior.matches(&review.spec){return Err(Denial::RevisionChanged);}
  }
  if !review.spec.binding_format(){return Err(Denial::Invalid);}
  let approved=Approved{system:None,peer:None,memory_confirmation:None,retained_authority:None,spec:review.spec,expires_at:review.expires_at,evidence:review.evidence};approved.current(goal,now).map_err(|_|Denial::RevisionChanged)?;Ok(approved)
 }
}

#[cfg(target_os="linux")]
impl Evidence{
 pub(in crate::pet_runtime)fn rule_binding(&self,p:&super::rules::RulePolicy)->bool{
  !self.system&&!self.deterministic&&self.owner==p.rule.owner&&self.model==p.model&&self.engine_binding==p.engine_binding&&self.policy_revision==p.policy_revision&&!self.context.has_selected_data()&&self.context.copied_messages==0
 }
}
