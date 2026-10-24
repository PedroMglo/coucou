//! Native drops append attachments to the conversation captured at drop time.
//! Tokens, source paths and copy tickets stay in RAM. Clearing context preserves
//! the inbox copy and the separate document/email permission grant.
use std::{collections::BTreeMap, path::{Component, Path}, sync::{Arc,Mutex, OnceLock}, time::{Duration, Instant}};
use ring::rand::{SecureRandom, SystemRandom};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
mod request;
#[cfg(target_os="linux")]
mod manifest;
#[cfg(target_os="linux")]
mod copy_review;
#[cfg(target_os="linux")]
mod folder_watch;
#[cfg(target_os="linux")]
mod renewal;
pub(crate) mod content;
pub(crate) mod policy;
pub(crate) mod folder_text;
pub(crate) use request::Request;

const DROP_TTL: Duration = Duration::from_secs(120);
const MAX_DROPS: usize = 64;
const MAX_PATHS: usize = 32;
const MAX_PATH_BYTES: usize = 4096;
const MAX_DROP_BYTES: usize = 64 * 1024;
const MAX_ATTACHMENTS: usize = 128;
const MAX_CONVERSATION_ATTACHMENTS:usize=32;
const MAX_CONVERSATION_BYTES:u64=100*1024*1024;
const CHAT_PROVIDERS: &[&str] = &["codex", "copilot", "antigravity", "ollama", "lmstudio", "anthropic", "openai", "google"];
const DROP_EXPIRED: &str = "A entrega do documento expirou ou foi cancelada. Volta a arrastá-lo para o pet.";
const CONTEXT_CHANGED: &str = "A conversa ou o documento selecionado mudou durante a cópia. O ficheiro ficou na caixa de entrada e não foi anexado a outra conversa.";
const CONTEXT_REVOKED: &str = "Este documento já não está selecionado nesta conversa. Volta a entregá-lo ao pet antes de o enviar.";
const COPY_DEADLINE:&str="O prazo da entrega terminou. Os anexos anteriores foram preservados; as novas cópias não foram selecionadas.";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id:String,
    #[serde(flatten)] pub file: crate::files::DroppedFile,
    pub provider: String,
    pub conversation_id: String,
    pub sequence: u64,
}
impl Attachment {
    pub(crate) fn context_file(&self)->crate::chat::ContextFile{crate::chat::ContextFile{id:self.id.clone(),name:self.file.name.clone(),path:self.file.path.clone(),sequence:self.sequence}}
}

#[derive(Clone, PartialEq, Eq)]
struct Ticket { provider: String, id: String, sequence: u64, drop_id: String, paths:Vec<String>,deadline:Instant }
struct Capture {
    sequence: u64,
    at: Instant,
    generation: u64,
    revisions: BTreeMap<String, u64>,
}
/// Provisional identity at native mouse release. No paths or usable grant exist
/// until GTK returns the requested URI data and the revisions still match.
pub struct NativeDrop {
    capture: Capture,
    owners: BTreeMap<String, String>,
}
struct DropGrant {
    sequence: u64,
    at: Instant,
    paths: Vec<String>,
    owners: BTreeMap<String, String>,
}
struct Ready { attachment: Attachment, drop_id: String }
#[derive(Default)]
struct Store {
    next: u64,
    generation: u64,
    revisions: BTreeMap<String, u64>,
    drops: BTreeMap<String, DropGrant>,
    selection_revisions:BTreeMap<(String,String),u64>,
    // Each drop owns its pending copy. Concurrent drops append independently;
    // resetting the owner still revokes every captured ticket for that owner.
    pending: BTreeMap<(String,String), Ticket>,
    files: BTreeMap<(String, String), Vec<Ready>>,
}
impl Store {
    #[cfg(target_os="linux")]
    fn copy_purge_available(&self,provider:&str,cid:&str,ids:&[String],sources:&[Attachment])->Result<(),String>{
        let touches=|path:&str|sources.iter().any(|a|Path::new(path)==Path::new(&a.file.path)||(a.file.kind=="folder"&&Path::new(path).starts_with(Path::new(&a.file.path))));
        if self.pending.values().any(|t|(t.provider==provider&&t.id==cid)||t.paths.iter().any(|p|touches(p)))
            ||self.drops.values().any(|d|d.paths.iter().any(|p|touches(p)))
            ||self.files.values().flatten().any(|r|ids.contains(&r.attachment.id)||touches(&r.attachment.file.path)){
            return Err("Uma cópia está selecionada ou participa numa entrega pendente. Retira-a e termina ou cancela as entregas antes de rever a eliminação.".into());
        }Ok(())
    }
    fn touch(&mut self,provider:&str,cid:&str){if let Some(r)=self.selection_revisions.get_mut(&(provider.into(),cid.into())){*r=r.saturating_add(1);}}
    #[cfg(target_os="linux")]
    fn review_fence(&mut self,provider:&str,cid:&str)->Result<(u64,u64,u64),String>{
        if self.selection_revisions.len()>=4096&&!self.selection_revisions.contains_key(&(provider.into(),cid.into())){return Err("O limite de revisões de conversas foi atingido. Reabre esta conversa antes de rever.".into());}
        let selection=*self.selection_revisions.entry((provider.into(),cid.into())).or_default();
        if selection==u64::MAX{return Err(CONTEXT_REVOKED.into());}
        Ok((self.generation,*self.revisions.get(provider).unwrap_or(&0),selection))
    }
    #[cfg(target_os="linux")]
    fn review_current(&self,provider:&str,cid:&str,fence:(u64,u64,u64))->bool{
        fence.2!=u64::MAX&&(self.generation,*self.revisions.get(provider).unwrap_or(&0),*self.selection_revisions.get(&(provider.into(),cid.into())).unwrap_or(&0))==fence
    }
    #[cfg(target_os="linux")]
    fn stage_restored(&self,provider:&str,cid:&str,files:&[Attachment])->Result<(u64,Vec<Attachment>),String>{
        if files.is_empty()||files.len()>MAX_CONVERSATION_ATTACHMENTS{return Err(CONTEXT_REVOKED.into());}
        let existing=self.contexts(provider,cid);let mut ids=std::collections::BTreeSet::new();let mut incoming=vec![];let mut selected=vec![];
        for file in files{
            if file.provider!=provider||file.conversation_id!=cid||!ids.insert(&file.id){return Err(CONTEXT_REVOKED.into());}
            if let Some(old)=existing.iter().find(|a|a.id==file.id){if old.file.path!=file.file.path||old.file.size!=file.file.size||old.file.kind!=file.file.kind||old.file.entries!=file.file.entries{return Err(CONTEXT_REVOKED.into());}selected.push(old.clone());}else{incoming.push(file);selected.push(file.clone());}
        }
        let total=existing.iter().map(|a|a.file.size).chain(incoming.iter().map(|a|a.file.size)).try_fold(0u64,u64::checked_add);
        let pending=self.pending.values().map(|t|t.paths.len()).sum::<usize>();
        let owner_pending=self.pending.values().filter(|t|t.provider==provider&&t.id==cid).map(|t|t.paths.len()).sum::<usize>();
        if existing.len()+incoming.len()+owner_pending>MAX_CONVERSATION_ATTACHMENTS||self.files.values().map(Vec::len).sum::<usize>()+incoming.len()+pending>MAX_ATTACHMENTS||total.is_none_or(|n|n>MAX_CONVERSATION_BYTES){return Err("Não há espaço para renovar este conjunto de anexos. Remove itens ou cancela entregas pendentes.".into());}
        let next=if incoming.is_empty(){self.next}else{self.next.checked_add(1).ok_or(CONTEXT_REVOKED)?};
        for a in &mut selected{if incoming.iter().any(|n|n.id==a.id){a.sequence=next;}}
        Ok((next,selected))
    }
    #[cfg(target_os="linux")]
    fn publish_restored(&mut self,provider:&str,cid:&str,next:u64,selected:&[Attachment]){
        self.next=next;let values=self.files.entry((provider.into(),cid.into())).or_default();
        for a in selected{if !values.iter().any(|r|r.attachment.id==a.id){values.push(Ready{attachment:a.clone(),drop_id:String::new()});}}
        values.sort_by_key(|r|r.attachment.sequence);self.touch(provider,cid);
    }
    fn expire(&mut self, now: Instant) {
        self.drops.retain(|_, grant| now.saturating_duration_since(grant.at) < DROP_TTL);
        self.pending.retain(|_,ticket|now<ticket.deadline);
    }

    fn prepare_capture(&mut self, providers: &[String], now: Instant) -> Result<Capture, String> {
        self.expire(now);
        if self.drops.len() >= MAX_DROPS { return Err("Há demasiadas entregas pendentes. Cancela uma entrega ou aguarda que expire.".into()); }
        self.next = self.next.checked_add(1).ok_or("Não foi possível reservar esta entrega.")?;
        Ok(Capture {
            sequence: self.next, at: now, generation: self.generation,
            revisions: providers.iter().map(|p| (p.clone(), *self.revisions.get(p).unwrap_or(&0))).collect(),
        })
    }

    fn finish_capture(&mut self, capture: Capture, token: String, paths: Vec<String>, mut owners: BTreeMap<String, String>, now: Instant) -> Result<String, String> {
        validate_paths(&paths)?;
        self.expire(now);
        if now.saturating_duration_since(capture.at) >= DROP_TTL || capture.generation != self.generation { return Err(DROP_EXPIRED.into()); }
        owners.retain(|provider, _| capture.revisions.get(provider).is_some_and(|revision| *revision == *self.revisions.get(provider).unwrap_or(&0)));
        if owners.is_empty() { return Err("A conversa mudou durante a entrega. Volta a arrastar o documento para o pet.".into()); }
        if self.drops.len() >= MAX_DROPS || self.drops.contains_key(&token) { return Err("Não foi possível reservar esta entrega. Volta a arrastar o documento.".into()); }
        self.drops.insert(token.clone(), DropGrant { sequence: capture.sequence, at: capture.at, paths, owners });
        Ok(token)
    }

