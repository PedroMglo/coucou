//! Tauri-facing adapter, registered separately so the daemon client stays portable
//! within this Linux project and can be built without a running desktop.
use std::{collections::{HashMap,HashSet},sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU64,Ordering}}};
use serde_json::{json,Value};
use tauri::{AppHandle,Emitter,Manager};
use crate::session_chat::{codex,rpc,types,types::{Target,Snapshot,Summary,ManagedSession,ManagedReference,ManagedPhase}};

#[path="managed.rs"]mod managed;

#[derive(Default)]
struct Backend {
    generation:AtomicU64,
    client:tokio::sync::Mutex<Option<Arc<rpc::Client>>>,
    tracked:Mutex<HashMap<String,Target>>,
    subscribed:tokio::sync::Mutex<HashSet<String>>,
    leases:Mutex<HashSet<String>>,
    approvals:Mutex<HashMap<String,Arc<AtomicBool>>>,
    live:Mutex<crate::session_chat::live::Store>,
    managed:Mutex<managed::Registry>,
    managed_create:tokio::sync::Mutex<()>,
}
fn backend()->&'static Backend{static STATE:OnceLock<Backend>=OnceLock::new();STATE.get_or_init(Backend::default)}

fn allowed_window(window:&tauri::WebviewWindow)->Result<(),String>{
    if ![crate::pet_chat::LABEL,crate::island::WINDOW_LABEL,crate::HUB_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL].contains(&window.label()){
        return Err("Abre o pet para consultar esta conversa.".into());
    }Ok(())
}
fn owners(config:&crate::pet_config::Configuration)->Vec<String>{
    config.pets.iter().filter(|(_,p)|!p.provider.is_empty()).map(|(id,p)|if crate::pet_config::service(id){id.clone()}else{p.provider.clone()}).collect()
}
fn owned(app:&AppHandle,id:&str,owners:&[String])->bool{
    let chat=app.state::<crate::chat::Chat>();
    owners.iter().any(|owner|chat.conversation_for_session(owner,id).is_some())
}
fn validate(app:&AppHandle,target:&Target)->Result<(),String>{
    match target.provider.as_str(){
        "codex"=>target.validate_external("codex","agent_codex")?,
        "copilot"=>target.validate_external("copilot","agent_copilot")?,
        _=>return Err("Este provider ainda não disponibiliza uma ligação nativa a sessões de terminal.".into()),
    }
    {let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();
     if target.pet_id!=settings.main_pill&&!settings.active_integrations.contains(&target.pet_id){return Err("Este pet não está configurado.".into());}}
    let config=crate::pet_config::read()?;
    if config.pets.get(&target.pet_id).map(|p|p.provider.as_str())!=Some(target.provider.as_str()){return Err("A sessão não pertence à identidade nativa deste pet.".into());}
    if owned(app,&target.session_id,&owners(&config)){return Err("Esta sessão pertence à biblioteca Mochi. Escolhe a conversa Mochi correspondente para manter a sua memória e tarefa.".into());}
    Ok(())
}
fn updated(app:&AppHandle,target:&Target){let _=app.emit("session-chat-updated",json!({"target":target}));}
fn approval_key(thread:&str,id:&Value)->String{format!("{thread}:{}",id)}

async fn client(app:&AppHandle)->Result<Arc<rpc::Client>,String>{
    let state=backend();let mut current=state.client.lock().await;
    if let Some(client)=current.as_ref().filter(|c|c.connected()){return Ok(client.clone());}
    // Retire a disconnected observer before establishing its replacement.
    // Cache/approval locks make generation changes atomic with event mutation.
    let generation={
        let mut approvals=state.approvals.lock().unwrap();let mut live=state.live.lock().unwrap();
        let generation=state.generation.fetch_add(1,Ordering::AcqRel)+1;
        for flag in approvals.values(){flag.store(true,Ordering::Release);}approvals.clear();live.reconnect();generation
    };
    let retired=state.managed.lock().unwrap().disconnect(generation.saturating_sub(1));
    for target in retired{updated(app,&target);}
    let client=rpc::Client::connect(&rpc::socket_path(&crate::platform::home_dir())).await.map_err(|e|e.to_string())?;
    state.subscribed.lock().await.clear();
    let events=client.events();let event_client=client.clone();let app=app.clone();
    tauri::async_runtime::spawn(async move{events_loop(app,event_client,events,generation).await;});
    *current=Some(client.clone());Ok(client)
}

