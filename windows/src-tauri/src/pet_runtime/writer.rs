//! Owned, single-use native writer. No serialized lease/attempt can create this
//! value. The guard and physical hold outlive the cancellable HTTP future.
use super::*;
use crate::pet_runtime::effect_budget::{Attempt,Kind,Phase};
use crate::chat::ConversationReservation;
use crate::local_chat::RuntimeFence;
use std::sync::atomic::AtomicBool;
struct ModelCall{worker:WorkerGuard,reservation:ConversationReservation,request_id:String,output_reserved:u64,attempt:Mutex<Option<Attempt>>,armed:AtomicBool,dispatched:AtomicBool}
impl Owned{
 fn check_model_goal(&self,request:&str,goal:&Record)->model::Result<()>{
  if !model::key(request)||goal.id!=self.live.lease.goal_id||goal.spec.owner!=self.live.spec.owner||model::fingerprint(&goal.spec)?!=self.live.spec.intent_hash||goal.execution_context.as_ref()!=Some(&self.live.spec.context)||goal.admissions.last().and_then(|a|a.lease.as_ref())!=Some(&self.live.lease)||goal.state!=model::State::Leased||!crate::pet_runtime::plans::matches(goal,&self.live.spec){return Err(model::Error::Conflict);}
  Ok(())
 }
 fn prepare_model(&self,request:&str,goal:&Record,output_limit:u64,now:u64)->model::Result<ModelCall>{
  self.check_model_goal(request,goal)?;
  let worker=self.worker(now)?;
  let reservation=self.live.chat.reserve_for_goal(&self.live.lease.provider,&self.live.lease.conversation_id,&self.live.lease.goal_id,self.live.hold.clone()).map_err(|_|model::Error::Busy)?;
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
  let remaining=authority.output_remaining(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(|_|model::Error::Conflict)?;
  let limit=output_limit.min(remaining).min(4*1024*1024);if limit==0{return Err(model::Error::LimitReached);}
  Ok(ModelCall{worker,reservation,request_id:request.into(),output_reserved:limit,attempt:Mutex::new(None),armed:AtomicBool::new(false),dispatched:AtomicBool::new(false)})
 }
 pub(in crate::pet_runtime)async fn respond_once(&self,app:Option<&tauri::AppHandle>,goal:&Record,server:&crate::local_chat::Server,request:&str,config:&crate::pet_config::Configuration)->Result<String,String>{
  // Native CLI agents have uncontrolled internal requests/tools. They remain
  // unavailable here until their finite limits can be enforced natively.
  if self.live.lease.provider!="ollama"||self.live.spec.owner.adapter!="ollama"||server.id!="ollama"{return Err("Este adaptador ainda não garante os limites de execução autónoma.".into());}
  self.check_model_goal(request,goal).map_err(|_|"O pedido já não corresponde ao objetivo original.")?;
  // Consult durable observations before any metadata or inference IO. An old
  // attempt without a completed receipt cannot be automatically replayed.
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|"Os recibos estão indisponíveis.")?;
  match crate::pet_runtime::results::lookup(&authority,&self.live.spec,&self.live.proof,request).map_err(|_|"Não foi possível verificar o resultado anterior; não repetir automaticamente.")?{
   crate::pet_runtime::results::Lookup::NotStarted=>{},
   crate::pet_runtime::results::Lookup::Available{..}=>return crate::pet_runtime::results::recover(&authority,&self.live.spec,&self.live.proof,request).map(|receipt|receipt.text).map_err(|_|"Não foi possível confirmar o recibo durável; não repetir automaticamente.".into()),
   crate::pet_runtime::results::Lookup::RecoveryRequired{..}=>return Err("A tentativa anterior requer recuperação; nenhum novo pedido foi enviado ao modelo.".into()),
  }
  let query=crate::pet_runtime::plans::model_input(goal,&self.live.spec).map_err(|_|"O contexto revisto mudou ou excedeu o limite; nenhum pedido foi enviado ao modelo.")?;
  let base=crate::local_chat::base_url(server)?;
  if !crate::net::is_loopback_url(&base){return Err("A execução autónoma Ollama requer o servidor local aprovado.".into());}
  // Only successful, fresh native model metadata can establish that this
  // configured loopback Ollama model is installed locally, not a cloud alias.
  let preparation=self.readiness_worker(crate::assistant_memory::now()).map_err(|_|"O objetivo deixou de estar pronto.")?;
  let observation=crate::pet_capabilities::begin_local(&server.url,&self.live.lease.model);
  let metadata=crate::net::while_current(||preparation.in_flight_current(crate::assistant_memory::now()),||async{
   let client=crate::net::local_client(&base,Duration::from_secs(3))?;
   let response=client.post(crate::net::join(&base,"api/show")).json(&serde_json::json!({"model":self.live.lease.model})).send().await.map_err(|_|"Não foi possível confirmar o modelo local.")?;
   if !response.status().is_success(){return Err("O modelo local não foi confirmado.".into());}
   let bytes=crate::net::read_capped(response,512*1024).await?;
   let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|"Metadados de modelo inválidos.")?;
   verify_local_model(&value)?;Ok(value)
  }).await;
  crate::pet_capabilities::finish_local(observation,metadata.as_ref().ok());
  metadata?;drop(preparation);
  let call=self.prepare_model(request,goal,self.live.spec.budget.max_output_bytes,crate::assistant_memory::now()).map_err(|e|format!("Não foi possível reservar a chamada: {e:?}"))?;
  let channel=call.reservation.channel();
  let result=crate::local_chat::send_runtime_once(app,&channel,server,&self.live.lease.model,goal.spec.intent.clone(),query,&call,config).await;
  // A failed/lost acknowledgment retains the full reservation. A successful
  // receipt records raw accepted content bytes, including hidden reasoning.
  let accepted=result.as_ref().ok().map(|r|r.accepted_output_bytes);
  if call.dispatched.load(Ordering::Acquire){call.finish(accepted)?;}
  let result=result?;
  call.record_result(&result.reply.text,result.accepted_output_bytes)?;
  Ok(result.reply.text)
 }
}
impl RuntimeFence for ModelCall{
 fn current(&self)->bool{let now=crate::assistant_memory::now();if self.dispatched.load(Ordering::Acquire){self.worker.in_flight_current(now)}else{self.worker.current(now)}}
 fn output_limit(&self)->usize{self.output_reserved as usize}
 fn before_send(&self,model:&str,body:&serde_json::Value)->Result<(),String>{
  if model!=self.worker.live.lease.model||self.armed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A autorização desta chamada já foi consumida.".into());}
  let live=&self.worker.live;let _gate=live.gate.lock().map_err(|_|"O controlador está indisponível.")?;
  if !self.current(){return Err("O objetivo deixou de estar autorizado/pronto.".into());}
  if body.get("model").and_then(|v|v.as_str())!=Some(model)||body.get("tools").is_some(){return Err("O pedido mudou ou incluiu ferramentas não autorizadas.".into());}
  let now=crate::assistant_memory::now();
  let attempt=Attempt{id:nonce().map_err(|_|"Não foi possível identificar a tentativa.")?,request_id:self.request_id.clone(),input_hash:model::fingerprint(&(&live.lease.id,model,body)).map_err(|_|"Pedido inválido.")?,kind:Kind::Model,output_reserved:self.output_reserved,phase:Phase::Reserved,accepted_output:None,reserved_at:now,started_at:None,finished_at:None};
  *self.attempt.lock().map_err(|_|"O controlador está indisponível.")?=Some(attempt.clone());
  let authority=Authority::existing(&live.authority_path).map_err(|_|"O orçamento está indisponível.")?;
  authority.reserve_attempt(&live.spec,&live.proof,&live.lease,&attempt,now).map_err(|_|"Não foi possível reservar o orçamento antes da chamada; não repetir automaticamente.")?;
  authority.start_attempt(&live.spec,&live.proof,&live.lease,&attempt,crate::assistant_memory::now()).map_err(|_|"Não foi possível confirmar o orçamento antes da chamada; não repetir automaticamente.")?;
  if !self.current(){return Err("O objetivo foi interrompido antes do envio.".into());}
  self.dispatched.store(true,Ordering::Release);Ok(())
 }
}
impl ModelCall{
 fn record_result(&self,text:&str,accepted:u64)->Result<(),String>{
  let live=&self.worker.live;let attempt=self.attempt.lock().map_err(|_|"O controlador está indisponível.")?;
  let attempt=attempt.as_ref().ok_or("A tentativa não foi identificada.")?;
  let authority=Authority::existing(&live.authority_path).map_err(|_|"O recibo está indisponível; não repetir o pedido.")?;
  let envelope=authority.seal_result(&live.spec,&live.proof,&live.lease,attempt,text,accepted,crate::assistant_memory::now()).map_err(|_|"O resultado requer reconciliação; não repetir o pedido.")?;
  crate::pet_runtime::results::publish(&authority,&live.spec,&live.proof,&envelope).map_err(|_|"A publicação do resultado requer reconciliação; não repetir o pedido.".into())
 }
 fn finish(&self,accepted:Option<u64>)->Result<(),String>{let live=&self.worker.live;
  let attempt=self.attempt.lock().map_err(|_|"O controlador está indisponível.")?;let attempt=attempt.as_ref().ok_or("A tentativa não foi identificada.")?;
  Authority::existing(&live.authority_path).and_then(|a|a.finish_attempt(&live.spec,&live.proof,&live.lease,attempt,accepted,crate::assistant_memory::now())).map_err(|_|"O recibo da chamada requer reconciliação; não repetir o pedido.".into())
 }
}

fn verify_local_model(value:&serde_json::Value)->Result<(),String>{
 if ["remote_host","remote_model"].iter().any(|k|value.get(k).is_some_and(|v|!v.is_null()&&v.as_str().is_none_or(|s|!s.trim().is_empty())))
  ||!value.get("model_info").and_then(|v|v.as_object()).is_some_and(|v|!v.is_empty())
  ||!value.get("capabilities").and_then(|v|v.as_array()).is_some_and(|v|v.iter().any(|v|v.as_str()==Some("completion"))){return Err("O modelo local instalado não foi confirmado; nenhum pedido de inferência foi enviado.".into());}
 Ok(())
}

#[cfg(test)]#[path="writer_tests.rs"]mod tests;
