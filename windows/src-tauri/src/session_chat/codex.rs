//! Read the daemon's complete public conversation, including every history page.
use std::collections::HashSet;
use serde_json::{json,Value};
use super::{rpc::Client,types::{Target,Origin,Snapshot,Summary},history};

const MAX_HISTORY_BYTES:usize=128*1024*1024;
const SOURCES:&[&str]=&["cli","vscode","exec","appServer","subAgent","subAgentReview","subAgentCompact","subAgentThreadSpawn","subAgentOther","unknown"];

fn checked_thread(value:Value,id:&str)->Result<Value,String>{
    let thread=value.get("thread").ok_or("O daemon Codex não devolveu a sessão pedida.")?;
    if thread["id"]!=id{return Err("O daemon devolveu outra sessão. A operação foi interrompida.".into());}
    Ok(thread.clone())
}

pub async fn read(client:&Client,id:&str)->Result<Value,String>{
    checked_thread(client.call("thread/read",json!({"threadId":id,"includeTurns":false})).await.map_err(|e|e.to_string())?,id)
}

pub async fn subscribe(client:&Client,target:&Target)->Result<Value,String>{
    target.validate_external("codex","agent_codex")?;
    // No model, permissions, cwd, instructions, history or path override.
    checked_thread(client.call("thread/resume",json!({"threadId":target.session_id,"excludeTurns":true})).await.map_err(|e|e.to_string())?,&target.session_id)
}

pub async fn list(client:&Client)->Result<Vec<Summary>,String>{
    let mut out=Vec::new();let mut seen=HashSet::new();let mut cursors=HashSet::new();
    for archived in [false,true]{
        let mut cursor=Value::Null;
        loop{
            let page=client.call("thread/list",json!({"limit":100,"cursor":cursor,"sortKey":"updated_at","sortDirection":"desc","sourceKinds":SOURCES,"archived":archived})).await.map_err(|e|e.to_string())?;
    let rows=page["data"].as_array().ok_or("O inventário Codex está incompleto.")?;
            for thread in rows{
                let id=thread["id"].as_str().ok_or("A sessão Codex não tem ID.")?;
                let target=Target{pet_id:"agent_codex".into(),provider:"codex".into(),origin:Origin::External,session_id:id.into(),conversation_id:None};
                target.validate_external("codex","agent_codex")?;
                if seen.insert(id.to_string()){
                    let mut summary=history::summary(thread,target)?;
                    if archived{summary.state="archived".into();summary.capabilities.send=false;summary.capabilities.cancel=false;}
                    out.push(summary);
                }
                if out.len()>100_000{return Err("Há demasiadas sessões para este inventário. Usa a gestão nativa para filtrar o arquivo.".into());}
            }
            let Some(next)=page["nextCursor"].as_str()else{break;};
            if next.is_empty()||!cursors.insert(format!("{archived}:{next}")){return Err("A paginação do inventário Codex repetiu o cursor; a lista não foi truncada.".into());}
            cursor=json!(next);
        }
    }
    // Loaded ephemeral threads and new subagents can be absent from stored listings.
    let mut cursor=Value::Null;let mut pages=HashSet::new();
    loop{
        let page=client.call("thread/loaded/list",json!({"limit":100,"cursor":cursor})).await.map_err(|e|e.to_string())?;
        for id in page["data"].as_array().ok_or("O daemon não listou as sessões carregadas.")?{
            let id=id.as_str().ok_or("O daemon devolveu um ID de sessão inválido.")?;
            if seen.insert(id.into()){
                let thread=read(client,id).await?;
                out.push(history::summary(&thread,Target{pet_id:"agent_codex".into(),provider:"codex".into(),origin:Origin::External,session_id:id.into(),conversation_id:None})?);
            }
        }
        let Some(next)=page["nextCursor"].as_str()else{break;};
        if next.is_empty()||!pages.insert(next.to_string()){return Err("O inventário carregado repetiu o cursor.".into());}cursor=json!(next);
    }
    out.sort_by_key(|row|std::cmp::Reverse(row.updated_at));Ok(out)
}

