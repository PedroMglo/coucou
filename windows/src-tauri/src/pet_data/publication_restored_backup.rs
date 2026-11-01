//! Pure portable data verification. A copied admission describes the old
//! location; this function cannot construct Active, a Claim or execution consent.
use super::*;
use crate::pet_data::{restored_transaction as tx,restored_history::History};
use std::{ffi::OsString,os::unix::ffi::OsStringExt};
pub(in crate::pet_data)fn verify(config:&Dir,data:&Dir,origin:&BackupOrigin)->Result<()>{
 if origin.version()!=2||!origin.bounded(){return Err(Error::UnsupportedProfile);}
 if super::super::settings_io::version(config)?!=origin.settings_io_protocol{return Err(Error::SourceChanged);}
 let old_layout=Layout::new(PathBuf::from(OsString::from_vec(origin.legacy_config.clone())),PathBuf::from(OsString::from_vec(origin.legacy_data.clone())))?;
 let key=layout_key(&old_layout);
 let get=|root:&Dir,name:&str|tx::read_file(root,name,if name==".restore-authority/manifest.json"||name==".restore-authority/history.json"{8*1024*1024}else{256*1024})?.ok_or(Error::SourceChanged);
 for(root,area,hbytes,cbytes)in[(config,Area::Config,&origin.config_header,&origin.config_complete),(data,Area::Data,&origin.data_header,&origin.data_complete)]{
  restore::root_controls(root,area)?;if get(root,HEADER)?!=*hbytes||get(root,COMPLETE)?!=*cbytes{return Err(Error::SourceChanged);}
 }
 let ch:Header=parse(&origin.config_header)?;let dh:Header=parse(&origin.data_header)?;
 if !matches!(ch.version,2|3)||dh.version!=ch.version||ch.area!=Area::Config||dh.area!=Area::Data||!valid_hex(&ch.pair,32)||ch.pair!=dh.pair||ch.layout!=key||dh.layout!=key{return Err(Error::UnsupportedProfile);}
 let cc:Complete=parse(&origin.config_complete)?;let dc:Complete=parse(&origin.data_complete)?;
 if cc.version!=ch.version||dc.version!=ch.version||cc.area!=Area::Config||dc.area!=Area::Data||cc.pair!=ch.pair||dc.pair!=ch.pair||cc.content!=dc.content||cc.summary!=dc.summary{return Err(Error::UnsupportedProfile);}
 let admission=get(config,ADMITTED)?;if admission!=origin.admission{return Err(Error::SourceChanged);}let a:active_restored::Admission2=parse(&admission)?;
 if a.profile_version()?!=ch.version||a.pair!=ch.pair||a.layout!=key||a.config!=digest(&origin.config_complete)||a.data!=digest(&origin.data_complete){return Err(Error::SourceChanged);}
 let job=config.dir(OsStr::new(if a.version==3{tx::EPOCH_JOB}else{tx::JOB}))?;let mb=get(&job,tx::MANIFEST)?;if digest(&mb)!=a.manifest{return Err(Error::SourceChanged);}let m:tx::Manifest=parse(&mb)?;
 if m.admission_version()?!=a.version||m.profile_version()?!=a.profile_version()?||m.epoch!=a.epoch||m.independent!=a.independent||m.depth()?!=origin.source_depth()?||m.pair!=a.pair||m.layout!=key||m.history!=a.history||m.authority!=a.authority||m.original_content!=cc.content||m.pending_features!=["autostart","mobile","voice"]{return Err(Error::UnsupportedProfile);}m.choices.canonical()?;
 for(root,controls)in[(config,&m.config_controls),(data,&m.data_controls)]{for(name,expected)in controls{if digest(&get(root,name)?)!=*expected{return Err(Error::SourceChanged);}}}
 let ready:tx::Ready=parse(&get(&job,tx::READY)?)?;if ready.version!=1||ready.manifest!=a.manifest||!valid_hex(&ready.review,64){return Err(Error::UnsupportedProfile);}
 let seal:serde_json::Value=parse(&get(&job,tx::SEALED)?)?;if seal!=serde_json::json!({"version":1,"admission":digest(&admission),"manifest":a.manifest}){return Err(Error::SourceChanged);}
 let hb=tx::read_file(&job,tx::HISTORY,crate::pet_data::restored_history::MAX_INDEX)?.ok_or(Error::SourceChanged)?;let history=if matches!(a.version,2|4){History::decode_first(&hb,&a.history)?}else{History::decode(&hb,&a.history)?};if let Some(e)=&m.epoch{if history.review()?!=e.scope.references{return Err(Error::SourceChanged);}restore::epoch::verify_retained_custody(config,&e.scope.custody,&m.pair,&m.layout,&m.source)?;}
 let authority=m.payload(config)?.bound_path()?.join("pet-runtime-consent-v1");let view=crate::pet_runtime::restoration::historical_view(&authority).map_err(|_|Error::UnsupportedProfile)?;
 if view["historyHash"].as_str()!=Some(a.authority.as_str())||view["historyEpochs"].as_u64()!=Some(u64::from(m.depth()?)){return Err(Error::SourceChanged);}
 let originals=config.dir(OsStr::new(tx::JOB))?.dir(OsStr::new("originals"))?;let originals_path=originals.bound_path()?;staging::verify_original_content(&originals_path.join("config"),&originals_path.join("data"),&m.original_content)?;if let Some(i)=&m.independent{restore::independent::operational::verify_originals(config,&originals.dir(OsStr::new("config"))?,&originals.dir(OsStr::new("data"))?,i,&m.pair,&m.layout,&m.source)?;}
 match tx::read_file(&job,tx::OPENED,4096)?{
  Some(b)=>{if parse::<serde_json::Value>(&b)?!=serde_json::json!({"version":1,"admission":digest(&admission)}){return Err(Error::SourceChanged);}},
  None=>active_restored::verify_initial_payload(config,data,&m,&job)?,
 }
 config.check()?;data.check()?;Ok(())
}
