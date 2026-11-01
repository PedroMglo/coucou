//! Phone-owned fields use the common settings transaction. A rejected write
//! must not change the features read by existing connections or publish an ACK.
//! This does not authorize starting a listener, connecting a relay or using keys.
use crate::settings::{self, Settings};
use std::{io, sync::Mutex};

pub(super) enum Change {
    Link(bool),
    Relay { enabled: bool, url: String },
    RelayOff,
    Details(bool),
    Answers(bool),
    Diffs(bool),
    Usage(bool),
    Services(Vec<String>),
    Chat(bool),
    ChatModels(Vec<String>),
}

pub(super) fn commit_with(
    current: &Mutex<Settings>, change: Change,
    persist: impl FnOnce(&Settings) -> io::Result<()>,
    publish: impl FnOnce(&Settings),
) -> io::Result<Settings> {
    settings::update_with(current, |next| match change {
        Change::Link(value) => next.phone_link = value,
        Change::Relay { enabled, url } => { next.phone_relay = enabled; next.phone_relay_url = url; }
        Change::RelayOff => next.phone_relay = false,
        Change::Details(value) => next.phone_details = value,
        Change::Answers(value) => next.phone_answers = value,
        Change::Diffs(value) => next.phone_diffs = value,
        Change::Usage(value) => next.phone_usage = value,
        Change::Services(value) => next.phone_services = value,
        Change::Chat(value) => next.phone_chat = value,
        Change::ChatModels(value) => next.phone_chat_models = value,
    }, persist, publish)
}

#[cfg(test)]
#[path = "preferences_tests.rs"]
mod tests;
