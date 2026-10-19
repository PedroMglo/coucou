//! Existing Gravity Language Server sessions through their captured local bridge.
pub mod client;
pub mod bridge;

use serde_json::Value;
use crate::session_bridge::{self, GRAVITY};
use crate::session_chat::types::{Snapshot, Summary, Target};
pub use crate::session_bridge::Subscription;

pub async fn list() -> Vec<Summary> { session_bridge::list(GRAVITY).await }
pub async fn history(target: &Target) -> Result<Snapshot, String> { session_bridge::history(GRAVITY,target).await }
pub async fn send(target: &Target, query: &str, model: Option<&str>) -> Result<Value, String> {
    if model.is_some() { return Err("Muda o modelo desta sessão no cliente Gravity nativo; o painel preserva a sua configuração.".into()); }
    session_bridge::send(GRAVITY,target,query).await
}
pub async fn cancel(target: &Target, turn: Option<&str>) -> Result<Value, String> { session_bridge::cancel(GRAVITY,target,turn).await }
pub async fn subscribe(target: &Target) -> Result<Subscription, String> { session_bridge::subscribe(GRAVITY,target).await }
