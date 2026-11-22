//! Copilot facade over the reviewed shared Unix session transport.
use serde_json::Value;
use crate::session_bridge::{self, COPILOT};
use crate::session_chat::types::{Snapshot, Summary, Target};
pub use crate::session_bridge::Subscription;

pub async fn list() -> Vec<Summary> { session_bridge::list(COPILOT).await }
pub async fn history(target: &Target) -> Result<Snapshot, String> { session_bridge::history(COPILOT,target).await }
pub async fn send(target: &Target, query: &str) -> Result<Value, String> { session_bridge::send(COPILOT,target,query).await }
pub async fn cancel(target: &Target, turn: Option<&str>) -> Result<Value, String> { session_bridge::cancel(COPILOT,target,turn).await }
pub async fn subscribe(target: &Target) -> Result<Subscription, String> { session_bridge::subscribe(COPILOT,target).await }
