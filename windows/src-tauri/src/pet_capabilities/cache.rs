//! Passive observations from existing explicit requests. No IO lives here.
use std::collections::BTreeMap;
use serde_json::Value;
pub(super) const TTL:u64=60_000;
pub(super) const MAX_DATE:u64=8_640_000_000_000_000;
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub(super) struct Key{pub provider:String,pub endpoint:String,pub model:Option<String>}
pub(crate) struct Ticket{pub(super) key:Key,sequence:u64,started:u64}
#[derive(Clone)]
pub(super) enum Evidence{Catalog,LmStudio(crate::local_chat::lmstudio::Catalog),Model{completion:Option<bool>,tools:Option<bool>,thinking:Option<bool>,remote:bool,installed:bool,context_tokens:Option<u64>},Unavailable}
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
    pub fn fresh_installed_local_model(&self,key:&Key,now:u64)->bool{
        if key.provider!="ollama"||key.model.is_none(){return false;}
        let observed=self.get(key,now);
        observed.at.is_some_and(|at|now>=at&&now-at<15_000)&&matches!(observed.evidence,Some(Evidence::Model{installed:true,completion:Some(true),remote:false,..}))
    }
    pub fn local_context_limit(&self,key:&Key,now:u64)->Option<u64>{
        if !self.fresh_installed_local_model(key,now){return None;}
        match self.get(key,now).evidence{Some(Evidence::Model{context_tokens,..})=>context_tokens,_=>None}
    }
    pub fn engine_binding(&self,key:&Key,now:u64)->Option<crate::local_chat::lmstudio::Binding>{
        if key.provider!="lmstudio"{return None;}let requested=key.model.as_ref()?;let mut catalog=key.clone();catalog.model=None;
        match self.get(&catalog,now).evidence{Some(Evidence::LmStudio(c))=>c.binding(requested).ok(),_=>None}
    }
    pub fn get(&self,key:&Key,now:u64)->Observation{
        if key.provider=="lmstudio"{if let Some(model)=&key.model{
            let mut catalog=key.clone();catalog.model=None;let mut observation=self.get(&catalog,now);
            if let Some(Evidence::LmStudio(c))=&observation.evidence{observation.evidence=Some(Evidence::Model{completion:Some(c.binding(model).is_ok()),tools:None,thinking:None,remote:false,installed:false,context_tokens:None});}
            return observation;
        }}
        let Some(s)=self.slots.get(key)else{return Observation{at:None,expires:None,stale:false,evidence:None};};
        let Some((at,evidence))=&s.observed else{return Observation{at:None,expires:None,stale:now<s.started||now.saturating_sub(s.started)>30_000,evidence:None};};
        let expires=at.checked_add(TTL).filter(|n|*n<=MAX_DATE);let stale=now<*at||expires.is_none_or(|e|now>=e);
        Observation{at:Some(*at),expires,stale,evidence:(!stale).then(||evidence.clone())}
    }
}
pub(super) fn model_id(value:&str)->bool{crate::pet_inference::model_key(value)}

pub(super) fn endpoint(raw:&str)->Option<String>{
    let url=crate::net::normalise_server_url(raw).ok()?;let key=url.as_str().trim_end_matches('/');(key.len()<=4096).then(||key.to_owned())
}
pub(super) fn model_evidence(value:&Value)->Evidence{
    let remote=["remote_model","remote_host"].iter().any(|k|value.get(k).is_some_and(|v|!v.is_null()&&v.as_str().is_none_or(|s|!s.trim().is_empty())));
    let caps=value.get("capabilities").and_then(Value::as_array).filter(|c|c.len()<=32&&c.iter().all(|v|v.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64&&!s.chars().any(char::is_control))));
    // Older Ollama metadata can advertise exact named/bool controls without a
    // "thinking" capability label. Share the real request parser's semantics.
    let thinking=crate::local_tasks::protocol::ModelCapabilities::from_show(value).reported_thinking();
    let context_tokens=value.pointer("/model_info/general.architecture").and_then(Value::as_str).filter(|s|!s.is_empty()&&s.len()<=64&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')).and_then(|architecture|value["model_info"][format!("{architecture}.context_length")].as_u64()).filter(|n|(256..=1_048_576).contains(n));
    Evidence::Model{completion:caps.map(|c|c.iter().any(|v|v=="completion")),tools:caps.map(|c|c.iter().any(|v|v=="tools")),thinking,remote,installed:crate::local_tasks::protocol::verify_local_model(value).is_ok(),context_tokens}
}

#[cfg(test)]mod context_tests{
 use super::*;
 #[test]fn runtime_capacity_is_exact_model_metadata_with_original_freshness_not_a_loaded_window_assumption(){
  let mut cache=Cache::default();let key=Key{provider:"ollama".into(),endpoint:"http://127.0.0.1:11434".into(),model:Some("chosen".into())};
  let show=serde_json::json!({"model_info":{"general.architecture":"gemma3","gemma3.context_length":131072},"capabilities":["completion"]});let t=cache.begin(key.clone(),1000).unwrap();cache.finish(t,model_evidence(&show),1001);
  assert_eq!(cache.local_context_limit(&key,1002),Some(131072));assert_eq!(cache.local_context_limit(&key,16001),None);let mut other=key.clone();other.model=Some("another".into());assert_eq!(cache.local_context_limit(&other,1002),None);
  for bad in [serde_json::json!({"model_info":{"general.architecture":"gemma3","other.context_length":131072},"capabilities":["completion"]}),serde_json::json!({"model_info":{"general.architecture":"gemma3","gemma3.context_length":"131072"},"capabilities":["completion"]}),serde_json::json!({"remote_host":"https://remote.invalid","model_info":{"general.architecture":"gemma3","gemma3.context_length":131072},"capabilities":["completion"]})]{let t=cache.begin(key.clone(),2000).unwrap();cache.finish(t,model_evidence(&bad),2001);assert_eq!(cache.local_context_limit(&key,2002),None);}
 }
}
