//! Native peer steps share the original Driver, authority, receipts and drain.
//! No inference; every envelope is derived only from the signed human DATA review.
use super::*;
impl Driver{
 pub(super)async fn drive_native_peer_plan(&self)->model::Result<()>{
  let policy=self.owner.live.spec.native_peer_plan.as_ref().ok_or(model::Error::Conflict)?;
  let authority=self.authority()?;
  for step in &policy.steps{
   // Check durable receipts first. A lost ACK never creates a new envelope.
   if authority.message_at(&self.owner.live.spec,&self.owner.live.proof,&step.cause.plan_id,&step.cause.step_id).map_err(crate::pet_runtime::authority::Failure::control_error)?.is_some(){
    self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work=None;continue;
   }
   if !self.current().await?{return self.stopped(0).await;}
   let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work.clone();
   let retained=if let Some(r)=retained{
    if r.approved.scope().envelope.cause!=step.cause{return Err(model::Error::Conflict);}r
   }else{
    let goal=self.store.read_control(&policy.goal_id,&policy.owner.pet_id)?;
    let(config,settings)=self.environment.peer_configuration()?;
    let target=Arc::new(crate::pet_runtime::messages::Target::capture(&config,&settings,&step.recipient.pet_id)?);
    let review=self.owner.review_message(&goal,&step.cause.plan_id,&step.cause.step_id,format!("native-peer-{}",model::fingerprint(&step.cause)?),target,crate::assistant_memory::now())?;
    let approved=review.approve_native_peer_plan(&self.owner,&goal,&config,&settings,crate::assistant_memory::now())?;
    let r=Arc::new(WorkRetained{goal,approved});self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work=Some(r.clone());r
   };
   self.notice(Phase::Executing,0,Some("A entregar o passo e os dados revistos; sem inferência ou autoridade adicional no destinatário."))?;
   let until=Instant::now()+Duration::from_secs(2);
   loop{match self.owner.send_message(&retained.goal,&retained.approved,crate::assistant_memory::now(),||self.environment.peer_configuration()){
    Ok(_)=>break,Err(model::Error::Busy)if Instant::now()<until=>{if !self.current().await?{return self.stopped(0).await;}tokio::time::sleep(Duration::from_millis(25)).await;},Err(e)=>return Err(e),
   }}
   self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.work=None;
  }
  self.finish_native_peer_plan(0).await
 }
 pub(super)async fn finish_native_peer_plan(&self,round:u8)->model::Result<()>{
  self.notice(Phase::Feedback,round,Some("A verificar os recibos originais e a confirmar a conclusão, sem reenviar mensagens."))?;
  if self.owner.live.spec.native_verification.is_some()||self.owner.live.spec.peer_planning(){
   let until=Instant::now()+Duration::from_secs(2);
   let progress=loop{
    let attempt=(||{let goal=self.store.read_control(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;self.owner.verify_native_goal(&self.store,&goal)})();
    match attempt{Ok(p)=>break p,Err(model::Error::Busy)if Instant::now()<until=>{
     // The native completion controller retains its exact immutable report.
     // Retry only this known lock conflict, never message effects/unknown IO.
     if self.owner.live.hold.stopped(){return self.stopped(round).await;}tokio::time::sleep(Duration::from_millis(25)).await;
    },Err(e)=>return Err(e)}
   };
   if let Some(receipt)=progress.completion{if receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}return self.close_completed(&receipt.record,round).await;}
  }
  self.finish(Phase::AwaitingVerification,round,Some("Entregas locais guardadas. Critérios sem prova nativa continuam por verificar; nenhuma execução do destinatário foi presumida.")).await
 }
}
