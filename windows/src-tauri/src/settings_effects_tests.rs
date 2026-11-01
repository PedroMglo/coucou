use super::*;
use crate::settings::{load_from,save_to,update_with};
use std::{fs,io::{Read,Write},net::TcpListener,path::PathBuf,sync::{atomic::{AtomicBool,AtomicUsize,Ordering},mpsc},time::{Duration,Instant}};
use std::os::unix::fs::PermissionsExt;
use serde_json::{json,Value};

pub(crate) struct Fixture {pub(crate) root:PathBuf,pub(crate) path:PathBuf,pub(crate) current:Arc<Mutex<Settings>>,pub(crate) probe:EffectProbe}
impl Fixture {
 pub(crate) fn new()->Self{
  static NEXT:AtomicUsize=AtomicUsize::new(0);
  let root=std::env::temp_dir().join(format!("coucou-settings-effect-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
  fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();
  let path=root.join("settings.json");let mut settings=Settings::default();settings.model="explicit-fixture-model".into();
  fs::write(&path,serde_json::to_vec(&settings).unwrap()).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();
  let current=Arc::new(Mutex::new(load_from(&path)));let probe=effect_probe(&current);Self{root,path,current,probe}
 }
 pub(crate) fn external(&self){let mut value:Value=serde_json::from_slice(&fs::read(&self.path).unwrap()).unwrap();value["soundVolume"]=json!(0.7123);fs::write(&self.path,serde_json::to_vec(&value).unwrap()).unwrap();}
 fn chat(&self)->crate::chat::Chat{crate::chat::Chat::persistent(self.root.join("chat.json")).with_effect_probe(self.probe.clone())}
 pub(crate) fn save(&self){update_with(&self.current,|next|next.sound_volume=0.45,|next|save_to(&self.path,next),|_|{}).unwrap();}
 pub(crate) fn update(&self,change:impl FnOnce(&mut Settings)){update_with(&self.current,change,|next|save_to(&self.path,next),|_|{}).unwrap();}
 // Real data-review writer, exposed only by the existing cfg(test) fixture.
 // Callers outside Settings do not gain access to private production APIs.
 pub(crate) fn review_data(&self){
  use crate::settings::durable;static NEXT:AtomicUsize=AtomicUsize::new(0);let id=format!("{:032x}",NEXT.fetch_add(1,Ordering::Relaxed));
  let(source,original)=crate::private_registry::preferences_file::read(&self.path,false).unwrap();let(journal,old_io)=crate::private_registry::preferences_file::read(&self.root.join(durable::FILE),false).unwrap();let bytes=original.as_deref().unwrap();
  let reference=durable::history::retain(&self.path,durable::history::Input{id:&id,fingerprint:&"a".repeat(64),original_settings:original.as_deref(),original_io:old_io.as_deref(),approved:bytes,memory_hash:&"b".repeat(64),configuration_revision:"fixture",configuration_values:&json!({}),observed_phase:"Ready"},||Ok(())).unwrap();
  let prepared=durable::review_prepare(&self.path,&journal,old_io.as_deref(),source.content_hash(),bytes,reference,||Ok(())).unwrap();source.commit_guarded(bytes,||if prepared.current(){Ok(())}else{Err(std::io::Error::other("changed"))}).unwrap();prepared.finish_reviewed().unwrap();*self.current.lock().unwrap()=load_from(&self.path);
 }
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}

#[test]fn settings_effect_observes_external_change_without_exposing_content_or_reloading(){
 let f=Fixture::new();assert_eq!(f.probe.status().phase,IoPhase::Ready);f.external();
 assert_eq!(f.probe.status().phase,IoPhase::Changed);assert!(f.probe.check().is_err());
 assert_eq!(f.current.lock().unwrap().model,"explicit-fixture-model");
 let public=serde_json::to_string(&f.probe.status()).unwrap();assert!(!public.contains(f.path.to_str().unwrap()));assert!(!public.contains("explicit-fixture-model"));
}
#[test]fn settings_effect_current_owner_survives_benign_commit_without_using_captured_old_revision(){
 let f=Fixture::new();let captured=f.current.lock().unwrap().clone();f.save();
 assert_eq!(io_status(&captured).phase,IoPhase::Changed);assert_eq!(f.probe.status().phase,IoPhase::Ready);
 assert_eq!(captured.model,f.current.lock().unwrap().model);assert_ne!(captured.sound_volume,0.45);
}
#[test]fn settings_effect_writing_is_nonblocking_backpressure_and_dropped_owner_is_unavailable(){
 let f=Fixture::new();let held=f.current.lock().unwrap();assert_eq!(f.probe.status().phase,IoPhase::Writing);assert!(f.probe.check().is_err());drop(held);
 assert_eq!(f.probe.status().phase,IoPhase::Ready);let owner=Arc::new(Mutex::new(Settings::default()));let weak=effect_probe(&owner);assert_eq!(weak.status().phase,IoPhase::Unbound);drop(owner);assert_eq!(weak.status().phase,IoPhase::Unavailable);
}
#[test]fn settings_effect_poisoned_owner_is_not_ready(){
 let owner=Arc::new(Mutex::new(Settings::default()));let weak=effect_probe(&owner);let other=owner.clone();
 assert!(std::thread::spawn(move||{let _held=other.lock().unwrap();panic!("synthetic writer panic");}).join().is_err());assert_eq!(weak.status().phase,IoPhase::Unavailable);
}
#[test]fn settings_effect_partial_parse_never_becomes_default_inference_authority_after_save(){
 let f=Fixture::new();fs::write(&f.path,br#"{"model":"chosen-fixture","soundVolume":"invalid"}"#).unwrap();
 let current=Arc::new(Mutex::new(load_from(&f.path)));let probe=effect_probe(&current);
 assert_eq!(probe.status().phase,IoPhase::ReadUnavailable);assert!(probe.check().is_err());
 update_with(&current,|next|next.sound_volume=0.3,|next|save_to(&f.path,next),|_|{}).unwrap();
 assert_eq!(probe.status().phase,IoPhase::ReviewRequired);assert!(probe.check().is_err());
 // No durable reconciliation or post-restart acceptance is claimed here.
}
#[test]fn settings_effect_postpublication_uncertainty_reaches_current_owner_and_never_announces_success(){
 let f=Fixture::new();let published=AtomicUsize::new(0);
 let result=update_with(&f.current,|next|next.sound_volume=0.33,
  |next|crate::settings::revision::uncertain_save_for_test(&f.path,next,||f.external()),
  |_|{published.fetch_add(1,Ordering::Relaxed);});
 assert!(result.is_err());assert_eq!(published.load(Ordering::Relaxed),0);
 assert_eq!(f.probe.status().phase,IoPhase::PublicationUnconfirmed);assert!(f.probe.check().is_err());
 let bytes=fs::read(&f.path).unwrap();assert!(update_with(&f.current,|next|next.autostart=true,|next|save_to(&f.path,next),|_|panic!("uncertain source published")).is_err());assert_eq!(fs::read(&f.path).unwrap(),bytes);
}
#[test]fn settings_effect_stop_and_history_remain_available_after_source_change(){
 let f=Fixture::new();let chat=f.chat();let turn=chat.begin_cli("custom",Arc::default()).unwrap();f.external();
 assert!(chat.check_new_effect().is_err());assert!(chat.history("custom").is_empty());chat.cancel_provider("custom");assert!(!chat.is_current(&turn));
}

