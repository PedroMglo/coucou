//! Ephemeral native IO provenance. It is skipped in JSON and is not a grant.
//! Clones keep their own expected revision; a browser object cannot renew it.
use super::Settings;
use crate::private_registry::preferences_file::{self, Witness};
use std::{io, path::{Path, PathBuf}, sync::{Mutex,Arc,Weak,atomic::{AtomicU64,Ordering}}};
use super::durable::{self,Observation,Phase};

#[derive(Clone,Debug)]struct Bound{witness:Witness,io:Observation}
impl Bound{
    fn observe(witness:Witness)->io::Result<Self>{let io=Observation::read(witness.selected())?;Ok(Self{witness,io})}
    fn phase(&self)->super::IoPhase{
        if !self.witness.current(){return super::IoPhase::Changed;}
        self.io.phase(self.witness.selected(),self.witness.content_hash())
    }
}

#[derive(Clone, Debug, Default)]
enum State { #[default] Unbound, Loaded(Bound), Protected(PathBuf), RecoveredUnreviewed(Bound), FailedBeforePublication(Bound), Unconfirmed }
#[derive(Debug, Default)]
pub(super) struct Revision(Mutex<State>,Arc<AtomicU64>);
impl Clone for Revision {
    fn clone(&self) -> Self { Self(Mutex::new(self.0.lock().map(|s| s.clone()).unwrap_or(State::Unconfirmed)),Arc::new(AtomicU64::new(0))) }
}
impl Revision{fn invalidate(&self){let _=self.1.fetch_update(Ordering::AcqRel,Ordering::Acquire,|v|Some(v.saturating_add(1)));}}
/// Ephemeral identity of the actual Revision, never a persisted grant. Cloned
/// Settings have a different Arc; replacing/revoking the owner invalidates it.
pub(super)struct Guard{owner:Weak<AtomicU64>,epoch:u64}
impl Guard{pub(super)fn current(&self)->bool{self.epoch!=u64::MAX&&self.owner.upgrade().is_some_and(|v|v.load(Ordering::Acquire)==self.epoch)}}
pub(super)fn guard(settings:&Settings)->Guard{Guard{owner:Arc::downgrade(&settings.file_revision.1),epoch:settings.file_revision.1.load(Ordering::Acquire)}}
fn blocked() -> io::Error { io::Error::other("A origem ou a publicação das preferências não está confirmada. Requer reconciliação antes de guardar ou executar esta alteração.") }
fn with(mut settings: Settings, state: State) -> Settings { settings.file_revision = Revision(Mutex::new(state),Arc::new(AtomicU64::new(0))); settings }
fn state_phase(state:&State)->super::IoPhase{
    use super::IoPhase;
    match state{
        State::Loaded(bound)|State::FailedBeforePublication(bound)=>bound.phase(),
        State::Protected(_)=>IoPhase::ReadUnavailable,
        State::RecoveredUnreviewed(bound)=>match bound.phase(){IoPhase::Ready=>IoPhase::ReviewRequired,p=>p},
        State::Unconfirmed=>IoPhase::PublicationUnconfirmed,State::Unbound=>IoPhase::Unbound,
    }
}
fn state_suspended(state:&State,autostart:bool)->Vec<String>{
    match state{State::Loaded(b)|State::FailedBeforePublication(b)|State::RecoveredUnreviewed(b)=>{
        let mut pending=b.io.suspended();
        if b.io.component("autostart").is_some()&&!b.io.component_revalidated("autostart",&durable::components::autostart_configuration(autostart))&&!pending.iter().any(|n|n=="autostart"){pending.push("autostart".into());}pending
    },_=>vec![]}
}
pub(super) fn phase(settings:&Settings)->super::IoPhase{
    match settings.file_revision.0.try_lock(){Ok(state)=>state_phase(&state),Err(std::sync::TryLockError::WouldBlock)=>super::IoPhase::Writing,Err(_)=>super::IoPhase::Unavailable}
}
pub(super) fn suspended(settings:&Settings)->Vec<String>{settings.file_revision.0.try_lock().map(|state|state_suspended(&state,settings.autostart)).unwrap_or_default()}
/// Immutable copies of the existing Bound/Observation + original owner epoch.
/// A captured validation has no mutex that another reader could mistake for a
/// writer; current filesystem/journal witnesses still detect real mutations.
pub(super) struct Captured{owner:Guard,state:State,autostart:bool}
pub(super) fn capture(settings:&Settings)->Captured{
    let owner=guard(settings);let state=settings.file_revision.0.lock().map(|s|s.clone()).unwrap_or(State::Unconfirmed);
    Captured{owner,state,autostart:settings.autostart}
}
impl Captured{
    pub(super) fn owner_current(&self)->bool{self.owner.current()}
    pub(super) fn phase(&self)->super::IoPhase{if self.owner_current(){state_phase(&self.state)}else{super::IoPhase::Unavailable}}
    pub(super) fn suspended(&self)->Vec<String>{state_suspended(&self.state,self.autostart)}
}

pub(super)fn component_change(settings:&Settings,path:&Path,change:impl FnOnce(&Observation,&Witness)->io::Result<Observation>)->io::Result<()>{
 let mut state=settings.file_revision.0.lock().map_err(|_|blocked())?;
 let bound=match &*state{State::Loaded(b)|State::FailedBeforePublication(b)if b.witness.selected()==path&&b.phase()==super::IoPhase::Ready=>b.clone(),_=>return Err(blocked())};
 let _writer=durable::writer(path)?;if bound.phase()!=super::IoPhase::Ready{return Err(blocked());}
 settings.file_revision.invalidate();
 match change(&bound.io,&bound.witness){
  Ok(io)=>{*state=State::Loaded(Bound{witness:bound.witness,io});Ok(())},
  Err(error)=>{if !bound.witness.current()||!bound.io.current(path){*state=State::Unconfirmed;}Err(error)},
 }
}
pub(super)fn component_read<T>(settings:&Settings,path:&Path,read:impl FnOnce(&Observation,&Witness)->io::Result<T>)->io::Result<T>{
 let state=settings.file_revision.0.try_lock().map_err(|_|blocked())?;
 let bound=match &*state{State::Loaded(b)|State::FailedBeforePublication(b)if b.witness.selected()==path&&b.phase()==super::IoPhase::Ready=>b,_=>return Err(blocked())};
 read(&bound.io,&bound.witness)
}
pub(super)fn component_revalidated(settings:&Settings,name:&str)->bool{
 if name!="autostart"{return false;}
 let Ok(state)=settings.file_revision.0.try_lock()else{return false;};
 match &*state{State::Loaded(b)|State::FailedBeforePublication(b)=>b.phase()==super::IoPhase::Ready&&b.io.component_revalidated(name,&durable::components::autostart_configuration(settings.autostart)),_=>false}
}
pub(super)fn accept_review(settings:&mut Settings,witness:Witness,io:Observation){settings.file_revision=Revision(Mutex::new(State::Loaded(Bound{witness,io})),Arc::new(AtomicU64::new(0)));}
pub(super)fn mark_unconfirmed(settings:&Settings){settings.file_revision.invalidate();if let Ok(mut state)=settings.file_revision.0.lock(){*state=State::Unconfirmed;}}
pub(super) fn note_failed(current:&Settings,candidate:&Settings){
    if let Ok(candidate)=candidate.file_revision.0.lock(){
        if matches!(&*candidate,State::Unconfirmed|State::FailedBeforePublication(_)){
            if let Ok(mut state)=current.file_revision.0.lock(){current.file_revision.invalidate();*state=candidate.clone();}
        }
    }
}

pub(super) fn load_from(path: &Path) -> Settings {
    let (witness, bytes) = match preferences_file::read(path, true) {
        Ok(value) => value,
        Err(error) => {
            super::note(format!("can't observe preferences: {error} — using protected defaults"));
            super::protect(path);
            return with(Settings::default(), State::Protected(path.into()));
        }
    };
    let Some(bytes) = bytes else { return match Bound::observe(witness){Ok(bound)=>with(Settings::default(),State::Loaded(bound)),Err(_)=>with(Settings::default(),State::Unconfirmed)}; };
    let fully_valid = super::fully_valid(&bytes);
    let settings = super::parse(&bytes).unwrap_or_else(|| {
        match super::set_aside(path, "corrupt", &bytes) {
            Ok(_) => super::note("settings.json is not usable — kept byte for byte; using defaults".into()),
            Err(_) => super::protect(path),
        }
        Settings::default()
    });
    if fully_valid { match Bound::observe(witness){Ok(bound)=>with(settings,State::Loaded(bound)),Err(_)=>with(settings,State::Unconfirmed)} }
    else { super::protect(path); with(settings, State::Protected(path.into())) }
}

/// Data projection only, after the profile itself was admitted. Even a fully
/// parseable but non-canonical file must never become execution authority here.
pub(super)fn load_for_review(path:&Path)->Settings{let settings=load_from(path);super::protect(path);with(settings,State::Protected(path.into()))}

pub(super) fn bind_strict(path: &Path, metadata: &std::fs::Metadata, bytes: &[u8], settings: &mut Settings) -> Result<(), String> {
    let (witness, _) = preferences_file::read(path, false).map_err(|_| "A revisão da configuração restaurada não foi confirmada.")?;
    if !witness.matches_metadata(metadata) || !witness.matches_bytes(bytes) { return Err("A configuração restaurada mudou durante a revisão.".into()); }
    settings.file_revision = Revision(Mutex::new(State::Loaded(Bound::observe(witness).map_err(|_|"O registo de IO restaurado não foi confirmado.")?)),Arc::new(AtomicU64::new(0))); Ok(())
}

pub(super) fn save_to(path: &Path, settings: &Settings) -> io::Result<()> {
    save_with(path,settings,|witness,bytes|witness.commit(bytes))
}
pub(super)fn save_guarded_to(path:&Path,settings:&Settings,guard:impl Fn()->io::Result<()>)->io::Result<()>{save_with(path,settings,|witness,bytes|witness.commit_guarded(bytes,guard))}
fn save_with(path:&Path,settings:&Settings,commit:impl FnOnce(&Witness,&[u8])->io::Result<Witness>)->io::Result<()>{
    let mut state = settings.file_revision.0.lock().map_err(|_| blocked())?;
    let bound = match &*state {
        State::Loaded(bound) | State::RecoveredUnreviewed(bound)|State::FailedBeforePublication(bound) if bound.witness.selected() == path => bound.clone(),
        State::Protected(original) if original == path => {
            // Keep the legacy unread-file preservation path, but first bound
            // and classify its content; never block on a FIFO/device here.
            Bound::observe(preferences_file::read(path, true)?.0)?
        }
        #[cfg(test)] State::Unbound => Bound::observe(preferences_file::read(path, true)?.0)?,
        _ => return Err(blocked()),
    };
    if !bound.witness.current() { return Err(io::Error::other("As preferências mudaram desde a leitura. A gravação não foi iniciada.")); }
    if !matches!(bound.phase(),super::IoPhase::Ready|super::IoPhase::ReviewRequired){return Err(blocked());}
    let _operation=durable::writer(path)?;
    settings.file_revision.invalidate();
    if !bound.witness.current() { return Err(io::Error::other("As preferências mudaram desde a leitura. A gravação não foi iniciada.")); }
    if !matches!(bound.phase(),super::IoPhase::Ready|super::IoPhase::ReviewRequired){return Err(blocked());}
    super::keep_what_was_not_loaded(path)?;
    let bytes = serde_json::to_vec_pretty(settings).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if bytes.len()>256*1024{return Err(io::Error::new(io::ErrorKind::InvalidInput,"As preferências excedem o limite de gravação."));}
    let review_required = matches!(&*state, State::Protected(_) | State::RecoveredUnreviewed(_))||bound.io.review_required();
    let prepared=match bound.io.prepare(path,bound.witness.content_hash(),&bytes,review_required){Ok(p)=>p,Err(error)=>{*state=State::Unconfirmed;return Err(error);}};
    match commit(&bound.witness,&bytes) {
        Ok(witness) => {
            match prepared.finish(Phase::Committed){
                Ok(io)=>{let next=Bound{witness,io};*state=if review_required{State::RecoveredUnreviewed(next)}else{State::Loaded(next)};Ok(())},
                Err(error)=>{*state=State::Unconfirmed;Err(error)},
            }
        }
        Err(error) => {
            if preferences_file::unconfirmed(&error) { *state = State::Unconfirmed; }
            else{match prepared.finish(Phase::NotPublished){Ok(io)=>*state=State::FailedBeforePublication(Bound{witness:bound.witness,io}),Err(_)=>*state=State::Unconfirmed}}
            Err(error)
        }
    }
}
#[cfg(test)]pub(super)fn uncertain_save_for_test(path:&Path,settings:&Settings,after:impl FnOnce())->io::Result<()>{
    save_with(path,settings,|witness,bytes|witness.commit_for_test(bytes,||{},after))
}
#[cfg(test)]pub(super)fn sync_failure_for_test(path:&Path,settings:&Settings)->io::Result<()>{save_with(path,settings,|w,b|w.commit_sync_for_test(b,|_|Err(io::Error::other("synthetic directory sync failure"))))}

#[cfg(test)]
#[path = "settings_revision_tests.rs"]
mod tests;
