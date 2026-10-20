//! Explicit floating conversation. Closing it never cancels an agent task.
use serde::{Serialize,Deserialize};
use std::sync::{Mutex,atomic::{AtomicU64,Ordering}};
use tauri::{AppHandle,Emitter,Manager,WebviewUrl,WebviewWindowBuilder,LogicalSize};
use crate::session_chat::types::{Origin,Target};
pub const LABEL:&str="pet-chat";
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Session {pub id:String,pub backend:String,pub title:String,pub cwd:String,#[serde(default)]pub steps:Vec<String>,#[serde(default)]pub final_line:String}
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]pub struct Request {pub pet:String,pub session:Option<Session>,#[serde(default)]pub show_sessions:bool,#[serde(default)]pub target:Option<Target>,#[serde(default)]pub request_id:Option<String>}
#[derive(Clone,Default,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Context {pub pet:String,pub name:String,pub provider:String,pub session:Option<Session>,pub conversation_id:Option<String>,pub show_sessions:bool,pub target:Option<Target>}
#[derive(Default)]struct ChatWindow {context:Mutex<Context>,rect:Mutex<Option<Rect>>,opening:AtomicU64,open_gate:tokio::sync::Mutex<()>}
fn logical_provider(pet:&str,configured:&str)->String {
 if crate::pet_config::service(pet){pet.into()}else{configured.into()}
}
/// Opening an exact destination reads its identity; it never restores, resumes
/// or creates a conversation as a side effect of following a help request.
fn selected_conversation(pet:&str,provider:&str,session:Option<&Session>,target:Option<&Target>,chat:&crate::chat::Chat,owners:&[String])->Result<Option<String>,String>{
 let Some(target)=target else{return Ok(session.and_then(|s|chat.conversation_for_session(provider,&s.id)));};
 if target.pet_id!=pet||target.provider!=provider||target.session_id.is_empty()||target.session_id.len()>512{
  return Err("O destino não pertence ao pet escolhido.".into());
 }
 match target.origin{
  Origin::Mochi=>{
   if target.conversation_id.as_deref().is_some_and(|id|id!=target.session_id)
       ||!chat.has_public_conversation(provider,&target.session_id){
    return Err("A conversa indicada não pertence a este pet ou já não existe.".into());
   }
   if session.is_some_and(|s|chat.conversation_for_session(provider,&s.id).as_deref()!=Some(target.session_id.as_str())){
    return Err("A sessão e a conversa indicadas não correspondem.".into());
   }
   Ok(Some(target.session_id.clone()))
  }
  Origin::External=>{
   match provider{
    "codex"=>target.validate_external("codex","agent_codex")?,
    "copilot"=>target.validate_external("copilot","agent_copilot")?,
    _=>return Err("Este pet ainda não disponibiliza uma ligação nativa a sessões de terminal.".into()),
   }
   if owners.iter().any(|owner|chat.conversation_for_session(owner,&target.session_id).is_some()){
    return Err("Esta sessão pertence à biblioteca Mochi. Abre a conversa Mochi correspondente.".into());
   }
   if session.is_some_and(|s|s.id!=target.session_id){return Err("As sessões indicadas não correspondem.".into());}
   Ok(None)
  }
 }
}
pub fn create(app:&AppHandle){
 app.manage(ChatWindow::default());
 #[allow(unused_mut)]let mut url=WebviewUrl::App("pet-chat.html".into());
 #[cfg(dev)]if let Some(mut base)=app.config().build.dev_url.clone(){base.set_path("/pet-chat.html");url=WebviewUrl::External(base);}
 match WebviewWindowBuilder::new(app,LABEL,url).additional_browser_args(crate::BROWSER_ARGS).title("Pets · Conversa")
  .inner_size(760.0,620.0).min_inner_size(320.0,280.0).decorations(false).transparent(true).shadow(false)
  .always_on_top(true).skip_taskbar(true).visible_on_all_workspaces(true).focused(false).visible(false).disable_drag_drop_handler().build(){
  Ok(win)=>{
   #[cfg(target_os="linux")]if !crate::platform::prepare_pet_chat(&win){let _=win.destroy();return;}
   #[cfg(target_os="linux")]if let Err(error)=crate::linux_documents::install(&win){crate::log::line(format!("pet chat drop: {error}"));}
   let app=app.clone();win.on_window_event(move|event|if let tauri::WindowEvent::CloseRequested{api,..}=event{api.prevent_close();pet_chat_hide(app.clone());});
  },Err(e)=>crate::log::line(format!("pet floating chat: {e}")),
 }
}
#[tauri::command]pub fn pet_chat_context(app:AppHandle)->Context{app.state::<ChatWindow>().context.lock().unwrap().clone()}
#[tauri::command]pub fn pet_chat_hide(app:AppHandle){
 let generation=app.state::<ChatWindow>().opening.fetch_add(1,Ordering::AcqRel)+1;
 let current=app.clone();let _=app.run_on_main_thread(move||{
  if current.state::<ChatWindow>().opening.load(Ordering::Acquire)!=generation{return;}
  if let Some(w)=current.get_webview_window(LABEL){crate::platform::set_activating(&w,false);let _=w.emit("pet-chat-hidden",());let _=w.hide();}
 });
}
#[derive(Clone,Copy,Serialize,Deserialize)]pub struct Rect{pub x:f64,pub y:f64,pub width:f64,pub height:f64}
fn fit(r:Rect,width:f64,height:f64)->Rect {
 let w=r.width.clamp(320.0,(width-16.0).max(320.0));let h=r.height.clamp(280.0,(height-56.0).max(280.0));
 Rect{x:r.x.clamp(8.0,(width-w-8.0).max(8.0)),y:r.y.clamp(48.0,(height-h-8.0).max(48.0)),width:w,height:h}
}
async fn layout(app:&AppHandle,r:Rect)->Result<Rect,String>{
 if ![r.x,r.y,r.width,r.height].iter().all(|v|v.is_finite()){return Err("Posição inválida.".into());}
 let pref=app.state::<crate::Shared>().settings.lock().unwrap().screen.clone();let s=crate::island::screen_info(app,&pref);let r=fit(r,s.width,s.height);
 let win=app.get_webview_window(LABEL).ok_or("A conversa flutuante não está disponível.")?;let other=app.clone();let(tx,rx)=tokio::sync::oneshot::channel();
 app.run_on_main_thread(move||{let _=win.set_size(LogicalSize::new(r.width,r.height));
  #[cfg(target_os="linux")]if crate::platform::pet_chat_normal_window(){crate::platform::place_pet_chat(s.x+r.x,s.y+r.y,r.width,r.height);}else{crate::platform::place_pet_panel(&other,&win,r.x,r.y);}
  #[cfg(not(target_os="linux"))]let _=win.set_position(tauri::LogicalPosition::new(s.x+r.x,s.y+r.y));
  let _=tx.send(r);
 }).map_err(|_|"Não foi possível posicionar a conversa.")?;
 let fitted=rx.await.map_err(|_|"A conversa não respondeu.")?;*app.state::<ChatWindow>().rect.lock().unwrap()=Some(fitted);Ok(fitted)
}
#[tauri::command]pub async fn pet_chat_layout(app:AppHandle,window:tauri::WebviewWindow,rect:Rect)->Result<Rect,String>{if window.label()!=LABEL{return Err("Abre a conversa para a mover.".into());}layout(&app,rect).await}
#[tauri::command]pub fn pet_chat_rect(app:AppHandle)->Option<Rect>{*app.state::<ChatWindow>().rect.lock().unwrap()}

