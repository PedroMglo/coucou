use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind { Output, Input, Playback, Recording, Card }
impl Kind {
    pub fn stream(self) -> bool { matches!(self, Self::Playback | Self::Recording) }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice { pub id: String, pub label: String, pub available: bool }

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub key: String, pub kind: Kind, pub label: String, pub description: String,
    pub volume: Option<f64>, pub muted: Option<bool>, pub balance: Option<f64>,
    pub is_default: bool, pub device_key: Option<String>, pub monitor: bool,
    pub controllable: bool, pub can_volume: bool, pub can_balance: bool,
    pub active: bool, pub options: Vec<Choice>, pub active_option: Option<String>,
    // Native selectors are never accepted from the webview.
    #[serde(skip)] pub selector: String,
    #[serde(skip)] pub index: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub available: bool, pub observed_at: Option<u64>, pub outputs: Vec<Node>,
    pub inputs: Vec<Node>, pub playback: Vec<Node>, pub recording: Vec<Node>,
    pub cards: Vec<Node>, pub error: Option<String>,
}
impl Snapshot {
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.outputs.iter().chain(&self.inputs).chain(&self.playback).chain(&self.recording).chain(&self.cards)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Action { Volume, Mute, Default, Route, Port, Profile, Balance }
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub target_key: String, pub action: Action, pub value: Option<f64>,
    pub muted: Option<bool>, pub destination_key: Option<String>, pub option: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt { pub snapshot: Snapshot, pub confirmed: bool, pub warning: Option<String> }
