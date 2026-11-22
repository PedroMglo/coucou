//! Cached aggregate of the already installed station-management inventory.
//! It never replaces the control/approval client and never invokes usage or actions.
use std::{sync::{Mutex,OnceLock},time::{Instant,SystemTime,UNIX_EPOCH}};
use tauri::{AppHandle,Emitter};
#[path="station_history_core.rs"]mod core;
fn cache()->&'static Mutex<core::Cache>{static STATE:OnceLock<Mutex<core::Cache>>=OnceLock::new();STATE.get_or_init(Mutex::default)}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()}
fn allowed(label:&str)->bool{[crate::island::WINDOW_LABEL,crate::HUB_LABEL,crate::companions::LABEL,crate::pet_chat::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL].contains(&label)}
struct Completion {ticket:Option<u64>,app:AppHandle}
impl Completion {
 fn finish(&mut self,result:Result<core::Collection,&'static str>){
  if let Some(ticket)=self.ticket.take(){
   cache().lock().unwrap_or_else(|p|p.into_inner()).finish(ticket,result,Instant::now());
   let _=self.app.emit("pet-station-history-changed",serde_json::json!({"schemaVersion":1}));
  }
 }
}
impl Drop for Completion {fn drop(&mut self){self.finish(Err("collectionFailed"));}}
#[tauri::command]
pub async fn pet_station_history(window:tauri::WebviewWindow,app:AppHandle,force:Option<bool>)->Result<core::Snapshot,String>{
 if !allowed(window.label()){return Err("Abre um pet para consultar o histórico da estação.".into());}
 let ticket=cache().lock().unwrap_or_else(|p|p.into_inner()).begin(Instant::now(),force.unwrap_or(false));
 if let Some(ticket)=ticket{
  // Construct before spawn, and retain the physical helper's gate in this
  // blocking worker until execute has terminated and reaped its own child.
  let mut completion=Completion{ticket:Some(ticket),app};
  tauri::async_runtime::spawn_blocking(move||{
   let result=super::execute(vec!["list".into()])
    .map_err(|_|"collectionFailed")
    .and_then(|v|core::collect(&v,now()).map_err(|_|"invalidInventory"));
   completion.finish(result);
  });
 }
 Ok(cache().lock().unwrap_or_else(|p|p.into_inner()).snapshot(now()))
}
