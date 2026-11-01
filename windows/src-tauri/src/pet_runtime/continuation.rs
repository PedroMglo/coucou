//! Signed carry of reviewed history, never an old controller or IO permit.
use super::{admission::{AdmissionSpec,Budget},history::Charge,model::{self,Record,State},recovery};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Anchor{pub decision_id:String,pub spec_hash:String,pub row_hash:String,pub grant_hash:String,pub budget:Budget,pub resolved_model:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Unused{pub decision_id:String,pub grant_hash:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
    #[serde(default,skip_serializing_if="Option::is_none")]pub uncertain_effects:Option<Vec<super::uncertain_effects::Effect>>,
    pub schema_version:u32,pub source_revision:String,pub source_sequence:u64,pub source_goal_hash:String,
    pub closure:recovery::Reference,pub history_hash:String,pub evidence_fence:String,pub previous:Vec<Anchor>,pub unused:Vec<Unused>,
    pub charge:Charge,pub history_continuity_verified:bool,pub additional_allowance:bool,pub reviewed_at:u64,
}
impl Reference{
    pub(super)fn basic(&self)->bool{
        let hash=|s:&str|super::authority::decode(s).is_some();
        let mut ids=std::collections::BTreeSet::new();
        (match(self.schema_version,&self.uncertain_effects){(1,None)=>self.charge.unknown_tool_results==0,(2,Some(effects))=>!effects.is_empty()&&effects.len()<=super::uncertain_effects::MAX_EFFECTS&&effects.len()as u64==self.charge.unknown_tool_results&&{let mut attempts=std::collections::BTreeSet::new();effects.iter().all(|e|e.valid()&&attempts.insert((&e.decision_id,&e.attempt_id))&&self.previous.iter().any(|a|a.decision_id==e.decision_id))},_=>false})&&model::token(&self.source_revision)&&self.source_sequence>0&&self.source_sequence<=model::MAX_EVENTS as u64
            &&hash(&self.source_goal_hash)&&hash(&self.history_hash)&&hash(&self.evidence_fence)&&self.closure.valid()&&self.closure.target_state==State::RecoveryRequired
            &&self.closure.target_goal_revision==self.source_revision&&self.closure.target_sequence==self.source_sequence
            &&(1..32).contains(&self.previous.len())&&self.unused.len()<=64&&!self.history_continuity_verified&&self.additional_allowance
            &&self.reviewed_at>=self.closure.approved_at&&self.reviewed_at<=8_640_000_000_000_000&&self.charge.valid()
            &&self.previous.iter().all(|a|model::token(&a.decision_id)&&ids.insert(&a.decision_id)&&[&a.spec_hash,&a.row_hash,&a.grant_hash].iter().all(|s|hash(s))&&a.budget.valid(self.reviewed_at.min(a.budget.deadline_at.saturating_sub(1)))&&!a.resolved_model.is_empty()&&a.resolved_model.len()<=256)
            &&self.unused.iter().all(|a|model::token(&a.decision_id)&&ids.insert(&a.decision_id)&&hash(&a.grant_hash))
            &&self.previous.last().is_some_and(|a|a.decision_id==self.closure.decision_id)
    }
    pub(super)fn source_matches(&self,goal:&Record)->bool{
        self.basic()&&goal.state==State::RecoveryRequired&&goal.id==self.closure.goal_id&&goal.spec.owner.pet_id==self.closure.pet_id
            &&goal.revision==self.source_revision&&goal.sequence==self.source_sequence&&model::fingerprint(goal).ok().as_ref()==Some(&self.source_goal_hash)
    }
    pub(super)fn present(&self,goal:&Record,spec:&AdmissionSpec)->bool{
        if !self.basic()||spec.source_revision!=self.source_revision||spec.source_sequence!=self.source_sequence||spec.approved_at<self.reviewed_at{return false;}
        let Some(index)=goal.admissions.iter().position(|r|r.spec.decision_id==spec.decision_id)else{return false;};
        index==self.previous.len()&&goal.admissions[..index].iter().zip(&self.previous).all(|(row,a)|row.spec.decision_id==a.decision_id&&model::fingerprint(&row.spec).ok().as_ref()==Some(&a.spec_hash)&&model::fingerprint(row).ok().as_ref()==Some(&a.row_hash)&&row.spec.budget==a.budget&&row.spec.resolved_model==a.resolved_model)
    }
}
#[cfg(target_os="linux")]
pub(super)struct Captured{reference:Reference,authority:super::authority::Authority}
#[cfg(target_os="linux")]
impl Captured{
    pub(super)fn native(reference:Reference,authority:super::authority::Authority)->Self{Self{reference,authority}}
    pub(super)fn into_parts(self)->(Reference,super::authority::Authority){(self.reference,self.authority)}
}
