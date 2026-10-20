//! HTTP Connect JSON contract from the installed agy 1.4.2 descriptors.
//! Native endpoint/CSRF are inherited by a hook, kept in this backend only.
use std::time::Duration;
use reqwest::{Client as Http,Url,header::{HeaderMap,HeaderValue}};
use serde_json::{json,Value};
use crate::session_chat::types::{Target,Origin,Snapshot,Message,Role,Capabilities};

const SERVICE:&str="/exa.language_server_pb.LanguageServerService/";
const MAX_HISTORY:usize=128*1024*1024;

#[derive(Debug)]
pub struct HistoryError { message:String, transient:bool }
impl HistoryError {
    fn changing(message:&str)->Self{Self{message:message.into(),transient:true}}
    pub fn transient(&self)->bool{self.transient}
}
impl std::fmt::Display for HistoryError {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.message)}
}
impl From<String> for HistoryError {fn from(message:String)->Self{Self{message,transient:false}}}
impl From<&str> for HistoryError {fn from(message:&str)->Self{message.to_string().into()}}
#[derive(Debug)]
pub enum DeliveryError {NotSent(String),Unknown(String)}


/// Deliberately not Debug or Serialize: the CSRF capability must not reach logs,
/// Tauri events, argv, a saved transcript, or the runtime metadata file.
pub struct Captured {pub session_id:String,pub cwd:String,address:String,csrf:String}
impl Captured {
    pub fn from_hook(raw:&Value)->Result<Option<Self>,String>{
        let(Some(address),Some(csrf))=(std::env::var("ANTIGRAVITY_LS_ADDRESS").ok(),std::env::var("ANTIGRAVITY_CSRF_TOKEN").ok())else{return Ok(None);};
        let session_id=raw["conversationId"].as_str().ok_or("O hook Gravity não identificou a conversa.")?.to_string();
        target(&session_id).validate_external("antigravity","agent_antigravity")?;
        endpoint(&address)?;
        if csrf.is_empty()||csrf.len()>4096||csrf.chars().any(char::is_control){return Err("O hook Gravity não forneceu uma capacidade local válida.".into());}
        let cwd=raw["workspacePaths"][0].as_str().filter(|p|std::path::Path::new(p).is_absolute()).unwrap_or("").into();
        Ok(Some(Self{session_id,cwd,address,csrf}))
    }

    pub fn connect(self)->Result<Native,String>{
        let base=endpoint(&self.address)?;
        let mut headers=HeaderMap::new();
        let mut token=HeaderValue::from_str(&self.csrf).map_err(|_|"A capacidade local Gravity é inválida.")?;token.set_sensitive(true);
        headers.insert("x-codeium-csrf-token",token);
        let http=Http::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(3)).timeout(Duration::from_secs(20)).default_headers(headers).build().map_err(|_|"Não foi possível preparar a ligação local Gravity.")?;
        Ok(Native{http,base,session_id:self.session_id,cwd:self.cwd})
    }
}

pub struct Native{http:Http,base:Url,pub session_id:String,pub cwd:String}
fn target(session:&str)->Target{Target{pet_id:"agent_antigravity".into(),provider:"antigravity".into(),origin:Origin::External,session_id:session.into(),conversation_id:None}}
fn endpoint(address:&str)->Result<Url,String>{
    let base=Url::parse(&format!("http://{address}")).map_err(|_|"O endereço local Gravity é inválido.")?;
    let local=base.host_str().is_some_and(|h|h=="localhost"||h=="[::1]"||h=="::1"||h.parse::<std::net::IpAddr>().is_ok_and(|ip|ip.is_loopback()));
    if !local||base.port().is_none()||!base.username().is_empty()||base.password().is_some()||base.path()!="/"||base.query().is_some()||base.fragment().is_some(){return Err("O endpoint Gravity tem de ser o servidor local identificado pelo hook nativo.".into());}
    Ok(base)
}