#[derive(Serialize)]pub struct Point{x:f64,y:f64}
#[cfg(target_os="linux")]
fn pointer()->Result<Point,String>{
 use std::{io::{Read,Write},os::unix::{net::UnixStream,fs::{MetadataExt,FileTypeExt}},time::Duration};
 let signature=std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_|"O compositor não disponibiliza a posição do rato.")?;
 if signature.is_empty()||signature.len()>256||!signature.bytes().all(|c|c.is_ascii_alphanumeric()||b"_-.".contains(&c)){return Err("A sessão do compositor não é válida.".into());}
 let uid=unsafe{libc::geteuid()};let path=std::path::PathBuf::from(format!("/run/user/{uid}/hypr")).join(signature).join(".socket.sock");
 let meta=std::fs::metadata(&path).map_err(|_|"O compositor não está disponível para deslocar a conversa.")?;
 if !meta.file_type().is_socket()||meta.uid()!=uid{return Err("O socket do compositor não pertence à sessão atual.".into());}
 let mut socket=UnixStream::connect(path).map_err(|_|"Não foi possível ler a posição do rato.")?;
 let limit=Some(Duration::from_millis(150));let _=socket.set_read_timeout(limit);let _=socket.set_write_timeout(limit);
 socket.write_all(b"j/cursorpos").map_err(|_|"O compositor não respondeu.")?;
 let mut bytes=Vec::new();socket.take(4096).read_to_end(&mut bytes).map_err(|_|"O compositor não devolveu a posição do rato.")?;
 let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|"Posição do rato inválida.")?;
 let(x,y)=(value["x"].as_f64().ok_or("Posição do rato inválida.")?,value["y"].as_f64().ok_or("Posição do rato inválida.")?);
 if !x.is_finite()||!y.is_finite(){return Err("Posição do rato inválida.".into());}Ok(Point{x,y})
}
#[tauri::command]pub async fn pet_chat_pointer(window:tauri::WebviewWindow)->Result<Point,String>{
 if window.label()!=LABEL{return Err("Desloca a conversa pelo seu cabeçalho.".into());}
 #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(pointer).await.map_err(|_|"A deslocação foi interrompida.".to_string())?}
 #[cfg(not(target_os="linux"))]{Err("Esta deslocação está disponível na sessão Linux.".into())}
}
fn validate_help(request_id:Option<&str>,target:Option<&Target>)->Result<(),String>{
 if let Some(id)=request_id{
  if id.is_empty()||id.len()>128||!id.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-')||target.is_none(){
   return Err("O pedido de ajuda não tem um destino válido.".into());
  }
 }Ok(())
}
fn help_is_live(app:&AppHandle,request_id:Option<&str>,target:Option<&Target>)->Result<bool,String>{
 let Some(id)=request_id else{return Ok(true);};
 let target=target.ok_or("O pedido de ajuda não tem um destino válido.")?;
 Ok(crate::cli_chat::with_pending_help(&app.state::<crate::chat::Chat>(),id,target,||())?.is_some())
}
#[tauri::command]pub async fn pet_chat_open(app:AppHandle,request:Request)->Result<bool,String>{
 if request.pet=="integration_spotify"{return Err("O Spoty controla música e som. Abre os seus controlos na ilha.".into());}
 validate_help(request.request_id.as_deref(),request.target.as_ref())?;
 let config=crate::pet_config::read()?;let p=config.pets.get(&request.pet).ok_or("Pet desconhecido.")?;
 {let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();if request.pet!=s.main_pill&&!s.active_integrations.contains(&request.pet){return Err("Este pet não está configurado.".into());}}
 if let Some(s)=&request.session{if s.id.is_empty()||s.id.len()>512||s.backend.len()>64||s.title.len()>2048||s.cwd.len()>4096||s.steps.len()>64||s.steps.iter().any(|s|s.len()>2048)||s.final_line.len()>16000{return Err("A sessão não tem metadados válidos.".into());}}
 if let Some(s)=&request.session {
  if crate::cli_chat::backend(&p.provider)!=Some(s.backend.as_str()){return Err("Esta sessão não pertence ao provider deste pet.".into());}
 }
 let provider=logical_provider(&request.pet,&p.provider);
 let owners=config.pets.iter().map(|(id,p)|logical_provider(id,&p.provider)).collect::<Vec<_>>();
 let conversation_id=selected_conversation(&request.pet,&provider,request.session.as_ref(),request.target.as_ref(),&app.state::<crate::chat::Chat>(),&owners)?;
 if !help_is_live(&app,request.request_id.as_deref(),request.target.as_ref())?{return Ok(false);}
 let request_id=request.request_id;
 let ctx=Context{pet:request.pet,name:p.name.clone(),provider,session:request.session,conversation_id,show_sessions:request.show_sessions,target:request.target};
 let state=app.state::<ChatWindow>();let generation=state.opening.fetch_add(1,Ordering::AcqRel)+1;
 let _gate=state.open_gate.lock().await;
 if state.opening.load(Ordering::Acquire)!=generation||!help_is_live(&app,request_id.as_deref(),ctx.target.as_ref())?{return Ok(false);}
 let pref=app.state::<crate::Shared>().settings.lock().unwrap().screen.clone();let s=crate::island::screen_info(&app,&pref);
 let previous=*app.state::<ChatWindow>().rect.lock().unwrap();
 let rect=layout(&app,previous.unwrap_or(Rect{x:(s.width-760.0)/2.0,y:80.0,width:760.0,height:620.0})).await?;
 if !help_is_live(&app,request_id.as_deref(),ctx.target.as_ref())?{return Ok(false);}
 // Context, event and visibility land together on GTK. A newer open/close
 // invalidates a layout that was already queued, before it can reopen A over B.
 let current=app.clone();let(tx,rx)=tokio::sync::oneshot::channel();
 app.run_on_main_thread(move||{
  let state=current.state::<ChatWindow>();
  let present=||->Result<bool,String>{
   if state.opening.load(Ordering::Acquire)!=generation{return Ok(false);}
   let win=current.get_webview_window(LABEL).ok_or("A conversa flutuante não está disponível.")?;
   *state.context.lock().unwrap()=ctx.clone();
   win.emit("pet-chat-opened",serde_json::json!({"context":ctx,"rect":rect})).map_err(|_|"Não foi possível abrir a conversa.")?;
   crate::platform::set_activating(&win,true);win.show().map_err(|_|"Não foi possível mostrar a conversa.")?;
   #[cfg(target_os="linux")]if crate::platform::pet_chat_normal_window(){crate::pet_control::collapse(&current);}
   let _=win.set_focus();Ok(true)
  };
  let result=if let Some(id)=request_id.as_deref(){
   let chat=current.state::<crate::chat::Chat>();
   match ctx.target.as_ref(){
    Some(target)=>crate::cli_chat::with_pending_help(&chat,id,target,present).and_then(|value|value.unwrap_or(Ok(false))),
    None=>Err("O pedido de ajuda não tem um destino válido.".into()),
   }
  }else{present()};
  let _=tx.send(result);
 }).map_err(|_|"Não foi possível abrir a conversa.")?;
 rx.await.map_err(|_|"A conversa não respondeu.".to_string())?
}

