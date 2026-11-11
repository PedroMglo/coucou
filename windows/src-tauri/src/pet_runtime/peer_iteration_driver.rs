//! Same Driver/owner/journal. An internal wait releases physical inference;
//! a second round still needs a distinct exact native DATA/body decision.
use super::*;
use crate::pet_runtime::{peer_round_data,peer_round_review,delegation_api::DelegationInput};
impl Driver{
 /// Wait for the actual branches through the original resource lease. This
 /// observes metadata only, never authorizes its use by a model or another turn.
 async fn await_derived_children(&self,round:u8)->model::Result<bool>{
  let owner=&self.owner;let spec=&owner.live.spec;let goal=self.store.read_control(&spec.goal_id,&spec.owner.pet_id)?;let reference=crate::pet_runtime::planned_peer::reference(&goal,spec)?;
  let plan=goal.plans.iter().find(|p|p.id==reference.plan_id).ok_or(model::Error::Conflict)?;
  if !plan.steps.iter().any(|s|s.operation.as_ref().is_some_and(|op|op.operation_id=="pet.delegate")){return Ok(true);}
  let environment=self.environment.clone();let wait=Source::capture_peer_wait(owner.clone(),&goal,self.store.clone(),Arc::new(move||environment.configuration()))?;
  owner.yield_for_decision(&self.store,&wait,crate::assistant_memory::now())?;
  self.notice(Phase::WaitingPeers,round,Some("A aguardar os ramos derivados admitidos com saldo próprio; recurso de inferência cedido e sem nova chamada ao modelo."))?;
  let capture=self.environment.capture(owner)?;let input=DelegationInput{pet_id:spec.owner.pet_id.clone(),goal_id:spec.goal_id.clone(),decision_id:spec.decision_id.clone()};
  loop{
   if !wait.pending(){self.stopped(round).await?;return Ok(false);}
   match peer_round_data::Captured::at(self.store.root_path(),&owner.live.authority_path,input.clone(),&capture.server,&owner.lease().model,crate::assistant_memory::now()){
    Ok(data)if data.outcome_ready()=>break,Ok(_)=>{},Err(model::Error::Busy|model::Error::StorageUnavailable)=>{},Err(e)=>return Err(e),
   }tokio::time::sleep(Duration::from_secs(1)).await;
  }
  self.resume_peer_wait(&wait,round).await?;wait.retire();Ok(true)
 }
 async fn finish_derived_plan(&self,round:u8)->model::Result<()>{if !self.await_derived_children(round).await?{return Ok(());}self.finish_native_peer_plan(round).await}

