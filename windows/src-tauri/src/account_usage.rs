//! Explicit Codex account-activity reader. Cache queries do not start a client.
use std::sync::{Arc,Mutex};
use std::time::{Duration,Instant};
use tauri::{AppHandle,Emitter,Manager,State,WebviewWindow};
use crate::account_usage_core::{Cache,Failure,RefreshError,Snapshot,Summary};

pub struct Runtime{cache:Arc<Mutex<Cache>>}
impl Default for Runtime {
 fn default()->Self{
  use ring::rand::SecureRandom;
  let mut bytes=[0u8;16];
  // Once at native Runtime construction, never from a cached query. This is
  // a non-secret producer incarnation, not an account or authorization token.
  let instance=ring::rand::SystemRandom::new().fill(&mut bytes).ok()
   .map(|()|bytes.iter().map(|b|format!("{b:02x}")).collect::<String>());
  Self{cache:Arc::new(Mutex::new(Cache::with_instance(instance)))}
 }
}
const CHANGED:&str="pet-account-usage-changed";
fn publish(app:&AppHandle,snapshot:&Snapshot){
 if let Some(changed)=snapshot.changed(){let _=app.emit(CHANGED,changed);}
}
fn caller(window:&WebviewWindow)->Result<(),RefreshError>{
 if [crate::island::WINDOW_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,
     crate::pet_chat::LABEL,crate::pet_panel::LABEL,crate::HUB_LABEL].contains(&window.label()) {Ok(())}
 else {Err(RefreshError::Unavailable)}
}
fn clock()->u64{crate::pet_observability::usage::now_seconds().unwrap_or(0)}

#[tauri::command]
pub fn pet_account_usage(window:WebviewWindow,runtime:State<Runtime>)->Result<Snapshot,RefreshError>{
 caller(&window)?;
 let cache=runtime.cache.lock().map_err(|_|RefreshError::Unavailable)?;
 Ok(cache.snapshot(clock()))
}
#[tauri::command]
pub async fn pet_account_usage_refresh(window:WebviewWindow,app:AppHandle)->Result<Snapshot,RefreshError>{
 caller(&window)?;
 let shared=app.state::<Runtime>().cache.clone();
 let ticket=begin_refresh(&shared,Instant::now(),|snapshot|publish(&app,snapshot))?;
 // The worker owns completion. Dropping a UI promise cannot free the slot
 // while its child process or bounded protocol IO is still in flight.
 tauri::async_runtime::spawn_blocking(move||refresh_worker(&shared,ticket,read_native,|snapshot|publish(&app,snapshot)))
  .await.map_err(|_|RefreshError::Unavailable)?
}
fn begin_refresh(shared:&Mutex<Cache>,now:Instant,publish:impl FnOnce(&Snapshot))->Result<u64,RefreshError>{
 let(ticket,snapshot)={
  let mut cache=shared.lock().map_err(|_|RefreshError::Unavailable)?;
  let ticket=cache.begin(now)?;(ticket,cache.snapshot(clock()))
 };
 notify(&snapshot,publish);Ok(ticket)
}
fn notify(snapshot:&Snapshot,publish:impl FnOnce(&Snapshot)){
 // Change events are best-effort invalidation hints. A publisher failure must
 // not strand admitted work or suppress its already committed completion ACK.
 let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||publish(snapshot)));
}
fn refresh_worker(shared:&Mutex<Cache>,ticket:u64,read:impl FnOnce()->Result<Summary,Failure>,publish:impl FnOnce(&Snapshot))->Result<Snapshot,RefreshError>{
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(read)).unwrap_or(Err(Failure::NativeUnavailable));
 let at=clock();let snapshot={
  let mut cache=shared.lock().map_err(|_|RefreshError::Unavailable)?;
  if !cache.finish(ticket,result,at,Instant::now()){return Err(RefreshError::Unavailable);}
  cache.snapshot(at)
 };
 notify(&snapshot,publish);Ok(snapshot)
}

#[cfg(not(target_os="linux"))]
fn read_native()->Result<Summary,Failure>{Err(Failure::UnsupportedPlatform)}
#[cfg(target_os="linux")]
fn read_native()->Result<Summary,Failure>{
 let exe=crate::platform::codex_candidates().into_iter().next().ok_or(Failure::NativeUnavailable)?;
 read_from(&exe,&crate::platform::home_dir())
}
#[cfg(target_os="linux")]
fn read_from(exe:&std::path::Path,cwd:&std::path::Path)->Result<Summary,Failure>{
 read_from_budget(exe,cwd,Duration::from_secs(8))
}
#[cfg(target_os="linux")]
fn read_from_budget(exe:&std::path::Path,cwd:&std::path::Path,budget:Duration)->Result<Summary,Failure>{
 let mut client=crate::codex_chat::rpc::Client::start_with_cancel(exe,cwd,
   budget,false,Arc::default()).map_err(|_|Failure::NativeUnavailable)?;
 client.require_account().map_err(|e|if e==crate::codex_chat::rpc::SIGN_IN{Failure::NotSignedIn}else{Failure::NativeUnavailable})?;
 let value=client.request("account/usage/read",serde_json::json!({})).map_err(|_|Failure::NativeUnavailable)?;
 crate::account_usage_core::parse(&value)
}

#[cfg(all(test,target_os="linux"))]
#[path="account_usage_native_tests.rs"]
mod tests;
