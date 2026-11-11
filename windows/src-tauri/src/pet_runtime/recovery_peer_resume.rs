// Same native recovery cache and window Frame, specialized for owner handoff.
// No serialized token, snapshot or ACK can manufacture this retained owner.
use crate::pet_runtime::{lease_controller::{peer_resume_controller as peer,Owned},leases::Lease,authority::{Authority,Failure}};
use tauri::Manager;
enum PeerState{
 Review(Box<peer::Review>),
 Approved{approval:Box<peer::Approved>,owner:Option<Arc<Owned>>,registered:bool},
 Acknowledged(Lease),Ended,
}
impl PeerState{
 fn keep(&mut self,p:&Pending,now:u64)->bool{match self{
  Self::Review(_)=>p.origin.frame.current()&&now<p.ticket.proposal.expires_at,
  Self::Approved{approval,owner,registered}=>{
   if !*registered&&approval.prepared()&&approval.write_started&&matches!(approval.published(),Ok(true)){
    if let Ok(goal)=approval.source.store.read_control(&approval.goal.id,&approval.goal.spec.owner.pet_id){if let Ok(pending)=approval.pending(){if crate::pet_runtime::lease_api::register_resumption(&goal,pending,owner.clone()).is_ok(){*registered=true;}}}
   }
   if now>=approval.source.snapshot.original_budget.deadline_at.saturating_add(30_000)&&*registered{return false;}
   if *registered{return true;}
   if !p.origin.frame.current()||now>=p.ticket.proposal.expires_at||approval.prepared()&&approval.current_native(now).is_err(){
    if matches!(approval.published(),Ok(false))&&matches!(approval.withdraw(),Ok(true)){return false;}
   }true
  },Self::Acknowledged(_)=>p.origin.frame.current()&&now<p.ticket.proposal.expires_at.saturating_add(300_000),Self::Ended=>false,
 }}
 fn cleanup_revision(&self,revision:&str)->bool{match self{Self::Approved{approval,..}=>revision==approval.goal.revision||approval.candidate.as_ref().is_some_and(|g|g.revision==revision),_=>false}}
 fn cleanup_allowed(&self)->model::Result<bool>{match self{Self::Approved{approval,..}=>approval.published(),Self::Acknowledged(_)=>Ok(true),_=>Ok(false)}}
 fn approve_choice(&mut self,p:&Pending,now:u64)->model::Result<()>{
  if !matches!(self,Self::Review(_)){return Ok(());}
  if p.kind!=Kind::PeerResume||!p.origin.frame.current(){return Err(Error::Conflict);}
  let Self::Review(review)=self else{return Err(Error::Conflict);};review.current(now)?;
  let Self::Review(review)=std::mem::replace(self,Self::Ended)else{return Err(Error::Conflict);};let frame=p.origin.frame.clone();
  *self=Self::Approved{approval:Box::new(review.approve(now,Arc::new(move||frame.current()))?),owner:None,registered:false};Ok(())
 }
 fn decline(&mut self)->model::Result<RecoveryConfirmed>{
  if let Self::Approved{approval,..}=self{if !approval.withdraw()?{return Ok(RecoveryConfirmed::Blocked{reason:Error::Conflict,retryable:false,retained:true});}}
  if matches!(self,Self::Acknowledged(_)){return Err(Error::Conflict);}*self=Self::Ended;Ok(RecoveryConfirmed::Declined)
 }
}
impl Pending{
 fn confirm_peer(&self,app:&tauri::AppHandle,store:&Store,approve:bool,now:u64)->model::Result<RecoveryConfirmed>{
  let mut native=self.state.try_lock().map_err(|_|Error::Busy)?;let State::Peer(peer)=&mut*native else{return Err(Error::Conflict);};
  if !approve{return peer.decline();}
  if let Err(reason)=peer.approve_choice(self,now){return Ok(RecoveryConfirmed::Blocked{reason,retryable:retryable(reason),retained:!matches!(**peer,PeerState::Ended)});}
  match &mut**peer{
   PeerState::Approved{approval,owner,registered}=>{
    let result=(||->model::Result<model::Receipt>{
     if !approval.prepared(){
      let evidence=crate::pet_runtime::admission_review::owner_resumption_evidence(app,&approval.goal,now)?.map_err(|_|Error::Conflict)?;
      approval.prepare_worker(app.state::<crate::chat::Chat>().inner(),&crate::settings::config_dir().join("pet-runtime-contexts-v1"),&evidence,crate::pet_runtime::lease_api::fence_for_app(app,&approval.goal),now)?;
     }
     let(receipt,native_owner)=approval.confirm(now)?;if let Some(native_owner)=native_owner{*owner=Some(Arc::new(native_owner));}Ok(receipt)
    })();
    // Even uncertain Goal ACK must retain its actual native Pending in the
    // common service, so Stop/deadline cleanup does not depend on this window.
    let registration=if approval.prepared()&&approval.published()?{
     let goal=store.read_control(&approval.goal.id,&approval.goal.spec.owner.pet_id)?;
     crate::pet_runtime::lease_api::register_resumption(&goal,approval.pending()?,owner.clone()).map(|_|{*registered=true;})
    }else{Ok(())};
    if let Err(reason)=registration{return Ok(RecoveryConfirmed::Blocked{reason,retryable:retryable(reason),retained:true});}
    match result{
     Ok(receipt)if receipt.status==model::CommitStatus::Uncertain=>Ok(RecoveryConfirmed::AcknowledgementPending{receipt}),
     Ok(receipt)=>{let lease=owner.as_ref().ok_or(Error::Conflict)?.lease().clone();*peer=Box::new(PeerState::Acknowledged(lease.clone()));Ok(RecoveryConfirmed::Resumed{receipt,lease,owned:true,execution_available:false})},
     Err(reason)=>Ok(RecoveryConfirmed::Blocked{reason,retryable:retryable(reason),retained:true}),
    }
   },PeerState::Acknowledged(lease)=>{
    let goal=store.read_control(&self.ticket.proposal.goal_id,&self.ticket.proposal.pet_id)?;
    if goal.admissions.last().and_then(|r|r.lease.as_ref())!=Some(lease){return Err(Error::Conflict);}
    let owned=crate::pet_runtime::lease_api::step_owner(&goal,&lease.id).is_ok();Ok(RecoveryConfirmed::Resumed{receipt:model::Receipt{record:goal,status:model::CommitStatus::Unchanged},lease:lease.clone(),owned,execution_available:false})
   },_=>Err(Error::Conflict),
  }
 }
}
impl Reviews{
 fn find_peer(&mut self,origin:&Origin,input:&RecoveryPrepareInput,now:u64)->model::Result<Option<RecoveryTicket>>{
  if !origin.frame.current(){return Err(Error::Conflict);}self.sweep(now);let mut rebind=None;
  for pending in self.pending.values(){let p=&pending.ticket.proposal;
   if !matches!(pending.kind,Kind::PeerResume|Kind::PeerCleanup)||p.goal_id!=input.goal_id||p.pet_id!=input.pet_id||pending.origin.window!=origin.window{continue;}
   let state=pending.state.try_lock().map_err(|_|Error::Busy)?;let State::Peer(peer)=&*state else{return Err(Error::Conflict);};
   if p.revision!=input.revision&&!peer.cleanup_revision(&input.revision){return Err(Error::Conflict);}
   let cleanup=peer.cleanup_allowed()?;let mut ticket=pending.ticket.clone();if cleanup{ticket.proposal.approval_scope="ackRecordedPeerResume";}
   if pending.origin.same(origin){if cleanup||peer.cleanup_revision(&input.revision)||pending.request==input.request_id{return Ok(Some(ticket));}return Err(Error::Conflict);}
   if pending.origin.frame.current()||!cleanup{return Err(Error::Conflict);}ticket.token=crate::pet_runtime::store::nonce()?;rebind=Some((ticket,pending.request.clone(),pending.state.clone()));break;
  }
  if let Some((ticket,request,state))=rebind{
   if !origin.frame.current(){return Err(Error::Conflict);}self.pending.retain(|_,p|!Arc::ptr_eq(&p.state,&state));
   self.pending.insert(ticket.token.clone(),Arc::new(Pending{origin:origin.clone(),request,ticket:ticket.clone(),kind:Kind::PeerCleanup,state}));return Ok(Some(ticket));
  }Ok(None)
 }
 fn insert_peer(&mut self,origin:&Origin,review:peer::Review,now:u64)->model::Result<RecoveryTicket>{
  self.sweep(now);let p=review.proposal();let goal=review.goal();
  if !origin.frame.current()||now>=p.expires_at{return Err(Error::Conflict);}if self.pending.len()>=64{return Err(Error::LimitReached);}
  let authority=Authority::existing(&review.authority_path()).map_err(Failure::control_error)?;
  let snapshot=authority.reconciliation(goal,now).map_err(Failure::control_error)?;
  let proposal=crate::pet_runtime::recovery_controller::Proposal{goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),revision:goal.revision.clone(),request_id:p.request_id.clone(),intent:goal.spec.intent.clone(),criteria:goal.spec.criteria.clone(),provider:goal.spec.owner.provider.clone(),lease_id:p.snapshot.lease_id.clone(),approval_scope:p.approval_scope,resulting_state:model::State::Leased,snapshot:serde_json::to_value(snapshot).map_err(|_|Error::InvalidRecord)?,peer_resumption:Some(p.snapshot),prepared_at:p.prepared_at,expires_at:p.expires_at,execution_available:false};
  let ticket=RecoveryTicket{token:crate::pet_runtime::store::nonce()?,proposal};self.pending.insert(ticket.token.clone(),Arc::new(Pending{origin:origin.clone(),request:p.request_id,ticket:ticket.clone(),kind:Kind::PeerResume,state:Arc::new(Mutex::new(State::Peer(Box::new(PeerState::Review(Box::new(review))))))}));Ok(ticket)
 }
}
pub(super)fn prepare_peer(origin:&Origin,input:RecoveryPrepareInput)->model::Result<RecoveryPrepared>{
 if !model::key(&input.request_id)||!model::token(&input.revision){return Err(Error::InvalidInput);}let now=crate::assistant_memory::now();
 if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.find_peer(origin,&input,now)?{return Ok(RecoveryPrepared::Review{ticket});}
 let root=crate::settings::config_dir();let path=root.join("pet-runtime-consent-v1");let store=Arc::new(Store::existing_native(&root.join("pet-runtime-v1"))?.ok_or(Error::NotFound)?);
 let result=(||->model::Result<RecoveryPrepared>{
  if let Some(receipt)=store.ack_peer_owner_resumption(&path,&input.goal_id,&input.pet_id,&input.revision)?{return Ok(RecoveryPrepared::Acknowledged{receipt,execution_available:false});}
  store.retire_lost_peer_owner_preparations(&path,&input.goal_id,&input.pet_id,&input.revision)?;
  let source=store.prepare_peer_planning_continuation(&path,&input.goal_id,&input.pet_id,&input.revision,now)?;let review=peer::Review::prepare(source,input.request_id,now)?;
  reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert_peer(origin,review,now).map(|ticket|RecoveryPrepared::Review{ticket})
 })();match result{Ok(reply)=>Ok(reply),Err(reason)=>Ok(RecoveryPrepared::Blocked{reason,retryable:retryable(reason)})}
}
pub(super)fn sweep_peers(){if let Ok(mut cache)=reviews().try_lock(){cache.sweep(crate::assistant_memory::now());}}

