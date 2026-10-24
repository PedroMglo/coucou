//! Codex daemon JSON-RPC over the user's existing local Unix WebSocket.
//! This client never starts an executor, imports a transcript, or retries a write.
use std::{collections::HashMap, path::{Path,PathBuf},sync::{Arc,Mutex,atomic::{AtomicBool,AtomicU64,Ordering}},time::{Duration,Instant}};
use futures_util::{SinkExt,StreamExt};
use serde_json::{json,Value};
use tokio::{net::UnixStream,sync::{broadcast,mpsc,oneshot},time::timeout};
use tokio_tungstenite::{client_async_with_config,tungstenite::{Message,protocol::WebSocketConfig}};

#[path="native_settings.rs"]
pub(crate) mod native_settings;
#[path="request_sources.rs"]mod request_sources;
#[path="native_write.rs"]mod native_write;
use crate::pet_native_decisions::Ticket;

const IO_TIMEOUT:Duration=Duration::from_secs(20);
const MAX_FRAME:usize=64*1024*1024;
const MAX_REQUEST:usize=1024*1024;
type Answer=Result<Value,Error>;

#[derive(Clone,Debug)]
pub struct Error {pub code:Option<i64>,pub message:String,pub native_message:Option<String>}
impl Error {
    fn local(message:&str)->Self{Self{code:None,message:message.into(),native_message:None}}
    pub fn unsupported(&self)->bool{self.code==Some(-32601)}
}
impl std::fmt::Display for Error{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.message)}}

struct Packet {message:Value,written:oneshot::Sender<Result<(),()>>,scope:Arc<native_write::Scope>}
impl Packet {
    fn expired(&self)->bool{self.written.is_closed()||self.scope.expired()}
}
struct PendingEntry {
    identity: Arc<()>,
    answer: oneshot::Sender<Answer>,
}
type Pending=Arc<Mutex<HashMap<String,PendingEntry>>>;

/// Owns only this call's pending slot across every suspension point.
struct PendingCall {
    pending: Pending,
    id: String,
    identity: Arc<()>,
}
impl Drop for PendingCall {
    fn drop(&mut self) {
        let removed = {
            let mut pending = self.pending.lock().unwrap_or_else(|poison| poison.into_inner());
            if pending.get(&self.id).is_some_and(|entry| Arc::ptr_eq(&entry.identity, &self.identity)) {
                pending.remove(&self.id)
            } else {
                None
            }
        };
        // Dropping a sender may wake a receiver; release the map lock first.
        drop(removed);
    }
}
pub struct Client {
    writer:mpsc::Sender<Packet>,pending:Pending,events:broadcast::Sender<Value>,
    next:AtomicU64,connected:Arc<AtomicBool>,stop:tokio::sync::watch::Sender<bool>,
    endpoint:PathBuf,endpoint_identity:(u64,u64,i64,i64),
    settings:Arc<Mutex<native_settings::Cache>>,
    sources:request_sources::Sources,
}

/// Default control socket only; a caller cannot route a pet to a remote endpoint.
pub fn socket_path(home:&Path)->PathBuf {
    let codex_home=std::env::var_os("CODEX_HOME").map(PathBuf::from).filter(|p|p.is_absolute()).unwrap_or_else(||home.join(".codex"));
    codex_home.join("app-server-control/app-server-control.sock")
}

fn validate_socket(path:&Path)->Result<(),Error>{
    use std::os::unix::fs::{FileTypeExt,MetadataExt};
    let meta=std::fs::symlink_metadata(path).map_err(|_|Error::local("O daemon Codex não está disponível. Abre uma sessão Codex com o controlo local ativo."))?;
    if !meta.file_type().is_socket()||meta.uid()!=unsafe{libc::geteuid()}||meta.mode()&0o022!=0 {
        return Err(Error::local("O socket Codex deve pertencer ao utilizador e impedir escrita por outros."));
    }
    Ok(())
}

