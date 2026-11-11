//! Per-pet System grants in the existing authoritative profile. No inference,
//! service IO, scheduler or additional execution-authority ledger.
use super::*;
use ring::rand::{SecureRandom,SystemRandom};
use std::sync::atomic::{AtomicBool,Ordering};
use tauri::Manager;

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Subplugin{Sound,Health,Inventory}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Operation{Observe,Volume,Mute}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Resource{AudioOutputs{},AudioOutput{key:String},HostHealth{},HostComponents{}}
pub(crate)fn output_key(key:&str)->bool{key.strip_prefix("audio-").is_some_and(|s|s.len()==16&&s.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f')))}
impl Resource{
 fn valid(&self)->bool{match self{Self::AudioOutputs{}|Self::HostHealth{}|Self::HostComponents{}=>true,Self::AudioOutput{key}=>output_key(key)}}
 fn contains(&self,key:&str)->bool{output_key(key)&&match self{Self::AudioOutputs{}=>true,Self::AudioOutput{key:expected}=>key==expected,Self::HostHealth{}|Self::HostComponents{}=>false}}
}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Grant{pub subplugin:Subplugin,pub operation:Operation,pub resource:Resource}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Policy{pub schema_version:u32,pub enabled:bool,pub revision:String,pub grants:Vec<Grant>}
fn grants_valid(grants:&[Grant])->bool{
 let mut unique=std::collections::BTreeSet::new();grants.len()<=32&&grants.iter().all(|g|
  g.resource.valid()&&unique.insert(g)&&
  match (&g.subplugin,&g.resource,g.operation){(Subplugin::Sound,Resource::AudioOutputs{}|Resource::AudioOutput{..},_)=>true,(Subplugin::Health,Resource::HostHealth{},Operation::Observe)|(Subplugin::Inventory,Resource::HostComponents{},Operation::Observe)=>true,_=>false}&&
  (g.operation==Operation::Observe||grants.iter().any(|r|r.subplugin==g.subplugin&&r.operation==Operation::Observe&&r.resource==g.resource)))
}
impl Policy{
 pub(crate)fn permits_inventory(&self)->bool{self.enabled&&self.valid()&&self.grants.iter().any(|g|g.subplugin==Subplugin::Inventory&&g.operation==Operation::Observe&&matches!(g.resource,Resource::HostComponents{}))}
 pub(crate)fn permits_health(&self)->bool{self.enabled&&self.valid()&&self.grants.iter().any(|g|g.subplugin==Subplugin::Health&&g.operation==Operation::Observe&&matches!(g.resource,Resource::HostHealth{}))}
 pub(super)fn valid(&self)->bool{self.schema_version==1&&crate::pet_inference::revision_key(&self.revision)&&grants_valid(&self.grants)}
 pub(crate)fn permits(&self,op:Operation,key:Option<&str>)->bool{self.enabled&&self.valid()&&self.grants.iter().any(|g|g.subplugin==Subplugin::Sound&&g.operation==op&&key.is_none_or(|key|g.resource.contains(key)))}
}
#[derive(Clone,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Change{pub pet_id:String,pub expected_revision:String,pub grants:Vec<Grant>}
fn set_at(file:&Path,request:&Change,current:impl Fn()->bool)->Result<Document,String>{
 if !current()||!grants_valid(&request.grants){return Err("Revê o pet, as operações e os recursos da concessão de Sistema.".into());}
 let original=read_at_policy(file,false)?;
 if original.revision!=request.expected_revision{return Err("O perfil mudou. Recarrega antes de rever o acesso ao Sistema.".into());}
 require_active(&request.pet_id)?;if blueprint(&request.pet_id).is_none_or(|b|b.runtime.kind==RuntimeKind::Observer){return Err("Este pet ainda não tem um contrato operacional de Sistema.".into());}
 let mut config=original.config.clone();let pet=config.pets.get_mut(&request.pet_id).ok_or("Pet desconhecido.")?;
 let mut nonce=[0u8;16];SystemRandom::new().fill(&mut nonce).map_err(|_|"Não foi possível criar uma revisão de Sistema.")?;
 let policy=Policy{schema_version:1,enabled:true,revision:nonce.iter().map(|b|format!("{b:02x}")).collect(),grants:request.grants.clone()};
 pet.system_access=Some(policy.clone());config.version=2;
 write_guarded(file,&config,||current()&&bytes(file).is_ok_and(|data|revision(&data)==original.revision))?;
 let result=read_at_policy(file,false)?;
 if result.config.pets.get(&request.pet_id).and_then(|p|p.system_access.as_ref())!=Some(&policy){return Err("A gravação pode ter ocorrido mas o perfil mudou antes da confirmação. Recarrega antes de repetir.".into());}
 Ok(result)
}
/// Common addressed NativeUser gate; model text cannot construct this origin.
pub(crate)fn addressed(window:&tauri::WebviewWindow,app:&AppHandle,pet_id:&str)->Result<(),String>{
 if !allowed(window.label())||!window.is_visible().unwrap_or(false){return Err("Abre a gestão nativa deste pet para consultar o Sistema.".into());}
 require_active(pet_id)?;let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().map_err(|_|"As preferências estão indisponíveis.")?;
 if settings.main_pill!=pet_id&&!settings.active_integrations.iter().any(|p|p==pet_id){return Err("Este pet não está ativo.".into());}Ok(())
}
fn allowed(label:&str)->bool{[crate::HUB_LABEL,crate::companions::LABEL,crate::pet_panel::LABEL,crate::pet_control::LABEL].contains(&label)}
#[tauri::command]
pub async fn pet_system_access_set(app:AppHandle,window:tauri::WebviewWindow,request:Change)->Result<Document,String>{
 if !allowed(window.label()){return Err("Revê as concessões na gestão nativa do pet.".into());}
 let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"A revisão nativa de Sistema terminou.")?;
 tauri::async_runtime::spawn_blocking(move||{
  crate::settings::effect_allowed()?;let _edit=EDIT.lock().map_err(|_|"O perfil dos pets está ocupado.")?;
  let doc=set_at(&path(),&request,||frame.current())?;publish_resource_snapshot(&doc.config);let _=app.emit("pet-configuration-changed",&doc.config);Ok(doc)
 }).await.map_err(|_|"A gravação terminou sem confirmação. Recarrega antes de repetir.".to_string())?
}

/// Backend-created immutable scope. A PetId supplied in model text is not a
/// caller identity; only trusted native/adapter entry points construct this.
pub(crate)struct Captured{pet_id:String,subplugin:Subplugin,policy:Policy,operation:Operation,key:Option<String>,retired:std::sync::Arc<AtomicBool>,#[cfg(test)]file:Option<PathBuf>}
impl Captured{
 fn from_config(config:&Configuration,pet_id:&str,operation:Operation,key:Option<&str>)->Result<Self,String>{Self::from_config_for(config,pet_id,Subplugin::Sound,operation,key)}
 fn from_config_for(config:&Configuration,pet_id:&str,subplugin:Subplugin,operation:Operation,key:Option<&str>)->Result<Self,String>{
  require_active(pet_id)?;if blueprint(pet_id).is_none_or(|b|b.runtime.kind==RuntimeKind::Observer){return Err("Este pet ainda não tem um contrato operacional de Sistema.".into());}let policy=config.pets.get(pet_id).and_then(|p|p.system_access.clone()).ok_or("Este pet não tem concessão própria para esta operação de Sistema.")?;
  if config.version!=2||match subplugin{Subplugin::Sound=>!policy.permits(operation,key)||operation!=Operation::Observe&&key.is_none(),Subplugin::Health=>!policy.permits_health()||operation!=Operation::Observe||key.is_some(),Subplugin::Inventory=>!policy.permits_inventory()||operation!=Operation::Observe||key.is_some()}{return Err("A concessão não inclui esta operação e recurso de Sistema.".into());}
  Ok(Self{pet_id:pet_id.into(),subplugin,policy,operation,key:key.map(str::to_owned),retired:std::sync::Arc::new(AtomicBool::new(false)),#[cfg(test)]file:None})
 }
 pub(crate)fn capture_inventory(pet_id:&str)->Result<Self,String>{Self::capture_inventory_reviewed(&read_existing()?,pet_id)}
 pub(crate)fn capture_inventory_reviewed(config:&Configuration,pet_id:&str)->Result<Self,String>{let c=Self::from_config_for(config,pet_id,Subplugin::Inventory,Operation::Observe,None)?;c.check()?;Ok(c)}
 pub(crate)fn capture_health(pet_id:&str)->Result<Self,String>{Self::capture_health_reviewed(&read_existing()?,pet_id)}
 pub(crate)fn capture_health_reviewed(config:&Configuration,pet_id:&str)->Result<Self,String>{let c=Self::from_config_for(config,pet_id,Subplugin::Health,Operation::Observe,None)?;c.check()?;Ok(c)}
 pub(crate)fn capture(pet_id:&str,operation:Operation,key:Option<&str>)->Result<Self,String>{Self::from_config(&read_existing()?,pet_id,operation,key)}
 /// The native recognition input is already captured. Do not replace its policy
 /// with a newer revision between recognition and the first service query.
 pub(crate)fn capture_reviewed(config:&Configuration,pet_id:&str,operation:Operation,key:Option<&str>)->Result<Self,String>{let scope=Self::from_config(config,pet_id,operation,key)?;scope.check()?;Ok(scope)}
 pub(crate)fn revision(&self)->&str{&self.policy.revision}
 pub(crate)fn has_operation(&self,operation:Operation)->bool{self.subplugin==Subplugin::Sound&&self.policy.permits(operation,None)}
 /// Narrow discovery to one observed physical output under the SAME revision.
 /// Aliases share sticky invalidation; this never reloads or renews a policy.
 pub(crate)fn pin_output(&self,operation:Operation,key:&str)->Result<Self,String>{
  self.check()?;
  if self.subplugin!=Subplugin::Sound||self.operation!=Operation::Observe||self.key.is_some()||!self.permits_output(key)||!self.policy.permits(operation,Some(key)){return Err("A concessão original não inclui esta operação e saída de Som.".into());}
  Ok(Self{pet_id:self.pet_id.clone(),subplugin:self.subplugin,policy:self.policy.clone(),operation,key:Some(key.into()),retired:self.retired.clone(),#[cfg(test)]file:self.file.clone()})
 }
 pub(crate)fn permits_output(&self,key:&str)->bool{self.subplugin==Subplugin::Sound&&self.policy.permits(Operation::Observe,Some(key))&&self.policy.permits(self.operation,Some(key))&&self.key.as_deref().is_none_or(|expected|expected==key)}
 pub(crate)fn check(&self)->Result<(),String>{
  let denied=||"A concessão original de Sistema foi retirada ou mudou. O pedido não foi redirecionado.".to_string();
  if self.retired.load(Ordering::Acquire){return Err(denied());}
  #[cfg(test)]let config=if let Some(file)=&self.file{read_existing_at(file)}else{read_existing()};
  #[cfg(not(test))]let config=read_existing();
  let valid=config.is_ok_and(|c|c.pets.get(&self.pet_id).and_then(|p|p.system_access.as_ref())==Some(&self.policy)&&match self.subplugin{Subplugin::Sound=>self.policy.permits(self.operation,self.key.as_deref()),Subplugin::Health=>self.policy.permits_health(),Subplugin::Inventory=>self.policy.permits_inventory()});
  if !valid{self.retired.store(true,Ordering::Release);return Err(denied());}Ok(())
 }
 #[cfg(test)]pub(crate)fn fixture_inventory(file:&Path,pet_id:&str)->Result<Self,String>{let mut c=Self::from_config_for(&read_existing_at(file)?,pet_id,Subplugin::Inventory,Operation::Observe,None)?;c.file=Some(file.into());Ok(c)}
 #[cfg(test)]pub(crate)fn fixture_health(file:&Path,pet_id:&str)->Result<Self,String>{let mut c=Self::from_config_for(&read_existing_at(file)?,pet_id,Subplugin::Health,Operation::Observe,None)?;c.file=Some(file.into());Ok(c)}
 #[cfg(test)]pub(crate)fn fixture(file:&Path,pet_id:&str,operation:Operation,key:Option<&str>)->Result<Self,String>{let mut capture=Self::from_config(&read_existing_at(file)?,pet_id,operation,key)?;capture.file=Some(file.into());Ok(capture)}
}

#[cfg(test)]#[path="pet_system_access_tests.rs"]mod tests;
