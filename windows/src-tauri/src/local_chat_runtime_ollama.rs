//! Common-runtime text inference only. Ordinary chat/voice retain their
//! existing transport. No daemon settings, model copies, or endpoint fallback.
use super::*;

pub(super)const CONTRACT:&str="ollama-native-runtime-context-v2-calendar-brief";
pub(super)const MAX_CONTEXT:u64=65_536;
const TEMPLATE_ALLOWANCE:u64=1024;

fn context_for(body:&Value)->Result<u64,String>{
 let messages=body["messages"].as_array().ok_or("Faltam as mensagens do pedido nativo.")?;
 if messages.is_empty()||messages.len()>256{return Err("Número de mensagens fora do limite de contexto do runtime.".into());}
 for message in messages{
  if !matches!(message["role"].as_str(),Some("system"|"user"|"assistant"))||!message["content"].is_string()||message.get("tool_calls").is_some()||message.get("images").is_some(){return Err("O contexto nativo do runtime admite apenas mensagens de texto explícitas; nenhum conteúdo foi retirado ou convertido.".into());}
 }
 // A conservative byte-based allowance, not a measured tokenizer count.
 // Keep every original message and leave room for generation and framing.
 let format=match body.get("format"){None=>0,Some(value)=>{
  #[cfg(target_os="linux")]let valid=crate::pet_calendar::brief::valid_schema(value);
  #[cfg(not(target_os="linux"))]let valid=false;
  if !valid{return Err("O formato factual do calendário mudou.".into());}serde_json::to_vec(value).map_err(|_|"Formato inválido.")?.len()as u64
 }};
 let bytes=serde_json::to_vec(messages).map_err(|_|"Contexto de texto inválido.")?.len()as u64+format;
 let need=bytes.checked_add(u64::from(MAX_TOKENS)).and_then(|n|n.checked_add(TEMPLATE_ALLOWANCE)).and_then(|n|n.checked_add(messages.len()as u64*128)).ok_or("Contexto demasiado grande.")?;
 let context=need.checked_add(4095).map(|n|n/4096*4096).filter(|n|*n<=MAX_CONTEXT).ok_or("O pedido completo excede a janela de contexto do runtime Ollama; nenhum dado foi truncado e nenhuma inferência foi reservada.")?;
 Ok(context)
}
pub(super)fn configure(body:&mut Value)->Result<(),String>{
 let context=context_for(body)?;let format=body.get("format").cloned();
 let model=body["model"].as_str().ok_or("Falta o modelo explícito.")?.to_owned();let messages=body["messages"].clone();
 *body=json!({"model":model,"messages":messages,"stream":true,"options":{"num_ctx":context,"num_predict":MAX_TOKENS}});if let Some(format)=format{body["format"]=format;}Ok(())
}
pub(super)fn validate(server:&Server,model:&str,body:&Value)->Result<(),String>{
 if server.id!="ollama"||server.key.is_some()||model.ends_with(":cloud")||model.ends_with("-cloud"){return Err("Este pedido exige o adaptador Ollama local explícito, sem credenciais ou modelo cloud.".into());}
 let base=base_url(server)?;if !net::is_loopback_url(&base){return Err("O contexto do runtime exige o destino Ollama local revisto.".into());}
 let context=context_for(body)?;
 if body["model"]!=model||body["stream"]!=true||body.get("max_tokens").is_some()||body.get("tools").is_some()||body["options"]!=json!({"num_ctx":context,"num_predict":MAX_TOKENS})||body.as_object().is_none_or(|b|b.len()!=4+usize::from(body.get("format").is_some())){return Err("O protocolo ou a janela de contexto do pedido preparado mudou.".into());}
 let limit=crate::pet_capabilities::local_runtime_context_limit(base.as_str(),model,crate::assistant_memory::now()).ok_or("Falta confirmação recente da capacidade de contexto do modelo local. Revê a prontidão; nenhuma inferência foi reservada ou enviada.")?;
 if context>limit{return Err(format!("O pedido requer contexto {context}, acima do máximo confirmado {limit} do modelo escolhido. Nenhum dado foi truncado e nenhum modelo alternativo foi usado."));}Ok(())
}

