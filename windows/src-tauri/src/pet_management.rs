//! Existing workstation AI inventory/actions, presented by the pets instead of a bar widget.
//! No shell, transcript parsing, credential export or implicit model execution.
use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Request {pub action:String,#[serde(default)]pub provider:String,#[serde(default)]pub identifier:String,#[serde(default)]pub keep_alive:String}
const ACTIONS:&[&str]=&["list","usage","usage-refresh","usage-page","settings","model-settings","edit-config","project-config","folder","docs","open","focus","new","interrupt","archive","unarchive","load","unload","service-start","service-stop"];
const PROVIDERS:&[&str]=&["codex","claude","copilot","ollama","gemini","opencode","aider","lmstudio"];
fn arguments(r:&Request)->Result<Vec<String>,String>{
 if !ACTIONS.contains(&r.action.as_str()) || (!r.provider.is_empty() && !PROVIDERS.contains(&r.provider.as_str())) || r.identifier.len()>4096 || r.identifier.chars().any(char::is_control) || r.identifier.starts_with('-') {return Err("Pedido de gestão de pets inválido.".into());}
 let keep=if r.keep_alive.is_empty(){"10m"}else{&r.keep_alive};
 if !["5m","10m","30m","1h"].contains(&keep){return Err("Retenção inválida.".into());}
 if ["load","unload","model-settings","service-start","service-stop"].contains(&r.action.as_str()) && r.provider!="ollama" {return Err("Esta ação pertence ao pet Ollama.".into());}
 Ok(vec![r.action.clone(),if r.provider.is_empty(){"codex".into()}else{r.provider.clone()},r.identifier.clone(),"--keep-alive".into(),keep.into()])
}
#[cfg(target_os="linux")]
fn execute(args:Vec<String>)->Result<Value,String>{
 use std::{io::Read,os::unix::{fs::MetadataExt,process::CommandExt},process::{Command,Stdio},time::{Duration,Instant}};
 let helper=crate::platform::home_dir().join(".local/bin/ia-barra");
 let meta=std::fs::symlink_metadata(&helper).map_err(|_|"Instala o mecanismo ia-barra do sysworkspace para gerir sessões e modelos.")?;
 if !meta.is_file() || meta.uid()!=unsafe{libc::geteuid()} || meta.mode()&0o022!=0 {return Err("O mecanismo de gestão deve ser um ficheiro do utilizador, sem escrita por outros.".into());}
 let mut child=Command::new(helper).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).process_group(0).spawn().map_err(|_|"Não foi possível iniciar a gestão dos pets.")?;
 let pid=child.id() as i32;let mut stdout=child.stdout.take().unwrap();
 use std::os::fd::AsRawFd;
 unsafe{let fd=stdout.as_raw_fd();let flags=libc::fcntl(fd,libc::F_GETFL);libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK);}
 let started=Instant::now();let mut data=Vec::new();let mut buffer=[0u8;16384];let mut eof=false;
 let result=(||->Result<Value,String>{loop{
  loop{match stdout.read(&mut buffer){Ok(0)=>{eof=true;break},Ok(n)=>{data.extend_from_slice(&buffer[..n]);if data.len()>16*1024*1024{return Err("O inventário excedeu o limite de leitura.".into());}},Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>break,Err(_)=>return Err("Falhou a leitura do inventário.".into())}}
  if eof && child.try_wait().map_err(|_|"Falhou a consulta da gestão.")?.is_some(){break;}
  if started.elapsed()>Duration::from_secs(70){return Err("A gestão demorou demasiado. Atualiza o estado antes de repetir a ação.".into());}
  std::thread::sleep(Duration::from_millis(30));
 }
 let value:Value=serde_json::from_slice(&data).map_err(|_|"A gestão devolveu um resultado inválido.")?;
 if let Some(error)=value.get("error").and_then(Value::as_str){return Err(error.chars().take(500).collect());}Ok(value)})();
 if result.is_err(){unsafe{libc::kill(-pid,libc::SIGTERM);}let _=child.kill();}let _=child.wait();result
}
#[cfg(not(target_os="linux"))]
fn execute(_:Vec<String>)->Result<Value,String>{Err("A gestão da estação está disponível no Linux com sysworkspace.".into())}

