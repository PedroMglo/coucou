//! Public progress only. Never exposes reasoning, commands, arguments or tokens.
use serde::Serialize;
use serde_json::Value;
use std::sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

static NEXT: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update { pub pet: String, pub turn: u64, pub phase: String, pub message: String, pub origin:String, #[serde(skip_serializing_if="Option::is_none")] pub target:Option<crate::session_chat::types::Target> }
pub fn pet(provider: &str) -> &'static str {
    crate::pet_config::integration_pet(provider).unwrap_or("")
}
struct Context { chat:crate::chat::Chat, turn:crate::chat::Turn, target:Option<crate::session_chat::types::Target>, origin:String, tracks_task:bool }
impl Context {
    fn capture(chat:crate::chat::Chat,turn:crate::chat::Turn,tracks_task:bool)->Self {
        let target=chat.usage_origin(&turn).and_then(|(cid,_)|crate::pet_config::integration_pet(&turn.provider).map(|id|crate::session_chat::types::Target{pet_id:id.into(),provider:turn.provider.clone(),origin:crate::session_chat::types::Origin::Mochi,session_id:cid.clone(),conversation_id:Some(cid)}));
        // A plain reply must never inherit the label of an earlier task.
        let origin=if tracks_task{chat.public_origin(&turn.provider)}else{target.as_ref().map(|t|format!("Mochi · #{}",t.session_id.chars().take(8).collect::<String>())).unwrap_or_else(||"Mochi".into())};
        Self{chat,turn,target,origin,tracks_task}
    }
    fn task_update(&self,phase:&str,message:&str)->bool {self.tracks_task&&self.chat.task_update(&self.turn,phase,message)}
}
pub struct Reporter { app: AppHandle, pet: String, turn: u64, last: Mutex<(String, String, Instant)>, done: AtomicBool, cancel: Option<Arc<AtomicBool>>, context: Option<Context> }
impl Reporter {
    pub fn new(app: &AppHandle, provider: &str, cancel: Option<Arc<AtomicBool>>) -> Self {
        let value = Self { app: app.clone(), pet: pet(provider).into(), turn: NEXT.fetch_add(1, Ordering::Relaxed), last: Mutex::new((String::new(),String::new(),Instant::now()-Duration::from_secs(1))), done: AtomicBool::new(false), cancel,context:None };
        value.update("thinking", ""); value
    }
    pub fn track(mut self,chat:crate::chat::Chat,turn:crate::chat::Turn)->Self {
        self.context=Some(Context::capture(chat,turn,true));let _=self.app.emit("assistant-tasks-changed",());self
    }
    /// Public conversation activity without creating or changing a task or receipt.
    pub fn observe(mut self,chat:crate::chat::Chat,turn:crate::chat::Turn)->Self {
        self.context=Some(Context::capture(chat,turn,false));self
    }
    pub fn update(&self, phase: &str, message: &str) {
        self.publish(phase,message,false);
    }
    fn publish(&self, phase:&str,message:&str,finishing:bool){
        let mut last = self.last.lock().unwrap();
        if !finishing&&self.done.load(Ordering::Relaxed){return;}
        let message = message.chars().take(4000).collect::<String>();
        if last.0 == phase && (last.1 == message || last.2.elapsed() < Duration::from_millis(80)) { return; }
        *last = (phase.into(), message.clone(), Instant::now());
        if let Some(context)=&self.context {if context.task_update(phase,&message){let _=self.app.emit("assistant-tasks-changed",());}}
        let origin=self.context.as_ref().map(|c|c.origin.clone()).unwrap_or_else(||"Mochi".into());
        let target=self.context.as_ref().and_then(|c|c.target.clone());
        let _ = self.app.emit_to(crate::companions::LABEL, "pet-activity", Update { pet: self.pet.clone(), turn: self.turn, phase: phase.into(), message,origin,target });
    }
    pub fn finish(&self, phase: &str, message: &str) { if !self.done.swap(true,Ordering::Relaxed){self.publish(phase,message,true);} }
    /// The reply and task are already committed together. Publish only; never
    /// mutate a journal that a newer turn may now own.
    pub fn finish_committed(&self,message:&str,origin:String){
        self.publish_committed_terminal(crate::local_tasks::types::LocalTerminal::Done,message,origin);
    }
    /// Called only after a matching plain reply committed. Persistence warnings
    /// remain visible; completion does not claim that the reply is durable.
    pub fn finish_observed(&self,message:&str,notice:Option<&str>){
        let public=observed_reply(message,notice);
        self.finish("done",&public);
    }
    pub(crate) fn publish_committed_terminal(&self,phase:crate::local_tasks::types::LocalTerminal,message:&str,origin:String){
        if self.done.swap(true,Ordering::Relaxed){return;}
        let message=message.chars().take(4000).collect::<String>();
        *self.last.lock().unwrap()=(phase.phase().into(),message.clone(),Instant::now());
        if self.context.as_ref().is_some_and(|c|c.tracks_task){let _=self.app.emit("assistant-tasks-changed",());}
        let target=self.context.as_ref().and_then(|c|c.target.clone());
        let _=self.app.emit_to(crate::companions::LABEL,"pet-activity",Update{pet:self.pet.clone(),turn:self.turn,phase:phase.phase().into(),message,origin,target});
    }
}
fn observed_reply(message:&str,notice:Option<&str>)->String {
    match notice {
        Some(notice)=>{let notice=notice.chars().take(1000).collect::<String>();let room=4000-notice.chars().count()-2;format!("{}\n\n{notice}",message.chars().take(room).collect::<String>())},
        None=>message.chars().take(4000).collect(),
    }
}
impl Drop for Reporter { fn drop(&mut self) { if !self.done.load(Ordering::Relaxed) { self.update(if self.cancel.as_ref().is_some_and(|s|s.load(Ordering::Relaxed)) { "stopped" } else { "failed" }, ""); } } }

