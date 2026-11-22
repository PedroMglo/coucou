//! Native-only planner context and remaining budget. A snapshot is not an IO
//! permit: reservation still rechecks the original grant at the HTTP boundary.
use super::{planning_feedback::Feedback,planning_journal::Request};
use serde::Serialize;
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(super)struct Remaining {
    pub model_calls:u16,pub tool_calls:u16,pub output_bytes:u64,pub deadline_at:u64,
    pub rounds:u8,pub consecutive_no_progress:u8,pub best_verified_criteria:usize,
    pub max_steps_this_round:u8,pub model_output_limit:u64,
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(super)struct Observation {pub feedback:Feedback,pub snapshot:Option<crate::spotify::goal::Snapshot>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(super)struct Input {pub remaining:Remaining,pub observations:Vec<Observation>}
/// Deliberately neither Clone nor Deserialize. Only native grant inspection
/// creates this value; callers/models cannot supply remaining budget/history.
pub(super)struct Context {pub request:Request,pub input:Input}