    fn claim(&mut self, token: &str, provider: &str, path: &str, expected: Option<&str>, current: &str, now: Instant) -> Result<Ticket, String> {
        self.claim_paths(token,provider,Some(path),expected,current,now)
    }

    fn claim_paths(&mut self,token:&str,provider:&str,path:Option<&str>,expected:Option<&str>,current:&str,now:Instant)->Result<Ticket,String>{
        self.expire(now);
        let grant = self.drops.get(token).ok_or(DROP_EXPIRED)?;
        // Substituted paths/owners cannot consume the real owner's grant. Once
        // both match, the entire OS drop is used exactly once, even on failure.
        if path.is_some_and(|path|!grant.paths.iter().any(|p|p==path)) || !grant.owners.contains_key(provider) { return Err("Este documento não foi entregue a este pet nesta conversa.".into()); }
        if self.pending.len()>=MAX_DROPS{return Err("Há demasiadas cópias pendentes. Aguarda ou cancela uma entrega.".into());}
        let incoming=if path.is_some(){1}else{grant.paths.len()};
        let pending_count=self.pending.values().map(|t|t.paths.len()).sum::<usize>();
        let owner_pending=self.pending.values().filter(|t|t.provider==provider&&t.id==current).map(|t|t.paths.len()).sum::<usize>();
        let ready=self.files.get(&(provider.into(),current.into())).map_or(0,Vec::len);
        if self.files.values().map(Vec::len).sum::<usize>()+pending_count+incoming>MAX_ATTACHMENTS
            ||ready+owner_pending+incoming>MAX_CONVERSATION_ATTACHMENTS{return Err("O limite de anexos desta conversa ou das entregas pendentes foi atingido. Remove um anexo ou cancela uma entrega.".into());}
        let grant = self.drops.remove(token).ok_or(DROP_EXPIRED)?;
        let id = grant.owners.get(provider).ok_or(DROP_EXPIRED)?;
        if id != current || expected.is_some_and(|expected| expected != id) { return Err("A conversa mudou desde a entrega. Volta a arrastar o documento para esta conversa.".into()); }
        let paths=path.map(|p|vec![p.into()]).unwrap_or(grant.paths);
        let ticket = Ticket { provider: provider.into(), id: id.clone(), sequence: grant.sequence, drop_id: token.into(),paths,deadline:now+DROP_TTL };
        self.pending.insert((provider.into(),token.into()), ticket.clone());
        Ok(ticket)
    }

    fn failed(&mut self, ticket: &Ticket) {
        let key=(ticket.provider.clone(),ticket.drop_id.clone());
        if self.pending.get(&key) == Some(ticket) { self.pending.remove(&key); }
    }

    #[cfg(test)]
    fn finish(&mut self, ticket: &Ticket, current: &str, file: crate::files::DroppedFile) -> Result<Attachment, String> {
        Ok(self.finish_batch(ticket,current,vec![file])?.remove(0))
    }
    fn stage_batch(&self,ticket:&Ticket,current:&str,files:Vec<crate::files::DroppedFile>)->Result<Vec<Attachment>,String>{
        if Instant::now()>=ticket.deadline{return Err(COPY_DEADLINE.into());}
        if ticket.id!=current||self.pending.get(&(ticket.provider.clone(),ticket.drop_id.clone()))!=Some(ticket){return Err(CONTEXT_CHANGED.into());}
        let previous=self.files.get(&(ticket.provider.clone(),ticket.id.clone()));
        let count=self.files.values().map(Vec::len).sum::<usize>();
        let total=previous.into_iter().flatten().map(|r|r.attachment.file.size).chain(files.iter().map(|f|f.size)).try_fold(0u64,u64::checked_add);
        if files.is_empty()||files.len()>ticket.paths.len()||count.saturating_add(files.len())>MAX_ATTACHMENTS
            ||previous.map_or(0,Vec::len).saturating_add(files.len())>MAX_CONVERSATION_ATTACHMENTS||total.is_none_or(|n|n>MAX_CONVERSATION_BYTES){
            return Err("O conjunto excede o limite de anexos ou 100 MiB nesta conversa. As cópias ficaram na caixa de entrada; os anexos anteriores foram preservados.".into());
        }
        Ok(files.into_iter().map(|file|Attachment{id:crate::assistant_memory::new_id(),file,provider:ticket.provider.clone(),conversation_id:ticket.id.clone(),sequence:ticket.sequence}).collect())
    }
    fn publish_batch(&mut self,ticket:&Ticket,attachments:&[Attachment]){
        self.touch(&ticket.provider,&ticket.id);self.failed(ticket);let values=self.files.entry((ticket.provider.clone(),ticket.id.clone())).or_default();
        values.extend(attachments.iter().cloned().map(|attachment|Ready{attachment,drop_id:ticket.drop_id.clone()}));values.sort_by_key(|r|r.attachment.sequence);
    }
    fn commit_staged(&mut self,ticket:&Ticket,current:&str,added:&[Attachment],publish:impl FnOnce()->Result<(),String>)->Result<(),String>{
        self.stage_batch(ticket,current,added.iter().map(|a|a.file.clone()).collect())?;
        publish()?;self.publish_batch(ticket,added);Ok(())
    }
    #[cfg(any(test,not(target_os="linux")))]
    fn finish_batch(&mut self,ticket:&Ticket,current:&str,files:Vec<crate::files::DroppedFile>)->Result<Vec<Attachment>,String>{
        let result=self.stage_batch(ticket,current,files);match result{Ok(added)=>{self.publish_batch(ticket,&added);Ok(added)},Err(e)=>{self.failed(ticket);Err(e)}}
    }

    fn invalidate(&mut self, provider: Option<&str>) {
        if let Some(provider) = provider {
            if crate::pet_config::integration_pet(provider).is_some() {
                let revision = self.revisions.entry(provider.into()).or_default();
                *revision = revision.saturating_add(1);
            }
            self.selection_revisions.retain(|(p,_),_|p!=provider);
            self.pending.retain(|(p,_),_|p!=provider);
            self.files.retain(|(p, _), _| p != provider);
            self.drops.retain(|_, grant| { grant.owners.remove(provider); !grant.owners.is_empty() });
        } else {
            self.generation = self.generation.saturating_add(1);
            self.selection_revisions.clear();
            self.drops.clear();
            self.pending.clear();
            self.files.clear();
        }
    }

    fn context(&self, provider: &str, id: &str) -> Option<Attachment> {
        self.files.get(&(provider.into(), id.into())).and_then(|items|items.last()).map(|ready| ready.attachment.clone())
    }
    fn contexts(&self,provider:&str,id:&str)->Vec<Attachment>{
        self.files.get(&(provider.into(),id.into())).into_iter().flatten().map(|r|r.attachment.clone()).collect()
    }
    fn references(&self,provider:&str,cid:&str,files:&[crate::chat::ContextFile])->Result<Vec<Attachment>,String>{
        if files.is_empty()||files.len()>MAX_CONVERSATION_ATTACHMENTS{return Err(CONTEXT_REVOKED.into());}
        let values=self.files.get(&(provider.into(),cid.into())).ok_or(CONTEXT_REVOKED)?;let mut ids=std::collections::BTreeSet::new();
        files.iter().map(|f|{
            if !ids.insert(&f.id){return Err(CONTEXT_REVOKED.into());}
            values.iter().find(|r|r.attachment.id==f.id&&r.attachment.sequence==f.sequence&&r.attachment.file.path==f.path).map(|r|r.attachment.clone()).ok_or_else(||CONTEXT_REVOKED.into())
        }).collect()
    }

    #[cfg(test)]
    fn validate(&self, provider: &str, id: &str, path: &str, sequence: u64) -> Result<(), String> {
        if self.files.get(&(provider.into(), id.into())).is_some_and(|items|items.iter().any(|ready| {
            let a = &ready.attachment;
            a.provider == provider && a.conversation_id == id && a.file.path == path && a.sequence == sequence
        })) { Ok(()) } else { Err(CONTEXT_REVOKED.into()) }
    }

    fn clear(&mut self, provider: &str, id: &str, sequence: u64) -> bool {
        self.touch(provider,id);let key = (provider.into(), id.into());
        let Some(items)=self.files.get_mut(&key)else{return false};let before=items.len();items.retain(|r|r.attachment.sequence!=sequence);let changed=items.len()!=before;if items.is_empty(){self.files.remove(&key);}changed
    }
    fn remove(&mut self,provider:&str,cid:&str,id:&str)->bool{
        self.touch(provider,cid);let key=(provider.into(),cid.into());let Some(items)=self.files.get_mut(&key)else{return false};let before=items.len();items.retain(|r|r.attachment.id!=id);let changed=before!=items.len();if items.is_empty(){self.files.remove(&key);}changed
    }
    fn forget_conversation(&mut self,provider:&str,cid:&str)->Vec<Attachment>{
        self.touch(provider,cid);let files=self.files.remove(&(provider.into(),cid.into())).unwrap_or_default().into_iter().map(|r|r.attachment).collect();
        self.pending.retain(|_,ticket|ticket.provider!=provider||ticket.id!=cid);
        self.drops.retain(|_,grant|{if grant.owners.get(provider).is_some_and(|id|id==cid){grant.owners.remove(provider);}!grant.owners.is_empty()});files
    }

