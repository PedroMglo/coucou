//! Explicit read-only local model inspection. Passive listing never performs
//! this IO, and metadata cannot grant execution or renew a goal's budget.
use super::model;
use tauri::AppHandle;
#[cfg(target_os="linux")]use tauri::Manager;
#[cfg(target_os="linux")]use super::store::Store;
#[cfg(target_os="linux")]
async fn inspect(server:&crate::local_chat::Server,model:&str,current:impl Fn()->bool)->model::Result<()> {
    if server.id!="ollama"||model.is_empty()||model.starts_with('@')||model.ends_with(":cloud")||model.ends_with("-cloud"){return Err(model::Error::InvalidInput);}
    let base=crate::local_chat::base_url(server).map_err(|_|model::Error::InvalidInput)?;
    if !crate::net::is_loopback_url(&base){return Err(model::Error::InvalidInput);}
    crate::net::while_current(current,||async{crate::local_tasks::protocol::capabilities(&base,model).await;Ok(())}).await.map_err(|_|model::Error::Conflict)
}
#[tauri::command]
pub async fn pet_runtime_refresh_readiness(app:AppHandle,window:tauri::WebviewWindow,pet_id:String,goal_id:String)->model::Result<crate::pet_capabilities::Descriptor>{
    if !super::adapter::allowed(window.label()){return Err(model::Error::InvalidInput);}
    #[cfg(not(target_os="linux"))]{let _=(app,pet_id,goal_id);Err(model::Error::UnsupportedPlatform)}
    #[cfg(target_os="linux")]{
        let root=crate::settings::config_dir().join("pet-runtime-v1");let store=Store::existing_native(&root)?.ok_or(model::Error::NotFound)?;let goal=store.read(&goal_id,&pet_id)?;
        if !matches!(goal.state,model::State::Draft|model::State::Queued){return Err(model::Error::Conflict);}
        let projection=crate::pet_capabilities::native_projection(&app,None).map_err(|_|model::Error::StorageUnavailable)?;
        let descriptor=projection.snapshot.pets.iter().find(|p|p.pet_id==pet_id).ok_or(model::Error::NotFound)?;
        let owner=model::Owner::from_descriptor(descriptor)?;if !owner.same_runtime_binding(&goal.spec.owner)||owner.provider!="ollama"||owner.adapter!="ollama"{return Err(model::Error::Conflict);}
        let model=descriptor.binding.model.as_ref().ok_or(model::Error::Conflict)?;
        let server=crate::local_chat::server(&projection.settings,&owner.provider).ok_or(model::Error::Conflict)?;
        let stop=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));let until=std::time::Instant::now()+std::time::Duration::from_secs(6);
        let chat=app.state::<crate::chat::Chat>();let hold=chat.runtime_resources().acquire(&owner.provider,stop,Some(until)).map_err(|_|model::Error::Busy)?;
        let revision=goal.revision.clone();
        inspect(&server,model,||hold.current()&&store.read(&goal_id,&pet_id).is_ok_and(|g|g.revision==revision&&g.spec.owner==goal.spec.owner&&matches!(g.state,model::State::Draft|model::State::Queued))).await?;
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
