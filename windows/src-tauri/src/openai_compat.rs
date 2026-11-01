// Cloud providers that speak the OpenAI chat completions API: OpenAI, Google AI
// (Gemini's OpenAI-compatible endpoint, as ClaudeService.swift uses it) and
// OpenRouter. One table describes them; one client talks to all three.
//
// Same contract as claude.rs: the key never leaves the credential store and
// file bytes never cross the IPC boundary. A selected ICS additionally exposes
// a sealed read-only calendar query; it never grants machine actions.

use std::time::Duration;

use reqwest::Url;
use serde_json::{json, Value};

use crate::chat::{self, Chat, ChatContext, ChatReply, ModelInfo};
use crate::i18n::{t, tf};
use crate::{net, secrets};

pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    /// Credential store entry of its API key.
    pub key: &'static str,
    /// Relative to `base_url`.
    pub models_path: &'static str,
    pub default_model: &'static str,
    /// Model ids containing any of these are not chat models.
    pub not_chat: &'static [&'static str],
    /// OpenAI's reasoning models refuse `max_tokens` and want
    /// `max_completion_tokens`; the compatible APIs take the older name.
    pub max_tokens_field: &'static str,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        key: "openai-api-key",
        models_path: "models",
        default_model: "gpt-4o",
        not_chat: &[
            "embed", "tts", "whisper", "dall-e", "audio", "realtime", "moderat", "codex",
            "computer-use", "transcribe", "image", "sora", "babbage", "davinci", "instruct",
        ],
        max_tokens_field: "max_completion_tokens",
    },
    Provider {
        id: "google",
        name: "Google AI",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        key: "google-api-key",
        models_path: "models",
        default_model: "gemini-2.0-flash",
        not_chat: &["embed", "imagen", "veo", "aqa", "tts", "audio", "live"],
        max_tokens_field: "max_tokens",
    },
    Provider {
        id: "openrouter",
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        key: "openrouter-api-key",
        models_path: "models",
        default_model: "openrouter/auto",
        not_chat: &[],
        max_tokens_field: "max_tokens",
    },
];

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}

const MAX_TOKENS: u32 = 4096;
/// Text and code files are inlined; anything larger is skipped.
const MAX_INLINE_TEXT: u64 = 200_000;
/// Images are sent inline as data URLs; past this size they are skipped.
const MAX_IMAGE: u64 = 5_000_000;

/// Shared dispatch/review route choice. A preview describes the known set,
/// not provider-side tools, model aliases or internal routing.
pub(crate)fn completion_tail(provider:&str,calendar:bool)->&'static str{if provider=="openai"&&calendar{"responses"}else{"chat/completions"}}
fn url(p: &Provider, tail: &str) -> Result<Url, String> {
    let base = Url::parse(p.base_url).map_err(|e| e.to_string())?;
    Url::parse(&net::join(&base, tail)).map_err(|e| e.to_string())
}

/// The user's message for one turn: plain text, or text and an image part
/// when the first turn carries a dropped picture.
fn user_message(first: bool, context: Option<&ChatContext>, query: &str) -> Value {
    if let Some(ChatContext::Files{files})=context{
        let mut parts=vec![];let mut budget=crate::pet_documents::content::Budget::default();
        for (index,file) in files.iter().enumerate(){
            let (name,path)=(file.name.as_str(),file.path.as_str());parts.push(json!({"type":"text","text":file.label(index)}));
            let part=if !budget.available(){crate::pet_documents::content::unavailable(name,"combined extraction deadline")}else{match file_part(path){
                Some(FilePart::Image(part))=>budget.block(part,name),
                Some(FilePart::Text(text))=>json!({"type":"text","text":budget.text(format!("File contents:\n{text}"))}),
                None if std::path::Path::new(path).is_dir()||std::path::Path::new(path).extension().is_some_and(|e|e.eq_ignore_ascii_case("pdf"))=>json!({"type":"text","text":budget.text(crate::local_chat::file_note(name,path))}),
                None=>crate::pet_documents::content::unavailable(name,"unsupported format, size or unreadable copy"),
            }};parts.push(part);
        }
        parts.push(json!({"type":"text","text":query}));return json!({"role":"user","content":parts});
    }
    let mut notes=vec![];let mut media=vec![];let mut budget=crate::pet_documents::content::Budget::default();
    if let Some(context)=context.filter(|c|first||matches!(c,ChatContext::Files{..})){
        if let ChatContext::Window{app_name,title,url}=context{notes.push(chat::window_line(app_name,title,url.as_deref()));}
        for (name,path) in context.file_paths(){
            if !budget.available(){notes.push(crate::pet_documents::content::unavailable(name,"combined extraction deadline")["text"].as_str().unwrap().into());continue;}
            match file_part(path){
                Some(FilePart::Image(part))=>{let part=budget.block(part,name);if part["type"]=="text"{notes.push(part["text"].as_str().unwrap().into());}else{media.push(part);}notes.push(format!("File: {name}"));}
                Some(FilePart::Text(body))=>notes.push(budget.text(format!("File: {name}\nFile contents:\n{body}"))),
                None=>notes.push(format!("File: {name}")),
            }
        }
    }
    notes.push(query.into());let text=notes.join("\n\n");
    if media.is_empty(){json!({"role":"user","content":text})}else{media.push(json!({"type":"text","text":text}));json!({"role":"user","content":media})}
}

