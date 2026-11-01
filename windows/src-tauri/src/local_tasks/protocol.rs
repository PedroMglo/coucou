//! Bounded, complete-round decoding. Prose and incomplete calls are never tools.
use std::{collections::{BTreeMap,HashSet},time::{Duration,Instant}};
use serde_json::{json,Value};
use super::types::{MAX_CALLS_PER_ROUND,MAX_PROPOSAL,CommandInput};
use crate::pet_observability::{capture::{Capture,Origin as UsageOrigin},usage::AttemptOutcome};

pub struct ToolCall {pub id:String,pub input:CommandInput,pub wire:Value}
pub struct CalendarCall {pub id:String,pub args:Value,pub name:String}
pub struct Round {pub text:String,pub calls:Vec<ToolCall>,pub calendar_calls:Vec<CalendarCall>,pub message:Value,pub usage:crate::pet_usage::Usage}
#[derive(Default)]struct Fragment {id:String,name:String,args:String}
#[derive(Default)]pub struct Decoder {text:String,calls:BTreeMap<usize,Fragment>,finish:Option<String>,done:bool,bytes:usize,usage:crate::pet_usage::Usage}
impl Decoder {
 #[cfg(test)]
 pub fn line(&mut self,line:&str)->Result<(),String>{self.line_observed(line,None,false)}
 pub fn line_captured(&mut self,line:&str,capture:Option<&mut Capture>)->Result<(),String>{
  // Prove the marker on untrimmed wire text. Legacy Unicode whitespace handling
  // remains unchanged below, but cannot turn NBSP or other bytes into Final.
  let complete=line.strip_prefix("data:").is_some_and(|raw|raw.trim_matches(|c:char|c.is_ascii_whitespace())=="[DONE]");
  self.line_observed(line.trim_end(),capture,complete)
 }
 fn line_observed(&mut self,line:&str,capture:Option<&mut Capture>,complete:bool)->Result<(),String>{
  self.bytes=self.bytes.saturating_add(line.len());if line.len()>1024*1024||self.bytes>4*1024*1024{return Err("A resposta do modelo excede os limites.".into());}
  let Some(raw)=line.strip_prefix("data:")else{return Ok(())};let raw=raw.trim();
  if raw=="[DONE]"{let first=!self.done;self.done=true;if first&&complete{if let Some(capture)=capture{capture.transport_complete();}}return Ok(());}if raw.is_empty(){return Ok(());}
  // Reject post-terminator data before parsing or observing it: an invalid late
  // usage event must never replace the already confirmed Final snapshot.
  if self.done{return Err("Dados após o fim da resposta do modelo.".into());}
  let v:Value=serde_json::from_str(raw).map_err(|_|"Resposta estruturada inválida do modelo.")?;
  // Capture decoded counters before server/content/tool validation can reject
  // this event. Observations replace snapshots and never add stream deltas.
  if let Some(capture)=capture{capture.observe_oai(&v);}
  if v.get("error").is_some(){return Err("O servidor recusou a ronda do modelo.".into());}
  self.usage.observe_oai(&v);
  let choices=v["choices"].as_array().ok_or("Faltam escolhas na resposta do modelo.")?;
  if choices.len()>1{return Err("O servidor devolveu várias escolhas inesperadas.".into());}
  let Some(choice)=choices.first()else{return Ok(())};
  if choice["index"].as_u64().is_some_and(|n|n!=0){return Err("Índice de resposta inesperado.".into());}
  let delta=&choice["delta"];
  if self.finish.is_some()&&(delta.get("content").is_some_and(|x|!x.is_null())||delta.get("tool_calls").is_some()){return Err("Conteúdo após o fim da ronda.".into());}
  if let Some(text)=delta.get("content").filter(|x|!x.is_null()){self.text.push_str(text.as_str().ok_or("Texto de resposta inválido.")?);}
  if let Some(calls)=delta.get("tool_calls").filter(|x|!x.is_null()){
   for call in calls.as_array().ok_or("Lista de ferramentas inválida.")?{
    let index=call["index"].as_u64().filter(|i|*i<MAX_CALLS_PER_ROUND as u64).ok_or("Demasiadas ferramentas na mesma ronda.")? as usize;
    if call.get("type").is_some_and(|t|t!="function"){return Err("Tipo de ferramenta indisponível.".into());}
    let f=self.calls.entry(index).or_default();
    for (value,out,cap) in [(call.get("id"),&mut f.id,128),(call.pointer("/function/name"),&mut f.name,64),(call.pointer("/function/arguments"),&mut f.args,MAX_PROPOSAL)]{
     if let Some(value)=value{out.push_str(value.as_str().ok_or("Fragmento de ferramenta inválido.")?);if out.len()>cap{return Err("A proposta excede os limites.".into());}}
    }
   }
  }
  if let Some(finish)=choice.get("finish_reason").filter(|f|!f.is_null()){
   let finish=finish.as_str().ok_or("Fim de resposta inválido.")?;
   if self.finish.is_some(){return Err("Fim de resposta repetido.".into());}self.finish=Some(finish.into());
  }Ok(())
 }
 pub fn visible(&self)->String{crate::local_chat::progressive_filter(&self.text)}
 #[cfg(test)]
 pub fn complete(self,model:&str,elapsed:Duration)->Result<Round,String>{self.complete_with_calendar(model,elapsed,false)}
 fn complete_with_calendar(self,model:&str,elapsed:Duration,calendar:bool)->Result<Round,String>{
  let expected=if self.calls.is_empty(){"stop"}else{"tool_calls"};
  if self.finish.as_deref()!=Some(expected){return Err("A ronda do modelo ficou incompleta; nenhum comando dessa ronda foi executado.".into());}
  let mut seen=HashSet::new();let mut calls=Vec::new();let mut calendar_calls=Vec::new();let mut wires=Vec::new();
  for (expected,(index,f)) in self.calls.into_iter().enumerate(){
   if index!=expected||(f.name!="execute_command"&&!(calendar&&crate::pet_calendar::query::is_name(&f.name)))||f.id.is_empty()||f.id.chars().any(char::is_control)||!seen.insert(f.id.clone()){return Err("Identidade de ferramenta inválida.".into());}
   let wire=json!({"id":f.id,"type":"function","function":{"name":f.name,"arguments":f.args}});
   if crate::pet_calendar::query::is_name(&f.name){
    let args:Value=serde_json::from_str(&f.args).map_err(|_|"Argumentos de calendário inválidos.")?;
    if !args.is_object()||f.args.len()>4096{return Err("Argumentos de calendário inválidos.".into());}
    wires.push(wire);calendar_calls.push(CalendarCall{id:f.id,args,name:f.name});
   }else{
    let input:CommandInput=serde_json::from_str(&f.args).map_err(|_|"Argumentos de comando inválidos.")?;
    let call=ToolCall{id:f.id,input,wire};wires.push(call.wire.clone());calls.push(call);
   }
  }
  let text=crate::local_chat::progressive_filter(&self.text);
  let mut message=json!({"role":"assistant","content":text});
  if !wires.is_empty(){message["tool_calls"]=json!(wires);}
  Ok(Round{text,calls,calendar_calls,message,usage:self.usage.finish(model,elapsed)})
 }
}