 async fn resume_peer_wait(&self,source:&Source,round:u8)->model::Result<()>{
  loop{match self.owner.resume_after_decision(&self.store,source,crate::assistant_memory::now()){
   Ok(())=>return Ok(()),Err(model::Error::Busy)=>{if self.owner.pending_validity(crate::assistant_memory::now())==Validity::Changed{return Err(model::Error::Conflict);}self.notice(Phase::WaitingResource,round,Some("A aguardar o recurso original; o prazo e saldo não se renovam."))?;tokio::time::sleep(Duration::from_millis(50)).await;},Err(e)=>return Err(e),
  }}
 }
 pub(super)async fn drive_peer_iteration(&self)->model::Result<()>{
  let owner=&self.owner;let spec=&owner.live.spec;let authority=self.authority()?;
  loop{
   if !self.current().await?{return self.stopped(0).await;}
   let journal=authority.planning_ledger(spec,&owner.live.proof,owner.lease(),crate::assistant_memory::now()).map_err(crate::pet_runtime::authority::Failure::control_error)?;let round=journal.rounds.len()as u8;
   let goal=self.store.read_control(&spec.goal_id,&spec.owner.pet_id)?;
   if let Some(last)=journal.rounds.last(){
    // Re-entering after an incomplete model/ACK never replays an effect.
    let Some(output)=&last.output else{return self.notice(Phase::RecoveryRequired,round,Some("A tentativa original não tem proposta final confirmada; reconciliar sem repetir inferência."));};
    if last.peer_data.is_some()&&output.kind==OutputKind::Plan{
     let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.data_publication.clone();
     if let Some(scope)=retained{if !scope.belongs_round(owner,&last.request_id){return Err(model::Error::Conflict);}}else{
      self.notice(Phase::Publishing,round,Some("A reconciliar apenas a publicação canónica existente; a evidência recuperada não é uma aprovação de modelo ou de efeitos pendentes."))?;
      let scope=owner.reconcile_peer_round_plan(self.store.clone(),&last.request_id,crate::assistant_memory::now())?;self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.data_publication=Some(scope);
     }
    }
    if last.peer_data.is_some()&&output.kind!=OutputKind::Plan{return if matches!(output.kind,OutputKind::Blocked|OutputKind::Invalid){self.finish(Phase::Limited,round,Some("Ronda DATA concluída sem um plano executável; os critérios continuam por verificar.")).await}else{self.notice(Phase::RecoveryRequired,round,Some("Plano e recibos conservados; a aprovação nativa original não está retida. Reconciliar o estado e rever a retoma sem recriar autoridade de JSON ou repetir inferência."))};}
    if output.kind!=OutputKind::Plan{return self.finish(Phase::Limited,round,Some("Sem plano peer executável; não repetir a mesma informação.")).await;}
    if last.peer_data.is_none()&&round!=1{return Err(model::Error::Conflict);}
    if last.publication.as_ref().is_none_or(|p|p.phase!=crate::pet_runtime::generated_plans::Phase::Acknowledged){
     self.notice(Phase::Publishing,round,None)?;let p=owner.publish_generated_plan(&self.store,&last.request_id,crate::assistant_memory::now())?;if !p.acknowledged||p.receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}continue;
    }
    self.drive_peer_plan_review(round).await?;
    if matches!(self.status()?.phase,Phase::Stopped|Phase::Declined|Phase::ApprovalEnded|Phase::Closing|Phase::Limited){return Ok(());}
    if round>=spec.planning_policy.as_ref().ok_or(model::Error::Conflict)?.limits.max_rounds{return self.finish_derived_plan(round).await;}
    let goal=self.store.read_control(&goal.id,&goal.spec.owner.pet_id)?;
    // This source is deliberately independent of a changing child outcome.
    let environment=self.environment.clone();let wait=Source::capture_peer_wait(owner.clone(),&goal,self.store.clone(),Arc::new(move||environment.configuration()))?;
    owner.yield_for_decision(&self.store,&wait,crate::assistant_memory::now())?;
    self.notice(Phase::WaitingPeers,round,Some("A aguardar resultados dos ramos autorizados; recurso físico cedido, sem inferência contínua."))?;
    let capture=self.environment.capture(owner)?;
    let input=DelegationInput{pet_id:spec.owner.pet_id.clone(),goal_id:spec.goal_id.clone(),decision_id:spec.decision_id.clone()};
    let data=loop{
     if !wait.pending(){return self.stopped(round).await;}
     match peer_round_data::Captured::at(self.store.root_path(),&owner.live.authority_path,input.clone(),&capture.server,&owner.lease().model,crate::assistant_memory::now()){
      Ok(data)if data.outcome_ready()=>break Arc::new(data),Ok(_)=>{},Err(model::Error::Busy|model::Error::StorageUnavailable)=>{},Err(e)=>return Err(e),
     }
     tokio::time::sleep(Duration::from_secs(1)).await;
    };
    if !wait.current(){return Err(model::Error::Conflict);}
    self.notice(Phase::Feedback,round,Some("Resultados observados como dados; recibos e texto não verificam os critérios do objetivo."))?;
    let until=Instant::now()+Duration::from_secs(2);loop{match authority.record_peer_round_feedback(spec,&owner.live.proof,owner.lease(),&goal,&data,crate::assistant_memory::now()){
     Ok(_)=>break,Err(crate::pet_runtime::authority::Failure::Busy)if Instant::now()<until=>{if !wait.pending(){return self.stopped(round).await;}tokio::time::sleep(Duration::from_millis(50)).await;},Err(e)=>return Err(e.control_error()),
    }}
    self.resume_peer_wait(&wait,round).await?;wait.retire();
    let request=format!("driver-plan-{}",round+1);
    // Typed passive preflight before requesting another DATA decision. It
    // neither reserves budget nor replaces the writer's boundary check.
    let next=crate::pet_runtime::planning_journal::Request::capture(&goal,spec,owner.lease(),&request)?;
    match authority.planning_context(spec,&owner.live.proof,owner.lease(),next,crate::assistant_memory::now()){
     Ok(_)=>{},Err(crate::pet_runtime::authority::Failure::Capacity)=>return self.finish(Phase::Limited,round,Some("Sem saldo, rondas ou informação nova suficientes; nenhum novo consentimento ou pedido ao modelo foi criado.")).await,Err(e)=>return Err(e.control_error()),
    }
    let prepared=owner.prepare_peer_round(&goal,&capture.server,&request,data,&capture.configuration).map_err(|message|{if let Ok(mut state)=self.state.lock(){state.last_failure=Some(message);}model::Error::Conflict})?;
    let review=prepared.review();let environment=self.environment.clone();let observed_owner=owner.clone();let observe:peer_round_review::Observe=Arc::new(move||{let c=environment.capture(&observed_owner)?;Ok((c.configuration,c.server))});
    let source=Source::capture_peer_data(owner.clone(),&goal,review.clone(),self.store.clone(),observe.clone())?;
    owner.yield_for_decision(&self.store,&source,crate::assistant_memory::now())?;
    self.notice(Phase::AwaitingApproval,round+1,Some("Revê a origem, fontes, destino/modelo e corpo exato desta ronda autónoma. A decisão anterior não a autoriza."))?;
    let Some(decision)=self.environment.decide(source.clone())else{return self.finish(Phase::ApprovalEnded,round+1,Some("O pedido DATA terminou; não foi recriado.")).await;};
    if !decision.approved(){return self.finish(Phase::Declined,round+1,Some("Uso dos dados recusado; sem outra chamada ao modelo.")).await;}
    self.resume_peer_wait(&source,round+1).await?;
    let approved=loop{match review.approve_retained(&decision,owner,&goal,&observe,crate::assistant_memory::now()){
     Ok(approved)=>break Arc::new(approved),Err(model::Error::Busy)=>{if owner.pending_validity(crate::assistant_memory::now())==Validity::Changed{return self.stopped(round+1).await;}tokio::time::sleep(Duration::from_millis(50)).await;},Err(e)=>return Err(e),
    }};
    self.notice(Phase::Planning,round+1,None)?;
    let output=owner.send_peer_round(capture.app.as_ref(),prepared,approved.clone()).await.map_err(|message|{if let Ok(mut state)=self.state.lock(){state.last_failure=Some(message);}model::Error::StorageUnavailable})?;
    if matches!(output.body.outcome,crate::pet_runtime::planning_output::Outcome::Plan{..}){
     // Do not deliver derived text before the recipient lineage/DATA path is
     // complete. Canonical publication remains governed by the retained proof.
     let scope=owner.publish_peer_round_plan(self.store.clone(),&request,approved).map_err(|e|{if let Ok(mut state)=self.state.lock(){state.last_failure=Some(format!("Publicação canónica DATA: {e:?}"));}e})?;self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.data_publication=Some(scope);
     // Re-enter through the common review/wait path. The same native scope
     // is retained; no extra decision, model, budget or resource owner exists.
     continue;
    }
    continue;
   }
   self.notice(Phase::Planning,1,None)?;let capture=self.environment.capture(owner)?;
   owner.plan_once(capture.app.as_ref(),&goal,&capture.server,"driver-plan-1",&capture.configuration).await.map_err(|message|{if let Ok(mut state)=self.state.lock(){state.last_failure=Some(message);}model::Error::StorageUnavailable})?;
  }
 }
}
