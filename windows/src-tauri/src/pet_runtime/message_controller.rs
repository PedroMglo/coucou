//! A retained native exact review, not a serialized message or a CLI session,
//! authorizes one local delivery. No model call or recipient execution here.
use super::*;
use crate::pet_runtime::{messages::{self,Input,Target,Scope,Envelope,Origin,Cause,Receipt},model::WorkState};
#[derive(Clone,serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct Proposal{#[serde(skip_serializing_if="Option::is_none")]pub received_message:Option<Envelope>,pub envelope:Envelope,pub goal_revision:String,pub intent:String,pub criteria:Vec<String>,pub expires_at:u64,pub destination_execution_authorized:bool}
pub(in crate::pet_runtime)struct Review{lineage:Option<Arc<super::PeerRoundPublished>>,scope:Scope,target:Arc<Target>,expires:Instant,received_message:Option<Envelope>,work:Option<crate::pet_runtime::work_report::Captured>}
pub(in crate::pet_runtime)struct Approved{review:Arc<Review>,human:Arc<dyn Fn()->bool+Send+Sync>}
fn denied()->model::Error{model::Error::Conflict}
impl Owned{
 pub(in crate::pet_runtime)fn message_goal(&self,goal:&Record,now:u64)->model::Result<()>{
  if goal.validate().is_err()||goal.state!=model::State::Leased||(!matches!(self.live.spec.schema_version,2|4|5|6|7|8|9|10|11|14|16|17|18|20)&&!self.live.spec.peer_planning())||self.live.spec.planning_policy.is_some()&&!self.live.spec.peer_planning()||goal.id!=self.live.lease.goal_id||goal.spec.owner!=self.live.spec.owner
   ||model::fingerprint(&goal.spec)?!=self.live.spec.intent_hash||goal.execution_context.as_ref()!=Some(&self.live.spec.context)||goal.admissions.last().is_none_or(|r|r.lease.as_ref()!=Some(&self.live.lease)||model::fingerprint(&r.spec).ok()!=model::fingerprint(&self.live.spec).ok())
   ||!crate::pet_runtime::plans::matches(goal,&self.live.spec)||self.policy_validity(now)!=Validity::Current{return Err(denied());}Ok(())
 }
 pub(in crate::pet_runtime)fn review_message(&self,goal:&Record,plan_id:&str,step_id:&str,request_id:String,target:Arc<Target>,now:u64)->model::Result<Review>{self.review_message_inner(goal,plan_id,step_id,request_id,target,now,true,None)}
 /// Only a native complete-plan review can prepare later-step envelopes before
 /// earlier receipts. The batch still needs one exact consumed human decision.
 pub(in crate::pet_runtime)fn review_message_for_batch(&self,goal:&Record,plan_id:&str,step_id:&str,request_id:String,target:Arc<Target>,now:u64)->model::Result<Review>{self.review_message_inner(goal,plan_id,step_id,request_id,target,now,false,None)}
 pub(in crate::pet_runtime)fn review_generated_message_for_batch(&self,goal:&Record,plan_id:&str,step_id:&str,request_id:String,target:Arc<Target>,now:u64,generated:crate::pet_runtime::generated_plans::Reference)->model::Result<Review>{self.review_message_inner(goal,plan_id,step_id,request_id,target,now,false,Some(generated))}
 fn review_message_inner(&self,goal:&Record,plan_id:&str,step_id:&str,request_id:String,target:Arc<Target>,now:u64,ordered:bool,generated:Option<crate::pet_runtime::generated_plans::Reference>)->model::Result<Review>{
  self.message_goal(goal,now)?;
  if self.live.spec.peer_planning(){let r=generated.as_ref().ok_or(model::Error::UnsupportedPlatform)?;let authority=Authority::existing(&self.live.authority_path).map_err(crate::pet_runtime::authority::Failure::control_error)?;let(reference,_)=authority.generated_candidate(&self.live.spec,&self.live.proof,&self.live.lease,goal,plan_id).map_err(crate::pet_runtime::authority::Failure::control_error)?;if r!=&reference{return Err(denied());}}
  else if generated.is_some(){return Err(denied());}
  if !model::key(&request_id)||self.live.spec.budget.max_tool_calls==0{return Err(model::Error::LimitReached);}
  let plan=goal.plans.iter().find(|p|p.id==plan_id).filter(|p|goal.tasks.iter().any(|t|t.id==p.task_id&&t.plan_id.as_deref()==Some(plan_id)&&t.state==WorkState::Proposed)).ok_or(denied())?;
  // This first direct ingress uses message-only plans. It does not pretend the
  // existing Spotify recipe executor understands mixed/peer operations.
  if plan.steps.iter().any(|s|s.operation.as_ref().is_none_or(|o|Input::parse(o).is_err())){return Err(model::Error::UnsupportedPlatform);}
  let step=plan.steps.iter().find(|s|s.id==step_id&&s.state==WorkState::Proposed).ok_or(denied())?;
  let arguments=Input::parse(step.operation.as_ref().ok_or(denied())?).map_err(|_|model::Error::InvalidInput)?;
  if arguments.to_pet()!=target.recipient().pet_id||arguments.to_pet()==goal.spec.owner.pet_id||arguments.deadline()>self.live.lease.deadline_at||arguments.deadline()<=now{return Err(denied());}
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
  if self.live.spec.continuation.as_ref().and_then(|c|c.native.as_ref()).is_some_and(|n|n.covers(plan_id,step_id))||authority.message_at(&self.live.spec,&self.live.proof,plan_id,step_id).map_err(|_|denied())?.is_some(){return Err(denied());}
  if ordered{for prior in plan.steps.iter().take_while(|s|s.id!=step_id){if !self.live.spec.continuation.as_ref().and_then(|c|c.native.as_ref()).is_some_and(|n|n.covers(plan_id,&prior.id))&&authority.message_at(&self.live.spec,&self.live.proof,plan_id,&prior.id).map_err(|_|denied())?.is_none(){return Err(denied());}}}
  if matches!(&arguments,Input::Delegate(_))&&!self.live.spec.pool_budget(){return Err(denied());}
  if let Input::FundedReply(a)=&arguments{if self.live.spec.funding.as_ref().is_none_or(|f|f.source.reference!=a.source||f.source.sender_pet_id!=a.to_pet_id){return Err(denied());}}
  let derived_result=if let Input::ResultReply(a)=&arguments{let p=self.live.spec.result_disclosure.as_ref().filter(|p|matches!(self.live.spec.schema_version,14|20)&&p.for_spec(&self.live.spec)&&p.matches(goal)&&p.arguments==*a&&p.cause.plan_id==plan_id&&p.cause.step_id==step_id&&p.recipient==*target.recipient()).ok_or(denied())?;Some(p.published())}else{if self.live.spec.result_disclosure.is_some(){return Err(denied());}None};
  let id=nonce()?;let origin=Origin{pet_id:goal.spec.owner.pet_id.clone(),provider:self.live.lease.provider.clone(),adapter:goal.spec.owner.adapter.clone(),conversation_id:self.live.lease.conversation_id.clone(),goal_id:goal.id.clone(),decision_id:self.live.lease.decision_id.clone(),lease_id:self.live.lease.id.clone(),controller_id:self.live.lease.controller_id.clone()};
  let recursive=self.live.spec.schema_version==17&&matches!(arguments,Input::Delegate(_));let source=if recursive{self.live.spec.funding.as_ref().map(|f|&f.source.reference)}else{arguments.source()};
  let received_message=source.map(|r|authority.received_message(&self.live.spec,&self.live.proof,&self.live.lease,r,now).map(|r|r.envelope().clone()).map_err(crate::pet_runtime::authority::Failure::control_error)).transpose()?;
  let work=match &arguments{Input::WorkReport(args)=>Some(crate::pet_runtime::work_report::Captured::at(&self.goal_path,&self.live.authority_path,args,&goal.spec.owner.pet_id,now)?),Input::WorkDisclosure(args)=>{let p=self.live.spec.work_disclosure.as_ref().filter(|p|p.matches(goal)&&p.plan_id==plan_id&&p.step_id==step_id&&p.arguments==*args&&p.recipient==*target.recipient()).ok_or(denied())?;let c=p.arguments.capture(&self.goal_path,&self.live.authority_path,&goal.spec.owner.pet_id,now)?;if !p.arguments.states.contains(&c.report.state){return Err(denied());}Some(c)},_=>None};
  let delegation=if let Input::Delegate(a)=&arguments{Some(if recursive||self.live.spec.peer_funded_iteration(){authority.recursive_payload(&self.live.spec,&self.live.proof,a,now).map_err(crate::pet_runtime::authority::Failure::control_error)?}else{a.payload(&goal.spec.owner.pet_id)})}else{None};
  let planned_data=if self.live.spec.peer_selected_data()&&generated.as_ref().is_some_and(|r|r.round_index==1){if !(matches!(&arguments,Input::Message(_))||self.live.spec.peer_selected_iteration()&&matches!(&arguments,Input::Delegate(_))){return Err(model::Error::UnsupportedPlatform);}Some(crate::pet_runtime::planned_peer_data::Published::from_memory(&self.live.spec,generated.as_ref().ok_or(denied())?.clone())?)}else{None};
  let planned_data=if self.live.spec.peer_iteration(){let r=generated.as_ref().ok_or(denied())?;let journal=authority.planning_ledger(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(crate::pet_runtime::authority::Failure::control_error)?;
   if let Some(data)=journal.rounds.iter().find(|round|round.attempt_id==r.output.attempt_id).and_then(|round|round.peer_data.as_ref()){
    if !matches!(&arguments,Input::Message(_)|Input::Delegate(_)){return Err(model::Error::UnsupportedPlatform);}Some(crate::pet_runtime::planned_peer_data::Published::from_round(&self.live.spec,r.clone(),data.clone())?)
   }else{planned_data}
  }else{planned_data};
  let envelope=Envelope{planned_data:planned_data.clone(),derived_result:derived_result.clone(),delegation:delegation.clone(),work_report:work.as_ref().map(|c|c.report.clone()),reply_to:source.cloned(),schema_version:if planned_data.as_ref().is_some_and(|p|p.schema_version==4){if delegation.is_some(){16}else{15}}else if planned_data.as_ref().is_some_and(|p|p.schema_version==3){if delegation.is_some(){14}else{13}}else if planned_data.as_ref().is_some_and(|p|p.schema_version==2){if delegation.is_some(){12}else{11}}else if planned_data.is_some(){10}else if recursive{7}else if self.live.spec.schema_version==18&&matches!(arguments,Input::FundedReply(_)){8}else if derived_result.as_ref().is_some_and(|d|d.schema_version==2){9}else if derived_result.is_some(){6}else if matches!(arguments,Input::FundedReply(_)){5}else if delegation.is_some(){4}else if work.is_some(){3}else if received_message.is_some(){2}else{1},message_id:id.clone(),origin,recipient:target.recipient().clone(),correlation_id:received_message.as_ref().map(|e|e.correlation_id.clone()).unwrap_or(id),cause:Cause{task_id:plan.task_id.clone(),plan_id:plan_id.into(),step_id:step_id.into()},kind:arguments.kind(),text:arguments.text().into(),created_at:now,deadline_at:arguments.deadline(),provider_use_authorized:false};
  if received_message.as_ref().is_some_and(|parent|!if recursive{crate::pet_runtime::delegation::matches_parent(&envelope,parent)}else if work.is_some(){crate::pet_runtime::work_report::matches_parent(&envelope,parent)}else{messages::reply_matches(&envelope,parent)}){return Err(denied());}
  let proposal_hash=if let Some(r)=&generated{r.proposal_hash.clone()}else{self.live.spec.proposal_hash.clone().ok_or(denied())?};
  let scope=Scope{generated_plan:generated,envelope,request_id,goal_revision:goal.revision.clone(),main_spec_hash:model::fingerprint(&self.live.spec)?,proposal_hash};
  if !scope.belongs(&self.live.spec,&self.live.lease)||!scope.matches(goal){return Err(denied());}
  let expires=Instant::now()+Duration::from_millis(120_000.min(scope.envelope.deadline_at-now));Ok(Review{lineage:None,scope,target,expires,received_message,work})
 }
 pub(in crate::pet_runtime)fn send_message(&self,goal:&Record,approved:&Approved,now:u64,observe:impl Fn()->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>)->model::Result<Receipt>{
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
  // Durable observation comes before checking current workers/Frame/metadata.
  // A late ACK cannot renew authority or debit a second message.
  if let Some(receipt)=authority.message_ack(&self.live.spec,&self.live.proof,approved.scope()).map_err(|_|denied())?{
   return if receipt.scope==*approved.scope(){Ok(receipt)}else{Err(denied())};
  }
  approved.sources_current()?;
  let store=Store::existing_verified(&self.goal_path,&self.live.authority_path)?.ok_or(model::Error::NotFound)?;
  store.with_message_goal(goal,|goal|{
  let(config,settings)=observe()?;self.message_goal(goal,now)?;if !approved.scope().matches(goal)||!approved.current(&config,&settings,now){return Err(denied());}
  let _worker=self.readiness_worker(now)?;
  authority.deliver_message(goal,&self.live.spec,&self.live.proof,&self.live.lease,approved,now,||{
   // Authority owns its exclusive lock here. Do not recursively acquire it via
   // Live.validity or the verified Store reader. Its exact goal is already
   // held by the source reader; signed phase/lease/budget stay checked here.
   observe().is_ok_and(|(config,settings)|approved.current(&config,&settings,crate::assistant_memory::now()))&&self.message_control_current(goal)
  }).map_err(crate::pet_runtime::authority::Failure::control_error)
  })
 }
 pub(in crate::pet_runtime)fn message_control_current(&self,goal:&Record)->bool{
  let now=crate::assistant_memory::now();let live=&self.live;
  !live.closed.load(Ordering::Acquire)&&live.activated.load(Ordering::Acquire)&&Instant::now()<live.deadline&&now>=live.lease.claimed_at&&now<live.lease.deadline_at
   &&live.hold.current()&&live.hold.captured().is_none_or(|c|c.current())&&live.owner.current().is_ok()&&live.chat.goal_context_current(&live.lease.goal_id)
   &&(live.fence)(&live.lease,&live.hold,Validation::Commit(goal))==Validity::Current
 }
}
impl Review{
 pub(in crate::pet_runtime)fn scope_for_recovery(&self)->&Scope{&self.scope}
 pub(in crate::pet_runtime)fn bind_round_lineage(&mut self,scope:Arc<super::PeerRoundPublished>)->model::Result<()>{if self.lineage.is_some()||!scope.matches_delivery(&self.scope){return Err(denied());}self.lineage=Some(scope);Ok(())}