impl Native{
    async fn call(&self,method:&str,params:Value)->Result<Value,String>{
        let url=self.base.join(&format!("{SERVICE}{method}")).map_err(|_|"O método Gravity não está disponível.")?;
        let mut response=self.http.post(url).header("Content-Type","application/json").header("Connect-Protocol-Version","1").json(&params).send().await.map_err(|_|"A ligação à sessão Gravity terminou ou não respondeu. O pedido não será reenviado.".to_string())?;
        if !response.status().is_success(){return Err(format!("O servidor Gravity recusou {method} (HTTP{}). Atualiza a sessão antes de continuar.",response.status().as_u16()));}
        let mut bytes=Vec::new();
        while let Some(chunk)=response.chunk().await.map_err(|_|"A resposta Gravity foi interrompida.")?{
            if bytes.len().saturating_add(chunk.len())>MAX_HISTORY{return Err("O histórico Gravity excede128 MiB; não foi apresentado um recorte como conversa completa.".into());}bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_|"O servidor Gravity não devolveu o protocolo JSON esperado.".into())
    }

    async fn metadata(&self)->Result<Value,String>{
        let value=self.call("GetConversationMetadata",json!({"conversationId":self.session_id})).await?;
        let id=value["trajectoryId"].as_str().filter(|s|!s.is_empty()&&s.len()<=128).ok_or("O servidor Gravity não confirmou a trajetória desta conversa.")?;
        if id.chars().any(char::is_control){return Err("O servidor Gravity devolveu uma trajetória inválida.".into());}Ok(value)
    }

    async fn trajectory(&self)->Result<(Value,Vec<Value>,Value),HistoryError>{
        let before=self.metadata().await?;
        let response=self.call("GetCascadeTrajectory",json!({"cascadeId":self.session_id,"trajectoryVerbosity":"CLIENT_TRAJECTORY_VERBOSITY_FULL"})).await?;
        let trajectory=&response["trajectory"];
        if trajectory["cascadeId"]!=self.session_id{return Err("A trajetória recebida não corresponde à sessão Gravity escolhida. Atualiza o histórico.".into());}
        if trajectory["trajectoryId"]!=before["trajectoryId"]{return Err(HistoryError::changing("A versão da conversa Gravity mudou durante a leitura. Atualiza o histórico."));}
        let total=number(&response["numTotalSteps"]).unwrap_or(0);
        let mut steps=trajectory["steps"].as_array().cloned().unwrap_or_default();
        if total>1_000_000||steps.len()>total{return Err("O servidor Gravity não confirmou a extensão completa do histórico.".into());}
        let mut bytes=serde_json::to_vec(&steps).map_err(|_|"Histórico Gravity inválido.")?.len();
        while steps.len()<total{
            let page=self.call("GetCascadeTrajectorySteps",json!({"cascadeId":self.session_id,"stepOffset":steps.len(),"trajectoryVerbosity":"CLIENT_TRAJECTORY_VERBOSITY_FULL"})).await?;
            let data=page["steps"].as_array().filter(|v|!v.is_empty()).ok_or("O servidor Gravity não forneceu todos os passos da conversa.")?;
            bytes=bytes.saturating_add(serde_json::to_vec(data).map_err(|_|"Histórico Gravity inválido.")?.len());
            if bytes>MAX_HISTORY{return Err("A paginação Gravity excede128 MiB; não foi apresentado um histórico parcial.".into());}
            if steps.len().saturating_add(data.len())>total{return Err(HistoryError::changing("A conversa Gravity cresceu durante a paginação. Atualiza o histórico."));}
            steps.extend(data.iter().cloned());
        }
        let after=self.metadata().await?;
        if after["trajectoryId"]!=before["trajectoryId"]||number(&after["numSteps"]).is_some_and(|n|n!=steps.len()){
            return Err(HistoryError::changing("A conversa Gravity mudou durante a leitura. Atualiza o histórico para receber uma versão coerente."));
        }
        Ok((response,steps,after))
    }

