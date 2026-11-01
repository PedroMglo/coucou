// Chat with a model server the user runs: Ollama, LM Studio, or any other
// server that speaks the OpenAI API (vLLM, llama.cpp, Unsloth…) — the same
// integration as LocalChat.swift, plus that last one.
//
// The answer is streamed token by token. Each step goes to the island as a
// `chat-delta` event carrying the text visible so far, and the island shows it
// growing; the reply of the command is the finished text. `<think>` blocks of
// reasoning models stay hidden while open and are dropped from the answer.
//
// Nothing leaves the machine unless the user pointed the server address
// somewhere else: that address is the only place this module talks to.

use std::time::{Duration, Instant};
#[path="lmstudio.rs"]pub(crate)mod lmstudio;
use std::sync::{Mutex,atomic::{AtomicU64,Ordering}};
static LMSTUDIO_KEY_GENERATION:AtomicU64=AtomicU64::new(2);
static LMSTUDIO_KEY_EDIT:Mutex<()>=Mutex::new(());
pub(crate)fn lmstudio_key_generation()->u64{LMSTUDIO_KEY_GENERATION.load(Ordering::Acquire)}
fn edit_lmstudio_key(operation:impl FnOnce()->Result<(),String>)->Result<(),String>{
    let _guard=LMSTUDIO_KEY_EDIT.lock().map_err(|_|"O cofre está ocupado.")?;
    LMSTUDIO_KEY_GENERATION.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n|n.checked_add(2).map(|_|n+1)).map_err(|_|"Limite de credenciais atingido.")?;
    let result=operation();LMSTUDIO_KEY_GENERATION.fetch_add(1,Ordering::AcqRel);result
}
pub(crate)fn clear_lmstudio_key()->Result<(),String>{edit_lmstudio_key(||secrets::clear(LMSTUDIO_KEY))}

use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::chat::{self, Chat, ChatContext, ChatReply, ModelInfo};
use crate::settings::Settings;
use crate::pet_observability::capture::{Capture, Origin as CaptureOrigin};
use crate::pet_observability::usage::AttemptOutcome;
use crate::i18n::{t, tf};
use crate::{net, secrets};

/// Credential store entry of the key of the user's own OpenAI-compatible server.
pub const CUSTOM_KEY: &str = "openai-compatible-key";
pub const LMSTUDIO_KEY:&str="lmstudio-local-api-key";

const MAX_TOKENS: u32 = 4096;
/// A text file is sent inline up to this many characters; the rest is cut.
pub(crate) const MAX_INLINE_CHARS: usize = 24_000;
/// The island is told about new text at most this often.
const DELTA_INTERVAL: Duration = Duration::from_millis(1000 / 15);
/// Ceilings for a streamed answer: one event line, and the whole answer.
const MAX_LINE: usize = 1024 * 1024;
const MAX_ANSWER: usize = 4 * 1024 * 1024;
/// Ollama and LM Studio ignore the key, but some clients insist on sending one.
const NO_KEY: &str = "ollama";
/// Models that embed or rank rather than chat are left out of the list.
const NOT_CHAT: &[&str] = &["embed", "bge-", "all-minilm", "clip", "rerank"];

// ── The custom server's key ───────────────────────────────────────────────────
//
// The key is stored together with the one address it was entered for, and is
// only ever sent there. The address in the settings, or the one given to
// "Connect", can be changed from the page; the key cannot follow it. And a key
// never goes over plain http to another machine.

#[derive(Serialize, serde::Deserialize)]
struct BoundKey {
    url: String,
    key: String,
}

fn may_carry_key(url: &Url) -> bool {
    url.scheme() == "https" || net::is_loopback_url(url)
}

/// Settings → Local models: stores the key for the address typed next to it.
pub fn set_custom_key(typed_url: &str, key: &str) -> Result<(), String> {set_server_key("custom",typed_url,key)}
pub fn set_server_key(provider:&str,typed_url: &str,key: &str)->Result<(),String>{
    let slot=match provider{"custom"=>CUSTOM_KEY,"lmstudio"=>LMSTUDIO_KEY,_=>return Err("Servidor de credenciais desconhecido.".into())};
    if key.len()>8192||key.chars().any(char::is_control){return Err("Credencial inválida.".into());}
    let url = net::normalise_server_url(typed_url)?;
    if !may_carry_key(&url) {
        return Err(t("A key is only sent over https, or to a server on this computer."));
    }
    let bound = BoundKey { url: url.as_str().trim_end_matches('/').to_string(), key: key.trim().to_string() };
    let value = serde_json::to_string(&bound).map_err(|e| e.to_string())?;
    if provider=="lmstudio"{
        edit_lmstudio_key(||secrets::set(slot,&value))
    }else{secrets::set(slot,&value)}
}

/// The key, if it was entered for exactly this address.
fn custom_key_for(url: &Url) -> Option<String> {
    let stored = secrets::get(CUSTOM_KEY)?;
    key_for(&stored, url)
}

fn key_for(stored: &str, url: &Url) -> Option<String> {
    let bound: BoundKey = serde_json::from_str(stored).ok()?;
    (may_carry_key(url) && bound.url == url.as_str().trim_end_matches('/') && !bound.key.is_empty())
        .then_some(bound.key)
}

/// A model server as the settings describe it.
pub struct Server {
    pub id: &'static str,
    pub name: &'static str,
    /// As stored: empty until the user connects it.
    pub url: String,
    pub key: Option<String>,
}

/// Ollama, LM Studio or "custom", from the settings; None for any other id.
pub fn server(settings: &Settings, id: &str) -> Option<Server> {
    let (id, name, url, key) = match id {
        "ollama" => ("ollama", "Ollama", &settings.ollama_url, None),
        "lmstudio" => ("lmstudio", "LM Studio", &settings.lmstudio_url, net::normalise_server_url(&settings.lmstudio_url).ok().and_then(|url|secrets::get(LMSTUDIO_KEY).and_then(|stored|key_for(&stored,&url)))),
        "custom" => {
            let key = net::normalise_server_url(&settings.custom_url).ok().and_then(|u| custom_key_for(&u));
            ("custom", crate::i18n::n_("OpenAI-compatible server"), &settings.custom_url, key)
        }
        _ => return None,
    };
    Some(Server { id, name, url: url.clone(), key })
}

/// The usual address of a server on this machine; none for "custom". Ollama's
/// own OLLAMA_HOST wins when it is set, as the Ollama CLI does.
fn usual_address(id: &str) -> Option<String> {
    match id {
        "ollama" => Some(
            std::env::var("OLLAMA_HOST")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "http://127.0.0.1:11434".into()),
        ),
        "lmstudio" => Some("http://127.0.0.1:1234".into()),
        _ => None,
    }
}

fn bearer(key: Option<&str>) -> String {
    format!("Bearer {}", key.filter(|k| !k.is_empty()).unwrap_or(NO_KEY))
}

fn unreachable(url: &Url) -> String {
    tf("Cannot reach {url}. Is the server running?", &[("url", url.as_str().trim_end_matches('/'))])
}

