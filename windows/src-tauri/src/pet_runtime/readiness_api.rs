//! Explicit read-only local model inspection. Passive listing never performs
//! this IO, and metadata cannot grant execution or renew a goal's budget.
use super::model;
use tauri::AppHandle;
#[cfg(target_os="linux")]use tauri::Manager;
#[cfg(target_os="linux")]use super::store::Store;
#[cfg(target_os="linux")]
async fn inspect(server:&crate::local_chat::Server,model:&str,current:impl Fn()->bool)->model::Result<()> {
    if server.id=="lmstudio"{return crate::local_chat::lmstudio::inspect(server,model,current).await.map(|_|()).map_err(|_|model::Error::Conflict);}
    if server.id!="ollama"||model.is_empty()||model.starts_with('@')||model.ends_with(":cloud")||model.ends_with("-cloud"){return Err(model::Error::InvalidInput);}
    let base=crate::local_chat::base_url(server).map_err(|_|model::Error::InvalidInput)?;
    if !crate::net::is_loopback_url(&base){return Err(model::Error::InvalidInput);}
    crate::net::while_current(current,||async{crate::local_tasks::protocol::capabilities(&base,model).await;Ok(())}).await.map_err(|_|model::Error::Conflict)
}
#[cfg(target_os="linux")]
pub(in crate::pet_runtime)async fn inspect_captured(server:&crate::local_chat::Server,model:&str,captured:Option<&crate::pet_inference::Captured>,current:impl Fn()->model::Result<()>)->model::Result<()>{
 use std::sync::atomic::{AtomicBool,Ordering};
 let busy=AtomicBool::new(false);
 let check=||{
  busy.store(false,Ordering::Release);
  let observed=current().and_then(|()|match captured.map(crate::pet_inference::Captured::observation){
   Some(crate::pet_inference::CaptureObservation::Busy)=>Err(model::Error::Busy),
   Some(crate::pet_inference::CaptureObservation::Changed)=>Err(model::Error::Conflict),_=>Ok(()),
  });
  if observed==Err(model::Error::Busy){busy.store(true,Ordering::Release);}observed.is_ok()
 };
 if !check(){return Err(if busy.load(Ordering::Acquire){model::Error::Busy}else{model::Error::Conflict});}
 let result=if let Some(captured)=captured.filter(|_|server.id=="lmstudio"){
  crate::local_chat::lmstudio::inspect_scoped(server,model,crate::local_chat::metadata_auth_for_capture(captured).map_err(|_|model::Error::Conflict)?,check).await.map(|_|()).map_err(|_|model::Error::Conflict)
 }else{inspect(server,model,check).await};
 if result.is_err()&&busy.load(Ordering::Acquire){Err(model::Error::Busy)}else{result}
}
#[cfg(target_os="linux")]pub(super)fn requested_inference(spec:&super::admission::AdmissionSpec)->bool{
 spec.owner.local_engine()&&spec.binding_format()&&spec.execution_requested()&&!spec.deterministic()
}
/// Active inspection belongs to the explicit signed requested goal, never a
/// passive status query. The acquired native hold and exact destination survive
/// the wait; no current UI selection may replace them.
#[cfg(target_os="linux")]
pub(super)async fn inspect_requested(app:&AppHandle,goal:&model::Record,hold:std::sync::Arc<crate::pet_resources::Hold>)->model::Result<()>{
    let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
    if row.spec.rule_occurrence.is_some(){let authority=super::authority::Authority::existing(&super::admission_review::authority_root()).map_err(super::authority::Failure::control_error)?;let reference=row.spec.rule_occurrence.as_ref().ok_or(model::Error::Conflict)?;let grant=authority.rule_grant(&reference.rule_decision_id).map_err(super::authority::Failure::control_error)?;if grant.policy.dispatch.is_some(){return super::rules::scheduled::inspect_admitted(app,goal,hold).await;}}
    if !requested_inference(&row.spec){return Err(model::Error::Conflict);}
    let store=Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(model::Error::NotFound)?;
    let authority=super::authority::Authority::existing(&super::admission_review::authority_root()).map_err(super::authority::Failure::control_error)?;
    authority.verify_queued(goal,crate::assistant_memory::now()).map_err(super::authority::Failure::control_error)?;
    let projection=crate::pet_capabilities::native_projection(app,Some(&hold)).map_err(|_|model::Error::StorageUnavailable)?;
    let config=projection.configuration.as_ref().ok_or(model::Error::StorageUnavailable)?;
    let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==row.spec.owner.pet_id).ok_or(model::Error::NotFound)?;
    if !projection.preferences_ready||descriptor.binding.model.as_deref()!=Some(row.spec.resolved_model.as_str())
        ||super::native_validity::policy_revision(config,&projection.settings,goal)?!=row.spec.policy_revision{return Err(model::Error::Conflict);}
    let captured=hold.captured();
    let server=if let Some(c)=&captured{crate::local_chat::server_for_capture(c).map_err(|_|model::Error::Conflict)?}else{crate::local_chat::server(&projection.settings,&row.spec.owner.provider).ok_or(model::Error::Conflict)?};
    if server.id!=row.spec.owner.adapter{return Err(model::Error::Conflict);}
    let until=std::time::Instant::now()+std::time::Duration::from_secs(6);
    inspect_captured(&server,&row.spec.resolved_model,captured.as_deref(),||{
        if !hold.current()||std::time::Instant::now()>=until{return Err(model::Error::Conflict);}
        let current=store.read(&goal.id,&goal.spec.owner.pet_id)?;
        if current.revision!=goal.revision{return Err(model::Error::Conflict);}
        authority.verify_queued(&current,crate::assistant_memory::now()).map_err(super::authority::Failure::control_error)?;
        let capture=crate::pet_capabilities::native_projection(app,Some(&hold)).map_err(|_|model::Error::StorageUnavailable)?;
        if !capture.configuration.as_ref().is_some_and(|config|capture.preferences_ready&&super::native_validity::policy_revision(config,&capture.settings,&current).ok().as_ref()==Some(&row.spec.policy_revision)){return Err(model::Error::Conflict);}Ok(())
    }).await
}
// This permits only explicitly requested metadata IO. Admission/F2/source
// validity and a new exact budget still guard every inference effect.
#[cfg(target_os="linux")]
fn refreshable_goal(g:&model::Record)->bool{matches!(g.state,model::State::Draft|model::State::Queued|model::State::RecoveryRequired)||g.state==model::State::Paused&&g.admissions.last().is_some_and(|r|r.spec.funded_response())}
#[tauri::command]
pub async fn pet_runtime_refresh_readiness(app:AppHandle,window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<crate::pet_capabilities::Descriptor>{
    if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,pet_id,goal_id);Err(model::Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{
        let root=crate::settings::config_dir().join("pet-runtime-v1");let store=Store::existing_native(&root)?.ok_or(model::Error::NotFound)?;let goal=store.read(&goal_id,&pet_id)?;
        if !refreshable_goal(&goal){return Err(model::Error::Conflict);}
        let projection=crate::pet_capabilities::native_projection(&app,None).map_err(|_|model::Error::StorageUnavailable)?;
        let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==pet_id).ok_or(model::Error::NotFound)?;
        let owner=model::Owner::from_descriptor(descriptor)?;if !owner.same_runtime_binding(&goal.spec.owner)||!owner.local_engine(){return Err(model::Error::Conflict);}
        let model=descriptor.binding.model.as_ref().ok_or(model::Error::Conflict)?;
        let stop=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));let until=std::time::Instant::now()+std::time::Duration::from_secs(6);
        let chat=app.state::<crate::chat::Chat>();let hold=chat.runtime_resources().acquire(&owner.provider,stop,Some(until)).map_err(|_|model::Error::Busy)?;
        let captured=hold.captured();let server=if let Some(c)=&captured{crate::local_chat::server_for_capture(c).map_err(|_|model::Error::Conflict)?}else{crate::local_chat::server(&projection.settings,&owner.provider).ok_or(model::Error::Conflict)?};
        if server.id!=owner.adapter{return Err(model::Error::Conflict);}
        let revision=goal.revision.clone();
        inspect_captured(&server,model,captured.as_deref(),||{if !hold.current(){return Err(model::Error::Conflict);}let g=store.read(&goal_id,&pet_id)?;if g.revision!=revision||g.spec.owner!=goal.spec.owner||!refreshable_goal(&g){return Err(model::Error::Conflict);}Ok(())}).await?;
        drop(hold);let current=store.read(&goal_id,&pet_id)?;if current.revision!=revision{return Err(model::Error::Conflict);}
        let fresh=crate::pet_capabilities::provider_descriptor(&app,&owner.provider).map_err(|_|model::Error::StorageUnavailable)?;
        if model::Owner::from_descriptor(&fresh)?!=owner||fresh.binding.model.as_ref()!=Some(model){return Err(model::Error::Conflict);}Ok(fresh)
    }
}


