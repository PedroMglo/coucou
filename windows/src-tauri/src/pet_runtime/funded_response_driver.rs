//! One bounded response on the existing original-owner Driver. The response is
//! an artefact, never a certificate for arbitrary delegated completion criteria.
use super::*;
impl Driver{
 pub(super)fn sync_funded_stop(&self)->model::Result<bool>{
  let g=self.store.read_control(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;
  if g.admissions.last().is_none_or(|r|model::fingerprint(&r.spec).ok()!=model::fingerprint(&self.owner.live.spec).ok()){return Err(model::Error::Conflict);}
  // Pause is a common authority-reducing control. A concurrent Cancel remains
  // sticky; this path never replaces it, renews budget, or creates an admission.
  let reason=self.authority()?.recorded_stop_reason(&self.owner.live.spec,&self.owner.live.proof).map_err(crate::pet_runtime::authority::Failure::control_error)?;
  let cancel=reason==Some(crate::pet_runtime::leases::StopReason::Cancel);
  if g.state==model::State::Leased||g.state==model::State::Stopping&&cancel&&g.admissions.last().is_some_and(|r|r.stop_reason!=reason){
   let receipt=crate::pet_runtime::authority_store::mutate_with_authority(self.store.as_ref(),&self.owner.live.authority_path,&g.id,&g.spec.owner.pet_id,&g.revision,format!("funded-response-close-{}-{}",self.owner.lease().id,if cancel{"cancel"}else{"pause"}),if cancel{model::Action::Cancel}else{model::Action::Pause},crate::assistant_memory::now())?;
   if receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}
  }
  Ok(cancel)
 }
 async fn close_funded_response(&self,phase:Phase,reason:&str)->model::Result<()>{
  self.finish(phase,1,Some(reason)).await
 }
 pub(super)async fn drive_funded_response(&self)->model::Result<()>{
  if !self.owner.live.spec.finite_response(){return Err(model::Error::Conflict);}
  let g=self.store.read_control(&self.owner.lease().goal_id,&self.owner.live.spec.owner.pet_id)?;
  if g.state!=model::State::Leased||!self.current().await?{
   return self.close_funded_response(Phase::Stopped,"A autorização deixou de estar atual; apenas o fecho original foi efetuado.").await;
  }
  self.notice(Phase::Executing,1,Some("Uma resposta com os dados e destino revistos; não verifica critérios nem autoriza ferramentas."))?;
  let captured=match self.environment.capture(&self.owner){Ok(c)=>c,Err(_)=>return self.close_funded_response(Phase::RecoveryRequired,"O destino configurado está indisponível; não foi escolhido outro motor ou modelo.").await};
  let request=format!("response-{}",self.owner.live.spec.decision_id);
  let result=self.owner.respond_once(captured.app.as_ref(),&g,&captured.server,&request,&captured.configuration).await;
  match result{
   Ok(_)=>self.close_funded_response(Phase::AwaitingVerification,"Resposta própria guardada; critérios do subobjetivo por verificar. Worker drenado e orçamento não renovado.").await,
   Err(_)=>self.close_funded_response(Phase::RecoveryRequired,"A resposta não foi confirmada. Consulta a tentativa e os dados já entregues; nenhum efeito incerto foi repetido.").await,
  }
 }
}