#[cfg(test)]mod tests {
 #[test]fn service_conversations_keep_their_own_identity_even_with_a_codex_backend(){
  assert_eq!(super::logical_provider("integration_spotify","codex"),"integration_spotify");
  assert_eq!(super::logical_provider("integration_github","codex"),"integration_github");
  assert_eq!(super::logical_provider("agent_codex","codex"),"codex");
  assert_eq!(super::logical_provider("ai_ollama","ollama"),"ollama");
 }
}

// Synthetic RAM-only Chat fixtures: no AppHandle, files, native CLI or models.
// The custom-pet rejection expects selected_conversation's canonical-pet guard.
#[cfg(test)]
mod target_tests {
    use super::{logical_provider, selected_conversation, Context, Request, Session};
    use crate::chat::Chat;
    use crate::session_chat::types::{Origin, Target};
    use serde_json::{json, Value};
    use std::sync::Arc;

    const NATIVE_B: &str = "019a734b-0000-7000-8000-000000000001";
    const NATIVE_A: &str = "019a734b-0000-7000-8000-000000000002";

    fn owners() -> Vec<String> {
        ["codex", "copilot", "integration_github", "ollama"]
            .into_iter().map(str::to_owned).collect()
    }

    fn target(pet: &str, provider: &str, origin: Origin, id: &str) -> Target {
        Target { pet_id: pet.into(), provider: provider.into(), origin,
            session_id: id.into(), conversation_id: None }
    }