async fn observe(app:&AppHandle,client:&rpc::Client,target:&Target)->Result<(),String>{
    validate(app,target)?;
    let state=backend();state.tracked.lock().unwrap().insert(target.session_id.clone(),target.clone());
    let mut subscribed=state.subscribed.lock().await;
    if !subscribed.contains(&target.session_id){
        let thread=codex::read(client,&target.session_id).await?;
        if thread["status"]["type"]!="notLoaded"{
            codex::subscribe(client,target).await?;
            subscribed.insert(target.session_id.clone());
        }
    }
    Ok(())
}

async fn events_loop(app:AppHandle,client:Arc<rpc::Client>,mut events:tokio::sync::broadcast::Receiver<Value>,generation:u64){
    loop{
        if backend().generation.load(Ordering::Acquire)!=generation{break;}
        let event=match events.recv().await{
            Ok(event)=>event,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_))=>{
                {let mut live=backend().live.lock().unwrap();if backend().generation.load(Ordering::Acquire)!=generation{break;}live.gap();}
                let changed=backend().managed.lock().unwrap().gap(generation);for target in changed{updated(&app,&target);}
                let targets:Vec<_>=backend().tracked.lock().unwrap().values().cloned().collect();for target in targets{updated(&app,&target);}continue;
            },Err(_)=>break,
        };
        if backend().generation.load(Ordering::Acquire)!=generation{break;}
        let method=event["method"].as_str().unwrap_or("");let params=&event["params"];
        if method=="mochi/disconnected"{
            {let pending=backend().approvals.lock().unwrap();if backend().generation.load(Ordering::Acquire)!=generation{break;}for flag in pending.values(){flag.store(true,Ordering::Release);}}
            let retired=backend().managed.lock().unwrap().disconnect(generation);for target in retired{updated(&app,&target);}
            let targets:Vec<_>=backend().tracked.lock().unwrap().values().cloned().collect();for target in targets{updated(&app,&target);}break;
        }
        let changed=backend().managed.lock().unwrap().event(generation,&event);for target in changed{updated(&app,&target);}
        let Some(thread)=params["threadId"].as_str().or_else(||params["thread"]["id"].as_str())else{continue;};
        let target=backend().tracked.lock().unwrap().get(thread).cloned();let Some(target)=target else{continue;};
        {let mut live=backend().live.lock().unwrap();if backend().generation.load(Ordering::Acquire)!=generation{break;}live.event(&event);}
        if method=="serverRequest/resolved"{
            if let Some(flag)=backend().approvals.lock().unwrap().get(&format!("{generation}:{}",approval_key(thread,&params["requestId"]))){flag.store(true,Ordering::Release);}
        }
        if let Some(id)=event.get("id"){
            // A subscribed terminal may receive the same request. Observation
            // never answers it; only a human broker result may produce a reply.
            if validate(&app,&target).is_err(){continue;}
            let key=format!("{generation}:{}",approval_key(thread,id));let flag=Arc::new(AtomicBool::new(false));
            {let mut pending=backend().approvals.lock().unwrap();if backend().generation.load(Ordering::Acquire)!=generation{break;}if pending.contains_key(&key)||pending.len()>=128{continue;}pending.insert(key.clone(),flag.clone());}
            let app=app.clone();let client=client.clone();let method=method.to_string();let params=params.clone();let id=id.clone();
            tauri::async_runtime::spawn(async move{
                let target2=target.clone();let flag2=flag.clone();
                let answer=tauri::async_runtime::spawn_blocking(move||broker(&app,&target2,&method,&params,&flag2)).await.ok().flatten();
                if backend().generation.load(Ordering::Acquire)==generation&&!flag.load(Ordering::Acquire){if let Some(answer)=answer{let _=client.respond_guarded(id,answer,flag.clone()).await;}}
                backend().approvals.lock().unwrap().remove(&key);
            });
        }else{updated(&app,&target);}
    }
}

