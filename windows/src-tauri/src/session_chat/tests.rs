use super::{history,live::Store,types::{Target,Origin,Snapshot,Capabilities,Role,Message}};
use serde_json::{json,Value};
fn target()->Target{Target{pet_id:"agent_codex".into(),provider:"codex".into(),origin:Origin::External,session_id:"00000000-0000-0000-0000-000000000001".into(),conversation_id:None}}
fn snapshot()->Snapshot{Snapshot{target:target(),messages:vec![],busy:true,control_conflict:false,turn:Some("turn".into()),capabilities:Capabilities{history:true,send:true,cancel:true},notice:None,model:None}}
fn event(method:&str,params:Value)->Value{json!({"method":method,"params":params})}
fn item(method:&str,text:&str)->Value{event(method,json!({"threadId":target().session_id,"turnId":"turn","item":{"id":"reply","type":"agentMessage","text":text}}))}
fn delta(text:&str)->Value{event("item/agentMessage/delta",json!({"threadId":target().session_id,"turnId":"turn","itemId":"reply","delta":text}))}
#[test]fn a_child_thread_never_uses_its_parent_session_id_as_the_reply_target(){
 let t=target();let parent="00000000-0000-0000-0000-000000000002";
 let thread=json!({"id":parent,"sessionId":t.session_id,"status":{"type":"idle"}});
 assert!(history::snapshot(&thread,t,&[]).is_err());
}
#[test]fn public_messages_preserve_order_and_never_promote_reasoning_or_tool_results(){
 let turns=vec![json!({"id":"turn","status":"completed","itemsView":"full","items":[
  {"id":"user","type":"userMessage","content":[{"type":"text","text":"Olá"},{"type":"image","url":"private://path"}]},
  {"id":"reason","type":"reasoning","text":"private reasoning"},
  {"id":"tool","type":"commandExecution","aggregatedOutput":"private tool text"},
  {"id":"reply","type":"agentMessage","text":"Resposta pública"}
 ]})];
 let messages=history::messages(&turns).unwrap();assert_eq!(messages.len(),2);assert_eq!(messages[0].content,"Olá");assert_eq!(messages[1].content,"Resposta pública");
}
#[test]fn incomplete_turn_items_fail_instead_of_becoming_a_complete_conversation(){
 assert!(history::messages(&[json!({"id":"turn","status":"completed","itemsView":"summary","items":[]})]).is_err());
}
#[test]fn direct_input_requires_an_explicit_native_capability(){
 let t=target();let mut thread=json!({"id":t.session_id,"status":{"type":"idle"}});
 assert!(!history::snapshot(&thread,t.clone(),&[]).unwrap().capabilities.send);
 thread["canAcceptDirectInput"]=json!(true);assert!(history::snapshot(&thread,t.clone(),&[]).unwrap().capabilities.send);
 thread["ephemeral"]=json!(true);let view=history::snapshot(&thread,t,&[]).unwrap();assert!(!view.capabilities.history);assert!(!view.capabilities.send);
}
#[test]fn live_deltas_have_a_known_start_and_final_items_replace_them_without_duplication(){
 let mut live=Store::default();live.event(&item("item/started",""));live.event(&delta("Olá "));live.event(&delta("Portugal"));
 let view=live.merge(snapshot());assert_eq!(view.messages[0].content,"Olá Portugal");assert!(view.notice.is_none());
 live.event(&item("item/completed","Olá Portugal!"));let mut saved=snapshot();saved.messages.push(Message{id:serde_json::to_string(&("turn","reply")).unwrap(),role:Role::Assistant,content:"Olá Portugal!".into()});
 let view=live.merge(saved);assert_eq!(view.messages.len(),1);assert_eq!(view.messages[0].content,"Olá Portugal!");
}
#[test]fn joining_mid_reply_explains_the_missing_prefix_until_the_authoritative_item_arrives(){
 let mut live=Store::default();live.event(&delta("suffix"));let view=live.merge(snapshot());assert!(view.notice.unwrap().contains("meio"));assert_eq!(view.messages[0].content,"suffix");
 live.event(&item("item/completed","complete prefix and suffix"));let view=live.merge(snapshot());assert_eq!(view.messages[0].content,"complete prefix and suffix");assert!(view.notice.is_none());
}
#[test]fn one_threads_live_message_cannot_enter_another_threads_snapshot(){
 let mut live=Store::default();live.event(&item("item/started","belongs to A"));let mut b=snapshot();b.target.session_id="00000000-0000-0000-0000-000000000002".into();assert!(live.merge(b).messages.is_empty());
}
#[test]fn idle_observers_are_released_before_the_stream_limit_blocks_future_sessions(){
 let mut live=Store::default();for n in 1..260{
  let id=format!("00000000-0000-0000-0000-{n:012}");
  live.event(&event("thread/status/changed",json!({"threadId":id,"status":{"type":"idle"}})));
  let mut view=snapshot();view.target.session_id=id;view.busy=false;view.turn=None;assert!(live.merge(view).notice.is_none());
 }
 live.event(&item("item/started","new live answer"));assert_eq!(live.merge(snapshot()).messages[0].content,"new live answer");
}

#[tokio::test]
async fn a_resolved_approval_is_discarded_in_the_writer_without_breaking_the_connection(){
 use std::{sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}},os::unix::fs::PermissionsExt};
 use futures_util::{SinkExt,StreamExt};
 let folder=std::env::temp_dir().join(format!("mochi-session-rpc-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir_all(&folder).unwrap();
 let path=folder.join("control.sock");let listener=tokio::net::UnixListener::bind(&path).unwrap();std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o600)).unwrap();
 let written=Arc::new(AtomicUsize::new(0));let observed=written.clone();
 let server=tokio::spawn(async move{
  let(stream,_)=listener.accept().await.unwrap();let mut ws=tokio_tungstenite::accept_async(stream).await.unwrap();
  while let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))=ws.next().await{
   let request:Value=serde_json::from_str(&text).unwrap();
   if request.get("method").is_none(){observed.fetch_add(1,Ordering::SeqCst);continue;}
   if request.get("id").is_some(){ws.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":request["id"],"result":{}}).to_string().into())).await.unwrap();}
  }
 });
 let client=super::rpc::Client::connect(&path).await.unwrap();
 assert!(client.respond_guarded(json!("old-request"),json!({"decision":"accept"}),Arc::new(AtomicBool::new(true))).await.is_err());
 client.call("thread/read",json!({})).await.unwrap();assert_eq!(written.load(Ordering::SeqCst),0);assert!(client.connected());
 drop(client);server.abort();let _=std::fs::remove_dir_all(folder);
}
