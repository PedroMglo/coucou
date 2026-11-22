//! Field-only, acknowledged changes to the native pet control surfaces.
//! The command wrapper supplies native window identity and performs off-UI IO.

use crate::settings::Settings;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::io;
use std::sync::Mutex;

/// Missing fields retain their current value. JSON null is not a missing field.
/// The visitor rejects duplicate keys if the input deserializer preserves them.
/// Tauri's JSON Value boundary may already have normalized raw duplicate keys.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Patch {
    pub(crate) pet_control_bar: Option<bool>,
    pub(crate) pet_system_center: Option<bool>,
}

impl Patch {
    fn is_empty(&self) -> bool {
        self.pet_control_bar.is_none() && self.pet_system_center.is_none()
    }
}

impl<'de> Deserialize<'de> for Patch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchVisitor;

        impl<'de> Visitor<'de> for PatchVisitor {
            type Value = Patch;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object containing at least one pet center boolean")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Patch, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut patch = Patch::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "petControlBar" => {
                            if patch.pet_control_bar.is_some() {
                                return Err(de::Error::duplicate_field("petControlBar"));
                            }
                            patch.pet_control_bar = Some(map.next_value::<bool>()?);
                        }
                        "petSystemCenter" => {
                            if patch.pet_system_center.is_some() {
                                return Err(de::Error::duplicate_field("petSystemCenter"));
                            }
                            patch.pet_system_center = Some(map.next_value::<bool>()?);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(
                                &key,
                                &["petControlBar", "petSystemCenter"],
                            ));
                        }
                    }
                }
                if patch.is_empty() {
                    return Err(de::Error::custom("pet center patch must not be empty"));
                }
                Ok(patch)
            }
        }

        // deserialize_struct may accept positional sequences; only maps are valid.
        deserializer.deserialize_map(PatchVisitor)
    }
}

/// Canonical committed state, never a copy of the incoming patch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Flags {
    pub(crate) pet_control_bar: bool,
    pub(crate) pet_system_center: bool,
}

impl From<&Settings> for Flags {
    fn from(settings: &Settings) -> Self {
        Self {
            pet_control_bar: settings.pet_control_bar,
            pet_system_center: settings.pet_system_center,
        }
    }
}

/// Uses the shared settings transaction: snapshot, persist, commit and publish
/// remain ordered under its mutex. Persistence failure leaves memory unchanged
/// and does not publish. The callbacks are the only IO performed here.
pub(crate) fn apply_with(
    current: &Mutex<Settings>,
    patch: Patch,
    persist: impl FnOnce(&Settings) -> io::Result<()>,
    publish: impl FnOnce(&Settings),
) -> io::Result<Flags> {
    // Callers may construct Patch directly, bypassing Deserialize.
    if patch.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pet center patch must not be empty",
        ));
    }
    let committed = crate::settings::update_with(
        current,
        |next| {
            if let Some(value) = patch.pet_control_bar {
                next.pet_control_bar = value;
            }
            if let Some(value) = patch.pet_system_center {
                next.pet_system_center = value;
            }
        },
        persist,
        publish,
    )?;
    Ok(Flags::from(&committed))
}

/// Call inside the legacy settings update transaction, using the current
/// authoritative settings. All other incoming preferences remain untouched.
pub(crate) fn preserve_incoming(current: &Settings, incoming: &mut Settings) {
    incoming.pet_control_bar = current.pet_control_bar;
    incoming.pet_system_center = current.pet_system_center;
}

/// Both values are native inputs: expected is the settings window constant,
/// never a label taken from an IPC payload. Matching is exact.
pub(crate) fn authorize_window(actual: &str, expected: &str) -> io::Result<()> {
    if actual.is_empty() || expected.is_empty() || actual != expected {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "pet center settings require the settings window",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "pet_center_settings_tests.rs"]
mod tests;
