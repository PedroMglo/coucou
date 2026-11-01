use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
pub const MAX_BYTES:usize=4*1024*1024;
pub const MAX_GOALS:usize=256;
pub const MAX_EVENTS:usize=4096;
const MAX_DATE:u64=8_640_000_000_000_000;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Error{InvalidInput,InvalidRecord,NotFound,Conflict,Busy,StorageUnavailable,UnsafePath,LimitReached,ClockRollback,#[allow(dead_code)]UnsupportedPlatform,EntropyUnavailable}
pub type Result<T>=std::result::Result<T,Error>;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum State{Draft,Paused,Cancelled,RecoveryRequired,Queued,Leased,Stopping,Completed}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum WorkState{Proposed,Cancelled,RecoveryRequired}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Owner{pub pet_id:String,pub provider:String,pub adapter:String,pub model_hint:Option<String>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Context{Unbound,Mochi{#[serde(rename="conversationId")]conversation_id:String}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Spec{pub request_id:String,pub owner:Owner,pub context:Context,pub intent:String,pub criteria:Vec<String>,#[serde(default,skip_serializing_if="Option::is_none")]pub restored_from:Option<super::restored_future::Origin>}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum PreparationMode{Dedicated,Snapshot}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ContextBinding{
    #[serde(default,skip_serializing_if="Option::is_none")]pub memory:Option<crate::pet_memory::context_model::Selection>,
    pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub provider:String,pub conversation_id:String,
    pub request_id:String,pub mode:PreparationMode,pub source_conversation_id:Option<String>,pub source_messages:usize,
    pub copied_messages:usize,pub seed_hash:String,pub prepared_at:u64,
}
pub(super)fn cid(goal:&Record)->String{format!("goal-{}",goal.id.trim_start_matches("g-"))}
impl ContextBinding{
    pub fn valid(&self,goal:&Record)->bool{
        if self.schema_version!=(if self.memory.is_some(){2}else{1})||self.memory.as_ref().is_some_and(|m|!m.valid(&self.pet_id,self.source_conversation_id.as_deref()))||self.goal_id!=goal.id||self.pet_id!=goal.spec.owner.pet_id||self.provider!=goal.spec.owner.provider||self.conversation_id!=cid(goal)||!key(&self.request_id)||self.prepared_at==0||self.prepared_at>8_640_000_000_000_000||self.seed_hash.len()!=64||!self.seed_hash.bytes().all(|b|b.is_ascii_hexdigit())||self.copied_messages>crate::assistant_memory::MAX_CONTEXT_MESSAGES||self.copied_messages>self.source_messages{return false;}
        match (&goal.spec.context,self.mode){
            (Context::Unbound,PreparationMode::Dedicated)=>self.source_conversation_id.is_none()&&self.source_messages==0&&self.copied_messages==0,
            (Context::Mochi{conversation_id},PreparationMode::Snapshot)=>self.source_conversation_id.as_ref()==Some(conversation_id),
            _=>false,
        }
    }
}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Task{pub id:String,pub description:String,pub state:WorkState,pub plan_id:Option<String>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Step{pub id:String,pub description:String,pub postcondition:String,pub state:WorkState,#[serde(default,skip_serializing_if="Option::is_none")]pub operation:Option<super::plans::OperationProposal>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Plan{pub id:String,pub task_id:String,pub steps:Vec<Step>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Legacy{pub task_id:String,pub provider:String,pub backend:String,pub conversation_id:Option<String>,pub phase:String,pub intent_may_be_incomplete:bool,pub receipts:Vec<crate::local_tasks::types::StepReceipt>}
#[derive(Clone,Copy,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum EventKind{Created,TaskProposed,PlanProposed,Paused,DraftResumed,Cancelled,CancelRequested,LegacyImported,ContextPrepared,AdmissionQueued,LeaseClaimed,StopRequested,LeaseDrained,ClaimAbandoned,GoalCompleted,PlanGenerated,LostOwnerClosed,UnclaimedOwnerClosed}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Event{pub sequence:u64,pub at:u64,pub request_id:String,pub fingerprint:String,pub kind:EventKind}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Record{
    pub schema_version:u32,pub id:String,pub revision:String,pub sequence:u64,pub created_at:u64,pub updated_at:u64,
    #[serde(default,skip_serializing_if="Option::is_none")]pub execution_context:Option<ContextBinding>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]pub admissions:Vec<super::admission::QueuedAdmission>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub completion:Option<super::criteria::CompletionRef>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]pub generated_plans:Vec<super::generated_plans::Reference>,
    pub spec:Spec,pub state:State,pub tasks:Vec<Task>,pub plans:Vec<Plan>,pub events:Vec<Event>,pub legacy:Vec<Legacy>,
}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct StepInput{pub description:String,pub postcondition:String,#[serde(default,skip_serializing_if="Option::is_none")]pub operation:Option<super::plans::OperationProposal>}
#[derive(Clone,Serialize,Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Action{
    ProposeTask{description:String},
    ProposePlan{#[serde(rename="taskId")]task_id:String,steps:Vec<StepInput>},
    Pause,ResumeDraft,Cancel,
    ImportLegacy{reference:Legacy},
    BindContext{binding:ContextBinding},
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum CommitStatus{Durable,Uncertain,Unchanged}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Receipt{pub status:CommitStatus,pub record:Record}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Summary{pub id:String,pub revision:String,pub pet_id:String,pub state:State,pub updated_at:u64,pub intent_preview:String,pub tasks:usize,pub steps:usize,#[serde(skip_serializing_if="is_false")]pub historical_source:bool}
fn is_false(v:&bool)->bool{!*v}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Listing{pub schema_version:u32,pub goals:Vec<Summary>,pub unavailable:usize,pub pending_artifacts:usize,pub complete:bool}

pub fn key(s:&str)->bool{!s.is_empty()&&s.len()<=128&&s.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_'))}
pub fn token(s:&str)->bool{s.len()==32&&s.bytes().all(|c|c.is_ascii_digit()||matches!(c,b'a'..=b'f'))}
pub fn goal_id(s:&str)->bool{s.strip_prefix("g-").is_some_and(|s|s.len()==64&&s.bytes().all(|c|c.is_ascii_digit()||matches!(c,b'a'..=b'f')))}
fn text(s:&str,max:usize)->bool{!s.trim().is_empty()&&s.len()<=max&&!s.chars().any(|c|c=='\0'||c=='\u{7f}'||c.is_control()&&!matches!(c,'\n'|'\r'|'\t'))}
fn date(n:u64)->bool{n>0&&n<=MAX_DATE}
pub fn hash(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
pub fn fingerprint<T:Serialize>(value:&T)->Result<String>{serde_json::to_vec(value).map(|v|hash(&v)).map_err(|_|Error::InvalidInput)}
pub fn id_for(spec:&Spec)->String{format!("g-{}",hash(format!("{}\0{}",spec.owner.pet_id,spec.request_id).as_bytes()))}
impl Owner{
    pub(super)fn local_engine(&self)->bool{self.provider==self.adapter&&matches!(self.adapter.as_str(),"ollama"|"lmstudio")}
    /// A model hint can resolve after a draft is created. Runtime identity is
    /// the logical factory owner/provider/adapter; admission seals the model.
    pub(super)fn same_runtime_binding(&self,other:&Self)->bool{self.pet_id==other.pet_id&&self.provider==other.provider&&self.adapter==other.adapter}

    /// A fresh native descriptor chooses the owner of new drafts. Stored identity
    /// compatibility alone does not make a retired/unimplemented route available.
    pub(super)fn from_descriptor(d:&crate::pet_capabilities::Descriptor)->Result<Self>{
        use crate::{pet_capabilities::{Kind,Lifecycle},pet_config::RuntimeKind};
        let b=crate::pet_config::blueprint(&d.pet_id).ok_or(Error::InvalidInput)?;
        let kind=match b.runtime.kind{RuntimeKind::Llm=>Kind::Llm,RuntimeKind::Service=>Kind::Service,RuntimeKind::Observer=>Kind::Observer};
        if !d.configuration_available||d.lifecycle!=Lifecycle::Active||d.kind!=kind||kind==Kind::Observer||b.retired||b.runtime.adapter.is_none(){return Err(Error::Conflict);}
        let owner=Self{pet_id:d.pet_id.clone(),provider:d.binding.provider.clone().ok_or(Error::Conflict)?,adapter:d.binding.adapter.ok_or(Error::Conflict)?.into(),model_hint:d.binding.model.clone()};
        if !owner.valid(){return Err(Error::Conflict);}Ok(owner)
    }

    fn valid(&self)->bool{
        crate::pet_config::blueprint(&self.pet_id).is_some_and(|b|b.recognizes_owner(&self.provider,&self.adapter))
            &&self.model_hint.as_ref().is_none_or(|m|text(m,256)&&!m.contains("://")&&!m.starts_with(['/', '\\', '~']))
    }
}
impl Spec{
    pub fn validate(&self)->Result<()>{
        if !key(&self.request_id)||!self.owner.valid()||!text(&self.intent,65536)||self.criteria.is_empty()||self.criteria.len()>32||self.criteria.iter().any(|c|!text(c,4096)){return Err(Error::InvalidInput);}
        if let Context::Mochi{conversation_id}= &self.context{if !crate::assistant_memory::valid_id(conversation_id){return Err(Error::InvalidInput);}}if self.restored_from.as_ref().is_some_and(|origin|!origin.valid_for(self)){return Err(Error::InvalidInput);}
        Ok(())
    }
}
impl Record{
    fn required_schema(&self)->u32{
        if self.spec.restored_from.is_some(){return 13;}
        if self.admissions.iter().any(|a|a.spec.memory_data.is_some()){return 12;}
        if self.execution_context.as_ref().is_some_and(|c|c.memory.is_some()){return 11;}
        if self.admissions.iter().any(|a|a.spec.continuation.is_some()){return 10;}
        if self.events.iter().any(|e|matches!(e.kind,EventKind::LostOwnerClosed|EventKind::UnclaimedOwnerClosed)){return 9;}
        if !self.generated_plans.is_empty()||self.admissions.iter().any(|a|a.spec.schema_version==3){return 8;}
        if self.completion.is_some(){return 7;}
        if super::plans::typed(self)||self.admissions.iter().any(|a|a.spec.schema_version>=2){6}
        else if self.admissions.iter().any(|a|a.lease.is_some()){5}
        else if self.admissions.iter().any(|a|a.proof.is_some()){4}
        else if !self.admissions.is_empty(){3}
        else if self.execution_context.is_some(){2}else{1}
    }
    pub(super)fn refresh_schema(&mut self){self.schema_version=self.required_schema();}

    pub fn new(spec:Spec,now:u64,revision:String)->Result<Self>{
        spec.validate()?;if !date(now)||!token(&revision){return Err(Error::InvalidInput);}
        let event=Event{sequence:1,at:now,request_id:spec.request_id.clone(),fingerprint:fingerprint(&spec)?,kind:EventKind::Created};
        let mut record=Self{schema_version:1,id:id_for(&spec),revision,sequence:1,created_at:now,updated_at:now,spec,state:State::Draft,execution_context:None,admissions:vec![],completion:None,generated_plans:vec![],tasks:vec![],plans:vec![],events:vec![event],legacy:vec![]};record.refresh_schema();record.validate()?;Ok(record)
    }
    pub fn validate(&self)->Result<()>{
        self.spec.validate().map_err(|_|Error::InvalidRecord)?;if self.spec.restored_from.as_ref().is_some_and(|origin|origin.body.approved_at>self.created_at){return Err(Error::InvalidRecord);}
        if self.schema_version!=self.required_schema()||!self.admissions.is_empty()&&self.execution_context.is_none()||self.execution_context.as_ref().is_some_and(|c|!c.valid(self))||self.id!=id_for(&self.spec)||!token(&self.revision)||!date(self.created_at)||!date(self.updated_at)||self.updated_at<self.created_at||self.tasks.len()>128||self.plans.len()>32||self.legacy.len()>128||self.events.is_empty()||self.events.len()>MAX_EVENTS||self.events.len() as u64!=self.sequence{return Err(Error::InvalidRecord);}
        let mut requests=BTreeSet::new();let mut at=self.created_at;
        for(i,e)in self.events.iter().enumerate(){if e.sequence!=i as u64+1||!date(e.at)||e.at<at||!key(&e.request_id)||!requests.insert(&e.request_id)||e.fingerprint.len()!=64||!e.fingerprint.bytes().all(|b|b.is_ascii_hexdigit()){return Err(Error::InvalidRecord);}at=e.at;}
        if at!=self.updated_at||self.events[0].at!=self.created_at||!matches!(self.events[0].kind,EventKind::Created)||self.events[0].request_id!=self.spec.request_id||self.events[0].fingerprint!=fingerprint(&self.spec)?{return Err(Error::InvalidRecord);}
        let expected=match self.events.last().ok_or(Error::InvalidRecord)?.kind{EventKind::Created|EventKind::TaskProposed|EventKind::PlanProposed|EventKind::DraftResumed|EventKind::ContextPrepared=>State::Draft,EventKind::AdmissionQueued=>State::Queued,EventKind::LeaseClaimed|EventKind::PlanGenerated=>State::Leased,EventKind::StopRequested=>State::Stopping,
            EventKind::LeaseDrained|EventKind::ClaimAbandoned|EventKind::LostOwnerClosed|EventKind::UnclaimedOwnerClosed=>match self.admissions.last().map(|a|a.phase){Some(super::admission::QueuePhase::Paused)=>State::Paused,Some(super::admission::QueuePhase::Cancelled)=>State::Cancelled,Some(super::admission::QueuePhase::RecoveryRequired)=>State::RecoveryRequired,_=>return Err(Error::InvalidRecord)},EventKind::GoalCompleted=>State::Completed,EventKind::Paused=>State::Paused,EventKind::Cancelled=>State::Cancelled,EventKind::CancelRequested|EventKind::LegacyImported=>State::RecoveryRequired};
        if self.state!=expected{return Err(Error::InvalidRecord);}
        if (self.state==State::Completed)!=self.completion.is_some(){return Err(Error::InvalidRecord);}
        if let Some(c)=&self.completion{
            let row=self.admissions.last().ok_or(Error::InvalidRecord)?;let event=self.events.last().ok_or(Error::InvalidRecord)?;
            if !c.valid()||c.goal_id!=self.id||c.pet_id!=self.spec.owner.pet_id||c.target_revision!=self.revision||c.observed_at>self.updated_at
                ||row.spec.decision_id!=c.decision_id||row.lease.as_ref().is_none_or(|l|l.id!=c.lease_id)||row.phase!=super::admission::QueuePhase::Completed
                ||event.request_id!=format!("complete-{}",c.recipe_id)||event.fingerprint!=fingerprint(c)?{return Err(Error::InvalidRecord);}
        }
        let mut generated=std::collections::BTreeSet::new();
        if self.generated_plans.len()>32*8||self.generated_plans.iter().any(|r|!r.structurally_present(self)||!generated.insert(&r.output.attempt_id))
            ||self.events.iter().filter(|e|matches!(e.kind,EventKind::PlanGenerated)).count()!=self.generated_plans.len(){return Err(Error::InvalidRecord);}
        if self.admissions.len()>32{return Err(Error::InvalidRecord);}
        let mut decisions=BTreeSet::new();let mut admission_requests=BTreeSet::new();
        for (i,a) in self.admissions.iter().enumerate(){
            let Some(e)=self.events.get(a.spec.source_sequence as usize)else{return Err(Error::InvalidRecord);};
            if a.lease.as_ref().is_some_and(|l|!l.valid(&a.spec)||a.proof.is_none())||a.stop_reason.is_some()&&a.lease.is_none()||a.proof.as_ref().is_some_and(|p|!p.valid()||!p.matches(&a.spec))||!a.spec.valid(self)||matches!(a.phase,super::admission::QueuePhase::Queued|super::admission::QueuePhase::Leased|super::admission::QueuePhase::StopRequested)&&!super::plans::matches(self,&a.spec)||!decisions.insert(&a.spec.decision_id)||!admission_requests.insert(&a.spec.request_id)
                ||!matches!(e.kind,EventKind::AdmissionQueued)||e.request_id!=a.spec.request_id||e.fingerprint!=fingerprint(&a.spec)?||e.at<a.spec.approved_at
                ||a.phase==super::admission::QueuePhase::Queued&&(i+1!=self.admissions.len()||self.state!=State::Queued){return Err(Error::InvalidRecord);}
        }
        if self.events.iter().filter(|e|matches!(e.kind,EventKind::AdmissionQueued)).count()!=self.admissions.len()
            ||self.state==State::Queued&&self.admissions.last().is_none_or(|a|a.phase!=super::admission::QueuePhase::Queued){return Err(Error::InvalidRecord);}
        let active=self.admissions.iter().filter(|a|matches!(a.phase,super::admission::QueuePhase::Queued|super::admission::QueuePhase::Leased|super::admission::QueuePhase::StopRequested)).count();
        if active>1||self.events.iter().filter(|e|matches!(e.kind,EventKind::LeaseClaimed|EventKind::ClaimAbandoned|EventKind::UnclaimedOwnerClosed)).count()!=self.admissions.iter().filter(|a|a.lease.is_some()).count(){return Err(Error::InvalidRecord);}
        for(i,a)in self.admissions.iter().enumerate(){use super::admission::QueuePhase as Q;
            if a.phase==Q::Completed&&(i+1!=self.admissions.len()||self.state!=State::Completed||a.lease.is_none()||a.proof.is_none()||a.stop_reason.is_some())
                ||matches!(a.phase,Q::Leased|Q::StopRequested)&&(i+1!=self.admissions.len()||a.lease.is_none())
                ||a.phase==Q::Leased&&(self.state!=State::Leased||a.stop_reason.is_some())
                ||a.phase==Q::StopRequested&&(self.state!=State::Stopping||a.stop_reason.is_none())
                ||a.phase==Q::RecoveryRequired&&a.lease.is_none(){return Err(Error::InvalidRecord);}
        }
        if matches!(self.state,State::Leased|State::Stopping)&&self.admissions.last().is_none_or(|a|a.lease.is_none()){return Err(Error::InvalidRecord);}
        let mut tasks=BTreeSet::new();let mut plans=BTreeSet::new();let mut steps=BTreeSet::new();
        for task in &self.tasks{if !key(&task.id)||!tasks.insert(&task.id)||!text(&task.description,16384){return Err(Error::InvalidRecord);}}
        for plan in &self.plans{if !key(&plan.id)||!plans.insert(&plan.id)||!tasks.contains(&plan.task_id)||plan.steps.is_empty()||plan.steps.len()>64{return Err(Error::InvalidRecord);}
            for step in &plan.steps{if !key(&step.id)||!steps.insert(&step.id)||!text(&step.description,4096)||!text(&step.postcondition,4096)||step.operation.as_ref().is_some_and(|op|!op.valid()){return Err(Error::InvalidRecord);}}
        }
        if steps.len()>512||self.tasks.iter().any(|t|t.plan_id.as_ref().is_some_and(|p|!self.plans.iter().any(|plan|&plan.id==p&&plan.task_id==t.id))){return Err(Error::InvalidRecord);}
        let mut legacy=BTreeSet::new();
        for old in &self.legacy{if !key(&old.task_id)||old.provider!=self.spec.owner.provider||!legacy.insert(&old.task_id)||!old.intent_may_be_incomplete||!text(&old.backend,32)||!text(&old.phase,32)||old.conversation_id.as_ref().is_some_and(|id|!crate::assistant_memory::valid_id(id))||old.receipts.len()>crate::local_tasks::types::MAX_CALLS||old.receipts.iter().any(|r|!r.valid()){return Err(Error::InvalidRecord);}}
        if !self.legacy.is_empty()&&self.state!=State::RecoveryRequired||self.legacy.is_empty()&&self.state==State::RecoveryRequired&&self.admissions.last().is_none_or(|a|a.phase!=super::admission::QueuePhase::RecoveryRequired){return Err(Error::InvalidRecord);}
        if self.state==State::Cancelled&&(self.tasks.iter().any(|t|t.state!=WorkState::Cancelled)||self.plans.iter().flat_map(|p|&p.steps).any(|s|s.state!=WorkState::Cancelled)){return Err(Error::InvalidRecord);}
        Ok(())
    }
    pub(super)fn enqueue(&mut self,approved:&super::admission::Approved,proof:Option<super::authority::ConsentProof>,now:u64,revision:String)->Result<()>{
        self.validate()?;approved.current(self,now)?;
        let a=approved.spec();if !self.legacy.is_empty()||match &a.continuation{None=>self.state!=State::Draft,Some(reference)=>proof.is_none()||!reference.source_matches(self)}{return Err(Error::Conflict);}
        if !date(now)||now<self.updated_at{return Err(Error::ClockRollback);}
        if !token(&revision)||revision==self.revision||self.events.iter().any(|e|e.request_id==a.request_id){return Err(Error::Conflict);}
        if self.events.len()>=MAX_EVENTS-1||self.admissions.len()>=32{return Err(Error::LimitReached);}
        self.sequence+=1;self.revision=revision;self.updated_at=now;self.schema_version=if self.admissions.iter().any(|a|a.lease.is_some()){5}else if proof.is_some()||self.admissions.iter().any(|a|a.proof.is_some()){4}else{3};self.state=State::Queued;
        self.events.push(Event{sequence:self.sequence,at:now,request_id:a.request_id.clone(),fingerprint:fingerprint(a)?,kind:EventKind::AdmissionQueued});
        self.admissions.push(super::admission::QueuedAdmission{spec:a.clone(),phase:super::admission::QueuePhase::Queued,proof,lease:None,stop_reason:None});self.refresh_schema();self.validate()
    }
    pub(super)fn claim(&mut self,lease:super::leases::Lease,now:u64,revision:String)->Result<()>{
        self.validate()?;let a=self.admissions.last().ok_or(Error::InvalidRecord)?;
        if self.state!=State::Queued||a.phase!=super::admission::QueuePhase::Queued||a.proof.is_none()||a.lease.is_some()||!lease.valid(&a.spec)||lease.source_revision!=self.revision||now<self.updated_at||now<lease.claimed_at||now>=lease.deadline_at||!token(&revision)||revision==self.revision||self.events.iter().any(|e|e.request_id==lease.request_id){return Err(Error::Conflict);}
        if self.events.len()>=MAX_EVENTS-3{return Err(Error::LimitReached);}
        let fingerprint=fingerprint(&lease)?;let request=lease.request_id.clone();let a=self.admissions.last_mut().unwrap();a.lease=Some(lease);a.phase=super::admission::QueuePhase::Leased;
        self.state=State::Leased;self.schema_version=5;self.sequence+=1;self.revision=revision;self.updated_at=now;self.events.push(Event{sequence:self.sequence,at:now,request_id:request,fingerprint,kind:EventKind::LeaseClaimed});self.refresh_schema();self.validate()
    }
    pub(super)fn abandoned(&mut self,lease:&super::leases::Lease,reason:Option<super::leases::StopReason>,now:u64,revision:String,request:String)->Result<()>{
        self.validate()?;let a=self.admissions.last_mut().ok_or(Error::InvalidRecord)?;
        // Pause/Cancel may advance the control revision before an unactivated
        // native preparation is audited. Only the original opaque proof can
        // enter here through Store; neither transition grants execution.
        let state_matches=match self.state{State::Queued=>lease.source_revision==self.revision,State::Paused=>reason==Some(super::leases::StopReason::Pause),State::Cancelled=>reason==Some(super::leases::StopReason::Cancel),_=>false};
        if !state_matches||a.lease.is_some()||!lease.valid(&a.spec)||now<self.updated_at||!date(now)||!token(&revision)||revision==self.revision||!key(&request)||self.events.iter().any(|e|e.request_id==request){return Err(Error::Conflict);}
        if self.events.len()>=MAX_EVENTS{return Err(Error::LimitReached);}
        a.lease=Some(lease.clone());a.stop_reason=reason;a.phase=match reason{Some(super::leases::StopReason::Cancel)=>super::admission::QueuePhase::Cancelled,Some(super::leases::StopReason::Pause)=>super::admission::QueuePhase::Paused,_=>super::admission::QueuePhase::RecoveryRequired};
        self.state=match a.phase{super::admission::QueuePhase::Cancelled=>State::Cancelled,super::admission::QueuePhase::Paused=>State::Paused,_=>State::RecoveryRequired};
        if self.state==State::Cancelled{for t in &mut self.tasks{t.state=WorkState::Cancelled;}for s in self.plans.iter_mut().flat_map(|p|&mut p.steps){s.state=WorkState::Cancelled;}}
        self.schema_version=5;self.sequence+=1;self.revision=revision;self.updated_at=now;self.events.push(Event{sequence:self.sequence,at:now,request_id:request,fingerprint:fingerprint(&(lease,reason))?,kind:EventKind::ClaimAbandoned});self.refresh_schema();self.validate()
    }
    pub(super)fn drained(&mut self,lease:&super::leases::Lease,confirmed:bool,now:u64,revision:String,request:String)->Result<()>{
        self.validate()?;let a=self.admissions.last_mut().ok_or(Error::InvalidRecord)?;
        if !matches!(self.state,State::Leased|State::Stopping)||a.lease.as_ref()!=Some(lease)||now<self.updated_at||!date(now)||!token(&revision)||revision==self.revision||!key(&request)||self.events.iter().any(|e|e.request_id==request){return Err(Error::Conflict);}
        if self.events.len()>=MAX_EVENTS{return Err(Error::LimitReached);}
        let reason=a.stop_reason;a.phase=if confirmed{match reason{Some(super::leases::StopReason::Cancel)=>super::admission::QueuePhase::Cancelled,Some(super::leases::StopReason::Pause)=>super::admission::QueuePhase::Paused,_=>super::admission::QueuePhase::RecoveryRequired}}else{super::admission::QueuePhase::RecoveryRequired};
        self.state=match a.phase{super::admission::QueuePhase::Cancelled=>State::Cancelled,super::admission::QueuePhase::Paused=>State::Paused,_=>State::RecoveryRequired};
        if self.state==State::Cancelled{for t in &mut self.tasks{t.state=WorkState::Cancelled;}for s in self.plans.iter_mut().flat_map(|p|&mut p.steps){s.state=WorkState::Cancelled;}}
        self.sequence+=1;self.revision=revision;self.updated_at=now;self.events.push(Event{sequence:self.sequence,at:now,request_id:request,fingerprint:fingerprint(&(lease,confirmed,reason))?,kind:EventKind::LeaseDrained});self.refresh_schema();self.validate()
    }
    pub fn summary(&self)->Summary{Summary{id:self.id.clone(),revision:self.revision.clone(),pet_id:self.spec.owner.pet_id.clone(),state:self.state,updated_at:self.updated_at,intent_preview:self.spec.intent.chars().take(120).collect(),tasks:self.tasks.len(),steps:self.plans.iter().map(|p|p.steps.len()).sum(),historical_source:false}}
    /// Only Store accepts an authenticated native closure transaction. This
    /// transition attaches audit metadata for an uncommitted claim, never work.
    pub(super)fn lost_owner_closed(&mut self,r:&super::recovery::Reference,lease:&super::leases::Lease)->Result<()>{
        self.validate()?;if !r.source_matches(self)||!super::recovery::lease_matches(r,lease)||r.approved_at<self.updated_at{return Err(Error::Conflict);}
        if r.target_state==State::Completed{return if self.state==State::Completed{Ok(())}else{Err(Error::Conflict)};}
        if !matches!(self.state,State::Queued|State::Leased|State::Stopping|State::Paused|State::RecoveryRequired|State::Cancelled)||self.events.len()>=MAX_EVENTS||self.events.iter().any(|e|e.request_id==r.request_id){return Err(Error::Conflict);}
        let kind=super::recovery::event(self,r);let row=self.admissions.last_mut().ok_or(Error::Conflict)?;
        if !lease.valid(&row.spec)||row.spec.decision_id!=r.decision_id||r.unclaimed!=row.lease.is_none()||row.lease.as_ref().is_some_and(|old|old!=lease)||row.lease.is_none()&&!(matches!(self.state,State::Queued|State::Paused|State::Cancelled)&&(self.state!=State::Queued||lease.source_revision==self.revision)){return Err(Error::Conflict);}
        row.lease=Some(lease.clone());row.phase=if r.target_state==State::Cancelled{super::admission::QueuePhase::Cancelled}else{super::admission::QueuePhase::RecoveryRequired};
        if r.target_state==State::Cancelled{row.stop_reason=Some(super::leases::StopReason::Cancel);for t in &mut self.tasks{t.state=WorkState::Cancelled;}for s in self.plans.iter_mut().flat_map(|p|&mut p.steps){s.state=WorkState::Cancelled;}}
        self.state=r.target_state;self.revision=r.target_goal_revision.clone();self.sequence=r.target_sequence;self.updated_at=r.approved_at;self.events.push(Event{sequence:self.sequence,at:self.updated_at,request_id:r.request_id.clone(),fingerprint:fingerprint(r)?,kind});self.refresh_schema();self.validate()
    }
    /// Only the native store accepts an authenticated certificate for this data.
    pub(super)fn complete_native(&mut self,c:super::criteria::CompletionRef,now:u64)->Result<()>{
        self.validate()?;
        if self.state!=State::Leased||self.revision!=c.source_revision||!c.valid()||c.goal_id!=self.id||c.pet_id!=self.spec.owner.pet_id||now<self.updated_at||now<c.observed_at||!date(now)||self.events.len()>=MAX_EVENTS{return Err(Error::Conflict);}
        let row=self.admissions.last_mut().ok_or(Error::Conflict)?;
        if row.spec.decision_id!=c.decision_id||row.phase!=super::admission::QueuePhase::Leased||row.lease.as_ref().is_none_or(|l|l.id!=c.lease_id){return Err(Error::Conflict);}
        let request=format!("complete-{}",c.recipe_id);if self.events.iter().any(|e|e.request_id==request){return Err(Error::Conflict);}
        row.phase=super::admission::QueuePhase::Completed;self.state=State::Completed;self.revision=c.target_revision.clone();self.sequence+=1;self.updated_at=now;
        self.events.push(Event{sequence:self.sequence,at:now,request_id:request,fingerprint:fingerprint(&c)?,kind:EventKind::GoalCompleted});self.completion=Some(c);self.refresh_schema();self.validate()
    }
    pub(super)fn source_before_completion(&self,sequence:u64,updated_at:u64)->Result<Self>{
        self.validate()?;let c=self.completion.as_ref().ok_or(Error::Conflict)?;
        if self.sequence!=sequence+1{return Err(Error::Conflict);}let mut old=self.clone();old.events.pop();old.sequence=sequence;old.updated_at=updated_at;old.revision=c.source_revision.clone();old.state=State::Leased;old.completion=None;
        old.admissions.last_mut().ok_or(Error::Conflict)?.phase=super::admission::QueuePhase::Leased;old.refresh_schema();old.validate()?;Ok(old)
    }
    pub fn replay(&self,request:&str,action:&Action)->Result<bool>{
        if !key(request){return Err(Error::InvalidInput);}let fingerprint=fingerprint(action)?;
        match self.events.iter().find(|e|e.request_id==request){Some(e)if e.fingerprint==fingerprint=>Ok(true),Some(_)=>Err(Error::Conflict),None=>Ok(false)}
    }
    pub fn apply(&mut self,request:String,action:Action,now:u64,revision:String)->Result<()>{
        self.validate()?;if self.replay(&request,&action)?{return Err(Error::Conflict);}
        if !date(now)||now<self.updated_at{return Err(Error::ClockRollback);}if !token(&revision)||revision==self.revision{return Err(Error::InvalidInput);}
        if self.events.len()>=MAX_EVENTS||self.events.len()>=MAX_EVENTS-1&&!matches!(action,Action::Cancel){return Err(Error::LimitReached);}
        if matches!(self.state,State::Cancelled|State::Completed){return Err(Error::Conflict);}
        let fingerprint=fingerprint(&action)?;let next=self.sequence.checked_add(1).ok_or(Error::LimitReached)?;
        let kind=match action{
            Action::ProposeTask{description}=>{if self.state!=State::Draft{return Err(Error::Conflict);}if self.tasks.len()>=128{return Err(Error::LimitReached);}if !text(&description,16384){return Err(Error::InvalidInput);}self.tasks.push(Task{id:format!("task-{next}"),description,state:WorkState::Proposed,plan_id:None});EventKind::TaskProposed},
            Action::ProposePlan{task_id,steps}=>{
                if self.state!=State::Draft{return Err(Error::Conflict);}if self.plans.len()>=32||self.plans.iter().map(|p|p.steps.len()).sum::<usize>()+steps.len()>512{return Err(Error::LimitReached);}
                if steps.is_empty()||steps.len()>64||steps.iter().any(|s|!text(&s.description,4096)||!text(&s.postcondition,4096)||s.operation.as_ref().is_some_and(|op|!op.valid())){return Err(Error::InvalidInput);}
                let task=self.tasks.iter_mut().find(|t|t.id==task_id&&t.state==WorkState::Proposed).ok_or(Error::NotFound)?;
                let id=format!("plan-{next}");task.plan_id=Some(id.clone());self.plans.push(Plan{id,task_id,steps:steps.into_iter().enumerate().map(|(i,s)|Step{id:format!("step-{next}-{i}"),description:s.description,postcondition:s.postcondition,state:WorkState::Proposed,operation:s.operation}).collect()});EventKind::PlanProposed
            },
            Action::Pause if matches!(self.state,State::Leased|State::Stopping)=>{
                if self.state==State::Stopping{return Err(Error::Conflict);}let a=self.admissions.last_mut().ok_or(Error::InvalidRecord)?;a.phase=super::admission::QueuePhase::StopRequested;a.stop_reason=Some(super::leases::StopReason::Pause);self.state=State::Stopping;EventKind::StopRequested
            },
            Action::Cancel if matches!(self.state,State::Leased|State::Stopping)=>{
                let a=self.admissions.last_mut().ok_or(Error::InvalidRecord)?;if a.stop_reason==Some(super::leases::StopReason::Cancel){return Err(Error::Conflict);}a.phase=super::admission::QueuePhase::StopRequested;a.stop_reason=Some(super::leases::StopReason::Cancel);self.state=State::Stopping;EventKind::StopRequested
            },
            Action::Pause=>{if !matches!(self.state,State::Draft|State::Queued){return Err(Error::Conflict);}if self.state==State::Queued{self.admissions.last_mut().ok_or(Error::InvalidRecord)?.phase=super::admission::QueuePhase::Paused;}self.state=State::Paused;EventKind::Paused},
            Action::ResumeDraft=>{if self.state!=State::Paused{return Err(Error::Conflict);}self.state=State::Draft;EventKind::DraftResumed},
            Action::Cancel=>{if self.legacy.is_empty(){if let Some(a)=self.admissions.last_mut(){if a.phase==super::admission::QueuePhase::Queued{a.phase=super::admission::QueuePhase::Cancelled;}}self.state=State::Cancelled;for t in &mut self.tasks{t.state=WorkState::Cancelled;}for s in self.plans.iter_mut().flat_map(|p|&mut p.steps){s.state=WorkState::Cancelled;}EventKind::Cancelled}else{self.state=State::RecoveryRequired;EventKind::CancelRequested}},
            Action::BindContext{binding}=>{
                if self.state!=State::Draft||self.execution_context.is_some()||!binding.valid(self){return Err(Error::Conflict);}
                self.execution_context=Some(binding);self.schema_version=2;EventKind::ContextPrepared
            },
            Action::ImportLegacy{reference}=>{
                if !matches!(self.state,State::Draft|State::Paused|State::RecoveryRequired){return Err(Error::Conflict);}if self.legacy.len()>=128{return Err(Error::LimitReached);}
                self.legacy.push(reference);self.state=State::RecoveryRequired;EventKind::LegacyImported
            },
        };
        self.sequence=next;self.revision=revision;self.updated_at=now;self.events.push(Event{sequence:next,at:now,request_id:request,fingerprint,kind});self.refresh_schema();self.validate()
    }
}
