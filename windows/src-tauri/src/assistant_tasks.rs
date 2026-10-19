//! Owned assistant jobs. Restoring a journal never starts or replays a command.
use serde::{Deserialize,Serialize};
use tauri::{AppHandle,Emitter,Manager};
use crate::chat::{Chat,ChatReply};

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Task {
    pub id:String,pub provider:String,pub backend:String,pub model:String,
    pub query:String,pub cwd:String,pub phase:String,pub progress:String,pub result:String,
    pub started:u64,pub updated:u64,pub finished:Option<u64>,
    #[serde(default)] pub resumes:Option<String>,
    #[serde(default)] pub resumed_by:Option<String>,
    #[serde(default)] pub conversation_id:Option<String>,
    #[serde(default)] pub session_id:Option<String>,
}
fn now()->u64 {std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64}
impl Task {
    pub fn new(provider:&str,query:&str,cwd:&str,backend:&str,model:&str,epoch:u64)->Self {
        let stamp=now();let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
        Self{id:format!("{nonce:x}-{epoch:x}"),provider:provider.into(),backend:backend.into(),model:model.into(),query:query.into(),cwd:cwd.into(),phase:"thinking".into(),progress:String::new(),result:String::new(),started:stamp,updated:stamp,finished:None,resumes:None,resumed_by:None,conversation_id:None,session_id:None}
    }
    pub fn terminal(&self)->bool {matches!(self.phase.as_str(),"done"|"failed"|"stopped"|"interrupted"|"resumed")}
    pub fn update(&mut self,phase:&str,message:&str) {
        self.phase=phase.into();self.updated=now();
        if !message.is_empty() {self.progress=message.chars().take(1000).collect();}
        if phase=="done" {self.result=message.chars().take(8000).collect();}
        if self.terminal() {self.finished=Some(self.updated);}
    }
    pub fn recover(&mut self) {
        if !self.terminal() {
            self.phase="interrupted".into();self.finished=None;
            self.progress="A aplicação reiniciou. O estado real da execução precisa de ser verificado antes de continuar.".into();
        }
    }
    pub fn bound(&mut self) {
        for (value,max) in [(&mut self.id,128),(&mut self.provider,32),(&mut self.backend,32),(&mut self.model,256),(&mut self.cwd,4096),(&mut self.query,4000),(&mut self.progress,1000),(&mut self.result,8000)] {*value=value.chars().take(max).collect();}
        for value in [&mut self.resumes,&mut self.resumed_by,&mut self.conversation_id,&mut self.session_id] {if let Some(text)=value {*text=text.chars().take(512).collect();}}
        if !matches!(self.phase.as_str(),"thinking"|"working"|"searching"|"message"|"approval"|"question"|"stopping"|"done"|"failed"|"stopped"|"interrupted"|"resumed") {self.phase="interrupted".into();}
    }
    pub fn valid(&self)->bool {
        match self.backend.as_str() {"copilot"=>self.provider=="copilot","codex"=>self.provider=="codex" || crate::pet_config::service(&self.provider),"antigravity"=>matches!(self.provider.as_str(),"antigravity"|"gemini-cli"|"claude-cli"),_=>false}
    }
    pub fn resume_prompt(&self)->Result<String,String> {
        if !self.valid() || !matches!(self.phase.as_str(),"interrupted"|"stopped"|"failed") {return Err("Esta tarefa não está disponível para retomar.".into());}
        Ok(format!("Verifica primeiro o estado real desta tarefa no computador, incluindo operações que possam ter continuado em segundo plano. Depois continua apenas o que falta. Não repitas operações já concluídas. Mantém as políticas nativas de permissões e as instruções AGENTS.md. Pasta anterior: {:?}. Pedido original: {}",self.cwd,self.query))
    }
}

fn resume_settings(task:&Task,source:&crate::settings::Settings,cwd:Option<String>)->Result<crate::settings::Settings,String> {
    let folder=match cwd {
        Some(path) if path.trim().is_empty()=>return Err("Indica a pasta para retomar esta tarefa.".into()),
        Some(path)=>path,None if task.cwd.trim().is_empty()=>return Err("Escolhe a pasta original desta tarefa antes de a retomar.".into()),
        None=>task.cwd.clone(),
    };
    let mut settings=source.clone();settings.cli_workspaces.insert(task.provider.clone(),folder);
    // Validate before the previous record is retired or a model is contacted.
    crate::cli_chat::workspace_for(&settings,&task.provider)?;Ok(settings)
}

#[tauri::command]
pub async fn assistant_tasks(app:AppHandle)->Result<Vec<Task>,String> {
    let chat=app.state::<Chat>().inner().clone();
    tauri::async_runtime::spawn_blocking(move||chat.tasks()).await
        .map_err(|_|"Não foi possível consultar os trabalhos.".to_string())
}

