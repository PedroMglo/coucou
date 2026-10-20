//! System controls are an independent native UI surface; no Spoty entitlement.
//! Per-application streams stay with Spoty. This module performs no device IO.
use super::{Snapshot,Node,Request,Action,Kind};
use serde::Serialize;

pub(super) fn allowed(correct_window:bool,visible:bool,enabled:bool)->bool {
    correct_window && visible && enabled
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct SystemSnapshot {
    pub scope: &'static str,
    pub available: bool,
    pub observed_at: Option<u64>,
    pub outputs: Vec<Node>,
    pub inputs: Vec<Node>,
    pub cards: Vec<Node>,
    pub error: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct SystemReceipt {
    pub snapshot: SystemSnapshot,
    pub confirmed: bool,
    pub warning: Option<String>,
}
/// Do not serialize absent stream lists as an observed global zero.
pub(super) fn devices(value:Snapshot)->SystemSnapshot {
    SystemSnapshot{scope:"systemDevices",available:value.available,observed_at:value.observed_at,
        outputs:value.outputs,inputs:value.inputs,cards:value.cards,error:value.error}
}
/// The shared engine still re-reads target identity and validates the exact
/// operation. A device key never authorizes per-application operations here.
pub(super) fn request_allowed(snapshot:&Snapshot,request:&Request,still_allowed:&(dyn Fn()->bool+Send+Sync))->bool {
    still_allowed() && snapshot.available && request.action!=Action::Route && snapshot.nodes().any(|n|
        n.key==request.target_key && n.controllable && matches!(n.kind,Kind::Output|Kind::Input|Kind::Card))
}
#[cfg(test)]
#[path="system_policy_tests.rs"]
mod tests;
