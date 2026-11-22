//! A private bridge to one existing Gravity Language Server conversation.
//! The native address and CSRF capability remain in inherited environment/RAM.
//! No command here launches, aborts, or changes a Language Server or terminal.

use std::{collections::{HashSet,VecDeque}, fs::{File, Metadata, OpenOptions}, io::{Read, Write},
    os::unix::{fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        process::CommandExt}, path::{Path, PathBuf}, process::{Command, Stdio},
    sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant as StdInstant}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{UnixListener, UnixStream},
    sync::{broadcast, mpsc, watch, Semaphore}, task::{JoinHandle, JoinSet}, time::{timeout,timeout_at,Instant}};
use super::client::{Captured, DeliveryError, Native};
use crate::session_chat::types::{Capabilities, Origin, Snapshot, Target};

const FLAG:&str="--gravity-session-server";
const MAX_STARTUP:usize=16*1024;
const MAX_REQUEST:usize=1024*1024;
const MAX_RESPONSE:usize=128*1024*1024;
const IO_TIMEOUT:Duration=Duration::from_secs(30);
const PROVIDER:&str="antigravity";
const PET:&str="agent_antigravity";

/// Called inside a native hook. Only reviewed public identity/workspace fields
/// enter stdin; native authentication is inherited, never copied to argv/files.
pub fn register_from_hook(raw:&Value)->Result<(),String>{
    let Some(captured)=Captured::from_hook(raw)? else{return Ok(());};
    let input=json!({"conversationId":captured.session_id,"workspacePaths":
        if captured.cwd.is_empty(){Vec::<String>::new()}else{vec![captured.cwd]}});
    let bytes=serde_json::to_vec(&input).map_err(|_|"Não foi possível preparar a ponte Gravity.".to_string())?;
    if bytes.len()>MAX_STARTUP{return Err("O registo público Gravity excede16 KiB.".into());}
    let executable=std::env::current_exe().map_err(|_|"Não foi possível localizar a ponte Gravity.".to_string())?;
    let mut child=Command::new(executable).arg(FLAG).process_group(0)
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().map_err(|_|"Não foi possível iniciar o observador Gravity local.".to_string())?;
    let result=child.stdin.take().ok_or_else(||"A ponte Gravity não recebeu o registo.".to_string())
        .and_then(|pipe|feed_startup(pipe,&bytes));
    if result.is_err(){let _=child.kill();let _=child.wait();return result;}
    // Reap while the hook remains alive; after hook exit, normal orphan reaping
    // takes over. No native session process is signalled by this observer.
    let _=std::thread::Builder::new().name("gravity-observer-reap".into()).spawn(move||{let _=child.wait();});
    Ok(())
}