#[derive(Default)]struct Decoder{ended:bool,text:String,terminal:Option<Value>}
impl Decoder{
 fn line(&mut self,line:&[u8],model:&str,max:usize,capture:&mut Option<Capture>)->Result<(),String>{
  if line.iter().all(u8::is_ascii_whitespace){return Ok(());}if line.len()>MAX_LINE{return Err(t("The server's answer is too large."));}
  if self.ended{return Err("Dados recebidos após o fim do stream nativo.".into());}
  let value:Value=serde_json::from_slice(line).map_err(|_|"Resposta JSON nativa inválida; não repetir inferência.")?;
  if let Some(error)=value.get("error"){return Err(format!("O servidor Ollama recusou o pedido: {}",error.as_str().unwrap_or("erro nativo")));}
  if value["model"].as_str()!=Some(model)||!value["done"].is_boolean(){return Err("O modelo ou o marcador de conclusão nativo não corresponde ao pedido revisto.".into());}
  let message=&value["message"];if !message.is_object()||message["role"].as_str()!=Some("assistant")||message.get("tool_calls").is_some_and(|v|!v.is_null()&&v.as_array().is_none_or(|a|!a.is_empty())){return Err("Envelope nativo inesperado; não foi concedida execução de ferramentas.".into());}
  let text=message["content"].as_str().ok_or("Texto nativo inválido.")?;
  if text.len()>max.saturating_sub(self.text.len()){return Err(t("The server's answer is too large."));}self.text.push_str(text);
  if value["done"]==true{
   if !matches!(value["done_reason"].as_str(),Some("stop"|"length")){return Err("Falta a razão final do pedido nativo; não repetir inferência.".into());}
   self.ended=true;self.terminal=Some(value.clone());
  }
  if let Some(capture)=capture{capture.observe_native(&value);}Ok(())
 }
}

pub(super)async fn stream<F:std::future::Future<Output=Result<(),String>>>(server:&Server,model:&str,body:&Value,origin:Option<&CaptureOrigin>,capture:&mut Option<Capture>,mut on_delta:impl FnMut(String),max_output:usize,before_send:impl FnOnce()->F)->Result<MeasuredReply,String>{
 if max_output==0||max_output>MAX_ANSWER{return Err("Limite de resposta inválido.".into());}validate(server,model,body)?;
 let base=base_url(server)?;let client=net::local_client(&base,Duration::from_secs(300))?;
 let request=client.post(net::join(&base,"api/chat")).json(body).build().map_err(|_|unreachable(&base))?;
 crate::settings::effect_allowed()?;before_send().await?;*capture=Some(Capture::begin(origin,model));let began=Instant::now();
 let mut response=client.execute(request).await.map_err(|_|unreachable(&base))?;
 if !response.status().is_success(){let status=response.status();let bytes=net::read_capped(response,net::MAX_ERROR_BODY).await.unwrap_or_default();return Err(format!("HTTP {}: {}",status.as_u16(),net::error_detail(&bytes)));}
 let mut pending=Vec::new();let mut decoder=Decoder::default();let mut last=Instant::now().checked_sub(DELTA_INTERVAL).unwrap_or_else(Instant::now);
 while let Some(chunk)=response.chunk().await.map_err(|_|unreachable(&base))?{
  pending.extend_from_slice(&chunk);
  while let Some(end)=pending.iter().position(|b|*b==b'\n'){
   let line:Vec<u8>=pending.drain(..=end).collect();decoder.line(&line,model,max_output,capture)?;
  }
  if pending.len()>MAX_LINE{return Err(t("The server's answer is too large."));}
  if last.elapsed()>=DELTA_INTERVAL{last=Instant::now();on_delta(progressive_filter(&decoder.text));}
 }
 decoder.line(&pending,model,max_output,capture)?;on_delta(progressive_filter(&decoder.text));
 let usage=decoder.terminal.as_ref().map(|v|crate::pet_usage::Usage::native(v,model,began.elapsed())).unwrap_or_default();
 Ok(MeasuredReply{protocol_complete:decoder.ended,accepted_output_bytes:decoder.text.len()as u64,text:filter_thinking_blocks(&decoder.text),usage})
}