fn socket_identity(path:&Path)->Result<(u64,u64,i64,i64),Error>{
    use std::os::unix::fs::MetadataExt;
    validate_socket(path)?;
    let meta=std::fs::symlink_metadata(path).map_err(|_|Error::local("O socket Codex mudou durante a ligação."))?;
    Ok((meta.dev(),meta.ino(),meta.ctime(),meta.ctime_nsec()))
}

impl Client {
    pub async fn connect(path:&Path)->Result<Arc<Self>,Error>{
        // Codex may publish its control endpoint through a stable symlink.
        let path=path.canonicalize().map_err(|_|Error::local("O socket de controlo Codex não está disponível."))?;
        let endpoint_identity=socket_identity(&path)?;
        let stream=timeout(IO_TIMEOUT,UnixStream::connect(&path)).await.map_err(|_|Error::local("O daemon Codex não respondeu à ligação."))?.map_err(|_|Error::local("Não foi possível ligar ao daemon Codex."))?;
        if stream.peer_cred().map_err(|_|Error::local("Não foi possível verificar o daemon Codex."))?.uid()!=unsafe{libc::geteuid()}{return Err(Error::local("O daemon Codex pertence a outro utilizador."));}
        if socket_identity(&path)?!=endpoint_identity{return Err(Error::local("O daemon Codex mudou durante a ligação. Atualiza as sessões."));}
        let write_slot:native_write::Slot=Arc::default();let stream=native_write::Socket::new(stream,write_slot.clone());
        let mut config=WebSocketConfig::default();config.max_message_size=Some(MAX_FRAME);config.max_frame_size=Some(MAX_FRAME);
        let(ws,_)=timeout(IO_TIMEOUT,client_async_with_config("ws://localhost/",stream,Some(config))).await.map_err(|_|Error::local("O daemon Codex não concluiu a ligação."))?.map_err(|_|Error::local("A ligação WebSocket local do Codex não está disponível."))?;
        let sources=request_sources::Sources::new().map_err(|_|Error::local("Não foi possível identificar a origem nativa Codex."))?;
        let(mut sink,mut source)=ws.split();
        let(writer,mut packets)=mpsc::channel::<Packet>(32);
        let(events,_)=broadcast::channel(256);
        let(stop,mut stopped)=tokio::sync::watch::channel(false);
        let pending:Pending=Arc::default();let connected=Arc::new(AtomicBool::new(true));
        let settings:Arc<Mutex<native_settings::Cache>>=Arc::default();
        let client=Arc::new(Self{writer,pending:pending.clone(),events:events.clone(),next:AtomicU64::new(1),connected:connected.clone(),stop,endpoint:path,endpoint_identity,settings:settings.clone(),sources:sources.clone()});
        let out_pending=pending.clone();let out_connected=connected.clone();let out_settings=settings.clone();let out_sources=sources.clone();let out_stop=client.stop.clone();
        tokio::spawn(async move{
            let mut clean_close=true;
            loop {
                tokio::select!{
                    _=stopped.changed()=>break,
                    packet=packets.recv()=>{
                        let Some(packet)=packet else{break;};
                        // A timed-out caller or a decision resolved in another
                        // client cannot leave a delayed write in this queue.
                        if packet.expired(){let _=packet.written.send(Err(()));continue;}
                        *write_slot.lock().unwrap_or_else(|p|p.into_inner())=Some(packet.scope.clone());
                        let data=packet.message.to_string();
                        let expired=async{while !packet.expired(){tokio::time::sleep(Duration::from_millis(2)).await;}};
                        let ok=tokio::select!{biased;
                            _=stopped.changed()=>false,
                            _=expired=>false,
                            result=timeout(packet.scope.gate.deadline.saturating_duration_since(Instant::now()),sink.send(Message::Text(data.into())))=>result.is_ok_and(|r|r.is_ok()),
                        };
                        if ok{*write_slot.lock().unwrap_or_else(|p|p.into_inner())=None;}
                        else{packet.scope.retire();clean_close=false;}
                        let _=packet.written.send(if ok{Ok(())}else{Err(())});
                        if !ok{break;}
                    }
                }
            }
            let _=out_stop.send(true);
            out_sources.disconnect();out_connected.store(false,Ordering::Release);
            out_settings.lock().unwrap_or_else(|p|p.into_inner()).disconnect();
            fail_pending(&out_pending).await;
            // Closing tungstenite flushes its output buffer. An abandoned
            // send retains its retired low-level guard and is dropped instead.
            if clean_close{let _=timeout(Duration::from_millis(200),sink.close()).await;}
        });
        let mut stopped=client.stop.subscribe();
        tokio::spawn(async move{
            loop{
                let message=tokio::select!{_=stopped.changed()=>break,message=source.next()=>message};
                let Some(Ok(message))=message else{break;};
                let Message::Text(data)=message else{if matches!(message,Message::Close(_)){break;}continue;};
                let Ok(mut value)=serde_json::from_str::<Value>(&data)else{break;};
                let at=settings_now();
                let settings_event={let mut cache=settings.lock().unwrap_or_else(|p|p.into_inner());let event=cache.event(&value,at);sources.observe(&mut value,&cache,at);event};
                if value.get("method").is_none(){
                    if let Some(id)=value["id"].as_str(){
                        let answer = {
                            let mut pending = pending.lock().unwrap_or_else(|poison| poison.into_inner());
                            pending.remove(id)
                        };
                        if let Some(entry)=answer{
                            let result=if value.get("error").is_some(){Err(Error{code:value["error"]["code"].as_i64(),message:"O daemon Codex recusou o pedido. Atualiza a sessão antes de continuar.".into(),native_message:value["error"]["message"].as_str().map(|s|s.chars().take(1024).collect())})}
                                else{value.get("result").cloned().ok_or_else(||Error::local("O daemon Codex devolveu uma resposta incompleta."))};
                            let _=entry.answer.send(result);
                        }
                    }
                }else if value["method"]=="thread/settings/updated"{
                    // Forward only an observation invalidation. ThreadSettings
                    // also carries private instructions/model configuration.
                    // Intercept malformed request-shaped frames too: Cache
                    // ignores them, and raw settings must never escape here.
                    if let Some(event)=settings_event{let _=events.send(event);}
                }else if allowed_event(&value){let _=events.send(value);}
            }
            sources.disconnect();connected.store(false,Ordering::Release);settings.lock().unwrap_or_else(|p|p.into_inner()).disconnect();fail_pending(&pending).await;
            let _=events.send(json!({"method":"mochi/disconnected","params":{}}));
        });
        client.call("initialize",json!({"clientInfo":{"name":"coucou_mochi_sessions","title":"Mochi · Sessões","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}})).await?;
        client.notify("initialized",json!({})).await?;
        Ok(client)
    }

    pub fn connected(&self)->bool{self.connected.load(Ordering::Acquire)}
    pub fn events(&self)->broadcast::Receiver<Value>{self.events.subscribe()}

    /// Read the latest permission projection observed on THIS connection only.
    /// This is not an update ACK, terminal binding, effective turn policy or an
    /// authority to approve a request. It performs no RPC or filesystem access.
    pub(crate) fn settings_observation(&self,thread_id:&str,now:u64,max_age:u64)->native_settings::Observation{
        let mut settings=self.settings.lock().unwrap_or_else(|p|p.into_inner());
        if !self.connected(){settings.disconnect();}
        settings.snapshot(thread_id,now,max_age)
    }

    /// Reuse only this connection's verified local endpoint when opening its
    /// TUI. A replacement socket is not the same executor/provenance.
    pub fn verified_endpoint(&self)->Result<PathBuf,Error>{
        if !self.connected()||socket_identity(&self.endpoint)?!=self.endpoint_identity{
            return Err(Error::local("A ligação ao daemon mudou. Atualiza a sessão antes de abrir o terminal."));
        }
        Ok(self.endpoint.clone())
    }

    async fn write(&self,message:Value)->Result<(),Error>{
        self.write_guarded(message,None).await
    }
    async fn write_guarded(&self,message:Value,cancel:Option<Arc<AtomicBool>>)->Result<(),Error>{self.write_native_guarded(message,cancel,None).await}
    async fn write_native_guarded(&self,message:Value,cancel:Option<Arc<AtomicBool>>,native:Option<Ticket>)->Result<(),Error>{
        if !self.connected(){return Err(Error::local("A ligação ao daemon Codex terminou. Atualiza a sessão; o pedido não será reenviado."));}
        if message.to_string().len()>MAX_REQUEST{return Err(Error::local("A mensagem excede o limite de envio ao Codex."));}
        let(tx,rx)=oneshot::channel();let caller=native_write::Caller(Arc::new(AtomicBool::new(true)));
        let scope=native_write::Scope::new(crate::pet_native_write::Guard{deadline:Instant::now()+IO_TIMEOUT,cancel,caller:Some(caller.0.clone()),source:native.map(|t|(t,crate::pet_native_write::Use::Decision))});
        timeout(IO_TIMEOUT,self.writer.send(Packet{message,written:tx,scope})).await.map_err(|_|Error::local("O daemon Codex não recebeu o pedido dentro do prazo. Confirma o histórico antes de repetir."))?.map_err(|_|Error::local("A ligação ao Codex terminou antes do envio."))?;
        timeout(IO_TIMEOUT,rx).await.map_err(|_|Error::local("Não foi possível confirmar o envio ao Codex. Confirma o histórico antes de repetir."))?.map_err(|_|Error::local("O envio ao Codex foi interrompido. Confirma o histórico antes de repetir."))?.map_err(|_|Error::local("A ligação terminou durante o envio ao Codex. Confirma o histórico antes de repetir."))
    }

    pub async fn call(&self,method:&str,params:Value)->Answer{
        let id=format!("mochi-{}-{}",std::process::id(),self.next.fetch_add(1,Ordering::Relaxed));
        let(tx,rx)=oneshot::channel();
        let identity = Arc::new(());
        let _pending_call = {
            let mut pending = self.pending.lock().unwrap_or_else(|poison| poison.into_inner());
            if pending.contains_key(&id) {
                return Err(Error::local("O identificador da consulta Codex já está reservado."));
            }
            if pending.len()>=128 {
                return Err(Error::local("Há demasiadas consultas Codex pendentes. Aguarda antes de atualizar."));
            }
            pending.insert(id.clone(), PendingEntry { identity: identity.clone(), answer: tx });
            PendingCall { pending: self.pending.clone(), id: id.clone(), identity }
        };
        if let Err(e)=self.write(json!({"id":id,"method":method,"params":params})).await{return Err(e);}
        let result=timeout(IO_TIMEOUT,rx).await;
        match result{Ok(Ok(answer))=>answer.map_err(|mut error|{if let Some(code)=error.code{error.message=format!("O daemon Codex recusou {method} (código{code}). Atualiza a sessão antes de continuar.");}error}),_=>Err(Error::local("O daemon Codex não confirmou o resultado. Consulta o histórico antes de repetir o pedido."))}
    }
    pub async fn notify(&self,method:&str,params:Value)->Result<(),Error>{self.write(json!({"method":method,"params":params})).await}
    pub(crate)fn native_request(&self,event:&Value)->Option<Ticket>{self.verified_endpoint().ok()?;self.sources.lookup(event)}
    /// Retain the opaque capture until the writer checks it at the actual send.
    /// Any uncertain attempt retires it; it is never retried automatically.
    pub(crate)async fn respond_native(&self,ticket:&Ticket,result:Value,cancel:Arc<AtomicBool>)->Result<(),Error>{
        self.verified_endpoint()?;
        if !self.sources.owns(ticket){return Err(Error::local("O pedido nativo já não está pendente nesta ligação."));}
        let result=self.write_native_guarded(json!({"id":ticket.identity().request_id,"result":result}),Some(cancel),Some(ticket.clone())).await;
        self.sources.finish(ticket);result
    }
    /// Called only after a human broker decision for the exact native request.
    pub async fn respond(&self,id:Value,result:Value)->Result<(),Error>{self.write(json!({"id":id,"result":result})).await}
    #[cfg(test)]pub async fn respond_guarded(&self,id:Value,result:Value,cancel:Arc<AtomicBool>)->Result<(),Error>{self.write_guarded(json!({"id":id,"result":result}),Some(cancel)).await}
}

impl Drop for Client{fn drop(&mut self){self.sources.disconnect();let _=self.stop.send(true);}}
async fn fail_pending(pending:&Pending){
    let entries: Vec<_> = {
        let mut pending = pending.lock().unwrap_or_else(|poison| poison.into_inner());
        pending.drain().map(|(_, entry)| entry).collect()
    };
    for entry in entries {
        let _=entry.answer.send(Err(Error::local("A ligação Codex terminou; atualiza o histórico antes de repetir.")));
    }
}

fn settings_now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}

fn allowed_event(message:&Value)->bool{
    if message.get("id").is_some(){return true;} // Native approval/question; never answer in the transport.
    match message["method"].as_str().unwrap_or(""){
        "thread/settings/updated"|"item/agentMessage/delta"|"thread/status/changed"|"thread/started"|"thread/name/updated"|"turn/started"|"turn/completed"|"serverRequest/resolved"|"thread/closed"=>true,
        "item/started"|"item/completed"=>matches!(message["params"]["item"]["type"].as_str(),Some("userMessage"|"agentMessage"|"commandExecution"|"fileChange"|"mcpToolCall"|"dynamicToolCall")),
        _=>false,
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::net::UnixListener;
    include!("rpc_cancel_tests.rs");
    include!("list_budget_rpc_tests.rs");
    include!("native_settings_rpc_tests.rs");
    include!("request_sources_rpc_tests.rs");

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT:AtomicU64=AtomicU64::new(0);
            let nonce=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path=std::env::temp_dir().join(format!("cep-{}-{nonce:x}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) { let _=std::fs::remove_dir_all(&self.0); }
    }

    enum Command {
        Emit(Vec<Value>,oneshot::Sender<()>),
        Close,
    }
    struct Fixture {
        client:Option<Arc<Client>>,
        server:tokio::task::JoinHandle<()>,
        commands:mpsc::UnboundedSender<Command>,
        socket:PathBuf,
        held:Arc<std::sync::Mutex<Vec<Value>>>,
        received:Arc<std::sync::Mutex<Vec<Value>>>,
        _scratch:Scratch,
    }
    impl Fixture {
        async fn new() -> Self {
            let scratch=Scratch::new();
            let socket=scratch.0.join("control.sock");
            let listener=UnixListener::bind(&socket).unwrap();
            std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
            let (commands,mut incoming)=mpsc::unbounded_channel();
            let initialized=Arc::new(tokio::sync::Notify::new());
            let ready=initialized.clone();
            let held=Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
            let observed=held.clone();
            let received=Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));let received_frames=received.clone();
            let server=tokio::spawn(async move {
                let (stream,_)=listener.accept().await.unwrap();
                let mut ws=tokio_tungstenite::accept_async(stream).await.unwrap();
                loop {
                    tokio::select! {
                        command=incoming.recv()=>match command {
                            Some(Command::Emit(messages,sent))=>{
                                for message in messages {
                                    if ws.send(Message::Text(message.to_string().into())).await.is_err(){return;}
                                }
                                let _=sent.send(());
                            },
                            Some(Command::Close)|None=>{
                                let _=ws.send(Message::Close(None)).await;
                                break;
                            },
                        },
                        message=ws.next()=>{
                            let Some(Ok(message))=message else {break;};
                            let Message::Text(text)=message else {if matches!(message,Message::Close(_)){break;}continue;};
                            let request:Value=serde_json::from_str(&text).unwrap();
                            received_frames.lock().unwrap().push(request.clone());
                            let method=request["method"].as_str().unwrap_or("");
                            if method=="initialized" {ready.notify_one();continue;}
                            if method=="fixture/hold" {observed.lock().unwrap().push(request);continue;}
                            let Some(id)=request.get("id") else {continue;};
                            let response=match method {
                                "initialize"|"fixture/barrier"=>json!({"id":id,"result":{}}),
                                _=>json!({"id":id,"error":{"code":-32601,"message":"Unexpected fixture request"}}),
                            };
                            if ws.send(Message::Text(response.to_string().into())).await.is_err(){break;}
                        },
                    }
                }
            });
            // Own cleanup before any fallible connection or initialization wait.
            let mut fixture=Self{client:None,server,commands,socket,held,received,_scratch:scratch};
            fixture.client=Some(Client::connect(&fixture.socket).await.unwrap());
            timeout(Duration::from_secs(5),initialized.notified()).await.unwrap();
            fixture
        }
        fn client(&self) -> &Client {self.client.as_deref().unwrap()}
        async fn emit(&self,messages:Vec<Value>) {
            let (sent,received)=oneshot::channel();
            self.commands.send(Command::Emit(messages,sent)).unwrap();
            timeout(Duration::from_secs(5),received).await.unwrap().unwrap();
            // This response is after all emitted frames on the same socket;
            // its receipt proves the client's reader processed those frames.
            timeout(Duration::from_secs(5),self.client().call("fixture/barrier",json!({}))).await.unwrap().unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {self.server.abort();drop(self.client.take());}
    }

    #[tokio::test]
    async fn verified_endpoint_accepts_the_original_private_socket() {
        let fixture=Fixture::new().await;
        assert!(fixture.client().connected());
        assert_eq!(fixture.client().verified_endpoint().unwrap(),fixture.socket.canonicalize().unwrap());
        assert_eq!(std::fs::symlink_metadata(&fixture.socket).unwrap().mode()&0o777,0o600);
        assert_eq!(std::fs::symlink_metadata(&fixture._scratch.0).unwrap().mode()&0o777,0o700);
        fixture.emit(Vec::new()).await;
        assert_eq!(fixture.client().verified_endpoint().unwrap(),fixture.socket.canonicalize().unwrap());
    }

    #[tokio::test]
    async fn verified_endpoint_rejects_replacement_inode_while_original_connection_is_live() {
        let fixture=Fixture::new().await;
        let original=std::fs::symlink_metadata(&fixture.socket).unwrap();
        // Retain the old socket inode so an allocator cannot reuse it here.
        let retained=fixture._scratch.0.join("original.sock");
        std::fs::rename(&fixture.socket,&retained).unwrap();
        let _replacement=UnixListener::bind(&fixture.socket).unwrap();
        std::fs::set_permissions(&fixture.socket,std::fs::Permissions::from_mode(0o600)).unwrap();
        let replacement=std::fs::symlink_metadata(&fixture.socket).unwrap();
        assert_ne!(original.ino(),replacement.ino());
        assert!(fixture.client().connected());
        assert!(fixture.client().verified_endpoint().is_err());
        // The live WebSocket still belongs to the old endpoint; it is not
        // silently reconnected to the replacement merely sharing its path.
        fixture.emit(Vec::new()).await;
        assert!(fixture.client().verified_endpoint().is_err());
    }

    #[tokio::test]
    async fn verified_endpoint_rejects_permissions_changed_after_connect() {
        let fixture=Fixture::new().await;
        for mode in [0o620,0o602,0o666] {
            std::fs::set_permissions(&fixture.socket,std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(fixture.client().connected());
            assert!(fixture.client().verified_endpoint().is_err());
        }
    }

    #[tokio::test]
    async fn connect_rejects_foreign_writable_socket_before_websocket_handshake() {
        let scratch=Scratch::new();let socket=scratch.0.join("unsafe.sock");
        let _listener=UnixListener::bind(&socket).unwrap();
        // No accept/handshake task exists: validation must refuse locally.
        for mode in [0o620,0o602,0o666] {
            std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(timeout(Duration::from_secs(2),Client::connect(&socket)).await.unwrap().is_err());
        }
    }

    #[tokio::test]
    async fn verified_endpoint_rejects_a_disconnected_original_socket() {
        let fixture=Fixture::new().await;
        let mut events=fixture.client().events();
        fixture.commands.send(Command::Close).unwrap();
        let event=timeout(Duration::from_secs(5),events.recv()).await.unwrap().unwrap();
        assert_eq!(event["method"],"mochi/disconnected");
        assert!(!fixture.client().connected());
        assert!(fixture.client().verified_endpoint().is_err());
        assert!(fixture.socket.exists());
    }

    fn excluded_notifications() -> Vec<Value> {
        let mut events=vec![
            "item/commandExecution/outputDelta","item/commandExecution/terminalInteraction",
            "item/fileChange/outputDelta","item/fileChange/patchUpdated","item/mcpToolCall/progress",
            "item/reasoning/summaryTextDelta","item/reasoning/summaryPartAdded","item/reasoning/textDelta",
            "item/plan/delta","rawResponseItem/completed","fixture/unknown",
        ].into_iter().map(|method|json!({"method":method,"params":{"threadId":"fixture-thread","delta":"private-payload"}})).collect::<Vec<_>>();
        for method in ["item/started","item/completed"] {
            for kind in ["reasoning","toolResult","unknownItem"] {
                events.push(json!({"method":method,"params":{"threadId":"fixture-thread","item":{"type":kind,"text":"private-payload"}}}));
            }
        }
        events
    }

    #[test]
    fn event_filter_allows_tool_lifecycle_metadata_and_excludes_private_notification_types() {
        for method in ["item/started","item/completed"] {
            for kind in ["userMessage","agentMessage","commandExecution","fileChange","mcpToolCall","dynamicToolCall"] {
                assert!(allowed_event(&json!({"method":method,"params":{"item":{"id":"fixture-item","type":kind}}})));
            }
        }
        for event in excluded_notifications() {assert!(!allowed_event(&event),"Unexpected forwarded notification: {}",event["method"]);}
        // Native requests must reach the approval/tool broker unchanged; this
        // filter authorizes delivery, never execution or an automatic answer.
        assert!(allowed_event(&json!({"id":"native-request","method":"item/tool/call","params":{"threadId":"fixture-thread"}})));
    }

    #[tokio::test]
    async fn client_event_stream_filters_outputs_and_reasoning_but_preserves_public_and_tool_metadata() {
        let fixture=Fixture::new().await;
        let mut events=fixture.client().events();
        let expected=vec![
            json!({"method":"item/started","params":{"threadId":"fixture-thread","item":{"id":"tool-1","type":"commandExecution","status":"inProgress"}}}),
            json!({"method":"item/agentMessage/delta","params":{"threadId":"fixture-thread","itemId":"answer-1","delta":"public-answer"}}),
            json!({"id":"native-request","method":"item/tool/call","params":{"threadId":"fixture-thread","turnId":"fixture-turn"}}),
        ];
        let mut sent=excluded_notifications();sent.extend(expected.clone());
        fixture.emit(sent).await;
        let mut received=Vec::new();
        loop {
            match events.try_recv() {
                Ok(event)=>received.push(event),
                Err(broadcast::error::TryRecvError::Empty)=>break,
                Err(error)=>panic!("Unexpected event stream state: {error}"),
            }
        }
        assert_eq!(received,expected);
        assert!(!serde_json::to_string(&received).unwrap().contains("private-payload"));
    }
}
