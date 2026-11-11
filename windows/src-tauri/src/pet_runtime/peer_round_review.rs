//! Exact native DATA review through the existing broker. Approval here covers
//! only this metadata/body/destination; the original journal, budget and writer
//! must independently admit a new round. DTOs cannot recreate Review/Approved.
use super::{model::{self,Record},leases::Lease,lease_controller::{Owned,Validity},peer_round_data::{Captured,Consent},recipe_decisions::Decision};
use crate::local_chat::{PreparedRuntimeRequest,Server};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
pub(in crate::pet_runtime)type Observe=Arc<dyn Fn()->model::Result<(crate::pet_config::Configuration,Server)>+Send+Sync>;
pub(in crate::pet_runtime)struct Review{
 pub id:String,pub goal_hash:String,pub hash:String,pub prepared_at:u64,pub expires_at:u64,deadline:std::time::Instant,
 turn_binding:(String,u64),turn_current:Arc<dyn Fn()->bool+Send+Sync>,lease:Lease,data:Arc<Captured>,body:serde_json::Value,proposal:serde_json::Value,consumed:AtomicBool,revoked:AtomicBool,
}
pub(in crate::pet_runtime)struct Approved{review:Arc<Review>,consent:Consent,owner:Arc<Owned>,goal:Record,observe:Observe,published:AtomicBool}
fn invalid()->model::Error{model::Error::Conflict}
impl Review{
 pub(in crate::pet_runtime)fn capture(owner:&Owned,g:&Record,data:Arc<Captured>,request:&PreparedRuntimeRequest,now:u64)->model::Result<Arc<Self>>{
  owner.message_goal(g,now)?;let row=g.admissions.last().ok_or(invalid())?;let lease=owner.lease();let source=data.reference();
  if !row.spec.peer_round_destination_matches(&source.destination)||source.goal_id!=g.id||source.pet_id!=g.spec.owner.pet_id||source.decision_id!=lease.decision_id||source.expires_at<=now||!data.current(request.server(),&lease.model,now){return Err(invalid());}
  let turn_binding=request.turn_binding().ok_or(invalid())?;if turn_binding.0!=lease.conversation_id{return Err(invalid());}let turn_current=request.turn_witness().ok_or(invalid())?;
  let body=request.body();let messages=body.get("messages").and_then(serde_json::Value::as_array).ok_or(invalid())?;
  if body["model"].as_str()!=Some(lease.model.as_str())||body.get("tools").is_some()||messages.len()<2||messages.get(messages.len()-2)!=Some(&data.model_message())||messages.iter().filter(|m|**m==data.model_message()).count()!=1{return Err(invalid());}
  let expires_at=source.expires_at.min(lease.deadline_at);let id=super::store::nonce()?;let goal_hash=model::fingerprint(g)?;
  let mut proposal=serde_json::json!({"schemaVersion":1,"scope":{"id":id,"goalId":g.id,"petId":g.spec.owner.pet_id,"dataConsent":if data.canonical_history().is_some(){"oneExactPeerPlanningInputWithCanonicalOwnHistory"}else{"oneExactPeerPlanningInputMetadataOnly"},"maxUses":1,"contextId":turn_binding.0,"turnId":turn_binding.1},"goalRevision":g.revision,"intent":g.spec.intent,"criteria":g.spec.criteria,"sources":data.review(),"destination":source.destination,"actualRequestBody":body,"preparedAt":now,"expiresAt":expires_at,"executionAuthorized":false,"recipientExecutionAuthorized":false,"newBudgetGranted":false,"policyAllowsAnotherRound":row.spec.peer_iteration(),"requiresOriginalJournalAndBudgetBeforeDispatch":true});
  if row.spec.peer_received_iteration(){proposal["originalGoalData"]=serde_json::json!({"receivedSelection":row.spec.context.received_request,"ownMemory":row.spec.context.memory,"originalConsent":row.spec.memory_data,"additionalDataAuthorityGranted":false});}
  if row.spec.peer_memory_iteration(){proposal["originalGoalMemory"]=serde_json::json!({"selection":row.spec.context.memory,"originalConsent":row.spec.memory_data,"additionalMemoryAuthorityGranted":false});}
  if serde_json::to_vec(&proposal).map_err(|_|model::Error::InvalidInput)?.len()>512*1024{return Err(model::Error::LimitReached);}
  Ok(Arc::new(Self{id,goal_hash,hash:model::fingerprint(&proposal)?,prepared_at:now,expires_at,deadline:std::time::Instant::now().checked_add(std::time::Duration::from_millis(expires_at-now)).ok_or(invalid())?,turn_binding,turn_current,lease:lease.clone(),data,body:body.clone(),proposal,consumed:AtomicBool::new(false),revoked:AtomicBool::new(false)}))
 }
 pub(in crate::pet_runtime)fn proposal(&self)->&serde_json::Value{&self.proposal}
 fn latched(&self,validity:Validity)->Validity{if validity==Validity::Changed{self.revoked.store(true,Ordering::Release);}validity}
 pub(in crate::pet_runtime)fn metadata_validity(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  if self.revoked.load(Ordering::Acquire){return Validity::Changed;}self.latched(self.observe_metadata(owner,g,observe,now))
 }
 fn observe_metadata(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  if !(self.turn_current)(){return Validity::Changed;}self.observe_sources(owner,g,observe,now)
 }
 fn observe_sources(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  if std::time::Instant::now()>=self.deadline||owner.lease()!=&self.lease||now<self.prepared_at||now>=self.expires_at||model::fingerprint(g).ok().as_ref()!=Some(&self.goal_hash){return Validity::Changed;}
  let(_,server)=match observe(){Ok(p)=>p,Err(model::Error::Busy)=>return Validity::Busy,Err(model::Error::StorageUnavailable)=>return Validity::Unknown,Err(_)=>return Validity::Changed};
  match self.data.current_result(&server,&self.lease.model,now){Ok(true)=>Validity::Current,Err(model::Error::Busy)=>Validity::Busy,Err(model::Error::StorageUnavailable)=>Validity::Unknown,_=>Validity::Changed}
 }
 pub(in crate::pet_runtime)fn current(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{
  match self.metadata_validity(owner,g,observe,now){Validity::Current=>self.latched(owner.pending_validity(now)),other=>other}
 }
 pub(in crate::pet_runtime)fn pending_validity(&self,owner:&Owned,g:&Record,observe:&Observe,now:u64)->Validity{if self.consumed.load(Ordering::Acquire){Validity::Changed}else{self.current(owner,g,observe,now)}}
 pub(in crate::pet_runtime)fn approve(self:&Arc<Self>,decision:Decision,owner:&Arc<Owned>,g:&Record,observe:&Observe,now:u64)->model::Result<Approved>{self.approve_retained(&decision,owner,g,observe,now)}
 /// A typed Busy retains the exact original native answer. Neither this borrow
 /// nor a retry creates a new body, turn, expiry or affirmative decision.
 pub(in crate::pet_runtime)fn approve_retained(self:&Arc<Self>,decision:&Decision,owner:&Arc<Owned>,g:&Record,observe:&Observe,now:u64)->model::Result<Approved>{
  if !decision.matches_peer_data(self,g,owner,now){return Err(invalid());}
  match self.current(owner,g,observe,now){Validity::Current=>{},Validity::Busy|Validity::Unknown=>return Err(model::Error::Busy),Validity::Changed=>return Err(invalid())}
  // Canonical attempt input hash is composed only AFTER the exact human answer.
  let input_hash=model::fingerprint(&(&self.lease.id,&self.lease.model,&self.body))?;
  let consent=Consent{schema_version:1,id:self.id.clone(),goal_hash:self.goal_hash.clone(),review_hash:self.hash.clone(),lease_id:self.lease.id.clone(),controller_id:self.lease.controller_id.clone(),context_id:self.turn_binding.0.clone(),turn_id:self.turn_binding.1,input_hash,source:self.data.reference().clone(),approved_at:decision.answered_at(),expires_at:self.expires_at};
  if self.consumed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(invalid());}Ok(Approved{review:self.clone(),consent,owner:owner.clone(),goal:g.clone(),observe:observe.clone(),published:AtomicBool::new(false)})
 }
}
impl Approved{
 pub(super)fn source_dependency(&self)->Option<super::peer_round_sources::Dependency>{self.goal.admissions.last().is_some_and(|r|r.spec.peer_iteration()).then(||self.review.data.dependency())}
 pub(super)fn reference(&self)->&Consent{&self.consent}
 pub(in crate::pet_runtime)fn observe_for_writer(&self)->Observe{self.observe.clone()}
 pub(in crate::pet_runtime)fn current(&self,owner:&Owned,g:&Record,_observe:&Observe,now:u64)->Validity{
  if !self.owner.same_live(owner)||model::fingerprint(g).ok().as_ref()!=Some(&self.review.goal_hash){return Validity::Changed;}self.bound_validity(now)
 }
 pub(in crate::pet_runtime)fn bound_validity(&self,now:u64)->Validity{if self.published.load(Ordering::Acquire){return Validity::Changed;}self.review.current(&self.owner,&self.goal,&self.observe,now)}
 pub(in crate::pet_runtime)fn matches_dispatch(&self,server:&Server,model:&str,body:&serde_json::Value,now:u64)->bool{
  self.bound_validity(now)==Validity::Current&&model==self.owner.lease().model&&self.review.body==*body&&self.review.data.current(server,model,now)
   &&model::fingerprint(&(&self.owner.lease().id,model,body)).ok().as_ref()==Some(&self.consent.input_hash)
 }
 pub(in crate::pet_runtime)fn for_attempt(&self,spec:&super::admission::AdmissionSpec,lease:&Lease,attempt:&super::effect_budget::Attempt,source:&super::planning_journal::Source,now:u64)->bool{
  self.bound_validity(now)==Validity::Current&&self.consent.for_spec(spec,lease)&&self.consent.for_source(source,&attempt.input_hash)&&attempt.reserved_at>=self.consent.approved_at&&attempt.reserved_at<self.consent.expires_at
 }
 /// Completion can outlive the logical turn, but it cannot authorize more IO.
 /// Require the canonical signed result for this exact reviewed input first.
 pub(in crate::pet_runtime)fn publication_validity(&self,authority:&super::authority::Authority,spec:&super::admission::AdmissionSpec,proof:&super::authority::ConsentProof,request:&str,now:u64)->Validity{
  let receipt=match super::results::lookup(authority,spec,proof,request){Ok(super::results::Lookup::Available{receipt})=>receipt,Ok(_)=>return Validity::Unknown,Err(_)=>return Validity::Changed};
  if !self.consent.for_spec(spec,self.owner.lease())||receipt.input_hash!=self.consent.input_hash||receipt.goal_id!=self.goal.id||receipt.lease_id!=self.consent.lease_id||receipt.decision_id!=self.consent.source.decision_id||receipt.completed_at<self.consent.approved_at{return Validity::Changed;}
  if authority.peer_data_result_bound(spec,proof,request,&self.consent).is_err(){return Validity::Changed;}
  if self.review.revoked.load(Ordering::Acquire){return Validity::Changed;}
  match self.review.latched(self.review.observe_sources(&self.owner,&self.goal,&self.observe,now)){Validity::Current=>self.review.latched(self.owner.policy_validity(now)),other=>other}
 }
 pub(in crate::pet_runtime)fn acknowledge_existing_plan(&self,authority:&super::authority::Authority,spec:&super::admission::AdmissionSpec,proof:&super::authority::ConsentProof,target:&Record,request:&str,now:u64)->model::Result<()>{
  super::peer_round_data::ParentAdvance::capture(authority,spec,proof,self.owner.lease(),&self.goal,target,&self.consent,request,true,now).map(|_|())
 }
 pub(in crate::pet_runtime)fn published_validity(&self,authority:&super::authority::Authority,spec:&super::admission::AdmissionSpec,proof:&super::authority::ConsentProof,target:&Record,request:&str,now:u64)->Validity{match self.published_metadata_validity(authority,spec,proof,target,request,now){Validity::Current=>self.review.latched(self.owner.pending_validity(now)),other=>other}}
 /// Source/configuration checks used DURING the original Prepared/Acquired
 /// resource transition. This is not an execution/held-resource authority check.
 pub(in crate::pet_runtime)fn published_metadata_validity(&self,authority:&super::authority::Authority,spec:&super::admission::AdmissionSpec,proof:&super::authority::ConsentProof,target:&Record,request:&str,now:u64)->Validity{
  if self.review.revoked.load(Ordering::Acquire)||std::time::Instant::now()>=self.review.deadline||now<self.review.prepared_at||now>=self.review.expires_at{return self.review.latched(Validity::Changed);}
  let advance=match super::peer_round_data::ParentAdvance::capture(authority,spec,proof,self.owner.lease(),&self.goal,target,&self.consent,request,false,now){Ok(p)=>p,Err(model::Error::Busy)=>return Validity::Busy,Err(model::Error::StorageUnavailable)=>return Validity::Unknown,Err(_)=>return Validity::Changed};
  let(_,server)=match(self.observe)(){Ok(p)=>p,Err(model::Error::Busy)=>return Validity::Busy,Err(model::Error::StorageUnavailable)=>return Validity::Unknown,Err(_)=>return self.review.latched(Validity::Changed)};
  match self.review.data.current_after_publication(&server,&self.owner.lease().model,now,&advance){Ok(true)=>Validity::Current,Err(model::Error::Busy)=>Validity::Busy,Err(model::Error::StorageUnavailable)=>Validity::Unknown,_=>self.review.latched(Validity::Changed)}
 }
 pub(in crate::pet_runtime)fn publication_not_revoked(&self)->bool{!self.review.revoked.load(Ordering::Acquire)&&std::time::Instant::now()<self.review.deadline&&crate::assistant_memory::now()<self.review.expires_at}
 pub(in crate::pet_runtime)fn revoke_publication(&self){self.review.revoked.store(true,Ordering::Release);}
 pub(in crate::pet_runtime)fn note_published(&self){self.published.store(true,Ordering::Release);}
 pub(in crate::pet_runtime)fn matches_request(&self,owner:&Owned,g:&Record,request:&PreparedRuntimeRequest,observe:&Observe,now:u64)->bool{
  self.current(owner,g,observe,now)==Validity::Current&&request.turn_binding().as_ref()==Some(&self.review.turn_binding)&&self.review.body==*request.body()
   &&self.review.data.current(request.server(),&owner.lease().model,now)
   &&model::fingerprint(&(&owner.lease().id,&owner.lease().model,request.body())).ok().as_ref()==Some(&self.consent.input_hash)
 }
}
