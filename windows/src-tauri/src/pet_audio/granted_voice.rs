//! NativeUser spoken commands over the SAME output broker and captured grant.
//! No provider/model dispatch or authority is inferred from spoken/generated text.
use super::{commands::SoundCommand,Snapshot,Receipt,Request,Action,Kind};
use crate::pet_config::system_access::{Captured,Operation};
use std::{future::Future,sync::Arc};
pub(crate)type Current=Arc<dyn Fn()->bool+Send+Sync>;
const ENDED:&str="O pedido de voz ou a concessão original terminou. Uma ação já enviada não deve ser repetida automaticamente.";
fn operation(command:SoundCommand)->Result<Operation,String>{
 match command{
  SoundCommand::Status=>Ok(Operation::Observe),
  SoundCommand::Volume{input:false,percent}if percent.is_finite()&&(0.0..=100.0).contains(&percent)=>Ok(Operation::Volume),
  SoundCommand::Mute{input:false,..}=>Ok(Operation::Mute),
  _=>Err("Esta concessão de Som inclui apenas saídas. O microfone e outras componentes precisam de concessões próprias.".into()),
 }
}
#[cfg(target_os="linux")]
pub(crate)async fn spoken(scope:Captured,command:SoundCommand,current:Current)->Result<(String,bool),String>{
 execute(scope,command,current,super::linux::pet_outputs_snapshot,super::linux::pet_outputs_control).await
}
#[cfg(not(target_os="linux"))]
pub(crate)async fn spoken(_scope:Captured,_command:SoundCommand,_current:Current)->Result<(String,bool),String>{Err("O broker de Som está disponível em Linux.".into())}

async fn execute<R,C,RF,CF>(scope:Captured,command:SoundCommand,current:Current,read:R,control:C)->Result<(String,bool),String>
where R:FnOnce(Current)->RF,C:FnOnce(Request,Current)->CF,RF:Future<Output=Snapshot>,CF:Future<Output=Result<Receipt,String>>{
 let operation=operation(command)?;
 if !current(){return Err(ENDED.into());}
 scope.check()?;
 if !scope.has_operation(operation){return Err("Este pet não tem concessão própria para essa operação de Som.".into());}
 let scope=Arc::new(scope);let original=scope.clone();let caller=current.clone();
 let check:Current=Arc::new(move||caller()&&original.check().is_ok());
 let snapshot=read(check.clone()).await;
 if !check(){return Err(ENDED.into());}
 if !snapshot.available{return Err(snapshot.error.unwrap_or_else(||"O serviço de som está indisponível.".into()));}
 // Resolve the actual default once, then pin its physical key. Never choose a
 // different granted device when the native default itself is not authorized.
 let mut defaults=snapshot.outputs.iter().filter(|n|n.kind==Kind::Output&&n.is_default);
 let output=defaults.next().ok_or("Não há uma saída de Som predefinida.")?;
 if defaults.next().is_some(){return Err("A saída predefinida é ambígua. Atualiza o estado antes de repetir.".into());}
 let pinned=Arc::new(scope.pin_output(operation,&output.key)?);
 if command==SoundCommand::Status{
  if !current()||pinned.check().is_err(){return Err(ENDED.into());}
  let volume=output.volume.map(|v|format!("{v:.0} por cento")).unwrap_or_else(||"volume desconhecido".into());
  return Ok((format!("Saída {}, {}{}.",output.label.chars().take(256).collect::<String>(),volume,if output.muted==Some(true){", sem som"}else{""}),false));
 }
 let request=match command{
  SoundCommand::Volume{percent,..}=>Request{target_key:output.key.clone(),action:Action::Volume,value:Some(percent),muted:None,destination_key:None,option:None},
  SoundCommand::Mute{muted,..}=>Request{target_key:output.key.clone(),action:Action::Mute,value:None,muted:Some(muted),destination_key:None,option:None},
  SoundCommand::Status=>unreachable!(),
 };
 let original=pinned.clone();let caller=current.clone();let check:Current=Arc::new(move||caller()&&original.check().is_ok());
 if !check(){return Err(ENDED.into());}
 let result=control(request,check.clone()).await?;
 if !check(){return Ok((ENDED.into(),false));}
 if !result.confirmed||result.warning.is_some(){return Ok((result.warning.unwrap_or_else(||"O serviço ainda não confirmou a alteração. Não repitas automaticamente.".into()),false));}
 let text=match command{SoundCommand::Volume{percent,..}=>format!("Volume do sistema definido para {percent:.0} por cento."),SoundCommand::Mute{muted,..}=>format!("Som do sistema {}.",if muted{"silenciado"}else{"reativado"}),SoundCommand::Status=>unreachable!()};
 Ok((text,true))
}
#[cfg(test)]#[path="granted_voice_tests.rs"]mod tests;