fn broker(app:&AppHandle,target:&Target,method:&str,params:&Value,cancel:&AtomicBool)->Option<Value>{
    if params["threadId"]!=target.session_id{return None;}
    let turn=params["turnId"].as_str()?;
    if turn.is_empty()||turn.len()>128{return None;}
    if serde_json::to_vec(params).ok()?.len()>64*1024{return None;}
    let origin=json!({"kind":"external","petId":target.pet_id,"provider":target.provider,"sessionId":target.session_id,"conversationId":null,"turnId":turn});
    match method{
        "item/commandExecution/requestApproval"|"item/fileChange/requestApproval"=>{
            let answer=crate::cli_chat::ask_target(app,&target.provider,method,params,cancel,origin)?;
            let decision=if answer["allow"]==true{"accept"}else{"decline"};
            if params["availableDecisions"].as_array().is_some_and(|choices|!choices.iter().any(|v|v==decision)){return None;}
            // Never select acceptForSession or an exec/network policy amendment.
            Some(json!({"decision":decision}))
        },
        "item/permissions/requestApproval"=>{
            let mut prompt=params.clone();prompt["reason"]=json!(format!("Permissões até ao fim deste turno (não apenas uma operação):\n{}",serde_json::to_string_pretty(&params["permissions"]).ok()?));
            let answer=crate::cli_chat::ask_target(app,&target.provider,method,&prompt,cancel,origin)?;
            Some(json!({"permissions":if answer["allow"]==true{params["permissions"].clone()}else{json!({})},"scope":"turn"}))
        },
        "item/tool/requestUserInput"=>{
            let answer=crate::cli_chat::ask_target(app,&target.provider,method,params,cancel,origin)?;
            let mut answers=serde_json::Map::new();
            for question in params["questions"].as_array()?{let id=question["id"].as_str()?;answers.insert(id.into(),json!({"answers":answer["answers"][id].as_array().cloned().unwrap_or_default()}));}
            Some(json!({"answers":answers}))
        },
        _=>None, // Unsupported native dialogs remain with their original client.
    }
}

#[path="copilot_observer.rs"]mod copilot_observer;

struct Lease(String);
impl Lease{fn acquire(id:&str)->Result<Self,String>{if !backend().leases.lock().unwrap().insert(id.into()){return Err("Já existe um envio Mochi pendente para esta sessão.".into());}Ok(Self(id.into()))}}
impl Drop for Lease{fn drop(&mut self){backend().leases.lock().unwrap().remove(&self.0);}}

#[tauri::command]
pub async fn session_chat_list(app:AppHandle,window:tauri::WebviewWindow,pet_id:String)->Result<Vec<Summary>,String>{
    allowed_window(&window)?;
    let config=crate::pet_config::read()?;let owners=owners(&config);
    {let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();if pet_id!=settings.main_pill&&!settings.active_integrations.contains(&pet_id){return Err("Este pet não está configurado.".into());}}
    let rows=match pet_id.as_str(){
        "agent_codex" if config.pets.get(&pet_id).is_some_and(|p|p.provider=="codex")=>{let client=client(&app).await?;codex::list(&client).await?},
        "agent_copilot" if config.pets.get(&pet_id).is_some_and(|p|p.provider=="copilot")=>crate::copilot_session::list().await,
        _=>return Ok(Vec::new()),
    };let mut out=Vec::new();
    for row in rows{if !owned(&app,&row.target.session_id,&owners){out.push(row);}}
    Ok(out)
}

#[tauri::command]
pub async fn session_chat_history(app:AppHandle,window:tauri::WebviewWindow,target:Target)->Result<Snapshot,String>{
    allowed_window(&window)?;validate(&app,&target)?;
    if target.provider=="copilot"{
        let snapshot=crate::copilot_session::history(&target).await?;
        copilot_observer::observe(&app,&target);return Ok(snapshot);
    }
    let client=client(&app).await?;observe(&app,&client,&target).await?;
    let snapshot=codex::snapshot(&client,&target).await?;
    Ok(backend().live.lock().unwrap().merge(snapshot))
}

#[tauri::command]
pub async fn session_chat_send(app:AppHandle,window:tauri::WebviewWindow,target:Target,query:String,model:Option<String>)->Result<Value,String>{
    allowed_window(&window)?;validate(&app,&target)?;let _lease=Lease::acquire(&format!("{}:{}",target.provider,target.session_id))?;
    if target.provider=="copilot"{
        if model.is_some(){return Err("Muda o modelo no terminal Copilot desta sessão; o pet preserva a configuração nativa.".into());}
        let result=crate::copilot_session::send(&target,&query).await;updated(&app,&target);return result;
    }
    let client=client(&app).await?;observe(&app,&client,&target).await?;
    let result=codex::send(&client,&target,&query,model.as_deref()).await;
    updated(&app,&target);result
}

