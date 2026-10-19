//! Access to sessions exposed by the opt-in extension inside Copilot CLI.
//!
//! This module never launches an ACP process, replays a transcript, imports a
//! terminal index, or treats an arbitrary socket path as a conversation target.

pub mod rpc;

pub use rpc::{cancel, history, list, send, subscribe, Subscription};