pub fn tool_schema()->Value{json!({"type":"function","function":{"name":"execute_command","description":"Propose one exact local command. The user must approve it once before execution; refusal is not approval. The working directory is fixed by the user.","parameters":{"type":"object","additionalProperties":false,"required":["program","args","reason"],"properties":{"program":{"type":"string"},"args":{"type":"array","items":{"type":"string"}},"reason":{"type":"string"},"timeoutMs":{"type":"integer","minimum":100,"maximum":60000}}}}})}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ToolsSupport{Confirmed,Unavailable,Remote}
pub struct ModelCapabilities{pub tools:ToolsSupport,thinking:Vec<Value>,thinking_default:Option<Value>,thinking_capable:Option<bool>}
impl Default for ModelCapabilities{fn default()->Self{Self{tools:ToolsSupport::Unavailable,thinking:vec![],thinking_default:None,thinking_capable:None}}}
impl ModelCapabilities{
 pub(crate) fn reported_thinking(&self)->Option<bool>{
  if !self.thinking.is_empty(){Some(self.thinking.iter().any(|v|v==&Value::Bool(true)||v.is_string()))}else{self.thinking_capable}
 }
 pub fn from_show(value:&Value)->Self{
  if ["remote_model","remote_host"].iter().any(|k|value.get(k).is_some_and(|v|!v.is_null()&&v.as_str().is_none_or(|s|!s.trim().is_empty()))){return Self{tools:ToolsSupport::Remote,..Self::default()};}
  let caps=value["capabilities"].as_array().filter(|caps|caps.iter().all(Value::is_string));
  let tools=if caps.is_some_and(|caps|caps.iter().any(|v|v=="tools")&&caps.iter().any(|v|v=="completion")){ToolsSupport::Confirmed}else{ToolsSupport::Unavailable};
  let valid_name=|v:&Value|v.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64&&!s.chars().any(char::is_control));
  let thinking=value.pointer("/thinking/values").and_then(Value::as_array).filter(|a|a.len()<=32&&a.iter().all(|v|v.is_boolean()||valid_name(v))).cloned().unwrap_or_default();
  let thinking_default=value.pointer("/thinking/default").filter(|v|thinking.contains(v)).cloned();
  Self{tools,thinking,thinking_default,thinking_capable:caps.map(|c|c.iter().any(|v|v=="thinking"))}
 }
 /// V1 documented reasoning_effort, never the native /api/chat `think` field.
 /// Named levels are chosen from this exact model's metadata, not invented.
 pub fn effort(&self,selection:Option<&crate::pet_config::Selection>)->Option<String>{
  let selection=selection?;
  if !selection.effort.is_empty(){
   if self.thinking.iter().any(|v|v==&selection.effort)||self.thinking.contains(&Value::Bool(true))||self.thinking.is_empty()&&self.thinking_capable!=Some(false){return Some(selection.effort.clone());}
   return self.thinking_default.as_ref().and_then(Value::as_str).map(str::to_owned);
  }
  let fast=match selection.mode.as_str(){"fast"=>true,"reasoning"=>false,_=>return None};
  if self.thinking==[Value::Bool(false)]{return None;}
  if self.thinking.contains(&Value::Bool(!fast)){return Some(if fast{"none"}else{"high"}.into());}
  let levels=if fast{["none","minimal","low","medium","high","max","xhigh","ultra"]}else{["ultra","xhigh","max","high","medium","low","minimal","none"]};
  for name in levels{if self.thinking.iter().any(|v|v==name){return Some(name.into());}}
  if let Some(name)=self.thinking_default.as_ref().and_then(Value::as_str){return Some(name.into());}
  if !self.thinking.is_empty(){return None;}
  if self.thinking_capable==Some(false)&&self.thinking.is_empty(){return None;}
  // Older thinking models have no values metadata: documented aliases map to
  // boolean controls. An unavailable metadata endpoint still gets the user's
  // requested control rather than silently defaulting a fast turn to thinking.
  Some(if fast{"none"}else{"high"}.into())
 }
 /// Native /api/chat uses exact metadata names or booleans, independently of
 /// the OpenAI-compatible endpoint's reasoning_effort aliases.
 pub(crate) fn native_think(&self,selection:Option<&crate::pet_config::Selection>)->Option<Value>{
  let fallback=crate::pet_config::Selection{model:String::new(),effort:String::new(),mode:"fast".into()};
  let selection=selection.unwrap_or(&fallback);
  if self.thinking==[Value::Bool(false)]{return Some(Value::Bool(false));}
  let effort=self.effort(Some(selection))?;
  if self.thinking.iter().any(Value::is_string){return self.thinking.iter().find(|v|v.as_str()==Some(effort.as_str())).cloned();}
  match effort.as_str(){
   "none"=>Some(Value::Bool(false)),
   "minimal"|"low"|"medium"|"high"|"max"|"xhigh"|"ultra"=>Some(Value::Bool(true)),
   _=>self.thinking_default.clone().filter(Value::is_boolean),
  }
 }
}
pub async fn capabilities(base:&reqwest::Url,model:&str)->ModelCapabilities{
 let mut observation=crate::pet_capabilities::begin_local(base.as_str(),model);
 let result=async{
  crate::settings::effect_allowed()?;
  let response=crate::net::exact_destination_client(base,Duration::from_secs(5))?.post(crate::net::join(base,"api/show")).json(&json!({"model":model})).send().await.map_err(|_|"metadata unavailable")?;
  if !response.status().is_success(){return Err("metadata unavailable".into());}
  let data=crate::net::read_capped(response,256*1024).await?;
  let value:Value=serde_json::from_slice(&data).map_err(|e|e.to_string())?;
  crate::pet_capabilities::finish_local(observation.take(),Some(&value));
  Ok::<_,String>(ModelCapabilities::from_show(&value))
 }.await;if result.is_err(){crate::pet_capabilities::finish_local(observation,None);}result.unwrap_or_default()
}