pub(crate) fn base_url(server: &Server) -> Result<Url, String> {
    if server.url.trim().is_empty() {
        return Err(tf("Connect {name} in Settings → Local models first.", &[("name", &t(server.name))]));
    }
    net::normalise_server_url(&server.url)
}

/// What the "Connect" button reports: the address as it will be stored, the
/// chat models the server has, and whether the address is this machine.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connected {
    pub url: String,
    pub models: Vec<String>,
    pub loopback: bool,
}

/// Settings → Local models → Connect: does the server answer, and which models
/// does it have? An empty address means the usual one on this machine.
pub async fn connect(id: &str, typed: &str) -> Result<Connected, String> {
    let raw = if typed.trim().is_empty() { usual_address(id).unwrap_or_default() } else { typed.to_string() };
    let url = net::normalise_server_url(&raw)?;
    let key = match id{"custom"=>custom_key_for(&url),"lmstudio"=>secrets::get(LMSTUDIO_KEY).and_then(|stored|key_for(&stored,&url)),_=>None};
    let models = list(&url, key.as_deref()).await?;
    Ok(Connected {
        url: url.as_str().trim_end_matches('/').to_string(),
        models,
        loopback: net::is_loopback_url(&url),
    })
}

/// The models of a connected server, for the picker in the chat view.
pub async fn models(server: &Server) -> Result<Vec<ModelInfo>, String> {
    if server.id=="lmstudio"&&crate::pet_config::read_existing().is_ok_and(|c|c.pets.get("ai_lmstudio").is_some_and(|p|p.provider=="lmstudio")){return Ok(lmstudio::catalog(server,||true).await?.models());}
    let url = base_url(server)?;
    let models = list(&url, server.key.as_deref()).await?;
    if models.is_empty() {
        return Err(tf("No models yet. Download one in {name} first.", &[("name", &t(server.name))]));
    }
    Ok(models.into_iter().map(|id| ModelInfo { label: id.clone(), id }).collect())
}

/// `GET /v1/models`, chat models only, or why the server can't be reached.
async fn list(base: &Url, key: Option<&str>) -> Result<Vec<String>, String> {
    let response = net::exact_destination_client(base, Duration::from_secs(5))?
        .get(net::join(base, "v1/models"))
        .header("Authorization", bearer(key))
        .send()
        .await
        .map_err(|_| unreachable(base))?;
    if matches!(response.status().as_u16(), 401 | 403) {
        return Err(t("The server refused the key. Check it, then connect again."));
    }
    if !response.status().is_success() {
        return Err(unreachable(base));
    }
    let bytes = net::read_capped(response, net::MAX_BODY).await?;
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| unreachable(base))?;
    parse_models(&body).ok_or_else(|| unreachable(base))
}

fn parse_models(body: &Value) -> Option<Vec<String>> {
    let items = body.get("data")?.as_array()?;
    Some(
        items
            .iter()
            .filter_map(|m| m.get("id").and_then(Value::as_str))
            .filter(|id| !NOT_CHAT.iter().any(|n| id.to_lowercase().contains(n)))
            .map(str::to_string)
            .collect(),
    )
}

// ── Messages ──────────────────────────────────────────────────────────────────

/// What a dropped file adds to the first message: text inline (cut), anything
/// else by name only.
pub(crate) fn file_note(name: &str, path: &str) -> String {
    if std::path::Path::new(path).is_dir(){return crate::pet_documents::folder_text::note(name,std::path::Path::new(path));}
    regular_file_note(name,path)
}
pub(crate) fn regular_file_note(name:&str,path:&str)->String{
    if matches!(crate::pet_documents::policy::format(path,false),crate::pet_documents::policy::Format::Binary|crate::pet_documents::policy::Format::Image){return format!("File: {name}\nContent unavailable or unsupported for text extraction; do not infer its contents.");}
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let binary = matches!(ext.as_str(), "pdf" | "jpg" | "jpeg" | "png" | "gif" | "webp");
    // Read no more than the inline limit can use (4 bytes per character at most).
    let text = if ext=="pdf" {match crate::document_text::pdf(path){Ok(text)=>Some(text),Err(error)=>return format!("File: {name}\nDocument unavailable: {error}. Tell the user honestly; do not infer its contents.")}} else if binary { None } else { read_prefix(path, MAX_INLINE_CHARS * 4) };
    match text {
        Some(text) if !text.is_empty() => {
            let mut body: String = text.chars().take(MAX_INLINE_CHARS).collect();
            if body.len() < text.len() {
                body.push_str("\n[truncated]");
            }
            format!("File: {name}\nFile contents:\n{body}")
        }
        _ => format!("File: {name}\nContent unavailable or unsupported; do not infer its contents."),
    }
}

/// The first `limit` bytes of a file as UTF-8 text (a character cut in half at
/// the end is dropped), or None for a file that is not text.
fn read_prefix(path: &str, limit: usize) -> Option<String> {
    crate::pet_documents::content::text_prefix(path,limit)
}

pub(crate) fn user_text(first: bool, context: Option<&ChatContext>, query: &str) -> String {
    match context.filter(|c| first || c.has_files()) {
        Some(ChatContext::File { name, path }) => format!("{}\n\n{query}", file_note(name, path)),
        Some(ChatContext::Window { app_name, title, url }) => {
            format!("{}\n\n{query}", chat::window_line(app_name, title, url.as_deref()))
        }
        Some(ChatContext::Files{files})=>{
            let mut budget=crate::pet_documents::content::Budget::default();let mut notes=vec![];
            for (index,file) in files.iter().enumerate(){let (name,path)=(file.name.as_str(),file.path.as_str());notes.push(file.label(index));let note=if budget.available(){file_note(name,path)}else{crate::pet_documents::content::unavailable(name,"combined extraction deadline")["text"].as_str().unwrap().into()};notes.push(budget.text(note));}
            notes.push(query.into());notes.join("\n\n")
        }
        None => query.to_string(),
    }
}

pub(crate) struct PreparedUser { pub text: String, calendar: Option<crate::pet_calendar::Note>, documents:crate::pet_documents::Request }
impl PreparedUser {
    pub(crate) fn validate(&self) -> Result<(), String> { self.documents.validate()?;crate::pet_calendar::validate(self.calendar.as_ref()) }
    pub(crate) async fn validate_contents(&self)->Result<(),String>{self.documents.validate_contents().await?;crate::pet_calendar::validate(self.calendar.as_ref())}
    pub(crate) fn calendar(&self) -> Option<&crate::pet_calendar::Note> {self.calendar.as_ref()}
}
pub(crate) async fn user_text_for_turn(chat: &Chat, turn: &crate::chat::Turn, context: Option<&ChatContext>, query: &str) -> Result<PreparedUser, String> {
    let documents=crate::pet_documents::Request::capture_async(chat,turn,context).await?;
    let calendar = crate::pet_calendar::note_async(chat, turn, context).await?;
    let ordinary=crate::pet_calendar::ordinary_context(context);let (first,question)=(turn.first,query.to_string());
    let base=tokio::task::spawn_blocking(move||user_text(first,ordinary.as_ref(),&question)).await.map_err(|_|"A preparação dos anexos foi interrompida.".to_string())?;
    let text=crate::pet_calendar::prepend_text(calendar.as_ref(),base);
    documents.validate_contents().await?;Ok(PreparedUser { text, calendar,documents })
}

