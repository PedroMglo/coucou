//! Bounded NDJSON ACP v1 transport for an owned Copilot child process.
//! Only public agent_message_chunk text leaves the protocol as a reply.
//! Permission choices preserve advertised ACP IDs/scopes and native policy.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver,SyncSender}, Arc};
use std::time::{Duration, Instant};

use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture,Ticket,PolicyBinding,RequestBook}};
use std::sync::Mutex;
enum ConfigRequest{New,Existing(String)}
#[derive(Default)]struct NativeMonitor{
    session:Option<String>,prompt:Option<Value>,catalog:Option<Catalog>,
    config_requests:std::collections::BTreeMap<u64,ConfigRequest>,config_sequence:u64,
}
impl NativeMonitor{
    fn observe(&mut self,value:&Value,source:&Endpoint)->Result<(),String>{
        if value.get("method").is_none(){
            if self.prompt.as_ref().is_some_and(|id|value.get("id")==Some(id)){source.close();}
            if let Some(expected)=value["id"].as_u64().and_then(|id|self.config_requests.remove(&id)){
                if value.get("error").is_some(){return Ok(());}
                let result=value.get("result").filter(|v|v.is_object()).ok_or("Copilot não confirmou a configuração da sessão.")?;
                let session=match expected{
                    ConfigRequest::New=>result["sessionId"].as_str().filter(|v|valid_name(v)).ok_or("Copilot devolveu uma sessão inválida.")?.to_string(),
                    ConfigRequest::Existing(expected)=>{
                        if result.get("sessionId").is_some_and(|v|v.as_str()!=Some(expected.as_str()))||self.session.as_ref().is_some_and(|v|v!=&expected){return Err("Copilot confirmou outra sessão; a conversa pedida foi preservada.".into());}expected
                    },
                };
                self.session=Some(session);self.catalog=Some(Catalog::from_result(result));
                self.config_sequence=self.config_sequence.checked_add(1).ok_or("A sequência de configuração Copilot esgotou-se.")?;
            }
        }else if value["method"]=="session/update"&&value["params"]["update"]["sessionUpdate"]=="config_option_update"
            &&self.session.as_deref()==value["params"]["sessionId"].as_str()&&self.session.is_some(){
            // Observe before delivery: a waiting RPC/human callback must not
            // hide a later native policy/model notification. This projection
            // is not a claim about all effective native permission rules.
            self.catalog=Some(Catalog::from_result(&json!({"configOptions":value["params"]["update"]["configOptions"]})));
            self.config_sequence=self.config_sequence.checked_add(1).ok_or("A sequência de configuração Copilot esgotou-se.")?;
            source.invalidate_policy(self.session.as_deref());
        }
        Ok(())
    }
}
const MAX_LINE: usize = 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;
const MAX_ANSWER: usize = 4 * 1024 * 1024;
pub const CANCELLED: &str = "The CLI request was cancelled.";
pub const SIGN_IN: &str = "Inicia sessão na tua conta GitHub com copilot login e tenta novamente.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Model { pub id: String, pub label: String }

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub models: Vec<Model>,
    pub current_model: String,
    pub config_options: Vec<Value>,
}

impl Catalog {
    fn from_result(result: &Value) -> Self {
        let config_options = result["configOptions"].as_array().cloned().unwrap_or_default();
        let mut seen = HashSet::new();
        let mut models = Vec::new();
        for item in result["models"]["availableModels"].as_array().into_iter().flatten().take(256) {
            if item.pointer("/_meta/copilotEnablement").and_then(Value::as_str).is_some_and(|v| v != "enabled") { continue; }
            let Some(id) = item["modelId"].as_str().filter(|v| valid_name(v)) else { continue; };
            if seen.insert(id.to_string()) {
                models.push(Model { id: id.into(), label: label(item["name"].as_str().unwrap_or(id)) });
            }
        }
        let mut current_model = result["models"]["currentModelId"].as_str().unwrap_or("").to_string();
        if let Some(option) = config_options.iter().find(|v| v["category"] == "model" || v["id"] == "model") {
            if let Some(current) = option["currentValue"].as_str() { current_model = current.into(); }
            if models.is_empty() {
                for item in values(option) {
                    if item.pointer("/_meta/copilotEnablement").and_then(Value::as_str).is_some_and(|v| v != "enabled") { continue; }
                    let Some(id) = item["value"].as_str().filter(|v| valid_name(v)) else { continue; };
                    if seen.insert(id.into()) { models.push(Model { id: id.into(), label: label(item["name"].as_str().unwrap_or(id)) }); }
                }
            }
        }
        Self { models, current_model, config_options }
    }

