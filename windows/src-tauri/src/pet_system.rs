//! Narrow native DMS panel entry points. No arbitrary targets, shell or settings writes.
use serde::{Deserialize,Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os="linux")]
static LEASE_OWNER: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
#[cfg(target_os="linux")]
fn lease_owner(app:&AppHandle)->Option<String>{
 let mut owner=LEASE_OWNER.lock().unwrap();
 if !app.state::<crate::Shared>().settings.lock().unwrap().pet_system_center{return None;}
 Some(owner.get_or_insert_with(||format!("coucou-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos())).clone())
}
/// Disabling System withdraws only this launch's claim. An in-flight request
/// from an earlier activation cannot withdraw a newer owner's surface.
pub async fn release_lease(){
 #[cfg(target_os="linux")]{
  let owner=LEASE_OWNER.lock().unwrap().take();
  if let Some(owner)=owner{let _=dms(&["ipc","call","pets-system","release",&owner],DmsUse::Release,std::sync::Arc::new(||Ok(()))).await;}
 }
}
/// RunEvent::Exit is outside the async runtime. Use a bounded IPC on a helper
/// thread; a crash still has the independent native twenty-second watchdog.
pub fn release_on_exit(){
 #[cfg(target_os="linux")]
 if let Err(_)=std::thread::spawn(||tauri::async_runtime::block_on(release_lease())).join(){
  crate::log::line("system lease release interrupted; native watchdog remains active");
 }
}
fn can_lease(label:&str,visible:bool)->bool{label==crate::pet_control::LABEL&&visible}

#[derive(Clone,Copy,Debug,Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum Panel {Calendar,Media,Weather,Notifications,QuickSettings,Wifi,Bluetooth,Battery,Audio,Clipboard,Disks,Recording,Tray}
impl Panel {
 fn arguments(self)->&'static [&'static str]{match self{
  Self::Calendar=>&["ipc","call","dash","openAt","overview","center"],
  Self::Media=>&["ipc","call","dash","openAt","media","center"],
  Self::Weather=>&["ipc","call","dash","openAt","weather","center"],
  Self::Notifications=>&["ipc","call","notifications","open"],
  Self::QuickSettings=>&["ipc","call","pets-system","open","quick-settings"],
  Self::Wifi=>&["ipc","call","pets-system","open","wifi"],
  Self::Bluetooth=>&["ipc","call","pets-system","open","bluetooth"],
  Self::Battery=>&["ipc","call","pets-system","open","battery"],
  Self::Audio=>&["ipc","call","pets-system","open","audio"],
  Self::Clipboard=>&["ipc","call","pets-system","open","clipboard"],
  Self::Disks=>&["ipc","call","pets-system","open","disks"],
  Self::Recording=>&["ipc","call","pets-system","open","recording"],
  Self::Tray=>&["ipc","call","pets-system","open","tray"],
 }}
 fn accepted(self,reply:&str)->bool{match self{
  Self::Calendar|Self::Media|Self::Weather=>reply=="DASH_OPEN_SUCCESS",
  Self::Notifications=>matches!(reply,"NOTIFICATION_MODAL_OPEN_SUCCESS"|"NOTIFICATION_ISLAND_OPEN_SUCCESS"),
  _=>reply=="PETS_SYSTEM_OPEN_SUCCESS",
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
pub struct Status{
 available:bool,updated_at:Option<u64>,media:Option<Media>,weather:Option<Weather>,notifications:Option<u64>,
 network:Option<Network>,bluetooth:Option<Bluetooth>,battery:Option<Battery>,audio:Option<Audio>,
 recording:Option<Recording>,clipboard:Option<Availability>,disks:Option<Availability>,tray:Option<Vec<TrayEntry>>,
}
#[derive(Serialize)]struct Media{title:String,artist:String,playing:bool}
#[derive(Serialize)]struct Weather{label:String,temperature:f64,unit:String}
#[derive(Serialize,Deserialize)]struct Network{available:bool,enabled:Option<bool>,connected:Option<bool>,ssid:Option<String>,signal:Option<f64>,ethernet:Option<bool>}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct Bluetooth{available:bool,enabled:Option<bool>,connected_names:Vec<String>}
#[derive(Serialize,Deserialize)]struct Battery{present:bool,percent:Option<f64>,charging:Option<bool>,plugged:Option<bool>}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct Audio{available:bool,volume:Option<f64>,muted:Option<bool>,device:Option<String>,mic_muted:Option<bool>}
#[derive(Serialize,Deserialize)]struct Recording{available:bool,active:Option<bool>,label:String}
#[derive(Serialize,Deserialize)]struct Availability{available:bool}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct TrayEntry{id:String,label:String,has_menu:bool,only_menu:bool}
#[derive(Clone,Copy,Deserialize)]#[serde(rename_all="kebab-case")]pub enum TrayAction{Activate,Menu}
impl TrayAction{fn argument(self)->&'static str{match self{Self::Activate=>"activate",Self::Menu=>"menu"}}}
fn decode<T:serde::de::DeserializeOwned>(value:&serde_json::Value)->Option<T>{serde_json::from_value(value.clone()).ok()}
fn valid_text(value:&str,max:usize)->bool{value.chars().count()<=max&&!value.chars().any(char::is_control)}
fn valid_tray_id(id:&str)->bool{
 if id.len()>96{return false;}
 let Some(tail)=id.strip_prefix("tray-") else{return false;};
 let parts:Vec<_>=tail.split('-').collect();
 parts.len()==3&&parts.iter().all(|p|!p.is_empty()&&p.bytes().all(|b|b.is_ascii_digit()))
}
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
 let network=decode::<Network>(&value["network"]).filter(|n|n.ssid.as_ref().is_none_or(|s|valid_text(s,128))&&n.signal.is_none_or(|s|s.is_finite()&&(0.0..=100.0).contains(&s)));
 let bluetooth=decode::<Bluetooth>(&value["bluetooth"]).filter(|b|b.connected_names.len()<=8&&b.connected_names.iter().all(|s|valid_text(s,64)));
 let battery=decode::<Battery>(&value["battery"]).filter(|b|b.percent.is_none_or(|p|p.is_finite()&&(0.0..=100.0).contains(&p)));
 let audio=decode::<Audio>(&value["audio"]).filter(|a|a.volume.is_none_or(|v|v.is_finite()&&(0.0..=300.0).contains(&v))&&a.device.as_ref().is_none_or(|s|valid_text(s,96)));
 let recording=decode::<Recording>(&value["recording"]).filter(|r|valid_text(&r.label,96));
 let tray=(||{
  let rows=value["tray"].as_array()?;if rows.len()>32{return None;}
  let entries:Vec<TrayEntry>=rows.iter().map(decode).collect::<Option<_>>()?;
  let mut seen=std::collections::HashSet::new();
  if entries.iter().any(|r|!valid_tray_id(&r.id)||!valid_text(&r.label,64)||!seen.insert(r.id.clone())){return None;}
  Some(entries)
 })();
 Some(Status{available:true,updated_at:Some(at),media,weather,notifications:value["notifications"].as_u64().filter(|n|*n<=10000),
  network,bluetooth,battery,audio,recording,clipboard:decode(&value["clipboard"]),disks:decode(&value["disks"]),tray})
}
fn authorize(window:&tauri::WebviewWindow,app:&AppHandle)->Result<(),String>{
 if !allowed(window.label()){return Err("Abre a ilha dos pets para aceder ao centro do sistema.".into());}
 if !app.state::<crate::Shared>().settings.lock().unwrap().pet_system_center{
  return Err("Ativa primeiro a integração do centro do sistema nas definições dos pets.".into());
 }
 Ok(())
}
fn present_system_panel(app:&AppHandle){
 // Fold visual cards only after native success. Pending decisions and drafts
 // stay with their conversations; a failed open leaves the retry UI visible.
 let _=app.emit_to(crate::island::WINDOW_LABEL,"companions-hide-popup",());
 crate::pet_control::collapse(app);
}

#[cfg(target_os="linux")]
#[derive(Clone,Copy)]enum DmsUse{Read,Renew,Action,Release}
#[cfg(target_os="linux")]
impl DmsUse{
 fn check(self)->Result<(),String>{self.check_with(crate::settings::effect_allowed)}
 fn check_with(self,current:impl FnOnce()->Result<(),String>)->Result<(),String>{match self{Self::Read|Self::Release=>Ok(()),Self::Renew|Self::Action=>current()}}
}
#[cfg(target_os="linux")]
type DmsCaller=std::sync::Arc<dyn Fn()->Result<(),String>+Send+Sync>;
#[cfg(target_os="linux")]
fn dms_caller(window:&tauri::WebviewWindow,app:&AppHandle)->DmsCaller{
 let window=window.clone();let app=app.clone();
 std::sync::Arc::new(move||authorize(&window,&app))
}
#[cfg(target_os="linux")]
fn dms_executable()->Result<(),String>{
 use std::os::unix::fs::MetadataExt;
 let metadata=std::fs::metadata("/usr/bin/dms").map_err(|_|"O DMS não está instalado nesta sessão.")?;
 if !metadata.is_file()||metadata.uid()!=0||metadata.mode()&0o022!=0||metadata.mode()&0o111==0{
  return Err("O executável do DMS não corresponde à instalação do sistema.".into());
 }
 Ok(())
}
#[cfg(target_os="linux")]
async fn dms(args:&[&str],use_:DmsUse,caller:DmsCaller)->Result<Vec<u8>,String>{
 use crate::pet_helper::{Lane,Session};use std::time::Duration;
 use_.check()?;dms_executable()?;
 let lane=match use_{DmsUse::Read=>Lane::DmsRead,DmsUse::Release=>Lane::DmsRelease,DmsUse::Renew|DmsUse::Action=>Lane::DmsEffect};
 let session=Session::acquire(lane,Duration::from_secs(3))?;
 let args=args.iter().map(|s|s.to_string()).collect::<Vec<_>>();
 session.run(crate::pet_helper::command("/usr/bin/dms",&args),16384,Duration::from_secs(3),move||{
  caller()?;dms_executable()?;use_.check()
 }).await
}

#[tauri::command]
pub async fn pet_system_status(window:tauri::WebviewWindow,app:AppHandle)->Result<Status,String>{
 authorize(&window,&app)?;
 #[cfg(target_os="linux")]{
  // Optional public metadata: an unavailable producer is unknown, never zero.
  // Only the visible control island claims the migrated native widgets.
  // Hub/diagnostic status reads remain pure and cannot hide the fallback.
  let ready=crate::settings::effect_allowed().is_ok();
  if !ready{release_lease().await;}
  let owner=(ready&&can_lease(window.label(),window.is_visible().unwrap_or(false))).then(||lease_owner(&app)).flatten();
  let args=match &owner{Some(owner)=>vec!["ipc","call","pets-system","lease",owner.as_str()],None=>vec!["ipc","call","pets-system","status"]};
  let caller=if let Some(expected)=owner.clone(){
   let original_window=window.clone();let original_app=app.clone();
   std::sync::Arc::new(move||{
    authorize(&original_window,&original_app)?;
    if !can_lease(original_window.label(),original_window.is_visible().unwrap_or(false))||!LEASE_OWNER.lock().is_ok_and(|v|v.as_ref()==Some(&expected)){
     return Err("A claim original do centro do sistema deixou de estar ativa.".into());
    }
    Ok(())
   })as DmsCaller
  }else{dms_caller(&window,&app)};
  let bytes=match dms(&args,if owner.is_some(){DmsUse::Renew}else{DmsUse::Read},caller).await{Ok(bytes)=>bytes,Err(_)=>return Ok(Status::default())};
  Ok(parse_status(&bytes,crate::assistant_memory::now()).unwrap_or_default())
 }
 #[cfg(not(target_os="linux"))]{Ok(Status::default())}
}

#[tauri::command]
pub async fn pet_system_panel(window:tauri::WebviewWindow,app:AppHandle,panel:Panel)->Result<(),String>{
 authorize(&window,&app)?;
 #[cfg(target_os="linux")]{
  let mut output=dms(panel.arguments(),DmsUse::Action,dms_caller(&window,&app)).await?;
  // A positioned dash requires a classic DMS bar. During migration an island
  // can still own that screen; only an explicit no-op failure permits fallback.
  if std::str::from_utf8(&output).is_ok_and(|reply|reply.trim()=="DASH_OPEN_FAILED"){
   if let Some(args)=panel.island_fallback(){output=dms(args,DmsUse::Action,dms_caller(&window,&app)).await?;}
  }
  if !std::str::from_utf8(&output).is_ok_and(|reply|panel.accepted(reply.trim())){
   return Err("O DMS não confirmou a abertura do painel.".into());
  }
  present_system_panel(&app);
  Ok(())
 }
 #[cfg(not(target_os="linux"))]{let _=panel;Err("Esta integração destina-se à sessão Linux com DMS.".into())}
}

#[tauri::command]
pub async fn pet_system_tray(window:tauri::WebviewWindow,app:AppHandle,id:String,action:TrayAction)->Result<(),String>{
 authorize(&window,&app)?;
 if !valid_tray_id(&id){return Err("Atualiza a lista de aplicações antes de abrir este ícone.".into());}
 #[cfg(target_os="linux")]{
  let bytes=dms(&["ipc","call","pets-system","tray",&id,action.argument()],DmsUse::Action,dms_caller(&window,&app)).await?;
  match std::str::from_utf8(&bytes).unwrap_or("").trim(){
   "PETS_SYSTEM_TRAY_SUCCESS"=>{present_system_panel(&app);Ok(())},
   "PETS_SYSTEM_TRAY_GONE"=>Err("A aplicação saiu da área de notificações; atualiza a lista.".into()),
   "PETS_SYSTEM_TRAY_NO_MENU"=>Err("Esta aplicação não disponibiliza um menu.".into()),
   _=>Err("O DMS não confirmou a ação sobre a aplicação.".into()),
  }
 }
 #[cfg(not(target_os="linux"))]{let _=(id,action);Err("Esta integração destina-se à sessão Linux com DMS.".into())}
}

#[cfg(test)]mod tests{
 use super::*;
 #[cfg(target_os="linux")]
 #[test]fn system_io_unknown_origin_never_grants_renewal_or_actions_and_never_blocks_release_or_read(){
  let io=crate::settings::EffectFixture::new();io.external();for kind in [DmsUse::Renew,DmsUse::Action]{assert!(kind.check_with(||io.probe.check()).is_err());}let held=io.current.lock().unwrap();for kind in [DmsUse::Read,DmsUse::Release]{assert!(kind.check_with(||panic!("control/read must not wait on preferences")).is_ok());}drop(held);
 }
 #[test]fn only_the_visible_control_surface_can_renew_the_native_bar_lease(){
  assert!(can_lease(crate::pet_control::LABEL,true));
  for label in [crate::pet_control::LABEL,crate::island::WINDOW_LABEL,crate::HUB_LABEL,"unknown"]{
   assert!(!can_lease(label,false));
   if label!=crate::pet_control::LABEL{assert!(!can_lease(label,true));}
  }
 }
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
  assert!(Panel::QuickSettings.accepted("PETS_SYSTEM_OPEN_SUCCESS"));assert!(!Panel::QuickSettings.accepted("CONTROL_CENTER_OPEN_SUCCESS"));assert!(!Panel::QuickSettings.accepted("true"));
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
 #[test]fn migrated_system_controls_use_only_fixed_native_routes(){
  for panel in [Panel::QuickSettings,Panel::Wifi,Panel::Bluetooth,Panel::Battery,Panel::Audio,Panel::Clipboard,Panel::Disks,Panel::Recording,Panel::Tray]{
   let args=panel.arguments();assert_eq!(&args[..5],["ipc","call","pets-system","open",args[4]]);
   assert_eq!(args.len(),5);assert!(panel.accepted("PETS_SYSTEM_OPEN_SUCCESS"));
   assert!(!panel.accepted("CONTROL_CENTER_OPEN_SUCCESS"));assert!(panel.island_fallback().is_none());
  }
  assert_eq!(Panel::QuickSettings.arguments(),["ipc","call","pets-system","open","quick-settings"]);
  for id in ["","udiskie","tray-a-2-3","tray-1-2-3;open","tray-1-2-3-4","tray-1--3"]{assert!(!valid_tray_id(id));}
  assert!(valid_tray_id("tray-1234-5678-1"));
  assert!(serde_json::from_value::<TrayAction>(serde_json::json!("toggle-wifi")).is_err());
 }
 #[test]fn system_device_values_are_optional_bounded_and_do_not_expose_private_fields(){
  let input=serde_json::json!({"version":1,"available":true,"updatedAt":100000,
   "network":{"available":true,"enabled":true,"connected":true,"ssid":"Rede","signal":80,"ethernet":false,"password":"private-omit"},
   "bluetooth":{"available":true,"enabled":true,"connectedNames":["Auscultadores"],"address":"private-omit"},
   "battery":{"present":true,"percent":50.5,"charging":false,"plugged":false},
   "audio":{"available":true,"volume":0,"muted":true,"device":"Saída","micMuted":null},
   "recording":{"available":true,"active":false,"label":"Ecrã"},"clipboard":{"available":true,"entries":["private-omit"]},"disks":{"available":true},
   "tray":[{"id":"tray-123-456-1","label":"Aplicação","hasMenu":true,"onlyMenu":false,"dbus":"private-omit"}]});
  let value=serde_json::to_value(parse_status(&serde_json::to_vec(&input).unwrap(),100001).unwrap()).unwrap();
  assert_eq!(value["audio"]["volume"],0.0);assert_eq!(value["battery"]["percent"],50.5);
  assert_eq!(value["tray"][0]["id"],"tray-123-456-1");assert!(!value.to_string().contains("private-omit"));
  let mut bad=input;bad["network"]["signal"]=serde_json::json!(101);bad["battery"]["percent"]=serde_json::json!(-1);
  bad["tray"][0]["id"]=serde_json::json!("org.example.app");bad["audio"]["device"]=serde_json::json!("control\ntext");
  let invalid=serde_json::to_value(parse_status(&serde_json::to_vec(&bad).unwrap(),100001).unwrap()).unwrap();
  assert!(invalid["network"].is_null()&&invalid["battery"].is_null()&&invalid["tray"].is_null()&&invalid["audio"].is_null());
  assert!(invalid["bluetooth"].is_object());
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
