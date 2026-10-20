//! In-memory projection of public native messages between persisted snapshots.
//! An observer joining mid-message never invents the missing prefix.
use std::collections::HashMap;
use serde_json::Value;
use super::types::{Message,Role,Snapshot};
const MAX_ITEM:usize=4*1024*1024;
const MAX_STREAM:usize=16*1024*1024;

#[derive(Default)]pub struct Store {threads:HashMap<String,Stream>,limited:bool}
#[derive(Default)]struct Stream {turn:Option<String>,items:Vec<Item>,notice:Option<String>}
struct Item {id:String,message:Message,known_start:bool,complete:bool}

impl Store {
    pub fn event(&mut self,event:&Value){
        let params=&event["params"];
        let Some(thread)=name(&params["threadId"])else{return;};
        if !self.threads.contains_key(thread)&&self.threads.len()>=128{
            let idle=self.threads.iter().find(|(_,s)|s.turn.is_none()&&s.items.is_empty()).map(|(id,_)|id.clone());
            if let Some(id)=idle{self.threads.remove(&id);}else{self.limited=true;return;}
        }
        let stream=self.threads.entry(thread.into()).or_default();
        let method=event["method"].as_str().unwrap_or("");
        if method=="turn/started"{stream.turn=name(&params["turn"]["id"]).map(str::to_string);return;}
        if method=="turn/completed"{if stream.turn.as_deref()==params["turn"]["id"].as_str(){stream.turn=None;}return;}
        let Some(turn)=name(&params["turnId"])else{return;};
        let (item_id,kind,text,complete)=match method{
            "item/agentMessage/delta"=>{
                let(Some(id),Some(text))=(name(&params["itemId"]),params["delta"].as_str())else{return;};
                (id,"delta",text.to_string(),false)
            },
            "item/started"|"item/completed"=>{
                let item=&params["item"];let Some(id)=name(&item["id"])else{return;};
                let text=match item["type"].as_str(){
                    Some("agentMessage")=>item["text"].as_str().unwrap_or("").to_string(),
                    Some("userMessage")=>item["content"].as_array().into_iter().flatten().filter(|v|v["type"]=="text").filter_map(|v|v["text"].as_str()).collect::<Vec<_>>().join("\n"),
                    _=>return,
                };
                (id,if item["type"]=="userMessage"{"user"}else{"assistant"},text,method=="item/completed")
            },
            _=>return,
        };
        let id=serde_json::to_string(&(turn,item_id)).unwrap_or_default();
        let size:usize=stream.items.iter().map(|i|i.message.content.len()).sum();
        if text.len()>MAX_ITEM||size.saturating_add(text.len())>MAX_STREAM||stream.items.len()>=512{
            stream.notice=Some("A resposta em direto excedeu o limite de memória. O histórico nativo completo será consultado no fim do turno.".into());return;
        }
        if let Some(item)=stream.items.iter_mut().find(|i|i.id==id){
            if kind=="delta"{
                if !item.complete{if item.message.content.len().saturating_add(text.len())>MAX_ITEM{stream.notice=Some("A resposta em direto excedeu o limite de memória; aguarda o histórico nativo final.".into());return;}item.message.content.push_str(&text);}
            }else{item.message.content=text;item.known_start=true;item.complete=complete;}
        }else{
            stream.items.push(Item{id:id.clone(),message:Message{id,role:if kind=="user"{Role::User}else{Role::Assistant},content:text},known_start:kind!="delta",complete});
        }
    }

    pub fn merge(&mut self,mut snapshot:Snapshot)->Snapshot{
        let Some(stream)=self.threads.get_mut(&snapshot.target.session_id)else{
            if self.limited&&!snapshot.control_conflict{snapshot.notice=Some("O limite de respostas simultâneas em direto foi atingido. O histórico nativo final continua disponível.".into());}return snapshot;
        };
        let mut partial=false;
        stream.items.retain(|item|{
            if item.message.content.is_empty(){return true;}
            if let Some(saved)=snapshot.messages.iter_mut().find(|m|m.id==item.id){
                if saved.content==item.message.content {return !item.complete;}
                if item.complete||item.known_start&&item.message.content.starts_with(&saved.content){saved.content=item.message.content.clone();}
                // Persisted full text is authoritative when a suffix lacks a baseline.
            }else{snapshot.messages.push(item.message.clone());if !item.known_start{partial=true;}}
            true
        });
        if snapshot.busy{
            if !snapshot.control_conflict{if let Some(turn)=&stream.turn{snapshot.turn=Some(turn.clone());snapshot.capabilities.cancel=snapshot.capabilities.history;}}
        }else{stream.turn=None;}
        if partial&&!snapshot.control_conflict{snapshot.notice=Some("Ligação iniciada a meio de uma mensagem: o trecho inicial aparecerá quando o Codex concluir essa mensagem.".into());}
        if !snapshot.control_conflict{if let Some(notice)=&stream.notice{snapshot.notice=Some(notice.clone());}}
        if stream.items.is_empty(){stream.notice=None;}
        if stream.items.is_empty()&&stream.turn.is_none(){self.threads.remove(&snapshot.target.session_id);self.limited=false;}
        snapshot
    }

    pub fn gap(&mut self){for stream in self.threads.values_mut(){for item in &mut stream.items{if !item.complete{item.known_start=false;}}stream.turn=None;}}
    pub fn reconnect(&mut self){self.threads.clear();self.limited=false;}
}
fn name(value:&Value)->Option<&str>{value.as_str().filter(|s|!s.is_empty()&&s.len()<=1024&&!s.chars().any(char::is_control))}