pub async fn turns(client:&Client,thread:&Value)->Result<Vec<Value>,String>{
    let id=thread["id"].as_str().ok_or("A sessão Codex não tem ID.")?;
    if thread["historyMode"].as_str().unwrap_or("legacy")=="legacy"{
        let thread=checked_thread(client.call("thread/read",json!({"threadId":id,"includeTurns":true})).await.map_err(|e|e.to_string())?,id)?;
        let turns=thread["turns"].as_array().ok_or("O histórico legado Codex está incompleto.")?;
        if serde_json::to_vec(turns).map_err(|_|"Histórico Codex inválido.")?.len()>MAX_HISTORY_BYTES{return Err("O histórico excede128 MiB. Abre-o no cliente nativo; não foi apresentado um recorte como histórico completo.".into());}
        return Ok(turns.clone());
    }
    let mut turns=Vec::new();let mut cursor=Value::Null;let mut seen=HashSet::new();let mut bytes=0usize;
    loop{
        let page=match client.call("thread/turns/list",json!({"threadId":id,"limit":50,"sortDirection":"asc","itemsView":"full","cursor":cursor})).await{
            Ok(page)=>page,
            Err(error) if error.code==Some(-32600)&&error.native_message.as_deref().is_some_and(|m|m.contains("not materialized yet")&&m.contains("before first user message"))=>return Ok(Vec::new()),
            Err(error)=>return Err(error.to_string()),
        };
        let data=page["data"].as_array().ok_or("A página de histórico Codex está incompleta.")?;
        bytes=bytes.saturating_add(serde_json::to_vec(data).map_err(|_|"Histórico Codex inválido.")?.len());
        if bytes>MAX_HISTORY_BYTES{return Err("O histórico excede128 MiB. Abre-o no cliente nativo; não foi apresentado um recorte como histórico completo.".into());}
        turns.extend(data.iter().cloned());
        let Some(next)=page["nextCursor"].as_str()else{break;};
        if next.is_empty()||!seen.insert(next.to_string()){return Err("O histórico repetiu o cursor. A leitura completa não pôde ser confirmada.".into());}cursor=json!(next);
    }
    Ok(turns)
}

pub async fn snapshot(client:&Client,target:&Target)->Result<Snapshot,String>{
    target.validate_external("codex","agent_codex")?;
    let thread=read(client,&target.session_id).await?;
    if thread["ephemeral"]==true{return Err("O daemon Codex não fornece histórico das sessões temporárias. Consulta esta sessão no cliente que a criou; não foi criado um replay noutra sessão.".into());}
    let turns=turns(client,&thread).await?;
    // Read current runtime status after history paging, without changing executor.
    let current=read(client,&target.session_id).await?;
    history::snapshot(&current,target.clone(),&turns)
}

