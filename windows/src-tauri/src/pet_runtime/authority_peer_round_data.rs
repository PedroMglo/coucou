//! Canonical source/receipt facts for round DATA review. Never an IO grant.
use super::*;
use crate::pet_runtime::{peer_feedback::View,peer_round_data::{Subject,Fact,ModelReference},delegation_api::DelegationInput,reconciliation::{EvidenceKind,EvidenceStatus},results};
impl Authority{
 pub(in crate::pet_runtime)fn peer_round_facts(&self,parent:&Record,input:&DelegationInput,view:&View,children:&[Record],now:u64)->Result<Vec<Fact>>{
  let _guard=self.lock_shared()?;let row=parent.admissions.iter().find(|r|r.spec.decision_id==input.decision_id).ok_or(Failure::Missing)?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
  if input.pet_id!=parent.spec.owner.pet_id||input.goal_id!=parent.id||view.parent_goal_hash!=model::fingerprint(parent).map_err(|_|Failure::Invalid)?||view.parent_grant_hash!=model::fingerprint(&body).map_err(|_|Failure::Invalid)?||view.items.len()!=body.delegation_reservations.len()||now<body.updated_at{return Err(Failure::Changed);}
  let mut facts=vec![];let mut bytes=0u64;let mut count=0usize;
  for i in &view.items{
   let r=body.delegation_reservations.iter().find(|r|r.message_id==i.message_id&&r.scope_hash==i.scope_hash&&r.recipient_pet_id==i.recipient_pet_id&&r.binding()==i.binding.as_ref()).ok_or(Failure::Changed)?;
   let e=body.messages.as_ref().and_then(|m|m.entries.iter().find(|m|m.envelope().message_id==r.message_id)).ok_or(Failure::Missing)?.envelope();let d=e.delegation.as_ref().ok_or(Failure::Invalid)?;
   let subject=Subject{recipient_pet_id:r.recipient_pet_id.clone(),subgoal_hash:model::fingerprint(&d.subgoal).map_err(|_|Failure::Invalid)?,criteria_hash:model::fingerprint(&d.criteria).map_err(|_|Failure::Invalid)?};let mut references=vec![];
   if let Some(observed)=&i.child{
    let goal=children.iter().find(|g|g.id==observed.goal_id&&g.spec.owner.pet_id==observed.pet_id).ok_or(Failure::Missing)?;let row=goal.admissions.last().ok_or(Failure::Missing)?;
    if model::fingerprint(goal).map_err(|_|Failure::Invalid)?!=observed.goal_hash||r.binding().is_none_or(|b|!b.matches(&row.spec)){return Err(Failure::Changed);}let child=self.branch_child_unlocked(&body,r)?;
    if model::fingerprint(&child).map_err(|_|Failure::Invalid)?!=observed.grant_hash{return Err(Failure::Changed);}
    let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
    for observed_attempt in &observed.model_attempts{
     count+=1;if count>32{return Err(Failure::Capacity);}let attempt=child.budget_usage.as_ref().and_then(|l|l.attempts.iter().find(|a|a.id==observed_attempt.attempt_id&&a.kind==crate::pet_runtime::effect_budget::Kind::Model)).ok_or(Failure::Changed)?;
     let ids=vec![attempt.id.clone()];let before=results::receipt_fence(self,&row.spec,&ids)?;
     let mut receipt_hash=None;let mut content_hash=None;
     match results::lookup_unlocked(self,&row.spec,proof,&attempt.request_id){
      Ok(results::Lookup::Available{receipt})=>{
       bytes=bytes.checked_add(receipt.text.len()as u64).ok_or(Failure::Capacity)?;if bytes>16*1024*1024{return Err(Failure::Capacity);}let hash=model::fingerprint(&receipt).map_err(|_|Failure::Invalid)?;
       if !observed_attempt.outcome_known||!observed_attempt.result_evidence.iter().any(|e|e.kind==EvidenceKind::ModelResult&&e.status==EvidenceStatus::Verified&&e.hash.as_ref()==Some(&hash)){return Err(Failure::Changed);}content_hash=Some(model::hash(receipt.text.as_bytes()));receipt_hash=Some(hash);
      },
      _=>if observed_attempt.outcome_known{return Err(Failure::Changed);},
     }
     if before!=results::receipt_fence(self,&row.spec,&ids)?{return Err(Failure::Changed);}references.push(ModelReference{attempt_id:attempt.id.clone(),receipt_hash,content_hash,file_hash:before[0].1.clone()});
    }
   }
   facts.push(Fact{message_id:r.message_id.clone(),reservation_hash:model::fingerprint(r).map_err(|_|Failure::Invalid)?,subject,model_results:references});
  }self.fence()?;Ok(facts)
 }
}
