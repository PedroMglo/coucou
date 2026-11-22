//! Read-only projection of the declared DMS API34 Bluetooth state.
//! Native paths, addresses, pairing prompts and command handles stay private.
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

const MAX_ADAPTERS: usize = 16;
const MAX_DEVICES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Invalid { Unsupported, Shape, Capacity }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    schema_version: u8,
    observed_at: u64,
    navigation_revision: crate::pet_native_binding::ipc::Revision,
    available: bool,
    adapters: Vec<Adapter>,
    devices: Vec<Device>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Adapter {
    display_id: String,
    label: Option<String>,
    powered: bool,
    discovering: bool,
    device_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Device {
    display_id: String,
    adapter_display_id: String,
    label: Option<String>,
    paired: bool,
    trusted: bool,
    blocked: bool,
    connected: bool,
    category: Category,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum Category { Headset, Headphones, Keyboard, Mouse, Phone, Computer, Other }

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, Invalid> {
    value.get(field).and_then(Value::as_str).filter(|s| s.len() <= 1024).ok_or(Invalid::Shape)
}

fn boolean(value: &Value, field: &str) -> Result<bool, Invalid> {
    value.get(field).and_then(Value::as_bool).ok_or(Invalid::Shape)
}

fn rows<'a>(value: &'a Value, field: &str, max: usize) -> Result<&'a [Value], Invalid> {
    match value.get(field) {
        // Go's snapshot copies empty device slices onto nil; null is a known
        // empty list, whereas an absent field cannot prove an empty inventory.
        Some(Value::Null) => Ok(&[]),
        Some(Value::Array(rows)) if rows.len() <= max => Ok(rows),
        Some(Value::Array(_)) => Err(Invalid::Capacity),
        _ => Err(Invalid::Shape),
    }
}

fn adapter_path(path: &str) -> bool {
    path.strip_prefix("/org/bluez/hci").is_some_and(|n|
        !n.is_empty() && n.len() <= 10 && n.bytes().all(|b| b.is_ascii_digit())
        && (n.len() == 1 || !n.starts_with('0')))
}

fn device_adapter(path: &str) -> Option<&str> {
    let (adapter, tail) = path.rsplit_once("/dev_")?;
    if !adapter_path(adapter) || tail.len() != 17 { return None; }
    let valid = tail.bytes().enumerate().all(|(i, b)| {
        if i % 3 == 2 { b == b'_' } else { b.is_ascii_digit() || (b'A'..=b'F').contains(&b) }
    });
    valid.then_some(adapter)
}

fn address_label(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 17 && [b':', b'-', b'_'].iter().any(|separator|
        bytes.iter().enumerate().all(|(i, b)| if i % 3 == 2 { b == separator } else { b.is_ascii_hexdigit() }))
}

fn public_label(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty() && s.chars().count() <= 128 && !address_label(s)
        && !s.starts_with("/org/bluez/")
        && !s.chars().any(|c| c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')))
        .then(|| s.to_owned())
}

fn category(s: &str) -> Category {
    match s {
        "audio-headset" => Category::Headset,
        "audio-headphones" => Category::Headphones,
        "input-keyboard" => Category::Keyboard,
        "input-mouse" => Category::Mouse,
        "phone" => Category::Phone,
        "computer" => Category::Computer,
        _ => Category::Other,
    }
}

pub(crate) fn project(value: &Value, api: Option<u64>, capabilities: &[String], now: u64,
    revision: crate::pet_native_binding::ipc::Revision) -> Result<Snapshot, Invalid> {
    if api != Some(34) || !capabilities.iter().any(|c| c == "bluetooth") { return Err(Invalid::Unsupported); }
    if !value.is_object() || now == 0 || now > 9_007_199_254_740_991 { return Err(Invalid::Shape); }
    // These aggregate fields describe the upstream primary adapter, not all
    // adapters. Validate their shape but expose per-adapter facts instead.
    boolean(value, "powered")?;
    boolean(value, "discovering")?;
    let raw_adapters = rows(value, "adapters", MAX_ADAPTERS)?;
    let raw_devices = rows(value, "devices", MAX_DEVICES)?;
    let mut adapter_rows = BTreeMap::new();
    for raw in raw_adapters {
        let path = text(raw, "path")?;
        if !adapter_path(path) || adapter_rows.insert(path, raw).is_some() { return Err(Invalid::Shape); }
    }
    let mut adapters = Vec::new();
    let mut ids = BTreeMap::new();
    for (index, (path, raw)) in adapter_rows.into_iter().enumerate() {
        let id = format!("adapter-{}", index + 1);
        ids.insert(path, index);
        adapters.push(Adapter { display_id: id, label: public_label(text(raw, "name")?),
            powered: boolean(raw, "powered")?, discovering: boolean(raw, "discovering")?, device_count: 0 });
    }
    let mut device_rows = BTreeMap::new();
    for raw in raw_devices {
        let path = text(raw, "path")?;
        if device_adapter(path).is_none() || device_rows.insert(path, raw).is_some() { return Err(Invalid::Shape); }
    }
    let mut devices = Vec::new();
    for (index, (path, raw)) in device_rows.into_iter().enumerate() {
        let adapter = *ids.get(device_adapter(path).ok_or(Invalid::Shape)?).ok_or(Invalid::Shape)?;
        let row = &mut adapters[adapter];
        row.device_count += 1;
        devices.push(Device { display_id: format!("device-{}", index + 1), adapter_display_id: row.display_id.clone(),
            label: public_label(text(raw, "alias")?).or_else(|| public_label(text(raw, "name").unwrap_or(""))),
            paired: boolean(raw, "paired")?, trusted: boolean(raw, "trusted")?, blocked: boolean(raw, "blocked")?,
            connected: boolean(raw, "connected")?, category: category(text(raw, "icon")?) });
    }
    // Validate the duplicated upstream inventories without exporting them or
    // inventing totals from incomplete lists. Match by native path only here.
    for (field, flag) in [("pairedDevices", "paired"), ("connectedDevices", "connected")] {
        let subset = rows(value, field, MAX_DEVICES)?;
        let mut seen = HashSet::new();
        for raw in subset {
            let path = text(raw, "path")?;
            if !seen.insert(path) { return Err(Invalid::Shape); }
            let original = raw_devices.iter().find(|d| d.get("path").and_then(Value::as_str) == Some(path)).ok_or(Invalid::Shape)?;
            if !boolean(original, flag)? { return Err(Invalid::Shape); }
        }
        if seen.len() != raw_devices.iter().filter(|d| d.get(flag).and_then(Value::as_bool) == Some(true)).count() {
            return Err(Invalid::Shape);
        }
    }
    Ok(Snapshot { schema_version: 1, observed_at: now, navigation_revision: revision,
        available: !adapters.is_empty(), adapters, devices })
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