#[tauri::command]
pub async fn assistant_cancel_task(app:AppHandle,id:String)->Result<bool,String> {
    let chat=app.state::<Chat>().inner().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let cancelled=chat.cancel_task(&id);if cancelled {let _=app.emit("assistant-tasks-changed",());}cancelled
    }).await.map_err(|_|"Não foi possível parar o trabalho.".to_string())
}

#[tauri::command]
pub async fn assistant_resume_task(app:AppHandle,id:String,cwd:Option<String>)->Result<ChatReply,String> {
    let chat=app.state::<Chat>().inner().clone();
    let source=app.state::<crate::Shared>().settings.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move|| {
        let before=chat.tasks().into_iter().find(|t|t.id==id).ok_or("A tarefa já não está no histórico local.")?;
        // Invalid folders must not switch the user's conversation or retire a task.
        let settings=resume_settings(&before,&source,cwd)?;
        let(task,reservation,changed)=chat.reserve_task_resume(&id)?;
        let query=task.resume_prompt()?;
        if changed{crate::pet_documents::invalidate(&app,Some(&task.provider));let _=app.emit("assistant-history-changed",&task.provider);}
        let reservation=reservation.broadcast(&app,&query);let channel=reservation.channel();
        crate::cli_chat::run(&app,&channel,&settings,&task.backend,&task.provider,&task.model,&query,None,false,&crate::voice::brain::tools(),None,Some(&task.id),|_|{}).map(|(text,_)|ChatReply{text})
    }).await.map_err(|_|"O assistente parou inesperadamente.".to_string())?
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn recovery_is_explicit_and_does_not_claim_old_effects_were_undone(){
        let mut t=Task::new("codex","prepara o postgres","/tmp/project","codex","model",1);t.update("approval","");t.recover();
        assert_eq!(t.phase,"interrupted");assert!(t.finished.is_none());
        let prompt=t.resume_prompt().unwrap();assert!(prompt.contains("Verifica primeiro"));assert!(prompt.contains("Não repitas"));
        t.update("done","Pronto");assert!(t.resume_prompt().is_err());
    }
    #[test]fn task_resume_cannot_change_its_backend_or_claim_an_unimplemented_account(){
        let mut t=Task::new("codex","pedido","/tmp","antigravity","model",1);t.recover();assert!(t.resume_prompt().is_err());
        t.provider="gemini-cli".into();assert!(t.resume_prompt().is_ok());
    }
    #[test]fn recovery_uses_the_original_folder_or_an_explicit_valid_replacement_without_changing_preferences(){
        let tmp=std::env::temp_dir().to_string_lossy().to_string();let home=crate::platform::home_dir().to_string_lossy().to_string();
        let mut task=Task::new("codex","pedido",&tmp,"codex","model",1);task.recover();
        let mut source=crate::settings::Settings::default();source.cli_workspaces.insert("codex".into(),home.clone());
        assert_eq!(resume_settings(&task,&source,None).unwrap().cli_workspaces["codex"],tmp);
        assert_eq!(resume_settings(&task,&source,Some(home.clone())).unwrap().cli_workspaces["codex"],home);
        assert!(resume_settings(&task,&source,Some("relative/folder".into())).is_err());assert!(resume_settings(&task,&source,Some(String::new())).is_err());
        assert_eq!(source.cli_workspaces["codex"],home);assert_eq!(task.phase,"interrupted");assert!(task.resumed_by.is_none());assert!(!source.cli_full_access);
        task.cwd=" ".into();assert!(resume_settings(&task,&source,None).is_err());
        assert_eq!(resume_settings(&task,&source,Some(tmp.clone())).unwrap().cli_workspaces["codex"],tmp);
    }

    #[test]fn stable_conversation_origin_is_optional_for_legacy_rows_and_uses_camel_case(){
        let task=Task::new("codex","synthetic","/tmp","codex","fixture",1);
        let mut legacy=serde_json::to_value(task).unwrap();legacy.as_object_mut().unwrap().remove("conversationId");
        let mut decoded:Task=serde_json::from_value(legacy).unwrap();
        assert!(decoded.conversation_id.is_none());assert!(decoded.session_id.is_none());
        decoded.conversation_id=Some("synthetic-public-origin".into());
        let value=serde_json::to_value(&decoded).unwrap();
        assert_eq!(value["conversationId"],"synthetic-public-origin");assert!(value.get("conversation_id").is_none());assert!(value["sessionId"].is_null());
        decoded.conversation_id=Some("x".repeat(600));decoded.bound();
        assert_eq!(decoded.conversation_id.unwrap().len(),512);
    }

}
