//! Explicit manual start of a native plan reviews the exact sequence once.
//! Old queued manual work is never promoted; no effect precedes the decision.
use super::*;
impl Driver{
 pub(super)async fn drive_peer_plan_review(&self,round:u8)->model::Result<()>{
  let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.peer.clone();
  let retained=if let Some(r)=retained{r}else{
   if !self.current().await?{return self.stopped(round).await;}
   let goal=self.store.read_control(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;
   let(config,settings)=self.environment.peer_configuration()?;
   let lineage=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.data_publication.clone();
   let batch=match crate::pet_runtime::peer_plan_review::Batch::capture_with_round(&self.owner,&goal,&config,&settings,crate::assistant_memory::now(),lineage){Ok(batch)=>batch,Err(model::Error::LimitReached)if self.owner.live.spec.peer_iteration()=>return self.finish(Phase::Limited,round,Some("A sequência excede o saldo ou os limites restantes; sem nova decisão, reserva ou envio. Encargos anteriores conservados.")).await,Err(e)=>return Err(e)};
   let Some(batch)=batch else{return if self.owner.live.spec.peer_iteration(){Ok(())}else{self.finish_native_peer_plan(round).await};};
   let environment=self.environment.clone();let observe:crate::pet_runtime::peer_plan_review::Observe=Arc::new(move||environment.peer_configuration());
   let source=Source::capture_peer(self.owner.clone(),&goal,batch.clone(),self.store.clone(),observe)?;
   let r=Arc::new(PeerAwaiting{goal,batch,source,decision:Mutex::new(None),approved:Mutex::new(None)});
   self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.peer=Some(r.clone());r
  };
  if retained.approved.lock().map_err(|_|model::Error::StorageUnavailable)?.is_none(){
   self.notice(Phase::AwaitingApproval,round,Some("Revê os dados, fontes, destinatários e prazos da sequência. A decisão não autoriza execução ou uso de modelos pelos destinatários."))?;
   if retained.decision.lock().map_err(|_|model::Error::StorageUnavailable)?.is_none(){
    if self.owner.live.spec.peer_planning(){self.owner.yield_for_decision(&self.store,&retained.source,crate::assistant_memory::now())?;}
    let Some(choice)=self.environment.decide(retained.source.clone())else{
     if self.owner.live.closed.load(Ordering::Acquire)||self.owner.live.hold.stopped()||self.owner.pending_validity(crate::assistant_memory::now())==Validity::Changed{return self.stopped(round).await;}
     return self.finish(Phase::ApprovalEnded,round,Some("O pedido original de revisão terminou; nenhuma entrega nova foi iniciada.")).await;
    };
    if !choice.approved(){return self.finish(Phase::Declined,round,Some("Sequência recusada; os recibos anteriores conservam-se, sem novos envios.")).await;}
    *retained.decision.lock().map_err(|_|model::Error::StorageUnavailable)?=Some(choice);
   }
   if self.owner.live.spec.peer_planning(){loop{match self.owner.resume_after_decision(&self.store,&retained.source,crate::assistant_memory::now()){
    Ok(())=>break,Err(model::Error::Busy)=>{self.notice(Phase::WaitingResource,round,Some("A aguardar o recurso original; sem nova inferência ou consentimento."))?;tokio::time::sleep(Duration::from_millis(50)).await;},Err(e)=>return Err(e),
   }}}
   let environment=self.environment.clone();let observe:crate::pet_runtime::peer_plan_review::Observe=Arc::new(move||environment.peer_configuration());
   let approved={let decision=retained.decision.lock().map_err(|_|model::Error::StorageUnavailable)?;retained.batch.approve(decision.as_ref().ok_or(model::Error::Conflict)?,&self.owner,&retained.goal,&observe,crate::assistant_memory::now())?};
   *retained.approved.lock().map_err(|_|model::Error::StorageUnavailable)?=Some(approved);
   retained.decision.lock().map_err(|_|model::Error::StorageUnavailable)?.take();
  }
  let approved=retained.approved.lock().map_err(|_|model::Error::StorageUnavailable)?.as_ref().cloned().ok_or(model::Error::Conflict)?;
  for approval in approved{
   // send_message itself observes the exact canonical ACK first. Retain every
   // original envelope on unknown IO; never generate another message to retry.
   if !self.current().await?{return self.stopped(round).await;}
   self.notice(Phase::Executing,round,Some("A entregar apenas a sequência nativa explicitamente revista."))?;
   let until=Instant::now()+Duration::from_secs(2);
   loop{match self.owner.send_message(&retained.goal,&approval,crate::assistant_memory::now(),||self.environment.peer_configuration()){
    Ok(_)=>break,Err(model::Error::Busy)if Instant::now()<until=>{if !self.current().await?{return self.stopped(round).await;}tokio::time::sleep(Duration::from_millis(25)).await;},Err(e)=>return Err(e),
   }}
  }
  if self.owner.live.spec.peer_iteration(){self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.peer=None;Ok(())}else{self.finish_native_peer_plan(round).await}
 }
}
