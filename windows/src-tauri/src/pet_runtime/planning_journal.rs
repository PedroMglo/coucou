//! Authenticated round provenance in the original grant and effect budget.
//! A serialized round supplies neither a worker nor a controller.
use super::{admission::AdmissionSpec, effect_budget::{self, Attempt, Kind, Phase}, leases::Lease, model};
use serde::{Deserialize, Serialize};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(super) enum OutputKind { Plan,Blocked,Invalid }
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct OutputRef {pub schema_version:u32,pub attempt_id:String,pub body_hash:String,pub kind:OutputKind,pub recorded_at:u64}
impl OutputRef {
    pub(super)fn valid(&self)->bool {self.schema_version==1&&model::token(&self.attempt_id)&&super::authority::decode(&self.body_hash).is_some()&&self.recorded_at>0&&self.recorded_at<=8_640_000_000_000_000}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(super) struct Source {
    pub revision:String, pub sequence:u64, pub updated_at:u64, pub record_hash:String,
    pub policy_hash:String, pub main_spec_hash:String, pub lease_id:String, pub controller_id:String,
}
impl Source {
    pub(super) fn valid(&self)->bool {
        model::token(&self.revision) && self.sequence>0 && self.updated_at>0 && self.updated_at<=8_640_000_000_000_000
            && [&self.record_hash,&self.policy_hash,&self.main_spec_hash].iter().all(|s|super::authority::decode(s).is_some())
            && model::token(&self.lease_id) && model::token(&self.controller_id)
    }
    pub(super) fn matches(&self,spec:&AdmissionSpec,lease:&Lease)->bool {
        self.valid() && self.main_spec_hash==model::fingerprint(spec).unwrap_or_default()
            && self.policy_hash==model::fingerprint(&spec.planning_policy).unwrap_or_default()
            && self.lease_id==lease.id && self.controller_id==lease.controller_id
            && self.updated_at>=lease.claimed_at && self.sequence>spec.source_sequence
    }
}
/// Native-only captured request, deliberately not Deserialize or an IO permit.
pub(super) struct Request { pub request_id:String, pub source:Source,#[cfg(target_os="linux")]pub(super)peer_data:Option<std::sync::Arc<super::peer_round_review::Approved>> }
impl Request {
    pub(super) fn capture(goal:&model::Record,spec:&AdmissionSpec,lease:&Lease,request_id:&str)->model::Result<Self> {
        goal.validate()?;
        if !model::key(request_id) || !spec.planning() || !spec.binding_format() || !lease.valid(spec)
            || goal.state!=model::State::Leased || goal.id!=spec.goal_id || goal.spec.owner!=spec.owner
            || goal.execution_context.as_ref()!=Some(&spec.context) || model::fingerprint(&goal.spec)?!=spec.intent_hash
            || goal.admissions.last().is_none_or(|a|a.lease.as_ref()!=Some(lease)||model::fingerprint(&a.spec).ok()!=model::fingerprint(spec).ok())
            || !super::plans::matches(goal,spec) {return Err(model::Error::Conflict);}
        Ok(Self {#[cfg(target_os="linux")]peer_data:None,request_id:request_id.into(), source:Source {revision:goal.revision.clone(),sequence:goal.sequence,updated_at:goal.updated_at,
            record_hash:model::fingerprint(goal)?,policy_hash:model::fingerprint(&spec.planning_policy)?,main_spec_hash:model::fingerprint(spec)?,
            lease_id:lease.id.clone(),controller_id:lease.controller_id.clone()}})
    }
    pub(super)fn data_reference(&self)->Option<&super::peer_round_data::Consent>{
        #[cfg(target_os="linux")]{self.peer_data.as_ref().map(|p|p.reference())}
        #[cfg(not(target_os="linux"))]{None}
    }
    pub(super)fn data_dependency(&self)->Option<super::peer_round_sources::Dependency>{
        #[cfg(target_os="linux")]{self.peer_data.as_ref().and_then(|p|p.source_dependency())}
        #[cfg(not(target_os="linux"))]{None}
    }
    #[cfg(target_os="linux")]
    pub(super)fn bind_peer_data(&mut self,approved:std::sync::Arc<super::peer_round_review::Approved>)->model::Result<()>{
        if self.peer_data.is_some()||!approved.reference().for_source(&self.source,&approved.reference().input_hash){return Err(model::Error::Conflict);}self.peer_data=Some(approved);Ok(())
    }
    pub(super) fn matches(&self,spec:&AdmissionSpec,lease:&Lease)->bool {
        model::key(&self.request_id) && spec.planning() && spec.binding_format() && self.source.matches(spec,lease)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(super) struct Round {
    pub index:u8, pub source:Source, pub attempt_id:String, pub request_id:String,
    pub input_hash:String, pub phase:Phase,
    #[serde(default,skip_serializing_if="Option::is_none")]pub output:Option<OutputRef>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub publication:Option<super::generated_plans::Publication>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub feedback:Option<super::planning_feedback::Feedback>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub peer_feedback:Option<super::peer_round_feedback::Feedback>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub peer_sources:Option<super::peer_round_sources::Dependency>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub peer_data:Option<super::peer_round_data::Consent>,
}
impl Round {
    pub(super)fn continuable(&self)->bool {
        match self.output.as_ref().map(|o|o.kind) {
            Some(OutputKind::Invalid|OutputKind::Blocked)=>true,
            Some(OutputKind::Plan)=>self.feedback.as_ref().is_some_and(|f|f.continuable())||self.peer_feedback.as_ref().is_some_and(|f|f.valid()),
            None=>false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(super) struct Ledger { pub schema_version:u32, pub rounds:Vec<Round> }
impl Ledger {
    pub(super) fn empty()->Self {Self {schema_version:1,rounds:vec![]}}
    pub(super) fn valid(&self, budget:Option<&effect_budget::Ledger>)->bool {
        if self.schema_version!=(if self.rounds.iter().any(|r|r.peer_sources.as_ref().is_some_and(|d|d.history.is_some())){5}else if self.rounds.iter().any(|r|r.peer_sources.is_some()){4}else if self.rounds.iter().any(|r|r.peer_feedback.is_some()){3}else if self.rounds.iter().any(|r|r.peer_data.is_some()){2}else{1}) || self.rounds.len()>8 {return false;}
        let attempts=budget.map(|b|b.attempts.as_slice()).unwrap_or(&[]);
        self.rounds.iter().enumerate().all(|(i,r)|r.index as usize==i+1 && r.source.valid()
            &&r.publication.as_ref().is_none_or(|p|p.reference.valid()&&Some(&p.reference.output)==r.output.as_ref()&&p.reference.source==r.source&&p.reference.round_index==r.index)
            && r.output.as_ref().is_none_or(|v|v.valid()&&v.attempt_id==r.attempt_id&&r.phase==Phase::Settled)
            &&r.feedback.as_ref().is_none_or(|f|f.valid()&&r.publication.as_ref().is_some_and(|p|p.phase==super::generated_plans::Phase::Acknowledged&&p.reference.plan_id==f.plan_id))
            &&r.peer_feedback.as_ref().is_none_or(|f|f.valid()&&r.feedback.is_none()&&r.publication.as_ref().is_some_and(|p|p.phase==super::generated_plans::Phase::Acknowledged&&p.reference==f.plan))
            &&r.peer_sources.as_ref().is_none_or(|s|r.peer_data.as_ref().is_some_and(|d|s.for_consent(d)))
            &&r.peer_data.as_ref().is_none_or(|d|d.for_source(&r.source,&r.input_hash)&&self.rounds.iter().filter(|other|other.peer_data.as_ref().is_some_and(|x|x.id==d.id)).count()==1)
            && (i==0||self.rounds[i-1].continuable())
            && attempts.iter().filter(|a|a.id==r.attempt_id&&a.request_id==r.request_id&&a.input_hash==r.input_hash&&a.kind==Kind::Model&&a.phase==r.phase&&r.output.as_ref().is_none_or(|o|a.finished_at.is_some_and(|n|n<=o.recorded_at))).count()==1)
            && attempts.iter().filter(|a|a.kind==Kind::Model).count()==self.rounds.len()
            && attempts.iter().filter(|a|a.kind==Kind::Model).all(|a|self.rounds.iter().filter(|r|r.attempt_id==a.id).count()==1)
    }
    pub(super) fn valid_for(&self,spec:&AdmissionSpec,lease:Option<&Lease>,budget:Option<&effect_budget::Ledger>)->bool {
        let Some(policy)=spec.planning_policy.as_ref() else {return false;};
        self.rounds.iter().enumerate().all(|(i,r)|r.peer_feedback.as_ref().is_none_or(|f|spec.peer_iteration()&&f.goal_spec_hash==spec.intent_hash&&f.source.goal_id==spec.goal_id&&f.source.pet_id==spec.owner.pet_id&&f.source.decision_id==spec.decision_id)&&(!spec.peer_iteration()||i==0||r.peer_data.as_ref().is_some_and(|d|self.rounds[i-1].peer_feedback.as_ref().is_some_and(|f|f.matches_data(d)))))&&self.valid(budget) && policy.valid_for(&spec.budget) && self.rounds.len()<=policy.limits.max_rounds as usize && self.history_valid(spec)
            && self.rounds.iter().all(|r|lease.is_some_and(|l|r.source.matches(spec,l))
                && budget.is_some_and(|b|b.attempts.iter().find(|a|a.id==r.attempt_id).is_some_and(|a|a.reserved_at>=r.source.updated_at&&a.output_reserved<=u64::from(policy.limits.max_output_bytes)&&r.peer_data.as_ref().is_none_or(|d|lease.is_some_and(|l|d.for_spec(spec,l))&&a.reserved_at>=d.approved_at&&a.reserved_at<d.expires_at))))
    }
    /// Only strict improvement in native verified criterion count resets the
    /// consecutive no-progress counter. Proposal/time/ACK changes do not.
    pub(super) fn progress(&self)->(u8,usize) {
        let mut seen=std::collections::BTreeSet::new();let mut stalled=0u8;let mut best=0usize;
        for r in &self.rounds {
            if let Some(f)=&r.peer_feedback{let before=seen.len();seen.extend(f.information_keys.iter().cloned());stalled=if seen.len()>before{0}else{stalled.saturating_add(1)};continue;}
            match r.output.as_ref().map(|o|o.kind) {
                Some(OutputKind::Invalid|OutputKind::Blocked)=>stalled=stalled.saturating_add(1),
                Some(OutputKind::Plan)=>if let Some(f)=&r.feedback {
                    if f.known&&f.met()>best{best=f.met();stalled=0;}else{stalled=stalled.saturating_add(1);}
                },
                None=>{},
            }
        }(stalled,best)
    }
    fn history_valid(&self,spec:&AdmissionSpec)->bool {
        let Some(policy)=&spec.planning_policy else{return false;};let mut seen=std::collections::BTreeSet::new();let mut stalled=0u8;let mut best=0usize;let mut hashes:Option<Vec<&str>>=None;
        for (i,r) in self.rounds.iter().enumerate() {
            if i>0&&stalled>=policy.limits.no_progress_limit{return false;}
            if let Some(f)=&r.peer_feedback{if f.goal_spec_hash!=spec.intent_hash{return false;}let before=seen.len();seen.extend(f.information_keys.iter().cloned());stalled=if seen.len()>before{0}else{stalled.saturating_add(1)};}else if let Some(f)=&r.feedback {
                if f.goal_spec_hash!=spec.intent_hash{return false;}
                let current=f.criteria.iter().map(|c|c.criterion_hash.as_str()).collect::<Vec<_>>();
                if hashes.as_ref().is_some_and(|old|old!=&current){return false;}hashes=Some(current);
                if f.known&&f.met()>best{best=f.met();stalled=0;}else{stalled=stalled.saturating_add(1);}
            }else if r.output.as_ref().is_some_and(|o|matches!(o.kind,OutputKind::Invalid|OutputKind::Blocked)){stalled=stalled.saturating_add(1);}
        }true
    }
    pub(super) fn require_next(&self,policy:&super::planning::Policy)->model::Result<()> {
        if self.rounds.len()>=policy.limits.max_rounds as usize||self.progress().0>=policy.limits.no_progress_limit{return Err(model::Error::LimitReached);}
        if self.rounds.last().is_some_and(|r|!r.continuable()){return Err(model::Error::Conflict);}
        Ok(())
    }
    pub(super) fn append(&mut self,request:&Request,attempt:&Attempt)->model::Result<()> {
        // Only an authenticated invalid/blocked output permits a bounded retry.
        // A proposed plan still needs canonical ACK and native final feedback.
        if self.rounds.last().is_some_and(|r|!r.continuable()) {return Err(model::Error::Conflict);}
        let peer_data=request.data_reference().cloned();let peer_sources=request.data_dependency();
        if peer_sources.as_ref().is_some_and(|s|peer_data.as_ref().is_none_or(|d|!s.for_consent(d))){return Err(model::Error::Conflict);}
        if peer_data.as_ref().is_some_and(|d|!d.for_source(&request.source,&attempt.input_hash)||attempt.reserved_at<d.approved_at||attempt.reserved_at>=d.expires_at||self.rounds.iter().any(|r|r.peer_data.as_ref().is_some_and(|old|old.id==d.id))){return Err(model::Error::Conflict);}
        if peer_data.is_some()&&self.schema_version<2{self.schema_version=2;}
        if peer_sources.as_ref().is_some_and(|d|d.history.is_some()){self.schema_version=5;}else if peer_sources.is_some(){self.schema_version=4;}
        self.rounds.push(Round {peer_sources,peer_data,index:(self.rounds.len()+1) as u8,source:request.source.clone(),attempt_id:attempt.id.clone(),request_id:request.request_id.clone(),input_hash:attempt.input_hash.clone(),phase:attempt.phase,output:None,publication:None,feedback:None,peer_feedback:None});
        Ok(())
    }
    pub(super) fn update_attempt(&mut self,attempt:&Attempt)->model::Result<()> {
        if attempt.kind!=Kind::Model {return Ok(());}
        let r=self.rounds.iter_mut().find(|r|r.attempt_id==attempt.id&&r.request_id==attempt.request_id&&r.input_hash==attempt.input_hash).ok_or(model::Error::Conflict)?;
        r.phase=attempt.phase;Ok(())
    }
}
