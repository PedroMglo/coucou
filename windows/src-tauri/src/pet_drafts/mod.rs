//! Native-user draft storage only. No model, runtime or provider caller route.
mod model;
mod store;
mod recovery;
mod archives;
mod reconcile;
use model::{Import,ImportReceipt,Record,Target,Write};
fn allowed(label:&str)->Result<(),String>{if [crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_chat::LABEL].contains(&label){Ok(())}else{Err("blocked".into())}}
fn path()->std::path::PathBuf{crate::platform::data_dir().join("assistant/session-drafts.json")}
fn current()->Result<(),&'static str>{crate::pet_data::bootstrap::write_allowed().map_err(|_|"unavailable")}
fn retain(chat:&crate::chat::Chat,target:&Target)->Result<(),&'static str>{
 if target.origin==model::Origin::External{return Ok(());}
 let pet=String::from_utf16(&target.pet_id).map_err(|_|"invalidDraft")?;
 let provider=String::from_utf16(&target.provider).map_err(|_|"invalidDraft")?;
 if crate::pet_config::integration_pet(&provider)!=Some(pet.as_str()){return Err("invalidDraft");}
 target.validate()?;if target.origin==model::Origin::Unbound{return Ok(());}
 let cid=String::from_utf16(&target.session_id).map_err(|_|"invalidDraft")?;
 if target.conversation_id.as_ref().is_some_and(|id|id!=&target.session_id){return Err("invalidDraft");}
 // Pin only the original existing CID. No restore, creation or sending, and no
 // attempt to turn absent legacy draft destinations into active conversations.
 chat.retain_public_conversation(&provider,&cid).map_err(|_|"unavailable")
}
/// Called under Chat's mutex: no Chat/configuration lock or mutation here.
/// Unknown draft state conservatively retains the original conversation.
pub(crate)fn retains_conversation(library:&std::path::Path,provider:&str,cid:&str)->bool{
 let provider:Vec<u16>=provider.encode_utf16().collect();let cid:Vec<u16>=cid.encode_utf16().collect();
 match crate::private_registry::load_at::<model::Registry>(&library.with_file_name("session-drafts.json")){
  Ok(snapshot)=>{
   let matches=|r:&Record|r.target.origin==model::Origin::Mochi&&r.target.provider==provider&&r.target.session_id==cid;
   if !snapshot.unchanged()||snapshot.registry.records.iter().any(matches){return true;}
   for reference in &snapshot.registry.retained_sources{match archives::read(&library.with_file_name("session-drafts.json"),reference){Ok(archive)=>{if archive.registry.source.as_ref().is_some_and(|s|s.records.iter().any(matches))||!archive.unchanged(){return true;}},Err(_)=>return true}}
   !snapshot.unchanged()
  },Err(_)=>true,
 }
}
fn message(code:&str)->String{match code{"invalidDraft"|"invalidDraftRegistry"|"invalidDraftImport"=>"invalidData","draftImportChanged"|"draftImportConflict"=>"migrationConflict","draftLimit"|"registryLimit"=>"limit","draftDestinationOccupied"=>"destinationOccupied","draftReviewExpired"|"draftReviewChanged"=>"reviewChanged","blocked"|"registryBusy"|"registryConflict"|"registryChanged"=>"blocked",_=>"unavailable"}.into()}
#[tauri::command]
pub(crate)async fn pet_draft_read(window:tauri::WebviewWindow,target:Target)->Result<Option<Record>,String>{
 allowed(window.label())?;tauri::async_runtime::spawn_blocking(move||store::read(&path(),target,current).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_write(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,target:Target,expected_revision:Option<String>,text_units:Vec<u16>)->Result<Write,String>{
 allowed(window.label())?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||store::write_prepared(&path(),target.clone(),expected_revision,text_units,current,||retain(&chat,&target)).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_import(window:tauri::WebviewWindow,batch:Import)->Result<ImportReceipt,String>{
 allowed(window.label())?;tauri::async_runtime::spawn_blocking(move||store::import(&path(),batch,current).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|v|v.as_secs()).unwrap_or(0)}
#[tauri::command]
pub(crate)async fn pet_draft_list(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,owner:recovery::Owner,offset:usize,limit:usize,expected_revision:Option<String>,source_archive:Option<model::Retained>)->Result<recovery::Page,String>{
 allowed(window.label())?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||if let Some(source)=source_archive{recovery::list_source(&path(),owner,offset,limit,expected_revision.as_deref(),Some(&source),current,|t|recovery::availability(&chat,t)).map_err(message)}else{recovery::list(&path(),owner,offset,limit,expected_revision.as_deref(),current,|t|recovery::availability(&chat,t)).map_err(message)}).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_recovery_review(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,request:recovery::Request)->Result<recovery::Review,String>{
 allowed(window.label())?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||recovery::review(&path(),request,now(),current,|t|if recovery::availability(&chat,t)==recovery::Availability::Existing{Ok(())}else{Err("unavailable")}).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_recovery_copy(window:tauri::WebviewWindow,chat:tauri::State<'_,crate::chat::Chat>,review:recovery::Review,approved:bool)->Result<Record,String>{
 allowed(window.label())?;let chat=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||recovery::copy(&path(),review,approved,now,current,|t|retain(&chat,t)).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[cfg(test)]mod tests;
#[cfg(test)]mod recovery_tests;
#[cfg(test)]mod reconcile_tests;
#[tauri::command]
pub(crate)async fn pet_draft_sources(window:tauri::WebviewWindow)->Result<Vec<archives::SourceInfo>,String>{
 allowed(window.label())?;tauri::async_runtime::spawn_blocking(move||archives::list(&path(),current).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_reconcile_review(window:tauri::WebviewWindow,batch:Import,policy:reconcile::Policy,expected_revision:Option<String>,expires_at:Option<u64>,offset:usize)->Result<reconcile::Page,String>{
 allowed(window.label())?;tauri::async_runtime::spawn_blocking(move||reconcile::review(&path(),&batch,policy,expected_revision.as_deref(),expires_at,offset,now(),current).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
#[tauri::command]
pub(crate)async fn pet_draft_reconcile_commit(window:tauri::WebviewWindow,batch:Import,review:reconcile::Review,approved:bool)->Result<ImportReceipt,String>{
 allowed(window.label())?;tauri::async_runtime::spawn_blocking(move||reconcile::commit(&path(),&batch,review,approved,now,current).map_err(message)).await.map_err(|_|"unavailable".to_string())?
}
