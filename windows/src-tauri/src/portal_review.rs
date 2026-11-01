//! Reviewed operations on the SAME portal actor. No model-visible raw authority
//! or second dispatcher: the native controller supplies the dispatched witness.
use super::*;
use std::sync::Arc;

pub(crate)type Guard=Arc<dyn Fn()->bool+Send+Sync>;
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Before{
    pub phase:&'static str,pub generation:Option<String>,pub session:Option<String>,pub request:Option<String>,
    pub shortcuts:Vec<Wanted>,pub accepted:Option<Vec<Bound>>,
}
impl Before{
    pub(super)fn capture(phase:&Phase)->Self{match phase{
        Phase::Idle=>Self{phase:"idle",generation:None,session:None,request:None,shortcuts:vec![],accepted:None},
        Phase::Creating{gen,session,request,want}=>Self{phase:"creating",generation:Some(gen.to_string()),session:Some(session.clone()),request:Some(request.clone()),shortcuts:want.clone(),accepted:None},
        Phase::Binding{gen,session,request,want}=>Self{phase:"binding",generation:Some(gen.to_string()),session:Some(session.clone()),request:Some(request.clone()),shortcuts:want.clone(),accepted:None},
        Phase::Bound{gen,session,want,shortcuts,..}=>Self{phase:"bound",generation:Some(gen.to_string()),session:Some(session.clone()),request:None,shortcuts:want.clone(),accepted:shortcuts.clone()},
    }}
    #[cfg(test)]pub(crate)fn idle()->Self{Self::capture(&Phase::Idle)}
}
#[derive(Clone,Debug,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Outcome{pub verified:bool,pub native_effects_started:bool,pub reason:String,pub session:Option<String>,pub shortcuts:Option<Vec<Bound>>}
impl Outcome{
    pub(crate)fn refused(reason:&str,started:bool)->Self{Self{verified:false,native_effects_started:started,reason:reason.into(),session:None,shortcuts:None}}
}
pub(super)struct Flight{
    pub(super)plan:Reservation,guard:Guard,lifetime:Guard,reply:Option<mpsc::Sender<Outcome>>,awaiting:bool,verified:bool,started:bool,new_session_possible:bool,
}
impl Portal{
    pub(super)fn begin_review(&mut self,plan:Reservation,guard:Guard,lifetime:Guard,reply:mpsc::Sender<Outcome>,on_event:&impl Fn(Event)){
        if self.reviewed.is_some()||!self.peer_alive||!guard()||!lifetime()||plan.peer!=self.peer||plan.client!=self.sender
            ||plan.generation<=self.last_gen||Before::capture(&self.phase)!=plan.previous||!matches!(plan.previous.phase,"idle"|"bound"){
            let _=reply.send(Outcome::refused("native plan changed or authority revoked before effect",false));return;
        }
        if self.require_live_peer().is_err(){let _=reply.send(Outcome::refused("native peer no longer live",false));return;}
        self.last_gen=plan.generation;
        self.reviewed=Some(Flight{plan,guard,lifetime,reply:Some(reply),awaiting:false,verified:false,started:false,new_session_possible:false});
        if let Err(reason)=self.start_review_io(){self.reply_review(false,&reason);}
        self.review_tick(on_event);
    }
    fn review_allowed(&self)->bool{self.peer_alive&&self.reviewed.as_ref().is_some_and(|f|(f.guard)()&&(f.lifetime)())}
    fn reviewed_call(&mut self,msg:Message)->Result<Message,String>{
        if !self.review_allowed(){return Err("review revoked before native dispatch".into());}
        self.reviewed.as_mut().unwrap().started=true;
        if msg.interface().as_deref()==Some(IFACE)&&msg.member().as_deref()==Some("CreateSession"){self.reviewed.as_mut().unwrap().new_session_possible=true;}
        let closing=if msg.interface().as_deref()==Some(SESSION)&&msg.member().as_deref()==Some("Close"){msg.path().map(|p|p.to_string())}else{None};
        let answer=self.call(msg)?;
        if let Some(path)=closing{self.remember_close(&path);}
        if !self.review_allowed(){return Err("review revoked during native IO; effect outcome unconfirmed".into());}Ok(answer)
    }
    fn start_review_io(&mut self)->Result<(),String>{
        let plan=self.reviewed.as_ref().unwrap().plan.clone();
        // The reviewed previous state is exact. Never close a renderer-selected
        // target or an alternative returned by a different service.
        if let Some(request)=&plan.previous.request{self.reviewed_call(Message::new_method_call(self.peer.as_str(),request.as_str(),REQUEST,"Close")?)?;}
        if let Some(session)=&plan.previous.session{self.reviewed_call(Message::new_method_call(self.peer.as_str(),session.as_str(),SESSION,"Close")?)?;}
        self.phase=Phase::Idle;
        if plan.shortcuts.is_empty(){self.reply_review(true,"all requested bindings are off; reviewed previous session closed");return Ok(());}
        let mut options=PropMap::new();
        options.insert("handle_token".into(),Variant(Box::new(plan.create_request.rsplit('/').next().ok_or("invalid request")?.to_owned())));
        options.insert("session_handle_token".into(),Variant(Box::new(plan.session.rsplit('/').next().ok_or("invalid session")?.to_owned())));
        // Set expected state before IO: a lost ACK may still have created it.
        self.phase=Phase::Creating{gen:plan.generation,request:plan.create_request.clone(),session:plan.session.clone(),want:plan.shortcuts.clone()};
        let reply=self.reviewed_call(method(OBJECT,IFACE,"CreateSession")?.append1(options))?;
        if reply.read1::<Path>().map(|p|p.to_string()).ok().as_deref()!=Some(plan.create_request.as_str()){return Err("CreateSession returned an unreviewed request; no fallback".into());}Ok(())
    }
    pub(super)fn review_response(&mut self,path:&str,msg:&Message,on_event:&impl Fn(Event)){
        let Some(f)=self.reviewed.as_ref()else{return};if f.awaiting{return;}
        let expected=match &self.phase{Phase::Creating{request,..}|Phase::Binding{request,..}=>request.as_str(),_=>return};if expected!=path{return;}
        if !self.review_allowed(){self.abort_review("authority revoked while native response awaited",on_event);return;}
        let answer=parse_response(msg);
        let plan=self.reviewed.as_ref().unwrap().plan.clone();
        match &self.phase{
            Phase::Creating{..}=>{
                let result=(||->Result<(),String>{
                    let values=answer.map_err(|code|format!("CreateSession native refusal {code}"))?;
                    if session_from_results(&values).as_deref()!=Some(plan.session.as_str()){return Err("CreateSession returned an unreviewed session; no fallback".into());}
                    let wanted:Vec<(String,PropMap)>=plan.shortcuts.iter().map(|w|{let mut p=PropMap::new();p.insert("description".into(),Variant(Box::new(w.description.clone())));if let Some(t)=&w.trigger{p.insert("preferred_trigger".into(),Variant(Box::new(t.clone())));}(w.id.clone(),p)}).collect();
                    let mut options=PropMap::new();options.insert("handle_token".into(),Variant(Box::new(plan.bind_request.rsplit('/').next().ok_or("invalid bind request")?.to_owned())));
                    self.phase=Phase::Binding{gen:plan.generation,request:plan.bind_request.clone(),session:plan.session.clone(),want:plan.shortcuts.clone()};
                    let reply=self.reviewed_call(method(OBJECT,IFACE,"BindShortcuts")?.append3(Path::new(plan.session.clone())?,wanted,"").append1(options))?;
                    if reply.read1::<Path>().map(|p|p.to_string()).ok().as_deref()!=Some(plan.bind_request.as_str()){return Err("BindShortcuts returned an unreviewed request; no fallback".into());}Ok(())
                })();if let Err(reason)=result{self.reply_review(false,&reason);}
            },
            Phase::Binding{..}=>match answer{
                Ok(values)=>{
                    let bound=values.get("shortcuts").and_then(|v|parse_shortcuts(&*v.0));
                    let wanted:std::collections::BTreeSet<_>=plan.shortcuts.iter().map(|w|w.id.as_str()).collect();
                    let complete=bound.as_ref().is_some_and(|list|list.len()==wanted.len()&&list.iter().map(|b|b.id.as_str()).collect::<std::collections::BTreeSet<_>>()==wanted);
                    self.phase=Phase::Bound{gen:plan.generation,session:plan.session.clone(),want:plan.shortcuts.clone(),shortcuts:bound,pressed:Default::default()};
                    self.reply_review(complete,if complete{"native binding observed; durable receipt pending"}else{"native list missing, partial or unexpected; no complete success"});
                },
                Err(code)=>self.reply_review(false,&format!("BindShortcuts native refusal {code}")),
            },_=>{}
        }
    }
    fn reply_review(&mut self,verified:bool,reason:&str){
        let Some(f)=self.reviewed.as_mut()else{return};f.awaiting=true;f.verified=verified;
        let shortcuts=match &self.phase{Phase::Bound{gen,shortcuts,..}if *gen==f.plan.generation=>shortcuts.clone(),_if_empty if f.plan.shortcuts.is_empty()&&verified=>Some(vec![]),_=>None};
        if let Some(reply)=f.reply.take(){let _=reply.send(Outcome{verified,native_effects_started:f.started,reason:reason.into(),session:f.new_session_possible.then(||f.plan.session.clone()),shortcuts});}
    }
    pub(super)fn review_tick(&mut self,on_event:&impl Fn(Event)){
        let revoked=self.reviewed.as_ref().is_some_and(|f|!(f.lifetime)()||(!f.awaiting&&!(f.guard)()));
        if revoked{self.abort_review("native review cancelled, expired or source revoked",on_event);}
    }
    pub(super)fn abort_review(&mut self,reason:&str,on_event:&impl Fn(Event)){
        let Some(mut f)=self.reviewed.take()else{return};
        if let Some(reply)=f.reply.take(){let _=reply.send(Outcome::refused(reason,f.started));}
        // Cleanup only a reviewed, owned target; no claim of remote rollback.
        if f.new_session_possible&&self.peer_alive{
            if matches!(&self.phase,Phase::Creating{gen,..}|Phase::Binding{gen,..}if *gen==f.plan.generation){
                let request=match &self.phase{Phase::Creating{request,..}|Phase::Binding{request,..}=>request.clone(),_=>unreachable!()};self.close(&request,REQUEST);
            }
            self.close(&f.plan.session,SESSION);
        }
        // Closing the previous session can have taken effect even when its ACK
        // was lost. Never resume dispatch from that old local Bound state.
        if f.started{self.phase=Phase::Idle;}
        if f.started{on_event(Event::Failed{gen:f.plan.generation,reason:format!("{reason}; no automatic replay")});}
    }
    pub(super)fn cleanup_published(&mut self,generation:u64,on_event:&impl Fn(Event))->bool{
        if self.reviewed.is_some(){return false;}
        let session=match &self.phase{
            Phase::Bound{gen,session,..}if *gen==generation=>Some(session.clone()),
            Phase::Idle if self.last_gen==generation=>None,
            _=>return false,
        };
        self.phase=Phase::Idle; // Withdraw input before waiting for a close ACK.
        let ok=session.as_ref().is_none_or(|path|self.close_ack(path));
        on_event(Event::Failed{gen:generation,reason:"local publication not committed; owned session cleanup only, no replay".into()});ok
    }
    pub(super)fn finish_review(&mut self,generation:u64,commit:Option<Guard>,on_event:&impl Fn(Event))->bool{
        // Discard key events received while publication was withheld. Also
        // observe Closed/Changed/owner loss before accepting a local commit.
        let _=self.channel.read_write(Some(Duration::ZERO));
        while let Some(msg)=self.channel.pop_message(){self.message(&msg,on_event);}
        let Some(f)=self.reviewed.as_ref()else{return false};
        if f.plan.generation!=generation{return false;}
        let native_matches=matches!(&self.phase,Phase::Bound{gen,session,..}if *gen==generation&&session==&f.plan.session)||(f.plan.shortcuts.is_empty()&&matches!(self.phase,Phase::Idle));
        let ok=f.awaiting&&f.verified&&self.peer_alive&&(f.lifetime)()&&native_matches&&commit.as_ref().is_some_and(|guard|guard());
        if !ok{self.abort_review("receipt not committed or current",on_event);return false;}
        let bound=match &self.phase{Phase::Bound{shortcuts,..}=>shortcuts.clone(),_=>Some(vec![])};
        self.reviewed=None;on_event(Event::Bound{gen:generation,shortcuts:bound});true
    }
}