pub fn apply_effort(body:&mut Value,effort:Option<&str>){if let Some(effort)=effort{body["reasoning_effort"]=json!(effort);}}
fn request_body(model:&str,messages:&[Value],tools:bool,effort:Option<&str>)->Value{
 let mut body=json!({"model":model,"messages":messages,"stream":true,"max_tokens":4096,"stream_options":{"include_usage":true}});
 if tools{body["tools"]=json!([tool_schema()]);}
 apply_effort(&mut body,effort);body
}
fn calendar_body(model:&str,messages:&[Value],tools:bool,calendar:Option<&crate::pet_calendar::query::Service>,effort:Option<&str>)->Result<Value,String>{
 let mut body=request_body(model,messages,tools,effort);
 if let Some(calendar)=calendar{
  calendar.validate()?;
  if !tools{body["tools"]=json!([]);}
  for definition in calendar.definitions(){body["tools"].as_array_mut().unwrap().push(json!({"type":"function","function":{"name":definition["name"],"description":definition["description"],"parameters":definition["input_schema"]}}));}
 }Ok(body)
}
#[allow(clippy::too_many_arguments)]
pub async fn round(base:&reqwest::Url,model:&str,messages:&[Value],tools:bool,effort:Option<&str>,deadline:Instant,cancel:&std::sync::atomic::AtomicBool,delta:impl FnMut(String))->Result<Round,String>{
 // Compatibility callers have no captured origin and never register attempts.
 round_captured(base,model,messages,tools,effort,deadline,cancel,None,delta).await
}