    fn cancel_drop(&mut self, provider: &str, token: &str) -> bool {
        let mut changed = false;
        if let Some(grant) = self.drops.get_mut(token) { changed |= grant.owners.remove(provider).is_some(); }
        if self.drops.get(token).is_some_and(|grant| grant.owners.is_empty()) { self.drops.remove(token); }
        if self.pending.remove(&(provider.into(),token.into())).is_some() {
            changed = true;
        }
        let owners=self.files.iter().filter(|((p,_),items)|p==provider&&items.iter().any(|r|r.drop_id==token)).map(|((p,cid),_)|(p.clone(),cid.clone())).collect::<Vec<_>>();
        for (p,cid) in owners{self.touch(&p,&cid);}
        self.files.retain(|(owner, _), items| {
            if owner==provider{let before=items.len();items.retain(|r|r.drop_id!=token);changed|=items.len()!=before;}
            !items.is_empty()
        });
        changed
    }
}

fn validate_paths(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() || paths.len() > MAX_PATHS || paths.iter().map(String::len).sum::<usize>() > MAX_DROP_BYTES || paths.iter().any(|text| {
        text.is_empty() || text.len() > MAX_PATH_BYTES || text.chars().any(char::is_control) || !Path::new(text).is_absolute()
            || Path::new(text).components().any(|c| matches!(c, Component::ParentDir))
            || text.split('/').any(|component| component == ".")
    }) { Err("A entrega contém um caminho inválido ou demasiados documentos.".into()) } else { Ok(()) }
}

fn configured_providers(settings: &crate::settings::Settings) -> Vec<String> {
    // The Linux card controller always addresses a configured pet explicitly,
    // even when the wandering companions or the System presentation are off.
    // Presentation preferences must never redirect a document to a global chat.
    if !cfg!(target_os = "linux") && !settings.companion_mode {
        let provider = if settings.chat_provider.is_empty() { crate::chat::ANTHROPIC } else { settings.chat_provider.as_str() };
        return crate::pet_config::integration_pet(provider).map(|_| vec![provider.into()]).unwrap_or_default();
    }
    CHAT_PROVIDERS.iter().copied().chain(crate::pet_config::service_ids()).filter_map(|provider| {
        let pet = crate::pet_config::integration_pet(provider)?;
        (pet == settings.main_pill || settings.active_integrations.iter().any(|id| id == pet)).then(|| provider.into())
    }).collect()
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(Mutex::default)
}

struct PendingCleanup(Ticket);
impl Drop for PendingCleanup{
    fn drop(&mut self){if let Ok(mut state)=store().lock(){state.failed(&self.0);}}
}

fn copy_slots()->&'static Arc<tokio::sync::Semaphore>{
    static SLOTS:OnceLock<Arc<tokio::sync::Semaphore>>=OnceLock::new();SLOTS.get_or_init(||Arc::new(tokio::sync::Semaphore::new(4)))
}
/// Queueing and blocking IO use the original ticket deadline. A timed-out or
/// dropped future cannot free a physical copy slot before its worker stops.
async fn bounded_copy<T:Send+'static>(deadline:Instant,copy:impl FnOnce()->T+Send+'static)->Result<T,String>{
    if Instant::now()>=deadline{return Err(COPY_DEADLINE.into());}
    let until=tokio::time::Instant::from_std(deadline);
    let permit=tokio::time::timeout_at(until,copy_slots().clone().acquire_owned()).await.map_err(|_|COPY_DEADLINE.to_string())?.map_err(|_|"O serviço de cópia foi interrompido.".to_string())?;
    let job=tauri::async_runtime::spawn_blocking(move||{
        let _physical_slot=permit;
        if Instant::now()>=deadline{return Err(COPY_DEADLINE.to_string());}Ok(copy())
    });
    let value=tokio::time::timeout_at(until,job).await.map_err(|_|COPY_DEADLINE.to_string())?.map_err(|_|"A receção dos documentos foi interrompida.".to_string())??;
    if Instant::now()>=deadline{return Err(COPY_DEADLINE.into());}Ok(value)
}
/// Stage/fsync metadata off the selection lock. A reset/cancel during staging
/// invalidates the same pending ticket; only rename + RAM publication use the
/// short commit fence. A published but unconfirmed write is never auto-retried.
async fn finish_ingested(chat:crate::chat::Chat,ticket:Ticket,files:Vec<crate::files::DroppedFile>)->Result<Vec<Attachment>,String>{
    let deadline=ticket.deadline;
    bounded_copy(deadline,move||{
        let current=chat.conversation_id(&ticket.provider)?;
        #[cfg(target_os="linux")]{
            let added=store().lock().unwrap().stage_batch(&ticket,&current,files)?;
            let mut saved=manifest::append(&ticket.provider,&ticket.id,&added)?;
            let current=chat.conversation_id(&ticket.provider)?;
            {
                let mut state=store().lock().unwrap();
                // Recheck count/bytes too: another drop may have completed
                // while our replacement was being fsynced.
                state.commit_staged(&ticket,&current,&added,||saved.publish())?;
            }
            saved.confirm()?;Ok(added)
        }
        #[cfg(not(target_os="linux"))]{store().lock().unwrap().finish_batch(&ticket,&current,files)}
    }).await?
}
fn persist_deselected(provider:&str,cid:&str,ids:&[String])->Result<(),String>{
    #[cfg(target_os="linux")]{manifest::deselect(provider,cid,ids)}
    #[cfg(not(target_os="linux"))]{let _=(provider,cid,ids);Ok(())}
}
fn changed(app: &AppHandle, provider: Option<&str>, id: Option<&str>) {
    let _ = app.emit("pet-document-context-changed", serde_json::json!({"provider":provider,"conversationId":id}));
}
pub fn invalidate(app: &AppHandle, provider: Option<&str>) {
    {let mut state=store().lock().unwrap();state.invalidate(provider);crate::pet_files::revoke_reviewed_reads(provider,None);}
    changed(app, provider, None);
}
pub(crate) fn forget_conversation(app:&AppHandle,provider:&str,cid:&str)->Result<(),String>{
    let files=store().lock().unwrap().forget_conversation(provider,cid);let paths=files.iter().map(|a|a.file.path.clone()).collect::<Vec<_>>();crate::pet_files::revoke_reviewed_reads(Some(provider),Some(&paths));
    changed(app,Some(provider),Some(cid));
    #[cfg(target_os="linux")]{manifest::deselect_owner(provider,cid)}
    #[cfg(not(target_os="linux"))]{Ok(())}
}

/// Called at the native drop signal, before requesting asynchronous URI data.
/// This reserves identity only, never path access or a token usable by IPC.
pub fn prepare_drop(app: &AppHandle) -> Result<NativeDrop, String> {
    let providers = {
        let shared = app.state::<crate::Shared>();
        // This runs inside GTK's completed-drop callback. A settings fsync
        // elsewhere must not make the whole desktop wait for our drop target.
        let settings = shared.settings.try_lock().map_err(|_| "As preferências do pet estão a ser guardadas. Aguarda um instante e volta a entregar o documento.".to_string())?;
        configured_providers(&settings)
    };
    if providers.is_empty() { return Err("Configura um pet com conversa antes de entregar o documento.".into()); }
    let capture = store().lock().unwrap().prepare_capture(&providers, Instant::now())?;
    let chat = app.state::<crate::chat::Chat>();
    // One nonblocking snapshot at the real OS drop, never owners discovered
    // later by an asynchronous worker after the selected conversation changed.
    let owners = chat.try_conversation_ids(&providers)?;
    Ok(NativeDrop { capture, owners })
}

/// Only the matching native data-received callback can complete this identity.
/// No grant is published if a reset/restore happened while GTK supplied URIs.
pub fn capture_drop(pending: NativeDrop, paths: &[String]) -> Result<String, String> {
    validate_paths(paths)?;
    let mut random = [0u8; 32];
    SystemRandom::new().fill(&mut random).map_err(|_| "Não foi possível autorizar esta entrega.")?;
    let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    store().lock().unwrap().finish_capture(pending.capture, token, paths.to_vec(), pending.owners, Instant::now())
}

