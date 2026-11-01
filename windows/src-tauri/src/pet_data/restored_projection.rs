//! A reviewed initial configuration, not a new permission ledger. Canonical
//! settings/pets/source writers retain ownership after native Admission2.
use super::{Error,Result};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Choices{pub schema_version:u32,pub inspection_receipt:String,pub main_pet:String,pub active_pets:Vec<String>,pub grants:Vec<Grant>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Grant{pub pet_id:String,pub operations:Vec<Operation>}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Operation{Documents,EmailDrafts,Music,OpenApps}
pub(crate)struct Projection{pub settings:Vec<u8>,pub pets:Vec<u8>,pub sources:Option<Vec<u8>>,pub flow:Option<Vec<u8>>,pub settings_io:Option<Vec<u8>>,pub differences:Value}
fn value<T:Serialize>(v:T)->Result<Value>{serde_json::to_value(v).map_err(|_|Error::UnsupportedProfile)}
fn bytes<T:Serialize>(v:T)->Result<Vec<u8>>{serde_json::to_vec(&v).map_err(|_|Error::UnsupportedProfile)}
impl Choices{
 pub(crate)fn parse(b:&[u8])->Result<Self>{if b.len()>16*1024{return Err(Error::Limit);}let c:Self=serde_json::from_slice(b).map_err(|_|Error::UnsupportedProfile)?;c.validate()?;Ok(c)}
 fn validate(&self)->Result<()>{
  if self.schema_version!=1||self.inspection_receipt.len()!=64||!self.inspection_receipt.bytes().all(|b|b.is_ascii_hexdigit())||self.active_pets.is_empty()||self.active_pets.len()>128||self.grants.len()>128{return Err(Error::UnsupportedProfile);}
  let active=self.active_pets.iter().collect::<BTreeSet<_>>();if active.len()!=self.active_pets.len()||!active.contains(&self.main_pet){return Err(Error::UnsupportedProfile);}
  let mut grants=BTreeSet::new();for g in &self.grants{if !active.contains(&g.pet_id)||!grants.insert(&g.pet_id)||g.operations.iter().collect::<BTreeSet<_>>().len()!=g.operations.len(){return Err(Error::UnsupportedProfile);}}
  for id in active{if crate::pet_config::blueprint(id).is_none_or(|b|b.retired){return Err(Error::UnsupportedProfile);}}
  Ok(())
 }
 pub(crate)fn canonical(&self)->Result<Vec<u8>>{self.validate()?;bytes(self)}
}
pub(crate)fn project(report:&Value,choice:&Choices,sources:Option<&[u8]>,flow:Option<&[u8]>)->Result<Projection>{
 choice.validate()?;let groups=report["settings"]["resolvedGroups"].as_object().ok_or(Error::UnsupportedProfile)?;if groups.contains_key("requiresClassification"){return Err(Error::UnsupportedProfile);}
 let mut setting=serde_json::Map::new();for group in groups.values(){for(k,v)in group.as_object().ok_or(Error::UnsupportedProfile)?{if setting.insert(k.clone(),v.clone()).is_some(){return Err(Error::UnsupportedProfile);}}}
 let mut settings:crate::settings::Settings=serde_json::from_value(Value::Object(setting.clone())).map_err(|_|Error::UnsupportedProfile)?;
 let mut pets:crate::pet_config::Configuration=serde_json::from_value(report["pets"]["resolved"].clone()).map_err(|_|Error::UnsupportedProfile)?;
 for id in &choice.active_pets{if !pets.pets.contains_key(id){return Err(Error::UnsupportedProfile);}}
 let grants:BTreeMap<_,_>=choice.grants.iter().map(|g|(g.pet_id.as_str(),g.operations.iter().copied().collect::<BTreeSet<_>>())).collect();
 for(id,p)in &mut pets.pets{
  let permission=|op|grants.get(id.as_str()).is_some_and(|ops|ops.contains(&op));p.permissions=Some(crate::pet_config::Permissions{documents:permission(Operation::Documents),email_drafts:permission(Operation::EmailDrafts),music:permission(Operation::Music),open_apps:permission(Operation::OpenApps)});
  // Shell policy needs its own command review. Never inherit an Ask/full-access
  // mode or executable allowlist from a copied preference.
  if p.local_execution.is_some(){p.local_execution=Some(crate::pet_config::LocalExecution::default());}
 }
 crate::pet_config::validate(&pets).map_err(|_|Error::UnsupportedProfile)?;
 settings.main_pill=choice.main_pet.clone();settings.active_integrations=choice.active_pets.iter().filter(|id|*id!=&choice.main_pet).cloned().collect();
 settings.autostart=false;settings.hooks_installed=false;settings.plan_relay_installed=false;settings.cli_full_access=false;settings.spotify_quiet_ads=false;settings.sound_enabled=false;
 settings.phone_link=false;settings.phone_chat=false;settings.phone_chat_models.clear();settings.phone_details=false;settings.phone_answers=false;settings.phone_diffs=false;settings.phone_usage=false;settings.phone_services.clear();settings.phone_relay=false;
 settings.pet_system_center=false;settings.shortcuts=crate::shortcuts::ACTIONS.iter().map(|a|(a.id.into(),crate::shortcuts::Binding{keys:String::new(),enabled:false})).collect();
 // Preserve chosen inference identities/models/endpoints as reviewed DATA.
 // Entry admission checks canonical active pet choices before inference; these
 // strings alone grant no access and do not discover/authenticate/load a model.
 let main=crate::pet_config::blueprint(&choice.main_pet).ok_or(Error::UnsupportedProfile)?;if let Some(route)=&main.routing.chat{settings.chat_provider=route.clone();}
 let sources=sources.map(crate::pet_sources::suspend_for_restoration).transpose().map_err(|_|Error::UnsupportedProfile)?;
 let flow=flow.map(crate::flow::suspend_for_restoration).transpose().map_err(|_|Error::UnsupportedProfile)?;
 let new_settings=value(&settings)?;let mut changed=vec![];for(k,v)in new_settings.as_object().ok_or(Error::UnsupportedProfile)?{if setting.get(k)!=Some(v){changed.push(json!({"field":k,"before":setting.get(k),"after":v}));}}
 let differences=json!({"settings":changed,"petGrants":choice.grants,"localCommandsEnabled":false,"persistentSourcesSuspended":sources.is_some(),"flowAndVoiceSuspended":flow.is_some(),"pendingFeatureRevalidation":["voice","mobile","autostart"],"inferenceFallbackAllowed":false});
 Ok(Projection{settings:bytes(settings)?,pets:bytes(pets)?,sources,flow,settings_io:None,differences})
}
