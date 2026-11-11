//! Native-reviewed ownership handoff. Original admission, quota, messages and
//! child reservations remain in the same signed Grant; DATA approvals do not.
use super::*;
use crate::pet_runtime::{continuation::{CapturedPeerPlanning,PeerPlanningSnapshot,PeerPlanningOutcome},native_owner::AbsentLease,owner_resumption::{Reference,Seal}};
#[derive(Clone,serde::Serialize)]#[serde(rename_all="camelCase")]
pub(in crate::pet_runtime)struct Proposal{
 pub request_id:String,pub snapshot:PeerPlanningSnapshot,pub prepared_at:u64,pub expires_at:u64,
 pub approval_scope:&'static str,pub new_budget:bool,pub new_model_data_consent:bool,
}
pub(in crate::pet_runtime)struct Review{source:CapturedPeerPlanning,goal:Record,body:crate::pet_runtime::authority::Grant,absence:AbsentLease,proposal:Proposal,id:String,target_revision:String}
pub(in crate::pet_runtime)struct Approved{
 pub(in crate::pet_runtime)source:CapturedPeerPlanning,pub(in crate::pet_runtime)goal:Record,
 body:crate::pet_runtime::authority::Grant,absence:Option<AbsentLease>,proposal:Proposal,frame:Arc<dyn Fn()->bool+Send+Sync>,
 reference:Reference,pub(in crate::pet_runtime)seal:Option<Seal>,pub(in crate::pet_runtime)candidate:Option<Record>,
 live:Option<Arc<Live>>,pub(in crate::pet_runtime)write_started:bool,drain_request:String,
}
pub(in crate::pet_runtime)struct OwnerProof{live:Arc<Live>}
impl OwnerProof{
 pub(in crate::pet_runtime)fn record(&self)->&crate::pet_runtime::runtime_owner::Record{self.live.owner.record()}
 pub(in crate::pet_runtime)fn current(&self)->model::Result<()>{self.live.owner.current()}
 pub(in crate::pet_runtime)fn resumes(&self,old:&crate::pet_runtime::runtime_owner::Record)->bool{self.live.owner.resumes(old)}
}
impl Review{
 pub(in crate::pet_runtime)fn authority_path(&self)->PathBuf{self.source.authority.root_path()}
 pub(in crate::pet_runtime)fn current(&self,now:u64)->model::Result<()>{
  if now<self.proposal.prepared_at||now>=self.proposal.expires_at{return Err(model::Error::Conflict);}self.source.current(now)?;self.absence.current(&self.source.authority)
 }
 pub(in crate::pet_runtime)fn prepare(source:CapturedPeerPlanning,request:String,now:u64)->model::Result<Self>{
  source.current(now)?;let snapshot=source.snapshot();
  if snapshot.outcome!=PeerPlanningOutcome::CanonicalPlan||snapshot.charge.unknown_model_results!=0||snapshot.charge.unknown_tool_results!=0||!model::key(&request){return Err(model::Error::Conflict);}
  let goal=source.store.read_control(&snapshot.goal_id,&snapshot.pet_id)?;
  if goal.events.iter().any(|e|e.request_id==request){return Err(model::Error::Conflict);}
  let body=source.authority.recovery_body(&goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  if body.owner_resumptions.len()>=crate::pet_runtime::owner_resumption::MAX_RESUMPTIONS{return Err(model::Error::LimitReached);}
  let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
  let absence=source.authority.retained_recovery_absence(&row.spec,row.proof.as_ref().ok_or(model::Error::Conflict)?).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  if !absence.matches(&snapshot.grant_hash,&body.revision){return Err(model::Error::Conflict);}
  let proposal=Proposal{request_id:request,snapshot,prepared_at:now,expires_at:now.checked_add(30_000).ok_or(model::Error::ClockRollback)?.min(row.spec.budget.deadline_at),approval_scope:"resumeOriginalPeerExecution",new_budget:false,new_model_data_consent:false};
  Ok(Self{source,goal,body,absence,proposal,id:nonce()?,target_revision:nonce()?})
 }
 pub(in crate::pet_runtime)fn goal(&self)->&Record{&self.goal}
 pub(in crate::pet_runtime)fn proposal(&self)->Proposal{self.proposal.clone()}
 /// Called only after the exact native review's own affirmative decision.
 pub(in crate::pet_runtime)fn approve(self,now:u64,frame:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Approved>{
  if !frame()||now<self.proposal.prepared_at||now>=self.proposal.expires_at{return Err(model::Error::Conflict);}
  self.current(now)?;
  let reference=Reference{schema_version:1,id:self.id,request_id:self.proposal.request_id.clone(),goal_id:self.goal.id.clone(),pet_id:self.goal.spec.owner.pet_id.clone(),decision_id:self.body.decision_id.clone(),lease_id:self.body.lease.as_ref().ok_or(model::Error::Conflict)?.id.clone(),source_revision:self.goal.revision.clone(),source_sequence:self.goal.sequence,source_goal_hash:model::fingerprint(&self.goal)?,source_grant_revision:self.body.revision.clone(),source_grant_hash:model::fingerprint(&self.body)?,target_revision:self.target_revision,target_sequence:self.goal.sequence+1,prepared_at:self.proposal.prepared_at,approved_at:now};
  Ok(Approved{source:self.source,goal:self.goal,body:self.body,absence:Some(self.absence),proposal:self.proposal,frame,reference,seal:None,candidate:None,live:None,write_started:false,drain_request:nonce()?})
 }
}
impl Approved{
 pub(in crate::pet_runtime)fn prepared(&self)->bool{self.live.is_some()}
 pub(in crate::pet_runtime)fn hold(&self)->Option<&Hold>{self.live.as_ref().map(|l|l.hold.as_ref())}
 pub(in crate::pet_runtime)fn pending(&self)->model::Result<Pending>{Ok(Pending{live:self.live.as_ref().ok_or(model::Error::Conflict)?.clone(),drain_request:self.drain_request.clone(),resumed:true})}
 pub(in crate::pet_runtime)fn withdraw(&self)->model::Result<bool>{if self.live.is_none(){return Ok(true);}self.source.store.withdraw_peer_owner(self)}
 pub(in crate::pet_runtime)fn published(&self)->model::Result<bool>{
  let Some(seal)=&self.seal else{return Ok(false);};let body=self.source.authority.recovery_body(&self.goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  Ok(body.runtime_owner.as_ref()==Some(&seal.owner)&&body.owner_resumptions.iter().any(|s|model::fingerprint(s).ok()==model::fingerprint(seal).ok()))
 }
 pub(in crate::pet_runtime)fn native_owner(&self)->model::Result<OwnerProof>{self.live.as_ref().map(|l|OwnerProof{live:l.clone()}).ok_or(model::Error::Conflict)}
 pub(in crate::pet_runtime)fn approval_current(&self,now:u64)->model::Result<()>{
  if !(self.frame)()||now<self.reference.approved_at||now>=self.proposal.expires_at{return Err(model::Error::Conflict);}Ok(())
 }
 pub(in crate::pet_runtime)fn current_native(&self,now:u64)->model::Result<()>{
  if now<self.reference.approved_at||now>=self.proposal.snapshot.original_budget.deadline_at||Instant::now()>=self.source.deadline{return Err(model::Error::Conflict);}
  if !self.write_started{self.approval_current(now)?;}
  if let Some(live)=&self.live{if live.closed.load(Ordering::Acquire)||!live.hold.current(){return Err(model::Error::Conflict);}live.owner.current()?;}
  Ok(())
 }
 pub(in crate::pet_runtime)fn prepare_worker(&mut self,main:&Chat,contexts:&Path,evidence:&crate::pet_runtime::admission::OwnerResumptionEvidence,fence:Fence,now:u64)->model::Result<()>{
  self.current_native(now)?;if self.live.is_some(){return Ok(());}self.source.current(now)?;
  let row=self.goal.admissions.last().ok_or(model::Error::Conflict)?;let lease=self.body.lease.clone().ok_or(model::Error::Conflict)?;
  let(capability,policy)=evidence.for_owner_resumption(&self.goal,now).map_err(|_|model::Error::Conflict)?;
  // Descriptor revisions include the backend's fresh observation incarnation.
  // Revalidate the actual binding, selected context and policy passively;
  // preserve the old lease epoch and record the new observation separately.
  if policy!=lease.policy_revision{return Err(model::Error::Conflict);}
  let(_,chat)=context::open_with_authority(contexts,&self.goal,main,&self.source.authority.root_path())?;
  if !chat.goal_context_current(&self.goal.id){return Err(model::Error::Conflict);}
  let hold=main.runtime_resources().acquire_goal(&lease.provider,Arc::new(AtomicBool::new(false)),Some(self.source.deadline)).map_err(|e|if e==crate::pet_resources::Failure::Busy{model::Error::Busy}else{model::Error::StorageUnavailable})?;
  let absence=self.absence.take().ok_or(model::Error::Conflict)?;
  let owner=absence.into_resumed_owner(&self.source.authority,&lease,now)?;
  let mut candidate=self.goal.clone();candidate.owner_resumed(&self.reference)?;
  let seal=Seal{schema_version:1,reference:self.reference.clone(),target_goal_hash:model::fingerprint(&candidate)?,closure:self.body.recovery.clone().ok_or(model::Error::Conflict)?,previous_owner:self.body.runtime_owner.clone().ok_or(model::Error::Conflict)?,owner:owner.record().clone(),previous_closed_at:self.body.closed_at.ok_or(model::Error::Conflict)?,previous_updated_at:self.body.updated_at,previous_resource_usage:self.body.resource_usage.clone(),usage_hash:Seal::usage_hash(&self.body)?,capability_revision:capability,policy_revision:policy};
  let live=Arc::new(Live{lease,spec:row.spec.clone(),proof:row.proof.clone().ok_or(model::Error::Conflict)?,authority_path:self.source.authority.root_path(),chat,hold,deadline:self.source.deadline,closed:AtomicBool::new(false),activated:AtomicBool::new(false),driver:AtomicBool::new(false),workers:AtomicUsize::new(0),gate:Mutex::new(()),fence,waiting:Mutex::new(None),native_completion:Mutex::new(None),native_drain:Mutex::new(None),owner,interruption:OnceLock::new()});
  {let mut map=live_registry().lock().map_err(|_|model::Error::StorageUnavailable)?;map.retain(|_,w|w.strong_count()>0);if map.contains_key(&live.lease.id)||map.len()>=64{return Err(model::Error::Busy);}map.insert(live.lease.id.clone(),Arc::downgrade(&live));}
  self.live=Some(live);self.seal=Some(seal);self.candidate=Some(candidate);Ok(())
 }
 pub(in crate::pet_runtime)fn confirm(&mut self,now:u64)->model::Result<(model::Receipt,Option<Owned>)>{
  self.current_native(now)?;let store=self.source.store.clone();let receipt=store.confirm_peer_owner_resumption(self,now)?;
  if receipt.status==CommitStatus::Uncertain{return Ok((receipt,None));}
  let live=self.live.as_ref().ok_or(model::Error::Conflict)?;
  if live.validity_at(now,Validation::InFlightPolicy)==Validity::Changed{return Err(model::Error::Conflict);}
  if live.activated.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Ok((receipt,None));}
  // Ownership is not an IO permit. Like common Pending::claim, return the
  // retained native owner even if fresh availability is temporarily Busy.
  // Every worker still performs its own strict effect-time authority check.
  Ok((receipt,Some(Owned{live:live.clone(),drain_request:self.drain_request.clone(),goal_path:store.root_path().to_owned()})))
 }
}