 pub(in crate::pet_runtime)fn proposal(&self,goal:&Record)->Proposal{Proposal{received_message:self.received_message.clone(),envelope:self.scope.envelope.clone(),goal_revision:self.scope.goal_revision.clone(),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),expires_at:self.scope.envelope.deadline_at.min(self.scope.envelope.created_at+120_000),destination_execution_authorized:false}}
 /// Only the native signed DATA review permits omitting a second per-message
 /// dialog. A plan, budget or caller-supplied envelope cannot grant this scope.
 pub(in crate::pet_runtime)fn approve_native_peer_plan(self,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->model::Result<Approved>{
  let p=owner.live.spec.native_peer_plan.as_ref().filter(|p|p.matches_envelope(&owner.live.spec,goal,&self.scope.envelope)&&p.targets_current(config,settings)&&now>=p.prepared_at&&now<p.expires_at).ok_or(denied())?;
  p.sources_current(goal,&owner.live.authority_path,now)?;
  self.approve(owner,goal,config,settings,now,Arc::new(||true))
 }
 pub(in crate::pet_runtime)fn approve_disclosure(self,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->model::Result<Approved>{
  if let Some(p)=&owner.live.spec.result_disclosure{if !matches!(owner.live.spec.schema_version,14|20)||!p.matches(goal)||!p.matches_envelope(&self.scope.envelope){return Err(denied());}return self.approve(owner,goal,config,settings,now,Arc::new(||true));}
  let p=owner.live.spec.work_disclosure.as_ref().filter(|p|owner.live.spec.schema_version==9&&p.matches(goal)&&p.plan_id==self.scope.envelope.cause.plan_id&&p.step_id==self.scope.envelope.cause.step_id&&p.recipient==self.scope.envelope.recipient&&p.arguments.matches(&self.scope.envelope)).ok_or(denied())?;
  if self.scope.envelope.reply_to.as_ref()!=Some(&p.arguments.source){return Err(denied());}
  self.approve(owner,goal,config,settings,now,Arc::new(||true))
 }
 pub(in crate::pet_runtime)fn approve(self,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Approved>{
  Arc::new(self).approve_shared(owner,goal,config,settings,now,human)
 }
 pub(in crate::pet_runtime)fn approve_shared(self:&Arc<Self>,owner:&Owned,goal:&Record,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->model::Result<Approved>{
  owner.message_goal(goal,now)?;if self.scope.envelope.planned_data.as_ref().is_some_and(|p|p.round_data.is_some())&&self.lineage.as_ref().is_none_or(|s|!s.matches_delivery(&self.scope)){return Err(denied());}if !self.scope.matches(goal)||!self.scope.belongs(&owner.live.spec,&owner.live.lease)||!human(){return Err(denied());}
  let approved=Approved{review:self.clone(),human};if !approved.current(config,settings,now){return Err(denied());}Ok(approved)
 }
 pub(in crate::pet_runtime)fn metadata_current(&self,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->bool{
  now>=self.scope.envelope.created_at&&now<self.scope.envelope.deadline_at&&Instant::now()<self.expires&&self.target.current(config,settings)&&self.work.as_ref().is_none_or(|w|w.current(now).is_ok())
 }
}
impl Approved{
 pub(in crate::pet_runtime)fn sources_current(&self)->model::Result<()>{match self.review.lineage.as_ref().map(|s|s.current()).unwrap_or(Validity::Current){Validity::Current=>Ok(()),Validity::Busy=>Err(model::Error::Busy),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>Err(denied())}}
 pub(in crate::pet_runtime)fn data_current(&self,now:u64)->bool{self.review.lineage.as_ref().is_none_or(|s|s.not_revoked())&&self.review.work.as_ref().is_none_or(|w|w.current(now).is_ok())}
 pub(in crate::pet_runtime)fn scope(&self)->&Scope{&self.review.scope}
 pub(in crate::pet_runtime)fn received_message(&self)->Option<&Envelope>{self.review.received_message.as_ref()}
 fn current(&self,config:&crate::pet_config::Configuration,settings:&crate::settings::Settings,now:u64)->bool{now>=self.scope().envelope.created_at&&now<self.scope().envelope.deadline_at&&Instant::now()<self.review.expires&&(self.human)()&&self.review.work.as_ref().is_none_or(|w|w.current(now).is_ok())&&self.review.target.current(config,settings)}
}
#[cfg(test)]#[path="message_tests.rs"]mod tests;
