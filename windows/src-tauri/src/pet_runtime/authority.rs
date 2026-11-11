//! Native consent provenance. A proof is data; verification and fresh policy
//! checks are still required before any execution permit can be constructed.
use super::{admission::AdmissionSpec,model};
use serde::{Deserialize,Serialize};
#[cfg(target_os="linux")]pub(super)use super::authority_store::{Authority,Certified,PlanCertified};
#[cfg(target_os="linux")]pub(super)use super::authority_store::CompletionCertified;
#[cfg(target_os="linux")]pub(super)use super::authority_store::LostOwnerSource;

fn is_false(v:&bool)->bool{!*v}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub enum Failure{Missing,Unsafe,Changed,Invalid,WrongKey,Revoked,Expired,NotDue,NotQueued,Capacity,Unavailable,Uncertain,Busy,RecoveryRequired}
impl Failure{
    pub(super)fn control_error(self)->model::Error{match self{
        Self::Busy=>model::Error::Busy,Self::Missing|Self::Unavailable|Self::Uncertain=>model::Error::StorageUnavailable,
        Self::Capacity=>model::Error::LimitReached,_=>model::Error::Conflict,
    }}
}
pub(super)type Result<T>=std::result::Result<T,Failure>;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Operation{ConversationRespond,GoalPlanning,GoalDeterministic}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ConsentProof{pub schema_version:u32,pub key_id:String,pub operation:Operation,pub tag:String}
impl ConsentProof{
    pub(super)fn valid(&self)->bool{matches!((self.schema_version,self.operation),(1,Operation::ConversationRespond)|(2,Operation::GoalPlanning)|(3,Operation::GoalDeterministic))&&model::token(&self.key_id)&&decode(&self.tag).is_some()}
    pub(super)fn matches(&self,spec:&AdmissionSpec)->bool{spec.binding_format()&&if spec.deterministic(){self.schema_version==3&&self.operation==Operation::GoalDeterministic}else if spec.planning(){self.schema_version==2&&self.operation==Operation::GoalPlanning}else{self.schema_version==1&&self.operation==Operation::ConversationRespond}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super)enum GrantPhase{Issued,Revoked,Claimed,Waiting,Draining,Closed}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Grant{
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_system_plan_hash:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub peer_memory_planning:Option<super::planned_peer_data::Snapshot>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]pub owner_resumptions:Vec<super::owner_resumption::Seal>,
    #[serde(default,skip_serializing_if="is_false")]pub peer_planning:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]pub result_disclosure:Option<super::result_disclosure_policy::Policy>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub funded_by:Option<super::funding::Funding>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub delegation_pool:Option<super::admission::Budget>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]pub delegation_reservations:Vec<super::delegation::Reservation>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub work_disclosure:Option<super::work_disclosure::Policy>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]pub native_plan_revisions:Vec<super::native_plan_revisions::Seal>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_drain:Option<super::native_drain::Seal>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_completion:Option<super::native_verification::Report>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub messages:Option<super::messages::Ledger>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub retirement:Option<super::retirement::Reference>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub recovery:Option<super::recovery::Closure>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub runtime_owner:Option<super::runtime_owner::Record>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub resource_usage:Option<super::resource_wait::Ledger>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub budget_usage:Option<super::effect_budget::Ledger>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub planning_usage:Option<super::planning_journal::Ledger>,
    pub schema_version:u32,pub decision_id:String,pub goal_id:String,pub operation:Operation,
    pub spec_hash:String,pub phase:GrantPhase,pub revision:String,pub approved_at:u64,pub updated_at:u64,
    #[serde(default,skip_serializing_if="Option::is_none")]pub lease:Option<super::leases::Lease>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub stop_reason:Option<super::leases::StopReason>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub closed_at:Option<u64>,
}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct GrantEnvelope{pub body:Grant,pub tag:String}
impl Grant{
    pub(super)fn message_schema(&self)->u32{if let Some(source)=&self.peer_memory_planning{return if self.funded_by.as_ref().is_some_and(|f|f.schema_version==9){47}else if source.schema_version==4{46}else{45};}if !self.owner_resumptions.is_empty(){return 44;}if self.peer_planning{return if self.planning_usage.as_ref().is_some_and(|p|p.schema_version==5){43}else if self.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().schema_version==12)){42}else if self.planning_usage.as_ref().is_some_and(|p|p.schema_version==4){41}else if self.planning_usage.as_ref().is_some_and(|p|p.schema_version==3){40}else if self.planning_usage.as_ref().is_some_and(|p|p.schema_version==2){39}else if self.delegation_pool.is_some(){38}else if self.messages.as_ref().is_some_and(|m|m.entries.iter().any(|r|r.envelope().schema_version==10)){37}else{36};}match self.messages.as_ref().map(|m|m.schema_version){Some(9)=>34,Some(8)=>30,Some(7)=>28,Some(6)=>25,Some(5)=>19,Some(4)=>18,Some(3)=>16,Some(2)=>15,_=>9}}
    pub(super)fn plan_schema(&self)->u32{if self.operation==Operation::GoalDeterministic{self.native_schema()}else{self.funded_schema()}}
    pub(super)fn funded_schema(&self)->u32{if self.funded_by.as_ref().is_some_and(|f|f.schema_version==7){if self.native_plan_revisions.is_empty(){31}else{33}}else if self.native_plan_revisions.iter().any(|s|s.schema_version==2){24}else if self.funded_by.as_ref().is_some_and(|f|f.schema_version==2){22}else{21}}
    pub(super)fn native_schema(&self)->u32{if self.native_system_plan_hash.is_some(){48}else if self.funded_by.as_ref().is_some_and(|f|f.schema_version==8){34}else if self.delegation_reservations.iter().any(|r|r.schema_version==8){35}else if self.delegation_reservations.iter().any(|r|r.schema_version==7){32}else if self.funded_by.as_ref().is_some_and(|f|f.schema_version==5){28}else if self.funded_by.as_ref().is_some_and(|f|f.schema_version==6){30}else if self.delegation_reservations.iter().any(|r|r.schema_version==6){29}else if self.delegation_reservations.iter().any(|r|r.schema_version==5){27}else if self.funded_by.as_ref().is_some_and(|f|f.schema_version==4)||self.funded_by.is_some()&&!self.native_plan_revisions.is_empty(){26}else if self.result_disclosure.is_some(){25}else if self.delegation_reservations.iter().any(|r|r.schema_version==4){23}else if self.delegation_reservations.iter().any(|r|matches!(r.state,super::delegation::State::CancelRequested{..}|super::delegation::State::Settled{..})){20}else if self.funded_by.is_some()||self.delegation_reservations.iter().any(|r|matches!(r.state,super::delegation::State::Bound{..})){19}else if self.delegation_pool.is_some(){18}else if self.work_disclosure.is_some(){17}else if self.message_schema()>=15{self.message_schema()}else if !self.native_plan_revisions.is_empty(){14}else if self.native_drain.is_some(){13}else if self.native_completion.as_ref().is_some_and(|r|r.schema_version==2){12}else if self.native_completion.is_some(){11}else{10}}
    pub(super)fn never_claimed_zero(&self)->bool{
        self.owner_resumptions.is_empty()&&self.native_plan_revisions.is_empty()&&self.native_drain.is_none()&&self.native_completion.is_none()&&self.messages.is_none()&&self.lease.is_none()&&self.runtime_owner.is_none()&&self.resource_usage.is_none()&&self.budget_usage.is_none()&&self.recovery.is_none()
            &&self.planning_usage.as_ref().is_none_or(|p|p.rounds.is_empty())
    }
    pub(super)fn valid(&self)->bool{
        if (self.schema_version==48)!=self.native_system_plan_hash.is_some()||self.native_system_plan_hash.as_ref().is_some_and(|h|decode(h).is_none()||self.operation!=Operation::GoalDeterministic||self.peer_memory_planning.is_some()||self.peer_planning||!self.owner_resumptions.is_empty()||self.result_disclosure.is_some()||self.funded_by.is_some()||self.delegation_pool.is_some()||!self.delegation_reservations.is_empty()||self.work_disclosure.is_some()||!self.native_plan_revisions.is_empty()||self.native_completion.is_some()||self.messages.is_some()||self.planning_usage.is_some()||self.budget_usage.as_ref().is_some_and(|l|l.attempts.iter().any(|a|a.kind!=super::effect_budget::Kind::Tool))){return false;}
        if self.peer_memory_planning.as_ref().is_some_and(|p|!self.peer_planning||!matches!(p.schema_version,3|4)||!p.valid()||p.goal_id!=self.goal_id||p.admission_hash!=self.spec_hash){return false;}
        if !self.owner_resumptions.is_empty(){
            let Some(lease)=&self.lease else{return false;};
            if !self.peer_planning||self.owner_resumptions.len()>super::owner_resumption::MAX_RESUMPTIONS
                ||self.owner_resumptions.iter().enumerate().any(|(i,r)|!r.valid(lease,&self.goal_id,&self.decision_id)||r.reference.approved_at>self.updated_at||i>0&&(self.owner_resumptions[i-1].owner!=r.previous_owner||self.owner_resumptions[i-1].reference.approved_at>=r.reference.approved_at))
                ||self.runtime_owner.as_ref()!=self.owner_resumptions.last().map(|r|&r.owner){return false;}
        }
        if self.funded_by.as_ref().is_some_and(|f|f.schema_version==9)!=(self.peer_planning&&self.schema_version==47)||self.schema_version==47&&self.peer_memory_planning.as_ref().is_none_or(|p|p.schema_version!=4||p.received_request.as_ref().is_none_or(|r|!matches!(r.schema_version,2|4|7)||self.funded_by.as_ref().is_none_or(|f|f.source!=r.source||f.own_origin_hash!=r.own_origin_hash))){return false;}
        if self.peer_planning&&(self.schema_version!=self.message_schema()||self.operation!=Operation::GoalPlanning||self.funded_by.as_ref().is_some_and(|f|f.schema_version!=9)||self.result_disclosure.is_some()||self.work_disclosure.is_some()||!self.native_plan_revisions.is_empty()||self.native_drain.is_some()){return false;}
        if !self.peer_planning&&matches!(self.schema_version,36|37|38|39|40|41|42|43|44|45|46|47){return false;}
        if self.funded_by.as_ref().is_some_and(|f|f.schema_version==7&&(self.operation!=Operation::ConversationRespond||self.native_plan_revisions.iter().any(|s|s.schema_version!=3)))||self.delegation_reservations.iter().any(|r|matches!(r.schema_version,7|8))&&self.funded_by.as_ref().is_none_or(|f|f.schema_version!=5){return false;}
        if !self.pool_valid()||self.funded_by.as_ref().is_some_and(|f|!matches!(self.operation,Operation::GoalDeterministic|Operation::ConversationRespond)&&!(f.schema_version==9&&self.operation==Operation::GoalPlanning)||self.delegation_pool.is_some()&&!matches!(f.schema_version,5|9)||!f.basic()||matches!(f.schema_version,4|5|6|8)&&self.operation!=Operation::GoalDeterministic||f.schema_version==5&&self.delegation_pool.is_none()||f.schema_version==6&&self.delegation_pool.is_some()){return false;}
        if self.funded_by.is_some()&&self.operation==Operation::ConversationRespond&&(self.schema_version!=self.funded_schema()||self.messages.is_some()||self.native_completion.is_some()||self.native_drain.is_some()||self.native_plan_revisions.iter().any(|s|s.schema_version!=(if self.funded_by.as_ref().is_some_and(|f|f.schema_version==7){3}else{2}))||self.planning_usage.is_some()||self.budget_usage.as_ref().is_some_and(|l|l.attempts.len()>1||l.attempts.iter().any(|a|a.kind!=super::effect_budget::Kind::Model)||!(super::delegation::Allowance{model_calls:l.attempts.len()as u16,tool_calls:0,output_bytes:l.attempts.iter().map(|a|a.charge()).sum()}).fits(&self.funded_by.as_ref().unwrap().allowance))){return false;}
        if self.result_disclosure.as_ref().is_some_and(|p|self.funded_by.as_ref().is_none_or(|f|f.schema_version!=(if p.schema_version==2{8}else{3}))){return false;}
        if self.funded_by.as_ref().is_some_and(|f|matches!(f.schema_version,3|8))!=self.result_disclosure.is_some()||self.result_disclosure.as_ref().is_some_and(|p|!p.valid()||self.operation!=Operation::GoalDeterministic||p.goal_id!=self.goal_id||self.work_disclosure.is_some()||self.delegation_pool.is_some()||!self.delegation_reservations.is_empty()||self.planning_usage.is_some()||self.budget_usage.as_ref().is_some_and(|l|l.attempts.len()>1||l.attempts.iter().any(|a|a.kind!=super::effect_budget::Kind::Tool))){return false;}
        if self.work_disclosure.as_ref().is_some_and(|p|!p.valid()||self.operation!=Operation::GoalDeterministic){return false;}
        if !model::token(&self.decision_id)||!model::goal_id(&self.goal_id)||decode(&self.spec_hash).is_none()||!model::token(&self.revision)||self.approved_at==0||self.updated_at<self.approved_at||self.updated_at>8_640_000_000_000_000{return false;}
        if self.native_plan_revisions.len()>32{return false;}let mut revisions=std::collections::BTreeSet::new();
        if self.native_plan_revisions.iter().any(|r|!r.valid_for(self)||!revisions.insert(&r.reference.id)){return false;}
        if self.native_drain.as_ref().is_some_and(|r|!r.valid_for(self)){return false;}
        if self.native_completion.as_ref().is_some_and(|r|!r.valid_for(self)){return false;}
        if self.budget_usage.as_ref().is_some_and(|u|!u.valid()){return false;}
        if self.planning_usage.as_ref().is_some_and(|u|!u.valid(self.budget_usage.as_ref()))||self.planning_usage.is_some()!=(self.operation==Operation::GoalPlanning){return false;}
        let planning=self.planning_usage.is_some();
        if self.messages.as_ref().is_some_and(|m|self.schema_version!=(if self.operation==Operation::GoalDeterministic{self.native_schema()}else{self.message_schema()})||self.runtime_owner.is_none()||!m.valid_for(self)){return false;}
        if self.recovery.as_ref().is_some_and(|r|!r.valid_for(self))||self.retirement.as_ref().is_some_and(|r|!r.valid_for(self)){return false;}
        match &self.lease{
            None=>self.runtime_owner.is_none()&&self.resource_usage.is_none()&&self.budget_usage.is_none()&&self.schema_version==(if self.peer_planning{self.message_schema()}else if self.operation==Operation::GoalDeterministic{self.native_schema()}else if self.funded_by.is_some(){self.funded_schema()}else if self.retirement.is_some(){8}else if planning{4}else{1})&&matches!(self.phase,GrantPhase::Issued|GrantPhase::Revoked)&&self.stop_reason.is_none()&&self.closed_at.is_none(),
            Some(l)=>self.schema_version==(if self.peer_planning{self.message_schema()}else if self.operation==Operation::GoalDeterministic{self.native_schema()}else if self.funded_by.is_some(){self.funded_schema()}else if self.messages.is_some(){self.message_schema()}else if self.recovery.is_some(){7}else if self.runtime_owner.is_some(){6}else if self.resource_usage.is_some(){5}else if planning{4}else if self.budget_usage.is_some(){3}else{2})&&l.basic()&&l.schema_version==(if self.operation==Operation::GoalDeterministic{2}else{1})&&l.goal_id==self.goal_id&&l.decision_id==self.decision_id
                &&self.runtime_owner.as_ref().is_none_or(|o|o.valid_for(l)&&o.captured_at<=self.updated_at)
                &&self.resource_usage.as_ref().is_none_or(|r|planning&&r.valid(l,self.updated_at))
                &&match self.phase{
                    GrantPhase::Claimed=>self.stop_reason.is_none()&&self.closed_at.is_none()&&!self.resource_usage.as_ref().is_some_and(|r|r.waiting()),
                    GrantPhase::Waiting=>self.stop_reason.is_none()&&self.closed_at.is_none()&&self.resource_usage.as_ref().is_some_and(|r|r.waiting()),
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
    if !proof.matches(spec){return Err(Failure::Invalid);}
    let mut bytes=if proof.operation==Operation::GoalDeterministic{b"coucou-native-deterministic-goal-consent-v1\0".to_vec()}else if proof.operation==Operation::GoalPlanning{b"coucou-native-goal-planning-consent-v1\0".to_vec()}else{b"coucou-native-goal-consent-v1\0".to_vec()};bytes.extend(serde_json::to_vec(&(proof.schema_version,&proof.key_id,proof.operation,spec)).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
pub(super)fn grant_message(body:&Grant)->Result<Vec<u8>>{
    let mut bytes=if body.native_system_plan_hash.is_some(){b"coucou-native-system-goal-grant-state-v1\0".to_vec()}else if body.operation==Operation::GoalDeterministic{b"coucou-native-deterministic-goal-grant-state-v1\0".to_vec()}else if body.planning_usage.is_some(){b"coucou-native-goal-planning-grant-state-v1\0".to_vec()}else{b"coucou-native-goal-grant-state-v1\0".to_vec()};bytes.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ProvenanceStatus{pub verified:bool,pub reason:Option<Failure>,pub execution_available:bool}
impl ProvenanceStatus{pub(super)fn from_result(r:Result<()>)->Self{Self{verified:r.is_ok(),reason:r.err(),execution_available:false}}}