fn request_body(p: &Provider, model: &str, system: &str, history: &[Value], user: &Value) -> Value {
    let mut messages = vec![json!({ "role": "system", "content": system })];
    messages.extend(history.iter().cloned());
    messages.push(user.clone());
    let mut body = json!({ "model": model, "messages": messages });
    body[p.max_tokens_field] = json!(MAX_TOKENS);
    body
}

/// The answer's text, or the error the provider put in a 200 reply
/// (OpenRouter does when the upstream model fails).
fn reply_text(p: &Provider, response: &Value) -> Result<String, String> {
    let text = response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if !text.is_empty() {
        return Ok(text);
    }
    if let Some(msg) = response.pointer("/error/message").and_then(Value::as_str) {
        return Err(format!("{}: {msg}", p.name));
    }
    Err(t("No response text."))
}

fn status_error(p: &Provider, status: u16, detail: &str) -> String {
    match status {
        401 | 403 => tf(
            "{name} rejected the API key ({status}). Check it in Settings.",
            &[("name", p.name), ("status", &status.to_string())],
        ),
        402 => tf("{name}: not enough credits (402). {detail}", &[("name", p.name), ("detail", detail)]),
        404 => tf(
            "{name}: model not found (404). Pick another one above the chat box. {detail}",
            &[("name", p.name), ("detail", detail)],
        ),
        429 => tf("{name} rate limit reached (429): {detail}", &[("name", p.name), ("detail", detail)]),
        _ => format!("{} {status}: {detail}", p.name),
    }
}

/// One chat turn with a cloud provider.
pub async fn send(
    chat: &Chat,
    p: &'static Provider,
    model: &str,
    query: String,
    context: Option<ChatContext>,
) -> Result<ChatReply, String> {
    let key = secrets::get(p.key).ok_or_else(|| tf("{name} API key missing. Add it in Settings.", &[("name", p.name)]))?;
    send_resolved(chat,p,model,query,context,&key).await
}

