//! Explicit native execution class, never an inference fallback or a model
//! sentinel. This first plan catalog reuses the exact local-message executor.
use super::{model::{Record,WorkState},messages::Input};
pub(super)fn plan(goal:&Record)->bool{
 !goal.plans.is_empty()&&goal.tasks.len()==goal.plans.len()&&goal.tasks.iter().any(|t|t.plan_id.is_some())
 &&goal.plans.iter().all(|p|!p.steps.is_empty()&&goal.tasks.iter().any(|t|t.id==p.task_id&&t.plan_id.as_deref()==Some(&p.id))
  &&p.steps.iter().all(|s|s.operation.as_ref().is_some_and(|o|Input::parse(o).is_ok_and(|a|a.to_pet()!=goal.spec.owner.pet_id&&match &a{Input::ResultReply(r)=>goal.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some()&&o.body.source.reference==r.source&&o.body.source.sender_pet_id==r.to_pet_id&&r.result.goal_id==goal.id&&r.result.pet_id==goal.spec.owner.pet_id&&r.expires_at<=o.body.source.deadline_at),Input::FundedReply(r)=>goal.spec.received_from.as_ref().is_some_and(|o|o.body.delegation.is_some()&&o.body.source.reference==r.source&&o.body.source.sender_pet_id==r.to_pet_id&&r.expires_at<=o.body.source.deadline_at),_=>true}))))
}
pub(super)fn proposed(goal:&Record)->bool{plan(goal)&&goal.tasks.iter().all(|t|t.state==WorkState::Proposed)&&goal.plans.iter().flat_map(|p|&p.steps).all(|s|s.state==WorkState::Proposed)}