    fn config(&self, category: &str, id: &str) -> Option<&Value> {
        self.config_options.iter().find(|v| v["category"] == category || v["id"] == id)
    }
}

fn valid_name(value: &str) -> bool { !value.is_empty() && value.len() <= 512 && !value.chars().any(char::is_control) }
fn label(value: &str) -> String { value.chars().filter(|c| !c.is_control()).take(256).collect() }
fn values(option: &Value) -> Vec<&Value> {
    option["options"].as_array().into_iter().flatten().take(256).flat_map(|value| {
        if let Some(group) = value["options"].as_array() { group.iter().take(256).collect() } else { vec![value] }
    }).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Thinking, Working, Approval }

pub struct TurnOptions<'a> {
    pub model: &'a str,
    pub effort: &'a str,
    pub system: &'a str,
    pub history: &'a [Value],
    pub query: &'a str,
    pub session_id: Option<&'a str>,
}

#[derive(Debug)]
pub struct Reply {
    pub text: String,
    pub acted: bool,
    pub session_id: String,
    pub model: String,
    pub effort: String,
    pub stop_reason: String,
}

struct NativeFrame{value:Value,ticket:Option<Ticket>}
fn observe_request_cancellation(requests:&RequestBook,value:&Value){
    if value["method"]=="$/cancel_request"&&value.get("id").is_none(){
        if let Some(id)=value.pointer("/params/requestId"){
            for ticket in requests.tickets(){if ticket.identity().request_id==*id{ticket.request_withdrawal();}}
        }
    }
}
impl std::ops::Deref for NativeFrame{type Target=Value;fn deref(&self)->&Value{&self.value}}

struct Outbound{bytes:Vec<u8>,guard:crate::pet_native_write::Guard,ack:SyncSender<Result<crate::pet_native_write::Outcome,crate::pet_native_write::Fault>>}
struct WriteWait(Arc<AtomicBool>);
impl Drop for WriteWait{fn drop(&mut self){self.0.store(false,Ordering::Release);}}
fn run_writer(mut stdin:impl Write,requests:Receiver<Outbound>,source:Endpoint){
    while let Ok(packet)=requests.recv(){
        let result=crate::pet_native_write::frame(&mut stdin,&packet.bytes,&packet.guard);
        let fatal=result.is_err();let _=packet.ack.send(result);
        if fatal{source.close();break;}
    }
}

pub struct Client {
    #[cfg(test)] io_probe:Option<crate::settings::EffectProbe>,
    child: Child,
    source:Endpoint,
    requests:RequestBook,
    monitor:Arc<Mutex<NativeMonitor>>,
    input: Option<SyncSender<Outbound>>,
    output: Receiver<Result<NativeFrame, String>>,
    deadline: Instant,
    next_id: u64,
    capabilities: Value,
    session: Option<String>,
    active: bool,
    catalog: Catalog,
    cwd: String,
    cancel: Option<Arc<AtomicBool>>,
    used:bool,
    broken:bool,
}

