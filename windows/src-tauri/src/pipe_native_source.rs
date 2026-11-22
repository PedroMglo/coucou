//! One app-owned Gravity hook connection. The documented hook has no native
//! request/turn/policy ID. A local epoch is a liveness guard, never a native ID.
use serde_json::Value;
use crate::{chat::HookBinding,pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture,Ticket,PolicyBinding}};
#[derive(Clone)]pub(super)struct Review{ticket:Ticket,binding:HookBinding}
pub(super)struct Gate{endpoint:Endpoint,review:Review}
impl Gate{
    pub(super)fn capture(binding:HookBinding,payload:&Value)->Result<Self,String>{
        if !binding.current()||payload["coucou_assistant_gate"]!=true||payload["coucou_agent"]!="antigravity"||payload["hook_event_name"]!="PermissionRequest"||payload["session_id"]!=binding.native_session()||payload["coucou_pet_provider"]!=binding.provider()||payload["tool_name"]!="Bash"{return Err("O hook já não pertence ao turno original.".into());}
        let call=&payload["coucou_native_call"];
        if call["name"]!="run_command"||!call["args"].is_object()||call["args"]["CommandLine"].as_str().is_none()||call["args"]["CommandLine"]!=payload["tool_input"]["command"]{return Err("A chamada nativa do hook não está disponível.".into());}
        let endpoint=Endpoint::new(Adapter::Antigravity,Transport::HookConnection)?;
        let ticket=endpoint.capture(Capture{request_id:None,session_id:binding.native_session(),turn_id:None,prompt_id:None,method:"PreToolUse",arguments:call,policy:PolicyBinding::unknown(),cancel:None})?;
        Ok(Self{endpoint,review:Review{ticket,binding}})
    }
    pub(super)fn presentation(&self,payload:&mut Value){
        // Original arguments stay on the private relay for capture/hash only.
        // The existing card gets its public command/cwd and readonly identity.
        payload.as_object_mut().map(|p|p.remove("coucou_native_call"));
        payload["nativeIdentity"]=serde_json::to_value(self.identity()).unwrap_or(Value::Null);
    }
    pub(super)fn review(&self)->Review{self.review.clone()}
    pub(super)fn identity(&self)->&crate::pet_native_decisions::Identity{self.review.ticket.identity()}
    pub(super)fn current(&self)->bool{self.review.current()}
    pub(super)fn binding(&self)->&HookBinding{&self.review.binding}
}
impl Drop for Gate{fn drop(&mut self){self.endpoint.close();self.review.ticket.cancel();}}
impl Review{
    pub(super)fn source_current(&self)->bool{self.ticket.current()}
    pub(super)fn current(&self)->bool{if !self.source_current(){return false;}if !self.binding.current(){self.ticket.cancel();return false;}true}
    pub(super)fn with_current<T>(&self,effect:impl FnOnce()->T)->Option<T>{let result=self.binding.with_current(||self.source_current().then(effect)).flatten();if result.is_none(){self.current();}result}
}