#[tauri::command]
pub async fn session_chat_cancel(app:AppHandle,window:tauri::WebviewWindow,target:Target,turn:Option<String>)->Result<Value,String>{
    allowed_window(&window)?;validate(&app,&target)?;
    if target.provider=="copilot"{return crate::copilot_session::cancel(&target,turn.as_deref()).await;}
    let turn=turn.ok_or("Escolhe o turno exato para interromper a sessão.")?;
    let client=client(&app).await?;
    let result=codex::cancel(&client,&target,&turn).await;updated(&app,&target);result
}

fn managed_entry(reference:&ManagedReference)->Result<ManagedSession,String>{
    reference.target.validate_external("codex","agent_codex")?;
    backend().managed.lock().unwrap().list().into_iter().find(|entry|
        entry.handle==reference.handle&&entry.target==reference.target&&entry.generation==reference.generation
    ).ok_or_else(||"O registo não pertence a esta sessão e geração nativas.".into())
}

async fn managed_client(reference:&ManagedReference)->Result<Arc<rpc::Client>,String>{
    let state=backend();
    let current=state.client.lock().await;
    let client=current.as_ref().filter(|client|client.connected()).cloned();
    if state.generation.load(Ordering::Acquire)!=reference.generation||client.as_ref().is_none_or(|client|client.verified_endpoint().is_err()){
        state.managed.lock().unwrap().disconnect(reference.generation);
        return Err("A ligação original terminou. A sessão não foi retomada automaticamente noutro executor.".into());
    }
    client.ok_or_else(||"A ligação original terminou.".into())
}

fn managed_reference(entry:&ManagedSession)->ManagedReference{
    ManagedReference{target:entry.target.clone(),handle:entry.handle.clone(),generation:entry.generation}
}

fn managed_notice(entry:&ManagedSession,launched:bool,notice:Option<String>)->Result<ManagedSession,String>{
    let result={backend().managed.lock().unwrap().terminal_result(&entry.handle,launched,notice)};
    // A native disconnect can have invalidated the receipt while an accepted
    // operation was in flight. Return that known lost provenance, not a false
    // failure that invites the user to create/send the same work again.
    result.or_else(|_|managed_entry(&managed_reference(entry)))
}

async fn refresh_managed(client:&rpc::Client,entry:&ManagedSession)->Result<ManagedSession,String>{
    let readiness=codex::managed_readiness(client,&entry.target).await?;
    if readiness.materialized{
        backend().managed.lock().unwrap().materialized(&entry.handle,readiness.turn)
    }else{
        backend().managed.lock().unwrap().get(&entry.handle,&entry.target,entry.generation)
    }
}

#[cfg(target_os="linux")]
async fn launch_managed_terminal(entry:&ManagedSession,client:Arc<rpc::Client>)->Result<(),String>{
    let candidate=crate::platform::codex_candidates().into_iter().next().ok_or("Instala o Codex CLI para abrir esta sessão no terminal.")?;
    let mut dirs=vec![candidate.parent().ok_or("O caminho do Codex CLI é inválido.")?.to_path_buf()];
    if let Some(path)=std::env::var_os("PATH"){dirs.extend(std::env::split_paths(&path));}
    let child_path=std::env::join_paths(dirs).map_err(|_|"Não foi possível preparar o ambiente do Codex CLI.")?;
    let codex=candidate.canonicalize().map_err(|_|"Não foi possível resolver o Codex CLI.")?;
    let terminal=crate::platform::find_on_path("wezterm").ok_or("O WezTerm não está disponível para abrir o terminal nativo.")?
        .canonicalize().map_err(|_|"Não foi possível resolver o terminal.")?;
    let cwd=std::path::Path::new(&entry.cwd).canonicalize().map_err(|_|"A pasta da sessão já não está disponível.")?;
    let endpoint=client.verified_endpoint().map_err(|error|error.to_string())?;
    // Native resume restores the thread's current configuration. Its birth
    // model may have changed since creation and must never override it here.
    let args=managed::terminal_arguments(&codex,&endpoint,&cwd,&entry.target.session_id)?;
    let reference=managed_reference(entry);
    tauri::async_runtime::spawn_blocking(move||{
        use std::{process::{Command,Stdio},time::{Duration,Instant}};
        // A queued blocking worker may start after disconnect/replacement.
        // Validate the original live receipt again at the actual launch site.
        let current=backend().generation.load(Ordering::Acquire)==reference.generation
            &&client.verified_endpoint().is_ok_and(|path|path==endpoint);
        if !current{
            backend().managed.lock().unwrap().disconnect(reference.generation);
            return Err("A ligação original terminou antes da abertura. A sessão não foi aberta noutro executor.".into());
        }
        backend().managed.lock().unwrap().get(&reference.handle,&reference.target,reference.generation)?;
        let mut child=Command::new(terminal).args(args).env("PATH",child_path).current_dir(cwd).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|_|"Não foi possível iniciar o terminal. A sessão continua no daemon.".to_string())?;
        let deadline=Instant::now()+Duration::from_millis(250);
        loop{
            match child.try_wait(){
                Ok(Some(status))=>return if status.success(){Ok(())}else{Err("O terminal terminou antes de confirmar o arranque. A sessão continua no daemon.".into())},
                Ok(None) if Instant::now()<deadline=>std::thread::sleep(Duration::from_millis(20)),
                Ok(None)=>{
                    // Reap only the launcher we own. Never close a terminal,
                    // interrupt the turn or stop the independent daemon here.
                    tauri::async_runtime::spawn_blocking(move||{let _=child.wait();});
                    return Ok(());
                },
                Err(_)=>{tauri::async_runtime::spawn_blocking(move||{let _=child.wait();});return Err("Não foi possível verificar o arranque do terminal. Confirma a janela antes de repetir.".into());},
            }
        }
    }).await.map_err(|_|"A abertura do terminal não foi confirmada. A sessão nativa foi preservada.".to_string())?
}