pub async fn ingest(app: &AppHandle, provider: String, path: String, expected_id: Option<String>, drop_id: String) -> Result<Attachment, String> {
    let started = Instant::now();
    let configured = {
        let shared = app.state::<crate::Shared>();
        let settings = shared.settings.lock().unwrap();
        configured_providers(&settings).iter().any(|p| p == &provider)
    };
    if !configured { return Err("Este pet não está configurado para receber este documento.".into()); }
    let id = app.state::<crate::chat::Chat>().conversation_id(&provider)?;
    let claimed = store().lock().unwrap().claim(&drop_id, &provider, &path, expected_id.as_deref(), &id, Instant::now());
    let ticket = claimed.map_err(|error| {
        crate::log::line(format!("document ingest: grant rejected ({provider})")); error
    })?;
    let _cleanup=PendingCleanup(ticket.clone());
    crate::log::line(format!("document ingest: copy started ({provider})"));
    let copy_provider = provider.clone();
    #[cfg(target_os="linux")]
    let copy_cid=id.clone();
    let copied = bounded_copy(ticket.deadline,move || {
        #[cfg(target_os="linux")]
        manifest::ensure_not_purging(&copy_provider,&copy_cid)?;
        crate::pet_files::ingest(&copy_provider, &path)
    }).await;
    let file = match copied {
        Ok(Ok(file)) => file,
        result => {
            store().lock().unwrap().failed(&ticket);
            crate::log::line(format!("document ingest: copy failed ({provider}, {} ms)", started.elapsed().as_millis()));
            return Err(match result { Ok(Err(error))|Err(error) => error, _ => "A receção do documento foi interrompida.".into() });
        }
    };
    let current = match app.state::<crate::chat::Chat>().conversation_id(&provider) {
        Ok(current) => current,
        Err(error) => { store().lock().unwrap().failed(&ticket); return Err(error); }
    };
    let chat=app.state::<crate::chat::Chat>().inner().clone();
    if current!=ticket.id{return Err(CONTEXT_CHANGED.into());}
    let mut values=finish_ingested(chat,ticket.clone(),vec![file]).await?;
    let value=values.remove(0);
    // Operational phases only: never paths, file names, drop tokens or dialogue IDs.
    crate::log::line(format!("document ingest: selected ({provider}, {} ms)", started.elapsed().as_millis()));
    changed(app, Some(&provider), Some(&id));
    Ok(value)
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct IngestFailure{pub index:usize,pub name:String,pub error:String}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct IngestBatch{pub attachments:Vec<Attachment>,pub failures:Vec<IngestFailure>,pub error:Option<String>}

/// Claim the complete native drop. The caller supplies no file paths; every
/// copied path comes from the OS grant captured for this exact pet and CID.
#[tauri::command]
pub async fn pet_document_ingest_drop(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,drop_id:String)->Result<IngestBatch,String>{
    allowed(&window)?;
    let configured={let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().unwrap();configured_providers(&settings).contains(&provider)};
    if !configured{return Err("Este pet não está configurado para receber estes documentos.".into());}
    let chat=app.state::<crate::chat::Chat>().inner().clone();let current=chat.conversation_id(&provider)?;
    let ticket=store().lock().unwrap().claim_paths(&drop_id,&provider,None,Some(&conversation_id),&current,Instant::now())?;
    let _cleanup=PendingCleanup(ticket.clone());let copying=ticket.clone();let copy_chat=chat.clone();
    let (files,failures)=bounded_copy(ticket.deadline,move||{
        let mut files=vec![];let mut failures=vec![];let deadline=copying.deadline;
        #[cfg(target_os="linux")]
        let retention=manifest::ensure_not_purging(&copying.provider,&copying.id);
        #[cfg(not(target_os="linux"))]
        let retention:Result<(),String>=Ok(());
        for (index,path) in copying.paths.iter().enumerate(){
            let active=Instant::now()<deadline&&copy_chat.conversation_id(&copying.provider).is_ok_and(|id|id==copying.id)
                &&store().lock().unwrap().pending.get(&(copying.provider.clone(),copying.drop_id.clone()))==Some(&copying);
            let result=if let Err(error)=&retention{Err(error.clone())}else if active{crate::pet_files::ingest(&copying.provider,path)}else{Err("A entrega foi interrompida ou a conversa mudou.".into())};
            match result{Ok(file)=>files.push(file),Err(error)=>failures.push(IngestFailure{index,name:Path::new(path).file_name().unwrap_or_default().to_string_lossy().into_owned(),error:error.chars().take(500).collect()})}
        }
        (files,failures)
    }).await?;
    let current=chat.conversation_id(&provider)?;
    // Even a wholly failed batch cannot be re-targeted or restore a cancelled
    // ticket. Previous ready attachments stay intact on every failure path.
    if current!=ticket.id||store().lock().unwrap().pending.get(&(provider.clone(),drop_id.clone()))!=Some(&ticket){return Err(CONTEXT_CHANGED.into());}
    let mut error=None;
    let attachments=if files.is_empty(){store().lock().unwrap().failed(&ticket);vec![]}else{
        match finish_ingested(chat.clone(),ticket.clone(),files).await{Ok(added)=>added,Err(reason)=>{error=Some(reason);vec![]}}
    };
    changed(&app,Some(&provider),Some(&current));Ok(IngestBatch{attachments,failures,error})
}

fn allowed_label(label: &str) -> bool {
    [crate::island::WINDOW_LABEL, crate::pet_chat::LABEL, crate::companions::LABEL, crate::pet_control::LABEL].contains(&label)
}
pub(crate) fn allowed(window: &tauri::WebviewWindow) -> Result<(), String> {
    if allowed_label(window.label()) { Ok(()) }
    else { Err("Abre a conversa do pet para consultar o documento.".into()) }
}
pub fn context(provider: &str, id: &str) -> Option<Attachment> { store().lock().unwrap().context(provider, id) }
pub(crate) fn contexts(provider:&str,cid:&str)->Vec<Attachment>{store().lock().unwrap().contexts(provider,cid)}
pub(crate) fn collection_context(files:&[Attachment])->Option<crate::chat::ChatContext>{(!files.is_empty()).then(||crate::chat::ChatContext::Files{files:files.iter().map(Attachment::context_file).collect()})}
pub(crate) fn find(provider:&str,cid:&str,path:&str)->Option<Attachment>{contexts(provider,cid).into_iter().find(|a|a.file.path==path)}
pub(crate) fn reference(provider:&str,cid:&str,value:&crate::chat::ContextFile)->Result<Attachment,String>{Ok(store().lock().unwrap().references(provider,cid,std::slice::from_ref(value))?.remove(0))}
pub(crate) fn checked_context(provider:&str,cid:&str,context:crate::chat::ChatContext,sequence:Option<u64>)->Result<crate::chat::ChatContext,String>{
    let references=match &context{
        crate::chat::ChatContext::File{path,..}=>{let attachment=find(provider,cid,path).filter(|a|sequence.is_none_or(|s|s==a.sequence)).ok_or(CONTEXT_REVOKED)?;vec![attachment.context_file()]},
        crate::chat::ChatContext::Files{files}=>files.clone(),_=>return Ok(context),
    };
    let files=store().lock().unwrap().references(provider,cid,&references)?;
    crate::pet_files::check_existing_many(provider,&files.iter().map(|f|f.file.path.as_str()).collect::<Vec<_>>())?;
    Ok(collection_context(&files).unwrap())
}
pub(crate) fn validate_attachment(value:&Attachment)->Result<(),String>{
    store().lock().unwrap().references(&value.provider,&value.conversation_id,&[value.context_file()])?;
    crate::pet_files::check_existing_many(&value.provider,&[&value.file.path])
}
pub(crate) fn validate_attachments(values:&[Attachment])->Result<(),String>{
    let Some(first)=values.first()else{return Ok(())};
    if values.iter().any(|a|a.provider!=first.provider||a.conversation_id!=first.conversation_id){return Err(CONTEXT_REVOKED.into());}
    let references=values.iter().map(Attachment::context_file).collect::<Vec<_>>();
    store().lock().unwrap().references(&first.provider,&first.conversation_id,&references)?;
    crate::pet_files::check_existing_many(&first.provider,&values.iter().map(|a|a.file.path.as_str()).collect::<Vec<_>>())
}
#[tauri::command]
pub fn pet_document_contexts(window:tauri::WebviewWindow,provider:String,conversation_id:String)->Result<Vec<Attachment>,String>{allowed(&window)?;Ok(store().lock().unwrap().contexts(&provider,&conversation_id))}
#[tauri::command]
pub async fn pet_document_remove(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,attachment_id:String)->Result<(),String>{
    allowed(&window)?;if !crate::assistant_memory::valid_id(&attachment_id){return Err("O identificador do anexo é inválido.".into());}
    // Revoke immediately, even if durable metadata is unavailable. Never turn
    // a failed save into continued model read authority.
    let (removed,paths)={let mut state=store().lock().unwrap();let paths=state.contexts(&provider,&conversation_id).iter().filter(|a|a.id==attachment_id).map(|a|a.file.path.clone()).collect::<Vec<_>>();(state.remove(&provider,&conversation_id,&attachment_id),paths)};
    crate::pet_files::revoke_reviewed_reads(Some(&provider),Some(&paths));
    if removed{changed(&app,Some(&provider),Some(&conversation_id));}
    tauri::async_runtime::spawn_blocking(move||persist_deselected(&provider,&conversation_id,&[attachment_id])).await.map_err(|_|"Não foi possível confirmar a remoção do anexo. Volta a carregar a conversa.".to_string())?
}
#[tauri::command]
pub async fn pet_document_inventory(window:tauri::WebviewWindow,provider:String,conversation_id:String)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{tauri::async_runtime::spawn_blocking(move||{
        let active=contexts(&provider,&conversation_id);serde_json::to_value(manifest::inventory(&provider,&conversation_id,&active)?).map_err(|_|"Não foi possível listar os anexos.".to_string())
    }).await.map_err(|_|"A leitura dos anexos foi interrompida.".to_string())?}
    #[cfg(not(target_os="linux"))]{let _=(provider,conversation_id);Err("O registo persistente de anexos ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub async fn pet_document_review(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,attachment_ids:Vec<String>,expected_revision:String)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{bounded_copy(Instant::now()+DROP_TTL,move||serde_json::to_value(renewal::prepare(&app,provider,conversation_id,attachment_ids,expected_revision)?).map_err(|_|"Não foi possível preparar a revisão dos anexos.".to_string())).await?}
    #[cfg(not(target_os="linux"))]{let _=(app,provider,conversation_id,attachment_ids,expected_revision);Err("A recuperação de anexos ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub async fn pet_document_renew(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,review_id:String,consent:String)->Result<Vec<Attachment>,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{let decision:renewal::Consent=serde_json::from_value(serde_json::Value::String(consent)).map_err(|_|"Confirma explicitamente a leitura dos itens apresentados na revisão.".to_string())?;
        bounded_copy(Instant::now()+DROP_TTL,move||renewal::renew(&app,provider,conversation_id,review_id,decision)).await?}
    #[cfg(not(target_os="linux"))]{let _=(app,provider,conversation_id,review_id,consent);Err("A recuperação de anexos ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub fn pet_document_review_cancel(window:tauri::WebviewWindow,provider:String,conversation_id:String,review_id:String)->Result<(),String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{renewal::cancel(&provider,&conversation_id,&review_id);}
    #[cfg(not(target_os="linux"))]{let _=(provider,conversation_id,review_id);}
    Ok(())
}

