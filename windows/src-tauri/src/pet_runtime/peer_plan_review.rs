//! Exact original native peer plan through the existing human decision broker.
//! A serialized proposal, scope hash or model text cannot construct Batch/Approved.
use crate::pet_runtime::{model::{self,Record},authority::Authority,lease_controller::{Owned,Validity,message_controller::{Review,Approved}},recipe_decisions::Decision,store::nonce};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
pub(in crate::pet_runtime)type Observe=crate::pet_runtime::native_peer_plan::Observe;
pub(in crate::pet_runtime)struct Batch{lineage:Option<Arc<crate::pet_runtime::lease_controller::PeerRoundPublished>>,lease:crate::pet_runtime::leases::Lease,pub id:String,pub goal_hash:String,pub hash:String,pub prepared_at:u64,pub expires_at:u64,reviews:Vec<Arc<Review>>,proposal:serde_json::Value,consumed:AtomicBool}
impl Batch{
 pub(in crate::pet_runtime)fn capture(owner:&Owned,g:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->model::Result<Option<Arc<Self>>>{
  Self::capture_with_round(owner,g,config,settings,now,None)
 }
 pub(in crate::pet_runtime)fn capture_with_round(owner:&Owned,g:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64,lineage:Option<Arc<crate::pet_runtime::lease_controller::PeerRoundPublished>>)->model::Result<Option<Arc<Self>>>{
  if !crate::pet_runtime::native_peer_plan::family(&owner.live.spec)&&!owner.live.spec.peer_planning()||owner.live.spec.native_peer_plan.is_some(){return Err(model::Error::Conflict);}owner.message_goal(g,now)?;
  let a=Authority::existing(&owner.live.authority_path).map_err(crate::pet_runtime::authority::Failure::control_error)?;let mut reviews=vec![];let mut observed_receipts=vec![];
  let generated=if owner.live.spec.peer_planning(){let r=crate::pet_runtime::planned_peer::reference(g,&owner.live.spec)?;a.generated_candidate(&owner.live.spec,&owner.live.proof,owner.lease(),g,&r.plan_id).map_err(crate::pet_runtime::authority::Failure::control_error)?;Some(r.clone())}else{None};
  for p in &g.plans{if generated.as_ref().is_some_and(|r|r.plan_id!=p.id){continue;}for s in &p.steps{
   if crate::pet_runtime::native_peer_plan::covered(&owner.live.spec,&p.id,&s.id){continue;}if let Some(receipt)=a.message_at(&owner.live.spec,&owner.live.proof,&p.id,&s.id).map_err(crate::pet_runtime::authority::Failure::control_error)?{observed_receipts.push(serde_json::json!({"planId":p.id,"stepId":s.id,"messageId":receipt.envelope().message_id,"attemptId":receipt.attempt_id,"receivedAt":receipt.received_at,"receiptHash":model::fingerprint(&receipt)?}));continue;}
   let input=crate::pet_runtime::messages::Input::parse(s.operation.as_ref().ok_or(model::Error::Conflict)?).map_err(|_|model::Error::InvalidInput)?;
   let target=Arc::new(crate::pet_runtime::messages::Target::capture(config,settings,input.to_pet())?);
   let request=format!("native-batch-{}",nonce()?);let mut review=if let Some(r)=&generated{owner.review_generated_message_for_batch(g,&p.id,&s.id,request,target,now,r.clone())?}else{owner.review_message_for_batch(g,&p.id,&s.id,request,target,now)?};if let Some(scope)=&lineage{review.bind_round_lineage(scope.clone())?;if scope.reconciled(){a.message_recovery_ready(&owner.live.spec,&owner.live.proof,review.scope_for_recovery()).map_err(crate::pet_runtime::authority::Failure::control_error)?;}}else if review.proposal(g).envelope.planned_data.as_ref().is_some_and(|p|p.round_data.is_some()){return Err(model::Error::UnsupportedPlatform);}reviews.push(Arc::new(review));
  }}
  if reviews.is_empty(){return Ok(None);}if reviews.len()>crate::pet_runtime::messages::MAX_MESSAGES{return Err(model::Error::LimitReached);}
  // Refuse a known-overbooked generated manifest BEFORE native review or
  // its first delivery. Canonical delivery still checks the current balance
  // atomically; this passive calculation cannot issue a reservation.
  let accounting=if owner.live.spec.peer_delegation(){
   let v=a.delegation_view(g,&owner.live.spec.decision_id,now)?;let remaining=v.remaining.ok_or(model::Error::Conflict)?;let mut needed=crate::pet_runtime::delegation::Allowance::default();
   for r in &reviews{let e=&r.proposal(g).envelope;needed.add(&crate::pet_runtime::delegation::Allowance{model_calls:0,tool_calls:1,output_bytes:serde_json::to_vec(e).map_err(|_|model::Error::InvalidInput)?.len()as u64}).ok_or(model::Error::LimitReached)?;if let Some(d)=&e.delegation{needed.add(&d.allowance).ok_or(model::Error::LimitReached)?;}}
   if !needed.fits(&remaining){return Err(model::Error::LimitReached);}Some(serde_json::json!({"sharedTotalBudget":owner.live.spec.budget,"remainingBeforeDispatch":remaining,"pendingDispatchAndChildren":needed,"ownInferenceAlreadyDebited":true,"recipientAdmissionRequired":true}))
  }else{None};
  let proposals=reviews.iter().map(|r|r.proposal(g)).collect::<Vec<_>>();let expires_at=proposals.iter().map(|p|p.expires_at).min().ok_or(model::Error::Conflict)?.min(lineage.as_ref().map(|p|p.expires_at()).unwrap_or(u64::MAX));let id=nonce()?;
  let mut proposal=serde_json::json!({"generatedPlan":generated,"dataOrigin":{"context":owner.live.spec.context,"inferenceAdmissionHash":model::fingerprint(&owner.live.spec)?},"scope":{"id":id,"goalId":g.id,"petId":g.spec.owner.pet_id,"leaseId":owner.lease().id,"controllerId":owner.lease().controller_id,"deadlineAt":owner.lease().deadline_at,"dataConsent":"exactReviewedNativePeerPlan"},"goalRevision":g.revision,"intent":g.spec.intent,"criteria":g.spec.criteria,"steps":proposals,"preparedAt":now,"expiresAt":expires_at,"criteriaVerified":false,"goalComplete":false,"recipientExecutionAuthorized":false,"providerDataUseAuthorized":false});
  if let Some(scope)=&lineage{proposal["roundDataProvenance"]=serde_json::json!({"goalId":scope.goal().id,"nativePublishedScopeRequired":true,"recipientModelConsentTransfers":false,"reconciledFromCanonicalPlan":scope.reconciled(),"nativeModelDataApprovalRetained":scope.retains_native_data_approval(),"originalDataExpiresAt":scope.expires_at(),"canonicalReceiptsAlreadyObserved":observed_receipts});}
  if let Some(accounting)=accounting{proposal["delegationAccounting"]=accounting;}
  if serde_json::to_vec(&proposal).map_err(|_|model::Error::InvalidInput)?.len()>64*1024{return Err(model::Error::LimitReached);}
  Ok(Some(Arc::new(Self{lineage,lease:owner.lease().clone(),id,goal_hash:model::fingerprint(g)?,hash:model::fingerprint(&proposal)?,prepared_at:now,expires_at,reviews,proposal,consumed:AtomicBool::new(false)})))
 }
 pub(in crate::pet_runtime)fn proposal(&self)->&serde_json::Value{&self.proposal}
 pub(in crate::pet_runtime)fn metadata_validity(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  if let Some(lineage)=&self.lineage{match lineage.metadata_current(){Validity::Current=>{},other=>return other}}
  if owner.lease()!=&self.lease||self.consumed.load(Ordering::Acquire)||now<self.prepared_at||now>=self.expires_at||model::fingerprint(g).ok().as_ref()!=Some(&self.goal_hash){return Validity::Changed;}
  let(config,settings)=match observe(){Ok(p)=>p,Err(_)=>return Validity::Unknown};
  if self.reviews.iter().any(|r|!r.metadata_current(&config,&settings,now)){return Validity::Changed;}Validity::Current
 }
 pub(in crate::pet_runtime)fn current(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  match self.metadata_validity(owner,g,observe,now){Validity::Current=>owner.pending_validity(now),other=>other}
 }
 pub(in crate::pet_runtime)fn approve(&self,decision:&Decision,owner:&Owned,g:&Record,observe:&Observe,now:u64)->model::Result<Vec<Arc<Approved>>>{
  if !decision.matches_peer(self,g,owner,now){return Err(model::Error::Conflict);}
  match self.current(owner,g,observe,now){Validity::Current=>{},Validity::Busy|Validity::Unknown=>return Err(model::Error::Busy),Validity::Changed=>return Err(model::Error::Conflict)}
  let(config,settings)=observe()?;let mut out=vec![];
  for r in &self.reviews{out.push(Arc::new(r.approve_shared(owner,g,&config,&settings,now,Arc::new(||true))?));}
  if self.consumed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(model::Error::Conflict);}Ok(out)
 }
}

#[cfg(test)]#[path="peer_plan_review_tests.rs"]mod tests;
