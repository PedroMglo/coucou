//! Single canonical delivery, receipt and budget transaction in the existing
//! Authority. Staging is never an inbox and stored JSON is never an Approved.
use super::*;
use crate::pet_runtime::{messages::{Ledger as Messages,Scope,Receipt,Stage,MAX_MESSAGES},lease_controller::message_controller::Approved,leases::Lease,effect_budget::{Ledger,Attempt,Kind,Phase}};
impl Authority{
 pub(in crate::pet_runtime)fn message_at(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<Receipt>>{
  let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
  Ok(body.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.scope.envelope.cause.plan_id==plan&&r.scope.envelope.cause.step_id==step)).cloned())
 }
 pub(in crate::pet_runtime)fn message_ack(&self,spec:&AdmissionSpec,proof:&ConsentProof,scope:&Scope)->Result<Option<Receipt>>{
  let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
  let Some(receipt)=body.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.scope.envelope.cause==scope.envelope.cause))else{return Ok(None);};
  if receipt.scope!=*scope{return Err(Failure::Changed);}self.stabilize(&body)?;Ok(Some(receipt.clone()))
 }
 pub(in crate::pet_runtime)fn message_recovery_ready(&self,spec:&AdmissionSpec,proof:&ConsentProof,scope:&Scope)->Result<()>{let _lock=self.lock_shared()?;self.load(spec,proof)?;self.pending_message_scope(spec,scope)}
 fn pending_message_scope(&self,spec:&AdmissionSpec,scope:&Scope)->Result<()>{
  // Known unpublished attempts may be retried only with the exact original
  // native approval. A new token cannot bypass a staging/ACK ambiguity.
  for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
   if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}
   if self.rule_record_data(&name)?.is_some(){continue;}let(body,_)=self.grant_file(&name)?;if body.decision_id!=spec.decision_id{continue;}
   if let Some(messages)=body.messages{for r in messages.entries{let old=&r.scope;let e=&scope.envelope;let o=&old.envelope;if o.cause.plan_id==e.cause.plan_id&&o.cause.step_id==e.cause.step_id&&old!=scope{return Err(Failure::RecoveryRequired);}}}
  }Ok(())
 }
 pub(in crate::pet_runtime)fn deliver_message(&self,goal:&Record,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,approved:&Approved,now:u64,current:impl Fn()->bool)->Result<Receipt>{
  let scope=approved.scope();if !scope.belongs(spec,lease){return Err(Failure::Invalid);}
  let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
  if let Some(old)=body.messages.as_ref().and_then(|m|m.entries.iter().find(|r|r.scope.envelope.cause==scope.envelope.cause||r.scope.request_id==scope.request_id)){
   if old.scope!=*scope{return Err(Failure::Changed);}self.stabilize(&body)?;return Ok(old.clone());
  }
  if let Some(c)=&spec.continuation{if !c.present(goal,spec){return Err(Failure::Changed);}self.continuation_prefix(goal,c)?;}
  if spec.continuation.as_ref().and_then(|c|c.native.as_ref()).is_some_and(|n|n.covers(&scope.envelope.cause.plan_id,&scope.envelope.cause.step_id)){return Err(Failure::Revoked);}
  if body.native_completion.is_some(){return Err(Failure::Revoked);}
  if body.messages.as_ref().is_some_and(|m|m.retired.iter().any(|r|r.scope.envelope.message_id==scope.envelope.message_id||r.scope.request_id==scope.request_id)){return Err(Failure::Revoked);}
  self.pending_message_scope(spec,scope)?;self.funding_current_unlocked(spec,now)?;self.recursive_message_unlocked(spec,scope,now)?;if let Some(p)=&spec.result_disclosure{if !p.matches_envelope(&scope.envelope){return Err(Failure::Changed);}self.derived_data_unlocked(goal,p,now)?;}
  if body.phase!=GrantPhase::Claimed||body.lease.as_ref()!=Some(lease)||body.runtime_owner.is_none()||now<body.updated_at||now<scope.envelope.created_at||now>=scope.envelope.deadline_at||spec.budget.max_tokens.is_some()||spec.budget.max_cost_micros.is_some()||!current()||!approved.data_current(crate::assistant_memory::now()){return Err(Failure::Revoked);}
  if body.messages.as_ref().is_some_and(|m|m.entries.len()+m.retired.len()>=MAX_MESSAGES){return Err(Failure::Capacity);}
  self.peer_message_current(goal,spec,&body,scope)?;
  let bytes=serde_json::to_vec(&scope.envelope).map_err(|_|Failure::Invalid)?.len()as u64;let id=nonce().map_err(|_|Failure::Unavailable)?;
  let attempt=Attempt{id:id.clone(),request_id:format!("message-{}",scope.envelope.message_id),input_hash:model::fingerprint(scope).map_err(|_|Failure::Invalid)?,kind:Kind::Tool,output_reserved:bytes,phase:Phase::Settled,accepted_output:Some(bytes),reserved_at:now,started_at:Some(now),finished_at:Some(now)};
  if scope.envelope.delegation.is_some(){if !spec.pool_budget()||body.delegation_pool.as_ref()!=Some(&spec.budget){return Err(Failure::Invalid);}body.delegation_reservations.push(crate::pet_runtime::delegation::Reservation::capture(scope,now).map_err(|_|Failure::Invalid)?);}
  let budget=body.budget_usage.get_or_insert_with(Ledger::empty);budget.attempts.push(attempt);if !budget.valid_for(spec){return Err(Failure::Capacity);}
  let receipt=Receipt{scope:scope.clone(),stage:Stage::Received,received_at:now,attempt_id:id};body.messages.get_or_insert_with(||Messages::empty()).entries.push(receipt.clone());body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;
  if !body.pool_valid(){return Err(Failure::Capacity);}
  let peer_body=if spec.peer_planning(){Some(body.clone())}else{None};
  self.publish_guarded(body,false,||{
   self.funding_current_unlocked(spec,crate::assistant_memory::now())?;self.recursive_message_unlocked(spec,scope,crate::assistant_memory::now())?;if let Some(p)=&spec.result_disclosure{if !p.matches_envelope(&scope.envelope){return Err(Failure::Changed);}self.derived_data_unlocked(goal,p,crate::assistant_memory::now())?;}
   if let Some(b)=&peer_body{self.peer_message_current(goal,spec,b,scope)?;}
   if !current()||!approved.data_current(crate::assistant_memory::now()){return Err(Failure::Revoked);}
   if let Some(reference)=&scope.envelope.reply_to{
    let source=self.received_message_unlocked(spec,proof,lease,reference,crate::assistant_memory::now())?;
    if approved.received_message()!=Some(source.envelope())||!if scope.envelope.schema_version==7{crate::pet_runtime::delegation::matches_parent(&scope.envelope,source.envelope())}else if scope.envelope.work_report.is_some(){crate::pet_runtime::work_report::matches_parent(&scope.envelope,source.envelope())}else{crate::pet_runtime::messages::reply_matches(&scope.envelope,source.envelope())}{return Err(Failure::Changed);}
   }else if approved.received_message().is_some(){return Err(Failure::Changed);}
   Ok(())
  })?;Ok(receipt)
 }
 fn peer_message_current(&self,goal:&Record,spec:&AdmissionSpec,body:&Grant,scope:&Scope)->Result<()>{
  if let Some(data)=scope.envelope.planned_data.as_ref().and_then(|p|p.round_data.as_ref()){self.peer_round_sources_unlocked(body,data,scope.generated_plan.as_ref().ok_or(Failure::Invalid)?,crate::assistant_memory::now())?;}
  if spec.peer_planning(){let r=scope.generated_plan.as_ref().ok_or(Failure::Invalid)?;Self::generated_acknowledged(body,spec,r)?;
   let data=body.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|round|round.attempt_id==r.output.attempt_id)).ok_or(Failure::Missing)?.peer_data.as_ref();if spec.peer_iteration()&&data!=scope.envelope.planned_data.as_ref().and_then(|p|p.round_data.as_ref()){return Err(Failure::Changed);}
   if crate::pet_runtime::planned_peer::reference(goal,spec).map_err(|_|Failure::Changed)?!=r{return Err(Failure::Changed);}
   let policy=spec.planning_policy.as_ref().ok_or(Failure::Invalid)?;let plan=goal.plans.iter().find(|p|p.id==r.plan_id).ok_or(Failure::Changed)?;if plan.steps.iter().any(|s|s.operation.as_ref().is_none_or(|op|!crate::pet_runtime::planned_peer::operation_for(op,&spec.owner.pet_id,spec.budget.deadline_at,policy))){return Err(Failure::Invalid);}
   for prior in plan.steps.iter().take_while(|s|s.id!=scope.envelope.cause.step_id){if !body.messages.as_ref().is_some_and(|m|m.entries.iter().any(|e|e.envelope().cause.plan_id==plan.id&&e.envelope().cause.step_id==prior.id)){return Err(Failure::Changed);}}
  }else if scope.generated_plan.is_some(){return Err(Failure::Invalid);}Ok(())
 }
 /// Native caller-specific inbox is only a filtered projection of canonical
 /// signed records. No inference/acceptance, mutation, staging promotion, grant
 /// renewal or second copy. Cursor bounds scanning even for sparse inboxes.
 pub(in crate::pet_runtime)fn message_inbox(&self,pet:&str,after:Option<&str>,limit:usize,now:u64)->Result<crate::pet_runtime::messages::Inbox>{
  self.message_inbox_view(pet,after,limit,now,false)
 }
 pub(in crate::pet_runtime)fn message_inbox_view(&self,pet:&str,after:Option<&str>,limit:usize,now:u64,include_historical:bool)->Result<crate::pet_runtime::messages::Inbox>{
  self.message_inbox_reply_view(pet,after,limit,now,include_historical,false)
 }
 pub(in crate::pet_runtime)fn message_inbox_reply_view(&self,pet:&str,after:Option<&str>,limit:usize,now:u64,include_historical:bool,include_reply_sources:bool)->Result<crate::pet_runtime::messages::Inbox>{
  self.message_inbox_funding_view(pet,after,limit,now,include_historical,include_reply_sources,false)
 }
 pub(in crate::pet_runtime)fn message_inbox_funding_view(&self,pet:&str,after:Option<&str>,limit:usize,now:u64,include_historical:bool,include_reply_sources:bool,include_funding_sources:bool)->Result<crate::pet_runtime::messages::Inbox>{
  use crate::pet_runtime::messages::{Inbox,InboxItem};
  if crate::pet_config::blueprint(pet).is_none()||!(1..=32).contains(&limit)||after.is_some_and(|s|!model::token(s))||now==0||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
  let _lock=self.lock_shared()?;let mut names=self.dir.names().map_err(|_|Failure::Unavailable)?;names.sort();let mut items=vec![];let mut unavailable=0;let mut pending_artifacts=0;let mut next=None;let mut scanned=0;
  for name in &names{
   if matches!(name.as_str(),".key.json"|".protocol.json"|".catalog.lock"){continue;}
   let historical=self.historical_file(name);if historical&&(!include_historical||!name.strip_suffix(".json").is_some_and(model::token)){continue;}
   match self.rule_record_data(name){Ok(Some(_))=>continue,Err(_)=>{unavailable+=1;continue;},Ok(None)=>{}}
   if name.strip_prefix(".grant-").is_some_and(model::token){pending_artifacts+=1;continue;}
   let Some(id)=name.strip_suffix(".json").filter(|s|model::token(s))else{unavailable+=1;continue;};if after.is_some_and(|a|id<=a){continue;}
   if scanned==128{break;}scanned+=1;next=Some(id.to_owned());
   let observed=if historical{super::restoration::message_snapshot_file(self,name).map(|(body,source)|(body,Some(source)))}else{self.grant_file(name).map(|(body,_)|(body,None))};
   let(body,historical_source)=match observed{Ok((b,s))if b.decision_id==id=>(b,s),_=>{unavailable+=1;continue;}};
   if let Some(messages)=body.messages{for receipt in messages.entries{let e=receipt.envelope();if e.recipient.pet_id==pet{let available=historical_source.is_none()&&now>=e.created_at&&now<e.deadline_at;
    let reply_source=if include_reply_sources&&available&&matches!(e.schema_version,1|10|11|13|15){Some(crate::pet_runtime::messages::ReceivedReference::from_receipt(&receipt).map_err(|_|Failure::Invalid)?)}else{None};
    let funding_source=if include_funding_sources&&available&&matches!(e.schema_version,4|7|12|14|16){Some(crate::pet_runtime::messages::ReceivedReference::from_receipt(&receipt).map_err(|_|Failure::Invalid)?)}else{None};
    let result_source=if include_funding_sources&&available&&matches!(e.schema_version,6|9){let reference=crate::pet_runtime::messages::ReceivedReference::from_receipt(&receipt).map_err(|_|Failure::Invalid)?;self.received_result_policy_unlocked(&reference,pet,now).ok().map(|_|reference)}else{None};
    items.push(InboxItem{result_source,funding_source,reply_source,available,receipt,historical_source:historical_source.clone()});}}}
   // A complete grant is one page unit (<=16 messages), never skip the tail.
   if items.len()>=limit{break;}
  }
  let has_more=next.as_ref().is_some_and(|last|names.iter().any(|n|n.strip_suffix(".json").is_some_and(|id|model::token(id)&&id>last.as_str()&&(include_historical||!self.historical_file(n)))));
  let mut after_names=self.dir.names().map_err(|_|Failure::Unavailable)?;after_names.sort();if after_names!=names{return Err(Failure::Changed);}self.fence()?;Ok(Inbox{schema_version:if include_reply_sources&&items.iter().any(|i|matches!(i.receipt.envelope().schema_version,10|11)){8}else if items.iter().any(|i|i.result_source.is_some()&&i.receipt.envelope().schema_version==9){7}else if items.iter().any(|i|i.funding_source.is_some()&&i.receipt.envelope().schema_version==7){6}else if items.iter().any(|i|i.result_source.is_some()){5}else if include_funding_sources{4}else if include_reply_sources{3}else if include_historical{2}else{1},pet_id:pet.into(),items,unavailable,pending_artifacts,scan_complete:after.is_none()&&!has_more&&unavailable==0&&pending_artifacts==0,has_more,next_grant:has_more.then_some(next).flatten(),execution_authorized:false})
 }
}