/// Native event names carry state; only agentMessage text is public speech.
pub fn codex_progress(value: &Value) -> Option<(&'static str, String)> {
    let method=value["method"].as_str()?; let item=&value["params"]["item"];
    if matches!(method,"item/started"|"item/completed") {
        return match item["type"].as_str()? {
            "reasoning" => Some(("thinking",String::new())),
            "webSearch" => Some(("searching",String::new())),
            "commandExecution"|"fileChange"|"mcpToolCall"|"dynamicToolCall" => Some(("working",String::new())),
            "agentMessage" => Some(("message",item["text"].as_str().unwrap_or("").into())),
            _ => None,
        };
    }
    None
}

#[cfg(test)] mod tests {
    use super::*; use serde_json::json;
    #[test] fn plain_activity_cannot_modify_an_earlier_task_or_inherit_its_label() {
        let chat=crate::chat::Chat::default();
        let first=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&first);
        chat.start_task(&first,"previous task","/tmp","ollama","synthetic",None).unwrap();
        chat.commit_task_reply(&first,json!({"role":"user"}),json!({"role":"assistant"}),"previous task","done").unwrap();drop(lease);
        let before=serde_json::to_value(chat.tasks()).unwrap();
        let turn=chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=chat.lease(&turn);
        let context=Context::capture(chat.clone(),turn,false);
        for phase in ["thinking","message","working","done","failed","stopped"]{assert!(!context.task_update(phase,"plain reply"));}
        assert_eq!(serde_json::to_value(chat.tasks()).unwrap(),before);
        assert!(context.origin.starts_with("Mochi · #"));assert!(!context.origin.contains("previous task"));
    }
    #[test] fn public_target_is_captured_before_completion_and_never_follows_current_conversation() {
        let chat=crate::chat::Chat::default();let turn=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&turn);
        let context=Context::capture(chat.clone(),turn.clone(),false);let target=context.target.clone().unwrap();
        assert_eq!(target.pet_id,"ai_ollama");assert_eq!(target.provider,"ollama");assert_eq!(target.conversation_id.as_ref(),Some(&target.session_id));
        chat.commit_observed_reply(&turn,json!({"role":"user"}),json!({"role":"assistant"}),"question","answer").unwrap();drop(lease);
        chat.reset_provider("ollama").unwrap();let replacement=chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=chat.lease(&replacement);
        assert_ne!(chat.conversation_id("ollama").unwrap(),target.session_id);assert_eq!(context.target.as_ref(),Some(&target));
        assert!(chat.commit_observed_reply(&turn,json!({}),json!({}),"late","late").is_err());
        assert!(chat.tasks().is_empty());
    }
    #[test] fn service_target_keeps_its_logical_owner_and_unknown_provider_is_not_ollama() {
        let chat=crate::chat::Chat::default();let turn=chat.begin_cli("integration_notion",Arc::default()).unwrap();let _lease=chat.lease(&turn);
        let context=Context::capture(chat,turn,false);let target=context.target.unwrap();
        assert_eq!(target.pet_id,"integration_notion");assert_eq!(target.provider,"integration_notion");
        assert_eq!(pet("lmstudio"),"ai_lmstudio");assert_eq!(pet("unknown-provider"),"");
    }
    #[test] fn long_public_reply_keeps_the_persistence_warning_visible() {
        let message="á".repeat(5000);let public=observed_reply(&message,Some("Ainda não foi guardada em disco."));
        assert!(public.ends_with("Ainda não foi guardada em disco."));assert_eq!(public.chars().count(),4000);
    }
    #[test] fn progress_does_not_reveal_reasoning_or_tool_arguments() {
        assert_eq!(codex_progress(&json!({"method":"item/started","params":{"item":{"type":"reasoning","text":"private"}}})),Some(("thinking",String::new())));
        assert_eq!(codex_progress(&json!({"method":"item/started","params":{"item":{"type":"commandExecution","command":"secret"}}})),Some(("working",String::new())));
        assert_eq!(codex_progress(&json!({"method":"item/completed","params":{"item":{"type":"agentMessage","text":"Vou preparar a base de dados."}}})),Some(("message","Vou preparar a base de dados.".into())));
    }
}
