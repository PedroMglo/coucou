//! Explicit offline DATA restoration. Never falls through into app setup and
//! never activates copied settings, native approvals, workers or services.
use super::{publication::{self,restore::{Mode,Review}},startup::Claim,Error,Layout};
use std::path::PathBuf;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(super)enum Action{Data,Authority,Configuration,Activate,RetainIndependent}
#[derive(Debug,PartialEq,Eq)]pub(super)struct Request{workspace:PathBuf,id:String,approve:Option<String>,mode:Mode,action:Action,plan:Option<PathBuf>}
pub(super)fn parse(args:&[String])->Result<Option<Request>,Error>{
 if !args.iter().skip(1).any(|s|s.starts_with("--pets-profile-restore")){return Ok(None);}
 let hex=|s:&str,n|s.len()==n&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b));
 let(mode,action)=match args.get(1).map(String::as_str){Some("--pets-profile-restore")=>(Mode::Fresh,Action::Data),Some("--pets-profile-restore-resume")=>(Mode::Resume,Action::Data),Some("--pets-profile-restore-authority")=>(Mode::Resume,Action::Authority),Some("--pets-profile-restore-configuration")=>(Mode::Resume,Action::Configuration),Some("--pets-profile-restore-activate")=>(Mode::Activation,Action::Activate),Some("--pets-profile-restore-retain-preparation")=>(Mode::RetainIndependent,Action::RetainIndependent),_=>return Err(Error::UnsupportedProfile)};
 let(plan,approve)=if action==Action::Activate{
  if !matches!(args.len(),6|8)||!hex(&args[3],32)||args[4]!="--plan"||!PathBuf::from(&args[5]).is_absolute()||args.len()==8&&(args[6]!="--approve"||!hex(&args[7],64)){return Err(Error::UnsupportedProfile);}(Some(PathBuf::from(&args[5])),args.get(7).cloned())
 }else{if !matches!(args.len(),4|6)||!hex(&args[3],32)||args.len()==6&&(args[4]!="--approve"||!hex(&args[5],64)){return Err(Error::UnsupportedProfile);}(None,args.get(5).cloned())};
 Ok(Some(Request{workspace:PathBuf::from(&args[2]),id:args[3].clone(),approve,mode,action,plan}))
}
fn activation_sealed(layout:&Layout)->Result<bool,Error>{
 let config=super::staging_fs::Dir::root(layout.config())?;let header=publication::read(&config,publication::HEADER)?;let version=header.as_deref().map(|b|serde_json::from_slice::<serde_json::Value>(b).map_err(|_|Error::UnsupportedProfile)).transpose()?;let job=if version.as_ref().and_then(|v|v["version"].as_u64())==Some(3){super::restored_transaction::EPOCH_JOB}else{super::restored_transaction::JOB};super::restored_transaction::read_file(&config,&format!("{job}/{}",super::restored_transaction::SEALED),4096).map(|v|v.is_some())
}
pub(super)fn execute(request:Request,address:&str,identifier:&str,layout:Layout)->Result<serde_json::Value,publication::Failure>{
 let failure=|error|publication::Failure{error,pair:None};
 let claim=Claim::maintenance(address,identifier,&layout).map_err(failure)?;
 if request.action==Action::Activate{
  let plan=request.plan.as_ref().ok_or_else(||failure(Error::UnsupportedProfile))?;let parent=crate::pet_data::staging_fs::Dir::parent(plan.parent().ok_or_else(||failure(Error::UnsafePath))?).map_err(failure)?;
  let bytes=crate::pet_data::restored_transaction::read_file(&parent,plan.file_name().and_then(|n|n.to_str()).ok_or_else(||failure(Error::UnsafePath))?,16*1024).map_err(failure)?.ok_or_else(||failure(Error::Unavailable))?;let choices=crate::pet_data::restored_projection::Choices::parse(&bytes).map_err(failure)?;
  if activation_sealed(&layout).map_err(failure)?{let active=publication::Active::open(claim,layout).map_err(failure)?;return active.activation_ack(&request.id,&choices,request.approve.as_deref()).map_err(failure);}
  let review=crate::pet_data::restored_transaction::Activation::capture(claim,layout,&request.workspace,&request.id,choices).map_err(failure)?;return match request.approve{None=>review.preview().map_err(failure),Some(hash)=>review.apply(&hash)};
 }
 let review=Review::capture(claim,layout,&request.workspace,&request.id,request.mode).map_err(failure)?;
 match request.action{
  Action::Authority=>{let review=review.authority().map_err(failure)?;match request.approve{None=>review.preview().map_err(failure),Some(hash)=>review.apply(&hash)}},
  Action::Configuration=>{let review=review.configuration().map_err(failure)?;match request.approve{None=>review.preview().map_err(failure),Some(hash)=>review.apply(&hash)}},
  Action::Activate=>Err(failure(Error::UnsupportedProfile)),
  Action::Data=>match request.approve{None=>review.preview().map_err(failure),Some(hash)=>review.apply(&hash)},
  Action::RetainIndependent=>{let review=publication::restore::retention::RetentionReview::capture(review).map_err(failure)?;match request.approve{None=>review.preview().map_err(failure),Some(hash)=>review.apply(&hash)}},
 }
}

