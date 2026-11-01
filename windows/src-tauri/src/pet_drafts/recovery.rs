//! Human recovery of opaque data. Never restores a session or authorizes use.
use super::{archives::{self,Archive},model::{Origin, Record, Registry, Retained, Target, Write}};
use crate::private_registry::{load_at, Snapshot};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Owner { pub pet_id: Vec<u16>, pub provider: Vec<u16> }
impl Owner {
    fn validate(&self) -> Result<(), &'static str> {
        if [&self.pet_id, &self.provider].iter().any(|v| v.is_empty() || v.len() > 128 || v.contains(&0)) { return Err("invalidDraft"); }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Availability { Existing, Orphan, Unbound, ExternalReference, Unknown }
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry { pub target: Target, pub revision: String, pub text_units: usize, pub availability: Availability }
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Page { pub catalogue_revision: String, pub entries: Vec<Entry>, pub total: usize, pub next_offset: Option<usize> }

pub(super) fn availability(chat: &crate::chat::Chat, target: &Target) -> Availability {
    let (Ok(pet), Ok(provider)) = (String::from_utf16(&target.pet_id), String::from_utf16(&target.provider)) else { return Availability::Unknown; };
    if crate::pet_config::integration_pet(&provider) != Some(pet.as_str()) { return Availability::Unknown; }
    match target.origin {
        Origin::Unbound => Availability::Unbound,
        Origin::External => Availability::ExternalReference,
        Origin::Mochi => {
            if target.conversation_id.as_ref().is_some_and(|cid| cid != &target.session_id) { return Availability::Unknown; }
            let Ok(cid) = String::from_utf16(&target.session_id) else { return Availability::Unknown; };
            match chat.contains_public_conversation(&provider, &cid) { Ok(true) => Availability::Existing, Ok(false) => Availability::Orphan, Err(_) => Availability::Unknown }
        }
    }
}
pub(super) fn list(path: &Path, owner: Owner, offset: usize, limit: usize, expected: Option<&str>, current: impl Fn() -> Result<(), &'static str>, observe: impl Fn(&Target) -> Availability) -> Result<Page, &'static str> {
    list_source(path,owner,offset,limit,expected,None,current,observe)
}
pub(super) fn list_source(path: &Path, owner: Owner, offset: usize, limit: usize, expected: Option<&str>, source:Option<&Retained>, current: impl Fn() -> Result<(), &'static str>, observe: impl Fn(&Target) -> Availability) -> Result<Page, &'static str> {
    owner.validate()?;
    if limit == 0 || limit > 16 || offset > super::model::RECORDS { return Err("invalidDraft"); }
    current()?;
    let before = load_at::<Registry>(path)?;
    if expected.is_some_and(|v| v != before.revision) { return Err("registryConflict"); }
    let archive=source.map(|r|archives::read(path,r)).transpose()?;
    let records=if let Some(archive)=&archive{&archive.registry.source.as_ref().ok_or("registryUnavailable")?.records}else{&before.registry.records};
    let mut rows: Vec<_> = records.iter().filter(|r| r.target.pet_id == owner.pet_id && r.target.provider == owner.provider).collect();
    rows.sort_by(|a,b| a.target.cmp(&b.target));
    let total = rows.len();
    let entries = rows.into_iter().skip(offset).take(limit).map(|r| Entry { target: r.target.clone(), revision: r.revision.clone(), text_units: r.text_units.len(), availability: observe(&r.target) }).collect();
    current()?;
    if !before.unchanged()||archive.is_some_and(|s|!s.unchanged()) { return Err("registryChanged"); }
    Ok(Page { catalogue_revision: before.revision, entries, total, next_offset: (offset.saturating_add(limit) < total).then_some(offset + limit) })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request { pub catalogue_revision: String, pub source: Target, pub destination: Target,#[serde(default,skip_serializing_if="Option::is_none")]pub source_archive:Option<Retained> }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Review { pub schema_version: u32, pub request: Request, pub expires_at: u64, pub source_record: Record, pub destination_record: Option<Record>, pub fingerprint: String }

fn plan(path:&Path,before: &Snapshot<Registry>, request: Request, expires_at: u64, now: u64) -> Result<(Review,Option<Snapshot<Archive>>), &'static str> {
    request.source.validate()?; request.destination.validate()?;
    if request.catalogue_revision != before.revision { return Err("registryConflict"); }
    if expires_at <= now || expires_at > now.checked_add(300).ok_or("invalidDraft")? { return Err("draftReviewExpired"); }
    let dest = &request.destination;
    if request.source == *dest || request.source.pet_id != dest.pet_id || request.source.provider != dest.provider || dest.origin != Origin::Mochi || dest.conversation_id.as_ref().is_some_and(|cid| cid != &dest.session_id) { return Err("invalidDraft"); }
    let pet = String::from_utf16(&dest.pet_id).map_err(|_| "invalidDraft")?;
    let provider = String::from_utf16(&dest.provider).map_err(|_| "invalidDraft")?;
    if crate::pet_config::integration_pet(&provider) != Some(pet.as_str()) { return Err("invalidDraft"); }
    let archive=request.source_archive.as_ref().map(|r|archives::read(path,r)).transpose()?;
    let records=if let Some(archive)=&archive{&archive.registry.source.as_ref().ok_or("registryUnavailable")?.records}else{&before.registry.records};
    let source_record = records.iter().find(|r| r.target == request.source).cloned().ok_or("invalidDraft")?;
    if source_record.text_units.is_empty() { return Err("invalidDraft"); }
    let destination_record = before.registry.records.iter().find(|r| r.target == *dest).cloned();
    if destination_record.as_ref().is_some_and(|r| !r.text_units.is_empty() && r.text_units != source_record.text_units) { return Err("draftDestinationOccupied"); }
    let bytes = serde_json::to_vec(&(1u32, &request, expires_at, &source_record, &destination_record)).map_err(|_| "invalidDraft")?;
    let fingerprint = ring::digest::digest(&ring::digest::SHA256, &bytes).as_ref().iter().map(|b| format!("{b:02x}")).collect();
    Ok((Review { schema_version: 1, request, expires_at, source_record, destination_record, fingerprint },archive))
}
pub(super) fn review(path: &Path, request: Request, now: u64, current: impl Fn() -> Result<(), &'static str>, destination: impl Fn(&Target) -> Result<(), &'static str>) -> Result<Review, &'static str> {
    current()?; let before = load_at::<Registry>(path)?;
    let (review,archive) = plan(path,&before, request, now.checked_add(300).ok_or("invalidDraft")?, now)?;
    destination(&review.request.destination)?; current()?;
    if !before.unchanged()||archive.is_some_and(|s|!s.unchanged()) { return Err("registryChanged"); }
    Ok(review)
}
pub(super) fn copy(path: &Path, reviewed: Review, approved: bool, now: impl Fn() -> u64, current: impl Fn() -> Result<(), &'static str>, retain: impl FnOnce(&Target) -> Result<(), &'static str>) -> Result<Record, &'static str> {
    if !approved { return Err("blocked"); }
    current()?; let before = load_at::<Registry>(path)?;
    let (exact,archive) = plan(path,&before, reviewed.request.clone(), reviewed.expires_at, now())?;
    if reviewed != exact { return Err("draftReviewChanged"); }
    let (next, result) = before.registry.write(exact.request.destination.clone(), exact.destination_record.as_ref().map(|r| r.revision.as_str()), exact.source_record.text_units)?;
    let Write::Saved { record } = result else { return Err("registryConflict"); };
    crate::private_registry::save_guarded(path, &before.revision, next, |_,_| { current()?; if now() >= exact.expires_at { return Err("draftReviewExpired"); } retain(&exact.request.destination)?; current() }, || { current()?;if archive.is_some_and(|s|!s.unchanged()){return Err("registryChanged");} if now() >= exact.expires_at { Err("draftReviewExpired") } else { Ok(()) } })?;
    Ok(record)
}
