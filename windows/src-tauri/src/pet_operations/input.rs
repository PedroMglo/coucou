use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{Failure, Operation};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="action", rename_all="snake_case", deny_unknown_fields)]
pub(crate) enum Music {
    PlayPause, Pause, Play, Next, Previous,
    SetVolume { volume: u8 },
    Search { query: String },
    PlayItem { uri: String },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Email { pub recipient: String, pub subject: String, pub body: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct App { name: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
pub(crate) enum Arguments {
    Roster(Value), Snapshot, Music(Music), OpenApp(String), Email(Email),
}
fn text(s: &str, max: usize, multiline: bool, empty: bool) -> bool {
    (empty || !s.trim().is_empty()) && s.chars().count() <= max
        && !s.chars().any(|c| c.is_control() && !(multiline && matches!(c, '\n'|'\r'|'\t')))
}
pub(super) fn parse(operation: Operation, input: &Value) -> Result<Arguments, Failure> {
    // A bounded counting writer avoids allocating another unbounded JSON copy.
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.checked_sub(bytes.len()).ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    serde_json::to_writer(Counter(super::MAX_INPUT_BYTES), input).map_err(|_| Failure::InvalidArguments)?;
    let invalid = |_| Failure::InvalidArguments;
    match operation {
        Operation::Roster => if super::roster::valid_request(input) { Ok(Arguments::Roster(input.clone())) } else { Err(Failure::InvalidArguments) },
        Operation::Snapshot => { serde_json::from_value::<Empty>(input.clone()).map_err(invalid)?; Ok(Arguments::Snapshot) },
        Operation::Music => {
            let value: Music = serde_json::from_value(input.clone()).map_err(invalid)?;
            let valid = match &value {
                Music::SetVolume { volume } => *volume <= 100,
                Music::Search { query } => text(query, 512, false, false),
                Music::PlayItem { uri } => crate::spotify::valid_item_uri(uri),
                // Serde's internally tagged unit variants may otherwise ignore
                // extra fields even with deny_unknown_fields on the enum.
                _ => input.as_object().is_some_and(|object| object.len() == 1),
            };
            if valid { Ok(Arguments::Music(value)) } else { Err(Failure::InvalidArguments) }
        },
        Operation::OpenApp => {
            let value: App = serde_json::from_value(input.clone()).map_err(invalid)?;
            if text(&value.name, 128, false, false) { Ok(Arguments::OpenApp(value.name)) } else { Err(Failure::InvalidArguments) }
        },
        Operation::Email => {
            let value: Email = serde_json::from_value(input.clone()).map_err(invalid)?;
            let parts: Vec<_> = value.recipient.split('@').collect();
            if text(&value.recipient, 320, false, false) && parts.len() == 2 && parts.iter().all(|s| !s.is_empty())
                && !value.recipient.chars().any(|c| c.is_whitespace() || matches!(c, ','|';'|'<'|'>'))
                && text(&value.subject, 256, false, true) && text(&value.body, 7000, true, true) {
                Ok(Arguments::Email(value))
            } else { Err(Failure::InvalidArguments) }
        },
    }
}
pub(crate) fn encode(text: &str) -> String {
    text.bytes().map(|b| match b {
        b'A'..=b'Z'|b'a'..=b'z'|b'0'..=b'9'|b'-'|b'_'|b'.'|b'~' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}
impl Email {
    pub(crate) fn uri(&self) -> String {
        format!("mailto:{}?subject={}&body={}", encode(&self.recipient).replace("%40", "@"), encode(&self.subject), encode(&self.body))
    }
}
