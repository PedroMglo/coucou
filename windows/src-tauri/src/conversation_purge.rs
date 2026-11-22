//! Native review only. These commands are never registered as model tools.
use crate::{chat::{Chat,retention::{Target,RecoverySummary,Receipt}},pet_documents};
use serde::Serialize;
use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::{Duration,Instant}};
use tauri::{AppHandle,Emitter,State};
const TTL:Duration=Duration::from_secs(120);
const INVALID:&str="A revisão da eliminação expirou ou já não corresponde à janela/conversa. Volta a rever.";
enum Payload{Conversation(Target),Recovery(RecoverySummary)}
impl Payload{
    fn target(&self)->&Target{match self{Self::Conversation(t)=>t,Self::Recovery(r)=>&r.target}}
    fn kind(&self)->&'static str{match self{Self::Conversation(_)=>"conversation",Self::Recovery(_)=>"recovery"}}
}
struct Review{window:String,deadline:Instant,payload:Payload}
#[derive(Default)]
struct Reviews(BTreeMap<String,Review>);
impl Reviews{
    fn insert(&mut self,window:String,payload:Payload,now:Instant)->Result<Preview,String>{
        self.0.retain(|_,r|r.deadline>now);if self.0.len()>=32{return Err("Fecha uma revisão antiga antes de iniciar outra eliminação.".into());}
        use ring::rand::SecureRandom;let mut nonce=[0u8;32];ring::rand::SystemRandom::new().fill(&mut nonce).map_err(|_|INVALID)?;let id=nonce.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let t=payload.target();let preview=Preview{review_id:id.clone(),kind:payload.kind().into(),provider:t.provider.clone(),conversation_id:t.conversation_id.clone(),title:t.title.clone(),message_count:t.message_count,task_count:t.task_count,attachment_count:t.attachment_count,current:t.current,expires_in_seconds:TTL.as_secs()};
        self.0.insert(id,Review{window,deadline:now+TTL,payload});Ok(preview)
    }
    fn take(&mut self,id:&str,window:&str,provider:&str,cid:&str,now:Instant)->Result<Review,String>{
        let review=self.0.remove(id).ok_or(INVALID)?;let target=review.payload.target();
        if review.deadline<=now||review.window!=window||target.provider!=provider||target.conversation_id!=cid{return Err(INVALID.into());}Ok(review)
    }
    fn cancel(&mut self,id:&str,window:&str,provider:&str,cid:&str){
        if self.0.get(id).is_some_and(|r|r.window==window&&r.payload.target().provider==provider&&r.payload.target().conversation_id==cid){self.0.remove(id);}
    }
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Preview{review_id:String,kind:String,provider:String,conversation_id:String,title:String,message_count:usize,task_count:usize,attachment_count:usize,current:bool,expires_in_seconds:u64}
#[derive(Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
pub(crate) enum Outcome{
    Conversation{receipt:Receipt,#[serde(rename="copySelectionCleared")]copy_selection_cleared:bool},
    Recovery{#[serde(rename="recoveryId")]recovery_id:String,finished:bool,restored:bool},
}
fn allowed(window:&tauri::WebviewWindow)->Result<String,String>{pet_documents::allowed(window)?;Ok(window.label().into())}
#[tauri::command]
pub(crate) async fn chat_purge_review(window:tauri::WebviewWindow,chat:State<'_,Chat>,provider:String,conversation_id:String)->Result<Preview,String>{
    let label=allowed(&window)?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
        let target=chat.purge_target(&provider,&conversation_id)?;reviews().lock().unwrap().insert(label,Payload::Conversation(target),Instant::now())
    }).await.map_err(|_|INVALID.to_string())?
}
#[tauri::command]
pub(crate) async fn chat_purge_recovery_review(window:tauri::WebviewWindow,chat:State<'_,Chat>,recovery_id:String)->Result<Preview,String>{
    let label=allowed(&window)?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
        let target=chat.purge_recovery_target(&recovery_id)?;reviews().lock().unwrap().insert(label,Payload::Recovery(target),Instant::now())
    }).await.map_err(|_|INVALID.to_string())?
}
#[tauri::command]
pub(crate) async fn chat_purge_recoveries(window:tauri::WebviewWindow,chat:State<'_,Chat>)->Result<Vec<RecoverySummary>,String>{
    allowed(&window)?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||chat.purge_recovery_inventory()).await.map_err(|_|INVALID.to_string())?
}
fn commit(review:Review,chat:&Chat,consent:&str)->Result<Outcome,String>{
    match review.payload{
        Payload::Conversation(target)if consent=="deleteLocalConversation"=>Ok(Outcome::Conversation{receipt:chat.purge_local_conversation(&target,review.deadline)?,copy_selection_cleared:false}),
        Payload::Recovery(target)if consent=="deleteRecoveryCopy"=>{let finished=chat.finish_purge_recovery(&target,review.deadline)?;Ok(Outcome::Recovery{recovery_id:target.id,finished,restored:false})},
        Payload::Recovery(target)if consent=="restoreRecoveryConversation"=>{let finished=chat.restore_purge_recovery(&target,review.deadline)?;Ok(Outcome::Recovery{recovery_id:target.id,finished,restored:true})},
        _=>Err("Confirma explicitamente a operação apresentada na revisão nativa.".into()),
    }
}
#[tauri::command]
pub(crate) async fn chat_purge_commit(app:AppHandle,window:tauri::WebviewWindow,chat:State<'_,Chat>,provider:String,conversation_id:String,review_id:String,consent:String)->Result<Outcome,String>{
    let label=allowed(&window)?;let review=reviews().lock().unwrap().take(&review_id,&label,&provider,&conversation_id,Instant::now())?;let chat=chat.inner().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let mut result=commit(review,&chat,&consent)?;
        if let Outcome::Conversation{receipt,copy_selection_cleared}=&mut result{
            *copy_selection_cleared=pet_documents::forget_conversation(&app,&receipt.provider,&receipt.removed_conversation_id).is_ok();
            let _=app.emit("assistant-history-changed",&receipt.provider);
        }else{let _=app.emit("assistant-history-changed",&provider);}Ok(result)
    }).await.map_err(|_|INVALID.to_string())?
}
#[tauri::command]
pub(crate) fn chat_purge_cancel(window:tauri::WebviewWindow,provider:String,conversation_id:String,review_id:String)->Result<(),String>{
    let label=allowed(&window)?;reviews().lock().unwrap().cancel(&review_id,&label,&provider,&conversation_id);Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    fn target()->Target{Target{provider:"codex".into(),conversation_id:"original".into(),revision:"revision".into(),current:true,title:"title".into(),message_count:2,task_count:0,attachment_count:1}}
    #[test]
    fn review_tokens_bind_window_owner_expiry_and_one_use(){
        let now=Instant::now();let mut cache=Reviews::default();let preview=cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();assert_eq!(preview.review_id.len(),64);assert!(cache.take(&preview.review_id,"other-window","codex","original",now).is_err());assert!(cache.take(&preview.review_id,"pet-chat","codex","original",now).is_err());
        let preview=cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();assert!(cache.take(&preview.review_id,"pet-chat","ollama","original",now).is_err());
        let preview=cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();assert!(cache.take(&preview.review_id,"pet-chat","codex","original",now+TTL).is_err());
        let preview=cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();assert!(cache.take(&preview.review_id,"pet-chat","codex","original",now).is_ok());assert!(cache.take(&preview.review_id,"pet-chat","codex","original",now).is_err());
    }
    #[test]
    fn review_cache_is_bounded_and_cancellation_cannot_remove_another_window_scope(){
        let now=Instant::now();let mut cache=Reviews::default();let preview=cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();cache.cancel(&preview.review_id,"wrong","codex","original");assert_eq!(cache.0.len(),1);cache.cancel(&preview.review_id,"pet-chat","codex","original");assert!(cache.0.is_empty());
        for _ in 0..32{cache.insert("pet-chat".into(),Payload::Conversation(target()),now).unwrap();}assert!(cache.insert("pet-chat".into(),Payload::Conversation(target()),now).is_err());assert!(cache.insert("pet-chat".into(),Payload::Conversation(target()),now+TTL).is_ok());
    }
    #[test]
    fn confirmation_kind_is_explicit_before_any_library_work(){
        let review=Review{window:"pet-chat".into(),deadline:Instant::now()+TTL,payload:Payload::Conversation(target())};assert!(commit(review,&Chat::default(),"deleteRecoveryCopy").is_err());
    }
}
