use super::{private_fs::{Node, Stamp}, Error, Layout, Result, Summary};
use std::{ffi::OsStr, io::Read, os::unix::ffi::OsStrExt, path::{Path, PathBuf}};

pub(super) const MAX_NODES: usize = 32768;
pub(super) const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(super) const MAX_FILE: u64 = crate::assistant_memory::MAX_BYTES;
pub(super) const MAX_DEPTH: usize = 32;
pub(super) const MAX_PATH: usize = 4096;
pub(super) const MAX_PATH_TOTAL: usize = 2 * 1024 * 1024;
const MAX_ROOT_NAMES: usize = 256;
pub(super) fn legacy_history_backup(name:&OsStr)->bool{
    let Some(text)=name.to_str()else{return false};let Some(rest)=text.strip_prefix("pre-mochi")else{return false};
    let parts:Vec<_>=rest.split('-').collect();
    parts.len()==3&&(1..=6).contains(&parts[0].len())&&parts[0].bytes().all(|b|b.is_ascii_digit())
        &&parts[1].len()==8&&parts[1].bytes().all(|b|b.is_ascii_digit())
        &&parts[2].len()==6&&parts[2].bytes().all(|b|b.is_ascii_digit())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub(super) enum Area { Config, Data }
#[derive(Clone, Copy)] enum Rule { Tree, Directory, Json, Pets, Sources, Memory, Assistant, Metrics, Inbox }
#[derive(Debug, PartialEq, Eq)] pub(super) struct Entry { pub area: Area, pub relative: PathBuf, pub stamp: Stamp, pub digest: Option<[u8; 32]> }
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Snapshot { pub summary: Summary, roots: [Option<Stamp>; 2], pub entries: Vec<Entry>, path_bytes: usize }
impl Snapshot {
    pub(super) fn fingerprint(&self) -> Result<String> {
        let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
        hash.update(b"coucou-legacy-review-v1\0");
        hash.update(&serde_json::to_vec(&self.roots).map_err(|_| Error::Unavailable)?);
        for entry in &self.entries {
            hash.update(&[match entry.area { Area::Config => 0, Area::Data => 1 }]);
            let path = entry.relative.as_os_str().as_bytes(); hash.update(&(path.len() as u64).to_le_bytes()); hash.update(path);
            hash.update(&serde_json::to_vec(&entry.stamp).map_err(|_| Error::Unavailable)?);
            hash.update(&[u8::from(entry.digest.is_some())]); if let Some(digest) = &entry.digest { hash.update(digest); }
        }
        Ok(hash.finish().as_ref().iter().map(|b| format!("{b:02x}")).collect())
    }
}

fn exists(path: &Path) -> Result<bool> {
    match Node::root(path) { Ok(Some(_)) => Ok(true), Ok(None) => Ok(false), Err(error) => Err(error) }
}
fn json(bytes: &[u8], rule: Rule) -> Result<()> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| Error::UnsupportedProfile)?;
    if !value.is_object() { return Err(Error::UnsupportedProfile); }
    let version = value.get("version").and_then(|v| v.as_u64());
    match rule {
        Rule::Pets | Rule::Sources if version != Some(1) => Err(Error::UnsupportedProfile),
        Rule::Memory => {
            if value.get("unavailableHistory").is_some(){return Err(Error::UnsupportedProfile);}
            if !matches!(version, Some(1..=4)) || !value["pets"].is_object()
                || value.get("archives").is_some_and(|v| !v.is_object()) { return Err(Error::UnsupportedProfile); }
            // Never downgrade a newer attachment schema hidden behind an old
            // version number. This is format admission, not dialogue validation.
            let pets = value["pets"].as_object().unwrap().values();
            let archives = value.get("archives").and_then(|v| v.as_object()).into_iter().flat_map(|a| a.values());
            if pets.clone().any(|c| !c.is_object()) || archives.clone().any(|a| !a.is_array() || a.as_array().unwrap().iter().any(|c| !c.is_object())) { return Err(Error::UnsupportedProfile); }
            if pets.chain(archives.flat_map(|a| a.as_array().into_iter().flatten()))
                .any(|c| c.get("attachmentHistory").is_some() || c.get("attachment_history").is_some()) { return Err(Error::UnsupportedProfile); }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn read(node: &Node, rule: Rule, used: &mut u64) -> Result<[u8; 32]> {
    if node.len > MAX_FILE || used.checked_add(node.len).is_none_or(|n| n > MAX_BYTES) { return Err(Error::Limit); }
    let capture = matches!(rule, Rule::Json | Rule::Pets | Rule::Sources | Rule::Memory);
    let mut bytes = vec![];
    let mut input = (&node.file).take(MAX_FILE + 1);
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let mut size = 0u64; let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = input.read(&mut buffer).map_err(|_| Error::Unavailable)?;
        if n == 0 { break; }
        size += n as u64;
        if size > MAX_FILE || used.checked_add(size).is_none_or(|n| n > MAX_BYTES) { return Err(Error::Limit); }
        hash.update(&buffer[..n]); if capture { bytes.extend_from_slice(&buffer[..n]); }
    }
    node.current()?;
    if size != node.len { return Err(Error::SourceChanged); }
    if capture { json(&bytes, rule)?; }
    *used += size; Ok(hash.finish().as_ref().try_into().unwrap())
}
fn children(rule: Rule, name: &OsStr) -> Result<Rule> {
    Ok(match rule {
        Rule::Inbox if name == ".coucou-purge-v1" => return Err(Error::UnsupportedProfile),
        Rule::Assistant => match name.to_str() {
            Some("conversations.json") => Rule::Memory,
            Some("metrics") => Rule::Metrics,
            _ if legacy_history_backup(name) => Rule::Directory,
            _ => return Err(Error::UnsupportedProfile),
        },
        Rule::Metrics if name == "ollama-usage-v1.json" => Rule::Json,
        Rule::Metrics => return Err(Error::UnsupportedProfile),
        _ => Rule::Tree,
    })
}
fn tree(snapshot: &mut Snapshot, parent: &Node, name: &OsStr, area: Area, relative: PathBuf, rule: Rule, depth: usize) -> Result<()> {
    if snapshot.entries.len() >= MAX_NODES || depth > MAX_DEPTH { return Err(Error::Limit); }
    let path_bytes = relative.as_os_str().as_bytes().len();
    if path_bytes > MAX_PATH || snapshot.path_bytes.checked_add(path_bytes).is_none_or(|n| n > MAX_PATH_TOTAL) { return Err(Error::Limit); }
    snapshot.path_bytes += path_bytes;
    let node = parent.child(name)?.ok_or(Error::SourceChanged)?;
    let directory = matches!(rule, Rule::Tree | Rule::Directory | Rule::Assistant | Rule::Metrics | Rule::Inbox);
    if !matches!(rule, Rule::Tree) && node.directory != directory { return Err(Error::UnsupportedProfile); }
    let entry = Entry { area, relative: relative.clone(), stamp: node.initial.clone(), digest: if node.directory { None } else { Some(read(&node, rule, &mut snapshot.summary.bytes)?) } };
    snapshot.entries.push(entry);
    if node.directory {
        snapshot.summary.directories += 1;
        for child in node.names(MAX_NODES - snapshot.entries.len())? {
            if matches!(rule,Rule::Metrics)&&child==".capture-owner-lock"{
                // This stable file is held by the existing usage writer. It
                // is not data or a transferable lease. Do not lock/unlink it.
                let lock=node.child(&child)?.ok_or(Error::SourceChanged)?;
                if lock.directory||lock.len!=0||!super::staging_fs::private_file(&lock.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
                lock.current()?;node.same_child(&child,&lock)?;continue;
            }
            tree(snapshot, &node, &child, area, relative.join(&child), children(rule, &child)?, depth + 1)?;
        }
    } else { snapshot.summary.files += 1; }
    node.current()?; parent.same_child(name, &node)
}
fn root(snapshot: &mut Snapshot, area: Area, path: &Path, index: usize) -> Result<()> {
    let Some(node) = Node::root(path)? else { return Ok(()); };
    let names = node.names(MAX_ROOT_NAMES)?;
    snapshot.roots[index] = Some(node.initial.clone());
    for name in names {
        let selected = match (area, name.to_str()) {
            (Area::Config, Some("settings.json")) => Some(Rule::Json),
            (Area::Config, Some("pets.json")) => Some(Rule::Pets),
            (Area::Config, Some("sources.json")) => Some(Rule::Sources),
            (Area::Config, Some("sounds")) => Some(Rule::Directory),
            (Area::Config, Some("pet-runtime-v1" | "pet-runtime-contexts-v1" | "pet-runtime-consent-v1")) => return Err(Error::UnsupportedProfile),
            (Area::Data, Some("assistant")) => Some(Rule::Assistant),
            (Area::Data, Some("inbox")) => Some(Rule::Inbox),
            (Area::Data, Some("Flow" | "Notes")) => Some(Rule::Directory),
            (Area::Data, Some("recap.json")) => Some(Rule::Json),
            _ => None,
        };
        if let Some(rule) = selected { tree(snapshot, &node, &name, area, PathBuf::from(&name), rule, 1)?; }
        else { snapshot.summary.excluded_roots += 1; }
    }
    node.current()?;
    if Node::root(path)?.is_none_or(|n| n.initial != node.initial) { return Err(Error::SourceChanged); }
    Ok(())
}
pub(super) fn inspect(layout: &Layout) -> Result<Snapshot> {
    inspect_with(layout,None)
}
pub(super) fn inspect_owned(layout:&Layout,roots:&super::publication::OwnedRoots)->Result<Snapshot>{
    inspect_with(layout,Some(roots))
}
fn targets(layout:&Layout,owned:Option<&super::publication::OwnedRoots>)->Result<()>{
    if let Some(roots)=owned{roots.current_for(layout)}else if exists(layout.config())?||exists(layout.data())?{Err(Error::TargetExists)}else{Ok(())}
}
fn inspect_with(layout:&Layout,owned:Option<&super::publication::OwnedRoots>)->Result<Snapshot>{
    targets(layout,owned)?;
    let mut snapshot = Snapshot { summary: Summary { files: 0, directories: 0, bytes: 0, excluded_roots: 0 }, roots: [None, None], entries: vec![], path_bytes: 0 };
    root(&mut snapshot, Area::Config, &layout.legacy_config, 0)?;
    root(&mut snapshot, Area::Data, &layout.legacy_data, 1)?;
    // Reopen both roots after all reads; a second root cannot hide replacement
    // of the first. This still does not replace exclusive migration ownership.
    for (path, before) in [&layout.legacy_config, &layout.legacy_data].into_iter().zip(&snapshot.roots) {
        if Node::root(path)?.map(|n| n.initial) != *before { return Err(Error::SourceChanged); }
    }
    targets(layout,owned)?;
    Ok(snapshot)
}
