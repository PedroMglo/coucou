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
pub(super) struct Request { pub request_id:String, pub source:Source }
impl Request {
    pub(super) fn capture(goal:&model::Record,spec:&AdmissionSpec,lease:&Lease,request_id:&str)->model::Result<Self> {
        goal.validate()?;
        if !model::key(request_id) || spec.schema_version!=3 || !spec.binding_format() || !lease.valid(spec)
            || goal.state!=model::State::Leased || goal.id!=spec.goal_id || goal.spec.owner!=spec.owner
            || goal.execution_context.as_ref()!=Some(&spec.context) || model::fingerprint(&goal.spec)?!=spec.intent_hash
            || goal.admissions.last().is_none_or(|a|a.lease.as_ref()!=Some(lease)||model::fingerprint(&a.spec).ok()!=model::fingerprint(spec).ok())
            || !super::plans::matches(goal,spec) {return Err(model::Error::Conflict);}
        Ok(Self {request_id:request_id.into(), source:Source {revision:goal.revision.clone(),sequence:goal.sequence,updated_at:goal.updated_at,
            record_hash:model::fingerprint(goal)?,policy_hash:model::fingerprint(&spec.planning_policy)?,main_spec_hash:model::fingerprint(spec)?,
            lease_id:lease.id.clone(),controller_id:lease.controller_id.clone()}})
    }
    pub(super) fn matches(&self,spec:&AdmissionSpec,lease:&Lease)->bool {
        model::key(&self.request_id) && spec.schema_version==3 && spec.binding_format() && self.source.matches(spec,lease)
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
}
impl Round {
    pub(super)fn continuable(&self)->bool {
        match self.output.as_ref().map(|o|o.kind) {
            Some(OutputKind::Invalid|OutputKind::Blocked)=>true,
            Some(OutputKind::Plan)=>self.feedback.as_ref().is_some_and(|f|f.continuable()),
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
        if self.schema_version!=1 || self.rounds.len()>8 {return false;}
        let attempts=budget.map(|b|b.attempts.as_slice()).unwrap_or(&[]);
        self.rounds.iter().enumerate().all(|(i,r)|r.index as usize==i+1 && r.source.valid()
            &&r.publication.as_ref().is_none_or(|p|p.reference.valid()&&Some(&p.reference.output)==r.output.as_ref()&&p.reference.source==r.source&&p.reference.round_index==r.index)
            && r.output.as_ref().is_none_or(|v|v.valid()&&v.attempt_id==r.attempt_id&&r.phase==Phase::Settled)
            &&r.feedback.as_ref().is_none_or(|f|f.valid()&&r.publication.as_ref().is_some_and(|p|p.phase==super::generated_plans::Phase::Acknowledged&&p.reference.plan_id==f.plan_id))
            && (i==0||self.rounds[i-1].continuable())
            && attempts.iter().filter(|a|a.id==r.attempt_id&&a.request_id==r.request_id&&a.input_hash==r.input_hash&&a.kind==Kind::Model&&a.phase==r.phase&&r.output.as_ref().is_none_or(|o|a.finished_at.is_some_and(|n|n<=o.recorded_at))).count()==1)
            && attempts.iter().filter(|a|a.kind==Kind::Model).count()==self.rounds.len()
            && attempts.iter().filter(|a|a.kind==Kind::Model).all(|a|self.rounds.iter().filter(|r|r.attempt_id==a.id).count()==1)
    }
    pub(super) fn valid_for(&self,spec:&AdmissionSpec,lease:Option<&Lease>,budget:Option<&effect_budget::Ledger>)->bool {
        let Some(policy)=spec.planning_policy.as_ref() else {return false;};
        self.valid(budget) && policy.valid_for(&spec.budget) && self.rounds.len()<=policy.limits.max_rounds as usize && self.history_valid(spec)
            && self.rounds.iter().all(|r|lease.is_some_and(|l|r.source.matches(spec,l))
                && budget.is_some_and(|b|b.attempts.iter().find(|a|a.id==r.attempt_id).is_some_and(|a|a.reserved_at>=r.source.updated_at&&a.output_reserved<=u64::from(policy.limits.max_output_bytes))))
    }
    /// Only strict improvement in native verified criterion count resets the
    /// consecutive no-progress counter. Proposal/time/ACK changes do not.
    pub(super) fn progress(&self)->(u8,usize) {
        let mut stalled=0u8;let mut best=0usize;
        for r in &self.rounds {
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
        let Some(policy)=&spec.planning_policy else{return false;};let mut stalled=0u8;let mut best=0usize;let mut hashes:Option<Vec<&str>>=None;
        for (i,r) in self.rounds.iter().enumerate() {
            if i>0&&stalled>=policy.limits.no_progress_limit{return false;}
            if let Some(f)=&r.feedback {
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
        self.rounds.push(Round {index:(self.rounds.len()+1) as u8,source:request.source.clone(),attempt_id:attempt.id.clone(),request_id:request.request_id.clone(),input_hash:attempt.input_hash.clone(),phase:attempt.phase,output:None,publication:None,feedback:None});
        Ok(())
    }
    pub(super) fn update_attempt(&mut self,attempt:&Attempt)->model::Result<()> {
        if attempt.kind!=Kind::Model {return Ok(());}
        let r=self.rounds.iter_mut().find(|r|r.attempt_id==attempt.id&&r.request_id==attempt.request_id&&r.input_hash==attempt.input_hash).ok_or(model::Error::Conflict)?;
        r.phase=attempt.phase;Ok(())
    }
}