    fn session(id: &str, backend: &str) -> Session {
        Session { id: id.into(), backend: backend.into(), title: "Synthetic session".into(),
            cwd: "/fixture/work".into(), ..Session::default() }
    }

    fn seed(chat: &Chat, provider: &str, native: &str, label: &str) -> String {
        let turn = chat.begin_cli(provider, Arc::default()).unwrap();
        let _lease = chat.lease(&turn);
        chat.remember_session(&turn, native, "/fixture/work", false);
        let question = format!("synthetic {label} question");
        let answer = format!("synthetic {label} answer");
        chat.commit_reply(&turn, json!({"role":"user","content":question}),
            json!({"role":"assistant","content":answer}), &question, &answer).unwrap();
        chat.conversation_id(provider).unwrap()
    }

    fn current(chat: &Chat, provider: &str) -> Value {
        serde_json::to_value(chat.conversation(provider).unwrap()).unwrap()
    }

    #[test]
    fn help_wire_keeps_manual_requests_compatible_and_requires_a_bounded_request_identity(){
        let old:Request=serde_json::from_value(json!({"pet":"agent_codex"})).unwrap();
        assert!(old.request_id.is_none());assert!(super::validate_help(None,None).is_ok());
        for origin in [Origin::Mochi,Origin::External]{
            let t=target("agent_codex","codex",origin,NATIVE_A);
            let new:Request=serde_json::from_value(json!({"pet":"agent_codex","target":t,"requestId":"cli-23"})).unwrap();
            assert_eq!(new.request_id.as_deref(),Some("cli-23"));
            assert!(super::validate_help(new.request_id.as_deref(),new.target.as_ref()).is_ok());
            assert!(super::validate_help(Some("cli-23"),None).is_err());
            for invalid in ["".to_string(),"x".repeat(129),"cli\n23".into(),"cli/23".into()]{
                assert!(super::validate_help(Some(&invalid),new.target.as_ref()).is_err());
            }
        }
    }