    pub async fn history(&self)->Result<Snapshot,HistoryError>{
        let(response,steps,metadata)=self.trajectory().await?;
        let messages=public_messages(&metadata["trajectoryId"],&steps)?;
        let idle=is_idle(&response["status"]);
        Ok(Snapshot{target:target(&self.session_id),messages,busy:!idle,control_conflict:false,turn:None,capabilities:Capabilities{history:true,send:idle,cancel:false},notice:Some("As aprovações e a interrupção desta sessão continuam no cliente Gravity que a executa.".into()),model:None})
    }

    pub async fn send(&self,query:&str,model:Option<&str>)->Result<Value,DeliveryError>{
        if query.trim().is_empty()||query.len()>256*1024{return Err(DeliveryError::NotSent("A mensagem Gravity deve ter texto e até256 KiB.".into()));}
        if model.is_some_and(|m|!m.is_empty()){return Err(DeliveryError::NotSent("Muda o modelo desta sessão no cliente Gravity nativo; o painel preserva a configuração da execução existente.".into()));}
        let(response,_,_)=self.trajectory().await.map_err(|error|DeliveryError::NotSent(error.to_string()))?;
        if !is_idle(&response["status"]){return Err(DeliveryError::NotSent("A sessão Gravity está ocupada. Atualiza o histórico antes de enviar.".into()));}
        self.call("SendUserCascadeMessage",json!({"cascadeId":self.session_id,"items":[{"text":query}],"deliveryStrategy":"MESSAGE_DELIVERY_STRATEGY_WHEN_IDLE"})).await.map_err(DeliveryError::Unknown)?;
        Ok(json!({"target":target(&self.session_id),"accepted":true}))
    }

    /// Abort in this protocol lacks an expected invocation ID. Observation must
    /// not accidentally cancel a later turn that another client has just started.
    pub fn cancel(&self)->Result<Value,String>{Err("Esta versão Gravity não oferece cancelamento externo preso ao turno. Interrompe a tarefa no cliente nativo.".into())}
}

fn number(value:&Value)->Option<usize>{value.as_u64().and_then(|v|usize::try_from(v).ok()).or_else(||value.as_str().and_then(|s|s.parse().ok()))}
fn is_idle(value:&Value)->bool{value=="CASCADE_RUN_STATUS_IDLE"||value==1}
fn public_messages(trajectory:&Value,steps:&[Value])->Result<Vec<Message>,String>{
    let trajectory=trajectory.as_str().ok_or("A conversa Gravity não tem trajetória.")?;
    let mut messages=Vec::new();let mut bytes=0usize;
    for(index,step)in steps.iter().enumerate(){
        let(public,role)=if let Some(user)=step.get("userInput"){
            let content=user["query"].as_str().filter(|s|!s.is_empty()).or_else(||user["userResponse"].as_str().filter(|s|!s.is_empty())).map(str::to_string).unwrap_or_else(||user["items"].as_array().into_iter().flatten().filter_map(|v|v["text"].as_str()).collect::<Vec<_>>().join("\n"));
            (content,Role::User)
        }else if let Some(assistant)=step.get("plannerResponse"){
            (assistant["modifiedResponse"].as_str().filter(|s|!s.is_empty()).or_else(||assistant["response"].as_str()).unwrap_or("").to_string(),Role::Assistant)
        }else if let Some(notification)=step.get("notifyUser"){
            (notification["notificationContent"].as_str().unwrap_or("").to_string(),Role::Assistant)
        }else{continue;};
        if public.is_empty(){continue;}
        bytes=bytes.saturating_add(public.len());if bytes>MAX_HISTORY{return Err("O histórico público Gravity excede128 MiB.".into());}
        messages.push(Message{id:format!("{trajectory}:{index}"),role,content:public});
    }Ok(messages)
}