// Production supplies its original native Provider and credential. Test-only
// callers use synthetic resolved inputs through the same complete send path.
async fn send_resolved(chat:&Chat,p:&'static Provider,model:&str,query:String,context:Option<ChatContext>,key:&str)->Result<ChatReply,String>{
    chat.check_new_effect()?;
    if model.is_empty() {
        return Err(tf("Pick a {name} model above the chat box.", &[("name", p.name)]));
    }
    let turn = chat.begin_cli(p.id,std::sync::Arc::default())?;
    let _lease=chat.lease(&turn);
    let documents=crate::pet_documents::Request::capture_async(chat,&turn,context.as_ref()).await?;
    let calendar = crate::pet_calendar::note_async(chat, &turn, context.as_ref()).await?;
    let ordinary=crate::pet_calendar::ordinary_context(context.as_ref());let (first,question)=(turn.first,query.clone());
    let user=tokio::task::spawn_blocking(move||user_message(first,ordinary.as_ref(),&question)).await.map_err(|_|"A preparação dos anexos foi interrompida.".to_string())?;
    let user=crate::pet_calendar::prepend_message(calendar.as_ref(),user)?;
    let mut body = request_body(p, model, &chat::system_prompt(false), &turn.history, &user);
    chat.append_selected_memory(p.id,&query,&crate::pet_memory::context_model::Destination::Api{provider:p.id.into(),endpoint:Url::parse(p.base_url).map_err(|_|"Destino inválido.")?.to_string(),model:model.into()},Some(key),body["messages"].as_array_mut().unwrap())?;
    use crate::pet_calendar::wire::{Session,Format};
    let format = if p.id == "openai" {Format::Responses} else {Format::Chat};
    let mut calendar_tools = Session::new(calendar.as_ref(),format);
    if let Some(tools) = &calendar_tools { tools.prepare(&mut body)?; }

    let endpoint = url(p,completion_tail(p.id,calendar_tools.is_some()))?;
    let response = loop {
        documents.validate_contents().await?;
        crate::pet_calendar::validate(calendar.as_ref())?;
        let response = net::while_current(
            || chat.is_current(&turn) && documents.validate().is_ok() && crate::pet_calendar::validate(calendar.as_ref()).is_ok()
                && calendar_tools.as_ref().is_none_or(|tools|tools.validate().is_ok()),
            || async {chat.check_new_effect()?;call(p,&endpoint,key,&body).await},
        ).await?;
        if let Some(tools) = &mut calendar_tools {
            if tools.follow_up(&response,&mut body).await? {continue;}
        }
        break response;
    };
    let text = if calendar_tools.is_some() && format == Format::Responses {
        crate::pet_calendar::wire::responses_text(&response)?
    } else { reply_text(p,&response)? };

    documents.validate_contents().await?;crate::pet_calendar::validate(calendar.as_ref())?;
    let plain = chat::plain_question(turn.first, context.as_ref(), &query);
    chat.commit_reply(&turn, user, json!({ "role": "assistant", "content": text }), &plain, &text)?;
    Ok(ChatReply { text })
}

#[cfg(test)]
pub(crate)async fn memory_wire_fixture(chat:&Chat,p:&'static Provider,model:&str,query:String,context:Option<ChatContext>,key:&str)->Result<ChatReply,String>{
    send_resolved(chat,p,model,query,context,key).await
}

async fn call(p: &Provider, endpoint: &Url, key: &str, body: &Value) -> Result<Value, String> {
    let response = net::exact_destination_client(endpoint, Duration::from_secs(90))?
        .post(endpoint.clone())
        .bearer_auth(key)
        .json(body)
        .send()
        .await
        .map_err(|e| tf("Network error: {error}", &[("error", &e.to_string())]))?;
    let status = response.status();
    if !status.is_success() {
        let body = net::read_capped(response, net::MAX_ERROR_BODY).await.unwrap_or_default();
        return Err(status_error(p, status.as_u16(), &net::error_detail(&body)));
    }
    let bytes = net::read_capped(response, net::MAX_BODY).await?;
    serde_json::from_slice(&bytes).map_err(|e| tf("Bad API response: {error}", &[("error", &e.to_string())]))
}

/// The provider's chat models. Only ever asked with the user's key, once they
/// picked this provider in the chat.
pub async fn models(p: &Provider, key: &str) -> Result<Vec<ModelInfo>, String> {
    let endpoint = url(p, p.models_path)?;
    let response = net::exact_destination_client(&endpoint, Duration::from_secs(15))?
        .get(endpoint)
        .bearer_auth(key)
        .send()
        .await
        .map_err(|e| tf("Network error: {error}", &[("error", &e.to_string())]))?;
    let status = response.status();
    if !status.is_success() {
        let body = net::read_capped(response, net::MAX_ERROR_BODY).await.unwrap_or_default();
        return Err(status_error(p, status.as_u16(), &net::error_detail(&body)));
    }
    let bytes = net::read_capped(response, net::MAX_BODY).await?;
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| t("Unexpected API response."))?;
    Ok(parse_models(p, &json))
}