fn request_body(model: &str, system: &str, history: &[Value], user: &Value) -> Value {
    let mut messages = vec![json!({ "role": "system", "content": system })];
    messages.extend(history.iter().cloned());
    messages.push(user.clone());
    json!({ "model": model, "messages": messages, "stream": true, "max_tokens": MAX_TOKENS })
}

/// One chat turn with a model server; the island gets the streamed text as `chat-delta` events.
pub async fn send(
    app: &AppHandle,
    chat: &Chat,
    settings:&Settings,
    server: &Server,
    model: &str,
    selection:Option<&crate::pet_config::Selection>,
    query: String,
    context: Option<ChatContext>,
) -> Result<ChatReply, String> {
    if server.id=="ollama"&&cfg!(target_os="linux"){
        let policy=crate::pet_config::read()?.local_execution();
        if policy.mode==crate::pet_config::LocalExecutionMode::Ask{return crate::local_tasks::respond(app,chat,settings,server,model,selection,query,context,policy,None).await;}
    }
    send_inner(chat,server,model,query,context,Some(app),selection,|_|{}).await
}

/// The same turn, with the streamed text handed to `on_delta` instead (the phone link uses this).
pub async fn send_with(
    chat: &Chat,
    server: &Server,
    model: &str,
    query: String,
    context: Option<ChatContext>,
    on_delta: impl FnMut(String),
) -> Result<ChatReply, String> {
    send_inner(chat,server,model,query,context,None,None,on_delta).await
}

async fn send_inner(
    chat:&Chat,server:&Server,model:&str,query:String,context:Option<ChatContext>,app:Option<&AppHandle>,selection:Option<&crate::pet_config::Selection>,mut on_delta:impl FnMut(String),
)->Result<ChatReply,String> {
    chat.check_new_effect()?;
    let base = base_url(server)?;
    if model.is_empty() {
        return Err(t("Pick a model above the chat box first."));
    }
    let turn = chat.begin_cli(server.id,std::sync::Arc::default())?;
    let capture_origin = chat.usage_capture_origin(&turn);
    let lease=chat.lease(&turn);
    let _lease=if let Some(app)=app{lease.broadcast(app,&query)}else{lease};
    let identity = if server.id == "ollama" {
        let config = crate::pet_config::read_existing()?;
        Some(chat.capture_local_identity(&turn, &config, "ai_ollama")?)
    } else if server.id=="lmstudio" {
        let config=crate::pet_config::read_existing()?;
        if config.pets.get("ai_lmstudio").is_some_and(|p|p.provider=="lmstudio"){
            lmstudio::inspect(server,model,||chat.is_current(&turn)).await?;
            Some(chat.capture_local_identity(&turn,&config,"ai_lmstudio")?)
        }else{if selection.is_some(){return Err("A associação do pet LM Studio mudou.".into());}None}
    }else{None};
    let activity=app.map(|app|crate::pet_activity::Reporter::new(app,server.id,Some(turn.cancel.clone())).observe(chat.clone(),turn.clone()));
    let prepared = user_text_for_turn(chat, &turn, context.as_ref(), &query).await?;
    let user = json!({ "role": "user", "content": prepared.text });
    let system = identity.as_ref().map(|identity| chat::system_prompt_for_pet(identity, false))
        .unwrap_or_else(|| chat::system_prompt(false));
    let mut body = request_body(model, &system, &turn.history, &user);
    chat.append_selected_memory(server.id,&query,&crate::pet_memory::context_model::Destination::Server{provider:server.id.into(),endpoint:net::join(&base,"v1/chat/completions"),model:model.into()},server.key.as_deref(),body["messages"].as_array_mut().unwrap())?;
    if server.id=="ollama"{body["stream_options"]=json!({"include_usage":true});}
    let mut calendar_tools = prepared.calendar().and_then(|n|n.query()).is_some();
    if server.id=="ollama"&&(selection.is_some()||calendar_tools){
        let caps=tokio::select!{caps=crate::local_tasks::protocol::capabilities(&base,model)=>caps,_=async{while !turn.cancel.load(std::sync::atomic::Ordering::Acquire){tokio::time::sleep(Duration::from_millis(25)).await;}}=>return Err("A tarefa foi interrompida.".into())};
        if caps.tools==crate::local_tasks::protocol::ToolsSupport::Remote{return Err("Escolhe um modelo local Ollama instalado.".into());}
        calendar_tools &= caps.tools==crate::local_tasks::protocol::ToolsSupport::Confirmed;
        crate::local_tasks::protocol::apply_effort(&mut body,caps.effort(selection).as_deref());
    }

    if calendar_tools {
        let (answer,usage) = calendar_answer(chat,&turn,&prepared,&base,server.key.as_deref(),model,&body,capture_origin.as_ref()).await?;
        prepared.validate_contents().await?;
        let plain=chat::plain_question(turn.first,context.as_ref(),&query);
        let notice=chat.commit_observed_reply(&turn,user,json!({"role":"assistant","content":answer}),&plain,&answer)?;
        if let Some(app)=app{chat.publish_delta(app,&turn,&answer);}
        on_delta(answer.clone());
        if let Some(activity)=&activity{activity.finish_observed(&answer,notice.as_deref());}
        crate::pet_usage::record(app,chat,&turn,usage);
        return Ok(ChatReply{text:answer});
    }

    // Retain the accounting guard outside the cancellable stream future. Its
    // captured owner/CID never changes when Stop or another window intervenes.
    // Metadata discovery above is not an inference attempt.
    prepared.validate_contents().await?;
    let mut capture = None;
    let streamed = tokio::select! {
        result=stream_bounded_captured(&base, server.key.as_deref(), model, &body, capture_origin.as_ref(), &mut capture, |text|{if prepared.validate().is_ok()&&chat.is_current(&turn){if let Some(app)=app {chat.publish_delta(app,&turn,&text);}if let Some(activity)=&activity{activity.update("message",text.rsplit("\n\n").next().unwrap_or(&text));}on_delta(text);}},MAX_ANSWER,false,||chat.check_new_effect())=>Some(result),
        _=async {while chat.is_current(&turn) && prepared.validate().is_ok() {tokio::time::sleep(Duration::from_millis(50)).await;}}=>None,
    };
    // The select futures have been dropped before accessing capture again.
    let answer = match streamed {
        Some(Ok(answer)) => answer,
        Some(Err(error)) => {
            finish_text_capture(&mut capture, AttemptOutcome::Failed);
            return Err(error);
        }
        None => {
            finish_text_capture(&mut capture, AttemptOutcome::Cancelled);
            return Err("The CLI request was cancelled.".into());
        }
    };
    if !chat.is_current(&turn) {
        finish_text_capture(&mut capture, AttemptOutcome::Cancelled);
        return Err("The CLI request was cancelled.".into());
    }
    let usage=answer.usage;let answer=answer.text;
    if answer.is_empty() {
        finish_text_capture(&mut capture, AttemptOutcome::Failed);
        return Err(t("No response text."));
    }
    prepared.validate_contents().await?;
    let plain = chat::plain_question(turn.first, context.as_ref(), &query);
    let notice = match chat.commit_observed_reply(&turn, user, json!({ "role": "assistant", "content": answer }), &plain, &answer) {
        Ok(notice) => notice,
        Err(error) => {
            let outcome = if chat.is_current(&turn) { AttemptOutcome::Failed } else { AttemptOutcome::Cancelled };
            finish_text_capture(&mut capture, outcome);
            return Err(error);
        }
    };
    // Saving public chat text and showing it do not own the usage record.
    finish_text_capture(&mut capture, AttemptOutcome::Completed);
    if let Some(activity)=&activity{activity.finish_observed(&answer,notice.as_deref());}
    crate::pet_usage::record(app,chat,&turn,usage);
    Ok(ChatReply { text: answer })
}

