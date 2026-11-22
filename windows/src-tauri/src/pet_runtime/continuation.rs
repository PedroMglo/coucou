//! Signed carry of reviewed history, never an old controller or IO permit.
use super::{admission::{AdmissionSpec,Budget},history::Charge,model::{self,Record,State},recovery};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Anchor{#[serde(default,skip_serializing_if="Option::is_none")]pub native_schema:Option<u32>,pub decision_id:String,pub spec_hash:String,pub row_hash:String,pub grant_hash:String,pub budget:Budget,pub resolved_model:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Unused{pub decision_id:String,pub grant_hash:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
    #[serde(default,skip_serializing_if="Option::is_none")]pub funded:Option<FundedClosure>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub plan_revision:Option<super::native_plan_revisions::Reference>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub drain:Option<super::native_drain::Reference>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native:Option<NativeCarry>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub uncertain_effects:Option<Vec<super::uncertain_effects::Effect>>,
    pub schema_version:u32,pub source_revision:String,pub source_sequence:u64,pub source_goal_hash:String,
    #[serde(default,skip_serializing_if="Option::is_none")]pub closure:Option<recovery::Reference>,pub history_hash:String,pub evidence_fence:String,pub previous:Vec<Anchor>,pub unused:Vec<Unused>,
    pub charge:Charge,pub history_continuity_verified:bool,pub additional_allowance:bool,pub reviewed_at:u64,
}
/// Original closed owner only; neither expiry nor a UI Pause is this fact.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FundedClosure{pub goal_id:String,pub pet_id:String,pub decision_id:String,pub closed_at:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeCarry{pub schema_version:u32,pub proposal_hash:String,pub receipts:Vec<super::native_verification::ReceiptRef>}
impl NativeCarry{
 pub(super)fn covers(&self,plan:&str,step:&str)->bool{self.receipts.iter().any(|r|r.plan_id==plan&&r.step_id==step)}
 fn valid(&self,previous:&[Anchor])->bool{
  let mut causes=std::collections::BTreeSet::new();self.schema_version==1&&super::authority::decode(&self.proposal_hash).is_some()&&self.receipts.len()<=512&&self.receipts.iter().all(|r|r.valid()&&r.source_decision_id.as_ref().is_some_and(|id|previous.iter().any(|a|&a.decision_id==id))&&causes.insert((&r.plan_id,&r.step_id)))
 }
 pub(super)fn matches(&self,goal:&Record)->bool{super::deterministic::plan(goal)&&super::plans::fingerprint(goal).ok().as_ref()==Some(&self.proposal_hash)&&self.receipts.iter().all(|r|super::plans::snapshot(goal).receipt_matches(r))}
 pub(super)fn matches_historical(&self,goal:&Record)->bool{super::native_plan_revisions::snapshot_at(goal,&self.proposal_hash).is_ok_and(|s|self.receipts.iter().all(|r|s.receipt_matches(r)))}
}
impl Anchor{
 fn valid_class(&self,at:u64)->bool{match self.native_schema{None=>self.budget.valid(at)&&!self.resolved_model.is_empty()&&self.resolved_model.len()<=256,Some(4|5)=>self.budget.valid_runtime(at)&&self.resolved_model.is_empty(),Some(6|7|8|16)=>self.budget.valid_runtime_continuation(at),Some(11)=>self.budget.valid_runtime(at)&&self.resolved_model.is_empty(),_=>false}}
 pub(super)fn matches_class(&self,spec:&AdmissionSpec)->bool{self.native_schema==spec.deterministic().then_some(spec.schema_version)}
}
impl Reference{
    fn source_identity(&self)->Option<(&str,&str,&str,State,u64)>{
        if let Some(f)=&self.funded{return (matches!(self.schema_version,6|9)&&self.closure.is_none()&&self.drain.is_none()&&self.plan_revision.is_none()&&model::goal_id(&f.goal_id)&&model::key(&f.pet_id)&&model::token(&f.decision_id)&&f.closed_at>0).then_some((f.goal_id.as_str(),f.pet_id.as_str(),f.decision_id.as_str(),State::Paused,f.closed_at));}
        match(&self.closure,&self.drain,&self.plan_revision){(Some(c),None,None)if matches!(self.schema_version,1|2|3|8)&&c.valid()&&c.target_state==State::RecoveryRequired&&c.target_goal_revision==self.source_revision&&c.target_sequence==self.source_sequence=>Some((&c.goal_id,&c.pet_id,&c.decision_id,c.target_state,c.approved_at)),(None,Some(d),None)if matches!(self.schema_version,4|8)&&d.continuable()&&d.target_revision==self.source_revision&&d.target_sequence==self.source_sequence=>Some((&d.goal_id,&d.pet_id,&d.decision_id,d.target_state,d.observed_at)),(None,None,Some(r))if matches!((self.schema_version,r.schema_version),(5,1)|(7,2)|(8,1)|(10,3))&&r.valid()&&r.target_revision==self.source_revision&&r.target_sequence==self.source_sequence=>Some((&r.goal_id,&r.pet_id,&r.decision_id,r.target_state,r.reviewed_at)),_=>None}
    }
    pub(super)fn basic(&self)->bool{
        let hash=|s:&str|super::authority::decode(s).is_some();
        let mut ids=std::collections::BTreeSet::new();
        (self.funded.is_some()==matches!(self.schema_version,6|9))&&(match(self.schema_version,&self.uncertain_effects){(6|7|9|10,None)=>self.native.is_none()&&self.charge.unknown_model_results==0&&self.charge.unknown_tool_results==0,(1,None)=>self.native.is_none()&&self.charge.unknown_tool_results==0,(3|4|5|8,None)=>self.charge.unknown_tool_results==0&&self.charge.unknown_model_results==0&&self.native.as_ref().is_some_and(|n|n.valid(&self.previous))&&self.previous.iter().all(|a|a.native_schema.is_some()),(2,Some(effects))=>self.native.is_none()&&!effects.is_empty()&&effects.len()<=super::uncertain_effects::MAX_EFFECTS&&effects.len()as u64==self.charge.unknown_tool_results&&{let mut attempts=std::collections::BTreeSet::new();effects.iter().all(|e|e.valid()&&attempts.insert((&e.decision_id,&e.attempt_id))&&self.previous.iter().any(|a|a.decision_id==e.decision_id))},_=>false})&&model::token(&self.source_revision)&&self.source_sequence>0&&self.source_sequence<=model::MAX_EVENTS as u64
            &&hash(&self.source_goal_hash)&&hash(&self.history_hash)&&hash(&self.evidence_fence)&&self.source_identity().is_some()
            &&(1..32).contains(&self.previous.len())&&self.unused.len()<=64&&!self.history_continuity_verified&&self.additional_allowance==(!matches!(self.schema_version,6|7|8|9|10))
            &&self.source_identity().is_some_and(|(_,_,_,_,at)|self.reviewed_at>=at)&&self.reviewed_at<=8_640_000_000_000_000&&self.charge.valid()
            &&self.previous.iter().all(|a|model::token(&a.decision_id)&&ids.insert(&a.decision_id)&&[&a.spec_hash,&a.row_hash,&a.grant_hash].iter().all(|s|hash(s))&&a.valid_class(self.reviewed_at.min(a.budget.deadline_at.saturating_sub(1)))&&(matches!(self.schema_version,3|4|5|8)||a.native_schema.is_none()))
            &&self.unused.iter().all(|a|model::token(&a.decision_id)&&ids.insert(&a.decision_id)&&hash(&a.grant_hash))
            &&self.source_identity().is_some_and(|(_,_,id,_,_)|self.previous.last().is_some_and(|a|a.decision_id==id))
    }
    pub(super)fn source_matches(&self,goal:&Record)->bool{
        self.basic()&&self.source_identity().is_some_and(|(id,pet,_,state,_)|goal.state==state&&goal.id==id&&goal.spec.owner.pet_id==pet)
            &&self.native.as_ref().is_none_or(|n|n.matches(goal))&&goal.revision==self.source_revision&&goal.sequence==self.source_sequence&&model::fingerprint(goal).ok().as_ref()==Some(&self.source_goal_hash)
    }
    pub(super)fn present(&self,goal:&Record,spec:&AdmissionSpec)->bool{
        if !self.basic()||self.native.as_ref().is_some_and(|n|!n.matches_historical(goal))||spec.source_revision!=self.source_revision||spec.source_sequence!=self.source_sequence||spec.approved_at<self.reviewed_at{return false;}
        let Some(index)=goal.admissions.iter().position(|r|r.spec.decision_id==spec.decision_id)else{return false;};
        index==self.previous.len()&&goal.admissions[..index].iter().zip(&self.previous).all(|(row,a)|row.spec.decision_id==a.decision_id&&model::fingerprint(&row.spec).ok().as_ref()==Some(&a.spec_hash)&&model::fingerprint(row).ok().as_ref()==Some(&a.row_hash)&&a.matches_class(&row.spec)&&row.spec.budget==a.budget&&row.spec.resolved_model==a.resolved_model)
    }
}
#[cfg(target_os="linux")]
pub(super)struct Captured{reference:Reference,authority:super::authority::Authority}
#[cfg(target_os="linux")]
impl Captured{
    pub(super)fn native(reference:Reference,authority:super::authority::Authority)->Self{Self{reference,authority}}
    pub(super)fn into_parts(self)->(Reference,super::authority::Authority){(self.reference,self.authority)}
}

