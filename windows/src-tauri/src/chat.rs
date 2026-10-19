// The chat, whoever answers it: the conversation, the system prompt, and which
// provider a turn goes to. Each provider's own wire format lives in its module:
// claude.rs (Anthropic), openai_compat.rs (OpenAI, Google AI, OpenRouter) and
// local_chat.rs (Ollama, LM Studio, any OpenAI-compatible server).
//
// API keys never leave the credential store and file bytes never cross the IPC
// boundary: the island sends the question and gets the answer's text back.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle,Emitter};

use crate::settings::Settings;
use crate::{claude, local_chat, openai_compat, secrets};

pub const ANTHROPIC: &str = "anthropic";

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChatContext {
    File { name: String, path: String },
    Window { app_name: String, title: String, url: Option<String> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReply {
    pub text: String,
}

/// A model a provider offers, for the picker in the chat view.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

// ── Conversation ──────────────────────────────────────────────────────────────

/// Conversations belong to their logical pet/provider, never to the currently
/// selected window. Other pets can continue working independently.
#[derive(Clone, Default)]
pub struct Chat {
    inner: Arc<Mutex<Conversations>>,
    disk: Option<Arc<std::path::PathBuf>>,
    disk_error:bool,
    admission:Option<Arc<AdmissionTicket>>,
}

struct AdmissionTicket {
    provider:String,
    id:String,
    epoch:u64,
    cancel:Arc<AtomicBool>,
    bound:AtomicBool,
}

#[derive(Default,Clone)]
struct Conversations { next: u64, pets: HashMap<String, Conversation>, archives:HashMap<String,Vec<SavedConversation>> }
#[derive(Default,Clone)]
struct Conversation {
    id:String,
    created:u64,
    updated:u64,
    pending:Option<Arc<AdmissionTicket>>,
    live_text:String,
    live_query:String,
    live_sent:Option<std::time::Instant>,
    epoch: u64,
    native: Vec<Value>,
    plain: Vec<Value>,
    session: Option<CliSession>,
    active: Option<Arc<AtomicBool>>,
    turn_cancel:Option<Arc<AtomicBool>>,
    tasks: Vec<crate::assistant_tasks::Task>,
    task_saved: Option<std::time::Instant>,
    save_error:Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CliSession { pub id: String, pub cwd: String, pub full_access: bool }
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SavedConversation {
    #[serde(default)] pub id:String,
    #[serde(default)] pub created:u64,
    #[serde(default)] pub updated:u64,
    pub messages: Vec<Value>, pub session: Option<CliSession>,
    #[serde(default)] pub tasks: Vec<crate::assistant_tasks::Task>,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct ConversationSummary {
    pub id:String,pub provider:String,pub title:String,pub preview:String,
    pub created:u64,pub updated:u64,pub message_count:usize,pub current:bool,
}
#[derive(Serialize)]
pub struct ConversationView {pub id:String,pub messages:Vec<Value>,pub busy:bool,pub turn:u64,pub stream:String,pub query:String,#[serde(skip_serializing_if="Option::is_none")]pub notice:Option<String>}

/// Pins the channel selected by a writing surface until its send finishes,
/// including async provider/model discovery before the native turn starts.
pub struct ConversationReservation {chat:Chat,ticket:Arc<AdmissionTicket>,app:Option<AppHandle>}
impl ConversationReservation {
    pub fn channel(&self)->Chat {
        let mut channel=self.chat.clone();channel.admission=Some(self.ticket.clone());channel
    }
    pub fn broadcast(mut self,app:&AppHandle,query:&str)->Self {
        self.app=Some(app.clone());
        let event={let mut all=self.chat.inner.lock().unwrap();
            all.pets.get_mut(&self.ticket.provider).filter(|c|pending_matches(c,&self.ticket)).map(|c|{
                c.live_query=query.chars().take(4000).collect();
                json!({"provider":self.ticket.provider,"conversationId":c.id,"turn":c.epoch,"busy":true,"query":c.live_query})
            })};
        if let Some(event)=event{let _=app.emit("chat-turn-state",event);}self
    }
}
impl Drop for ConversationReservation {
    fn drop(&mut self) {
        let mut all=self.chat.inner.lock().unwrap();
        let event=all.pets.get_mut(&self.ticket.provider).and_then(|c|{
            if pending_matches(c,&self.ticket){
                self.ticket.cancel.store(true,Ordering::Release);c.pending=None;c.live_text.clear();c.live_query.clear();
            }else if c.id!=self.ticket.id||c.epoch!=self.ticket.epoch||c.busy(){return None;}
            // Cloud providers do not own an AppHandle in their TurnLease. The
            // send's guard still announces completion without touching state.
            Some(json!({"provider":self.ticket.provider,"conversationId":c.id,"turn":c.epoch,"busy":false,"query":""}))
        });drop(all);
        if let(Some(app),Some(event))=(&self.app,event){let _=app.emit("chat-turn-state",event);let _=app.emit("assistant-history-changed",&self.ticket.provider);}
    }
}

fn pending_matches(conversation:&Conversation,ticket:&Arc<AdmissionTicket>)->bool {
    conversation.id==ticket.id&&conversation.epoch==ticket.epoch
        &&conversation.pending.as_ref().is_some_and(|pending|Arc::ptr_eq(pending,ticket))
}
fn active_matches(conversation:&Conversation,ticket:&Arc<AdmissionTicket>)->bool {
    conversation.id==ticket.id&&conversation.epoch==ticket.epoch
        &&conversation.active.as_ref().is_some_and(|active|Arc::ptr_eq(active,&ticket.cancel))
        &&conversation.turn_cancel.as_ref().is_some_and(|active|Arc::ptr_eq(active,&ticket.cancel))
}

impl Conversation {
    fn identify(&mut self) {
        if self.id.is_empty() {self.id=crate::assistant_memory::new_id();}
        if self.created==0 {self.created=crate::assistant_memory::now();self.updated=self.created;}
    }
    fn saved(&self)->SavedConversation {SavedConversation{id:self.id.clone(),created:self.created,updated:self.updated,messages:self.plain.clone(),session:self.session.clone(),tasks:self.tasks.clone()}}
    fn from_saved(c:SavedConversation)->Self {Self{id:c.id,created:c.created,updated:c.updated,native:crate::assistant_memory::context(&c.messages),plain:c.messages,session:c.session,tasks:c.tasks,..Default::default()}}
    fn busy(&self)->bool {self.active.is_some() || self.pending.is_some()}
    fn view(&self)->ConversationView {
        let busy=self.busy();
        ConversationView{id:self.id.clone(),messages:self.plain.clone(),busy,turn:self.epoch,stream:if busy{self.live_text.clone()}else{String::new()},query:if busy{self.live_query.clone()}else{String::new()},notice:self.save_error.clone()}
    }
}

fn summary(provider:&str,c:&SavedConversation,current:bool)->ConversationSummary {
    let title=c.messages.iter().find(|m|m["role"]=="user").and_then(|m|m["content"].as_str()).unwrap_or("Nova conversa").lines().next().unwrap_or("Nova conversa").chars().take(80).collect();
    let preview=c.messages.last().and_then(|m|m["content"].as_str()).unwrap_or("").chars().take(160).collect();
    ConversationSummary{id:c.id.clone(),provider:provider.into(),title,preview,created:c.created,updated:c.updated,message_count:c.messages.len(),current}
}

#[derive(Clone)]
pub struct Turn {
    epoch: u64,
    pub provider: String,
    pub first: bool,
    pub history: Vec<Value>,
    pub session: Option<CliSession>,
    pub cancel: Arc<AtomicBool>,
}
pub const BUSY_ERROR:&str="Este pet já está a executar uma tarefa. Abre a tarefa para a acompanhar ou interromper.";
pub struct TurnLease {chat:Chat,turn:Turn,app:Option<AppHandle>}
impl TurnLease {
    pub fn broadcast(mut self,app:&AppHandle,query:&str)->Self {
        self.app=Some(app.clone());
        let event={
            let mut all=self.chat.inner.lock().unwrap();
            all.pets.get_mut(&self.turn.provider).filter(|c|c.epoch==self.turn.epoch&&c.turn_cancel.as_ref().is_some_and(|cancel|Arc::ptr_eq(cancel,&self.turn.cancel))).map(|c|{
                c.live_query=query.chars().take(4000).collect();
                json!({"provider":self.turn.provider,"conversationId":c.id,"turn":c.epoch,"busy":true,"query":c.live_query})
            })
        };
        if let Some(event)=event {let _=app.emit("chat-turn-state",event);}
        self
    }
}
impl Drop for TurnLease {
    fn drop(&mut self) {
        let mut all=self.chat.inner.lock().unwrap();
        if let Some(c)=all.pets.get_mut(&self.turn.provider).filter(|c|c.epoch==self.turn.epoch
            &&c.turn_cancel.as_ref().is_some_and(|cancel|Arc::ptr_eq(cancel,&self.turn.cancel))
            &&c.active.as_ref().is_none_or(|cancel|Arc::ptr_eq(cancel,&self.turn.cancel))) {
            c.active=None;c.turn_cancel=None;c.live_text.clear();c.live_query.clear();
            let event=json!({"provider":self.turn.provider,"conversationId":c.id,"turn":c.epoch,"busy":false,"query":""});
            drop(all);
            if let Some(app)=&self.app {let _=app.emit("chat-turn-state",event);let _=app.emit("assistant-history-changed",&self.turn.provider);}
        }
    }
}

impl Chat {
    pub fn persistent(path: std::path::PathBuf) -> Self {
        let (saved,disk_error)=match crate::assistant_memory::load(&path) {Ok(saved)=>(saved,false),Err(_)=>{crate::log::line("assistant memory: could not read library; keeping disk unchanged");(Default::default(),true)}};
        let pets=saved.pets.into_iter().map(|(provider,mut c)| {
            for task in &mut c.tasks {task.recover();}
            (provider,Conversation::from_saved(c))
        }).collect();
        Self {inner:Arc::new(Mutex::new(Conversations{next:0,pets,archives:saved.archives})),disk:Some(Arc::new(path)),disk_error,admission:None}
    }

    fn persist(&self,c:&Conversations) {
        if self.persist_checked(c).is_err() {crate::log::line("assistant memory: could not save local conversation");}
    }
    fn persist_checked(&self,c:&Conversations)->Result<(),String> {
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        let Some(path)=&self.disk else{return Ok(())};
        let pets=c.pets.iter().filter(|(p,_)|crate::assistant_memory::supported(p)).map(|(p,c)|(p.clone(),c.saved())).collect();
        crate::assistant_memory::save(path,&crate::assistant_memory::Library{pets,archives:c.archives.clone()})
            .map_err(|_|"Não foi possível guardar a biblioteca (limite de 128 MiB). O ficheiro anterior foi preservado; verifica o espaço e as permissões da pasta local.".into())
    }

    /// Retry a failed dialogue write before starting work or replacing a channel.
    fn flush_pending(&self,all:&mut Conversations)->Result<(),String>{
        if all.pets.values().any(|c|c.save_error.is_some()){
            self.persist_checked(all)?;
            for c in all.pets.values_mut(){c.save_error=None;}
        }Ok(())
    }

    pub fn history(&self,provider:&str)->Vec<Value> {
        self.inner.lock().unwrap().pets.get(provider).map(|c|c.plain.clone()).unwrap_or_default()
    }

    pub fn conversation(&self,provider:&str)->Result<ConversationView,String> {
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        if !crate::assistant_memory::supported(provider) {return Err("Este pet não tem biblioteca de conversas.".into());}
        let mut all=self.inner.lock().unwrap();let c=all.pets.entry(provider.into()).or_default();c.identify();
        Ok(c.view())
    }

    /// Capture a document drop's owner without cloning its public archive.
    pub fn conversation_id(&self,provider:&str)->Result<String,String> {
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        if !crate::assistant_memory::supported(provider) {return Err("Este pet não tem biblioteca de conversas.".into());}
        let mut all=self.inner.lock().unwrap();let c=all.pets.entry(provider.into()).or_default();c.identify();
        Ok(c.id.clone())
    }
    pub fn try_conversation_ids(&self,providers:&[String])->Result<std::collections::BTreeMap<String,String>,String>{
        if self.disk_error{return Err("A biblioteca local não pôde ser lida. O documento não foi anexado.".into());}
        if providers.iter().any(|p|!crate::assistant_memory::supported(p)){return Err("Este pet não tem biblioteca de conversas.".into());}
        let mut all=self.inner.try_lock().map_err(|_|"O histórico está a ser guardado. Volta a arrastar o documento daqui a instantes.".to_string())?;
        Ok(providers.iter().map(|p|{let c=all.pets.entry(p.clone()).or_default();c.identify();(p.clone(),c.id.clone())}).collect())
    }
    pub(crate) fn has_public_conversation(&self,provider:&str,id:&str)->bool{
        if self.disk_error||!crate::assistant_memory::supported(provider){return false;}
        let all=self.inner.lock().unwrap();all.pets.get(provider).is_some_and(|c|c.id==id)
            ||all.archives.get(provider).is_some_and(|list|list.iter().any(|c|c.id==id))
    }

    /// Reading a saved conversation never switches the provider's live channel.
    pub fn read_conversation(&self,provider:&str,id:&str)->Result<ConversationView,String> {
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        if !crate::assistant_memory::supported(provider) || !crate::assistant_memory::valid_id(id) {return Err("Conversa inválida para este pet.".into());}
        let all=self.inner.lock().unwrap();
        if let Some(c)=all.pets.get(provider).filter(|c|c.id==id) {return Ok(c.view());}
        let saved=all.archives.get(provider).and_then(|list|list.iter().find(|c|c.id==id)).ok_or("A conversa não pertence a este pet ou já não existe.")?;
        Ok(ConversationView{id:saved.id.clone(),messages:saved.messages.clone(),busy:false,turn:0,stream:String::new(),query:String::new(),notice:None})
    }

    pub fn conversation_for_session(&self,provider:&str,session:&str)->Option<String> {
        let all=self.inner.lock().unwrap();
        if let Some(c)=all.pets.get(provider).filter(|c|c.session.as_ref().is_some_and(|s|s.id==session)){return Some(c.id.clone());}
        all.archives.get(provider)?.iter().find(|c|c.session.as_ref().is_some_and(|s|s.id==session)).map(|c|c.id.clone())
    }

    pub fn reserve_conversation(&self,provider:&str,expected:Option<&str>)->Result<ConversationReservation,String> {
        let provider=if provider.is_empty(){ANTHROPIC}else{provider};
        if self.admission.is_some(){return Err("The CLI request was cancelled.".into());}
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        let mut all=self.inner.lock().unwrap();self.flush_pending(&mut all)?;let c=all.pets.entry(provider.into()).or_default();c.identify();
        if expected.is_some_and(|id|id!=c.id) {return Err("A conversa mudou noutra janela. Atualiza o histórico antes de enviar.".into());}
        if c.busy() {return Err(BUSY_ERROR.into());}
        all.next+=1;let epoch=all.next;
        let c=all.pets.get_mut(provider).unwrap();c.epoch=epoch;c.turn_cancel=None;c.live_text.clear();c.live_query.clear();c.live_sent=None;
        let ticket=Arc::new(AdmissionTicket{provider:provider.into(),id:c.id.clone(),epoch,cancel:Arc::new(AtomicBool::new(false)),bound:AtomicBool::new(false)});
        c.pending=Some(ticket.clone());
        Ok(ConversationReservation{chat:self.clone(),ticket,app:None})
    }

    /// An explicit task resume selects and reserves its original dialogue in
    /// one transaction. A journal row is not the currently selected conversation.
    pub fn reserve_task_resume(&self,id:&str)->Result<(crate::assistant_tasks::Task,ConversationReservation,bool),String>{
        if self.admission.is_some()||self.disk_error{return Err("Não foi possível reservar a retoma na biblioteca local.".into());}
        let mut all=self.inner.lock().unwrap();self.flush_pending(&mut all)?;
        let mut matching=all.pets.values().flat_map(|c|c.tasks.iter()).filter(|task|task.id==id);
        let task=matching.next().cloned().ok_or("A tarefa já não está no histórico local.")?;
        if matching.next().is_some(){return Err("O identificador da tarefa é ambíguo. A retoma foi recusada.".into());}
        task.resume_prompt()?;
        let provider=task.provider.clone();
        if all.pets.get(&provider).is_some_and(Conversation::busy){return Err(BUSY_ERROR.into());}
        let mut next=all.clone();let mut changed=false;
        let origin=if task.conversation_id.is_some()||task.session_id.is_some(){
            // A saved logical origin is authoritative. Only legacy rows without
            // it may infer their conversation from a confirmed native session.
            let matches_origin=|id:&str,session:Option<&CliSession>|match task.conversation_id.as_deref(){
                Some(origin)=>id==origin,
                None=>task.session_id.as_deref().is_some_and(|native|session.is_some_and(|s|s.id==native)),
            };
            let mut origins=Vec::new();
            if let Some(c)=next.pets.get(&provider).filter(|c|matches_origin(&c.id,c.session.as_ref())){origins.push(c.id.clone());}
            if let Some(archives)=next.archives.get(&provider){origins.extend(archives.iter().filter(|c|matches_origin(&c.id,c.session.as_ref())).map(|c|c.id.clone()));}
            if origins.len()!=1{return Err("Não foi possível identificar uma única conversa de origem desta tarefa. A conversa atual foi preservada.".into());}
            Some(origins.remove(0))
        }else{None};
        if let Some(target)=origin.as_deref(){
            if next.pets.get(&provider).is_none_or(|c|c.id!=target){
                let list=next.archives.get_mut(&provider).unwrap();
                let at=list.iter().position(|c|c.id==target).unwrap();let saved=list.remove(at);
                Self::archive(&mut next,&provider)?;
                let tasks=next.pets.get_mut(&provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
                let mut restored=Conversation::from_saved(saved);restored.tasks=tasks;
                next.pets.insert(provider.clone(),restored);changed=true;
            }
        }else{
            // Legacy rows with neither logical nor native origin start clean;
            // they must never inherit a different task's public context.
            Self::archive(&mut next,&provider)?;
            let tasks=next.pets.get_mut(&provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
            let mut fresh=Conversation{tasks,..Default::default()};fresh.identify();
            next.pets.insert(provider.clone(),fresh);changed=true;
        }
        next.next+=1;let epoch=next.next;
        let c=next.pets.get_mut(&provider).unwrap();c.identify();c.epoch=epoch;c.turn_cancel=None;c.live_text.clear();c.live_query.clear();c.live_sent=None;
        let ticket=Arc::new(AdmissionTicket{provider,id:c.id.clone(),epoch,cancel:Arc::new(AtomicBool::new(false)),bound:AtomicBool::new(false)});
        c.pending=Some(ticket.clone());
        if changed{self.persist_checked(&next)?;}
        *all=next;
        Ok((task,ConversationReservation{chat:self.clone(),ticket,app:None},changed))
    }

    /// Bind the reservation made synchronously by the command to this send's
    /// asynchronous future. Only the first unscoped caller may adopt it.
    pub fn bind_reservation(&self,provider:&str)->Result<ConversationReservation,String>{
        let all=self.inner.lock().unwrap();
        let ticket=if let Some(ticket)=&self.admission{
            if ticket.provider!=provider||ticket.cancel.load(Ordering::Acquire)
                ||!all.pets.get(provider).is_some_and(|c|pending_matches(c,ticket)){return Err("The CLI request was cancelled.".into());}
            ticket.clone()
        }else{
            let ticket=all.pets.get(provider).and_then(|c|c.pending.as_ref()).ok_or("The CLI request was cancelled.")?;
            if ticket.cancel.load(Ordering::Acquire)||ticket.bound.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("The CLI request was cancelled.".into());}
            ticket.clone()
        };
        Ok(ConversationReservation{chat:self.clone(),ticket,app:None})
    }

    pub fn check_admission(&self,provider:&str)->Result<(),String>{
        let Some(ticket)=&self.admission else{return Ok(());};
        if ticket.provider!=provider||ticket.cancel.load(Ordering::Acquire){return Err("The CLI request was cancelled.".into());}
        let all=self.inner.lock().unwrap();
        if !all.pets.get(provider).is_some_and(|c|pending_matches(c,ticket)||active_matches(c,ticket)){return Err("The CLI request was cancelled.".into());}
        Ok(())
    }

    pub fn conversations(&self,provider:&str,query:&str)->Result<Vec<ConversationSummary>,String> {
        if self.disk_error {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        if !crate::assistant_memory::supported(provider) {return Err("Este pet não tem biblioteca de conversas.".into());}
        let mut all=self.inner.lock().unwrap();let query=query.chars().take(256).collect::<String>().to_lowercase();
        let c=all.pets.entry(provider.into()).or_default();c.identify();let current=c.saved();
        let mut result:Vec<_>=std::iter::once((&current,true)).chain(all.archives.get(provider).into_iter().flatten().map(|c|(c,false)))
            .filter(|(c,_)|query.is_empty() || c.messages.iter().any(|m|m["content"].as_str().is_some_and(|s|s.to_lowercase().contains(&query))))
            .map(|(c,active)|summary(provider,c,active)).collect();
        result.sort_by_key(|c|(std::cmp::Reverse(c.current),std::cmp::Reverse(c.updated)));Ok(result)
    }

    fn archive(all:&mut Conversations,provider:&str)->Result<(),String> {
        if let Some(c)=all.pets.get(provider).filter(|c|!c.plain.is_empty() || c.session.is_some()) {
            let mut saved=c.saved();saved.tasks.clear();
            let archives=all.archives.entry(provider.into()).or_default();
            if archives.len()>=crate::assistant_memory::MAX_ARCHIVES {return Err("A biblioteca deste pet atingiu 100 conversas arquivadas. A conversa atual foi preservada.".into());}
            archives.push(saved);
        }
        Ok(())
    }

    pub fn restore_conversation(&self,provider:&str,id:&str)->Result<ConversationView,String> {
        if !crate::assistant_memory::supported(provider) || !crate::assistant_memory::valid_id(id) {return Err("Conversa inválida para este pet.".into());}
        let mut all=self.inner.lock().unwrap();
        self.flush_pending(&mut all)?;
        if all.pets.get(provider).is_some_and(Conversation::busy) {return Err(BUSY_ERROR.into());}
        if let Some(c)=all.pets.get(provider).filter(|c|c.id==id) {return Ok(c.view());}
        let mut next=all.clone();
        let saved=next.archives.get_mut(provider).and_then(|list|list.iter().position(|c|c.id==id).map(|at|list.remove(at))).ok_or("A conversa não pertence a este pet ou já não existe.")?;
        Self::archive(&mut next,provider)?;
        let tasks=next.pets.get_mut(provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
        let mut c=Conversation::from_saved(saved);c.tasks=tasks;next.next+=1;c.epoch=next.next;
        let view=c.view();next.pets.insert(provider.into(),c);
        self.persist_checked(&next)?;*all=next;Ok(view)
    }

    /// Public origin for the Mochi channel; never inferred from an external hook.
    pub fn public_origin(&self,provider:&str)->String {
        let all=self.inner.lock().unwrap();
        Self::task_origin(all.pets.get(provider).and_then(|c|c.tasks.last()))
    }
    fn task_origin(task:Option<&crate::assistant_tasks::Task>)->String{
        let Some(task)=task else{return "Mochi".into()};
        let title=task.query.lines().next().unwrap_or("").chars().take(64).collect::<String>();
        let id=task.session_id.as_deref().map(|s|format!(" · #{}",s.chars().take(8).collect::<String>())).unwrap_or_default();
        format!("Mochi · {title}{id} · {}",task.cwd)
    }

    pub fn tasks(&self)->Vec<crate::assistant_tasks::Task> {
        let all=self.inner.lock().unwrap();let mut tasks:Vec<_>=all.pets.values().flat_map(|c|c.tasks.clone()).collect();tasks.sort_by_key(|t|std::cmp::Reverse(t.updated));tasks
    }

    /// Only an exact, currently owned CLI session can redirect a hook to a pet.
    /// A different external CLI conversation keeps its own identity and controls.
    pub fn hook_owner(&self,backend:&str,session:&str)->Option<String> {
        self.hook_turn(backend,session).map(|t|t.provider)
    }
    fn hook_turn(&self,backend:&str,session:&str)->Option<Turn> {
        if session.is_empty() {return None;}
        self.inner.lock().unwrap().pets.iter().find_map(|(provider,c)| {
            (c.active.is_some() &&
                c.tasks.last().is_some_and(|t|!t.terminal() && t.backend==backend && t.session_id.as_deref()==Some(session)))
                .then(||Turn{epoch:c.epoch,provider:provider.clone(),first:false,history:Vec::new(),session:c.session.clone(),cancel:c.active.as_ref().unwrap().clone()})
        })
    }
    pub fn hook_activity(&self,backend:&str,session:&str,phase:&str)->bool {
        let Some(turn)=self.hook_turn(backend,session).filter(|t|!t.cancel.load(Ordering::Relaxed)) else{return false};
        self.task_update(&turn,phase,"");true
    }

    pub fn start_task(&self,turn:&Turn,query:&str,cwd:&str,backend:&str,model:&str,resume_from:Option<&str>)->Result<(),String> {
        let mut all=self.inner.lock().unwrap();let mut next=all.clone();
        let c=next.pets.get_mut(&turn.provider).filter(|c|c.epoch==turn.epoch)
            .ok_or("O pedido já não pertence ao turno ativo desta conversa.")?;
        c.identify();
        let mut task=crate::assistant_tasks::Task::new(&turn.provider,query,cwd,backend,model,turn.epoch);
        // Record the stable public dialogue before any native startup. Native
        // session identity is confirmed separately by remember_session.
        task.conversation_id=Some(c.id.clone());
        if let Some(id)=resume_from {
            let parent=c.tasks.iter_mut().find(|t|t.id==id && matches!(t.phase.as_str(),"interrupted"|"stopped"|"failed"))
                .ok_or("A tarefa original já não está disponível para retomar.")?;
            task.query=parent.query.clone();task.resumes=Some(parent.id.clone());parent.resumed_by=Some(task.id.clone());parent.update("resumed","");
        }
        c.tasks.push(task);
        if c.tasks.len()>10 {c.tasks.drain(..c.tasks.len()-10);}
        c.task_saved=Some(std::time::Instant::now());
        self.persist_checked(&next)?;*all=next;Ok(())
    }

    pub fn task_update(&self,turn:&Turn,phase:&str,message:&str)->bool {
        let mut all=self.inner.lock().unwrap();
        let Some(c)=all.pets.get_mut(&turn.provider).filter(|c|c.epoch==turn.epoch) else{return false};
        let flush=c.task_saved.is_none_or(|at|at.elapsed()>=std::time::Duration::from_secs(1));
        let Some(task)=c.tasks.last_mut() else{return false};
        if task.terminal() || (turn.cancel.load(Ordering::Relaxed)&&!matches!(phase,"stopping"|"stopped"|"failed")){return false;}
        let changed=task.phase!=phase;task.update(phase,message);
        let urgent=task.terminal() || matches!(phase,"approval"|"question");
        if flush || urgent {c.task_saved=Some(std::time::Instant::now());self.persist(&all);}
        changed || flush || urgent
    }

    pub fn cancel_task(&self,id:&str)->bool {
        let mut all=self.inner.lock().unwrap();
        for c in all.pets.values_mut() {
            if let Some(task)=c.tasks.last_mut().filter(|t|t.id==id && !t.terminal()) {
                if let Some(active)=&c.active {active.store(true,Ordering::Relaxed);task.update("stopping","");self.persist(&all);return true;}
            }
        }
        false
    }

    pub fn reset(&self) {
        if self.disk.is_some(){if self.reset_checked().is_err(){crate::log::line("assistant memory: reset rejected; dialogue preserved");}return;}
        let mut all=self.inner.lock().unwrap();
        if self.flush_pending(&mut all).is_err(){return;}
        for c in all.pets.values() {
            if let Some(active)=&c.active {active.store(true,Ordering::Release);}
            if let Some(pending)=&c.pending {pending.cancel.store(true,Ordering::Release);}
        }
        all.pets.clear();self.persist(&all);
    }

    /// A reset of the persisted library archives every current dialogue and
    /// preserves each pet's task journal, just like a provider-specific reset.
    pub fn reset_checked(&self)->Result<(),String>{
        let mut all=self.inner.lock().unwrap();self.flush_pending(&mut all)?;
        if all.pets.values().any(Conversation::busy){return Err(BUSY_ERROR.into());}
        let mut next=all.clone();let providers:Vec<_>=next.pets.keys().cloned().collect();
        for provider in providers{
            if !crate::assistant_memory::supported(&provider){continue;}
            Self::archive(&mut next,&provider)?;
            let tasks=next.pets.get_mut(&provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
            next.next+=1;let mut c=Conversation{epoch:next.next,tasks,..Default::default()};c.identify();next.pets.insert(provider,c);
        }
        self.persist_checked(&next)?;*all=next;Ok(())
    }

    pub fn begin(&self,provider:&str)->Turn {self.begin_with_cancel(provider,Arc::new(AtomicBool::new(false)))}

    pub fn begin_with_cancel(&self,provider:&str,cancel:Arc<AtomicBool>)->Turn {
        self.start(provider,cancel,false).unwrap()
    }

    pub fn begin_cli(&self,provider:&str,cancel:Arc<AtomicBool>)->Result<Turn,String> {self.start(provider,cancel,true)}
    pub fn lease(&self,turn:&Turn)->TurnLease {TurnLease{chat:self.clone(),turn:turn.clone(),app:None}}

    pub fn publish_delta(&self,app:&AppHandle,turn:&Turn,text:&str) {
        let mut all=self.inner.lock().unwrap();
        let Some(c)=all.pets.get_mut(&turn.provider).filter(|c|c.epoch==turn.epoch && c.active.is_some() && !turn.cancel.load(Ordering::Relaxed)) else{return};
        c.live_text=text.chars().take(128*1024).collect();
        if c.live_sent.is_some_and(|at|at.elapsed()<std::time::Duration::from_millis(65)){return;}
        c.live_sent=Some(std::time::Instant::now());
        let event=json!({"provider":turn.provider,"conversationId":c.id,"turn":c.epoch,"text":c.live_text});
        drop(all);let _=app.emit("chat-delta",event);
    }

    pub fn cancel_conversation(&self,provider:&str,id:Option<&str>,turn:Option<u64>)->bool {
        let(Some(id),Some(turn))=(id,turn)else{return false;};
        let all=self.inner.lock().unwrap();
        let Some(c)=all.pets.get(provider).filter(|c|c.id==id&&c.epoch==turn) else{return false};
        if let Some(ticket)=&c.pending {ticket.cancel.store(true,Ordering::Release);true}
        else if let Some(cancel)=&c.active {cancel.store(true,Ordering::Release);true}else{false}
    }

    fn start(&self,provider:&str,cancel:Arc<AtomicBool>,exclusive:bool)->Result<Turn,String> {
        if self.disk_error && exclusive {return Err("A biblioteca local não pôde ser lida. O ficheiro original foi preservado para recuperação.".into());}
        let mut all=self.inner.lock().unwrap();
        if let Some(ticket)=&self.admission {
            // Validate before creating entries, incrementing epochs, flushing
            // history, or touching a later pending/active turn.
            if ticket.provider!=provider||ticket.cancel.load(Ordering::Acquire)
                ||cancel.load(Ordering::Acquire)
                ||!all.pets.get(provider).is_some_and(|c|pending_matches(c,ticket)&&c.active.is_none()){
                return Err("The CLI request was cancelled.".into());
            }
            self.flush_pending(&mut all)?;
            let c=all.pets.get_mut(provider).unwrap();
            c.pending=None;c.active=Some(ticket.cancel.clone());c.turn_cancel=Some(ticket.cancel.clone());
            c.live_text.clear();c.live_sent=None;
            return Ok(Turn{epoch:ticket.epoch,provider:provider.into(),first:c.plain.is_empty(),history:c.native.clone(),session:c.session.clone(),cancel:ticket.cancel.clone()});
        }
        if all.pets.get(provider).is_some_and(|c|c.pending.is_some()||(exclusive&&c.active.is_some())){return Err(BUSY_ERROR.into());}
        if exclusive {self.flush_pending(&mut all)?;}
        all.next+=1;let epoch=all.next;
        let c=all.pets.entry(provider.into()).or_default();
        c.identify();
        c.epoch=epoch;
        c.live_text.clear();c.live_query.clear();c.live_sent=None;
        if let Some(previous)=c.active.replace(cancel.clone()) {if !Arc::ptr_eq(&previous,&cancel) {previous.store(true,Ordering::Relaxed);}}
        c.turn_cancel=Some(cancel.clone());
        Ok(Turn{epoch,provider:provider.into(),first:c.plain.is_empty(),history:c.native.clone(),session:c.session.clone(),cancel})
    }

    pub fn reset_provider(&self,provider:&str)->Result<(),String> {
        let mut all=self.inner.lock().unwrap();
        self.flush_pending(&mut all)?;
        if crate::assistant_memory::supported(provider) {
            // Starting a fresh dialogue must never erase the task journal or
            // secretly stop voice/recovery work that the text view cannot see.
            if all.pets.get(provider).is_some_and(Conversation::busy) {return Err(BUSY_ERROR.into());}
            let mut next=all.clone();Self::archive(&mut next,provider)?;
            let tasks=next.pets.get_mut(provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
            next.next+=1;let mut c=Conversation{epoch:next.next,tasks,..Default::default()};c.identify();
            next.pets.insert(provider.into(),c);self.persist_checked(&next)?;*all=next;return Ok(());
        } else if let Some(c)=all.pets.remove(provider) {
            if let Some(active)=c.active {active.store(true,Ordering::Release);}
            if let Some(pending)=c.pending {pending.cancel.store(true,Ordering::Release);}
        }
        self.persist(&all);
        Ok(())
    }

    pub fn is_current(&self,turn:&Turn)->bool {
        !turn.cancel.load(Ordering::Acquire) && self.inner.lock().unwrap().pets.get(&turn.provider)
            .is_some_and(|c|c.epoch==turn.epoch&&c.turn_cancel.as_ref().is_some_and(|cancel|Arc::ptr_eq(cancel,&turn.cancel)))
    }
    pub(crate) fn usage_origin(&self,turn:&Turn)->Option<(String,u64)>{
        if turn.cancel.load(Ordering::Acquire){return None;}
        self.inner.lock().unwrap().pets.get(&turn.provider)
            .filter(|c|c.epoch==turn.epoch&&c.turn_cancel.as_ref().is_some_and(|flag|Arc::ptr_eq(flag,&turn.cancel)))
            .map(|c|(c.id.clone(),turn.epoch))
    }

    pub fn cancel_provider(&self,provider:&str) {
        if let Some(c)=self.inner.lock().unwrap().pets.get(provider){
            if let Some(active)=&c.active {active.store(true,Ordering::Release);}
            if let Some(pending)=&c.pending {pending.cancel.store(true,Ordering::Release);}
        }
    }

    pub fn cancel(&self) {
        for c in self.inner.lock().unwrap().pets.values() {
            if let Some(active)=&c.active {active.store(true,Ordering::Release);}
            if let Some(pending)=&c.pending {pending.cancel.store(true,Ordering::Release);}
        }
    }

    pub fn remember_session(&self,turn:&Turn,id:&str,cwd:&str,full_access:bool) {
        let mut all=self.inner.lock().unwrap();
        if let Some(c)=all.pets.get_mut(&turn.provider).filter(|c|c.epoch==turn.epoch && !turn.cancel.load(Ordering::Relaxed)) {
            c.session=Some(CliSession{id:id.into(),cwd:cwd.into(),full_access});
            if let Some(task)=c.tasks.last_mut(){task.session_id=Some(id.into());}
            self.persist(&all);
        }
    }

    /// Only successful public turns are saved. Native tool blocks, audio,
    /// reasoning and permission responses never enter the local memory file.
    pub fn commit(&self,turn:&Turn,user:Value,assistant:Value,user_text:&str,answer:&str)->Result<(),String> {
        let(_,notice)=self.commit_outcome(turn,user,assistant,user_text,answer,false)?;
        match notice{Some(error)=>Err(error),None=>Ok(())}
    }

    fn commit_outcome(&self,turn:&Turn,user:Value,assistant:Value,user_text:&str,answer:&str,finish_task:bool)->Result<(String,Option<String>),String>{
        let mut all=self.inner.lock().unwrap();
        let Some(c)=all.pets.get_mut(&turn.provider).filter(|c|c.epoch==turn.epoch && !turn.cancel.load(Ordering::Relaxed)) else{return Err("Esta resposta já não pertence ao turno ativo da conversa.".into());};
        if finish_task{
            if let Some(task)=c.tasks.last_mut(){task.update("done",answer);}
            c.task_saved=Some(std::time::Instant::now());
        }
        let origin=Self::task_origin(c.tasks.last());
        c.native.extend([user,assistant]);
        c.plain.extend([json!({"role":"user","content":user_text}),json!({"role":"assistant","content":answer})]);
        c.updated=crate::assistant_memory::now();
        c.native=crate::assistant_memory::context(&c.native);
        c.active=None;
        match self.persist_checked(&all){
            Ok(())=>{for c in all.pets.values_mut(){c.save_error=None;}Ok((origin,None))},
            Err(error)=>{
                let notice=format!("{error} A resposta completa permanece nesta janela, mas ainda não foi guardada. Corrige o problema antes de fechar a aplicação; o próximo envio tentará guardar novamente.");
                if let Some(c)=all.pets.get_mut(&turn.provider){c.save_error=Some(notice.clone());}
                Ok((origin,Some(notice)))
            }
        }
    }

    /// An executed turn remains successful when only persistence fails. Its
    /// complete public reply and warning are already retained in the snapshot;
    /// reporting a send failure could encourage repeating an executed action.
    pub fn commit_reply(&self,turn:&Turn,user:Value,assistant:Value,user_text:&str,answer:&str)->Result<(),String> {
        self.commit_outcome(turn,user,assistant,user_text,answer,false).map(|_|())
    }
    /// Cancellation and successful completion share the journal lock. A stop
    /// accepted first cannot subsequently become "done" with a missing reply.
    pub fn commit_task_reply(&self,turn:&Turn,user:Value,assistant:Value,user_text:&str,answer:&str)->Result<String,String>{
        self.commit_outcome(turn,user,assistant,user_text,answer,true).map(|(origin,_)|origin)
    }
}

// ── System prompt ─────────────────────────────────────────────────────────────

/// Mochi's instructions. Greets the user by their first name when the account
/// has one worth using (identity.rs), and only claims web search where the
/// provider runs it (Claude).
pub fn system_prompt(web_search: bool) -> String {
    system_prompt_for(crate::identity::first_name(), web_search)
}

fn system_prompt_for(first_name: Option<&str>, web_search: bool) -> String {
    let opening = match first_name {
        Some(name) => format!("You are Mochi, {name}'s personal AI assistant living at the top of their screen."),
        None => "You are Mochi, a personal AI assistant living at the top of the user's screen.".to_string(),
    };
    let abilities = if web_search {
        "You have web search access and can help with absolutely anything — research, coding, finding places, recommendations, tasks, questions."
    } else {
        "You can help with absolutely anything — research, coding, recommendations, tasks, questions. You have no web access: say so when something needs current information."
    };
    format!(
        "{opening} {abilities} \
Respond in the user's language. Be thorough and complete — use as much detail as the task requires. \
Use light Markdown when it helps: short paragraphs, bullet lists, **bold**, `inline code` and fenced code blocks. Avoid tables and big headings: the chat window is small."
    )
}

/// The window context line, as ClaudeService.chat() writes it.
pub fn window_line(app_name: &str, title: &str, url: Option<&str>) -> String {
    let mut text = format!("Context — App: {app_name}, Window: {title}");
    if let Some(url) = url {
        text.push_str(&format!(", URL: {url}"));
    }
    text
}

/// The plain-text record of what the user asked, context included.
pub fn plain_question(first: bool, context: Option<&ChatContext>, query: &str) -> String {
    match context {
        Some(ChatContext::File { name, .. }) => format!("File: {name}\n\n{query}"),
        Some(ChatContext::Window { app_name, title, url }) if first => {
            format!("{}\n\n{query}", window_line(app_name, title, url.as_deref()))
        }
        _ => query.to_string(),
    }
}

// ── Which provider ────────────────────────────────────────────────────────────

/// The model chosen for `provider`, or its default.
pub(crate) fn select_for_turn(settings:&mut Settings,model:String)->Result<(),String>{
    if model.is_empty()||model.len()>512||model.chars().any(char::is_control){return Err("Modelo inválido para esta mensagem.".into());}
    if settings.chat_provider=="ollama"&&model!=model_for(settings,"ollama"){
        return Err("O modelo Ollama mudou noutra janela. Volta a selecionar o modelo antes de enviar; o pedido ainda não foi iniciado.".into());
    }
    settings.chat_models.insert(settings.chat_provider.clone(),model);Ok(())
}

pub fn model_for(settings: &Settings, provider: &str) -> String {
    if provider == ANTHROPIC {
        let m = settings.model.trim();
        return if m.is_empty() { claude::DEFAULT_MODEL.to_string() } else { m.to_string() };
    }
    settings
        .chat_models
        .get(provider)
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .or_else(|| openai_compat::provider(provider).map(|p| p.default_model.to_string()))
        .unwrap_or_else(||if crate::pet_config::pet_id(provider).is_some(){"@default".into()}else{String::new()})
}

/// A file rides along only if it is one of Coucou's own copies of a dropped
/// file (files.rs puts them in the inbox). The page names the path, so without
/// this any file the user can read could be sent to a chat provider.
fn checked_context(context: ChatContext) -> Result<ChatContext, String> {
    match context {
        ChatContext::File { name, path } => {
            let inbox = crate::files::inbox_dir();
            if !is_inside(&inbox, std::path::Path::new(&path)) {
                return Err(crate::i18n::t("Only a file dropped on the island can be sent with a question."));
            }
            Ok(ChatContext::File { name, path })
        }
        other => Ok(other),
    }
}

/// True when `path` is a regular file directly inside `dir`, both resolved
/// (no `..`, no symlink pointing out of it).
fn is_inside(dir: &std::path::Path, path: &std::path::Path) -> bool {
    let (Ok(dir), Ok(file)) = (dir.canonicalize(), path.canonicalize()) else { return false };
    file.parent() == Some(dir.as_path())
        && std::fs::symlink_metadata(&file).map(|m| m.is_file() || m.is_dir()).unwrap_or(false)
}

/// One chat turn with the provider chosen in the settings.
pub async fn send(
    app: &AppHandle,
    chat: &Chat,
    settings: &Settings,
    query: String,
    context: Option<ChatContext>,
) -> Result<ChatReply, String> {
    let provider = if settings.chat_provider.is_empty(){ANTHROPIC}else{settings.chat_provider.as_str()};
    let reservation=chat.bind_reservation(provider)?.broadcast(app,&query);
    let channel=reservation.channel();let chat=&channel;
    chat.check_admission(provider)?;
    if let Some(ChatContext::File{path,..})=&context{crate::pet_files::check(provider,path)?;}
    let context = context.map(checked_context).transpose()?;
    let model = model_for(settings, provider);
    if crate::cli_chat::backend(provider).is_some() {
        return crate::cli_chat::send(app, chat, settings, provider, &model, query, context).await;
    }
    if provider == ANTHROPIC || provider.is_empty() {
        return claude::send(chat, &model, query, context).await;
    }
    if let Some(p) = openai_compat::provider(provider) {
        return openai_compat::send(chat, p, &model, query, context).await;
    }
    if let Some(server) = local_chat::server(settings, provider) {
        let model=if provider=="ollama" {crate::pet_config::select(provider,&model,&query)?.model}else{model};
        return local_chat::send(app, chat, &server, &model, query, context).await;
    }
    Err(format!("Unknown chat provider: {provider}"))
}

/// One turn of the phone's conversation (phone_link/chat_backend.rs): the same providers as
/// [`send`], with no file or window context and no `AppHandle`. `on_delta` gets the streamed
/// text so far, which only the model servers produce; the cloud providers answer all at once.
pub async fn send_for_phone(
    settings: &Settings,
    chat: &Chat,
    provider: &str,
    model: &str,
    query: String,
    on_delta: impl FnMut(String),
) -> Result<ChatReply, String> {
    if provider == ANTHROPIC {
        return claude::send(chat, model, query, None).await;
    }
    if let Some(p) = openai_compat::provider(provider) {
        return openai_compat::send(chat, p, model, query, None).await;
    }
    if let Some(server) = local_chat::server(settings, provider) {
        return local_chat::send_with(chat, &server, model, query, None, on_delta).await;
    }
    Err(format!("Unknown chat provider: {provider}"))
}

/// The models `provider` offers. Asked only when the user opens the picker on
/// that provider, and only once it has a key (or, for a local server, an
/// address): nothing is sent anywhere before that.
pub async fn models(settings: &Settings, provider: &str) -> Result<Vec<ModelInfo>, String> {
    if provider=="copilot" {return crate::copilot_adapter::models().await;}
    if crate::cli_chat::backend(provider)==Some("codex") { return crate::codex_chat::models().await; }
    if matches!(provider,"antigravity"|"gemini-cli"|"claude-cli") {
        let models=crate::gravity_chat::models().await?;
        return Ok(models.into_iter().filter(|m|crate::cli_chat::model_matches(provider,&m.id)).collect());
    }
    let no_key = || crate::i18n::t("No API key — add it in Settings.");
    if provider == ANTHROPIC {
        let key = secrets::get(claude::KEY).ok_or_else(no_key)?;
        return claude::models(&key).await;
    }
    if let Some(p) = openai_compat::provider(provider) {
        let key = secrets::get(p.key).ok_or_else(no_key)?;
        return openai_compat::models(p, &key).await;
    }
    if let Some(server) = local_chat::server(settings, provider) {
        return local_chat::models(&server).await;
    }
    Err(format!("Unknown chat provider: {provider}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_identity_snapshot_does_not_wait_for_a_locked_archive(){
        let chat=Chat::default();let locked=chat.inner.lock().unwrap();let other=chat.clone();
        let(tx,rx)=std::sync::mpsc::channel();let task=std::thread::spawn(move||{let result=other.try_conversation_ids(&["ollama".into()]);tx.send(result).unwrap();});
        let result=rx.recv_timeout(std::time::Duration::from_millis(100));drop(locked);task.join().unwrap();
        assert!(result.unwrap().is_err());let ids=chat.try_conversation_ids(&["ollama".into(),"codex".into()]).unwrap();
        assert_eq!(ids["ollama"],chat.conversation_id("ollama").unwrap());assert_ne!(ids["ollama"],ids["codex"]);
    }

    #[test]
    fn a_stale_ollama_picker_cannot_override_a_confirmed_selection(){
        let mut settings=Settings::default();settings.chat_provider="ollama".into();
        settings.chat_models.insert("ollama".into(),"confirmed:b".into());
        settings.chat_models.insert("codex".into(),"codex:unchanged".into());
        assert!(select_for_turn(&mut settings,"cached:a".into()).is_err());
        assert_eq!(settings.chat_models["ollama"],"confirmed:b");assert_eq!(settings.chat_models["codex"],"codex:unchanged");
        assert!(select_for_turn(&mut settings,"confirmed:b".into()).is_ok());
    }
    #[test]
    fn an_ollama_profile_ack_keeps_its_token_for_prompt_based_selection(){
        let mut settings=Settings::default();settings.chat_provider="ollama".into();
        settings.chat_models.insert("ollama".into(),"@auto".into());
        assert!(select_for_turn(&mut settings,"local:fast".into()).is_err());
        assert!(select_for_turn(&mut settings,"@auto".into()).is_ok());assert_eq!(model_for(&settings,"ollama"),"@auto");
    }
    #[test]
    fn other_providers_keep_an_explicit_session_model_without_changing_ollama(){
        let mut settings=Settings::default();settings.chat_provider="codex".into();
        settings.chat_models.insert("ollama".into(),"local:unchanged".into());
        assert!(select_for_turn(&mut settings,"session:chosen".into()).is_ok());
        assert_eq!(model_for(&settings,"codex"),"session:chosen");assert_eq!(model_for(&settings,"ollama"),"local:unchanged");
        assert!(select_for_turn(&mut settings,"\ninvalid".into()).is_err());assert_eq!(model_for(&settings,"codex"),"session:chosen");
    }

    #[test]
    fn capturing_a_conversation_id_preserves_the_owner_and_live_turn() {
        let chat=Chat::default();
        assert!(chat.conversation_id("unknown").is_err());
        let id=chat.conversation_id("codex").unwrap();
        assert_eq!(chat.conversation("codex").unwrap().id,id);
        let reservation=chat.reserve_conversation("codex",Some(&id)).unwrap();
        let queued=chat.conversation("codex").unwrap();
        assert_eq!(chat.conversation_id("codex").unwrap(),id);
        let after=chat.conversation("codex").unwrap();
        assert!(after.busy);assert_eq!(after.turn,queued.turn);
        drop(reservation);chat.reset_provider("codex").unwrap();
        assert_ne!(chat.conversation_id("codex").unwrap(),id);
    }

    #[test]
    fn reservation_publishes_busy_and_the_epoch_before_starting_any_worker() {
        let chat=Chat::default();let initial=chat.conversation("codex").unwrap();
        let reservation=chat.reserve_conversation("codex",Some(&initial.id)).unwrap();
        let queued=chat.conversation("codex").unwrap();
        assert!(queued.busy);assert!(queued.turn>initial.turn);assert_eq!(queued.id,initial.id);
        assert!(chat.inner.lock().unwrap().pets["codex"].active.is_none());
        assert!(chat.reset_provider("codex").is_err());assert!(chat.reset_checked().is_err());
        drop(reservation);assert!(!chat.conversation("codex").unwrap().busy);
    }

    #[test]
    fn an_exact_queued_stop_prevents_the_worker_from_starting() {
        let chat=Chat::default();let reservation=chat.reserve_conversation("codex",None).unwrap();
        let channel=reservation.channel();let queued=chat.conversation("codex").unwrap();
        assert!(!chat.cancel_conversation("codex",None,None));
        assert!(!chat.cancel_conversation("codex",Some(&queued.id),None));
        assert!(!chat.cancel_conversation("codex",None,Some(queued.turn)));
        assert!(!chat.cancel_conversation("codex",Some(&queued.id),Some(queued.turn+1)));
        assert!(channel.check_admission("codex").is_ok());
        assert!(chat.cancel_conversation("codex",Some(&queued.id),Some(queued.turn)));
        assert_eq!(channel.check_admission("codex").unwrap_err(),"The CLI request was cancelled.");
        assert!(channel.begin_cli("codex",Arc::default()).is_err());
        assert_eq!(chat.inner.lock().unwrap().next,queued.turn);
        drop(reservation);assert!(!chat.conversation("codex").unwrap().busy);
        assert!(channel.begin_cli("codex",Arc::default()).is_err());
        assert!(chat.inner.lock().unwrap().pets["codex"].active.is_none());
    }

    #[test]
    fn voice_cannot_consume_a_manual_reservation_and_the_scope_inherits_its_token() {
        let chat=Chat::default();let reservation=chat.reserve_conversation("ollama",None).unwrap();
        let queued=chat.conversation("ollama").unwrap();let channel=reservation.channel();
        assert!(chat.begin_cli("ollama",Arc::default()).is_err());
        assert_eq!(chat.conversation("ollama").unwrap().turn,queued.turn);
        let turn=channel.begin_cli("ollama",Arc::default()).unwrap();let lease=channel.lease(&turn);
        assert_eq!(turn.epoch,queued.turn);assert!(Arc::ptr_eq(&turn.cancel,&reservation.ticket.cancel));
        assert!(channel.check_admission("ollama").is_ok());assert!(chat.begin_cli("ollama",Arc::default()).is_err());
        drop(reservation);assert!(!turn.cancel.load(Ordering::Acquire));assert!(chat.is_current(&turn));
        drop(lease);assert!(!chat.conversation("ollama").unwrap().busy);assert!(channel.check_admission("ollama").is_err());
    }

    #[test]
    fn an_abandoned_scope_and_its_old_drop_or_stop_cannot_touch_a_new_reservation() {
        let chat=Chat::default();let original=chat.reserve_conversation("codex",None).unwrap();
        let bound=chat.bind_reservation("codex").unwrap();let old=bound.channel();
        assert!(chat.bind_reservation("codex").is_err());
        let previous=chat.conversation("codex").unwrap();drop(original);
        let current=chat.reserve_conversation("codex",None).unwrap();let queued=chat.conversation("codex").unwrap();
        assert!(queued.turn>previous.turn);drop(bound);
        assert!(!current.ticket.cancel.load(Ordering::Acquire));
        assert!(!chat.cancel_conversation("codex",Some(&previous.id),Some(previous.turn)));
        let before={let all=chat.inner.lock().unwrap();(all.next,all.pets.len())};
        assert!(old.begin_cli("codex",Arc::default()).is_err());assert!(old.begin_cli("other-provider",Arc::default()).is_err());
        assert_eq!({let all=chat.inner.lock().unwrap();(all.next,all.pets.len())},before);
        let channel=current.channel();let turn=channel.begin_cli("codex",Arc::default()).unwrap();
        let lease=channel.lease(&turn);assert_eq!(turn.epoch,queued.turn);drop(current);assert!(chat.is_current(&turn));drop(lease);
    }

    #[test]
    fn old_turn_leases_cannot_clear_a_later_pending_or_active_turn() {
        let chat=Chat::default();let original=chat.reserve_conversation("codex",None).unwrap();let channel=original.channel();
        let turn=channel.begin_cli("codex",Arc::default()).unwrap();let old_pending=channel.lease(&turn);let old_active=channel.lease(&turn);
        channel.commit_reply(&turn,json!({"role":"user"}),json!({"role":"assistant"}),"old","complete").unwrap();drop(original);
        let current=chat.reserve_conversation("codex",None).unwrap();let queued=chat.conversation("codex").unwrap();drop(old_pending);
        assert!(chat.conversation("codex").unwrap().busy);assert!(!current.ticket.cancel.load(Ordering::Acquire));
        let channel=current.channel();let turn=channel.begin_cli("codex",Arc::default()).unwrap();let lease=channel.lease(&turn);drop(old_active);
        assert_eq!(chat.conversation("codex").unwrap().turn,queued.turn);assert!(chat.is_current(&turn));
        assert!(chat.conversation("codex").unwrap().busy);drop(lease);drop(current);
    }

    #[test]
    fn concurrent_scoped_workers_can_consume_the_admission_only_once() {
        let chat=Chat::default();let reservation=chat.reserve_conversation("codex",None).unwrap();
        let epoch=chat.conversation("codex").unwrap().turn;let gate=Arc::new(std::sync::Barrier::new(2));
        let workers:Vec<_>=(0..2).map(|_|{let channel=reservation.channel();let gate=gate.clone();
            std::thread::spawn(move||{gate.wait();channel.begin_cli("codex",Arc::default())})}).collect();
        let turns:Vec<_>=workers.into_iter().map(|worker|worker.join().unwrap()).filter_map(Result::ok).collect();
        assert_eq!(turns.len(),1);assert_eq!(turns[0].epoch,epoch);assert_eq!(chat.inner.lock().unwrap().next,epoch);
        assert!(chat.is_current(&turns[0]));drop(chat.lease(&turns[0]));drop(reservation);
    }

    #[test]
    fn broad_explicit_cancellation_also_marks_pending_tickets() {
        let chat=Chat::default();let codex=chat.reserve_conversation("codex",None).unwrap();let local=chat.reserve_conversation("ollama",None).unwrap();
        chat.cancel_provider("codex");assert!(codex.ticket.cancel.load(Ordering::Acquire));assert!(!local.ticket.cancel.load(Ordering::Acquire));
        assert!(local.channel().check_admission("ollama").is_ok());chat.cancel();assert!(local.ticket.cancel.load(Ordering::Acquire));
        assert!(local.channel().begin_cli("ollama",Arc::default()).is_err());drop(codex);drop(local);
    }

    #[test]
    fn a_pending_reservation_prevents_restoring_an_archive() {
        let chat=Chat::default();let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);
        chat.commit_reply(&turn,json!({"role":"user"}),json!({"role":"assistant"}),"old","answer").unwrap();drop(lease);
        let archive=chat.conversation("codex").unwrap().id;chat.reset_provider("codex").unwrap();
        let reservation=chat.reserve_conversation("codex",None).unwrap();let queued=chat.conversation("codex").unwrap();
        assert!(chat.restore_conversation("codex",&archive).is_err());assert!(chat.reset_provider("codex").is_err());
        assert_eq!(chat.conversation("codex").unwrap().id,queued.id);assert_eq!(chat.read_conversation("codex",&archive).unwrap().messages.len(),2);
        drop(reservation);chat.restore_conversation("codex",&archive).unwrap();
    }

    #[test]
    fn service_work_owns_its_session_and_never_reuses_another_pet_or_external_cli_context() {
        let chat=Chat::default();
        let codex=chat.begin_cli("codex",Arc::default()).unwrap();let _codex=chat.lease(&codex);
        chat.start_task(&codex,"query","/tmp","codex","fast",None).unwrap();chat.remember_session(&codex,"codex-own","/tmp",false);
        let service=chat.begin_cli("integration_spotify",Arc::default()).unwrap();let _service=chat.lease(&service);
        assert!(service.first && service.history.is_empty() && service.session.is_none());
        chat.start_task(&service,"play","/tmp","codex","fast",None).unwrap();chat.remember_session(&service,"spotify-own","/tmp",false);
        assert_eq!(chat.hook_owner("codex","spotify-own").as_deref(),Some("integration_spotify"));
        assert_eq!(chat.hook_owner("codex","codex-own").as_deref(),Some("codex"));assert!(chat.hook_owner("codex","external").is_none());
        assert!(chat.cancel_task(&chat.tasks().into_iter().find(|t|t.provider=="integration_spotify").unwrap().id));
        assert!(!codex.cancel.load(Ordering::Relaxed));assert!(service.cancel.load(Ordering::Relaxed));
    }

    #[test]
    fn cancelled_job_keeps_its_owner_until_the_process_has_released_it() {
        let chat=Chat::default();let first=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&first);
        chat.cancel_provider("codex");assert!(chat.begin_cli("codex",Arc::default()).is_err());
        drop(lease);assert!(chat.begin_cli("codex",Arc::default()).is_ok());
    }

    #[test]
    fn fresh_dialogue_never_cancels_or_detaches_an_active_voice_job() {
        let chat=Chat::default();let mic=Arc::new(AtomicBool::new(false));
        let turn=chat.begin_cli("codex",mic.clone()).unwrap();let _lease=chat.lease(&turn);
        chat.start_task(&turn,"prepara postgres","/tmp","codex","model",None).unwrap();
        chat.remember_session(&turn,"owned-voice","/tmp",false);
        let id=chat.tasks()[0].id.clone();
        assert_eq!(chat.reset_provider("codex").unwrap_err(),BUSY_ERROR);
        assert!(chat.is_current(&turn));assert!(!mic.load(Ordering::Relaxed));
        assert_eq!(chat.hook_owner("codex","owned-voice").as_deref(),Some("codex"));
        assert_eq!(chat.tasks()[0].id,id);assert!(chat.reset_provider("antigravity").is_ok());
        assert!(chat.is_current(&turn));
    }

    #[test]
    fn fresh_dialogue_preserves_durable_outcomes_and_recoverable_tasks() {
        let dir=std::env::temp_dir().join(format!("coucou-fresh-dialogue-{}",std::process::id()));let path=dir.join("conversations.json");
        let _=std::fs::remove_dir_all(&dir);let chat=Chat::persistent(path.clone());
        for phase in ["done","stopped","failed","interrupted"] {
            let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);
            chat.start_task(&turn,"prepara postgres","/tmp","codex","model",None).unwrap();
            chat.remember_session(&turn,"owned-thread","/tmp",false);
            chat.task_update(&turn,phase,"Estado público da tarefa");
            if phase=="done" {chat.commit(&turn,json!({"role":"user","content":"work"}),json!({"role":"assistant","content":"ready"}),"work","ready");}
            drop(lease);chat.reset_provider("codex").unwrap();
            assert!(chat.history("codex").is_empty());assert!(!chat.is_current(&turn));
            chat.commit(&turn,json!({"role":"user"}),json!({"role":"assistant"}),"late","late");
            assert!(chat.history("codex").is_empty());
        }
        let restored=Chat::persistent(path);let tasks=restored.tasks();assert_eq!(tasks.len(),4);
        assert_eq!(tasks.iter().find(|t|t.phase=="done").unwrap().result,"Estado público da tarefa");
        assert!(tasks.iter().filter(|t|t.phase!="done").all(|t|t.resume_prompt().is_ok()));
        let fresh=restored.begin_cli("codex",Arc::default()).unwrap();assert!(fresh.first && fresh.history.is_empty() && fresh.session.is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hook_owner_requires_the_exact_live_session_and_actual_backend() {
        let chat=Chat::default();let turn=chat.begin_cli("gemini-cli",Arc::default()).unwrap();
        let lease=chat.lease(&turn);chat.start_task(&turn,"build","/tmp","antigravity","gemini-example",None).unwrap();
        chat.remember_session(&turn,"owned-1","/tmp",false);
        assert_eq!(chat.hook_owner("antigravity","owned-1").as_deref(),Some("gemini-cli"));
        assert!(chat.hook_owner("codex","owned-1").is_none());assert!(chat.hook_owner("antigravity","external-2").is_none());
        assert!(chat.hook_activity("antigravity","owned-1","approval"));assert_eq!(chat.tasks()[0].phase,"approval");
        assert!(!chat.hook_activity("antigravity","external-2","approval"));
        chat.cancel_provider("gemini-cli");assert!(!chat.hook_activity("antigravity","owned-1","approval"));
        drop(lease);assert!(chat.hook_owner("antigravity","owned-1").is_none());
    }

    #[test]
    fn consecutive_voice_turns_share_stop_without_cancelling_the_dialogue() {
        let chat = Chat::default();
        let stop = Arc::new(AtomicBool::new(false));
        let first = chat.begin_with_cancel("codex", stop.clone());
        chat.commit(&first, json!({"role":"user","content":"Olá"}), json!({"role":"assistant","content":"Olá"}), "Olá", "Olá");
        let second = chat.begin_with_cancel("codex", stop.clone());
        assert!(!second.first);
        assert!(!stop.load(Ordering::Relaxed));
        assert_eq!(second.history.len(), 2);
        stop.store(true, Ordering::Relaxed);
        assert!(!chat.is_current(&second));
        assert!(chat.begin_with_cancel("codex", stop).cancel.load(Ordering::Relaxed));
    }

    #[test]
    fn different_pets_can_finish_in_parallel_and_cancel_is_scoped() {
        let chat=Chat::default();let a=chat.begin_cli("codex",Arc::default()).unwrap();let b=chat.begin_cli("antigravity",Arc::default()).unwrap();
        assert!(chat.is_current(&a) && chat.is_current(&b));
        assert!(chat.begin_cli("codex",Arc::default()).is_err());assert!(!a.cancel.load(Ordering::Relaxed));
        chat.cancel_provider("codex");assert!(!chat.is_current(&a));assert!(chat.is_current(&b));
        chat.commit(&b,json!({"role":"user","content":"gravity"}),json!({"role":"assistant","content":"done"}),"gravity","done");
        assert_eq!(chat.history("antigravity").len(),2);assert!(chat.history("codex").is_empty());
    }

    #[test]
    fn failed_cli_releases_only_its_own_pet() {
        let chat=Chat::default();let a=chat.begin_cli("codex",Arc::default()).unwrap();let b=chat.begin_cli("antigravity",Arc::default()).unwrap();
        {let _lease=chat.lease(&a);}
        assert!(chat.begin_cli("codex",Arc::default()).is_ok());assert!(chat.is_current(&b));
    }

    #[test]
    fn voice_and_text_keep_cli_context_after_microphone_cycles_and_restart() {
        let dir=std::env::temp_dir().join(format!("coucou-dialogue-{}",std::process::id()));let path=dir.join("conversations.json");
        let _=std::fs::remove_dir_all(&dir);
        let chat=Chat::persistent(path.clone());let mic=Arc::new(AtomicBool::new(false));let voice=chat.begin_cli("codex",mic.clone()).unwrap();
        chat.remember_session(&voice,"owned-codex-thread","/tmp/assistant-project",false);
        chat.commit(&voice,json!({"role":"user","content":"work"}),json!({"role":"assistant","content":"ready"}),"work","ready");
        mic.store(true,Ordering::Relaxed);
        let next=chat.begin_cli("codex",Arc::default()).unwrap();assert!(!next.first);assert_eq!(next.session.unwrap().id,"owned-codex-thread");
        let restored=Chat::persistent(path);assert_eq!(restored.history("codex").len(),2);assert!(restored.history("antigravity").is_empty());
        let continued=restored.begin_cli("codex",Arc::default()).unwrap();let lease=restored.lease(&continued);assert_eq!(continued.session.as_ref().unwrap().cwd,"/tmp/assistant-project");
        assert!(!continued.cancel.load(Ordering::Relaxed));
        drop(lease);
        restored.reset_provider("codex").unwrap();assert!(Chat::persistent(dir.join("conversations.json")).history("codex").is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cancelling_a_finished_spoken_pet_never_stops_the_next_spoken_pet() {
        let chat=Chat::default();let mic=Arc::new(AtomicBool::new(false));let codex=chat.begin_cli("codex",mic.clone()).unwrap();
        chat.commit(&codex,json!({"role":"user","content":"a"}),json!({"role":"assistant","content":"b"}),"a","b");
        let gravity=chat.begin_cli("antigravity",mic.clone()).unwrap();chat.cancel_provider("codex");
        assert!(chat.is_current(&gravity));assert!(!mic.load(Ordering::Relaxed));
    }

    #[test]
    fn task_journal_recovers_pending_work_and_resuming_retires_only_its_old_record() {
        let dir=std::env::temp_dir().join(format!("coucou-task-journal-{}",std::process::id()));let path=dir.join("conversations.json");let _=std::fs::remove_dir_all(&dir);
        let chat=Chat::persistent(path.clone());let first=chat.begin_cli("codex",Arc::default()).unwrap();
        chat.start_task(&first,"cria postgres","/tmp/project","codex","model",None).unwrap();
        let old_id=chat.tasks()[0].id.clone();chat.task_update(&first,"approval","");
        let restored=Chat::persistent(path);let old=restored.tasks()[0].clone();assert_eq!(old.phase,"interrupted");assert!(old.finished.is_none());
        let next=restored.begin_cli("codex",Arc::default()).unwrap();restored.start_task(&next,&old.resume_prompt().unwrap(),"/tmp/project","codex","model",Some(&old.id)).unwrap();
        let tasks=restored.tasks();let current=tasks.iter().find(|t|t.resumes.as_deref()==Some(&old_id)).unwrap();
        assert_eq!(current.query,"cria postgres");assert_eq!(tasks.iter().find(|t|t.id==old_id).unwrap().phase,"resumed");
        assert!(!restored.cancel_task(&old_id));assert!(restored.is_current(&next));assert!(restored.cancel_task(&current.id));assert!(!restored.is_current(&next));
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;assert_eq!(std::fs::metadata(dir.join("conversations.json")).unwrap().permissions().mode()&0o777,0o600);}
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn failed_task(chat:&Chat,session:Option<&str>,text:&str)->crate::assistant_tasks::Task{
        let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);
        chat.start_task(&turn,text,"/tmp","codex","fixture",None).unwrap();
        if let Some(id)=session{chat.remember_session(&turn,id,"/tmp",false);}
        chat.commit(&turn,json!({"role":"user","content":text}),json!({"role":"assistant","content":"synthetic"}),text,"synthetic").unwrap();
        chat.task_update(&turn,"failed","");drop(lease);
        chat.tasks().into_iter().find(|task|task.query==text).unwrap()
    }

    fn legacy_failed_task(chat:&Chat,session:Option<&str>,text:&str)->crate::assistant_tasks::Task{
        let mut task=failed_task(chat,session,text);task.conversation_id=None;
        chat.inner.lock().unwrap().pets.get_mut("codex").unwrap().tasks.iter_mut().find(|row|row.id==task.id).unwrap().conversation_id=None;
        task
    }

    fn completion_task(chat:&Chat)->(Turn,TurnLease,String){
        let turn=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&turn);
        chat.start_task(&turn,"synthetic completion","/tmp","codex","fixture",None).unwrap();
        let id=chat.tasks()[0].id.clone();(turn,lease,id)
    }
    fn finish_task(chat:&Chat,turn:&Turn)->Result<String,String>{
        chat.commit_task_reply(turn,json!({"role":"user","content":"request"}),json!({"role":"assistant","content":"answer"}),"request","answer")
    }
    #[test]fn accepted_stop_cannot_turn_into_done_or_lose_an_acknowledged_completion(){
        let chat=Chat::default();let(turn,lease,id)=completion_task(&chat);
        assert!(chat.cancel_task(&id));assert!(finish_task(&chat,&turn).is_err());
        assert!(chat.history("codex").is_empty());assert_eq!(chat.tasks()[0].phase,"stopping");
        assert!(!chat.task_update(&turn,"working","late progress"));assert!(chat.task_update(&turn,"stopped",""));
        assert!(!chat.task_update(&turn,"done","late reply"));assert_eq!(chat.tasks()[0].phase,"stopped");drop(lease);

        let chat=Chat::default();let(turn,lease,id)=completion_task(&chat);finish_task(&chat,&turn).unwrap();
        assert!(!chat.cancel_task(&id));assert!(!chat.task_update(&turn,"working","late progress"));
        assert_eq!(chat.tasks()[0].phase,"done");assert_eq!(chat.history("codex").len(),2);drop(lease);
    }
    #[test]fn stop_and_completion_have_one_consistent_winner_under_contention(){
        for _ in 0..80{
            let chat=Chat::default();let(turn,lease,id)=completion_task(&chat);let gate=Arc::new(std::sync::Barrier::new(2));
            let worker={let chat=chat.clone();let turn=turn.clone();let gate=gate.clone();std::thread::spawn(move||{gate.wait();finish_task(&chat,&turn)})};
            gate.wait();let accepted=chat.cancel_task(&id);let completed=worker.join().unwrap();
            if accepted{assert!(completed.is_err());assert!(chat.history("codex").is_empty());assert_eq!(chat.tasks()[0].phase,"stopping");}
            else{assert!(completed.is_ok());assert_eq!(chat.history("codex").len(),2);assert_eq!(chat.tasks()[0].phase,"done");}
            drop(lease);
        }
    }
    #[test]fn completed_task_and_full_reply_survive_a_persistence_error_without_inviting_reexecution(){
        let directory=std::env::temp_dir().join(format!("coucou-completion-{}",crate::assistant_memory::new_id()));let path=directory.join("conversations.json");
        let chat=Chat::persistent(path.clone());let(turn,lease,id)=completion_task(&chat);
        std::fs::remove_file(&path).unwrap();std::fs::create_dir(&path).unwrap();
        assert!(finish_task(&chat,&turn).is_ok());assert!(!chat.cancel_task(&id));
        let view=chat.conversation("codex").unwrap();assert_eq!(view.messages.len(),2);assert!(view.notice.is_some());assert_eq!(chat.tasks()[0].phase,"done");
        drop(lease);std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]fn resuming_archived_task_restores_and_reserves_its_session_without_importing_another_dialogue(){
        let chat=Chat::default();let old=failed_task(&chat,Some("session-b"),"history B");
        let b=chat.conversation_id("codex").unwrap();chat.reset_provider("codex").unwrap();
        failed_task(&chat,Some("session-a"),"history A");let a=chat.conversation_id("codex").unwrap();
        let(task,reservation,changed)=chat.reserve_task_resume(&old.id).unwrap();assert!(changed);assert_eq!(task.id,old.id);
        assert_eq!(chat.conversation_id("codex").unwrap(),b);
        assert!(chat.reset_provider("codex").is_err());assert!(chat.reserve_task_resume(&old.id).is_err());
        let channel=reservation.channel();let turn=channel.begin_cli("codex",Arc::default()).unwrap();let lease=channel.lease(&turn);
        assert_eq!(turn.session.as_ref().unwrap().id,"session-b");
        let text=serde_json::to_string(&turn.history).unwrap();assert!(text.contains("history B"));assert!(!text.contains("history A"));
        assert_eq!(chat.read_conversation("codex",&a).unwrap().messages[0]["content"],"history A");
        channel.start_task(&turn,"resume","/tmp","codex","fixture",Some(&task.id)).unwrap();
        drop(lease);drop(reservation);assert!(chat.reserve_task_resume(&old.id).is_err());
    }

    #[test]fn resumed_child_restarted_before_native_callback_keeps_its_original_dialogue(){
        struct Scratch(std::path::PathBuf);
        impl Drop for Scratch{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let scratch=Scratch(std::env::temp_dir().join(format!("coucou-retry-origin-{}",crate::assistant_memory::new_id())));
        let path=scratch.0.join("conversations.json");let chat=Chat::persistent(path.clone());
        let native_b="019a734b-0000-7000-8000-000000000001";
        let parent=failed_task(&chat,Some(native_b),"synthetic B history");
        let b=chat.conversation_id("codex").unwrap();let history_b=chat.history("codex");
        chat.reset_provider("codex").unwrap();failed_task(&chat,Some("session-a"),"synthetic A history");
        let a=chat.conversation_id("codex").unwrap();
        let(task,reservation,changed)=chat.reserve_task_resume(&parent.id).unwrap();assert!(changed);
        let channel=reservation.channel();let turn=channel.begin_cli("codex",Arc::default()).unwrap();let lease=channel.lease(&turn);
        assert_eq!(turn.session.as_ref().unwrap().id,native_b);
        channel.start_task(&turn,&task.resume_prompt().unwrap(),"/tmp","codex","fixture",Some(&task.id)).unwrap();
        let child=chat.tasks().into_iter().find(|row|row.resumes.as_deref()==Some(parent.id.as_str())).unwrap();
        assert!(child.session_id.is_none());assert_eq!(child.conversation_id.as_deref(),Some(b.as_str()));
        // No initialize/session callback occurs before this synthetic restart.
        drop(lease);drop(turn);drop(channel);drop(reservation);drop(chat);
        let restored=Chat::persistent(path);
        let rows=restored.tasks();let recovered=rows.iter().find(|row|row.id==child.id).unwrap();
        assert_eq!(recovered.phase,"interrupted");assert!(recovered.session_id.is_none());assert_eq!(recovered.conversation_id.as_deref(),Some(b.as_str()));
        assert_eq!(rows.iter().find(|row|row.id==parent.id).unwrap().phase,"resumed");
        assert!(!restored.conversation("codex").unwrap().busy);
        let(_,reservation,changed)=restored.reserve_task_resume(&child.id).unwrap();assert!(!changed);
        let channel=reservation.channel();let retry=channel.begin_cli("codex",Arc::default()).unwrap();let lease=channel.lease(&retry);
        assert_eq!(restored.conversation_id("codex").unwrap(),b);
        assert_eq!(retry.session.as_ref().unwrap().id,native_b);assert_eq!(retry.history,history_b);
        assert_eq!(restored.read_conversation("codex",&a).unwrap().messages[0]["content"],"synthetic A history");
        drop(lease);
    }

    #[test]fn legacy_retry_without_an_origin_gets_clean_context_and_preserves_selected_history(){
        let chat=Chat::default();let old=legacy_failed_task(&chat,None,"unstarted B");
        chat.reset_provider("codex").unwrap();failed_task(&chat,Some("session-a"),"history A");let a=chat.conversation_id("codex").unwrap();
        let(_,reservation,changed)=chat.reserve_task_resume(&old.id).unwrap();assert!(changed);
        let channel=reservation.channel();let turn=channel.begin_cli("codex",Arc::default()).unwrap();let lease=channel.lease(&turn);
        assert!(turn.session.is_none());assert!(turn.history.is_empty());
        assert_eq!(chat.read_conversation("codex",&a).unwrap().messages[0]["content"],"history A");drop(lease);
    }

    #[test]fn missing_ambiguous_or_busy_task_origin_refuses_without_changing_selected_conversation(){
        for ambiguous in [false,true]{
            let chat=Chat::default();let old=legacy_failed_task(&chat,Some("session-b"),"history B");chat.reset_provider("codex").unwrap();
            failed_task(&chat,Some("session-a"),"history A");let before=chat.conversation_id("codex").unwrap();
            {let mut all=chat.inner.lock().unwrap();let rows=all.archives.get_mut("codex").unwrap();if ambiguous{let mut copy=rows[0].clone();copy.id=crate::assistant_memory::new_id();rows.push(copy);}else{rows.clear();}}
            assert!(chat.reserve_task_resume(&old.id).is_err());assert_eq!(chat.conversation_id("codex").unwrap(),before);
            assert_eq!(chat.tasks().into_iter().find(|t|t.id==old.id).unwrap().phase,"failed");
        }
        let chat=Chat::default();let old=legacy_failed_task(&chat,Some("session-b"),"history B");chat.reset_provider("codex").unwrap();
        let a=chat.reserve_conversation("codex",None).unwrap();let before=chat.conversation_id("codex").unwrap();
        assert!(chat.reserve_task_resume(&old.id).is_err());assert_eq!(chat.conversation_id("codex").unwrap(),before);drop(a);
    }

    #[test]
    fn a_turn_is_recorded_only_once_it_succeeds() {
        let chat = Chat::default();
        let t = chat.begin("anthropic");
        assert!(t.first);
        assert!(t.history.is_empty());
        // A failed turn records nothing: the next one is still the first.
        let t = chat.begin("anthropic");
        assert!(t.first);
        chat.commit(&t, json!({"role":"user","content":[{"type":"text","text":"hi"}]}), json!({"role":"assistant","content":[{"type":"text","text":"hello"}]}), "hi", "hello");
        let t = chat.begin("anthropic");
        assert!(!t.first);
        assert_eq!(t.history.len(), 2);
        assert!(t.history[0]["content"].is_array());
    }

    #[test]
    fn switching_provider_keeps_each_context_private() {
        let chat = Chat::default();
        let t = chat.begin("anthropic");
        chat.commit(&t,json!({"role":"user","content":"private"}),json!({"role":"assistant","content":"reply"}),"private","reply");
        let other = chat.begin("ollama");
        assert!(other.first && other.history.is_empty());
        chat.commit(&other,json!({"role":"user","content":"local"}),json!({"role":"assistant","content":"ok"}),"local","ok");
        let inspection=chat.begin("anthropic");assert_eq!(inspection.history.len(),2);drop(chat.lease(&inspection));
        chat.reset_provider("anthropic").unwrap();
        assert!(chat.begin("anthropic").first);
        assert_eq!(chat.begin("ollama").history.len(),2);
    }

    #[test]
    fn reading_an_archive_does_not_switch_or_interrupt_the_live_conversation() {
        let chat=Chat::default();
        let original=chat.conversation("codex").unwrap().id;
        let t=chat.begin("codex");
        chat.commit(&t,json!({"role":"user","content":"old"}),json!({"role":"assistant","content":"old answer"}),"old","old answer");
        drop(chat.lease(&t));chat.reset_provider("codex").unwrap();
        let current=chat.conversation("codex").unwrap().id;
        let active=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&active);
        let saved=chat.read_conversation("codex",&original).unwrap();
        assert_eq!(saved.messages.len(),2);assert!(!saved.busy);
        assert_eq!(chat.conversation("codex").unwrap().id,current);
        assert!(chat.is_current(&active));
        assert!(chat.read_conversation("ollama",&original).is_err());
        assert!(chat.read_conversation("codex","unknown").is_err());drop(lease);
    }

    #[test]
    fn library_migrates_v1_and_keeps_public_dialogues_sessions_and_pet_tasks_across_restore() {
        let dir=std::env::temp_dir().join(format!("coucou-library-{}",crate::assistant_memory::new_id()));std::fs::create_dir_all(&dir).unwrap();
        let path=dir.join("conversations.json");
        std::fs::write(&path,json!({"version":1,"pets":{"codex":{"messages":[{"role":"user","content":"primeira conversa"},{"role":"assistant","content":"pronto"}],"session":{"id":"owned-first","cwd":"/tmp","full_access":false}}}}).to_string()).unwrap();
        let chat=Chat::persistent(path.clone());let original=chat.conversation("codex").unwrap().id;
        let work=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&work);
        chat.start_task(&work,"tarefa persistente","/tmp","codex","test",None).unwrap();chat.task_update(&work,"done","resultado");drop(lease);
        let task=chat.tasks()[0].id.clone();
        chat.reset_provider("codex").unwrap();let fresh=chat.conversation("codex").unwrap().id;assert_ne!(fresh,original);
        let ollama=chat.begin_cli("ollama",Arc::default()).unwrap();
        chat.commit(&ollama,json!({"role":"user","content":"local"}),json!({"role":"assistant","content":"resposta"}),"local","resposta");
        assert!(chat.conversations("ollama","primeira").unwrap().is_empty());
        assert!(chat.restore_conversation("ollama",&original).is_err());
        let result=chat.restore_conversation("codex",&original).unwrap();assert_eq!(result.messages.len(),2);
        assert_eq!(chat.tasks()[0].id,task);assert_eq!(chat.tasks()[0].result,"resultado");
        let continued=chat.begin_cli("codex",Arc::default()).unwrap();assert_eq!(continued.session.as_ref().unwrap().id,"owned-first");drop(chat.lease(&continued));
        let restored=Chat::persistent(path.clone());assert_eq!(restored.conversation("codex").unwrap().id,original);
        assert_eq!(restored.history("ollama").len(),2);assert_eq!(restored.tasks()[0].id,task);
        let disk:Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();assert_eq!(disk["version"],3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn library_refuses_selection_during_model_discovery_or_voice_and_rejects_stale_window_sends() {
        let chat=Chat::default();let original=chat.conversation("ollama").unwrap().id;
        let selection=chat.reserve_conversation("ollama",Some(&original)).unwrap();
        assert!(chat.reset_provider("ollama").is_err());drop(selection);
        let voice=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&voice);
        assert!(chat.begin_cli("ollama",Arc::default()).is_err());assert!(chat.reset_provider("ollama").is_err());
        chat.commit(&voice,json!({"role":"user","content":"voz"}),json!({"role":"assistant","content":"texto"}),"voz","texto");drop(lease);
        chat.reset_provider("ollama").unwrap();
        assert!(chat.reserve_conversation("ollama",Some(&original)).is_err());
        let work=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&work);
        assert!(chat.restore_conversation("ollama",&original).is_err());drop(lease);
        chat.restore_conversation("ollama",&original).unwrap();
        let text=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&text);
        assert_eq!(text.history.len(),2);assert_eq!(text.history[0]["content"],"voz");
        let snapshot=chat.conversation("ollama").unwrap();assert!(snapshot.busy);assert_eq!(snapshot.turn,text.epoch);
        assert!(!chat.cancel_conversation("ollama",Some("other-archive"),Some(text.epoch)));
        assert!(!chat.cancel_conversation("ollama",Some(&original),Some(text.epoch-1)));assert!(chat.is_current(&text));
        assert!(chat.cancel_conversation("ollama",Some(&original),Some(text.epoch)));assert!(!chat.is_current(&text));
        assert!(chat.conversation("ollama").unwrap().busy);drop(lease);assert!(!chat.conversation("ollama").unwrap().busy);
    }

    #[test]
    fn a_failed_archive_write_and_unreadable_library_never_discard_the_previous_dialogue() {
        let dir=std::env::temp_dir().join(format!("coucou-library-errors-{}",crate::assistant_memory::new_id()));let path=dir.join("conversations.json");
        let chat=Chat::persistent(path.clone());let t=chat.begin_cli("ollama",Arc::default()).unwrap();
        chat.commit(&t,json!({"role":"user","content":"preservar"}),json!({"role":"assistant","content":"sim"}),"preservar","sim");
        let id=chat.conversation("ollama").unwrap().id;
        std::fs::remove_file(&path).unwrap();std::fs::create_dir(&path).unwrap();
        assert!(chat.reset_provider("ollama").is_err());assert_eq!(chat.conversation("ollama").unwrap().id,id);assert_eq!(chat.history("ollama").len(),2);
        std::fs::remove_dir(&path).unwrap();std::fs::write(&path,b"{invalid library").unwrap();
        let unreadable=Chat::persistent(path.clone());assert!(unreadable.conversation("ollama").is_err());assert!(unreadable.reset_provider("ollama").is_err());
        assert!(unreadable.begin_cli("ollama",Arc::default()).is_err());assert_eq!(std::fs::read(&path).unwrap(),b"{invalid library");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_durable_public_archive_keeps_long_messages_and_more_than_eighty_messages() {
        let dir=std::env::temp_dir().join(format!("coucou-full-archive-{}",crate::assistant_memory::new_id()));let path=dir.join("conversations.json");
        let chat=Chat::persistent(path.clone());let answer="resposta completa ".repeat(1000);
        for n in 0..45 {
            let t=chat.begin_cli("ollama",Arc::default()).unwrap();let lease=chat.lease(&t);let question=format!("pergunta {n}");
            chat.commit_reply(&t,json!({"role":"user","content":question}),json!({"role":"assistant","content":answer}),&question,&answer).unwrap();drop(lease);
        }
        let original=chat.conversation("ollama").unwrap().id;assert_eq!(chat.history("ollama").len(),90);
        chat.reset_provider("ollama").unwrap();let restored=Chat::persistent(path);
        let archive=restored.read_conversation("ollama",&original).unwrap();assert_eq!(archive.messages.len(),90);assert_eq!(archive.messages[1]["content"],answer);
        restored.restore_conversation("ollama",&original).unwrap();let next=restored.begin_cli("ollama",Arc::default()).unwrap();
        assert!(next.history.len()<80);assert!(serde_json::to_vec(&next.history).unwrap().len()<=crate::assistant_memory::MAX_CONTEXT_BYTES);drop(restored.lease(&next));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_delivered_reply_with_a_save_warning_is_not_reported_as_an_unexecuted_send() {
        let dir=std::env::temp_dir().join(format!("coucou-save-retry-{}",crate::assistant_memory::new_id()));let path=dir.join("conversations.json");
        let chat=Chat::persistent(path.clone());let t=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&t);
        chat.commit_reply(&t,json!({"role":"user","content":"first"}),json!({"role":"assistant","content":"one"}),"first","one").unwrap();drop(lease);
        let id=chat.conversation("codex").unwrap().id;let previous=std::fs::read(&path).unwrap();let backup=path.with_extension("backup");std::fs::rename(&path,&backup).unwrap();std::fs::create_dir(&path).unwrap();
        let t=chat.begin_cli("codex",Arc::default()).unwrap();let lease=chat.lease(&t);
        assert!(chat.commit_reply(&t,json!({"role":"user","content":"executed"}),json!({"role":"assistant","content":"done"}),"executed","done").is_ok());drop(lease);
        let view=chat.conversation("codex").unwrap();assert_eq!(view.id,id);assert_eq!(view.messages.len(),4);assert!(view.notice.is_some());assert_eq!(std::fs::read(&backup).unwrap(),previous);
        assert!(chat.reset_provider("codex").is_err());assert_eq!(chat.conversation("codex").unwrap().messages.len(),4);
        std::fs::remove_dir(&path).unwrap();std::fs::rename(&backup,&path).unwrap();
        let t=chat.begin_cli("codex",Arc::default()).unwrap();drop(chat.lease(&t));assert!(chat.conversation("codex").unwrap().notice.is_none());
        assert_eq!(Chat::persistent(path).history("codex").len(),4);std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn migration_of_a_v2_library_never_introduces_a_new_public_history_limit() {
        let dir=std::env::temp_dir().join(format!("coucou-v2-full-{}",crate::assistant_memory::new_id()));std::fs::create_dir_all(&dir).unwrap();let path=dir.join("conversations.json");
        let messages:Vec<_>=(0..122).map(|n|json!({"role":if n%2==0{"user"}else{"assistant"},"content":format!("mensagem {n}")})).collect();
        std::fs::write(&path,json!({"version":2,"pets":{"ollama":{"messages":messages,"session":null}}}).to_string()).unwrap();
        let chat=Chat::persistent(path.clone());let id=chat.conversation("ollama").unwrap().id;assert_eq!(chat.history("ollama").len(),122);chat.reset_provider("ollama").unwrap();
        assert_eq!(Chat::persistent(path.clone()).read_conversation("ollama",&id).unwrap().messages.len(),122);assert_eq!(serde_json::from_slice::<Value>(&std::fs::read(path).unwrap()).unwrap()["version"],3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reset_rejects_late_answers_but_switching_pets_keeps_owned_work() {
        let chat = Chat::default();
        let t = chat.begin("openai");
        chat.reset();
        chat.commit(&t, json!({"role":"user","content":"q"}), json!({"role":"assistant","content":"a"}), "q", "a");
        assert!(chat.begin("openai").first);

        let t = chat.begin("openai");
        let _other = chat.begin("google");
        chat.commit(&t, json!({"role":"user","content":"q"}), json!({"role":"assistant","content":"a"}), "q", "a");
        assert!(chat.begin("google").first);
        assert_eq!(chat.history("openai").len(),2);
    }

    #[test]
    fn cli_sessions_belong_only_to_the_conversation_that_created_them() {
        let chat = Chat::default();
        let turn = chat.begin("codex");
        chat.remember_session(&turn, "owned-thread", "/tmp/my-project", false);
        let next = chat.begin("codex");
        assert_eq!(next.session.as_ref().unwrap().id, "owned-thread");
        assert!(!next.session.as_ref().unwrap().full_access);
        chat.cancel();
        assert!(!chat.is_current(&next));
        assert!(chat.begin("antigravity").session.is_none());
        chat.reset();
        assert!(chat.begin("codex").session.is_none());
    }

    #[test]
    fn the_prompt_greets_by_first_name_and_claims_web_search_only_for_claude() {
        let p = system_prompt_for(Some("Louis"), true);
        assert!(p.starts_with("You are Mochi, Louis's personal AI assistant living at the top of their screen."));
        assert!(p.contains("web search access"));
        assert!(p.contains("light Markdown"));
        let p = system_prompt_for(None, false);
        assert!(p.starts_with("You are Mochi, a personal AI assistant living at the top of the user's screen."));
        assert!(!p.contains("web search"));
        assert!(p.contains("no web access"));
    }

    #[test]
    fn explicit_files_go_with_every_question_but_window_context_is_first_only() {
        let file = ChatContext::File { name: "a.txt".into(), path: "/x/a.txt".into() };
        assert_eq!(plain_question(true, Some(&file), "why?"), "File: a.txt\n\nwhy?");
        assert_eq!(plain_question(false, Some(&file), "why?"), "File: a.txt\n\nwhy?");
        let win = ChatContext::Window { app_name: "Code".into(), title: "main.rs".into(), url: None };
        assert_eq!(plain_question(true, Some(&win), "q"), "Context — App: Code, Window: main.rs\n\nq");
        assert_eq!(plain_question(false, Some(&win), "q"), "q");
        assert_eq!(window_line("Edge", "Docs", Some("https://x.dev")), "Context — App: Edge, Window: Docs, URL: https://x.dev");
    }

    #[test]
    fn the_model_comes_from_the_settings_or_the_provider_default() {
        let mut s = Settings::default();
        assert_eq!(model_for(&s, "anthropic"), claude::DEFAULT_MODEL);
        assert_eq!(model_for(&s, "openai"), openai_compat::provider("openai").unwrap().default_model);
        assert_eq!(model_for(&s, "ollama"), "@default");
        s.chat_models.insert("openai".into(), " gpt-x ".into());
        s.chat_models.insert("ollama".into(), "llama3.2".into());
        s.model = "claude-haiku-4-5".into();
        assert_eq!(model_for(&s, "openai"), "gpt-x");
        assert_eq!(model_for(&s, "ollama"), "llama3.2");
        assert_eq!(model_for(&s, "anthropic"), "claude-haiku-4-5");
    }

    #[test]
    fn only_files_in_the_inbox_ride_along() {
        let base = std::env::temp_dir().join(format!("coucou-chat-ctx-{}", std::process::id()));
        let inbox = base.join("inbox");
        std::fs::create_dir_all(inbox.join("sub")).unwrap();
        std::fs::write(inbox.join("a.txt"), b"a").unwrap();
        std::fs::write(base.join("secret.txt"), b"s").unwrap();
        std::fs::write(inbox.join("sub").join("b.txt"), b"b").unwrap();
        assert!(is_inside(&inbox, &inbox.join("a.txt")));
        assert!(!is_inside(&inbox, &base.join("secret.txt")));
        assert!(!is_inside(&inbox, &inbox.join("..").join("secret.txt")));
        assert!(!is_inside(&inbox, &inbox.join("sub").join("b.txt")));
        assert!(!is_inside(&inbox, &inbox.join("missing.txt")));
        assert!(!is_inside(&inbox, &inbox));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(base.join("secret.txt"), inbox.join("link.txt")).unwrap();
            assert!(!is_inside(&inbox, &inbox.join("link.txt")));
        }
        let _ = std::fs::remove_dir_all(&base);
    }
    struct TaskTransactionScratch(std::path::PathBuf);
    impl TaskTransactionScratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "coucou-task-start-transaction-{}", crate::assistant_memory::new_id()));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path).unwrap();
            Self(path)
        }
        fn journal(&self) -> std::path::PathBuf { self.0.join("conversations.json") }
    }
    impl Drop for TaskTransactionScratch {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    fn task_transaction_snapshot(chat: &Chat) -> Value {
        let all = chat.inner.lock().unwrap();
        let pets: serde_json::Map<String, Value> = all.pets.iter().map(|(provider, c)| (
            provider.clone(), json!({"saved":c.saved(),"native":c.native,"epoch":c.epoch,
                "pending":c.pending.is_some(),"active":c.active.is_some(),
                "liveText":c.live_text,"liveQuery":c.live_query})
        )).collect();
        json!({"next":all.next,"pets":pets,"archives":all.archives})
    }

    #[test]
    fn task_start_write_failure_keeps_ram_parent_and_original_bytes_without_new_row() {
        for retry in [false, true] {
            let scratch = TaskTransactionScratch::new();
            let path = scratch.journal();
            let chat = Chat::persistent(path.clone());
            let parent = failed_task(&chat, Some("synthetic-session-b"), "synthetic B history");
            let turn = chat.begin_cli("codex", Arc::default()).unwrap();
            let _lease = chat.lease(&turn);
            let before = task_transaction_snapshot(&chat);
            let previous_bytes = std::fs::read(&path).unwrap();
            let backup = scratch.0.join("preserved-baseline.json");
            std::fs::rename(&path, &backup).unwrap();
            std::fs::create_dir(&path).unwrap(); // Deterministic rename obstruction.
            let result = chat.start_task(&turn, "synthetic new request", "/tmp", "codex",
                "fixture", if retry { Some(parent.id.as_str()) } else { None });
            assert!(result.is_err(), "journal failure must refuse task admission");
            assert_eq!(task_transaction_snapshot(&chat), before);
            assert_eq!(std::fs::read(&backup).unwrap(), previous_bytes);
            let rows = chat.tasks();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].id, parent.id);
            assert_eq!(rows[0].phase, "failed");
            assert!(rows[0].resumed_by.is_none());
            assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 2,
                "failed save must remove its own temporary file");
            assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
        }
    }

    #[test]
    fn task_conversation_origin_is_durable_before_startup_without_claiming_native_session() {
        let scratch = TaskTransactionScratch::new();
        let path = scratch.journal();
        let chat = Chat::persistent(path.clone());
        let parent = failed_task(&chat, Some("synthetic-session-b"), "synthetic B history");
        let b = chat.conversation_id("codex").unwrap();
        let history_b = chat.history("codex");
        let (_, reservation, _) = chat.reserve_task_resume(&parent.id).unwrap();
        let channel = reservation.channel();
        let turn = channel.begin_cli("codex", Arc::default()).unwrap();
        let lease = channel.lease(&turn);
        assert_eq!(turn.session.as_ref().unwrap().id, "synthetic-session-b");
        channel.start_task(&turn, "synthetic retry", "/tmp", "codex", "fixture",
            Some(&parent.id)).unwrap();
        let child = chat.tasks().into_iter()
            .find(|row| row.resumes.as_deref() == Some(parent.id.as_str())).unwrap();
        assert_eq!(child.conversation_id.as_deref(), Some(b.as_str()));
        assert!(child.session_id.is_none(), "an inherited turn is not a new native ACK");
        // No remember_session call occurs before this synthetic restart.
        drop(lease); drop(turn); drop(channel); drop(reservation); drop(chat);
        let restored = Chat::persistent(path);
        let recovered = restored.tasks().into_iter().find(|row| row.id == child.id).unwrap();
        assert_eq!(recovered.phase, "interrupted");
        assert_eq!(recovered.conversation_id.as_deref(), Some(b.as_str()));
        assert!(recovered.session_id.is_none());
        restored.reset_provider("codex").unwrap();
        failed_task(&restored, Some("synthetic-session-a"), "synthetic A history");
        let a = restored.conversation_id("codex").unwrap();
        let (_, reservation, changed) = restored.reserve_task_resume(&child.id).unwrap();
        assert!(changed);
        let channel = reservation.channel();
        let turn = channel.begin_cli("codex", Arc::default()).unwrap();
        let _lease = channel.lease(&turn);
        assert_eq!(restored.conversation_id("codex").unwrap(), b);
        assert_eq!(turn.history, history_b);
        assert_eq!(turn.session.as_ref().unwrap().id, "synthetic-session-b");
        assert_eq!(restored.read_conversation("codex", &a).unwrap().messages[0]["content"],
            "synthetic A history");
    }

    #[test]
    fn exact_task_conversation_id_wins_over_ambiguous_native_id_and_missing_id_refuses() {
        let chat = Chat::default();
        let old = failed_task(&chat, Some("synthetic-shared-native"), "synthetic B history");
        let b = chat.conversation_id("codex").unwrap();
        chat.reset_provider("codex").unwrap();
        failed_task(&chat, Some("synthetic-shared-native"), "synthetic A history");
        let a = chat.conversation_id("codex").unwrap();
        // Both native IDs match. The persisted conversation ID is the exact origin.
        assert_eq!(old.conversation_id.as_deref(), Some(b.as_str()));
        let (_, reservation, changed) = chat.reserve_task_resume(&old.id).unwrap();
        assert!(changed);
        assert_eq!(chat.conversation_id("codex").unwrap(), b);
        assert_eq!(chat.read_conversation("codex", &a).unwrap().messages[0]["content"],
            "synthetic A history");
        drop(reservation);

        let chat = Chat::default();
        let old = failed_task(&chat, Some("synthetic-unique-native-b"), "synthetic B history");
        chat.reset_provider("codex").unwrap();
        failed_task(&chat, Some("synthetic-unique-native-a"), "synthetic A history");
        {
            let mut all = chat.inner.lock().unwrap();
            let row = all.pets.get_mut("codex").unwrap().tasks.iter_mut()
                .find(|row| row.id == old.id).unwrap();
            row.conversation_id = Some("synthetic-missing-origin".into());
        }
        let before = task_transaction_snapshot(&chat);
        // A valid legacy session could locate B, but must not override an explicit,
        // now-missing conversation identity and silently choose a different origin.
        assert!(chat.conversation_for_session("codex", "synthetic-unique-native-b").is_some());
        assert!(chat.reserve_task_resume(&old.id).is_err());
        assert_eq!(task_transaction_snapshot(&chat), before);
    }

    #[test]
    fn legacy_task_without_conversation_id_keeps_unique_session_fallback_and_refuses_ambiguity() {
        for ambiguous in [false, true] {
            let chat = Chat::default();
            let old = failed_task(&chat, Some("synthetic-session-b"), "synthetic B history");
            let b = chat.conversation_id("codex").unwrap();
            chat.reset_provider("codex").unwrap();
            failed_task(&chat, Some(if ambiguous { "synthetic-session-b" } else {
                "synthetic-session-a" }), "synthetic A history");
            {
                let mut all = chat.inner.lock().unwrap();
                all.pets.get_mut("codex").unwrap().tasks.iter_mut()
                    .find(|row| row.id == old.id).unwrap().conversation_id = None;
            }
            let before = task_transaction_snapshot(&chat);
            let result = chat.reserve_task_resume(&old.id);
            if ambiguous {
                assert!(result.is_err());
                assert_eq!(task_transaction_snapshot(&chat), before);
            } else {
                let (_, reservation, changed) = result.unwrap();
                assert!(changed);
                assert_eq!(chat.conversation_id("codex").unwrap(), b);
                let channel = reservation.channel();
                let turn = channel.begin_cli("codex", Arc::default()).unwrap();
                let _lease = channel.lease(&turn);
                assert_eq!(turn.session.as_ref().unwrap().id, "synthetic-session-b");
                assert_eq!(turn.history[0]["content"], "synthetic B history");
            }
        }
    }

    #[test]
    fn legacy_task_with_neither_origin_gets_clean_context_and_archives_current_a() {
        let chat = Chat::default();
        let old = failed_task(&chat, None, "synthetic sessionless B");
        {
            let mut all = chat.inner.lock().unwrap();
            let row = all.pets.get_mut("codex").unwrap().tasks.iter_mut()
                .find(|row| row.id == old.id).unwrap();
            row.conversation_id = None;
            assert!(row.session_id.is_none());
        }
        chat.reset_provider("codex").unwrap();
        failed_task(&chat, Some("synthetic-session-a"), "synthetic A history");
        let a = chat.conversation_id("codex").unwrap();
        let (_, reservation, changed) = chat.reserve_task_resume(&old.id).unwrap();
        assert!(changed);
        let channel = reservation.channel();
        let turn = channel.begin_cli("codex", Arc::default()).unwrap();
        let _lease = channel.lease(&turn);
        assert_ne!(chat.conversation_id("codex").unwrap(), a);
        assert!(turn.session.is_none());
        assert!(turn.history.is_empty());
        assert_eq!(chat.read_conversation("codex", &a).unwrap().messages[0]["content"],
            "synthetic A history");
        let preserved = chat.tasks().into_iter().find(|row| row.id == old.id).unwrap();
        assert_eq!(preserved.phase, "failed");
        assert!(preserved.resumed_by.is_none());
    }

}
