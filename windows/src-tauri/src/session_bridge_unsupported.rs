//! These companion bridges authenticate a private Unix socket.
use crate::session_chat::types::{Snapshot, Summary, Target};
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct Backend(&'static str);
pub const COPILOT: Backend = Backend("Copilot");
pub const GRAVITY: Backend = Backend("Gravity");

impl Backend {
    fn unavailable(self) -> String {
        format!("A ligação nativa a sessões {} ainda não está disponível nesta plataforma.", self.0)
    }
}

pub async fn list(_: Backend) -> Vec<Summary> { Vec::new() }
pub async fn history(backend: Backend, _: &Target) -> Result<Snapshot, String> { Err(backend.unavailable()) }
pub async fn send(backend: Backend, _: &Target, _: &str) -> Result<Value, String> { Err(backend.unavailable()) }
pub async fn cancel(backend: Backend, _: &Target, _: Option<&str>) -> Result<Value, String> { Err(backend.unavailable()) }

pub enum Subscription {}
impl Subscription {
    pub async fn recv(&mut self) -> Option<Target> { match *self {} }
}
pub async fn subscribe(backend: Backend, _: &Target) -> Result<Subscription, String> { Err(backend.unavailable()) }
