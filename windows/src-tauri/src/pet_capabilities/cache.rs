//! Passive observations from existing explicit requests. No IO lives here.
use std::collections::BTreeMap;
use serde_json::Value;
pub(super) const TTL:u64=60_000;
pub(super) const MAX_DATE:u64=8_640_000_000_000_000;
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub(super) struct Key{pub provider:String,pub endpoint:String,pub model:Option<String>}
pub(crate) struct Ticket{pub(super) key:Key,sequence:u64,started:u64}
#[derive(Clone)]
pub(super) enum Evidence{Catalog,Model{completion:Option<bool>,tools:Option<bool>,thinking:Option<bool>,remote:bool},Unavailable}
struct Slot{sequence:u64,started:u64,observed:Option<(u64,Evidence)>}
#[derive(Default)]
pub(super) struct Cache{next:u64,slots:BTreeMap<Key,Slot>}
#[derive(Clone)]
pub(super) struct Observation{pub at:Option<u64>,pub expires:Option<u64>,pub stale:bool,pub evidence:Option<Evidence>}
impl Cache{
    pub fn begin(&mut self,key:Key,now:u64)->Option<Ticket>{
        if now==0||now>MAX_DATE||key.provider.len()>128||key.endpoint.len()>4096{return None;}
        self.next=self.next.checked_add(1)?;
        if self.slots.len()>=64&&!self.slots.contains_key(&key){let oldest=self.slots.iter().min_by_key(|(_,s)|s.sequence).map(|(k,_)|k.clone())?;self.slots.remove(&oldest);}
        self.slots.insert(key.clone(),Slot{sequence:self.next,started:now,observed:None});Some(Ticket{key,sequence:self.next,started:now})
    }
    pub fn finish(&mut self,t:Ticket,evidence:Evidence,now:u64)->bool{
        let Some(slot)=self.slots.get_mut(&t.key)else{return false;};
        if slot.sequence!=t.sequence{return false;}
        if now<t.started||now>MAX_DATE||now-t.started>30_000{slot.observed=None;return false;}
        slot.observed=Some((now,evidence));true
    }
    pub fn get(&self,key:&Key,now:u64)->Observation{
        let Some(s)=self.slots.get(key)else{return Observation{at:None,expires:None,stale:false,evidence:None};};
        let Some((at,evidence))=&s.observed else{return Observation{at:None,expires:None,stale:now<s.started||now.saturating_sub(s.started)>30_000,evidence:None};};
        let expires=at.checked_add(TTL).filter(|n|*n<=MAX_DATE);let stale=now<*at||expires.is_none_or(|e|now>=e);
        Observation{at:Some(*at),expires,stale,evidence:(!stale).then(||evidence.clone())}
    }
}
pub(super) fn model_id(value:&str)->bool{
    !value.is_empty()&&value.len()<=256&&!value.starts_with(['@','/','\\','~'])&&!value.contains(['?','#','\\'])&&!value.contains("://")&&!value.chars().any(|c|c.is_control()||c.is_whitespace())
        &&!std::path::Path::new(value).components().any(|c|matches!(c,std::path::Component::ParentDir))
}
pub(super) fn endpoint(raw:&str)->Option<String>{
    let url=crate::net::normalise_server_url(raw).ok()?;let key=url.as_str().trim_end_matches('/');(key.len()<=4096).then(||key.to_owned())
}
pub(super) fn model_evidence(value:&Value)->Evidence{
    let remote=["remote_model","remote_host"].iter().any(|k|value.get(k).is_some_and(|v|!v.is_null()&&v.as_str().is_none_or(|s|!s.trim().is_empty())));
    let caps=value.get("capabilities").and_then(Value::as_array).filter(|c|c.len()<=32&&c.iter().all(|v|v.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64&&!s.chars().any(char::is_control))));
    // Older Ollama metadata can advertise exact named/bool controls without a
    // "thinking" capability label. Share the real request parser's semantics.
    let thinking=crate::local_tasks::protocol::ModelCapabilities::from_show(value).reported_thinking();
    Evidence::Model{completion:caps.map(|c|c.iter().any(|v|v=="completion")),tools:caps.map(|c|c.iter().any(|v|v=="tools")),thinking,remote}
}
