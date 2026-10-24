// « OK Coucou »'s brain: Claude with the user's key and Coucou's tools —
// pills, music, the user's own data, opening an app, preparing an email (the
// mail app opens with it filled in; the user sends it) and Anthropic's web
// search. The conversation lives in memory only and is forgotten 90 s after
// the last turn. Port of ClaudeVoiceBrain.swift.

use std::time::{Duration, Instant};
use std::sync::atomic::AtomicBool;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

const FAST_MODEL: &str = "claude-haiku-4-5";
const FORGET_AFTER: Duration = Duration::from_secs(90);

pub struct Brain {
    messages: Vec<Value>,
    local: super::local::Brain,
    last: Instant,
    family_models: std::collections::HashMap<String,String>,
}

impl Default for Brain {
    fn default() -> Self {
        Self { messages: Vec::new(), local: Default::default(), last: Instant::now(), family_models: Default::default() }
    }
}

pub struct Reply {
    pub text: String,
    pub acted: bool,
    pub language: Option<String>,
}

pub(crate) fn tools() -> Vec<Value> {
    let mut tools = crate::pet_operations::definitions();
    tools.push(json!({"type":"web_search_20250305","name":"web_search","max_uses":3}));
    tools
}

fn system_prompt(pills: &str) -> String {
    let t = crate::platform::local_time();
    let os = if cfg!(windows) { "Windows" } else { "Linux" };
    format!(
        "You are Coucou, a voice assistant living at the top of the screen of the user's {os} computer, a bit like Jarvis: \
quick, warm, a little playful. It is {:04}-{:02}-{:02} {:02}:{:02}.\n\
Everything you write is read aloud: answer in the language the user spoke, in one or two short sentences, \
no markdown, no lists, no emojis, no links.\n\
Act with the tools instead of describing: pills, music, service_info for the user's own data, open_app, \
prepare_email, and web_search for anything current (weather, news, scores, prices, facts you are not sure of). \
Never invent the user's data.\n\
Email: gather the recipient's address, the subject and what to say (write the text yourself from what the user \
wants to say, in the language they used), asking for what is missing one short question at a time, then call \
prepare_email. The email is never sent by you: the user checks it in their mail app and sends it.\n\
When you need an answer from the user, end with one short question. Otherwise never end with a question.\n\
Pills: {pills}",
        t.year, t.month, t.day, t.hour, t.minute
    )
}

impl Brain {
    /// The legacy hotword conversation keeps its original retention policy.
    /// CLI pets use the shared assistant memory across microphone sessions.
    pub fn forget_if_idle(&mut self) {
        if self.last.elapsed() > FORGET_AFTER {
            self.messages.clear();
            self.local.clear();
        }
    }

