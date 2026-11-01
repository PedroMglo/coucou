//! Observation receipts use the original effect slot and retain its previous
//! outcome, without granting registration or asserting a historical success.
use super::*;
pub(super)fn sources(path:&Path,value:&Artifact)->io::Result<Vec<Snapshot<Artifact>>>{
 if value.evidence.get("reconciliationProtocol").is_none(){return Ok(vec![]);}
 if value.component!="shortcuts"||value.phase!=Phase::ReconciledObservation||value.evidence.get("reconciliationProtocol")!=Some(&serde_json::json!(1))||value.evidence.get("noRegistrationEffect")!=Some(&serde_json::json!(true))||value.evidence.get("noReactivation")!=Some(&serde_json::json!(true))||value.evidence.get("pastEffectOutcomeKnown")!=Some(&serde_json::json!(false)){return Err(error("componentShortcutObservation"));}
 let subject:Slot=serde_json::from_value(value.evidence.get("subject").cloned().ok_or_else(||error("componentShortcutSubject"))?).map_err(|_|error("componentShortcutSubject"))?;
 subject.validate().map_err(error)?;
 if subject.id!=value.effect||subject.component!=value.component||subject.binding!=value.binding{return Err(error("componentShortcutSubject"));}
 let native=value.evidence.get("nativeObservation").ok_or_else(||error("componentShortcutObservation"))?;
 if native.get("canReconcile")!=Some(&serde_json::json!(true))||native.get("currentInstance").and_then(serde_json::Value::as_str).is_none_or(|s|!hex(s,32)){return Err(error("componentShortcutObservation"));}
 let intent=observe(path,&subject.intent)?;if !matches(&intent.registry,&subject,Phase::Prepared){return Err(error("componentShortcutSubject"));}
 let mut held=vec![intent];if let Some(reference)=&subject.outcome{let outcome=observe(path,reference)?;if !matches(&outcome.registry,&subject,subject.phase){return Err(error("componentShortcutSubject"));}held.push(outcome);}Ok(held)
}
pub(super)fn transition(path:&Path,old:&Slot,next:&Slot)->io::Result<()>{
 let outcome=observe(path,next.outcome.as_ref().ok_or_else(||error("componentShortcutObservation"))?)?;
 if next.component!="shortcuts"||next.phase!=Phase::ReconciledObservation||outcome.registry.evidence.get("reconciliationProtocol")!=Some(&serde_json::json!(1))||outcome.registry.evidence.get("subject")!=Some(&serde_json::to_value(old).map_err(|_|error("componentShortcutSubject"))?){return Err(error("componentShortcutSubject"));}
 sources(path,&outcome.registry)?;Ok(())
}