fn command(exe: &Path, cwd: &Path) -> Command {
    let mut cmd = Command::new(exe);
    cmd.args(["--acp", "--stdio", "--no-auto-update", "--log-level", "none", "--no-remote", "--no-remote-export"])
        .current_dir(cwd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    // This factory adapter uses the native GitHub account. Preserve native
    // permission/mode environment; only avoid a different BYOK provider identity.
    for (key, _) in std::env::vars_os() {
        if key.to_str().is_some_and(|k| k.starts_with("COPILOT_PROVIDER_") || k=="COPILOT_PROVIDERS_CONFIG") { cmd.env_remove(key); }
    }
    #[cfg(unix)] { use std::os::unix::process::CommandExt; cmd.process_group(0); }
    cmd
}

impl Client {
    pub fn start(exe: &Path, cwd: &Path, timeout: Duration) -> Result<Self, String> {
        Self::start_with_cancel(exe,cwd,timeout,None)
    }

    pub fn start_with_cancel(exe:&Path,cwd:&Path,timeout:Duration,cancel:Option<Arc<AtomicBool>>)->Result<Self,String> {
        let cwd = cwd.canonicalize().map_err(|_| "A pasta do Copilot já não está disponível.")?;
        if !cwd.is_dir() { return Err("O Copilot precisa de uma pasta de trabalho válida.".into()); }
        let source=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe)?;let monitor:Arc<Mutex<NativeMonitor>>=Arc::default();
        crate::settings::effect_allowed()?;
        let mut child = command(exe, &cwd).spawn().map_err(|error| format!("Não foi possível iniciar Copilot ACP ({:?}). Verifica a instalação e a conta no terminal.",error.kind()))?;
        let stdin = child.stdin.take().ok_or("Copilot não abriu a entrada ACP.")?;
        #[cfg(unix)]{
            use std::os::fd::AsRawFd;
            let fd=stdin.as_raw_fd();let flags=unsafe{libc::fcntl(fd,libc::F_GETFL)};
            if flags<0||unsafe{libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{
                unsafe{libc::kill(-(child.id()as i32),libc::SIGKILL);}let _=child.kill();let _=child.wait();
                return Err("Não foi possível preparar a entrada cancelável do Copilot.".into());
            }
        }
        let (input,requests) = mpsc::sync_channel::<Outbound>(1);let writer_source=source.clone();
        std::thread::spawn(move||run_writer(stdin,requests,writer_source));
        let stdout = child.stdout.take().ok_or("Copilot não abriu a ligação ACP.")?;
        // Wire bytes and frame count bound delivery while the human callback
        // waits; the reader still observes configuration/completion/EOF.
        let requests=RequestBook::new(source.clone());
        let (tx, output) = mpsc::channel();let reader_source=source.clone();let reader_monitor=monitor.clone();let reader_requests=requests.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut total = 0;let mut frames=0usize;
            loop {
                frames+=1;if frames>65_536{let _=tx.send(Err("Copilot excedeu o limite de eventos deste turno.".into()));break;}
                let result = read_message(&mut reader, &mut total).and_then(|value|{
                    let mut state=reader_monitor.lock().map_err(|_|"A origem Copilot está indisponível.")?;
                    state.observe(&value,&reader_source)?;
                    observe_request_cancellation(&reader_requests,&value);
                    let ticket=if value["method"]=="session/request_permission"&&value.get("id").is_some()&&state.session.is_some()&&state.session.as_deref()==value["params"]["sessionId"].as_str()&&reader_source.is_open(){
                        let prompt=state.prompt.as_ref().ok_or("Copilot pediu uma decisão sem um prompt ativo.")?;
                        Some(reader_requests.register(Capture{request_id:Some(&value["id"]),session_id:state.session.as_deref().unwrap(),turn_id:None,prompt_id:Some(prompt),method:"session/request_permission",arguments:&value["params"],policy:PolicyBinding::unknown(),cancel:None})?)
                    }else{None};
                    Ok(NativeFrame{value,ticket})
                });
                let failed = result.is_err();
                if tx.send(result).is_err() || failed { break; }
            }
            reader_source.close();
        });
        let mut client = Self { #[cfg(test)]io_probe:None,child, source, requests, monitor, input:Some(input), output, deadline: Instant::now() + timeout.min(Duration::from_secs(30)), next_id: 0, capabilities: Value::Null, session: None, active: false, catalog: Catalog::default(), cwd: cwd.to_string_lossy().into(), cancel,used:false,broken:false };
        let init = client.request("initialize", json!({"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"coucou_mochi","version":env!("CARGO_PKG_VERSION")}}))?;
        if init["protocolVersion"] != 1 { return Err("A versão ACP do Copilot não é compatível com este cliente.".into()); }
        client.capabilities = init["agentCapabilities"].clone();
        client.deadline=Instant::now()+timeout;
        Ok(client)
    }

    pub fn cancel_with(&mut self, cancel: Arc<AtomicBool>) { self.cancel = Some(cancel); }
    fn cancelled(&self) -> bool { self.cancel.as_ref().is_some_and(|v| v.load(Ordering::Relaxed)) }

    fn write(&mut self, message: Value) -> Result<(), String> {
        if self.broken {return Err("A ligação Copilot terminou. Abre uma nova ligação para continuar a sessão guardada.".into());}
        let result=self.write_until(message,self.deadline,true);
        if result.is_err(){self.broken=true;}
        result
    }

    fn write_until(&self,message:Value,deadline:Instant,respect_cancel:bool)->Result<(),String>{
        match self.write_until_guarded(message,deadline,respect_cancel,None)?{
            crate::pet_native_write::Outcome::Written=>Ok(()),
            crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::Cancelled)=>Err(CANCELLED.into()),
            crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::PreferencesUnconfirmed)=>Err("A origem das preferências não está confirmada; o novo pedido Copilot não foi enviado.".into()),
            crate::pet_native_write::Outcome::Skipped(_)=>Err("Copilot não recebeu a mensagem antes de expirar.".into()),
        }
    }
    fn write_until_guarded(&self,message:Value,deadline:Instant,respect_cancel:bool,source:Option<(Ticket,crate::pet_native_write::Use)>)->Result<crate::pet_native_write::Outcome,String>{
        let input=self.input.as_ref().ok_or("Copilot fechou a ligação ACP.")?;
        let mut bytes=serde_json::to_vec(&message).map_err(|_|"Não foi possível preparar a mensagem Copilot.")?;
        if bytes.len()>MAX_LINE{return Err("O pedido Copilot excedeu o limite de tamanho.".into());}bytes.push(b'\n');
        let waiting=WriteWait(Arc::new(AtomicBool::new(true)));
        let dispatch=if !respect_cancel||source.as_ref().is_some_and(|(_,u)|matches!(u,crate::pet_native_write::Use::Refusal|crate::pet_native_write::Use::Withdrawal)){crate::pet_native_write::Dispatch::Control}else{crate::pet_native_write::Dispatch::NewEffect};
        #[cfg(test)]let dispatch=match (&dispatch,&self.io_probe){(crate::pet_native_write::Dispatch::NewEffect,Some(p))=>crate::pet_native_write::Dispatch::ProbedEffect(p.clone()),_=>dispatch};
        let guard=crate::pet_native_write::Guard{dispatch,deadline,cancel:if respect_cancel{self.cancel.clone()}else{None},caller:Some(waiting.0.clone()),source};
        let(ack,answer)=mpsc::sync_channel(1);let mut packet=Outbound{bytes,guard,ack};
        loop{
            if let Some(reason)=packet.guard.write_reason(){return Ok(crate::pet_native_write::Outcome::Skipped(reason));}
            match input.try_send(packet){Ok(())=>break,Err(mpsc::TrySendError::Full(value))=>{packet=value;std::thread::sleep(Duration::from_millis(2));},Err(_)=>return Err("Copilot fechou a entrada ACP.".into())}
        }
        loop{
            // Once queued, the writer returns proof of Written/Skipped/partial
            // fault. A timeout abandons the caller guard and is never retried.
            let left=deadline.saturating_duration_since(Instant::now());if left.is_zero(){return Err("Copilot não confirmou o envio dentro do limite de tempo.".into());}
            match answer.recv_timeout(left.min(Duration::from_millis(10))){
                Ok(Ok(outcome))=>return Ok(outcome),
                Ok(Err(fault))=>return Err(if fault.reason==crate::pet_native_write::Reason::Cancelled{CANCELLED}else if fault.partial{"A resposta Copilot perdeu validade durante uma escrita parcial; a ligação própria foi retirada sem repetir o envio."}else{"Não foi possível escrever para o Copilot."}.into()),
                Err(mpsc::RecvTimeoutError::Disconnected)=>return Err("A escrita Copilot terminou sem confirmação.".into()),
                Err(mpsc::RecvTimeoutError::Timeout)=>{},
            }
        }
    }

    fn read(&self) -> Result<NativeFrame, String> {
        loop {
            if self.cancelled() { return Err(CANCELLED.into()); }
            let left = self.deadline.saturating_duration_since(Instant::now());
            if left.is_zero() { return Err("O Copilot não terminou dentro do limite de tempo.".into()); }
            match self.output.recv_timeout(left.min(Duration::from_millis(100))) {
                Ok(value) => return value,
                Err(mpsc::RecvTimeoutError::Timeout) => {},
                Err(_) => return Err("Copilot fechou a ligação antes de concluir. Verifica a conta no terminal.".into()),
            }
        }
    }

    fn begin_request(&mut self, method: &str, params: Value) -> Result<u64, String> {
        self.next_id=self.next_id.checked_add(1).ok_or("A sequência de pedidos Copilot esgotou-se.")?;
        let id = self.next_id;
        {
            let mut state=self.monitor.lock().map_err(|_|"A origem Copilot está indisponível.")?;
            if method=="session/prompt"{state.prompt=Some(json!(id));}
            let expected=match method{
                "session/new"=>Some(ConfigRequest::New),
                "session/load"|"session/set_config_option"=>Some(ConfigRequest::Existing(params["sessionId"].as_str().filter(|v|valid_name(v)).ok_or("Identificador de sessão Copilot inválido.")?.into())),
                _=>None,
            };
            if let Some(expected)=expected{if state.config_requests.len()>=8{return Err("Demasiadas confirmações Copilot pendentes.".into());}state.config_requests.insert(id,expected);}
        }
        if let Err(error)=self.write(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})){
            if let Ok(mut state)=self.monitor.lock(){state.config_requests.remove(&id);}return Err(error);
        }
        Ok(id)
    }

    fn reject(&mut self, message: &Value) -> Result<(), String> {
        if message.get("id").is_some() && message.get("method").is_some() {
            let reply = if message["method"] == "session/request_permission" { json!({"result":cancelled_outcome()}) }
                else { json!({"error":{"code":-32601,"message":"Client capability not advertised."}}) };
            self.respond(&message["id"], reply)?;
        }
        Ok(())
    }

    fn respond(&mut self, id: &Value, reply: Value) -> Result<(), String> {
        let mut response = json!({"jsonrpc":"2.0","id":id});
        if let Some(value) = reply.get("result") { response["result"] = value.clone(); }
        if let Some(value) = reply.get("error") { response["error"] = value.clone(); }
        // An incoming permission request still needs its cancelled outcome
        // after the work token is set. Bound the control write independently.
        let result=self.write_until(response,Instant::now()+Duration::from_millis(100),false);
        if result.is_err(){self.broken=true;}result
    }

    fn respond_native(&mut self,result:Value,ticket:&Ticket,params:&Value)->Result<crate::pet_native_write::Outcome,String>{
        let withdrawal=result==cancelled_outcome();
        let refusal=crate::pet_native_choices::pure_refusal("copilot","session/request_permission",params,&result);
        let response=json!({"jsonrpc":"2.0","id":ticket.identity().request_id,"result":result});
        let use_=if withdrawal{crate::pet_native_write::Use::Withdrawal}else if refusal{crate::pet_native_write::Use::Refusal}else{crate::pet_native_write::Use::Decision};
        let result=self.write_until_guarded(response,Instant::now()+Duration::from_millis(100),!withdrawal,Some((ticket.clone(),use_)));
        if result.is_err(){self.broken=true;}result
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.begin_request(method, params)?;
        loop {
            let message = self.read()?;
            if message["id"] == id && message.get("method").is_none() { return response_result(&message); }
            self.reject(&message)?;
        }
    }

    fn open(&mut self, session: Option<&str>) -> Result<(), String> {
        if let Some(id) = session {
            if !valid_name(id) { return Err("Identificador de sessão Copilot inválido.".into()); }
            if self.capabilities["loadSession"] != true { return Err("Esta versão do Copilot não permite retomar a conversa guardada. A conversa foi preservada.".into()); }
            self.request("session/load", json!({"sessionId":id,"cwd":self.cwd,"mcpServers":[]}))?;
            self.session = Some(id.into());
        } else {
            let result = self.request("session/new", json!({"cwd":self.cwd,"mcpServers":[]}))?;
            let id = result["sessionId"].as_str().filter(|v| valid_name(v)).ok_or("Copilot devolveu uma sessão inválida.")?;
            self.session = Some(id.into());
        }
        self.sync_catalog()?;
        Ok(())
    }
    fn observed_catalog(&self)->Result<Catalog,String>{
        let state=self.monitor.lock().map_err(|_|"A configuração Copilot está indisponível.")?;
        if state.session!=self.session{return Err("A configuração não pertence à sessão Copilot pedida.".into());}
        state.catalog.clone().ok_or_else(||"Copilot ainda não confirmou a configuração da sessão.".into())
    }
    fn sync_catalog(&mut self)->Result<(),String>{self.catalog=self.observed_catalog()?;Ok(())}

    pub fn models(&mut self) -> Result<Vec<Model>, String> {
        if self.used {return Err("Este cliente Copilot já foi usado para um turno.".into());}
        if self.session.is_none() { self.open(None)?; }
        self.sync_catalog()?;
        if self.catalog.models.is_empty() { return Err("Copilot não disponibilizou modelos. Verifica a tua conta no terminal.".into()); }
        Ok(self.catalog.models.clone())
    }

    pub fn catalog(&self) -> &Catalog { &self.catalog }

    fn set_config(&mut self, id: &str, value: &str) -> Result<(), String> {
        self.sync_catalog()?;
        let option = self.catalog.config_options.iter().find(|v| v["id"] == id).ok_or("A opção pedida não é suportada pelo Copilot.")?;
        if option["type"] != "select" || !values(option).iter().any(|v| v["value"] == value) { return Err("O valor pedido não está disponível nesta sessão Copilot.".into()); }
        self.request("session/set_config_option", json!({"sessionId":self.session,"configId":id,"value":value}))?;
        let updated = self.observed_catalog()?;
        if !updated.config_options.iter().any(|v| v["id"] == id && v["currentValue"] == value) { return Err("Copilot não confirmou a configuração pedida.".into()); }
        self.catalog = updated;
        Ok(())
    }

    fn configure(&mut self, model: &str, effort: &str) -> Result<(), String> {
        self.sync_catalog()?;
        if !model.is_empty() {
            if !self.catalog.models.iter().any(|m| m.id == model) { return Err("O modelo escolhido não está no catálogo desta conta Copilot.".into()); }
            if self.catalog.current_model != model {
                let id = self.catalog.config("model", "model").and_then(|v| v["id"].as_str()).ok_or("Copilot não suporta seleção de modelo nesta sessão.")?.to_string();
                self.set_config(&id, model)?;
            }
        }
        if !effort.is_empty() {
            let option = self.catalog.config("thought_level", "reasoning_effort").ok_or("O modelo Copilot não disponibiliza o esforço pedido.")?;
            if option["currentValue"] != effort {
                let id = option["id"].as_str().ok_or("Opção de esforço Copilot inválida.")?.to_string();self.set_config(&id, effort)?;
            }
        }
        self.validate_selection(model,effort)
    }

    fn validate_selection(&self,model:&str,effort:&str)->Result<(),String> {
        let catalog=self.observed_catalog()?;
        if !model.is_empty() && model!="auto" && catalog.current_model!=model {return Err("Copilot alterou o modelo escolhido. O turno foi interrompido para preservar a seleção explícita.".into());}
        if !effort.is_empty() && catalog.config("thought_level","reasoning_effort").is_none_or(|v|v["currentValue"]!=effort) {return Err("Copilot alterou o esforço escolhido. O turno foi interrompido.".into());}
        Ok(())
    }

    /// The caller supplies its existing native permission broker. It returns
    /// Some(true) for Allow once, Some(false) for Deny, None for cancellation.
    #[cfg(test)]pub fn answer(&mut self,options:TurnOptions<'_>,on_session:impl FnMut(&str),on_delta:impl FnMut(String),on_phase:impl FnMut(Phase),mut permission:impl FnMut(&Value)->Option<bool>)->Result<Reply,String>{self.answer_native(options,on_session,on_delta,on_phase,|params,_|permission(params).map(|allow|json!({"allow":allow})))}
    pub fn answer_native(
        &mut self,
        options: TurnOptions<'_>,
        mut on_session: impl FnMut(&str),
        mut on_delta: impl FnMut(String),
        mut on_phase: impl FnMut(Phase),
        mut permission: impl FnMut(&Value,&Ticket) -> Option<Value>,
    ) -> Result<Reply, String> {
        if self.used || self.broken {return Err("Este cliente Copilot já terminou um turno. Abre uma nova ligação para continuar a sessão guardada.".into());}
        if options.query.is_empty() || options.query.len() > 128 * 1024 || options.system.len() > 16 * 1024 { return Err("O pedido excede o limite desta conversa Copilot.".into()); }
        self.used=true;
        if self.session.is_some() && options.session_id.is_some_and(|id| self.session.as_deref() != Some(id)) { return Err("Este processo Copilot já pertence a outra conversa.".into()); }
        if self.session.is_none() { self.open(options.session_id)?; }
        self.configure(options.model, options.effort)?;
        let session = self.session.clone().unwrap();
        on_session(&session);
        let prompt = prompt_text(&options);
        let id = self.begin_request("session/prompt", json!({"sessionId":session,"prompt":[{"type":"text","text":prompt}]}))?;
        self.active = true;
        let result = (|| {
            let mut text = String::new();let mut acted = false;
            loop {
                let message = self.read()?;
                if message["id"] == id && message.get("method").is_none() {
                    let result = response_result(&message)?;
                    self.sync_catalog()?;self.validate_selection(options.model,options.effort)?;
                    let stop = result["stopReason"].as_str().ok_or("Copilot não indicou o resultado do turno.")?;
                    if stop == "cancelled" { return Err(CANCELLED.into()); }
                    if !matches!(stop, "end_turn" | "max_tokens" | "max_turn_requests" | "refusal") { return Err("Copilot devolveu um resultado de turno desconhecido.".into()); }
                    if text.trim().is_empty() { return Err("Copilot terminou sem uma resposta pública em texto.".into()); }
                    let effort=self.catalog.config("thought_level","reasoning_effort").and_then(|v|v["currentValue"].as_str()).unwrap_or("").to_string();
                    return Ok(Reply { text, acted, session_id: session.clone(), model: self.catalog.current_model.clone(),effort, stop_reason: stop.into() });
                }
                match message["method"].as_str() {
                    Some("session/update") if message["params"]["sessionId"] == session => {
                        let update = &message["params"]["update"];
                        match update["sessionUpdate"].as_str() {
                            Some("agent_message_chunk") if update["content"]["type"] == "text" => {
                                let chunk = update["content"]["text"].as_str().ok_or("Bloco de resposta Copilot inválido.")?;
                                if text.len().saturating_add(chunk.len()) > MAX_ANSWER { return Err("A resposta Copilot excedeu o limite de tamanho.".into()); }
                                text.push_str(chunk);on_delta(text.clone());
                            }
                            Some("agent_thought_chunk") => on_phase(Phase::Thinking),
                            Some("tool_call" | "tool_call_update") => {
                                if matches!(update["status"].as_str(), Some("in_progress" | "completed")) { acted = true; }
                                on_phase(Phase::Working);
                            }
                            Some("config_option_update")=>{
                                self.sync_catalog()?;
                                self.validate_selection(options.model,options.effort)?;
                            }
                            _ => {},
                        }
                    }
                    Some("session/request_permission") if message.get("id").is_some() => {
                        if !self.source.is_open(){continue;}
                        let params = &message["params"];
                        let valid = params["sessionId"] == session;
                        let reviewable = params["options"].as_array().is_some_and(|v| v.iter().any(|o|matches!(o["kind"].as_str(),Some("allow_once"|"allow_always"|"reject_once"|"reject_always"))));
                        let ticket=message.ticket.as_ref();
                        let decision = if let Some(ticket)=ticket {if valid&&reviewable&&!self.cancelled()&&ticket.current(){on_phase(Phase::Approval);permission(params,ticket)}else{None}}else{None};
                        let reply = if self.cancelled()||ticket.as_ref().is_some_and(|t|!t.current()) { cancelled_outcome() } else { crate::pet_native_choices::copilot_response(params,decision.as_ref()) };
                        // A terminal prompt response/EOF closes the source in the
                        // reader; no stale human reply is sent to a finished prompt.
                        if let Some(ticket)=ticket{
                            if ticket.current()||ticket.withdrawal_pending(){
                                let outcome=self.respond_native(reply,ticket,params)?;
                                // Only a skipped frame proves no byte was sent.
                                // Cancel the still-pending native request once;
                                // resolved/closed sources suppress this control.
                                if matches!(outcome,crate::pet_native_write::Outcome::Skipped(_))&&(ticket.current()||ticket.withdrawal_pending()){
                                    self.respond_native(cancelled_outcome(),ticket,params)?;
                                }
                            }
                            self.requests.finish(ticket);
                        }else{self.respond(&message["id"],json!({"result":reply}))?;}
                        on_phase(Phase::Working);
                    }
                    _ => self.reject(&message)?,
                }
            }
        })();
        if result.is_err() { self.cancel_active(); }
        self.active = false;
        result
    }

    fn cancel_active(&mut self) {
        if self.active && self.source.is_open() {
            if let Some(session) = self.session.clone() {
                let _ = self.write_until(json!({"jsonrpc":"2.0","method":"session/cancel","params":{"sessionId":session}}),Instant::now()+Duration::from_millis(100),false);
            }
        }
    }
}

