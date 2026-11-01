//! Present native observations are separate from unknown historical outcomes.
//! They do not grant effects, renew a consent, or choose archived target paths.
use super::*;
use crate::pet_autostart::native::View;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct HistoricalObservation{
 pub schema_version:u32,pub subject:Slot,pub profile:String,pub data_review:Option<ReconciliationRef>,pub native_plan:View,
 #[serde(default,skip_serializing_if="not_set")]pub legacy_origin_unverifiable:bool,
 #[serde(default,skip_serializing_if="not_set")]pub legacy_uncertainty_acknowledged:bool,
 pub registration_matches_expected:bool,pub no_registration_effect:bool,pub no_reactivation:bool,pub login_verified:bool,pub past_effect_outcome_known:bool,
}
fn not_set(value:&bool)->bool{!*value}
impl HistoricalObservation{
 fn legacy(&self)->bool{self.schema_version==2&&self.legacy_origin_unverifiable&&self.legacy_uncertainty_acknowledged}
 fn action(&self)->&'static str{if self.legacy(){"reconcileLegacyObservation"}else{"reconcileArchivedObservation"}}
 fn subject_plan(&self,artifact:&Artifact)->io::Result<()>{
  let old:View=serde_json::from_value(artifact.evidence.get("nativePlan").cloned().ok_or_else(||error("componentObservationSubjectPlan"))?).map_err(|_|error("componentObservationSubjectPlan"))?;
  if old.resource!=self.subject.binding.resource||old.fingerprint!=self.subject.binding.plan||old.enabled!=self.native_plan.enabled||if self.legacy(){old.schema_version!=1}else{old.schema_version!=self.native_plan.schema_version}{return Err(error("componentObservationSubjectPlan"));}Ok(())
 }
 fn validate(&self,binding:&Binding)->io::Result<()>{
  self.subject.validate().map_err(error)?;let mut plan=self.native_plan.clone();let fingerprint=plan.fingerprint.clone();plan.fingerprint.clear();
  if !(self.schema_version==1&&!self.legacy_origin_unverifiable&&!self.legacy_uncertainty_acknowledged||self.legacy()&&self.native_plan.schema_version==2)||self.subject.component!="autostart"||self.subject.phase.replaceable()||self.profile!=binding.profile
   ||(!self.legacy()&&self.subject.binding.resource!=binding.resource)||self.native_plan.resource!=binding.resource||fingerprint!=binding.plan
   ||hash(&serde_json::to_vec(&plan).map_err(|_|error("componentObservation"))?)!=fingerprint
   ||!matches!(plan.schema_version,1|2)||plan.component!="autostart"||!self.no_registration_effect||!self.no_reactivation||self.login_verified||self.past_effect_outcome_known
   ||self.data_review.as_ref().is_some_and(|r|!reference_valid(r)){return Err(error("componentHistoricalObservation"));}Ok(())
 }
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct HistoricalProjection{pub effect_id:String,pub reference:ReconciliationRef,pub observation:HistoricalObservation,pub current_for_profile:bool}
pub(crate)struct HistoryView{pub unresolved:Vec<Slot>,pub observed:Vec<HistoricalProjection>}
pub(crate)struct HistoricalWitness{chain:Chain}
impl HistoricalWitness{pub(crate)fn current(&self)->bool{self.chain.current()}}
struct Node{slot:Slot,intent:Snapshot<Artifact>,outcome:Option<Snapshot<Artifact>>,related:Vec<Snapshot<Artifact>>}
struct Chain{nodes:Vec<Node>}
impl Chain{
 fn capture(settings:&Path,slot:Option<&Slot>)->io::Result<Self>{
  let mut nodes=vec![];let mut seen=std::collections::BTreeSet::new();let mut next=slot.cloned();
  while let Some(slot)=next{
   if seen.len()>=64||!seen.insert(slot.id.clone()){return Err(error("componentHistoryCycleOrLimit"));}
   let intent=observe(settings,&slot.intent)?;if !matches(&intent.registry,&slot,Phase::Prepared){return Err(error("componentIntent"));}
   let outcome=slot.outcome.as_ref().map(|r|observe(settings,r)).transpose()?;
   if outcome.as_ref().is_some_and(|o|!matches(&o.registry,&slot,slot.phase)){return Err(error("componentOutcome"));}
   let related=outcome.as_ref().map(|o|super::linked_sources(settings,&o.registry)).transpose()?.unwrap_or_default();
   next=intent.registry.previous.clone();nodes.push(Node{slot,intent,outcome,related});
  }
  for(index,node)in nodes.iter().enumerate(){super::shortcut_scope::validate(&node.intent.registry,&nodes[index+1..].iter().map(|n|n.slot.clone()).collect::<Vec<_>>())?;}
  Ok(Self{nodes})
 }
 fn current(&self)->bool{self.nodes.iter().all(|n|n.intent.unchanged()&&n.outcome.as_ref().is_none_or(|o|o.unchanged())&&n.related.iter().all(|s|s.unchanged()))}
 fn view(&self,profile:Option<&str>,review:Option<&ReconciliationRef>,resource:Option<&str>)->io::Result<HistoryView>{
  let mut observed=vec![];let mut covered=vec![];let mut scope_uncertain=vec![];
  for(index,node)in self.nodes.iter().enumerate(){
   if node.slot.component=="shortcuts"&&node.intent.registry.evidence.pointer("/review/schemaVersion")==Some(&serde_json::json!(3)){
    scope_uncertain.extend(self.nodes[index+1..].iter().filter(|n|n.slot.phase!=Phase::NotDispatched).map(|n|n.slot.clone()));
   }
  }
  for(index,node)in self.nodes.iter().enumerate(){
   let Some(raw)=node.outcome.as_ref().and_then(|o|o.registry.evidence.get("archivedObservation"))else{
    if node.slot.phase==Phase::ReconciledObservation&&node.intent.registry.evidence.pointer("/review/action").and_then(serde_json::Value::as_str).is_some_and(|a|matches!(a,"reconcileArchivedObservation"|"reconcileLegacyObservation")){return Err(error("componentObservationMissing"));}continue;
   };
   let value:HistoricalObservation=serde_json::from_value(raw.clone()).map_err(|_|error("componentObservation"))?;value.validate(&node.slot.binding)?;
   let older=self.nodes[index+1..].iter().find(|older|older.slot==value.subject).ok_or_else(||error("componentObservationSubject"))?;value.subject_plan(&older.intent.registry)?;
   if node.slot.phase!=Phase::ReconciledObservation
    ||node.intent.registry.evidence.pointer("/review/action").and_then(serde_json::Value::as_str)!=Some(value.action())
    ||value.legacy()&&node.intent.registry.evidence.get("legacyUncertaintyAcknowledged")!=Some(&serde_json::Value::Bool(true))
    ||node.intent.registry.evidence.get("historicalSubject")!=Some(&serde_json::to_value(&value.subject).map_err(|_|error("componentObservation"))?)
    ||!self.nodes[index+1..].iter().any(|older|older.slot==value.subject){return Err(error("componentObservationSubject"));}
   let eligible=value.native_plan.schema_version==2&&profile==Some(value.profile.as_str())&&review==value.data_review.as_ref()&&resource.is_none_or(|r|r==value.native_plan.resource);
   if eligible{covered.push(value.subject.clone());}
   observed.push(HistoricalProjection{effect_id:node.slot.id.clone(),reference:node.slot.outcome.clone().ok_or_else(||error("componentOutcome"))?,observation:value,current_for_profile:eligible});
  }
  Ok(HistoryView{unresolved:self.nodes.iter().skip(1).filter(|n|(!n.slot.phase.replaceable()||scope_uncertain.contains(&n.slot))&&!covered.contains(&n.slot)).map(|n|n.slot.clone()).collect(),observed})
 }
}
pub(super)fn shortcut_prior_artifacts(path:&Path,slot:Option<&Slot>)->io::Result<Vec<(Slot,Artifact)>>{
 Ok(Chain::capture(path,slot)?.nodes.into_iter().map(|n|(n.slot,n.intent.registry)).collect())
}
pub(super)fn shortcut_sources(path:&Path,slot:Option<&Slot>)->io::Result<Vec<Snapshot<Artifact>>>{
 let chain=Chain::capture(path,slot)?;let mut seen=std::collections::BTreeSet::new();let mut held=vec![];
 for node in chain.nodes{for source in std::iter::once(node.intent).chain(node.outcome).chain(node.related){
  if seen.insert(source.registry.id.clone()){if seen.len()>128{return Err(error("componentReferenceLimit"));}held.push(source);}
 }}Ok(held)
}
impl Observation{
 pub(super)fn component_history(&self,path:&Path,component:&str)->io::Result<HistoryView>{self.component_history_for(path,component,None)}
 pub(super)fn component_history_for(&self,path:&Path,component:&str,resource:Option<&str>)->io::Result<HistoryView>{
  let scope=self.format.as_ref().map(|f|f.scope().map_err(|_|error("profile"))).transpose()?;
  // No admitted Format exists only in explicitly synthetic Rust fixtures.
  let scope=scope.as_deref().or_else(||self.component(component).map(|s|s.binding.profile.as_str()));
  Chain::capture(path,self.component(component))?.view(scope,self.snapshot.registry.reconciliation.as_ref(),resource)
 }
 fn historical_observation(&self,path:&Path,content:Option<&str>,slot:Slot,guard:impl Fn()->Result<(),&'static str>)->io::Result<Self>{
  if self.phase(path,content)!=crate::settings::IoPhase::Ready||slot.component!="autostart"||slot.phase!=Phase::ReconciledObservation{return Err(error("componentObservationAdmission"));}
  slot.validate().map_err(error)?;
  
  if self.format.as_ref().is_some_and(|f|!f.scope().is_ok_and(|s|s==slot.binding.profile)){return Err(error("componentProfileChanged"));}
  let chain=Chain::capture(path,self.component("autostart"))?;let old_view=self.component_history_for(path,"autostart",Some(&slot.binding.resource))?;
  let intent=observe(path,&slot.intent)?;let outcome=observe(path,slot.outcome.as_ref().ok_or_else(||error("componentOutcome"))?)?;
  if !matches(&intent.registry,&slot,Phase::Prepared)||intent.registry.previous.as_ref()!=self.component("autostart")||!matches(&outcome.registry,&slot,Phase::ReconciledObservation){return Err(error("componentObservationPlan"));}
  let claim:HistoricalObservation=serde_json::from_value(outcome.registry.evidence.get("archivedObservation").cloned().ok_or_else(||error("componentObservation"))?).map_err(|_|error("componentObservation"))?;claim.validate(&slot.binding)?;
  let selected=chain.nodes.iter().find(|n|n.slot==claim.subject).ok_or_else(||error("componentObservationSubject"))?;claim.subject_plan(&selected.intent.registry)?;
  if let Some(old)=self.component("autostart"){if !old.phase.replaceable()&&!(claim.legacy()&&old==&claim.subject)&&(legacy_slot(path,old)?||!old.independent_autostart_destination(&slot.binding.profile,&slot.binding.resource)){return Err(error("componentCurrentReconciliationRequired"));}}
  if !(old_view.unresolved.contains(&claim.subject)||claim.legacy()&&self.component("autostart")==Some(&claim.subject))||claim.data_review.as_ref()!=self.snapshot.registry.reconciliation.as_ref()
   ||intent.registry.evidence.pointer("/review/action").and_then(serde_json::Value::as_str)!=Some(claim.action())
   ||claim.legacy()&&intent.registry.evidence.get("legacyUncertaintyAcknowledged")!=Some(&serde_json::Value::Bool(true))
   ||intent.registry.evidence.get("historicalSubject")!=Some(&serde_json::to_value(&claim.subject).map_err(|_|error("componentObservation"))?){return Err(error("componentObservationSubject"));}
  guard().map_err(error)?;crate::pet_data::bootstrap::settings_io_component_protocol(path,true).map_err(|_|error("profile"))?;
  let format=crate::pet_data::bootstrap::settings_io_observation(path).map_err(|_|error("profile"))?;let profile=slot.binding.profile.clone();
  let mut record=self.snapshot.registry.clone();record.version=record.version.max(6);if !record.suspended_operations.iter().any(|s|s=="autostart"){record.suspended_operations.push("autostart".into());}record.components.insert("autostart".into(),slot);
  let snapshot=private_registry::save_guarded(&self.path,&self.snapshot.revision,record,|_,_|Ok(()),||{
   guard()?;if self.snapshot.unchanged()&&self.history.as_ref().is_none_or(|h|h.unchanged())&&self.components.iter().all(|h|h.unchanged())&&chain.current()&&intent.unchanged()&&outcome.unchanged()
    &&format.as_ref().is_none_or(|f|f.current()&&f.scope().is_ok_and(|s|s==profile)){Ok(())}else{Err("componentObservationChanged")}
  }).map_err(error)?;Self::from_snapshot(path,snapshot)
 }
}
/// Missing or legacy native provenance cannot establish an independent resource.
pub(crate)fn legacy_slot(path:&Path,slot:&Slot)->io::Result<bool>{let intent=observe(path,&slot.intent)?;if !matches(&intent.registry,slot,Phase::Prepared){return Err(error("componentIntent"));}Ok(intent.registry.evidence.pointer("/nativePlan/schemaVersion").and_then(serde_json::Value::as_u64)!=Some(2))}
pub(crate)fn blocks_resource(path:&Path,slot:&Slot,resource:&str)->io::Result<bool>{Ok(slot.binding.resource==resource||slot.component=="autostart"&&legacy_slot(path,slot)?)}
pub(crate)fn history_for_resource(settings:&crate::settings::Settings,path:&Path,component:&str,resource:&str)->io::Result<HistoryView>{crate::settings::revision::component_read(settings,path,|io,_|io.component_history_for(path,component,Some(resource)))}
pub(crate)fn legacy_subject(settings:&crate::settings::Settings,path:&Path,id:&str)->io::Result<(Slot,Artifact)>{
 if !hex(id,32){return Err(error("componentObservationSubject"));}
 crate::settings::revision::component_read(settings,path,|io,_|{let chain=Chain::capture(path,io.component("autostart"))?;let n=chain.nodes.iter().find(|n|n.slot.id==id&&!n.slot.phase.replaceable()).ok_or_else(||error("componentObservationSubject"))?;let plan:View=serde_json::from_value(n.intent.registry.evidence.get("nativePlan").cloned().ok_or_else(||error("componentLegacyPlan"))?).map_err(|_|error("componentLegacyPlan"))?;if plan.schema_version!=1||plan.resource!=n.slot.binding.resource||plan.fingerprint!=n.slot.binding.plan{return Err(error("componentLegacyPlan"));}Ok((n.slot.clone(),n.intent.registry.clone()))})
}
pub(crate)fn history_view(settings:&crate::settings::Settings,path:&Path,component:&str)->io::Result<HistoryView>{
 crate::settings::revision::component_read(settings,path,|io,_|io.component_history(path,component))
}
pub(crate)fn historical_subject(settings:&crate::settings::Settings,path:&Path,id:&str)->io::Result<(Slot,Artifact)>{
 if !hex(id,32){return Err(error("componentObservationSubject"));}
 let subject=history_view(settings,path,"autostart")?.unresolved.into_iter().find(|s|s.id==id).ok_or_else(||error("componentObservationSubject"))?;
 let intent=observe(path,&subject.intent)?;Ok((subject,intent.registry))
}
pub(crate)fn historical_witness(settings:&crate::settings::Settings,path:&Path,subject:&Slot)->io::Result<HistoricalWitness>{
 crate::settings::revision::component_read(settings,path,|io,_|{
  let chain=Chain::capture(path,io.component("autostart"))?;if subject.phase.replaceable()||!chain.nodes.iter().any(|n|&n.slot==subject){return Err(error("componentObservationSubject"));}
  Ok(HistoricalWitness{chain})
 })
}
pub(crate)fn record_historical_observation(settings:&crate::settings::Settings,path:&Path,slot:Slot,guard:impl Fn()->Result<(),&'static str>)->io::Result<()>{
 crate::settings::revision::component_change(settings,path,|io,witness|io.historical_observation(path,witness.content_hash(),slot,||{guard()?;if witness.current(){Ok(())}else{Err("componentSettingsChanged")}}))
}
