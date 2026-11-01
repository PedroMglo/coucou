//! Component effects extend the existing preferences journal. These records
//! are neither pet grants nor permission to replay an interrupted operation.
use super::{hex, hash, error, Observation, ReconciliationRef};
use crate::private_registry::{self, Document, Snapshot};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io, path::{Path, PathBuf}};

pub(crate) const DIRECTORY: &str = "settings-component-history-v1";
#[path="settings_shortcut_publication.rs"]mod shortcut_publication;
#[path="settings_shortcut_reconciliation.rs"]mod shortcut_reconciliation;
pub(crate)use shortcut_publication::execution as shortcut_publication_execution;
#[path="settings_component_observations.rs"]mod historical;
#[path="settings_shortcut_scope.rs"]mod shortcut_scope;
pub(crate)use historical::{HistoricalObservation,HistoricalProjection,history_view,history_for_resource,legacy_slot,legacy_subject,blocks_resource,historical_subject,historical_witness,record_historical_observation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Phase { Prepared, Dispatched, Verified, Unconfirmed, NotDispatched, ReconciledObservation }
#[derive(Clone,Copy,PartialEq,Eq)]
enum Preparation { Ordinary, IndependentAutostart, CurrentShortcuts }

impl Phase {
    pub(crate) fn replaceable(self) -> bool {
        matches!(self, Self::Verified | Self::NotDispatched | Self::ReconciledObservation)
    }
    fn follows(self, prior: Self) -> bool {
        matches!((prior, self),
            (Self::Prepared, Self::Dispatched | Self::NotDispatched | Self::ReconciledObservation) |
            (Self::Dispatched, Self::Verified | Self::Unconfirmed | Self::NotDispatched | Self::ReconciledObservation) |
            (Self::Unconfirmed, Self::Verified | Self::ReconciledObservation) |
            (Self::Verified | Self::ReconciledObservation,Self::ReconciledObservation))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Binding {
    pub profile: String, pub configuration: String, pub resource: String, pub plan: String,
}
impl Binding {
    fn valid(&self) -> bool {
        [&self.profile, &self.configuration, &self.resource, &self.plan].into_iter().all(|h| hex(h, 64))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Slot {
    pub id: String, pub component: String, pub phase: Phase, pub binding: Binding,
    /// Evidence about the original worker, not a liveness test or a lease.
    pub owner: String, pub intent: ReconciliationRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<ReconciliationRef>,
}
impl Slot {
    /// A restored unknown effect remains unknown. Only a different admitted
    /// profile AND a different native resource can start independent work.
    pub(crate) fn independent_autostart_destination(&self, profile: &str, resource: &str) -> bool {
        self.component == "autostart" && !self.phase.replaceable()
            && self.binding.profile != profile && self.binding.resource != resource
    }
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if !hex(&self.id, 32) || !super::OPERATIONS.contains(&self.component.as_str()) || !self.binding.valid()
            || !hex(&self.owner, 64) || !reference_valid(&self.intent)
            || self.outcome.as_ref().is_some_and(|r| !reference_valid(r))
            || matches!(self.phase, Phase::Prepared | Phase::Dispatched) == self.outcome.is_some() {
            return Err("invalidComponentEffect");
        }
        Ok(())
    }
}
fn reference_valid(r: &ReconciliationRef) -> bool { hex(&r.id, 32) && hex(&r.sha256, 64) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Artifact {
    pub version: u32, pub generation: u64, pub id: String, pub effect: String,
    pub component: String, pub binding: Binding, pub phase: Phase,
    /// A plan/diff or an observed result. Never credentials, a shell command,
    /// a renderer-selected target, or an implicit authorization.
    pub evidence: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<Slot>,
}
impl Document for Artifact {
    const MAX_BYTES: usize = 128 * 1024;
    const LOCK_NAME: &'static str = ".component-archive-never-replace.lock";
    const TEMP_PREFIX: &'static str = ".component-archive-never-replace-";
    fn empty() -> Self {
        Self { version: 0, generation: 0, id: String::new(), effect: String::new(), component: String::new(),
            binding: Binding { profile: String::new(), configuration: String::new(), resource: String::new(), plan: String::new() },
            phase: Phase::Prepared, evidence: serde_json::Value::Null, previous: None }
    }
    fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1 || self.generation != 1 || !hex(&self.id, 32) || !hex(&self.effect, 32)
            || !super::OPERATIONS.contains(&self.component.as_str()) || !self.binding.valid()
            || self.phase == Phase::Dispatched || !self.evidence.is_object()
            || serde_json::to_vec(&self.evidence).map_or(true, |v| v.len() > 96 * 1024)
            || self.previous.as_ref().is_some_and(|s| s.validate().is_err() || s.component != self.component)
            || self.phase != Phase::Prepared && self.previous.is_some() {
            return Err("invalidComponentArtifact");
        }
        Ok(())
    }
    fn generation(&self) -> u64 { self.generation }
    fn next(self, _: &Self) -> Result<Self, &'static str> { Err("immutableComponentArtifact") }
}
fn path(settings: &Path, id: &str) -> io::Result<PathBuf> {
    if !hex(id, 32) { return Err(error("componentReference")); }
    Ok(settings.parent().ok_or_else(|| error("path"))?.join(DIRECTORY).join(format!("component-{id}.json")))
}
pub(crate) fn retain(settings: &Path, artifact: &Artifact, guard: impl Fn() -> Result<(), &'static str>) -> io::Result<ReconciliationRef> {
    artifact.validate().map_err(error)?;
    let bytes = serde_json::to_vec(artifact).map_err(|_| error("componentArtifact"))?;
    if bytes.len() > Artifact::MAX_BYTES { return Err(error("componentLimit")); }
    private_registry::retain_new(&path(settings, &artifact.id)?, &bytes, 64, 8 * 1024 * 1024, guard).map_err(error)?;
    let reference = ReconciliationRef { id: artifact.id.clone(), sha256: hash(&bytes) };
    observe(settings, &reference)?;
    Ok(reference)
}
fn observe(settings: &Path, reference: &ReconciliationRef) -> io::Result<Snapshot<Artifact>> {
    let held = private_registry::load_at::<Artifact>(&path(settings, &reference.id)?).map_err(error)?;
    if held.revision != reference.sha256 || held.registry.id != reference.id { return Err(error("componentArtifactChanged")); }
    Ok(held)
}
fn matches(artifact: &Artifact, slot: &Slot, phase: Phase) -> bool {
    artifact.effect == slot.id && artifact.component == slot.component && artifact.binding == slot.binding && artifact.phase == phase
}
/// Read only local, hash-bound artifacts. Archived native target paths are
/// evidence, never filesystem targets. A later operation cannot hide an old
/// unknown effect by replacing the current slot with a verified one.
pub(super) fn observations(settings: &Path, slots: &BTreeMap<String, Slot>) -> io::Result<Vec<Snapshot<Artifact>>> {
    let mut held = vec![];
    for slot in slots.values() {
        if slot.component=="shortcuts" {
            held.extend(historical::shortcut_sources(settings,Some(slot))?);
            continue;
        }
        let intent = observe(settings, &slot.intent)?;
        if !matches(&intent.registry, slot, Phase::Prepared) { return Err(error("componentIntent")); }
        held.push(intent);
        if let Some(reference) = &slot.outcome {
            let outcome = observe(settings, reference)?;
            if !matches(&outcome.registry, slot, slot.phase) { return Err(error("componentOutcome")); }
            held.extend(linked_sources(settings,&outcome.registry)?);
            held.push(outcome);
        }
    }
    Ok(held)
}

impl Observation {
    pub(in crate::settings) fn component(&self, name: &str) -> Option<&Slot> { self.snapshot.registry.components.get(name) }

    pub(in crate::settings) fn component_revalidated(&self, name: &str, configuration: &str) -> bool {
        self.snapshot.registry.version>=4&&self.component(name).is_some_and(|s| s.phase == Phase::Verified && s.binding.configuration == configuration
            && !self.snapshot.registry.suspended_operations.iter().any(|n| n == name)
            && self.format.as_ref().is_some_and(|f| f.current() && f.scope().is_ok_and(|scope| scope == s.binding.profile)))
    }

    /// Caller holds the existing preferences operation lock, checks native
    /// authority, and retains the exact plan before invoking this method.
    pub(in crate::settings) fn component_prepare(&self, settings: &Path, content: Option<&str>, slot: Slot,
        guard: impl Fn() -> Result<(), &'static str>) -> io::Result<Self> {
        self.component_prepare_reviewed(settings, content, slot, Preparation::Ordinary, guard)
    }
    fn component_prepare_reviewed(&self, settings: &Path, content: Option<&str>, slot: Slot, mode: Preparation,
        guard: impl Fn() -> Result<(), &'static str>) -> io::Result<Self> {
        if self.phase(settings, content) != crate::settings::IoPhase::Ready || slot.phase != Phase::Prepared { return Err(error("componentAdmission")); }
        slot.validate().map_err(error)?;
        if self.format.as_ref().is_some_and(|f| !f.scope().is_ok_and(|scope| scope == slot.binding.profile)) {
            return Err(error("componentProfileChanged"));
        }
        let previous = self.component(&slot.component);
        let independent=mode==Preparation::IndependentAutostart;
        let current_shortcuts=mode==Preparation::CurrentShortcuts;
        if !current_shortcuts && self.component_history_for(settings,&slot.component,Some(&slot.binding.resource))?.unresolved.iter().map(|old|blocks_resource(settings,old,&slot.binding.resource)).collect::<io::Result<Vec<_>>>()?.into_iter().any(|blocked|blocked) {
            return Err(error("componentArchivedReconciliationRequired"));
        }
        if independent {
            if slot.component != "autostart" || previous.is_none_or(|s| !s.independent_autostart_destination(&slot.binding.profile, &slot.binding.resource))||previous.map(|s|legacy_slot(settings,s)).transpose()?.unwrap_or(true) {
                return Err(error("componentIndependentDestinationRequired"));
            }
        } else if !current_shortcuts && previous.is_some_and(|s| !s.phase.replaceable()) { return Err(error("componentReconciliationRequired")); }
        let intent = observe(settings, &slot.intent)?;
        if !matches(&intent.registry, &slot, Phase::Prepared) || intent.registry.previous.as_ref() != previous { return Err(error("componentPlan")); }
        let prior=if slot.component=="shortcuts"{historical::shortcut_prior_artifacts(settings,previous)?}else{vec![]};
        let scope_review=shortcut_scope::validate(&intent.registry,&prior.iter().map(|(s,_)|s.clone()).collect::<Vec<_>>())?;
        if current_shortcuts!=scope_review || current_shortcuts&&(slot.component!="shortcuts"||previous.is_none()) { return Err(error("componentShortcutScopeReviewRequired")); }
        if independent && (intent.registry.evidence.get("externalUncertaintyAcknowledged") != Some(&serde_json::Value::Bool(true))
            || intent.registry.evidence.pointer("/review/action").and_then(serde_json::Value::as_str) != Some("setIndependentRegistration")) {
            return Err(error("componentIndependentReviewRequired"));
        }
        guard().map_err(error)?;
        if current_shortcuts{crate::pet_data::bootstrap::settings_io_shortcut_scope_protocol(settings)}else if slot.component=="shortcuts"{crate::pet_data::bootstrap::settings_io_shortcuts_protocol(settings)}else{crate::pet_data::bootstrap::settings_io_component_protocol(settings, independent)}.map_err(|_| error("profile"))?;
        // The marker upgrade invalidates only our previous format witness. The
        // source record and retained intent must still be the exact originals.
        let format = crate::pet_data::bootstrap::settings_io_observation(settings).map_err(|_| error("profile"))?;
        let profile = slot.binding.profile.clone();
        let mut record = self.snapshot.registry.clone();
        if self.snapshot.revision == "missing" {
            record.transaction = slot.id.clone(); record.phase = super::Phase::NotPublished;
            record.before = content.map(str::to_string); record.proposed = content.map(str::to_string).unwrap_or_else(|| hash(b""));
            record.review_required = false;
        }
        record.version = record.version.max(if current_shortcuts{8}else if slot.component=="shortcuts"{7}else{6});
        if !record.suspended_operations.contains(&slot.component) { record.suspended_operations.push(slot.component.clone()); }
        record.components.insert(slot.component.clone(), slot);
        let snapshot = private_registry::save_guarded(&self.path, &self.snapshot.revision, record, |_, _| Ok(()), || {
            guard()?;
            if self.snapshot.unchanged() && intent.unchanged() && self.history.as_ref().is_none_or(|h|h.unchanged())
                && self.components.iter().all(|h|h.unchanged()) && format.as_ref().is_none_or(|f| f.current() && f.scope().is_ok_and(|s|s==profile)) { Ok(()) } else { Err("componentChanged") }
        }).map_err(error)?;
        Self::from_snapshot(settings, snapshot)
    }

    pub(in crate::settings) fn component_transition(&self, settings: &Path, content: Option<&str>, component: &str, id: &str, phase: Phase,
        outcome: Option<ReconciliationRef>, guard: impl Fn() -> Result<(), &'static str>) -> io::Result<Self> {
        if !self.current(settings) { return Err(error("componentChanged")); }
        if matches!(phase, Phase::Dispatched | Phase::Verified) && self.phase(settings, content) != crate::settings::IoPhase::Ready {
            return Err(error("componentDataNotReady"));
        }
        let old = self.component(component).ok_or_else(|| error("componentMissing"))?;
        if old.id != id || !phase.follows(old.phase) { return Err(error("componentTransition")); }
        if matches!(phase, Phase::Dispatched | Phase::Verified)
            && self.format.as_ref().is_some_and(|f| !f.scope().is_ok_and(|scope| scope == old.binding.profile)) {
            return Err(error("componentProfileChanged"));
        }
        let mut record = self.snapshot.registry.clone();
        let next = record.components.get_mut(component).ok_or_else(|| error("componentMissing"))?;
        next.phase = phase; next.outcome = outcome; next.validate().map_err(error)?;
        if old.phase==Phase::Dispatched&&phase==Phase::NotDispatched{
            let artifact=next.outcome.as_ref().map(|r|observe(settings,r)).transpose()?.ok_or_else(||error("componentNoDispatchProof"))?;
            if component!="shortcuts"||artifact.registry.evidence.get("nativeEffectsStarted")!=Some(&serde_json::Value::Bool(false)){return Err(error("componentNoDispatchProof"));}
        }
        if component=="shortcuts"&&phase==Phase::ReconciledObservation{if record.version<7{return Err(error("componentShortcutObservationVersion"));}shortcut_reconciliation::transition(settings,old,next)?;}
        else if matches!(old.phase,Phase::Verified|Phase::ReconciledObservation)&&phase==Phase::ReconciledObservation{return Err(error("componentTransition"));}
        if old.phase==Phase::Unconfirmed&&phase==Phase::Verified{shortcut_publication::final_transition(settings,old,next)?;}
        let held = observations(settings, &record.components)?;
        // Only a live operation's verified outcome can remove its own barrier.
        // A recovered observation never renews the interrupted authorization.
        // A shortcut receipt is a fact about one native session, never a
        // persistent permission to register again (including in older readers).
        if phase == Phase::Verified && component != "shortcuts" { record.suspended_operations.retain(|n| n != component); }
        let snapshot = private_registry::save_guarded(&self.path, &self.snapshot.revision, record, |_, _| Ok(()), || {
            guard()?;
            if self.current(settings) && held.iter().all(|h| h.unchanged())
                && (!matches!(phase, Phase::Dispatched | Phase::Verified)
                    || self.format.as_ref().is_none_or(|f| f.scope().is_ok_and(|s|s==old.binding.profile))) { Ok(()) } else { Err("componentChanged") }
        }).map_err(error)?;
        Self::from_snapshot(settings, snapshot)
    }
}

pub(crate) fn autostart_configuration(enabled: bool) -> String {
    hash(if enabled { b"coucou-autostart-configuration-v1:true" } else { b"coucou-autostart-configuration-v1:false" })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Context{pub profile:Option<String>,pub settings_sha256:Option<String>,pub journal_revision:String,pub data_review:Option<ReconciliationRef>,pub slot:Option<Slot>}
pub(crate)fn context(settings:&crate::settings::Settings,path:&Path,component:&str)->io::Result<Context>{
 crate::settings::revision::component_read(settings,path,|io,witness|Ok(Context{
  profile:io.format.as_ref().map(|f|f.scope().map_err(|_|error("profile"))).transpose()?,
  settings_sha256:witness.content_hash().map(str::to_string),journal_revision:io.snapshot.revision.clone(),data_review:io.snapshot.registry.reconciliation.clone(),slot:io.component(component).cloned(),
 }))
}
pub(crate)fn current_artifacts(settings:&crate::settings::Settings,path:&Path,component:&str)->io::Result<Option<(Slot,Artifact,Option<Artifact>)>>{
 crate::settings::revision::component_read(settings,path,|io,_|io.component(component).map(|slot|Ok((slot.clone(),observe(path,&slot.intent)?.registry,slot.outcome.as_ref().map(|r|observe(path,r).map(|o|o.registry)).transpose()?))).transpose())
}
fn linked_sources(path:&Path,value:&Artifact)->io::Result<Vec<Snapshot<Artifact>>>{
 let mut held=vec![];let mut queue=vec![value.clone()];let mut seen=std::collections::BTreeSet::new();seen.insert(value.id.clone());
 while let Some(value)=queue.pop(){
  let mut sources=shortcut_publication::sources(path,&value)?;sources.extend(shortcut_reconciliation::sources(path,&value)?);
  for source in sources{if seen.len()>=128{return Err(error("componentReferenceLimit"));}if seen.insert(source.registry.id.clone()){queue.push(source.registry.clone());held.push(source);}}
 }Ok(held)
}
pub(crate)fn intent_artifact(settings:&crate::settings::Settings,path:&Path,component:&str)->io::Result<Option<Artifact>>{
 crate::settings::revision::component_read(settings,path,|io,_|io.component(component).map(|s|observe(path,&s.intent).map(|v|v.registry)).transpose())
}
pub(crate)fn unresolved_history(settings:&crate::settings::Settings,path:&Path,component:&str)->io::Result<Vec<Slot>>{
 Ok(history_view(settings,path,component)?.unresolved)
}
pub(crate)fn save_autostart(settings:&mut crate::settings::Settings,path:&Path,enabled:bool,guard:impl Fn()->io::Result<()>)->io::Result<()>{
 let mut next=settings.clone();next.autostart=enabled;
 match crate::settings::revision::save_guarded_to(path,&next,guard){Ok(())=>{*settings=next;Ok(())},Err(error)=>{crate::settings::revision::note_failed(settings,&next);Err(error)}}
}
/// Revoke only future registration. Preserve every component slot, including
/// an uncertain effect, and the existing data review/history. This is not a
/// fabricated native-effect intent or a grant.
pub(crate)fn suspend_shortcut_registration(settings:&crate::settings::Settings,path:&Path,guard:impl Fn()->Result<(),&'static str>)->io::Result<()>{
 crate::settings::revision::component_change(settings,path,|io,witness|{
  if io.phase(path,witness.content_hash())!=crate::settings::IoPhase::Ready{return Err(error("componentDataNotReady"));}
  guard().map_err(error)?;
  crate::pet_data::bootstrap::settings_io_component_protocol(path,false).map_err(|_|error("profile"))?;
  let format=crate::pet_data::bootstrap::settings_io_observation(path).map_err(|_|error("profile"))?;
  let mut record=io.snapshot.registry.clone();
  if io.snapshot.revision=="missing"{
   use ring::rand::SecureRandom;let mut nonce=[0;16];ring::rand::SystemRandom::new().fill(&mut nonce).map_err(|_|error("entropy"))?;
   record.transaction=nonce.iter().map(|n|format!("{n:02x}")).collect();record.phase=super::Phase::NotPublished;
   record.before=witness.content_hash().map(str::to_owned);record.proposed=witness.content_hash().map(str::to_owned).unwrap_or_else(||hash(b""));record.review_required=false;
  }
  record.version=record.version.max(6);if !record.suspended_operations.iter().any(|n|n=="shortcuts"){record.suspended_operations.push("shortcuts".into());}
  let snapshot=private_registry::save_guarded(&io.path,&io.snapshot.revision,record,|_,_|Ok(()),||{
   guard()?;if witness.current()&&io.snapshot.unchanged()&&io.history.as_ref().is_none_or(|h|h.unchanged())&&io.components.iter().all(|h|h.unchanged())&&format.as_ref().is_none_or(|f|f.current()){Ok(())}else{Err("componentChanged")}
  }).map_err(error)?;Observation::from_snapshot(path,snapshot)
 })
}
pub(crate)fn save_shortcuts(settings:&mut crate::settings::Settings,path:&Path,bindings:crate::shortcuts::Bindings,guard:impl Fn()->io::Result<()>)->io::Result<()>{
 let mut next=settings.clone();next.shortcuts=bindings;
 match crate::settings::revision::save_guarded_to(path,&next,guard){Ok(())=>{*settings=next;Ok(())},Err(error)=>{crate::settings::revision::note_failed(settings,&next);Err(error)}}
}
/// Exact dispatched-operation witnesses plus a weak guard of the actual native
/// Settings Revision. No Shared mutex is taken during OS IO. A reader leaves it
/// valid; a mutation attempt, failed IO state, replacement or disk change revokes.
pub(crate)struct Execution{path:PathBuf,io:Observation,witness:crate::private_registry::preferences_file::Witness,owner:crate::settings::revision::Guard,profile:String}
impl Execution{pub(crate)fn current(&self)->bool{
 self.owner.current()&&self.witness.current()&&self.io.phase(&self.path,self.witness.content_hash())==crate::settings::IoPhase::Ready
  &&self.io.format.as_ref().is_none_or(|f|f.scope().is_ok_and(|s|s==self.profile))&&self.owner.current()
}}
pub(crate)fn execution(settings:&crate::settings::Settings,path:&Path,id:&str,binding:&Binding,review:Option<&ReconciliationRef>)->io::Result<Execution>{
 execution_for_component(settings,path,"autostart",id,binding,review)
}
/// Same provenance witness for each component's own native controller. This
/// checks the exact dispatched slot; it neither admits a new effect nor grants
/// an operation. Native authority/resources remain the executor's responsibility.
pub(crate)fn execution_for_component(settings:&crate::settings::Settings,path:&Path,component:&str,id:&str,binding:&Binding,review:Option<&ReconciliationRef>)->io::Result<Execution>{
 crate::settings::revision::component_read(settings,path,|io,witness|{
  if io.snapshot.registry.version<4||io.component(component).is_none_or(|s|s.id!=id||s.phase!=Phase::Dispatched||&s.binding!=binding)||io.snapshot.registry.reconciliation.as_ref()!=review{return Err(error("componentExecutionChanged"));}
  Ok(Execution{path:path.into(),io:io.clone(),witness:witness.clone(),owner:crate::settings::revision::guard(settings),profile:binding.profile.clone()})
 })
}
pub(crate)fn read_execution(settings:&crate::settings::Settings,path:&Path,profile:&str)->io::Result<Execution>{
 crate::settings::revision::component_read(settings,path,|io,witness|{
  if io.format.as_ref().is_some_and(|f|!f.scope().is_ok_and(|s|s==profile)){return Err(error("componentProfileChanged"));}
  Ok(Execution{path:path.into(),io:io.clone(),witness:witness.clone(),owner:crate::settings::revision::guard(settings),profile:profile.into()})
 })
}

/// Narrow native APIs: only the real Settings owner is rebound. Rebinding a
/// cloned Settings value cannot advance the backend owner's IO observation.
pub(crate) fn prepare(settings: &crate::settings::Settings, path: &Path, slot: Slot,
    guard: impl Fn() -> Result<(), &'static str>) -> io::Result<()> {
    crate::settings::revision::component_change(settings, path, |io, witness| io.component_prepare(path, witness.content_hash(), slot, || {
        guard()?; if witness.current() { Ok(()) } else { Err("componentSettingsChanged") }
    }))
}
/// Only the native review controller calls this after its additional, exact
/// acknowledgement. It grants no authority over the previous resource.
pub(crate) fn prepare_independent_autostart(settings: &crate::settings::Settings, path: &Path, slot: Slot,
    guard: impl Fn() -> Result<(), &'static str>) -> io::Result<()> {
    crate::settings::revision::component_change(settings, path, |io, witness| io.component_prepare_reviewed(path, witness.content_hash(), slot, Preparation::IndependentAutostart, || {
        guard()?; if witness.current() { Ok(()) } else { Err("componentSettingsChanged") }
    }))
}
/// A new, reviewed attempt on the current owned registrar. This neither
/// reconciles the old scope nor claims physical independence from it.
pub(crate) fn prepare_current_shortcuts(settings:&crate::settings::Settings,path:&Path,slot:Slot,guard:impl Fn()->Result<(),&'static str>)->io::Result<()> {
    crate::settings::revision::component_change(settings,path,|io,witness|io.component_prepare_reviewed(path,witness.content_hash(),slot,Preparation::CurrentShortcuts,||{guard()?;if witness.current(){Ok(())}else{Err("componentSettingsChanged")}}))
}
pub(crate)fn shortcut_prior_artifacts(settings:&crate::settings::Settings,path:&Path)->io::Result<Vec<(Slot,Artifact)>> {
    crate::settings::revision::component_read(settings,path,|io,_|historical::shortcut_prior_artifacts(path,io.component("shortcuts")))
}
pub(crate) fn transition(settings: &crate::settings::Settings, path: &Path, component: &str, id: &str, phase: Phase,
    outcome: Option<ReconciliationRef>, guard: impl Fn() -> Result<(), &'static str>) -> io::Result<()> {
    crate::settings::revision::component_change(settings, path, |io, witness| {
        if component == "autostart" && matches!(phase, Phase::Dispatched | Phase::Verified)
            && io.component(component).is_none_or(|s|s.binding.configuration != autostart_configuration(settings.autostart)) {
            return Err(error("componentConfigurationChanged"));
        }
        io.component_transition(path, witness.content_hash(), component, id, phase, outcome, || {
            guard()?; if witness.current() { Ok(()) } else { Err("componentSettingsChanged") }
        })
    })
}

pub(super) fn validate(slots: &BTreeMap<String, Slot>) -> Result<(), &'static str> {
    if slots.len() > super::OPERATIONS.len() { return Err("componentLimit"); }
    for (name, slot) in slots { slot.validate()?; if name != &slot.component { return Err("invalidComponentEffect"); } }
    Ok(())
}

#[cfg(test)] #[path = "settings_component_journal_tests.rs"] mod tests;