fn code(error:Error)->&'static str{match error{Error::Busy=>"profileBusy",Error::OwnershipLost=>"ownershipLost",Error::SourceChanged=>"reviewOrDataChanged",Error::TargetExists=>"targetExists",Error::Limit=>"limit",Error::UnsafePath=>"unsafePath",Error::UnsupportedProfile=>"unsupportedProfileOrRequest",Error::Unavailable=>"unavailable"}}
pub(crate)fn entry(identifier:&str)->bool{
 let args=std::env::args_os().collect::<Vec<_>>();if !args.iter().skip(1).any(|s|s.to_str().is_some_and(|s|s.starts_with("--pets-profile-restore"))){return false;}
 let result=(||{let fail=|error|publication::Failure{error,pair:None};let args=args.into_iter().map(|v|v.into_string().map_err(|_|Error::UnsupportedProfile)).collect::<Result<Vec<_>,_>>().map_err(fail)?;let request=parse(&args).map_err(fail)?.ok_or_else(||fail(Error::UnsupportedProfile))?;let layout=Layout::new(crate::platform::legacy_config_dir(),crate::platform::local_dir()).map_err(fail)?;let address=std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();execute(request,&address,identifier,layout)})();
 let(output,exit)=match result{Ok(value)=>(value,0),Err(f)=>(serde_json::json!({"status":"refused","reason":code(f.error),"restorationPair":f.pair,"mayContainPartialData":f.pair.is_some(),"retryAutomatically":false,"executionAuthorityGranted":false,"runtimeActivationAvailable":false}),2)};
 println!("{}",output);std::process::exit(exit)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn literal_activation_seal_is_a_file_and_unsafe_aliases_are_refused(){
  use std::{fs,os::unix::fs::{DirBuilderExt,symlink}};let temp=crate::pet_data::tests::Temp::new();let layout=temp.layout();fs::DirBuilder::new().recursive(true).mode(0o700).create(layout.config()).unwrap();assert!(!activation_sealed(&layout).unwrap());let seal=layout.config().join(super::super::restored_transaction::JOB).join(super::super::restored_transaction::SEALED);temp.file(&seal,b"private marker only; Active validates its contents");assert!(activation_sealed(&layout).unwrap());fs::remove_file(&seal).unwrap();symlink(temp.0.join("outside"),&seal).unwrap();assert!(activation_sealed(&layout).is_err());
 }
 #[test]fn parser_accepts_only_exact_restore_forms_and_cannot_fall_through_mixed_flags(){
  let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();let id="0123456789abcdef0123456789abcdef";let hash="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
  assert_eq!(parse(&args(&["coucou","--voice-session"])).unwrap(),None);
  assert_eq!(parse(&args(&["coucou","--pets-profile-restore","/private/archive",id])).unwrap().unwrap().mode,Mode::Fresh);
  assert_eq!(parse(&args(&["coucou","--pets-profile-restore-resume","/private/archive",id,"--approve",hash])).unwrap().unwrap().mode,Mode::Resume);
  let authority=parse(&args(&["coucou","--pets-profile-restore-authority","/private/archive",id,"--approve",hash])).unwrap().unwrap();assert_eq!(authority.action,Action::Authority);assert_eq!(authority.mode,Mode::Resume);
  let configuration=parse(&args(&["coucou","--pets-profile-restore-configuration","/private/archive",id])).unwrap().unwrap();assert_eq!(configuration.action,Action::Configuration);assert_eq!(configuration.mode,Mode::Resume);
  let activation=parse(&args(&["coucou","--pets-profile-restore-activate","/private/archive",id,"--plan","/private/choices.json","--approve",hash])).unwrap().unwrap();assert_eq!(activation.action,Action::Activate);assert_eq!(activation.mode,Mode::Activation);assert_eq!(activation.plan,Some(PathBuf::from("/private/choices.json")));
  for v in [vec!["coucou","--pets-profile-restore-activate","/private/archive",id],vec!["coucou","--pets-profile-restore-activate","/private/archive",id,"--plan","relative.json"],vec!["coucou","--pets-profile-restore-activate","/private/archive",id,"--plan","/private/choices.json","--approve","bad"],vec!["coucou","--pets-profile-restore-activate","/private/archive",id,"--plan","/private/choices.json","--voice-session"]]{assert!(parse(&args(&v)).is_err());}
  for v in [vec!["coucou","--pets-profile-restore","/tmp"],vec!["coucou","--pets-profile-restore-unknown","/tmp",id],vec!["coucou","--pets-profile-backup","--pets-profile-restore","/tmp",id],vec!["coucou","--pets-profile-restore","/tmp","../old"],vec!["coucou","--pets-profile-restore","/tmp",id,"--approve","bad"]]{assert!(parse(&args(&v)).is_err());}
 }
}
