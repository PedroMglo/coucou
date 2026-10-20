use std::{path::{Path,PathBuf},time::{Duration,Instant},sync::atomic::Ordering};
use super::types::*;
use crate::pet_config::LocalExecution;

#[cfg(target_os="linux")]
pub(crate) fn stamp(path:&Path)->Result<ExecutableStamp,String>{
 use std::os::unix::fs::MetadataExt;
 let m=std::fs::metadata(path).map_err(|_|"O programa já não está disponível.")?;
 if !m.is_file()||m.mode()&0o111==0{return Err("O alvo não é um programa executável.".into());}
 Ok(ExecutableStamp{dev:m.dev(),ino:m.ino(),len:m.len(),modified:i128::from(m.mtime())*1_000_000_000+i128::from(m.mtime_nsec()),mode:m.mode()})
}
#[cfg(not(target_os="linux"))]
pub(crate) fn stamp(_: &Path)->Result<ExecutableStamp,String>{Err("A execução local só está disponível em Linux.".into())}

pub fn freeze(input:CommandInput,cwd:&Path,policy:&LocalExecution)->Result<CommandSpec,String>{
 if input.program.trim().is_empty()||input.program.len()>4096||input.program.chars().any(char::is_control)||input.args.len()>128||input.args.iter().any(|a|a.contains('\0'))||input.reason.trim().is_empty()||input.reason.len()>2048{return Err("A proposta de comando é inválida.".into());}
 if input.timeout_ms.is_some_and(|n|!(100..=60_000).contains(&n)){return Err("Prazo de comando inválido.".into());}
 let requested=PathBuf::from(&input.program);
 let program=if requested.is_absolute(){requested}else{
  if requested.components().count()!=1{return Err("Usa um programa do PATH ou um caminho absoluto.".into());}
  std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).filter(|p|p.is_absolute()).map(|p|p.join(&requested)).find(|p|stamp(p).is_ok()).ok_or("O programa não existe no PATH.")?
 };
 let executable=program.canonicalize().map_err(|_|"O programa não está disponível.")?;
 if let Some(allowed)=&policy.allowed_programs{if !allowed.iter().any(|p|Path::new(p).canonicalize().is_ok_and(|p|p==executable)){return Err("O programa não está na lista permitida para este pet.".into());}}
 let executable_stamp=stamp(&executable)?;
 let directory=std::fs::File::open(cwd).map_err(|_|"Não foi possível fixar a pasta de execução.")?;
 if !directory.metadata().map_err(|e|e.to_string())?.is_dir(){return Err("A pasta de execução já não está disponível.".into());}
 let spec=CommandSpec{executable,args:input.args,cwd:cwd.into(),reason:input.reason,timeout_ms:input.timeout_ms.unwrap_or(policy.command_timeout_ms).min(policy.command_timeout_ms),directory,executable_stamp};
 if serde_json::to_vec_pretty(&spec.display()).map_err(|e|e.to_string())?.len()>MAX_PROPOSAL{return Err("O comando completo excede o tamanho permitido para aprovação.".into());}Ok(spec)
}

pub enum Outcome {Allowed(ApprovedCommand),Denied,Expired,Cancelled}
pub fn policy_current(scope:&ExecutionScope)->bool{crate::pet_config::read().is_ok_and(|c|c.local_execution()==scope.policy)}
pub async fn request(app:&tauri::AppHandle,scope:&ExecutionScope,step:&StepReceipt,spec:CommandSpec)->Result<Outcome,String>{
 if scope.turn.cancel.load(Ordering::Acquire){return Ok(Outcome::Cancelled);}
 if !policy_current(scope){return Err("As permissões de execução foram alteradas; o comando não foi iniciado.".into());}
 let deadline=(Instant::now()+Duration::from_millis(scope.policy.approval_timeout_ms)).min(scope.deadline);
 if Instant::now()>=deadline{return Ok(Outcome::Expired);}
 let params=spec.display();let app=app.clone();let cancel=scope.turn.cancel.clone();let origin=serde_json::json!({"kind":"mochi","provider":"ollama","petId":"ai_ollama","sessionId":scope.target.session_id,"conversationId":scope.target.conversation_id,"label":format!("Mochi · Ollama · #{}",scope.target.session_id.chars().take(8).collect::<String>())});
 let answer=tauri::async_runtime::spawn_blocking(move||crate::cli_chat::ask_local_once(&app,&params,cancel,origin,deadline)).await.map_err(|_|"Não foi possível apresentar a aprovação.")?;
 if scope.turn.cancel.load(Ordering::Acquire){return Ok(Outcome::Cancelled);}
 if Instant::now()>=deadline{return Ok(Outcome::Expired);}
 if !policy_current(scope){return Err("As permissões de execução foram alteradas; o comando não foi iniciado.".into());}
 if answer.as_ref().is_some_and(|a|a["allow"]==true){Ok(Outcome::Allowed(ApprovedCommand::new(spec,&scope.task_id,&step.step_id)))}else{Ok(Outcome::Denied)}
}

#[cfg(all(test,target_os="linux"))]mod tests{
 use super::*;
 #[test]fn exact_spec_is_resolved_and_bounded_before_approval(){let p=LocalExecution::default();let input=CommandInput{program:"/bin/true".into(),args:vec!["literal; $HOME".into()],reason:"synthetic".into(),timeout_ms:Some(1000)};let s=freeze(input,&std::env::temp_dir(),&p).unwrap();assert!(s.executable.is_absolute());assert_eq!(s.args,["literal; $HOME"]);assert_eq!(s.display()["scope"],"once");}
 #[test]fn allowlist_and_proposal_size_fail_closed(){let mut p=LocalExecution::default();p.allowed_programs=Some(vec![]);let input=CommandInput{program:"/bin/true".into(),args:vec![],reason:"synthetic".into(),timeout_ms:None};assert!(freeze(input.clone(),&std::env::temp_dir(),&p).is_err());p.allowed_programs=None;let mut i=input;i.args.push("x".repeat(MAX_PROPOSAL));assert!(freeze(i,&std::env::temp_dir(),&p).is_err());}
}
