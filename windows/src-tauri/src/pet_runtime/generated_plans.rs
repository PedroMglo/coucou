//! Canonical generated-plan references are data. Only native signed journal
//! membership and publication acknowledgment can make them usable for review.
use super::{model::{self,Record},planning_journal::{OutputKind,OutputRef,Source}};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference {
    pub(super) schema_version:u32,pub(super) decision_id:String,pub(super) round_index:u8,
    pub(super) output:OutputRef,pub(super) source:Source,pub(super) task_id:String,pub(super) plan_id:String,
    pub(super) target_revision:String,pub(super) target_sequence:u64,pub(super) published_at:u64,pub(super) proposal_hash:String,
}
impl Reference {
    pub(super)fn valid(&self)->bool {
        self.schema_version==1&&model::token(&self.decision_id)&&(1..=8).contains(&self.round_index)&&self.output.valid()&&self.output.kind==OutputKind::Plan
            &&self.source.valid()&&self.task_id==format!("planning-task-{}",self.output.attempt_id)&&self.plan_id==format!("planning-plan-{}",self.output.attempt_id)
            &&model::token(&self.target_revision)&&self.target_revision!=self.source.revision&&self.target_sequence==self.source.sequence.saturating_add(1)&&self.target_sequence<=model::MAX_EVENTS as u64
            &&self.published_at>=self.output.recorded_at&&self.published_at>=self.source.updated_at&&self.published_at<=8_640_000_000_000_000
            &&super::authority::decode(&self.proposal_hash).is_some()
    }
    pub(super)fn structurally_present(&self,goal:&Record)->bool {
        self.valid()&&goal.generated_plans.iter().filter(|r|*r==self).count()==1
            &&goal.tasks.iter().any(|t|t.id==self.task_id&&t.plan_id.as_deref()==Some(self.plan_id.as_str()))
            &&goal.plans.iter().any(|p|p.id==self.plan_id&&p.task_id==self.task_id)
            &&goal.events.get(self.target_sequence as usize-1).is_some_and(|e|matches!(e.kind,model::EventKind::PlanGenerated)
                &&e.sequence==self.target_sequence&&e.at==self.published_at&&e.request_id==format!("publish-plan-{}",self.output.attempt_id)&&model::fingerprint(self).ok().as_ref()==Some(&e.fingerprint))
            &&goal.admissions.iter().any(|a|a.spec.schema_version==3&&a.spec.decision_id==self.decision_id
                &&a.lease.as_ref().is_some_and(|l|self.source.matches(&a.spec,l)))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(super)enum Phase {Prepared,Acknowledged,Abandoned}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Publication {pub phase:Phase,pub reference:Reference}

/// This structural seed match is deliberately separate from native signature
/// and authoritative-journal checks. It never permits execution by itself.
pub(super)fn seed_matches(goal:&Record,spec:&super::admission::AdmissionSpec)->bool {
    if spec.schema_version!=3||!spec.binding_format(){return false;}
    let refs=goal.generated_plans.iter().filter(|r|r.decision_id==spec.decision_id).collect::<Vec<_>>();
    if refs.iter().any(|r|!r.structurally_present(goal)){return false;}
    let mut seed=goal.clone();seed.tasks.retain(|t|!refs.iter().any(|r|r.task_id==t.id));seed.plans.retain(|p|!refs.iter().any(|r|r.plan_id==p.id));
    super::plans::fingerprint(&seed).ok().as_ref()==spec.proposal_hash.as_ref()
}
#[cfg(target_os="linux")]
fn add_data(goal:&mut Record,reference:&Reference,candidate:&super::planning_output::Candidate)->model::Result<()> {
    if goal.tasks.len()>=128||goal.plans.len()>=32||goal.tasks.iter().any(|t|t.id==reference.task_id)||goal.plans.iter().any(|p|p.id==reference.plan_id){return Err(model::Error::LimitReached);}
    goal.tasks.push(model::Task {id:reference.task_id.clone(),description:candidate.description.clone(),state:model::WorkState::Proposed,plan_id:Some(reference.plan_id.clone())});
    goal.plans.push(model::Plan {id:reference.plan_id.clone(),task_id:reference.task_id.clone(),steps:candidate.steps.iter().enumerate().map(|(i,s)|model::Step {
        id:format!("planning-step-{}-{i}",reference.output.attempt_id),description:s.description.clone(),postcondition:s.postcondition.clone(),state:model::WorkState::Proposed,operation:s.operation.clone(),
    }).collect()});Ok(())
}
#[cfg(target_os="linux")]
pub(super)fn prepare(goal:&Record,output:&super::planning_output::Envelope,now:u64)->model::Result<Reference> {
    let body=&output.body;let super::planning_output::Outcome::Plan{candidate}= &body.outcome else{return Err(model::Error::InvalidInput);};
    if !body.valid()||goal.state!=model::State::Leased||model::fingerprint(goal)?!=body.source.record_hash||now<goal.updated_at||now<body.recorded_at{return Err(model::Error::Conflict);}
    let mut reference=Reference {schema_version:1,decision_id:body.decision_id.clone(),round_index:body.round_index,output:body.reference().map_err(|_|model::Error::InvalidRecord)?,source:body.source.clone(),
        task_id:candidate.task_id.clone(),plan_id:candidate.plan_id.clone(),target_revision:candidate.target_revision.clone(),target_sequence:candidate.target_sequence,published_at:now,proposal_hash:String::new()};
    let mut next=goal.clone();add_data(&mut next,&reference,candidate)?;reference.proposal_hash=super::plans::fingerprint(&next)?;
    if !reference.valid(){return Err(model::Error::InvalidRecord);}Ok(reference)
}
#[cfg(target_os="linux")]
pub(super)fn output_matches(reference:&Reference,output:&super::planning_output::Envelope)->bool {
    let b=&output.body;let super::planning_output::Outcome::Plan{candidate:c}=&b.outcome else{return false;};
    reference.valid()&&reference.output==match b.reference(){Ok(r)=>r,Err(_)=>return false}
        &&reference.source==b.source&&reference.decision_id==b.decision_id&&reference.round_index==b.round_index
        &&reference.task_id==c.task_id&&reference.plan_id==c.plan_id&&reference.target_revision==c.target_revision&&reference.target_sequence==c.target_sequence
}
#[cfg(target_os="linux")]
pub(super)fn content_matches(goal:&Record,reference:&Reference,output:&super::planning_output::Envelope)->bool {
    if !reference.structurally_present(goal)||!output_matches(reference,output){return false;}
    let super::planning_output::Outcome::Plan{candidate:c}=&output.body.outcome else{return false;};
    let Some(task)=goal.tasks.iter().find(|t|t.id==reference.task_id)else{return false;};let Some(plan)=goal.plans.iter().find(|p|p.id==reference.plan_id)else{return false;};
    task.description==c.description&&plan.steps.len()==c.steps.len()&&plan.steps.iter().zip(&c.steps).enumerate().all(|(i,(s,expected))|
        s.id==format!("planning-step-{}-{i}",reference.output.attempt_id)&&s.description==expected.description&&s.postcondition==expected.postcondition&&s.operation==expected.operation)
}
#[cfg(target_os="linux")]
impl Record {
    pub(super)fn append_generated(&mut self,reference:Reference,output:&super::planning_output::Envelope)->model::Result<()> {
        self.validate()?;
        if !output_matches(&reference,output)||self.state!=model::State::Leased||self.revision!=reference.source.revision||self.sequence!=reference.source.sequence
            ||self.updated_at!=reference.source.updated_at||model::fingerprint(self)?!=reference.source.record_hash||self.events.len()>=model::MAX_EVENTS-1{return Err(model::Error::Conflict);}
        let super::planning_output::Outcome::Plan{candidate}=&output.body.outcome else{return Err(model::Error::InvalidInput);};
        add_data(self,&reference,candidate)?;if super::plans::fingerprint(self)?!=reference.proposal_hash{return Err(model::Error::Conflict);}
        self.sequence=reference.target_sequence;self.revision=reference.target_revision.clone();self.updated_at=reference.published_at;
        self.events.push(model::Event{sequence:self.sequence,at:self.updated_at,request_id:format!("publish-plan-{}",reference.output.attempt_id),fingerprint:model::fingerprint(&reference)?,kind:model::EventKind::PlanGenerated});
        self.generated_plans.push(reference);self.refresh_schema();self.validate()
    }
}