/// The native goal writer supplies this non-serializable, single-use boundary.
pub(crate)trait RuntimeFence:Send+Sync{
 fn current(&self)->bool;
 fn output_limit(&self)->usize;
 fn before_send(&self,model:&str,body:&Value)->Result<(),String>;
 fn memory_message(&self)->Result<Option<Value>,String>{Ok(None)}
 fn before_send_to(&self,_server:&Server,model:&str,body:&Value)->Result<(),String>{self.before_send(model,body)}
}
pub(crate)struct RuntimeReply{pub reply:ChatReply,pub accepted_output_bytes:u64}
pub(crate)async fn send_runtime_once(app:Option<&AppHandle>,chat:&Chat,server:&Server,model:&str,query:String,model_input:String,fence:&impl RuntimeFence,config:&crate::pet_config::Configuration)->Result<RuntimeReply,String>{
 let base=base_url(server)?;if !fence.current(){return Err("O objetivo deixou de estar pronto.".into());}
 let turn=chat.begin_cli(server.id,std::sync::Arc::default())?;let lease=chat.lease(&turn);let _lease=if let Some(app)=app{lease.broadcast(app,&query)}else{lease};
 let identity=chat.capture_local_identity(&turn,config,crate::pet_config::pet_id(server.id).ok_or("O pet do modelo mudou.")?)?;
 let user=json!({"role":"user","content":model_input});let mut body=request_body(model,&chat::system_prompt_for_pet(&identity,false),&turn.history,&user);
 // This distinct data message is never committed as the public user question
 // or copied into turn history. The fence hashes the actual complete body.
 if let Some(memory)=fence.memory_message()?{
  let messages=body.get_mut("messages").and_then(Value::as_array_mut).filter(|m|!m.is_empty()).ok_or("O pedido do objetivo é inválido.")?;
  messages.insert(messages.len()-1,memory);
 }
 let origin=chat.usage_capture_origin(&turn);let mut capture=None;
 let result=tokio::select!{
  result=stream_bounded_captured(&base,server.key.as_deref(),model,&body,origin.as_ref(),&mut capture,|text|{if fence.current()&&chat.is_current(&turn){if let Some(app)=app{chat.publish_delta(app,&turn,&text);}}},fence.output_limit(),true,||fence.before_send_to(server,model,&body))=>result,
  _=async{loop{if !fence.current()||!chat.is_current(&turn){break;}tokio::time::sleep(Duration::from_millis(50)).await;}}=>Err("A execução foi interrompida.".into()),
 };
 let answer=match result{Ok(answer)=>answer,Err(error)=>{finish_text_capture(&mut capture,AttemptOutcome::Failed);return Err(error);}};
 if !fence.current()||!chat.is_current(&turn){finish_text_capture(&mut capture,AttemptOutcome::Cancelled);return Err("A execução foi interrompida.".into());}
 if !answer.protocol_complete{finish_text_capture(&mut capture,AttemptOutcome::Failed);return Err("O stream terminou sem confirmação do protocolo; não repetir automaticamente.".into());}
 if answer.text.is_empty(){finish_text_capture(&mut capture,AttemptOutcome::Failed);return Err(t("No response text."));}
 chat.commit_observed_reply(&turn,user,json!({"role":"assistant","content":answer.text}),&query,&answer.text)?;
 finish_text_capture(&mut capture,AttemptOutcome::Completed);crate::pet_usage::record(app,chat,&turn,answer.usage);
 Ok(RuntimeReply{reply:ChatReply{text:answer.text},accepted_output_bytes:answer.accepted_output_bytes})
}

/// Calendar reads use actual non-streamed function-call envelopes. A measured
/// attempt is retained outside each cancellable request, including failed ones.
#[allow(clippy::too_many_arguments)]
async fn calendar_answer(chat:&Chat,turn:&crate::chat::Turn,prepared:&PreparedUser,base:&Url,key:Option<&str>,model:&str,initial:&Value,origin:Option<&CaptureOrigin>)->Result<(String,crate::pet_usage::Usage),String>{
    use crate::pet_calendar::wire::{Session,Format};
    let mut session=Session::new(prepared.calendar(),Format::Chat).ok_or("A consulta de calendário não está disponível.")?;
    let mut body=initial.clone();body["stream"]=json!(false);body.as_object_mut().unwrap().remove("stream_options");session.prepare(&mut body)?;
    let mut total=None::<crate::pet_usage::Usage>;
    loop{
        prepared.validate_contents().await?;session.validate()?;
        let mut capture=None::<Capture>;let began=Instant::now();
        let response=net::while_current(
            ||chat.is_current(turn)&&prepared.validate().is_ok()&&session.validate().is_ok(),
            ||async{
                chat.check_new_effect()?;
                let client=net::exact_destination_client(base,Duration::from_secs(90))?;
                let request=client.post(net::join(base,"v1/chat/completions")).header("Authorization",bearer(key)).json(&body).build().map_err(|_|"Não foi possível preparar a consulta.")?;
                capture=Some(Capture::begin(origin,model));
                let response=client.execute(request).await.map_err(|_|"A ligação ao modelo foi interrompida.")?;
                if !response.status().is_success(){return Err(format!("O servidor recusou as ferramentas para o modelo selecionado (HTTP {}). O modelo foi mantido.",response.status().as_u16()));}
                let bytes=net::read_capped(response,MAX_ANSWER).await?;
                let value:Value=serde_json::from_slice(&bytes).map_err(|_|"Resposta estruturada inválida do modelo.")?;
                if let Some(capture)=&mut capture{capture.observe_oai(&value);capture.transport_complete();}
                session.validate_response(&value)?;
                Ok(value)
            },
        ).await;
        let outcome=if response.is_ok(){AttemptOutcome::Completed}else if !chat.is_current(turn){AttemptOutcome::Cancelled}else{AttemptOutcome::Failed};
        finish_text_capture(&mut capture,outcome);
        let response=response?;
        let mut usage=crate::pet_usage::Usage::default();usage.observe_oai(&response);let usage=usage.finish(model,began.elapsed());
        if let Some(total)=&mut total{total.add(&usage);}else{total=Some(usage);}
        if session.follow_up(&response,&mut body).await?{continue;}
        let text=response.pointer("/choices/0/message/content").and_then(Value::as_str).map(progressive_filter).filter(|s|!s.is_empty()).ok_or("O modelo não devolveu uma resposta de texto.")?;
        prepared.validate_contents().await?;
        return Ok((text,total.unwrap_or_default()));
    }
}

