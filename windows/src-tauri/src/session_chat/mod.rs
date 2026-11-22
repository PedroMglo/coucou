//! Explicit native-session targets. No transcript replay or terminal automation.
pub mod types;
pub mod history;
pub mod rpc;
pub mod codex;
pub mod live;
#[cfg(test)]mod tests;

#[cfg(test)]
mod ambiguity_tests;
