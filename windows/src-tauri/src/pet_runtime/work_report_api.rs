//! Data-only preparation of a separate native reporting task. Sending still
//! requires the normal own admission, lease and exact message/data review.
use super::{model,messages::ReceivedReference,work_report};
use serde::{Serialize,Deserialize};
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Input{#[serde(default)]pub autonomous:bool,pub pet_id:String,pub goal_id:String,pub revision:String,pub source:ReceivedReference,pub expires_at:u64,#[serde(default)]pub include_attempts:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Draft{#[serde(skip_serializing_if="Option::is_none")]pub future_disclosure:Option<super::work_disclosure::Arguments>,pub schema_version:u32,pub pet_id:String,pub subject_goal_id:String,pub subject_revision:String,pub source:ReceivedReference,pub to_pet_id:String,pub expires_at:u64,pub include_attempts:bool,pub intent:String,pub criteria:Vec<String>,pub steps:Vec<model::StepInput>,pub preview:work_report::Report,pub execution_authorized:bool,pub budget_reserved:bool}
#[cfg(target_os="linux")]
pub(super)fn prepare(goals:&std::path::Path,authority:&std::path::Path,input:Input,now:u64)->model::Result<Draft>{
 let store=super::store::Store::existing_verified(goals,authority)?.ok_or(model::Error::NotFound)?;
 let v=store.received_work(authority,&super::received_work::Input{pet_id:input.pet_id.clone(),goal_id:input.goal_id.clone(),revision:input.revision.clone(),source:input.source.clone()},now)?;
 let args=work_report::Arguments{to_pet_id:v.source.sender_pet_id,subject_goal_id:input.goal_id.clone(),subject_revision:input.revision.clone(),source:input.source.clone(),expires_at:input.expires_at,include_attempts:input.include_attempts};
 let capture=work_report::Captured::at(goals,authority,&args,&input.pet_id,now)?;
 let future=if input.autonomous{let subject=store.read(&input.goal_id,&input.pet_id)?;Some(super::work_disclosure::Arguments{to_pet_id:args.to_pet_id.clone(),subject_goal_id:input.goal_id.clone(),subject_spec_hash:model::fingerprint(&subject.spec)?,source:input.source.clone(),expires_at:input.expires_at,include_attempts:input.include_attempts,states:vec![super::received_work::State::Completed,super::received_work::State::Cancelled]})}else{None};
 let operation=if let Some(f)=&future{super::plans::OperationProposal{operation_id:"pet.workReport".into(),version:2,arguments:serde_json::to_value(f).map_err(|_|model::Error::InvalidRecord)?}}else{super::plans::OperationProposal{operation_id:"pet.workReport".into(),version:1,arguments:serde_json::to_value(args.clone()).map_err(|_|model::Error::InvalidRecord)?}};
 Ok(Draft{future_disclosure:future,schema_version:1,pet_id:input.pet_id,subject_goal_id:input.goal_id,subject_revision:input.revision,source:input.source.clone(),to_pet_id:args.to_pet_id.clone(),expires_at:args.expires_at,include_attempts:args.include_attempts,intent:format!("Partilhar estado observado do pedido {}",input.source.message_id),criteria:vec![super::criteria::NATIVE_MESSAGE_CRITERION.into()],
  steps:vec![model::StepInput{description:"Partilhar metadados de estado revistos com o remetente certificado".into(),postcondition:"Relatório local recebido; não concede execução nem prova estado atual na receção".into(),operation:Some(operation)}],preview:capture.report,execution_authorized:false,budget_reserved:false})
}
#[tauri::command]
pub async fn pet_runtime_work_report_draft(window:tauri::WebviewWindow,input:Input)->model::Result<Draft>{
 if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
 #[cfg(not(target_os="linux"))]{let _=input;Err(model::Error::UnsupportedPlatform)}
 #[cfg(target_os="linux")]
 tauri::async_runtime::spawn_blocking(move||prepare(&crate::settings::config_dir().join("pet-runtime-v1"),&super::admission_review::authority_root(),input,crate::assistant_memory::now())).await.map_err(|_|model::Error::StorageUnavailable)?
}
