#![cfg(target_os="linux")]
use super::*;
use std::{cell::Cell,sync::Arc};
use serde_json::json;
fn settings()->Settings{
 let mut s=Settings::default();s.chat_provider="ollama".into();s.ollama_url="http://127.0.0.1:9".into();s.chat_models.insert("ollama".into(),"synthetic-model".into());s
}
fn stopped()->AtomicBool{AtomicBool::new(false)}
#[test]fn explicit_task_address_preserves_the_existing_ordinary_voice_routes(){
 for query in ["tarefa local: verifica a pasta","  TAREFA LOCAL prepara um resumo","Local task: inspect this folder"]{assert!(requested(query),"{query}");}
 for query in ["tarefa local","tarefa local: ","tarefas locais: resumo","a tarefa local terminou?","toca música","adiciona o Gravity","prepara um email","abre o browser","o que estão a fazer os pets?","olá"]{assert!(!requested(query),"{query}");}
}
#[test]fn only_explicit_ask_for_the_local_ollama_owner_is_admitted(){
 let chat=Chat::default();let s=settings();let mut c=crate::pet_config::defaults();assert!(enabled(&c));
 c.pets.get_mut("ai_ollama").unwrap().local_execution.as_mut().unwrap().mode=LocalExecutionMode::Disabled;
 assert!(!enabled(&c));assert!(Prepared::prepare(&chat,&s,&c,"olá",&stopped()).is_err());
 c=crate::pet_config::defaults();
 for owner in ["codex","copilot","antigravity","integration_github"]{let mut wrong=settings();wrong.chat_provider=owner.into();assert!(Prepared::prepare(&chat,&wrong,&c,"olá",&stopped()).is_err());}
 for model in ["synthetic:cloud","synthetic-cloud"]{let mut wrong=settings();wrong.chat_models.insert("ollama".into(),model.into());assert!(Prepared::prepare(&chat,&wrong,&c,"olá",&stopped()).is_err());}
 // An absent UI choice intentionally resolves the pet's configured default,
 // exactly as written chat does; it must not invent another model.
 let mut default=settings();default.chat_models.remove("ollama");
 let expected=c.select("ollama","@default","olá").unwrap();
 let prepared=Prepared::prepare(&chat,&default,&c,"olá",&stopped()).unwrap();
 assert_eq!(prepared.input.selection.model,expected.model);drop(prepared);
 for url in ["http://192.0.2.1:9","https://example.invalid"]{let mut wrong=settings();wrong.ollama_url=url.into();assert!(Prepared::prepare(&chat,&wrong,&c,"olá",&stopped()).is_err());}
 for query in [String::new()," ".into(),"a".repeat(4001)]{assert!(Prepared::prepare(&chat,&s,&c,&query,&stopped()).is_err());}
 assert!(!chat.conversation("ollama").unwrap().busy);assert!(chat.tasks().is_empty());
}
#[tokio::test]async fn model_profile_and_exact_query_are_captured_without_borrowing_later_settings(){
 let chat=Chat::default();let mut s=settings();let mut c=crate::pet_config::defaults();
 c.pets.get_mut("ai_ollama").unwrap().fast.model="synthetic-fast".into();c.pets.get_mut("ai_ollama").unwrap().reasoning.model="synthetic-deep".into();
 s.chat_models.insert("ollama".into(),"@default".into());let query="pensa a fundo: mantém PT/中文\nsem duplicar";let cid=chat.conversation_id("ollama").unwrap();
 let p=Prepared::prepare(&chat,&s,&c,query,&stopped()).unwrap();assert_eq!(p.cid,cid);
 s.chat_models.insert("ollama".into(),"later-model".into());c.pets.get_mut("ai_ollama").unwrap().reasoning.model="later-profile".into();
 let reply=p.invoke_with(&stopped(),|input|async move{
  assert_eq!(input.query,query);assert_eq!(input.selection.model,"synthetic-deep");assert_eq!(input.selection.mode,"reasoning");assert_eq!(input.settings.chat_models["ollama"],"@default");assert_eq!(input.server.id,"ollama");
  let turn=input.chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=input.chat.lease(&turn);assert!(turn.history.is_empty());
  input.chat.commit_reply(&turn,json!({"role":"user","content":input.query}),json!({"role":"assistant","content":"synthetic reply"}),query,"synthetic reply").unwrap();Ok(ChatReply{text:"synthetic reply".into()})
 }).await.unwrap();assert_eq!(reply.text,"synthetic reply");assert_eq!(chat.conversation_id("ollama").unwrap(),cid);assert_eq!(chat.history("ollama").len(),2);assert_eq!(chat.history("ollama")[0]["content"],query);assert!(chat.history("codex").is_empty());
}
#[test]fn a_busy_ollama_is_rejected_and_another_pets_reservation_is_independent(){
 let chat=Chat::default();let s=settings();let c=crate::pet_config::defaults();let _codex=chat.reserve_conversation("codex",None).unwrap();
 let p=Prepared::prepare(&chat,&s,&c,"olá",&stopped()).unwrap();let cid=p.cid.clone();
 let error=Prepared::prepare(&chat,&s,&c,"outro",&stopped()).err().unwrap();assert_eq!(error,crate::chat::BUSY_ERROR);assert_eq!(chat.conversation_id("ollama").unwrap(),cid);
 drop(p);assert!(!chat.conversation("ollama").unwrap().busy);
}
#[tokio::test]async fn microphone_stop_before_acceptance_never_invokes_the_runner(){
 let chat=Chat::default();let s=settings();let c=crate::pet_config::defaults();let voice=stopped();voice.store(true,Ordering::Release);
 assert!(Prepared::prepare(&chat,&s,&c,"olá",&voice).is_err());voice.store(false,Ordering::Release);
 let p=Prepared::prepare(&chat,&s,&c,"olá",&voice).unwrap();let called=Cell::new(false);voice.store(true,Ordering::Release);
 let result=p.invoke_with(&voice,|_|{called.set(true);async{Ok(ChatReply{text:"not reached".into()})}}).await;
 assert!(result.is_err());assert!(!called.get());assert!(!chat.conversation("ollama").unwrap().busy);assert!(chat.history("ollama").is_empty());assert!(chat.tasks().is_empty());
}
#[tokio::test]async fn cancelled_admission_cannot_dispatch_or_move_to_a_different_conversation(){
 let chat=Chat::default();let c=crate::pet_config::defaults();let p=Prepared::prepare(&chat,&settings(),&c,"olá",&stopped()).unwrap();let view=chat.conversation("ollama").unwrap();
 assert!(chat.reset_provider("ollama").is_err());assert!(chat.cancel_conversation("ollama",Some(&view.id),Some(view.turn)));
 let called=Cell::new(false);let result=p.invoke_with(&stopped(),|_|{called.set(true);async{Ok(ChatReply{text:"not reached".into()})}}).await;
 assert!(result.is_err());assert!(!called.get());assert_eq!(chat.conversation_id("ollama").unwrap(),view.id);assert!(chat.history("ollama").is_empty());
}
#[tokio::test]async fn accepted_work_outlives_microphone_stop_and_explicit_task_stop_owns_cancellation(){
 let chat=Chat::default();let voice=stopped();let c=crate::pet_config::defaults();let p=Prepared::prepare(&chat,&settings(),&c,"pedido exato",&voice).unwrap();
 let microphone=&voice;
 let result=p.invoke_with(&voice,|input|async move{
  microphone.store(true,Ordering::Release);
  let turn=input.chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=input.chat.lease(&turn);
  assert!(!turn.cancel.load(Ordering::Acquire));assert!(input.chat.is_current(&turn));
  input.chat.start_local_task(&turn,&input.query,"/synthetic","synthetic-model",None,None).unwrap();let(task,cid)=input.chat.local_task_identity(&turn).unwrap();
  assert_eq!(input.chat.tasks()[0].query,"pedido exato");assert_eq!(input.chat.tasks()[0].conversation_id.as_deref(),Some(cid.as_str()));
  assert!(input.chat.cancel_task(&task));assert!(turn.cancel.load(Ordering::Acquire));assert!(microphone.load(Ordering::Acquire));
  assert_eq!(input.chat.tasks()[0].phase,"stopping");Err("synthetic task interrupted".into())
 }).await;assert!(result.is_err());assert!(chat.history("ollama").is_empty());
}
