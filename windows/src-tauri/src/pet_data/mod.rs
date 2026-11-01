//! Native data ownership, migration, backups and reviewed inactive restoration.
//! Restored history never constitutes executable admission or renewed consent.
#![allow(dead_code)] // Recovery helpers also expose isolated private QA paths.
mod inventory;
mod private_fs;
mod staging;
mod current_profile;
pub(crate)mod restored_history;
pub(crate)mod restored_projection;
pub(crate)mod restored_transaction;
pub(crate)mod backup_cli;
pub(crate)mod restore_cli;
mod staging_fs;
mod startup;
mod settings_io;
pub(crate)use settings_io::Format as SettingsIoFormat;
#[cfg(test)]pub(crate)use startup::fixture_native_claim;
mod publication;
pub(crate)mod bootstrap;
#[cfg(test)] mod tests;
#[cfg(test)] mod legacy_tests;

use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error { UnsafePath, Unavailable, Limit, UnsupportedProfile, TargetExists, SourceChanged, Busy, OwnershipLost }
type Result<T> = std::result::Result<T, Error>;

/// Layout version is independent of conversation JSON and application versions.
/// Keep shared Models/bin/log paths in legacy_data; mutable new data is a sibling
/// of the old inbox, never one of its children. Same XDG bases are supported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Layout {
    legacy_config: PathBuf,
    legacy_data: PathBuf,
    config: PathBuf,
    data: PathBuf,
}
fn absolute(path: &Path) -> bool {
    path.is_absolute() && path.file_name().is_some()
        && path.components().all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
}
fn overlaps(a: &Path, b: &Path) -> bool { a.starts_with(b) || b.starts_with(a) }
impl Layout {
    pub(crate) fn new(config: PathBuf, data: PathBuf) -> Result<Self> {
        if !absolute(&config) || !absolute(&data)
            || config.file_name().is_none_or(|n| n != "coucou")
            || data.file_name().is_none_or(|n| n != "coucou") { return Err(Error::UnsafePath); }
        let layout = Self { config: config.join("pets-config-v1"), data: data.join("pets-data-v1"), legacy_config: config, legacy_data: data };
        if overlaps(&layout.config, &layout.data)
            || overlaps(&layout.config, &layout.legacy_data.join("inbox"))
            || overlaps(&layout.data, &layout.legacy_data.join("inbox"))
            || layout.legacy_config.starts_with(&layout.data)
            || layout.legacy_data.starts_with(&layout.config) { return Err(Error::UnsafePath); }
        Ok(layout)
    }
    pub(crate) fn config(&self) -> &Path { &self.config }
    pub(crate) fn data(&self) -> &Path { &self.data }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub(crate) struct Summary { pub files: usize, pub directories: usize, pub bytes: u64, pub excluded_roots: usize }

/// Native, non-serializable review. A digest, path or old JSON cannot reconstruct
/// it or authorize activation. Revalidation is a prerequisite, not exclusivity:
/// the eventual migrator must also hold the application/startup ownership gate.
pub(crate) struct LegacyReview { layout: Layout, snapshot: inventory::Snapshot, outputs:Option<publication::OwnedRoots> }
impl LegacyReview {
    pub(crate) fn inspect(layout: Layout) -> Result<Self> {
        let snapshot = inventory::inspect(&layout)?;
        Ok(Self { layout, snapshot, outputs:None })
    }
    fn for_publication(layout:Layout,outputs:publication::OwnedRoots)->Result<Self>{
        let snapshot=inventory::inspect_owned(&layout,&outputs)?;
        Ok(Self{layout,snapshot,outputs:Some(outputs)})
    }
    pub(crate) fn summary(&self) -> Summary { self.snapshot.summary }
    pub(crate) fn ensure_current(&self) -> Result<()> {
        let current=match &self.outputs{Some(roots)=>inventory::inspect_owned(&self.layout,roots),None=>inventory::inspect(&self.layout)};
        match current {
            Ok(now) if now == self.snapshot => Ok(()),
            _ => Err(Error::SourceChanged),
        }
    }
}

mod revision_fence;