#[cfg(test)]
mod calendar_query_tests {
    use super::*;
    use std::{io::{Read,Write},sync::{Arc,atomic::{AtomicUsize,Ordering}}};
    #[tokio::test]
    async fn two_http_rounds_keep_model_history_key_and_exact_tool_results_without_inference(){
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();let base=Url::parse(&format!("http://{}",listener.local_addr().unwrap())).unwrap();
        let args=json!({"start":"2026-11-21T00:00:00Z","end":"2026-11-22T00:00:00Z","timeZone":"Europe/Lisbon","limit":128});
        let replies=[json!({"choices":[{"message":{"role":"assistant","content":null,"tool_calls":[{"id":"exact-id","type":"function","function":{"name":"calendar_query","arguments":args.to_string()}}]},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}}),json!({"choices":[{"message":{"role":"assistant","content":"Resposta com consulta."},"finish_reason":"stop"}],"usage":{"prompt_tokens":7,"completion_tokens":11,"total_tokens":18}})];
        let (tx,rx)=std::sync::mpsc::channel();
        let server=std::thread::spawn(move||{
            for reply in replies{
                let (mut conn,_)=listener.accept().unwrap();conn.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut request=Vec::new();let mut buffer=[0;4096];
                let split=loop{let n=conn.read(&mut buffer).unwrap();assert!(n>0);request.extend_from_slice(&buffer[..n]);if let Some(p)=request.windows(4).position(|s|s==b"\r\n\r\n"){break p+4;}};
                let header=String::from_utf8(request[..split].to_vec()).unwrap();assert!(header.to_lowercase().contains("authorization: bearer same-bound-key"));
                let size:usize=header.lines().find_map(|line|line.to_lowercase().strip_prefix("content-length:").and_then(|n|n.trim().parse().ok())).unwrap();
                while request.len()<split+size{let n=conn.read(&mut buffer).unwrap();assert!(n>0);request.extend_from_slice(&buffer[..n]);}
                tx.send(serde_json::from_slice::<Value>(&request[split..split+size]).unwrap()).unwrap();
                let bytes=reply.to_string();write!(conn,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",bytes.len(),bytes).unwrap();
            }
        });
        let count=Arc::new(AtomicUsize::new(0));let observed=count.clone();
        let note=crate::pet_calendar::synthetic_query_note(move|q,_|{assert_eq!(q.limit,128);observed.fetch_add(1,Ordering::AcqRel);Ok(json!({"status":"unavailable","errorCode":"synthetic","events":[]}))});
        let prepared=PreparedUser{text:"query".into(),calendar:Some(note),documents:Default::default()};
        let chat=Chat::default();let turn=chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=chat.lease(&turn);
        let mut initial=request_body("same-model:tag","same identity",&[json!({"role":"assistant","content":"earlier"})],&json!({"role":"user","content":"query"}));initial["reasoning_effort"]=json!("none");
        let (text,usage)=calendar_answer(&chat,&turn,&prepared,&base,Some("same-bound-key"),"same-model:tag",&initial,None).await.unwrap();
        assert_eq!(text,"Resposta com consulta.");assert_eq!(count.load(Ordering::Acquire),1);assert_eq!(usage.total_tokens,Some(23));
        let first=rx.recv_timeout(Duration::from_secs(1)).unwrap();let second=rx.recv_timeout(Duration::from_secs(1)).unwrap();
        for body in [&first,&second]{assert_eq!(body["model"],"same-model:tag");assert_eq!(body["reasoning_effort"],"none");assert_eq!(body["stream"],false);assert_eq!(body["messages"][0]["content"],"same identity");assert_eq!(body["messages"][1]["content"],"earlier");}
        assert_eq!(second["messages"][4]["tool_call_id"],"exact-id");assert!(second["messages"][4]["content"].as_str().unwrap().contains("synthetic"));server.join().unwrap();assert!(chat.history("ollama").is_empty());
    }
}

// ── Streaming ─────────────────────────────────────────────────────────────────

/// One server-sent event line: `Some(Ok(text))` for a content delta,
/// `Some(Err(message))` for an error event, None for anything else (other
/// lines, `[DONE]`, a delta without content, such as reasoning-only ones).
fn parse_sse_line(line: &str) -> Option<Result<String, String>> {
    let payload = line.strip_prefix("data:")?.trim_start();
    if payload == "[DONE]" {
        return None;
    }
    let json: Value = serde_json::from_str(payload).ok()?;
    if let Some(msg) = json.pointer("/error/message").and_then(Value::as_str) {
        return Some(Err(msg.to_string()));
    }
    json.pointer("/choices/0/delta/content")?.as_str().map(|s| Ok(s.to_string()))
}

/// Removes finished `<think>…</think>` blocks (reasoning models such as DeepSeek-R1).
fn filter_thinking_blocks(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<think>") {
        let Some(len) = rest[start..].find("</think>") else { break };
        out.push_str(&rest[..start]);
        rest = &rest[start + len + "</think>".len()..];
    }
    out.push_str(rest);
    out.trim().to_string()
}

/// For the text shown while streaming: finished blocks go, and so does
/// everything after a block that has not been closed yet.
pub(crate) fn progressive_filter(text: &str) -> String {
    let cleaned = filter_thinking_blocks(text);
    match cleaned.find("<think>") {
        Some(at) => cleaned[..at].trim().to_string(),
        None => cleaned,
    }
}

/// Posts the request and reads the event stream. `on_delta` gets the visible
/// text at most 15 times a second, and once more at the end; the result is the
/// whole answer without its thinking.
struct MeasuredReply{text:String,usage:crate::pet_usage::Usage,accepted_output_bytes:u64,protocol_complete:bool}

#[cfg(test)]
async fn stream(base:&Url,key:Option<&str>,model:&str,body:&Value,on_delta:impl FnMut(String))->Result<String,String>{
    stream_measured(base,key,model,body,on_delta).await.map(|reply|reply.text)
}

// Compatibility wrapper for existing internal callers and offline fixtures.
// No captured origin means no registered ledger attempt, even if a service runs.
#[allow(dead_code)]
async fn stream_measured(
    base: &Url,
    key: Option<&str>,
    model: &str,
    body: &Value,
    on_delta: impl FnMut(String),
) -> Result<MeasuredReply, String> {
    let mut capture = None;
    let result = stream_measured_captured(base, key, model, body, None, &mut capture, on_delta).await;
    let outcome = if result.is_ok() { AttemptOutcome::Completed } else { AttemptOutcome::Failed };
    finish_text_capture(&mut capture, outcome);
    result
}

fn finish_text_capture(capture: &mut Option<Capture>, outcome: AttemptOutcome) {
    if let Some(capture) = capture { capture.finish(outcome); }
}

/// Observe server counters before text/error validation. Accounting is stricter
/// about protocol-end evidence than the legacy text renderer, whose behavior is
/// unchanged. A lossy UTF-8 frame can still render as before but is not proof.
/// Valid DONE seals capture only; legacy usage/rendering continue unchanged.
fn observe_stream_usage(line: &[u8], usage: &mut crate::pet_usage::Usage, capture: &mut Option<Capture>, protocol_ended: &mut bool) {
    let text = String::from_utf8_lossy(line);
    let Some(payload) = text.strip_prefix("data:") else { return; };
    let framed = payload.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\r' | '\n'));
    let payload = payload.trim();
    let valid_frame = std::str::from_utf8(line).is_ok() && framed == payload;
    if payload == "[DONE]" {
        if valid_frame && !*protocol_ended {
            *protocol_ended = true;
            if let Some(capture) = capture { capture.transport_complete(); }
        }
        return;
    }
    let Ok(value) = serde_json::from_str::<Value>(payload) else { return; };
    usage.observe_oai(&value);
    if !valid_frame || *protocol_ended { return; }
    let Some(capture) = capture else { return; };
    capture.observe_oai(&value);
    // A well-shaped single-choice completion marker proves this request ended.
    // Known terminal reasons include truncated/filtered/tool responses: token
    // completion is independent of whether the public answer is acceptable.
    let finished = value.get("error").is_none() && value["choices"].as_array().is_some_and(|choices| {
        choices.len() == 1 && choices[0]["index"].as_u64() == Some(0)
            && choices[0]["delta"].is_object()
            && matches!(choices[0]["finish_reason"].as_str(),
                Some("stop" | "length" | "content_filter" | "tool_calls" | "function_call"))
    });
    if finished { capture.transport_complete(); }
}

#[allow(clippy::too_many_arguments)]
async fn stream_measured_captured(
    base: &Url,
    key: Option<&str>,
    model: &str,
    body: &Value,
    origin: Option<&CaptureOrigin>,
    capture: &mut Option<Capture>,
    mut on_delta: impl FnMut(String),
) -> Result<MeasuredReply, String> {
    stream_bounded_captured(base,key,model,body,origin,capture,&mut on_delta,MAX_ANSWER,false,||Ok(())).await
}
async fn stream_bounded_captured(
    base:&Url,key:Option<&str>,model:&str,body:&Value,origin:Option<&CaptureOrigin>,capture:&mut Option<Capture>,
    mut on_delta:impl FnMut(String),max_output:usize,require_local:bool,before_send:impl FnOnce()->Result<(),String>,
)->Result<MeasuredReply,String>{
    if max_output==0||max_output>MAX_ANSWER{return Err("Limite de resposta inválido.".into());}
    let began=Instant::now();let mut usage=crate::pet_usage::Usage::default();
    let client = if require_local{net::local_client(base,Duration::from_secs(300))?}else{net::exact_destination_client(base, Duration::from_secs(300))?};
    let request = client.post(net::join(base, "v1/chat/completions"))
        .header("Authorization", bearer(key))
        .json(body)
        .build()
        .map_err(|_| unreachable(base))?;
    // One guard per actual dispatch attempt, after request construction and
    // metadata discovery. No credentials, prompts or response bytes are kept.
    crate::settings::effect_allowed()?;
    before_send()?;
    *capture = Some(Capture::begin(origin, model));
    let mut reply = client.execute(request).await.map_err(|_| unreachable(base))?;

    let status = reply.status();
    if status.as_u16() == 404 {
        return Err(tf("{model} isn't installed. Pick another model above the chat box.", &[("model", model)]));
    }
    if !status.is_success() {
        let body = net::read_capped(reply, net::MAX_ERROR_BODY).await.unwrap_or_default();
        // Preserve any counters in the already-read error body, without treating
        // an HTTP error or its EOF as proof of a completed model response.
        if let Ok(value) = serde_json::from_slice::<Value>(&body) {
            if let Some(capture) = capture { capture.observe_oai(&value); }
        }
        let detail = net::error_detail(&body);
        return Err(if detail.is_empty() { format!("HTTP {}", status.as_u16()) } else { detail });
    }

    let mut accumulated = String::new();
    let mut pending = Vec::<u8>::new();
    let mut protocol_ended = false;
    let mut last = Instant::now().checked_sub(DELTA_INTERVAL).unwrap_or_else(Instant::now);
    while let Some(chunk) = reply.chunk().await.map_err(|_| unreachable(base))? {
        pending.extend_from_slice(&chunk);
        // Whole lines only: a chunk may end in the middle of one, or of a character.
        while let Some(end) = pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = pending.drain(..=end).collect();
            observe_stream_usage(&line, &mut usage, capture, &mut protocol_ended);
            let line = String::from_utf8_lossy(&line);
            match parse_sse_line(line.trim_end()) {
                Some(Ok(delta)) => {if protocol_ended{return Err("Conteúdo recebido após o fim do stream.".into());}if delta.len()>max_output.saturating_sub(accumulated.len()){return Err(t("The server's answer is too large."));}accumulated.push_str(&delta);},
                Some(Err(message)) => return Err(message),
                None => {}
            }
        }
        if pending.len() > MAX_LINE || accumulated.len() > max_output {
            return Err(t("The server's answer is too large."));
        }
        if last.elapsed() >= DELTA_INTERVAL {
            last = Instant::now();
            on_delta(progressive_filter(&accumulated));
        }
    }
    // A last line without its newline.
    observe_stream_usage(&pending, &mut usage, capture, &mut protocol_ended);
    let tail=String::from_utf8_lossy(&pending);
    match parse_sse_line(tail.trim_end()){Some(Ok(delta))=>{if protocol_ended{return Err("Conteúdo recebido após o fim do stream.".into());}if delta.len()>max_output.saturating_sub(accumulated.len()){return Err(t("The server's answer is too large."));}accumulated.push_str(&delta);},Some(Err(error))=>return Err(error),None=>{}}
    // The last state always goes out, whatever the throttle skipped.
    on_delta(progressive_filter(&accumulated));
    Ok(MeasuredReply{protocol_complete:protocol_ended,accepted_output_bytes:accumulated.len() as u64,text:filter_thinking_blocks(&accumulated),usage:usage.finish(model,began.elapsed())})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::tests::serve_once;

