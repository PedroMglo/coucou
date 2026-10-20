//! Private API34 structural evidence only, never a public dispatch grant.
use serde_json::Value;
use std::collections::BTreeSet;
fn text(row:&Value,key:&str,max:usize,nonempty:bool)->bool {
    row.get(key).and_then(Value::as_str).is_some_and(|s|s.len()<=max&&(!nonempty||!s.is_empty())
        &&!s.chars().any(|c|c.is_control()||matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')))
}
fn unsigned(row:&Value,key:&str,max:u64)->bool {row.get(key).and_then(Value::as_u64).is_some_and(|v|v<=max)}
fn flag(row:&Value,key:&str)->bool {row.get(key).is_some_and(Value::is_boolean)}
fn optional(row:&Value,key:&str,valid:impl FnOnce(&Value)->bool)->bool {row.get(key).is_none_or(valid)}
fn network(row:&Value)->bool {
    row.is_object()&&text(row,"ssid",128,false)&&text(row,"bssid",128,false)&&text(row,"mode",64,false)
        &&unsigned(row,"signal",u8::MAX.into())
        &&["frequency","rate","channel"].iter().all(|k|unsigned(row,k,u32::MAX.into()))
        &&["secured","enterprise","connected","saved","autoconnect","hidden","outOfRange"].iter().all(|k|flag(row,k))
        &&optional(row,"device",|_|text(row,"device",64,false))
}
fn device(row:&Value)->bool {
    if !row.is_object()||!text(row,"name",64,true)||!text(row,"hwAddress",128,false)||!text(row,"state",64,false)
        ||!flag(row,"connected")||!flag(row,"apCapable"){return false;}
    if !["ssid","bssid","ip"].iter().all(|k|optional(row,k,|_|text(row,k,128,false)))
        ||!optional(row,"signal",|_|unsigned(row,"signal",u8::MAX.into())){return false;}
    // Go nil slices serialize null; an absent/wrong-type list is not a native row.
    match row.get("networks") {
        Some(Value::Null)=>true,
        Some(Value::Array(rows))=>rows.len()<=256&&rows.iter().all(network),
        _=>false,
    }
}
pub(super) fn supported(value:&Value,api:Option<u64>,caps:&[String])->bool {
    if api!=Some(34)||!caps.iter().any(|s|s=="network")||value.get("backend").and_then(Value::as_str)!=Some("networkmanager")
        ||!flag(value,"wifiEnabled"){return false;}
    let Some(rows)=value.get("wifiDevices").and_then(Value::as_array)else{return false;};
    if rows.is_empty()||rows.len()>32||!rows.iter().all(device){return false;}
    let mut names=BTreeSet::new();rows.iter().all(|r|names.insert(r["name"].as_str().unwrap()))
}
