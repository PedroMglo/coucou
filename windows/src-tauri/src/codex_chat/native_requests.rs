//! Native request identity and liveness are captured by the pipe reader, before
//! a human callback can block. They are not reconstructed from UI JSON.
use serde_json::Value;
use std::{collections::BTreeMap,sync::{Arc,Mutex},ops::Deref};
use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture}};
pub(crate)use crate::pet_native_decisions::{PolicyBinding,Ticket};
fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_millis()as u64).unwrap_or(0)}
pub(super)struct Frame{pub value:Value,pub ticket:Option<Ticket>,pub invalid_request:bool}
impl Deref for Frame{type Target=Value;fn deref(&self)->&Value{&self.value}}
struct State{endpoint:Endpoint,pending:BTreeMap<String,Ticket>,overflow:bool,closed:bool,handshakes:BTreeMap<String,Option<String>>,
    #[cfg(target_os="linux")]policies:crate::session_chat::rpc::native_settings::Cache,
    #[cfg(target_os="linux")]policy_sources:BTreeMap<String,&'static str>,
}
impl State{
    fn policy(&self,raw:&str,at:u64)->PolicyBinding{
        #[cfg(target_os="linux")]{let o=self.policies.snapshot(raw,at,u64::MAX);if !o.stale{if let Some(hash)=o.last_observed.as_ref().and_then(|s|s.fingerprint()){return PolicyBinding{coverage:"approvalSandboxSnapshot",hash:Some(hash),observed_at:o.observed_at,sequence:o.sequence,source:self.policy_sources.get(&o.thread_id).copied()};}}}
        #[cfg(not(target_os="linux"))]let _=(raw,at);
        PolicyBinding::unknown()
    }
    #[cfg(target_os="linux")]fn invalidate_changed(&self,scope:Option<&str>,at:u64){
        for ticket in self.pending.values(){let own=crate::session_chat::rpc::native_settings::thread_id(&ticket.identity().session_id);if scope.is_none()||own.as_deref()==scope{let current=self.policy(&ticket.identity().session_id,at);if current.hash.is_none()||current.hash!=ticket.identity().native_policy.hash{ticket.invalidate_policy();}}}
    }
    fn observe_policy(&mut self,value:&Value){
        #[cfg(target_os="linux")]{
            use crate::session_chat::rpc::native_settings::thread_id;
            let at=now();
            if value.get("method").is_none(){
                if let Some(k)=id(&value["id"]){if let Some(expected)=self.handshakes.remove(&k){let scope=self.policies.response(expected.as_deref(),&value["result"],at);if let Some(scope)=&scope{if self.policy(scope,at).hash.is_some(){self.policy_sources.insert(scope.clone(),"threadResponse");}else{self.policy_sources.remove(scope);}}self.invalidate_changed(scope.as_deref(),at);}}
            }else if value.get("id").is_none()&&matches!(value["method"].as_str(),Some("thread/settings/updated"|"thread/closed"|"thread/started")){
                let scope=value.pointer("/params/threadId").or_else(||value.pointer("/params/thread/id")).and_then(Value::as_str).and_then(thread_id);self.policies.event(value,at);
                if let Some(scope)=&scope{if self.policy(scope,at).hash.is_some(){self.policy_sources.insert(scope.clone(),"threadNotification");}else{self.policy_sources.remove(scope);}}if value["method"]=="thread/closed"{
                    for ticket in self.pending.values(){if scope.is_none()||thread_id(&ticket.identity().session_id).as_deref()==scope.as_deref(){ticket.cancel();}}
                }else{self.invalidate_changed(scope.as_deref(),at);}
            }
        }
        #[cfg(not(target_os="linux"))]{if value.get("method").is_none(){if let Some(k)=id(&value["id"]){self.handshakes.remove(&k);}}}
    }
}
#[derive(Clone)]pub(super)struct Requests(Arc<Mutex<State>>);
fn id(value:&Value)->Option<String>{
    if value.as_i64().is_none()&&value.as_str().is_none_or(|v|v.is_empty()||v.len()>512||v.chars().any(char::is_control)){return None;}serde_json::to_string(value).ok()
}
fn key(params:&Value,name:&str)->Option<String>{params[name].as_str().filter(|v|!v.is_empty()&&v.len()<=512&&!v.chars().any(char::is_control)).map(str::to_string)}
impl Requests{
    pub(super)fn new()->Result<Self,String>{Endpoint::new(Adapter::Codex,Transport::ManagedPipe).map(Self::from_endpoint)}
    #[cfg(test)]fn with_endpoint(id:String)->Self{Self::from_endpoint(Endpoint::fixture(Adapter::Codex,Transport::ManagedPipe,&id))}
    fn from_endpoint(endpoint:Endpoint)->Self{Self(Arc::new(Mutex::new(State{endpoint,pending:BTreeMap::new(),overflow:false,closed:false,handshakes:BTreeMap::new(),
        #[cfg(target_os="linux")]policies:Default::default(),#[cfg(target_os="linux")]policy_sources:BTreeMap::new()})))}
    pub(super)fn expect_handshake(&self,request:&Value,method:&str,params:&Value)->Result<(),String>{
        if !matches!(method,"thread/start"|"thread/resume"){return Ok(());}
        let k=id(request).ok_or("Invalid native handshake ID.")?;let expected=if method=="thread/resume"{Some(key(params,"threadId").ok_or("Invalid native resume target.")?)}else{None};
        let mut state=self.0.lock().map_err(|_|"Native policy observations unavailable.")?;if state.closed||state.handshakes.len()>=16||state.handshakes.contains_key(&k){return Err("Native policy handshake is not available.".into());}state.handshakes.insert(k,expected);Ok(())
    }
    #[cfg(test)]pub(super)fn policy_snapshot(&self,thread:&str)->PolicyBinding{self.0.lock().unwrap().policy(thread,now())}
    pub(super)fn observe(&self,value:Value)->Frame{
        let mut state=self.0.lock().unwrap_or_else(|p|p.into_inner());let mut frame=Frame{value,ticket:None,invalid_request:false};state.observe_policy(&frame.value);
        let method=frame.value["method"].as_str().unwrap_or("");
        if method=="serverRequest/resolved"&&frame.value.get("id").is_none(){
            if let Some(k)=id(&frame.value["params"]["requestId"]){if state.pending.get(&k).is_some_and(|t|frame.value["params"]["threadId"]==t.identity().session_id){if let Some(t)=state.pending.remove(&k){t.cancel();}}}return frame;
        }
        if !crate::pet_codex_request_scope::supported(method)||frame.value.get("id").is_none(){return frame;}
        let p=&frame.value["params"];let identity=id(&frame.value["id"]).zip(crate::pet_codex_request_scope::extract(method,p));
        let Some((k,scope))=identity else{frame.invalid_request=true;return frame;};
        if state.closed||state.overflow||state.pending.len()>=128{frame.invalid_request=true;return frame;}
        if let Some(old)=state.pending.get(&k){old.cancel();frame.invalid_request=true;return frame;}
        let ticket=match state.endpoint.capture(Capture{request_id:Some(&frame.value["id"]),session_id:scope.session,turn_id:scope.turn,prompt_id:None,method,arguments:p,policy:state.policy(scope.session,now()),cancel:None}){Ok(t)=>t,Err(_)=>{frame.invalid_request=true;return frame;}};
        state.pending.insert(k,ticket.clone());frame.ticket=Some(ticket);frame
    }
    pub(super)fn finish(&self,ticket:&Ticket){let mut state=self.0.lock().unwrap_or_else(|p|p.into_inner());if let Some(k)=id(&ticket.identity().request_id){if state.pending.get(&k).is_some_and(|old|old.same_capture(ticket)){state.pending.remove(&k);}}ticket.cancel();}
    pub(super)fn disconnect(&self,overflow:bool){let mut state=self.0.lock().unwrap_or_else(|p|p.into_inner());state.closed=true;state.endpoint.close();state.overflow|=overflow;state.handshakes.clear();#[cfg(target_os="linux")]state.policies.disconnect();for ticket in state.pending.values(){ticket.cancel();}state.pending.clear();}
    pub(super)fn overflowed(&self)->bool{self.0.lock().map(|s|s.overflow).unwrap_or(true)}
}
#[cfg(test)]mod tests{
    use super::*;use serde_json::json;
    fn request(id:Value,thread:&str,turn:&str)->Value{json!({"id":id,"method":"item/commandExecution/requestApproval","params":{"threadId":thread,"turnId":turn,"command":"private fixture command"}})}
    #[test]fn native_numeric_and_text_ids_remain_distinct_and_endpoint_bound(){let a=Requests::with_endpoint("one".into());let b=Requests::with_endpoint("two".into());let one=a.observe(request(json!(1),"thread","turn")).ticket.unwrap();let text=a.observe(request(json!("1"),"thread","turn")).ticket.unwrap();let other=b.observe(request(json!(1),"thread","turn")).ticket.unwrap();assert_ne!(one.identity().request_id,text.identity().request_id);assert_ne!(one.identity().endpoint,other.identity().endpoint);assert!(one.current()&&text.current()&&other.current());assert_eq!(one.identity().arguments_hash.len(),64);assert!(!serde_json::to_string(one.identity()).unwrap().contains("private fixture command"));}
    #[test]fn terminal_resolution_cancels_only_the_exact_live_native_request(){let r=Requests::with_endpoint("one".into());let a=r.observe(request(json!(1),"thread","turn")).ticket.unwrap();let b=r.observe(request(json!(2),"thread","turn")).ticket.unwrap();r.observe(json!({"method":"serverRequest/resolved","params":{"threadId":"other","requestId":1}}));assert!(a.current());r.observe(json!({"method":"serverRequest/resolved","params":{"threadId":"thread","requestId":1}}));assert!(!a.current());assert!(b.current());r.finish(&b);assert!(!b.current());}
    #[test]fn repeated_wire_ids_cancel_the_original_and_never_create_another_pending_approval(){let r=Requests::with_endpoint("one".into());let a=r.observe(request(json!(1),"thread","turn")).ticket.unwrap();let duplicate=r.observe(request(json!(1),"other","different"));assert!(duplicate.invalid_request&&duplicate.ticket.is_none());assert!(!a.current());}
    #[test]fn disconnect_and_queue_overflow_retire_all_captured_tickets(){for overflow in [false,true]{let r=Requests::with_endpoint("one".into());let a=r.observe(request(json!(1),"thread","turn")).ticket.unwrap();r.disconnect(overflow);assert!(!a.current());assert_eq!(r.overflowed(),overflow);assert!(r.observe(request(json!(2),"thread","turn")).invalid_request);}}
    #[test]fn old_frame_or_ticket_cannot_finish_a_reused_id_on_a_later_native_request(){let r=Requests::with_endpoint("one".into());let old=r.observe(request(json!(1),"thread","turn1")).ticket.unwrap();r.finish(&old);let new=r.observe(request(json!(1),"thread","turn2")).ticket.unwrap();r.finish(&old);assert!(new.current());assert!(!old.current());assert_ne!(old.identity().sequence,new.identity().sequence);}
    #[test]fn malformed_native_scope_is_not_promoted_to_an_unscoped_approval(){let r=Requests::with_endpoint("one".into());for value in [request(Value::Null,"thread","turn"),request(json!(1),"","turn"),request(json!(1),"thread","")]{let f=r.observe(value);assert!(f.invalid_request&&f.ticket.is_none());}}
    #[test]fn legacy_and_mcp_capture_only_their_schema_scope_and_original_outer_id(){
        let r=Requests::with_endpoint("scope-fixture".into());
        for(index,(method,params))in[("execCommandApproval",json!({"conversationId":"thread","callId":"call-not-turn"})),("applyPatchApproval",json!({"conversationId":"thread","callId":"patch-not-turn"})),("mcpServer/elicitation/request",json!({"threadId":"thread","serverName":"fixture","turnId":null}))].into_iter().enumerate(){
            let id=json!(index+10);let frame=r.observe(json!({"id":id,"method":method,"params":params}));let t=frame.ticket.unwrap();assert_eq!(t.identity().request_id,id);assert_eq!(t.identity().session_id,"thread");assert!(t.identity().turn_id.is_none());
            r.observe(json!({"method":"serverRequest/resolved","params":{"threadId":"foreign","requestId":id}}));assert!(t.current());r.observe(json!({"method":"serverRequest/resolved","params":{"threadId":"thread","requestId":id}}));assert!(!t.current());
        }
    }
    #[test]fn malformed_legacy_and_mcp_frames_never_become_unbound_human_callbacks(){
        let r=Requests::with_endpoint("malformed-scope".into());for(method,params)in[("execCommandApproval",json!({"conversationId":"thread"})),("applyPatchApproval",json!({"threadId":"thread","callId":"call"})),("mcpServer/elicitation/request",json!({"threadId":"thread","serverName":"fixture","turnId":false}))]{let f=r.observe(json!({"id":10,"method":method,"params":params}));assert!(f.invalid_request&&f.ticket.is_none());}
    }
}

#[cfg(all(test,target_os="linux"))]#[path="native_policy_tests.rs"]mod policy_tests;
