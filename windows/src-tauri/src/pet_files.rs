//! Explicit dropped-file handoff. Integration grants do not replace native CLI policies.
use std::{collections::BTreeMap,path::{Path,PathBuf},sync::{Mutex,OnceLock}};
use tauri::{AppHandle,Manager};
fn grants()->&'static Mutex<BTreeMap<PathBuf,String>>{static GRANTS:OnceLock<Mutex<BTreeMap<PathBuf,String>>>=OnceLock::new();GRANTS.get_or_init(||Mutex::new(BTreeMap::new()))}
fn analysis_supported(provider:&str,path:&Path)->bool{
 crate::pet_documents::policy::can_analyze(provider,&path.to_string_lossy(),std::fs::symlink_metadata(path).is_ok_and(|m|m.is_dir()))
}
pub fn ingest(provider:&str,source:&str)->Result<crate::files::DroppedFile,String>{
 crate::pet_config::require_active_provider(provider)?;
 let pet=crate::pet_config::integration_pet(provider).ok_or("Este pet não recebe documentos.")?;
 let config=crate::pet_config::read()?;let p=config.permissions(pet)?;
 let folder=std::fs::symlink_metadata(source).is_ok_and(|m|m.is_dir());
 let analysis=p.documents && analysis_supported(provider,Path::new(source));
 if !analysis && (!p.email_drafts || folder){return Err(format!("{} não tem nenhuma funcionalidade ativa para este {}. Revê os acessos deste pet.",config.pets[pet].name,if folder{"tipo de pasta"}else{"documento"}));}
 let file=if folder{crate::document_folder::ingest(source)?}else{crate::files::ingest(source)?};
 let mut g=grants().lock().unwrap();g.retain(|path,_|path.exists());
 g.insert(PathBuf::from(&file.path),pet.into());Ok(file)
}
/// Captured document requests never recreate defaults when a grant/config
/// disappears. Read the permission once for the complete captured set.
pub(crate) fn check_existing_many(provider:&str,paths:&[&str])->Result<(),String>{
 let pet=crate::pet_config::integration_pet(provider).ok_or("Este pet não recebe documentos.")?;crate::pet_config::require_active(pet)?;
 let config=crate::pet_config::read_existing()?;
 if !config.permissions(pet)?.documents{return Err("Este pet não tem acesso aos documentos desta conversa.".into());}
 for path in paths{if !analysis_supported(provider,Path::new(path)){return Err("Um dos formatos não é suportado por este pet.".into());}read_owned(pet,path)?;}Ok(())
}
fn owned(pet:&str,path:&str)->Result<(),String>{
 let path=Path::new(path).canonicalize().map_err(|_|"O documento já não está disponível. Volta a arrastá-lo para o pet.")?;
 if grants().lock().unwrap().get(&path).is_some_and(|owner|owner==pet){Ok(())}else{Err("Este documento não foi entregue a este pet. Arrasta-o para o pet que o deve receber.".into())}
}
// Renewing document reads is deliberately separate from the native drop grant
// used by mail drafts. It cannot authorize email export or original source IO.
fn read_grants()->&'static Mutex<BTreeMap<(PathBuf,String),String>>{static READ:OnceLock<Mutex<BTreeMap<(PathBuf,String),String>>>=OnceLock::new();READ.get_or_init(||Mutex::new(BTreeMap::new()))}
fn read_owned(pet:&str,path:&str)->Result<(),String>{
    if owned(pet,path).is_ok(){return Ok(());}
    let path=Path::new(path).canonicalize().map_err(|_|"A cópia do documento está indisponível.".to_string())?;
    if read_grants().lock().unwrap().contains_key(&(path,pet.into())){Ok(())}else{Err("Autoriza novamente a leitura deste anexo nesta conversa.".into())}
}
pub(crate) struct ReviewedReads{inserted:Vec<(PathBuf,String)>,token:String,keep:bool}
impl ReviewedReads{pub(crate) fn keep(mut self){self.keep=true;}}
impl Drop for ReviewedReads{fn drop(&mut self){if !self.keep{let mut grants=read_grants().lock().unwrap();for key in &self.inserted{if grants.get(key)==Some(&self.token){grants.remove(key);}}}}}
fn stage_reads(pet:&str,paths:&[&str])->Result<ReviewedReads,String>{
    let token=crate::assistant_memory::new_id();let paths=paths.iter().map(|p|Path::new(p).canonicalize().map_err(|_|"Uma cópia deixou de estar disponível.".to_string())).collect::<Result<Vec<_>,_>>()?;
    let mut grants=read_grants().lock().unwrap();grants.retain(|(p,_),_|p.exists());
    if grants.len().saturating_add(paths.iter().filter(|p|!grants.contains_key(&((*p).clone(),pet.into()))).count())>128{return Err("O limite de autorizações de documentos foi atingido. Reinicia a aplicação ou remove anexos antes de renovar.".into());}
    let mut inserted=vec![];
    for path in paths{let key=(path,pet.into());if !grants.contains_key(&key){grants.insert(key.clone(),token.clone());inserted.push(key);}}
    Ok(ReviewedReads{inserted,token,keep:false})
}
pub(crate) fn grant_reviewed_reads_at(provider:&str,paths:&[&str],config_path:&Path)->Result<ReviewedReads,String>{
    let pet=crate::pet_config::integration_pet(provider).ok_or("Pet desconhecido.")?;crate::pet_config::require_active(pet)?;
    let config=crate::pet_config::read_existing_at(config_path)?;if !config.permissions(pet)?.documents{return Err("Este pet não tem acesso a documentos. Revê as permissões antes de renovar.".into());}
    for path in paths{if !analysis_supported(provider,Path::new(path)){return Err("Um dos formatos não é suportado por este pet.".into());}}
    stage_reads(pet,paths)
}
pub(crate) fn revoke_reviewed_reads(provider:Option<&str>,paths:Option<&[String]>){
    let pet=provider.and_then(crate::pet_config::integration_pet);
    let paths=paths.map(|paths|paths.iter().flat_map(|p|{let path=PathBuf::from(p);[Some(path.clone()),path.canonicalize().ok()]}).flatten().collect::<Vec<_>>());
    read_grants().lock().unwrap().retain(|(path,owner),_|!((provider.is_none()||pet==Some(owner.as_str()))&&paths.as_ref().is_none_or(|ps|ps.contains(path))));
}
/// The retention coordinator already pins these canonical owned copies. Revoke
/// both document and native mail authority, without resolving a replaced path.
#[cfg(target_os="linux")]
pub(crate) fn revoke_copy_grants(provider:&str,paths:&[String])->Result<(),String>{
    let pet=crate::pet_config::integration_pet(provider).ok_or("Pet desconhecido.")?;
    let paths=paths.iter().map(PathBuf::from).collect::<Vec<_>>();let mut native=grants().lock().unwrap();let mut reads=read_grants().lock().unwrap();
    if native.iter().any(|(p,owner)|paths.contains(p)&&owner!=pet)||reads.keys().any(|(p,owner)|paths.contains(p)&&owner!=pet){return Err("Uma cópia tem uma autorização de outro pet. As cópias foram preservadas.".into());}
    native.retain(|p,_|!paths.contains(p));reads.retain(|(p,_),_|!paths.contains(p));Ok(())
}
fn compose(to:&str,subject:&str,body:&str,attachment:&Path)->Result<String,String>{
 if to.len()>320 || subject.len()>500 || body.len()>32000 || to.chars().any(char::is_control) || subject.chars().any(char::is_control) || (!to.is_empty() && (!to.contains('@') || to.contains(['\'',',','"',' ']))){return Err("Revê o destinatário e o assunto do rascunho.".into());}
 // GetArgs in Thunderbird/Betterbird treats single quotes as RAW literals;
 // only double-quoted/unquoted values are URI-decoded. Encode every value before
 // adding double quotes, including the file URI, so delimiters cannot become
 // fields and literal percent sequences survive exactly one decoding pass.
 let enc=|s:&str|s.bytes().map(|b|if b.is_ascii_alphanumeric() || b"-_.~".contains(&b){(b as char).to_string()}else{format!("%{b:02X}")}).collect::<String>();
 let uri=reqwest::Url::from_file_path(attachment).map_err(|_|"Caminho do anexo inválido.")?;
 Ok(format!("to=\"{}\",subject=\"{}\",body=\"{}\",attachment=\"{}\",format=text",enc(to),enc(subject),enc(body),enc(uri.as_str())))
}
#[cfg(target_os="linux")]
fn next_mail_request()->Result<u64,String>{
 use std::sync::atomic::{AtomicU64,Ordering};
 static NEXT:AtomicU64=AtomicU64::new(1);
 NEXT.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|id|id.checked_add(1)).map_err(|_|"Não foi possível identificar o pedido de rascunho.".into())
}
#[cfg(target_os="linux")]
fn draft(request:u64,provider:&str,path:&str,recipient:&str,subject:&str,body:&str)->Result<String,String>{
 use std::process::{Command,Stdio};
 let pet=crate::pet_config::integration_pet(provider).ok_or("Pet desconhecido.")?;
 crate::pet_config::require(pet,"emailDrafts")?;owned(pet,path)?;
 if !Path::new(path).is_file(){return Err("O plugin de email recebe um ficheiro de cada vez. Escolhe um documento dentro da pasta.".into());}
 let _=compose(recipient,subject,body,Path::new(path))?;
 let client=Command::new("/usr/bin/xdg-mime").args(["query","default","x-scheme-handler/mailto"]).output().map_err(|_|"Não foi possível consultar a aplicação de email.")?;
 let client=String::from_utf8_lossy(&client.stdout).trim().to_string();
 // Do not let xdg-email silently discard an attachment with a browser handler.
 let app=match client.as_str(){"eu.betterbird.Betterbird.desktop"=>"eu.betterbird.Betterbird","org.mozilla.Thunderbird.desktop"=>"org.mozilla.Thunderbird",_=>return Err("O rascunho com anexo suporta Betterbird ou Thunderbird como cliente de email predefinido. Configura um destes clientes; o documento continua no pet.".into())};
 crate::log::line(format!("mail draft {request}: client selected"));
 let export=Command::new("/usr/bin/flatpak").args(["document-export","--transient","--unique","--app",app,"--allow-read","--forbid-write","--forbid-delete","--forbid-grant-permission",path]).output().map_err(|_|"Não foi possível partilhar o anexo com a aplicação de email.")?;
 if !export.status.success(){return Err("O portal não disponibilizou o anexo ao cliente de email.".into());}
 let exported=PathBuf::from(String::from_utf8_lossy(&export.stdout).trim());
 if !exported.is_absolute() || !exported.is_file(){return Err("O portal devolveu um anexo indisponível.".into());}
 crate::log::line(format!("mail draft {request}: attachment exported"));
 let compose=compose(recipient,subject,body,&exported)?;
 // Check again after portal I/O: a revoked grant must not launch a draft.
 crate::pet_config::require(pet,"emailDrafts")?;owned(pet,path)?;
 crate::log::line(format!("mail draft {request}: launching"));
 let mut child=Command::new("/usr/bin/flatpak").args(["run",app,"-compose",&compose]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|_|{crate::log::line(format!("mail draft {request}: launch failed"));"Não foi possível pedir o rascunho à aplicação de email."})?;
 crate::log::line(format!("mail draft {request}: launcher spawned"));
 // An exit status does not prove that the remote compose window failed to open.
 // Record only static diagnostics; never retry/fallback after this handoff.
 std::thread::spawn(move||{let phase=match child.wait(){Ok(status) if status.success()=>"launcher exited successfully",Ok(_)=>"launcher exited unsuccessfully",Err(_)=>"launcher exit unknown"};crate::log::line(format!("mail draft {request}: {phase}"));});
 Ok("Pedi à aplicação de email para abrir um rascunho com o anexo. Confirma o conteúdo e o anexo na aplicação antes de enviar.".into())
}
#[tauri::command]
pub async fn pet_mail_draft(window:tauri::WebviewWindow,app:AppHandle,provider:String,path:String,recipient:String,subject:String,body:String)->Result<String,String>{
 if window.label()!=crate::island::WINDOW_LABEL{return Err("Abre o documento no cartão do pet para preparar o email.".into());}
 let _=app.app_handle();
 #[cfg(target_os="linux")]{let request=next_mail_request()?;crate::log::line(format!("mail draft {request}: received"));tauri::async_runtime::spawn_blocking(move||draft(request,&provider,&path,&recipient,&subject,&body)).await.map_err(|_|"A preparação do rascunho foi interrompida.".to_string())?}
 #[cfg(not(target_os="linux"))]{let _=(provider,path,recipient,subject,body);Err("Rascunhos com anexo ainda não estão disponíveis nesta plataforma.".into())}
}
#[cfg(test)]mod tests{use super::*;
#[test]
#[cfg(target_os="linux")]
fn copy_retention_revokes_exact_native_mail_and_read_grants_and_keeps_other_paths(){
 let root=std::env::temp_dir().join(format!("coucou-purge-grants-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&root).unwrap();let a=root.join("a.txt");let b=root.join("b.txt");std::fs::write(&a,b"abc").unwrap();std::fs::write(&b,b"def").unwrap();
 grants().lock().unwrap().extend([(a.clone(),"ai_ollama".into()),(b.clone(),"agent_codex".into())]);stage_reads("ai_ollama",&[a.to_str().unwrap()]).unwrap().keep();stage_reads("agent_codex",&[b.to_str().unwrap()]).unwrap().keep();
 revoke_copy_grants("ollama",&[a.to_string_lossy().into_owned()]).unwrap();assert!(owned("ai_ollama",a.to_str().unwrap()).is_err());assert!(read_owned("ai_ollama",a.to_str().unwrap()).is_err());assert!(owned("agent_codex",b.to_str().unwrap()).is_ok());assert!(read_owned("agent_codex",b.to_str().unwrap()).is_ok());
 revoke_copy_grants("codex",&[b.to_string_lossy().into_owned()]).unwrap();std::fs::remove_dir_all(root).unwrap();
}
#[test]
#[cfg(target_os="linux")]
fn foreign_copy_grants_block_purge_without_revoking_either_owner(){
 let path=std::env::temp_dir().join(format!("coucou-foreign-purge-grants-{}",crate::assistant_memory::new_id()));std::fs::write(&path,b"abc").unwrap();grants().lock().unwrap().insert(path.clone(),"ai_ollama".into());stage_reads("agent_codex",&[path.to_str().unwrap()]).unwrap().keep();
 assert!(revoke_copy_grants("ollama",&[path.to_string_lossy().into_owned()]).is_err());assert!(owned("ai_ollama",path.to_str().unwrap()).is_ok());assert!(read_owned("agent_codex",path.to_str().unwrap()).is_ok());
 revoke_reviewed_reads(Some("codex"),Some(&[path.to_string_lossy().into_owned()]));revoke_copy_grants("ollama",&[path.to_string_lossy().into_owned()]).unwrap();std::fs::remove_file(path).unwrap();
}
#[test]fn reviewed_read_grants_are_scoped_rollback_safe_and_do_not_authorize_mail(){
 let path=std::env::temp_dir().join(format!("coucou-reviewed-read-{}",crate::assistant_memory::new_id()));std::fs::write(&path,b"synthetic").unwrap();let text=path.to_str().unwrap();
 let staged=stage_reads("ai_ollama",&[text]).unwrap();assert!(read_owned("ai_ollama",text).is_ok());assert!(read_owned("agent_codex",text).is_err());assert!(owned("ai_ollama",text).is_err());drop(staged);assert!(read_owned("ai_ollama",text).is_err());
 stage_reads("ai_ollama",&[text]).unwrap().keep();let again=stage_reads("ai_ollama",&[text]).unwrap();drop(again);assert!(read_owned("ai_ollama",text).is_ok());assert!(owned("ai_ollama",text).is_err());
 read_grants().lock().unwrap().remove(&(path.clone(),"ai_ollama".into()));std::fs::remove_file(path).unwrap();
}
#[test]fn document_read_revocation_preserves_other_pets_with_the_same_copy(){
 let path=std::env::temp_dir().join(format!("coucou-scoped-revoke-{}",crate::assistant_memory::new_id()));std::fs::write(&path,b"one").unwrap();let text=path.to_str().unwrap();stage_reads("ai_ollama",&[text]).unwrap().keep();stage_reads("agent_codex",&[text]).unwrap().keep();
 revoke_reviewed_reads(Some("ollama"),Some(&[text.into()]));assert!(read_owned("ai_ollama",text).is_err());assert!(read_owned("agent_codex",text).is_ok());revoke_reviewed_reads(Some("codex"),Some(&[text.into()]));assert!(read_owned("agent_codex",text).is_err());std::fs::remove_file(path).unwrap();
}
#[test]fn compose_cannot_inject_recipient_fields_or_drop_attachment(){
 let text=compose("example@example.invalid","Olá, it's a test","PT & English\nattachment='evil'",Path::new("/tmp/Test document.pdf")).unwrap();
 assert!(text.contains("subject=\"Ol%C3%A1%2C%20it%27s%20a%20test\""));assert!(text.contains("attachment=\"file%3A%2F%2F%2Ftmp%2FTest%2520document.pdf\""));assert!(text.ends_with(",format=text"));
 assert!(compose("x@y,evil@example.invalid","test","",Path::new("/tmp/a.pdf")).is_err());assert!(compose("x@y\nBcc: evil@a","test","",Path::new("/tmp/a.pdf")).is_err());
 assert!(compose("","teste","",Path::new("/tmp/a.pdf")).is_ok());
}
#[test]fn compose_does_not_allow_text_or_filename_delimiters_to_change_the_native_grammar(){
 let args=compose("qa@example.invalid","a\",bcc='other@invalid',format=html","x='y',attachment='evil'\n100%",Path::new("/tmp/a',bcc='other@invalid.pdf")).unwrap();
 assert_eq!(args.split(',').count(),5);assert!(!args.contains("bcc="));assert!(!args.contains("attachment='evil'"));assert!(args.ends_with(",format=text"));
 assert!(args.contains("%27%2Cbcc%3D%27"));
}
#[test]fn compose_preserves_literal_percent_sequences_and_rejects_header_controls(){
 let args=compose("qa@example.invalid","literal %20 and á","%C3%A1\nPT & EN",Path::new("/tmp/100% literal.pdf")).unwrap();
 assert!(args.contains("subject=\"literal%20%2520%20and%20%C3%A1\""));assert!(args.contains("body=\"%25C3%25A1%0APT%20%26%20EN\""));
 assert!(compose("qa@example.invalid","line\nbcc: other@invalid","",Path::new("/tmp/a.pdf")).is_err());
 assert!(compose("qa@example.invalid","ok","",Path::new("relative.pdf")).is_err());
}
#[test]fn compose_native_client_interoperability_vectors(){
 let cases=[
  ("qa@example.invalid","Mochi QA · Olá, it's a test","PT/EN & attachment='literal'.\nLinha 2.","/tmp/Test document.pdf"),
  ("qa@example.invalid","100% %20 %C3%A1","literal %00 and \"quotes\", = signs","/tmp/100% literal.pdf"),
  ("qa@example.invalid","a\",bcc='other@invalid'","x',attachment='evil'","/tmp/a',bcc='other@invalid.pdf"),
  ("","Sem destinatário","","/tmp/Olá, report.pdf"),
  ("qa@example.invalid","中文 · café","行 1\nlinha 2\r\nline 3","/tmp/中文 'document'.pdf"),
 ];
 for(to,subject,body,path)in cases {
  let encoded=compose(to,subject,body,Path::new(path)).unwrap();assert_eq!(encoded.split(',').count(),5);
  println!("MAIL_COMPOSE_VECTOR {}",serde_json::json!({"to":to,"subject":subject,"body":body,"attachment":reqwest::Url::from_file_path(path).unwrap().as_str(),"args":encoded}));
 }
}
}
