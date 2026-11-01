//! Opaque application history is data only. It is never a native session,
//! model context, execution/permission grant, or an automatically adopted pet.
use super::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(crate) struct Retained {
    format: u32,
    #[serde(default)] pets: BTreeMap<String, Vec<Value>>,
    #[serde(default)] archives: BTreeMap<String, Vec<Value>>,
    #[serde(default)] fields: BTreeMap<String, Vec<Value>>,
}
impl Default for Retained {
    fn default() -> Self { Self { format: 1, pets: Default::default(), archives: Default::default(), fields: Default::default() } }
}
impl Retained {
    pub(crate) fn is_empty(&self) -> bool { self.pets.is_empty() && self.archives.is_empty() && self.fields.is_empty() }
    fn validate(&self) -> std::io::Result<()> {
        if self.format != 1 { return Err(std::io::Error::other("unknown inactive history format")); }
        if self.pets.values().chain(self.archives.values()).chain(self.fields.values()).any(Vec::is_empty) {
            return Err(std::io::Error::other("empty inactive history record"));
        }
        Ok(())
    }
    fn add(target: &mut BTreeMap<String, Vec<Value>>, key: String, value: Value) {
        let variants = target.entry(key).or_default();
        if !variants.contains(&value) { variants.push(value); }
    }
}

#[derive(Deserialize)]
struct RawDisk {
    version: u32,
    pets: HashMap<String, Value>,
    #[serde(default)] archives: HashMap<String, Value>,
    #[serde(default, rename="unavailableHistory")] unavailable_history: Option<Retained>,
    #[serde(flatten)] fields: Map<String, Value>,
}

fn conversation(value: Value, version: u32) -> std::io::Result<SavedConversation> {
    let saved: SavedConversation = serde_json::from_value(value)?;
    if version<7&&saved.native_memory_context.is_some(){return Err(std::io::Error::other("delivered memory provenance requires version7"));}
    if version < 5 && !saved.attachment_history.is_empty() {
        return Err(std::io::Error::other("attachment history requires version5"));
    }
    validate_captured_messages(&saved)?;
    Ok(saved)
}

pub(super) fn decode(bytes: &[u8], goal: Option<(&str, &str)>) -> std::io::Result<Library> {
    let raw: RawDisk = serde_json::from_slice(bytes)?;
    if !matches!(raw.version, 1..=7) { return Err(std::io::Error::other("unknown memory version")); }
    if raw.version < 6 && raw.unavailable_history.is_some() {
        return Err(std::io::Error::other("inactive history requires version6"));
    }
    if let Some((provider, cid)) = goal {
        if !supported(provider) || !valid_id(cid) || raw.pets.len() != 1 || !raw.archives.is_empty() || raw.unavailable_history.is_some()
            || !raw.fields.is_empty() || !raw.pets.get(provider).is_some_and(|c|c["id"] == cid) {
            return Err(std::io::Error::other("goal journal owner mismatch"));
        }
    }
    let mut retained = raw.unavailable_history.unwrap_or_default(); retained.validate()?;
    for (key, value) in raw.fields { Retained::add(&mut retained.fields, key, value); }
    let mut pets = HashMap::new();
    for (provider, value) in raw.pets {
        if !supported(&provider) { Retained::add(&mut retained.pets, provider, value); continue; }
        let mut c = public(conversation(value, raw.version)?);
        c.tasks.retain(|task|task.provider == provider && task.valid());
        pets.insert(provider, c);
    }
    let mut archives = HashMap::new();
    for (provider, value) in raw.archives {
        if !supported(&provider) { Retained::add(&mut retained.archives, provider, value); continue; }
        let values: Vec<Value> = serde_json::from_value(value)?;
        if values.len() > MAX_ARCHIVES { return Err(std::io::Error::other("too many archived conversations")); }
        let mut seen = std::collections::HashSet::new();
        if let Some(current) = pets.get(&provider) { seen.insert(current.id.clone()); }
        let mut rows = vec![];
        for value in values {
            let mut c = public(conversation(value, raw.version)?); c.tasks.clear();
            if !seen.insert(c.id.clone()) { return Err(std::io::Error::other("duplicate conversation id")); }
            rows.push(c);
        }
        archives.insert(provider, rows);
    }
    let library = Library { pets, archives, retained }; validate_request_history(&library)?; Ok(library)
}

pub(super) fn normalize(library: &Library) -> std::io::Result<Library> {
    let mut retained = library.retained.clone(); retained.validate()?;
    let mut pets = HashMap::new(); let mut archives = HashMap::new();
    for (provider, c) in &library.pets {
        if supported(provider) { pets.insert(provider.clone(), public(c.clone())); }
        else { Retained::add(&mut retained.pets, provider.clone(), serde_json::to_value(c)?); }
    }
    for (provider, rows) in &library.archives {
        if !supported(provider) { Retained::add(&mut retained.archives, provider.clone(), serde_json::to_value(rows)?); continue; }
        if rows.len() > MAX_ARCHIVES { return Err(std::io::Error::other("too many archived conversations")); }
        archives.insert(provider.clone(), rows.iter().map(|c| { let mut c=public(c.clone()); c.tasks.clear(); c }).collect());
    }
    Ok(Library { pets, archives, retained })
}

#[cfg(test)]
#[path="assistant_memory_retained_tests.rs"]
mod tests;
