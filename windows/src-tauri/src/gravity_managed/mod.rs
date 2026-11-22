//! Isolated next-stage Gravity owner. Deliberately not registered in lib.rs.
//!
//! A new native child owns one conversation, independently of a webview. There
//! is no external-session adoption, resume, terminal handoff or public approval
//! transport. The historical journal store is isolated from execution; private
//! facade DTOs are not registered as application commands.
#![cfg(target_os = "linux")]

pub(crate) mod durability;
pub(crate) mod events;
pub(crate) mod facade;
pub(crate) mod gate;
pub(crate) mod journal;
pub(crate) mod owner;
pub(crate) mod persistence;
pub(crate) mod transport;
pub(crate) mod types;
