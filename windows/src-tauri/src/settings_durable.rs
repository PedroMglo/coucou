//! Preferences IO outcomes, not grants. Exact hashes and optimistic native
//! snapshots; no secrets, provider calls or authority renewed by this record.
use crate::private_registry::{self,Document,Snapshot};
use serde::{Deserialize,Serialize};
use std::{io,path::{Path,PathBuf}};
#[path="settings_review_history.rs"]
pub(crate)mod history;
#[path="settings_component_journal.rs"]
pub(crate)mod components;
pub(crate)const OPERATIONS:[&str;6]=["voice","mobile","autostart","shortcuts","hooks","credentials"];
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct ReconciliationRef{pub id:String,pub sha256:String}
pub(crate)const MAX_RECORD_BYTES:usize=16*1024;
pub(crate)const FILE:&str="settings-io-v1.json";
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Phase{Prepared,Committed,NotPublished}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Record{
    version:u32,generation:u64,transaction:String,phase:Phase,review_required:bool,
    before:Option<String>,proposed:String,
    #[serde(default,skip_serializing_if="Vec::is_empty")]suspended_operations:Vec<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]reconciliation:Option<ReconciliationRef>,
    #[serde(default,skip_serializing_if="std::collections::BTreeMap::is_empty")]components:std::collections::BTreeMap<String,components::Slot>,
}
pub(super)fn hex(s:&str,n:usize)->bool{s.len()==n&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
pub(super)fn hash(b:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,b).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn error(s:&str)->io::Error{io::Error::other(format!("O registo de IO das preferências não foi confirmado ({s}); requer reconciliação antes de novos efeitos."))}
impl Document for Record{
    const MAX_BYTES:usize=MAX_RECORD_BYTES;const LOCK_NAME:&'static str=".settings-io-record.lock";const TEMP_PREFIX:&'static str=".settings-io-record-";
    fn empty()->Self{Self{version:1,generation:0,transaction:String::new(),phase:Phase::Prepared,review_required:true,before:None,proposed:String::new(),suspended_operations:vec![],reconciliation:None,components:Default::default()}}
    fn validate(&self)->Result<(),&'static str>{
        if !matches!(self.version,1|2|3|4|5|6|7|8)||self.generation==0||!hex(&self.transaction,32)||!hex(&self.proposed,64)||self.before.as_ref().is_some_and(|h|!hex(h,64)){return Err("invalidSettingsIo");}
        let unique:std::collections::BTreeSet<_>=self.suspended_operations.iter().collect();
        if self.suspended_operations.len()>OPERATIONS.len()||unique.len()!=self.suspended_operations.len()||self.suspended_operations.iter().any(|s|!OPERATIONS.contains(&s.as_str()))||match self.version{1=>!self.suspended_operations.is_empty()||self.reconciliation.is_some()||!self.components.is_empty(),2=>!self.components.is_empty()||self.reconciliation.as_ref().is_none_or(|r|!hex(&r.id,32)||!hex(&r.sha256,64)),3|4|5|6|7|8=>self.reconciliation.as_ref().is_some_and(|r|!hex(&r.id,32)||!hex(&r.sha256,64)),_=>true}{return Err("invalidSettingsIo");}components::validate(&self.components)?;Ok(())
    }
    fn generation(&self)->u64{self.generation}
    fn next(mut self,previous:&Self)->Result<Self,&'static str>{self.generation=previous.generation.checked_add(1).ok_or("settingsIoLimit")?;self.validate()?;Ok(self)}
}
#[derive(Clone)]pub(super)struct Observation{path:PathBuf,snapshot:Snapshot<Record>,format:Option<crate::pet_data::SettingsIoFormat>,history:Option<Snapshot<history::Artifact>>,components:Vec<Snapshot<components::Artifact>>}
impl std::fmt::Debug for Observation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("SettingsIoObservation").field("present",&(self.snapshot.revision!="missing")).finish_non_exhaustive()}
}
impl Observation{
    pub(super)fn read(settings:&Path)->io::Result<Self>{
        let path=settings.parent().ok_or_else(||error("path"))?.join(FILE);
        let snapshot=private_registry::load_at::<Record>(&path).map_err(error)?;
        let format=crate::pet_data::bootstrap::settings_io_observation(settings).map_err(|_|error("profile"))?;
        if snapshot.revision!="missing"&&!format.as_ref().is_some_and(|f|f.present())&&crate::pet_data::bootstrap::admitted(){return Err(error("format"));}
        let history=snapshot.registry.reconciliation.as_ref().map(|r|history::observe(settings,r)).transpose()?;
        let components=components::observations(settings,&snapshot.registry.components)?;
        if snapshot.registry.version<8&&components.iter().any(|a|a.registry.evidence.pointer("/review/schemaVersion")==Some(&serde_json::json!(3))){return Err(error("componentShortcutScopeVersion"));}
        Ok(Self{path,snapshot,format,history,components})
    }
    pub(super)fn current(&self,settings:&Path)->bool{
        settings.parent().is_some_and(|p|p.join(FILE)==self.path)&&self.snapshot.unchanged()&&self.format.as_ref().is_none_or(|f|f.current())&&self.history.as_ref().is_none_or(|h|h.unchanged())&&self.components.iter().all(|h|h.unchanged())
    }
    pub(super)fn phase(&self,settings:&Path,content:Option<&str>)->super::IoPhase{
        use super::IoPhase;
        if !self.current(settings)||self.format.is_none()&&crate::pet_data::bootstrap::admitted(){return IoPhase::PublicationUnconfirmed;}
        if self.snapshot.revision=="missing"{return if self.format.as_ref().is_some_and(|f|f.present()){IoPhase::PublicationUnconfirmed}else{IoPhase::Ready};}
        let r=&self.snapshot.registry;
        if self.format.as_ref().is_some_and(|f|f.version()!=Some(r.version)){return IoPhase::PublicationUnconfirmed;}
        if r.phase==Phase::Prepared{return IoPhase::PublicationUnconfirmed;}
        let expected=match r.phase{Phase::Committed=>Some(r.proposed.as_str()),Phase::NotPublished=>r.before.as_deref(),Phase::Prepared=>unreachable!()};
        if expected!=content{return IoPhase::PublicationUnconfirmed;}
        if r.review_required{IoPhase::ReviewRequired}else{IoPhase::Ready}
    }
    pub(super)fn suspended(&self)->Vec<String>{if self.snapshot.revision=="missing"{vec![]}else{self.snapshot.registry.suspended_operations.clone()}}
    pub(super)fn review_required(&self)->bool{self.snapshot.revision!="missing"&&self.snapshot.registry.review_required}
    pub(super)fn prepare(&self,settings:&Path,before:Option<&str>,proposed:&[u8],review_required:bool)->io::Result<Prepared>{
        if !self.current(settings){return Err(error("changed"));}
        // The immutable root extension is durable before the first mutable
        // intent; an old reader refuses it before loading Settings or Chat.
        crate::pet_data::bootstrap::settings_io_protocol(settings,true).map_err(|_|error("profile"))?;
        let mut random=[0;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut random).map_err(|_|error("random"))?;
        let record=Record{version:if self.snapshot.revision=="missing"{1}else{self.snapshot.registry.version},generation:0,transaction:random.iter().map(|v|format!("{v:02x}")).collect(),phase:Phase::Prepared,review_required:review_required||self.review_required(),before:before.map(str::to_string),proposed:hash(proposed),suspended_operations:self.snapshot.registry.suspended_operations.clone(),reconciliation:self.snapshot.registry.reconciliation.clone(),components:self.snapshot.registry.components.clone()};
        let snapshot=private_registry::save_checked(&self.path,&self.snapshot.revision,record,|_,_|Ok(())).map_err(error)?;
        Ok(Prepared{settings:settings.into(),observation:Self::from_snapshot(settings,snapshot)?})
    }
    fn from_snapshot(settings:&Path,snapshot:Snapshot<Record>)->io::Result<Self>{
        let format=crate::pet_data::bootstrap::settings_io_observation(settings).map_err(|_|error("profile"))?;
        let history=snapshot.registry.reconciliation.as_ref().map(|r|history::observe(settings,r)).transpose()?;
        let components=components::observations(settings,&snapshot.registry.components)?;
        if snapshot.registry.version<8&&components.iter().any(|a|a.registry.evidence.pointer("/review/schemaVersion")==Some(&serde_json::json!(3))){return Err(error("componentShortcutScopeVersion"));}
        Ok(Self{path:settings.parent().ok_or_else(||error("path"))?.join(FILE),snapshot,format,history,components})
    }
}
pub(super)fn review_prepare(settings:&Path,previous:&crate::private_registry::preferences_file::Witness,original:Option<&[u8]>,before:Option<&str>,proposed:&[u8],reference:ReconciliationRef,guard:impl FnOnce()->io::Result<()>)->io::Result<Prepared>{
 if previous.selected()!=settings.parent().ok_or_else(||error("path"))?.join(FILE)||!previous.current(){return Err(error("changed"));}
 history::observe(settings,&reference)?;
 let generation=original.and_then(|b|serde_json::from_slice::<Record>(b).ok()).filter(|r|r.validate().is_ok()).map_or(Ok(1),|r|r.generation.checked_add(1).ok_or_else(||error("limit")))?;
 let prior=original.and_then(|b|serde_json::from_slice::<Record>(b).ok()).filter(|r|r.validate().is_ok());
 let component_format=crate::pet_data::bootstrap::settings_io_observation(settings).map_err(|_|error("profile"))?.as_ref().and_then(|f|f.version()).filter(|v|*v>=3);
 // A crash between marker3 and its first record may leave a valid v1/v2
 // record. It contains no component dispatch and can be promoted by a new
 // data review. Malformed/unknown v3 records must not lose their effect slots.
 let version=review_version(component_format,original,prior.as_ref())?;
 let record=Record{version,generation,transaction:reference.id.clone(),phase:Phase::Prepared,review_required:true,before:before.map(str::to_string),proposed:hash(proposed),suspended_operations:OPERATIONS.iter().map(|s|s.to_string()).collect(),reconciliation:Some(reference),components:prior.map(|r|r.components).unwrap_or_default()};
 record.validate().map_err(error)?;let bytes=serde_json::to_vec_pretty(&record).map_err(|_|error("record"))?;
 previous.commit_guarded(&bytes,guard)?;
 let observation=Observation::read(settings)?;
 Ok(Prepared{settings:settings.into(),observation})
}
fn review_version(component_format:Option<u32>,original:Option<&[u8]>,prior:Option<&Record>)->io::Result<u32>{
 if component_format.is_some()&&original.is_some()&&prior.is_none(){return Err(error("componentReconciliationRequired"));}
 Ok(component_format.unwrap_or(2).max(prior.map(|r|r.version).filter(|v|*v>=3).unwrap_or(2)))
}
pub(super)struct Prepared{settings:PathBuf,observation:Observation}
impl Prepared{
    pub(super)fn current(&self)->bool{self.observation.current(&self.settings)}
    pub(super)fn finish_reviewed(mut self)->io::Result<Observation>{
        if self.observation.snapshot.registry.version<2{return Err(error("review"));}
        self.observation.snapshot.registry.review_required=false;self.finish(Phase::Committed)
    }
    pub(super)fn finish(self,phase:Phase)->io::Result<Observation>{
        if phase==Phase::Prepared||!self.observation.current(&self.settings){return Err(error("changed"));}
        let mut record=self.observation.snapshot.registry.clone();record.phase=phase;
        let snapshot=private_registry::save_checked(&self.observation.path,&self.observation.snapshot.revision,record,|_,_|Ok(())).map_err(error)?;
        Observation::from_snapshot(&self.settings,snapshot)
    }
}
pub(super)fn writer(settings:&Path)->io::Result<private_registry::WriterLock>{private_registry::operation_lock(settings,".settings-io-commit.lock").map_err(error)}