    #[test]
    fn archived_mochi_b_is_selected_without_restoring_or_displacing_busy_a() {
        let chat = Chat::default();
        let b = seed(&chat, "codex", NATIVE_B, "B");
        let history_b = chat.read_conversation("codex", &b).unwrap().messages;
        chat.reset_provider("codex").unwrap();
        let a = seed(&chat, "codex", NATIVE_A, "A");
        let reservation_a = chat.reserve_conversation("codex", Some(&a)).unwrap();
        let before = current(&chat, "codex");
        let mut t = target("agent_codex", "codex", Origin::Mochi, &b);
        t.conversation_id = Some(b.clone());
        assert_eq!(selected_conversation("agent_codex", "codex", None, Some(&t),
            &chat, &owners()).unwrap(), Some(b.clone()));
        assert_eq!(current(&chat, "codex"), before);
        assert_eq!(chat.conversation_id("codex").unwrap(), a);
        assert_eq!(chat.read_conversation("codex", &b).unwrap().messages, history_b);
        assert!(chat.conversation("codex").unwrap().busy);
        drop(reservation_a);
    }

    #[test]
    fn mochi_rejects_wrong_pet_provider_missing_id_and_conversation_id_mismatch() {
        let chat = Chat::default();
        let id = seed(&chat, "codex", NATIVE_B, "B");
        let before = current(&chat, "codex");
        let valid = target("agent_codex", "codex", Origin::Mochi, &id);
        let mut wrong_pet = valid.clone(); wrong_pet.pet_id = "agent_copilot".into();
        let mut wrong_provider = valid.clone(); wrong_provider.provider = "copilot".into();
        let missing = target("agent_codex", "codex", Origin::Mochi, "synthetic-missing");
        let empty = target("agent_codex", "codex", Origin::Mochi, "");
        let mut wrong_conversation = valid.clone();
        wrong_conversation.conversation_id = Some("another-conversation".into());
        for bad in [wrong_pet, wrong_provider, missing, empty, wrong_conversation] {
            assert!(selected_conversation("agent_codex", "codex", None, Some(&bad),
                &chat, &owners()).is_err(), "unexpected acceptance: {bad:?}");
        }
        assert_eq!(current(&chat, "codex"), before);
    }

