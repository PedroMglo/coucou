//! Measured local inference metadata. Never infer token counts from chat text.
use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::Duration};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle,Emitter,Manager};
const MAX_SAFE:u64=9_007_199_254_740_991;
#[derive(Clone,Debug,Serialize)]#[serde(rename_all="camelCase")]
pub struct Usage{
 pub input_tokens:Option<u64>,pub output_tokens:Option<u64>,pub total_tokens:Option<u64>,
 pub duration_ms:Option<u64>,pub measured:bool,pub scope:String,
 #[serde(skip_serializing_if="Option::is_none")]pub model:Option<String>,
 #[serde(skip_serializing_if="Option::is_none")]pub updated_at:Option<u64>,
}
impl Default for Usage{fn default()->Self{Self{input_tokens:None,output_tokens:None,total_tokens:None,duration_ms:None,measured:false,scope:"unavailable".into(),model:None,updated_at:None}}}
fn count(value:&Value)->Option<u64>{value.as_u64().filter(|n|*n<=MAX_SAFE)}
fn sum(a:Option<u64>,b:Option<u64>)->Option<u64>{a?.checked_add(b?).filter(|n|*n<=MAX_SAFE)}
impl Usage{
 pub fn observe_oai(&mut self,value:&Value){
  if let Some(model)=value["model"].as_str().filter(|m|!m.is_empty()&&m.len()<=512&&!m.chars().any(char::is_control)){self.model=Some(model.into());}
  let Some(u)=value.get("usage").filter(|u|u.is_object())else{return};
  self.input_tokens=count(&u["prompt_tokens"]);self.output_tokens=count(&u["completion_tokens"]);
  self.total_tokens=count(&u["total_tokens"]).or_else(||sum(self.input_tokens,self.output_tokens));
 }
 pub fn finish(mut self,model:&str,elapsed:Duration)->Self{
  if self.duration_ms.is_none(){self.duration_ms=u64::try_from(elapsed.as_millis()).ok().filter(|n|*n<=MAX_SAFE);}
  self.measured=self.duration_ms.is_some()||self.input_tokens.is_some()||self.output_tokens.is_some();
  self.scope="last-completed-turn".into();
  if self.model.is_none(){self.model=(!model.is_empty()&&model.len()<=512&&!model.chars().any(char::is_control)).then(||model.into());}
  self.updated_at=Some(crate::assistant_memory::now());self
 }
 pub fn native(value:&Value,model:&str,elapsed:Duration)->Self{
  let mut result=Self::default();result.input_tokens=count(&value["prompt_eval_count"]);result.output_tokens=count(&value["eval_count"]);
  if let Some(name)=value["model"].as_str().filter(|m|!m.is_empty()&&m.len()<=512&&!m.chars().any(char::is_control)){result.model=Some(name.into());}
  result.total_tokens=sum(result.input_tokens,result.output_tokens);
  result.duration_ms=value["total_duration"].as_u64().map(|n|n/1_000_000);
  result.finish(model,elapsed)
 }
 pub fn add(&mut self,next:&Self){
  self.input_tokens=sum(self.input_tokens,next.input_tokens);self.output_tokens=sum(self.output_tokens,next.output_tokens);
  self.total_tokens=sum(self.total_tokens,next.total_tokens);self.duration_ms=sum(self.duration_ms,next.duration_ms);
  self.measured|=next.measured;self.updated_at=next.updated_at.or(self.updated_at);
 }
}
#[derive(Default)]struct Store{values:BTreeMap<(String,String),(u64,Usage)>}
impl Store{
 fn put(&mut self,provider:&str,id:&str,epoch:u64,usage:Usage)->bool{
  let key=(provider.into(),id.into());if self.values.get(&key).is_some_and(|(e,_)|*e>epoch){return false;}
  if self.values.len()>=128&&!self.values.contains_key(&key){if let Some(old)=self.values.iter().min_by_key(|(_,(_,u))|u.updated_at.unwrap_or(0)).map(|(k,_)|k.clone()){self.values.remove(&old);}}
  self.values.insert(key,(epoch,usage));true
 }
 fn get(&self,provider:&str,id:&str)->Usage{self.values.get(&(provider.into(),id.into())).map(|(_,u)|u.clone()).unwrap_or_default()}
}
fn store()->&'static Mutex<Store>{static STORE:OnceLock<Mutex<Store>>=OnceLock::new();STORE.get_or_init(Mutex::default)}
pub fn record(app:Option<&AppHandle>,chat:&crate::chat::Chat,turn:&crate::chat::Turn,usage:Usage){
 if turn.provider!="ollama"{return;}
 let Some((id,epoch))=chat.usage_origin(turn)else{return};
 if store().lock().unwrap().put(&turn.provider,&id,epoch,usage.clone()){
  if let Some(app)=app{let _=app.emit("pet-usage-changed",serde_json::json!({"provider":turn.provider,"conversationId":id,"usage":usage}));}
 }
}
#[tauri::command]
pub async fn pet_usage(window:tauri::WebviewWindow,app:AppHandle,provider:String,conversation_id:Option<String>)->Result<Usage,String>{
 if ![crate::island::WINDOW_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,crate::pet_chat::LABEL,crate::pet_panel::LABEL,crate::HUB_LABEL].contains(&window.label()){return Err("Abre o pet para consultar as métricas.".into());}
 tauri::async_runtime::spawn_blocking(move||{
  let id=match conversation_id{Some(id)=>{if !app.state::<crate::chat::Chat>().has_public_conversation(&provider,&id){return Ok(Usage::default());}id},None=>app.state::<crate::chat::Chat>().conversation_id(&provider)?};
  Ok(store().lock().unwrap().get(&provider,&id))
 }).await.map_err(|_|"Não foi possível consultar as métricas.".to_string())?
}
#[cfg(test)]mod tests{
 use super::*;use serde_json::json;
 #[test]fn oai_usage_is_taken_only_from_server_counters(){let mut u=Usage::default();u.observe_oai(&json!({"choices":[{"delta":{"content":"a very long answer"}}]}));assert!(u.total_tokens.is_none());u.observe_oai(&json!({"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}}));let u=u.finish("local:test",Duration::from_millis(34));assert_eq!((u.input_tokens,u.output_tokens,u.total_tokens,u.duration_ms),(Some(12),Some(8),Some(20),Some(34)));}
 #[test]fn native_nanoseconds_and_zero_tokens_are_real_measurements(){let u=Usage::native(&json!({"prompt_eval_count":0,"eval_count":7,"total_duration":2_500_000}),"local",Duration::from_secs(9));assert_eq!((u.input_tokens,u.total_tokens,u.duration_ms),(Some(0),Some(7),Some(2)));}
 #[test]fn invalid_counts_stay_unknown(){let mut u=Usage::default();u.observe_oai(&json!({"usage":{"prompt_tokens":-1,"completion_tokens":"10","total_tokens":9_007_199_254_740_992_u64}}));assert!(u.input_tokens.is_none()&&u.output_tokens.is_none()&&u.total_tokens.is_none());assert!(!u.measured);}
 #[test]fn old_turns_and_other_dialogues_cannot_replace_usage(){let mut s=Store::default();let a=Usage::native(&json!({"eval_count":2}),"a",Duration::from_millis(1));s.put("ollama","one",2,a);assert!(!s.put("ollama","one",1,Usage::default()));assert_eq!(s.get("ollama","one").output_tokens,Some(2));assert!(!s.get("ollama","two").measured);assert!(!s.get("codex","one").measured);}
 #[test]fn voice_requests_are_aggregated_without_estimating_missing_counts(){let mut a=Usage::native(&json!({"prompt_eval_count":5,"eval_count":2}),"a",Duration::from_millis(10));let b=Usage::native(&json!({"prompt_eval_count":7,"eval_count":3}),"a",Duration::from_millis(20));a.add(&b);assert_eq!((a.input_tokens,a.output_tokens,a.total_tokens,a.duration_ms),(Some(12),Some(5),Some(17),Some(30)));a.add(&Usage::default());assert!(a.total_tokens.is_none());}
}