/// Reviewed restore initialization changes settings bytes. Only a stable old
/// outcome may be rebound as stable; unresolved/recovered state remains blocked.
pub(crate)fn restored_record(original:&[u8],old_settings:Option<&[u8]>,new_settings:&[u8])->Result<Vec<u8>,&'static str>{
    if original.len()>Record::MAX_BYTES{return Err("settingsIoLimit");}
    let mut r:Record=serde_json::from_slice(original).map_err(|_|"invalidSettingsIo")?;r.validate()?;
    let old=old_settings.map(hash);
    let stable=match r.phase{Phase::Prepared=>false,Phase::Committed=>old.as_deref()==Some(r.proposed.as_str()),Phase::NotPublished=>old==r.before};
    // Uncertain intent keeps its exact references and proposed hash as data.
    // Copy/activation cannot manufacture its missing outcome or review.
    if !stable{return Ok(original.to_vec());}
    r.generation=r.generation.checked_add(1).ok_or("settingsIoLimit")?;
    r.transaction=hash(format!("coucou-settings-io-reviewed-restore-v1:{}:{}",hash(original),hash(new_settings)).as_bytes())[..32].into();
    r.before=old;r.proposed=hash(new_settings);r.phase=Phase::Committed;
    // Restored receipts remain evidence, never a native reactivation grant.
    if r.version>=3{
        // No copied receipt (or absence of one) revalidates another native
        // component in the new profile. Use the same six-operation contract.
        for name in OPERATIONS{if !r.suspended_operations.iter().any(|n|n==name){r.suspended_operations.push(name.into());}}
    }else{
        // Older formats cannot express the same component journal. Keep the
        // format and require the existing human data-review migration first.
        r.review_required=true;
    }
    serde_json::to_vec(&r).map_err(|_|"invalidSettingsIo")
}

#[cfg(test)]#[path="settings_durable_tests.rs"]mod tests;

#[cfg(test)]#[path="settings_review_foundation_tests.rs"]mod review_foundation;
