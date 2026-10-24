use super::model::*;
use super::Repository;
use serde::{Deserialize,Serialize};
use tauri::{AppHandle,Manager};
pub(super)fn allowed(label:&str)->bool{[crate::HUB_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL].contains(&label)}
fn window_check(w:&tauri::WebviewWindow)->Result<()>{if allowed(w.label()){Ok(())}else{Err(Error::InvalidInput)}}
fn pet(id:&str)->Result<()>{if crate::pet_config::defaults().pets.contains_key(id){Ok(())}else{Err(Error::InvalidInput)}}
fn owner(app:&AppHandle,id:&str)->Result<Owner>{
    let snapshot=crate::pet_capabilities::passive_snapshot(app).map_err(|_|Error::StorageUnavailable)?;
    let p=snapshot.pets.into_iter().find(|p|p.pet_id==id).ok_or(Error::InvalidInput)?;
    Owner::from_descriptor(&p)
}
fn root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-v1")}
#[cfg(target_os="linux")]fn existing()->Result<impl Repository>{super::store::Store::existing_native(&root())?.ok_or(Error::NotFound)}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CreateInput{pub pet_id:String,pub request_id:String,pub intent:String,pub criteria:Vec<String>,pub conversation_id:Option<String>}
#[derive(Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum DraftAction{ProposeTask{description:String},ProposePlan{#[serde(rename="taskId")]task_id:String,steps:Vec<StepInput>},Pause,ResumeDraft,Cancel}
impl From<DraftAction> for Action{fn from(a:DraftAction)->Self{match a{DraftAction::ProposeTask{description}=>Self::ProposeTask{description},DraftAction::ProposePlan{task_id,steps}=>Self::ProposePlan{task_id,steps},DraftAction::Pause=>Self::Pause,DraftAction::ResumeDraft=>Self::ResumeDraft,DraftAction::Cancel=>Self::Cancel}}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MutationInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub action:DraftAction}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ImportInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub task_id:String}

