//! Narrow native DMS panel entry points. No arbitrary targets, shell or settings writes.
use serde::{Deserialize,Serialize};
use tauri::{AppHandle, Manager};

#[derive(Clone,Copy,Debug,Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum Panel {Calendar,Media,Weather,Notifications,QuickSettings}
impl Panel {
 fn arguments(self)->&'static [&'static str]{match self{
  Self::Calendar=>&["ipc","call","dash","openAt","overview","center"],
  Self::Media=>&["ipc","call","dash","openAt","media","center"],
  Self::Weather=>&["ipc","call","dash","openAt","weather","center"],
  Self::Notifications=>&["ipc","call","notifications","open"],
  Self::QuickSettings=>&["ipc","call","control-center","open"],
 }}
 fn accepted(self,reply:&str)->bool{match self{
  Self::Calendar|Self::Media|Self::Weather=>reply=="DASH_OPEN_SUCCESS",
  Self::Notifications=>matches!(reply,"NOTIFICATION_MODAL_OPEN_SUCCESS"|"NOTIFICATION_ISLAND_OPEN_SUCCESS"),
  Self::QuickSettings=>reply=="CONTROL_CENTER_OPEN_SUCCESS",
 }}
 fn island_fallback(self)->Option<&'static [&'static str]>{match self{
  Self::Calendar=>Some(&["ipc","call","dash","open","overview"]),
  Self::Media=>Some(&["ipc","call","dash","open","media"]),
  Self::Weather=>Some(&["ipc","call","dash","open","weather"]),
  _=>None,
 }}
}
fn allowed(label:&str)->bool{[crate::pet_control::LABEL,crate::island::WINDOW_LABEL,crate::HUB_LABEL].contains(&label)}

#[derive(Default,Serialize)]#[serde(rename_all="camelCase")]
pub struct Status{available:bool,updated_at:Option<u64>,media:Option<Media>,weather:Option<Weather>,notifications:Option<u64>}
#[derive(Serialize)]struct Media{title:String,artist:String,playing:bool}
#[derive(Serialize)]struct Weather{label:String,temperature:f64,unit:String}
fn text(value:&serde_json::Value,max:usize)->Option<String>{
 let value=value.as_str()?;
 (value.chars().count()<=max&&!value.chars().any(char::is_control)).then(||value.to_string())
}
fn parse_status(bytes:&[u8],now:u64)->Option<Status>{
 if bytes.len()>16384{return None;}
 let value:serde_json::Value=serde_json::from_slice(bytes).ok()?;
 if value["version"]!=1||value["available"]!=true{return None;}
 let at=value["updatedAt"].as_u64()?;
 if at==0||at>now.saturating_add(5000)||now.saturating_sub(at)>15000{return None;}
 let media=(||{let m=&value["media"];Some(Media{title:text(&m["title"],512)?,artist:text(&m["artist"],512)?,playing:m["playing"].as_bool()?})})();
 let weather=(||{
  let w=&value["weather"];let temperature=w["temperature"].as_f64()?;let unit=w["unit"].as_str()?;
  if !temperature.is_finite()||!(-500.0..=500.0).contains(&temperature)||!matches!(unit,"°C"|"°F"){return None;}
  Some(Weather{label:text(&w["label"],128)?,temperature,unit:unit.into()})
 })();
 Some(Status{available:true,updated_at:Some(at),media,weather,notifications:value["notifications"].as_u64().filter(|n|*n<=10000)})
}
fn authorize(window:&tauri::WebviewWindow,app:&AppHandle)->Result<(),String>{
 if !allowed(window.label()){return Err("Abre a ilha dos pets para aceder ao centro do sistema.".into());}
 if !app.state::<crate::Shared>().settings.lock().unwrap().pet_system_center{
  return Err("Ativa primeiro a integração do centro do sistema nas definições dos pets.".into());
 }
 Ok(())
}

#[cfg(target_os="linux")]
async fn dms(args:&[&str])->Result<Vec<u8>,String>{
 use std::{os::unix::fs::MetadataExt,process::Stdio,time::Duration};
 let metadata=std::fs::metadata("/usr/bin/dms").map_err(|_|"O DMS não está instalado nesta sessão.")?;
 if !metadata.is_file()||metadata.uid()!=0||metadata.mode()&0o022!=0||metadata.mode()&0o111==0{
  return Err("O executável do DMS não corresponde à instalação do sistema.".into());
 }
 let output=tokio::time::timeout(Duration::from_secs(3),tokio::process::Command::new("/usr/bin/dms")
  .args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
  .kill_on_drop(true).output()).await
  .map_err(|_|"O centro do sistema não respondeu a tempo.".to_string())?
  .map_err(|_|"Não foi possível contactar o DMS nesta sessão.".to_string())?;
 if !output.status.success()||output.stdout.len()>16384{return Err("O DMS não confirmou o pedido.".into());}
 Ok(output.stdout)
}

#[tauri::command]
pub async fn pet_system_status(window:tauri::WebviewWindow,app:AppHandle)->Result<Status,String>{
 authorize(&window,&app)?;
 #[cfg(target_os="linux")]{
  // Optional public metadata: an unavailable producer is unknown, never zero.
  let bytes=match dms(&["ipc","call","pets-system","status"]).await{Ok(bytes)=>bytes,Err(_)=>return Ok(Status::default())};
  Ok(parse_status(&bytes,crate::assistant_memory::now()).unwrap_or_default())
 }
 #[cfg(not(target_os="linux"))]{Ok(Status::default())}
}

