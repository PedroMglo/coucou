//! Bundled route declarations. These never represent a runtime permission.
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub enum RuntimeKind {Llm,Service,Observer}
#[derive(Clone,Copy,Debug,Deserialize,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub enum Adapter {Codex,Copilot,Antigravity,Ollama,Lmstudio,Spotify}
impl Adapter {pub fn key(self)->&'static str{match self{Self::Codex=>"codex",Self::Copilot=>"copilot",Self::Antigravity=>"antigravity",Self::Ollama=>"ollama",Self::Lmstudio=>"lmstudio",Self::Spotify=>"spotify"}}}
#[derive(Clone,Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Runtime {pub kind:RuntimeKind,pub adapter:Option<Adapter>,#[serde(default)]pub assistant_adapter:Option<Adapter>}
#[derive(Clone,Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Routing {pub chat:Option<String>,pub voice:Option<String>,pub execute:bool,pub task_provider:Option<String>,pub role:Option<String>,pub task_name:Option<String>}
#[derive(Clone,Debug)]
pub struct Blueprint {pub id:String,pub service:bool,pub retired:bool,pub runtime:Runtime,pub routing:Routing,expected_provider:String}
impl Blueprint {
    pub fn configured_adapter(&self,provider:&str)->Option<Adapter>{
        if self.runtime.adapter==Some(Adapter::Lmstudio){return (provider=="lmstudio").then_some(Adapter::Lmstudio);}
        self.runtime.assistant_adapter.filter(|adapter|provider==adapter.key())
            .or_else(||(provider==self.expected_provider).then_some(self.runtime.adapter).flatten())
    }
    /// Serialized identity compatibility only; no configuration or execution grant.
    /// Adapter-less historical owners remain readable under their original route.
    pub fn recognizes_owner(&self,provider:&str,adapter:&str)->bool{
        [&self.routing.chat,&self.routing.voice,&self.routing.task_provider].iter().any(|route|route.as_deref()==Some(provider))
            && match self.runtime.adapter{
                Some(native)=>adapter==native.key()||self.runtime.assistant_adapter.is_some_and(|a|adapter==a.key()),
                None=>adapter==provider,
            }
    }
    pub fn accepts_provider(&self,provider:&str)->bool{
        provider==self.expected_provider||self.runtime.adapter==Some(Adapter::Lmstudio)&&provider=="lmstudio"||self.runtime.assistant_adapter.is_some_and(|adapter|provider==adapter.key())
    }
    pub fn history_owner(&self)->bool{self.routing.chat.is_some()||self.routing.task_provider.is_some()}
}
fn key(s:&str)->bool{!s.is_empty()&&s.len()<=128&&s.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_'))}
pub(super)fn parse(source:&Value,config:&super::Configuration)->Result<BTreeMap<String,Blueprint>,String>{
    let invalid=||"Invalid bundled pet factory".to_string();
    let pets=source["pets"].as_object().ok_or_else(invalid)?;let mut entries=BTreeMap::new();let mut owners=BTreeMap::new();
    for(id,raw)in pets{
        if !key(id)||matches!(id.as_str(),"__proto__"|"prototype"|"constructor"){return Err(invalid());}
        let rt=raw.get("runtime").ok_or_else(invalid)?;
        if rt.get("kind").is_none()||rt.get("adapter").is_none(){return Err(invalid());}
        let runtime:Runtime=serde_json::from_value(rt.clone()).map_err(|_|invalid())?;
        let r=raw.get("routing").ok_or_else(invalid)?;
        if !["chat","voice","execute","taskProvider","role","taskName"].iter().all(|k|r.get(k).is_some()){return Err(invalid());}
        let mut routing:Routing=serde_json::from_value(r.clone()).map_err(|_|invalid())?;
        if [&routing.role,&routing.task_name].into_iter().flatten().any(|s|s.trim().is_empty()||s.len()>256||s.chars().any(char::is_control)){return Err(invalid());}
        let service=raw.get("service").and_then(Value::as_bool).ok_or_else(invalid)?;
        let mold=raw.get("mold").and_then(Value::as_str).ok_or_else(invalid)?;
        if (runtime.kind==RuntimeKind::Service)!=(mold=="application")||service&&runtime.kind!=RuntimeKind::Service{return Err(invalid());}
        let provider=&config.pets.get(id).ok_or_else(invalid)?.provider;
        for route in [&routing.chat,&routing.voice,&routing.task_provider].into_iter().flatten(){
            if !key(route)||owners.get(route).is_some_and(|owner|owner!=id){return Err(invalid());}owners.insert(route.clone(),id.clone());
        }
        let logical=if service{id.as_str()}else{provider.as_str()};
        if runtime.assistant_adapter.is_some()&&(!service||runtime.adapter!=Some(Adapter::Spotify)||runtime.assistant_adapter!=Some(Adapter::Codex)){return Err(invalid());}
        match runtime.adapter{
            Some(Adapter::Lmstudio)=>{if runtime.kind!=RuntimeKind::Llm||service||!provider.is_empty()||routing.chat.as_deref()!=Some("lmstudio")||routing.task_provider.as_deref()!=Some("lmstudio")||routing.voice.is_some()||routing.execute{return Err(invalid());}},
            Some(Adapter::Spotify)=>{if !service||!provider.is_empty()||routing.chat.as_deref()!=runtime.assistant_adapter.map(|_|logical)||routing.voice.as_deref()!=Some(logical)||routing.task_provider.as_deref()!=Some(logical)||routing.execute!=runtime.assistant_adapter.is_some(){return Err(invalid());}},
            Some(adapter)=>{
                if runtime.kind==RuntimeKind::Observer||provider!=adapter.key()||service&&adapter!=Adapter::Codex
                    ||routing.chat.as_deref()!=Some(logical)||routing.voice.as_deref()!=Some(logical)||routing.task_provider.as_deref()!=Some(logical)
                    ||routing.execute!=(adapter!=Adapter::Ollama){return Err(invalid());}
            },
            None=>{if routing.voice.is_some()||routing.execute||service||!provider.is_empty(){return Err(invalid());}
                if runtime.kind==RuntimeKind::Llm&&routing.chat.is_none()||runtime.kind!=RuntimeKind::Llm&&routing.chat.is_some(){return Err(invalid());}}
        }
        let retired=raw["retired"].as_bool().unwrap_or(false);if retired{routing.execute=false;}
        entries.insert(id.clone(),Blueprint{id:id.clone(),service,retired,runtime,routing,expected_provider:provider.clone()});
    }
    // A route may be repeated for the same owner in different channels, never for another pet.
    let ids:BTreeSet<_>=entries.keys().collect();if ids.len()!=config.pets.len(){return Err(invalid());}Ok(entries)
}
