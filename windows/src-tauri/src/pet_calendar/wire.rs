//! Native function-call envelopes. No prose is ever interpreted as a tool.
use super::{query::{self, Service}, Note};
use serde_json::{json, Value};
use std::{collections::HashSet,time::{Duration,Instant}};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format { Chat, Responses, Claude }
pub(crate) struct Session {
    service: Service,
    format: Format,
    seen: HashSet<String>,
    rounds: usize,
    deadline: Instant,
}
struct Call { id: String, args: Value, name:String }
impl Session {
    pub(crate) fn new(note: Option<&Note>, format: Format) -> Option<Self> {
        Some(Self {service:note?.query()?,format,seen:HashSet::new(),rounds:0,deadline:Instant::now()+Duration::from_secs(120)})
    }
    pub(crate) fn validate(&self) -> Result<(),String> {
        self.service.validate()?;
        if Instant::now() >= self.deadline { return Err("O tempo da consulta de calendário terminou.".into()); }
        Ok(())
    }
    pub(crate) fn prepare(&self, body: &mut Value) -> Result<(),String> {
        self.validate()?;
        let definitions=self.service.definitions();
        let functions:Vec<Value>=definitions.iter().map(|d|json!({"name":d["name"],"description":d["description"],"parameters":d["input_schema"],"strict":true})).collect();
        match self.format {
            Format::Chat => {
                for function in &functions{append_tool(body,json!({"type":"function","function":function}))?;}
            }
            Format::Claude => {for definition in &definitions{append_tool(body,definition.clone())?;}}
            Format::Responses => {
                let messages = body["messages"].as_array().ok_or("Histórico inválido para a consulta.")?;
                let mut input = Vec::new();
                let mut instructions = String::new();
                for message in messages {
                    if message["role"] == "system" { instructions.push_str(message["content"].as_str().ok_or("Instruções inválidas.")?); continue; }
                    let mut message = message.clone();
                    let assistant = message["role"] == "assistant";
                    if let Some(parts) = message["content"].as_array_mut() {
                        for part in parts {
                            match part["type"].as_str() {
                                Some("text") => part["type"] = json!(if assistant {"output_text"} else {"input_text"}),
                                Some("image_url") if !assistant => {
                                    *part = json!({"type":"input_image","image_url":part["image_url"]["url"],"detail":part["image_url"].get("detail").cloned().unwrap_or(json!("auto"))});
                                }
                                _ => return Err("O contexto guardado não é suportado por esta consulta.".into()),
                            }
                        }
                    }
                    input.push(message);
                }
                *body = json!({"model":body["model"],"instructions":instructions,"input":input,"max_output_tokens":4096,"store":false,"include":["reasoning.encrypted_content"],
                    "tools":functions.iter().map(|f|json!({"type":"function","name":f["name"],"description":f["description"],"parameters":f["parameters"],"strict":true})).collect::<Vec<_>>()});
            }
        }
        Ok(())
    }
    fn calls(&self, response: &Value) -> Result<Vec<Call>,String> {
        let mut calls = Vec::new();
        match self.format {
            Format::Chat => {
                let choices = response["choices"].as_array().filter(|c| c.len()==1).ok_or("Resposta de consulta inválida.")?;
                let message = &choices[0]["message"];
                if let Some(raw) = message.get("tool_calls").filter(|v| !v.is_null()) {
                    for call in raw.as_array().ok_or("Chamadas de consulta inválidas.")? {
                        if call["type"] != "function" || !call["function"]["name"].as_str().is_some_and(query::is_name) { return Err("Ferramenta de consulta desconhecida.".into()); }
                        calls.push(parse_call(&call["id"],&call["function"]["arguments"],true,call["function"]["name"].as_str().unwrap())?);
                    }
                }
                let finish = choices[0]["finish_reason"].as_str();
                if finish != Some(if calls.is_empty() {"stop"} else {"tool_calls"}) { return Err("A ronda de consulta ficou incompleta.".into()); }
            }
            Format::Responses => {
                if response["status"] != "completed" { return Err("A ronda de consulta ficou incompleta.".into()); }
                for item in response["output"].as_array().ok_or("Resposta de consulta inválida.")? {
                    if item["type"] == "function_call" {
                        if !item["name"].as_str().is_some_and(query::is_name) || item.get("status").is_some_and(|s| s != "completed") { return Err("Ferramenta de consulta desconhecida ou incompleta.".into()); }
                        calls.push(parse_call(&item["call_id"],&item["arguments"],true,item["name"].as_str().unwrap())?);
                    }
                }
            }
            Format::Claude => {
                for block in response["content"].as_array().ok_or("Resposta de consulta inválida.")? {
                    if block["type"] == "tool_use" {
                        if !block["name"].as_str().is_some_and(query::is_name) { return Err("Ferramenta de consulta desconhecida.".into()); }
                        calls.push(parse_call(&block["id"],&block["input"],false,block["name"].as_str().unwrap())?);
                    }
                }
                if !calls.is_empty() && response["stop_reason"] != "tool_use" { return Err("A ronda de consulta ficou incompleta.".into()); }
                if calls.is_empty() && response["stop_reason"] == "tool_use" { return Err("Falta a chamada de consulta.".into()); }
                if response["stop_reason"] == "pause_turn" { return Err("A pesquisa do fornecedor ficou suspensa. O pedido não foi repetido.".into()); }
            }
        }
        let mut ids = HashSet::new();
        if calls.len()>4 || calls.iter().any(|c| self.seen.contains(&c.id) || !ids.insert(c.id.clone())) { return Err("Identidade ou limite de consultas inválido.".into()); }
        Ok(calls)
    }
    pub(crate) fn validate_response(&self,response:&Value)->Result<(),String>{
        self.validate()?;self.calls(response).map(|_|())
    }
    /// Freeze/validate the complete batch before the first read. Internal tool
    /// exchanges stay in this turn, never as unresolved public-history items.
    pub(crate) async fn follow_up(&mut self, response: &Value, body: &mut Value) -> Result<bool,String> {
        self.validate()?;
        let calls = self.calls(response)?;
        if calls.is_empty() { return Ok(false); }
        if self.rounds>=4 { return Err("Foi atingido o limite de rondas da consulta.".into()); }
        self.rounds+=1;
        let mut results = Vec::new();
        for call in calls {
            self.validate()?;
            let output = crate::net::while_current(||self.validate().is_ok(),||self.service.call_named(&call.name,call.args)).await?;
            self.validate()?;
            self.seen.insert(call.id.clone());
            results.push((call.id,output));
        }
        match self.format {
            Format::Chat => {
                let messages = body["messages"].as_array_mut().ok_or("Histórico inválido.")?;
                messages.push(response["choices"][0]["message"].clone());
                for (id,output) in results { messages.push(json!({"role":"tool","tool_call_id":id,"content":output})); }
            }
            Format::Responses => {
                let input = body["input"].as_array_mut().ok_or("Histórico inválido.")?;
                input.extend(response["output"].as_array().ok_or("Resposta inválida.")?.iter().cloned());
                for (id,output) in results { input.push(json!({"type":"function_call_output","call_id":id,"output":output})); }
            }
            Format::Claude => {
                let messages = body["messages"].as_array_mut().ok_or("Histórico inválido.")?;
                messages.push(json!({"role":"assistant","content":response["content"]}));
                messages.push(json!({"role":"user","content":results.into_iter().map(|(id,output)| json!({"type":"tool_result","tool_use_id":id,"content":output})).collect::<Vec<_>>()}));
            }
        }
        Ok(true)
    }
}
fn append_tool(body: &mut Value, tool: Value) -> Result<(),String> {
    if body.get("tools").is_none() { body["tools"] = json!([]); }
    body["tools"].as_array_mut().ok_or("Definições de ferramentas inválidas.")?.push(tool);
    Ok(())
}
fn parse_call(id: &Value, args: &Value, encoded: bool,name:&str) -> Result<Call,String> {
    let id = id.as_str().filter(|s| !s.is_empty() && s.len()<=128 && !s.chars().any(char::is_control)).ok_or("Identidade de consulta inválida.")?;
    if serde_json::to_vec(args).map_err(|_| "Argumentos inválidos.")?.len()>4096 { return Err("Os argumentos da consulta excedem os limites.".into()); }
    let args = if encoded { serde_json::from_str(args.as_str().ok_or("Argumentos inválidos.")?).map_err(|_| "Argumentos inválidos.")? } else {args.clone()};
    if !args.is_object() { return Err("Argumentos inválidos.".into()); }
    Ok(Call {id:id.into(),args,name:name.into()})
}
pub(crate) fn responses_text(response: &Value) -> Result<String,String> {
    let mut text = String::new();
    for item in response["output"].as_array().ok_or("Resposta inválida.")? {
        if item["type"] != "message" { continue; }
        for block in item["content"].as_array().ok_or("Conteúdo inválido.")? {
            if block["type"] == "output_text" { text.push_str(block["text"].as_str().ok_or("Texto inválido.")?); }
            if block["type"] == "refusal" { return Err("O fornecedor recusou o pedido.".into()); }
        }
    }
    if text.trim().is_empty() { return Err("O fornecedor não devolveu texto.".into()); }
    Ok(text.trim().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
    fn args() -> Value { json!({"start":"2026-11-21T00:00:00Z","end":"2026-11-22T00:00:00Z","timeZone":"Europe/Lisbon","limit":128}) }
    fn fixture(format:Format) -> (crate::pet_calendar::Guard,Session,Arc<AtomicUsize>) {
        let count=Arc::new(AtomicUsize::new(0));let observed=count.clone();
        let (guard,service,_,_)=crate::pet_calendar::query::tests::fixture(move|query,_|{assert_eq!(query.start,"2026-11-21T00:00:00Z");observed.fetch_add(1,Ordering::AcqRel);Ok(crate::pet_calendar::unavailable("synthetic"))});
        (guard,Session {service,format,seen:HashSet::new(),rounds:0,deadline:Instant::now()+Duration::from_secs(2)},count)
    }
    fn response(format:Format,id:&str) -> Value {
        match format {
            Format::Chat=>json!({"choices":[{"index":0,"message":{"role":"assistant","content":null,"tool_calls":[{"id":id,"type":"function","function":{"name":query::NAME,"arguments":args().to_string()},"extra_content":{"google":{"thought_signature":"synthetic-preserved"}}}]},"finish_reason":"tool_calls"}]}),
            Format::Claude=>json!({"stop_reason":"tool_use","content":[{"type":"tool_use","id":id,"name":query::NAME,"input":args()}]}),
            Format::Responses=>json!({"status":"completed","output":[{"type":"reasoning","id":"synthetic","summary":[],"encrypted_content":"synthetic-opaque-encrypted-state"},{"type":"function_call","call_id":id,"name":query::NAME,"arguments":args().to_string(),"status":"completed"}]}),
        }
    }
    fn body() -> Value { json!({"model":"same-selected-model","max_tokens":4096,"messages":[{"role":"system","content":"same identity"},{"role":"user","content":"other dates"}]}) }
    #[tokio::test]
    async fn every_api_envelope_preserves_the_explicit_second_calendar_selection(){
        for format in [Format::Chat,Format::Responses,Format::Claude]{
            let(_guard,service,_,_)=query::multi_tests::fixture(|source,query,_|Ok(query::multi_tests::ready(source,query)));
            let mut session=Session{service,format,seen:HashSet::new(),rounds:0,deadline:Instant::now()+Duration::from_secs(3)};let mut body=body();session.prepare(&mut body).unwrap();
            for definition in body["tools"].as_array().unwrap(){
                let schema=match format{Format::Chat=>&definition["function"]["parameters"],Format::Responses=>&definition["parameters"],Format::Claude=>&definition["input_schema"]};
                assert!(schema["required"].as_array().unwrap().contains(&json!("sourceId")));assert_eq!(schema["properties"]["sourceId"]["enum"].as_array().unwrap().len(),2);
            }
            let args=query::multi_tests::arguments('b');let mut reply=response(format,"explicit-second-source");
            match format{
                Format::Chat=>reply["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]=json!(args.to_string()),
                Format::Responses=>reply["output"][1]["arguments"]=json!(args.to_string()),
                Format::Claude=>reply["content"][0]["input"]=args,
            }
            assert!(session.follow_up(&reply,&mut body).await.unwrap());
            let output=match format{Format::Chat=>&body["messages"][3]["content"],Format::Responses=>&body["input"][3]["output"],Format::Claude=>&body["messages"][3]["content"][0]["content"]};
            let result:Value=serde_json::from_str(output.as_str().unwrap()).unwrap();assert_eq!(result["source"],query::multi_tests::source('b'));assert_eq!(result["events"][0]["summary"],"revision-b");
        }
    }
    #[cfg(target_os="linux")]
    #[tokio::test]
    async fn native_day_dispatch_is_shared_by_chat_responses_and_claude(){
        let _serial=crate::pet_calendar::process::TEST_SERIAL.lock().unwrap();
        for format in [Format::Chat,Format::Responses,Format::Claude]{
            let(guard,service,_,_)=crate::pet_calendar::query::tests::fixture(|q,_|{
                assert_eq!(query::instant(&q.start),query::instant("2026-10-24T23:00:00Z"));assert_eq!(query::instant(&q.end),query::instant("2026-10-26T00:00:00Z"));assert_eq!(q.limit,32);Ok(json!({"status":"ready","events":[],"syntheticDay":true}))
            });
            let mut session=Session{service,format,seen:HashSet::new(),rounds:0,deadline:Instant::now()+Duration::from_secs(3)};
            let mut body=body();session.prepare(&mut body).unwrap();
            let definitions=body["tools"].as_array().unwrap();assert_eq!(definitions.len(),2);
            let day=json!({"date":"2026-10-25","timeZone":"Europe/Lisbon"});let mut reply=response(format,"same-day-id");
            match format{
                Format::Chat=>{reply["choices"][0]["message"]["tool_calls"][0]["function"]["name"]=json!(super::super::day::NAME);reply["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]=json!(day.to_string());}
                Format::Responses=>{reply["output"][1]["name"]=json!(super::super::day::NAME);reply["output"][1]["arguments"]=json!(day.to_string());}
                Format::Claude=>{reply["content"][0]["name"]=json!(super::super::day::NAME);reply["content"][0]["input"]=day;}
            }
            assert!(session.follow_up(&reply,&mut body).await.unwrap());
            let result=match format{Format::Chat=>&body["messages"][3]["content"],Format::Responses=>&body["input"][3]["output"],Format::Claude=>&body["messages"][3]["content"][0]["content"]};
            assert_eq!(serde_json::from_str::<Value>(result.as_str().unwrap()).unwrap()["syntheticDay"],true);drop(guard);
        }
    }
    #[tokio::test]
    async fn native_envelopes_pair_exact_ids_and_preserve_model_reasoning_and_vendor_extensions() {
        for format in [Format::Chat,Format::Claude,Format::Responses] {
            let (_guard,mut session,count)=fixture(format);let mut body=body();
            if format==Format::Claude {body["tools"]=json!([{"name":"web_search","type":"web_search_20260209"}]);}
            session.prepare(&mut body).unwrap();assert_eq!(body["model"],"same-selected-model");
            assert!(session.follow_up(&response(format,"exact-call"),&mut body).await.unwrap());assert_eq!(count.load(Ordering::Acquire),1);
            match format {
                Format::Chat=>{assert_eq!(body["messages"][3]["tool_call_id"],"exact-call");assert_eq!(body["messages"][2]["tool_calls"][0]["extra_content"]["google"]["thought_signature"],"synthetic-preserved");}
                Format::Claude=>{assert_eq!(body["tools"][0]["name"],"web_search");assert_eq!(body["messages"][3]["content"][0]["tool_use_id"],"exact-call");}
                Format::Responses=>{assert_eq!(body["input"][1]["type"],"reasoning");assert_eq!(body["input"][1]["encrypted_content"],"synthetic-opaque-encrypted-state");assert_eq!(body["include"],json!(["reasoning.encrypted_content"]));assert_eq!(body["input"][3]["call_id"],"exact-call");assert_eq!(body["store"],false);assert_eq!(body["instructions"],"same identity");}
            }
            // No repeated ID can read again, even if all arguments are valid.
            assert!(session.follow_up(&response(format,"exact-call"),&mut body).await.is_err());assert_eq!(count.load(Ordering::Acquire),1);
        }
    }
    #[test]
    fn responses_preserve_multimodal_history_and_keep_stateless_reasoning_explicit(){
        let (_guard,session,_)=fixture(Format::Responses);
        let mut body=body();body["messages"]=json!([
            {"role":"system","content":"same identity"},
            {"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,c3ludGhldGlj","detail":"low"}},{"type":"text","text":"prior picture"}]},
            {"role":"assistant","content":[{"type":"text","text":"prior answer"}]},
            {"role":"user","content":"query"}]);
        session.prepare(&mut body).unwrap();
        assert_eq!(body["model"],"same-selected-model");assert_eq!(body["store"],false);
        assert_eq!(body["include"],json!(["reasoning.encrypted_content"]));
        assert_eq!(body["input"][0]["content"][0],json!({"type":"input_image","image_url":"data:image/png;base64,c3ludGhldGlj","detail":"low"}));
        assert_eq!(body["input"][0]["content"][1]["type"],"input_text");
        assert_eq!(body["input"][1]["content"][0]["type"],"output_text");
        assert_eq!(body["instructions"],"same identity");assert!(body.get("previous_response_id").is_none());
    }
    #[tokio::test]
    async fn malformed_later_call_prevents_every_read_in_the_batch() {
        let (_guard,mut session,count)=fixture(Format::Chat);let mut body=body();session.prepare(&mut body).unwrap();
        let mut response=response(Format::Chat,"first");let mut second=response["choices"][0]["message"]["tool_calls"][0].clone();second["id"]=json!("second");second["function"]["name"]=json!("execute_command");response["choices"][0]["message"]["tool_calls"].as_array_mut().unwrap().push(second);
        let before=body.clone();assert!(session.follow_up(&response,&mut body).await.is_err());assert_eq!(count.load(Ordering::Acquire),0);assert_eq!(body,before);
    }
    #[test]
    fn partial_envelopes_and_unknown_tools_never_become_queries() {
        for format in [Format::Chat,Format::Claude,Format::Responses] {
            let (_guard,session,count)=fixture(format);let mut response=response(format,"one");
            match format {Format::Chat=>response["choices"][0]["finish_reason"]=json!("length"),Format::Claude=>response["stop_reason"]=json!("max_tokens"),Format::Responses=>response["status"]=json!("incomplete")};
            assert!(session.calls(&response).is_err());assert_eq!(count.load(Ordering::Acquire),0);
        }
    }
    #[tokio::test]
    async fn bounded_rounds_and_total_deadline_apply_before_another_read() {
        let (_guard,mut session,count)=fixture(Format::Chat);let mut body=body();session.prepare(&mut body).unwrap();
        for n in 0..4 {assert!(session.follow_up(&response(Format::Chat,&format!("call{n}")),&mut body).await.unwrap());}
        assert!(session.follow_up(&response(Format::Chat,"fifth"),&mut body).await.is_err());assert_eq!(count.load(Ordering::Acquire),4);
        session.deadline=Instant::now()-Duration::from_secs(1);assert!(session.validate().is_err());
    }
    #[test]
    fn responses_answer_preserves_only_public_text_and_reports_refusal() {
        let response=json!({"output":[{"type":"reasoning","summary":[{"text":"not public"}]},{"type":"message","content":[{"type":"output_text","text":"public answer"}]}]});
        assert_eq!(responses_text(&response).unwrap(),"public answer");assert!(responses_text(&json!({"output":[{"type":"message","content":[{"type":"refusal","refusal":"no"}]}]})).is_err());
        assert!(Session::new(None,Format::Chat).is_none());
    }
}