    /// The spoken pet belongs to this dialogue, independent of the text chat's
    /// provider. CLI context belongs to the logical pet in the shared Chat store.
    pub fn respond_as(&mut self, app: &AppHandle, transcript: &str, provider: &str, stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>) -> Option<Reply> {
        self.last = Instant::now();
        #[cfg(target_os="linux")]if let Some(text)=crate::cli_chat::goal_recipe_voice(app,provider,transcript,stop.as_deref().unwrap_or(&AtomicBool::new(false))){return Some(Reply{text,acted:false,language:None});}
        let settings = app.try_state::<crate::Shared>().map(|s| {
            crate::cli_chat::voice_settings(&s.settings.lock().unwrap(), provider)
        });
        if let Some(settings) = settings.as_ref().filter(|s| s.chat_provider == "ollama") {
            self.messages.clear();
            let not_stopped = AtomicBool::new(false);
            // Additive explicit route. Ordinary voice keeps its native music,
            // pills, service data, app-opening and prepare-email actions.
            if super::local_task::requested(transcript){
                let config=match crate::pet_config::read_existing(){
                    Ok(config)=>config,
                    Err(text)=>return Some(Reply{text,acted:false,language:None}),
                };
                return Some(match super::local_task::respond(app,settings,&config,transcript,stop.as_deref().unwrap_or(&not_stopped)){
                    Ok(reply)=>reply,
                    Err(text)=>Reply{text,acted:false,language:None},
                });
            }
            let activity = crate::pet_activity::Reporter::new(app, provider, stop.clone());
            activity.update("working", "");
            let reply = self.local.respond(app, settings, transcript, stop.as_deref().unwrap_or(&not_stopped));
            if let Some(reply) = &reply { activity.finish("done", &reply.text); }
            return reply;
        }
        self.local.clear();
        if let Some(settings) = settings.clone().filter(|s| crate::cli_chat::backend(&s.chat_provider).is_some()) {
            let chat=app.state::<crate::chat::Chat>();
            let id=match chat.conversation_id(provider){Ok(id)=>id,Err(text)=>return Some(Reply{text,acted:false,language:None})};
            let reservation=match chat.reserve_conversation(provider,Some(&id)){Ok(r)=>r.broadcast(app,transcript),Err(text)=>return Some(Reply{text,acted:false,language:None})};
            let channel=reservation.channel();
            let settings=app.try_state::<crate::Shared>().map(|s|crate::cli_chat::voice_settings(&s.settings.lock().unwrap(),provider)).unwrap_or(settings);
            let attachments=crate::pet_documents::contexts(provider,&id);
            if let Err(text)=crate::pet_documents::validate_attachments(&attachments){return Some(Reply{text,acted:false,language:None});}
            let context=crate::pet_documents::collection_context(&attachments);
            let selected=crate::chat::model_for(&settings,provider);
            let selected=if selected.is_empty() {self.family_models.get(provider).cloned().unwrap_or_default()} else {selected};
            let model=tauri::async_runtime::block_on(super::until_stopped(crate::cli_chat::resolve_model(&settings,provider,&selected),stop.as_deref()))?;
            let model=match model {
                Ok(model)=>model,
                Err(err)=>return Some(Reply{text:err,acted:false,language:None}),
            };
            if matches!(provider,"gemini-cli"|"claude-cli") {self.family_models.insert(provider.into(),model.clone());}
            let backend=crate::cli_chat::backend(provider)?;
            if let Err(text)=crate::pet_documents::validate_attachments(&attachments){return Some(Reply{text,acted:false,language:None});}
            return match crate::cli_chat::run(app, &channel, &settings, backend, provider, &model, transcript, context.as_ref(), true, &tools(), stop.clone(), None, |_| {}) {
                Ok((text, acted)) => {
                    Some(Reply { text: spoken_text(&text), acted, language: None })
                }
                Err(err) => {
                    if err==crate::chat::BUSY_ERROR {return Some(Reply{text:err,acted:false,language:None});}
                    if err=="The CLI request was cancelled."||stop.as_ref().is_some_and(|s|s.load(std::sync::atomic::Ordering::Relaxed)) {
                        return Some(Reply{text:"A tarefa foi interrompida.".into(),acted:false,language:None});
                    }
                    crate::log::line(format!("voice: CLI turn failed: {err}"));
                    Some(Reply { text: crate::i18n::t("The CLI agent could not answer. Check its connection in Settings → Chat."), acted: false, language: None })
                }
            };
        }
        if !matches!(provider, "" | "anthropic") { return None; }
        self.messages.push(json!({ "role": "user", "content": transcript }));
        self.trim();
        let pills = super::ask_island(app, "pills", json!({ "action": "list" })).unwrap_or_default();
        let system = system_prompt(&pills);
        let tools = tools();
        let chat_model = app
            .try_state::<crate::Shared>()
            .map(|s| s.settings.lock().unwrap().model.clone())
            .unwrap_or_default();
        let mut acted = false;
        let mut spoken = String::new();
        for _ in 0..5 {
            let turn = tauri::async_runtime::block_on(super::until_stopped(async {
                match crate::claude::voice_turn(FAST_MODEL, &system, &self.messages, &tools).await {
                    Ok(v) => Ok(v),
                    Err(err) if !chat_model.is_empty() => {
                        crate::log::line(format!("voice: {err}; the chat's model instead"));
                        crate::claude::voice_turn(&chat_model, &system, &self.messages, &tools).await
                    }
                    Err(err) => Err(err),
                }
            }, stop.as_deref()))?;
            let turn = match turn {
                Ok(t) => t,
                Err(err) => {
                    crate::log::line(format!("voice: Claude turn failed: {err}"));
                    if self.messages.last().is_some_and(|m| m["content"].is_string()) {
                        self.messages.pop();
                    }
                    return acted.then(|| Reply { text: String::new(), acted, language: None });
                }
            };
            let content = turn["content"].as_array().cloned().unwrap_or_default();
            self.messages.push(json!({ "role": "assistant", "content": content.clone() }));
            let text: String = content.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join(" ");
            if !text.trim().is_empty() {
                spoken = text;
            }
            let reason = turn["stop_reason"].as_str().unwrap_or("end_turn");
            if reason == "pause_turn" {
                continue;
            }
            let uses: Vec<&Value> = content.iter().filter(|b| b["type"] == "tool_use").collect();
            if reason != "tool_use" || uses.is_empty() {
                break;
            }
            let mut results = Vec::new();
            for u in uses {
                if stop.as_ref().is_some_and(|s| s.load(std::sync::atomic::Ordering::Relaxed)) { self.repair(); return None; }
                let (result, did) = super::run_tool_for(app,"ai_anthropic",u["name"].as_str().unwrap_or(""),&u["input"],||!stop.as_ref().is_some_and(|s|s.load(std::sync::atomic::Ordering::Acquire)));
                acted |= did;
                results.push(json!({ "type": "tool_result", "tool_use_id": u["id"], "content": result }));
            }
            self.messages.push(json!({ "role": "user", "content": results }));
            spoken.clear();
        }
        self.repair();
        Some(Reply { text: spoken_text(&spoken), acted, language: None })
    }

