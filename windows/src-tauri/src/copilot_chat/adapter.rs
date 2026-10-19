//! Thin application adapter. Register `copilot_adapter` when the shared provider
//! integration is applied; the transport itself remains independently testable.
use std::{path::Path,time::Duration};
use tauri::{AppHandle,Emitter};
use crate::{chat::{Chat,Turn,ModelInfo},pet_activity::Reporter};
use crate::copilot_chat::rpc::{Client,Phase,TurnOptions};

pub async fn models()->Result<Vec<ModelInfo>,String> {
    tauri::async_runtime::spawn_blocking(||{
        let exe=crate::copilot_chat::executable()?;
        let mut client=Client::start(&exe,&crate::platform::home_dir(),Duration::from_secs(30))?;
        client.models().map(|models|models.into_iter().map(|m|ModelInfo{id:m.id,label:m.label}).collect())
    }).await.map_err(|_|"A consulta de modelos Copilot foi interrompida.".to_string())?
}

pub fn answer(
    app:&AppHandle,chat:&Chat,turn:&Turn,activity:&Reporter,cwd:&Path,model:&str,effort:&str,
    system:&str,query:&str,session:Option<&str>,mut on_delta:impl FnMut(String),
)->Result<(String,bool),String> {
    let exe=crate::copilot_chat::executable()?;
    let mut client=Client::start_with_cancel(&exe,cwd,Duration::from_secs(1800),Some(turn.cancel.clone()))?;
    let cwd_text=cwd.to_string_lossy().to_string();
    let reply=client.answer(
        TurnOptions{model,effort,system,query,session_id:session,history:&turn.history},
        |id|{chat.remember_session(turn,id,&cwd_text,false);let _=app.emit("assistant-tasks-changed",());},
        |text|{activity.update("message",text.rsplit("\n\n").next().unwrap_or(&text));chat.publish_delta(app,turn,&text);on_delta(text);},
        |phase|activity.update(match phase{Phase::Thinking=>"thinking",Phase::Working=>"working",Phase::Approval=>"approval"},""),
        |params|{
            let tool=&params["toolCall"];
            let detail=serde_json::to_string_pretty(&serde_json::json!({"title":tool["title"],"kind":tool["kind"],"locations":tool["locations"],"input":tool["rawInput"]})).ok()?;
            // A truncated command is not a reviewable one-shot approval.
            if detail.len()>16*1024{return None;}
            let request=serde_json::json!({"command":detail,"sessionId":params["sessionId"]});
            crate::cli_chat::ask(app,&turn.provider,"session/request_permission",&request,&turn.cancel).map(|a|a["allow"]==true)
        },
    )?;
    if matches!(reply.stop_reason.as_str(),"max_tokens"|"max_turn_requests") {
        return Err("Copilot atingiu o limite do turno. A sessão própria foi preservada; verifica o resultado parcial antes de continuar.".into());
    }
    Ok((reply.text,reply.acted))
}