fn parse_models(p: &Provider, json: &Value) -> Vec<ModelInfo> {
    let items = json.get("data").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut models: Vec<(ModelInfo, i64, bool)> = items
        .iter()
        .filter(|m| {
            // OpenRouter lists image and audio generators too.
            m.pointer("/architecture/output_modalities")
                .and_then(Value::as_array)
                .is_none_or(|mods| mods.iter().any(|x| x.as_str() == Some("text")))
        })
        .filter_map(|m| {
            let raw = m.get("id")?.as_str()?;
            // Gemini sometimes answers "models/gemini-…".
            let id = raw.strip_prefix("models/").unwrap_or(raw).to_string();
            let lower = id.to_lowercase();
            if id.is_empty() || p.not_chat.iter().any(|x| lower.contains(x)) {
                return None;
            }
            let created = m.get("created").and_then(Value::as_i64).unwrap_or(0);
            let free = p.id == "openrouter" && is_free(m, &id);
            let name = m.get("name").and_then(Value::as_str).unwrap_or(&id);
            let label = if free && !name.to_lowercase().contains("(free)") { format!("{name} (free)") } else { name.to_string() };
            Some((ModelInfo { id, label }, created, free))
        })
        .collect();
    match p.id {
        // Newest first, as on the Mac.
        "openai" => models.sort_by_key(|m| std::cmp::Reverse(m.1)),
        // Free models first, then by name.
        "openrouter" => models.sort_by(|a, b| {
            b.2.cmp(&a.2).then_with(|| a.0.label.to_lowercase().cmp(&b.0.label.to_lowercase()))
        }),
        _ => {}
    }
    models.into_iter().map(|(m, _, _)| m).collect()
}

fn is_free(m: &Value, id: &str) -> bool {
    let zero = |ptr: &str| {
        m.pointer(ptr)
            .and_then(Value::as_str)
            .is_some_and(|v| v.parse::<f64>().ok() == Some(0.0))
    };
    id.ends_with(":free") || (zero("/pricing/prompt") && zero("/pricing/completion"))
}

enum FilePart {
    Image(Value),
    Text(String),
}

