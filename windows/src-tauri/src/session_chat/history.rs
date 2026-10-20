//! Public conversation projection of Codex app-server history.
//!
//! The transport must supply every turn in order and load every turn's items.
//! Only the two explicit public message variants are projected. No other item
//! field is ever treated as text, including tool output and reasoning summaries.

use std::collections::HashSet;
use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

use super::types::{Capabilities, Message, Role, Snapshot, Summary, Target};

pub const MAX_HISTORY_BYTES: usize = 128 * 1024 * 1024;

const INVALID: &str = "O terminal devolveu um histórico de sessão inválido.";
const TOO_LARGE: &str =
    "O histórico excede o limite de 128 MiB; não foi apresentado um histórico parcial.";
const INCOMPLETE: &str =
    "O terminal ainda não forneceu todos os itens desta conversa. Volta a carregar o histórico.";

/// Extract public text while retaining chronological order and stable identities.
/// An incomplete or oversized history is an error rather than a partial success.
pub fn messages(turns: &[Value]) -> Result<Vec<Message>, String> {
    bounded(turns)?;
    let mut result = Vec::new();
    let mut turn_ids = HashSet::new();
    let mut item_ids = HashSet::new();
    let mut output_budget = Budget(2); // JSON array brackets.
    for turn in turns {
        let turn_id = identifier(turn.get("id"))?;
        if !turn_ids.insert(turn_id) {
            return Err(INVALID.into());
        }
        // Older servers omit itemsView; the protocol default is explicitly full.
        match turn.get("itemsView") {
            None => {}
            Some(Value::String(view)) if view == "full" => {}
            Some(Value::String(view)) if view == "summary" || view == "notLoaded" => {
                return Err(INCOMPLETE.into());
            }
            _ => return Err(INVALID.into()),
        }
        let items = turn
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| INVALID.to_string())?;
        for item in items {
            let kind = item
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| INVALID.to_string())?;
            let role = match kind {
                "userMessage" => Role::User,
                "agentMessage" => Role::Assistant,
                _ => continue,
            };
            let item_id = identifier(item.get("id"))?;
            if !item_ids.insert((turn_id, item_id)) {
                return Err(INVALID.into());
            }
            let content = if role == Role::User {
                user_text(item)?
            } else {
                item.get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| INVALID.to_string())?
                    .to_owned()
            };
            if content.is_empty() {
                continue;
            }
            // A JSON tuple avoids collisions even when an item ID has separators.
            let id = serde_json::to_string(&(turn_id, item_id))
                .map_err(|_| INVALID.to_string())?;
            let message = Message { id, role, content };
            if !result.is_empty() {
                output_budget.write_all(b",").map_err(|_| TOO_LARGE.to_string())?;
            }
            serde_json::to_writer(&mut output_budget, &message)
                .map_err(|_| TOO_LARGE.to_string())?;
            result.push(message);
        }
    }
    Ok(result)
}

/// Build a list entry from thread metadata; `history` advertises a readable
/// history endpoint, not a claim that thread/list included its turn contents.
pub fn summary(thread: &Value, target: Target) -> Result<Summary, String> {
    bounded(thread)?;
    validate_thread(thread, &target)?;
    let state = state(thread)?;
    let title = nonempty(thread.get("name"))
        .or_else(|| nonempty(thread.get("preview")))
        .unwrap_or("Sessão Codex")
        .to_owned();
    let cwd = thread
        .get("cwd")
        .and_then(Value::as_str)
        .ok_or_else(|| INVALID.to_string())?
        .to_owned();
    let updated_at = thread
        .get("updatedAt")
        .and_then(Value::as_i64)
        .ok_or_else(|| INVALID.to_string())?;
    let turns = thread.get("turns").and_then(Value::as_array);
    let active = match turns {
        Some(turns) => active_turn(turns)?,
        None => ActiveTurns::default(),
    };
    let capabilities = capabilities(thread, state, &active);
    let model = nonempty(thread.get("model")).map(str::to_owned);
    Ok(Summary {
        target,
        title,
        cwd,
        updated_at,
        state: state.to_owned(),
        model,
        capabilities,
    })
}

