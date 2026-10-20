//! Explicit dropped-file handoff. Integration grants do not replace native CLI policies.
use std::{collections::BTreeMap,path::{Path,PathBuf},sync::{Mutex,OnceLock}};
use tauri::{AppHandle,Manager};
fn grants()->&'static Mutex<BTreeMap<PathBuf,String>>{static GRANTS:OnceLock<Mutex<BTreeMap<PathBuf,String>>>=OnceLock::new();GRANTS.get_or_init(||Mutex::new(BTreeMap::new()))}
fn local_unsupported(provider:&str,path:&Path)->bool{
 ["ollama","lmstudio"].contains(&provider) && path.is_file() && path.extension().and_then(|e|e.to_str()).is_some_and(|e|["png","jpg","jpeg","gif","webp","mp3","mp4","wav","ogg","zip","doc","docx","xls","xlsx","ppt","pptx","exe","bin"].contains(&e.to_ascii_lowercase().as_str()))
}
pub fn ingest(provider:&str,source:&str)->Result<crate::files::DroppedFile,String>{
 crate::pet_config::require_active_provider(provider)?;
 let pet=crate::pet_config::integration_pet(provider).ok_or("Este pet não recebe documentos.")?;
 let config=crate::pet_config::read()?;let p=config.permissions(pet)?;
 let folder=std::fs::symlink_metadata(source).is_ok_and(|m|m.is_dir());
 let analysis=p.documents && !local_unsupported(provider,Path::new(source)) && (!folder || crate::cli_chat::backend(provider).is_some() || ["ollama","lmstudio"].contains(&provider));
 if !analysis && (!p.email_drafts || folder){return Err(format!("{} não tem nenhuma funcionalidade ativa para este {}. Revê os acessos deste pet.",config.pets[pet].name,if folder{"tipo de pasta"}else{"documento"}));}
 let file=if folder{crate::document_folder::ingest(source)?}else{crate::files::ingest(source)?};
 let mut g=grants().lock().unwrap();g.retain(|path,_|path.exists());
 g.insert(PathBuf::from(&file.path),pet.into());Ok(file)
}
pub fn check(provider:&str,path:&str)->Result<(),String>{
 let pet=crate::pet_config::integration_pet(provider).ok_or("Este pet não recebe documentos.")?;
 crate::pet_config::require(pet,"documents")?;
 if local_unsupported(provider,Path::new(path)){return Err("Este formato ainda não é analisado pelo chat local. Entrega texto ou um PDF com texto, ou escolhe um pet com ferramentas de documentos.".into());}
 owned(pet,path)
}
fn owned(pet:&str,path:&str)->Result<(),String>{
 let path=Path::new(path).canonicalize().map_err(|_|"O documento já não está disponível. Volta a arrastá-lo para o pet.")?;
 if grants().lock().unwrap().get(&path).is_some_and(|owner|owner==pet){Ok(())}else{Err("Este documento não foi entregue a este pet. Arrasta-o para o pet que o deve receber.".into())}
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
#[cfg(test)]mod tests{use super::*;#[test]fn compose_cannot_inject_recipient_fields_or_drop_attachment(){
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