/// Facts for the reviewed handoff of a closed peer planner. This DTO cannot
/// reconstruct its original controller, DATA approval, or an execution permit.
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct PeerPlanningSnapshot{
    pub goal_id:String,pub pet_id:String,pub goal_revision:String,pub goal_hash:String,
    pub decision_id:String,pub lease_id:String,pub closure_id:String,
    pub spec_hash:String,pub grant_hash:String,pub history_hash:String,pub receipt_fence:String,
    pub original_budget:Budget,pub original_policy:super::planning::Policy,
    pub model:String,pub last_reviewed_destination:Option<crate::pet_memory::context_model::Destination>,
    pub remaining:super::delegation::Allowance,pub delegated_held:super::delegation::Allowance,
    pub charge:Charge,pub journal_hash:String,pub rounds_observed:usize,
    pub outcome:PeerPlanningOutcome,pub receipts:Vec<super::native_verification::ReceiptRef>,
    pub pending_steps:Vec<super::messages::Cause>,
    pub observed_at:u64,pub execution_available:bool,pub model_use_authorized:bool,pub additional_allowance:bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub enum PeerPlanningOutcome{CanonicalPlan,UnpublishedPlan,UncertainModelResult,NoExecutablePlan}

#[cfg(target_os="linux")]
pub(super)struct CapturedPeerPlanning{
    pub(super)snapshot:PeerPlanningSnapshot,pub(super)authority:super::authority::Authority,
    pub(super)store:std::sync::Arc<super::store::Store>,pub(super)invalidated:std::sync::atomic::AtomicBool,
    pub(super)deadline:std::time::Instant,
}
#[cfg(target_os="linux")]
impl CapturedPeerPlanning{
    pub(super)fn snapshot(&self)->PeerPlanningSnapshot{self.snapshot.clone()}
    pub(super)fn current(&self,now:u64)->model::Result<()>{
        use std::sync::atomic::Ordering;
        if self.invalidated.load(Ordering::Acquire){return Err(model::Error::Conflict);}
        let result=(||{
            if now<self.snapshot.observed_at||std::time::Instant::now()>=self.deadline{return Err(model::Error::Conflict);}
            let goal=self.store.read_control(&self.snapshot.goal_id,&self.snapshot.pet_id)?;
            let mut current=match self.authority.peer_continuation_snapshot(&goal,now){
                Ok(snapshot)=>snapshot,
                Err(super::authority::Failure::Busy)=>return Err(model::Error::Busy),
                Err(super::authority::Failure::Unavailable|super::authority::Failure::Uncertain)=>return Err(model::Error::StorageUnavailable),
                Err(_)=>return Err(model::Error::Conflict),
            };
            current.observed_at=self.snapshot.observed_at;
            if current!=self.snapshot{return Err(model::Error::Conflict);}Ok(())
        })();
        // Unavailability is not proof of revocation. Actual changed/invalid
        // provenance is latched, so restoring old bytes cannot revive a choice.
        if matches!(result,Err(model::Error::Conflict|model::Error::InvalidRecord|model::Error::UnsafePath|model::Error::NotFound)){
            self.invalidated.store(true,Ordering::Release);
        }
        result
    }
}
