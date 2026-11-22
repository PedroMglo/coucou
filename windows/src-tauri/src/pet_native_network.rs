//! Native/Core resolve -> SAME immutable Prepared -> synchronous actor enqueue.
//! No renderer identity, direct daemon method, model call or default target.
use crate::{pet_native_binding::{self as binding,ipc::*},lifecycle,executor};
use tauri::{AppHandle,WebviewWindow};
async fn prepared(app:&AppHandle,window:&WebviewWindow,expected:Revision)->Result<(executor::Client,executor::Prepared),Failure>{
    let client=binding::manager(app)?.client()?;
    let context=binding::prepare(window,expected,client.clone()).await?;
    Ok((client,context.into_prepared()))
}
#[tauri::command]
pub(crate) async fn pet_system_network_read(app:AppHandle,window:WebviewWindow,request:Checked<ReadRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_read(p,request.0.force).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_view(app:AppHandle,window:WebviewWindow,request:Checked<ContextRequest>)->Result<lifecycle::View,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_view(p).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_scan(app:AppHandle,window:WebviewWindow,request:Checked<ScanRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_scan(p,request.0.scope.as_str(),request.0.intent_id.as_str()).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_enable_wifi(app:AppHandle,window:WebviewWindow,request:Checked<EnableWifiRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_enable_wifi(p,request.0.scope.as_str(),request.0.intent_id.as_str()).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_disable_wifi(app:AppHandle,window:WebviewWindow,request:Checked<DisableWifiRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_disable_wifi(p,request.0.scope.as_str(),request.0.intent_id.as_str()).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_ticket(app:AppHandle,window:WebviewWindow,request:Checked<TicketRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_ticket(p,request.0.ticket_id.as_str()).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
#[tauri::command]
pub(crate) async fn pet_system_network_cancel(app:AppHandle,window:WebviewWindow,request:Checked<TicketRequest>)->Result<lifecycle::Ticket,Failure>{
    let(client,p)=prepared(&app,&window,request.0.expected_revision).await?;
    client.enqueue_prepared_cancel(p,request.0.ticket_id.as_str()).map_err(Failure::network)?.wait().await.map_err(Failure::network)
}
