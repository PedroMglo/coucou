//! Entire current application payload, held before Chat/workers exist.
//! Native ownership is not serialized into the archive as a reusable permit.
use super::{inventory::{self,Area,Snapshot},publication::{Active,Paths,BackupOrigin},staging::{self,Source,Kind},staging_fs::Dir,private_fs,Error,Result};
use std::{path::{Path,PathBuf},os::unix::ffi::OsStrExt};
pub(super)struct Review{active:Active,paths:Paths,snapshot:Snapshot,workspace:PathBuf,target:Dir,origin:BackupOrigin}
impl Review{
    pub(super)fn capture(active:Active,workspace:&Path)->Result<Self>{
        active.current()?;
        if !super::absolute(workspace)||[&active.layout().legacy_config,&active.layout().legacy_data].into_iter().any(|p|super::overlaps(workspace,p)){return Err(Error::UnsafePath);}
        let target=Dir::root(workspace)?;let origin=active.backup_origin()?;let paths=if origin.version()==2{Paths{config:active.layout().config().into(),data:active.layout().data().into()}}else{active.paths()?};
        let snapshot=inventory::inspect_current(&paths.config,&paths.data)?;
        let review=Self{active,paths,snapshot,workspace:workspace.into(),target,origin};review.ensure_current()?;Ok(review)
    }
    pub(super)fn fingerprint(&self)->Result<String>{
        self.ensure_current()?;let at=private_fs::location(&self.target.node()?.file)?;
        let bytes=serde_json::to_vec(&("coucou-current-profile-backup-review-v1",self.snapshot.fingerprint()?,&self.origin,self.paths.config.as_os_str().as_bytes(),self.paths.data.as_os_str().as_bytes(),self.workspace.as_os_str().as_bytes(),at.device,at.inode,at.mount)).map_err(|_|Error::Unavailable)?;
        Ok(ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect())
    }
    pub(super)fn source_depth(&self)->Result<u8>{self.origin.source_depth()}
    pub(super)fn data_restoration_available(&self)->Result<bool>{Ok(self.source_depth()?<super::restored_transaction::MAX_PAYLOAD_DEPTH)}
    pub(super)fn format_version(&self)->u32{self.origin.version()}
    // Legacy restorationAvailable refers to executable activation. Data copy
    // support and its explicit review are projected separately.
    pub(super)fn preview(&self)->Result<serde_json::Value>{
        Ok(serde_json::json!({"status":"review","schemaVersion":1,"fingerprint":self.fingerprint()?,"source":{"config":self.paths.config,"data":self.paths.data},"destinationWorkspace":self.workspace,"summary":self.snapshot.summary,"containsPrivateApplicationDataAndKeys":true,"archiveVersion":self.format_version(),"includesRecoveryMetadata":self.origin.version()==2,"scope":if self.origin.version()==2{"entireAdmittedRootPairWithRecoveryMetadata"}else{"entireAdmittedPayloadPair"},"executionAuthorityGranted":false,"restorationAvailable":false,"sourcePayloadDepth":self.source_depth()?,"dataRestorationAvailable":self.data_restoration_available()?,"runtimeActivationAvailable":false,"explicitReviewRequired":true}))
    }
    pub(super)fn copy(self,approved:&str)->std::result::Result<(String,super::Summary),staging::Failure>{
        let fingerprint=self.fingerprint().map_err(|error|staging::Failure{error,candidate_id:None})?;
        if approved!=fingerprint{return Err(staging::Failure{error:Error::SourceChanged,candidate_id:None});}
        let path=self.workspace.clone();let prepared=staging::prepare(self,&path)?;let id=prepared.id().to_string();
        let summary=prepared.verify().map_err(|error|staging::Failure{error,candidate_id:Some(id.clone())})?;Ok((id,summary))
    }
}
impl Source for Review{
    fn snapshot(&self)->&Snapshot{&self.snapshot}
    fn source_root(&self,area:Area)->&Path{match area{Area::Config=>&self.paths.config,Area::Data=>&self.paths.data}}
    fn ensure_current(&self)->Result<()>{
        self.active.current()?;self.target.check()?;if self.active.backup_origin()?!=self.origin{return Err(Error::SourceChanged);}
        if inventory::inspect_current(&self.paths.config,&self.paths.data)?!=self.snapshot{return Err(Error::SourceChanged);}
        self.active.current()
    }
    fn kind(&self)->Kind{if self.origin.version()==2{Kind::Restored}else{Kind::Current}}
    fn origin(&self)->Option<BackupOrigin>{Some(self.origin.clone())}
    fn workspace(&self,path:&Path)->Result<Dir>{if path!=self.workspace{return Err(Error::SourceChanged);}self.target.check()?;Ok(self.target.clone())}
}
#[cfg(test)]#[path="backup_tests.rs"]mod tests;