    #[test]fn selected_fast_mode_reaches_plain_body_while_unselected_phone_body_is_unchanged(){
        let user=json!({"role":"user","content":"same"});let mut body=request_body("same-model:tag","system",&[],&user);
        let phone_body=body.clone();let selection=crate::pet_config::Selection{model:"same-model:tag".into(),mode:"fast".into(),effort:String::new()};let caps=crate::local_tasks::protocol::ModelCapabilities::from_show(&json!({"capabilities":["completion","thinking"],"thinking":null}));crate::local_tasks::protocol::apply_effort(&mut body,caps.effort(Some(&selection)).as_deref());
        assert_eq!(body["reasoning_effort"],"none");assert_eq!(body["model"],phone_body["model"]);assert_eq!(body["messages"],phone_body["messages"]);assert!(body.get("think").is_none());assert!(phone_body.get("reasoning_effort").is_none());assert!(body.get("tools").is_none());
    }

    fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(f)
    }

    fn url(s: &str) -> Url {
        net::normalise_server_url(s).unwrap()
    }

    #[test]
    fn only_content_deltas_are_taken_from_the_event_stream() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hel"}}]}"#;
        assert_eq!(parse_sse_line(line), Some(Ok("Hel".into())));
        let line = r#"data:{"choices":[{"delta":{"content":"lo"}}]}"#;
        assert_eq!(parse_sse_line(line), Some(Ok("lo".into())));
        assert_eq!(parse_sse_line("data: [DONE]"), None);
        assert_eq!(parse_sse_line(""), None);
        assert_eq!(parse_sse_line(": keep-alive"), None);
        assert_eq!(parse_sse_line(r#"data: {"choices":[{"delta":{"role":"assistant"}}]}"#), None);
        assert_eq!(parse_sse_line(r#"data: {"choices":[{"delta":{"content":null,"reasoning_content":"x"}}]}"#), None);
        assert_eq!(parse_sse_line("data: not json"), None);
        assert_eq!(parse_sse_line(r#"data: {"error":{"message":"out of memory"}}"#), Some(Err("out of memory".into())));
    }

    #[test]
    fn finished_thinking_blocks_go_and_an_open_one_hides_what_follows() {
        assert_eq!(filter_thinking_blocks("<think>hmm</think>\n\nThe answer."), "The answer.");
        assert_eq!(filter_thinking_blocks("a<think>x</think>b<think>y</think>c"), "abc");
        assert_eq!(filter_thinking_blocks("plain"), "plain");
        // Streaming: nothing of an unfinished block shows.
        assert_eq!(progressive_filter("Sure. <think>let me think"), "Sure.");
        assert_eq!(progressive_filter("<think>still thinking"), "");
        assert_eq!(progressive_filter("<think>done</think>Visible <think>again"), "Visible");
        // A tag that is only half written is not a block yet.
        assert_eq!(progressive_filter("text <thi"), "text <thi");
    }

    #[test]
    fn the_request_streams_with_system_history_and_the_new_turn() {
        let history = vec![json!({"role":"user","content":"a"}), json!({"role":"assistant","content":"b"})];
        let body = request_body("llama3.2", "sys", &history, &json!({"role":"user","content":"c"}));
        assert_eq!(body["stream"], true);
        assert_eq!(body["model"], "llama3.2");
        assert_eq!(body["messages"].as_array().unwrap().len(), 4);
        assert_eq!(body["messages"][0]["role"], "system");
    }

    #[test]
    fn a_key_is_sent_as_the_bearer_and_without_one_the_placeholder_is() {
        assert_eq!(bearer(Some("sk-abc")), "Bearer sk-abc");
        assert_eq!(bearer(Some("")), "Bearer ollama");
        assert_eq!(bearer(None), "Bearer ollama");
    }

    #[test]
    fn a_server_that_is_not_connected_says_so() {
        let s = Settings::default();
        let ollama = server(&s, "ollama").unwrap();
        assert_eq!(base_url(&ollama).unwrap_err(), "Connect Ollama in Settings → Local models first.");
        assert!(server(&s, "openai").is_none());
        assert_eq!(usual_address("lmstudio").as_deref(), Some("http://127.0.0.1:1234"));
        assert_eq!(usual_address("custom"), None);
    }

    #[test]
    fn the_model_list_leaves_out_embedding_models_and_says_when_nothing_answers() {
        let body = br#"{"data":[{"id":"llama3.2"},{"id":"nomic-embed-text"},{"id":"BGE-large"},{"id":"qwen2.5-coder"}]}"#;
        let u = serve_once("200 OK", &format!("Content-Length: {}\r\n", body.len()), body.to_vec());
        assert_eq!(block_on(list(&url(&u), None)).unwrap(), vec!["llama3.2", "qwen2.5-coder"]);

        let u = serve_once("500 Internal Server Error", "Content-Length: 2\r\n", b"{}".to_vec());
        assert!(block_on(list(&url(&u), None)).unwrap_err().starts_with("Cannot reach"));
        let u = serve_once("200 OK", "Content-Length: 8\r\n", b"not json".to_vec());
        assert!(block_on(list(&url(&u), None)).unwrap_err().starts_with("Cannot reach"));
        assert!(block_on(list(&url("http://127.0.0.1:1"), None)).unwrap_err().starts_with("Cannot reach"));
        let u = serve_once("401 Unauthorized", "Content-Length: 2\r\n", b"{}".to_vec());
        assert!(block_on(list(&url(&u), Some("bad"))).unwrap_err().contains("refused the key"));
    }

    #[test]
    fn connect_cleans_a_pasted_address_and_reports_whether_it_is_this_machine() {
        let body = br#"{"data":[{"id":"m"}]}"#;
        let u = serve_once("200 OK", &format!("Content-Length: {}\r\n", body.len()), body.to_vec());
        let c = block_on(connect("lmstudio", &format!("{u}/v1/"))).unwrap();
        assert_eq!(c.url, u);
        assert_eq!(c.models, vec!["m"]);
        assert!(c.loopback);
        assert_eq!(block_on(connect("custom", " ")).unwrap_err(), "Enter the server address first.");
        assert!(block_on(connect("custom", "file:///etc")).is_err());
    }

    #[test]
    fn the_stream_is_read_line_by_line_and_thinking_stays_hidden() {
        let events = [
            r#"{"choices":[{"delta":{"content":"<think>\nstep "}}]}"#,
            r#"{"choices":[{"delta":{"content":"one\n</think>"}}]}"#,
            r#"{"choices":[{"delta":{"content":null,"reasoning_content":"internal"}}]}"#,
            r###"{"choices":[{"delta":{"content":"## Answer\n\n"}}]}"###,
            r#"{"choices":[{"delta":{"content":"- **item 1**\n"}}]}"#,
            r#"{"choices":[{"delta":{"content":"```python\nprint('hello')\n```"}}]}"#,
        ];
        let mut body = String::new();
        for e in events {
            body.push_str(&format!("data: {e}\n\n"));
        }
        body.push_str("data: [DONE]\n\n");
        let u = serve_once("200 OK", "Content-Type: text/event-stream\r\n", body.into_bytes());
        let mut seen = Vec::new();
        let answer = block_on(stream(&url(&u), None, "m", &json!({}), |v| seen.push(v))).unwrap();
        assert_eq!(answer, "## Answer\n\n- **item 1**\n```python\nprint('hello')\n```");
        assert_eq!(seen.last().unwrap(), &answer);
        assert!(seen.iter().all(|v| !v.contains("step") && !v.contains("internal")));
    }

    #[test]
    fn a_final_usage_only_event_is_measured_without_becoming_chat_text(){
        let body=b"data: {\"model\":\"actual:local\",\"choices\":[{\"delta\":{\"content\":\"Ola\"}}]}\n\ndata: {\"choices\":[],\"usage\":{\"prompt_tokens\":9,\"completion_tokens\":2,\"total_tokens\":11}}";
        let endpoint=serve_once("200 OK",&format!("Content-Length: {}\r\nContent-Type: text/event-stream\r\n",body.len()),body.to_vec());
        let reply=block_on(stream_measured(&url(&endpoint),None,"requested:alias",&json!({}),|_|{})).unwrap();
        assert_eq!(reply.text,"Ola");assert_eq!(reply.usage.input_tokens,Some(9));assert_eq!(reply.usage.output_tokens,Some(2));
        assert_eq!(reply.usage.total_tokens,Some(11));assert_eq!(reply.usage.model.as_deref(),Some("actual:local"));assert!(reply.usage.measured);
    }
    #[test]
    fn an_error_at_eof_does_not_commit_a_partial_local_answer(){
        let body=b"data: {\"choices\":[{\"delta\":{\"content\":\"Partial\"}}]}\n\ndata: {\"error\":{\"message\":\"synthetic failure\"}}";
        let endpoint=serve_once("200 OK",&format!("Content-Length: {}\r\n",body.len()),body.to_vec());
        assert!(block_on(stream_measured(&url(&endpoint),None,"local",&json!({}),|_|{})).is_err());
    }

    #[test]
    fn stream_errors_say_what_happened() {
        let u = serve_once("404 Not Found", "Content-Length: 2\r\n", b"{}".to_vec());
        let err = block_on(stream(&url(&u), None, "unknown-model", &json!({}), |_| {})).unwrap_err();
        assert_eq!(err, "unknown-model isn't installed. Pick another model above the chat box.");
        let body = br#"{"error":{"message":"context too long"}}"#;
        let u = serve_once("400 Bad Request", &format!("Content-Length: {}\r\n", body.len()), body.to_vec());
        assert_eq!(block_on(stream(&url(&u), None, "m", &json!({}), |_| {})).unwrap_err(), "context too long");
        let u = serve_once("200 OK", "", b"data: {\"error\":{\"message\":\"crashed\"}}\n".to_vec());
        assert_eq!(block_on(stream(&url(&u), None, "m", &json!({}), |_| {})).unwrap_err(), "crashed");
    }

    #[test]
    fn an_endless_line_is_cut_off() {
        let u = serve_once("200 OK", "", vec![b'x'; MAX_LINE + 10]);
        let err = block_on(stream(&url(&u), None, "m", &json!({}), |_| {})).unwrap_err();
        assert_eq!(err, "The server's answer is too large.");
    }

    #[test]
    fn text_files_go_inline_cut_and_everything_else_by_name() {
        let dir = std::env::temp_dir().join(format!("coucou-local-chat-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let txt = dir.join("notes.txt");
        std::fs::write(&txt, "é".repeat(MAX_INLINE_CHARS + 500)).unwrap();
        let note = file_note("notes.txt", txt.to_str().unwrap());
        let body = note.strip_prefix("File: notes.txt\nFile contents:\n").expect("inline text");
        let body = body.strip_suffix("\n[truncated]").expect("marked as cut");
        assert_eq!(body.chars().count(), MAX_INLINE_CHARS);

        let small = dir.join("small.md");
        std::fs::write(&small, "# hi").unwrap();
        assert_eq!(file_note("small.md", small.to_str().unwrap()), "File: small.md\nFile contents:\n# hi");

        let png = dir.join("pic.png");
        std::fs::write(&png, "not really a picture").unwrap();
        assert!(file_note("pic.png", png.to_str().unwrap()).contains("unavailable or unsupported"));
        assert!(file_note("gone.txt", "/no/such/file").contains("unavailable or unsupported"));
        let bin = dir.join("blob.bin");
        std::fs::write(&bin, [0xff, 0xfe, 0x00, 0x80]).unwrap();
        assert!(file_note("blob.bin", bin.to_str().unwrap()).contains("unavailable or unsupported"));
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn attachment_collection_keeps_multiple_texts_and_folder_contents_on_later_turns(){
        let fixture=crate::pet_documents::fixtures::Files::new();fixture.write("a.txt",b"first sentinel");fixture.write("b.txt",b"second sentinel");fixture.write("folder/nested.txt",b"folder sentinel");let context=fixture.context(&[("a.txt","same.txt"),("b.txt","same.txt"),("folder","folder")]);let text=user_text(false,Some(&context),"compare all");
        for sentinel in ["first sentinel","second sentinel","folder sentinel","fixture-1","fixture-2","fixture-3"]{assert!(text.contains(sentinel),"{sentinel}");}assert!(text.ends_with("compare all"));
    }

    #[test]
    fn the_custom_key_only_goes_to_the_address_it_was_entered_for() {
        let url = |u: &str| net::normalise_server_url(u).unwrap();
        let stored = serde_json::to_string(&BoundKey { url: "https://llm.example.com".into(), key: "sk-1".into() }).unwrap();
        assert_eq!(key_for(&stored, &url("https://llm.example.com")).as_deref(), Some("sk-1"));
        assert_eq!(key_for(&stored, &url("https://attacker.example")), None);
        assert_eq!(key_for(&stored, &url("http://llm.example.com")), None);
        // A plain string (the page can write one through secret_set) binds nothing.
        assert_eq!(key_for("sk-raw", &url("https://llm.example.com")), None);
        let local = serde_json::to_string(&BoundKey { url: "http://127.0.0.1:8080".into(), key: "k".into() }).unwrap();
        assert_eq!(key_for(&local, &url("http://127.0.0.1:8080")).as_deref(), Some("k"));
        assert!(!may_carry_key(&url("http://192.168.1.20:8080")));
    }
}


#[cfg(test)]
mod ollama_identity_tests {
    use super::*;
    #[test]
    fn query_is_last_exactly_once_with_model_stream_and_history_unchanged() {
        let history = vec![json!({"role":"user","content":"earlier"}), json!({"role":"assistant","content":"previous response"})];
        let user = json!({"role":"user","content":"SYNTHETIC-QUERY-ONCE: Please reply in English. `cargo check` and ```rust\nfn main() {}\n```"});
        let before = history.clone();
        let generic = request_body("same-model:tag", "old identity", &history, &user);
        let mut body = request_body("same-model:tag", "captured identity; pt-PT unless another language requested", &history, &user);
        assert_eq!(body["messages"].as_array().unwrap().len(), 4);
        assert_eq!(body["messages"][3], user);
        assert_eq!(body["messages"][1], history[0]);
        assert_eq!(body["messages"][2], history[1]);
        assert_eq!(body["messages"].as_array().unwrap().iter().filter(|m|m["content"]==user["content"]).count(), 1);
        assert_eq!(history, before);
        assert_eq!(body["model"], "same-model:tag");
        assert_eq!(body["stream"], true);
        assert!(body.get("tools").is_none());
        body["messages"][0]["content"] = json!("old identity");
        assert_eq!(body, generic);
    }
}
