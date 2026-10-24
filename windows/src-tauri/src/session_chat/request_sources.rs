//! Capture pending native requests in receive order on one verified socket.
//! Broadcast JSON only identifies an existing capture; it never creates one.
use serde_json::Value;
use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,RequestBook,Capture,Ticket,PolicyBinding}};
use super::native_settings::{self,Cache};
const MARKER:&str="_mochiNativeRequestSequence";
#[derive(Clone)]pub(super)struct Sources{endpoint:Endpoint,book:RequestBook}
fn policy(cache:&Cache,thread:&str,at:u64)->PolicyBinding{
    let o=cache.snapshot(thread,at,u64::MAX);
    if !o.stale{if let Some(hash)=o.last_observed.as_ref().and_then(|s|s.fingerprint()){
        return PolicyBinding{coverage:"approvalSandboxSnapshot",hash:Some(hash),observed_at:o.observed_at.and_then(|s|s.checked_mul(1000)),sequence:o.sequence,source:Some("threadNotification")};
    }}PolicyBinding::unknown()
}
impl Sources{
    pub(super)fn new()->Result<Self,String>{let endpoint=Endpoint::new(Adapter::Codex,Transport::ControlSocket)?;Ok(Self{book:RequestBook::new(endpoint.clone()),endpoint})}
    pub(super)fn observe(&self,value:&mut Value,cache:&Cache,at:u64){
        if let Some(object)=value.as_object_mut(){object.remove(MARKER);}
        let method=value["method"].as_str().unwrap_or("");let p=&value["params"];
        if value.get("id").is_none(){
            if method=="serverRequest/resolved"{
                for t in self.book.tickets(){if p["requestId"]==t.identity().request_id&&p["threadId"]==t.identity().session_id{self.book.finish(&t);}}
            }else if matches!(method,"thread/settings/updated"|"thread/closed"|"thread/started"){
                let scope=p["threadId"].as_str().or_else(||p["thread"]["id"].as_str()).and_then(native_settings::thread_id);
                for t in self.book.tickets(){let own=native_settings::thread_id(&t.identity().session_id);
                    if scope.is_none()||own==scope{
                        if method=="thread/closed"{self.book.finish(&t);}else{let current=policy(cache,&t.identity().session_id,at);if current.hash.is_none()||current.hash!=t.identity().native_policy.hash{t.invalidate_policy();}}
                    }
                }
            }
            return;
        }
        let Some(scope)=crate::pet_codex_request_scope::extract(method,p)else{return;};
        let Ok(ticket)=self.book.register(Capture{request_id:value.get("id"),session_id:scope.session,turn_id:scope.turn,prompt_id:None,method,arguments:p,policy:policy(cache,scope.session,at),cancel:None})else{return;};
        value[MARKER]=serde_json::json!({"endpoint":ticket.identity().endpoint,"sequence":ticket.identity().sequence.to_string()});
    }
    pub(super)fn lookup(&self,event:&Value)->Option<Ticket>{
        let ticket=self.book.lookup(event.get("id")?,event[MARKER]["sequence"].as_str()?.parse().ok()?,event["method"].as_str()?,&event["params"])?;
        (event[MARKER]["endpoint"].as_str()==Some(ticket.identity().endpoint.as_str())).then_some(ticket)
    }
    pub(super)fn owns(&self,ticket:&Ticket)->bool{self.book.owns(ticket)}
    pub(super)fn finish(&self,ticket:&Ticket){self.book.finish(ticket);}
    pub(super)fn disconnect(&self){self.endpoint.close();for t in self.book.tickets(){self.book.finish(&t);}}
}
