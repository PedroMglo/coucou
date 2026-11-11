//! Owned, single-use native writer. No serialized lease/attempt can create this
//! value. The guard and physical hold outlive the cancellable HTTP future.
use super::*;
use crate::pet_runtime::effect_budget::{Attempt,Kind,Phase};
use crate::chat::ConversationReservation;
use crate::local_chat::RuntimeFence;
use crate::local_tasks::protocol::verify_local_model;
use std::sync::atomic::AtomicBool;
/// Complete immutable native request plus its original conversation admission.
/// Preparing it creates no execution worker or effect-budget reservation. The
/// same value is consumed by the writer; JSON cannot reconstruct this handoff.
struct PreparedCall{peer_data:Option<Arc<crate::pet_runtime::peer_round_data::Captured>>,goal_hash:String,request:crate::local_chat::PreparedRuntimeRequest,reservation:ConversationReservation,planning:Option<crate::pet_runtime::planning_context::Context>}
/// Retained native data lineage after exact own canonical plan publication.
/// It can validate that publication/delivery, never authorize another model IO.
enum PeerPublicationLineage{Retained(Arc<crate::pet_runtime::peer_round_review::Approved>),Reconciled(crate::pet_runtime::authority_store::ReconciledPeerPublication)}
pub(in crate::pet_runtime)struct PeerRoundPublished{goal:Record,request_id:String,lineage:PeerPublicationLineage,owner:Arc<Owned>,store:Arc<Store>,revoked:AtomicBool}
#[path="peer_publication_controller.rs"]mod peer_publication;
pub(in crate::pet_runtime)struct PreparedPeerRound{goal:Record,request_id:String,prepared:PreparedCall,review:Arc<crate::pet_runtime::peer_round_review::Review>}
impl PreparedPeerRound{
 pub(in crate::pet_runtime)fn goal(&self)->&Record{&self.goal}
 pub(in crate::pet_runtime)fn review(&self)->Arc<crate::pet_runtime::peer_round_review::Review>{self.review.clone()}
}
struct ModelCall{planning:Option<crate::pet_runtime::planning_journal::Request>,worker:WorkerGuard,reservation:ConversationReservation,request_id:String,output_reserved:u64,attempt:Mutex<Option<Attempt>>,armed:AtomicBool,dispatched:AtomicBool}
impl Owned{
 fn check_model_goal(&self,request:&str,goal:&Record)->model::Result<()>{
  if !model::key(request)||goal.id!=self.live.lease.goal_id||goal.spec.owner!=self.live.spec.owner||model::fingerprint(&goal.spec)?!=self.live.spec.intent_hash||goal.execution_context.as_ref()!=Some(&self.live.spec.context)||goal.admissions.last().and_then(|a|a.lease.as_ref())!=Some(&self.live.lease)||goal.state!=model::State::Leased||!crate::pet_runtime::plans::matches(goal,&self.live.spec){return Err(model::Error::Conflict);}
  Ok(())
 }
 fn prepare_model(&self,request:&str,goal:&Record,output_limit:u64,now:u64)->model::Result<ModelCall>{
  if self.live.spec.planning_policy.is_some(){return Err(model::Error::Conflict);}
  self.prepare_model_call(request,goal,output_limit,None,now)
 }
 fn prepare_planning_model(&self,request:&str,goal:&Record,now:u64)->model::Result<ModelCall>{
  let captured=crate::pet_runtime::planning_journal::Request::capture(goal,&self.live.spec,&self.live.lease,request)?;
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
  let context=authority.planning_context(&self.live.spec,&self.live.proof,&self.live.lease,captured,now).map_err(|_|model::Error::Conflict)?;
  self.prepare_planning_captured(goal,context,now)
 }
 fn prepare_planning_captured(&self,goal:&Record,context:crate::pet_runtime::planning_context::Context,now:u64)->model::Result<ModelCall>{
  if !context.request.matches(&self.live.spec,&self.live.lease)||context.request.source.record_hash!=model::fingerprint(goal)?{return Err(model::Error::Conflict);}
  let request=context.request.request_id.clone();self.prepare_model_call(&request,goal,context.input.remaining.model_output_limit,Some(context.request),now)
 }
 fn prepare_model_call(&self,request:&str,goal:&Record,output_limit:u64,planning:Option<crate::pet_runtime::planning_journal::Request>,now:u64)->model::Result<ModelCall>{
  self.prepare_model_call_with_reservation(request,goal,output_limit,planning,None,now)
 }
 fn prepare_model_call_with_reservation(&self,request:&str,goal:&Record,output_limit:u64,planning:Option<crate::pet_runtime::planning_journal::Request>,reservation:Option<ConversationReservation>,now:u64)->model::Result<ModelCall>{
  if self.live.spec.deterministic(){return Err(model::Error::Conflict);}
  self.check_model_goal(request,goal)?;
  if planning.is_some()!=self.live.spec.planning_policy.is_some(){return Err(model::Error::Conflict);}
  let worker=self.worker(now)?;
  let reservation=if let Some(r)=reservation{r}else{self.live.chat.reserve_for_goal(&self.live.lease.provider,&self.live.lease.conversation_id,&self.live.lease.goal_id,self.live.hold.clone()).map_err(|_|model::Error::Busy)?};
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;
  let remaining=authority.output_remaining(&self.live.spec,&self.live.proof,&self.live.lease,now).map_err(|_|model::Error::Conflict)?;
  let limit=output_limit.min(remaining).min(4*1024*1024);if limit==0{return Err(model::Error::LimitReached);}
  Ok(ModelCall{planning,worker,reservation,request_id:request.into(),output_reserved:limit,attempt:Mutex::new(None),armed:AtomicBool::new(false),dispatched:AtomicBool::new(false)})
 }
 /// Capture the actual identity/history/data/body while the goal has zero
 /// workers. A future exact DATA decision must retain this object through its
 /// wait, not rebuild the prompt after consent or reuse a caller-supplied body.
 fn prepare_actual_call(&self,goal:&Record,server:&crate::local_chat::Server,query:String,planning:Option<crate::pet_runtime::planning_context::Context>,config:&crate::pet_config::Configuration)->Result<PreparedCall,String>{
  self.prepare_actual_call_with_peer_data(goal,server,query,planning,None,config)
 }
 fn prepare_actual_call_with_peer_data(&self,goal:&Record,server:&crate::local_chat::Server,query:String,planning:Option<crate::pet_runtime::planning_context::Context>,peer_data:Option<Arc<crate::pet_runtime::peer_round_data::Captured>>,config:&crate::pet_config::Configuration)->Result<PreparedCall,String>{
  // Composing the exact DATA body is passive. Cached model readiness may age
  // while children run; send_peer_round resolves the original model explicitly
  // after DATA approval, before the FreshEffect worker/reservation/HTTP gates.
  let preparation_current=||if peer_data.is_some(){self.policy_validity(crate::assistant_memory::now())==Validity::Current}else{self.current(crate::assistant_memory::now())};
  if self.workers()!=0||!preparation_current(){return Err("O objetivo não está disponível para preparar o pedido.".into());}
  let goal_hash=model::fingerprint(goal).map_err(|_|"O objetivo não pode ser capturado.")?;
  self.check_model_goal(planning.as_ref().map(|c|c.request.request_id.as_str()).unwrap_or("prepare-actual-call"),goal).map_err(|_|"O objetivo não corresponde ao dono original do pedido.")?;
  let reservation=self.live.chat.reserve_for_goal(&self.live.lease.provider,&self.live.lease.conversation_id,&self.live.lease.goal_id,self.live.hold.clone()).map_err(|_|"A conversa original está ocupada.")?;
  let channel=reservation.channel();let memory=channel.goal_memory_message(&self.live.lease.goal_id)?;
  if self.live.spec.memory_data.is_some()!=memory.is_some(){return Err("O contexto de dados não corresponde à concessão do objetivo.".into());}
  let own_memory=if self.live.spec.peer_selected_iteration(){memory.clone()}else{None};
  let memory=if let Some(data)=&peer_data{
   if !self.live.spec.peer_round_destination_matches(&data.reference().destination)||memory.is_some()&&!self.live.spec.peer_selected_iteration()||!data.current(server,&self.live.lease.model,crate::assistant_memory::now()){return Err("Os dados da ronda não correspondem ao objetivo ou destino original.".into());}
   Some(data.model_message())
  }else{memory};
  let mut request=crate::local_chat::prepare_runtime_request(&channel,server,&self.live.lease.model,goal.spec.intent.clone(),query,&self.live.spec.owner.provider,memory,config)?;
  if let Some(data)=&peer_data{if data.canonical_history().is_some()||own_memory.is_some(){request.select_peer_context(data.canonical_history(),own_memory)?;}}
  if !preparation_current(){return Err("O objetivo mudou durante a preparação do pedido.".into());}
  Ok(PreparedCall{peer_data,goal_hash,request,reservation,planning})
 }
 fn consume_actual_call(&self,goal:&Record,request_id:&str,prepared:PreparedCall,now:u64)->model::Result<(ModelCall,crate::local_chat::PreparedRuntimeRequest)>{
  self.consume_actual_call_with_peer_data(goal,request_id,prepared,None,now)
 }
 fn consume_actual_call_with_peer_data(&self,goal:&Record,request_id:&str,prepared:PreparedCall,approved:Option<Arc<crate::pet_runtime::peer_round_review::Approved>>,now:u64)->model::Result<(ModelCall,crate::local_chat::PreparedRuntimeRequest)>{
  match(&prepared.peer_data,&approved){(None,None)=>{},(Some(data),Some(a))if a.reference().source==*data.reference()&&a.matches_request(self,goal,&prepared.request,&a.observe_for_writer(),now)=>{},_=>return Err(model::Error::Conflict)}
  if prepared.goal_hash!=model::fingerprint(goal)?{return Err(model::Error::Conflict);}
  let PreparedCall{peer_data:_,goal_hash:_,request,reservation,planning}=prepared;
  let(planning,output_limit)=if let Some(mut context)=planning{
   if !context.request.matches(&self.live.spec,&self.live.lease)||context.request.request_id!=request_id||context.request.source.record_hash!=model::fingerprint(goal)?{return Err(model::Error::Conflict);}
   if let Some(approved)=approved{context.request.bind_peer_data(approved)?;}
   (Some(context.request),context.input.remaining.model_output_limit)
  }else{if approved.is_some()||self.live.spec.planning_policy.is_some(){return Err(model::Error::Conflict);}(None,self.live.spec.budget.max_output_bytes)};
  let call=self.prepare_model_call_with_reservation(request_id,goal,output_limit,planning,Some(reservation),now)?;
  Ok((call,request))
 }
 /// Prepare one exact native body without a model worker or data authority.
 /// The Driver retains this object across its existing human/resource wait.
 pub(in crate::pet_runtime)fn prepare_peer_round(&self,goal:&Record,server:&crate::local_chat::Server,request:&str,data:Arc<crate::pet_runtime::peer_round_data::Captured>,config:&crate::pet_config::Configuration)->Result<PreparedPeerRound,String>{
  if !self.live.spec.peer_delegation(){return Err("Este objetivo não autoriza o catálogo peer revisto.".into());}
  self.check_model_goal(request,goal).map_err(|_|"O dono do pedido mudou.")?;
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|"O orçamento original está indisponível.")?;
  match crate::pet_runtime::results::lookup(&authority,&self.live.spec,&self.live.proof,request).map_err(|_|"Os recibos originais estão indisponíveis; não repetir o pedido.")?{crate::pet_runtime::results::Lookup::NotStarted=>{},_=>return Err("O pedido já tem uma tentativa; requer a reconciliação original, sem repetir inferência.".into())}
  crate::pet_runtime::results::preflight(&authority,&self.live.spec.decision_id).map_err(|_|"O armazenamento dos resultados está indisponível.")?;
  let captured=crate::pet_runtime::planning_journal::Request::capture(goal,&self.live.spec,&self.live.lease,request).map_err(|_|"O objetivo mudou.")?;
  let context=authority.planning_context(&self.live.spec,&self.live.proof,&self.live.lease,captured,crate::assistant_memory::now()).map_err(|_|"O journal ou orçamento original não admite esta ronda.")?;
  let input=crate::pet_runtime::planning::model_input(goal,&self.live.spec,&context.input).map_err(|_|"O contexto do objetivo não está disponível.")?;
  let prepared=self.prepare_actual_call_with_peer_data(goal,server,input,Some(context),Some(data.clone()),config)?;
  let review=crate::pet_runtime::peer_round_review::Review::capture(self,goal,data,&prepared.request,crate::assistant_memory::now()).map_err(|_|"Os dados ou o corpo real mudaram.")?;
  Ok(PreparedPeerRound{goal:goal.clone(),request_id:request.into(),prepared,review})
 }
 pub(in crate::pet_runtime)async fn send_peer_round(&self,app:Option<&tauri::AppHandle>,round:PreparedPeerRound,approved:Arc<crate::pet_runtime::peer_round_review::Approved>)->Result<crate::pet_runtime::planning_output::Envelope,String>{
  let PreparedPeerRound{goal,request_id,prepared,review}=round;
  if approved.reference().id!=review.id||!approved.matches_request(self,&goal,&prepared.request,&approved.observe_for_writer(),crate::assistant_memory::now()){return Err("A decisão DATA não corresponde ao corpo e turno originais.".into());}
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|"Os recibos estão indisponíveis; não repetir o pedido.")?;
  match crate::pet_runtime::results::lookup(&authority,&self.live.spec,&self.live.proof,&request_id).map_err(|_|"Não foi possível verificar a tentativa original.")?{crate::pet_runtime::results::Lookup::NotStarted=>{},_=>return Err("A tentativa original requer reconciliação; não foi iniciada outra chamada.".into())}
  // Active native model resolution is explicit. Metadata never consumes an
  // inference attempt, forwards peer data, changes destination or renews DATA.
  let captured=self.live.hold.captured();self.verify_runtime_model(prepared.request.server(),captured.as_deref()).await?;
  let(call,prepared)=self.consume_actual_call_with_peer_data(&goal,&request_id,prepared,Some(approved.clone()),crate::assistant_memory::now()).map_err(|_|"A decisão, dados ou orçamento deixaram de ser válidos.")?;
  let channel=call.reservation.channel();let result=crate::local_chat::send_runtime_prepared(app,&channel,prepared,&call).await;
  let accepted=result.as_ref().ok().map(|r|r.accepted_output_bytes);if call.dispatched.load(Ordering::Acquire){call.finish(accepted)?;}
  let result=result?;call.record_result(&result.reply.text,result.accepted_output_bytes)?;
  if approved.publication_validity(&authority,&self.live.spec,&self.live.proof,&request_id,crate::assistant_memory::now())!=Validity::Current{return Err("O resultado existe, mas os dados mudaram antes da publicação da proposta; requer reconciliação, sem repetir a chamada.".into());}
  crate::pet_runtime::planning_output::reconcile(&authority,&self.live.spec,&self.live.proof,&self.live.lease,&goal,&request_id,crate::assistant_memory::now()).map_err(|_|"O resultado/proposta requer reconciliação durável, sem repetir inferência.".into())
 }
 pub(in crate::pet_runtime)fn publish_peer_round_plan(self:&Arc<Self>,store:Arc<Store>,request:&str,approved:Arc<crate::pet_runtime::peer_round_review::Approved>)->model::Result<Arc<PeerRoundPublished>>{
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|model::Error::StorageUnavailable)?;let before=store.read_control(&self.live.spec.goal_id,&self.live.spec.owner.pet_id)?;
  if model::fingerprint(&before)?!=approved.reference().goal_hash{
   // Reconcile only an already present exact own publication/ACK. This does
   // not treat stale DATA as permission for a fresh canonical write or IO.
   approved.acknowledge_existing_plan(&authority,&self.live.spec,&self.live.proof,&before,request,crate::assistant_memory::now())?;
  }
  let published=self.publish_generated_plan_with_data(&store,request,&approved,crate::assistant_memory::now())?;
  if !published.acknowledged||published.receipt.status==model::CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}
  let scope=Arc::new(PeerRoundPublished{goal:published.receipt.record,request_id:request.into(),lineage:PeerPublicationLineage::Retained(approved),owner:self.clone(),store,revoked:AtomicBool::new(false)});
  match scope.current(){Validity::Current=>{if let PeerPublicationLineage::Retained(a)=&scope.lineage{a.note_published();}Ok(scope)},Validity::Busy=>Err(model::Error::Busy),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>Err(model::Error::Conflict)}
 }
 async fn verify_runtime_model(&self,server:&crate::local_chat::Server,captured:Option<&crate::pet_inference::Captured>)->Result<(),String>{
  let base=crate::local_chat::base_url(server)?;
  if !crate::net::is_loopback_url(&base){return Err(if server.id=="lmstudio"{"A execução autónoma LM Studio requer o servidor local aprovado."}else{"A execução autónoma Ollama requer o servidor local aprovado."}.into());}
  // Only successful, fresh native model metadata can establish that this
  // configured loopback Ollama model is installed locally, not a cloud alias.
  let preparation=self.readiness_worker(crate::assistant_memory::now()).map_err(|_|"O objetivo deixou de estar pronto.")?;
  if let Some(reference)=&self.live.spec.rule_occurrence{
   let parent=Authority::existing(&self.live.authority_path).and_then(|a|a.rule_grant(&reference.rule_decision_id)).map_err(|_|"A regra original requer reconciliação.")?;
   if parent.policy.dispatch.is_some(){
    // Automatic readiness IO is already charged durably in this occurrence's
    // parent reservation. Do not dispatch another hidden metadata request here.
    if server.id!="ollama"||!crate::pet_capabilities::fresh_installed_local_model(&server.url,&self.live.lease.model,crate::assistant_memory::now())||!preparation.in_flight_current(crate::assistant_memory::now()){
     return Err("A evidência local da rotina expirou ou não confirmou o modelo instalado; requer prontidão pelo percurso limitado original.".into());
    }
    return Ok(());
   }
  }
  if server.id=="lmstudio"{
   let binding=if let Some(captured)=captured{
    crate::local_chat::lmstudio::inspect_scoped(server,&self.live.lease.model,crate::local_chat::metadata_auth_for_capture(captured)?,||preparation.in_flight_current(crate::assistant_memory::now())).await?
   }else{crate::local_chat::lmstudio::inspect(server,&self.live.lease.model,||preparation.in_flight_current(crate::assistant_memory::now())).await?};
   if self.live.spec.engine_binding.as_ref()!=Some(&binding){return Err("A identidade do modelo LM Studio mudou; é necessária outra revisão.".into());}
  }else{
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
  metadata?;
  }
  drop(preparation);
  Ok(())
 }
 /// Pure transport projection for app/dispatcher capture. Secrets are resolved
 /// from the original hold only AFTER the durable result/uncertainty lookup in
 /// model_once. The projection itself is neither a grant nor a dispatch server.
 pub(in crate::pet_runtime)fn server_projection(&self,settings:&crate::settings::Settings)->Result<crate::local_chat::Server,String>{
  if self.live.spec.deterministic(){return Err("Este objetivo não autoriza inferência.".into());}
  if let Some(captured)=self.live.hold.captured(){
   if captured.provider!=self.live.spec.owner.provider||captured.adapter!=self.live.spec.owner.adapter||captured.explicit.as_ref().is_some_and(|t|t.choice.model!=self.live.lease.model){return Err("O motor capturado não corresponde ao objetivo revisto.".into());}
   return Ok(crate::local_chat::Server{id:captured.adapter,name:if captured.adapter=="lmstudio"{"LM Studio"}else{"Ollama"},url:captured.endpoint.clone(),key:None});
  }
  crate::local_chat::server(settings,&self.live.spec.owner.provider).filter(|s|s.id==self.live.spec.owner.adapter).ok_or("O adaptador não corresponde ao objetivo revisto.".into())
 }
 pub(in crate::pet_runtime)async fn respond_once(&self,app:Option<&tauri::AppHandle>,goal:&Record,server:&crate::local_chat::Server,request:&str,config:&crate::pet_config::Configuration)->Result<String,String>{
  if self.live.spec.planning_policy.is_some(){return Err("Este consentimento permite planeamento tipado, não uma resposta genérica.".into());}
  self.model_once(app,goal,server,request,config,false).await
 }
 pub(in crate::pet_runtime)async fn plan_once(&self,app:Option<&tauri::AppHandle>,goal:&Record,server:&crate::local_chat::Server,request:&str,config:&crate::pet_config::Configuration)->Result<crate::pet_runtime::planning_output::Envelope,String>{
  if self.live.spec.planning_policy.is_none()||!self.live.spec.peer_planning()&&!config.permissions(&self.live.spec.owner.pet_id).map_err(|_|"Configuração indisponível.")?.music{return Err("O planeamento não está autorizado neste objetivo.".into());}
  self.model_once(app,goal,server,request,config,true).await?;
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|"O recibo de planeamento está indisponível; não repetir a chamada.")?;
  crate::pet_runtime::planning_output::reconcile(&authority,&self.live.spec,&self.live.proof,&self.live.lease,goal,request,crate::assistant_memory::now()).map_err(|_|"A proposta requer reconciliação durável; não repetir a chamada.".into())
 }
 async fn model_once(&self,app:Option<&tauri::AppHandle>,goal:&Record,server:&crate::local_chat::Server,request:&str,config:&crate::pet_config::Configuration,planning:bool)->Result<String,String>{
  // Native CLI agents have uncontrolled internal requests/tools. They remain
  // unavailable here until their finite limits can be enforced natively.
  if self.live.spec.deterministic()||!self.live.spec.owner.local_engine(){return Err("Este adaptador ainda não garante os limites de execução autónoma.".into());}
  self.check_model_goal(request,goal).map_err(|_|"O pedido já não corresponde ao objetivo original.")?;
  // Consult durable observations before any metadata or inference IO. An old
  // attempt without a completed receipt cannot be automatically replayed.
  let authority=Authority::existing(&self.live.authority_path).map_err(|_|"Os recibos estão indisponíveis.")?;
  match crate::pet_runtime::results::lookup(&authority,&self.live.spec,&self.live.proof,request).map_err(|_|"Não foi possível verificar o resultado anterior; não repetir automaticamente.")?{
   crate::pet_runtime::results::Lookup::NotStarted=>{},
   crate::pet_runtime::results::Lookup::Available{..}=>return crate::pet_runtime::results::recover(&authority,&self.live.spec,&self.live.proof,request).map(|receipt|receipt.text).map_err(|_|"Não foi possível confirmar o recibo durável; não repetir automaticamente.".into()),
   crate::pet_runtime::results::Lookup::RecoveryRequired{..}=>return Err("A tentativa anterior requer recuperação; nenhum novo pedido foi enviado ao modelo.".into()),
  }
  crate::pet_runtime::results::preflight(&authority,&self.live.spec.decision_id).map_err(|_|"O armazenamento dos resultados está inválido ou indisponível; nenhum pedido foi enviado ao modelo.")?;
  let captured=self.live.hold.captured();let resolved=captured.as_ref().map(|c|crate::local_chat::server_for_capture(c)).transpose()?;let server=resolved.as_ref().unwrap_or(server);
  if server.id!=self.live.spec.owner.adapter||captured.as_ref().and_then(|c|c.explicit.as_ref()).is_some_and(|t|t.choice.model!=self.live.lease.model){return Err("O destino capturado não corresponde ao objetivo revisto.".into());}
  if let Some(data)=&self.live.spec.memory_data{
   let destination=crate::pet_runtime::memory_review::destination_for(&self.live.spec.owner,server,&self.live.lease.model).map_err(|_|"O destino revisto dos dados mudou.")?;
   if destination!=data.destination||!data.valid(server.id,&self.live.lease.model)||server.id=="ollama"&&server.key.is_some(){return Err("O destino revisto dos dados mudou; nenhuma chamada foi enviada.".into());}
  }
  let context=if planning {
   let captured=crate::pet_runtime::planning_journal::Request::capture(goal,&self.live.spec,&self.live.lease,request).map_err(|_|"A origem do planeamento mudou.")?;
   Some(authority.planning_context(&self.live.spec,&self.live.proof,&self.live.lease,captured,crate::assistant_memory::now()).map_err(|_|"O planeamento atingiu um limite ou requer reconciliação; nenhuma chamada foi enviada.")?)
  }else{None};
  let query=if let Some(context)=&context{crate::pet_runtime::planning::model_input(goal,&self.live.spec,&context.input)}else{crate::pet_runtime::plans::model_input(goal,&self.live.spec)}.map_err(|_|"O contexto revisto mudou ou excedeu o limite; nenhum pedido foi enviado ao modelo.")?;
  self.verify_runtime_model(server,captured.as_deref()).await?;
  let prepared=self.prepare_actual_call(goal,server,query,context,config)?;
  let(call,prepared)=self.consume_actual_call(goal,request,prepared,crate::assistant_memory::now()).map_err(|e|format!("Não foi possível reservar a chamada: {e:?}"))?;
  let channel=call.reservation.channel();
  let result=crate::local_chat::send_runtime_prepared(app,&channel,prepared,&call).await;
  // A failed/lost acknowledgment retains the full reservation. A successful
  // receipt records raw accepted content bytes, including hidden reasoning.
  let accepted=result.as_ref().ok().map(|r|r.accepted_output_bytes);
  if call.dispatched.load(Ordering::Acquire){call.finish(accepted)?;}
  let result=result?;
  call.record_result(&result.reply.text,result.accepted_output_bytes)?;
  Ok(result.reply.text)
 }
}
impl ModelCall{
 /// Only Busy, before HTTP, on this exact retained worker/attempt. A published
 /// or uncertain ACK is not retried; neither the budget nor deadline is renewed.
 fn budget_when_available(&self,mut operation:impl FnMut()->crate::pet_runtime::authority::Result<()>)->crate::pet_runtime::authority::Result<()>{
  let until=Instant::now()+Duration::from_secs(1);
  loop{
   if self.worker.live.closed.load(Ordering::Acquire)||self.worker.live.hold.stopped(){return Err(crate::pet_runtime::authority::Failure::Revoked);}
   match operation(){Err(crate::pet_runtime::authority::Failure::Busy)if Instant::now()<until=>std::thread::sleep(Duration::from_millis(10)),result=>return result}
  }
 }
}
impl RuntimeFence for ModelCall{
 fn provider(&self)->&str{&self.worker.live.spec.owner.provider}
 fn current(&self)->bool{let now=crate::assistant_memory::now();let owned=if self.dispatched.load(Ordering::Acquire){self.worker.in_flight_current(now)}else{self.worker.current(now)};
  owned&&self.planning.as_ref().and_then(|p|p.peer_data.as_ref()).is_none_or(|d|d.bound_validity(now)==Validity::Current)
 }
 fn output_limit(&self)->usize{self.output_reserved as usize}
 fn memory_message(&self)->Result<Option<serde_json::Value>,String>{
  let message=self.reservation.channel().goal_memory_message(&self.worker.live.lease.goal_id)?;
  if self.worker.live.spec.memory_data.is_some()!=message.is_some(){return Err("O contexto de dados não corresponde à concessão do objetivo.".into());}Ok(message)
 }
 fn before_send(&self,model:&str,body:&serde_json::Value)->Result<(),String>{
  if self.worker.live.spec.memory_data.is_some()||self.planning.as_ref().is_some_and(|p|p.peer_data.is_some()){return Err("O destino dos dados tem de ser validado antes da chamada.".into());}
  self.authorize_body(model,body)
 }
 fn before_send_to(&self,server:&crate::local_chat::Server,model:&str,body:&serde_json::Value)->Result<(),String>{
  if server.id!=self.worker.live.spec.owner.adapter{return Err("O adaptador não corresponde ao objetivo revisto.".into());}
  if self.planning.as_ref().and_then(|p|p.peer_data.as_ref()).is_some_and(|d|!d.matches_dispatch(server,model,body,crate::assistant_memory::now())){return Err("Os dados, corpo ou destino da ronda mudaram; nenhum novo envio foi autorizado.".into());}
  if let Some(captured)=self.worker.live.hold.captured(){
   captured.check()?;
   if captured.adapter!=server.id||captured.endpoint!=server.url||captured.explicit.as_ref().is_some_and(|t|t.choice.model!=model){return Err("O destino não corresponde à reserva original do objetivo.".into());}
  }
  if let Some(data)=&self.worker.live.spec.memory_data{
   let destination=crate::pet_runtime::memory_review::destination_for(&self.worker.live.spec.owner,server,model).map_err(|_|"O destino dos dados mudou.")?;
   if destination!=data.destination||!data.valid(server.id,model)||server.id=="ollama"&&server.key.is_some(){return Err("O destino dos dados mudou; é necessária outra revisão.".into());}
   let message=self.memory_message()?.ok_or("As recordações selecionadas estão indisponíveis.")?;
   let messages=body.get("messages").and_then(serde_json::Value::as_array).ok_or("O pedido do objetivo é inválido.")?;
   let joint=self.worker.live.spec.peer_selected_iteration()&&self.planning.as_ref().is_some_and(|p|p.peer_data.is_some());
   let memory_matches=if joint{messages.len()>=4&&messages.get(1)==Some(&message)&&messages.iter().filter(|m|**m==message).count()==1}else{messages.len()>=2&&messages.get(messages.len()-2)==Some(&message)};
   if !memory_matches||serde_json::to_vec(body).map_err(|_|"Pedido inválido.")?.len()>data.max_input_bytes{return Err("O pedido mudou ou excedeu o limite revisto; nenhuma chamada foi enviada.".into());}
  }
  self.authorize_body(model,body)
 }
}
impl ModelCall{
 fn authorize_body(&self,model:&str,body:&serde_json::Value)->Result<(),String>{
  if model!=self.worker.live.lease.model||self.armed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A autorização desta chamada já foi consumida.".into());}
  let live=&self.worker.live;let _gate=live.gate.lock().map_err(|_|"O controlador está indisponível.")?;
  if !self.current(){return Err("O objetivo deixou de estar autorizado/pronto.".into());}
  if body.get("model").and_then(|v|v.as_str())!=Some(model)||body.get("tools").is_some(){return Err("O pedido mudou ou incluiu ferramentas não autorizadas.".into());}
  let authority=Authority::existing(&live.authority_path).map_err(|_|"O orçamento está indisponível.")?;
  // Metadata IO may have taken time since the first check. Inspect the exact
  // native decision again before reserving anything; never bless an old folder.
  crate::pet_runtime::results::preflight(&authority,&live.spec.decision_id).map_err(|_|"O armazenamento dos resultados está inválido ou indisponível; nenhum pedido foi enviado ao modelo.")?;
  let now=crate::assistant_memory::now();
  let attempt=Attempt{id:nonce().map_err(|_|"Não foi possível identificar a tentativa.")?,request_id:self.request_id.clone(),input_hash:model::fingerprint(&(&live.lease.id,model,body)).map_err(|_|"Pedido inválido.")?,kind:Kind::Model,output_reserved:self.output_reserved,phase:Phase::Reserved,accepted_output:None,reserved_at:now,started_at:None,finished_at:None};
  *self.attempt.lock().map_err(|_|"O controlador está indisponível.")?=Some(attempt.clone());
  self.budget_when_available(||{
   let at=crate::assistant_memory::now();
   if let Some(request)=&self.planning{authority.reserve_planning_attempt(&live.spec,&live.proof,&live.lease,&attempt,request,at)}else{authority.reserve_attempt(&live.spec,&live.proof,&live.lease,&attempt,at)}
  }).map_err(|error|format!("Não foi possível reservar o orçamento antes da chamada ({error:?}); não repetir automaticamente."))?;
  self.budget_when_available(||authority.start_attempt(&live.spec,&live.proof,&live.lease,&attempt,crate::assistant_memory::now())).map_err(|error|format!("Não foi possível confirmar o orçamento antes da chamada ({error:?}); não repetir automaticamente."))?;
  if !self.current(){return Err("O objetivo foi interrompido antes do envio.".into());}
  self.dispatched.store(true,Ordering::Release);Ok(())
 }
}
impl ModelCall{
 fn record_result(&self,text:&str,accepted:u64)->Result<(),String>{
  if self.planning.as_ref().and_then(|p|p.peer_data.as_ref()).is_some_and(|d|d.bound_validity(crate::assistant_memory::now())!=Validity::Current){return Err("Os dados foram revogados antes da publicação do resultado; requer reconciliação, sem repetir a chamada.".into());}
  let live=&self.worker.live;let attempt=self.attempt.lock().map_err(|_|"O controlador está indisponível.")?;
  let attempt=attempt.as_ref().ok_or("A tentativa não foi identificada.")?;
  let authority=Authority::existing(&live.authority_path).map_err(|_|"O recibo está indisponível; não repetir o pedido.")?;
  let envelope=authority.seal_result(&live.spec,&live.proof,&live.lease,attempt,text,accepted,crate::assistant_memory::now()).map_err(|_|"O resultado requer reconciliação; não repetir o pedido.")?;
  if self.planning.as_ref().and_then(|p|p.peer_data.as_ref()).is_some_and(|d|d.bound_validity(crate::assistant_memory::now())!=Validity::Current){return Err("Os dados mudaram antes da publicação do resultado; não repetir o pedido.".into());}
  crate::pet_runtime::results::publish(&authority,&live.spec,&live.proof,&envelope).map_err(|_|"A publicação do resultado requer reconciliação; não repetir o pedido.".into())
 }
 fn finish(&self,accepted:Option<u64>)->Result<(),String>{let live=&self.worker.live;
  let attempt=self.attempt.lock().map_err(|_|"O controlador está indisponível.")?;let attempt=attempt.as_ref().ok_or("A tentativa não foi identificada.")?;
  Authority::existing(&live.authority_path).and_then(|a|a.finish_attempt(&live.spec,&live.proof,&live.lease,attempt,accepted,crate::assistant_memory::now())).map_err(|_|"O recibo da chamada requer reconciliação; não repetir o pedido.".into())
 }
}


#[cfg(test)]#[path="writer_tests.rs"]pub(in crate::pet_runtime)mod tests;
