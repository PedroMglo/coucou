//! Explicit offline native data maintenance. Runs before Tauri UI/Chat/workers.
//! No provider, model, credential-vault, service or host configuration operations.
use super::{current_profile::Review,publication::Active,startup::Claim,staging,Layout,Error};
use std::path::PathBuf;
#[derive(Debug,PartialEq,Eq)]pub(super)enum Request{Backup{workspace:PathBuf,approve:Option<String>},Inspect{workspace:PathBuf,id:String}}
pub(super)fn parse(args:&[String])->std::result::Result<Option<Request>,Error>{
    if !args.iter().skip(1).any(|s|s.starts_with("--pets-profile-")){return Ok(None);}
    let valid=|s:&str,n|s.len()==n&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b));
    match args.get(1).map(String::as_str){
        Some("--pets-profile-backup")if args.len()==3=>Ok(Some(Request::Backup{workspace:PathBuf::from(args[2].as_str()),approve:None})),
        Some("--pets-profile-backup")if args.len()==5&&args[3]=="--approve"&&valid(&args[4],64)=>Ok(Some(Request::Backup{workspace:PathBuf::from(args[2].as_str()),approve:Some(args[4].clone())})),
        Some("--pets-profile-inspect")if args.len()==4&&valid(&args[3],32)=>Ok(Some(Request::Inspect{workspace:PathBuf::from(args[2].as_str()),id:args[3].clone()})),
        _=>Err(Error::UnsupportedProfile),
    }
}
fn code(error:Error)->&'static str{match error{Error::Busy=>"profileBusy",Error::OwnershipLost=>"ownershipLost",Error::SourceChanged=>"reviewOrDataChanged",Error::TargetExists=>"targetExists",Error::Limit=>"limit",Error::UnsafePath=>"unsafePath",Error::UnsupportedProfile=>"unsupportedProfileOrRequest",Error::Unavailable=>"unavailable"}}
pub(super)fn execute(request:Request,address:&str,identifier:&str,layout:Layout)->std::result::Result<serde_json::Value,staging::Failure>{
    let failure=|error|staging::Failure{error,candidate_id:None};
    match request{
        Request::Inspect{workspace,id}=>{
            let inspection=staging::inspect_current(&workspace,&id).map_err(failure)?;
            Ok(serde_json::json!({"status":if inspection.phase==staging::Phase::VerifiedData{"verifiedData"}else{"incomplete"},"schemaVersion":1,"candidateId":id,"summary":inspection.expected,"archiveVersion":inspection.archive_version,"includesRecoveryMetadata":inspection.includes_recovery_metadata,"executionAuthorityGranted":false,"restorationAvailable":false,"sourcePayloadDepth":inspection.source_payload_depth,"dataRestorationAvailable":inspection.phase==staging::Phase::VerifiedData&&inspection.source_payload_depth.is_some_and(|d|d<super::restored_transaction::MAX_PAYLOAD_DEPTH),"runtimeActivationAvailable":false,"explicitReviewRequired":true}))
        },
        Request::Backup{workspace,approve}=>{
            let claim=Claim::maintenance(address,identifier,&layout).map_err(failure)?;
            let active=Active::open(claim,layout).map_err(failure)?;let review=Review::capture(active,&workspace).map_err(failure)?;
            let Some(approved)=approve else{return review.preview().map_err(failure);};
            let archive_version=review.format_version();let source_depth=review.source_depth().map_err(failure)?;let data_restoration_available=review.data_restoration_available().map_err(failure)?;let(id,summary)=review.copy(&approved)?;
            Ok(serde_json::json!({"status":"verifiedData","schemaVersion":1,"candidateId":id,"summary":summary,"archiveVersion":archive_version,"includesRecoveryMetadata":archive_version==2,"executionAuthorityGranted":false,"restorationAvailable":false,"sourcePayloadDepth":source_depth,"dataRestorationAvailable":data_restoration_available,"runtimeActivationAvailable":false,"explicitReviewRequired":true}))
        },
    }
}
pub(crate)fn entry(identifier:&str)->bool{
    let args=std::env::args_os().collect::<Vec<_>>();
    if !args.iter().skip(1).any(|s|s.to_str().is_some_and(|s|s.starts_with("--pets-profile-"))){return false;}
    let result=(||{
        let args=args.into_iter().map(|s|s.into_string().map_err(|_|Error::UnsupportedProfile)).collect::<std::result::Result<Vec<_>,_>>().map_err(|error|staging::Failure{error,candidate_id:None})?;
        let request=parse(&args).map_err(|error|staging::Failure{error,candidate_id:None})?.ok_or(staging::Failure{error:Error::UnsupportedProfile,candidate_id:None})?;
        let address=std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();
        let layout=Layout::new(crate::platform::legacy_config_dir(),crate::platform::local_dir()).map_err(|error|staging::Failure{error,candidate_id:None})?;
        execute(request,&address,identifier,layout)
    })();
    let (output,exit)=match result{Ok(value)=>(value,0),Err(f)=>(serde_json::json!({"status":"refused","reason":code(f.error),"candidateId":f.candidate_id,"mayContainPartialData":f.candidate_id.is_some(),"retryAutomatically":false,"executionAuthorityGranted":false}),2)};
    println!("{}",output);std::process::exit(exit)
}
