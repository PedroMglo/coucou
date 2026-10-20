//! Tauri cache/worker integration. Collection is the pure local metadata reader.
use super::process_census::{self,Census,Cache,PROVIDERS,summarize,stamp};
use std::{collections::BTreeMap,time::{Duration,Instant}};
#[cfg(target_os="linux")]
fn cache()->&'static std::sync::Mutex<Cache>{static CACHE:std::sync::OnceLock<std::sync::Mutex<Cache>>=std::sync::OnceLock::new();CACHE.get_or_init(Default::default)}
#[cfg(target_os="linux")]
struct Completion(tauri::AppHandle,bool);
#[cfg(target_os="linux")]
impl Drop for Completion {fn drop(&mut self){if self.1{{let mut slot=cache().lock().unwrap_or_else(|p|p.into_inner());slot.inflight=false;for row in slot.rows.values_mut(){row.stale=true;row.refreshing=false;row.reason=Some("processRefreshFailed".into());}}use tauri::Emitter;let _=self.0.emit("pet-observability-changed",serde_json::json!({"schemaVersion":1,"source":"localProcesses"}));}}}
/// One shared scan, not one process traversal per card. Physical single flight
/// remains owned by the blocking collector until it finishes, including on Drop.
#[cfg(target_os="linux")]
pub(crate) fn snapshots(app:&tauri::AppHandle)->BTreeMap<String,Census>{
 let mut slot=cache().lock().unwrap_or_else(|p|p.into_inner());let now=Instant::now();
 if !slot.inflight&&slot.last.is_none_or(|last|now.saturating_duration_since(last)>=Duration::from_secs(5)){
  slot.inflight=true;slot.last=Some(now);let mut completion=Completion(app.clone(),true);
  tauri::async_runtime::spawn_blocking(move||{
   let result=process_census::collect_now(&crate::platform::home_dir());
   {let mut slot=cache().lock().unwrap_or_else(|p|p.into_inner());
    slot.finish(result);
   }completion.1=false;use tauri::Emitter;let _=completion.0.emit("pet-observability-changed",serde_json::json!({"schemaVersion":1,"source":"localProcesses"}));
  });
 }
 PROVIDERS.iter().map(|(p,_)|{let mut row=slot.rows.get(*p).cloned().unwrap_or_else(||summarize(p,vec![],"unavailable",None,Some("processesNotObserved".into())));row.refreshing=slot.inflight;if row.observed_at.is_some_and(|t|stamp().saturating_sub(t)>15){row.stale=true;}((*p).into(),row)}).collect()
}
#[cfg(not(target_os="linux"))]
pub(crate) fn snapshots(_: &tauri::AppHandle)->BTreeMap<String,Census>{PROVIDERS.iter().map(|(p,_)|((*p).into(),summarize(p,vec![],"notSupported",None,Some("unsupportedPlatform".into())))).collect()}