#[tauri::command]
pub async fn pet_runtime_create(app:AppHandle,window:tauri::WebviewWindow,input:CreateInput)->Result<Receipt>{
    window_check(&window)?;
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let owner=owner(&app,&input.pet_id)?;
        let context=if let Some(id)=input.conversation_id{
            if !app.state::<crate::chat::Chat>().has_public_conversation(&owner.provider,&id){return Err(Error::Conflict);}Context::Mochi{conversation_id:id}
        }else{Context::Unbound};
        let spec=Spec{owner,context,request_id:input.request_id,intent:input.intent,criteria:input.criteria};spec.validate()?;
        Repository::create(&super::store::Store::ensure_native(&root())?,spec,crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}
#[tauri::command]
pub async fn pet_runtime_list(window:tauri::WebviewWindow,pet_id:Option<String>)->Result<Listing>{
    window_check(&window)?;if let Some(id)=&pet_id{pet(id)?;}
    #[cfg(not(target_os="linux"))]{let _=pet_id;Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||match super::store::Store::existing_native(&root())?{
        Some(store)=>Repository::list(&store,pet_id.as_deref()),None=>Ok(Listing{schema_version:1,goals:vec![],unavailable:0,pending_artifacts:0,complete:true})
    }).await.map_err(|_|Error::StorageUnavailable)?
}
#[tauri::command]
pub async fn pet_runtime_read(window:tauri::WebviewWindow,pet_id:String,goal_id:String)->Result<Record>{
    window_check(&window)?;pet(&pet_id)?;
    #[cfg(not(target_os="linux"))]{let _=(pet_id,goal_id);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||existing()?.read(&goal_id,&pet_id)).await.map_err(|_|Error::StorageUnavailable)?
}
#[tauri::command]
pub async fn pet_runtime_mutate(app:AppHandle,window:tauri::WebviewWindow,input:MutationInput)->Result<Receipt>{
    window_check(&window)?;pet(&input.pet_id)?;
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let store=existing()?;let action:Action=input.action.into();
        if !matches!(action,Action::Cancel|Action::Pause){
            let current=owner(&app,&input.pet_id)?;let record=store.read(&input.goal_id,&input.pet_id)?;
            if record.spec.owner.provider!=current.provider||record.spec.owner.adapter!=current.adapter{return Err(Error::Conflict);}
        }
        super::authority_store::mutate_with_authority(&store,&super::admission_review::authority_root(),&input.goal_id,&input.pet_id,&input.revision,input.request_id,action,crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}
#[tauri::command]
pub async fn pet_runtime_import_legacy(app:AppHandle,window:tauri::WebviewWindow,input:ImportInput)->Result<Receipt>{
    window_check(&window)?;pet(&input.pet_id)?;
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let store=existing()?;let record=store.read(&input.goal_id,&input.pet_id)?;
        let mut matches=app.state::<crate::chat::Chat>().tasks().into_iter().filter(|t|t.id==input.task_id&&t.provider==record.spec.owner.provider&&t.valid()&&match &record.spec.context{Context::Mochi{conversation_id}=>t.conversation_id.as_ref()==Some(conversation_id),Context::Unbound=>true});
        let task=matches.next().ok_or(Error::NotFound)?;if matches.next().is_some(){return Err(Error::Conflict);}
        let reference=Legacy{task_id:task.id,provider:task.provider,backend:task.backend,conversation_id:task.conversation_id,phase:task.phase,intent_may_be_incomplete:true,receipts:task.local_execution};
        store.mutate(&input.goal_id,&input.pet_id,&input.revision,input.request_id,Action::ImportLegacy{reference},crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PrepareContextInput{pub pet_id:String,pub goal_id:String,pub revision:String,pub request_id:String,pub mode:PreparationMode}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ContextView{pub binding:ContextBinding,pub messages:Vec<serde_json::Value>,pub message_count:usize,pub truncated:bool}
fn context_root()->std::path::PathBuf{crate::settings::config_dir().join("pet-runtime-contexts-v1")}
#[tauri::command]
pub async fn pet_runtime_prepare_context(app:AppHandle,window:tauri::WebviewWindow,input:PrepareContextInput)->Result<Receipt>{
    window_check(&window)?;pet(&input.pet_id)?;
    #[cfg(not(target_os="linux"))]{let _=(app,input);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let store=existing()?;let goal=store.read(&input.goal_id,&input.pet_id)?;
        if let Some(binding)=&goal.execution_context{
            if binding.request_id!=input.request_id||binding.mode!=input.mode{return Err(Error::Conflict);}
            return store.mutate(&goal.id,&input.pet_id,&input.revision,input.request_id,Action::BindContext{binding:binding.clone()},crate::assistant_memory::now());
        }
        if goal.execution_context.is_none(){let current=owner(&app,&input.pet_id)?;if current.provider!=goal.spec.owner.provider||current.adapter!=goal.spec.owner.adapter{return Err(Error::Conflict);}}
        let main=app.state::<crate::chat::Chat>();
        let(binding,_chat)=super::context::prepare(&context_root(),&goal,&main,input.mode,&input.request_id)?;
        store.mutate(&goal.id,&input.pet_id,&input.revision,input.request_id,Action::BindContext{binding},crate::assistant_memory::now())
    }).await.map_err(|_|Error::StorageUnavailable)?
}
#[tauri::command]
pub async fn pet_runtime_context_read(app:AppHandle,window:tauri::WebviewWindow,pet_id:String,goal_id:String)->Result<ContextView>{
    window_check(&window)?;pet(&pet_id)?;
    #[cfg(not(target_os="linux"))]{let _=(app,pet_id,goal_id);Err(Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn_blocking(move||{
        let goal=existing()?.read(&goal_id,&pet_id)?;let main=app.state::<crate::chat::Chat>();
        let(binding,chat)=super::context::open(&context_root(),&goal,&main)?;
        let snapshot=chat.snapshot_public_context(&binding.provider,&binding.conversation_id).map_err(|_|Error::Conflict)?;
        Ok(ContextView{binding,truncated:snapshot.messages.len()<snapshot.source_messages,message_count:snapshot.source_messages,messages:snapshot.messages})
    }).await.map_err(|_|Error::StorageUnavailable)?
}