#[cfg(all(test,target_os="linux"))]mod tests{
    use super::*;use std::{net::TcpListener,io::{Read,Write},sync::{Arc,atomic::{AtomicBool,Ordering}}};
    fn server(url:String)->crate::local_chat::Server{crate::local_chat::Server{id:"ollama",name:"Private metadata fixture",url,key:None}}
    fn rt()->tokio::runtime::Runtime{tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()}
    #[test]fn resolving_a_model_hint_keeps_the_goal_owner_but_never_changes_its_adapter(){
        let original=model::Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None};let mut resolved=original.clone();resolved.model_hint=Some("private-local-model".into());assert!(resolved.same_runtime_binding(&original));
        for field in 0..3{let mut wrong=resolved.clone();match field{0=>wrong.pet_id="agent_codex".into(),1=>wrong.provider="codex".into(),_=>wrong.adapter="codex".into()};assert!(!wrong.same_runtime_binding(&original));}
    }
    #[test]fn explicit_readiness_only_posts_the_exact_model_to_local_metadata_and_never_infers(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();let worker=std::thread::spawn(move||{
            let(mut client,_)=listener.accept().unwrap();client.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();let mut all=Vec::new();let mut chunk=[0u8;1024];let body;
            loop{let n=client.read(&mut chunk).unwrap();assert!(n>0);all.extend_from_slice(&chunk[..n]);if let Some(split)=all.windows(4).position(|w|w==b"\r\n\r\n"){let head=String::from_utf8_lossy(&all[..split]);assert!(head.starts_with("POST /api/show HTTP/1.1"));let len=head.lines().find_map(|l|l.to_lowercase().strip_prefix("content-length:").and_then(|n|n.trim().parse::<usize>().ok())).unwrap();if all.len()>=split+4+len{body=all[split+4..split+4+len].to_vec();break;}}}
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap(),serde_json::json!({"model":"private-metadata-model"}));
            let data=r#"{"capabilities":["completion"],"model_info":{"fixture":true}}"#;write!(client,"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",data.len(),data).unwrap();
        });rt().block_on(inspect(&server(format!("http://{address}")),"private-metadata-model",||true)).unwrap();worker.join().unwrap();
    }
    #[test]fn readiness_never_contacts_cloud_nonlocal_unresolved_or_cancelled_targets(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let base=format!("http://{}",listener.local_addr().unwrap());let runtime=rt();
        for model in ["","@auto","model:cloud","model-cloud"]{assert!(runtime.block_on(inspect(&server(base.clone()),model,||true)).is_err());}
        assert!(runtime.block_on(inspect(&server("https://example.invalid".into()),"private",||true)).is_err());assert!(runtime.block_on(inspect(&server(base),"private",||false)).is_err());assert!(matches!(listener.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));
    }
    #[test]fn changing_goal_lifetime_cancels_a_stalled_metadata_read_without_an_inference_fallback(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let base=format!("http://{}",listener.local_addr().unwrap());let live=Arc::new(AtomicBool::new(true));let flag=live.clone();
        let worker=std::thread::spawn(move||{let(mut client,_)=listener.accept().unwrap();client.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();let mut bytes=[0u8;1024];let n=client.read(&mut bytes).unwrap();assert!(String::from_utf8_lossy(&bytes[..n]).starts_with("POST /api/show"));flag.store(false,Ordering::Release);std::thread::sleep(std::time::Duration::from_millis(100));});
        let start=std::time::Instant::now();assert!(rt().block_on(inspect(&server(base),"private-cancelled-metadata",||live.load(Ordering::Acquire))).is_err());assert!(start.elapsed()<std::time::Duration::from_secs(2));worker.join().unwrap();
    }
}