// A bounded loopback wire fixture. Only parsed request bodies are retained;
// no headers, actual credentials, accounts, models or external destinations.
struct Wire{url:String,requests:Arc<Mutex<Vec<Value>>>,received:mpsc::Receiver<()>,release:Arc<AtomicBool>,stop:Arc<AtomicBool>,thread:Option<std::thread::JoinHandle<()>>}
impl Wire{
 fn new(hold:bool)->Self{
  let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let url=format!("http://{}",listener.local_addr().unwrap());
  let requests=Arc::new(Mutex::new(Vec::new()));let release=Arc::new(AtomicBool::new(!hold));let stop=Arc::new(AtomicBool::new(false));let(tx,received)=mpsc::channel();
  let(rows,go,end)=(requests.clone(),release.clone(),stop.clone());
  let thread=std::thread::spawn(move||while !end.load(Ordering::Acquire){
   let(mut socket,_)=match listener.accept(){Ok(v)=>v,Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{std::thread::sleep(Duration::from_millis(3));continue;},Err(_)=>break};
   socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();socket.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
   let mut bytes=Vec::new();let mut chunk=[0;4096];let deadline=Instant::now()+Duration::from_secs(3);
   let body=loop{
    assert!(bytes.len()<262_144&&Instant::now()<deadline,"fixture request exceeded bounds");let n=socket.read(&mut chunk).unwrap();assert!(n>0);bytes.extend_from_slice(&chunk[..n]);
    if let Some(at)=bytes.windows(4).position(|w|w==b"\r\n\r\n"){
     let head=std::str::from_utf8(&bytes[..at]).unwrap();let length=head.lines().find_map(|l|l.to_ascii_lowercase().strip_prefix("content-length:").map(|v|v.trim().parse::<usize>().unwrap())).unwrap();
     if bytes.len()>=at+4+length{break serde_json::from_slice::<Value>(&bytes[at+4..at+4+length]).unwrap();}
    }
   };
   let stream=body["stream"]==true;rows.lock().unwrap().push(body);tx.send(()).unwrap();
   while !go.load(Ordering::Acquire)&&!end.load(Ordering::Acquire)&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(3));}
   if end.load(Ordering::Acquire){break;}
   let answer=if stream{"data: {\"choices\":[{\"delta\":{\"content\":\"observed-fixture-answer\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n".to_string()}else{json!({"content":[{"type":"text","text":"observed-fixture-answer"}],"choices":[{"message":{"role":"assistant","content":"observed-fixture-answer"}}]}).to_string()};
   write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",if stream{"text/event-stream"}else{"application/json"},answer.len(),answer).unwrap();
  });Self{url,requests,received,release,stop,thread:Some(thread)}
 }
 async fn wait(&self){let deadline=Instant::now()+Duration::from_secs(3);loop{if self.received.try_recv().is_ok(){return;}assert!(Instant::now()<deadline,"no synthetic dispatch");tokio::time::sleep(Duration::from_millis(3)).await;}}
}
impl Drop for Wire{fn drop(&mut self){self.stop.store(true,Ordering::Release);self.release.store(true,Ordering::Release);if let Some(thread)=self.thread.take(){thread.join().unwrap();}}}
fn provider(url:&str)->&'static crate::openai_compat::Provider{Box::leak(Box::new(crate::openai_compat::Provider{id:"openrouter",name:"synthetic",base_url:Box::leak(url.to_string().into_boxed_str()),key:"synthetic-only",models_path:"models",default_model:"unused-fixture",not_chat:&[],max_tokens_field:"max_tokens"}))}
fn server(url:&str)->crate::local_chat::Server{crate::local_chat::Server{id:"custom",name:"synthetic",url:url.into(),key:None}}
#[tokio::test]async fn settings_effect_changed_source_blocks_all_three_http_transports_before_dispatch(){
 let f=Fixture::new();let wire=Wire::new(false);let chat=f.chat();f.external();let endpoint=reqwest::Url::parse(&wire.url).unwrap();
 assert!(crate::claude::memory_wire_fixture(&chat,"explicit-fixture","question".into(),None,&endpoint,"synthetic-only").await.is_err());
 assert!(crate::openai_compat::memory_wire_fixture(&chat,provider(&wire.url),"explicit-fixture","question".into(),None,"synthetic-only").await.is_err());
 assert!(crate::local_chat::send_with(&chat,&server(&wire.url),"explicit-fixture","question".into(),None,|_|{}).await.is_err());
 assert!(wire.requests.lock().unwrap().is_empty());assert!(chat.history("custom").is_empty());
}
#[tokio::test]async fn settings_effect_legitimate_commit_during_local_io_keeps_captured_model_and_reply(){
 let f=Fixture::new();let wire=Wire::new(true);let chat=f.chat();let server=server(&wire.url);
 let(result,())=tokio::join!(crate::local_chat::send_with(&chat,&server,"captured-fixture","question".into(),None,|_|{}),async{wire.wait().await;f.save();wire.release.store(true,Ordering::Release);});
 assert_eq!(result.unwrap().text,"observed-fixture-answer");assert_eq!(f.probe.status().phase,IoPhase::Ready);
 let requests=wire.requests.lock().unwrap();assert_eq!(requests.len(),1);assert_eq!(requests[0]["model"],"captured-fixture");assert_eq!(chat.history("custom").len(),2);
}
#[tokio::test]async fn settings_effect_external_change_preserves_observed_reply_but_blocks_next_request(){
 let f=Fixture::new();let wire=Wire::new(true);let chat=f.chat();let server=server(&wire.url);
 let(result,())=tokio::join!(crate::local_chat::send_with(&chat,&server,"captured-fixture","question".into(),None,|_|{}),async{wire.wait().await;f.external();wire.release.store(true,Ordering::Release);});
 assert_eq!(result.unwrap().text,"observed-fixture-answer");assert_eq!(chat.history("custom").len(),2);assert_eq!(f.probe.status().phase,IoPhase::Changed);
 assert!(crate::local_chat::send_with(&chat,&server,"captured-fixture","another question".into(),None,|_|{}).await.is_err());assert_eq!(wire.requests.lock().unwrap().len(),1);
}

