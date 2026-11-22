//! Typed content for the common canonical plan-publication transaction. These
//! data values are neither consent nor execution authority. Only a native
//! certificate can pass the authority fences in Store::commit_plan.
use super::{authority,generated_plans,model::{self,Record},planning_output::Envelope};

pub(super)trait Change:Clone {
    type Reference;
    fn reference(&self)->&Self::Reference;
    fn goal_id(&self)->&str;
    fn present(&self,goal:&Record)->bool;
    fn apply(&self,goal:&mut Record)->model::Result<()>;
    fn content_matches(&self,goal:&Record)->bool;
    fn absent_in(&self,goal:&Record)->bool;
}
// No Deserialize/Serialize: a parsed source, hash, UI ACK or model output
// cannot instantiate the authority fence needed by the transaction.
pub(super)trait Certificate {
    type Change:Change;
    fn change(&self)->Self::Change;
    fn current(&self,goal:&Record,now:u64)->authority::Result<()>;
}

#[derive(Clone)]
pub(super)struct Generated {goal_id:String,reference:generated_plans::Reference,output:Option<Envelope>}
impl Generated {
    pub(super)fn new(reference:generated_plans::Reference,output:Envelope)->Self {Self{goal_id:output.body.goal_id.clone(),reference,output:Some(output)}}
    pub(super)fn absent(goal_id:&str,reference:generated_plans::Reference)->Self {Self{goal_id:goal_id.into(),reference,output:None}}
}
impl Change for Generated {
    type Reference=generated_plans::Reference;
    fn reference(&self)->&Self::Reference {&self.reference}
    fn goal_id(&self)->&str {&self.goal_id}
    fn present(&self,goal:&Record)->bool {goal.generated_plans.contains(&self.reference)}
    fn apply(&self,goal:&mut Record)->model::Result<()> {goal.append_generated(self.reference.clone(),self.output.as_ref().ok_or(model::Error::InvalidInput)?)}
    fn content_matches(&self,goal:&Record)->bool {self.output.as_ref().is_some_and(|output|generated_plans::content_matches(goal,&self.reference,output))}
    fn absent_in(&self,goal:&Record)->bool {!matches!(goal.state,model::State::Draft|model::State::Queued|model::State::Leased|model::State::Completed)
        &&!goal.generated_plans.iter().any(|r|r.output.attempt_id==self.reference.output.attempt_id)
        &&!goal.tasks.iter().any(|t|t.id==self.reference.task_id)&&!goal.plans.iter().any(|p|p.id==self.reference.plan_id)}
}

#[derive(Clone)]pub(super)struct Native {seal:super::native_plan_revisions::Seal}
impl Native {pub(super)fn new(seal:super::native_plan_revisions::Seal)->Self{Self{seal}}}
impl Change for Native {
    type Reference=super::native_plan_revisions::Reference;
    fn reference(&self)->&Self::Reference{&self.seal.reference}
    fn goal_id(&self)->&str{&self.seal.reference.goal_id}
    fn present(&self,goal:&Record)->bool{goal.native_plan_revisions.contains(&self.seal.reference)}
    fn apply(&self,goal:&mut Record)->model::Result<()>{goal.append_native_plan_revision(self.seal.reference.clone())?;if model::fingerprint(goal)?!=self.seal.target_goal_hash{return Err(model::Error::Conflict);}Ok(())}
    fn content_matches(&self,goal:&Record)->bool{self.seal.content_matches(goal)}
    fn absent_in(&self,goal:&Record)->bool{self.seal.absent_in(goal)}
}
