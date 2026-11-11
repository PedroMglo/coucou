//! Presentation routing for independent terminals. Never an execution grant.
//! The existing Pending mutex serializes registration and route withdrawal.
use super::*;
use serde::{Deserialize,Serialize};
use std::sync::{Arc,atomic::AtomicBool};
#[derive(Clone,Copy,Debug,Eq,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Mode{Terminal,Coucou}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Status{mode:Mode,revision:String,scope:&'static str,development_build:bool,restart_mode:Mode}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Change{mode:Mode,revision:String}
fn default_mode(development:bool)->Mode{if development{Mode::Terminal}else{Mode::Coucou}}
fn development()->bool{cfg!(debug_assertions)||cfg!(dev)}
pub(super)struct Routing{mode:Mode,revision:u64,active:Arc<AtomicBool>}
impl Default for Routing{fn default()->Self{Self{mode:default_mode(development()),revision:1,active:Arc::new(AtomicBool::new(default_mode(development())==Mode::Coucou))}}}
impl Routing{fn status(&self)->Status{Status{mode:self.mode,revision:self.revision.to_string(),scope:"processOnly",development_build:development(),restart_mode:default_mode(development())}}}
pub(super)enum Registration{Terminal,Queued(Option<Arc<AtomicBool>>),Refused}
pub(super)fn register(state:&mut RelayState,id:String,mut request:PendingRequest,external:bool)->Registration{
 if external&&state.routing.mode==Mode::Terminal{return Registration::Terminal;}
 if state.requests.len()>=128||state.requests.contains_key(&id){return Registration::Refused;}
 let route=external.then(||state.routing.active.clone());request.external_route=route.clone();state.requests.insert(id,request);Registration::Queued(route)
}
pub(super)async fn until_current<F:std::future::Future<Output=DecisionOutcome>>(route:Option<&Arc<AtomicBool>>,future:F)->DecisionOutcome{
 let Some(route)=route else{return future.await;};
 let invalid=async{while route.load(Ordering::Acquire){tokio::time::sleep(Duration::from_millis(25)).await;}};
 tokio::select!{biased;_=invalid=>DecisionOutcome::Handoff("external-terminal-mode"),reply=future=>reply}
}
fn change(state:&mut RelayState,request:Change,current:impl FnOnce()->bool)->Result<(Status,Vec<(String,String)>),String>{
 if request.revision!=state.routing.revision.to_string()||!current(){return Err("O modo mudou ou esta janela já não está disponível. Consulta o estado antes de decidir novamente.".into());}
 let mut withdrawn=Vec::new();
 if request.mode!=state.routing.mode{
  let next=state.routing.revision.checked_add(1).ok_or("A revisão do encaminhamento esgotou-se.")?;
  // A dispatched UI response may already have left the Pending map while
  // its relay future has not sent the wire reply. Its original route epoch
  // must be retired too. A new mode never revives that captured epoch.
  state.routing.active.store(false,Ordering::Release);
  state.routing.active=Arc::new(AtomicBool::new(request.mode==Mode::Coucou));
  state.routing.mode=request.mode;state.routing.revision=next;
  if request.mode==Mode::Terminal{for(id,row)in &state.requests{if let Some(route)=&row.external_route{
   route.store(false,Ordering::Release);let _=row.sender.try_send(Reply::Decline);withdrawn.push((id.clone(),row.session_id.clone()));
  }}}
 }
 Ok((state.routing.status(),withdrawn))
}
fn authorize(window:&tauri::WebviewWindow)->Result<(),String>{
 if window.label()!=crate::HUB_LABEL{return Err("Altera este modo nas Definições do Coucou.".into());}Ok(())
}
#[tauri::command]
pub(crate)fn relay_external_approval_status(app:AppHandle,window:tauri::WebviewWindow)->Result<Status,String>{
 authorize(&window)?;app.state::<Pending>().0.try_lock().map(|state|state.routing.status()).map_err(|_|"O encaminhamento está ocupado. Volta a consultar.".into())
}
#[tauri::command]
pub(crate)async fn relay_external_approval_set(app:AppHandle,window:tauri::WebviewWindow,request:Change)->Result<Status,String>{
 authorize(&window)?;
 #[cfg(target_os="linux")]let frame=crate::settings::review::capture_window(&window).await.map_err(|_|"Abre as Definições nativas para alterar o encaminhamento.".to_string())?;
 let(status,withdrawn)={
  let pending=app.state::<Pending>();let mut state=pending.0.try_lock().map_err(|_|"O encaminhamento está ocupado. Volta a consultar.".to_string())?;
  change(&mut state,request,||{
   #[cfg(target_os="linux")]{frame.current()}
   #[cfg(not(target_os="linux"))]{window.is_visible().unwrap_or(false)}
  })?
 };
 for(id,session)in withdrawn{
  crate::recap::forget_request(&app,&id);
  let _=app.emit_to(WINDOW_LABEL,"approval-handoff",handoff_payload(&id,&session,"",None,"external-terminal-mode"));
 }
 Ok(status)
}
#[cfg(test)]mod tests{
 use super::*;
 fn state(mode:Mode)->RelayState{RelayState{requests:HashMap::new(),routing:Routing{mode,revision:1,active:Arc::new(AtomicBool::new(mode==Mode::Coucou))}}}
 fn add(state:&mut RelayState,id:&str,external:bool)->(Registration,mpsc::Receiver<Reply>){let(tx,rx)=mpsc::channel(4);(register(state,id.into(),PendingRequest::new(tx,format!("session-{id}"),"agent_codex".into(),Instant::now()),external),rx)}
 fn route(reg:Registration)->Option<Arc<AtomicBool>>{match reg{Registration::Queued(route)=>route,_=>panic!("not queued")}}
 fn set(state:&mut RelayState,mode:Mode)->(Status,Vec<(String,String)>){{let revision=state.routing.revision.to_string();change(state,Change{mode,revision},||true).unwrap()}}
 #[test]fn only_development_changes_the_restart_default(){assert_eq!(default_mode(true),Mode::Terminal);assert_eq!(default_mode(false),Mode::Coucou);}
 #[test]fn terminal_mode_never_reserves_external_requests_but_owned_still_waits(){
  let mut s=state(Mode::Terminal);let(r,mut rx)=add(&mut s,"external",true);assert!(matches!(r,Registration::Terminal));assert!(rx.try_recv().is_err());assert!(s.requests.is_empty());
  assert!(route(add(&mut s,"owned",false).0).is_none());assert_eq!(s.requests.len(),1);
 }
 #[test]fn withdrawal_is_scoped_and_late_ack_decision_and_reenable_cannot_revive_it(){
  let mut s=state(Mode::Coucou);let(a,mut arx)=add(&mut s,"a",true);let a=route(a).unwrap();let(b,_brx)=add(&mut s,"b",true);let b=route(b).unwrap();let(_,mut owned)=add(&mut s,"owned",false);
  assert!(present_at(&mut s.requests,"a","session-a","agent_codex",true,Instant::now()));assert!(matches!(arx.try_recv(),Ok(Reply::Ack)));
  let(status,withdrawn)=set(&mut s,Mode::Terminal);assert_eq!(status.revision,"2");assert_eq!(withdrawn.len(),2);assert!(!a.load(Ordering::Acquire)&&!b.load(Ordering::Acquire));assert!(owned.try_recv().is_err());
  assert!(!present_at(&mut s.requests,"b","session-b","agent_codex",true,Instant::now()));assert!(!card_dispatch_at(&mut s.requests,"a",Reply::Decision("allow".into()),true,Instant::now()));
  set(&mut s,Mode::Coucou);assert!(!dispatch(&mut s.requests,"a",Reply::Decision("allow".into())));let(fresh,_rx)=add(&mut s,"fresh",true);assert!(route(fresh).unwrap().load(Ordering::Acquire));assert!(s.requests["owned"].live(Instant::now()));
 }
 #[test]fn stale_revision_and_lost_native_frame_never_change_routing(){
  let mut s=state(Mode::Coucou);let(_reg,_rx)=add(&mut s,"a",true);
  for(revision,current)in[("01",true),("0",true),("1",false)]{assert!(change(&mut s,Change{mode:Mode::Terminal,revision:revision.into()},||current).is_err());assert_eq!(s.routing.mode,Mode::Coucou);assert!(s.requests["a"].live(Instant::now()));}
  set(&mut s,Mode::Terminal);assert!(change(&mut s,Change{mode:Mode::Coucou,revision:"1".into()},||true).is_err());
 }
 #[test]fn routing_change_is_not_an_execution_policy_payload(){
  for raw in [r#"{"mode":"allow","revision":"1"}"#,r#"{"mode":"terminal","revision":1}"#,r#"{"mode":"terminal","revision":"1","petId":"forged"}"#]{assert!(serde_json::from_str::<Change>(raw).is_err());}
 }
 #[tokio::test]async fn withdrawal_also_fences_a_decision_already_removed_from_the_pending_map(){
  let mut s=state(Mode::Coucou);let(reg,mut rx)=add(&mut s,"already-dispatched",true);let captured=route(reg).unwrap();
  assert!(present_at(&mut s.requests,"already-dispatched","session-already-dispatched","agent_codex",true,Instant::now()));
  assert!(card_dispatch_at(&mut s.requests,"already-dispatched",Reply::Decision("allow".into()),true,Instant::now()));assert!(s.requests.is_empty());
  let(_,withdrawn)=set(&mut s,Mode::Terminal);assert!(withdrawn.is_empty());set(&mut s,Mode::Coucou);assert!(!captured.load(Ordering::Acquire));
  let outcome=until_current(Some(&captured),wait_for_decision("already-dispatched",&mut rx,Instant::now())).await;
  assert!(matches!(outcome,DecisionOutcome::Handoff("external-terminal-mode")));
 }
 #[tokio::test]async fn channel_saturation_and_queued_allow_do_not_prevent_withdrawal(){
  let mut s=state(Mode::Coucou);let(reg,mut rx)=add(&mut s,"a",true);let a=route(reg).unwrap();
  let tx=s.requests["a"].sender.clone();for _ in 0..4{assert!(tx.try_send(Reply::Decision("allow".into())).is_ok());}
  set(&mut s,Mode::Terminal);set(&mut s,Mode::Coucou);
  let outcome=tokio::time::timeout(Duration::from_millis(100),until_current(Some(&a),wait_for_decision("a",&mut rx,Instant::now()))).await.unwrap();assert!(matches!(outcome,DecisionOutcome::Handoff("external-terminal-mode")));
 }
 #[tokio::test]async fn changing_route_interrupts_a_real_pending_wait_without_waiting_for_the_card_deadline(){
  let mut s=state(Mode::Coucou);let(reg,mut rx)=add(&mut s,"a",true);let a=route(reg).unwrap();let before=Instant::now();
  let wait=until_current(Some(&a),wait_for_decision("a",&mut rx,before));let withdraw=async{tokio::time::sleep(Duration::from_millis(10)).await;set(&mut s,Mode::Terminal);};
  let(outcome,_)=tokio::join!(wait,withdraw);assert!(matches!(outcome,DecisionOutcome::Handoff("not-shown"|"external-terminal-mode")));assert!(before.elapsed()<Duration::from_millis(500));
 }
}
