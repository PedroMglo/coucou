//! Two-stage result in the existing component journal. An independently
//! observed registration is not yet a fully published/completed operation.
use super::*;
fn stage(value:&Artifact)->io::Result<Option<&str>>{
 if value.evidence.get("publicationProtocol").is_none(){return Ok(None);}
 let stage=value.evidence.get("publication").and_then(serde_json::Value::as_str).ok_or_else(||error("componentPublication"))?;
 if value.component!="shortcuts"||value.evidence.get("publicationProtocol")!=Some(&serde_json::json!(1))
  ||value.evidence.pointer("/nativeObservation/verified")!=Some(&serde_json::Value::Bool(true))
  ||!matches!((value.phase,stage),(Phase::Unconfirmed,"pending")|(Phase::Verified,"published")){
  return Err(error("componentPublication"));
 }Ok(Some(stage))
}
pub(super)fn sources(settings:&Path,value:&Artifact)->io::Result<Vec<Snapshot<Artifact>>>{
 match stage(value)?{
  None|Some("pending")=>Ok(vec![]),
  Some("published")=>{
   let reference:ReconciliationRef=serde_json::from_value(value.evidence.get("pendingOutcome").cloned().ok_or_else(||error("componentPublicationSource"))?).map_err(|_|error("componentPublicationSource"))?;
   let pending=observe(settings,&reference)?;
   if stage(&pending.registry)?!=Some("pending")||pending.registry.effect!=value.effect||pending.registry.binding!=value.binding
    ||pending.registry.evidence.get("nativeObservation")!=value.evidence.get("nativeObservation"){
    return Err(error("componentPublicationSource"));
   }Ok(vec![pending])
  },_=>Err(error("componentPublication"))
 }
}
pub(super)fn final_transition(settings:&Path,old:&Slot,next:&Slot)->io::Result<()>{
 let outcome=observe(settings,next.outcome.as_ref().ok_or_else(||error("componentPublicationSource"))?)?;
 if old.component!="shortcuts"||old.phase!=Phase::Unconfirmed||next.phase!=Phase::Verified||stage(&outcome.registry)?!=Some("published"){
  return Err(error("componentPublicationSource"));
 }
 let reference:ReconciliationRef=serde_json::from_value(outcome.registry.evidence.get("pendingOutcome").cloned().ok_or_else(||error("componentPublicationSource"))?).map_err(|_|error("componentPublicationSource"))?;
 if old.outcome.as_ref()!=Some(&reference){return Err(error("componentPublicationSource"));}
 sources(settings,&outcome.registry)?;Ok(())
}
pub(crate)fn execution(settings:&crate::settings::Settings,path:&Path,id:&str,binding:&Binding,reference:&ReconciliationRef,review:Option<&ReconciliationRef>)->io::Result<Execution>{
 crate::settings::revision::component_read(settings,path,|io,witness|{
  let slot=io.component("shortcuts").ok_or_else(||error("componentPublicationSource"))?;
  if slot.id!=id||slot.phase!=Phase::Unconfirmed||&slot.binding!=binding||slot.outcome.as_ref()!=Some(reference)||io.snapshot.registry.reconciliation.as_ref()!=review{return Err(error("componentPublicationSource"));}
  let result=observe(path,reference)?;if stage(&result.registry)?!=Some("pending"){return Err(error("componentPublicationSource"));}
  Ok(Execution{path:path.into(),io:io.clone(),witness:witness.clone(),owner:crate::settings::revision::guard(settings),profile:binding.profile.clone()})
 })
}