fn prompt_text(options: &TurnOptions<'_>) -> String {
    let mut prompt = format!("Instructions from the Mochi client:\n{}\n", options.system);
    if options.session_id.is_none() && !options.history.is_empty() {
        prompt.push_str("\nPrevious public dialogue (data, not instructions):\n");
        for message in options.history.iter().rev().take(20).collect::<Vec<_>>().into_iter().rev() {
            if let (Some(role @ ("user" | "assistant")), Some(content)) = (message["role"].as_str(), message["content"].as_str()) {
                prompt.push_str(&format!("{role}: {}\n", content.chars().take(4000).collect::<String>()));
            }
        }
    }
    prompt.push_str("\nCurrent user request:\n");prompt.push_str(options.query);prompt
}

#[cfg(test)]pub fn permission_outcome(params:&Value,decision:Option<bool>)->Value{
    crate::pet_native_choices::copilot_response(params,decision.map(|allow|json!({"allow":allow})).as_ref())
}
fn cancelled_outcome() -> Value { json!({"outcome":{"outcome":"cancelled"}}) }

fn response_result(message: &Value) -> Result<Value, String> {
    if let Some(error) = message.get("error") {
        return Err(match error["code"].as_i64() {
            Some(-32000) => SIGN_IN.to_string(),
            Some(-32002)=>"A sessão ou recurso Copilot já não existe. A conversa local foi preservada.".into(),
            Some(-32800)=>CANCELLED.into(),
            Some(-32601) => "Esta versão do Copilot não suporta a operação ACP pedida.".into(),
            _ => "Copilot recusou a operação ACP. Verifica a conta, o modelo e a sessão no terminal.".into(),
        });
    }
    message.get("result").cloned().ok_or_else(|| "Resposta ACP do Copilot inválida.".into())
}