/// Images → image_url data URL, text/code → inline text. PDFs and other binary
/// files go by name only (there is no portable document part).
fn file_part(path: &str) -> Option<FilePart> {
    if crate::pet_documents::policy::format(path,false)==crate::pet_documents::policy::Format::Binary{return None;}
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let len = std::fs::metadata(path).ok()?.len();
    let image = match ext.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    };
    if let Some(media) = image {
        if len > MAX_IMAGE {
            return None;
        }
        let bytes = crate::pet_documents::content::bytes(path,MAX_IMAGE as usize)?;
        let url = format!("data:{media};base64,{}", crate::claude::base64_for(&bytes));
        return Some(FilePart::Image(json!({ "type": "image_url", "image_url": { "url": url } })));
    }
    if ext == "pdf" || len > MAX_INLINE_TEXT {
        return None;
    }
    crate::pet_documents::content::text(path,MAX_INLINE_TEXT as usize).map(FilePart::Text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_folder_context_preserves_text_and_honest_unsupported_members(){
        let files=crate::pet_documents::fixtures::Files::new();files.write("folder/nested/notes.txt",b"cloud folder sentinel");files.write("folder/report.docx",b"not supported as text");
        let context=files.context(&[("folder","Folder")]);let message=user_message(false,Some(&context),"use this folder");let text=message.to_string();assert!(text.contains("cloud folder sentinel"));assert!(text.contains("report.docx"));assert!(text.contains("unsupported for text extraction"));assert!(!text.contains("not supported as text"));
        files.write("legacy.docx",b"apparently UTF8 binary format");assert!(file_part(files.root.join("legacy.docx").to_str().unwrap()).is_none());
        let policy:Value=serde_json::from_str(include_str!("../documents.policy.json")).unwrap();for extension in policy["images"].as_array().unwrap(){let name=format!("image.{}",extension.as_str().unwrap());files.write(&name,b"synthetic image bytes");assert!(matches!(file_part(files.root.join(&name).to_str().unwrap()),Some(FilePart::Image(_))),"{name}");}
    }
    #[test]
    fn attachment_collection_preserves_ordered_images_text_and_unique_labels_after_first_turn(){
        let fixture=crate::pet_documents::fixtures::Files::new();fixture.write("a.png",b"first-image");fixture.write("b.png",b"second-image");fixture.write("notes.txt",b"unique document sentinel");
        let context=fixture.context(&[("a.png","same.png"),("b.png","same.png"),("notes.txt","notes")]);let message=user_message(false,Some(&context),"compare all three");let parts=message["content"].as_array().unwrap();assert_eq!(parts.len(),7);
        assert!(parts[0]["text"].as_str().unwrap().contains("fixture-1"));assert!(parts[2]["text"].as_str().unwrap().contains("fixture-2"));assert_eq!(parts[1]["type"],"image_url");assert_eq!(parts[3]["type"],"image_url");assert_ne!(parts[1]["image_url"]["url"],parts[3]["image_url"]["url"]);assert!(parts[5]["text"].as_str().unwrap().contains("unique document sentinel"));assert_eq!(parts[6]["text"],"compare all three");
        let body=request_body(p("google"),"same-model","same-system",&[],&message);assert_eq!(body["messages"][1],message);assert_eq!(body["model"],"same-model");
    }
    #[test]
    fn attachment_collection_reports_missing_content_without_omitting_the_other_documents(){
        let fixture=crate::pet_documents::fixtures::Files::new();fixture.write("ok.txt",b"retained source");let context=fixture.context(&[("missing.bin","missing"),("ok.txt","kept")]);let message=user_message(true,Some(&context),"q");let parts=message["content"].as_array().unwrap();assert!(parts[1]["text"].as_str().unwrap().contains("content unavailable"));assert!(parts[3]["text"].as_str().unwrap().contains("retained source"));assert_eq!(parts.last().unwrap()["text"],"q");
    }

    #[tokio::test]
    async fn turn_cancellation_closes_all_compatible_provider_waits_and_bodies() {
        use crate::net::cancel_tests::{assert_http_cancelled, Stage};
        for provider in ["openai", "google", "openrouter"] {
            for stage in [Stage::Headers, Stage::Body] {
                assert_http_cancelled(stage, |endpoint| async move {
                    call(p(provider), &endpoint, "synthetic-key", &json!({"model":"synthetic","messages":[]})).await
                }).await;
            }
        }
    }

    #[tokio::test]
    async fn compatible_call_keeps_original_success_and_error_interpretation() {
        let body = br#"{"choices":[{"message":{"content":"synthetic answer"}}]}"#.to_vec();
        let endpoint = Url::parse(&crate::net::tests::serve_once("200 OK", "", body)).unwrap();
        let response = call(p("openai"), &endpoint, "synthetic-key", &json!({})).await.unwrap();
        assert_eq!(reply_text(p("openai"), &response).unwrap(), "synthetic answer");
        let endpoint = Url::parse(&crate::net::tests::serve_once("400 Bad Request", "", br#"{"error":{"message":"synthetic rejection"}}"#.to_vec())).unwrap();
        assert!(call(p("openai"), &endpoint, "synthetic-key", &json!({})).await.unwrap_err().contains("synthetic rejection"));
    }

    fn p(id: &str) -> &'static Provider {
        provider(id).unwrap()
    }

    #[test]
    fn the_table_points_at_each_providers_own_api_and_key() {
        assert_eq!(url(p("openai"), "chat/completions").unwrap().as_str(), "https://api.openai.com/v1/chat/completions");
        assert_eq!(
            url(p("google"), "chat/completions").unwrap().as_str(),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert_eq!(url(p("openrouter"), "models").unwrap().as_str(), "https://openrouter.ai/api/v1/models");
        for prov in PROVIDERS {
            assert!(crate::secrets::KNOWN_KEYS.contains(&prov.key), "{}", prov.key);
            assert!(prov.base_url.starts_with("https://"));
        }
        assert!(provider("anthropic").is_none());
        assert!(provider("ollama").is_none());
    }

    #[test]
    fn the_request_is_system_then_history_then_the_new_turn() {
        let history = vec![json!({"role":"user","content":"a"}), json!({"role":"assistant","content":"b"})];
        let user = user_message(false, None, "c");
        assert_eq!(user, json!({"role":"user","content":"c"}));
        let body = request_body(p("google"), "gemini-x", "sys", &history, &user);
        assert_eq!(body["model"], "gemini-x");
        assert_eq!(body["max_tokens"], MAX_TOKENS);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0], json!({"role":"system","content":"sys"}));
        assert_eq!(msgs[3], user);
        assert!(body.get("tools").is_none(), "chat only: no tools");
        // OpenAI's reasoning models refuse the old field name.
        let body = request_body(p("openai"), "gpt-5", "sys", &[], &user);
        assert_eq!(body["max_completion_tokens"], MAX_TOKENS);
        assert!(body.get("max_tokens").is_none());
    }

    #[test]
    fn context_rides_with_the_first_turn_as_text_or_an_image() {
        let dir = std::env::temp_dir().join(format!("coucou-oai-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let txt = dir.join("notes.txt");
        std::fs::write(&txt, "hello").unwrap();
        let ctx = ChatContext::File { name: "notes.txt".into(), path: txt.to_string_lossy().into() };
        assert_eq!(
            user_message(true, Some(&ctx), "sum up")["content"],
            "File: notes.txt\nFile contents:\nhello\n\nsum up"
        );
        assert_eq!(user_message(false, Some(&ctx), "more")["content"], "more");

        let png = dir.join("pic.png");
        std::fs::write(&png, [0x89, b'P', b'N', b'G']).unwrap();
        let ctx = ChatContext::File { name: "pic.png".into(), path: png.to_string_lossy().into() };
        let msg = user_message(true, Some(&ctx), "what is it?");
        let parts = msg["content"].as_array().unwrap();
        assert_eq!(parts[0]["type"], "image_url");
        assert!(parts[0]["image_url"]["url"].as_str().unwrap().starts_with("data:image/png;base64,"));
        assert_eq!(parts[1], json!({"type":"text","text":"File: pic.png\n\nwhat is it?"}));

        let pdf = dir.join("doc.pdf");
        std::fs::write(&pdf, "%PDF").unwrap();
        let ctx = ChatContext::File { name: "doc.pdf".into(), path: pdf.to_string_lossy().into() };
        assert_eq!(user_message(true, Some(&ctx), "q")["content"], "File: doc.pdf\n\nq");
        let _ = std::fs::remove_dir_all(dir);

        let ctx = ChatContext::Window { app_name: "Code".into(), title: "x".into(), url: None };
        assert_eq!(user_message(true, Some(&ctx), "q")["content"], "Context — App: Code, Window: x\n\nq");
    }

    #[test]
    fn the_reply_text_or_the_error_inside_a_200_is_read() {
        let ok = json!({"choices":[{"message":{"role":"assistant","content":"  Hi!  "}}]});
        assert_eq!(reply_text(p("openai"), &ok).unwrap(), "Hi!");
        let failed = json!({"error":{"message":"upstream down"}});
        assert_eq!(reply_text(p("openrouter"), &failed).unwrap_err(), "OpenRouter: upstream down");
        assert_eq!(reply_text(p("google"), &json!({"choices":[]})).unwrap_err(), "No response text.");
    }

    #[test]
    fn status_errors_say_what_to_do() {
        assert!(status_error(p("openai"), 401, "x").contains("rejected the API key"));
        assert!(status_error(p("openrouter"), 402, "x").contains("credits"));
        assert_eq!(status_error(p("google"), 500, "boom"), "Google AI 500: boom");
    }

    #[test]
    fn model_lists_keep_chat_models_only() {
        let openai = json!({"data":[
            {"id":"gpt-4o","created":100},
            {"id":"text-embedding-3-small","created":300},
            {"id":"gpt-5-mini","created":200},
            {"id":"whisper-1","created":400},
            {"id":"dall-e-3","created":500}
        ]});
        let ids: Vec<_> = parse_models(p("openai"), &openai).into_iter().map(|m| m.id).collect();
        assert_eq!(ids, vec!["gpt-5-mini", "gpt-4o"]);

        let google = json!({"data":[
            {"id":"models/gemini-2.0-flash"},
            {"id":"models/text-embedding-004"},
            {"id":"gemini-2.5-pro"},
            {"id":"models/imagen-3.0"}
        ]});
        let ids: Vec<_> = parse_models(p("google"), &google).into_iter().map(|m| m.id).collect();
        assert_eq!(ids, vec!["gemini-2.0-flash", "gemini-2.5-pro"]);

        let openrouter = json!({"data":[
            {"id":"b/paid","name":"B Paid","pricing":{"prompt":"0.000001","completion":"0.000002"}},
            {"id":"a/free:free","name":"A (free)"},
            {"id":"c/zero","name":"C Zero","pricing":{"prompt":"0","completion":"0"}},
            {"id":"d/image","name":"D Image","architecture":{"output_modalities":["image"]}}
        ]});
        let models = parse_models(p("openrouter"), &openrouter);
        let labels: Vec<_> = models.iter().map(|m| m.label.as_str()).collect();
        assert_eq!(labels, vec!["A (free)", "C Zero (free)", "B Paid"]);
    }

    #[test]
    fn an_unknown_answer_gives_an_empty_list() {
        assert!(parse_models(p("openai"), &json!({"nope":true})).is_empty());
    }
}
