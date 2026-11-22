//! Common native source facts/lifecycle. Transport adapters create these values;
//! no JSON, identity, hash or ticket grants an operation or owns a CLI process.
use crate::pet_config::Adapter;
use serde::Serialize;
use serde_json::Value;
use std::sync::{Arc,Weak,Mutex,atomic::{AtomicBool,AtomicU8,AtomicU64,Ordering}};
use ring::{digest,rand::{SecureRandom,SystemRandom}};
#[derive(Clone,Copy,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Transport{ManagedPipe,ControlSocket,HookConnection}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct PolicyBinding{pub coverage:&'static str,pub hash:Option<String>,pub observed_at:Option<u64>,pub sequence:Option<u64>,pub source:Option<&'static str>}
impl PolicyBinding{pub(crate)fn unknown()->Self{Self{coverage:"notObserved",hash:None,observed_at:None,sequence:None,source:None}}}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Identity{
    pub schema_version:u32,pub adapter:&'static str,pub transport:Transport,pub endpoint:String,
    pub request_id:Value,pub correlation:&'static str,pub session_id:String,
    pub turn_id:Option<String>,pub prompt_id:Option<Value>,pub method:String,
    pub arguments_hash:String,pub sequence:u64,pub native_policy:PolicyBinding,
}
struct Life{session:String,cancelled:Arc<AtomicBool>,reason:AtomicU8}
struct Source{id:String,adapter:Adapter,transport:Transport,closed:AtomicBool,next:AtomicU64,pending:Mutex<Vec<Weak<Life>>>}
#[derive(Clone)]pub(crate)struct Endpoint(Arc<Source>);
pub(crate)struct Capture<'a>{
    pub request_id:Option<&'a Value>,pub session_id:&'a str,pub turn_id:Option<&'a str>,
    pub prompt_id:Option<&'a Value>,pub method:&'a str,pub arguments:&'a Value,
    pub policy:PolicyBinding,pub cancel:Option<Arc<AtomicBool>>,
}
fn text(s:&str)->bool{!s.is_empty()&&s.len()<=512&&!s.chars().any(char::is_control)}
fn valid_id(v:&Value)->bool{v.as_i64().is_some()||v.as_str().is_some_and(text)}
impl Endpoint{
    pub(crate)fn new(adapter:Adapter,transport:Transport)->Result<Self,String>{let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|"Não foi possível identificar a origem nativa.".to_string())?;Ok(Self::with_id(adapter,transport,bytes.iter().map(|v|format!("{v:02x}")).collect()))}
    fn with_id(adapter:Adapter,transport:Transport,id:String)->Self{Self(Arc::new(Source{id,adapter,transport,closed:AtomicBool::new(false),next:AtomicU64::new(0),pending:Mutex::new(vec![])}))}
    #[cfg(test)]pub(crate)fn fixture(adapter:Adapter,transport:Transport,id:&str)->Self{Self::with_id(adapter,transport,id.into())}
    pub(crate)fn capture(&self,c:Capture<'_>)->Result<Ticket,String>{
        if !text(c.session_id)||!text(c.method)||c.turn_id.is_some_and(|s|!text(s))||c.request_id.is_some_and(|v|!valid_id(v))||c.prompt_id.is_some_and(|v|!valid_id(v))||c.request_id.is_none()&&!matches!(self.0.transport,Transport::HookConnection){return Err("A identidade do pedido nativo está incompleta.".into());}
        let args=serde_json::to_vec(c.arguments).map_err(|_|"Argumentos nativos inválidos.")?;if args.len()>256*1024{return Err("O pedido nativo excede o limite da revisão.".into());}
        let mut watched=self.0.pending.lock().map_err(|_|"A origem nativa está indisponível.")?;watched.retain(|w|w.upgrade().is_some_and(|l|l.reason.load(Ordering::Acquire)==0&&!l.cancelled.load(Ordering::Acquire)));if watched.len()>=128{return Err("A origem nativa tem demasiados pedidos pendentes.".into());}
        let mut current=self.0.next.load(Ordering::Acquire);
        let sequence=loop{if self.0.closed.load(Ordering::Acquire){return Err("A origem nativa já terminou.".into());}let next=current.checked_add(1).ok_or("A sequência da origem nativa esgotou-se.")?;match self.0.next.compare_exchange_weak(current,next,Ordering::AcqRel,Ordering::Acquire){Ok(_)=>break next,Err(actual)=>current=actual}};
        let arguments_hash=digest::digest(&digest::SHA256,&args).as_ref().iter().map(|v|format!("{v:02x}")).collect();
        let life=Arc::new(Life{session:c.session_id.into(),cancelled:c.cancel.unwrap_or_else(||Arc::new(AtomicBool::new(false))),reason:AtomicU8::new(0)});watched.push(Arc::downgrade(&life));
        Ok(Ticket{identity:Identity{schema_version:1,adapter:self.0.adapter.key(),transport:self.0.transport,endpoint:self.0.id.clone(),request_id:c.request_id.cloned().unwrap_or(Value::Null),correlation:if c.request_id.is_some(){"nativeRequest"}else{"connectionOnly"},session_id:c.session_id.into(),turn_id:c.turn_id.map(str::to_string),prompt_id:c.prompt_id.cloned(),method:c.method.into(),arguments_hash,sequence,native_policy:c.policy},source:Arc::downgrade(&self.0),life,#[cfg(test)]_fixture_owner:None})
    }
    pub(crate)fn invalidate_policy(&self,session:Option<&str>){if let Ok(all)=self.0.pending.lock(){for life in all.iter().filter_map(Weak::upgrade){if session.is_none_or(|s|s==life.session){if life.reason.compare_exchange(0,1,Ordering::AcqRel,Ordering::Acquire).is_ok(){life.cancelled.store(true,Ordering::Release);}}}}}
    pub(crate)fn is_open(&self)->bool{!self.0.closed.load(Ordering::Acquire)}
    pub(crate)fn close(&self){self.0.closed.store(true,Ordering::Release);}
}
#[derive(Clone)]pub(crate)struct Ticket{identity:Identity,source:Weak<Source>,life:Arc<Life>,#[cfg(test)]_fixture_owner:Option<Endpoint>}
impl Ticket{
    fn source_current(&self)->bool{self.source.upgrade().is_some_and(|s|!s.closed.load(Ordering::Acquire))}
    pub(crate)fn current(&self)->bool{self.source_current()&&!self.life.cancelled.load(Ordering::Acquire)&&self.life.reason.load(Ordering::Acquire)==0}
    pub(crate)fn identity(&self)->&Identity{&self.identity}
    pub(crate)fn cancel(&self){self.life.reason.store(2,Ordering::Release);self.life.cancelled.store(true,Ordering::Release);}
    pub(crate)fn policy_invalidated(&self)->bool{self.source_current()&&self.life.reason.load(Ordering::Acquire)==1}
    /// The native caller withdrew this request but still needs its cancelled
    /// response. This is different from an already resolved/closed request.
    pub(crate)fn request_withdrawal(&self){if self.life.reason.compare_exchange(0,3,Ordering::AcqRel,Ordering::Acquire).is_ok(){self.life.cancelled.store(true,Ordering::Release);}}
    pub(crate)fn withdrawal_pending(&self)->bool{self.source_current()&&matches!(self.life.reason.load(Ordering::Acquire),1|3)}
    pub(crate)fn invalidate_policy(&self){if self.life.reason.compare_exchange(0,1,Ordering::AcqRel,Ordering::Acquire).is_ok(){self.life.cancelled.store(true,Ordering::Release);}}
    pub(crate)fn same_capture(&self,other:&Self)->bool{Weak::ptr_eq(&self.source,&other.source)&&Arc::ptr_eq(&self.life,&other.life)}
    #[cfg(test)]pub(crate)fn fixture()->(Self,Arc<AtomicBool>){let endpoint=Endpoint::fixture(Adapter::Codex,Transport::ManagedPipe,"private-native-fixture");let cancel=Arc::new(AtomicBool::new(false));let mut t=endpoint.capture(Capture{request_id:Some(&Value::from(77)),session_id:"native-thread",turn_id:Some("native-turn"),prompt_id:None,method:"item/commandExecution/requestApproval",arguments:&Value::Null,policy:PolicyBinding::unknown(),cancel:Some(cancel.clone())}).unwrap();t._fixture_owner=Some(endpoint);(t,cancel)}
}
/// Reader-owned correlation of pending wire IDs. A ticket is never recreated
/// from a delayed consumer frame. Retiring an old capture cannot affect reuse.
#[derive(Clone)]pub(crate)struct RequestBook{endpoint:Endpoint,pending:Arc<Mutex<std::collections::BTreeMap<String,Ticket>>>}
impl RequestBook{
    pub(crate)fn new(endpoint:Endpoint)->Self{Self{endpoint,pending:Arc::default()}}
    pub(crate)fn register(&self,c:Capture<'_>)->Result<Ticket,String>{
        let id=c.request_id.filter(|v|valid_id(v)).ok_or("O pedido nativo não tem um ID válido.")?;
        let key=serde_json::to_string(id).map_err(|_|"ID nativo inválido.")?;
        let mut pending=self.pending.lock().map_err(|_|"A origem nativa está indisponível.")?;
        if let Some(old)=pending.get(&key){old.cancel();return Err("O agente reutilizou um ID ainda pendente.".into());}
        if pending.len()>=128{return Err("A origem nativa tem demasiados pedidos pendentes.".into());}
        let ticket=self.endpoint.capture(c)?;pending.insert(key,ticket.clone());Ok(ticket)
    }
    pub(crate)fn tickets(&self)->Vec<Ticket>{self.pending.lock().map(|p|p.values().cloned().collect()).unwrap_or_default()}
    pub(crate)fn owns(&self,ticket:&Ticket)->bool{
        let Ok(key)=serde_json::to_string(&ticket.identity().request_id)else{return false;};
        ticket.current()&&self.pending.lock().is_ok_and(|p|p.get(&key).is_some_and(|t|t.same_capture(ticket)))
    }
    pub(crate)fn lookup(&self,id:&Value,sequence:u64,method:&str,arguments:&Value)->Option<Ticket>{
        let key=serde_json::to_string(id).ok()?;let p=self.pending.lock().ok()?;let t=p.get(&key)?;
        if !t.current()||t.identity().sequence!=sequence||t.identity().method!=method{return None;}
        let args=serde_json::to_vec(arguments).ok()?;if args.len()>256*1024{return None;}
        let hash=digest::digest(&digest::SHA256,&args).as_ref().iter().map(|v|format!("{v:02x}")).collect::<String>();
        (hash==t.identity().arguments_hash).then(||t.clone())
    }
    pub(crate)fn finish(&self,ticket:&Ticket){if let Ok(key)=serde_json::to_string(&ticket.identity().request_id){if let Ok(mut pending)=self.pending.lock(){if pending.get(&key).is_some_and(|t|t.same_capture(ticket)){pending.remove(&key);}}}ticket.cancel();}
}
#[cfg(test)]mod tests{
    use super::*;use serde_json::json;
    fn capture(e:&Endpoint,request:Option<&Value>)->Ticket{e.capture(Capture{request_id:request,session_id:"native-session",turn_id:None,prompt_id:Some(&json!(42)),method:"native/request",arguments:&json!({"private":"PRIVATE COMMAND"}),policy:PolicyBinding::unknown(),cancel:None}).unwrap()}
    #[test]fn every_native_adapter_keeps_its_declared_identity_and_unknown_policy(){for adapter in [Adapter::Codex,Adapter::Copilot,Adapter::Antigravity]{let e=Endpoint::new(adapter,Transport::ManagedPipe).unwrap();let t=capture(&e,Some(&json!(1)));assert!(t.current());assert_eq!(t.identity().adapter,adapter.key());assert!(t.identity().turn_id.is_none());assert_eq!(t.identity().prompt_id,Some(json!(42)));assert_eq!(t.identity().native_policy.coverage,"notObserved");assert!(!serde_json::to_string(t.identity()).unwrap().contains("PRIVATE COMMAND"));}}
    #[test]fn connection_only_hooks_do_not_invent_native_request_or_turn_ids(){let e=Endpoint::new(Adapter::Antigravity,Transport::HookConnection).unwrap();let t=capture(&e,None);assert!(t.identity().request_id.is_null());assert_eq!(t.identity().correlation,"connectionOnly");assert!(t.identity().turn_id.is_none());let pipe=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();assert!(pipe.capture(Capture{request_id:None,session_id:"s",turn_id:None,prompt_id:None,method:"request",arguments:&Value::Null,policy:PolicyBinding::unknown(),cancel:None}).is_err());}
    #[test]fn tickets_do_not_keep_their_native_source_alive(){let t={let e=Endpoint::new(Adapter::Codex,Transport::ControlSocket).unwrap();capture(&e,Some(&json!(1)))};assert!(!t.current());let e=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let t=capture(&e,Some(&json!(1)));e.close();assert!(!t.current());assert!(e.capture(Capture{request_id:Some(&json!(2)),session_id:"s",turn_id:None,prompt_id:None,method:"request",arguments:&Value::Null,policy:PolicyBinding::unknown(),cancel:None}).is_err());}
    #[test]fn reused_wire_ids_and_clones_preserve_capture_identity_without_retiring_a_later_capture(){let e=Endpoint::new(Adapter::Codex,Transport::ControlSocket).unwrap();let first=capture(&e,Some(&json!(1)));let clone=first.clone();let later=capture(&e,Some(&json!(1)));assert!(first.same_capture(&clone));assert!(!first.same_capture(&later));assert_ne!(first.identity().sequence,later.identity().sequence);first.cancel();assert!(!clone.current());assert!(later.current());}
    #[test]fn source_cancellation_and_policy_invalidation_cannot_be_reversed_by_a_clone(){let e=Endpoint::new(Adapter::Codex,Transport::ManagedPipe).unwrap();let t=capture(&e,Some(&json!(1)));t.invalidate_policy();assert!(!t.current()&&t.policy_invalidated());t.cancel();assert!(!t.policy_invalidated());let clone=t.clone();clone.invalidate_policy();assert!(!clone.current()&&!clone.policy_invalidated());}
    #[test]fn reader_book_rejects_duplicate_pending_ids_but_old_finish_cannot_retire_reuse(){
        let e=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let b=RequestBook::new(e);
        let request=json!(7);let args=json!({"sessionId":"s"});
        let capture=||Capture{request_id:Some(&request),session_id:"s",turn_id:None,prompt_id:None,method:"native/request",arguments:&args,policy:PolicyBinding::unknown(),cancel:None};
        let old=b.register(capture()).unwrap();assert!(b.register(capture()).is_err());assert!(!old.current());b.finish(&old);
        let new=b.register(capture()).unwrap();b.finish(&old);assert!(new.current());b.finish(&new);assert!(!new.current());
    }

}
