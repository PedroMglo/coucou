//! Native field mutation only. This never dispatches inference or grants tools.
use super::*;
use ring::rand::{SecureRandom,SystemRandom};
#[derive(Clone,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Update{Set{choice:InferenceChoice},Clear{}}
#[derive(Clone,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Change{pub pet_id:String,pub expected_revision:String,pub update:Update}
fn next_revision()->Result<String,String>{
 let mut bytes=[0u8;16];SystemRandom::new().fill(&mut bytes).map_err(|_|"Não foi possível criar uma revisão de inferência.")?;
 Ok(bytes.iter().map(|b|format!("{b:02x}")).collect())
}
fn set_at(file:&Path,request:&Change,current:impl Fn()->bool)->Result<Document,String>{
 if !current(){return Err("A revisão humana do motor deixou de estar ativa.".into());}
 let original=read_at_policy(file,false)?;
 if original.revision!=request.expected_revision{return Err("O perfil mudou entretanto. Recarrega-o e revê novamente o motor.".into());}
 let blueprint=blueprint(&request.pet_id).filter(|b|!b.retired&&!b.runtime.inference_adapters.is_empty()).ok_or("Este pet não tem um contrato de inferência configurável.")?;
 let choice=match &request.update{Update::Set{choice}=>{choice.validate(blueprint)?;Some(choice.clone())},Update::Clear{}=>None};
 let mut config=original.config.clone();let pet=config.pets.get_mut(&request.pet_id).ok_or("O pet já não existe.")?;
 if choice.is_none()&&pet.inference.is_none()&&pet.inference_revision.is_none(){return if current(){Ok(original)}else{Err("A revisão humana foi retirada.".into())};}
 pet.inference=choice;pet.inference_revision=Some(next_revision()?);config.version=2;
 // Reapplying an identical choice also retires old captures. Clear retains a
 // fresh revision, so A -> clear -> A and legacy -> set -> clear cannot be ABA.
 write_guarded(file,&config,||current()&&bytes(file).is_ok_and(|data|revision(&data)==original.revision))?;
 let result=read_at_policy(file,false)?;
 let expected=&config.pets[&request.pet_id];let observed=&result.config.pets[&request.pet_id];
 if observed.inference_revision!=expected.inference_revision||observed.inference!=expected.inference{return Err("A alteração foi gravada mas o perfil mudou antes da confirmação. Recarrega antes de repetir.".into());}
 Ok(result)
}
fn allowed(label:&str)->bool{[crate::HUB_LABEL,crate::companions::LABEL,crate::pet_panel::LABEL,crate::pet_control::LABEL].contains(&label)}
#[tauri::command]
pub async fn pet_inference_set(app:AppHandle,window:tauri::WebviewWindow,request:Change)->Result<Document,String>{
 if !allowed(window.label()){return Err("Revê o motor na gestão nativa do pet.".into());}
 let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"A gestão nativa deixou de estar disponível. Nenhum motor foi alterado.")?;
 tauri::async_runtime::spawn_blocking(move||{
  crate::settings::effect_allowed()?;
  let _edit=EDIT.lock().map_err(|_|"O perfil dos pets está ocupado.")?;
  let doc=set_at(&path(),&request,||frame.current())?;
  publish_resource_snapshot(&doc.config);let _=app.emit("pet-configuration-changed",&doc.config);Ok(doc)
 }).await.map_err(|_|"A gravação terminou sem confirmação. Recarrega o perfil antes de repetir.".to_string())?
}
#[cfg(test)]#[path="pet_inference_config_tests.rs"]mod tests;