/// Project a fully loaded history. `turns` is separate because the transport
/// paginates full turn/item responses rather than relying on metadata previews.
pub fn snapshot(thread: &Value, target: Target, turns: &[Value]) -> Result<Snapshot, String> {
    validate_thread(thread, &target)?;
    let state = state(thread)?;
    let messages = messages(turns)?;
    let active = active_turn(turns)?;
    let turn = active.exact().map(str::to_owned);
    let busy = state == "active" || active.id.is_some();
    let capabilities = capabilities(thread, state, &active);
    let notice = if active.conflict {
        Some("O terminal anunciou vários turnos ativos. Podes consultar o histórico; o envio e a interrupção ficam indisponíveis até existir um turno inequívoco.".into())
    } else if state == "notLoaded" {
        Some("Sessão guardada; o terminal não a tem carregada para receber mensagens.".into())
    } else if state == "systemError" {
        Some("O terminal reporta um erro de sistema nesta sessão.".into())
    } else if busy && turn.is_none() {
        Some("A sessão está ativa, mas o terminal não anunciou o turno exato para cancelar.".into())
    } else if thread
        .get("canAcceptDirectInput")
        .and_then(Value::as_bool)
        .is_none()
    {
        Some("O terminal não anunciou se esta sessão aceita mensagens diretas.".into())
    } else if !capabilities.send {
        Some("Esta sessão não aceita mensagens diretas neste terminal.".into())
    } else {
        None
    };
    let model = nonempty(thread.get("model")).map(str::to_owned);
    Ok(Snapshot {
        target,
        messages,
        busy,
        turn,
        capabilities,
        notice,
        model,
        control_conflict: active.conflict,
    })
}

fn validate_thread(thread: &Value, target: &Target) -> Result<(), String> {
    target.validate_external("codex", "agent_codex")?;
    // Thread.sessionId groups subagents and must never identify a send target.
    if thread.get("id").and_then(Value::as_str) != Some(target.session_id.as_str()) {
        return Err("O histórico recebido pertence a outra sessão externa.".into());
    }
    Ok(())
}

fn state(thread: &Value) -> Result<&str, String> {
    match thread
        .get("status")
        .and_then(|status| status.get("type"))
        .and_then(Value::as_str)
    {
        Some(state @ ("notLoaded" | "idle" | "active" | "systemError")) => Ok(state),
        _ => Err(INVALID.into()),
    }
}

fn capabilities(thread: &Value, state: &str, active: &ActiveTurns<'_>) -> Capabilities {
    let loaded = state != "notLoaded";
    Capabilities {
        history: thread.get("ephemeral").and_then(Value::as_bool) != Some(true),
        // Only an explicit advertised capability permits input. A readable
        // stored thread and an idle status do not imply ownership of its input.
        send: loaded
            && !active.conflict
            && thread.get("ephemeral").and_then(Value::as_bool) != Some(true)
            && state != "systemError"
            && thread.get("canAcceptDirectInput").and_then(Value::as_bool) == Some(true),
        cancel: loaded && active.exact().is_some(),
    }
}

// Public history remains readable when runtime control has no exact turn.
// This state never selects a turn by order or recency.
#[derive(Default)]
struct ActiveTurns<'a> {
    id: Option<&'a str>,
    conflict: bool,
}
impl<'a> ActiveTurns<'a> {
    fn exact(&self) -> Option<&'a str> {
        if self.conflict { None } else { self.id }
    }
}

fn active_turn(turns: &[Value]) -> Result<ActiveTurns<'_>, String> {
    let mut active = ActiveTurns::default();
    let mut active_ids = HashSet::new();
    for turn in turns {
        match turn.get("status").and_then(Value::as_str) {
            Some("inProgress") => {
                let id = identifier(turn.get("id"))?;
                if !active_ids.insert(id) { return Err(INVALID.into()); }
                if let Some(previous) = active.id {
                    active.conflict |= previous != id;
                } else {
                    active.id = Some(id);
                }
            }
            Some("completed" | "interrupted" | "failed") => {}
            _ => return Err(INVALID.into()),
        }
    }
    Ok(active)
}

fn user_text(item: &Value) -> Result<String, String> {
    let blocks = item
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| INVALID.to_string())?;
    let mut text = String::new();
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let part = block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| INVALID.to_string())?;
                if !text.is_empty() && !part.is_empty() {
                    text.push('\n');
                }
                text.push_str(part);
            }
            Some(_) => {} // Do not expose attachment paths, audio URLs, or skill bodies.
            None => return Err(INVALID.into()),
        }
    }
    Ok(text)
}

fn identifier(value: Option<&Value>) -> Result<&str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| {
            !value.is_empty() && value.len() <= 1024 && !value.chars().any(char::is_control)
        })
        .ok_or_else(|| INVALID.to_string())
}

fn nonempty(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

/// Enforce a byte budget without constructing a second copy of private history.
fn bounded<T: Serialize + ?Sized>(value: &T) -> Result<(), String> {
    serde_json::to_writer(Budget(0), value).map_err(|_| TOO_LARGE.to_string())
}

struct Budget(usize);

impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|size| *size <= MAX_HISTORY_BYTES)
            .ok_or_else(|| io::Error::other("history budget exceeded"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