fn feed_startup(mut pipe:std::process::ChildStdin,bytes:&[u8])->Result<(),String>{
    use std::os::fd::AsRawFd;
    let descriptor=pipe.as_raw_fd();let flags=unsafe{libc::fcntl(descriptor,libc::F_GETFL)};
    if flags<0||unsafe{libc::fcntl(descriptor,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err("Não foi possível preparar o registo Gravity.".into());}
    let deadline=StdInstant::now()+Duration::from_secs(1);let mut offset=0;
    while offset<bytes.len(){
        match pipe.write(&bytes[offset..]){
            Ok(0)=>return Err("A ponte Gravity fechou o registo.".into()),
            Ok(count)=>offset+=count,
            Err(error)if error.kind()==std::io::ErrorKind::Interrupted=>continue,
            Err(error)if error.kind()==std::io::ErrorKind::WouldBlock=>{
                let remaining=deadline.saturating_duration_since(StdInstant::now());
                if remaining.is_zero(){return Err("O registo Gravity expirou.".into());}
                let mut poll=libc::pollfd{fd:descriptor,events:libc::POLLOUT,revents:0};
                if unsafe{libc::poll(&mut poll,1,remaining.as_millis().min(i32::MAX as u128) as i32)}<0
                    &&std::io::Error::last_os_error().kind()!=std::io::ErrorKind::Interrupted{return Err("O registo Gravity foi interrompido.".into());}
            }
            Err(_)=>return Err("A ponte Gravity não recebeu o registo.".into()),
        }
    }Ok(())
}

/// Must run before GUI startup. Errors are deliberately not printed: neither
/// inherited capabilities nor conversation payloads belong in hook output.
pub fn maybe_run()->bool{
    let args=std::env::args_os().collect::<Vec<_>>();
    if args.get(1).is_none_or(|argument|argument!=FLAG){return false;}
    if args.len()!=2{return true;}
    if let Ok(raw)=read_startup(){
        if let Ok(runtime)=tokio::runtime::Builder::new_current_thread().enable_all().build(){
            let _=runtime.block_on(serve(raw));
        }
    }
    true
}

fn read_startup()->Result<Value,String>{
    let deadline=StdInstant::now()+Duration::from_secs(3);
    let mut bytes=Vec::new();let mut buffer=[0u8;4096];
    loop{
        let remaining=deadline.saturating_duration_since(StdInstant::now());
        if remaining.is_zero(){return Err("O registo Gravity expirou.".into());}
        let mut poll=libc::pollfd{fd:libc::STDIN_FILENO,events:libc::POLLIN,revents:0};
        let ready=unsafe{libc::poll(&mut poll,1,remaining.as_millis().min(i32::MAX as u128) as i32)};
        if ready<0&&std::io::Error::last_os_error().kind()==std::io::ErrorKind::Interrupted{continue;}
        if ready<=0{return Err("O registo Gravity não está disponível.".into());}
        let count=unsafe{libc::read(libc::STDIN_FILENO,buffer.as_mut_ptr().cast(),buffer.len())};
        if count==0{break;}
        if count<0{return Err("O registo Gravity foi interrompido.".into());}
        if bytes.len().saturating_add(count as usize)>MAX_STARTUP{return Err("O registo Gravity excede16 KiB.".into());}
        bytes.extend_from_slice(&buffer[..count as usize]);
    }
    let value:Value=serde_json::from_slice(&bytes).map_err(|_|"O registo Gravity é inválido.".to_string())?;
    let object=value.as_object().ok_or("O registo Gravity é inválido.")?;
    if object.keys().any(|key|key!="conversationId"&&key!="workspacePaths"){
        return Err("O registo Gravity contém campos não permitidos.".into());
    }
    Ok(value)
}

#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Record{provider:String,session_id:String,cwd:String,pid:u32,parent_pid:u32,capabilities:Capabilities}
struct Endpoint{socket:PathBuf,metadata:PathBuf,_lock:File,socket_identity:Option<(u64,u64)>,metadata_identity:Option<(u64,u64)>}
impl Drop for Endpoint{
    fn drop(&mut self){remove_own(&self.metadata,self.metadata_identity);remove_own(&self.socket,self.socket_identity);}
}
struct Pending{request_id:String,last_user:Option<String>}
#[derive(Default)]
struct Admission{in_flight:bool,pending:Option<Pending>,ids:HashSet<String>,order:VecDeque<String>}
struct Shared{native:Native,target:Target,latest:RwLock<Arc<Snapshot>>,healthy:AtomicBool,
    admission:Mutex<Admission>,updates:broadcast::Sender<()>,stop:watch::Sender<bool>}
struct AbortOnDrop(JoinHandle<()>);
impl Drop for AbortOnDrop{fn drop(&mut self){self.0.abort();}}

async fn serve(raw:Value)->Result<(),String>{
    let captured=Captured::from_hook(&raw)?.ok_or("O hook não forneceu uma sessão Gravity nativa.")?;
    let target=Target{pet_id:PET.into(),provider:PROVIDER.into(),origin:Origin::External,
        session_id:captured.session_id.clone(),conversation_id:None};
    target.validate_external(PROVIDER,PET)?;
    let Some(mut endpoint)=reserve(&target)? else{return Ok(());};
    if !clear_stale(&endpoint,&target).await?{return Ok(());}
    let native=captured.connect()?;
    let initial=initial_history(&native).await?;
    validate_snapshot(&target,&initial)?;
    let initial_hash=hash_snapshot(&initial)?;
    // This umask belongs only to the dedicated helper process.
    unsafe{libc::umask(0o077);}
    let listener=UnixListener::bind(&endpoint.socket).map_err(|_|"Não foi possível abrir a ponte Gravity privada.")?;
    let socket_meta=std::fs::symlink_metadata(&endpoint.socket).map_err(|_|"Não foi possível confirmar a ponte Gravity.")?;
    endpoint.socket_identity=Some(identity(&socket_meta));
    std::fs::set_permissions(&endpoint.socket,std::fs::Permissions::from_mode(0o600)).map_err(|_|"Não foi possível proteger a ponte Gravity.")?;
    let record=Record{provider:PROVIDER.into(),session_id:target.session_id.clone(),cwd:native.cwd.clone(),
        pid:std::process::id(),parent_pid:unsafe{libc::getppid()} as u32,
        capabilities:Capabilities{history:true,send:true,cancel:false}};
    publish_metadata(&mut endpoint,&record)?;
    let(updates,_)=broadcast::channel(16);let(stop,mut stopped)=watch::channel(false);
    let shared=Arc::new(Shared{native,target,latest:RwLock::new(Arc::new(initial)),healthy:AtomicBool::new(true),
        admission:Mutex::new(Admission::default()),updates,stop});
    let poll=shared.clone();
    let _poll=AbortOnDrop(tokio::spawn(async move{poll_native(poll,initial_hash).await;}));
    let permits=Arc::new(Semaphore::new(8));let mut connections=JoinSet::new();
    loop{
        tokio::select!{
            _=stopped.changed()=>break,
            accepted=listener.accept()=>{
                let Ok((socket,_))=accepted else{break;};
                let Ok(credentials)=socket.peer_cred() else{continue;};
                if credentials.uid()!=unsafe{libc::geteuid()}{continue;}
                let Ok(permit)=permits.clone().try_acquire_owned()else{continue;};
                let shared=shared.clone();connections.spawn(async move{let _permit=permit;let _=connection(socket,shared).await;});
            }
            Some(_)=connections.join_next(),if !connections.is_empty()=>{}
        }
    }
    let _=shared.stop.send(true);drop(listener);connections.shutdown().await;
    drop(endpoint);Ok(())
}

async fn initial_history(native:&Native)->Result<Snapshot,String>{
    let deadline=Instant::now()+IO_TIMEOUT;
    loop{
        match timeout_at(deadline,native.history()).await{
            Ok(Ok(snapshot))=>return Ok(snapshot),
            Ok(Err(error))if error.transient()=>{
                let remaining=deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero(){return Err("A conversa Gravity não estabilizou dentro do prazo; a ponte não foi anunciada.".into());}
                tokio::time::sleep(remaining.min(Duration::from_millis(250))).await;
            }
            Ok(Err(error))=>return Err(error.to_string()),
            Err(_)=>return Err("A confirmação Gravity expirou; a ponte não foi anunciada.".into()),
        }
    }
}

async fn poll_native(shared:Arc<Shared>,mut hash:u64){
    let mut failures=0;let mut interval=tokio::time::interval(Duration::from_millis(750));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop{
        interval.tick().await;
        let result=match timeout(IO_TIMEOUT,shared.native.history()).await{
            Ok(Ok(snapshot))=>validate_snapshot(&shared.target,&snapshot).and_then(|_|hash_snapshot(&snapshot).map(|hash|(snapshot,hash))),
            Ok(Err(error))if error.transient()=>{
                // A changing trajectory proves the native endpoint answered.
                // Wait for a coherent public snapshot without retiring it.
                failures=0;shared.healthy.store(false,Ordering::Release);let _=shared.updates.send(());continue;
            }
            _=>Err("A sessão Gravity não respondeu.".into()),
        };
        match result{
            Ok((snapshot,next_hash))=>{
                failures=0;let recovered=!shared.healthy.swap(true,Ordering::AcqRel);
                let user=last_user(&snapshot);
                let pending_changed={let mut admission=shared.admission.lock().unwrap_or_else(|e|e.into_inner());
                    let clear=admission.pending.as_ref().is_some_and(|pending|snapshot.busy||pending.last_user.as_deref()!=user);
                    if clear{admission.pending=None;}clear};
                *shared.latest.write().unwrap_or_else(|e|e.into_inner())=Arc::new(snapshot);
                if recovered||pending_changed||hash!=next_hash{hash=next_hash;let _=shared.updates.send(());}
            }
            Err(_)=>{
                failures+=1;shared.healthy.store(false,Ordering::Release);let _=shared.updates.send(());
                if failures>=3{let _=shared.stop.send(true);break;}
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request{id:Value,method:String,target:Target,query:Option<String>,model:Option<String>,turn:Option<String>}
#[derive(Serialize)]
struct Success<'a,T:Serialize>{id:&'a Value,result:T}
#[derive(Serialize)]
struct Failure<'a>{id:&'a Value,error:ErrorReply}
#[derive(Serialize)]
struct ErrorReply{code:&'static str,message:&'static str}

async fn connection(socket:UnixStream,shared:Arc<Shared>)->Result<(),String>{
    let(reader,mut writer)=socket.into_split();let(mut reader,mut updates)=(BufReader::new(reader),shared.updates.subscribe());
    let(tx,mut requests)=mpsc::channel(1);
    let _reader=AbortOnDrop(tokio::spawn(async move{
        while let Ok(value)=read_packet(&mut reader).await{if tx.send(value).await.is_err(){break;}}
    }));
    loop{
        tokio::select!{
            request=requests.recv()=>{
                let Some(value)=request else{break;};
                let id=value.get("id").filter(|id|valid_id(id)).cloned().unwrap_or(Value::Null);
                let request=serde_json::from_value::<Request>(value).ok();
                let Some(request)=request.filter(|request|valid_request(request,&shared.target))else{
                    write_packet(&mut writer,&Failure{id:&id,error:ErrorReply{code:"invalid_request",message:"O pedido não identifica esta sessão Gravity ou tem parâmetros inválidos."}}).await?;continue;
                };
                match request.method.as_str(){
                    "history"=>{
                        if !shared.healthy.load(Ordering::Acquire){
                            write_packet(&mut writer,&Failure{id:&id,error:ErrorReply{code:"session_unavailable",message:"A ligação à sessão Gravity está por confirmar. Atualiza o histórico antes de continuar."}}).await?;continue;
                        }
                        let snapshot=shared.latest.read().unwrap_or_else(|e|e.into_inner()).clone();
                        let pending={let admission=shared.admission.lock().unwrap_or_else(|e|e.into_inner());admission.in_flight||admission.pending.is_some()};
                        let mut capabilities=snapshot.capabilities;if pending{capabilities.send=false;}
                        let view=SnapshotView{target:&snapshot.target,messages:&snapshot.messages,busy:snapshot.busy||pending,
                            turn:if pending{None}else{snapshot.turn.as_deref()},capabilities,
                            notice:if pending{Some("A entrega está a ser confirmada na sessão nativa; não reenvies o pedido.")}else{snapshot.notice.as_deref()},model:snapshot.model.as_deref()};
                        write_packet(&mut writer,&Success{id:&id,result:view}).await?;
                    }
                    "send"=>{
                        let result=admit(&shared,&request).await;
                        match result{Ok(value)=>write_packet(&mut writer,&Success{id:&id,result:value}).await?,
                            Err(error)=>write_packet(&mut writer,&Failure{id:&id,error}).await?,}
                    }
                    _=>write_packet(&mut writer,&Failure{id:&id,error:ErrorReply{code:"cancel_unavailable",message:"Esta versão Gravity não permite cancelar um turno exato; usa o cliente nativo."}}).await?,
                }
            }
            result=updates.recv()=>{
                if matches!(result,Err(broadcast::error::RecvError::Closed)){break;}
                write_packet(&mut writer,&json!({"method":"updated","target":shared.target})).await?;
            }
        }
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct SnapshotView<'a>{target:&'a Target,messages:&'a[crate::session_chat::types::Message],busy:bool,
    #[serde(skip_serializing_if="Option::is_none")]turn:Option<&'a str>,capabilities:Capabilities,
    #[serde(skip_serializing_if="Option::is_none")]notice:Option<&'a str>,
    #[serde(skip_serializing_if="Option::is_none")]model:Option<&'a str>}

fn valid_request(request:&Request,target:&Target)->bool{
    if !valid_id(&request.id)||request.target!=*target||request.target.validate_external(PROVIDER,PET).is_err(){return false;}
    match request.method.as_str(){
        "history"=>request.query.is_none()&&request.model.is_none()&&request.turn.is_none(),
        "send"=>request.turn.is_none()&&request.query.as_ref().is_some_and(|query|!query.trim().is_empty()&&query.len()<=256*1024)
            &&request.model.as_ref().is_none_or(|model|model.len()<=256&&!model.chars().any(char::is_control)),
        "cancel"=>request.query.is_none()&&request.model.is_none()&&request.turn.as_ref().is_none_or(|turn|!turn.is_empty()&&turn.len()<=256),
        _=>false,
    }
}
fn valid_id(id:&Value)->bool{match id{Value::String(id)=>!id.is_empty()&&id.len()<=128&&!id.chars().any(char::is_control),
    Value::Number(id)=>id.as_u64().is_some_and(|id|id<=9_007_199_254_740_991),_=>false}}

async fn admit(shared:&Shared,request:&Request)->Result<Value,ErrorReply>{
    if !shared.healthy.load(Ordering::Acquire){return Err(ErrorReply{code:"session_unavailable",message:"A ligação à sessão Gravity não está confirmada."});}
    if request.model.as_ref().is_some_and(|model|!model.is_empty()){return Err(ErrorReply{code:"model_unavailable",message:"Muda o modelo desta sessão no cliente Gravity nativo."});}
    let snapshot=shared.latest.read().unwrap_or_else(|e|e.into_inner()).clone();
    let admission_id=request.id.to_string();
    {let mut admission=shared.admission.lock().unwrap_or_else(|e|e.into_inner());
        if admission.in_flight||admission.pending.is_some()||snapshot.busy{return Err(ErrorReply{code:"busy",message:"Esta sessão Gravity já tem trabalho ativo ou uma entrega por confirmar."});}
        if admission.ids.contains(&admission_id){return Err(ErrorReply{code:"duplicate_send",message:"Este pedido já foi admitido e não será repetido."});}
        if admission.order.len()>=4096{if let Some(oldest)=admission.order.pop_front(){admission.ids.remove(&oldest);}}
        admission.order.push_back(admission_id.clone());admission.ids.insert(admission_id.clone());admission.in_flight=true;
        admission.pending=Some(Pending{request_id:admission_id.clone(),last_user:last_user(&snapshot).map(str::to_owned)});
    }
    let _=shared.updates.send(());
    let result=timeout(IO_TIMEOUT,shared.native.send(request.query.as_deref().unwrap_or(""),request.model.as_deref())).await;
    {let mut admission=shared.admission.lock().unwrap_or_else(|e|e.into_inner());
        admission.in_flight=false;
        if matches!(&result,Ok(Err(DeliveryError::NotSent(_))))
            &&admission.pending.as_ref().is_some_and(|pending|pending.request_id==admission_id){admission.pending=None;}
    }
    let _=shared.updates.send(());
    match result{
        Ok(Ok(value)) if value.get("target").is_some_and(|target|serde_json::from_value::<Target>(target.clone()).is_ok_and(|target|target==shared.target))=>Ok(value),
        Ok(Err(DeliveryError::NotSent(_)))=>Err(ErrorReply{code:"not_sent",message:"O pedido não foi enviado. Atualiza o histórico e confirma o estado desta sessão antes de enviar."}),
        _=>Err(ErrorReply{code:"send_status_unknown",message:"A entrega Gravity não foi confirmada. Consulta o histórico e não reenvies automaticamente."}),
    }
}

fn last_user(snapshot:&Snapshot)->Option<&str>{snapshot.messages.iter().rev().find(|message|message.role==crate::session_chat::types::Role::User).map(|message|message.id.as_str())}
fn validate_snapshot(target:&Target,snapshot:&Snapshot)->Result<(),String>{
    if snapshot.target!=*target||!snapshot.capabilities.history||snapshot.capabilities.cancel{return Err("O histórico Gravity não confirmou a sessão e as capacidades esperadas.".into());}Ok(())
}

async fn read_packet(reader:&mut BufReader<tokio::net::unix::OwnedReadHalf>)->Result<Value,String>{
    // Idle observer connections may remain open. Once a frame starts, enforce
    // its deadline without cancelling/discarding a partially read frame.
    if reader.fill_buf().await.map_err(|_|"Ligação Gravity interrompida.")?.is_empty(){return Err("Ligação Gravity terminada.".into());}
    timeout(IO_TIMEOUT,async{
        let mut bytes=Vec::new();
        loop{let buffer=reader.fill_buf().await.map_err(|_|"Ligação Gravity interrompida.")?;
            if buffer.is_empty(){return Err("Ligação Gravity terminada.".into());}
            let newline=buffer.iter().position(|byte|*byte==b'\n');let count=newline.map(|index|index+1).unwrap_or(buffer.len());
            if bytes.len().saturating_add(count)>MAX_REQUEST{return Err("O pedido Gravity excede1 MiB.".into());}
            bytes.extend_from_slice(&buffer[..count]);reader.consume(count);
            if newline.is_some(){return serde_json::from_slice(&bytes).map_err(|_|"O pedido Gravity é inválido.".into());}
        }
    }).await.map_err(|_|"O pedido Gravity expirou.".to_string())?
}

struct Bounded{bytes:Vec<u8>,limit:usize}
impl Write for Bounded{
    fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{
        if self.bytes.len().saturating_add(bytes.len())>self.limit{return Err(std::io::Error::other("frame budget"));}
        self.bytes.extend_from_slice(bytes);Ok(bytes.len())
    }
    fn flush(&mut self)->std::io::Result<()>{Ok(())}
}
async fn write_packet<T:Serialize>(writer:&mut tokio::net::unix::OwnedWriteHalf,value:&T)->Result<(),String>{
    let mut frame=Bounded{bytes:Vec::new(),limit:MAX_RESPONSE-1};
    serde_json::to_writer(&mut frame,value).map_err(|_|"A resposta Gravity excede128 MiB e não foi truncada.".to_string())?;
    frame.bytes.push(b'\n');timeout(IO_TIMEOUT,writer.write_all(&frame.bytes)).await
        .map_err(|_|"A entrega da resposta Gravity expirou.".to_string())?
        .map_err(|_|"A ligação Gravity terminou durante a resposta.".into())
}
fn hash_snapshot(snapshot:&Snapshot)->Result<u64,String>{
    use std::hash::{Hasher,DefaultHasher};
    struct Digest{hash:DefaultHasher,count:usize}
    impl Write for Digest{
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{self.count=self.count.saturating_add(bytes.len());if self.count>MAX_RESPONSE{return Err(std::io::Error::other("history budget"));}self.hash.write(bytes);Ok(bytes.len())}
        fn flush(&mut self)->std::io::Result<()>{Ok(())}
    }
    let mut digest=Digest{hash:DefaultHasher::new(),count:0};
    serde_json::to_writer(&mut digest,snapshot).map_err(|_|"O histórico Gravity excede128 MiB.".to_string())?;Ok(digest.hash.finish())
}

fn private_directory(path:&Path,create:bool)->Result<(),String>{
    if create{match std::fs::DirBuilder::new().mode(0o700).create(path){Ok(())=>{},Err(error)if error.kind()==std::io::ErrorKind::AlreadyExists=>{},Err(_)=>return Err("Não foi possível criar a pasta privada Gravity.".into())}}
    let metadata=std::fs::symlink_metadata(path).map_err(|_|"A pasta privada Gravity não está disponível.".to_string())?;
    if !metadata.is_dir()||metadata.uid()!=unsafe{libc::geteuid()}||metadata.mode()&0o7777!=0o700
        ||std::fs::canonicalize(path).ok().as_deref()!=Some(path){return Err("A pasta Gravity tem de ser privada, real e pertencer ao utilizador.".into());}Ok(())
}
fn reserve(target:&Target)->Result<Option<Endpoint>,String>{
    let runtime=std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).filter(|path|path.is_absolute()).ok_or("A sessão não disponibiliza XDG_RUNTIME_DIR privado.")?;
    private_directory(&runtime,false)?;let coucou=runtime.join("coucou");private_directory(&coucou,true)?;
    let directory=coucou.join(PROVIDER);private_directory(&directory,true)?;
    let socket=directory.join(format!("{}.sock",target.session_id));let metadata=directory.join(format!("{}.json",target.session_id));
    use std::os::unix::ffi::OsStrExt;
    if socket.as_os_str().as_bytes().len()>=108{return Err("O caminho da ponte Gravity excede o limite Unix.".into());}
    let lock=OpenOptions::new().read(true).write(true).create(true).mode(0o600)
        .custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK).open(directory.join(format!("{}.lock",target.session_id)))
        .map_err(|_|"Não foi possível reservar esta ponte Gravity.".to_string())?;
    private_file(&lock.metadata().map_err(|_|"Não foi possível verificar a reserva Gravity.")?)?;
    use std::os::fd::AsRawFd;
    if unsafe{libc::flock(lock.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{
        let error=std::io::Error::last_os_error();if error.kind()==std::io::ErrorKind::WouldBlock{return Ok(None);}return Err("Não foi possível reservar esta sessão Gravity.".into());
    }
    Ok(Some(Endpoint{socket,metadata,_lock:lock,socket_identity:None,metadata_identity:None}))
}
fn private_file(metadata:&Metadata)->Result<(),String>{
    if !metadata.is_file()||metadata.uid()!=unsafe{libc::geteuid()}||metadata.mode()&0o7777!=0o600||metadata.len()>MAX_STARTUP as u64{return Err("Os metadados Gravity têm de ser um ficheiro privado, regular e limitado a16 KiB.".into());}Ok(())
}
fn identity(metadata:&Metadata)->(u64,u64){(metadata.dev(),metadata.ino())}
fn remove_own(path:&Path,expected:Option<(u64,u64)>){
    if let(Some(expected),Ok(actual))=(expected,std::fs::symlink_metadata(path)){
        if identity(&actual)==expected&&actual.uid()==unsafe{libc::geteuid()}{let _=std::fs::remove_file(path);}
    }
}
fn optional_metadata(path:&Path)->Result<Option<Metadata>,String>{
    match std::fs::symlink_metadata(path){Ok(metadata)=>Ok(Some(metadata)),Err(error)if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(_)=>Err("Não foi possível verificar a ponte Gravity existente.".into())}
}
async fn clear_stale(endpoint:&Endpoint,target:&Target)->Result<bool,String>{
    let metadata=optional_metadata(&endpoint.metadata)?;let socket=optional_metadata(&endpoint.socket)?;
    if metadata.is_none()&&socket.is_none(){return Ok(true);}
    let metadata=metadata.ok_or("Existe um socket Gravity sem metadados verificáveis; foi preservado.")?;private_file(&metadata)?;
    let mut file=OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK).open(&endpoint.metadata).map_err(|_|"Não foi possível verificar a ponte Gravity existente.")?;
    let actual=file.metadata().map_err(|_|"Não foi possível verificar os metadados Gravity.")?;private_file(&actual)?;
    if identity(&actual)!=identity(&metadata){return Err("A ponte Gravity mudou durante a verificação.".into());}
    let mut bytes=Vec::new();(&mut file).take(MAX_STARTUP as u64+1).read_to_end(&mut bytes).map_err(|_|"Não foi possível ler os metadados Gravity.")?;
    if bytes.len()>MAX_STARTUP{return Err("Os metadados Gravity excedem16 KiB.".into());}
    let record:Record=serde_json::from_slice(&bytes).map_err(|_|"Os metadados Gravity existentes são inválidos e foram preservados.")?;
    if record.provider!=PROVIDER||record.session_id!=target.session_id||record.pid==0||record.pid>i32::MAX as u32||record.capabilities.cancel{return Err("A ponte Gravity existente não confirmou a sua identidade e foi preservada.".into());}
    if let Some(socket)=&socket{
        if !socket.file_type().is_socket()||socket.uid()!=unsafe{libc::geteuid()}||socket.mode()&0o7777!=0o600{return Err("O socket Gravity existente não é privado e foi preservado.".into());}
        match timeout(Duration::from_secs(1),UnixStream::connect(&endpoint.socket)).await{
            Ok(Ok(stream))=>{let credentials=stream.peer_cred().map_err(|_|"Não foi possível confirmar a ponte Gravity existente.")?;
                if credentials.uid()==unsafe{libc::geteuid()}&&credentials.pid().map(|pid|pid as u32)==Some(record.pid){return Ok(false);}
                return Err("O socket Gravity existente pertence a outro processo e foi preservado.".into());
            }
            Ok(Err(error))if matches!(error.kind(),std::io::ErrorKind::ConnectionRefused|std::io::ErrorKind::NotFound)=>{},
            _=>return Err("O estado da ponte Gravity existente é inconclusivo; foi preservada.".into()),
        }
    }
    if unsafe{libc::kill(record.pid as i32,0)}==0||std::io::Error::last_os_error().raw_os_error()!=Some(libc::ESRCH){return Err("O processo da ponte Gravity existente continua ativo; foi preservado.".into());}
    if let Some(socket)=socket{remove_own(&endpoint.socket,Some(identity(&socket)));}
    remove_own(&endpoint.metadata,Some(identity(&metadata)));
    if optional_metadata(&endpoint.socket)?.is_some()||optional_metadata(&endpoint.metadata)?.is_some(){return Err("A ponte Gravity mudou durante a limpeza e foi preservada.".into());}Ok(true)
}
fn publish_metadata(endpoint:&mut Endpoint,record:&Record)->Result<(),String>{
    let parent=endpoint.metadata.parent().ok_or("Pasta Gravity inválida.")?;
    let temporary=parent.join(format!(".{}.{}.tmp",record.session_id,std::process::id()));
    let mut file=OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(&temporary).map_err(|_|"Não foi possível preparar os metadados Gravity.")?;
    let identity=identity(&file.metadata().map_err(|_|"Não foi possível confirmar os metadados Gravity.")?);
    let result=(||{serde_json::to_writer(&mut file,record).map_err(|_|"Não foi possível guardar os metadados públicos Gravity.")?;
        file.sync_all().map_err(|_|"Não foi possível confirmar os metadados Gravity.")?;
        std::fs::hard_link(&temporary,&endpoint.metadata).map_err(|_|"Já existe uma ponte Gravity; os metadados existentes foram preservados.")?;
        endpoint.metadata_identity=Some(identity);Ok(())})();
    remove_own(&temporary,Some(identity));result
}