#[cfg(target_os="linux")]
const USAGE_CACHE_LIMIT:u64=256*1024;
/// The inventory can carry existing quota metadata without refreshing a provider.
/// Reject unsafe files and rebuild a small allowlist: the cache may also contain
/// account names and other fields that do not belong in companion alerts.
#[cfg(target_os="linux")]
fn read_usage_at(path:&std::path::Path,now:u64)->Option<Value>{
 use std::{io::Read,os::unix::fs::{MetadataExt,OpenOptionsExt}};
 let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK).open(path).ok()?;
 let meta=file.metadata().ok()?;
 if !meta.is_file()||meta.uid()!=unsafe{libc::geteuid()}||meta.mode()&0o022!=0||meta.len()>USAGE_CACHE_LIMIT{return None;}
 let mut bytes=Vec::new();file.take(USAGE_CACHE_LIMIT+1).read_to_end(&mut bytes).ok()?;
 if bytes.len() as u64>USAGE_CACHE_LIMIT{return None;}
 let source:Value=serde_json::from_slice(&bytes).ok()?;let source=source.get("providers")?.as_object()?;
 let mut providers=serde_json::Map::new();
 for provider in ["codex","claude","copilot","antigravity"]{
  if let Some(quota)=source.get(provider).and_then(|value|public_quota(value,now)){providers.insert(provider.into(),quota);}
 }
 if providers.is_empty(){None}else{Some(serde_json::json!({"providers":providers}))}
}
#[cfg(target_os="linux")]
fn public_quota(source:&Value,now:u64)->Option<Value>{
 let source=source.as_object()?;let mut quota=serde_json::Map::new();
 let updated=match source.get("updated"){None|Some(Value::Null)=>None,Some(value)=>{let stamp=value.as_u64()?;if stamp==0||stamp>now{return None;}Some(stamp)}};
 if let Some(stamp)=updated{quota.insert("updated".into(),stamp.into());}
 for key in ["available","blocked"]{if let Some(value)=source.get(key){quota.insert(key.into(),value.as_bool()?.into());}}
 let stale=match source.get("stale"){Some(value)=>value.as_bool()?,None=>false}||updated.is_none_or(|stamp|now-stamp>300);
 if let Some(value)=source.get("windows"){
  let values=value.as_array()?;if values.len()>32{return None;}let mut windows=Vec::new();
  for value in values{
   let source=value.as_object()?;let used=source.get("used")?.as_f64()?;if !used.is_finite()||!(0.0..=100.0).contains(&used){return None;}
   let mut window=serde_json::Map::new();window.insert("used".into(),source.get("used")?.clone());
   for key in ["label","name"]{if let Some(value)=source.get(key){window.insert(key.into(),quota_name(value)?.into());}}
   for key in ["reset","reset_at"]{if let Some(value)=source.get(key){
    if value.is_null(){continue;}
    let reset=if let Some(stamp)=value.as_u64(){if stamp==0{return None;}stamp.into()}else if key=="reset"{quota_name(value)?.into()}else{return None;};
    window.insert(key.into(),reset);
   }}
   windows.push(Value::Object(window));
  }
  quota.insert("windows".into(),windows.into());
 }
 if let Some(value)=source.get("unlimited"){
  let values=value.as_array()?;if values.len()>16{return None;}let names:Option<Vec<_>>=values.iter().map(|value|quota_name(value).map(Value::from)).collect();
  quota.insert("unlimited".into(),names?.into());
 }
 if quota.is_empty(){return None;}quota.insert("stale".into(),stale.into());Some(Value::Object(quota))
}
#[cfg(target_os="linux")]
fn quota_name(value:&Value)->Option<&str>{let text=value.as_str()?;if text.is_empty()||text.len()>128||text.chars().any(char::is_control){None}else{Some(text)}}
fn attach_cached_usage(value:&mut Value){
 #[cfg(target_os="linux")]{
  let root=std::env::var_os("XDG_CACHE_HOME").map(std::path::PathBuf::from).filter(|path|path.is_absolute()).unwrap_or_else(||crate::platform::home_dir().join(".cache"));
  let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|value|value.as_secs());
  if let (Some(object),Some(now))=(value.as_object_mut(),now){if let Some(usage)=read_usage_at(&root.join("ia-barra/usage.json"),now){object.insert("usage".into(),usage);}}
 }
 #[cfg(not(target_os="linux"))]let _=value;
}