#[cfg(test)]pub(super)fn fixture_peer_cache_scope(review:peer::Review,now:u64){
 let goal=review.goal().clone();let origin=Origin{window:crate::HUB_LABEL.into(),frame:crate::settings::review::Frame::fixture()};let mut cache=Reviews::default();
 let ticket=cache.insert_peer(&origin,review,now).unwrap();assert!(ticket.proposal.peer_resumption.as_ref().is_some_and(|p|!p.additional_allowance&&!p.model_use_authorized));
 let input=RecoveryPrepareInput{pet_id:goal.spec.owner.pet_id.clone(),goal_id:goal.id.clone(),revision:goal.revision.clone(),request_id:ticket.proposal.request_id.clone()};
 assert!(cache.find(&origin,&input,now).unwrap().is_none());assert_eq!(cache.find_peer(&origin,&input,now).unwrap().unwrap().token,ticket.token);
 let mut choice=RecoveryConfirmInput{pet_id:input.pet_id.clone(),goal_id:input.goal_id.clone(),revision:input.revision.clone(),token:ticket.token.clone(),approve:true};
 let foreign=Origin{window:origin.window.clone(),frame:crate::settings::review::Frame::fixture()};assert!(cache.lookup(&foreign,&choice).is_err());choice.pet_id="agent_codex".into();assert!(cache.lookup(&origin,&choice).is_err());choice.pet_id=input.pet_id;choice.revision="a".repeat(32);assert!(cache.lookup(&origin,&choice).is_err());choice.revision=input.revision;
 let pending=cache.lookup(&origin,&choice).unwrap();origin.frame.invalidate_for_test();
 {let mut state=pending.state.lock().unwrap();let State::Peer(peer)=&mut*state else{panic!("native peer state");};assert!(peer.approve_choice(&pending,now).is_err());assert!(matches!(**peer,PeerState::Review(_)));}
 drop(pending);cache.sweep(now);assert!(cache.pending.is_empty());
}