#[tauri::command]
pub async fn pet_document_retention_inventory(app:AppHandle,window:tauri::WebviewWindow)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{let chat=app.state::<crate::chat::Chat>().inner().clone();bounded_copy(Instant::now()+DROP_TTL,move||serde_json::to_value(manifest::purge::native::inventory(&chat)?).map_err(|_|"Não foi possível listar as cópias guardadas.".into())).await?}
    #[cfg(not(target_os="linux"))]{let _=app;Err("A retenção de cópias ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub async fn pet_document_purge_review(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,attachment_ids:Vec<String>,expected_revision:String)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{let label=window.label().to_owned();let chat=app.state::<crate::chat::Chat>().inner().clone();let deadline=Instant::now()+DROP_TTL;
        bounded_copy(deadline,move||serde_json::to_value(manifest::purge::native::prepare(&chat,label,provider,conversation_id,attachment_ids,expected_revision,deadline)?).map_err(|_|"Não foi possível rever as cópias.".into())).await?}
    #[cfg(not(target_os="linux"))]{let _=(app,provider,conversation_id,attachment_ids,expected_revision);Err("A retenção de cópias ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub async fn pet_document_purge_recovery_review(app:AppHandle,window:tauri::WebviewWindow,recovery_id:String)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{let label=window.label().to_owned();let chat=app.state::<crate::chat::Chat>().inner().clone();let deadline=Instant::now()+DROP_TTL;
        bounded_copy(deadline,move||serde_json::to_value(manifest::purge::native::recovery_review(&chat,label,&recovery_id,deadline)?).map_err(|_|"Não foi possível rever a recuperação das cópias.".into())).await?}
    #[cfg(not(target_os="linux"))]{let _=(app,recovery_id);Err("A retenção de cópias ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub async fn pet_document_purge_commit(app:AppHandle,window:tauri::WebviewWindow,provider:String,conversation_id:String,review_id:String,consent:String)->Result<serde_json::Value,String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{let label=window.label().to_owned();let chat=app.state::<crate::chat::Chat>().inner().clone();
        bounded_copy(Instant::now()+DROP_TTL,move||{
            let receipt=manifest::purge::native::commit(&chat,&label,&provider,&conversation_id,&review_id,&consent)?;changed(&app,Some(&provider),Some(&conversation_id));
            serde_json::to_value(receipt).map_err(|_|"Não foi possível apresentar o resultado da retenção.".into())
        }).await?}
    #[cfg(not(target_os="linux"))]{let _=(app,provider,conversation_id,review_id,consent);Err("A retenção de cópias ainda não está disponível nesta plataforma.".into())}
}
#[tauri::command]
pub fn pet_document_purge_cancel(window:tauri::WebviewWindow,provider:String,conversation_id:String,review_id:String)->Result<(),String>{
    allowed(&window)?;
    #[cfg(target_os="linux")]{manifest::purge::native::cancel(window.label(),&provider,&conversation_id,&review_id);}
    #[cfg(not(target_os="linux"))]{let _=(provider,conversation_id,review_id);}Ok(())
}
#[tauri::command]
pub fn pet_document_context(window: tauri::WebviewWindow, provider: String, conversation_id: String) -> Result<Option<Attachment>, String> {
    allowed(&window)?;
    Ok(context(&provider, &conversation_id))
}
#[tauri::command]
pub async fn pet_document_clear(app: AppHandle, window: tauri::WebviewWindow, provider: String, conversation_id: String, sequence: u64) -> Result<(), String> {
    allowed(&window)?;
    let (ids,paths)={let mut state=store().lock().unwrap();let files=state.contexts(&provider,&conversation_id).into_iter().filter(|a|a.sequence==sequence).collect::<Vec<_>>();state.clear(&provider,&conversation_id,sequence);(files.iter().map(|a|a.id.clone()).collect::<Vec<_>>(),files.iter().map(|a|a.file.path.clone()).collect::<Vec<_>>())};
    crate::pet_files::revoke_reviewed_reads(Some(&provider),Some(&paths));
    if !ids.is_empty(){changed(&app,Some(&provider),Some(&conversation_id));}
    tauri::async_runtime::spawn_blocking(move||persist_deselected(&provider,&conversation_id,&ids)).await.map_err(|_|"Não foi possível confirmar a remoção dos anexos. Volta a carregar a conversa.".to_string())?
}
#[tauri::command]
pub async fn pet_document_cancel_drop(app: AppHandle, window: tauri::WebviewWindow, provider: String, drop_id: String) -> Result<(), String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let (cancelled,removed)={let mut state=store().lock().unwrap();
            let removed=state.files.iter().filter(|((p,_),_)|p==&provider).map(|((_,cid),items)|(cid.clone(),items.iter().filter(|r|r.drop_id==drop_id).map(|r|(r.attachment.id.clone(),r.attachment.file.path.clone())).collect::<Vec<_>>())).collect::<Vec<_>>();
            (state.cancel_drop(&provider,&drop_id),removed)
        };
        if cancelled {
            // A cancelled pending drop can leave an earlier attachment selected.
            // Refresh canonical context instead of telling every surface to clear it.
            let id = app.state::<crate::chat::Chat>().conversation_id(&provider)?;
            changed(&app, Some(&provider), Some(&id));
        }
        for (cid,files) in removed{let ids=files.iter().map(|a|a.0.clone()).collect::<Vec<_>>();let paths=files.iter().map(|a|a.1.clone()).collect::<Vec<_>>();crate::pet_files::revoke_reviewed_reads(Some(&provider),Some(&paths));persist_deselected(&provider,&cid,&ids)?;}
        Ok(())
    }).await.map_err(|_| "Não foi possível confirmar o cancelamento desta entrega.".to_string())?
}

