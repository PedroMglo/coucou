//! Native driver references only; no serialized controller, approval or model.
use super::model;
use serde::Deserialize;
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DriverInput {pub pet_id:String,pub goal_id:String,pub lease_id:String}
#[cfg(target_os="linux")]use super::{lease_controller::dispatcher::{Driver,Status},store::Store};
#[cfg(target_os="linux")]use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
#[cfg(target_os="linux")]fn drivers()->&'static Mutex<BTreeMap<String,Arc<Driver>>>{static DRIVERS:OnceLock<Mutex<BTreeMap<String,Arc<Driver>>>>=OnceLock::new();DRIVERS.get_or_init(||Mutex::new(BTreeMap::new()))}
fn check(window:&tauri::WebviewWindow)->model::Result<()>{if super::adapter::allowed(window.label()){Ok(())}else{Err(model::Error::InvalidInput)}}
#[cfg(target_os="linux")]fn lookup(input:&DriverInput)->model::Result<Arc<Driver>>{
    let cache=drivers().lock().map_err(|_|model::Error::StorageUnavailable)?;
    let driver=cache.get(&input.lease_id).cloned().ok_or(model::Error::NotFound)?;let status=driver.status()?;
    if status.pet_id!=input.pet_id||status.goal_id!=input.goal_id{return Err(model::Error::Conflict);}Ok(driver)
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_start_driver(app:tauri::AppHandle,window:tauri::WebviewWindow,input:DriverInput)->model::Result<Status>{
    check(&window)?;tauri::async_runtime::spawn_blocking(move||{
        let retained=drivers().lock().map_err(|_|model::Error::StorageUnavailable)?.get(&input.lease_id).cloned();
        if let Some(driver)=retained{
            let status=driver.status()?;
            if status.pet_id!=input.pet_id||status.goal_id!=input.goal_id{return Err(model::Error::Conflict);}
            if status.retained_closure{if !status.active{driver.spawn()?;}return driver.status();}
        }
        let store=Arc::new(Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(model::Error::NotFound)?);
        let goal=store.read_control(&input.goal_id,&input.pet_id)?;let owner=super::lease_api::step_owner(&goal,&input.lease_id)?;
        let driver={let mut cache=drivers().lock().map_err(|_|model::Error::StorageUnavailable)?;
            if let Some(driver)=cache.get(&input.lease_id){driver.clone()}else {
                cache.retain(|_,d|!d.removable());if cache.len()>=64{return Err(model::Error::LimitReached);}
                let driver=Driver::for_app(owner,store,app)?;cache.insert(input.lease_id.clone(),driver.clone());driver
            }};
        let status=driver.status()?;if status.pet_id!=input.pet_id||status.goal_id!=input.goal_id{return Err(model::Error::Conflict);}
        if !status.active{driver.spawn()?;}driver.status()
    }).await.map_err(|_|model::Error::StorageUnavailable)?
}
#[cfg(target_os="linux")]#[tauri::command]
pub async fn pet_runtime_driver_status(window:tauri::WebviewWindow,input:DriverInput)->model::Result<Status>{check(&window)?;tauri::async_runtime::spawn_blocking(move||lookup(&input)?.status()).await.map_err(|_|model::Error::StorageUnavailable)?}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_start_driver(app:tauri::AppHandle,window:tauri::WebviewWindow,input:DriverInput)->model::Result<serde_json::Value>{check(&window)?;let _=(app,input);Err(model::Error::UnsupportedPlatform)}
#[cfg(not(target_os="linux"))]#[tauri::command]
pub async fn pet_runtime_driver_status(window:tauri::WebviewWindow,input:DriverInput)->model::Result<serde_json::Value>{check(&window)?;let _=input;Err(model::Error::UnsupportedPlatform)}
