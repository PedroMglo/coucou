//! Public progress only. Never exposes reasoning, commands, arguments or tokens.
use serde::Serialize;
use serde_json::Value;
use std::sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

static NEXT: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update { pub pet: String, pub turn: u64, pub phase: String, pub message: String, pub origin:String }
pub fn pet(provider: &str) -> &'static str {
    match provider { "integration_resend" => "integration_resend", "integration_n8n" => "integration_n8n", "integration_vercel" => "integration_vercel", "integration_github" => "integration_github", "integration_notion" => "integration_notion", "integration_calcom" => "integration_calcom", "integration_stripe" => "integration_stripe", "integration_spotify" => "integration_spotify", "codex" => "agent_codex", "copilot"=>"agent_copilot", "antigravity" => "agent_antigravity", "gemini-cli" => "agent_gemini", "claude-cli" | "anthropic" => "integration_claude", _ => "ai_ollama" }
}
pub struct Reporter { app: AppHandle, pet: String, turn: u64, last: Mutex<(String, String, Instant)>, done: AtomicBool, cancel: Option<Arc<AtomicBool>>, tracking: Option<(crate::chat::Chat,crate::chat::Turn)> }
impl Reporter {
    pub fn new(app: &AppHandle, provider: &str, cancel: Option<Arc<AtomicBool>>) -> Self {
        let value = Self { app: app.clone(), pet: pet(provider).into(), turn: NEXT.fetch_add(1, Ordering::Relaxed), last: Mutex::new((String::new(),String::new(),Instant::now()-Duration::from_secs(1))), done: AtomicBool::new(false), cancel,tracking:None };
        value.update("thinking", ""); value
    }
    pub fn track(mut self,chat:crate::chat::Chat,turn:crate::chat::Turn)->Self {
        self.tracking=Some((chat,turn));let _=self.app.emit("assistant-tasks-changed",());self
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
        if let Some((chat,turn))=&self.tracking {if chat.task_update(turn,phase,&message){let _=self.app.emit("assistant-tasks-changed",());}}
        let origin=self.tracking.as_ref().map(|(chat,turn)|chat.public_origin(&turn.provider)).unwrap_or_else(||"Mochi".into());
        let _ = self.app.emit_to(crate::companions::LABEL, "pet-activity", Update { pet: self.pet.clone(), turn: self.turn, phase: phase.into(), message,origin });
    }
    pub fn finish(&self, phase: &str, message: &str) { if !self.done.swap(true,Ordering::Relaxed){self.publish(phase,message,true);} }
    /// The reply and task are already committed together. Publish only; never
    /// mutate a journal that a newer turn may now own.
    pub fn finish_committed(&self,message:&str,origin:String){
        if self.done.swap(true,Ordering::Relaxed){return;}
        let message=message.chars().take(4000).collect::<String>();
        *self.last.lock().unwrap()=("done".into(),message.clone(),Instant::now());
        let _=self.app.emit("assistant-tasks-changed",());
        let _=self.app.emit_to(crate::companions::LABEL,"pet-activity",Update{pet:self.pet.clone(),turn:self.turn,phase:"done".into(),message,origin});
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
    #[test] fn progress_does_not_reveal_reasoning_or_tool_arguments() {
        assert_eq!(codex_progress(&json!({"method":"item/started","params":{"item":{"type":"reasoning","text":"private"}}})),Some(("thinking",String::new())));
        assert_eq!(codex_progress(&json!({"method":"item/started","params":{"item":{"type":"commandExecution","command":"secret"}}})),Some(("working",String::new())));
        assert_eq!(codex_progress(&json!({"method":"item/completed","params":{"item":{"type":"agentMessage","text":"Vou preparar a base de dados."}}})),Some(("message","Vou preparar a base de dados.".into())));
    }
}