/// Memory and service changes share the pet's admission gate with text and
/// voice. Keep the reservation in the blocking worker until the helper exits.
fn reserve_maintenance(chat:&crate::chat::Chat,request:&Request)->Result<Option<crate::chat::ConversationReservation>,String>{
 if request.provider=="ollama" && ["load","unload","service-start","service-stop"].contains(&request.action.as_str()) {
  chat.reserve_conversation("ollama",None).map(Some)
 }else{Ok(None)}
}
fn run_admitted<T>(reservation:Option<crate::chat::ConversationReservation>,work:impl FnOnce()->Result<T,String>)->Result<T,String>{
 if let Some(guard)=&reservation{guard.channel().check_admission("ollama")?;}
 work()
}
#[tauri::command]
pub async fn pet_management(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,request:Request)->Result<Value,String>{
 use tauri::Manager;
 let metadata=window.label()==crate::island::WINDOW_LABEL&&request.action=="list";
 if !metadata && window.label()!=crate::HUB_LABEL && window.label()!=crate::pet_panel::LABEL && window.label()!=crate::pet_control::LABEL{return Err("Abre a gestão dos pets para executar esta ação.".into());}
 let args=arguments(&request)?;
 let reservation=reserve_maintenance(&chat,&request)?.map(|guard|guard.broadcast(window.app_handle(),""));
 let inventory=request.action=="list";
 tauri::async_runtime::spawn_blocking(move||run_admitted(reservation,||{let mut value=execute(args)?;if inventory{attach_cached_usage(&mut value);}Ok(value)})).await.map_err(|_|"A gestão parou inesperadamente.".to_string())?
}
/// Focus only: never resumes or sends text to an unrelated native conversation.
#[tauri::command]
pub async fn pet_focus_session(window:tauri::WebviewWindow,provider:String,id:String)->Result<Value,String>{
 if ![crate::island::WINDOW_LABEL,crate::pet_chat::LABEL,crate::pet_panel::LABEL,crate::HUB_LABEL,crate::companions::LABEL,crate::pet_control::LABEL].contains(&window.label()){return Err("Abre o pet para consultar esta sessão.".into());}
 if id.is_empty(){return Err("A sessão não tem identificador.".into());}
 let args=arguments(&Request{action:"focus".into(),provider,identifier:id,keep_alive:String::new()})?;
 tauri::async_runtime::spawn_blocking(move||execute(args)).await.map_err(|_|"A consulta da sessão parou inesperadamente.".to_string())?
}
#[cfg(test)]mod tests{
 use super::*;
 use crate::chat::Chat;
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 fn request(action:&str)->Request{Request{action:action.into(),provider:"ollama".into(),identifier:"test-model".into(),keep_alive:"10m".into()}}
 #[test]fn helper_has_no_shell_or_arbitrary_actions(){let mut r=Request{action:"load".into(),provider:"ollama".into(),identifier:"qwen3:8b".into(),keep_alive:"10m".into()};assert_eq!(arguments(&r).unwrap(),["load","ollama","qwen3:8b","--keep-alive","10m"]);r.identifier="--help".into();assert!(arguments(&r).is_err());r.identifier="qwen3:8b".into();r.action="claude-status".into();assert!(arguments(&r).is_err());r.action="unload".into();r.provider="codex".into();assert!(arguments(&r).is_err());}
 #[test]fn model_changes_refuse_active_voice_and_pending_text_requests(){
  let chat=Chat::default();let voice=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&voice);
  for action in ["load","unload","service-start","service-stop"]{assert!(reserve_maintenance(&chat,&request(action)).is_err());}
  assert!(chat.is_current(&voice));drop(lease);
  let text=chat.reserve_conversation("ollama",None).unwrap();let before=chat.conversation("ollama").unwrap();
  for action in ["load","unload","service-start","service-stop"]{assert!(reserve_maintenance(&chat,&request(action)).is_err());}
  assert_eq!(chat.conversation("ollama").unwrap().turn,before.turn);assert!(text.channel().check_admission("ollama").is_ok());drop(text);
 }
 #[test]fn maintenance_keeps_the_pet_reserved_until_the_helper_returns_even_on_error(){
  let chat=Chat::default();let reservation=reserve_maintenance(&chat,&request("unload")).unwrap();
  let result=run_admitted(reservation,||->Result<(),String>{
   assert!(chat.conversation("ollama").unwrap().busy);
   assert!(chat.begin_cli("ollama",Arc::default()).is_err());
   assert!(chat.reserve_conversation("ollama",None).is_err());
   assert!(reserve_maintenance(&chat,&request("service-stop")).is_err());
   Err("controlled helper failure".into())
  });
  assert!(result.is_err());assert!(!chat.conversation("ollama").unwrap().busy);
  let turn=chat.begin_cli("ollama",Arc::default()).unwrap();drop(chat.lease(&turn));
 }
 #[test]fn inventory_and_other_pets_do_not_take_the_ollama_reservation(){
  let chat=Chat::default();let voice=chat.begin_cli("ollama",Arc::default()).unwrap();let _lease=chat.lease(&voice);
  for action in ["list","usage","model-settings","docs"]{assert!(reserve_maintenance(&chat,&request(action)).unwrap().is_none());}
  let mut r=request("open");r.provider="codex".into();assert!(reserve_maintenance(&chat,&r).unwrap().is_none());
  let other=chat.begin_cli("codex",Arc::default()).unwrap();assert!(chat.is_current(&other));drop(chat.lease(&other));assert!(chat.is_current(&voice));
 }
 #[tokio::test]async fn cancelled_queued_maintenance_never_calls_the_helper(){
  let chat=Chat::default();let reservation=reserve_maintenance(&chat,&request("unload")).unwrap();
  let queued=chat.conversation("ollama").unwrap();let counter=Arc::new(AtomicUsize::new(0));let called=counter.clone();
  let (release,wait)=std::sync::mpsc::channel();
  let job=tauri::async_runtime::spawn_blocking(move||{
   wait.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
   run_admitted(reservation,||{called.fetch_add(1,Ordering::SeqCst);Ok(())})
  });
  assert!(chat.cancel_conversation("ollama",Some(&queued.id),Some(queued.turn)));release.send(()).unwrap();
  assert!(job.await.unwrap().is_err());assert_eq!(counter.load(Ordering::SeqCst),0);assert!(!chat.conversation("ollama").unwrap().busy);
 }
 #[cfg(target_os="linux")]
 struct UsageFixture{directory:std::path::PathBuf,path:std::path::PathBuf}
 #[cfg(target_os="linux")]
 impl UsageFixture{
  fn new()->Self{
   use std::os::unix::fs::DirBuilderExt;
   static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
   let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
   let directory=std::env::temp_dir().join(format!("coucou-usage-{}-{nonce}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
   std::fs::DirBuilder::new().mode(0o700).create(&directory).unwrap();let path=directory.join("usage.json");Self{directory,path}
  }
  fn bytes(&self,bytes:&[u8]){
   use std::{io::Write,os::unix::fs::OpenOptionsExt};
   let mut file=std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&self.path).unwrap();file.write_all(bytes).unwrap();
  }
  fn json(&self,value:Value){self.bytes(&serde_json::to_vec(&value).unwrap());}
  fn read(&self)->Option<Value>{read_usage_at(&self.path,1_000)}
 }
 #[cfg(target_os="linux")]
 impl Drop for UsageFixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.directory);}}
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_allowlists_providers_and_strips_account_secrets(){
  let fixture=UsageFixture::new();fixture.json(serde_json::json!({"attempted":999,"token":"root-secret","providers":{
   "codex":{"updated":999,"stale":false,"blocked":false,"available":true,"account":"private@example.com","plan":"private","note":"token=secret","arbitrary":{"token":"secret"},"windows":[{"name":"Five hours","label":"Quota","used":86.5,"reset":1200,"reset_at":1200,"token":"window-secret","remaining":13.5}],"unlimited":["Completions"]},
   "antigravity":{"updated":999,"available":false},"unknown":{"updated":999,"windows":[{"used":99}]}
  }}));
  assert_eq!(fixture.read().unwrap(),serde_json::json!({"providers":{
   "codex":{"updated":999,"stale":false,"blocked":false,"available":true,"windows":[{"name":"Five hours","label":"Quota","used":86.5,"reset":1200,"reset_at":1200}],"unlimited":["Completions"]},
   "antigravity":{"updated":999,"available":false,"stale":false}
  }}));
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_marks_source_and_age_stale_without_refreshing(){
  let fixture=UsageFixture::new();
  for (updated,source,expected) in [(700,false,false),(699,false,true),(999,true,true)]{
   fixture.json(serde_json::json!({"providers":{"codex":{"updated":updated,"stale":source,"windows":[{"used":85}]}}}));
   assert_eq!(fixture.read().unwrap()["providers"]["codex"]["stale"],expected);
  }
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_missing_timestamp_never_fabricates_current_or_zero_usage(){
  let fixture=UsageFixture::new();fixture.json(serde_json::json!({"providers":{"claude":{"available":false}}}));
  assert_eq!(fixture.read().unwrap(),serde_json::json!({"providers":{"claude":{"available":false,"stale":true}}}));
  fixture.json(serde_json::json!({"providers":{"codex":{"updated":999,"windows":[{"label":"Missing percentage"}]}}}));assert!(fixture.read().is_none());
  fixture.json(serde_json::json!({"providers":{"codex":{"secret":"no known metadata"}}}));assert!(fixture.read().is_none());
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_missing_corrupt_and_empty_sources_are_optional(){
  let fixture=UsageFixture::new();assert!(fixture.read().is_none());fixture.bytes(b"{");assert!(fixture.read().is_none());
  for value in [serde_json::json!({"providers":{}}),serde_json::json!({"providers":{"unknown":{"updated":999}}}),serde_json::json!({"providers":[]}),Value::Null]{fixture.json(value);assert!(fixture.read().is_none());}
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_rejects_oversize_files(){
  let fixture=UsageFixture::new();fixture.bytes(&vec![b' ';USAGE_CACHE_LIMIT as usize+1]);assert!(fixture.read().is_none());
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_rejects_symlinks_and_nonregular_files(){
  let source=UsageFixture::new();source.json(serde_json::json!({"providers":{"codex":{"updated":999,"available":true}}}));assert!(source.read().is_some());
  let link=UsageFixture::new();std::os::unix::fs::symlink(&source.path,&link.path).unwrap();assert!(link.read().is_none());
  let directory=UsageFixture::new();std::fs::create_dir(&directory.path).unwrap();assert!(directory.read().is_none());
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_rejects_group_and_other_writable_files(){
  use std::os::unix::fs::PermissionsExt;
  let fixture=UsageFixture::new();fixture.json(serde_json::json!({"providers":{"codex":{"updated":999,"available":true}}}));
  for mode in [0o620,0o602,0o666]{std::fs::set_permissions(&fixture.path,std::fs::Permissions::from_mode(mode)).unwrap();assert!(fixture.read().is_none());}
  std::fs::set_permissions(&fixture.path,std::fs::Permissions::from_mode(0o600)).unwrap();assert!(fixture.read().is_some());
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_future_timestamps_and_invalid_shapes_are_not_current(){
  let fixture=UsageFixture::new();
  for quota in [serde_json::json!({"updated":1001,"available":true}),serde_json::json!({"updated":"999","available":true}),serde_json::json!({"updated":999,"blocked":"false"}),serde_json::json!({"updated":999,"stale":0}),serde_json::json!({"updated":999,"windows":[{"used":"85"}]}),serde_json::json!({"updated":999,"windows":[{"used":-1}]}),serde_json::json!({"updated":999,"windows":[{"used":101}]}),serde_json::json!({"updated":999,"windows":[{"used":85,"reset_at":"1200"}]})]{
   fixture.json(serde_json::json!({"providers":{"codex":quota}}));assert!(fixture.read().is_none());
  }
 }
 #[cfg(target_os="linux")]
 #[test]fn cached_usage_bounds_windows_and_names_instead_of_truncating(){
  let fixture=UsageFixture::new();
  for quota in [serde_json::json!({"updated":999,"windows":vec![serde_json::json!({"used":85});33]}),serde_json::json!({"updated":999,"unlimited":vec!["Quota";17]}),serde_json::json!({"updated":999,"windows":[{"used":85,"label":"x".repeat(129)}]}),serde_json::json!({"updated":999,"unlimited":["bad\nname"]})]{
   fixture.json(serde_json::json!({"providers":{"codex":quota}}));assert!(fixture.read().is_none());
  }
 }
}
