//! Original authority transaction and canonical result store only. Read-only
//! capture; old DATA consent never becomes a new native Approved.
use super::*;
use crate::pet_runtime::{peer_round_history::{Captured,Reference},results};
impl Authority{
 fn peer_history_collect_unlocked(&self,body:&Grant,spec:&AdmissionSpec,proof:&ConsentProof,count:usize,now:u64)->Result<Captured>{
  let rounds=&body.planning_usage.as_ref().ok_or(Failure::Missing)?.rounds;if count>8||count>rounds.len(){return Err(Failure::Invalid);}let mut refs=vec![];let mut messages=vec![];let mut bytes=0usize;
  for r in rounds.iter().take(count){
   let output=r.output.as_ref().ok_or(Failure::RecoveryRequired)?;let ids=[r.attempt_id.clone()];let before=results::receipt_fence(self,spec,&ids)?;let file_hash=before.first().and_then(|(_,h)|h.clone()).ok_or(Failure::RecoveryRequired)?;
   let receipt=match results::lookup_unlocked(self,spec,proof,&r.request_id)?{results::Lookup::Available{receipt}=>receipt,_=>return Err(Failure::RecoveryRequired)};
   if receipt.attempt_id!=r.attempt_id||receipt.input_hash!=r.input_hash||results::receipt_fence(self,spec,&ids)?!=before{return Err(Failure::Changed);}
   if let Some(data)=&r.peer_data{let plan=r.publication.as_ref().filter(|p|p.phase==crate::pet_runtime::generated_plans::Phase::Acknowledged).ok_or(Failure::RecoveryRequired)?;self.peer_round_sources_core_unlocked(body,data,&plan.reference,now)?;}
   bytes=bytes.checked_add(receipt.text.len()).ok_or(Failure::Capacity)?;if bytes>64*1024{return Err(Failure::Capacity);}
   refs.push(Reference{round:r.index,request_id:r.request_id.clone(),attempt_id:r.attempt_id.clone(),input_hash:r.input_hash.clone(),output_hash:output.body_hash.clone(),receipt_hash:model::fingerprint(&receipt).map_err(|_|Failure::Invalid)?,content_hash:model::hash(receipt.text.as_bytes()),file_hash,dependency_hash:r.peer_sources.as_ref().map(model::fingerprint).transpose().map_err(|_|Failure::Invalid)?});messages.push(serde_json::json!({"role":"assistant","content":receipt.text}));
  }Captured::verified(refs,messages).map_err(|_|Failure::Capacity)
 }
 pub(in crate::pet_runtime)fn peer_history_capture(&self,parent:&Record,decision:&str,limit:Option<usize>,now:u64)->Result<Option<Captured>>{
  let row=parent.admissions.iter().find(|r|r.spec.decision_id==decision).ok_or(Failure::Missing)?;if !row.spec.peer_iteration(){return Ok(None);}let _lock=self.lock_shared()?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;let body=self.load(&row.spec,proof)?;
  if now<body.updated_at||body.phase==GrantPhase::Revoked||body.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel){return Err(Failure::Revoked);}
  let rounds=&body.planning_usage.as_ref().ok_or(Failure::Missing)?.rounds;let count=limit.unwrap_or_else(||rounds.iter().take_while(|r|r.output.is_some()).count());self.peer_history_collect_unlocked(&body,&row.spec,proof,count,now).map(Some)
 }
 pub(in crate::pet_runtime::authority_store)fn peer_history_revalidate_unlocked(&self,body:&Grant,data:&crate::pet_runtime::peer_round_data::Consent,plan:&crate::pet_runtime::generated_plans::Reference,now:u64)->Result<()>{
  let round=body.planning_usage.as_ref().and_then(|l|l.rounds.iter().find(|r|r.publication.as_ref().is_some_and(|p|p.reference==*plan)&&r.peer_data.as_ref()==Some(data))).ok_or(Failure::Missing)?;let deps=round.peer_sources.as_ref().ok_or(Failure::Missing)?;let Some(expected)=&deps.history else{return Ok(());};if expected.len()+1!=usize::from(round.index){return Err(Failure::Changed);}
  // The signed admission is read from the exact own goal; its source bytes
  // alone cannot authenticate an effect or a history reference.
  let store=crate::pet_runtime::store::Store::existing(&self.peer_source_goals()?).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;let parent=store.source_snapshot(&body.goal_id,&data.source.pet_id).map_err(|_|Failure::Changed)?;let row=parent.admissions.iter().find(|r|r.spec.decision_id==body.decision_id&&model::fingerprint(&r.spec).ok().as_ref()==Some(&body.spec_hash)).ok_or(Failure::Changed)?;
  let found=self.peer_history_collect_unlocked(body,&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?,expected.len(),now)?;if found.references()!=expected{return Err(Failure::Changed);}Ok(())
 }
}
