//! Read-only view proposal: explicit fields only, never daemon payload passthrough.
//! Labels are private UI data. Do not log or place this snapshot in chat/history.
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {
    pub schema_version: u8,
    pub observed_at: u64,
    pub source: &'static str,
    pub read_only: bool,
    pub wifi: Wifi,
    pub ethernet_connected: Option<bool>,
    pub cellular_connected: Option<bool>,
    pub cellular_enabled: Option<bool>,
    pub cellular_hardware_enabled: Option<bool>,
    pub hotspot: Hotspot,
    pub vpn_profiles: Option<Collection<Profile>>,
    pub active_vpns: Option<Collection<Profile>>,
    pub warnings: Vec<&'static str>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Wifi {
    pub enabled: Option<bool>, pub connected: Option<bool>,
    pub connecting: Option<bool>, pub current_name: Option<String>,
    pub signal: Option<u8>, pub nearby: Option<Collection<Network>>,
    pub saved: Option<Collection<Network>>, pub devices: Option<Collection<Device>>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Hotspot {
    pub supported: Option<bool>, pub available: Option<bool>,
    pub configured: Option<bool>, pub enabled: Option<bool>,
    pub activating: Option<bool>, pub secured: Option<bool>, pub name: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Collection<T> {
    pub entries: Vec<T>, pub complete: bool, pub observed_rows: usize, pub rejected_rows: usize,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Network {
    pub name: String, pub signal: Option<u8>, pub secured: Option<bool>,
    pub saved: Option<bool>, pub connected: Option<bool>, pub out_of_range: Option<bool>,
}
#[derive(Serialize)]
pub struct Device { pub name: String }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Profile { pub name: String, pub state: Option<&'static str> }
#[derive(Debug,PartialEq,Eq)]
pub enum Error { MissingCapability, InvalidState, InvalidTimestamp }

fn boolean(row:&Value,key:&str)->Option<bool>{row.get(key)?.as_bool()}
fn name(row:&Value,key:&str,max:usize)->Option<String>{
    let v=row.get(key)?.as_str()?;
    if v.is_empty()||v.len()>max||v.chars().any(|c|c.is_control()||matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')){return None;}
    Some(v.to_string())
}
fn signal(row:&Value,key:&str)->Option<u8>{
    row.get(key)?.as_u64().filter(|v|*v<=100).map(|v|v as u8)
}
fn network(row:&Value)->Option<Network>{
    Some(Network{name:name(row,"ssid",128)?,signal:signal(row,"signal"),secured:boolean(row,"secured"),saved:boolean(row,"saved"),connected:boolean(row,"connected"),out_of_range:boolean(row,"outOfRange")})
}
fn device(row:&Value)->Option<Device>{Some(Device{name:name(row,"name",64)?})}
fn profile(row:&Value)->Option<Profile>{
    let state=match row.get("state").and_then(Value::as_str){
        Some("activated")=>Some("activated"),Some("activating")=>Some("activating"),
        Some("deactivated")=>Some("deactivated"),Some("deactivating")=>Some("deactivating"),
        Some("failed")=>Some("failed"),_=>None,
    };
    Some(Profile{name:name(row,"name",128)?,state})
}
fn collection<T>(root:&Value,key:&str,limit:usize,parse:fn(&Value)->Option<T>,warnings:&mut Vec<&'static str>)->Option<Collection<T>>{
    let value=root.get(key)?;
    if value.is_null(){return None;}
    let Some(rows)=value.as_array() else{warnings.push("invalidCollection");return None;};
    if rows.len()>limit{warnings.push("collectionTooLarge");return None;}
    let entries:Vec<_>=rows.iter().filter_map(parse).collect();
    let rejected=rows.len()-entries.len();
    if rejected>0{warnings.push("invalidRows");}
    Some(Collection{entries,complete:rejected==0,observed_rows:rows.len(),rejected_rows:rejected})
}
/// observed_at is Unix epoch MILLISECONDS, bounded for exact JavaScript/Date transport.
/// A successful read is not proof of available device actions. No action handles
/// or method names are exported until a separately authorised operation API exists.
pub fn project(value:&Value,api_version:Option<u64>,capabilities:&[String],observed_at:u64)->Result<Snapshot,Error>{
    if !capabilities.iter().any(|v|v=="network"){return Err(Error::MissingCapability);}
    if !value.is_object(){return Err(Error::InvalidState);}
    if observed_at==0||observed_at>8_640_000_000_000_000{return Err(Error::InvalidTimestamp);}
    let mut warnings=Vec::new();
    let nearby=collection(value,"wifiNetworks",256,network,&mut warnings);
    // Old daemon views cannot prove a complete catalogue of saved/out-of-range networks.
    let saved=if api_version.is_some_and(|v|v>=26){collection(value,"savedWifiNetworks",256,network,&mut warnings)}else{warnings.push("savedNetworksUnsupported");None};
    let devices=collection(value,"wifiDevices",32,device,&mut warnings);
    let vpn_profiles=collection(value,"vpnProfiles",128,profile,&mut warnings);
    let active_vpns=collection(value,"vpnActive",128,profile,&mut warnings);
    let hotspot=if api_version.is_some_and(|v|v>=28){
        Hotspot{supported:boolean(value,"hotspotSupported"),available:boolean(value,"hotspotAvailable"),configured:boolean(value,"hotspotConfigured"),enabled:boolean(value,"hotspotEnabled"),activating:boolean(value,"hotspotActivating"),secured:boolean(value,"hotspotSecured"),name:name(value,"hotspotSSID",128)}
    }else{Hotspot{supported:None,available:None,configured:None,enabled:None,activating:None,secured:None,name:None}};
    Ok(Snapshot{schema_version:1,observed_at,source:"dmsNetwork",read_only:true,
        wifi:Wifi{enabled:boolean(value,"wifiEnabled"),connected:boolean(value,"wifiConnected"),connecting:boolean(value,"isConnecting"),current_name:name(value,"wifiSSID",128),signal:signal(value,"wifiSignal"),nearby,saved,devices},
        ethernet_connected:boolean(value,"ethernetConnected"),cellular_connected:boolean(value,"cellularConnected"),cellular_enabled:boolean(value,"cellularEnabled"),cellular_hardware_enabled:boolean(value,"cellularHardwareEnabled"),hotspot,vpn_profiles,active_vpns,warnings})
}
#[cfg(test)]
#[path="network_tests.rs"]
mod tests;
