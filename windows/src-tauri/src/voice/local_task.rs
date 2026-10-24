//! Spoken Ollama requests enter the same owned, approved runner as written chat.
//! Microphone cancellation can stop admission; accepted work has its own Stop.
use std::{future::Future,sync::atomic::{AtomicBool,Ordering}};
use tauri::{AppHandle,Manager};
use crate::{chat::{Chat,ChatReply,ConversationReservation},pet_config::{Configuration,LocalExecution,LocalExecutionMode,Selection},settings::Settings};

const CANCELLED:&str="A tarefa foi interrompida antes de ser recebida.";
pub(super) fn requested(query:&str)->bool {
 let query=query.trim_start().to_lowercase();
 ["tarefa local","local task"].iter().any(|prefix|query.strip_prefix(prefix).is_some_and(|rest|{
  rest.starts_with([' ',':'])&&!rest.trim_start_matches([' ',':']).trim().is_empty()
 }))
}
pub(super) fn enabled(config:&Configuration)->bool {
 cfg!(target_os="linux")&&config.local_execution().mode==LocalExecutionMode::Ask
}

struct Invocation {
 chat:Chat,settings:Settings,server:crate::local_chat::Server,selection:Selection,
 query:String,policy:LocalExecution,attachments:Vec<crate::pet_documents::Attachment>,
}
struct Prepared {reservation:ConversationReservation,input:Invocation,cid:String}
impl Prepared {
 fn prepare(chat:&Chat,settings:&Settings,config:&Configuration,query:&str,voice_stop:&AtomicBool)->Result<Self,String>{
  if voice_stop.load(Ordering::Acquire){return Err(CANCELLED.into());}
  if !enabled(config){return Err("Os pedidos de execução local estão desativados.".into());}
  if settings.chat_provider!="ollama"{return Err("O pedido de voz não pertence ao Ollama.".into());}
  if query.trim().is_empty()||query.len()>4000{return Err("O pedido de voz está vazio ou excede os limites.".into());}
  let policy=config.local_execution();policy.validate()?;
  let selected=crate::chat::model_for(settings,"ollama");
  if selected.is_empty(){return Err("Escolhe um modelo local Ollama.".into());}
  let selection=config.select("ollama",&selected,query)?;
  if selection.model.is_empty()||selection.model.ends_with(":cloud")||selection.model.ends_with("-cloud"){return Err("Escolhe um modelo local Ollama instalado.".into());}
  let server=crate::local_chat::server(settings,"ollama").ok_or("O Ollama está indisponível.")?;
  let base=crate::local_chat::base_url(&server)?;
  if !crate::net::is_loopback_url(&base){return Err("A voz Ollama usa apenas o servidor local.".into());}
  // No provider metadata, inference, command or file read precedes this owner CAS.
  let cid=chat.conversation_id("ollama")?;
  let reservation=chat.reserve_conversation("ollama",Some(&cid))?;
  if voice_stop.load(Ordering::Acquire){return Err(CANCELLED.into());}
  let attachments=crate::pet_documents::contexts("ollama",&cid);
  crate::pet_documents::validate_attachments(&attachments)?;
  let input=Invocation{chat:reservation.channel(),settings:settings.clone(),server,selection,query:query.into(),policy,attachments};
  Ok(Self{reservation,input,cid})
 }
 fn announce(self,app:&AppHandle)->Self{
  let Self{reservation,input,cid}=self;
  Self{reservation:reservation.broadcast(app,&input.query),input,cid}
 }
 async fn invoke_with<F,Fut>(self,voice_stop:&AtomicBool,run:F)->Result<ChatReply,String>
 where F:FnOnce(Invocation)->Fut,Fut:Future<Output=Result<ChatReply,String>> {
  let Self{reservation,input,cid}=self;
  if voice_stop.load(Ordering::Acquire){return Err(CANCELLED.into());}
  input.chat.check_admission("ollama")?;
  if input.chat.conversation_id("ollama")?!=cid{return Err(CANCELLED.into());}
  crate::pet_documents::validate_attachments(&input.attachments)?;
  // Acceptance point. Do not drop the runner when the microphone closes: its
  // existing task/turn cancellation, approvals and cleanup now own the work.
  let result=run(input).await;
  drop(reservation);
  result
 }
}

pub(super) fn respond(app:&AppHandle,settings:&Settings,config:&Configuration,query:&str,voice_stop:&AtomicBool)->Result<super::brain::Reply,String>{
 let chat=app.state::<Chat>();
 let prepared=Prepared::prepare(&chat,settings,config,query,voice_stop)?.announce(app);
 let reply=tauri::async_runtime::block_on(prepared.invoke_with(voice_stop,|input|async move{
  let context=crate::pet_documents::collection_context(&input.attachments);
  let result=crate::local_tasks::respond(app,&input.chat,&input.settings,&input.server,&input.selection.model,Some(&input.selection),input.query,context,input.policy,None).await?;
  Ok(result)
 }))?;
 // Commands report their exact outcomes through the existing task receipts.
 // Never synthesize the legacy empty-answer success tick from model prose.
 Ok(super::brain::Reply{text:super::brain::spoken_text(&reply.text),acted:false,language:None})
}

#[cfg(test)]
#[path="local_task_tests.rs"]
mod tests;
