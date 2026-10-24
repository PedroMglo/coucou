//! Secondary launches can arrive while native profile migration is running.
//! Do not touch absent state, replay stale voice or defer targetless decisions.
use std::{collections::VecDeque,sync::Mutex,time::{Duration,Instant}};
use tauri::{AppHandle,Manager};
const TTL:Duration=Duration::from_secs(30);
const MAX:usize=8;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Request{Preview,Voice,Shortcut(&'static str),Hub}
enum Route{Now(Request),Queued,Rejected}
#[derive(Default)]struct Queue{ready:bool,pending:VecDeque<(Instant,Request)>}
impl Queue{
    fn receive(&mut self,request:Request,now:Instant)->Route{
        if self.ready{return Route::Now(request);}
        self.pending.retain(|(at,_)|now.saturating_duration_since(*at)<=TTL);
        // A delayed shortcut could act on a different pending decision/session.
        if matches!(request,Request::Shortcut(_)){return Route::Rejected;}
        if self.pending.back().is_some_and(|(_,last)|*last==request){
            if request==Request::Voice{self.pending.pop_back();}return Route::Queued;
        }
        if self.pending.len()>=MAX{return Route::Rejected;}self.pending.push_back((now,request));Route::Queued
    }
    fn finish(&mut self,now:Instant)->Vec<Request>{self.ready=true;self.pending.drain(..).filter(|(at,_)|now.saturating_duration_since(*at)<=TTL).map(|(_,r)|r).collect()}
}
#[derive(Default)]pub(crate)struct Requests(Mutex<Queue>);
impl Requests{
    pub(crate)fn initial(args:&[String])->Self{
        let mut queue=Queue::default();let initial=request(args);
        if matches!(initial,Request::Voice|Request::Preview){queue.receive(initial,Instant::now());}Self(Mutex::new(queue))
    }
}
fn request(args:&[String])->Request{
    if args.iter().any(|a|a=="--voice-preview"){Request::Preview}
    else if crate::voice_request(args){Request::Voice}
    else if let Some(action)=crate::shortcuts::from_args(args){Request::Shortcut(action)}else{Request::Hub}
}
fn dispatch(app:&AppHandle,request:Request){match request{
    Request::Preview=>{let app=app.clone();tauri::async_runtime::spawn(async move{crate::voice::voice_preview(app,None).await;});},
    Request::Voice=>crate::voice::session::toggle(app),
    Request::Shortcut(action)=>crate::shortcuts::dispatch(app,action),
    Request::Hub=>crate::show_hub(app,None),
}}
pub(crate)fn receive(app:&AppHandle,args:&[String]){
    let Some(state)=app.try_state::<Requests>()else{return;};
    let route=state.0.lock().map(|mut q|q.receive(request(args),Instant::now())).unwrap_or(Route::Rejected);
    match route{Route::Now(request)=>dispatch(app,request),Route::Queued=>{},Route::Rejected=>crate::log::line("O pedido não foi executado: a aplicação ainda está a abrir. Repete-o quando estiver pronta.")}
}
pub(crate)fn ready(app:&AppHandle){
    let app=app.clone();tauri::async_runtime::spawn(async move{
        // Keep primary/secondary voice toggles in one queue while the surfaces
        // settle, so a second toggle cancels instead of starting late.
        tokio::time::sleep(Duration::from_millis(800)).await;
        let Some(state)=app.try_state::<Requests>()else{return;};
        let pending=state.0.lock().map(|mut q|q.finish(Instant::now())).unwrap_or_default();for request in pending{dispatch(&app,request);}
    });
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn early_voice_is_delivered_once_only_after_ready(){let mut q=Queue::default();let now=Instant::now();assert!(matches!(q.receive(Request::Voice,now),Route::Queued));assert_eq!(q.finish(now),vec![Request::Voice]);assert!(q.finish(now).is_empty());}
    #[test]fn repeated_toggle_cancels_and_preview_duplicates_do_not_multiply(){let mut q=Queue::default();let now=Instant::now();q.receive(Request::Voice,now);q.receive(Request::Voice,now);q.receive(Request::Preview,now);q.receive(Request::Preview,now);assert_eq!(q.finish(now),vec![Request::Preview]);}
    #[test]fn expired_voice_is_not_replayed_after_long_migration(){let mut q=Queue::default();let now=Instant::now();q.receive(Request::Voice,now);assert!(q.finish(now+TTL+Duration::from_millis(1)).is_empty());}
    #[test]fn targetless_shortcuts_are_rejected_early_and_forwarded_exactly_when_ready(){let mut q=Queue::default();let now=Instant::now();assert!(matches!(q.receive(Request::Shortcut("openChat"),now),Route::Rejected));q.finish(now);assert!(matches!(q.receive(Request::Shortcut("openChat"),now),Route::Now(Request::Shortcut("openChat"))));}
    #[test]fn pending_requests_are_bounded_and_existing_actions_are_not_overwritten(){let mut q=Queue::default();let now=Instant::now();for n in 0..MAX{assert!(matches!(q.receive(if n%2==0{Request::Hub}else{Request::Preview},now),Route::Queued));}assert!(matches!(q.receive(Request::Voice,now),Route::Rejected));assert_eq!(q.finish(now).len(),MAX);}
    #[test]fn primary_and_secondary_voice_toggles_share_the_same_pending_state(){let requests=Requests::initial(&["coucou".into(),"--voice-session".into()]);let mut queue=requests.0.lock().unwrap();queue.receive(Request::Voice,Instant::now());assert!(queue.finish(Instant::now()).is_empty());}
}