pub async fn send(client:&Client,target:&Target,query:&str,model:Option<&str>)->Result<Value,String>{
    target.validate_external("codex","agent_codex")?;
    if query.trim().is_empty()||query.len()>256*1024{return Err("A mensagem deve ter texto e até256 KiB.".into());}
    let thread=read(client,&target.session_id).await?;
    if thread["ephemeral"]==true{return Err("Esta sessão temporária não fornece o histórico necessário para responder por este painel.".into());}
    if thread["canAcceptDirectInput"]!=true{return Err("Esta sessão não aceita mensagens diretas no daemon. Abre a sessão no cliente nativo.".into());}
    if thread["status"]["type"]!="idle"{return Err("A sessão está ocupada ou indisponível. Atualiza o histórico antes de enviar.".into());}
    let mut params=json!({"threadId":target.session_id,"input":[{"type":"text","text":query}],"clientUserMessageId":format!("mochi-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos())});
    if let Some(model)=model.filter(|m|!m.is_empty()){
        if model.len()>512||model.starts_with('@')||model.chars().any(char::is_control){return Err("Modelo nativo inválido.".into());}
        params["model"]=json!(model);
    }
    client.call("turn/start",params).await.map_err(|e|e.to_string())
}

pub async fn cancel(client:&Client,target:&Target,turn:&str)->Result<Value,String>{
    target.validate_external("codex","agent_codex")?;
    if turn.is_empty()||turn.len()>128||turn.chars().any(char::is_control){return Err("Falta o identificador exato do turno a interromper.".into());}
    let snapshot=snapshot(client,target).await?;
    if !snapshot.busy||snapshot.turn.as_deref()!=Some(turn){return Err("O turno selecionado já terminou ou mudou. A sessão atual não foi interrompida.".into());}
    client.call("turn/interrupt",json!({"threadId":target.session_id,"turnId":turn})).await.map_err(|e|e.to_string())
}

pub struct ManagedBirth {
    pub target: Target,
    pub cwd: String,
    pub model: Option<String>,
}

/// Native account and policy are used directly. No copied transcript,
/// developer instructions, client-side tools, approval bypass or inference.
pub async fn create_managed(client:&Client,cwd:&str)->Result<ManagedBirth,String>{
    if !std::path::Path::new(cwd).is_absolute()||cwd.len()>4096||cwd.chars().any(char::is_control){
        return Err("A pasta da sessão nativa é inválida.".into());
    }
    client.verified_endpoint().map_err(|error|error.to_string())?;
    let account=client.call("account/read",json!({"refreshToken":false})).await.map_err(|error|error.to_string())?;
    if account["account"]["type"]!="chatgpt"{
        return Err(crate::codex_chat::rpc::SIGN_IN.into());
    }
    let response=client.call("thread/start",json!({"cwd":cwd,"ephemeral":false,"modelProvider":"openai"})).await.map_err(|error|error.to_string())?;
    let thread=response.get("thread").ok_or("O daemon não confirmou a sessão criada. Nenhum pedido foi enviado ao modelo.")?;
    let id=thread["id"].as_str().ok_or("O daemon não confirmou o ID criado. Nenhum pedido foi enviado ao modelo.")?;
    let target=Target{pet_id:"agent_codex".into(),provider:"codex".into(),origin:Origin::External,session_id:id.into(),conversation_id:None};
    target.validate_external("codex","agent_codex")?;
    if thread["ephemeral"]==true||thread["cwd"]!=cwd||response["modelProvider"].as_str().is_some_and(|provider|provider!="openai"){
        return Err("O daemon não confirmou a pasta e a conta da sessão. Nenhum pedido foi enviado ao modelo.".into());
    }
    let model=response["model"].as_str().or_else(||thread["model"].as_str()).filter(|model|!model.is_empty()).map(str::to_string);
    Ok(ManagedBirth{target,cwd:cwd.into(),model})
}

pub struct ManagedReadiness {
    pub materialized: bool,
    pub busy: bool,
    pub turn: Option<String>,
}

/// Empty native thread/start drafts have no rollout and cannot be resumed by
/// the TUI. Check storage without resuming the creator or starting a fake turn.
pub async fn managed_readiness(client:&Client,target:&Target)->Result<ManagedReadiness,String>{
    target.validate_external("codex","agent_codex")?;
    let thread=read(client,&target.session_id).await?;
    if thread["ephemeral"]==true||matches!(thread["status"]["type"].as_str(),Some("notLoaded"|"systemError")){
        return Err("Esta sessão já não tem execução carregada no daemon original.".into());
    }
    let page=match client.call("thread/turns/list",json!({"threadId":target.session_id,"limit":1,"sortDirection":"desc","itemsView":"notLoaded"})).await{
        Ok(page)=>page,
        Err(error) if error.code==Some(-32600)&&error.native_message.as_deref().is_some_and(|message|message.contains("not materialized yet")&&message.contains("before first user message"))=>{
            return Ok(ManagedReadiness{materialized:false,busy:thread["status"]["type"]=="active",turn:None});
        },
        Err(error)=>return Err(error.to_string()),
    };
    let turns=page["data"].as_array().ok_or("O daemon não confirmou a materialização da sessão.")?;
    let turn=turns.first().filter(|turn|turn["status"]=="inProgress").and_then(|turn|turn["id"].as_str()).map(str::to_string);
    Ok(ManagedReadiness{materialized:!turns.is_empty(),busy:thread["status"]["type"]=="active",turn})
}

#[cfg(test)]
mod managed_tests {
    use super::*;
    use std::{path::PathBuf, sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}}, time::{Duration, SystemTime, UNIX_EPOCH}};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::UnixListener;
    use tokio_tungstenite::tungstenite::Message;

    const THREAD: &str = "019a734b-0000-7000-8000-000000000001";
    const TREE: &str = "019a734b-0000-7000-8000-000000000002";
    const CWD: &str = "/fixture/managed-workspace";

    fn target() -> Target {
        Target { pet_id:"agent_codex".into(), provider:"codex".into(), origin:Origin::External,
            session_id:THREAD.into(), conversation_id:None }
    }
    fn native_thread() -> Value {
        json!({"id":THREAD,"sessionId":TREE,"cwd":CWD,"ephemeral":false,"modelProvider":"openai",
            "model":"native-default","status":{"type":"idle"},"canAcceptDirectInput":true})
    }

    struct Replies { account:Value, birth:Value, read:Value, page:Value }
    impl Default for Replies {
        fn default() -> Self {
            Self { account:json!({"account":{"type":"chatgpt"}}),
                birth:json!({"thread":native_thread(),"modelProvider":"openai","model":"native-default"}),
                read:json!({"thread":native_thread()}), page:json!({"result":{"data":[],"nextCursor":null}}) }
        }
    }

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT:AtomicU64=AtomicU64::new(0);
            let nonce=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path=std::env::temp_dir().join(format!("cmg-{}-{nonce:x}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap(); Self(path)
        }
    }
    impl Drop for Scratch { fn drop(&mut self) { let _=std::fs::remove_dir_all(&self.0); } }

    struct Fixture {
        client:Option<Arc<Client>>,
        server:tokio::task::JoinHandle<()>,
        _scratch:Scratch,
        requests:Arc<Mutex<Vec<Value>>>,
    }
    impl Fixture {
        async fn new(replies:Replies) -> Self {
            let scratch=Scratch::new(); let socket=scratch.0.join("control.sock");
            let listener=UnixListener::bind(&socket).unwrap();
            std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
            let requests:Arc<Mutex<Vec<Value>>>=Arc::default(); let seen=requests.clone();
            let initialized=Arc::new(tokio::sync::Notify::new()); let ready=initialized.clone();
            let server=tokio::spawn(async move {
                let (stream,_)=listener.accept().await.unwrap();
                let mut ws=tokio_tungstenite::accept_async(stream).await.unwrap();
                while let Some(message)=ws.next().await {
                    let Ok(message)=message else { break; };
                    let Message::Text(text)=message else { if matches!(message,Message::Close(_)){break;} continue; };
                    let request:Value=serde_json::from_str(&text).unwrap();
                    seen.lock().unwrap().push(request.clone());
                    let method=request["method"].as_str().unwrap_or("");
                    if method=="initialized" { ready.notify_one(); continue; }
                    let Some(id)=request.get("id") else { continue; };
                    let mut response=match method {
                        "initialize"=>json!({"result":{}}),
                        "account/read"=>json!({"result":replies.account}),
                        "thread/start"=>json!({"result":replies.birth}),
                        "thread/read"=>json!({"result":replies.read}),
                        "thread/turns/list"=>replies.page.clone(),
                        _=>json!({"error":{"code":-32601,"message":"Unexpected fixture request"}}),
                    };
                    response["id"]=id.clone();
                    if ws.send(Message::Text(response.to_string().into())).await.is_err(){break;}
                }
            });
            // Construct the RAII guard before awaiting connection/initialization.
            let mut fixture=Self { client:None,server,_scratch:scratch,requests };
            fixture.client=Some(Client::connect(&socket).await.unwrap());
            tokio::time::timeout(Duration::from_secs(5),initialized.notified()).await.unwrap();
            fixture
        }
        fn client(&self) -> &Client { self.client.as_deref().unwrap() }
        fn calls(&self) -> Vec<Value> { self.requests.lock().unwrap().clone() }
        fn methods(&self) -> Vec<String> {
            self.calls().into_iter().map(|request|request["method"].as_str().unwrap().to_owned()).collect()
        }
        fn only_calls(&self, expected:&[&str]) {
            assert_eq!(self.methods(),expected.iter().map(|method|(*method).to_owned()).collect::<Vec<_>>());
            let calls=self.calls();
            assert_eq!(calls[0]["method"],"initialize");
            assert_eq!(calls[0]["params"]["capabilities"]["experimentalApi"],true);
            assert_eq!(calls[1]["method"],"initialized");
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) { self.server.abort(); drop(self.client.take()); }
    }

    #[tokio::test]
    async fn managed_creation_refuses_missing_or_api_key_auth_before_thread_start() {
        for account in [Value::Null,json!({"type":"apiKey"}),json!({"type":"unknown"})] {
            let fixture=Fixture::new(Replies { account:json!({"account":account}),..Replies::default() }).await;
            let error=create_managed(fixture.client(),CWD).await.err().unwrap();
            assert_eq!(error,crate::codex_chat::rpc::SIGN_IN);
            fixture.only_calls(&["initialize","initialized","account/read"]);
        }
    }

    #[tokio::test]
    async fn managed_creation_uses_native_thread_id_and_only_metadata_start_params() {
        let fixture=Fixture::new(Replies::default()).await;
        let birth=create_managed(fixture.client(),CWD).await.unwrap();
        assert_eq!(birth.target,target()); assert_ne!(birth.target.session_id,TREE);
        assert_eq!(birth.cwd,CWD); assert_eq!(birth.model.as_deref(),Some("native-default"));
        fixture.only_calls(&["initialize","initialized","account/read","thread/start"]);
        let calls=fixture.calls();
        assert_eq!(calls[2]["params"],json!({"refreshToken":false}));
        assert_eq!(calls[3]["params"],json!({"cwd":CWD,"ephemeral":false,"modelProvider":"openai"}));
        // Exact equality excludes history, instructions, tools, model overrides,
        // approval/sandbox bypasses and any turn or inference request.
    }

    #[tokio::test]
    async fn managed_creation_rejects_invalid_native_birth_metadata_and_uuid() {
        let original=Replies::default().birth;
        let mut ephemeral=original.clone();ephemeral["thread"]["ephemeral"]=json!(true);
        let mut cwd=original.clone();cwd["thread"]["cwd"]=json!("/fixture/other-workspace");
        let mut provider=original.clone();provider["modelProvider"]=json!("other-provider");
        let mut id=original.clone();id["thread"]["id"]=json!("not-a-native-uuid");
        let mut absent=original;absent["thread"].as_object_mut().unwrap().remove("id");
        for birth in [ephemeral,cwd,provider,id,absent] {
            let fixture=Fixture::new(Replies { birth,..Replies::default() }).await;
            assert!(create_managed(fixture.client(),CWD).await.is_err());
            fixture.only_calls(&["initialize","initialized","account/read","thread/start"]);
        }
    }

    #[tokio::test]
    async fn managed_local_path_and_target_guards_precede_native_requests() {
        let fixture=Fixture::new(Replies::default()).await;
        for cwd in [String::new(),"relative/workspace".into(),"/fixture/bad\nworkspace".into(),format!("/{}","x".repeat(4096))] {
            assert!(create_managed(fixture.client(),&cwd).await.is_err());
        }
        let mut invalid=target();invalid.session_id="invalid-id".into();
        assert!(managed_readiness(fixture.client(),&invalid).await.is_err());
        let mut invalid=target();invalid.provider="copilot".into();
        assert!(managed_readiness(fixture.client(),&invalid).await.is_err());
        fixture.only_calls(&["initialize","initialized"]);
    }

    #[tokio::test]
    async fn managed_empty_draft_does_not_resume_or_start_a_turn_to_materialize() {
        for active in [false,true] {
            let mut replies=Replies::default();
            if active { replies.read["thread"]["status"]=json!({"type":"active","activeFlags":[]}); }
            replies.page=json!({"error":{"code":-32600,"message":"Thread is not materialized yet before first user message"}});
            let fixture=Fixture::new(replies).await;
            let readiness=managed_readiness(fixture.client(),&target()).await.unwrap();
            assert!(!readiness.materialized); assert_eq!(readiness.busy,active); assert!(readiness.turn.is_none());
            fixture.only_calls(&["initialize","initialized","thread/read","thread/turns/list"]);
        }
    }

    #[tokio::test]
    async fn managed_loaded_running_turn_uses_exact_id_and_bounded_metadata_page() {
        let mut replies=Replies::default();
        replies.read["thread"]["status"]=json!({"type":"active","activeFlags":["waitingOnApproval"]});
        replies.page=json!({"result":{"data":[{"id":"exact-native-turn","status":"inProgress","items":[]}],"nextCursor":null}});
        let fixture=Fixture::new(replies).await;
        let readiness=managed_readiness(fixture.client(),&target()).await.unwrap();
        assert!(readiness.materialized); assert!(readiness.busy); assert_eq!(readiness.turn.as_deref(),Some("exact-native-turn"));
        fixture.only_calls(&["initialize","initialized","thread/read","thread/turns/list"]);
        assert_eq!(fixture.calls()[2]["params"],json!({"threadId":THREAD,"includeTurns":false}));
        assert_eq!(fixture.calls()[3]["params"],json!({"threadId":THREAD,"limit":1,"sortDirection":"desc","itemsView":"notLoaded"}));
    }

    #[tokio::test]
    async fn managed_unloaded_ephemeral_or_mismatched_thread_is_refused_before_paging() {
        let original=Replies::default().read;
        let mut unloaded=original.clone();unloaded["thread"]["status"]=json!({"type":"notLoaded"});
        let mut failed=original.clone();failed["thread"]["status"]=json!({"type":"systemError"});
        let mut ephemeral=original.clone();ephemeral["thread"]["ephemeral"]=json!(true);
        let mut wrong=original;wrong["thread"]["id"]=json!(TREE);
        for read in [unloaded,failed,ephemeral,wrong] {
            let fixture=Fixture::new(Replies { read,..Replies::default() }).await;
            assert!(managed_readiness(fixture.client(),&target()).await.is_err());
            fixture.only_calls(&["initialize","initialized","thread/read"]);
        }
    }

    #[tokio::test]
    async fn managed_readiness_distinguishes_empty_completed_and_unrelated_native_errors() {
        for page in [json!({"error":{"code":-32600,"message":"A different native failure"}}),
            json!({"error":{"code":-32601,"message":"Thread is not materialized yet before first user message"}}),
            json!({"result":{"data":null}})] {
            let fixture=Fixture::new(Replies { page,..Replies::default() }).await;
            assert!(managed_readiness(fixture.client(),&target()).await.is_err());
            fixture.only_calls(&["initialize","initialized","thread/read","thread/turns/list"]);
        }
        for (data,materialized) in [(json!([]),false),(json!([{"id":"finished-turn","status":"completed"}]),true)] {
            let fixture=Fixture::new(Replies { page:json!({"result":{"data":data,"nextCursor":null}}),..Replies::default() }).await;
            let readiness=managed_readiness(fixture.client(),&target()).await.unwrap();
            assert_eq!(readiness.materialized,materialized); assert!(!readiness.busy); assert!(readiness.turn.is_none());
            fixture.only_calls(&["initialize","initialized","thread/read","thread/turns/list"]);
        }
    }
}