#[allow(clippy::too_many_arguments)]
pub async fn round_captured(base:&reqwest::Url,model:&str,messages:&[Value],tools:bool,effort:Option<&str>,deadline:Instant,cancel:&std::sync::atomic::AtomicBool,origin:Option<&UsageOrigin>,delta:impl FnMut(String))->Result<Round,String>{
 round_calendar_captured(base,model,messages,tools,None,effort,deadline,cancel,origin,delta).await
}

#[allow(clippy::too_many_arguments)]
pub async fn round_calendar_captured(base:&reqwest::Url,model:&str,messages:&[Value],tools:bool,calendar:Option<&crate::pet_calendar::query::Service>,effort:Option<&str>,deadline:Instant,cancel:&std::sync::atomic::AtomicBool,origin:Option<&UsageOrigin>,mut delta:impl FnMut(String))->Result<Round,String>{
 let began=Instant::now();let body=calendar_body(model,messages,tools,calendar,effort)?;
 // The guard outlives the selected stream future. Stop/deadline can discard that
 // future without discarding counters already observed for this exact attempt.
 let mut capture=None::<Capture>;
 let stream=async{
  let remaining=deadline.saturating_duration_since(Instant::now());if remaining.is_zero(){return Err("O prazo da tarefa terminou.".into());}
  let client=crate::net::exact_destination_client(base,remaining)?;
  let request=client.post(crate::net::join(base,"v1/chat/completions")).header("Authorization","Bearer ollama").json(&body).build().map_err(|_|"Não foi possível contactar o servidor Ollama.".to_string())?;
  crate::settings::effect_allowed()?;
  capture=Some(Capture::begin(origin,model));
  let mut response=client.execute(request).await.map_err(|_|"Não foi possível contactar o servidor Ollama.".to_string())?;
  if !response.status().is_success(){let status=response.status();return Err(format!("O servidor Ollama recusou a ronda (HTTP {}).",status.as_u16()));}
  let mut decoder=Decoder::default();let mut pending=Vec::new();let mut last=Instant::now();
  while let Some(chunk)=response.chunk().await.map_err(|_|"A ligação ao servidor Ollama foi interrompida.")?{
   if pending.len().saturating_add(chunk.len())>2*1024*1024{return Err("O fragmento de resposta excede os limites.".into());}
   pending.extend_from_slice(&chunk);
   while let Some(end)=pending.iter().position(|b|*b==b'\n'){
    let line:Vec<u8>=pending.drain(..=end).collect();let line=std::str::from_utf8(&line).map_err(|_|"Texto inválido na resposta do modelo.")?;decoder.line_captured(line,capture.as_mut())?;
   }
   if pending.len()>1024*1024{return Err("A linha de resposta excede os limites.".into());}
   if last.elapsed()>=Duration::from_millis(67){delta(decoder.visible());last=Instant::now();}
  }
  if !pending.is_empty(){decoder.line_captured(std::str::from_utf8(&pending).map_err(|_|"Texto inválido na resposta do modelo.")?,capture.as_mut())?;}
  let result=decoder.complete_with_calendar(model,began.elapsed(),calendar.is_some())?;delta(result.text.clone());Ok(result)
 };
 let (result,outcome)=tokio::select!{
  result=stream=>{
   let outcome=if result.is_ok(){AttemptOutcome::Completed}
    else if cancel.load(std::sync::atomic::Ordering::Acquire){AttemptOutcome::Cancelled}
    else if Instant::now()>=deadline{AttemptOutcome::Interrupted}
    else{AttemptOutcome::Failed};
   (result,outcome)
  },
  _=async{loop{if cancel.load(std::sync::atomic::Ordering::Acquire){break;}tokio::time::sleep(Duration::from_millis(25)).await;}}=>(Err("A tarefa foi interrompida.".into()),AttemptOutcome::Cancelled),
  _=tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))=>(Err("O prazo da tarefa terminou.".into()),AttemptOutcome::Interrupted)
 };
 if let Some(capture)=capture.as_mut(){capture.finish(outcome);}
 result
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn local_multi_source_body_uses_captured_schemas_without_adding_command_permission(){
  let note=crate::pet_calendar::query::multi_tests::note_fixture();let service=note.query().unwrap();
  for commands in [false,true]{
   let body=calendar_body("same-selected-model",&[],commands,Some(&service),None).unwrap();let tools=body["tools"].as_array().unwrap();assert_eq!(tools.len(),if commands{3}else{2});
   for tool in tools.iter().filter(|t|t["function"]["name"].as_str().is_some_and(crate::pet_calendar::query::is_name)){
    let schema=&tool["function"]["parameters"];assert!(schema["required"].as_array().unwrap().contains(&json!("sourceId")));assert_eq!(schema["properties"]["sourceId"]["enum"].as_array().unwrap().len(),2);assert!(schema["properties"].get("path").is_none());
   }
  }
 }
 #[test]fn calendar_day_keeps_its_native_identity_without_command_authority(){
  let value=json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"day-read","type":"function","function":{"name":"calendar_query_day","arguments":json!({"date":"tomorrow","timeZone":"Europe/Lisbon"}).to_string()}}]},"finish_reason":"tool_calls"}]});
  let mut decoder=Decoder::default();decoder.line(&format!("data: {value}")).unwrap();assert!(decoder.complete_with_calendar("same",Duration::ZERO,false).is_err());
  let mut decoder=Decoder::default();decoder.line(&format!("data: {value}")).unwrap();let round=decoder.complete_with_calendar("same",Duration::ZERO,true).unwrap();assert!(round.calls.is_empty());assert_eq!(round.calendar_calls.len(),1);assert_eq!(round.calendar_calls[0].name,"calendar_query_day");assert_eq!(round.calendar_calls[0].id,"day-read");assert_eq!(round.calendar_calls[0].args["date"],"tomorrow");assert_eq!(round.message["tool_calls"][0]["function"]["name"],"calendar_query_day");
 }
 #[test]fn calendar_and_commands_share_native_ids_but_never_share_command_authority(){
  let event=json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"read","type":"function","function":{"name":"calendar_query","arguments":"{\"start\":\"2026-11-21T00:00:00Z\",\"end\":\"2026-11-22T00:00:00Z\",\"timeZone\":\"Europe/Lisbon\",\"limit\":32}"}},{"index":1,"id":"command","type":"function","function":{"name":"execute_command","arguments":"{\"program\":\"/bin/true\",\"args\":[],\"reason\":\"synthetic\"}"}}]},"finish_reason":"tool_calls"}]});
  let line=format!("data: {event}");let mut decoder=Decoder::default();decoder.line(&line).unwrap();assert!(decoder.complete("same",Duration::ZERO).is_err());
  let mut decoder=Decoder::default();decoder.line(&line).unwrap();let round=decoder.complete_with_calendar("same",Duration::ZERO,true).unwrap();assert_eq!(round.calls.len(),1);assert_eq!(round.calendar_calls.len(),1);assert_eq!(round.calls[0].input.program,"/bin/true");assert_eq!(round.calendar_calls[0].id,"read");assert_eq!(round.message["tool_calls"][0]["id"],"read");assert_eq!(round.message["tool_calls"][1]["id"],"command");
  let mut duplicate=event;duplicate["choices"][0]["delta"]["tool_calls"][1]["id"]=json!("read");let mut decoder=Decoder::default();decoder.line(&format!("data: {duplicate}")).unwrap();assert!(decoder.complete_with_calendar("same",Duration::ZERO,true).is_err());
 }
 fn event(v:Value)->String{format!("data: {v}")}
 #[test]fn fragmented_calls_are_correlated_and_require_complete_round(){let mut d=Decoder::default();d.line(&event(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","type":"function","function":{"name":"execute_","arguments":"{\"program\":\"/bin/true\","}}]}}]}))).unwrap();d.line(&event(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"command","arguments":"\"args\":[],\"reason\":\"r\"}"}}]},"finish_reason":"tool_calls"}]}))).unwrap();let r=d.complete("m",Duration::ZERO).unwrap();assert_eq!(r.calls[0].input.program,"/bin/true");}
 #[test]fn prose_is_never_a_command(){let mut d=Decoder::default();d.line(&event(json!({"choices":[{"delta":{"content":"```sh\nrm x\n```"},"finish_reason":"stop"}]}))).unwrap();assert!(d.complete("m",Duration::ZERO).unwrap().calls.is_empty());}
 #[test]fn incomplete_length_and_unknown_tool_are_nonexecutable(){for end in [None,Some("length"),Some("content_filter")] {let mut d=Decoder::default();d.line(&event(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"execute_command","arguments":"{}"}}]},"finish_reason":end}]}))).unwrap();assert!(d.complete("m",Duration::ZERO).is_err());}}
 #[test]fn oversized_indices_and_malformed_json_are_rejected(){let mut d=Decoder::default();assert!(d.line("data: rubbish").is_err());assert!(d.line(&event(json!({"choices":[{"delta":{"tool_calls":[{"index":4}]}}]}))).is_err());}
 fn profile(mode:&str)->crate::pet_config::Selection{crate::pet_config::Selection{model:"synthetic-fixed:tag".into(),effort:String::new(),mode:mode.into()}}
 #[test]fn native_thinking_uses_exact_metadata_controls_and_keeps_manual_defaults(){
  for show in [json!({"capabilities":["completion","thinking"],"thinking":{"values":[false,true],"default":true}}),json!({"capabilities":["completion","thinking"]})]{let caps=ModelCapabilities::from_show(&show);assert_eq!(caps.native_think(None),Some(json!(false)));assert_eq!(caps.native_think(Some(&profile("fast"))),Some(json!(false)));assert_eq!(caps.native_think(Some(&profile("reasoning"))),Some(json!(true)));assert_eq!(caps.native_think(Some(&profile("manual"))),None);}
  let caps=ModelCapabilities::from_show(&json!({"capabilities":["completion","thinking"],"thinking":{"values":["low","medium","high"],"default":"medium"}}));assert_eq!(caps.native_think(None),Some(json!("low")));assert_eq!(caps.native_think(Some(&profile("reasoning"))),Some(json!("high")));assert_eq!(caps.native_think(Some(&profile("manual"))),None);
  let caps=ModelCapabilities::from_show(&json!({"capabilities":["thinking"],"thinking":{"values":["brief","thorough"],"default":"brief"}}));let mut selected=profile("reasoning");selected.effort="thorough".into();assert_eq!(caps.native_think(Some(&selected)),Some(json!("thorough")));selected.effort="invented".into();assert_eq!(caps.native_think(Some(&selected)),Some(json!("brief")));
  let caps=ModelCapabilities::from_show(&json!({"capabilities":["completion"],"thinking":{"values":[false],"default":false}}));assert_eq!(caps.native_think(Some(&profile("reasoning"))),Some(json!(false)));
 }
 #[test]fn fast_and_reasoning_control_boolean_and_legacy_thinking_without_changing_model(){for show in [json!({"capabilities":["completion","thinking"],"thinking":{"values":[false,true],"default":true}}),json!({"capabilities":["completion","thinking"],"thinking":null})]{let caps=ModelCapabilities::from_show(&show);assert_eq!(caps.effort(Some(&profile("fast"))).as_deref(),Some("none"));assert_eq!(caps.effort(Some(&profile("reasoning"))).as_deref(),Some("high"));assert!(caps.effort(Some(&profile("manual"))).is_none());}}
 #[test]fn named_efforts_use_supported_levels_or_the_declared_custom_default(){let caps=ModelCapabilities::from_show(&json!({"capabilities":["thinking"],"thinking":{"values":["low","medium","high"],"default":"medium"}}));assert_eq!(caps.effort(Some(&profile("fast"))).as_deref(),Some("low"));assert_eq!(caps.effort(Some(&profile("reasoning"))).as_deref(),Some("high"));let caps=ModelCapabilities::from_show(&json!({"thinking":{"values":["brief","thorough"],"default":"brief"}}));assert_eq!(caps.effort(Some(&profile("fast"))).as_deref(),Some("brief"));let caps=ModelCapabilities::from_show(&json!({"thinking":{"values":["brief","thorough"]}}));assert!(caps.effort(Some(&profile("reasoning"))).is_none());}
 #[test]fn nonthinking_models_keep_plain_request_and_v1_never_gets_native_think(){let caps=ModelCapabilities::from_show(&json!({"capabilities":["completion","vision"],"thinking":{"values":[false],"default":false}}));for mode in ["fast","reasoning","manual"]{assert!(caps.effort(Some(&profile(mode))).is_none());}for tools in [false,true]{for effort in [None,Some("none"),Some("high")]{let body=request_body("synthetic-fixed:tag",&[json!({"role":"user","content":"same"})],tools,effort);assert_eq!(body["model"],"synthetic-fixed:tag");assert_eq!(body["reasoning_effort"].as_str(),effort);assert!(body.get("think").is_none());assert_eq!(body.get("tools").is_some(),tools);}}}
 #[test]fn empty_remote_metadata_does_not_mark_a_local_model_as_cloud(){let caps=ModelCapabilities::from_show(&json!({"remote_model":"","remote_host":"","capabilities":["completion","tools"]}));assert_eq!(caps.tools,ToolsSupport::Confirmed);}
 #[test]fn explicit_effort_uses_exact_named_metadata_instead_of_an_ignored_flag(){let caps=ModelCapabilities::from_show(&json!({"capabilities":["thinking"],"thinking":{"values":["brief","thorough"],"default":"brief"}}));let mut selected=profile("reasoning");selected.effort="high".into();assert_eq!(caps.effort(Some(&selected)).as_deref(),Some("brief"));selected.effort="thorough".into();assert_eq!(caps.effort(Some(&selected)).as_deref(),Some("thorough"));}
 #[tokio::test]async fn tools_are_verified_on_the_captured_server_without_inference(){use ToolsSupport::*;for (body,want) in [(json!({"capabilities":["completion","tools"]}),Confirmed),(json!({"capabilities":["completion"]}),Unavailable),(json!({"capabilities":"tools"}),Unavailable),(json!({"capabilities":["completion","tools"],"remote_model":"cloud"}),Remote)]{let address=crate::net::tests::serve_once("200 OK","Content-Type: application/json\r\n",body.to_string().into_bytes());assert_eq!(capabilities(&reqwest::Url::parse(&address).unwrap(),"synthetic").await.tools,want);}}
 #[tokio::test]async fn mock_http_round_preserves_measured_usage_and_filters_reasoning(){let s=event(json!({"model":"synthetic","choices":[{"delta":{"content":"<think>private</think>Public"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}}))+"\n\ndata: [DONE]\n\n";let address=crate::net::tests::serve_once("200 OK","Content-Type: text/event-stream\r\n",s.into_bytes());let r=round(&reqwest::Url::parse(&address).unwrap(),"synthetic",&[],false,None,Instant::now()+Duration::from_secs(2),&std::sync::atomic::AtomicBool::new(false),|_|{}).await.unwrap();assert_eq!(r.text,"Public");assert_eq!(r.usage.total_tokens,Some(5));}
 #[tokio::test]async fn http_eof_without_complete_tool_round_is_never_executable(){let s=event(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"execute_command","arguments":"{\"program\":\"/bin/true\",\"args\":[],\"reason\":\"r\"}"}}]}}]}));let address=crate::net::tests::serve_once("200 OK","Content-Type: text/event-stream\r\n",s.into_bytes());assert!(round(&reqwest::Url::parse(&address).unwrap(),"synthetic",&[],true,None,Instant::now()+Duration::from_secs(2),&std::sync::atomic::AtomicBool::new(false),|_|{}).await.is_err());}
}