#[cfg(not(target_os="linux"))]
async fn launch_managed_terminal(_: &ManagedSession,_:Arc<rpc::Client>)->Result<(),String>{
    Err("As sessões geridas deste fork estão disponíveis no Linux.".into())
}

async fn open_managed(app:&AppHandle,reference:&ManagedReference)->Result<ManagedSession,String>{
    let _opening=Lease::acquire(&format!("managed-open:{}",reference.handle))?;
    validate(app,&reference.target)?;
    let entry=backend().managed.lock().unwrap().get(&reference.handle,&reference.target,reference.generation)?;
    let client=managed_client(reference).await?;
    let entry=refresh_managed(&client,&entry).await?;
    if !entry.materialized{
        return Err("Envia primeiro um pedido explícito nesta conversa. O Codex ainda não consegue abrir uma sessão vazia no terminal.".into());
    }
    match launch_managed_terminal(&entry,client).await{
        Ok(())=>managed_notice(&entry,true,None),
        Err(notice)=>managed_notice(&entry,false,Some(notice)),
    }
}

/// Only the private native thread/start result can enter the provenance book.
/// The task keeps running if its invoking webview goes away after acceptance.
async fn create_managed(app:AppHandle,query:Option<String>,open_terminal:bool)->Result<ManagedSession,String>{
    let state=backend();let _creation=state.managed_create.lock().await;
    state.managed.lock().unwrap().ensure_capacity()?;
    let cwd={
        let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();
        if settings.main_pill!="agent_codex"&&!settings.active_integrations.iter().any(|id|id=="agent_codex"){
            return Err("Configura o pet Codex antes de criar uma sessão gerida.".into());
        }
        crate::cli_chat::workspace_for(&settings,"codex")?
    };
    if crate::pet_config::read()?.pets.get("agent_codex").map(|pet|pet.provider.as_str())!=Some("codex"){
        return Err("Este pet não tem a identidade Codex nativa.".into());
    }
    let cwd=cwd.to_str().ok_or("A pasta da sessão não pode ser representada pelo protocolo nativo.")?;
    let client=client(&app).await?;let generation=state.generation.load(Ordering::Acquire);
    let birth=codex::create_managed(&client,cwd).await?;
    let entry=state.managed.lock().unwrap().register(birth.target.clone(),generation,birth.cwd,birth.model)?;
    state.tracked.lock().unwrap().insert(entry.target.session_id.clone(),entry.target.clone());
    // thread/start already subscribes its creator. Resuming it again can
    // redeliver pending approvals; the TUI alone resumes when it is opened.
    let original_connection={
        // Reconnection takes client then subscribed in the same order. Hold
        // both identity and generation stable while marking the creator's
        // subscription; an old birth must not pollute a replacement observer.
        let current=state.client.lock().await;
        if state.generation.load(Ordering::Acquire)==generation&&client.connected()
            &&current.as_ref().is_some_and(|active|Arc::ptr_eq(active,&client)){
            state.subscribed.lock().await.insert(entry.target.session_id.clone());true
        }else{false}
    };
    if !original_connection{
        state.managed.lock().unwrap().disconnect(generation);updated(&app,&entry.target);
        return managed_entry(&managed_reference(&entry));
    }
    let mut entry=entry;
    let _sending=match Lease::acquire(&format!("{}:{}",entry.target.provider,entry.target.session_id)){
        Ok(lease)=>lease,
        Err(_)=>{
            return managed_notice(&entry,false,Some("A sessão foi criada e outra operação já a está a usar. O primeiro pedido não foi reenviado.".into()));
        },
    };
    if let Some(query)=query{
        if codex::send(&client,&entry.target,&query,None).await.is_err(){
            entry=managed_notice(&entry,false,Some("A sessão foi criada, mas não foi possível confirmar o primeiro envio. Consulta o histórico antes de repetir o pedido.".into()))?;
            updated(&app,&entry.target);return Ok(entry);
        }
        // Read only. Never manufacture a turn or replay input to make a draft
        // resumable; turn/start above contains the caller's explicit query.
        let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(3);
        loop{
            match refresh_managed(&client,&entry).await{
                Ok(next)=>{entry=next;if entry.materialized{break;}},
                Err(_)=>break,
            }
            if tokio::time::Instant::now()>=deadline{break;}
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    if open_terminal{
        entry=match open_managed(&app,&managed_reference(&entry)).await{
            Ok(next)=>next,
            Err(_)=>managed_notice(&entry,false,Some("A sessão e o pedido nativos foram preservados. O terminal ainda não está disponível; confirma o histórico antes de o abrir novamente.".into()))?,
        };
    }
    updated(&app,&entry.target);Ok(entry)
}

#[tauri::command]
pub async fn session_chat_managed_create(app:AppHandle,window:tauri::WebviewWindow,query:Option<String>,open_terminal:bool)->Result<ManagedSession,String>{
    allowed_window(&window)?;
    if query.as_ref().is_some_and(|query|query.trim().is_empty()||query.len()>256*1024){return Err("O primeiro pedido deve ter texto e até 256 KiB.".into());}
    if open_terminal&&query.is_none(){return Err("Escreve o primeiro pedido para abrir a mesma sessão no terminal nativo.".into());}
    tauri::async_runtime::spawn(create_managed(app,query,open_terminal)).await.map_err(|_|"O resultado da criação não foi confirmado. Consulta as sessões antes de repetir.".to_string())?
}

#[tauri::command]
pub fn session_chat_managed_list(window:tauri::WebviewWindow)->Result<Vec<ManagedSession>,String>{
    allowed_window(&window)?;Ok(backend().managed.lock().unwrap().list())
}

#[tauri::command]
pub async fn session_chat_managed_inspect(app:AppHandle,window:tauri::WebviewWindow,reference:ManagedReference)->Result<ManagedSession,String>{
    allowed_window(&window)?;validate(&app,&reference.target)?;
    let entry=managed_entry(&reference)?;
    if entry.phase==ManagedPhase::Lost{return Ok(entry);}
    let client=match managed_client(&reference).await{Ok(client)=>client,Err(_)=>return managed_entry(&reference)};
    refresh_managed(&client,&entry).await
}

#[tauri::command]
pub async fn session_chat_managed_open(app:AppHandle,window:tauri::WebviewWindow,reference:ManagedReference)->Result<ManagedSession,String>{
    allowed_window(&window)?;
    tauri::async_runtime::spawn(async move{
        managed_entry(&reference)?;
        let _lease=Lease::acquire(&format!("{}:{}",reference.target.provider,reference.target.session_id))?;
        open_managed(&app,&reference).await
    }).await
        .map_err(|_|"A abertura não foi confirmada. A sessão nativa foi preservada.".to_string())?
}

#[tauri::command]
pub async fn session_chat_managed_release(window:tauri::WebviewWindow,reference:ManagedReference)->Result<ManagedSession,String>{
    allowed_window(&window)?;
    let entry=managed_entry(&reference)?;
    let _lease=Lease::acquire(&format!("{}:{}",entry.target.provider,entry.target.session_id))?;
    if entry.phase!=ManagedPhase::Lost{
        if let Ok(client)=managed_client(&reference).await{
            if codex::managed_readiness(&client,&entry.target).await?.busy{
                return Err("A sessão ainda está a trabalhar. Não foi cancelada nem libertada; usa a interrupção explícita se quiseres parar o turno.".into());
            }
        }
    }
    // Forget provenance only. No interrupt, unsubscribe, terminal close or
    // daemon stop; ordinary external-history controls remain independent.
    backend().managed.lock().unwrap().release(&reference.handle,&reference.target,reference.generation)
}
