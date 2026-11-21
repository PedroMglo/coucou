//! Shared broker + real HTTP client, with private profiles and synthetic SSE.
//! No application, real inference, account, credential or host service.
use super::*;
use crate::{chat::Chat,pet_inference::{Backend,Choice,CredentialReference},settings::Settings};
use serde_json::{json,Value};
use std::{io::{Read,Write},net::TcpListener,path::PathBuf};
struct Fixture{root:PathBuf,profile:PathBuf,config:crate::pet_config::Configuration,settings:Arc<Mutex<Settings>>}
impl Fixture{
 fn new(endpoint:&str)->Self{
  let root=std::env::temp_dir().join(format!("coucou-inference-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  std::fs::create_dir(&root).unwrap();use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&root,std::fs::Permissions::from_mode(0o700)).unwrap();
  let mut config=crate::pet_config::defaults();config.version=2;
  for(id,model)in[("ai_ollama","first-fixture"),("ai_lmstudio","second-fixture")]{config.pets.get_mut(id).unwrap().inference=Some(Choice{schema_version:1,backend:Backend::Ollama,endpoint:endpoint.into(),model:model.into(),credential:CredentialReference::None});}
  config.pets.get_mut("ai_ollama").unwrap().inference_revision=Some("a".repeat(32));config.pets.get_mut("ai_lmstudio").unwrap().inference_revision=Some("b".repeat(32));
  crate::pet_config::validate(&config).unwrap();let profile=root.join("pets.json");
  let mut settings=Settings::default();settings.ollama_url="http://127.0.0.1:1".into();settings.lmstudio_url="http://127.0.0.1:2".into();
  let f=Self{root,profile,config,settings:Arc::new(Mutex::new(settings))};f.save();f
 }
 fn save(&self){std::fs::write(&self.profile,serde_json::to_vec(&self.config).unwrap()).unwrap();}
 fn coordinator(&self)->Arc<Coordinator>{Arc::new(Coordinator::with_fixture(self.settings.clone(),self.profile.clone()))}
}
impl Drop for Fixture{fn drop(&mut self){std::fs::remove_dir_all(&self.root).unwrap();}}
fn synthetic_server(count:usize)->(String,std::thread::JoinHandle<Vec<Value>>,Arc<std::sync::atomic::AtomicUsize>){
 let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let url=format!("http://{}",listener.local_addr().unwrap());
 let seen=Arc::new(std::sync::atomic::AtomicUsize::new(0));let accepted=seen.clone();
 let worker=std::thread::spawn(move||{
  let end=Instant::now()+Duration::from_secs(8);let mut requests=Vec::new();
  while requests.len()<count{
   let(mut socket,_)=loop{match listener.accept(){Ok(s)=>break s,Err(e)if e.kind()==std::io::ErrorKind::WouldBlock&&Instant::now()<end=>std::thread::sleep(Duration::from_millis(5)),Err(e)=>panic!("synthetic accept: {e}")}};
   accepted.fetch_add(1,Ordering::AcqRel);
   socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut bytes=Vec::new();let mut buffer=[0u8;4096];
   let split=loop{let n=socket.read(&mut buffer).unwrap();assert!(n>0&&bytes.len()+n<128*1024);bytes.extend_from_slice(&buffer[..n]);if let Some(i)=bytes.windows(4).position(|s|s==b"\r\n\r\n"){break i+4;}};
   let head=std::str::from_utf8(&bytes[..split]).unwrap();assert!(head.starts_with("POST /v1/chat/completions HTTP/1.1\r\n"));
   let length=head.lines().find_map(|l|{let(k,v)=l.split_once(':')?;k.eq_ignore_ascii_case("content-length").then(||v.trim().parse::<usize>().unwrap())}).unwrap();assert!(length<64*1024);
   while bytes.len()<split+length{let n=socket.read(&mut buffer).unwrap();assert!(n>0);bytes.extend_from_slice(&buffer[..n]);}
   requests.push(serde_json::from_slice(&bytes[split..split+length]).unwrap());
   let body="data: {\"choices\":[{\"delta\":{\"content\":\"synthetic reply\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
   write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
  }requests
 });(url,worker,seen)
}
#[tokio::test]
async fn pet_inference_two_logical_pets_use_one_captured_server_without_global_destination_or_shared_history(){
 let(url,worker,_)=synthetic_server(2);let f=Fixture::new(&url);let coordinator=f.coordinator();
 let chat=Chat::persistent(f.root.join("conversation.json")).with_runtime_resources(coordinator);
 for(provider,model)in[("ollama","first-fixture"),("lmstudio","second-fixture")]{
  let reservation=chat.reserve_conversation(provider,None).unwrap();let other=if provider=="ollama"{"lmstudio"}else{"ollama"};
  assert!(chat.reserve_conversation(other,None).is_err(),"one physical server despite distinct owners");
  let busy=chat.runtime_resources().busy_with_config(&f.config,&f.settings.lock().unwrap()).unwrap();assert!(busy.contains(provider)&&busy.contains(other));
  let legacy=crate::local_chat::Server{id:provider,name:"synthetic",url:"http://127.0.0.1:1".into(),key:None};
  let reply=crate::local_chat::send_with(&reservation.channel(),&legacy,model,format!("question for {provider}"),None,|_|{}).await.unwrap();assert_eq!(reply.text,"synthetic reply");
  assert!(chat.reserve_conversation(other,None).is_err(),"the original reservation still owns its physical hold");drop(reservation);
 }
 let requests=worker.join().unwrap();assert_eq!(requests.len(),2);assert_eq!(requests[0]["model"],"first-fixture");assert_eq!(requests[1]["model"],"second-fixture");
 assert!(requests[0]["messages"][0]["content"].as_str().unwrap().contains("Ollama"));assert!(requests[1]["messages"][0]["content"].as_str().unwrap().contains("LM Studio"));
 assert_eq!(requests[1]["messages"].as_array().unwrap().len(),2,"the first owner's history was not copied");
 assert_ne!(chat.conversation_id("ollama").unwrap(),chat.conversation_id("lmstudio").unwrap());
 assert_eq!(f.config.pets["ai_lmstudio"].permissions,crate::pet_config::defaults().pets["ai_lmstudio"].permissions);
 let restored=Chat::persistent(f.root.join("conversation.json"));assert_ne!(restored.conversation_id("ollama").unwrap(),restored.conversation_id("lmstudio").unwrap());
 assert_eq!(restored.history("ollama").len(),2);assert_eq!(restored.history("lmstudio").len(),2);
}
#[test]
fn pet_inference_queued_destination_is_not_redirected_and_cancelled_worker_keeps_physical_exclusion(){
 let mut f=Fixture::new("http://127.0.0.1:18471");let c=f.coordinator();let manual=c.acquire("ollama",Arc::default(),None).unwrap();
 let queued=c.request_goal("lmstudio",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();assert_eq!(queued.waiting_info().unwrap().position,1);
 f.config.pets.get_mut("ai_lmstudio").unwrap().inference.as_mut().unwrap().endpoint="http://127.0.0.1:18472".into();f.save();
 drop(manual);let worker=queued.poll().unwrap();let captured=worker.captured().unwrap();assert_eq!(captured.endpoint,"http://127.0.0.1:18471");assert_eq!(captured.adapter,"ollama");
 assert!(!captured.current());assert!(crate::local_chat::server_for_capture(&captured).is_err(),"changed binding refused before HTTP/credentials");
 queued.cancel();drop(queued);assert!(!worker.current());
 assert!(c.acquire("ollama",Arc::default(),None).is_err(),"old physical server still held");assert!(c.acquire("lmstudio",Arc::default(),None).is_err(),"same logical owner cannot escape via another server");
 drop(worker);let first=c.acquire("ollama",Arc::default(),None).unwrap();let second=c.acquire("lmstudio",Arc::default(),None).unwrap();assert!(first.current()&&second.current());
 assert_eq!(first.provider_for(&c),Some("ollama"));assert_eq!(second.provider_for(&c),Some("lmstudio"));
}

fn busy_reader(settings:Arc<Mutex<Settings>>)->(std::sync::mpsc::Sender<()>,std::thread::JoinHandle<()>){
 let(held,ready)=std::sync::mpsc::channel();let(release,done)=std::sync::mpsc::channel();
 let worker=std::thread::spawn(move||{let _reader=settings.lock().unwrap();held.send(()).unwrap();done.recv_timeout(Duration::from_secs(3)).unwrap();});
 ready.recv_timeout(Duration::from_secs(3)).unwrap();(release,worker)
}

#[tokio::test]
async fn interactive_inference_waits_for_readers_then_dispatches_the_original_target_once(){
 let(url,worker,seen)=synthetic_server(1);let f=Fixture::new(&url);let c=f.coordinator();let chat=Chat::default().with_runtime_resources(c);
 let reservation=chat.reserve_conversation("ollama",None).unwrap();let channel=reservation.channel();let(release,reader)=busy_reader(f.settings.clone());
 let legacy=crate::local_chat::Server{id:"ollama",name:"unused legacy address",url:"http://127.0.0.1:1".into(),key:None};
 let sending=crate::local_chat::send_with(&channel,&legacy,"first-fixture","synthetic reader wait".into(),None,|_|{});
 let unlock=async{tokio::time::sleep(Duration::from_millis(25)).await;assert_eq!(seen.load(Ordering::Acquire),0,"no HTTP while the captured settings cannot be checked");release.send(()).unwrap();};
 let(reply,())=tokio::join!(sending,unlock);reader.join().unwrap();assert_eq!(reply.unwrap().text,"synthetic reply");let requests=worker.join().unwrap();assert_eq!(seen.load(Ordering::Acquire),1);assert_eq!(requests.len(),1);assert_eq!(requests[0]["model"],"first-fixture");assert_eq!(chat.history("ollama").len(),2);
}

#[tokio::test]
async fn interactive_inference_stop_during_reader_wait_sends_nothing_and_retains_physical_exclusion(){
 let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let f=Fixture::new(&format!("http://{}",listener.local_addr().unwrap()));let c=f.coordinator();let chat=Chat::default().with_runtime_resources(c.clone());
 let reservation=chat.reserve_conversation("ollama",None).unwrap();let channel=reservation.channel();let(release,reader)=busy_reader(f.settings.clone());let legacy=crate::local_chat::Server{id:"ollama",name:"unused",url:"http://127.0.0.1:1".into(),key:None};
 let sending=crate::local_chat::send_with(&channel,&legacy,"first-fixture","cancel while reading".into(),None,|_|{});
 let stop=async{tokio::time::sleep(Duration::from_millis(25)).await;chat.cancel_provider("ollama");};
 let(reply,())=tokio::join!(sending,stop);assert_eq!(reply.err().unwrap(),crate::net::CANCELLED);assert!(matches!(listener.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));assert!(chat.history("ollama").is_empty());
 release.send(()).unwrap();reader.join().unwrap();assert!(matches!(c.acquire("lmstudio",Arc::default(),None),Err(Failure::Busy)));drop(reservation);assert!(c.acquire("lmstudio",Arc::default(),None).is_ok());
}

#[tokio::test]
async fn interactive_inference_binding_change_during_reader_wait_never_dispatches_or_redirects(){
 let first=TcpListener::bind("127.0.0.1:0").unwrap();let second=TcpListener::bind("127.0.0.1:0").unwrap();first.set_nonblocking(true).unwrap();second.set_nonblocking(true).unwrap();let original=format!("http://{}",first.local_addr().unwrap());let mut f=Fixture::new(&original);let old=f.config.clone();let chat=Chat::default().with_runtime_resources(f.coordinator());let reservation=chat.reserve_conversation("ollama",None).unwrap();let channel=reservation.channel();let captured=channel.captured_inference("ollama").unwrap().unwrap();let(release,reader)=busy_reader(f.settings.clone());let legacy=crate::local_chat::Server{id:"ollama",name:"unused",url:"http://127.0.0.1:1".into(),key:None};
 let sending=crate::local_chat::send_with(&channel,&legacy,"first-fixture","binding swap during wait".into(),None,|_|{});
 let change=async{tokio::time::sleep(Duration::from_millis(25)).await;let pet=f.config.pets.get_mut("ai_ollama").unwrap();pet.inference.as_mut().unwrap().endpoint=format!("http://{}",second.local_addr().unwrap());pet.inference_revision=Some("c".repeat(32));f.save();release.send(()).unwrap();};
 let(reply,())=tokio::join!(sending,change);reader.join().unwrap();assert_eq!(reply.err().unwrap(),crate::pet_inference::CAPTURE_CHANGED);assert_eq!(captured.endpoint,original);assert!(matches!(first.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));assert!(matches!(second.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));assert!(chat.history("ollama").is_empty());f.config=old;f.save();assert_eq!(captured.observation(),crate::pet_inference::CaptureObservation::Changed,"restoring a file cannot revive this retired turn");
}

#[tokio::test]
async fn inference_reader_wait_is_bounded_without_retiring_the_captured_target(){
 let f=Fixture::new("http://127.0.0.1:18471");let hold=f.coordinator().acquire("ollama",Arc::default(),None).unwrap();let captured=hold.captured().unwrap();let reader=f.settings.lock().unwrap();let until=Instant::now()+Duration::from_millis(20);assert_eq!(captured.wait_current(||true,until).await.unwrap_err(),crate::pet_inference::CAPTURE_BUSY);assert!(hold.current()&&hold.occupies_slot());drop(reader);assert!(captured.check().is_ok());assert_eq!(captured.wait_current(||true,until).await.unwrap_err(),crate::pet_inference::CAPTURE_BUSY,"a late ready observation cannot extend the wait deadline");assert!(captured.check().is_ok());
}

#[test]fn pet_inference_lmstudio_turn_never_claims_ollama_observed_usage(){
 let mut f=Fixture::new("http://127.0.0.1:18474");f.config.pets.get_mut("ai_ollama").unwrap().inference.as_mut().unwrap().backend=Backend::Lmstudio;f.save();
 let chat=Chat::persistent(f.root.join("usage.json")).with_runtime_resources(f.coordinator());let turn=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&turn);assert!(chat.usage_capture_origin(&turn).is_none());drop(lease);
}

#[test]fn captured_settings_contention_is_not_destination_revocation_and_never_reactivates_a_changed_binding(){
 use crate::pet_inference::CaptureObservation;
 let mut f=Fixture::new("http://127.0.0.1:18471");let c=f.coordinator();let hold=c.acquire("ollama",Arc::default(),None).unwrap();let captured=hold.captured().unwrap();let endpoint=captured.endpoint.clone();assert_eq!(captured.observation(),CaptureObservation::Current);
 let lock=f.settings.lock().unwrap();assert_eq!(captured.observation(),CaptureObservation::Busy);assert!(!captured.current());assert!(hold.current()&&hold.occupies_slot());assert!(captured.check().unwrap_err().contains("ocupada"),"temporary contention must not be reported as a changed destination");drop(lock);
 assert_eq!(captured.observation(),CaptureObservation::Current);assert_eq!(captured.endpoint,endpoint);
 let old=f.config.clone();f.config.pets.get_mut("ai_ollama").unwrap().inference.as_mut().unwrap().model="changed-fixture".into();f.config.pets.get_mut("ai_ollama").unwrap().inference_revision=Some("c".repeat(32));f.save();assert_eq!(captured.observation(),CaptureObservation::Changed);f.config=old;f.save();assert_eq!(captured.observation(),CaptureObservation::Changed);assert!(hold.occupies_slot(),"a changed destination does not prove worker drain");
}