fn read_message(reader: &mut impl BufRead, total: &mut usize) -> Result<Value, String> {
    let mut bytes = Vec::new();
    let size = reader.take(MAX_LINE as u64 + 1).read_until(b'\n', &mut bytes).map_err(|_| "Não foi possível ler Copilot ACP.")?;
    if size == 0 { return Err("Copilot fechou a ligação ACP.".into()); }
    *total = total.saturating_add(size);
    if size > MAX_LINE || *total > MAX_TOTAL { return Err("A ligação Copilot excedeu o limite de dados.".into()); }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "Copilot enviou uma mensagem ACP inválida.")?;
    if !value.is_object() || value["jsonrpc"] != "2.0" { return Err("Copilot enviou uma mensagem fora do protocolo ACP.".into()); }
    Ok(value)
}

impl Drop for Client {
    fn drop(&mut self) {
        self.source.close();
        self.cancel_active();self.input.take();
        // Keep the leader unreaped until its group is cleaned: an exited
        // leader can leave native tool children behind, including on cancel.
        std::thread::sleep(Duration::from_millis(300));
        unsafe { libc::kill(-(self.child.id() as i32),libc::SIGTERM); }
        std::thread::sleep(Duration::from_millis(100));
        unsafe { libc::kill(-(self.child.id() as i32),libc::SIGKILL); }
        let _ = self.child.kill();let _ = self.child.wait();
    }
}

#[cfg(test)]
#[path="tests.rs"]
mod tests;