#[cfg(test)]
pub(crate) mod fixtures{
    pub(crate) struct Files{pub root:std::path::PathBuf}
    impl Files{
        pub(crate) fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-prompt-fixtures-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&root).unwrap();Self{root}}
        pub(crate) fn write(&self,name:&str,bytes:&[u8]){let path=self.root.join(name);if let Some(parent)=path.parent(){std::fs::create_dir_all(parent).unwrap();}std::fs::write(path,bytes).unwrap();}
        pub(crate) fn context(&self,files:&[(&str,&str)])->crate::chat::ChatContext{crate::chat::ChatContext::Files{files:files.iter().enumerate().map(|(i,(path,name))|crate::chat::ContextFile{id:format!("fixture-{}",i+1),name:(*name).into(),path:self.root.join(path).to_string_lossy().into_owned(),sequence:(i+1)as u64}).collect()}}
    }
    impl Drop for Files{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(target_os="linux")]
    fn forgetting_one_conversation_keeps_other_cids_pending_drops_selections_and_fences(){
        let mut state=Store::default();let now=Instant::now();let a=ticket(&mut state,"a-ready","ollama","a",now);let removed=state.finish(&a,"a",file("a.txt")).unwrap();
        let b=ticket(&mut state,"b-ready","ollama","b",now);let kept=state.finish(&b,"b",file("b.txt")).unwrap();let b_fence=state.review_fence("ollama","b").unwrap();let a_fence=state.review_fence("ollama","a").unwrap();
        let _a=ticket(&mut state,"a-pending","ollama","a",now);let b_pending=ticket(&mut state,"b-pending","ollama","b",now);let foreign=ticket(&mut state,"other-pending","codex","a",now);
        capture(&mut state,"shared-drop",&["/source/shared.txt"],&[("ollama","a"),("openai","x")],now);
        assert_eq!(state.forget_conversation("ollama","a").iter().map(|a|&a.id).collect::<Vec<_>>(),vec![&removed.id]);
        assert!(state.contexts("ollama","a").is_empty());assert_eq!(state.contexts("ollama","b")[0].id,kept.id);assert!(!state.pending.contains_key(&("ollama".into(),"a-pending".into())));
        assert!(state.pending.get(&("ollama".into(),"b-pending".into()))==Some(&b_pending));assert!(state.pending.get(&("codex".into(),"other-pending".into()))==Some(&foreign));
        assert!(!state.drops["shared-drop"].owners.contains_key("ollama"));assert_eq!(state.drops["shared-drop"].owners["openai"],"x");
        assert_eq!(state.review_fence("ollama","b").unwrap(),b_fence);assert_ne!(state.review_fence("ollama","a").unwrap(),a_fence);
    }
    #[test]
    fn cancelled_staged_batch_cannot_publish_metadata_or_restore_selection(){
        let mut state=Store::default();let now=Instant::now();let t=ticket(&mut state,"cancel-before-commit","ollama","cid",now);
        let added=state.stage_batch(&t,"cid",vec![file("a.txt")]).unwrap();assert!(state.cancel_drop("ollama","cancel-before-commit"));
        let published=std::cell::Cell::new(false);assert!(state.commit_staged(&t,"cid",&added,||{published.set(true);Ok(())}).is_err());assert!(!published.get());assert!(state.contexts("ollama","cid").is_empty());
    }
    #[test]
    fn metadata_failure_preserves_existing_selection_and_pending_cleanup_is_scoped(){
        let mut state=Store::default();let now=Instant::now();let first=ticket(&mut state,"first","ollama","cid",now);let old=state.finish(&first,"cid",file("old.txt")).unwrap();
        let next=ticket(&mut state,"next","ollama","cid",now);let added=state.stage_batch(&next,"cid",vec![file("new.txt")]).unwrap();
        assert!(state.commit_staged(&next,"cid",&added,||Err("disk unavailable".into())).is_err());state.failed(&next);assert_eq!(state.contexts("ollama","cid").iter().map(|a|a.id.as_str()).collect::<Vec<_>>(),vec![old.id.as_str()]);
    }
    #[test]
    fn staged_items_commit_with_their_original_ids_and_expiry_never_calls_publish(){
        let mut state=Store::default();let now=Instant::now();let t=ticket(&mut state,"successful","ollama","cid",now);assert!(state.stage_batch(&t,"cid",vec![file("one.txt"),file("two.txt")]).is_err());
        let t=ticket(&mut state,"retry","ollama","cid",now);let added=state.stage_batch(&t,"cid",vec![file("one.txt")]).unwrap();state.commit_staged(&t,"cid",&added,||Ok(())).unwrap();assert_eq!(state.contexts("ollama","cid")[0].id,added[0].id);
        let mut t=ticket(&mut state,"expired","ollama","cid",now);t.deadline=Instant::now()-Duration::from_millis(1);state.pending.insert((t.provider.clone(),t.drop_id.clone()),t.clone());let published=std::cell::Cell::new(false);assert!(state.commit_staged(&t,"cid",&added,||{published.set(true);Ok(())}).is_err());assert!(!published.get());
    }
    #[test]
    fn only_pet_document_surfaces_can_access_attachment_context() {
        for label in [crate::island::WINDOW_LABEL, crate::pet_chat::LABEL, crate::companions::LABEL, crate::pet_control::LABEL] {
            assert!(allowed_label(label), "{label}");
        }
        for label in ["", "settings", "hub", "pet-manager", "untrusted", "pet-chat-extra"] {
            assert!(!allowed_label(label), "{label}");
        }
    }
    #[test]
    fn request_references_cannot_borrow_ids_paths_sequences_or_another_owner(){
        let mut s=Store::default();let now=Instant::now();let a=ticket(&mut s,"a","ollama","cid",now);let a=s.finish(&a,"cid",file("a.pdf")).unwrap();let reference=a.context_file();
        assert_eq!(s.references("ollama","cid",std::slice::from_ref(&reference)).unwrap()[0].id,a.id);
        assert!(s.references("codex","cid",std::slice::from_ref(&reference)).is_err());assert!(s.references("ollama","other",std::slice::from_ref(&reference)).is_err());assert!(s.references("ollama","cid",&[reference.clone(),reference.clone()]).is_err());
        for field in ["id","path","sequence"]{let mut changed=reference.clone();match field{"id"=>changed.id="another-id".into(),"path"=>changed.path="/private/not-dropped".into(),_=>changed.sequence+=1};assert!(s.references("ollama","cid",&[changed]).is_err());}
        let mut changed=reference;changed.name="caller tries to rename the copy".into();assert_eq!(s.references("ollama","cid",&[changed]).unwrap()[0].file.name,"a.pdf");
    }
    #[test]
    fn collection_ipc_requires_exact_item_fields_and_uses_ids_instead_of_bare_paths(){
        use serde_json::json;
        let valid=json!({"kind":"files","files":[{"id":"fixture-id","name":"file","path":"/inbox/file","sequence":1}]});assert!(serde_json::from_value::<crate::chat::ChatContext>(valid.clone()).is_ok());
        for key in ["id","name","path","sequence"]{let mut value=valid.clone();value["files"][0].as_object_mut().unwrap().remove(key);assert!(serde_json::from_value::<crate::chat::ChatContext>(value).is_err(),"{key}");}
        let mut value=valid;value["files"][0]["permission"]=json!("always");assert!(serde_json::from_value::<crate::chat::ChatContext>(value).is_err());
    }
    fn file(name: &str) -> crate::files::DroppedFile { crate::files::DroppedFile { name: name.into(), path: format!("/inbox/{name}"), size: 1, kind: "file".into(), entries: 1 } }
    fn capture(s: &mut Store, token: &str, paths: &[&str], owners: &[(&str, &str)], now: Instant) {
        let providers = owners.iter().map(|(p, _)| (*p).into()).collect::<Vec<_>>();
        let capture = s.prepare_capture(&providers, now).unwrap();
        s.finish_capture(capture, token.into(), paths.iter().map(|p| (*p).into()).collect(), owners.iter().map(|(p, id)| ((*p).into(), (*id).into())).collect(), now).unwrap();
    }
    fn ticket(s: &mut Store, token: &str, provider: &str, id: &str, now: Instant) -> Ticket {
        capture(s, token, &["/source/document.pdf"], &[(provider, id)], now);
        s.claim(token, provider, "/source/document.pdf", Some(id), id, now).unwrap()
    }

    #[test]
    fn reset_before_ingest_revokes_the_old_drop_even_after_restoring_the_same_id() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a.pdf"], &[("ollama", "old"), ("codex", "other")], now);
        s.invalidate(Some("ollama"));
        assert!(s.claim("a", "ollama", "/source/a.pdf", Some("old"), "old", now).is_err());
        assert!(s.claim("a", "codex", "/source/a.pdf", Some("other"), "other", now).is_ok());
    }

    #[test]
    fn provisional_mouse_release_snapshot_has_no_usable_grant_before_uri_delivery() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        assert!(s.drops.is_empty());
        assert!(s.claim("pending", "ollama", "/source/a.pdf", Some("at-release"), "at-release", now).is_err());
        s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now).unwrap();
        assert!(s.claim("completed", "ollama", "/source/a.pdf", Some("later-conversation"), "later-conversation", now).is_err());
        assert!(s.drops.is_empty());
        assert!(s.context("ollama", "later-conversation").is_none());
    }

    #[test]
    fn reset_after_mouse_release_before_uri_delivery_cannot_retarget_the_completed_drop() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        s.invalidate(Some("ollama"));
        assert!(s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now).is_err());
        assert!(s.drops.is_empty());
        assert!(s.pending.is_empty());
        assert!(s.context("ollama", "at-release").is_none());
    }

    #[test]
    fn delayed_uri_delivery_expires_from_mouse_release_without_publishing_a_grant() {
        let mut s = Store::default(); let now = Instant::now();
        let pending = s.prepare_capture(&["ollama".into()], now).unwrap();
        let owners = BTreeMap::from([("ollama".into(), "at-release".into())]);
        assert!(s.finish_capture(pending, "completed".into(), vec!["/source/a.pdf".into()], owners, now + DROP_TTL).is_err());
        assert!(s.drops.is_empty());
    }

    #[test]
    fn resets_racing_the_owner_snapshot_cannot_publish_a_revoked_owner() {
        let mut s = Store::default(); let now = Instant::now();
        let providers = vec!["ollama".into(), "codex".into()];
        let pending = s.prepare_capture(&providers, now).unwrap();
        s.invalidate(Some("ollama"));
        s.finish_capture(pending, "a".into(), vec!["/source/a".into()], BTreeMap::from([("ollama".into(), "same".into()), ("codex".into(), "other".into())]), now).unwrap();
        assert!(!s.drops["a"].owners.contains_key("ollama"));
        assert!(s.drops["a"].owners.contains_key("codex"));
        let pending = s.prepare_capture(&providers, now).unwrap();
        s.invalidate(None);
        assert!(s.finish_capture(pending, "b".into(), vec!["/source/a".into()], BTreeMap::from([("codex".into(), "other".into())]), now).is_err());
    }

    #[test]
    fn independently_completed_copies_append_in_capture_order_without_replacement() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let b = ticket(&mut s, "b", "ollama", "one", now);
        s.finish(&b, "one", file("new.pdf")).unwrap();
        s.finish(&a, "one", file("old.pdf")).unwrap();
        assert_eq!(s.contexts("ollama","one").iter().map(|a|a.file.name.as_str()).collect::<Vec<_>>(),["old.pdf","new.pdf"]);
        assert_eq!(s.context("ollama", "one").unwrap().file.name, "new.pdf");
    }

    #[test]
    fn reverse_ipc_order_preserves_both_drops_in_native_capture_order() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/same.pdf"], &[("ollama", "one")], now);
        capture(&mut s, "b", &["/source/same.pdf"], &[("ollama", "one")], now);
        let b = s.claim("b", "ollama", "/source/same.pdf", None, "one", now).unwrap();
        s.finish(&b, "one", file("b.pdf")).unwrap();
        let a=s.claim("a", "ollama", "/source/same.pdf", None, "one", now).unwrap();s.finish(&a,"one",file("a.pdf")).unwrap();
        assert_eq!(s.contexts("ollama","one").len(),2);
        assert_eq!(s.context("ollama", "one").unwrap().sequence, b.sequence);
    }

    #[test]
    fn the_same_path_needs_distinct_one_use_tokens() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        assert!(s.claim("a", "ollama", "/source/document.pdf", None, "one", now).is_err());
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(b.sequence > a.sequence);
        assert_ne!(b.drop_id, a.drop_id);
    }

    #[test]
    fn a_claim_consumes_the_entire_drop_and_requires_exact_path_owner_and_conversation() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a.pdf", "/source/b.pdf"], &[("ollama", "one")], now);
        for (provider, path) in [("codex", "/source/a.pdf"), ("ollama", "/source/../source/a.pdf"), ("ollama", "/private/key"), ("ollama", "/source//a.pdf")] {
            assert!(s.claim("a", provider, path, Some("one"), "one", now).is_err());
        }
        assert!(s.drops.contains_key("a"));
        s.claim("a", "ollama", "/source/a.pdf", Some("one"), "one", now).unwrap();
        assert!(s.claim("a", "ollama", "/source/b.pdf", None, "one", now).is_err());
        capture(&mut s, "b", &["/source/a.pdf"], &[("ollama", "one")], now);
        assert!(s.claim("b", "ollama", "/source/a.pdf", Some("other"), "one", now).is_err());
        assert!(!s.drops.contains_key("b"));
        capture(&mut s, "c", &["/source/a.pdf"], &[("ollama", "one")], now);
        assert!(s.claim("c", "ollama", "/source/a.pdf", None, "other", now).is_err());
    }

    #[test]
    fn expiry_and_limits_are_explicit_without_evicting_a_live_grant() {
        let mut s = Store::default(); let now = Instant::now();
        for i in 0..MAX_DROPS { capture(&mut s, &format!("{i}"), &["/source/a"], &[("ollama", "one")], now); }
        assert!(s.prepare_capture(&["ollama".into()], now).is_err());
        assert_eq!(s.drops.len(), MAX_DROPS);
        assert!(s.claim("0", "ollama", "/source/a", None, "one", now + DROP_TTL).is_err());
        assert!(s.drops.is_empty());
        assert!(s.prepare_capture(&["ollama".into()], now + DROP_TTL).is_ok());
        assert!(validate_paths(&vec!["/source/a".into(); MAX_PATHS + 1]).is_err());
        assert!(validate_paths(&[format!("/{}", "a".repeat(MAX_PATH_BYTES))]).is_err());
        assert!(validate_paths(&vec![format!("/{}", "a".repeat(MAX_PATH_BYTES - 1)); MAX_PATHS]).is_err());
        for path in ["relative", "/source/../a", "/source/./a", "/source/a\0b", "/source/a\nb"] { assert!(validate_paths(&[path.into()]).is_err()); }
    }

    #[test]
    fn failed_or_reset_copies_cannot_repopulate_context_and_leave_other_pets_alone() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let other = ticket(&mut s, "other", "codex", "two", now);
        s.finish(&other, "two", file("other.pdf")).unwrap();
        s.invalidate(Some("ollama"));
        assert!(s.finish(&a, "one", file("old.pdf")).is_err());
        assert!(s.context("ollama", "one").is_none());
        assert!(s.context("codex", "two").is_some());
        let b = ticket(&mut s, "b", "ollama", "one", now);
        let c = ticket(&mut s, "c", "ollama", "one", now);
        s.failed(&b);
        assert!(s.pending.get(&(c.provider.clone(),c.drop_id.clone())) == Some(&c));
        s.failed(&c);
        assert!(!s.pending.values().any(|t|t.provider=="ollama"));
    }

    #[test]
    fn consumption_and_validation_require_the_exact_current_attachment() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let a = s.finish(&a, "one", file("a.pdf")).unwrap();
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence).is_ok());
        assert!(s.validate("codex", "one", &a.file.path, a.sequence).is_err());
        assert!(s.validate("ollama", "other", &a.file.path, a.sequence).is_err());
        assert!(s.validate("ollama", "one", "/inbox/other.pdf", a.sequence).is_err());
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence + 1).is_err());
        assert!(!s.clear("codex", "one", a.sequence));
        assert!(!s.clear("ollama", "one", a.sequence + 1));
        let b = ticket(&mut s, "b", "ollama", "one", now);
        let b = s.finish(&b, "one", file("b.pdf")).unwrap();
        assert!(s.validate("ollama","one",&a.file.path,a.sequence).is_ok());
        assert!(s.clear("ollama", "one", a.sequence));
        assert!(s.validate("ollama","one",&b.file.path,b.sequence).is_ok());
        assert!(s.clear("ollama", "one", b.sequence));
        assert!(s.validate("ollama", "one", &b.file.path, b.sequence).is_err());
        let c = ticket(&mut s, "c", "ollama", "one", now);
        let c = s.finish(&c, "one", file("c.pdf")).unwrap();
        s.invalidate(Some("ollama"));
        assert!(s.validate("ollama", "one", &c.file.path, c.sequence).is_err());
    }

    #[test]
    fn cancel_before_ingest_revokes_only_that_owner_and_cannot_reappear() {
        let mut s = Store::default(); let now = Instant::now();
        capture(&mut s, "a", &["/source/a"], &[("ollama", "one"), ("codex", "other")], now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(!s.cancel_drop("ollama", "a"));
        assert!(s.claim("a", "ollama", "/source/a", None, "one", now).is_err());
        assert!(s.claim("a", "codex", "/source/a", None, "other", now).is_ok());
    }

    #[test]
    fn cancelling_an_in_flight_drop_preserves_the_previous_ready_attachment() {
        let mut s = Store::default(); let now = Instant::now();
        let old = ticket(&mut s, "old", "ollama", "one", now);
        let old = s.finish(&old, "one", file("old.pdf")).unwrap();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(s.finish(&a, "one", file("a.pdf")).is_err());
        assert_eq!(s.context("ollama", "one").unwrap().sequence, old.sequence);
    }

    #[test]
    fn cancelling_a_completed_drop_clears_it_without_cancelling_the_new_copy() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let a = s.finish(&a, "one", file("a.pdf")).unwrap();
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(s.validate("ollama", "one", &a.file.path, a.sequence).is_err());
        assert!(s.pending.get(&(b.provider.clone(),b.drop_id.clone())) == Some(&b));
        assert!(s.finish(&b, "one", file("b.pdf")).is_ok());
    }

    #[test]
    fn stale_cancellation_cannot_remove_a_newer_pending_or_ready_document() {
        let mut s = Store::default(); let now = Instant::now();
        let a = ticket(&mut s, "a", "ollama", "one", now);
        let b = ticket(&mut s, "b", "ollama", "one", now);
        assert!(s.cancel_drop("ollama", "a"));
        assert!(s.pending.get(&(b.provider.clone(),b.drop_id.clone())) == Some(&b));
        let b = s.finish(&b, "one", file("b.pdf")).unwrap();
        assert!(!s.cancel_drop("ollama", "a"));
        assert!(s.finish(&a, "one", file("a.pdf")).is_err());
        assert!(s.validate("ollama", "one", &b.file.path, b.sequence).is_ok());
    }

    #[test]
    fn a_batch_claim_uses_only_the_complete_native_drop_paths_and_original_owner(){
        let mut s=Store::default();let now=Instant::now();capture(&mut s,"batch",&["/source/a.pdf","/source/folder"],&[("ollama","one")],now);
        assert!(s.claim_paths("batch","codex",None,Some("one"),"one",now).is_err());assert!(s.drops.contains_key("batch"));
        let batch=s.claim_paths("batch","ollama",None,Some("one"),"one",now).unwrap();assert_eq!(batch.paths,["/source/a.pdf","/source/folder"]);assert!(s.claim_paths("batch","ollama",None,None,"one",now).is_err());
        let mut folder=file("folder");folder.kind="folder".into();folder.entries=3;
        let added=s.finish_batch(&batch,"one",vec![file("a.pdf"),folder]).unwrap();assert_eq!(added.len(),2);assert_ne!(added[0].id,added[1].id);assert!(added.iter().all(|a|crate::assistant_memory::valid_id(&a.id)));assert_eq!(added[1].file.kind,"folder");
        assert_eq!(s.contexts("ollama","one").len(),2);assert!(s.finish_batch(&batch,"one",vec![file("replay.pdf")]).is_err());
    }

    #[test]
    fn an_item_can_be_removed_without_clearing_other_items_in_the_same_drop(){
        let mut s=Store::default();let now=Instant::now();capture(&mut s,"batch",&["/source/a","/source/b"],&[("ollama","one")],now);
        let batch=s.claim_paths("batch","ollama",None,None,"one",now).unwrap();let added=s.finish_batch(&batch,"one",vec![file("a.pdf"),file("b.pdf")]).unwrap();
        assert!(!s.remove("codex","one",&added[0].id));assert!(!s.remove("ollama","other",&added[0].id));assert!(s.remove("ollama","one",&added[0].id));assert!(!s.remove("ollama","one",&added[0].id));
        assert!(s.validate("ollama","one",&added[0].file.path,added[0].sequence).is_err());assert!(s.validate("ollama","one",&added[1].file.path,added[1].sequence).is_ok());assert_eq!(s.contexts("ollama","one").len(),1);
    }

    #[test]
    fn appending_does_not_mutate_a_previously_captured_set_or_revoke_its_items(){
        let mut s=Store::default();let now=Instant::now();let a=ticket(&mut s,"a","ollama","one",now);let a=s.finish(&a,"one",file("a.pdf")).unwrap();let captured=s.contexts("ollama","one");
        let b=ticket(&mut s,"b","ollama","one",now);s.finish(&b,"one",file("b.pdf")).unwrap();assert_eq!(captured.len(),1);assert_eq!(captured[0].id,a.id);assert_eq!(s.contexts("ollama","one").len(),2);assert!(s.validate("ollama","one",&captured[0].file.path,captured[0].sequence).is_ok());
        assert!(s.remove("ollama","one",&a.id));assert_eq!(captured.len(),1);assert!(s.validate("ollama","one",&captured[0].file.path,captured[0].sequence).is_err());
    }

    #[test]
    fn cancelling_one_batch_preserves_other_pending_and_ready_batches(){
        let mut s=Store::default();let now=Instant::now();let a=ticket(&mut s,"a","ollama","one",now);let b=ticket(&mut s,"b","ollama","one",now);let c=ticket(&mut s,"c","codex","two",now);
        assert!(s.cancel_drop("ollama","a"));assert!(s.finish(&a,"one",file("a.pdf")).is_err());let b=s.finish(&b,"one",file("b.pdf")).unwrap();let c=s.finish(&c,"two",file("c.pdf")).unwrap();
        assert!(s.cancel_drop("ollama","b"));assert!(s.contexts("ollama","one").is_empty());assert!(s.validate("codex","two",&c.file.path,c.sequence).is_ok());assert_ne!(b.id,c.id);
    }

    #[test]
    fn conversation_capacity_reserves_pending_items_without_evicting_ready_items(){
        let mut s=Store::default();let now=Instant::now();let paths=(0..MAX_CONVERSATION_ATTACHMENTS).map(|i|format!("/source/{i}")).collect::<Vec<_>>();let refs=paths.iter().map(String::as_str).collect::<Vec<_>>();capture(&mut s,"full",&refs,&[("ollama","one")],now);
        let full=s.claim_paths("full","ollama",None,None,"one",now).unwrap();capture(&mut s,"extra",&["/source/extra"],&[("ollama","one")],now);assert!(s.claim_paths("extra","ollama",None,None,"one",now).is_err());assert!(s.drops.contains_key("extra"));
        let added=s.finish_batch(&full,"one",(0..MAX_CONVERSATION_ATTACHMENTS).map(|i|file(&format!("{i}.pdf"))).collect()).unwrap();assert!(s.claim_paths("extra","ollama",None,None,"one",now).is_err());assert_eq!(s.contexts("ollama","one").len(),MAX_CONVERSATION_ATTACHMENTS);
        assert!(s.remove("ollama","one",&added[0].id));let extra=s.claim_paths("extra","ollama",None,None,"one",now).unwrap();s.finish(&extra,"one",file("extra.pdf")).unwrap();assert_eq!(s.contexts("ollama","one").len(),MAX_CONVERSATION_ATTACHMENTS);
    }

    #[test]
    fn rejected_batch_size_preserves_previous_items_and_releases_its_reservation(){
        let mut s=Store::default();let now=Instant::now();let a=ticket(&mut s,"a","ollama","one",now);let a=s.finish(&a,"one",file("a.pdf")).unwrap();let b=ticket(&mut s,"b","ollama","one",now);let mut huge=file("huge.pdf");huge.size=MAX_CONVERSATION_BYTES;
        assert!(s.finish_batch(&b,"one",vec![huge]).is_err());assert_eq!(s.contexts("ollama","one").len(),1);assert!(s.validate("ollama","one",&a.file.path,a.sequence).is_ok());assert!(!s.pending.values().any(|t|t.drop_id=="b"));
        let c=ticket(&mut s,"c","ollama","one",now);assert!(s.finish_batch(&c,"one",vec![file("x"),file("y")]).is_err());assert_eq!(s.contexts("ollama","one").len(),1);
    }

    #[test]
    fn a_batch_cannot_publish_any_item_after_reset_or_in_another_conversation(){
        let mut s=Store::default();let now=Instant::now();capture(&mut s,"a",&["/source/a","/source/b"],&[("ollama","one")],now);let a=s.claim_paths("a","ollama",None,None,"one",now).unwrap();
        assert!(s.finish_batch(&a,"other",vec![file("a"),file("b")]).is_err());assert!(s.finish_batch(&a,"one",vec![file("a"),file("b")]).is_err());assert!(s.contexts("ollama","one").is_empty());assert!(s.contexts("ollama","other").is_empty());
        let b=ticket(&mut s,"b","ollama","one",now);s.invalidate(Some("ollama"));assert!(s.finish(&b,"one",file("b")).is_err());
    }

    #[test]
    fn global_attachment_capacity_is_counted_per_item_across_owners(){
        let mut s=Store::default();let now=Instant::now();let paths=(0..32).map(|i|format!("/source/{i}")).collect::<Vec<_>>();let refs=paths.iter().map(String::as_str).collect::<Vec<_>>();
        for (i,provider) in ["codex","copilot","antigravity","ollama"].iter().enumerate(){let token=format!("batch{i}");capture(&mut s,&token,&refs,&[(provider,"cid")],now);let ticket=s.claim_paths(&token,provider,None,None,"cid",now).unwrap();s.finish_batch(&ticket,"cid",(0..32).map(|i|file(&format!("{i}.pdf"))).collect()).unwrap();}
        capture(&mut s,"extra",&["/source/extra"],&[("lmstudio","cid")],now);assert!(s.claim_paths("extra","lmstudio",None,None,"cid",now).is_err());assert_eq!(s.files.values().map(Vec::len).sum::<usize>(),MAX_ATTACHMENTS);
    }

    #[test]
    fn pending_cleanup_releases_only_its_own_drop(){
        let provider=format!("fixture-{}",crate::assistant_memory::new_id());let now=Instant::now();let (a,b)={let mut s=store().lock().unwrap();(ticket(&mut s,"a",&provider,"cid",now),ticket(&mut s,"b",&provider,"cid",now))};
        drop(PendingCleanup(a.clone()));let mut s=store().lock().unwrap();assert!(!s.pending.values().any(|t|t==&a));assert!(s.pending.values().any(|t|t==&b));s.failed(&b);
    }

    #[test]
    fn partial_batch_success_appends_only_received_items_without_replacing_old_ones(){
        let mut s=Store::default();let now=Instant::now();let old=ticket(&mut s,"old","ollama","cid",now);let old=s.finish(&old,"cid",file("old.pdf")).unwrap();capture(&mut s,"batch",&["/source/good","/source/fails"],&[("ollama","cid")],now);
        let batch=s.claim_paths("batch","ollama",None,None,"cid",now).unwrap();let received=s.finish_batch(&batch,"cid",vec![file("good.pdf")]).unwrap();assert_eq!(received.len(),1);assert_eq!(s.contexts("ollama","cid").len(),2);assert!(s.validate("ollama","cid",&old.file.path,old.sequence).is_ok());
    }

    #[test]
    fn expired_copy_reservations_release_and_late_results_cannot_publish(){
        let mut s=Store::default();let earlier=Instant::now()-DROP_TTL-Duration::from_secs(1);let expired=ticket(&mut s,"expired","ollama","cid",earlier);
        assert!(s.finish(&expired,"cid",file("late.pdf")).is_err());assert!(s.contexts("ollama","cid").is_empty());
        let expired=ticket(&mut s,"expired-again","ollama","cid",earlier);let current=ticket(&mut s,"current","ollama","cid",Instant::now());assert!(!s.pending.values().any(|t|t==&expired));assert!(s.finish(&current,"cid",file("current.pdf")).is_ok());
    }

    pub(super) static COPY_TEST_SERIAL:Mutex<()>=Mutex::new(());
    #[tokio::test]
    async fn already_expired_copy_never_enters_the_queue_or_worker(){
        use std::sync::atomic::{AtomicBool,Ordering};
        let _serial=COPY_TEST_SERIAL.lock().unwrap();let ran=Arc::new(AtomicBool::new(false));let observed=ran.clone();
        assert!(bounded_copy(Instant::now()-Duration::from_secs(1),move||observed.store(true,Ordering::Release)).await.is_err());assert!(!ran.load(Ordering::Acquire));assert_eq!(copy_slots().available_permits(),4);
    }
    async fn interrupted_copy_keeps_its_physical_slot(cancel:bool){
        let _serial=COPY_TEST_SERIAL.lock().unwrap();assert_eq!(copy_slots().available_permits(),4);
        let provider=format!("fixture-copy-{}",crate::assistant_memory::new_id());let ticket={let mut s=store().lock().unwrap();ticket(&mut s,"copy",&provider,"cid",Instant::now())};let captured=ticket.clone();
        let (started_tx,started_rx)=tokio::sync::oneshot::channel();let (release_tx,release_rx)=std::sync::mpsc::channel();
        struct Release(Option<std::sync::mpsc::Sender<()>>);impl Drop for Release{fn drop(&mut self){if let Some(tx)=self.0.take(){let _=tx.send(());}}}
        let release=Release(Some(release_tx));let deadline=Instant::now()+Duration::from_secs(if cancel{5}else{2});
        let job=tokio::spawn(async move{let _cleanup=PendingCleanup(ticket);bounded_copy(deadline,move||{let _=started_tx.send(());release_rx.recv_timeout(Duration::from_secs(10)).unwrap();42usize}).await});
        tokio::time::timeout(Duration::from_secs(3),started_rx).await.unwrap().unwrap();assert_eq!(copy_slots().available_permits(),3);
        if cancel{job.abort();assert!(job.await.unwrap_err().is_cancelled());}else{assert!(job.await.unwrap().is_err());}
        assert_eq!(copy_slots().available_permits(),3,"only the still-running worker may release its slot");assert!(!store().lock().unwrap().pending.values().any(|t|t==&captured));
        drop(release);let until=Instant::now()+Duration::from_secs(3);while copy_slots().available_permits()!=4{assert!(Instant::now()<until);tokio::time::sleep(Duration::from_millis(5)).await;}
    }
    #[tokio::test]
    async fn timed_out_copy_releases_its_ticket_but_holds_the_worker_slot_until_exit(){interrupted_copy_keeps_its_physical_slot(false).await;}
    #[tokio::test]
    async fn cancelled_copy_releases_its_ticket_but_holds_the_worker_slot_until_exit(){interrupted_copy_keeps_its_physical_slot(true).await;}

    #[test]
    fn configured_owners_use_logical_service_and_api_ids() {
        let mut settings = crate::settings::Settings::default();
        settings.companion_mode = true;
        settings.main_pill = "ai_openai".into();
        settings.active_integrations = vec!["integration_spotify".into(), "agent_codex".into()];
        assert_eq!(configured_providers(&settings), vec!["codex", "openai", "integration_spotify"]);
        settings.companion_mode = false;
        settings.chat_provider.clear();
        #[cfg(target_os="linux")]{
            assert_eq!(configured_providers(&settings), vec!["codex", "openai", "integration_spotify"]);
            settings.pet_system_center=false;
            settings.pet_control_bar=false;
            settings.chat_provider="unknown".into();
            assert_eq!(configured_providers(&settings), vec!["codex", "openai", "integration_spotify"]);
            settings.main_pill="unknown".into();settings.active_integrations.clear();
            assert!(configured_providers(&settings).is_empty());
        }
        #[cfg(not(target_os="linux"))]{
        assert_eq!(configured_providers(&settings), vec!["anthropic"]);
        settings.chat_provider = "unknown".into();
        assert!(configured_providers(&settings).is_empty());
        }
    }
}
