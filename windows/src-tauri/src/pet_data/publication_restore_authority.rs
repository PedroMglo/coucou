//! Quarantine under the same inactive-data claim, not restored admission.
use super::*;
use crate::pet_runtime::restoration;
pub(super)fn error(e:restoration::HistoryFailure)->Error{use restoration::HistoryFailure as F;match e{F::Busy=>Error::Busy,F::Capacity=>Error::Limit,F::Changed|F::Missing=>Error::SourceChanged,F::Unsafe|F::Invalid|F::WrongKey|F::RecoveryRequired=>Error::UnsupportedProfile,_=>Error::Unavailable}}
pub(in crate::pet_data)enum AuthorityReview{First(FirstReview),Next(super::epoch::EpochAuthorityReview),Independent(super::independent::IndependentReview)}
pub(in crate::pet_data)struct FirstReview{data:Review,authority:restoration::Review}
impl Review{
 pub(in crate::pet_data)fn authority(self)->Result<AuthorityReview>{
  if self.mode==Mode::RetainIndependent{return Err(Error::UnsupportedProfile);}
  if self.profile_version()==3{return super::epoch::EpochAuthorityReview::capture(self).map(AuthorityReview::Next);}
  if private_fs::Node::root(self.source.source_root(Area::Config).join("pet-runtime-consent-v1").as_path())?.is_none(){return super::independent::IndependentReview::capture(self).map(AuthorityReview::Independent);}
  let path=self.layout.config().join(PAYLOAD).join("pet-runtime-consent-v1");let goals=self.layout.config().join(PAYLOAD).join("pet-runtime-v1");
  self.authority_data_current()?;
  let delta=match restoration::publication_delta(&path){Ok(v)=>v,Err(restoration::HistoryFailure::Missing)=>vec![],Err(e)=>return Err(error(e))};self.authority_verify_delta(&delta)?;
  let authority=restoration::Review::capture(&path,&goals,&self.source_fence,||self.authority_data_current().map_err(|_|restoration::HistoryFailure::Changed)).map_err(error)?;
  Ok(AuthorityReview::First(FirstReview{data:self,authority}))
 }
 pub(super)fn authority_data_current(&self)->Result<()>{
  if self.profile_version()!=2{return Err(Error::UnsupportedProfile);}self.restored_data_current()
 }
 pub(in crate::pet_data)fn restored_data_current(&self)->Result<()>{
  self.current()?;let(roots,pair)=self.existing()?;if pair!=self.pair{return Err(Error::SourceChanged);}let id=pair.ok_or(Error::UnsupportedProfile)?;let config=roots[0].as_ref().ok_or(Error::UnsupportedProfile)?;let data=roots[1].as_ref().ok_or(Error::UnsupportedProfile)?;
  let records=[encode(&Complete{version:self.profile_version(),pair:id.clone(),area:Area::Config,content:self.content.clone(),summary:self.source.summary()})?,encode(&Complete{version:self.profile_version(),pair:id.clone(),area:Area::Data,content:self.content.clone(),summary:self.source.summary()})?];
  if read(&config.dir,COMPLETE)?.as_deref()!=Some(records[0].as_slice())||read(&data.dir,COMPLETE)?.as_deref()!=Some(records[1].as_slice()){return Err(Error::SourceChanged);}
  let expected=encode(&Restored{version:self.source.archive_version(),pair:id,layout:layout_key(&self.layout),source:digest(&encode(&self.intent(self.pair.as_deref().ok_or(Error::UnsupportedProfile)?))?),config:digest(&records[0]),data:digest(&records[1]),authority_state:if self.source.archive_version()==1{"historicalOnlyPendingReconciliation"}else{"historicalRootPairPendingEpochReconciliation"}.into()})?;
  if read(&config.dir,RESTORED)?.as_deref()!=Some(expected.as_slice()){return Err(Error::SourceChanged);}Ok(())
 }
 pub(super)fn authority_verify_delta(&self,delta:&[(String,Vec<u8>)])->Result<()>{
  let(roots,_)=self.existing()?;let config=roots[0].as_ref().ok_or(Error::SourceChanged)?.dir.dir(OsStr::new(PAYLOAD))?;let data=roots[1].as_ref().ok_or(Error::SourceChanged)?.dir.dir(OsStr::new(PAYLOAD))?;
  let delta:Vec<_>=delta.iter().map(|(name,bytes)|(Area::Config,PathBuf::from("pet-runtime-consent-v1").join(name),bytes.clone())).collect();staging::verify_publication_delta(&self.source,&config,&data,&delta)?;self.authority_data_current()
 }
}
impl FirstReview{
 fn fingerprint(&self)->Result<String>{self.data.authority_data_current()?;Ok(digest(&serde_json::to_vec(&("coucou-inactive-restore-authority-review-v1",self.data.fingerprint()?,self.authority.fingerprint().map_err(error)?)).map_err(|_|Error::Unavailable)?))}
 pub(in crate::pet_data)fn preview(&self)->Result<serde_json::Value>{Ok(serde_json::json!({"status":"restoredAuthorityReview","schemaVersion":1,"fingerprint":self.fingerprint()?,"pair":self.data.pair,"authority":self.authority.preview().map_err(error)?,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"sensitiveSettingsNotActivated":true}))}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<serde_json::Value,Failure>{
  let pair=self.data.pair.clone();let result=(||->Result<serde_json::Value>{if approved!=self.fingerprint()?{return Err(Error::SourceChanged);}let core_hash=self.authority.fingerprint().map_err(error)?;let result=self.authority.apply(&core_hash,||self.data.authority_data_current().map_err(|_|restoration::HistoryFailure::Changed)).map_err(error)?;
   let path=self.data.layout.config().join(PAYLOAD).join("pet-runtime-consent-v1");let delta=restoration::publication_delta(&path).map_err(error)?;self.data.authority_verify_delta(&delta)?;let history=restoration::historical_view(&path).map_err(error)?;
   Ok(serde_json::json!({"status":"restoredAuthorityQuarantined","schemaVersion":1,"pair":self.data.pair,"authority":result,"history":history,"executionAuthorityGranted":false,"runtimeActivationAvailable":false,"sensitiveSettingsNotActivated":true}))})();result.map_err(|error|Failure{error,pair})
 }
}

impl AuthorityReview{
 pub(in crate::pet_data)fn preview(&self)->Result<serde_json::Value>{match self{Self::First(r)=>r.preview(),Self::Next(r)=>r.preview(),Self::Independent(r)=>r.preview()}}
 pub(in crate::pet_data)fn apply(self,approved:&str)->std::result::Result<serde_json::Value,Failure>{match self{Self::First(r)=>r.apply(approved),Self::Next(r)=>r.apply(approved),Self::Independent(r)=>r.apply(approved)}}
}