    /// Keeps the last exchanges, never cutting between a tool call and its result.
    fn trim(&mut self) {
        if self.messages.len() <= 24 {
            return;
        }
        let mut cut = self.messages.len() - 24;
        while cut < self.messages.len() && !(self.messages[cut]["role"] == "user" && self.messages[cut]["content"].is_string()) {
            cut += 1;
        }
        self.messages.drain(..cut.min(self.messages.len() - 1));
    }

    /// The API refuses a history ending on a tool call with no result after it.
    fn repair(&mut self) {
        let Some(last) = self.messages.last() else { return };
        if last["role"] != "assistant" {
            return;
        }
        let calls = last["content"].as_array().is_some_and(|c| c.iter().any(|b| b["type"] == "tool_use"));
        if calls {
            self.messages.pop();
        }
    }
}

/// What Claude wrote, ready to be said: citations and markdown left out.
pub fn spoken_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut skip = 0;
    for c in text.chars() {
        match c {
            '[' => skip += 1,
            ']' if skip > 0 => skip -= 1,
            '*' | '#' | '`' | '_' if skip == 0 => {}
            _ if skip == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_and_citations_are_not_read_aloud() {
        assert_eq!(spoken_text("**Done** — Stripe [1] is in the island."), "Done — Stripe is in the island.");
        assert_eq!(spoken_text("  Il fait  20 °C à Bordeaux. "), "Il fait 20 °C à Bordeaux.");
    }

    #[test]
    fn every_tool_has_a_schema() {
        for tool in tools() {
            assert!(tool["name"].is_string());
            assert!(tool.get("input_schema").is_some() || tool["type"] == "web_search_20250305");
        }
    }
}