    #[test]
    fn github_with_codex_backend_keeps_its_logical_mochi_owner() {
        let chat = Chat::default();
        let provider = logical_provider("integration_github", "codex");
        assert_eq!(provider, "integration_github");
        let id = seed(&chat, &provider, NATIVE_B, "GitHub");
        let good = target("integration_github", &provider, Origin::Mochi, &id);
        let legacy = session(NATIVE_B, "codex");
        assert_eq!(selected_conversation("integration_github", &provider, Some(&legacy),
            Some(&good), &chat, &owners()).unwrap(), Some(id.clone()));
        let wrong = target("integration_github", "codex", Origin::Mochi, &id);
        assert!(selected_conversation("integration_github", &provider, None,
            Some(&wrong), &chat, &owners()).is_err());
        assert!(selected_conversation("agent_codex", "codex", None,
            Some(&target("agent_codex", "codex", Origin::Mochi, &id)),
            &chat, &owners()).is_err());
    }

    #[test]
    fn external_canonical_codex_and_copilot_keep_exact_native_uuid() {
        let chat = Chat::default();
        for (pet, provider) in [("agent_codex", "codex"), ("agent_copilot", "copilot")] {
            let t = target(pet, provider, Origin::External, NATIVE_B);
            let before = serde_json::to_value(&t).unwrap();
            assert_eq!(selected_conversation(pet, provider, None, Some(&t),
                &chat, &owners()).unwrap(), None);
            assert_eq!(serde_json::to_value(&t).unwrap(), before);
            assert_eq!(t.session_id, NATIVE_B);
        }
    }

    #[test]
    fn external_rejects_mochi_id_bad_uuid_unsupported_provider_and_custom_pet() {
        let chat = Chat::default();
        let mut with_conversation = target("agent_codex", "codex", Origin::External, NATIVE_B);
        with_conversation.conversation_id = Some("synthetic-conversation".into());
        assert!(selected_conversation("agent_codex", "codex", None,
            Some(&with_conversation), &chat, &owners()).is_err());
        for id in ["", "not-a-uuid", "019a734b-0000-7000-8000-00000000000g",
            "019a734b000070008000000000000001", "019a734b-0000-7000-8000-000000000001/child"] {
            let t = target("agent_codex", "codex", Origin::External, id);
            assert!(selected_conversation("agent_codex", "codex", None, Some(&t),
                &chat, &owners()).is_err(), "unexpected UUID acceptance: {id}");
        }
        for (pet, provider) in [("agent_claude", "claude-cli"), ("ai_ollama", "ollama"),
            ("integration_github", "integration_github"),
            ("synthetic_custom_codex", "codex"), ("synthetic_custom_copilot", "copilot")] {
            let t = target(pet, provider, Origin::External, NATIVE_B);
            assert!(selected_conversation(pet, provider, None, Some(&t),
                &chat, &owners()).is_err(), "unexpected external owner acceptance: {pet}/{provider}");
        }
    }

    #[test]
    fn external_uuid_owned_by_any_mochi_owner_must_open_as_mochi_instead() {
        for owner in ["codex", "copilot", "integration_github"] {
            let chat = Chat::default();
            let owned = seed(&chat, owner, NATIVE_B, "owned B");
            chat.reset_provider(owner).unwrap();
            seed(&chat, owner, NATIVE_A, "current A");
            let before = current(&chat, owner);
            for (pet, provider) in [("agent_codex", "codex"), ("agent_copilot", "copilot")] {
                let external = target(pet, provider, Origin::External, NATIVE_B);
                assert!(selected_conversation(pet, provider, None, Some(&external),
                    &chat, &owners()).is_err(), "owned UUID escaped owner {owner}");
            }
            assert_eq!(chat.conversation_for_session(owner, NATIVE_B), Some(owned));
            assert_eq!(current(&chat, owner), before);
        }
    }

    #[test]
    fn legacy_session_must_agree_with_an_explicit_mochi_or_external_target() {
        let chat = Chat::default();
        let b = seed(&chat, "codex", NATIVE_B, "B");
        chat.reset_provider("codex").unwrap();
        seed(&chat, "codex", NATIVE_A, "A");
        let mochi = target("agent_codex", "codex", Origin::Mochi, &b);
        assert_eq!(selected_conversation("agent_codex", "codex",
            Some(&session(NATIVE_B, "codex")), Some(&mochi), &chat, &owners()).unwrap(), Some(b));
        for id in [NATIVE_A, "synthetic-unknown"] {
            assert!(selected_conversation("agent_codex", "codex", Some(&session(id, "codex")),
                Some(&mochi), &chat, &owners()).is_err());
        }
        let fresh = Chat::default();
        let external = target("agent_codex", "codex", Origin::External, NATIVE_B);
        assert_eq!(selected_conversation("agent_codex", "codex",
            Some(&session(NATIVE_B, "codex")), Some(&external), &fresh, &owners()).unwrap(), None);
        assert!(selected_conversation("agent_codex", "codex", Some(&session(NATIVE_A, "codex")),
            Some(&external), &fresh, &owners()).is_err());
    }