#[cfg(test)]mod tests{
 use super::*;
 fn body()->Value{json!({"model":"context-fixture","messages":[{"role":"system","content":"Keep the selected facts"},{"role":"user","content":"Dados novos: ÓRBITA-93 às09:00"},{"role":"user","content":"Resume apenas os dados selecionados"}],"stream":true,"max_tokens":MAX_TOKENS})}
 fn observed(url:String,limit:Option<u64>)->Server{
  let server=Server{id:"ollama",name:"Private context fixture",url,key:None};let base=base_url(&server).unwrap();let ticket=crate::pet_capabilities::begin_local(base.as_str(),"context-fixture");let mut show=json!({"model_info":{"general.architecture":"synthetic"},"capabilities":["completion"]});if let Some(limit)=limit{show["model_info"]["synthetic.context_length"]=json!(limit);}crate::pet_capabilities::finish_local(ticket,Some(&show));server
 }
 #[test]fn native_request_keeps_every_selected_message_with_explicit_bounded_context(){
  let mut b=body();let messages=b["messages"].clone();configure(&mut b).unwrap();assert_eq!(b["messages"],messages);assert_eq!(b["options"],json!({"num_ctx":8192,"num_predict":4096}));assert!(b.get("max_tokens").is_none());let again=b.clone();configure(&mut b).unwrap();assert_eq!(b,again);
  b["messages"][1]["content"]=json!("novo".repeat(4000));configure(&mut b).unwrap();assert!(b["options"]["num_ctx"].as_u64().unwrap()>8192);assert!(b["options"]["num_ctx"].as_u64().unwrap()<=MAX_CONTEXT);
 }
 #[test]fn unknown_small_remote_or_changed_capacity_never_authorizes_a_model_request(){
  let mut b=body();configure(&mut b).unwrap();let s=observed("http://127.0.0.1:32241".into(),None);assert!(validate(&s,"context-fixture",&b).is_err());
  let s=observed("http://127.0.0.1:32242".into(),Some(4096));assert!(validate(&s,"context-fixture",&b).is_err());let s=observed("http://127.0.0.1:32243".into(),Some(8192));assert!(validate(&s,"context-fixture",&b).is_ok());
  for mode in ["model","context","predict","legacy","tools"]{let mut forged=b.clone();match mode{"model"=>forged["model"]=json!("another"),"context"=>forged["options"]["num_ctx"]=json!(65536),"predict"=>forged["options"]["num_predict"]=json!(4097),"legacy"=>forged["max_tokens"]=json!(4096),_=>forged["tools"]=json!([])}assert!(validate(&s,"context-fixture",&forged).is_err(),"{mode}");}
  let mut remote=s;remote.url="https://example.invalid".into();assert!(validate(&remote,"context-fixture",&b).is_err());
 }
 #[test]fn oversized_or_nontext_input_is_refused_without_slicing_selected_data(){
  let mut b=body();b["messages"][1]["content"]=json!("x".repeat(65536));let original=b.clone();assert!(configure(&mut b).is_err());assert_eq!(b,original);
  for content in [json!([{"type":"image_url","image_url":{"url":"private"}}]),json!(null)]{let mut b=body();b["messages"][1]["content"]=content;let original=b.clone();assert!(configure(&mut b).is_err());assert_eq!(b,original);}
 }
 fn line(content:&str,done:bool)->Vec<u8>{let mut value=json!({"model":"context-fixture","message":{"role":"assistant","content":content},"done":done});if done{value["done_reason"]=json!("stop");value["prompt_eval_count"]=json!(123);value["eval_count"]=json!(9);}serde_json::to_vec(&value).unwrap()}
 #[test]fn native_completion_and_counters_are_distinct_from_eof_or_public_semantics(){
  let mut d=Decoder::default();d.line(&line("Olá ",false),"context-fixture",100,&mut None).unwrap();assert!(!d.ended);d.line(&line("mundo",true),"context-fixture",100,&mut None).unwrap();assert!(d.ended);assert_eq!(d.text,"Olá mundo");assert_eq!(d.terminal.unwrap()["prompt_eval_count"],123);
 }
 #[test]fn malformed_foreign_tool_postfinal_and_oversized_streams_cannot_finish_a_reply(){
  for value in [json!({"model":"foreign","message":{"role":"assistant","content":"wrong"},"done":true,"done_reason":"stop"}),json!({"model":"context-fixture","message":{"role":"assistant","content":"","tool_calls":[{}]},"done":true,"done_reason":"stop"}),json!({"model":"context-fixture","message":{"role":"assistant","content":"x"},"done":true})]{assert!(Decoder::default().line(&serde_json::to_vec(&value).unwrap(),"context-fixture",10,&mut None).is_err());}
  assert!(Decoder::default().line(b"data: [DONE]","context-fixture",10,&mut None).is_err());assert!(Decoder::default().line(&line("too long",false),"context-fixture",2,&mut None).is_err());let mut d=Decoder::default();d.line(&line("end",true),"context-fixture",10,&mut None).unwrap();assert!(d.line(&line("late",false),"context-fixture",10,&mut None).is_err());
 }
 #[test]fn native_http_consumes_reported_counters_and_requires_the_real_terminal_frame(){
  let mut raw=line("Resposta real do protocolo",false);raw.push(b'\n');raw.extend(line("",true));raw.push(b'\n');let url=crate::net::tests::serve_once("200 OK","Content-Type: application/x-ndjson\r\n",raw);let s=observed(url,Some(32768));let mut b=body();configure(&mut b).unwrap();let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  let r=runtime.block_on(stream(&s,"context-fixture",&b,None,&mut None,|_|{},1024,||async{Ok(())})).unwrap();assert!(r.protocol_complete);assert_eq!(r.text,"Resposta real do protocolo");assert_eq!(r.usage.input_tokens,Some(123));assert_eq!(r.usage.output_tokens,Some(9));
 }
}
