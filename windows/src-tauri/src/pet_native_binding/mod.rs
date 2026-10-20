//! Native object identity + backend accepted intent; not DOM/gesture attestation.
pub(crate) mod core;
pub(crate) mod ipc;
mod native_policy;
#[cfg(target_os="linux")]
mod linux;
use std::sync::Arc;
use tauri::{AppHandle,Emitter,Manager as _,WebviewWindow};
use ipc::*;
pub(crate) use core::{Hint,Plan,PreparedContext};

#[derive(Clone)]
pub(crate) struct Policy {dev:Option<tauri::Url>}
impl Policy {
    pub(crate) fn for_app(app:&AppHandle)->Self {
        #[cfg(dev)]let dev=app.config().build.dev_url.clone();
        #[cfg(not(dev))]let dev=None;
        let _=app;Self {dev}
    }
    pub(crate) fn allowed(&self,url:&tauri::Url)->bool {
        if !url.username().is_empty()||url.password().is_some()||url.query().is_some()||url.path()!="/pet-control.html"{return false;}
        let bundled=url.scheme()=="tauri"&&url.host_str()==Some("localhost")&&url.port().is_none();
        bundled||self.dev.as_ref().is_some_and(|base|base.origin()==url.origin())
    }
}
pub(crate) fn manager(app:&AppHandle)->Result<Arc<core::Manager>,Failure>{
    app.try_state::<Arc<core::Manager>>().map(|s|s.inner().clone()).ok_or_else(Failure::unavailable)
}
pub(crate) fn setup(app:&AppHandle,enabled:bool){
    if app.try_state::<Arc<core::Manager>>().is_some(){return;}
    let manager=core::Manager::new(enabled);app.manage(manager.clone());
    // Tauri's GTK setup thread need not be entered in Tokio. Create the actor
    // inside its runtime; setup performs no native DMS request or device action.
    #[cfg(target_os="linux")]
    tauri::async_runtime::spawn(async move {
        use ring::rand::SecureRandom;
        let Ok(native)=crate::transport::Config::from_environment() else{return;};
        let mut random=[0u8;24];if ring::rand::SystemRandom::new().fill(&mut random).is_err(){return;}
        let boot_nonce=u128::from_be_bytes(random[..16].try_into().unwrap());
        let producer_nonce=u64::from_be_bytes(random[16..].try_into().unwrap());
        if boot_nonce==0||producer_nonce==0{return;}
        let config=crate::executor::Config {lifecycle:crate::lifecycle::Config::default(),boot_nonce,producer_nonce,command_capacity:64};
        if let Ok(owner)=crate::executor::Executor::start(native,config,&manager.binding){let _=manager.install_network(owner);}
    });
}
pub(crate) fn emit_context(app:&AppHandle,proposed:&Context){
    // Emit only current accepted state, never a queued old native callback.
    let current=manager(app).and_then(|m|m.current()).and_then(|c|c.context()).unwrap_or_else(|_|proposed.clone());
    let _=app.emit_to(crate::pet_control::LABEL,"pet-control-navigation-context",current);
}
pub(crate) fn settings_committed(app:&AppHandle,enabled:bool){
    if let Ok(manager)=manager(app){
        if let Ok(Some(context))=manager.settings_committed(enabled){
            // No GTK/native calls or emits in this transaction hook. The caller
            // emits settings/context only AFTER this synchronous state update.
            let _=context;
        }
    }
}
pub(crate) fn publish_current(app:&AppHandle){
    if let Ok(c)=manager(app).and_then(|m|m.current()).and_then(|c|c.context()){emit_context(app,&c);}
    #[cfg(target_os="linux")]linux::request_reconcile(app);
}
pub(crate) fn attach(app:&AppHandle,win:&WebviewWindow)->Result<(),Failure>{
    #[cfg(target_os="linux")]{linux::attach(app,win)}
    #[cfg(not(target_os="linux"))]{let _=(app,win);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
pub(crate) fn page_load(win:&WebviewWindow,started:bool){#[cfg(target_os="linux")]linux::page_load(win,started);#[cfg(not(target_os="linux"))]let _=(win,started);}
pub(crate) fn navigation_allowed(app:&AppHandle,url:&tauri::Url)->bool{Policy::for_app(app).allowed(url)}
pub(crate) async fn prepare(win:&WebviewWindow,expected:Revision,client:crate::executor::Client)->Result<PreparedContext,Failure>{
    #[cfg(target_os="linux")]{linux::prepare(win,expected,client).await}
    #[cfg(not(target_os="linux"))]{let _=(win,expected,client);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
pub(crate) async fn hint(win:&WebviewWindow,hint:Hint)->Result<(),Failure>{
    #[cfg(target_os="linux")]{linux::hint(win,hint).await}
    #[cfg(not(target_os="linux"))]{let _=(win,hint);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
pub(crate) fn begin_trusted(app:&AppHandle,target:Mode)->Result<Plan,Failure>{
    let core=manager(app)?.current()?;let(plan,context)=core.begin_trusted(target)?;emit_context(app,&context);Ok(plan)
}
#[derive(Clone,Copy)]
pub(crate) struct Placement {pub(crate) x:f64,pub(crate) y:f64,pub(crate) screen_width:f64,pub(crate) width:f64,pub(crate) height:f64,pub(crate) top:f64}
pub(crate) async fn apply(win:&WebviewWindow,plan:Plan,placement:Option<Placement>)->Result<NavigationReceipt,Failure>{
    #[cfg(target_os="linux")]{linux::apply(win,plan,placement).await}
    #[cfg(not(target_os="linux"))]{let _=(win,plan,placement);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
pub(crate) async fn place_current(win:&WebviewWindow,placement:Placement)->Result<(),Failure>{
    #[cfg(target_os="linux")]{linux::place_current(win,placement).await}
    #[cfg(not(target_os="linux"))]{let _=(win,placement);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
pub(crate) fn collapse(app:&AppHandle){
    let Ok(plan)=begin_trusted(app,Mode::Compact) else{return;};
    let Some(win)=app.get_webview_window(crate::pet_control::LABEL) else{return;};
    let app=app.clone();tauri::async_runtime::spawn(async move {
        if let Ok(receipt)=apply(&win,plan,None).await {
            if is_current(&app,receipt.context.revision){let payload=CollapseEvent {navigation_revision:receipt.context.revision,context:receipt.context};let _=app.emit_to(crate::pet_control::LABEL,"pet-control-collapse",payload);}
        }
    });
}
pub(crate) fn is_current(app:&AppHandle,revision:Revision)->bool{manager(app).and_then(|m|m.current()).and_then(|c|c.context()).is_ok_and(|c|c.revision==revision)}
pub(crate) fn shutdown(app:&AppHandle){if let Ok(m)=manager(app){m.shutdown();}}

#[tauri::command]
pub(crate) async fn pet_control_navigation_context(window:WebviewWindow)->Result<Context,Failure>{
    #[cfg(target_os="linux")]{linux::context(&window).await}
    #[cfg(not(target_os="linux"))]{let _=window;Err(Failure::binding(ErrorCode::Unsupported,None))}
}
#[tauri::command]
pub(crate) async fn pet_control_navigate(window:WebviewWindow,request:Checked<NavigationRequest>)->Result<NavigationReceipt,Failure>{
    #[cfg(target_os="linux")]{linux::navigate(&window,request.0).await}
    #[cfg(not(target_os="linux"))]{let _=(window,request);Err(Failure::binding(ErrorCode::Unsupported,None))}
}