#[cfg(test)]mod tests{
    use super::*;use serde_json::json;use crate::chat::{Chat,Turn,TurnLease};use std::sync::Arc;
    fn owned()->(Chat,Turn,TurnLease){let chat=Chat::default();let turn=chat.begin_cli("gemini-cli",Arc::default()).unwrap();let lease=chat.lease(&turn);chat.start_task(&turn,"synthetic","/fixture","antigravity","synthetic-model",None).unwrap();chat.remember_session(&turn,"native-session","/fixture",false);(chat,turn,lease)}
    fn payload()->Value{json!({"coucou_assistant_gate":true,"coucou_pet_provider":"gemini-cli","coucou_agent":"antigravity","hook_event_name":"PermissionRequest","session_id":"native-session","tool_name":"Bash","tool_input":{"command":"synthetic command"},"coucou_native_call":{"name":"run_command","args":{"CommandLine":"synthetic command","Cwd":"/fixture","privateArgument":"PRIVATE ARGUMENT"}},"coucou_native_step":7})}
    #[test]fn hook_captures_connection_and_arguments_without_inventing_native_request_turn_or_policy(){
        let(chat,_turn,_lease)=owned();let raw=payload();let gate=Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&raw).unwrap();assert!(gate.current());let identity=gate.identity();
        assert_eq!(identity.adapter,Adapter::Antigravity.key());assert!(identity.request_id.is_null());assert!(identity.turn_id.is_none()&&identity.prompt_id.is_none());assert_eq!(identity.correlation,"connectionOnly");assert_eq!(identity.method,"PreToolUse");assert_eq!(identity.native_policy.coverage,"notObserved");
        let expected=ring::digest::digest(&ring::digest::SHA256,&serde_json::to_vec(&raw["coucou_native_call"]).unwrap()).as_ref().iter().map(|v|format!("{v:02x}")).collect::<String>();assert_eq!(identity.arguments_hash,expected);
        let mut public=raw.clone();gate.presentation(&mut public);assert!(public.get("coucou_native_call").is_none());assert!(!public.to_string().contains("PRIVATE ARGUMENT"));assert_eq!(public["coucou_native_step"],7);assert_eq!(public["tool_input"]["command"],"synthetic command");assert_eq!(public["nativeIdentity"]["transport"],"hookConnection");
    }
    #[test]fn unowned_mismatched_or_unobserved_calls_never_create_a_gate(){
        let(chat,_turn,_lease)=owned();assert!(chat.hook_binding("codex","native-session").is_none());assert!(chat.hook_binding("antigravity","foreign-session").is_none());
        for(field,value)in[("session_id",json!("foreign-session")),("coucou_pet_provider",json!("antigravity")),("coucou_agent",json!("codex")),("coucou_assistant_gate",json!(false)),("tool_name",json!("Write")),("coucou_native_call",Value::Null),("tool_input",json!({"command":"changed"}))]{let mut p=payload();p[field]=value;assert!(Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&p).is_err());}
    }
    #[test]fn retiring_one_hook_closes_only_that_source_and_never_cancels_the_owned_turn(){
        let(chat,turn,_lease)=owned();let binding=chat.hook_binding("antigravity","native-session").unwrap();let gate=Gate::capture(binding.clone(),&payload()).unwrap();let review=gate.review();let other=Gate::capture(binding.clone(),&payload()).unwrap();assert!(review.current());drop(gate);assert!(!review.source_current());assert!(other.current()&&binding.current());assert!(!turn.cancel.load(std::sync::atomic::Ordering::Acquire));
    }
    #[test]fn completing_and_reusing_the_same_native_session_never_rebinds_the_old_review(){
        let(chat,old,lease)=owned();let gate=Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&payload()).unwrap();drop(lease);assert!(!old.cancel.load(std::sync::atomic::Ordering::Acquire));assert!(!gate.current());
        let turn=chat.begin_cli("gemini-cli",Arc::default()).unwrap();let _next=chat.lease(&turn);chat.start_task(&turn,"next","/fixture","antigravity","synthetic-model",None).unwrap();chat.remember_session(&turn,"native-session","/fixture",false);
        assert!(!gate.current());let fresh=Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&payload()).unwrap();assert!(fresh.current());assert_ne!(gate.identity().endpoint,fresh.identity().endpoint);
    }
    #[test]fn a_hook_binding_does_not_keep_the_chat_or_native_turn_alive(){
        let(chat,_turn,lease)=owned();let gate=Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&payload()).unwrap();drop(lease);drop(chat);assert!(!gate.current());assert!(gate.review().with_current(||panic!("expired binding must not dispatch")).is_none());
    }
    #[test]fn an_ambiguous_native_alias_retires_the_old_review_without_choosing_an_arbitrary_pet(){
        let(chat,_turn,_lease)=owned();let gate=Gate::capture(chat.hook_binding("antigravity","native-session").unwrap(),&payload()).unwrap();
        let other=chat.begin_cli("antigravity",Arc::default()).unwrap();let lease=chat.lease(&other);chat.start_task(&other,"synthetic other","/fixture","antigravity","synthetic-model",None).unwrap();chat.remember_session(&other,"native-session","/fixture",false);
        assert!(chat.hook_owner("antigravity","native-session").is_none());assert!(chat.hook_binding("antigravity","native-session").is_none());assert!(!gate.current());drop(lease);assert!(chat.hook_binding("antigravity","native-session").is_some());assert!(!gate.current(),"an ended review must never revive when the ambiguity clears");
    }

}
