//! Restart an inactive import under a fresh native claim. Never repair/reimport
//! an admitted profile, replace divergent data, or recover execution authority.
use super::*;

pub(super) const INTENT: &str = ".import.json";

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Intent {
    version: u32,
    pair: String,
    layout: String,
    content: String,
}

pub(super) fn intent(roots: &OwnedRoots, review: &LegacyReview) -> Result<()> {
    roots.current_for(&roots.layout)?; review.ensure_current()?;
    let expected = Intent {
        version: 1, pair: roots.pair.clone(), layout: layout_key(&roots.layout),
        content: staging::content_fingerprint(review)?,
    };
    if let Some(bytes) = read(&roots.config, INTENT)? {
        if parse::<Intent>(&bytes)? != expected { return Err(Error::SourceChanged); }
    } else {
        roots.config.publish_new(INTENT, &encode(&expected)?)?;
    }
    roots.current_for(&roots.layout)?; review.ensure_current()
}

fn empty(root: &Dir) -> Result<bool> { Ok(root.node()?.names(super::super::inventory::MAX_NODES)?.is_empty()) }

fn finish_empty_half(claim: &Claim, layout: &Layout) -> Result<()> {
    let config_missing = missing(layout.config())?;
    let data_missing = missing(layout.data())?;
    if !config_missing && !data_missing { return Ok(()); }
    if config_missing && data_missing { return Err(Error::UnsupportedProfile); }
    claim.for_layout(layout)?;
    let (existing, area, parent, target, other) = if config_missing {
        (layout.data(), Area::Data, &layout.legacy_config, layout.config(), Area::Config)
    } else {
        (layout.config(), Area::Config, &layout.legacy_data, layout.data(), Area::Data)
    };
    let present = Dir::root(existing)?; controls(&present, area)?;
    let bytes = read(&present, HEADER)?.ok_or(Error::UnsupportedProfile)?;
    let header: Header = parse(&bytes)?;
    if header.version != 1 || header.area != area || header.layout != layout_key(layout)
        || !valid_hex(&header.pair, 32) || read(&present, COMPLETE)?.is_some()
        || read(&present, ADMITTED)?.is_some() || read(&present, INTENT)?.is_some()
        || !empty(&present.dir(OsStr::new(PAYLOAD))?)? {
        return Err(Error::UnsupportedProfile);
    }
    claim.current()?; present.check()?;
    root(parent, target, other, &header.pair, &header.layout)?;
    claim.current()
}

pub(super) fn recover(claim: Claim, layout: Layout) -> Result<Active> {
    finish_empty_half(&claim, &layout)?;
    let roots = OwnedRoots::existing(&claim, layout.clone())?;
    if read(&roots.config, ADMITTED)?.is_some() {
        return Active::from_roots(claim, roots);
    }
    // A completion receipt means all bytes were present already. A missing or
    // changed file after that boundary is never silently reconstructed.
    if read(&roots.config, COMPLETE)?.is_some() || read(&roots.data, COMPLETE)?.is_some() {
        return recover_complete(claim, layout);
    }
    let config = roots.payload(Area::Config)?; let data = roots.payload(Area::Data)?;
    if read(&roots.config, INTENT)?.is_none() && !(empty(&config)? && empty(&data)?) {
        // Older complete copies can still finish, but partial copies without a
        // pre-copy content receipt cannot acquire a new history after restart.
        return recover_complete(claim, layout);
    }
    let review = LegacyReview::for_publication(layout.clone(), roots.clone())?;
    claim.current()?; intent(&roots, &review)?; claim.current()?;
    staging::resume_publication(&review, &config, &data, |_| {
        claim.for_layout(&layout)?; roots.current_for(&layout)
    })?;
    claim.current()?; review.ensure_current()?;
    recover_complete(claim, layout)
}

#[cfg(test)]
#[path = "publication_resume_tests.rs"]
mod tests;