#[test]fn settings_effect_captured_origin_survives_read_contention_but_not_external_change(){
    let f=Fixture::new();let origin=capture_origin(&f.current.lock().unwrap());
    let reader=f.current.lock().unwrap();assert_eq!(f.probe.status().phase,IoPhase::Writing);
    for name in ["voice","mobile","shortcuts","autostart","hooks","credentials"]{assert!(origin.operation(name).is_ok());}
    f.external();assert!(origin.operation("voice").is_err());drop(reader);
}
#[test]fn settings_effect_captured_origin_cannot_follow_a_replaced_owner_with_identical_bytes(){
    let f=Fixture::new();let origin=capture_origin(&f.current.lock().unwrap());let bytes=fs::read(&f.path).unwrap();
    *f.current.lock().unwrap()=load_from(&f.path);assert_eq!(fs::read(&f.path).unwrap(),bytes);
    assert!(origin.operation("voice").is_err());assert!(capture_origin(&f.current.lock().unwrap()).operation("voice").is_ok());
}
#[test]fn settings_effect_captured_origin_retains_component_suspensions_and_uncertainty(){
    let f=Fixture::new();let old=capture_origin(&f.current.lock().unwrap());f.review_data();assert!(old.operation("voice").is_err());
    let reviewed=capture_origin(&f.current.lock().unwrap());
    for name in ["voice","mobile","shortcuts","autostart","hooks","credentials"]{assert!(reviewed.operation(name).is_err());}
    let f=Fixture::new();let old=capture_origin(&f.current.lock().unwrap());crate::settings::revision::mark_unconfirmed(&f.current.lock().unwrap());assert!(old.operation("voice").is_err());
}
#[test]fn settings_effect_new_commit_never_renews_a_captured_origin(){
    let f=Fixture::new();let origin=capture_origin(&f.current.lock().unwrap());f.save();assert!(f.probe.operation("voice").is_ok());assert!(origin.operation("voice").is_err());
}

#[test]fn settings_effect_captured_origin_parallel_readers_do_not_create_false_writing(){
    let f=Fixture::new();let origin=Arc::new(capture_origin(&f.current.lock().unwrap()));let barrier=Arc::new(std::sync::Barrier::new(16));
    let reader=f.current.lock().unwrap();assert_eq!(f.probe.status().phase,IoPhase::Writing);
    std::thread::scope(|scope|{for _ in 0..16{let origin=origin.clone();let barrier=barrier.clone();scope.spawn(move||{barrier.wait();for _ in 0..32{assert!(origin.operation("voice").is_ok());}});}});
    drop(reader);f.external();assert!(origin.operation("voice").is_err());
}
