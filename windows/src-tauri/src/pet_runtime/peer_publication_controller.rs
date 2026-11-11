//! Publication provenance can survive a lost Driver object; a model Approved
//! cannot. Every pending delivery still uses the original exact decision broker.
use super::*;
impl PeerRoundPublished{
 pub(in crate::pet_runtime)fn goal(&self)->&Record{&self.goal}
 pub(in crate::pet_runtime)fn reconciled(&self)->bool{matches!(self.lineage,PeerPublicationLineage::Reconciled(_))}
 pub(in crate::pet_runtime)fn retains_native_data_approval(&self)->bool{!self.reconciled()}
 fn data(&self)->&crate::pet_runtime::peer_round_data::Consent{match &self.lineage{PeerPublicationLineage::Retained(a)=>a.reference(),PeerPublicationLineage::Reconciled(p)=>p.data()}}
 pub(in crate::pet_runtime)fn expires_at(&self)->u64{match &self.lineage{PeerPublicationLineage::Retained(a)=>a.reference().expires_at,PeerPublicationLineage::Reconciled(p)=>p.expires_at()}}
 fn original_deadline_current(&self)->bool{self.owner.live.lease.deadline_at.checked_sub(self.expires_at()).and_then(|delta|self.owner.live.deadline.checked_sub(Duration::from_millis(delta))).is_some_and(|deadline|Instant::now()<deadline)}
 pub(in crate::pet_runtime)fn not_revoked(&self)->bool{
  !self.revoked.load(Ordering::Acquire)&&!self.owner.live.closed.load(Ordering::Acquire)&&!self.owner.live.hold.stopped()&&match &self.lineage{PeerPublicationLineage::Retained(a)=>a.publication_not_revoked(),PeerPublicationLineage::Reconciled(p)=>p.pending(crate::assistant_memory::now())&&self.original_deadline_current()}
 }
 pub(in crate::pet_runtime)fn matches_delivery(&self,scope:&crate::pet_runtime::messages::Scope)->bool{self.not_revoked()&&self.current()==Validity::Current&&scope.generated_plan.as_ref().is_some_and(|r|self.goal.generated_plans.last()==Some(r))&&scope.envelope.planned_data.as_ref().is_some_and(|p|p.round_data.as_ref()==Some(self.data()))}
 fn latch(&self,result:Validity)->Validity{if result==Validity::Changed{self.revoked.store(true,Ordering::Release);if let PeerPublicationLineage::Retained(a)=&self.lineage{a.revoke_publication();}}result}
 pub(in crate::pet_runtime)fn current(&self)->Validity{if self.revoked.load(Ordering::Acquire){return Validity::Changed;}self.latch(self.observe_current(true))}
 pub(in crate::pet_runtime)fn metadata_current(&self)->Validity{if !self.not_revoked(){return self.latch(Validity::Changed);}self.latch(self.observe_current(false))}
 pub(in crate::pet_runtime)fn belongs_round(&self,owner:&Owned,request:&str)->bool{self.owner.same_live(owner)&&self.request_id==request}
 fn observe_current(&self,held:bool)->Validity{
  let target=match self.store.read(&self.goal.id,&self.goal.spec.owner.pet_id){Ok(g)=>g,Err(model::Error::Busy)=>return Validity::Busy,Err(model::Error::StorageUnavailable)=>return Validity::Unknown,Err(_)=>return Validity::Changed};
  let authority=match Authority::existing(&self.owner.live.authority_path){Ok(a)=>a,Err(_)=>return Validity::Unknown};let now=crate::assistant_memory::now();
  match &self.lineage{
   PeerPublicationLineage::Retained(a)=>if held{a.published_validity(&authority,&self.owner.live.spec,&self.owner.live.proof,&target,&self.request_id,now)}else{a.published_metadata_validity(&authority,&self.owner.live.spec,&self.owner.live.proof,&target,&self.request_id,now)},
   PeerPublicationLineage::Reconciled(p)=>{if !self.not_revoked(){return Validity::Changed;}match p.verify(&authority,&self.owner.live.spec,&self.owner.live.proof,&self.owner.live.lease,&target,now){Ok(())=>{},Err(crate::pet_runtime::authority::Failure::Busy)=>return Validity::Busy,Err(crate::pet_runtime::authority::Failure::Unavailable|crate::pet_runtime::authority::Failure::Uncertain)=>return Validity::Unknown,Err(_)=>return Validity::Changed}if held{self.owner.pending_validity(now)}else{Validity::Current}},
  }
 }
}
impl Owned{
 pub(in crate::pet_runtime)fn reconcile_peer_round_plan(self:&Arc<Self>,store:Arc<Store>,request:&str,now:u64)->model::Result<Arc<PeerRoundPublished>>{
  if !self.live.spec.peer_iteration()||self.workers()!=0{return Err(model::Error::Conflict);}let goal=store.read_control(&self.live.spec.goal_id,&self.live.spec.owner.pet_id)?;self.message_goal(&goal,now)?;
  let authority=Authority::existing(&self.live.authority_path).map_err(crate::pet_runtime::authority::Failure::control_error)?;let journal=authority.planning_ledger(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  let round=journal.rounds.last().filter(|r|r.request_id==request&&r.peer_data.is_some()).ok_or(model::Error::Conflict)?;let p=round.publication.as_ref().ok_or(model::Error::Conflict)?;
  // ACK-only: never use persisted DATA to certify a fresh, absent plan write.
  if !goal.generated_plans.contains(&p.reference)||crate::pet_runtime::planned_peer::reference(&goal,&self.live.spec)?!=&p.reference{return Err(model::Error::Conflict);}
  if p.phase!=crate::pet_runtime::generated_plans::Phase::Acknowledged{let ack=self.publish_generated_plan(&store,request,now)?;if !ack.acknowledged||ack.receipt.status==model::CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}}
  let goal=store.read_control(&self.live.spec.goal_id,&self.live.spec.owner.pet_id)?;
  let proof=authority.reconciled_peer_publication(&self.live.spec,&self.live.proof,&self.live.lease,&goal,request,crate::assistant_memory::now()).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  let scope=Arc::new(PeerRoundPublished{goal,request_id:request.into(),lineage:PeerPublicationLineage::Reconciled(proof),owner:self.clone(),store,revoked:AtomicBool::new(false)});let validity=scope.current();
  match validity{Validity::Current=>Ok(scope),Validity::Busy=>Err(model::Error::Busy),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>Err(model::Error::Conflict)}
 }
}
