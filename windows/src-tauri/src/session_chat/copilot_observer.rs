//! Bounded, expiring observers. Closing an observer never closes its parent CLI.
use std::{collections::HashMap,sync::{Mutex,OnceLock,atomic::{AtomicU64,Ordering}},time::{Duration,Instant}};
use crate::session_chat::types::Target;
struct Entry{generation:u64,touched:Instant,task:Option<tokio::task::AbortHandle>}
fn watchers()->&'static Mutex<HashMap<Target,Entry>>{static STATE:OnceLock<Mutex<HashMap<Target,Entry>>>=OnceLock::new();STATE.get_or_init(Mutex::default)}
static GENERATION:AtomicU64=AtomicU64::new(0);
const IDLE:Duration=Duration::from_secs(90);
fn remove(target:&Target,generation:u64){let mut entries=watchers().lock().unwrap();if entries.get(target).is_some_and(|e|e.generation==generation){entries.remove(target);}}
pub(super)fn observe(app:&tauri::AppHandle,target:&Target){
 let generation={
  let mut entries=watchers().lock().unwrap();let now=Instant::now();
  if let Some(entry)=entries.get_mut(target){entry.touched=now;return;}
  if entries.len()>=16{if let Some(old)=entries.iter().min_by_key(|(_,e)|e.touched).map(|(k,_)|k.clone()){
   if let Some(entry)=entries.remove(&old){if let Some(task)=entry.task{task.abort();}}
  }}
  let generation=GENERATION.fetch_add(1,Ordering::Relaxed);entries.insert(target.clone(),Entry{generation,touched:now,task:None});generation
 };
 let app=app.clone();let target=target.clone();let key=target.clone();
 let task=tauri::async_runtime::spawn(async move{
  let mut subscription=match crate::copilot_session::subscribe(&target).await{Ok(s)=>s,Err(_)=>{remove(&target,generation);return;}};
  loop{
   let live=watchers().lock().unwrap().get(&target).is_some_and(|e|e.generation==generation&&e.touched.elapsed()<IDLE);
   if !live{break;}
   match tokio::time::timeout(Duration::from_secs(15),subscription.recv()).await{
    Ok(Some(updated))=>{if updated==target{super::updated(&app,&target);}},
    Ok(None)=>{super::updated(&app,&target);break;},Err(_)=>{},
   }
  }
  remove(&target,generation);
 });
 let mut entries=watchers().lock().unwrap();
 if let Some(entry)=entries.get_mut(&key).filter(|e|e.generation==generation){entry.task=Some(task.inner().abort_handle());}else{task.abort();}
}