    #[test]
    fn no_target_retains_legacy_session_lookup_without_restoring_or_creating() {
        let chat = Chat::default();
        assert_eq!(selected_conversation("agent_codex", "codex", None, None,
            &chat, &owners()).unwrap(), None);
        assert_eq!(selected_conversation("agent_codex", "codex", Some(&session(NATIVE_B, "codex")),
            None, &chat, &owners()).unwrap(), None);
        assert!(chat.history("codex").is_empty());
        let b = seed(&chat, "codex", NATIVE_B, "B");
        chat.reset_provider("codex").unwrap();
        seed(&chat, "codex", NATIVE_A, "A");
        let before = current(&chat, "codex");
        assert_eq!(selected_conversation("agent_codex", "codex", Some(&session(NATIVE_B, "codex")),
            None, &chat, &owners()).unwrap(), Some(b));
        assert_eq!(selected_conversation("agent_codex", "codex", Some(&session("unknown", "codex")),
            None, &chat, &owners()).unwrap(), None);
        assert_eq!(current(&chat, "codex"), before);
    }

    #[test]
    fn request_accepts_legacy_and_target_wire_shapes_but_denies_unknown_fields() {
        let legacy: Request = serde_json::from_value(json!({"pet":"agent_codex"})).unwrap();
        assert_eq!(legacy.pet, "agent_codex");
        assert!(legacy.session.is_none()); assert!(legacy.target.is_none()); assert!(!legacy.show_sessions);
        let t = target("agent_copilot", "copilot", Origin::External, NATIVE_B);
        let wire = json!({"pet":"agent_copilot","showSessions":true,
            "session":{"id":NATIVE_B,"backend":"copilot","title":"Fixture","cwd":"/fixture/work"},
            "target":t});
        let new: Request = serde_json::from_value(wire.clone()).unwrap();
        assert!(new.show_sessions); assert_eq!(new.target.as_ref(), Some(&t));
        assert_eq!(new.session.as_ref().unwrap().id, NATIVE_B);
        let mut unknown = wire.clone(); unknown["decisionId"] = json!("not-in-contract");
        assert!(serde_json::from_value::<Request>(unknown).is_err());
        let mut unknown = wire.clone(); unknown["target"]["turnId"] = json!("not-in-contract");
        assert!(serde_json::from_value::<Request>(unknown).is_err());
        let mut unknown = wire; unknown["session"]["privatePayload"] = json!("not-in-contract");
        assert!(serde_json::from_value::<Request>(unknown).is_err());
    }

    #[test]
    fn context_serializes_borrowed_target_identity_verbatim_in_camel_case() {
        for origin in [Origin::Mochi, Origin::External] {
            let mut t = target("agent_codex", "codex", origin.clone(), NATIVE_B);
            if origin == Origin::Mochi { t.conversation_id = Some(NATIVE_B.into()); }
            let expected = serde_json::to_value(&t).unwrap();
            let context = Context { pet: "agent_codex".into(), name: "Synthetic pet".into(),
                provider: "codex".into(), session: None,
                conversation_id: t.conversation_id.clone(), show_sessions: true, target: Some(t.clone()) };
            let wire = serde_json::to_value(context).unwrap();
            assert_eq!(wire["target"], expected);
            assert_eq!(wire["target"]["sessionId"], NATIVE_B);
            assert_eq!(wire["showSessions"], true);
            assert!(wire.get("conversationId").is_some());
            assert!(wire.get("conversation_id").is_none());
            assert_eq!(serde_json::from_value::<Target>(wire["target"].clone()).unwrap(), t);
        }
        let old = serde_json::to_value(Context::default()).unwrap();
        assert!(old["target"].is_null());
    }
}
