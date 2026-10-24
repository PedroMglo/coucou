//! Values shared by the native session bridge and the conversation surface.
//!
//! A target identifies one native thread. In Codex, `sessionId` here is the
//! thread's `id`, not the `sessionId` shared by a whole subagent tree.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Mochi,
    External,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub pet_id: String,
    pub provider: String,
    pub origin: Origin,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

impl Target {
    /// Bind an external request to the pet/provider selected by native config.
    /// Callers must pass authoritative values rather than values from this target.
    pub fn validate_external(&self, provider: &str, pet_id: &str) -> Result<(), String> {
        if self.origin != Origin::External {
            return Err("O destino não é uma sessão externa.".into());
        }
        if provider.is_empty()
            || pet_id.is_empty()
            || self.provider != provider
            || self.pet_id != pet_id
        {
            return Err("A sessão não pertence ao pet e fornecedor selecionados.".into());
        }
        if self.conversation_id.is_some() {
            return Err("Uma sessão externa não pode indicar uma conversa Mochi.".into());
        }
        if !is_uuid(&self.session_id) {
            return Err("O identificador da sessão externa é inválido.".into());
        }
        Ok(())
    }
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub history: bool,
    pub send: bool,
    pub cancel: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub target: Target,
    pub messages: Vec<Message>,
    pub busy: bool,
    /// Backend-only barrier: live events cannot grant control over ambiguous
    /// turns in an otherwise valid public history. Never accepted from IPC.
    #[serde(skip)]
    pub(crate) control_conflict: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<String>,
    pub capabilities: Capabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Backend-authored passive metadata. Peers cannot inject this projection.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub capability_descriptor: Option<crate::pet_capabilities::TargetDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub target: Target,
    pub title: String,
    pub cwd: String,
    /// Unix timestamp in seconds, matching Codex's `Thread.updatedAt`.
    pub updated_at: i64,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub capabilities: Capabilities,
}

/// Runtime provenance belongs to a session created by Mochi on the existing
/// daemon. This is separate from ordinary external history and its targets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedPhase {
    Draft,
    Ready,
    Active,
    NeedsNativeClient,
    Lost,
    Released,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSession {
    pub target: Target,
    pub handle: String,
    pub generation: u64,
    pub phase: ManagedPhase,
    pub turn: Option<String>,
    pub materialized: bool,
    /// A launch acknowledgement, not proof that a TUI is still connected.
    pub terminal_launched: bool,
    pub cwd: String,
    pub model: Option<String>,
    pub notice: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagedReference {
    pub target: Target,
    pub handle: String,
    pub generation: u64,
}