#[tauri::command]
pub async fn pet_system_panel(window:tauri::WebviewWindow,app:AppHandle,panel:Panel)->Result<(),String>{
 authorize(&window,&app)?;
 #[cfg(target_os="linux")]{
  let mut output=dms(panel.arguments()).await?;
  // A positioned dash requires a classic DMS bar. During migration an island
  // can still own that screen; only an explicit no-op failure permits fallback.
  if std::str::from_utf8(&output).is_ok_and(|reply|reply.trim()=="DASH_OPEN_FAILED"){
   if let Some(args)=panel.island_fallback(){output=dms(args).await?;}
  }
  if !std::str::from_utf8(&output).is_ok_and(|reply|panel.accepted(reply.trim())){
   return Err("O DMS não confirmou a abertura do painel.".into());
  }
  Ok(())
 }
 #[cfg(not(target_os="linux"))]{let _=panel;Err("Esta integração destina-se à sessão Linux com DMS.".into())}
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn panel_identifiers_cannot_turn_into_commands_or_settings_writes(){
  for value in ["settings","calendar;rm","quick-settings open","notifications/clearAll","", "control-center"]{
   assert!(serde_json::from_value::<Panel>(serde_json::json!(value)).is_err());
  }
  for value in [Panel::Calendar,Panel::Media,Panel::Weather,Panel::Notifications,Panel::QuickSettings]{
   let args=value.arguments();assert_eq!(&args[..2],["ipc","call"]);assert!(!args.contains(&"settings"));assert!(!args.contains(&"clearAll"));
  }
 }
 #[test]fn only_native_success_for_the_requested_panel_is_acknowledged(){
  assert!(Panel::Calendar.accepted("DASH_OPEN_SUCCESS"));assert!(!Panel::Calendar.accepted("DASH_OPEN_FAILED"));
  assert!(!Panel::Notifications.accepted("DASH_OPEN_SUCCESS"));assert!(Panel::Notifications.accepted("NOTIFICATION_MODAL_OPEN_SUCCESS"));
  assert!(Panel::QuickSettings.accepted("CONTROL_CENTER_OPEN_SUCCESS"));assert!(!Panel::QuickSettings.accepted("true"));
 }
 #[test]fn background_or_other_app_surfaces_cannot_open_system_panels(){
  assert!(allowed(crate::pet_control::LABEL));assert!(!allowed(crate::pet_chat::LABEL));assert!(!allowed(crate::companions::LABEL));assert!(!allowed("phone"));
 }
 #[test]fn wider_compact_reservation_follows_the_same_user_setting(){
  assert_eq!(crate::pet_control::width(1920.0,false),288.0);assert_eq!(crate::pet_control::width(1920.0,true),432.0);
  assert_eq!(crate::pet_control::width(360.0,true),344.0);
 }
 #[test]fn transitional_island_fallback_uses_only_the_same_requested_activity(){
  assert_eq!(Panel::Calendar.island_fallback().unwrap(),["ipc","call","dash","open","overview"]);
  assert_eq!(Panel::Media.island_fallback().unwrap(),["ipc","call","dash","open","media"]);
  assert_eq!(Panel::Weather.island_fallback().unwrap(),["ipc","call","dash","open","weather"]);
  assert!(Panel::Notifications.island_fallback().is_none()&&Panel::QuickSettings.island_fallback().is_none());
 }
 #[test]fn system_metadata_keeps_only_public_fields_and_native_units(){
  let input=serde_json::json!({"version":1,"available":true,"updatedAt":100000,"media":{"title":"Música","artist":"Artista","playing":false,"privateUrl":"omit"},"weather":{"label":"Céu limpo","temperature":18.5,"unit":"°C"},"notifications":0,"notificationBodies":["omit"]});
  let status=parse_status(&serde_json::to_vec(&input).unwrap(),100100).unwrap();let value=serde_json::to_value(status).unwrap();
  assert_eq!(value["media"]["playing"],false);assert_eq!(value["weather"]["temperature"],18.5);assert_eq!(value["notifications"],0);
  assert!(!value.to_string().contains("omit"));
 }
 #[test]fn unavailable_stale_and_invalid_values_never_become_fake_measurements(){
  for input in [serde_json::json!({"version":1,"available":true,"updatedAt":1}),serde_json::json!({"version":2,"available":true,"updatedAt":100000}),serde_json::json!({"version":1,"available":true,"updatedAt":200000})]{
   assert!(parse_status(&serde_json::to_vec(&input).unwrap(),100000).is_none());
  }
  let input=serde_json::json!({"version":1,"available":true,"updatedAt":100000,"media":{"title":"x","artist":"y","playing":"true"},"weather":{"label":"x","temperature":"0","unit":"°C"},"notifications":-1});
  let value=serde_json::to_value(parse_status(&serde_json::to_vec(&input).unwrap(),100001).unwrap()).unwrap();
  assert!(value["media"].is_null()&&value["weather"].is_null()&&value["notifications"].is_null());
  assert!(serde_json::to_value(Status::default()).unwrap()["updatedAt"].is_null());
 }
}
